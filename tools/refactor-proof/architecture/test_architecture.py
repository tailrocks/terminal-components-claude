#!/usr/bin/env python3
"""Focused tests for shipped architecture functions."""

from __future__ import annotations

import base64
import json
import sys
import tempfile
import types
import unittest
from pathlib import Path


PROOF = Path(__file__).resolve().parents[1]


def _install_packages() -> None:
    if "refactor_proof" in sys.modules:
        return
    package = types.ModuleType("refactor_proof")
    package.__path__ = [str(PROOF)]
    sys.modules["refactor_proof"] = package
    sys.modules["refactor_proof.runner"] = types.ModuleType("refactor_proof.runner")
    sys.modules["refactor_proof.runner"].__path__ = [str(PROOF / "runner")]
    sys.modules["refactor_proof.accounting"] = types.ModuleType("refactor_proof.accounting")
    sys.modules["refactor_proof.accounting"].__path__ = [str(PROOF / "accounting")]
    sys.modules["refactor_proof.architecture"] = types.ModuleType("refactor_proof.architecture")
    sys.modules["refactor_proof.architecture"].__path__ = [str(PROOF / "architecture")]


_install_packages()

from refactor_proof.architecture.broker import (  # noqa: E402
    ADJ13_POLICY,
    SESSION,
    validate_broker_event,
    validate_broker_observation,
)
from refactor_proof.architecture.dispatch import (  # noqa: E402
    _validate_architecture_context,
    _validate_event,
)
from refactor_proof.architecture.extension import validate_performance_event  # noqa: E402
from refactor_proof.architecture.fixture import validate_fixture_event  # noqa: E402
from refactor_proof.architecture.rust_model import (  # noqa: E402
    analyze_standalone,
    validate_runtime,
)
from refactor_proof.accounting.qualification import is_qualification_schema  # noqa: E402
from refactor_proof.runner.context import Reject  # noqa: E402


VALID_APP = """use crate::library::{Props, SlotFn, Ui, Widget};

#[derive(Clone, Copy)]
pub struct App { pub value: u32, pub disabled: bool }
impl App {
    fn control_props(disabled: bool) -> Props {
        Props::new(7).disabled(disabled)
    }
    pub fn update(&mut self, ui: &mut Ui) {
        Widget::update(Self::control_props(self.disabled), &mut self.value, ui);
    }
    pub fn draw(&self, busy: bool, slot: Option<(u32, SlotFn)>, ui: &mut Ui) {
        Widget::draw(Self::control_props(self.disabled), self.value, busy, slot, ui);
        crate::library::compose(Self::control_props(self.disabled), ui);
        ui.row(|ui| ui.paint(3, 64));
    }
}
"""

VALID_LIBRARY = """pub struct Props { pub id: u32, pub disabled: bool }
impl Props {
    pub fn new(id: u32) -> Self { Self { id, disabled: false } }
    pub fn disabled(mut self, disabled: bool) -> Self { self.disabled = disabled; self }
}
pub struct Ui;
impl Ui {
    pub fn resolve(&mut self, id: u32, part: u32) { let _ = (id, part); }
    pub fn paint(&mut self, _part: u32, _ch: u32) {}
    pub fn row<F: FnOnce(&mut Ui)>(&mut self, f: F) { f(self); }
}
pub type SlotFn = fn(&mut Ui);
pub struct Widget;
impl Widget {
    pub fn update(props: Props, value: &mut u32, ui: &mut Ui) {
        if !props.disabled { *value += 1; }
        let _ = ui;
    }
    pub fn draw(props: Props, value: u32, busy: bool, slot: Option<(u32, SlotFn)>, ui: &mut Ui) {
        let _ = (props, value, busy, slot, ui);
    }
}
pub const PARTS: &[u32] = &[0, 1, 2];
pub const DOCUMENTED_SLOTS: &[u32] = &[1, 2];
pub const REGISTRY: &[u32] = &[7];
pub fn compose(props: Props, ui: &mut Ui) {
    ui.resolve(props.id, 0);
    ui.resolve(props.id, 1);
    ui.resolve(props.id, 2);
}
"""


def _profile_with(app: str, library: str) -> dict:
    root = tempfile.TemporaryDirectory(prefix="tc-arch-unit-")
    path = Path(root.name)
    (path / "app.rs").write_text(app)
    (path / "library.rs").write_text(library)
    profile = {
        "schema": "tc-architecture-rust-profile/v1",
        "source_roots": [str(path)],
        "seed": 7,
    }
    # Keep the directory alive for the duration of the caller's assertion.
    profile["_keep"] = root
    return profile


