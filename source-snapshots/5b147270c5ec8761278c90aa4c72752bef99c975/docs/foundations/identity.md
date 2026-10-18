# Identity, keys, and revisions

**Foundation F01 · canonical owner of control identity, source identity, and reconciliation generations**

**Legacy family:** C01.

This page defines the identity vocabulary shared by every Termrock component. It is a library contract, not an application or product subsystem. Components and task descriptions link here instead of defining their own ID, row-key, or revision rules.

Termrock is produced by refactoring this repository in place on `termrock-implementation`, starting from the frozen visual baseline commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. The current implementation uses `WidgetId` in [`src/core/id.rs`](../../src/core/id.rs); that is a legacy source name to be replaced during the implementation phase. The target public vocabulary is `Id`, `ItemKey`, `ColumnKey`, `FieldKey`, `ActionKey`, `PartRef`, and `Revision`.

## Source and oracle references

- Migration input: `termrock-library-spec/foundations/identity.md` and the F01 entry in `termrock-library-spec/reference/foundations.json`.
- Frozen source: [`src/core/id.rs`](../../src/core/id.rs) at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
- Shared type vocabulary: [`../api/types.md`](../api/types.md).
- State and collection ownership: [`../foundations/input-actions.md`](../foundations/input-actions.md), [`collections.md`](collections.md), and [`text.md`](text.md).
- Public component boundary: [`../api/public-api.md`](../api/public-api.md).
- Baseline provenance and comparison rules: [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md).

The source pack's proposed signatures are design declarations, not an existing Rust API. The implementation phase may choose private representations, but it must preserve the semantics below and update the public API reference before freezing a spelling.

## Target vocabulary

```rust
Id::new(namespace: &'static str) -> Id
Id::child(&self, key: ItemKey) -> Id
ItemKey::new(value: u64) -> ItemKey
ColumnKey::new(value: u64) -> ColumnKey
FieldKey::new(value: u64) -> FieldKey
ActionKey::new(name: &'static str) -> ActionKey
Revision::new(value: u64) -> Revision
PartRef::new(owner: Id, part: Part, item: Option<ItemKey>) -> PartRef
trait Keyed { fn key(&self) -> ItemKey; }
```

These types are intentionally distinct:

| Type | Owner and meaning | Must not be used as |
| --- | --- | --- |
| `Id` | Component/control identity used by runtime focus, hit testing, capture, response ownership, and part addressing. | A row key, display label, or array position. |
| `ItemKey` | Stable identity supplied by a dynamic source for a row, item, line, or other source record. | A control identity or a transient filtered index. |
| `ColumnKey` | Stable identity for a grid column. | A visual column offset. |
| `FieldKey` | Stable identity for a form field. | A field label or child position. |
| `ActionKey` | Stable identity for a semantic command/binding. | A display string or a widget ID. |
| `Revision` | Monotonic accepted source generation. It invalidates derived projections and detects stale edits. | A clock, frame number, or draw count. |
| `PartRef` | Owner, advertised part, and optional item identity for constrained styling/slot diagnostics. | An unrestricted paint target. |

`Keyed` is the borrowed model contract for dynamic collections. Noninteractive headings and separators do not become fake selectable records merely to satisfy it.

## Identity construction and collision policy

Identity composition is structured and length-delimited. A hash may accelerate lookup, but a hash value is not allowed to silently define equality across concatenation boundaries. `Id::child` and equivalent item/part derivations must preserve namespace boundaries and distinguish an owner from any child of that owner.

Duplicate live `Id`, `ItemKey`, or `ColumnKey` values are diagnosed before the frame publishes interactive geometry. Production dispatch fails closed: it does not pick the first, last, or hash bucket winner when two controls could receive the same event. `FieldKey` and `ActionKey` receive the same duplicate diagnostics within their respective scopes.

Labels, rendered text, pointer positions, current display indexes, and hash-map iteration order are never identity. A display index can be an observation for layout or navigation, but it cannot be persisted as a key for a dynamic source.

