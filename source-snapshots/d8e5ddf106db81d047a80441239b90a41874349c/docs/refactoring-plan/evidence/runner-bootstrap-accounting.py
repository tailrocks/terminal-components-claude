"""Protected actual-test fixture for accounting and source-preservation qualification."""
import json
import importlib.util
from pathlib import Path
import sys
import unittest

config = json.loads(Path(sys.argv[1]).read_text())
spec = importlib.util.spec_from_file_location("production", Path(__file__).with_name("production.py"))
production = importlib.util.module_from_spec(spec)
spec.loader.exec_module(production)


def production_value():
    return production.value()


class ProductionTests(unittest.TestCase):
    def test_compatible(self):
        self.assertEqual(production_value(), 7)

    def test_future(self):
        if config.get("classification_error"):
            raise RuntimeError("actual worker error, not an assertion failure")
        self.assertEqual(production.future_value(config["package"]), 7)


class Results(unittest.TestResult):
    def __init__(self):
        super().__init__()
        self.records = []

    def addSuccess(self, test):
        super().addSuccess(test)
        self.records.append({"name": test._testMethodName, "status": "passed"})

    def addFailure(self, test, error):
        super().addFailure(test, error)
        self.records.append({"name": test._testMethodName, "status": "failed"})

    def addError(self, test, error):
        super().addError(test, error)
        self.records.append({"name": test._testMethodName, "status": "error"})

    def addSkip(self, test, reason):
        super().addSkip(test, reason)
        self.records.append({"name": test._testMethodName, "status": "skipped"})


suite = unittest.defaultTestLoader.loadTestsFromTestCase(ProductionTests)
results = Results()
suite.run(results)
print(json.dumps({"results": results.records, "production_witness": production_value()}, sort_keys=True))
