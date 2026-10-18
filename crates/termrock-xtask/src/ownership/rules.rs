#![doc = r" Ownership rules: what counts as raw preview painting and where the"]
#![doc = r" consumer/author boundary lies."]
#![doc = r""]
#![doc = r" Every rule carries a stable `OWN-nn` id, a scope (preview crates,"]
#![doc = r" library crates, or all production sources), and a matcher over scanned"]
#![doc = r" files. Findings route to the versioned exceptions file, which ratchets"]
#![doc = r" the grandfathered corpus: counts may shrink, never grow."]

use std::collections::BTreeMap;

use super::scan::{ScannedFile, is_raw_buffer_path, receiver_before, words};

#[doc = r" A single rule hit."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    #[doc = r" Stable rule id (`OWN-01` …)."]
    pub rule: &'static str,
    #[doc = r" Workspace-relative file path."]
    pub file: String,
    #[doc = r" 1-based line number."]
    pub line: usize,
    #[doc = r" Enclosing production `fn`, if any."]
    pub symbol: Option<String>,
    #[doc = r" Human-readable detail (method, import, macro name …)."]
    pub detail: String,
    #[doc = r" True when the hit sits in a generated file."]
    pub generated: bool,
}

#[doc = r" Rule scope over workspace members."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    #[doc = r" Preview applications and their supporting crates"]
    #[doc = r" (every member outside `crates/termrock*`)."]
    Preview,
    #[doc = r" Library crates (`crates/termrock*`)."]
    Library,
    #[doc = r" All production sources except tooling crates."]
    Production,
}

#[doc = r" A rule definition."]
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    pub scope: Scope,
    pub summary: &'static str,
}

#[doc = r" The full catalog. Ids are stable: never reuse or renumber."]
pub const RULES: &[Rule] = &[
    Rule {
        id: "OWN-01",
        name: "preview-raw-buffer",
        scope: Scope::Preview,
        summary: "preview code imports a raw ratatui buffer/cell type",
    },
    Rule {
        id: "OWN-02",
        name: "preview-paint-call",
        scope: Scope::Preview,
        summary: "preview code calls a cell-writing paint method",
    },
    Rule {
        id: "OWN-03",
        name: "preview-raw-buffer-escape",
        scope: Scope::Preview,
        summary: "preview code escapes to the raw buffer via Ui::raw/peek",
    },
    Rule {
        id: "OWN-04",
        name: "preview-paint-wrapper-call",
        scope: Scope::Preview,
        summary: "preview code calls a local paint-wrapper fn",
    },
    Rule {
        id: "OWN-05",
        name: "preview-paint-macro",
        scope: Scope::Preview,
        summary: "a preview macro paints cells or is invoked",
    },
    Rule {
        id: "OWN-06",
        name: "preview-stored-frame-include",
        scope: Scope::Preview,
        summary: "preview code includes a stored frame/asset file",
    },
    Rule {
        id: "OWN-07",
        name: "preview-screen-literal",
        scope: Scope::Preview,
        summary: "preview code embeds a screen-shaped string literal",
    },
    Rule {
        id: "OWN-08",
        name: "preview-fixture-render-gate",
        scope: Scope::Preview,
        summary: "preview rendering branches on test/debug/env input",
    },
    Rule {
        id: "OWN-09",
        name: "preview-paused-bypass",
        scope: Scope::Preview,
        summary: "preview code keys behavior off paused state",
    },
    Rule {
        id: "OWN-10",
        name: "preview-scenario-render-switch",
        scope: Scope::Preview,
        summary: "preview code selects behavior by scenario id",
    },
    Rule {
        id: "OWN-11",
        name: "preview-size-render-switch",
        scope: Scope::Preview,
        summary: "preview code selects behavior by width/height comparison",
    },
    Rule {
        id: "OWN-12",
        name: "library-preview-dependency",
        scope: Scope::Library,
        summary: "library code depends on a preview crate",
    },
    Rule {
        id: "OWN-13",
        name: "library-stored-frame",
        scope: Scope::Library,
        summary: "production library code references stored expected frames",
    },
    Rule {
        id: "OWN-14",
        name: "expect-in-production",
        scope: Scope::Production,
        summary: "production code names expect outside the allowed place",
    },
];

