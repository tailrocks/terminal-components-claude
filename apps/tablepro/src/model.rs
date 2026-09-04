//! TablePro-owned query history, completion and quick-switcher models.
//!
//! These types intentionally contain no terminal state.  Keeping them here
//! makes the product behaviour testable without a runtime and keeps the
//! generic `tui-next` components free of SQL vocabulary.

use tui_next::fuzzy;

use crate::db::{Catalog, ColType, Table};
use crate::sql::{self, FUNCTIONS, KEYWORDS, TokKind, tokenize};

/// The source of a query-history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistorySource {
    /// Query editor execution.
    Editor,
    /// Explain-plan execution.
    Explain,
    /// Table browsing.
    Browsing,
    /// Row editing.
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

/// One retained query execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Monotonic history id.
    pub id: usize,
    /// SQL text.
    pub sql: String,
    /// Connection label.
    pub connection: String,
    /// Database label.
    pub database: String,
    /// Schema label.
    pub schema: String,
    /// Deterministic age in minutes.
    pub minutes_ago: u32,
    /// Execution duration, when known.
    pub duration_ms: Option<u32>,
    /// Returned/affected row count, when known.
    pub rows: Option<usize>,
    /// Error text for failed executions.
    pub error: Option<String>,
    /// Product surface that issued the query.
    pub source: HistorySource,
}

impl HistoryEntry {
    /// Whether the execution succeeded.
    pub fn ok(&self) -> bool {
        self.error.is_none()
    }

    /// First line for compact list rows.
    pub fn first_line(&self) -> String {
        let first = self.sql.lines().next().unwrap_or_default().trim();
        if self.sql.lines().count() > 1 {
            format!("{first} …")
        } else {
            first.to_owned()
        }
    }

    /// Deterministic relative-time label.
    pub fn when(&self) -> String {
        match self.minutes_ago {
            0 => "just now".to_owned(),
            minutes if minutes < 60 => format!("{minutes} min ago"),
            minutes if minutes < 24 * 60 => format!("{} h ago", minutes / 60),
            minutes => format!("{} d ago", minutes / (24 * 60)),
        }
    }

    /// Human-readable duration.
    pub fn duration(&self) -> String {
        match self.duration_ms {
            None => "–".to_owned(),
            Some(0) => "<1 ms".to_owned(),
            Some(ms) if ms < 1_000 => format!("{ms} ms"),
            Some(ms) if ms < 60_000 => format!("{:.2} s", ms as f64 / 1_000.0),
            Some(ms) => format!("{}m {}s", ms / 60_000, (ms % 60_000) / 1_000),
        }
    }
}

/// Bounded query history.
#[derive(Debug, Clone, Default)]
pub struct History {
    /// Newest entry first.
    pub entries: Vec<HistoryEntry>,
    next_id: usize,
}

impl History {
    /// Build the deterministic demo history used by the application and
    /// visual tests.
    pub fn seeded() -> Self {
        let mut history = Self::default();
        let entries = [
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
                Some("relation \"ordres\" does not exist"),
                HistorySource::Editor,
            ),
            (
                "ALTER TABLE orders ADD COLUMN is_gift boolean NOT NULL DEFAULT false",
                "Development",
                "acme_dev",
                26 * 60,
                Some(88),
                Some(0),
                None,
                HistorySource::Structure,
            ),
        ];
        for (sql_text, connection, database, minutes, duration, rows, error, source) in entries {
            history.push(HistoryEntry {
                id: 0,
                sql: sql_text.to_owned(),
                connection: connection.to_owned(),
                database: database.to_owned(),
                schema: "public".to_owned(),
                minutes_ago: minutes,
                duration_ms: duration,
                rows,
                error: error.map(str::to_owned),
                source,
            });
        }
        history
    }

    /// Add newest-first and retain at most 10,000 entries.
    pub fn push(&mut self, mut entry: HistoryEntry) {
        self.next_id = self.next_id.saturating_add(1);
        entry.id = self.next_id;
        self.entries.insert(0, entry);
        self.entries.truncate(10_000);
    }

    /// Search using case-insensitive ANDed terms and optional scope filters.
    pub fn search<'a>(
        &'a self,
        query: &str,
        connection: Option<&str>,
        failed_only: bool,
    ) -> Vec<&'a HistoryEntry> {
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|term| term.to_ascii_lowercase())
            .collect();
        self.entries
            .iter()
            .filter(|entry| connection.is_none_or(|name| entry.connection == name))
            .filter(|entry| !failed_only || !entry.ok())
            .filter(|entry| {
                let haystack = entry.sql.to_ascii_lowercase();
                terms.iter().all(|term| haystack.contains(term))
            })
            .collect()
    }
}

