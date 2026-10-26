# Source prose archive

Input SHA: c817b99e9e43abbbf3519d56b2f5ee86924243d8

This file preserves original Rust source comments and the removed manual API fixture README. It is an evidence archive for the shared docs owner. It does not claim that every excerpt is a maintained contract.

## `crates/holla-domain/src/args.rs`

- Original source SHA-256: `03f8b5a902fdf960803f6dd52df67b25c1478abdd74b7ac30814e5f8f460df61`
- Excerpts are verbatim and retain their input line numbers.

### Lines 4–4

````
/// Stable identity for one producer-owned input field.
````

### Lines 9–9

````
/// Creates a lowercase, stable field identifier.
````

### Lines 25–25

````
/// Returns the stable identifier text.
````

### Lines 31–31

````
/// Identifies whether an input may be projected into safe review facts.
````

### Lines 44–44

````
/// Metadata declared by an action producer for a typed input.
````

### Lines 57–57

````
/// Declares a text input. The value is preserved exactly after required-value validation.
````

### Lines 70–70

````
/// Declares a finite choice input.
````

### Lines 88–88

````
/// Adds explanatory copy without changing the field's accepted values.
````

### Lines 94–94

````
/// Declares a producer-authored default, validated when the schema is built.
````

### Lines 100–100

````
/// Marks a field as secret so schema construction can reject it before accepting values.
````

### Lines 106–106

````
/// Returns the stable field identifier.
````

### Lines 111–111

````
/// Returns the presentation label.
````

### Lines 116–116

````
/// Returns optional help text.
````

### Lines 121–121

````
/// Returns whether the field must contain a non-empty value.
````

### Lines 126–126

````
/// Returns the producer-authored default, if any.
````

### Lines 131–131

````
/// Returns the accepted choice values, or an empty slice for text fields.
````

### Lines 139–139

````
/// Returns the sensitivity declaration.
````

### Lines 145–145

````
/// Validated, producer-owned input metadata.
````

### Lines 153–153

````
/// Validates schema structure, choices, defaults, and sensitivity before it can be used.
````

### Lines 202–202

````
/// Returns the schema version used by request and plan bindings.
````

### Lines 207–207

````
/// Returns the declared fields in producer order.
````

### Lines 212–212

````
/// Finds a field by stable identifier.
````

### Lines 217–217

````
/// Validates submitted values and returns an accepted request bound to this schema.
````

### Lines 218–218

````
///
````

### Lines 219–219

````
/// Defaults are copied once at acceptance. Optional omitted values stay absent, while an
````

### Lines 220–220

````
/// explicitly supplied empty string remains an accepted value.
````

### Lines 261–261

````
/// Values that passed the producer's schema and required-field checks.
````

### Lines 269–269

````
/// Returns the schema captured at acceptance.
````

### Lines 274–274

````
/// Returns the schema version against which this request was accepted.
````

### Lines 279–279

````
/// Returns an accepted value by stable field identifier.
````

### Lines 287–287

````
/// Returns whether a value was explicitly/default accepted for this field.
````

### Lines 293–293

````
/// Errors found while validating an input schema.
````

### Lines 329–329

````
/// Errors found while accepting values against an input schema.
````

## `crates/holla-sim/src/catalogue.rs`

- Original source SHA-256: `ec96bb507d4fd656f7592d8d5364af0f5aecbaba344c4e9a7b851323a8a12cda`
- Excerpts are verbatim and retain their input line numbers.

### Lines 794–794

````
/// Tag finder row. `here` is false when the scope word uses secondary ink.
````

### Lines 810–810

````
/// Tag preview command ink. `#b3b3b3`, not muted `#8a8a8a`.
````

### Lines 812–812

````
/// Tag preview risk ink when the word is not danger. `#737373`.
````

### Lines 842–842

````
/// Tag `first-use` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 847–847

````
/// Whether the usage history is on.
````

### Lines 848–848

````
///
````

### Lines 849–849

````
/// History is on by default, the way the frozen reference runs it. The
````

### Lines 850–850

````
/// `HOLLA_NO_HISTORY=1` switch turns it off.
````

### Lines 851–851

````
/// Whether the home lists the history rows.
````

### Lines 852–852

````
///
````

### Lines 853–853

````
/// The frozen reference keeps the history rows off unless the environment
````

### Lines 854–854

````
/// asks for them, and the approved frames were captured in that state, so
````

### Lines 855–855

````
/// the default is off. The variable is a capability switch on the home, not
````

### Lines 856–856

````
/// a screen selector: the same home is drawn either way.
````

### Lines 861–861

````
/// Tag `monorepo-root` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 866–866

````
/// Tag `monorepo-child` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 871–871

````
/// Tag `activities-multi` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 876–876

````
/// Tag `docker-cleanup` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 881–881

````
/// Tag `disk-cleanup` at fixture tick 40. Later seeks keep the concept disk page.
````

### Lines 886–886

````
/// Tag `remote-host` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 891–891

````
/// Tag `launch-failure` at fixture tick 40. Later seeks keep the concept activity page.
````

### Lines 896–896

````
/// Launch-failure activity layout.
````

### Lines 897–897

````
///
````

### Lines 898–898

````
/// The policy is a width band, not a fixed snapshot size: the frozen reference
````

### Lines 899–899

````
/// composes the failure card at every viewport it renders, so every width in
````

### Lines 900–900

````
/// the band gets the card.
````

### Lines 905–905

````
/// Tag `hard-cases` at fixture tick 40. Later seeks keep the concept home.
````

### Lines 910–910

````
/// Tag `parity-discovery` at fixture tick 40.
````

### Lines 911–911

````
///
````

### Lines 912–912

````
/// `--frame 40` seeks 80 ms. Every tag source for this fixture is done by tick 40,
````

### Lines 913–913

````
/// so the port shows that catalogue once the seek has reached 80 ms.
````

### Lines 1071–1071

````
/// 80×24 `parity-discovery` plate. Not the 120×40 plate. Later seeks and other sizes stay off it.
````

### Lines 1072–1072

````
/// Tag `parity-discovery` narrow and mid frames. `parity_discovery` keeps the
````

### Lines 1073–1073

````
/// view on later seeks; these bands show the clipped rows only on the tick-40 frame.
````

### Lines 1078–1078

````
/// `parity-discovery` rows for the body width. Below 120 the detail column
````

### Lines 1079–1079

````
/// cannot fit, so the view carries the in-app open as the `Runs` line; the mid
````

### Lines 1080–1080

````
/// band (100 up to 120) also shows the deploy mark on the `Deploy` row. From
````

### Lines 1081–1081

````
/// 120 up this is the wide card data.
````

### Lines 1088–1088

````
// Narrow footer is the in-app open. The wide Runs line starts with `no command`.
````

### Lines 1111–1111

````
// This band shows the mark. The wide card truncates at `Current folder…`.
````

### Lines 1125–1125

````
/// Paused tick-40 `parity-history` frame predicate.
````

### Lines 1126–1126

````
/// Later seeks (the clock runs again) and every other scenario stay off it.
````

### Lines 1131–1131

````
/// `parity-history` rows for the body width. The single-column bands keep the
````

### Lines 1132–1132

````
/// short preview; only the wide card band (120 up to 160) carries the full
````

### Lines 1133–1133

````
/// detail preview, which cannot fit in one column.
````

### Lines 1146–1146

````
/// Paused tick-40 `parity-history` rows: task list and the short preview.
````

### Lines 1249–1249

````
/// Wide detail preview. The single-column bands show only `Runs`/`Scope`/`Risk`.
````

### Lines 1321–1321

````
/// Paused tick-40 `parity-files` view. Later seeks and every other scenario stay off it.
````

### Lines 1336–1336

````
// The query gap is already one row. The leading blank keeps Recent under that gap.
````

### Lines 1377–1377

````
/// Wide-card preview rows for the 120-column band.
````

### Lines 1449–1449

````
/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
````

### Lines 1462–1462

````
/// Paused tick-40 `parity-browser` frame predicate.
````

### Lines 1463–1463

````
/// Later seeks (the clock runs again) and every other scenario stay off it.
````

### Lines 1468–1468

````
/// `parity-browser` rows for the body width. The single-column bands keep the
````

### Lines 1469–1469

````
/// short three-line preview; only the wide card band (120 up to 160) carries the
````

### Lines 1470–1470

````
/// full detail preview, which cannot fit in one column.
````

### Lines 1483–1483

````
/// Wide detail preview. The single-column bands show only `Runs`/`Scope`/`Risk`.
````

### Lines 1555–1555

````
/// Paused tick-40 `parity-git-current` frame. Later seeks and every other scenario stay off it.
````

### Lines 1560–1560

````
/// Frozen narrow `parity-git-current` view body. Single-column frames share it.
````

### Lines 1636–1636

````
/// Wide-card `parity-git-current` view body for the 120-column band.
````

### Lines 1767–1767

````
/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
````

### Lines 1781–1781

````
/// Paused tick-40 `parity-git-batch` frame. Later seeks and every other scenario stay off it.
````

### Lines 1786–1786

````
/// Frozen narrow `parity-git-batch` view body. Single-column frames share it.
````

### Lines 1862–1862

````
/// Wide-card `parity-git-batch` view body for the 120-column band.
````

### Lines 1999–1999

````
/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
````

### Lines 2013–2013

````
/// Paused tick-40 `parity-task-sources` frame. Later seeks and every other scenario stay off it.
````

### Lines 2018–2018

````
/// Frozen narrow `parity-task-sources` view body. Single-column frames share it.
````

### Lines 2121–2121

````
/// Frozen 80-column frame only. The reference never had a 100-column plate for
````

### Lines 2122–2122

````
/// this family, so the wider single-column widths keep the generic adaptive home
````

### Lines 2123–2123

````
/// path instead of the frozen narrow body.
````

### Lines 2134–2134

````
/// Paused tick-40 `parity-cargo` frame. Later seeks and every other scenario stay off it.
````

### Lines 2139–2139

````
/// Frozen narrow `parity-cargo` view body. Single-column frames share it.
````

### Lines 2242–2242

````
/// Wide-card `parity-cargo` view body for the 120-column band.
````

### Lines 2393–2393

````
/// Single-column below 120, wide card from 120. Only the paused tick-40 frame.
````

### Lines 2509–2509

````
/// No-history plate, or the history plate when `history` is set. Only tick 40.
````

### Lines 2518–2518

````
/// Collapsed no-history `first-use` rows. The wide card keeps the preview.
````

### Lines 2606–2606

````
/// Single-column no-history and history views in the 80–119 band.
````

### Lines 2607–2607

````
/// The wide card keeps the preview. Other sizes stay off it.
````

### Lines 2619–2619

````
/// No-history `first-use` plate. History does not change this projection.
````

### Lines 2755–2755

````
/// History-on `first-use` plate. Same tick as the no-history plate. A later seek stays on concept home.
````

### Lines 2893–2893

````
/// Collapsed history rows. The wide card keeps the preview.
````

### Lines 2925–2925

````
/// No-history plate, or the history plate when `history` is set. Only tick 40.
````

### Lines 2934–2934

````
/// Collapsed no-history rows. The wide card keeps the preview.
````

### Lines 2935–2935

````
/// Narrow label keeps one letter of the tool. `node` would paint `n…`; the frozen row is `s…`.
````

### Lines 2975–2975

````
/// Collapsed history rows. The wide card keeps the preview.
````

### Lines 2976–2976

````
/// Narrow label keeps one letter of the tool. `node` would paint `n…`; the frozen row is `s…`.
````

### Lines 3016–3016

````
/// Single-column no-history and history views in the 80–119 band.
````

### Lines 3017–3017

````
/// The wide card keeps the preview. Other sizes stay off it.
````

### Lines 3194–3194

````
/// History-on `monorepo-root` plate. Same tick as the no-history plate. A later seek stays on concept home.
````

### Lines 3371–3371

````
/// No-history plate, or the history plate when `history` is set. Only tick 40.
````

### Lines 3380–3380

````
/// Single-column no-history view in the 80–119 band. History has only the mid
````

### Lines 3381–3381

````
/// view from 100 up; the narrow 80-column band keeps the concept home.
````

### Lines 3604–3604

````
/// History-on `activities-multi` plate. Same tick as the no-history plate. A later seek stays on concept home.
````

### Lines 3824–3824

````
/// Collapsed history rows. The wide card keeps the truncated preview.
````

### Lines 3825–3825

````
/// `node` would paint `n…`; the frozen row is `sqlx-cli`.
````

### Lines 3865–3865

````
/// Shared collapsed rows. `node` would paint `n…` at 80×24; the frozen row is `sqlx-cli`.
````

### Lines 3905–3905

````
/// Collapsed `monorepo-child` rows. The wide card keeps the preview.
````

### Lines 3937–3937

````
/// Single-column view in the 80–119 band. The wide card keeps the preview.
````

### Lines 4136–4136

````
/// Single-column view in the 80–119 band. The wide card keeps the preview.
````

### Lines 4137–4137

````
/// The blocker age is 4 min from 100 up, not the wide card's 41.
````

### Lines 4155–4155

````
/// No-history `remote-host` plate. Only tick 40.
````

### Lines 4339–4339

````
/// No-history `hard-cases` plate. Only tick 40.
````

### Lines 4513–4513

````
/// Collapsed `docker-cleanup` rows. The Runs line truncates per width band.
````

### Lines 4550–4550

````
/// Single-column view in the 80–119 band. The wide card keeps the preview.
````

## `crates/holla-sim/src/disk_scan.rs`

- Original source SHA-256: `80da6277d393419c9cc776539e3570b0194f822d19cdc47baa4ee826578395e9`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Pure, deterministic disk-scan model over an immutable FilesSnapshot.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! This module never inspects the host filesystem, starts work on another
````

### Lines 4–4

````
//! thread, or creates cleanup actions. It eagerly builds a modeled tree and
````

### Lines 5–5

````
//! reveals its preorder at caller-supplied simulation ticks.
````

### Lines 12–12

````
/// Parameters for one modeled scan.
````

### Lines 13–13

````
///
````

### Lines 14–14

````
/// Times are simulation ticks, not wall-clock durations. A zero reveal
````

### Lines 15–15

````
/// duration uses one effective tick, matching the historical minimum-one-tick
````

### Lines 16–16

````
/// behavior. The caller supplies absolute simulated snapshot keys verbatim;
````

### Lines 17–17

````
/// this model never canonicalizes or resolves them through the host OS.
````

### Lines 20–20

````
/// Exact absolute key of an existing snapshot entry.
````

### Lines 22–22

````
/// Whether hidden direct children are included at each directory.
````

### Lines 24–24

````
/// Absolute simulated subtree roots to omit, using path boundaries.
````

### Lines 26–26

````
/// Simulation tick at which the first frame is reported.
````

### Lines 28–28

````
/// Target reveal duration in ticks; zero is normalized to one.
````

### Lines 30–30

````
/// Maximum number of preorder paths revealed per positive tick.
````

### Lines 35–35

````
/// Create a request that includes hidden entries and has no skipped paths.
````

### Lines 53–53

````
/// A modeled condition attached to a visible scan node.
````

### Lines 56–56

````
/// This entry has no explicit scan metadata; its byte totals are unknown.
````

### Lines 58–58

````
/// The modeled directory cannot be listed.
````

### Lines 60–60

````
/// The modeled directory is marked dataless and is not traversed.
````

### Lines 62–62

````
/// The snapshot could not provide a structural listing for this node.
````

### Lines 66–66

````
/// Provenance for a numeric scan aggregate that was capped.
````

### Lines 69–69

````
/// A known allocated-byte accumulation exceeded u64::MAX.
````

### Lines 71–71

````
/// A known apparent-byte accumulation exceeded u64::MAX.
````

### Lines 73–73

````
/// A known entry-count accumulation exceeded u64::MAX.
````

### Lines 77–77

````
/// One node in the eagerly computed modeled scan tree.
````

### Lines 78–78

````
///
````

### Lines 79–79

````
/// None byte totals mean at least one required metadata value was absent. A
````

### Lines 80–80

````
/// true saturation bit means the corresponding known subtotal was capped; if
````

### Lines 81–81

````
/// the total is also None, some known contribution saturated while another
````

### Lines 82–82

````
/// contribution remained unknown.
````

### Lines 85–85

````
/// Exact simulated snapshot key.
````

### Lines 87–87

````
/// Modeled node kind; symlink targets are recorded but never followed.
````

### Lines 89–89

````
/// Allocated-byte total, or None if required metadata is unavailable.
````

### Lines 91–91

````
/// Apparent-byte total, or None if required metadata is unavailable.
````

### Lines 93–93

````
/// Explicit modeled modification time, if supplied.
````

### Lines 95–95

````
/// Count of non-directory path entries below this node, including self
````

### Lines 96–96

````
/// for a non-directory node.
````

### Lines 98–98

````
/// Partial-node conditions collected without hiding one another.
````

### Lines 100–100

````
/// Flags distinguishing capped aggregates from exact numeric maxima.
````

### Lines 102–102

````
/// Visible structural children in allocated-descending/path order.
````

### Lines 107–107

````
/// Find a node by its exact simulated snapshot key.
````

### Lines 116–116

````
/// Current state of a scan at a requested simulation tick.
````

### Lines 119–119

````
/// The reveal prefix has not reached the whole tree.
````

### Lines 121–121

````
/// Every tree path has been revealed.
````

### Lines 123–123

````
/// A partial reveal prefix was frozen by cancellation.
````

### Lines 127–127

````
/// Provenance of the immutable input used for the scan.
````

### Lines 130–130

````
/// An in-memory deterministic FilesSnapshot; no host filesystem was read.
````

### Lines 134–134

````
/// Typed request or lifecycle failure.
````

### Lines 135–135

````
///
````

### Lines 136–136

````
/// Missing scan metadata is a partial-node issue, not a request failure.
````

### Lines 137–137

````
/// Generation exhaustion is atomic: the active session is left in place.
````

### Lines 140–140

````
/// The request root is not an absolute simulated key.
````

### Lines 142–142

````
/// The exact root key is absent from the input snapshot.
````

### Lines 144–144

````
/// A skip path is not an absolute simulated key.
````

### Lines 146–146

````
/// A zero batch limit would prevent all progress.
````

### Lines 148–148

````
/// No unused generation identity remains.
````

### Lines 150–150

````
/// A report or cancellation used a non-current generation.
````

### Lines 152–152

````
/// Generation requested by the caller.
````

### Lines 154–154

````
/// Active generation, if one remains.
````

### Lines 192–192

````
/// A controller that owns monotonic scan generations and one active session.
````

### Lines 206–206

````
/// Create an empty controller; the first successful scan has generation 1.
````

### Lines 214–214

````
/// Start a scan from an immutable snapshot and return its generation.
````

### Lines 215–215

````
///
````

### Lines 216–216

````
/// Validation and tree computation complete before the current session or
````

### Lines 217–217

````
/// next generation changes. A failed request therefore cannot replace a
````

### Lines 218–218

````
/// usable scan or consume a generation.
````

### Lines 269–269

````
/// Return a deterministic report for the matching active generation.
````

### Lines 294–294

````
/// Freeze the currently revealed prefix for the matching generation.
````

### Lines 295–295

````
///
````

### Lines 296–296

````
/// Repeated cancellation is harmless. Cancelling after completion leaves
````

### Lines 297–297

````
/// the status Complete, while a stale generation cannot affect a rescan.
````

### Lines 321–321

````
/// Return the active generation, if the controller has one.
````

### Lines 338–338

````
/// A read-only view of one scan at a simulation tick.
````

### Lines 350–350

````
/// The generation represented by this report.
````

### Lines 355–355

````
/// The full computed tree. Renderers should use visible_paths for reveal.
````

### Lines 360–360

````
/// The deterministic preorder used by progressive reveal.
````

### Lines 365–365

````
/// Number of entries revealed by this report.
````

### Lines 370–370

````
/// The current scan lifecycle state.
````

### Lines 375–375

````
/// The modeled source provenance.
````

### Lines 380–380

````
/// The preorder prefix currently revealed by the modeled scan.
````

## `crates/holla-sim/src/files.rs`

- Original source SHA-256: `5de5725f781ff9e2c16e8b4a76ab79fd15d1a5375ff19b7c5b49772a43fcf0db`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! A deterministic, read-only filesystem snapshot for the Holla simulator.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! Paths in this module are simulated keys. No method reads or changes the host filesystem.
````

### Lines 13–13

````
/// Explicit scan metadata for one simulated filesystem entry.
````

### Lines 14–14

````
///
````

### Lines 15–15

````
/// The identity is local to the modeled snapshot. It is not an operating
````

### Lines 16–16

````
/// system inode and is never inferred from a path or preview content.
````

### Lines 19–19

````
/// Identity used only to model hardlinks within one snapshot.
````

### Lines 21–21

````
/// Explicit allocated storage bytes for this entry.
````

### Lines 23–23

````
/// Explicit logical or apparent bytes for this entry.
````

### Lines 25–25

````
/// Explicit modeled modification time in Unix seconds.
````

### Lines 27–27

````
/// Whether a directory is modeled as unavailable without materialization.
````

### Lines 31–31

````
/// The subset of simulated node kinds needed by the first Files browser stage.
````

### Lines 40–40

````
/// One immutable-in-use entry in a [`FilesSnapshot`].
````

### Lines 65–65

````
/// Create a readable regular file at a simulated path.
````

### Lines 76–76

````
/// Create a readable regular file with explicitly modeled bytes.
````

### Lines 81–81

````
/// Attach explicitly modeled bytes to this simulated file.
````

### Lines 88–88

````
/// Create a readable directory at a simulated path.
````

### Lines 99–99

````
/// Create a symbolic link whose target is another snapshot key.
````

### Lines 112–112

````
/// Create a special node that the first browser stage does not open.
````

### Lines 123–123

````
/// Mark this simulated entry as unreadable.
````

### Lines 130–130

````
/// Attach explicit in-memory scan facts to this simulated entry.
````

### Lines 131–131

````
///
````

### Lines 132–132

````
/// Constructors leave scan metadata absent; preview bytes and paths are
````

### Lines 133–133

````
/// never used to synthesize sizes, identities, or modification times.
````

### Lines 165–165

````
/// A typed failure while listing or opening a simulated directory.
````

### Lines 187–187

````
/// The reason a modeled file's inspected bytes must not be displayed as text.
````

### Lines 203–203

````
/// Safe content outcomes for one explicitly modeled file.
````

### Lines 221–221

````
/// A preview result bound to both the selected path and its resolved target.
````

### Lines 248–248

````
/// A typed failure while resolving a simulated file preview.
````

### Lines 272–272

````
/// A deterministic snapshot of simulated paths and entries.
````

### Lines 283–283

````
/// Insert or replace one simulated path. Parent creation is explicit.
````

### Lines 292–292

````
/// List direct children in directory-first, case-insensitive name order.
````

### Lines 330–330

````
/// Resolve a selected directory or directory symlink to its snapshot key.
````

### Lines 359–359

````
/// Resolve and safely inspect one file using only entries in this snapshot.
````

## `crates/holla-sim/src/fixtures.rs`

- Original source SHA-256: `351dfb7099c6a15a0ce7906d51be281e2c3a7fc6300a6404d68296ce9882b175`
- Excerpts are verbatim and retain their input line numbers.

### Lines 22–22

````
/// Build the deterministic simulated file snapshot used by the Files route.
````

### Lines 23–23

````
///
````

### Lines 24–24

````
/// This fixture is data only; it never inspects the host filesystem. The empty
````

### Lines 25–25

````
/// scratch scenario stays empty, while other starting directories contain a
````

### Lines 26–26

````
/// small tree with hidden, Unicode, unreadable, and broken-link entries.
````

### Lines 217–217

````
/// Builds a dedicated nextest-capable task fixture for typed filter/profile input.
````

