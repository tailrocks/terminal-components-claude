use core::cmp::Reverse;

/// Measurement and growth constraints for one column in a shared row.
///
/// Public constructors establish `min <= ideal <= max`, including normalization
/// in [`ColumnSpec::new`]. Private fields preserve those bounds for the solver.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ColumnSpec {
    min: u16,
    ideal: u16,
    max: u16,
    flex: u16,
    drop_priority: Option<u8>,
}

impl ColumnSpec {
    /// Builds a column specification, raising `max` and clamping `ideal` as
    /// needed to maintain `min <= ideal <= max`.
    pub const fn new(min: u16, ideal: u16, max: u16, flex: u16, drop_priority: Option<u8>) -> Self {
        let max = if max < min { min } else { max };
        let ideal = if ideal < min {
            min
        } else if ideal > max {
            max
        } else {
            ideal
        };
        Self {
            min,
            ideal,
            max,
            flex,
            drop_priority,
        }
    }

    #[must_use]
    pub const fn min(self) -> u16 {
        self.min
    }

    #[must_use]
    pub const fn ideal(self) -> u16 {
        self.ideal
    }

    #[must_use]
    pub const fn max(self) -> u16 {
        self.max
    }

    #[must_use]
    pub const fn flex(self) -> u16 {
        self.flex
    }

    #[must_use]
    pub const fn drop_priority(self) -> Option<u8> {
        self.drop_priority
    }
}

/// Resolved widths and offsets for a [`ColumnSpec`] slice.
///
/// A hidden column has no offset and `is_visible` is false. A visible column
/// may have width zero; it remains distinct from a hidden column so callers
/// can preserve intentional gaps without creating gaps for dropped columns.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ResolvedColumns {
    widths: Vec<u16>,
    offsets: Vec<Option<u16>>,
    visible: Vec<bool>,
    content_width: u16,
}

struct ColumnWorkspace<'a> {
    widths: &'a mut [u64],
    offsets: &'a mut [Option<u16>],
    visible: &'a mut [bool],
    drop_order: &'a mut [(u8, Reverse<usize>)],
    open: &'a mut [bool],
}

impl ResolvedColumns {
    #[must_use]
    pub fn len(&self) -> usize {
        self.widths.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.widths.is_empty()
    }

    #[must_use]
    pub fn width(&self, index: usize) -> Option<u16> {
        self.widths.get(index).copied()
    }

    #[must_use]
    pub fn offset(&self, index: usize) -> Option<u16> {
        self.offsets.get(index).copied().flatten()
    }

    #[must_use]
    pub fn is_visible(&self, index: usize) -> bool {
        self.visible.get(index).copied().unwrap_or(false)
    }

    #[must_use]
    pub const fn content_width(&self) -> u16 {
        self.content_width
    }
}

/// Resolves column widths within `available` cells.
///
/// Droppable columns disappear by ascending priority; equal priorities remove
/// rightmost columns first. Remaining minima shrink from right to left, ideal
/// widths grow in round-robin order, weighted flex receives spare width, and
/// final unclaimed width grows toward maxima from left to right.
#[must_use]
pub fn solve_columns(specs: &[ColumnSpec], available: u16, gap: u16) -> ResolvedColumns {
    let count = specs.len();
    let mut widths = vec![0u64; count];
    let mut offsets = vec![None; count];
    let mut visible = vec![false; count];
    let mut drop_order = vec![(0u8, Reverse(0usize)); count];
    let mut open = vec![false; count];
    let content_width = resolve_columns(
        specs,
        available,
        gap,
        ColumnWorkspace {
            widths: &mut widths,
            offsets: &mut offsets,
            visible: &mut visible,
            drop_order: &mut drop_order,
            open: &mut open,
        },
    );
    let widths = widths
        .into_iter()
        .map(|width| u16::try_from(width).unwrap_or(u16::MAX))
        .collect();
    ResolvedColumns {
        widths,
        offsets,
        visible,
        content_width,
    }
}

/// Resolves only the first column without allocating when `specs` has at most
/// four entries. Larger slices use the same general solver without truncation.
#[must_use]
pub fn solve_columns_first(specs: &[ColumnSpec], available: u16, gap: u16) -> Option<u16> {
    solve_columns_head(specs, available, gap).map(|widths| widths[0])
}

