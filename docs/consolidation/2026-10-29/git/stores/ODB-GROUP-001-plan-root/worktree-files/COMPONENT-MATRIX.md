# Component, route, and application matrix

Use this matrix with [API-RULES.md](API-RULES.md), [VALIDATION.md](VALIDATION.md), [TASKS.md](TASKS.md), [SOURCES.md](SOURCES.md), and [TESTING-RULES.md](TESTING-RULES.md). The testing rules govern conflicts about tests and source comments. Keep the component, route, and journey requirements in this matrix.

## Evidence status

This matrix sets required work and acceptance. It does not mark components as complete. The author inspected source and four distinct committed PNG samples. Corresponding target files had identical hashes. No application ran during that review. No fresh Rust test result or live capture was produced.

The target is PR #38 at `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`. The design reference is PR #26 at `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`. The frozen `visual-baseline` tag object is `2718a6b8127a522a3d351d0b4bdb7542fc8da542`. It peels to source commit `4169783ae883788186d161af09ba9c481bd84e5c`. Preserve that tag and release. Do not confuse the frozen source with the current design reference.

The [product contract](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/product.md) makes the design reference the appearance and interaction authority. Historical images give comparison evidence. They do not prove current output.

## Common acceptance for each component

Every row requires the applicable behavior evidence below. Record a reason for each non-applicable requirement. Preserve source-backed capabilities. Do not add a feature only because a generic test category names it. T38 defines the functional test architecture. Each later task needs its own reviewed plan before it writes or changes a test. These scenarios describe required outcomes, not a required test count.

- Use the public API from an external consumer. Show default use and one meaningful policy override. Compile the example. Observe behavior through input and output.
- Cover stable owner, item, part, and layer identity. Use two instances in one application. Replace, sort, filter, and remove data where supported.
- Cover keyboard-only operation and unhandled input. Cover Press, Repeat, and Release at the adapter, plus remapping and terminal deliverability.
- Cover hover, press, click, right click, drag, wheel, and capture cancellation where supported. Use registered hit regions and the event path.
- Cover rest, focus, selected, hover, pressed, disabled, read-only, error, empty, loading, and covered states where supported. A disabled control must not activate or react to hover.
- Cover Unicode graphemes, terminal-cell width, clipping, long labels, zero and tiny areas, and resize in the same runtime.
- Cover TrueColor, 256, 16, `none`, and `NO_COLOR`. Preserve semantic roles and deliberate glyph or modifier changes. Cover dark and light theme overrides.
- Cover operation through keys, visible focus, text state labels, and clear error messages. Keep required distinctions readable without color. Cover reduced and paused motion through the existing motion provider.
- Cover failure before commit. A rejected edit or action keeps valid prior state. Cover secret redaction and copy policy where sensitive text is present.
- Link one real Showcase use and one relevant application journey. Add an independent second consumer when shared ownership is in doubt.
- Produce three forms of evidence for each visible behavior change: semantic state, interaction result, and rendered output. Preserve exact pinned reference output for accepted states.

Static state specimens can display several focus examples. Mark them as noninteractive specimens. They must not register live focus or hit targets. The one-focus-owner rule applies to reachable controls in the active layer. It does not count all `▎` glyphs in a screenshot.

Record terminal and assistive-tool limits separately. A screenshot or semantic test does not prove screen-reader support. Such a claim requires a named terminal, tool, tested journey, and result.

## W01–W45 ownership and acceptance

The IDs and declared owners below come from [component-ownership.json](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/component-ownership.json). Its schema is `proposal-v1`. Treat ownership as a proposal until the implementation boundary is verified. T00 records the old conformance check as historical evidence. Do not retain or run its test code after T37. A re-export alone does not establish physical ownership or behavior completeness.

