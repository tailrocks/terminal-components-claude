//! `tablepro-sql`
//!
//! SQL parsing, classification, completion analysis and deterministic
//! pending-change SQL generation.

#![forbid(unsafe_code)]

mod ast;
mod completion;
mod parser;
mod preview;
mod safety;
mod tokenizer;

pub use ast::{Cmp, Predicate, Select, Statement};
pub use completion::{
    Completion, CompletionBatch, CompletionKind, FuzzyBoundary, auto_trigger, complete,
    completion_batch, fuzzy_with_boundary,
};
pub use parser::{ParseError, parse};
pub use preview::{preview_sql, sql_literal};
pub use safety::{
    Decision, Risk, Tier, assess, fmt_rows, gate, is_dangerous, risk_assessment, tier,
};
pub use tokenizer::{
    FUNCTIONS, KEYWORDS, TokKind, Token, is_keyword, split_statements, statement_at, tokenize,
};
