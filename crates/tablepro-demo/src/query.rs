//! Deterministic query execution and plan synthesis against in-memory catalog.

use tablepro_domain::{Catalog, ColType, ObjectKind, ResultSet, Table, Value};
use tablepro_sql::{Cmp, Predicate, Select};

use crate::db::rows;

/// Maximum rows materialized for a single result.
pub const ROW_CAP: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecError {
    pub message: String,
    pub detail: Option<String>,
    pub at: Option<usize>,
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

/// Evaluate a SELECT against the demo catalog.
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
    // projection
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
    let mut all = rows(table, 0, scan);
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
        all.sort_by(|a, b| {
            let o = cmp_values(&a[ci], &b[ci]);
            if *asc { o } else { o.reverse() }
        });
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
    let editable = table.kind == ObjectKind::Table
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

/// Build a deterministic PostgreSQL-style plan for a SELECT.
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

/// Render a plan as PostgreSQL-style text lines.
pub fn plan_text(node: &PlanNode, depth: usize, out: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let arrow = if depth == 0 { "" } else { "->  " };
    let rel = node
        .relation
        .as_ref()
        .map(|r| format!(" on {r}"))
        .unwrap_or_default();
    let actual = node
        .actual_ms
        .map(|m| {
            format!(
                " (actual time=0.031..{m:.3} rows={} loops={})",
                node.rows, node.loops
            )
        })
        .unwrap_or_default();
    out.push(format!(
        "{indent}{arrow}{}{rel}  (cost={:.2}..{:.2} rows={} width=64){actual}",
        node.op, node.cost.0, node.cost.1, node.rows
    ));
    for (k, v) in &node.detail {
        out.push(format!("{indent}      {k}: {v}"));
    }
    for c in &node.children {
        plan_text(c, depth.saturating_add(1), out);
    }
}


