import copy
import io
import importlib.util
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
STATUS_SCRIPT = ROOT / "tools/visibility/status.py"
FACTS_PATH = ROOT / "tools/visibility/source-facts.json"
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"

spec = importlib.util.spec_from_file_location("visibility_status", STATUS_SCRIPT)
status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(status)


def inputs():
    return (
        json.loads(FACTS_PATH.read_text(encoding="utf-8")),
        json.loads(TASKS_PATH.read_text(encoding="utf-8")),
    )


class StatusTests(unittest.TestCase):
    def test_initial_conclusions_are_not_run_and_source_pair_is_exact(self):
        facts, tasks = inputs()
        output = status.render_status(facts, tasks)

        self.assertIn("Visibility / Complete | NOT_RUN", output)
        self.assertIn("Refactor / Ready | NOT_RUN", output)
        self.assertIn("Reference / Qualified | NOT_RUN", output)
        self.assertIn("Command / Ready | NOT_RUN", output)
        self.assertIn("Evidence freshness | NOT_RUN", output)
        self.assertIn(
            "Ready has its own column, separate from Build, Launch, First frame, "
            "input and interaction, Exit, Restoration, Visual, and Ownership.",
            output,
        )
        self.assertIn(facts["candidate"]["head_sha"], output)
        self.assertIn(facts["reference"]["head_sha"], output)
        self.assertIn("Candidate observed commit", output)
        self.assertIn("Reference observed commit", output)
        self.assertIn("Report commit | Set after this report is committed", output)
        self.assertNotIn("source S", output)
        self.assertNotIn("commit P", output)
        self.assertIn("A run receipt is a record of execution results", output)
        self.assertIn("NOT_RUN is a result status, not a run-receipt status", output)
        self.assertIn("Product requirements remain in [CHECKLIST.md]", output)
        self.assertNotIn("is generated from accepted records", output)
        self.assertIn("required shared-case and per-component checkpoint sets are Unknown", output)
        self.assertNotIn("| Required set | 0 |", output)

    def test_all_seven_commands_and_each_layer_are_visible(self):
        facts, tasks = inputs()
        output = status.render_status(facts, tasks)

        for run_id, command in status.COMMANDS:
            self.assertIn("| {} | {} |".format(run_id, command), output)
        self.assertEqual(7, sum(1 for line in output.splitlines() if line.startswith("| RUN-")))
        for layer in ("Build", "Launch", "First frame", "Interaction", "Exit",
                      "Restoration", "Visual", "Ownership", "Ready"):
            self.assertIn(layer, output)

    def test_role_changes_local_presentation_and_keeps_candidate_queue_link(self):
        facts, tasks = inputs()
        candidate = status.render_status(facts, tasks, role="candidate")
        reference = status.render_status(facts, tasks, role="reference")
        queue_url = "https://github.com/{}/blob/{}/WORK_QUEUE.md".format(
            status.REPOSITORY, facts["candidate"]["branch"]
        )

        self.assertIn("| Local role | Candidate (termrock-implementation) |", candidate)
        self.assertIn("| Local role | Reference (visual-baseline) |", reference)
        self.assertIn("[WORK_QUEUE.md]({})".format(queue_url), reference)
        self.assertIn("[Work queue]({})".format(queue_url), reference)
        for run_id, command in status.COMMANDS:
            row = "| {} | {} |".format(run_id, command)
            self.assertIn(row, candidate)
            self.assertIn(row, reference)

        candidate_common = candidate.replace(
            "| Local role | Candidate (termrock-implementation) |",
            "| Local role | LOCAL_ROLE |",
        ).replace("](WORK_QUEUE.md)", "]({})".format(queue_url))
        reference_common = reference.replace(
            "| Local role | Reference (visual-baseline) |",
            "| Local role | LOCAL_ROLE |",
        )
        self.assertEqual(candidate_common, reference_common)

    def test_cli_accepts_reference_role(self):
        output = io.StringIO()
        with redirect_stdout(output):
            result = status.main(["--role", "reference"])

        self.assertEqual(0, result)
        self.assertIn("| Local role | Reference (visual-baseline) |", output.getvalue())

    def test_visual_interaction_api_and_ownership_are_separate(self):
        facts, tasks = inputs()
        output = status.render_status(facts, tasks)

        for column in (
            "Reference visual", "Candidate visual",
            "Reference interaction", "Candidate interaction",
            "Reference API", "Candidate API",
            "Reference ownership", "Candidate ownership",
        ):
            self.assertIn(column, output)
        self.assertIn("NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN", output)

    def test_ci_and_dco_remain_separate_from_product_results(self):
        facts, tasks = inputs()
        output = status.render_status(facts, tasks)

        self.assertIn("37721476033", output)
        self.assertIn("Workflow file exceeds the maximum allowed size of 500 KB.", output)
        self.assertIn(
            "[source](https://github.com/tailrocks/terminal-components-claude/actions/runs/37721476033)",
            output,
        )
        self.assertIn("0 jobs; 0 artifacts", output)
        self.assertIn("This workflow result is not a product test result", output)
        self.assertIn("failure at", output)
        self.assertIn("113129904697", output)
        self.assertIn("action_required", output)
        self.assertIn("separate from product results", output)
        self.assertIn("cargo-nextest 0.9.146 through mise", output)

    def test_changed_frozen_tag_is_rejected(self):
        facts, tasks = inputs()
        facts = copy.deepcopy(facts)
        facts["visual_authority"]["commit_sha"] = "0" * 40

        with self.assertRaisesRegex(ValueError, "frozen visual tag identity changed"):
            status.render_status(facts, tasks)

    def test_candidate_remote_mismatch_is_rejected(self):
        facts, tasks = inputs()
        facts = copy.deepcopy(facts)
        facts["candidate"]["remote_head_sha"] = "0" * 40

        with self.assertRaisesRegex(ValueError, "remote candidate identity differs"):
            status.render_status(facts, tasks)

    def test_new_typed_observations_do_not_require_reporter_code_changes(self):
        facts, tasks = inputs()
        facts = copy.deepcopy(facts)
        facts["ci_observation"].update(
            run_id="123456",
            status="completed",
            conclusion="success",
            job_count=4,
            artifact_count=1,
            url="https://github.com/{}/actions/runs/123456".format(status.REPOSITORY),
            failure_reason=None,
        )
        facts["dco_observation"].update(
            check_run_id="654321",
            conclusion="success",
            affected_commit_count=0,
        )
        facts["ambient_tools"].update(
            python="3.12.1",
            mise_rust="1.99.0",
            cargo_nextest="0.9.200",
        )

        output = status.render_status(facts, tasks)
        self.assertIn("CI run 123456", output)
        self.assertIn("4 jobs; 1 artifacts", output)
        self.assertIn("DCO check 654321", output)
        self.assertIn("cargo-nextest 0.9.200", output)
        self.assertIn("success at", output)
        self.assertIn("No commits are reported with sign-off problems", output)
        self.assertNotIn("Failure reason", output)
        self.assertNotIn("workflow failure", output)

        facts["ci_observation"]["conclusion"] = None
        facts["dco_observation"]["conclusion"] = None
        unknown_output = status.render_status(facts, tasks)
        self.assertIn("unknown at", unknown_output)
        self.assertIn("DCO check 654321", unknown_output)
        self.assertIn("| unknown; No commits are reported", unknown_output)

        facts["ci_observation"]["conclusion"] = "failure"
        facts["ci_observation"]["failure_reason"] = {
            "reason": "A different recorded workflow failure.",
            "source_url": "https://example.test/run/123456",
        }
        changed_failure_output = status.render_status(facts, tasks)
        self.assertIn("Failure reason: A different recorded workflow failure.", changed_failure_output)
        self.assertIn("[source](https://example.test/run/123456)", changed_failure_output)

    def test_unknown_ci_state_and_invalid_count_are_rejected(self):
        facts, tasks = inputs()
        facts = copy.deepcopy(facts)
        facts["ci_observation"]["conclusion"] = "invented"
        with self.assertRaisesRegex(ValueError, "unknown CI conclusion"):
            status.render_status(facts, tasks)

        facts, tasks = inputs()
        facts = copy.deepcopy(facts)
        facts["ci_observation"]["job_count"] = -1
        with self.assertRaisesRegex(ValueError, "nonnegative integer"):
            status.render_status(facts, tasks)

        facts, tasks = inputs()
        facts = copy.deepcopy(facts)
        facts["ci_observation"]["failure_reason"]["source_url"] = "file:///tmp/failure"
        with self.assertRaisesRegex(ValueError, "source URL must be HTTPS"):
            status.render_status(facts, tasks)

    def test_duplicate_or_unknown_task_records_are_rejected(self):
        facts, tasks = inputs()
        tasks = copy.deepcopy(tasks)
        tasks["tasks"].append(copy.deepcopy(tasks["tasks"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate work ID"):
            status.render_status(facts, tasks)

        facts, tasks = inputs()
        tasks = copy.deepcopy(tasks)
        tasks["tasks"][0]["state"] = "passed"
        with self.assertRaisesRegex(ValueError, "unknown task state"):
            status.render_status(facts, tasks)

    def test_generation_is_deterministic(self):
        facts, tasks = inputs()
        first = status.render_status(facts, tasks)
        second = status.render_status(facts, tasks)
        self.assertEqual(first, second)


if __name__ == "__main__":
    unittest.main()
