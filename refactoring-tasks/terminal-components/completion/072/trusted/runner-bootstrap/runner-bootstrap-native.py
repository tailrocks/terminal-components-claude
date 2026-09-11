"""Fixed-row native assertions and source-only resized input mapping fixture."""
import json
from pathlib import Path
import sys

config = json.loads(Path(sys.argv[1]).read_text())


class Harness:
    def __init__(self, width, height, candidate=False):
        self.width, self.height, self.candidate = width, height, candidate
        self.attached = False

    def pointer(self, x, y):
        hit_row = self.height - (3 if self.candidate and config.get("candidate_geometry") else 2)
        if x == 2 and y == hit_row:
            self.attached = True

    def rows(self):
        rows = ["." * self.width for _ in range(self.height)]
        label = "attached" if self.attached else "unavailable"
        rows[self.height - 2] = label.ljust(self.width, ".")
        return rows


native = []
for width, height, row, expected in [(120, 40, 38, "attached"), (100, 30, 28, "unavailable")]:
    h = Harness(width, height)
    if expected == "attached":
        h.pointer(2, row)
    assert expected in h.rows()[row], "original fixed-row assertion"
    native.append({"width": width, "height": height, "row": row, "pointer": [2, row], "expected": expected, "rows": h.rows()})

resized = []
for width, height in [(80, 24), (100, 30), (120, 40), (160, 50)]:
    # This mapping is derived from the original source owner, never candidate geometry.
    row = height - 2
    h = Harness(width, height, candidate=True)
    pointer = [2, row + (1 if config.get("wrong_mapping") else 0)]
    if config.get("reuse_fixed_row"):
        pointer = [2, 38]
    if config.get("candidate_retargets"):
        pointer = [2, row - 1]
    h.pointer(*pointer)
    resized.append({"width": width, "height": height, "row": row, "pointer": pointer,
                    "rows": h.rows(), "attached": h.attached})
print(json.dumps({"native": native, "resized": resized}, sort_keys=True))