/// Completion category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// SQL keyword.
    Keyword,
    /// Table relation.
    Table,
    /// View relation.
    View,
    /// Column.
    Column,
    /// Function.
    Function,
    /// Schema.
    Schema,
    /// Query alias.
    Alias,
}

impl CompletionKind {
    /// Compact marker used by the query editor.
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Keyword => "K",
            Self::Table => "T",
            Self::View => "V",
            Self::Column => "C",
            Self::Function => "F",
            Self::Schema => "S",
            Self::Alias => "A",
        }
    }

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

/// One context-aware SQL completion item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// Category.
    pub kind: CompletionKind,
    /// Display label.
    pub label: String,
    /// Secondary detail.
    pub detail: String,
    /// Text inserted on accept.
    pub insert: String,
    /// Lower scores rank first.
    pub score: u32,
    /// Matched grapheme ordinals in `label`.
    pub matched: Vec<usize>,
}

/// SQL clause at the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clause {
    /// No clause yet.
    Start,
    /// SELECT projection.
    SelectList,
    /// Relation position.
    From,
    /// Predicate position.
    Where,
    /// Ordering position.
    OrderBy,
    /// Qualified member position.
    Member,
}

/// Determine the current word, clause and optional qualifier.
pub fn context(src: &str, cursor: usize) -> (String, Clause, Option<String>) {
    let cursor = cursor.min(src.len());
    let before = &src[..cursor];
    let word_start = before
        .char_indices()
        .rev()
        .find(|(_, character)| !(character.is_alphanumeric() || *character == '_'))
        .map_or(0, |(index, character)| index.saturating_add(character.len_utf8()));
    let word = before[word_start..].to_owned();
    let qualifier = if before[..word_start].ends_with('.') {
        let end = word_start.saturating_sub(1);
        let start = before[..end]
            .char_indices()
            .rev()
            .find(|(_, character)| !(character.is_alphanumeric() || *character == '_'))
            .map_or(0, |(index, character)| index.saturating_add(character.len_utf8()));
        Some(before[start..end].to_owned())
    } else {
        None
    };
    if qualifier.is_some() {
        return (word, Clause::Member, qualifier);
    }
    let statement_start = sql::statement_at(src, cursor).map_or(0, |(start, _)| start);
    let mut clause = Clause::Start;
    for token in tokenize(&src[statement_start..word_start]) {
        if token.kind != TokKind::Keyword {
            continue;
        }
        let keyword = src[statement_start + token.start..statement_start + token.end]
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

fn kind_for(table: &Table) -> CompletionKind {
    if table.kind == crate::db::ObjectKind::View {
        CompletionKind::View
    } else {
        CompletionKind::Table
    }
}

fn column_detail(column: &crate::db::Column) -> String {
    let mut detail = column.ty.sql().to_owned();
    if column.primary {
        detail.push_str(" · pk");
    }
    if column.references.is_some() {
        detail.push_str(" · fk");
    }
    if column.nullable {
        detail.push_str(" · null");
    }
    detail
}

fn tables_in_statement<'a>(catalog: &'a Catalog, statement: &str) -> Vec<(&'a Table, Option<String>)> {
    let tokens: Vec<(TokKind, String)> = tokenize(statement)
        .into_iter()
        .filter(|token| !matches!(token.kind, TokKind::Whitespace | TokKind::Comment))
        .map(|token| (token.kind, statement[token.start..token.end].to_owned()))
        .collect();
    let mut result = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let keyword = tokens[index].1.to_ascii_uppercase();
        if tokens[index].0 == TokKind::Keyword
            && matches!(keyword.as_str(), "FROM" | "JOIN" | "INTO" | "UPDATE")
            && let Some((kind, name)) = tokens.get(index.saturating_add(1))
            && *kind == TokKind::Ident
        {
            let (schema, table_name, after) =
                if tokens.get(index.saturating_add(2)).map(|token| token.1.as_str()) == Some(".") {
                    (
                        Some(name.as_str()),
                        tokens
                            .get(index.saturating_add(3))
                            .map_or_else(String::new, |token| token.1.clone()),
                        index.saturating_add(4),
                    )
                } else {
                    (None, name.clone(), index.saturating_add(2))
                };
            let alias = tokens.get(after).and_then(|(alias_kind, alias)| {
                if alias.eq_ignore_ascii_case("AS") {
                    tokens.get(after.saturating_add(1)).map(|token| token.1.clone())
                } else if *alias_kind == TokKind::Ident {
                    Some(alias.clone())
                } else {
                    None
                }
            });
            if let Some(table) = catalog.find(schema, &table_name) {
                result.push((table, alias));
            }
        }
        index = index.saturating_add(1);
    }
    result
}

