//! Named source-correct branches frozen beside their parent scenario IDs.
//!
//! The 16 normative branches come from
//! `trusted/holla-trace-corrections.md` (4 preview, 4 Args, 8 editing/idle
//! overlay branches). The remaining 17 are named in the frozen
//! `holla-scenarios.tsv` actions column (clock, CLI-parse, files, alias,
//! menu, output-find families); each name occurs in exactly one parent row,
//! which this table records.

/// One named branch identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedBranch {
    /// Parent HO-* row.
    pub parent: &'static str,
    /// Immutable branch identity.
    pub id: &'static str,
}

/// Trace-correction and scenario-register branches TASK-003 must materialize.
pub const NAMED_BRANCHES: &[NamedBranch] = &[
    NamedBranch {
        parent: "HO-BASE-01",
        id: "clock-firstuse-frame-0-1-40",
    },
    NamedBranch {
        parent: "HO-BASE-01",
        id: "clock-cadence-not-delta",
    },
    NamedBranch {
        parent: "HO-CLI-PREVIEW",
        id: "cli-parse-all34",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "preview-copy-find-live",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "preview-selection-sweep",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "preview-ctrl-c-exit",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "preview-end-selection-noop",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "files-browse-find-paste",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "files-mode-find-paste-outer",
    },
    NamedBranch {
        parent: "HO-PREVIEW-SELECT",
        id: "files-browse-paste-no-find",
    },
    NamedBranch {
        parent: "HO-ARGS",
        id: "args-insert-fallback",
    },
    NamedBranch {
        parent: "HO-ARGS",
        id: "args-valid-replacement",
    },
    NamedBranch {
        parent: "HO-ARGS",
        id: "args-required-empty",
    },
    NamedBranch {
        parent: "HO-ARGS",
        id: "args-nonnumeric-fallback",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-help-isolated",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-picker-isolated",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-quit-isolated",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-menu-passthrough",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-help-idle-isolated",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-picker-idle-isolated",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-quit-idle-isolated",
    },
    NamedBranch {
        parent: "HO-OVERLAY-PASTE",
        id: "paste-menu-idle-begins-edit",
    },
    NamedBranch {
        parent: "HO-ROUTE-10",
        id: "alias-transfer-exact",
    },
    NamedBranch {
        parent: "HO-ROUTE-10",
        id: "alias-case-sensitive-ownership",
    },
    NamedBranch {
        parent: "HO-ROUTE-10",
        id: "ranking-reset-all-paths",
    },
    NamedBranch {
        parent: "HO-MENUS",
        id: "menu-ctrl-c-captured",
    },
    NamedBranch {
        parent: "HO-MENUS",
        id: "menu-ctrl-q-captured",
    },
    NamedBranch {
        parent: "HO-MENUS",
        id: "menu-dismiss-ctrl-c-exit",
    },
    NamedBranch {
        parent: "HO-MENUS",
        id: "menu-dismiss-ctrl-q-confirm",
    },
    NamedBranch {
        parent: "HO-OUTPUT-FIND",
        id: "output-find-paste-trim",
    },
    NamedBranch {
        parent: "HO-OUTPUT-FIND",
        id: "output-find-escape-keeps-follow-paused",
    },
    NamedBranch {
        parent: "HO-OUTPUT-FIND",
        id: "output-find-live-coordinate-reconcile",
    },
    NamedBranch {
        parent: "HO-OUTPUT-FIND",
        id: "output-find-empty-no-match",
    },
];

/// Number of named branches.
#[must_use]
pub fn named_branch_count() -> usize {
    NAMED_BRANCHES.len()
}
