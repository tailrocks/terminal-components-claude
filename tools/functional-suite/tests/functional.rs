use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tui_test::profile::Profile;
use tui_test::render::style::Style;
use tui_test::terminal::cell::Attrs;
use tui_test::{
    AutomaticRecording, Backend, KeyAction, LaunchEnvMode, LaunchOptions, MouseAction, MouseButton,
    MouseOptions, Operation, OperationResult, RunOptions, Session, SnapshotScope, Timeouts,
    WaitQuietOptions,
};
use tuiscotti::check::{CheckFailure, CheckOutcome, FrozenStore, MismatchReason};
use tuiscotti::render_profile::{
    PROFILE_NAME, Profile as PixelProfile, platform_key, resolve_profile,
};
use tuiscotti::sample::{ContractFont, Observation, RenderContract, Sample, SampleImage};

const MANIFEST_TOML: &str = include_str!("../cases/cases.toml");
const DATASET_JSON: &str = include_str!("../fixtures/dataset.json");
const CASE_LOCK_TIMEOUT: Duration = Duration::from_secs(120);
const CASE_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(25);
const OWNER_MARKER_TIMEOUT: Duration = Duration::from_secs(5);
const OWNER_MARKER_POLL_INTERVAL: Duration = Duration::from_millis(25);
const READINESS_TIMEOUT: Duration = Duration::from_secs(8);
const READINESS_POLL_INTERVAL: Duration = Duration::from_millis(100);
const SETTLE_TIMEOUT: Duration = Duration::from_secs(8);
const SETTLE_QUIET: Duration = Duration::from_millis(400);
const EXIT_TIMEOUT_MS: u64 = 20_000;
const RAW_DRAIN_TIMEOUT: Duration = Duration::from_secs(10);
const LEGACY_FIRST_CASES: [&str; 4] = [
    "first_showcase",
    "first_holla",
    "first_jackin",
    "first_tablepro",
];
const DELIVERY_CASES: [&str; 4] = [
    "first_delivery_showcase_buttons",
    "first_delivery_holla_first_use",
    "first_delivery_jackin_settings",
    "first_delivery_tablepro_connections",
];
const DELIVERY_FRAME_TOTAL: usize = 30;

const LEGACY_CASES: [&str; 84] = [
    "first_showcase",
    "first_holla",
    "first_jackin",
    "first_tablepro",
    "showcase_overview",
    "showcase_chrome",
    "showcase_status_and_hints",
    "showcase_breadcrumbs_and_rails",
    "showcase_help_and_keys",
    "showcase_quit_and_leave",
    "showcase_focus_and_navigation",
    "showcase_buttons",
    "showcase_inputs",
    "showcase_text_areas",
    "showcase_chips_and_selects",
    "showcase_forms",
    "showcase_checks_and_radios",
    "showcase_toolbars",
    "showcase_basic_table",
    "showcase_editable",
    "showcase_data_grid",
    "showcase_selection",
    "showcase_cell_editors",
    "showcase_columns",
    "showcase_lists",
    "showcase_trees",
    "showcase_sidebars",
    "showcase_tabs",
    "showcase_menus",
    "showcase_dialogs",
    "showcase_pickers",
    "showcase_dialog_kinds",
    "showcase_popups_and_tooltips",
    "showcase_file_browser",
    "showcase_progress",
    "showcase_empty_and_error",
    "showcase_meters",
    "showcase_notices",
    "showcase_panels",
    "showcase_scrolling",
    "showcase_splits_and_stacks",
    "showcase_master_detail",
    "showcase_pane_tree",
    "showcase_code_editor",
    "showcase_diff",
    "showcase_terminal",
    "showcase_text_utilities",
    "showcase_settings",
    "showcase_task_runner",
    "showcase_query_workbench",
    "showcase_connections",
    "showcase_finder",
    "showcase_disk_cleanup",
    "showcase_activity",
    "showcase_workspaces",
    "showcase_accounts_and_usage",
    "showcase_launch_cockpit",
    "showcase_read_only_report",
    "showcase_explorer",
    "showcase_orders",
    "showcase_query_edit_while_running",
    "showcase_query_tab_switching",
    "holla_home_actions",
    "holla_plan_args_gate",
    "holla_activity_stop",
    "holla_disk_cleanup_review",
    "holla_files_clone",
    "holla_taskrun_args_cancel",
    "holla_database_blockers",
    "jackin_intro_prelude",
    "jackin_editor_save_cancel_leave",
    "jackin_accounts_usage",
    "jackin_settings",
    "jackin_launch_cancel_failure",
    "jackin_handoff_capsule",
    "jackin_capsule_controls",
    "jackin_protected_quit_outro",
    "tablepro_connection_retry",
    "tablepro_explorer_structure_grid",
    "tablepro_edit_transaction",
    "tablepro_query_execution",
    "tablepro_safety_ack",
    "tablepro_history_filter_reconnect",
    "tablepro_switchers_dirty_close",
];
const EXPECTED_COVERAGE_ID_COUNT: usize = 276;

#[derive(Debug, Deserialize)]
struct SuiteManifest {
    version: SuiteVersion,
    suite: SuiteConfig,
    required_coverage: toml::Value,
    engine_pins: BTreeMap<String, String>,
    cases: Vec<Journey>,
}

