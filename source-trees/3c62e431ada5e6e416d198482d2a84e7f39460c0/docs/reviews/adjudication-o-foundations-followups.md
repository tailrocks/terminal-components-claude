# Adjudication O — Slice 3 foundations follow-ups

**Source.** Four research requests returned by the Slice 3 foundations correction pass (F1–F26 + Adjudication N) at `HEAD 7899678`. Each is a conflict between two already-accepted statements. Nothing here reopens Adjudications A–N; each item decides only *which side moves*.

**Method.** Facts are `path:line`. Arithmetic marked **[derived]** was recomputed in this pass from the checked-in token values; the review's `(estimate)` figures are corrected where they were wrong.

---

## O1. Style memo associativity — **accept 2-way; amend §20.9-2 and the stale module doc; keep ≥ 90 % but re-purpose it**

### Facts

* The implementation is 2-way set-associative: `CACHE_SLOTS = 256` (`crates/tui/src/theme/resolve.rs:265`), `WAYS = 2`, `CACHE_SETS = 128` (`:280-281`), insert-at-way-0 with a 1-entry shift (`:368-379`), `promote` on a way-1 hit (`:385-401`), one construction-time `Box::new([...; 256])` (`:304`), generation-stamp clear (`:312-314`).
* **§11.1 A3 never says "direct-mapped."** `COMPONENT_ARCHITECTURE.md:786` says only *"a small per-frame array cache (≤ 256 entries, cleared each frame)"*. The word appears in exactly two places: **§20.9-2** (`:3821`, *"A `[Option<(u64, Resolved)>; 256]` direct-mapped array embedded in `Ui`"*) and the **module doc of the file that contradicts it 273 lines later** — `resolve.rs:7` still reads *"memoised in a statically sized direct-mapped cache"*. That is a live doc/code contradiction inside one shipped file.
* §20.9-2's sketch is wrong in three further ways already: there is no `Option` (the sentinel is `key | 1`, `:347-349`), the value is a `StylePatch` not a `Resolved` (its own prose already corrects this), and the cache lives in `self.core.style_cache` behind a `Box`, not "embedded in `Ui`" by value (`runtime.rs:1080`).
* The threshold: `rate >= 0.90` (`crates/tui/tests/perf.rs:193-196`), read from `Runtime::style_cache_stats()` (`runtime.rs:1079-1081`).
* Benchmark key set: `Family::LIST × Variant::DEFAULT × {CONTAINER, GUTTER, MARKER, META} × 8 states` = **32 distinct keys**, 10 000 queries per draw, cache cleared per draw (`tests/perf.rs:118-149`).

### Verification of the arithmetic **[derived]**

**Collision count.** 32 keys into 256 direct-mapped slots: expected colliding pairs `C(32,2)/256 = 496/256 = 1.94`. The builder's figure is correct.

**Hit rate is the load-bearing number, and it also checks out.** Balls-in-bins: expected keys landing alone `= 32·(255/256)^31 = 28.35`, so `≈ 3.65` keys sit in multi-key slots. Access order is round-robin over all 32 keys (`STATES[i % 8]` × 4 parts), so **every** key in a shared slot misses on **every** access — LRU-hostile by construction. Predicted rate `≈ (10 000 − 3.65·312.5 − 28)/10 000 = 88.3 %`. Measured 87.2 %. Consistent.

**No hash fixes it.** The 88 % figure is a property of *any* uniform hash: it is the birthday load of 32 keys over 256 buckets, not of FNV. Only a perfect hash over a statically known key set would help, and the key set is not statically known. **Confirmed.**

**2-way.** A miss needs ≥ 3 keys in one set. Poisson, λ = 32/128 = 0.25: `P(set ≥ 3) = 0.00216`, ×128 sets = **0.28 expected sets**, matching the builder's `C(32,3)/128² ≈ 0.3`. Expected rate ≈ 99.7 %.

### The finding the builder did not make: the benchmark **understates** the problem

A real frame does not resolve 32 keys. §16.6's `frame_showcase_lists_120x40` covers a sidebar list, buttons, tabs, panels, a status bar — order 100–300 distinct `(family, variant, part, flags)` tuples. At 200 keys, a 256-slot direct-mapped table has load factor 0.78 and thrashes catastrophically; the 87.2 % is a *synthetic best case*, not a floor. So the case for 2-way is stronger than the benchmark shows, and the ≥ 90 % bar is a **lower** bound on a favourable distribution — which is precisely why "change the benchmark's key distribution" is the wrong lever (below).

### Decision

**Accept 2-way. Amend §20.9-2 and `resolve.rs:7`. Keep ≥ 90 % as the perf-gate floor, but restate in §16.6 what it is for, because it is a statistical property, not a guarantee, and the document currently reads as though it were one.**

Rationale: every invariant §20.9-2 actually asserts survives 2-way — statically sized, one construction-time allocation, no per-frame allocation, no growth, no `HashMap`, no `Vec`, generation-stamp clearing. The single clause 2-way contradicts is *"there is no eviction policy to get wrong"*, and that must be amended honestly rather than glossed. "Direct-mapped" is a sketch detail of the same class as `[Option<(u64, Resolved)>; 256]` — which §25/§26 already corrected in prose — not an invariant.

### Exact changes

**Code (three, all small).**

1. `crates/tui/src/theme/resolve.rs:7` — replace *"statically sized direct-mapped cache"* with *"statically sized two-way set-associative cache (256 entries in 128 sets)"*. A file whose header contradicts its own `WAYS` const is a landmine.
2. `crates/tui/src/theme/resolve.rs:312-314` — **generation wrap is a latent correctness bug.** `self.generation.wrapping_add(1).max(1)` returns to 1 after 2³² clears, at which point a slot stamped with the original generation 1 becomes a false hit returning a stale `StylePatch`. Fix:
   ```rust
   pub(crate) fn clear(&mut self) {
       if self.generation == u32::MAX {
           self.slots.fill((0, 0, StylePatch::new()));
           self.generation = 1;
       } else {
           self.generation = self.generation.saturating_add(1);
       }
   }
   ```
   Cost: one comparison per frame; the 256-entry fill runs once per 2³² frames.
3. `crates/tui/tests/perf.rs:192` — the `PERF-CACHE` line already prints `hits/misses/rate`; add the measured 2-way rate to `crates/tui/tests/perf_baseline.txt`'s header block, beside the F18 notes, so the number is reviewed in a diff rather than living only in CI scrollback.

