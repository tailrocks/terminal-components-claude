# SonarCloud check on terminal-components-baseline (2026-11-01)

## Finding

Check run `SonarCloud Code Analysis` (id 124863424719) on
`terminal-components-baseline` `main` @ `a1086507` completed
`cancelled` with output title "SonarQube Cloud analysis failed" /
"❌ The last analysis has failed." Started 13:32:01Z, ended 13:32:33Z.

## Root cause

The baseline repository is bound to SonarQube Cloud project
`tailrocks_terminal-components-baseline` with Automatic Analysis.
Rust is not supported by SonarQube Cloud Automatic Analysis
(SonarSource: "for now, Rust is not supported in Automatic Analysis
on SonarQube Cloud"); Rust analysis is CI-based only (scanner on a
runner with Cargo + Clippy). The automatic job therefore cannot
succeed on this Rust crate. The Claude repository has no Sonar
check at all (not bound), which is why only baseline shows it.

This is not a code defect and not a GitHub Actions workflow in this
repo: no workflow, secret (`gh secret list` = 0) or `sonar-project`
configuration exists here, and CI may only change through
`.velnor/config.toml` + `velnor-actions generate`, which has no
Sonar support today.

## Fix required (owner, SonarQube Cloud side)

Pick one in the SonarQube Cloud project administration:

1. Turn off Automatic Analysis (Administration → Analysis Method)
   if Sonar is not wanted for this repo; the failing check stops.
2. Or switch the project to CI-based analysis and provide a
   `SONAR_TOKEN` repository secret; the scanner job then needs
   velnor support (or an explicit exception to the generated-CI
   rule), plus Cargo/Clippy on the runner.

## Status

Update 2026-11-01: owner created the token
(`SONAR_TOKEN` secret present on `terminal-components-baseline`
since 13:42Z) and was pointed at the project's Analysis Method
setting. Remaining: a CI scanner job. velnor-actions has no Sonar
support (no sonar/sonarqube support in the workflow renderer), and
project rules forbid hand-adding a workflow, so the compliant path
is a velnor upstream Sonar feature; a hand-written workflow needs
an explicit owner exception to that rule. The check is not in the
required CI set (Plan, Suite, Rust read/write, Required,
Jactionlint).
