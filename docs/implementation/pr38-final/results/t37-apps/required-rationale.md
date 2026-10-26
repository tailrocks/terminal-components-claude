# Required source rationale

Input commit: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.

These comments and excerpts came from scoped source that T37 removed. They retain the source text, file SHA-256, and original line numbers. The shared docs owner can use them to update maintained contracts. The full comment archive is in `source-prose.md`.

## Showcase route and caller state

Source: `crates/showcase-ui/src/app.rs`, SHA-256 `9e838f350342b0614d32feacf33f1c3a54febc7c19eae9d8f2b9b7388027d297`, lines 237–245 and 1446–1450.

```text
/// Controls how navigation groups follow committed route changes.
/// Keep every group open by default; later routes preserve manual folds.
/// Keep only the committed destination group open; query previews preserve saved folds.
/// Close only the group opened automatically for the previous route.
/// Sets the navigation expansion policy and resets saved folds for the current route.
///
/// This is an explicit reset, even when `policy` is already selected. During a navigation
/// query it preserves the query text, focus, and current route, then refreshes the filtered
/// cursor against the reset expansion state.
```

Source: `crates/showcase-demos/src/pages/checks_radios.rs`, SHA-256 `83dd62b560c57a8c9ab2e0f14b604dc7191347c282b1a7ff44d49cc64c0fa595`, lines 187–188.

```text
// The source page rebuilds this first field from the Mixed option on every option
// change; the other values and radio cursor selections remain user state.
```

Source: `crates/holla-ui/src/dispatch.rs`, SHA-256 `bb291b9e638009621fab0425598078c7a57cf7505845fc6efbcd3cbd041e5213`, lines 65, 174, and 367.

```text
// Every reserved typed producer stays off the legacy fixed-command preview route.
// Raw identity matters: recent/pin wrappers intentionally keep their ordinary static route.
// Direct generic confirmation must not bypass either typed input producer.
```

## Holla simulation and secret limits

Source: `crates/holla-domain/src/args.rs`, SHA-256 `03f8b5a902fdf960803f6dd52df67b25c1478abdd74b7ac30814e5f8f460df61`, line 100.

```text
/// Marks a field as secret so schema construction can reject it before accepting values.
```

Source: `crates/holla-sim/src/disk_scan.rs`, SHA-256 `80da6277d393419c9cc776539e3570b0194f822d19cdc47baa4ee826578395e9`, lines 1–5 and 14–17.

```text
//! Pure, deterministic disk-scan model over an immutable FilesSnapshot.
//!
//! This module never inspects the host filesystem, starts work on another
//! thread, or creates cleanup actions. It eagerly builds a modeled tree and
//! reveals its preorder at caller-supplied simulation ticks.
/// Times are simulation ticks, not wall-clock durations. A zero reveal
/// duration uses one effective tick, matching the historical minimum-one-tick
/// behavior. The caller supplies absolute simulated snapshot keys verbatim;
/// this model never canonicalizes or resolves them through the host OS.
```

Source: `crates/holla-sim/src/task_run.rs`, SHA-256 `592fc3a1d29e4c57140e19abcfc7f7b23970d2a1b279a348f174b5f1985c65c8`, lines 1–3 and 140–145.

```text
//! Deterministic task-run model for authored simulated tasks.
//!
//! This module models scripted output and prompts. It never launches a process.
/// Returns the effective world-clock boundary where the prompt opened.
/// Returns whether a future UI must treat the answer as secret.
```

Source: `crates/holla-sim/src/plans.rs`, SHA-256 `9df67fdda87d6288358c1b83c419f3d64e307a35735d2ff5647d29dab58ea84c`, lines 233–238.

```text
/// Build the existing disk-reclaim review for an exact nonempty subset of the current inventory.
///
/// The full inventory is validated; duplicate or absent candidate paths and ActiveToday rows are refused.
/// This path-only API cannot detect metadata changes at an unchanged path before construction, so
/// callers must compare their complete opening inventory and context. The resulting review then
/// binds its full target for approval.
```

