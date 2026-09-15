# Jackin branch-review interaction obligations — TASK-056

## Authority and evidence boundary

This supplement binds BD-JT-11/19/25/28 Inspect, Capsule exit and diff fixture families owned primarily by TASK-056. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-CAPSULE-EXIT: source dirty-exit choices — BD-JT-19 (JA-059 owner)

`CAPSULE-RUNNING` reaches Capsule with dirty nested state.

1. **capsule-exit-keep-discard-cancel:** open exit choices. Require Start new agent, Inspect changes, Exit&keep, Exit&discard plus separate Cancel; not main Stay/Exit/Cancel inversion.
2. **capsule-discard-typed-confirmation:** choose Exit&discard. Require typed-confirmation facts Dialog before mutation.

Sources: oracle `screens/capsule.rs:1080–1117,2296–2333`; main `app.rs:3580–3625`.

## JT-DIFF-FIXTURE: exact hunks — BD-JT-11

1. **inspect-retry-three-hunks:** Inspect retry.rs; jump next hunk twice. Require three hunks with source line numbers; main single-hunk extraction278–287 is wrong restoration target.

Sources: oracle `sim/changes.rs:65–103`; main `sim/changes.rs:275–347`.

## JT-INSPECT-CONTRACTS: compact/advanced independence — BD-JT-25

Preserve F09 no-clipboard branch for compact/advanced Diff copy events; do not infer delivered clipboard from visible `y Copy` hint.

Sources: oracle `screens/inspect.rs:250,343`; ADJ-09.

## JT-CAPSULE-ROUTES

Capsule final application proof repeats ADJ-20 route enumeration with TASK-055 component witnesses.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-057 closes Jackin union.
