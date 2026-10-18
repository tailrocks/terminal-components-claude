//! Result values and result set.

use crate::schema::ColType;

/// One result-cell value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// SQL NULL.
    Null,
    /// Column default.
    Default,
    /// Text value.
    Text(String),
    /// Signed integer.
    Int(i64),
    /// Decimal number.
    Num(f64),
    /// Boolean value.
    Bool(bool),
    /// JSON text.
    Json(String),
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Value::Null => "NULL".into(),
            Value::Default => "DEFAULT".into(),
            Value::Text(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Num(n) => format!("{n:.2}"),
            Value::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Value::Json(j) => j.clone(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Num(n) => Some(*n),
            _ => None,
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

/// Rectangular query result set.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultSet {
    pub columns: Vec<(String, ColType)>,
    pub rows: Vec<Vec<Value>>,
    pub total: usize,
    pub source: Option<String>,
    pub duration_ms: u32,
    pub editable: bool,
}
