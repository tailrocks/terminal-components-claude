#!/usr/bin/env python3
"""Exercise the perf pipeline's actual task bodies (stdlib only).

The generated Perf workflow (scheduled-checks file) runs mise tasks; the
cargo pipelines live in those task bodies, not in the workflow. This contract
pins both sides: the workflow shape (blocking vs advisory, depth, base) and
the task bodies' failure propagation under POSIX sh.
"""

import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/perf.yml"
MISE = ROOT / "mise.toml"

# The four release-test pipelines: (task, package marker).
PIPELINES = [
    ("ci-perf-lib", "-p junie-tui "),
    ("ci-perf-apps", "-p xtask -- app-perf"),
    ("ci-perf-strict-lib", "-p junie-tui "),
    ("ci-perf-strict-apps", "-p xtask -- app-perf"),
]


def task_body(source, name):
    match = re.search(
        r'^\[tasks\.' + re.escape(name) + r'\]\n(?:.*\n)*?^run = "(.*)"$',
        source,
        re.MULTILINE,
    )
    if not match:
        raise AssertionError("mise task missing or not a single run string: " + name)
    return match[1]


def exercise_body(body, exit_code):
    with tempfile.TemporaryDirectory(prefix="perf-pipeline-") as directory:
        root = Path(directory)
        cargo = root / "cargo"
        cargo.write_text("#!/bin/sh\nprintf 'PERF injected failure\\n'\nexit " + str(exit_code) + "\n")
        cargo.chmod(0o755)
        script = root / "step.sh"
        script.write_text(body + "\n")
        env = dict(os.environ, PATH=str(root) + ":" + os.environ["PATH"])
        # POSIX sh: the strictest shell mise may use. No pipefail assumed.
        result = subprocess.run(["sh", str(script)], cwd=root, env=env,
                                capture_output=True, text=True)
        if "PERF injected failure" not in result.stdout:
            raise AssertionError("task body did not execute the injected Cargo command")
        return result.returncode


class PerfWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = WORKFLOW.read_text()
        cls.mise = MISE.read_text()
        cls.bodies = {name: task_body(cls.mise, name) for name, _ in PIPELINES}

    def test_workflow_triggers_and_depth(self):
        self.assertIn("  push:\n    branches: [main]", self.source)
        self.assertIn("  pull_request:", self.source)
        perf, strict = self.source.split("  perf-strict:\n")
        self.assertIn("fetch-depth: 0", perf)
        self.assertNotIn("fetch-depth", strict)

    def test_perf_job_carries_an_explicit_base(self):
        self.assertIn(
            "BLESS_GUARD_BASE: \"${{ github.event_name == 'pull_request' "
            "&& github.event.pull_request.base.sha || github.event.before }}\"",
            self.source,
        )

    def test_blocking_and_advisory_split(self):
        blocking, advisory = self.source.split("  perf-strict:\n")
        self.assertIsNone(re.search(r"^\s+continue-on-error:", blocking, re.MULTILINE))
        self.assertIn("    needs: [perf]\n", advisory)
        self.assertEqual(advisory.count("continue-on-error: true"), 1)
        self.assertIn('      PERF_STRICT: "1"', advisory)

    def test_all_current_perf_targets_have_blocking_and_advisory_tasks(self):
        self.assertEqual(len(self.bodies), 4)
        lib = [name for name, marker in PIPELINES if marker == "-p junie-tui "]
        apps = [name for name, marker in PIPELINES if marker != "-p junie-tui "]
        self.assertEqual(len(lib), 2)
        self.assertEqual(len(apps), 2)
        for name, marker in PIPELINES:
            body = self.bodies[name]
            self.assertIn(marker, body, name)
            self.assertIn("--locked", body, name)
            if "apps" not in name:
                # app-perf runs each package's release suite inside xtask; the
                # --release flag lives there, not in the task body.
                self.assertIn("--release", body, name)
            # Explicit status capture: no bare `| tee` whose verdict depends
            # on a shell pipefail option the task does not control.
            self.assertIn("rc=$?", body, name)
            self.assertIn("exit $rc", body, name)
        inventory = json.loads((ROOT / "tools/app-inventory.json").read_text())
        self.assertEqual([app["id"] for app in inventory["apps"]],
                         ["showcase", "tablepro", "jackin-preview", "holla"])
        self.assertTrue(all(app["perf_targets"] == ["perf"] for app in inventory["apps"]))

    def test_each_task_propagates_success_and_failure(self):
        for name in self.bodies:
            # Compilation and assertion failures share Cargo's nonzero process
            # contract. Distinct codes also guard against checking only exit 101.
            for kind, code in (("success", 0), ("compile", 101), ("correctness", 1),
                               ("allocations", 2), ("bytes", 3)):
                with self.subTest(task=name, kind=kind):
                    self.assertEqual(exercise_body(self.bodies[name], code), code)

    def test_mutation_dropping_status_capture_is_detected(self):
        name, _ = PIPELINES[0]
        mutated = self.bodies[name].replace("; exit $rc", "")
        self.assertEqual(exercise_body(mutated, 101), 0)
        with self.assertRaises(AssertionError):
            self.assertEqual(exercise_body(mutated, 101), 101)


if __name__ == "__main__":
    unittest.main(verbosity=2)
