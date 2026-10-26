use std::{collections::HashSet, fmt, sync::Arc};

use crate::sim::{
    DiskScanController, FileKind, FilesSnapshot, ScanNode, ScanNodeIssue, ScanRequest,
    ScanSaturation, ScanSource, ScanStatus, World,
};
use termrock::{
    Button, Cx, EmptyState, FgStep, FrameRead, Id, ItemKey, Rect, Response, Role, StylePatch,
    TextSection, TextViewport, Tree, TreeAction, TreeNode, TreeState, Ui, ViewportLine,
    ViewportState,
};

pub(crate) const TREE: Id = Id::root("holla.disk-scan.tree");
pub(crate) const RESCAN: Id = Id::root("holla.disk-scan.rescan");
pub(crate) const CANCEL: Id = Id::root("holla.disk-scan.cancel");
pub(crate) const SORT: Id = Id::root("holla.disk-scan.sort");
pub(crate) const BACK: Id = Id::root("holla.disk-scan.back");
pub(crate) const DETAILS: Id = Id::root("holla.disk-scan.details");

const REVEAL_DURATION_TICKS: u64 = 48;
const BATCH_LIMIT: usize = 4;
const TICK_MILLIS: i64 = 80;
const EXHAUSTED_PROJECTION: u64 = u64::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortBy {
    Allocated,
    Apparent,
}

