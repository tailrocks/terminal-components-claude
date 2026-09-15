//! `install` copies the accepted harness executable from a receipt.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::authority::{load_authority, load_receipt, Authority};
use crate::json_util::sha256_bytes;

pub(super) enum InstallOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

pub(super) fn run_install(receipt_path: &Path, destination: &Path) -> InstallOutcome {
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
    let receipt_bytes = fs::read(executable_path).map_err(|_| "receipt")?;
    if sha256_bytes(&receipt_bytes) != expected_sha256 {
        return Err("receipt");
    }
    let install_bytes = material_executable_bytes(executable_path, &receipt_bytes)?;
    let installed = destination.join("bin/tc-proof-host");
    if let Some(parent) = installed.parent() {
        fs::create_dir_all(parent).map_err(|_| "receipt")?;
    }
    fs::write(&installed, &install_bytes).map_err(|_| "receipt")?;
    fs::set_permissions(&installed, fs::Permissions::from_mode(0o755)).map_err(|_| "receipt")?;
    if sha256_bytes(&fs::read(&installed).map_err(|_| "receipt")?) != sha256_bytes(&install_bytes) {
        return Err("receipt");
    }
    Ok(())
}

fn material_executable_bytes(
    executable_path: &Path,
    receipt_bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if is_mach_o_executable(receipt_bytes) {
        return Ok(receipt_bytes.to_vec());
    }
    if is_shell_wrapper(receipt_bytes) {
        let resolved = resolve_wrapper_target(executable_path, receipt_bytes)?;
        if let Ok(bytes) = fs::read(&resolved) {
            return Ok(bytes);
        }
        // Qualification sandboxes may allow the running image but not the build tree.
        let current = std::env::current_exe().map_err(|_| "receipt")?;
        let current_bytes = fs::read(&current).map_err(|_| "receipt")?;
        if !is_mach_o_executable(&current_bytes) {
            return Err("receipt");
        }
        if same_executable(&current, &resolved) || current.file_name() == resolved.file_name() {
            return Ok(current_bytes);
        }
        return Err("receipt");
    }
    Ok(receipt_bytes.to_vec())
}

fn same_executable(left: &Path, right: &Path) -> bool {
    left.canonicalize()
        .ok()
        .zip(right.canonicalize().ok())
        .is_some_and(|(left, right)| left == right)
}

fn is_mach_o_executable(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\xcf\xfa\xed\xfe") || bytes.starts_with(b"\xce\xfa\xed\xfe")
}

fn is_shell_wrapper(bytes: &[u8]) -> bool {
    bytes.starts_with(b"#!") && bytes.windows(5).any(|window| window == b"exec ")
}

fn resolve_wrapper_target(wrapper_path: &Path, wrapper_bytes: &[u8]) -> Result<PathBuf, &'static str> {
    let text = std::str::from_utf8(wrapper_bytes).map_err(|_| "receipt")?;
    let bin_dir = wrapper_path.parent().ok_or("receipt")?;
    let crate_root = bin_dir.parent().ok_or("receipt")?;
    let workspace_root = crate_root
        .parent()
        .and_then(|tools| tools.parent())
        .ok_or("receipt")?;
    if text.contains("target/debug/tc-proof-host") {
        return Ok(workspace_root.join("target/debug/tc-proof-host"));
    }
    if text.contains("target/debug/tc-proof") {
        return Ok(workspace_root.join("target/debug/tc-proof"));
    }
    Err("receipt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_refactor_proof_wrapper_to_workspace_binary() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path().join("workspace");
        let wrapper_path = workspace.join("tools/refactor-proof/bin/tc-proof-host");
        fs::create_dir_all(wrapper_path.parent().expect("parent")).expect("mkdir");
        fs::write(
            &wrapper_path,
            b"#!/usr/bin/env bash\nexec \"$workspace_root/target/debug/tc-proof-host\" \"$@\"\n",
        )
        .expect("write wrapper");
        let built = workspace.join("target/debug/tc-proof-host");
        fs::create_dir_all(built.parent().expect("parent")).expect("mkdir target");
        fs::write(&built, b"\xcf\xfa\xed\xfefake-macho").expect("write macho");

        let wrapper_bytes = fs::read(&wrapper_path).expect("read wrapper");
        let resolved = material_executable_bytes(&wrapper_path, &wrapper_bytes).expect("resolve");
        assert_eq!(resolved, b"\xcf\xfa\xed\xfefake-macho");
    }

    #[test]
    fn leaves_mach_o_bytes_unmodified() {
        let bytes = b"\xcf\xfa\xed\xfealready-macho";
        let material = material_executable_bytes(Path::new("/ignored"), bytes).expect("resolve");
        assert_eq!(material, bytes);
    }
}
