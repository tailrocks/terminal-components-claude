//! TASK-013: one logical grapheme and editor grammar governs display,
//! movement and source-preserving copy.
//!
//! Segmented styled fragments paint as one logical line with first-byte
//! style ownership; every editor entry point shares one normalization
//! invariant; selection floors and edits repair whole graphemes; word
//! motion follows alphanumeric grapheme runs; replacement reports the
//! whole transaction; fuzzy matching compares lowercased scalars and
//! reports original-grapheme ordinals with byte-based penalties; overwide
//! graphemes wrap to ellipsis rows.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines
    )
)]

use junie_tui::{
    App, ColorLevel, Cx, EditAction, EditOutcome, Extend, FuzzyBoundary, Motion, Position, Rect,
    Response, Role, Span, StylePatch, Surface, TextBuffer, TextEditorCore, Theme, Ui,
    fuzzy_with_boundary, width, wrap, wrapped_rows,
};
use junie_tui_testing::Harness;
use junie_tui_testing::perf;
use ratatui_core::buffer::CellWidth;
use std::cell::Cell;
use unicode_segmentation::UnicodeSegmentation;

#[global_allocator]
static GLOBAL: perf::Counting = perf::Counting;

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn themes() -> [Theme; 2] {
    [Theme::junie(), Theme::paper()]
}

/// Split-fragment page: row 0 paints the independent reference, row 1
/// paints the same logical line through `paint_spans` with the fragment
/// boundary at `split`. The styled reference segments the joined line
/// directly, attributes each cluster to its first byte's span, and paints
/// one complete cluster per call; the unstyled reference paints the whole
/// line at once.
struct SplitPage {
    text: String,
    split: usize,
    styled: bool,
    empties: bool,
    width: u16,
    clipped: bool,
    used: Cell<(u16, u16)>,
}

impl SplitPage {
    fn fragments(&self) -> Vec<Span<'_>> {
        let (first, second) = (
            self.text.get(..self.split).unwrap_or(""),
            self.text.get(self.split..).unwrap_or(""),
        );
        if !self.styled {
            return vec![Span::new(first), Span::new(second)];
        }
        let pair = vec![
            Span::new(first).role(Role::Accent),
            Span::new(second).role(Role::Danger),
        ];
        if !self.empties {
            return pair;
        }
        vec![
            Span::new(""),
            Span::new(first).role(Role::Success),
            Span::new(""),
            Span::new(second).role(Role::Info),
            Span::new(""),
        ]
    }

    fn paint_rows(&self, ui: &mut Ui<'_>, area: Rect) {
        let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
        let reference = Rect::new(area.x, 0, area.width, 1);
        let split = Rect::new(area.x, 1, area.width, 1);
        let ref_used = if self.styled {
            // Byte ranges owned by each fragment over the joined line.
            let mut ranges = Vec::new();
            let mut at = 0usize;
            let fragments = self.fragments();
            for span in &fragments {
                ranges.push((at, at + span.text.len()));
                at += span.text.len();
            }
            let mut col = reference.x;
            for (index, cluster) in self.text.grapheme_indices(true) {
                if col >= reference.right() {
                    break;
                }
                let owner = ranges
                    .iter()
                    .position(|(lo, hi)| *lo <= index && index < *hi)
                    .unwrap_or(0);
                let mut style = base;
                if let Some(role) = fragments.get(owner).and_then(|s| s.role) {
                    style = style.patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
                }
                let cell = Rect::new(col, 0, reference.right() - col, 1);
                let before = col;
                col += ui.paint_str(cell, cluster, style);
                if col == before
                    && !cluster.contains(char::is_control)
                    && cluster.cell_width() > cell.width
                {
                    break;
                }
            }
            col - reference.x
        } else {
            ui.paint_str(reference, &self.text, base)
        };
        let split_used = ui.paint_spans(split, &self.fragments(), base);
        self.used.set((ref_used, split_used));
    }
}

impl App for SplitPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.with_surface(Surface::FieldHover, |ui| {
            if self.clipped {
                // Both rows share the narrowed clip; ancestor intersection
                // still applies inside every painter call.
                ui.with_area(Rect::new(4, 0, 5, 2), |ui| {
                    self.paint_rows(ui, Rect::new(4, 0, self.width.min(5), 2));
                });
            } else {
                self.paint_rows(ui, Rect::new(1, 0, self.width, 2));
            }
        });
    }
}

impl SplitPage {
    fn harness(self, theme: Theme, level: ColorLevel) -> Harness<Self> {
        Harness::new(self, theme.for_level(level), 40, 2)
    }