#[doc = r" Cell-writing `Ui` methods. Registration, measurement, and style"]
#[doc = r" resolution are intentionally absent: they are not painting."]
const UI_PAINT_METHODS: &[&str] = &[
    "paint_cell",
    "paint_str",
    "paint_matched",
    "paint_middle",
    "paint_spans",
    "paint_style",
    "fill",
    "rule",
    "frame",
    "glyph",
    "dim_layer",
    "scroll_edges",
    "scroll_edges_except",
];

#[doc = r" Distinctive ratatui `Buffer` methods. These names are buffer-specific,"]
#[doc = r" so unlike the `Ui` list they match on any receiver."]
const BUFFER_PAINT_METHODS: &[&str] = &[
    "set_string",
    "set_stringn",
    "set_span",
    "set_style",
    "set_line",
    "cell_mut",
];

#[doc = r" Tooling crates excluded from [`Scope::Production`]: harness and xtask"]
#[doc = r" code fails fast on violated test protocol by design."]
pub const TOOLING_MEMBERS: &[&str] = &["termrock-test-support", "termrock-xtask"];

#[doc = r" True for members under `crates/termrock*` (library and tooling)."]
pub fn is_library_member(member: &str) -> bool {
    member == "crates/termrock"
        || member.starts_with("crates/termrock-")
        || member.starts_with("crates/termrock/")
}

#[doc = r" True for production-scope members: everything but tooling crates."]
pub fn is_production_member(member: &str) -> bool {
    let name = member.rsplit('/').next().unwrap_or(member);
    !TOOLING_MEMBERS.contains(&name)
}

#[doc = r" Crate names (`jackin_preview_app`) of preview members, derived from the"]
#[doc = r" workspace member list."]
pub fn preview_crate_names(members: &[String]) -> Vec<String> {
    members
        .iter()
        .filter(|m| !is_library_member(m))
        .map(|m| m.rsplit('/').next().unwrap_or(m).replace('-', "_"))
        .collect()
}

#[doc = r" Run every rule over one scanned file. Wrapper call-site detection"]
#[doc = r" ([`RULES`] `OWN-04`) runs separately per crate once wrappers are known."]
pub fn match_file(file: &ScannedFile, preview_crates: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let preview = !is_library_member(&file.member);
    if preview {
        match_imports(file, &mut findings);
        match_paint_calls(file, &mut findings);
        match_raw_escapes(file, &mut findings);
        match_macros(file, &mut findings);
        match_includes(file, &mut findings);
        match_screen_literals(file, &mut findings);
        match_fixture_gates(file, &mut findings);
        match_paused(file, &mut findings);
        match_scenario_switches(file, &mut findings);
        match_size_switches(file, &mut findings);
    } else {
        match_library_preview_deps(file, preview_crates, &mut findings);

        if is_production_member(&file.member) {
            match_library_stored_frames(file, &mut findings);
        }
    }
    if is_production_member(&file.member) {
        match_expects(file, &mut findings);
    }
    findings
}

fn production_lines(file: &ScannedFile) -> impl Iterator<Item = &super::scan::CodeLine> {
    file.lines
        .iter()
        .filter(|line| !file.is_test_line(line.number))
}

fn finding(file: &ScannedFile, rule: &'static str, line: usize, detail: String) -> Finding {
    Finding {
        rule,
        file: file.path.clone(),
        line,
        symbol: file.enclosing_fn(line).map(str::to_string),
        detail,
        generated: file.generated,
    }
}

#[doc = r" OWN-01: raw buffer/cell imports resolve through aliases."]
fn match_imports(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for (local, path) in &file.aliases {
        if !is_raw_buffer_path(path) {
            continue;
        }
        let line = import_line(file, local).unwrap_or(1);
        if file.is_test_line(line) {
            continue;
        }
        findings.push(finding(
            file,
            "OWN-01",
            line,
            format!("imports raw buffer type {path} as {local}"),
        ));
    }
}

#[doc = r" The production line importing `local` (best effort for diagnostics)."]
fn import_line(file: &ScannedFile, local: &str) -> Option<usize> {
    production_lines(file)
        .filter(|line| line.code.contains("use "))
        .find(|line| words(&line.code).contains(&local))
        .map(|line| line.number)
}

