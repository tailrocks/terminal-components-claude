# Generic composition fixtures, not a Jackin implementation

These fixtures prove that the library can compose the required surfaces. They contain inert typed data and deterministic UI scripts. They do not implement application routes, service effects, config files, account discovery, launch decisions, authentication or daemon behavior.

| Fixture | Components | What it proves |
|---|---|---|
| R01 List/detail shell | Brand, MenuBar, Panel, Tree/List/NavList, SplitPane, PropsList, Button, HintBar, TooSmall | Identity, current-vs-cursor, detail focus, drawer/narrow behavior and source reorder |
| R02 Tabbed configuration form | Tabs, Form, Field, TextInput/TextArea, Checkbox/Toggle/Select, Grid, ChipBar | Draft lifecycle, errors, controlled values, row/cell presentation and same public components |
| R03 Nested selection dialog | Dialog, Picker, PickerChain, Completion, Menu, TextInput, HelpOverlay | Modal-first paste, one-level Escape, exact focus restoration, retry/readiness and stable target |
| R04 Status/measurement panel | Props, List, Meter, Spinner, StatusBar, Empty | Unknown/stale/error/refreshing states, thresholds and narrow priority dropping |
| R05 Progress and output | Steps, ProgressBar, TextViewport, Panel, Button, HintBar | Deterministic phases, cancel intent only, tail/reading retention, logs versus prose |
| R06 Tabbed cell surfaces | Tabs, ContextMenu, SplitPane, TerminalView, StatusBar | Prepared terminal cell composition, cursor/input ownership, per-tab focus and resize |
| R07 Inspect/review surface | Tree/List, DiffView, CodeEditor read-only mode, PropsList, Dialog | Source-preserving selection/copy, review fallback and caller-owned actions |
| R08 Lifecycle/size harness | Tiny Scene, TooSmall, optional TerminalSession and author paint | Cleanup, resize recovery and custom-art clipping without a product animation engine |

## Reference use

The pinned preview may be run **only as an oracle** to capture a known fixture or isolate an app-local visual component. Candidate fixture code uses only Termrock's public API. It must not clone the preview's World, account models, launch simulator, route enum or Capsule daemon.

A candidate composite may carry synthetic baseline labels such as a workspace name in fixture data to compare a frame, but those labels cannot leak into the library API or its production logic. A snapshot-compatible frame is a conformance scene, not a functional product claim.

No individual fixture introduces a second paint path for a standard widget. Swapping a button glyph in the library must change every fixture using Button and fail the corresponding exact gate. Otherwise the fixture is not exercising the real component.
