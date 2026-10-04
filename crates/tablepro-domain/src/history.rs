//! Query history domain types.

/// Origin of a history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistorySource {
    /// Editor execution.
    Editor,
    /// Explain-plan execution.
    Explain,
    /// Table browsing.
    Browsing,
    /// Row edits.
    RowEdits,
    /// Structure inspection.
    Structure,
}

impl HistorySource {
    /// Human-readable source label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Editor => "Editor",
            Self::Explain => "Explain",
            Self::Browsing => "Table Browsing",
            Self::RowEdits => "Row Edits",
            Self::Structure => "Structure Changes",
        }
    }
}

/// One query-history record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Stable id.
    pub id: usize,
    /// SQL text.
    pub sql: String,
    /// Connection name.
    pub connection: String,
    /// Database name.
    pub database: String,
    /// Schema name.
    pub schema: String,
    /// Deterministic age in minutes.
    pub minutes_ago: u32,
    /// Duration in milliseconds.
    pub duration_ms: Option<u32>,
    /// Returned/affected rows.
    pub rows: Option<usize>,
    /// Error, when execution failed.
    pub error: Option<String>,
    /// Origin surface.
    pub source: HistorySource,
}

impl HistoryEntry {
    /// Whether execution succeeded.
    pub fn ok(&self) -> bool {
        self.error.is_none()
    }

    /// First line for compact rows.
    pub fn first_line(&self) -> String {
        let mut lines = self.sql.lines();
        let first = lines.next().unwrap_or_default().trim();
        if lines.next().is_some() {
            format!("{first} …")
        } else {
            first.to_owned()
        }
    }

    /// Stable relative time label.
    pub fn when(&self) -> String {
        match self.minutes_ago {
            0 => "just now".to_owned(),
            n if n < 60 => format!("{n} min ago"),
            n if n < 1_440 => format!("{} h ago", n / 60),
            n => format!("{} d ago", n / 1_440),
        }
    }

    /// Stable duration label.
    pub fn duration(&self) -> String {
        match self.duration_ms {
            None => "–".to_owned(),
            Some(0) => "<1 ms".to_owned(),
            Some(n) if n < 1_000 => format!("{n} ms"),
            Some(n) if n < 60_000 => format!("{:.2} s", f64::from(n) / 1_000.0),
            Some(n) => format!("{}m {}s", n / 60_000, (n % 60_000) / 1_000),
        }
    }
}

/// Bounded newest-first history.
#[derive(Debug, Clone, Default)]
pub struct History {
    /// Entries, newest first.
    pub entries: Vec<HistoryEntry>,
    next_id: usize,
}

impl History {
    /// Build the deterministic demo history.
    pub fn seeded() -> Self {
        let mut out = Self::default();
        let rows = [
            (
                "SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 200",
                "Production",
                "acme_prod",
                4,
                Some(38),
                Some(200),
                None,
                HistorySource::Editor,
            ),
            (
                "SELECT count(*) FROM orders WHERE created_at >= '2025-06-01'",
                "Production",
                "acme_prod",
                12,
                Some(412),
                Some(1),
                None,
                HistorySource::Editor,
            ),
            (
                "EXPLAIN ANALYZE SELECT * FROM orders WHERE customer_id = '3f1a…'",
                "Production",
                "acme_prod",
                15,
                Some(9),
                Some(4),
                None,
                HistorySource::Explain,
            ),
            (
                "SELECT * FROM customers ORDER BY created_at DESC",
                "Production",
                "acme_prod",
                40,
                Some(21),
                Some(1_000),
                None,
                HistorySource::Browsing,
            ),
            (
                "UPDATE orders SET status = 'shipped' WHERE id = '9c2e…'",
                "Production",
                "acme_prod",
                58,
                Some(3),
                Some(1),
                None,
                HistorySource::RowEdits,
            ),
            (
                "SELECT * FROM ordres",
                "Production",
                "acme_prod",
                96,
                None,
                None,
                Some("relation \\\"ordres\\\" does not exist"),
                HistorySource::Editor,
            ),
            (
                "ALTER TABLE orders ADD COLUMN is_gift boolean NOT NULL DEFAULT false",
                "Development",
                "acme_dev",
                1_560,
                Some(88),
                Some(0),
                None,
                HistorySource::Structure,
            ),
        ];
        for (sql_text, connection, database, minutes, duration, rows_count, error, source) in rows {
            out.push(HistoryEntry {
                id: 0,
                sql: sql_text.to_owned(),
                connection: connection.to_owned(),
                database: database.to_owned(),
                schema: "public".to_owned(),
                minutes_ago: minutes,
                duration_ms: duration,
                rows: rows_count,
                error: error.map(str::to_owned),
                source,
            });
        }
        out
    }

    /// Whether this history contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add an entry and retain the newest 10,000 records.
    pub fn push(&mut self, mut entry: HistoryEntry) {
        self.next_id = self.next_id.saturating_add(1);
        entry.id = self.next_id;
        self.entries.insert(0, entry);
        self.entries.truncate(10_000);
    }

    /// Search with case-insensitive AND semantics.
    pub fn search<'a>(
        &'a self,
        query: &str,
        connection: Option<&str>,
        failed_only: bool,
    ) -> Vec<&'a HistoryEntry> {
        let terms: Vec<String> = query
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        self.entries
            .iter()
            .filter(|entry| {
                connection.is_none_or(|want| entry.connection.eq_ignore_ascii_case(want))
            })
            .filter(|entry| !failed_only || !entry.ok())
            .filter(|entry| {
                terms
                    .iter()
                    .all(|term| entry.sql.to_ascii_lowercase().contains(term))
            })
            .collect()
    }
}
