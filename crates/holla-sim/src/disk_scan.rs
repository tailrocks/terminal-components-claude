use crate::files::{FileEntry, FileKind, FilesError, FilesSnapshot};
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanRequest {
    pub root: String,

    pub include_hidden: bool,

    pub skip_paths: Vec<String>,

    pub started_at: u64,

    pub reveal_duration: u64,

    pub batch_limit: usize,
}

impl ScanRequest {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanNodeIssue {
    MetadataUnavailable,

    PermissionDenied,

    Dataless,

    ListingUnavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScanSaturation {
    pub allocated_bytes: bool,

    pub apparent_bytes: bool,

    pub entries: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanNode {
    pub path: String,

    pub kind: FileKind,

    pub allocated_bytes: Option<u64>,

    pub apparent_bytes: Option<u64>,

    pub modified_at_secs: Option<i64>,

    pub entries: u64,

    pub issues: Vec<ScanNodeIssue>,

    pub saturation: ScanSaturation,

    pub children: Vec<ScanNode>,
}

impl ScanNode {
    pub fn find(&self, path: &str) -> Option<&ScanNode> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(path))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanStatus {
    Scanning,

    Complete,

    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanSource {
    ModeledFilesSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanError {
    InvalidRoot(String),

    RootNotFound(String),

    InvalidSkipPath(String),

    InvalidBatchLimit,

    GenerationExhausted,

    StaleGeneration {
        requested: u64,

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
    pub fn new() -> Self {
        Self {
            next_generation: Some(1),
            current: None,
        }
    }

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
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn root(&self) -> &'a ScanNode {
        self.root
    }

    pub fn reveal_order(&self) -> &'a [String] {
        self.reveal_order
    }

    pub fn revealed_count(&self) -> usize {
        self.revealed_count
    }

    pub fn status(&self) -> ScanStatus {
        self.status
    }

    pub fn source(&self) -> ScanSource {
        self.source
    }

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
