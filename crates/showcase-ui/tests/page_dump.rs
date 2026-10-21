use showcase_ui::{App, PageId};
use termrock::Theme;
use termrock_test_support::Harness;

#[test]
fn dump_page_text() {
    let page = std::env::var("DUMP_PAGE").unwrap_or_else(|_| "Pickers".to_owned());
    let page = match page.as_str() {
        "Pickers" => PageId::Pickers,
        "Grid" => PageId::Grid,
        "Lists" => PageId::Lists,
        "Trees" => PageId::Trees,
        "Tables" => PageId::Tables,
        "Dialogs" => PageId::Dialogs,
        "Selectable" => PageId::Overview,
        _ => PageId::Pickers,
    };
    let h = Harness::new(App::with_page(page), Theme::junie(), 120, 40);
    for y in 0..40 {
        let row: String = (0..120).map(|x| h.cell(x, y).symbol().to_owned()).collect();
        let styles: String = (0..120)
            .map(|x| {
                let c = h.cell(x, y);
                format!("{:?}|{:?}|{}", c.fg, c.bg, c.modifier.bits())
            })
            .collect::<Vec<_>>()
            .join(" ");
        println!("ROW {y:02} |{row}|");
        println!("STY {y:02} |{styles}|");
    }
}