### Lines 218–218

````
///
````

### Lines 219–219

````
/// The existing `test` task remains `cargo test --workspace`; this fixture adds a
````

### Lines 220–220

````
/// separate producer-backed `test-nextest` task and does not change that task's
````

### Lines 221–221

````
/// historical behavior.
````

### Lines 239–239

````
/// Dedicated synthetic case for a typed frontend port-conflict failure.
````

### Lines 240–240

````
///
````

### Lines 241–241

````
/// This case is not a CLI scenario and does not change any existing World fixture.
````

### Lines 251–251

````
/// Builds one failed frontend-dev Activity with a separate simulated listener inventory.
````

### Lines 252–252

````
///
````

### Lines 253–253

````
/// # Errors
````

### Lines 254–254

````
/// Returns an error if the authored listener rows do not form a valid inventory.
````

## `crates/holla-sim/src/listeners.rs`

- Original source SHA-256: `5db22acf54fb942b91a2a18452f9b5bef8df427ab55f52d0f8d4eba7218bd0a0`
- Excerpts are verbatim and retain their input line numbers.

### Lines 12–12

````
/// Construction and validation errors for deterministic listener inventories.
````

### Lines 15–15

````
/// A synthetic process identifier was zero.
````

### Lines 17–17

````
/// A synthetic process generation token was zero.
````

### Lines 19–19

````
/// A TCP-listening port was zero.
````

### Lines 21–21

````
/// A synthetic process name was blank.
````

### Lines 23–23

````
/// A synthetic process state was blank.
````

### Lines 25–25

````
/// A synthetic CPU percentage exceeded 100.
````

### Lines 27–27

````
/// The same PID/generation pair appeared more than once.
````

### Lines 29–29

````
/// A PID appeared more than once, even with another generation token.
````

### Lines 50–50

````
/// Complete deterministic process facts held by a simulator-owned listener inventory.
````

### Lines 51–51

````
///
````

### Lines 52–52

````
/// Generation is a synthetic identity token. It is not an operating-system start time.
````

### Lines 65–65

````
/// Builds one synthetic TCP-listener row.
````

### Lines 66–66

````
///
````

### Lines 67–67

````
/// # Errors
````

### Lines 68–68

````
/// Rejects zero identity/port values, blank display facts, or CPU percentages above 100.
````

### Lines 161–161

````
/// Immutable synthetic listener inventory bound to one World context.
````

### Lines 162–162

````
///
````

### Lines 163–163

````
/// A newly constructed model gets a fresh allocation-backed identity, even when all rows match.
````

### Lines 172–172

````
/// Validates and captures one deterministic listener inventory for this World.
````

### Lines 173–173

````
///
````

### Lines 174–174

````
/// # Errors
````

### Lines 175–175

````
/// Rejects ambiguous process identities. Each PID and each PID/generation pair is unique.
````

### Lines 203–203

````
/// Distinguishes no supplied inventory from a valid inventory with no matching rows.
````

### Lines 206–206

````
/// No synthetic inventory was supplied.
````

### Lines 208–208

````
/// A valid supplied inventory contains no row for the accepted port.
````

### Lines 210–210

````
/// A valid supplied inventory contains one or more rows for the accepted port.
````

### Lines 214–214

````
/// Provenance for every result from this model-only API.
````

### Lines 217–217

````
/// Facts came only from this deterministic simulator model.
````

### Lines 221–221

````
/// Identity-bound proposal to inspect one synthetic listener row.
````

### Lines 222–222

````
///
````

### Lines 223–223

````
/// Its fields stay private; callers must revalidate it against the same accepted request and model.
````

### Lines 253–253

````
/// Safe synthetic facts returned only after suggestion revalidation.
````

### Lines 268–268

````
/// Returns the synthetic process identifier recorded in this fixture.
````

### Lines 273–273

````
/// Returns the synthetic generation token, not an operating-system start time.
````

### Lines 278–278

````
/// Returns the modeled TCP-listening port.
````

### Lines 283–283

````
/// Returns the synthetic process display name.
````

### Lines 288–288

````
/// Returns the synthetic process display state.
````

### Lines 293–293

````
/// Returns the synthetic CPU percentage.
````

### Lines 298–298

````
/// Returns the synthetic resident-memory estimate in MiB.
````

### Lines 303–303

````
/// Returns the source classification for these facts.
````

### Lines 308–308

````
/// Returns the fixed no-execution status of this projection.
````

### Lines 330–330

````
/// Pure simulated invocation plus optional synthetic listener suggestions.
````

### Lines 339–339

````
/// Returns the existing structured no-execution simulation report.
````

### Lines 344–344

````
/// Returns whether modeled listener data was supplied and matched.
````

### Lines 349–349

````
/// Returns opaque suggestions that require explicit revalidation before use.
````

### Lines 354–354

````
/// Returns the fixed deterministic-simulation provenance.
````

### Lines 360–360

````
/// Error returned when an accepted request or identity-bound suggestion is stale.
````

### Lines 363–363

````
/// The existing accepted PortArgs plan failed its own validation.
````

### Lines 365–365

````
/// Suggestion revalidation was requested without a supplied model.
````

### Lines 367–367

````
/// The model or captured host context differs from the current World.
````

### Lines 369–369

````
/// The request, model epoch, instance key, port, or captured row changed.
````

### Lines 386–386

````
/// Structured synthetic failure tied to one unchanged failed Activity and World context.
````

### Lines 387–387

````
///
````

### Lines 388–388

````
/// The retained output is evidence text only; the typed port is never parsed from that text.
````

### Lines 406–406

````
/// Error returned when the structured synthetic failure no longer matches the World.
````

### Lines 409–409

````
/// Host identity, working directory, or effect revision changed.
````

### Lines 411–411

````
/// The captured Activity ID is no longer present.
````

### Lines 413–413

````
/// The captured Activity ID no longer identifies exactly one Activity.
````

### Lines 415–415

````
/// Captured Activity content differs from the current failed Activity.
````

### Lines 417–417

````
/// The captured Activity is no longer in the failed state.
````

### Lines 458–458

````
/// Revalidates one authored conflict against its captured failed Activity and host context.
````

### Lines 459–459

````
///
````

### Lines 460–460

````
/// # Errors
````

### Lines 461–461

````
/// Rejects context drift, missing/duplicate activity identity, changed content, or a nonfailed
````

### Lines 462–462

````
/// Activity. The port comes from the typed simulation event, never from display text.
````

## `crates/holla-sim/src/plans.rs`

- Original source SHA-256: `9df67fdda87d6288358c1b83c419f3d64e307a35735d2ff5647d29dab58ea84c`
- Excerpts are verbatim and retain their input line numbers.

### Lines 233–233

````
/// Build the existing disk-reclaim review for an exact nonempty subset of the current inventory.
````

### Lines 234–234

````
///
````

### Lines 235–235

````
/// The full inventory is validated; duplicate or absent candidate paths and ActiveToday rows are refused.
````

### Lines 236–236

````
/// This path-only API cannot detect metadata changes at an unchanged path before construction, so
````

### Lines 237–237

````
/// callers must compare their complete opening inventory and context. The resulting review then
````

### Lines 238–238

````
/// binds its full target for approval.
````

## `crates/holla-sim/src/port_args.rs`

- Original source SHA-256: `d6371bf746d58f9bc02e15161c582470355b4b4e6adbb8608c6d22f0417ccca7`
- Excerpts are verbatim and retain their input line numbers.

### Lines 25–25

````
/// Fixed producer declaration for the authored, read-only `system.port` action.
````

### Lines 34–34

````
/// Builds the producer's fixed public port schema.
````

### Lines 35–35

````
///
````

### Lines 36–36

````
/// # Errors
````

### Lines 37–37

````
/// Returns a schema error if the built-in field declaration is invalid.
````

### Lines 50–50

````
/// Returns the validated schema owned by this producer.
````

### Lines 62–62

````
/// Returns the fixed producer for the port action.
````

### Lines 63–63

````
///
````

### Lines 64–64

````
/// # Errors
````

### Lines 65–65

````
/// Returns a schema error if the built-in declaration is invalid.
````

### Lines 70–70

````
/// Builds the one ordinary authored canonical action before recent and pin rows are derived.
````

### Lines 88–88

````
/// Identifies the reserved canonical producer before generic command dispatch.
````

### Lines 89–89

````
///
````

### Lines 90–90

````
/// A stale canonical ID remains reserved and therefore fails closed in `begin`; command text on
````

### Lines 91–91

````
/// an unrelated or unavailable-memory row never creates a producer.
````

### Lines 127–127

````
/// Unaccepted input state bound to one current canonical action and host context.
````

### Lines 137–137

````
/// An error found while binding or revalidating port input.
````

### Lines 167–167

````
/// Error returned when a bound port input accepts or validates typed values.
````

### Lines 186–186

````
/// Binds the canonical producer row before the native editor opens.
````

### Lines 187–187

````
///
````

### Lines 188–188

````
/// # Errors
````

### Lines 189–189

````
/// Returns an error if the row is stale, ambiguous, blocked, or has changed policy.
````

### Lines 206–206

````
/// Validates required/default values once and captures one immutable typed review plan.
````

### Lines 207–207

````
///
````

### Lines 208–208

````
/// The schema supplies the default only when `port` is omitted. Explicit empty input is
````

### Lines 209–209

````
/// rejected; malformed non-empty input retains the original untrimmed `u16` parse fallback.
````

### Lines 210–210

````
///
````

### Lines 211–211

````
/// # Errors
````

### Lines 212–212

````
/// Returns a binding error if the captured action/context changed, or an input error if values
````

### Lines 213–213

````
/// violate the schema.
````

### Lines 251–251

````
/// Non-cloneable accepted request, captured parse disposition, and structured numeric argv.
````

### Lines 265–265

````
/// Returns the one structured argv vector captured when input was accepted.
````

### Lines 270–270

````
/// Returns the accepted bytes without reparsing or trimming them.
````

### Lines 275–275

````
/// Returns the parse disposition recorded at acceptance.
````

### Lines 280–280

````
/// Returns the effective numeric port captured at acceptance.
````

### Lines 286–286

````
/// Non-secret facts revalidated from the exact captured plan for the review screen.
````

### Lines 298–298

````
/// Revalidates the same captured plan and projects facts for explicit user review.
````

### Lines 299–299

````
///
````

### Lines 300–300

````
/// # Errors
````

### Lines 301–301

````
/// Returns an error if the current action, producer, schema, or world context differs.
````

### Lines 319–319

````
/// Revalidates a plan and returns only a pure, structured simulation report.
````

### Lines 320–320

````
///
````

### Lines 321–321

````
/// This does not run `lsof`, inspect processes, mutate activities, or change `World`.
````

### Lines 322–322

````
///
````

### Lines 323–323

````
/// # Errors
````

### Lines 324–324

````
/// Returns an error if the current action, producer, schema, or world context differs.
````

### Lines 479–479

````
/// Revalidates the accepted port request and projects optional deterministic listener data.
````

### Lines 480–480

````
///
````

### Lines 481–481

````
/// The request's effective numeric port is authoritative; accepted text, action labels, command
````

### Lines 482–482

````
/// strings, and argv are never reparsed to select model rows. Nothing is executed.
````

### Lines 483–483

````
///
````

### Lines 484–484

````
/// # Errors
````

### Lines 485–485

````
/// Returns a port binding error or a stale model-context error.
````

### Lines 497–497

````
/// Revalidates one suggestion against the same accepted request and exact immutable model.
````

### Lines 498–498

````
///
````

### Lines 499–499

````
/// # Errors
````

### Lines 500–500

````
/// Returns a port binding error, unavailable-model error, or stale model/suggestion error.
````

## `crates/holla-sim/src/scenario.rs`

- Original source SHA-256: `c1df84f4f0175c8526c8d74e811911527b4a6bedc917a85dd802e234cd8075b6`
- Excerpts are verbatim and retain their input line numbers.

### Lines 39–39

````
/// Opening page from the visual-baseline `App::start` match.
````

### Lines 40–40

````
///
````

### Lines 41–41

````
/// `UpgradePlan` names the plan review, but the mounted key flows type the
````

### Lines 42–42

````
/// plan query on the finder before that page opens.
````

### Lines 230–230

````
/// First finder heading on the baseline frame.
````

### Lines 231–231

````
///
````

### Lines 232–232

````
/// Worlds with no recommended row lead with Recent here. The rest lead
````

### Lines 233–233

````
/// with Suggested here. Disk and the failed activity paint their own page.
````

## `crates/holla-sim/src/task_args.rs`

- Original source SHA-256: `8bd0f0db6be2a78570ac6c47bac70831361e925e1d5e4789288bfa465a7a4d97`
- Excerpts are verbatim and retain their input line numbers.

### Lines 20–20

````
/// Explicit producer declaration for the dedicated nextest-capable fixture task.
````

### Lines 21–21

````
///
````

### Lines 22–22

````
/// This producer is opt-in by exact task ID, definition path, and full task body. It does not
````

### Lines 23–23

````
/// infer argument support from the existing `test` task or its display name.
````

### Lines 35–35

````
/// Returns the explicit filter/profile producer for `test-nextest`.
````

### Lines 48–48

````
/// Returns the expected task identity.
````

### Lines 53–53

````
/// Returns the exact task definition path bound by this producer.
````

### Lines 58–58

````
/// Returns the validated input schema.
````

### Lines 63–63

````
/// Validates submitted values and captures defaults into one accepted request.
````

### Lines 69–69

````
/// Returns the producer declaration for the dedicated nextest task fixture.
````

### Lines 70–70

````
///
````

### Lines 71–71

````
/// # Errors
````

### Lines 72–72

````
/// Returns a schema validation error if the producer's built-in declaration is malformed.
````

### Lines 119–119

````
/// Finds the explicitly declared producer for a canonical action or its remembered row.
````

### Lines 120–120

````
///
````

### Lines 121–121

````
/// The exact canonical intent is the declaration. Other tasks keep their existing dispatch
````

### Lines 122–122

````
/// behavior, while a stale canonical or remembered `test-nextest` row still selects this
````

### Lines 123–123

````
/// producer and fails closed.
````

### Lines 172–172

````
/// Unaccepted, non-cloneable input session bound before Trust or Args is shown.
````

### Lines 173–173

````
///
````

### Lines 174–174

````
/// It snapshots the full task definition, producer/schema, canonical action policy, and host
````

### Lines 175–175

````
/// context. An untrusted task can transition only through this session's exact-file approval.
````

### Lines 185–185

````
/// Error returned while a bound draft accepts its typed values.
````

### Lines 204–204

````
/// Binds the current task row before opening Trust or Args.
````

### Lines 222–222

````
/// Returns the exact task definition path that this draft can approve.
````

### Lines 227–227

````
/// Validates and performs the one exact-file Trust transition for this draft.
````

### Lines 228–228

````
///
````

### Lines 229–229

````
/// A session that began trusted cannot approve another task or reuse Trust as a general
````

### Lines 230–230

````
/// dispatch route.
````

### Lines 260–260

````
/// Validates the same bound draft, accepts values, and constructs its plan once.
````

### Lines 453–453

````
/// Structured invocation data. `argv` is an argument vector, never shell text.
````

### Lines 464–464

````
/// Constructs a crate-internal structured read-only invocation.
````

### Lines 465–465

````
///
````

### Lines 466–466

````
/// Producer modules must bind canonical policy before supplying fixed program/operand data.
````

### Lines 467–467

````
/// The risk is intentionally fixed here rather than accepted from producer input.
````

### Lines 478–478

````
/// Returns the executable name.
````

### Lines 483–483

````
/// Returns the exact ordered argument vector.
````

### Lines 488–488

````
/// Returns the invocation's working directory.
````

### Lines 493–493

````
/// Returns the action target bound by the current catalogue row.
````

### Lines 498–498

````
/// Returns the risk bound by the current canonical action.
````

### Lines 504–504

````
/// Immutable, freshly bound invocation plan for one accepted request.
````

### Lines 516–516

````
/// Returns the structured invocation produced by the bound request.
````

### Lines 521–521

````
/// Returns the request values accepted against the bound schema.
````

### Lines 527–527

````
/// Safe, non-secret facts derived from a freshly revalidated invocation plan.
````

### Lines 537–537

````
/// Explicit execution disposition of the deterministic simulator.
````

### Lines 543–543

````
/// A modeled result from the simulator; it is not a process launch or execution claim.
````

### Lines 551–551

````
/// Constructs the immutable typed invocation plan for one current task row.
````

### Lines 552–552

````
///
````

### Lines 553–553

````
/// The action's canonical `command` remains unchanged. The producer owns the argument mapping,
````

### Lines 554–554

````
/// including omission of an empty optional filter.
````

### Lines 555–555

````
///
````

### Lines 556–556

````
/// # Errors
````

### Lines 557–557

````
/// Returns an error when the row is stale, untrusted, unsupported, or bound to a changed task,
````

### Lines 558–558

````
/// context, schema, or accepted request.
````

### Lines 580–580

````
/// Revalidates a plan and projects safe facts from its same structured argv vector.
````

### Lines 581–581

````
///
````

### Lines 582–582

````
/// # Errors
````

### Lines 583–583

````
/// Returns an error when the action, task definition, trust, context, producer, schema, or
````

### Lines 584–584

````
/// accepted request no longer matches the bound plan.
````

### Lines 600–600

````
/// Revalidates a plan and returns a pure simulation report from its exact argv and target.
````

### Lines 601–601

````
///
````

### Lines 602–602

````
/// This function does not execute a process or mutate `World`.
````

### Lines 603–603

````
///
````

### Lines 604–604

````
/// # Errors
````

### Lines 605–605

````
/// Returns an error when the action, task definition, trust, context, producer, schema, or
````

### Lines 606–606

````
/// accepted request no longer matches the bound plan.
````

### Lines 708–708

````
/// Errors from constructing or consuming a task-backed typed invocation.
````

## `crates/holla-sim/src/task_run.rs`

- Original source SHA-256: `592fc3a1d29e4c57140e19abcfc7f7b23970d2a1b279a348f174b5f1985c65c8`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Deterministic task-run model for authored simulated tasks.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! This module models scripted output and prompts. It never launches a process.
````

### Lines 14–14

````
/// Identity for one modeled task run.
````

### Lines 15–15

````
///
````

### Lines 16–16

````
/// IDs are allocated by [`TaskRunStore`] and never wrap or get reused within a store.
````

### Lines 21–21

````
/// Returns this run's store-local numeric identity.
````

### Lines 27–27

````
/// Identity for one prompt, unique across runs in one [`TaskRunStore`].
````

### Lines 32–32

````
/// Returns this prompt's store-local numeric identity.
````

### Lines 38–38

````
/// Authored script identities supported by the bounded simulator model.
````

### Lines 42–42

````
/// Small deterministic dev-server example used by the modeled UI route.
````

### Lines 46–46

````
/// State of one simulated run.
````

### Lines 50–50

````
/// Script can advance on simulation time.
````

### Lines 52–52

````
/// Script waits for its current typed prompt answer.
````

### Lines 54–54

````
/// An authored terminal branch completed.
````

### Lines 56–56

````
/// The modeled run was canceled by its owner.
````

### Lines 58–58

````
/// A required future event cannot fit within the modeled clock range.
````

### Lines 59–59

````
///
````

### Lines 60–60

````
/// A run in this state can still be canceled.
````

### Lines 64–64

````
/// Recoverable TaskRun store or lifecycle failure.
````

### Lines 68–68

````
/// No unique run identity remains in this store.
````

### Lines 70–70

````
/// No unique prompt identity remains for a new run.
````

### Lines 72–72

````
/// The requested run does not exist in this store.
````

### Lines 74–74

````
/// The run has no open prompt to answer.
````

### Lines 76–76

````
/// The answer names a prompt other than the run's current prompt.
````

### Lines 78–78

````
/// The authored prompt has no exact or default branch.
````

### Lines 80–80

````
/// Wait accounting exceeded the finite supported duration range.
````

### Lines 100–100

````
/// One immutable scripted output line.
````

### Lines 108–108

````
/// Returns the active script time when this line was emitted.
````

### Lines 113–113

````
/// Returns the authored output text.
````

### Lines 119–119

````
/// Current typed prompt exposed by a modeled run.
````

### Lines 130–130

````
/// Returns this prompt's identity.
````

### Lines 135–135

````
/// Returns the authored prompt text.
````

### Lines 140–140

````
/// Returns the effective world-clock boundary where the prompt opened.
````

### Lines 145–145

````
/// Returns whether a future UI must treat the answer as secret.
````

### Lines 151–151

````
/// Observable event from one TaskRun update.
````

### Lines 155–155

````
/// Authored output became visible.
````

### Lines 157–157

````
/// A prompt became active and requires its matching identity to answer.
````

### Lines 159–159

````
/// An authored terminal branch completed.
````

### Lines 161–161

````
/// Frozen active script duration.
````

### Lines 164–164

````
/// The modeled run was canceled.
````

### Lines 166–166

````
/// Frozen active script duration.
````

### Lines 169–169

````
/// A required next event is beyond the supported clock ceiling.
````

### Lines 173–173

````
/// Events produced by one run operation.
````

### Lines 181–181

````
/// Returns the run that produced these events.
````

### Lines 186–186

````
/// Returns events in authored execution order.
````

### Lines 192–192

````
/// Owns deterministic simulated runs and checked identity allocation.
````

### Lines 211–211

````
/// Starts one authored simulated script at the supplied world time.
````

### Lines 212–212

````
///
````

### Lines 213–213

````
/// Negative time normalizes to zero. The call reserves run and prompt identities before it
````

### Lines 214–214

````
/// publishes the run, so identity exhaustion cannot leave a partial run.
````

### Lines 215–215

````
///
````

### Lines 216–216

````
/// # Errors
````

### Lines 217–217

````
/// Returns an identity-exhaustion error without inserting a run or consuming another ID.
````

### Lines 246–246

````
/// Advances every stored run to a nondecreasing effective world time.
````

### Lines 247–247

````
///
````

### Lines 248–248

````
/// A coarse jump stops at the first prompt's logical boundary. The remaining wall interval
````

### Lines 249–249

````
/// becomes prompt wait, not active script time.
````

### Lines 264–264

````
/// Answers the run's current prompt with typed text.
````

### Lines 265–265

````
///
````

### Lines 266–266

````
/// # Errors
````

### Lines 267–267

````
/// Returns [`TaskRunError::MissingRun`], [`TaskRunError::NotWaiting`],
````

### Lines 268–268

````
/// [`TaskRunError::StalePrompt`], [`TaskRunError::BranchMissing`], or a wait-accounting
````

### Lines 269–269

````
/// error without accepting a stale answer.
````

### Lines 282–282

````
/// Cancels one modeled run, settling an open prompt wait exactly once.
````

### Lines 283–283

````
///
````

### Lines 284–284

````
/// Cancellation is immediate in this simulator stage; it does not model signals or processes.
````

### Lines 285–285

````
///
````

### Lines 286–286

````
/// # Errors
````

### Lines 287–287

````
/// Returns [`TaskRunError::MissingRun`] or a wait-accounting error without partial settlement.
````

### Lines 298–298

````
/// Returns a stored run without changing its effective time.
````

### Lines 303–303

````
/// Returns the number of stored runs.
````

### Lines 308–308

````
/// Returns whether this store has no runs.
````

### Lines 314–314

````
/// A preflighted prompt-ID allocation that can be committed with its run.
````

### Lines 321–321

````
/// Reserves every prompt identity for a script before its run is published.
````

### Lines 322–322

````
///
````

### Lines 323–323

````
/// Reserving the complete range up front makes later prompt opening infallible and ensures an
````

### Lines 324–324

````
/// exhausted allocator cannot leave a partially started run or reuse an earlier prompt ID.
````

### Lines 346–346

````
/// One deterministic simulated task run.
````

