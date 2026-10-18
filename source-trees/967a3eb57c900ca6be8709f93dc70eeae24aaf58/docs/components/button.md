# Button

Status: current/proposed split. `Button::new`, `variant`, `disabled`, `update`, and `draw` are source-checked at `4d117b48`; all other builders and the toggle/check semantics below are the proposed target.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W02 · Group: Controls · Phase: P2.

## Purpose and exclusions

`Button` is a one-row action control. The caller supplies label, variant, checked value, and eligibility. Termrock supplies stable identity, focus/hit/capture integration, semantic styles, and the shared 140 ms activation feedback. A toggle variant shares this action surface while keeping its controlled value external.

Exclusions:

- No universal boxed-button recipe; the baseline is a one-row label with a focus gutter and no box.
- No domain mutation, persistence, command routing, or application-specific busy policy.
- No `WidgetId`, `Outcome`, or `RenderCtx` requirement in the future API. Those are current legacy implementation names only.

## Public API

Current source-checked declarations at `4d117b48` (`crates/termrock-controls/src/button.rs`):

```rust
Button::new(id, label)   // button.rs:173
variant(...)             // button.rs:194
disabled(...)            // button.rs:201
update(...)              // button.rs:280
draw(...)                // button.rs:362
```

Proposed target on top of the current declarations:

```rust
Button::new(id: Id, label: &'a str) -> Button<'a>

button.update(&mut cx) -> Response<Activated>
button.draw(&mut ui, area) -> Rect
button.measure(&measure_cx, constraints) -> Size
```

Builders:

```rust
variant(ButtonVariant) // Secondary default; Primary, Subtle, Danger, Toggle, Quiet, Ghost
disabled(bool)
status(ControlStatus)
checked(Option<bool>)
icon(Option<Glyph>)
autofocus(bool)
patch(StylePatch)
patch_part(Part, StylePatch)
slot(Part, &SlotPainter)
```

All props are borrowed or copied configuration. No `ButtonState` exists merely to duplicate runtime focus, hover, press, or feedback. The typed action is `Activated { origin: ActivationOrigin }`. A checked value is controlled input; the action tells the caller what to update when the variant is toggle-like.

Update, draw, and measure:

- `update` handles Enter, Space, and completed pointer clicks. It emits one activation only when enabled and not busy. It requests one-shot autofocus for a mounted identity; it never steals focus because of draw, re-enable, repaint, or overlay coverage.
- `draw` paints one row from semantic parts and runtime state. It sets focus gutter, icon/marker, label, and trailing padding inside the clipped rectangle. It does not toggle the controlled value, request focus, or run validation.
- `measure` accounts for gutter, optional icon/marker, label, and padding using display-cell width. Zero/tiny areas return clipped sizes without writes outside the allocation.

## Ordinary use

Consumer recipe: [EX-01 — A button with one owner](../api/consumer-recipes.md) (`current_source_checked`).

Showcase button and chrome pages exercise the baseline variants and interactions.

Ordinary use constructs `Button::new(id, label)` with a stable `Id`, sets `variant` and `disabled`, and handles one typed `Activated` action per eligible gesture. The caller applies its own domain effect and, for the toggle variant, its own controlled checked value.

## Ownership

`Id` stays stable while a button remains mounted. Focus, hover, capture, and activation feedback are keyed by that identity. Removing or disabling a button cancels capture and feedback. Reordering buttons preserves identity and cannot retarget a held press. Autofocus is one-shot per mounted identity and is resolved in update through the runtime focus owner.

Shared dependencies:

- [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md) — IDs, key/pointer actions, typed activation origin.
- [`runtime`](../foundations/runtime.md) — focus, hit, capture, hover suppression, autofocus, and time.
- [`layout`](../foundations/layout.md) — one-row measurement and clipping.
- [`theme`](../foundations/theme.md) — semantic variant and disabled styles.
- [`author`](../foundations/author.md) — constrained part/slot customization.
- [`conformance`](../foundations/conformance.md) — exact oracle comparison and mutation gates.

## Customization

Parts are `container`, `gutter`, `icon`, `marker`, and `label`. [`theme`](../foundations/theme.md) defines variant precedence and capability fallback. Parts may accept explicit style patches or slots; geometry, focus/capture ownership, and whole-surface replacement are not customizable. Preserve baseline label padding, no-box silhouette, marker glyphs, variant surfaces, gutter, clipping, disabled treatment, and busy spinner treatment. Focus and hover remain independent: focus shows the gutter; hover lifts the surface; focus+hover shows both.

