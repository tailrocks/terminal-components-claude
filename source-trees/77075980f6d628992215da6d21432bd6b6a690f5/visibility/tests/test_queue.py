import copy
import hashlib
import importlib.util
import json
import tempfile
import threading
import unittest
from contextlib import contextmanager
from unittest import mock
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
QUEUE_SCRIPT = ROOT / "tools/visibility/queue.py"
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"

spec = importlib.util.spec_from_file_location("visibility_queue", QUEUE_SCRIPT)
queue = importlib.util.module_from_spec(spec)
spec.loader.exec_module(queue)


NOW = datetime(2026, 10, 8, 12, 0, tzinfo=timezone.utc)
BASE = "a" * 40


def task(
    work_id="VIS-01",
    *,
    priority="P0",
    state="in_progress",
    paths=None,
    owner="/builder",
    reviewer="/reviewer",
    dependencies=None,
    token=None,
    expiry="2026-10-09T00:00:00Z",
    api_decisions=None,
    review_subject_sha256=None,
):
    result = {
        "work_id": work_id,
        "requirement_ids": ["VIS-P02"],
        "priority": priority,
        "priority_reason": "Visibility prerequisite.",
        "dependencies": list(dependencies or []),
        "branch": "termrock-implementation",
        "base_sha": BASE,
        "allowed_paths": list(paths or ["tools/visibility/status.py"]),
        "owner": owner,
        "reviewer": reviewer,
        "claim_token": token or "claim-{}".format(work_id.lower()),
        "expiry": expiry,
        "state": state,
        "evidence": ["Accepted scope."],
        "handoff": None,
        "accepted_queue_revision": 1,
    }
    if api_decisions is not None:
        result["shared_api_decisions"] = api_decisions
    if review_subject_sha256 is not None:
        result["review_subject_sha256"] = review_subject_sha256
    return result


def records(*tasks, revision=5):
    return {
        "schema_version": 1,
        "queue_revision": revision,
        "accepted_by": "/root",
        "acceptance_mode": "single-integrator",
        "accepted_at": "2026-10-08T02:55:05.893785+00:00",
        "tasks": list(tasks),
    }


def new_claim(work_id="VIS-06", *, priority="P0", paths=None, dependencies=None, token=None, state="claimed"):
    return {
        "work_id": work_id,
        "requirement_ids": ["VIS-P03"],
        "priority": priority,
        "priority_reason": "Independent P0 control work.",
        "dependencies": list(dependencies or []),
        "branch": "termrock-implementation",
        "base_sha": BASE,
        "allowed_paths": list(paths or ["tools/visibility/new.py"]),
        "owner": "/new-builder",
        "reviewer": "/new-reviewer",
        "claim_token": token or "claim-{}".format(work_id.lower()),
        "expiry": "2026-10-10T00:00:00Z",
        "state": state,
        "evidence": ["Plan approved."],
        "handoff": None,
    }


def handoff(*, owner_released=False, coordinator_stopped=True):
    result = {
        "handoff_at": "2026-10-08T12:00:00Z",
        "handoff_by": "/root",
        "from_owner": "/builder",
        "to_owner": "/new-builder",
        "owner_released": owner_released,
        "coordinator_verified_stopped": coordinator_stopped,
        "worktree": "/tmp/worktree-123",
        "git_status": " M file.rs; ?? new.md",
        "diff_sha256": "b" * 64,
        "changed_paths": ["file.rs", "new.md"],
        "unpublished_commits": ["1234567"],
        "checks": [{"command": "python -m unittest", "result": "1 failure"}],
        "next_action": "Preserve the diff and reproduce the one failure.",
    }
    if coordinator_stopped:
        result["stop_evidence"] = "Coordinator confirmed process exit and captured the worktree."
    return result


def approved_review(reviewer="/reviewer", subject_sha256="c" * 64):
    return {
        "reviewer": reviewer,
        "decision": "approved",
        "subject_sha256": subject_sha256,
        "reviewed_at": "2026-10-08T12:00:00Z",
        "evidence": "Reviewed all changed paths at the frozen diff digest.",
    }


def verified_task_review(task_value):
    review = approved_review(task_value["reviewer"])
    raw = queue._canonical_json(review)
    task_value["review_subject_sha256"] = review["subject_sha256"]
    task_value["review_record"] = review
    task_value["review_record_sha256"] = hashlib.sha256(raw).hexdigest()


