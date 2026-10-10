# PR-17 provider observation at candidate 977d (raw archive)

This VIS-13 archive preserves a read-only provider observation captured on 2026-10-10. The original capture manifest is retained byte-for-byte and lists 118 payload files with their source paths, sizes, and SHA-256 digests. The capture summary records its source script, native command-capture binary, and GitHub CLI identities.

At capture time, pre/post native ref reads agreed on candidate 977d7f4bb2bc7e6d3d4541651feede4e77ada1bb, reference 4b473a98a8641a9dae8dbf9c31c0c94b15465496, and tag object 1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5 peeled to 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b. The PR head/state was read before the provider queries; it was not re-read after them. Treat the record as a time-bounded observation if source refs later advance.

The selected push run 38031747998 completed with conclusion failure for candidate 977d. Its check suite had zero check runs; the jobs and artifacts endpoints returned empty lists. A separate DCO check run passed. The run-log endpoint returned HTTP/2 404 with a Not Found body, so this archive does not identify a provider failure cause. The workflow-source API payload is preserved separately: .github/workflows/ci.yml was 537,470 bytes and matched Git blob 672078dc6c964a768c05b53f73de8bb99b90bffa (SHA-256 607ed7980b6c18f95cfc08e1f8dee1aac12477b5d1e678f26c84a75b8f238c24). That source measurement is not a causal diagnosis.

This is raw provider/source evidence only. The capture did not run product tests or product commands, does not normalize a current status, and makes no readiness or qualification claim. Any report must keep the captured run separate from current source pointers and preserve NOT_RUN for unobserved current tests.
