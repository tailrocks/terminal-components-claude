# TablePro branch-review interaction obligations — TASK-061

## Authority and evidence boundary

This supplement binds BD-JT-04/07/08/09 Query, History and fixture families owned primarily by TASK-061. ADJ-18 governs Query read-only composition. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-HISTORY-FIXTURE: exact bytes and order — BD-JT-04

1. **history-fifteen-records-multiline-error:** open History. Require 15 seeded records including multiline SQL and two failure records with exact quote bytes; seven-record main fixture is insufficient.
2. **history-first-line-ellipsis:** display first line. Require U+22EF oracle vocabulary, not U+2026 substitution.

Sources: oracle `model.rs:218–385`; main `model.rs:117–188`.

## JT-QUERY-READONLY: source composition — BD-JT-09 (ADJ-18)

1. **query-results-read-only:** editable SQL result grid attempted. Require read-only Query rows with source reason even when engine identifies editable table.

Sources: oracle `tabs.rs:2030–2070`.

## JT-PICKER-PASTE: History search — BD-JT-07

`HISTORY-SEARCH-IDLE` opens History with search field idle; TabList Picker above.

1. **history-search-picker-paste-begins-edit:** PASTE(" find "). Require History search draft changes and editing begins from idle; Picker query unchanged.

Sources: oracle `app.rs:236–254`, History search consumer paths from BD-JT-07 read checkpoint.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-064 closes TablePro union.
