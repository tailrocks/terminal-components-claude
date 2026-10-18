# F09 · Secret-aware fields and validation

**Status:** canonical Termrock foundation contract; implementation is future work.

**Legacy families:** C19.

**Visual authority:** the unchanged `visual-baseline` commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` and its approved application output. This document defines ownership and safety rules; it does not authorize a visual redesign.

**Scope:** shared field, form, validation, and conformance infrastructure. Product authentication, persistence, credential retrieval, and policy decisions remain caller responsibilities.

This contract is implemented by the future in-place refactor on `termrock-implementation` in this repository. It is not a destination for a separate repository or a product-specific credential subsystem.

See the [public API contract](../api/public-api.md), [shared text contract](text.md), [component index](../components/README.md), and [conformance contract](../verification/conformance.md).

## Source evidence

The starting implementation is read-only evidence for the future refactor:

- [`src/widgets/input.rs`](../../src/widgets/input.rs) supplies the current single-line editing, validation, masking, reveal-tail, cancel, clear, paste, and grapheme hit behavior.
- [`src/widgets/textarea.rs`](../../src/widgets/textarea.rs) supplies the current multiline editing and field-specific Escape/paste policy.
- [`src/widgets/dialog.rs`](../../src/widgets/dialog.rs) supplies baseline form/overlay composition and focus behavior.
- The pinned source and approved snapshots remain the oracle. Current names such as `TextInput`, `TextBuffer`, and `validator: fn(&str) -> Option<String>` are implementation evidence, not the future Termrock API.

## Proposed public surface

These declarations describe the target API. They are not an implementation or a promise that the current crate already exports these types.

```rust
pub struct Secret { /* private, zeroized owned value */ }

impl Secret {
    pub fn new(value: String) -> Self;
    pub fn expose<R>(&self, read: impl FnOnce(&str) -> R) -> R;
    pub fn clear(&mut self);
}

pub struct SecretPolicy {
    pub reveal_tail: u8,
    pub allow_copy: bool,
}

pub struct Plain;
pub struct SecretText;

pub struct TextInputState<Mode = Plain> { /* private mode-specific storage */ }

pub enum TextAction<V> {
    Edited,
    Commit { value: V },
    Cancelled,
    Conflict,
}

pub struct ValidationMessage {
    pub code: &'static str,
    pub display: String,
}

pub struct FieldError {
    pub field: FieldKey,
    pub message: ValidationMessage,
}

