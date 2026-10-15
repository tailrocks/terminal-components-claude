//! Scratch probe of narrow toggle/checkbox paints; deleted after use.
use termrock::Buffer;
use termrock::{Checkbox, ColorLevel, Id, Rect, Runtime, Theme, Toggle, Ui};
use termrock::runtime::stub::Stub;

const CB: Id = Id::root("probe.cb");
const TG: Id = Id::root("probe.tg");

fn paint(id: Id, on: bool, width: u16, toggle: bool) -> String {
    let area = Rect::new(0, 0, width, 1);
    let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
    let mut buffer = Buffer::empty(area);
    runtime.draw_scene(area, &mut buffer, |ui: &mut Ui<'_>, area| {
        if toggle {
            Toggle::new(id, "Choice").on(on).draw(ui, area);
        } else {
            Checkbox::new(id, "Choice").checked(on).draw(ui, area);
        }
    }).commit_presented();
    let mut row = String::new();
    for x in 0..width {
        if let Some(c) = buffer.cell(termrock::Position::new(x, 0)) {
            row.push_str(c.symbol());
        }
    }
    row
}

#[test]
fn probe() {
    for w in [1u16, 2, 3, 4, 5] {
        for on in [false, true] {
            println!("w={w} on={on} cb={:?} tg={:?}", paint(CB, on, w, false), paint(TG, on, w, true));
        }
    }
}
