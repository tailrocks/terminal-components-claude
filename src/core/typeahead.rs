//! Type-ahead: typing a prefix jumps to the matching row, as in macOS
//! lists, outline views, popup buttons and menus.
//!
//! Keys arrive without a timestamp, so the buffer keeps its own clock
//! (`std::time::Instant`); a pause longer than [`TypeAhead::timeout`]
//! starts a new prefix.

use std::time::{Duration, Instant};

/// A typed prefix that resets after a pause.
#[derive(Debug, Clone)]
pub struct TypeAhead {
    buf: String,
    last: Option<Instant>,
    /// Pause after which the next letter starts a new prefix. Default 1 s.
    pub timeout: Duration,
}

impl Default for TypeAhead {
    fn default() -> Self {
        Self {
            buf: String::new(),
            last: None,
            timeout: Duration::from_millis(1000),
        }
    }
}

impl TypeAhead {
    /// Add `c` to the prefix (starting over after a pause) and return it.
    pub fn feed(&mut self, c: char) -> &str {
        let now = Instant::now();
        if self
            .last
            .is_none_or(|t| now.duration_since(t) > self.timeout)
        {
            self.buf.clear();
        }
        self.last = Some(now);
        self.buf.extend(c.to_lowercase());
        &self.buf
    }

    /// Forget the prefix.
    pub fn reset(&mut self) {
        self.buf.clear();
        self.last = None;
    }

    /// The first row at or after `from` (wrapping) whose label starts with
    /// `prefix`, case-insensitively, skipping rows `eligible` rejects. A
    /// single repeated letter cycles through the rows starting with it.
    pub fn find<'a>(
        prefix: &str,
        labels: impl ExactSizeIterator<Item = &'a str> + Clone,
        from: usize,
        eligible: impl Fn(usize) -> bool,
    ) -> Option<usize> {
        let n = labels.len();
        if n == 0 || prefix.is_empty() {
            return None;
        }
        let mut chars = prefix.chars();
        let first = chars.next()?;
        let repeated = chars.all(|c| c == first);
        let (needle, start) = if repeated && prefix.chars().count() > 1 {
            // "ooo": cycle among rows starting with "o", after the current
            (first.to_string(), from + 1)
        } else {
            (prefix.to_owned(), from)
        };
        let all: Vec<&str> = labels.collect();
        (0..n)
            .map(|k| (start + k) % n)
            .find(|&i| eligible(i) && all[i].to_lowercase().starts_with(&needle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prefix_finds_and_a_repeated_letter_cycles() {
        let labels = ["Alpha", "beta", "Bravo", "charlie"];
        let f = |p: &str, from| TypeAhead::find(p, labels.iter().copied(), from, |_| true);
        assert_eq!(f("b", 0), Some(1));
        assert_eq!(f("br", 0), Some(2));
        assert_eq!(f("bb", 1), Some(2), "a repeated letter moves on");
        assert_eq!(f("bb", 2), Some(1), "and wraps");
        assert_eq!(f("z", 0), None);
        assert_eq!(
            TypeAhead::find("b", labels.iter().copied(), 0, |i| i != 1),
            Some(2),
            "ineligible rows are skipped"
        );
        let mut t = TypeAhead::default();
        t.feed('B');
        assert_eq!(t.feed('r'), "br");
        t.timeout = Duration::ZERO;
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(t.feed('c'), "c", "a pause starts over");
    }
}
