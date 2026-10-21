# Holla NavList header text policy

## Scope

This increment changes only the generic Home section-header rendering in
`crates/holla-ui/src/screens/home.rs`. It does not migrate the separate Finder
presentation selectors, catalogue projections, app chrome, or footer paths.

## Evidence and decision

The pre-change Home path configured `NavList` with `Part::HEADER` padding and
then repainted each visible section title from application code. The existing
public `NavList::header_text_patch` contract at
`crates/termrock-navigation/src/nav_list.rs` resolves a text-only patch after
the header fill. The library already tests that the text patch leaves header
padding unchanged and that a full header slot retains precedence. The frozen
baseline commit `4169783ae883788186d161af09ba9c481bd84e5c` renders Finder
headings with `Heading::new` at a three-cell inset
(`src/bin/holla/screens/finder.rs`, around line 865). That is useful style
context, not a direct source-equivalence proof: the current change is limited
to Home's generic `NavList` header, so Finder's old renderer is not restored.

Keep `SECTION_INDENT`, `HEADER_PAD`, `HEADER_PARTS`, live `HomeRow` data, the
existing row renderer, and NavList state. Configure the existing text policy
with the same `HEADER_TEXT` patch, then remove the duplicate title-position
calculation and postpaint. No Termrock API change or fixed output data was
needed.

## Verification

The focused real Home fixture test covers 79/80/81, 99/100/101, 119/120/121
columns and truecolor, ANSI 256, ANSI 16, and monochrome. It checks the text
indent, uniform unchanged padding style, non-bold header text, stable header
cells after keyboard focus enters the rows, and retained row hit bounds after
navigation. The existing Home tests continue to cover real keyboard and mouse
row activation identity. A second real `App`/`Harness` case opens the MenuBar
layer and checks that the same Home header text and padding remain coherent
while focus moves to `holla.menu`. The current app's menu flow does not dismiss
on Escape, so focus restoration after close is not asserted here; this is a
coverage gap, not a passing result. Independently, the current Home row painter
emits 15 visible `▎` glyphs in the default fixture. This pre-existing one-focus
violation remains untouched by the header-only increment and must be fixed in
a separate row-renderer migration. No claim of single visible focus indicator
is made.

Focused command:

```text
CARGO_TARGET_DIR=/private/tmp/holla-nav-header-target-20261021 rustup run 1.99.1 cargo nextest run --locked -p holla-ui --lib section_headers_use_nav_list_text_policy_across_widths_and_color_levels home_header_text_policy_survives_open_menu_layer
```

Result: 2 passed, 139 filtered. Full Holla UI library command:

```text
CARGO_TARGET_DIR=/private/tmp/holla-nav-header-target-20261021 rustup run 1.99.1 cargo nextest run --locked -p holla-ui --lib --no-fail-fast
```

Result: 140 passed, 1 failed, 0 skipped. The header grid test passes across the documented matrix. The real
MenuBar layer case checks header content/padding and focus-owner transfer, but
not close/restoration because the current MenuBar flow does not dismiss on
Escape. The one full-library failure is pre-existing and unrelated:
`app::tests::menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit`
(`crates/holla-ui/src/app.rs:2577`); it has the same assertion failure on the
pre-edit `623bb46b82b3e1e021f25a33f42a2522177fda64` worktree. The 15-gutter
Home row issue is recorded above and remains open. The fixed frozen revision remains
`4169783ae883788186d161af09ba9c481bd84e5c`; no expected artifact was changed.

## Remaining work

This proves only the generic NavList header ownership change. Full Holla
presentation, functional parity, and reusable composition acceptance remain
incomplete. No baseline approval is claimed.
