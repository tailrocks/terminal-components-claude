//! Source-only expansion of `docs/refactoring-plan/showcase-scenarios.tsv`.

use crate::actions::ActionProgram;
use crate::color::{ColorSpec, TerminalSize};
use crate::error::AdapterError;
use crate::pages::OraclePageId;
use crate::{DEFAULT_PAGE_FRAME_COUNT, ORACLE_PAGE_COUNT};

/// Pinned scenario inventory. Authority remains the repository TSV.
const TSV: &str = include_str!("../../../../docs/refactoring-plan/showcase-scenarios.tsv");

/// One immutable SC row before size/color expansion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioRow {
    /// Stable scenario id (`SC-BASE-overview`).
    pub scenario_id: String,
    /// `all23` or a single oracle page slug.
    pub page: ScenarioPage,
    /// Finite size axis.
    pub sizes: Vec<TerminalSize>,
    /// Flattened action program.
    pub program: ActionProgram,
    /// Required observable proof text.
    pub required_observable_proof: String,
    /// Source references.
    pub source_refs: String,
    /// Component tags.
    pub components: String,
}

/// Page column: one oracle page or every oracle page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioPage {
    /// One page.
    One(OraclePageId),
    /// Expand across all 23 oracle pages including Diff.
    All23,
}

/// One finite expanded capture identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpandedCase {
    /// Parent scenario id.
    pub scenario_id: String,
    /// Oracle page.
    pub page: OraclePageId,
    /// Size.
    pub size: TerminalSize,
    /// Colour.
    pub color: ColorSpec,
    /// Action program (fresh-run per case).
    pub program: ActionProgram,
}

impl ExpandedCase {
    /// Stable membership key. Candidate output does not select this.
    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.scenario_id,
            self.page.slug(),
            self.size.token(),
            self.color.token()
        )
    }
}

/// Parse the pinned TSV.
pub fn rows() -> Result<Vec<ScenarioRow>, AdapterError> {
    let mut lines = TSV.lines();
    let Some(header) = lines.next() else {
        return Err(catalog("empty showcase-scenarios.tsv"));
    };
    if header
        != "scenario_id\tpage\tsizes\taction_checkpoints\trequired_observable_proof\tsource_refs\tcomponents"
    {
        return Err(catalog("unexpected showcase-scenarios.tsv header"));
    }
    let mut rows = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        rows.push(parse_row(index.saturating_add(2), line)?);
    }
    if rows.is_empty() {
        return Err(catalog("showcase-scenarios.tsv has no data rows"));
    }
    Ok(rows)
}

fn parse_row(line_no: usize, line: &str) -> Result<ScenarioRow, AdapterError> {
    let cols: Vec<&str> = line.split('\t').collect();
    if cols.len() != 7 {
        return Err(catalog(&format!(
            "line {line_no}: expected 7 columns, got {}",
            cols.len()
        )));
    }
    let Some(scenario_id) = cols.first() else {
        return Err(catalog(&format!("line {line_no}: missing scenario_id")));
    };
    let Some(page_col) = cols.get(1) else {
        return Err(catalog(&format!("line {line_no}: missing page")));
    };
    let Some(sizes_col) = cols.get(2) else {
        return Err(catalog(&format!("line {line_no}: missing sizes")));
    };
    let Some(actions) = cols.get(3) else {
        return Err(catalog(&format!("line {line_no}: missing actions")));
    };
    let Some(proof) = cols.get(4) else {
        return Err(catalog(&format!("line {line_no}: missing proof")));
    };
    let Some(source_refs) = cols.get(5) else {
        return Err(catalog(&format!("line {line_no}: missing source_refs")));
    };
    let Some(components) = cols.get(6) else {
        return Err(catalog(&format!("line {line_no}: missing components")));
    };
    if !scenario_id.starts_with("SC-") {
        return Err(catalog(&format!(
            "line {line_no}: scenario id {scenario_id} does not start with SC-"
        )));
    }
    Ok(ScenarioRow {
        scenario_id: (*scenario_id).to_owned(),
        page: parse_page(line_no, page_col)?,
        sizes: parse_sizes(line_no, sizes_col)?,
        program: ActionProgram::parse(actions),
        required_observable_proof: (*proof).to_owned(),
        source_refs: (*source_refs).to_owned(),
        components: (*components).to_owned(),
    })
}

fn parse_page(line_no: usize, value: &str) -> Result<ScenarioPage, AdapterError> {
    if value == "all23" {
        return Ok(ScenarioPage::All23);
    }
    OraclePageId::from_slug(value)
        .map(ScenarioPage::One)
        .ok_or_else(|| catalog(&format!("line {line_no}: unknown page {value}")))
}

