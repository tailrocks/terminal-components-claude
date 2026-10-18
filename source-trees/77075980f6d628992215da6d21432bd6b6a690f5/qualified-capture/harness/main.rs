use std::{fs, path::PathBuf, time::{Duration, Instant}};
use tuisnap::{pty::{Session, PtyOptions}, frame::{Frame, Rgb}};
fn save(p: &PathBuf, name: &str, frame: &Frame) { fs::write(p.join(name), frame.to_json_pretty()).unwrap(); }
fn main() {
 let p = PathBuf::from(std::env::args().nth(1).unwrap());
 let opts = PtyOptions { cols:32, rows:12, timeout:Duration::from_secs(4), ..Default::default() };
 let mut s = Session::spawn(&["python3".into(), p.join("fixture.py").display().to_string(), p.join("snap-events.json").display().to_string()], &opts).unwrap();
 s.wait_for_text("READY").unwrap();
 let f = s.wait_stable(Duration::from_millis(250)).unwrap(); save(&p,"snap.frame.json",&f);
 let (fg,bg)=Frame::resolve_cell(f.get(0,8).unwrap(),Rgb::new(255,255,255),Rgb::new(0,0,0));
 fs::write(p.join("snap-dim-resolution.txt"),format!("actual={fg:?}/{bg:?}; expected fg=(153,153,153)\n")).unwrap();
 s.click(4,3).unwrap(); s.paste_literal("PASTE\n二").unwrap(); s.drag(2,2,5,4).unwrap();
 s.resize(36,14).unwrap(); s.send_key("s").unwrap();
 let t=Instant::now(); s.wait_for_text("T").unwrap(); let settled=s.wait_stable(Duration::from_millis(300)).unwrap();
 save(&p,"snap-settled.frame.json",&settled); fs::write(p.join("snap-settle-ms.txt"),t.elapsed().as_millis().to_string()).unwrap();
 s.send_key("q").unwrap(); fs::write(p.join("snap-exit.txt"),format!("{:?}",s.wait_exit().unwrap())).unwrap();
}
