//! `TablePro`'s typed filter editor and local filtering policy.

use crate::db::{ColType, Value};

/// All operators exposed by the filter editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Substring.
    Contains,
    /// Negated substring.
    NotContains,
    /// Prefix.
    StartsWith,
    /// Suffix.
    EndsWith,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
    /// NULL.
    IsNull,
    /// Not NULL.
    IsNotNull,
    /// Empty text.
    IsEmpty,
    /// Non-empty text.
    IsNotEmpty,
    /// Membership.
    In,
    /// Negated membership.
    NotIn,
    /// Inclusive range.
    Between,
    /// Wildcard pattern.
    Regex,
}

impl FilterOp {
    /// Stable menu order.
    pub const ALL: [Self; 18] = [
        Self::Eq,
        Self::Ne,
        Self::Contains,
        Self::NotContains,
        Self::StartsWith,
        Self::EndsWith,
        Self::Gt,
        Self::Ge,
        Self::Lt,
        Self::Le,
        Self::IsNull,
        Self::IsNotNull,
        Self::IsEmpty,
        Self::IsNotEmpty,
        Self::In,
        Self::NotIn,
        Self::Between,
        Self::Regex,
    ];
    /// Display label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Contains => "contains",
            Self::NotContains => "not contains",
            Self::StartsWith => "starts with",
            Self::EndsWith => "ends with",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::IsNull => "is NULL",
            Self::IsNotNull => "is not NULL",
            Self::IsEmpty => "is empty",
            Self::IsNotEmpty => "is not empty",
            Self::In => "in list",
            Self::NotIn => "not in list",
            Self::Between => "between",
            Self::Regex => "matches",
        }
    }
    /// Whether the operator needs a value.
    pub const fn needs_value(self) -> bool {
        !matches!(
            self,
            Self::IsNull | Self::IsNotNull | Self::IsEmpty | Self::IsNotEmpty
        )
    }
    /// Put type-appropriate options first, retaining the complete menu.
    pub fn ordered_for(ty: ColType) -> Vec<Self> {
        let preferred: &[Self] = match ty {
            ColType::Int | ColType::Numeric | ColType::Timestamp | ColType::Date => &[
                Self::Eq,
                Self::Ne,
                Self::Gt,
                Self::Ge,
                Self::Lt,
                Self::Le,
                Self::Between,
                Self::IsNull,
                Self::IsNotNull,
            ],
            ColType::Bool => &[Self::Eq, Self::IsNull, Self::IsNotNull],
            ColType::Enum => &[
                Self::Eq,
                Self::Ne,
                Self::In,
                Self::NotIn,
                Self::IsNull,
                Self::IsNotNull,
            ],
            ColType::Json => &[Self::Contains, Self::IsNull, Self::IsNotNull],
            _ => &[
                Self::Eq,
                Self::Ne,
                Self::Contains,
                Self::StartsWith,
                Self::EndsWith,
                Self::IsNull,
                Self::IsNotNull,
                Self::IsEmpty,
            ],
        };
        let mut out = preferred.to_vec();
        for op in Self::ALL {
            if !out.contains(&op) {
                out.push(op);
            }
        }
        out
    }
}

/// One filter predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// Column name.
    pub column: String,
    /// Operator.
    pub op: FilterOp,
    /// Primary value.
    pub value: String,
    /// Secondary value.
    pub value2: String,
    /// Whether this filter is active.
    pub enabled: bool,
}

