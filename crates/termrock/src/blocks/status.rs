use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use termrock_runtime::Ui;
use termrock_text::width;
use termrock_theme::{FgStep, Theme};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StatusTone {
    #[default]
    Normal,

    Secondary,

    Success,

    Danger,
}

pub struct StatusSeg<'a> {
    pub text: &'a str,

    pub tone: StatusTone,
}

fn tone_style(tone: StatusTone, theme: &Theme) -> Style {
    let color = match tone {
        StatusTone::Normal => theme.color.fg[FgStep::Primary.index()],
        StatusTone::Secondary => theme.color.fg[FgStep::Secondary.index()],
        StatusTone::Success => theme.color.success,
        StatusTone::Danger => theme.color.danger,
    };
    Style::default().fg(color)
}

fn fitting_right_suffix(right: &[StatusSeg<'_>], available: usize) -> (usize, usize) {
    let mut first = right.len();
    let mut total = 0usize;
    for index in (0..right.len()).rev() {
        let gap = if first < right.len() { 1usize } else { 0 };
        let Some(next_total) = total
            .checked_add(usize::from(width(right[index].text)))
            .and_then(|width| width.checked_add(gap))
        else {
            break;
        };
        if next_total > available {
            break;
        }
        first = index;
        total = next_total;
    }
    (first, total)
}

pub fn paint_status(
    ui: &mut Ui<'_>,
    area: Rect,
    left: &[StatusSeg<'_>],
    right: &[StatusSeg<'_>],
    theme: &Theme,
) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let end = area.right();
    let mut painted = 0u16;
    let mut x = area.x;
    for (index, seg) in left.iter().enumerate() {
        if index > 0 {
            if x >= end {
                break;
            }
            x = x.saturating_add(1);
        }
        if x >= end {
            break;
        }
        painted = painted.saturating_add(ui.paint_str(
            Rect::new(x, area.y, end - x, 1),
            seg.text,
            tone_style(seg.tone, theme),
        ));
        x = x.saturating_add(width(seg.text));
    }
    let left_end = x.min(end);
    let (first, total) = fitting_right_suffix(right, usize::from(end.saturating_sub(left_end)));

    let Ok(total) = u16::try_from(total) else {
        return painted;
    };
    let mut x = end.saturating_sub(total);
    for (offset, seg) in right[first..].iter().enumerate() {
        if offset > 0 {
            x = x.saturating_add(1);
        }
        painted = painted.saturating_add(ui.paint_str(
            Rect::new(x, area.y, end.saturating_sub(x), 1),
            seg.text,
            tone_style(seg.tone, theme),
        ));
        x = x.saturating_add(width(seg.text));
    }
    painted
}
