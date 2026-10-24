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
