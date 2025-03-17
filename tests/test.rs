use assert_cmd::prelude::*;
use std::path::Path;
use std::process::Command;

fn clean_up_string(mut res: Vec<&str>) -> String {
    res.swap_remove(0);
    res.sort_by_key(|s| s.split(",").next().unwrap());
    for s in &mut res {
        *s = s.trim_start();
    }
    res.join("")
}

#[test]
fn test_deposit_withdrawal() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tx_engine")?;
    let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("deposit_withdrawal.csv")
        .into_os_string()
        .into_string()
        .unwrap();
    cmd.arg(test_path);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let ans = "1, 3.0, 0.0, 3.0, false\
    2, 2397.0, 0.0, 2397.0, false\
    3, 45.0, 0.0, 45.0, false\
    4, 3222.0, 0.0, 3222.0, false\
    5, 23213.0, 0.0, 23213.0, false";

    let v: Vec<&str> = stdout.split("\n").collect();
    let s = clean_up_string(v.clone());

    Ok(assert_eq!(s, ans))
}

#[test]
fn test_dispute_resolve() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tx_engine")?;
    let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("dispute_resolve.csv")
        .into_os_string()
        .into_string()
        .unwrap();
    cmd.arg(test_path);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let ans = "1, 2443.629, 0.0, 2443.629, false\
    2, 0.0, 0.0, 0.0, true\
    3, 100.0, 0.0, 100.0, false";

    let v: Vec<&str> = stdout.split("\n").collect();
    let s = clean_up_string(v.clone());

    Ok(assert_eq!(s, ans))
}

#[test]
fn test_chargeback() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tx_engine")?;
    let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("chargeback.csv")
        .into_os_string()
        .into_string()
        .unwrap();
    cmd.arg(test_path);
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let ans = "1, 4832.1223, 0.0, 4832.1223, true\
    2, 8750.2, 0.0, 8750.2, false\
    3, 0.0, 0.0, 0.0, true";

    let v: Vec<&str> = stdout.split("\n").collect();
    let s = clean_up_string(v.clone());

    Ok(assert_eq!(s, ans))
}
