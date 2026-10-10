# VIS-08 R17 reader R3 execution packet

State: prepared for Root review; not authorized by this packet.

The candidate is the reviewed R3 source successor. Its only change from R2 corrects the synthetic failed-run suite event label. R2’s observed 25/26 Nextest failure remains preserved as historical evidence.

The runner executes the exact 26 source-declared Rust tests, then builds and invokes the Rust reader only if each earlier stage exits successfully. It records every stage and reports reader reprocessing as NOT_RUN when skipped. Node stores raw process streams and status; it does not interpret Nextest JSON or Rust report JSON.

The reader stage consumes the pinned immutable R17 request, selection, and raw captures. Its exit status is a parser exercise result, not product-test success. Product behavior and acceptance remain unestablished.

Launch argv (Root authorization required):

/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node /private/tmp/termrock-vis08-r17-aliasfix-r3-execution-runner-20261010/runner.cjs --run

Preflight was read-only: no Cargo, tests, or CLI were run. The fresh output root is /private/tmp/termrock-vis08-r17-aliasfix-r3-execution-20261010.