    fn new(
        text: &str,
        split: usize,
        styled: bool,
        empties: bool,
        width: u16,
        clipped: bool,
    ) -> Self {
        SplitPage {
            text: text.to_string(),
            split,
            styled,
            empties,
            width,
            clipped,
            used: Cell::new((0, 0)),
        }
    }
}

/// Every split of every corpus line paints exactly its reference row:
/// combining marks, ZWJ sequences and regional-indicator pairs stay
/// indivisible with first-byte style ownership, across themes, color
/// modes, widths, clips and empty fragments.
#[test]
fn split_clusters_paint_indivisibly_with_first_byte_style() {
    let corpus = [
        "e\u{301}x",
        "👩‍💻ab",
        "🇺🇸x",
        "a\u{301}\u{301}b",
        "界x",
        "\r\nX",
        "a\tb",
        "plain",
    ];
    for theme in themes() {
        for level in LEVELS {
            for text in corpus {
                let splits: Vec<usize> = text
                    .char_indices()
                    .map(|(i, _)| i)
                    .chain([text.len()])
                    .collect();
                for split in splits {
                    for (styled, empties) in [(false, false), (true, false), (true, true)] {
                        for width in [0u16, 1, 2, 3, 5, 8, 20] {
                            for clipped in [false, true] {
                                let h =
                                    SplitPage::new(text, split, styled, empties, width, clipped)
                                        .harness(theme.clone(), level);
                                for x in 0..40 {
                                    assert_eq!(
                                        h.cell(x, 0),
                                        h.cell(x, 1),
                                        "{text:?} split {split} styled={styled} empties={empties} \
                                         width {width} clipped {clipped} {level:?} x{x}"
                                    );
                                }
                                let (ref_used, split_used) = h.app().used.get();
                                assert_eq!(ref_used, split_used);
                                assert!(h.diagnostics().is_empty());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The styled sweep above discriminates ownership only where the two
/// fragment roles resolve differently: prove they do in TrueColor, so a
/// last-byte-style mutant cannot hide behind collapsed styles.
#[test]
fn fragment_roles_resolve_distinctly_in_truecolor() {
    struct Probe;
    impl App for Probe {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            let accent = base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
            let danger = base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Danger)));
            ui.paint_str(Rect::new(0, 0, 1, 1), "a", accent);
            ui.paint_str(Rect::new(1, 0, 1, 1), "a", danger);
        }
    }
    let h = Harness::new(Probe, Theme::junie(), 4, 1);
    assert_ne!(h.cell(0, 0).fg, h.cell(1, 0).fg);
    assert!(h.diagnostics().is_empty());
}

/// A per-span segmenter paints `e` then drops or isolates the combining
/// acute; the shared projection paints one `e\u{301}` cluster. Byte
/// coordinates would bold the wrong cells from fuzzy ordinals.
#[test]
fn per_span_segmentation_and_byte_coordinates_are_rejected() {
    struct Page;
    impl App for Page {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            let spans = [Span::new("e"), Span::new("\u{301}x")];
            ui.paint_spans(Rect::new(0, 0, 4, 1), &spans, base);
            // Fuzzy ordinals feed `paint_matched` directly: `ün` in the
            // Turkish label is graphemes 9 and 10, bytes 11 and 13.
            let label = "İstanbul_Ünïted";
            let (_, ordinals) = junie_tui::fuzzy(label, "ün").unwrap_or((0, Vec::new()));
            assert_eq!(ordinals, vec![9, 10]);
            ui.paint_matched(Rect::new(0, 1, 20, 1), label, &ordinals, base);
        }
    }
    let h = Harness::new(Page, Theme::junie(), 24, 2);
    assert_eq!(h.cell(0, 0).symbol(), "e\u{301}");
    assert_eq!(h.cell(1, 0).symbol(), "x");
    // Graphemes 9 (`Ü`) and 10 (`n`) are bold; byte 11/13 neighbors are not.
    let bold_at: Vec<u16> = (0..15)
        .filter(|x| h.cell(*x, 1).modifier.contains(junie_tui::Modifier::BOLD))
        .collect();
    assert_eq!(bold_at, vec![9, 10]);
    assert!(h.diagnostics().is_empty());
}

/// Overwriting either half of a wide glyph clears the stale lead and the
/// shadow; neighbors are untouched. Clipping half a wide cell paints
/// nothing — never half a glyph.
#[test]
fn wide_overwrite_clears_lead_and_shadow_at_nonzero_origins() {
    struct Overwrite {
        first: &'static str,
        x: u16,
        second: &'static str,
        at: u16,
    }
    impl App for Overwrite {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            ui.paint_str(Rect::new(self.x, 0, 6, 1), self.first, base);
            ui.paint_str(Rect::new(self.at, 0, 6, 1), self.second, base);
        }
    }
    // Overwrite the second cell of `界` with ASCII: the stale lead resets.
    let h = Harness::new(
        Overwrite {
            first: "界",
            x: 7,
            second: "a",
            at: 8,
        },
        Theme::junie(),
        16,
        1,
    );
    assert_eq!(h.cell(7, 0).symbol(), " ");
    assert_eq!(h.cell(8, 0).symbol(), "a");
    assert_eq!(h.cell(6, 0).symbol(), " ");
    assert_eq!(h.cell(9, 0).symbol(), " ");
    // Overwrite the lead cell instead: the shadow resets with it.
    let h = Harness::new(
        Overwrite {
            first: "界",
            x: 7,
            second: "a",
            at: 7,
        },
        Theme::junie(),
        16,
        1,
    );
    assert_eq!(h.cell(7, 0).symbol(), "a");
    assert_eq!(h.cell(8, 0).symbol(), " ");
    // A wide glyph clipped to one column paints nothing at all.
    struct Clip;
    impl App for Clip {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            let used = ui.paint_str(Rect::new(7, 0, 1, 1), "界x", base);
            assert_eq!(used, 0);
        }
    }
    let h = Harness::new(Clip, Theme::junie(), 16, 1);
    assert_eq!(h.cell(7, 0).symbol(), " ");
    assert_eq!(h.cell(8, 0).symbol(), " ");
    assert!(h.diagnostics().is_empty());
}

/// Tabs and controls keep display coordinates and copied source bytes
/// consistent at clipping edges: the painter skips them, the width agrees,
/// and the buffer text is untouched.
#[test]
fn tab_control_geometry_and_copy_stay_consistent_at_clip_edges() {
    struct Tabs {
        text: &'static str,
        width: u16,
    }
    impl App for Tabs {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            let used = ui.paint_str(Rect::new(1, 0, self.width, 1), self.text, base);
            assert_eq!(used, width(self.text).min(self.width));
        }
    }
    for text in ["a\tb", "aa\tb", "a\tb\tc", "\u{7}ab", "a\r\nb", "\ta"] {
        for w in [0u16, 1, 3, 4, 5, 6] {
            let h = Harness::new(Tabs { text, width: w }, Theme::junie(), 12, 1);
            assert!(h.diagnostics().is_empty());
            // Painted columns never exceed the shared width, and the source
            // text (tabs/controls included) is preserved for copying.
            let painted: String = (1..12)
                .map(|x| h.cell(x, 0).symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string();
            assert!(width(&painted) <= width(text));
        }
    }
    // Copy preserves original bytes: selection over the buffer returns the
    // source text with tabs and controls, never display text.
    let mut b = TextBuffer::single("a\tb");
    b.select_range(0, 3);
    assert_eq!(b.selected_text(), Some("a\tb"));
}

/// One normalization invariant governs every mutation entrance:
/// constructors, `set_text`, character insertion and paste. Single-line
/// storage never retains a line break; multiline folds CRLF/CR to LF
/// without manufacturing double breaks.
#[test]
fn normalization_holds_at_every_entry_point() {
    for multiline in [false, true] {
        for input in ["", "a\nb", "a\r\nb", "a\rb", "a\r\nb\rc\nd\r\r\n"] {
            let make = |s: &str| {
                if multiline {
                    TextBuffer::multi(s)
                } else {
                    TextBuffer::single(s)
                }
            };
            // Constructor.
            let b = make(input);
            if multiline {
                assert_eq!(b.text(), &input.replace("\r\n", "\n").replace('\r', "\n"));
            } else {
                assert!(!b.text().contains(['\r', '\n']), "{input:?}");
                assert_eq!(b.text(), &input.replace(['\r', '\n'], ""));
            }
            // `set_text`.
            let mut b = make("");
            b.set_text(input);
            if multiline {
                assert_eq!(b.text(), &input.replace("\r\n", "\n").replace('\r', "\n"));
            } else {
                assert!(!b.text().contains(['\r', '\n']), "{input:?}");
            }
            assert_eq!(b.cursor_offset(), b.text().len());
            // Character insertion: each keystroke folds independently, so
            // a lone `\r` becomes `\n` exactly like the oracle's
            // per-character entrance (a CRLF typed as two keystrokes is two
            // line breaks; only string entrances join the pair).
            let mut b = make("");
            for c in input.chars() {
                b.insert_char(c);
            }
            if multiline {
                let keyed: String = input
                    .chars()
                    .map(|c| if c == '\r' { '\n' } else { c })
                    .collect();
                assert_eq!(b.text(), &keyed, "{input:?}");
            } else {
                assert!(!b.text().contains(['\r', '\n']), "{input:?}");
            }
            // Paste.
            let mut b = make("");
            b.insert_str(input);
            if multiline {
                assert_eq!(b.text(), &input.replace("\r\n", "\n").replace('\r', "\n"));
            } else {
                assert!(!b.text().contains(['\r', '\n']), "{input:?}");
            }
        }
    }
    // Lone `\r` folds to `\n` in multiline insertion; both reject single-line.
    let mut m = TextBuffer::multi("");
    assert!(m.insert_char('\r'));
    assert_eq!(m.text(), "\n");
    let mut s = TextBuffer::single("a");
    assert!(!s.insert_char('\r'));
    assert!(!s.insert_char('\n'));
    assert_eq!(s.text(), "a");
}

/// Requested selection endpoints floor to whole graphemes; after an atomic
/// edit the cursor ceils and a retained anchor floors against the new
/// string; replacement clears the anchor. Covers every byte and scalar
/// boundary in both directions, with merges that join clusters.
#[test]
fn selection_floors_and_edits_repair_clusters() {
    let fam = "👨‍👩‍👧‍👦";
    for text in [
        "e\u{301}x".to_string(),
        format!("{fam}!"),
        "plain".to_string(),
    ] {
        // Independent grapheme boundaries of the source text.
        let bounds: Vec<usize> = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([text.len()])
            .collect();
        // Every byte offset floors; every endpoint pair selects whole
        // clusters in both directions.
        for a in 0..=text.len() {
            for b in 0..=text.len() {
                let mut buf = TextBuffer::single(text.as_str());
                buf.select_range(a, b);
                if buf.selection().is_none() {
                    // Collapsed endpoints still rest on a boundary.
                    assert!(bounds.contains(&buf.cursor_offset()));
                    continue;
                }
                let range = buf.selection().unwrap();
                assert!(
                    bounds.contains(&range.start) && bounds.contains(&range.end),
                    "selection {range:?} splits a cluster for request ({a}, {b})"
                );
                // The selected text re-segments to whole source clusters.
                let selected = buf.selected_text().unwrap();
                assert!(selected.graphemes(true).count() >= 1);
                assert!(text.contains(selected));
            }
        }
    }
    {
        // Mid-cluster requests floor to the preceding boundary.
        let mut buf = TextBuffer::single("e\u{301}x");
        buf.select_range(1, 3);
        assert_eq!(buf.selected_text(), Some("e\u{301}"));
        buf.select_range(3, 1);
        assert_eq!(buf.selected_text(), Some("e\u{301}"));
        buf.select_range(2, 2);
        assert!(buf.selection().is_none());
        assert_eq!(buf.cursor_offset(), 0);
        // Inserting a combining mark merges clusters: the cursor ceils past
        // the joined cluster against the new string.
        let mut buf = TextBuffer::single("ex");
        buf.select_range(1, 1);
        buf.insert_char('\u{301}');
        assert_eq!(buf.text(), "e\u{301}x");
        assert_eq!(buf.cursor_offset(), 3);
        buf.backspace();
        assert_eq!(buf.text(), "x");
        // Inserting ZWJ between emoji joins them; the cursor leaves the
        // joined cluster instead of resting inside it.
        let mut buf = TextBuffer::single("👨👩");
        buf.select_range(4, 4);
        buf.insert_char('\u{200d}');
        assert_eq!(buf.text(), "👨\u{200d}👩");
        assert!(buf.text().graphemes(true).count() == 1);
        assert_eq!(buf.cursor_offset(), buf.text().len());
        // Replacement clears the anchor; the next selection starts fresh.
        let mut buf = TextBuffer::single("hello");
        buf.select_range(0, 5);
        buf.insert_str("bye");
        assert_eq!(buf.text(), "bye");
        assert!(buf.selection().is_none());
        buf.move_left(true);
        assert_eq!(buf.selected_text(), Some("e"));
    }
}

/// Word motion and deletion follow alphanumeric grapheme runs through one
/// shared walk: skips separators, consumes the run, in both selection
/// directions. `_` splits words; combining marks ride with their base.
#[test]
fn word_grammar_follows_alphanumeric_graphemes() {
    // `snake_case` is three runs: `snake`, `case`, and the `-kebab` tail.
    let mut b = TextBuffer::single("snake_case-kebab");
    b.move_word_left(false);
    assert_eq!(b.cursor_offset(), "snake_case-".len());
    b.move_word_left(false);
    assert_eq!(b.cursor_offset(), "snake_".len());
    b.move_word_left(false);
    assert_eq!(b.cursor_offset(), 0);
    b.move_word_right(false);
    assert_eq!(b.cursor_offset(), "snake".len());
    b.move_word_right(false);
    assert_eq!(b.cursor_offset(), "snake_case".len());
    // Selection extends in both directions over the same runs.
    b.move_word_left(true);
    assert_eq!(b.selected_text(), Some("case"));
    b.move_word_right(true);
    assert!(b.selection().is_none());
    // Combining marks ride with their base next to punctuation.
    let mut b = TextBuffer::single("e\u{301},x");
    b.move_home(false);
    b.move_word_right(false);
    assert_eq!(b.cursor_offset(), "e\u{301}".len());
    b.move_word_right(false);
    assert_eq!(b.cursor_offset(), "e\u{301},x".len());
    // Repeated spaces collapse as separators, not words: deleting the
    // previous word from the start of `beta` removes `alpha` and the gap.
    let mut b = TextBuffer::single("alpha  beta");
    b.move_word_left(false);
    assert_eq!(b.cursor_offset(), 7);
    b.delete_word_left();
    assert_eq!(b.text(), "beta");
    assert_eq!(b.cursor_offset(), 0);
}

/// Selection deletion is a real mutation even when the inserted text is
/// empty or filters to nothing; an empty paste with no selection reports
/// no change. Observed as state and as the typed edit outcome.
#[test]
fn empty_paste_reports_the_whole_transaction() {
    // Nonempty selection plus empty paste: text changes, outcome Changed.
    let mut b = TextBuffer::single("hello");
    b.select_range(0, 5);
    assert!(b.insert_str(""));
    assert_eq!(b.text(), "");
    let mut e = TextEditorCore::single("hello");
    e.select_range(0, 5);
    assert_eq!(e.apply(EditAction::Paste("")), EditOutcome::Changed);
    assert_eq!(e.text(), "");
    // Fully filtered paste over a selection still deletes the selection.
    let mut e = TextEditorCore::single("hello");
    e.select_range(1, 4);
    assert_eq!(e.apply(EditAction::Paste("\n\r")), EditOutcome::Changed);
    assert_eq!(e.text(), "ho");
    // Empty paste with no selection: no change, no fabricated outcome.
    let mut b = TextBuffer::single("hello");
    assert!(!b.insert_str(""));
    assert_eq!(b.text(), "hello");
    let mut e = TextEditorCore::single("hello");
    assert_eq!(e.apply(EditAction::Paste("")), EditOutcome::Ignored);
    // A newline-only paste into a multiline buffer is a real insertion.
    let mut m = TextEditorCore::multi("ab");
    m.select_range(1, 1);
    assert_eq!(m.apply(EditAction::Paste("\r\n")), EditOutcome::Changed);
    assert_eq!(m.text(), "a\nb");
}

/// The shared grammar preserves the oracle's deferred Ctrl+Home/End
/// routing shape: document motions accept both extends, line motions
/// extend under Shift, and word motions keep their own branches — the
/// field binding selects among these primitives without new core paths.
#[test]
fn modifier_grammar_preserves_oracle_routing() {
    let mut e = TextEditorCore::multi("one\ntwo\nthree");
    e.select_range(4, 4);
    // Ctrl+Home/End route to non-extending document motions even when a
    // selection is active (deferred F08c: no Ctrl+Shift extension).
    e.select_range(0, 8);
    assert_eq!(
        e.apply(EditAction::Move(Motion::DocStart, Extend::No)),
        EditOutcome::Moved
    );
    assert_eq!(e.cursor_offset(), 0);
    assert!(e.selection().is_none());
    e.select_range(0, 8);
    assert_eq!(
        e.apply(EditAction::Move(Motion::DocEnd, Extend::No)),
        EditOutcome::Moved
    );
    assert_eq!(e.cursor_offset(), e.text().len());
    assert!(e.selection().is_none());
    // Shift+Home/End extend; Ctrl+Shift+Left/Right keep their own
    // word branches through the same primitives.
    e.select_range(5, 5);
    assert_eq!(
        e.apply(EditAction::Move(Motion::Home, Extend::Select)),
        EditOutcome::Moved
    );
    assert_eq!(e.selected_text(), Some("t"));
    // The anchor is retained across motions: extending to line end from
    // the kept anchor selects the rest of the line.
    assert_eq!(
        e.apply(EditAction::Move(Motion::End, Extend::Select)),
        EditOutcome::Moved
    );
    assert_eq!(e.selected_text(), Some("wo"));
    e.select_range(9, 9);
    assert_eq!(
        e.apply(EditAction::Move(Motion::WordLeft, Extend::Select)),
        EditOutcome::Moved
    );
    assert_eq!(e.selected_text(), Some("t"));
}

/// Fuzzy matching compares Unicode scalars after whole-string lowercase
/// and reports original-grapheme ordinals with source byte-based
/// subsequence penalties. `Word` keeps its space/hyphen bonus;
/// `Identifier` is the exact oracle lane (underscore/dot only).
#[test]
fn fuzzy_matches_scalars_and_reports_grapheme_ordinals() {
    // Greek final sigma: whole-string lowercase matches; ordinals project.
    assert_eq!(
        fuzzy_with_boundary("ΟΣ", "ος", FuzzyBoundary::Word),
        Some((0, vec![0, 1]))
    );
    // A combining-scalar query matches its whole source grapheme.
    assert_eq!(
        fuzzy_with_boundary("a\u{301}b", "\u{301}", FuzzyBoundary::Word),
        Some((30, vec![0]))
    );
    // Byte offsets and grapheme ordinals diverge: the subsequence penalty
    // is byte-based (60 + byte 4 of `c`), the highlights are ordinals.
    assert_eq!(
        fuzzy_with_boundary("a\u{301}bc", "ac", FuzzyBoundary::Word),
        Some((64, vec![0, 2]))
    );
    // Turkish dotted capital keeps its original ordinals and boundary bonus.
    assert_eq!(
        fuzzy_with_boundary("İstanbul_Ünïted", "ün", FuzzyBoundary::Word),
        Some((10, vec![9, 10]))
    );
    // Boundary policy: Word bonuses space/hyphen/underscore/dot,
    // Identifier only underscore/dot.
    for (label, word) in [("a-b", "b"), ("a b", "b"), ("a_b", "b"), ("a.b", "b")] {
        assert_eq!(
            fuzzy_with_boundary(label, word, FuzzyBoundary::Word).map(|m| m.0),
            Some(10),
            "{label:?} under Word"
        );
    }
    for (label, word, score) in [
        ("a-b", "b", 30),
        ("a b", "b", 30),
        ("a_b", "b", 10),
        ("a.b", "b", 10),
    ] {
        assert_eq!(
            fuzzy_with_boundary(label, word, FuzzyBoundary::Identifier).map(|m| m.0),
            Some(score),
            "{label:?} under Identifier"
        );
    }
    // ASCII ranking is preserved exactly.
    assert_eq!(
        fuzzy_with_boundary("alpha", "al", FuzzyBoundary::Identifier),
        Some((0, vec![0, 1]))
    );
    assert_eq!(
        fuzzy_with_boundary("alpha", "aa", FuzzyBoundary::Identifier),
        Some((64, vec![0, 4]))
    );
}

/// Per-grapheme lowercase, ordinal-based penalties and lost boundary
/// policy are rejected: each mutant produces a different observable.
#[test]
fn fuzzy_scalar_and_policy_mutants_are_rejected() {
    // Per-grapheme fold misses the final-sigma match entirely.
    assert_ne!(fuzzy_with_boundary("ΟΣ", "ος", FuzzyBoundary::Word), None,);
    // An ordinal-based subsequence penalty would report 62, not 64.
    assert_ne!(
        fuzzy_with_boundary("a\u{301}bc", "ac", FuzzyBoundary::Word).map(|m| m.0),
        Some(62),
    );
    // A lost Word policy would score the hyphen boundary 30, not 10.
    assert_ne!(
        fuzzy_with_boundary("order-items", "items", FuzzyBoundary::Word).map(|m| m.0),
        Some(30),
    );
    // A lost Identifier lane would bonus the space boundary.
    assert_ne!(
        fuzzy_with_boundary("GROUP BY", "B", FuzzyBoundary::Identifier).map(|m| m.0),
        Some(10),
    );
}

/// A grapheme wider than the whole line wraps to one ellipsis row; row
/// count and emitted rows always agree, and zero width stays safe.
#[test]
fn overwide_graphemes_wrap_to_ellipsis_rows() {
    assert_eq!(wrap("日", 1), vec!["…"]);
    assert_eq!(wrap("日", 0), vec!["…"]);
    assert_eq!(wrap("日👩‍💻e\u{301}", 1), vec!["…", "…", "e\u{301}"]);
    assert_eq!(wrap("日👩‍💻e\u{301}", 2), vec!["日", "👩‍💻", "e\u{301}"]);
    assert_eq!(wrap("", 1), vec![""]);
    assert_eq!(wrap("a\nb", 1), vec!["a", "b"]);
    for text in [
        "",
        "a",
        "aa bb cc",
        "日本語です",
        "日👩‍💻e\u{301}",
        "a\nb\nc",
        "x 日 y",
    ] {
        for w in 0..8u16 {
            let rows = wrap(text, w);
            assert_eq!(
                usize::from(wrapped_rows(text, w)),
                rows.len(),
                "{text:?} at width {w}"
            );
            for row in &rows {
                assert!(width(row) <= w.max(1), "{text:?} row {row:?} at {w}");
            }
        }
    }
}

/// Truncation (end and middle) never splits a grapheme, never overflows
/// its budget, and stays safe at zero/one-cell widths.
#[test]
fn truncation_never_splits_clusters_or_overflows() {
    for text in [
        "",
        "a",
        "hello world",
        "日本語",
        "e\u{301}x界👩‍💻",
        "very_long_identifier_name",
    ] {
        for max in 0..40u16 {
            let end = junie_tui::truncate(text, max);
            assert!(width(&end) <= max, "{text:?} end @{max}: {end:?}");
            let mid = junie_tui::truncate_middle(text, max);
            assert!(width(&mid) <= max, "{text:?} mid @{max}: {mid:?}");
            // Every emitted grapheme is a whole source cluster.
            for g in end.graphemes(true).chain(mid.graphemes(true)) {
                assert!(
                    g == "…" || text.contains(g),
                    "{text:?} @{max} splits a cluster: {g:?}"
                );
            }
        }
    }
    assert_eq!(junie_tui::truncate("hello world", 5), "hell…");
    assert_eq!(junie_tui::truncate("日本語", 3), "日…");
    assert_eq!(junie_tui::truncate("abc", 0), "");
    assert_eq!(
        junie_tui::truncate_middle("very_long_identifier_name", 12),
        "very_lon…ame"
    );
}

/// Every layout/wrap/hit/cursor path measures with the one `CellWidth`
/// source: the public width agrees with painted columns, including
/// halfwidth sound marks, ZWJ, CJK and controls.
#[test]
fn cell_width_is_the_only_measure() {
    for s in [
        "",
        "a",
        "hello",
        "ｶﾞ",
        "あ",
        "a\u{FF9E}",
        "日本語",
        "👨‍👩‍👧‍👦",
        "e\u{301}",
        "é",
        "abc\u{FF9F}x",
    ] {
        assert_eq!(width(s), s.cell_width(), "{s:?}");
    }
    // Painted columns agree with the measured width across clips.
    struct Measure {
        text: &'static str,
        width: u16,
    }
    impl App for Measure {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            let used = ui.paint_str(Rect::new(0, 0, self.width, 1), self.text, base);
            // Independent greedy expectation from direct segmentation.
            let mut remaining = self.width;
            let mut expected = 0u16;
            for g in self.text.graphemes(true) {
                if g.contains(char::is_control) {
                    continue;
                }
                let gw = g.cell_width();
                if gw == 0 || gw > remaining {
                    if gw > 0 {
                        break;
                    }
                    continue;
                }
                remaining -= gw;
                expected += gw;
            }
            assert_eq!(used, expected);
        }
    }
    for text in ["hello", "日本", "e\u{301}x", "👩‍💻", "ｶﾞ", "a\u{FF9E}b"] {
        for w in [0u16, 1, 2, 3, 5, 8] {
            let h = Harness::new(Measure { text, width: w }, Theme::junie(), 10, 1);
            assert!(h.diagnostics().is_empty());
        }
    }
}

