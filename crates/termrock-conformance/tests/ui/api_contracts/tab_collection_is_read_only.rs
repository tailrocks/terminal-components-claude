use tablepro_ui::{Tab, TableProApp};

fn main() {
    let mut app = TableProApp::default();
    app.workbench
        .tabs()
        .push(Tab::Query(tablepro_ui::QueryTab::new(1, "")));
}
