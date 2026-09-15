# TablePro branch-review interaction obligations — TASK-062

## Authority and evidence boundary

This supplement binds BD-JT-05/08 database fixture and live queue consumer families owned primarily by TASK-062. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-DB-FIXTURE: preserve migrated executor — BD-JT-05

Retain catalog values, SQL tier/gate, deterministic executor, row cap and plan tree from main db/sql modules. Restoration obligation is the missing live queue/acknowledgement consumer, not reimporting database semantics into shared Grid.

Sources: branch-diff-jackin-tablepro.md BD-JT-05 read checkpoint; main `db/` modules.

## JT-TEST-JOURNEYS

Replace marker save/cancel tests with actual public event sequences from BD-JT-08; bind through TASK-064/073, not fixture-only assertions.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-064 closes TablePro union.
