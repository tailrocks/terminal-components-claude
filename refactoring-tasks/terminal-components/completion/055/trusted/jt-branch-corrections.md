# Jackin branch-review interaction obligations — TASK-055

## Authority and evidence boundary

This supplement binds BD-JT-13/17/18/20/22/28 Capsule, PTY and viewport families owned primarily by TASK-055. ADJ-20 governs route-specific Capsule admission. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-CAPSULE-ROUTES: enumerated admission paths — full owner TASK-055 (ADJ-20)

`CAPSULE-EDITING` opens Capsule with focused simulated pane editing.

1. **capsule-pane-paste-unrestricted:** paste with CR/LF payload. Require focused pane receives full sanitized paste2180–2194 without modal/menu/prefix/drag guard; contrast shell modal stack489–502.
2. **capsule-menu-paste-200-cap:** menu Paste with long payload. Require first 200 characters including CR/LF408–427.
3. **capsule-prefix-double-ctrl-b:** Ctrl+B Ctrl+B within 2000ms. Require Changed without forwarding1258–1263.
4. **capsule-usage-local-r-no-service-refresh:** Capsule Usage tab; press `r`. Require 800ms local spinner only2634–2676; no provider refresh mutation.

Sources: oracle `screens/capsule.rs:408–427,1258–1263,2180–2194,2634–2676`; shell `app.rs:489–502`.

## JT-PTY-TRANSCRIPTS: exact spans and deadlines — BD-JT-13

1. **pty-echo-delay-700ms:** ordinary input. Require Thinking until 700ms, not immediate echo.
2. **pty-permission-rotation-third-reply:** enough Claude inputs for third reply. Require permission block, not first-response repeat.
3. **pty-permission-deadlines:** accept/reject boundaries. Require source 400/900/1500 and 300ms rejection, not main 350/750/1100.

Sources: oracle `sim/pty.rs:927–1011,1140–1164,1174–1184`; main `sim/pty.rs:1258–1429`.

## JT-HISTORICAL-PAINT; JT-LIVE-REDUCERS; JT-PERF-WORKLOAD; JT-WORLD-POLICY

Remove screenshot-specific second composition (BD-JT-18), borrowed transcript projection (BD-JT-22) and workspace label projection (BD-JT-17) through named JA-* branches.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-057 closes Jackin union.
