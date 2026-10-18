use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecKind {
    Argv,

    Shell,

    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub host: String,
    pub kind: ExecKind,
}

impl Command {
    pub fn argv(program: &str, args: &[&str], cwd: &str, host: &str) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|a| (*a).to_owned()).collect(),
            cwd: cwd.into(),
            host: host.into(),
            kind: ExecKind::Argv,
        }
    }

    pub fn from_vec(argv: Vec<String>, cwd: &str, host: &str) -> Self {
        let mut it = argv.into_iter();
        Self {
            program: it.next().unwrap_or_default(),
            args: it.collect(),
            cwd: cwd.into(),
            host: host.into(),
            kind: ExecKind::Argv,
        }
    }

    pub fn internal(label: &str, cwd: &str, host: &str) -> Self {
        Self {
            program: label.into(),
            args: vec![],
            cwd: cwd.into(),
            host: host.into(),
            kind: ExecKind::Internal,
        }
    }

    pub fn argv_vec(&self) -> Vec<String> {
        let mut v = vec![self.program.clone()];
        v.extend(self.args.iter().cloned());
        v
    }

    pub fn display(&self) -> String {
        self.argv_vec()
            .iter()
            .map(|a| quote(a))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn argv_facts(&self) -> Vec<(String, String)> {
        self.argv_vec()
            .iter()
            .enumerate()
            .map(|(i, a)| (format!("argv[{i}]"), format!("{a:?}")))
            .collect()
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}

pub fn quote(a: &str) -> String {
    let safe = !a.is_empty()
        && a.chars().all(|c| {
            c.is_alphanumeric()
                || matches!(c, '-' | '_' | '.' | '/' | ':' | '=' | '@' | '+' | ',' | '%')
        });
    if safe {
        a.to_owned()
    } else {
        format!("'{}'", a.replace('\'', "'\\''"))
    }
}

pub fn displays(cmds: &[Command]) -> Vec<String> {
    cmds.iter().map(Command::display).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_derives_from_argv_and_never_interpolates() {
        let c = Command::argv("git", &["-C", "/w/a b", "pull", "--ff-only"], "/w", "mbp");
        assert_eq!(c.display(), "git -C '/w/a b' pull --ff-only");
        assert_eq!(c.argv_vec().len(), 5);
        let mut s = Command::argv(
            "sh",
            &["-c", "docker rmi $(docker images -q)"],
            "/",
            "devbox",
        );
        s.kind = ExecKind::Shell;
        assert_eq!(s.kind, ExecKind::Shell);
        assert_eq!(s.program, "sh");
        assert_eq!(s.args[1], "docker rmi $(docker images -q)");
        assert_eq!(s.display(), "sh -c 'docker rmi $(docker images -q)'");
        let meta = Command::from_vec(vec!["echo".into(), "it's; rm -rf /".into()], "/", "h");
        assert_eq!(meta.display(), "echo 'it'\\''s; rm -rf /'");
        assert_eq!(meta.args, vec!["it's; rm -rf /"]);
        assert_eq!(meta.argv_facts()[1].1, "\"it's; rm -rf /\"");
        assert_eq!(quote(""), "''");
    }
}