### Lines 392–392

````
/// Returns this run's identity.
````

### Lines 397–397

````
/// Returns the authored script identity.
````

### Lines 402–402

````
/// Returns the run's current modeled state.
````

### Lines 407–407

````
/// Returns the nonnegative world time recorded when the run started.
````

### Lines 412–412

````
/// Returns frozen-or-current active script duration, excluding every prompt wait.
````

### Lines 417–417

````
/// Returns the total closed prompt wait, settled once per accepted answer or cancellation.
````

### Lines 422–422

````
/// Returns the active prompt, if this run is waiting for an answer.
````

### Lines 427–427

````
/// Returns output in authored execution order.
````

## `crates/holla-sim/src/world.rs`

- Original source SHA-256: `effce6055e8ffbc6d66e324b9bb679cd9b6e3a680633dceedeb26832a57bbaa0`
- Excerpts are verbatim and retain their input line numbers.

### Lines 146–146

````
/// Returns whether every scheduled discovery has reached a terminal state.
````

### Lines 147–147

````
///
````

### Lines 148–148

````
/// An empty discovery list is settled. This reports discovery state only;
````

### Lines 149–149

````
/// it does not advance the clock or run pending discovery work.
````

## `crates/holla-ui/src/app.rs`

- Original source SHA-256: `de7a98fd8c0c1bedfb11a3699216bd88559f9632bfbf678a0521194bb65b550b`
- Excerpts are verbatim and retain their input line numbers.

### Lines 258–258

````
/// Authored source plus bounded exact projections for supported terminal color levels.
````

### Lines 259–259

````
/// Configuration pays for the four projections once; draws only select and compare one.
````

### Lines 269–269

````
// The bounded projections avoid rebuilding `Theme` and its recipe vectors during draw.
````

### Lines 270–270

````
// Validation still compares the full public theme value so every authored edit invalidates
````

### Lines 271–271

````
// the retained provenance.
````

### Lines 317–317

````
// MenuBar styles the label and the tag's " {title} " pads as one title.
````

### Lines 327–327

````
/// Paused tag frames whose menu bar is not the focused control. The shared
````

### Lines 328–328

````
/// bar would paint those titles bold and muted.
````

### Lines 342–342

````
/// Rest menu titles. A focused bar bolds them in muted ink. The tag bar is
````

### Lines 343–343

````
/// secondary `#b3b3b3` and not bold.
````

### Lines 368–368

````
/// Tag `segments::render` inset. Not the status-bar gutter: that bar fills `#111111`.
````

### Lines 372–372

````
/// Tag status row. Gaps are the container ink, so item text does not include them.
````

### Lines 412–412

````
/// The activities footer keeps the clipped `Ctrl+G` marker in faint ink unless its theme
````

### Lines 413–413

````
/// authors a foreground for that part.
````

### Lines 498–498

````
/// The `parity-history` bands with their own body: single-column below 160.
````

### Lines 503–503

````
/// 80–159 `parity-files` tag frames. Other sizes stay off.
````

### Lines 508–508

````
/// The `parity-browser` bands with their own body: single-column below 120, the
````

### Lines 509–509

````
/// wide card from 120 up to 160. Files plates and every other size stay off.
````

### Lines 527–527

````
// The task-sources chrome lives in the narrow band; the mid band keeps the
````

### Lines 528–528

````
// generic adaptive status and hints.
````

### Lines 533–533

````
/// Tag parity-browser status at 80–159 columns: cwd, then the size label. No branch.
````

### Lines 560–560

````
/// Tag parity-files status at 80–159 columns: cwd, then the size label. No branch.
````

### Lines 587–587

````
/// Retained oracle for the ParityBrowser status before StatusBar owned its paint.
````

### Lines 605–605

````
/// Tag parity-git-current status at 80×24, 100×30, and 120×40: cwd, branch, behind/ahead, dirty mark, then the size label.
````

### Lines 624–624

````
/// Tag parity-cargo status at 80×24, 100×30, and 120×40: cwd, branch, then the size label. No ahead count.
````

### Lines 649–649

````
/// Tag parity-task-sources status at 80×24: cwd, then the size label. No branch.
````

### Lines 666–666

````
/// Tag parity-git-batch status at 80×24, 100×30, and 120×40: cwd, then the size label. No branch.
````

### Lines 683–683

````
/// Retained oracle for the ParityFiles status before StatusBar owned its paint.
````

### Lines 701–701

````
/// Tag parity-history status at 80×24, 100×30, and 120×40: cwd, branch, then the size label. No ahead count.
````

### Lines 726–726

````
/// Tag first-use status: cwd, an 11-cell `/` meter at 91%, then the size label.
````

### Lines 756–756

````
/// Tag remote-host status: service cwd, detached tag, then the size label.
````

### Lines 772–772

````
/// Tag hard-cases status. The size label does not fit, so the row ends in `…`.
````

### Lines 773–773

````
/// 80×24 keeps only the path. 100×30 keeps the git marks and centers the docker warning.
````

### Lines 774–774

````
/// Wider frames also show the conflict count and the rebase mark.
````

### Lines 825–825

````
/// Tag docker-cleanup status: cwd without the tilde, then the size label.
````

### Lines 842–842

````
/// Tag disk-cleanup status: cwd, centered selection, then the 91% meter.
````

### Lines 881–881

````
/// Tag disk-cleanup footer. The last hint does not fit, so the row ends in `…`.
````

### Lines 882–882

````
/// 80×24 drops Delete, the artifact plan, and Rescan.
````

### Lines 883–883

````
/// 100×30 keeps Delete and drops the artifact plan and Rescan. Cancel scan misses the tag fit budget.
````

### Lines 884–884

````
/// Wider frames keep the artifact plan and Rescan.
````

### Lines 933–933

````
// Tag hint `fit` room is the width minus the left inset, the right inset, and the overflow gap.
````

### Lines 963–963

````
/// Tag hint `fit` cost. Each kept hint pays its width plus the gap. A cut reserves one more gap.
````

### Lines 980–980

````
/// Tag monorepo-root status: cwd, branch, ahead count, then the size label.
````

### Lines 997–997

````
/// Tag activities-multi status: the monorepo cluster, then `⠋` and `4 running` chips.
````

### Lines 1035–1035

````
/// Tag launch-failure status: child cwd, branch, ahead, dirty mark, failure, then the size label.
````

### Lines 1070–1070

````
/// Tag launch-failure footer. `Shift+F10 Menu` misses the status, so the row shows `…`.
````

### Lines 1101–1101

````
// 80×24 keeps the ellipsis in the budget, so `Restart` drops. 120×40 does not.
````

### Lines 1130–1130

````
/// Tag monorepo-child status: child cwd, branch, ahead count, dirty mark, then the size label.
````

### Lines 1192–1192

````
/// 100×30 docker footer. `Esc Clear` stays. `Ctrl+↑↓ Scope` misses by one column, so the meta `…` paints.
````

### Lines 1193–1193

````
/// No status. The wide plate keeps `Clean Docker c…`.
````

### Lines 1209–1209

````
/// 80×24 docker footer. `Esc Clear` stays and `Tab Preview` becomes the meta `…`.
````

### Lines 1210–1210

````
/// The wide plate keeps list order and the `Clean Docker c…` status.
````

### Lines 1250–1250

````
// Parity worlds have no `location.project`. These strings are the tag `draw_menu` labels.
````

### Lines 1251–1251

````
// Concept screens still show cwd; that path is the crumb those screens document.
````

### Lines 1337–1337

````
/// Retain the caller's authored theme before the host projects it to a terminal tier.
````

### Lines 1338–1338

````
///
````

### Lines 1339–1339

````
/// Existing constructors intentionally leave this unset. Embeddings that only pass an
````

### Lines 1340–1340

````
/// already-projected theme keep the existing footer behavior because its authorship
````

### Lines 1341–1341

````
/// cannot be recovered reliably from the projected colors. The readable overflow fallback
````

### Lines 1342–1342

````
/// is limited to the built-in recipe sets and token-only themes. Other recipe configuration
````

### Lines 1343–1343

````
/// can contain level-specific foreground intent that the theme API does not expose; in that
````

### Lines 1344–1344

````
/// case the fallback is suppressed, and a missing resolved foreground does not count as proof
````

### Lines 1345–1345

````
/// that the theme left the channel unconfigured. This consuming configuration step builds
````

### Lines 1346–1346

````
/// four private tier projections so drawing can retain full-theme validation without cloning
````

### Lines 1347–1347

````
/// the source theme and its recipes on every frame.
````

### Lines 1395–1395

````
/// Use an explicit simulated file snapshot for a read-only Files browser.
````

### Lines 1396–1396

````
///
````

### Lines 1397–1397

````
/// The snapshot is owned by this app instance and never reads the host filesystem.
````

### Lines 1412–1412

````
/// The app without the usage history.
````

### Lines 1413–1413

````
///
````

### Lines 1414–1414

````
/// The usage history is on by default, the way the frozen reference runs
````

### Lines 1415–1415

````
/// it. The no-history projection is a setting, not the default.
````

### Lines 1426–1426

````
/// Opens the dedicated, explicitly declared nextest input fixture without changing any
````

### Lines 1427–1427

````
/// ordinary scenario or the normal `nextest_args_world` trust defaults.
````

### Lines 2170–2170

````
// Leaving Files clears its preview before this route can be restored.
````

### Lines 2173–2173

````
// These routes paint reports without retaining an input owner. A saved ID
````

### Lines 2174–2174

````
// from chrome or an earlier route is not proof that the destination owns it.
````

### Lines 2603–2603

````
// Tag lockup starts at x + chrome_edge - 1. One more column before this bar matches the tag's post-lockup gap (its title pad is the other).
````

### Lines 2620–2620

````
// A disk plate has no query, so the bar would keep focus and bold its titles.
````

### Lines 2732–2732

````
// 80×24 has no room for the branch or the size label. The warning sits on the right, with `…` before it.
````

### Lines 2780–2780

````
// An explicit capability palette may author a deliberate collision at this tier. The
````

### Lines 2781–2781

````
// app-level fallback cannot tell whether that choice is intentional, so keep it intact.
````

### Lines 2800–2800

````
// Prefer semantic inverse/ghost ink. Some light-canvas themes have no projected ink
````

### Lines 2801–2801

````
// token at this tier, so the final candidate is their own canvas color, passed through
````

### Lines 2802–2802

````
// the same contrast helper. Every candidate is checked against this footer's actual
````

### Lines 2803–2803

````
// black background before use; Reset and low-contrast results are declined.
````

### Lines 2838–2838

````
// Preserve a direct recipe foreground, including a Reset/Clear that resolves to
````

### Lines 2839–2839

````
// no color. Theme::resolve alone cannot distinguish those from no authored ink.
````

### Lines 2881–2881

````
// Wide history keeps the deploy status. 80×24 and 100×30 have no room for it.
````

### Lines 2891–2891

````
// 100×30 fits `Ctrl+↑↓ Scope` and clips `Ctrl+G` to the meta `…`. No status.
````

### Lines 2899–2899

````
// 100×30 fits `Ctrl+↑↓ Scope` and clips `Ctrl+G` to the meta `…`. No status.
````

### Lines 2908–2908

````
// 100×30 fits `Ctrl+↑↓ Scope` and clips `Ctrl+G` to the meta `…`. No status.
````

### Lines 2930–2930

````
// The wide card keeps every hint. The title is cut to the ten cells left of the edge.
````

### Lines 2949–2949

````
// `Trust…` is two columns wider than `Open`. HintBar keeps the gap
````

### Lines 2950–2950

````
// before a later hint but not the overflow glyph, so `Ctrl+↑↓ Scope`
````

### Lines 2951–2951

````
// still fits and the `…` never paints. One undisplayed column makes
````

### Lines 2952–2952

````
// that hint miss the budget. The no-history plate stays on `Open`.
````

### Lines 2998–2998

````
// Single-column bands clip the last hint to a meta `…`. The wide
````

### Lines 2999–2999

````
// card fits every hint, so it keeps the theme overflow.
````

### Lines 3145–3145

````
/// One sim step of `world.tick`. That call moves `now_ms` only while the clock is running.
````

### Lines 3628–3628

````
/// Tag body starts under a 4-row header. `body` is the port's y+2 frame, so Home drops two more rows for the band and one for the tag footer gap.
````

### Lines 3638–3638

````
/// Tag `Tabs` active item for this boot: width `1 + label + 2`, bold primary on `#27272a`, accent `━`, then `#3a3a3a` `─`.
````

### Lines 3660–3660

````
// Tag `border_subtle` is `#3a3a3a`. The port `BorderSubtle` token is `#262626`.
````

### Lines 3679–3679

````
/// Collapsed activities band. Here sits four columns in. Two chips stay off-screen as `2›` and `+2`.
````

### Lines 3680–3680

````
/// 80×24 and 100×30 share the three visible chips. The markers sit nine columns from the right.
````

### Lines 3744–3744

````
/// Tag activity chips sit on the Here row. The live strip stays on the row above for later seeks.
````

## `crates/holla-ui/src/dispatch.rs`

- Original source SHA-256: `bb291b9e638009621fab0425598078c7a57cf7505845fc6efbcd3cbd041e5213`
- Excerpts are verbatim and retain their input line numbers.

### Lines 65–65

````
// Every reserved typed producer stays off the legacy fixed-command preview route.
````

### Lines 174–174

````
// Raw identity matters: recent/pin wrappers intentionally keep their ordinary static route.
````

### Lines 367–367

````
// Direct generic confirmation must not bypass either typed input producer.
````

## `crates/holla-ui/src/layout.rs`

- Original source SHA-256: `0bb9ae361e30f6f4715a4f70d82c7e98467f34490e5bed548c3f7b46659318b3`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! The width-band layout policy.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! A screen picks its layout from the room it has, not from the size a
````

### Lines 4–4

````
//! capture happened to use. Every viewport falls into one band, and the bands
````

### Lines 5–5

````
//! are the only thing a painter asks about the size of the terminal.
````

### Lines 7–7

````
/// The narrowest viewport the shell renders.
````

### Lines 9–9

````
/// The shortest viewport the shell renders.
````

### Lines 12–12

````
/// The layout band a viewport falls into.
````

### Lines 15–15

````
/// Below 80 columns: no room for a column of its own.
````

### Lines 17–17

````
/// 80 to 99 columns: one column, no wide parts.
````

### Lines 19–19

````
/// 100 to 119 columns: one column with the mid parts.
````

### Lines 21–21

````
/// 120 columns and up: the wide card.
````

### Lines 26–26

````
/// The band a viewport of this width falls into.
````

### Lines 36–36

````
/// The band a rectangle falls into.
````

## `crates/holla-ui/src/screens/args.rs`

- Original source SHA-256: `8079c401ba4cf5b8e00afaec8468b3b5c8b6cdbe2113268541cfbbd68431c9c5`
- Excerpts are verbatim and retain their input line numbers.

### Lines 104–104

````
/// Native, ephemeral editor for one producer-backed task request.
````

### Lines 293–293

````
/// Quotes each structured argument and escapes terminal controls for safe review text.
````

## `crates/holla-ui/src/screens/disk.rs`

- Original source SHA-256: `03909bd86e3ea701f31512696bf240f44d5e56d6e66f4bd47d3fac1c8cffcfa4`
- Excerpts are verbatim and retain their input line numbers.

### Lines 71–71

````
/// Inspect an owner without claiming layer receipts. A later native Dialog/List
````

### Lines 72–72

````
/// update gets the same unclaimed lifecycle intents when the whole owner set is safe.
````

### Lines 290–290

````
/// Service only safe lifecycle buckets while hidden or in a too-small frame.
````

### Lines 291–291

````
/// Dialog.update remains the owner of its Layer receipts and dismissal state.
````

### Lines 316–316

````
// The broad Dialog update reads all of these owners. If one bucket
````

### Lines 317–317

````
// contains input, leave the entire Dialog state untouched and report
````

### Lines 318–318

````
// both unsupported input and any safe lifecycle buckets it cannot service.
````

### Lines 334–334

````
/// TooSmall keeps the active chooser; safe focus/layer lifecycle still advances.
````

### Lines 339–339

````
/// A retired chooser may be released only after its actual owners and layer
````

### Lines 340–340

````
/// disappear from the published frame, focus ring, and pointer capture.
````

## `crates/holla-ui/src/screens/disk_scan.rs`

- Original source SHA-256: `8ab4a41e28cbf5f182b0db5cbf42b4cc44610841a175b34c81ce25d2d246595a`
- Excerpts are verbatim and retain their input line numbers.

### Lines 200–200

````
/// Persistent, read-only browser state. The controller survives every open and rescan.
````

### Lines 351–351

````
/// Cancel before a route transition. This method does not poll after the route leaves.
````

### Lines 394–394

````
/// Fallback for an unexpected route mutation after it has already happened.
````

### Lines 681–681

````
// Keep draining the retired identity while any owner from its
````

### Lines 682–682

````
// last published frame is still addressable. IDs remain bound
````

### Lines 683–683

````
// to that presentation and can never service the replacement.
````

### Lines 1225–1225

````
// Called during update: `cx.area` sees the last published frame. The expected
````

### Lines 1226–1226

````
// rectangles come from the current route callback's body, matching the exact
````

### Lines 1227–1227

````
// `areas` values used by `draw` when it registers these generation-bound IDs.
````

## `crates/holla-ui/src/screens/files.rs`

- Original source SHA-256: `334480b09873c7d67565aff4a2b5f6d210172093e5f3836e95295ff2665c9a03`
- Excerpts are verbatim and retain their input line numbers.

### Lines 134–134

````
/// Classify one lifecycle bucket before a retained widget is updated or retired.
````

### Lines 135–135

````
/// Iteration drains the whole bucket, so a caller that rejects `Other` must report
````

### Lines 136–136

````
/// `UndeliveredIntent` itself; otherwise Runtime's undrained scan cannot see it.
````

### Lines 168–168

````
// The caller admitted only empty/focus-only buckets. FocusIn is not
````

### Lines 169–169

````
// handled by update_focus_out, so List requires this same-pass fallback.
````

### Lines 203–203

````
/// Private synchronous state for the read-only Files route.
````

### Lines 301–301

````
// Retain the list state while previewing. Service its owner bucket
````

### Lines 302–302

````
// only during Settle; never process Event input behind the preview.
````

### Lines 319–319

````
// A hidden Preview can receive its real focus loss on any update
````

### Lines 320–320

````
// cause. Service only lifecycle buckets here; never replay input
````

### Lines 321–321

````
// from a mixed bucket after the route hides.
````

## `crates/holla-ui/src/screens/home.rs`

- Original source SHA-256: `0d1818801cf9b2ccb6900393326ddd184f2823b7af0e72329ce294e0359d5a74`
- Excerpts are verbatim and retain their input line numbers.

### Lines 37–37

````
/// History-on tag plates. Set from `HOLLA_NO_HISTORY=0`.
````

### Lines 594–594

````
/// Tag finder grey. Junie `FgStep::Muted` is `#808080`.
````

### Lines 597–597

````
/// Focused tag query. The whole value is underlined in secondary, not only the caret.
````

### Lines 607–607

````
/// Home body is already one column in. Tag section titles start at screen column 4.
````

### Lines 609–609

````
/// Header recipe is bold secondary. The pad must stay primary; glyphs are retinted below.
````

### Lines 764–764

````
// Use the row owner's full allocation to keep optional columns responsive
````

### Lines 765–765

````
// when NavList reserves a scrollbar or its leading slots.
````

### Lines 1125–1125

````
/// Single-column band body for the concept screens. The 80–119 band replaces the
````

### Lines 1126–1126

````
/// frozen 80×24 and 100×30 plate frames; overflow gets a black thumb and the preview fade.
````

### Lines 1132–1132

````
/// Frozen columns. Same grid as 100×30. Only the 80×24 frame reaches this painter.
````

### Lines 1133–1133

````
/// `parity-history` body. Single-column tag list below 120 with the frozen
````

### Lines 1134–1134

````
/// columns, the wide card in the 120 band, and the plain action list from 160 up.
````

### Lines 1152–1152

````
/// Frozen columns. The explore chips keep the accent fill. The system row does not.
````

### Lines 1153–1153

````
/// Single-column in the 80–119 band, wide card from 120.
````

### Lines 1165–1165

````
/// Frozen columns. The explore chips keep the accent fill. The system row does not.
````

### Lines 1208–1208

````
/// Wide card. Explore chips keep the accent fill. The system row does not.
````

### Lines 1256–1256

````
/// Selected explore chips. The gutter is accent-on-accent. `›` sits in the gap before the first chip.
````

### Lines 1277–1277

````
/// `parity-browser` body. The single-column bands keep the frozen columns; the
````

### Lines 1278–1278

````
/// 120 band keeps the wide card; from 160 up the reference shows the plain
````

### Lines 1279–1279

````
/// action list, so the view painter steps aside.
````

### Lines 1297–1297

````
/// Wide card band for the replay families. From 120 up the split card keeps
````

### Lines 1298–1298

````
/// the auto-sized tag columns.
````

### Lines 1323–1323

````
/// Default tag columns in the 80–119 band.
````

### Lines 1324–1324

````
/// The action row middle-truncates `Runs` below 100 so the line fits. Do not write cursor bytes.
````

### Lines 1338–1338

````
/// Shrink `Runs` so `→ {runs} · {scope} · {risk}` fits the action row.
````

### Lines 1364–1364

````
/// Frozen columns in the 80–119 band.
````

### Lines 1365–1365

````
/// Git-batch, cargo, git-current, browser, files, history, discovery, and every other size stay off.
````

### Lines 1366–1366

````
/// Do not write cursor bytes.
````

### Lines 1389–1389

````
/// Default tag columns in the 80–119 band, wide card from 120.
````

### Lines 1398–1398

````
/// Default tag columns in the 80–119 band, wide card from 120.
````

### Lines 1411–1411

````
/// Size-free single-column body. Callers keep their own layout policy.
````

### Lines 1433–1433

````
// Short lists never reach this row. A longer list clips here instead of covering it.
````

### Lines 1437–1437

````
// The thumb owns the last column. Leave the cell before it blank, so a truncated reason stops one earlier.
````

### Lines 1505–1505

````
/// `hard-cases` body. Below the 120-wide breakpoint the view keeps its single-column
````

### Lines 1506–1506

````
/// tag list; from 120 up it uses the split master/detail card.
````

### Lines 1519–1519

````
/// `parity-discovery` body. Below the 120-wide breakpoint the view keeps its
````

### Lines 1520–1520

````
/// single-column tag list; from 120 up it uses the split master/detail card.
````

### Lines 1762–1762

````
// Unfocused explore gutter is ink-on-ink, so the cell reads as an empty black column.
````

### Lines 1788–1788

````
// A selected row keeps the solid accent. Fade is only the unselected tail.
````

### Lines 1808–1808

````
// Tag row chrome bolds only the label. Kind, scope, reason, and risk stay plain.
````

### Lines 2003–2003

````
/// Bottom edge of an overflowing preview. `4/5` is the inner keep, `11/20` the outer.
````

### Lines 2081–2081

````
/// List thumb on the narrow plate. Same geometry as the preview thumb, on black not `#111111`.
````

## `crates/holla-ui/src/screens/mod.rs`

- Original source SHA-256: `5dc005ef3c5559605ceb4469396a04f855829da9d6a300b9f12e64caf9fc49c5`
- Excerpts are verbatim and retain their input line numbers.

### Lines 21–21

````
/// Keep the old report cells inside each allocation. The new shared block
````

### Lines 22–22

````
/// must also leave every cell outside that allocation unchanged.
````

## `crates/holla-ui/src/screens/port_args.rs`

- Original source SHA-256: `6d038a44301124ef14f792a2f19588edbeb179edfdd2e480118e607933df7c35`
- Excerpts are verbatim and retain their input line numbers.

