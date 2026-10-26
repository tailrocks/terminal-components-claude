# T37 component caller-contract transfer evidence

## Input and scope

- Frozen prose source: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
- Frozen product source: `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
- Candidate source inspected: `cfe0587ad034d9f0fd18694934f071f6f50c7dc5`.
- Raw archive: `results/t37-components/removed-rustdoc.jsonl`, SHA-256 `44540f7e878b477d87f0b8b74185398baa0c56f0218e8ac07a18d083394dae93`.
- Archive inventory: 722 groups across 61 source files.

The high decision `T37-COMPONENT-PROSE-01` selected 112 new caller clauses, 15 exact existing mappings, and one approved retirement. The coordinator added G343 as one narrow Form pointer-hold clause because the existing Form-lanes prose does not state its press/release lifecycle. The evidence ledger records the gap and retained `Form::sync_pointer_hold` range.

The final ledger has 113 direct transfers, 15 existing mappings, and one approved retirement. It does not claim that unselected API descriptions migrated. The raw archive remains historical provenance; T01 and T35 own the complete API inventory and guide review.

G581 is an approved retirement under `T37-SELECTION-BINDING-01`. Its existing receipt has a pending output-source-commit field. The current candidate source hash and final high disposition are recorded separately.

## Evidence and limits

- `transfer.json` holds 129 selected groups with exact source commit, path, line range, file SHA-256, attachment SHA-256, full attachment, maintained excerpt, and decision record.
- The ledger records target source SHA-256 values for all 33 selected component source paths.
- G343's retained-code evidence is `form.rs:1172-1214`; its new clause needs independent output review.
- CodeEditor Cancel still commits when blur commit is disabled. Any rollback expectation remains open under T17.
- NoticeReveal tracks PAGE hover only without active capture. Any broader layer-observation expectation remains open under T15.
- Selected behavior cases: 0. Behavior status: `NOT VERIFIED`.
- No Cargo command, build, or test ran.

## Commands and results

- `sha256sum docs/implementation/pr38-final/results/t37-components/removed-rustdoc.jsonl docs/implementation/pr38-final/results/t37-components/approved-selection-binding-retirement.json` — PASS. Hashes match the values above and the retirement record.
- Inline Python validation using `git show` — PASS. Verified archive hash, all 129 source attachments against c817 line ranges and file hashes, 33 retained cfe file hashes, 129 destination excerpts, the retirement receipt, and G343's retained source range.
- Inline clause sentence-length check — PASS. Maximum length is 22 words. The two sentences over 20 words are descriptive; all remain within 25 words.
- `python3 -m json.tool docs/implementation/pr38-final/results/t37-component-api-transfer/transfer.json >/dev/null` — PASS, exit 0.
- `git diff --cached --check` — PASS, exit 0, after staging only the three owned files.

An initial validation attempt expected G581 to be the final sorted record; G609 follows it. The checker was corrected to locate G581 by group ID, and the full validation passed.

## Output hashes

| Path | SHA-256 |
|---|---|
| `docs/contracts/component-caller-contracts.md` | `4b7a25d7d7f6a9b653cdbb8d920fa070f6eb616f8ffa162ddd6e4f76982d397f` |
| `docs/implementation/pr38-final/results/t37-component-api-transfer/transfer.json` | `62f689ceb048f327dce010b569da85834262e7e5dd7018f3d0506855a9ca8f97` |

## Review status

The implementation agent did not self-approve. Independent output review is pending, with special attention to C14/G343. Required reviewer selection: GPT-6.1-Sol, high effort.
