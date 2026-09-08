//! Code editor page with cursor, insert mode and diagnostics.

use std::ops::Range;

use junie_tui::{
    CodeAction, CodeEditor, CodeEditorState, Completion, CompletionState, Cx, DiffView,
    DiffViewState, Id, Item, ItemKey, Panel, Part, Props, Rect, StateFlags, Surface, SyntaxRole,
    TabBehavior, Ui, Variant, id, layout,
};

use super::{Page, PageUpdate, frame};

const EDITOR: Id = id!("editor.code");
const EDITOR_PANEL: Id = id!("editor.code.panel");
const STATE_PANEL: Id = id!("editor.state.panel");
const COMPLETION: Id = id!("editor.completion");
const DIFF: Id = id!("editor.diff");
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

/// Both phases label the editor card from the same live block count (§13).
fn editor_meta(state: &CodeEditorState) -> String {
    if state.is_editing() {
        String::from("running ")
    } else {
        format!("{} blocks ", editor().blocks(state).len())
    }
}

/// The editor's durable document and semantic spans are initialized from the
/// historical retry sample.
#[derive(Debug)]
pub(crate) struct EditorPage {
    state: CodeEditorState,
    completion_state: CompletionState,
    diff_state: DiffViewState,
    last: &'static str,
}

impl EditorPage {
    pub(crate) fn new() -> Self {
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
        // Both phases build the same two cards (§13); the draw pass reaches
        // the completion constructor beside them.
        let _ = editor_panel(&editor_meta(&self.state));
        let _ = state_panel();
        result.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
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
                // The completion list renders through the state panel's readout,
                // but the draw pass still builds the same props as update (§13).
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

    fn hints(&self, _ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if self.completion_state.is_open() {
            vec![("↑ ↓", "Move"), ("Enter", "Accept"), ("Esc", "Close")]
        } else if self.state.is_editing() {
            vec![
                ("Ctrl+Space", "Complete"),
                ("Ctrl+R", "Run block"),
                ("Esc", "Done"),
            ]
        } else {
            vec![
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
                junie_tui::Family::CODE,
                Variant::DEFAULT,
                Part::GUTTER,
                StateFlags::FOCUSED,
            )
            .style
        });
        let marker = ui.with_surface(Surface::Surface, |ui| {
            ui.style(
                junie_tui::Family::CODE,
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
                junie_tui::Family::CODE,
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
}
