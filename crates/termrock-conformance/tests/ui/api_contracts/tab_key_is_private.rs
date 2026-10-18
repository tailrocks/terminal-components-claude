use tablepro_ui::TableProApp;

fn main() {
    let app = TableProApp::default();
    let tabs = app.workbench.tabs();
    let _ = tabs[0].key;
}