#[doc = r" OWN-02: `Ui` paint methods on Ui-typed receivers (the conventional"]
#[doc = r" `ui` plus import-alias params), and distinctive `Buffer` methods on"]
#[doc = r" any receiver."]
fn match_paint_calls(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        for method in UI_PAINT_METHODS.iter().chain(BUFFER_PAINT_METHODS.iter()) {
            for dot in call_dots(&line.code, method) {
                let Some(receiver) = receiver_before(&line.code, dot) else {
                    continue;
                };
                let ui_method = UI_PAINT_METHODS.contains(method);
                if ui_method && !file.is_ui_ident(&receiver) {
                    continue;
                }
                findings.push(finding(
                    file,
                    "OWN-02",
                    line.number,
                    format!("{receiver}.{method} paints cells"),
                ));
            }
        }
    }
}

#[doc = r" OWN-03: `Ui::raw`/`peek` escapes, receiver-resolved so iterator"]
#[doc = r" `peek` and unrelated `raw` methods never match."]
fn match_raw_escapes(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        for method in ["raw", "peek"] {
            for dot in call_dots(&line.code, method) {
                let Some(receiver) = receiver_before(&line.code, dot) else {
                    continue;
                };
                if !file.is_ui_ident(&receiver) {
                    continue;
                }
                findings.push(finding(
                    file,
                    "OWN-03",
                    line.number,
                    format!("{receiver}.{method} escapes to the raw buffer"),
                ));
            }
        }
    }
}

#[doc = r" True when a macro body calls `ui.<method>(` or `$ui.<method>(`."]
#[doc = r" Macro bodies lack type context, so only the conventional `ui` receiver"]
#[doc = r" (concrete or fragment) counts; other receivers stay silent."]
fn macro_body_paints(body: &str, method: &str) -> bool {
    for receiver in ["ui.", "$ui."] {
        let needle = format!("{receiver}{method}");
        let mut search = 0;
        while let Some(offset) = body[search..].find(&needle) {
            let at = search + offset;
            let before_ok = body[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '$');
            let after = &body[at + needle.len()..];
            if before_ok && (after.starts_with('(') || after.starts_with("::")) {
                return true;
            }
            search = at + needle.len();
        }
    }
    false
}

#[doc = r" Byte offsets of `.method(` call dots in blanked code."]
fn call_dots(code: &str, method: &str) -> Vec<usize> {
    let mut dots = Vec::new();
    let mut search = 0;
    let needle = format!(".{method}");
    while let Some(offset) = code[search..].find(&needle) {
        let dot = search + offset;
        let after = dot + needle.len();
        let rest = &code[after..];

        let is_call = rest.starts_with('(') || rest.starts_with("::");
        let boundary = rest
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if is_call && boundary {
            dots.push(dot);
        }
        search = after;
    }
    dots
}

#[doc = r" OWN-05: macro definitions whose body paints, plus their call sites."]
#[doc = r" Macro bodies lack type context, so only the conventional `ui` receiver"]
#[doc = r" counts inside a definition; call sites of a flagged macro always count."]
fn match_macros(file: &ScannedFile, findings: &mut Vec<Finding>) {
    let mut painted = Vec::new();
    for (name, range) in &file.macros {
        let body: String = file
            .lines
            .iter()
            .filter(|line| range.contains(line.number) && !file.is_test_line(line.number))
            .map(|line| line.code.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let paints = UI_PAINT_METHODS
            .iter()
            .any(|method| macro_body_paints(&body, method));
        if paints {
            painted.push(name.clone());
            findings.push(finding(
                file,
                "OWN-05",
                range.start,
                format!("macro {name} paints cells"),
            ));
        }
    }
    for name in &painted {
        let bang = format!("{name}!");
        for line in production_lines(file) {
            if line.code.contains(&bang) && !line.code.contains("macro_rules") {
                findings.push(finding(
                    file,
                    "OWN-05",
                    line.number,
                    format!("invokes paint macro {name}"),
                ));
            }
        }
    }
}

#[doc = r" OWN-06: `include_str!`/`include_bytes!` (stored frames/assets as output)."]
fn match_includes(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        if line.code.contains("include_str!") || line.code.contains("include_bytes!") {
            findings.push(finding(
                file,
                "OWN-06",
                line.number,
                "includes a stored file as application output".to_string(),
            ));
        }
    }
}

