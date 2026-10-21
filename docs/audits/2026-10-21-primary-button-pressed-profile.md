# Primary button pressed color profile

Frozen reference: commit `4169783ae883788186d161af09ba9c481bd84e5c`.

The frozen primary-button pressed background comes from the accent ladder used by the rendered button: true color `#2e9a38` (`GREEN_60`, `src/theme.rs:516, 790`), ANSI 256 index `65` (`src/theme.rs:945`), ANSI 16 `LightGreen`, and mono `White` (`src/theme.rs:1046, 1082`). ANSI 16's separate `accent_pressed` scalar is `Green` (`src/theme.rs:1014`); it is not the rendered button's ladder value. Candidate Junie's generic Primary `AccentPressed` recipe rendered `#2b8632` in true color, index `238` in ANSI 256, and `Green` in ANSI 16.

Correction: define only the Junie `Family::BUTTON`, `Variant::PRIMARY`, `Part::CONTAINER`, `PRESSED` background at each color level. `define_variant_for_level` keeps the profile below global, scoped, and instance overrides; its source recipe and token checks prevent stale profile application after consumer customization. Other families, variants, channels, quantization, and the global 73-color expectation remain unchanged.

The Controls integration test uses `termrock_test_support::Harness`, backed by Ratatui `TestBackend`; it presses a rendered Primary button at all four color levels and checks its cell background. This is component-level cell evidence, not a PTY or application CLI capture. Frozen button `shortcut` metadata does not paint inline text or create tooltip text without a tooltip body, so this change adds no Button hint API or visible label text.

Independent review of `97e339521322d1e85756e09327d2285469051eed` is recorded at `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/button-primary-pressed-independent-97e3395-20261021/review.json` (SHA-256 `8578a92ded28b63e5156f5e42a85e28cc35bbe8f3e9e5fe346dded55b49d8569`). Review added an external component consumer probe across 16 pressed combinations and checked 956 unchanged frames / 19,120 unchanged cells. It did not run paired application CLI or Tuiscotti captures; those remain missing.

Checks and review status are recorded in the commit and migration matrix after validation. Application-level Showcase adoption and full frozen product parity remain separate, incomplete requirements.
