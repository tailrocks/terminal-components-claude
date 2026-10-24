# Code editor completion composition

`CodeEditor::with_completion(popup_id, provider)` composes the existing code editor and completion overlay. `CompletingCodeEditorState` owns editor state, popup state and candidate values. Providers return owned batches with a byte replacement length; candidate text and insertion are redacted in debug output.

The composition requests suggestions only after a text change, after cursor motion while the popup was already open, or after the manual completion binding. Drawing, focus and ticks do not call the provider. Completion bindings retain priority over editor bindings, so Tab accepts a suggestion; the next Tab keeps the editor's configured indent behavior. Escape dismisses the popup first and commits the editor on the next Escape. F6 and Shift+F6 commit and move focus forward and backward. All three actions support the component `KeyMap`.

The shared editor resolves its normal bindings when an overlay publishes a temporary binding table for the editor owner. This keeps cursor movement and focus navigation working while completion is open, and respects remapped and removed component chords.

The editing key path checks effective bindings before inserting a printable character, so remapped printable commands behave the same with completion open or closed while unbound characters still edit normally. Ctrl+F enters Find from editing mode; the completion adapter closes and clears its popup as soon as Find starts, leaving the editor to receive Enter, Tab, and Escape normally.

Focused integration coverage is in `crates/termrock-editors/tests/completing_code_editor.rs`. SQL-specific candidate generation remains consumer domain logic.

The current runtime outside-click path dismisses the top layer and returns without forwarding the pointer to its owner (`crates/termrock-runtime/src/runtime.rs`, `outside_click`). Tests prove popup dismissal only. Owner click-through requires a shared backdrop-region API and remains a follow-up outside this editor-only change.

Temporary Showcase coverage exception: this composition is not yet demonstrated in an application. Demonstrate it through the existing Query Workbench/SQL editor conversion, preserving the frozen screen and navigation, before claiming consumer acceptance. No additional Showcase screen is authorized. The exception ends with that conversion.

The follow-up review found that Ctrl+Space could invoke completion while Find was accepting query text, and that the popup's owner binding publication replaced the editor's dynamic manual-completion binding. The adapter now rejects manual requests during Find and republishes its dynamic binding after the popup publishes its bindings. Bare `CodeEditor` Ctrl+Space coverage also verifies that no `CodeAction` escapes. These regressions fail on the pre-fix adapter: the default and remapped manual-binding tests trigger only once while the popup is open, and the Find test opens completion.

The follow-up review found that Ctrl+Space could invoke completion while Find was accepting query text, and that the popup's owner binding publication replaced the editor's dynamic manual-completion binding. The adapter now rejects manual requests during Find and republishes its dynamic binding after the popup publishes its bindings. Bare `CodeEditor` Ctrl+Space coverage also verifies that no `CodeAction` escapes. These regressions fail on the pre-fix adapter: the default and remapped manual-binding tests trigger only once while the popup is open, and the Find test opens completion.