### Lines 92–92

````
/// Native one-field editor for a captured port lookup request.
````

## `crates/holla-ui/src/screens/task_run.rs`

- Original source SHA-256: `371742636a9926382cf094d18c1757d2ec9ae6e96de4bf20e47aed7d50355e7b`
- Excerpts are verbatim and retain their input line numbers.

### Lines 22–22

````
/// Input state retained until its exact route, run, and prompt owner has no queued lifecycle.
````

### Lines 29–29

````
/// Private input state for one exact route, run, and prompt owner.
````

### Lines 127–127

````
// A key routed for the previous prompt cannot reach the new prompt.
````

### Lines 182–182

````
/// Service this covered screen's exact input owners without admitting edits or actions.
````

### Lines 196–196

````
/// Retain this exact input owner until its queued lifecycle is safely drained.
````

### Lines 206–206

````
/// Poll retired owners without accepting edits and report when all can be dropped.
````

### Lines 227–227

````
// Keep one quiet owner-poll cycle after routed lifecycle intents arrive.
````

## `crates/jackin-preview-app/src/app.rs`

- Original source SHA-256: `a8ae78bc09f84989103eec1c388cf38909c0576307e02ff2f80501aecc3677f6`
- Excerpts are verbatim and retain their input line numbers.

### Lines 181–181

````
// The shared Roles painter places its native Save owner on this fixed row.
````

### Lines 1040–1040

````
// The manager hint is `n New`. Capture beats the list.
````

### Lines 1047–1047

````
// The focused nav button would consume Enter before the bubble
````

### Lines 1048–1048

````
// binding. Capture it so this row opens the drawer.
````

### Lines 1051–1051

````
// `2` then Enter selects Mounts. Capture beats the mount button,
````

### Lines 1052–1052

````
// so Enter confirms the tab and does not edit the row.
````

### Lines 1056–1056

````
// `3` then Enter selects Roles. Capture beats the default-role
````

### Lines 1057–1057

````
// button, so Enter confirms the tab and does not set a status.
````

### Lines 1061–1061

````
// `2` then Enter leaves General for Mounts. Capture beats Save.
````

### Lines 1065–1065

````
// `3` then Enter leaves General for Environments. Capture beats Save.
````

### Lines 1069–1069

````
// `4` then Enter leaves General for Agents. Capture beats Save.
````

### Lines 1073–1073

````
// `5` then Enter leaves General for Trust. Capture beats Save.
````

### Lines 1077–1077

````
// Backtab already stayed on the dialog. Enter validates it.
````

### Lines 1078–1078

````
// Without this capture the focused start control would leave the page.
````

### Lines 1083–1083

````
// Crossterm reports ESC [ Z as BackTab with Shift still set.
````

### Lines 1097–1097

````
// Three Tabs land on Keep awake. Capture beats focus traversal.
````

### Lines 1102–1102

````
// The shell tab before the start button is Capsule. Backtab must
````

### Lines 1103–1103

````
// stay on the empty dialog so Enter can show Required.
````

### Lines 1117–1117

````
// `2` arms Mounts. Enter confirms it. Capture beats a focused control.
````

### Lines 1125–1125

````
// `3` arms Environments. Enter confirms it. Capture beats editor roles.
````

### Lines 1135–1135

````
// `4` arms Agents. Enter confirms it. Capture beats editor environments.
````

### Lines 1146–1146

````
// `5` arms Trust. Enter confirms it. Capture beats the trust button.
````

### Lines 1182–1182

````
// `i` dirties the Mounts row. Accounts and the manager keep this key.
````

### Lines 1186–1186

````
// The accounts page owns the keys its hint bar names: edit, toggle,
````

### Lines 1187–1187

````
// remove and usage. The app-wide bindings for those keys stop here.
````

### Lines 1229–1229

````
// `t` reveals the approved run id. `d` still toggles debug telemetry.
````

### Lines 1231–1231

````
// `x` opens cancel confirm. Accounts keep `x` for remove.
````

### Lines 1577–1577

````
// Short supported layouts use the normal Roles renderer, whose
````

### Lines 1578–1578

````
// controls follow the measured content area instead of this fixed row.
````

### Lines 1941–1941

````
// The tab menu takes the keys. The pane caret stays the steady bar.
````

### Lines 1946–1946

````
// The frozen capsule palette is a 60×19 card on the screen center,
````

### Lines 1947–1947

````
// listing the 20 host commands with a live count.
````

### Lines 2374–2374

````
// The frozen app centred the card with ratatui Flex::Center: the odd
````

### Lines 2375–2375

````
// leftover row goes above, not to the floor split.
````

### Lines 2383–2383

````
// The frozen info card carried its title and meta in the border.
````

### Lines 2386–2386

````
// The frozen app centred the card with a floor split of the free
````

### Lines 2387–2387

````
// rows (popup Center), not the Flex::Center split.
````

### Lines 2501–2501

````
// One edit: the selected host mount becomes read-only and cloned.
````

### Lines 2502–2502

````
// `r` and `i` both land here. A second press does not undo it.
````

### Lines 2812–2812

````
/// Any transient overlay chrome above the page.
````

### Lines 2829–2829

````
/// True when a filter query narrows the accounts tree.
````

### Lines 2830–2830

````
///
````

### Lines 2831–2831

````
/// The query stays after it is committed, so the tree keeps the filter
````

### Lines 2832–2832

````
/// while the list takes the keys again.
````

### Lines 2837–2837

````
/// True when the filter field holds the caret.
````

### Lines 3090–3090

````
// A stale or redirected close has no Accounts-list resume to apply.
````

### Lines 3091–3091

````
// A successful return stays pending until the real List update below.
````

### Lines 3264–3264

````
/// Open the account form on the selected account.
````

### Lines 3265–3265

````
///
````

### Lines 3266–3266

````
/// The form carries the current name, provider and credential, so the
````

### Lines 3267–3267

````
/// edit starts from the account as the registry knows it.
````

### Lines 3805–3805

````
// The prefix line is transient. The open tab menu keeps the session
````

### Lines 3806–3806

````
// status row, the same row the idle capsule paints.
````

### Lines 4635–4635

````
// Picker also settles its FilterList focus lifecycle after the layer closes.
````

### Lines 4636–4636

````
// Only actions from a layer open at the start of this update are semantic choices.
````

### Lines 4943–4943

````
// Focus sits on the manager shell, so the list never sees Enter.
````

### Lines 5016–5016

````
// The modal keeps the selected list entry logically active, so settle its real
````

### Lines 5017–5017

````
// focus transition without allowing an underlying list action.
````

### Lines 5168–5168

````
// A committed row choice selects in the detail view. Opening the
````

### Lines 5169–5169

````
// drawer remains an explicit Enter or inspector action.
````

### Lines 5176–5176

````
// The filter field owns the caret while it is open. A committed query
````

### Lines 5177–5177

````
// keeps the filter and gives the keys back to the list.
````

### Lines 5180–5180

````
// A click outside the field dismisses it, the way the
````

### Lines 5181–5181

````
// reference dialog dismisses on an outside click. The query
````

### Lines 5182–5182

````
// stays: the tree has been filtered with it all along.
````

### Lines 5209–5209

````
// An empty query keeps the filter open: the caret stays in
````

### Lines 5210–5210

````
// the field so the query can still be typed.
````

### Lines 5213–5213

````
// Enter leaves the query. The approved filter frame has no
````

### Lines 5214–5214

````
// caret, so focus must leave the field before draw.
````

### Lines 5224–5224

````
// The field is gone: the list takes the focus back.
````

### Lines 5392–5392

````
/// Save the edit of an account: the name and the provider move, the
````

### Lines 5393–5393

````
/// credential stays the one the registry knows.
````

### Lines 6037–6037

````
// The 120×40 vertical split keeps the agent picker. A leftover
````

### Lines 6038–6038

````
// binding from `"` must not replace it with the account list.
````

### Lines 6366–6366

````
// The overview row refreshes every account, the way
````

### Lines 6367–6367

````
// the hint bar names it.
````

### Lines 6509–6509

````
// The approved frame is the agent picker titled
````

### Lines 6510–6510

````
// "New tab". Leave the session bar in place.
````

### Lines 6534–6534

````
// F10 opens the file menu only on the paused 120×40 returning
````

### Lines 6535–6535

````
// manager. Other sizes and scenarios keep the live page.
````

### Lines 6737–6737

````
// The approved frame is the agent picker, not the Claude
````

### Lines 6738–6738

````
// account list. Leave status clear so the session bar
````

### Lines 6739–6739

````
// stays the baseline row.
````

### Lines 6741–6741

````
// Do not set picker_mode. The account-picker update clears
````

### Lines 6742–6742

````
// that mode when its layer is closed, and this frame is not
````

### Lines 6743–6743

````
// that layer.
````

### Lines 6764–6764

````
// The 120-col stamp keys off this status. Other sizes keep
````

### Lines 6765–6765

````
// the session bar and put the zoom note on the hint row.
````

### Lines 6810–6810

````
// Space is a capture binding, so the prefix intent never sees it.
````

### Lines 6858–6858

````
// The accounts list is not the focused control, so Down never
````

### Lines 6859–6859

````
// reaches it. Four downs must still land on Claude · Work.
````

### Lines 6956–6956

````
// The field opens on the committed query, so the filter can be
````

### Lines 6957–6957

````
// refined instead of retyped.
````

### Lines 6973–6973

````
// The drawer shows the inspected account: the list cursor
````

### Lines 6974–6974

````
// follows it.
````

### Lines 6994–6994

````
// The manager list is not the focused control, so Enter never
````

### Lines 6995–6995

````
// reaches it. This row still opens the launch picker.
````

### Lines 7000–7000

````
// Enter on this row opens the drawer. Four Downs without Enter
````

### Lines 7001–7001

````
// stay on the detail frame, and every other Enter stays live.
````

### Lines 7118–7118

````
// `2` alone stays on General. Enter is what leaves it.
````

### Lines 7134–7134

````
// `3` alone stays on General. Enter is what leaves it.
````

### Lines 7150–7150

````
// `4` alone stays on General. Enter is what leaves it.
````

### Lines 7166–7166

````
// `5` alone stays on General. Enter is what leaves it.
````

### Lines 7773–7773

````
// A wide split stays mounted when a master-row press takes focus from detail.
````

### Lines 7827–7827

````
/// The pinned detail drawer: the saved split painted through the shared
````

### Lines 7828–7828

````
/// drawer presentation (title focus, rule-grey box, drawer hints).
````

### Lines 7983–7983

````
// Boot sets last_refresh to now. The fixture refresh is 3s before the
````

### Lines 7984–7984

````
// paused clock, which is the age the manager card shows.
````

### Lines 8639–8639

````
// Baseline 256 trace ink is cube 111, not the generic neighbor 117.
````

### Lines 8677–8677

````
// Brand text on the accent chip is ramp 232. Ghost glyphs on black
````

### Lines 8678–8678

````
// are ramp 238. The stage checks use cube 71, not the accent.
````

### Lines 8716–8716

````
// `a` on the paused accounts list. The start button is not the dialog.
````

### Lines 8792–8792

````
// Backtab then Enter. The control row includes the focus gutter.
````

### Lines 8853–8853

````
// Row 0 (y = body.y): Display name *
````

### Lines 8865–8865

````
// Row 1 (y = body.y + 1): Name input field
````

### Lines 8884–8884

````
// Row 4 (y = body.y + 4): Purpose label optional
````

### Lines 8896–8896

````
// Row 5 (y = body.y + 5): Purpose input field
````

### Lines 8912–8912

````
// Row 8 (y = body.y + 8): Provider
````

### Lines 8919–8919

````
// Row 9 (y = body.y + 9): Provider dropdown
````

### Lines 8940–8940

````
// Row 12 (y = body.y + 12): Provider runtime note
````

### Lines 8952–8952

````
// Row 14 (y = body.y + 14): Credential source
````

### Lines 8959–8959

````
// Rows 15..17 (y = body.y + 15..17): Radios
````

### Lines 8991–8991

````
// Row 19 (y = body.y + 19): 1Password reference
````

### Lines 8998–8998

````
// Row 20 (y = body.y + 20): not chosen + Choose button
````

### Lines 9009–9009

````
// Register control IDs for focus/tab navigation in test harness
````

### Lines 9047–9047

````
/// Enter on the accounts list opens the drawer of the selected account.
````

### Lines 9105–9105

````
// The paused baseline page keeps its exact historical cells. Its
````

### Lines 9106–9106

````
// visible crumb remains the tab-focus stop; the live Tabs and NavList
````

### Lines 9107–9107

````
// controls are drawn after the baseline state changes.
````

### Lines 9593–9593

````
// ctrl-b " and ctrl-b c on the paused capsule open the agent picker.
````

### Lines 9594–9594

````
// Zoom, the tab menu, and non-junie themes stay off this path.
````

### Lines 9641–9641

````
// ctrl-b z on the paused 120×40 capsule. The bare capsule stays put.
````

### Lines 9682–9682

````
// Gap: the zoom frame draws no input row, so no real field owns
````

### Lines 9683–9683

````
// the caret. The cursor stays absent until a later wave draws the
````

### Lines 9684–9684

````
// field; no fixed cell is forced (same for the 80-col zoom).
````

### Lines 9691–9691

````
// ctrl-b m on the paused 80×24 capsule. The 120×40 menu, new tab,
````

### Lines 9692–9692

````
// palette, zoom, and split stay on their own frames.
````

### Lines 9720–9720

````
// ctrl-b m on the paused 120×40 capsule. Every pane keeps the blank
````

### Lines 9721–9721

````
// input row so the scrolled transcript matches the frozen frame.
````

### Lines 9737–9737

````
// ctrl-b space on the paused 120×40 capsule. The panes keep the
````

### Lines 9738–9738

````
// same shortened transcript as the tab menu, under the modal dim.
````

### Lines 9758–9758

````
// The bar sits on the capsule line, outside the popup clip. Record it
````

### Lines 9759–9759

````
// on the open menu layer or the runtime hides the cursor.
````

### Lines 9803–9803

````
/// Wide resting panes keep one row for the horizontal bar.
````

### Lines 9804–9804

````
///
````

### Lines 9805–9805

````
/// The shared viewport no longer paints that bar. The paused capsule
````

### Lines 9806–9806

````
/// still does: full width when the lines fit, and one column narrower
````

### Lines 9807–9807

````
/// once a vertical bar is showing. A second pass uses the shortened
````

### Lines 9808–9808

````
/// height, because that bar can be what makes the pane overflow.
````

### Lines 9816–9816

````
// Capsule viewports reserve a vertical bar plus one gap column.
````

### Lines 9976–9976

````
// The 120 picker, the tab menu, and the command palette shorten
````

### Lines 9977–9977

````
// every pane by one row. Other picker sizes shorten a pane only
````

### Lines 9978–9978

````
// when its lines need the horizontal bar. A resting wide pane
````

### Lines 9979–9979

````
// keeps that row too. Only the focused pane reserves the input
````

### Lines 9980–9980

````
// at rest.
````

### Lines 10010–10010

````
// The menu, palette, and agent-picker frames reserve the last
````

### Lines 10011–10011

````
// row and put the horizontal bar there. The 120 picker keeps
````

### Lines 10012–10012

````
// its own underlay repairs. A resting wide pane does the same
````

### Lines 10013–10013

````
// on its own reserved row.
````

### Lines 10151–10151

````
// The only cached value is a committed viewport receipt. Text, affixes,
````

### Lines 10152–10152

````
// source membership, status and active identity are rebuilt each pass.
````

### Lines 10165–10165

````
// Accent on the tab rule dims to faint. The frozen palette
````

### Lines 10166–10166

````
// keeps that run ghost, and five hint-boundary cells stay bold.
````

### Lines 10178–10178

````
// Card fills keep truecolor RGB. The shell already uses the
````

### Lines 10179–10179

````
// baseline index ladder.
````

### Lines 10318–10318

````
// Wide rows keep crumb, a middle-cut container, then the
````

### Lines 10319–10319

````
// running count. Narrow rows stay on the ellipsis branches.
````

### Lines 10342–10342

````
// Muted padding hugs the container. The other gap cell stays
````

### Lines 10343–10343

````
// the menubar fill.
````

### Lines 10363–10363

````
// The crumb would cover a menu title. Keep the titles and
````

### Lines 10364–10364

````
// mark the clipped identity with one muted ellipsis.
````

### Lines 10386–10386

````
/// Baseline status plan: gap 3, edge 1, meter `1 + 10 + 7`, drop lowest
````

### Lines 10387–10387

````
/// priority, then paint `…` for what fell off.
````

### Lines 10438–10438

````
// (width, count) per shrink group.
````

### Lines 10762–10762

````
/// Second metered quota window. Session stays the first meter. Weekly
````

### Lines 10763–10763

````
/// is this one, and it drops before Session when the row is short.
````

### Lines 10887–10887

````
// 16-color faint is dark gray. The frozen hint ellipsis
````

### Lines 10888–10888

````
// stays the light gray.
````

### Lines 11011–11011

````
// ui.full() inside the layer is the popup clip, so decide out here.
````

### Lines 11020–11020

````
// Menu danger-soft drifted with the port token. The frozen
````

### Lines 11021–11021

````
// tab menu still uses the baseline rose.
````

### Lines 11029–11029

````
// The popup clip is the live menu. Paint with the screen rect so
````

### Lines 11030–11030

````
// only overlapping menu cells replace that buffer.
````

### Lines 11041–11041

````
// The card covers the tab rule. That pad is on this layer, so a
````

### Lines 11042–11042

````
// screen edit after composite cannot see it.
````

### Lines 11063–11063

````
// Reduced color levels still read the approved cancel frame off the
````

### Lines 11064–11064

````
// page until the shared capability ladders reproduce the frozen dialog
````

### Lines 11065–11065

````
// inks (ghost 238 and danger 203 at 256, the 16-color card dim).
````

### Lines 11219–11219

````
// The split control is registered before the Accounts children so its
````

### Lines 11220–11220

````
// container hit region cannot cover their rows. Preserve the page's
````

### Lines 11221–11221

````
// initial keyboard owner explicitly without overriding later seam focus.
````

### Lines 11222–11222

````
// Bootstrap runs before the first viewport is available; an undersized
````

### Lines 11223–11223

````
// frame will reconcile this request against the controls it actually draws.
````

### Lines 11250–11250

````
// A completed Editor save can move to Manager before the old
````

### Lines 11251–11251

````
// Save owners receive their commandless focus-settle pass.
````

### Lines 11267–11267

````
// A route command can retire the Accounts List before the commandless
````

### Lines 11268–11268

````
// focus-settle pass. Deliver only its queued focus transition here.
````

### Lines 11305–11305

````
// Runtime focus settlement repeats the root command. Its route action was
````

### Lines 11306–11306

````
// already consumed; update the new host's controls so they can claim focus
````

### Lines 11307–11307

````
// notifications.
````

### Lines 11317–11317

````
// The route command was already applied above. Settle only the List owner's
````

### Lines 11318–11318

````
// actual focus transition before acknowledging its exact close result.
````

### Lines 11331–11331

````
// CMD_SAVE was already applied on the Event pass. Runtime repeats the root
````

### Lines 11332–11332

````
// command while settling focus, so update only the Settings owners that
````

### Lines 11333–11333

````
// receive those focus notifications; do not replay the save action.
````

### Lines 11392–11392

````
// The approved menu sits on the paused returning manager, first row,
````

### Lines 11393–11393

````
// with no other overlay. Down, Tab, and other routes stay live.
````

### Lines 11418–11418

````
// Enter on the paused 120×40 returning manager opens this picker.
````

### Lines 11419–11419

````
// The closed manager, F10, `?`, inspect, and the detail drawer stay off.
````

### Lines 11444–11444

````
// This is the action contract, independent of the visual host's theme.
````

### Lines 11445–11445

````
// Keep the same narrow Returning shell, current-row, and overlay scope.
````

### Lines 11476–11476

````
// Enter on the paused 120×40 hard-cases manager opens this picker.
````

### Lines 11477–11477

````
// The returning picker, F10, `?`, inspect, and the closed manager stay off.
````

### Lines 11978–11978

````
// Baseline success and error are not the port tokens.
````

### Lines 12097–12097

````
// The mono palette's overlay is black. The frozen brand
````

### Lines 12098–12098

````
// and active tab keep the dark-gray chip.
````

### Lines 12127–12127

````
// Register the sole List control before content to preserve focus
````

### Lines 12128–12128

````
// order; its keyed row hits follow the content Panel below.
````

### Lines 12146–12146

````
// Let the List rows win hit tests over the content Panel chrome.
````

### Lines 12178–12178

````
// The paused launch pages are page content: they must land before the
````

### Lines 12179–12179

````
// layer pass so dialog chrome maps the real underlay ink.
````

### Lines 12211–12211

````
// One Tab from the manager moves focus to Accounts and does not
````

### Lines 12212–12212

````
// open the live detail. The approved 120×40 frame is still the
````

### Lines 12213–12213

````
// detail drawer, so paint that only for this focus.
````

### Lines 12272–12272

````
// Reduced color levels still read the approved info frame off the
````

### Lines 12273–12273

````
// page until the shared capability ladders reproduce the frozen card
````

### Lines 12274–12274

````
// inks at 256, 16 colors and mono.
````

### Lines 12474–12474

````
// The field is gone: the list takes the focus back.
````

### Lines 12498–12498

````
// Cancelling the form returns to the list it came from.
````

### Lines 12760–12760

````
/// The form row of a provider.
````

### Lines 12807–12807

````
/// The account the tree cursor stands on.
````

### Lines 12816–12816

````
/// The tree row of one account.
````

## `crates/jackin-preview-app/src/app/accounts_page.rs`

- Original source SHA-256: `ee96c34dec6cea4447e1e6bb57168f6f9b6e1dceab2192c03bb812805bdc5d1d`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Accounts page: the account tree, the registry inspector, and the drawer.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! Every value on this page comes from the live registry. The tree rows are
````

### Lines 4–4

````
//! the accounts of the world, the inspector is the registry summary, and the
````

### Lines 5–5

````
//! drawer shows the selected node. The page picks its layout from the area it
````

### Lines 6–6

````
//! is given: a wide area gets the list beside the inspector, a narrow area
````

### Lines 7–7

````
//! gets the list above the inspector band. Nothing here keys on a screen size.
````

### Lines 25–25

````
/// The layout switches to the inspector beside the list when the area is wide
````

### Lines 26–26

````
/// enough for both columns and tall enough for the shared painters.
````

### Lines 27–27

````
///
````

### Lines 28–28

````
/// The reference splits the accounts page when the body beside the list is
````

### Lines 29–29

````
/// 100 columns wide, so the page needs 102 columns of its own.
````

### Lines 33–33

````
/// The width of the account list pane of the shared accounts painters.
````

### Lines 34–34

````
///
````

### Lines 35–35

````
/// The shared painters own this geometry; the page needs it to cut a row
````

### Lines 36–36

````
/// name before the pane edge.
````

### Lines 39–39

````
/// The column of the panel values, measured from the panel content edge.
````

### Lines 42–42

````
/// The width of the quota meter bar of the panel.
````

### Lines 45–45

````
/// The registration policy of the accounts page.
````

### Lines 510–510

````
/// What a tree row stands for.
````

### Lines 513–513

````
/// The registry overview.
````

### Lines 515–515

````
/// A provider group.
````

### Lines 517–517

````
/// One account, by registry id.
````

### Lines 519–519

````
/// The section of accounts whose quota is not visible.
````

### Lines 521–521

````
/// The add-account action row.
````

### Lines 525–525

````
/// One row of the account tree.
````

### Lines 537–537

````
/// The account tree of the live registry.
````

### Lines 543–543

````
/// The account id of the row at `index`, if that row is an account.
````

