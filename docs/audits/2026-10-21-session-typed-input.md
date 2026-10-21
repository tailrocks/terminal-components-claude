# Session typed-input preservation — 2026-10-21

## Decision

Remove `promote_escaped_alt` from `crates/termrock-session/src/session.rs` and return to the session boundary's direct `Input::from_crossterm(read()?)` conversion. Crossterm's event conversion preserves the key code and modifiers it reports. Session timing, character case, or a preceding Escape event cannot safely infer a missing modifier.

This follows frozen commit `4169783ae883788186d161af09ba9c481bd84e5c`: `src/runtime.rs::next_ready_input` passes each returned Crossterm event to `Input::from_crossterm_with` and returns that decoded input without a later clock-based modifier rewrite. The independent review of PR #38 candidate `bfefea9ce4b3ab91f846239d5f315534ccc2ff43` concluded removal restores that behavior. Review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/session-alt-review-bfefea9-20261021/review.json`, SHA-256 `945926dd56ee0fa63c23d114e53c2fa0fd1bb6749763c212ad72fcb235550731`.

## Rejected evidence retained

The removed author test at `bfefea9ce` created a synthetic bare Escape `Input`, called the private promotion helper, then created a synthetic bare `x` `Input` and asserted the helper added `ALT`. Its relevant body is retained here as rejected evidence:

```rust
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

The replacement test `converted_crossterm_keys_dispatch_with_only_reported_modifiers` converts Crossterm events and sends the resulting `Input` through `Runtime::handle` with competing plain and Alt bindings. Against the old production promotion path, it failed: the Escape-following plain `x` dispatched as `ALT_X` instead of `X`. With promotion removed, it verifies that Escape followed by lowercase `x`, Unicode `é`, and shifted uppercase `X` preserves the reported input, while explicit `Alt+x` still dispatches its distinct action. It also checks that resize and paste convert as non-key inputs.

## Remaining decoder gap

The pinned Crossterm byte parser can consume the second Escape in the legacy coalesced byte stream `ESC ESC x`; after decoding, the resulting bare `x` is indistinguishable from ordinary input. This change does not claim legacy Alt+x parity for that stream. Do not add a time-based inference, a second stdin parser, vendor or patch Crossterm, or enable CSI-u by default. Keep the optional keyboard negotiation off by default and preserve its existing terminal cleanup. Record this case as unresolved pending a supported decoder/reference decision.

The automated test proves conversion and runtime dispatch, not the PTY parser's byte-level behavior. No pinned Tuiscotti PTY trace was run for this bounded increment; E2E evidence remains missing and is not green.

## Scope and verification

Only Session input conversion and its focused test changed. No application action mapping changed. The global all-application Alt+x scenario remains a separate review item.

Before removal, `CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-targets/session-typed-input-20261021 rustup run 1.99.1 cargo nextest run -p termrock-session --lib converted_crossterm_keys_dispatch_with_only_reported_modifiers --no-fail-fast` failed as intended (exit 100), with Nextest run ID `65fa885d-5f9d-409c-a66a-d7d4cfbf8943`: expected the plain `X` action, received `ALT_X` after bare Escape. After removal and merge of PR #38 source `40bc297e5f1b18f58f1466d5aaa8d80f19c051a2`, `cargo nextest run -p termrock-session --all-targets --no-fail-fast` passed 21 tests with one pre-existing ignored test (run ID `2336abb0-619f-4238-a23b-0085ab178716`); strict `cargo clippy -p termrock-session --all-targets -- -D warnings` passed; `cargo fmt --all --check` and `git diff --check` passed. The WIP source commit is `7150ed36bc5c972dd2e074dcc1b689a9fa1fa615`; post-merge tree is `f36d2069e6a10dad92218f353c9d005dec9bf350`. Full PTY decoder behavior remains unverified as noted above.

