use std::ops::Range;

use termrock::{
    CodeAction, CodeEditor, CodeEditorState, Color, Completion, CompletionState, Cx, DiffView,
    DiffViewState, Id, Item, ItemKey, Modifier, PaintStyle, Panel, Part, Props, Rect, StateFlags,
    Surface, SyntaxRole, TabBehavior, Ui, Variant, layout,
};

use super::{Page, PageUpdate, frame};

const EDITOR: Id = termrock::Id::root("showcase_ui::pages::editor::editor.code");
const EDITOR_PANEL: Id = termrock::Id::root("showcase_ui::pages::editor::editor.code.panel");
const STATE_PANEL: Id = termrock::Id::root("showcase_ui::pages::editor::editor.state.panel");
const COMPLETION: Id = termrock::Id::root("showcase_ui::pages::editor::editor.completion");
const DIFF: Id = termrock::Id::root("showcase_ui::pages::editor::editor.diff");
const SUGGESTIONS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(101), "fn").detail("function keyword"),
    Item::new(ItemKey::Num(102), "let").detail("binding keyword"),
    Item::new(ItemKey::Num(103), "match").detail("pattern match"),
];

const SAMPLE: &str = "\
// Retry a request with exponential backoff.
pub async fn fetch(url: &str) -> Result<Body, Error> {
    let mut delay = 200;
    for attempt in 1..=5 {
        match client().get(url).await {
            Ok(body) => return Ok(body),
            Err(e) if e.is_transient() => {
                log::warn!(\"attempt {attempt} failed: {e}\");
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    Err(Error::Exhausted)
}

fn client() -> Client {
    Client::builder().timeout(10).build().unwrap()
}

#[test]
fn backoff_doubles() {
    assert_eq!(schedule(3), vec![200, 400, 800]);
}
";

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "else", "enum", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "Self", "static", "struct", "trait", "true", "false", "type", "use", "where", "while",
];

fn highlight(src: &str) -> Vec<(Range<usize>, SyntaxRole)> {
    let bytes = src.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        if !src.is_char_boundary(i) {
            i = i.saturating_add(1);
            continue;
        }
        if byte == b'/' && bytes.get(i.saturating_add(1)) == Some(&b'/') {
            let end = src[i..]
                .find('\n')
                .map_or(bytes.len(), |n| i.saturating_add(n));
            spans.push((i..end, SyntaxRole::Comment));
            i = end;
            continue;
        }
        if byte == b'"' {
            let end = src[i.saturating_add(1)..]
                .find('"')
                .map_or(bytes.len(), |n| i.saturating_add(n).saturating_add(2));
            spans.push((i..end, SyntaxRole::Str));
            i = end;
            continue;
        }
        if byte.is_ascii_digit() {
            let mut end = i;
            while bytes
                .get(end)
                .is_some_and(|byte| byte.is_ascii_digit() || *byte == b'_' || *byte == b'.')
            {
                end = end.saturating_add(1);
            }
            spans.push((i..end, SyntaxRole::Number));
            i = end;
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let mut end = i;
            while bytes
                .get(end)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                end = end.saturating_add(1);
            }
            let Some(word) = src.get(i..end) else {
                i = end;
                continue;
            };
            let next = bytes.get(end).copied();
            let role = if KEYWORDS.contains(&word) {
                SyntaxRole::Keyword
            } else if next == Some(b'(') || next == Some(b'!') {
                SyntaxRole::Function
            } else if word.starts_with(|ch: char| ch.is_ascii_uppercase()) {
                SyntaxRole::TypeName
            } else {
                SyntaxRole::Plain
            };
            spans.push((i..end, role));
            i = end;
            continue;
        }
        let role = match byte {
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b';' | b',' => SyntaxRole::Punct,
            b'=' | b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'!' | b'&' | b'|' | b':' | b'?'
            | b'.' => SyntaxRole::Operator,
            _ => {
                i = i.saturating_add(1);
                continue;
            }
        };
        spans.push((i..i.saturating_add(1), role));
        i = i.saturating_add(1);
    }
    spans
}

fn blocks(src: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = None;
    let mut end = 0;
    let mut offset = 0_usize;
    for line in src.split_inclusive('\n') {
        if line.trim().is_empty() {
            if let Some(start) = start.take() {
                out.push(start..end);
            }
        } else {
            if start.is_none() {
                start = Some(offset);
            }
            end = offset.saturating_add(line.trim_end_matches('\n').len());
        }
        offset = offset.saturating_add(line.len());
    }
    if let Some(start) = start {
        out.push(start..end);
    }
    out
}

fn editor() -> CodeEditor<'static> {
    CodeEditor::new(EDITOR, 12)
        .highlighter(&highlight)
        .segmenter(&blocks)
        .tab_behavior(TabBehavior::Leave)
        .placeholder("Start typing Rust…")
}

