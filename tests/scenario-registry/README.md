# Scenario Registry

Executable requirements registry for visual-baseline parity (goal S5).
Testing-only: these files pin reference behavior so the Stage B refactor
can prove parity. No production code depends on them.

## Layout

- `v1/schema.json` — JSON Schema (draft 2020-12) for the registry.
  Required scenario fields: stable `id`, `app`, `component`, `part`,
  `ref_symbols`, `cand_symbols`, `data`/`state`/`env`/`viewport`/`color`/
  `motion`, `inputs` sequence, `checkpoints`, `assertions`
  (visual + state + action + negative), `provenance`, `snapshots`
  (ref/cand paths), `adapters`, `applicability`, `results`.
- `v1/registry.json` — seed slice plus the legacy migration inventory.

## Seed slice

| ID | What it pins |
|----|--------------|
| `BTN-BUSY-FRAMES-001` | Busy Button spinner advances across ticks: tick 0 shows `⠋`, tick 1 shows `⠙` (`spinner_frame(tick)` = `SPINNER[tick % 10]`). Guards against frozen first-frame captures. |
| `BTN-BUSY-CHECKED-GEOM-002` | Busy + checked (toggle on) Button shares one 2-cell marker slot: spinner overwrites `●` at `area.x+1`, `on=true` preserved, width does not grow a second slot. |

Reference behavior read from `src/widgets/button.rs`,
`src/widgets/progress.rs`, and `src/bin/showcase/pages/buttons.rs`
at `4a79c0a2d` (visual-baseline). See each scenario's `provenance`.

## Legacy migration inventory

`legacy_roots` lists all **302** legacy snapshot roots (unique parents of
strict `WxH` size dirs under `snapshots/`), each with its real `sizes`
list and an `old -> new` placeholder (`new: null`, `status: pending`):

- holla: 125, jackin: 49, showcase: 84, tablepro: 44.

To migrate a root: add a scenario, set the root's `new` to its stable
ID and `status` to `mapped`. Retire dead roots with `status: retired`
(never delete the entry; IDs and history stay stable).

## Validate

```sh
python3 -m json.tool tests/scenario-registry/v1/schema.json > /dev/null
python3 -m json.tool tests/scenario-registry/v1/registry.json > /dev/null
```
