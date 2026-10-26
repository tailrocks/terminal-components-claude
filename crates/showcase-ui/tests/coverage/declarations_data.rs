// Historical page/title leads carried forward from the Agent-B candidate.
// These lexical associations are navigation hints, not behavior proofs.

use super::{BehaviorProof, PageLead, ProofKind};

/// Existing page associations retained as review leads only.
pub static PAGE_LEADS: &[(&str, PageLead)] = &[
    (
        "termrock-collections::collection::key::SelectMode",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-collections::collection::key::SelectMode::Multi",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-collections::scroll::ScrollState",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-collections::scroll::ScrollState::clamp",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-collections::scroll::ScrollState::offset",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-collections::scroll::ScrollState::scroll_to",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-collections::scroll::ScrollState::set_viewport",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-collections::scroll::ScrollState::thumb",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-controls::brand::Brand",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::brand::Brand::clickable",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-controls::brand::Brand::draw",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::brand::Brand::tagline",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::brand::Brand::update",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::button::BusyMarkerPolicy",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::BusyMarkerPolicy::Replace",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::busy_marker_policy",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::checked",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::disabled",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::draw",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::icon",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-controls::button::Button::measure",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::patch_part",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::slot",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::status",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::update",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::button::Button::variant",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::choice::Checkbox",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Checkbox::checked",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Checkbox::disabled",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Checkbox::draw",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Checkbox::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Checkbox::update",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroup",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroup::disabled",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroup::draw",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroup::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroup::update",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroup::value",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::RadioGroupState",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Toggle",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Toggle::disabled",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Toggle::draw",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Toggle::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Toggle::on",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::choice::Toggle::update",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::empty::Empty",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-controls::empty::Empty::draw",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-controls::empty::Empty::frame",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-controls::empty::Empty::patch_part",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-controls::empty::Empty::state",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-controls::panel::Panel",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::draw",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::inner",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-controls::panel::Panel::kind",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::measure",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::meta",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::patch_part",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::slot",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::Panel::title",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::PanelKind",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::PanelKind::Card",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-controls::panel::PanelKind::Framed",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-controls::props::PropsList",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::props::PropsList::draw",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::props::PropsList::update",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::props::PropsList::variant",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-controls::props::PropsRow",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::props::PropsRow::wrap",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-controls::props::PropsState",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::action::Action",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-core::action::Action::enabled",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-core::action::Action::key",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-core::action::Action::label",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-core::action::Action::variant",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-core::action::ActionKey",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-core::event::Chord",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::geometry::Axes",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-core::geometry::Axes::V",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-core::geometry::Headroom",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-core::id::Id",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::id::Id::index",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::id::Id::part",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::id::ItemKey",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::id::ItemKey::Index",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-core::id::ItemKey::Num",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-core::id::Part",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::id::PartRef",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::intent::Intent",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-core::intent::Intent::Binding",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-core::keys::KeyCode",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::keys::KeyCode::Char",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::keys::KeyCode::Enter",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-core::keys::keyboard::active",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-core::keys::keyboard::push",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-core::layer_types::LayerId",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::layer_types::LayerId::index",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::list::Moved",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-core::response::Activated",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-core::response::Response",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::response::Response::action_ref",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-core::response::Response::activated",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::response::Response::erase",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-core::response::Response::is_changed",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-core::response::Response::repaint",
        PageLead {
            page: "progress",
            title_hint: "Progress",
        },
    ),
    (
        "termrock-core::response::Response::state",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-core::validate::FieldError",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-core::variant::Variant",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-editors::code::CodeAction",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeAction::Changed",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeAction::Committed",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeAction::CursorMoved",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeAction::Leave",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeDiagnostic",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::blocks",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::current_block",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::draw",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::highlighter",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::indent",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::open_find",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::placeholder",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::segmenter",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::tab_behavior",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::update",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeEditor::with_completion",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeSeverity",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeSeverity::Error",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeSeverity::Info",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::CodeSeverity::Warning",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::TabBehavior",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::code::TabBehavior::Indent",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditor",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditor::draw",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditor::open_find",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditor::update",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditorState",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditorState::editor",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditorState::editor_mut",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletingCodeEditorState::is_completion_open",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionBatch",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionTrigger",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionTrigger::Manual",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionTrigger::TextChanged",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionValue",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionValue::detail",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionValue::glyph",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-editors::completion::CompletionValue::matched",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-feedback::meter::Meter",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::meter::Meter::draw",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::meter::Meter::frame",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::meter::MeterTone",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::meter::MeterTone::Stale",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar::draw",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar::frame",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar::label",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar::measure",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar::ratio",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::ProgressBar::variant",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-feedback::progress::Spinner",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-feedback::progress::Spinner::draw",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-feedback::progress::Spinner::frame",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-feedback::progress::Spinner::measure",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-feedback::progress::Spinner::variant",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-feedback::status::Group",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-feedback::status::Group::Center",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::status::Group::Left",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-feedback::status::Group::Right",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-fields::field::Field",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::field::Field::draw",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::field::Field::error",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::field::Field::help",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::field::Field::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-fields::field::Field::optional_suffix",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::field::Field::required",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::BlurPolicy",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::BlurPolicy::Commit",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::BlurPolicy::CommitAndValidate",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextAction",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextAction::Cancelled",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextAction::Changed",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextAction::Committed",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextAction::MoveNext",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextAction::MovePrev",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::blur",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::disabled",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::draw",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-fields::input::TextInput::placeholder",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::status",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::update",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::validate",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInput::value",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInputState",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInputState::blur",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInputState::error",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-fields::input::TextInputState::is_editing",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::input::TextInputState::set_error",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-fields::textarea::TextArea",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::disabled",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::draw",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::placeholder",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::read_only",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::status",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::update",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::validate",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-fields::textarea::TextArea::value",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextAreaState",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextAreaState::error",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextAreaState::is_editing",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-fields::textarea::TextAreaState::set_error",
        PageLead {
            page: "textareas",
            title_hint: "Text areas",
        },
    ),
    (
        "termrock-forms::form::FieldKind",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldKind::Area",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldKind::Radio",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldKind::Select",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldMut",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldMut::Choice",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldMut::ReadOnly",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldMut::Text",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldRef",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldRef::Choice",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldRef::Text",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldSpec",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldSpec::help",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FieldSpec::required",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::Form",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::Form::actions",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::Form::draw",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::Form::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::Form::submit",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::Form::update",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FormState",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::form::FormState::error",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::Wizard",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::Wizard::draw",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::Wizard::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::Wizard::update",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::WizardState",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::WizardStep",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-forms::wizard::WizardStep::enabled",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-grid::grid::CellRef",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::CellRef::align",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::CellRef::tone",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::grid::Column",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::ColumnKey",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::EditIntent",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::EditIntent::Inline",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::EditIntent::Refuse",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::Grid",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::Grid::cell",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::Grid::draw",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::Grid::nav",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::Grid::patch_part",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::Grid::update",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::Grid::update_editable",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::GridAction",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::GridAction::Activated",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::GridState",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::GridState::edit_error",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::GridState::is_editing",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::NavUnit",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::NavUnit::Cell",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-grid::grid::NavUnit::Row",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-grid::grid::SortDir",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::grid::SortDir::Asc",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::Table",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::Table::draw",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::Table::position_label",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::Table::tone_styles",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::Table::update",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableAction",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableAction::Grid",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableColumn",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableColumn::flexible",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableColumn::sort",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableSort",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableSort::NumericDisplay",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableState",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-grid::table::TableState::sort",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-layout::composition_plan::Composition",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-layout::composition_plan::Composition::gap",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-layout::composition_plan::row",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-layout::composition_plan::wrap",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-layout::layout::RowAlign",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-layout::layout::RowAlign::Start",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-layout::layout::Track",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-layout::layout::Track::Fixed",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-layout::layout::Track::Flex",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-layout::layout::columns",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-layout::measure::Constraints",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar::closable",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar::draw",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar::key",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar::row",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar::select_mode",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBar::update",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarAction",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarAction::Activated",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarAction::AddRequested",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarAction::Closed",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarAction::Toggled",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarState",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::chip::ChipBarState::checked_mut",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::draw",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::filter",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::frame",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::placeholder",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::searchable",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::selection_policy",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterList::update",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterListState",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterListState::cursor",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterListState::query",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterListState::scroll",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::FilterListState::set_cursor",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::SelectionPolicy",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::filter_list::SelectionPolicy::FirstEnabledOnFilterChange",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::item::Item",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::item::Item::detail",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::item::Item::disabled",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::item::Item::glyph",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::item::Item::group",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::item::Item::tag",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::list::List",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::disabled_item",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::draw",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::empty",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::key",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::patch_part",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::row",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::select_mode",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::table",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::List::update",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListAction",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListAction::Activated",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListAction::Chose",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListAction::Moved",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::list::ListAction::Toggled",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListAction::ToggledAll",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListState",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListState::checked",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListState::checked_mut",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListState::choose",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::list::ListState::cursor",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::list::ListState::scroll",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::badge",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::disabled_item",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::draw",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::icon",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::key",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::mode",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::row",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::section",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::update",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavList::width",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavListAction",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavListAction::Chose",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavListAction::EnterContent",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavListState",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavMode",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavMode::Collapsed",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::nav_list::NavMode::Full",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-navigation::steps::StepState",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepState::Blocked",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepState::Done",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepState::Failed",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepState::Queued",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepState::Running",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepState::Skipped",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::StepState::label",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::Steps",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::Steps::disabled",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::Steps::draw",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::Steps::key",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::Steps::measure",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::Steps::metadata",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::Steps::row",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::Steps::step",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::Steps::update",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::steps::StepsAction",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::StepsAction::Activated",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-navigation::steps::StepsState",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-navigation::tabs::Tabs",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::tabs::Tabs::draw",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::tabs::Tabs::key",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tabs::Tabs::row",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tabs::Tabs::update",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-navigation::tabs::TabsAction",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tabs::TabsAction::Activated",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tabs::TabsState",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tabs::TabsState::active",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tabs::TabsState::cursor",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-navigation::tree::Tree",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::Tree::draw",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::Tree::key",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::Tree::node",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::Tree::patch_part",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::Tree::row",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::Tree::update",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeAction",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeAction::Activated",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeAction::Chose",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeAction::Collapsed",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeAction::Expanded",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeAction::Moved",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeNode",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeNode::depth",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeNode::has_children",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeNode::key",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeNode::kind",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState::chosen",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState::cursor",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState::expand",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState::expanded",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState::is_expanded",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-navigation::tree::TreeState::scroll",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::draw",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::error",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::layer",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::measure",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::meta",
        PageLead {
            page: "settings",
            title_hint: "Settings",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::patch_part",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::title",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::update",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::Dialog::width",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::DialogAction",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::DialogAction::Action",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::DialogAction::Dismissed",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::DialogState",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::dialog::DialogState::draft",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-overlays::help::HelpOverlay",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-overlays::help::HelpOverlay::draw",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-overlays::help::HelpOverlay::update",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-overlays::help::HelpOverlayState",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-overlays::help::HelpSection",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-overlays::help::HelpSection::title",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-overlays::menu::ContextMenu",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::ContextMenu::draw",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::ContextMenu::layer",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::ContextMenu::title",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::ContextMenu::update",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::Menu",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuBar",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuBar::draw",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuBar::update",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuItem",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuItem::chord",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuItem::disabled",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuState",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuState::cursor",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::menu::MenuState::is_open",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::align",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::draw",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::filter",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::hints",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::layer",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::placeholder",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::scopes",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::searchable",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::selection_policy",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::size",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::title",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::update",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::Picker::width",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction::Back",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction::Chosen",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction::ChosenAlt",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction::QueryChanged",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction::Scope",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerAction::Secondary",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerState",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerState::cursor",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerState::list",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerState::query",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::PickerState::set_cursor",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::ScopeKey",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker::ScopeKey::get",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker_chain::PickerChain",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker_chain::PickerChain::draw",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker_chain::PickerChain::update",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker_chain::PickerChainState",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::picker_chain::PickerStage",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-overlays::select::Select",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::Select::disabled",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::Select::draw",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::Select::key",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-overlays::select::Select::measure",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::Select::placeholder",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::Select::popup_rows",
        PageLead {
            page: "celleditors",
            title_hint: "Cell editors",
        },
    ),
    (
        "termrock-overlays::select::Select::row",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-overlays::select::Select::update",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::SelectAction",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-overlays::select::SelectAction::Chose",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock-overlays::select::SelectState",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::SelectState::set_value",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-overlays::select::SelectState::value",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-runtime::decor::CellDecor",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-runtime::decor::RowDecor",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-runtime::empty::EmptyState",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::empty::EmptyState::Empty",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::empty::EmptyState::draw",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::empty::EmptyState::title",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::empty::RowTotal",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-runtime::empty::RowTotal::Exact",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-runtime::empty::Status",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::empty::Status::Busy",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::empty::Status::Error",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-runtime::empty::Status::Ready",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-runtime::focus::Focusability",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::focus::Focusability::Disabled",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-runtime::focus::Focusability::Focusable",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::keymap::Binding",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-runtime::keymap::Hint",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::keymap::HintKey",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::keymap::HintKey::Chord",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::keymap::HintLayer",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::keymap::HintLayer::is_empty",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::layer::LayerSize",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-runtime::layer::LayerSize::Fixed",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-runtime::layer::ScreenAlign",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-runtime::layer::ScreenAlign::Center",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-runtime::rowui::RowUi",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::rowui::RowUi::columns",
        PageLead {
            page: "trees",
            title_hint: "Trees",
        },
    ),
    (
        "termrock-runtime::rowui::RowUi::key",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::rowui::RowUi::label",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::rowui::RowUi::meta",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::runtime::Moment",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::runtime::Moment::saturating_add",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::runtime::UpdateCause",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::runtime::UpdateCause::Bootstrap",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-runtime::runtime::UpdateCause::Tick",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::runtime::stub::key",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::runtime::stub::step",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-runtime::scroll_region::ScrollRegion",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-runtime::scroll_region::ScrollRegion::draw",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-runtime::scroll_region::ScrollRegion::preserve_underlay",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-runtime::scroll_region::ScrollRegion::update",
        PageLead {
            page: "chrome",
            title_hint: "Chrome",
        },
    ),
    (
        "termrock-runtime::ui::ReferenceState",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::ReferenceState::union",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::ReferenceTarget",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::ReferenceTarget::part",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::Ui",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::bg",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::fill",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::frame",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::full",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::glyph",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::Ui::is_inert",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::layer",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-runtime::ui::Ui::paint_str",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::publish_additional_bindings",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-runtime::ui::Ui::publish_bindings",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::reference",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::Ui::register_control",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::register_part",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::register_scroll",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::report_layout",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::set_cursor",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-runtime::ui::Ui::style",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-runtime::ui::Ui::surface_style",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::Ui::theme_ref",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::Ui::with_area",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-runtime::ui::Ui::with_surface",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::close_layer",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::command",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::focus",
        PageLead {
            page: "forms",
            title_hint: "Forms",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::intents",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::is_open",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::now",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::open_layer",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::request_repaint_after",
        PageLead {
            page: "terminal",
            title_hint: "Terminal",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::request_repaint_at",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::top_layer",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::cx::Cx::update_cause",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-runtime::ui::cx::LayoutFacts",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-shell::footer::Hint",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-shell::quit::Leave",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-shell::router::chord",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-text::measure::truncate",
        PageLead {
            page: "inputs",
            title_hint: "Inputs",
        },
    ),
    (
        "termrock-text::measure::width",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-text::measure::wrap",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-text::measure::wrapped_rows",
        PageLead {
            page: "sidebars",
            title_hint: "Sidebars",
        },
    ),
    (
        "termrock-theme::Theme",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::Theme::bg",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::Theme::builder",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::Theme::capability",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::Theme::color",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::Theme::design",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::glyph::GlyphRole",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::glyph::GlyphRole::DotOff",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::glyph::GlyphRole::DotOn",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::glyph::GlyphRole::FocusBar",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::glyph::GlyphRole::ScrollThumb",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-theme::glyph::GlyphRole::ScrollTrack",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-theme::patch::StylePatch",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::patch::StylePatch::add",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::patch::StylePatch::is_empty",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::patch::StylePatch::remove",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::patch::StylePatch::set_bg",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::patch::StylePatch::set_fg",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::patch::StylePatch::set_glyph",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-theme::recipe::Overlay",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::resolve::PaintStyle",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::resolve::PaintStyle::add_modifier",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::resolve::PaintStyle::as_style",
        PageLead {
            page: "grid",
            title_hint: "Data grid",
        },
    ),
    (
        "termrock-theme::resolve::PaintStyle::bg",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::resolve::PaintStyle::fg",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::resolve::PaintStyle::remove_modifier",
        PageLead {
            page: "lists",
            title_hint: "Lists",
        },
    ),
    (
        "termrock-theme::role::Align",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-theme::role::Align::Center",
        PageLead {
            page: "pickers",
            title_hint: "Pickers",
        },
    ),
    (
        "termrock-theme::role::Align::Left",
        PageLead {
            page: "editable",
            title_hint: "Editable tables",
        },
    ),
    (
        "termrock-theme::role::Align::Right",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-theme::role::FgStep",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::FgStep::Muted",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-theme::role::FgStep::Primary",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-theme::role::FgStep::Secondary",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::FgStep::index",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Role",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Role::Accent",
        PageLead {
            page: "dialogs",
            title_hint: "Dialogs",
        },
    ),
    (
        "termrock-theme::role::Role::Custom",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Role::Danger",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Role::Fg",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Role::Focus",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Role::Warning",
        PageLead {
            page: "tables",
            title_hint: "Tables",
        },
    ),
    (
        "termrock-theme::role::Surface",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::Surface::Overlay",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Comment",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Constant",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Function",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Keyword",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Number",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Operator",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Plain",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Punct",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::Str",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::role::SyntaxRole::TypeName",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock-theme::tokens::ColorLevel",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::tokens::ColorLevel::Ansi16",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::tokens::ColorLevel::Ansi256",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::tokens::ColorLevel::Mono",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-theme::tokens::ColorLevel::TrueColor",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock-viewport::viewport::TextViewport",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::TextViewport::draw",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::TextViewport::update",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::TextViewport::wrap",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::ViewportLine",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::ViewportLine::Plain",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::ViewportState",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::ViewportState::scroll",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock-viewport::viewport::ViewportState::set_follow",
        PageLead {
            page: "scrolling",
            title_hint: "Scrolling",
        },
    ),
    (
        "termrock::composition::Group",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock::composition::Group::into_child",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock::composition::Group::show",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock::composition::Group::spec",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock::composition::button",
        PageLead {
            page: "buttons",
            title_hint: "Buttons",
        },
    ),
    (
        "termrock::composition::code_editor",
        PageLead {
            page: "editor",
            title_hint: "Code editor",
        },
    ),
    (
        "termrock::composition::props_list",
        PageLead {
            page: "overview",
            title_hint: "Overview",
        },
    ),
    (
        "termrock::composition::select",
        PageLead {
            page: "chips",
            title_hint: "Chips & selects",
        },
    ),
    (
        "termrock::composition::steps",
        PageLead {
            page: "taskrunner",
            title_hint: "Task runner",
        },
    ),
];

/// Small current-head examples for distinct declaration forms.
pub static PROOFS: &[BehaviorProof] = &[
    BehaviorProof {
        capability: "termrock-editors::diff::DiffView",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Demo,
        probe: "config/service.toml",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffMode::Review",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Option,
        probe: " · review",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffViewState::set_mode",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Option,
        probe: " · review",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffViewState",
        page: "diff",
        route: "Diff",
        kind: ProofKind::StateRow,
        probe: "No file selected",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffView::draw",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Via,
        probe: "config/service.toml",
        owner: Some("termrock-editors::diff::DiffView"),
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffView::update",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Via,
        probe: " · review",
        owner: Some("termrock-editors::diff::DiffView"),
    },
    BehaviorProof {
        capability: "termrock-fields::textarea::TextArea::edit_entry",
        page: "textareas",
        route: "Text areas",
        kind: ProofKind::Option,
        probe: "Type to edit",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-fields::textarea::TextArea::focus_select",
        page: "textareas",
        route: "Text areas",
        kind: ProofKind::Option,
        probe: "Select all",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-fields::textarea::TextArea::esc_reverts",
        page: "textareas",
        route: "Text areas",
        kind: ProofKind::Option,
        probe: "Revert",
        owner: None,
    },
];
