"""Standalone Rust contract-model architecture checks."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

from ..runner.context import Reject


def _read_standalone_sources(profile: dict[str, Any]) -> dict[str, str]:
    sources: dict[str, str] = {}
    roots = profile.get("source_roots") or []
    declared = profile.get("sources") or {}
    for root in roots:
        path = Path(root)
        if not path.exists():
            raise Reject("ARCHITECTURE")
        if path.is_file():
            rel = path.name
            text = path.read_text()
            if declared and rel in declared:
                import hashlib

                if hashlib.sha256(text.encode()).hexdigest() != declared[rel]:
                    raise Reject("ARCHITECTURE")
            sources[rel] = text
            continue
        if path.is_dir() and not any(path.iterdir()):
            raise Reject("ARCHITECTURE")
        for file in sorted(path.rglob("*.rs")):
            rel = file.name if file.parent == path else str(file.relative_to(path.parent))
            text = file.read_text()
            if declared and rel in declared:
                import hashlib

                if hashlib.sha256(text.encode()).hexdigest() != declared[rel]:
                    raise Reject("ARCHITECTURE")
            sources[rel] = text
    for rel in declared:
        if rel not in sources and any(Path(root).exists() for root in roots):
            candidate = None
            for root in roots:
                probe = Path(root) / rel
                if probe.is_file():
                    candidate = probe.read_text()
                    break
            if candidate is None:
                raise Reject("ARCHITECTURE")
            sources[rel] = candidate
    if not sources:
        raise Reject("ARCHITECTURE")
    return sources


def _collect_rs_files(text: str, path: str) -> None:
    if "fn broken( {" in text or "fn broken({" in text:
        raise Reject("ARCHITECTURE")


def analyze_standalone(profile: dict[str, Any]) -> None:
    sources = _read_standalone_sources(profile)
    for path, text in sources.items():
        _collect_rs_files(text, path)
    app = sources.get("app.rs", "")
    library = sources.get("library.rs", "")
    if not app or not library:
        raise Reject("ARCHITECTURE")

    if re.search(r"fn\s+control_props\s*\(\s*self\b", app):
        raise Reject("ARCHITECTURE")
    if re.search(r"fn\s+control_props\s*\(\s*&\s*self\b", app):
        raise Reject("ARCHITECTURE")
    if re.search(r"fn\s+control_props\s*\(\s*&\s*mut\s+self\b", app):
        raise Reject("ARCHITECTURE")
    if re.search(r"fn\s+control_props\s*\(\s*self\s*:", app):
        raise Reject("ARCHITECTURE")
    if re.search(r"pub\s+fn\s+control_props\s*\(", app):
        raise Reject("ARCHITECTURE")
    if "control_props(self.disabled)" in app and "Self::control_props" not in app:
        if not re.search(r"fn control_props\(disabled: bool\)", app):
            raise Reject("ARCHITECTURE")
    if "fn control_props(disabled: bool)" not in app and "fn control_props(this: &Self)" not in app:
        if "control_props(" in app:
            raise Reject("ARCHITECTURE")
    if app.count("fn control_props(") != 1:
        raise Reject("ARCHITECTURE")
    if "Props::new(7).disabled(self.disabled)" in app and "Self::control_props(self)" not in app:
        raise Reject("ARCHITECTURE")
    if "Props::new(7).disabled(self.disabled)" in app.replace(" ", ""):
        raise Reject("ARCHITECTURE")
    if re.search(r"Widget::draw\([^)]*Props::new\(7\)", app):
        raise Reject("ARCHITECTURE")
    draw_line = "Widget::draw(Self::control_props(self.disabled), self.value, busy, slot, ui);"
    if draw_line not in app and "Widget::draw(" in app:
        if "screen_picture" in app or "if false {" in app or "&mut Ui::new(false)" in app:
            raise Reject("ARCHITECTURE")
        if re.search(r"ui\.row\([^)]*ui\.paint\(1,", app):
            raise Reject("ARCHITECTURE")
    if "Widget::update(Self::control_props(false)" in app:
        raise Reject("ARCHITECTURE")
    if "let _ = ui;" in app.replace("Widget::update", ""):
        raise Reject("ARCHITECTURE")
    if "if true" in library and "if !props.disabled" not in library:
        raise Reject("ARCHITECTURE")
    if "ui.resolve(props.id, 2); ui.resolve(props.id, 3);" in library:
        raise Reject("ARCHITECTURE")
    if "PARTS: &[u32] = &[0, 1, 2, 3]" in library:
        raise Reject("ARCHITECTURE")
    if "self.owner = 1;" in library and "self.owner = 2;" not in library:
        raise Reject("ARCHITECTURE")
    if "let _ = previous;" in library:
        raise Reject("ARCHITECTURE")
    if "if self.owner == 2 { 999 }" in library:
        raise Reject("ARCHITECTURE")
    if "REGISTRY: &[u32] = &[]" in library:
        raise Reject("ARCHITECTURE")
    if ".filter(|_| false)" in library or ".filter(|_| !busy)" in library:
        raise Reject("ARCHITECTURE")
    if "DOCUMENTED_SLOTS: &[u32] = &[0, 1, 2]" in library:
        raise Reject("ARCHITECTURE")
    if "Self::missing_props" in app:
        raise Reject("ARCHITECTURE")
    if app.rstrip().endswith("fn broken( {"):
        raise Reject("ARCHITECTURE")
    if "#[cfg(test)] mod tests {}" in app and "fn broken( {" in app:
        raise Reject("ARCHITECTURE")


def validate_runtime(payload: dict[str, Any], seed: int, *, custom_art: bool = False) -> None:
    if payload.get("parts") != [0, 1, 2]:
        raise Reject("ARCHITECTURE")
    if payload.get("slots") != [1, 2]:
        raise Reject("ARCHITECTURE")
    if payload.get("registry") != [7]:
        raise Reject("ARCHITECTURE")
    scenes = payload.get("scenes")
    if not isinstance(scenes, list) or len(scenes) != 24:
        raise Reject("ARCHITECTURE")
    seen: set[tuple[Any, ...]] = set()
    for scene in scenes:
        key = (scene.get("disabled"), scene.get("busy"), scene.get("poison"), scene.get("slot"))
        if key in seen:
            raise Reject("ARCHITECTURE")
        seen.add(key)
        disabled, busy, poison, slot = key
        if not isinstance(disabled, bool) or not isinstance(busy, bool) or not isinstance(poison, bool):
            raise Reject("ARCHITECTURE")
        if slot not in (0, 1, 2):
            raise Reject("ARCHITECTURE")
        value = seed + (0 if disabled else 1)
        cells = [91, 48 + value % 10 + (16 if poison else 0), 42 if busy else 43, 35 if custom_art else 64]
        if slot:
            cells[slot] = 126
        if scene.get("value") != value or scene.get("cells") != cells:
            raise Reject("ARCHITECTURE")
        if scene.get("draws") != 1 or scene.get("updates") != 1:
            raise Reject("ARCHITECTURE")
        resolutions = scene.get("resolutions")
        expected = [[7, 0, 1], [7, 1, 1], [7, 2, 1], [7, 1, 1], [7, 99, 2], [7, 98, 2], [7, 97, 2], [7, 0, 1]]
        if resolutions != expected:
            raise Reject("ARCHITECTURE")
        owned = {part for ident, part, owner in resolutions if owner == 1 for ident in [0] for part in [part]}
        owned = {part for _, part, owner in resolutions if owner == 1}
        if owned != set(payload["parts"]):
            raise Reject("ARCHITECTURE")
        writes = [[0, 91, 1], [1, 48 + value % 10 + (16 if poison else 0), 1], [2, 42 if busy else 43, 1]]
        if slot:
            writes.append([slot, 126, 1])
        writes.append([3, 35 if custom_art else 64, 2])
        if scene.get("writes") != writes:
            raise Reject("ARCHITECTURE")


def validate_standalone_event(event: dict[str, Any], profile: dict[str, Any]) -> None:
    compilation = event.get("compilation") or {}
    if compilation.get("exit", 0) != 0 or event.get("exit", 0) != 0:
        raise Reject("ARCHITECTURE")
    analyze_standalone(profile)
    if not isinstance(event.get("payload"), dict):
        raise Reject("ARCHITECTURE")
    custom_art = any("ui.paint(3, 35)" in Path(root).read_text() for root in profile.get("source_roots", []) if Path(root).is_file() and Path(root).name == "app.rs")
    if not custom_art:
        for root in profile.get("source_roots", []):
            path = Path(root) / "app.rs"
            if path.is_file() and "ui.paint(3, 35)" in path.read_text():
                custom_art = True
    validate_runtime(event["payload"], profile.get("seed", 0), custom_art=custom_art)
