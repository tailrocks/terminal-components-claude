# Downloads TCC common ODB local-object closure

This inventory reads only the 5,012 loose objects physically under the Downloads common Git object directory. Its `objects/info/alternates` points to the canonical Projects TCC ODB; that external ODB is not counted as local. Each loose object was decompressed, checked against its SHA-1 pathname, and compared by full OID, type, and size with Integration’s canonical 257,216-row TCC object list.

Only local object IDs absent from the canonical typed list are copied, as exact compressed loose-object bytes under `new-loose-objects/`. Matching OIDs are not duplicated. Refs and reflogs are mapped in the Global r12 addendum; this report covers the physical local loose-object set only.
