//! Custom actions and trust (HP17). Global `actions.toml` and the project's
//! `.holla.toml` are parsed with a bounded TOML subset into typed argv
//! actions with source-indexed diagnostics. Trust binds to the whole file's
//! SHA-256, its path and the effective cwd; a store is versioned, tolerant
//! of corruption (untrusted) and refuses to launch when it cannot save.

use crate::domain::digest::sha256_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Global,
    Project,
}

impl Origin {
    pub fn label(self) -> &'static str {
        match self {
            Origin::Global => "global",
            Origin::Project => "project",
        }
    }
    pub fn default_group(self) -> &'static str {
        match self {
            Origin::Global => "Custom",
            Origin::Project => "Current folder",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Danger {
    Safe,
    Mutating,
    Destructive,
}

impl Danger {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "safe" => Some(Danger::Safe),
            "mutating" => Some(Danger::Mutating),
            "destructive" => Some(Danger::Destructive),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Danger::Safe => "safe",
            Danger::Mutating => "mutating",
            Danger::Destructive => "destructive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomAction {
    pub id: String,
    pub label: String,
    /// Exact argument vector, program first. Never a shell string.
    pub argv: Vec<String>,
    pub danger: Danger,
    pub confirm: bool,
    pub description: String,
    pub keywords: Vec<String>,
    pub group: String,
    pub origin: Origin,
    /// Zero-based `[[action]]` index in its file.
    pub index: usize,
}

/// A parse or validation problem: file plus optional entry index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub index: Option<usize>,
    pub message: String,
}

impl Diagnostic {
    pub fn text(&self) -> String {
        match self.index {
            Some(i) => format!("{} action[{i}]: {}", self.path, self.message),
            None => format!("{}: {}", self.path, self.message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomConfig {
    pub path: String,
    pub origin: Origin,
    pub text: String,
    pub digest: String,
    pub actions: Vec<CustomAction>,
    pub diagnostics: Vec<Diagnostic>,
}

// ------------------------------------------------------------ TOML subset

#[derive(Debug, Clone, PartialEq)]
enum Val {
    Str(String),
    Bool(bool),
    Arr(Vec<String>),
    Other,
}

fn parse_string(s: &str) -> Result<(String, &str), String> {
    let s = s.trim_start();
    let quote = s.chars().next().ok_or("expected a string")?;
    if quote != '"' && quote != '\'' {
        return Err("expected a quoted string".into());
    }
    let body = &s[1..];
    if quote == '\'' {
        let end = body.find('\'').ok_or("unterminated string")?;
        return Ok((body[..end].to_owned(), &body[end + 1..]));
    }
    let mut out = String::new();
    let mut chars = body.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Ok((out, &body[i + 1..])),
            '\\' => match chars.next() {
                Some((_, 'n')) => out.push('\n'),
                Some((_, 't')) => out.push('\t'),
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, other)) => {
                    out.push('\\');
                    out.push(other);
                }
                None => return Err("unterminated escape".into()),
            },
            c => out.push(c),
        }
    }
    Err("unterminated string".into())
}

fn parse_value(s: &str) -> Result<Val, String> {
    let s = s.trim();
    if s == "true" {
        return Ok(Val::Bool(true));
    }
    if s == "false" {
        return Ok(Val::Bool(false));
    }
    if let Some(inner) = s.strip_prefix('[') {
        let inner = inner.strip_suffix(']').ok_or("unterminated array")?;
        let mut items = vec![];
        let mut rest = inner.trim();
        while !rest.is_empty() {
            let (item, after) = parse_string(rest)?;
            items.push(item);
            rest = after.trim_start();
            if let Some(r) = rest.strip_prefix(',') {
                rest = r.trim_start();
            } else if !rest.is_empty() {
                return Err("expected ',' in array".into());
            }
        }
        return Ok(Val::Arr(items));
    }
    if s.starts_with('"') || s.starts_with('\'') {
        let (v, rest) = parse_string(s)?;
        if !rest.trim().is_empty() && !rest.trim().starts_with('#') {
            return Err("trailing data after string".into());
        }
        return Ok(Val::Str(v));
    }
    Ok(Val::Other)
}

/// Strip a trailing `# comment` outside quotes.
fn strip_comment(line: &str) -> &str {
    let mut in_str: Option<char> = None;
    for (i, c) in line.char_indices() {
        match in_str {
            Some(q) if c == q => in_str = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => in_str = Some(c),
            None if c == '#' => return &line[..i],
            None => {}
        }
    }
    line
}

const REQUIRED: [&str; 4] = ["id", "label", "command", "danger"];

