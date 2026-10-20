# Semantic frame receipts

The pinned `tuiscotti` revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`
exposes `Frame::to_json()`, which serializes every cell and the complete cursor
record. Its `ansi_dump_with_cursor()` emits only `ESC[?25l` for a hidden cursor;
hidden cursor position, shape, and blink intent therefore disappear from the
ANSI projection. ANSI-only paired receipts could report a false match.

The paired harness now stores a sibling
`checkpoint-<ordinal>-<identity>.frame.json` beside every runtime `.ansi`
expectation. Semantic JSON is produced by cloning `Frame`, setting provenance
to `Provenance::default()`, and calling the pinned `Frame::to_json()`. This
preserves dimensions, all cell symbols, widths, continuation flags, colors,
modifiers, and cursor position, visibility, shape, and blink intent while
removing host-specific capture metadata. The file is created only by baseline
capture. Replay and candidate phases require it and compare its exact canonical
bytes before writing a receipt.

Each receipt retains `sha256` for canonical ANSI and adds the required
`semantic_frame_sha256` for the canonical semantic JSON. Consumers must reject
receipts missing either field or either sibling artifact. Both expected files
use the paired store's path validation and create-new rules. The external
runner must also seal the `.frame.json` inventory, reject symlink and hardlink
aliases, verify the semantic hash, and compare both receipt digests across the
baseline capture, baseline replay, and candidate phases.

Focused helper tests establish a matching baseline as a positive control and
prove that a character mutation fails. They also prove hidden cursor position,
shape, and blink mutations fail even when pinned ANSI output remains identical.

This is a test-harness integrity repair. It does not resolve the separate
stored approval mismatch between the frozen resize capture and its existing
approval. Full paired acceptance remains incomplete until the external runner
requires and compares the semantic receipts on both consumer branches.
