use std::time::Duration;

use termrock::{
    Action, ActionKey, BlurPolicy, Button, Checkbox, Constraints, Cx, Family, Field, FieldError,
    FieldKind, FieldMut, FieldRef, FieldSpec, Form, FormData, FormState, FrameRead, Id, ItemKey,
    Moment, Panel, PanelKind, Part, RadioGroup, RadioGroupState, Rect, Response, RowAlign, Select,
    SelectState, StateFlags, Status, TextArea, TextAreaState, TextInput, TextInputState, Toggle,
    Track, Ui, UpdateCause, Variant, Wizard, WizardState, WizardStep, layout, truncate,
};

use super::{Page, PageStatus, PageUpdate, frame};

pub const SUBMIT: ActionKey = ActionKey::application("showcase.form.submit");

const SUMMARY: Id = termrock::Id::root("showcase_ui::pages::forms::forms.summary");
const DETAILS: Id = termrock::Id::root("showcase_ui::pages::forms::forms.details");
const PRIORITY: Id = termrock::Id::root("showcase_ui::pages::forms::forms.priority");
const CONFIRM: Id = termrock::Id::root("showcase_ui::pages::forms::forms.confirm");
const SAVE: Id = termrock::Id::root("showcase_ui::pages::forms::forms.save");
const REVIEWER: Id = termrock::Id::root("showcase_ui::pages::forms::forms.reviewer");
const MODE: Id = termrock::Id::root("showcase_ui::pages::forms::forms.mode");
const RUN_TESTS: Id = termrock::Id::root("showcase_ui::pages::forms::forms.run_tests");
const OPEN_PR: Id = termrock::Id::root("showcase_ui::pages::forms::forms.open_pr");
const AUTO_APPROVE: Id = termrock::Id::root("showcase_ui::pages::forms::forms.auto_approve");
const NOTIFY: Id = termrock::Id::root("showcase_ui::pages::forms::forms.notify");
const RESET: Id = termrock::Id::root("showcase_ui::pages::forms::forms.reset");
const FLOW: Id = termrock::Id::root("showcase_ui::pages::forms::forms.flow");
const DEPLOY_FORM: Id = termrock::Id::root("showcase_ui::pages::forms::forms.deploy.form");
const DEPLOY_PANEL: Id = termrock::Id::root("showcase_ui::pages::forms::forms.deploy.panel");
const DEPLOY_STAGE: Id = termrock::Id::root("showcase_ui::pages::forms::forms.deploy.stage");
const DEPLOY_REGION: Id = termrock::Id::root("showcase_ui::pages::forms::forms.deploy.region");
const DEPLOY_NOTES: Id = termrock::Id::root("showcase_ui::pages::forms::forms.deploy.notes");
const APPLY: ActionKey = ActionKey::custom("showcase.form.apply");
const ACCOUNT: ItemKey = ItemKey::index(0);
const REVIEW: ItemKey = ItemKey::index(1);
const CONFIRM_STEP: ItemKey = ItemKey::index(2);
const LAUNCH: ItemKey = ItemKey::index(3);
const MODES: &[&str] = &["Fast", "Balanced", "Thorough"];
const STAGES: &[&str] = &["Staging", "Production"];
const REGIONS: &[&str] = &["us-east", "eu-west", "ap-south"];
const DEFAULT_DETAILS: &str = "Describe the change and its rollback plan.";
const SUBMISSION_TICKS: u8 = 23;
const SUBMISSION_INTERVAL: Duration = Duration::from_millis(80);

fn task_name_field<'a>(value: &'a str, error: Option<&'a str>) -> Field<'a, TextInput<'a>> {
    Field::new("Task name", FormsPage::summary().value(value))
        .error(error)
        .required(true)
}

fn description_field() -> Field<'static, TextArea<'static>> {
    Field::new("Description", FormsPage::details())
        .optional_suffix(false)
        .help("Optional · Markdown")
}

fn reviewer_email(value: &str) -> Result<(), FieldError> {
    if value.is_empty() || (value.contains('@') && value.contains('.')) {
        Ok(())
    } else {
        Err(FieldError::new("Enter a valid email address"))
    }
}

fn accept_text(_: &str) -> Result<(), FieldError> {
    Ok(())
}

fn reviewer_control<'a>() -> TextInput<'a> {
    TextInput::new(REVIEWER)
        .placeholder("name@company.com")
        .validate(&reviewer_email)
        .blur(BlurPolicy::CommitAndValidate)
}

fn reviewer_field<'a>(value: &'a str, error: Option<&'a str>) -> Field<'a, TextInput<'a>> {
    Field::new("Reviewer", reviewer_control().value(value))
        .help("Optional")
        .error(error)
}

fn mode_group() -> RadioGroup<'static, &'static str> {
    RadioGroup::new(MODE)
}

fn run_tests_checkbox(checked: bool) -> Checkbox<'static> {
    Checkbox::new(RUN_TESTS, "Run tests before opening a PR").checked(checked)
}

fn open_pr_checkbox(checked: bool) -> Checkbox<'static> {
    Checkbox::new(OPEN_PR, "Open a pull request when done").checked(checked)
}

fn auto_approve_toggle(on: bool) -> Toggle<'static> {
    Toggle::new(AUTO_APPROVE, "Auto-approve changes").on(on)
}

fn notify_toggle(on: bool) -> Toggle<'static> {
    Toggle::new(NOTIFY, "Notify on completion")
        .on(on)
        .disabled(true)
}

