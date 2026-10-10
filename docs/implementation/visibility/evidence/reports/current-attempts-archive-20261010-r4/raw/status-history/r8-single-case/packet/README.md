# VIS-01 attempt-history R8 single-case gate

This packet retries only `status_renders_previewed_v1_and_v2_identically_and_rejects_invalid_v2`, the sole failure in the preserved R7 run. The exact Nextest filter selects one test. The pinned source inventory retains all 53 status integration cases; the other 52 remain unselected. The fixture helper's one unit test remains inventoried and unselected, while its regular executable and `.d` source provenance remain required.

The R8 source change is fixture-only: normalize `branch_scopes` in the two scoped claim-history snapshots used to construct the representable schema-v1 migration fixture, assert those exact contexts, and verify every non-scope field is preserved. Production validation is unchanged.

R7's 52/53 pass result and one failure remain preserved. R8 is not a full rerun. Test success does not qualify product readiness, capture, or admission. Root alone owns Cargo/Nextest execution.
