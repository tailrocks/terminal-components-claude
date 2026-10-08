# Component contracts

These 46 pages define the proposed Termrock component and API surfaces. Each page owns that surface's purpose, construction, state/action model, applicable behavior, visual parts, and component-specific conformance cases. Shared mechanisms belong to [foundations](../foundations/README.md); public type and author rules belong to [API](../api/README.md).

These are target contracts, not claims that the current source already implements Termrock. Source names and paths describe the frozen starting implementation. Per-surface capture cases are planning metadata, not approved output; see [verification](../verification/README.md).

## Controls and forms

| Surface | Contract |
|---|---|
| Brand | [brand.md](brand.md) |
| Button | [button.md](button.md) |
| Checkbox | [checkbox.md](checkbox.md) |
| Toggle | [toggle.md](toggle.md) |
| RadioGroup | [radio-group.md](radio-group.md) |
| Select | [select.md](select.md) |
| ChipBar | [chip-bar.md](chip-bar.md) |
| Field | [field.md](field.md) |
| TextInput | [text-input.md](text-input.md) |
| TextArea | [text-area.md](text-area.md) |
| Form | [form.md](form.md) |
| Note | [note.md](note.md) |

## Collections and navigation

| Surface | Contract |
|---|---|
| List | [list.md](list.md) |
| FilterList | [filter-list.md](filter-list.md) |
| NavList | [nav-list.md](nav-list.md) |
| Tree | [tree.md](tree.md) |
| Tabs | [tabs.md](tabs.md) |
| Steps | [steps.md](steps.md) |
| Picker | [picker.md](picker.md) |
| CommandPalette | [command-palette.md](command-palette.md) |
| PickerChain | [picker-chain.md](picker-chain.md) |
| Completion | [completion.md](completion.md) |

## Overlays and menus

| Surface | Contract |
|---|---|
| Dialog | [dialog.md](dialog.md) |
| Menu | [menu.md](menu.md) |
| ContextMenu | [context-menu.md](context-menu.md) |
| MenuBar | [menu-bar.md](menu-bar.md) |
| HelpOverlay | [help-overlay.md](help-overlay.md) |
| Wizard | [wizard.md](wizard.md) |

## Data, text, and layout

| Surface | Contract |
|---|---|
| Grid | [grid.md](grid.md) |
| TextViewport | [text-viewport.md](text-viewport.md) |
| CodeEditor | [code-editor.md](code-editor.md) |
| DiffView | [diff-view.md](diff-view.md) |
| Panel | [panel.md](panel.md) |
| SplitPane | [split-pane.md](split-pane.md) |
| ScrollRegion | [scroll-region.md](scroll-region.md) |
| Props | [props.md](props.md) |
| PropsList | [props-list.md](props-list.md) |

## Status and terminal presentation

| Surface | Contract |
|---|---|
| Empty | [empty.md](empty.md) |
| ProgressBar | [progress-bar.md](progress-bar.md) |
| Spinner | [spinner.md](spinner.md) |
| Meter | [meter.md](meter.md) |
| StatusBar | [status-bar.md](status-bar.md) |
| HintBar | [hint-bar.md](hint-bar.md) |
| KeyHint | [key-hint.md](key-hint.md) |
| TooSmall | [too-small.md](too-small.md) |
| TerminalView | [terminal-view.md](terminal-view.md) |

`Grid` is one engine for table-row and cell/grid modes; `Menu` underlies `ContextMenu` and `MenuBar`; `Picker` underlies `CommandPalette` and chained selection; `TextEditorCore` supports `TextInput`, `TextArea`, and `CodeEditor`; `ScrollRegion` owns shared scrolling, thumb capture, and fades. These shared mechanisms keep distinct surface-specific visuals and behavior.
