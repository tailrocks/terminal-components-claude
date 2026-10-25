//! Pure, deterministic disk-scan model over an immutable FilesSnapshot.
//!
//! This module never inspects the host filesystem, starts work on another
//! thread, or creates cleanup actions. It eagerly builds a modeled tree and
//! reveals its preorder at caller-supplied simulation ticks.

use crate::files::{FileEntry, FileKind, FilesError, FilesSnapshot};
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

/// Parameters for one modeled scan.
///
/// Times are simulation ticks, not wall-clock durations. A zero reveal
/// duration uses one effective tick, matching the historical minimum-one-tick
/// behavior. The caller supplies absolute simulated snapshot keys verbatim;
/// this model never canonicalizes or resolves them through the host OS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanRequest {
    /// Exact absolute key of an existing snapshot entry.
    pub root: String,
    /// Whether hidden direct children are included at each directory.
    pub include_hidden: bool,
    /// Absolute simulated subtree roots to omit, using path boundaries.
    pub skip_paths: Vec<String>,
    /// Simulation tick at which the first frame is reported.
    pub started_at: u64,
    /// Target reveal duration in ticks; zero is normalized to one.
    pub reveal_duration: u64,
    /// Maximum number of preorder paths revealed per positive tick.
    pub batch_limit: usize,
}

impl ScanRequest {
    /// Create a request that includes hidden entries and has no skipped paths.
    pub fn new(
        root: impl Into<String>,
        started_at: u64,
        reveal_duration: u64,
        batch_limit: usize,
    ) -> Self {
        Self {
            root: root.into(),
            include_hidden: true,
            skip_paths: Vec::new(),
            started_at,
            reveal_duration,
            batch_limit,
        }
    }
}

/// A modeled condition attached to a visible scan node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanNodeIssue {
    /// This entry has no explicit scan metadata; its byte totals are unknown.
    MetadataUnavailable,
    /// The modeled directory cannot be listed.
    PermissionDenied,
    /// The modeled directory is marked dataless and is not traversed.
    Dataless,
    /// The snapshot could not provide a structural listing for this node.
    ListingUnavailable,
}

/// Provenance for a numeric scan aggregate that was capped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScanSaturation {
    /// A known allocated-byte accumulation exceeded u64::MAX.
    pub allocated_bytes: bool,
    /// A known apparent-byte accumulation exceeded u64::MAX.
    pub apparent_bytes: bool,
    /// A known entry-count accumulation exceeded u64::MAX.
    pub entries: bool,
}

/// One node in the eagerly computed modeled scan tree.
///
/// None byte totals mean at least one required metadata value was absent. A
/// true saturation bit means the corresponding known subtotal was capped; if
/// the total is also None, some known contribution saturated while another
/// contribution remained unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanNode {
    /// Exact simulated snapshot key.
    pub path: String,
    /// Modeled node kind; symlink targets are recorded but never followed.
    pub kind: FileKind,
    /// Allocated-byte total, or None if required metadata is unavailable.
    pub allocated_bytes: Option<u64>,
    /// Apparent-byte total, or None if required metadata is unavailable.
    pub apparent_bytes: Option<u64>,
    /// Explicit modeled modification time, if supplied.
    pub modified_at_secs: Option<i64>,
    /// Count of non-directory path entries below this node, including self
    /// for a non-directory node.
    pub entries: u64,
    /// Partial-node conditions collected without hiding one another.
    pub issues: Vec<ScanNodeIssue>,
    /// Flags distinguishing capped aggregates from exact numeric maxima.
    pub saturation: ScanSaturation,
    /// Visible structural children in allocated-descending/path order.
    pub children: Vec<ScanNode>,
}

impl ScanNode {
    /// Find a node by its exact simulated snapshot key.
    pub fn find(&self, path: &str) -> Option<&ScanNode> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(path))
    }
}

/// Current state of a scan at a requested simulation tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanStatus {
    /// The reveal prefix has not reached the whole tree.
    Scanning,
    /// Every tree path has been revealed.
    Complete,
    /// A partial reveal prefix was frozen by cancellation.
    Cancelled,
}

/// Provenance of the immutable input used for the scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanSource {
    /// An in-memory deterministic FilesSnapshot; no host filesystem was read.
    ModeledFilesSnapshot,
}

/// Typed request or lifecycle failure.
///
/// Missing scan metadata is a partial-node issue, not a request failure.
/// Generation exhaustion is atomic: the active session is left in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanError {
    /// The request root is not an absolute simulated key.
    InvalidRoot(String),
    /// The exact root key is absent from the input snapshot.
    RootNotFound(String),
    /// A skip path is not an absolute simulated key.
    InvalidSkipPath(String),
    /// A zero batch limit would prevent all progress.
    InvalidBatchLimit,
    /// No unused generation identity remains.
    GenerationExhausted,
    /// A report or cancellation used a non-current generation.
    StaleGeneration {
        /// Generation requested by the caller.
        requested: u64,
        /// Active generation, if one remains.
        current: Option<u64>,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(path) => {
                write!(
                    formatter,
                    "{path}: scan root must be an absolute simulated key"
                )
            }
            Self::RootNotFound(path) => {
                write!(formatter, "{path}: no such simulated snapshot entry")
            }
            Self::InvalidSkipPath(path) => {
                write!(
                    formatter,
                    "{path}: skipped paths must be absolute simulated keys"
                )
            }
            Self::InvalidBatchLimit => formatter.write_str("scan batch limit must be positive"),
            Self::GenerationExhausted => formatter.write_str("scan generation space is exhausted"),
            Self::StaleGeneration { requested, current } => match current {
                Some(current) => write!(
                    formatter,
                    "scan generation {requested} is stale; current generation is {current}"
                ),
                None => write!(formatter, "scan generation {requested} is no longer active"),
            },
        }
    }
}