| ID | Component | Declared owner | Specific required acceptance |
| --- | --- | --- | --- |
| W01 | Brand | `termrock-controls` | Static form has no Tab stop or action. Clickable form activates once. Metadata clips by cells. Two brands remain independent. |
| W02 | Button | `termrock-controls` | Enter/Space and completed click activate once. Release outside cancels. Busy/loading/disabled refuse activation. Slots keep action and focus geometry. |
| W03 | Checkbox | `termrock-controls` | Checked and mixed states remain distinct. Space toggles. Read-only and disabled rules hold. Chosen mark remains distinct from focus. |
| W04 | Toggle | `termrock-controls` | On/off changes once per activation. Label and mark agree. Hover and press do not change the committed value. |
| W05 | RadioGroup | `termrock-controls` | One chosen value; declared arrow-selection policy; disabled option skip; stable value on reorder; focus and choice stay distinct. |
| W06 | ChipBar | `termrock-navigation` | Single/multiple selection policies; disabled chips; overflow; cursor versus selected set; pointer and key parity. |
| W07 | Field | `termrock-fields` | Label, help, error, required mark, and child align. Error text remains reachable. Wrapper preserves the child identity and edit behavior. |
| W08 | TextInput | `termrock-fields` | Grapheme edit, cell cursor, SelectAll/CaretEnd, validation, commit/cancel/blur, undo/redo, secret policy, copy-before-quit. |
| W09 | TextArea | `termrock-fields` | Multi-line selection, wrap, vertical/horizontal scroll, newline policy, paste, undo/redo, and cursor reveal survive resize. |
| W10 | Select | `termrock-overlays` | Open/commit/cancel; owner focus; popup row fill; empty/disabled options; placement clamp; outside dismiss; choice survives reorder. |
| W11 | Form | `termrock-forms` | First-error focus; grouped layout; optional fields; submit validation; atomic commit; dirty leave; long error and narrow form reachability. |
| W12 | List | `termrock-navigation` | Stable keyed cursor/selection/anchor; wheel does not move focus; selectable-only SelectAll; row metadata; empty and virtualized rows. |
| W13 | FilterList | `termrock-navigation` | Query owns text and shortcuts; Unicode matching policy; zero results; hidden chosen item policy; query clear and focus restore. |
| W14 | NavList | `termrock-navigation` | Route choice distinct from cursor; badges and overflow; current route reveal; collapsed/expanded presentation; disabled destinations. |
| W15 | Tree | `termrock-navigation` | Parent/leaf actions; branch key/click policies; expansion identity; filtered ancestors; stable cursor after removal; scroll/reveal. |
| W16 | Steps | `termrock-navigation` | Current/completed/error/disabled states; forward/back rules; compact labels; overflow reveal; stable step identity. |
| W17 | Tabs | `termrock-navigation` | Chosen tab versus focus; arrow policy; disabled skip; long-title overflow; neutral covered selection; content focus transition. |
| W18 | Picker | `termrock-overlays` | Search, caller-ranked items, disabled item, no results, preview, commit/cancel, modal trap, and owner restoration. |
| W19 | CommandPalette | `termrock-overlays` | Command identity, disabled action, query, execution result, and closure. Verify alias semantics: current type aliases `Picker`. |
| W20 | PickerChain | `termrock-overlays` | Forward/back chain; retained prior state; cancel at each depth; no background action leak. Test stale responses only for an existing async adapter. |
| W21 | Completion | `termrock-overlays` | Owner-focused fill; candidate/detail views; accept/cancel; stale revision rejection; placement; editor cursor and focus preserved. |
| W22 | Dialog | `termrock-overlays` | Safe initial action; Tab/BackTab wrap; nested modal; Esc/outside policy resolves cancel; validation; owner restoration; narrow body scroll. |
| W23 | Menu | `termrock-overlays` | Test this descriptor through MenuBar and ContextMenu. Check separators, disabled entries, submenu actions, stable identity, and dismissal. |
| W24 | ContextMenu | `termrock-overlays` | Right-click selected row keeps multi-selection. Unselected row becomes context. Clamp placement. Resolve current action at dispatch. |
| W25 | MenuBar | `termrock-overlays` | Keyboard menu switching; hover-switch override; submenu restoration; Escape depth; inactive/covered styling; terminal-valid hints. |
| W26 | HelpOverlay | `termrock-overlays` | Actual remapped and deliverable keys; search/scroll if supported; top-layer ownership; close restores the prior owner. |
| W27 | Wizard | `termrock-forms` | Step validation; Back preserves data; Finish commits once; cancel/dirty guard; invalid step and hidden control handling. |
| W28 | Grid | `termrock-grid` | Keyed rows/columns; selection/cursor/anchor; sort/filter/resize; horizontal reveal; cell edit/validation; seams; sticky columns and horizontal overflow. |
| W29 | CodeEditor | `termrock-editors` | Grapheme text; indentation policy; find; selection/copy; diagnostics; scroll; undo; completion owner; large source and secret diagnostics. |
| W30 | DiffView | `termrock-editors` | Unified and Review modes; context rows; line identity; empty/no file; wide Unicode; horizontal clipping; responsive fallback preserves the requested mode. |
| W31 | TextViewport | `termrock-viewport` | Wrap policy; selection; cell mapping; long lines; vertical/horizontal reveal; viewport work bound; resize retains anchor. |
| W32 | Panel | `termrock-controls` | Insets/title/footer; clipping window; empty and zero area; slot geometry; theme surfaces; no paint outside the panel. |
| W33 | SplitPane | `termrock-controls` | Mouse/keyboard resize; min/max; reset policy; drag release outside; seam-only paint; focus and children preserved across resize. |
| W34 | Props | `termrock-controls` | Key/value alignment; long values; Unicode truncation; empty rows; deterministic natural size; no fake focus on static properties. |
| W35 | PropsList | `termrock-navigation` | Property navigation, selection/reveal, overflow, independent instances, and documented action. Verify actual owner: defined in `termrock-controls`. |
| W36 | Empty | `termrock-controls` | Empty/error/loading presentation; accessible message; optional action; retry outcome; narrow text and zero area. |
| W37 | ProgressBar | `termrock-feedback` | Zero/full/out-of-range input policy; determinate/indeterminate state; done and status variants; caller-supplied frame; legible label and track. |
| W38 | Spinner | `termrock-feedback` | Deterministic caller-supplied frame; empty/custom frame sets; clipped Unicode glyphs; monochrome state. Test motion policy in its existing provider. |
| W39 | Meter | `termrock-feedback` | Threshold boundaries; unknown/stale/empty value; role colors; precise labels; overflow; no reliance on color alone. |
| W40 | StatusBar | `termrock-feedback` | Priority-based truncation; left/center/right layout; keyed action hit regions; long Unicode at minimum width. Test expiry and layer selection in existing providers. |
| W41 | HintBar | `termrock-feedback` | Hint priority/overflow; explicit metadata clear/inherit; badge and status clipping. Test effective binding and capability filtering through T19 integration. |
| W42 | KeyHint | `termrock-feedback` | Chord label matches delivered/remapped command; Unicode key glyph width; modifier readability in every color mode. |
| W43 | TooSmall | `termrock-controls` | Exact minimum notice; fit at 0x0 and tiny bounds; configured quit remains reachable; resize back restores state and focus. |
| W44 | TerminalView | `termrock-terminal` | Consume TerminalSource. Test projected cells, scrollback, selection/copy, and resize. Test sanitation, pause, and storage bounds in existing source providers. |
| W45 | ScrollRegion | `termrock-viewport` | Nested wheel route; consume only applied delta; reveal; horizontal retry; thumb drag; clipping; owner identity. Verify actual owner: runtime. |