impl SortBy {
    fn toggled(self) -> Self {
        match self {
            Self::Allocated => Self::Apparent,
            Self::Apparent => Self::Allocated,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Allocated => "Allocated",
            Self::Apparent => "Apparent",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorldContext {
    host_name: String,
    host_kind: crate::domain::host::HostKind,
    environment: crate::domain::host::Environment,
    cwd: String,
    effect_revision: u64,
}

impl WorldContext {
    fn capture(world: &World) -> Self {
        Self {
            host_name: world.host.name.clone(),
            host_kind: world.host.kind,
            environment: world.host.env,
            cwd: world.cwd.clone(),
            effect_revision: world.effect_revision,
        }
    }

    fn matches(&self, world: &World) -> bool {
        self.host_name == world.host.name
            && self.host_kind == world.host.kind
            && self.environment == world.host.env
            && self.cwd == world.cwd
            && self.effect_revision == world.effect_revision
    }
}

struct Binding {
    generation: u64,
    root: String,
    snapshot: Arc<FilesSnapshot>,
    context: WorldContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    path: String,
    node: TreeNode,
    label: String,
    kind: FileKind,
    allocated_bytes: Option<u64>,
    apparent_bytes: Option<u64>,
    modified_at_secs: Option<i64>,
    entries: u64,
    issues: Vec<ScanNodeIssue>,
    saturation: ScanSaturation,
}

impl fmt::Display for Row {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Owners {
    tree: Id,
    rescan: Id,
    cancel: Id,
    sort: Id,
    back: Id,
    details: Id,
}

impl Owners {
    fn new(session: u64, generation: u64, projection: u64) -> Self {
        let bind = |owner: Id| {
            owner
                .item(ItemKey::pair(session, generation))
                .item(ItemKey::num(projection))
        };
        Self {
            tree: bind(TREE),
            rescan: bind(RESCAN),
            cancel: bind(CANCEL),
            sort: bind(SORT),
            back: bind(BACK),
            details: bind(DETAILS),
        }
    }

    const fn unbound() -> Self {
        Self {
            tree: TREE,
            rescan: RESCAN,
            cancel: CANCEL,
            sort: SORT,
            back: BACK,
            details: DETAILS,
        }
    }

    fn all(self) -> [Id; 6] {
        [
            self.tree,
            self.rescan,
            self.cancel,
            self.sort,
            self.back,
            self.details,
        ]
    }

    fn focus_pairs(self, next: Self) -> [(Id, Id); 6] {
        [
            (self.tree, next.tree),
            (self.rescan, next.rescan),
            (self.cancel, next.cancel),
            (self.sort, next.sort),
            (self.back, next.back),
            (self.details, next.details),
        ]
    }
}

#[derive(Clone, Copy)]
struct Areas {
    title: Rect,
    status: Rect,
    rescan: Rect,
    cancel: Rect,
    sort: Rect,
    back: Rect,
    tree: Rect,
    details: Rect,
}

pub(crate) struct State {
    controller: DiskScanController,
    binding: Option<Binding>,
    session: u64,
    projection: u64,
    retired_owners: Vec<Owners>,
    tree: TreeState,
    sort: SortBy,
    rows: Vec<Row>,
    detail_lines: Vec<String>,
    details: ViewportState,
    status: Option<ScanStatus>,
    revealed_count: usize,
    modeled_path_count: usize,
    notice: Option<String>,
    stale: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            controller: DiskScanController::new(),
            binding: None,
            session: 0,
            projection: 0,
            retired_owners: Vec::new(),
            tree: TreeState::default(),
            sort: SortBy::Allocated,
            rows: Vec::new(),
            detail_lines: Vec::new(),
            details: ViewportState::default(),
            status: None,
            revealed_count: 0,
            modeled_path_count: 0,
            notice: None,
            stale: false,
        }
    }
}

impl State {
    pub(crate) fn open(&mut self, world: &World) -> Result<u64, String> {
        let root = map_cwd(&world.cwd)?;
        let snapshot = Arc::new(crate::sim::fixtures::files_for_cwd(&root));
        self.start_snapshot(
            root,
            snapshot,
            world,
            world.now_ms(),
            REVEAL_DURATION_TICKS,
            BATCH_LIMIT,
        )
    }

    fn start_snapshot(
        &mut self,
        root: String,
        snapshot: Arc<FilesSnapshot>,
        world: &World,
        now_ms: i64,
        reveal_duration: u64,
        batch_limit: usize,
    ) -> Result<u64, String> {
        let next_session = self
            .session
            .checked_add(1)
            .ok_or_else(|| "scan session identity exhausted".to_owned())?;
        let request = ScanRequest::new(
            root.clone(),
            simulation_tick(now_ms),
            reveal_duration,
            batch_limit,
        );
        let generation = self
            .controller
            .start(Arc::clone(&snapshot), request)
            .map_err(|error| error.to_string())?;
        self.retire_current_owners();
        self.session = next_session;
        self.projection = 0;
        self.binding = Some(Binding {
            generation,
            root,
            snapshot,
            context: WorldContext::capture(world),
        });
        self.stale = false;
        self.status = Some(ScanStatus::Scanning);
        self.revealed_count = 0;
        self.modeled_path_count = 0;
        self.notice = None;
        self.tree.invalidate();
        self.refresh_projection_at(simulation_tick(now_ms))?;
        Ok(generation)
    }

    fn rescan(&mut self, world: &World, now_ms: i64) -> Result<u64, String> {
        if self.projection == EXHAUSTED_PROJECTION {
            return Err("presentation identity exhausted".to_owned());
        }
        let Some(binding) = self.binding.as_ref() else {
            return Err("No modeled scan is open".to_owned());
        };
        if self.stale || !binding.context.matches(world) {
            self.mark_stale();
            return Err("World context changed; open a new modeled scan".to_owned());
        }
        let root = binding.root.clone();
        let snapshot = Arc::clone(&binding.snapshot);
        let context = binding.context.clone();
        let request = ScanRequest::new(
            root.clone(),
            simulation_tick(now_ms),
            REVEAL_DURATION_TICKS,
            BATCH_LIMIT,
        );
        let generation = self
            .controller
            .start(Arc::clone(&snapshot), request)
            .map_err(|error| error.to_string())?;
        self.retire_current_owners();
        self.projection = 0;
        self.binding = Some(Binding {
            generation,
            root,
            snapshot,
            context,
        });
        self.status = Some(ScanStatus::Scanning);
        self.revealed_count = 0;
        self.modeled_path_count = 0;
        self.notice = None;
        self.tree.invalidate();
        self.refresh_projection_at(simulation_tick(now_ms))?;
        Ok(generation)
    }

    fn cancel(&mut self, now_ms: i64) -> Result<(), String> {
        if self.projection == EXHAUSTED_PROJECTION {
            return Err("presentation identity exhausted".to_owned());
        }
        let Some(binding) = self.binding.as_ref() else {
            return Err("No modeled scan is open".to_owned());
        };
        self.controller
            .cancel(binding.generation, simulation_tick(now_ms))
            .map_err(|error| error.to_string())?;
        self.refresh_projection_at(simulation_tick(now_ms))
    }

    pub(crate) fn leave(&mut self, world: &World) {
        if self.projection == EXHAUSTED_PROJECTION {
            if let Some(binding) = self.binding.as_ref()
                && self.status == Some(ScanStatus::Scanning)
                && self.controller.current_generation() == Some(binding.generation)
                && self
                    .controller
                    .cancel(binding.generation, simulation_tick(world.now_ms()))
                    .is_ok()
            {
                self.status = Some(ScanStatus::Cancelled);
            }
            self.retire_current_owners();
            self.tree.invalidate();
            return;
        }
        let Some(binding) = self.binding.as_ref() else {
            self.retire_current_owners();
            let _ = self.advance_projection();
            self.tree.invalidate();
            return;
        };
        let generation = binding.generation;
        if self.status == Some(ScanStatus::Scanning)
            && self.controller.current_generation() == Some(generation)
            && self
                .controller
                .cancel(generation, simulation_tick(world.now_ms()))
                .is_ok()
        {
            self.notice = Some("Scan paused when this page closed".to_owned());
            let _ = self.refresh_projection_at(simulation_tick(world.now_ms()));
        }
        if self.projection == EXHAUSTED_PROJECTION {
            self.tree.invalidate();
            return;
        }
        self.retire_current_owners();
        let _ = self.advance_projection();
        self.tree.invalidate();
    }

    pub(crate) fn cancel_after_route_change(&mut self, world: &World) {
        if self.projection == EXHAUSTED_PROJECTION {
            if let Some(binding) = self.binding.as_ref()
                && self.status == Some(ScanStatus::Scanning)
                && self.controller.current_generation() == Some(binding.generation)
                && self
                    .controller
                    .cancel(binding.generation, simulation_tick(world.now_ms()))
                    .is_ok()
            {
                self.status = Some(ScanStatus::Cancelled);
            }
            self.retire_current_owners();
            self.tree.invalidate();
            return;
        }
        let Some(binding) = self.binding.as_ref() else {
            self.tree.invalidate();
            return;
        };
        if self.status == Some(ScanStatus::Scanning)
            && self.controller.current_generation() == Some(binding.generation)
            && self
                .controller
                .cancel(binding.generation, simulation_tick(world.now_ms()))
                .is_ok()
        {
            self.status = Some(ScanStatus::Cancelled);
        }
        self.retire_current_owners();
        let _ = self.advance_projection();
        self.tree.invalidate();
    }

    pub(crate) fn update(
        &mut self,
        world: &World,
        cx: &mut Cx<'_>,
        body: Rect,
        active: bool,
    ) -> (Response<()>, Option<Event>) {
        if !active {
            self.settle_owners(cx);
            return (Response::ignored(), None);
        }
        if self.binding.is_none() {
            self.settle_owners(cx);
            return (Response::ignored(), None);
        }
        if self.projection == EXHAUSTED_PROJECTION {
            self.refresh_details();
            return self.update_back_only(cx, body);
        }
        if !self.context_is_current(world) {
            let previous = self.owners();
            let became_stale = !self.stale;
            self.mark_stale();
            if self.projection == EXHAUSTED_PROJECTION {
                self.refresh_details();
                return self.update_back_only_with_response(
                    cx,
                    body,
                    if became_stale {
                        Response::changed()
                    } else {
                        Response::ignored()
                    },
                    became_stale,
                );
            }
            self.transfer_focus(cx, previous, self.owners());
            self.refresh_details();
            return self.update_back_only_with_response(
                cx,
                body,
                if became_stale {
                    Response::changed()
                } else {
                    Response::ignored()
                },
                became_stale,
            );
        }

        let previous = self.owners();
        let tick = simulation_tick(world.now_ms());
        if let Err(error) = self.refresh_projection_at(tick) {
            if self.projection != EXHAUSTED_PROJECTION {
                self.mark_stale();
                self.notice = Some(format!("Modeled scan is stale · {error}"));
            }
            if self.projection == EXHAUSTED_PROJECTION {
                self.refresh_details();
                return self.update_back_only_with_response(cx, body, Response::changed(), true);
            }
            self.transfer_focus(cx, previous, self.owners());
            self.refresh_details();
            return self.update_back_only_with_response(cx, body, Response::changed(), true);
        }
        self.transfer_focus(cx, previous, self.owners());

        let owners = self.owners().unwrap_or_else(Owners::unbound);
        let areas = areas(body, self.sort);
        if cx.update_cause() != termrock::UpdateCause::Event || !draw_witness(cx, owners, areas) {
            self.settle_owners(cx);
            return (Response::ignored(), None);
        }
        self.settle_retired_owners(cx);

        let tree = tree_widget(owners.tree, false);
        let mut tree_response = tree.update(cx, &mut self.tree, &self.rows);
        let tree_action = tree_response.take_action();
        let mut response = tree_response.erase();
        if matches!(
            tree_action,
            Some(
                TreeAction::Moved
                    | TreeAction::Chose(_)
                    | TreeAction::Activated(_)
                    | TreeAction::Expanded(_)
                    | TreeAction::Collapsed(_)
            )
        ) {
            if self.advance_projection().is_err() {
                self.refresh_details();
                return self.update_back_only_with_response(
                    cx,
                    body,
                    response | Response::changed(),
                    true,
                );
            }
            self.refresh_details();
            if let Some(owners) = self.owners() {
                cx.focus(owners.tree);
            }
        }

        let rescan_response = Button::new(owners.rescan, "Rescan")
            .disabled(self.stale)
            .update(cx);
        let rescan_activated = rescan_response.activated();
        let cancel_response = Button::new(owners.cancel, "Cancel")
            .disabled(self.stale || self.status != Some(ScanStatus::Scanning))
            .update(cx);
        let cancel_activated = cancel_response.activated();
        let sort_response = Button::new(owners.sort, self.sort.label())
            .disabled(self.stale || self.binding.is_none())
            .update(cx);
        let sort_activated = sort_response.activated();
        let back_response = Button::new(owners.back, "Back").update(cx);
        let back_activated = back_response.activated();
        let details_lines = viewport_lines(&self.detail_lines);
        let details_response = detail_viewport(owners.details)
            .update(cx, &mut self.details, &details_lines)
            .erase();
        response |= rescan_response.erase();
        response |= cancel_response.erase();
        response |= sort_response.erase();
        response |= back_response.erase();
        response |= details_response;

        if rescan_activated {
            if let Err(error) = self.rescan(world, world.now_ms()) {
                if self.projection == EXHAUSTED_PROJECTION {
                    self.refresh_details();
                    return self.update_back_only_with_response(
                        cx,
                        body,
                        response | Response::changed(),
                        true,
                    );
                }
                self.notice = Some(format!("Rescan failed · {error}"));
                self.refresh_details();
            }
            if let Some(owners) = self.owners() {
                cx.focus(owners.rescan);
            }
            response |= Response::changed();
        }
        if cancel_activated {
            let previous = self.owners();
            if let Err(error) = self.cancel(world.now_ms()) {
                if self.projection == EXHAUSTED_PROJECTION {
                    self.refresh_details();
                    return self.update_back_only_with_response(
                        cx,
                        body,
                        response | Response::changed(),
                        true,
                    );
                }
                self.notice = Some(format!("Cancel failed · {error}"));
                self.refresh_details();
            }
            self.transfer_focus(cx, previous, self.owners());
            if let Some(owners) = self.owners() {
                cx.focus(owners.tree);
            }
            response |= Response::changed();
        }
        if sort_activated {
            let next_sort = self.sort.toggled();
            if let Err(error) = self.refresh_projection_with_sort(tick, next_sort) {
                if self.projection != EXHAUSTED_PROJECTION {
                    self.mark_stale();
                    self.notice = Some(format!("Modeled scan is stale · {error}"));
                }
                self.refresh_details();
                return self.update_back_only_with_response(
                    cx,
                    body,
                    response | Response::changed(),
                    true,
                );
            }
            if let Some(owners) = self.owners() {
                cx.focus(owners.sort);
            }
            response |= Response::changed();
        }
        let event = back_activated.then_some(Event::Back);
        (response, event)
    }

    fn update_back_only(&mut self, cx: &mut Cx<'_>, body: Rect) -> (Response<()>, Option<Event>) {
        self.update_back_only_with_response(cx, body, Response::ignored(), false)
    }

    fn update_back_only_with_response(
        &mut self,
        cx: &mut Cx<'_>,
        body: Rect,
        prior: Response<()>,
        changed: bool,
    ) -> (Response<()>, Option<Event>) {
        let owners = self.owners().unwrap_or_else(Owners::unbound);
        let areas = areas(body, self.sort);
        let mut response = prior;
        for owner in [
            owners.tree,
            owners.rescan,
            owners.cancel,
            owners.sort,
            owners.details,
        ] {
            settle_owner(cx, owner);
        }
        self.settle_retired_owners(cx);
        if cx.update_cause() == termrock::UpdateCause::Event
            && cx.area(owners.back) == Some(areas.back)
            && !areas.back.is_empty()
        {
            let back = Button::new(owners.back, "Back").update(cx);
            let activated = back.activated();
            response |= back.erase();
            if changed {
                response |= Response::changed();
            }
            return (response, activated.then_some(Event::Back));
        }
        settle_owner(cx, owners.back);
        if changed {
            response |= Response::changed();
        }
        (response, None)
    }

    fn settle_owners(&mut self, cx: &mut Cx<'_>) {
        if let Some(owners) = self.owners() {
            for owner in owners.all() {
                settle_owner(cx, owner);
            }
        }
        self.settle_retired_owners(cx);
    }

    fn settle_retired_owners(&mut self, cx: &mut Cx<'_>) {
        let retired = std::mem::take(&mut self.retired_owners);
        self.retired_owners = retired
            .into_iter()
            .filter(|owners| {
                for owner in owners.all() {
                    settle_owner(cx, owner);
                }

                owners
                    .all()
                    .into_iter()
                    .any(|owner| cx.area(owner).is_some())
            })
            .collect();
    }

    fn retire_current_owners(&mut self) {
        if let Some(owners) = self.owners()
            && !self.retired_owners.contains(&owners)
        {
            self.retired_owners.push(owners);
        }
    }

    fn advance_projection(&mut self) -> Result<(), String> {
        if self.projection == EXHAUSTED_PROJECTION {
            return Err("presentation identity exhausted".to_owned());
        }
        self.retire_current_owners();
        if self.projection == EXHAUSTED_PROJECTION - 1 {
            self.projection = EXHAUSTED_PROJECTION;
            self.stale = true;
            self.notice = Some("Presentation identity exhausted".to_owned());
            return Err("presentation identity exhausted".to_owned());
        }
        self.projection += 1;
        Ok(())
    }

    fn owners(&self) -> Option<Owners> {
        self.binding
            .as_ref()
            .map(|binding| Owners::new(self.session, binding.generation, self.projection))
    }

    fn transfer_focus(&self, cx: &mut Cx<'_>, previous: Option<Owners>, next: Option<Owners>) {
        let (Some(previous), Some(next)) = (previous, next) else {
            return;
        };
        if let Some((_, target)) = previous
            .focus_pairs(next)
            .into_iter()
            .find(|(owner, _)| cx.state(*owner).contains(termrock::StateFlags::FOCUSED))
        {
            cx.focus(target);
        }
    }

    pub(crate) fn tree_owner(&self) -> Option<Id> {
        self.owners().map(|owners| owners.tree)
    }

    fn refresh_projection_at(&mut self, tick: u64) -> Result<(), String> {
        self.refresh_projection_with_sort(tick, self.sort)
    }

    fn refresh_projection_with_sort(&mut self, tick: u64, sort: SortBy) -> Result<(), String> {
        if self.projection == EXHAUSTED_PROJECTION {
            return Err("presentation identity exhausted".to_owned());
        }
        let Some(binding) = self.binding.as_ref() else {
            self.rows.clear();
            self.detail_lines.clear();
            self.status = None;
            self.revealed_count = 0;
            self.modeled_path_count = 0;
            return Ok(());
        };
        let root = binding.root.clone();
        let generation = binding.generation;
        let (status, revealed_count, modeled_path_count, rows) = {
            let report = self
                .controller
                .report_at(generation, tick)
                .map_err(|error| error.to_string())?;
            if report.root().path != root || report.source() != ScanSource::ModeledFilesSnapshot {
                return Err("scan source or root changed".to_owned());
            }
            (
                report.status(),
                report.revealed_count(),
                report.reveal_order().len(),
                project_rows(report.root(), report.visible_paths(), sort),
            )
        };
        let rows_changed = self.rows != rows;
        let changed = self.sort != sort
            || self.status != Some(status)
            || self.revealed_count != revealed_count
            || self.modeled_path_count != modeled_path_count
            || rows_changed;
        if changed {
            self.advance_projection()?;
        }
        self.sort = sort;
        self.status = Some(status);
        self.revealed_count = revealed_count;
        self.modeled_path_count = modeled_path_count;
        self.replace_rows(rows);
        self.refresh_details();
        Ok(())
    }

    fn replace_rows(&mut self, rows: Vec<Row>) {
        if self.rows != rows {
            self.rows = rows;
            self.tree.invalidate();
            if self
                .tree
                .cursor()
                .is_none_or(|key| !self.rows.iter().any(|row| row_key(row) == key))
                && let Some(first) = self.rows.first()
            {
                self.tree.set_cursor(0, row_key(first));
            }
        }
    }

    fn context_is_current(&self, world: &World) -> bool {
        !self.stale
            && self.binding.as_ref().is_some_and(|binding| {
                binding.context.matches(world)
                    && self.controller.current_generation() == Some(binding.generation)
            })
    }

    fn mark_stale(&mut self) {
        if !self.stale {
            let _ = self.advance_projection();
        }
        self.stale = true;
        self.notice.get_or_insert_with(|| {
            "World or scan identity changed · return, then open a new scan".into()
        });
    }

    fn refresh_details(&mut self) {
        let Some(binding) = self.binding.as_ref() else {
            self.detail_lines = vec![
                "No modeled scan is open".to_owned(),
                "Open Modeled disk scan from Go".to_owned(),
            ];
            return;
        };
        let mut lines = vec![
            "Modeled snapshot".to_owned(),
            format!("Root: {}", binding.root),
            format!(
                "Host: {} · {} · {}",
                binding.context.host_name,
                binding.context.host_kind.label(),
                binding.context.environment.label()
            ),
            format!("World cwd: {}", binding.context.cwd),
            format!("Effect revision: {}", binding.context.effect_revision),
            format!("Generation: {}", binding.generation),
            format!("Sort: {} descending · path tie-break", self.sort.label()),
            format!("Status: {}", self.status_label()),
            format!(
                "Revealed: {} of {} modeled paths",
                self.revealed_count, self.modeled_path_count
            ),
        ];
        if let Some(notice) = &self.notice {
            lines.push(format!("Notice: {notice}"));
        }
        if self.stale {
            lines.push("Stale session · scan controls disabled".to_owned());
        }
        let selected = self
            .tree
            .cursor()
            .and_then(|key| self.rows.iter().find(|row| row_key(row) == key))
            .or_else(|| self.rows.first());
        if let Some(row) = selected {
            lines.extend([
                String::new(),
                format!("Inspected path: {}", row.path),
                format!("Kind: {}", file_kind_label(&row.kind)),
                "Full modeled subtree totals; scan may be partial".to_owned(),
                format!(
                    "Allocated: {}",
                    size_label(row.allocated_bytes, row.saturation.allocated_bytes)
                ),
                format!(
                    "Apparent: {}",
                    size_label(row.apparent_bytes, row.saturation.apparent_bytes)
                ),
                format!("Modeled entries: {}", row.entries),
                format!(
                    "Modified: {}",
                    row.modified_at_secs
                        .map_or_else(|| "unknown".to_owned(), |value| format!("{value} s"))
                ),
                format!("Issues: {}", issues_label(&row.issues)),
                format!("Saturation: {}", saturation_label(row.saturation)),
            ]);
        } else if let Some(binding) = self.binding.as_ref() {
            lines.push(format!("Waiting for reveal at {}", binding.root));
        }
        self.detail_lines = lines;
    }

    fn status_label(&self) -> &'static str {
        if self.projection == EXHAUSTED_PROJECTION {
            return "Presentation identity exhausted";
        }
        if self.stale {
            return "Stale";
        }
        if let Some(notice) = &self.notice
            && notice.starts_with("Rescan failed")
        {
            return "Previous scan retained";
        }
        match self.status {
            Some(ScanStatus::Scanning) => "Scanning",
            Some(ScanStatus::Complete) => "Complete",
            Some(ScanStatus::Cancelled) => "Cancelled",
            None => "Not started",
        }
    }
}

pub(crate) fn map_cwd(cwd: &str) -> Result<String, String> {
    if cwd.starts_with('/') {
        return Ok(cwd.to_owned());
    }
    if cwd == "~" {
        return Ok(crate::sim::fixtures::HOME.to_owned());
    }
    if let Some(suffix) = cwd.strip_prefix("~/") {
        if suffix.is_empty() {
            return Err("`~/` has no modeled suffix".to_owned());
        }
        return Ok(format!("{}/{suffix}", crate::sim::fixtures::HOME));
    }
    Err(format!(
        "{cwd}: expected an absolute path, `~`, or `~/suffix`"
    ))
}

fn simulation_tick(now_ms: i64) -> u64 {
    u64::try_from(now_ms.max(0)).unwrap_or_default() / u64::try_from(TICK_MILLIS).unwrap_or(1)
}

fn row_key(row: &Row) -> ItemKey {
    ItemKey::text(&row.path)
}

fn row_node(row: &Row) -> TreeNode {
    row.node.keyed(row_key(row))
}

fn tree_widget(id: Id, disabled: bool) -> Tree<'static, Row> {
    Tree::new(id)
        .node(&row_node)
        .disabled(disabled)
        .empty(EmptyState::Empty {
            title: "No revealed paths",
            hint: Some("The modeled scan reveals paths as simulation time advances"),
        })
}

