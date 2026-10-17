//! SQL tokenizer and keyword definitions.

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

/// Byte ranges of individual statements (split on `;` outside strings).
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

/// The statement containing byte `cursor` (or nearest before it).
pub fn statement_at(src: &str, cursor: usize) -> Option<(usize, usize)> {
    let stmts = split_statements(src);
    stmts
        .iter()
        .copied()
        .find(|&(a, b)| cursor >= a && cursor <= b)
        .or_else(|| stmts.iter().copied().rev().find(|&(_, b)| b <= cursor))
        .or_else(|| stmts.first().copied())
}