pub trait Validator {
    fn validate(&self, value: &str) -> Result<(), ValidationMessage>;
}
```

`Secret` is intentionally opaque. `expose` is a short-lived read boundary, not a general getter. Secret-bearing states and actions must not acquire blanket `Clone`, ordinary `Debug`, or serialization implementations. The exact generic spelling may change during the P1 API freeze, but no change may weaken these guarantees.

## Ownership and data flow

1. The caller owns the committed domain value, its `FieldKey`, source revision, persistence, and whether a field is allowed to accept or reveal input.
2. The caller owns the component state object. The state may own an active draft and the rollback value, but it never borrows caller data and never owns a service, credential store, or persistence handle.
3. `update` owns edit, commit, cancel, validation-result, and conflict transitions. It returns one typed `TextAction<V>` through the common response path. `draw` receives immutable state and borrowed props; it may project a masked value and publish geometry/cursor facts but may not validate, commit, log, copy, or mutate the domain.
4. `Commit` transfers a `Secret` (or plain value) to the caller. The action must not make the secret freely cloneable or serializable. `Edited` carries no text payload, so observers cannot accidentally retain a draft.
5. A `Validator` receives a temporary `&str` and returns a safe `ValidationMessage`. It must not retain the input or place it in logs, diagnostics, snapshots, action labels, or error text. A validator cannot be assumed safe merely because it satisfies the trait; leak tests remain required.
6. Form-level validation decides whether submit is eligible after child fields have committed. It does not persist data and it does not turn a field error into a domain action. Required, invalid, editing, focused, disabled, and read-only are separate states.

## Secret storage and lifecycle

The implementation must use a reviewed zeroization primitive for every owned active or rollback secret buffer. Clear the old value before replacing it where the storage model permits; clear on cancel, explicit `clear`, drop, and any transition that removes secret access (for example a documented hidden or disabled transition). If a mode intentionally preserves a draft, that policy must be explicit, narrowly scoped, and tested rather than implied by a generic state transition.

Zeroization reduces exposure of Termrock-owned buffers. It cannot promise that the operating system, allocator, terminal, debugger, compiler, or caller never copied the bytes. The API therefore also avoids ordinary getters, clone/serialize derives, debug output, automatic clipboard operations, and secret-bearing snapshot labels.

The default policy is:

- do not copy a secret;
- reveal no tail while editing or unless the caller explicitly opts into a policy;
- do not include secret input in validation messages;
- permit a synthetic reveal policy only in a controlled oracle fixture;
- never capture real credentials in tests, snapshots, reports, or examples.

The frozen baseline's synthetic four-character tail is an explicit fixture policy. It is not a security default and must not be generalized to production credentials.

## Validation and interaction rules

- Plain and secret fields share the text engine, measurement, selection, and painter infrastructure, but use separate storage and permission specializations.
- Masked display maps each source grapheme to the documented mask glyph and display width. Pointer-to-cursor mapping uses the displayed geometry, including the mask's one-cell width, so a wide source grapheme cannot make a click target drift. The raw source never enters the frame or an observation artifact.
- Single-line and multiline fields retain their baseline differences. In the frozen implementation, single-line Escape restores its starting value while multiline Escape commits the document; single-line paste can begin editing while the baseline multiline paste requires editing already active. Shared text code must preserve these policies through explicit component configuration.
- Validation may run on commit and, when the component contract says so, while editing. A validation transition may update a caller-visible error state but cannot execute persistence or a product command.
- Read-only fields may be navigated and copied only under an explicit caller policy. Disabled fields do not accept focus, hover activation, editing, paste, or copy and must not leak their value through an underlying action.
- Conflict handling uses the source revision captured when editing began. A changed external revision preserves the active draft and reports `Conflict`; it does not silently overwrite the caller value or retarget to a new source. The caller chooses a documented keep-draft or reload resolution.

## Foundation boundaries

| Concern | Owner | Secret-specific rule |
| --- | --- | --- |
| Durable field state and draft | Caller-owned `TextInputState<Mode>` | Secret mode owns private, clearable buffers only |
| Committed value and revision | Caller | Commit transfers ownership through a typed action |
| Editing, validation, conflict | Text component/update phase | No persistence, IO, or hidden logging |
| Focus, hover, pointer capture, cursor arbitration | Runtime | Disabled/hidden secret fields cannot retain ownership |
| Glyphs, mask, reveal, error, disabled styling | Theme/component recipe | Reveals are explicit policy and oracle-labeled |
| Field identity and errors | Identity/response foundations | `FieldKey` is stable; text is never used as identity |
| Clipboard and external display | Caller/optional host adapter | No automatic copy; protected values require explicit policy |

The author surface may paint a documented field part, but it cannot read a private secret buffer, bypass masking, install a raw hit registry, or turn a slot into a copy path. The conformance registry owns leak and compile-fail checks; it is not a production secret manager.

## Required verification

The implementation phase must add these gates. They are acceptance requirements, not evidence that the current implementation already passes them.

### Type and lifecycle gates

- Compile-fail checks reject `Clone`, ordinary `Debug`, serde/serialization, and direct draft getters for secret state and actions.
- A synthetic secret is absent from debug output, error messages, action labels, rendered text, tui-snap artifacts, PTY transcripts, reports, and test failure diagnostics.
- Replacement, cancel, explicit clear, hidden/disabled access removal, and drop all exercise the clearing path. A test-only reviewed probe may verify the call without exposing the value.
- A validator returning unsafe-looking text is rejected by leak tests; standard required/custom messages remain safe and deterministic.
- Copy and tail reveal are denied by default, and explicit policies are tested independently. Real credentials never enter a conformance program.

### Behavior and visual gates

- Masked Unicode input has exact grapheme hit mapping at ASCII, wide, combining, and emoji boundaries.
- Editing, commit, cancel, conflict, invalid, disabled, read-only, focus, hover suppression, pointer capture, and cursor placement are compared through the owning component's applicable cases.
- The four applications retain their baseline field appearance and observable interactions. This foundation does not invent a standalone screenshot for a nonvisual security rule; component and composed fixture evidence proves its visible effects.
- Mutation tests that expose one secret character, permit an unintended copy, duplicate a commit, or route a stale action to a new field must fail.

## Migration note

The current baseline's public fields and `text()` method are intentionally more permissive than this target contract. The future refactor may use compatibility internals temporarily, but they must not become Termrock's public compatibility promise. This document is the canonical destination for the secret, validation, and field-safety rules; component pages link here instead of restating them.
