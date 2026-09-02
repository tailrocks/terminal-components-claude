//! Code editor: block-aware editing, syntax tones, diagnostics, a running
//! block, and the completion popup.

use std::ops::Range;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{SyntaxTone, Tone};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::text::fuzzy;
use junie_tui::widgets::code::{CodeEditor, Diagnostic, EditorEvent, Severity};
use junie_tui::widgets::completion::{Completion, CompletionEvent, CompletionItem};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::scrollbar;

const ID: WidgetId = WidgetId::of("editor");

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

/// (label, detail, glyph) — a static symbol table stands in for a language server.
const CANDIDATES: &[(&str, &str, &str)] = &[
    ("fetch(", "async fn (url: &str) -> Result<Body, Error>", "ƒ"),
    ("client(", "fn () -> Client", "ƒ"),
    ("sleep(", "async fn (ms: u64)", "ƒ"),
    ("schedule(", "fn (attempts: u32) -> Vec<u64>", "ƒ"),
    ("Client", "struct", "T"),
    ("Body", "struct", "T"),
    ("Error", "enum · Transient, Exhausted", "T"),
    ("Result<T, E>", "enum", "T"),
    ("Option<T>", "enum", "T"),
    ("String", "struct", "T"),
    ("Vec<T>", "struct", "T"),
    ("delay", "local · u64", "v"),
    ("attempt", "local · u32", "v"),
    ("url", "param · &str", "v"),
    ("assert_eq!(", "macro", "m"),
    ("format!(", "macro", "m"),
    ("println!(", "macro", "m"),
    ("log::warn!(", "macro", "m"),
    ("await", "keyword", "k"),
    ("async", "keyword", "k"),
    ("match", "keyword", "k"),
    ("return", "keyword", "k"),
];

fn highlight(src: &str) -> Vec<(Range<usize>, SyntaxTone)> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if !src.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let c = b[i];
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            let end = src[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            out.push((i..end, SyntaxTone::Comment));
            i = end;
            continue;
        }
        if c == b'#' && b.get(i + 1) == Some(&b'[') {
            let end = src[i..].find(']').map(|n| i + n + 1).unwrap_or(b.len());
            out.push((i..end, SyntaxTone::Comment));
            i = end;
            continue;
        }
        if c == b'"' {
            let end = src[i + 1..].find('"').map(|n| i + n + 2).unwrap_or(b.len());
            out.push((i..end, SyntaxTone::Str));
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let mut j = i;
            while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_' || b[j] == b'.') {
                j += 1;
            }
            out.push((i..j, SyntaxTone::Number));
            i = j;
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let mut j = i;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                j += 1;
            }
            let word = &src[i..j];
            let next = b.get(j).copied();
            let tone = if KEYWORDS.contains(&word) {
                SyntaxTone::Keyword
            } else if next == Some(b'(')
                || next == Some(b'!')
                || word.starts_with(|ch: char| ch.is_ascii_uppercase())
            {
                SyntaxTone::Ident
            } else {
                SyntaxTone::Plain
            };
            out.push((i..j, tone));
            i = j;
            continue;
        }
        let tone = match c {
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b';' | b',' => SyntaxTone::Punct,
            b'=' | b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'!' | b'&' | b'|' | b':' | b'?'
            | b'.' => SyntaxTone::Operator,
            _ => {
                i += 1;
                continue;
            }
        };
        out.push((i..i + 1, tone));
        i += 1;
    }
    out
}

/// Blocks are paragraphs: runs of non-blank lines.
fn blocks(src: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let mut end = 0;
    let mut off = 0;
    for line in src.split_inclusive('\n') {
        if line.trim().is_empty() {
            if let Some(s) = start.take() {
                out.push(s..end);
            }
        } else {
            if start.is_none() {
                start = Some(off);
            }
            end = off + line.trim_end_matches('\n').len();
        }
        off += line.len();
    }
    if let Some(s) = start {
        out.push(s..end);
    }
    out
}

pub struct EditorPage {
    editor: CodeEditor,
    completion: Completion,
    run_ticks: u8,
    runs: u32,
    last_ms: Option<u32>,
}

impl EditorPage {
    pub fn new() -> Self {
        Self {
            editor: CodeEditor::new(ID.sub("code"), SAMPLE)
                .highlighter(highlight)
                .segmenter(blocks)
                .placeholder("Type code. i edits, Ctrl+R runs the block under the cursor."),
            completion: Completion::new(ID.sub("complete")),
            run_ticks: 0,
            runs: 0,
            last_ms: None,
        }
    }

    /// The identifier being typed, as (start offset, word).
    fn word_before_cursor(&self) -> (usize, String) {
        let cur = self.editor.cursor_offset();
        let head = &self.editor.text()[..cur];
        let start = head
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .map(|p| p + head[p..].chars().next().map_or(1, char::len_utf8))
            .unwrap_or(0);
        (start, head[start..].to_owned())
    }

