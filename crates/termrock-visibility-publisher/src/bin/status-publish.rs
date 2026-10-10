#[path = "../repository_writer.rs"]
mod repository_writer;

use repository_writer::{
    LIVE_PUBLISHING_ENABLED, PublishError, StatusTransport, parse_invocation, publish,
    read_policy_from_test_environment, read_stdin_bounded, run_context_from_environment,
    test_mode_enabled, token_from_environment,
};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("status publication failed: {error}");
            std::process::exit(exit_code(&error));
        }
    }
}

fn run() -> Result<(), PublishError> {
    if !test_mode_enabled() && !LIVE_PUBLISHING_ENABLED {
        return Err(PublishError::LivePublishingDisabled);
    }
    if test_mode_enabled() && !cfg!(debug_assertions) {
        return Err(PublishError::TestModeUnavailable);
    }
    if std::env::args_os().len() != 1 {
        return Err(PublishError::InvalidInput(
            "this command accepts request JSON on stdin and no arguments",
        ));
    }

    let request_bytes = read_stdin_bounded()?;
    let invocation = parse_invocation(&request_bytes)?;
    let policy = read_policy_from_test_environment()?;
    let context = run_context_from_environment()?;
    let token = token_from_environment()?;
    let transport = StatusTransport::from_environment(token)?;
    let result = publish(invocation, policy, context, transport)?;
    let encoded = serde_json::to_string(&result).map_err(|_| PublishError::InvalidResponse)?;
    println!("{encoded}");
    Ok(())
}

fn exit_code(error: &PublishError) -> i32 {
    match error {
        PublishError::LivePublishingDisabled | PublishError::TestModeUnavailable => 3,
        PublishError::InvalidInput(_) | PublishError::UntrustedWorkflow => 2,
        _ => 1,
    }
}
