# Shared startup input behavior

The frozen `visual-baseline` source commit is `4169783ae883788186d161af09ba9c481bd84e5c`; its tag object is `2718a6b8127a522a3d351d0b4bdb7542fc8da542`. It pins tuiscotti at `a1ea27f757d75d07b05499276cd932e9f8e067ad`.

Frozen shell startup waits for `JUNIE_TUI_START_DELAY_MS` when it parses as an unsigned millisecond count, discards pending terminal events, then starts the runtime. The current `termrock-session` path entered the session and runtime without that delay or drain. The frozen CLI control passed all nine selected cases; the candidate control at source commit `da911656e168b65cb0e4667138cbe0708ebbcd2d` passed seven and failed both `startup_input` cases. The captured run also included three unrelated TablePro failures; those are outside this focused startup check.

The shared session now applies the same startup order before terminal entry. `StartupInput::default()` reads the legacy environment variable and enables pending-input discard. `StartupInput` builders and `run_with_startup` / `run_with_feedback_clock_and_startup` let callers override delay and discard policy. Delay parse failures remain ignored, and pending-input drain failures remain ignored, matching the frozen shell.

Verification used the real CLI paths through the pinned tuiscotti harness, with `VISUAL_BIN_DIR` set explicitly for each binary set:

| CLI set | Selected checks | Result |
| --- | --- | --- |
| Frozen baseline binaries | 4 terminal modes, 2 startup resizes, 2 startup-input cases, environment isolation | 9 passed |
| Candidate binaries from source commit `078fffeb35a724d8d74c526d0b20756f41ff8498` plus this session change | Same 9 cases | 9 passed |

Nextest run IDs: frozen `af543ea4-b549-4982-8cac-b527059801d9`; candidate `96c5832f-275b-428b-8c87-32b58e86d5f4`.

Candidate binary hashes:

| Binary | SHA-256 |
| --- | --- |
| `showcase` | `82020c5d9e8842ae3706f172cfd2c5c6d315a65ebdfcf91d1bffee44dd0fd7c0` |
| `tablepro` | `9bff0c0719ece731d292bfc3603490bf89dda3cb9aec7b65bdefa4db68dcf7df` |
| `holla` | `ee49a57a2383646d01691f177c53bc6e7a2bb82e2885e2d7f56cf0c1abfa1037` |
| `jackin-preview` | `e5ac09009e8bd920b2ae604225c3ff3bca31d3ba996fdd8e84f3690a8a7b3802` |

Persistent run logs and exit codes are under `/Users/donbeave/Projects/termrock-cli-evidence-20261020/session-startup/`. The frozen CLI binary hashes are in the shared `provenance.json`; the frozen source and tuiscotti identities above are fixed.

Commands:

```sh
CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/independent-library-gates/cargo-target cargo nextest run -p termrock-session --lib
CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/independent-library-gates/cargo-target cargo clippy -p termrock-session --all-targets
cargo fmt --check
CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/session-startup-candidate-target cargo build --locked --bins -p showcase -p tablepro -p holla -p jackin-preview
VISUAL_BIN_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/session-startup-candidate-target/debug CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/session-startup-candidate-target cargo nextest run -p visual-harness --test visual_baseline --no-fail-fast -E 'test(/^modes::/) | test(/^startup_input::/) | test(/^startup_resize::/) | test(=isolation::env_isolation)'
VISUAL_BIN_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/cargo-target/debug CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-cli-evidence-20261020/session-startup-candidate-target cargo nextest run -p visual-harness --test visual_baseline --no-fail-fast -E 'test(/^modes::/) | test(/^startup_input::/) | test(/^startup_resize::/) | test(=isolation::env_isolation)'
```

This gate covers startup mode, resize, environment isolation, and the two registered startup-input cases. It does not claim full application parity. No baseline snapshots or fixtures changed.
