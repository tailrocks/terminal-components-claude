"""Protected synthetic execution instrument; never imported from submitted code."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import unittest


source, operation, config_path = sys.argv[1:]
config = json.loads(Path(config_path).read_text())
spec = importlib.util.spec_from_file_location("fixture_app", source)
app = importlib.util.module_from_spec(spec)
spec.loader.exec_module(app)
calls = []


def profile(frame, event, arg):
    if event == "call" and frame.f_code.co_filename == source:
        owner = type(frame.f_locals["self"]).__name__ if "self" in frame.f_locals else "module"
        calls.append(owner + "." + frame.f_code.co_name)


sys.setprofile(profile)
if operation in {"direct", "pty", "architecture"}:
    if operation == "pty":
        if not os.isatty(0) or not os.isatty(1):
            raise RuntimeError("a real PTY is required")
        key = os.read(0, 1).decode()
    else:
        key = "+"
    model = app.App(config["seed"] + (17 if config.get("wrong_seed") else 0), config["palette"], config.get("disabled", False))
    if not config.get("skip_seed_transition"):
        for seed_key in config.get("seed_steps", []):
            model.update(seed_key)
    before = model.draw()
    model.update(key)
    after = model.draw()
    result = {"before": before, "after": after, "value": model.value,
              "calls": calls, "pty": operation == "pty"}
    if operation in {"direct", "pty"}:
        captures = []
        for width in (8, 12):
            for palette in ("blue", "yellow"):
                sample = app.App(config["seed"] + (17 if config.get("wrong_seed") else 0), palette, width=width)
                if not config.get("skip_seed_transition"):
                    for seed_key in config.get("seed_steps", []):
                        sample.update(seed_key)
                before = sample.draw()
                sample.update(key)
                captures.append({"id": f"tiny/{operation}/{width}/{palette}",
                                 "before": before, "after": sample.draw()})
        if config.get("omit_capture"):
            captures.pop()
        if config.get("duplicate_capture"):
            captures.append(captures[0])
        if config.get("omit_checkpoint"):
            del captures[0]["after"]
        result["captures"] = captures
        if config.get("extractor") == "constant":
            result["value"] = 0
        elif config.get("extractor") == "wrong":
            result["value"] = model.value - 1
        elif config.get("extractor") == "omitted":
            del result["value"]
elif operation == "tests":
    class ProductionTests(unittest.TestCase):
        def test_draw(self):
            self.assertEqual(app.App(4, "blue").draw()["control"]["text"], "4")

        def test_update(self):
            model = app.App(4, "blue")
            model.update("+")
            self.assertEqual(model.value, 5)

        def test_closed(self):
            self.assertFalse(config.get("closed_failure", False))

        def test_future(self):
            self.assertFalse(config.get("future_failure", True))

    if config.get("drop_test"):
        delattr(ProductionTests, "test_draw")
    if config.get("rename_test"):
        ProductionTests.test_renamed = ProductionTests.test_draw
        delattr(ProductionTests, "test_draw")
    if config.get("unknown_test"):
        ProductionTests.test_unknown = lambda self: self.fail("unregistered failure")
    names = unittest.defaultTestLoader.getTestCaseNames(ProductionTests)
    selected = names[:-1] if config.get("partial") else names

    class Results(unittest.TestResult):
        def __init__(self):
            super().__init__()
            self.records = []

        def addSuccess(self, test):
            super().addSuccess(test)
            self.records.append({"id": test._testMethodName, "status": "passed"})

        def addFailure(self, test, error):
            super().addFailure(test, error)
            self.records.append({"id": test._testMethodName, "status": "failed"})

        def addError(self, test, error):
            super().addError(test, error)
            self.records.append({"id": test._testMethodName, "status": "error"})

    observed = Results()
    unittest.TestSuite(ProductionTests(name) for name in selected).run(observed)
    shell = app.App(config["seed"], config["palette"])
    if not config.get("route_regression"):
        shell.update("+")
    scenario = {"id": "tiny-shell", "frame": shell.draw(), "state": {"route_value": shell.value}}
    if config.get("missing_contribution"):
        scenario["state"] = {}
    result = {"discovered": names, "results": observed.records,
              "profile": config.get("profile", "primary"), "package": "tiny", "target": "unit",
              "scenarios": [scenario], "calls": calls}
else:
    raise RuntimeError("unknown protected operation")
sys.setprofile(None)
print(json.dumps(result, sort_keys=True), flush=True)
