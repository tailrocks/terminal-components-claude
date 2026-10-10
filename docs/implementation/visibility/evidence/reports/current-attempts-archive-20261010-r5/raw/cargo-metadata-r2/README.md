# Diagnostic metadata index-fill attempt (R2)

This packet proposes one Root-run command, and only after independent review:

```text
/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo metadata --manifest-path /private/tmp/termrock-vis11-diagnostic-candidate-e0bc-20261010/Cargo.toml --locked --no-deps --format-version 1
```

It reuses the already-isolated R3b Cargo home without recopying it. R3b stopped offline because the sparse registry index lacked `reqwest`; the candidate lock, source, index, and donor remained unchanged. R2 freshly walks and hashes the current working cache before execution rather than inferring its state from the donor-copy record. The fresh R2 preflight currently binds 23,570 entries and the exact cache-root lstat identity. It does not claim that Cargo bookkeeping was unchanged across earlier failed metadata attempts. R1 was reviewed read-only and correctly required a post-command check of the Cargo-home root identity, because walking its children alone would not detect root replacement or mode changes. R2 adds that physical-directory/non-symlink/device/inode/mode check and puts it in the pass predicate. R1 remains byte-preserved at `/private/tmp/termrock-vis11-diagnostic-metadata-e0bc-r1-luna-20261010`; its preflight was NOT_RUN.

The metadata command uses an empty inherited environment, private HOME/TMP/XDG/target, the existing isolated CARGO_HOME, explicit `CARGO_NET_OFFLINE=false`, sparse crates.io, bounded retry/HTTP timeout, a 600-second process deadline, and an 8 MiB combined-output cap. The pinned Git wrapper blocks Git network subcommands; the exact locked `tuiscotti` revision is already cached.

The runner records a complete before/after cache inventory. It permits only additions or updates under `registry/index/` plus in-place content changes to the three already-existing regular files `.global-cache`, `.package-cache`, and `.package-cache-mutate`. Each remains regular mode 0644 and is capped at 1 MiB; no deletion or addition of these markers is allowed. Every other path change fails closed, including package archives, registry source/cache trees, and Git checkout paths. The Cargo-home root must remain the exact physical non-symlink directory with the same device, inode, and mode 0755.

This metadata-only attempt does not build or test, invoke the Velnor CLI, generate a preview, or establish complete cache closure or qualification. A successful result means only that this bounded metadata query resolved and the expected index entry exists afterward.
