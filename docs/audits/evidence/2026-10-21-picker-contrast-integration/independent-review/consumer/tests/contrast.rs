use ratatui_core::style::Color;
use termrock_navigation::{FilterList,FilterListState,Item};
use termrock_runtime::{App,Cx,Ui,Response,Id,ItemKey,Part,UpdateCause,KeyCode};
use termrock_test_support::Harness;
use termrock_theme::{Theme,ColorLevel,CursorStyle,Role,StylePatch,Modifier,Family,contrast_ratio};
use std::io::Write;
const ID:Id=Id::root("independent.contrast-picker");
struct Page {state:FilterListState,cursor:CursorStyle}
impl Page {fn list(&self)->FilterList<'static,Item<'static>> {FilterList::new(ID).searchable(false).cursor_style(self.cursor)}}
const ITEMS:&[Item<'static>]=&[Item::new(ItemKey::num(17),"ROW").glyph("I").detail("D").tag("T"),Item::new(ItemKey::num(98),"OTHER").glyph("J").detail("E").tag("U")];
impl App for Page {
 fn update(&mut self,cx:&mut Cx<'_>)->Response<()> {if cx.update_cause()==UpdateCause::Bootstrap {cx.focus(ID);}self.list().update(cx,&mut self.state,ITEMS).erase()}
 fn draw(&self,ui:&mut Ui<'_>) {let area=ui.full();self.list().draw_results(ui,area,&self.state,ITEMS);}
}
fn view(level:ColorLevel,cursor:CursorStyle,bg:Option<Color>,fg:Option<Color>,modifiers:bool)->Harness<Page> {
 let mut theme=Theme::junie();
 if bg.is_some()||fg.is_some()||modifiers {theme=theme.override_family(Family::PICKER,|recipe|{for part in [Part::META,Part::ICON]{let mut p=StylePatch::new();if let Some(bg)=bg{p=p.set_bg(Role::Custom(bg));}if let Some(fg)=fg{p=p.set_fg(Role::Custom(fg));}if modifiers{p=p.add(Modifier::ITALIC).remove(Modifier::BOLD);}recipe.part(part).base(p);}});}
 {let h=Harness::new(Page{state:FilterListState::default(),cursor},theme,60,12).with_color(level);assert_eq!(h.focus(),Some(ID));h}
}
fn cell(h:&Harness<Page>,symbol:&str)->ratatui_core::buffer::Cell {h.buffer().content.iter().find(|c|c.symbol()==symbol).unwrap_or_else(||panic!("missing {symbol}")).clone()}
#[test]
fn recreated_white_background_and_authored_channels_contrast() {
 let mut rows=Vec::new();let mut failures=Vec::new();
 for level in [ColorLevel::TrueColor,ColorLevel::Ansi256,ColorLevel::Ansi16,ColorLevel::Mono] {
  let same=match level {ColorLevel::TrueColor=>Color::Rgb(138,138,138),ColorLevel::Ansi256=>Color::Indexed(244),ColorLevel::Ansi16|ColorLevel::Mono=>Color::Gray,_=>unreachable!("only known profiles selected")};
  for cursor in [CursorStyle::Bar,CursorStyle::Fill] {for bg in [None,Some(Color::White),Some(Color::Black)] {for fg in [None,Some(Color::Black),Some(Color::White),Some(same)] {
   let h=view(level,cursor,bg,fg,true);
   for symbol in ["I","D","T"] {let c=cell(&h,symbol);let ratio=contrast_ratio(c.fg,c.bg).unwrap_or(0.0);rows.push(format!("{level:?}/{cursor:?}/bg={bg:?}/fg={fg:?}/{symbol}: {c:?} ratio={ratio:.8}"));
    if let Some(expected)=bg {assert_eq!(c.bg,expected,"authored BG independent");}
    if let Some(expected)=fg {let direct_theme=Theme::junie().downgrade(level);let direct=termrock_theme::resolve::bind(&direct_theme,StylePatch::new().set_fg(Role::Custom(expected)),None,termrock_theme::Surface::Canvas).style.fg.expect("authored foreground binds");assert_eq!(c.fg,direct,"explicit foreground preserves canonical capability binding even same-value/low-contrast choices");}else if ratio<4.5{failures.push(format!("{level:?}/{cursor:?}/{bg:?}/{symbol}: fg={:?} bg={:?} ratio={ratio:.8}",c.fg,c.bg));}
    assert!(c.modifier.contains(Modifier::ITALIC),"authored italic lost");assert!(!c.modifier.contains(Modifier::BOLD),"authored bold removal lost");
   }
   assert!(h.diagnostics().is_empty(),"{:?}",h.diagnostics());
  }}}
 }
 std::fs::write(std::env::var("PICKER_CONTRAST_CELLS").unwrap(),rows.join("\n")).unwrap();
 assert!(failures.is_empty(),"Default-ink contrast failures:\n{}",failures.join("\n"));
}
fn dump(w:&mut impl Write,h:&Harness<Page>,label:&str) {writeln!(w,"{label} cursor={:?} focus={:?} ring={:?} state={:?}",h.cursor(),h.focus(),h.ring(),h.app().state).unwrap();let cells=&h.buffer().content;let mut i=0;while i<cells.len(){let mut j=i+1;while j<cells.len()&&cells[j]==cells[i]{j+=1;}writeln!(w,"{} {:?}",j-i,cells[i]).unwrap();i=j;}}
#[test]
fn default_frames_remain_unmodified() {
 let mut w=std::io::BufWriter::new(std::fs::File::create(std::env::var("PICKER_CONTRAST_DEFAULT_DUMP").unwrap()).unwrap());
 for level in [ColorLevel::TrueColor,ColorLevel::Ansi256,ColorLevel::Ansi16,ColorLevel::Mono] {for cursor in [CursorStyle::Bar,CursorStyle::Fill] {writeln!(w,"CASE {level:?}/{cursor:?}").unwrap();let mut h=view(level,cursor,None,None,false);dump(&mut w,&h,"initial");let _=h.key(KeyCode::Down);dump(&mut w,&h,"down");let _=h.resize(29,5);dump(&mut w,&h,"resized");}}
}
