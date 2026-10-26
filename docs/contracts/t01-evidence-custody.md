# T01 Evidence Custody Contract

## Scope

This contract covers allowlisted T01 artifact lookups. It preserves exact commit, tree, path, mode, blob, byte-length, and hash identities.

The archive is partial. It does not provide checkout, build source, ordinary ancestry, or full-history support. It does not replace refs or alter source history.

The public verifier is `verify_archive(provider_root, manifest_path, binding_map_path, git_dir)`. It publishes only the verified partial Git object set.

## Trusted destination

The destination hierarchy must be owned by the system or effective user. It must have no untrusted write permissions or extended ACL entries.

Every inspected directory must be opened without following symlinks. The verifier retains its descriptors and checks their identities. It rejects symlink ancestors, identity changes, unsupported APIs, and failed checks.

The ACL decision is `D1-DARWIN-ACL-INSPECTION`, approved by `/root/t01_task_acceptance_review` at Sol high effort. It uses a fresh opaque `filesec_t` for each retained directory descriptor.

The verifier calls `fstatx_np` and then `filesec_query_property(security, FILESEC_ACL, &valid)`. Both calls must return zero. Only `valid == 0` establishes that no extended ACL is present. A present ACL or any API error rejects.

`acl_get_fd_np` returning `NULL` with `ENOENT` is not treated as proof that an ACL is absent. The query result controls acceptance. A stale `errno` after a successful call does not change that result.

The verifier must release every security object and close every descriptor on all paths. It must fail closed when the SDK, platform, architecture, or filesystem cannot support the required inspection.

## Publication boundary

The final path must be absent. This includes a dangling symlink. The verifier must reject an existing entry without changing it.

The verifier checks platform and filesystem capability before reading source artifacts or importing objects. It uses `renameatx_np` with `RENAME_EXCL` on one filesystem.

Each attempt creates a unique sibling container and a fixed `repository` child. Both are effective-user-owned and mode `0700`. The verifier retains the parent, container, and child descriptors.

The verifier imports and checks every declared object through the child descriptor. It waits for subprocesses and closes writable handles before sealing the child identity.

A child identity change before seal must reject and leave the final path absent. A container name replacement immediately before rename must never publish the replacement directory.

The only publication operation is `renameatx_np(container_fd, "repository", destination_parent_fd, final_name, RENAME_EXCL)`. The verifier must not resolve the container path again to select the source.

The container and repository child must share the destination filesystem. Unsupported platforms, filesystems, or capabilities reject before object import.

## Failure and retry

Before publication, a failure must leave the final path absent. A bounded `UNVERIFIED` record must remain inside the private child when the child exists.

Each retry uses a new container. It preserves earlier containers and their evidence. It does not replace or reuse an existing final path.

An interruption after rename is an uncertain outcome. The verifier preserves the published directory and requires inspection. It must not report semantic rejection or verified success.

The verifier does not use copy, overwrite, delete-and-recreate, or unsafe fallback publication. It never removes failed or published evidence.

## Threat model

The publication decision is `D1-TRUSTED-LOCAL-PUBLICATION-AMENDMENT`, approved by `/root/t01_task_acceptance_review` at Sol high effort. The decision narrows the earlier publication guarantee.

The destination hierarchy must be controlled by the system or current user. The design excludes hostile same-UID mutation, root mutation, and arbitrary inner-repository mutation after seal.

Descriptor checks do not protect against those excluded actors. The earlier unqualified guarantee remains blocked.

## Current evidence status

The capability check reads `VOL_CAP_INT_RENAME_EXCL` from capability and valid word 1 with mask `0x00080000`. A trusted temp hierarchy passed. `/dev` failed closed because its capability was invalid.

The current helper passed 16 archive cases, 11 nested-type cases, and target and plan publication. It passed source/cache denial, network denial, and one pre-commit and one post-commit interruption check.

The recorded outer-container substitution happened after pack import and before the seal checks. It rejected and preserved the imported pack. It did not prove the exact interval immediately before native rename.

The current helper has no exact native `EEXIST` observation. Two LLDB attempts failed before verifier execution because process attach was denied. Inner-child replacement before seal also remains pending.

The partial archive does not complete the source lookup census, ordinary prerequisite bundles, active-reader integration, or T01 acceptance. Product cases remain `0 / NOT VERIFIED`. The static artifact plan is `NOT_APPLICABLE_STATIC_EVIDENCE`.
