//! tc-proof comparator entrypoint (Phase 0 stub).

#![expect(
    clippy::print_stdout,
    reason = "tc-proof emits schema-versioned JSON results on stdout"
)]
#![expect(
    clippy::print_stderr,
    reason = "tc-proof emits diagnostics on stderr"
)]

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use refactor_proof::context_sha256;
use serde_json::{json, Value};

const COMPARE_HELP: &str = "tc-proof compare --context PATH\n";

fn usage() -> ! {
    eprint!(
        "usage: tc-proof compare --context PATH\n\
         \n\
         Phase 0 stub; see docs/refactoring-plan/proof-contract.md.\n"
    );
    std::process::exit(2);
}

fn parse_compare_args(args: &[String]) -> PathBuf {
    let mut context = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--context" => {
                index += 1;
                context = Some(
                    args.get(index)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| usage()),
                );
            }
            "--help" | "-h" => {
                print!("{COMPARE_HELP}");
                std::process::exit(0);
            }
            other => {
                eprintln!("tc-proof: unknown option: {other}");
                usage();
            }
        }
        index += 1;
    }
    context.unwrap_or_else(|| usage())
}

fn load_context(path: &PathBuf) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|error| {
        eprintln!("tc-proof: read context {}: {error}", path.display());
        std::process::exit(2);
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        eprintln!("tc-proof: parse context {}: {error}", path.display());
        std::process::exit(2);
    })
}

fn write_stub_comparison(context: &Value) -> io::Result<()> {
    let report_path = context
        .get("report_path")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing report_path"))?;
    let run_id = context
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown-run");
    let task_id = context
        .get("task_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown-task");
    let required_count = context
        .get("required_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let report = json!({
        "schema": "tc-proof-comparison/v1",
        "run_id": run_id,
        "task_id": task_id,
        "context_sha256": context_sha256(context),
        "required_count": required_count,
        "checked_count": 0,
        "passed_count": 0,
        "results": [],
        "failures": [{"id": null, "code": "INTEGRITY"}]
    });
    if let Some(parent) = PathBuf::from(report_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(report_path, format!("{report}\n"))
}

fn compare(context_path: PathBuf) -> ExitCode {
    let context = load_context(&context_path);
    if let Err(error) = write_stub_comparison(&context) {
        eprintln!("tc-proof: write comparison report: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(1)
}

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print!(
            "usage: tc-proof compare --context PATH\n\
             \n\
             Phase 0 stub; see docs/refactoring-plan/proof-contract.md.\n"
        );
        return ExitCode::SUCCESS;
    }
    match args.remove(0).as_str() {
        "compare" => compare(parse_compare_args(&args)),
        other => {
            eprintln!("tc-proof: unknown command: {other}");
            usage();
        }
    }
}
