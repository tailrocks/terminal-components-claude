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

/// A rolling type-ahead prefix: feed printable characters, find the next
/// row whose label starts with the prefix. A pause longer than the timeout
/// starts a fresh prefix, and a repeated single letter cycles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAhead {
    buf: String,
    last: Option<ClockMark>,

    /// How long the prefix survives without a new character.
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
    /// One second, the standard type-ahead window.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);

    /// Build with a custom timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::default()
        }
    }

    /// The bare alphanumeric character a key press types, if any.
    /// Punctuation and modified keys stay the app's (`?` help, `/` find).
    pub fn char_of(key: &Key) -> Option<char> {
        let chord = key.ctrl() || key.alt() || key.mods.contains(KeyModifiers::SUPER);
        match key.code {
            KeyCode::Char(c) if !chord && c.is_alphanumeric() => Some(c),
            _ => None,
        }
    }

    /// Feed a character now.
    pub fn feed(&mut self, c: char) -> &str {
        self.feed_at(c, Instant::now())
    }

    /// Feed a character at a given instant (tests, synthetic input).
    pub fn feed_at(&mut self, c: char, now: Instant) -> &str {
        self.feed_at_clock(c, ClockMark::Instant(now))
    }

    /// Feed a character at an elapsed-duration clock value.
    ///
    /// Use this when the owning runtime provides its own monotonic clock. A
    /// switch between this clock and the `Instant` clock used by [`Self::feed`]
    /// or [`Self::feed_at`] starts a fresh prefix because the clock epochs are
    /// not comparable.
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

    /// Forget the prefix.
    pub fn reset(&mut self) {
        self.buf.clear();
        self.last = None;
    }

    /// Find the first eligible row at or after `from` (wrapping) whose label
    /// starts with `prefix`. A prefix of one letter repeated (like `bb`)
    /// moves on from the cursor so typing the same letter cycles.
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
            // Same letter again: cycle to the next match instead of staying.
            (first.to_string(), from + 1)
        } else {
            (prefix.to_owned(), from)
        };
        let all: Vec<&str> = labels.collect();
        (0..n)
            .map(|k| (start + k) % n)
            .find(|&i| eligible(i) && all[i].to_lowercase().starts_with(&needle))
    }

    /// Find a matching row through indexed access without collecting labels.
    ///
    /// `len` is the caller's concrete search bound. Only indices in `0..len`
    /// are queried; pass the number of loaded rows, not an unknown total or an
    /// unloaded sentinel.
    ///
    /// `eligible` runs before `label` for each visited index. Labels are read
    /// only for eligible rows, and search stops at the first match. For pure
    /// callbacks, matching order and prefix semantics match [`Self::find`].
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

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

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
    fn indexed_lookup_matches_existing_prefix_cycle_and_miss_results() {
        let labels = ["Alpha", "beta", "Bravo", "charlie"];
        let cases = [
            ("b", 0),
            ("br", 0),
            ("bb", 1),
            ("bb", 2),
            ("b", 3),
            ("z", 0),
        ];

        for (prefix, from) in cases {
            let existing = TypeAhead::find(prefix, labels.iter().copied(), from, |_| true);
            let indexed =
                TypeAhead::find_indexed(prefix, labels.len(), from, |i| labels[i], |_| true);
            assert_eq!(indexed, existing, "prefix={prefix}, from={from}");
        }
    }

    #[test]
    fn indexed_lookup_checks_eligibility_before_labels_and_stops_at_match() {
        let labels = ["disabled", "miss", "Match", "match later"];
        let calls = RefCell::new(Vec::new());
        let found = TypeAhead::find_indexed(
            "match",
            labels.len(),
            0,
            |i| {
                calls.borrow_mut().push(("label", i));
                labels[i]
            },
            |i| {
                calls.borrow_mut().push(("eligible", i));
                i != 0
            },
        );

        assert_eq!(found, Some(2));
        assert_eq!(
            calls.into_inner(),
            vec![
                ("eligible", 0),
                ("eligible", 1),
                ("label", 1),
                ("eligible", 2),
                ("label", 2),
            ]
        );
    }

    #[test]
    fn indexed_lookup_does_not_access_empty_searches() {
        let label_calls = Cell::new(0_usize);
        let eligibility_calls = Cell::new(0_usize);
        let label = |_| {
            label_calls.set(label_calls.get() + 1);
            "alpha"
        };
        let eligible = |_| {
            eligibility_calls.set(eligibility_calls.get() + 1);
            true
        };

        assert_eq!(TypeAhead::find_indexed("a", 0, 0, label, eligible), None);
        assert_eq!(label_calls.get(), 0);
        assert_eq!(eligibility_calls.get(), 0);
        assert_eq!(TypeAhead::find_indexed("", 1, 0, label, eligible), None);
        assert_eq!(label_calls.get(), 0);
        assert_eq!(eligibility_calls.get(), 0);
    }

    #[test]
    fn indexed_lookup_normalizes_maximum_start_without_overflow() {
        let labels = ["Alpha", "beta", "Bravo", "charlie"];
        assert_eq!(
            TypeAhead::find_indexed("c", labels.len(), usize::MAX, |i| labels[i], |_| true),
            Some(3)
        );
        assert_eq!(
            TypeAhead::find_indexed("bb", labels.len(), usize::MAX, |i| labels[i], |_| true),
            Some(1)
        );
    }

    #[test]
    fn indexed_lookup_wraps_after_the_current_row() {
        let labels = ["Alpha", "beta", "Bravo", "charlie"];
        assert_eq!(
            TypeAhead::find_indexed("b", labels.len(), 3, |i| labels[i], |_| true),
            Some(1)
        );
        assert_eq!(
            TypeAhead::find_indexed("bb", labels.len(), 2, |i| labels[i], |_| true),
            Some(1)
        );
    }

    #[test]
    fn the_prefix_resets_after_duration_clock_timeout_and_not_before() {
        let mut t = TypeAhead::with_timeout(Duration::from_millis(500));
        assert_eq!(t.feed_at_duration('b', Duration::ZERO), "b");
        assert_eq!(t.feed_at_duration('r', Duration::from_millis(500)), "br");
        assert_eq!(t.feed_at_duration('a', Duration::from_secs(1)), "bra");
        let just_after_timeout = Duration::from_millis(1_500) + Duration::from_nanos(1);
        assert_eq!(t.feed_at_duration('c', just_after_timeout), "c");
    }

    #[test]
    fn switching_clock_domains_starts_a_fresh_prefix() {
        let start = Instant::now();
        let mut t = TypeAhead::default();
        assert_eq!(t.feed_at('b', start), "b");
        assert_eq!(t.feed_at_duration('r', Duration::ZERO), "r");
        assert_eq!(t.feed_at('a', start + Duration::from_secs(1)), "a");
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
