//! Borrowed semantic layout descriptions for [`Form`](crate::Form).
//!
//! A layout names fields by their stable [`Id`]. It
//! contains no terminal rectangles and owns no field values or widget state.

use core::fmt;

use termrock_core::id::Id;
use termrock_layout::{Maximized, SplitAxis, SplitModel};

/// One field row in a form lane.
#[derive(Clone, Copy, Debug)]
pub enum FormRow {
    /// Place one field at the lane's full width.
    Field(Id),
    /// Place two fields side by side using one canonical split policy.
    Pair {
        first: Id,
        second: Id,
        split: SplitModel,
        gap: u16,
    },
    /// Reserve a fixed number of empty rows.
    Spacer(u16),
    /// Give the remaining lane height to a field, subject to its measured minimum.
    Fill(Id),
}

impl FormRow {
    #[must_use]
    pub const fn field(id: Id) -> Self {
        Self::Field(id)
    }

    #[must_use]
    pub const fn pair(first: Id, second: Id, split: SplitModel, gap: u16) -> Self {
        Self::Pair {
            first,
            second,
            split,
            gap,
        }
    }

    #[must_use]
    pub const fn spacer(rows: u16) -> Self {
        Self::Spacer(rows)
    }

    #[must_use]
    pub const fn fill(id: Id) -> Self {
        Self::Fill(id)
    }
}

/// A stable semantic lane. `id` owns its independent scroll state.
#[derive(Clone, Copy, Debug)]
pub struct FormLane<'a> {
    pub id: Id,
    rows: &'a [FormRow],
}

impl<'a> FormLane<'a> {
    #[must_use]
    pub const fn new(id: Id, rows: &'a [FormRow]) -> Self {
        Self { id, rows }
    }

    #[must_use]
    pub const fn rows(self) -> &'a [FormRow] {
        self.rows
    }
}

/// Borrowed lane composition for a form. One lane is vertical; two lanes use
/// the supplied horizontal split. More lanes are rejected by Form preflight.
#[derive(Clone, Copy, Debug)]
pub struct FormLayout<'a> {
    lanes: &'a [FormLane<'a>],
    split: Option<(SplitModel, u16)>,
}

impl<'a> FormLayout<'a> {
    #[must_use]
    pub const fn new(lanes: &'a [FormLane<'a>]) -> Self {
        Self { lanes, split: None }
    }

    /// Set the one horizontal sibling split for a two-lane form.
    #[must_use]
    pub const fn split(mut self, model: SplitModel, gap: u16) -> Self {
        self.split = Some((model, gap));
        self
    }

    pub const fn lanes(self) -> &'a [FormLane<'a>] {
        self.lanes
    }

    pub(crate) const fn split_policy(self) -> Option<(SplitModel, u16)> {
        self.split
    }
}

/// A malformed borrowed layout rejected before it can register controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormLayoutError {
    Empty,
    TooManyLanes,
    DuplicateLane(Id),
    MissingSplit,
    UnexpectedSplit,
    SplitAxis,
    SplitSizing,
    UnknownField(Id),
    DuplicateField(Id),
    MissingVisibleField(Id),
    FillMustBeLast(Id),
}

impl fmt::Display for FormLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("form layout must contain at least one lane"),
            Self::TooManyLanes => f.write_str("form layout supports at most two lanes"),
            Self::DuplicateLane(id) => write!(f, "duplicate form lane id: {id:?}"),
            Self::MissingSplit => f.write_str("two-lane form layout requires a split policy"),
            Self::UnexpectedSplit => f.write_str("one-lane form layout cannot have a split policy"),
            Self::SplitAxis => f.write_str("form lane split must be horizontal"),
            Self::SplitSizing => f.write_str(
                "form split must use a percentage from 0 through 100 and no maximized lane",
            ),
            Self::UnknownField(id) => write!(f, "form layout references unknown field: {id:?}"),
            Self::DuplicateField(id) => {
                write!(f, "form layout references field more than once: {id:?}")
            }
            Self::MissingVisibleField(id) => {
                write!(f, "active form layout omits visible field: {id:?}")
            }
            Self::FillMustBeLast(id) => {
                write!(f, "fill field must be the last non-spacer row: {id:?}")
            }
        }
    }
}

