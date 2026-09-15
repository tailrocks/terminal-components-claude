use std::{fs, time::Duration};
use serde_json::{json, Value};
use tuisnap::{Color, pty::{PtyOptions, Session}};
fn color(c: Color) -> Value {
    match c {
        Color::Default => json!("default"),
        Color::Indexed(i) => json!(i),
        Color::Rgb(c) => json!(format!("#{:02x}{:02x}{:02x}", c.r,c.g,c.b)),
    }
}
fn main() {
    let package = std::env::args().nth(1).expect("qualified-capture directory");
    let output = std::env::args().nth(2).expect("existing output directory");
    let opts = PtyOptions { cols:32, rows:12, timeout:Duration::from_secs(4), ..Default::default() };
    let mut s = Session::spawn(&["python3".into(), format!("{package}/fixture.py"), format!("{output}/events.json")], &opts).unwrap();
    s.wait_for_text("READY").unwrap();
    let frame = s.wait_stable(Duration::from_millis(250)).unwrap();
    frame.validate().unwrap();
    fs::write(format!("{output}/actual.frame.json"), frame.to_json()).unwrap();
    let expected:Value = serde_json::from_str(&fs::read_to_string(format!("{package}/expected-cells.json")).unwrap()).unwrap();
    let cells:Vec<Value> = frame.cells.iter().map(|c| json!({"x":c.x,"y":c.y,"symbol":c.symbol,"width":c.width,"continuation":c.continuation,"fg":color(c.fg),"bg":color(c.bg),"bold":c.mods.bold,"dim":c.mods.dim,"reverse":c.mods.reverse,"underline":c.mods.underline,"italic":c.mods.italic,"strikethrough":c.mods.strikethrough,"hidden":c.mods.hidden,"blink":c.mods.blink})).collect();
    assert_eq!(json!(cells), expected, "all 384 independent oracle cells");
    assert_eq!(serde_json::to_value(frame.cursor).unwrap(), json!({"x":4,"y":9,"visible":true,"style":"Bar","blinking":false}));
    s.click(4,3).unwrap();
    s.paste_literal("PASTE\n二").unwrap();
    s.drag(2,2,5,4).unwrap();
    s.type_text("\u{1b}[<35;5;4M\u{1b}[<64;5;4M\u{1b}[<65;5;4M").unwrap();
    s.resize(36,14).unwrap();
    s.send_key("s").unwrap();
    s.wait_for_text("T").unwrap();
    let settled=s.wait_stable(Duration::from_millis(300)).unwrap();
    assert_eq!((settled.cols,settled.rows),(36,14));
    assert_eq!(settled.get(0,9).unwrap().symbol,"T");
    assert_eq!(settled.get(0,9).unwrap().fg,Color::Indexed(4));
    s.send_key("q").unwrap();
    assert!(s.wait_exit().unwrap().success());
    let events:Value=serde_json::from_str(&fs::read_to_string(format!("{output}/events.json")).unwrap()).unwrap();
    let inputs:String=events.as_array().unwrap().iter().filter(|e|e["kind"]=="input").map(|e| e["hex"].as_str().unwrap()).collect();
    for raw in ["\u{1b}[<0;5;4M\u{1b}[<0;5;4m", "\u{1b}[200~PASTE\n二\u{1b}[201~", "\u{1b}[<0;3;3M", "\u{1b}[<0;6;5m", "\u{1b}[<35;5;4M", "\u{1b}[<64;5;4M", "\u{1b}[<65;5;4M"] {
        let hex:String=raw.as_bytes().iter().map(|b|format!("{b:02x}")).collect();
        assert!(inputs.contains(&hex), "input transport {raw:?}");
    }
    assert!(events.as_array().unwrap().iter().any(|e|e["kind"]=="resize" && e["cols"]==36 && e["rows"]==14));
    println!("PASS: 384 independent cells; all modifiers, wide styles, clipping, cursor; click/drag/hover/wheel transport; literal LF paste; resize; style-aware settle; child exit");
}

