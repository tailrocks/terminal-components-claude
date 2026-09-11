# Independent BF witness review — components A

Read-only review of coordinator additions W-010-09/10, W-012-08, W-013-10…15 after ADJ-15/16 authoring. Main pin `7b27732a8c3c131760ec3438f641cb3c11343a42`; oracle pin `02f5294bfdbf38004cc49130d0aff1d01f31434c`. No production experiment or parity claim; source control-flow proof only. No reviewed witness file was changed.

## Findings

1. **Scope blocker:** completion/010/verify.toml:3 omits keymap.rs/event.rs, but new W-010-10 requires changing keymap's conflict and focused-hint equivalence. Counterexample: Char('A') NONE and SHIFT match the same physical key under event.rs132–139, while keymap.rs83/275/312/327/648 compare structural equality. Root condition is split effective-key identity, not a missing test. Add narrow keymap.rs + event.rs ownership for one effective-equivalence helper and its matching/conflict/hint consumers, preserving structural Eq/Hash and case. Later keymap writer028 must follow010. Existing runtime-only scope cannot close this witness. intent.rs452's exact claim comparison has only one production consumer (Form Enter at1263), so this review does not demand a speculative additional claim-routing change.

2. **Existing contradictory oracle tab wording:** W-013-04 says “four-column tab stops.” Oracle ui/text.rs8–15 expands every tab into four spaces independent of starting column. With a TAB b, b appears at column5, not next stop4. Qualify shared generic tab-stop API separately from source fixed-four expansion; preserve original copy offsets. Source bounds must not compare a newly invented stop policy to oracle frames.

3. **Fuzzy boundary preservation clarification:** W-013-14 correctly repairs whole-string casing/scalar matches and source byte-based subsequence scores while retaining public original-grapheme ordinals. Its general “oracle ... score ordering” must not erase main's explicit FuzzyBoundary::Word default versus Identifier distinction (fuzzy.rs18–27). Oracle uses only underscore/dot for boundary bonus. Add explicit Word/Identifier controls (labels a-b, a b, a_b, a.b with needle b), retaining accepted default Word while source-qualified callers select Identifier where required. The casing/coordinate repair does not authorize changing default boundary semantics.

## Authorized repair follow-up

After the read-only findings above, coordinator authorized precision/scope repairs. TASK-010 now owns event.rs/keymap.rs only for shared effective equivalence, explicitly retaining structural identity and existing exact Form Enter claim. Read-only ancestry verification confirms009 precedes010 and028 contains010 among ancestors. W-013-04 now freezes fixed-four source tab expansion; W-013-14 separates Word default from oracle Identifier; W-013-11 explicitly floors requested selection endpoints and ceils cursor/floors retained anchor after atomic edit. No production implementation or oracle frame changed. Independent coordinator review remains pending.

## Per-witness semantic review

| Witness | Source check and conclusion |
| --- | --- |
| W-010-09 | runtime run_update exits without clearing when focus does not move; Bubble/Esc calls it again before finish clears. Intent iter starts pos0 and only sets drained diagnostic bit. Always-draining app and raw-consumed/notification controls genuinely distinguish the defect. Runtime pass-boundary scope is sufficient. |
| W-010-10 | Exact effective versus structural identity counterexample confirmed, including all four conflict loops and hint dedup; matching case/non-character controls are correct. Scope blocker above remains. |
| W-012-08 | Oracle horizontal automatic minima failure keeps second pane; vertical keeps first. Main same-first branch and named test deliberately differ. Proposed repair preserves modern anchored empty rectangles/saturation rather than copying Rect::ZERO. Correct source/disposition split. |
| W-013-10 | Oracle normalized_text, constructors, set_text and insert_char/insert_str share line normalization; main constructors/set_text omit it and insert_char permits CR. Ordinary/sensitive entrance matrix correctly identifies structural invariant. |
| W-013-11 | Oracle select_range floors both requested endpoints; normalize_positions ceils cursor and floors anchor after atomic replacement. Main snap only floors to scalar and post-edit arithmetic can land inside joined cluster. Witness correctly avoids invented illegal-offset app trajectory and requires exact independent endpoints. |
| W-013-12 | Oracle word walk classifies whole grapheme by any alphanumeric scalar and underscore is separator. Main scalar walk/is_word_char underscore differ; snake_case, combining and punctuation controls distinguish actual endpoints. Keep one shared predicate/walk. |
| W-013-13 | Both sensitive and ordinary insert_str report inserted length after deleting selection, producing false on real deletion. Whole replacement transaction outcome is correct; no-selection empty remains unchanged. TextInput's independent idle lifecycle remains016-owned. |
| W-013-14 | Oracle SearchText lowercases whole string, maps expanded scalars back to whole original graphemes, and source subsequence score uses last original byte offset. Main per-grapheme eq_fold and ordinal score differ. ΟΣ/ος and a+acute/acute controls distinguish these; retain public ordinal mapping plus boundary clarification above. |
| W-013-15 | Oracle projects an overwide grapheme to ellipsis before row width accounting. Main emits Break from empty then original wide text. Width1 日 case is decisive; shared emitted/count traversal and zero-width policy remain correct. |

## Actual source read coverage

Ranges below were read completely, not inferred from search hits. This is a bounded function audit, not a new whole-file coverage claim.

| Pin/path | Git blob | Complete ranges read |
| --- | --- | --- |
| M crates/tui/src/text/buffer.rs | bea24207e111916d5aeff79aea5781d1e1b61596 | 1–455 |
| M crates/tui/src/text/fuzzy.rs | d09c144981a370277d2b4404dd395e00b16b019d | 1–end (137) |
| M crates/tui/src/text/measure.rs | 44b774b9678eccd4eada70ab40f2d21f773fa28a | 1–225 |
| M crates/tui/src/runtime.rs | bca132db3e0c75dcbe6b7325d4889ca390730a1c | 1200–1325,1420–1510 |
| M crates/tui/src/intent.rs | 603d7787a3ae88d788a19fa559d8c7c20a9f486a | 375–465 |
| M crates/tui/src/keymap.rs | c417d575b4967c0f89e5987e74e723e5c84dcc92 | 65–102,250–345,620–675 |
| M crates/tui/src/event.rs | 696abe05d513690a7585b031b80a83204f694e7f | 110–155 |
| M crates/tui/src/layout.rs | d04a0b8c37685d8588b28fe5596ca021ef387614 | 450–520,815–860 |
| O src/core/text.rs | c4fa0a7d98af6825548fa5be6c44112e23d60390 | 1–330,400–495 |
| O src/ui/text.rs | 5c0685716ff0362ac6502901be75c77b3c0cc0b9 | 1–end (405) |
| O src/ui/layout.rs | 170bb28cd20dc0e81b414eaf3cdec795f7e8f89b | 70–165 |

Original complete branch foundation report was read as supporting inventory; it does not replace these independent source checks. Parent owns repairs and final adjudication. Source feasibility is not protected-host execution or candidate parity.
