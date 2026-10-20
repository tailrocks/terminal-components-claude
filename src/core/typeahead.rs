use std::time::{Duration, Instant};

use crate::core::event::Key;
use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct TypeAhead {
    buf: String,
    last: Option<Instant>,

    pub timeout: Duration,
}

impl Default for TypeAhead {
    fn default() -> Self {
        Self {
            buf: String::new(),
            last: None,
            timeout: Self::DEFAULT_TIMEOUT,
        }
    }
}

impl TypeAhead {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::default()
        }
    }

    pub fn char_of(key: &Key) -> Option<char> {
        match key.code {
            KeyCode::Char(c) if !key.chord() && c.is_alphanumeric() => Some(c),
            _ => None,
        }
    }

    pub fn feed(&mut self, c: char) -> &str {
        self.feed_at(c, Instant::now())
    }

    pub fn feed_at(&mut self, c: char, now: Instant) -> &str {
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

    pub fn reset(&mut self) {
        self.buf.clear();
        self.last = None;
    }

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

    #[test]
    fn the_prefix_resets_after_the_timeout_and_not_before() {
        let t0 = Instant::now();
        let mut t = TypeAhead::with_timeout(Duration::from_millis(500));
        t.feed_at('b', t0);
        assert_eq!(t.feed_at('r', t0 + Duration::from_millis(400)), "br");
        assert_eq!(
            t.feed_at('a', t0 + Duration::from_millis(800)),
            "bra",
            "the pause is measured from the last key"
        );
        assert_eq!(t.feed_at('c', t0 + Duration::from_millis(1400)), "c");
        assert_eq!(TypeAhead::default().timeout, TypeAhead::DEFAULT_TIMEOUT);
    }
}
