# Shared picker baseline policies

Frozen visual oracle: `visual-baseline` peeled commit `4169783ae883788186d161af09ba9c481bd84e5c`.

This change puts three reusable picker policies in the shared layer:

- `Picker::surface(Surface)` chooses the surface passed to shared overlay chrome. Default remains `Surface::Overlay`; `row()` forwards it.
- `CursorStyle::{Bar, Fill}` is exported from `termrock-theme` and the `termrock` facade. `Bar` remains the default. `Fill` retains focus flags and uses the selection background and foreground, leaving the reserved gutter blank. `FilterList` and `Picker` share this policy; no app-side focus styling is required.
- `metadata_gap(u16)` defaults to `1` and flows from `Picker` through `FilterList` to `ItemColumns`. Existing fixed spacing remains unchanged for the default value; the requested gap is used when metadata columns exist. A zero gap changes only the spacer and never removes the independent icon glyph track.

Source paths confirmed in this workspace: `crates/termrock-overlays/src/picker.rs`, `crates/termrock-navigation/src/filter_list.rs`, `crates/termrock-navigation/src/item.rs`, and `crates/termrock-runtime/src/rowui.rs`. The collection/navigation and RowUi crate names differ from the original task's shorthand.

Focused regression coverage fingerprints every cell and cursor from the pre-policy constructor-default implementation at truecolor, 256-color, 16-color, and monochrome levels, then compares the new constructor defaults against that frozen fixture. It also checks that fill selection is a style default (so authored theme, overlay, and owner patches keep precedence), the focused gutter remains blank, zero-width metadata spacing keeps the icon glyph, metadata-free rows ignore metadata gaps, and rich Unicode rows retain all metadata.

This policy test does not establish end-to-end 1:1 parity for any application. The full paired `tuiscotti` baseline suite remains the acceptance gate.