impl std::error::Error for FormLayoutError {}

impl FormLayoutError {
    /// Returns the field associated with a malformed layout, when the error
    /// names one. Structural errors are reported against the owning form.
    #[must_use]
    pub const fn field_id(self) -> Option<Id> {
        match self {
            Self::UnknownField(id)
            | Self::DuplicateField(id)
            | Self::MissingVisibleField(id)
            | Self::FillMustBeLast(id) => Some(id),
            Self::Empty
            | Self::TooManyLanes
            | Self::DuplicateLane(_)
            | Self::MissingSplit
            | Self::UnexpectedSplit
            | Self::SplitAxis
            | Self::SplitSizing => None,
        }
    }
}

impl FormLayout<'_> {
    pub(crate) fn validate(
        self,
        field_ids: impl Iterator<Item = Id>,
    ) -> Result<(), FormLayoutError> {
        let lanes = self.lanes;
        if lanes.is_empty() {
            return Err(FormLayoutError::Empty);
        }
        if lanes.len() > 2 {
            return Err(FormLayoutError::TooManyLanes);
        }
        for (index, lane) in lanes.iter().enumerate() {
            if lanes[..index].iter().any(|previous| previous.id == lane.id) {
                return Err(FormLayoutError::DuplicateLane(lane.id));
            }
        }
        match (lanes.len(), self.split) {
            (1, Some(_)) => return Err(FormLayoutError::UnexpectedSplit),
            (2, None) => return Err(FormLayoutError::MissingSplit),
            (2, Some((split, _))) => {
                if split.axis != SplitAxis::Horizontal {
                    return Err(FormLayoutError::SplitAxis);
                }
                if split.percent > 100 || split.maximized != Maximized::None {
                    return Err(FormLayoutError::SplitSizing);
                }
            }
            _ => {}
        }
        let ids: Vec<Id> = field_ids.collect();
        for (index, id) in ids.iter().enumerate() {
            if ids.iter().take(index).any(|previous| previous == id) {
                return Err(FormLayoutError::DuplicateField(*id));
            }
        }
        let mut used = Vec::new();
        for lane in lanes {
            for (row_index, row) in lane.rows.iter().enumerate() {
                if let FormRow::Fill(id) = row
                    && lane
                        .rows
                        .iter()
                        .skip(row_index.saturating_add(1))
                        .any(|following| !matches!(following, FormRow::Spacer(_)))
                {
                    return Err(FormLayoutError::FillMustBeLast(*id));
                }
                let row_ids: &[Id] = match row {
                    FormRow::Field(id) | FormRow::Fill(id) => core::slice::from_ref(id),
                    FormRow::Pair {
                        first,
                        second,
                        split,
                        ..
                    } => {
                        if split.axis != SplitAxis::Horizontal {
                            return Err(FormLayoutError::SplitAxis);
                        }
                        if split.percent > 100 || split.maximized != Maximized::None {
                            return Err(FormLayoutError::SplitSizing);
                        }
                        if first == second {
                            return Err(FormLayoutError::DuplicateField(*first));
                        }
                        // Validate the pair members individually below.
                        core::slice::from_ref(first)
                    }
                    FormRow::Spacer(_) => continue,
                };
                for id in row_ids {
                    if !ids.contains(id) {
                        return Err(FormLayoutError::UnknownField(*id));
                    }
                    if used.contains(id) {
                        return Err(FormLayoutError::DuplicateField(*id));
                    }
                    used.push(*id);
                }
                if let FormRow::Pair { second, .. } = row {
                    if !ids.contains(second) {
                        return Err(FormLayoutError::UnknownField(*second));
                    }
                    if used.contains(second) {
                        return Err(FormLayoutError::DuplicateField(*second));
                    }
                    used.push(*second);
                }
            }
        }
        Ok(())
    }
}
