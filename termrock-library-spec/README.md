# Termrock · library-only specification pack

**Pinned visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Deliverable:** specifications and reference files; no Jackin implementation, no Rust implementation and no claimed captured snapshots.

Open `index.html` for an offline, searchable reference. The Markdown and JSON files are suitable for a future repository's `docs/` and `reference/` directories. This pack does not contain fonts, mirrored repositories, product code or generated screenshot approximations.

## Start here

[Master specification](SPECIFICATION.md) · [45 component references](COMPONENTS.md) · [12 foundation references](FOUNDATIONS.md) · [Proposed public API](PUBLIC-API.md) · [Shared type dictionary](reference/TYPES.md)

[Implementation plan](IMPLEMENTATION-PLAN.md) · [Exact visual verification](VISUAL-VERIFICATION.md) · [Generic composition fixtures](recipes/README.md) · [Review/evidence status](REVIEW.md)

## Machine-readable references

`reference/components.json` holds the full component catalog; `reference/public-api.json` extracts its constructor, state, action and signature proposals. `reference/family-disposition.json` maps all 54 earlier families into this scope. `reference/sources.lock.json` pins source/tool references and observed blob IDs. `capture-plans/` contains one unapproved plan per component, validated by `reference/capture-plan.schema.json`. `reference/validation.json` records document-pack checks, not application test results.

## Scope correction

The earlier plan's Jackin integration stages are deferred. Current implementation work is only Termrock, generic helpers, examples, and a test laboratory consuming public APIs. Jackin is requirements/appearance evidence, not a second project to rebuild inside this one.
