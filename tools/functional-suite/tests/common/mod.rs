use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tui_test::Backend;
use tui_test::api::{
    AutomaticRecording, KeyAction, MouseAction, MouseButton, MouseOptions, Operation,
    OperationResult, RawOutputCaptureOptions, RunOptions, State as SessionState, Timeouts,
};
use tui_test::launch::{LaunchEnvMode, LaunchOptions};
use tui_test::profile::Profile;
use tui_test::render::style::Style;
use tui_test::runtime::Session;
use tui_test::snapshot::{SnapshotScope, TerminalSnapshot};
use tui_test::terminal::cell::Attrs;
use tui_test::wait::{WaitQuietMode, WaitQuietOptions};
use tuiscotti::check::{CheckOutcome, FrozenStore};
use tuiscotti::render_profile::{PROFILE_NAME, platform_key, resolve_profile};
use tuiscotti::sample::{Observation, Sample};

mod source_manifest;

const DEFAULT_COLS: u16 = 120;
const DEFAULT_ROWS: u16 = 40;
const FIRST30_PRIMARY_CASE_DENOMINATOR: usize = 4;
const FIRST30_FRAME_DENOMINATOR: usize = 30;
const FIRST30_PRIMARY_CASE_IDS: [&str; FIRST30_PRIMARY_CASE_DENOMINATOR] = [
    "first30-v1/showcase.buttons",
    "first30-v1/holla.first_use",
    "first30-v1/jackin.settings",
    "first30-v1/tablepro.connections",
];
const RUNTIME_CASE_IDS: [&str; 7] = [
    "first30-v1/showcase.buttons",
    "first30-v1/holla.first_use",
    "first30-v1/jackin.settings",
    "first30-v1/tablepro.connections",
    "first30-v1/holla.first_use.frame1",
    "showcase-help/80x24-v1",
    "holla-first-use/action-identity-v1",
];
const HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR: usize = 8;
const HOLLA_RESULT_LIST_X_START: u32 = 0;
const HOLLA_RESULT_LIST_X_END: u32 = 78;
const HOLLA_RESULT_LIST_Y_START: u32 = 6;
const HOLLA_RESULT_LIST_Y_END: u32 = 38;
const HELP_TIMEOUT_MS: u64 = 15_000;
const FIRST30_READY_TIMEOUT_MS: u64 = 8_000;
const EXIT_TIMEOUT_MS: u64 = 30_000;
const QUIET_TIMEOUT: Duration = Duration::from_secs(15);
const QUIET_INTERVAL: Duration = Duration::from_millis(400);
const BUILD_EXPORT_SHA256: &str = env!("TERMROCK_SUITE_EXPORT_SHA256");
const CASE_INVENTORY_FILE: &str = "cases.toml";
const TERMINAL_PROFILES_FILE: &str = "terminal.toml";
const CASE_INVENTORY_SCHEMA: &str = "termrock-functional-case-inventory.v1";
const TERMINAL_PROFILES_SCHEMA: &str = "termrock-functional-terminal-profiles.v1";
const EXPORT_LOADER: &str = r###"
import hashlib
import json
import os
import pathlib
import re
import stat
import sys

def fail(message):
    raise SystemExit(message)

def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail("duplicate manifest key")
        result[key] = value
    return result

def safe_read(path):
    if not path.is_absolute() or ".." in path.parts:
        fail("unsafe export file path")
    components = str(path).split("/")
    if components[0] != "" or any(part in ("", ".", "..") for part in components[1:]):
        fail("unsafe export file path")
    descriptor = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for component in path.parts[1:-1]:
            info = os.stat(component, dir_fd=descriptor, follow_symlinks=False)
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
                fail("symlink or non-directory in export file path")
            next_descriptor = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        info = os.stat(path.parts[-1], dir_fd=descriptor, follow_symlinks=False)
        if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
            fail("export file is not regular")
        file_descriptor = os.open(path.parts[-1], os.O_RDONLY | os.O_NOFOLLOW, dir_fd=descriptor)
        try:
            if not stat.S_ISREG(os.fstat(file_descriptor).st_mode):
                fail("export file is not regular")
            chunks = []
            while True:
                chunk = os.read(file_descriptor, 1024 * 1024)
                if not chunk:
                    return b"".join(chunks)
                chunks.append(chunk)
        finally:
            os.close(file_descriptor)
    finally:
        os.close(descriptor)

arguments = sys.argv[1:]
if len(arguments) < 4:
    fail("missing export authentication arguments")
manifest_path = pathlib.Path(arguments[0])
helper_path = pathlib.Path(arguments[1])
expected = arguments[2]
helper_arguments = arguments[3:]
if re.fullmatch(r"[0-9a-f]{64}", expected) is None:
    fail("invalid export seal")
if manifest_path.parent != helper_path.parent or manifest_path.name != "export-manifest.json" or helper_path.name != "export.py":
    fail("unexpected export helper paths")
manifest_bytes = safe_read(manifest_path)
if hashlib.sha256(manifest_bytes).hexdigest() != expected:
    fail("export manifest seal mismatch")
try:
    manifest = json.loads(manifest_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)
except (UnicodeDecodeError, json.JSONDecodeError) as error:
    fail("invalid sealed export manifest: " + str(error))
files = manifest.get("files") if isinstance(manifest, dict) else None
if not isinstance(files, list):
    fail("sealed export manifest has no file entries")
helpers = [entry for entry in files if isinstance(entry, dict) and entry.get("path") == "export.py"]
if len(helpers) != 1:
    fail("sealed export manifest must name one helper")
entry = helpers[0]
if set(entry) != {"path", "mode", "size", "sha256"}:
    fail("sealed helper entry has unknown fields")
if type(entry["size"]) is not int or entry["size"] < 0 or re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]) is None:
    fail("sealed helper entry is malformed")
helper_bytes = safe_read(helper_path)
if len(helper_bytes) != entry["size"] or hashlib.sha256(helper_bytes).hexdigest() != entry["sha256"]:
    fail("sealed helper digest mismatch")
sys.argv = [str(helper_path), *helper_arguments]
exec(compile(helper_bytes, str(helper_path), "exec"), {"__name__": "__main__", "__file__": str(helper_path)})
"###;
const FIRST30_SHOWCASE_CHECKPOINTS: [&str; 7] = [
    "initial", "run", "preview", "help", "returned", "small", "restored",
];
const FIRST30_HOLLA_CHECKPOINTS: [&str; 7] = [
    "initial", "empty", "cleared", "help", "returned", "small", "restored",
];
const FIRST30_JACKIN_CHECKPOINTS: [&str; 10] = [
    "initial",
    "selected",
    "general",
    "mounts",
    "general_return",
    "manager_return",
    "help",
    "help_return",
    "small",
    "restored",
];
const FIRST30_TABLEPRO_CHECKPOINTS: [&str; 6] = [
    "initial", "selected", "help", "returned", "small", "restored",
];
const FIRST30_SHOWCASE_REQUIREMENTS: [&str; 7] = [
    "first30-v1/showcase.buttons/initial",
    "first30-v1/showcase.buttons/run",
    "first30-v1/showcase.buttons/preview",
    "first30-v1/showcase.buttons/help",
    "first30-v1/showcase.buttons/returned",
    "first30-v1/showcase.buttons/small",
    "first30-v1/showcase.buttons/restored",
];
const FIRST30_HOLLA_REQUIREMENTS: [&str; 7] = [
    "first30-v1/holla.first_use/initial",
    "first30-v1/holla.first_use/empty",
    "first30-v1/holla.first_use/cleared",
    "first30-v1/holla.first_use/help",
    "first30-v1/holla.first_use/returned",
    "first30-v1/holla.first_use/small",
    "first30-v1/holla.first_use/restored",
];
const FIRST30_JACKIN_REQUIREMENTS: [&str; 10] = [
    "first30-v1/jackin.settings/initial",
    "first30-v1/jackin.settings/selected",
    "first30-v1/jackin.settings/general",
    "first30-v1/jackin.settings/mounts",
    "first30-v1/jackin.settings/general_return",
    "first30-v1/jackin.settings/manager_return",
    "first30-v1/jackin.settings/help",
    "first30-v1/jackin.settings/help_return",
    "first30-v1/jackin.settings/small",
    "first30-v1/jackin.settings/restored",
];
const FIRST30_TABLEPRO_REQUIREMENTS: [&str; 6] = [
    "first30-v1/tablepro.connections/initial",
    "first30-v1/tablepro.connections/selected",
    "first30-v1/tablepro.connections/help",
    "first30-v1/tablepro.connections/returned",
    "first30-v1/tablepro.connections/small",
    "first30-v1/tablepro.connections/restored",
];
const FIRST30_HOLLA_FRAME_ONE_REQUIREMENTS: [&str; 7] = [
    "first30-v1/holla.first_use.frame1/initial",
    "first30-v1/holla.first_use.frame1/empty",
    "first30-v1/holla.first_use.frame1/cleared",
    "first30-v1/holla.first_use.frame1/help",
    "first30-v1/holla.first_use.frame1/returned",
    "first30-v1/holla.first_use.frame1/small",
    "first30-v1/holla.first_use.frame1/restored",
];
const _: () = assert!(
    FIRST30_SHOWCASE_CHECKPOINTS.len()
        + FIRST30_HOLLA_CHECKPOINTS.len()
        + FIRST30_JACKIN_CHECKPOINTS.len()
        + FIRST30_TABLEPRO_CHECKPOINTS.len()
        == FIRST30_FRAME_DENOMINATOR
);
const SHOWCASE_HELP_CHECKPOINTS: [&str; 7] = [
    "opened",
    "help-start",
    "help-scrolled",
    "returned",
    "help-reopened",
    "restored-after-modal-q",
    "exited",
];
const HOLLA_FIRST_USE_CHECKPOINTS: [&str; HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR] = [
    "opened",
    "initial-resource",
    "home-restored",
    "query-selected",
    "query-resource",
    "query-restored",
    "query-cleared",
    "exited",
];
const SHOWCASE_HELP_REQUIREMENTS: [&str; 7] = [
    "showcase-help/80x24-v1/opened",
    "showcase-help/80x24-v1/help-start",
    "showcase-help/80x24-v1/help-scrolled",
    "showcase-help/80x24-v1/returned",
    "showcase-help/80x24-v1/help-reopened",
    "showcase-help/80x24-v1/restored-after-modal-q",
    "showcase-help/80x24-v1/exited",
];
const HOLLA_FIRST_USE_ACTION_REQUIREMENTS: [&str; HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR] = [
    "holla-first-use/action-identity-v1/opened",
    "holla-first-use/action-identity-v1/initial-resource",
    "holla-first-use/action-identity-v1/home-restored",
    "holla-first-use/action-identity-v1/query-selected",
    "holla-first-use/action-identity-v1/query-resource",
    "holla-first-use/action-identity-v1/query-restored",
    "holla-first-use/action-identity-v1/query-cleared",
    "holla-first-use/action-identity-v1/exited",
];

#[derive(Clone, Copy)]
enum Journey {
    First30(First30Flow),
    ShowcaseHelp80x24,
    HollaFirstUseAction,
}

#[derive(Clone, Copy)]
enum First30Flow {
    ShowcaseButtons,
    HollaFirstUse,
    JackinSettings,
    TableProConnections,
}

#[derive(Clone, Copy)]
enum First30Step {
    Key(&'static str),
    Text(&'static str),
    MouseDown { x: u16, y: u16 },
    MouseUp { x: u16, y: u16 },
    Resize { cols: u16, rows: u16 },
    Checkpoint(&'static str),
}

const FIRST30_SHOWCASE_STEPS: &[First30Step] = &[
    First30Step::MouseDown { x: 30, y: 10 },
    First30Step::MouseUp { x: 30, y: 10 },
    First30Step::Checkpoint("run"),
    First30Step::Key("Tab"),
    First30Step::Key("Enter"),
    First30Step::Checkpoint("preview"),
    First30Step::Key("?"),
    First30Step::Checkpoint("help"),
    First30Step::Key("Escape"),
    First30Step::Checkpoint("returned"),
    First30Step::Resize { cols: 80, rows: 24 },
    First30Step::Checkpoint("small"),
    First30Step::Resize {
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
    },
    First30Step::Checkpoint("restored"),
];
const FIRST30_HOLLA_STEPS: &[First30Step] = &[
    First30Step::Text("zzzz-no-such-action"),
    First30Step::Checkpoint("empty"),
    First30Step::Key("Escape"),
    First30Step::Checkpoint("cleared"),
    First30Step::Key("F1"),
    First30Step::Checkpoint("help"),
    First30Step::Key("Escape"),
    First30Step::Checkpoint("returned"),
    First30Step::Resize { cols: 80, rows: 24 },
    First30Step::Checkpoint("small"),
    First30Step::Resize {
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
    },
    First30Step::Checkpoint("restored"),
];
const FIRST30_JACKIN_STEPS: &[First30Step] = &[
    First30Step::Key("Down"),
    First30Step::Checkpoint("selected"),
    First30Step::Key("s"),
    First30Step::Checkpoint("general"),
    First30Step::Key("]"),
    First30Step::Checkpoint("mounts"),
    First30Step::Key("["),
    First30Step::Checkpoint("general_return"),
    First30Step::Key("Escape"),
    First30Step::Checkpoint("manager_return"),
    First30Step::Key("?"),
    First30Step::Checkpoint("help"),
    First30Step::Key("Escape"),
    First30Step::Checkpoint("help_return"),
    First30Step::Resize { cols: 80, rows: 24 },
    First30Step::Checkpoint("small"),
    First30Step::Resize {
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
    },
    First30Step::Checkpoint("restored"),
];
const FIRST30_TABLEPRO_STEPS: &[First30Step] = &[
    First30Step::Key("Down"),
    First30Step::Checkpoint("selected"),
    First30Step::Key("F1"),
    First30Step::Checkpoint("help"),
    First30Step::Key("Escape"),
    First30Step::Checkpoint("returned"),
    First30Step::Resize { cols: 80, rows: 24 },
    First30Step::Checkpoint("small"),
    First30Step::Resize {
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
    },
    First30Step::Checkpoint("restored"),
];

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    test_name: &'static str,
    inventory_class: &'static str,
    selected_in_primary_lane: bool,
    binary: &'static str,
    args: &'static [&'static str],
    initial_marker: &'static str,
    help_key: &'static str,
    help_marker: &'static str,
    quit_key: &'static str,
    cols: u16,
    rows: u16,
    terminal_profile: &'static str,
    fixture_path: &'static str,
    checkpoint_labels: &'static [&'static str],
    requirement_ids: &'static [&'static str],
    exit_receipt_label: &'static str,
    exit_receipt_is_separate: bool,
    journey: Journey,
}

const CASES: [Case; FIRST30_PRIMARY_CASE_DENOMINATOR + 1] = [
    Case {
        id: "first30-v1/showcase.buttons",
        test_name: "first30_showcase_buttons",
        inventory_class: "primary",
        selected_in_primary_lane: true,
        binary: "showcase",
        args: &[
            "--page",
            "buttons",
            "--motion",
            "paused",
            "--frame",
            "0",
            "--color",
            "truecolor",
        ],
        initial_marker: "Buttons",
        help_key: "?",
        help_marker: "Keyboard & mouse",
        quit_key: "q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        terminal_profile: "first30-v1",
        fixture_path: "data/first30-v1/showcase.buttons",
        checkpoint_labels: &FIRST30_SHOWCASE_CHECKPOINTS,
        requirement_ids: &FIRST30_SHOWCASE_REQUIREMENTS,
        exit_receipt_label: "exit-receipt",
        exit_receipt_is_separate: true,
        journey: Journey::First30(First30Flow::ShowcaseButtons),
    },
    Case {
        id: "first30-v1/holla.first_use",
        test_name: "first30_holla_first_use",
        inventory_class: "primary",
        selected_in_primary_lane: true,
        binary: "holla",
        args: &[
            "--scenario",
            "first-use",
            "--motion",
            "paused",
            "--frame",
            "0",
            "--color",
            "truecolor",
        ],
        initial_marker: "holla❯",
        help_key: "F1",
        help_marker: "Key reference",
        quit_key: "Ctrl+q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        terminal_profile: "first30-v1",
        fixture_path: "data/first30-v1/holla.first_use",
        checkpoint_labels: &FIRST30_HOLLA_CHECKPOINTS,
        requirement_ids: &FIRST30_HOLLA_REQUIREMENTS,
        exit_receipt_label: "exit-receipt",
        exit_receipt_is_separate: true,
        journey: Journey::First30(First30Flow::HollaFirstUse),
    },
    Case {
        id: "first30-v1/jackin.settings",
        test_name: "first30_jackin_settings",
        inventory_class: "primary",
        selected_in_primary_lane: true,
        binary: "jackin-preview",
        args: &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "0",
            "--color",
            "truecolor",
        ],
        initial_marker: "Current directory",
        help_key: "?",
        help_marker: "Keyboard shortcuts",
        quit_key: "q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        terminal_profile: "first30-v1",
        fixture_path: "data/first30-v1/jackin.settings",
        checkpoint_labels: &FIRST30_JACKIN_CHECKPOINTS,
        requirement_ids: &FIRST30_JACKIN_REQUIREMENTS,
        exit_receipt_label: "exit-receipt",
        exit_receipt_is_separate: true,
        journey: Journey::First30(First30Flow::JackinSettings),
    },
    Case {
        id: "first30-v1/tablepro.connections",
        test_name: "first30_tablepro_connections",
        inventory_class: "primary",
        selected_in_primary_lane: true,
        binary: "tablepro",
        args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
        initial_marker: "TablePro",
        help_key: "F1",
        help_marker: "Keyboard shortcuts",
        quit_key: "Ctrl+q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        terminal_profile: "first30-v1",
        fixture_path: "data/first30-v1/tablepro.connections",
        checkpoint_labels: &FIRST30_TABLEPRO_CHECKPOINTS,
        requirement_ids: &FIRST30_TABLEPRO_REQUIREMENTS,
        exit_receipt_label: "exit-receipt",
        exit_receipt_is_separate: true,
        journey: Journey::First30(First30Flow::TableProConnections),
    },
    Case {
        id: "first30-v1/holla.first_use.frame1",
        test_name: "holla_first_use_frame_one_variant",
        inventory_class: "additional_variant",
        selected_in_primary_lane: false,
        binary: "holla",
        args: &[
            "--scenario",
            "first-use",
            "--motion",
            "paused",
            "--frame",
            "1",
            "--color",
            "truecolor",
        ],
        initial_marker: "holla❯",
        help_key: "F1",
        help_marker: "Key reference",
        quit_key: "Ctrl+q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        terminal_profile: "first30-v1",
        fixture_path: "data/first30-v1/holla.first_use.frame1",
        checkpoint_labels: &FIRST30_HOLLA_CHECKPOINTS,
        requirement_ids: &FIRST30_HOLLA_FRAME_ONE_REQUIREMENTS,
        exit_receipt_label: "exit-receipt",
        exit_receipt_is_separate: true,
        journey: Journey::First30(First30Flow::HollaFirstUse),
    },
];