**Document — §20.9-2, replace the first sentence of the Amendment column (`COMPONENT_ARCHITECTURE.md:3821`):**

> **The §11.1 A3 memo cache is allocation-free and statically sized.** A `[(u64, u32, StylePatch); 256]` behind one `Box`, owned by the runtime's frame core and reused across frames, keyed by a 64-bit mix of `(Family, Variant, Part, StateFlags, overlay_stack_hash)` with the low bit forced so `0` stays the empty sentinel, and cleared by a generation stamp rather than by zeroing. <!-- amended by §27 (Adjudication O1) --> The 256 entries are grouped into **128 two-way sets**, insert-at-most-recent. ~~direct-mapped~~ ~~there is no eviction policy to get wrong~~ are **struck**: a direct-mapped table of 256 entries cannot meet §16.6's ≥ 90 % hit rate for *any* realistic key set, because with `k` hot keys the expected number of colliding pairs is `C(k,2)/256` and a colliding pair in a round-robin loop misses on **every** access. `style_resolve_10k_parts` touches 32 keys (4 parts × 8 states); ≈ 1.94 pairs collide, ≈ 3.65 keys thrash, and the measured rate is **87.2 %** whatever the hash — this is the birthday load of 32 keys over 256 buckets, not a property of FNV, so no re-hashing recovers it. A realistic frame resolves 100–300 distinct tuples, so 87.2 % is a synthetic *best* case, not a floor. Two ways make a miss require three keys in one set (`C(32,3)/128² ≈ 0.28` expected sets), which is what makes the memo's health assertable. The array shape, the single construction-time allocation, the absence of any per-frame allocation or growth, and the generation stamp are **unchanged**; the eviction policy is exactly "shift way 0 into way 1, insert at way 0, promote a way-1 hit", bounded at two entries and independent of the key count.

**Document — §16.6, `style_resolve_10k_parts` row (`:2041`) and §25.8's table row (`:5327`), append:**

> The ≥ 90 % figure is a **key-correctness floor**, not a performance bound: a broken key drops it to ≈ 0.3 %, a direct-mapped table to ≈ 87 %, and the shipped two-way memo measures ≈ 99.7 %. At 256 entries no associativity makes a hit rate a *guarantee* — with 32 keys over 128 two-way sets, a hash configuration that puts three keys in one set yields ≈ 90.3 % and two such sets ≈ 81 %, i.e. a ≈ 5 % chance of a sub-90 % rate under an unrelated renumbering of `Part`/`Family`/`Variant` or a change to `fnv1a`. The **deterministic** guarantee is `theme::cache_hits_after_the_first_query_and_clears_by_generation` (`resolve.rs:685-727`, `stats() == (1, 1)` after two identical queries, `(1, 2)` after a `clear()`), which is hash-independent and is the assertion that actually proves the mechanism. Should the perf floor ever trip, diagnose the cache geometry from the `PERF-CACHE` line before suspecting the key.

**§11.1 A3 (`:786`) needs no change** — it never claimed direct mapping. Only its cross-reference is affected, and it already reads correctly.

### Rejected alternatives

* **Revert to direct-mapped and lower the threshold below 87 %.** Rejected: it re-admits a memo that misses ≈ 12 % on a 32-key set and far worse on a real frame, for no saving — 2-way costs the same 256 entries, the same one allocation, and one extra tag compare on a miss. Lowering the bar to accommodate a worse mechanism inverts the purpose of the gate.
* **Change the benchmark's key distribution.** Rejected on the evidence: a realistic frame has *more* distinct keys, so a "realistic" distribution makes the rate worse and the assertion more fragile, not less. The benchmark is not unrepresentative in the direction the request assumed.
* **`WAYS = 4` (64 sets).** Would make ≥ 90 % robust under any hash (expected sets with ≥ 5 of 32 keys ≈ 0.014). Rejected: a 4-way tag scan touches four ~64-byte entries — four cache lines — on the **hit** path, which is the 12 ns budget §25.8 accepted. The robustness it buys is already provided deterministically by the unit test. Revisit only if tags are split into their own array.
* **Drop `promote` (insert-at-way-0, never promote on hit).** Provably equivalent for ≤ 2 keys per set, which is 99.7 % of sets, and it removes the only *write* from the hit path. Not adopted: the difference is unmeasurable, the code is landed and gated green, and MRU is the standard, least-surprising policy. Recorded so a later reader does not re-derive it.

### Allocation and worst-case-latency consequences (as requested)

| | Direct-mapped (documented) | 2-way (shipped) |
|---|---|---|
| Allocations | one `Box`, construction time | **identical** — `resolve.rs:304`, `style_resolve_10k_parts` allocs `= 0` (`perf_baseline.txt:50`) |
| Memory | 256 entries | **identical** (`CACHE_SLOTS = 256`, `:265`) |
| Per-frame cost of `clear` | one `u32` bump | **identical** (`:312-314`) |
| Hit, way 0 | 1 tag+gen compare, no write | **identical** |
| Hit, way 1 | — | 2 compares + `promote` copies 2 entries ≈ 128 B (`:385-401`) — the only new hit-path cost, and only for keys sharing a set |
| Miss | 1 compare + `accumulate` + 1 write | 2 compares + `accumulate` + a 1-entry shift + 1 write (`:352-379`) |
| Measured | 87.2 % hit | 12.0 ns/query (`120141 ns / 10 000`, `perf_baseline.txt:50`), inside §25.8's accepted ≈ 13 ns |

Every path is O(1) and independent of the key count. No new worst case.

### Test

```
cargo test -p tui-next --lib theme::resolve::tests::cache_hits_after_the_first_query_and_clears_by_generation
cargo test -p tui-next --lib theme::resolve::tests::cache_generation_wrap_does_not_serve_a_stale_entry   # new, F-O1
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 style_resolve_10k_parts
! rg -n 'direct-mapped' crates/tui/src COMPONENT_ARCHITECTURE.md
```

New test `theme::cache_generation_wrap_does_not_serve_a_stale_entry`: seed a key at generation 1, set `generation = u32::MAX`, `clear()`, assert the same key **misses**.

---

## O2. `borders_set(border::ASCII)` glyph rebinding — **coupling CONFIRMED, four glyphs CONFIRMED; mechanism amended; full table scheduled for 4E, not deferred**

### Facts