#[doc = r" OWN-07: screen-shaped literals: 3+ physical lines with box-drawing"]
#[doc = r" characters on 2+ lines. Single box glyphs in content strings never match."]
fn match_screen_literals(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for span in &file.strings {
        if span.end_line - span.start_line < 2 {
            continue;
        }
        let boxed = span
            .text
            .lines()
            .filter(|line| line.chars().any(is_box_char))
            .count();
        if boxed >= 2 {
            findings.push(finding(
                file,
                "OWN-07",
                span.start_line,
                "embeds a screen-shaped string literal".to_string(),
            ));
        }
    }
}

fn is_box_char(char: char) -> bool {
    ('\u{2500}'..='\u{257f}').contains(&char)
}

#[doc = r" OWN-08: `cfg!(test/debug_assertions)` branches and compile-time env"]
#[doc = r" reads gating preview rendering. `env::args`/`std::env::var` (runtime"]
#[doc = r" CLI handling) are not compile-time gates and never match."]
fn match_fixture_gates(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        let code = &line.code;
        let gated = code.contains("cfg!(test)")
            || code.contains("cfg!(debug_assertions)")
            || code.contains("option_env!")
            || code.contains("env!(");
        if gated {
            findings.push(finding(
                file,
                "OWN-08",
                line.number,
                "gates rendering on test/debug/env input".to_string(),
            ));
        }
    }
}

#[doc = r" OWN-09: `paused` keys behavior off paused state; each site needs a"]
#[doc = r" review verdict that input, layers, and component behavior stay live."]
fn match_paused(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        if words(&line.code).contains(&"paused") {
            findings.push(finding(
                file,
                "OWN-09",
                line.number,
                "keys behavior off paused state".to_string(),
            ));
        }
    }
}

#[doc = r" OWN-10: `match` on a scenario scrutinee and scenario-id equality."]
fn match_scenario_switches(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        let tokens = words(&line.code);
        let has_scenario = tokens.contains(&"scenario");
        if !has_scenario {
            continue;
        }
        let code = &line.code;
        let switched = tokens.contains(&"match")
            || code.contains("scenario ==")
            || code.contains("== scenario")
            || code.contains("scenario !=")
            || code.contains("!= scenario");
        if switched {
            findings.push(finding(
                file,
                "OWN-10",
                line.number,
                "selects behavior by scenario id".to_string(),
            ));
        }
    }
}

#[doc = r" OWN-11: width/height comparisons selecting behavior. `->` and `=>`"]
#[doc = r" are removed first so return types and match arms never match."]
fn match_size_switches(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        let tokens = words(&line.code);
        let sized = tokens.iter().any(|w| *w == "width" || *w == "height");
        if !sized {
            continue;
        }
        let normalized = line.code.replace("->", "  ").replace("=>", "  ");
        let compared = ["<=", ">=", "==", "!=", "<", ">"]
            .iter()
            .any(|op| normalized.contains(op));
        if compared {
            findings.push(finding(
                file,
                "OWN-11",
                line.number,
                "selects behavior by width/height comparison".to_string(),
            ));
        }
    }
}

#[doc = r" OWN-12: library code naming a preview crate as a dependency path."]
fn match_library_preview_deps(
    file: &ScannedFile,
    preview_crates: &[String],
    findings: &mut Vec<Finding>,
) {
    for line in production_lines(file) {
        for krate in preview_crates {
            let qualified = format!("{krate}::");
            if line.code.contains(&qualified) {
                findings.push(finding(
                    file,
                    "OWN-12",
                    line.number,
                    format!("library code depends on preview crate {krate}"),
                ));
                break;
            }
        }
    }
}

#[doc = r" OWN-13: library string literals referencing `baselines/` frame stores."]
fn match_library_stored_frames(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for span in &file.strings {
        if span.text.contains("baselines/") {
            findings.push(finding(
                file,
                "OWN-13",
                span.start_line,
                "library code references stored expected frames".to_string(),
            ));
        }
    }
}

#[doc = r" OWN-14: `.expect(`/`expect!` in production sources. The `#[expect]`"]
#[doc = r" lint attribute and `expect_used` never match: only real calls count."]
fn match_expects(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        for (offset, _) in line.code.match_indices("expect") {
            let after = &line.code[offset + "expect".len()..];
            let before = line.code[..offset].trim_end();
            let method_call = after.starts_with('(') && before.ends_with('.');
            let macro_call = after.starts_with('!');
            if method_call || macro_call {
                findings.push(finding(
                    file,
                    "OWN-14",
                    line.number,
                    "names expect in production code".to_string(),
                ));
                break;
            }
        }
    }
}