const SHOWCASE_HELP_CASE: Case = Case {
    id: "showcase-help/80x24-v1",
    test_name: "showcase_help_paging",
    inventory_class: "retained_requirement",
    selected_in_primary_lane: false,
    binary: "showcase",
    args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
    initial_marker: "Overview",
    help_key: "?",
    help_marker: "Keyboard & mouse",
    quit_key: "q",
    cols: 80,
    rows: 24,
    terminal_profile: "retained-showcase-help-v1",
    fixture_path: "data/showcase-help/80x24-v1",
    checkpoint_labels: &SHOWCASE_HELP_CHECKPOINTS,
    requirement_ids: &SHOWCASE_HELP_REQUIREMENTS,
    exit_receipt_label: "exited",
    exit_receipt_is_separate: false,
    journey: Journey::ShowcaseHelp80x24,
};

const HOLLA_FIRST_USE_ACTION_CASE: Case = Case {
    id: "holla-first-use/action-identity-v1",
    test_name: "holla_first_use_action",
    inventory_class: "retained_requirement",
    selected_in_primary_lane: false,
    binary: "holla",
    args: &[
        "--scenario",
        "first-use",
        "--motion",
        "paused",
        "--frame",
        "0",
        "--color",
        "truecolor",
    ],
    initial_marker: "holla❯",
    help_key: "",
    help_marker: "",
    quit_key: "Ctrl+q",
    cols: DEFAULT_COLS,
    rows: DEFAULT_ROWS,
    terminal_profile: "retained-holla-action-v1",
    fixture_path: "data/holla-first-use/action-identity-v1",
    checkpoint_labels: &HOLLA_FIRST_USE_CHECKPOINTS,
    requirement_ids: &HOLLA_FIRST_USE_ACTION_REQUIREMENTS,
    exit_receipt_label: "exited",
    exit_receipt_is_separate: false,
    journey: Journey::HollaFirstUseAction,
};

#[derive(Debug)]
struct Checkpoint {
    label: String,
    checkpoint_id: String,
    sample_id: String,
    offset: u64,
    captured_at_unix_ms: u128,
    snapshot: TerminalSnapshot,
    sample_bytes: Vec<u8>,
    png_sha256: String,
    rgba_sha256: String,
}

struct FailureScreenEvidence {
    record: Value,
    snapshot: TerminalSnapshot,
    state_observation: Value,
    child_active: bool,
    screen_observable: bool,
}

pub(crate) type AppResult<T> = Result<T, Box<dyn Error>>;

pub(crate) fn run_case_id(case_id: &str) -> AppResult<()> {
    let case = case_by_id(case_id).ok_or_else(|| format!("unknown functional case {case_id:?}"))?;
    let suite = suite_root()?;
    let bin_dir = required_absolute_directory("TERMROCK_PARITY_BIN_DIR")?;
    let out_root = required_absolute_directory("TERMROCK_PARITY_OUT_DIR")?;
    let case_out = out_root.join(case.id);
    let export_provenance = verify_export(&bin_dir, &case_out)?;
    prepare_empty_output(&case_out)?;

    let case_inventory = match load_and_validate_case_inventory(&suite) {
        Ok(inventory) => inventory,
        Err(error) => {
            write_json_new(
                &case_out.join("setup-error.json"),
                &json!({"stage":"case-inventory","error":error.to_string()}),
            )?;
            return Err(error);
        }
    };

    let source_manifest = match source_manifest::load_verified(&bin_dir) {
        Ok(manifest) => manifest,
        Err(error) => {
            write_json_new(
                &case_out.join("setup-error.json"),
                &json!({"stage":"source-manifest","error":error.to_string()}),
            )?;
            return Err(error);
        }
    };
    if let Err(error) = verify_application_root(&source_manifest, &export_provenance) {
        write_json_new(
            &case_out.join("setup-error.json"),
            &json!({"stage":"application-source-binding","error":error.to_string()}),
        )?;
        return Err(error);
    }

    let cwd = case_out.join("working-directory");
    if let Err(error) = copy_case_fixture(&suite.join(case.fixture_path), &cwd) {
        write_json_new(
            &case_out.join("setup-error.json"),
            &json!({"stage":"fixture-copy","error":error.to_string()}),
        )?;
        return Err(error);
    }
    let (fixture_sha256, fixture_files) = match fixture_tree_hash(&cwd) {
        Ok(evidence) => evidence,
        Err(error) => {
            write_json_new(
                &case_out.join("setup-error.json"),
                &json!({"stage":"fixture-hash","error":error.to_string()}),
            )?;
            return Err(error);
        }
    };
    let frozen_store =
        match FrozenStore::new(&suite.join("expected"), &case_out.join("frozen-checks")) {
            Ok(store) => store,
            Err(error) => {
                write_json_new(
                    &case_out.join("setup-error.json"),
                    &json!({"stage":"frozen-store","error":error.to_string()}),
                )?;
                return Err(error.into());
            }
        };
    let executable = std::env::current_exe()?.canonicalize()?;
    let executable_sha256_before = sha256_file(&executable)?;
    let execution = json!({
        "test_name":case.test_name,
        "case_class":case.inventory_class,
        "terminal_profile":case.terminal_profile,
        "compiled_export_sha256": BUILD_EXPORT_SHA256,
        "case_inventory_sha256":sha256_file(&suite.join(CASE_INVENTORY_FILE))?,
        "terminal_profiles_sha256":sha256_file(&suite.join(TERMINAL_PROFILES_FILE))?,
        "suite_lock_sha256": export_provenance["suite_lock_sha256"],
        "tuiscotti_commit": export_provenance["tuiscotti_commit"],
        "engine": export_provenance["engine"],
        "application_source": source_manifest,
        "test_executable": executable,
        "test_executable_sha256_before": executable_sha256_before,
        "bin_dir": bin_dir,
        "out_dir": out_root,
        "case_output": case_out,
        "working_directory":cwd,
        "fixture_source":suite.join(case.fixture_path),
        "fixture_sha256":fixture_sha256,
        "fixture_files":fixture_files,
    });
    let result = run_case(
        case,
        &bin_dir.join(case.binary),
        &suite,
        &case_out,
        &cwd,
        &frozen_store,
        &execution,
        &case_inventory,
    )?;
    let executable_sha256_after = sha256_file(&executable)?;
    let execution_record = json!({
        "schema":"termrock-functional-execution.v1",
        "case_id":case.id,
        "test_name":case.test_name,
        "case_class":case.inventory_class,
        "terminal_profile":case.terminal_profile,
        "compiled_export_sha256":BUILD_EXPORT_SHA256,
        "case_inventory_sha256":sha256_file(&suite.join(CASE_INVENTORY_FILE))?,
        "terminal_profiles_sha256":sha256_file(&suite.join(TERMINAL_PROFILES_FILE))?,
        "test_executable":executable,
        "test_executable_sha256_before":executable_sha256_before,
        "test_executable_sha256_after":executable_sha256_after,
        "test_executable_unchanged":executable_sha256_before == executable_sha256_after,
        "suite_lock_sha256":export_provenance["suite_lock_sha256"],
        "tuiscotti_commit":export_provenance["tuiscotti_commit"],
        "engine":export_provenance["engine"],
        "bin_dir":bin_dir,
        "out_dir":out_root,
        "case_output":case_out,
        "working_directory":cwd,
        "fixture_source":suite.join(case.fixture_path),
        "fixture_sha256":fixture_sha256,
        "fixture_files":fixture_files,
    });
    write_json_new(
        &case_out.join("execution-provenance.json"),
        &execution_record,
    )?;
    if executable_sha256_before != executable_sha256_after {
        return Err("functional test executable changed during the test".into());
    }
    if result["status"] != "passed" {
        return Err(format!(
            "case {} failed after saving evidence under {}: {}",
            case.id,
            case_out.display(),
            result["error"]
        )
        .into());
    }
    Ok(())
}

fn case_by_id(case_id: &str) -> Option<Case> {
    all_cases().into_iter().find(|case| case.id == case_id)
}

fn all_cases() -> Vec<Case> {
    CASES
        .iter()
        .copied()
        .chain([SHOWCASE_HELP_CASE, HOLLA_FIRST_USE_ACTION_CASE])
        .collect()
}

fn load_and_validate_case_inventory(suite: &Path) -> AppResult<toml::Value> {
    let cases_text = fs::read_to_string(suite.join(CASE_INVENTORY_FILE))?;
    let cases: toml::Value = toml::from_str(&cases_text)?;
    let profiles_text = fs::read_to_string(suite.join(TERMINAL_PROFILES_FILE))?;
    let profiles: toml::Value = toml::from_str(&profiles_text)?;
    validate_inventory_document(&cases, &profiles)?;
    Ok(cases)
}

fn validate_inventory_document(cases: &toml::Value, profiles: &toml::Value) -> AppResult<()> {
    require_toml_keys(
        cases,
        &[
            "schema",
            "protocol_version",
            "sample_id_template",
            "historical_protocol",
            "historical_case_ids",
            "historical_live_frame_count",
            "preserved_reference_sample_count",
            "transition_reason",
            "primary_case_count",
            "primary_live_frame_count",
            "primary_exit_receipt_count",
            "full_application_inventory_ready",
            "full_inventory_test",
            "full_inventory_status",
            "case",
            "not_run_requirement",
        ],
        CASE_INVENTORY_FILE,
    )?;
    require_toml_string(cases, "schema", CASE_INVENTORY_SCHEMA)?;
    require_toml_string(cases, "protocol_version", "first30-v1")?;
    require_toml_string(
        cases,
        "sample_id_template",
        "{case_id}/{checkpoint_label}/pixels-{platform_key}",
    )?;
    require_toml_string(cases, "historical_protocol", "first-v1")?;
    require_toml_string_array(
        cases,
        "historical_case_ids",
        &[
            "first/showcase",
            "first/holla",
            "first/jackin",
            "first/tablepro",
        ],
    )?;
    require_toml_integer(cases, "historical_live_frame_count", 16)?;
    require_toml_integer(cases, "preserved_reference_sample_count", 29)?;
    let transition_reason = toml_field(cases, "transition_reason")?
        .as_str()
        .ok_or("case inventory transition_reason must be a string")?;
    if !transition_reason.contains("Preserve the old runs and failures as history.") {
        return Err("case inventory must keep old runs and failures as history".into());
    }
    require_toml_integer(
        cases,
        "primary_case_count",
        FIRST30_PRIMARY_CASE_DENOMINATOR as i64,
    )?;
    require_toml_integer(
        cases,
        "primary_live_frame_count",
        FIRST30_FRAME_DENOMINATOR as i64,
    )?;
    require_toml_integer(
        cases,
        "primary_exit_receipt_count",
        FIRST30_PRIMARY_CASE_DENOMINATOR as i64,
    )?;
    require_toml_bool(cases, "full_application_inventory_ready", false)?;
    require_toml_string(cases, "full_inventory_test", "required_inventory_ready")?;
    let full_inventory_status = toml_field(cases, "full_inventory_status")?
        .as_str()
        .ok_or("case inventory full_inventory_status must be a string")?;
    if !full_inventory_status.contains("NOT VERIFIED") {
        return Err("case inventory must keep full inventory status NOT VERIFIED".into());
    }

    require_toml_keys(profiles, &["schema", "profiles"], TERMINAL_PROFILES_FILE)?;
    require_toml_string(profiles, "schema", TERMINAL_PROFILES_SCHEMA)?;
    let profile_table = toml_field(profiles, "profiles")?;
    require_toml_keys(
        profile_table,
        &[
            "first30-v1",
            "retained-showcase-help-v1",
            "retained-holla-action-v1",
        ],
        "terminal profiles",
    )?;

    let case_entries = toml_field(cases, "case")?
        .as_array()
        .ok_or("case inventory case entries must be an array")?;
    let runtime_cases = all_cases();
    if case_entries.len() != runtime_cases.len() {
        return Err(format!(
            "case inventory declares {} cases, Rust registers {}",
            case_entries.len(),
            runtime_cases.len()
        )
        .into());
    }
    let registered_case_ids = runtime_cases.iter().map(|case| case.id).collect::<Vec<_>>();
    if registered_case_ids.as_slice() != RUNTIME_CASE_IDS.as_slice() {
        return Err(format!(
            "Rust case registry differs from the fixed inventory order: {registered_case_ids:?}"
        )
        .into());
    }
    let functional_tests = super::FUNCTIONAL_CASE_TESTS;
    if super::AUXILIARY_TEST_NAMES.len() != 1
        || super::AUXILIARY_TEST_NAMES[0] != "required_inventory_ready"
    {
        return Err(format!(
            "auxiliary functional test inventory changed: {:?}",
            super::AUXILIARY_TEST_NAMES
        )
        .into());
    }
    if functional_tests.len() != runtime_cases.len() {
        return Err(format!(
            "functional target declares {} case tests, Rust registers {} cases",
            functional_tests.len(),
            runtime_cases.len()
        )
        .into());
    }
    let selected_test_names = functional_tests
        .iter()
        .filter(|(test_name, _)| test_name.starts_with("first30_"))
        .map(|(test_name, _)| *test_name)
        .collect::<Vec<_>>();
    if selected_test_names.as_slice()
        != [
            "first30_showcase_buttons",
            "first30_holla_first_use",
            "first30_jackin_settings",
            "first30_tablepro_connections",
        ]
        .as_slice()
    {
        return Err(format!(
            "first30 test filter selects unexpected tests: {selected_test_names:?}"
        )
        .into());
    }
    let mut seen_ids = Vec::with_capacity(case_entries.len());
    let mut primary_cases = 0usize;
    let mut primary_frames = 0usize;
    let mut primary_exit_receipts = 0usize;
    let mut additional_variants = 0usize;
    let mut retained_requirements = 0usize;
    for (entry, case) in case_entries.iter().zip(runtime_cases.iter().copied()) {
        require_toml_keys(
            entry,
            &[
                "id",
                "test_name",
                "class",
                "selected_in_primary_lane",
                "binary",
                "args",
                "initial_marker",
                "help_key",
                "help_marker",
                "quit_key",
                "terminal_profile",
                "fixture_path",
                "checkpoint_labels",
                "requirement_ids",
                "exit_receipt_label",
                "exit_receipt_key",
                "exit_receipt_is_separate",
            ],
            "case entry",
        )?;
        require_toml_string(entry, "id", case.id)?;
        if seen_ids.contains(&case.id) {
            return Err(format!("duplicate case id {:?}", case.id).into());
        }
        seen_ids.push(case.id);
        require_toml_string(entry, "test_name", case.test_name)?;
        require_toml_string(entry, "class", case.inventory_class)?;
        require_toml_bool(
            entry,
            "selected_in_primary_lane",
            case.selected_in_primary_lane,
        )?;
        require_toml_string(entry, "binary", case.binary)?;
        require_toml_string_array(entry, "args", case.args)?;
        require_toml_string(entry, "initial_marker", case.initial_marker)?;
        require_toml_string(entry, "help_key", case.help_key)?;
        require_toml_string(entry, "help_marker", case.help_marker)?;
        require_toml_string(entry, "quit_key", case.quit_key)?;
        require_toml_string(entry, "terminal_profile", case.terminal_profile)?;
        require_toml_string(entry, "fixture_path", case.fixture_path)?;
        require_toml_string_array(entry, "checkpoint_labels", case.checkpoint_labels)?;
        require_toml_string_array(entry, "requirement_ids", case.requirement_ids)?;
        require_toml_string(entry, "exit_receipt_label", case.exit_receipt_label)?;
        require_toml_string(entry, "exit_receipt_key", case.quit_key)?;
        require_toml_bool(
            entry,
            "exit_receipt_is_separate",
            case.exit_receipt_is_separate,
        )?;
        validate_case_checkpoint_registry(case)?;
        if !functional_tests.iter().any(|(test_name, registered_id)| {
            *test_name == case.test_name && *registered_id == case.id
        }) {
            return Err(format!(
                "case {:?} does not match a discovered functional test name and ID",
                case.id
            )
            .into());
        }
        validate_case_terminal_profile(profiles, case)?;
        match case.inventory_class {
            "primary" => {
                if !case.selected_in_primary_lane || !matches!(case.journey, Journey::First30(_)) {
                    return Err(format!(
                        "primary case {:?} is not a selected first30 journey",
                        case.id
                    )
                    .into());
                }
                primary_cases += 1;
                primary_frames += case.checkpoint_labels.len();
                primary_exit_receipts += usize::from(case.exit_receipt_is_separate);
            }
            "additional_variant" => {
                if case.selected_in_primary_lane
                    || case.id != "first30-v1/holla.first_use.frame1"
                    || case.checkpoint_labels.len() != 7
                    || !case.exit_receipt_is_separate
                {
                    return Err(
                        format!("additional variant {:?} has invalid scope", case.id).into(),
                    );
                }
                additional_variants += 1;
            }
            "retained_requirement" => {
                if case.selected_in_primary_lane || case.exit_receipt_is_separate {
                    return Err(
                        format!("retained requirement {:?} has invalid scope", case.id).into(),
                    );
                }
                retained_requirements += 1;
            }
            other => return Err(format!("unknown case class {other:?}").into()),
        }
    }
    if primary_cases != FIRST30_PRIMARY_CASE_DENOMINATOR
        || primary_frames != FIRST30_FRAME_DENOMINATOR
        || primary_exit_receipts != FIRST30_PRIMARY_CASE_DENOMINATOR
        || additional_variants != 1
        || retained_requirements != 2
    {
        return Err(format!(
            "inventory totals changed: primary cases={primary_cases}, frames={primary_frames}, exit receipts={primary_exit_receipts}, variants={additional_variants}, retained requirements={retained_requirements}"
        )
        .into());
    }
    let primary_ids = runtime_cases
        .iter()
        .filter(|case| case.inventory_class == "primary")
        .map(|case| case.id)
        .collect::<Vec<_>>();
    if primary_ids.as_slice() != FIRST30_PRIMARY_CASE_IDS.as_slice() {
        return Err(format!("primary case IDs changed: {primary_ids:?}").into());
    }
    validate_frame_one_variant(&runtime_cases)?;
    validate_not_run_requirements(cases)?;
    Ok(())
}

