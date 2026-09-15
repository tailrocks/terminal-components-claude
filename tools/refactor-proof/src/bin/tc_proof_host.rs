//! tc-proof-host entrypoint.

#![expect(
    clippy::print_stdout,
    reason = "tc-proof-host emits schema-versioned JSON results on stdout"
)]
#![expect(
    clippy::print_stderr,
    reason = "tc-proof-host emits diagnostics on stderr"
)]

use std::env;
use std::process::ExitCode;

use refactor_proof::host::{dispatch_install, dispatch_prepare, dispatch_unimplemented, HostOperation};

const OPERATIONS: [HostOperation; 6] = [
    HostOperation::Install,
    HostOperation::Prepare,
    HostOperation::Freeze,
    HostOperation::Verify,
    HostOperation::Seal,
    HostOperation::Integrate,
];

const HOST_HELP: &str = "\
tc-proof-host — host-only proof operations

operations:
  install    install accepted harness executable from receipt
  prepare    initialize run preparation and progress
  freeze     freeze candidate tree and operation contexts
  verify     run observer-driven workers and taskfmt gate
  seal       seal oracle bundle (rejected for non-baseline tasks)
  integrate  compare-and-swap integration ref update

See docs/refactoring-plan/proof-contract.md.
";

fn usage() -> ! {
    eprint!("{HOST_HELP}");
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print!("{HOST_HELP}");
        return ExitCode::SUCCESS;
    }
    if args[0].starts_with('-') {
        eprintln!("tc-proof-host: unknown option: {}", args[0]);
        usage();
    }
    let operation = args.remove(0);
    let Some(host_operation) = operation.parse().ok() else {
        eprintln!("tc-proof-host: unknown operation: {operation}");
        usage();
    };
    if !OPERATIONS.contains(&host_operation) {
        eprintln!("tc-proof-host: unknown operation: {operation}");
        usage();
    }
    match host_operation {
        HostOperation::Install => dispatch_install(&args),
        HostOperation::Prepare => dispatch_prepare(&args),
        HostOperation::Freeze
        | HostOperation::Verify
        | HostOperation::Seal
        | HostOperation::Integrate => dispatch_unimplemented(host_operation),
    }
}
