# Jackin branch-review interaction obligations — TASK-053

## Authority and evidence boundary

This supplement binds BD-JT-16/20/24/26 families for Editor, Settings, Config and global state owned primarily by TASK-053. ADJ-19 governs non-atomic config quirks. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-CONFIG-QUIRKS: source-qualified pending transitions — full owner TASK-053 (ADJ-19)

`CONFIG-ENV-TWO` seeds pending environments A and B where B already exists.

1. **config-rename-removes-before-duplicate-check:** rename A→B name. Require A removed from pending before duplicate error; observe pending/original immediately before and after rejection.
2. **config-scope-move-replaces-same-key:** move scoped row to existing destination key. Require deliberate replacement2026–2038, not atomic merge failure.
3. **config-numeric-isolation-bypass:** running isolated workspace; press `1`/`2`/`3`. Require bypass of running-isolated guard1309–1315; contrast with `i` path1295–1306.

Sources: oracle `screens/config.rs:1295–1315,2026–2038,2199–2208`.

## JT-SETTINGS-SCOPED-STATE: real GlobalConfig pending — BD-JT-24

1. **settings-agent-ignore-after-save:** change agent mode to Ignore; save successfully. Require subsequent session availability changes only after successful save, not on selection alone.
2. **settings-trust-row-not-row0-toggle:** change selected trust row. Require toggle affects selected row only; odd propagation guard349–357 preserved as parity input.

Sources: oracle `screens/settings.rs:316–364,705–778,826+`.

## JT-PICKER-KEY-RESOLUTION: nonfirst filtered match — BD-JT-16

Query matching two roles; choose second filtered match. Require override applied to chosen key, not first substring match.

Sources: main ignores key at `app.rs:1966–1975`; oracle role catalog journeys JA-020/022/024.

## JT-WORKSPACE-TARGET; JT-WORLD-POLICY; JT-HISTORICAL-PAINT; JT-LIVE-REDUCERS; JT-MIGRATED-TESTS

Editor/settings reducers, environment reveal/remask and first-workspace shortcuts bind through JA-* rows with explicit named branches from BD-JT-17/18/20/23.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-057 closes Jackin union.