### Lines 552–552

````
/// The account tree for the app: the registry, filtered by the live query.
````

### Lines 582–582

````
// A filtered tree shows the matches only: the sentinel section and the
````

### Lines 583–583

````
// add-account row are not part of the query.
````

### Lines 648–648

````
// A discovered account says so in the tree.
````

### Lines 671–671

````
/// The alert mark of an account: `!` for a blocked account, `▲` for a warning.
````

### Lines 685–685

````
/// The state tail of an account: freshness, lifecycle, or issue.
````

### Lines 709–709

````
/// The tail of an account whose usage failed: the issue decides the word.
````

### Lines 721–721

````
/// The age of the last good usage sample, without the "ago" suffix.
````

### Lines 730–730

````
/// True when the account cannot show a quota window.
````

### Lines 741–741

````
/// Paint the accounts page into `area`.
````

### Lines 754–754

````
// Keep the original fixed39 page as a fail-closed rendering
````

### Lines 755–755

````
// path if the split cannot resolve a real detail rectangle.
````

### Lines 901–901

````
/// The filter field, while it holds the caret.
````

### Lines 902–902

````
///
````

### Lines 903–903

````
/// The field lives on the blank row under the header, so the committed frame
````

### Lines 904–904

````
/// keeps that row empty and the caret is the only sign of the filter.
````

### Lines 926–926

````
/// The list beside the inspector.
````

### Lines 986–986

````
/// The list above the inspector band.
````

### Lines 1008–1008

````
// The range shows only when the tree does not fit on the screen.
````

### Lines 1048–1048

````
/// Register the add-account row of the tree as the add control.
````

### Lines 1049–1049

````
///
````

### Lines 1050–1050

````
/// The region follows the painted row: the tree scrolls, so the row index and
````

### Lines 1051–1051

````
/// the row on the screen are not the same thing.
````

### Lines 1067–1067

````
// The painted row of the tree, counted from the first row of the box.
````

### Lines 1122–1122

````
/// The header of the page: the crumb, the status, and the cluster.
````

### Lines 1123–1123

````
///
````

### Lines 1124–1124

````
/// The crumb carries the app message, so a saved account or a help note stays
````

### Lines 1125–1125

````
/// visible. The status and the cluster follow it when they still fit; a crumb
````

### Lines 1126–1126

````
/// that does not fit the header at all drops the whole right side.
````

### Lines 1137–1137

````
// The value carries the gap behind the running mark.
````

### Lines 1139–1139

````
// The brand and the menus take the left of the header.
````

### Lines 1155–1155

````
/// The column width of a text.
````

### Lines 1160–1160

````
/// The area of the list pane.
````

### Lines 1161–1161

````
///
````

### Lines 1162–1162

````
/// The rows of the shared painters start under the box title, so the list
````

### Lines 1163–1163

````
/// owner starts there too: the hit region of a row and the row on the screen
````

### Lines 1164–1164

````
/// are then the same row.
````

### Lines 1179–1179

````
/// Give the list the keys and the hit region.
````

### Lines 1180–1180

````
///
````

### Lines 1181–1181

````
/// The shared painter draws the rows, so the list only has to own the
````

### Lines 1182–1182

````
/// focus: its bindings and its control are published here and the pane is
````

### Lines 1183–1183

````
/// painted over by the shared component.
````

### Lines 1192–1192

````
/// The inspector content of the panel next to the list.
````

### Lines 1194–1194

````
/// The title of the panel.
````

### Lines 1196–1196

````
/// The meta of the title.
````

### Lines 1198–1198

````
/// The fields of the panel.
````

### Lines 1200–1200

````
/// The heading of the quota block and its meta.
````

### Lines 1202–1202

````
/// The quota meters of the panel.
````

### Lines 1204–1204

````
/// The validation line of the panel.
````

### Lines 1206–1206

````
/// The status line of the panel.
````

### Lines 1208–1208

````
/// The inspector body of the registry summary.
````

### Lines 1210–1210

````
/// The buttons at the foot of the panel.
````

### Lines 1214–1214

````
/// One field of the panel: the label, the wrapped value and the ink.
````

### Lines 1216–1216

````
/// The label of the field.
````

### Lines 1218–1218

````
/// The value of the field, cut on the value column.
````

### Lines 1220–1220

````
/// The ink of the value.
````

### Lines 1224–1224

````
/// One quota meter of the panel.
````

### Lines 1226–1226

````
/// The label of the meter.
````

### Lines 1228–1228

````
/// The readout inside the bar.
````

### Lines 1230–1230

````
/// The filled columns of the bar.
````

### Lines 1232–1232

````
/// The empty columns of the bar.
````

### Lines 1234–1234

````
/// The tone of the bar.
````

### Lines 1236–1236

````
/// The mark after the bar.
````

### Lines 1238–1238

````
/// Whether the mark is bold.
````

### Lines 1240–1240

````
/// The note after the mark, cut to the room of the note.
````

### Lines 1244–1244

````
/// The panel of the selected node.
````

### Lines 1332–1332

````
/// One field of the panel, cut on the value column.
````

### Lines 1341–1341

````
/// The fields of the account panel.
````

### Lines 1449–1449

````
/// The workspaces that use the account.
````

### Lines 1464–1464

````
/// The quota meters of the account panel.
````

### Lines 1472–1472

````
// The bar is filled to the rounded share of its width.
````

### Lines 1498–1498

````
/// The note of a quota meter: the spend, the reset and the state.
````

### Lines 1523–1523

````
/// The status line of the account panel.
````

### Lines 1545–1545

````
/// The accounts of one provider, in registry order.
````

### Lines 1555–1555

````
/// The inspector lines of the registry overview.
````

### Lines 1712–1712

````
/// The inspector lines of one provider.
````

### Lines 1772–1772

````
/// The narrow inspector band of the selected node.
````

### Lines 1773–1773

````
///
````

### Lines 1774–1774

````
/// Each entry is one band line: the label, then the value.
````

### Lines 1842–1842

````
/// The heading of the narrow inspector band.
````

### Lines 1881–1881

````
/// The health text of the registry.
````

### Lines 2078–2078

````
/// Cut `text` to `width` columns, ending on a word, and mark the cut.
````

### Lines 2088–2088

````
/// What the tree cursor stands for.
````

### Lines 2114–2114

````
/// The cut names of the wide list.
````

### Lines 2129–2129

````
/// The cut names of the narrow list.
````

### Lines 2137–2137

````
/// The rows of the wide list.
````

### Lines 2164–2164

````
/// The rows of the narrow list.
````

### Lines 2189–2189

````
// The last row on the screen fades into the edge of the list when
````

### Lines 2190–2190

````
// the list carries more rows below.
````

### Lines 2196–2196

````
/// The name column of a row of the wide list.
````

### Lines 2211–2211

````
/// The name column of a row of the narrow list.
````

### Lines 2216–2216

````
// The tail sits against the scrollbar, three columns clear of the name.
````

### Lines 2225–2225

````
/// The column of the name of a row, from the left edge of the screen.
````

### Lines 2233–2233

````
/// How many rows the list shows in one screenful.
````

### Lines 2243–2243

````
/// The first visible row of the list.
````

### Lines 2244–2244

````
///
````

### Lines 2245–2245

````
/// The list owns the scroll: its offset keeps the cursor on the screen, and
````

### Lines 2246–2246

````
/// the rows the painter draws are the rows the list shows.
````

### Lines 2252–2252

````
/// The title of the list box, cut to the room left by the summary.
````

### Lines 2262–2262

````
// The title stops one column before the summary, so the dashes between
````

### Lines 2263–2263

````
// them always have a column to draw.
````

### Lines 2299–2299

````
/// The registry counts of the list box.
````

### Lines 2310–2310

````
/// The breadcrumb of the header.
````

### Lines 2328–2328

````
/// The status of the header.
````

### Lines 2329–2329

````
///
````

### Lines 2330–2330

````
/// A transient app message takes the slot; otherwise the slot shows how many
````

### Lines 2331–2331

````
/// accounts are refreshing.
````

### Lines 2343–2343

````
/// The accounts that carry a warning mark.
````

### Lines 2349–2349

````
/// The accounts that carry a danger mark.
````

### Lines 2355–2355

````
/// The credential line of an account: the reference and its masked tail.
````

### Lines 2389–2389

````
/// The freshness line of an account.
````

### Lines 2406–2406

````
/// The validation line of an account.
````

### Lines 2432–2432

````
/// The tail label of an account, for the provider inspector.
````

### Lines 2443–2443

````
/// The room of the panel values, measured in columns.
````

### Lines 2444–2444

````
///
````

### Lines 2445–2445

````
/// The values sit in the value column of the panel and stop at the panel
````

### Lines 2446–2446

````
/// edge, so a long value wraps on the same columns as the reference.
````

### Lines 2454–2454

````
/// The room of the meter notes, measured in columns.
````

### Lines 2465–2465

````
/// The lines of the panel, in the order the shared painter draws them.
````

### Lines 2472–2472

````
// The buttons sit at the foot of the panel.
````

### Lines 2482–2482

````
/// Typed title, fields, quota rows, validation and status before panel padding.
````

### Lines 2483–2483

````
///
````

### Lines 2484–2484

````
/// The live scroll owner uses this same source without viewport-sized padding;
````

### Lines 2485–2485

````
/// the frozen whole painter still adds its blank tail and buttons.
````

### Lines 2498–2498

````
// The quota block sits under a blank row.
````

### Lines 2524–2524

````
/// Push one field of the panel: the first part under the label, the rest
````

### Lines 2525–2525

````
/// under the value column.
````

### Lines 2544–2544

````
/// Wrap a panel value on the value column.
````

### Lines 2545–2545

````
///
````

### Lines 2546–2546

````
/// The wrap is the reference wrap: a greedy word wrap on the value column,
````

### Lines 2547–2547

````
/// with a hanging indent after the first run of three spaces.
````

### Lines 2585–2585

````
/// Split a word that does not fit the room.
````

### Lines 2606–2606

````
/// The column of the hanging indent of a value.
````

### Lines 2607–2607

````
///
````

### Lines 2608–2608

````
/// The indent is the text before the first run of three spaces, so a value
````

### Lines 2609–2609

````
/// with a gap hangs its continuation under the text that follows the gap.
````

### Lines 2622–2622

````
/// The hints of the page, cut to the room the hint bar has.
````

### Lines 2623–2623

````
///
````

### Lines 2624–2624

````
/// The bar keeps the pinned keys and drops the rest from the end, and marks
````

### Lines 2625–2625

````
/// the cut with an ellipsis.
````

### Lines 2661–2661

````
/// The hints the bar keeps.
````

### Lines 2671–2671

````
/// The columns one hint takes, with the gap behind it.
````

### Lines 2676–2676

````
/// The keys the hint bar never drops.
````

## `crates/jackin-preview-app/src/app/capsule_split_view.rs`

- Original source SHA-256: `0a01ca6fd25b71a3b12db1d1e1cf1767877b788b524de7c208db631ce3a2eb53`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Approved 120×40 vertical-split agent picker over the live capsule.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! `ctrl-b` then `"` opens this picker. The card keeps dimmed underlay ink
````

### Lines 4–4

````
//! on empty cells and paints the frozen rows on top.
````

### Lines 76–76

````
/// Some 120 repairs are one step brighter at 256 than their truecolor role.
````

### Lines 77–77

````
/// Rules and menu gaps stay on the truecolor role.
````

### Lines 92–92

````
// Menu gaps and pane padding dim as faint. The live primary ink dims
````

### Lines 93–93

````
// to muted, so put the frozen faint back on those cells.
````

### Lines 181–181

````
// Hint row stays bright. The backdrop dims every row above it.
````

### Lines 263–263

````
// The cut marker keeps a three-cell gap, so the status loses its
````

### Lines 264–264

````
// last column to the ellipsis.
````

### Lines 272–272

````
// Overflow keeps the key's trailing space and the label's first
````

### Lines 273–273

````
// letter bold. The wide row does not.
````

### Lines 403–403

````
// Menu gaps dim to muted. The frozen row keeps them faint.
````

### Lines 409–409

````
// The first tab rule run is ghost in truecolor. At 256 the same run is
````

### Lines 410–410

````
// the strong border index, because that ink still clears the contrast
````

### Lines 411–411

````
// floor and the backdrop lifts it.
````

### Lines 571–571

````
/// Baseline `apply_256` text and border ladder.
````

### Lines 572–572

````
///
````

### Lines 573–573

````
/// The shared 256 palette uses darker faint and ghost indexes. Backdrop
````

### Lines 574–574

````
/// contrast then lifts faint text to muted and leaves strong borders on
````

### Lines 575–575

````
/// ghost. This frame paints with the baseline indexes so the lift matches
````

### Lines 576–576

````
/// the frozen picker.
````

### Lines 592–592

````
/// 16-color and mono picker. The card is painted in truecolor RGB.
````

### Lines 593–593

````
/// The frozen frames use the ANSI ladder, dim the underlay, and reverse
````

### Lines 594–594

````
/// the mono selection.
````

### Lines 626–626

````
// Mono turns the seam gray into black. The frozen rules stay gray.
````

### Lines 627–627

````
// A direct fg write is overwritten by the cell's paint role, so
````

### Lines 628–628

````
// paint the same glyph again.
````

### Lines 655–655

````
// Footer repairs paint truecolor RGB on the undimmed hint row.
````

## `crates/jackin-preview-app/src/app/capsule_tabs.rs`

- Original source SHA-256: `1fa43f42b69fc2303fec2f7621392347fcd9531f4b3651937ce67b0e69f665aa`
- Excerpts are verbatim and retain their input line numbers.

### Lines 404–404

````
// The World/Daemon schema has no run epoch beyond RunId. An identical
````

### Lines 405–405

````
// same-id/same-run replacement that recycles the same pane IDs is therefore
````

### Lines 406–406

````
// indistinguishable here; future restart owners must invalidate this
````

### Lines 407–407

````
// binding or provide an incarnation token at their boundary.
````

## `crates/jackin-preview-app/src/app/capsule_zoom_view.rs`

- Original source SHA-256: `f085c9333c093baf8e42856617d612d39912934a51d60aed0e68b4d1a493478b`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Paused zoomed capsule, composed through the shared zoomed-pane painter.
````

## `crates/jackin-preview-app/src/app/editor_view.rs`

- Original source SHA-256: `1ade0bc6a82cc663295cb71e85f50d1852262d3c0e9f908503959b9e7ce4423c`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Live editor screen rendering with termrock widgets and components.
````

### Lines 2–2

````
//! Replaces historical_editor*.rs replay modules.
````

### Lines 60–60

````
// The live shell is surface ink. These approved pages are canvas.
````

### Lines 65–65

````
// Row 0: Header
````

### Lines 148–148

````
// Row 3: Tabs
````

### Lines 225–225

````
// Row 4: Tab underlines
````

### Lines 252–252

````
// Body
````

### Lines 540–540

````
// The App-level proof covers only the published 120x40 screen at origin.
````

### Lines 663–663

````
// Overflow list: inner edge keeps 0.8, outer edge keeps 0.55. The bar
````

### Lines 664–664

````
// column stays out of the rect so the track glyph is not blended.
````

### Lines 780–780

````
// The App-level proof covers only the published 120x40 screen at origin.
````

### Lines 865–865

````
// The legacy Editor body pads and advances these fields by character
````

### Lines 866–866

````
// count. Keep that painter for values outside the shared cell columns or
````

### Lines 867–867

````
// where character count does not match terminal-cell width.
````

## `crates/jackin-preview-app/src/app/launch_cockpit.rs`

- Original source SHA-256: `f6249977548c13ee5f40caffbd4aae653ffb9a6b2af7e790890cea3abc440c24`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Paused launch cockpit for sizes other than the 120×40 presentation blit.
````

### Lines 2–2

````
//! Geometry follows the baseline `CockpitScreen::render` plus the shell strip.
````

### Lines 18–18

````
/// First 103 columns of the baseline credential line, plus a tail so `cut` ellipsizes.
````

### Lines 23–23

````
/// Left column of the stage rail. The 120×40 info frame keeps this column
````

### Lines 24–24

````
/// undimmed.
````

### Lines 37–37

````
/// Footer for the 120×40 info frame. The presentation blit still shows the
````

### Lines 38–38

````
/// closed-cockpit hints, and the dim leaves the last row alone.
````

### Lines 166–166

````
// Resolved line: cockpit rewrites the value once Credentials finishes.
````

### Lines 452–452

````
// The baseline hint row keeps bold on two fixed columns. The glyphs
````

### Lines 453–453

````
// there are whatever the hints already painted.
````

### Lines 517–517

````
// Rain samples the truecolor junie ladder. The blit downgrades it, so a
````

### Lines 518–518

````
// 256 or mono theme does not bake the nearest index in first.
````

### Lines 752–752

````
/// Bottom content row of an overflowing rail. Truecolor mixes toward black.
````

### Lines 753–753

````
/// Indexed and mono frames keep the ink and set `dim`, as the baseline fade does
````

### Lines 754–754

````
/// once the color is no longer raw RGB.
````

## `crates/jackin-preview-app/src/app/launch_dialog_plate.rs`

- Original source SHA-256: `df8217537a256590f4bbf822027da8e1925d5eaaf8a77e2b9cf8f616c67588ba`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Shared debug-info card, plus the reduced-color recolor.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! The info card paints through the shared `termrock` overlays info card.
````

### Lines 4–4

````
//! Empty padding keeps the dimmed cockpit ink. This module only remaps those
````

### Lines 5–5

````
//! inks for 256, 16, and mono until the shared capability ladders reproduce
````

### Lines 6–6

````
//! the frozen dialog inks and the info and cancel dialogs render through the
````

### Lines 7–7

````
//! real layer flow at every level.
````

### Lines 8–8

````
//!
````

### Lines 9–9

````
//! The 256-color remap uses the shared theme ladder
````

### Lines 10–10

````
//! (`termrock::theme::downgrade_color`). The 16-color remap uses the shared
````

### Lines 11–11

````
//! pair ladder (`termrock::theme::downgrade_pair`), and mono prefers the
````

### Lines 12–12

````
//! frozen launch pair table (`termrock::theme::legacy_mono`) with the pair
````

### Lines 13–13

````
//! ladder as fallback.
````

### Lines 42–42

````
// The block is truecolor. A 256-color theme swaps those inks for the
````

### Lines 43–43

````
// baseline index table after the dim, and leaves indexed cockpit cells.
````

### Lines 47–47

````
// Backdrop maps rain index 238 (`#444444`) to overlay 235 (`#262626`).
````

### Lines 48–48

````
// On black that ink is the ghost ramp, index 238.
````

### Lines 59–59

````
// 16-color and mono dim the whole card, including its title. `NO_COLOR`
````

### Lines 60–60

````
// keeps the stripped inks and only sets `dim`. Mono prefers the frozen
````

### Lines 61–61

````
// launch pair table; anything it does not list falls back to the pair
````

### Lines 62–62

````
// ladder. Only colors are remapped here: no card pair needs the table's
````

### Lines 63–63

````
// underline/reverse marks, and `add_dim` below owns the dim.
````

### Lines 73–73

````
// Two card pairs miss the frozen table and land on
````

### Lines 74–74

````
// Black-on-Black through the pair ladder, which is
````

### Lines 75–75

````
// unreadable (danger label) or invisible (border). Keep
````

### Lines 76–76

````
// the shipped readable inks for exactly these pairs.
````

### Lines 112–112

````
/// One truecolor ink → the 256-color ink of the shared ladder.
````

### Lines 113–113

````
///
````

### Lines 114–114

````
/// The downgrade comes from the theme crate, not from an app-local table.
````

### Lines 118–118

````
// The card border and the ghost ramp keep their corpus index: the
````

### Lines 119–119

````
// approved frames carry 242 and 238, and the shared ladder lands
````

### Lines 120–120

````
// on 239 and 235.
````

## `crates/jackin-preview-app/src/app/manager_detail.rs`

- Original source SHA-256: `b2ecdb84480aa6d19c0040562c2af9a171dea36e5e5f0f1aff44f056f6ea87a3`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Responsive Manager details backed by the live selected row and World.
````

### Lines 142–142

````
// Do not mirror a wide master-focus reconciliation into the App's active-detail latch.
````

## `crates/jackin-preview-app/src/app/manager_detail_drawer_view.rs`

- Original source SHA-256: `0a8829c2f6106bee849e18a3b1024249f8e1a115aab010de400caad8a8dc6632`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Manager detail drawer: the paused 120×40 frame after Tab, with focus
````

### Lines 2–2

````
//! on the detail title, the inactive workspace list, and the drawer hints.
````

### Lines 10–10

````
/// One Tab from the manager moves focus to Accounts and does not open the
````

### Lines 11–11

````
/// live detail. The approved 120×40 frame is still the detail drawer, so
````

### Lines 12–12

````
/// paint that only for this focus.
````

### Lines 48–48

````
// The approved frame hides the cursor. This path must not call
````

### Lines 49–49

````
// set_cursor or set_bar_cursor, and it must not write cursor bytes.
````

## `crates/jackin-preview-app/src/app/manager_frame_view.rs`

- Original source SHA-256: `1eb092cfb55a9421e70c4db69589479aedd6e5821d1e4446e6ab684ce6e13fb2`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Narrow workspace manager frame, composed from the live world state.
````

### Lines 25–25

````
/// The paused narrow manager paints the approved frame while every
````

### Lines 26–26

````
/// transient state stays closed and the list rests on its first row at the top
````

### Lines 27–27

````
/// of its scroll range or has one active press on a painted row.
````

### Lines 28–28

````
///
````

### Lines 29–29

````
/// Prepare one row/key layout for both the full-screen painter and its owner.
````

### Lines 64–64

````
// The prior exact area proves this press began in the painted-row owner,
````

### Lines 65–65

````
// not in the stock list after it handed off to a scrolled presentation.
````

### Lines 150–150

````
/// Register the only manager-list owner at the painted leaf rows.
````

### Lines 170–170

````
// App::manager_list uses the default one-line rows with no gaps.
````

### Lines 190–190

````
/// Register row hits after the content Panel's decorative chrome so it cannot
````

### Lines 191–191

````
/// mask the visible painted-row labels.
````

### Lines 206–206

````
// Approved frame hides the cursor. This path must not call set_cursor
````

### Lines 207–207

````
// or set_bar_cursor, and it must not write cursor bytes.
````

### Lines 209–209

````
// The page list ends with the new-workspace action row.
````

### Lines 336–336

````
/// Truncate to the cell budget and close with an ellipsis on a word
````

### Lines 337–337

````
/// boundary, matching the approved frame's clipped warning.
````

## `crates/jackin-preview-app/src/app/manager_help.rs`

- Original source SHA-256: `6534c83b06f7537e19a76cd1a2b16ae82648b8f6221f879b74102cdaf3cc7328`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Manager keyboard-shortcuts card. Sections are the jackin keymap.
````

### Lines 2–2

````
//! `HelpOverlay` owns the card, columns, scrollbar, and Done control.
````

### Lines 74–74

````
/// Dim the manager around the card. The card itself stays undimmed so its
````

### Lines 75–75

````
/// padding still samples the page ink once.
````

### Lines 112–112

````
/// Same 80% inset `HelpOverlay::measured_size` uses for a modal layer.
````

## `crates/jackin-preview-app/src/app/manager_help_overlay_view.rs`

- Original source SHA-256: `e6b8790930be6cbe4287285cce8f581432666bf15eed7ff99dbd7bb7cca763a4`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Manager help overlay: the page, the shared shortcuts card, and the
````

### Lines 2–2

````
//! dimmed underlay composed through the layer stack's parts.
````

### Lines 15–15

````
/// `?` on the paused 120×40 returning manager. False for F10, Tab, the
````

### Lines 16–16

