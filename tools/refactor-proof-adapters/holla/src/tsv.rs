//! Tiny TSV reader for frozen adapter inventories.
//!
//! The frozen registers contain no quoted tabs or embedded newlines (every
//! row has the header width), so a strict line/field split is exact. Any
//! ragged row fails the whole parse instead of silently shifting columns.

/// Parsed TSV table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table<'a> {
    /// Column names.
    pub header: Vec<&'a str>,
    /// Row fields aligned to `header`.
    pub rows: Vec<Vec<&'a str>>,
}

impl<'a> Table<'a> {
    /// Parse a TSV document. Empty trailing lines are ignored.
    #[must_use]
    pub fn parse(text: &'a str) -> Option<Self> {
        let mut lines = text.lines().filter(|line| !line.is_empty());
        let header: Vec<&str> = lines.next()?.split('\t').collect();
        if header.is_empty() {
            return None;
        }
        let width = header.len();
        let mut rows = Vec::new();
        for line in lines {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != width {
                return None;
            }
            rows.push(fields);
        }
        Some(Self { header, rows })
    }

    /// Named field of `row`.
    #[must_use]
    pub fn get(&self, row: usize, name: &str) -> Option<&'a str> {
        let column = self.header.iter().position(|item| *item == name)?;
        self.rows.get(row)?.get(column).copied()
    }

    /// Number of data rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the table has no data rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn ragged_row_fails_closed() {
        assert!(Table::parse("a\tb\n1\n").is_none());
        let table = Table::parse("a\tb\n1\t2\n").expect("well-formed");
        assert_eq!(table.len(), 1);
        assert_eq!(table.get(0, "b"), Some("2"));
        assert_eq!(table.get(0, "zzz"), None);
        assert_eq!(table.get(9, "a"), None);
    }
}