    fn trigger(&mut self, manual: bool) {
        let (_, word) = self.word_before_cursor();
        if !manual && word.len() < 2 {
            self.completion.close();
            return;
        }
        let mut ranked: Vec<(u32, CompletionItem)> = CANDIDATES
            .iter()
            .filter_map(|(label, detail, glyph)| {
                let (penalty, matched) = fuzzy(label, &word)?;
                Some((
                    penalty,
                    CompletionItem {
                        label: (*label).to_owned(),
                        glyph,
                        detail: (*detail).to_owned(),
                        insert: (*label).to_owned(),
                        matched,
                    },
                ))
            })
            .collect();
        ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.label.cmp(&b.1.label)));
        let items: Vec<CompletionItem> = ranked.into_iter().map(|r| r.1).collect();
        let replace = word.len();
        let anchor = self
            .editor
            .cursor_cell()
            .map(|c| Rect::new(c.x.saturating_sub(replace as u16), c.y, 1, 1))
            .unwrap_or(Rect::ZERO);
        if items.is_empty() {
            self.completion.close();
        } else {
            self.completion.open(items, anchor, replace);
        }
    }

    fn accept(&mut self, i: usize) {
        let Some(item) = self.completion.items.get(i).cloned() else {
            return;
        };
        let replace = self.completion.replace_len;
        let cur = self.editor.cursor_offset();
        self.editor.buffer.remove_range(cur - replace..cur);
        self.editor.buffer.insert_str(&item.insert);
        if item.insert.ends_with('(') {
            self.editor.buffer.insert_char(')');
            self.editor.buffer.move_left(false);
        }
        self.completion.close();
    }

    fn run(&mut self, cx: &mut PageCtx) {
        let Some(block) = self.editor.current_block() else {
            cx.status("Nothing to run: the cursor is between blocks");
            return;
        };
        self.completion.close();
        self.editor.set_running(Some(block));
        self.run_ticks = 10;
    }

    fn finish_run(&mut self, cx: &mut PageCtx) {
        if let Some(block) = self.editor.running.clone() {
            let text = self.editor.text()[block.clone()].to_owned();
            if let Some(p) = text.find(".unwrap()") {
                self.editor.diagnostics.push(Diagnostic {
                    range: block.start + p + 1..block.start + p + 9,
                    severity: Severity::Warning,
                    message: "unwrap() panics on Err; propagate with ? instead".into(),
                });
            }
            if let Some(p) = text.find("todo!") {
                self.editor.diagnostics.push(Diagnostic {
                    range: block.start + p..block.start + p + 5,
                    severity: Severity::Error,
                    message: "not yet implemented".into(),
                });
            }
        }
        self.editor.set_running(None);
        self.runs += 1;
        let ms = 40 + (self.runs * 37) % 90;
        self.last_ms = Some(ms);
        cx.status(format!("Block ran in {ms} ms"));
    }
}

