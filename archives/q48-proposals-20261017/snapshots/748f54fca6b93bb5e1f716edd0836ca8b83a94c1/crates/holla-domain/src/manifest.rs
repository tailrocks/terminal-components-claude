//! Native project-task adapters (HP07): bounded parsers over manifest text
//! and tool output. Discovery never runs a recipe; each adapter states its
//! defining file, runner, effective cwd and exact argv, and reports its own
//! failure instead of inventing tasks.

use std::collections::BTreeSet;

/// Visible cap on dynamically discovered tasks per source (legacy 30).
pub const TASK_CAP: usize = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredTask {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Exact argument vector, program first.
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovery {
    pub source: &'static str,
    /// Defining file (or the tool's own listing).
    pub defined_by: String,
    pub runner: String,
    pub tasks: Vec<DiscoveredTask>,
    /// Total before the visible cap.
    pub total: usize,
    /// Why nothing (or less) was found; never a task.
    pub diagnostic: Option<String>,
}

impl Discovery {
    pub fn capped(&self) -> bool {
        self.total > self.tasks.len()
    }
    pub fn title(&self, label: &str) -> String {
        if self.capped() {
            format!("{label} ({} of {})", self.tasks.len(), self.total)
        } else {
            label.to_owned()
        }
    }
}

// ------------------------------------------------------------ mini JSON

/// A minimal JSON value: enough for `package.json` scripts and Taskfile's
/// `--list --json`. Strings, objects, arrays, numbers, booleans, null.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    s: &'a [u8],
    i: usize,
}

impl JsonParser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] as char).is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }
    fn expect(&mut self, b: u8) -> Result<(), String> {
        self.ws();
        if self.peek() == Some(b) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", b as char, self.i))
        }
    }
    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.peek() {
            None => Err("unexpected end".into()),
            Some(b'{') => {
                self.i += 1;
                let mut kv = vec![];
                self.ws();
                if self.peek() == Some(b'}') {
                    self.i += 1;
                    return Ok(Json::Obj(kv));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.expect(b':')?;
                    let v = self.value()?;
                    kv.push((k, v));
                    self.ws();
                    match self.peek() {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Json::Obj(kv));
                        }
                        _ => return Err(format!("expected ',' or '}}' at byte {}", self.i)),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut v = vec![];
                self.ws();
                if self.peek() == Some(b']') {
                    self.i += 1;
                    return Ok(Json::Arr(v));
                }
                loop {
                    v.push(self.value()?);
                    self.ws();
                    match self.peek() {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Json::Arr(v));
                        }
                        _ => return Err(format!("expected ',' or ']' at byte {}", self.i)),
                    }
                }
            }
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Json::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Json::Bool(false))
            }
            Some(b'n') if self.s[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(Json::Null)
            }
            Some(_) => {
                let start = self.i;
                while self.i < self.s.len()
                    && matches!(
                        self.s[self.i],
                        b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
                    )
                {
                    self.i += 1;
                }
                std::str::from_utf8(&self.s[start..self.i])
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .map(Json::Num)
                    .ok_or_else(|| format!("bad number at byte {start}"))
            }
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.ws();
        if self.peek() != Some(b'"') {
            return Err(format!("expected string at byte {}", self.i));
        }
        self.i += 1;
        let mut out = String::new();
        loop {
            let Some(b) = self.peek() else {
                return Err("unterminated string".into());
            };
            self.i += 1;
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(e) = self.peek() else {
                        return Err("bad escape".into());
                    };
                    self.i += 1;
                    match e {
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'u' => {
                            let hex =
                                std::str::from_utf8(self.s.get(self.i..self.i + 4).unwrap_or(b""))
                                    .map_err(|_| "bad \\u".to_owned())?;
                            let cp =
                                u32::from_str_radix(hex, 16).map_err(|_| "bad \\u".to_owned())?;
                            self.i += 4;
                            out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                        }
                        _ => out.push(e as char),
                    }
                }
                _ => {
                    // collect a UTF-8 sequence
                    let start = self.i - 1;
                    let mut end = self.i;
                    while end < self.s.len() && (self.s[end] & 0b1100_0000) == 0b1000_0000 {
                        end += 1;
                    }
                    out.push_str(std::str::from_utf8(&self.s[start..end]).unwrap_or("\u{fffd}"));
                    self.i = end;
                }
            }
        }
    }
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    let mut p = JsonParser {
        s: text.as_bytes(),
        i: 0,
    };
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("trailing data at byte {}", p.i));
    }
    Ok(v)
}

// ------------------------------------------------------------ node