fn detail_viewport(id: Id) -> TextViewport<'static> {
    TextViewport::new(id).wrap(true).reserve_gutter(false)
}

fn viewport_lines(lines: &[String]) -> Vec<ViewportLine<'_>> {
    lines
        .iter()
        .map(|line| ViewportLine::Plain(line.as_str()))
        .collect()
}

fn project_rows(root: &ScanNode, visible_paths: &[String], sort: SortBy) -> Vec<Row> {
    let visible: HashSet<&str> = visible_paths.iter().map(String::as_str).collect();
    let mut rows = Vec::with_capacity(visible_paths.len());
    project_node(root, 0, &visible, sort, &mut rows);
    rows
}

fn project_node(
    node: &ScanNode,
    depth: u16,
    visible: &HashSet<&str>,
    sort: SortBy,
    rows: &mut Vec<Row>,
) {
    if !visible.contains(node.path.as_str()) {
        return;
    }
    let mut children: Vec<&ScanNode> = node
        .children
        .iter()
        .filter(|child| visible.contains(child.path.as_str()))
        .collect();
    children.sort_by(|left, right| compare_siblings(left, right, sort));
    let node_kind = match (children.is_empty(), node.children.is_empty()) {
        (false, _) => TreeNode::parent(depth),
        (true, false) => TreeNode::lazy(depth),
        (true, true) => TreeNode::leaf(depth),
    };
    rows.push(Row {
        path: node.path.clone(),
        node: node_kind,
        label: row_label(node),
        kind: node.kind.clone(),
        allocated_bytes: node.allocated_bytes,
        apparent_bytes: node.apparent_bytes,
        modified_at_secs: node.modified_at_secs,
        entries: node.entries,
        issues: node.issues.clone(),
        saturation: node.saturation,
    });
    for child in children {
        project_node(child, depth.saturating_add(1), visible, sort, rows);
    }
}

