//! Quick-switcher model and re-exports of history and completion.

use tablepro_domain::{Catalog, ColType, Connection, Table};
use tablepro_sql::{FuzzyBoundary, fuzzy_with_boundary, tokenize};

pub use tablepro_domain::{History, HistoryEntry, HistorySource};
pub use tablepro_sql::{
    Completion, CompletionBatch, CompletionKind, auto_trigger, complete, completion_batch,
};

use crate::tabs::{Tab, TabKey, TabRecord};

/// An actionable switcher destination, independent of its display position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchTarget {
    /// Catalog table, identified by schema and name.
    Table { schema: String, name: String },
    /// Catalog view, identified by schema and name.
    View { schema: String, name: String },
    /// Schema name.
    Schema(String),
    /// Database name.
    Database(String),
    /// Immutable owning tab identity.
    OpenTab(TabKey),
    /// Stable history entry id.
    Query(usize),
    /// Additive connection destination.
    Connection(String),
}

impl SwitchTarget {
    /// Source group label, derived from actual destination.
    pub const fn group(&self) -> &'static str {
        match self {
            Self::Table { .. } => "Tables",
            Self::View { .. } => "Views",
            Self::Schema(_) => "Schemas",
            Self::Database(_) => "Databases",
            Self::OpenTab(_) => "Open tabs",
            Self::Query(_) => "Recent queries",
            Self::Connection(_) => "Connections",
        }
    }

    const fn rank(&self) -> u32 {
        match self {
            Self::Table { .. } => 0,
            Self::View { .. } => 1,
            Self::OpenTab(_) => 2,
            Self::Schema(_) => 3,
            Self::Database(_) => 4,
            Self::Query(_) => 5,
            Self::Connection(_) => 6,
        }
    }
}

/// One quick-switcher result.
#[derive(Clone, PartialEq, Eq)]
pub struct SwitchItem {
    pub key: String,
    pub label: String,
    pub detail: String,
    pub target: SwitchTarget,
    pub open: bool,
    pub score: u32,
    pub matched: Vec<usize>,
}

impl core::fmt::Debug for SwitchItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SwitchItem")
            .field("key_bytes", &self.key.len())
            .field("group", &self.target.group())
            .field("label_bytes", &self.label.len())
            .field("detail_bytes", &self.detail.len())
            .field("open", &self.open)
            .field("score", &self.score)
            .field("matched_count", &self.matched.len())
            .finish()
    }
}

/// Owned snapshot of available quick-switcher destinations.
#[derive(Debug, Clone, Default)]
pub struct SwitcherIndex {
    pub items: Vec<SwitchItem>,
}

impl SwitcherIndex {
    /// Build without open-tab metadata for standalone catalog consumers.
    pub fn from_catalog(catalog: &Catalog, history: &History, connections: &[Connection]) -> Self {
        Self::build(catalog, history, connections, &[])
    }

    pub fn from_workbench(
        catalog: &Catalog,
        history: &History,
        connection: &Connection,
        tabs: &[TabRecord],
    ) -> Self {
        Self::build(catalog, history, std::slice::from_ref(connection), tabs)
    }

    fn push(
        &mut self,
        key: String,
        label: String,
        detail: String,
        target: SwitchTarget,
        open: bool,
    ) {
        self.items.push(SwitchItem {
            key,
            label,
            detail,
            target,
            open,
            score: 0,
            matched: Vec::new(),
        });
    }

    fn build(
        catalog: &Catalog,
        history: &History,
        connections: &[Connection],
        tabs: &[TabRecord],
    ) -> Self {
        let mut index = Self::default();
        let connection = connections
            .first()
            .map_or("", |connection| connection.name.as_str());
        for table in &catalog.tables {
            let target = match table.kind {
                tablepro_domain::ObjectKind::Table => SwitchTarget::Table {
                    schema: table.schema.clone(),
                    name: table.name.clone(),
                },
                tablepro_domain::ObjectKind::View => SwitchTarget::View {
                    schema: table.schema.clone(),
                    name: table.name.clone(),
                },
                tablepro_domain::ObjectKind::Function | tablepro_domain::ObjectKind::Sequence => {
                    continue;
                }
            };
            let open = tabs.iter().any(|record| {
                matches!(record.payload(), Tab::Table(tab) if tab.table.schema == table.schema && tab.table.name == table.name)
            });
            index.push(
                table.qualified(),
                table.name.clone(),
                format!("{} · {connection}", table.schema),
                target,
                open,
            );
        }
        for schema in &catalog.schemas {
            index.push(
                format!("schema-{schema}"),
                schema.clone(),
                format!("{} · {connection}", catalog.database),
                SwitchTarget::Schema(schema.clone()),
                false,
            );
        }
        index.push(
            format!("database-{}", catalog.database),
            catalog.database.clone(),
            connection.to_owned(),
            SwitchTarget::Database(catalog.database.clone()),
            true,
        );
        for record in tabs {
            index.push(
                format!("tab-{}", record.key().get()),
                record.payload().label(),
                "open tab".to_owned(),
                SwitchTarget::OpenTab(record.key()),
                true,
            );
        }
        for entry in history.entries.iter().take(50) {
            index.push(
                format!("history-{}", entry.id),
                entry.first_line(),
                format!("{} · {}", entry.connection, entry.when()),
                SwitchTarget::Query(entry.id),
                false,
            );
        }
        for connection in connections {
            index.push(
                format!("connection-{}", connection.name),
                connection.name.clone(),
                connection.environment.label().to_owned(),
                SwitchTarget::Connection(connection.name.clone()),
                false,
            );
        }
        index
    }

    /// Rank name matches before path matches, preserving source group priorities.
    pub fn search(&self, query: &str) -> Vec<SwitchItem> {
        let query = query.trim();
        let path_query = query.to_lowercase();
        let mut out = self
            .items
            .iter()
            .filter_map(|item| {
                let mut item = item.clone();
                if query.is_empty() {
                    item.score = item.target.rank().saturating_mul(10);
                } else if let Some((penalty, matched)) =
                    fuzzy_with_boundary(&item.label, query, FuzzyBoundary::Identifier)
                {
                    item.score = penalty
                        .saturating_add(item.target.rank().saturating_mul(5))
                        .saturating_add(if item.open { 0 } else { 3 });
                    item.matched = matched;
                } else if item.detail.to_lowercase().contains(&path_query) {
                    item.score = 120u32.saturating_add(item.target.rank().saturating_mul(5));
                    item.matched.clear();
                } else {
                    return None;
                }
                Some(item)
            })
            .collect::<Vec<_>>();
        out.sort_by(|a, b| a.score.cmp(&b.score).then_with(|| a.label.cmp(&b.label)));
        out.truncate(200);
        out
    }
}

pub fn table_columns(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

pub fn statement_tokens(source: &str) -> Vec<String> {
    tokenize(source)
        .into_iter()
        .filter_map(|token| source.get(token.start..token.end).map(str::to_owned))
        .collect()
}
