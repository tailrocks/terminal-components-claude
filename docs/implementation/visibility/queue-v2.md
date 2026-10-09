# Queue v2 plan and root registry

The Termrock repository remains the sole authority for `tasks.json` and the
generated `WORK_QUEUE.md`; the queue-coordinator worktree owns those files.
Registry paths live in a local file outside the repository. Schema-v2 task
records name stable repository and worktree IDs; they never store a
machine-specific root path.

This increment adds read-only schema-v2 role validation, queue read/render, and
a migration preview. It does not change the accepted task file, run a live
registry preflight, migrate accepted records, or grant write capability to a
reference or tool worktree. Existing schema-v1 `check`, `render`, and mutation
behavior remains the live queue contract during this stage.

## Registry schema 1

The registry file has these exact top-level fields. It must bind both the
candidate and reference worktrees. The Velnor source-pin and scratch entries
are optional during preflight; if either is present, both must be present and
the scratch must refer to the separate source pin.

```json
{
  "schema_version": 1,
  "repositories": {
    "terminal-components-claude": {
      "origin_url": "https://github.com/tailrocks/terminal-components-claude.git"
    },
    "velnor-new": {
      "origin_url": "https://github.com/tailrocks/velnor-new.git"
    }
  },
  "worktrees": {
    "candidate": {
      "repository_id": "terminal-components-claude",
      "root": "/local/path/to/candidate",
      "common_dir": "/local/path/to/candidate/.git",
      "mode": "branch",
      "branch": "termrock-implementation",
      "pinned_sha": null,
      "writable": true,
      "source_pin_id": null,
      "allowed_paths": [],
      "source_selection_review": null
    },
    "reference": {
      "repository_id": "terminal-components-claude",
      "root": "/local/path/to/reference",
      "common_dir": "/local/path/to/candidate/.git",
      "mode": "branch",
      "branch": "visual-baseline",
      "pinned_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "writable": true,
      "source_pin_id": null,
      "allowed_paths": [],
      "source_selection_review": null
    },
    "velnor-source-pin": {
      "repository_id": "velnor-new",
      "root": "/local/path/to/velnor-source-pin",
      "common_dir": "/local/path/to/velnor-source-pin/.git",
      "mode": "source-pin",
      "branch": null,
      "pinned_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "writable": false,
      "source_pin_id": null,
      "allowed_paths": [],
      "source_selection_review": {
        "review_record_path": "/local/control/source-selection-review.json",
        "review_record_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
      }
    },
    "velnor-scratch": {
      "repository_id": "velnor-new",
      "root": "/local/path/to/velnor-scratch",
      "common_dir": "/local/path/to/velnor-scratch/.git",
      "mode": "scratch",
      "branch": null,
      "pinned_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "writable": true,
      "source_pin_id": "velnor-source-pin",
      "allowed_paths": ["src/lib.rs"],
      "source_selection_review": null
    }
  },
  "visual_authority": {
    "tag": "visual-baseline",
    "tag_object_sha": "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5",
    "peeled_commit_sha": "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
  }
}
```

The local paths and the `a`-filled reference SHA above are schema examples.
The visual tag object and peeled commit are the fixed Termrock authority
values. The registered `origin_url` value is compared literally to both fetch
and push URLs for `origin`.
Worktree roots and Git common
directories are absolute, normalized paths. A scratch worktree has a distinct
root from its source pin and starts at the same pinned commit. The source-pin
and scratch entries are separate identities; the queue never writes into the
source-pin entry. The `writable` field records the integrator's policy; it is
not an operating-system permission boundary.
Registered worktree roots must be disjoint: one root cannot be inside another.

Every worktree entry declares an exact, sorted `allowed_paths` list. Recursive
globs, duplicate paths, and parent/child overlaps are rejected. Candidate and
source-pin entries use an empty list in this read-only increment; the reference
may later use an exact allowlist. A scratch requires a nonempty exact list.
These fields describe policy; this read-only command does not grant write
access or sandbox a process.

An external source pin and its writable scratch use the same stable repository
identity and pinned commit, but separate roots. The source pin is detached and
declared read-only. Its repository ID and registered `origin_url` value must
both differ from the candidate's. The scratch is detached and is the only
writable target.
Each source pin has exactly one associated scratch. The source-pin entry's
`source_selection_review` field points to a separate external receipt by an
absolute normalized path and its raw SHA-256. The receipt path must be outside
every registered worktree and must not equal the registry file. Every path
component is opened without following symlinks, the leaf must be a regular
file, and the receipt is limited to 64 KiB. The registry's own raw SHA-256 is
independently caller-pinned before the registry is parsed.

