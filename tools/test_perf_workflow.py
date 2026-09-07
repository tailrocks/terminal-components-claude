#!/usr/bin/env python3
"""Exercise the workflow's actual pipelines under its declared shell (stdlib only)."""

import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import unittest

WORKFLOW = Path(__file__).resolve().parents[1] / ".github/workflows/perf.yml"


def shell_template(source):
    # Deliberately support this workflow's explicit global template only. Fail
    # closed if job/step overrides appear instead of silently testing another shell.
    declarations = re.findall(r"^\s*shell: (.+)$", source, re.MULTILINE)
    if len(declarations) != 1:
        raise AssertionError("expected exactly one global shell declaration")
    match = re.search(r"^defaults:\n  run:\n    shell: (.+)$", source, re.MULTILINE)
    if not match or declarations != [match[1]]:
        raise AssertionError("shell must be the workflow run default")
    return shlex.split(match[1])


def exercise_pipeline(shell, pipeline, exit_code):
    with tempfile.TemporaryDirectory(prefix="perf-pipeline-") as directory:
        root = Path(directory)
        cargo = root / "cargo"
        cargo.write_text("#!/bin/sh\nprintf 'PERF injected failure\\n'\nexit " + str(exit_code) + "\n")
        cargo.chmod(0o755)
        script = root / "step.sh"
        script.write_text(pipeline + "\n")
        env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ["PATH"])
        command = [str(script) if arg == "{0}" else arg for arg in shell]
        result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True)
        if "PERF injected failure" not in result.stdout:
            raise AssertionError("pipeline did not execute the injected Cargo command")
        return result.returncode


class PerfWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = WORKFLOW.read_text()
        cls.shell = shell_template(cls.source)
        cls.pipelines = re.findall(r"^          (cargo test .+ \| tee .+)$", cls.source, re.MULTILINE)

    def test_all_current_perf_targets_have_blocking_and_advisory_pipelines(self):
        self.assertEqual(len(self.pipelines), 8)
        for package in ("junie-tui", "showcase", "tablepro", "jackin-preview"):
            self.assertEqual(sum(f"-p {package} " in line for line in self.pipelines), 2)
        blocking, advisory = self.source.split("  perf-strict:\n")
        self.assertIsNone(re.search(r"^\s+continue-on-error:", blocking, re.MULTILINE))
        self.assertIn("    needs: perf\n", advisory)
        self.assertEqual(advisory.count("continue-on-error: true"), 1)
        self.assertIn('      PERF_STRICT: "1"', advisory)

    def test_each_pipeline_propagates_success_and_failure(self):
        for pipeline in self.pipelines:
            # Compilation and assertion failures share Cargo's nonzero process
            # contract. Distinct codes also guard against checking only exit 101.
            for kind, code in (("success", 0), ("compile", 101), ("correctness", 1),
                               ("allocations", 2), ("bytes", 3)):
                with self.subTest(pipeline=pipeline, kind=kind):
                    self.assertEqual(exercise_pipeline(self.shell, pipeline, code), code)

    def test_mutation_removing_pipefail_is_detected(self):
        mutated = shell_template(self.source.replace(" -o pipefail", ""))
        pipeline = self.pipelines[0]
        self.assertEqual(exercise_pipeline(mutated, pipeline, 101), 0)
        with self.assertRaises(AssertionError):
            self.assertEqual(exercise_pipeline(mutated, pipeline, 101), 101)


if __name__ == "__main__":
    unittest.main(verbosity=2)
