# Session typed-input preservation — 2026-10-21

## Decision

Remove `promote_escaped_alt` from `crates/termrock-session/src/session.rs` and return to the session boundary's direct `Input::from_crossterm(read()?)` conversion. Crossterm's event conversion preserves the key code and modifiers it reports. Session timing, character case, or a preceding Escape event cannot safely infer a missing modifier.

This follows frozen commit `4169783ae883788186d161af09ba9c481bd84e5c`: `src/runtime.rs::next_ready_input` passes each returned Crossterm event to `Input::from_crossterm_with` and returns that decoded input without a later clock-based modifier rewrite. The independent review of PR #38 candidate `bfefea9ce4b3ab91f846239d5f315534ccc2ff43` concluded removal restores that behavior. Review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/session-alt-review-bfefea9-20261021/review.json`, SHA-256 `945926dd56ee0fa63c23d114e53c2fa0fd1bb6749763c212ad72fcb235550731`.

## Rejected evidence retained

The removed author test at `bfefea9ce` created a synthetic bare Escape `Input`, called the private promotion helper, then created a synthetic bare `x` `Input` and asserted the helper added `ALT`. Its relevant body is retained here as rejected evidence. The helper it called no longer exists, so the excerpt is quoted as text and is not a compiled example:

```text
fn a_bare_character_right_after_esc_becomes_alt() {
    let mut esc_at = None;
    let mut esc = Input::Key(Key {
        code: KeyCode::Esc,
        mods: KeyModifiers::NONE,
    });
    super::promote_escaped_alt(&mut esc, &mut esc_at);
    assert!(esc_at.is_some());
    let mut typed = Input::Key(Key {
        code: KeyCode::Char('x'),
        mods: KeyModifiers::NONE,
    });
    super::promote_escaped_alt(&mut typed, &mut esc_at);
    let Input::Key(key) = typed else {
        panic!("key");
    };
    assert!(key.mods.contains(KeyModifiers::ALT));
    assert_eq!(key.code, KeyCode::Char('x'));
    assert!(esc_at.is_none());
}
```

It exercised the heuristic in isolation. It did not convert terminal events or dispatch them through the runtime, so it did not establish that ordinary typed input should acquire a modifier.

### Superseded RED claim and corrected coverage

The earlier statement that the committed test failed against the old production promotion path is withdrawn. The run `65fa885d-5f9d-409c-a66a-d7d4cfbf8943` was a synthetic injected-helper probe, not a run of the committed test against unmodified production code. The test then directly called `Input::from_crossterm`, bypassing the Session loop. Independent review of commit `51fe8afbd347a4ac543e4c12320507cfe6c9799d` found this gap (packet SHA-256 `9dfc88fd6977f992dcb0ed83aa8c476013d20e538a48b77a1314709382f84f7f`).

The corrected regression now calls the private `decode_session_input(Event) -> Option<Input>` boundary, which is also the sole conversion boundary invoked by the real Session loop. It sends those converted inputs through `Runtime::handle` with competing plain and Alt bindings. The test covers Escape followed by lowercase `x`, Unicode `é`, shifted uppercase `X`, explicit `Alt+x`, plus Esc, resize, and paste events. A separate deliberate mutation experiment restores the legacy promotion inside this shared boundary and must fail the unchanged regression; that result is mutation evidence, not an unmodified-old-code RED claim. The exact mutation patch, hash, and run receipt are recorded below after completion.

## Remaining decoder gap

The pinned Crossterm byte parser can consume the second Escape in the legacy coalesced byte stream `ESC ESC x`; after decoding, the resulting bare `x` is indistinguishable from ordinary input. This change does not claim legacy Alt+x parity for that stream. Do not add a time-based inference, a second stdin parser, vendor or patch Crossterm, or enable CSI-u by default. Keep the optional keyboard negotiation off by default and preserve its existing terminal cleanup. Record this case as unresolved pending a supported decoder/reference decision.

The automated test proves conversion and runtime dispatch, not the PTY parser's byte-level behavior. No Tuiscotti PTY trace was run for this bounded increment: the available canonical visual runner in `crates/termrock-conformance/tests/visual_baseline/main.rs` internally requires Tuiscotti `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274` and historical oracle `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, while this task fixes Tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad` and baseline `4169783ae883788186d161af09ba9c481bd84e5c`. Its cases are also ignored capture jobs, not an already admitted paired byte-to-action trace. E2E evidence remains missing and is not green.

