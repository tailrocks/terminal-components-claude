//! Keyed PickerChain component for sequential multi-stage selection workflows.
//!
//! Manages navigation between ordered picker stages, preserving per-stage queries,
//! highlighted keys, and scroll state across backtracking and revisions.

use crate::termrock::empty::Readiness;
use crate::termrock::identity::{Id, ItemKey, Revision};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::picker::{Picker, PickerAction, PickerItem, PickerState};
use crate::termrock::response::{Flow, Invalidate, Response};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// A single step in a [`PickerChain`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerChainStep<'a> {
    pub key: ItemKey,
    pub title: &'a str,
    pub items: &'a [PickerItem<'a>],
}

impl<'a> PickerChainStep<'a> {
    pub fn new(key: impl Into<ItemKey>, title: &'a str, items: &'a [PickerItem<'a>]) -> Self {
        Self {
            key: key.into(),
            title,
            items,
        }
    }
}

/// Compatibility alias for [`PickerChainStep`].
pub type PickerStage<'a> = PickerChainStep<'a>;

/// Durable view state for [`PickerChain`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PickerChainState {
    pub step_index: usize,
    pub selections: Vec<(ItemKey, ItemKey)>,
    pub picker_state: PickerState,
    pub stage_states: Vec<PickerState>,
}

impl PickerChainState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step_index(&self) -> usize {
        self.step_index
    }

    pub fn selections(&self) -> &[(ItemKey, ItemKey)] {
        &self.selections
    }

    pub fn current_selection(&self, step: ItemKey) -> Option<ItemKey> {
        self.selections
            .iter()
            .find(|(s, _)| *s == step)
            .map(|(_, item)| *item)
    }

    pub fn picker_state(&self) -> &PickerState {
        &self.picker_state
    }

    pub fn picker_state_mut(&mut self) -> &mut PickerState {
        &mut self.picker_state
    }
}

/// Typed action emitted by [`PickerChain`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerChainAction {
    Advance { step: ItemKey, chosen: ItemKey },
    Back,
    Complete { selections: Vec<(ItemKey, ItemKey)> },
    Dismissed,
}

/// Multi-step sequential drilldown picker chain.
#[derive(Clone)]
pub struct PickerChain<'a> {
    pub id: Id,
    pub steps: &'a [PickerChainStep<'a>],
    pub revision: Revision,
    pub readiness: Option<Readiness<'a>>,
    pub patch: StylePatch,
}

impl<'a> PickerChain<'a> {
    pub fn new(id: Id, steps: &'a [PickerChainStep<'a>], revision: Revision) -> Self {
        Self {
            id,
            steps,
            revision,
            readiness: None,
            patch: StylePatch::empty(),
        }
    }

    pub fn readiness(mut self, readiness: Readiness<'a>) -> Self {
        self.readiness = Some(readiness);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    fn current_step(&self, state: &PickerChainState) -> Option<&'a PickerChainStep<'a>> {
        self.steps.get(state.step_index)
    }

    fn build_picker(&self, state: &PickerChainState) -> Option<Picker<'a>> {
        let step = self.current_step(state)?;
        let mut picker = Picker::new(
            self.id.sub("step").sub(&step.key.to_string()),
            step.items,
            self.revision,
        )
        .title(step.title)
        .patch(self.patch);

        if let Some(r) = self.readiness {
            picker = picker.readiness(r);
        }
        Some(picker)
    }

    /// Handles events, advancing through steps or backtracking.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut PickerChainState,
    ) -> Response<PickerChainAction> {
        let step = match self.current_step(state) {
            Some(s) => s,
            None => {
                return Response::action(self.id.clone(), PickerChainAction::Dismissed);
            }
        };

        let picker = match self.build_picker(state) {
            Some(p) => p,
            None => {
                return Response::action(self.id.clone(), PickerChainAction::Dismissed);
            }
        };

        let resp = picker.update(cx, &mut state.picker_state);

        match resp.action {
            Some(PickerAction::Accept { key: chosen, .. }) => {
                state.selections.retain(|(s, _)| *s != step.key);
                state.selections.push((step.key, chosen));

                if state.step_index + 1 < self.steps.len() {
                    if state.stage_states.len() <= state.step_index {
                        state
                            .stage_states
                            .resize(state.step_index + 1, PickerState::new());
                    }
                    state.stage_states[state.step_index] = state.picker_state.clone();
                    state.step_index += 1;

                    if state.stage_states.len() > state.step_index {
                        state.picker_state = state.stage_states[state.step_index].clone();
                    } else {
                        state.picker_state = PickerState::new();
                    }

                    cx.request_invalidate(Invalidate::Paint);
                    Response::action(
                        self.id.clone(),
                        PickerChainAction::Advance {
                            step: step.key,
                            chosen,
                        },
                    )
                    .with_flow(Flow::Consumed)
                    .with_invalidate(Invalidate::Paint)
                } else {
                    cx.request_invalidate(Invalidate::Paint);
                    Response::action(
                        self.id.clone(),
                        PickerChainAction::Complete {
                            selections: state.selections.clone(),
                        },
                    )
                    .with_flow(Flow::Consumed)
                    .with_invalidate(Invalidate::Paint)
                }
            }
            Some(PickerAction::Back) => {
                if state.step_index > 0 {
                    state.step_index -= 1;
                    state.selections.pop();
                    if let Some(saved) = state.stage_states.get(state.step_index) {
                        state.picker_state = saved.clone();
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::action(self.id.clone(), PickerChainAction::Back)
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint)
                } else {
                    cx.request_invalidate(Invalidate::Paint);
                    Response::action(self.id.clone(), PickerChainAction::Dismissed)
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint)
                }
            }
            Some(PickerAction::Dismissed) => {
                cx.request_invalidate(Invalidate::Paint);
                Response::action(self.id.clone(), PickerChainAction::Dismissed)
                    .with_flow(Flow::Consumed)
                    .with_invalidate(Invalidate::Paint)
            }
            _ => Response {
                id: self.id.clone(),
                flow: resp.flow,
                invalidate: resp.invalidate,
                state: resp.state,
                action: None,
            },
        }
    }

    /// Renders the active step picker modal.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &PickerChainState) -> Rect {
        if let Some(picker) = self.build_picker(state) {
            picker.draw(ui, area, &state.picker_state)
        } else {
            Rect::zero()
        }
    }

    /// Measures the active step picker modal under constraints.
    pub fn measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let dummy_state = PickerChainState::new();
        if let Some(picker) = self.build_picker(&dummy_state) {
            picker.measure(cx, constraints)
        } else {
            Size::zero()
        }
    }
}
