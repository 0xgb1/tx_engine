#[derive(Debug)]
pub struct Tx {
    type_: String,
    client_id: u16,
    id: u32,
    amount: Option<f64>,
    disputed: bool,
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