fn validate_frame_one_variant(cases: &[Case]) -> AppResult<()> {
    let primary = cases
        .iter()
        .find(|case| case.id == "first30-v1/holla.first_use")
        .ok_or("primary Holla first-use case is missing")?;
    let variant = cases
        .iter()
        .find(|case| case.id == "first30-v1/holla.first_use.frame1")
        .ok_or("Holla frame-one variant is missing")?;
    let mut expected = primary.args.to_vec();
    let frame_option = expected
        .iter()
        .position(|argument| *argument == "--frame")
        .ok_or("primary Holla case has no frame argument")?;
    let value_index = frame_option + 1;
    if expected.get(value_index) != Some(&"0") {
        return Err("primary Holla first-use case must use frame 0".into());
    }
    expected[value_index] = "1";
    if variant.args != expected.as_slice() {
        return Err(
            "Holla frame-one variant must differ from the primary case only at --frame".into(),
        );
    }
    Ok(())
}

fn validate_case_checkpoint_registry(case: Case) -> AppResult<()> {
    if case.requirement_ids.len() != case.checkpoint_labels.len() {
        return Err(format!(
            "case {:?} has {} named requirements for {} checkpoints",
            case.id,
            case.requirement_ids.len(),
            case.checkpoint_labels.len()
        )
        .into());
    }
    for (label, requirement_id) in case
        .checkpoint_labels
        .iter()
        .zip(case.requirement_ids.iter())
    {
        let expected = format!("{}/{label}", case.id);
        if *requirement_id != expected.as_str() {
            return Err(format!(
                "case {:?} checkpoint {label:?} has requirement ID {requirement_id:?}, expected {expected:?}",
                case.id
            )
            .into());
        }
    }
    if let Journey::First30(flow) = case.journey {
        let steps = match flow {
            First30Flow::ShowcaseButtons => FIRST30_SHOWCASE_STEPS,
            First30Flow::HollaFirstUse => FIRST30_HOLLA_STEPS,
            First30Flow::JackinSettings => FIRST30_JACKIN_STEPS,
            First30Flow::TableProConnections => FIRST30_TABLEPRO_STEPS,
        };
        let mut labels = vec!["initial"];
        labels.extend(steps.iter().filter_map(|step| match step {
            First30Step::Checkpoint(label) => Some(*label),
            _ => None,
        }));
        if labels.as_slice() != case.checkpoint_labels {
            return Err(format!(
                "case {:?} step checkpoints differ from its declared label order: {labels:?}",
                case.id
            )
            .into());
        }
    }
    Ok(())
}

fn validate_case_terminal_profile(profiles: &toml::Value, case: Case) -> AppResult<()> {
    let profile = toml_field(toml_field(profiles, "profiles")?, case.terminal_profile)?;
    let is_first30 = matches!(case.journey, Journey::First30(_));
    let profile_keys: &[&str] = if is_first30 {
        &[
            "cols",
            "rows",
            "small_cols",
            "small_rows",
            "readiness_timeout_ms",
            "quiet_timeout_ms",
            "quiet_interval_ms",
            "exit_timeout_ms",
            "theme",
            "no_color",
            "common_cli_args",
            "child_environment",
        ]
    } else {
        &[
            "cols",
            "rows",
            "readiness_timeout_ms",
            "quiet_timeout_ms",
            "quiet_interval_ms",
            "exit_timeout_ms",
            "theme",
            "no_color",
            "common_cli_args",
            "child_environment",
        ]
    };
    require_toml_keys(profile, profile_keys, case.terminal_profile)?;
    require_toml_integer(profile, "cols", i64::from(case.cols))?;
    require_toml_integer(profile, "rows", i64::from(case.rows))?;
    if is_first30 {
        require_toml_integer(profile, "small_cols", 80)?;
        require_toml_integer(profile, "small_rows", 24)?;
    }
    require_toml_integer(
        profile,
        "readiness_timeout_ms",
        if is_first30 {
            FIRST30_READY_TIMEOUT_MS as i64
        } else {
            HELP_TIMEOUT_MS as i64
        },
    )?;
    require_toml_integer(
        profile,
        "quiet_timeout_ms",
        QUIET_TIMEOUT.as_millis() as i64,
    )?;
    require_toml_integer(
        profile,
        "quiet_interval_ms",
        QUIET_INTERVAL.as_millis() as i64,
    )?;
    require_toml_integer(profile, "exit_timeout_ms", EXIT_TIMEOUT_MS as i64)?;
    require_toml_string(profile, "theme", "default")?;
    require_toml_string(profile, "no_color", "unset")?;
    require_toml_string_array(
        profile,
        "common_cli_args",
        &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
    )?;
    let environment = toml_field(profile, "child_environment")?;
    require_toml_keys(
        environment,
        &["TERM", "COLORTERM", "LANG", "LC_ALL", "TZ"],
        "terminal child_environment",
    )?;
    for (name, value) in [
        ("TERM", "xterm-256color"),
        ("COLORTERM", "truecolor"),
        ("LANG", "C.UTF-8"),
        ("LC_ALL", "C.UTF-8"),
        ("TZ", "UTC"),
    ] {
        require_toml_string(environment, name, value)?;
    }
    let frame_value = if case.id == "first30-v1/holla.first_use.frame1" {
        "1"
    } else {
        "0"
    };
    let required_args = [
        "--motion",
        "paused",
        "--frame",
        frame_value,
        "--color",
        "truecolor",
    ];
    if !case
        .args
        .windows(required_args.len())
        .any(|window| window == required_args.as_slice())
    {
        return Err(format!(
            "case {:?} does not use its terminal profile CLI arguments",
            case.id
        )
        .into());
    }
    Ok(())
}

fn validate_not_run_requirements(cases: &toml::Value) -> AppResult<()> {
    const EXPECTED: [(&str, &str, &str); 4] = [
        (
            "showcase-help/80x24-v1",
            "showcase_help_paging",
            "not run when unselected",
        ),
        (
            "holla-first-use/action-identity-v1",
            "holla_first_use_action",
            "not run when unselected",
        ),
        (
            "first30-v1/holla.first_use.frame1",
            "holla_first_use_frame_one_variant",
            "separate seven-frame case; outside the primary denominator",
        ),
        (
            "full-application-inventory",
            "required_inventory_ready",
            "NOT VERIFIED; remains failing until full inventory evidence exists",
        ),
    ];
    let entries = toml_field(cases, "not_run_requirement")?
        .as_array()
        .ok_or("case inventory not_run_requirement entries must be an array")?;
    if entries.len() != EXPECTED.len() {
        return Err(format!(
            "case inventory declares {} not-run requirements, expected {}",
            entries.len(),
            EXPECTED.len()
        )
        .into());
    }
    for (entry, (id, test_name, status)) in entries.iter().zip(EXPECTED) {
        require_toml_keys(
            entry,
            &["id", "test_name", "selected_by_first30_filter", "status"],
            "not-run requirement",
        )?;
        require_toml_string(entry, "id", id)?;
        require_toml_string(entry, "test_name", test_name)?;
        require_toml_bool(entry, "selected_by_first30_filter", false)?;
        require_toml_string(entry, "status", status)?;
        if !super::FUNCTIONAL_CASE_TESTS
            .iter()
            .any(|(registered_name, _)| *registered_name == test_name)
            && !super::AUXILIARY_TEST_NAMES.contains(&test_name)
        {
            return Err(format!(
                "not-run requirement {id:?} has no matching Nextest test {test_name:?}"
            )
            .into());
        }
    }
    Ok(())
}

fn toml_field<'a>(value: &'a toml::Value, field: &str) -> AppResult<&'a toml::Value> {
    value
        .get(field)
        .ok_or_else(|| format!("missing TOML field {field:?}").into())
}

fn require_toml_keys(value: &toml::Value, expected: &[&str], context: &str) -> AppResult<()> {
    let table = value
        .as_table()
        .ok_or_else(|| format!("{context} must be a TOML table"))?;
    if table.len() != expected.len() || expected.iter().any(|key| !table.contains_key(*key)) {
        let mut actual = table.keys().map(String::as_str).collect::<Vec<_>>();
        actual.sort_unstable();
        let mut required = expected.to_vec();
        required.sort_unstable();
        return Err(
            format!("{context} fields differ: required={required:?}, actual={actual:?}").into(),
        );
    }
    Ok(())
}

fn require_toml_string(value: &toml::Value, field: &str, expected: &str) -> AppResult<()> {
    let actual = toml_field(value, field)?
        .as_str()
        .ok_or_else(|| format!("TOML field {field:?} must be a string"))?;
    if actual != expected {
        return Err(format!("TOML field {field:?} is {actual:?}, expected {expected:?}").into());
    }
    Ok(())
}

