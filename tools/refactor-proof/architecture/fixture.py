"""Tiny synthetic fixture architecture checks."""

from __future__ import annotations

from typing import Any

from ..runner.context import Reject
from ..runner.validate import seeded_value


def validate_fixture_event(event: dict[str, Any], config: dict[str, Any]) -> None:
    if event.get("exit", 0) != 0 or not isinstance(event.get("payload"), dict):
        raise Reject("ARCHITECTURE")
    body = event["payload"]
    calls = body.get("calls", [])
    if "App.update" not in calls:
        raise Reject("ARCHITECTURE")
    if calls.count("Widget.draw") != 2:
        raise Reject("ARCHITECTURE")
    if "Props.enabled" not in calls:
        raise Reject("ARCHITECTURE")
    seeded = seeded_value(config)
    value = body.get("value")
    if value is None or value != seeded:
        raise Reject("ARCHITECTURE")
    if body.get("pty") is not False:
        raise Reject("ARCHITECTURE")