/// Return ranked completions and the number of bytes in the replace span.
pub fn complete(catalog: &Catalog, source: &str, cursor: usize) -> (Vec<Completion>, usize) {
    let (word, clause, qualifier) = context(source, cursor);
    let statement = sql::statement_at(source, cursor)
        .map_or("", |(start, end)| &source[start..end]);
    let in_statement = tables_in_statement(catalog, statement);
    let mut items = Vec::new();
    let mut push = |kind: CompletionKind,
                    label: &str,
                    detail: String,
                    insert: Option<String>,
                    boost: i32| {
        if let Some((penalty, matched)) = fuzzy(label, &word) {
            let score = (kind.priority() as i32 + penalty as i32 + boost).max(0) as u32;
            items.push(Completion {
                kind,
                label: label.to_owned(),
                detail,
                insert: insert.unwrap_or_else(|| label.to_owned()),
                score,
                matched,
            });
        }
    };
    match clause {
        Clause::Member => {
            let qualifier = qualifier.unwrap_or_default();
            if let Some(table) = in_statement
                .iter()
                .find(|(table, alias)| {
                    alias
                        .as_deref()
                        .is_some_and(|alias| alias.eq_ignore_ascii_case(&qualifier))
                        || table.name.eq_ignore_ascii_case(&qualifier)
                })
                .map(|(table, _)| *table)
                .or_else(|| catalog.find(None, &qualifier))
            {
                for column in &table.columns {
                    push(
                        CompletionKind::Column,
                        &column.name,
                        column_detail(column),
                        None,
                        0,
                    );
                }
            } else if catalog
                .schemas
                .iter()
                .any(|schema| schema.eq_ignore_ascii_case(&qualifier))
            {
                for table in catalog
                    .tables
                    .iter()
                    .filter(|table| table.schema.eq_ignore_ascii_case(&qualifier))
                {
                    push(
                        kind_for(table),
                        &table.name,
                        format!("{} · {}", table.schema, sql::fmt_rows(table.row_count)),
                        None,
                        0,
                    );
                }
            }
        }
        Clause::From => {
            for table in &catalog.tables {
                if !matches!(table.kind, crate::db::ObjectKind::Table | crate::db::ObjectKind::View) {
                    continue;
                }
                let public = table.schema == "public";
                push(
                    kind_for(table),
                    &table.name,
                    format!("{} · {} rows", table.schema, sql::fmt_rows(table.row_count)),
                    (!public).then(|| table.qualified()),
                    if public { -50 } else { 0 },
                );
            }
            for schema in &catalog.schemas {
                push(CompletionKind::Schema, schema, "schema".to_owned(), None, 0);
            }
            for keyword in ["WHERE", "ORDER BY", "LIMIT", "JOIN", "LEFT JOIN", "GROUP BY"] {
                push(CompletionKind::Keyword, keyword, String::new(), None, 0);
            }
        }
        Clause::SelectList | Clause::Where | Clause::OrderBy => {
            let sources: Vec<&Table> = if in_statement.is_empty() {
                catalog.tables.iter().filter(|table| !table.columns.is_empty()).collect()
            } else {
                in_statement.iter().map(|(table, _)| *table).collect()
            };
            let ambiguous = sources.len() > 1;
            for table in sources {
                for column in &table.columns {
                    let label = if ambiguous && in_statement.is_empty() {
                        format!("{}.{}", table.name, column.name)
                    } else {
                        column.name.clone()
                    };
                    push(
                        CompletionKind::Column,
                        &label,
                        column_detail(column),
                        None,
                        if in_statement.is_empty() { 40 } else { 0 },
                    );
                }
            }
            for (table, alias) in &in_statement {
                if let Some(alias) = alias {
                    push(CompletionKind::Alias, alias, format!("alias of {}", table.name), None, 0);
                }
            }
            for function in FUNCTIONS {
                push(
                    CompletionKind::Function,
                    function,
                    "function".to_owned(),
                    Some(format!("{function}(")),
                    0,
                );
            }
            let keywords: &[&str] = match clause {
                Clause::SelectList => &["FROM", "DISTINCT", "AS", "CASE", "COUNT", "SUM", "AVG", "MAX", "MIN", "*"],
                Clause::Where => &["AND", "OR", "NOT", "IS NULL", "IS NOT NULL", "IN", "LIKE", "ILIKE", "BETWEEN", "ORDER BY", "LIMIT", "TRUE", "FALSE", "NULL"],
                _ => &["ASC", "DESC", "LIMIT", "OFFSET"],
            };
            for keyword in keywords {
                push(CompletionKind::Keyword, keyword, String::new(), None, 0);
            }
        }
        Clause::Start => {
            for keyword in ["SELECT", "SELECT * FROM", "INSERT INTO", "UPDATE", "DELETE FROM", "EXPLAIN", "EXPLAIN ANALYZE", "WITH", "CREATE TABLE", "ALTER TABLE", "DROP TABLE", "TRUNCATE", "BEGIN", "COMMIT", "ROLLBACK"] {
                push(CompletionKind::Keyword, keyword, String::new(), None, 0);
            }
            if !word.is_empty() {
                for keyword in KEYWORDS {
                    push(CompletionKind::Keyword, keyword, String::new(), None, 20);
                }
            }
        }
    }
    items.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| left.label.len().cmp(&right.label.len()))
            .then_with(|| left.label.cmp(&right.label))
    });
    items.dedup_by(|left, right| left.label == right.label && left.kind == right.kind);
    items.truncate(60);
    (items, word.len())
}

