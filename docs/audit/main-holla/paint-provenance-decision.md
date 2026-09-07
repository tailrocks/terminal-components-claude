# Follow-up: semantic paint provenance cannot be inferred from Style

Reviewed73a5ee9 immutable. Followup branch codex/theme-paint-provenance contains four additional regression tests only; no broader production edit made pending parent API/ownership decision. Do not integrate73a5ee9 as complete.

## Reproduction

provenance-adversarial-red.log:11 tests,7 pass,4 fail.
1. Ui::style(HOVERED) immediately followed by fill(Style::new().bg(Red)), then dim: raw red incorrectly treated as hoverElevated, not unknown-background fallback. Equal RGB raw background is included to reject equality-based attempted repair.
2. Save resolved Overlay Style; resolve unrelated Canvas style; paint saved style: dim incorrectly uses latest Canvas role, not saved Overlay.
3. Paint identical wide CJK grapheme through paint_cell/string/spans over same hover plane; dim: continuation backgrounds diverge.
4. paint_cell writes partial wide grapheme at one-cell right clip; set_stringn correctly writes nothing.

Root class: Resolved.style is a bare Copy ratatui Style, while Ui::style writes fg/bg to Ui.roles mutable ambient state. Every painter accepts only Style, then mark consumes whichever roles were most recently resolved, irrespective of which Style is painted. with_part merely reborrows Ui; it does not bind provenance to the actual value either. RowUi::style_of and CellUi store bare Style, so these are real production authoring paths. No cache keyed by RGB/Style, pointer identity, last style equality, or immediate-call convention can restore erased identity, especially palette collisions.73a5ee9 solved inherited actual backgrounds and exited scopes but not erased value identity; previous raw sentinel test missed this due preceding LABEL resolution.

## Proposed decision: explicit immutable paint carrier

Introduce Copy PaintStyle containing bound ratatui Style plus per-channel provenance (semantic role AND source Surface, or explicit Raw/Unknown). Plain Style converts to raw PaintStyle without ambient provenance. Theme binding/Ui::style produce a carrier; remove Ui.roles query side effects as authority. All paint entrypoints accept/forward the carrier, and mark receives it, never global last roles. Each glyph's cells get precisely that carrier's metadata. Inherited style channels retain prior per-cell provenance, explicit raw channels clear it. Semantic provenance must be recorded before the owner surface scope exits, and its resolution may never use the later caller's surface.

Recommended public shape: Resolved.style becomes PaintStyle (read-only raw access via as_style/into_style), with supported modifier methods preserving roles, raw fg/bg setters clearing just that channel, semantic patches preserving declared roles, and patch/over merging provenance exactly as Style channels merge. No DerefMut/public mutable raw fields that could silently retain stale roles. Existing public low-level paint calls remain source-compatible for raw Style via Into<PaintStyle>; changing Resolved.style intentionally makes semantic-to-raw erasure explicit at external Ratatui boundaries. Adding only an optional Resolved.paint() while existing .style semantic callers remain is insufficient as final migration: those would silently become raw and lose dim semantics. API review should pick explicit cutover or fully enumerated staged caller migration with no claim of completion before closure.

## Required caller scope

1. Theme Resolved/bind and public exports (author/prelude/lib); Ui style/surface_style/with_part plus all paint functions; remove ambient last-role authority.
2. RowUi/CellUi/ColumnsUi (stored styles, patch, formatting, glyph/metadata), PartStyle wrappers, internal component style helpers and every production component paint path.
3. App custom renderers: showcase/TablePro/Jackin and later Holla public author API consumers; explicit typed/raw boundaries. Tests/docs/examples and architecture allow/disposition if public shape changes.
4. Layer compositing and internal cell moves must transfer metadata with cells. Arbitrary raw buffer mutation explicitly invalidates metadata; CellUi alignment cannot hide behind arbitrary raw if it must preserve semantic provenance.

paint-carrier-candidate-sites.tsv is a textual candidate inventory (.style/.over), not a claim every row requires an edit. The complete migration must be compiler- and caller-audit-driven; this exceeds original theme/ui/test ownership. Parent should split ownership by files before work starts.

## Wide rendering decision

Ratatui-core0.1.2 buffer.rs336–379 uses grapheme walk: filter controls/zero-width, refuse grapheme wider than remaining area, paint lead cell then reset continuation cells. set_span delegates to that path. Current paint_str/paint_spans subsequently mark entire written width with lead Style, resurrecting provenance on reset continuation cells;73a5ee9 paint_cell instead cleared continuation provenance, correctly reflecting reset bytes but creating disagreement. Shared writer must report lead and reset spans or perform one identical grapheme loop with metadata writes; no second inconsistent width parser, no partial grapheme at right clip. Preserve clipping, zero-width/control handling, text-width counts, written-cell bitset, allocation/performance contracts.

## Acceptance

Old seven tests plus four red tests; raw and semantic same-color collisions in all four modes; delayed/interleaved styles from different source surfaces; scoped overrides; inherited fg/bg independently; raw explicit same/different colors; every painter including styled spans/glyphs/fill/style; wide/narrow transitions, right-edge CJK/emoji, zero-width/control, per-cell provenance and layer write masks. Existing component/production snapshots independently reviewed; no blanket blessing. Re-run public external consumer compilation and performance gates after complete cutover.
