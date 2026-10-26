# T01 application and route fragment

## Record

- Task: T01 application and route inventory.
- Fragment: `app-routes` in `fragment.json`.
- Schema: `termrock.pr38.t01.fragment.v1`.
- Fixed input: `542349e2f70ec9883d743594f906ac6c4e96ba5a`.
- PR26 reference: `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`.
- Fragment SHA-256: `2fcbc58eb09074e8fc0663ac5c9d43f47ef571248a50fe758c3fa16b7769a1ed`.
- Worktree: `/Users/donbeave/Downloads/termrock-t01-apps`.
- Branch: `pr38/T01-apps`.
- Owner: `/root/t01_app_routes`.
- Required selection: `gpt-6-luna`, max effort.
- Review: independent Sol high review is pending.

## Scope

The fragment inventories 22 current application packages and their public root surfaces. It records public definitions, aliases, states, actions, builders, values, and functions separately from re-exports. It counts 368 type definitions, 98 constructors or builders, and 221 public values or functions.

It records all 54 PR26 Showcase routes. It records current route entries, aliases, factories, draw paths, shared component definitions, owners, tasks, and proposed cases.

The fragment records four required application journeys. They cover Showcase, Holla, Jackin Preview, and TablePro.

## Route findings

The current navigation registry has 54 entries. It has 24 `Live` tags, one `Selection` tag, and 29 `Placeholder` tags.

Five `Placeholder` entries select stored page objects. These entries are Checks & radios at index 12, Empty & error at 31, Notices at 33, Master-detail at 37, and Diff at 40. Their target tag, state dispatch, page definition path, constructor site, and draw path remain separate fields.

The remaining 24 placeholder entries paint their labels. Their route rows set the current content owner to null and record the planned page owner separately.

Selection uses a separate model and paint function. It has no `PageId` factory.

Current CLI name lookup accepts normalized labels and aliases. Navigation search checks labels, live page titles or slugs, and aliases.

Current navigation keys derive from entry and group indices. Reordering can change these keys.

Current CLI parsing ignores unknown options. An invalid page name keeps Overview. An invalid theme selects Junie, and an invalid color value has no effect.

Current motion options accept `full` and `paused`. They do not expose a Reduced mode.

All route cases remain proposed. They are not approved T38 plans. Product behavior evidence remains `NOT VERIFIED`.

## Application and boundary findings

Holla uses a deterministic simulation. Keep task execution, disk cleanup, port checks, and file actions inside modeled state.

Jackin Preview uses virtual-world transitions and masked credentials. T07 moves Jackin page assembly out of `termrock-feedback`.

TablePro uses in-memory demo data and simulated SQL. Its current comparator keeps NULL last in both sort directions.

T08 moves app page assembly and UI state to consumer crates. It keeps routes, SQL policy, domain data, and simulations with their app owners.

The historical performance workloads remain definitions. No measurements ran. T33 owns missing workload measurements.

## Commands and results

| Command or action | Result |
| --- | --- |
| `git rev-parse HEAD` | `542349e2f70ec9883d743594f906ac6c4e96ba5a` |
| `git -C /Users/donbeave/Downloads/termrock-pr26-reference-20261026 rev-parse HEAD` | `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948` |
| `python3 -m json.tool docs/implementation/pr38-final/results/t01-app-routes/fragment.json >/dev/null` | Exit 0. |
| Read-only Python structural and source-hash audit | Exit 0. It checked 54 unique routes, four apps, 22 packages, 270 source records, and 270 source hashes. |
| Route census Python command | Exit 0. It found 54 routes, 24 Live tags, one Selection tag, 29 Placeholder tags, and five stored pages. |
| `git diff --cached --check` | Exit 0. It reported no whitespace errors. |
| Cargo, tests, benchmarks, PTY probes, and cleanup | Not run. T01 cannot prove product behavior. |

An earlier route-summary command had a Python quoting error and exited 1. The corrected route-summary command passed. This failure affected only the inline reporting command.

## Output evidence

The fragment records exact source commits, file hashes, and line spans. Its structural audit resolved every source reference and verified every recorded source hash.

The final fragment SHA-256 is `2fcbc58eb09074e8fc0663ac5c9d43f47ef571248a50fe758c3fa16b7769a1ed`. The fragment remains pending independent review.