Source: `crates/holla-sim/src/port_args.rs`, SHA-256 `d6371bf746d58f9bc02e15161c582470355b4b4e6adbb8608c6d22f0417ccca7`, lines 25, 321, and 479–482.

```text
/// Fixed producer declaration for the authored, read-only `system.port` action.
/// This does not run `lsof`, inspect processes, mutate activities, or change `World`.
/// Revalidates the accepted port request and projects optional deterministic listener data.
///
/// The request's effective numeric port is authoritative; accepted text, action labels, command
/// strings, and argv are never reparsed to select model rows. Nothing is executed.
```

## Jackin credential and terminal limits

Source: `crates/jackin-preview-app/src/app/accounts_page.rs`, SHA-256 `ee96c34dec6cea4447e1e6bb57168f6f9b6e1dceab2192c03bb812805bdc5d1d`, line 2355.

```text
/// The credential line of an account: the reference and its masked tail.
```

Source: `crates/jackin-preview-app/src/app/manager_detail_drawer_view.rs`, SHA-256 `0a8829c2f6106bee849e18a3b1024249f8e1a115aab010de400caad8a8dc6632`, lines 48–49.

```text
// The approved frame hides the cursor. This path must not call
// set_cursor or set_bar_cursor, and it must not write cursor bytes.
```

## TablePro caller, identity, and SQL limits

Source: `crates/tablepro-domain/src/connections.rs`, SHA-256 `72c63ab2224bb8a961cb70309f3c26e42ce95a4e29887006d94c41325c5df5fa`, lines 171–175.

```text
/// Creates the blank draft used for a new connection.
///
/// Driver defaults are retained while identity fields remain empty so the
/// user does not accidentally save the currently selected connection as a
/// duplicate.
```

Source: `crates/tablepro-domain/src/domain.rs`, SHA-256 `749d77bab032282808e92e1ca8d6b8ea9df41eccf76f6e30785da02d27a17154`, lines 647–649, 677, and 697–698.

```text
/// Refresh a clean table result while carrying row keys across records whose identity is
/// uniquely proven by a declared primary tuple or, when unavailable, an exact full row.
/// Ambiguous rows receive fresh monotonic keys so filtering never aliases their selection.
// A NULL component cannot prove a primary-key identity, even in a unique result.
// In a complete typed record, NULL is a known cell value and participates in
// exact tuple equality when that tuple is unique in both results.
```

Source: `crates/tablepro-sql/src/sql.rs`, SHA-256 `d1a20b2a5eb7f76ad48541de22a83a62c717a04313964b9bd44905a95007b79e`, lines 1003–1006.

```text
/// Compares values using `TablePro`'s simulated sort policy: NULL sorts last in
/// either direction, while non-NULL values follow the requested direction.
///
/// The generic [`cmp_values`] ordering remains unchanged for existing callers.
```

Source: `crates/tablepro-ui/src/app/query_safety.rs`, SHA-256 `fec6f6b896b4240d8a94318dfc29d088ca6941379f4ad31cde43d0c1dc05e08e`, lines 44–46 and 520–522.

```text
// Ctrl+R and F5 currently execute the full editor buffer. The editor
// selection does not select a different statement and is not execution
// input for this route.
// The current write AST retains only the table token, not its SQL
// schema qualifier. The active schema is useful context for the user,
// but it cannot establish which object a qualified write names.
```

Source: `crates/tablepro-ui/manual-api-contracts/workbench-tabs-privacy/README.md`, SHA-256 `337fd8f5e35daea746bffe3afbfbf2a1bf43ec9242e5ba6a914716f071222d96`, lines 1–45.

The full original README is preserved verbatim in `source-prose.md`. It records the public-consumer privacy fixture and its historical source references. The README and both manual fixture programs were removed with the old manual target.