## Caller, component, and runtime ownership

| Concern | Owner |
| --- | --- |
| Domain records, committed values, eligibility, and source revision | Caller |
| Stable source keys and field/action metadata | Caller-provided borrowed model/props |
| Cursor, draft, anchor, selection intent, and edit phase | Caller-owned component state |
| Control IDs, live geometry, focus, hover, pointer capture, and layer ownership | Termrock runtime and frame publication |
| Part names, validated style patches, and slot scope | Component contract and theme/author APIs |
| Key lookup and source reconciliation | Component update path, using this foundation's keys/revision rules |

Props borrow caller data for one update/draw/measure call. Durable state never borrows a model, stores a terminal rectangle, or caches a domain object by position. A component may derive child control IDs from the parent `Id` and a stable key. It must not mint a new identity from the row's current index each frame.

The runtime consumes `Id` for focus/hit/capture and records the owner in `Response<A>`. The component returns typed semantic actions; it does not inspect or mutate a global ID registry. See [`../foundations/runtime.md`](runtime.md) for frame ownership and [`../foundations/input-actions.md`](input-actions.md) for response flow.

## Revision and reconciliation lifecycle

For a dynamic source:

1. The caller accepts a source snapshot and advances its `Revision` whenever content or order changes.
2. The component receives borrowed props plus caller-owned durable state.
3. `update` reconciles keys against the new revision before interpreting input. It repairs stale cursor, selection, draft, and derived caches before returning an action.
4. Surviving keys retain their semantic target across reorder, insertion, filtering, rename, and refresh.
5. If the current cursor disappears, navigation may choose the nearest eligible successor, then predecessor, then `None`, according to that component's contract.
6. A pending activation, acknowledgement, capture, or edit draft never transfers to that fallback item. The caller must explicitly choose a new target.
7. `draw` reads the reconciled state and current props. It never performs reconciliation or advances a revision.

The caller must advance the revision before the next update/draw after mutating a borrowed source. A borrowed slice cannot enforce that discipline itself. Revision overflow is checked and reported as a typed error; wrapping to an old generation is not valid.

## Part identity and customization boundary

Standard parts are named by a family-scoped `Part`. `PartRef` carries the component owner and, for collection components, the relevant `ItemKey`. A part override is accepted only when that component advertises the part. An unsupported part is diagnosed rather than silently ignored.

Part identity does not grant access to runtime registries or arbitrary buffer coordinates. `patch_part` and `slot` remain constrained to the reserved rectangle and style/clip rules documented by the component. The theme owns semantic role resolution; this foundation only identifies the target.

## Required proof and parity coverage

The implementation and conformance suites must include:

- namespace/child collision negatives, including owner-versus-child ambiguity;
- duplicate live control IDs, row keys, column keys, field keys, and action keys rejected before geometry publication;
- reorder, insertion, filtering, and rename retaining the keyed command/selection target;
- removal of a captured target followed by reuse of its old display index, proving no stale capture retargets;
- source revision changes invalidating stale derived data and detecting conflicting edits;
- cursor removal selecting only the documented fallback while leaving pending activation/draft ownership unassigned;
- draw repetition leaving IDs, revisions, cursor, selection, and domain state unchanged;
- stable `PartRef` resolution with unsupported part overrides rejected.

Visual proof is indirect: this foundation has no independent hover/pressed screenshot. Components and composed Showcase/TablePro/Jackin Preview/Holla fixtures prove visible focus, selection, capture, and action-target effects against the frozen oracle. Headless, state, compile-fail, and protocol tests prove the nonvisual invariants. A candidate implementation never creates or blesses its own expected baseline.

## Implementation acceptance

P1 is complete only when the identity types are available behind the Termrock public facade, dynamic component contracts use stable keys and explicit revisions, duplicate identities fail closed, and reconciliation occurs in update before input interpretation. No future public compatibility requirement is created for the current `junie_tui`/`WidgetId` spelling. `WidgetId` may remain as a temporary private migration name while source files are refactored, but documentation and new examples use the target types.