The receipt schema has exactly these top-level fields:

```json
{
  "schema": "termrock-source-selection-review/v1",
  "reviewer": "independent reviewer identifier",
  "decision": "approved_for_source_selection",
  "reviewed_at": "2026-10-08T00:00:00Z",
  "subject": {
    "repository_id": "velnor-new",
    "commit_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "tree_sha": "dddddddddddddddddddddddddddddddddddddddd",
    "path_policy_sha256": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
  },
  "evidence": ["https://example.invalid/source-review/record"]
}
```

The `reviewer` value must be a nonempty string, `reviewed_at` must be a
canonical UTC RFC 3339 timestamp, and `evidence` must be a nonempty array of
HTTPS links.

The receipt binds only source selection. Its `subject.commit_sha` must match
the source-pin entry's `pinned_sha`. Its `subject.tree_sha` must match the
actual source-pin Git tree, read with Git replacement refs disabled. Its
path-policy digest binds the stable repository ID; source-pin and scratch IDs
and roles; the source-pin commit and tree; and the scratch's sorted exact
allowlist. The digest is SHA-256 over the
`termrock-source-selection-path-policy/v1\0` domain prefix followed by UTF-8
canonical JSON (sorted keys, no insignificant spaces) for that policy object.
The exact object is:

```json
{
  "repository_id": "velnor-new",
  "source_pin": {
    "commit_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "role": "immutable-source-pin",
    "tree_sha": "dddddddddddddddddddddddddddddddddddddddd",
    "worktree_id": "velnor-source-pin"
  },
  "scratch": {
    "allowed_paths": ["src/lib.rs"],
    "role": "writable-isolated-scratch",
    "worktree_id": "velnor-scratch"
  },
  "schema": "termrock-source-selection-path-policy/v1"
}
```

The receipt is parsed with duplicate-key and non-finite-number rejection.

Reviewer identifiers and receipt hashes are integrator-provided data. They do
not authenticate a person, prove an independent review occurred, qualify a
binary, or establish release readiness. The receipt's decision is limited to
`approved_for_source_selection`. No external tool-source commit is hardcoded in
the validator; an external caller must pin the registry bytes and supply the
reviewed source identity. Rust fixtures use synthetic receipt data to exercise
the protocol; those values are not actual source approvals. They read the
immutable visual tag through a local Git object alternate and never check out
the baseline's large file tree. A dirty shared release checkout does not
qualify as either the immutable source pin or the isolated scratch.

The reference branch's pinned source SHA and the frozen tag identities are
separate facts. The reference worktree is the executable source. The annotated
tag object and peeled commit remain the visual oracle. The validator pins those
identities to tag object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` and peeled
commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` in code; caller-supplied
registry values cannot retarget them. The reference worktree's pinned commit
must also be an ancestor of the local canonical
`refs/remotes/origin/visual-baseline` ref, using Git with replacement refs
disabled. This is a local repository check, not proof that the remote server
currently advertises that ref.

## First-increment schema-v2 roles

Schema-v2 records keep the same top-level fields as schema v1, with
`schema_version` set to `2`. Every task and every prior claim snapshot adds
`repository_id` and `worktree_id`. These are logical identifiers. Records do
not contain local root paths.

The role registry adds one `queue_authority` and a role for each registered
worktree. The coordinator owns the only task file, generated view, and queue
lock. It is a writable `termrock-implementation` worktree. The separate
`candidate-source` role is a pinned, read-only source worktree in the same
repository and Git common directory. The `reference` role is pinned and
read-only during this increment. Velnor source-pin and scratch roles retain
their separate roots and repository identity.

```json
{
  "schema_version": 2,
  "queue_authority": {
    "repository_id": "terminal-components-claude",
    "worktree_id": "queue-coordinator"
  },
  "repositories": {
    "terminal-components-claude": {
      "origin_url": "https://github.com/tailrocks/terminal-components-claude.git"
    }
  },
  "worktrees": {
    "queue-coordinator": {
      "role": "queue-coordinator",
      "repository_id": "terminal-components-claude",
      "root": "/local/path/queue-coordinator",
      "common_dir": "/local/path/repository/.git",
      "mode": "branch",
      "branch": "termrock-implementation",
      "pinned_sha": null,
      "writable": true,
      "source_pin_id": null,
      "allowed_paths": [],
      "source_selection_review": null
    },
    "candidate-source": {
      "role": "candidate-source-authority",
      "repository_id": "terminal-components-claude",
      "root": "/local/path/candidate-source",
      "common_dir": "/local/path/repository/.git",
      "mode": "source-pin",
      "branch": null,
      "pinned_sha": "cccccccccccccccccccccccccccccccccccccccc",
      "writable": false,
      "source_pin_id": null,
      "allowed_paths": [],
      "source_selection_review": null
    },
    "reference": {
      "role": "reference-source",
      "repository_id": "terminal-components-claude",
      "root": "/local/path/reference",
      "common_dir": "/local/path/repository/.git",
      "mode": "branch",
      "branch": "visual-baseline",
      "pinned_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "writable": false,
      "source_pin_id": null,
      "allowed_paths": [],
      "source_selection_review": null
    }
  },
  "visual_authority": {
    "tag": "visual-baseline",
    "tag_object_sha": "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5",
    "peeled_commit_sha": "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
  }
}
```

