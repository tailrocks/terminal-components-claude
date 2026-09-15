# Jackin branch-review interaction obligations — TASK-052

## Authority and evidence boundary

This supplement binds BD-JT-12/15/25/27 families for Accounts, Usage and service-backed modal stages owned primarily by TASK-052. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; main keyed architecture retained. Source-inspected planning branches only.

## JT-ACCOUNTS-OPERATIONS: scoped account UI — BD-JT-27

`ACCOUNTS-PRELUDE` opens global Accounts with seeded fixtures visible.

1. **accounts-duplicate-name-warning-not-rejection:** save duplicate display name. Require post-save warning893–975, not rejection.
2. **accounts-discovered-refresh-only-row:** discovered account selected. Require Refresh only in visible action row1118–1133; keyboard `v`1997–2000 has no mutation-allowed guard.
3. **accounts-row-validate-scheduled:** row Validate. Require 900ms scheduled path1047–1061 distinct from synchronous Form Validate2363–2378.

Sources: oracle `screens/accounts.rs:893–975,1047–1061,1118–1133,1997–2000,2212–2221,2363–2378`.

## JT-USAGE-COCKPIT-INSPECT: independent scroll/focus contracts — BD-JT-25 (Usage slice)

1. **usage-refresh-stagger-120ms:** enabled accounts refresh. Require provider duration plus 120ms per index128–160, distinct from Accounts 160ms stagger.
2. **usage-last-good-retry:** rate-limit failure. Require last-good/retry outcome866–888 preserved.

Sources: oracle `screens/usage.rs:128–160,866–888`.

## JT-ACCOUNT-PRECEDENCE; JT-SERVICE-MODALS; JT-WORLD-POLICY

Account precedence, OpFlow/FileBrowser service stages and workspace effective-set explanations bind through JA-023/032–039 primary rows with exact oracle multi-account/vault/item enumerations from BD-JT-12/15.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-057 closes Jackin union.