fn compare_siblings(left: &ScanNode, right: &ScanNode, sort: SortBy) -> std::cmp::Ordering {
    let (left_size, right_size) = match sort {
        SortBy::Allocated => (left.allocated_bytes, right.allocated_bytes),
        SortBy::Apparent => (left.apparent_bytes, right.apparent_bytes),
    };
    match (left_size, right_size) {
        (Some(left_size), Some(right_size)) => right_size
            .cmp(&left_size)
            .then_with(|| left.path.cmp(&right.path)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.path.cmp(&right.path),
    }
}

fn row_label(node: &ScanNode) -> String {
    let kind = match &node.kind {
        FileKind::Directory => "▸",
        FileKind::Symlink { .. } => "↪",
        FileKind::File => "·",
        FileKind::Special => "◇",
    };
    let name = node.path.rsplit('/').next().unwrap_or(&node.path);
    format!("{kind} {}", safe_display_text(name))
}

fn safe_display_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                '�'
            } else {
                character
            }
        })
        .collect()
}

fn file_kind_label(kind: &FileKind) -> String {
    match kind {
        FileKind::Directory => "directory".to_owned(),
        FileKind::Symlink { target } => format!("symlink · target recorded as {target}"),
        FileKind::File => "file".to_owned(),
        FileKind::Special => "special".to_owned(),
    }
}

