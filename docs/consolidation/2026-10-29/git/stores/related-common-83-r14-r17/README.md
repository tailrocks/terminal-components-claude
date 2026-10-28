# Related Git-store preservation candidate (R14/R15/R17)

This is an external staging candidate for the 83 non-canonical Git common stores inventoried in R14, related-object groups in R15, and raw admin-state capture in R17. It is not yet approved for publication.

The candidate keeps all 83 store identities and 11 OID-set equivalence classes in metadata; 584 source ODB files are represented by 289 content-addressed raw byte payloads. The store and file maps preserve source paths, modes, sizes, SHA-256 values, object-file kinds, ref/HEAD lineage, and relative payload locators. Raw object bytes are not merged into the canonical TCC pack.

All 387 R17 admin source records (83 HEAD, 83 raw index, 109 reflog, 73 FETCH_HEAD, 26 loose refs, 10 packed-refs, 2 shallow files, 1 alternates file) map to 129 exact raw payloads under `inputs/r17/raw-admin-payloads/`. R14/R15/R17 source inventories and their sealed checksum files are copied unchanged under `inputs/`. A high-confidence pattern scan over the unique raw admin payloads found no matches; this is static screening only and independent Safety review remains required. Raw config values and hooks are excluded.

The R17 snapshot originally marked 30 ref-only OIDs as pending physical closure. The later Downloads-TCC-v4 map in `inputs/downloads-v4/` closes that question: all 46 refs / 44 OIDs resolve through the canonical TCC store and/or the separate Downloads-TCC store. The Downloads-TCC store remains a distinct group and is not embedded here.

Portable payload references are relative to this directory. Original absolute paths are retained only as source-identity metadata. This candidate is not a runnable Git repository and does not claim import/fsck validation. No source Git store was modified, and no local copy deletion or publication is implied.

One 68,749,298-byte pack payload is stored as two ordered chunks because individual files are limited to 48 MiB. `object-file-map.jsonl` points its three source aliases to `objects/chunks/sha256/6828da74633a9d9e15b45682b1e17ac8efb035691aa13c1c1da9a84b749e36ad/chunk-map.json`; concatenation in index order reconstructs the exact source SHA-256. All other payloads remain direct content-addressed files.

## Decoded size and pack verification

The original R14 pack-OID manifests remain byte-identical. For 4,510 unique OIDs their `size_bytes` values record pack delta-representation sizes, not decoded logical sizes. The supplemental `inputs/decoded-size-review-v2/` directory contains the sealed correction table, per-pack source-row provenance, and a decoded-size inventory for all 9,026 OID/type entries. All 11 packs were imported in an isolated verifier. OIDSET-007 and OIDSET-009 have cross-group links that resolve in the wider store union; OIDSET-011 is empty and depends on canonical TCC plus Downloads-TCC refs/objects.

The R18 correction record is also preserved under `inputs/r18/`; its nested `SHA256SUMS.txt` is included verbatim. It states explicitly that 4,510 original R14 `size_bytes` rows are packed-delta representation sizes, and that decoded logical sizes come from the supplemental verified map.

The R18 correction record is preserved verbatim, including its schema text. Its listed JSON field names/nesting do not exactly match the serialized reviewer outputs; `inputs/decoded-size-review-v2/schema-attestation.json` records the actual schemas: `oid`/`type`, and the nested `pack_manifest_provenance` array. The 4,510 correction rows are unique OIDs; there are 15,588 mismatch occurrences across pack rows.
