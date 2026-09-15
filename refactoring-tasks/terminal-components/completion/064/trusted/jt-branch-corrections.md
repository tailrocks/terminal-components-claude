# TablePro branch-review interaction obligations — TASK-064

## Authority and evidence boundary

This supplement binds BD-JT-08 application journey restoration and full TablePro closure. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-TEST-JOURNEYS: genuine public sequences — BD-JT-08

Replace marker/flag substitutes in main `app_tests.rs` and identical baseline digests with actual cancellation, typed acknowledgement, save and per-Surface handler observations.

1. **connections-cancel-real-token:** cancellation journey. Require typed/mouse acknowledgement matching oracle `app_tests.rs:365–381,438–494`, not bool toggle substitute.
2. **surface-header-not-parity:** named Surface sweep. Require handler/state changes per Surface, not generic Production/Connections header only.

Sources: main `app_tests.rs:108–271`, `tests/baselines/tablepro.txt:50–73`; oracle legacy `app_tests.rs`.

## Verification binding

R-001/CHK-004 frame+semantic conjunction with TASK-005 oracle fixtures and TASK-073 instrumentation qualification. Zero unresolved TablePro entries from sibling supplements 058–063.
