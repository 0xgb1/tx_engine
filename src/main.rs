use csv::ReaderBuilder;
use tx_engine::{State, Tx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = State::default();
    let input_path = match std::env::args_os().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("Usage error: no path specified. Please specify an input path.");
            std::process::exit(1);
        }
    };

    let rdr = ReaderBuilder::new().flexible(true).from_path(input_path)?;
    for result in rdr.into_records() {
        let line = result?;

        let tx = Tx::try_from(line)?;
        ledger.handle_tx(tx);
    }
    ledger.show();
    Ok(())
}
