# Primary button pressed color profile

Frozen reference: commit `4169783ae883788186d161af09ba9c481bd84e5c`.

The baseline `Theme::junie` token uses `GREEN_60` (`#2e9a38`) for primary button pressed fill in true color. Its capability palettes set `accent_pressed` to ANSI 256 index `65`, ANSI 16 `LightGreen`, and mono `White` (`src/theme.rs:516, 945, 1012, 1082`). The baseline primary button renderer uses this pressed token. Candidate Junie used the generic Primary `AccentPressed` recipe, yielding `#2b8632` and indexed `238`; actual Button rendering failed the added PTY-backed component test before the fix.

Correction: define only the Junie `Family::BUTTON`, `Variant::PRIMARY`, `Part::CONTAINER`, `PRESSED` background at each color level. `define_variant_for_level` keeps the profile below global, scoped, and instance overrides; its source recipe and token checks prevent stale profile application after consumer customization. Other families, variants, channels, quantization, and the global 73-color expectation remain unchanged.

The real Controls integration test presses a rendered Primary button at all four color levels and checks its cell background. Frozen button `shortcut` metadata does not paint inline text or create tooltip text without a tooltip body, so this change adds no Button hint API or visible label text.

Checks and review status are recorded in the commit and migration matrix after validation. Application-level Showcase adoption and full frozen product parity remain separate, incomplete requirements.
