# Picker row contrast defaults

Base: `f46caa2ce68966d319ffbb4cf03428e8234368f3`.

The independent review at `/private/tmp/picker-row-review-f46c-20261021-followup.json` (SHA-256 `601139db431c0d10651b413ecf42a45cfb89477ca2015f708fdeb5c9252bb2ff`) identified unreadable default `META` foregrounds on an authored light owner background. The review's decoded cells measured 3.45:1 for muted gray, 2.10:1 for secondary gray, and 1:1 for white on white. Its foreground provenance gap was not itself a demonstrated render defect.

`termrock-runtime` now adjusts only generated `PICKER` `ICON` and `META` foreground defaults when their resolved contrast falls below 4.5:1. It chooses the better-contrasting black or white ink against the already-resolved background. The resolver still applies theme recipes and explicit overrides above defaults; explicit foregrounds, including one equal to the computed fallback, retain ownership. Backgrounds, modifiers, and existing readable default colors stay unchanged. No application flags or value-equality provenance checks were added.

The regression uses the real `FilterList` runtime lifecycle and checks Bar and Fill cursor policies, light/dark/default backgrounds, all four color profiles, authored modifiers, and explicit white-on-black plus black-on-white foregrounds. At the base commit, the same test failed with `Rgb(179, 179, 179)` at 2.10:1 on white. After the fix:

```text
CARGO_TARGET_DIR=/private/tmp/picker-row-contrast-target-20261021 \
  rustup run 1.99.1 cargo nextest run -p termrock-navigation --lib \
  picker_tag_default_ink_tracks_resolved_owner_contrast
PASS 1/1; 111 filtered out
```

Complete scoped suites also pass: `cargo nextest run -p termrock-navigation --lib` (112/112) and `cargo nextest run -p termrock-runtime --lib` (261/261). Scoped all-target Clippy with `-D warnings`, `cargo fmt --all --check`, and `git diff --check` pass.

The historical fixture failure recorded for the picker-default-precedence increment remains unwaived and unchanged. This bounded fix does not certify picker interaction parity, frozen baselines, application parity, or PR #38 acceptance.