/// The borrowed projection holds no document: painting 80 columns of a
/// 100k-grapheme line costs the same as 10k, bounded by painted columns,
/// with no whole-line materialization.
#[test]
fn borrowed_projection_holds_no_document() {
    let _guard = perf::lock();
    struct Rows {
        line: String,
    }
    impl App for Rows {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let base = ui.paint_patch(&StylePatch::new().set_bg(Role::CurrentSurface));
            // Borrowed spans over one long line, split mid-cluster: the
            // leading `e` and its combining acute live in different spans.
            let spans = [
                Span::new(self.line.get(..1).unwrap_or("")),
                Span::new(self.line.get(1..).unwrap_or("")).role(Role::Accent),
            ];
            ui.paint_spans(Rect::new(0, 0, 80, 1), &spans, base);
        }
    }
    // A cross-fragment cluster joins the two spans mid-line.
    let short = format!("e\u{301}{}z", "界".repeat(10_000));
    let long = format!("e\u{301}{}z", "界".repeat(100_000));
    let mut h = Harness::new(Rows { line: short }, Theme::junie(), 80, 1);
    h.draw();
    let before = perf::allocs();
    h.draw();
    let short_allocs = perf::allocs() - before;
    let mut h = Harness::new(Rows { line: long }, Theme::junie(), 80, 1);
    h.draw();
    let before = perf::allocs();
    h.draw();
    let long_allocs = perf::allocs() - before;
    assert_eq!(short_allocs, long_allocs, "work tracks visible columns");
    assert!(
        long_allocs <= 80,
        "bounded by painted columns, got {long_allocs}"
    );
}