fn size_label(value: Option<u64>, saturated: bool) -> String {
    match (value, saturated) {
        (Some(value), true) => format!("at least {value} bytes · capped"),
        (Some(value), false) => format!("{value} bytes"),
        (None, true) => "unknown · known subtotal capped".to_owned(),
        (None, false) => "unknown".to_owned(),
    }
}

fn issue_label(issue: ScanNodeIssue) -> &'static str {
    match issue {
        ScanNodeIssue::MetadataUnavailable => "metadata unavailable",
        ScanNodeIssue::PermissionDenied => "permission denied",
        ScanNodeIssue::Dataless => "dataless",
        ScanNodeIssue::ListingUnavailable => "listing unavailable",
    }
}

fn issues_label(issues: &[ScanNodeIssue]) -> String {
    if issues.is_empty() {
        return "none".to_owned();
    }
    issues
        .iter()
        .copied()
        .map(issue_label)
        .collect::<Vec<_>>()
        .join(" · ")
}

fn saturation_label(saturation: ScanSaturation) -> String {
    let mut values = Vec::new();
    if saturation.allocated_bytes {
        values.push("allocated bytes");
    }
    if saturation.apparent_bytes {
        values.push("apparent bytes");
    }
    if saturation.entries {
        values.push("entry count");
    }
    if values.is_empty() {
        "none".to_owned()
    } else {
        format!("capped: {}", values.join(", "))
    }
}

