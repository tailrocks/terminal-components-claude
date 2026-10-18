# Independent physical cursor follow-up review

ACCEPT bounded follow-up e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2. Clean detached committed source reviewed; no source/approval changes made.

Only production delta is cursor_position getter: min(internal_column, cols.saturating_sub(1)). For positive width N this correctly returns0..N-1, including N=1, while preserving pending-wrap internal columnN. It neither rewrites Grid.pos nor changes parsing. No internal vt100 caller uses this public getter; formatter/parser internals still use grid position. Rows are unchanged and grid resize clamps positive row/column bounds.

Independent external source-pinned test harness passes3 suites: every exact column in width8; pending margin at widths1,2,8,80 with DECAWM both modes, repeated getter calls, combining marks and next-character behavior; shrink resize across widths1,2,8,80 and heights1,2,3 followed by more input; raw replay rejects zero width/height before parser construction. Direct upstream vt100 zero-sized Grid construction/resize remains unsupported (existing subtract-one assumptions); this fix does not claim to add zero-sized terminal support. Public replay zero-dimension rejection is verified.

Pending-wrap proof: combining mark stays attached to margin A; next B wraps only when DECAWM enabled, otherwise overwrites margin. Repeated getter queries cannot consume pending wrap. No premature N-2 clamp or N-column exposure.

All24approved fixture files independently compared byte-for-byte to parent, unchanged; hashes in approved-unchanged.json. Commit changes only getter and26-line qualification test. Existing tool matrix rerun remains owner responsibility; this standalone harness isolates the exact patchedvt100 source and tuisnapreplay with its own saved Cargo.lock rather than claiming whole locked-tool dependency parity. No application snapshots or new visual approvals granted.
