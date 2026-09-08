//! Query-history, completion and quick-switcher models.

use junie_tui::{FuzzyBoundary, fuzzy_with_boundary};

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
    #[expect(
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
    #[expect(
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

/// Completion item category, preserving SQL context and display semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// SQL keyword.
    Keyword,
    /// Table name.
    Table,
    /// View name.
    View,
    /// Column name.
    Column,
    /// SQL function.
    Function,
    /// Schema qualifier.
    Schema,
    /// Alias declared in the current statement.
    Alias,
}
impl CompletionKind {
    const fn priority(self) -> u32 {
        match self {
            Self::Column => 100,
            Self::Alias => 150,
            Self::Table => 200,
            Self::View => 210,
            Self::Function => 300,
            Self::Keyword => 400,
            Self::Schema => 500,
        }
    }
}
/// One SQL completion candidate.
#[derive(Clone, PartialEq, Eq)]
pub struct Completion {
    /// Text inserted into the editor, including any necessary qualifier.
    pub text: String,
    /// Display label.
    pub label: String,
    /// Type, relation or alias metadata.
    pub detail: String,
    /// Candidate category.
    pub kind: CompletionKind,
    /// Rank; lower values sort first.
    pub score: u32,
    /// Matched grapheme ordinals in the original label.
    pub matched: Vec<usize>,
}
impl core::fmt::Debug for Completion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Completion")
            .field("kind", &self.kind)
            .field("text_bytes", &self.text.len())
            .field("label_bytes", &self.label.len())
            .field("detail_bytes", &self.detail.len())
            .field("score", &self.score)
            .field("matched_count", &self.matched.len())
            .finish()
    }
}
/// Suggestions and the exact UTF-8 range they replace in the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionBatch {
    /// Ranked candidates.
    pub items: Vec<Completion>,
    /// Current word, ending at the normalized cursor boundary.
    pub replace: core::ops::Range<usize>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Clause {
    Start,
    SelectList,
    From,
    Where,
    OrderBy,
    Member,
}