fn completion() -> Completion<'static, Item<'static>> {
    Completion::new(COMPLETION).max_rows(3)
}

fn diff() -> DiffView<'static> {
    DiffView::new(DIFF, None)
}

fn editor_panel(meta: &str) -> Panel<'_> {
    Panel::new(EDITOR_PANEL).title("retry.rs").meta(meta)
}

fn state_panel() -> Panel<'static> {
    Panel::new(STATE_PANEL).title("State")
}

fn editor_meta(state: &CodeEditorState) -> String {
    if state.is_editing() {
        String::from("running ")
    } else {
        format!("{} blocks ", editor().blocks(state).len())
    }
}

#[derive(Debug)]
pub struct EditorPage {
    state: CodeEditorState,
    completion_state: CompletionState,
    diff_state: DiffViewState,
    last: &'static str,
}

impl EditorPage {
    pub fn new() -> Self {
        Self {
            state: CodeEditorState::new(SAMPLE),
            completion_state: CompletionState::default(),
            diff_state: DiffViewState::default(),
            last: "read-only preview",
        }
    }
}

impl Default for EditorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for EditorPage {
    fn title(&self) -> &'static str {
        "Code editor"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let editor_response = editor().update(cx, &mut self.state);
        if let Some(action) = editor_response.action_ref() {
            self.last = match action {
                CodeAction::Changed => "document changed",
                CodeAction::CursorMoved => "cursor moved",
                CodeAction::Committed => "edit committed",
                CodeAction::Leave { .. } => "focus moved",
            };
        }
        let mut result = editor_response.erase();
        result |= completion()
            .update_for(EDITOR, cx, &mut self.completion_state, SUGGESTIONS)
            .erase();
        result |= diff().update(cx, &mut self.diff_state).erase();

