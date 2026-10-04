//! SQL completion analysis and candidate ranking.

use unicode_segmentation::UnicodeSegmentation;

use tablepro_domain::{Catalog, Table};

use crate::safety::fmt_rows;
use crate::tokenizer::{FUNCTIONS, KEYWORDS, TokKind, tokenize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzyBoundary {
    Identifier,
    Path,
    None,
}

impl FuzzyBoundary {
    pub fn contains(self, grapheme: &str) -> bool {
        match self {
            Self::Identifier => matches!(grapheme, "_" | "-" | "." | "::"),
            Self::Path => matches!(grapheme, "/" | "\\" | "." | "_" | "-"),
            Self::None => false,
        }
    }
}

pub fn fuzzy_with_boundary(
    label: &str,
    word: &str,
    boundary: FuzzyBoundary,
) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, Vec::new()));
    }
    let lg: Vec<&str> = label.graphemes(true).collect();
    let wg: Vec<&str> = word.graphemes(true).collect();
    if wg.len() > lg.len() {
        return subsequence(&lg, &wg);
    }
    let last_start = lg.len().saturating_sub(wg.len());
    for start in 0..=last_start {
        let hit = wg.iter().enumerate().all(|(k, w)| {
            lg.get(start.saturating_add(k))
                .is_some_and(|l| l.eq_ignore_ascii_case(w))
        });
        if hit {
            let idx: Vec<usize> = (start..start.saturating_add(wg.len())).collect();
            if start == 0 {
                return Some((0, idx));
            }
            let at_boundary = lg
                .get(start.saturating_sub(1))
                .is_some_and(|grapheme| boundary.contains(grapheme));
            return Some((if at_boundary { 10 } else { 30 }, idx));
        }
    }
    subsequence(&lg, &wg)
}

fn subsequence(lg: &[&str], wg: &[&str]) -> Option<(u32, Vec<usize>)> {
    let mut matched = Vec::with_capacity(wg.len());
    let mut li = 0usize;
    for w in wg {
        loop {
            let l = lg.get(li)?;
            if l.eq_ignore_ascii_case(w) {
                matched.push(li);
                li = li.saturating_add(1);
                break;
            }
            li = li.saturating_add(1);
        }
    }
    let last = matched.last().copied().unwrap_or(0) as u32;
    Some((60u32.saturating_add(last), matched))
}

/// Completion item category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Table,
    View,
    Column,
    Function,
    Schema,
    Alias,
}

impl CompletionKind {
    pub const fn priority(self) -> u32 {
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
    pub text: String,
    pub label: String,
    pub detail: String,
    pub kind: CompletionKind,
    pub score: u32,
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

/// Suggestions and exact UTF-8 range they replace in input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionBatch {
    pub items: Vec<Completion>,
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
    let statement_start = crate::tokenizer::statement_at(source, cursor).map_or(0, |(start, _)| start);
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

/// Complete token while retaining byte range needed to apply safely.
pub fn completion_batch(src: &str, cursor: usize, cat: &Catalog) -> CompletionBatch {
    let cursor = normalized_cursor(src, cursor);
    let (word, clause, qualifier) = context(src, cursor);
    let stmt = crate::tokenizer::statement_at(src, cursor)
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
                format!("{} · {}", t.schema, fmt_rows(t.row_count)),
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
            tablepro_domain::ObjectKind::Table | tablepro_domain::ObjectKind::View
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
                format!("{} · {} rows", t.schema, fmt_rows(t.row_count)),
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
    if t.kind == tablepro_domain::ObjectKind::View {
        CompletionKind::View
    } else {
        CompletionKind::Table
    }
}

fn col_detail(c: &tablepro_domain::Column) -> String {
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