fn normalized_cursor(source: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(source.len());
    while !source.is_char_boundary(cursor) {
        cursor = cursor.saturating_sub(1);
    }
    cursor
}
fn word_start(source: &str) -> usize {
    source
        .char_indices()
        .rev()
        .find(|(_, ch)| !(ch.is_alphanumeric() || *ch == '_'))
        .map_or(0, |(index, ch)| index.saturating_add(ch.len_utf8()))
}
fn context(source: &str, cursor: usize) -> (&str, Clause, Option<&str>) {
    let before = source.get(..cursor).unwrap_or_default();
    let start = word_start(before);
    let word = before.get(start..).unwrap_or_default();
    let preceding = before.get(..start).unwrap_or_default();
    if let Some(qualifier) = preceding.strip_suffix('.') {
        return (word, Clause::Member, qualifier.get(word_start(qualifier)..));
    }
    let statement_start = crate::sql::statement_at(source, cursor).map_or(0, |(start, _)| start);
    let preceding = source.get(statement_start..start).unwrap_or_default();
    let mut clause = Clause::Start;
    for token in tokenize(preceding)
        .into_iter()
        .filter(|token| token.kind == TokKind::Keyword)
    {
        let keyword = preceding
            .get(token.start..token.end)
            .unwrap_or_default()
            .to_ascii_uppercase();
        clause = match keyword.as_str() {
            "SELECT" => Clause::SelectList,
            "FROM" | "JOIN" | "INTO" | "UPDATE" | "TABLE" => Clause::From,
            "WHERE" | "AND" | "OR" | "ON" | "HAVING" | "SET" => Clause::Where,
            "BY" => Clause::OrderBy,
            _ => clause,
        };
    }
    (word, clause, None)
}
fn tables_in_statement<'a>(
    catalog: &'a Catalog,
    statement: &str,
) -> Vec<(&'a Table, Option<String>)> {
    let tokens = tokenize(statement)
        .into_iter()
        .filter(|token| !matches!(token.kind, TokKind::Whitespace | TokKind::Comment))
        .filter_map(|token| {
            statement
                .get(token.start..token.end)
                .map(|text| (token.kind, text))
        })
        .collect::<Vec<_>>();
    let mut out = Vec::new();
    for (index, (kind, word)) in tokens.iter().enumerate() {
        if *kind != TokKind::Keyword
            || !matches!(
                word.to_ascii_uppercase().as_str(),
                "FROM" | "JOIN" | "INTO" | "UPDATE"
            )
        {
            continue;
        }
        let Some((TokKind::Ident, name)) = tokens.get(index.saturating_add(1)) else {
            continue;
        };
        let qualified = tokens
            .get(index.saturating_add(2))
            .is_some_and(|(_, text)| *text == ".");
        let (schema, name, after) = if qualified {
            let Some((_, table)) = tokens.get(index.saturating_add(3)) else {
                continue;
            };
            (
                Some(name.trim_matches('"')),
                table.trim_matches('"'),
                index.saturating_add(4),
            )
        } else {
            (None, name.trim_matches('"'), index.saturating_add(2))
        };
        let alias = tokens.get(after).and_then(|(kind, text)| {
            if text.eq_ignore_ascii_case("AS") {
                tokens
                    .get(after.saturating_add(1))
                    .filter(|(kind, _)| *kind == TokKind::Ident)
                    .map(|(_, text)| text.trim_matches('"').to_owned())
            } else if *kind == TokKind::Ident {
                Some(text.trim_matches('"').to_owned())
            } else {
                None
            }
        });
        if let Some(table) = catalog.find(schema, name) {
            out.push((table, alias));
        }
    }
    out
}
struct CompletionPool<'a> {
    word: &'a str,
    items: Vec<Completion>,
}
impl CompletionPool<'_> {
    fn push(
        &mut self,
        kind: CompletionKind,
        label: &str,
        detail: String,
        insert: Option<String>,
        boost: i32,
    ) {
        if let Some((penalty, matched)) =
            fuzzy_with_boundary(label, self.word, FuzzyBoundary::Identifier)
        {
            let base = kind.priority().saturating_add(penalty);
            let score = if boost < 0 {
                base.saturating_sub(boost.unsigned_abs())
            } else {
                base.saturating_add(boost.unsigned_abs())
            };
            self.items.push(Completion {
                kind,
                label: label.to_owned(),
                detail,
                text: insert.unwrap_or_else(|| label.to_owned()),
                score,
                matched,
            });
        }
    }
}
/// Complete a token while retaining the byte range needed to apply it safely.
pub fn completion_batch(src: &str, cursor: usize, cat: &Catalog) -> CompletionBatch {
    let cursor = normalized_cursor(src, cursor);
    let (word, clause, qualifier) = context(src, cursor);
    let stmt = crate::sql::statement_at(src, cursor)
        .and_then(|(a, b)| src.get(a..b))
        .unwrap_or_default();
    let in_stmt = tables_in_statement(cat, stmt);
    let mut pool = CompletionPool {
        word,
        items: Vec::new(),
    };
    match clause {
        Clause::Member => {
            member_candidates(cat, &in_stmt, qualifier.unwrap_or_default(), &mut pool);
        }
        Clause::From => relation_candidates(cat, &mut pool),
        Clause::SelectList | Clause::Where | Clause::OrderBy => {
            column_candidates(cat, &in_stmt, clause, &mut pool);
        }
        Clause::Start => statement_candidates(&mut pool),
    }
    pool.items.sort_by(|a, b| {
        a.score
            .cmp(&b.score)
            .then_with(|| a.label.len().cmp(&b.label.len()))
            .then_with(|| a.label.cmp(&b.label))
    });
    pool.items
        .dedup_by(|a, b| a.label == b.label && a.kind == b.kind);
    pool.items.truncate(60);
    CompletionBatch {
        items: pool.items,
        replace: cursor.saturating_sub(word.len())..cursor,
    }
}
fn member_candidates(
    cat: &Catalog,
    in_stmt: &[(&Table, Option<String>)],
    q: &str,
    pool: &mut CompletionPool<'_>,
) {
    // alias or table name → its columns; schema → its tables
    let table = in_stmt
        .iter()
        .find(|(t, a)| {
            a.as_deref().is_some_and(|a| a.eq_ignore_ascii_case(q))
                || t.name.eq_ignore_ascii_case(q)
        })
        .map(|(t, _)| *t)
        .or_else(|| cat.find(None, q));
    if let Some(t) = table {
        for c in &t.columns {
            pool.push(CompletionKind::Column, &c.name, col_detail(c), None, 0);
        }
    } else if cat.schemas.iter().any(|s| s.eq_ignore_ascii_case(q)) {
        for t in cat
            .tables
            .iter()
            .filter(|t| t.schema.eq_ignore_ascii_case(q))
        {
            pool.push(
                kind_of(t),
                &t.name,
                format!("{} · {}", t.schema, crate::sql::fmt_rows(t.row_count)),
                None,
                0,
            );
        }
    }
}
fn relation_candidates(cat: &Catalog, pool: &mut CompletionPool<'_>) {
    for t in &cat.tables {
        if matches!(
            t.kind,
            crate::db::ObjectKind::Table | crate::db::ObjectKind::View
        ) {
            let boost = if t.schema == "public" { -50 } else { 0 };
            let insert = if t.schema == "public" {
                None
            } else {
                Some(t.qualified())
            };
            pool.push(
                kind_of(t),
                &t.name,
                format!("{} · {} rows", t.schema, crate::sql::fmt_rows(t.row_count)),
                insert,
                boost,
            );
        }
    }
    for s in &cat.schemas {
        pool.push(CompletionKind::Schema, s, "schema".into(), None, 0);
    }
    for k in [
        "WHERE",
        "ORDER BY",
        "LIMIT",
        "JOIN",
        "LEFT JOIN",
        "GROUP BY",
    ] {
        pool.push(CompletionKind::Keyword, k, String::new(), None, 0);
    }
}
fn column_candidates(
    cat: &Catalog,
    in_stmt: &[(&Table, Option<String>)],
    clause: Clause,
    pool: &mut CompletionPool<'_>,
) {
    let sources: Vec<&Table> = if in_stmt.is_empty() {
        cat.tables
            .iter()
            .filter(|t| !t.columns.is_empty())
            .collect()
    } else {
        in_stmt.iter().map(|(t, _)| *t).collect()
    };
    let ambiguous = sources.len() > 1;
    for t in &sources {
        for c in &t.columns {
            let label = if ambiguous && in_stmt.is_empty() {
                format!("{}.{}", t.name, c.name)
            } else {
                c.name.clone()
            };
            let detail = if ambiguous {
                format!("{} · {}", t.name, col_detail(c))
            } else {
                col_detail(c)
            };
            pool.push(
                CompletionKind::Column,
                &label,
                detail,
                None,
                if in_stmt.is_empty() { 40 } else { 0 },
            );
        }
    }
    for (t, a) in in_stmt {
        if let Some(a) = a {
            pool.push(
                CompletionKind::Alias,
                a,
                format!("alias of {}", t.name),
                None,
                0,
            );
        }
    }
    for f in FUNCTIONS {
        pool.push(
            CompletionKind::Function,
            f,
            "function".into(),
            Some(format!("{f}(")),
            0,
        );
    }
    let kws: &[&str] = match clause {
        Clause::SelectList => &[
            "FROM", "DISTINCT", "AS", "CASE", "COUNT", "SUM", "AVG", "MAX", "MIN", "*",
        ],
        Clause::Where => &[
            "AND",
            "OR",
            "NOT",
            "IS NULL",
            "IS NOT NULL",
            "IN",
            "LIKE",
            "ILIKE",
            "BETWEEN",
            "ORDER BY",
            "LIMIT",
            "TRUE",
            "FALSE",
            "NULL",
        ],
        _ => &["ASC", "DESC", "LIMIT", "OFFSET"],
    };
    for k in kws {
        pool.push(CompletionKind::Keyword, k, String::new(), None, 0);
    }
}
fn statement_candidates(pool: &mut CompletionPool<'_>) {
    for k in [
        "SELECT",
        "SELECT * FROM",
        "INSERT INTO",
        "UPDATE",
        "DELETE FROM",
        "EXPLAIN",
        "EXPLAIN ANALYZE",
        "WITH",
        "CREATE TABLE",
        "ALTER TABLE",
        "DROP TABLE",
        "TRUNCATE",
        "BEGIN",
        "COMMIT",
        "ROLLBACK",
    ] {
        pool.push(CompletionKind::Keyword, k, String::new(), None, 0);
    }
    for k in KEYWORDS.iter().filter(|_| !pool.word.is_empty()) {
        pool.push(CompletionKind::Keyword, k, String::new(), None, 20);
    }
}