impl Page for EditorPage {
    fn title(&self) -> &'static str {
        "Code editor"
    }
    fn blurb(&self) -> &'static str {
        "Blocks, tones, diagnostics and completion; the gutter says where you are"
    }
    fn editing(&self) -> bool {
        self.editor.editing
    }
    fn animating(&self) -> bool {
        self.run_ticks > 0
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let (l, r) = crate::pages::layout::columns(area, (area.width * 62 / 100).max(40), 2);
        let focused = ctx.interaction.focused(self.editor.id);
        let meta = if self.run_ticks > 0 {
            "running".to_owned()
        } else {
            format!("{} blocks", self.editor.blocks().len())
        };
        let panel = Panel::card(Some("retry.rs")).focused(focused).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(Rect::new(l.x, l.y, l.width, l.height.min(26)), buf, t);
        self.editor.render(inner, buf, ctx, bg);

        let panel = Panel::card(Some("State"));
        let bg = panel.bg(t);
        let inner = panel.render(Rect::new(r.x, r.y, r.width, r.height.min(11)), buf, t);
        let pos = self.editor.buffer.cursor_pos();
        let all = self.editor.blocks();
        let cur = self.editor.cursor_offset();
        let block = all
            .iter()
            .position(|b| b.start <= cur && cur <= b.end)
            .map(|i| format!("{} of {}", i + 1, all.len()))
            .unwrap_or_else(|| "between blocks".into());
        let diags = self.editor.diagnostics.len();
        let props = vec![
            Prop::new(
                "Mode",
                if self.editor.editing {
                    "editing"
                } else {
                    "navigating"
                },
            )
            .tone(if self.editor.editing {
                Tone::Success
            } else {
                Tone::Normal
            }),
            Prop::new(
                "Cursor",
                format!("ln {} · col {}", pos.line + 1, pos.col + 1),
            ),
            Prop::new("Block", block),
            Prop::new("Runs", self.runs.to_string()),
            Prop::new(
                "Last run",
                self.last_ms
                    .map(|ms| format!("{ms} ms"))
                    .unwrap_or_else(|| "—".into()),
            ),
            Prop::new("Diagnostics", diags.to_string()).tone(if diags > 0 {
                Tone::Warning
            } else {
                Tone::Normal
            }),
            Prop::new(
                "Completion",
                if self.completion.is_open() {
                    format!("{} items", self.completion.items.len())
                } else {
                    "closed".into()
                },
            ),
        ];
        props::render(inner, buf, t, &props, bg);

        let y = r.y + 12;
        if y + 4 < r.bottom() {
            let panel = Panel::card(Some("Diagnostics"));
            let bg = panel.bg(t);
            let inner = panel.render(Rect::new(r.x, y, r.width, (r.bottom() - y).min(9)), buf, t);
            if self.editor.diagnostics.is_empty() {
                empty::render(
                    inner,
                    buf,
                    t,
                    &EmptyState::new("Nothing flagged")
                        .hint("Run the second block: its unwrap() gets a warning"),
                    bg,
                );
            } else {
                for (i, d) in self.editor.diagnostics.iter().enumerate() {
                    let yy = inner.y + i as u16;
                    if yy >= inner.bottom() {
                        break;
                    }
                    let line = junie_tui::core::text::TextBuffer::pos_of(
                        self.editor.text(),
                        d.range.start,
                    )
                    .line
                        + 1;
                    let (glyph, st) = match d.severity {
                        Severity::Error => ("!", Style::new().fg(t.error)),
                        Severity::Warning => ("!", Style::new().fg(t.warning)),
                    };
                    buf.set_string(inner.x, yy, glyph, st.bg(bg));
                    buf.set_string(
                        inner.x + 2,
                        yy,
                        junie_tui::ui::text::truncate(
                            &format!("ln {line} · {}", d.message),
                            inner.width.saturating_sub(2) as usize,
                        ),
                        t.secondary().bg(bg),
                    );
                }
            }
        }

        // the popup sits above everything else on the page
        if self.completion.is_open() {
            let screen = *buf.area();
            self.completion.render(screen, buf, ctx);
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                if self.run_ticks == 0 {
                    return Outcome::Ignored;
                }
                self.run_ticks -= 1;
                if self.run_ticks == 0 {
                    self.finish_run(cx);
                }
                Outcome::Changed
            }
            PageEvent::Key(key) => {
                if !cx.focus.is(self.editor.id) {
                    return Outcome::Ignored;
                }
                if self.completion.is_open() {
                    let (o, ev) = self.completion.on_key(key);
                    match ev {
                        Some(CompletionEvent::Accept(i)) => {
                            self.accept(i);
                            return Outcome::Changed;
                        }
                        Some(CompletionEvent::Dismiss) => return Outcome::Changed,
                        None => {
                            if o.consumed() {
                                return o;
                            }
                        }
                    }
                }
                if key.ctrl_char(' ') || key.code == KeyCode::Null {
                    if !self.editor.editing {
                        self.editor.begin_edit();
                    }
                    self.trigger(true);
                    return Outcome::Changed;
                }
                if key.ctrl_char('r') {
                    self.run(cx);
                    return Outcome::Changed;
                }
                let (o, ev) = self.editor.on_key(key);
                match ev {
                    Some(EditorEvent::Changed) => {
                        self.editor.diagnostics.clear();
                        self.trigger(false);
                    }
                    Some(EditorEvent::CursorMoved) => {
                        if self.completion.is_open() {
                            self.trigger(false);
                        }
                    }
                    Some(EditorEvent::Committed) => self.completion.close(),
                    Some(EditorEvent::Leave { backward }) => {
                        if backward {
                            cx.focus_prev();
                        } else {
                            cx.focus_next();
                        }
                    }
                    None => {}
                }
                o
            }
            PageEvent::Paste(text) => self.editor.on_paste(text),
            PageEvent::Click { id, pos } => {
                if self.completion.is_open() {
                    if let Some(CompletionEvent::Accept(i)) = self.completion.on_click(*id) {
                        self.accept(i);
                        return Outcome::Changed;
                    }
                    if !self.completion.owns(*id) {
                        self.completion.close();
                    }
                }
                if *id == self.editor.id {
                    let was = cx.focus.is(*id);
                    cx.focus.focus(*id);
                    return self.editor.on_click(*pos, was);
                }
                if *id == scrollbar::id_for(self.editor.id) {
                    return self.editor.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if *pressed == self.editor.id {
                    self.editor.on_drag(*pos)
                } else if *pressed == scrollbar::id_for(self.editor.id) {
                    self.editor.on_scrollbar(*pos)
                } else {
                    Outcome::Ignored
                }
            }
            PageEvent::Wheel { id, delta } => {
                if self.completion.owns(*id) {
                    self.completion.on_wheel(*delta)
                } else if *id == self.editor.id {
                    self.editor.on_wheel(*delta, false)
                } else {
                    Outcome::Ignored
                }
            }
            PageEvent::DialogClosed { .. } => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        if self.completion.is_open() {
            vec![("↑ ↓", "Move"), ("Enter", "Accept"), ("Esc", "Close")]
        } else if self.editor.editing {
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
}
