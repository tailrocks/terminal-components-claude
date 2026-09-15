//! tc-proof comparator entrypoint.

#![expect(
    clippy::print_stdout,
    reason = "tc-proof emits schema-versioned JSON results on stdout"
)]
#![expect(
    clippy::print_stderr,
    reason = "tc-proof emits diagnostics on stderr"
)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use refactor_proof::compare;

const COMPARE_HELP: &str = "tc-proof compare --context PATH\n";

fn usage() -> ! {
    eprint!(
        "usage: tc-proof compare --context PATH\n\
         \n\
         See docs/refactoring-plan/proof-contract.md.\n"
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

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print!(
            "usage: tc-proof compare --context PATH\n\
             \n\
             See docs/refactoring-plan/proof-contract.md.\n"
        );
        return ExitCode::SUCCESS;
    }
    match args.remove(0).as_str() {
        "compare" => {
            let code = compare::run_compare(&parse_compare_args(&args)).unwrap_or(2);
            ExitCode::from(code as u8)
        }
        other => {
            eprintln!("tc-proof: unknown command: {other}");
            usage();
        }
    }
}
