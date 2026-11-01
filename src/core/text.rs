use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn normalized_text(text: &str, multiline: bool) -> std::borrow::Cow<'_, str> {
    if multiline && text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n").into()
    } else if !multiline && text.contains(['\r', '\n']) {
        text.chars()
            .filter(|c| !matches!(c, '\r' | '\n'))
            .collect::<String>()
            .into()
    } else {
        text.into()
    }
}

fn normalize_owned(text: String, multiline: bool) -> String {
    match normalized_text(&text, multiline) {
        std::borrow::Cow::Borrowed(_) => text,
        std::borrow::Cow::Owned(normalized) => normalized,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,

    cursor: usize,

    anchor: Option<usize>,
    multiline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub line: usize,

    pub col: usize,
}

impl TextBuffer {
    pub fn single(text: impl Into<String>) -> Self {
        let text = normalize_owned(text.into(), false);
        Self {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: false,
        }
    }

    pub fn multi(text: impl Into<String>) -> Self {
        let text = normalize_owned(text.into(), true);
        Self {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn cursor_offset(&self) -> usize {
        self.cursor
    }

    pub fn select_range(&mut self, a: usize, b: usize) {
        self.anchor = Some(Self::floor_boundary(&self.text, a));
        self.cursor = Self::floor_boundary(&self.text, b);
    }

    pub fn selection_lines(&self) -> (usize, usize) {
        match self.selection() {
            Some(r) => {
                let a = Self::pos_of(&self.text, r.start).line;

                let before = &self.text[..r.end];
                let b = before.matches('\n').count() - usize::from(before.ends_with('\n'));
                (a, b)
            }
            None => {
                let l = self.cursor_pos().line;
                (l, l)
            }
        }
    }

    pub fn has_selection_lines(&self) -> bool {
        self.selection()
            .is_some_and(|r| self.text[r].contains('\n'))
    }

    pub fn insert_at(&mut self, at: usize, s: &str) {
        let at = Self::floor_boundary(&self.text, at);
        let s = normalized_text(s, self.multiline);
        self.text.insert_str(at, &s);
        if self.cursor >= at {
            self.cursor += s.len();
        }
        if let Some(a) = self.anchor.as_mut()
            && *a >= at
        {
            *a += s.len();
        }
        self.normalize_positions();
    }

    pub fn remove_range(&mut self, r: std::ops::Range<usize>) {
        let r = r.start.min(self.text.len())..r.end.min(self.text.len());
        if r.start >= r.end {
            return;
        }
        let r = Self::floor_boundary(&self.text, r.start)..Self::ceil_boundary(&self.text, r.end);
        let n = r.end - r.start;
        self.text.replace_range(r.clone(), "");
        let adjust = |p: &mut usize| {
            if *p >= r.end {
                *p -= n;
            } else if *p > r.start {
                *p = r.start;
            }
        };
        adjust(&mut self.cursor);
        if let Some(a) = self.anchor.as_mut() {
            adjust(a);
        }
        self.normalize_positions();
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = normalize_owned(text.into(), self.multiline);
        self.cursor = self.text.len();
        self.anchor = None;
    }

    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        Some(a.min(self.cursor)..a.max(self.cursor))
    }

    pub fn has_selection(&self) -> bool {
        self.selection().is_some()
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.len();
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    pub fn select_to(&mut self, offset: usize) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        self.cursor = Self::floor_boundary(&self.text, offset);
    }

    pub fn word_range_at(&self, offset: usize) -> std::ops::Range<usize> {
        let offset = Self::floor_boundary(&self.text, offset);
        let (start, end) = (self.line_start(offset), self.line_end(offset));
        let line = &self.text[start..end];
        let gs: Vec<(usize, &str)> = line
            .grapheme_indices(true)
            .map(|(i, g)| (start + i, g))
            .collect();
        let Some(at) = gs
            .iter()
            .position(|&(i, g)| offset < i + g.len())
            .or_else(|| gs.len().checked_sub(1))
        else {
            return offset..offset;
        };
        #[derive(PartialEq)]
        enum Class {
            Word,
            Space,
            Other,
        }
        let class = |g: &str| {
            if g.chars().any(char::is_alphanumeric) || g == "_" {
                Class::Word
            } else if g.chars().all(char::is_whitespace) {
                Class::Space
            } else {
                Class::Other
            }
        };
        let c = class(gs[at].1);
        if c == Class::Other {
            return gs[at].0..gs[at].0 + gs[at].1.len();
        }
        let mut lo = at;
        while lo > 0 && class(gs[lo - 1].1) == c {
            lo -= 1;
        }
        let mut hi = at;
        while hi + 1 < gs.len() && class(gs[hi + 1].1) == c {
            hi += 1;
        }
        gs[lo].0..gs[hi].0 + gs[hi].1.len()
    }

    pub fn select_word_at(&mut self, offset: usize) {
        let r = self.word_range_at(offset);
        self.select_range(r.start, r.end);
    }

    pub fn select_line_at(&mut self, offset: usize) {
        let offset = Self::floor_boundary(&self.text, offset);
        let (start, end) = (self.line_start(offset), self.line_end(offset));
        self.select_range(start, end);
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selection().map(|r| &self.text[r])
    }

    fn begin_move(&mut self, select: bool) {
        if select {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
    }

    fn floor_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|i| *i <= offset)
            .last()
            .unwrap_or(0)
    }

    fn ceil_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|i| *i >= offset)
            .unwrap_or(text.len())
    }

    fn normalize_positions(&mut self) {
        self.cursor = Self::ceil_boundary(&self.text, self.cursor);
        self.anchor = self.anchor.map(|a| Self::floor_boundary(&self.text, a));
    }

    fn prev_boundary(&self, from: usize) -> usize {
        self.text[..from]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, from: usize) -> usize {
        self.text[from..]
            .graphemes(true)
            .next()
            .map(|g| from + g.len())
            .unwrap_or(from)
    }

    fn line_start(&self, from: usize) -> usize {
        self.text[..from].rfind('\n').map(|i| i + 1).unwrap_or(0)
    }

    fn line_end(&self, from: usize) -> usize {
        self.text[from..]
            .find('\n')
            .map(|i| from + i)
            .unwrap_or(self.text.len())
    }

    fn prev_word(&self, from: usize) -> usize {
        let mut graphemes = self.text[..from].grapheme_indices(true).rev().peekable();
        while graphemes
            .peek()
            .is_some_and(|(_, g)| !g.chars().any(char::is_alphanumeric))
        {
            graphemes.next();
        }
        let mut start = 0;
        while let Some(&(i, g)) = graphemes.peek() {
            if !g.chars().any(char::is_alphanumeric) {
                break;
            }
            start = i;
            graphemes.next();
        }
        start
    }

    fn next_word(&self, from: usize) -> usize {
        let after = &self.text[from..];
        let mut it = after.grapheme_indices(true);
        let mut i = 0;
        while let Some((idx, g)) = it.next() {
            i = idx + g.len();
            if !g.chars().any(char::is_alphanumeric) {
                continue;
            }
            for (idx2, g2) in it.by_ref() {
                if !g2.chars().any(char::is_alphanumeric) {
                    i = idx2;
                    return from + i;
                }
                i = idx2 + g2.len();
            }
            break;
        }
        from + i
    }

    pub fn move_left(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.cursor = r.start;
            return;
        }
        self.begin_move(select);
        self.cursor = self.prev_boundary(self.cursor);
    }

    pub fn move_right(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.cursor = r.end;
            return;
        }
        self.begin_move(select);
        self.cursor = self.next_boundary(self.cursor);
    }

    pub fn move_word_left(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.prev_word(self.cursor);
    }

    pub fn move_word_right(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.next_word(self.cursor);
    }

    pub fn move_home(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.line_start(self.cursor);
    }

    pub fn move_end(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.line_end(self.cursor);
    }

    pub fn move_doc_start(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = 0;
    }

    pub fn move_doc_end(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.text.len();
    }

    pub fn move_up(&mut self, select: bool) -> bool {
        if !self.multiline {
            return false;
        }
        let CursorPos { line, col } = self.cursor_pos();
        if line == 0 {
            return false;
        }
        self.begin_move(select);
        self.cursor = self.offset_at(line - 1, col);
        true
    }

    pub fn move_down(&mut self, select: bool) -> bool {
        if !self.multiline {
            return false;
        }
        let CursorPos { line, col } = self.cursor_pos();
        if line + 1 >= self.line_count() {
            return false;
        }
        self.begin_move(select);
        self.cursor = self.offset_at(line + 1, col);
        true
    }

    pub fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.anchor = None;
        self.cursor = self.offset_at(line, col);
    }

    fn delete_selection(&mut self) -> bool {
        match self.selection() {
            Some(r) => {
                self.text.replace_range(r.clone(), "");
                self.cursor = r.start;
                self.anchor = None;
                self.normalize_positions();
                true
            }
            None => {
                self.anchor = None;
                false
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if matches!(c, '\r' | '\n') && !self.multiline {
            return;
        }
        let c = if c == '\r' { '\n' } else { c };
        self.insert_str(c.encode_utf8(&mut [0; 4]));
    }

    pub fn insert_str(&mut self, s: &str) {
        let s = normalized_text(s, self.multiline);

        if let Some(r) = self.selection() {
            self.cursor = r.start + s.len();
            self.text.replace_range(r, &s);
        } else {
            self.text.insert_str(self.cursor, &s);
            self.cursor += s.len();
        }
        self.anchor = None;
        self.normalize_positions();
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.prev_boundary(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.normalize_positions();
    }

    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.next_boundary(self.cursor);
        self.text.replace_range(self.cursor..end, "");
        self.normalize_positions();
    }

    pub fn delete_word_left(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.prev_word(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.normalize_positions();
    }

    pub fn delete_to_line_end(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.line_end(self.cursor);
        self.text.replace_range(self.cursor..end, "");
        self.normalize_positions();
    }

    pub fn delete_to_line_start(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.line_start(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.normalize_positions();
    }

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    pub fn cursor_pos(&self) -> CursorPos {
        Self::pos_of(&self.text, self.cursor)
    }

    pub fn pos_of(text: &str, offset: usize) -> CursorPos {
        let offset = Self::floor_boundary(text, offset);
        let before = &text[..offset];
        let line = before.matches('\n').count();
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = UnicodeWidthStr::width(&text[line_start..offset]);
        CursorPos { line, col }
    }

    pub fn offset_at(&self, line: usize, col: usize) -> usize {
        let mut start = 0;
        for (i, l) in self.text.split('\n').enumerate() {
            if i == line {
                let mut width = 0;
                for (gi, g) in l.grapheme_indices(true) {
                    let w = UnicodeWidthStr::width(g);
                    if width + w > col {
                        return start + gi;
                    }
                    width += w;
                }
                return start + l.len();
            }
            start += l.len() + 1;
        }
        self.text.len()
    }

    pub fn width(&self) -> usize {
        UnicodeWidthStr::width(self.text.as_str())
    }
}