* `Ui::rule` reads `GlyphRole::RuleQuiet` from `theme.design.glyphs`, never from `design.borders` (`crates/tui/src/ui/paint.rs:148-168`). `Ui::frame` reads `design.borders` (`:171-199`). The two are independent stores. **Coupling confirmed at the source.**
* `GlyphSet::get`/`set` route `RuleQuiet`/`RuleActive` to `line::Set.horizontal` and `ScrollTrack`/`ScrollThumb` to `scrollbar::Set.track`/`.thumb` (`theme/glyph.rs:177-200`), so `borders_set(ASCII)`'s four `g.set(..)` calls (`theme/builder.rs:213-219`) do hit the live path.
* Junie binds them to `line::NORMAL` (`─`), `line::THICK` (`━`), and `scrollbar::Set { track: "│", thumb: "┃", begin: "│", end: "│" }` (`theme/builtin/junie.rs:133-141`).
* `theme::ascii_theme_renders_without_box_drawing_glyphs` paints a frame **and a `ui.rule(..)`** and scans the whole frame for `U+2500..=U+257F` (`crates/tui/tests/render.rs:493-541`). Without the rebinding it fails on the `─`. The test also asserts the Junie control renders and that plain Junie *does* emit box drawing, so it is not vacuous.
* **The four are not arbitrary [derived].** Auditing every entry of Junie's 39-glyph table (`junie.rs:129-131`) plus the typed sets: `▎ U+258E`, `› U+203A`, `✓ U+2713`, `● U+25CF`, `• U+2022`, `− U+2212`, `▲ U+25B2`, `▸ U+25B8`, `▾ U+25BE`, `▴ U+25B4`, `∇ U+2207`, `▪ U+25AA`, `→ U+2192`, `↓ U+2193`, `‹ U+2039`, `… U+2026`, `× U+00D7`, `◆ U+25C6`, `◇ U+25C7`, `∥ U+2225`. **None** falls in `U+2500..=U+257F`. `RuleQuiet`, `RuleActive`, `ScrollTrack`, `ScrollThumb` are *exactly* the roles whose Junie binding lies in the box-drawing block — which is exactly the scan range of the named test. The coupling is complete with respect to a statable principle, not to whatever the test happened to catch.

### Visual-design judgement on the four values (against `DESIGN.md:552-563`)

`DESIGN.md:555-556` — *`─` is a quiet rule*; `:557-559` — *`━` is an active rule* (tab underline, filled progress); `:560` — *`│`/`┃` are the scrollbar track and thumb*; `:491` — *"A one-column scrollbar (`│` track, `┃` thumb)"*.

| Role | Junie | ASCII | Verdict |
|---|---|---|---|
| `RuleQuiet` | `─` | `-` | The direct ASCII equivalent, one stroke, width 1. ✔ |
| `RuleActive` | `━` | `=` | Must read **heavier** than quiet. `=` is two strokes against one — the conventional ASCII heavy rule, and it survives monochrome, where weight is the only channel left (`DESIGN.md:321-322`). ✔ `#` was the alternative and reads as hatch/fill, not a rule; rejected. |
| `ScrollTrack` | `│` | `\|` | Direct equivalent, width 1. ✔ |
| `ScrollThumb` | `┃` | `#` | Must read **denser** than the track in monochrome. `\|` (one stroke) → `#` (crosshatch) is an unambiguous density step and is the conventional ASCII thumb. ✔ `H`, `*`, `+` were considered: `+` collides with `border::ASCII`'s corners, `*` reads as a marker, `H` as a letter. |

All four are ASCII, one byte, width 1 — the same property `theme::ascii_border_set_is_pure_ascii` pins for the border set (`theme/border.rs:31-46`).

### Three residual gaps the coupling does **not** close

1. **`scrollbar::Set.begin` / `.end` stay `│`** (`junie.rs:136-137`). `GlyphSet::set` has **no role** that reaches them (`glyph.rs:192-200`), so nothing — not even a manual `.glyph(..)` — can make them ASCII. Invisible today only because no component paints scrollbar caps; `ScrollRegion` is 4E's file. This will silently break `ascii_theme_renders_without_box_drawing_glyphs` the day 4E lands.
2. **`line::Set`'s other ten fields stay box-drawing** — `vertical` `│`, `cross` `┼`, four corners, four tees. `GlyphSet::set` rewrites only `.horizontal`. §22.2 item 12 designates `line::Set` for *"rules and seams"*; the first seam painter leaks.
3. **The swap is sticky and order-dependent.** `borders_set(ASCII).borders_set(PLAIN)` keeps ASCII rules; `.glyph(RuleQuiet, "~").borders_set(ASCII)` **silently discards** the author's explicit glyph — unlike every colour setter, which honours `Explicit` (`builder.rs:25-41`, `:63-81`, `:130-134`, `:177-179`). The prose warns about order (`builder.rs:209`) but the type does not.
4. Array slots 29–32 of `GLYPHS` (`"─","━","│","┃"`, `junie.rs:130`) are **shadowed dead data** — `GlyphSet::get`/`set` route those four roles to the typed sets and never index the array. Harmless, misleading, and it will mislead the author of the ASCII table.

### Decision — (c), a middle design

**(a) is confirmed on the substance: the coupling is necessary and the four glyphs are right. (b) is rejected as stated. The mechanism moves, and the full table is scheduled.**

Three parts:

1. **Confirm the coupling and the four values.** §24 M2's own rejection of automatic ASCII selection rests on reason (2) — *"a border-only auto-switch renders a frame that is ASCII at the edges and unicode everywhere else, worse than either consistent choice"* (`:5031`). That argument applies with identical force to a `borders_set(ASCII)` that leaves `─` in every divider. A border-only `borders_set(ASCII)` is the outcome M2 called worse than either consistent choice.
2. **Move the swap out of `borders_set`'s implicit body into a named, reusable, whole-set step**, and widen it to replace the *typed sets*, not four fields — which closes gaps 1 and 2 in the same edit and needs no new `GlyphRole` (so no §11.2 widening).
3. **Reject narrowing the test's scope.** The whole-frame `U+2500..=U+257F` scan is the only thing that made the coupling visible; narrowing it would restore the invisibility.

### Exact changes

**Code — `crates/tui/src/theme/glyph.rs`,** three whole-set mutators beside the per-role ones:

```rust
/// Replace the typed scrollbar set (`ScrollTrack`/`ScrollThumb` read from it,
/// and `begin`/`end`, which no `GlyphRole` names).
pub const fn set_scrollbar(&mut self, s: scrollbar::Set<'static>) { self.scroll = s; }
/// Replace the typed quiet-rule line set (`RuleQuiet` reads `.horizontal`;
/// seams read the rest — §22.2 item 12).
pub const fn set_rule_quiet(&mut self, s: line::Set<'static>) { self.rule_quiet = s; }
/// Replace the typed active-rule line set.
pub const fn set_rule_active(&mut self, s: line::Set<'static>) { self.rule_active = s; }
```

