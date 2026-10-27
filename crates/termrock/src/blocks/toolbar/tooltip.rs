use std::time::Duration;

use ratatui_core::layout::Rect;
use ratatui_core::style::Modifier;
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::intent::{Intent, Phase};
use termrock_runtime::layer::{Anchor, CrossAlign, DismissReason, LayerEvent, LayerSize, Side};
use termrock_runtime::{Cx, FrameRead, LayerOpening, Ui};
use termrock_text::{width, wrap};
use termrock_theme::{ColorLevel, Family, Surface, Variant};

use super::{ToolItem, ToolbarLayout, ToolbarTip, ToolbarTooltipPolicy};

const TOOLTIP_ID: &str = "hover-tooltip";
const TOOLTIP_PART: PartRef = PartRef::of(Part::LABEL);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Item(ItemKey),
    More,
}

impl Target {
    const fn part(self) -> PartRef {
        match self {
            Self::Item(key) => PartRef::item(Part::ROW, key),
            Self::More => PartRef::of(Part::OVERFLOW),
        }
    }

    fn from_part(part: PartRef) -> Option<Self> {
        match (part.part, part.item) {
            (Part::ROW, Some(key)) => Some(Self::Item(key)),
            (Part::OVERFLOW, None) => Some(Self::More),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
struct Candidate {
    owner: Id,
    target: Target,
    text: String,
    shortcut: Option<String>,
    policy: ToolbarTooltipPolicy,
    area: Rect,
    viewport: Rect,
}

impl Candidate {
    fn matches_identity(&self, active: &Active) -> bool {
        self.owner == active.owner
            && self.target == active.target
            && self.text == active.text
            && self.shortcut == active.shortcut
    }

    fn matches(&self, active: &Active) -> bool {
        self.matches_identity(active) && self.policy == active.policy
    }
}

#[derive(Clone, Debug)]
struct Active {
    owner: Id,
    target: Target,
    text: String,
    shortcut: Option<String>,
    policy: ToolbarTooltipPolicy,
    since: termrock_runtime::Moment,
    area: Rect,
    viewport: Rect,
    opening: Option<LayerOpening>,
}

impl Active {
    fn from_candidate(candidate: Candidate, since: termrock_runtime::Moment) -> Self {
        Self {
            owner: candidate.owner,
            target: candidate.target,
            text: candidate.text,
            shortcut: candidate.shortcut,
            policy: candidate.policy,
            since,
            area: candidate.area,
            viewport: candidate.viewport,
            opening: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct TooltipController {
    owner: Option<Id>,
    active: Option<Active>,
    dismissed: Option<Target>,
    source_press: Option<Target>,
}

#[derive(Clone, Copy)]
pub(super) struct TooltipConfig<'a> {
    pub(super) owner: Id,
    pub(super) items: &'a [ToolItem],
    pub(super) item_keys: &'a [ItemKey],
    pub(super) policy: ToolbarTooltipPolicy,
    pub(super) more_tip: &'a ToolbarTip,
}

impl TooltipController {
    pub(super) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        config: TooltipConfig<'_>,
        layout: &ToolbarLayout,
        intents: &[Intent<'_>],
        menu_open: bool,
    ) {
        let owner = config.owner;
        if self.owner != Some(owner) {
            if self.owner.is_some() {
                self.reset_for_owner_change(cx);
            }
            self.owner = Some(owner);
        }
        self.consume_managed_events(cx);

        let hovered_candidate = resolve_candidate(cx, config, layout);
        self.reconcile_source_press(cx, config, layout);
        self.observe_source_press(cx, owner, intents, hovered_candidate.as_ref());

        if menu_open {
            self.close(cx, owner);
            self.active = None;
            self.dismissed = None;
            self.source_press = None;
            return;
        }

        let tip_owner = owner.sub(TOOLTIP_ID);
        let tip_hovered = cx.hovered_part(tip_owner) == Some(TOOLTIP_PART);
        let candidate = if tip_hovered {
            let Some(active) = self.active.as_ref() else {
                return;
            };
            let Some(candidate) = resolve_target(cx, config, layout, active.target) else {
                self.close(cx, owner);
                self.active = None;
                self.dismissed = None;
                return;
            };
            Some(candidate)
        } else {
            hovered_candidate
        };
        let native_hover_suppressed = cx.hover_suppressed();
        if native_hover_suppressed {
            if let Some(active) = self.active.as_ref() {
                self.dismissed = Some(active.target);
            }
            self.close(cx, owner);
            return;
        }

        let captured = cx.capture_owner().is_some();
        if self.source_press.is_none()
            && self.dismissed.is_some()
            && candidate.as_ref().is_some_and(|candidate| {
                self.active.as_ref().is_some_and(|active| {
                    active.target == candidate.target && !candidate.matches_identity(active)
                })
            })
        {
            self.dismissed = None;
            self.active = None;
        }
        if let Some(dismissed) = self.dismissed {
            let still_over_target = cx.hovered_part(owner) == Some(dismissed.part());
            if still_over_target || tip_hovered || captured || self.source_press.is_some() {
                self.close(cx, owner);
                return;
            }
            self.dismissed = None;
            self.active = None;
        }

        if let Some(candidate) = candidate {
            let replace = self
                .active
                .as_ref()
                .is_none_or(|active| !candidate.matches(active));
            if replace {
                self.close(cx, owner);
                self.active = Some(Active::from_candidate(candidate, cx.now()));
                self.dismissed = None;
            } else if let Some(active) = self.active.as_mut() {
                active.area = candidate.area;
                active.viewport = candidate.viewport;
            }
        } else if let Some(active) = self.active.as_ref() {
            let source_hovered = cx.hovered_part(owner) == Some(active.target.part());
            if source_hovered || (!tip_hovered && !captured && self.source_press.is_none()) {
                self.close(cx, owner);
                self.active = None;
                self.dismissed = None;
            }
        }

        if captured || self.source_press.is_some() {
            return;
        }

        let Some(active) = self.active.as_mut() else {
            return;
        };
        let measured = TooltipLayout::measure(
            &active.text,
            active.shortcut.as_deref(),
            active.policy,
            active.viewport,
        );
        let Some((width, height)) = measured.layer_size() else {
            self.close(cx, owner);
            self.active = None;
            return;
        };
        let anchor = Anchor::Rect {
            rect: active.area,
            side: Side::Below,
            align: CrossAlign::Start,
        };
        let size = LayerSize::Fixed(width, height);
        let tooltip_id = owner.sub(TOOLTIP_ID);

        if let Some(opening) = active.opening
            && !cx.refresh_hover_tooltip(opening, anchor, size)
        {
            active.opening = None;

            active.since = cx.now();
        }

        if active.opening.is_none() {
            let delay = Duration::from_millis(active.policy.delay_ms);
            let deadline = active.since.saturating_add(delay);
            if cx.now() < deadline {
                cx.request_repaint_at(deadline);
                return;
            }
            let Some(opening) =
                cx.open_hover_tooltip(tooltip_id, owner, active.target.part(), anchor, size)
            else {
                self.active = None;
                return;
            };
            active.opening = Some(opening);
        }
    }

    pub(super) fn draw(
        &self,
        ui: &mut Ui<'_>,
        config: TooltipConfig<'_>,
        visible_rows: &[(ItemKey, usize, Rect)],
        more_area: Option<Rect>,
        menu_open: bool,
    ) {
        let TooltipConfig {
            owner,
            items,
            item_keys,
            policy,
            more_tip,
        } = config;
        if menu_open || ui.is_inert() {
            return;
        }
        if self.owner != Some(owner) {
            return;
        }
        let Some(active) = self.active.as_ref() else {
            return;
        };
        if active.owner != owner {
            return;
        }
        let Some(opening) = active.opening else {
            return;
        };
        if !policy.enabled || active.policy != policy || active.viewport.is_empty() {
            return;
        }

        let source = match active.target {
            Target::Item(key) => {
                let Some((_, index, area)) =
                    visible_rows.iter().find(|(row_key, _, _)| *row_key == key)
                else {
                    return;
                };
                let Some(item) = items.get(*index) else {
                    return;
                };
                let current_key = item_keys.get(*index).copied();
                if current_key != Some(key)
                    || item.tip.as_deref() != Some(active.text.as_str())
                    || item.shortcut.as_deref().filter(|value| !value.is_empty())
                        != active.shortcut.as_deref()
                {
                    return;
                }
                (*area, active.target.part())
            }
            Target::More => {
                let Some(area) = more_area.filter(|area| !area.is_empty()) else {
                    return;
                };
                if more_tip.text != active.text
                    || more_tip
                        .shortcut
                        .as_deref()
                        .filter(|value| !value.is_empty())
                        != active.shortcut.as_deref()
                {
                    return;
                }
                (area, active.target.part())
            }
        };
        let source_area = ui.area_of_part(owner, source.1);
        if source_area != Some(source.0) || source.0 != active.area {
            return;
        }

        let measured = TooltipLayout::measure(
            &active.text,
            active.shortcut.as_deref(),
            active.policy,
            active.viewport,
        );
        if measured.layer_size().is_none() {
            return;
        }
        let tooltip_id = owner.sub(TOOLTIP_ID);
        if ui.hovered_part(owner) != Some(source.1)
            && ui.hovered_part(tooltip_id) != Some(TOOLTIP_PART)
        {
            return;
        }
        let parent_surface = ui.surface();
        let _ = ui.layer_for(opening, |ui, area| {
            measured.paint(ui, tooltip_id, area, parent_surface);
        });
    }

    fn observe_source_press(
        &mut self,
        cx: &mut Cx<'_>,
        owner: Id,
        intents: &[Intent<'_>],
        candidate: Option<&Candidate>,
    ) {
        let active_target = self.active.as_ref().map(|active| active.target);
        let candidate_target = candidate.map(|candidate| candidate.target);
        for intent in intents {
            let Intent::Pointer { phase, part, .. } = intent else {
                continue;
            };
            if *phase == Phase::Release && self.source_press.is_some() {
                self.source_press = None;
                continue;
            }
            let Some(target) = Target::from_part(*part) else {
                continue;
            };
            if Some(target) != active_target && Some(target) != candidate_target {
                continue;
            }
            match phase {
                Phase::Press => {
                    self.source_press = Some(target);
                    self.dismissed = Some(target);
                    self.close(cx, owner);
                }
                Phase::Release => {
                    self.source_press = None;
                }
                _ => {}
            }
        }
    }

    fn reconcile_source_press(
        &mut self,
        cx: &Cx<'_>,
        config: TooltipConfig<'_>,
        layout: &ToolbarLayout,
    ) {
        let Some(target) = self.source_press else {
            return;
        };
        let source_is_live = resolve_target(cx, config, layout, target).is_some();
        let owner = config.owner;
        let pressed_source = cx.pressed_part(owner) == Some(target.part());
        let capture_is_compatible = cx
            .capture_owner()
            .is_none_or(|captured_owner| captured_owner == owner);
        if !source_is_live || !pressed_source || !capture_is_compatible {
            self.source_press = None;
        }
    }

    fn consume_managed_events(&mut self, cx: &mut Cx<'_>) {
        let Some(opening) = self.active.as_ref().and_then(|active| active.opening) else {
            return;
        };
        let mut dismissed_target = None;
        let mut terminal = false;
        while let Some(receipt) = cx.take_layer_intent_receipt_for(opening) {
            match receipt.event() {
                LayerEvent::Opened => {}
                LayerEvent::Dismissed(DismissReason::Esc)
                | LayerEvent::Dismissed(DismissReason::OutsideClick) => {
                    dismissed_target = self.active.as_ref().map(|active| active.target);
                }
                _ => terminal = true,
            }
        }
        if let Some(target) = dismissed_target {
            self.dismissed = Some(target);
            if let Some(active) = self.active.as_mut()
                && active.opening == Some(opening)
            {
                active.opening = None;
            }
        } else if terminal
            && self
                .active
                .as_ref()
                .is_some_and(|active| active.opening == Some(opening))
        {
            self.active = None;
            self.dismissed = None;
            self.source_press = None;
        }
    }

    fn reset_for_owner_change(&mut self, cx: &mut Cx<'_>) {
        self.consume_managed_events(cx);
        if let Some(opening) = self.active.as_ref().and_then(|active| active.opening) {
            let _ = cx.close_hover_tooltip(opening);
            while cx.take_layer_intent_receipt_for(opening).is_some() {}
        }
        self.active = None;
        self.dismissed = None;
        self.source_press = None;
    }

    fn close(&mut self, cx: &mut Cx<'_>, _owner: Id) {
        let Some(opening) = self
            .active
            .as_mut()
            .and_then(|active| active.opening.take())
        else {
            return;
        };
        let _ = cx.close_hover_tooltip(opening);
        while cx.take_layer_intent_receipt_for(opening).is_some() {}
    }
}

fn resolve_candidate(
    cx: &Cx<'_>,
    config: TooltipConfig<'_>,
    layout: &ToolbarLayout,
) -> Option<Candidate> {
    let target = Target::from_part(cx.hovered_part(config.owner)?)?;
    resolve_target(cx, config, layout, target)
}

fn resolve_target(
    cx: &Cx<'_>,
    config: TooltipConfig<'_>,
    layout: &ToolbarLayout,
    target: Target,
) -> Option<Candidate> {
    let TooltipConfig {
        owner,
        items,
        item_keys,
        policy,
        more_tip,
    } = config;
    if !policy.enabled {
        return None;
    }
    let source_area = match target {
        Target::Item(key) => {
            let row = layout.rows.iter().find(|row| row.key == key)?;
            if item_keys.get(row.index).copied() != Some(key) {
                return None;
            }
            let item = items.get(row.index)?;
            let text = item.tip.as_deref().filter(|text| !text.is_empty())?;
            let area = cx.area_of_part(owner, target.part())?;
            if area.is_empty() || area != row.area {
                return None;
            }
            (area, text.to_owned(), item.shortcut.as_deref())
        }
        Target::More => {
            let expected = layout.more?;
            let area = cx.area_of_part(owner, target.part())?;
            if area.is_empty() || area != expected {
                return None;
            }
            if more_tip.text.is_empty() {
                return None;
            }
            (area, more_tip.text.clone(), more_tip.shortcut.as_deref())
        }
    };
    let (area, text, shortcut) = source_area;
    Some(Candidate {
        owner,
        target,
        text,
        shortcut: shortcut
            .filter(|shortcut| !shortcut.is_empty())
            .map(str::to_owned),
        policy,
        area,
        viewport: cx.viewport(),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TooltipLine {
    text: String,
    shortcut: Option<String>,
    shortcut_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TooltipLayout {
    lines: Vec<TooltipLine>,
    width: u16,
    height: u16,
    inner_width: u16,
}

impl TooltipLayout {
    fn measure(
        text: &str,
        shortcut: Option<&str>,
        policy: ToolbarTooltipPolicy,
        viewport: Rect,
    ) -> Self {
        let inner_width = viewport
            .width
            .saturating_sub(2)
            .min(policy.max_width)
            .max(1);
        let wrapped = wrap(text, inner_width);
        let mut lines = wrapped
            .into_iter()
            .map(|text| TooltipLine {
                text,
                shortcut: None,
                shortcut_only: false,
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push(TooltipLine {
                text: String::new(),
                shortcut: None,
                shortcut_only: false,
            });
        }
        if let Some(shortcut) = shortcut.filter(|shortcut| !shortcut.is_empty())
            && let Some(last) = lines.last_mut()
        {
            if width(&last.text)
                .saturating_add(2)
                .saturating_add(width(shortcut))
                <= inner_width
            {
                last.shortcut = Some(shortcut.to_owned());
            } else {
                let wrapped_shortcut = wrap(shortcut, inner_width);
                if wrapped_shortcut.is_empty() {
                    lines.push(TooltipLine {
                        text: String::new(),
                        shortcut: None,
                        shortcut_only: true,
                    });
                } else {
                    lines.extend(wrapped_shortcut.into_iter().map(|text| TooltipLine {
                        text,
                        shortcut: None,
                        shortcut_only: true,
                    }));
                }
            }
        }
        let content_width = lines.iter().fold(0u16, |widest, line| {
            let line_width = if line.shortcut_only {
                width(&line.text)
            } else {
                line.shortcut
                    .as_deref()
                    .filter(|shortcut| !shortcut.is_empty())
                    .map_or_else(
                        || width(&line.text),
                        |shortcut| {
                            width(&line.text)
                                .saturating_add(2)
                                .saturating_add(width(shortcut))
                        },
                    )
            };
            widest.max(line_width)
        });
        let width = content_width.saturating_add(2).min(viewport.width);
        let height = u16::try_from(lines.len())
            .unwrap_or(u16::MAX)
            .min(viewport.height);
        Self {
            lines,
            width,
            height,
            inner_width,
        }
    }

    fn layer_size(&self) -> Option<(u16, u16)> {
        (self.width > 0 && self.height > 0).then_some((self.width, self.height))
    }

    fn paint(&self, ui: &mut Ui<'_>, owner: Id, area: Rect, parent_surface: Surface) {
        let area = area.intersection(ui.full());
        if area.is_empty() {
            return;
        }
        ui.register_hover_part(owner, TOOLTIP_PART, area);

        let surface = if parent_surface == Surface::Popover {
            Surface::Overlay
        } else {
            Surface::Popover
        };
        let mono = ui.theme_ref().capability.color == ColorLevel::Mono;
        let mut body = ui.surface_style().bg(ui.theme_ref().bg(surface));
        if mono {
            body = body.add_modifier(Modifier::REVERSED);
        }
        ui.fill(area, body);

        let mut shortcut_style = ui
            .style(
                Family::MENU,
                Variant::DEFAULT,
                Part::TITLE,
                termrock_core::response::StateFlags::empty(),
            )
            .style
            .bg(ui.theme_ref().bg(surface));
        if mono {
            shortcut_style = shortcut_style.add_modifier(Modifier::REVERSED);
        }

        let content_x = area.x.saturating_add(1);
        let content_width = area.width.saturating_sub(2);
        for (index, line) in self.lines.iter().take(area.height as usize).enumerate() {
            let y = area
                .y
                .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
            if y >= area.bottom() {
                break;
            }
            let text_width = width(&line.text).min(content_width).min(self.inner_width);
            if text_width > 0 {
                ui.paint_str(
                    Rect::new(content_x, y, text_width, 1),
                    &line.text,
                    if line.shortcut_only {
                        shortcut_style
                    } else {
                        body
                    },
                );
            }
            if line.shortcut_only {
                continue;
            }
            let Some(shortcut) = line
                .shortcut
                .as_deref()
                .filter(|shortcut| !shortcut.is_empty())
            else {
                continue;
            };
            let shortcut_x = content_x
                .saturating_add(width(&line.text))
                .saturating_add(2);
            if shortcut_x >= area.right().saturating_sub(1) {
                continue;
            }
            let remaining = area.right().saturating_sub(1).saturating_sub(shortcut_x);
            let shortcut_width = width(shortcut).min(remaining);
            if shortcut_width > 0 {
                ui.paint_str(
                    Rect::new(shortcut_x, y, shortcut_width, 1),
                    shortcut,
                    shortcut_style,
                );
            }
        }
    }
}