fn id_ok(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

/// Parse one configuration file. Malformed entries are skipped with an
/// indexed diagnostic while valid siblings survive; a whole-file syntax
/// error reports the file without an entry.
pub fn parse_config(
    path: &str,
    origin: Origin,
    text: &str,
    builtin_ids: &[&str],
    reserved: &[String],
) -> CustomConfig {
    let mut cfg = CustomConfig {
        path: path.into(),
        origin,
        text: text.into(),
        digest: sha256_hex(text.as_bytes()),
        actions: vec![],
        diagnostics: vec![],
    };
    let mut entries: Vec<Vec<(String, Val, usize)>> = vec![];
    let mut in_action = false;
    let mut saw_action_key = false;
    for (ln, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[action]]" {
            entries.push(vec![]);
            in_action = true;
            saw_action_key = true;
            continue;
        }
        if line.starts_with("[[") || line.starts_with('[') {
            in_action = false;
            if line.starts_with("[action") && !line.starts_with("[[") {
                cfg.diagnostics.push(Diagnostic {
                    path: path.into(),
                    index: None,
                    message: format!(
                        "line {}: `action` must be an array of tables ([[action]])",
                        ln + 1
                    ),
                });
            }
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            cfg.diagnostics.push(Diagnostic {
                path: path.into(),
                index: None,
                message: format!("line {}: expected `key = value`", ln + 1),
            });
            return cfg;
        };
        let key = k.trim().to_owned();
        let val = match parse_value(v) {
            Ok(val) => val,
            Err(e) => {
                cfg.diagnostics.push(Diagnostic {
                    path: path.into(),
                    index: None,
                    message: format!("line {}: {e}", ln + 1),
                });
                return cfg;
            }
        };
        if key == "action" && !in_action {
            cfg.diagnostics.push(Diagnostic {
                path: path.into(),
                index: None,
                message: format!("line {}: `action` must be an array of tables", ln + 1),
            });
            return cfg;
        }
        if in_action && let Some(e) = entries.last_mut() {
            e.push((key, val, ln + 1));
        }
    }
    if !saw_action_key && !text.trim().is_empty() && cfg.diagnostics.is_empty() {
        // a file without [[action]] contributes nothing, silently
    }
    let mut ids_here: Vec<String> = vec![];
    for (index, e) in entries.iter().enumerate() {
        let get = |k: &str| {
            e.iter()
                .find(|(key, _, _)| key == k)
                .map(|(_, v, _)| v.clone())
        };
        let mut problems = vec![];
        for r in REQUIRED {
            if get(r).is_none() {
                problems.push(format!("missing `{r}`"));
            }
        }
        let id = match get("id") {
            Some(Val::Str(s)) => s,
            Some(_) => {
                problems.push("`id` must be a string".into());
                String::new()
            }
            None => String::new(),
        };
        if !id.is_empty() && !id_ok(&id) {
            problems.push(format!(
                "invalid id {id:?}: lowercase ASCII letters, digits, `.`, `_`, `-` only"
            ));
        }
        let label = match get("label") {
            Some(Val::Str(s)) if !s.trim().is_empty() => s,
            Some(Val::Str(_)) => {
                problems.push("`label` is blank".into());
                String::new()
            }
            Some(_) => {
                problems.push("`label` must be a string".into());
                String::new()
            }
            None => String::new(),
        };
        let argv = match get("command") {
            Some(Val::Arr(a)) if !a.is_empty() && !a[0].trim().is_empty() => a,
            Some(Val::Arr(_)) => {
                problems.push("`command` needs a nonempty program".into());
                vec![]
            }
            Some(Val::Str(_)) => {
                problems.push("`command` must be an argv array, not a shell string".into());
                vec![]
            }
            Some(_) => {
                problems.push("`command` must be an array of strings".into());
                vec![]
            }
            None => vec![],
        };
        let danger = match get("danger") {
            Some(Val::Str(s)) => match Danger::parse(&s) {
                Some(d) => Some(d),
                None => {
                    problems.push(format!(
                        "unknown danger {s:?}: safe, mutating or destructive (lowercase)"
                    ));
                    None
                }
            },
            Some(_) => {
                problems.push("`danger` must be a string".into());
                None
            }
            None => None,
        };
        let confirm = match get("confirm") {
            Some(Val::Bool(b)) => b,
            Some(_) => {
                problems.push("`confirm` must be true or false".into());
                false
            }
            None => false,
        };
        let description = match get("description") {
            Some(Val::Str(s)) => s,
            Some(_) => {
                problems.push("`description` must be a string".into());
                String::new()
            }
            None => String::new(),
        };
        let keywords = match get("keywords") {
            Some(Val::Arr(a)) => a,
            Some(_) => {
                problems.push("`keywords` must be an array of strings".into());
                vec![]
            }
            None => vec![],
        };
        let group = match get("group") {
            Some(Val::Str(s)) if !s.trim().is_empty() => s,
            Some(Val::Str(_)) => {
                problems.push("`group` is blank".into());
                String::new()
            }
            Some(_) => {
                problems.push("`group` must be a string".into());
                String::new()
            }
            None => origin.default_group().to_owned(),
        };
        if !id.is_empty() && problems.is_empty() {
            if builtin_ids.contains(&id.as_str()) {
                problems.push(format!("id {id:?} collides with a built-in action"));
            } else if reserved.iter().any(|r| r == &id) {
                problems.push(format!(
                    "id {id:?} is already defined by the global configuration"
                ));
            } else if ids_here.contains(&id) {
                problems.push(format!("duplicate id {id:?} in this file"));
            }
        }
        if !problems.is_empty() {
            cfg.diagnostics.push(Diagnostic {
                path: path.into(),
                index: Some(index),
                message: problems.join(" · "),
            });
            continue;
        }
        ids_here.push(id.clone());
        cfg.actions.push(CustomAction {
            id,
            label,
            argv,
            danger: danger.unwrap_or(Danger::Safe),
            confirm,
            description,
            keywords,
            group,
            origin,
            index,
        });
    }
    cfg
}

