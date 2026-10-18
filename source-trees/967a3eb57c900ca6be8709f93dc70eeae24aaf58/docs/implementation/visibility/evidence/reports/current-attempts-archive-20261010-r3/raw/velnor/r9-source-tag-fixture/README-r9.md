# R9 focused expected-fixture verification

Status: frozen, not run; qualification remains NOT_QUALIFIED. This packet selects exactly `impl_schema2_routing::schema2_workflows_match_expected_bytes` from package `velnor-actions-orchestrator-generation`.

The external source snapshot changes one markerless expected workflow fixture to the body derived from the Root-authorized R15 capture. The capture’s full workflow SHA-256 is `0a5fee8dd77dad5a6ef426ce9b3bda9789028bc51ca4f51bf65fceeba8c9eef2`; independent R15-vs-R14 output review is `7c153b316ed342b123a6da780cfab4122ba14f5dbfbad343829c0fb8298368c3`. Root’s admission decision is `b77ef986f5b7fff7eed9353e85edac513136fdd525ffc6cb79973068fcaa687c`. The changed Rust snapshot path is `crates/services/velnor-actions-orchestrator-generation/tests/snapshots/generator-release-source-tag.yml`, with body SHA-256 `7af373e587f8729e3a51ff9c174710fdbae418599849ecb313f2f69eb4c24828`.

The source is a Git-less external snapshot. The one-test result cannot qualify Velnor, the CLI, the generated workflow, a release, or the Termrock corpus. The prior R8 failure remains preserved at its original packet and receipt.

Runner: `/private/tmp/termrock-vis11-tool-fixture-gate-r9-source-tag-fixture-luna-20261010/run-root/runner-r9.cjs` (SHA-256 `8708f48faa4374d732d1e5c975467e89e96f4919af6f881346c7533775f8c463`). Preflight stdout SHA-256: `de799040af6f6ec348b4cb15c690436b11346cd4ed208e56d5cca6472b56df4c`; stderr is empty (SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`). Both execution and target directories remain absent after preflight.

Preflight only: `/opt/homebrew/Cellar/node/26.11.0_1/bin/node /private/tmp/termrock-vis11-tool-fixture-gate-r9-source-tag-fixture-luna-20261010/run-root/runner-r9.cjs --preflight`. Root-controlled execution only: `/opt/homebrew/Cellar/node/26.11.0_1/bin/node /private/tmp/termrock-vis11-tool-fixture-gate-r9-source-tag-fixture-luna-20261010/run-root/runner-r9.cjs --execute`.