fn require_toml_string_array(value: &toml::Value, field: &str, expected: &[&str]) -> AppResult<()> {
    let actual = toml_field(value, field)?
        .as_array()
        .ok_or_else(|| format!("TOML field {field:?} must be an array"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("TOML field {field:?} must contain only strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected = expected
        .iter()
        .map(|entry| (*entry).to_owned())
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(format!("TOML field {field:?} is {actual:?}, expected {expected:?}").into());
    }
    Ok(())
}

fn require_toml_integer(value: &toml::Value, field: &str, expected: i64) -> AppResult<()> {
    let actual = toml_field(value, field)?
        .as_integer()
        .ok_or_else(|| format!("TOML field {field:?} must be an integer"))?;
    if actual != expected {
        return Err(format!("TOML field {field:?} is {actual}, expected {expected}").into());
    }
    Ok(())
}

fn require_toml_bool(value: &toml::Value, field: &str, expected: bool) -> AppResult<()> {
    let actual = toml_field(value, field)?
        .as_bool()
        .ok_or_else(|| format!("TOML field {field:?} must be a boolean"))?;
    if actual != expected {
        return Err(format!("TOML field {field:?} is {actual}, expected {expected}").into());
    }
    Ok(())
}

fn required_absolute_directory(name: &str) -> AppResult<PathBuf> {
    let value = std::env::var_os(name).ok_or_else(|| format!("{name} is required"))?;
    let path = PathBuf::from(value);
    if !path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
    {
        return Err(format!("{name} must be a normalized absolute path").into());
    }
    let canonical = path.canonicalize()?;
    if canonical != path || !canonical.is_dir() {
        return Err(format!("{name} must name a canonical directory").into());
    }
    Ok(canonical)
}

fn copy_case_fixture(source: &Path, destination: &Path) -> AppResult<()> {
    let source_info = fs::symlink_metadata(source)?;
    if source_info.file_type().is_symlink() || !source_info.is_dir() {
        return Err(format!(
            "case fixture is not a regular directory: {}",
            source.display()
        )
        .into());
    }
    if fs::symlink_metadata(destination).is_ok() {
        return Err(format!(
            "case working directory already exists: {}",
            destination.display()
        )
        .into());
    }
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let info = fs::symlink_metadata(&source_path)?;
        if info.file_type().is_symlink() {
            return Err(
                format!("case fixture contains a symlink: {}", source_path.display()).into(),
            );
        }
        if info.is_dir() {
            copy_case_fixture(&source_path, &destination_path)?;
        } else if info.is_file() {
            write_new(&destination_path, &fs::read(&source_path)?)?;
            fs::set_permissions(&destination_path, info.permissions())?;
        } else {
            return Err(format!(
                "case fixture contains a special file: {}",
                source_path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn fixture_tree_hash(root: &Path) -> AppResult<(String, Vec<Value>)> {
    fn visit(
        root: &Path,
        directory: &Path,
        records: &mut Vec<Value>,
        digest_input: &mut Vec<u8>,
    ) -> AppResult<()> {
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path.strip_prefix(root)?;
            let relative = relative
                .components()
                .map(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .ok_or("fixture path is not valid UTF-8")
                })
                .collect::<Result<Vec<_>, _>>()?
                .join("/");
            let info = fs::symlink_metadata(&path)?;
            if info.file_type().is_symlink() {
                return Err(format!("fixture contains a symlink: {}", path.display()).into());
            }
            if info.is_dir() {
                digest_input.extend_from_slice(format!("directory\0{relative}\0").as_bytes());
                records.push(json!({"path":relative,"kind":"directory"}));
                visit(root, &path, records, digest_input)?;
            } else if info.is_file() {
                let bytes = fs::read(&path)?;
                digest_input
                    .extend_from_slice(format!("file\0{relative}\0{}\0", bytes.len()).as_bytes());
                digest_input.extend_from_slice(&bytes);
                records.push(json!({
                    "path":relative,
                    "kind":"file",
                    "size":bytes.len(),
                    "sha256":sha256(&bytes),
                }));
            } else {
                return Err(format!("fixture contains a special file: {}", path.display()).into());
            }
        }
        Ok(())
    }

    let mut records = Vec::new();
    let mut digest_input = Vec::new();
    visit(root, root, &mut records, &mut digest_input)?;
    Ok((sha256(&digest_input), records))
}

fn run_case(
    case: Case,
    binary: &Path,
    suite: &Path,
    case_out: &Path,
    cwd: &Path,
    frozen_store: &FrozenStore,
    execution: &Value,
    case_inventory: &toml::Value,
) -> AppResult<Value> {
    let case_out = case_out.to_path_buf();
    let cwd = cwd.to_path_buf();
    let case_start = Instant::now();
    let started_at = unix_ms();
    let raw_path = case_out.join("raw.ansi");
    let session = Session::new(format!("termrock-live-{}", case.id.replace('/', "-")));
    let mut spawned = false;
    let mut child_exited = false;
    let mut final_quit_attempted = false;
    let mut checkpoints = Vec::with_capacity(case.checkpoint_labels.len());
    let mut exit_receipt: Option<Checkpoint> = None;
    let mut actions = Vec::new();
    let mut semantic_assertions = Vec::new();
    let mut failure_finalization = Value::Null;
    let mut process_state = Value::Null;
    let mut exit_validation_errors = Vec::new();
    let work = (|| -> AppResult<()> {
        let launch = LaunchOptions::new(binary.as_os_str().to_owned())
            .with_args(case.args.iter().copied())
            .with_cwd(&cwd)
            .with_env_mode(LaunchEnvMode::Cleared)
            .with_envs([
                ("LANG", "C.UTF-8"),
                ("LC_ALL", "C.UTF-8"),
                ("TZ", "UTC"),
                ("TERM", "xterm-256color"),
                ("COLORTERM", "truecolor"),
            ]);
        let run_options = RunOptions {
            backend: Backend::default(),
            program: String::new(),
            args: Vec::new(),
            profile: Profile::default(),
            cols: case.cols,
            rows: case.rows,
            cwd: Some(cwd.to_string_lossy().into_owned()),
            env: Vec::new(),
            wait_ready: Some(false),
            restart: false,
            timeouts: Timeouts::default(),
            recording: AutomaticRecording::default(),
            style: Style::default(),
        };
        session.spawn_with_raw_output_capture(
            launch,
            run_options,
            RawOutputCaptureOptions::new(&raw_path),
        )?;
        spawned = true;

        let sample_profile = resolve_profile(PROFILE_NAME)?;
        ensure_child_active(&session)?;
        if let Journey::First30(flow) = case.journey {
            capture_first30_checkpoint(
                &session,
                sample_profile,
                case,
                flow,
                "initial",
                &case_out,
                &mut checkpoints,
                &mut semantic_assertions,
            )?;
        } else {
            session
                .get_by_text(case.initial_marker)
                .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
            wait_visual_quiet(&session)?;
            if !matches!(case.journey, Journey::HollaFirstUseAction) {
                let initial_text = match session.execute(Operation::Text { full: true })? {
                    OperationResult::Text(text) => text,
                    other => return Err(format!("unexpected text result: {other:?}").into()),
                };
                if initial_text.trim().is_empty() {
                    return Err("initial visible marker appeared but screen text is empty".into());
                }
            }
            checkpoints.push(capture_checkpoint(
                &session,
                sample_profile,
                case,
                case.checkpoint_labels[0],
                &case_out,
            )?);
            if matches!(case.journey, Journey::HollaFirstUseAction)
                && snapshot_text(last_checkpoint_snapshot(&checkpoints)?)
                    .trim()
                    .is_empty()
            {
                return Err("initial visible marker appeared but captured screen is empty".into());
            }
        }

        match case.journey {
            Journey::First30(flow) => run_first30_steps(
                &session,
                sample_profile,
                case,
                flow,
                &case_out,
                &mut checkpoints,
                &mut actions,
                &mut semantic_assertions,
            )?,
            Journey::ShowcaseHelp80x24 => run_showcase_help_steps(
                &session,
                sample_profile,
                case,
                &case_out,
                &mut checkpoints,
                &mut actions,
            )?,
            Journey::HollaFirstUseAction => run_holla_first_use_action_steps(
                &session,
                sample_profile,
                case,
                &case_out,
                &mut checkpoints,
                &mut actions,
                &mut semantic_assertions,
            )?,
        }

        actions.push(json!({"event":"key_press","key":case.quit_key,"at_unix_ms":unix_ms()}));
        final_quit_attempted = true;
        send_key(&session, case.quit_key)?;
        session.execute(Operation::WaitExit {
            timeout_ms: Some(EXIT_TIMEOUT_MS),
        })?;
        let state = match session.execute(Operation::State) {
            Ok(OperationResult::State(state)) => state,
            Ok(other) => {
                let reason = format!("unexpected process state result: {other:?}");
                process_state = json!({
                    "status":"unavailable",
                    "api":"Operation::State",
                    "reason":reason,
                });
                return Err(reason.into());
            }
            Err(error) => {
                process_state = json!({
                    "status":"unavailable",
                    "api":"Operation::State",
                    "reason":error.to_string(),
                });
                return Err(error.into());
            }
        };
        child_exited = state.exited.is_some() || state.exit_signal.is_some();
        let restoration = terminal_restoration_observation(&state, case.cols, case.rows);
        process_state = serde_json::to_value(state.as_ref())?;
        if let Some(object) = process_state.as_object_mut() {
            object.insert("wait_exit_completed".to_string(), json!(true));
            object.insert("terminal_restoration".to_string(), restoration.clone());
        }
        if state.exited != Some(0) || state.exit_signal.is_some() {
            exit_validation_errors.push(format!(
                "child did not exit cleanly: exited={:?}, signal={:?}",
                state.exited, state.exit_signal
            ));
        }
        if restoration["status"] != "pass" {
            exit_validation_errors.push(format!(
                "terminal restoration failed or was unavailable: {restoration}"
            ));
        }

        let quiet_started = Instant::now();
        let final_quiet = match wait_visual_quiet(&session) {
            Ok(()) => json!({
                "status":"quiet",
                "interval_ms":QUIET_INTERVAL.as_millis(),
                "elapsed_ms":quiet_started.elapsed().as_millis(),
            }),
            Err(error) => {
                exit_validation_errors.push(format!(
                    "terminal was not visually quiet before the exit receipt: {error}"
                ));
                json!({
                    "status":"not_quiet_within_bound",
                    "interval_ms":QUIET_INTERVAL.as_millis(),
                    "timeout_ms":QUIET_TIMEOUT.as_millis(),
                    "elapsed_ms":quiet_started.elapsed().as_millis(),
                    "error":error.to_string(),
                })
            }
        };
        if let Some(object) = process_state.as_object_mut() {
            object.insert("final_visual_quiet".to_string(), final_quiet);
        }
        if case.exit_receipt_is_separate {
            exit_receipt = Some(capture_checkpoint(
                &session,
                sample_profile,
                case,
                case.exit_receipt_label,
                &case_out,
            )?);
            let receipt = exit_receipt
                .as_ref()
                .ok_or("separate exit receipt was not captured")?;
            let dimensions_match = receipt.snapshot.cols() == u32::from(case.cols)
                && receipt.snapshot.rows() == u32::from(case.rows);
            let dimension_observation = json!({
                "status":if dimensions_match {"pass"} else {"fail"},
                "expected":{"cols":case.cols,"rows":case.rows},
                "observed":{"cols":receipt.snapshot.cols(),"rows":receipt.snapshot.rows()},
            });
            if let Some(restoration) = process_state
                .get_mut("terminal_restoration")
                .and_then(Value::as_object_mut)
            {
                restoration.insert(
                    "exit_snapshot_dimensions".to_string(),
                    dimension_observation,
                );
                if !dimensions_match {
                    restoration.insert("status".to_string(), json!("fail"));
                    if let Some(failed) = restoration
                        .get_mut("failed_fields")
                        .and_then(Value::as_array_mut)
                    {
                        failed.push(json!("exit_snapshot_dimensions"));
                    }
                }
            }
            if !dimensions_match {
                exit_validation_errors.push(format!(
                    "exit receipt snapshot dimensions are {}x{}, expected {}x{}",
                    receipt.snapshot.cols(),
                    receipt.snapshot.rows(),
                    case.cols,
                    case.rows
                ));
            }
        } else {
            let exit_label = case.exit_receipt_label;
            if case.checkpoint_labels.last().copied() != Some(exit_label) {
                return Err(format!("case ends with unexpected checkpoint {exit_label:?}").into());
            }
            checkpoints.push(capture_checkpoint(
                &session,
                sample_profile,
                case,
                exit_label,
                &case_out,
            )?);
        }

        let status = session
            .raw_output_capture_status()
            .ok_or("raw output capture status is unavailable")?;
        if !status.setup_complete || !status.complete || !status.errors.is_empty() {
            return Err(format!("raw capture is incomplete or has errors: {status:?}").into());
        }
        let processed = status
            .processed_output_offset
            .ok_or("processed output byte offset overflowed")?;
        let captured = status
            .captured_output_bytes
            .ok_or("captured output byte count overflowed")?;
        if !status.drained || !status.flushed || captured != processed {
            return Err(format!(
                "raw capture drain mismatch: drained={}, flushed={}, captured={}, processed={}",
                status.drained, status.flushed, captured, processed
            )
            .into());
        }
        let raw = fs::read(&raw_path)?;
        if u64::try_from(raw.len())? != captured {
            return Err(format!(
                "raw file length {} does not match reported byte count {captured}",
                raw.len()
            )
            .into());
        }
        let final_capture = exit_receipt
            .as_ref()
            .or_else(|| checkpoints.last())
            .ok_or("exit checkpoint was not captured")?;
        if final_capture.offset != captured
            || final_capture.offset != processed
            || final_capture.offset != u64::try_from(raw.len())?
        {
            return Err(format!(
                "exit checkpoint offset {} does not match captured {}, processed {}, and raw file length {}",
                final_capture.offset,
                captured,
                processed,
                raw.len()
            )
            .into());
        }
        for checkpoint in &checkpoints {
            if checkpoint.offset > captured {
                return Err(format!(
                    "checkpoint {} offset {} exceeds drained stream {captured}",
                    checkpoint.checkpoint_id, checkpoint.offset
                )
                .into());
            }
            let end = usize::try_from(checkpoint.offset)?;
            let checkpoint_dir = case_out.join("checkpoints").join(&checkpoint.label);
            fs::create_dir_all(&checkpoint_dir)?;
            write_new(&checkpoint_dir.join("raw-prefix.ansi"), &raw[..end])?;
            write_json_new(
                &checkpoint_dir.join("capture.json"),
                &json!({
                    "checkpoint_id": checkpoint.checkpoint_id,
                    "sample_id": checkpoint.sample_id,
                    "processed_output_offset": checkpoint.offset,
                    "raw_prefix_sha256": sha256(&raw[..end]),
                    "raw_stream_sha256": sha256(&raw),
                    "raw_stream_bytes": captured,
                }),
            )?;
        }
        if let Some(receipt) = exit_receipt.as_ref() {
            let directory = case_out.join("checkpoints").join(case.exit_receipt_label);
            let prefix_end = usize::try_from(receipt.offset)?;
            write_new(&directory.join("raw-prefix.ansi"), &raw[..prefix_end])?;
            write_json_new(
                &directory.join("capture.json"),
                &json!({
                    "schema":"termrock-live-exit-receipt.v1",
                    "case_id":case.id,
                    "sample_id":receipt.sample_id,
                    "processed_output_offset":receipt.offset,
                    "raw_prefix_sha256":sha256(&raw[..prefix_end]),
                    "raw_stream_sha256":sha256(&raw),
                    "raw_stream_bytes":captured,
                    "exit_code":process_state["exited"],
                    "exit_signal":process_state["exit_signal"],
                    "capture_complete":status.complete,
                    "drained":status.drained,
                    "flushed":status.flushed,
                    "capture_errors":status.errors,
                    "final_state":process_state.clone(),
                }),
            )?;
        }

        if !exit_validation_errors.is_empty() {
            return Err(format!(
                "exit validation failed: {}",
                exit_validation_errors.join("; ")
            )
            .into());
        }
        Ok(())
    })();

    let mut cleanup_error = None;
    let mut closed_for_cleanup = false;
    if work.is_err() && spawned {
        if matches!(case.journey, Journey::HollaFirstUseAction) && !final_quit_attempted {
            let (finalization, state, exited) = finalize_holla_after_error(&session, &mut actions);
            failure_finalization = finalization;
            process_state = state;
            child_exited = exited;
        }
        if !child_exited {
            closed_for_cleanup = true;
            if let Err(error) = session.close() {
                cleanup_error = Some(error.to_string());
            }
        }
    }
    if let Some(object) = process_state.as_object_mut() {
        object.insert(
            "exit_validation_errors".to_string(),
            json!(exit_validation_errors),
        );
    }
    let mut checkpoint_evidence_errors = Vec::new();
    if let Some(status) = session.raw_output_capture_status() {
        match serde_json::to_value(&status) {
            Ok(mut status_value) => {
                if let Some(object) = status_value.as_object_mut() {
                    object.insert(
                        "raw_file_bytes".to_string(),
                        json!(fs::metadata(&raw_path).ok().map(|metadata| metadata.len())),
                    );
                    object.insert(
                        "exit_checkpoint_offset".to_string(),
                        json!(
                            exit_receipt
                                .as_ref()
                                .map(|receipt| receipt.offset)
                                .or_else(|| {
                                    checkpoints
                                        .last()
                                        .filter(|checkpoint| checkpoint.label == "exited")
                                        .map(|checkpoint| checkpoint.offset)
                                })
                        ),
                    );
                    object.insert(
                        "observed_after_cleanup_close".to_string(),
                        json!(closed_for_cleanup),
                    );
                    object.insert("cleanup_error".to_string(), json!(cleanup_error.clone()));
                }
                if let Err(error) = write_json_new(&case_out.join("raw-status.json"), &status_value)
                {
                    checkpoint_evidence_errors.push(format!("raw status record failed: {error}"));
                }
            }
            Err(error) => {
                checkpoint_evidence_errors.push(format!("raw status serialization failed: {error}"))
            }
        }
    }
    if !process_state.is_null()
        && let Err(error) = write_json_new(&case_out.join("process-state.json"), &process_state)
    {
        checkpoint_evidence_errors.push(format!("process state record failed: {error}"));
    }
    if work.is_err() {
        let errors = preserve_reached_checkpoint_prefixes(
            &raw_path,
            &case_out,
            &checkpoints,
            exit_receipt.as_ref(),
        );
        checkpoint_evidence_errors.extend(errors);
    }
    if !checkpoint_evidence_errors.is_empty() {
        if let Err(error) = write_json_new(
            &case_out.join("checkpoint-evidence-errors.json"),
            &checkpoint_evidence_errors,
        ) {
            checkpoint_evidence_errors.push(format!("evidence error record failed: {error}"));
        }
    }
    if let Err(error) = write_json_new(
        &case_out.join("actions.json"),
        &json!({
            "schema":"termrock-live-actions.v1",
            "started_at_unix_ms":started_at,
            "finished_at_unix_ms":unix_ms(),
            "actions":actions,
        }),
    ) {
        checkpoint_evidence_errors.push(format!("action record failed: {error}"));
    }
    if matches!(
        case.journey,
        Journey::First30(_) | Journey::HollaFirstUseAction
    ) {
        if let Err(error) = write_json_new(
            &case_out.join("semantic-assertions.json"),
            &semantic_assertions,
        ) {
            checkpoint_evidence_errors.push(format!("semantic assertion record failed: {error}"));
        }
    }

    let (process_status, work_error) = match work {
        Ok(()) => ("complete", None),
        Err(error) => ("incomplete", Some(error.to_string())),
    };
    let mut comparison_records = Vec::new();
    for checkpoint in &checkpoints {
        let is_exit_receipt = checkpoint.label == case.exit_receipt_label;
        let (record, write_error) = compare_checkpoint(
            frozen_store,
            suite,
            &case_out,
            case.id,
            checkpoint,
            if is_exit_receipt {
                "exit_receipt"
            } else {
                "live_frame"
            },
        );
        if let Some(error) = write_error {
            checkpoint_evidence_errors.push(error);
        }
        comparison_records.push(record);
    }
    if let Some(receipt) = exit_receipt.as_ref() {
        let (record, write_error) = compare_checkpoint(
            frozen_store,
            suite,
            &case_out,
            case.id,
            receipt,
            "exit_receipt",
        );
        if let Some(error) = write_error {
            checkpoint_evidence_errors.push(error);
        }
        comparison_records.push(record);
    }
    if !checkpoint_evidence_errors.is_empty() {
        if let Err(error) = write_json_new(
            &case_out.join("checkpoint-evidence-errors-after-comparison.json"),
            &checkpoint_evidence_errors,
        ) {
            checkpoint_evidence_errors.push(format!(
                "post-comparison evidence error record failed: {error}"
            ));
        }
    }
    let mut check_failures = 0;
    let mut checkpoints_passed = 0;
    let mut setup_failed_comparisons = 0;
    let mut comparison_unavailable = 0;
    let mut exit_receipt_check_failures = 0;
    let mut exit_receipt_checks_passed = 0;
    let mut exit_receipt_setup_failed = 0;
    let mut exit_receipt_comparison_unavailable = 0;
    for comparison in &comparison_records {
        let is_exit_receipt = comparison["capture_kind"] == "exit_receipt";
        match comparison["status"].as_str() {
            Some("pass") if is_exit_receipt => exit_receipt_checks_passed += 1,
            Some("pass") => checkpoints_passed += 1,
            Some("failure") if is_exit_receipt => exit_receipt_check_failures += 1,
            Some("failure") => check_failures += 1,
            Some("setup_failed") if is_exit_receipt => exit_receipt_setup_failed += 1,
            Some("setup_failed") => setup_failed_comparisons += 1,
            Some("unavailable") if is_exit_receipt => exit_receipt_comparison_unavailable += 1,
            Some("unavailable") => comparison_unavailable += 1,
            _ => {
                if is_exit_receipt {
                    exit_receipt_comparison_unavailable += 1;
                } else {
                    comparison_unavailable += 1;
                }
            }
        }
    }
    let semantic_failures = semantic_assertions
        .iter()
        .filter(|assertion| assertion["status"] != "pass")
        .count();
    let setup_failed_comparisons_total = setup_failed_comparisons + exit_receipt_setup_failed;
    let comparison_unavailable_total = comparison_unavailable + exit_receipt_comparison_unavailable;
    let application_or_visual_failure = process_status != "complete"
        || check_failures > 0
        || exit_receipt_check_failures > 0
        || semantic_failures > 0;
    let setup_failure = setup_failed_comparisons_total > 0 || comparison_unavailable_total > 0;
    let evidence_failure = !checkpoint_evidence_errors.is_empty();
    let final_status = if application_or_visual_failure || evidence_failure {
        "failed"
    } else if setup_failure {
        "setup_failed"
    } else {
        "passed"
    };
    let error = if let Some(error) = work_error {
        json!(error)
    } else if application_or_visual_failure {
        json!(format!(
            "live frame comparison failures={check_failures}; exit receipt comparison failures={exit_receipt_check_failures}; semantic checkpoint failures={semantic_failures}"
        ))
    } else if setup_failure {
        json!(format!(
            "missing baseline comparisons={setup_failed_comparisons_total}; unavailable comparisons={comparison_unavailable_total}"
        ))
    } else if evidence_failure {
        json!(format!(
            "{} checkpoint evidence records could not be saved",
            checkpoint_evidence_errors.len()
        ))
    } else {
        Value::Null
    };
    let first30 = matches!(case.journey, Journey::First30(_));
    let live_frame_count = if first30 {
        checkpoints.len()
    } else {
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.label != case.exit_receipt_label)
            .count()
    };
    let reached_checkpoint_labels: Vec<&str> = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.label.as_str())
        .collect();
    let unreached_checkpoint_labels: Vec<&str> = case
        .checkpoint_labels
        .iter()
        .copied()
        .filter(|label| !reached_checkpoint_labels.contains(label))
        .collect();
    let primary_lane_exclusion_entries = toml_field(case_inventory, "not_run_requirement")?
        .as_array()
        .ok_or("case inventory not_run_requirement entries must be an array")?;
    let primary_lane_excluded_requirements = serde_json::to_value(primary_lane_exclusion_entries)?;
    let not_run_requirements = if first30 {
        serde_json::to_value(
            primary_lane_exclusion_entries
                .iter()
                .filter(|requirement| {
                    requirement.get("id").and_then(toml::Value::as_str) != Some(case.id)
                })
                .cloned()
                .collect::<Vec<_>>(),
        )?
    } else {
        json!([])
    };
    let mut failed_checkpoint_labels: Vec<String> = semantic_assertions
        .iter()
        .filter(|assertion| assertion["status"] != "pass")
        .filter_map(|assertion| {
            assertion["id"]
                .as_str()
                .and_then(|id| id.rsplit('/').next())
                .map(str::to_owned)
        })
        .collect();
    let mut setup_failed_checkpoint_labels = Vec::new();
    let mut comparison_unavailable_checkpoint_labels = Vec::new();
    for comparison in &comparison_records {
        if comparison["capture_kind"] != "live_frame" {
            continue;
        }
        if let Some(label) = comparison["label"].as_str() {
            match comparison["status"].as_str() {
                Some("failure") => failed_checkpoint_labels.push(label.to_owned()),
                Some("setup_failed") => setup_failed_checkpoint_labels.push(label.to_owned()),
                Some("unavailable") => {
                    comparison_unavailable_checkpoint_labels.push(label.to_owned());
                }
                _ => {}
            }
        }
    }
    failed_checkpoint_labels.sort();
    failed_checkpoint_labels.dedup();
    setup_failed_checkpoint_labels.sort();
    setup_failed_checkpoint_labels.dedup();
    comparison_unavailable_checkpoint_labels.sort();
    comparison_unavailable_checkpoint_labels.dedup();
    let exit_receipt_comparison = exit_receipt.as_ref().and_then(|receipt| {
        comparison_records
            .iter()
            .find(|record| {
                record["capture_kind"] == "exit_receipt"
                    && record["sample_id"].as_str() == Some(receipt.sample_id.as_str())
            })
            .cloned()
    });
    let live_frame_comparison_count = comparison_records
        .iter()
        .filter(|record| record["capture_kind"] == "live_frame")
        .count();
    let exit_capture_comparison_count = comparison_records
        .iter()
        .filter(|record| record["capture_kind"] == "exit_receipt")
        .count();
    let mut result = json!({
        "schema":"termrock-live-flow.v1",
        "id":case.id,
        "test_name":case.test_name,
        "case_class":case.inventory_class,
        "terminal_profile":case.terminal_profile,
        "execution":execution,
        "binary":case.binary,
        "initial_marker":case.initial_marker,
        "status":final_status,
        "process_status":process_status,
        "error":error,
        "cleanup_error":cleanup_error,
        "started_at_unix_ms":started_at,
        "finished_at_unix_ms":unix_ms(),
        "live_process_elapsed_ms":case_start.elapsed().as_millis() as u64,
        "expected_checkpoints":case.checkpoint_labels,
        "reached_checkpoint_labels":reached_checkpoint_labels,
        "failed_checkpoint_labels":failed_checkpoint_labels,
        "setup_failed_checkpoint_labels":setup_failed_checkpoint_labels,
        "comparison_unavailable_checkpoint_labels":comparison_unavailable_checkpoint_labels,
        "unreached_checkpoint_labels":unreached_checkpoint_labels,
        "not_run_requirements":not_run_requirements,
        "checkpoints_completed":checkpoints.len(),
        "checkpoints_passed":checkpoints_passed,
        "check_failures":check_failures,
        "setup_failed_comparisons":setup_failed_comparisons,
        "comparison_unavailable":comparison_unavailable,
        "checkpoint_records":checkpoints.iter().map(|checkpoint| json!({
            "id":checkpoint.checkpoint_id,
            "sample_id":checkpoint.sample_id,
            "label":checkpoint.label,
            "captured_at_unix_ms":checkpoint.captured_at_unix_ms,
            "processed_output_offset":checkpoint.offset,
            "sample_sha256":sha256(&checkpoint.sample_bytes),
            "png_sha256":checkpoint.png_sha256,
            "rgba_sha256":checkpoint.rgba_sha256,
            "sample_path":format!("{}/checkpoints/{}/sample.tui.json", case.id, checkpoint.label),
            "expected_path":format!("expected/{}.tui.json", checkpoint.sample_id),
            "comparison":comparison_records.iter().find(|record| {
                record["checkpoint_id"].as_str() == Some(checkpoint.checkpoint_id.as_str())
            }),
        })).collect::<Vec<_>>(),
        "checkpoint_count":checkpoints.len(),
        "live_frame_count":live_frame_count,
        "live_frame_comparison_count":live_frame_comparison_count,
        "exit_receipt_count":u8::from(exit_receipt.is_some()),
        "exit_receipt_comparison_count":exit_capture_comparison_count,
        "exit_receipt_checks_passed":exit_receipt_checks_passed,
        "exit_receipt_check_failures":exit_receipt_check_failures,
        "exit_receipt_setup_failed_comparisons":exit_receipt_setup_failed,
        "exit_receipt_comparison_unavailable":exit_receipt_comparison_unavailable,
        "exit_receipt":exit_receipt.as_ref().map(|receipt| json!({
            "schema":"termrock-live-exit-receipt.v1",
            "sample_id":receipt.sample_id,
            "processed_output_offset":receipt.offset,
            "captured_at_unix_ms":receipt.captured_at_unix_ms,
            "sample_sha256":sha256(&receipt.sample_bytes),
            "png_sha256":receipt.png_sha256,
            "rgba_sha256":receipt.rgba_sha256,
            "sample_path":format!("{}/checkpoints/{}/sample.tui.json", case.id, case.exit_receipt_label),
            "raw_prefix_path":format!("{}/checkpoints/{}/raw-prefix.ansi", case.id, case.exit_receipt_label),
            "terminal_restoration":process_state["terminal_restoration"],
            "comparison":exit_receipt_comparison,
            "visual_denominator_member":false,
        })),
        "comparison_records":comparison_records,
        "process_state":process_state,
        "checkpoint_evidence_errors":checkpoint_evidence_errors,
        "semantic_failures":semantic_failures,
        "reference_approval":"not performed",
    });
    if matches!(
        case.journey,
        Journey::First30(_) | Journey::HollaFirstUseAction
    ) && let Some(object) = result.as_object_mut()
    {
        object.insert(
            "semantic_assertions".to_string(),
            json!(semantic_assertions),
        );
        object.insert("failure_finalization".to_string(), failure_finalization);
    }
    if matches!(case.journey, Journey::First30(_))
        && let Some(object) = result.as_object_mut()
    {
        object.insert(
            "protocol".to_string(),
            json!({
                "version":"first30-v1",
                "selected_primary_case_count":FIRST30_PRIMARY_CASE_DENOMINATOR,
                "selected_primary_live_frame_count":FIRST30_FRAME_DENOMINATOR,
                "selected_primary_exit_receipt_count":FIRST30_PRIMARY_CASE_DENOMINATOR,
                "additional_holla_frame_one_variant":{
                    "case_id":"first30-v1/holla.first_use.frame1",
                    "live_frame_count":FIRST30_HOLLA_CHECKPOINTS.len(),
                    "exit_receipt_count":1,
                    "included_in_primary_denominator":false,
                },
                "primary_lane_excluded_requirements":primary_lane_excluded_requirements,
            }),
        );
    }
    write_json_new(&case_out.join("result.json"), &result)?;
    Ok(result)
}

fn run_first30_steps(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    flow: First30Flow,
    case_out: &Path,
    checkpoints: &mut Vec<Checkpoint>,
    actions: &mut Vec<Value>,
    semantic_assertions: &mut Vec<Value>,
) -> AppResult<()> {
    let steps = match flow {
        First30Flow::ShowcaseButtons => FIRST30_SHOWCASE_STEPS,
        First30Flow::HollaFirstUse => FIRST30_HOLLA_STEPS,
        First30Flow::JackinSettings => FIRST30_JACKIN_STEPS,
        First30Flow::TableProConnections => FIRST30_TABLEPRO_STEPS,
    };
    for step in steps {
        match *step {
            First30Step::Key(key) => record_first30_key(session, actions, key)?,
            First30Step::Text(text) => {
                actions.push(json!({
                    "event":"text_input",
                    "text":text,
                    "at_unix_ms":unix_ms(),
                }));
                session.execute(Operation::Write {
                    data: text.to_owned(),
                })?;
            }
            First30Step::MouseDown { x, y } => record_mouse_action(
                session,
                actions,
                MouseAction::Down {
                    x,
                    y,
                    options: MouseOptions::new(MouseButton::Left),
                },
                "mouse_down",
                x,
                y,
            )?,
            First30Step::MouseUp { x, y } => record_mouse_action(
                session,
                actions,
                MouseAction::Up {
                    x,
                    y,
                    options: MouseOptions::new(MouseButton::Left),
                },
                "mouse_up",
                x,
                y,
            )?,
            First30Step::Resize { cols, rows } => {
                record_first30_resize(session, actions, cols, rows)?
            }
            First30Step::Checkpoint(label) => capture_first30_checkpoint(
                session,
                profile,
                case,
                flow,
                label,
                case_out,
                checkpoints,
                semantic_assertions,
            )?,
        }
    }
    Ok(())
}

fn capture_first30_checkpoint(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    flow: First30Flow,
    label: &str,
    case_out: &Path,
    checkpoints: &mut Vec<Checkpoint>,
    semantic_assertions: &mut Vec<Value>,
) -> AppResult<()> {
    let ready_marker = first30_ready_marker(flow, label);
    let readiness_wait = wait_first30_marker(session, ready_marker);
    let quiet_started = Instant::now();
    let quiet_result = match wait_visual_quiet(session) {
        Ok(()) => json!({
            "status":"quiet",
            "interval_ms":QUIET_INTERVAL.as_millis(),
            "elapsed_ms":quiet_started.elapsed().as_millis(),
        }),
        Err(error) => json!({
            "status":"not_quiet_within_bound",
            "interval_ms":QUIET_INTERVAL.as_millis(),
            "timeout_ms":QUIET_TIMEOUT.as_millis(),
            "elapsed_ms":quiet_started.elapsed().as_millis(),
            "error":error.to_string(),
        }),
    };
    let checkpoint = capture_checkpoint(session, profile, case, label, case_out)?;
    let initial = checkpoints
        .iter()
        .find(|reached| reached.label == "initial")
        .map(|reached| &reached.snapshot);
    let child_state = observe_child_state(session);
    let child_active = child_state["child_active"] == true;
    semantic_assertions.push(first30_assertion(
        case.id,
        flow,
        label,
        &checkpoint.snapshot,
        checkpoint.offset,
        initial,
        readiness_wait,
        quiet_result,
        child_active,
        child_state,
    ));
    let assertion_passed = semantic_assertions
        .last()
        .is_some_and(|assertion| assertion["status"] == "pass");
    checkpoints.push(checkpoint);
    if !child_active {
        return Err(format!("child was not active at first30 checkpoint {label}").into());
    }
    if !assertion_passed {
        return Err(format!(
            "first30 checkpoint {label} did not prove its visible readiness and input postconditions"
        )
        .into());
    }
    Ok(())
}

fn record_first30_key(session: &Session, actions: &mut Vec<Value>, key: &str) -> AppResult<()> {
    actions.push(json!({"event":"key_press","key":key,"at_unix_ms":unix_ms()}));
    send_key(session, key)
}

fn record_first30_resize(
    session: &Session,
    actions: &mut Vec<Value>,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    actions.push(json!({
        "event":"resize",
        "cols":cols,
        "rows":rows,
        "at_unix_ms":unix_ms(),
    }));
    session.execute(Operation::Resize { cols, rows })?;
    Ok(())
}

fn record_mouse_action(
    session: &Session,
    actions: &mut Vec<Value>,
    action: MouseAction,
    event: &str,
    x: u16,
    y: u16,
) -> AppResult<()> {
    actions.push(json!({
        "event":event,
        "button":"left",
        "x":x,
        "y":y,
        "at_unix_ms":unix_ms(),
    }));
    session.execute(Operation::Mouse { action })?;
    Ok(())
}

fn first30_ready_marker(flow: First30Flow, label: &str) -> &'static str {
    match (flow, label) {
        (First30Flow::ShowcaseButtons, "initial") => "Buttons",
        (First30Flow::ShowcaseButtons, "run") => "last: Run task",
        (First30Flow::ShowcaseButtons, "preview") => "last: Preview",
        (First30Flow::ShowcaseButtons, "help") => "Keyboard & mouse",
        (First30Flow::ShowcaseButtons, "returned") => "Preview ✓ · 2 activations",
        (First30Flow::ShowcaseButtons, "small") => "Buttons",
        (First30Flow::ShowcaseButtons, "restored") => "Preview ✓ · 2 activations",
        (First30Flow::HollaFirstUse, "initial") => "Search actions and resources…",
        (First30Flow::HollaFirstUse, "empty") => "No matches for “zzzz-no-such-action”",
        (First30Flow::HollaFirstUse, "cleared") => "Search actions and resources…",
        (First30Flow::HollaFirstUse, "help") => "Key reference",
        (First30Flow::HollaFirstUse, "returned") => "Search actions and resources…",
        (First30Flow::HollaFirstUse, "small") => "holla❯",
        (First30Flow::HollaFirstUse, "restored") => "Show system resources",
        (First30Flow::JackinSettings, "initial") => "Current directory",
        (First30Flow::JackinSettings, "selected") => "payments-platform",
        (First30Flow::JackinSettings, "general") => "Settings › global › General",
        (First30Flow::JackinSettings, "mounts") => "Settings › global › Mounts",
        (First30Flow::JackinSettings, "general_return") => "Settings › global › General",
        (First30Flow::JackinSettings, "manager_return") => "Current directory",
        (First30Flow::JackinSettings, "help") => "Keyboard shortcuts",
        (First30Flow::JackinSettings, "help_return") => "Current directory",
        (First30Flow::JackinSettings, "small") => "Current directory",
        (First30Flow::JackinSettings, "restored") => "Current directory",
        (First30Flow::TableProConnections, "initial") => "Local PostgreSQL",
        (First30Flow::TableProConnections, "selected") => "Local PostgreSQL",
        (First30Flow::TableProConnections, "help") => "Keyboard shortcuts",
        (First30Flow::TableProConnections, "returned") => "Local PostgreSQL",
        (First30Flow::TableProConnections, "small") => "Connections",
        (First30Flow::TableProConnections, "restored") => "Local PostgreSQL",
        _ => "",
    }
}

fn wait_first30_marker(session: &Session, marker: &str) -> Value {
    let started = Instant::now();
    match session
        .get_by_text(marker)
        .wait_with_timeout(Some(FIRST30_READY_TIMEOUT_MS))
    {
        Ok(_) => json!({
            "status":"visible",
            "marker":marker,
            "timeout_ms":FIRST30_READY_TIMEOUT_MS,
            "elapsed_ms":started.elapsed().as_millis(),
        }),
        Err(error) => json!({
            "status":"not_visible_within_bound",
            "marker":marker,
            "timeout_ms":FIRST30_READY_TIMEOUT_MS,
            "elapsed_ms":started.elapsed().as_millis(),
            "error":error.to_string(),
        }),
    }
}

fn first30_assertion(
    case_id: &str,
    flow: First30Flow,
    label: &str,
    snapshot: &TerminalSnapshot,
    processed_output_offset: u64,
    initial: Option<&TerminalSnapshot>,
    readiness_wait: Value,
    quiet_result: Value,
    child_active: bool,
    child_state: Value,
) -> Value {
    let text = snapshot_text(snapshot);
    let expected_dimensions = if label == "small" {
        (80, 24)
    } else {
        (DEFAULT_COLS, DEFAULT_ROWS)
    };
    let dimensions_match = snapshot.cols() == u32::from(expected_dimensions.0)
        && snapshot.rows() == u32::from(expected_dimensions.1);
    let ready_marker = first30_ready_marker(flow, label);
    let marker_visible_after_quiet = text.contains(ready_marker);
    let readiness_pass = readiness_wait["status"] == "visible" && marker_visible_after_quiet;
    let quiet_pass = quiet_result["status"] == "quiet";
    let mut required: Vec<&str> = Vec::new();
    let mut forbidden: Vec<&str> = Vec::new();
    let mut condition_failures: Vec<&str> = Vec::new();
    let mut details = json!({});

    match flow {
        First30Flow::ShowcaseButtons => match label {
            "initial" => {
                required.extend(["Buttons", "Run task", "Preview", "Playground"]);
                forbidden.push("Keyboard & mouse");
            }
            "small" => {
                required.push("Buttons");
                forbidden.push("Keyboard & mouse");
            }
            "run" => required.push("last: Run task ✓ · 1 activations"),
            "preview" => required.push("last: Preview ✓ · 2 activations"),
            "help" => required.extend(["Keyboard & mouse", "Shell", "Mouse"]),
            "returned" => {
                required.push("Preview ✓ · 2 activations");
                forbidden.push("Keyboard & mouse");
            }
            "restored" => {
                required.extend(["Buttons", "Preview ✓ · 2 activations"]);
                forbidden.push("Keyboard & mouse");
            }
            _ => {}
        },
        First30Flow::HollaFirstUse => {
            let result_list = holla_result_list_text(snapshot);
            let query_line = text.lines().nth(4).unwrap_or_default();
            details = json!({
                "result_list_text":result_list,
                "query_line":query_line,
            });
            match label {
                "initial" => {
                    required.extend(["holla❯", "Search actions and resources…"]);
                    if !result_list.contains("Show system resources") {
                        condition_failures.push("initial action rows missing from the result list");
                    }
                }
                "empty" => {
                    let title = "No matches for “zzzz-no-such-action”";
                    let query_visible = query_line.contains("zzzz-no-such-action");
                    let title_in_region = result_list.contains(title);
                    let action_row_visible = result_list.contains("›");
                    if !query_visible {
                        condition_failures.push("query missing from the query line");
                    }
                    if !title_in_region {
                        condition_failures.push("No matches title missing from the result list");
                    }
                    if action_row_visible {
                        condition_failures.push("action result row present in the result list");
                    }
                }
                "cleared" | "returned" | "restored" => {
                    required.extend(["holla❯", "Search actions and resources…"]);
                    if !result_list.contains("Show system resources") {
                        condition_failures.push("initial action rows missing from the result list");
                    }
                    forbidden.push("No matches for “zzzz-no-such-action”");
                    forbidden.push("Key reference");
                }
                "help" => {
                    required.push("Key reference");
                    forbidden.push("No matches for “zzzz-no-such-action”");
                }
                "small" => {
                    required.extend(["holla❯", "Search actions and resources…"]);
                    forbidden.extend(["Key reference", "No matches for “zzzz-no-such-action”"]);
                }
                _ => {}
            }
        }
        First30Flow::JackinSettings => match label {
            "initial" | "small" => {
                required.push("Current directory");
                forbidden.extend(["Settings › global ›", "Keyboard shortcuts"]);
            }
            "help_return" => {
                required.push("Current directory");
                required.push("payments-platform");
                if !visible_text_in_columns(
                    snapshot,
                    "payments-platform",
                    snapshot.cols() / 3,
                    snapshot.cols(),
                ) {
                    condition_failures.push("manager detail heading missing after help");
                }
                forbidden.extend(["Settings › global ›", "Keyboard shortcuts"]);
            }
            "selected" => {
                required.push("payments-platform");
                if !visible_text_in_columns(
                    snapshot,
                    "payments-platform",
                    snapshot.cols() / 3,
                    snapshot.cols(),
                ) {
                    condition_failures.push("manager detail heading missing from the detail pane");
                }
                let transition = row_style_transition(
                    initial,
                    Some(snapshot),
                    "Current directory",
                    "payments-platform",
                    snapshot.cols() / 3,
                );
                if transition["status"] != "pass" {
                    condition_failures.push("manager tree selection style did not change");
                }
                details["tree_selection"] = transition;
            }
            "general" | "general_return" => {
                required.extend(["Settings › global › General", "Commits"]);
                forbidden.push("Settings › global › Mounts");
            }
            "mounts" => {
                required.extend([
                    "Settings › global › Mounts",
                    "Destination",
                    "Scope",
                    "Mode",
                    "Isolation",
                    "Kind",
                    "Source",
                    "+ Add mount",
                ]);
                forbidden.push("Commits");
            }
            "manager_return" | "restored" => {
                required.push("Current directory");
                required.push("payments-platform");
                if !visible_text_in_columns(
                    snapshot,
                    "payments-platform",
                    snapshot.cols() / 3,
                    snapshot.cols(),
                ) {
                    condition_failures.push("manager detail heading missing from the detail pane");
                }
                forbidden.extend(["Settings › global ›", "Keyboard shortcuts"]);
            }
            "help" => required.extend([
                "Keyboard shortcuts",
                "Shell",
                "Settings",
                "Leave this screen",
            ]),
            _ => {}
        },
        First30Flow::TableProConnections => match label {
            "initial" => {
                required.extend([
                    "TablePro",
                    "Connections",
                    "Personal",
                    "Local PostgreSQL",
                    "localhost:5432",
                    "acme_dev",
                    "postgres",
                ]);
            }
            "selected" => {
                required.extend(["Personal", "Local PostgreSQL"]);
                let transition = row_style_transition(
                    initial,
                    Some(snapshot),
                    "Personal",
                    "Local PostgreSQL",
                    snapshot.cols() / 3,
                );
                if transition["status"] != "pass" {
                    condition_failures.push("connection tree selection style did not change");
                }
                details["tree_selection"] = transition;
            }
            "help" => required.extend(["Keyboard shortcuts", "Grid and editor"]),
            "returned" | "restored" => {
                required.extend([
                    "Connections",
                    "Personal",
                    "Local PostgreSQL",
                    "localhost:5432",
                    "acme_dev",
                    "postgres",
                ]);
                forbidden.push("Keyboard shortcuts");
            }
            "small" => {
                required.push("Connections");
                forbidden.push("Keyboard shortcuts");
            }
            _ => {}
        },
    }

    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|marker| !text.contains(marker))
        .collect();
    let matched: Vec<&str> = required
        .iter()
        .copied()
        .filter(|marker| text.contains(marker))
        .collect();
    let forbidden_visible: Vec<&str> = forbidden
        .iter()
        .copied()
        .filter(|marker| text.contains(marker))
        .collect();
    let status = if missing.is_empty()
        && forbidden_visible.is_empty()
        && condition_failures.is_empty()
        && dimensions_match
        && readiness_pass
        && quiet_pass
        && child_active
    {
        "pass"
    } else {
        "fail"
    };
    json!({
        "id":format!("{case_id}/{label}"),
        "status":status,
        "check_outcome":if status == "pass" {"CheckOutcomePass"} else {"CheckOutcomeFailure"},
        "required_visible":required,
        "matched_visible":matched,
        "missing_visible":missing,
        "forbidden_visible":forbidden,
        "forbidden_found":forbidden_visible,
        "condition_failures":condition_failures,
        "expected_dimensions":{"cols":expected_dimensions.0,"rows":expected_dimensions.1},
        "observed_dimensions":{"cols":snapshot.cols(),"rows":snapshot.rows()},
        "processed_output_offset":processed_output_offset,
        "dimensions_match":dimensions_match,
        "readiness":{
            "wait":readiness_wait,
            "marker_visible_in_captured_snapshot":marker_visible_after_quiet,
            "quiet_interval_ms":QUIET_INTERVAL.as_millis(),
            "visible_timeout_ms":FIRST30_READY_TIMEOUT_MS,
        },
        "visual_quiet":quiet_result,
        "child_active":child_active,
        "child_state":child_state,
        "observed_visible_text":text,
        "details":details,
    })
}