class QueueRecordTests(unittest.TestCase):
    def test_current_accepted_records_validate(self):
        current = json.loads(TASKS_PATH.read_text(encoding="utf-8"))
        queue.validate_records(current)

    def test_schema_and_sha_fields_are_strict(self):
        invalid = records(task())
        invalid["schema_version"] = True
        with self.assertRaisesRegex(queue.QueueError, "schema"):
            queue.validate_records(invalid)

        invalid = records(task())
        invalid["tasks"][0]["base_sha"] = "A" * 40
        with self.assertRaisesRegex(queue.QueueError, "base_sha"):
            queue.validate_records(invalid)

    def test_json_rejects_duplicate_keys_and_nonfinite_numbers(self):
        with self.assertRaisesRegex(queue.QueueError, "repeats key"):
            queue.strict_json_loads('{"schema_version":1,"schema_version":1}')
        with self.assertRaisesRegex(queue.QueueError, "non-finite"):
            queue.strict_json_loads('{"value":NaN}')
        with self.assertRaisesRegex(queue.QueueError, "unknown fields"):
            queue.validate_records({**records(task()), "unexpected": True})
        with self.assertRaisesRegex(queue.QueueError, "nonempty"):
            queue.validate_records(records())

    def test_timestamp_requires_real_canonical_utc_rfc3339(self):
        for value in (
            "2026-02-30T12:00:00Z",
            "2026-10-08t12:00:00Z",
            "2026-10-08T12:00Z",
            "2026-10-08T12:00:00+01:00",
            "2026-10-08T12:00:00.1234567Z",
        ):
            with self.subTest(value=value), self.assertRaises(queue.QueueError):
                queue.parse_time(value, "test timestamp")

    def test_requirement_trace_and_evidence_cannot_be_empty(self):
        invalid = records(task())
        invalid["tasks"][0]["requirement_ids"] = []
        with self.assertRaisesRegex(queue.QueueError, "requirement_ids"):
            queue.validate_records(invalid)

        invalid = records(task())
        invalid["tasks"][0]["evidence"] = []
        with self.assertRaisesRegex(queue.QueueError, "evidence"):
            queue.validate_records(invalid)

    def test_duplicate_work_id_and_claim_token_are_rejected(self):
        first = task("VIS-01")
        with self.assertRaisesRegex(queue.QueueError, "duplicate work ID"):
            queue.validate_records(records(first, copy.deepcopy(first)))

        second = task("VIS-02", paths=["crates/other.rs"], token=first["claim_token"])
        with self.assertRaisesRegex(queue.QueueError, "duplicate claim token"):
            queue.validate_records(records(first, second))

    def test_task_requirements_can_be_shared_but_api_owner_cannot(self):
        first = task("VIS-01", api_decisions=["grid.sort"])
        second = task("VIS-02", paths=["crates/other.rs"], owner="/other", api_decisions=["grid.sort"])
        with self.assertRaisesRegex(queue.QueueError, "shared API decision grid.sort"):
            queue.validate_records(records(first, second))

        second["owner"] = first["owner"]
        queue.validate_records(records(first, second))

    def test_dependencies_must_exist_and_be_acyclic(self):
        first = task("VIS-01", state="blocked", dependencies=["VIS-02"])
        second = task("VIS-02", state="blocked", paths=["crates/other.rs"], dependencies=["VIS-01"])
        with self.assertRaisesRegex(queue.QueueError, "dependency cycle"):
            queue.validate_records(records(first, second))

        with self.assertRaisesRegex(queue.QueueError, "unknown work ID"):
            queue.validate_records(records(task("VIS-01", dependencies=["VIS-09"])))

    def test_path_claim_grammar_and_overlap_are_conservative(self):
        self.assertTrue(queue.scopes_overlap("src/a.rs", "src/a.rs"))
        self.assertTrue(queue.scopes_overlap("src/**", "src/nested/a.rs"))
        self.assertTrue(queue.scopes_overlap("src/**", "src/nested/**"))
        self.assertFalse(queue.scopes_overlap("src/a/**", "src/ab/**"))
        self.assertFalse(queue.scopes_overlap("src/a.rs", "src/b.rs"))
        for invalid in ("../escape", "/absolute", "src/**/a.rs", "src/*.rs", "src//a.rs"):
            with self.subTest(invalid=invalid):
                with self.assertRaises(queue.QueueError):
                    queue.normalize_scope(invalid)

    def test_active_path_overlap_is_rejected_but_verified_paths_are_released(self):
        first = task("VIS-01", paths=["tools/visibility/**"])
        second = task("VIS-02", paths=["tools/visibility/tests/test_queue.py"])
        with self.assertRaisesRegex(queue.QueueError, "active path conflict"):
            queue.validate_records(records(first, second))

        first["state"] = "verified"
        verified_task_review(first)
        queue.validate_records(records(first, second))

    def test_render_is_deterministic_and_priority_sorted(self):
        p1 = task("VIS-02", priority="P1", state="ready", paths=["crates/one.rs"])
        p0 = task("VIS-01", paths=["tools/one.py"])
        data = records(p1, p0)
        first = queue.render_queue(data)
        second = queue.render_queue(data)
        self.assertEqual(first, second)
        self.assertLess(first.index("VIS-01"), first.index("VIS-02"))
        self.assertIn("Highest open priority: **P0**", first)
        self.assertIn("Expiry does not release an active claim", first)


