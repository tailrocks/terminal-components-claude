# T37 component caller-contract transfer evidence

## Input and scope

- Frozen prose source: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
- Frozen product source: `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
- Candidate source inspected: `cfe0587ad034d9f0fd18694934f071f6f50c7dc5`.
- Raw archive: `results/t37-components/removed-rustdoc.jsonl`, SHA-256 `44540f7e878b477d87f0b8b74185398baa0c56f0218e8ac07a18d083394dae93`.
- Archive inventory: 722 groups across 61 source files.

The high decision `T37-COMPONENT-PROSE-01` selected 112 new caller clauses, 15 exact existing mappings, and one approved retirement. The coordinator added G343 as one narrow Form pointer-hold clause because the existing Form-lanes prose does not state its press/release lifecycle. The evidence ledger records the gap and retained `Form::sync_pointer_hold` range.

The final ledger has 113 direct transfers, 15 existing mappings, and one approved retirement. It does not claim that unselected API descriptions migrated. The raw archive remains historical provenance; T01 and T35 own the complete API inventory and guide review.

The first high output review returned `CORRECTION` for commit `932d36c18c3dc5dbce350e27290a732e0b8957bf`. The immutable review is complete. Its final correction list now covers split reset policy, Form submit ownership, Panel and scroll-window coordinates, fixed Settings painters, Tabs plans and bindings, disabled editor find calls, NoticeReveal timing, and viewport revisions. Corrections are applied; independent review of the new exact output commit remains pending.

G581 is an approved retirement under `T37-SELECTION-BINDING-01`. Its existing receipt has a pending output-source-commit field. The current candidate source hash and final high disposition are recorded separately.

## Evidence and limits

- `transfer.json` holds 129 selected groups with exact source commit, path, line range, file SHA-256, attachment SHA-256, full attachment, maintained excerpt, and decision record.
- The ledger records target source SHA-256 values for all 33 selected component source paths.
- G343's retained-code evidence is `form.rs:1172-1214`. C14 limits the hold to automatic reveal and records missing, hidden, disabled, and stale-press clearing.
- C37 records both disabled `open_find` guards: neither method changes find or completion state. Enabled `CompletingCodeEditor` dismisses and clears before delegation. Read-only editors remain eligible.
- The Roles gate accepts printable ASCII or `·` and checks one-cell text. Broader Unicode support remains open under T07/T28; no source fix is included.
- CodeEditor Cancel still commits when blur commit is disabled; any rollback expectation remains open under T17. NoticeReveal tracks PAGE hover only without active capture; broader layer observation remains open under T15.
- Selected behavior cases: 0. Behavior status: `NOT VERIFIED`.
- No Cargo command, build, or test ran.

## Commands and results

- `sha256sum docs/implementation/pr38-final/results/t37-components/removed-rustdoc.jsonl docs/implementation/pr38-final/results/t37-components/approved-selection-binding-retirement.json docs/contracts/component-caller-contracts.md docs/implementation/pr38-final/results/t37-component-api-transfer/transfer.json` — PASS. Archive and receipt match their recorded values; output hashes appear below.
- `python3 - <<'PY'` archive/source/destination/locator/anchor/sentence audit using `git show` — PASS. Verified 129 source attachments and excerpts, 33 retained cfe file hashes, 63 current-code locator hashes, natural links, and G343/G129/G139 ranges.
- Inline clause sentence-length check — PASS. Maximum length is 22 words. Sentences over 20 words are descriptive; procedural clauses remain within 20 words.
- `python3 -m json.tool docs/implementation/pr38-final/results/t37-component-api-transfer/transfer.json >/dev/null` — PASS, exit 0.
- Natural heading slug to ledger section mapping — PASS; the guide uses default Markdown anchors only.
- `git diff --check` — PASS, exit 0, on the uncommitted correction set.

An earlier checker expected G581 to be the final sorted record; G609 follows it. The checker was corrected to locate G581 by group ID, and the full validation passed.

## Output hashes

| Path | SHA-256 |
|---|---|
| `docs/contracts/component-caller-contracts.md` | `3de91e0ed9af1e2125992cc839956f8ab1aa3def4512ec2ab1444670c01f9348` |
| `docs/implementation/pr38-final/results/t37-component-api-transfer/transfer.json` | `f6c20e642c2f67500dbfb5d0e2ef6d8bde454042c29e89ffb8b250fb2598f619` |

## Review status

The implementation agent did not self-approve. The complete bounded correction list is applied; independent exact-output review remains pending. Required reviewer selection: GPT-6.1-Sol, high effort.