/// Lockfile precedence: pnpm, yarn, bun, then npm (OP21).
pub fn node_runner(present: &[&str]) -> &'static str {
    if present.contains(&"pnpm-lock.yaml") {
        "pnpm"
    } else if present.contains(&"yarn.lock") {
        "yarn"
    } else if present.contains(&"bun.lock") || present.contains(&"bun.lockb") {
        "bun"
    } else {
        "npm"
    }
}

/// Discover package.json scripts (OP20/OP22): string-valued keys of the
/// `scripts` object, sorted, first 30 visible. Runner availability is not
/// checked. Invalid or missing input yields no group with a diagnostic.
pub fn node_scripts(cwd: &str, package_json: Option<&str>, lockfiles: &[&str]) -> Discovery {
    let runner = node_runner(lockfiles);
    let file = format!("{cwd}/package.json");
    let mut d = Discovery {
        source: "node",
        defined_by: file.clone(),
        runner: runner.into(),
        tasks: vec![],
        total: 0,
        diagnostic: None,
    };
    let Some(text) = package_json else {
        d.diagnostic = Some("no package.json here".into());
        return d;
    };
    let json = match parse_json(text) {
        Ok(j) => j,
        Err(e) => {
            d.diagnostic = Some(format!("package.json unreadable: {e}"));
            return d;
        }
    };
    let Some(Json::Obj(scripts)) = json.get("scripts") else {
        d.diagnostic = Some("package.json has no scripts object".into());
        return d;
    };
    let mut names: Vec<(String, String)> = scripts
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
        .collect();
    names.sort();
    names.dedup_by(|a, b| a.0 == b.0);
    d.total = names.len();
    if names.is_empty() {
        d.diagnostic = Some("scripts is empty".into());
    }
    d.tasks = names
        .into_iter()
        .take(TASK_CAP)
        .map(|(name, body)| DiscoveredTask {
            id: format!("node.script.{name}"),
            argv: vec![runner.into(), "run".into(), name.clone()],
            description: body,
            name,
        })
        .collect();
    d
}

// ------------------------------------------------------------ just

/// `just --summary` output: whitespace-separated names, sorted, deduped
/// (OP23/OP24). A failed command hides the group.
pub fn just_recipes(cwd: &str, file: &str, summary: Result<&str, &str>) -> Discovery {
    let mut d = Discovery {
        source: "just",
        defined_by: format!("{cwd}/{file}"),
        runner: "just".into(),
        tasks: vec![],
        total: 0,
        diagnostic: None,
    };
    let text = match summary {
        Ok(t) => t,
        Err(e) => {
            d.diagnostic = Some(format!("just --summary failed: {e}"));
            return d;
        }
    };
    let names: BTreeSet<&str> = text.split_whitespace().collect();
    d.total = names.len();
    if names.is_empty() {
        d.diagnostic = Some("just --summary listed no recipes".into());
    }
    d.tasks = names
        .into_iter()
        .take(TASK_CAP)
        .map(|n| DiscoveredTask {
            id: format!("just.recipe.{n}"),
            name: n.into(),
            description: String::new(),
            argv: vec!["just".into(), n.into()],
        })
        .collect();
    d
}

// ------------------------------------------------------------ make

fn make_target_ok(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Conservative Makefile parser (OP25): column-zero declarations only,
/// multi-target lines accepted, comments/whitespace/`:=` assignments
/// skipped, names limited to ASCII alphanumerics, `_` and `-`, first
/// declaration order kept. Make is never invoked during discovery.
pub fn make_targets(text: &str) -> (Vec<String>, Vec<String>) {
    let mut out: Vec<String> = vec![];
    let mut rejected = vec![];
    for line in text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') || line.starts_with('#') {
            continue;
        }
        let Some(colon) = line.find(':') else {
            continue;
        };
        let head = &line[..colon];
        if line[colon..].starts_with(":=") || head.contains('=') {
            continue;
        }
        for name in head.split_whitespace() {
            if name.contains('%')
                || name.contains('$')
                || name.contains('/')
                || !make_target_ok(name)
            {
                rejected.push(name.to_owned());
                continue;
            }
            if !out.iter().any(|o| o == name) {
                out.push(name.to_owned());
            }
        }
    }
    (out, rejected)
}

