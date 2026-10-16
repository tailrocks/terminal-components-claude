**(1) Focus hierarchy rule.** Each state gets exactly one treatment.

- **Keyboard focus:** a single `▎` in `t.focus`, drawn through `t.gutter_symbol`/`t.gutter`. It appears once per screen, inside the topmost layer. When a popup list has an input that owns it (picker query, completion's editor, select field), the bar belongs to that owner.
- **Cursor row inside a focused popup list** (picker, completion, select, menu): a fill with no bar. Use the `t.highlight`/`t.highlight_danger` fill with BOLD, the way `menu.rs:323-335` already does. Do not use `accent_bg`, so green stays reserved for focus.
- **Cursor row in a focused in-page list:** that row is the focus stop, so `▎` plus BOLD. The `accent_bg` tint is optional.
- **Selected or current item in an unfocused control** (active tab, chosen list item, current nav page, selected tree node): neutral. Use `t.lift(bg)` plane, `text_primary`, BOLD, markers in `text_secondary`, rule in `border_strong`. Never `accent` or `focus`.
- **Containers:** focus-within shows as title `t.label(true)` or `t.border(true)`. Never a `▎`.
- **Under a modal or popup layer:** the page renders with `focus_hidden=true`. No `▎`, no `accent_bg`, no BOLD. `Theme::backdrop` collapses accent to `text_faint`.
- **Data states** (spinners, ✓ done, meters) may stay accent. They are not position indicators.

**(2) Findings.** Snapshot paths are relative to `/Users/donbeave/Projects/terminal-components-claude/snapshots/`, size 120x40 unless noted. I checked them with a scan of the truecolor.ansi files for green `▎`/`━`/`accent_bg` cells, plus the PNGs.

1. **HIGH** | `src/widgets/picker.rs:436-449` (field `▎` hardcoded at 447) + `picker.rs:567-577` (`s.focused = i == self.cursor` gives the row `t.gutter_symbol`)
   - Screens: `jackin/capsule/new_tab`, `jackin/capsule/palette`, `jackin/capsule/split_vertical`, `tablepro/overlays/picker_open`, `tablist_open`, `tablist_tables`, `safemode_picker`, `showcase/flows/pickers/open`, `showcase/flows/pickers/tabs`
   - Shows: `▎ Type to search…` and `▎▪ Claude Code`, two identical green bars a row apart. In `nocolor.txt` they are indistinguishable.
   - Fix: add a shared `Theme::cursor_row(bg, danger) -> Style` (highlight fill + BOLD, no gutter). Picker rows use it and draw a blank gutter cell. The field uses `t.gutter_symbol/gutter(VisualState{focused:true})` instead of the literal. A non-searchable picker (safemode) also uses the fill only, matching the menu.

2. **HIGH** | `src/ui/ctx.rs:31-41`: `Interaction::focus_hidden` is read but never set to `true` anywhere. Only `begin_modal` (ctx.rs:179) and app init write it, and both write `false`.
   - Screens: `tablepro/overlays/picker_open` (txt line 9 `│ ▎  ▾ S public` under the modal, so 3 bars), `tablepro/overlays/safemode_picker`
   - Shows: page widgets under a modal still render as focused (`▎`, `accent_bg` tint, BOLD). Only colour dims them, so in nocolor and 16-colour they look identical to the modal's focus.
   - Fix: `modal_backdrop` and every modal path set `interaction.focus_hidden = true` before the page draws. Apps know their modal stack at frame start, so add a helper such as `ctx.page_under_layer(open: bool)`, and `begin_modal` resets it as it does now. Also make `RenderCtx::state()` return `focused=false` while `ctx.inert`.

3. **HIGH** | Non-modal keyboard popups leave page focus at full phosphor. Causes: the menu-bar dropdown and ContextMenu in `src/widgets/menu.rs` never hide the layer below; `src/bin/holla/app.rs:3007-3009` uses `Modal::Menu` with `begin_modal` and no backdrop.
   - Screens: `jackin/manager/menu_open` (green `▎` + `accent_bg` row under the File menu), `showcase/flows/chrome/context` (panel `▎Sessions` + row `▎ 2 Codex` + blue menu cursor), `holla/flows/menu/file` (query `▎` + green tab `━` + green row tint + menu cursor)
   - Shows: three or four "you are here" cues while the keyboard is in the menu.
   - Fix: the popup layer sets `focus_hidden` for content below it. Add a ContextMenu/MenuBar knob `hide_owner_focus: bool`, default `true`. For holla `Modal::Menu`, use `modal_backdrop` instead of `begin_modal`.

4. **HIGH** | `src/widgets/tabs.rs:436-445`: the active-tab rule is `t.accent` whenever `!quiet`, ignoring strip focus. Only tablepro's mode and structure tabs set `quiet` (`src/bin/tablepro/tabs.rs:408,421`); jackin editor and settings force `quiet=false` (`editor.rs:2084`, `settings.rs:1169`).
   - Screens: `tablepro/query/results` (two green rules plus the editor `▎`), `tablepro/query/*`, `tablepro/table/*`, `tablepro/ack/executed`, almost every `holla/**` (row 3 rule plus `▎` on row 4, e.g. `holla/concept/first-use`), `jackin/editor/{auth,env,mounts,roles}`, `jackin/settings/{agents,env,mounts,trust}`, `showcase/flows/settings/members`
   - Shows: the active tab looks focused while focus is elsewhere.
   - Fix: replace `quiet: bool` with `TabRule { Auto (default), Accent, Neutral }`. `Auto` uses accent only when `ctx.interaction.focused(self.id)`, otherwise `t.border_strong`. Update the test at tabs.rs:523-546.
   - Superseded (2026-10-16, product owner): the selected tab is a selection, not focus. macOS draws a tab view's or segmented control's selection in the accent while its window is active, so `TabRule::Auto` is now the accent unless a modal or popup covers the strip (`page_under_layer`), and the old rule is `TabRule::FocusOnly`. Focus on the strip moved off the rule onto the single `▎` gutter in the cursor tab's leading cell (plus weight), so a focused strip whose cursor is on the active tab still shows focus. tablepro's mode and structure strips drop their `Neutral` override (they are segmented choices of the front page); jackin's usage dialog uses the default rule and draws the strip as focused while its tab bar holds the keys.

5. **MED** | `src/widgets/panel.rs:115-118`: a focused card panel draws `▎` on its title, contradicting its own doc comment at panel.rs:6-7, while the inner control draws its own `▎`.
   - Screens: `showcase/flows/lists/moved` (`▎Language` + `▎  Python`), `showcase/flows/datagrid/*`, `showcase/flows/editable/*`, `showcase/flows/tables/selected`, `showcase/flows/trees/expanded`, `showcase/flows/chips/toggled`, `showcase/flows/editor/completion`, `showcase/flows/settings/members`
   - Fix: drop the bar. Focus-within becomes title `t.label(true)` (card) or `t.border(true)` (framed). Optional knob `PanelFocus::{Title (default), Bar}`.

6. **MED** | `src/widgets/completion.rs:186-195`: completion rows use the gutter bar while the editor keeps its own.
   - Screens: `showcase/flows/editor/completion` (panel, editor line and completion row all show `▎`; 3 bars), `tablepro/query/completion`, `tablepro/query/completion_columns`
   - Fix: use `cursor_row` from #1. The editor keeps the only `▎`.

7. **MED** | `src/widgets/select.rs:378-389`: dropdown rows draw `gutter_symbol(rs)` with an `accent_bg` tint while the field at :297-302 keeps its `▎`. The chosen `›` is always `t.accent`.
   - Screens: `showcase/flows/chips/select_open` (bars at rows 14 and 16)
   - Fix: rows use `cursor_row`; the chosen `›` uses `text_secondary`.

8. **MED** | `src/theme.rs:347` (`row()`: `selected && focused` gives `accent_bg`) and `src/bin/holla/screens/finder.rs:946-949` (cursor row gets `selected` plus the green tint and an accent `›`, while the query row holds the `▎`)
   - Screens: `holla/concept/first-use` and most `holla/flows/*`, which pair query `▎`, green tab `━` and a green-tinted row
   - Shows: green is used for both focus and selection.
   - Fix: finder's subordinate cursor row uses `cursor_row` (non-green fill). Restrict `accent_bg` in `row()` to the row that also carries the `▎`, or drop it.

9. **MED** | `src/widgets/tree.rs:612`: the selected node label is `t.accent` regardless of tree focus.
   - Screens: tablepro explorer, jackin inspect
   - Fix: follow `list.rs:293-299` (accent only when focused, `text_secondary` or primary BOLD otherwise).

10. **MED** | `src/bin/showcase/app.rs:1067-1070`, `src/bin/showcase/pages/sidebars.rs:226,251`: the current-page `›` is always accent. `pages/overview.rs:166` codifies `› current / chosen` as accent.
    - Screens: `showcase/flows/editor/completion` (green `› Code editor` in the nav while focus is in the editor)
    - Fix: accent only while the nav is focused, else `text_secondary`. Update the overview legend.

11. **MED** | `src/bin/jackin_preview/screens/capsule.rs:2742-2748`: the usage tab strip is hand-rolled and draws a `▎` on a tab. This violates the rule at `tabs.rs:4`.
    - Fix: use the `Tabs` widget (#4 handles focus).

12. **LOW-MED** | `src/theme.rs:297-320` (`backdrop`): accent and focus map to `text_muted` (#808080; 16-colour index 7, silver). The `▎`/`━` glyphs survive, and `modal_backdrop` (`src/ui/ctx.rs:184-200`) strips modifiers but not glyphs.
    - Screens: `jackin/capsule/new_tab` (truecolor: the tab `━` is still a bold grey bar; `16.ansi` row 4 uses `38;5;7`), `tablepro/overlays/picker_open` (grey `▎` on `public`)
    - This explains the reported "active underline behind the picker": it still reads as active.
    - Fix: map `accent`/`focus` to `text_faint`. In the `modal_backdrop` loop, replace `▎` with a space.

13. **LOW** | Hardcoded bars that bypass `gutter_symbol`, so central suppression (#2, #3) cannot reach them: `picker.rs:447`, `menu.rs:651`, `panel.rs:117`, `capsule.rs:2742`, `src/bin/holla/screens/finder.rs:747`, `src/bin/holla/screens/files.rs:1640`.
    - Fix: route all through `t.gutter_symbol/gutter(VisualState)` built from `ctx.state`.

14. **LOW** | `src/widgets/code.rs:755`: the block marker `›` is accent while focused, placed beside the `▎`.
    - Screens: `tablepro/query/results`, `showcase/flows/editor/completion`
    - Fix: `text_primary`.

15. **LOW** | `src/widgets/picker.rs:458-463`: the query is underlined in accent on the same row as the field `▎`, adding a second green cue there.
    - Fix: use `text_secondary` as the underline colour, or none.

16. **LOW** | Checked or chosen value glyphs in accent compete with focus green: `src/widgets/choice.rs:101,235,324`, `src/bin/holla/screens/cleanup.rs:713`, `src/bin/holla/screens/plan.rs:285`.
    - Fix: accent only when the owning control is focused, otherwise `text_primary`.

17. **LOW** | Showcase specimen matrices deliberately show many `▎`.
    - Screens: `showcase/audit/buttons`, `showcase/flows/buttons/focus`, `showcase/flows/inputs/*` (col 44), `showcase/pages/overview`
    - Not a bug. Exclude them from the one-focus audit, or frame them as static specimens.

**Shared fix order:** #2 and #12 (layer focus hiding plus backdrop) first, then #1, #6, #7 and #8 (one `Theme::cursor_row` for all popup lists), then #4 (`TabRule::Auto`), then #5 (Panel without a bar).

No repo edits. The scan script and its output are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/audit-focus/` (`scan.py`, `all.txt`).