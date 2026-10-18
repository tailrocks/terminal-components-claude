# Finding-to-work crosswalk

This page maps earlier finding identifiers to the active improvement contracts. The task files own current scope, priority, implementation status, regression requirements, and acceptance evidence. This crosswalk does not certify that implementation is complete.

| Finding family | Active work |
|---|---|
| Collection identity and action eligibility: PLAN-API-01–03, CV1–CV3 | [F02](f02-picker-action-eligibility-and-destructive-target-identity.md), [F03](f03-atomic-listbox-mutation.md), [F05](f05-preserve-treeview-target-across-lazy-insertion.md) |
| Grid identity and accepted data shape: PLAN-API-04–06, C01–C04 | [F06](f06-preserve-datagrid-cursor-identity-through-local-sort.md), [F07](f07-checked-row-schema-ingestion.md) |
| Modal, picker, and editor input ownership: INT01–INT05 | [F01](f01-modal-first-paste-routing.md), [F08a](f08a-between-upper-value-paste.md), [F08b](f08b-picker-grapheme-editing-query-paste.md), [F08c](f08c-ctrl-shift-home-end.md), [F08d](f08d-unassigned-modifiers-perform-plain-actions.md) |
| Inspect copy destination: INT07 and R8 | [F09](f09-deliver-inspect-copy-without-closing-the-modal.md) |
| Text rendering, retention, and ownership: TXT-01–TXT-07 | [F10](f10-segment-logical-text-before-applying-styles.md), [F11](f11-one-retention-aware-mutation-boundary.md), [F12](f12-rebase-retained-identity-never-silently-retarget-selection.md), [F13](f13-make-cache-validity-part-of-content-ownership.md), [F14](f14-explicit-tab-control-geometry-and-copy-policy.md), [F15](f15-incremental-live-output-work-measured-separately-from-idle-redraw.md) |
| Showcase reachability and empty geometry: D1, D2, D4, D5 | [F16](f16-reach-every-showcase-section-at-supported-sizes.md), [F17](f17-empty-allocations-emit-nothing.md), [F18](f18-distinguish-read-only-from-disabled.md), [F23a](f23a-state-reachability-design-d4.md) |
| Text selection and output search: INT06, R1–R2 | [F19](f19-keyboard-selection-in-the-existing-textviewport.md), [F20](f20-find-in-read-only-output-through-existing-composition.md) |
| Terminal lifecycle, rendering, and verification: TV1–TV5, V01–V09 | [F21](f21-restore-terminal-state-for-supported-job-control-suspension.md), [F22](f22-separate-rasterizer-fidelity-from-terminal-correctness.md), [F23b](f23b-lifecycle-v01-v02.md), [F23c](f23c-event-freshness-resize-v03.md), [F23d](f23d-input-flood-fairness-v04.md), [F23e](f23e-dirty-idle-performance-v05.md), [F23f](f23f-rendering-color-compatibility-v06-v07.md), [F23g](f23g-evidence-provenance-v08.md), [F23h](f23h-precise-subsidiary-regressions-v09.md) |
| Optional matrix paste, terminal extensions, and read-only APIs | [O04](o04-matrix-paste.md), [O05](o05-terminal-extensions-os-clipboard.md), [O07](o07-read-only-field-apis.md) |

The active specs retain accepted concrete reproductions and acceptance cases. The separate [Holla parity matrix](../parity/holla-parity-matrix.md) remains the capability authority for its Covered, Partial, and Not Covered distinctions; simulated UI work does not close operational parity.
