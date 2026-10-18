# R15 actual-capture archive proposal

This is a new external, append-only byte-copy proposal. It is not a canonical repository write and does not perform admission or qualification.

## Preserved outcome

- Original attempt: wrapper exit 125; child Nextest exit 0 with one selected test passing.
- Original wrapper receipt: input_stable=false; after_snapshot=null; capture_gate_ok=false.
- Reconstructed postflight report: created after completion; it preserves the original failure and does not prove continuous input stability during the child run.
- Capture receipt: capture_status=COMPLETE; execution_anchor_status=unverified; qualification=blocked; admission_status=NOT_RUN.
- The copy includes the capture receipt and its 40 actual output artifacts across four checkpoints. It contains no expected corpus or suite source.

## Copy limits

The byte copies and manifest bind the available files at archive time. They do not repair or replace the original wrapper receipt, create a missing after-snapshot, verify the execution anchor, qualify the reference, or admit expected output.
