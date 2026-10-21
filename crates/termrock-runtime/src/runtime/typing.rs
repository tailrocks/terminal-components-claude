use crate::diagnostics::Diagnostic;
use crate::id::Id;
use crate::layer::LayerId;
use crate::response::StateFlags;
use crate::ui::FrameState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TypingPolicy {
    #[default]
    Focused,
    Fallback {
        cursor: bool,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TypingDeclaration {
    pub(crate) owner: Id,
    pub(crate) layer: LayerId,
    pub(crate) cursor: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TypingResolved {
    pub(crate) owner: Option<Id>,
    pub(crate) fallback: Option<Id>,
    pub(crate) cursor: Option<Id>,
}

fn flags(frame: &FrameState, owner: Id) -> StateFlags {
    frame
        .declared
        .iter()
        .find(|(id, _)| *id == owner)
        .map_or(StateFlags::empty(), |(_, flags)| *flags)
}

pub(crate) fn admissible(frame: &FrameState, owner: Id) -> bool {
    frame.ring.entry(owner).is_some_and(|entry| {
        !entry.disabled
            && !entry.area.is_empty()
            && entry.layer == frame.top
            && frame.registry.layer_of(owner) == Some(entry.layer)
            && frame
                .ring
                .active_trap()
                .is_none_or(|scope| frame.ring.within(entry.scope, scope))
    })
}

fn editable(frame: &FrameState, owner: Id) -> bool {
    admissible(frame, owner)
        && frame
            .ring
            .entry(owner)
            .is_some_and(|entry| entry.swallows_typing)
        && flags(frame, owner).contains(StateFlags::EDITING)
        && !flags(frame, owner).intersects(StateFlags::READ_ONLY | StateFlags::DISABLED)
}

pub(crate) fn resolve(frame: &mut FrameState, focus: Option<Id>) -> TypingResolved {
    let focus = focus.filter(|owner| admissible(frame, *owner));
    let mut resolved = TypingResolved {
        cursor: focus,
        ..TypingResolved::default()
    };
    let mut candidate = None;
    let mut ambiguous = false;
    for declaration in &frame.typing {
        if declaration.layer != frame.top || !editable(frame, declaration.owner) {
            continue;
        }
        if let Some((first, _)) = candidate {
            frame.diagnostics.push(Diagnostic::TypingTargetConflict {
                a: first,
                b: declaration.owner,
                layer: frame.top,
            });
            ambiguous = true;
        } else {
            candidate = Some((declaration.owner, declaration.cursor));
        }
    }

    if let Some(owner) = focus
        && frame
            .ring
            .entry(owner)
            .is_some_and(|entry| entry.swallows_typing)
    {
        resolved.owner = editable(frame, owner).then_some(owner);
        if flags(frame, owner).contains(StateFlags::READ_ONLY) {
            // Read-only controls may still expose a visible caret for navigation or
            // inspection. Preserve that cursor only when the control itself published
            // an ordinary cursor request; read-only focus must not manufacture one.
            resolved.cursor = frame
                .cursors
                .iter()
                .any(|request| {
                    request.owner == owner
                        && request.layer == frame.top
                        && !request.inert
                        && !request.typing_offer
                })
                .then_some(owner);
        }
        return resolved;
    }
    if !ambiguous && let Some((owner, cursor)) = candidate {
        resolved.owner = Some(owner);
        resolved.fallback = Some(owner);
        if cursor {
            resolved.cursor = Some(owner);
        }
    }
    resolved
}
