# TASK-031 frozen host context templates

Host `prepare`/`freeze` MUST materialize CHK-006 from `trusted/check-context-templates/CHK-006.template.json` plus run/task/tree bindings. Candidates cannot alter template bytes.

## CHK-006 — architecture + branch host projection

- Operation: `architecture`
- Required-set membership index: `/task/trusted/branch-host-projection.tsv`
- Every `branch_host_projection` row must resolve to an accepted producer receipt before TASK-031 dispatch
- Native conformance witnesses (`native_conformance` role) include W-031-10/11 from this task's `source-witnesses.md`
- **Forbidden:** W-042-JUMP-SUBMIT (owned by TASK-042)

Host merges the projection index into the frozen architecture context; CHK-006 rejects missing, extra, or unaccepted branch witnesses.