pub fn make_discovery(cwd: &str, makefile: Option<&str>) -> Discovery {
    let mut d = Discovery {
        source: "make",
        defined_by: format!("{cwd}/Makefile"),
        runner: "make".into(),
        tasks: vec![],
        total: 0,
        diagnostic: None,
    };
    let Some(text) = makefile else {
        d.diagnostic = Some("no Makefile here (exact name)".into());
        return d;
    };
    let (targets, rejected) = make_targets(text);
    d.total = targets.len();
    if !rejected.is_empty() {
        d.diagnostic = Some(format!(
            "{} unsupported target {} skipped ({})",
            rejected.len(),
            if rejected.len() == 1 {
                "syntax"
            } else {
                "syntaxes"
            },
            rejected.join(", ")
        ));
    }
    if targets.is_empty() && d.diagnostic.is_none() {
        d.diagnostic = Some("Makefile declares no plain targets".into());
    }
    d.tasks = targets
        .into_iter()
        .take(TASK_CAP)
        .map(|n| DiscoveredTask {
            id: format!("make.target.{n}"),
            argv: vec!["make".into(), n.clone()],
            description: String::new(),
            name: n,
        })
        .collect();
    d
}

// ------------------------------------------------------------ taskfile

/// `task --list --json` (OP27/OP28): object with a `tasks` array of objects
/// with string `name`; empty names skipped, sorted, deduped; descriptions
/// not retained. Bad schema hides the group.
pub fn taskfile_tasks(cwd: &str, file: &str, listing: Result<&str, &str>) -> Discovery {
    let mut d = Discovery {
        source: "taskfile",
        defined_by: format!("{cwd}/{file}"),
        runner: "task".into(),
        tasks: vec![],
        total: 0,
        diagnostic: None,
    };
    let text = match listing {
        Ok(t) => t,
        Err(e) => {
            d.diagnostic = Some(format!("task --list --json failed: {e}"));
            return d;
        }
    };
    let json = match parse_json(text) {
        Ok(j) => j,
        Err(e) => {
            d.diagnostic = Some(format!("task --list --json: {e}"));
            return d;
        }
    };
    let Some(Json::Arr(tasks)) = json.get("tasks") else {
        d.diagnostic = Some("task --list --json: no tasks array".into());
        return d;
    };
    let mut names: Vec<String> = tasks
        .iter()
        .filter_map(|t| t.get("name").and_then(Json::as_str))
        .filter(|n| !n.trim().is_empty())
        .map(|n| n.to_owned())
        .collect();
    names.sort();
    names.dedup();
    d.total = names.len();
    if names.is_empty() {
        d.diagnostic = Some("task --list --json listed no tasks".into());
    }
    d.tasks = names
        .into_iter()
        .take(TASK_CAP)
        .map(|n| DiscoveredTask {
            id: format!("taskfile.task.{n}"),
            argv: vec!["task".into(), n.clone()],
            description: String::new(),
            name: n,
        })
        .collect();
    d
}

// ------------------------------------------------------------ mise

