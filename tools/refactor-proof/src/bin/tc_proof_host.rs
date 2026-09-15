//! tc-proof-host entrypoint (Phase 0 stub).

#![expect(
    clippy::print_stdout,
    reason = "tc-proof-host emits schema-versioned JSON results on stdout"
)]
#![expect(
    clippy::print_stderr,
    reason = "tc-proof-host emits diagnostics on stderr"
)]

use std::env;
use std::io::{self, Write as _};
use std::process::ExitCode;

use serde_json::json;

const OPERATIONS: [&str; 6] = [
    "install",
    "prepare",
    "freeze",
    "verify",
    "seal",
    "integrate",
];

const HOST_HELP: &str = "\
tc-proof-host — host-only proof operations (Phase 0 stub)

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

fn emit_result(operation: &str) -> io::Result<()> {
    let result = json!({
        "schema": "tc-proof-host-result/v1",
        "operation": operation,
        "status": "rejected",
        "category": "unsupported"
    });
    writeln!(io::stdout(), "{result}")
}

fn dispatch(operation: &str) -> ExitCode {
    if let Err(error) = emit_result(operation) {
        eprintln!("tc-proof-host: write result: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(1)
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
    if !OPERATIONS.contains(&operation.as_str()) {
        eprintln!("tc-proof-host: unknown operation: {operation}");
        usage();
    }
    dispatch(&operation)
}