Schema validation requires the coordinator, candidate source, and reference
roles. Their roots must be absolute, normalized, and disjoint. The coordinator
and source roles must share one repository ID and Git common directory. The
reference uses that same repository ID. Velnor may be absent; if present, its
source pin and scratch must both be present, use a separate repository ID and
registered origin value, share the source commit and common directory, and use
disjoint roots. The scratch alone declares a sorted exact-path allowlist.

`registry-validate-v2` reads a registry whose raw SHA-256 is supplied by the
caller. It checks JSON shape, IDs, role relationships, origin syntax, pins,
path normalization, and root disjointness. It does not open or inspect those
roots, verify Git state, read a source-selection receipt, or qualify a source
tree. Its output sets `live_roots_qualified` to `false`. Registry values and
hashes are data; they do not authenticate an actor or isolate a process.

```text
python3 tools/visibility/queue.py registry-validate-v2 \
  --registry <registry.json> --registry-sha256 <caller-pinned-sha256>
python3 tools/visibility/queue.py --root <queue-coordinator-root> registry-check-v2 \
  --registry <registry.json> --registry-sha256 <caller-pinned-sha256> \
  --worktree <queue-coordinator|candidate-source|reference>
python3 tools/visibility/queue.py --root <fixture-root> check-v2
python3 tools/visibility/queue.py --root <fixture-root> render-v2
python3 tools/visibility/queue.py --root <accepted-queue-root> migration-preview \
  --repository-id terminal-components-claude --worktree-id queue-coordinator
```

`registry-check-v2` is a separate read-only observation. The caller supplies
the SHA-256 of the registry's raw bytes. The command requires `--root` to
equal the normalized registered queue-coordinator root, then checks only the
queue coordinator and the one requested live role: `queue-coordinator`,
`candidate-source`, or `reference`. It checks the selected role's root and
common Git directory, worktree membership, fetch and push URLs for `origin`,
branch or detached state, HEAD pin, and clean status. For `reference`, it also
checks that the pinned commit is an ancestor of the existing local
`refs/remotes/origin/visual-baseline` ref. It does not fetch or update that ref.

The output reports selected-role identity match and cleanliness separately.
Identity qualification is true only when the identity checks pass and the
selected worktree is clean. A dirty coordinator or selected source is reported
as unqualified without making a clean, separately selected source appear
dirty. When another role is selected, the output reports the coordinator's
identity and cleanliness separately. It reads the immutable visual tag from
the registered candidate-source root, but that tag check does not qualify an
unselected role. Optional Velnor roots are reported as `not_probed`; selecting
either optional role fails closed.

For every Git probe, the command clears inherited `GIT_*` variables, disables
system and global configuration, sets `GIT_OPTIONAL_LOCKS=0`,
`GIT_NO_LAZY_FETCH=1`, and `GIT_NO_REPLACE_OBJECTS=1`, and passes
`--no-replace-objects`. It performs no fetch, checkout, ref update, maintenance,
or other Git write. The registry hash pins input bytes but does not
authenticate the caller. `writable` is policy data; this command gives no
writer capability. Its identity result does not select a source or qualify a
build, test, binary, or product result. Schema validation alone remains
static and continues to report `live_roots_qualified: false`.

Schema-v2 `check-v2` validates records and confirms their generated view is
current. `render-v2` prints the deterministic v2 view, including each task's
repository and worktree IDs. V2 active path conflicts are checked within one
repository across its worktrees; equal path strings in different repositories
are independent. Global priority and shared-API-owner validation remain in
force. These record commands do not bind task IDs to live roots or authorize
instruction discovery, digesting, transitions, or writes. This increment adds
no v2 accept, transition, handoff, amendment, or migration-write command.

