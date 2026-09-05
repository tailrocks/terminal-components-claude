//! Query-history, completion and quick-switcher models.

use junie_tui::fuzzy;

use crate::db::{Catalog, ColType, Table};
use crate::sql::{FUNCTIONS, KEYWORDS, TokKind, tokenize};

/// Origin of a history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistorySource {
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
    #[allow(
        dead_code,
        reason = "history labels remain available to the private history adapter"
    )]
    pub(crate) const fn label(self) -> &'static str {
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
pub(crate) struct HistoryEntry {
    /// Stable id.
    pub(crate) id: usize,
    /// SQL text.
    pub(crate) sql: String,
    /// Connection name.
    pub(crate) connection: String,
    /// Database name.
    pub(crate) database: String,
    /// Schema name.
    pub(crate) schema: String,
    /// Deterministic age in minutes.
    pub(crate) minutes_ago: u32,
    /// Duration in milliseconds.
    pub(crate) duration_ms: Option<u32>,
    /// Returned/affected rows.
    pub(crate) rows: Option<usize>,
    /// Error, when execution failed.
    pub(crate) error: Option<String>,
    /// Origin surface.
    pub(crate) source: HistorySource,
}

impl HistoryEntry {
    /// Whether execution succeeded.
    pub(crate) fn ok(&self) -> bool {
        self.error.is_none()
    }
    /// First line for compact rows.
    pub(crate) fn first_line(&self) -> String {
        let mut lines = self.sql.lines();
        let first = lines.next().unwrap_or_default().trim();
        if lines.next().is_some() {
            format!("{first} …")
        } else {
            first.to_owned()
        }
    }
    /// Stable relative time label.
    pub(crate) fn when(&self) -> String {
        match self.minutes_ago {
            0 => "just now".to_owned(),
            n if n < 60 => format!("{n} min ago"),
            n if n < 1_440 => format!("{} h ago", n / 60),
            n => format!("{} d ago", n / 1_440),
        }
    }
    /// Stable duration label.
    #[allow(
        dead_code,
        reason = "history duration remains available to the private history adapter"
    )]
    pub(crate) fn duration(&self) -> String {
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
    pub(crate) entries: Vec<HistoryEntry>,
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
    pub(crate) fn push(&mut self, mut entry: HistoryEntry) {
        self.next_id = self.next_id.saturating_add(1);
        entry.id = self.next_id;
        self.entries.insert(0, entry);
        self.entries.truncate(10_000);
    }
    /// Search with case-insensitive AND semantics.
    pub(crate) fn search<'a>(
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

/// Completion item category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompletionKind {
    /// SQL keyword.
    Keyword,
    /// Relation name.
    Table,
    /// Column name.
    Column,
    /// SQL function.
    Function,
}

/// SQL completion item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// Text inserted into the editor.
    pub text: String,
    /// Display label.
    pub label: String,
    /// Completion category.
    pub(crate) kind: CompletionKind,
    /// Fuzzy-match score.
    pub score: u32,
}

/// SQL context at a cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Clause {
    /// Beginning of a statement.
    Statement,
    /// After a relation-introducing clause.
    Relation,
    /// After a projection or predicate clause.
    Column,
    /// General expression context.
    Expression,
}

/// Infer completion context from the preceding token.
pub(crate) fn context(source: &str, cursor: usize) -> Clause {
    let prefix = source.get(..cursor.min(source.len())).unwrap_or(source);
    let tokens = tokenize(prefix);
    let last = tokens
        .iter()
        .rev()
        .find(|token| !matches!(token.kind, TokKind::Whitespace | TokKind::Comment))
        .and_then(|token| prefix.get(token.start..token.end))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let prior = tokens
        .iter()
        .rev()
        .skip(1)
        .find(|token| !matches!(token.kind, TokKind::Whitespace | TokKind::Comment))
        .and_then(|token| prefix.get(token.start..token.end))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let word = if matches!(
        tokens.last().map(|token| token.kind),
        Some(TokKind::Whitespace)
    ) {
        prior
    } else {
        last
    };
    match word.as_str() {
        "from" | "join" | "into" | "update" => Clause::Relation,
        "select" | "where" | "and" | "or" | "order" | "by" => Clause::Column,
        "" => Clause::Statement,
        _ => Clause::Expression,
    }
}