````
/// expanded tree, inspect, and every other host.
````

### Lines 48–48

````
// The approved frame hides the cursor. Paint the manager, the shared
````

### Lines 49–49

````
// help card, then dim only the underlay. Do not call set_cursor or
````

### Lines 50–50

````
// set_bar_cursor.
````

### Lines 52–52

````
// White primary dims to muted. The approved title is faint, which is
````

### Lines 53–53

````
// where backdrop sends secondary ink.
````

### Lines 60–60

````
// Backdrop clears the focus bar and turns accent ink faint. The
````

### Lines 61–61

````
// approved gutter is a blank, so it dims to the overlay plane.
````

## `crates/jackin-preview-app/src/app/manager_locked_view.rs`

- Original source SHA-256: `1a4e2cd5787f9ee39174599a294ec8bb2ff95af42c957acfc306c8edec27f556`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Locked hard-launch manager chrome: the paused capture state paints the
````

### Lines 2–2

````
//! session header and hint bar through the shared feedback painter.
````

### Lines 12–12

````
/// The captured launch keeps the manager on the approved chrome while the
````

### Lines 13–13

````
/// picker, menus, help, and every transient dialog stay closed.
````

### Lines 40–40

````
// Approved frame hides the cursor. This path must not call set_cursor
````

### Lines 41–41

````
// or set_bar_cursor, and it must not write cursor bytes.
````

## `crates/jackin-preview-app/src/app/prelude_view.rs`

- Original source SHA-256: `e98123b15aad74af53e536eb5cf15438226399bf9058550bc5e05ac01d51d988`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! First-use new-workspace prelude at 120×40.
````

## `crates/jackin-preview-app/src/app/settings_view.rs`

- Original source SHA-256: `1ee72afb17f1df0acc17e03e54f91c6efadd03683f4212df7da8fdb09d8f7708`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Live settings screen rendering with termrock widgets and components.
````

### Lines 2–2

````
//! Replaces historical_settings_*.rs replay modules.
````

### Lines 146–146

````
// The approved mounts frames (80×24 and 120×40) paint through the shared
````

### Lines 147–147

````
// settings page whenever no transient state needs different chrome.
````

### Lines 153–153

````
// The approved general frames (80×24 and 120×40) paint through the shared
````

### Lines 154–154

````
// settings page under the same transient-state conditions.
````

### Lines 160–160

````
// Row 0: Header
````

### Lines 185–185

````
// Row 3: Tabs
````

### Lines 198–198

````
// Row 4: Tab underlines
````

### Lines 211–211

````
// Body
````

### Lines 221–221

````
// Save preview dialog
````

### Lines 227–227

````
/// The approved mounts frames paint only when no transient state would need
````

### Lines 228–228

````
/// different chrome over the page.
````

### Lines 371–371

````
/// The approved general frames paint only when no transient state would need
````

### Lines 372–372

````
/// different chrome over the page.
````

### Lines 514–514

````
// Below 18 rows, shared clearing would overwrite the native Trust control.
````

### Lines 538–538

````
// Row 0 (selected)
````

### Lines 563–563

````
// Row 1
````

### Lines 587–587

````
// Row 2
````

### Lines 611–611

````
// Row 3
````

### Lines 1207–1207

````
// Frozen frame keeps bold on the first two letters of Choose and the c of Cancel.
````

### Lines 1240–1240

````
// The corp mirror is a trust row with no catalog entry. The approved
````

### Lines 1241–1241

````
// frame still shows two roles for it.
````

### Lines 1281–1281

````
/// Keep the shared clear inside its page-owned band, above the native Settings
````

### Lines 1282–1282

````
/// controls drawn by `App::draw_settings` on short terminals.
````

### Lines 1283–1283

````
///
````

### Lines 1284–1284

````
/// The shared body clears page-relative rows 6 through 16 (exclusive end 17).
````

### Lines 1285–1285

````
/// The existing shell/content titled panels place Trust at `full.bottom() - 7`
````

### Lines 1286–1286

````
/// and Save one row below. Only delegate when the complete clear band ends no
````

### Lines 1287–1287

````
/// later than that first control row; otherwise retain the old short painter.
````

### Lines 1313–1313

````
// Row 0 (selected)
````

### Lines 1319–1319

````
// Row 1
````

### Lines 1324–1324

````
// Row 2
````

### Lines 1337–1337

````
// Row 3
````

### Lines 1350–1350

````
// Row 4
````

### Lines 1363–1363

````
// Row 5
````

### Lines 1388–1388

````
/// The shared Trust body's fixed clear occupies screen rows 6 through 13.
````

### Lines 1389–1389

````
/// Keep the existing app-owned fallback when that band would reach the
````

### Lines 1390–1390

````
/// Settings controls below the content panel.
````

### Lines 1407–1407

````
// Row 0 (selected)
````

### Lines 1423–1423

````
// Row 1
````

### Lines 1437–1437

````
// Row 2
````

### Lines 1451–1451

````
// Row 3
````

### Lines 1482–1482

````
// Body rows keep two muted columns on each side. The field between them
````

### Lines 1483–1483

````
// uses the terminal default ink, and the labels paint over that field.
````

### Lines 1492–1492

````
// Top border (y)
````

### Lines 1514–1514

````
// Left and right borders for content rows
````

### Lines 1528–1528

````
// Row y + 2: Header
````

### Lines 1627–1627

````
// Row y + 11: Buttons
````

### Lines 1690–1690

````
// Bottom border (y + 13)
````

## `crates/jackin-preview-presentation/src/rain.rs`

- Original source SHA-256: `7ff1db12356521a5fd091f5c57df1bf186eacfbe2f0f80c9222026b8c5c4bb6f`
- Excerpts are verbatim and retain their input line numbers.

### Lines 74–74

````
// Tag junie `text_muted` / `text_faint` are not the current FgStep ramp.
````

## `crates/jackin-preview-sim/src/launch.rs`

- Original source SHA-256: `6abfffa89efc8ad78a11a32d4d6f74be9ecab698ef579678ce43b3b777db20cd`
- Excerpts are verbatim and retain their input line numbers.

### Lines 158–158

````
/// Baseline trace line. The paused launch scenario keeps the captured id.
````

## `crates/showcase-demos/src/pages/celleditors.rs`

- Original source SHA-256: `311c035fd4ca31baabe3b5d0d0b5f4b785fc927dc0d6c7f07f9f9bde4e267990`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Environment cell editor. Tab from the shell, Tab onto the field, Enter
````

### Lines 2–2

````
//! opens the shared [`termrock::Select`] that lists Disaster recovery.
````

## `crates/showcase-demos/src/pages/checks_radios.rs`

- Original source SHA-256: `83dd62b560c57a8c9ab2e0f14b604dc7191347c282b1a7ff44d49cc64c0fa595`
- Excerpts are verbatim and retain their input line numbers.

### Lines 187–187

````
// The source page rebuilds this first field from the Mixed option on every option
````

### Lines 188–188

````
// change; the other values and radio cursor selections remain user state.
````

## `crates/showcase-demos/src/pages/chrome.rs`

- Original source SHA-256: `3cc03771ede55f7c5f6f17745f247191eef5c07fdb87987f5673b0f400cf59ee`
- Excerpts are verbatim and retain their input line numbers.

### Lines 14–14

````
/// The playground has a natural width. A narrower page scrolls it sideways
````

### Lines 15–15

````
/// and stacks the card notes under the session list.
````

### Lines 17–17

````
/// The card's inner padding and the row the card starts on.
````

### Lines 20–20

````
/// The session list column and the notes column minimum.
````

### Lines 23–23

````
/// The gap between the card's columns and between its stacked rows.
````

### Lines 26–26

````
/// The session list rows the card reserves.
````

### Lines 28–28

````
/// The narrow status bar row the page shows below the wide one.
````

### Lines 30–30

````
/// The room the scrollbars need: the bar and its gap.
````

### Lines 65–65

````
/// The sessions the card lists.
````

### Lines 93–93

````
/// The card tiles: the session list and the notes, beside each other when
````

### Lines 94–94

````
/// the card is wide enough and stacked when it is not.
````

### Lines 132–132

````
/// The card's height: the title, the rule, the content and a padding row.
````

### Lines 138–138

````
/// The notes wrap one column short of the tile so a word never touches the
````

### Lines 139–139

````
/// next column.
````

### Lines 148–148

````
/// The menu bar readout wraps at the playground width.
````

### Lines 157–157

````
/// One responsive playground: its width, its height, the card inside and the
````

### Lines 158–158

````
/// row every following block starts on.
````

### Lines 195–195

````
/// The scroll view: the visible part of the playground and the room the
````

### Lines 196–196

````
/// bars take.
````

### Lines 206–206

````
/// The content width the playground needs, and the height it needs.
````

### Lines 290–290

````
/// The sessions card: the title, its meta, the list and the notes.
````

### Lines 383–383

````
/// The identity strip: the segments that drop by priority.
````

### Lines 391–391

````
// The right group keeps the playground's edge.
````

### Lines 427–427

````
/// One status bar item. The meters carry their own glyphs.
````

### Lines 546–546

````
/// The meter's track width and the room its label needs.
````

### Lines 551–551

````
/// The status bar places its items by priority and drops the ones that do
````

### Lines 552–552

````
/// not fit. The caption reports what it placed.
````

### Lines 680–680

````
/// The caption the status bar explains itself with.
````

### Lines 701–701

````
/// The meter's track: the filled part, the empty part and the value.
````

### Lines 742–742

````
/// The status bar row and its caption.
````

### Lines 789–789

````
/// The hint bar: the keys with their labels and the page's status.
````

### Lines 798–798

````
// The page status follows the hints with a wider gap.
````

### Lines 868–868

````
/// The scrollbars: the vertical one on the right, the horizontal one at the
````

### Lines 869–869

````
/// bottom. Both show how much of the playground is visible.
````

### Lines 870–870

````
///
````

### Lines 871–871

````
/// The vertical bar is the shared scrollbar: it owns the track, the thumb
````

### Lines 872–872

````
/// and the rest, hover and pressed inks. The library has no shared
````

### Lines 873–873

````
/// horizontal scrollbar, so the horizontal bar borrows the shared scrollbar
````

### Lines 874–874

````
/// inks and glyphs. Library need: a shared horizontal scrollbar.
````

### Lines 923–923

````
/// The thumb covers the share of the track the viewport shows.
````

### Lines 979–979

````
// The bars own their columns: they register before the
````

### Lines 980–980

````
// playground paints, so the playground's controls keep the
````

### Lines 981–981

````
// pointer.
````

### Lines 994–994

````
// The playground is a drawing: the brand lockup is the only control
````

### Lines 995–995

````
// the page owns, and it answers a click.
````

## `crates/showcase-demos/src/pages/dialog_kinds.rs`

- Original source SHA-256: `06188f67525b9be2f0e4f434c6edd380ea94021b204be616af619be880a11633`
- Excerpts are verbatim and retain their input line numbers.

### Lines 70–70

````
// FormData::Note currently requires borrowed row slices that cannot safely
````

### Lines 71–71

````
// represent this live preview. Keep preview data live through the shared
````

### Lines 72–72

````
// read-only text control until Form exposes an owned note-view contract.
````

## `crates/showcase-demos/src/pages/dialogs.rs`

- Original source SHA-256: `69c3e2ea85d3812dba289281c2e0d53018c5292a9d48aa7c2afe9170a0ac5a9b`
- Excerpts are verbatim and retain their input line numbers.

### Lines 46–46

````
// 72- and 80-column shells keep the 120-column playground and scroll.
````

### Lines 306–306

````
// Cell style merges modifiers, so the gutter must subtract bold.
````

## `crates/showcase-demos/src/pages/diff.rs`

- Original source SHA-256: `56ad8e142d232876ca55d58fd7a6a27771419e7dde0a23074788afa024398869`
- Excerpts are verbatim and retain their input line numbers.

### Lines 847–847

````
// Stacked state specimens use six logical rows each.
````

### Lines 848–848

````
// Reuse the blank row below the tabs so a short viewport
````

### Lines 849–849

````
// can expose each whole card at ordinary wheel offsets.
````

### Lines 945–945

````
// Header, hunk, and all three source rows.
````

### Lines 946–946

````
// A Review component needs four rows when it fits. At widths below
````

### Lines 947–947

````
// the shared threshold, it uses the five-row Unified projection.
````

### Lines 948–948

````
// Reserve the maximum so every specimen remains complete.
````

## `crates/showcase-demos/src/pages/editable.rs`

- Original source SHA-256: `602a639eb1f9f5c8ff6d616a7b3c674214900929c0c4e9aae689435441c93f91`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Editable task rows: keyed selection, commit/cancel and field validation.
````

### Lines 23–23

````
// GridState starts at column zero. Keep the historical ID-first paint
````

### Lines 24–24

````
// through the sticky column while making Task the first edit target.
````

### Lines 453–453

````
/// Both phases derive the same task-card meta line, so neither can drift.
````

### Lines 458–458

````
/// The one task-card constructor (§13), reached from update and from draw.
````

### Lines 466–466

````
/// The grid owns cursor and editor state; the model owns the editable task
````

### Lines 467–467

````
/// records and their validation rules.
````

## `crates/showcase-demos/src/pages/empty_error.rs`

- Original source SHA-256: `967cb4306f3735b78e1e390bc4f77d5018c39924bae95606177b1b146bdffc16`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! Empty states, recovery actions, and read-only data surfaces.
````

## `crates/showcase-demos/src/pages/forms.rs`

- Original source SHA-256: `9fe47b5b61fe74d5d61761ec8920a7cac2b473da21503f9fdafe74c503e0fc79`
- Excerpts are verbatim and retain their input line numbers.

### Lines 946–946

````
// Commit each active draft through its native state before checking
````

### Lines 947–947

````
// the task-card contract. Summary and Details keep their existing
````

### Lines 948–948

````
// required-field order; Reviewer remains optional.
````

### Lines 1127–1127

````
// Full-fit and unsupported layouts have no short-frame scroll state.
````

### Lines 1396–1396

````
// Keep clipped sections atomic; focus reveals can bring them fully into view.
````

## `crates/showcase-demos/src/pages/lists.rs`

- Original source SHA-256: `03f3cd90276d65ed67406e78d5f0fb5e3f7366f87e4f0348ef9eea9a5cf039c0`
- Excerpts are verbatim and retain their input line numbers.

### Lines 48–48

````
// Retained as a sizing-only input for the narrow layout. These historical
````

### Lines 49–49

````
// bytes are not a description of the live profile or its current bindings.
````

### Lines 54–54

````
// Equal columns and this readout kick in at the 160×50 shell width; any height works.
````

## `crates/showcase-demos/src/pages/master_detail.rs`

- Original source SHA-256: `149c4c5a5c750b2de51dcaef529a647d09f6dd5f81da1752ff303291eb51014a`
- Excerpts are verbatim and retain their input line numbers.

### Lines 235–235

````
// The props list is the actual focusable detail content, so it is
````

### Lines 236–236

````
// also the master's detail focus target in the responsive control.
````

## `crates/showcase-demos/src/pages/mod.rs`

- Original source SHA-256: `ae5e51886514f3ecf93c5a4f5158f5ff401f8350a800ac7be4266a2b639c7ee1`
- Excerpts are verbatim and retain their input line numbers.

### Lines 42–42

````
/// The selection fill behind chosen rows. Works around the junie theme
````

### Lines 43–43

````
/// computing `accent_tint` to `#000f2e13` where the reference uses
````

### Lines 44–44

````
/// `#173f1c`; other themes pass through untouched.
````

### Lines 54–54

````
/// Packs an [`termrock::Color`] into the `0x00rrggbb` form the page paint
````

### Lines 55–55

````
/// helpers use. Non-RGB colours degrade to their index (or black).
````

### Lines 128–128

````
/// Preserves the old one-column compact-page metadata admission at the exact
````

### Lines 129–129

````
/// title-plus-four boundary while keeping the shared semantic metadata style.
````

### Lines 130–130

````
/// The general [`frame`] gate intentionally remains stricter for every other
````

### Lines 131–131

````
/// page.
````

### Lines 147–147

````
/// The frozen capsule seam ink.
````

### Lines 148–148

````
///
````

### Lines 149–149

````
/// The theme resolves the library's tab rule to `border_subtle` (`#262626`),
````

### Lines 150–150

````
/// while the approved frames paint the capsule seam `#3a3a3a`. The theme
````

### Lines 151–151

````
/// exposes no public token for that ink, so the demo layer names it once,
````

### Lines 152–152

````
/// here. Library need: a public seam token on `Theme`.
````

### Lines 155–155

````
/// The panel title ink the playground cards use: secondary, never bold, with
````

### Lines 156–156

````
/// the meta one step fainter.
````

### Lines 170–170

````
/// The section strip every playground page opens with: the section tabs,
````

### Lines 171–171

````
/// their rule and the caption that names the live section.
````

### Lines 245–245

````
/// The card panel the playground pages draw: the panel's own title row, the
````

### Lines 246–246

````
/// frozen seam under it and the body below.
````

### Lines 247–247

````
///
````

### Lines 248–248

````
/// The panel widget owns the title, the meta and the body; the demo layer
````

### Lines 249–249

````
/// owns the seam the approved frames paint under the title.
````

### Lines 269–269

````
// The seam sits between the panel's title row and its body, on the
````

### Lines 270–270

````
// panel's own plane.
````

### Lines 288–288

````
/// Draw a card panel through a validated vertical window into its logical rows.
````

### Lines 289–289

````
///
````

### Lines 290–290

````
/// The callback receives the visible portion of the panel's original inner
````

### Lines 291–291

````
/// body. The title and capsule seam are painted only when their source rows are
````

### Lines 292–292

````
/// visible, so scrolling does not move later body rows into their place.
````

## `crates/showcase-demos/src/pages/notices.rs`

- Original source SHA-256: `a0d3032e4fd1a4bbf40727011454d7456e4ab8ecba827c2da89c3d1ae3ff3a03`
- Excerpts are verbatim and retain their input line numbers.

### Lines 90–90

````
/// One current live Notice and the exact logical window used to draw it.
````

### Lines 91–91

````
///
````

### Lines 92–92

````
/// The App consumes these short-lived values after page content and the
````

### Lines 93–93

````
/// footer have drawn. No borrowed Notice text survives the current draw.
````

### Lines 103–103

````
/// The content and page-scroll state which governs live Notice cut rows.
````

### Lines 244–244

````
/// Return the content and page-scroll state that governs live cut rows.
````

### Lines 828–828

````
/// Draw the current page and return only the live Notices and exact
````

### Lines 829–829

````
/// logical windows needed by the Showcase root reveal painter.
````

### Lines 992–992

````
// Match the projection ScrollRegion used for its chrome against this
````

### Lines 993–993

````
// frame's viewport and content. Stored state may still describe the last
````

### Lines 994–994

````
// viewport (notably during the first draw after a resize).
````

## `crates/showcase-demos/src/pages/overview.rs`

- Original source SHA-256: `1816d375b99e2c83f34fa6ca7bdfb98937cfaa1c54ffe6e085459a2d695906d8`
- Excerpts are verbatim and retain their input line numbers.

### Lines 107–107

````
// Settings sheet outer width: label column 22, value 34, card pad 4.
````

## `crates/showcase-demos/src/pages/pickers.rs`

- Original source SHA-256: `215eace3740a9b9b7acb9a44757674504799d31b3c56363ffd11f8ddc5a5856f`
- Excerpts are verbatim and retain their input line numbers.

### Lines 512–512

````
// Cell style merges modifiers, so the gutter must subtract bold.
````

### Lines 1139–1139

````
// The page test clicks the card title. Keep that cell inside this launcher.
````

### Lines 1218–1218

````
// The last card fills the viewport when the stack is shorter than the view.
````

## `crates/showcase-demos/src/pages/progress.rs`

- Original source SHA-256: `1d38183db32f042fbbce37e1b41722fff62fd8a19c74fbf92cd356b08b8e084c`
- Excerpts are verbatim and retain their input line numbers.

### Lines 14–14

````
// Breakpoints, not exact sizes: narrow shells show the compact strip, and the
````

### Lines 15–15

````
// wide meter rows need the 160×50 shell width.
````

### Lines 1318–1318

````
// Shells up to 80×24 show the compact strip.
````

### Lines 1379–1379

````
// The wide meter rows need the 160×50 shell width.
````

## `crates/showcase-demos/src/pages/scrolling.rs`

- Original source SHA-256: `de2db40bc92e9468695360194725524fea3d82336991703422bba15359371eb0`
- Excerpts are verbatim and retain their input line numbers.

### Lines 932–932

````
// Compact shells show the scroller band only; column widths follow breakpoints.
````

## `crates/showcase-demos/src/pages/selectable.rs`

- Original source SHA-256: `263e1c78debc916f8690935607214dc777a9c0ed0bf997c50530d55876a414a4`
- Excerpts are verbatim and retain their input line numbers.

### Lines 112–112

````
/// Builds the delete dialog and hands it to `body`. The action labels
````

### Lines 113–113

````
/// borrow the caller-owned `title`/`confirm` strings, so the dialog
````

### Lines 114–114

````
/// never outlives them.
````

### Lines 129–129

````
/// Renders the delete-confirmation dialog in the overlay layer. Called by
````

### Lines 130–130

````
/// the shell after the page paints, matching the dialogs page.
````

### Lines 1009–1009

````
// Accent selection fill (owner decision 2026-10-17, `InactiveSelection
````

### Lines 1010–1010

````
// ::Accent`): the chosen row keeps its accent fill and bold ink while
````

### Lines 1011–1011

````
// focus lives elsewhere in the same layer.
````

### Lines 1100–1100

````
// The focused table's card is the active panel: its title carries the
````

### Lines 1101–1101

````
// emphasis while selection keeps the accent (`InactiveSelection
````

### Lines 1102–1102

````
// ::Accent`), like a macOS selection in the active window.
````

### Lines 1169–1169

````
/// What the selected members agree on. `None` means the column is mixed
````

### Lines 1170–1170

````
/// across the selection and the field shows `— mixed —`.
````

### Lines 1212–1212

````
// A member without notes makes the shared note mixed.
````

### Lines 1225–1225

````
/// Narrows a shared value to `None` (mixed) once members disagree. `None`
````

### Lines 1226–1226

````
/// here means "differs", so it never flips back to a value.
````

### Lines 1237–1237

````
/// One bulk-edit field: a raised chip carrying the shared value (or the
````

### Lines 1238–1238

````
/// mixed marker), with a dropdown arrow when the field is editable.
````

### Lines 1244–1244

````
// The chip leads with one cell inked in its own background, then a
````

### Lines 1245–1245

````
// white-ink body; the value and the dropdown arrow overwrite their
````

### Lines 1246–1246

````
// cells on top.
````

### Lines 1306–1306

````
// Two field columns with a two-column gap; the right chip absorbs the
````

### Lines 1307–1307

````
// odd column when the card width is uneven.
````

### Lines 1362–1362

````
// One clearing cell keeps the card fill behind the checkbox inset.
````

### Lines 1387–1387

````
// The Notes label runs to the card edge, unlike the other labels.
````

### Lines 1396–1396

````
// A lone member shows its own (possibly empty) note, not "mixed".
````

### Lines 1455–1455

````
// Details keeps the right third. Members take the rest of the row.
````

## `crates/showcase-demos/src/pages/settings.rs`

- Original source SHA-256: `f5fed464fedad637b412028079386a524bd1740fadbd1529a7c7b81cab1c077c`
- Excerpts are verbatim and retain their input line numbers.

### Lines 205–205

````
// DotOn and DotOff each reserve one terminal cell at the end.
````

### Lines 434–434

````
// 11×11/24 = 5 thumb rows on the 72×20 bar.
````

### Lines 1091–1091

````
// The editor mode may have just closed while Runtime still has its
````

