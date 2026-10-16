# Showcase: navigation tree and constrained table editors

Plan for the product-owner request "improve the showcase": (1) group every
page in a tree in the sidebar, tables first among equals; (2) show editable
table columns that only accept a fixed set of values.

The behaviour lives in the library (`src/widgets`), the showcase only
configures and demonstrates it (AGENTS.md: component library first).

## 1. Sidebar information architecture

The sidebar is the library `TreeView`, not a hand-rolled list. Every page is
a leaf under exactly one group; there are no top-level pages.

| Group | Pages (tree order) |
| --- | --- |
| Foundations | Overview, Chrome |
| Inputs & forms | Buttons, Inputs, Text areas, Chips & selects, Forms |
| Tables | Tables, Editable tables, Data grid, Selectable table, Option editors |
| Navigation | Lists, Trees, Sidebars |
| Overlays | Dialogs, Pickers |
| Feedback | Progress |
| Layout | Panels, Scrolling |
| Text & code | Code editor, Diff, Terminal |
| Screens | Settings, Task runner |

Rules:

- **Stable addresses.** Page ids, labels and therefore `--page` slugs do not
  change; the existing aliases (`selection`, `bulkedit`, `selectable`) stay.
  The new page adds `optioneditors` (label) plus `options`, `enums` and
  `choices`. Group names are not pages and are not `--page` targets.
- **Tree order is page order.** `[` / `]` walk the leaves in tree order
  (crossing groups, collapsed or not) and wrap.
- **Expansion.** `NavExpand` policy on the shell, default `CurrentGroup`:
  at start only the current page's group is open; whatever the user opens
  or closes stays that way. `Accordion` keeps only the current group open
  after each navigation; `All` starts with every group open. Whatever the
  policy, navigating to a page (keys, `[`/`]`, `--page`, click) expands its
  ancestors so the current page is always visible.
- **Keyboard (macOS outline view).** ↑/↓ (and `j`/`k`) move the cursor,
  → opens a group or steps into it, ← closes it or moves to the parent,
  Enter/Space on a group toggles it, Enter/Space on a page opens it, → on a
  page opens it and moves focus into the page (the long-standing shortcut),
  Home/End/PageUp/PageDown page the tree, `*` / `-` open / close all.
  **Type-ahead** jumps to the next row whose label starts with the typed
  prefix (library `TreeView::type_ahead`, off by default, on in the
  sidebar).
- **Mouse.** Click the `▸`/`▾` disclosure or a group row to toggle, click a
  page to open it; wheel scrolls; the divider column carries the scrollbar
  thumb when the tree overflows (library `TreeView::divider`).
- **States.** One focus indicator: the `▎` gutter on the cursor row while
  the sidebar has focus, none otherwise. The current page is a fact: a `›`
  marker in its disclosure slot (library `TreeView::selected_marker`), accent
  only while the sidebar holds focus, primary ink otherwise. Hover and press
  come from the shared row style.

## 2. Catalogue of constrained cell editors

One per-column editor API shared by `DataTable` and `DataGrid`:

```rust
ColumnEditor::{ Text, Select(Options), Combo(Options), Radio(Options),
                Toggle(Options), Segmented(Options), Tags(Options) }
Options = [OptionItem { value, label, tone, description, disabled: Option<reason> }]
```

| Editor | When | Opens with | Popup / inline | Commits |
| --- | --- | --- | --- | --- |
| Select | 3–15 values, short labels | Enter, F2, Space, click on the focused cell or its `▾` | list anchored under the cell (flips above) | Enter/Space/click on an option |
| Combo | many values (owners, regions) | same; typing starts the search | search line + filtered list | Enter on the highlighted match; never free text |
| Radio | few values that need a description | same | list with `( )`/`(•)` and a muted description | Enter/Space/click |
| Toggle | two values (on/off, yes/no) | Enter, Space, click | none: flips in place | immediately |
| Segmented | 2–4 short values (status) | same | one-row strip anchored to the cell | ←/→ then Enter, or click a segment |
| Tags | multi-select from a set (labels) | same | checklist anchored to the cell | Space toggles, Enter commits all, Esc cancels |

Shared behaviour (library, configurable through `EditorConfig`, defaults
follow macOS popup buttons and menus):

- **Keyboard.** ↑/↓ move and skip disabled options, Home/End, type-ahead on
  labels (Select, Radio, Tags), Enter commits, Esc cancels, Tab commits and
  moves to the next editable cell (Shift+Tab back), the same pipeline as a
  text cell.
- **Mouse.** Click an option to choose it; a click outside dismisses per the
  `popup::Dismiss` policy (default: Esc or outside click cancels).
- **Overlay.** Every popup goes through `RenderCtx::overlay`, anchored to the
  cell with `popup::place_with`, so it is drawn above the page, flips or
  shrinks at the screen edge and follows horizontal scrolling (the anchor is
  the cell as drawn this frame).
- **Cursor row.** The popup's cursor row is a fill (`CursorStyle::Fill`):
  the table keeps the only focus bar. The current value is marked neutrally
  (`›`, or `(•)` / `[✓]`).
- **Disabled options.** Drawn disabled with the reason as their description;
  never chosen by keys, type-ahead or click.
- **Tones.** An option's tone colours the cell (a status pill per value).
- **Affordance.** Editable option cells show a trailing `▾` on hover and on
  the cursor cell, read-only cells never do.
- **Out-of-set values.** Imported data whose value is not an option is shown
  as is, in the error tone with a `!`, and reported by `out_of_set()`; the
  editor opens on it with no current value, and choosing an option fixes it.
- **Validation.** Commits still run the table's validator closure (DataTable)
  or the grid's validator and pending-change queue (DataGrid); blur ends an
  open popup per `EditorConfig::blur` (default `Cancel`, as a macOS popup
  closes without a choice when it loses focus).
- **Bulk apply.** The Selectable table already edits several rows through a
  form dialog; constrained columns there use the same `Select`. Applying a
  popup choice to every selected row is deferred (see below).

## 3. Showcase changes

- The grouped tree sidebar (above).
- New page **Option editors** (Tables group): a deployment-targets table with
  environment (Select), region (Combo), status (Segmented, toned), owner
  (Combo with search), notify (Toggle), labels (Tags), priority (Radio with
  descriptions and a disabled option), a read-only id column, one imported
  row with an out-of-set value, and a legend that explains each editor.
- **Editable tables** gains a Select column; **Data grid** wires its enum
  column to a Select editor.

## Deferred

- Bulk apply of a popup choice to all selected rows from the table itself
  (the Selectable table's bulk form covers it today).
- A free-text "create option" mode for Combo (the request is fixed sets).
