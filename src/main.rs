use csv::ReaderBuilder;
use toml_edit::{DocumentMut, value};
use tx_engine::{State, Tx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // updating of the tracing config is to ensure that the log path
    // specified in it is valid at compile-time
    let tracing_config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tracing.toml");

    let toml_str = std::fs::read_to_string(tracing_config_path).expect("Failed to read TOML file");
    let mut toml_cfg: DocumentMut = toml_str.parse().unwrap();

    toml_cfg["writer"]["log-file"]["directory_path"] = value(env!("CARGO_MANIFEST_DIR"));
    let updated_toml = toml_cfg.to_string();
    std::fs::write(tracing_config_path, updated_toml).expect("Failed to write TOML file");

    tracing_config::init! {
        path: std::path::Path::new(tracing_config_path)
    };

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
