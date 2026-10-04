//! SQL AST types.

/// Comparison predicate used by a parsed `WHERE` clause.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmp {
    /// Equality.
    Eq,
    /// Inequality.
    Ne,
    /// Greater-than comparison.
    Gt,
    /// Greater-than-or-equal comparison.
    Ge,
    /// Less-than comparison.
    Lt,
    /// Less-than-or-equal comparison.
    Le,
    /// SQL `LIKE` comparison.
    Like,
    /// SQL `IS NULL` comparison.
    IsNull,
    /// SQL `IS NOT NULL` comparison.
    IsNotNull,
    /// SQL `IN` comparison.
    In(Vec<String>),
}

/// One parsed column predicate.
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    /// Column name.
    pub column: String,
    /// Comparison operator and operands.
    pub cmp: Cmp,
    /// Primary comparison value.
    pub value: String,
}

/// Parsed `SELECT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    /// Projection names, or `*`.
    pub columns: Vec<String>,
    /// Optional schema qualifier.
    pub schema: Option<String>,
    /// Relation name.
    pub table: String,
    /// Predicates applied to the relation.
    pub predicates: Vec<Predicate>,
    /// Optional `(column, descending)` ordering.
    pub order: Option<(String, bool)>,
    /// Optional row limit.
    pub limit: Option<usize>,
    /// Whether this is a `count(*)` projection.
    pub count_only: bool,
}

/// Parsed statement in supported deterministic SQL subset.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Row-selecting query.
    Select(Select),
    /// Update statement.
    Update {
        table: String,
        has_where: bool,
    },
    /// Delete statement.
    Delete {
        table: String,
        has_where: bool,
    },
    /// Insert statement.
    Insert {
        table: String,
    },
    /// Drop statement.
    Drop {
        kind: String,
        name: String,
    },
    /// Truncate statement.
    Truncate {
        table: String,
    },
    /// Alter statement.
    Alter {
        table: String,
        destructive: bool,
    },
    /// Create statement.
    Create {
        kind: String,
        name: String,
    },
    /// Explain wrapper.
    Explain {
        analyze: bool,
        inner: Box<Statement>,
    },
    /// Statement retained as unsupported text.
    Other(String),
}

impl Statement {
    /// Return the normalized verb for this statement.
    pub fn verb(&self) -> &'static str {
        match self {
            Statement::Select(_) => "SELECT",
            Statement::Update { .. } => "UPDATE",
            Statement::Delete { .. } => "DELETE",
            Statement::Insert { .. } => "INSERT",
            Statement::Drop { .. } => "DROP",
            Statement::Truncate { .. } => "TRUNCATE",
            Statement::Alter { .. } => "ALTER",
            Statement::Create { .. } => "CREATE",
            Statement::Explain { .. } => "EXPLAIN",
            Statement::Other(_) => "STATEMENT",
        }
    }

    /// Return the primary relation or object target, when present.
    pub fn target(&self) -> Option<&str> {
        match self {
            Statement::Select(s) => Some(&s.table),
            Statement::Update { table, .. }
            | Statement::Delete { table, .. }
            | Statement::Insert { table }
            | Statement::Truncate { table }
            | Statement::Alter { table, .. } => Some(table),
            Statement::Drop { name, .. } | Statement::Create { name, .. } => Some(name),
            Statement::Explain { inner, .. } => inner.target(),
            Statement::Other(_) => None,
        }
    }
}
