use std::ops::Range;
use std::time::Duration;

use termrock::composition::{self, Group};
use termrock::editors::{CompletionBatch, CompletionProvider, CompletionTrigger, CompletionValue};
use termrock::layout::Composition as LayoutComposition;
use termrock::{
    ActionKey, Binding, Button, Chord, CodeAction, CodeDiagnostic, CodeEditor, CodeSeverity,
    CompletingCodeEditor, CompletingCodeEditorState, Cx, FrameRead, Id, ItemKey, Panel, PropsList,
    PropsRow, PropsState, Rect, Response, SyntaxRole, TabBehavior, Ui, Variant,
};

use super::{Page, PageStatus, PageUpdate, frame};

const EDITOR: Id = Id::root("showcase_ui::pages::editor::editor.code");
const EDITOR_PANEL: Id = Id::root("showcase_ui::pages::editor::editor.code.panel");
const STATE_PANEL: Id = Id::root("showcase_ui::pages::editor::editor.state.panel");
const STATE_LIST: Id = Id::root("showcase_ui::pages::editor::editor.state.list");
const DIAGNOSTICS_PANEL: Id = Id::root("showcase_ui::pages::editor::editor.diagnostics.panel");
const DIAGNOSTICS_LIST: Id = Id::root("showcase_ui::pages::editor::editor.diagnostics.list");
const COMPLETION: Id = Id::root("showcase_ui::pages::editor::editor.completion");
const FIND: Id = Id::root("showcase_ui::pages::editor::editor.find");
const RUN: Id = Id::root("showcase_ui::pages::editor::editor.run");
const STATE_KEYS: [ItemKey; 8] = [
    ItemKey::Num(1),
    ItemKey::Num(2),
    ItemKey::Num(3),
    ItemKey::Num(4),
    ItemKey::Num(5),
    ItemKey::Num(6),
    ItemKey::Num(7),
    ItemKey::Num(8),
];
const RUN_ACTION: ActionKey = ActionKey::application("showcase.editor.run_block");
const RUN_BINDINGS: &[Binding<()>] = &[Binding {
    action: RUN_ACTION,
    chord: Some(Chord::with(
        termrock::KeyCode::Char('r'),
        termrock::KeyModifiers::CONTROL,
    )),
    cmd: (),
    label: "Run block",
    priority: 70,
    visible: true,
}];

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
    Client::builder().timeout(10).gzip(true).build().unwrap()
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

const FETCH_FULL_DETAIL: &str = "async fn (url: &str) -> Result<Body, Error>";
const FETCH_MENU_SUMMARY: &str = "async fn (url: &str)";

