//! `install` copies the accepted harness executable from a receipt.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::authority::{load_authority, load_receipt, Authority};
use crate::json_util::sha256_bytes;

pub enum InstallOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

pub fn run_install(receipt_path: &Path, destination: &Path) -> InstallOutcome {
    let authority = match load_authority() {
        Ok(value) => value,
        Err(error) => return InstallOutcome::Failed(error),
    };
    match install_from_receipt(receipt_path, destination, &authority) {
        Ok(()) => InstallOutcome::Passed,
        Err(category) => InstallOutcome::Rejected(category),
    }
}

fn install_from_receipt(
    receipt_path: &Path,
    destination: &Path,
    authority: &Authority,
) -> Result<(), &'static str> {
    let receipt = load_receipt(receipt_path, authority).map_err(|_| "receipt")?;
    let executable_path = receipt.executable.as_ref().ok_or("receipt")?;
    let expected_sha256 = receipt
        .executable_sha256
        .as_deref()
        .ok_or("receipt")?;
    let executable_bytes = fs::read(executable_path).map_err(|_| "receipt")?;
    if sha256_bytes(&executable_bytes) != expected_sha256 {
        return Err("receipt");
    }
    let installed = destination.join("bin/tc-proof-host");
    if let Some(parent) = installed.parent() {
        fs::create_dir_all(parent).map_err(|_| "receipt")?;
    }
    fs::write(&installed, executable_bytes).map_err(|_| "receipt")?;
    fs::set_permissions(&installed, fs::Permissions::from_mode(0o755)).map_err(|_| "receipt")?;
    if sha256_bytes(&fs::read(&installed).map_err(|_| "receipt")?) != expected_sha256 {
        return Err("receipt");
    }
    Ok(())
}