/// Solves the leading columns without allocating when `specs` has at most
/// four entries, returning the solved widths of the first two columns (0 for
/// a column beyond the slice). Larger slices fall back to the allocating
/// solver.
#[must_use]
pub fn solve_columns_head(specs: &[ColumnSpec], available: u16, gap: u16) -> Option<[u16; 2]> {
    if specs.is_empty() {
        return None;
    }
    if specs.len() > 4 {
        let solved = solve_columns(specs, available, gap);
        return Some([solved.width(0).unwrap_or(0), solved.width(1).unwrap_or(0)]);
    }
    let mut widths = [0u64; 4];
    let mut offsets = [None; 4];
    let mut visible = [false; 4];
    let mut drop_order = [(0u8, Reverse(0usize)); 4];
    let mut open = [false; 4];
    let _ = resolve_columns(
        specs,
        available,
        gap,
        ColumnWorkspace {
            widths: &mut widths[..specs.len()],
            offsets: &mut offsets[..specs.len()],
            visible: &mut visible[..specs.len()],
            drop_order: &mut drop_order[..specs.len()],
            open: &mut open[..specs.len()],
        },
    );
    let to_u16 = |w: u64| u16::try_from(w).unwrap_or(u16::MAX);
    Some([to_u16(widths[0]), to_u16(widths[1])])
}

fn resolve_columns(
    specs: &[ColumnSpec],
    available: u16,
    gap: u16,
    workspace: ColumnWorkspace<'_>,
) -> u16 {
    let ColumnWorkspace {
        widths,
        offsets,
        visible,
        drop_order,
        open,
    } = workspace;
    let count = specs.len();
    debug_assert!(widths.len() >= count);
    debug_assert!(offsets.len() >= count);
    debug_assert!(visible.len() >= count);
    debug_assert!(drop_order.len() >= count);
    debug_assert!(open.len() >= count);
    visible[..count].fill(true);
    widths[..count].fill(0);
    offsets[..count].fill(None);
    let mut drop_count = 0;
    for (index, spec) in specs.iter().enumerate() {
        if let Some(priority) = spec.drop_priority {
            drop_order[drop_count] = (priority, Reverse(index));
            drop_count += 1;
        }
    }
    drop_order[..drop_count].sort_unstable();

    let available = u64::from(available);
    let mut visible_count = count;
    let mut minimum_sum = specs
        .iter()
        .fold(0u64, |sum, spec| sum.saturating_add(u64::from(spec.min)));
    for (_, Reverse(index)) in drop_order.iter().take(drop_count).copied() {
        let gap_sum = u64::from(gap).saturating_mul(visible_count.saturating_sub(1) as u64);
        if minimum_sum.saturating_add(gap_sum) <= available {
            break;
        }
        visible[index] = false;
        visible_count -= 1;
        minimum_sum = minimum_sum.saturating_sub(u64::from(specs[index].min));
    }

    let gaps = u64::from(gap).saturating_mul(visible_count.saturating_sub(1) as u64);
    let room = available.saturating_sub(gaps);
    for (index, spec) in specs.iter().enumerate() {
        if visible[index] {
            widths[index] = u64::from(spec.min);
        }
    }
    let mut used: u64 = widths.iter().take(count).sum();
    if used > room {
        for width in widths[..count].iter_mut().rev() {
            if used <= room {
                break;
            }
            let cut = (used - room).min(*width);
            *width -= cut;
            used -= cut;
        }
    }

    let mut spare = room.saturating_sub(used);
    grow_to_ideal(specs, visible, widths, &mut spare);
    grow_flex(specs, visible, widths, &mut spare, open);
    for index in 0..count {
        if spare == 0 {
            break;
        }
        if visible[index] && widths[index] < u64::from(specs[index].max) {
            let give = (u64::from(specs[index].max) - widths[index]).min(spare);
            widths[index] += give;
            spare -= give;
        }
    }

    let mut cursor = 0u64;
    let mut seen = false;
    for index in 0..count {
        if !visible[index] {
            continue;
        }
        if seen {
            cursor = cursor.saturating_add(u64::from(gap)).min(available);
        }
        offsets[index] = Some(u16::try_from(cursor.min(available)).unwrap_or(u16::MAX));
        cursor = cursor.saturating_add(widths[index]);
        seen = true;
    }
    u16::try_from(cursor.min(available)).unwrap_or(u16::MAX)
}