fn reset_button() -> Button<'static> {
    Button::new(RESET, "Reset").variant(Variant::SUBTLE)
}

fn legacy_gutter(
    ui: &mut Ui<'_>,
    area: Rect,
    family: Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) {
    if area.is_empty() {
        return;
    }
    let container = ui.style(family, variant, part, flags);
    let mut gutter = ui.style(family, variant, Part::GUTTER, flags).style;
    gutter = gutter.with_bg_from(container.style);
    if !flags.contains(StateFlags::FOCUSED) {
        gutter = gutter.with_fg_from_bg(container.style);
    }
    for offset in 0..area.height {
        let _ = ui.paint_str(
            Rect {
                y: area.y.saturating_add(offset),
                height: 1,
                ..area
            },
            "▎",
            gutter,
        );
    }
}

fn legacy_marker(ui: &mut Ui<'_>, area: Rect, flags: StateFlags, text: &str) {
    let style = ui
        .style(Family::CHOICE, Variant::DEFAULT, Part::MARKER, flags)
        .style;
    let _ = ui.paint_str(
        Rect {
            x: area.x.saturating_add(1),
            width: 3,
            ..area
        },
        text,
        style,
    );
}

fn legacy_placeholder(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(3),
        height: 1,
    };
    if inner.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::TEXTAREA,
            Variant::DEFAULT,
            Part::PLACEHOLDER,
            StateFlags::empty(),
        )
        .style;
    let fitted_width = inner.width.saturating_sub(2);
    let fitted = truncate(text, fitted_width);
    let blank = " ".repeat(usize::from(inner.width));
    let _ = ui.paint_str(inner, &blank, style);
    let _ = ui.paint_str(
        Rect {
            width: fitted_width,
            ..inner
        },
        &fitted,
        style,
    );
}

fn legacy_input_placeholder(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(3),
        height: 1,
    };
    if inner.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::INPUT,
            Variant::DEFAULT,
            Part::PLACEHOLDER,
            StateFlags::empty(),
        )
        .style;
    let fitted_width = inner.width.saturating_sub(1);
    let fitted = truncate(text, fitted_width);
    let blank = " ".repeat(usize::from(inner.width));
    let _ = ui.paint_str(inner, &blank, style);
    let _ = ui.paint_str(
        Rect {
            width: fitted_width,
            ..inner
        },
        &fitted,
        style,
    );
}

fn legacy_choice_label(ui: &mut Ui<'_>, area: Rect, flags: StateFlags, text: &str) {
    let label = Rect {
        x: area.x.saturating_add(5),
        width: area.width.saturating_sub(5),
        height: 1,
        ..area
    };
    if label.is_empty() {
        return;
    }
    let style = ui
        .style(Family::CHOICE, Variant::DEFAULT, Part::LABEL, flags)
        .style;
    let fitted = truncate(text, label.width);
    let blank = " ".repeat(usize::from(label.width));
    let _ = ui.paint_str(label, &blank, style);
    let _ = ui.paint_str(label, &fitted, style);
}

#[derive(Debug)]
pub struct FormsPage {
    summary: String,
    details: String,
    reviewer: String,
    summary_state: TextInputState,
    details_state: TextAreaState,
    reviewer_state: TextInputState,
    priority: SelectState,
    mode: usize,
    mode_state: RadioGroupState,
    confirm: bool,
    run_tests: bool,
    open_pr: bool,
    auto_approve: bool,
    notify: bool,
    error: Option<&'static str>,
    submission: Submission,
    next_deadline: Option<Moment>,
    flow: WizardState<()>,
    deploy: Deploy,
    deploy_state: FormState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Submission {
    #[default]
    Idle,
    Busy {
        remaining: u8,
    },
    Done,
}

impl FormsPage {
    pub fn new() -> Self {
        let mut priority = SelectState::default();
        priority.set_value(Some(ItemKey::index(0)));
        Self {
            summary: String::new(),
            details: String::from(DEFAULT_DETAILS),
            reviewer: String::new(),
            summary_state: TextInputState::default(),
            details_state: TextAreaState::default(),
            reviewer_state: TextInputState::default(),
            priority,
            mode: 1,
            mode_state: RadioGroupState::default(),
            confirm: false,
            run_tests: true,
            open_pr: false,
            auto_approve: false,
            notify: true,
            error: None,
            submission: Submission::Idle,
            next_deadline: None,
            flow: WizardState::default(),
            deploy: Deploy::default(),
            deploy_state: FormState::default(),
        }
    }

    fn summary() -> TextInput<'static> {
        TextInput::new(SUMMARY).placeholder("Short imperative summary")
    }

    fn details() -> TextArea<'static> {
        TextArea::new(DETAILS, 4).placeholder("What should Junie do, and what does done look like?")
    }

    fn priority() -> Select<'static, &'static str> {
        Select::new(PRIORITY).placeholder("Priority")
    }

    fn confirmation() -> Checkbox<'static> {
        Checkbox::new(CONFIRM, "I reviewed the rollback plan")
    }

    fn save_button(confirmed: bool, busy: bool) -> Button<'static> {
        let button = Button::new(SAVE, "Create task")
            .variant(Variant::PRIMARY)
            .disabled(!confirmed);
        if busy {
            button.status(Status::Busy)
        } else {
            button
        }
    }
}

#[cfg(test)]
mod task_card_contract {
    use super::*;
    use termrock::{App, KeyCode, PartRef, Theme, Ui};
    use termrock_test_support::Harness;