/// `mise tasks ls --no-header` (OP29): nonblank lines, first field is the
/// name, the rest a description with a leading `#` stripped; output order
/// kept, no cap, no dedup. A nonzero exit is a failure, never a listing.
pub fn mise_tasks(cwd: &str, output: Result<&str, &str>) -> Discovery {
    let mut d = Discovery {
        source: "mise",
        defined_by: format!("{cwd}/mise.toml"),
        runner: "mise".into(),
        tasks: vec![],
        total: 0,
        diagnostic: None,
    };
    let text = match output {
        Ok(t) => t,
        Err(e) => {
            d.diagnostic = Some(format!("mise tasks ls failed: {e}"));
            return d;
        }
    };
    for l in text.lines() {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        let (name, rest) = l.split_once(char::is_whitespace).unwrap_or((l, ""));
        let desc = rest.trim().trim_start_matches('#').trim().to_owned();
        d.tasks.push(DiscoveredTask {
            id: format!("mise.task.{name}"),
            name: name.into(),
            description: if desc.is_empty() {
                "Run mise task".into()
            } else {
                desc
            },
            argv: vec!["mise".into(), "run".into(), name.into()],
        });
    }
    d.total = d.tasks.len();
    if d.tasks.is_empty() {
        d.diagnostic = Some("mise tasks ls listed nothing".into());
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_scripts_sort_keep_strings_and_pick_the_runner() {
        let pj = r#"{"name":"x","scripts":{"test":"vitest","build":"vite build","weird":{"not":"a string"},"dev":"vite --host"}}"#;
        let d = node_scripts("/p", Some(pj), &["yarn.lock", "pnpm-lock.yaml"]);
        assert_eq!(d.runner, "pnpm", "pnpm beats yarn");
        let names: Vec<&str> = d.tasks.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["build", "dev", "test"]);
        assert_eq!(d.tasks[0].argv, vec!["pnpm", "run", "build"]);
        assert_eq!(d.tasks[0].id, "node.script.build");
        assert_eq!(node_runner(&["bun.lockb"]), "bun");
        assert_eq!(node_runner(&[]), "npm");
        assert!(
            node_scripts("/p", Some("{not json"), &[])
                .diagnostic
                .unwrap()
                .contains("unreadable")
        );
        assert!(
            node_scripts("/p", Some(r#"{"scripts":{}}"#), &[])
                .diagnostic
                .unwrap()
                .contains("empty")
        );
        assert!(node_scripts("/p", None, &[]).tasks.is_empty());
        let many: Vec<String> = (0..31).map(|i| format!("\"s{i:02}\":\"echo\"")).collect();
        let d = node_scripts(
            "/p",
            Some(&format!("{{\"scripts\":{{{}}}}}", many.join(","))),
            &[],
        );
        assert_eq!(d.tasks.len(), 30);
        assert_eq!(d.total, 31);
        assert_eq!(d.title("Node scripts"), "Node scripts (30 of 31)");
        // metacharacters stay one argument, never shell text
        let d = node_scripts("/p", Some(r#"{"scripts":{"it's; rm":"x","ünï":"y"}}"#), &[]);
        assert_eq!(d.tasks[0].argv[2], "it's; rm");
        assert_eq!(d.tasks[1].name, "ünï");
    }

    #[test]
    fn just_make_taskfile_and_mise_follow_their_legacy_rules() {
        let j = just_recipes("/p", "justfile", Ok("build test  build\ndeploy"));
        assert_eq!(
            j.tasks.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            vec!["build", "deploy", "test"]
        );
        assert_eq!(j.tasks[0].argv, vec!["just", "build"]);
        assert!(
            just_recipes("/p", "Justfile", Err("exit 1"))
                .diagnostic
                .unwrap()
                .contains("failed")
        );
        assert!(just_recipes("/p", ".justfile", Ok("")).tasks.is_empty());

        let mk = "# comment\nVAR := 1\n\ttab: no\nall build: dep\n.PHONY: all\n%.o: %.c\n$(X): y\nclean:\nbuild:\nsrc/dir: x\n";
        let (t, rejected) = make_targets(mk);
        assert_eq!(t, vec!["all", "build", "clean"]);
        assert_eq!(rejected, vec![".PHONY", "%.o", "$(X)", "src/dir"]);
        let d = make_discovery("/p", Some(mk));
        assert_eq!(d.tasks[0].argv, vec!["make", "all"]);
        assert!(d.diagnostic.unwrap().contains("unsupported"));
        assert!(
            make_discovery("/p", None)
                .diagnostic
                .unwrap()
                .contains("exact name")
        );

        let tf = taskfile_tasks(
            "/p",
            "Taskfile.yml",
            Ok(
                r#"{"tasks":[{"name":"lint","desc":"x"},{"name":""},{"name":"build"},{"name":"build"}]}"#,
            ),
        );
        assert_eq!(
            tf.tasks.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            vec!["build", "lint"]
        );
        assert_eq!(tf.tasks[1].argv, vec!["task", "lint"]);
        assert!(
            tf.tasks[1].description.is_empty(),
            "descriptions are not retained"
        );
        assert!(
            taskfile_tasks("/p", "Taskfile.yaml", Ok(r#"{"tasks":"no"}"#))
                .diagnostic
                .is_some()
        );
        assert!(
            taskfile_tasks("/p", "Taskfile.yaml", Ok("["))
                .diagnostic
                .is_some()
        );

        let m = mise_tasks("/p", Ok("build  # compile everything\ntest\nzzz first\n\n"));
        assert_eq!(m.tasks.len(), 3);
        assert_eq!(m.tasks[0].description, "compile everything");
        assert_eq!(m.tasks[1].description, "Run mise task");
        assert_eq!(m.tasks[2].name, "zzz", "output order is kept, not sorted");
        assert_eq!(m.tasks[0].argv, vec!["mise", "run", "build"]);
        assert!(
            mise_tasks("/p", Err("mise: not trusted")).tasks.is_empty(),
            "stdout after failure is never authoritative"
        );
    }

    #[test]
    fn json_parser_handles_nesting_escapes_and_errors() {
        let j = parse_json(r#"{"a":[1,2.5,{"b":"c\"é"}],"t":true,"n":null}"#).unwrap();
        assert_eq!(j.get("t"), Some(&Json::Bool(true)));
        assert!(parse_json("{").is_err());
        assert!(parse_json("{} x").is_err());
        assert!(parse_json(r#"{"a":}"#).is_err());
    }
}
