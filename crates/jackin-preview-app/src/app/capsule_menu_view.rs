use termrock::feedback::{
    CapsuleMenu, CapsuleMenuFrame, ClipSpan, ClipTone, MenuEntry, ZoomHint, ZoomSpan, ZoomTab,
    ZoomTone,
};
use termrock::{Rect, Ui};

fn clip(x: u16, text: &str, tone: ClipTone, bold: bool) -> ClipSpan<'_> {
    ClipSpan {
        x,
        text,
        tone,
        bold,
    }
}
fn z(text: &str, tone: ZoomTone, bold: bool, italic: bool) -> ZoomSpan<'_> {
    ZoomSpan {
        text,
        tone,
        bold,
        italic,
    }
}

pub(super) fn paint(ui: &mut Ui<'_>) {
    paint_at(ui, ui.full());
}

pub(super) fn paint_at(ui: &mut Ui<'_>, area: Rect) {
    let r3: &[ClipSpan<'_>] = &[clip(
        30,
        "─────────────────────────────────────────────────",
        ClipTone::Rule,
        false,
    )];
    let r4: &[ClipSpan<'_>] = &[
        clip(0, "╭", ClipTone::Border, false),
        clip(30, "────────╮│", ClipTone::Border, false),
        clip(40, "╭─", ClipTone::Rule, false),
        clip(42, " Codex (Primary) ○", ClipTone::Secondary, false),
        clip(60, "───────────────────╮", ClipTone::Rule, false),
    ];
    let r5: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(30, "pow(at", ClipTone::Dim, false),
        clip(36, " ", ClipTone::Faded, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(
            41,
            "  rounding precedes FX conversion in ",
            ClipTone::Faded,
            false,
        ),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r6: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(30, "       ", ClipTone::Soft, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r7: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(30, "TEMPTS", ClipTone::Secondary, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(41, "○ ", ClipTone::Secondary, false),
        clip(43, "Done · 38 s · 12.4k tokens", ClipTone::Muted, false),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r8: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r9: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(30, "nt ret", ClipTone::Secondary, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(41, "❯ ", ClipTone::Secondary, false),
        clip(78, "┃", ClipTone::SeamHeavy, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r10: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(
            1,
            "  running 6 tests … 6 passed (1.42 ",
            ClipTone::Muted,
            false,
        ),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(41, "━━━━━━━━━━━━━━━━━━━━━━━", ClipTone::SeamHeavy, false),
        clip(64, "┄┄┄┄┄┄┄┄┄┄┄┄┄┄", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r11: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(
            40,
            "╰──────────────────────────────────────╯",
            ClipTone::Rule,
            false,
        ),
    ];
    let r12: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(1, "● ", ClipTone::Secondary, false),
        clip(3, "Edit ", ClipTone::Primary, false),
        clip(8, "src/settlement/mod.rs", ClipTone::Secondary, false),
        clip(37, "┆", ClipTone::Seam, false),
        clip(
            38,
            "││────────────────────────────────────────",
            ClipTone::Border,
            false,
        ),
    ];
    let r13: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(1, "  +  ", ClipTone::Ok, false),
        clip(
            6,
            "pub use retry::{MAX_ATTEMPTS, ",
            ClipTone::Secondary,
            false,
        ),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "╭─", ClipTone::Rule, false),
        clip(42, " Shell ", ClipTone::Secondary, false),
        clip(49, "──────────────────────────────╮", ClipTone::Rule, false),
    ];
    let r14: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(
            41,
            "    Finished dev [unoptimized] targe",
            ClipTone::Shell,
            false,
        ),
        clip(77, " ", ClipTone::Faded, false),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r15: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(1, "● ", ClipTone::Secondary, false),
        clip(
            3,
            "Retries now back off 250 ms → 4 s ",
            ClipTone::Primary,
            false,
        ),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(41, "payments-platform ❯ ", ClipTone::Secondary, false),
        clip(61, "ls docs/adr      ", ClipTone::Primary, false),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r16: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(
            3,
            "One more edit: expose the policy  ",
            ClipTone::Primary,
            false,
        ),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(
            41,
            "0001-record-architecture.md  0004-le ",
            ClipTone::Primary,
            false,
        ),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r17: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(
            41,
            "0002-settlement-batches.md   0005-fx ",
            ClipTone::Primary,
            false,
        ),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r18: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(1, "▶ ", ClipTone::Warning, false),
        clip(
            3,
            "Allow edit to src/settlement/conf ",
            ClipTone::Primary,
            false,
        ),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(
            41,
            "0003-retry-policy.md         0007-re ",
            ClipTone::Primary,
            false,
        ),
        clip(78, "┆", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r19: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(1, "❯ ", ClipTone::Secondary, false),
        clip(37, "┃", ClipTone::SeamHeavy, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(41, "payments-platform ❯ ", ClipTone::Secondary, false),
        clip(78, "┃", ClipTone::SeamHeavy, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r20: &[ClipSpan<'_>] = &[
        clip(0, "│", ClipTone::Border, false),
        clip(1, "━━━━━━━━━━━━━━━━━━━━━━", ClipTone::SeamHeavy, false),
        clip(23, "┄┄┄┄┄┄┄┄┄┄┄┄┄┄", ClipTone::Seam, false),
        clip(38, "││", ClipTone::Border, false),
        clip(40, "│", ClipTone::Rule, false),
        clip(41, "━━━━━━━━━━━━━━━━━━", ClipTone::SeamHeavy, false),
        clip(59, "┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄", ClipTone::Seam, false),
        clip(79, "│", ClipTone::Rule, false),
    ];
    let r21: &[ClipSpan<'_>] = &[
        clip(
            0,
            "╰─────────────────────────────────────╯│",
            ClipTone::Border,
            false,
        ),
        clip(
            40,
            "╰──────────────────────────────────────╯",
            ClipTone::Rule,
            false,
        ),
    ];
    let rows: [&[ClipSpan<'_>]; 19] = [
        r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17, r18, r19, r20, r21,
    ];
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
    let entries = [
        MenuEntry {
            label: "Change title…",
            shortcut: "2×click",
            danger: false,
            selected: true,
        },
        MenuEntry {
            label: "Split right",
            shortcut: "Ctrl+B %",
            danger: false,
            selected: false,
        },
        MenuEntry {
            label: "Close tab",
            shortcut: "Ctrl+B &",
            danger: true,
            selected: false,
        },
    ];
    let status = [
        z(" ", ZoomTone::Primary, false, false),
        z(
            "PR #482 · Settlement retry backoff",
            ZoomTone::Primary,
            true,
            false,
        ),
        z("               ", ZoomTone::Primary, false, false),
        z("…", ZoomTone::Muted, false, false),
        z("   ", ZoomTone::Primary, false, false),
        z("Session", ZoomTone::Muted, false, false),
        z(" ", ZoomTone::Primary, false, false),
        z("╍╍╍╍╍╍╍╍", ZoomTone::Warning, false, false),
        z("───", ZoomTone::Faint, false, false),
        z(" ", ZoomTone::Primary, false, false),
        z("76%", ZoomTone::Muted, false, true),
        z(" ▲", ZoomTone::Warning, false, false),
        z(" ", ZoomTone::Primary, false, false),
    ];
    let hints = [ZoomHint {
        key: "Esc",
        action: "Close",
    }];
    // Header menus follow the live capsule menu bar. Rows, entries, tabs and
    // status still need App instance state, which this view cannot reach
    // through its `Ui`-only call sites in `app.rs`.
    let menus: Vec<&str> = super::CAPSULE_MENUS.iter().map(|menu| menu.label).collect();
    termrock::feedback::paint_capsule_menu(
        ui,
        area,
        &CapsuleMenuFrame {
            brand: "jackin❯",
            menus: &menus,
            overflow: "…",
            tabs: &tabs,
            menu: CapsuleMenu {
                title: "Mix (3)",
                entries: &entries,
                separator_after: 2,
            },
            rows: &rows,
            status: &status,
            hints: &hints,
            hint_overflow: "…",
            attached: "Attached to payments-platform · tabs and panes restored",
        },
    );
}