// ------------------------------------------------------------ trust

/// One accepted review: the whole-file digest, the reviewed path and the
/// effective cwd. Legacy digest-only entries carry no path and require a
/// renewed review before broader binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustEntry {
    pub digest: String,
    pub path: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrustStore {
    pub entries: Vec<TrustEntry>,
    /// The persisted store could not be read: everything is untrusted and
    /// the reason is shown.
    pub corrupt: Option<String>,
    /// Saving fails: no launch may proceed on a new approval.
    pub save_failure: Option<String>,
    pub saves: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStatus {
    /// Digest, path and cwd match a reviewed entry.
    Trusted,
    /// The bytes were once accepted at another path: review again.
    Moved,
    /// A legacy digest-only record: review again to bind path and cwd.
    LegacyDigestOnly,
    Untrusted,
}

impl TrustStore {
    pub fn status(&self, digest: &str, path: &str, cwd: &str) -> TrustStatus {
        let mut best = TrustStatus::Untrusted;
        for e in &self.entries {
            if e.digest != digest {
                continue;
            }
            match (&e.path, &e.cwd) {
                (Some(p), Some(c)) if p == path && c == cwd => return TrustStatus::Trusted,
                (Some(_), _) => best = TrustStatus::Moved,
                (None, _) => {
                    if best == TrustStatus::Untrusted {
                        best = TrustStatus::LegacyDigestOnly;
                    }
                }
            }
        }
        best
    }

    /// Record an approval. Returns the persisted text, or the save error;
    /// on error nothing is remembered.
    pub fn approve(&mut self, digest: &str, path: &str, cwd: &str) -> Result<String, String> {
        if let Some(e) = &self.save_failure {
            return Err(format!("trust store not saved: {e}"));
        }
        self.entries
            .retain(|e| !(e.digest == digest && e.path.as_deref() == Some(path)));
        self.entries.push(TrustEntry {
            digest: digest.into(),
            path: Some(path.into()),
            cwd: Some(cwd.into()),
        });
        self.entries
            .sort_by(|a, b| a.digest.cmp(&b.digest).then_with(|| a.path.cmp(&b.path)));
        self.saves += 1;
        Ok(self.serialize())
    }

    pub fn revoke(&mut self, digest: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.digest != digest);
        before != self.entries.len()
    }

    /// `{"v":2,"entries":[{"digest":…,"path":…,"cwd":…}]}` (sorted).
    pub fn serialize(&self) -> String {
        let items: Vec<String> = self
            .entries
            .iter()
            .map(|e| {
                format!(
                    "{{\"digest\":\"{}\",\"path\":{},\"cwd\":{}}}",
                    e.digest,
                    json_str(e.path.as_deref()),
                    json_str(e.cwd.as_deref())
                )
            })
            .collect();
        format!("{{\"v\":2,\"entries\":[{}]}}", items.join(","))
    }

    /// Load a persisted store. v1 (`{"v":1,"hashes":[…]}`) is read as
    /// digest-only legacy evidence; unknown or corrupt input is untrusted.
    pub fn load(text: Option<&str>) -> Self {
        let Some(text) = text else {
            return Self::default();
        };
        let json = match crate::domain::manifest::parse_json(text) {
            Ok(j) => j,
            Err(e) => {
                return Self {
                    corrupt: Some(format!("trusted.json unreadable: {e}")),
                    ..Default::default()
                };
            }
        };
        use crate::domain::manifest::Json;
        let v = match json.get("v") {
            Some(Json::Num(n)) => *n as i64,
            _ => {
                return Self {
                    corrupt: Some("trusted.json has no version".into()),
                    ..Default::default()
                };
            }
        };
        let mut store = Self::default();
        match v {
            1 => {
                if let Some(Json::Arr(h)) = json.get("hashes") {
                    for d in h.iter().filter_map(Json::as_str) {
                        store.entries.push(TrustEntry {
                            digest: d.into(),
                            path: None,
                            cwd: None,
                        });
                    }
                }
            }
            2 => {
                if let Some(Json::Arr(es)) = json.get("entries") {
                    for e in es {
                        let Some(d) = e.get("digest").and_then(Json::as_str) else {
                            continue;
                        };
                        store.entries.push(TrustEntry {
                            digest: d.into(),
                            path: e.get("path").and_then(Json::as_str).map(str::to_owned),
                            cwd: e.get("cwd").and_then(Json::as_str).map(str::to_owned),
                        });
                    }
                }
            }
            other => {
                store.corrupt = Some(format!("trusted.json version {other} is unknown"));
            }
        }
        store
    }
}