#[derive(Debug, Deserialize)]
struct SuiteVersion {
    schema_version: u32,
    suite_id: String,
    predecessor_suite_id: String,
    result_inheritance: bool,
    crosswalk: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SuiteConfig {
    coverage_scope: String,
    terminal: TerminalConfig,
    render_profile: String,
    fixture_dataset: String,
    expected_store: String,
    theme: String,
    no_color: String,
    readiness_timeout_ms: u64,
    readiness_poll_interval_ms: u64,
    settle_timeout_ms: u64,
    settle_quiet_ms: u64,
}

#[derive(Debug, Deserialize)]
struct TerminalConfig {
    cols: u16,
    rows: u16,
    locale: String,
    timezone: String,
    term: String,
    color_term: String,
    color_mode_arguments: Vec<String>,
    motion_arguments: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Journey {
    name: String,
    case_id: Option<String>,
    binary: String,
    arguments: Vec<String>,
    help_key: Option<String>,
    return_key: Option<String>,
    exit_key: Option<String>,
    native_steps: Option<Vec<String>>,
    frame: Option<u64>,
    steps: Option<Vec<CheckpointPlan>>,
    phases: Option<Vec<JourneyPhase>>,
    coverage_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct JourneyPhase {
    name: String,
    motion: String,
    dialog_checkpoint: String,
    cleanup: String,
    steps: Vec<CheckpointPlan>,
}

#[derive(Debug, Deserialize)]
struct CheckpointPlan {
    name: String,
    required_anchors: Vec<String>,
    forbidden_anchors: Vec<String>,
    #[serde(default = "default_terminal_cols")]
    expected_cols: u16,
    #[serde(default = "default_terminal_rows")]
    expected_rows: u16,
    #[serde(default)]
    actions: Vec<NativeAction>,
    #[serde(default)]
    selection_observations: Vec<SelectionObservationPlan>,
}

#[derive(Debug, Deserialize, Clone)]
struct SelectionObservationPlan {
    label: String,
    x: u16,
    y: u16,
    selected: bool,
}

fn default_terminal_cols() -> u16 {
    120
}

fn default_terminal_rows() -> u16 {
    40
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NativeAction {
    Key { key: String },
    Write { text: String },
    Resize { cols: u16, rows: u16 },
    MouseClick { x: u16, y: u16, button: String },
}

#[derive(Debug, Deserialize)]
struct FixtureDataset {
    schema_version: u32,
    dataset_id: String,
    description: String,
    sandbox_root: String,
    work_directory: String,
    home_directory: String,
    data_directory: String,
    config_directory: String,
    cache_directory: String,
    state_directory: String,
    temporary_directory: String,
    owner_marker: String,
    owner_marker_contents: String,
}

#[derive(Debug)]
struct FixturePaths {
    cwd: PathBuf,
    env: Vec<(String, String)>,
    _case_lock: File,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CaseResultStatus {
    Pending,
    Passed,
    Failed,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CaseFailureKind {
    Setup,
    Application,
    Execution,
    Capture,
    Comparison,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ComparisonFailureKind {
    MissingBaseline,
    TestId,
    Policy,
    State,
    Contract,
    Dimensions,
    Pixels,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CaseStage {
    Manifest,
    Plan,
    Fixture,
    Launch,
    Journey,
    Finalize,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PhaseRunStatus {
    NotRun,
    SetupFailed,
    Launched,
    Passed,
    Failed,
}

#[derive(Debug, Serialize, Clone)]
struct PhaseResultEvidence {
    name: String,
    motion: String,
    cleanup: String,
    planned_checkpoint_names: Vec<String>,
    status: PhaseRunStatus,
    comparison_failure_kind: Option<ComparisonFailureKind>,
    launched: bool,
    cleanup_actions_attempted: Vec<String>,
    wait_exit_succeeded: Option<bool>,
    exit_status: Option<i32>,
    exit_status_zero: Option<bool>,
    exit_signal: Option<String>,
    no_signal: Option<bool>,
    eof_drained: Option<bool>,
    post_drain_exit_confirmed: Option<bool>,
    raw_pty_sha256: Option<String>,
    raw_checkpoint_prefixes: BTreeMap<String, String>,
    terminal_restored: Option<bool>,
    /// Relative to the directory containing this case-result receipt.
    cleanup_snapshot_file: Option<String>,
    cleanup_snapshot_sha256: Option<String>,
    cleanup_raw_prefix_sha256: Option<String>,
    error: Option<String>,
}

enum ReadinessOutcome {
    Ready,
    TimedOut(String),
}

#[derive(Debug)]
struct StoreCheckFailure(String);

impl std::fmt::Display for StoreCheckFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for StoreCheckFailure {}

#[derive(Debug, Serialize)]
struct CaseResultEvidence {
    schema_version: u32,
    suite_id: Option<String>,
    run_id: Option<String>,
    case_started_at: String,
    case_ended_at: Option<String>,
    test_name: String,
    case_id: Option<String>,
    engine_pins: BTreeMap<String, String>,
    fault_control: FaultControlEvidence,
    actions: Vec<ActionEventEvidence>,
    planned_checkpoint_names: Vec<String>,
    not_run_checkpoints: Vec<NotReachedCheckpointEvidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    phases: Vec<PhaseResultEvidence>,
    status: CaseResultStatus,
    failure_kind: Option<CaseFailureKind>,
    comparison_failure_kind: Option<ComparisonFailureKind>,
    stage: CaseStage,
    launched: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct FaultControlEvidence {
    active: bool,
    spec: Option<String>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ActionDispatchState {
    Attempted,
    Dispatched,
    Failed,
    SuppressedByControl,
}

#[derive(Debug, Serialize, Clone)]
struct ActionEventEvidence {
    action_id: String,
    case_id: String,
    phase_name: Option<String>,
    checkpoint_name: String,
    sequence: usize,
    action_kind: String,
    attempted_at: String,
    dispatch_completed_at: Option<String>,
    dispatch_state: ActionDispatchState,
    observation_at: Option<String>,
    observation_state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SelectionSample {
    test_id: String,
    snapshot: SelectionSnapshot,
}

#[derive(Debug, Deserialize)]
struct SelectionSnapshot {
    cols: u16,
    rows: u16,
    cells: Vec<SelectionCell>,
}

#[derive(Debug, Deserialize)]
struct SelectionCell {
    text: String,
    fg: String,
    bg: String,
    bold: bool,
    inverse: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
struct SelectionStyleEvidence {
    fg: String,
    bg: String,
    bold: bool,
    inverse: bool,
}

struct SelectionLabelMatch {
    occurrences: usize,
    x: Option<u16>,
    y: Option<u16>,
    style: Option<SelectionStyleEvidence>,
    attribute_uniform: bool,
}

#[derive(Debug, Serialize, Clone)]
struct SelectionObservationEvidence {
    label: String,
    x: u16,
    y: u16,
    expected_selected: bool,
    observed_selected: Option<bool>,
    matched: bool,
    occurrences: usize,
    actual_x: Option<u16>,
    actual_y: Option<u16>,
    attribute_uniform: bool,
    expected_style: SelectionStyleEvidence,
    actual_style: Option<SelectionStyleEvidence>,
    reference_files: BTreeMap<String, String>,
    problem: Option<String>,
}

struct CaseResultWriter {
    root: PathBuf,
    file_name: String,
    persist_to_disk: bool,
    evidence: CaseResultEvidence,
}

fn fault_control_evidence_from_env() -> FaultControlEvidence {
    match env::var("TERMROCK_PARITY_INPUT_CONTROL") {
        Ok(spec) => FaultControlEvidence {
            active: true,
            spec: Some(spec),
        },
        Err(env::VarError::NotPresent) => FaultControlEvidence {
            active: false,
            spec: None,
        },
        Err(env::VarError::NotUnicode(_)) => FaultControlEvidence {
            active: true,
            spec: Some("<non-utf8>".to_string()),
        },
    }
}

impl CaseResultWriter {
    fn create(output_root: &Path, test_name: &str) -> Result<Self> {
        ensure!(is_safe_single_component(test_name), "unsafe test name");
        let version_root = ensure_child_dir(output_root, "v3")?;
        let root = ensure_child_dir(&version_root, "case-results")?;
        let writer = Self {
            root,
            file_name: format!("{test_name}.json"),
            persist_to_disk: true,
            evidence: CaseResultEvidence {
                schema_version: 2,
                suite_id: None,
                run_id: env::var("TERMROCK_PARITY_RUN_ID")
                    .ok()
                    .and_then(|value| validate_run_id(&value)),
                case_started_at: utc_timestamp(),
                case_ended_at: None,
                test_name: test_name.to_string(),
                case_id: None,
                engine_pins: BTreeMap::new(),
                fault_control: fault_control_evidence_from_env(),
                actions: Vec::new(),
                planned_checkpoint_names: Vec::new(),
                not_run_checkpoints: Vec::new(),
                phases: Vec::new(),
                status: CaseResultStatus::Pending,
                failure_kind: None,
                comparison_failure_kind: None,
                stage: CaseStage::Manifest,
                launched: false,
                error: None,
            },
        };
        writer.persist()?;
        Ok(writer)
    }

    fn begin_action(
        &mut self,
        case_id: &str,
        phase_name: Option<&str>,
        checkpoint_name: &str,
        action_kind: &str,
    ) -> Result<usize> {
        let run_id = self
            .evidence
            .run_id
            .as_deref()
            .context("action evidence requires a valid invocation run ID")?;
        let sequence = self.evidence.actions.len() + 1;
        let phase_segment = phase_name.unwrap_or("null");
        let action_id = format!(
            "{run_id}/{case_id}/{phase_segment}/{checkpoint_name}/{sequence:04}"
        );
        self.evidence.actions.push(ActionEventEvidence {
            action_id,
            case_id: case_id.to_string(),
            phase_name: phase_name.map(str::to_string),
            checkpoint_name: checkpoint_name.to_string(),
            sequence,
            action_kind: action_kind.to_string(),
            attempted_at: utc_timestamp(),
            dispatch_completed_at: None,
            dispatch_state: ActionDispatchState::Attempted,
            observation_at: None,
            observation_state: None,
        });
        let index = self.evidence.actions.len() - 1;
        self.persist()?;
        Ok(index)
    }

    fn complete_action_dispatch(
        &mut self,
        index: usize,
        state: ActionDispatchState,
    ) -> Result<()> {
        let event = self
            .evidence
            .actions
            .get_mut(index)
            .context("action event disappeared before dispatch completion")?;
        event.dispatch_state = state;
        if state == ActionDispatchState::Dispatched {
            event.dispatch_completed_at = Some(utc_timestamp());
        }
        self.persist()
    }

    fn observe_checkpoint_actions(
        &mut self,
        phase_name: Option<&str>,
        checkpoint_name: &str,
        observation_state: &str,
    ) -> Result<()> {
        let observed_at = utc_timestamp();
        let mut changed = false;
        for event in &mut self.evidence.actions {
            if event.phase_name.as_deref() == phase_name
                && event.checkpoint_name == checkpoint_name
                && event.observation_state.is_none()
            {
                event.observation_at = Some(observed_at.clone());
                event.observation_state = Some(observation_state.to_string());
                changed = true;
            }
        }
        if changed {
            self.persist()?;
        }
        Ok(())
    }

    fn observe_exit(&mut self, phase_name: Option<&str>, matched: bool) -> Result<()> {
        let observed_at = utc_timestamp();
        let mut changed = false;
        for event in &mut self.evidence.actions {
            if event.phase_name.as_deref() == phase_name
                && event.checkpoint_name == "exit"
                && event.observation_state.is_none()
            {
                event.observation_at = Some(observed_at.clone());
                event.observation_state = Some(if matched {
                    "matched"
                } else {
                    "mismatched"
                }
                .to_string());
                changed = true;
            }
        }
        if changed {
            self.persist()?;
        }
        Ok(())
    }

    fn persist(&self) -> Result<()> {
        if !self.persist_to_disk {
            return Ok(());
        }
        let bytes = serde_json::to_vec_pretty(&self.evidence)
            .context("serialize structured case result")?;
        write_atomic(&self.root, &self.file_name, &bytes).context("write structured case result")
    }
}

fn with_action_writer<T>(
    local_writer: &mut CaseResultWriter,
    invocation_writer: &mut Option<&mut CaseResultWriter>,
    write: impl FnOnce(&mut CaseResultWriter) -> Result<T>,
) -> Result<T> {
    match invocation_writer.as_deref_mut() {
        Some(writer) => write(writer),
        None => write(local_writer),
    }
}

fn dispatch_recorded_action(
    local_writer: &mut CaseResultWriter,
    invocation_writer: &mut Option<&mut CaseResultWriter>,
    session: &Session,
    case_id: &str,
    phase_name: Option<&str>,
    checkpoint_name: &str,
    action: &NativeAction,
    suppressed_by_control: bool,
) -> Result<(Result<()>, Result<()>)> {
    let event_index = with_action_writer(local_writer, invocation_writer, |writer| {
        writer.begin_action(
            case_id,
            phase_name,
            checkpoint_name,
            native_action_kind(action),
        )
    })?;
    let dispatch_result = if suppressed_by_control {
        Ok(())
    } else {
        execute_native_action(session, action)
    };
    let dispatch_state = if suppressed_by_control {
        ActionDispatchState::SuppressedByControl
    } else if dispatch_result.is_ok() {
        ActionDispatchState::Dispatched
    } else {
        ActionDispatchState::Failed
    };
    let evidence_result = with_action_writer(local_writer, invocation_writer, |writer| {
        writer.complete_action_dispatch(event_index, dispatch_state)
    });
    Ok((dispatch_result, evidence_result))
}

fn checkpoint_observation_state(checkpoint: &CheckpointEvidence) -> &'static str {
    if checkpoint.comparison.passed
        && checkpoint
            .selection_observations
            .iter()
            .all(|observation| observation.matched)
    {
        "matched"
    } else {
        "mismatched"
    }
}

#[derive(Debug, Serialize)]
struct CaptureIndex {
    schema_version: u32,
    suite_id: String,
    case_name: String,
    case_id: String,
    test_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    motion: Option<String>,
    case_status: String,
    failure_kind: Option<CaseFailureKind>,
    comparison_failure_kind: Option<ComparisonFailureKind>,
    frame_checkpoint_count: usize,
    /// Relative to the directory containing this capture-index receipt.
    exit_result_file: Option<String>,
    executable_name: String,
    arguments: Vec<String>,
    terminal: TerminalEvidence,
    child_environment: BTreeMap<String, String>,
    no_color_present: bool,
    working_directory: String,
    fixture_dataset_id: String,
    coverage_scope: String,
    capture_api: String,
    evidence_roots: EvidenceRootsEvidence,
    raw_pty: Option<RawPtyEvidence>,
    raw_tail: Option<RawTailEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cleanup_selection: Option<CleanupSelectionEvidence>,
    drain_succeeded: bool,
    process_state_before_finalize: Option<ProcessStateEvidence>,
    process_state_after_finalize: Option<ProcessStateEvidence>,
    terminal_restoration: Option<TerminalRestorationEvidence>,
    checkpoints: Vec<CheckpointEvidence>,
    checkpoint_attempts: Vec<CheckpointAttemptEvidence>,
    not_reached_checkpoints: Vec<NotReachedCheckpointEvidence>,
    setup_errors: Vec<String>,
    application_errors: Vec<String>,
    capture_errors: Vec<String>,
    execution_errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DeliveryExitResult {
    schema_version: u32,
    suite_id: String,
    case_id: String,
    test_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    motion: Option<String>,
    case_status: String,
    failure_kind: Option<CaseFailureKind>,
    comparison_failure_kind: Option<ComparisonFailureKind>,
    exit_key: Option<String>,
    exit_key_attempted: bool,
    exit_key_error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cleanup_actions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cleanup_selection: Option<CleanupSelectionEvidence>,
    wait_exit_succeeded: bool,
    process_state_before_finalize: Option<ProcessStateEvidence>,
    drain_succeeded: bool,
    raw_pty: Option<RawPtyEvidence>,
    raw_tail: Option<RawTailEvidence>,
    process_state_after_finalize: Option<ProcessStateEvidence>,
    terminal_restoration: Option<TerminalRestorationEvidence>,
    setup_errors: Vec<String>,
    application_errors: Vec<String>,
    execution_errors: Vec<String>,
    capture_errors: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct CleanupSelectionEvidence {
    /// Relative to the directory containing the capture-index and exit-result receipts.
    snapshot_file: String,
    snapshot_sha256: String,
    raw_prefix_offset: usize,
    raw_prefix_sha256: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct RawTailEvidence {
    byte_offset: usize,
    byte_length: usize,
    sha256: String,
    hex: String,
}

#[derive(Debug, Serialize)]
struct TerminalEvidence {
    cols: u16,
    rows: u16,
    locale: String,
    timezone: String,
    term: String,
    color_term: String,
    render_profile: String,
    platform_key: String,
}

#[derive(Debug, Serialize, Clone)]
struct EvidenceRootsEvidence {
    expected_store_root: String,
    output_root: String,
    comparison_root: String,
    roots_disjoint: bool,
}

#[derive(Debug, Serialize, Clone)]
struct ProcessStateEvidence {
    exited: Option<i32>,
    exit_signal: Option<String>,
    modes: BTreeMap<String, bool>,
    mouse_mode: String,
}

#[derive(Debug, Serialize, Clone)]
struct TerminalRestorationEvidence {
    expected_modes: BTreeMap<String, bool>,
    expected_mouse_mode: String,
    restored: bool,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct RawPtyEvidence {
    byte_length: usize,
    sha256: String,
    maximum_retained_bytes: usize,
    /// Relative to the directory containing its capture-index or exit-result receipt.
    bytes_file: String,
    exact_bytes: bool,
    drain_reached_eof: bool,
}

#[derive(Debug, Serialize)]
struct CheckpointAttemptEvidence {
    name: String,
    reached: bool,
    capture_attempted: bool,
    outcome: String,
    reason: Option<String>,
    raw_prefix_offset: Option<usize>,
    raw_prefix_sha256: Option<String>,
    /// Relative to the directory containing the capture-index receipt.
    sample_file: Option<String>,
    sample_sha256: Option<String>,
    /// Relative to the directory containing the capture-index receipt.
    native_snapshot_file: Option<String>,
    native_snapshot_sha256: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct NotReachedCheckpointEvidence {
    name: String,
    status: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CheckpointEvidence {
    name: String,
    test_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    selection_observations: Vec<SelectionObservationEvidence>,
    source_revision: u64,
    raw_prefix_offset: usize,
    raw_prefix_sha256: Option<String>,
    /// Relative to the directory containing the capture-index receipt.
    sample_file: String,
    sample_sha256: String,
    /// Relative to the directory containing the capture-index receipt.
    native_snapshot_file: String,
    native_snapshot_sha256: String,
    cols: u32,
    rows: u32,
    cursor: CursorEvidence,
    resolved_cells: Vec<CellEvidence>,
    image: ImageEvidence,
    render_contract: RenderContractEvidence,
    comparison: ComparisonEvidence,
}

#[derive(Debug, Serialize)]
struct CursorEvidence {
    col: u32,
    row: u32,
    in_capture: bool,
    visible: bool,
    shape: String,
    blink: Option<bool>,
    color: String,
}

#[derive(Debug, Serialize)]
struct CellEvidence {
    col: u32,
    row: u32,
    text: String,
    width: u16,
    continuation: bool,
    foreground: String,
    background: String,
    bold: bool,
    dim: bool,
    italic: bool,
    inverse: bool,
    invisible: bool,
    strike: bool,
    blink: bool,
    underline: String,
    underline_color: String,
    link_uri: Option<String>,
    link_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ImageEvidence {
    width: u32,
    height: u32,
    png_sha256: String,
    rgba_sha256: String,
}

#[derive(Debug, Serialize)]
struct RenderContractEvidence {
    profile: String,
    renderer_version: String,
    raster_policy: String,
    fonts: Vec<FontEvidence>,
    palette_digest: String,
    cell_width: f32,
    cell_height: f32,
    font_size: f32,
    baseline: f32,
    scale: f64,
    cursor_phase: String,
    cursor_blink: Option<bool>,
}

#[derive(Debug, Serialize)]
struct FontEvidence {
    sha256: String,
    face_index: u32,
    bold: bool,
    italic: bool,
    fallback_position: usize,
}

#[derive(Debug, Serialize)]
struct ComparisonEvidence {
    test_id: String,
    gate: String,
    outcome: String,
    passed: bool,
    reason: Option<String>,
    failure_kind: Option<ComparisonFailureKind>,
    baseline_sha256: Option<String>,
    candidate_sha256: String,
    changed_fields: Vec<String>,
    first_changed_cell: Option<ChangedCellEvidence>,
    changed_cell_total: Option<usize>,
    pixel_comparison: String,
    pixel_changed: Option<u64>,
    pixel_compared: Option<u64>,
    pixel_bounds: Option<[u32; 4]>,
    report_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChangedCellEvidence {
    col: u32,
    row: u32,
    fields: Vec<String>,
    actual_text: String,
    baseline_text: String,
}

macro_rules! journey_test {
    ($test_name:ident) => {
        #[test]
        fn $test_name() -> Result<()> {
            run_case(stringify!($test_name))
        }
    };
}

journey_test!(first_showcase);
journey_test!(first_holla);
journey_test!(first_jackin);
journey_test!(first_tablepro);
journey_test!(showcase_overview);
journey_test!(showcase_chrome);
journey_test!(showcase_status_and_hints);
journey_test!(showcase_breadcrumbs_and_rails);
journey_test!(showcase_help_and_keys);
journey_test!(showcase_quit_and_leave);
journey_test!(showcase_focus_and_navigation);
journey_test!(showcase_buttons);
journey_test!(showcase_inputs);
journey_test!(showcase_text_areas);
journey_test!(showcase_chips_and_selects);
journey_test!(showcase_forms);
journey_test!(showcase_checks_and_radios);
journey_test!(showcase_toolbars);
journey_test!(showcase_basic_table);
journey_test!(showcase_editable);
journey_test!(showcase_data_grid);
journey_test!(showcase_selection);
journey_test!(showcase_cell_editors);
journey_test!(showcase_columns);
journey_test!(showcase_lists);
journey_test!(showcase_trees);
journey_test!(showcase_sidebars);
journey_test!(showcase_tabs);
journey_test!(showcase_menus);
journey_test!(showcase_dialogs);
journey_test!(showcase_pickers);
journey_test!(showcase_dialog_kinds);
journey_test!(showcase_popups_and_tooltips);
journey_test!(showcase_file_browser);
journey_test!(showcase_progress);
journey_test!(showcase_empty_and_error);
journey_test!(showcase_meters);
journey_test!(showcase_notices);
journey_test!(showcase_panels);
journey_test!(showcase_scrolling);
journey_test!(showcase_splits_and_stacks);
journey_test!(showcase_master_detail);
journey_test!(showcase_pane_tree);
journey_test!(showcase_code_editor);
journey_test!(showcase_diff);
journey_test!(showcase_terminal);
journey_test!(showcase_text_utilities);
journey_test!(showcase_settings);
journey_test!(showcase_task_runner);
journey_test!(showcase_query_workbench);
journey_test!(showcase_connections);
journey_test!(showcase_finder);
journey_test!(showcase_disk_cleanup);
journey_test!(showcase_activity);
journey_test!(showcase_workspaces);
journey_test!(showcase_accounts_and_usage);
journey_test!(showcase_launch_cockpit);
journey_test!(showcase_read_only_report);
journey_test!(showcase_explorer);
journey_test!(showcase_orders);
journey_test!(showcase_query_edit_while_running);
journey_test!(showcase_query_tab_switching);
journey_test!(holla_home_actions);
journey_test!(holla_plan_args_gate);
journey_test!(holla_activity_stop);
journey_test!(holla_disk_cleanup_review);
journey_test!(holla_files_clone);
journey_test!(holla_taskrun_args_cancel);
journey_test!(holla_database_blockers);
journey_test!(jackin_intro_prelude);
journey_test!(jackin_editor_save_cancel_leave);
journey_test!(jackin_accounts_usage);
journey_test!(jackin_settings);
journey_test!(jackin_launch_cancel_failure);
journey_test!(jackin_handoff_capsule);
journey_test!(jackin_capsule_controls);
journey_test!(jackin_protected_quit_outro);
journey_test!(tablepro_connection_retry);
journey_test!(tablepro_explorer_structure_grid);
journey_test!(tablepro_edit_transaction);
journey_test!(tablepro_query_execution);
journey_test!(tablepro_safety_ack);
journey_test!(tablepro_history_filter_reconnect);
journey_test!(tablepro_switchers_dirty_close);
journey_test!(first_delivery_showcase_buttons);
journey_test!(first_delivery_holla_first_use);
journey_test!(first_delivery_jackin_settings);
journey_test!(first_delivery_tablepro_connections);
journey_test!(full_holla_first_use_frame1);

fn run_case(case_name: &str) -> Result<()> {
    let output_root = required_directory("TERMROCK_PARITY_OUT_DIR")?;
    ensure_output_disjoint_from_expected(&output_root)?;
    let mut result = CaseResultWriter::create(&output_root, case_name)?;
    let run_result = run_case_with_result(case_name, &output_root, &mut result);
    match run_result {
        Ok(()) => {
            result.evidence.case_ended_at = Some(utc_timestamp());
            result.evidence.status = CaseResultStatus::Passed;
            result.evidence.failure_kind = None;
            result.evidence.error = None;
            result.evidence.stage = CaseStage::Finalize;
            result.persist()?;
            Ok(())
        }
        Err(error) => {
            result.evidence.case_ended_at = Some(utc_timestamp());
            result.evidence.status = CaseResultStatus::Failed;
            if result.evidence.failure_kind.is_none() {
                result.evidence.failure_kind = Some(if result.evidence.launched {
                    CaseFailureKind::Execution
                } else {
                    CaseFailureKind::Setup
                });
            }
            result.evidence.error = Some(format!("{error:#}"));
            if !result.evidence.launched {
                let reason = result.evidence.error.clone().unwrap_or_default();
                for checkpoint in &mut result.evidence.not_run_checkpoints {
                    checkpoint.reason.clone_from(&reason);
                }
                for phase in &mut result.evidence.phases {
                    if phase.status == PhaseRunStatus::NotRun {
                        phase.status = PhaseRunStatus::SetupFailed;
                    }
                    phase.error.get_or_insert_with(|| reason.clone());
                }
            }
            result.persist().context("persist failed case result")?;
            Err(error)
        }
    }
}

fn run_case_with_result(
    case_name: &str,
    output_root: &Path,
    result: &mut CaseResultWriter,
) -> Result<()> {
    result.evidence.stage = CaseStage::Manifest;
    let manifest: SuiteManifest =
        toml::from_str(MANIFEST_TOML).context("parse TOML suite manifest")?;
    result.evidence.suite_id = Some(manifest.version.suite_id.clone());
    let journey = manifest
        .cases
        .iter()
        .find(|case| case.name == case_name)
        .with_context(|| format!("manifest has no case named {case_name}"))?;
    ensure!(
        result.evidence.run_id.is_some(),
        "setup failure: TERMROCK_PARITY_RUN_ID must be a safe non-empty invocation ID"
    );
    result.evidence.engine_pins = manifest.engine_pins.clone();
    validate_fault_control(&result.evidence.fault_control, journey)?;
    result.evidence.case_id = Some(case_id(journey).to_string());
    result.evidence.planned_checkpoint_names = checkpoint_names(journey);
    result.evidence.not_run_checkpoints = result
        .evidence
        .planned_checkpoint_names
        .iter()
        .map(|name| NotReachedCheckpointEvidence {
            name: name.clone(),
            status: "not-run".to_string(),
            reason: "case has not reached this checkpoint".to_string(),
        })
        .collect();
    result.evidence.phases = journey
        .phases
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(phase_result_evidence)
        .collect();
    result
        .persist()
        .context("persist full planned case inventory before launch")?;
    let dataset: FixtureDataset =
        serde_json::from_str(DATASET_JSON).context("parse fixture dataset")?;
    validate_manifest(&manifest, &dataset)?;
    result.evidence.stage = CaseStage::Plan;
    ensure_supported_plan(journey)?;
    let case_id = case_id(journey);

    result.evidence.stage = CaseStage::Fixture;
    let binary_dir = required_directory("TERMROCK_PARITY_BIN_DIR")?;
    let binary_path = binary_dir
        .join(&journey.binary)
        .canonicalize()
        .with_context(|| {
            format!(
                "resolve application binary {}",
                binary_dir.join(&journey.binary).display()
            )
        })?;
    ensure!(
        binary_path.is_file(),
        "application binary is not a file: {}",
        binary_path.display()
    );

    if let Some(phases) = journey.phases.as_deref() {
        return run_phased_case_with_result(
            journey,
            phases,
            &manifest,
            &dataset,
            &binary_path,
            output_root,
            case_name,
            result,
        );
    }

    let fixture = prepare_fixture(&dataset, journey, &manifest)?;
    let output_dir = prepare_case_output(output_root, case_id)?;
    let samples_dir = ensure_child_dir(&output_dir, "samples")?;
    let compare_dir = output_dir.join("comparison");
    let expected_store = safe_manifest_path(&manifest.suite.expected_store)?;
    let expected_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(expected_store)
        .canonicalize()
        .context("canonicalize expected frame store")?;
    ensure_disjoint_roots(&expected_root, output_root)?;
    let store =
        FrozenStore::new(&expected_root, &compare_dir).context("open expected frame store")?;
    let comparison_root = compare_dir
        .canonicalize()
        .context("canonicalize comparison artifact root")?;
    ensure!(
        comparison_root.starts_with(output_root),
        "comparison artifact root escaped TERMROCK_PARITY_OUT_DIR"
    );
    ensure_disjoint_roots(&expected_root, &comparison_root)?;
    let evidence_roots = EvidenceRootsEvidence {
        expected_store_root: expected_root.to_string_lossy().into_owned(),
        output_root: output_root.to_string_lossy().into_owned(),
        comparison_root: comparison_root.to_string_lossy().into_owned(),
        roots_disjoint: true,
    };
    let pixel_profile =
        resolve_profile(&manifest.suite.render_profile).context("resolve pinned pixel profile")?;

    result.evidence.stage = CaseStage::Launch;
    let (session, args, child_environment) =
        spawn_application(journey, None, &binary_path, &fixture, &manifest)?;
    result.evidence.launched = true;
    result.evidence.stage = CaseStage::Journey;

    let is_delivery = journey.steps.is_some();
    let generated_legacy_plans;
    let plans = if let Some(plans) = journey.steps.as_deref() {
        plans
    } else {
        generated_legacy_plans = legacy_checkpoint_plans(journey)?;
        &generated_legacy_plans
    };
    let journey_result = run_unified_journey(
        &session,
        journey,
        plans,
        is_delivery,
        &manifest,
        pixel_profile,
        &store,
        &evidence_roots,
        &samples_dir,
        &output_dir,
        &child_environment,
        &fixture,
        &dataset,
        &args,
        case_name,
        result,
        None,
        None,
        None,
    );
    let close_result = session
        .close()
        .context("close native session after journey");
    result.evidence.stage = CaseStage::Finalize;
    if let Err(close_error) = &close_result {
        if output_dir.join("capture-index.json").is_file() {
            append_capture_index_execution_error(
                &output_dir,
                &format!("checked session close failed: {close_error:#}"),
            )
            .context("record session close failure in capture index")?;
            append_exit_result_execution_error(
                &output_dir,
                &format!("checked session close failed: {close_error:#}"),
            )
            .context("record session close failure in exit result")?;
        }
    }
    match (journey_result, close_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(journey_error), Ok(())) => Err(journey_error),
        (Ok(()), Err(close_error)) => Err(close_error),
        (Err(journey_error), Err(close_error)) => Err(anyhow::anyhow!(
            "journey failed: {journey_error:#}; session cleanup also failed: {close_error:#}"
        )),
    }
}

fn spawn_application(
    journey: &Journey,
    phase: Option<&JourneyPhase>,
    binary_path: &Path,
    fixture: &FixturePaths,
    manifest: &SuiteManifest,
) -> Result<(Session, Vec<String>, BTreeMap<String, String>)> {
    let mut args = journey.arguments.clone();
    let mut motion_arguments = manifest.suite.terminal.motion_arguments.clone();
    if let Some(phase) = phase {
        let motion_index = motion_arguments
            .iter()
            .position(|argument| argument == "--motion")
            .context("suite motion arguments lack a motion value")?;
        let motion_value = motion_arguments
            .get_mut(motion_index + 1)
            .context("suite motion argument lacks a value")?;
        *motion_value = phase.motion.clone();
    }
    if let Some(frame) = journey.frame {
        let frame_index = motion_arguments
            .iter()
            .position(|argument| argument == "--frame")
            .context("suite motion arguments lack a frame value")?;
        let frame_value = motion_arguments
            .get_mut(frame_index + 1)
            .context("suite frame argument lacks a value")?;
        *frame_value = frame.to_string();
    }
    args.extend(motion_arguments);
    args.extend(manifest.suite.terminal.color_mode_arguments.iter().cloned());
    let child_environment = fixture.env.iter().cloned().collect::<BTreeMap<_, _>>();
    let launch = LaunchOptions::new(binary_path.as_os_str())
        .with_args(args.iter().cloned())
        .with_cwd(&fixture.cwd)
        .with_env_mode(LaunchEnvMode::Cleared)
        .with_remove_env("NO_COLOR")
        .with_envs(fixture.env.iter().cloned())
        .with_raw_pty_capture();
    let resolved_launch = launch
        .resolve()
        .context("resolve deterministic native launch")?;
    ensure!(
        !resolved_launch
            .env()
            .iter()
            .any(|(key, _)| key.as_os_str() == std::ffi::OsStr::new("NO_COLOR")),
        "NO_COLOR remained in the resolved child environment"
    );
    ensure!(
        resolved_launch.captures_raw_pty(),
        "exact raw PTY capture is disabled"
    );

    let run_options = RunOptions {
        backend: Backend::Alacritty,
        program: binary_path.to_string_lossy().into_owned(),
        args: args.clone(),
        profile: Profile::default(),
        cols: manifest.suite.terminal.cols,
        rows: manifest.suite.terminal.rows,
        cwd: Some(fixture.cwd.to_string_lossy().into_owned()),
        env: fixture.env.clone(),
        wait_ready: Some(false),
        restart: false,
        timeouts: Timeouts::default(),
        recording: AutomaticRecording::default(),
        style: Style::default(),
    };
    let session_name = phase.map_or_else(
        || format!("termrock-functional-{}", case_id(journey)),
        |phase| format!("termrock-functional-{}-{}", case_id(journey), phase.name),
    );
    let session = Session::new(session_name);
    session
        .spawn_with(launch, run_options)
        .context("spawn native application")?;
    Ok((session, args, child_environment))
}

fn phase_result_evidence(phase: &JourneyPhase) -> PhaseResultEvidence {
    PhaseResultEvidence {
        name: phase.name.clone(),
        motion: phase.motion.clone(),
        cleanup: phase.cleanup.clone(),
        planned_checkpoint_names: phase.steps.iter().map(|step| step.name.clone()).collect(),
        status: PhaseRunStatus::NotRun,
        comparison_failure_kind: None,
        launched: false,
        cleanup_actions_attempted: Vec::new(),
        wait_exit_succeeded: None,
        exit_status: None,
        exit_status_zero: None,
        exit_signal: None,
        no_signal: None,
        eof_drained: None,
        post_drain_exit_confirmed: None,
        raw_pty_sha256: None,
        raw_checkpoint_prefixes: BTreeMap::new(),
        terminal_restored: None,
        cleanup_snapshot_file: None,
        cleanup_snapshot_sha256: None,
        cleanup_raw_prefix_sha256: None,
        error: None,
    }
}

fn phase_case_result_writer(
    output_dir: &Path,
    journey: &Journey,
    manifest: &SuiteManifest,
    phase: &JourneyPhase,
) -> Result<CaseResultWriter> {
    let planned_checkpoint_names = phase
        .steps
        .iter()
        .map(|step| step.name.clone())
        .collect::<Vec<_>>();
    let writer = CaseResultWriter {
        root: output_dir.to_path_buf(),
        file_name: "phase-result.json".to_string(),
        persist_to_disk: false,
        evidence: CaseResultEvidence {
            schema_version: 2,
            suite_id: Some(manifest.version.suite_id.clone()),
            run_id: env::var("TERMROCK_PARITY_RUN_ID")
                .ok()
                .and_then(|value| validate_run_id(&value)),
            case_started_at: utc_timestamp(),
            case_ended_at: None,
            test_name: journey.name.clone(),
            case_id: Some(case_id(journey).to_string()),
            engine_pins: manifest.engine_pins.clone(),
            fault_control: fault_control_evidence_from_env(),
            actions: Vec::new(),
            planned_checkpoint_names: planned_checkpoint_names.clone(),
            not_run_checkpoints: planned_checkpoint_names
                .iter()
                .map(|name| NotReachedCheckpointEvidence {
                    name: name.clone(),
                    status: "not-run".to_string(),
                    reason: "phase has not reached this checkpoint".to_string(),
                })
                .collect(),
            phases: Vec::new(),
            status: CaseResultStatus::Pending,
            failure_kind: None,
            comparison_failure_kind: None,
            stage: CaseStage::Fixture,
            launched: false,
            error: None,
        },
    };
    writer.persist()?;
    Ok(writer)
}

fn run_phased_case_with_result(
    journey: &Journey,
    phases: &[JourneyPhase],
    manifest: &SuiteManifest,
    dataset: &FixtureDataset,
    binary_path: &Path,
    output_root: &Path,
    case_name: &str,
    result: &mut CaseResultWriter,
) -> Result<()> {
    let output_dir = prepare_case_output(output_root, case_id(journey))?;
    let compare_dir = output_dir.join("comparison");
    let expected_store = safe_manifest_path(&manifest.suite.expected_store)?;
    let expected_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(expected_store)
        .canonicalize()
        .context("canonicalize expected frame store")?;
    ensure_disjoint_roots(&expected_root, output_root)?;
    let store =
        FrozenStore::new(&expected_root, &compare_dir).context("open expected frame store")?;
    let comparison_root = compare_dir
        .canonicalize()
        .context("canonicalize comparison artifact root")?;
    ensure!(
        comparison_root.starts_with(output_root),
        "comparison artifact root escaped TERMROCK_PARITY_OUT_DIR"
    );
    ensure_disjoint_roots(&expected_root, &comparison_root)?;
    let evidence_roots = EvidenceRootsEvidence {
        expected_store_root: expected_root.to_string_lossy().into_owned(),
        output_root: output_root.to_string_lossy().into_owned(),
        comparison_root: comparison_root.to_string_lossy().into_owned(),
        roots_disjoint: true,
    };
    let pixel_profile =
        resolve_profile(&manifest.suite.render_profile).context("resolve pinned pixel profile")?;
    let phase_root = ensure_child_dir(&output_dir, "phases")?;
    let mut completed_not_run = Vec::new();
    let mut failures = Vec::new();

    for (phase_index, phase) in phases.iter().enumerate() {
        let phase_output_dir = match ensure_child_dir(&phase_root, &phase.name) {
            Ok(path) => path,
            Err(error) => {
                let message = format!("prepare phase output failed: {error:#}");
                result
                    .evidence
                    .failure_kind
                    .get_or_insert(CaseFailureKind::Setup);
                result.evidence.phases[phase_index].status = PhaseRunStatus::SetupFailed;
                result.evidence.phases[phase_index].error = Some(message.clone());
                completed_not_run.extend(phase.steps.iter().map(|step| {
                    NotReachedCheckpointEvidence {
                        name: step.name.clone(),
                        status: "not-run".to_string(),
                        reason: message.clone(),
                    }
                }));
                failures.push(format!("{}: {message}", phase.name));
                update_phased_not_run(result, &completed_not_run, phases, phase_index + 1)?;
                continue;
            }
        };
        let mut phase_writer =
            match phase_case_result_writer(&phase_output_dir, journey, manifest, phase) {
                Ok(writer) => writer,
                Err(error) => {
                    let message = format!("prepare phase result failed: {error:#}");
                    result
                        .evidence
                        .failure_kind
                        .get_or_insert(CaseFailureKind::Setup);
                    result.evidence.phases[phase_index].status = PhaseRunStatus::SetupFailed;
                    result.evidence.phases[phase_index].error = Some(message.clone());
                    completed_not_run.extend(phase.steps.iter().map(|step| {
                        NotReachedCheckpointEvidence {
                            name: step.name.clone(),
                            status: "not-run".to_string(),
                            reason: message.clone(),
                        }
                    }));
                    failures.push(format!("{}: {message}", phase.name));
                    update_phased_not_run(result, &completed_not_run, phases, phase_index + 1)?;
                    continue;
                }
            };
        let phase_samples_dir = match ensure_child_dir(&phase_output_dir, "samples") {
            Ok(path) => path,
            Err(error) => {
                let message = format!("prepare phase samples failed: {error:#}");
                mark_phase_setup_failure(result, phase_index, &mut phase_writer, &message)?;
                completed_not_run.extend(phase_writer.evidence.not_run_checkpoints.clone());
                failures.push(format!("{}: {message}", phase.name));
                update_phased_not_run(result, &completed_not_run, phases, phase_index + 1)?;
                continue;
            }
        };
        let fixture = match prepare_fixture(dataset, journey, manifest) {
            Ok(fixture) => fixture,
            Err(error) => {
                let message = format!("prepare phase fixture failed: {error:#}");
                mark_phase_setup_failure(result, phase_index, &mut phase_writer, &message)?;
                completed_not_run.extend(phase_writer.evidence.not_run_checkpoints.clone());
                failures.push(format!("{}: {message}", phase.name));
                update_phased_not_run(result, &completed_not_run, phases, phase_index + 1)?;
                continue;
            }
        };

        result.evidence.stage = CaseStage::Launch;
        let (session, args, child_environment) =
            match spawn_application(journey, Some(phase), binary_path, &fixture, manifest) {
                Ok(child) => child,
                Err(error) => {
                    let message = format!("prepare or spawn phase child failed: {error:#}");
                    mark_phase_setup_failure(result, phase_index, &mut phase_writer, &message)?;
                    completed_not_run.extend(phase_writer.evidence.not_run_checkpoints.clone());
                    failures.push(format!("{}: {message}", phase.name));
                    update_phased_not_run(result, &completed_not_run, phases, phase_index + 1)?;
                    continue;
                }
            };
        result.evidence.launched = true;
        result.evidence.stage = CaseStage::Journey;
        result.evidence.phases[phase_index].status = PhaseRunStatus::Launched;
        result.evidence.phases[phase_index].launched = true;
        phase_writer.evidence.launched = true;
        phase_writer.evidence.stage = CaseStage::Journey;
        if let Err(error) = result.persist() {
            let message = format!("persist launched phase receipt failed: {error:#}");
            result
                .evidence
                .failure_kind
                .get_or_insert(CaseFailureKind::Capture);
            result.evidence.error = Some(message.clone());
            failures.push(format!("{}: {message}", phase.name));
        }

        let journey_result = run_unified_journey(
            &session,
            journey,
            &phase.steps,
            true,
            manifest,
            pixel_profile,
            &store,
            &evidence_roots,
            &phase_samples_dir,
            &phase_output_dir,
            &child_environment,
            &fixture,
            dataset,
            &args,
            case_name,
            &mut phase_writer,
            Some(phase),
            Some(&mut *result),
            Some(phase_index),
        );
        let close_result = session
            .close()
            .context("close native session after phased journey");
        if let Some(comparison_failure_kind) = phase_writer.evidence.comparison_failure_kind {
            result.evidence.phases[phase_index].comparison_failure_kind =
                Some(comparison_failure_kind);
            result
                .evidence
                .comparison_failure_kind
                .get_or_insert(comparison_failure_kind);
        }
        let close_succeeded = close_result.is_ok();
        if let Err(close_error) = &close_result {
            let message = format!("checked session close failed: {close_error:#}");
            if phase_output_dir.join("capture-index.json").is_file() {
                if let Err(error) =
                    append_capture_index_execution_error(&phase_output_dir, &message)
                {
                    failures.push(format!(
                        "{}: record close failure in capture index: {error:#}",
                        phase.name
                    ));
                }
                if let Err(error) = append_exit_result_execution_error(&phase_output_dir, &message)
                {
                    failures.push(format!(
                        "{}: record close failure in exit result: {error:#}",
                        phase.name
                    ));
                }
            }
            phase_writer.evidence.status = CaseResultStatus::Failed;
            phase_writer.evidence.failure_kind = Some(CaseFailureKind::Execution);
            phase_writer.evidence.error = Some(message.clone());
            result.evidence.phases[phase_index].status = PhaseRunStatus::Failed;
            result.evidence.phases[phase_index].error = Some(message.clone());
            result
                .evidence
                .failure_kind
                .get_or_insert(CaseFailureKind::Execution);
            if let Err(error) = phase_writer.persist() {
                failures.push(format!("{}: persist close failure: {error:#}", phase.name));
            }
        }

        let phase_error = match (journey_result, close_result) {
            (Ok(()), Ok(())) if phase_writer.evidence.status == CaseResultStatus::Passed => None,
            (Err(journey_error), Ok(())) => Some(format!("{journey_error:#}")),
            (Ok(()), Err(close_error)) => Some(format!("{close_error:#}")),
            (Err(journey_error), Err(close_error)) => Some(format!(
                "journey failed: {journey_error:#}; session cleanup also failed: {close_error:#}"
            )),
            (Ok(()), Ok(())) => Some("phase result did not reach passed status".to_string()),
        };
        if let Some(message) = phase_error {
            result.evidence.phases[phase_index].status = PhaseRunStatus::Failed;
            result.evidence.phases[phase_index]
                .error
                .get_or_insert_with(|| message.clone());
            result.evidence.failure_kind.get_or_insert_with(|| {
                phase_writer
                    .evidence
                    .failure_kind
                    .unwrap_or(CaseFailureKind::Execution)
            });
            failures.push(format!("{}: {message}", phase.name));
        } else {
            result.evidence.phases[phase_index].status = PhaseRunStatus::Passed;
        }
        if let Err(error) = phase_writer.persist() {
            let message = format!("persist phase result failed: {error:#}");
            result.evidence.phases[phase_index].status = PhaseRunStatus::Failed;
            result.evidence.phases[phase_index].error = Some(message.clone());
            result
                .evidence
                .failure_kind
                .get_or_insert(CaseFailureKind::Capture);
            failures.push(format!("{}: {message}", phase.name));
        }
        completed_not_run.extend(phase_writer.evidence.not_run_checkpoints.clone());
        update_phased_not_run(result, &completed_not_run, phases, phase_index + 1)?;
        let phase_lifecycle = &result.evidence.phases[phase_index];
        let lifecycle_complete = close_succeeded
            && phase_lifecycle.wait_exit_succeeded == Some(true)
            && phase_lifecycle.exit_status == Some(0)
            && phase_lifecycle.exit_status_zero == Some(true)
            && phase_lifecycle.exit_signal.is_none()
            && phase_lifecycle.no_signal == Some(true)
            && phase_lifecycle.eof_drained == Some(true)
            && phase_lifecycle.post_drain_exit_confirmed == Some(true)
            && phase_lifecycle.terminal_restored == Some(true);
        if !lifecycle_complete && phase_index + 1 < phases.len() {
            let reason = format!(
                "phase {} was not launched because the preceding child did not complete required lifecycle checks",
                phases[phase_index + 1].name
            );
            for later_phase in &mut result.evidence.phases[phase_index + 1..] {
                later_phase.status = PhaseRunStatus::NotRun;
                later_phase.error = Some(reason.clone());
            }
            for later_phase in phases.iter().skip(phase_index + 1) {
                completed_not_run.extend(later_phase.steps.iter().map(|step| {
                    NotReachedCheckpointEvidence {
                        name: step.name.clone(),
                        status: "not-run".to_string(),
                        reason: reason.clone(),
                    }
                }));
            }
            result.evidence.not_run_checkpoints = completed_not_run.clone();
            result.persist()?;
            break;
        }
    }

    result.evidence.stage = CaseStage::Finalize;
    if !failures.is_empty() {
        bail!("phased case failed: {}", failures.join("; "));
    }
    ensure!(
        result.evidence.phases.len() == phases.len()
            && result
                .evidence
                .phases
                .iter()
                .all(|phase| phase.status == PhaseRunStatus::Passed && phase.launched),
        "phased case lacks a passed, launched result for every planned phase"
    );
    ensure!(
        result.evidence.planned_checkpoint_names.len() == 16
            && result.evidence.not_run_checkpoints.is_empty(),
        "phased case did not reach its complete checkpoint inventory"
    );
    Ok(())
}

fn mark_phase_setup_failure(
    result: &mut CaseResultWriter,
    phase_index: usize,
    phase_writer: &mut CaseResultWriter,
    message: &str,
) -> Result<()> {
    result
        .evidence
        .failure_kind
        .get_or_insert(CaseFailureKind::Setup);
    result.evidence.phases[phase_index].status = PhaseRunStatus::SetupFailed;
    result.evidence.phases[phase_index].error = Some(message.to_string());
    phase_writer.evidence.status = CaseResultStatus::Failed;
    phase_writer.evidence.failure_kind = Some(CaseFailureKind::Setup);
    phase_writer.evidence.error = Some(message.to_string());
    for checkpoint in &mut phase_writer.evidence.not_run_checkpoints {
        checkpoint.reason = message.to_string();
    }
    phase_writer.persist()
}

fn update_phased_not_run(
    result: &mut CaseResultWriter,
    completed_not_run: &[NotReachedCheckpointEvidence],
    phases: &[JourneyPhase],
    next_phase_index: usize,
) -> Result<()> {
    let mut not_run = completed_not_run.to_vec();
    for phase in phases.iter().skip(next_phase_index) {
        not_run.extend(phase.steps.iter().map(|step| NotReachedCheckpointEvidence {
            name: step.name.clone(),
            status: "not-run".to_string(),
            reason: format!("phase {} has not been attempted", phase.name),
        }));
    }
    result.evidence.not_run_checkpoints = not_run;
    result.persist()
}

fn append_capture_index_execution_error(output_dir: &Path, message: &str) -> Result<()> {
    append_json_array_error(
        output_dir,
        "capture-index.json",
        "execution_errors",
        message,
    )
}

fn append_exit_result_execution_error(output_dir: &Path, message: &str) -> Result<()> {
    let exit_result = output_dir.join("exit-result.json");
    if !exit_result.exists() {
        return Ok(());
    }
    append_json_array_error(output_dir, "exit-result.json", "execution_errors", message)
}

fn append_json_array_error(
    output_dir: &Path,
    file_name: &str,
    array_name: &str,
    message: &str,
) -> Result<()> {
    let index_path = output_dir.join(file_name);
    ensure_direct_child(output_dir, &index_path)?;
    let metadata = fs::symlink_metadata(&index_path).context("inspect JSON evidence file")?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_file(),
        "JSON evidence path is not a regular file: {}",
        index_path.display()
    );
    let bytes = fs::read(&index_path).context("read JSON evidence for cleanup update")?;
    let mut index: serde_json::Value =
        serde_json::from_slice(&bytes).context("parse JSON evidence for cleanup update")?;
    if index.get("case_status").and_then(serde_json::Value::as_str) == Some("passed") {
        index["case_status"] = serde_json::Value::String("failed/execution".to_string());
        if index.get("failure_kind").is_some() {
            index["failure_kind"] = serde_json::Value::String("execution".to_string());
        }
    }
    let errors = index
        .get_mut(array_name)
        .and_then(serde_json::Value::as_array_mut)
        .with_context(|| format!("{file_name} lacks {array_name} array"))?;
    errors.push(serde_json::Value::String(message.to_string()));
    let updated = serde_json::to_vec_pretty(&index).context("serialize updated JSON evidence")?;
    write_atomic(output_dir, file_name, &updated)
}

fn ensure_supported_plan(journey: &Journey) -> Result<()> {
    if let Some(phases) = journey.phases.as_ref() {
        ensure!(
            journey.steps.is_none() && journey.native_steps.is_none(),
            "setup failure: phased case {} also declares a single-phase plan",
            journey.name
        );
        ensure!(
            phases.len() == 2
                && phases[0].name == "paused_pending"
                && phases[0].motion == "paused"
                && phases[1].name == "full_completion"
                && phases[1].motion == "full",
            "setup failure: phased case {} must declare paused_pending then full_completion",
            journey.name
        );
        let mut phase_names = BTreeSet::new();
        let mut checkpoint_names = BTreeSet::new();
        for phase in phases {
            ensure!(
                is_safe_single_component(&phase.name)
                    && phase_names.insert(phase.name.as_str())
                    && phase.cleanup == "ctrl_q_right_enter"
                    && !phase.steps.is_empty(),
                "setup failure: case {} has an invalid phase {:?}",
                journey.name,
                phase.name
            );
            for step in &phase.steps {
                ensure!(
                    is_safe_single_component(&step.name)
                        && checkpoint_names.insert(step.name.as_str())
                        && !step.required_anchors.is_empty(),
                    "setup failure: case {} has an invalid phase checkpoint {:?}",
                    journey.name,
                    step.name
                );
                for action in &step.actions {
                    match action {
                        NativeAction::Key { key } => ensure!(
                            !key.is_empty(),
                            "setup failure: case {} has an empty phase key",
                            journey.name
                        ),
                        NativeAction::Write { text } => ensure!(
                            !text.is_empty(),
                            "setup failure: case {} has an empty phase text action",
                            journey.name
                        ),
                        NativeAction::Resize { cols, rows } => ensure!(
                            matches!((*cols, *rows), (80, 24) | (120, 40)),
                            "setup failure: case {} has an invalid phase resize",
                            journey.name
                        ),
                        NativeAction::MouseClick { x, y, button } => ensure!(
                            *x == 30 && *y == 10 && button == "left",
                            "setup failure: case {} has an unsupported phase mouse click",
                            journey.name
                        ),
                    }
                }
            }
            let dialog = phase
                .steps
                .last()
                .context("phase lacks its final quit-dialog checkpoint")?;
            ensure!(
                dialog.name == phase.dialog_checkpoint
                    && matches!(dialog.actions.as_slice(), [NativeAction::Key { key }] if key == "Ctrl+Q"),
                "setup failure: case {} has an invalid phase quit-dialog contract",
                journey.name
            );
        }
        ensure!(
            checkpoint_names.len() == 16,
            "setup failure: phased case {} must declare all 16 checkpoints",
            journey.name
        );
        return Ok(());
    }
    if let Some(steps) = journey.steps.as_ref() {
        ensure!(
            !steps.is_empty(),
            "setup failure: delivery case {} has no checkpoint steps",
            journey.name
        );
        ensure!(
            journey.exit_key.is_some(),
            "setup failure: delivery case {} is missing its exit key",
            journey.name
        );
        let mut names = BTreeSet::new();
        for step in steps {
            ensure!(
                is_safe_single_component(&step.name)
                    && names.insert(step.name.as_str())
                    && !step.required_anchors.is_empty(),
                "setup failure: delivery case {} has an invalid checkpoint plan {:?}",
                journey.name,
                step.name
            );
            for action in &step.actions {
                match action {
                    NativeAction::Key { key } => ensure!(
                        !key.is_empty(),
                        "setup failure: delivery case {} has an empty key",
                        journey.name
                    ),
                    NativeAction::Write { text } => ensure!(
                        !text.is_empty(),
                        "setup failure: delivery case {} has an empty text action",
                        journey.name
                    ),
                    NativeAction::Resize { cols, rows } => ensure!(
                        matches!((*cols, *rows), (80, 24) | (120, 40)),
                        "setup failure: delivery case {} has an invalid resize",
                        journey.name
                    ),
                    NativeAction::MouseClick { x, y, button } => ensure!(
                        *x == 30 && *y == 10 && button == "left",
                        "setup failure: delivery case {} has an unsupported mouse click",
                        journey.name
                    ),
                }
            }
        }
        return Ok(());
    }
    let Some(native_steps) = journey.native_steps.as_deref() else {
        bail!(
            "setup failure: case {} is registered for coverage {:?}, but has no native_steps plan",
            journey.name,
            journey.coverage_ids
        );
    };
    ensure!(
        same_strings(native_steps, &["help", "return", "exit"]),
        "setup failure: case {} has a native plan not implemented by this runner: {:?}",
        journey.name,
        native_steps
    );
    ensure!(
        journey.help_key.is_some() && journey.return_key.is_some() && journey.exit_key.is_some(),
        "setup failure: case {} is missing its help, return, or exit key",
        journey.name
    );
    Ok(())
}

fn legacy_checkpoint_plans(journey: &Journey) -> Result<Vec<CheckpointPlan>> {
    ["opened", "help", "returned", "exited"]
        .into_iter()
        .map(|name| {
            let required_anchors = if name == "exited" {
                Vec::new()
            } else {
                required_checkpoint_anchors(&journey.name, name)?
                    .iter()
                    .map(|anchor| (*anchor).to_string())
                    .collect()
            };
            let forbidden_anchors = if name == "exited" {
                Vec::new()
            } else {
                forbidden_checkpoint_anchors(&journey.name, name)?
                    .iter()
                    .map(|anchor| (*anchor).to_string())
                    .collect()
            };
            let actions = match name {
                "help" => vec![NativeAction::Key {
                    key: journey
                        .help_key
                        .clone()
                        .context("validated legacy help key disappeared")?,
                }],
                "returned" => vec![NativeAction::Key {
                    key: journey
                        .return_key
                        .clone()
                        .context("validated legacy return key disappeared")?,
                }],
                _ => Vec::new(),
            };
            Ok(CheckpointPlan {
                name: name.to_string(),
                required_anchors,
                forbidden_anchors,
                expected_cols: 120,
                expected_rows: 40,
                actions,
                selection_observations: Vec::new(),
            })
        })
        .collect()
}

fn checkpoint_names(journey: &Journey) -> Vec<String> {
    if let Some(phases) = journey.phases.as_deref() {
        return phases
            .iter()
            .flat_map(|phase| phase.steps.iter().map(|step| step.name.clone()))
            .collect();
    }
    if let Some(steps) = journey.steps.as_deref() {
        return steps.iter().map(|step| step.name.clone()).collect();
    }
    if journey
        .native_steps
        .as_deref()
        .is_some_and(|steps| same_strings(steps, &["help", "return", "exit"]))
    {
        return ["opened", "help", "returned", "exited"]
            .into_iter()
            .map(str::to_string)
            .collect();
    }
    Vec::new()
}

fn run_unified_journey(
    session: &Session,
    journey: &Journey,
    plans: &[CheckpointPlan],
    is_delivery: bool,
    manifest: &SuiteManifest,
    pixel_profile: &PixelProfile,
    store: &FrozenStore,
    evidence_roots: &EvidenceRootsEvidence,
    samples_dir: &Path,
    output_dir: &Path,
    child_environment: &BTreeMap<String, String>,
    fixture: &FixturePaths,
    dataset: &FixtureDataset,
    args: &[String],
    case_name: &str,
    result_writer: &mut CaseResultWriter,
    phase: Option<&JourneyPhase>,
    action_evidence_writer: Option<&mut CaseResultWriter>,
    phase_result_index: Option<usize>,
) -> Result<()> {
    let expected_root = Path::new(&evidence_roots.expected_store_root);
    let mut attempts = plans
        .iter()
        .map(|step| CheckpointAttemptEvidence {
            name: step.name.clone(),
            reached: false,
            capture_attempted: false,
            outcome: "not-run".to_string(),
            reason: None,
            raw_prefix_offset: None,
            raw_prefix_sha256: None,
            sample_file: None,
            sample_sha256: None,
            native_snapshot_file: None,
            native_snapshot_sha256: None,
        })
        .collect::<Vec<_>>();
    let mut checkpoints = Vec::new();
    let mut capture_errors = Vec::new();
    let mut execution_errors = Vec::new();
    let mut application_errors = Vec::new();
    let mut setup_errors = Vec::new();
    let mut journey_active = true;
    let mut quit_dialog_ready = false;
    let mut quit_action_attempted = false;
    let mut quit_dialog_readiness_attempted = false;
    let mut cleanup_actions = Vec::new();
    let mut cleanup_selection = None;
    let mut action_evidence_writer = action_evidence_writer;

    for (index, step) in plans.iter().enumerate() {
        if !is_delivery && step.name == "exited" {
            continue;
        }
        if !journey_active {
            continue;
        }
        for (action_index, action) in step.actions.iter().enumerate() {
            if phase.is_some_and(|phase| {
                step.name == phase.dialog_checkpoint
                    && matches!(action, NativeAction::Key { key } if key == "Ctrl+Q")
            }) {
                quit_action_attempted = true;
            }
            let fault_control_active = action_evidence_writer
                .as_deref()
                .map(|writer| writer.evidence.fault_control.active)
                .unwrap_or(result_writer.evidence.fault_control.active);
            let suppressed_by_control = fault_control_active
                && journey.name == "first_delivery_jackin_settings"
                && step.name == "selected"
                && action_index == 0;
            let (action_result, evidence_result) = match dispatch_recorded_action(
                result_writer,
                &mut action_evidence_writer,
                session,
                case_id(journey),
                phase.map(|phase| phase.name.as_str()),
                &step.name,
                action,
                suppressed_by_control,
            ) {
                Ok(outcome) => outcome,
                Err(error) => {
                    execution_errors.push(format!(
                        "persist {} action attempt for {} failed: {error:#}",
                        step.name, journey.name
                    ));
                    journey_active = false;
                    break;
                }
            };
            if let Err(error) = evidence_result {
                capture_errors.push(format!(
                    "persist {} action dispatch result failed: {error:#}",
                    step.name
                ));
            }
            if !record_execution_step(
                &mut execution_errors,
                &format!("execute {} action for {}", step.name, journey.name),
                action_result,
            ) {
                journey_active = false;
                break;
            }
        }
        if !journey_active {
            continue;
        }
        if phase.is_some_and(|phase| step.name == phase.dialog_checkpoint) {
            quit_dialog_readiness_attempted = true;
        }
        journey_active = match wait_for_step_readiness(session, case_id(journey), step) {
            Ok(ReadinessOutcome::Ready) => {
                if phase.is_some_and(|phase| step.name == phase.dialog_checkpoint) {
                    quit_dialog_ready = true;
                }
                true
            }
            Ok(ReadinessOutcome::TimedOut(message)) => {
                application_errors.push(message.clone());
                if phase.is_some() {
                    let _ = try_capture_checkpoint(
                        session,
                        step,
                        &mut attempts[index],
                        journey,
                        pixel_profile,
                        store,
                        expected_root,
                        samples_dir,
                        &mut checkpoints,
                        &mut capture_errors,
                        &mut application_errors,
                        &mut setup_errors,
                    );
                }
                let observation_state = checkpoints
                    .iter()
                    .rev()
                    .find(|checkpoint| {
                        checkpoint.name == step.name
                            && checkpoint
                                .test_id
                                .starts_with(&format!("termrock/{}/", journey.name))
                    })
                    .map(checkpoint_observation_state)
                    .unwrap_or("mismatched");
                if let Err(error) = with_action_writer(
                    result_writer,
                    &mut action_evidence_writer,
                    |writer| {
                        writer.observe_checkpoint_actions(
                            phase.map(|phase| phase.name.as_str()),
                            &step.name,
                            observation_state,
                        )
                    },
                ) {
                    capture_errors.push(format!(
                        "persist {} readiness observation failed: {error:#}",
                        step.name
                    ));
                }
                attempts[index].reason = Some(message);
                false
            }
            Err(error) => {
                let message = format!("capture {} readiness failed: {error:#}", step.name);
                capture_errors.push(message.clone());
                attempts[index].reason = Some(message);
                false
            }
        };
        if !journey_active {
            continue;
        }
        journey_active = record_execution_step(
            &mut execution_errors,
            &format!("settle {} screen", step.name),
            settle(session),
        );
        if !journey_active {
            attempts[index].reason = execution_errors.last().cloned();
            continue;
        }
        journey_active = try_capture_checkpoint(
            session,
            step,
            &mut attempts[index],
            journey,
            pixel_profile,
            store,
            expected_root,
            samples_dir,
            &mut checkpoints,
            &mut capture_errors,
            &mut application_errors,
            &mut setup_errors,
        );
        let checkpoint = checkpoints.iter().rev().find(|checkpoint| {
            checkpoint.name == step.name
                && checkpoint.test_id.starts_with(&format!("termrock/{}/", journey.name))
        });
        if let Some(checkpoint) = checkpoint {
            for observation in checkpoint
                .selection_observations
                .iter()
                .filter(|observation| !observation.matched)
            {
                application_errors.push(format!(
                    "{}/{} selection assertion failed: {}",
                    case_id(journey),
                    step.name,
                    observation
                        .problem
                        .as_deref()
                        .unwrap_or("selected-row observation did not match the reference")
                ));
            }
        }
        let action_observation = checkpoint
            .map(checkpoint_observation_state)
            .or_else(|| {
                (attempts[index].outcome == "application-failure").then_some("mismatched")
            });
        if let Some(action_observation) = action_observation {
            if let Err(error) = with_action_writer(
                result_writer,
                &mut action_evidence_writer,
                |writer| {
                    writer.observe_checkpoint_actions(
                        phase.map(|phase| phase.name.as_str()),
                        &step.name,
                        action_observation,
                    )
                },
            ) {
                capture_errors.push(format!(
                    "persist {} action observation failed: {error:#}",
                    step.name
                ));
            }
        }
    }

    let (exit_key, exit_key_attempted, exit_key_error) = if let Some(phase) = phase {
        let dialog_step = plans
            .iter()
            .find(|step| step.name == phase.dialog_checkpoint)
            .context("validated phase quit-dialog checkpoint disappeared")?;
        let mut cleanup_errors = Vec::new();
        cleanup_actions.push("Ctrl+Q".to_string());
        if !quit_action_attempted {
            let action = NativeAction::Key {
                key: "Ctrl+Q".to_string(),
            };
            match dispatch_recorded_action(
                result_writer,
                &mut action_evidence_writer,
                session,
                case_id(journey),
                Some(phase.name.as_str()),
                "cleanup_dialog",
                &action,
                false,
            ) {
                Ok((dispatch_result, evidence_result)) => {
                    if let Err(error) = evidence_result {
                        capture_errors.push(format!(
                            "persist cleanup Ctrl+Q dispatch result failed: {error:#}"
                        ));
                    }
                    if let Err(error) = dispatch_result {
                        let message = format!("send cleanup Ctrl+Q failed: {error:#}");
                        execution_errors.push(message.clone());
                        cleanup_errors.push(message);
                    }
                }
                Err(error) => {
                    let message = format!("persist cleanup Ctrl+Q action attempt failed: {error:#}");
                    execution_errors.push(message.clone());
                    cleanup_errors.push(message);
                }
            }
        }
        if !quit_dialog_ready && !quit_dialog_readiness_attempted {
            match wait_for_step_readiness(session, case_id(journey), dialog_step) {
                Ok(ReadinessOutcome::Ready) => quit_dialog_ready = true,
                Ok(ReadinessOutcome::TimedOut(message)) => application_errors.push(message),
                Err(error) => capture_errors.push(format!(
                    "cleanup quit-dialog readiness capture failed: {error:#}"
                )),
            }
        }
        cleanup_actions.push("Right".to_string());
        let right_action = NativeAction::Key {
            key: "Right".to_string(),
        };
        match dispatch_recorded_action(
            result_writer,
            &mut action_evidence_writer,
            session,
            case_id(journey),
            Some(phase.name.as_str()),
            "cleanup_selected_quit",
            &right_action,
            false,
        ) {
            Ok((dispatch_result, evidence_result)) => {
                if let Err(error) = evidence_result {
                    capture_errors.push(format!(
                        "persist cleanup Right dispatch result failed: {error:#}"
                    ));
                }
                if let Err(error) = dispatch_result {
                    let message = format!("send cleanup Right failed: {error:#}");
                    execution_errors.push(message.clone());
                    cleanup_errors.push(message);
                }
            }
            Err(error) => {
                let message = format!("persist cleanup Right action attempt failed: {error:#}");
                execution_errors.push(message.clone());
                cleanup_errors.push(message);
            }
        }
        if quit_dialog_ready {
            if let Err(error) = settle(session) {
                execution_errors.push(format!("settle selected quit action failed: {error:#}"));
            }
            let snapshot_name = format!("{}_cleanup_selected_quit", phase.name);
            let mut attempt = CheckpointAttemptEvidence {
                name: snapshot_name.clone(),
                reached: true,
                capture_attempted: true,
                outcome: "captured".to_string(),
                reason: None,
                raw_prefix_offset: None,
                raw_prefix_sha256: None,
                sample_file: None,
                sample_sha256: None,
                native_snapshot_file: None,
                native_snapshot_sha256: None,
            };
            match session.capture_checkpoint(SnapshotScope::ActiveScreen) {
                Ok(captured) => {
                    let raw_prefix_offset = captured.raw_pty_offset();
                    attempt.raw_prefix_offset = Some(raw_prefix_offset);
                    match save_native_snapshot(
                        captured.snapshot(),
                        &snapshot_name,
                        samples_dir,
                        &mut attempt,
                    ) {
                        Ok((snapshot_file, snapshot_sha256)) => {
                            cleanup_selection = Some(CleanupSelectionEvidence {
                                snapshot_file,
                                snapshot_sha256,
                                raw_prefix_offset,
                                raw_prefix_sha256: None,
                            });
                        }
                        Err(error) => capture_errors.push(format!(
                            "retain selected quit action snapshot failed: {error:#}"
                        )),
                    }
                }
                Err(error) => {
                    capture_errors.push(format!("capture selected quit action failed: {error:#}"))
                }
            }
        }
        cleanup_actions.push("Enter".to_string());
        let enter_action = NativeAction::Key {
            key: "Enter".to_string(),
        };
        match dispatch_recorded_action(
            result_writer,
            &mut action_evidence_writer,
            session,
            case_id(journey),
            Some(phase.name.as_str()),
            "exit",
            &enter_action,
            false,
        ) {
            Ok((dispatch_result, evidence_result)) => {
                if let Err(error) = evidence_result {
                    capture_errors.push(format!(
                        "persist cleanup Enter dispatch result failed: {error:#}"
                    ));
                }
                if let Err(error) = dispatch_result {
                    let message = format!("send cleanup Enter failed: {error:#}");
                    execution_errors.push(message.clone());
                    cleanup_errors.push(message);
                }
            }
            Err(error) => {
                let message = format!("persist cleanup Enter action attempt failed: {error:#}");
                execution_errors.push(message.clone());
                cleanup_errors.push(message);
            }
        }
        (None, true, cleanup_errors.first().cloned())
    } else {
        let exit_key = journey.exit_key.clone();
        let exit_intended = is_delivery || journey_active;
        let exit_event = if exit_intended {
            match with_action_writer(
                result_writer,
                &mut action_evidence_writer,
                |writer| {
                    writer.begin_action(
                        case_id(journey),
                        phase.map(|phase| phase.name.as_str()),
                        "exit",
                        "key",
                    )
                },
            ) {
                Ok(index) => Some(index),
                Err(error) => {
                    execution_errors.push(format!(
                        "persist application exit action attempt failed: {error:#}"
                    ));
                    None
                }
            }
        } else {
            None
        };
        let exit_key_attempted = exit_intended && exit_event.is_some();
        let exit_key_error = if exit_intended && !exit_key_attempted {
            Some("exit action attempt was not persisted".to_string())
        } else if exit_key_attempted {
            match press_required(session, journey.exit_key.as_deref(), "exit") {
                Ok(()) => None,
                Err(error) => {
                    execution_errors.push(format!("send application exit key failed: {error:#}"));
                    Some(format!("{error:#}"))
                }
            }
        } else {
            None
        };
        if let Some(exit_event) = exit_event {
            let dispatch_state = if exit_key_error.is_none() {
                ActionDispatchState::Dispatched
            } else {
                ActionDispatchState::Failed
            };
            if let Err(error) = with_action_writer(
                result_writer,
                &mut action_evidence_writer,
                |writer| writer.complete_action_dispatch(exit_event, dispatch_state),
            ) {
                capture_errors.push(format!(
                    "persist application exit action dispatch result failed: {error:#}"
                ));
            }
        }
        (exit_key, exit_key_attempted, exit_key_error)
    };
    let wait_exit_succeeded = match session.execute(Operation::WaitExit {
        timeout_ms: Some(EXIT_TIMEOUT_MS),
    }) {
        Ok(_) => true,
        Err(error) => {
            execution_errors.push(format!(
                "wait for application process exit failed: {error:#}"
            ));
            false
        }
    };
    let process_state_before_finalize = match session.execute(Operation::State) {
        Ok(OperationResult::State(state)) => {
            if state.exited != Some(0) {
                execution_errors.push(format!(
                    "child did not exit with status 0 before drain: {:?}",
                    state.exited
                ));
            }
            if let Some(signal) = state.exit_signal.as_ref() {
                execution_errors.push(format!("child exited from signal before drain {signal:?}"));
            }
            Some(process_state_evidence(&state))
        }
        Ok(other) => {
            execution_errors.push(format!(
                "Operation::State returned unexpected result before drain: {other:?}"
            ));
            None
        }
        Err(error) => {
            execution_errors.push(format!("read process state before drain failed: {error:#}"));
            None
        }
    };

    let mut raw_bytes = None;
    let mut process_state_after_finalize = None;
    let mut terminal_restoration = None;
    let mut post_drain_exit_confirmed = false;
    let mut drain_succeeded = false;
    match session.finish_capture(RAW_DRAIN_TIMEOUT) {
        Ok(bytes) => {
            if bytes.is_empty() {
                capture_errors
                    .push("checked PTY finalization returned an empty byte stream".to_string());
            } else {
                raw_bytes = Some(bytes);
            }
            match session.execute(Operation::State) {
                Ok(OperationResult::State(state)) => {
                    post_drain_exit_confirmed =
                        state.exited == Some(0) && state.exit_signal.is_none();
                    if state.exited != Some(0) {
                        execution_errors.push(format!(
                            "post-drain child did not exit with status 0: {:?}",
                            state.exited
                        ));
                    }
                    if let Some(signal) = state.exit_signal.as_ref() {
                        execution_errors
                            .push(format!("post-drain child exited from signal {signal:?}"));
                    }
                    let restoration =
                        terminal_restoration_evidence(&state.modes, &state.mouse_mode);
                    if !restoration.restored {
                        execution_errors.extend(restoration.errors.iter().cloned());
                    }
                    terminal_restoration = Some(restoration);
                    process_state_after_finalize = Some(process_state_evidence(&state));
                }
                Ok(other) => execution_errors.push(format!(
                    "Operation::State returned unexpected result after drain: {other:?}"
                )),
                Err(error) => execution_errors
                    .push(format!("read process state after drain failed: {error:#}")),
            }
            drain_succeeded = raw_bytes.is_some() && post_drain_exit_confirmed;
        }
        Err(error) => capture_errors.push(format!(
            "checked PTY capture finalization failed; no raw bytes are marked exact: {error:#}"
        )),
    }

    let exit_observed = wait_exit_succeeded
        && drain_succeeded
        && post_drain_exit_confirmed
        && terminal_restoration
            .as_ref()
            .is_some_and(|restoration| restoration.restored);
    if let Err(error) = with_action_writer(
        result_writer,
        &mut action_evidence_writer,
        |writer| {
            writer.observe_exit(phase.map(|phase| phase.name.as_str()), exit_observed)
        },
    ) {
        capture_errors.push(format!("persist exit action observation failed: {error:#}"));
    }

    if !is_delivery {
        let exit_index = plans
            .iter()
            .position(|step| step.name == "exited")
            .context("legacy plan lacks exited checkpoint")?;
        let pre_finalize_exit_confirmed = wait_exit_succeeded
            && process_state_before_finalize
                .as_ref()
                .is_some_and(|state| state.exited == Some(0) && state.exit_signal.is_none());
        let finalization_succeeded =
            raw_bytes.as_ref().is_some_and(|bytes| !bytes.is_empty()) && post_drain_exit_confirmed;
        if pre_finalize_exit_confirmed && finalization_succeeded {
            let _ = try_capture_checkpoint(
                session,
                &plans[exit_index],
                &mut attempts[exit_index],
                journey,
                pixel_profile,
                store,
                expected_root,
                samples_dir,
                &mut checkpoints,
                &mut capture_errors,
                &mut application_errors,
                &mut setup_errors,
            );
        } else {
            attempts[exit_index].reason = Some(
                "exited checkpoint was not reached because checked process exit or drain was incomplete"
                    .to_string(),
            );
            execution_errors.push(
                "exited checkpoint not-run after incomplete process exit or drain".to_string(),
            );
        }
    }

    let mut raw_pty = None;
    let mut raw_tail = None;
    if let Some(bytes) = raw_bytes.as_deref() {
        for attempt in &mut attempts {
            let Some(offset) = attempt.raw_prefix_offset else {
                continue;
            };
            if offset > bytes.len() {
                capture_errors.push(format!(
                    "{} checkpoint PTY offset {} exceeds drained stream length {}",
                    attempt.name,
                    offset,
                    bytes.len()
                ));
                continue;
            }
            let prefix_sha256 = sha256_hex(&bytes[..offset]);
            attempt.raw_prefix_sha256 = Some(prefix_sha256.clone());
            if let Some(checkpoint) = checkpoints
                .iter_mut()
                .find(|checkpoint| checkpoint.name == attempt.name)
            {
                checkpoint.raw_prefix_sha256 = Some(prefix_sha256);
            }
        }
        if let Some(selection) = cleanup_selection.as_mut() {
            if selection.raw_prefix_offset > bytes.len() {
                capture_errors.push(format!(
                    "selected quit snapshot PTY offset {} exceeds drained stream length {}",
                    selection.raw_prefix_offset,
                    bytes.len()
                ));
            } else {
                selection.raw_prefix_sha256 =
                    Some(sha256_hex(&bytes[..selection.raw_prefix_offset]));
            }
        }
        let tail_start = bytes.len().saturating_sub(512);
        let tail = &bytes[tail_start..];
        if is_delivery {
            raw_tail = Some(RawTailEvidence {
                byte_offset: tail_start,
                byte_length: tail.len(),
                sha256: sha256_hex(tail),
                hex: hex_bytes(tail),
            });
        }
        match write_atomic(output_dir, "raw_pty.bin", bytes) {
            Ok(()) => {
                raw_pty = Some(RawPtyEvidence {
                    byte_length: bytes.len(),
                    sha256: sha256_hex(bytes),
                    maximum_retained_bytes: 32 * 1024 * 1024,
                    bytes_file: "raw_pty.bin".to_string(),
                    exact_bytes: true,
                    drain_reached_eof: true,
                });
            }
            Err(error) => capture_errors.push(format!(
                "write exact retained raw PTY bytes failed: {error:#}"
            )),
        }
    }

    let first_failure = setup_errors
        .first()
        .or_else(|| application_errors.first())
        .or_else(|| execution_errors.first())
        .or_else(|| capture_errors.first())
        .cloned()
        .unwrap_or_else(|| "delivery journey did not complete".to_string());
    for attempt in &mut attempts {
        if !attempt.reached && attempt.reason.is_none() {
            attempt.reason = Some(format!(
                "checkpoint phase not-run after prerequisite failure: {first_failure}"
            ));
        }
    }
    let not_reached_checkpoints = attempts
        .iter()
        .filter(|attempt| !attempt.reached)
        .map(|attempt| NotReachedCheckpointEvidence {
            name: attempt.name.clone(),
            status: "not-run".to_string(),
            reason: attempt
                .reason
                .clone()
                .unwrap_or_else(|| "checkpoint phase was not reached".to_string()),
        })
        .collect::<Vec<_>>();
    result_writer.evidence.not_run_checkpoints = not_reached_checkpoints.clone();
    let comparison_failure = checkpoints
        .iter()
        .any(|checkpoint| !checkpoint.comparison.passed);
    let comparison_failure_kind = checkpoints
        .iter()
        .find(|checkpoint| !checkpoint.comparison.passed)
        .and_then(|checkpoint| checkpoint.comparison.failure_kind);
    let (_, exit_failure_kind, case_status) = classify_case_result(
        &setup_errors,
        &capture_errors,
        &application_errors,
        &execution_errors,
        comparison_failure,
    );

    if is_delivery {
        let exit_result = DeliveryExitResult {
            schema_version: 1,
            suite_id: manifest.version.suite_id.clone(),
            case_id: case_id(journey).to_string(),
            test_name: journey.name.clone(),
            phase_name: phase.map(|phase| phase.name.clone()),
            motion: phase.map(|phase| phase.motion.clone()),
            case_status: case_status.to_string(),
            failure_kind: exit_failure_kind,
            comparison_failure_kind,
            exit_key: exit_key.clone(),
            exit_key_attempted,
            exit_key_error: exit_key_error.clone(),
            cleanup_actions: cleanup_actions.clone(),
            cleanup_selection: cleanup_selection.clone(),
            wait_exit_succeeded,
            process_state_before_finalize: process_state_before_finalize.clone(),
            drain_succeeded,
            raw_pty: raw_pty.clone(),
            raw_tail: raw_tail.clone(),
            process_state_after_finalize: process_state_after_finalize.clone(),
            terminal_restoration: terminal_restoration.clone(),
            setup_errors: setup_errors.clone(),
            application_errors: application_errors.clone(),
            execution_errors: execution_errors.clone(),
            capture_errors: capture_errors.clone(),
        };
        match serde_json::to_vec_pretty(&exit_result) {
            Ok(exit_bytes) => {
                if let Err(error) = write_atomic(output_dir, "exit-result.json", &exit_bytes) {
                    capture_errors.push(format!("write exit result failed: {error:#}"));
                }
            }
            Err(error) => capture_errors.push(format!("serialize exit result failed: {error:#}")),
        }
    }

    let comparison_failure = checkpoints
        .iter()
        .any(|checkpoint| !checkpoint.comparison.passed);
    let (typed_status, failure_kind, case_status) = classify_case_result(
        &setup_errors,
        &capture_errors,
        &application_errors,
        &execution_errors,
        comparison_failure,
    );
    result_writer.evidence.status = typed_status;
    result_writer.evidence.failure_kind = failure_kind;
    result_writer.evidence.comparison_failure_kind = comparison_failure_kind;
    result_writer.evidence.error = setup_errors
        .first()
        .or_else(|| capture_errors.first())
        .or_else(|| application_errors.first())
        .or_else(|| execution_errors.first())
        .or_else(|| {
            checkpoints
                .iter()
                .find(|checkpoint| !checkpoint.comparison.passed)
                .map(|checkpoint| &checkpoint.test_id)
        })
        .cloned();
    result_writer.persist()?;

    let index = CaptureIndex {
        schema_version: 3,
        suite_id: manifest.version.suite_id.clone(),
        case_name: case_name.to_string(),
        case_id: case_id(journey).to_string(),
        test_name: journey.name.clone(),
        phase_name: phase.map(|phase| phase.name.clone()),
        motion: phase.map(|phase| phase.motion.clone()),
        case_status: case_status.to_string(),
        failure_kind,
        comparison_failure_kind,
        frame_checkpoint_count: plans.len(),
        exit_result_file: is_delivery.then(|| "exit-result.json".to_string()),
        executable_name: journey.binary.clone(),
        arguments: args.to_vec(),
        terminal: TerminalEvidence {
            cols: manifest.suite.terminal.cols,
            rows: manifest.suite.terminal.rows,
            locale: manifest.suite.terminal.locale.clone(),
            timezone: manifest.suite.terminal.timezone.clone(),
            term: manifest.suite.terminal.term.clone(),
            color_term: manifest.suite.terminal.color_term.clone(),
            render_profile: manifest.suite.render_profile.clone(),
            platform_key: platform_key(),
        },
        child_environment: child_environment.clone(),
        no_color_present: child_environment.contains_key("NO_COLOR"),
        working_directory: fixture.cwd.to_string_lossy().into_owned(),
        fixture_dataset_id: dataset.dataset_id.clone(),
        coverage_scope: manifest.suite.coverage_scope.clone(),
        capture_api: "tuiscotti-tui-test::Session::capture_checkpoint(ActiveScreen+raw-prefix-offset-under-one-terminal-state-lock);offset-saved-before-render;native-snapshot-written-before-sample-validation;Session::finish_capture(bounded-checked-drain)".to_string(),
        evidence_roots: evidence_roots.clone(),
        raw_pty,
        raw_tail,
        cleanup_selection: cleanup_selection.clone(),
        drain_succeeded,
        process_state_before_finalize,
        process_state_after_finalize,
        terminal_restoration,
        checkpoints,
        checkpoint_attempts: attempts,
        not_reached_checkpoints,
        setup_errors,
        application_errors,
        capture_errors,
        execution_errors,
    };
    let failures = index
        .checkpoints
        .iter()
        .filter(|checkpoint| !checkpoint.comparison.passed)
        .map(|checkpoint| {
            format!(
                "{}: reason={:?}, report={:?}, first_cell={:?}",
                checkpoint.test_id,
                checkpoint.comparison.reason,
                checkpoint.comparison.report_path,
                checkpoint.comparison.first_changed_cell
            )
        })
        .collect::<Vec<_>>();
    let all_prefixes_bound = index
        .checkpoints
        .iter()
        .all(|checkpoint| checkpoint.raw_prefix_sha256.is_some());
    let raw_stream_complete = index
        .raw_pty
        .as_ref()
        .is_some_and(|raw| raw.byte_length > 0 && raw.exact_bytes && raw.drain_reached_eof);
    let terminal_restored = index
        .terminal_restoration
        .as_ref()
        .is_some_and(|state| state.restored);
    let cleanup_capture_complete = phase.is_none()
        || cleanup_selection
            .as_ref()
            .is_some_and(|selection| selection.raw_prefix_sha256.is_some());
    let phase_passed = index.checkpoints.len() == index.frame_checkpoint_count
        && index.not_reached_checkpoints.is_empty()
        && index.setup_errors.is_empty()
        && index.application_errors.is_empty()
        && index.execution_errors.is_empty()
        && index.capture_errors.is_empty()
        && failures.is_empty()
        && index.drain_succeeded
        && raw_stream_complete
        && terminal_restored
        && all_prefixes_bound
        && cleanup_capture_complete;
    if let Some(phase_result_index) = phase_result_index {
        let phase_result = action_evidence_writer
            .as_deref_mut()
            .context("phase summary writer is missing")?
            .evidence
            .phases
            .get_mut(phase_result_index)
            .context("phase summary index is out of range")?;
        phase_result.status = if phase_passed {
            PhaseRunStatus::Passed
        } else {
            PhaseRunStatus::Failed
        };
        phase_result.launched = true;
        phase_result.cleanup_actions_attempted = cleanup_actions.clone();
        phase_result.wait_exit_succeeded = Some(wait_exit_succeeded);
        phase_result.exit_status = index
            .process_state_after_finalize
            .as_ref()
            .and_then(|state| state.exited)
            .or_else(|| {
                index
                    .process_state_before_finalize
                    .as_ref()
                    .and_then(|state| state.exited)
            });
        phase_result.exit_status_zero = index
            .process_state_after_finalize
            .as_ref()
            .map(|state| state.exited == Some(0));
        phase_result.exit_signal = index
            .process_state_after_finalize
            .as_ref()
            .and_then(|state| state.exit_signal.clone())
            .or_else(|| {
                index
                    .process_state_before_finalize
                    .as_ref()
                    .and_then(|state| state.exit_signal.clone())
            });
        phase_result.no_signal = index
            .process_state_after_finalize
            .as_ref()
            .map(|state| state.exit_signal.is_none());
        phase_result.eof_drained = index.raw_pty.as_ref().map(|raw| raw.drain_reached_eof);
        phase_result.post_drain_exit_confirmed = index
            .process_state_after_finalize
            .as_ref()
            .map(|state| state.exited == Some(0) && state.exit_signal.is_none());
        phase_result.raw_pty_sha256 = index.raw_pty.as_ref().map(|raw| raw.sha256.clone());
        phase_result.raw_checkpoint_prefixes = index
            .checkpoints
            .iter()
            .filter_map(|checkpoint| {
                checkpoint
                    .raw_prefix_sha256
                    .as_ref()
                    .map(|hash| (checkpoint.name.clone(), hash.clone()))
            })
            .collect();
        phase_result.terminal_restored = Some(terminal_restored);
        phase_result.cleanup_snapshot_file = if let Some(selection) = cleanup_selection.as_ref() {
            let phase = phase.context("phase cleanup evidence is missing its phase")?;
            Some(
                Path::new("..")
                    .join(case_id(journey))
                    .join("phases")
                    .join(phase.name.as_str())
                    .join(selection.snapshot_file.as_str())
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        };
        phase_result.cleanup_snapshot_sha256 = cleanup_selection
            .as_ref()
            .map(|selection| selection.snapshot_sha256.clone());
        phase_result.cleanup_raw_prefix_sha256 = cleanup_selection
            .as_ref()
            .and_then(|selection| selection.raw_prefix_sha256.clone());
        phase_result.error = result_writer.evidence.error.clone().or_else(|| {
            (!phase_passed).then(|| {
                "phase did not satisfy every checkpoint and lifecycle obligation".to_string()
            })
        });
    }
    let index_bytes = serde_json::to_vec_pretty(&index).context("serialize capture index")?;
    write_atomic(output_dir, "capture-index.json", &index_bytes).context("write capture index")?;
    ensure!(
        index.checkpoints.len() == index.frame_checkpoint_count,
        "captured {} of {} frame checkpoints; not reached: {:?}; capture errors: {:?}",
        index.checkpoints.len(),
        index.frame_checkpoint_count,
        index.not_reached_checkpoints,
        index.capture_errors
    );
    ensure!(
        index.capture_errors.is_empty(),
        "checkpoint capture failures: {:?}",
        index.capture_errors
    );
    ensure!(
        index.setup_errors.is_empty(),
        "suite setup/evidence-store failures: {:?}",
        index.setup_errors
    );
    ensure!(
        index.application_errors.is_empty(),
        "application visible-state failures: {:?}",
        index.application_errors
    );
    ensure!(
        index.execution_errors.is_empty(),
        "journey execution failures: {:?}",
        index.execution_errors
    );
    ensure!(
        failures.is_empty(),
        "frozen visual checks failed: {failures:#?}"
    );
    ensure!(
        drain_succeeded,
        "exit result lacks successful checked PTY drain and zero exit"
    );
    ensure!(
        index
            .raw_pty
            .as_ref()
            .is_some_and(|raw| raw.byte_length > 0 && raw.exact_bytes && raw.drain_reached_eof),
        "raw PTY stream is incomplete"
    );
    ensure!(
        index
            .terminal_restoration
            .as_ref()
            .is_some_and(|state| state.restored),
        "terminal restoration failed or has no post-finalization state evidence"
    );
    ensure!(
        index
            .checkpoints
            .iter()
            .all(|checkpoint| checkpoint.raw_prefix_sha256.is_some()),
        "one or more frame checkpoints lack a post-drain raw-prefix hash"
    );
    Ok(())
}

fn process_state_evidence(state: &tui_test::State) -> ProcessStateEvidence {
    ProcessStateEvidence {
        exited: state.exited,
        exit_signal: state
            .exit_signal
            .as_ref()
            .map(|signal| format!("{signal:?}")),
        modes: state.modes.clone(),
        mouse_mode: state.mouse_mode.clone(),
    }
}

fn classify_case_result(
    setup_errors: &[String],
    capture_errors: &[String],
    application_errors: &[String],
    execution_errors: &[String],
    comparison_failed: bool,
) -> (CaseResultStatus, Option<CaseFailureKind>, &'static str) {
    let failure_kind = if !setup_errors.is_empty() {
        Some(CaseFailureKind::Setup)
    } else if !capture_errors.is_empty() {
        Some(CaseFailureKind::Capture)
    } else if !application_errors.is_empty() {
        Some(CaseFailureKind::Application)
    } else if !execution_errors.is_empty() {
        Some(CaseFailureKind::Execution)
    } else if comparison_failed {
        Some(CaseFailureKind::Comparison)
    } else {
        None
    };
    match failure_kind {
        None => (CaseResultStatus::Passed, None, "passed"),
        Some(CaseFailureKind::Setup) => (CaseResultStatus::Failed, failure_kind, "failed/setup"),
        Some(CaseFailureKind::Capture) => {
            (CaseResultStatus::Failed, failure_kind, "failed/capture")
        }
        Some(CaseFailureKind::Application) => {
            (CaseResultStatus::Failed, failure_kind, "failed/application")
        }
        Some(CaseFailureKind::Execution) => {
            (CaseResultStatus::Failed, failure_kind, "failed/execution")
        }
        Some(CaseFailureKind::Comparison) => {
            (CaseResultStatus::Failed, failure_kind, "failed/comparison")
        }
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn execute_native_action(session: &Session, action: &NativeAction) -> Result<()> {
    match action {
        NativeAction::Key { key } => press(session, key),
        NativeAction::Write { text } => {
            session
                .execute(Operation::Write { data: text.clone() })
                .with_context(|| format!("write native text {text:?}"))?;
            Ok(())
        }
        NativeAction::Resize { cols, rows } => {
            session
                .execute(Operation::Resize {
                    cols: *cols,
                    rows: *rows,
                })
                .with_context(|| format!("resize native terminal to {cols}x{rows}"))?;
            Ok(())
        }
        NativeAction::MouseClick { x, y, button } => {
            let button = match button.as_str() {
                "left" => MouseButton::Left,
                _ => bail!("unsupported native mouse button {button:?}"),
            };
            let options = MouseOptions::new(button);
            session
                .execute(Operation::Mouse {
                    action: MouseAction::Down {
                        x: *x,
                        y: *y,
                        options,
                    },
                })
                .context("press native mouse button")?;
            session
                .execute(Operation::Mouse {
                    action: MouseAction::Up {
                        x: *x,
                        y: *y,
                        options,
                    },
                })
                .context("release native mouse button")?;
            Ok(())
        }
    }
}

fn wait_for_step_readiness(
    session: &Session,
    case_id: &str,
    step: &CheckpointPlan,
) -> Result<ReadinessOutcome> {
    let deadline = Instant::now() + READINESS_TIMEOUT;
    loop {
        let captured = session
            .capture_checkpoint(SnapshotScope::ActiveScreen)
            .with_context(|| format!("capture {case_id}/{} readiness", step.name))?;
        let snapshot = captured.snapshot();
        let revision = snapshot.metadata().revision;
        let raw_offset = captured.raw_pty_offset();
        let problems = step_readiness_problems(snapshot, step);
        if problems.is_empty() {
            return Ok(ReadinessOutcome::Ready);
        }
        let now = Instant::now();
        if now >= deadline {
            return Ok(ReadinessOutcome::TimedOut(format!(
                "{case_id}/{} did not reach required visible state within {:?}; anchor problems {:?}, revision {:?}, raw PTY offset {:?}",
                step.name, READINESS_TIMEOUT, problems, revision, raw_offset
            )));
        }
        thread::sleep(READINESS_POLL_INTERVAL.min(deadline.saturating_duration_since(now)));
    }
}

fn step_readiness_problems(
    snapshot: &tui_test::TerminalSnapshot,
    step: &CheckpointPlan,
) -> Vec<String> {
    let required = step
        .required_anchors
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let forbidden = step
        .forbidden_anchors
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut problems = missing_snapshot_anchors(snapshot, &required);
    if snapshot.cols() != u32::from(step.expected_cols)
        || snapshot.rows() != u32::from(step.expected_rows)
    {
        problems.push(format!(
            "terminal size is {}x{}, expected {}x{}",
            snapshot.cols(),
            snapshot.rows(),
            step.expected_cols,
            step.expected_rows
        ));
    }
    problems.extend(
        forbidden
            .iter()
            .filter(|anchor| snapshot_has_anchor(snapshot, anchor))
            .map(|anchor| format!("forbidden anchor {anchor:?} is visible")),
    );
    problems
}

fn assert_step_snapshot(
    snapshot: &tui_test::TerminalSnapshot,
    step: &CheckpointPlan,
) -> Result<()> {
    if step.name == "exited" {
        return Ok(());
    }
    let problems = step_readiness_problems(snapshot, step);
    ensure!(
        problems.is_empty(),
        "captured {} checkpoint does not match required visible state: {:?}",
        step.name,
        problems
    );
    Ok(())
}

fn save_native_snapshot(
    snapshot: &tui_test::TerminalSnapshot,
    checkpoint_name: &str,
    samples_dir: &Path,
    attempt: &mut CheckpointAttemptEvidence,
) -> Result<(String, String)> {
    let bytes =
        serde_json::to_vec_pretty(snapshot).context("serialize native terminal snapshot")?;
    let hash = sha256_hex(&bytes);
    let file_name = format!("{checkpoint_name}.snapshot.json");
    write_atomic(samples_dir, &file_name, &bytes)
        .with_context(|| format!("write {checkpoint_name} native terminal snapshot"))?;
    let relative_path = format!("samples/{file_name}");
    attempt.native_snapshot_file = Some(relative_path.clone());
    attempt.native_snapshot_sha256 = Some(hash.clone());
    Ok((relative_path, hash))
}

fn try_capture_checkpoint(
    session: &Session,
    step: &CheckpointPlan,
    attempt: &mut CheckpointAttemptEvidence,
    journey: &Journey,
    pixel_profile: &PixelProfile,
    store: &FrozenStore,
    expected_root: &Path,
    samples_dir: &Path,
    checkpoints: &mut Vec<CheckpointEvidence>,
    capture_errors: &mut Vec<String>,
    application_errors: &mut Vec<String>,
    setup_errors: &mut Vec<String>,
) -> bool {
    attempt.reached = true;
    attempt.capture_attempted = true;
    let mut readiness_failed = false;
    match capture_checkpoint(
        session,
        step,
        attempt,
        &mut readiness_failed,
        journey,
        pixel_profile,
        store,
        expected_root,
        samples_dir,
    ) {
        Ok(checkpoint) => {
            attempt.sample_file = Some(checkpoint.sample_file.clone());
            attempt.sample_sha256 = Some(checkpoint.sample_sha256.clone());
            if checkpoint.comparison.passed {
                attempt.outcome = "captured".to_string();
            } else {
                attempt.outcome = "comparison-failure".to_string();
                attempt.reason = checkpoint.comparison.reason.clone();
            }
            checkpoints.push(checkpoint);
            true
        }
        Err(error) => {
            let message = format!("{}: {error:#}", step.name);
            let store_check_failed = error.chain().any(|cause| cause.is::<StoreCheckFailure>());
            attempt.outcome = if readiness_failed {
                "application-failure"
            } else if store_check_failed {
                "setup-failure"
            } else {
                "capture-failure"
            }
            .to_string();
            attempt.reason = Some(message.clone());
            match partial_sample_evidence(samples_dir, &step.name) {
                Ok(Some((sample_file, sample_sha256))) => {
                    attempt.sample_file = Some(sample_file);
                    attempt.sample_sha256 = Some(sample_sha256);
                }
                Ok(None) => {}
                Err(sample_error) => capture_errors.push(format!(
                    "{}: reading partial sample evidence failed: {sample_error:#}",
                    step.name
                )),
            }
            if readiness_failed {
                application_errors.push(message);
                false
            } else if store_check_failed {
                setup_errors.push(message);
                true
            } else {
                capture_errors.push(message);
                true
            }
        }
    }
}

fn capture_checkpoint(
    session: &Session,
    step: &CheckpointPlan,
    attempt: &mut CheckpointAttemptEvidence,
    readiness_failed: &mut bool,
    journey: &Journey,
    pixel_profile: &PixelProfile,
    store: &FrozenStore,
    expected_root: &Path,
    samples_dir: &Path,
) -> Result<CheckpointEvidence> {
    let captured = session
        .capture_checkpoint(SnapshotScope::ActiveScreen)
        .with_context(|| format!("capture {} ActiveScreen and raw PTY offset", step.name))?;
    let raw_offset = captured.raw_pty_offset();
    attempt.raw_prefix_offset = Some(raw_offset);
    let snapshot = captured.snapshot();
    let (native_snapshot_file, native_snapshot_sha256) =
        save_native_snapshot(snapshot, &step.name, samples_dir, attempt)?;
    let anchor_result = assert_step_snapshot(snapshot, step);
    if anchor_result.is_err() {
        *readiness_failed = true;
    }
    let platform = platform_key();
    ensure!(
        !platform.is_empty()
            && platform
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')),
        "platform key is invalid for a frozen pixel test ID: {platform:?}"
    );
    let test_id = format!("termrock/{}/{}/pixels-{platform}", journey.name, step.name);
    let sample = Sample::with_pixels(&test_id, snapshot, Observation::empty(), pixel_profile)
        .with_context(|| format!("render {} pinned pixels", step.name))?;
    let sample_bytes = sample.to_bytes().context("serialize state+pixel sample")?;
    let sample_hash = sha256_hex(&sample_bytes);
    let sample_name = format!("{}.sample.json", step.name);
    write_atomic(samples_dir, &sample_name, &sample_bytes)
        .with_context(|| format!("write {} candidate sample", step.name))?;
    anchor_result?;
    let contract = sample
        .render_contract()
        .context("state+pixels sample has no render contract")?;
    ensure!(
        snapshot.cursor().blink().is_some() && contract.blink().is_some(),
        "{} checkpoint has unknown cursor blink state",
        step.name
    );
    let image = sample.image().context("state+pixels sample has no image")?;
    let compared_pixels = u64::from(image.width()) * u64::from(image.height());
    let comparison = comparison_evidence(
        store.check(&sample_bytes).map_err(|error| {
            StoreCheckFailure(format!("frozen store check API failed: {error:#}"))
        })?,
        &test_id,
        sample.gate().as_str(),
        &sample_hash,
        compared_pixels,
    )?;
    let selection_observations = selection_observation_evidence(
        expected_root,
        journey,
        step,
        &sample_bytes,
        &platform,
    )?;
    Ok(CheckpointEvidence {
        name: step.name.clone(),
        test_id,
        selection_observations,
        source_revision: snapshot.metadata().revision,
        raw_prefix_offset: raw_offset,
        raw_prefix_sha256: None,
        sample_file: format!("samples/{sample_name}"),
        sample_sha256: sample_hash,
        native_snapshot_file,
        native_snapshot_sha256,
        cols: snapshot.cols(),
        rows: snapshot.rows(),
        cursor: cursor_evidence(snapshot),
        resolved_cells: cell_evidence(snapshot),
        image: image_evidence(image),
        render_contract: render_contract_evidence(contract),
        comparison,
    })
}

fn selection_observation_evidence(
    expected_root: &Path,
    journey: &Journey,
    step: &CheckpointPlan,
    candidate_bytes: &[u8],
    platform: &str,
) -> Result<Vec<SelectionObservationEvidence>> {
    if step.selection_observations.is_empty() {
        return Ok(Vec::new());
    }
    ensure!(
        journey.name == "first_delivery_jackin_settings"
            && matches!(step.name.as_str(), "initial" | "selected")
            && step.expected_cols == 120
            && step.expected_rows == 40,
        "selection observations are only approved for Jackin's 120x40 initial/selected checkpoints"
    );

    let (initial_file, initial_sha256, initial) =
        read_selection_reference(expected_root, journey, "initial", platform)?;
    let (selected_file, selected_sha256, selected) =
        read_selection_reference(expected_root, journey, "selected", platform)?;
    let initial_current = require_reference_label(&initial.snapshot, "Current directory", 7, 3)?;
    let initial_workspace =
        require_reference_label(&initial.snapshot, "payments-platform", 7, 4)?;
    let selected_current =
        require_reference_label(&selected.snapshot, "Current directory", 7, 3)?;
    let selected_workspace =
        require_reference_label(&selected.snapshot, "payments-platform", 7, 4)?;
    let active_style = selected_workspace.clone();
    let inactive_style = initial_workspace.clone();
    let approved_active_style = SelectionStyleEvidence {
        fg: "#48e054".to_string(),
        bg: "#173f1c".to_string(),
        bold: true,
        inverse: false,
    };
    let approved_inactive_style = SelectionStyleEvidence {
        fg: "#ffffff".to_string(),
        bg: "#000000".to_string(),
        bold: false,
        inverse: false,
    };
    ensure!(
        initial_current == active_style
            && selected_current == inactive_style
            && initial_workspace == inactive_style
            && selected_workspace == active_style
            && active_style == approved_active_style
            && inactive_style == approved_inactive_style,
        "approved Jackin reference selection signature changed; independent review is required"
    );

    let actual: SelectionSample =
        serde_json::from_slice(candidate_bytes).context("parse captured selection sample")?;
    ensure!(
        actual.snapshot.cols == 120
            && actual.snapshot.rows == 40
            && actual.snapshot.cells.len() == 120 * 40,
        "captured Jackin selection sample has an unexpected terminal shape"
    );

    let mut reference_files = BTreeMap::new();
    reference_files.insert(initial_file, initial_sha256);
    reference_files.insert(selected_file, selected_sha256);
    let mut observations = Vec::with_capacity(step.selection_observations.len());
    for plan in &step.selection_observations {
        let expected_style = if plan.selected {
            active_style.clone()
        } else {
            inactive_style.clone()
        };
        let found = locate_selection_label(&actual.snapshot, &plan.label);
        let actual_style = found.style.clone();
        let observed_selected = actual_style.as_ref().and_then(|style| {
            if style == &active_style {
                Some(true)
            } else if style == &inactive_style {
                Some(false)
            } else {
                None
            }
        });
        let matched = found.occurrences == 1
            && found.x == Some(plan.x)
            && found.y == Some(plan.y)
            && found.attribute_uniform
            && observed_selected == Some(plan.selected)
            && actual_style.as_ref() == Some(&expected_style);
        let problem = if found.occurrences == 0 {
            Some(format!("label {:?} is missing from the workspace list", plan.label))
        } else if found.occurrences != 1 {
            Some(format!(
                "label {:?} is ambiguous in the workspace list ({} occurrences)",
                plan.label, found.occurrences
            ))
        } else if found.x != Some(plan.x) || found.y != Some(plan.y) {
            Some(format!(
                "label {:?} moved to {:?}, expected ({}, {})",
                plan.label, (found.x, found.y), plan.x, plan.y
            ))
        } else if !found.attribute_uniform {
            Some(format!("label {:?} has mixed cell attributes", plan.label))
        } else if !matched {
            Some(format!(
                "label {:?} selected state was {:?}, expected {}",
                plan.label,
                observed_selected,
                if plan.selected { "selected" } else { "unselected" }
            ))
        } else {
            None
        };
        observations.push(SelectionObservationEvidence {
            label: plan.label.clone(),
            x: plan.x,
            y: plan.y,
            expected_selected: plan.selected,
            observed_selected,
            matched,
            occurrences: found.occurrences,
            actual_x: found.x,
            actual_y: found.y,
            attribute_uniform: found.attribute_uniform,
            expected_style,
            actual_style,
            reference_files: reference_files.clone(),
            problem,
        });
    }
    Ok(observations)
}

fn read_selection_reference(
    expected_root: &Path,
    journey: &Journey,
    checkpoint: &str,
    platform: &str,
) -> Result<(String, String, SelectionSample)> {
    ensure!(
        is_safe_single_component(&journey.name)
            && is_safe_single_component(checkpoint)
            && is_safe_single_component(platform),
        "unsafe selection reference path component"
    );
    let relative = PathBuf::from("termrock")
        .join(&journey.name)
        .join(checkpoint)
        .join(format!("pixels-{platform}.tui.json"));
    let path = expected_root.join(&relative);
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("inspect frozen selection reference {}", path.display()))?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_file(),
        "selection reference is not a regular file: {}",
        path.display()
    );
    let bytes = fs::read(&path)
        .with_context(|| format!("read frozen selection reference {}", path.display()))?;
    let sample: SelectionSample = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse frozen selection reference {}", path.display()))?;
    let expected_test_id = format!(
        "termrock/{}/{}/pixels-{platform}",
        journey.name, checkpoint
    );
    ensure!(
        sample.test_id == expected_test_id,
        "selection reference test ID mismatch: expected {expected_test_id:?}, found {:?}",
        sample.test_id
    );
    let relative = relative.to_string_lossy().replace('\\', "/");
    Ok((relative, sha256_hex(&bytes), sample))
}

fn require_reference_label(
    snapshot: &SelectionSnapshot,
    label: &str,
    expected_x: u16,
    expected_y: u16,
) -> Result<SelectionStyleEvidence> {
    ensure!(
        snapshot.cols == 120
            && snapshot.rows == 40
            && snapshot.cells.len() == usize::from(snapshot.cols) * usize::from(snapshot.rows),
        "frozen selection reference has an unexpected terminal shape"
    );
    let found = locate_selection_label(snapshot, label);
    ensure!(
        found.occurrences == 1
            && found.x == Some(expected_x)
            && found.y == Some(expected_y)
            && found.attribute_uniform,
        "frozen selection reference label {label:?} is missing, ambiguous, misplaced, or has mixed attributes"
    );
    found
        .style
        .context("frozen selection reference label has no cell style")
}

fn locate_selection_label(snapshot: &SelectionSnapshot, label: &str) -> SelectionLabelMatch {
    const WORKSPACE_LIST_COLUMNS: usize = 40;
    let empty = SelectionLabelMatch {
        occurrences: 0,
        x: None,
        y: None,
        style: None,
        attribute_uniform: false,
    };
    if label.is_empty()
        || !label.is_ascii()
        || snapshot.cols == 0
        || snapshot.rows == 0
        || snapshot.cells.len() != usize::from(snapshot.cols) * usize::from(snapshot.rows)
    {
        return empty;
    }
    let label_chars = label.chars().map(|character| character.to_string()).collect::<Vec<_>>();
    let columns = usize::from(snapshot.cols);
    let region_columns = columns.min(WORKSPACE_LIST_COLUMNS);
    if label_chars.len() > region_columns {
        return empty;
    }
    let mut found = empty;
    for y in 0..usize::from(snapshot.rows) {
        for x in 0..=region_columns - label_chars.len() {
            let start = y * columns + x;
            let span = &snapshot.cells[start..start + label_chars.len()];
            if !span
                .iter()
                .zip(&label_chars)
                .all(|(cell, character)| cell.text == *character)
            {
                continue;
            }
            found.occurrences += 1;
            if found.occurrences == 1 {
                let style = selection_style(&span[0]);
                found.x = Some(x as u16);
                found.y = Some(y as u16);
                found.attribute_uniform = span
                    .iter()
                    .all(|cell| selection_style(cell) == style);
                found.style = Some(style);
            }
        }
    }
    if found.occurrences != 1 {
        found.x = None;
        found.y = None;
        found.style = None;
        found.attribute_uniform = false;
    }
    found
}

fn selection_style(cell: &SelectionCell) -> SelectionStyleEvidence {
    SelectionStyleEvidence {
        fg: cell.fg.clone(),
        bg: cell.bg.clone(),
        bold: cell.bold,
        inverse: cell.inverse,
    }
}

fn partial_sample_evidence(samples_dir: &Path, phase: &str) -> Result<Option<(String, String)>> {
    let sample_name = format!("{phase}.sample.json");
    let sample_path = samples_dir.join(&sample_name);
    ensure_direct_child(samples_dir, &sample_path)?;
    let metadata = match fs::symlink_metadata(&sample_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("inspect partial sample file"),
    };
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_file(),
        "partial sample path is not a regular file: {}",
        sample_path.display()
    );
    let bytes = fs::read(&sample_path).context("read partial sample file")?;
    Ok(Some((format!("samples/{sample_name}"), sha256_hex(&bytes))))
}

fn cursor_evidence(snapshot: &tui_test::TerminalSnapshot) -> CursorEvidence {
    let cursor = snapshot.cursor();
    CursorEvidence {
        col: cursor.col(),
        row: cursor.row(),
        in_capture: cursor.in_capture(),
        visible: cursor.visible(),
        shape: format!("{:?}", cursor.shape()),
        blink: cursor.blink(),
        color: format!("{:?}", cursor.color()),
    }
}

fn cell_evidence(snapshot: &tui_test::TerminalSnapshot) -> Vec<CellEvidence> {
    snapshot
        .cells()
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let cols = snapshot.cols() as usize;
            CellEvidence {
                col: (index % cols) as u32,
                row: (index / cols) as u32,
                text: cell.text().to_string(),
                width: cell.width(),
                continuation: cell.is_continuation(),
                foreground: format!("{:?}", cell.fg()),
                background: format!("{:?}", cell.bg()),
                bold: cell.has(Attrs::BOLD),
                dim: cell.has(Attrs::DIM),
                italic: cell.has(Attrs::ITALIC),
                inverse: cell.has(Attrs::INVERSE),
                invisible: cell.has(Attrs::INVISIBLE),
                strike: cell.has(Attrs::STRIKE),
                blink: cell.has(Attrs::BLINK),
                underline: format!("{:?}", cell.underline()),
                underline_color: format!("{:?}", cell.underline_color()),
                link_uri: cell.link_uri().map(str::to_string),
                link_id: cell.link_id().map(str::to_string),
            }
        })
        .collect()
}

fn image_evidence(image: &SampleImage) -> ImageEvidence {
    ImageEvidence {
        width: image.width(),
        height: image.height(),
        png_sha256: image.png_sha256().to_string(),
        rgba_sha256: image.rgba_sha256().to_string(),
    }
}

fn render_contract_evidence(contract: &RenderContract) -> RenderContractEvidence {
    RenderContractEvidence {
        profile: contract.profile().to_string(),
        renderer_version: contract.renderer_version().to_string(),
        raster_policy: contract.raster_policy().to_string(),
        fonts: contract.fonts().iter().map(font_evidence).collect(),
        palette_digest: contract.palette_digest().to_string(),
        cell_width: contract.cell_width(),
        cell_height: contract.cell_height(),
        font_size: contract.font_size(),
        baseline: contract.baseline(),
        scale: contract.scale(),
        cursor_phase: format!("{:?}", contract.cursor_phase()),
        cursor_blink: contract.blink(),
    }
}

fn font_evidence(font: &ContractFont) -> FontEvidence {
    FontEvidence {
        sha256: font.digest().to_string(),
        face_index: font.face_index(),
        bold: font.bold(),
        italic: font.italic(),
        fallback_position: font.fallback_position(),
    }
}

fn comparison_evidence(
    outcome: CheckOutcome,
    expected_test_id: &str,
    gate: &str,
    expected_candidate_sha256: &str,
    compared_pixels: u64,
) -> Result<ComparisonEvidence> {
    match outcome {
        CheckOutcome::Pass(success) => {
            ensure!(
                gate == "state-and-pixels",
                "passing comparison did not use the state-and-pixels gate: {gate:?}"
            );
            ensure!(
                success.test_id() == expected_test_id,
                "passing comparison returned unexpected test ID {:?}",
                success.test_id()
            );
            ensure!(
                success.candidate_sha256() == expected_candidate_sha256,
                "passing comparison candidate digest differs from the saved sample"
            );
            Ok(ComparisonEvidence {
                test_id: success.test_id().to_string(),
                gate: gate.to_string(),
                outcome: "pass".to_string(),
                passed: true,
                reason: None,
                failure_kind: None,
                baseline_sha256: Some(success.baseline_sha256().to_string()),
                candidate_sha256: success.candidate_sha256().to_string(),
                changed_fields: Vec::new(),
                first_changed_cell: None,
                changed_cell_total: Some(0),
                pixel_comparison: "pixel-equal".to_string(),
                pixel_changed: Some(0),
                pixel_compared: Some(compared_pixels),
                pixel_bounds: None,
                report_path: None,
            })
        }
        CheckOutcome::Failure(failure) => {
            ensure!(
                failure.test_id() == expected_test_id,
                "failing comparison returned unexpected test ID {:?}",
                failure.test_id()
            );
            ensure!(
                failure.gate().as_str() == gate,
                "failing comparison gate differs from the submitted sample"
            );
            ensure!(
                failure.candidate_sha256() == expected_candidate_sha256,
                "failing comparison candidate digest differs from the saved sample"
            );
            Ok(failure_evidence(&failure))
        }
    }
}

fn failure_evidence(failure: &CheckFailure) -> ComparisonEvidence {
    let reason = failure.reason();
    let first_changed_cell = failure
        .changed_cells()
        .first()
        .map(|cell| ChangedCellEvidence {
            col: cell.col(),
            row: cell.row(),
            fields: cell.fields().to_vec(),
            actual_text: cell.actual_text().to_string(),
            baseline_text: cell.baseline_text().to_string(),
        });
    ComparisonEvidence {
        test_id: failure.test_id().to_string(),
        gate: failure.gate().as_str().to_string(),
        outcome: "failure".to_string(),
        passed: false,
        reason: Some(reason.as_str().to_string()),
        failure_kind: Some(match reason {
            MismatchReason::Missing => ComparisonFailureKind::MissingBaseline,
            MismatchReason::TestId => ComparisonFailureKind::TestId,
            MismatchReason::Policy => ComparisonFailureKind::Policy,
            MismatchReason::State => ComparisonFailureKind::State,
            MismatchReason::Contract => ComparisonFailureKind::Contract,
            MismatchReason::Dimensions => ComparisonFailureKind::Dimensions,
            MismatchReason::Pixels => ComparisonFailureKind::Pixels,
        }),
        baseline_sha256: Some(failure.baseline_sha256().to_string()),
        candidate_sha256: failure.candidate_sha256().to_string(),
        changed_fields: failure.changed_fields().to_vec(),
        first_changed_cell,
        changed_cell_total: Some(failure.changed_cell_total()),
        pixel_comparison: failure.pixel_comparison().as_str().to_string(),
        pixel_changed: Some(failure.pixel_changed()),
        pixel_compared: Some(failure.pixel_compared()),
        pixel_bounds: failure.pixel_bounds(),
        report_path: Some(failure.report_path().to_string_lossy().into_owned()),
    }
}

fn expected_terminal_modes() -> BTreeMap<String, bool> {
    [
        ("application_cursor_keys".to_string(), false),
        ("application_keypad".to_string(), false),
        ("origin".to_string(), false),
        ("wraparound".to_string(), true),
        ("insert".to_string(), false),
        ("focus_events".to_string(), false),
        ("bracketed_paste".to_string(), false),
        ("alternate_screen".to_string(), false),
        ("cursor_visible".to_string(), true),
    ]
    .into_iter()
    .collect()
}

fn terminal_restoration_evidence(
    actual_modes: &BTreeMap<String, bool>,
    actual_mouse_mode: &str,
) -> TerminalRestorationEvidence {
    let expected_modes = expected_terminal_modes();
    let expected_mouse_mode = "none".to_string();
    let mut errors = Vec::new();
    if *actual_modes != expected_modes {
        errors.push(format!(
            "terminal modes were not restored: expected {expected_modes:?}, found {actual_modes:?}"
        ));
    }
    if actual_mouse_mode != expected_mouse_mode {
        errors.push(format!(
            "mouse mode was not restored: expected {expected_mouse_mode:?}, found {actual_mouse_mode:?}"
        ));
    }
    TerminalRestorationEvidence {
        restored: errors.is_empty(),
        expected_modes,
        expected_mouse_mode,
        errors,
    }
}

fn ensure_disjoint_roots(baseline_root: &Path, output_root: &Path) -> Result<()> {
    let baseline = baseline_root
        .canonicalize()
        .with_context(|| format!("canonicalize baseline root {}", baseline_root.display()))?;
    let output = output_root
        .canonicalize()
        .with_context(|| format!("canonicalize output root {}", output_root.display()))?;
    ensure!(
        baseline != output && !baseline.starts_with(&output) && !output.starts_with(&baseline),
        "baseline and output roots must be disjoint: baseline={}, output={}",
        baseline.display(),
        output.display()
    );
    Ok(())
}

fn ensure_output_disjoint_from_expected(output_root: &Path) -> Result<()> {
    let suite_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .context("canonicalize functional suite root")?;
    let expected_root = suite_root.join("expected");
    match expected_root.canonicalize() {
        Ok(expected_root) => ensure_disjoint_roots(&expected_root, output_root),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let output = output_root
                .canonicalize()
                .context("canonicalize output root while expected store is absent")?;
            ensure!(
                output != expected_root
                    && !output.starts_with(&expected_root)
                    && !expected_root.starts_with(&output),
                "expected store and output roots must be disjoint: expected={}, output={}",
                expected_root.display(),
                output.display()
            );
            Ok(())
        }
        Err(error) => Err(error).context("canonicalize expected frame store before case setup"),
    }
}

fn record_execution_step(errors: &mut Vec<String>, label: &str, result: Result<()>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => {
            errors.push(format!("{label} failed: {error:#}"));
            false
        }
    }
}

fn press_required(session: &Session, key: Option<&str>, label: &str) -> Result<()> {
    let key = key.ok_or_else(|| anyhow::anyhow!("validated {label} key is missing"))?;
    press(session, key)
}

fn native_action_kind(action: &NativeAction) -> &'static str {
    match action {
        NativeAction::Key { .. } => "key",
        NativeAction::Write { .. } => "write",
        NativeAction::Resize { .. } => "resize",
        NativeAction::MouseClick { .. } => "mouse_click",
    }
}

fn settle(session: &Session) -> Result<()> {
    let wait = WaitQuietOptions::new(SETTLE_TIMEOUT, SETTLE_QUIET)
        .context("construct visual quiet wait")?;
    session
        .wait_quiet(&wait)
        .context("wait for visual screen stability")?;
    Ok(())
}

fn required_checkpoint_anchors(journey_name: &str, phase: &str) -> Result<&'static [&'static str]> {
    match (journey_name, phase) {
        ("first_showcase", "opened" | "returned") => Ok(&["Design system", "Overview"]),
        ("first_showcase", "help") => Ok(&["Keyboard & mouse"]),
        ("first_holla", "opened" | "returned") => {
            Ok(&["holla❯", "Here", "Search actions and resources"])
        }
        ("first_holla", "help") => Ok(&["Key reference"]),
        ("first_jackin", "opened" | "returned") => Ok(&["jackin❯", "Workspaces"]),
        ("first_jackin", "help") => Ok(&["Keyboard shortcuts"]),
        ("first_tablepro", "opened" | "returned") => {
            Ok(&["TablePro", "Connections", "Local PostgreSQL"])
        }
        ("first_tablepro", "help") => Ok(&["Keyboard shortcuts"]),
        _ => bail!("setup failure: no readiness anchors are admitted for {journey_name}/{phase}"),
    }
}

fn forbidden_checkpoint_anchors(
    journey_name: &str,
    phase: &str,
) -> Result<&'static [&'static str]> {
    match (journey_name, phase) {
        ("first_showcase", "opened" | "returned") => Ok(&["Keyboard & mouse"]),
        ("first_holla", "opened" | "returned") => Ok(&["Key reference"]),
        ("first_jackin" | "first_tablepro", "opened" | "returned") => Ok(&["Keyboard shortcuts"]),
        ("first_showcase" | "first_holla" | "first_jackin" | "first_tablepro", "help") => Ok(&[]),
        _ => bail!(
            "setup failure: no forbidden readiness anchors are admitted for {journey_name}/{phase}"
        ),
    }
}

fn missing_snapshot_anchors(
    snapshot: &tui_test::TerminalSnapshot,
    anchors: &[&str],
) -> Vec<String> {
    anchors
        .iter()
        .filter(|anchor| !snapshot_has_anchor(snapshot, anchor))
        .map(|anchor| (*anchor).to_string())
        .collect()
}

fn snapshot_has_anchor(snapshot: &tui_test::TerminalSnapshot, anchor: &str) -> bool {
    let columns = snapshot.cols() as usize;
    columns > 0
        && snapshot.cells().chunks(columns).any(|row| {
            let mut row_text = String::new();
            for cell in row {
                row_text.push_str(cell.text());
            }
            row_text.contains(anchor)
        })
}

fn press(session: &Session, key: &str) -> Result<()> {
    session
        .execute(Operation::Key {
            keys: vec![key.to_string()],
            action: KeyAction::Press,
        })
        .with_context(|| format!("press native key {key:?}"))?;
    Ok(())
}

fn validate_manifest(manifest: &SuiteManifest, dataset: &FixtureDataset) -> Result<()> {
    ensure!(
        manifest.version.schema_version == 3,
        "unsupported suite manifest schema"
    );
    ensure!(
        manifest.version.suite_id == "termrock-functional-suite-v3"
            && manifest.version.predecessor_suite_id == "termrock-functional-suite-v2"
            && !manifest.version.result_inheritance,
        "unexpected suite ID"
    );
    ensure!(
        !manifest.version.crosswalk.is_empty()
            && manifest
                .version
                .crosswalk
                .iter()
                .all(|entry| !entry.trim().is_empty()),
        "suite v3 crosswalk must contain nonempty revision notes"
    );
    ensure!(
        manifest.cases.len() == 89,
        "functional inventory must contain 89 cases"
    );
    let query_execution = manifest
        .cases
        .iter()
        .find(|case| case.name == "tablepro_query_execution")
        .context("functional inventory lacks tablepro_query_execution")?;
    ensure!(
        manifest
            .cases
            .iter()
            .filter(|case| case.phases.is_some())
            .count()
            == 1
            && query_execution.binary == "tablepro"
            && query_execution.case_id.is_none()
            && same_strings(
                &query_execution.arguments,
                &["--connect", "Local PostgreSQL"]
            )
            && query_execution.frame.is_none()
            && query_execution.exit_key.is_none()
            && query_execution.steps.is_none()
            && query_execution.native_steps.is_none()
            && same_strings(
                &query_execution.coverage_ids,
                &[
                    "suite.case.tablepro_query_execution",
                    "tablepro.screen.workbenchdefault",
                    "tablepro.screen.queryediting",
                    "tablepro.screen.completionpopup",
                    "tablepro.screen.resultsgrid",
                    "tablepro.screen.errorresult",
                    "tablepro.screen.explainplan",
                    "tablepro.behavior.completion",
                    "interaction.error",
                ],
            ),
        "TablePro query execution identity, inputs, or coverage changed"
    );
    ensure_supported_plan(query_execution)?;
    ensure!(
        query_execution.phases.as_ref().is_some_and(|phases| {
            phases
                .iter()
                .flat_map(|phase| phase.steps.iter().map(|step| step.name.as_str()))
                .eq([
                    "paused_workbench",
                    "paused_query_input",
                    "paused_running",
                    "paused_cancelled",
                    "paused_quit_dialog",
                    "complete_workbench",
                    "completion_popup",
                    "completion_accepted",
                    "completed_rows",
                    "completed_explain",
                    "batch_input",
                    "batch_error",
                    "retained_customers",
                    "retained_orders",
                    "execution_history",
                    "complete_quit_dialog",
                ])
        }),
        "TablePro query execution must retain its ordered 16-checkpoint phase inventory"
    );
    let actual_legacy = manifest.cases[..LEGACY_CASES.len()]
        .iter()
        .map(|case| case.name.as_str())
        .collect::<Vec<_>>();
    ensure!(
        actual_legacy.as_slice() == LEGACY_CASES.as_slice(),
        "legacy functional case order or denominator changed: {actual_legacy:?}"
    );
    ensure!(
        manifest.cases[LEGACY_CASES.len()..]
            .iter()
            .map(|case| case.name.as_str())
            .eq([
                "first_delivery_showcase_buttons",
                "first_delivery_holla_first_use",
                "first_delivery_jackin_settings",
                "first_delivery_tablepro_connections",
                "full_holla_first_use_frame1",
            ]),
        "delivery case order or denominator changed"
    );
    ensure!(
        !manifest.suite.coverage_scope.trim().is_empty(),
        "coverage scope must not be empty"
    );

    let exact_first_cases = [
        (
            "first_showcase",
            "showcase",
            &["--page", "overview"][..],
            "?",
            "Esc",
            "q",
        ),
        (
            "first_holla",
            "holla",
            &["--scenario", "first-use"][..],
            "F1",
            "Esc",
            "Ctrl+q",
        ),
        (
            "first_jackin",
            "jackin-preview",
            &["--scenario", "returning"][..],
            "?",
            "Esc",
            "q",
        ),
        ("first_tablepro", "tablepro", &[][..], "?", "Esc", "q"),
    ];
    for (case, (name, binary, arguments, help_key, return_key, exit_key)) in manifest
        .cases
        .iter()
        .take(LEGACY_FIRST_CASES.len())
        .zip(exact_first_cases)
    {
        ensure!(
            case.name == name && case.binary == binary && same_strings(&case.arguments, arguments),
            "first-priority journey command changed for {name}"
        );
        ensure!(
            case.help_key.as_deref() == Some(help_key)
                && case.return_key.as_deref() == Some(return_key)
                && case.exit_key.as_deref() == Some(exit_key),
            "first-priority journey keys changed for {name}"
        );
        ensure!(
            same_strings(
                case.native_steps.as_deref().unwrap_or_default(),
                &["help", "return", "exit"]
            ),
            "first-priority native steps changed for {name}"
        );
    }

    let expected_delivery = [
        (
            "first_delivery_showcase_buttons",
            "showcase.buttons",
            "showcase",
            &["--page", "buttons"][..],
            "q",
        ),
        (
            "first_delivery_holla_first_use",
            "holla.first_use",
            "holla",
            &["--scenario", "first-use"][..],
            "Ctrl+q",
        ),
        (
            "first_delivery_jackin_settings",
            "jackin.settings",
            "jackin-preview",
            &["--scenario", "returning"][..],
            "q",
        ),
        (
            "first_delivery_tablepro_connections",
            "tablepro.connections",
            "tablepro",
            &[][..],
            "Ctrl+q",
        ),
    ];
    for (case, (name, id, binary, arguments, exit_key)) in manifest.cases
        [LEGACY_CASES.len()..LEGACY_CASES.len() + DELIVERY_CASES.len()]
        .iter()
        .zip(expected_delivery)
    {
        ensure!(
            case.name == name
                && case.case_id.as_deref() == Some(id)
                && case.binary == binary
                && same_strings(&case.arguments, arguments)
                && case.exit_key.as_deref() == Some(exit_key)
                && case.steps.is_some()
                && case.frame.is_none(),
            "first-delivery command or identity changed for {name}"
        );
    }
    let expected_steps: [(&str, Vec<(&str, Vec<&str>)>); 4] = [
        (
            "first_delivery_showcase_buttons",
            vec![
                ("initial", vec![]),
                ("run", vec!["mouse_click:left:30:10"]),
                ("preview", vec!["key:Tab", "key:Enter"]),
                ("help", vec!["key:?"]),
                ("returned", vec!["key:Esc"]),
                ("small", vec!["resize:80:24"]),
                ("restored", vec!["resize:120:40"]),
            ],
        ),
        (
            "first_delivery_holla_first_use",
            vec![
                ("initial", vec![]),
                ("empty", vec!["write:zzzz-no-such-action"]),
                ("cleared", vec!["key:Esc"]),
                ("help", vec!["key:F1"]),
                ("returned", vec!["key:Esc"]),
                ("small", vec!["resize:80:24"]),
                ("restored", vec!["resize:120:40"]),
            ],
        ),
        (
            "first_delivery_jackin_settings",
            vec![
                ("initial", vec![]),
                ("selected", vec!["key:Down"]),
                ("general", vec!["key:s"]),
                ("mounts", vec!["key:]"]),
                ("general_return", vec!["key:["]),
                ("manager_return", vec!["key:Esc"]),
                ("help", vec!["key:?"]),
                ("help_return", vec!["key:Esc"]),
                ("small", vec!["resize:80:24"]),
                ("restored", vec!["resize:120:40"]),
            ],
        ),
        (
            "first_delivery_tablepro_connections",
            vec![
                ("initial", vec![]),
                ("selected", vec!["key:Down"]),
                ("help", vec!["key:F1"]),
                ("returned", vec!["key:Esc"]),
                ("small", vec!["resize:80:24"]),
                ("restored", vec!["resize:120:40"]),
            ],
        ),
    ];
    for (case, (name, expected)) in manifest.cases
        [LEGACY_CASES.len()..LEGACY_CASES.len() + DELIVERY_CASES.len()]
        .iter()
        .zip(expected_steps)
    {
        ensure!(case.name == name, "delivery plan order changed for {name}");
        let steps = case.steps.as_deref().unwrap_or_default();
        ensure!(
            steps.len() == expected.len(),
            "checkpoint count changed for {name}"
        );
        let help_heading = match case.binary.as_str() {
            "showcase" => "Keyboard & mouse",
            "holla" => "Key reference",
            "jackin-preview" | "tablepro" => "Keyboard shortcuts",
            _ => bail!("unsupported delivery executable {}", case.binary),
        };
        for (step, (expected_name, expected_actions)) in steps.iter().zip(expected) {
            let actual_actions = step
                .actions
                .iter()
                .map(action_signature)
                .collect::<Vec<_>>();
            ensure!(
                step.name == expected_name
                    && actual_actions
                        == expected_actions
                            .into_iter()
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                "native action sequence changed for {name}/{}",
                step.name
            );
            let expected_selection: &[(&str, u16, u16, bool)] = match (name, step.name.as_str()) {
                ("first_delivery_jackin_settings", "initial") => &[
                    ("Current directory", 7, 3, true),
                    ("payments-platform", 7, 4, false),
                ],
                ("first_delivery_jackin_settings", "selected") => &[
                    ("Current directory", 7, 3, false),
                    ("payments-platform", 7, 4, true),
                ],
                _ => &[],
            };
            ensure!(
                step.selection_observations.len() == expected_selection.len()
                    && step
                        .selection_observations
                        .iter()
                        .zip(expected_selection)
                        .all(|(actual, (label, x, y, selected))| {
                            actual.label == *label
                                && actual.x == *x
                                && actual.y == *y
                                && actual.selected == *selected
                        }),
                "selection observation plan changed for {name}/{}",
                step.name
            );
            if step.name == "help" {
                ensure!(
                    step.required_anchors
                        .iter()
                        .any(|anchor| anchor == help_heading)
                        && !step
                            .forbidden_anchors
                            .iter()
                            .any(|anchor| anchor == help_heading),
                    "help heading readiness changed for {name}"
                );
                if case.binary == "jackin-preview" {
                    ensure!(
                        step.required_anchors
                            .iter()
                            .any(|anchor| anchor == "Workspaces")
                            && step
                                .required_anchors
                                .iter()
                                .any(|anchor| anchor == "Instances"),
                        "Jackin help headings are missing for {name}"
                    );
                }
            } else {
                ensure!(
                    step.forbidden_anchors
                        .iter()
                        .any(|anchor| anchor == help_heading)
                        && !step
                            .required_anchors
                            .iter()
                            .any(|anchor| anchor == help_heading),
                    "returned/readiness help heading exclusion changed for {name}/{}",
                    step.name
                );
            }
            if step.name == "selected" {
                let selected_anchor = match case.binary.as_str() {
                    "jackin-preview" => None,
                    "tablepro" => Some("Local PostgreSQL"),
                    _ => bail!("unexpected selected checkpoint for {}", case.binary),
                };
                if let Some(selected_anchor) = selected_anchor {
                    ensure!(
                        step.required_anchors
                            .iter()
                            .any(|anchor| anchor == selected_anchor),
                        "selected checkpoint lacks visible row content for {name}"
                    );
                } else {
                    ensure!(
                        step.selection_observations.len() == 2,
                        "Jackin selected row must be asserted after capture for {name}"
                    );
                }
            }
            ensure!(
                (step.name == "small" && step.expected_cols == 80 && step.expected_rows == 24)
                    || (step.name != "small"
                        && step.expected_cols == 120
                        && step.expected_rows == 40),
                "checkpoint terminal dimensions changed for {name}/{}",
                step.name
            );
        }
    }
    let holla_delivery = &manifest.cases[LEGACY_CASES.len() + 1];
    let empty_step = holla_delivery
        .steps
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|step| step.name == "empty")
        .context("Holla delivery plan lacks empty checkpoint")?;
    ensure!(
        empty_step
            .required_anchors
            .iter()
            .any(|anchor| anchor == "No matches for “zzzz-no-such-action”")
            && empty_step
                .required_anchors
                .iter()
                .any(|anchor| anchor == "Nothing to preview"),
        "Holla must expose the exact query's empty-results and empty-preview state"
    );
    let delivery_steps = manifest.cases
        [LEGACY_CASES.len()..LEGACY_CASES.len() + DELIVERY_CASES.len()]
        .iter()
        .flat_map(|case| case.steps.as_deref().unwrap_or_default())
        .count();
    ensure!(
        DELIVERY_CASES.len() == 4 && delivery_steps == DELIVERY_FRAME_TOTAL,
        "first delivery subset must contain exactly four cases and 30 frame checkpoints"
    );
    let frame_one = manifest
        .cases
        .last()
        .context("missing Holla frame-1 variant")?;
    let first_delivery_holla_steps = manifest
        .cases
        .iter()
        .find(|case| case.name == "first_delivery_holla_first_use")
        .and_then(|case| case.steps.as_deref())
        .context("missing Holla first-delivery checkpoint plan")?;
    ensure!(
        frame_one.name == "full_holla_first_use_frame1"
            && frame_one.case_id.as_deref() == Some("holla.first_use.frame1")
            && frame_one.binary == "holla"
            && same_strings(&frame_one.arguments, &["--scenario", "first-use"])
            && frame_one.exit_key.as_deref() == Some("Ctrl+q")
            && frame_one.frame == Some(1)
            && frame_one.steps.as_ref().is_some_and(|steps| {
                steps.len() == 7
                    && steps.len() == first_delivery_holla_steps.len()
                    && steps
                        .iter()
                        .zip(first_delivery_holla_steps)
                        .all(|(variant, first)| {
                            variant.name == first.name
                                && variant.required_anchors == first.required_anchors
                                && variant.forbidden_anchors == first.forbidden_anchors
                                && variant.expected_cols == first.expected_cols
                                && variant.expected_rows == first.expected_rows
                                && variant
                                    .actions
                                    .iter()
                                    .map(action_signature)
                                    .eq(first.actions.iter().map(action_signature))
                        })
            }),
        "Holla frame-1 variant must repeat the full first-use journey outside the first subset"
    );

    let mut coverage_ids = BTreeSet::new();
    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        let expected_binary = if case.name == "first_showcase" || case.name.starts_with("showcase_")
        {
            Some("showcase")
        } else if case.name == "first_holla" || case.name.starts_with("holla_") {
            Some("holla")
        } else if case.name == "first_jackin" || case.name.starts_with("jackin_") {
            Some("jackin-preview")
        } else if case.name == "first_tablepro" || case.name.starts_with("tablepro_") {
            Some("tablepro")
        } else if case.name.starts_with("first_delivery_showcase_") {
            Some("showcase")
        } else if case.name.starts_with("first_delivery_holla_")
            || case.name == "full_holla_first_use_frame1"
        {
            Some("holla")
        } else if case.name.starts_with("first_delivery_jackin_") {
            Some("jackin-preview")
        } else if case.name.starts_with("first_delivery_tablepro_") {
            Some("tablepro")
        } else {
            None
        };
        ensure!(
            expected_binary == Some(case.binary.as_str()),
            "unexpected executable for case {}: {}",
            case.name,
            case.binary
        );
        let case_id = case_id(case);
        ensure!(
            is_safe_single_component(case_id) && case_ids.insert(case_id),
            "invalid or duplicate case ID for {}: {case_id}",
            case.name
        );
        if case.steps.is_none() {
            ensure!(
                case.coverage_ids
                    .contains(&format!("suite.case.{}", case.name)),
                "legacy case {} lacks its stable suite.case coverage ID",
                case.name
            );
        }
        for coverage_id in &case.coverage_ids {
            ensure!(
                is_safe_single_component(coverage_id),
                "unsafe coverage ID in case {}: {coverage_id:?}",
                case.name
            );
            ensure!(
                coverage_ids.insert(coverage_id.as_str()),
                "duplicate coverage ID: {coverage_id}"
            );
        }
    }
    ensure!(
        coverage_ids.len() == EXPECTED_COVERAGE_ID_COUNT,
        "coverage inventory changed: expected {} IDs, found {}",
        EXPECTED_COVERAGE_ID_COUNT,
        coverage_ids.len()
    );
    let owners = manifest
        .required_coverage
        .get("coverage_id_owners")
        .and_then(toml::Value::as_table)
        .context("required coverage lacks coverage_id_owners")?;
    ensure!(
        owners.len() == EXPECTED_COVERAGE_ID_COUNT
            && owners.keys().map(String::as_str).collect::<BTreeSet<_>>() == coverage_ids,
        "coverage ID owner map differs from case coverage IDs"
    );
    let inventory_names = manifest
        .required_coverage
        .get("journey_ids")
        .and_then(toml::Value::as_array)
        .context("required coverage lacks journey_ids")?
        .iter()
        .map(|value| value.as_str().context("journey ID is not a string"))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        inventory_names.len() == 89
            && inventory_names
                .iter()
                .copied()
                .eq(manifest.cases.iter().map(|case| case.name.as_str())),
        "coverage journey inventory differs from the 89 registered cases"
    );
    ensure!(
        manifest.suite.terminal.cols == 120 && manifest.suite.terminal.rows == 40,
        "functional terminal size must be 120x40"
    );
    ensure!(
        manifest.suite.terminal.locale == "C.UTF-8",
        "functional locale changed"
    );
    ensure!(
        manifest.suite.terminal.timezone == "UTC",
        "functional timezone changed"
    );
    ensure!(
        manifest.suite.terminal.term == "xterm-256color",
        "functional TERM changed"
    );
    ensure!(
        manifest.suite.terminal.color_term == "truecolor",
        "functional COLORTERM changed"
    );
    ensure!(
        manifest.suite.theme == "default" && manifest.suite.no_color == "unset",
        "functional default theme or NO_COLOR contract changed"
    );
    ensure!(
        manifest.suite.readiness_timeout_ms == READINESS_TIMEOUT.as_millis() as u64
            && manifest.suite.readiness_poll_interval_ms
                == READINESS_POLL_INTERVAL.as_millis() as u64
            && manifest.suite.settle_timeout_ms == SETTLE_TIMEOUT.as_millis() as u64
            && manifest.suite.settle_quiet_ms == SETTLE_QUIET.as_millis() as u64,
        "visible readiness or quiet-settle timing changed"
    );
    ensure!(
        same_strings(
            &manifest.suite.terminal.motion_arguments,
            &["--motion", "paused", "--frame", "0"]
        ),
        "motion arguments changed"
    );
    ensure!(
        same_strings(
            &manifest.suite.terminal.color_mode_arguments,
            &["--color", "truecolor"]
        ),
        "color arguments changed"
    );
    ensure!(
        manifest.suite.render_profile == PROFILE_NAME,
        "render profile is not the pinned TuiScotti profile"
    );
    ensure!(
        manifest.suite.fixture_dataset == "fixtures/dataset.json",
        "unexpected fixture dataset path"
    );
    ensure!(
        manifest.suite.expected_store == "expected",
        "unexpected expected-store path"
    );
    ensure!(
        dataset.schema_version == 1,
        "unsupported fixture dataset schema"
    );
    ensure!(
        dataset.dataset_id == "empty-user-state-v1",
        "unexpected fixture dataset ID"
    );
    ensure!(
        manifest
            .engine_pins
            .get("tuiscotti-tui-test")
            .map(String::as_str)
            == Some("7abeac8478aa9c05ec7bef93afb10209cc348bec")
            && manifest.engine_pins.get("tuiscotti").map(String::as_str)
                == Some("d8d54155947f6303a41dfd6079b53c0abc16f29a"),
        "pinned native capture engine revisions changed"
    );
    ensure!(
        !dataset.description.is_empty(),
        "fixture dataset description is empty"
    );
    ensure!(
        dataset.owner_marker_contents.ends_with('\n'),
        "fixture owner marker must be line-terminated"
    );
    for name in [
        &dataset.sandbox_root,
        &dataset.work_directory,
        &dataset.home_directory,
        &dataset.data_directory,
        &dataset.config_directory,
        &dataset.cache_directory,
        &dataset.state_directory,
        &dataset.temporary_directory,
        &dataset.owner_marker,
    ] {
        ensure!(
            is_safe_single_component(name),
            "unsafe fixture path component {name:?}"
        );
    }
    Ok(())
}

fn validate_fault_control(
    fault_control: &FaultControlEvidence,
    journey: &Journey,
) -> Result<bool> {
    if !fault_control.active {
        ensure!(
            fault_control.spec.is_none(),
            "inactive input control must not carry a specification"
        );
        return Ok(false);
    }

    ensure!(
        fault_control.spec.as_deref()
            == Some("drop:jackin.settings:selected:1"),
        "setup failure: malformed or unsupported TERMROCK_PARITY_INPUT_CONTROL"
    );
    ensure!(
        journey.name == "first_delivery_jackin_settings"
            && case_id(journey) == "jackin.settings"
            && journey.phases.is_none(),
        "setup failure: Jackin selection control is only valid for first_delivery_jackin_settings"
    );
    let selected = journey
        .steps
        .as_deref()
        .and_then(|steps| steps.iter().find(|step| step.name == "selected"))
        .context("setup failure: controlled Jackin case lacks selected checkpoint")?;
    ensure!(
        selected.actions.len() == 1
            && matches!(selected.actions.first(), Some(NativeAction::Key { key }) if key == "Down"),
        "setup failure: controlled Jackin selected checkpoint must have exactly one Down action"
    );
    Ok(true)
}

fn same_strings(actual: &[String], expected: &[&str]) -> bool {
    actual
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
}

fn action_signature(action: &NativeAction) -> String {
    match action {
        NativeAction::Key { key } => format!("key:{key}"),
        NativeAction::Write { text } => format!("write:{text}"),
        NativeAction::Resize { cols, rows } => format!("resize:{cols}:{rows}"),
        NativeAction::MouseClick { x, y, button } => {
            format!("mouse_click:{button}:{x}:{y}")
        }
    }
}

fn required_directory(variable: &str) -> Result<PathBuf> {
    let raw = env::var_os(variable)
        .with_context(|| format!("required environment variable {variable} is unset"))?;
    let path = PathBuf::from(raw);
    ensure!(
        path.is_absolute(),
        "{variable} must be an absolute path: {}",
        path.display()
    );
    ensure!(
        path.exists(),
        "{variable} does not exist: {}",
        path.display()
    );
    ensure!(
        path.is_dir(),
        "{variable} is not a directory: {}",
        path.display()
    );
    Ok(path
        .canonicalize()
        .with_context(|| format!("canonicalize {variable}"))?)
}

fn safe_manifest_path(relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    ensure!(path.is_relative(), "manifest path must be relative");
    ensure!(
        path.components()
            .all(|part| matches!(part, std::path::Component::Normal(_))),
        "manifest path escapes its root: {relative:?}"
    );
    Ok(path.to_path_buf())
}

fn case_id(journey: &Journey) -> &str {
    journey.case_id.as_deref().unwrap_or(&journey.name)
}

fn prepare_fixture(
    dataset: &FixtureDataset,
    journey: &Journey,
    manifest: &SuiteManifest,
) -> Result<FixturePaths> {
    let temp_root = env::temp_dir()
        .canonicalize()
        .context("canonicalize platform temp directory")?;
    let case_id = case_id(journey);
    let lock_name = format!(".{}-{case_id}.case-lock", dataset.sandbox_root);
    ensure!(
        is_safe_single_component(&lock_name),
        "unsafe fixture lock name {lock_name:?}"
    );
    let lock_path = temp_root.join(lock_name);
    let case_lock = acquire_case_lock(&lock_path)?;
    let sandbox = temp_root.join(&dataset.sandbox_root);
    create_owned_root(
        &sandbox,
        &dataset.owner_marker,
        &dataset.owner_marker_contents,
    )?;
    verify_shared_root_entries(&sandbox, &dataset.owner_marker, &manifest.cases)?;
    let case_root = sandbox.join(case_id);
    create_case_root(
        &case_root,
        &dataset.owner_marker,
        &dataset.owner_marker_contents,
        case_id,
    )?;

    let owned = [
        dataset.work_directory.as_str(),
        dataset.home_directory.as_str(),
        dataset.data_directory.as_str(),
        dataset.config_directory.as_str(),
        dataset.cache_directory.as_str(),
        dataset.state_directory.as_str(),
        dataset.temporary_directory.as_str(),
    ];
    for entry in fs::read_dir(&case_root).context("list owned fixture case directory")? {
        let entry = entry?;
        let name = entry.file_name();
        if name == std::ffi::OsStr::new(&dataset.owner_marker) {
            continue;
        }
        let name = name
            .to_str()
            .context("fixture case directory contains a non-UTF-8 entry name")?;
        ensure!(
            owned.contains(&name),
            "refuse to reset unknown fixture entry {}",
            entry.path().display()
        );
        let metadata = fs::symlink_metadata(entry.path()).context("inspect fixture entry")?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "refuse to reset symlink fixture entry {}",
            entry.path().display()
        );
        ensure!(
            metadata.is_dir(),
            "refuse to reset non-directory fixture entry {}",
            entry.path().display()
        );
        fs::remove_dir_all(entry.path()).context("reset owned fixture directory")?;
    }

    let mut paths = BTreeMap::new();
    for (name, directory) in [
        ("cwd", &dataset.work_directory),
        ("home", &dataset.home_directory),
        ("data", &dataset.data_directory),
        ("config", &dataset.config_directory),
        ("cache", &dataset.cache_directory),
        ("state", &dataset.state_directory),
        ("tmp", &dataset.temporary_directory),
    ] {
        let path = case_root.join(directory);
        ensure_direct_child(&case_root, &path)?;
        fs::create_dir(&path).with_context(|| format!("create fixture {name} directory"))?;
        paths.insert(name, path);
    }
    let cwd = paths.get("cwd").context("missing fixture cwd")?.clone();
    let env = vec![
        ("LANG".to_string(), "C.UTF-8".to_string()),
        ("LC_ALL".to_string(), "C.UTF-8".to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("TERM".to_string(), "xterm-256color".to_string()),
        ("COLORTERM".to_string(), "truecolor".to_string()),
        (
            "PATH".to_string(),
            "/usr/bin:/bin:/usr/sbin:/sbin".to_string(),
        ),
        ("SHELL".to_string(), "/bin/sh".to_string()),
        (
            "HOME".to_string(),
            paths.get("home").unwrap().to_string_lossy().into_owned(),
        ),
        (
            "XDG_DATA_HOME".to_string(),
            paths.get("data").unwrap().to_string_lossy().into_owned(),
        ),
        (
            "XDG_CONFIG_HOME".to_string(),
            paths.get("config").unwrap().to_string_lossy().into_owned(),
        ),
        (
            "XDG_CACHE_HOME".to_string(),
            paths.get("cache").unwrap().to_string_lossy().into_owned(),
        ),
        (
            "XDG_STATE_HOME".to_string(),
            paths.get("state").unwrap().to_string_lossy().into_owned(),
        ),
        (
            "TMPDIR".to_string(),
            paths.get("tmp").unwrap().to_string_lossy().into_owned(),
        ),
    ];
    Ok(FixturePaths {
        cwd,
        env,
        _case_lock: case_lock,
    })
}

fn create_owned_root(root: &Path, marker_name: &str, marker_contents: &str) -> Result<()> {
    let created = match fs::create_dir(root) {
        Ok(()) => true,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => false,
        Err(error) => {
            return Err(error).with_context(|| format!("create fixture root {}", root.display()));
        }
    };
    let metadata = fs::symlink_metadata(root).context("inspect fixture root")?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_dir(),
        "fixture root is not a real directory: {}",
        root.display()
    );
    if created {
        publish_owner_marker(root, marker_name, marker_contents)?;
        ensure!(
            owner_marker_matches(root, marker_name, marker_contents)?,
            "fixture root creator did not publish its owner marker: {}",
            root.display()
        );
    } else {
        wait_for_owner_marker(root, marker_name, marker_contents)?;
    }
    Ok(())
}

fn create_case_root(
    root: &Path,
    marker_name: &str,
    marker_contents: &str,
    case_name: &str,
) -> Result<()> {
    ensure!(
        is_safe_single_component(case_name),
        "unsafe case directory name"
    );
    let created = match fs::create_dir(root) {
        Ok(()) => true,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => false,
        Err(error) => {
            return Err(error)
                .with_context(|| format!("create fixture case root {}", root.display()));
        }
    };
    let metadata = fs::symlink_metadata(root).context("inspect fixture case root")?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_dir(),
        "fixture case root is not a real directory: {}",
        root.display()
    );
    if created {
        publish_owner_marker(root, marker_name, marker_contents)?;
        ensure!(
            owner_marker_matches(root, marker_name, marker_contents)?,
            "fixture case creator did not publish its owner marker: {}",
            root.display()
        );
    } else {
        wait_for_owner_marker(root, marker_name, marker_contents)?;
    }
    Ok(())
}

fn publish_owner_marker(root: &Path, name: &str, contents: &str) -> Result<()> {
    let marker = root.join(name);
    ensure_direct_child(root, &marker)?;
    let temporary_root = env::temp_dir()
        .canonicalize()
        .context("canonicalize platform temp directory for owner marker")?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before Unix epoch")?
        .as_nanos();
    let temporary = temporary_root.join(format!(
        ".termrock-owner-marker-{}-{nonce}",
        std::process::id()
    ));
    ensure_direct_child(&temporary_root, &temporary)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .context("create temporary fixture owner marker")?;
    if let Err(error) = file
        .write_all(contents.as_bytes())
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(error).context("write temporary fixture owner marker");
    }
    drop(file);
    if let Err(error) = fs::hard_link(&temporary, &marker) {
        let _ = fs::remove_file(&temporary);
        return Err(error).context("atomically publish fixture owner marker");
    }
    fs::remove_file(&temporary).context("remove temporary fixture owner marker")?;
    Ok(())
}

fn owner_marker_matches(root: &Path, name: &str, contents: &str) -> Result<bool> {
    let marker = root.join(name);
    ensure_direct_child(root, &marker)?;
    let metadata = match fs::symlink_metadata(&marker) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error).context("inspect fixture owner marker"),
    };
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_file(),
        "fixture owner marker is not a regular file: {}",
        marker.display()
    );
    let existing = fs::read(&marker).context("read fixture owner marker")?;
    ensure!(
        existing == contents.as_bytes(),
        "fixture owner marker contents differ: {}",
        marker.display()
    );
    Ok(true)
}

fn wait_for_owner_marker(root: &Path, name: &str, contents: &str) -> Result<()> {
    let deadline = Instant::now() + OWNER_MARKER_TIMEOUT;
    loop {
        if owner_marker_matches(root, name, contents)? {
            return Ok(());
        }
        let now = Instant::now();
        if now >= deadline {
            bail!("existing fixture directory is unowned: {}", root.display());
        }
        thread::sleep(OWNER_MARKER_POLL_INTERVAL.min(deadline.saturating_duration_since(now)));
    }
}

fn verify_shared_root_entries(root: &Path, marker_name: &str, cases: &[Journey]) -> Result<()> {
    let expected_case_ids = cases.iter().map(case_id).collect::<BTreeSet<_>>();
    for entry in fs::read_dir(root).context("list shared fixture root")? {
        let entry = entry?;
        let name = entry.file_name();
        if name == std::ffi::OsStr::new(marker_name) {
            continue;
        }
        let name = name
            .to_str()
            .context("shared fixture root contains a non-UTF-8 entry name")?;
        ensure!(
            expected_case_ids.contains(name),
            "refuse unknown shared fixture entry {}",
            entry.path().display()
        );
        let metadata =
            fs::symlink_metadata(entry.path()).context("inspect shared fixture entry")?;
        ensure!(
            !metadata.file_type().is_symlink() && metadata.is_dir(),
            "shared fixture case entry is not a real directory: {}",
            entry.path().display()
        );
    }
    Ok(())
}

fn acquire_case_lock(path: &Path) -> Result<File> {
    let parent = path
        .parent()
        .context("fixture lock path has no parent directory")?;
    ensure_direct_child(parent, path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => ensure!(
            !metadata.file_type().is_symlink() && metadata.is_file(),
            "fixture case lock is not a regular file: {}",
            path.display()
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("inspect fixture case lock"),
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .context("open persistent fixture case lock")?;
    let path_metadata =
        fs::symlink_metadata(path).context("verify persistent fixture case lock")?;
    let file_metadata = lock
        .metadata()
        .context("inspect opened fixture case lock")?;
    ensure!(
        !path_metadata.file_type().is_symlink()
            && path_metadata.is_file()
            && file_metadata.is_file(),
        "fixture case lock is not a regular file: {}",
        path.display()
    );

    let deadline = Instant::now() + CASE_LOCK_TIMEOUT;
    loop {
        match lock.try_lock() {
            Ok(()) => return Ok(lock),
            Err(std::fs::TryLockError::WouldBlock) => {
                let now = Instant::now();
                if now >= deadline {
                    bail!(
                        "timed out after {:?} waiting for fixture case lock {}",
                        CASE_LOCK_TIMEOUT,
                        path.display()
                    );
                }
                thread::sleep(CASE_LOCK_POLL_INTERVAL.min(deadline.saturating_duration_since(now)));
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(error).context("acquire persistent fixture case lock");
            }
        }
    }
}

fn prepare_case_output(root: &Path, case_name: &str) -> Result<PathBuf> {
    ensure!(
        is_safe_single_component(case_name),
        "unsafe output case name"
    );
    let version_root = ensure_child_dir(root, "v3")?;
    let path = version_root.join(case_name);
    if !path.exists() {
        fs::create_dir(&path).with_context(|| format!("create case output {}", path.display()))?;
    }
    let metadata = fs::symlink_metadata(&path).context("inspect case output directory")?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_dir(),
        "case output is not a real directory: {}",
        path.display()
    );
    let canonical = path
        .canonicalize()
        .context("canonicalize case output directory")?;
    ensure_direct_child(&version_root, &canonical)?;
    Ok(canonical)
}

fn ensure_child_dir(parent: &Path, name: &str) -> Result<PathBuf> {
    ensure!(
        is_safe_single_component(name),
        "unsafe output child name {name:?}"
    );
    let path = parent.join(name);
    match fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("create output directory {}", path.display()));
        }
    }
    let metadata = fs::symlink_metadata(&path).context("inspect output child directory")?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_dir(),
        "output child is not a real directory: {}",
        path.display()
    );
    ensure_direct_child(parent, &path)?;
    Ok(path)
}

