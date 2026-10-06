//! Unique scoped targets. Replaces `find(row,col)` + magic offsets.
//!
//! A [`Target`] resolves a needle to exactly one visible point: zero or
//! 2+ matches fail the case (no first-hit roulette), the hit is
//! visibility- and bounds-checked, and [`Target::refresh`] re-resolves
//! after scroll/resize/overlay instead of reusing a stale cell. Hand the
//! point to [`typed_input`](super::typed_input) in `(col, row)` order.
//!
//! Positions are exact for ASCII needles/rows (the suite's case); a match
//! past a non-ASCII row prefix falls back to the global first hit plus a
//! containment check, and fails when that hit is outside the scope.

use tuiscotti::Screen;
use unicode_width::UnicodeWidthStr;

use super::{DEFAULT_WAIT, Session, row_text, screen_find, try_wait_screen};

/// A column×row rectangle, 0-based inclusive. See [`Scope`].
#[derive(Debug, Clone, Copy)]
pub struct Scope {
    col_start: u16,
    col_end: u16,
    row_start: u16,
    row_end: u16,
}

impl Scope {
    /// Whole visible grid.
    #[must_use]
    pub fn full() -> Self {
        Self {
            col_start: 0,
            col_end: u16::MAX,
            row_start: 0,
            row_end: u16::MAX,
        }
    }

    /// Rows `start..=end`, all columns (panes stacked vertically).
    #[must_use]
    pub fn rows(start: u16, end: u16) -> Self {
        assert!(start <= end, "scope rows run backwards: {start}..={end}");
        Self {
            col_start: 0,
            col_end: u16::MAX,
            row_start: start,
            row_end: end,
        }
    }

    /// Columns `start..=end`, all rows (panes side by side).
    #[must_use]
    pub fn cols(start: u16, end: u16) -> Self {
        assert!(start <= end, "scope cols run backwards: {start}..={end}");
        Self {
            col_start: start,
            col_end: end,
            row_start: 0,
            row_end: u16::MAX,
        }
    }

    /// Exact rectangle, 0-based inclusive.
    #[must_use]
    pub fn rect(col_start: u16, row_start: u16, col_end: u16, row_end: u16) -> Self {
        assert!(
            col_start <= col_end && row_start <= row_end,
            "scope rect runs backwards: ({col_start}, {row_start})..=({col_end}, {row_end})"
        );
        Self {
            col_start,
            col_end,
            row_start,
            row_end,
        }
    }

    /// Clamp to the grid; `None` when the scope misses the grid entirely
    /// (a test bug, failed by the caller — never silently clamped).
    fn intersect(self, cols: u16, rows: u16) -> Option<(u16, u16, u16, u16)> {
        if self.col_start >= cols || self.row_start >= rows {
            return None;
        }
        Some((
            self.col_start,
            self.col_end.min(cols.saturating_sub(1)),
            self.row_start,
            self.row_end.min(rows.saturating_sub(1)),
        ))
    }

    fn contains(self, col: u16, row: u16) -> bool {
        (self.col_start..=self.col_end).contains(&col)
            && (self.row_start..=self.row_end).contains(&row)
    }
}

/// A resolved unique point: the needle, its scope, and the point.
#[derive(Debug, Clone)]
pub struct Target {
    needle: String,
    scope: Scope,
    col: u16,
    row: u16,
    cols: u16,
    rows: u16,
}

impl Target {
    /// Wait (session deadline) for `needle` to resolve to exactly one
    /// visible point on the full grid. Fails on zero or 2+ matches.
    pub fn resolve(s: &mut Session, needle: &str) -> Self {
        Self::resolve_in(s, Scope::full(), needle)
    }

    /// [`resolve`](Self::resolve) inside `scope` — the disambiguator when
    /// the needle legitimately appears twice (two panes, header + body).
    pub fn resolve_in(s: &mut Session, scope: Scope, needle: &str) -> Self {
        assert!(!needle.is_empty(), "target needle must not be empty");
        assert!(
            !needle.contains('\n'),
            "target needle must be single-line, got {needle:?}"
        );
        let mut last: Option<(Vec<(u16, u16)>, u16, u16)> = None;
        let outcome = try_wait_screen(s, DEFAULT_WAIT, |screen| {
            let (cols, rows) = (screen.cols(), screen.rows());
            let hits = find_in_scope(screen, scope, needle, cols, rows);
            let unique = hits.len() == 1;
            last = Some((hits, cols, rows));
            unique
        });
        match (outcome, last) {
            (Ok(_), Some((hits, cols, rows))) => {
                let (row, col) = hits[0];
                Self {
                    needle: needle.to_string(),
                    scope,
                    col,
                    row,
                    cols,
                    rows,
                }
            }
            (Ok(_), None) => unreachable!("try_wait_screen always evaluates once"),
            (Err(e), last) => {
                let seen = last.map_or_else(
                    || "no observation".to_string(),
                    |(hits, cols, rows)| {
                        format!("{} match(es) on {cols}x{rows}: {hits:?}", hits.len())
                    },
                );
                panic!("unique target `{needle}` never resolved ({seen}): {e:#}");
            }
        }
    }