const CANDIDATES: &[(&str, &str, &str, &str, Option<&str>)] = &[
    (
        "fetch(",
        FETCH_FULL_DETAIL,
        "ƒ",
        "fetch()",
        Some(FETCH_MENU_SUMMARY),
    ),
    ("client(", "fn () -> Client", "ƒ", "client()", None),
    ("sleep(", "async fn (ms: u64)", "ƒ", "sleep()", None),
    (
        "schedule(",
        "fn (attempts: u32) -> Vec<u64>",
        "ƒ",
        "schedule()",
        None,
    ),
    ("Client", "struct", "T", "Client", None),
    ("Body", "struct", "T", "Body", None),
    ("Error", "enum · Transient, Exhausted", "T", "Error", None),
    ("Result<T, E>", "enum", "T", "Result<T, E>", None),
    ("Option<T>", "enum", "T", "Option<T>", None),
    ("String", "struct", "T", "String", None),
    ("Vec<T>", "struct", "T", "Vec<T>", None),
    ("delay", "local · u64", "v", "delay", None),
    ("attempt", "local · u32", "v", "attempt", None),
    ("url", "param · &str", "v", "url", None),
    ("assert_eq!(", "macro", "m", "assert_eq!()", None),
    ("format!(", "macro", "m", "format!()", None),
    ("println!(", "macro", "m", "println!()", None),
    ("log::warn!(", "macro", "m", "log::warn!()", None),
    ("await", "keyword", "k", "await", None),
    ("async", "keyword", "k", "async", None),
    ("match", "keyword", "k", "match", None),
    ("return", "keyword", "k", "return", None),
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
        if byte == b'#' && bytes.get(i.saturating_add(1)) == Some(&b'[') {
            let end = src[i..]
                .find(']')
                .map_or(bytes.len(), |n| i.saturating_add(n).saturating_add(1));
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
            let role = if matches!(word, "true" | "false") {
                SyntaxRole::Constant
            } else if KEYWORDS.contains(&word) {
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

#[derive(Clone, Copy, Debug)]
struct ShowcaseCompletion;

impl CompletionProvider for ShowcaseCompletion {
    fn complete(
        &self,
        text: &str,
        cursor: usize,
        trigger: CompletionTrigger,
    ) -> Option<CompletionBatch> {
        let before = text.get(..cursor)?;
        let start = before
            .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == ':'))
            .map_or(0, |index| index.saturating_add(1));
        let word = before.get(start..)?;
        if trigger != CompletionTrigger::Manual && word.chars().count() < 2 {
            return None;
        }
        let mut ranked: Vec<_> = CANDIDATES
            .iter()
            .enumerate()
            .filter_map(|(index, (label, detail, glyph, insertion, menu_summary))| {
                let (penalty, matched) = termrock::fuzzy(label, word)?;
                let value = CompletionValue::new(
                    ItemKey::num(u64::try_from(index).ok()?.saturating_add(1)),
                    *label,
                    *insertion,
                )
                .glyph(*glyph)
                .matched(matched)
                .detail(*detail);
                let value = match menu_summary {
                    Some(summary) => value.menu_summary(*summary),
                    None => value,
                };
                Some((penalty, *label, value))
            })
            .collect();
        ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
        let items = ranked.into_iter().map(|(_, _, item)| item).collect();
        Some(CompletionBatch {
            items,
            replace_len: word.len(),
        })
    }
}

static COMPLETION_PROVIDER: ShowcaseCompletion = ShowcaseCompletion;

fn code_editor() -> CodeEditor<'static> {
    CodeEditor::new(EDITOR, 12)
        .highlighter(&highlight)
        .segmenter(&blocks)
        .indent(4)
        .tab_behavior(TabBehavior::Indent)
        .placeholder("Type code. i edits, Ctrl+R runs the block under the cursor.")
}

fn editor() -> CompletingCodeEditor<'static> {
    code_editor()
        .with_completion(COMPLETION, &COMPLETION_PROVIDER)
        .selected_details()
}

fn find_button() -> Button<'static> {
    Button::new(FIND, "Find")
}

fn run_button() -> Button<'static> {
    Button::new(RUN, "Run block").variant(Variant::SECONDARY)
}

fn state_list() -> PropsList<'static> {
    PropsList::new(STATE_LIST)
}

fn diagnostics_list() -> PropsList<'static> {
    PropsList::new(DIAGNOSTICS_LIST)
}

#[derive(Debug)]
pub struct EditorPage {
    state: CompletingCodeEditorState,
    properties: PropsState,
    diagnostics: PropsState,
    run_ticks: u8,
    running_block: Option<Range<usize>>,
    runs: u32,
    last_ms: Option<u32>,
}

impl EditorPage {
    pub fn new() -> Self {
        Self {
            state: CompletingCodeEditorState::new(SAMPLE),
            properties: PropsState::default(),
            diagnostics: PropsState::default(),
            run_ticks: 0,
            running_block: None,
            runs: 0,
            last_ms: None,
        }
    }

