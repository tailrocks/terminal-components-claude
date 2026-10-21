# Typed Dialog and PropsList slot adapters

Plan: `jackin-info-typed-slot-and-scope-plan.json`, SHA-256 `a7cee804ab65d1f343e48af82585a876ef4c9337e7c6f3305ef009c1310b8ff0`.

This increment adds a component-author-only nested `LayerSlot` bridge, a `Dialog::draw_body_slot` adapter, public `DialogSlotExt` and `PropsSlotExt` methods, and a `composition::props_list` leaf. Dialog drawing delegates to the existing `Dialog::draw` path, so chrome, body geometry, registration, focus, and clipping remain shared. PropsList drawing and measurement delegate to its existing APIs; its current measurement intentionally does not take row data and keeps its existing maximum-height behavior.

`crates/termrock/tests/slots.rs` adds a different-data consumer with Unicode property labels and values, stable keys, reordering, modal body composition, keyboard copy, and a clipped-parent comparison. The first independent review rejected commit `a0c620e5806c2b1aa92ac6dbdfc11aa62fc22ce9`: intersecting the logical layer area before constructing `LayerSlot` changed child measurement and registered regions. The bridge now retains the original logical area and relies on `Ui::with_area` for clipping, matching the existing direct renderer. The review then found the first committed regression placed the clip outside `layer_slot`, whose layer setup reset it; that test passed with the broken implementation. The regression now applies clipping inside the layer's component-author callback, immediately before both typed and direct Dialog draws. Temporarily restoring the old intersection makes this committed test fail at TrueColor; the corrected implementation passes full cell, region, focus, and cursor comparisons in all four color profiles. The adapter tests exercise the public facade; they do not wire Jackin or claim frozen application parity. Runtime command-scope behavior, Dialog metadata slots, PropsList data-aware intrinsic measurement, and application migration remain separate work.

The shared worktree contained 1,301 tracked deletions under `crates/termrock-conformance/target/tuiscotti-*` before this increment. These paths were left untouched and must not be staged. Their disappearance cause remains unproven; integration must preserve the parent investigation record and must not describe the cause as fixed.

Focused verification on the authored source:

- `cargo nextest run -p termrock --test slots --features termrock/testing --offline`: 5 passed, 0 skipped. The consumer exercises Down, stable-key reorder, and Enter-copy without closing the Dialog. Direct and typed Dialog/PropsList rendering match at 60×20 and 24×8 across TrueColor, Ansi256, Ansi16, and Mono; complete terminal cells/styles, registered regions, focus, and cursor match. The added clipped-parent case compares the complete frame, all registered regions, focus, and cursor at Rect(20,7,12,4) in all four color profiles.
- Independent external consumer suite from `typed-dialog-props-slots-independent-review-a0c620e.json`, rerun against this worktree source: 11 passed, 0 skipped. Its clipping regression applies the clip within the component-author callback and also compares against the original renderer.
- `cargo clippy -p termrock-runtime -p termrock-overlays -p termrock --all-targets --features termrock/testing --offline -- -D warnings`: passed on final source.
- `rustfmt --edition 2024 --check` on all seven changed Rust files and `git diff --check`: passed on final source.
- `cargo fmt --all -- --check`: fails on the pre-existing formatting in `crates/showcase-demos/src/pages/chrome.rs`. That unowned file was restored to its committed content and left unchanged.

Independent tests do not yet exercise Tab/ShiftTab dismissal and restoration of a background focus owner; the slot adapter does not change input routing, and the authored consumer currently focuses on the PropsList's existing keyboard flow.

This is library/API evidence only. It does not establish the frozen Jackin workflow or complete product parity.
