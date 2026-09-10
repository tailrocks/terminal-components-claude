# Qualified capture package integration review

Root reviewed worker `263be4f9828d9f734e0ad2dfda9a06b90d1c28e4`, integrated as
`16e4e7a`. This approves the portable acquisition package and its explicit tool
limits, not application snapshots or parity.

Reviewed acquisition source: immutable upstream/final commits and trees, lockfile
hashes, source cleanliness, external targets, compiler/Zig pins, checked repair
payloads, fresh output roots, unique PTY sessions and failure status. All 43
payload hashes and three license files verify. Two independent integrity tests
pass, exercising four missing/tampered patch and bundle cases before acquisition.

Fresh worker acquisition executed 29 commands successfully. Its immutable ledger
SHA-256 is `30067a3b070b728f6c6e81a2c4acfe9fd7ff860baa8b7931ce5728675bd76792`.
Root verified all current smoke-artifact hashes and all three executable hashes
against that ledger, then reran the harness, complete 22-command tui-test process
journey and oracle into a separate empty artifact directory. Both engines pass
their declared contracts; five mutations per engine are detected.

Repaired tui-snap has zero missing fields or canonical differences in this
384-cell fixture. tui-test retains exactly one known blink difference and absent
width/continuation fields; any extra loss is rejected. Cursor appearance, blink
phase, general termios equality and application parity are not proved. See the
versioned package README for the full limits and previous full tool test matrix.

Root evidence: `qualified-capture-root-review/root-commands.json` and generated
raw artifacts under the external integration evidence root. The package's native
unified patch retains required blank context markers; those bytes are hash-bound
source evidence, not authored trailing whitespace to strip.
