use termrock::{App, ColorLevel, Cx, Response, Theme, Ui};
use termrock_test_support::Scene;

struct Model(u8);

impl App for Model {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, _: &mut Ui<'_>) {}
}

fn main() {
    let mut model = Model(0);
    let mut scene = Scene::new("bound", Theme::junie(), ColorLevel::TrueColor, 8, 2);
    let mut bound = scene.bind_app(&model);
    model.0 = 1;
    assert_eq!(model.0, 1);
    bound.draw();
}
