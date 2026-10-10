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

Schema v2 remains a read-only record format in this increment and has no v2
mutation command. To make migration lossless, it preserves schema-v1 task and
claim-snapshot `branch_scopes` where present. It continues to validate those
legacy grants and expands them in repository- and branch-scoped conflict checks;
it does not add a schema-v2 amendment operation or reinterpret the grants.

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

## Schema-v1 review-subject binding

The schema-v1 queue has a narrow compare-and-swap operation for recording the
subject that an assigned reviewer must inspect:

```text
python3 tools/visibility/queue.py --root <candidate-root> bind-review-subject <work-id> \
  --expected-revision <queue-revision> \
  --claim-token <current-claim-token> \
  --expected-subject-sha256 <caller-observed-scope-digest>
```

The command uses the existing queue mutation lock and source/view write path. It
requires the current schema-v1 queue revision, the current task claim token,
task state `review`, and no active review-subject binding. While holding the
queue lock, it reads the accepted task record, verifies the generated view and
allowed filesystem scopes, computes the digest of the task's current allowed
paths, and requires that digest to equal the caller's expected value. On
success it stores that digest as `review_subject_sha256`, records the bind in
task evidence, increments `queue_revision`, and regenerates the view. A second
bind is rejected until the task leaves review or is handed off.

The digest binds the observed allowed-path names and regular-file bytes at the
time of the command. The queue lock does not lock task-owner file writes, so it
is cooperative coordination rather than isolation or authentication. The
claim token, reviewer name, and expected digest are queue data supplied by the
integrator; they do not prove a person's identity. The final `verified`
transition still requires the assigned reviewer's external review record,
checks its raw-file SHA-256 and subject digest, and recomputes the current
allowed-path digest. A changed scope fails verification.

Newly accepted claims cannot prepopulate `review_subject_sha256`. Leaving
`review` for `in_progress` or `blocked` archives any active digest in task
evidence and clears the active field. Returning from `blocked` or
`in_progress` to `review` does not restore it; the current subject must be
bound again. Some older records can already have a digest while in
`blocked` or `in_progress`. When such a record enters `review`, the transition
archives and clears the stale digest and requires a fresh binding. A handoff
similarly archives the digest in the prior claim's evidence before recording
the prior snapshot, then clears the active field for the new claim. These rules
preserve old verified records and prevent a reviewer receipt for a prior edit
from verifying a changed subject.

This operation is schema-v1 only. It does not perform registry qualification,
schema migration, scope amendment, or task promotion. V2 records remain
read-only, and their legacy review digests are not reinterpreted.

## Schema-v1 evidence append

Use `append-evidence` to add one evidence string to an active claim without
changing its state or claim details:

```text
python3 tools/visibility/queue.py --root <candidate-root> append-evidence <work-id> \
  --expected-revision <queue-revision> \
  --claim-token <current-claim-token> \
  --evidence <evidence-text>
```

The command uses the existing queue mutation lock and source/view write path.
It requires the current queue revision, the matching claim token, an active
claim, nonempty evidence that is not already present, and the highest open
priority. It appends the evidence string and increments `queue_revision`.
The only record fields that change are the queue revision and the target
claim's evidence list. The command does not change task state, owner, reviewer,
claim token, scope, base, expiry, dependencies, or accepted revision. It does
not accept, transition, hand off, renew, or verify a claim.

The claim token is queue data supplied by the caller. It provides a compare
check against the current claim, but it does not authenticate a person's
identity. An evidence append records a supplied string; it does not validate
the evidence or imply task completion.

## Schema-v1 branch-scope amendment

The accepted VIS-02 schema-v1 assignment can receive its reviewed reference
branch grant through the queue's existing revision-CAS mutation path:

```text
python3 tools/visibility/queue.py amend-branch-scope <work-id> \
  --expected-revision <queue-revision> \
  --claim-token <current-claim-token> \
  --record <reviewed-amendment.json>
```

The request is an operation, not a replacement `tasks.json`. It binds the
current owner, reviewer, priority, candidate branch, candidate base, and
candidate paths in `expected_primary`; supplies the replacement
`branch_scopes` list, exact user authorization text, and the accepted plan and
independent-review SHA-256 references. The current VIS-10 evidence must contain
that accepted plan/review pair. The queue reads both referenced artifacts,
checks their raw SHA-256 values, and extracts the exact primary assignment,
reference branch grant, and authorization text from the plan's reviewed JSON
request. The caller cannot choose another task, primary assignment, branch
base, or path. References and authorization are queue data; they do not
authenticate a person. This local operation also requires the absolute artifact
paths recorded in VIS-10 evidence to remain readable; it does not claim fresh
clone portability. If these records are archived in-repository later, retain
the exact artifact bytes and recorded SHA-256 values. The request has exactly
these fields:

