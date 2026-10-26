# T01 historical requirements

Input commit: 542349e2f70ec9883d743594f906ac6c4e96ba5a.

The JSONL retains 4,511 rows, including duplicates and original lanes.
Stable IDs combine the source commit, original path, and physical line.
The preserved 1d6 allowlist is byte-identical to the e08 source file.
Source allowlist SHA-256: c83bfcc2c6c1f57289332d1c837683503f1c2a41d7fe13385b825eafb4540c0d.
The JSONL is hashed as exact UTF-8 bytes with LF separators.

Historical JSONL SHA-256: 97bf85da8fdd14df32331988a29bc45bea9330c2c7315d595d7919dd20f7b410.
Lane counts: SC1 100; SC2 1,008; SC3 247; review 3,156.
Disposition counts: ADAPT 1535, KEEP 2909, RETIRE 67.
Unresolved row count: 0.
Rows and migration items carry current source pointers and named tasks.
Source bindings do not prove behavior, compiler visibility, or test results.

The fragment maps 165 migration items, 54 routes, and four application owners.
The performance inventory retains 71 historical entrypoints for T33.
Historical and current measurements remain NOT VERIFIED.
The current route census records 24 Live, one Selection, and 29 Placeholder entries.
Five Placeholder entries retain pages; 24 paint labels. Route behavior remains NOT VERIFIED.
Current SQL source keeps NULL last in both sort directions; behavior remains NOT VERIFIED.
The shell reach decision retires 11 helper rows; four route behavior obligations remain proposed.
Historical route and NULL-sort text remains unchanged.
The three sibling artifacts are supplemental and are not accepted here.

Run the structural inventory producer:
`python3 docs/implementation/pr38-final/results/t01-history-requirements/build_fragment.py`

No legacy command, benchmark, functional test, or product code ran.
Selected cases: zero. All proposed cases await T38 design and review.
This fragment awaits independent Sol high review.