#[doc = r" A paint wrapper: a production `fn` taking `&mut Ui` (or alias) that"]
#[doc = r" contains an OWN-02/OWN-03 hit."]
#[derive(Debug, Clone)]
pub struct Wrapper {
    pub member: String,
    pub file: String,
    pub name: String,
}

#[doc = r" Collect paint wrappers from OWN-02/OWN-03 findings."]
pub fn collect_wrappers(
    files: &BTreeMap<String, ScannedFile>,
    findings: &[Finding],
) -> Vec<Wrapper> {
    let mut wrappers = Vec::new();
    for finding in findings {
        if finding.rule != "OWN-02" && finding.rule != "OWN-03" {
            continue;
        }
        let Some(symbol) = &finding.symbol else {
            continue;
        };
        let Some(file) = files.get(&finding.file) else {
            continue;
        };
        if !takes_ui_param(file, symbol) {
            continue;
        }
        if wrappers
            .iter()
            .any(|w: &Wrapper| w.member == file.member && w.name == *symbol)
        {
            continue;
        }
        wrappers.push(Wrapper {
            member: file.member.clone(),
            file: file.path.clone(),
            name: symbol.clone(),
        });
    }
    wrappers
}

#[doc = r" True when the `fn` signature mentions `&mut Ui` (or a file-local Ui"]
#[doc = r" alias) before its opening brace."]
fn takes_ui_param(file: &ScannedFile, name: &str) -> bool {
    let Some((_, range)) = file.functions.iter().find(|(n, _)| n == name) else {
        return false;
    };
    let mut signature = String::new();
    for line in &file.lines {
        if line.number < range.start || line.number > range.end {
            continue;
        }
        signature.push_str(&line.code);
        if line.code.contains('{') {
            break;
        }
    }
    signature.contains("&mut Ui")
        || signature.contains("&Ui")
        || file.aliases.keys().any(|alias| {
            file.is_ui_ident(alias)
                && (signature.contains(&format!("&mut {alias}"))
                    || signature.contains(&format!("&{alias}")))
        })
}

#[doc = r" OWN-04: call sites of paint wrappers elsewhere in the same crate."]
#[doc = r" Definitions, method calls (`.name(`), and macro calls (`name!`) never"]
#[doc = r" match; qualified `path::name(` calls do."]
pub fn match_wrapper_calls(
    files: &BTreeMap<String, ScannedFile>,
    wrappers: &[Wrapper],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for wrapper in wrappers {
        for file in files.values().filter(|f| f.member == wrapper.member) {
            for line in production_lines(file) {
                if is_wrapper_definition(&line.code, &wrapper.name) {
                    continue;
                }
                if calls_wrapper(&line.code, &wrapper.name) {
                    findings.push(finding(
                        file,
                        "OWN-04",
                        line.number,
                        format!("calls paint wrapper {}", wrapper.name),
                    ));
                }
            }
        }
    }
    findings
}

fn is_wrapper_definition(code: &str, name: &str) -> bool {
    let tokens = words(code);
    tokens
        .windows(2)
        .any(|pair| pair[0] == "fn" && pair[1] == name)
}

