# Picker family contract increment (2026-10-20)

This increment moves query editing for `FilterList` and `Picker` onto the shared `TextInput`/`TextInputState` editor core, adds case-insensitive contiguous substring filtering without changing the existing fuzzy `Label` policy, makes the picker row cap explicit (`max_rows`, default 12), and supports reverse scope navigation with BackTab. An external consumer test exercises stable domain item keys through the public API.

The query field now uses the shared text editing path for grapheme-aware insertion/deletion, caret movement, paste, selection state/rendering, and horizontal scroll. Query layout and drawing continue through the existing Picker/FilterList render path. Focus-owner registration is singular for searchable pickers.

## Remaining contract gap

The frozen baseline picker (`src/widgets/picker.rs` at baseline source commit `4169783ae883788186d161af09ba9c481bd84e5c`) integrates `EditMenu` through `edit_menu_key`, `edit_menu_pointer`, `edit_menu_route`, and `point_field`. The shared `TextInput` API in this increment does not provide that edit-menu integration. Pointer drag range selection is also not yet routed as a query-field gesture; the query currently supports pointer cursor placement only. This increment therefore does not establish complete picker interaction parity and must not be reported as a completed picker contract.

Next, add reusable field-owned pointer gesture and edit-menu routing to the shared TextInput/controller contract, preserving one query owner and using the same editor state/cursor as keyboard input. Route Picker query pointer events through that shared API; test click, drag selection, menu actions, copy/cut/paste, Home/End, Escape clear-then-close, and outside-dismiss focus restoration through the runtime. Then run paired baseline/candidate CLI checkpoints with pinned tuiscotti before claiming parity.

## Verification scope

Focused crate tests cover TextInput-backed query editing and filtering, picker row measurement, scope forward/reverse navigation, and an external public consumer. These tests do not replace paired CLI captures, the full scenario inventory, or the four-application baseline parity gate. Exact commands and outcomes are recorded with the commit report.