/// Complete the current SQL token from catalog names and SQL vocabulary.
pub fn complete(source: &str, cursor: usize, catalog: &Catalog) -> Vec<Completion> {
    let prefix = source.get(..cursor.min(source.len())).unwrap_or(source);
    let needle = prefix
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .next_back()
        .unwrap_or_default();
    let clause = context(source, cursor);
    let mut candidates: Vec<(String, CompletionKind)> = Vec::new();
    if matches!(clause, Clause::Statement | Clause::Expression) {
        candidates.extend(
            KEYWORDS
                .iter()
                .map(|word| ((*word).to_owned(), CompletionKind::Keyword)),
        );
    }
    if matches!(clause, Clause::Relation | Clause::Expression) {
        candidates.extend(
            catalog
                .tables
                .iter()
                .map(|table| (table.name.clone(), CompletionKind::Table)),
        );
    }
    if matches!(clause, Clause::Column | Clause::Expression) {
        candidates.extend(catalog.tables.iter().flat_map(|table| {
            table
                .columns
                .iter()
                .map(|column| (column.name.clone(), CompletionKind::Column))
        }));
    }
    if matches!(clause, Clause::Expression) {
        candidates.extend(
            FUNCTIONS
                .iter()
                .map(|word| ((*word).to_owned(), CompletionKind::Function)),
        );
    }
    let mut out = candidates
        .into_iter()
        .filter_map(|(label, kind)| {
            fuzzy(&label, needle).map(|(score, _)| Completion {
                text: label.clone(),
                label,
                kind,
                score,
            })
        })
        .collect::<Vec<_>>();
    out.sort_by_key(|item| (item.score, item.label.to_ascii_lowercase()));
    out.dedup_by(|left, right| left.label.eq_ignore_ascii_case(&right.label));
    out.truncate(20);
    out
}

/// Whether completion should open automatically at this cursor.
#[allow(
    dead_code,
    reason = "completion trigger remains available to the private editor adapter"
)]
pub(crate) fn auto_trigger(source: &str, cursor: usize) -> bool {
    let prefix = source.get(..cursor.min(source.len())).unwrap_or(source);
    let ch = prefix.chars().next_back();
    matches!(ch, Some('.' | ' ' | '\n') | None)
}

/// A quick-switcher target kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchTarget {
    /// Relation.
    Table,
    /// Query history entry.
    Query,
    /// Connection.
    Connection,
}

/// One quick-switcher result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchItem {
    /// Stable item key.
    pub key: String,
    /// Main label.
    pub label: String,
    /// Secondary metadata.
    pub detail: String,
    /// Target category.
    pub target: SwitchTarget,
    /// Fuzzy-match score.
    pub score: u32,
}

/// Search index for quick switching.
#[derive(Debug, Clone, Default)]
pub struct SwitcherIndex {
    /// Indexed switch targets.
    pub items: Vec<SwitchItem>,
}

impl SwitcherIndex {
    /// Build an index from app-owned catalog/history/connection data.
    pub fn from_catalog(
        catalog: &Catalog,
        history: &History,
        connections: &[crate::db::Connection],
    ) -> Self {
        let mut items = Vec::new();
        items.extend(catalog.tables.iter().map(|table| SwitchItem {
            key: format!("{}.{}", table.schema, table.name),
            label: table.name.clone(),
            detail: format!("{} · {} rows", table.schema, table.row_count),
            target: SwitchTarget::Table,
            score: 0,
        }));
        items.extend(history.entries.iter().map(|entry| SwitchItem {
            key: format!("history-{}", entry.id),
            label: entry.first_line(),
            detail: entry.when(),
            target: SwitchTarget::Query,
            score: 0,
        }));
        items.extend(connections.iter().map(|connection| SwitchItem {
            key: format!("connection-{}", connection.name),
            label: connection.name.clone(),
            detail: connection.environment.label().to_owned(),
            target: SwitchTarget::Connection,
            score: 0,
        }));
        Self { items }
    }
    /// Return ranked matches for a query.
    pub fn search(&self, query: &str) -> Vec<SwitchItem> {
        let mut out = self
            .items
            .iter()
            .filter_map(|item| {
                fuzzy(&item.label, query).map(|(score, _)| {
                    let mut copy = item.clone();
                    copy.score = score;
                    copy
                })
            })
            .collect::<Vec<_>>();
        out.sort_by_key(|item| {
            (
                matches!(item.target, SwitchTarget::Table)
                    .then_some(0)
                    .unwrap_or(1),
                item.score,
                item.label.len(),
                !item.label.eq_ignore_ascii_case(query),
                item.label.to_ascii_lowercase(),
            )
        });
        out
    }
}

/// Return the columns of a table for filter/editor construction.
#[allow(
    dead_code,
    reason = "column projection remains available to the private editor adapter"
)]
pub(crate) fn table_columns(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

/// Explicit app-level wrapper around the SQL tokenizer for tests and editor UI.
#[allow(
    dead_code,
    reason = "token projection remains available to the private editor adapter"
)]
pub(crate) fn statement_tokens(source: &str) -> Vec<String> {
    tokenize(source)
        .into_iter()
        .filter_map(|token| source.get(token.start..token.end).map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_search_is_multi_term_and() {
        let history = History::seeded();
        assert_eq!(history.search("orders pending", None, false).len(), 1);
    }
    #[test]
    fn completion_is_context_aware() {
        let catalog = Catalog::acme_prod();
        assert!(
            complete("SELECT * FROM ord", 16, &catalog)
                .iter()
                .any(|item| item.label == "orders")
        );
    }
    #[test]
    fn switcher_ranks_tables_first_and_prefix_first() {
        let catalog = Catalog::acme_prod();
        let index =
            SwitcherIndex::from_catalog(&catalog, &History::seeded(), &crate::db::connections());
        let items = index.search("ord");
        assert_eq!(
            items.first().map(|item| item.label.as_str()),
            Some("orders")
        );
    }
}