class ClaimOperationTests(unittest.TestCase):
    def test_accept_allows_parallel_same_priority_and_increments_revision(self):
        before = records(task())
        original = copy.deepcopy(before)
        after = queue.accept_record(
            before, new_claim(), 5,
            current_branch="termrock-implementation", current_head=BASE, now=NOW,
        )
        self.assertEqual(6, after["queue_revision"])
        self.assertEqual(original, before)
        self.assertEqual(2, len(after["tasks"]))
        self.assertEqual(6, after["tasks"][1]["accepted_queue_revision"])

    def test_accept_rejects_stale_revision_without_mutation(self):
        before = records(task())
        snapshot = copy.deepcopy(before)
        with self.assertRaisesRegex(queue.QueueError, "stale queue revision"):
            queue.accept_record(
                before, new_claim(), 4,
                current_branch="termrock-implementation", current_head=BASE, now=NOW,
            )
        self.assertEqual(snapshot, before)

    def test_accept_rejects_wrong_branch_or_base(self):
        before = records(task())
        with self.assertRaisesRegex(queue.QueueError, "only allowed"):
            queue.accept_record(
                before, new_claim(), 5,
                current_branch="main", current_head=BASE, now=NOW,
            )
        with self.assertRaisesRegex(queue.QueueError, "base SHA"):
            queue.accept_record(
                before, new_claim(), 5,
                current_branch="termrock-implementation", current_head="c" * 40, now=NOW,
            )

    def test_accept_rejects_lower_priority_until_higher_priority_closes(self):
        before = records(task())
        with self.assertRaisesRegex(queue.QueueError, "lower priority P1"):
            queue.accept_record(
                before, new_claim(priority="P1"), 5,
                current_branch="termrock-implementation", current_head=BASE, now=NOW,
            )

    def test_accept_rejects_overlapping_active_scope_even_after_expiry(self):
        before = records(task(expiry="2026-10-08T11:00:00Z"))
        candidate = new_claim(paths=["tools/visibility/**"])
        with self.assertRaisesRegex(queue.QueueError, "active path conflict"):
            queue.accept_record(
                before, candidate, 5,
                current_branch="termrock-implementation", current_head=BASE, now=NOW,
            )

    def test_accept_rejects_dependency_until_verified(self):
        before = records(task())
        with self.assertRaisesRegex(queue.QueueError, "must start blocked"):
            queue.accept_record(
                before, new_claim(dependencies=["VIS-01"]), 5,
                current_branch="termrock-implementation", current_head=BASE, now=NOW,
            )
        blocked = queue.accept_record(
            before, new_claim(dependencies=["VIS-01"], state="blocked"), 5,
            current_branch="termrock-implementation", current_head=BASE, now=NOW,
        )
        self.assertEqual("blocked", blocked["tasks"][1]["state"])
        with self.assertRaisesRegex(queue.QueueError, "until dependencies are verified"):
            queue.transition_record(blocked, "VIS-06", "in_progress", "Start work.", 6)

        before["tasks"][0]["state"] = "verified"
        verified_task_review(before["tasks"][0])
        accepted = queue.accept_record(
            before, new_claim(dependencies=["VIS-01"]), 5,
            current_branch="termrock-implementation", current_head=BASE, now=NOW,
        )
        self.assertEqual(2, len(accepted["tasks"]))

    def test_transition_requires_valid_edge_and_evidence(self):
        before = records(task())
        after = queue.transition_record(before, "VIS-01", "review", "Targeted checks passed.", 5)
        self.assertEqual("review", after["tasks"][0]["state"])
        self.assertEqual(6, after["queue_revision"])
        self.assertEqual(1, len(before["tasks"][0]["evidence"]))
        with self.assertRaisesRegex(queue.QueueError, "invalid task state transition"):
            queue.transition_record(before, "VIS-01", "claimed", "Try again.", 5)
        with self.assertRaisesRegex(queue.QueueError, "evidence"):
            queue.transition_record(before, "VIS-01", "review", "", 5)

    def test_verified_transition_binds_external_review_and_expected_scope_subject(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            subject_file = root / "src/example.rs"
            subject_file.parent.mkdir(parents=True)
            subject_file.write_text("reviewed source\n", encoding="utf-8")
            expected_subject = queue.scope_digest(root, ["src/example.rs"])
            before = records(task(
                state="review", paths=["src/example.rs"],
                review_subject_sha256=expected_subject,
            ))
            review = approved_review(subject_sha256=expected_subject)
            raw = queue._canonical_json(review)
            raw_sha256 = hashlib.sha256(raw).hexdigest()
            missing_subject = records(task(state="review", paths=["src/example.rs"]))
            with self.assertRaisesRegex(queue.QueueError, "review_subject_sha256"):
                queue.transition_record(
                    missing_subject, "VIS-01", "verified", "Reviewed.", 5,
                    review_record_raw=raw, review_record_sha256=raw_sha256,
                    current_subject_sha256=expected_subject,
                )
            with self.assertRaisesRegex(queue.QueueError, "raw-file SHA-256"):
                queue.transition_record(
                    before, "VIS-01", "verified", "Reviewed.", 5,
                    review_record_raw=raw, review_record_sha256="d" * 64,
                    current_subject_sha256=expected_subject,
                )
            verified = queue.transition_record(
                before, "VIS-01", "verified", "Required evidence is retained.", 5,
                review_record_raw=raw, review_record_sha256=raw_sha256,
                current_subject_sha256=queue.scope_digest(root, ["src/example.rs"]),
            )
            self.assertEqual("verified", verified["tasks"][0]["state"])
            self.assertEqual(review, verified["tasks"][0]["review_record"])
            self.assertEqual(raw_sha256, verified["tasks"][0]["review_record_sha256"])

            subject_file.write_text("changed after review\n", encoding="utf-8")
            changed_subject = queue.scope_digest(root, ["src/example.rs"])
            with self.assertRaisesRegex(queue.QueueError, "current allowed-path digest"):
                queue.transition_record(
                    before, "VIS-01", "verified", "Reviewed.", 5,
                    review_record_raw=raw, review_record_sha256=raw_sha256,
                    current_subject_sha256=queue.scope_digest(root, ["src/example.rs"]),
                )
            self.assertNotEqual(expected_subject, changed_subject)

            wrong_subject = approved_review(subject_sha256="f" * 64)
            wrong_raw = queue._canonical_json(wrong_subject)
            with self.assertRaisesRegex(queue.QueueError, "review subject does not match"):
                queue.transition_record(
                    before, "VIS-01", "verified", "Reviewed.", 5,
                    review_record_raw=wrong_raw,
                    review_record_sha256=hashlib.sha256(wrong_raw).hexdigest(),
                    current_subject_sha256=expected_subject,
                )
            wrong_reviewer = approved_review("/owner", expected_subject)
            wrong_reviewer_raw = queue._canonical_json(wrong_reviewer)
            with self.assertRaisesRegex(queue.QueueError, "identify the assigned independent reviewer"):
                queue.transition_record(
                    before, "VIS-01", "verified", "Reviewed.", 5,
                    review_record_raw=wrong_reviewer_raw,
                    review_record_sha256=hashlib.sha256(wrong_reviewer_raw).hexdigest(),
                    current_subject_sha256=expected_subject,
                )

    def test_handoff_requires_safe_record_and_preserves_old_claim(self):
        before = records(task(expiry="2026-10-08T11:00:00Z"))
        after = queue.reassign_record(
            before, "VIS-01",
            {"owner": "/new-builder", "reviewer": "/new-reviewer",
             "claim_token": "claim-vis-01-new", "expiry": "2026-10-10T00:00:00Z"},
            handoff(), 5, now=NOW,
        )
        current = after["tasks"][0]
        self.assertEqual("/new-builder", current["owner"])
        self.assertEqual("claimed", current["state"])
        self.assertEqual("claim-vis-01", current["claim_history"][0]["claim"]["claim_token"])
        self.assertEqual("/builder", current["claim_history"][0]["handoff"]["from_owner"])
        self.assertEqual(6, current["accepted_queue_revision"])

        corrupted = copy.deepcopy(after)
        corrupted["tasks"][0]["claim_history"][0]["handoff"]["to_owner"] = "/other"
        with self.assertRaisesRegex(queue.QueueError, "target does not match next claim owner"):
            queue.validate_records(corrupted)

    def test_expiry_alone_does_not_allow_reassignment(self):
        before = records(task(expiry="2026-10-08T11:00:00Z"))
        unsafe = handoff(coordinator_stopped=False)
        with self.assertRaisesRegex(queue.QueueError, "owner release or coordinator-verified stop"):
            queue.reassign_record(
                before, "VIS-01",
                {"owner": "/new-builder", "reviewer": "/new-reviewer",
                 "claim_token": "claim-vis-01-new", "expiry": "2026-10-10T00:00:00Z"},
                unsafe, 5, now=NOW,
            )

    def test_unexpired_handoff_requires_explicit_safe_release(self):
        before = records(task(expiry="2026-10-09T00:00:00Z"))
        new_owner = {
            "owner": "/new-builder", "reviewer": "/new-reviewer",
            "claim_token": "claim-vis-01-new", "expiry": "2026-10-10T00:00:00Z",
        }
        with self.assertRaisesRegex(queue.QueueError, "owner release or coordinator-verified stop"):
            queue.reassign_record(
                before, "VIS-01", new_owner,
                handoff(owner_released=False, coordinator_stopped=False), 5, now=NOW,
            )
        updated = queue.reassign_record(
            before, "VIS-01", new_owner,
            handoff(owner_released=True, coordinator_stopped=False), 5, now=NOW,
        )
        self.assertEqual("/new-builder", updated["tasks"][0]["owner"])


class FileAndInstructionTests(unittest.TestCase):
    def test_check_rejects_stale_markdown_and_explicit_render_repairs_only_view(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "tasks.json"
            view = root / "WORK_QUEUE.md"
            source.write_bytes(json.dumps(records(task())).encode("utf-8"))
            view.write_text("stale\n", encoding="utf-8")
            with self.assertRaisesRegex(queue.QueueError, "is stale"):
                queue.check_rendered_queue(source, view)
            original = source.read_bytes()
            queue._atomic_write(view, queue.render_queue(json.loads(original)).encode("utf-8"))
            queue.check_rendered_queue(source, view)
            self.assertEqual(original, source.read_bytes())

    def test_repair_render_reads_latest_revision_inside_lock(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "tasks.json"
            view = root / "WORK_QUEUE.md"
            initial = records(task())
            source.write_bytes(queue._canonical_json(initial))
            view.write_text("stale\n", encoding="utf-8")
            read_started = threading.Event()
            repair_done = threading.Event()
            errors = []
            original_read = queue.read_records

            def observed_read(path):
                read_started.set()
                return original_read(path)

            def repair():
                try:
                    queue._repair_render(
                        source, view, repo_root=root,
                        subject_reader=lambda: ("termrock-implementation", BASE),
                    )
                except BaseException as error:
                    errors.append(error)
                finally:
                    repair_done.set()

            with mock.patch.object(queue, "read_records", observed_read):
                with queue._exclusive_lock(source):
                    worker = threading.Thread(target=repair)
                    worker.start()
                    self.assertFalse(read_started.wait(0.1))
                    latest = records(task(), revision=6)
                    queue._atomic_write(source, queue._canonical_json(latest))
                self.assertTrue(repair_done.wait(2))
                worker.join()
            self.assertEqual([], errors)
            self.assertIn("Queue revision: 6.", view.read_text(encoding="utf-8"))
            queue.check_rendered_queue(source, view)

    def test_parent_and_leaf_symlinks_cannot_redirect_state_io(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            outside = root / "outside"
            outside.mkdir()
            target = outside / "data.json"
            target.write_text("unchanged\n", encoding="utf-8")
            state_root = root / "state"
            state_root.mkdir()
            (state_root / "redirect").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(queue.QueueError, "safely open parent"):
                queue._safe_read(state_root / "redirect/data.json", "test state")
            with self.assertRaisesRegex(queue.QueueError, "safely open parent"):
                queue._atomic_write(state_root / "redirect/data.json", b"redirected\n")

            (state_root / "leaf-link").symlink_to(target)
            with self.assertRaisesRegex(queue.QueueError, "safely open"):
                queue._safe_read(state_root / "leaf-link", "test state")
            with self.assertRaisesRegex(queue.QueueError, "non-symlink"):
                queue._atomic_write(state_root / "leaf-link", b"redirected\n")
            self.assertEqual("unchanged\n", target.read_text(encoding="utf-8"))

            root_link = root / "state-link"
            root_link.symlink_to(state_root, target_is_directory=True)
            with self.assertRaisesRegex(queue.QueueError, "safely open parent"):
                queue._safe_read(root_link / "data.json", "test state")

    def test_active_claim_scopes_reject_exact_and_recursive_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "tools/visibility").mkdir(parents=True)
            outside = root / "outside"
            outside.write_text("outside\n", encoding="utf-8")
            (root / "tools/visibility/status.py").symlink_to(outside)
            with self.assertRaisesRegex(queue.QueueError, "write claim leaf is a symlink"):
                queue.validate_filesystem_scopes(root, records(task()))

            (root / "tools/visibility/status.py").unlink()
            (root / "tools/owned/nested").mkdir(parents=True)
            (root / "tools/owned/nested/escape").symlink_to(outside)
            scoped = records(task(paths=["tools/owned/**"]))
            with self.assertRaisesRegex(queue.QueueError, "write claim scope contains a symlink"):
                queue.validate_filesystem_scopes(root, scoped)

            (root / "tools/owned/nested/escape").unlink()
            queue.validate_filesystem_scopes(root, scoped)

    def test_revision_cas_persists_source_and_view_together(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "tasks.json"
            view = root / "WORK_QUEUE.md"
            initial = records(task())
            source.write_bytes(queue._canonical_json(initial))
            view.write_text(queue.render_queue(initial), encoding="utf-8")
            updated = queue._mutate(
                5,
                lambda current, branch, head: queue.accept_record(
                    current, new_claim(), 5,
                    current_branch=branch, current_head=head, now=NOW,
                ),
                tasks_path=source,
                queue_path=view,
                repo_root=root,
                subject_reader=lambda: ("termrock-implementation", BASE),
            )
            saved = json.loads(source.read_text(encoding="utf-8"))
            self.assertEqual(6, saved["queue_revision"])
            self.assertEqual(updated, saved)
            self.assertEqual(queue.render_queue(saved), view.read_text(encoding="utf-8"))

    def test_failed_revision_cas_does_not_write_either_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "tasks.json"
            view = root / "WORK_QUEUE.md"
            initial = records(task())
            source.write_bytes(queue._canonical_json(initial))
            view.write_text(queue.render_queue(initial), encoding="utf-8")
            source_before, view_before = source.read_bytes(), view.read_bytes()
            with self.assertRaisesRegex(queue.QueueError, "stale queue revision"):
                queue._mutate(
                    4,
                    lambda current, branch, head: queue.accept_record(
                        current, new_claim(), 4,
                        current_branch=branch, current_head=head, now=NOW,
                    ),
                    tasks_path=source,
                    queue_path=view,
                    repo_root=root,
                    subject_reader=lambda: ("termrock-implementation", BASE),
                )
            self.assertEqual(source_before, source.read_bytes())
            self.assertEqual(view_before, view.read_bytes())

    def test_mutation_requires_implementation_branch_without_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "tasks.json"
            view = root / "WORK_QUEUE.md"
            initial = records(task())
            source.write_bytes(queue._canonical_json(initial))
            view.write_text(queue.render_queue(initial), encoding="utf-8")
            source_before, view_before = source.read_bytes(), view.read_bytes()
            with self.assertRaisesRegex(queue.QueueError, "only allowed on termrock-implementation"):
                queue._mutate(
                    5, lambda *_: self.fail("mutator ran on the wrong branch"),
                    tasks_path=source, queue_path=view, repo_root=root,
                    subject_reader=lambda: ("main", BASE),
                )
            self.assertEqual(source_before, source.read_bytes())
            self.assertEqual(view_before, view.read_bytes())

    def test_accept_reads_branch_and_head_after_lock_acquisition(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "tasks.json"
            view = root / "WORK_QUEUE.md"
            initial = records(task())
            source.write_bytes(queue._canonical_json(initial))
            view.write_text(queue.render_queue(initial), encoding="utf-8")
            real_lock = queue._exclusive_lock
            lock_held = False

            @contextmanager
            def observed_lock(path):
                nonlocal lock_held
                with real_lock(path):
                    lock_held = True
                    try:
                        yield
                    finally:
                        lock_held = False

            def current_subject():
                self.assertTrue(lock_held)
                return "termrock-implementation", BASE

            with mock.patch.object(queue, "_exclusive_lock", observed_lock):
                updated = queue._mutate(
                    5,
                    lambda current, branch, head: queue.accept_record(
                        current, new_claim(), 5,
                        current_branch=branch, current_head=head, now=NOW,
                    ),
                    tasks_path=source, queue_path=view, repo_root=root,
                    subject_reader=current_subject,
                )
            self.assertEqual(6, updated["queue_revision"])

    def test_scope_digest_binds_scope_files_and_excludes_source_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            scoped = root / "docs/implementation/visibility"
            scoped.mkdir(parents=True)
            subject_file = scoped / "plan.md"
            subject_file.write_text("reviewed plan\n", encoding="utf-8")
            source_record = scoped / "tasks.json"
            source_record.write_text("revision 1\n", encoding="utf-8")
            before = queue.scope_digest(root, ["docs/implementation/visibility/**"])
            source_record.write_text("revision 2\n", encoding="utf-8")
            self.assertEqual(
                before, queue.scope_digest(root, ["docs/implementation/visibility/**"])
            )
            subject_file.write_text("changed plan\n", encoding="utf-8")
            self.assertNotEqual(
                before, queue.scope_digest(root, ["docs/implementation/visibility/**"])
            )

    def test_instruction_packet_lists_ancestor_rules_and_hashes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "AGENTS.md").write_text("root rules\n", encoding="utf-8")
            (root / "CLAUDE.md").symlink_to("AGENTS.md")
            (root / "docs/implementation").mkdir(parents=True)
            (root / "docs/AGENTS.md").write_text("docs rules\n", encoding="utf-8")
            (root / "docs/CLAUDE.md").symlink_to("AGENTS.md")
            packet = queue.instruction_packet(root, ["docs/implementation/plan.md"])
            self.assertEqual(["AGENTS.md", "docs/AGENTS.md"], [item["path"] for item in packet])
            self.assertEqual(hashlib.sha256(b"root rules\n").hexdigest(), packet[0]["sha256"])

    def test_instruction_packet_rejects_duplicate_claude_body(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "AGENTS.md").write_text("root rules\n", encoding="utf-8")
            (root / "CLAUDE.md").write_text("copied rules\n", encoding="utf-8")
            with self.assertRaisesRegex(queue.QueueError, "must remain a symlink"):
                queue.instruction_packet(root, ["some/file.rs"])

    def test_instruction_packet_allows_only_adjacent_claude_link_and_rejects_scoped_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "AGENTS.md").write_text("root rules\n", encoding="utf-8")
            (root / "CLAUDE.md").symlink_to("AGENTS.md")
            packet = queue.instruction_packet(root, ["CLAUDE.md"])
            self.assertEqual(["AGENTS.md"], [item["path"] for item in packet])

            (root / "owned/nested").mkdir(parents=True)
            (root / "outside").mkdir()
            (root / "owned/nested/escape").symlink_to(root / "outside", target_is_directory=True)
            with self.assertRaisesRegex(queue.QueueError, "instruction scope contains a symlink"):
                queue.instruction_packet(root, ["owned/**"])

            (root / "owned/nested/escape").unlink()
            (root / "owned/leaf").symlink_to(root / "outside", target_is_directory=True)
            with self.assertRaisesRegex(queue.QueueError, "instruction scope leaf is a symlink"):
                queue.instruction_packet(root, ["owned/leaf"])


if __name__ == "__main__":
    unittest.main()