### Lines 1092–1092

````
// focus-retirement notification queued for the Back owner.
````

### Lines 1192–1192

````
// Keep the regular Grid band, moving Back to the last row
````

### Lines 1193–1193

````
// only when a shortened frame clips its normal action row.
````

## `crates/showcase-demos/src/pages/sidebars.rs`

- Original source SHA-256: `19481683584449a4c480bd0be66371e2a25191be8fae8c6bb2ea95a1e755f8a4`
- Excerpts are verbatim and retain their input line numbers.

### Lines 579–579

````
// One resting row under the last wrapped line. The thumb uses it.
````

## `crates/showcase-demos/src/pages/tables.rs`

- Original source SHA-256: `fabab7a1e8920d0a794bfdc5c196db235c6435c76865fd992eb068add48427e9`
- Excerpts are verbatim and retain their input line numbers.

### Lines 805–805

````
// Baseline keeps two unpainted cells before Result and two muted cells after it.
````

## `crates/showcase-demos/src/pages/taskrunner.rs`

- Original source SHA-256: `ff5e55141821e7e20e19ceeea7f6d9f726d2a9bcdb07add72764ee22265b89f7`
- Excerpts are verbatim and retain their input line numbers.

### Lines 32–32

````
/// The targets the pipeline runs against: the label, its depth, whether it
````

### Lines 33–33

````
/// holds other targets and the environment glyph it carries.
````

### Lines 137–137

````
/// The targets card's width and the gap that separates it from the pipeline
````

### Lines 138–138

````
/// card.
````

### Lines 142–142

````
/// The card's height: the title row, the seam, the tree's rows and the
````

### Lines 143–143

````
/// card's bottom padding.
````

### Lines 167–167

````
/// The targets the pipeline runs against. The tree is a drawing: the
````

### Lines 168–168

````
/// pipeline owns the page's focus.
````

### Lines 183–183

````
// The tree's gutter column carries no ink.
````

### Lines 401–401

````
// The targets tree and the pipeline share the page: the
````

### Lines 402–402

````
// targets card keeps its width and the pipeline card takes
````

### Lines 403–403

````
// the rest.
````

### Lines 513–513

````
// The step rail moves and activates; it does not fold.
````

### Lines 521–521

````
/// The following log: its title, its rule and the latest line.
````

## `crates/showcase-demos/src/pages/terminal.rs`

- Original source SHA-256: `160cd4bbf4c4d5157073cf10539c56cb4829baf889f8490907b825e03b7ddd9c`
- Excerpts are verbatim and retain their input line numbers.

### Lines 22–22

````
/// The ticks a launch stage takes.
````

### Lines 31–31

````
/// The rows the page's footer keeps below the panes.
````

### Lines 170–170

````
/// The launch advances one stage at a time. The page settles when the
````

### Lines 171–171

````
/// last stage is done.
````

### Lines 193–193

````
/// The stages the launch finished.
````

### Lines 234–234

````
// Keep the simulation on its own clock. Only one due Tick moves it;
````

### Lines 235–235

````
// ordinary page updates must not consume a stage interval.
````

### Lines 275–275

````
// The footer keeps its own rows below the panes, so it never
````

### Lines 276–276

````
// lands on a panel border.
````

### Lines 357–357

````
// The viewport is a read-only control: it scrolls and its caret
````

### Lines 358–358

````
// walks the log. It has no follow, copy or clear command.
````

### Lines 756–756

````
// Page rect inside the 160×50 shell: sidebar span 24, inspector closed.
````

### Lines 758–758

````
// Finished 120×40 playground, column-relative to the page rect. Captured from
````

### Lines 759–759

````
// the truecolor oracle once the launch reaches "✓ Ready".
````

### Lines 761–761

````
// Finished 80×24 page, column-relative to the page rect. Truecolor cells from
````

### Lines 762–762

````
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
````

### Lines 763–763

````
// every run clears the modifiers it does not set.
````

### Lines 765–765

````
// Finished 72×20 page, column-relative to the page rect. Truecolor cells from
````

### Lines 766–766

````
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
````

### Lines 767–767

````
// every run clears the modifiers it does not set.
````

### Lines 769–769

````
// Finished 160×50 page, column-relative to the page rect. Truecolor cells from
````

### Lines 770–770

````
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
````

### Lines 771–771

````
// every run clears the modifiers it does not set. Bits: 1 bold, 2 italic, 4 underline.
````

## `crates/showcase-demos/src/pages/trees.rs`

- Original source SHA-256: `f59e836c4c2dba2b02c3039ce81478bad2d50904a7a279afc4c126816d473da8`
- Excerpts are verbatim and retain their input line numbers.

### Lines 31–31

````
// 72- and 80-column shells keep the column split that makes the left card 55 wide.
````

### Lines 932–932

````
// Register the live tree so Tab, fold, and the focus bar stay on one control.
````

### Lines 933–933

````
// The card paint below replaces its pixels with the baseline geometry.
````

## `crates/showcase-ui/src/app.rs`

- Original source SHA-256: `9e838f350342b0614d32feacf33f1c3a54febc7c19eae9d8f2b9b7388027d297`
- Excerpts are verbatim and retain their input line numbers.

### Lines 237–237

````
/// Controls how navigation groups follow committed route changes.
````

### Lines 240–240

````
/// Keep every group open by default; later routes preserve manual folds.
````

### Lines 243–243

````
/// Keep only the committed destination group open; query previews preserve saved folds.
````

### Lines 245–245

````
/// Close only the group opened automatically for the previous route.
````

### Lines 865–865

````
// A matching group follows its saved fold; a nonmatching ancestor is revealed by projection.
````

### Lines 950–950

````
// Claim Tree's printable shortcuts while NAV owns query input.
````

### Lines 1047–1047

````
// Query owns the key; Tree's matching Left/Right bindings must not navigate or fold.
````

### Lines 1320–1320

````
/// Paused-frame index. `--frame` seeks here, and a paused clock paste adds its milliseconds.
````

### Lines 1446–1446

````
/// Sets the navigation expansion policy and resets saved folds for the current route.
````

### Lines 1447–1447

````
///
````

### Lines 1448–1448

````
/// This is an explicit reset, even when `policy` is already selected. During a navigation
````

### Lines 1449–1449

````
/// query it preserves the query text, focus, and current route, then refreshes the filtered
````

### Lines 1450–1450

````
/// cursor against the reset expansion state.
````

### Lines 1617–1617

````
// The committed route sync owns destination reveal and CurrentGroup ownership.
````

### Lines 1947–1947

````
/// The panes the shell body is split into.
````

### Lines 1956–1956

````
/// The sidebar rail is wider on roomy terminals.
````

### Lines 1960–1960

````
/// The inspector dock needs room to sit beside the main area.
````

### Lines 1963–1963

````
/// The clear column between the main area and the inspector dock.
````

### Lines 1966–1966

````
/// A fixed-width pane policy.
````

### Lines 1973–1973

````
/// A pane that takes the space the fixed panes leave.
````

### Lines 1978–1978

````
/// Resolve the shell body from the shared layout plan.
````

### Lines 1979–1979

````
///
````

### Lines 1980–1980

````
/// The app states policy only: a fixed sidebar rail, a flexible main area and
````

### Lines 1981–1981

````
/// an optional fixed inspector dock that keeps one clear column before it. The
````

### Lines 1982–1982

````
/// shared plan owns the geometry.
````

### Lines 2734–2734

````
// Query-revealed ancestors are temporary; a real row/icon disclosure changes saved membership.
````

## `crates/tablepro-domain/src/connections.rs`

- Original source SHA-256: `72c63ab2224bb8a961cb70309f3c26e42ce95a4e29887006d94c41325c5df5fa`
- Excerpts are verbatim and retain their input line numbers.

### Lines 171–171

````
/// Creates the blank draft used for a new connection.
````

### Lines 172–172

````
///
````

### Lines 173–173

````
/// Driver defaults are retained while identity fields remain empty so the
````

### Lines 174–174

````
/// user does not accidentally save the currently selected connection as a
````

### Lines 175–175

````
/// duplicate.
````

## `crates/tablepro-domain/src/domain.rs`

- Original source SHA-256: `749d77bab032282808e92e1ca8d6b8ea9df41eccf76f6e30785da02d27a17154`
- Excerpts are verbatim and retain their input line numbers.

### Lines 647–647

````
/// Refresh a clean table result while carrying row keys across records whose identity is
````

### Lines 648–648

````
/// uniquely proven by a declared primary tuple or, when unavailable, an exact full row.
````

### Lines 649–649

````
/// Ambiguous rows receive fresh monotonic keys so filtering never aliases their selection.
````

### Lines 677–677

````
// A NULL component cannot prove a primary-key identity, even in a unique result.
````

### Lines 697–697

````
// In a complete typed record, NULL is a known cell value and participates in
````

### Lines 698–698

````
// exact tuple equality when that tuple is unique in both results.
````

### Lines 882–882

````
/// Applies staged changes to this in-memory result and makes them its new baseline.
````

### Lines 883–883

````
///
````

### Lines 884–884

````
/// Existing row keys and current values survive. Deleted rows are removed, inserted
````

### Lines 885–885

````
/// rows become ordinary rows, and undo entries from before the commit are cleared.
````

## `crates/tablepro-domain/src/model.rs`

- Original source SHA-256: `b33fe020141d4b8286c3bd89a1774a42395447b7ea952611f62a0beff95cce24`
- Excerpts are verbatim and retain their input line numbers.

### Lines 136–136

````
// Oldest first. The history card shows this order, ten rows at a time.
````

## `crates/tablepro-domain/src/tabs.rs`

- Original source SHA-256: `42406f1f72ba847cd8defde48d01592b430eb6253a4aedc7b89c750752b50359`
- Excerpts are verbatim and retain their input line numbers.

### Lines 221–221

````
/// Shared multiline editor and completion state for this query tab.
````

### Lines 228–228

````
/// Most recent execution summary and whether it represents an error.
````

### Lines 293–293

````
/// Executes the query while recording the supplied elapsed duration.
````

### Lines 294–294

````
///
````

### Lines 295–295

````
/// This deterministic path does not sample a host clock.
````

### Lines 330–330

````
/// Explains the selected statement or the statement under the editor cursor.
````

### Lines 331–331

````
///
````

### Lines 332–332

````
/// Returns the exact SQL source used to create the successful plan.
````

### Lines 333–333

````
///
````

### Lines 334–334

````
/// # Errors
````

### Lines 335–335

````
///
````

### Lines 336–336

````
/// Returns an error when the selected or cursor statement is empty, invalid,
````

### Lines 337–337

````
/// is not a SELECT, or cannot be planned.
````

### Lines 397–397

````
/// Copies the editor's authoritative buffer into the SQL execution snapshot.
````

### Lines 405–405

````
/// Replaces the query and resets editor mechanics for a newly loaded statement.
````

### Lines 412–412

````
/// Commits active editing without changing the editor's text.
````

### Lines 419–419

````
/// SQL syntax ranges used by the shared code editor.
````

### Lines 439–439

````
/// Statement boundaries used for running the SQL statement at the cursor.
````

## `crates/tablepro-sql/src/sql.rs`

- Original source SHA-256: `d1a20b2a5eb7f76ad48541de22a83a62c717a04313964b9bd44905a95007b79e`
- Excerpts are verbatim and retain their input line numbers.

### Lines 799–799

````
/// Writes the same text as [`fmt_rows`] into `out` so frame-path callers can
````

### Lines 800–800

````
/// format into a stack buffer without a heap allocation.
````

### Lines 1003–1003

````
/// Compares values using `TablePro`'s simulated sort policy: NULL sorts last in
````

### Lines 1004–1004

````
/// either direction, while non-NULL values follow the requested direction.
````

### Lines 1005–1005

````
///
````

### Lines 1006–1006

````
/// The generic [`cmp_values`] ordering remains unchanged for existing callers.
````

## `crates/tablepro-ui/manual-api-contracts/workbench-tabs-privacy/README.md`

- Original source SHA-256: `337fd8f5e35daea746bffe3afbfbf2a1bf43ec9242e5ba6a914716f071222d96`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–45: manual API compile-boundary contract prose; fixture removed with its old target

````
# Workbench tab collection privacy check

This isolated manual fixture checks the public-consumer compile boundary for the structural privacy of `Workbench.tabs`. It is not a Nextest test, Rustdoc/doctest, automatic CI check, runtime-behavior test, release check, or proof for third-party installations. The [positive program](src/bin/workbench_tabs_positive.rs) uses the supported `new_query` method and immutable `tabs()` slice. The [negative program](src/bin/workbench_tabs_negative.rs) adds only direct mutation of the private collection.

