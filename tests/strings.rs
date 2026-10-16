//! `theme::Strings` reaches the library's own copy, and per-widget
//! builders win over it (api #27, #28).

use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::{Strings, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::choice_dialog::ChoiceDialog;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::diff::{DiffFile, DiffHunk, DiffLine, DiffMode, DiffStatus, DiffView};
use junie_tui::widgets::form_dialog::FormDialog;
use junie_tui::widgets::grid::{CellKind, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::help::HelpOverlay;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::list::{ListBox, SelectMode};
use junie_tui::widgets::picker::Picker;
use junie_tui::widgets::table::{Column, DataTable};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

const DE: Strings = Strings {
    cancel: "Abbrechen",
    save: "Sichern",
    next: "Weiter",
    type_to_confirm: "Tippe {} zum Bestätigen",
    required: "Pflichtfeld",
    optional: "freiwillig",
    no_matches: "Keine Treffer",
    search_placeholder: "Suchen…",
    empty_list: "Noch leer",
    no_rows: "Keine Zeilen",
    loading_rows: "Lade Zeilen…",
    fetch_more: "{} geladen, Enter holt mehr",
    diff_old: "Alt",
    diff_new: "Neu",
    diff_no_file: "Keine Datei",
    reveal_open: "Ganzer Text",
    hint_choose: "Wählen",
    help_title: "Tastenkürzel",
    help_footer: "Esc Schließen",
    ..Strings::DEFAULT
};

fn de() -> Theme {
    Theme::junie().with_strings(DE)
}

fn draw(theme: &Theme, size: (u16, u16), f: impl FnOnce(&mut Buffer, &mut RenderCtx)) -> String {
    let mut buf = Buffer::empty(Rect::new(0, 0, size.0, size.1));
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(theme, Interaction::default(), &mut hits, &mut ring);
    f(&mut buf, &mut ctx);
    ctx.flush_overlays(&mut buf);
    (0..size.1)
        .map(|y| {
            (0..size.0)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn defaults_are_todays_copy() {
    assert_eq!(Theme::junie().strings, Strings::DEFAULT);
    assert_eq!(
        Strings::fill(Strings::DEFAULT.type_to_confirm, &[&"DROP"]),
        "Type DROP to confirm"
    );
    assert_eq!(Strings::fill("{} of {}", &[&1]), "1 of {}");
    for s in [
        Strings::ASCII.more_cut,
        Strings::ASCII.search_placeholder,
        Strings::ASCII.loading_rows,
        Strings::ASCII.fetch_more,
        Strings::ASCII.help_footer,
    ] {
        assert!(s.is_ascii(), "{s}");
    }
}

#[test]
fn dialogs_follow_the_theme_and_keep_a_set_label() {
    let id = WidgetId::of("d");
    let mut d = Dialog::confirm(id, "Delete", "Sure?", "Delete");
    let text = draw(&de(), (60, 20), |buf, ctx| {
        d.render(Rect::new(0, 0, 60, 20), buf, ctx)
    });
    assert!(text.contains("Abbrechen"), "{text}");
    assert!(!text.contains("Cancel"), "{text}");
    let hints: Vec<String> = d.hints(None).iter().map(|h| h.action.to_string()).collect();
    assert!(hints.iter().any(|a| a == "Wählen"), "{hints:?}");
    // back to the default theme: the label follows back
    let text = draw(&Theme::junie(), (60, 20), |buf, ctx| {
        d.render(Rect::new(0, 0, 60, 20), buf, ctx)
    });
    assert!(text.contains("Cancel"), "{text}");
    // a label the caller set stays
    let mut d = Dialog::confirm(id, "Delete", "Sure?", "Delete").cancel_label("Keep");
    let text = draw(&de(), (60, 20), |buf, ctx| {
        d.render(Rect::new(0, 0, 60, 20), buf, ctx)
    });
    assert!(
        text.contains("Keep") && !text.contains("Abbrechen"),
        "{text}"
    );
    // the typed acknowledgement's instruction
    let confirm = junie_tui::widgets::button::Button::danger(id.sub("go"), "Drop");
    let mut d = Dialog::facts(id, "Drop", vec![], vec![], Some("DROP"), confirm);
    let text = draw(&de(), (60, 24), |buf, ctx| {
        d.render(Rect::new(0, 0, 60, 24), buf, ctx)
    });
    assert!(text.contains("Tippe DROP zum Bestätigen"), "{text}");

    let mut f = FormDialog::new(WidgetId::of("f"), "Form", vec![]);
    let text = draw(&de(), (80, 20), |buf, ctx| {
        f.render(Rect::new(0, 0, 80, 20), buf, ctx)
    });
    assert!(
        text.contains("Abbrechen") && text.contains("Sichern"),
        "{text}"
    );

    let mut c = ChoiceDialog::new(WidgetId::of("c"), "Pick", "Kind", &["a", "b"], 0);
    let text = draw(&de(), (80, 20), |buf, ctx| {
        c.render(Rect::new(0, 0, 80, 20), buf, ctx)
    });
    assert!(
        text.contains("Abbrechen") && text.contains("Weiter"),
        "{text}"
    );
}

#[test]
fn empty_lists_tables_and_pickers_follow_the_theme() {
    let mut l = ListBox::new(WidgetId::of("l"), vec![], SelectMode::Single);
    let text = draw(&de(), (30, 3), |buf, ctx| {
        l.render(
            Rect::new(0, 0, 30, 3),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Noch leer"), "{text}");
    let mut l = ListBox::new(WidgetId::of("l"), vec![], SelectMode::Single).empty_text("Nada");
    let text = draw(&de(), (30, 3), |buf, ctx| {
        l.render(
            Rect::new(0, 0, 30, 3),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Nada"), "{text}");

    let mut tb = DataTable::new(
        WidgetId::of("t"),
        vec![Column::new("a", Constraint::Length(5))],
        vec![],
    );
    let text = draw(&de(), (30, 4), |buf, ctx| {
        tb.render(
            Rect::new(0, 0, 30, 4),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Keine Zeilen"), "{text}");

    let mut p = Picker::new(WidgetId::of("p"), "Open");
    let text = draw(&de(), (80, 24), |buf, ctx| {
        p.draw(Rect::new(0, 0, 80, 24), buf, ctx)
    });
    assert!(
        text.contains("Suchen…") && text.contains("Keine Treffer"),
        "{text}"
    );
    let mut p = Picker::new(WidgetId::of("p"), "Open").placeholder("Find a file");
    let text = draw(&de(), (80, 24), |buf, ctx| {
        p.draw(Rect::new(0, 0, 80, 24), buf, ctx)
    });
    assert!(text.contains("Find a file"), "{text}");
}

#[test]
fn grids_follow_the_theme() {
    let cols = vec![ColumnSpec::new("a", CellKind::Text)];
    let mut g = DataGrid::new(WidgetId::of("g"), cols.clone());
    let text = draw(&de(), (40, 8), |buf, ctx| {
        g.render(
            Rect::new(0, 0, 40, 8),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Keine Zeilen"), "{text}");
    g.set_loading(true);
    let text = draw(&de(), (40, 8), |buf, ctx| {
        g.render(
            Rect::new(0, 0, 40, 8),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Lade Zeilen"), "{text}");
    let mut g = DataGrid::new(WidgetId::of("g"), cols);
    g.set_rows(GridRows {
        rows: vec![vec![junie_tui::widgets::grid::CellValue::Int(1)]],
        total: RowTotal::Unknown,
        more: true,
    });
    let text = draw(&de(), (50, 8), |buf, ctx| {
        g.render(
            Rect::new(0, 0, 50, 8),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("1 geladen, Enter holt mehr"), "{text}");
}

#[test]
fn required_fields_and_their_suffix_follow_the_theme() {
    let mut i = TextInput::new(WidgetId::of("i"), "Name").required(true);
    draw(&de(), (40, 3), |buf, ctx| {
        i.render(
            Rect::new(0, 0, 40, 3),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    i.validate();
    assert_eq!(i.error.as_deref(), Some("Pflichtfeld"));
    // an error already shown follows a theme change
    draw(&Theme::junie(), (40, 3), |buf, ctx| {
        i.render(
            Rect::new(0, 0, 40, 3),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert_eq!(i.error.as_deref(), Some("Required"));
    let mut i = TextInput::new(WidgetId::of("i"), "Name")
        .required(true)
        .required_message("Enter a name");
    draw(&de(), (40, 3), |buf, ctx| {
        i.render(
            Rect::new(0, 0, 40, 3),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    i.validate();
    assert_eq!(i.error.as_deref(), Some("Enter a name"));
    let mut o = TextInput::new(WidgetId::of("o"), "Note");
    let text = draw(&de(), (40, 3), |buf, ctx| {
        o.render(
            Rect::new(0, 0, 40, 3),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("freiwillig"), "{text}");
}

#[test]
fn diffs_help_and_reveal_follow_the_theme() {
    let mut v = DiffView::new(WidgetId::of("v"));
    let text = draw(&de(), (80, 6), |buf, ctx| {
        v.render(
            Rect::new(0, 0, 80, 6),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Keine Datei"), "{text}");
    v.set_file(Some(DiffFile {
        path: "a.rs".into(),
        status: DiffStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![DiffLine::remove("x"), DiffLine::add("y")],
        }],
    }));
    v.mode = DiffMode::Review;
    let text = draw(&de(), (100, 8), |buf, ctx| {
        v.render(
            Rect::new(0, 0, 100, 8),
            buf,
            ctx,
            ratatui::style::Color::Reset,
        )
    });
    assert!(text.contains("Alt") && text.contains("Neu"), "{text}");

    let mut h = HelpOverlay::new(WidgetId::of("h"), "App", vec![("Move", vec![("k", "Up")])]);
    let text = draw(&de(), (60, 20), |buf, ctx| {
        h.render(Rect::new(0, 0, 60, 20), buf, ctx)
    });
    assert!(
        text.contains("Tastenkürzel") && text.contains("Esc Schließen"),
        "{text}"
    );
}