## Scope and verification

Only Session input conversion and its focused test changed. No application action mapping changed. The global all-application Alt+x scenario remains a separate review item.

The deliberate mutation restored the old Escape timing promotion in `decode_session_input`, the exact private boundary shared by the production loop and regression. The unchanged regression failed as required (Nextest run ID `710822bf-aad9-4383-be42-73f35b8324f9`; 1 failed, 20 filtered), dispatching only two `ALT_X` actions instead of the four expected distinct bindings. The mutation patch is retained at `docs/audits/evidence/session-alt-promotion-mutation.patch`, SHA-256 `7503872344170d7407434b86b083f44f153115b9b56e112780c1c81f76556f18`; it is applied atop the final source/test edits against WIP parent `51fe8afbd347a4ac543e4c12320507cfe6c9799d`, and intentionally includes the boundary/test changes needed to run this mutation experiment. The mutation was then reverted.

Independent review of exact commit `77e93e43c318c11c95479ecbe0ed7cf09da0099a` is CLEAR with no findings; it closes the prior P2 reachability concern. The reviewer independently restored the old promotion inside the canonical boundary using thread-local previous-Escape state while preserving the regression test byte-for-byte, and observed the full runtime-dispatch test fail. This is controlled mutation evidence, not a historical unmodified-code RED run. Review packet: `docs/audits/evidence/session-77e93e4-independent-review.json`, SHA-256 `dc0e1403d494b1134d8e5fae36862795bb98b17a1aa16b6d1ae14f52db047c3f`.

On corrected source at WIP commit `26255e048cca777de1a40d3bf7accacabffc69bb`, after a normal merge of PR #38 remote `2c58386344d73b4ac3552498f776ba428603f8c8`, `cargo nextest run -p termrock-session --all-targets --no-fail-fast` passed 21 tests with one existing ignored test (run ID `cc1a2664-8285-4a59-b84c-ac266e7a7efa`); strict `cargo clippy -p termrock-session --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` passed. The same Session gates passed immediately before the merge at commit `77e93e43c318c11c95479ecbe0ed7cf09da0099a` (run ID `6a336e32-e8a4-44eb-8b83-5e19eaf4f289`). Tested source SHA-256: `25d64b039c1d799f7df7a74221c31e7776232e874dcd70697b66cfa95251c5a1`; nextest binary SHA-256: `926f53a8e41edc9e863403c124f3a006e87f85e4bfa26e611af491f44630a810`. All seven held expectation/assertion files remain byte-identical to PR #38 commit `52e1572caaf7d9effd0265bbfdf6edcd0e57533b`, with hashes listed above. WIP publication succeeded and remote verification matched `26255e048cca777de1a40d3bf7accacabffc69bb`. Latest PR #38 merge included the current candidate tools/docs commit `2c58386344d73b4ac3552498f776ba428603f8c8`; its other incoming changes are preserved without implying independent approval. The previously tested source-synced tree was `2a8301d2b0aadcd0326bed6603be01aa80c18075`.

All seven held candidate expectation/assertion files remained byte-identical to PR #38 commit `52e1572caaf7d9effd0265bbfdf6edcd0e57533b` after merge. Their SHA-256 values are: `crates/showcase-ui/tests/baselines/showcase.txt` `41f705abaabafc3e9630c64b8e41fa1e965d659159b45b026b7934bd3c32346a`; `crates/holla-ui/tests/baselines/app.txt` `80893794601ddd758e18da668c7a772b541d0c971bc5ecf79778fa63c0e506e1`; `crates/showcase-ui/tests/fixtures/holla-page-headings.tsv` `faa2a62d72736bc6ead7b99135a7d6e3f1ff1193b8bf48dc5f424241e3ccf0c9`; `crates/showcase-ui/tests/fixtures/holla-shell-headers.tsv` `0c049901918b25d610dca3b3d4a693cd2621af2a6d673056159bb16988fc098b`; `crates/jackin-preview-app/tests/baselines/jackin.txt` `326308bbca17a9ed77df2f6b9e3faa9036bead2fdc58c30b2ed624f81be3cedc`; `crates/showcase-ui/tests/page_headings.rs` `9d1646c9500d7e6cf143e5f9873451308fcf814e8ec41efa7667b4af723f6d16`; and `crates/tablepro-ui/tests/baselines/tablepro.txt` `eaddd6b3099a1af851641d156f5aa754a5d2a85fb859bbcc92350092eb420ecb`.

