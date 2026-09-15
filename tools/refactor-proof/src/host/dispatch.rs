//! Host operation dispatch and stdout result emission.

use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;

use super::freeze::{run_freeze, FreezeOutcome};
use super::install::{run_install, InstallOutcome};
use super::operation::HostOperation;
use super::prepare::{run_prepare, PrepareOutcome};
use super::result::HostResult;
use super::verify::{run_verify, VerifyOutcome};

pub fn emit_result(result: &HostResult) -> io::Result<()> {
    let json = serde_json::to_string(result).map_err(io::Error::other)?;
    writeln!(io::stdout(), "{json}")
}

pub fn finish(result: HostResult) -> ExitCode {
    let code = match result.status {
        super::result::HostStatus::Passed => 0,
        super::result::HostStatus::Rejected => 1,
    };
    if let Err(error) = emit_result(&result) {
        eprintln!("tc-proof-host: write result: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(code)
}

pub fn dispatch_unimplemented(operation: HostOperation) -> ExitCode {
    finish(HostResult::rejected(operation, "unsupported"))
}

pub fn dispatch_install(args: &[String]) -> ExitCode {
    let Some(receipt) = flag_value(args, "--receipt") else {
        return usage_exit("install requires --receipt and --destination");
    };
    let Some(destination) = flag_value(args, "--destination") else {
        return usage_exit("install requires --receipt and --destination");
    };
    if has_unknown_flags(args, &["--receipt", "--destination"]) {
        return usage_exit("install received unknown option");
    }
    match run_install(&receipt, &destination) {
        InstallOutcome::Passed => finish(HostResult::passed(HostOperation::Install)),
        InstallOutcome::Rejected(category) => {
            finish(HostResult::rejected(HostOperation::Install, category))
        }
        InstallOutcome::Failed(error) => {
            eprintln!("tc-proof-host: install: {error}");
            finish(HostResult::rejected(HostOperation::Install, "receipt"))
        }
    }
}

pub fn dispatch_freeze(args: &[String]) -> ExitCode {
    let Some(run) = flag_value(args, "--run") else {
        return usage_exit("freeze requires --run and --candidate");
    };
    let Some(candidate) = flag_value(args, "--candidate") else {
        return usage_exit("freeze requires --run and --candidate");
    };
    if has_unknown_flags(args, &["--run", "--candidate"]) {
        return usage_exit("freeze received unknown option");
    }
    match run_freeze(&run, &candidate) {
        FreezeOutcome::Passed => finish(HostResult::passed(HostOperation::Freeze)),
        FreezeOutcome::Rejected(category) => {
            finish(HostResult::rejected(HostOperation::Freeze, category))
        }
        FreezeOutcome::Failed(error) => {
            eprintln!("tc-proof-host: freeze: {error}");
            finish(HostResult::rejected(HostOperation::Freeze, "integrity"))
        }
    }
}

pub fn dispatch_verify(args: &[String]) -> ExitCode {
    let Some(run) = flag_value(args, "--run") else {
        return usage_exit("verify requires --run");
    };
    if has_unknown_flags(args, &["--run"]) {
        return usage_exit("verify received unknown option");
    }
    match run_verify(&run) {
        VerifyOutcome::Passed => finish(HostResult::passed(HostOperation::Verify)),
        VerifyOutcome::Rejected(category) => {
            finish(HostResult::rejected(HostOperation::Verify, category))
        }
        VerifyOutcome::Failed(error) => {
            eprintln!("tc-proof-host: verify: {error}");
            finish(HostResult::rejected(HostOperation::Verify, "integrity"))
        }
    }
}

pub fn dispatch_prepare(args: &[String]) -> ExitCode {
    let Some(campaign) = flag_value(args, "--campaign") else {
        return usage_exit("prepare requires --campaign, --task, --parent, and --run");
    };
    let Some(task) = flag_string(args, "--task") else {
        return usage_exit("prepare requires --campaign, --task, --parent, and --run");
    };
    let Some(parent) = flag_string(args, "--parent") else {
        return usage_exit("prepare requires --campaign, --task, --parent, and --run");
    };
    let Some(run) = flag_value(args, "--run") else {
        return usage_exit("prepare requires --campaign, --task, --parent, and --run");
    };
    if has_unknown_flags(args, &["--campaign", "--task", "--parent", "--run"]) {
        return usage_exit("prepare received unknown option");
    }
    match run_prepare(&campaign, &task, &parent, &run) {
        PrepareOutcome::Passed => finish(HostResult::passed(HostOperation::Prepare)),
        PrepareOutcome::Rejected(category) => {
            finish(HostResult::rejected(HostOperation::Prepare, category))
        }
        PrepareOutcome::Failed(error) => {
            eprintln!("tc-proof-host: prepare: {error}");
            finish(HostResult::rejected(HostOperation::Prepare, "integrity"))
        }
    }
}

fn flag_value(args: &[String], flag: &str) -> Option<PathBuf> {
    let index = args.iter().position(|arg| arg == flag)?;
    args.get(index + 1).map(PathBuf::from)
}

fn flag_string(args: &[String], flag: &str) -> Option<String> {
    let index = args.iter().position(|arg| arg == flag)?;
    args.get(index + 1).cloned()
}

fn has_unknown_flags(args: &[String], allowed: &[&str]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        if arg.starts_with('-') {
            if !allowed.contains(&arg) {
                eprintln!("tc-proof-host: unknown option: {arg}");
                return true;
            }
            index += 2;
            continue;
        }
        index += 1;
    }
    false
}

fn usage_exit(message: &str) -> ExitCode {
    eprintln!("tc-proof-host: {message}");
    ExitCode::from(2)
}
