# TUI ecosystem comparison

Research retrieved on 2026-09-10 against the current `holla-fable` worktree.
The comparison uses primary project documentation. Existing Junie components,
applications and design rules determine suitability; another project's catalogue
does not establish a need for another widget here.

## Findings and existing coverage

**Architecture/API weakness, P1: command dispatch and help can disagree.**
[Bubbles Key and Help](https://github.com/charmbracelet/bubbles#key) share binding
metadata, while [Textual bindings](https://textual.textualize.io/guide/input/#bindings)
declare action, description and priority together. Here `widgets/keyhint.rs::Hint`
stores display strings separately from executable key matches. The concrete
collision was `DataGrid::on_key` assigning Ctrl+D to row duplication while
`tablepro::App::workbench_chord` intercepted it for Data/Structure. The existing
Ctrl+D application behavior remains; Alt+D now reaches the grid's duplicate action
and appears in editable-grid hints. Regression tests cover dispatch, editing,
modal isolation, read-only rejection, generated-column defaults and undo. A
general binding framework remains an opportunity, not an implementation mandate;
its risk is introducing a second dispatch abstraction before its boundary is proven.

**Architecture/API weakness, P2: modal lifecycle is duplicated.**
[Textual modal screens](https://textual.textualize.io/guide/screens/#modal-screens)
block application bindings as well as background interaction. Junie's
`RenderCtx::begin_modal` already barriers hit testing and focus. Saving/restoring
focus and routing keyboard events remain repeated in the four applications.
Duplication is confirmed; arbitrary event leakage is not. Further consolidation
should follow reproductions involving app shortcuts, outside clicks, nested
overlays or an opener removed while its dialog is open. Risk: lifecycle changes
can alter intentional screen-specific Escape behavior. No replacement dialog
component is needed.

**Coverage gap, P2: viewport selection originally required a mouse.**
[Lazygit's keyboard range selection](https://github.com/jesseduffield/lazygit#stage-individual-lines)
exposes a useful parity check. At audit time `TextViewport::on_key` scrolled,
toggled follow, copied and cleared selection, but could not create one. Click,
drag and word selection supplied the selection; `DiffView::on_key` delegated to
the same viewport. The relevant improvement belongs to the shared selection
primitive, with tests for equivalent copied text across keyboard/mouse, Unicode,
wrapping and resize. Risk: selection movement must not accidentally restore
tail-follow. Git staging behavior itself remains application-specific.

**Coverage/documentation gap, P1/P2: the diff capability was hidden from the
catalogue.** `widgets/diff.rs::DiffView` already supported unified and review
layouts, reused `TextViewport`, and served Jackin's inspect screen. The showcase
had no diff page, while DESIGN.md incorrectly said no diff viewer or context menu
existed. Existing `ContextMenu` and `MenuBar` also contradicted that statement.
The justified addition is documentation and a showcase for an existing component,
verified at small/normal/wide sizes and monochrome, not a second diff widget.

**Existing coverage: finder with an independent preview.**
[fzf's preview window](https://github.com/junegunn/fzf#preview-window) supports
independent scrolling and configurable size, placement and visibility. Picker,
Holla's finder/preview composition, Splitter and TextViewport already cover this
interaction family. Verify focus transfer, wheel ownership and narrow preview
access; no additional command-palette or preview component is justified.

**Existing coverage: list filtering, pagination and activity.**
[Bubbles List](https://github.com/charmbracelet/bubbles#list) combines filtering,
pagination, help, activity and status. Junie already composes List, Picker,
ScrollState, Progress, EmptyState and HintBar around application data. The
comparison does not justify a new file picker, paginator, toast or timer widget.
Domain adapters should remain application-owned until multiple applications
demonstrate an absent reusable interaction.

**Optional opportunity, P3: future asynchronous task ownership.**
[Textual workers](https://textual.textualize.io/guide/workers/#worker-lifetime)
tie cleanup to the creating widget/screen/app; exclusive work prevents stale
responses from replacing newer results. Junie's applications explicitly use
deterministic simulated services. No stale real-network result defect is proven.
A future real-service integration should test cancellation on removal and
out-of-order responses before selecting a shared task abstraction. Adding an
asynchronous framework now is not supported by this coverage audit.

## Implementation evidence from this comparison

The acknowledgement field in `DialogBody::Facts` was rendered and hit-registered
but `Dialog::on_click` handled only `DialogBody::Input`. Reusing `input_mut()`
for clicks now gives both input-bearing variants the same focus/edit path.
The TablePro mouse regression first failed because clicking the acknowledgement
left focus on Cancel; after the fix, it proves first-click focus, second-click
editing, wrong-token rejection and explicit confirming-click execution.

The duplicate-row app regression first failed because Alt+D had no action.
The fix adds Alt+D as an alias of the existing grid operation; standalone Ctrl+D
remains supported and TablePro Ctrl+D retains Data/Structure navigation.
No new visual component or speculative binding framework was added.