fn areas(body: Rect, sort: SortBy) -> Areas {
    let x = body.x.saturating_add(1);
    let width = body.width.saturating_sub(2);
    let title = Rect::new(x, body.y, width, 1);
    let status = Rect::new(x, body.y.saturating_add(1), width, 1);
    let buttons_y = body.y.saturating_add(2);
    let labels = ["Rescan", "Cancel", sort.label(), "Back"];
    let widths = labels.map(|label| termrock::width(label).saturating_add(2));
    let mut button_x = x;
    let rescan = Rect::new(button_x, buttons_y, widths[0], 1);
    button_x = button_x.saturating_add(widths[0]).saturating_add(1);
    let cancel = Rect::new(button_x, buttons_y, widths[1], 1);
    button_x = button_x.saturating_add(widths[1]).saturating_add(1);
    let sort = Rect::new(button_x, buttons_y, widths[2], 1);
    button_x = button_x.saturating_add(widths[2]).saturating_add(1);
    let back = Rect::new(button_x, buttons_y, widths[3], 1);
    let content_y = body.y.saturating_add(4);
    let content_height = body.height.saturating_sub(4);
    let content_width = body.width.saturating_sub(2);
    let gap = 1;
    let details_width = content_width / 3;
    let tree_width = content_width
        .saturating_sub(details_width)
        .saturating_sub(gap);
    let tree = Rect::new(x, content_y, tree_width, content_height);
    let details = Rect::new(
        x.saturating_add(tree_width).saturating_add(gap),
        content_y,
        details_width,
        content_height,
    );
    Areas {
        title,
        status,
        rescan,
        cancel,
        sort,
        back,
        tree,
        details,
    }
}

