fn main() {
    let mut app = tablepro_ui::TableProApp::default();
    let _ = app.workbench.new_query("");
    let _: &[tablepro_ui::TabRecord] = app.workbench.tabs();
    app.workbench.tabs.clear();
}