**Code — `crates/tui/src/theme/glyph.rs`,** the ASCII counterparts of the typed sets, beside `border::ASCII`:

```rust
/// ASCII scrollbar: `|` track and caps, `#` thumb — the thumb must read
/// denser than its track in monochrome (`DESIGN.md:491`, `:560`).
pub const ASCII_SCROLLBAR: scrollbar::Set<'static> =
    scrollbar::Set { track: "|", thumb: "#", begin: "|", end: "|" };
/// ASCII quiet rule (`-`) and its seams (`|`, `+`) — `DESIGN.md:555`.
pub const ASCII_RULE_QUIET: line::Set<'static> = /* horizontal "-", vertical "|", every junction "+" */;
/// ASCII active rule (`=`): heavier than the quiet rule — `DESIGN.md:557`.
pub const ASCII_RULE_ACTIVE: line::Set<'static> = /* horizontal "=", vertical "|", every junction "+" */;
```

**Code — `crates/tui/src/theme/builder.rs:202-221`,** replace the implicit four-`set` block:

```rust
/// Rebind every glyph whose default falls in the box-drawing block
/// (`U+2500..=U+257F`) to its ASCII equivalent: the quiet rule, the active
/// rule and the scrollbar track, thumb and caps. Idempotent. Call `.glyph(..)`
/// **after** this to override any of them (§24 M2, Adjudication O2).
#[must_use]
pub fn ascii_glyphs(mut self) -> Self {
    let g = &mut self.theme.design.glyphs;
    g.set_rule_quiet(glyph::ASCII_RULE_QUIET);
    g.set_rule_active(glyph::ASCII_RULE_ACTIVE);
    g.set_scrollbar(glyph::ASCII_SCROLLBAR);
    self
}

#[must_use]
pub fn borders_set(mut self, b: BorderSet) -> Self {
    self.theme.design.borders = b;
    if b == crate::theme::border::ASCII { self = self.ascii_glyphs(); }
    self
}
```

Ordering stays "last write wins" and is now documented on a method that *says* it changes glyphs. The stickiness of `borders_set(ASCII).borders_set(PLAIN)` is accepted and documented: the author asked for two things in sequence.

**Code — `crates/tui/src/theme/builtin/junie.rs:130`,** replace the four shadowed array entries at indices 29–32 with a comment naming them dead, or with `""`, so the ASCII-table author is not misled into thinking they are live.

**Document — §11.2 (`:861-863`) and §24.2 (`:5009`, `:5033`), add:**

> <!-- amended by §27 (Adjudication O2) --> `ThemeBuilder::borders_set(border::ASCII)` also applies `ThemeBuilder::ascii_glyphs()`, which rebinds the typed `line` and `scrollbar` sets. This is not a widening of §24 M2's scope: `RuleQuiet`, `RuleActive`, `ScrollTrack` and `ScrollThumb` are **exactly** the `GlyphRole`s whose Junie binding falls in `U+2500..=U+257F`, the block `theme::ascii_theme_renders_without_box_drawing_glyphs` scans — verified role by role against `theme/builtin/junie.rs:129-141`. A `borders_set(ASCII)` that left `─` in every divider would produce precisely the outcome §24 M2 rejected automatic selection for ("ASCII at the edges and unicode everywhere else, worse than either consistent choice"). The swap replaces the **whole typed sets**, not four fields, so `scrollbar::Set.begin`/`.end` and `line::Set`'s seam junctions — which no `GlyphRole` names — are covered too. The remaining ~31 roles stay unicode; that is §24 M2 risk 3, and its scheduling is below.

**Document — §24 M2 "Deferred root cause, named" (`:5033`), replace the closing sentence:**

> <!-- amended by §27 --> The full `GlyphSet` ASCII fallback table is **scheduled for Slice 4E, not deferred indefinitely.** 4E ships `ScrollRegion`, the first component to paint `scrollbar::Set.begin`/`.end`; until Adjudication O2's whole-set swap those two glyphs were unreachable by any `GlyphRole` and would have made `theme::ascii_theme_renders_without_box_drawing_glyphs` fail the day 4E landed. `ascii_glyphs()` covers the box-drawing block completely; the remaining ~31 roles (`›`, `✓`, `▎`, `…`, `×`, `▸`, the spinner frames) are a **visual-design** decision against `DESIGN.md`'s marker table and belong with 4E's own review, together with re-blessed baselines under §20.10. `Capability` still gains no unicode axis (§21 item 19); the table stays a manual, theme-author-visible opt-in.

### Rejected alternatives

* **(b) `ascii_glyphs()` as a *separate* step and narrow the test to border cells.** Rejected on two grounds. First, narrowing the scan destroys the only mechanism that surfaced the coupling — the test would then pass on a frame that is ASCII at the edges and box-drawing everywhere else, which is what §24 M2 declared worse than either consistent choice. Second, it makes correctness depend on the author remembering a second call, with a silent, visible-only-at-runtime failure mode. `ascii_glyphs()` is kept as a *public, named* step; what is rejected is making it the only path.
* **Add `GlyphRole::ScrollBegin` / `ScrollEnd`.** Would reach `begin`/`end` through the per-role API, but widens `GlyphRole::ALL` from 39 to 41, touches every `GlyphSet` literal in `junie.rs` and `paper.rs`, and is a §11.2 amendment — all to solve by enumeration what a whole-set setter solves structurally. The same argument holds tenfold for `line::Set`'s eleven fields.
* **Schedule the full table now (Slice 3).** Rejected: ~31 glyph choices against `DESIGN.md`'s marker table plus re-blessed baselines is a fresh visual adjudication, and §24 M2 already ruled it so. Rejected equally is leaving it open-ended: 4E is a dated forcing function, an indefinite deferral is not.

### Risks

1. `ascii_glyphs()` still leaves ~31 unicode glyph roles; an author may read "ASCII theme" as a full guarantee. Mitigated by the rustdoc on `ascii_glyphs()` and by §24 M2 risk 3, both of which say the scan is the box-drawing block only.
2. `borders_set(ASCII).borders_set(PLAIN)` keeps ASCII rules. Documented, not fixed — restoring the theme's own glyphs would clobber a deliberate `.glyph(..)`.
3. The new `theme::ascii_glyph_set_has_no_box_drawing` (below) **fails today** on `scroll.begin`/`.end`. That is the intended effect: it converts a latent 4E failure into a Slice-3 one.

### Test

```
cargo test -p tui-next --test render theme::ascii_theme_renders_without_box_drawing_glyphs
cargo test -p tui-next --lib theme::glyph::tests::ascii_glyph_set_has_no_box_drawing      # new, F-O2
cargo test -p tui-next --lib theme::builder::tests::ascii_glyphs_is_idempotent_and_glyph_overrides_it   # new
cargo test -p tui-next --lib theme::border::tests::ascii_border_set_is_pure_ascii
```

`theme::ascii_glyph_set_has_no_box_drawing` — component-free, so it is not hostage to which painters exist: build `Theme::junie().builder().borders_set(border::ASCII).build()`, iterate `GlyphRole::ALL` through `GlyphSet::get`, **plus every field of `scrollbar()`, `rule_quiet()` and `rule_active()`**, and assert no `char` in `'\u{2500}'..='\u{257F}'`. This is the assertion `ascii_theme_renders_without_box_drawing_glyphs` can only approximate.

`ascii_glyphs_is_idempotent_and_glyph_overrides_it` — `.ascii_glyphs().ascii_glyphs()` equals `.ascii_glyphs()`; `.borders_set(ASCII).glyph(GlyphRole::RuleQuiet, "~")` yields `"~"`.

---

## O3. `border_subtle → Black` — **the document is wrong; `DarkGray` is correct; two ΔE estimates also need correcting**

### Verification **[derived]**, against `theme/downgrade.rs:161-203` and `theme/builtin/junie.rs:21-49`

`border_subtle = WHITE_15 = #262626 = (38, 38, 38)`.
* spread `= 38 − 38 = 0 < 40` → grey ladder (`downgrade.rs:170`).
* BT.601 luma `= (38·299 + 38·587 + 38·114)/1000 = 38 000/1000 = 38`.
* `38 ∈ 31..=110` → **`Color::DarkGray`** (`downgrade.rs:173`).