fn calls_wrapper(code: &str, name: &str) -> bool {
    let mut search = 0;
    while let Some(offset) = code[search..].find(name) {
        let at = search + offset;
        let before = &code[..at];
        let after = &code[at + name.len()..];
        search = at + name.len();
        let before_ok = before
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
        let after_ok = after.starts_with('(');
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::scan::scan_file;
    use super::*;

    fn preview(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ui_paint_needs_a_ui_receiver() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use termrock::Ui;\nfn a(ui: &mut Ui) {\n    ui.fill(area, s);\n    style.fill(x);\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        let paint: Vec<&Finding> = hits.iter().filter(|f| f.rule == "OWN-02").collect();
        assert_eq!(paint.len(), 1);
        assert_eq!(paint[0].line, 3);
    }

    #[test]
    fn aliased_receivers_flag_paint_calls() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use termrock::author::Ui as Canvas;\nfn a(canvas: &mut Canvas) {\n    canvas.fill(area, s);\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        let paint: Vec<&Finding> = hits.iter().filter(|f| f.rule == "OWN-02").collect();
        assert_eq!(paint.len(), 1);
        assert_eq!(paint[0].line, 3);
    }

    #[test]
    fn buffer_imports_resolve_through_aliases() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use ratatui::buffer::Buffer as Buf;\nfn a(b: &mut Buf) {\n    b.set_string(0, 0, \"x\", s);\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        assert!(hits.iter().any(|f| f.rule == "OWN-01"));
        assert!(hits.iter().any(|f| f.rule == "OWN-02"));
    }

    #[test]
    fn set_stringn_counts_as_paint_call() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use ratatui::buffer::Buffer;\nfn a(buf: &mut Buffer) {\n    buf.set_stringn(0, 0, \"x\", 1, s);\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        let paint: Vec<&Finding> = hits.iter().filter(|f| f.rule == "OWN-02").collect();
        assert_eq!(paint.len(), 1);
        assert_eq!(paint[0].line, 3);
    }

    #[test]
    fn local_cell_models_are_not_raw_buffers() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use crate::widgets::Cell;\nfn a(c: &Cell) {}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        assert!(!hits.iter().any(|f| f.rule == "OWN-01"));
    }

    #[test]
    fn raw_escapes_ignore_iterator_peek() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use termrock::Ui;\nfn a(ui: &mut Ui, it: &mut Iter) {\n    let (b, _) = ui.raw();\n    let _ = it.peek();\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        let escapes: Vec<&Finding> = hits.iter().filter(|f| f.rule == "OWN-03").collect();
        assert_eq!(escapes.len(), 1);
    }

    #[test]
    fn wrappers_resolve_to_call_sites() {
        let mut files = BTreeMap::new();
        files.insert(
            "crates/demo/src/lib.rs".to_string(),
            scan_file(
                "crates/demo/src/lib.rs",
                "crates/demo",
                "use termrock::Ui;\nfn paint_banner(ui: &mut Ui) {\n    ui.fill(a, s);\n}\nfn show(ui: &mut Ui) {\n    paint_banner(ui);\n}\n",
            ),
        );
        let mut hits = Vec::new();
        for file in files.values() {
            hits.extend(match_file(file, &preview(&["demo"])));
        }
        let wrappers = collect_wrappers(&files, &hits);
        assert_eq!(wrappers.len(), 1);
        assert_eq!(wrappers[0].name, "paint_banner");
        let calls = match_wrapper_calls(&files, &wrappers);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].line, 6);
    }

    #[test]
    fn macros_resolve_bodies_and_call_sites() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "macro_rules! banner {\n    ($ui:expr) => { $ui.fill(a, s); };\n}\nfn show(ui: &mut Ui) {\n    banner!(ui);\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        assert_eq!(
            hits.iter().filter(|f| f.rule == "OWN-05").count(),
            2,
            "definition plus call site"
        );

        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "macro_rules! zero {\n    ($v:expr) => { $v.fill(0); };\n}\nfn show() {\n    zero!(buf);\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        assert!(!hits.iter().any(|f| f.rule == "OWN-05"));
    }

    #[test]
    fn single_box_glyphs_are_not_screen_literals() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "const V: &str = \"\u{2502}\";\nconst SCREEN: &str = \"\u{250c}\u{2500}\u{2510}\\n\u{2502}x\u{2502}\\n\u{2514}\u{2500}\u{2518}\";\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        assert!(!hits.iter().any(|f| f.rule == "OWN-07"));
    }

    #[test]
    fn expect_matcher_skips_attributes() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "#[expect(clippy::expect_used, reason = \"x\")]\nfn a(o: Option<u32>) -> u32 {\n    o.expect(\"present\")\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        let expects: Vec<&Finding> = hits.iter().filter(|f| f.rule == "OWN-14").collect();
        assert_eq!(expects.len(), 1);
        assert_eq!(expects[0].line, 3);
    }

    #[test]
    fn return_types_are_not_size_switches() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "fn width(&self) -> u16 {\n    self.width\n}\nfn a(w: u16) {\n    if w < 70 {\n    }\n}\n",
        );
        let hits = match_file(&file, &preview(&["demo"]));
        assert!(!hits.iter().any(|f| f.rule == "OWN-11"));
    }
}
