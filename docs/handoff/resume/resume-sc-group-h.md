> **Status 2026-10-18: historical, landed.** SC17-SC20 are done (plan §7.8 status table). Kept for the design notes only; nothing to resume.

I stopped as asked. Nothing was pushed. I had made no code changes yet, so there was nothing to discard and I never took the push lock.

**State:** worktree `/Users/donbeave/Projects/terminal-components-claude/.claude/worktrees/agent-a5c0f7fa0394c2974` is on branch `sc-group-h` at `9ae6afc7e` (origin/design/interaction-polish), with a clean tree.

**Resume at SC17 (Dialogs). SC17–SC20 are all still to do.** `remaining_gaps` on that base shows SC17 36, SC18 54, SC19 101, SC20 36 and SC31 5.

What I found while reading, to save the next agent the time:

- **Dialogs page (SC17):** `pages/dialogs.rs` is still the old page, not on the template. Dialogs open through `cx.open(Dialog)` and the shell owns the only dialog slot. The shell draws it with `d.render_in(area, body_bounds(area, 1, 1))`, so the `chrome`, `placement` and `anchor` options take effect through that call.
- **Dialog kinds (SC18):** this page needs its own `ShellLayers<LayerResult>` to host `FormDialog`, `ChoiceDialog`, `InfoDialog` and `HelpOverlay`, the way `pages/pickers.rs` already does. That routing would then exist in two pages. To avoid duplicating it (CLAUDE.md treats duplicated routing as a defect), move it into one shared showcase helper used by both pages until S6/S7 move it into the library.
- **Click routing on that helper:** under `begin_modal`, every click the page receives belongs to the layer. But `Layer::owns` is `false` for every modal except Picker and ContextMenu, so copying the picker's "click not owned → dismiss" branch would dismiss on every click in a form or dialog. Route all clicks to the top layer instead. Outside clicks never reach the page in the showcase (S5 deviation 5), so leave that to S6.
- **Layer API lines:** about 40 lines (`ui::layer::*`, `ui::layers::LayerResult*`, `Frame::overlay`, `Dismiss::focus_out`) are allowlisted to SC19 with the reason "layer API (plan S5)". Plan §7.8 says SC31 should show `focus_out`, `LayerSpec` and `ShellLayers`, so re-point those lines to SC31 with that reason rather than demo them on Popups & tooltips.
- **`Dialog::with_actions`:** it is a deprecated alias. The precedent is `TextInput::type_to_edit`, allowlisted as `permanent` ("deprecated alias of …, shown through …").
- **Doc comment bug:** in `src/widgets/dialog.rs` around lines 656–663, the comment "Set how the dialog closes without an action." sits above `title_status` instead of `dismiss`. It is a small fix to make in SC17.
- **States grids:** `Specimen` exists only for Button, TextInput, Tabs and ContextMenu among the relevant widgets. SC17 needs one for `Dialog`, SC20 one for `Picker` (Loading and Error rows close `state::loading` and `SpecimenState::Loading`), and SC18 one for at least `ChoiceDialog` or `InfoDialog`. The model to copy is the ContextMenu impl at `src/ui/specimen.rs:847`.
- **Templates to follow:** `pages/menus.rs` (an `Opts` struct, `option_value`/`set_option`, overlay rendering) and the SC14 notes in plan §7.8 (`Template::sync_options`, `Request::SetOption`, `popup_open` feeding `layer_open`).
- **Tooling:** in this environment `rtk` condenses long tool output. Read files in chunks of about 110 lines with `sed -n` to get them in full.