fn observe_child_state(session: &Session) -> Value {
    match session.execute(Operation::State) {
        Ok(OperationResult::State(state)) => {
            let child_active = state.exited.is_none() && state.exit_signal.is_none();
            json!({
                "status":"available",
                "exited":state.exited,
                "exit_signal":state.exit_signal,
                "cols":state.cols,
                "rows":state.rows,
                "child_active":child_active,
            })
        }
        Ok(other) => json!({
            "status":"unavailable",
            "error":format!("unexpected process state result: {other:?}"),
            "child_active":false,
        }),
        Err(error) => json!({
            "status":"unavailable",
            "error":error.to_string(),
            "child_active":false,
        }),
    }
}

fn terminal_restoration_observation(
    state: &SessionState,
    expected_cols: u16,
    expected_rows: u16,
) -> Value {
    let mut checks = Vec::new();
    let mut failed = Vec::new();
    let mut unavailable = Vec::new();

    for (name, expected) in [
        ("alternate_screen", false),
        ("cursor_visible", true),
        ("bracketed_paste", false),
    ] {
        match state.modes.get(name).copied() {
            Some(observed) => {
                let status = if observed == expected { "pass" } else { "fail" };
                if status == "fail" {
                    failed.push(name);
                }
                checks.push(json!({
                    "field":format!("modes.{name}"),
                    "expected":expected,
                    "observed":observed,
                    "status":status,
                }));
            }
            None => {
                unavailable.push(name);
                checks.push(json!({
                    "field":format!("modes.{name}"),
                    "expected":expected,
                    "observed":null,
                    "status":"unavailable",
                    "reason":"the direct State mode map does not contain this required key",
                }));
            }
        }
    }

    let cursor_visible_status = if state.cursor.visible { "pass" } else { "fail" };
    if cursor_visible_status == "fail" {
        failed.push("cursor.visible");
    }
    checks.push(json!({
        "field":"cursor.visible",
        "expected":true,
        "observed":state.cursor.visible,
        "status":cursor_visible_status,
    }));

    let mouse_status = if state.mouse_mode == "none" {
        "pass"
    } else {
        "fail"
    };
    if mouse_status == "fail" {
        failed.push("mouse_mode");
    }
    checks.push(json!({
        "field":"mouse_mode",
        "expected":"none",
        "observed":state.mouse_mode,
        "status":mouse_status,
    }));

    for (name, observed, expected) in [
        ("cols", state.cols, expected_cols),
        ("rows", state.rows, expected_rows),
    ] {
        let status = if observed == expected { "pass" } else { "fail" };
        if status == "fail" {
            failed.push(name);
        }
        checks.push(json!({
            "field":name,
            "expected":expected,
            "observed":observed,
            "status":status,
        }));
    }

    let status = if !unavailable.is_empty() {
        "unavailable"
    } else if !failed.is_empty() {
        "fail"
    } else {
        "pass"
    };
    json!({
        "status":status,
        "checks":checks,
        "failed_fields":failed,
        "unavailable_fields":unavailable,
    })
}

