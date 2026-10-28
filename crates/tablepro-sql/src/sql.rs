#![allow(
    missing_docs,
    reason = "items promoted from pub(crate) during the crate split keep their original undocumented fixtures"
)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    reason = "This migrated deterministic SQL adapter validates catalog-derived indexes and keeps parser/executor behavior aligned with the product subset."
)]

use tablepro_demo::db::{Catalog, ColType, Table, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    Keyword,
    Ident,
    Number,
    String,
    Operator,
    Punct,
    Comment,
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokKind,
    pub start: usize,
    pub end: usize,
}

pub const KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "NOT",
    "NULL",
    "IS",
    "IN",
    "LIKE",
    "ILIKE",
    "ORDER",
    "BY",
    "ASC",
    "DESC",
    "LIMIT",
    "OFFSET",
    "GROUP",
    "HAVING",
    "JOIN",
    "LEFT",
    "RIGHT",
    "INNER",
    "OUTER",
    "FULL",
    "CROSS",
    "ON",
    "AS",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "DROP",
    "TABLE",
    "TRUNCATE",
    "ALTER",
    "CREATE",
    "INDEX",
    "VIEW",
    "DATABASE",
    "SCHEMA",
    "COLUMN",
    "ADD",
    "RENAME",
    "TO",
    "CASCADE",
    "RESTRICT",
    "EXPLAIN",
    "ANALYZE",
    "BEGIN",
    "COMMIT",
    "ROLLBACK",
    "DISTINCT",
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "BETWEEN",
    "EXISTS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "UNION",
    "ALL",
    "WITH",
    "RETURNING",
    "TRUE",
    "FALSE",
    "INTERVAL",
    "NOW",
    "CAST",
    "COALESCE",
    "PRIMARY",
    "KEY",
    "REFERENCES",
    "DEFAULT",
    "UNIQUE",
    "CHECK",
    "CONSTRAINT",
    "IF",
    "USING",
    "GRANT",
    "REVOKE",
    "VACUUM",
    "REINDEX",
];

pub const FUNCTIONS: &[&str] = &[
    "count",
    "sum",
    "avg",
    "min",
    "max",
    "now",
    "coalesce",
    "lower",
    "upper",
    "length",
    "date_trunc",
    "extract",
    "to_char",
    "jsonb_extract_path_text",
    "array_agg",
    "string_agg",
    "row_number",
    "gen_random_uuid",
];

pub fn is_keyword(word: &str) -> bool {
    let up = word.to_ascii_uppercase();
    KEYWORDS.contains(&up.as_str())
}

pub fn tokenize(src: &str) -> Vec<Token> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let Some(c) = src.get(i..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        let start = i;
        let kind = if c.is_whitespace() {
            while let Some(ch) = src.get(i..).and_then(|rest| rest.chars().next()) {
                if !ch.is_whitespace() {
                    break;
                }
                i = i.saturating_add(ch.len_utf8());
            }
            TokKind::Whitespace
        } else if c == '-' && b.get(i + 1) == Some(&b'-') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            TokKind::Comment
        } else if c == '/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                i += 1;
            }
            i = (i + 2).min(b.len());
            TokKind::Comment
        } else if c == '\'' {
            i += 1;
            while i < b.len() && b[i] != b'\'' {
                i += 1;
            }
            i = (i + 1).min(b.len());
            TokKind::String
        } else if c == '"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                i += 1;
            }
            i = (i + 1).min(b.len());
            TokKind::Ident
        } else if c.is_ascii_digit() {
            while i < b.len() && ((b[i] as char).is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            TokKind::Number
        } else if c.is_alphabetic() || c == '_' {
            while let Some(ch) = src.get(i..).and_then(|rest| rest.chars().next()) {
                if !ch.is_alphanumeric() && ch != '_' {
                    break;
                }
                i = i.saturating_add(ch.len_utf8());
            }
            if is_keyword(&src[start..i]) {
                TokKind::Keyword
            } else {
                TokKind::Ident
            }
        } else if "=<>!+-*/%|".contains(c) {
            while i < b.len() && "=<>!+-*/%|".contains(b[i] as char) {
                i += 1;
            }
            TokKind::Operator
        } else {
            i += c.len_utf8().max(1);
            TokKind::Punct
        };
        out.push(Token {
            kind,
            start,
            end: i,
        });
    }
    out
}