impl std::error::Error for ScanError {}

/// A controller that owns monotonic scan generations and one active session.
#[derive(Debug)]
pub struct DiskScanController {
    next_generation: Option<u64>,
    current: Option<ScanSession>,
}

impl Default for DiskScanController {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskScanController {
    /// Create an empty controller; the first successful scan has generation 1.
    pub fn new() -> Self {
        Self {
            next_generation: Some(1),
            current: None,
        }
    }

    /// Start a scan from an immutable snapshot and return its generation.
    ///
    /// Validation and tree computation complete before the current session or
    /// next generation changes. A failed request therefore cannot replace a
    /// usable scan or consume a generation.
    pub fn start(
        &mut self,
        snapshot: Arc<FilesSnapshot>,
        request: ScanRequest,
    ) -> Result<u64, ScanError> {
        if request.batch_limit == 0 {
            return Err(ScanError::InvalidBatchLimit);
        }
        if !request.root.starts_with('/') {
            return Err(ScanError::InvalidRoot(request.root));
        }
        if let Some(path) = request
            .skip_paths
            .iter()
            .find(|path| !path.starts_with('/'))
        {
            return Err(ScanError::InvalidSkipPath(path.clone()));
        }
        let root_entry = snapshot
            .get(&request.root)
            .cloned()
            .ok_or_else(|| ScanError::RootNotFound(request.root.clone()))?;
        let generation = self.next_generation.ok_or(ScanError::GenerationExhausted)?;

        let mut seen_identities = BTreeSet::new();
        let root = build_node(
            snapshot.as_ref(),
            root_entry,
            request.include_hidden,
            &request.skip_paths,
            &mut seen_identities,
        )
        .into_node();
        let mut reveal_order = Vec::new();
        preorder(&root, &mut reveal_order);

        self.next_generation = generation.checked_add(1);
        self.current = Some(ScanSession {
            generation,
            root,
            reveal_order,
            started_at: request.started_at,
            effective_duration: request.reveal_duration.max(1),
            batch_limit: request.batch_limit,
            cancelled_count: None,
            _source_snapshot: snapshot,
        });
        Ok(generation)
    }

    /// Return a deterministic report for the matching active generation.
    pub fn report_at(
        &self,
        generation: u64,
        simulation_time: u64,
    ) -> Result<ScanReport<'_>, ScanError> {
        let session = self.session(generation)?;
        let revealed_count = session.revealed_count(simulation_time);
        let status = if session.cancelled_count.is_some() {
            ScanStatus::Cancelled
        } else if revealed_count == session.reveal_order.len() {
            ScanStatus::Complete
        } else {
            ScanStatus::Scanning
        };
        Ok(ScanReport {
            generation: session.generation,
            root: &session.root,
            reveal_order: &session.reveal_order,
            revealed_count,
            status,
            source: ScanSource::ModeledFilesSnapshot,
        })
    }

    /// Freeze the currently revealed prefix for the matching generation.
    ///
    /// Repeated cancellation is harmless. Cancelling after completion leaves
    /// the status Complete, while a stale generation cannot affect a rescan.
    pub fn cancel(&mut self, generation: u64, simulation_time: u64) -> Result<(), ScanError> {
        let current = self.current.as_ref().map(|session| session.generation);
        let Some(session) = self.current.as_mut() else {
            return Err(ScanError::StaleGeneration {
                requested: generation,
                current,
            });
        };
        if session.generation != generation {
            return Err(ScanError::StaleGeneration {
                requested: generation,
                current,
            });
        }
        if session.cancelled_count.is_some()
            || session.revealed_count(simulation_time) == session.reveal_order.len()
        {
            return Ok(());
        }
        session.cancelled_count = Some(session.revealed_count(simulation_time));
        Ok(())
    }

    /// Return the active generation, if the controller has one.
    pub fn current_generation(&self) -> Option<u64> {
        self.current.as_ref().map(|session| session.generation)
    }

    fn session(&self, generation: u64) -> Result<&ScanSession, ScanError> {
        let current = self.current.as_ref().map(|session| session.generation);
        match self.current.as_ref() {
            Some(session) if session.generation == generation => Ok(session),
            _ => Err(ScanError::StaleGeneration {
                requested: generation,
                current,
            }),
        }
    }
}

/// A read-only view of one scan at a simulation tick.
#[derive(Clone, Copy, Debug)]
pub struct ScanReport<'a> {
    generation: u64,
    root: &'a ScanNode,
    reveal_order: &'a [String],
    revealed_count: usize,
    status: ScanStatus,
    source: ScanSource,
}