class FixtureTests(unittest.TestCase):
    def test_missing_widget_draw_is_architecture(self) -> None:
        event = {
            "exit": 0,
            "payload": {
                "calls": ["App.update", "Props.enabled"],
                "value": 3,
                "pty": False,
            },
        }
        with self.assertRaises(Reject) as raised:
            validate_fixture_event(event, {"seed": 3})
        self.assertEqual(raised.exception.category, "ARCHITECTURE")

    def test_valid_tiny_fixture_accepts_disabled_value(self) -> None:
        event = {
            "exit": 0,
            "payload": {
                "calls": ["App.update", "Widget.draw", "Widget.draw", "Props.enabled"],
                "value": 11,
                "pty": False,
            },
        }
        validate_fixture_event(event, {"seed": 11})


class RustModelTests(unittest.TestCase):
    def test_valid_helper_is_accepted(self) -> None:
        analyze_standalone(_profile_with(VALID_APP, VALID_LIBRARY))

    def test_receiver_form_is_rejected(self) -> None:
        app = VALID_APP.replace(
            "fn control_props(disabled: bool)",
            "fn control_props(&self, disabled: bool)",
        )
        with self.assertRaises(Reject) as raised:
            analyze_standalone(_profile_with(app, VALID_LIBRARY))
        self.assertEqual(raised.exception.category, "ARCHITECTURE")

    def test_duplicate_same_id_construction_is_rejected(self) -> None:
        app = VALID_APP.replace(
            "Widget::draw(Self::control_props(self.disabled), self.value, busy, slot, ui);",
            "Widget::draw(Props::new(7).disabled(self.disabled), self.value, busy, slot, ui);",
        )
        with self.assertRaises(Reject) as raised:
            analyze_standalone(_profile_with(app, VALID_LIBRARY))
        self.assertEqual(raised.exception.category, "ARCHITECTURE")

    def test_missing_root_is_rejected(self) -> None:
        profile = _profile_with(VALID_APP, VALID_LIBRARY)
        profile["source_roots"].append(str(Path(profile["_keep"].name) / "absent"))
        with self.assertRaises(Reject) as raised:
            analyze_standalone(profile)
        self.assertEqual(raised.exception.category, "ARCHITECTURE")

    def test_runtime_custom_art_cells(self) -> None:
        payload = {
            "parts": [0, 1, 2],
            "slots": [1, 2],
            "registry": [7],
            "scenes": [],
        }
        scenes = []
        for disabled in (False, True):
            for busy in (False, True):
                for poison in (False, True):
                    for slot in (0, 1, 2):
                        value = 7 + (0 if disabled else 1)
                        cells = [91, 48 + value % 10 + (16 if poison else 0), 42 if busy else 43, 35]
                        if slot:
                            cells[slot] = 126
                        writes = [
                            [0, 91, 1],
                            [1, 48 + value % 10 + (16 if poison else 0), 1],
                            [2, 42 if busy else 43, 1],
                        ]
                        if slot:
                            writes.append([slot, 126, 1])
                        writes.append([3, 35, 2])
                        scenes.append(
                            {
                                "disabled": disabled,
                                "busy": busy,
                                "poison": poison,
                                "slot": slot,
                                "value": value,
                                "cells": cells,
                                "draws": 1,
                                "updates": 1,
                                "resolutions": [
                                    [7, 0, 1],
                                    [7, 1, 1],
                                    [7, 2, 1],
                                    [7, 1, 1],
                                    [7, 99, 2],
                                    [7, 98, 2],
                                    [7, 97, 2],
                                    [7, 0, 1],
                                ],
                                "writes": writes,
                            }
                        )
        payload["scenes"] = scenes
        validate_runtime(payload, 7, custom_art=True)


class PerformanceTests(unittest.TestCase):
    def test_style_share_above_five_percent_is_performance(self) -> None:
        event = {
            "payload": {
                "measurements": [
                    {
                        "draws": 8,
                        "allocations": 8,
                        "style_calls": 8,
                        "work": 16001600,
                        "style_ns": 20,
                        "frame_ns": 100,
                    }
                ]
            }
        }
        contract = {
            "profile": "release",
            "serial": True,
            "strict": True,
            "denominator": "frame_ns",
            "counter_owner": "global_allocator",
            "draws": 8,
            "allocations": 8,
            "style_calls": 8,
            "work": 16001600,
        }
        with self.assertRaises(Reject) as raised:
            validate_performance_event(event, contract)
        self.assertEqual(raised.exception.category, "PERFORMANCE")

    def test_debug_profile_is_performance(self) -> None:
        event = {"payload": {"measurements": [{"draws": 8, "allocations": 8, "style_calls": 8, "work": 1, "style_ns": 1, "frame_ns": 100}]}}
        contract = {
            "profile": "debug",
            "serial": True,
            "strict": True,
            "denominator": "frame_ns",
            "counter_owner": "global_allocator",
            "draws": 8,
            "allocations": 8,
            "style_calls": 8,
            "work": 1,
        }
        with self.assertRaises(Reject) as raised:
            validate_performance_event(event, contract)
        self.assertEqual(raised.exception.category, "PERFORMANCE")