    fn state_values(&self) -> [(&'static str, String); 8] {
        let state = self.state.editor();
        let cursor = state.cursor_pos();
        let component = code_editor();
        let all_blocks = component.blocks(state);
        let cursor_offset = state.cursor_offset();
        let block = all_blocks
            .iter()
            .position(|range| range.start <= cursor_offset && cursor_offset <= range.end)
            .map_or_else(
                || "between blocks".to_owned(),
                |index| format!("{} of {}", index.saturating_add(1), all_blocks.len()),
            );
        let find = state.find_info().map_or_else(
            || String::from("closed"),
            |find| {
                if find.query.is_empty() {
                    String::from("type to find")
                } else if let Some(current) = find.current {
                    format!(
                        "“{}” · {} of {}",
                        find.query,
                        current.saturating_add(1),
                        find.matches
                    )
                } else {
                    format!("“{}” · no matches", find.query)
                }
            },
        );
        [
            (
                "Mode",
                if state.is_editing() {
                    "editing"
                } else {
                    "navigating"
                }
                .to_owned(),
            ),
            (
                "Cursor",
                format!(
                    "ln {} · col {}",
                    cursor.line.saturating_add(1),
                    cursor.col.saturating_add(1)
                ),
            ),
            ("Block", block),
            ("Runs", self.runs.to_string()),
            (
                "Last run",
                self.last_ms
                    .map_or_else(|| String::from("—"), |ms| format!("{ms} ms")),
            ),
            ("Diagnostics", state.diagnostics().len().to_string()),
            (
                "Completion",
                if self.state.is_completion_open() {
                    "open"
                } else {
                    "closed"
                }
                .to_owned(),
            ),
            ("Find", find),
        ]
    }

    fn diagnostic_values(&self) -> Vec<(&'static str, String)> {
        if self.state.editor().diagnostics().is_empty() {
            return vec![
                ("Status", String::from("Nothing flagged")),
                (
                    "Hint",
                    String::from("Run the second block: unwrap() gets a warning"),
                ),
            ];
        }
        self.state
            .editor()
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                let line = self
                    .state
                    .editor()
                    .text()
                    .get(..diagnostic.range.start)
                    .map_or(1, |before| {
                        before.bytes().filter(|byte| *byte == b'\n').count() + 1
                    });
                let label = match diagnostic.severity {
                    CodeSeverity::Error => "Error",
                    CodeSeverity::Warning => "Warning",
                    CodeSeverity::Info => "Info",
                };
                let message = format!("ln {line} · {}", diagnostic.message);
                (label, message)
            })
            .collect()
    }

    fn start_run(&mut self, cx: &mut Cx<'_>) -> Option<PageStatus> {
        let component = code_editor();
        let block = component.current_block(self.state.editor())?;
        self.state.editor_mut().set_running(Some(block.clone()));
        self.running_block = Some(block);
        self.run_ticks = 10;
        cx.request_repaint_after(Duration::from_millis(40));
        Some(PageStatus(String::from("Running block…")))
    }

    fn finish_run(&mut self) -> PageStatus {
        let block_range = self.running_block.take();
        if let Some(block_range) = block_range {
            let text = self.state.editor().text();
            let block = text.get(block_range.clone()).unwrap_or_default();
            let mut diagnostics = self.state.editor().diagnostics().to_vec();
            if let Some(position) = block.find(".unwrap()") {
                let start = block_range.start.saturating_add(position).saturating_add(1);
                diagnostics.push(CodeDiagnostic::new(
                    start..start.saturating_add("unwrap()".len()),
                    CodeSeverity::Warning,
                    "unwrap() panics on Err; propagate with ? instead",
                ));
            }
            if let Some(position) = block.find("todo!") {
                let start = block_range.start.saturating_add(position);
                diagnostics.push(CodeDiagnostic::new(
                    start..start.saturating_add("todo!".len()),
                    CodeSeverity::Error,
                    "not yet implemented",
                ));
            }
            self.state.editor_mut().set_diagnostics(diagnostics);
        }
        self.state.editor_mut().set_running(None);
        self.runs = self.runs.saturating_add(1);
        let ms = 40_u32.saturating_add(self.runs.saturating_mul(37) % 90);
        self.last_ms = Some(ms);
        PageStatus(format!("Block ran in {ms} ms"))
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
        let component = editor();
        let mut response = Response::ignored();
        let mut status = None;
        if cx.intents(EDITOR).any(
            |intent| matches!(intent, termrock::Intent::Binding(action) if action == RUN_ACTION),
        ) {
            status = self.start_run(cx);
            response |= Response::changed();
        }

        let editor_response = component.update(cx, &mut self.state);
        if let Some(action) = editor_response.action_ref() {
            match action {
                CodeAction::Changed => {
                    if self.run_ticks == 0 {
                        self.state.editor_mut().set_diagnostics(Vec::new());
                    }
                }
                CodeAction::CursorMoved | CodeAction::Committed | CodeAction::Leave { .. } => {}
            }
        }
        response |= editor_response.erase();

        if find_button().update(cx).activated() {
            let opened = component.open_find(cx, &mut self.state);
            if opened.is_changed() {
                response |= opened.erase();
            }
        }
        if run_button().update(cx).activated() {
            status = self.start_run(cx);
            response |= Response::changed();
        }

        let state_values = self.state_values();
        let state_rows: Vec<_> = state_values
            .iter()
            .zip(STATE_KEYS)
            .map(|((label, value), key)| PropsRow::new(key, label, value))
            .collect();
        response |= state_list()
            .update(cx, &mut self.properties, &state_rows)
            .erase();
        let diagnostic_values = self.diagnostic_values();
        let diagnostic_rows: Vec<_> = diagnostic_values
            .iter()
            .enumerate()
            .map(|(index, (label, value))| {
                PropsRow::new(
                    ItemKey::num(u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1)),
                    label,
                    value,
                )
                .wrap()
            })
            .collect();
        response |= diagnostics_list()
            .update(cx, &mut self.diagnostics, &diagnostic_rows)
            .erase();

        if cx.update_cause() == termrock::UpdateCause::Tick && self.run_ticks > 0 {
            self.run_ticks = self.run_ticks.saturating_sub(1);
            if self.run_ticks == 0 {
                status = Some(self.finish_run());
                response |= Response::changed();
            } else {
                cx.request_repaint_after(Duration::from_millis(40));
            }
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Blocks, tones, diagnostics and completion; the gutter says where you are",
            |ui, body| {
                let component = editor();
                let editor_meta = if self.run_ticks > 0 {
                    String::from("running")
                } else {
                    format!("{} blocks", code_editor().blocks(self.state.editor()).len())
                };
                let editor_panel = Panel::new(EDITOR_PANEL)
                    .title("retry.rs")
                    .meta(&editor_meta);
                let editor_child = composition::panel(
                    &editor_panel,
                    composition::completing_code_editor(&component, &self.state),
                )
                .min(40, 8)
                .flex(2);

                let find = find_button();
                let run = run_button();
                let actions = Group::row([composition::button(&find), composition::button(&run)])
                    .spec(LayoutComposition::new().gap(2))
                    .into_child();

                let state_values = self.state_values();
                let state_rows: Vec<_> = state_values
                    .iter()
                    .zip(STATE_KEYS)
                    .map(|((label, value), key)| PropsRow::new(key, label, value))
                    .collect();
                let state_component = state_list();
                let state_panel = Panel::new(STATE_PANEL).title("State");
                let state_child = composition::panel(
                    &state_panel,
                    composition::props_list(&state_component, &self.properties, &state_rows),
                )
                .min(20, 11)
                .flex(2);

                let diagnostic_values = self.diagnostic_values();
                let diagnostic_rows: Vec<_> = diagnostic_values
                    .iter()
                    .enumerate()
                    .map(|(index, (label, value))| {
                        PropsRow::new(
                            ItemKey::num(
                                u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1),
                            ),
                            label,
                            value,
                        )
                        .wrap()
                    })
                    .collect();
                let diagnostic_component = diagnostics_list();
                let diagnostics_panel = Panel::new(DIAGNOSTICS_PANEL).title("Diagnostics");
                let diagnostics_child = composition::panel(
                    &diagnostics_panel,
                    composition::props_list(
                        &diagnostic_component,
                        &self.diagnostics,
                        &diagnostic_rows,
                    ),
                )
                .flex(1);

                let details = Group::column([actions, state_child, diagnostics_child])
                    .spec(LayoutComposition::new().gap(1))
                    .into_child()
                    .min(34, 10)
                    .flex(1);
                let playground =
                    Group::wrap([editor_child, details]).spec(LayoutComposition::new().gap(2));
                ui.with_area(body, |ui| {
                    let _ = playground.show(ui);
                    ui.publish_additional_bindings(EDITOR, ui.state(EDITOR), RUN_BINDINGS);
                });
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.state.is_completion_details_open() {
            &[
                ("↑ ↓ / PgUp Dn", "Scroll detail"),
                ("Enter / Tab", "Accept"),
                ("Esc", "Back to list"),
            ]
        } else if self.state.is_completion_open()
            && self
                .state
                .selected_completion()
                .is_some_and(|item| !item.full_detail().is_empty())
        {
            &[
                ("↑ ↓", "Move"),
                ("Ctrl+Shift+D", "Details"),
                ("Enter / Tab", "Accept"),
                ("Esc", "Close"),
            ]
        } else if self.state.is_completion_open() {
            &[("↑ ↓", "Move"), ("Enter", "Accept"), ("Esc", "Close")]
        } else if self.state.editor().is_editing() {
            &[
                ("Ctrl+Space", "Complete"),
                ("Tab", "Indent"),
                ("Esc", "Done"),
            ]
        } else if self.state.editor().find_info().is_some() {
            &[("Enter", "Next match"), ("Esc", "Close find")]
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
        self.state.editor().is_editing()
    }
}
