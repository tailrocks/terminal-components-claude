use std::collections::HashMap;
use std::sync::Weak;

use tablepro_domain::tabs::{Tab, TabKey};
use termrock::Rect;

use crate::workbench::Workbench;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum QueryPaneMode {
    #[default]
    None,
    Editor,
    Results,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QueryContent {
    Idle,
    Results,
    Error,
    Plan,
}

#[derive(Debug, Default)]
pub(super) struct QueryPaneState {
    owner: Option<Weak<()>>,
    modes: HashMap<TabKey, QueryPaneMode>,
}

impl QueryPaneState {
    pub(super) fn sync(&mut self, workbench: &Workbench) {
        let same_owner = self
            .owner
            .as_ref()
            .is_some_and(|owner| workbench.matches_owner(owner));
        if !same_owner {
            self.owner = Some(workbench.owner_token());
            self.modes.clear();
        }
        self.modes
            .retain(|key, _| matches!(workbench.tab(*key), Some(Tab::Query(_))));
    }

    pub(super) fn mode(&self, key: TabKey) -> QueryPaneMode {
        self.modes.get(&key).copied().unwrap_or_default()
    }

    pub(super) fn set(&mut self, key: TabKey, mode: QueryPaneMode) {
        if mode == QueryPaneMode::None {
            self.modes.remove(&key);
        } else {
            self.modes.insert(key, mode);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct QueryLayout {
    pub(super) body: Rect,
    pub(super) toolbar: Rect,
    pub(super) editor: Option<Rect>,
    pub(super) seam: Option<Rect>,
    pub(super) content: Rect,
}

impl QueryLayout {
    pub(super) fn new(
        area: Rect,
        inner: Rect,
        inset: u16,
        mode: QueryPaneMode,
        content_kind: QueryContent,
    ) -> Option<Self> {
        if inner.width == 0 || inner.height == 0 {
            return None;
        }
        let body = Rect {
            x: area.x.saturating_add(inset),
            y: inner.y,
            width: area.width.saturating_sub(inset),
            height: inner.height,
        };
        match mode {
            QueryPaneMode::None => Self::legacy(area, inner, body, content_kind),
            QueryPaneMode::Editor => {
                let toolbar = Rect { height: 1, ..body };
                let editor_y = inner.y.saturating_add(1);
                let editor = Rect {
                    y: editor_y,
                    height: inner.bottom().saturating_sub(editor_y),
                    width: body.width.saturating_sub(2),
                    ..body
                };
                Some(Self {
                    body,
                    toolbar,
                    editor: Some(editor),
                    seam: None,
                    content: Rect::ZERO,
                })
            }
            QueryPaneMode::Results => {
                let toolbar = Rect { height: 1, ..body };
                let content_y = inner.y.saturating_add(1);
                let content = Rect {
                    y: content_y,
                    height: inner.bottom().saturating_sub(content_y),
                    ..body
                };
                Some(Self {
                    body,
                    toolbar,
                    editor: None,
                    seam: None,
                    content,
                })
            }
        }
    }

    fn legacy(area: Rect, inner: Rect, body: Rect, content_kind: QueryContent) -> Option<Self> {
        if inner.height < 8 {
            return None;
        }
        let top = idle_query_top(inner.height);
        let editor_rows = query_editor_rows(top);
        let editor_y = body.y.saturating_add(1);
        let editor = Rect {
            y: editor_y,
            height: editor_rows,
            width: body.width.saturating_sub(2),
            ..body
        };
        let seam_y = editor.y.saturating_add(editor.height);
        let seam = Rect {
            y: seam_y,
            height: 1,
            ..body
        };
        let content_y = seam_y.saturating_add(1);
        let content = if content_kind == QueryContent::Idle {
            Rect {
                x: area.x.saturating_add(1),
                y: content_y,
                width: area.width.saturating_sub(2),
                height: inner.bottom().saturating_sub(content_y),
            }
        } else {
            Rect {
                y: content_y,
                height: body.bottom().saturating_sub(content_y),
                ..body
            }
        };
        Some(Self {
            body,
            toolbar: Rect { height: 1, ..body },
            editor: Some(editor),
            seam: Some(seam),
            content,
        })
    }
}

pub(super) fn idle_query_top(height: u16) -> u16 {
    let ratio = u16::try_from(u32::from(height).saturating_mul(38).saturating_add(50) / 100)
        .unwrap_or(height);
    ratio.max(4).min(height.saturating_sub(6).max(4))
}

pub(super) fn query_editor_rows(top: u16) -> u16 {
    let rows = top.saturating_sub(2);
    if top > 3 { rows.max(3) } else { rows }
}
