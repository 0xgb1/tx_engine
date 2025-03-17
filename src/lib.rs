use convert_case::{Case, Casing};
use rust_decimal::{
    Decimal,
    prelude::{FromPrimitive, ToPrimitive},
};
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
struct Client {
    id: u16,
    available: u64,
    held: u64,
    total: u64,
    locked: bool,
}
#[derive(Debug)]
pub struct Tx {
    type_: String,
    client_id: u16,
    id: u32,
    amount: Option<u64>,
    disputed: bool,
}

#[derive(Default)]
pub struct State {
    client_store: HashMap<u16, Client>,
    tx_store: HashMap<u32, Tx>,
    dispute_store: HashMap<u32, u64>,
}

impl State {
    pub fn show(&self) {
        println!("client, available, held, total, locked");
        for client in self.client_store.values() {
            println!("{}", client);
        }
    }

    fn amt_u64_parse(val: u64) -> String {
        // converts the u64 values back to decimal values with four places and gets
        // them to display nicely
        let val = Decimal::from(val) / Decimal::from(10000);
        let val_str: String = if val.fract().is_zero() {
            format!("{:.1}", val)
        } else {
            val.normalize().to_string()
        };
        val_str
    }

    pub fn handle_tx(&mut self, tx: Tx) {
        match tx.type_.as_str() {
            "deposit" => self.deposit(tx),
            "withdrawal" => self.withdrawal(tx),
            "dispute" => self.dispute(tx),
            "resolve" => self.resolve(tx),
            "chargeback" => self.chargeback(tx),
            _ => warn!("Invalid transaction type. Transaction: {}", tx.log_fmt()),
        };
    }

    fn dispute_checks(&self, tx_ref: &Tx, tx_type: &str) -> Option<u64> {
        if !self.tx_store.contains_key(&tx_ref.id) {
            warn!(
                "Transaction ID not found in tx store. Transaction: {}",
                tx_ref.log_fmt()
            );
            return None;
        }
        let tx = self.tx_store.get(&tx_ref.id).unwrap();
        match tx_type {
            // handles case where deposit gets disputed in bad input
            "dispute" if tx.type_ == "deposit" => {
                warn!(
                    "Dispute's tx id references a deposit. Transaction: {}",
                    tx_ref.log_fmt()
                );
                return None;
            }
            "resolve" | "chargeback" if !tx.disputed => {
                warn!(
                    "{} failed because transaction is not disputed. Transaction: {}",
                    tx_type.to_case(Case::Sentence),
                    tx_ref.log_fmt()
                );
                return None;
            }
            _ => {
                // sanity check for case where tx exists but client doesn't (shouldn't happen)
                if !self.client_store.contains_key(&tx_ref.client_id) {
                    warn!(
                        "Client ID not found while tx ID is. Transaction: {}",
                        tx_ref.log_fmt()
                    );
                    return None;
                }
                if tx.client_id != tx_ref.client_id {
                    warn!(
                        "{}'s client ID does not match the tx's client ID. Transaction: {}",
                        tx_type.to_case(Case::Sentence),
                        tx_ref.log_fmt()
                    );
                    return None;
                }
            }
        }
        // if every case isn't matched and every condition not met (successful tx)
        tx.amount
    }

    fn deposit(&mut self, tx: Tx) {
        if self.client_store.contains_key(&tx.client_id)
            && self.client_store.get(&tx.client_id).unwrap().locked
        {
            info!(
                "Client account is locked; deposit failed. Transaction: {}",
                tx.log_fmt()
            );
            return;
        }
        let dep_amt = tx.amount.unwrap();
        self.client_store
            .entry(tx.client_id)
            .and_modify(|acct| {
                acct.available += dep_amt;
                acct.total += dep_amt
            })
            .or_insert(Client {
                id: tx.client_id,
                available: dep_amt,
                held: 0,
                total: dep_amt,
                locked: false,
            });
        info!("Deposit succeeded: {}", tx.log_fmt());
        self.tx_store.insert(tx.id, tx);
    }

    fn withdrawal(&mut self, tx: Tx) {
        if self.client_store.contains_key(&tx.client_id) {
            if self.client_store.get(&tx.client_id).unwrap().locked {
                info!(
                    "Client account is locked; deposit failed. Transaction: {}",
                    tx.log_fmt()
                );
                return;
            }

            let withd_amt = tx.amount.unwrap();
            if withd_amt <= self.client_store.get(&tx.client_id).unwrap().available {
                self.client_store.entry(tx.client_id).and_modify(|acct| {
                    acct.available -= withd_amt;
                    acct.total -= withd_amt;
                });
            } else {
                warn!(
                    "Withdrawal exceeded client funds. Transaction: {}",
                    tx.log_fmt()
                );
                return;
            }
            debug!("Withdrawal succeeded: {}", tx.log_fmt());
            self.tx_store.insert(tx.id, tx);
        } else {
            warn!(
                "Client ID not found in client store. Transaction: {}",
                tx.log_fmt()
            );
        }
    }