impl<'a> ScanReport<'a> {
    /// The generation represented by this report.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The full computed tree. Renderers should use visible_paths for reveal.
    pub fn root(&self) -> &'a ScanNode {
        self.root
    }

    /// The deterministic preorder used by progressive reveal.
    pub fn reveal_order(&self) -> &'a [String] {
        self.reveal_order
    }

    /// Number of entries revealed by this report.
    pub fn revealed_count(&self) -> usize {
        self.revealed_count
    }

    /// The current scan lifecycle state.
    pub fn status(&self) -> ScanStatus {
        self.status
    }

    /// The modeled source provenance.
    pub fn source(&self) -> ScanSource {
        self.source
    }

    /// The preorder prefix currently revealed by the modeled scan.
    pub fn visible_paths(&self) -> &'a [String] {
        &self.reveal_order[..self.revealed_count]
    }
}

#[derive(Debug)]
struct ScanSession {
    generation: u64,
    root: ScanNode,
    reveal_order: Vec<String>,
    started_at: u64,
    effective_duration: u64,
    batch_limit: usize,
    cancelled_count: Option<usize>,
    _source_snapshot: Arc<FilesSnapshot>,
}

impl ScanSession {
    fn revealed_count(&self, simulation_time: u64) -> usize {
        if let Some(count) = self.cancelled_count {
            return count.min(self.reveal_order.len());
        }
        let total = self.reveal_order.len();
        let elapsed = simulation_time.saturating_sub(self.started_at);
        if elapsed == 0 || total == 0 {
            return 0;
        }

        let total_wide = total as u128;
        let duration_wide = u128::from(self.effective_duration.max(1));
        let quotient = total_wide / duration_wide;
        let has_remainder = !total_wide.is_multiple_of(duration_wide);
        let target_rate = quotient + u128::from(has_remainder);
        let per_tick = target_rate.max(1).min(self.batch_limit as u128);
        let bounded = u128::from(elapsed).saturating_mul(per_tick).min(total_wide);
        bounded as usize
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ByteAggregate {
    known_subtotal: u64,
    unknown: bool,
    saturated: bool,
}

impl ByteAggregate {
    fn from_optional(value: Option<u64>) -> Self {
        match value {
            Some(known_subtotal) => Self {
                known_subtotal,
                ..Self::default()
            },
            None => Self {
                unknown: true,
                ..Self::default()
            },
        }
    }

    fn include(&mut self, contribution: Self) {
        let (known_subtotal, overflowed) =
            saturating_add_u64(self.known_subtotal, contribution.known_subtotal);
        self.known_subtotal = known_subtotal;
        self.unknown |= contribution.unknown;
        self.saturated |= contribution.saturated || overflowed;
    }

    fn exposed_total(self) -> Option<u64> {
        if self.unknown {
            None
        } else {
            Some(self.known_subtotal)
        }
    }
}

struct BuiltScanNode {
    node: ScanNode,
    allocated: ByteAggregate,
    apparent: ByteAggregate,
}

impl BuiltScanNode {
    fn into_node(mut self) -> ScanNode {
        self.node.allocated_bytes = self.allocated.exposed_total();
        self.node.apparent_bytes = self.apparent.exposed_total();
        self.node.saturation.allocated_bytes = self.allocated.saturated;
        self.node.saturation.apparent_bytes = self.apparent.saturated;
        self.node
    }
}

fn build_node(
    snapshot: &FilesSnapshot,
    entry: FileEntry,
    include_hidden: bool,
    skip_paths: &[String],
    seen_identities: &mut BTreeSet<u64>,
) -> BuiltScanNode {
    let metadata = entry.scan_metadata().copied();
    let mut built = BuiltScanNode {
        node: ScanNode {
            path: entry.path().to_owned(),
            kind: entry.kind().clone(),
            allocated_bytes: metadata.map(|value| value.allocated_bytes),
            apparent_bytes: metadata.map(|value| value.apparent_bytes),
            modified_at_secs: metadata.map(|value| value.modified_at_secs),
            entries: 0,
            issues: Vec::new(),
            saturation: ScanSaturation::default(),
            children: Vec::new(),
        },
        allocated: ByteAggregate::from_optional(metadata.map(|value| value.allocated_bytes)),
        apparent: ByteAggregate::from_optional(metadata.map(|value| value.apparent_bytes)),
    };
    if metadata.is_none() {
        built.node.issues.push(ScanNodeIssue::MetadataUnavailable);
    }

    match &built.node.kind {
        FileKind::Symlink { .. } => {
            built.node.entries = 1;
            return built;
        }
        FileKind::File | FileKind::Special => {
            if let Some(metadata) = metadata
                && !seen_identities.insert(metadata.modeled_identity)
            {
                built.allocated = ByteAggregate::from_optional(Some(0));
                built.apparent = ByteAggregate::from_optional(Some(0));
            }
            built.node.entries = 1;
            return built;
        }
        FileKind::Directory => {}
    }

    if metadata.is_some_and(|value| value.dataless) {
        built.apparent = ByteAggregate::from_optional(Some(0));
        built.node.issues.push(ScanNodeIssue::Dataless);
        return built;
    }

    let children = match snapshot.list(&built.node.path) {
        Ok(children) => children,
        Err(FilesError::PermissionDenied(_)) => {
            built.node.issues.push(ScanNodeIssue::PermissionDenied);
            return built;
        }
        Err(_) => {
            built.node.issues.push(ScanNodeIssue::ListingUnavailable);
            return built;
        }
    };

    for child in children {
        if is_skipped(child.path(), skip_paths) || (!include_hidden && child.is_hidden()) {
            continue;
        }
        let child = build_node(snapshot, child, include_hidden, skip_paths, seen_identities);
        built.allocated.include(child.allocated);
        built.apparent.include(child.apparent);

        let (entries, overflowed) = saturating_add_u64(built.node.entries, child.node.entries);
        built.node.entries = entries;
        built.node.saturation.entries |= child.node.saturation.entries || overflowed;
        built.node.children.push(child.into_node());
    }
    built.node.children.sort_by(|left, right| {
        right
            .allocated_bytes
            .cmp(&left.allocated_bytes)
            .then_with(|| left.path.cmp(&right.path))
    });
    built
}

fn is_skipped(path: &str, skip_paths: &[String]) -> bool {
    skip_paths.iter().any(|skip| {
        if path == skip {
            return true;
        }
        if skip == "/" {
            return path.starts_with('/');
        }
        path.strip_prefix(skip)
            .is_some_and(|remainder| remainder.starts_with('/'))
    })
}

fn saturating_add_u64(left: u64, right: u64) -> (u64, bool) {
    match left.checked_add(right) {
        Some(sum) => (sum, false),
        None => (u64::MAX, true),
    }
}

fn preorder(node: &ScanNode, paths: &mut Vec<String>) {
    paths.push(node.path.clone());
    for child in &node.children {
        preorder(child, paths);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiskScanController, ScanError, ScanNode, ScanNodeIssue, ScanRequest, ScanSource,
        ScanStatus, saturating_add_u64,
    };
    use crate::files::{FileEntry, FileKind, FilesSnapshot, ScanMetadata};
    use std::sync::Arc;

    fn metadata(
        identity: u64,
        allocated_bytes: u64,
        apparent_bytes: u64,
        modified_at_secs: i64,
        dataless: bool,
    ) -> ScanMetadata {
        ScanMetadata {
            modeled_identity: identity,
            allocated_bytes,
            apparent_bytes,
            modified_at_secs,
            dataless,
        }
    }

    fn directory(path: &str, identity: u64, allocated: u64, apparent: u64) -> FileEntry {
        FileEntry::directory(path)
            .with_scan_metadata(metadata(identity, allocated, apparent, 1, false))
    }

    fn file(path: &str, identity: u64, allocated: u64, apparent: u64) -> FileEntry {
        FileEntry::file(path).with_scan_metadata(metadata(identity, allocated, apparent, 2, false))
    }

    fn request(root: &str, started_at: u64, duration: u64, batch: usize) -> ScanRequest {
        ScanRequest::new(root, started_at, duration, batch)
    }

    fn report_tree(controller: &DiskScanController, generation: u64, at: u64) -> ScanNode {
        controller
            .report_at(generation, at)
            .expect("generation is current")
            .root()
            .clone()
    }

    #[test]
    fn deterministic_directory_and_file_roots_keep_path_errors_typed() {
        let mut files = FilesSnapshot::new();
        files.insert(directory("/", 1, 1, 0));
        files.insert(directory("/work", 2, 2, 0));
        files.insert(file("/work/item", 3, 8, 12));
        let snapshot = Arc::new(files);

        let mut first = DiskScanController::new();
        let first_generation = first
            .start(Arc::clone(&snapshot), request("/", 10, 4, 8))
            .expect("modeled directory root scans");
        let first_report = first
            .report_at(first_generation, 10)
            .expect("current generation");
        assert_eq!(first_report.source(), ScanSource::ModeledFilesSnapshot);
        assert_eq!(first_report.status(), ScanStatus::Scanning);
        assert_eq!(first_report.revealed_count(), 0);
        assert_eq!(
            first_report
                .reveal_order()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["/", "/work", "/work/item"]
        );
        let first_tree = first_report.root().clone();

        let mut repeated = DiskScanController::new();
        let repeated_generation = repeated
            .start(Arc::clone(&snapshot), request("/", 10, 4, 8))
            .expect("same immutable input scans again");
        assert_eq!(report_tree(&repeated, repeated_generation, 100), first_tree);

        let file_generation = repeated
            .start(Arc::clone(&snapshot), request("/work/item", 10, 0, 1))
            .expect("regular file roots are accepted");
        let file_report = repeated
            .report_at(file_generation, 11)
            .expect("file-root generation is current");
        assert_eq!(file_report.root().kind, FileKind::File);
        assert_eq!(file_report.root().entries, 1);
        assert_eq!(file_report.root().allocated_bytes, Some(8));
        assert_eq!(file_report.revealed_count(), 1);
        assert_eq!(file_report.status(), ScanStatus::Complete);

        let mut invalid = DiskScanController::new();
        assert_eq!(
            invalid.start(Arc::clone(&snapshot), request("work", 0, 1, 1)),
            Err(ScanError::InvalidRoot("work".into()))
        );
        assert_eq!(
            invalid.start(Arc::clone(&snapshot), request("/missing", 0, 1, 1)),
            Err(ScanError::RootNotFound("/missing".into()))
        );
        let mut invalid_skip = request("/", 0, 1, 1);
        invalid_skip.skip_paths = vec!["relative".into()];
        assert_eq!(
            invalid.start(Arc::clone(&snapshot), invalid_skip),
            Err(ScanError::InvalidSkipPath("relative".into()))
        );

        let mut unmeasured = FilesSnapshot::new();
        unmeasured.insert(FileEntry::file("/plain"));
        let mut partial = DiskScanController::new();
        let generation = partial
            .start(Arc::new(unmeasured), request("/plain", 0, 1, 1))
            .expect("absent optional metadata produces a partial node");
        let root = report_tree(&partial, generation, 1);
        assert_eq!(root.allocated_bytes, None);
        assert!(root.issues.contains(&ScanNodeIssue::MetadataUnavailable));
    }

    #[test]
    fn hardlinks_count_each_path_but_charge_known_identity_once_and_links_stay_leaves() {
        let mut files = FilesSnapshot::new();
        files.insert(directory("/root", 1, 0, 0));
        files.insert(file("/root/a", 7, 8, 16));
        files.insert(file("/root/b", 7, 8, 16));
        files.insert(file("/outside", 9, 100, 200));
        files.insert(
            FileEntry::symlink("/root/outside-link", "/outside")
                .with_scan_metadata(metadata(20, 0, 0, 3, false)),
        );
        files.insert(
            FileEntry::symlink("/root/broken", "/absent")
                .with_scan_metadata(metadata(21, 0, 0, 4, false)),
        );
        files.insert(
            FileEntry::symlink("/root/cycle-a", "/root/cycle-b")
                .with_scan_metadata(metadata(22, 0, 0, 5, false)),
        );
        files.insert(
            FileEntry::symlink("/root/cycle-b", "/root/cycle-a")
                .with_scan_metadata(metadata(23, 0, 0, 6, false)),
        );

        let mut controller = DiskScanController::new();
        let generation = controller
            .start(Arc::new(files), request("/root", 0, 1, 100))
            .expect("modeled root scans");
        let tree = report_tree(&controller, generation, 100);
        assert_eq!(tree.allocated_bytes, Some(8));
        assert_eq!(tree.apparent_bytes, Some(16));
        assert_eq!(tree.entries, 6);
        assert!(tree.find("/outside").is_none());
        for path in [
            "/root/outside-link",
            "/root/broken",
            "/root/cycle-a",
            "/root/cycle-b",
        ] {
            let link = tree.find(path).expect("symlink path remains visible");
            assert!(matches!(link.kind, FileKind::Symlink { .. }));
            assert!(link.children.is_empty());
            assert_eq!(link.entries, 1);
        }
    }

    #[test]
    fn sparse_directory_hidden_skip_boundary_and_tie_order_are_deterministic() {
        let mut files = FilesSnapshot::new();
        files.insert(directory("/", 1, 10, 0));
        files.insert(directory("/a", 2, 1, 0));
        files.insert(file("/a/skipped", 3, 100, 100));
        files.insert(directory("/ab", 4, 2, 0));
        files
            .insert(FileEntry::file("/ab/.hidden").with_scan_metadata(metadata(5, 8, 8, 5, false)));
        files
            .insert(FileEntry::file("/ab/sparse").with_scan_metadata(metadata(6, 3, 50, 6, false)));
        files.insert(file("/ab/tie-b", 7, 5, 7));
        files.insert(file("/ab/tie-a", 8, 5, 7));
        let snapshot = Arc::new(files);

        let mut filtered = DiskScanController::new();
        let mut filtered_request = request("/", 0, 1, 100);
        filtered_request.include_hidden = false;
        filtered_request.skip_paths = vec!["/a".into()];
        let generation = filtered
            .start(Arc::clone(&snapshot), filtered_request)
            .expect("absolute skip path is accepted");
        let tree = report_tree(&filtered, generation, 100);
        assert!(tree.find("/a").is_none());
        let ab = tree.find("/ab").expect("component-prefix sibling remains");
        assert!(ab.find("/ab/.hidden").is_none());
        assert_eq!(ab.allocated_bytes, Some(15));
        assert_eq!(ab.apparent_bytes, Some(64));
        assert_eq!(
            ab.children
                .iter()
                .map(|child| child.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/ab/tie-a", "/ab/tie-b", "/ab/sparse"]
        );
        let sparse = ab.find("/ab/sparse").expect("sparse modeled file exists");
        assert_eq!(
            (sparse.allocated_bytes, sparse.apparent_bytes),
            (Some(3), Some(50))
        );

        let mut included = DiskScanController::new();
        let generation = included
            .start(Arc::clone(&snapshot), request("/", 0, 1, 100))
            .expect("default includes hidden entries");
        let tree = report_tree(&included, generation, 100);
        assert!(tree.find("/a/skipped").is_some());
        assert!(tree.find("/ab/.hidden").is_some());
    }

    #[test]
    fn unreadable_dataless_and_unmeasured_nodes_remain_partial_without_opening_content() {
        let mut files = FilesSnapshot::new();
        files.insert(directory("/root", 1, 1, 0));
        files.insert(directory("/root/locked", 2, 5, 1).unreadable());
        files.insert(file("/root/locked/secret", 3, 9, 9));
        files.insert(
            FileEntry::directory("/root/cloud").with_scan_metadata(metadata(4, 7, 12, 4, true)),
        );
        files.insert(file("/root/cloud/evicted", 5, 100, 100));
        files.insert(FileEntry::directory("/root/unavailable-locked").unreadable());
        files.insert(file("/root/unavailable-locked/hidden", 6, 10, 10));
        files.insert(FileEntry::directory("/root/unavailable"));
        files.insert(file("/root/unavailable/known", 7, 11, 12));
        files.insert(FileEntry::file_with_bytes(
            "/root/content-only",
            b"content-is-not-size-metadata".to_vec(),
        ));
        files.insert(FileEntry::symlink(
            "/root/unmeasured-link",
            "/root/unavailable/known",
        ));
        files.insert(
            FileEntry::special("/root/device").with_scan_metadata(metadata(8, 2, 2, 7, false)),
        );

        let mut controller = DiskScanController::new();
        let generation = controller
            .start(Arc::new(files), request("/root", 0, 1, 100))
            .expect("partial metadata does not reject the scan");
        let root = report_tree(&controller, generation, 100);

        let locked = root
            .find("/root/locked")
            .expect("unreadable directory is visible");
        assert!(locked.issues.contains(&ScanNodeIssue::PermissionDenied));
        assert!(locked.children.is_empty());
        assert_eq!(locked.allocated_bytes, Some(5));

        let cloud = root
            .find("/root/cloud")
            .expect("dataless directory is visible");
        assert!(cloud.issues.contains(&ScanNodeIssue::Dataless));
        assert!(cloud.children.is_empty());
        assert_eq!(cloud.apparent_bytes, Some(0));

        let unavailable_locked = root
            .find("/root/unavailable-locked")
            .expect("unmeasured unreadable directory remains visible");
        assert!(
            unavailable_locked
                .issues
                .contains(&ScanNodeIssue::MetadataUnavailable)
        );
        assert!(
            unavailable_locked
                .issues
                .contains(&ScanNodeIssue::PermissionDenied)
        );
        assert!(unavailable_locked.children.is_empty());
        assert_eq!(unavailable_locked.allocated_bytes, None);

        let unavailable = root
            .find("/root/unavailable")
            .expect("readable unmeasured directory remains visible");
        assert!(
            unavailable
                .issues
                .contains(&ScanNodeIssue::MetadataUnavailable)
        );
        assert!(
            unavailable
                .children
                .iter()
                .any(|child| child.path == "/root/unavailable/known")
        );
        assert_eq!(unavailable.allocated_bytes, None);
        assert_eq!(unavailable.apparent_bytes, None);

        let content_only = root
            .find("/root/content-only")
            .expect("file path remains visible");
        assert!(
            content_only
                .issues
                .contains(&ScanNodeIssue::MetadataUnavailable)
        );
        assert_eq!(content_only.allocated_bytes, None);
        assert_eq!(content_only.apparent_bytes, None);
        assert_eq!(content_only.entries, 1);

        let unmeasured_link = root
            .find("/root/unmeasured-link")
            .expect("unmeasured symlink remains visible");
        assert!(
            unmeasured_link
                .issues
                .contains(&ScanNodeIssue::MetadataUnavailable)
        );
        assert!(unmeasured_link.children.is_empty());
        assert_eq!(unmeasured_link.entries, 1);

        let device = root
            .find("/root/device")
            .expect("special node remains a leaf");
        assert_eq!(device.kind, FileKind::Special);
        assert!(device.children.is_empty());
        assert_eq!(device.allocated_bytes, Some(2));
        assert!(root.find("/root/locked/secret").is_none());
        assert!(root.find("/root/cloud/evicted").is_none());
    }

    #[test]
    fn tick_reveal_and_saturation_provenance_are_bounded_and_explicit() {
        let mut files = FilesSnapshot::new();
        files.insert(directory("/timed", 1, 0, 0));
        for index in 0..3 {
            files.insert(file(&format!("/timed/{index}"), 10 + index, 1, 1));
        }
        let mut controller = DiskScanController::new();
        let generation = controller
            .start(Arc::new(files), request("/timed", u64::MAX - 2, 0, 1))
            .expect("zero duration normalizes to one effective tick");
        assert_eq!(
            controller
                .report_at(generation, u64::MAX - 3)
                .expect("pre-start report")
                .revealed_count(),
            0
        );
        let at_start = controller
            .report_at(generation, u64::MAX - 2)
            .expect("start frame");
        assert_eq!(at_start.revealed_count(), 0);
        assert_eq!(at_start.status(), ScanStatus::Scanning);
        let after_one = controller
            .report_at(generation, u64::MAX - 1)
            .expect("first positive tick");
        assert_eq!(after_one.revealed_count(), 1);
        assert_eq!(after_one.status(), ScanStatus::Scanning);
        let after_two = controller
            .report_at(generation, u64::MAX)
            .expect("large simulation time is bounded");
        assert_eq!(after_two.revealed_count(), 2);
        assert_eq!(after_two.status(), ScanStatus::Scanning);
        assert_eq!(after_two.visible_paths().len(), 2);

        let mut exact = FilesSnapshot::new();
        exact.insert(directory("/exact", 1, u64::MAX, u64::MAX));
        exact.insert(file("/exact/zero", 2, 0, 0));
        let mut exact_controller = DiskScanController::new();
        let exact_generation = exact_controller
            .start(Arc::new(exact), request("/exact", 0, 1, 10))
            .expect("exact maximum remains representable");
        let exact_root = report_tree(&exact_controller, exact_generation, 10);
        assert_eq!(exact_root.allocated_bytes, Some(u64::MAX));
        assert_eq!(exact_root.apparent_bytes, Some(u64::MAX));
        assert!(!exact_root.saturation.allocated_bytes);
        assert!(!exact_root.saturation.apparent_bytes);

        let mut overflowing = FilesSnapshot::new();
        overflowing.insert(directory("/overflow", 1, u64::MAX, u64::MAX));
        overflowing.insert(file("/overflow/a", 2, 1, 1));
        overflowing.insert(file("/overflow/b", 3, 0, 0));
        let mut overflow_controller = DiskScanController::new();
        let overflow_generation = overflow_controller
            .start(Arc::new(overflowing), request("/overflow", 0, 1, 10))
            .expect("overflow preserves the inspectable tree");
        let overflow_root = report_tree(&overflow_controller, overflow_generation, 10);
        assert_eq!(overflow_root.allocated_bytes, Some(u64::MAX));
        assert_eq!(overflow_root.apparent_bytes, Some(u64::MAX));
        assert!(overflow_root.saturation.allocated_bytes);
        assert!(overflow_root.saturation.apparent_bytes);
        assert!(overflow_root.find("/overflow/a").is_some());
        assert_eq!(saturating_add_u64(u64::MAX, 0), (u64::MAX, false));
        assert_eq!(saturating_add_u64(u64::MAX, 1), (u64::MAX, true));

        let mut unknown = FilesSnapshot::new();
        unknown.insert(FileEntry::directory("/unknown"));
        unknown.insert(directory("/unknown/sub", 2, 0, 0));
        unknown.insert(file("/unknown/sub/a", 3, u64::MAX, 0));
        unknown.insert(file("/unknown/sub/b", 4, 1, 0));
        let mut unknown_controller = DiskScanController::new();
        let unknown_generation = unknown_controller
            .start(Arc::new(unknown), request("/unknown", 0, 1, 10))
            .expect("unknown metadata remains a partial scan");
        let unknown_root = report_tree(&unknown_controller, unknown_generation, 10);
        let sub = unknown_root
            .find("/unknown/sub")
            .expect("known child exists");
        assert_eq!(sub.allocated_bytes, Some(u64::MAX));
        assert!(sub.saturation.allocated_bytes);
        assert_eq!(unknown_root.allocated_bytes, None);
        assert!(unknown_root.saturation.allocated_bytes);
        assert!(unknown_root.find("/unknown/sub/a").is_some());
        assert!(unknown_root.find("/unknown/sub/b").is_some());

        let mut capped_duration = FilesSnapshot::new();
        capped_duration.insert(directory("/batch", 1, 0, 0));
        capped_duration.insert(file("/batch/a", 2, 1, 1));
        capped_duration.insert(file("/batch/b", 3, 1, 1));
        let mut capped_controller = DiskScanController::new();
        let capped_generation = capped_controller
            .start(Arc::new(capped_duration), request("/batch", 0, 1, 1))
            .expect("positive batch cap accepted");
        let capped = capped_controller
            .report_at(capped_generation, 1)
            .expect("bounded first tick");
        assert_eq!(capped.revealed_count(), 1);
        assert_eq!(capped.status(), ScanStatus::Scanning);
    }

    #[test]
    fn unknown_root_preserves_per_dimension_overflow_and_exact_maximum_provenance() {
        let mut allocated_only = FilesSnapshot::new();
        allocated_only.insert(FileEntry::directory("/allocated"));
        allocated_only.insert(file("/allocated/a", 2, u64::MAX, 0));
        allocated_only.insert(file("/allocated/b", 3, 1, 0));
        let mut allocated_controller = DiskScanController::new();
        let allocated_generation = allocated_controller
            .start(Arc::new(allocated_only), request("/allocated", 0, 1, 10))
            .expect("unknown root keeps its measured children");
        let allocated_root = report_tree(&allocated_controller, allocated_generation, 10);
        assert_eq!(allocated_root.allocated_bytes, None);
        assert_eq!(allocated_root.apparent_bytes, None);
        assert!(allocated_root.saturation.allocated_bytes);
        assert!(!allocated_root.saturation.apparent_bytes);
        assert_eq!(allocated_root.children.len(), 2);

        let mut apparent_only = FilesSnapshot::new();
        apparent_only.insert(FileEntry::directory("/apparent"));
        apparent_only.insert(file("/apparent/a", 2, 0, u64::MAX));
        apparent_only.insert(file("/apparent/b", 3, 0, 1));
        let mut apparent_controller = DiskScanController::new();
        let apparent_generation = apparent_controller
            .start(Arc::new(apparent_only), request("/apparent", 0, 1, 10))
            .expect("unknown root keeps its measured children");
        let apparent_root = report_tree(&apparent_controller, apparent_generation, 10);
        assert_eq!(apparent_root.allocated_bytes, None);
        assert_eq!(apparent_root.apparent_bytes, None);
        assert!(!apparent_root.saturation.allocated_bytes);
        assert!(apparent_root.saturation.apparent_bytes);
        assert_eq!(apparent_root.children.len(), 2);

        let mut exact = FilesSnapshot::new();
        exact.insert(FileEntry::directory("/exact-unknown"));
        exact.insert(file("/exact-unknown/max", 2, u64::MAX, u64::MAX));
        exact.insert(file("/exact-unknown/zero", 3, 0, 0));
        let mut exact_controller = DiskScanController::new();
        let exact_generation = exact_controller
            .start(Arc::new(exact), request("/exact-unknown", 0, 1, 10))
            .expect("unknown root keeps exact maximum subtotals");
        let exact_root = report_tree(&exact_controller, exact_generation, 10);
        assert_eq!(exact_root.allocated_bytes, None);
        assert_eq!(exact_root.apparent_bytes, None);
        assert!(!exact_root.saturation.allocated_bytes);
        assert!(!exact_root.saturation.apparent_bytes);
    }

    #[test]
    fn known_sibling_overflow_survives_unknown_child_before_between_and_after() {
        let orders: [[Option<u64>; 3]; 3] = [
            [None, Some(u64::MAX), Some(1)],
            [Some(u64::MAX), None, Some(1)],
            [Some(u64::MAX), Some(1), None],
        ];
        for (case_index, order) in orders.into_iter().enumerate() {
            let root_path = format!("/root-{case_index}");
            let mut files = FilesSnapshot::new();
            files.insert(directory(&root_path, 1, 0, 0));
            for (child_index, size) in order.into_iter().enumerate() {
                let path = format!("{root_path}/{child_index}-{size:?}");
                let identity = 10 + u64::try_from(child_index).expect("fixture index fits u64");
                let entry = match size {
                    Some(value) => file(&path, identity, value, value),
                    None => FileEntry::file(&path),
                };
                files.insert(entry);
            }

            let mut controller = DiskScanController::new();
            let generation = controller
                .start(Arc::new(files), request(&root_path, 0, 1, 10))
                .expect("known root scans all ordered children");
            let root = report_tree(&controller, generation, 10);
            assert_eq!(root.allocated_bytes, None);
            assert_eq!(root.apparent_bytes, None);
            assert!(root.saturation.allocated_bytes);
            assert!(root.saturation.apparent_bytes);
            assert_eq!(root.children.len(), 3);
        }
    }

    #[test]
    fn cancellation_rescan_snapshot_capture_and_generation_exhaustion_are_atomic() {
        let mut files = FilesSnapshot::new();
        files.insert(directory("/root", 1, 0, 0));
        files.insert(file("/root/a", 2, 1, 1));
        files.insert(file("/root/b", 3, 1, 1));
        files.insert(file("/root/c", 4, 1, 1));
        let mut controller = DiskScanController::new();
        let first = controller
            .start(Arc::new(files.clone()), request("/root", 10, 4, 1))
            .expect("first generation starts");
        let mut replacement = Arc::new(files.clone());
        let second = controller
            .start(Arc::clone(&replacement), request("/root", 10, 4, 1))
            .expect("replacement scan starts");
        assert_eq!(second, first + 1);
        assert_eq!(
            controller.cancel(first, 11),
            Err(ScanError::StaleGeneration {
                requested: first,
                current: Some(second),
            })
        );

        let mut bad_batch = request("/root", 10, 4, 0);
        bad_batch.include_hidden = false;
        assert_eq!(
            controller.start(Arc::clone(&replacement), bad_batch),
            Err(ScanError::InvalidBatchLimit)
        );
        assert_eq!(controller.current_generation(), Some(second));
        let captured_before = report_tree(&controller, second, 11);

        let new_file = FileEntry::file("/root/a").with_scan_metadata(metadata(2, 99, 99, 2, false));
        Arc::make_mut(&mut replacement).insert(new_file);
        let captured_after = report_tree(&controller, second, 100);
        assert_eq!(
            captured_after
                .find("/root/a")
                .map(|node| node.allocated_bytes),
            Some(Some(1))
        );
        assert_eq!(
            captured_before
                .find("/root/a")
                .map(|node| node.allocated_bytes),
            Some(Some(1))
        );

        let third = controller
            .start(Arc::new(files.clone()), request("/root", 20, 4, 1))
            .expect("next replacement starts without a consumed invalid generation");
        assert_eq!(third, second + 1);
        assert_eq!(
            controller
                .report_at(third, 21)
                .expect("current generation reports")
                .revealed_count(),
            1
        );
        controller
            .cancel(third, 21)
            .expect("partial cancellation succeeds");
        let cancelled = controller
            .report_at(third, u64::MAX)
            .expect("cancelled generation remains reportable");
        assert_eq!(cancelled.revealed_count(), 1);
        assert_eq!(cancelled.status(), ScanStatus::Cancelled);
        controller
            .cancel(third, u64::MAX)
            .expect("repeat cancel is harmless");
        let repeated = controller
            .report_at(third, u64::MAX)
            .expect("repeat cancellation report");
        assert_eq!(repeated.revealed_count(), 1);
        assert_eq!(repeated.status(), ScanStatus::Cancelled);

        let completed = controller
            .start(Arc::new(files.clone()), request("/root", 0, 1, 100))
            .expect("completed scan starts");
        assert_eq!(
            controller
                .report_at(completed, 10)
                .expect("scan completed")
                .status(),
            ScanStatus::Complete
        );
        controller
            .cancel(completed, 10)
            .expect("cancel-after-complete is harmless");
        assert_eq!(
            controller
                .report_at(completed, 10)
                .expect("completed scan remains complete")
                .status(),
            ScanStatus::Complete
        );

        let mut exhausted = DiskScanController::new();
        exhausted.next_generation = Some(u64::MAX);
        let last = exhausted
            .start(Arc::new(files.clone()), request("/root", 0, 1, 100))
            .expect("final unique generation is usable");
        assert_eq!(last, u64::MAX);
        let before_failure = report_tree(&exhausted, last, 10);
        assert_eq!(
            exhausted.start(Arc::new(files), request("/root", 0, 1, 100)),
            Err(ScanError::GenerationExhausted)
        );
        assert_eq!(exhausted.current_generation(), Some(last));
        assert_eq!(report_tree(&exhausted, last, 10), before_failure);
    }
}
