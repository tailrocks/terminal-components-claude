use crate::scenarios;
use crate::support;
use termrock_core::platform::ClientPlatform;

pub const CLIENT_PLATFORM_PROBE_END: &str = "END-OF-CLIENT-PLATFORM-PROBE";

pub fn run(tool: &str) -> Result<(), String> {
    match tool {
        "list" => {
            list();
            Ok(())
        }
        "capture" => {
            let name = std::env::var("VISUAL_NAME")
                .map_err(|_| "set VISUAL_NAME=<scenario name>".to_string())?;
            let tag = std::env::var("VISUAL_AS").unwrap_or_else(|_| "capture".into());
            capture(&name, &tag)
        }
        "env-probe" => {
            env_probe();
            Ok(())
        }
        "client-platform-probe" => {
            client_platform_probe();
            Ok(())
        }
        "client-platform-proxy" => {
            client_platform_proxy();
            Ok(())
        }
        "harness-hash" => {
            let dir = std::env::var_os("VISUAL_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(crate::oracle::harness_dir);
            if !dir.is_dir() {
                return Err(format!("{} is not a directory", dir.display()));
            }
            println!("{}", crate::oracle::harness_sha256(&dir));
            Ok(())
        }
        other => Err(format!(
            "unknown VISUAL_TOOL `{other}` (list, capture, env-probe, client-platform-probe, client-platform-proxy, harness-hash)"
        )),
    }
}

fn client_platform_probe() {
    let label = match ClientPlatform::current() {
        ClientPlatform::MacOs => "macos",
        ClientPlatform::Other => "other",
    };
    println!("CLIENT_PLATFORM={label}");
    println!(
        "CLIENT_PLATFORM_OVERRIDE={}",
        std::env::var(ClientPlatform::OVERRIDE_ENV).unwrap_or_else(|_| "<unset>".to_string())
    );
    println!("{CLIENT_PLATFORM_PROBE_END}");
}

fn client_platform_proxy() {
    let executable = std::env::current_exe().expect("test binary path");
    let case = support::Case::new(
        "probe/client-platform/120x40/nocolor",
        "unused",
        &[],
        120,
        40,
        support::Color::NoColorEnv,
        CLIENT_PLATFORM_PROBE_END,
    );
    let mut extra = vec![("VISUAL_TOOL", "client-platform-probe")];
    match std::env::var("VISUAL_CHILD_CLIENT_PLATFORM")
        .ok()
        .as_deref()
    {
        Some("other") => {
            extra.push((ClientPlatform::OVERRIDE_ENV, "other"));
            extra.push(("TERM_PROGRAM", "Apple_Terminal"));
            extra.push(("LC_TERMINAL", "iTerm2"));
        }
        Some("macos") => {
            extra.push((ClientPlatform::OVERRIDE_ENV, "macos"));
            extra.push(("TERM_PROGRAM", "WindowsTerminal"));
            extra.push(("LC_TERMINAL", "WindowsTerminal"));
        }
        Some(other) => panic!("unsupported child platform override `{other}`"),
        None => {}
    }

    let mut session = support::spawn_with_program(&case, &executable, &extra);
    session
        .wait_for_text(CLIENT_PLATFORM_PROBE_END)
        .unwrap_or_else(|e| panic!("isolated platform child did not report its result: {e}"));
    let frame = session
        .wait_stable(std::time::Duration::from_millis(100))
        .unwrap_or_else(|e| panic!("isolated platform child did not settle: {e}"));
    print!("{}", frame.text());
}

fn approval(name: &str) -> &'static str {
    let root = support::approved_root();
    if support::is_reapproved(name) {
        "reapproved"
    } else if root.join(format!("{name}.ansi")).exists() {
        "legacy"
    } else {
        "missing"
    }
}

fn list() {
    for s in scenarios::all() {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            s.name,
            s.test_name(),
            s.kind.table(),
            s.tier,
            if s.pending { "pending" } else { "stored" },
            approval(&s.name),
        );
    }
}

pub const ENV_PROBE_END: &str = "END-OF-ENV";

fn env_probe() {
    let case = support::Case::new(
        "probe/env/200x50/nocolor",
        "/bin/sh",
        &["-c", "env | sort; echo END-OF-ENV; sleep 30"],
        200,
        50,
        support::Color::NoColorEnv,
        ENV_PROBE_END,
    );
    let mut session = support::spawn_boot(&case);
    let frame = session
        .wait_stable(std::time::Duration::from_millis(100))
        .unwrap_or_else(|e| panic!("env probe never settled: {e}"));
    print!("{}", frame.text());
}

fn capture(name: &str, tag: &str) -> Result<(), String> {
    let name = name.replace("::", "/");
    let scenario = scenarios::all()
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("`{name}` is not a scenario (cargo xtask visual list)"))?;
    let frame = scenario.capture(&format!("probe/{tag}/{name}"))?;
    print!("{}", tuiscotti::render::ansi_dump_with_cursor(&frame));
    Ok(())
}
