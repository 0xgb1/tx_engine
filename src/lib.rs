use std::collections::HashMap;

#[derive(Debug)]
struct Client {
    id: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}
#[derive(Debug)]
pub struct Tx {
    type_: String,
    client_id: u16,
    id: u32,
    amount: Option<f64>,
    disputed: bool,
}

#[derive(Default)]
pub struct State {
    client_store: HashMap<u16, Client>,
    tx_store: HashMap<u32, Tx>,
}

impl State {
    pub fn show(&self) {
        for client in self.client_store.values() {
            println!("{:?}", client);
        }
    }

    pub fn handle_tx(&mut self, tx: Tx) {
        match tx.type_.as_str() {
            "deposit" => self.deposit(tx),
            "withdrawal" => self.withdrawal(tx),
            "dispute" => self.dispute(tx),
            "resolve" => self.resolve(tx),
            "chargeback" => self.chargeback(tx),
            _ => todo!("add to log, invalid transaction type"),
        };
    }

    fn dispute_checks(&self, tx_ref: &Tx, tx_type: &str) -> Option<f64> {
        match tx_type {
            // handles case where deposit gets disputed in bad input
            "dispute" if self.tx_store.get(&tx_ref.id).unwrap().type_ == "deposit" => {
                todo!("add to log, dispute's tx id references a deposit");
            }
            "resolve" | "chargeback" if !self.tx_store.get(&tx_ref.id).unwrap().disputed => {
                todo!(
                    "add to log, {} failed because transaction is not disputed",
                    tx_type
                );
            }
            _ => {
                // sanity check for case where tx exists but client doesn't (shouldn't happen)
                if !self.client_store.contains_key(&tx_ref.client_id) {
                    todo!("add to log, client doesn't exist but tx does");
                }
                if self.tx_store.get(&tx_ref.id).unwrap().client_id != tx_ref.client_id {
                    todo!(
                        "add to log, {}'s client ID does not match the tx's client id",
                        tx_type
                    );
                }
            }
        }
        // if every case isn't matched and every condition not met (successful tx)
        let amt = Some(self.tx_store.get(&tx_ref.id).unwrap().amount.unwrap());
        amt
    }

    fn deposit(&mut self, tx: Tx) {
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
                held: 0.0,
                total: dep_amt,
                locked: false,
            });
        self.tx_store.insert(tx.id, tx);
    }

    fn withdrawal(&mut self, tx: Tx) {
        if self.client_store.contains_key(&tx.client_id) {
            let withd_amt = tx.amount.unwrap();
            self.client_store.entry(tx.client_id).and_modify(|acct| {
                if withd_amt <= acct.available {
                    acct.available -= withd_amt;
                    acct.total -= withd_amt
                } else {
                    todo!("add to log, withdrawal exceeded client funds)");
                }
            });
            self.tx_store.insert(tx.id, tx);
        } else {
            todo!("add to log, client doesn't exist");
        }
    }

    fn dispute(&mut self, tx: Tx) {
        if self.tx_store.contains_key(&tx.id) {
            let amt: Option<f64> = self.dispute_checks(&tx, tx.type_.as_str());

            if amt.is_none() {
                return;
            }

            let disp_amt = amt.unwrap();
            self.client_store.entry(tx.client_id).and_modify(|acct| {
                acct.available -= disp_amt;
                acct.held += disp_amt;
            });
            self.tx_store
                .entry(tx.id)
                .and_modify(|tx| tx.disputed = true);
        } else {
            todo!("add to log, dispute failed because transaction doesn't exist");
        }
    }

    fn resolve(&mut self, tx: Tx) {
        if self.tx_store.contains_key(&tx.id) {
            let amt: Option<f64> = self.dispute_checks(&tx, tx.type_.as_str());

            if amt.is_none() {
                return;
            }

            let disp_amt = amt.unwrap();
            self.client_store.entry(tx.client_id).and_modify(|acct| {
                acct.available += disp_amt;
                acct.held -= disp_amt;
            });
            self.tx_store
                .entry(tx.id)
                .and_modify(|tx| tx.disputed = false);
        } else {
            todo!("add to log, resolve failed because transaction doesn't exist");
        }
    }

    fn chargeback(&mut self, tx: Tx) {
        if self.tx_store.contains_key(&tx.id) {
            let amt: Option<f64> = self.dispute_checks(&tx, tx.type_.as_str());

            if amt.is_none() {
                return;
            }

            let disp_amt = amt.unwrap();

            self.client_store.entry(tx.client_id).and_modify(|acct| {
                acct.total -= disp_amt;
                acct.held -= disp_amt;
            });
            self.client_store
                .entry(tx.client_id)
                .and_modify(|acct| acct.locked = true);
        } else {
            todo!("add to log, chargeback failed because transaction id doesn't exist");
        }
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
                            v => Some(v.parse::<f64>()?),
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
            _ => Err("Invalid transaction format".into()),
        }
    }
}