/// Oracle-pinned buffer trajectories: exclusive-newline selection lines,
/// combining/CJK word runs, regional-indicator replacement positions,
/// multiline entry line-mode, and public offset clamping.
#[test]
fn oracle_buffer_trajectories_are_preserved() {
    // Exclusive endpoints at a line start do not select that line.
    let mut b = TextBuffer::multi("é\n日本\n👩‍💻");
    b.select_all();
    assert_eq!(b.selection_lines(), (0, 2));
    b.select_range(0, "é\n".len());
    assert_eq!(b.selection_lines(), (0, 0));
    b.select_range("é\n".len(), "é\n日本".len());
    assert_eq!(b.selection_lines(), (1, 1));
    // Combining clusters move as one word unit; CJK is alphanumeric.
    let mut b = TextBuffer::single("cafe\u{301} 日本");
    b.move_home(false);
    b.move_word_right(true);
    assert_eq!(b.selected_text(), Some("cafe\u{301}"));
    b.clear_selection();
    b.move_word_left(false);
    assert_eq!(b.cursor_offset(), 0);
    b.move_end(false);
    b.delete_word_left();
    b.delete_word_left();
    assert_eq!(b.text(), "");
    // Atomic replacement beside joining neighbors keeps its position.
    for paste in [false, true] {
        let mut b = TextBuffer::single("🇺 🇸");
        b.select_range(4, 5);
        if paste {
            b.insert_str("X");
        } else {
            b.insert_char('X');
        }
        assert_eq!(b.text(), "🇺X🇸");
        assert_eq!(b.cursor_offset(), 5);
    }
    // Multiline entry mode: folding, cursor motion and line count agree.
    let mut b = TextBuffer::multi("a\r\nb\rc");
    assert_eq!(b.text(), "a\nb\nc");
    b.insert_str("\r\nd");
    b.move_doc_start(false);
    b.move_end(false);
    assert_eq!(b.cursor_offset(), 1);
    assert_eq!(b.line_count(), 4);
    // Public offsets clamp to graphemes, never into UTF-8 or a cluster.
    assert_eq!(TextBuffer::pos_of("é", usize::MAX).col, 1);
    assert_eq!(TextBuffer::pos_of("é", 1).col, 0);
    let mut b = TextBuffer::single("e\u{301}日👩‍💻");
    b.select_range(1, 2);
    assert_eq!(b.cursor_offset(), 0);
    assert!(b.selection().is_none());
}

/// A scalar-only snap would select half a cluster; the floor repair keeps
/// whole graphemes, so deletion removes the whole cluster.
#[test]
fn scalar_snap_and_unrepaired_edit_mutants_are_rejected() {
    let mut buf = TextBuffer::single("e\u{301}x");
    buf.select_range(1, 3);
    assert_ne!(buf.selected_text(), Some("\u{301}"));
    assert_eq!(buf.selected_text(), Some("e\u{301}"));
    buf.delete();
    assert_eq!(buf.text(), "x");
    // Deletion-before-insertion repair would move the replacement: atomic
    // replacement keeps the cursor at the inserted end with whole-cluster
    // positions.
    let mut buf = TextBuffer::single("ab");
    buf.select_range(0, 1);
    buf.insert_str("e\u{301}");
    assert_eq!(buf.text(), "e\u{301}b");
    assert_eq!(buf.cursor_offset(), 3);
}