fn draw_witness(cx: &Cx<'_>, owners: Owners, expected: Areas) -> bool {
    [
        (owners.tree, expected.tree),
        (owners.rescan, expected.rescan),
        (owners.cancel, expected.cancel),
        (owners.sort, expected.sort),
        (owners.back, expected.back),
        (owners.details, expected.details),
    ]
    .into_iter()
    .all(|(owner, area)| !area.is_empty() && cx.area(owner) == Some(area))
}

fn settle_owner(cx: &mut Cx<'_>, owner: Id) {
    let mut rejected = false;
    for intent in cx.intents(owner) {
        if !matches!(
            intent,
            termrock::Intent::FocusIn { .. }
                | termrock::Intent::FocusOut { .. }
                | termrock::Intent::Cancel
        ) {
            rejected = true;
        }
    }
    if rejected {
        cx.diagnostic(termrock::Diagnostic::UndeliveredIntent { owner });
    }
}

pub(crate) fn draw(state: &State, ui: &mut Ui<'_>, body: Rect) {
    if body.is_empty() {
        return;
    }
    let owners = state.owners().unwrap_or_else(Owners::unbound);
    let areas = areas(body, state.sort);
    let title_style = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(termrock::Modifier::BOLD),
    );
    let muted = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    TextSection::new("Modeled disk scan")
        .inset(0)
        .title_style(title_style)
        .render(ui, areas.title);
    let status = format!(
        "{} · {} · {} paths revealed",
        state.status_label(),
        if state.stale {
            "stale"
        } else {
            "Modeled snapshot"
        },
        state.revealed_count
    );
    TextSection::new(&status)
        .inset(0)
        .title_style(muted)
        .render(ui, areas.status);

    Button::new(owners.rescan, "Rescan")
        .disabled(state.stale || state.binding.is_none())
        .draw(ui, areas.rescan);
    Button::new(owners.cancel, "Cancel")
        .disabled(state.stale || state.status != Some(ScanStatus::Scanning))
        .draw(ui, areas.cancel);
    Button::new(owners.sort, state.sort.label())
        .disabled(state.stale || state.binding.is_none())
        .draw(ui, areas.sort);
    Button::new(owners.back, "Back").draw(ui, areas.back);

    tree_widget(owners.tree, state.stale).draw(ui, areas.tree, &state.tree, &state.rows);
    let lines = viewport_lines(&state.detail_lines);
    detail_viewport(owners.details).draw(ui, areas.details, &state.details, &lines);
}