fn grow_to_ideal(specs: &[ColumnSpec], visible: &[bool], widths: &mut [u64], spare: &mut u64) {
    if *spare == 0 {
        return;
    }
    let mut max_rounds = 0u64;
    for (index, spec) in specs.iter().enumerate() {
        if visible[index] && widths[index] < u64::from(spec.ideal) {
            max_rounds = max_rounds.max(u64::from(spec.ideal) - widths[index]);
        }
    }
    if max_rounds == 0 {
        return;
    }

    let cost_for_rounds = |rounds: u64| -> u64 {
        specs
            .iter()
            .enumerate()
            .filter(|(index, _)| visible[*index])
            .map(|(index, spec)| (u64::from(spec.ideal) - widths[index]).min(rounds))
            .sum()
    };
    let mut low = 0u64;
    let mut high = max_rounds;
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if cost_for_rounds(middle) <= *spare {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    let spent = cost_for_rounds(low);
    for (index, spec) in specs.iter().enumerate() {
        if visible[index] && widths[index] < u64::from(spec.ideal) {
            widths[index] += (u64::from(spec.ideal) - widths[index]).min(low);
        }
    }
    *spare -= spent;
    for (index, spec) in specs.iter().enumerate() {
        if *spare == 0 {
            break;
        }
        if visible[index] && widths[index] < u64::from(spec.ideal) {
            widths[index] += 1;
            *spare -= 1;
        }
    }
}

fn grow_flex(
    specs: &[ColumnSpec],
    visible: &[bool],
    widths: &mut [u64],
    spare: &mut u64,
    open: &mut [bool],
) {
    while *spare > 0 {
        open.fill(false);
        let mut weight = 0u64;
        for (index, spec) in specs.iter().enumerate() {
            if visible[index] && spec.flex > 0 && widths[index] < u64::from(spec.max) {
                open[index] = true;
                weight = weight.saturating_add(u64::from(spec.flex));
            }
        }
        if weight == 0 {
            break;
        }
        let round = *spare;
        let mut given = 0u64;
        for (index, spec) in specs.iter().enumerate() {
            if !open[index] || *spare == 0 {
                continue;
            }
            let capacity = u64::from(spec.max) - widths[index];
            let proportional = (round.saturating_mul(u64::from(spec.flex)) / weight).max(1);
            let share = proportional.min(capacity).min(*spare);
            widths[index] += share;
            *spare -= share;
            given = given.saturating_add(share);
        }
        if given == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnSpec, solve_columns};

    #[derive(Clone, Copy)]
    struct FrozenColumn {
        min: u16,
        ideal: u16,
        max: u16,
        flex: u16,
        drop: bool,
    }

    // Frozen four-column implementation retained only as a differential test
    // oracle while ItemColumns moves onto the shared solver.
    fn frozen_solve(columns: &[FrozenColumn], available: u16, gap: u16) -> Vec<u16> {
        let count = columns.len().min(4);
        let mut visible = [true; 4];
        let gaps = |visible: &[bool; 4]| -> u32 {
            u32::from(gap)
                * visible
                    .iter()
                    .take(count)
                    .filter(|on| **on)
                    .count()
                    .saturating_sub(1) as u32
        };
        let minima = |visible: &[bool; 4]| -> u32 {
            columns
                .iter()
                .take(count)
                .zip(visible.iter().take(count))
                .filter(|(_, on)| **on)
                .map(|(column, _)| u32::from(column.min))
                .sum()
        };
        while minima(&visible).saturating_add(gaps(&visible)) > u32::from(available) {
            let victim = columns
                .iter()
                .take(count)
                .enumerate()
                .filter(|(index, column)| visible[*index] && column.drop)
                .min_by_key(|(index, _)| core::cmp::Reverse(*index))
                .map(|(index, _)| index);
            match victim {
                Some(index) => visible[index] = false,
                None => break,
            }
        }
        let room = u32::from(available).saturating_sub(gaps(&visible));
        let mut widths = [0u32; 4];
        for (index, column) in columns.iter().take(count).enumerate() {
            if visible[index] {
                widths[index] = u32::from(column.min);
            }
        }
        let mut used: u32 = widths.iter().take(count).sum();
        for width in widths.iter_mut().take(count).rev() {
            if used <= room {
                break;
            }
            let cut = (used - room).min(*width);
            *width -= cut;
            used -= cut;
        }
        let mut spare = room.saturating_sub(used);
        while spare > 0 {
            let mut grew = false;
            for index in 0..count {
                if spare == 0 {
                    break;
                }
                if visible[index] && widths[index] < u32::from(columns[index].ideal) {
                    widths[index] += 1;
                    spare -= 1;
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        while spare > 0 {
            let mut open = [false; 4];
            let mut open_count = 0u32;
            let mut weight = 0u32;
            for index in 0..count {
                if visible[index]
                    && columns[index].flex > 0
                    && widths[index] < u32::from(columns[index].max)
                {
                    open[index] = true;
                    open_count += 1;
                    weight = weight.saturating_add(u32::from(columns[index].flex));
                }
            }
            if open_count == 0 || weight == 0 {
                break;
            }
            let round = spare;
            let mut given = 0u32;
            for index in 0..count {
                if !open[index] || spare == 0 {
                    continue;
                }
                let share = (round.saturating_mul(u32::from(columns[index].flex)) / weight)
                    .max(1)
                    .min(u32::from(columns[index].max) - widths[index])
                    .min(spare);
                widths[index] += share;
                spare -= share;
                given = given.saturating_add(share);
            }
            if given == 0 {
                break;
            }
        }
        for index in 0..count {
            if spare == 0 {
                break;
            }
            if visible[index] && widths[index] < u32::from(columns[index].max) {
                let give = (u32::from(columns[index].max) - widths[index]).min(spare);
                widths[index] += give;
                spare -= give;
            }
        }
        widths
            .iter()
            .take(count)
            .map(|width| u16::try_from(*width).unwrap_or(u16::MAX))
            .collect()
    }

    #[test]
    fn preserves_frozen_solver_widths_across_terminal_widths_and_gaps() {
        let profiles = [
            FrozenColumn {
                min: 0,
                ideal: 0,
                max: 0,
                flex: 0,
                drop: false,
            },
            FrozenColumn {
                min: 1,
                ideal: 3,
                max: 7,
                flex: 0,
                drop: false,
            },
            FrozenColumn {
                min: 4,
                ideal: 4,
                max: 9,
                flex: 1,
                drop: true,
            },
            FrozenColumn {
                min: 0,
                ideal: 4,
                max: 40,
                flex: 1,
                drop: true,
            },
            FrozenColumn {
                min: 2,
                ideal: 6,
                max: u16::MAX,
                flex: 2,
                drop: false,
            },
        ];
        for count in 0..=4 {
            let combinations = profiles.len().pow(count as u32);
            for combination in 0..combinations {
                let mut choice = combination;
                let columns: Vec<FrozenColumn> = (0..count)
                    .map(|_| {
                        let column = profiles[choice % profiles.len()];
                        choice /= profiles.len();
                        column
                    })
                    .collect();
                let specs: Vec<ColumnSpec> = columns
                    .iter()
                    .map(|column| {
                        ColumnSpec::new(
                            column.min,
                            column.ideal,
                            column.max,
                            column.flex,
                            column.drop.then_some(0),
                        )
                    })
                    .collect();
                for available in 0..=160 {
                    for gap in 0..=3 {
                        let resolved = solve_columns(&specs, available, gap);
                        let expected = frozen_solve(&columns, available, gap);
                        assert_eq!(
                            (0..count)
                                .map(|index| resolved.width(index).unwrap())
                                .collect::<Vec<_>>(),
                            expected,
                            "count={count}, combination={combination}, available={available}, gap={gap}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn normalizes_constraints_and_handles_arbitrary_empty_or_zero_columns() {
        let normalized = ColumnSpec::new(8, 2, 4, 1, None);
        assert_eq!(
            (normalized.min(), normalized.ideal(), normalized.max()),
            (8, 8, 8)
        );
        let empty = solve_columns(&[], 0, 3);
        assert!(empty.is_empty());
        assert_eq!(empty.content_width(), 0);

        let specs = vec![ColumnSpec::new(0, 0, 0, 0, None); 24];
        let zero = solve_columns(&specs, 5, 2);
        assert_eq!(zero.len(), 24);
        assert!((0..24).all(|index| zero.is_visible(index)));
        assert_eq!(zero.content_width(), 5);
        assert!(!zero.is_visible(24));
        assert_eq!(zero.width(24), None);
        assert_eq!(zero.offset(24), None);
    }

    #[test]
    fn drops_by_priority_and_rightmost_ties_without_phantom_hidden_gaps() {
        let specs = [
            ColumnSpec::new(2, 2, 2, 0, None),
            ColumnSpec::new(2, 2, 2, 0, Some(1)),
            ColumnSpec::new(2, 2, 2, 0, Some(0)),
            ColumnSpec::new(2, 2, 2, 0, Some(0)),
        ];
        let resolved = solve_columns(&specs, 6, 1);
        assert!(resolved.is_visible(0));
        assert!(resolved.is_visible(1));
        assert!(!resolved.is_visible(2));
        assert!(!resolved.is_visible(3));
        assert_eq!(resolved.offset(0), Some(0));
        assert_eq!(resolved.offset(1), Some(3));
        assert_eq!(resolved.content_width(), 5);
    }
}
