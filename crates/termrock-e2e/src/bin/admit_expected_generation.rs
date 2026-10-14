use std::collections::BTreeMap;
use std::path::PathBuf;

use termrock_e2e::admit_expected_generation;

fn main() {
    if let Err(error) = run() {
        eprintln!("admit_expected_generation: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut values = BTreeMap::new();
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        if !flag.starts_with("--") || values.insert(flag.clone(), value).is_some() {
            return Err(format!("invalid or repeated argument {flag:?}"));
        }
    }
    let required = [
        "--generation-id",
        "--generation-root",
        "--generation-sha256",
        "--capture-receipt",
        "--capture-sha256",
        "--qualification-review",
        "--qualification-review-sha256",
        "--write-root",
        "--output",
    ];
    if values.len() != required.len() || required.iter().any(|key| !values.contains_key(*key)) {
        return Err(format!("required arguments are: {}", required.join(" ")));
    }
    let get = |key: &str| {
        values
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("missing {key}"))
    };
    let receipt = admit_expected_generation(
        get("--generation-id")?,
        &PathBuf::from(get("--generation-root")?),
        get("--generation-sha256")?,
        &PathBuf::from(get("--capture-receipt")?),
        get("--capture-sha256")?,
        &PathBuf::from(get("--qualification-review")?),
        get("--qualification-review-sha256")?,
        &PathBuf::from(get("--write-root")?),
        &PathBuf::from(get("--output")?),
    )?;
    let json = serde_json::to_string(&receipt)
        .map_err(|error| format!("serialize admission result: {error}"))?;
    println!("{json}");
    Ok(())
}