The implementation and `theme/downgrade.rs:504` are right; **§16.1 and §25.3 are wrong.** `#111111` (luma 17) is the value that reaches `Black`, and that is `surfaces[1]`, not `border_subtle`.

**Crucially, "Black" was never a carried fact.** The legacy pin `theme::tests::accent_survives_downgrade` (`src/theme.rs:647-655`) asserts exactly three things — `accent → LightGreen`, `error → LightRed`, `canvas → Black` — and `canvas = #000000`, luma 0. It says nothing about `border_subtle`. The review invented the claim while paraphrasing the legacy contract, and marked its own colour arithmetic *(estimate)* for exactly this reason. The re-derivation obligation did its job.

### Every other colour claim in §16.1 / §25.3, re-derived

| Token | Value | spread | luma / bright | Result | Doc says | Verdict |
|---|---|---|---|---|---|---|
| `accent` | `#48e054` | 152 | max 224 > 180 | `LightGreen` | LightGreen | ✔ |
| `danger` | `#e44545` | 159 | max 228 > 180 | `LightRed` | LightRed | ✔ |
| `danger_soft` | `#d98a8a` | 79 | max 217 > 180; `g=138>120` but `b=138 ≮ 80` → not Yellow | `LightRed` | LightRed | ✔ |
| `border_subtle` | `#262626` | 0 | luma 38 | `DarkGray` | **Black** | ✘ **wrong** |
| `fg[1]` | `#b3b3b3` | 0 | luma 179 ∈ 111..=200 | `Gray` | Gray | ✔ |
| `fg[0]` | `#ffffff` | 0 | luma 255 | `White` | — | ✔ (code `:507`) |
| `surfaces[1]` | `#111111` | 0 | luma 17 ∈ 0..=30 | `Black` | — | ✔ (code `:505`) |
| `warning` | `#f59e09` | 236 | `g=158>120 ∧ b=9<80` | `Yellow` | Yellow | ✔ |
| `info` | `#8787ff` | 120 | neither r nor g dominant, max 255 > 180 | `LightBlue` | — | ✔ (code `:512`) |
| `accent_pressed` | `#2b8632` | 91 | max 134 ≤ 180 | `Green` | — | ✔ (code `:510`, proves the dark half is reachable) |
| `highlight_danger_bg` | `#7a2a2a` | 80 | max 122 ≤ 180 | `Red` | — | ✔ (code `:511`) |
| `Indexed(196)` | → `(255,0,0)` | 255 | max 255 > 180 | `LightRed` | — | ✔ (code `:467-469`) |

Also confirmed: `downgrade.rs:161-203` is `src/theme.rs:604-641` **verbatim**, modulo saturating arithmetic and the dropped third (unused) match arm. F5 is discharged correctly.

### The two CIE76 estimates in §25.3, re-derived

Both were flagged *(estimate: re-derive before blessing)*. They are rationale for a **rejected** metric, so they gate nothing — but the obligation stands and they are off.

* **§25.3 reason 2**: `danger_soft #d98a8a` under CIE76. L\*a\*b\* `(65.6, 30.2, 12.7)`. ΔE to `DarkGray (127,127,127)` = **35.0**; to `Red (205,0,0)` = **62.6**; to `LightRed (255,0,0)` = 75.1; to `Gray (229,229,229)` = 41.4; to `White` = 47.5. **Minimum is `DarkGray`.** The *conclusion* is confirmed; the numbers `≈30` / `≈61` should read **`≈35` / `≈63`**.
* **§25.3 reason 3**: `#48e054` is L\* **79.2** (doc: ≈78); `Green (0,205,0)` L\* **72.0** (✔ 72); `LightGreen (0,255,0)` L\* **87.7** (✔ ≈88). Full ΔE: to `Green` **17.8**, to `LightGreen` **34.9**. The dark primary wins by nearly 2×. **Confirmed, strongly** — the review understated its own case.

### Exact sentences to write

**§16.1, `theme/` block (`COMPONENT_ARCHITECTURE.md:1685`)** — replace the parenthetical of `ansi16_preserves_hue_family_and_brightness`:

> `ansi16_preserves_hue_family_and_brightness` (F5: pins `DESIGN.md:320` — accent `LightGreen`, error `LightRed` — plus `danger_soft → LightRed`, `warning → Yellow`, `info → LightBlue`, and the grey ladder `surfaces[1] (#111111, luma 17) → Black`, `border_subtle (#262626, luma 38) → DarkGray`, `fg[1] (#b3b3b3, luma 179) → Gray`, `fg[0] → White`; plus the dark half `#2b8632 → Green` and `#7a2a2a → Red`. <!-- amended by §27 (Adjudication O3) --> The earlier `border_subtle → Black` was an unverified *(estimate)*: `#262626` has BT.601 luma 38 and lands in the `31..=110` `DarkGray` band. `#111111` is the value that reaches `Black`, and it is `surfaces[1]`. The legacy pin `theme::tests::accent_survives_downgrade` (`src/theme.rs:647-655`) constrains only `accent`, `error` and `canvas`; it never constrained `border_subtle`. No baseline is re-blessed.)

**§25.3, closing paragraph (`:5258`)** — replace the last sentence:

> `theme::downgrade_is_deterministic_per_level` asserts `LightGreen`/`LightRed`/`Yellow`; `theme::ansi16_preserves_hue_family_and_brightness` pins `DESIGN.md:320` plus `danger_soft → LightRed`, `warning → Yellow`, `info → LightBlue`, `surfaces[1] → Black`, **`border_subtle → DarkGray`** and `fg[1] → Gray`. <!-- amended by §27 --> `border_subtle → Black` was one of this section's *(estimate)* claims and is **wrong**: `#262626` is BT.601 luma 38, inside the `31..=110` `DarkGray` band. **No baseline is re-blessed for this change** — it restores the recorded output.

**§25.3, reasons 2 and 3 (`:5227-5228`)** — replace the bracketed estimates:

> …lands on `DarkGray` *(re-derived: ΔE 35.0 to `DarkGray` against 62.6 to `Red` and 75.1 to `LightRed`; the review's ≈30/≈61 were hand arithmetic)*, so a destructive label at rest stops being red at all.
> 3. Both `#48e054` and `#e44545` genuinely minimise ΔE against the **dark** primaries *(re-derived: L\* 79.2 for `#48e054` against 72.0 for `Green` and 87.7 for `LightGreen`; full ΔE 17.8 to `Green` against 34.9 to `LightGreen` — a factor of two, wider than the review's estimate)*, so no tie-break or bias recovers `DESIGN.md`'s answer while keeping ΔE.

### Rejected alternative

**Change the implementation so `border_subtle` reaches `Black`** — by widening the `0..=30` band or by special-casing chrome. Rejected outright: it would alter `nearest_16`'s categorical bands to satisfy a sentence that was never a contract, and `DESIGN.md:313-322` fixes only accent, error and the surviving glyph/modifier language. Restoring the legacy metric verbatim is F5's whole point (§25.3 reason 4); tuning it to match a paraphrase inverts the authority order.

### Risk

Only that the corrected value is itself unverified. It is not: the arithmetic is exact integer BT.601 over a checked-in `const`, and `theme/downgrade.rs:471-478` already pins both `#262626 → DarkGray` and `#111111 → Black` as separate assertions.

### Test

```
cargo test -p tui-next --lib theme::downgrade::tests::ansi16_preserves_hue_family_and_brightness
cargo test -p tui-next --lib theme::downgrade::tests::downgrade_is_deterministic_per_level
cargo test --all-targets theme::tests::accent_survives_downgrade          # the legacy pin, unchanged
! rg -n 'border_subtle → Black|border_subtle -> Black' COMPONENT_ARCHITECTURE.md docs/
```

---

## O4. Two perf assertions that cannot hold as written

### (a) `style_resolve_per_frame` — **substitute CONFIRMED in principle, with two corrections; reinstated in Slice 5**

**Facts.** `frame_showcase_lists_120x40` lives in `apps/showcase/tests/perf.rs` (§16.6 `:2025`), which Slice 5 owns (`:3985`). It does not exist. The stand-in measures a 40-row × 5-part frame twice — styles resolved per row versus hoisted — and takes the difference (`crates/tui/tests/perf.rs:211-298`). The asserted quantity is `resolution_ns × 10 ≤ 32 000` under `PERF_STRICT` (`:283-288`); the ≤ 5 % share is printed, not asserted (`:294-296`).

**Confirmed.** §25.8's own arithmetic — *"≈ 13 ns × ~2 000 style queries per realistic frame ≈ 26 µs, under 0.2 % of a 16 ms budget"* (`:5321`) — is machine-independent, absolute, and does not need a frame that does not exist. Substituting it for a share of a nonexistent frame is the right move, and it is the same move §25.8 itself made when it struck the unmeetable 2× ratio. The 0.027–0.060 share straddling ≤ 5 % is exactly the symptom of measuring a 3–6 % difference between two independently-taken medians on a shared runner.

**Correction 1 — the extrapolation multiplier is wrong.** Arm A performs **200** `ui.style` calls (40 rows × 5 parts, `:243-252`); arm B performs **40** (8 states × 5 parts, hoisted out of the loop, `:255-260`). The difference therefore covers **160** queries, not 200. `×10` extrapolates to **1 600** queries, not 2 000, so the assertion is ~20 % **weaker** than it claims. It must be `×12.5`:

```rust
const QUERIES_A: u128 = 200;   // 40 rows × 5 parts
const QUERIES_B: u128 = 40;    // 8 states × 5 parts, hoisted
const DELTA: u128 = QUERIES_A - QUERIES_B;                 // 160
let per_frame_2k = resolution_ns.saturating_mul(2_000) / DELTA;
assert!(per_frame_2k <= 32_000, …);
```

**Correction 2 — the asserted budget must not come from the noisy estimator.** `resolution_ns = a.ns − b.ns` is 3–6 % of `a.ns`, while run-to-run median noise on `a.ns` alone is comparable. The budget assertion should be computed from the **low-noise** measurement that already exists: `style_resolve_10k_parts` is a pure resolution loop, `120 141 ns / 10 000 = 12.0 ns` per query (`perf_baseline.txt:50`), with no differencing at all. So:

* **Assert** (under `PERF_STRICT`, in `style_resolve_10k_parts`): `s.ns / 10_000 × 2_000 ≤ 32_000`, i.e. **≤ 16.0 ns per query**. Absolute, machine-independent, one-sided, currently 12.0 ns with 33 % headroom, and it is *literally* §25.8's sentence turned into code.
* **Report** (in `style_resolve_per_frame`): the in-situ share and the corrected `per_frame_2k`. Keep the corrected `×12.5` extrapolation as a **second, looser** strict-mode net — it is the only measurement that includes real painting alongside resolution, and it is the number that will be compared against the real frame in Slice 5.

**Correction 3 — the test emits no baseline line.** `style_resolve_per_frame` never calls `report`, so it is absent from `perf_baseline.txt` (`:38-57`) while §16.6 requires additions to be marked there (`:2071`). Either call `report("style_resolve_per_frame", &a)` or add a `#`-header line naming it as a differential test that deliberately carries no baseline. Prefer the latter — a baselined `ns` for a differential invites a meaningless `×1.2` regression check.

**Reinstatement — exactly when and against what.** **Slice 5**, in `apps/showcase/tests/perf.rs::frames`, against `frame_showcase_lists_120x40`, as part of the Slice 5 gate (`:3988`). Not 4x: no work package before Slice 5 owns a showcase frame. Concretely: when the showcase list page exists, add `style_resolve_share_of_frame_showcase_lists_120x40` measuring the same A/B differential against the real frame and asserting `≤ 5 %` under `PERF_STRICT`; at that point `style_resolve_per_frame`'s extrapolation drops from *asserted* to *reported*.

**Rejected alternatives.**
* *Assert the ≤ 5 % share against the stand-in.* Rejected: the stand-in is the style-densest frame constructible from foundations — five resolutions per painted row, no chrome, no borders, no status bar — so its share is an upper bound on the real one, and asserting an upper bound against a threshold written for the real frame either fails spuriously or passes vacuously. This is the builder's own reasoning (`tests/perf.rs:290-296`) and it is right.
* *Delete the test until Slice 5.* Rejected: §25.8's budget is the only style-cost bound that binds today, and deleting it would leave §20.9-1's acceptance column naming a test that does not exist — the exact failure mode `every_named_test_exists` (F12) was added to prevent.

**Risk.** The `×12.5` differential can still flake at ~2× noise. Named and accepted, because the *binding* budget now comes from `style_resolve_10k_parts`; the differential is a secondary net.

### (b) `intents_drain_is_o_1_when_the_queue_is_empty` — **substitutes CONFIRMED; two document corrections**

**Facts, all verified.**
* The 14.9× is real and is the stub's own cost: `Probes::update` is `for i in 0..self.0 { cx.intents(..).count() }` (`crates/tui/tests/perf.rs:638-645`) and `Probes::draw` registers `n` controls (`:647-657`). O(n) by construction. The raw ratio is reported with `strict = false` (`:754`).
* Zero probes on an empty queue is **structural**, not statistical: `IntentQueue::iter` returns before `bucket_index` when `used == 0` (`crates/tui/src/intent.rs:342-349`), and `bucket_index` is the only site that bumps the counter (`:196-197`). Asserted at `tests/perf.rs:685-698`.
* `probes(500) − probes(20) == 480` (`:733-737`). The **differential** form is necessary and correct: `probes()` also counts the enqueue path (`bucket_slot → bucket_index`, `intent.rs:213-214`) and `was_drained`, so no absolute count is stable. The difference cancels every constant.
* 0 allocations on both the empty and the one-intent path (`:711`, `:749`).
* Normalised ratio **[derived]**: baseline `s500 = 632 ns` (`perf_baseline.txt:44`); at 14.9× that is `s20 ≈ 42 ns`; `(632 × 20)/(42 × 500) = 0.602`. The builder's 0.60 is confirmed. Asserted `≤ 1.25` under `PERF_STRICT` (`:755-761`).
* The normalised ratio is a genuine detector, not theatre: with `s = C + n·k` it reads `(20C + 10000k)/(500C + 10000k)`; if per-drain cost became O(n) — total O(n²) — it reads ≈ 25 and fails. It is exactly the "costs the same *per control*" property §16.6 meant.

**Confirm both substitutes as implemented.** They are deterministic, structural, and measure the named property, which the 1.25× wall-clock band never could.

**Correction 1 — §16.6's row (`:2077`) and §25.6 (`:5291`) state a count the code cannot assert.** Both say *"with 2 intents, probes are exactly one per drain call (500)"*. The test drives **one** intent and asserts the **difference** 480, because an absolute 500 is unattainable — the enqueue path probes too. Replace with:

> …with the queue non-empty, exactly **one probe per `cx.intents` call**: a 500-component frame performs exactly **480 more** probes than a 20-component frame in the same single update pass (`probes()` also counts the enqueue path, so only the difference is stable), allocations are 0, and a frame with an empty queue performs **0** probes because `IntentQueue::iter` short-circuits on `used == 0` before `bucket_index` is reached. <!-- amended by §27 (Adjudication O4b) --> ~~total probe cost is ≤ 500 × 5 ns~~ is **struck**: 2.5 µs against a measured 632 ns for the whole 500-control `handle` is not a bound, it is a tautology. The wall-clock ratio is **reported always and never asserted raw** — the raw 500-vs-20 ratio measures 14.9× because the stub application's own `for i in 0..n` update loop is O(n) by construction. What is asserted under `PERF_STRICT=1`, with a 1.25× band, is the **normalised** per-control ratio `intents_drain_ns_per_control = (ns₅₀₀ × 20)/(ns₂₀ × 500)`, which measures **0.60** and reads ≈ 25 if per-drain cost ever became O(n).

**Correction 2 — the constant 480 silently encodes "one update pass."** `Runtime::handle`'s focus re-run loop is bounded at four passes (§3.3 step 7); a legitimate second pass makes the delta 960. That is a real behaviour change worth catching, so keep the equality — but say so in the test comment and in §16.6, or the next reader will "fix" it to `% 480 == 0`.

**Rejected alternatives.**
* *Keep the 1.25× raw wall-clock band and make the stub's `update` O(1).* Rejected: an application that does **not** call `cx.intents` per component is not measuring the drain path at all. The O(n) loop is the workload; normalising it is the correct response.
* *Assert an absolute probe count.* Rejected: `probes()` is cumulative since construction and counts the enqueue path; no absolute number survives a change to focus staging. The differential is the invariant.
* *Reset the counter per frame so absolute counts work.* Rejected: it would make `probes()` a per-frame statistic and break the "since construction" contract §25.6 wrote, for no gain over the differential.

**Risks.**
1. `s20 ≈ 42 ns` is a small median; the normalised ratio inherits its noise. Mitigated by a 2× margin (0.60 against 1.25) and by the deterministic probe assertions being the binding ones.
2. The 480 constant couples the test to the update-pass count. Named above.

---

## Consolidated document amendments

| § | Line | Change |
|---|---|---|
| §11.2 | `:861-863` | `borders_set(border::ASCII)` applies `ascii_glyphs()`; the four roles are *exactly* the box-drawing-block bindings; whole typed sets are replaced (O2) |
| §16.1 `theme/` | `:1685` | `ansi16_preserves_hue_family_and_brightness`: `border_subtle → **DarkGray**`, not Black; add `surfaces[1] → Black`, `fg[0] → White`, `warning → Yellow`, `info → LightBlue`, `#2b8632 → Green`, `#7a2a2a → Red` (O3) |
| §16.1 `theme/` | `:1685` | `ascii_theme_renders_without_box_drawing_glyphs`: wording says "a `Scene` digest"; the test uses `Harness::text()`. Add the new `theme::ascii_glyph_set_has_no_box_drawing` (O2) |
| §16.6 | `:2041` | `style_resolve_10k_parts`: ≥ 90 % is a **key-correctness floor**, not a guarantee; record the geometry arithmetic; add the absolute ≤ 16 ns/query budget under `PERF_STRICT` (O1, O4a) |
| §16.6 | `:2042` | `style_resolve_per_frame`: the ≤ 5 % share is **deferred to Slice 5** against `frame_showcase_lists_120x40`; until then the extrapolated 32 µs budget stands, with the `×12.5` multiplier; the test carries no baseline line, deliberately (O4a) |
| §16.6 | `:2077` | `intents_drain_…`: differential probe form (480, one pass), strike the `≤ 500 × 5 ns` tautology, name the **normalised** ratio as the asserted one (O4b) |
| §20.9-1 | `:3820` | Acceptance column: the frame-level bound is deferred to Slice 5; the standing bound is the per-query budget (O4a) |
| §20.9-2 | `:3821` | Strike "direct-mapped" and "no eviction policy to get wrong"; correct the type sketch to `[(u64, u32, StylePatch); 256]` behind one `Box`, owned by the runtime core; record the two-way geometry and its arithmetic (O1) |
| §24.2 | `:5009`, `:5033` | `borders_set(ASCII)` → `ascii_glyphs()`; the full `GlyphSet` ASCII table is **scheduled for Slice 4E**, not deferred (O2) |
| §25.3 | `:5227-5228`, `:5258` | ΔE estimates → 35.0/62.6 and L\* 79.2 / ΔE 17.8 vs 34.9; `border_subtle → DarkGray` (O3) |
| §25.6 | `:5291` | Differential probe form; normalised ratio (O4b) |
| §25.8 | `:5327` | ≥ 90 % is a key-correctness floor; the deterministic unit test is the guarantee (O1) |
| §27 *(new)* | end | Adjudication O, recording O1–O4, mirrored in `REFACTORING_STATE.md` per the change-control rule (`:3`) |

Also: `crates/tui/src/theme/resolve.rs:7` (stale "direct-mapped"), and `crates/tui/tests/perf_baseline.txt` header — add the measured two-way hit rate and a line naming `style_resolve_per_frame` as baseline-free.

---

## Executable acceptance conditions

```bash
# — O1: the memo's shape, the document, and the wrap fix —
! rg -n 'direct-mapped' crates/tui/src COMPONENT_ARCHITECTURE.md
rg -n 'two-way set-associative' COMPONENT_ARCHITECTURE.md crates/tui/src/theme/resolve.rs
cargo test -p tui-next --lib theme::resolve::tests::cache_hits_after_the_first_query_and_clears_by_generation
cargo test -p tui-next --lib theme::resolve::tests::cache_generation_wrap_does_not_serve_a_stale_entry
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 style_resolve_10k_parts
#   PERF-CACHE line prints rate >= 0.99 on the shipped hash; the gate floor stays 0.90

# — O2: the coupling, the whole-set swap, and the residual gap —
cargo test -p tui-next --lib theme::glyph::tests::ascii_glyph_set_has_no_box_drawing
cargo test -p tui-next --lib theme::builder::tests::ascii_glyphs_is_idempotent_and_glyph_overrides_it
cargo test -p tui-next --lib theme::border::tests::ascii_border_set_is_pure_ascii
cargo test -p tui-next --test render theme::ascii_theme_renders_without_box_drawing_glyphs
rg -n 'ascii_glyphs' crates/tui/src/theme/builder.rs COMPONENT_ARCHITECTURE.md
rg -n 'Slice 4E' COMPONENT_ARCHITECTURE.md   # the full GlyphSet table is scheduled, not deferred

# — O3: re-derived, not estimated —
cargo test -p tui-next --lib theme::downgrade::tests::ansi16_preserves_hue_family_and_brightness
cargo test -p tui-next --lib theme::downgrade::tests::downgrade_is_deterministic_per_level
cargo test --all-targets theme::tests::accent_survives_downgrade
! rg -n 'border_subtle . Black' COMPONENT_ARCHITECTURE.md docs/
git diff --exit-code crates/tui/tests/perf_baseline.txt   # O3 re-blesses nothing

# — O4: both substitutes, and the deferral is dated —
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 style_resolve_per_frame
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 intents_drain
#   PERF-PROBES probes_500 - probes_20 == 480; PERF-RATIO intents_drain_ns_per_control <= 1.25
rg -n 'style_resolve_per_frame' COMPONENT_ARCHITECTURE.md   # names Slice 5 + frame_showcase_lists_120x40

# — change control —
rg -n 'Adjudication O' COMPONENT_ARCHITECTURE.md REFACTORING_STATE.md
cargo run -p xtask -- doc-check
cargo test --workspace --test architecture every_named_test_exists
```

**Gate pass condition.** Every command exits 0; `crates/tui/tests/perf_baseline.txt` changes only in its `#` header (O1's recorded hit rate, O4a's baseline-free note) unless `ascii_glyphs()` moves an allocation count; `every_named_test_exists` reports no missing name after the three new tests are added to §16.1; and `COMPONENT_ARCHITECTURE.md` carries the amendments to §11.2, §16.1, §16.6, §20.9-1/-2, §24.2, §25.3, §25.6, §25.8 and the new §27, each mirrored in `REFACTORING_STATE.md`.

**Slice 4 wave 1 is not blocked by any of the four.** O2's whole-set swap must land before **4E** (`ScrollRegion` paints `scrollbar.begin`/`.end`); the full `GlyphSet` ASCII table is scheduled *with* 4E; O4a's ≤ 5 % share is reinstated in **Slice 5**.