pub fn split_statements(src: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    for t in tokenize(src) {
        if t.kind == TokKind::Punct && &src[t.start..t.end] == ";" {
            let s = trim_range(src, start, t.start);
            if s.0 < s.1 {
                out.push(s);
            }
            start = t.end;
        }
    }
    let s = trim_range(src, start, src.len());
    if s.0 < s.1 {
        out.push(s);
    }
    out
}

fn trim_range(src: &str, mut a: usize, mut b: usize) -> (usize, usize) {
    while a < b && src.as_bytes()[a].is_ascii_whitespace() {
        a += 1;
    }
    while b > a && src.as_bytes()[b - 1].is_ascii_whitespace() {
        b -= 1;
    }
    (a, b)
}

pub fn statement_at(src: &str, cursor: usize) -> Option<(usize, usize)> {
    let stmts = split_statements(src);
    stmts
        .iter()
        .copied()
        .find(|&(a, b)| cursor >= a && cursor <= b)
        .or_else(|| stmts.iter().copied().rev().find(|&(_, b)| b <= cursor))
        .or_else(|| stmts.first().copied())
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmp {
    Eq,

    Ne,

    Gt,

    Ge,

    Lt,

    Le,

    Like,

    IsNull,

    IsNotNull,

    In(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    pub column: String,

    pub cmp: Cmp,

    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub columns: Vec<String>,

    pub schema: Option<String>,

    pub table: String,

    pub predicates: Vec<Predicate>,

    pub order: Option<(String, bool)>,

    pub limit: Option<usize>,

    pub count_only: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(Select),

    Update {
        table: String,

        has_where: bool,
    },

    Delete {
        table: String,

        has_where: bool,
    },

    Insert {
        table: String,
    },

    Drop {
        kind: String,

        name: String,
    },

    Truncate {
        table: String,
    },

    Alter {
        table: String,

        destructive: bool,
    },

    Create {
        kind: String,

        name: String,
    },

    Explain {
        analyze: bool,

        inner: Box<Statement>,
    },

    Other(String),
}

impl Statement {
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

struct Words<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
}

impl<'a> Words<'a> {
    fn new(src: &'a str) -> Self {
        let toks = tokenize(src)
            .into_iter()
            .filter(|t| !matches!(t.kind, TokKind::Whitespace | TokKind::Comment))
            .collect();
        Self { src, toks, pos: 0 }
    }
    fn peek(&self) -> Option<&str> {
        self.toks.get(self.pos).map(|t| &self.src[t.start..t.end])
    }
    fn peek_up(&self) -> String {
        self.peek().unwrap_or("").to_ascii_uppercase()
    }
    fn next(&mut self) -> Option<&str> {
        let t = self.toks.get(self.pos)?;
        self.pos += 1;
        Some(&self.src[t.start..t.end])
    }
    fn accept(&mut self, kw: &str) -> bool {
        if self.peek_up() == kw {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn ident(&mut self) -> Option<String> {
        let t = self.toks.get(self.pos)?;
        if t.kind == TokKind::Ident || t.kind == TokKind::Keyword {
            self.pos += 1;
            Some(self.src[t.start..t.end].trim_matches('"').to_owned())
        } else {
            None
        }
    }

    fn qualified(&mut self) -> Option<(Option<String>, String)> {
        let first = self.ident()?;
        if self.peek() == Some(".") {
            self.pos += 1;
            let second = self.ident()?;
            Some((Some(first), second))
        } else {
            Some((None, first))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,

    pub at: usize,
}

pub fn parse(stmt: &str) -> Result<Statement, ParseError> {
    let mut w = Words::new(stmt);
    let err = |w: &Words<'_>, m: &str| ParseError {
        message: m.to_owned(),
        at: w.toks.get(w.pos).map_or(stmt.len(), |t| t.start),
    };
    match w.peek_up().as_str() {
        "SELECT" => parse_select(&mut w).map(Statement::Select),
        "EXPLAIN" => {
            w.next();
            let mut analyze = false;
            loop {
                if w.accept("ANALYZE") {
                    analyze = true;
                } else if w.accept("VERBOSE") || w.accept("(") {
                    while let Some(t) = w.peek() {
                        if t == ")" {
                            w.next();
                            break;
                        }
                        if t.eq_ignore_ascii_case("ANALYZE") {
                            analyze = true;
                        }
                        w.next();
                    }
                } else {
                    break;
                }
            }
            let rest = &stmt[w.toks.get(w.pos).map_or(stmt.len(), |t| t.start)..];
            let inner = parse(rest).map_err(|e| ParseError {
                message: e.message,
                at: e.at.saturating_add(stmt.len().saturating_sub(rest.len())),
            })?;
            Ok(Statement::Explain {
                analyze,
                inner: Box::new(inner),
            })
        }
        "UPDATE" => {
            w.next();
            let (_, table) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected a table name after UPDATE"))?;
            if !w.accept("SET") {
                return Err(err(&w, "Expected SET"));
            }
            let has_where = stmt.to_ascii_uppercase().contains(" WHERE ")
                || stmt.to_ascii_uppercase().contains("\nWHERE");
            Ok(Statement::Update { table, has_where })
        }
        "DELETE" => {
            w.next();
            if !w.accept("FROM") {
                return Err(err(&w, "Expected FROM after DELETE"));
            }
            let (_, table) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected a table name"))?;
            let up = stmt.to_ascii_uppercase();
            let has_where = up.contains(" WHERE ") || up.contains("\nWHERE");
            Ok(Statement::Delete { table, has_where })
        }
        "INSERT" => {
            w.next();
            if !w.accept("INTO") {
                return Err(err(&w, "Expected INTO after INSERT"));
            }
            let (_, table) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected a table name"))?;
            Ok(Statement::Insert { table })
        }
        "DROP" => {
            w.next();
            let kind = w.next().unwrap_or("TABLE").to_ascii_uppercase();
            w.accept("IF");
            w.accept("EXISTS");
            let (_, name) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected an object name"))?;
            Ok(Statement::Drop { kind, name })
        }
        "TRUNCATE" => {
            w.next();
            w.accept("TABLE");
            let (_, table) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected a table name"))?;
            Ok(Statement::Truncate { table })
        }
        "ALTER" => {
            w.next();
            w.accept("TABLE");
            w.accept("IF");
            w.accept("EXISTS");
            let (_, table) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected a table name"))?;
            let up = stmt.to_ascii_uppercase();
            let destructive = up.contains("DROP COLUMN")
                || up.contains("DROP CONSTRAINT")
                || up.contains(" TYPE ");
            Ok(Statement::Alter { table, destructive })
        }
        "CREATE" => {
            w.next();
            let kind = w.next().unwrap_or("TABLE").to_ascii_uppercase();
            let (_, name) = w
                .qualified()
                .ok_or_else(|| err(&w, "Expected an object name"))?;
            Ok(Statement::Create { kind, name })
        }
        "" => Err(ParseError {
            message: "Empty statement".into(),
            at: 0,
        }),
        other => {
            if is_keyword(other) {
                Ok(Statement::Other(other.to_owned()))
            } else {
                Err(err(&w, &format!("syntax error at or near \"{other}\"")))
            }
        }
    }
}

fn parse_select(w: &mut Words<'_>) -> Result<Select, ParseError> {
    let err = |w: &Words<'_>, m: &str| ParseError {
        message: m.to_owned(),
        at: w.toks.get(w.pos).map_or(w.src.len(), |t| t.start),
    };
    w.accept("SELECT");
    w.accept("DISTINCT");
    let mut columns = Vec::new();
    let mut count_only = false;
    loop {
        if w.accept("FROM") {
            break;
        }
        let Some(tok) = w.next().map(str::to_owned) else {
            return Err(err(w, "Expected FROM"));
        };
        if tok == "," {
            continue;
        }
        if tok.eq_ignore_ascii_case("count") {
            count_only = true;

            while let Some(t) = w.next() {
                if t == ")" {
                    break;
                }
            }
            columns.push("count".into());
            continue;
        }
        if tok == "*" {
            columns.push("*".into());
            continue;
        }
        if w.accept("AS") {
            w.next();
        }
        columns.push(tok.trim_matches('"').to_owned());
    }
    let (schema, table) = w
        .qualified()
        .ok_or_else(|| err(w, "Expected a table name after FROM"))?;

    if w.peek()
        .is_some_and(|p| !is_keyword(p) && p != ";" && p != ")")
    {
        w.next();
    }
    let mut predicates = Vec::new();
    let mut order = None;
    let mut limit = None;
    if w.accept("WHERE") {
        loop {
            let column = w.ident().ok_or_else(|| err(w, "Expected a column name"))?;
            let column = if w.peek() == Some(".") {
                w.next();
                w.ident().ok_or_else(|| err(w, "Expected a column name"))?
            } else {
                column
            };
            let op = w
                .next()
                .map(str::to_ascii_uppercase)
                .ok_or_else(|| err(w, "Expected an operator"))?;
            let (cmp, value) = match op.as_str() {
                "=" => (Cmp::Eq, w.next().unwrap_or("").to_owned()),
                "!=" | "<>" => (Cmp::Ne, w.next().unwrap_or("").to_owned()),
                ">" => (Cmp::Gt, w.next().unwrap_or("").to_owned()),
                ">=" => (Cmp::Ge, w.next().unwrap_or("").to_owned()),
                "<" => (Cmp::Lt, w.next().unwrap_or("").to_owned()),
                "<=" => (Cmp::Le, w.next().unwrap_or("").to_owned()),
                "LIKE" | "ILIKE" => (Cmp::Like, w.next().unwrap_or("").to_owned()),
                "IS" => {
                    if w.accept("NOT") {
                        w.accept("NULL");
                        (Cmp::IsNotNull, String::new())
                    } else {
                        w.accept("NULL");
                        (Cmp::IsNull, String::new())
                    }
                }
                "IN" => {
                    let mut items = Vec::new();
                    w.accept("(");
                    while let Some(t) = w.next() {
                        if t == ")" {
                            break;
                        }
                        if t != "," {
                            items.push(t.trim_matches('\'').to_owned());
                        }
                    }
                    (Cmp::In(items), String::new())
                }
                _ => return Err(err(w, &format!("Unsupported operator {op}"))),
            };
            predicates.push(Predicate {
                column,
                cmp,
                value: value.trim_matches('\'').to_owned(),
            });
            if !w.accept("AND") {
                break;
            }
        }
    }
    if w.accept("ORDER") {
        if !w.accept("BY") {
            return Err(err(w, "Expected BY after ORDER"));
        }
        let c = w
            .ident()
            .ok_or_else(|| err(w, "Expected a column to order by"))?;
        let asc = !w.accept("DESC");
        w.accept("ASC");
        order = Some((c, asc));
    }
    if w.accept("LIMIT") {
        let n = w
            .next()
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| err(w, "Expected a number after LIMIT"))?;
        limit = Some(n);
    }
    if let Some(extra) = w.peek()
        && extra != ";"
    {
        return Err(err(w, &format!("syntax error at or near \"{extra}\"")));
    }
    Ok(Select {
        columns,
        schema,
        table,
        predicates,
        order,
        limit,
        count_only,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Safe,
    Write,
    Destructive,
}

pub fn tier(stmt: &Statement) -> Tier {
    match stmt {
        Statement::Select(_) => Tier::Safe,
        Statement::Explain { inner, .. } => tier(inner),
        Statement::Drop { .. }
        | Statement::Truncate { .. }
        | Statement::Alter {
            destructive: true, ..
        } => Tier::Destructive,
        _ => Tier::Write,
    }
}

pub fn is_dangerous(stmt: &Statement) -> bool {
    match stmt {
        Statement::Explain { inner, .. } => is_dangerous(inner),
        _ => {
            tier(stmt) == Tier::Destructive
                || matches!(
                    stmt,
                    Statement::Delete {
                        has_where: false,
                        ..
                    }
                )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Run,

    Confirm { deliberate: bool },

    Deny,
}

pub fn gate(level: tablepro_demo::db::SafeMode, stmt: &Statement) -> Decision {
    let t = tier(stmt);
    let write = t != Tier::Safe;
    if level == tablepro_demo::db::SafeMode::ReadOnly && write {
        return Decision::Deny;
    }
    let confirm = is_dangerous(stmt)
        || (level.requires_confirmation() && (write || level.applies_to_all_queries()));
    if !confirm {
        return Decision::Run;
    }
    let deliberate = level.requires_authentication() && (write || level.applies_to_all_queries());
    Decision::Confirm { deliberate }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Risk {
    pub tier: Tier,
    pub dangerous: bool,
    pub action: String,
    pub scope: String,
    pub summary: String,
    pub reversible: &'static str,
}

pub fn fmt_rows_to(n: usize, out: &mut impl core::fmt::Write) -> core::fmt::Result {
    if n >= 1_000_000 {
        write!(out, "{:.1} M", n as f64 / 1e6)
    } else if n >= 1000 {
        write!(out, "{:.1} k", n as f64 / 1e3)
    } else {
        write!(out, "{n}")
    }
}

pub fn fmt_rows(n: usize) -> String {
    let mut text = String::new();
    let _ = fmt_rows_to(n, &mut text);
    text
}

pub fn assess(stmt: &Statement, table: Option<&Table>) -> Risk {
    let rows = table.map_or(0, |t| t.row_count);
    let t = tier(stmt);
    let dangerous = is_dangerous(stmt);
    let (action, scope, risk, reversible) = match stmt {
        Statement::Update {
            table: tb,
            has_where: false,
        } => (
            "UPDATE without WHERE".to_owned(),
            format!("every row in {tb} ({} rows)", fmt_rows(rows)),
            "Overwrites the column values of all rows at once.".to_owned(),
            "Reversible only by a compensating UPDATE or a backup",
        ),
        Statement::Delete {
            table: tb,
            has_where: false,
        } => (
            "DELETE without WHERE".to_owned(),
            format!("every row in {tb} ({} rows)", fmt_rows(rows)),
            "Removes all rows; dependent rows may go with ON DELETE CASCADE.".to_owned(),
            "Not reversible without a backup",
        ),
        Statement::Update { table: tb, .. } => (
            "UPDATE".to_owned(),
            format!("matching rows in {tb}"),
            String::new(),
            "Reversible by a compensating UPDATE",
        ),
        Statement::Delete { table: tb, .. } => (
            "DELETE".to_owned(),
            format!("matching rows in {tb}"),
            String::new(),
            "Not reversible without a backup",
        ),
        Statement::Insert { table: tb } => (
            "INSERT".to_owned(),
            format!("new rows in {tb}"),
            String::new(),
            "Reversible by deleting the inserted rows",
        ),
        Statement::Drop { kind, name } => (
            format!("DROP {kind}"),
            if kind == "DATABASE" {
                format!("the whole database {name}")
            } else {
                format!(
                    "{name}, its {} rows, indexes and constraints",
                    fmt_rows(rows)
                )
            },
            "The object and everything stored in it disappears immediately.".to_owned(),
            "Not reversible",
        ),
        Statement::Truncate { table: tb } => (
            "TRUNCATE".to_owned(),
            format!("every row in {tb} ({} rows)", fmt_rows(rows)),
            "Removes all rows without firing row triggers; audit rows are not written.".to_owned(),
            "Not reversible without a backup",
        ),
        Statement::Alter {
            table: tb,
            destructive: true,
        } => (
            "ALTER TABLE with DROP".to_owned(),
            format!("{tb} structure"),
            "Dropping columns or constraints discards data and may break dependent views."
                .to_owned(),
            "Not reversible for dropped data",
        ),
        Statement::Alter { table: tb, .. } => (
            "ALTER TABLE".to_owned(),
            format!("{tb} structure"),
            "Schema changes lock the table while they run.".to_owned(),
            "Reversible by a compensating ALTER",
        ),
        Statement::Create { kind, name } => (
            format!("CREATE {kind}"),
            name.clone(),
            String::new(),
            "Reversible by DROP",
        ),
        Statement::Explain { inner, .. } => {
            let r = assess(inner, table);
            (
                format!("EXPLAIN ANALYZE runs {}", r.action),
                r.scope,
                r.summary,
                r.reversible,
            )
        }
        Statement::Select(s) => ("SELECT".to_owned(), s.table.clone(), String::new(), ""),
        Statement::Other(v) => (v.clone(), String::new(), String::new(), ""),
    };
    Risk {
        tier: t,
        dangerous,
        action,
        scope,
        summary: risk,
        reversible,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResultSet {
    pub columns: Vec<(String, ColType)>,

    pub rows: Vec<Vec<Value>>,

    pub total: usize,

    pub source: Option<String>,

    pub duration_ms: u32,

    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecError {
    pub message: String,
    pub detail: Option<String>,

    pub at: Option<usize>,
}

pub const ROW_CAP: usize = 500;

fn matches(pred: &Predicate, table: &Table, row: &[Value]) -> bool {
    let Some(ci) = table
        .columns
        .iter()
        .position(|c| c.name.eq_ignore_ascii_case(&pred.column))
    else {
        return false;
    };
    let v = &row[ci];
    match &pred.cmp {
        Cmp::IsNull => *v == Value::Null,
        Cmp::IsNotNull => *v != Value::Null,
        Cmp::In(items) => items.iter().any(|i| v.display().eq_ignore_ascii_case(i)),
        Cmp::Like => {
            let pat = pred.value.to_lowercase();
            let s = v.display().to_lowercase();
            let core = pat.trim_matches('%');
            match (pat.starts_with('%'), pat.ends_with('%')) {
                (true, true) => s.contains(core),
                (true, false) => s.ends_with(core),
                (false, true) => s.starts_with(core),
                (false, false) => s == core,
            }
        }
        cmp => {
            if *v == Value::Null {
                return false;
            }
            let ord = match (v.as_f64(), pred.value.parse::<f64>()) {
                (Some(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
                _ => v.display().to_lowercase().cmp(&pred.value.to_lowercase()),
            };
            match cmp {
                Cmp::Eq => ord.is_eq(),
                Cmp::Ne => !ord.is_eq(),
                Cmp::Gt => ord.is_gt(),
                Cmp::Ge => ord.is_ge(),
                Cmp::Lt => ord.is_lt(),
                Cmp::Le => ord.is_le(),
                _ => false,
            }
        }
    }
}

pub fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        (Value::Null, _) => std::cmp::Ordering::Greater,
        (_, Value::Null) => std::cmp::Ordering::Less,
        _ => match (a.as_f64(), b.as_f64()) {
            (Some(x), Some(y)) => x.total_cmp(&y),
            _ => a.display().to_lowercase().cmp(&b.display().to_lowercase()),
        },
    }
}

pub fn cmp_values_nulls_last(a: &Value, b: &Value, ascending: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Greater,
        (_, Value::Null) => Ordering::Less,
        _ => {
            let ordering = cmp_values(a, b);
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        }
    }
}

pub fn run_select(cat: &Catalog, sel: &Select) -> Result<ResultSet, ExecError> {
    let table = cat
        .find(sel.schema.as_deref(), &sel.table)
        .ok_or_else(|| ExecError {
            message: format!(
                "relation \"{}\" does not exist",
                match &sel.schema {
                    Some(s) => format!("{s}.{}", sel.table),
                    None => sel.table.clone(),
                }
            ),
            detail: Some("Check the schema search path or qualify the table name.".into()),
            at: None,
        })?;
    if table.columns.is_empty() {
        return Err(ExecError {
            message: format!("\"{}\" is not a table or view", table.name),
            detail: None,
            at: None,
        });
    }
    for p in &sel.predicates {
        if table.column(&p.column).is_none() {
            return Err(ExecError {
                message: format!("column \"{}\" does not exist", p.column),
                detail: Some(format!(
                    "Columns of {}: {}",
                    table.name,
                    table
                        .columns
                        .iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                at: None,
            });
        }
    }
    if let Some((c, _)) = &sel.order
        && table.column(c).is_none()
    {
        return Err(ExecError {
            message: format!("column \"{c}\" does not exist"),
            detail: None,
            at: None,
        });
    }

    let proj: Vec<usize> = if sel.columns.iter().any(|c| c == "*") || sel.count_only {
        (0..table.columns.len()).collect()
    } else {
        let mut idx = Vec::new();
        for c in &sel.columns {
            match table
                .columns
                .iter()
                .position(|tc| tc.name.eq_ignore_ascii_case(c))
            {
                Some(i) => idx.push(i),
                None => {
                    return Err(ExecError {
                        message: format!("column \"{c}\" does not exist"),
                        detail: Some(format!(
                            "Perhaps you meant one of: {}",
                            table
                                .columns
                                .iter()
                                .map(|c| c.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                        at: None,
                    });
                }
            }
        }
        idx
    };
    let scan = table.row_count.min(2_000);
    let mut all = tablepro_demo::db::rows(table, 0, scan);
    all.retain(|r| sel.predicates.iter().all(|p| matches(p, table, r)));
    if let Some((c, asc)) = &sel.order {
        let Some(ci) = table
            .columns
            .iter()
            .position(|tc| tc.name.eq_ignore_ascii_case(c))
        else {
            return Err(ExecError {
                message: format!("column \"{c}\" does not exist"),
                detail: None,
                at: None,
            });
        };
        all.sort_by(|a, b| cmp_values_nulls_last(&a[ci], &b[ci], *asc));
    }

    let total = if sel.predicates.is_empty() {
        table.row_count
    } else {
        ((all.len() as f64 / scan as f64) * table.row_count as f64).round() as usize
    };
    if sel.count_only {
        return Ok(ResultSet {
            columns: vec![("count".into(), ColType::Int)],
            rows: vec![vec![Value::Int(total as i64)]],
            total: 1,
            source: None,
            duration_ms: 12 + (table.row_count / 50_000) as u32,
            editable: false,
        });
    }
    let row_limit = sel.limit.unwrap_or(ROW_CAP).min(ROW_CAP);
    all.truncate(row_limit);
    let rows: Vec<Vec<Value>> = all
        .into_iter()
        .map(|r| proj.iter().map(|&i| r[i].clone()).collect())
        .collect();
    let columns = proj
        .iter()
        .map(|&i| (table.columns[i].name.clone(), table.columns[i].ty))
        .collect();
    let editable = table.kind == tablepro_demo::db::ObjectKind::Table
        && table
            .primary_key()
            .iter()
            .all(|pk| proj.iter().any(|&i| table.columns[i].name == pk.name))
        && !table.primary_key().is_empty();
    Ok(ResultSet {
        columns,
        rows,
        total,
        source: Some(table.qualified()),
        duration_ms: 3 + (total.min(5_000_000) / 20_000) as u32 + sel.predicates.len() as u32 * 2,
        editable,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNode {
    pub op: String,
    pub relation: Option<String>,
    pub detail: Vec<(String, String)>,
    pub cost: (f64, f64),
    pub rows: usize,
    pub actual_ms: Option<f64>,
    pub loops: usize,
    pub warning: Option<String>,
    pub children: Vec<PlanNode>,
}

pub fn explain(cat: &Catalog, sel: &Select, analyze: bool) -> Result<PlanNode, ExecError> {
    let table = cat
        .find(sel.schema.as_deref(), &sel.table)
        .ok_or_else(|| ExecError {
            message: format!("relation \"{}\" does not exist", sel.table),
            detail: None,
            at: None,
        })?;
    let n = table.row_count as f64;
    let indexed_pred = sel.predicates.iter().find(|p| {
        table.indexes.iter().any(|i| {
            i.columns
                .first()
                .is_some_and(|c| c.eq_ignore_ascii_case(&p.column))
        })
    });
    let selectivity = if sel.predicates.is_empty() {
        1.0
    } else {
        0.08_f64.powi(sel.predicates.len() as i32).max(0.0005)
    };
    let out_rows = ((n * selectivity).round() as usize).max(1);
    let scan = if let Some(p) = indexed_pred {
        let Some(index) = table.indexes.iter().find(|i| {
            i.columns
                .first()
                .is_some_and(|c| c.eq_ignore_ascii_case(&p.column))
        }) else {
            return Err(ExecError {
                message: format!("no index available for column \"{}\"", p.column),
                detail: None,
                at: None,
            });
        };
        PlanNode {
            op: "Index Scan".into(),
            relation: Some(table.qualified()),
            detail: vec![
                ("Index".into(), index.name.clone()),
                (
                    "Index Cond".into(),
                    format!("({} = '{}')", p.column, p.value),
                ),
            ]
            .into_iter()
            .chain(
                sel.predicates
                    .iter()
                    .filter(|q| q.column != p.column)
                    .map(|q| {
                        (
                            "Filter".into(),
                            format!("({} {} '{}')", q.column, cmp_sym(&q.cmp), q.value),
                        )
                    }),
            )
            .collect(),
            cost: (0.56, 8.4 + out_rows as f64 * 0.012),
            rows: out_rows,
            actual_ms: analyze.then_some(0.3 + out_rows as f64 * 0.004),
            loops: 1,
            warning: None,
            children: vec![],
        }
    } else {
        let mut detail = Vec::new();
        for q in &sel.predicates {
            detail.push((
                "Filter".into(),
                format!("({} {} '{}')", q.column, cmp_sym(&q.cmp), q.value),
            ));
        }
        if !sel.predicates.is_empty() {
            detail.push((
                "Rows Removed by Filter".into(),
                fmt_int((n - out_rows as f64).max(0.0) as usize),
            ));
        }
        let expensive = n > 500_000.0;
        PlanNode {
            op: if n > 100_000.0 {
                "Parallel Seq Scan".into()
            } else {
                "Seq Scan".into()
            },
            relation: Some(table.qualified()),
            detail,
            cost: (0.0, n * 0.0125 + 12.0),
            rows: out_rows,
            actual_ms: analyze.then_some(n * 0.00021),
            loops: 1,
            warning: expensive.then(|| {
                format!(
                    "sequential scan over {} rows; consider an index on {}",
                    fmt_int(table.row_count),
                    sel.predicates
                        .first()
                        .map_or("the filter column", |p| p.column.as_str())
                )
            }),
            children: vec![],
        }
    };
    let mut root = scan;
    if root.op.starts_with("Parallel") {
        root = PlanNode {
            op: "Gather".into(),
            relation: None,
            detail: vec![("Workers Planned".into(), "2".into())],
            cost: (root.cost.0 + 1000.0, root.cost.1 + 1200.0),
            rows: root.rows,
            actual_ms: root.actual_ms.map(|m| m + 4.2),
            loops: 1,
            warning: None,
            children: vec![root],
        };
    }
    if let Some((col, asc)) = &sel.order {
        let uses_index = table.indexes.iter().any(|i| {
            i.columns
                .first()
                .is_some_and(|c| c.eq_ignore_ascii_case(col))
        });
        let big = out_rows > 50_000;
        root = PlanNode {
            op: "Sort".into(),
            relation: None,
            detail: vec![
                (
                    "Sort Key".into(),
                    format!("{col}{}", if *asc { "" } else { " DESC" }),
                ),
                (
                    "Sort Method".into(),
                    if big {
                        format!("external merge  Disk: {}kB", out_rows / 8)
                    } else {
                        format!("quicksort  Memory: {}kB", (out_rows / 12).max(25))
                    },
                ),
            ],
            cost: (
                root.cost.1 + out_rows as f64 * 0.02,
                root.cost.1 + out_rows as f64 * 0.025,
            ),
            rows: out_rows,
            actual_ms: root.actual_ms.map(|m| m + out_rows as f64 * 0.0015),
            loops: 1,
            warning: (big && !uses_index)
                .then(|| "sort spills to disk; an index on the sort key would avoid it".into()),
            children: vec![root],
        };
    }
    if let Some(l) = sel.limit {
        root = PlanNode {
            op: "Limit".into(),
            relation: None,
            detail: vec![("Actual rows".into(), l.to_string())],
            cost: (
                root.cost.0,
                root.cost.0
                    + (root.cost.1 - root.cost.0) * (l as f64 / root.rows.max(1) as f64).min(1.0),
            ),
            rows: l.min(root.rows),
            actual_ms: root.actual_ms,
            loops: 1,
            warning: None,
            children: vec![root],
        };
    }
    if sel.count_only {
        root = PlanNode {
            op: "Aggregate".into(),
            relation: None,
            detail: vec![("Output".into(), "count(*)".into())],
            cost: (root.cost.1, root.cost.1 + 0.02),
            rows: 1,
            actual_ms: root.actual_ms.map(|m| m + 0.05),
            loops: 1,
            warning: None,
            children: vec![root],
        };
    }
    Ok(root)
}

fn cmp_sym(c: &Cmp) -> &'static str {
    match c {
        Cmp::Eq => "=",
        Cmp::Ne => "<>",
        Cmp::Gt => ">",
        Cmp::Ge => ">=",
        Cmp::Lt => "<",
        Cmp::Le => "<=",
        Cmp::Like => "~~*",
        Cmp::IsNull => "IS NULL",
        Cmp::IsNotNull => "IS NOT NULL",
        Cmp::In(_) => "= ANY",
    }
}

pub fn fmt_int(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && s.len().saturating_sub(i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}
