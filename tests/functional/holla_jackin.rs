use super::support::{App, HOLLA, JACKIN, Point, crop, find, find_all, row_text, text};

const PAUSED: &[&str] = &["--motion", "paused", "--frame", "40"];

fn button(app: &App, label: &str) -> Point {
    let screen = app.wait_text(label);
    // The dialog action row contains Cancel as well as the requested action;
    // titles and footer hints are deliberately excluded.
    let candidates: Vec<_> = find_all(&screen, label)
        .into_iter()
        .filter(|point| {
            u32::from(point.y) + 2 < screen.rows()
                && row_text(&screen, u32::from(point.y)).contains("Cancel")
                && row_text(&screen, u32::from(point.y)).contains('│')
        })
        .collect();
    assert_eq!(
        candidates.len(),
        1,
        "one dialog action {label:?}:\n{}",
        text(&screen)
    );
    candidates[0]
}

#[test]
fn holla_trust_cancel_then_run_uses_the_child_directory() {
    let app = App::new(
        "holla-trust-run",
        HOLLA,
        &["--scenario", "monorepo-child", "--motion", "reduced"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("From acme · the parent root");
    app.type_text("test");
    let results = app.wait_text("alias test");
    assert!(text(&results).contains("Run tests"));
    app.key("enter");
    app.wait_text("Trust ~/work/acme/apps/frontend/mise.toml?");
    app.wait_text("[tasks.test]");
    app.key("escape");
    let cancelled = app.wait_absent("Trust ~/work/acme/apps/frontend/mise.toml?");
    assert!(text(&cancelled).contains("Run tests"));
    assert!(!text(&cancelled).contains("19 passed"));

    app.key("enter");
    app.wait_text("Trust ~/work/acme/apps/frontend/mise.toml?");
    app.key("right");
    app.key("enter");
    app.wait_text("arguments");
    app.key("ctrl+s");
    let launched = app.wait_screen("frontend test has its own activity and output", |screen| {
        let visible = text(screen);
        visible
            .lines()
            .any(|line| line.trim_start().starts_with("frontend test ·"))
            && visible.contains("╭─ output")
    });
    let launched_text = text(&launched);
    assert!(launched_text.lines().any(|line| {
        line.trim_start()
            .starts_with("$ mise run //apps/frontend:test")
    }));
    let activity_header = launched_text
        .lines()
        .find(|line| line.trim_start().starts_with("frontend test ·"))
        .expect("the activity header");
    let directory = activity_header
        .split("here ·")
        .nth(1)
        .and_then(|location| location.split('·').next())
        .expect("activity header states its working directory")
        .trim();
    assert_eq!(directory, "~/work/acme/apps/frontend");
    app.wait_text("19 passed (19)");
    app.wait_text("succeeded");
    app.finish();
}

#[test]
fn holla_browser_hidden_files_preview_scroll_and_bad_path_recovery() {
    let app = App::new(
        "holla-browser",
        HOLLA,
        &["--scenario", "parity-browser", "--motion", "reduced"],
        120,
        40,
        "nocolor",
    );
    app.wait_text("holla❯");
    app.type_text("Browse ~/work/site");
    app.key("enter");
    let initial = app.wait_text("16 entries");
    assert!(!text(&initial).contains(".env"));
    app.key("ctrl+h");
    app.wait_text("Hidden entries shown");
    app.wait_text(".env");
    app.key("ctrl+h");
    let hidden = app.wait_text("Hidden entries hidden");
    assert!(!text(&hidden).contains(".env"));
    app.key("home");
    app.key("down");
    app.key("down");
    app.key("down");
    app.wait_text("first 2000 lines");
    let original = app.wait_text("line 5");
    let preview = find(&original, "line 5");
    // The preview's line-number gutter also scrolls. Its visible left border
    // separates that gutter from the file list, which must stay unchanged.
    let preview_border = original.row(u32::from(preview.y)).expect("preview row")
        [..usize::from(preview.x)]
        .iter()
        .rposition(|cell| cell.text() == "│")
        .expect("preview has a visible left border");
    let segments = |screen: &tui_test::TerminalSnapshot| {
        let cells = screen.row(u32::from(preview.y)).expect("preview row");
        let split = usize::from(preview.x);
        let left: String = cells[..preview_border]
            .iter()
            .map(|cell| cell.text())
            .collect();
        let preview: String = cells[split..].iter().map(|cell| cell.text()).collect();
        (left, preview)
    };
    let (before_left, before_preview) = segments(&original);
    app.wheel(preview, true, 3);
    let scrolled = app.wait_screen("preview content advances after wheel", |screen| {
        segments(screen).1 != before_preview && text(screen).contains("first 2000 lines")
    });
    assert_eq!(
        segments(&scrolled).0,
        before_left,
        "wheel leaves the file list unchanged"
    );
    assert!(text(&scrolled).contains("16 entries"));
    app.wheel(preview, false, 3);
    let restored = app.wait_screen("preview wheel up restores original content", |screen| {
        segments(screen).1 == before_preview
    });
    assert_eq!(
        segments(&restored).0,
        before_left,
        "reverse wheel leaves the file list unchanged"
    );
    app.key("g");
    app.wait_text("type a path or a name");
    app.type_text("nope");
    app.key("enter");
    app.wait_text("no such file or directory");
    app.key("escape");
    app.wait_absent("no such file or directory");
    app.wait_text("type a path or a name");
    app.key("escape");
    app.wait_absent("type a path or a name");
    app.wait_text("16 entries");
    app.key("home");
    app.key("g");
    app.wait_text("type a path or a name");
    app.key("escape");
    app.wait_absent("type a path or a name");
    app.finish();
}

#[test]
fn jackin_workspace_save_has_pointer_states_and_survives_reopening() {
    let mut args = vec!["--scenario", "returning"];
    args.extend_from_slice(PAUSED);
    let app = App::new("jackin-workspace", JACKIN, &args, 120, 40, "truecolor");
    app.wait_text("jackin❯");
    app.key("down");
    app.key("e");
    app.wait_text("payments-platform › edit");
    app.key("]");
    app.wait_text("Destination");
    let original = app.wait_text("/workspace/payments-platform");
    let mount = find(&original, "/workspace/payments-platform");
    let original_row = row_text(&original, u32::from(mount.y));
    assert!(original_row.split_whitespace().any(|token| token == "rw"));
    assert!(
        original_row
            .split_whitespace()
            .any(|token| token == "worktree")
    );
    app.key("enter");
    app.key("r");
    app.key("i");
    let dirty = app.wait_text("• 1 change");
    assert!(text(&dirty).contains("Mounts •"));
    app.key("escape");
    app.key("escape");
    app.wait_text("Save changes before leaving?");
    app.key("escape");
    app.wait_absent("Save changes before leaving?");
    app.wait_text("• 1 change");
    app.key("ctrl+s");
    app.wait_text("Save workspace");
    app.wait_text("1 modified");

    let save = button(&app, "Save");
    app.move_to(Point { x: 1, y: 1 });
    let rest = crop(&app.settle(), save, 4, 1);
    app.move_to(save);
    let hovered = app.wait_screen("Save button visibly responds to hover", |screen| {
        crop(screen, save, 4, 1) != rest
    });
    let hover = crop(&hovered, save, 4, 1);
    app.down(save);
    let held = app.wait_screen("Save button visibly responds while held", |screen| {
        let cells = crop(screen, save, 4, 1);
        cells != rest && cells != hover
    });
    assert!(
        text(&held).contains("Save workspace"),
        "press must not save before release"
    );
    assert!(!text(&held).contains("Saving payments-platform"));
    app.up(save);
    app.wait_text("Saving payments-platform");
    app.advance_clock(1000);
    app.wait_text("Workspace payments-platform saved");
    app.key("e");
    app.wait_text("payments-platform › edit");
    app.key("]");
    app.key("enter");
    app.wait_text("Destination");
    let saved = app.wait_text("/workspace/payments-platform");
    let saved_mount = find(&saved, "/workspace/payments-platform");
    let saved_row = row_text(&saved, u32::from(saved_mount.y));
    assert!(
        saved_row.split_whitespace().any(|token| token == "ro"),
        "saved mount is read-only: {saved_row}"
    );
    assert!(
        saved_row.split_whitespace().any(|token| token == "clone"),
        "saved mount uses clone isolation: {saved_row}"
    );
    assert!(
        !text(&saved).contains("• 1 change"),
        "reopening begins clean"
    );
    app.key("escape");
    app.key("escape");
    app.wait_absent("payments-platform › edit");
    app.finish();
}

#[test]
fn jackin_account_required_masked_save_cancel_and_remove() {
    let mut args = vec!["--scenario", "accounts-mixed"];
    args.extend_from_slice(PAUSED);
    let app = App::new("jackin-account", JACKIN, &args, 120, 40, "nocolor");
    app.wait_text("Overview");
    app.key("a");
    app.wait_text("New account");
    app.click(button(&app, "Save"));
    let required = app.wait_text("Required");
    assert!(text(&required).contains("New account"));
    app.key("enter");
    app.type_text("Spare");
    app.key("tab");
    app.key("tab");
    app.key("tab");
    app.key("down");
    app.key("down");
    app.wait_text("API key");
    app.key("tab");
    app.key("enter");
    app.type_text("sk-ant-valid-abcdef1234");
    let masked = app.wait_text("••••••••••••");
    assert!(!text(&masked).contains("sk-ant-valid"));
    assert!(!text(&masked).contains("abcdef"));
    app.key("tab");
    app.wait_text("1234");
    app.click(button(&app, "Save"));
    let saved = app.wait_text("Saved Claude · Spare");
    assert!(!text(&saved).contains("abcdef"));
    assert!(!text(&saved).contains("New account"));
    app.key("d");
    app.wait_text("Remove account Spare?");
    app.key("escape");
    app.wait_text("Cancelled · account kept");
    app.wait_text("Accounts › Claude › Spare");
    app.key("d");
    app.wait_text("Remove account Spare?");
    app.click(button(&app, "Remove"));
    app.wait_text("Removed Claude · Spare");
    app.key("/");
    app.wait_text("Filter accounts");
    app.type_text("Spare");
    app.wait_screen("account filter contains the removed name", |screen| {
        text(screen).contains("Filter accounts")
            && text(screen)
                .lines()
                .any(|line| line.contains("Spare") && line.contains('│'))
    });
    app.key("enter");
    let removed = app.wait_text("Accounts · filt");
    assert!(text(&removed).contains("Accounts › Overview"));
    let header = find(&removed, "Accounts · filt");
    let list: Vec<String> = text(&removed)
        .lines()
        .skip(usize::from(header.y) + 1)
        .take_while(|line| !line.contains('╰'))
        .filter_map(|line| line.split('│').nth(1))
        .map(|cell| cell.replace('▎', "").trim().to_owned())
        .filter(|cell| !cell.is_empty())
        .collect();
    assert_eq!(
        list,
        ["Overview"],
        "filter has no remaining matching account"
    );
    app.finish();
}
