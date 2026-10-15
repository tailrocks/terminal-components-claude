//! SQL parser for deterministic workbench subset.

use crate::ast::{Cmp, Predicate, Select, Statement};
use crate::tokenizer::{TokKind, Token, is_keyword, tokenize};

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

    /// `schema.name` or `name`
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

/// Error returned when parsing a statement fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable parse error.
    pub message: String,
    /// Byte offset within the statement.
    pub at: usize,
}

/// Parse one SQL statement in deterministic `TablePro` subset.
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
