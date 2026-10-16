# Showcase: navigation tree and constrained cell editors

Plan for the product-owner request "improve the showcase": (1) group the
pages in a tree in the sidebar, tables first among equals; (2) show
editable table columns that only accept a fixed set of values. Revised
after a standards review (Storybook, Material, Apple HIG, WAI-ARIA APG,
Windows UX mouse guidance); the review's rules are listed in
[Standards applied](#standards-applied).

The behaviour lives in the library (`src/widgets`), the showcase only
configures and demonstrates it (AGENTS.md: component library first).

## 1. Sidebar information architecture

The sidebar is the library `TreeView`, not a hand-rolled list. It is at
most two levels deep: Overview is the one top-level page, every other page
is a leaf under exactly one group, and no group holds a page with the
group's own name.

| Group | Pages (tree order) |
| --- | --- |
| — | Overview |
| App shell | Chrome |
| Inputs & forms | Buttons, Inputs, Text areas, Chips & selects, Forms |
| Tables | Basic table, Editable, Data grid, Selection, Cell editors |
| Navigation | Lists, Trees, Sidebars |
| Overlays | Dialogs, Pickers |
| Feedback | Progress |
| Layout | Panels, Scrolling |
| Text & code | Code editor, Diff, Terminal |
| Screens | Settings, Task runner |

Rules:

- **Stable addresses.** Page ids and every earlier `--page` name keep
  working. Renamed labels keep their old names as aliases: `tables`
  (Basic table), `editabletables` (Editable), `selectabletable`
  (Selection), plus the existing `selection`, `bulkedit`, `selectable`.
  The new page answers to `celleditors`, `optioneditors`, `options`,
  `enums` and `choices`. Group names are not pages and are not `--page`
  targets. The header breadcrumb reads `/ group / page` (`/ Overview` for
  the top-level page).
- **Tree order is page order.** `[` / `]` walk the pages in tree order
  (crossing groups, open or closed) and wrap.
- **Expansion.** `NavExpand` policy on the shell (`--nav
  current|accordion|all`), default `CurrentGroup`: only the current page's
  group is open at start. A group navigation opened on its own closes again
  when navigation moves to another group; once the user opens or closes it,
  it stays as left. `Accordion` keeps only the current group open after
  every navigation; `All` starts with every group open. Navigating to a
  page (keys, `[`/`]`, `--page`, click) always opens its group so the
  current page is visible.
- **Keyboard (macOS outline view).** ↑/↓ (and `j`/`k`) move the cursor,
  → opens a group or steps into it, ← closes it or moves to the parent,
  Enter/Space on a group toggles it, Enter/Space on a page opens it, → on a
  page opens it and moves focus into the page, Home/End/PageUp/PageDown,
  `*` / `-` open / close all.
- **Type-ahead.** A typed letter jumps to the next row whose label starts
  with the prefix (library `TreeView::type_ahead`, 1 s reset, wraps), after
  the page's own shortcuts and the shell's letters (`q`, `i`, `?`, `[`,
  `]`, `0`) decline it.
- **Type-to-filter.** `/` opens a query row at the top of the sidebar;
  typing narrows the tree to matching pages (their groups stay as context,
  `TreeView::set_filter`), ↑/↓ move, Enter opens the page under the cursor
  (or the first match), Esc clears.
- **Mouse.** The sidebar is navigation, so one click on a page opens it
  (data views select on one click instead, see below). A click on a
  disclosure or a group row folds the group; the wheel scrolls; the
  scrollbar takes the last column inside the card when the tree overflows.
- **Panel.** The tree sits in a "Components" card (`Panel::card`, the
  page count as meta) like every other pane: one column of padding on each
  side (`Panel::pad_x(1)`, so labels keep their room at the narrow width
  and the scrollbar never touches the card's edge), the `/` query row
  inside it, focus-within on the title (`PanelFocus::Title`) and the `▎`
  only on the cursor row. A click on the card's padding or title focuses
  the tree and the wheel there scrolls it. The card's edge is the constant
  divider at every size, so the tree keeps no divider column of its own.
- **States.** One focus indicator: the `▎` gutter on the cursor row while
  the sidebar has focus, none otherwise. The current page is a fact: a `›`
  marker in its disclosure slot (`TreeView::selected_marker`), so it never
  relies on colour alone; accent only while the sidebar holds focus,
  primary ink otherwise. Hover and press come from the shared row style.

## 2. Constrained cell editors

One per-column editor API shared by `DataTable` and `DataGrid`:

```rust
ColumnEditor::{ Text, Select(Options), Combo(Options), Radio(Options),
                Checkbox(Options), Segmented(Options), Tags(Options) }
Options = [OptionItem { value, label, tone, description, disabled: Option<reason> }]
ColumnEditor::for_options(options)   // picks by the option-count ladder
```

### Option-count ladder

| Options | Editor | Why |
| --- | --- | --- |
| 2 (a boolean) | Checkbox: `[✓]` / `[ ]` in the cell, flips in place | one key, no popup; a checkbox, not a switch (HIG: switches are for settings that apply at once, not for data) |
| 2–5 | Segmented strip, or a Radio popover when values need descriptions | every choice visible at once |
| 6–15 | Select dropdown | a short scannable list |
| more than 15 | Combo: a search line over a filtered list (APG combobox, list autocomplete) | scanning stops working; typing is faster |
| any number, several at once | Tags checklist | multi-select from a set |

`ColumnEditor::for_options` applies the ladder (Radio and Tags are chosen
explicitly).

### Behaviour (library, `EditorConfig`, defaults follow macOS and APG)

- **Opening.** Enter, F2, Space and Alt+↓ open the editor on the cursor
  cell (plain ↓ keeps moving between rows: in a grid the arrows belong to
  cell navigation, APG grid pattern). A click on the current cell or on
  its `▾` opens it; one click on another cell only moves the cursor
  (select on first click, edit on second).
- **In the popup.** ↑/↓ move and skip disabled options, Home/End,
  type-ahead on labels (1 s reset, wraps around), Enter or Space chooses,
  Tab chooses and moves to the next editable cell (Shift+Tab back), Esc
  cancels; the same pipeline as a text cell.
- **Mouse.** Click an option to choose it; a click outside dismisses per
  the `popup::Dismiss` policy (default: Esc or outside click cancels).
- **Overlay.** Every popup goes through `RenderCtx::overlay`, anchored to
  the cell with `popup::place_with`: drawn above the page, flipping above
  or shrinking at the screen edge, following horizontal scrolling (the
  anchor is the cell as drawn this frame).
- **Cursor row.** The popup's cursor row is a fill (`CursorStyle::Fill`):
  the table keeps the only focus bar. The current value always carries a
  glyph besides any fill (`›`, `(●)`, `[✓]`, or bold underlined on a
  segmented strip), never colour alone (WCAG 1.4.1, 1.4.11).
- **Disabled options.** Drawn disabled with the reason in place of the
  description; never chosen by keys, type-ahead or click.
- **Affordance.** `EditorConfig::affordance` (`Affordance::Focused`
  default): the `▾` shows only on the focused cursor cell and the hovered
  editable cell; `Always` shows it on every editable option cell; `Never`
  hides it. Read-only cells never show it.
- **Status pills are read-only.** A toned value (a status pill) in a
  read-only column has no `▾` and no hover underline; option tones on
  editable columns colour the value but the cell keeps the editable
  affordance, so the two never look alike.
- **Out-of-set values.** Imported data whose value is not an option is
  shown as stored, in the error tone with a `!`, and reported by
  `out_of_set_cells()`; the editor opens on it with no current value, and
  choosing an option fixes it.
- **Validation.** Commits run the table's validator closure (DataTable)
  or the grid's validator and pending-change queue (DataGrid); a rejected
  choice keeps the popup open with the message. An open popup ends on blur
  per `EditorConfig::blur` (default `Cancel`: a macOS popup closes without
  a choice when it loses focus).

## 3. Showcase changes

- The tree sidebar (above).
- New page **Cell editors** (Tables group): a deployment-targets table
  with environment (Select), region (Combo with search), rollout
  (Segmented), owner (Combo), notify (Checkbox), labels (Tags), priority
  (Radio with descriptions and a disabled option), a read-only id and a
  read-only status pill, one imported row with an out-of-set value, and a
  legend explaining each editor.
- **Editable** gains a constrained column; **Data grid** wires its enum
  column to a Select editor.

## Standards applied

1. No group holds a page with its own name; Overview is top level; at most
   two levels; only the current page's group is open by default;
   type-to-filter.
2. The option-count ladder above; a two-value cell is a checkbox, not a
   toggle switch.
3. `▾` only on the focused or hovered editable cell by default.
4. Read-only status pills never look editable.
5. Selected and current items carry a non-colour cue (a glyph or bold) in
   addition to any fill.
6. Editors open with Enter, F2, Space and Alt+↓; Tab in an open list
   chooses and moves on; type-ahead is on in option lists.
7. Navigation lists (the sidebar) open on one click; data views select on
   one click and edit or activate on a second click, double click or Enter.
8. Selection: changing the selected rows commits the editor's edits (as
   blur commits a field, `LeaveEdits::Commit`); only edits that fail
   validation ask Save / Don't Save / Cancel. Terminals often keep
   Shift+click for their own text selection, so a range also grows from
   the clicked row with Shift+↑↓ (the footer says so).

## Deferred

- Plain ↓ opening a closed editor (APG combobox) is not applied inside
  tables: there ↓ moves between rows (APG grid); Alt+↓ opens instead.
- Bulk apply of a popup choice to every selected row from the table
  itself; the Selection page's bulk form covers it.
- A free-text "create option" mode for Combo (the request is fixed sets).
- DataTable in single-selection, row mode still activates a row on its
  first click (`TableEvent::Activated`); "select on one click, activate on
  a double click or Enter" for every data view is a cross-app change to
  the table's click model (tablepro, holla and jackin use it) and belongs
  with that work, not with the cell editors, which already follow it
  (first click moves the cursor, a second click or the `▾` opens).
- Space opens a constrained cell in DataTable but not in DataGrid, where
  Space marks the row (the grid's documented selection key); Enter, F2
  and Alt+↓ open there.
