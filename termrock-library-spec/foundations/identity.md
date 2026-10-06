# F01 · Identity and revisions

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C01  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/core/id.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/id.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

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

## Contract

1. Id is control identity; ItemKey is source identity; ColumnKey identifies a column; ActionKey identifies behavior. Do not interchange these types or use labels as keys.
2. Use structured/length-delimited identity composition. A hash may optimize lookup but must not silently define equality across concatenation collisions. Duplicate IDs/keys are diagnosed before publishing interactive geometry; production behavior fails closed rather than dispatching ambiguously.
3. Keys survive reorder, insertions, filtering, renaming and refresh. Positional keys are forbidden for dynamic collections; no implicit ByIndex fallback.
4. Revision means accepted source content/order changed. Constructors for dynamic sources require it, or read it from a source trait. A caller that mutates data must advance revision before update/draw; this remains a caller contract, not a claim that a borrowed slice can enforce revision discipline by itself.
5. Reconciliation happens during update. Retain surviving cursor/selection anchors; when a cursor is removed choose the nearest eligible successor, then predecessor, then None. Never transfer pending activation/edit ownership to that fallback item.

## Required proof

- namespace/child collision negative test
- duplicate live ID, duplicate row key and duplicate column key rejected
- rename/reorder retains key and command target
- remove captured target then reuse old display index
- source revision change invalidates stale derived data

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