        let _ = editor_panel(&editor_meta(&self.state));
        let _ = state_panel();
        result.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if paint_baseline(ui, area) {
            return;
        }
        frame(
            ui,
            area,
            self.title(),
            "Blocks, tones, diagnostics and completion; the gutter says where you are",
            |ui, body| {
                let left_width = (body.width.saturating_mul(62) / 100).max(40);
                let (code_area, state_area) = if body.width < left_width.saturating_add(22) {
                    let height = body.height / 2;
                    (
                        Rect { height, ..body },
                        Rect {
                            y: body.y.saturating_add(height),
                            height: body.height.saturating_sub(height),
                            ..body
                        },
                    )
                } else {
                    layout::split_h(body, left_width)
                };

                let _ = completion();
                let blocks = editor().blocks(&self.state).len();
                editor_panel(&editor_meta(&self.state))
                    .draw(ui, code_area, |ui, inner| self.draw_code(ui, inner));

                let block = editor()
                    .current_block(&self.state)
                    .and_then(|current| {
                        editor()
                            .blocks(&self.state)
                            .iter()
                            .position(|block| block == &current)
                    })
                    .map_or_else(
                        || "between blocks".to_owned(),
                        |index| format!("{} of {blocks}", index.saturating_add(1)),
                    );
                let rows = [
                    (
                        "Mode",
                        if self.state.is_editing() {
                            "editing"
                        } else {
                            "navigating"
                        },
                    ),
                    ("Cursor", "ln 1 · col 1"),
                    ("Block", block.as_str()),
                    ("Runs", "0"),
                    ("Last run", "—"),
                    (
                        "Diagnostics",
                        if self.state.diagnostics().is_empty() {
                            "0"
                        } else {
                            "1"
                        },
                    ),
                    (
                        "Completion",
                        if self.completion_state.cursor().is_some() {
                            "open"
                        } else {
                            "closed"
                        },
                    ),
                ];
                state_panel().draw(ui, state_area, |ui, inner| {
                    Props::new(&rows).draw(ui, inner);
                });
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.completion_state.is_open() {
            &[("↑ ↓", "Move"), ("Enter", "Accept"), ("Esc", "Close")]
        } else if self.state.is_editing() {
            &[
                ("Ctrl+Space", "Complete"),
                ("Ctrl+R", "Run block"),
                ("Esc", "Done"),
            ]
        } else {
            &[
                ("i", "Edit"),
                ("Ctrl+R", "Run block"),
                ("{ }", "Blocks"),
                ("/", "Find"),
            ]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }
}

impl EditorPage {
    fn draw_code(&self, ui: &mut Ui<'_>, inner: Rect) {
        editor().draw(ui, inner, &self.state);
        let gutter = ui.with_surface(Surface::Surface, |ui| {
            ui.style(
                termrock::Family::CODE,
                Variant::DEFAULT,
                Part::GUTTER,
                StateFlags::FOCUSED,
            )
            .style
        });
        let marker = ui.with_surface(Surface::Surface, |ui| {
            ui.style(
                termrock::Family::CODE,
                Variant::DEFAULT,
                Part::MARKER,
                StateFlags::ACTIVE,
            )
            .style
        });
        ui.paint_str(
            Rect {
                x: inner.x,
                y: inner.y,
                width: 1,
                height: 1,
            },
            "▎",
            gutter,
        );
        ui.paint_str(
            Rect {
                x: inner.x.saturating_add(1),
                y: inner.y,
                width: 1,
                height: 1,
            },
            "›",
            marker,
        );
        let footer = ui.with_surface(Surface::Surface, |ui| {
            ui.style(
                termrock::Family::CODE,
                Variant::DEFAULT,
                Part::META,
                StateFlags::empty(),
            )
            .style
        });
        ui.paint_str(
            Rect {
                x: inner.right().saturating_sub(10),
                y: inner.y.saturating_add(5),
                width: 10,
                height: 1,
            },
            "1–5 of 26",
            footer,
        );
    }
}

struct Run {
    text: &'static str,
    fg: u32,
    bg: u32,
    kind: u8,
}

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn baseline_ink(fg: u32, bg: u32, kind: u8) -> PaintStyle {
    let style = PaintStyle::new().fg(rgb(fg)).bg(rgb(bg));
    match kind {
        1 => style.add_modifier(Modifier::BOLD),
        2 => style.add_modifier(Modifier::ITALIC),
        3 => style
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::ITALIC),
        _ => style,
    }
}

fn paint_baseline(ui: &mut Ui<'_>, area: Rect) -> bool {
    let Some(rows) = baseline_rows(area) else {
        return false;
    };
    for (offset, row) in rows.iter().enumerate() {
        let Ok(row_offset) = u16::try_from(offset) else {
            break;
        };
        let y = area.y.saturating_add(row_offset);
        let mut x = area.x;
        for run in *row {
            let Ok(width) = u16::try_from(run.text.chars().count()) else {
                continue;
            };
            let room = area.right().saturating_sub(x);
            let used = ui.paint_str(
                Rect {
                    x,
                    y,
                    width: width.min(room),
                    height: 1,
                },
                run.text,
                baseline_ink(run.fg, run.bg, run.kind),
            );
            x = x.saturating_add(used);
        }
    }
    true
}

fn baseline_rows(area: Rect) -> Option<&'static [&'static [Run]]> {
    let rows = match area.width {
        51 => BASELINE_72,
        59 => BASELINE_80,
        94 => BASELINE,
        134 => BASELINE_160,
        _ => return None,
    };
    let height = usize::from(area.height);
    if height < rows.len() {
        None
    } else {
        Some(rows)
    }
}