fn visible_text_in_columns(
    snapshot: &TerminalSnapshot,
    marker: &str,
    x_start: u32,
    x_end: u32,
) -> bool {
    let end = x_end.min(snapshot.cols());
    (0..snapshot.rows()).any(|row_index| {
        let Some(row) = snapshot.row(row_index) else {
            return false;
        };
        row.iter()
            .skip(x_start.min(end) as usize)
            .take(end.saturating_sub(x_start) as usize)
            .filter(|cell| !cell.has(Attrs::INVISIBLE))
            .map(|cell| cell.text())
            .collect::<String>()
            .contains(marker)
    })
}

fn row_style_transition(
    before: Option<&TerminalSnapshot>,
    after: Option<&TerminalSnapshot>,
    first_marker: &str,
    second_marker: &str,
    tree_end: u32,
) -> Value {
    let first_before =
        before.and_then(|snapshot| row_style_for_marker(snapshot, first_marker, tree_end));
    let second_before =
        before.and_then(|snapshot| row_style_for_marker(snapshot, second_marker, tree_end));
    let first_after =
        after.and_then(|snapshot| row_style_for_marker(snapshot, first_marker, tree_end));
    let second_after =
        after.and_then(|snapshot| row_style_for_marker(snapshot, second_marker, tree_end));
    let first_changed = first_before
        .as_ref()
        .zip(first_after.as_ref())
        .is_some_and(|(before, after)| before != after);
    let second_changed = second_before
        .as_ref()
        .zip(second_after.as_ref())
        .is_some_and(|(before, after)| before != after);
    let status = if first_changed && second_changed {
        "pass"
    } else {
        "fail"
    };
    json!({
        "status":status,
        "first_row_marker":first_marker,
        "first_row_before":first_before,
        "first_row_after":first_after,
        "first_row_changed":first_changed,
        "second_row_marker":second_marker,
        "second_row_before":second_before,
        "second_row_after":second_after,
        "second_row_changed":second_changed,
        "tree_column_end":tree_end,
    })
}

fn row_style_for_marker(snapshot: &TerminalSnapshot, marker: &str, tree_end: u32) -> Option<Value> {
    let end = tree_end.min(snapshot.cols()) as usize;
    for row_index in 0..snapshot.rows() {
        let row = snapshot.row(row_index)?;
        let row_text = row
            .iter()
            .take(end)
            .filter(|cell| !cell.has(Attrs::INVISIBLE))
            .map(|cell| cell.text())
            .collect::<String>();
        if row_text.contains(marker) {
            let signature = row
                .iter()
                .take(end)
                .map(|cell| {
                    format!(
                        "{:?}:{:?}:{}:{}:{}:{}:{}:{}",
                        cell.fg(),
                        cell.bg(),
                        cell.has(Attrs::BOLD),
                        cell.has(Attrs::DIM),
                        cell.has(Attrs::INVERSE),
                        cell.has(Attrs::ITALIC),
                        cell.has(Attrs::STRIKE),
                        cell.underline(),
                    )
                })
                .collect::<Vec<_>>()
                .join("|");
            return Some(json!({"row":row_index,"style":signature}));
        }
    }
    None
}

fn run_showcase_help_steps(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    case_out: &Path,
    checkpoints: &mut Vec<Checkpoint>,
    actions: &mut Vec<Value>,
) -> AppResult<()> {
    actions.push(json!({"event":"key_press","key":case.help_key,"at_unix_ms":unix_ms()}));
    send_key(session, case.help_key)?;
    session
        .get_by_text(case.help_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[1],
        case_out,
    )?);
    let help_start_index = checkpoints.len() - 1;
    session
        .get_by_text("Mouse")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;

    actions.push(json!({"event":"key_press","key":"PageDown","at_unix_ms":unix_ms()}));
    send_key(session, "PageDown")?;
    session
        .get_by_text("Mouse")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.help_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    let help_scrolled =
        capture_checkpoint(session, profile, case, case.checkpoint_labels[2], case_out)?;
    let scrolled_text_changed = visible_text_changed(
        &checkpoints[help_start_index].snapshot,
        &help_scrolled.snapshot,
    );
    checkpoints.push(help_scrolled);
    if !scrolled_text_changed {
        return Err("PageDown did not change visible help text or range".into());
    }

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text(case.help_marker)
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[3],
        case_out,
    )?);

    actions.push(json!({"event":"key_press","key":case.help_key,"at_unix_ms":unix_ms()}));
    send_key(session, case.help_key)?;
    session
        .get_by_text(case.help_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    let help_reopened =
        capture_checkpoint(session, profile, case, case.checkpoint_labels[4], case_out)?;
    let help_reopened_index = checkpoints.len();
    checkpoints.push(help_reopened);
    assert_same_checkpoint_state(
        &checkpoints[help_start_index],
        &checkpoints[help_reopened_index],
        "reopened help did not return to the initial help state",
    )?;

    actions.push(json!({"event":"key_press","key":"q","at_unix_ms":unix_ms()}));
    send_key(session, "q")?;
    session
        .get_by_text(case.help_marker)
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("Quit the showcase?")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    ensure_child_active(session)?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[5],
        case_out,
    )?);
    Ok(())
}