    fn dispute(&mut self, tx: Tx) {
        let amt: Option<u64> = self.dispute_checks(&tx, tx.type_.as_str());

        // if amt.is_none(), then the tx didn't pass the checks
        if amt.is_none() {
            return;
        } else if amt.unwrap() == 0 {
            debug!(
                "Dispute's transaction is of zero value. Transaction: {}",
                tx.log_fmt()
            );
            return;
        }

        if self.client_store.get(&tx.client_id).unwrap().locked {
            error!(
                // not allowing disputes if consumer is disputing
                "Client account is locked; dispute failed. Transaction: {}",
                tx.log_fmt()
            );
            return;
        }

        let disp_amt = amt.unwrap();

        self.client_store.entry(tx.client_id).and_modify(|acct| {
            acct.total += disp_amt;
            acct.held += disp_amt;
        });

        self.dispute_store.insert(tx.id, disp_amt);
        self.tx_store
            .entry(tx.id)
            .and_modify(|tx| tx.disputed = true);
        debug!(
            "Dispute succeeded for amount {}: {}",
            Decimal::from_u64(disp_amt).unwrap() / Decimal::from(10000),
            self.tx_store.get(&tx.id).unwrap().log_fmt()
        )
    }

    fn resolve(&mut self, tx: Tx) {
        let amt: Option<u64> = self.dispute_checks(&tx, tx.type_.as_str());

        // if amt.is_none(), then the tx didn't pass the checks
        if amt.is_none() {
            return;
        }

        if self.client_store.get(&tx.client_id).unwrap().locked {
            error!(
                "Client account is locked; resolve failed. Transaction: {}",
                tx.log_fmt()
            );
            return;
        }
        // if the resolve has proceeded this far, it should be for a recorded
        // dispute, so this time it pulls the amount from dispute_store
        let disp_amt = self.dispute_store.get(&tx.id).unwrap();
        self.client_store.entry(tx.client_id).and_modify(|acct| {
            acct.total -= disp_amt;
            acct.held -= disp_amt;
        });
        self.tx_store
            .entry(tx.id)
            .and_modify(|tx| tx.disputed = false);
        debug!(
            "Resolve succeeded for amount {}: {}",
            Decimal::from_u64(*disp_amt).unwrap() / Decimal::from(10000),
            tx.log_fmt()
        )
    }

    fn chargeback(&mut self, tx: Tx) {
        let amt: Option<u64> = self.dispute_checks(&tx, tx.type_.as_str());

        // if amt.is_none(), then the tx didn't pass the checks
        if amt.is_none() {
            return;
        }

        if self.client_store.get(&tx.client_id).unwrap().locked {
            info!(
                "Client account is locked; chargeback failed. Transaction: {}",
                tx.log_fmt()
            );
            return;
        }
        // if the chargeback has proceeded this far, it should be for a recorded
        // dispute, so this time it pulls the amount from dispute_store
        let disp_amt = self.dispute_store.get(&tx.id).unwrap();
        self.client_store.entry(tx.client_id).and_modify(|acct| {
            acct.available += disp_amt;
            acct.held -= disp_amt;
        });
        self.client_store
            .entry(tx.client_id)
            .and_modify(|acct| acct.locked = true);
        debug!(
            "Chargeback succeeded for amount {}: {}",
            Decimal::from_u64(*disp_amt).unwrap() / Decimal::from(10000),
            tx.log_fmt()
        )
    }
}

impl TryFrom<csv::StringRecord> for Tx {
    type Error = Box<dyn std::error::Error>;

    fn try_from(record: csv::StringRecord) -> Result<Self, Self::Error> {
        let rlen = record.len();
        match rlen {
            3 | 4 => {
                let type_ = record.get(0).unwrap().trim().to_string();
                let client_id = record.get(1).unwrap().trim().parse::<u16>()?;
                let id = record.get(2).unwrap().trim().parse::<u32>()?;
                let disputed = false;
                if rlen == 4 {
                    Ok(Tx {
                        type_,
                        client_id,
                        id,
                        amount: match record.get(3).unwrap().trim() {
                            "None" => None,
                            v => {
                                let value =
                                    (Decimal::from_str_exact(v)? * Decimal::from(10000)).to_u64();
                                if value.is_none() {
                                    return Err(format!("Invalid number amount: {}", v).into());
                                } else {
                                    value
                                }
                            }
                        },
                        disputed,
                    })
                } else {
                    Ok(Tx {
                        type_,
                        client_id,
                        id,
                        amount: None,
                        disputed,
                    })
                }
            }
            _ => Err(format!("Invalid transaction format: {:?}", record).into()),
        }
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let available = State::amt_u64_parse(self.available);
        let held = State::amt_u64_parse(self.held);
        let total = State::amt_u64_parse(self.total);
        write!(
            f,
            "{}, {}, {}, {}, {}",
            self.id, available, held, total, self.locked
        )
    }
}

impl Tx {
    fn log_fmt(&self) -> String {
        let amount = if let Some(amt) = self.amount {
            State::amt_u64_parse(amt)
        } else {
            "None".to_string()
        };
        format!(
            "Tx {{ type_: \"{}\", client_id: {}, id: {}, amount: {}, disputed: {} }}",
            self.type_, self.client_id, self.id, amount, self.disputed
        )
    }
}
