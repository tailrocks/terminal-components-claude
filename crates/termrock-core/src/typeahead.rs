//! Type-ahead prefix matching (ported from PR #26 `src/core/typeahead.rs`,
//! commit f4867e9b5).

use std::time::{Duration, Instant};

use crate::event::Key;
use crate::keys::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockMark {
    Instant(Instant),
    Elapsed(Duration),
}

impl ClockMark {
    fn duration_since(self, earlier: Self) -> Option<Duration> {
        match (earlier, self) {
            (Self::Instant(earlier), Self::Instant(now)) => Some(now.duration_since(earlier)),
            (Self::Elapsed(earlier), Self::Elapsed(now)) => Some(now.saturating_sub(earlier)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAhead {
    buf: String,
    last: Option<ClockMark>,

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
        let chord = key.ctrl() || key.alt() || key.mods.contains(KeyModifiers::SUPER);
        match key.code {
            KeyCode::Char(c) if !chord && c.is_alphanumeric() => Some(c),
            _ => None,
        }
    }

    pub fn feed(&mut self, c: char) -> &str {
        self.feed_at(c, Instant::now())
    }

    pub fn feed_at(&mut self, c: char, now: Instant) -> &str {
        self.feed_at_clock(c, ClockMark::Instant(now))
    }

    pub fn feed_at_duration(&mut self, c: char, now: Duration) -> &str {
        self.feed_at_clock(c, ClockMark::Elapsed(now))
    }

    fn feed_at_clock(&mut self, c: char, now: ClockMark) -> &str {
        if self
            .last
            .and_then(|last| now.duration_since(last))
            .is_none_or(|elapsed| elapsed > self.timeout)
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

    pub fn find_indexed<'a>(
        prefix: &str,
        len: usize,
        from: usize,
        mut label: impl FnMut(usize) -> &'a str,
        mut eligible: impl FnMut(usize) -> bool,
    ) -> Option<usize> {
        if len == 0 || prefix.is_empty() {
            return None;
        }
        let mut chars = prefix.chars();
        let first = chars.next()?;
        let repeated = chars.all(|c| c == first);
        let repeated_cycle = repeated && prefix.chars().count() > 1;
        let needle = if repeated_cycle {
            first.to_string()
        } else {
            prefix.to_owned()
        };
        let from = from % len;
        let mut index = if repeated_cycle {
            if from == len - 1 { 0 } else { from + 1 }
        } else {
            from
        };
        for _ in 0..len {
            if eligible(index) && label(index).to_lowercase().starts_with(&needle) {
                return Some(index);
            }
            index = if index == len - 1 { 0 } else { index + 1 };
        }
        None
    }
}