Apply the ownership decisions in [ARCHITECTURE.md](ARCHITECTURE.md). W35 uses controls as its implementation owner. W45 uses runtime as its implementation owner. [PropsList](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-controls/src/props.rs#L310) is defined in controls and [re-exported by navigation](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-navigation/src/lib.rs#L53). [ScrollRegion](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/scroll_region.rs#L54) is defined in runtime and [re-exported by viewport](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-viewport/src/lib.rs#L16). Keep compatible re-exports during the migration. Do not move these implementations to match stale owner labels. [CommandPalette](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-overlays/src/picker.rs#L788) is a `Picker` alias. Do not count the alias as a second independent implementation.

## Additional public components and shared contracts

The 45 IDs are not the complete current public surface. Add the following to the inventory. Do not assign a new W ID without the inventory migration. Preserve source names while deciding ownership.

| Existing surface | Current source owner | Required additional proof |
| --- | --- | --- |
| `MasterDetail`, `RadioField` | `termrock-controls` | Narrow master/detail switch, back navigation, selection and focus restore; radio field label/error composition. |
| `SplitBlock`, block `MasterDetail` | `termrock` | Reuse control geometry and caller state. Validate nested content before output. Test missing panes, responsive layout, focus handoff, and effect propagation. |
| `Table`, `GridViewport` | `termrock-grid` | Table sorting, row selection, column metadata, seam resize, empty data; viewport bar interdependence and focus reveal. |
| `FormDialog`, `FormLayout` | `termrock-forms` | Form-owned validation inside a modal; submit and cancel outcomes; lane sizing, error overflow, nested picker focus. |
| `SelectField`, `NoticeReveal` | `termrock-overlays` | Field/popup consistency; hover reveal when text is cut; revision and window checks; dismiss without data loss. |
| `CompletingCodeEditor` | `termrock-editors` | Real provider/candidate flow; stale result rejection; one edit transaction; manual and automatic trigger policy. |
| `Note`, `FieldErrorChrome`, edit capabilities | `termrock-fields` | Long help/error reachability; shared edit menu; secret/read-only eligibility; no app-local copy implementation. |
| `Notice`, `TextSection`, `DerivedHintBar` | `termrock-feedback` | Test semantic tones, cut records, clipping, section measurement, and patches. Test top/mode/focus/screen/global hint precedence with explicit metadata overrides. |
| `AccountsDetail`, `AccountsDrawer`, `AccountsFilter`, `UsageOverview` | `termrock-feedback` | Move account and usage page models to Jackin UI owners under T07. Keep generic primitives shared. Preserve stable projections and row windows. |
| `ManagerFrame`, `ManagerSplit`, `CapsuleMenu`, `SessionChrome`, `ZoomedPane` | `termrock-feedback` | Move product page assembly to the Jackin owners in ARCHITECTURE.md. Keep reusable layout primitives shared. Preserve menu, split, and layer behavior. |
| `SettingsPage`, `SettingsAgents`, `SettingsEnv`, `SettingsTrust`, `EditorEnvironmentBody`, `EditorRolesBody`, inspector paint APIs | `termrock-feedback` | Move product models and page assembly under T07. Preserve field actions, projected rows, clipping, and natural size. Cover retained generic primitives through independent consumers. |
| `TextReport`, `ReportContent`, `ReportRows` | `termrock-shell` | Test row grids, insets, heading/detail/body patches, empty content, height policy, and cell clipping. Count lazy row access. Keep this painter noninteractive. |
| `AppShell`, page/router/leave policy, keyboard capability filter | `termrock-shell` | One shared route/quit guard; one active hint owner; runtime/session-local key capabilities; second consumer. |
| `ReferenceState`, `ReferenceTarget`, composition slots, scrollbar, focus, layers, clipboard, typed bindings | `termrock-runtime` | No semantic mutation in measure/draw; generation validity; paint override preserves behavior; one owner per event; result-bearing clipboard integration. |

Sources: [controls exports](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-controls/src/lib.rs#L43-L59), [grid exports](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-grid/src/lib.rs#L27-L39), [forms exports](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-forms/src/lib.rs#L30-L38), [feedback exports](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-feedback/src/lib.rs#L36-L92), [editor exports](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-editors/src/lib.rs#L26-L37), and [TextReport](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-shell/src/report.rs#L95-L225).

## Showcase route census

The target has 54 navigation entries. It has 24 `NavTarget::Live` entries, one `Selection` entry, and 29 `Placeholder` entries. Five Placeholder entries already draw live pages. These are Checks & radios at index 12, Empty & error at 31, Notices at 33, Master-detail at 37, and Diff at 40. Thus 30 entries have a content path and 24 still draw placeholder content. This is a route census. It is not a behavior pass.

Sources: [PageId and targets](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L91-L267), [live special cases](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L1697-L1729), and [placeholder draw](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L2844-L2867). T00 records old census assertions and visual scenario registries as historical inputs. They do not identify test code to preserve or execute after T37.

Replace the parallel route tables and numeric special cases with one typed registry. Give all 54 routes stable identity, aliases, factory, group, capabilities, and minimum-size policy. Migrate Selection and the five live Placeholder routes into that registry. Preserve their current behavior during the migration.

The table below covers every remaining placeholder. S-A maps to T23. S-B maps to T24. S-C maps to T25. S-D maps to T26. T22 completes typed route integration after all four batches. The current route owner is `crates/showcase-ui/src/app.rs`. Every reference file is under `src/bin/showcase/pages/` in PR #26. Every proposed demo file is under `crates/showcase-demos/src/pages/`. The proposed files do not yet exist at the target SHA.

| Batch | Index | Route | Reference file and proposed demo filename | Shared owner and task |
| --- | --- | --- | --- | --- |
| S-A | 2 | Status & hints | `status_hints.rs` | Shell/feedback: live priority, status expiry, remapping, top-layer hints. |
| S-A | 3 | Breadcrumbs & rails | `breadcrumbs_rails.rs` | Navigation/shell: stable path, overflow, current route, keyboard travel. |
| S-A | 4 | Help & keys | `help_keys.rs` | Shell/runtime/overlays: actual bindings, deliverability, help focus and dismiss. |
| S-A | 5 | Quit & leave | `quit_leave.rs` | Shell/overlays: clean, dirty, running, save failure, copy-before-quit. |
| S-A | 6 | Focus & navigation | `focus_navigation.rs` | Runtime: scopes, traps, disabled skip, restore, nested layer. |
| S-B | 13 | Toolbars | `toolbars.rs` | Controls/navigation: actions, group movement, disabled state, overflow. |
| S-B | 23 | Tabs | `tabs.rs` | Navigation: tab choice, focus, key policy, covered state, overflow. |
| S-B | 24 | Menus | `menus.rs` | Overlays: menu bar, submenus, context menu, hover-switch policy. |
| S-B | 28 | Popups & tooltips | `popups_tooltips.rs` | Overlays: placement, owner, light dismiss, delayed/reduced motion, reveal. |
| S-B | 29 | File browser | `file_browser.rs` | Collections/overlays: deterministic demo tree, filter, selection, cancel. |
| S-C | 19 | Columns | `columns.rs` | Grid: width policy, column identity, resize, hide/reveal, metadata. |
| S-C | 32 | Meters | `meters.rs` | Feedback: thresholds, unknown/stale, semantic tones and labels. |
| S-C | 36 | Splits & stacks | `splits_stacks.rs` | Layout/controls: measured composition, seam resize, minimums, reset. |
| S-C | 38 | Pane tree | `pane_tree.rs` | Layout/viewport: nested resize, stable leaf focus, close/remove fallback. |
| S-C | 42 | Text utilities | `text_utilities.rs` | Text/editors: grapheme and cell widths, wrap, truncate, selection and reveal. |
| S-D | 45 | Query workbench | `query_workbench.rs` | Compose editor/grid/shell with deterministic demo queries and results. |
| S-D | 46 | Connections | `connections.rs` | Compose shared lists/forms/dialogs with deterministic demo connection data. |
| S-D | 47 | Finder | `finder.rs` | Compose tree/list/viewport with deterministic demo search data and selection. |
| S-D | 48 | Disk cleanup | `disk_cleanup.rs` | Compose selection/meter/confirmation with simulated effects only. |
| S-D | 49 | Activity | `activity.rs` | Compose progress/viewport/time policy with deterministic jobs. |
| S-D | 50 | Workspaces | `workspaces.rs` | Compose navigation/list/overlay with stable workspace identity. |
| S-D | 51 | Accounts & usage | `accounts_usage.rs` | Compose forms/meters/list. Keep account domain data outside primitives. |
| S-D | 52 | Launch cockpit | `launch_cockpit.rs` | Demonstrate reusable composition. Keep Jackin intro/outro/branding in Jackin. |
| S-D | 53 | Read-only report | `readonly_report.rs` | Compose facts, meters, notices, refresh/copy, and navigation actions. Preserve read-only, offline, and stale states. |

The [reference page factory](https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/bin/showcase/app.rs#L680-L733) lists these page implementations. Port the required experience through shared APIs. Do not copy app-local drawing and routing merely to match a frame.

Each restored route must show a default interactive use, relevant state specimens, and meaningful configuration. An approved functional scenario must enter through the actual route, operate the shared control, and observe the result. A title match or public-symbol mention does not prove behavior.

T00 records the target's capability inventory and Diff proof records as historical inputs. They are not the post-T37 active inventory. T01 creates the production requirement inventory after cleanup. Do not interpret a source count as a quality measure.

## Application journeys

Use the actual application entry points and shared components. Keep all domain effects simulated. A shared-component case does not close the full app journey.

| App | Required journeys | Required results |
| --- | --- | --- |
| Showcase | All 54 routes; query and aliases; group expansion policies; inspector dock/modal; help; edit/validation; nested popup; dirty/running leave | Real content; route and focus stability; accurate hints; no placeholder; no duplicate event ownership; same reference appearance. |
| TablePro | Connection navigation; query edit/run/error; result sort/filter; multi-selection; cell editor; column resize; context menu; save/cancel; leave | Stable row/column IDs; atomic invalid-edit rejection; data-model and frame agreement; no background edit during modal; reference layout. |
| Holla | Catalog search; alias and ranking; command review; dependency plan; required/optional/excluded steps; simulated run/pause/stop; help and quit | Current action re-resolved; review bound to target; deterministic progress; correct stop/stay policy; secret-safe output; reference layout. |
| Jackin Preview | Host/capsule navigation; settings; accounts/usage; manager split; overlay menu; selection/copy; launch transition; stop/leave | Shared behavior stays shared; brand transitions stay app-owned; focus restored across host/capsule layers; deterministic transition frames. |

T00 records the old visual scenario registries as historical inputs for the route and journey census. Do not retain or run their test code after T37. T38 and later reviewed task plans define replacement scenarios. Each replacement must reach its named state through the production path; registry membership alone does not prove behavior.

## Visual and behavior matrix

| Axis | Required values | Acceptance |
| --- | --- | --- |
| Normal viewport | 72x20, 80x24, 100x30, 120x40, 160x50 | Exact accepted reference frame for the same app/scenario/state. |
| Narrow viewport | 40x12 and 60x18 | Deliberate supported layout or TooSmall state; no hidden required action; recovery on resize. |
| Resize thresholds | 99↔100 and 109↔110 columns; minimum-height crossings | Same runtime preserves route, draft, selection, focus, and overlay lifecycle. |
| Color mode | truecolor, 256, 16, none, nocolor | Preserve role distinctions. `NO_COLOR` uses deliberate glyphs/modifiers. |
| Pointer state | rest, hover, press, drag, release outside | Correct owner, action count, capture lifecycle, and visual ladder. |
| Focus state | active, inactive layer, restored, disabled skip | One live focus owner; no background focus paint; deterministic restore. |
| Data state | populated, empty, no match, error, loading, stale, disabled, read-only | Clear state and available action; no stale target or selection. |
| Text | ASCII, combining mark, CJK, emoji ZWJ, flag, long unbroken text | Grapheme-safe edit and cell-safe geometry; no partial glyph or invalid UTF-8. |
| Motion | Full, Reduced, Paused | Deterministic progress and frame; no time drift from drawing. |
| Override | theme, slot, key map, app/profile policy, per-widget policy | Documented precedence; no hidden behavior loss; another consumer can use it. |

The target currently requires a 72x20 Showcase minimum. Its [draw path](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/showcase-ui/src/app.rs#L2801-L2809) renders TooSmall below that size. Do not require full page controls at 40x12 unless a deliberate product change lowers the minimum. Require a correct fallback and state recovery. Component-level tiny-area behavior remains required.

## Reference output and baseline approval

1. Build the pinned PR #26 source in an isolated checkout. Record exact source, toolchain, lockfile, binary, and harness identities. Any capture adapter must be reviewed and must not change product behavior.
2. Capture to a separate candidate directory. Pin terminal dimensions, color mode, font/render profile, environment, locale, random seed, and monotonic time. Use the project's tuiscotti capture path.
3. Assert the semantic state before each screenshot. An `inspector/open` capture must contain the actual inspector controls. A dialog case must identify its active layer, initial focused owner, and allowed actions.
4. Review the candidate at the required viewport and color combinations. Separate source defects, unsupported states, harness defects, and deliberate changes. Record exact exceptions. Do not silently canonize a defect.
5. Compare current target output to the accepted source output. Keep expected and actual outputs separate. Review a delta image plus semantic and interaction results. Pixel equality alone does not prove behavior.
6. Seal only the reviewed candidate. Keep the frozen historical tag unchanged. Obtain any required approval for the concrete candidate after generation and review work is complete.
7. Make missing, skipped, stale, and pending required cases fail the claimed parity tier. Report counts separately. Do not turn a pending return into a successful capture.

The [root oracle manifest](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/baselines/oracle/ORACLE.toml) is pending at the target SHA. T00 records the old harness fallback and capture guard as historical evidence. T37 removes old test-only harness code. Preserve approved immutable artifacts separately. Do not describe the current manifest or old harness as an approved PR #26 oracle.

Four retrieved PNG pairs from the two heads were byte-identical. The samples cover Inputs audit, Dialogs open, and Inputs selected at 120x40/truecolor. The fourth sample covers Inspector open at 72x20/nocolor. They show older navigation than the current source. The narrow Inspector sample has an “on” label without visible inspector controls. These are historical assets. They are not current-head pass evidence.

## Closure record

For each component, route, and app journey, record status as Not reviewed, Failing, In progress, Verified, or Approved exception. Attach the exact source SHA, approved case IDs and commands, result artifacts, capture IDs, semantic assertions, reviewed deltas, and reviewer. Report results separately from any count. Separate API/architecture, behavior, visual, and accessibility evidence. Do not collapse them into one green status.

A component is complete only when its required public API, real consumers, behavior cases, and visual states pass at the same final head. A route is complete only when it exercises the shared implementation. An application is complete only when its required journeys pass. An approved exception must name the unresolved contract and the human decision that permits it.
