# Independent historical renderer review

ACCEPT installation of the exact 499 HTML and 499 PNG outputs into the required missing historical baseline paths. This decision covers generator commit c051c8fa61c2fd67eb06e122dd91fd2553c8c810 and source provenance commit 040a2726c6f8d48590bc5b787afae2a0839096a0. It grants no approval to application acceptance snapshots or a Holla acceptance namespace. No source or baseline files were changed by this reviewer.

## Hash binding and exhaustive checks

- Initial manifest SHA256: 24e492a6aee8b1719b16e10d72ee72ccf2c14a81bb702917e941d5f5a3959608.
- Committed pins SHA256: e2dcdb2952e5cbd283b841ce055962c4d961dfa967d43c879bdbc49a5abdb6cb.
- Independently fetched and hashed all 1,500 pinned immutable Git blobs: 1,497 ANSI/text/cursor inputs, two renderer sources, and the recipe source. Manifest captures equal committed pins.
- Independently checked all 998 expected output hashes in each of generated, reproduced, and clean-checkout-generated: 2,994 comparisons, all equal.
- Verified the exact font archive and all four font hashes. Archive SHA256 04d5e8f903693f9dd13e16f867e994834e681eb3c72c0d337a770dcda09010cf also matches the retained release checksum. Read embedded OFL.txt: JetBrains Mono copyright attribution and SIL Open Font License 1.1; its SHA256 is 30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad. Published license copies match the archive.
- Independently verified d5e7075 and 0cd1bf7 have identical src, tools, Cargo.toml, and Cargo.lock objects. Their seven changed paths are documentation only; source-tree-equivalence.json records object IDs.

## Source and publication review

Read the entire committed generator, failure tests, README, requirements, baseline provenance, and both pinned renderer implementations. Inputs come from capture archive c12cad8728755cd2d03eefdd8e02891143fca86d; renderer source comes from d5e7075f436f0e437c7d12cf3d1e638e763b26f6. Candidate application output cannot enter the expected artifact pipeline. Every recipe input and output is hash checked; all 499 unique IDs and viewports must match pins. Pillow is pinned to 11.3.0. Font extraction uses verified archive bytes and exact file members, without extracting arbitrary archive paths.

Generation occurs in a temporary directory on the destination filesystem. Existing destinations are refused. Publication uses rename only after all hashes succeed; failures remove scratch output. This script does not install baselines. Keep FONT-OFL.txt and generated PROVENANCE.json associated with installed artifacts and retain their regenerated-historical-renderings-not-originals classification. The original executable hash is unknown and remains null.

Independently ran all eight committed negative tests: wrong font, wrong renderer, missing source, missing recipe, duplicate recipe, wrong input, wrong output, and stale destination. All passed; failure-tests.log preserves results.

## Actual visual inspection and limitations

Viewed generated PNGs and opened actual generated HTML in local headless Chrome for showcase_overview_default_80x24, tablepro_safety_dialog_delete_120x40, and jackin_manager_default_120x40. Inspected overview palette/navigation, destructive SQL confirmation content and dim backdrop, and workspace details/actions. Chrome screenshots are retained here for inspection only; they are not baseline files or expected rendering hashes.

Visual inspection covers three of 499 captures per format; hash coverage is exhaustive. Historical PNG glyph/tofu and width behavior remains visible, particularly in JackIn. HTML depends on browser font resolution; HTML and PNG have distinct historical cursor/style rendering behavior. Renderer source retains its historical limitations rather than silently correcting images. Approval establishes reproducible historical evidence, not current application visual correctness, original binary recovery, or recovery of previously unavailable original image bytes.

No blocking findings within this bounded installation scope.