fn run_holla_first_use_action_steps(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    case_out: &Path,
    checkpoints: &mut Vec<Checkpoint>,
    actions: &mut Vec<Value>,
    semantic_assertions: &mut Vec<Value>,
) -> AppResult<()> {
    let raw_path = case_out.join("raw.ansi");
    let initial_assertion = holla_home_assertion(
        "initial",
        last_checkpoint_snapshot(checkpoints)?,
        "empty",
        true,
        false,
    );
    semantic_assertions.push(initial_assertion.clone());

    actions.push(json!({"event":"key_press","key":"Enter","at_unix_ms":unix_ms()}));
    send_key(session, "Enter")?;
    if let Err(wait_error) = session
        .get_by_text("System resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))
    {
        let quiet_error = wait_visual_quiet(session)
            .err()
            .map(|error| error.to_string());
        match capture_failure_evidence(
            session,
            profile,
            case,
            case_out,
            &raw_path,
            "initial-resource",
        ) {
            Ok(evidence) => {
                let actual_text = snapshot_text(&evidence.snapshot);
                let mut assertion =
                    holla_resource_page_assertion("initial-resource", &evidence.snapshot);
                let expected_page_visible = assertion_passed(&assertion);
                let screen_quiet = quiet_error.is_none();
                let wrong_destination = evidence.child_active
                    && evidence.screen_observable
                    && screen_quiet
                    && actual_text.contains("System snapshot")
                    && !actual_text.contains("System resources");
                if let Some(object) = assertion.as_object_mut() {
                    object.insert(
                        "locator_wait_error".to_string(),
                        json!(wait_error.to_string()),
                    );
                    object.insert("visual_quiet_error".to_string(), json!(quiet_error));
                    object.insert("failure_evidence".to_string(), evidence.record.clone());
                    object.insert(
                        "failure_state_observation".to_string(),
                        evidence.state_observation.clone(),
                    );
                    object.insert(
                        "screen_observable".to_string(),
                        json!(evidence.screen_observable),
                    );
                    object.insert("screen_quiet".to_string(), json!(screen_quiet));
                    object.insert(
                        "wrong_destination_confirmed".to_string(),
                        json!(wrong_destination),
                    );
                    if !evidence.child_active || !evidence.screen_observable {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("active screen or live child could not be verified"),
                        );
                    } else if !screen_quiet {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("captured screen did not reach visual quiet"),
                        );
                    } else if expected_page_visible {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("locator wait timed out although the captured snapshot showed the expected page"),
                        );
                    }
                }
                semantic_assertions.push(assertion.clone());
                let evidence_path = evidence.record["directory"]
                    .as_str()
                    .unwrap_or("failure-evidence");
                if wrong_destination {
                    return Err(format!(
                        "initial-resource opened the visible System snapshot dialog instead of the System resources page; evidence: {evidence_path}"
                    )
                    .into());
                }
                return Err(format!(
                    "initial-resource did not show the System resources page before timeout; wrong destination was not confirmed; evidence: {evidence_path}"
                )
                .into());
            }
            Err(capture_error) => {
                semantic_assertions.push(json!({
                    "id":"initial-resource",
                    "status":"unverified",
                    "expected_visible":["System resources","12.4 of 16 GB","swap 1.1 GB","rust-analyzer","4812"],
                    "locator_wait_error":wait_error.to_string(),
                    "visual_quiet_error":quiet_error,
                    "failure_evidence_error":capture_error.to_string(),
                    "wrong_destination_confirmed":false,
                }));
                return Err(format!(
                    "initial-resource wait failed and active-screen evidence could not be captured: {capture_error}"
                )
                .into());
            }
        }
    }

    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[1],
        case_out,
    )?);
    let initial_resource_assertion =
        holla_resource_page_assertion("initial-resource", last_checkpoint_snapshot(checkpoints)?);
    semantic_assertions.push(initial_resource_assertion.clone());
    if !assertion_passed(&initial_resource_assertion) {
        return Err(assertion_error(&initial_resource_assertion).into());
    }
    session
        .get_by_text("System snapshot")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;

    if !assertion_passed(&initial_assertion) {
        return Err(assertion_error(&initial_assertion).into());
    }

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text("System resources")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("4812")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    ensure_child_active(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[2],
        case_out,
    )?);
    let home_restored_assertion = holla_home_assertion(
        "home-restored",
        last_checkpoint_snapshot(checkpoints)?,
        "empty",
        false,
        false,
    );
    semantic_assertions.push(home_restored_assertion.clone());
    if !assertion_passed(&home_restored_assertion) {
        return Err(assertion_error(&home_restored_assertion).into());
    }

    for key in ["r", "e", "s", "o", "u", "r", "c", "e", "s"] {
        actions.push(json!({"event":"key_press","key":key,"at_unix_ms":unix_ms()}));
        send_key(session, key)?;
    }
    session
        .get_by_text("resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("Show system resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[3],
        case_out,
    )?);
    let query_selected_assertion = holla_home_assertion(
        "query-selected",
        last_checkpoint_snapshot(checkpoints)?,
        "resources",
        false,
        true,
    );
    semantic_assertions.push(query_selected_assertion.clone());
    if !assertion_passed(&query_selected_assertion) {
        return Err(assertion_error(&query_selected_assertion).into());
    }

    actions.push(json!({"event":"key_press","key":"Enter","at_unix_ms":unix_ms()}));
    send_key(session, "Enter")?;
    if let Err(wait_error) = session
        .get_by_text("System resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))
    {
        let quiet_error = wait_visual_quiet(session)
            .err()
            .map(|error| error.to_string());
        match capture_failure_evidence(
            session,
            profile,
            case,
            case_out,
            &raw_path,
            "query-resource",
        ) {
            Ok(evidence) => {
                let mut assertion =
                    holla_resource_page_assertion("query-resource", &evidence.snapshot);
                let expected_page_visible = assertion_passed(&assertion);
                let screen_quiet = quiet_error.is_none();
                if let Some(object) = assertion.as_object_mut() {
                    object.insert(
                        "locator_wait_error".to_string(),
                        json!(wait_error.to_string()),
                    );
                    object.insert("visual_quiet_error".to_string(), json!(quiet_error));
                    object.insert("failure_evidence".to_string(), evidence.record.clone());
                    object.insert(
                        "failure_state_observation".to_string(),
                        evidence.state_observation.clone(),
                    );
                    object.insert(
                        "screen_observable".to_string(),
                        json!(evidence.screen_observable),
                    );
                    object.insert("screen_quiet".to_string(), json!(screen_quiet));
                    if !evidence.child_active || !evidence.screen_observable {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("active screen or live child could not be verified"),
                        );
                    } else if !screen_quiet {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("captured screen did not reach visual quiet"),
                        );
                    } else if expected_page_visible {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("locator wait timed out although the captured snapshot showed the expected page"),
                        );
                    }
                }
                semantic_assertions.push(assertion);
                return Err(
                    "query-resource did not show the System resources page before timeout; evidence: failure-evidence/query-resource"
                        .into(),
                );
            }
            Err(capture_error) => {
                semantic_assertions.push(json!({
                    "id":"query-resource",
                    "status":"unverified",
                    "expected_visible":["System resources","12.4 of 16 GB","swap 1.1 GB","rust-analyzer","4812"],
                    "locator_wait_error":wait_error.to_string(),
                    "visual_quiet_error":quiet_error,
                    "failure_evidence_error":capture_error.to_string(),
                    "screen_observable":false,
                }));
                return Err(format!(
                    "query-resource wait failed and active-screen evidence could not be captured: {capture_error}"
                )
                .into());
            }
        }
    }
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[4],
        case_out,
    )?);
    let query_resource_assertion =
        holla_resource_page_assertion("query-resource", last_checkpoint_snapshot(checkpoints)?);
    semantic_assertions.push(query_resource_assertion.clone());
    if !assertion_passed(&query_resource_assertion) {
        return Err(assertion_error(&query_resource_assertion).into());
    }
    session
        .get_by_text("System snapshot")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text("System resources")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("4812")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    ensure_child_active(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[5],
        case_out,
    )?);
    let query_restored_assertion = holla_home_assertion(
        "query-restored",
        last_checkpoint_snapshot(checkpoints)?,
        "resources",
        false,
        false,
    );
    semantic_assertions.push(query_restored_assertion.clone());
    if !assertion_passed(&query_restored_assertion) {
        return Err(assertion_error(&query_restored_assertion).into());
    }

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text("Search actions and resources…")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    ensure_child_active(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[6],
        case_out,
    )?);
    let query_cleared_assertion = holla_home_assertion(
        "query-cleared",
        last_checkpoint_snapshot(checkpoints)?,
        "empty",
        false,
        false,
    );
    semantic_assertions.push(query_cleared_assertion.clone());
    if !assertion_passed(&query_cleared_assertion) {
        return Err(assertion_error(&query_cleared_assertion).into());
    }

    Ok(())
}

fn last_checkpoint_snapshot(checkpoints: &[Checkpoint]) -> AppResult<&TerminalSnapshot> {
    checkpoints
        .last()
        .map(|checkpoint| &checkpoint.snapshot)
        .ok_or_else(|| "no captured checkpoint is available for semantic assertions".into())
}