`migration-preview` reads the checked-in schema-v1 record and view, verifies
that they agree, and applies an explicit `--repository-id` and `--worktree-id`
mapping to every task and prior claim snapshot. This uniform mapping matches
the schema-v1 queue's single repository root. It changes the proposed record's
schema version and adds only the two identity fields. It preserves the queue
revision, every old task and history value, and legacy review digests without
reinterpreting them. The JSON output includes raw source-file hashes, proposed
records, the identity-map schema, and the deterministic v2 view. Its source
object reports the observed queue revision and raw SHA-256 values for both
`tasks.json` and `WORK_QUEUE.md`; `writes_files` is `false`. The command does
not acquire the queue lock, write files, or advance the accepted revision.

The selected candidate source commit is already accepted as
`cee7e2e307514e49a70d8b8fcb028923ccdc7322` (fixed-pair fetch-receipt prefix
`6645a0d4`, source-audit prefix `43a57d70`). This selection does not qualify a
clean detached source worktree or a live registry. A v2 registry example or
synthetic test fixture is not that qualification.

## Schema-v1 registry check

The caller supplies the expected SHA-256 of the raw registry bytes. The hash is
not read from the registry itself:

```text
python3 tools/visibility/queue.py --root <candidate-root> registry-check \
  --registry <local-registry.json> \
  --registry-sha256 <caller-pinned-64-character-sha256> \
  --worktree candidate
```

The command reads the registry as a regular file without following symlinks
and checks its caller-pinned raw-byte digest before parsing. It requires the
registry's `candidate` root to match `--root` exactly. The registry file must
be outside every registered worktree root.

The command verifies the candidate authority worktree and the selected
worktree, checking the candidate once when both IDs are `candidate`. It opens
their root and Git common-directory paths without following symlink
components. It checks Git top-level, worktree membership, exact `origin` fetch
and push URLs, branch or detached mode, pinned HEAD when present, and clean
status. A scratch check also verifies its separate source-pin root and matching
commit.

Schema-v1 `registry-check` remains strict: a dirty candidate, selected
worktree, or recursively checked source pin fails the command. Only
`registry-check-v2` records cleanliness separately; a dirty role remains
unqualified.

The command resolves the annotated `visual-baseline` tag and its peeled commit
in the candidate repository, or in the selected reference worktree. Both
object IDs must match the immutable values in code and the caller-pinned
registry. Git queries ignore inherited `GIT_*` repository overrides and
replacement refs. The JSON result includes the selected worktree identity,
observed HEAD, visual tag identities, authority root, and caller-pinned raw
registry SHA-256 for later operation binding.

This is a read-only preflight. Recorded identities and hashes are data supplied
by the integrator; they do not authenticate a person or isolate a process.
Future claim operations will use this registry under the one candidate queue
lock and will add their own accepted-record, review, and compare-and-swap
checks.

## Queue lock file

Queue mutations and explicit view repair serialize on a per-user advisory lock
whose key is derived from the absolute task-record path. The lock leaf is
opened relative to the already-open private lock-directory descriptor. The
queue first tries exclusive creation with
`O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC`. Only `EEXIST` selects a
second open of the existing leaf with `O_RDWR | O_NOFOLLOW | O_CLOEXEC` and
without `O_CREAT`; any other error, or a missing leaf during that second open,
fails closed without retrying creation.

Before and after acquiring `flock`, the queue checks that the opened descriptor
is a regular file owned by the current user with no group or other permissions,
and that a no-follow lookup of the leaf through the same directory descriptor
names the same device and inode with the same type, owner, and mode. A
disappeared or replaced leaf fails closed. The lock leaf remains in place after
release so later queue processes continue to lock the same inode. These checks
support coordination among cooperating processes and detect observed path
replacement; they do not authenticate against a hostile process running as the
same user.

## Staged compatibility

Schema-v1 records remain readable by `check` and `render`; their v1 review
digest is not reinterpreted. This increment can read and render schema-v2
records and preview a v1-to-v2 identity mapping, but it does not persist the
preview. V2 verified records retain their legacy review digests as opaque
history. Any future v2 review digest must use a separate context-bound format
with explicit deleted-file tombstones. The migration remains deferred until
`status.py` can read v2 records and a separately authorized preflight confirms
the migration inputs.

V2 pending claims are plans only. They will contain no owner, reviewer, claim
token, expiry, task state, or accepted revision. A pending path list grants no
instruction packet, digest operation, transition, handoff, or write claim.
Promotion and reviewer/scope amendment will use separate compare-and-swap
operations after their release, review, registry, and overlap conditions pass.