```json
{
  "expected_primary": {
    "owner": "/root/rust_test_infrastructure",
    "reviewer": "/root/controls_wrapper_review_luna",
    "priority": "P0",
    "branch": "termrock-implementation",
    "base_sha": "cc3ce8f6ac149aaee406047c5180d1a658d6bb9c",
    "allowed_paths": ["crates/termrock-e2e/**"]
  },
  "branch_scopes": [
    {
      "branch": "visual-baseline",
      "base_sha": "b274dd57f4dd078ade6e424d546d83efbd2e8526",
      "allowed_paths": ["crates/termrock-e2e/**"]
    }
  ],
  "authorization_evidence": "User authorized /root/rust_test_infrastructure to install and execute the identical shared Termrock E2E package on refs/heads/visual-baseline at b274dd57f4dd078ade6e424d546d83efbd2e8526, limited to crates/termrock-e2e/**; keep the frozen visual-baseline tag and release unchanged.",
  "plan_sha256": "<accepted-plan-sha256>",
  "review_sha256": "<independent-review-sha256>"
}
```

Setting `branch_scopes` to `[]` removes the additional grant through another
CAS. The operation appends both the authorization statement and an evidence
entry containing the previous and replacement grants, queue revision, fresh
reference tip, and accepted plan/review references.

This increment keeps the existing VIS-02 request shape and adds exact task
binding for VIS-01 and VIS-11. The accepted VIS-02 plan may omit `work_id`; that
legacy shape is interpreted as VIS-02 only. A VIS-01 or VIS-11 plan must include
its exact `work_id`. The target task's owner, reviewer, priority, candidate
branch, base, and primary paths still must match the accepted plan byte for
byte. The operation accepts only the plan's complete reference grant or `[]`
to remove it, and it preserves the task's primary assignment and paths.

VIS-01 reference grants contain exactly 89 normalized literal paths. VIS-11
grants contain normalized literal paths only; their accepted plan must freeze
the generated workflow outputs and shards, Velnor configuration and tool pins,
and any other required producer leaves. `/**` scopes and other globs are
rejected for both tasks. The implementation does not invent or preaccept an
S6 path list: each VIS-11 grant needs a producer-frozen plan/review pair bound
in VIS-10 evidence. Tests use synthetic plan fixtures and do not authorize a
real grant. New grants use the reviewed current `visual-baseline` base SHA; the
locked `ls-remote` check fails closed if that ref moves before the CAS. This
operation reads the reference ref and never updates it or the frozen tag.

Old v1 tasks and handoff snapshots without `branch_scopes` remain valid. New
handoff snapshots preserve the field when present. New `accept` requests cannot
preseed it. The generated view prints each added branch, base, and path.
Candidate review-subject digests continue to cover the primary scope only;
amendments are rejected while a subject is active, and a candidate-only digest
cannot be bound or used to verify a task while an additional branch scope is
active. The reference grant does not imply parity or verification.

A legacy schema-v1 primary grant on `visual-baseline` is accepted only with the
same test-package path so the overlap scan can account for older active claims.
New claims still start on `termrock-implementation`.

The operation checks revision, work ID, token, the accepted plan and review
artifacts, current primary assignment, replacement/no-op state, authorization,
and branch-scoped path conflicts before starting a remote process. While still
holding the existing queue lock, it then checks the exact
`refs/heads/visual-baseline` advertisement using `git ls-remote --exit-code
--refs origin refs/heads/visual-baseline`. The child runs without a shell, in a
dedicated process group, with Git repository/config overrides cleared, system
and global Git configuration disabled, and terminal/credential prompts
disabled. For the canonical SSH origin `git@github.com:tailrocks/terminal-components-claude.git`,
the command also supplies process-scoped `-c credential.helper=` and an exact
`url.https://github.com/tailrocks/terminal-components-claude.git.insteadOf=git@github.com:tailrocks/terminal-components-claude.git`
rewrite to reach the same repository over HTTPS. These options do not edit the
local origin or any global/system Git configuration; local bare origins used
by fixtures do not match this rewrite and continue to work unchanged. The probe
has a 10-second monotonic deadline and a 4096-byte combined stdout/stderr cap.
It accepts only one byte-exact lowercase SHA-1 plus the exact branch ref line
and requires empty stderr and exit status zero. Timeout, overflow,
malformed/duplicate output, wrong tip, or cleanup failure kills and reaps the
child, closes its pipes, writes neither canonical queue file, and releases the
lock. The exact heads-only query excludes the same-name tag; this operation
never queries or writes tag refs, fetches, checks out, pushes, or updates refs.

Conflict checks expand every active task into its legacy primary grant and any
optional branch scopes. Same-branch overlapping paths conflict across all
active states even after expiry; grants on different branches may name the
same relative package path. A base SHA does not permit overlapping grants on
the same branch. Failed compare-and-swap and probe cases leave the task record
and generated view byte-for-byte unchanged.
