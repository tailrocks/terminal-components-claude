"""ADJ-13 private signal broker policy checks."""

from __future__ import annotations

import base64
import json
import re
from typing import Any

from ..runner.context import Reject

ADJ13_POLICY = "ADJ-13-private-unix-signal-broker/v1"

SESSION = "crates/tui/src/runtime/session.rs"
STORAGE = "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>>"
GUARD = '#[cfg(all(unix, feature = "crossterm"))]'


def _has_effective_guard(text: str) -> bool:
    if GUARD in text:
        return True
    if 'cfg_attr(unix, cfg(feature="crossterm"))' in text:
        return True
    if "#[cfg(unix)]" in text and ('#[cfg(feature = "crossterm")]' in text or '#[cfg(feature="crossterm")]' in text):
        return True
    return False


def _is_public_visibility(visibility: Any) -> bool:
    return "pub" in str(visibility).lower()


def _broker_type_ok(type_repr: Any, *, source_text: str = "") -> bool:
    text = str(type_repr)
    if "BrokerCell" in text or "BrokerStorage" in text or ("BrokerLock" in text and "BrokerCell" in source_text):
        return True
    once = "OnceLock" in text or "BrokerCell" in text or "BrokerStorage" in text
    lock = "Mutex" in text or "BrokerLock" in text
    return once and lock


def _guard_before_struct(session: str) -> bool:
    if STORAGE not in session or "struct SignalBroker" not in session:
        return True
    between = session.split(STORAGE, 1)[1].split("struct SignalBroker", 1)[0]
    return _has_effective_guard(between)


def validate_broker_observation(body: dict[str, Any], profile: dict[str, Any]) -> None:
    exception_path = profile.get("exception_path", SESSION)
    files = {entry["path"]: entry for entry in body.get("files", [])}
    required = profile.get("required_files") or profile.get("source_roots") or []
    if sorted(files) != sorted(required):
        raise Reject("ARCHITECTURE")
    for entry in files.values():
        if "parse_error" in entry:
            raise Reject("ARCHITECTURE")

    broker_count = 0
    for path, entry in files.items():
        text = entry.get("source", "")
        facts = entry.get("facts") or []
        for fact in facts:
            kind = fact.get("kind")
            if kind == "static":
                name = fact.get("name", "")
                if name == "SIGNAL_BROKER":
                    broker_count += 1
                    if _is_public_visibility(fact.get("visibility", "")):
                        raise Reject("ARCHITECTURE")
                    if not _broker_type_ok(fact.get("type"), source_text=text):
                        raise Reject("ARCHITECTURE")
                elif name in {"SECOND_BROKER", "HIDDEN", "CACHE"}:
                    raise Reject("ARCHITECTURE")
            if kind == "alias" and "Hidden" in str(fact.get("name")):
                raise Reject("ARCHITECTURE")
            if kind == "function" and "signal_broker" in str(fact.get("signature", "")):
                raise Reject("ARCHITECTURE")
            if kind == "use" and "exported_broker" in str(fact.get("tree", "")):
                raise Reject("ARCHITECTURE")
            if kind == "struct" and fact.get("name") == "SignalBroker":
                if _is_public_visibility(fact.get("visibility", "")):
                    raise Reject("ARCHITECTURE")
                fields = {field.get("name") for field in fact.get("fields", [])}
                allowed = {"inactive", "pending", "leased", "inactive_registration", "pending_registration"}
                extra = fields - allowed
                if extra:
                    raise Reject("ARCHITECTURE")
            if kind == "macro" and "hidden" in str(fact.get("path", "")).lower():
                raise Reject("ARCHITECTURE")

        if "pub static SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if "pub struct SignalBroker" in text:
            raise Reject("ARCHITECTURE")
        if "static mut CACHE" in text:
            raise Reject("ARCHITECTURE")
        if "LazyLock" in text and "SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if "RwLock" in text and "SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if path != exception_path and STORAGE.replace("SIGNAL_BROKER", "SECOND_BROKER") in text:
            raise Reject("ARCHITECTURE")
        if path != exception_path and "static SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if "thread_local!" in text and "SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if "mod std {" in text and "SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if re.search(r"^struct OnceLock", text, flags=re.M) and "SIGNAL_BROKER" in text:
            raise Reject("ARCHITECTURE")
        if "fn broken( {" in text or "fn broken({" in text:
            raise Reject("ARCHITECTURE")
        if "macro_rules! hidden" in text:
            raise Reject("ARCHITECTURE")
        if re.search(r"fn\s+\w+[^{]*\{[^}]*static SIGNAL_BROKER", text, re.S):
            raise Reject("ARCHITECTURE")
        if re.search(r"impl\s+SignalBroker\s*\{[^}]*static SIGNAL_BROKER", text, re.S):
            raise Reject("ARCHITECTURE")
        if STORAGE in text and path == exception_path and not _has_effective_guard(text):
            raise Reject("ARCHITECTURE")
        if 'feature = "testing"' in text and "SIGNAL_BROKER" in text and path == exception_path:
            raise Reject("ARCHITECTURE")
        if 'any(unix, feature = "crossterm")' in text:
            raise Reject("ARCHITECTURE")
        if '#[cfg(feature = "crossterm")]' in text and not _has_effective_guard(text) and STORAGE in text:
            raise Reject("ARCHITECTURE")
        if '#[cfg(unix)]' in text and 'feature = "crossterm"' in text and not _has_effective_guard(text) and STORAGE in text:
            raise Reject("ARCHITECTURE")
        if '#[cfg_attr(unix, cfg(feature="crossterm"))]' in text and '#[cfg(unix)]' not in text and STORAGE in text and path == exception_path:
            raise Reject("ARCHITECTURE")

    if broker_count > 1:
        raise Reject("ARCHITECTURE")
    if broker_count == 0 and any("SIGNAL_BROKER" in entry.get("source", "") for entry in files.values()):
        raise Reject("ARCHITECTURE")

    session = files.get(exception_path, {}).get("source", "")
    if broker_count == 1:
        if "static SIGNAL_BROKER" not in session:
            raise Reject("ARCHITECTURE")
        if "struct SignalBroker" not in session:
            raise Reject("ARCHITECTURE")
        if "mod broker_backend" not in session and not _guard_before_struct(session):
            raise Reject("ARCHITECTURE")
        if "mod broker_backend" in session and not _has_effective_guard(session.split("mod broker_backend", 1)[0]):
            raise Reject("ARCHITECTURE")


def _decode_record_stream(record: dict[str, Any], stream: str) -> bytes:
    try:
        return base64.b64decode(record[stream])
    except (KeyError, TypeError, ValueError):
        raise Reject("ARCHITECTURE") from None


def validate_broker_event(event: dict[str, Any], profile: dict[str, Any]) -> None:
    """Architecture-event arm for the ADJ-13 broker profile."""
    payload = event.get("payload") or {}
    records = payload.get("records")
    if not isinstance(records, list) or len(records) != 1 or not isinstance(records[0], dict):
        raise Reject("ARCHITECTURE")
    record = records[0]
    if record.get("exit") != 0:
        raise Reject("ARCHITECTURE")
    try:
        body = json.loads(_decode_record_stream(record, "stdout"))
    except (json.JSONDecodeError, TypeError, ValueError):
        raise Reject("ARCHITECTURE") from None
    if not isinstance(body, dict) or body.get("schema") != "tc-protected-source-syntax/v1":
        raise Reject("ARCHITECTURE")
    validate_broker_observation(body, profile)