fn json_str(s: Option<&str>) -> String {
    match s {
        Some(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        None => "null".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
# team actions
[[action]]
id = "deploy.preview"
label = "Deploy preview"
command = ["tools/deploy-preview.sh", "--env", "preview"]
danger = "mutating"
description = "push the current branch to the preview stack"
keywords = ["deploy", "preview"]

[[action]]
id = "wipe"
label = "Wipe caches"
command = ["sh", "-c", "rm -rf .cache && echo done"]
danger = "destructive"
confirm = true
group = "Maintenance"
"#;

    #[test]
    fn parses_the_schema_with_defaults_and_argv_only_commands() {
        let c = parse_config("/p/.holla.toml", Origin::Project, GOOD, &["git.pull"], &[]);
        assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics);
        assert_eq!(c.actions.len(), 2);
        let d = &c.actions[0];
        assert_eq!(d.argv, vec!["tools/deploy-preview.sh", "--env", "preview"]);
        assert_eq!(d.group, "Current folder");
        assert!(!d.confirm);
        assert_eq!(d.keywords, vec!["deploy", "preview"]);
        let w = &c.actions[1];
        assert_eq!(w.argv[0], "sh");
        assert_eq!(w.argv[2], "rm -rf .cache && echo done");
        assert!(w.confirm);
        assert_eq!(w.group, "Maintenance");
        assert_eq!(w.index, 1);
        assert_eq!(c.digest.len(), 64);
        let g = parse_config("/h/actions.toml", Origin::Global, GOOD, &[], &[]);
        assert_eq!(g.actions[0].group, "Custom");
    }

    #[test]
    fn diagnostics_are_indexed_and_valid_siblings_survive() {
        let text = r#"
[[action]]
id = "Bad ID"
label = "x"
command = ["a"]
danger = "safe"

[[action]]
id = "shell"
label = "y"
command = "rm -rf /"
danger = "safe"

[[action]]
id = "ok"
label = "fine"
command = ["true"]
danger = "safe"

[[action]]
id = "ok"
label = "dup"
command = ["true"]
danger = "safe"

[[action]]
id = "git.pull"
label = "collides"
command = ["true"]
danger = "SAFE"

[[action]]
id = "blank"
label = "   "
command = []
danger = "safe"
confirm = "yes"
"#;
        let c = parse_config(
            "/p/.holla.toml",
            Origin::Project,
            text,
            &["git.pull"],
            &["reserved".into()],
        );
        assert_eq!(c.actions.len(), 1);
        assert_eq!(c.actions[0].id, "ok");
        let d: Vec<String> = c.diagnostics.iter().map(Diagnostic::text).collect();
        assert!(
            d[0].starts_with("/p/.holla.toml action[0]: invalid id"),
            "{}",
            d[0]
        );
        assert!(
            d[1].contains("action[1]") && d[1].contains("argv array"),
            "{}",
            d[1]
        );
        assert!(
            d[2].contains("action[3]") && d[2].contains("duplicate"),
            "{}",
            d[2]
        );
        assert!(
            d[3].contains("action[4]") && d[3].contains("unknown danger"),
            "{}",
            d[3]
        );
        assert!(
            d[4].contains("action[5]")
                && d[4].contains("blank")
                && d[4].contains("nonempty program")
                && d[4].contains("confirm"),
            "{}",
            d[4]
        );
        // the id collision with a built-in is diagnosed only when otherwise valid
        let text =
            "[[action]]\nid = \"git.pull\"\nlabel = \"x\"\ncommand = [\"a\"]\ndanger = \"safe\"\n";
        let c = parse_config("/p/.holla.toml", Origin::Project, text, &["git.pull"], &[]);
        assert!(c.diagnostics[0].text().contains("built-in"));
        let c = parse_config(
            "/p/.holla.toml",
            Origin::Project,
            text.replace("git.pull", "reserved").as_str(),
            &[],
            &["reserved".into()],
        );
        assert!(c.diagnostics[0].text().contains("global configuration"));
        // whole-file syntax error: file only, no entry index
        let c = parse_config(
            "/p/.holla.toml",
            Origin::Project,
            "[[action]]\nid = \"a\nlabel",
            &[],
            &[],
        );
        assert_eq!(c.diagnostics.len(), 1);
        assert!(c.diagnostics[0].index.is_none());
        assert!(
            c.diagnostics[0]
                .text()
                .starts_with("/p/.holla.toml: line 2")
        );
        // `action = ...` that is not an array of tables
        let c = parse_config(
            "/p/.holla.toml",
            Origin::Project,
            "action = \"x\"\n",
            &[],
            &[],
        );
        assert!(c.diagnostics[0].text().contains("array of tables"));
        // an empty or missing action list is a successful zero configuration
        let c = parse_config("/p/.holla.toml", Origin::Project, "# nothing\n", &[], &[]);
        assert!(c.actions.is_empty() && c.diagnostics.is_empty());
        // quoting keeps spaces, newlines and metacharacters as one argument
        let c = parse_config(
            "/p/.holla.toml",
            Origin::Project,
            "[[action]]\nid = \"q\"\nlabel = \"q\"\ncommand = [\"echo\", \"a b\", \"x\\ny\", \"$(rm)\"]\ndanger = \"safe\"\n",
            &[],
            &[],
        );
        assert_eq!(c.actions[0].argv, vec!["echo", "a b", "x\ny", "$(rm)"]);
    }

    #[test]
    fn trust_binds_digest_path_and_cwd_and_survives_persistence() {
        let mut t = TrustStore::default();
        assert_eq!(
            t.status("d1", "/p/.holla.toml", "/p"),
            TrustStatus::Untrusted
        );
        let saved = t.approve("d1", "/p/.holla.toml", "/p").unwrap();
        assert_eq!(t.status("d1", "/p/.holla.toml", "/p"), TrustStatus::Trusted);
        assert_eq!(
            t.status("d1", "/q/.holla.toml", "/q"),
            TrustStatus::Moved,
            "same bytes elsewhere re-prompt"
        );
        assert_eq!(
            t.status("d2", "/p/.holla.toml", "/p"),
            TrustStatus::Untrusted,
            "an edit revokes"
        );
        let again = TrustStore::load(Some(&saved));
        assert_eq!(
            again.status("d1", "/p/.holla.toml", "/p"),
            TrustStatus::Trusted
        );
        assert!(again.corrupt.is_none());
        // legacy digest-only records are migration evidence, not trust
        let legacy = TrustStore::load(Some(r#"{"v":1,"hashes":["d1"]}"#));
        assert_eq!(
            legacy.status("d1", "/p/.holla.toml", "/p"),
            TrustStatus::LegacyDigestOnly
        );
        // corruption and unknown versions are untrusted and say why
        let bad = TrustStore::load(Some("{nope"));
        assert!(bad.corrupt.is_some());
        assert_eq!(
            bad.status("d1", "/p/.holla.toml", "/p"),
            TrustStatus::Untrusted
        );
        let v9 = TrustStore::load(Some(r#"{"v":9}"#));
        assert!(v9.corrupt.unwrap().contains("version 9"));
        // a save failure remembers nothing and refuses
        let mut f = TrustStore {
            save_failure: Some("read-only cache dir".into()),
            ..Default::default()
        };
        assert!(f.approve("d1", "/p/.holla.toml", "/p").is_err());
        assert_eq!(
            f.status("d1", "/p/.holla.toml", "/p"),
            TrustStatus::Untrusted
        );
        assert!(t.revoke("d1"));
        assert!(!t.revoke("d1"));
        // entries are sorted for a stable file
        t.approve("zz", "/a", "/a").unwrap();
        t.approve("aa", "/b", "/b").unwrap();
        assert!(t.serialize().find("\"aa\"").unwrap() < t.serialize().find("\"zz\"").unwrap());
    }
}