/// Whether the completion popup should open automatically.
pub fn auto_trigger(source: &str, cursor: usize) -> bool {
    let (word, clause, _) = context(source, cursor);
    match clause {
        Clause::Member | Clause::From => true,
        Clause::Where | Clause::OrderBy | Clause::SelectList => word.len() >= 2,
        Clause::Start => word.len() >= 3,
    }
}

/// A destination returned by the quick switcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchTarget {
    /// A table.
    Table { schema: String, name: String },
    /// A view.
    View { schema: String, name: String },
    /// A schema.
    Schema(String),
    /// A database.
    Database(String),
    /// Existing tab index.
    OpenTab(usize),
    /// History entry id.
    RecentQuery(usize),
}

/// One quick-switcher row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchItem {
    /// Destination.
    pub target: SwitchTarget,
    /// Primary label.
    pub label: String,
    /// Secondary path/detail.
    pub path: String,
    /// Group heading.
    pub group: &'static str,
    /// Whether this target is already open.
    pub open: bool,
    /// Query ranking score.
    pub score: u32,
    /// Fuzzy matched grapheme ordinals.
    pub matched: Vec<usize>,
}

/// Search index for Ctrl+O.
#[derive(Debug, Clone, Default)]
pub struct SwitcherIndex {
    /// All indexed targets.
    pub items: Vec<SwitchItem>,
}

impl SwitcherIndex {
    /// Build an index from catalog, open tabs and history.
    pub fn build(
        catalog: &Catalog,
        connection: &str,
        open_tabs: &[(usize, String)],
        history: &History,
    ) -> Self {
        let mut items = Vec::new();
        for table in &catalog.tables {
            let (target, group) = match table.kind {
                crate::db::ObjectKind::Table => (
                    SwitchTarget::Table {
                        schema: table.schema.clone(),
                        name: table.name.clone(),
                    },
                    "Tables",
                ),
                crate::db::ObjectKind::View => (
                    SwitchTarget::View {
                        schema: table.schema.clone(),
                        name: table.name.clone(),
                    },
                    "Views",
                ),
                _ => continue,
            };
            let open = open_tabs.iter().any(|(_, label)| label == &table.name || label == &table.qualified());
            items.push(SwitchItem {
                target,
                label: table.name.clone(),
                path: format!("{} · {connection}", table.schema),
                group,
                open,
                score: 0,
                matched: Vec::new(),
            });
        }
        for schema in &catalog.schemas {
            items.push(SwitchItem {
                target: SwitchTarget::Schema(schema.clone()),
                label: schema.clone(),
                path: format!("{} · {connection}", catalog.database),
                group: "Schemas",
                open: false,
                score: 0,
                matched: Vec::new(),
            });
        }
        items.push(SwitchItem {
            target: SwitchTarget::Database(catalog.database.clone()),
            label: catalog.database.clone(),
            path: connection.to_owned(),
            group: "Databases",
            open: true,
            score: 0,
            matched: Vec::new(),
        });
        for (index, label) in open_tabs {
            items.push(SwitchItem {
                target: SwitchTarget::OpenTab(*index),
                label: label.clone(),
                path: "open tab".to_owned(),
                group: "Open tabs",
                open: true,
                score: 0,
                matched: Vec::new(),
            });
        }
        for entry in history.entries.iter().take(50) {
            items.push(SwitchItem {
                target: SwitchTarget::RecentQuery(entry.id),
                label: entry.first_line(),
                path: format!("{} · {}", entry.connection, entry.when()),
                group: "Recent queries",
                open: false,
                score: 0,
                matched: Vec::new(),
            });
        }
        Self { items }
    }