    struct FormsHost {
        page: FormsPage,
        status_events: Vec<String>,
    }

    impl App for FormsHost {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let update = self.page.update(cx);
            if let Some(PageStatus(message)) = update.status {
                self.status_events.push(message);
            }
            update.response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, ui.full());
        }
    }

    fn harness(width: u16, height: u16) -> Harness<FormsHost> {
        Harness::new(
            FormsHost {
                page: FormsPage::new(),
                status_events: Vec::new(),
            },
            Theme::junie(),
            width,
            height,
        )
    }

    fn set_valid_task(h: &mut Harness<FormsHost>) {
        h.app_mut().page.summary = String::from("Recover the task card");
        h.app_mut().page.details = String::from("Keep the existing deployment path.");
        h.draw();
    }

    fn confirm_task(h: &mut Harness<FormsHost>) {
        assert!(
            h.area_of(CONFIRM).is_some(),
            "Confirm has a native hit area"
        );
        assert!(!h.app().page.confirm);
        let _ = h.click_id(CONFIRM);
        assert!(h.app().page.confirm, "Confirm updates the task model");
    }

    fn assert_task_defaults(page: &FormsPage) {
        assert!(page.summary.is_empty());
        assert_eq!(page.details, DEFAULT_DETAILS);
        assert!(page.reviewer.is_empty());
        assert_eq!(page.priority.value(), Some(ItemKey::index(0)));
        assert_eq!(page.mode, 1);
        assert!(page.run_tests);
        assert!(!page.open_pr);
        assert!(!page.auto_approve);
        assert!(page.notify);
        assert!(!page.confirm);
        assert_eq!(page.submission, Submission::Idle);
        assert_eq!(page.next_deadline, None);
        assert_eq!(page.error, None);
    }

    #[test]
    fn options_are_live_and_reset_preserves_deploy_and_wizard_state() {
        let mut h = harness(120, 50);
        assert_eq!(h.app().page.mode, 1, "Balanced is the initial selection");
        assert!(h.app().page.run_tests);
        assert!(!h.app().page.open_pr);
        assert!(!h.app().page.auto_approve);
        assert!(h.app().page.notify);
        for id in [MODE, RUN_TESTS, OPEN_PR, AUTO_APPROVE, NOTIFY, RESET] {
            assert!(h.area_of(id).is_some(), "missing live owner {id:?}");
        }

        let _ = h.click_part(MODE, PartRef::item(Part::ROW, ItemKey::index(2)));
        let _ = h.click_id(RUN_TESTS);
        let _ = h.click_id(OPEN_PR);
        let _ = h.click_id(AUTO_APPROVE);
        assert_eq!(h.app().page.mode, 2);
        assert!(!h.app().page.run_tests);
        assert!(h.app().page.open_pr);
        assert!(h.app().page.auto_approve);

        assert!(h.area_of(NOTIFY).is_some());
        assert!(
            !h.ring().contains(NOTIFY),
            "disabled Notify is not tabbable"
        );
        let _ = h.click_id(NOTIFY);
        assert!(
            h.app().page.notify,
            "the disabled toggle cannot change value"
        );
        assert!(!h.ring().contains(NOTIFY));

        let _ = h.click_part(DEPLOY_STAGE, PartRef::of(Part::FIELD));
        assert!(
            h.is_open(DEPLOY_STAGE),
            "the existing Deploy select still opens"
        );
        let _ = h.click_part(DEPLOY_STAGE, PartRef::item(Part::ROW, ItemKey::index(1)));
        assert_eq!(h.app().page.deploy.stage, 1);
        let _ = h.click_part(FLOW, PartRef::item(Part::LABEL, CONFIRM_STEP));
        assert_eq!(h.app().page.flow.current(), Some(CONFIRM_STEP));

        set_valid_task(&mut h);
        confirm_task(&mut h);
        let _ = h.click_id(SAVE);
        assert!(matches!(h.app().page.submission, Submission::Busy { .. }));
        assert!(h.text().contains("Creating task…"), "{}", h.text());
        assert!(h.app().page.reviewer.is_empty(), "Reviewer is optional");
        let deadline = h.app().page.next_deadline;
        let _ = h.click_id(SAVE);
        assert_eq!(
            h.app().page.next_deadline,
            deadline,
            "repeated activation cannot restart the Busy deadline"
        );

        let _ = h.click_id(RESET);
        assert_task_defaults(&h.app().page);
        assert_eq!(h.focus(), Some(SUMMARY), "Reset returns focus to Summary");
        assert!(
            h.app()
                .status_events
                .iter()
                .any(|status| status == "Form reset")
        );
        assert_eq!(h.app().page.deploy.stage, 1, "Reset preserves Deploy data");
        assert_eq!(
            h.app().page.flow.current(),
            Some(CONFIRM_STEP),
            "Reset preserves Wizard progress"
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn reviewer_paste_validation_and_active_draft_submit_use_one_path() {
        let mut h = harness(120, 50);
        set_valid_task(&mut h);
        confirm_task(&mut h);

        let reviewer_text = h
            .area_of_part(REVIEWER, PartRef::of(Part::TEXT))
            .expect("the real Reviewer text area is registered");
        let _ = h.click(reviewer_text.x, reviewer_text.y);
        let _ = h.paste("not-an-address");
        assert_eq!(
            h.app().page.reviewer_state.draft_text(),
            Some("not-an-address"),
            "paste enters the live Reviewer draft"
        );
        let _ = h.click_id(SAVE);
        assert_eq!(h.focus(), Some(REVIEWER));
        assert_eq!(h.app().page.submission, Submission::Idle);
        assert!(h.text().contains("valid email"), "{}", h.text());

        let reviewer_text = h
            .area_of_part(REVIEWER, PartRef::of(Part::TEXT))
            .expect("invalid Reviewer remains visible and addressable");
        let _ = h.click(reviewer_text.x, reviewer_text.y);
        let _ = h.ctrl('l');
        let _ = h.paste("dev@example.com");
        assert_eq!(
            h.app().page.reviewer_state.draft_text(),
            Some("dev@example.com")
        );
        let _ = h.click_id(SAVE);
        assert_eq!(h.app().page.reviewer, "dev@example.com");
        assert_eq!(
            h.app().page.submission,
            Submission::Busy {
                remaining: SUBMISSION_TICKS
            }
        );
        assert_eq!(
            h.app().page.next_deadline,
            Some(h.now().saturating_add(SUBMISSION_INTERVAL))
        );
        assert!(h.text().contains("Creating task…"), "{}", h.text());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

        let first_deadline = h.app().page.next_deadline.expect("Busy has a deadline");
        let early = Moment::from_duration(
            first_deadline
                .as_duration()
                .saturating_sub(Duration::from_millis(1)),
        );
        let _ = h
            .advance_to(early)
            .expect("clock moves to just before the deadline");
        let _ = h.tick();
        let _ = h.key(KeyCode::Char('x'));
        h.draw();
        assert_eq!(
            h.app().page.submission,
            Submission::Busy {
                remaining: SUBMISSION_TICKS
            },
            "an early Tick, ordinary Event and draw/Settle do not advance work"
        );

        let delayed_tick = first_deadline.saturating_add(Duration::from_millis(500));
        let _ = h
            .advance_to(delayed_tick)
            .expect("the overdue Tick advances one step");
        assert_eq!(
            h.app().page.submission,
            Submission::Busy {
                remaining: SUBMISSION_TICKS - 1
            }
        );
        assert_eq!(
            h.app().page.next_deadline,
            Some(delayed_tick.saturating_add(SUBMISSION_INTERVAL)),
            "a delayed Tick rearms from now without catch-up"
        );

        for remaining in (1..SUBMISSION_TICKS).rev() {
            let deadline = h.app().page.next_deadline.expect("Busy rearms once");
            let _ = h
                .advance_to(deadline)
                .expect("the next due Tick is accepted");
            if remaining == 1 {
                assert_eq!(h.app().page.submission, Submission::Done);
            } else {
                assert_eq!(
                    h.app().page.submission,
                    Submission::Busy {
                        remaining: remaining - 1
                    }
                );
            }
        }
        assert_eq!(h.app().page.next_deadline, None, "Done stops the clock");
        assert!(h.text().contains("Task created ✓"), "{}", h.text());
        assert!(
            h.app()
                .status_events
                .iter()
                .any(|status| status == "Task created ✓")
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn reviewer_preserves_unicode_paste_and_native_escape_cancellation() {
        let mut h = harness(120, 50);
        let reviewer_text = h
            .area_of_part(REVIEWER, PartRef::of(Part::TEXT))
            .expect("the real Reviewer text area is registered");
        let _ = h.click(reviewer_text.x, reviewer_text.y);
        let _ = h.paste("léa@example.com");
        assert_eq!(
            h.app().page.reviewer_state.draft_text(),
            Some("léa@example.com")
        );
        let _ = h.key(KeyCode::Esc);
        assert!(h.app().page.reviewer.is_empty());
        assert!(!h.app().page.reviewer_state.is_editing());

        let reviewer_text = h
            .area_of_part(REVIEWER, PartRef::of(Part::TEXT))
            .expect("Reviewer remains available after draft cancellation");
        let _ = h.click(reviewer_text.x, reviewer_text.y);
        let _ = h.paste("léa@example.com");
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.reviewer, "léa@example.com");
        assert!(!h.app().page.reviewer_state.is_editing());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn summary_and_details_keep_first_invalid_focus_order() {
        let mut gated_button = harness(120, 50);
        set_valid_task(&mut gated_button);
        let _ = gated_button.click_id(SAVE);
        assert_eq!(gated_button.app().page.submission, Submission::Idle);
        assert!(!gated_button.app().page.confirm);

        let mut summary_first = harness(120, 50);
        summary_first.app_mut().page.reviewer = String::from("invalid");
        summary_first.draw();
        confirm_task(&mut summary_first);
        let _ = summary_first.click_id(SAVE);
        assert_eq!(summary_first.focus(), Some(SUMMARY));
        assert_eq!(summary_first.app().page.submission, Submission::Idle);
        assert!(summary_first.text().contains("Required: summary"));

        let mut details_first = harness(120, 50);
        details_first.app_mut().page.summary = String::from("A valid summary");
        details_first.app_mut().page.details.clear();
        details_first.app_mut().page.reviewer = String::from("invalid");
        details_first.draw();
        confirm_task(&mut details_first);
        let _ = details_first.click_id(SAVE);
        assert_eq!(details_first.focus(), Some(DETAILS));
        assert_eq!(details_first.app().page.submission, Submission::Idle);
        assert_eq!(details_first.app().page.error, Some("Required: details"));
        assert!(summary_first.diagnostics().is_empty());
        assert!(details_first.diagnostics().is_empty());
    }

    #[test]
    fn reset_remains_reachable_in_the_existing_short_card_geometry() {
        let mut h = harness(72, 20);
        let reset = h
            .area_of(RESET)
            .expect("the clipped task-card Reset stays reachable");
        assert!(reset.width > 0 && reset.height > 0);
        assert!(reset.right() <= 72 && reset.bottom() <= 20, "{reset:?}");
        let _ = h.click_id(RESET);
        assert_task_defaults(&h.app().page);
        assert_eq!(h.focus(), Some(SUMMARY));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

fn submission_deadline(now: Moment) -> Option<Moment> {
    let deadline = now.saturating_add(SUBMISSION_INTERVAL);
    (deadline > now).then_some(deadline)
}

fn onboarding_steps() -> [WizardStep<'static>; 4] {
    [
        WizardStep::new(ACCOUNT, "Account"),
        WizardStep::new(REVIEW, "Review").enabled(false),
        WizardStep::new(CONFIRM_STEP, "Confirm"),
        WizardStep::new(LAUNCH, "Launch"),
    ]
}

fn onboarding<'a>(steps: &'a [WizardStep<'static>]) -> Wizard<'a> {
    Wizard::new(FLOW, steps)
}

#[derive(Debug, Default)]
struct Deploy {
    stage: usize,
    region: usize,
    notes: String,
}

impl FormData for Deploy {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            DEPLOY_STAGE => FieldRef::Choice(self.stage),
            DEPLOY_REGION => FieldRef::Choice(self.region),
            DEPLOY_NOTES => FieldRef::Text(&self.notes),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            DEPLOY_STAGE => FieldMut::Choice(&mut self.stage),
            DEPLOY_REGION => FieldMut::Choice(&mut self.region),
            DEPLOY_NOTES => FieldMut::Text(&mut self.notes),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        match id {
            DEPLOY_STAGE => STAGES,
            DEPLOY_REGION => REGIONS,
            _ => &[],
        }
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        match id {
            DEPLOY_STAGE => (FieldMut::Choice(&mut self.stage), STAGES),
            DEPLOY_REGION => (FieldMut::Choice(&mut self.region), REGIONS),
            _ => (FieldMut::ReadOnly, &[]),
        }
    }
}

fn deploy_fields() -> [FieldSpec<'static>; 3] {
    [
        FieldSpec::new(
            DEPLOY_STAGE,
            "Stage",
            FieldKind::Select(Select::new(DEPLOY_STAGE)),
        )
        .required(true),
        FieldSpec::new(
            DEPLOY_REGION,
            "Region",
            FieldKind::Radio(RadioGroup::new(DEPLOY_REGION)),
        )
        .help("Where the release lands"),
        FieldSpec::new(
            DEPLOY_NOTES,
            "Release notes",
            FieldKind::Area(TextArea::new(DEPLOY_NOTES, 2)),
        )
        .help("Optional"),
    ]
}

fn deploy_actions() -> [Action<'static>; 2] {
    [
        Action::new(ActionKey::CANCEL, "Cancel"),
        Action::new(APPLY, "Apply"),
    ]
}

fn deploy_form<'a>(fields: &'a [FieldSpec<'static>], actions: &'a [Action<'a>]) -> Form<'a> {
    Form::new(DEPLOY_FORM, fields)
        .actions(actions)
        .submit(APPLY)
}

fn deploy_panel() -> Panel<'static> {
    Panel::new(DEPLOY_PANEL)
        .kind(PanelKind::Card)
        .title("Deploy form")
        .meta("Form · stage, region, notes")
}

impl FormsPage {
    fn submit_task(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if matches!(self.submission, Submission::Busy { .. }) {
            return Response::ignored();
        }

        // Commit each active draft through its native state before checking
        // the task-card contract. Summary and Details keep their existing
        // required-field order; Reviewer remains optional.
        let _ = self.summary_state.commit(&mut self.summary, &accept_text);
        let _ = self.details_state.commit(&mut self.details, &accept_text);
        let reviewer_result = self
            .reviewer_state
            .commit(&mut self.reviewer, &reviewer_email);

        if self.summary.trim().is_empty() {
            self.reviewer_state.set_error(None);
            self.error = Some("Required: summary");
            cx.focus(SUMMARY);
            return Response::changed();
        }
        if self.details.trim().is_empty() {
            self.reviewer_state.set_error(None);
            self.error = Some("Required: details");
            cx.focus(DETAILS);
            return Response::changed();
        }
        if reviewer_result.is_err() {
            self.error = Some("Invalid reviewer");
            cx.focus(REVIEWER);
            return Response::changed();
        }

        self.error = None;
        self.submission = Submission::Busy {
            remaining: SUBMISSION_TICKS,
        };
        let now = cx.now();
        self.next_deadline = submission_deadline(now);
        if let Some(deadline) = self.next_deadline.filter(|deadline| *deadline > now) {
            cx.request_repaint_at(deadline);
        }
        Response::changed()
    }

    fn reset_task_card(&mut self, cx: &mut Cx<'_>) {
        self.summary.clear();
        self.details = String::from(DEFAULT_DETAILS);
        self.reviewer.clear();
        self.summary_state = TextInputState::default();
        self.details_state = TextAreaState::default();
        self.reviewer_state = TextInputState::default();
        self.priority = SelectState::default();
        self.priority.set_value(Some(ItemKey::index(0)));
        self.mode = 1;
        self.mode_state = RadioGroupState::default();
        self.confirm = false;
        self.run_tests = true;
        self.open_pr = false;
        self.auto_approve = false;
        self.notify = true;
        self.error = None;
        self.submission = Submission::Idle;
        self.next_deadline = None;
        cx.focus(SUMMARY);
    }

    fn service_submission_clock(
        &mut self,
        cx: &mut Cx<'_>,
        response: &mut Response<()>,
    ) -> Option<PageStatus> {
        let Submission::Busy { remaining } = self.submission else {
            self.next_deadline = None;
            return None;
        };

        let now = cx.now();
        if self.next_deadline.is_none() {
            self.next_deadline = submission_deadline(now);
        }
        if cx.update_cause() == UpdateCause::Tick
            && self.next_deadline.is_some_and(|deadline| now >= deadline)
        {
            if remaining == 1 {
                self.submission = Submission::Done;
                self.next_deadline = None;
                *response = (*response).clone().repaint();
                return Some(PageStatus("Task created ✓".to_owned()));
            }

            self.submission = Submission::Busy {
                remaining: remaining - 1,
            };
            self.next_deadline = submission_deadline(now);
            *response = (*response).clone().repaint();
        }
        if let Some(deadline) = self.next_deadline.filter(|deadline| *deadline > now) {
            cx.request_repaint_at(deadline);
        }
        None
    }
}

impl Default for FormsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for FormsPage {
    fn title(&self) -> &'static str {
        "Forms"
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action == SUBMIT {
            self.submit_task(cx)
        } else {
            Response::ignored()
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut result = Response::ignored();
        let mut status = None;
        let _ = description_field();
        result |= Self::summary()
            .update(cx, &mut self.summary_state, &mut self.summary)
            .erase();
        result |= Self::details()
            .update(cx, &mut self.details_state, &mut self.details)
            .erase();
        result |= reviewer_control()
            .update(cx, &mut self.reviewer_state, &mut self.reviewer)
            .erase();
        result |= Self::priority()
            .update(cx, &mut self.priority, &["Normal", "High", "Urgent"])
            .erase();
        result |= mode_group()
            .update_in_form(cx, &mut self.mode_state, &mut self.mode, MODES, false)
            .erase();
        result |= run_tests_checkbox(self.run_tests)
            .update(cx, &mut self.run_tests)
            .erase();
        result |= open_pr_checkbox(self.open_pr)
            .update(cx, &mut self.open_pr)
            .erase();
        result |= auto_approve_toggle(self.auto_approve)
            .update(cx, &mut self.auto_approve)
            .erase();
        result |= notify_toggle(self.notify)
            .update(cx, &mut self.notify)
            .erase();
        result |= Self::confirmation().update(cx, &mut self.confirm).erase();
        let busy = matches!(self.submission, Submission::Busy { .. });
        let save = Self::save_button(self.confirm, busy).update(cx);
        if save.activated() {
            result |= self.submit_task(cx);
        }
        result |= save.erase();

        let reset = reset_button().update(cx);
        if reset.activated() {
            self.reset_task_card(cx);
            status = Some(PageStatus("Form reset".to_owned()));
            result |= Response::changed();
        }
        result |= reset.erase();

        let fields = deploy_fields();
        let actions = deploy_actions();
        result |= deploy_form(&fields, &actions)
            .update(cx, &mut self.deploy_state, &mut self.deploy)
            .erase();
        let steps = onboarding_steps();
        result |= onboarding(&steps).update(cx, &mut self.flow).erase();
        let _ = deploy_panel();
        if status.is_none() {
            status = self.service_submission_clock(cx, &mut result);
        }
        PageUpdate {
            response: result,
            status,
        }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let _ = Self::priority();
        let _ = Self::summary();
        let _ = Self::details();
        let _ = Self::confirmation();
        let panel_width = area.width.min(70);
        frame(
            ui,
            area,
            self.title(),
            "Sections, required fields, validation, submission",
            |ui, body| {
                let regions =
                    layout::rows(body, &[Track::Fixed(1), Track::Fixed(1), Track::Flex(1)]);
                let steps = onboarding_steps();
                onboarding(&steps).draw(ui, regions.first().copied().unwrap_or(body), &self.flow);
                let rest = regions.get(2).copied().unwrap_or(body);
                let slots =
                    layout::rows(rest, &[Track::Fixed(rest.height.min(24)), Track::Flex(1)]);
                let card_body = slots.first().copied().unwrap_or(rest);
                self.draw_task_card(ui, card_body, panel_width);
                if let Some(deploy_area) = slots.get(1).copied() {
                    self.draw_deploy_form(ui, deploy_area);
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.summary_state.is_editing()
            || self.details_state.is_editing()
            || self.reviewer_state.is_editing()
        {
            &[
                ("Enter", "Commit"),
                ("Esc", "Cancel"),
                ("Tab", "Next field"),
            ]
        } else if ui.state(PRIORITY).contains(StateFlags::FOCUSED)
            || ui.state(MODE).contains(StateFlags::FOCUSED)
        {
            &[("↑ ↓", "Choose"), ("Ctrl+S", "Submit")]
        } else if ui.state(CONFIRM).contains(StateFlags::FOCUSED)
            || ui.state(RUN_TESTS).contains(StateFlags::FOCUSED)
            || ui.state(OPEN_PR).contains(StateFlags::FOCUSED)
            || ui.state(AUTO_APPROVE).contains(StateFlags::FOCUSED)
            || ui.state(NOTIFY).contains(StateFlags::FOCUSED)
        {
            &[("Space", "Toggle"), ("Ctrl+S", "Submit")]
        } else if ui.state(SAVE).contains(StateFlags::FOCUSED)
            || ui.state(RESET).contains(StateFlags::FOCUSED)
        {
            &[("Enter", "Activate"), ("Ctrl+S", "Submit")]
        } else {
            &[("Enter", "Edit"), ("Ctrl+S", "Submit")]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.summary_state.is_editing()
            || self.details_state.is_editing()
            || self.reviewer_state.is_editing()
    }
}

impl FormsPage {
    fn draw_task_card(&self, ui: &mut Ui<'_>, card_body: Rect, panel_width: u16) {
        let panel_area = Rect {
            width: if card_body.width < 70 {
                panel_width.saturating_sub(1)
            } else {
                card_body.width
            },
            height: card_body.height.min(24),
            ..card_body
        };
        let panel = Panel::new(termrock::Id::root(
            "showcase_ui::pages::forms::forms.new_task",
        ))
        .kind(PanelKind::Card)
        .title("New task")
        .meta(if card_body.width < 70 {
            "Ctrl+S Submit"
        } else {
            "Ctrl+S Submit "
        });
        let panel_inner = panel.inner(ui, panel_area);
        panel.draw(ui, panel_area, |ui, inner| {
            let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
            let left = columns.first().copied().unwrap_or(inner);
            let right = columns.get(1).copied().unwrap_or(inner);
            self.draw_task_fields(ui, left);
            let _ = ui.paint_str(Rect { height: 1, ..right }, "Options", ui.surface_style());
            let _ = ui.paint_str(
                Rect {
                    x: right.x.saturating_add(2),
                    width: right.width.saturating_sub(2),
                    y: right.y.saturating_add(1),
                    height: 1,
                },
                "Mode",
                ui.surface_style(),
            );
            let mode_area = Rect {
                y: right.y.saturating_add(2),
                height: 3,
                ..right
            };
            mode_group().value(ItemKey::index(self.mode)).draw(
                ui,
                mode_area,
                &self.mode_state,
                MODES,
            );
            for index in 0..MODES.len() {
                let selected = self.mode == index;
                let row = Rect {
                    y: mode_area.y.saturating_add(index as u16),
                    height: 1,
                    ..mode_area
                };
                let flags = ui.state(MODE)
                    | if selected {
                        StateFlags::SELECTED
                    } else {
                        StateFlags::default()
                    };
                legacy_gutter(
                    ui,
                    row,
                    Family::CHOICE,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    flags,
                );
                legacy_marker(ui, row, flags, if selected { "(●)" } else { "( )" });
            }
            self.draw_test_options(ui, right);
            self.draw_notification_options(ui, right);

            self.draw_actions(ui, inner);
        });
        if card_body.width < 70
            && !panel_inner.is_empty()
            && let Some(right) = layout::columns(panel_inner, &[Track::Flex(1), Track::Flex(1)], 3)
                .get(1)
                .copied()
        {
            let _ = ui.paint_str(
                Rect {
                    y: right.y.saturating_add(11),
                    width: card_body.right().saturating_sub(right.x),
                    height: 1,
                    ..right
                },
                "  Managed by your organization",
                ui.surface_style(),
            );
        }
    }

    fn draw_deploy_form(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let fields = deploy_fields();
        let actions = deploy_actions();
        deploy_panel().draw(ui, area, |ui, inner| {
            deploy_form(&fields, &actions).draw(ui, inner, &self.deploy_state, &self.deploy);
        });
    }

    fn draw_task_fields(&self, ui: &mut Ui<'_>, left: Rect) {
        let _ = Self::priority();
        let _ = ui.paint_str(Rect { height: 1, ..left }, "Task", ui.surface_style());
        let summary_error =
            (self.error == Some("Required: summary")).then_some("Required: summary");
        let task = Rect {
            y: left.y.saturating_add(1),
            height: if summary_error.is_some() { 3 } else { 2 },
            ..left
        };
        task_name_field(&self.summary, summary_error).draw(ui, task, &self.summary_state);
        if self.summary.is_empty() {
            legacy_input_placeholder(ui, task, "Short imperative summary");
        }
        legacy_gutter(
            ui,
            Rect {
                y: task.y.saturating_add(1),
                height: 1,
                ..task
            },
            Family::FIELD,
            Variant::DEFAULT,
            Part::FIELD,
            StateFlags::empty(),
        );
        let description = Rect {
            y: left.y.saturating_add(4),
            height: 6,
            ..left
        };
        description_field().draw(ui, description, &self.details_state);
        legacy_placeholder(
            ui,
            description,
            "What should Junie do, and what does done look like?",
        );
        legacy_gutter(
            ui,
            Rect {
                y: description.y.saturating_add(1),
                height: 4,
                ..description
            },
            Family::FIELD,
            Variant::DEFAULT,
            Part::FIELD,
            StateFlags::empty(),
        );
        let review_y = left.y.saturating_add(11);
        let _ = ui.paint_str(
            Rect {
                y: review_y,
                height: 1,
                ..left
            },
            "Review",
            ui.surface_style(),
        );
        let reviewer = Rect {
            y: review_y.saturating_add(1),
            height: 3,
            ..left
        };
        let reviewer_error = self
            .reviewer_state
            .error()
            .map(|error| error.message.as_ref());
        reviewer_field(&self.reviewer, reviewer_error).draw(ui, reviewer, &self.reviewer_state);
        let reviewer_flags = ui.state(REVIEWER);
        legacy_gutter(
            ui,
            Rect {
                y: reviewer.y.saturating_add(1),
                height: 1,
                ..reviewer
            },
            Family::FIELD,
            Variant::DEFAULT,
            Part::FIELD,
            reviewer_flags,
        );
    }
}

impl FormsPage {
    fn draw_test_options(&self, ui: &mut Ui<'_>, right: Rect) {
        let run_tests = Rect {
            y: right.y.saturating_add(6),
            height: 1,
            ..right
        };
        run_tests_checkbox(self.run_tests).draw(ui, run_tests);
        let run_tests_flags = ui.state(RUN_TESTS)
            | if self.run_tests {
                StateFlags::CHECKED | StateFlags::SELECTED
            } else {
                StateFlags::empty()
            };
        legacy_choice_label(
            ui,
            run_tests,
            run_tests_flags,
            "Run tests before opening a PR",
        );
        legacy_gutter(
            ui,
            run_tests,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::CONTAINER,
            run_tests_flags,
        );
        let open_pr = Rect {
            y: right.y.saturating_add(7),
            height: 1,
            ..right
        };
        open_pr_checkbox(self.open_pr).draw(ui, open_pr);
        let open_pr_flags = ui.state(OPEN_PR)
            | if self.open_pr {
                StateFlags::CHECKED | StateFlags::SELECTED
            } else {
                StateFlags::empty()
            };
        legacy_choice_label(ui, open_pr, open_pr_flags, "Open a pull request when done");
        legacy_gutter(
            ui,
            open_pr,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::CONTAINER,
            open_pr_flags,
        );
    }
}

impl FormsPage {
    fn draw_notification_options(&self, ui: &mut Ui<'_>, right: Rect) {
        Self::confirmation().checked(self.confirm).draw(
            ui,
            Rect {
                y: right.y.saturating_add(13),
                height: 1,
                ..right
            },
        );
        let auto_approve = Rect {
            y: right.y.saturating_add(9),
            height: 1,
            ..right
        };
        auto_approve_toggle(self.auto_approve).draw(ui, auto_approve);
        let auto_approve_flags = ui.state(AUTO_APPROVE)
            | if self.auto_approve {
                StateFlags::CHECKED | StateFlags::SELECTED
            } else {
                StateFlags::empty()
            };
        legacy_gutter(
            ui,
            auto_approve,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::CONTAINER,
            auto_approve_flags,
        );
        legacy_marker(
            ui,
            auto_approve,
            auto_approve_flags,
            if self.auto_approve {
                "●──"
            } else {
                "○──"
            },
        );
        let notify = Rect {
            y: right.y.saturating_add(10),
            height: 1,
            ..right
        };
        notify_toggle(self.notify).draw(ui, notify);
        let notify_flags = ui.state(NOTIFY)
            | StateFlags::DISABLED
            | if self.notify {
                StateFlags::CHECKED | StateFlags::SELECTED
            } else {
                StateFlags::empty()
            };
        legacy_gutter(
            ui,
            notify,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::CONTAINER,
            notify_flags,
        );
        legacy_marker(
            ui,
            notify,
            notify_flags,
            if self.notify {
                "──●"
            } else {
                "──○"
            },
        );
        let _ = ui.paint_str(
            Rect {
                y: right.y.saturating_add(11),
                height: 1,
                width: right.width.saturating_add(1),
                ..right
            },
            "  Managed by your organization",
            ui.surface_style(),
        );
    }
}

impl FormsPage {
    fn draw_actions(&self, ui: &mut Ui<'_>, inner: Rect) {
        let action_area = Rect {
            y: inner.bottom().saturating_sub(1),
            height: 1,
            ..inner
        };
        let busy = matches!(self.submission, Submission::Busy { .. });
        let create = Self::save_button(self.confirm, busy);
        let reset = reset_button();
        let widths = [
            create
                .measure(ui, Constraints::loose(action_area.width, 1))
                .preferred
                .0,
            reset
                .measure(ui, Constraints::loose(action_area.width, 1))
                .preferred
                .0,
        ];
        let rects = layout::action_row(action_area, &widths, 2, RowAlign::Start);
        let create_area = rects.first().copied().unwrap_or(action_area);
        create.draw(ui, create_area);
        let create_flags = ui.state(SAVE);
        legacy_gutter(
            ui,
            create_area,
            Family::BUTTON,
            Variant::PRIMARY,
            Part::CONTAINER,
            create_flags,
        );
        let reset_area = rects.get(1).copied().unwrap_or(action_area);
        reset.draw(ui, reset_area);
        let reset_flags = ui.state(RESET);
        legacy_gutter(
            ui,
            reset_area,
            Family::BUTTON,
            Variant::SUBTLE,
            Part::CONTAINER,
            reset_flags,
        );
        let status = self
            .error
            .map(|_| "Fix the highlighted fields")
            .or(match self.submission {
                Submission::Idle => None,
                Submission::Busy { .. } => Some("Creating task…"),
                Submission::Done => Some("Task created ✓"),
            });
        if let Some(status) = status {
            let x = reset_area.right().saturating_add(3);
            let width = action_area.right().saturating_sub(x);
            if width > 0 {
                let _ = ui.paint_str(
                    Rect {
                        x,
                        y: action_area.y,
                        width,
                        height: 1,
                    },
                    status,
                    ui.surface_style(),
                );
            }
        }
    }
}