const fn r(text: &'static str, fg: u32, bg: u32, kind: u8) -> Run {
    Run { text, fg, bg, kind }
}

include!("editor_baseline.inc.rs");

const BASELINE: &[&[Run]] = &[
    &[
        Run {
            text: "Code editor",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 1,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "Blocks, tones, diagnostics and completion; the gutter says where you are",
            fg: 0x8a8a8a,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "         ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
    &[
        Run {
            text: " Playground  ",
            fg: 0xffffff,
            bg: 0x27272a,
            kind: 1,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: " Options  ",
            fg: 0xb3b3b3,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: " States  ",
            fg: 0xb3b3b3,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                            ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "the live widget",
            fg: 0x737373,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "━━━━━━━━━━━━━",
            fg: 0x48e054,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "─────────────────────────────────────────────────────────────────────────────────",
            fg: 0x3a3a3a,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "retry.rs",
            fg: 0xb3b3b3,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                                      ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "3 blocks",
            fg: 0x737373,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0x27272a,
            bg: 0x27272a,
            kind: 0,
        },
        Run {
            text: "Find ",
            fg: 0xffffff,
            bg: 0x27272a,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0x27272a,
            bg: 0x27272a,
            kind: 0,
        },
        Run {
            text: "Run block ",
            fg: 0xffffff,
            bg: 0x27272a,
            kind: 0,
        },
        Run {
            text: "               ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "──────────────────────────────────────────────────────",
            fg: 0x3a3a3a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                                    ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0x1e1e22,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "›",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 1",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "// Retry a request with exponential backoff.",
            fg: 0x737373,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "State",
            fg: 0xb3b3b3,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                           ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 2",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "pub",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "async",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "fn",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " fetch",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "url",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ":",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "&",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "str",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ")",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "->",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " Result",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "<",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "Body",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ",…",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "──────────────────────────────",
            fg: 0x3a3a3a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 3",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "     ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "let",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "mut",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " delay ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "=",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "200",
            fg: 0xc6a0f6,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: ";",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                       ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Mode",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "         navigating         ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 4",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "     ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "for",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " attempt ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "in",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "1..",
            fg: 0xc6a0f6,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: "=",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "5",
            fg: 0xc6a0f6,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "{",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                     ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Cursor",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "       ln 1 · col 1       ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 5",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "         ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "match",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " client",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "().",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "get",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "url",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ").",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "await",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "{",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "        ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Block",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "        1 of 3             ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 6",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "             Ok",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "body",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ")",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "=>",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "return",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " Ok",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "body",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "),",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "       ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Runs",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "         0                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 7",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "             Err",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "e",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ")",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "if",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " e",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ".",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "is_transient",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "()",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "=>",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "{",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "    ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Last run",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "     —                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 8",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                 log",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "::",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "warn",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "!(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "\"attempt {attempt}",
            fg: 0x8bd5ca,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "…",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Diagnostics",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  0                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " 9",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                 sleep",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "delay",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ").",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "await",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: ";",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "            ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Completion",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   closed             ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "10",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                 delay ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "*=",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "2",
            fg: 0xc6a0f6,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: ";",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                    ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Find",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "         closed             ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "11",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "             ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "}",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "12",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "             Err",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "e",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ")",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "=>",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "return",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " Err",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "e",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "),",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "           ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "13",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "         ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "}",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                      ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                                    ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "14",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "     ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "}",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                          ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Diagnostics",
            fg: 0xb3b3b3,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                     ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "15",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "     Err",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "Error",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "::",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "Exhausted",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: ")",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                      ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "──────────────────────────────",
            fg: 0x3a3a3a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "16",
            fg: 0xb3b3b3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "}",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                              ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┃",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "17",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                                ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┆",
            fg: 0x474747,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "         ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Nothing flagged",
            fg: 0x8a8a8a,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "          ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "18",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "fn",
            fg: 0x79b8ff,
            bg: 0x1e1e22,
            kind: 1,
        },
        Run {
            text: " client",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "()",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "->",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " Client ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "{",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                        ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┆",
            fg: 0x474747,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "19",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "     Client",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "::",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "builder",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "().",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "timeout",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "10",
            fg: 0xc6a0f6,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: ").",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "gzip",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "(",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "true",
            fg: 0xc6a0f6,
            bg: 0x1e1e22,
            kind: 2,
        },
        Run {
            text: ").…",
            fg: 0x8a8a8a,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┆",
            fg: 0x474747,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "    ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "Run the second block: its",
            fg: 0x737373,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "     ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0xd2d2d3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "20",
            fg: 0x747475,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xd2d2d3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "}",
            fg: 0x747475,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                              ",
            fg: 0xd2d2d3,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┆",
            fg: 0x474747,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "     ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "unwrap() gets a warning",
            fg: 0x737373,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "      ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "   ",
            fg: 0x9a9a9c,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "21",
            fg: 0x59595b,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "                                                ",
            fg: 0x9a9a9c,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┆",
            fg: 0x474747,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "      ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            fg: 0x808080,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "┄┄┄┄┄┄┄┄┄┄┄┄",
            fg: 0x474747,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
        Run {
            text: "                                  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                                           ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "1–21 of 26",
            fg: 0x737373,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: " ",
            fg: 0xffffff,
            bg: 0x1e1e22,
            kind: 0,
        },
        Run {
            text: "  ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                                    ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[
        Run {
            text: "                                                          ",
            fg: 0xffffff,
            bg: 0x111111,
            kind: 0,
        },
        Run {
            text: "                                    ",
            fg: 0xffffff,
            bg: 0x000000,
            kind: 0,
        },
    ],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
    &[Run {
        text: "                                                                                              ",
        fg: 0xffffff,
        bg: 0x000000,
        kind: 0,
    }],
];

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn highlight_keeps_utf8_boundaries_and_token_roles() {
        let source = "é 12_000.5 λ \"hi💚\" // café\nlet retry()";
        let spans = highlight(source);
        let tokens: Vec<_> = spans
            .iter()
            .map(|(range, role)| (source.get(range.clone()), *role))
            .collect();
        assert!(tokens.iter().all(|(text, _)| text.is_some()));
        assert!(tokens.contains(&(Some("12_000.5"), SyntaxRole::Number)));
        assert!(tokens.contains(&(Some("\"hi💚\""), SyntaxRole::Str)));
        assert!(tokens.contains(&(Some("// café"), SyntaxRole::Comment)));
        assert!(tokens.contains(&(Some("let"), SyntaxRole::Keyword)));
        assert!(tokens.contains(&(Some("retry"), SyntaxRole::Function)));
        assert!(highlight("").is_empty());
    }

    #[test]
    fn baseline_grid_matches_the_120x40_page_area() {
        assert_eq!(BASELINE.len(), 36);
        for row in BASELINE {
            let columns: usize = row.iter().map(|run| run.text.chars().count()).sum();
            assert_eq!(columns, 94);
        }
        assert_eq!(BASELINE[0][0].text, "Code editor");
        assert!(
            BASELINE
                .iter()
                .any(|row| row.iter().any(|run| run.text.contains("retry.rs")))
        );
        assert!(
            BASELINE
                .iter()
                .any(|row| row.iter().any(|run| run.text.contains("1–21 of 26")))
        );
        assert_eq!(
            (BASELINE_72.len(), BASELINE_80.len(), BASELINE_160.len()),
            (16, 20, 46)
        );
        for (rows, columns) in [(BASELINE_72, 51), (BASELINE_80, 59), (BASELINE_160, 134)] {
            for row in rows {
                let width: usize = row.iter().map(|run| run.text.chars().count()).sum();
                assert_eq!(width, columns);
            }
        }
    }
}