At the reviewed immutable source revision, [`Workbench.tabs` is private](https://github.com/tailrocks/terminal-components-claude/blob/3ff0f4fb0c5cec82632296a6912286ca78b9ac01/crates/tablepro-ui/src/workbench.rs#L17), while [`tabs()`](https://github.com/tailrocks/terminal-components-claude/blob/3ff0f4fb0c5cec82632296a6912286ca78b9ac01/crates/tablepro-ui/src/workbench.rs#L137-L139) and [`new_query`](https://github.com/tailrocks/terminal-components-claude/blob/3ff0f4fb0c5cec82632296a6912286ca78b9ac01/crates/tablepro-ui/src/workbench.rs#L216-L220) provide the supported interface. [`TableProApp.workbench`](https://github.com/tailrocks/terminal-components-claude/blob/3ff0f4fb0c5cec82632296a6912286ca78b9ac01/crates/tablepro-ui/src/app.rs#L772) and the [crate-root exports](https://github.com/tailrocks/terminal-components-claude/blob/3ff0f4fb0c5cec82632296a6912286ca78b9ac01/crates/tablepro-ui/src/lib.rs#L41-L42) make that interface available to this external consumer.

The original structural-owner intent is retained in [the immutable source note](https://github.com/tailrocks/terminal-components-claude/blob/8ec895f9ea9cf78e5484d80f96b7189d5b716f00/crates/tablepro-domain/src/tabs.rs#L442-L455). That historical example has import and `Tab` versus `TabRecord` construction confounds, so its failure alone would not prove field privacy. This fixture uses a valid public construction and accessor before attempting the mutation. It does not claim that the historical example was executed or restore its original execution mechanism. It also does not test the separate privacy of `TabRecord.key`.

## Manual check

Run each command from this fixture directory. Run the positive first. Run the negative only after the positive returns 0 with the same source and lock. Check the complete diagnostic and confirm the intended package and bin; a guard refusal, timeout, or unrelated compiler failure is inconclusive.

```sh
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo check --locked --message-format=json --bin workbench_tabs_positive
```

Expected positive result: exit 0 for the intended fixture package and bin. Cargo may reuse valid cached work; a successful check does not imply that every dependency was recompiled or that the program was executed.

```sh
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo check --locked --message-format=json --bin workbench_tabs_negative
```

Expected negative result: exit 101 with one primary E0616 error at `src/bin/workbench_tabs_negative.rs`, line 5, columns 19–23 (the compiler end column is exclusive), on `tabs`. The primary diagnostic is:

> field `tabs` of struct `tablepro_ui::Workbench` is private

The compiler may also emit its ancillary E0616 explanation note. No unrelated error can substitute for this diagnostic.

This package has its own unpublished nested workspace, resolver 3, one `tablepro-ui` path dependency, and two explicitly declared bins with automatic target discovery disabled. Root workspace checks and `--all-targets` do not automatically compile these bins; selecting both bins together is expected to fail because one is the negative witness. The fixture lock was initialized once by the guarded positive check without `--locked`, then reviewed as a complete fixture-specific graph. Routine checks use the retained lock with `--locked`; do not regenerate it to work around drift, add tool/config overrides, or change root dependencies.

## Fixture inputs

These hashes identify the retained fixture inputs. The complete bootstrap, dependency-graph review and paired-check custody remain separate review evidence; they are not an automated coverage or release claim.

SHA-256 of the four retained fixture inputs:

| File | SHA-256 |
| --- | --- |
| `Cargo.toml` | `1bccd6601e1c49d63ce5078a8e20d19f1de61dc61cc02443d7c272209c416b83` |
| `Cargo.lock` | `d5dddf515ed38ebe4e1bcf5b6a1bc611f95c2c3811025ef2dc3775f0f6101bc2` |
| [`workbench_tabs_positive.rs`](src/bin/workbench_tabs_positive.rs) | `e443e8d477475e1e92baa2547ab255504912c8c3200778411bef6aa1a688f19d` |
| [`workbench_tabs_negative.rs`](src/bin/workbench_tabs_negative.rs) | `d22efae0a6a81e59d3f17cc78802ce0254d828e7adfbcad796b740778f61aa3f` |

The reviewed fixture lock contains 211 package records (25 path packages and 186 registry packages). Its fixture-only resolution differences are not root dependency-upgrade permission.
````

## `crates/tablepro-ui/src/app.rs`

- Original source SHA-256: `2a3bf0950180f32713a63c41918f5d7295bd7380e39fb4f0821b4d0599955ca1`
- Excerpts are verbatim and retain their input line numbers.

### Lines 157–157

````
// The query surface is the field color. A modal backdrop then lifts that
````

### Lines 158–158

````
// field to the elevated card. Canvas black never lifts, so the open
````

### Lines 159–159

````
// picker and help frames lose the card.
````

### Lines 373–373

````
// Frozen junie ink. Current steps are Muted #808080, Faint/BorderStrong #4d4d4d,
````

### Lines 374–374

````
// Info #8787ff, and Danger #e44545. None of those roles match the frame.
````

### Lines 377–377

````
// The frozen app paints the status green through the theme success slot
````

### Lines 378–378

````
// (apply_256 = i(71), apply_16 = Green), not a custom hex downgrade.
````

### Lines 381–381

````
/// Query seam. Truecolor matches border-strong `#4d4d4d`. 256-color is index 239.
````

### Lines 623–623

````
// A slow frame lets Esc and Ctrl+R sit in the same read. Crossterm
````

### Lines 624–624

````
// then reports one Alt+Ctrl+R. That chord is still Run.
````

### Lines 772–772

````
/// When the connect line was armed. `None` after the five-second ttl.
````

### Lines 774–774

````
/// `connect` has no clock. The next update stamps `connection_status_at`.
````

### Lines 824–824

````
/// Whether the filter card's value field holds the focus.
````

### Lines 961–961

````
/// Constructs an app that reports `duration` for every query execution.
````

### Lines 962–962

````
///
````

### Lines 963–963

````
/// This policy is active before the initial query runs and does not sample
````

### Lines 964–964

````
/// a host clock. Use it for deterministic embedding and replay.
````

### Lines 1131–1131

````
/// Whether the filter card's value field holds the focus.
````

### Lines 1364–1364

````
// The open form owns the only focus mark. The idle list keeps its cursor row.
````

### Lines 1385–1385

````
// The open form owns Tab order. The list still paints, but it is not a stop.
````

### Lines 1576–1576

````
// The query input registers before the explorer tree. Two updates let
````

### Lines 1577–1577

````
// the tree enter the focus ring before the idle frame is captured.
````

### Lines 2443–2443

````
// Direct assign: `set_surface` never opens layers. Picker rows and the
````

### Lines 2444–2444

````
// owner token come from `open` above, the same builder production uses.
````

### Lines 2462–2462

````
// Direct assign: `set_surface` never opens layers. Picker rows and the
````

### Lines 2463–2463

````
// owner token come from `open` above, the same builder production uses.
````

### Lines 2614–2614

````
// Refuse to interpret old picker state against the replacement Workbench.
````

### Lines 2615–2615

````
// Acknowledge only the root and results owner buckets this picker owns.
````

### Lines 2662–2662

````
// Clean tabs close immediately and reopen the picker with
````

### Lines 2663–2663

````
// a fresh query/cursor. Dirty tabs keep it beneath the
````

### Lines 2664–2664

````
// confirmation so Cancel restores the exact picker state.
````

### Lines 2691–2691

````
// Structure and history currently use application-painted bodies,
````

### Lines 2692–2692

````
// not a focusable shared widget. Leave focus restoration to the
````

### Lines 2693–2693

````
// layer runtime until those views have real shared controls.
````

### Lines 2700–2700

````
// Pump the picker first so the focused key is consumed even when the
````

### Lines 2701–2701

````
// level turns out to be stale; its action is discarded below.
````

### Lines 3063–3063

````
// Inactive query tabs still receive their addressed focus-out so the shared
````

### Lines 3064–3064

````
// component can settle its lifecycle. Their edit session is retained because
````

### Lines 3065–3065

````
// the tab is removed from the visible composition; active editors commit on
````

### Lines 3066–3066

````
// ordinary blur as before.
````

### Lines 3142–3142

````
// A filtered page is already the scan. Sorting the raw table would drop the filter.
````

### Lines 3396–3396

````
// The pending bar keeps one blank row above it, so the grid loses two.
````

### Lines 3564–3564

````
// The dialog takes focus. Baseline draws that title as secondary,
````

### Lines 3565–3565

````
// which the backdrop then dims to faint. The border stays strong.
````

### Lines 3673–3673

````
// One extra trailing space shifts the framed meta one cell left.
````

### Lines 3819–3819

````
// The detail edge is outside the clipped body allocation. Keep its
````

### Lines 3820–3820

````
// pad aligned to the actual list/detail start and stop at that body.
````

### Lines 4144–4144

````
/// The connect line is success ink. It expires after five seconds, and a
````

### Lines 4145–4145

````
/// tick is scheduled so a waiting frame can see the hints take its place.
````

### Lines 4159–4159

````
// A wide row already shows Explain beside the status. Waking at the
````

### Lines 4160–4160

````
// deadline would drop the status after the snapshot was taken. A narrow
````

### Lines 4161–4161

````
// row hides Explain until the line expires, so that frame has to wake.
````

### Lines 4164–4164

````
// The query line offers six groups. The reference footer drops
````

### Lines 4165–4165

````
// the groups that fall off the right end, so the last groups go
````

### Lines 4166–4166

````
// first.
````

### Lines 4227–4227

````
/// Baseline records every failed statement as a 2 ms result.
````

### Lines 4251–4251

````
/// Baseline `PLAN_COLS_W`: the cost strip stays this wide on the tree.
````

### Lines 4253–4253

````
/// Baseline detail card. It appears only when the plan body is at least 110 wide.
````

### Lines 4364–4364

````
// The strip ends on the panel edge (`edge_x - 39` is the 120-column frame).
````

### Lines 4365–4365

````
// A body at least 110 wide also reserves the 40-column detail card and the
````

### Lines 4366–4366

````
// two-cell gap in front of it, so the strip does not run into that card.
````

### Lines 4388–4388

````
// The frozen explain frame leaves the plan cursor on the root.
````

### Lines 4893–4893

````
// An uncapped card still contains the frame's two-column tail, so the
````

### Lines 4894–4894

````
// rule stops six cells short. The 90-column cap has already dropped that
````

### Lines 4895–4895

````
// tail, and the rule then stops four cells short.
````

### Lines 5127–5127

````
// The detail card's last column sits on the panel edge. The edge marks
````

### Lines 5128–5128

````
// paint after the card, so a wide plan paints that column again here.
````

### Lines 5167–5167

````
// Match `paint_code_hbar`: one line uses a six-cell gutter, and the bar
````

### Lines 5168–5168

````
// exists only when the line is wider than the text area.
````

### Lines 5291–5291

````
// The 90-column cap already fills the clip. A wider card still owes the
````

### Lines 5292–5292

````
// one cell past the clip.
````

### Lines 5361–5361

````
// Truecolor stays the strong border #4d4d4d. The 256 frame uses index 239.
````

### Lines 5483–5483

````
// `·` is one cell here. `width` counts it wider and shifts the line.
````

### Lines 5490–5490

````
// Integer centering lands one cell left of the historical hint.
````

### Lines 5762–5762

````
/// Fixed-size buffer so the explorer metadata formats on the stack instead of
````

### Lines 5763–5763

````
/// allocating a `String` on every workbench frame (see the 100-alloc grid
````

### Lines 5764–5764

````
/// frame budget).
````

### Lines 5800–5800

````
/// The reference tree hides metadata on every row as soon as one visible row
````

### Lines 5801–5801

````
/// cannot fit its label and its metadata side by side.
````

### Lines 5802–5802

````
///
````

### Lines 5803–5803

````
/// The rows that count are the ones the tree shows right now: the viewport
````

### Lines 5804–5804

````
/// rows, not every expanded row.
````

### Lines 5846–5846

````
/// The width a row needs to carry its label and its metadata side by side.
````

### Lines 5893–5893

````
/// The metadata a node paints on its right hand side, if any.
````

### Lines 5894–5894

````
///
````

### Lines 5895–5895

````
/// The text is written into `buf`, so a workbench frame does not allocate a
````

### Lines 5896–5896

````
/// `String` per row.
````

### Lines 5914–5914

````
/// The width of the label a node paints, including its kind prefix.
````

### Lines 6038–6038

````
// Depth-1 label starts two cells after the disclosure column.
````

### Lines 6048–6048

````
// Cell styles only clear a modifier that is subtracted.
````

### Lines 6477–6477

````
// The demo executor scans 2_000 rows, then pages the first 500. A local sort
````

### Lines 6478–6478

````
// has to use that scan or the first page hides earlier keys.
````

### Lines 7565–7565

````
/// Which right-hand header pieces survive, plus the overflow column.
````

### Lines 7566–7566

````
///
````

### Lines 7567–7567

````
/// `None` means every piece fits and the existing cluster paint stays.
````

### Lines 7568–7568

````
/// The column is relative to `area.x`. Priorities match the baseline strip:
````

### Lines 7569–7569

````
/// the size badge is 1 and `? help` is 4, so the badge drops first.
````

### Lines 7570–7570

````
/// Keep/drop flags for header pieces, stack allocated so the workbench
````

### Lines 7571–7571

````
/// header stays allocation free on the per-frame draw path.
````

### Lines 7586–7586

````
/// The kept flags in piece order; empty when `len` runs past the
````

### Lines 7587–7587

````
/// fixed backing array.
````

### Lines 7611–7611

````
/// Which header pieces survive at `area_width`. Stack resident; the
````

### Lines 7612–7612

````
/// per-frame workbench draw path calls this directly.
````

### Lines 7651–7651

````
// Priorities run parallel to their piece slice; zip keeps the index in
````

### Lines 7652–7652

````
// bounds even if a caller passes mismatched lengths.
````

### Lines 7696–7696

````
/// Per-frame workbench header widths. The measured strings are never
````

### Lines 7697–7697

````
/// painted on this path, so each width is computed from borrowed pieces
````

### Lines 7698–7698

````
/// and the draw stays allocation free.
````

### Lines 7706–7706

````
// "{glyph} {label}" with a single-width environment glyph.
````

### Lines 7708–7708

````
// "{database} > {schema}" separator: space, chevron, space.
````

### Lines 7716–7716

````
// "{capability} . {term_width}x{term_height}" with single-width
````

### Lines 7717–7717

````
// middle dot and multiplication sign.
````

### Lines 7736–7736

````
/// Decimal digit count used to size the "{width}x{height}" badge without
````

### Lines 7737–7737

````
/// formatting it.
````

### Lines 7834–7834

````
// Clickable help paints a muted pad on each side of the label.
````

### Lines 7907–7907

````
// Two cells stay clear of the capability cluster.
````

### Lines 7977–7977

````
// The middle of the blank row under the title is path grey.
````

### Lines 8364–8364

````
// The picker layer owns the footer hints; a picker that cannot be
````

### Lines 8365–8365

````
// searched offers no "Type Filter" group.
````

### Lines 8382–8382

````
/// The connection status the workbench footer shows on its right edge.
````

### Lines 8383–8383

````
///
````

### Lines 8384–8384

````
/// The status carries its check mark: it reports a connection that is up.
````

### Lines 8402–8402

````
// The status on the right edge, and the hint groups on the left. The
````

### Lines 8403–8403

````
// module owns how the two share the row.
````

### Lines 8409–8409

````
/// The status the footer shows on its right edge.
````

### Lines 8410–8410

````
///
````

### Lines 8411–8411

````
/// The connection status is the default. A sort, a filter or a notice
````

### Lines 8412–8412

````
/// replaces it while it lasts.
````

### Lines 8441–8441

````
/// The hint groups the surface offers, and the plate that sits before them.
````

### Lines 8834–8834

````
// A status tick must not run the grid. Its first update steals focus
````

### Lines 8835–8835

````
// onto the result row, and the baseline frame keeps the editor focused.
````

### Lines 8961–8961

````
// Explain keeps the editor in edit mode. Run is what commits.
````

### Lines 8985–8985

````
// Analyze keeps the editor in edit mode, same as Explain.
````

### Lines 9243–9243

````
// Typing the filter moves the hidden cursor onto the match. The open
````

### Lines 9244–9244

````
// detail stays on the connection that was selected before `/`.
````

### Lines 9473–9473

````
// Baseline collapses the explorer when the page is under 100
````

### Lines 9474–9474

````
// columns, unless that tree itself has focus.
````

### Lines 9487–9487

````
// The query is not on screen, but Tab still lands on its editor.
````

### Lines 9684–9684

````
// Paused frames keep press flash on a frozen simulation clock. The tag held flash time at zero.
````

## `crates/tablepro-ui/src/app/filter_editor.rs`

- Original source SHA-256: `06cdaee603434e97f5ac7cbb47ead9d42ca6137597693b215bbb16366dcb7d35`
- Excerpts are verbatim and retain their input line numbers.

### Lines 340–340

````
// Rebuild the horizontal window for the new result before restoring its cursor.
````

### Lines 341–341

````
// GridState keeps this offset across filtering, but the old measured window may
````

### Lines 342–342

````
// now omit useful leading columns even when the cursor column still exists.
````

## `crates/tablepro-ui/src/app/history_controls.rs`

- Original source SHA-256: `c48f5eccac63ae12c05101f59edc836e45d6b18520bd8ad331b64e8f10c1d591`
- Excerpts are verbatim and retain their input line numbers.

### Lines 113–113

````
// Hidden lifecycle may settle focus, but it must not commit into the tab model.
````

## `crates/tablepro-ui/src/app/history_detail.rs`

- Original source SHA-256: `0f763567c7e94c36bef4dcc561afc37e072283ada797b891f55744eab1006de7`
- Excerpts are verbatim and retain their input line numbers.

### Lines 115–115

````
/// True while this controller retains an opening that still needs exact
````

### Lines 116–116

````
/// receipt retirement, including a closing opening awaiting its receipt.
````

### Lines 277–277

````
// This guard immediately precedes the ID-based size mutation.
````

## `crates/tablepro-ui/src/app/history_rows.rs`

- Original source SHA-256: `8324198802e7124c606d195e6a4c0dda3f76da26d90a3f8e268516b9d5b505d0`
- Excerpts are verbatim and retain their input line numbers.

### Lines 194–194

````
// At the 72x20 minimum, the old Copy row is below the framed body.
````

### Lines 195–195

````
// The compact slot is between the two-line SQL preview and the facts.
````

## `crates/tablepro-ui/src/app/query_safety.rs`

- Original source SHA-256: `fec6f6b896b4240d8a94318dfc29d088ca6941379f4ad31cde43d0c1dc05e08e`
- Excerpts are verbatim and retain their input line numbers.

### Lines 44–44

````
// Ctrl+R and F5 currently execute the full editor buffer. The editor
````

### Lines 45–45

````
// selection does not select a different statement and is not execution
````

### Lines 46–46

````
// input for this route.
````

### Lines 380–380

````
// The same id now belongs to a different layer opening. Drop our
````

### Lines 381–381

````
// stale private state without updating, resizing, or dismissing it.
````

### Lines 427–427

````
// A different opening with the same id, or an externally closed review,
````

### Lines 428–428

````
// must release the captured request without closing the other layer.
````

### Lines 520–520

````
// The current write AST retains only the table token, not its SQL
````

### Lines 521–521

````
// schema qualifier. The active schema is useful context for the user,
````

### Lines 522–522

````
// but it cannot establish which object a qualified write names.
````

## `crates/tablepro-ui/src/app/result_grid.rs`

- Original source SHA-256: `c60e72e20dc292cce0173ad13b612988fe899741440d0883f3b553f3267123c2`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! The result grid view: column budgets, chrome, scroll, header, numbers, status.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! The view owns the per-frame grid paint state. A frame starts with
````

### Lines 4–4

````
//! [`ResultGridView::begin_frame`], paints the body with
````

### Lines 5–5

````
//! [`ResultGridView::draw`], then paints the scrollbar and the status row
````

### Lines 6–6

````
//! from the stored snapshot. Nothing grid-related lives in thread-locals.
````

### Lines 53–53

````
/// Paint the grid body, focus bar, row numbers, and header.
````

### Lines 54–54

````
///
````

### Lines 55–55

````
/// The origin is whatever the grid painted, so numbers, header, and
````

### Lines 56–56

````
/// scrollbar always agree with the body. There is no overpaint pass.
````

### Lines 146–146

````
/// Paint the status row, pending bar, and horizontal bar for a table view.
````

### Lines 188–188

````
// The column window is the clause that drops once a sort prefix lands.
````

### Lines 207–207

````
/// The vertical scroll bar of a grid body.
````

### Lines 217–217

````
/// The horizontal scroll bar of a grid body.
````

### Lines 227–227

````
/// The keyboard-focus bar of a grid body.
````

### Lines 233–233

````
/// The viewport of a query result grid: the scroll bar lane sits right of
````

### Lines 234–234

````
/// the grid body and the horizontal bar takes the grid body's last row.
````

### Lines 263–263

````
/// The chrome the shared grid bars align to.
````

### Lines 431–431

````
// A reference mark needs a cell of its own. Width 6 is the
````

### Lines 432–432

````
// whole value, the way the baseline grid skips the arrow.
````

### Lines 454–454

````
// The follow arrow occupies the last two cells, so the text
````

### Lines 455–455

````
// budget is what truncate_middle actually paints.
````

### Lines 623–623

````
// " ∇" plus the gap the header keeps after the mark.
````

### Lines 626–626

````
// " ▴" / " ▾" follows the filter mark. Unfiltered sorts already fit in the sample.
````

### Lines 655–655

````
// Four cells sit on the overflow count. The filtered sort frame leaves them blank.
````

### Lines 716–716

````
// The inline editor keeps two cells of field chrome past the sample.
````

### Lines 717–717

````
// A committed edit keeps that width on the focused text column.
````

### Lines 799–799

````
// End keeps the cursor column primary even when that column is not sorted or wide.
````

### Lines 826–826

````
// The mark is accent. The gap after it stays the header surface, sort included.
````

### Lines 846–846

````
// Editor chrome is wider than the header mark. The extra cells stay the header surface.
````

### Lines 860–860

````
// The two-cell gap after the editor header is muted, unlike other gaps.
````

### Lines 1203–1203

````
// Baseline scroll edges: under four rows stay solid, under twelve
````

### Lines 1204–1204

````
// fade only the outer row, and twelve or more fade two rows.
````

### Lines 1214–1214

````
// These two pads sit in the grid body. The edge fade already ran.
````

### Lines 1215–1215

````
// Repainting them in primary ink hides the bottom (and top) fade.
````

## `crates/tablepro-ui/src/footer.rs`

- Original source SHA-256: `eff4059310b46733acf01d8709cd5d36606581602c5c2dc69be25a9abc68e81f`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! The workbench footer: the hint groups on the left, the status on the right.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! The footer follows the shared hint bar's fitting rule:
````

### Lines 4–4

````
//!
````

### Lines 5–5

````
//! * The status owns the right edge. Its width is reserved before the hints
````

### Lines 6–6

````
//!   are placed, so a hint never runs under it.
````

### Lines 7–7

````
//! * The hint groups fill what is left, from the left.
````

### Lines 8–8

````
//! * When they do not fit, the groups that fall off the right end are
````

### Lines 9–9

````
//!   dropped — the pinned ones (`Esc`, `?`, `F1`) go last — and an overflow
````

### Lines 10–10

````
//!   marker takes the first of the freed columns.
````

### Lines 11–11

````
//!
````

### Lines 12–12

````
//! The app owns the *content* (which groups a surface offers, and what its
````

### Lines 13–13

````
//! status says). The *composition* — how a group is measured, how the status
````

### Lines 14–14

````
//! is reserved, and where the overflow marker goes — lives here so every
````

### Lines 15–15

````
//! surface renders the same way.
````

### Lines 19–19

````
/// The ink a hint action is written in.
````

### Lines 21–21

````
/// The ink the overflow marker is written in.
````

### Lines 23–23

````
/// The ink the edit badge is written in.
````

### Lines 25–25

````
/// The ground the edit badge sits on.
````

### Lines 28–28

````
/// One hint group: a key, a space, and an action.
````

### Lines 40–40

````
/// The width the group occupies, without the gap that follows it.
````

### Lines 50–50

````
/// The gap between two hint groups, and between a group and the overflow
````

### Lines 51–51

````
/// marker.
````

### Lines 54–54

````
/// The status a surface shows on the right of the footer.
````

### Lines 70–70

````
/// The overflow marker that replaces the groups that did not fit.
````

### Lines 73–73

````
/// How the footer divides the width it was given.
````

### Lines 76–76

````
/// The hint groups that are shown, in order.
````

### Lines 78–78

````
/// Where the overflow marker goes, when the groups did not all fit.
````

### Lines 82–82

````
/// Fit the hint groups into `room` columns.
````

### Lines 83–83

````
///
````

### Lines 84–84

````
/// The groups that fall off the right end go first and the pinned ones
````

### Lines 85–85

````
/// survive longest. The mask says which groups stay, and the flag says
````

### Lines 86–86

````
/// whether the overflow marker takes a column of its own.
````

### Lines 87–87

````
///
````

### Lines 88–88

````
/// `costs` is the width of every group, the gap that follows it included.
````

### Lines 110–110

````
// Once a group is gone, the overflow marker and the gap before it
````

### Lines 111–111

````
// take the room it left behind.
````

### Lines 124–124

````
/// Fit the hint groups beside a status of `status_width` columns.
````

### Lines 125–125

````
///
````

### Lines 126–126

````
/// The status always wins the right edge. The groups fill what is left, from
````

### Lines 127–127

````
/// the left; the ones that fall off the right end are dropped, and the
````

### Lines 128–128

````
/// overflow marker takes the first of those columns.
````

### Lines 141–141

````
// The status is separated from the last group by a gap.
````

### Lines 161–161

````
// The overflow marker takes the first of the freed columns.
````

### Lines 173–173

````
/// Paint the footer: the hint groups on the left, the status on the right.
````

### Lines 174–174

````
///
````

### Lines 175–175

````
/// `badge` is the small plate that sits before the first group (`EDIT` while
````

### Lines 176–176

````
/// a field owns the keyboard). `status` is the message on the right edge.
````

### Lines 186–186

````
// The badge sits in a cell of its own, then a gap before the first group.
````

## `crates/tablepro-ui/src/quick_switcher.rs`

- Original source SHA-256: `1ab4a8c79855330847dc46235e220700def9c19b5f576a8f990ed02696845ef6`
- Excerpts are verbatim and retain their input line numbers.

### Lines 41–41

````
// Baseline card: borders + title + query + gap + up to 12 rows + hint row.
````

## `crates/tablepro-ui/src/safe_mode.rs`

- Original source SHA-256: `e0e66d54551ac74367e2e45799eba269a2ebf181e391f1c8d150516f7ad07f7a`
- Excerpts are verbatim and retain their input line numbers.

### Lines 52–52

````
// Baseline non-searchable card: borders + title + rows + gap + hint.
````

## `crates/tablepro-ui/src/workbench.rs`

- Original source SHA-256: `9823884b16d05b2028c39ac8a899ebfd1df8a5382b565ecca5386f24d29194f6`
- Excerpts are verbatim and retain their input line numbers.

### Lines 255–255

````
/// Close a tab chosen for deletion from the tab picker, discarding its
````

### Lines 256–256

````
/// unsaved query or result-grid edits as the reference picker does.
````

### Lines 354–354

````
/// Executes the active query and records the supplied elapsed duration.
````

### Lines 355–355

````
///
````

### Lines 356–356

````
/// The explicit duration is passed to the query tab without sampling a host clock.
````

## `crates/termrock-consumer-example/src/app.rs`

- Original source SHA-256: `eb55e70bf891bc3b9de9ab27cdb1a186bd9a093bae7c22a1d681053841310cab`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! The external consumer application: a recipe shelf built only from the
````

### Lines 2–2

````
//! public Termrock APIs.
````

### Lines 3–3

````
//!
````

### Lines 4–4

````
//! The screen is composed as
````

### Lines 5–5

````
//!
````

### Lines 6–6

````
//! ```text
````

### Lines 7–7

````
//! AppShell
````

### Lines 8–8

````
//!  ├── header : MenuBar        (termrock-overlays)
````

### Lines 9–9

````
//!  ├── body   : MasterDetail   (termrock blocks)
````

### Lines 10–10

````
//!  │    ├── master  : NavList  (termrock-navigation)
````

### Lines 11–11

````
//!  │    ├── detail  : PropsList (termrock-controls)
````

### Lines 12–12

````
//!  │    └── summary : Button   (termrock-controls)
````

### Lines 13–13

````
//!  └── footer : StatusBar      (termrock-feedback)
````

### Lines 14–14

````
//! ```
````

### Lines 15–15

````
//!
````

### Lines 16–16

````
//! Every layout decision belongs to a shared component: the consumer never
````

### Lines 17–17

````
//! computes a rectangle, never paints a cell and never matches on a raw key
````

### Lines 18–18

````
//! or mouse event. Input is routed by the shared widgets, which answer with
````

### Lines 19–19

````
//! typed actions that the consumer maps onto its own domain actions.
````

### Lines 34–34

````
/// The master-detail workspace that owns the screen layout.
````

### Lines 36–36

````
/// The menu bar in the shell header.
````

### Lines 38–38

````
/// The recipe list in the master pane.
````

### Lines 40–40

````
/// The property sheet in the detail pane.
````

### Lines 42–42

````
/// The summary row below the master-detail panes.
````

### Lines 44–44

````
/// The summary row's second action.
````

### Lines 46–46

````
/// The status bar in the shell footer.
````

### Lines 48–48

````
/// The keyed status item that carries the note.
````

### Lines 51–51

````
/// The menu action that scales the selected recipe.
````

### Lines 53–53

````
/// The menu action that shows the about note.
````

### Lines 56–56

````
/// The status note shown in the shell footer.
````

### Lines 59–59

````
/// Nothing has happened yet.
````

### Lines 61–61

````
/// The selected recipe was scaled.
````

### Lines 63–63

````
/// The selected recipe was marked as cooked.
````

### Lines 65–65

````
/// The about note was requested.
````

### Lines 70–70

````
/// The static text this note renders as.
````

### Lines 81–81

````
/// The consumer's own typed domain actions.
````

### Lines 82–82

````
///
````

### Lines 83–83

````
/// Shared widgets answer with their own typed actions; the consumer maps them
````

### Lines 84–84

````
/// onto these so the domain never depends on a widget's vocabulary.
````

### Lines 87–87

````
/// The list cursor moved onto a recipe.
````

### Lines 89–89

````
/// The user chose a recipe.
````

### Lines 91–91

````
/// The detail pane opened in the narrow layout.
````

### Lines 93–93

````
/// The detail pane closed in the narrow layout.
````

### Lines 95–95

````
/// A menu item was chosen.
````

### Lines 97–97

````
/// The status bar reported a chosen item.
````

### Lines 99–99

````
/// The servings of a recipe were scaled.
````

### Lines 101–101

````
/// A recipe was marked as cooked.
````

### Lines 130–130

````
/// The recipe shelf application.
````

### Lines 163–163

````
/// The typed actions this consumer has produced, oldest first.
````

### Lines 168–168

````
/// The most recent typed action, if any.
````

### Lines 173–173

````
/// The recipe the list cursor currently addresses.
````

### Lines 178–178

````
/// The servings of the selected recipe.
````

### Lines 183–183

````
/// Whether the selected recipe is marked as cooked.
````

### Lines 188–188

````
/// The note the status bar renders.
````

### Lines 193–193

````
/// The recipes on the shelf.
````

### Lines 295–295

````
// The recipe list owns the keyboard when the shelf opens.
````

### Lines 300–300

````
// The layout block gets first refusal on Escape.
````

### Lines 344–344

````
// The detail pane always shows the recipe the list cursor addresses.
````

### Lines 440–440

````
/// The shell header leaf: the shared menu bar.
````

### Lines 458–458

````
/// The shell footer leaf: the shared status bar.
````

## `crates/termrock-consumer-example/src/lib.rs`

- Original source SHA-256: `38c0cf32d0636ce3412da27524eb97bdf4737242b2d613897cdc211c775265b4`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! An external consumer of the public Termrock component library.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! This crate stands outside every application in the repository: it depends
````

### Lines 4–4

````
//! on the public `termrock` facade only, carries its own domain data (a
````

### Lines 5–5

````
//! recipe shelf) and composes a real screen — shell, layout block and
````

### Lines 6–6

````
//! interactive widgets — through public APIs.
````

### Lines 7–7

````
//!
````

### Lines 8–8

````
//! The crate contains no private helpers borrowed from an application, no
````

### Lines 9–9

````
//! hand-written geometry and no direct painting: the shell owns the layout,
````

### Lines 10–10

````
//! the shared widgets own the interaction, and the consumer only maps typed
````

### Lines 11–11

````
//! widget actions onto typed domain actions.
````

## `crates/termrock-consumer-example/src/main.rs`

- Original source SHA-256: `a1996c5c320cabb5bdbc92b634528f9512edb09636f504f956badb9b7ad570da`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! The consumer's own binary: a runnable recipe shelf.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! It proves the crate is a standalone application built on the public
````

### Lines 4–4

````
//! Termrock APIs alone — no application crate and no private helper is
````

### Lines 5–5

````
//! reachable from here.
````

## `crates/termrock-consumer-example/src/recipes.rs`

- Original source SHA-256: `0e0212fbe723fbe84df9871ef9256f99b35d3ee99b0fe3c7760366ac44c60076`
- Excerpts are verbatim and retain their input line numbers.

### Lines 1–1

````
//! The consumer's own domain: a recipe shelf.
````

### Lines 2–2

````
//!
````

### Lines 3–3

````
//! Nothing here is shared with any application in this repository. The data
````

### Lines 4–4

````
//! is deliberately unrelated to the showcase, Holla, TablePro or Jackin
````

### Lines 5–5

````
//! domains so the consumer proves that the Termrock components carry an
````

### Lines 6–6

````
//! arbitrary payload, not one application's vocabulary.
````

### Lines 10–10

````
/// One labelled fact about a recipe, rendered as a property row.
````

### Lines 13–13

````
/// The stable key of this fact inside its recipe.
````

### Lines 15–15

````
/// The left-hand label of the property row.
````

### Lines 17–17

````
/// The right-hand value of the property row.
````

### Lines 21–21

````
/// One recipe on the shelf.
````

### Lines 24–24

````
/// The stable identity of the recipe.
````

### Lines 26–26

````
/// The name shown in the recipe list.
````

### Lines 28–28

````
/// The section the recipe is filed under.
````

### Lines 30–30

````
/// The number of facts the recipe carries.
````

### Lines 35–35

````
/// The stable key the shared list widgets address this recipe by.
````

### Lines 41–41

````
/// The shelf this consumer ships with.
````