fn snapshot_text(snapshot: &TerminalSnapshot) -> String {
    (0..snapshot.rows())
        .filter_map(|row_index| snapshot.row(row_index))
        .map(|row| {
            row.iter()
                .map(|cell| {
                    if cell.has(Attrs::INVISIBLE) {
                        ""
                    } else {
                        cell.text()
                    }
                })
                .collect::<Vec<_>>()
                .join("")
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn holla_result_list_text(snapshot: &TerminalSnapshot) -> String {
    let x_start = HOLLA_RESULT_LIST_X_START;
    let x_end = HOLLA_RESULT_LIST_X_END.min(snapshot.cols());
    let y_start = HOLLA_RESULT_LIST_Y_START.min(snapshot.rows());
    let y_end = HOLLA_RESULT_LIST_Y_END.min(snapshot.rows());
    (y_start..y_end)
        .filter_map(|row_index| snapshot.row(row_index))
        .map(|row| {
            row.iter()
                .skip(x_start as usize)
                .take(x_end.saturating_sub(x_start) as usize)
                .map(|cell| {
                    if cell.has(Attrs::INVISIBLE) {
                        ""
                    } else {
                        cell.text()
                    }
                })
                .collect::<Vec<_>>()
                .join("")
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn holla_home_assertion(
    id: &str,
    snapshot: &TerminalSnapshot,
    query_state: &str,
    initial: bool,
    filtered: bool,
) -> Value {
    let actual_text = snapshot_text(snapshot);
    let result_list_text = holla_result_list_text(snapshot);
    let mut required = vec!["› Show system resources", "holla snapshot system", "mbp"];
    if initial {
        required.extend(["CPU 78% for 4 min", "still discovering…", "no project"]);
    }
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|marker| !actual_text.contains(marker))
        .collect();
    let forbidden = ["System snapshot"];
    let forbidden_visible: Vec<&str> = forbidden
        .iter()
        .copied()
        .filter(|marker| actual_text.contains(marker))
        .collect();
    let query_line = actual_text.lines().nth(4).unwrap_or_default();
    let query_matches = match query_state {
        "empty" => query_line.contains("Search actions and resources…"),
        "resources" => {
            query_line.contains("resources")
                && !query_line.contains("Search actions and resources…")
        }
        _ => false,
    };
    let unexpected_rows: Vec<&str> = if filtered {
        ["Analyze disk usage", "Open btm"]
            .into_iter()
            .filter(|marker| result_list_text.contains(marker))
            .collect()
    } else {
        Vec::new()
    };
    let status = if missing.is_empty()
        && forbidden_visible.is_empty()
        && unexpected_rows.is_empty()
        && query_matches
    {
        "pass"
    } else {
        "fail"
    };
    json!({
        "id":id,
        "status":status,
        "required_visible":required,
        "missing_visible":missing,
        "forbidden_visible":forbidden_visible,
        "unexpected_default_rows":unexpected_rows,
        "observed_result_list_text":result_list_text,
        "query_line":query_line,
        "query_state":query_state,
        "query_matches":query_matches,
        "observed_visible_text":actual_text,
    })
}

fn holla_resource_page_assertion(id: &str, snapshot: &TerminalSnapshot) -> Value {
    let actual_text = snapshot_text(snapshot);
    let required = [
        "System resources",
        "mbp",
        "CPU",
        "Load",
        "Cores",
        "Memory",
        "Pressure",
        "12.4 of 16 GB",
        "swap 1.1 GB",
        "rust-analyzer",
        "4812",
    ];
    let missing: Vec<&str> = required
        .into_iter()
        .filter(|marker| !actual_text.contains(marker))
        .collect();
    let forbidden = ["System snapshot"];
    let forbidden_visible: Vec<&str> = forbidden
        .into_iter()
        .filter(|marker| actual_text.contains(marker))
        .collect();
    let status = if missing.is_empty() && forbidden_visible.is_empty() {
        "pass"
    } else {
        "fail"
    };
    json!({
        "id":id,
        "status":status,
        "required_visible":required,
        "missing_visible":missing,
        "forbidden_visible":forbidden_visible,
        "wrong_destination_visible":actual_text.contains("System snapshot")
            && !actual_text.contains("System resources"),
        "observed_visible_text":actual_text,
    })
}

fn assertion_passed(assertion: &Value) -> bool {
    assertion["status"] == "pass"
}

fn assertion_error(assertion: &Value) -> String {
    let missing = assertion["missing_visible"].to_string();
    let forbidden = assertion["forbidden_visible"].to_string();
    let unexpected = assertion["unexpected_default_rows"].to_string();
    format!(
        "{} semantic assertion failed: missing={missing}, forbidden={forbidden}, unexpected_rows={unexpected}",
        assertion["id"].as_str().unwrap_or("unknown")
    )
}

fn capture_failure_evidence(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    case_out: &Path,
    raw_path: &Path,
    stage: &str,
) -> AppResult<FailureScreenEvidence> {
    let (snapshot, offset) =
        session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
    let (state_observation, child_active) = failure_state_observation(session);
    let offset = offset.ok_or_else(|| {
        format!(
            "processed PTY byte offset is unavailable or overflowed; state: {state_observation}"
        )
    })?;
    if snapshot.cols() != u32::from(case.cols) || snapshot.rows() != u32::from(case.rows) {
        return Err(format!(
            "failure snapshot dimensions are {}x{}, expected {}x{}; state: {state_observation}",
            snapshot.cols(),
            snapshot.rows(),
            case.cols,
            case.rows
        )
        .into());
    }
    let actual_text = snapshot_text(&snapshot);
    let screen_observable = !actual_text.trim().is_empty();
    let directory = case_out.join("failure-evidence").join(stage);
    let sample_id = format!(
        "{}/failure-evidence/{stage}/pixels-{}",
        case.id,
        platform_key()
    );
    let sample = Sample::with_pixels(&sample_id, &snapshot, Observation::empty(), profile)?;
    let sample_bytes = sample.to_bytes()?;
    let image = sample
        .image()
        .ok_or("failure evidence sample has no PNG image")?;
    let png = base64::engine::general_purpose::STANDARD.decode(image.png_base64())?;
    let raw = fs::read(raw_path)?;
    let prefix_end = usize::try_from(offset)?;
    if prefix_end > raw.len() {
        return Err(format!(
            "failure snapshot offset {prefix_end} exceeds raw file length {}",
            raw.len()
        )
        .into());
    }
    fs::create_dir_all(&directory)?;
    write_new(&directory.join("sample.tui.json"), &sample_bytes)?;
    write_new(
        &directory.join("decoded-snapshot.json"),
        &serde_json::to_vec_pretty(&snapshot)?,
    )?;
    write_new(&directory.join("screen.png"), &png)?;
    write_new(&directory.join("visible-text.txt"), actual_text.as_bytes())?;
    write_new(&directory.join("raw-prefix.ansi"), &raw[..prefix_end])?;
    write_json_new(
        &directory.join("capture.json"),
        &json!({
            "stage":stage,
            "sample_id":sample_id,
            "processed_output_offset":offset,
            "state_observation":state_observation,
            "child_active":child_active,
            "screen_observable":screen_observable,
            "raw_prefix_sha256":sha256(&raw[..prefix_end]),
            "raw_stream_bytes_at_capture":raw.len(),
            "raw_stream_sha256_at_capture":sha256(&raw),
        }),
    )?;
    Ok(FailureScreenEvidence {
        record: json!({
            "directory":format!("failure-evidence/{stage}"),
            "sample_id":sample_id,
            "processed_output_offset":offset,
            "child_active":child_active,
            "screen_observable":screen_observable,
            "state_observation":state_observation,
            "sample_sha256":sha256(&sample_bytes),
            "png_sha256":image.png_sha256(),
            "rgba_sha256":image.rgba_sha256(),
            "raw_prefix_sha256":sha256(&raw[..prefix_end]),
        }),
        snapshot,
        state_observation,
        child_active,
        screen_observable,
    })
}

fn failure_state_observation(session: &Session) -> (Value, bool) {
    match session.execute(Operation::State) {
        Ok(OperationResult::State(state)) => {
            let child_active = state.exited.is_none() && state.exit_signal.is_none();
            (
                json!({
                    "status":"available",
                    "exited":state.exited,
                    "exit_signal":state.exit_signal,
                    "cols":state.cols,
                    "rows":state.rows,
                    "child_active":child_active,
                }),
                child_active,
            )
        }
        Ok(other) => (
            json!({
                "status":"unavailable",
                "error":format!("unexpected process state result: {other:?}"),
            }),
            false,
        ),
        Err(error) => (
            json!({"status":"unavailable","error":error.to_string()}),
            false,
        ),
    }
}

fn finalize_holla_after_error(session: &Session, actions: &mut Vec<Value>) -> (Value, Value, bool) {
    actions.push(json!({
        "event":"cleanup_key_press",
        "key":"Ctrl+q",
        "reason":"bounded_finalization_after_assertion_failure",
        "at_unix_ms":unix_ms(),
    }));
    let key_result = send_key(session, "Ctrl+q");
    let wait_result = session.execute(Operation::WaitExit {
        timeout_ms: Some(EXIT_TIMEOUT_MS),
    });
    let state_result = session.execute(Operation::State);
    let (state_value, process_state, child_exited, state_error) = match state_result {
        Ok(OperationResult::State(state)) => {
            let exited = state.exited.is_some() || state.exit_signal.is_some();
            let value = json!({
                "exited":state.exited,
                "exit_signal":state.exit_signal,
                "cols":state.cols,
                "rows":state.rows,
            });
            (value.clone(), value, exited, Value::Null)
        }
        Ok(other) => {
            let error = format!("unexpected process state result: {other:?}");
            (Value::Null, Value::Null, false, json!(error))
        }
        Err(error) => (Value::Null, Value::Null, false, json!(error.to_string())),
    };
    let (wait_exit_completed, wait_exit_error) = match wait_result {
        Ok(_) => (true, Value::Null),
        Err(error) => (false, json!(error.to_string())),
    };
    let clean_exit = wait_exit_completed
        && state_error.is_null()
        && process_state["exited"] == 0
        && process_state["exit_signal"].is_null();
    let finalization = json!({
        "attempted":true,
        "quit_key":"Ctrl+q",
        "quit_key_sent":key_result.is_ok(),
        "quit_key_error":key_result.err().map(|error| error.to_string()),
        "wait_exit_completed":wait_exit_completed,
        "wait_exit_error":wait_exit_error,
        "state":state_value,
        "state_error":state_error,
        "child_exited":child_exited,
        "clean_exit":clean_exit,
    });
    (finalization, process_state, child_exited)
}

fn preserve_reached_checkpoint_prefixes(
    raw_path: &Path,
    case_out: &Path,
    checkpoints: &[Checkpoint],
    exit_receipt: Option<&Checkpoint>,
) -> Vec<String> {
    let raw = match fs::read(raw_path) {
        Ok(raw) => raw,
        Err(error) => return vec![format!("cannot read raw stream: {error}")],
    };
    let mut errors = Vec::new();
    for checkpoint in checkpoints.iter().chain(exit_receipt) {
        let checkpoint_dir = case_out.join("checkpoints").join(&checkpoint.label);
        let prefix_path = checkpoint_dir.join("raw-prefix.ansi");
        let capture_path = checkpoint_dir.join("capture.json");
        if prefix_path.is_file() && capture_path.is_file() {
            continue;
        }
        let end = match usize::try_from(checkpoint.offset) {
            Ok(end) if end <= raw.len() => end,
            Ok(end) => {
                errors.push(format!(
                    "{} offset {end} exceeds final raw length {}",
                    checkpoint.checkpoint_id,
                    raw.len()
                ));
                continue;
            }
            Err(error) => {
                errors.push(format!(
                    "{} offset conversion failed: {error}",
                    checkpoint.checkpoint_id
                ));
                continue;
            }
        };
        if let Err(error) = fs::create_dir_all(&checkpoint_dir) {
            errors.push(format!(
                "{} output directory failed: {error}",
                checkpoint.checkpoint_id
            ));
            continue;
        }
        if !prefix_path.exists()
            && let Err(error) = write_new(&prefix_path, &raw[..end])
        {
            errors.push(format!(
                "{} raw prefix failed: {error}",
                checkpoint.checkpoint_id
            ));
            continue;
        }
        if !capture_path.exists()
            && let Err(error) = write_json_new(
                &capture_path,
                &json!({
                    "checkpoint_id":checkpoint.checkpoint_id,
                    "sample_id":checkpoint.sample_id,
                    "processed_output_offset":checkpoint.offset,
                    "raw_prefix_sha256":sha256(&raw[..end]),
                    "raw_stream_sha256":sha256(&raw),
                    "raw_stream_bytes":raw.len(),
                    "exit_receipt":checkpoint.label == "exit-receipt",
                }),
            )
        {
            errors.push(format!(
                "{} capture record failed: {error}",
                checkpoint.checkpoint_id
            ));
        }
    }
    errors
}

fn capture_checkpoint(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    label: &str,
    case_out: &Path,
) -> AppResult<Checkpoint> {
    let (snapshot, offset) =
        session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
    let offset = offset.ok_or("processed PTY byte offset is unavailable or overflowed")?;
    let captured_at_unix_ms = unix_ms();
    let (expected_cols, expected_rows) = checkpoint_dimensions(case, label);
    if !matches!(case.journey, Journey::First30(_))
        && (snapshot.cols() != u32::from(expected_cols)
            || snapshot.rows() != u32::from(expected_rows))
    {
        return Err(format!(
            "snapshot dimensions are {}x{}, expected {}x{}",
            snapshot.cols(),
            snapshot.rows(),
            expected_cols,
            expected_rows
        )
        .into());
    }
    let checkpoint_id = format!("{}/{label}", case.id);
    let sample_suffix = if matches!(case.journey, Journey::First30(_)) {
        label
    } else {
        match label {
            "opened" => "initial",
            "returned" => "restored",
            "exited" => "exit",
            other => other,
        }
    };
    let sample_id = format!("{}/{sample_suffix}/pixels-{}", case.id, platform_key());
    let sample = Sample::with_pixels(&sample_id, &snapshot, Observation::empty(), profile)?;
    let sample_bytes = sample.to_bytes()?;
    let image = sample.image().ok_or("pixel sample has no PNG image")?;
    let png = base64::engine::general_purpose::STANDARD.decode(image.png_base64())?;
    let checkpoint_dir = case_out.join("checkpoints").join(label);
    fs::create_dir_all(&checkpoint_dir)?;
    write_new(&checkpoint_dir.join("sample.tui.json"), &sample_bytes)?;
    write_new(
        &checkpoint_dir.join("decoded-snapshot.json"),
        &serde_json::to_vec_pretty(&snapshot)?,
    )?;
    write_new(&checkpoint_dir.join("screen.png"), &png)?;
    Ok(Checkpoint {
        label: label.to_string(),
        checkpoint_id,
        sample_id,
        offset,
        captured_at_unix_ms,
        snapshot,
        sample_bytes,
        png_sha256: image.png_sha256().to_string(),
        rgba_sha256: image.rgba_sha256().to_string(),
    })
}

fn checkpoint_dimensions(case: Case, label: &str) -> (u16, u16) {
    if matches!(case.journey, Journey::First30(_)) {
        if label == "small" {
            (80, 24)
        } else {
            (DEFAULT_COLS, DEFAULT_ROWS)
        }
    } else {
        (case.cols, case.rows)
    }
}

fn visible_text_changed(left: &TerminalSnapshot, right: &TerminalSnapshot) -> bool {
    for row in 0..left.rows().min(right.rows()) {
        for col in 0..left.cols().min(right.cols()) {
            let Some(left_cell) = left.cell_at(col, row) else {
                continue;
            };
            let Some(right_cell) = right.cell_at(col, row) else {
                continue;
            };
            if left_cell.text() != right_cell.text() {
                return true;
            }
        }
    }
    false
}

fn assert_same_checkpoint_state(
    left: &Checkpoint,
    right: &Checkpoint,
    message: &str,
) -> AppResult<()> {
    if let Some(difference) = first_difference(&left.snapshot, &right.snapshot) {
        return Err(format!("{message}: {difference}").into());
    }
    let left_sample = Sample::from_bytes(&left.sample_bytes)?;
    let right_sample = Sample::from_bytes(&right.sample_bytes)?;
    if left_sample.render_contract() != right_sample.render_contract() {
        return Err(format!("{message}: render contracts differ").into());
    }
    if left.png_sha256 != right.png_sha256 || left.rgba_sha256 != right.rgba_sha256 {
        return Err(format!("{message}: rendered pixels differ").into());
    }
    Ok(())
}

macro_rules! compare_cell_field {
    ($prefix:expr, $field:expr, $left:expr, $right:expr) => {
        if $left != $right {
            return Some(format!(
                "{}.{}: left={:?}, right={:?}",
                $prefix, $field, $left, $right
            ));
        }
    };
}

fn first_difference(left: &TerminalSnapshot, right: &TerminalSnapshot) -> Option<String> {
    macro_rules! compare_field {
        ($field:literal, $left:expr, $right:expr) => {
            if $left != $right {
                return Some(format!("{}: left={:?}, right={:?}", $field, $left, $right));
            }
        };
    }
    compare_field!("cols", left.cols(), right.cols());
    compare_field!("rows", left.rows(), right.rows());
    compare_field!("scope", left.scope(), right.scope());
    compare_field!(
        "viewport_origin",
        left.viewport_origin(),
        right.viewport_origin()
    );
    compare_field!("palette", left.palette(), right.palette());
    compare_field!(
        "default_foreground",
        left.default_foreground(),
        right.default_foreground()
    );
    compare_field!(
        "default_background",
        left.default_background(),
        right.default_background()
    );
    compare_field!(
        "default_cursor",
        left.default_cursor(),
        right.default_cursor()
    );
    compare_field!("title", left.title(), right.title());
    let a = left.cursor();
    let b = right.cursor();
    compare_field!("cursor.col", a.col(), b.col());
    compare_field!("cursor.row", a.row(), b.row());
    compare_field!("cursor.in_capture", a.in_capture(), b.in_capture());
    compare_field!("cursor.visible", a.visible(), b.visible());
    compare_field!("cursor.shape", a.shape(), b.shape());
    compare_field!("cursor.color", a.color(), b.color());
    compare_field!("cursor.blink", a.blink(), b.blink());
    for row in 0..left.rows().min(right.rows()) {
        for col in 0..left.cols().min(right.cols()) {
            let Some(a) = left.cell_at(col, row) else {
                continue;
            };
            let Some(b) = right.cell_at(col, row) else {
                continue;
            };
            let prefix = format!("cell[{col},{row}]");
            if a.text() != b.text() {
                return Some(format!(
                    "{prefix}.text: left={:?}, right={:?}",
                    a.text(),
                    b.text()
                ));
            }
            compare_cell_field!(&prefix, "width", a.width(), b.width());
            compare_cell_field!(&prefix, "fg", a.fg(), b.fg());
            compare_cell_field!(&prefix, "bg", a.bg(), b.bg());
            for (name, attr) in [
                ("bold", Attrs::BOLD),
                ("dim", Attrs::DIM),
                ("italic", Attrs::ITALIC),
                ("inverse", Attrs::INVERSE),
                ("invisible", Attrs::INVISIBLE),
                ("strike", Attrs::STRIKE),
                ("blink", Attrs::BLINK),
            ] {
                compare_cell_field!(&prefix, name, a.has(attr), b.has(attr));
            }
            compare_cell_field!(&prefix, "underline", a.underline(), b.underline());
            compare_cell_field!(
                &prefix,
                "underline_color",
                a.underline_color(),
                b.underline_color()
            );
            compare_cell_field!(&prefix, "link_uri", a.link_uri(), b.link_uri());
            compare_cell_field!(&prefix, "link_id", a.link_id(), b.link_id());
        }
    }
    None
}

fn compare_checkpoint(
    frozen_store: &FrozenStore,
    suite: &Path,
    case_out: &Path,
    case_id: &str,
    checkpoint: &Checkpoint,
    capture_kind: &str,
) -> (Value, Option<String>) {
    let mut check_value = match frozen_store.check(&checkpoint.sample_bytes) {
        Ok(outcome) => {
            let mut value = check_json(&outcome);
            if let CheckOutcome::Failure(failure) = &outcome
                && failure.reason().as_str() != "missing"
            {
                let baseline_path = suite
                    .join("expected")
                    .join(format!("{}.tui.json", checkpoint.sample_id));
                if baseline_path.is_file()
                    && let Ok(bytes) = fs::read(baseline_path)
                    && let Ok(baseline) = Sample::from_bytes(&bytes)
                    && let Some(object) = value.as_object_mut()
                {
                    object.insert(
                        "first_difference".to_string(),
                        json!(first_difference(baseline.snapshot(), &checkpoint.snapshot)),
                    );
                    object.insert(
                        "changed_fields".to_string(),
                        json!(failure.changed_fields()),
                    );
                    object.insert(
                        "changed_cells".to_string(),
                        json!(
                            failure
                                .changed_cells()
                                .iter()
                                .map(|cell| json!({
                                    "col": cell.col(),
                                    "row": cell.row(),
                                    "fields": cell.fields(),
                                    "actual_text": cell.actual_text(),
                                    "baseline_text": cell.baseline_text(),
                                }))
                                .collect::<Vec<_>>()
                        ),
                    );
                }
            }
            value
        }
        Err(error) => json!({
            "status":"unavailable",
            "comparison_status":"unavailable",
            "classification":"setup",
            "outcome":"ComparisonUnavailable",
            "test_id":checkpoint.sample_id,
            "candidate_sha256":sha256(&checkpoint.sample_bytes),
            "reason":error.to_string(),
        }),
    };
    let check_path = case_out
        .join("checkpoints")
        .join(&checkpoint.label)
        .join("check.json");
    let write_error = write_json_new(&check_path, &check_value)
        .err()
        .map(|error| {
            format!(
                "{} comparison record write failed: {error}",
                checkpoint.checkpoint_id
            )
        });
    if let Some(error) = write_error.as_ref()
        && let Some(object) = check_value.as_object_mut()
    {
        object.insert("check_record_write_error".to_string(), json!(error));
    }
    let record = json!({
        "case_id":case_id,
        "checkpoint_id":checkpoint.checkpoint_id,
        "label":checkpoint.label,
        "sample_id":checkpoint.sample_id,
        "capture_kind":capture_kind,
        "status":check_value["status"],
        "comparison_status":check_value["comparison_status"],
        "classification":check_value["classification"],
        "check_path":format!("checkpoints/{}/check.json", checkpoint.label),
        "check":check_value,
    });
    (record, write_error)
}

fn check_json(outcome: &CheckOutcome) -> Value {
    match outcome {
        CheckOutcome::Pass(success) => json!({
            "status":"pass",
            "comparison_status":"compared",
            "classification":"comparison",
            "outcome":"CheckOutcomePass",
            "test_id":success.test_id(),
            "candidate_sha256":success.candidate_sha256(),
            "baseline_sha256":success.baseline_sha256(),
        }),
        CheckOutcome::Failure(failure) => {
            let missing_baseline = failure.reason().as_str() == "missing";
            json!({
                "status":if missing_baseline {"setup_failed"} else {"failure"},
                "comparison_status":if missing_baseline {"missing_baseline"} else {"compared"},
                "classification":if missing_baseline {"setup"} else {"comparison"},
                "outcome":"CheckOutcomeFailure",
                "test_id":failure.test_id(),
                "reason":failure.reason().as_str(),
                "candidate_sha256":failure.candidate_sha256(),
                "baseline_sha256":failure.baseline_sha256(),
                "candidate_path":failure.candidate_path(),
                "evidence_dir":failure.evidence_dir(),
                "report_path":failure.report_path(),
                "changed_fields":failure.changed_fields(),
                "changed_cell_total":failure.changed_cell_total(),
                "pixel_changed":failure.pixel_changed(),
                "pixel_compared":failure.pixel_compared(),
                "pixel_bounds":failure.pixel_bounds(),
            })
        }
    }
}

fn ensure_child_active(session: &Session) -> AppResult<()> {
    let state = match session.execute(Operation::State)? {
        OperationResult::State(state) => state,
        other => return Err(format!("unexpected state result: {other:?}").into()),
    };
    if state.exited.is_some() || state.exit_signal.is_some() {
        return Err(format!(
            "child exited before the initial checkpoint: exited={:?}, signal={:?}",
            state.exited, state.exit_signal
        )
        .into());
    }
    Ok(())
}

fn wait_visual_quiet(session: &Session) -> AppResult<()> {
    let options =
        WaitQuietOptions::new(QUIET_TIMEOUT, QUIET_INTERVAL)?.with_mode(WaitQuietMode::Visual);
    session.wait_quiet(&options)?;
    Ok(())
}

fn send_key(session: &Session, key: &str) -> AppResult<()> {
    session.execute(Operation::Key {
        keys: vec![key.to_string()],
        action: KeyAction::Press,
    })?;
    Ok(())
}

fn suite_root() -> AppResult<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest.canonicalize()?)
}

fn verify_export(bin_dir: &Path, run_out: &Path) -> AppResult<Value> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("export-manifest.json");
    let helper = root.join("export.py");
    let output = std::process::Command::new("python3")
        .arg("-I")
        .arg("-c")
        .arg(EXPORT_LOADER)
        .arg(&manifest)
        .arg(&helper)
        .arg(BUILD_EXPORT_SHA256)
        .arg("verify")
        .arg("--root")
        .arg(&root)
        .arg("--sha256")
        .arg(BUILD_EXPORT_SHA256)
        .arg("--bin-dir")
        .arg(bin_dir)
        .arg("--run-out")
        .arg(run_out)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "sealed export verification failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let provenance: Value = serde_json::from_slice(&output.stdout)?;
    if provenance["export_sha256"] != BUILD_EXPORT_SHA256 {
        return Err("export verifier returned a different seal".into());
    }
    Ok(provenance)
}

fn verify_application_root(source_manifest: &Value, export_provenance: &Value) -> AppResult<()> {
    let source_worktree = source_manifest["source_worktree"]
        .as_str()
        .ok_or("application source manifest has no source_worktree")?;
    let boundaries = &export_provenance["boundaries"];
    let reference = boundaries["reference_application_root"].as_str();
    let candidate = boundaries["candidate_application_root"].as_str();
    if reference != Some(source_worktree) && candidate != Some(source_worktree) {
        return Err("application source_worktree is not a sealed application root".into());
    }
    Ok(())
}

fn prepare_empty_output(path: &Path) -> AppResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            return Err(format!("output path already exists: {}", path.display()).into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path)?;
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

fn write_json_new(path: &Path, value: &impl serde::Serialize) -> AppResult<()> {
    write_new(path, &serde_json::to_vec_pretty(value)?)
}

fn sha256_file(path: &Path) -> AppResult<String> {
    Ok(sha256(&fs::read(path)?))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut result = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut result, "{byte:02x}");
    }
    result
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