    /// Filter and rank indexed targets.
    pub fn query(&self, query: &str) -> Vec<SwitchItem> {
        let query = query.trim();
        let mut result: Vec<SwitchItem> = self
            .items
            .iter()
            .filter_map(|item| {
                let mut item = item.clone();
                if query.is_empty() {
                    item.score = group_rank(item.group).saturating_mul(10);
                    return Some(item);
                }
                if let Some((penalty, matched)) = fuzzy(&item.label, query) {
                    item.score = penalty
                        .saturating_add(group_rank(item.group).saturating_mul(5))
                        .saturating_add(u32::from(!item.open).saturating_mul(3));
                    item.matched = matched;
                    Some(item)
                } else if item.path.to_ascii_lowercase().contains(&query.to_ascii_lowercase()) {
                    item.score = 120u32.saturating_add(group_rank(item.group).saturating_mul(5));
                    Some(item)
                } else {
                    None
                }
            })
            .collect();
        result.sort_by(|left, right| left.score.cmp(&right.score).then_with(|| left.label.cmp(&right.label)));
        result.truncate(200);
        result
    }
}

fn group_rank(group: &str) -> u32 {
    match group {
        "Tables" => 0,
        "Views" => 1,
        "Open tabs" => 2,
        "Schemas" => 3,
        "Databases" => 4,
        _ => 5,
    }
}

/// Convert catalog columns to a compact public-grid column description.
pub fn column_labels(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_search_is_multi_term_and() {
        let history = History::seeded();
        assert_eq!(history.search("orders pending", None, false).len(), 1);
        assert!(history
            .search("", Some("Development"), false)
            .iter()
            .all(|entry| entry.connection == "Development"));
        assert!(history
            .search("", None, true)
            .iter()
            .all(|entry| !entry.ok()));
    }

    #[test]
    fn completion_is_context_aware() {
        let catalog = Catalog::acme_prod();
        let source = "SELECT o. FROM orders o WHERE ";
        let (items, _) = complete(&catalog, source, 9);
        assert_eq!(items.first().map(|item| item.kind), Some(CompletionKind::Column));
        assert!(items.iter().any(|item| item.label == "total_amount"));
        let source = "SELECT * FROM ord";
        let (items, replace) = complete(&catalog, source, source.len());
        assert_eq!(replace, 3);
        assert_eq!(items.first().map(|item| item.label.as_str()), Some("orders"));
        assert_eq!(items.first().map(|item| item.matched.as_slice()), Some([0, 1, 2].as_slice()));
        assert!(auto_trigger("SELECT * FROM ", 14));
        assert!(!auto_trigger("SELECT * FROM orders WHERE s", 28));
    }

    #[test]
    fn switcher_ranks_tables_first_and_prefix_first() {
        let catalog = Catalog::acme_prod();
        let history = History::seeded();
        let index = SwitcherIndex::build(&catalog, "Production", &[(0, "orders".to_owned())], &history);
        let result = index.query("ord");
        assert_eq!(result.first().map(|item| item.label.as_str()), Some("orders"));
        assert!(result.first().is_some_and(|item| item.open));
        assert!(result.iter().any(|item| item.label == "order_items"));
        assert!(result.iter().any(|item| item.group == "Recent queries"));
        assert!(index.query("").len() > 15);
    }
}