The baseline one-row width is the label plus two padding cells, with two more
cells when a toggle marker or busy spinner occupies its own slot. Primary is
bold on-accent over the primary fill; hover uses `accent-hover`, and press
uses `accent-pressed` with the baseline reversed foreground. Secondary and
toggle use the overlay surface and lift to popover on hover; subtle uses
secondary text on its inherited container and lifts one surface plane. Danger
uses the error-soft label on overlay at rest and white-on-error while pressed.
Focus adds the gutter and bold label; the primary is already bold, so its
focus cue is only the gutter. Busy uses a spinner plus secondary label and
rejects activation; disabled is faint and has no hover. Button is not boxed,
and a single decision row has at most one primary action. A danger label ends
in `…` when it opens a follow-up dialog; terminal actions do not.

Ordinary example: `Button::new(id, "Save").variant(ButtonVariant::Primary)`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Enter and Space activate exactly once when eligible. Other keys pass to the runtime. |
| Pointer | Press inside captures the clipped control. Release inside activates once; release outside cancels. Down alone never activates. |
| Focus/hover | Runtime owns focus and hover. Keyboard input suppresses stale hover until pointer motion. Held press has its own visual state. |
| Disabled/read-only | Disabled or blocked/busy controls consume or ignore activation, clear hover/pressed state, and emit no action. Read-only is represented by caller eligibility/status; it is not an implicit toggle. |
| Checked | Checked, focused, hovered, held-pressed, and activation feedback are independent axes. |
| Resize/Unicode | Re-measure, clip by display cells, and preserve combining/wide glyph correctness. |
| Capability/color | Resolve semantic variant and disabled colors through the theme for truecolor, 256, 16, none, and nocolor. |
| Motion | Feedback lasts 140 ms. Verify 0, 139, 140, and 141 ms boundaries and expiry. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | pointer down, held press, release inside, release outside, removed target, disabled target, keyboard activation, feedback active/expired |
| Value/status | checked and unchecked, each variant, disabled, busy, blocked |

Apply only meaningful axes to each variant. Run dimensions 72×20, 80×24, 100×30, 120×40, 160×50 and capabilities truecolor, 256, 16, none, nocolor.

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Starting implementation is [`src/widgets/button.rs`](../../src/widgets/button.rs), legacy `Button`, source blob `d2c52e75084ff783a9b3bf5d95f736870f480051b`.

Use [`capture-plans/button.json`](../reference/capture-plans/button.json) (W02, planned and uncaptured). Expected output must be bound from ExistingOracle or ExtractedOracle before candidate comparison; candidate code never creates the oracle.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W02-01 | Every variant at normal, focus, hover, focus+hover, and disabled. |
| W02-02 | Pointer down inside; release outside; repeat with target removed or disabled. |
| W02-03 | Successful keyboard and pointer activation at 0, 139, 140, and 141 ms after feedback starts. |
| W02-04 | Checked but unfocused; unchecked but focused; busy activation rejection. |
| W02-05 | Combining characters, CJK, and empty labels. |

Capture exact cells, cursor position, focus/capture/layer owner, action count and target, and controlled value before/after. Do not approve missing or fabricated oracle setup.

Negative tests:

- Pointer down or release outside never emits activation.
- Disabled, busy, removed, or covered buttons cannot emit activation or retain capture.
- Repeated draw, re-enable, and autofocus repaint cannot steal modal focus.
- One gesture yields at most one typed action; toggle state does not change unless caller applies it.
- `draw` cannot mutate checked value or start timers; `measure` cannot mutate semantics.
- Part slots cannot bypass clipping or replace focus/capture ownership.

Accept after the external API example, source-state tests, exact snapshots, timing boundaries, Unicode cases, and independent review all pass.

## Rejected use

Forbidden: painting a padded label as a fake button and toggling an app flag on click instead of using Button.

```rust
// Forbidden: preview paints its own button and owns the toggle.
ui.fill(row, 0, 12, style);
ui.paint_str(row, 2, "[ Save ]");
if clicked { saved = true; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.fill`/`ui.paint_str` rendering of controls and padded strings that simulate buttons. Use `Button::new` and handle the typed `Activated` action.

## Known gaps

- Capture plan W02 is planned and uncaptured; no expected artifacts are bound.
- Signature refinements of existing builders are future work on `termrock-implementation`: `icon`, `autofocus`, `status`, `checked`, `patch`, `patch_part`, `slot`, and `measure` already exist at `4d117b48` (`button.rs:208/217/224/231/238/245/252/516`) but their proposed shapes differ (for example current `autofocus()` takes no flag and `checked(bool)` takes no option).
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): only `new`, `variant`, `disabled`, `update`, and `draw` match the proposed target exactly; reconcile one signature per builder before implementation.