    /// Re-resolve after scroll/resize/overlay. Fails when the target is
    /// gone or ambiguous; updates the point and grid otherwise.
    pub fn refresh(&mut self, s: &mut Session) {
        let fresh = Self::resolve_in(s, self.scope, &self.needle);
        self.col = fresh.col;
        self.row = fresh.row;
        self.cols = fresh.cols;
        self.rows = fresh.rows;
    }

    /// [`refresh`](Self::refresh), failing unless the point moved — the
    /// proof a scroll had an effect.
    ///
    /// Ported reference API; flows adopt it per-target once live scroll
    /// effects are pinned.
    #[allow(dead_code)]
    pub fn refresh_moved(&mut self, s: &mut Session) {
        let (col, row) = (self.col, self.row);
        self.refresh(s);
        assert!(
            (self.col, self.row) != (col, row),
            "target `{}` never moved from ({col}, {row}) — the scroll had no effect",
            self.needle
        );
    }

    /// [`refresh`](Self::refresh), failing when the point moved — the
    /// proof an overlay/resize left the content in place.
    pub fn refresh_same(&mut self, s: &mut Session) {
        let (col, row) = (self.col, self.row);
        self.refresh(s);
        assert!(
            (self.col, self.row) == (col, row),
            "target `{}` moved ({col}, {row})→({}, {}) — expected in place",
            self.needle,
            self.col,
            self.row
        );
    }

    /// Click point in `(col, row)` input order.
    #[must_use]
    pub fn point(&self) -> (u16, u16) {
        (self.col, self.row)
    }

    /// 0-based column.
    #[must_use]
    pub fn col(&self) -> u16 {
        self.col
    }

    /// 0-based row.
    #[must_use]
    pub fn row(&self) -> u16 {
        self.row
    }

    /// Grid size at resolve time.
    #[must_use]
    pub fn grid(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Last cell of the needle span — the drag endpoint that replaces
    /// magic `col + len` arithmetic.
    #[must_use]
    pub fn span_end(&self) -> (u16, u16) {
        let w = UnicodeWidthStr::width(self.needle.as_str()).max(1) as u16;
        (self.col + w - 1, self.row)
    }

    /// Explicit offset from the point, bounds-checked against the
    /// resolve-time grid. Fails off-grid (no silent clamp).
    #[must_use]
    pub fn offset(&self, dcol: i16, drow: i16) -> (u16, u16) {
        let col = i32::from(self.col) + i32::from(dcol);
        let row = i32::from(self.row) + i32::from(drow);
        assert!(
            (0..i32::from(self.cols)).contains(&col) && (0..i32::from(self.rows)).contains(&row),
            "offset ({dcol}, {drow}) from ({}, {}) leaves the {}x{} grid",
            self.col,
            self.row,
            self.cols,
            self.rows
        );
        (col as u16, row as u16)
    }
}

/// All `(row, col)` matches of `needle` inside `scope`, top-to-bottom.
/// Exact for ASCII rows; a match past a non-ASCII row prefix falls back
/// to the global first hit plus containment (see module docs).
fn find_in_scope(
    screen: &Screen,
    scope: Scope,
    needle: &str,
    cols: u16,
    rows: u16,
) -> Vec<(u16, u16)> {
    let Some((c0, c1, r0, r1)) = scope.intersect(cols, rows) else {
        return Vec::new();
    };
    // Fast path: every in-scope row is ASCII, so byte offsets are columns.
    let ascii_rows = (r0..=r1).all(|row| row_text(screen, row).is_ascii());
    if ascii_rows {
        let mut hits = Vec::new();
        for row in r0..=r1 {
            let line = row_text(screen, row);
            let trimmed = line.trim_end();
            for (off, _) in trimmed.match_indices(needle) {
                let col = off as u16;
                if col + (needle.len() as u16) - 1 > c1 || col < c0 {
                    continue;
                }
                if screen.get(col, row).is_some_and(|c| !c.symbol.is_empty()) {
                    hits.push((row, col));
                }
            }
        }
        return hits;
    }
    // Fallback: global first hit (wide-aware) + containment.
    match screen_find(screen, needle) {
        Some((row, col)) if scope.contains(col, row) => vec![(row, col)],
        _ => Vec::new(),
    }
}

#[test]
fn scope_math_is_exact() {
    let full = Scope::full();
    assert_eq!(full.intersect(80, 24), Some((0, 79, 0, 23)));
    assert_eq!(Scope::rows(2, 5).intersect(80, 24), Some((0, 79, 2, 5)));
    assert_eq!(Scope::cols(3, 9).intersect(80, 24), Some((3, 9, 0, 23)));
    assert_eq!(
        Scope::rect(1, 2, 200, 99).intersect(80, 24),
        Some((1, 79, 2, 23))
    );
    assert_eq!(Scope::rows(30, 40).intersect(80, 24), None);
    assert!(Scope::rows(2, 5).contains(70, 3));
    assert!(!Scope::rows(2, 5).contains(70, 6));
    let t = Target {
        needle: "attempts = 3".to_string(),
        scope: Scope::full(),
        col: 10,
        row: 4,
        cols: 120,
        rows: 40,
    };
    assert_eq!(t.point(), (10, 4));
    assert_eq!(t.span_end(), (21, 4));
    assert_eq!(t.offset(2, 0), (12, 4));
}