impl Filter {
    /// Compact chip text.
    pub fn chip_label(&self) -> String {
        match self.op {
            FilterOp::Between => {
                format!("{} between {} and {}", self.column, self.value, self.value2)
            }
            op if !op.needs_value() => format!("{} {}", self.column, op.label()),
            op => format!(
                "{} {} '{}'",
                self.column,
                op.label(),
                self.value.replace('\'', "''")
            ),
        }
    }
    /// SQL predicate.
    pub fn to_sql(&self) -> String {
        let c = identifier(&self.column);
        let q = |s: &str| format!("'{}'", s.replace('\'', "''"));
        match self.op {
            FilterOp::Eq => format!("{c} = {}", q(&self.value)),
            FilterOp::Ne => format!("{c} <> {}", q(&self.value)),
            FilterOp::Contains => format!("{c} ILIKE {}", q(&format!("%{}%", self.value))),
            FilterOp::NotContains => format!("{c} NOT ILIKE {}", q(&format!("%{}%", self.value))),
            FilterOp::StartsWith => format!("{c} ILIKE {}", q(&format!("{}%", self.value))),
            FilterOp::EndsWith => format!("{c} ILIKE {}", q(&format!("%{}", self.value))),
            FilterOp::Gt => format!("{c} > {}", q(&self.value)),
            FilterOp::Ge => format!("{c} >= {}", q(&self.value)),
            FilterOp::Lt => format!("{c} < {}", q(&self.value)),
            FilterOp::Le => format!("{c} <= {}", q(&self.value)),
            FilterOp::IsNull => format!("{c} IS NULL"),
            FilterOp::IsNotNull => format!("{c} IS NOT NULL"),
            FilterOp::IsEmpty => format!("{c} = ''"),
            FilterOp::IsNotEmpty => format!("{c} <> ''"),
            FilterOp::In => format!(
                "{c} IN ({})",
                self.value
                    .split(',')
                    .map(|s| q(s.trim()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FilterOp::NotIn => format!(
                "{c} NOT IN ({})",
                self.value
                    .split(',')
                    .map(|s| q(s.trim()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FilterOp::Between => format!("{c} BETWEEN {} AND {}", q(&self.value), q(&self.value2)),
            FilterOp::Regex => format!("{c} ~ {}", q(&self.value)),
        }
    }
    /// Apply this filter to one value.
    pub fn matches(&self, value: &Value) -> bool {
        if !self.enabled {
            return true;
        }
        let text = value.display();
        let left = text.to_ascii_lowercase();
        let right = self.value.to_ascii_lowercase();
        match self.op {
            FilterOp::Eq => eq(value, &self.value),
            FilterOp::Ne => !eq(value, &self.value),
            FilterOp::Contains => left.contains(&right),
            FilterOp::NotContains => !left.contains(&right),
            FilterOp::StartsWith => left.starts_with(&right),
            FilterOp::EndsWith => left.ends_with(&right),
            FilterOp::Gt => cmp(value, &self.value).is_gt(),
            FilterOp::Ge => cmp(value, &self.value).is_ge(),
            FilterOp::Lt => cmp(value, &self.value).is_lt(),
            FilterOp::Le => cmp(value, &self.value).is_le(),
            FilterOp::IsNull => matches!(value, Value::Null),
            FilterOp::IsNotNull => !matches!(value, Value::Null),
            FilterOp::IsEmpty => text.is_empty(),
            FilterOp::IsNotEmpty => !text.is_empty(),
            FilterOp::In => self.value.split(',').any(|item| eq(value, item.trim())),
            FilterOp::NotIn => self.value.split(',').all(|item| !eq(value, item.trim())),
            FilterOp::Between => {
                cmp(value, &self.value).is_ge() && cmp(value, &self.value2).is_le()
            }
            FilterOp::Regex => wildcard(&left, &right),
        }
    }
}

/// Controlled filter form state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterDraft {
    /// Available column names.
    pub columns: Vec<String>,
    /// Selected column index.
    pub column: usize,
    /// Selected operator.
    pub op: FilterOp,
    /// Primary value.
    pub value: String,
    /// Secondary value for range predicates.
    pub value2: String,
    /// Whether the editor is open.
    pub open: bool,
}
impl FilterDraft {
    /// Build from column labels.
    pub fn new(columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            column: 0,
            op: FilterOp::Eq,
            value: String::new(),
            value2: String::new(),
            open: false,
        }
    }
    /// Open for a column/value.
    pub fn open_for(&mut self, column: usize, value: impl Into<String>) {
        self.column = column.min(self.columns.len().saturating_sub(1));
        self.value = value.into();
        self.value2.clear();
        self.open = true;
    }
    /// Close the editor.
    pub const fn close(&mut self) {
        self.open = false;
    }
    /// Build an active filter.
    pub fn build(&self) -> Option<Filter> {
        Some(Filter {
            column: self.columns.get(self.column)?.clone(),
            op: self.op,
            value: self.value.clone(),
            value2: self.value2.clone(),
            enabled: true,
        })
    }
}

fn identifier(name: &str) -> String {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        name.to_owned()
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}
fn eq(value: &Value, text: &str) -> bool {
    value
        .display()
        .eq_ignore_ascii_case(text.trim_matches('\''))
}
fn cmp(value: &Value, text: &str) -> std::cmp::Ordering {
    match (value.as_f64(), text.trim().parse::<f64>()) {
        (Some(a), Ok(b)) => a.total_cmp(&b),
        _ => value
            .display()
            .to_ascii_lowercase()
            .cmp(&text.to_ascii_lowercase()),
    }
}
fn wildcard(value: &str, pattern: &str) -> bool {
    let mut at = 0usize;
    let parts: Vec<&str> = pattern.split('*').filter(|part| !part.is_empty()).collect();
    for part in parts {
        let Some(rest) = value.get(at..) else {
            return false;
        };
        let Some(offset) = rest.find(part) else {
            return false;
        };
        let consumed = offset.saturating_add(part.len());
        at = at.saturating_add(consumed);
    }
    pattern.ends_with('*') || at == value.len()
}
