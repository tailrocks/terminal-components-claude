"""Test identity helpers for accounting validation."""

from __future__ import annotations

import json
from typing import Any


def canonical_identity(identity: dict[str, Any]) -> bytes:
    return json.dumps(identity, sort_keys=True, separators=(",", ":")).encode()


def resolve_required_names(inventory: dict[str, Any]) -> list[str]:
    relocations = inventory.get("relocations", {})
    return sorted(relocations.get(name, name) for name in inventory["required"])


def original_name(test_id: str, relocations: dict[str, str]) -> str:
    for original, replacement in relocations.items():
        if replacement == test_id:
            return original
    return test_id
