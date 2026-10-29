//! Paused zoomed capsule, composed through the shared zoomed-pane painter.

use termrock::Ui;
use termrock::feedback::{ZoomHint, ZoomSpan, ZoomTab, ZoomTone, ZoomedPane};

fn line<'a>(text: &'a str, tone: ZoomTone) -> ZoomSpan<'a> {
    ZoomSpan {
        text,
        tone,
        bold: false,
        italic: false,
    }
}

fn bold_line<'a>(text: &'a str, tone: ZoomTone) -> ZoomSpan<'a> {
    ZoomSpan {
        text,
        tone,
        bold: true,
        italic: false,
    }
}

fn italic_line<'a>(text: &'a str, tone: ZoomTone) -> ZoomSpan<'a> {
    ZoomSpan {
        text,
        tone,
        bold: false,
        italic: true,
    }
}

pub(super) fn paint(ui: &mut Ui<'_>) {
    let menus = ["File", "Edit", "View", "Session", "Help"];
    let tabs = [
        ZoomTab {
            index: "1",
            label: "Mix (3)",
            live: true,
            selected: true,
        },
        ZoomTab {
            index: "2",
            label: "Shell",
            live: false,
            selected: false,
        },
        ZoomTab {
            index: "3",
            label: "docs",
            live: true,
            selected: false,
        },
    ];
    let banner = [
        line("▐ ", ZoomTone::Ok),
        line("Claude Code v2.1.14", ZoomTone::Primary),
        line(" · Opus 4.5 · ", ZoomTone::Muted),
        line("~/payments-platform", ZoomTone::Secondary),
    ];
    let user = [
        line("› ", ZoomTone::Muted),
        line(
            "Refactor the settlement retry loop so failed batches",
            ZoomTone::Primary,
        ),
    ];
    let user_cont = [line(
        "  back off exponentially and cap at 5 attempts.",
        ZoomTone::Primary,
    )];
    let read_retry = [
        line("● ", ZoomTone::Secondary),
        line("Read ", ZoomTone::Primary),
        line("src/settlement/retry.rs", ZoomTone::Secondary),
        line(" (142 lines)", ZoomTone::Muted),
    ];
    let read_mod = [
        line("● ", ZoomTone::Secondary),
        line("Read ", ZoomTone::Primary),
        line("src/settlement/mod.rs", ZoomTone::Secondary),
        line(" (88 lines)", ZoomTone::Muted),
    ];
    let explain = [
        line("● ", ZoomTone::Secondary),
        line(
            "The loop retries with a fixed 3 attempts. I'll add",
            ZoomTone::Primary,
        ),
    ];
    let explain_cont = [line(
        "  exponential backoff with jitter and cap it at 5.",
        ZoomTone::Primary,
    )];
    let edit_retry = [
        line("● ", ZoomTone::Secondary),
        line("Edit ", ZoomTone::Primary),
        line("src/settlement/retry.rs", ZoomTone::Secondary),
    ];
    let plus_const = [
        line("  +  ", ZoomTone::Ok),
        line("const MAX_ATTEMPTS: u32 = 5;", ZoomTone::Secondary),
    ];
    let plus_delay = [
        line("  +  ", ZoomTone::Ok),
        line(
            "let delay = BASE * 2u32.pow(attempt) + jitter();",
            ZoomTone::Secondary,
        ),
    ];
    let minus_loop = [
        line("  -  ", ZoomTone::Danger),
        line("for attempt in 0..3 {", ZoomTone::Muted),
    ];
    let plus_loop = [
        line("  +  ", ZoomTone::Ok),
        line("for attempt in 0..MAX_ATTEMPTS {", ZoomTone::Secondary),
    ];
    let bash = [
        line("● ", ZoomTone::Secondary),
        line("Bash ", ZoomTone::Primary),
        line("cargo test -p settlement retry", ZoomTone::Secondary),
    ];
    let bash_out = [line(
        "  running 6 tests … 6 passed (1.42 s)",
        ZoomTone::Muted,
    )];
    let edit_mod = [
        line("● ", ZoomTone::Secondary),
        line("Edit ", ZoomTone::Primary),
        line("src/settlement/mod.rs", ZoomTone::Secondary),
    ];
    let plus_use = [
        line("  +  ", ZoomTone::Ok),
        line(
            "pub use retry::{MAX_ATTEMPTS, RetryPolicy};",
            ZoomTone::Secondary,
        ),
    ];
    let done = [
        line("● ", ZoomTone::Secondary),
        line(
            "Retries now back off 250 ms → 4 s, capped at 5 tries.",
            ZoomTone::Primary,
        ),
    ];
    let done_cont = [line(
        "  One more edit: expose the policy in settlement config.",
        ZoomTone::Primary,
    )];
    let allow = [
        line("▶ ", ZoomTone::Warning),
        line(
            "Allow edit to src/settlement/config.rs? (y/n)",
            ZoomTone::Primary,
        ),
    ];
    let prompt = [line("❯ ", ZoomTone::Secondary)];
    let empty: [ZoomSpan<'_>; 0] = [];
    let think = [
        line("● ", ZoomTone::Secondary),
        line("I'll read the retry loop first.", ZoomTone::Primary),
    ];
    let lines: [&[ZoomSpan<'_>]; 32] = [
        &banner,
        &empty,
        &user,
        &user_cont,
        &empty,
        &think,
        &empty,
        &read_retry,
        &read_mod,
        &empty,
        &explain,
        &explain_cont,
        &empty,
        &edit_retry,
        &plus_const,
        &plus_delay,
        &minus_loop,
        &plus_loop,
        &empty,
        &bash,
        &bash_out,
        &empty,
        &edit_mod,
        &plus_use,
        &empty,
        &done,
        &done_cont,
        &empty,
        &allow,
        &prompt,
        &empty,
        &empty,
    ];
    let status = [
        line(" ", ZoomTone::Primary),
        bold_line("PR #482 · Settlement retry backoff", ZoomTone::Primary),
        line("           ", ZoomTone::Primary),
        line("Claude Code · Work · needs input", ZoomTone::Warning),
        line("            ", ZoomTone::Primary),
        line("…", ZoomTone::Muted),
        line("   ", ZoomTone::Primary),
        line("Session", ZoomTone::Muted),
        line(" ", ZoomTone::Primary),
        line("╍╍╍╍╍╍╍╍", ZoomTone::Warning),
        line("───", ZoomTone::Faint),
        line(" ", ZoomTone::Primary),
        italic_line("76%", ZoomTone::Muted),
        line(" ▲", ZoomTone::Warning),
        line(" ", ZoomTone::Primary),
    ];
    let hints = [
        ZoomHint {
            key: "Ctrl+B",
            action: "Pane commands…",
        },
        ZoomHint {
            key: "F10",
            action: "Menu",
        },
        ZoomHint {
            key: "Ctrl+\\",
            action: "Palette",
        },
        ZoomHint {
            key: "Alt+Shift+↑↓←→",
            action: "Resize",
        },
    ];
    termrock::feedback::paint_zoomed_pane(
        ui,
        ui.full(),
        &ZoomedPane {
            brand: "jackin❯",
            menus: &menus,
            crumb: "payments-platform › the-architect",
            tabs: &tabs,
            title: " Claude Code (Work) ",
            title_mark: "●",
            zoom_label: " zoomed ",
            lines: &lines,
            status: &status,
            hints: &hints,
            hint_overflow: "…",
            zoom_hint: "Zoomed · z restores the layout",
        },
    );
}