class BrokerTests(unittest.TestCase):
    def test_second_singleton_is_rejected(self) -> None:
        body = {
            "files": [
                {
                    "path": "crates/tui/src/runtime/session.rs",
                    "source": (
                        '#[cfg(all(unix, feature = "crossterm"))]\n'
                        "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
                        "static SECOND_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
                        "struct SignalBroker { inactive: bool, pending: bool, leased: bool }\n"
                    ),
                    "facts": [
                        {"kind": "static", "name": "SIGNAL_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {"kind": "static", "name": "SECOND_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {"kind": "struct", "name": "SignalBroker", "visibility": "", "fields": [{"name": "inactive"}, {"name": "pending"}, {"name": "leased"}]},
                    ],
                }
            ]
        }
        profile = {
            "required_files": ["crates/tui/src/runtime/session.rs"],
            "exception_path": "crates/tui/src/runtime/session.rs",
        }
        with self.assertRaises(Reject) as raised:
            validate_broker_observation(body, profile)
        self.assertEqual(raised.exception.category, "ARCHITECTURE")

    def test_qualified_storage_is_accepted(self) -> None:
        source = (
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "struct SignalBroker {\n"
            "    inactive: std::sync::Arc<std::sync::atomic::AtomicBool>,\n"
            "    pending: std::sync::Arc<std::sync::atomic::AtomicBool>,\n"
            "    leased: bool,\n"
            "    inactive_registration: Option<signal_hook::SigId>,\n"
            "    pending_registration: Option<signal_hook::SigId>,\n"
            "}\n"
        )
        body = {
            "files": [
                {
                    "path": "crates/tui/src/runtime/session.rs",
                    "source": source,
                    "facts": [
                        {"kind": "static", "name": "SIGNAL_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {
                            "kind": "struct",
                            "name": "SignalBroker",
                            "visibility": "",
                            "fields": [
                                {"name": "inactive"},
                                {"name": "pending"},
                                {"name": "leased"},
                                {"name": "inactive_registration"},
                                {"name": "pending_registration"},
                            ],
                        },
                    ],
                }
            ]
        }
        profile = {
            "required_files": ["crates/tui/src/runtime/session.rs"],
            "exception_path": "crates/tui/src/runtime/session.rs",
        }
        validate_broker_observation(body, profile)

    def test_unknown_signal_broker_field_is_rejected(self) -> None:
        source = (
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "struct SignalBroker {\n"
            "    inactive: bool,\n"
            "    pending: bool,\n"
            "    leased: bool,\n"
            "    retry_budget: u8,\n"
            "}\n"
        )
        body = {
            "files": [
                {
                    "path": "crates/tui/src/runtime/session.rs",
                    "source": source,
                    "facts": [
                        {"kind": "static", "name": "SIGNAL_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {
                            "kind": "struct",
                            "name": "SignalBroker",
                            "visibility": "",
                            "fields": [
                                {"name": "inactive"},
                                {"name": "pending"},
                                {"name": "leased"},
                                {"name": "retry_budget"},
                            ],
                        },
                    ],
                }
            ]
        }
        profile = {
            "required_files": ["crates/tui/src/runtime/session.rs"],
            "exception_path": "crates/tui/src/runtime/session.rs",
        }
        with self.assertRaises(Reject) as raised:
            validate_broker_observation(body, profile)
        self.assertEqual(raised.exception.category, "ARCHITECTURE")

    def test_unrelated_macro_tokens_with_hidden_are_accepted(self) -> None:
        source = (
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "struct SignalBroker { inactive: bool, pending: bool, leased: bool }\n"
        )
        body = {
            "files": [
                {
                    "path": "crates/tui/src/components/grid.rs",
                    "source": "fn geometry() { assert_eq!(g.hidden_left, 0); }\n",
                    "facts": [
                        {
                            "kind": "macro",
                            "path": "assert_eq",
                            "tokens": "g.hidden_left, 0",
                        }
                    ],
                },
                {
                    "path": "crates/tui/src/runtime/session.rs",
                    "source": source,
                    "facts": [
                        {"kind": "static", "name": "SIGNAL_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {
                            "kind": "struct",
                            "name": "SignalBroker",
                            "visibility": "",
                            "fields": [{"name": "inactive"}, {"name": "pending"}, {"name": "leased"}],
                        },
                    ],
                },
            ]
        }
        profile = {
            "required_files": ["crates/tui/src/components/grid.rs", "crates/tui/src/runtime/session.rs"],
            "exception_path": "crates/tui/src/runtime/session.rs",
        }
        validate_broker_observation(body, profile)


def _syntax_event(files: list[dict]) -> dict:
    body = {"schema": "tc-protected-source-syntax/v1", "files": files}
    record = {
        "exit": 0,
        "stdout": base64.b64encode(json.dumps(body).encode()).decode(),
        "stderr": base64.b64encode(b"").decode(),
    }
    return {"exit": 0, "payload": {"kind": "source-policy", "records": [record]}}


def _adj13_profile(required: list[str]) -> dict:
    return {
        "schema": "tc-architecture-source-profile/v1",
        "kind": "source-policy",
        "policy": ADJ13_POLICY,
        "source_roots": ["crates/tui/src"],
        "required_files": required,
        "exception_path": SESSION,
    }


class DispatchTests(unittest.TestCase):
    def test_qualification_schema_allows_architecture_profile(self) -> None:
        context = {
            "schema": "tc-proof-runner-context/v1",
            "run_id": "r",
            "operation": "architecture",
            "tree": "0" * 40,
            "oracle_commit": "1" * 40,
            "oracle_tree": "2" * 40,
            "bundle": "/tmp/oracle.bundle",
            "bundle_sha256": "a" * 64,
            "tool": {"path": "/usr/bin/python3", "sha256": "b" * 64},
            "dependencies": [{"accepted": True, "integrated": True}],
            "adapter": {"changes": []},
            "lane": "direct",
            "axes": {"lanes": ["direct"], "widths": [8], "palettes": ["blue"]},
            "members": ["tiny/direct/8/blue"],
            "inventory": {},
            "evidence": [],
            "configuration": {},
            "architecture_profile": {"schema": "tc-architecture-rust-profile/v1"},
        }
        self.assertTrue(is_qualification_schema(context))
        _validate_architecture_context(context)
        self.assertEqual(context["architecture_profile"]["schema"], "tc-architecture-rust-profile/v1")

    def test_performance_family_uses_performance_category(self) -> None:
        event = {"payload": {"measurements": []}}
        context = {"qualification": {"family": "performance", "profile": "release"}}
        with self.assertRaises(Reject) as raised:
            _validate_event(event, context, None)
        self.assertEqual(raised.exception.category, "PERFORMANCE")

    def test_adj13_policy_uses_broker_event_arm(self) -> None:
        source = (
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "struct SignalBroker { inactive: bool, pending: bool, leased: bool }\n"
        )
        event = _syntax_event(
            [
                {
                    "path": SESSION,
                    "source": source,
                    "facts": [
                        {"kind": "static", "name": "SIGNAL_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {
                            "kind": "struct",
                            "name": "SignalBroker",
                            "visibility": "",
                            "fields": [{"name": "inactive"}, {"name": "pending"}, {"name": "leased"}],
                        },
                    ],
                }
            ]
        )
        _validate_event(event, {}, _adj13_profile([SESSION]))
        validate_broker_event(event, _adj13_profile([SESSION]))

    def test_adj13_policy_rejects_second_singleton_through_dispatch(self) -> None:
        source = (
            '#[cfg(all(unix, feature = "crossterm"))]\n'
            "static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
            "static SECOND_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n"
            "struct SignalBroker { inactive: bool, pending: bool, leased: bool }\n"
        )
        event = _syntax_event(
            [
                {
                    "path": SESSION,
                    "source": source,
                    "facts": [
                        {"kind": "static", "name": "SIGNAL_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {"kind": "static", "name": "SECOND_BROKER", "visibility": "", "type": "OnceLock<Mutex<SignalBroker>>"},
                        {
                            "kind": "struct",
                            "name": "SignalBroker",
                            "visibility": "",
                            "fields": [{"name": "inactive"}, {"name": "pending"}, {"name": "leased"}],
                        },
                    ],
                }
            ]
        )
        with self.assertRaises(Reject) as raised:
            _validate_event(event, {}, _adj13_profile([SESSION]))
        self.assertEqual(raised.exception.category, "ARCHITECTURE")


if __name__ == "__main__":
    unittest.main()
