//! Command-line entry point for the deterministic Jackin Preview shell.

fn main() -> std::io::Result<()> {
    let mut scenario = jackin_app::Scenario::Returning;
    let mut motion =
        jackin_app::Motion::resolve(None, std::env::var_os("JACKIN_NO_MOTION").is_some());
    let mut frame = 0_u64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => {
                if let Some(value) = args.next()
                    && let Some(parsed) = jackin_app::Scenario::from_name(&value)
                {
                    scenario = parsed;
                }
            }
            "--motion" => {
                if let Some(value) = args.next()
                    && let Some(parsed) = jackin_app::Motion::from_name(&value)
                {
                    motion = parsed;
                }
            }
            "--frame" => {
                if let Some(value) = args.next()
                    && let Ok(parsed) = value.parse::<u64>()
                {
                    frame = parsed;
                }
            }
            _ => {}
        }
    }
    jackin_app::run_scenario(scenario, motion, frame)
}
