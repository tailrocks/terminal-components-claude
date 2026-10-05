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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_keywords_and_tokenize() {
        assert!(is_keyword("SELECT"));
        assert!(is_keyword("from"));
        assert!(!is_keyword("my_custom_table"));

        let tokens = tokenize("SELECT 1;");
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].kind, TokKind::Keyword);
    }
}
