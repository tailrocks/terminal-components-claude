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
row activation identity. The real `App`/`Harness` case uses a simulation
feedback clock initialized from `App::fixture_time_ms()`, then covers menu open,
Escape close, and focus restoration at 119/120/121 columns and all four color
profiles. It compares the same header text and padding before, during, and after
the menu layer; focus ownership moves from `home.query` to `holla.menu` and back,
and runtime diagnostics stay empty.

**Superseded finding:** the first version used `Harness::new`'s monotonic
clock for an app with a simulation clock. Its Escape failure was a fixture
mismatch, not a MenuBar defect. The independent review at
`/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/holla-focus-contract-b796-independent-review-20261021/review.json`
(reviewed commit `b7963f5241b584cb239a66724f7a6ac33be9654f`, SHA-256
`c5ca992578c0ba7cfd29c767d3223219623c53d39224d461cb269af6e5867401`) records
both the negative control (monotonic clock reproduces the mismatch) and the
simulation-clock positive control. No MenuBar or runtime behavior was changed.

The review also confirms the current Home row painter emits 15 white `▎`
glyphs in the correctly clocked frame. This genuine pre-existing one-focus
violation remains untouched by the header-only increment and must be fixed in a
separate row-renderer migration. No single-visible-focus claim is made.

Focused command:

```text
CARGO_TARGET_DIR=/private/tmp/holla-nav-header-target-20261021 rustup run 1.99.1 cargo nextest run --locked -p holla-ui --lib section_headers_use_nav_list_text_policy_across_widths_and_color_levels home_header_text_policy_survives_menu_open_close_and_focus_restoration
```

Result: 2 passed, 139 filtered. Full Holla UI library command:

```text
CARGO_TARGET_DIR=/private/tmp/holla-nav-header-target-20261021 rustup run 1.99.1 cargo nextest run --locked -p holla-ui --lib --no-fail-fast
```

Result: 140 passed, 1 failed, 0 skipped. The header grid and full MenuBar lifecycle cases pass across their documented
matrices. The inherited failure
`app::tests::menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit`
(`crates/holla-ui/src/app.rs:2577`) still fails with its existing correctly
clocked helper; its source and expected values were not changed. The 15-gutter
Home row issue is recorded above and remains open. The fixed frozen revision remains
`4169783ae883788186d161af09ba9c481bd84e5c`; no expected artifact was changed.

## Remaining work

This proves only the generic NavList header ownership change. Full Holla
presentation, functional parity, and reusable composition acceptance remain
incomplete. No baseline approval is claimed.
