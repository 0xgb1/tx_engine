use csv::ReaderBuilder;
use std::path::Path;
use toml_edit::{DocumentMut, value};
use tracing::error;
use tx_engine::{State, Tx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // updating of the tracing config is to ensure that the log path
    // specified in it is valid at compile-time
    let tracing_config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tracing.toml");

    let toml_str = std::fs::read_to_string(&tracing_config_path).expect("Failed to read TOML file");
    let mut toml_cfg: DocumentMut = toml_str.parse().unwrap();

    toml_cfg["writer"]["log-file"]["directory_path"] = value(env!("CARGO_MANIFEST_DIR"));
    let updated_toml = toml_cfg.to_string();
    std::fs::write(&tracing_config_path, updated_toml).expect("Failed to write TOML file");

    tracing_config::init! {
        path: std::path::Path::new(&tracing_config_path)
    };

    let input_path = match std::env::args_os().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("Usage error: no target file path specified. Please specify an input path.");
            std::process::exit(1);
        }
    };

    let rdr = ReaderBuilder::new().flexible(true).from_path(input_path)?;
    let mut rdr_iter = rdr.into_records().peekable();

    let first;

    if let Some(entry) = rdr_iter.peek() {
        first = entry;
    } else {
        eprintln!("Error: Input file appears to be empty. Please supply non-empty file.");
        std::process::exit(1);
    }

    let _test = match first {
        Ok(f) => f,
        Err(err) => {
            eprintln!(
                "Error: Input file is not a valid CSV file. Please validate \
            the file. \n{}",
                err
            );
            std::process::exit(1);
        }
    };

    let mut ledger = State::default();
    for result in rdr_iter {
        let line = result?;

        match Tx::try_from(line) {
            Ok(tx) => ledger.handle_tx(tx),
            Err(err) => error!("Unable to create transaction from csv line record. {}", err),
        };
    }
    ledger.show();
    Ok(())
}