fn kind_of(t: &Table) -> CompletionKind {
    if t.kind == crate::db::ObjectKind::View {
        CompletionKind::View
    } else {
        CompletionKind::Table
    }
}

fn col_detail(c: &crate::db::Column) -> String {
    let mut d = c.ty.sql().to_owned();
    if c.primary {
        d.push_str(" · pk");
    }
    if c.references.is_some() {
        d.push_str(" · fk");
    }
    if c.nullable {
        d.push_str(" · null");
    }
    d
}

/// Whether this SQL context has enough input to open completion automatically.
pub fn auto_trigger(src: &str, cursor: usize) -> bool {
    let (word, clause, _) = context(src, normalized_cursor(src, cursor));
    match clause {
        Clause::Member | Clause::From => true,
        Clause::Where | Clause::OrderBy | Clause::SelectList => word.len() >= 2,
        Clause::Start => word.len() >= 3,
    }
}

/// Return ranked candidates; use [`completion_batch`] to apply a replacement.
pub fn complete(source: &str, cursor: usize, catalog: &Catalog) -> Vec<Completion> {
    completion_batch(source, cursor, catalog).items
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
                fuzzy_with_boundary(&item.label, query, FuzzyBoundary::Identifier).map(
                    |(score, _)| {
                        let mut copy = item.clone();
                        copy.score = score;
                        copy
                    },
                )
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
#[expect(
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
#[expect(
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
        let development = history.search("", Some("Development"), false);
        assert!(!development.is_empty());
        assert!(
            development
                .iter()
                .all(|entry| entry.connection == "Development")
        );
        let failed = history.search("", None, true);
        assert!(!failed.is_empty() && failed.iter().all(|entry| !entry.ok()));
    }
    #[test]
    fn completion_is_context_aware() {
        let catalog = Catalog::acme_prod();
        let items = complete("SELECT o. FROM orders o WHERE ", 9, &catalog);
        assert_eq!(
            items.first().map(|item| item.kind),
            Some(CompletionKind::Column)
        );
        assert!(items.iter().any(|item| item.label == "total_amount"));
        let source = "SELECT * FROM ord";
        let batch = completion_batch(source, source.len(), &catalog);
        assert_eq!(batch.replace, 14..17);
        let items = batch.items;
        assert_eq!(
            items.first().map(|item| item.matched.as_slice()),
            Some([0, 1, 2].as_slice())
        );
        assert_eq!(
            items.first().map(|item| item.label.as_str()),
            Some("orders")
        );
        assert_eq!(
            items.first().map(|item| item.kind),
            Some(CompletionKind::Table)
        );
        let source = "SELECT * FROM orders WHERE st";
        assert_eq!(
            complete(source, source.len(), &catalog)
                .first()
                .map(|item| item.label.as_str()),
            Some("status")
        );
        let source = "SELECT * FROM orders WHERE status = 'x' ORDER BY cre";
        assert_eq!(
            complete(source, source.len(), &catalog)
                .first()
                .map(|item| item.label.as_str()),
            Some("created_at")
        );
        let source = "SELECT * FROM analytics.";
        assert!(
            complete(source, source.len(), &catalog)
                .iter()
                .any(|item| item.label == "events")
        );
        assert!(auto_trigger("SELECT * FROM ", 14));
        assert!(!auto_trigger("SELECT * FROM orders WHERE s", 28));
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
