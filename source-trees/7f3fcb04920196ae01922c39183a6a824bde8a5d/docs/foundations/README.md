# Shared foundations

Each mechanism has one owner. Components refer here for shared rules and keep only component-specific policy in their own contracts.

| Contract | Owner |
|---|---|
| [`identity.md`](identity.md) | Semantic IDs, stable item/column/field/action keys, and revisions. |
| [`input-actions.md`](input-actions.md) | Normalized events, keymaps, typed actions, response flow, and invalidation. |
| [`runtime.md`](runtime.md) | Update/draw frame lifecycle, focus, hit testing, pointer capture, feedback, and time. |
| [`layers.md`](layers.md) | Overlay order, anchors, backdrop, dismissal, focus trapping, and restoration. |
| [`layout.md`](layout.md) | Constraints, measurement, geometry, clipping, surfaces, and narrow layout. |
| [`theme.md`](theme.md) | Semantic roles, recipe resolution, capabilities, parts, and patches; exact visual tokens are in the visual contract. |
| [`text.md`](text.md) | Shared Unicode editing, source projection, selection, cursor, and viewport mapping. |
| [`collections.md`](collections.md) | Borrowed collections, stable-key reconciliation, ordering, filtering, and scroll state. |
| [`secret-validation.md`](secret-validation.md) | Secret storage, masking, validation lifecycle, redaction, and zeroization. |
| [`author.md`](author.md) | Constrained custom parts, rows, cells, and author composition. |
| [`conformance.md`](conformance.md) | Test-only registry shape and production dependency boundary; verification owns oracle and acceptance rules. |
| [`session.md`](session.md) | Optional terminal backend/session lifecycle and cleanup edge. |

The canonical target architecture is summarized in [the architecture overview](../architecture/overview.md). Public API signatures and type rules live under [API](../api/README.md). The frozen baseline remains the source for observable behavior.
