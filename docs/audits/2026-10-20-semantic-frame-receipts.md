# Semantic frame receipts

The pinned `tuiscotti` revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`
exposes `Frame::to_json()`, which serializes every cell and the complete cursor
record. Its `ansi_dump_with_cursor()` emits only `ESC[?25l` for a hidden cursor;
hidden cursor position, shape, and blink intent therefore disappear from the
ANSI projection. ANSI-only paired receipts could report a false match.

The paired harness now stores a sibling
`checkpoint-<ordinal>-<identity>.frame.json` beside every runtime `.ansi`
expectation. It also writes each attempted frame, before comparison, to the
sealed diagnostic root `receipts.semantic-actual/` using the same checkpoint
filename. This normalized canonical JSON preserves actual frames when a
semantic mismatch fails before the grouped ANSI store can emit its artifacts.
Semantic JSON is produced by cloning `Frame`, setting provenance to the pinned
tool's default metadata with stable `source: "screen"`, and calling
`Frame::to_json()`. This
preserves dimensions, all cell symbols, widths, continuation flags, colors,
modifiers, and cursor position, visibility, shape, and blink intent while
removing host-specific capture metadata. The file is created only by baseline
capture. Replay and candidate phases require it and compare its exact canonical
bytes before writing a receipt. The diagnostic root uses create-new writes and
the same symlink/hardlink path checks; it never overwrites expected data.

Each receipt retains `sha256` for canonical ANSI and adds the required
`semantic_frame_sha256` for the canonical semantic JSON. Consumers must reject
receipts missing either field or either sibling artifact. Both expected files
use the paired store's path validation and create-new rules. The external
runner must also seal the `.frame.json` inventory, reject symlink and hardlink
aliases, verify the semantic hash, and compare both receipt digests across the
baseline capture, baseline replay, and candidate phases.

Focused helper tests establish a matching baseline as a positive control and
prove that a character mutation fails. They also prove hidden cursor position,
shape, and blink mutations fail even when pinned ANSI output remains identical,
preserve full diagnostic JSON for each mismatch, and reject a hardlinked
diagnostic destination. The frozen runner's case identity is the slash-form
scenario name; the separate colon-form Nextest test ID is used only to select
the test, not passed to `begin_case`.

This is a test-harness integrity repair. It does not resolve the separate
stored approval mismatch between the frozen resize capture and its existing
approval. Full paired acceptance remains incomplete until the external runner
requires and compares the semantic receipts on both consumer branches.

## Narrow frozen approval conflict decision

The independent pinned-binary reruns identify one frozen data conflict:
Nextest selection ID `showcase::resize::overview_grown::120x40::truecolor`,
runner scenario identity `showcase/resize/overview_grown/120x40/truecolor`, checkpoint 0,
`showcase/resize/overview_grown/120x40/truecolor`. The frozen source tree hash
is `f74646a92505bd9baff29c89398af4d31a4812dd7d51acc84d755be61a383850`; the
baseline tag object is
`2718a6b8127a522a3d351d0b4bdb7542fc8da542` and the frozen source commit is
`4169783ae883788186d161af09ba9c481bd84e5c`; the
showcase binary hash is
`21e8b1c1af30b47b331aed033525c4682726b99de5fe1238d4507e58f84359d1`; and the
pinned tuiscotti revision is
`a1ea27f757d75d07b05499276cd932e9f8e067ad`. The retained approval is 18,474
bytes with SHA-256
`7769a6686dfc5cb5c69facb03f54a0fa57e963b35fcf315b1c4625745e2b7708`. Repeated
frozen CLI output is 19,454 bytes, with ANSI SHA-256
`f2e07ed284673b1d543441767460193e987034e163ca7ea7d3d4496473a0d442` and
semantic frame SHA-256
`01e9f7d349fc8075df7b64188db97a05705792aac8c781184db14a6d1ecc253a`. The
current pinned decision JSON SHA-256 is
`2a43fb118c268300c8bbd6439ea11bb9a24ffff03dc3c9f4601893eee26a096b`.
The independent test source digest is
`bdd772e6d049efd700912bc5d7b4464d11bb80e69a5d18b0f175b9421546af05`; the
inventory digest is
`d260d2f0644d63a3a1b4c1611c0d7c61777e59e61249e3ca513d1d6217b0639c`. The
runner fixes `VISUAL_FULL=1`, `VISUAL_REVIEW=0`, `TERM=dumb`, `NO_COLOR=1`, an
empty `COLORTERM`, and `LANG=LC_ALL=C.UTF-8` for this decision.

The helper exception is limited to that decision digest, test case, ordinal,
logical gate name, original approval bytes, and repeated frozen output hashes.
It accepts only an ANSI cell mismatch (`CellsDiffer` with `ansi_match == false`)
for baseline capture. It records the decision digest on that checkpoint receipt
and preserves original approval bytes. Baseline replay and candidate comparison
remain exact against the newly recorded ANSI and semantic frames. A missing or
changed approval, any changed cursor or cell, another status, a partial policy,
or another checkpoint remains a failure. The Python runner must validate the
decision document and its frozen source, binary, tool, and environment pins
before passing the policy values to the one-case baseline capture.
