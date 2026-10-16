//! Tool mode of the test binary for `cargo xtask visual`: when
//! `VISUAL_TOOL` is set, `main` runs one of these instead of the tests.
//! They reuse the scenario tables and the tuiscotti renderer, so xtask needs
//! neither.
//!
//! - `list`: one TSV row per scenario:
//!   `name  test_name  table  tier  pending  approval` where `approval` is
//!   `reapproved` (tuiscotti `.ansi`), `legacy` (tuisnap's four files) or
//!   `missing`.
//! - `render`: `VISUAL_NAME=<store name>` renders the approved `.ansi` to
//!   `target/tuiscotti/render/<name>.{png,html}` and prints both paths.

use crate::scenarios;
use crate::support;

pub fn run(tool: &str) -> Result<(), String> {
    match tool {
        "list" => {
            list();
            Ok(())
        }
        "render" => {
            let name = std::env::var("VISUAL_NAME")
                .map_err(|_| "set VISUAL_NAME=<scenario name>".to_string())?;
            render(&name)
        }
        other => Err(format!("unknown VISUAL_TOOL `{other}` (list, render)")),
    }
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

fn render(name: &str) -> Result<(), String> {
    let name = name.replace("::", "/");
    if !scenarios::all().iter().any(|s| s.name == name) {
        return Err(format!(
            "`{name}` is not a scenario (cargo xtask visual list)"
        ));
    }
    let (png, html) = support::render_approved(&name)?;
    println!("{}", png.display());
    println!("{}", html.display());
    Ok(())
}
