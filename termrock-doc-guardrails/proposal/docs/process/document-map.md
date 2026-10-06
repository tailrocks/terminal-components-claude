# Canonical document map

## One owner per requirement

Keep the existing detailed contracts where they remain useful.
Merge these proposed documents by topic.
Do not paste a second complete specification beside the old one.

| Existing path | Required change |
| --- | --- |
| `AGENTS.md` | Add component ownership and exact-subject review rules. Preserve repository conventions. |
| `GOAL.md` | State the active documentation-only task. Preserve the later implementation goal through links. |
| `README.md` | Explain the library and preview purpose. Link to the normal consumer examples. |
| `SPECIFICATION.md` | Summarize the target. Link to canonical contracts instead of duplicating them. |
| `CRATES.md`, `crate-map.json` | Preserve verified crate boundaries. Require functional ownership, not a package-count target. |
| `CHECKLIST.md`, `checklist.json` | Reconcile stale claims and add evidence requirements without losing existing IDs. |
| `component-ownership.json` | Add or link semantic/pixel/interaction ownership, not only destination paths. |
| `docs/architecture/overview.md` | Resolve obsolete one-library wording and link to the composition boundary. |
| `docs/api/public-api.md`, `types.md` | Separate implemented and proposed signatures. Choose one documented action/value contract per component. |
| `docs/api/authoring.md` | Separate component authors from normal preview authors. Remove unrestricted repaint implications. |
| `docs/components/*.md` | Apply the component template and add correct use, rejected use, parts, and verification. |
| `docs/foundations/*.md` | Preserve one owner for runtime, text, layout, theme, layers, and collections. |
| `docs/applications/*.md` | Describe real component compositions and fixture behavior. Do not define independent product redesigns. |
| `docs/design/*.md` | Keep exact baseline defaults. Link customization to separately tested extensions. |
| `docs/verification/*.md` | Distinguish conversion, integrity, execution, parity, ownership, and approval. |
| `docs/implementation/*.md` | Add component-owned repair work and review gates. Keep implementation separate from this task. |
| `refactoring-tasks/**` documents | Add per-work and per-commit review requirements. Reference canonical rules. |
| Scoped `AGENTS.md` and `README.md` files | Remove conflicting permission for app-local rendering or review bypasses. |
| `.github/pull_request_template.md` | Summarize commit receipts and separate documentation acceptance from runtime acceptance. |

## Status records

Do not replace root implementation status with a new parallel status ledger.
Import new requirement rows into the current authoritative ledger or link a compatible companion section.
Keep the readable view derived from that authority.

The separate documentation checklist in this pack is an import proposal.
Its status does not supersede current repository evidence.
All rows start pending.

Keep full immutable source references in the research record.
Keep code examples at their canonical API location.
Link to examples from application and component documents.

## Non-document files

Do not change implementation source to reconcile a documentation example.
Do not alter executable verification data as a document cleanup.
Do not change Cargo, CI, Mise, or baseline files during this task.
Record such changes as explicit later tasks.