fn write_atomic(directory: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    ensure!(
        is_safe_single_component(name),
        "unsafe output filename {name:?}"
    );
    let target = directory.join(name);
    ensure_direct_child(directory, &target)?;
    if let Ok(metadata) = fs::symlink_metadata(&target) {
        ensure!(
            !metadata.file_type().is_symlink() && metadata.is_file(),
            "refuse to replace non-file output {}",
            target.display()
        );
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before Unix epoch")?
        .as_nanos();
    let temp = directory.join(format!(".{name}.tmp-{}-{nonce}", std::process::id()));
    ensure_direct_child(directory, &temp)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .context("create temporary evidence file")?;
    if let Err(error) = file
        .write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&temp);
        return Err(error).context("write temporary evidence file");
    }
    drop(file);
    fs::rename(&temp, &target).context("atomically publish evidence file")?;
    Ok(())
}

fn ensure_direct_child(parent: &Path, child: &Path) -> Result<()> {
    let parent = parent
        .canonicalize()
        .with_context(|| format!("canonicalize parent {}", parent.display()))?;
    ensure!(
        child.parent() == Some(parent.as_path()),
        "path is not a direct child: {}",
        child.display()
    );
    Ok(())
}

fn is_safe_single_component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn validate_run_id(value: &str) -> Option<String> {
    (value.len() <= 128 && is_safe_single_component(value)).then(|| value.to_string())
}

fn utc_timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let shifted_days = days + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days - 146_096
    }
    .div_euclid(146_097);
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{:09}Z",
        elapsed.subsec_nanos()
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
