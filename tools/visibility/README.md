# Visibility evidence tools

This note describes the operational boundary between source-pinned observations, generated views, and source-record mutations at [revision `7b2eb51`](https://github.com/tailrocks/terminal-components-claude/commit/7b2eb51c925a105f210c853e81b8ad440dad210f). A recorded status is not current remote state, product readiness, or proof that a check ran.

## Status report

[`status.py`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/status.py#L4481-L4530) reads [`source-facts.json`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/source-facts.json#L1-L46) and accepted task records, then renders `target/visibility/STATUS.md` by default. That generated report is an ignored operational artifact; it does not recreate a maintained root `STATUS.md`.

| Mode | Effect |
| --- | --- |
| no mode | Prints the rendered report to stdout. |
| `--validate` | Validates inputs and rendering without reading or writing the report view. |
| `--check` | Reads and compares the generated view with current inputs. |
| `--write` | Writes the generated view. |

`--output` selects a report-view path only within `target/visibility/<name>.md`; the path must be relative and the output directories must not be symlinks. `--role candidate|reference` changes presentation metadata, not source authority. See [`report_view_path`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/queue.py#L2557-L2579) and the CLI implementation above.

The pinned record says its latest top-level observation was made on 2026-10-09 and contains a branch API observation from 2026-10-08. It records that raw API response files were not preserved; the fetch record is coordinator-reported, has no raw fetch output, and has no completion timestamp. It also records that branch last-update time was not captured. These facts describe that dated record only; they do not establish later remote heads or current authentication. The generated report links to the `source-facts.json` data source, not to this README: the reporter constructs that link at [`status.py:3976`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/status.py#L3976) and includes it in the rendered report at [`status.py:4268`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/status.py#L4268). See also the [`source-facts.json` observation fields](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/source-facts.json#L4-L46).

The separate live publisher and Observer-ingest paths are source-gated at this revision. Live publication is disabled by `LIVE_PUBLISHING_ENABLED = false` outside the debug-only test mode; Observer ingestion reports collection disabled when its trusted workflow policy is not pinned. This local renderer and queue do not enable those paths, authorize a policy change, or provide credentials. See [`repository_writer.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-visibility-publisher/src/repository_writer.rs#L20-L25) and [`observer-ingest.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-visibility-publisher/src/bin/observer-ingest.rs#L166-L182).

## Queue views and source records

The accepted task JSON is authoritative; `WORK_QUEUE.md` is a derived view. The queue CLI has separate validate, check, render, mutation, and registry operations. Read the specific subcommand implementation before using it; a command name alone does not establish that it is read-only. See the [`queue.py` parser](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/queue.py#L2858-L2947).

| Operation | Effect |
| --- | --- |
| `validate` | Validates accepted queue records without reading the Markdown view. |
| `check` / `check-v2` | Reads records and checks that the corresponding generated view matches. |
| `render` / `render-v2` | Prints the deterministic view to stdout by default. With `--output`, writes a generated view under `target/visibility`. |
| `repair-render` | Explicitly writes the schema-v1 derived view after checking the repository branch and records. The current schema-v2 check reports that it has no v2 repair command. |
| `accept`, `transition`, and related record operations | Mutate authoritative task records and the derived view after their validations. They are not report checks. |

`accept` and `transition` use `_mutate`: under a lock it reads the current records, rejects a stale view, validates the branch and filesystem scopes, applies the change, validates the changed scopes, and then writes. `_write_pair` replaces the authoritative task file first and the rendered view second. An interruption between those replacements can leave a stale view; a later check fails and explicit repair is needed. These two writes are not one transaction. See [`_mutate` and `_write_pair`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/queue.py#L2784-L2834), [render dispatch](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/queue.py#L2986-L3025), and [repair-render](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/visibility/queue.py#L2837-L2855).

The status and queue integration-test sources are [`status.rs`](../../crates/termrock-visibility-tests/tests/status.rs) and [`queue_v2.rs`](../../crates/termrock-visibility-tests/tests/queue_v2.rs); the Python queue tests are in [`test_queue.py`](tests/test_queue.py). Their presence is source evidence, not a report that these tests passed on the current revision.

For local workflow context, see [Development — Documentation compatibility](../../docs/development.md#documentation-compatibility). Local validation and generated views remain separate from the publisher and Observer-ingest interfaces. Their current disabled status is specific to the reviewed source; this note does not authorize publishing, enable a publisher, or grant credentials.
