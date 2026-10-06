# F09 · Secret-aware fields and validation

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C19  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/widgets/input.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/input.rs)
- [src/widgets/textarea.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/textarea.rs)
- [src/widgets/dialog.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/dialog.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
Secret::new(value: String) -> Secret
Secret::expose<R>(&self, read: impl FnOnce(&str) -> R) -> R
Secret::clear(&mut self)
struct SecretPolicy { reveal_tail: u8, allow_copy: bool }
struct Plain;
struct SecretText;
struct TextInputState<Mode = Plain> { /* private, mode-dependent storage */ }
enum TextAction<V> { Edited, Commit { value: V }, Cancelled, Conflict }
struct ValidationMessage { code: &'static str, display: String }
struct FieldError { field: FieldKey, message: ValidationMessage }
trait Validator { fn validate(&self, value: &str) -> Result<(), ValidationMessage>; }
```

## Contract

1. Plain and secret editing share one text engine/painter but different storage/permission specializations. Secret and secret-bearing state have redacted Debug, no Clone, no general serialization and no plain draft getter.
2. Zeroize owned active/rollback secret drafts on replacement/cancel/drop using a reviewed zeroization primitive. This reduces owned-buffer exposure; it is not a guarantee that the OS, allocator or application never copied a secret.
3. Default secret policy is no copy and no tail reveal. The baseline synthetic four-character tail is an explicit oracle fixture policy. Never capture real credentials in conformance artifacts.
4. Validation is caller-supplied and deterministic. Standard messages never echo secret input. Domain validators must return safe display text; the type system cannot prove an arbitrary String contains no secret, so negative leak tests are required.
5. Commit transfers a Secret value to the caller without making the action freely cloneable. Edited is payload-free. Cancellation/hidden/disabled transitions clear drafts according to the documented mode policy.
6. Form validation decides whether Submit is eligible after field commits. A required/custom validation error is a separate visual state from editing/focus, not permission to perform persistence.

## Required proof

- compile-fail Clone/serde/ordinary draft access for secrets
- synthetic secret absent from Debug and all generated artifact text
- no copy/tail reveal without explicit policy
- cancel/drop/replacement invoke clearing path
- masked wide-grapheme hit mapping

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
