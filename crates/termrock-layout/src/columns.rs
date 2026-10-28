use core::cmp::Reverse;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ColumnSpec {
    min: u16,
    ideal: u16,
    max: u16,
    flex: u16,
    drop_priority: Option<u8>,
}

impl ColumnSpec {
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

#[must_use]
pub fn solve_columns_first(specs: &[ColumnSpec], available: u16, gap: u16) -> Option<u16> {
    solve_columns_head(specs, available, gap).map(|widths| widths[0])
}

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