fn parse_sizes(line_no: usize, value: &str) -> Result<Vec<TerminalSize>, AdapterError> {
    let mut sizes = Vec::new();
    for token in value.split(',') {
        let token = token.trim();
        let Some(size) = TerminalSize::parse(token) else {
            return Err(catalog(&format!("line {line_no}: unparsable size {token}")));
        };
        sizes.push(size);
    }
    if sizes.is_empty() {
        return Err(catalog(&format!("line {line_no}: empty sizes")));
    }
    Ok(sizes)
}

fn catalog(message: &str) -> AdapterError {
    AdapterError::Catalog {
        message: message.to_owned(),
    }
}

/// Expand every SC row across its page/size/color axis.
///
/// `SC-BASE` rows expand across the default four-colour axis (368 frames);
/// stateful rows replay at truecolor per the scenario contract, which keeps
/// full geometry on the defaults while avoiding redundant Cartesian products.
pub fn expand() -> Result<Vec<ExpandedCase>, AdapterError> {
    let mut out = Vec::new();
    for row in rows()? {
        let singleton;
        let pages: &[OraclePageId] = match row.page {
            ScenarioPage::One(page) => {
                singleton = [page];
                &singleton
            }
            ScenarioPage::All23 => &OraclePageId::ALL,
        };
        let colors = colors_for(&row.scenario_id);
        for page in pages {
            for size in &row.sizes {
                for color in colors {
                    out.push(ExpandedCase {
                        scenario_id: row.scenario_id.clone(),
                        page: *page,
                        size: *size,
                        color: *color,
                        program: row.program.clone(),
                    });
                }
            }
        }
    }
    reject_duplicates(&out)?;
    Ok(out)
}

fn colors_for(scenario_id: &str) -> &'static [ColorSpec] {
    if scenario_id.starts_with("SC-BASE-") {
        &ColorSpec::DEFAULT_AXIS
    } else {
        &[ColorSpec::TrueColor]
    }
}

fn reject_duplicates(cases: &[ExpandedCase]) -> Result<(), AdapterError> {
    let mut seen = std::collections::BTreeSet::new();
    for case in cases {
        let id = case.identity();
        if !seen.insert(id.clone()) {
            return Err(catalog(&format!("duplicate expansion identity {id}")));
        }
    }
    Ok(())
}

/// Default page frames: every SC-BASE row × default sizes × four colours.
pub fn default_page_frames() -> Result<Vec<ExpandedCase>, AdapterError> {
    let cases = expand()?
        .into_iter()
        .filter(|case| case.scenario_id.starts_with("SC-BASE-"))
        .collect::<Vec<_>>();
    if cases.len() != DEFAULT_PAGE_FRAME_COUNT {
        return Err(catalog(&format!(
            "default page frames: expected {DEFAULT_PAGE_FRAME_COUNT}, got {}",
            cases.len()
        )));
    }
    Ok(cases)
}

/// SC-BASE-overview at 80×24 truecolor — first capture leaf.
#[must_use]
pub fn sc_base_overview_80x24_truecolor() -> ExpandedCase {
    ExpandedCase {
        scenario_id: "SC-BASE-overview".to_owned(),
        page: OraclePageId::Overview,
        size: TerminalSize::new(80, 24),
        color: ColorSpec::TrueColor,
        program: ActionProgram::parse("fresh draw"),
    }
}

/// Membership facts that CHK-002-style accounting can re-check.
pub fn membership() -> Result<Membership, AdapterError> {
    let rows = rows()?;
    let frames = default_page_frames()?;
    let expanded = expand()?;
    if OraclePageId::ALL.len() != ORACLE_PAGE_COUNT {
        return Err(catalog("oracle ALL is not 23"));
    }
    if !OraclePageId::ALL.contains(&OraclePageId::Diff) {
        return Err(catalog("oracle ALL missing Diff"));
    }
    Ok(Membership {
        scenario_rows: rows.len(),
        oracle_pages: ORACLE_PAGE_COUNT,
        default_page_frames: frames.len(),
        expanded_cases: expanded.len(),
        production_pages: crate::pages::production_page_len(),
        production_nav: crate::pages::production_nav_len(),
        diff_production_present: OraclePageId::Diff.production_present(),
    })
}

/// Exact inventory counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Membership {
    /// TSV data rows.
    pub scenario_rows: usize,
    /// 23 oracle pages.
    pub oracle_pages: usize,
    /// 368 default frames.
    pub default_page_frames: usize,
    /// Full finite expansion.
    pub expanded_cases: usize,
    /// Production `PageId::ALL` length (22 on this candidate).
    pub production_pages: usize,
    /// Production nav length.
    pub production_nav: usize,
    /// Whether production exposes Diff.
    pub diff_production_present: bool,
}
