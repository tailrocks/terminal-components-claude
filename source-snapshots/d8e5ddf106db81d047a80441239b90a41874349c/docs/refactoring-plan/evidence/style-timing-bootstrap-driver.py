#!/usr/bin/env python3
"""Source-copy-only actual production style/frame qualification; private judge."""
from __future__ import annotations

import argparse
import ast
import base64
import importlib.util
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import tempfile
import select
import time
import sys
import inspect

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("style_actual_transport", HERE / "architecture-bootstrap-actual-driver.py")
transport = importlib.util.module_from_spec(spec)
spec.loader.exec_module(transport)
actual, runner = transport.actual, transport.runner
require, sha, canonical = runner.require, runner.sha, runner.canonical
UI = "crates/tui/src/ui/mod.rs"
PROBE = "crates/tui/src/style_timing_bootstrap.rs"
TEST = "apps/showcase/tests/style_timing_bootstrap.rs"
UI_CENSUS = ["style", "style_inherited", "style_defaults", "paint_patch", "style_patched", "resolve", "surface_style", "bg"]
CENSUS = UI_CENSUS+["CellUi::drop::bind","StatusBar::item_style::bind","Grid::apply_style_delta::bind"]
POLICY = {"schema": "tc-style-timing-policy/v1", "profile": "release", "warmup": 12, "batches": 9,
          "order": ["disabled", "calibration", "measured"], "frame": "Showcase::Lists@120x40/Junie",
          "denominator": "one uninstrumented production Harness.draw, no unrelated work",
          "aggregation": "per-frame paired raw-minus-calibration; max corrected/min denominator conservative ratio",
          "threshold": 0.05, "uncertainty": "entire observed batch range; no negative clamp",
          "clock_domains":{"attestation":"std::time::Instant","control":"explicit injected arithmetic only"}}


def replace(text, before, after):
    return actual.replace_once(text, before, after)


def function_body(text, name):
    match = re.search(r"\bfn " + re.escape(name) + r"(?:<[^\n]*>)?\(", text)
    require(match is not None, "missing pinned function " + name)
    start = text.index("{", match.end())
    depth, end = 1, start + 1
    while depth:
        require(end < len(text), "unterminated pinned function")
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return start, end


def extraction_source(original):
    source = dict(original)
    path="crates/tui/src/runtime.rs"
    source[path]=replace(source[path].decode(),"    pub fn style_cache_stats(&self) -> (u64, u64) {",
        '    pub fn style_timing_bootstrap_cache(&self)->String { format!("{:?}",self.core.style_cache) }\n    pub fn style_cache_stats(&self) -> (u64, u64) {').encode()
    path = "apps/showcase/src/pages/mod.rs"
    source[path] = replace(source[path].decode(), "pub(crate) trait Page: Send {",
        'pub(crate) trait Page: Send {\n    fn style_timing_bootstrap_state(&self) -> String { String::new() }').encode()
    path = "apps/showcase/src/pages/lists.rs"
    source[path] = replace(source[path].decode(), "impl Page for ListsPage {",
        'impl Page for ListsPage {\n    fn style_timing_bootstrap_state(&self) -> String { format!("{self:?}") }').encode()
    path = "apps/showcase/src/app.rs"
    source[path] = replace(source[path].decode(), "impl App {", 'impl App {\n    pub fn style_timing_bootstrap_state(&self) -> String { format!("{:?}|{}|{:?}", self, self.pages[self.page.index()].style_timing_bootstrap_state(), self.status.as_ref().map(|(s,t)| (&s.0,t))) }').encode()
    return source


def source_fixture(original, case="valid", baseline=False):
    original_case=case
    attestation=case.startswith("attest-")
    if attestation: case=case.removeprefix("attest-")
    if case=="strict-over-budget": case="expensive-resolution"
    if case=="strict-injected-forgery": case="strict-valid"
    source = extraction_source(original)
    if case == "expensive-resolution":
        t=source[UI].decode(); a,b=function_body(t,"style"); body=t[a:b]
        expression="        crate::theme::resolve::bind(self.theme, acc, None, self.surface)"
        body=replace(body,expression,'''        for _ in 0..256 {
            std::hint::black_box(crate::theme::resolve::bind(
                std::hint::black_box(self.theme),std::hint::black_box(acc),None,std::hint::black_box(self.surface)));
        }
'''+expression)
        source[UI]=(t[:a]+body+t[b:]).encode()
    if case == "paint-only":
        path="crates/tui/src/ui/paint.rs"; t=source[path].decode(); a,b=function_body(t,"fill"); body=t[a:b]
        body=replace(body,"            for pos in area.positions() {","            for _paint_repeat in 0..64 { for pos in area.positions() {")
        body=replace(body,"        self.mark_area(area, Some(s));","        }\n        self.mark_area(area, Some(s));")
        source[path]=(t[:a]+body+t[b:]).encode()
    frame = (HERE / "style-timing-bootstrap-frame.rs").read_text()
    if baseline:
        frame = frame.replace("/* PROBE_IMPORT_AND_SUBJECT */", "")
        frame = frame.replace("/* HARNESS_INIT */", "let mut h=Harness::new(App::with_page(PageId::Lists),Theme::junie(),120,40);")
        frame = frame.replace("/* MODE_RESET */", "")
        frame = frame.replace("/* OBSERVATION */", "let reported=0u128; let reported_mode=mode; let cal_source=1; let policy=1; let witness:Vec<u8>=vec![]; let intervals:Vec<(u8,u128,u128,u32)>=vec![];")
        frame = frame.replace("/* EFFECTIVE_CLOCK */", 'println!("STYLE_EFFECTIVE_TIME|{batch}|{mode}|{denominator}");')
        frame = frame.replace("/* EXTRA_BRANCH_PROOFS */", "")
        source[TEST] = frame.encode()
        return source
    text = source[UI].decode()
    text = replace(text, "    reference: Option<ReferenceScope>,\n", "    reference: Option<ReferenceScope>,\n    #[cfg(feature = \"testing\")]\n    style_timing_probe: Option<&'f crate::style_timing_bootstrap::Probe>,\n")
    text = replace(text, "            reference: None,\n", "            reference: None,\n            #[cfg(feature = \"testing\")]\n            style_timing_probe: None,\n")
    text = replace(text, "            reference: self.reference,\n", "            reference: self.reference,\n            #[cfg(feature = \"testing\")]\n            style_timing_probe: self.style_timing_probe,\n")
    for identifier, name in enumerate(UI_CENSUS):
        start, _ = function_body(text, name)
        prefix = f'\n        #[cfg(feature = "testing")]\n        if let Some(probe)=self.style_timing_probe {{ probe.visit({identifier}); }}\n'
        if not (case == "omitted-path" and name == "surface_style"):
            prefix += f'        #[cfg(feature = "testing")]\n        let _style_guard=self.style_timing_probe.map(|probe| probe.enter({identifier}));\n'
        if case == "skipped-calibration-resolver" and name == "style":
            prefix += '        #[cfg(feature="testing")] if self.style_timing_probe.is_some_and(|p| p.mode()==crate::style_timing_bootstrap::Mode::Calibration) { return crate::theme::Resolved::default(); }\n'
        text = text[:start+1] + prefix + text[start+1:]
    if case == "double-count":
        start, _ = function_body(text, "with_part")
        text = text[:start+1] + '\n        #[cfg(feature="testing")] let _double=self.style_timing_probe.map(|p|p.enter(99));\n' + text[start+1:]
    # On the actual Lists draw all direct resolve/bind uses live in these leaf
    # entries. Unexpected Ui direct binding call sites are frozen in the census
    # witness below and cannot silently become a zero-valued timing path.
    method = '''
    #[cfg(feature="testing")]
    pub fn style_timing_bootstrap<'s, R>(&'s mut self, probe: &'s crate::style_timing_bootstrap::Probe, f: impl FnOnce(&mut Ui<'_>)->R)->R {
        let mut nested=self.reborrow();
        nested.style_timing_probe=Some(probe);
        f(&mut nested)
    }
    #[cfg(feature="testing")]
    pub(crate) fn style_timing_bootstrap_visit(&self,id:u8) {
        if let Some(probe)=self.style_timing_probe {probe.visit(id);}
    }
    #[cfg(feature="testing")]
    pub(crate) fn style_timing_bootstrap_enter(&self,id:u8)->Option<crate::style_timing_bootstrap::Guard<'_>> {
        self.style_timing_probe.map(|p|p.enter(id))
    }
    #[cfg(feature="testing")]
    pub fn style_timing_bootstrap_branches(&mut self)->String {
        use crate::theme::{StylePatch,StyleDefaults};
        let mut values=Vec::new();
        for family in [Family::BUTTON,Family::custom("timing-empty"),Family::custom("timing-custom")] {
            let v=Variant::DEFAULT; let p=Part::CONTAINER; let flags=StateFlags::empty();
            values.push(self.style(family,v,p,flags));
            values.push(self.style(family,v,p,flags)); // same-key hit after cold miss
            let inherited=self.surface_style();
            values.push(self.style_inherited(family,v,p,flags,inherited));
            values.push(self.style_defaults(family,v,p,flags,StyleDefaults::new(StylePatch::new()),None));
            values.push(self.style_patched(family,v,p,flags,&StylePatch::new()));
            values.push(self.resolve(family,v,p,flags));
            let paint=self.paint_patch(&StylePatch::new());
            self.fill(self.full(),paint);
            let _=self.bg();
            self.with_part(family,v,p,flags,|ui,r| { ui.fill(ui.full(),r.style); });
        }
        {
            let area=self.full();
            let mut row=crate::RowUi::new(self,crate::Id::root("timing-row"),Family::LIST,Variant::DEFAULT,StateFlags::empty(),crate::ItemKey::num(1),area);
            row.part(Part::META,2).text("x");
            row.part(Part::META,0).text("");
        }
        let status=crate::components::status::style_timing_bootstrap_branch(self);
        let grid=crate::components::grid::style_timing_bootstrap_branch(self);
        format!("{values:?}|{status:?}|{grid:?}|{:?}",self.style_cache_stats())
    }
    #[cfg(feature="testing")]
    pub fn style_timing_bootstrap_scope_proof(&mut self) {
        let original=self.style_timing_probe.map(core::ptr::from_ref);
        let inner=crate::style_timing_bootstrap::Probe::new();
        self.style_timing_bootstrap(&inner,|ui| { let _=ui.bg(); });
        assert_eq!(self.style_timing_probe.map(core::ptr::from_ref),original,"normal probe scope leaked");
        let unwind=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.style_timing_bootstrap(&inner,|ui| { let _=ui.bg(); panic!("expected probe scope unwind"); });
        }));
        assert!(unwind.is_err());
        assert_eq!(self.style_timing_probe.map(core::ptr::from_ref),original,"unwind probe scope leaked");
        inner.reset(crate::style_timing_bootstrap::Mode::Disabled);
    }
'''
    text = replace(text, "    /// The current clip rect.\n", method + "\n    /// The current clip rect.\n")
    source[UI] = text.encode()
    for identifier,path,receiver,expression in [
        (8,"crates/tui/src/collection/rowui.rs","self.ui","crate::theme::resolve::bind(theme, delta, self.patch.as_ref(), surface).style"),
        (9,"crates/tui/src/components/status.rs","ui","crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style"),
        (10,"crates/tui/src/components/grid.rs","ui","crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style")]:
        body=f'{{ #[cfg(feature="testing")] {receiver}.style_timing_bootstrap_visit({identifier}); '
        if case!={8:"omitted-row-bind",9:"omitted-status-bind",10:"omitted-grid-bind"}[identifier]:
            body+=f'#[cfg(feature="testing")] let _leaf_guard={receiver}.style_timing_bootstrap_enter({identifier}); '
        body+=expression+' }'
        source[path]=replace(source[path].decode(),expression,body).encode()
    source["crates/tui/src/components/status.rs"]+=b'''
#[cfg(feature="testing")]
pub(crate) fn style_timing_bootstrap_branch(ui:&mut Ui<'_>)->[PaintStyle;2] {
    let bar=StatusBar::new(Id::root("timing-status"));
    [bar.item_style(ui,&StatusItem::new("plain"),StateFlags::empty()),
     bar.item_style(ui,&StatusItem::new("tone").tone(Role::Warning),StateFlags::empty())]
}
'''
    source["crates/tui/src/components/grid.rs"]+=b'''
#[cfg(feature="testing")]
pub(crate) fn style_timing_bootstrap_branch(ui:&mut Ui<'_>)->[PaintStyle;2] {
    [apply_style_delta(ui,PaintStyle::new(),StylePatch::new()),
     apply_style_delta(ui,PaintStyle::new(),StylePatch::new().set_fg(crate::theme::Role::Warning))]
}
'''
    # Color-binding paint helpers are not reached by the closed Lists fixture.
    # If that premise changes, reject uncovered work rather than time painting.
    path="crates/tui/src/ui/paint.rs"; paint=source[path].decode(); start,_=function_body(paint,"dim_layer")
    paint=paint[:start+1]+'\n        #[cfg(feature="testing")] self.style_timing_bootstrap_visit(255);\n'+paint[start+1:]
    source[path]=paint.encode()
    source["crates/tui/src/lib.rs"] += b'\n#[cfg(feature="testing")]\npub mod style_timing_bootstrap;\n'
    probe = (HERE / "style-timing-bootstrap-probe.rs").read_text()
    substitutions = {
        "constant-numerator": ("reported_ns: raw,", "reported_ns: if self.mode.get()==Mode::Measured { 1 } else { raw },"),
        "foreign-numerator": ("reported_ns: raw,", "reported_ns: if self.mode.get()==Mode::Measured { raw + 1_000_000 } else { raw },"),
        "disabled-as-measured": ("Mode::Disabled=>0", "Mode::Disabled=>2"),
        "wrong-membership": ("witness: self.witness.borrow().clone(), intervals,", "witness: self.witness.borrow().iter().rev().copied().collect(), intervals,"),
        "foreign-calibration": ("cal_source: 1,", "cal_source: 7,"),
        "changed-policy": ("policy: 1", "policy: 2"),
    }
    if case in substitutions:
        probe = replace(probe, *substitutions[case])
    if case == "foreign-calibration":
        probe=replace(probe,"        let calibration_stop = if self.mode.get() == Mode::Calibration {\n",
            "        let calibration_stop = if self.mode.get() == Mode::Calibration {\n            for i in 0..64u64 { std::hint::black_box(i.wrapping_mul(19)); }\n")
    if not attestation:
        calibration=1_000_000 if case=="negative-corrected" else 100
        measured=1000 if original_case=="strict-over-budget" else 110
        probe=replace(probe,"Mode::Calibration => self.calibration_stop,",f"Mode::Calibration => self.start + {calibration},")
        probe=replace(probe,"Mode::Measured => self.probe.epoch.elapsed().as_nanos(),",f"Mode::Measured => self.start + {measured},")
    if case == "contaminated-allocation-cache":
        probe = replace(probe, "self.witness.borrow_mut().push(id);", "self.witness.borrow_mut().push(id); if self.mode.get()==Mode::Measured { std::hint::black_box(Box::new([7u8;32])); }")
    source[PROBE] = probe.encode()
    subject = '''
use junie_tui::style_timing_bootstrap::{Probe, Mode};
struct Subject { app: App, probe: Probe }
impl AppTrait for Subject {
    fn update(&mut self,cx:&mut Cx<'_>)->Response<()> { self.app.update(cx) }
    fn draw(&self,ui:&mut Ui<'_>) { ui.style_timing_bootstrap(&self.probe, |ui| self.app.draw(ui)); }
    fn should_quit(&self)->bool { self.app.should_quit() }
    fn keymap(&self)->&junie_tui::KeyMap { self.app.keymap() }
    fn min_size(&self)->junie_tui::Size { self.app.min_size() }
    fn on_esc(&mut self,cx:&mut Cx<'_>)->Response<()> { self.app.on_esc(cx) }
}
impl Observed for Subject { fn semantic(&self)->String { self.app.style_timing_bootstrap_state() } }
'''
    frame = frame.replace("/* PROBE_IMPORT_AND_SUBJECT */", subject)
    frame = frame.replace("/* HARNESS_INIT */", "let mut h=Harness::new(Subject { app:App::with_page(PageId::Lists),probe:Probe::new() },Theme::junie(),120,40);")
    frame = frame.replace("/* MODE_RESET */", "h.app().probe.reset(match mode { 0=>Mode::Disabled,1=>Mode::Calibration,_=>Mode::Measured });")
    frame = frame.replace("/* OBSERVATION */", "let obs=h.app().probe.observation(); let reported=obs.reported_ns; let reported_mode=obs.mode; let cal_source=obs.cal_source; let policy=obs.policy; let witness=obs.witness; let intervals=obs.intervals;")
    if attestation:
        frame=frame.replace("/* EFFECTIVE_CLOCK */",'println!("STYLE_EFFECTIVE_TIME|{batch}|{mode}|{denominator}");')
    else:
        duration="if batch%2==0 {190} else {210}" if case=="uncertain-threshold" else "1000"
        frame=frame.replace("/* EFFECTIVE_CLOCK */",f'let effective_denominator=(witness.len() as u128)*({duration}); println!("STYLE_EFFECTIVE_TIME|{{batch}}|{{mode}}|{{effective_denominator}}");')
        frame=replace(frame,"{mode}|{denominator}|","{mode}|{effective_denominator}|")
    frame = frame.replace("/* EXTRA_BRANCH_PROOFS */", '''
    struct Coverage { probe:Probe, values:std::cell::RefCell<String> }
    impl AppTrait for Coverage {
        fn update(&mut self,_:&mut Cx<'_>)->Response<()> { Response::default() }
        fn draw(&self,ui:&mut Ui<'_>) {
            let value=ui.style_timing_bootstrap(&self.probe,|ui|ui.style_timing_bootstrap_branches());
            *self.values.borrow_mut()=value;
            ui.style_timing_bootstrap_scope_proof();
        }
    }
    let mut prior=None;
    for mode in [Mode::Disabled,Mode::Calibration,Mode::Measured] {
        let probe=Probe::new(); probe.reset(mode);
        let theme=Theme::junie().define_family(junie_tui::theme::Family::custom("timing-custom"), |_| {});
        let mut branch=Harness::new(Coverage{probe,values:std::cell::RefCell::new(String::new())},theme,8,2);
        let mut value=branch.app().values.borrow().clone();
        for y in 0..2 { for x in 0..8 { write!(&mut value,"|{:?}",branch.cell(x,y)).unwrap(); } }
        if let Some(v)=&prior { assert_eq!(v,&value,"branch resolved/cache output changed"); } else {prior=Some(value);}
        let obs=branch.app().probe.observation();
        println!("STYLE_BRANCH|{}|{:?}|{:?}",obs.mode,obs.witness,obs.intervals);
    }
    ''')
    if case=="changed-policy": frame=replace(frame,"for _ in 0..12 { h.draw(); }","for _ in 0..13 { h.draw(); }")
    if case == "forged-denominator":
        frame = replace(frame, "{mode}|{effective_denominator}|", "{mode}|{reported_denominator}|")
        frame = replace(frame, "let cell_bytes=cells(&h);", "let reported_denominator=effective_denominator*1000; let cell_bytes=cells(&h);")
    if case == "altered-cells-state":
        path = "apps/showcase/src/pages/lists.rs"
        source[path] = replace(source[path].decode(), '"Chosen: "', '"Forged: "').encode()
    source[TEST] = frame.encode()
    return source


def build(root, target, source):
    actual.write_sources(root,source)
    identity=actual.compiler_identity()
    actual.validate_compiler_identity(identity)
    argv=[identity["cargo"]["path"],"test","--release","--no-run","--locked","--offline","--message-format=json","-p","showcase","--test","style_timing_bootstrap"]
    env=build_environment(target,identity)
    done=subprocess.run(argv,cwd=root,env=env,capture_output=True,timeout=300,check=False)
    actual.validate_compiler_identity(identity)
    require(done.returncode==0,"style source compilation failed: "+done.stderr.decode()[-3500:]+done.stdout.decode()[-3500:])
    artifacts=[json.loads(line) for line in done.stdout.splitlines() if line.startswith(b'{')]
    executables=[Path(row["executable"]) for row in artifacts if row.get("reason")=="compiler-artifact" and row.get("executable") and row["target"]["name"]=="style_timing_bootstrap"]
    require(len(executables)==1,"unique compiler-produced style test missing")
    binary=executables[0].read_bytes()
    artifact=next(row for row in artifacts if row.get("executable")==str(executables[0]))
    require(artifact["profile"]["opt_level"]=="3" and artifact["profile"]["debug_assertions"] is False and artifact["profile"]["overflow_checks"] is False,"effective compiler artifact profile changed")
    return {"executable":binary,"compilation":{"argv":argv,"exit":done.returncode,"toolchain":identity,
        "executable_sha256":sha(binary),"compiler_messages_sha256":sha(done.stdout),"compiler_stderr_sha256":sha(done.stderr),
        "cargo_lock_sha256":sha(source["Cargo.lock"]),"source_inventory_sha256":sha(canonical({p:sha(b) for p,b in source.items()})),
        "build_policy":{"release":True,"offline":True,"locked":True,"environment":{k:v for k,v in env.items() if k.startswith(("CARGO_","RUST"))},"artifact_profile":artifact["profile"]}},
        "source":source}


def build_environment(target,identity):
    # Cargo environment can override both CLI-looking profiles and RUSTFLAGS.
    # Remove all inherited Cargo/Rust settings, then fix the effective contract.
    env={k:v for k,v in os.environ.items() if not k.startswith(("CARGO_","RUST"))}
    env.update(CARGO_TARGET_DIR=str(target),CARGO_NET_OFFLINE="true",RUSTC=identity["rustc"]["path"],
        RUSTC_WRAPPER="",RUSTC_WORKSPACE_WRAPPER="",RUSTFLAGS="--cap-lints=allow",
        CARGO_PROFILE_RELEASE_OPT_LEVEL="3",CARGO_PROFILE_RELEASE_DEBUG="0",
        CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS="false",CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS="false",
        CARGO_PROFILE_RELEASE_LTO="false",CARGO_PROFILE_RELEASE_CODEGEN_UNITS="16",
        CARGO_PROFILE_RELEASE_PANIC="unwind",CARGO_PROFILE_RELEASE_INCREMENTAL="false",CARGO_PROFILE_RELEASE_STRIP="none")
    return env


def execute_pair(baseline, subject, directory, unreadable=()):
    """Two warmed actual binaries; one control draw then one subject draw per slot."""
    processes=[]; errors=[]; buffers=[b"",b""]; output=[b"",b""]
    deadline=time.monotonic()+55
    def until(index, marker):
        process=processes[index]
        while True:
            while b"\n" in buffers[index]:
                line,buffers[index]=buffers[index].split(b"\n",1)
                if marker in line: return True
            ready,_,_=select.select([process.stdout],[],[],max(0,deadline-time.monotonic()))
            require(ready and time.monotonic()<deadline,"actual frame handshake timeout")
            data=os.read(process.stdout.fileno(),65536)
            if not data: return False
            buffers[index]+=data; output[index]+=data
    try:
        for index,binary in enumerate((baseline,subject)):
            executable=directory/("style-control" if index==0 else "style-subject")
            executable.write_bytes(binary); executable.chmod(0o755)
            profile=actual.sandbox_module().sandbox(writable=[],unreadable=list(unreadable))
            errors.append(tempfile.TemporaryFile(dir=directory))
            processes.append(subprocess.Popen(["/usr/bin/sandbox-exec","-p",profile,str(executable),"--nocapture","--test-threads=1"],cwd=directory,
                env={"PATH":"/usr/bin:/bin","LC_ALL":"C"},stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=errors[-1],bufsize=0))
        require(all(until(i,b"STYLE_READY") for i in range(2)),"actual warmup failed")
        active=True
        for _ in range(27):
            for i in range(2):
                processes[i].stdin.write(b"draw\n"); processes[i].stdin.flush()
                if not until(i,b"STYLE_ROW|"): active=False; break
            if not active: break
        results=[]
        for i,process in enumerate(processes):
            process.stdin.close(); process.stdin=None
            tail,_=process.communicate(timeout=max(1,deadline-time.monotonic()))
            errors[i].seek(0); stderr=errors[i].read()
            results.append(subprocess.CompletedProcess(process.args,process.returncode,output[i]+tail,stderr))
        return results
    finally:
        for process in processes:
            if process.poll() is None: process.kill(); process.wait()
        for file in errors: file.close()


def parse(result):
    require(result.returncode==0,"real style subject failed: "+result.stderr.decode()[-2000:])
    frames=[line.split(b"STYLE_FRAME|",1)[1] for line in result.stdout.splitlines() if b"STYLE_FRAME|" in line]
    require(len(frames)==1,"complete production frame missing")
    rows=[]; times={}; effective={}; branches=[]; caches={}
    for line in result.stdout.decode().splitlines():
        if "STYLE_EFFECTIVE_TIME|" in line:
            b,m,n=map(int,line.split("STYLE_EFFECTIVE_TIME|",1)[1].split("|")); effective[(b,m)]=n
        if "STYLE_CACHE|" in line:
            b,m,value=line.split("STYLE_CACHE|",1)[1].split("|",2); caches[(int(b),int(m))]=value
        if "STYLE_BRANCH|" in line:
            mode,witness,intervals=line.split("STYLE_BRANCH|",1)[1].split("|",2)
            branches.append({"mode":int(mode),"witness":ast.literal_eval(witness),"intervals":ast.literal_eval(intervals)})
        if "STYLE_FRAME_TIME|" in line:
            b,m,n=map(int,line.split("STYLE_FRAME_TIME|",1)[1].split("|")); times[(b,m)]=n
        if "STYLE_ROW|" not in line: continue
        f=line.split("STYLE_ROW|",1)[1].split("|",12)
        require(len(f)==13,"style observation field count")
        nums=list(map(int,f[:11])); witness=ast.literal_eval(f[11]); intervals=ast.literal_eval(f[12])
        rows.append(dict(zip(["batch","mode","frame_ns","allocations","bytes","cache_hits","cache_misses","reported_ns","reported_mode","cal_source","policy"],nums),witness=witness,intervals=intervals))
    require([(r["batch"],r["mode"]) for r in rows]==[(b,m) for b in range(9) for m in range(3)],"fixed interleaved batch schedule changed")
    require(len(times)==27,"independent draw boundary ledger missing")
    require(len(effective)==27,"independent effective clock ledger missing")
    require(len(caches)==27,"actual cache contents missing")
    for row in rows:
        row["boundary_ns"]=times[(row["batch"],row["mode"])]; row["effective_ns"]=effective[(row["batch"],row["mode"])]; row["cache_contents"]=caches[(row["batch"],row["mode"])]
    return {"frame":frames[0].decode(),"rows":rows,"branches":branches,"stdout_sha256":sha(result.stdout),"stderr_sha256":sha(result.stderr)}


def judge(observed, baseline, *, injected=False, strict=False, arithmetic=False, expected_witness=None, attest=False):
    require(not (strict and injected and not arithmetic),"strict product measurement requires Instant")
    require(observed["frame"]==baseline["frame"],"altered full cells/cursor/semantic state")
    rows=observed["rows"]; baseline_rows=baseline["rows"]
    reference=rows[0]["witness"]
    require(reference and set(reference)<=set(range(len(CENSUS))),"actual invocation witness missing or foreign")
    require(8 in reference,"real compact List RowUi binding missing")
    require(expected_witness is None or reference==expected_witness,"frozen source invocation census changed")
    branches=observed["branches"]
    require([b["mode"] for b in branches]==[0,1,2],"branch modes missing")
    for branch in branches:
        require(set(branch["witness"])==set(range(len(CENSUS))),"branch census incomplete")
        require(branch["witness"]==branches[0]["witness"],"branch membership changed")
        require(branch["mode"]==0 or ([i[0] for i in branch["intervals"]]==branch["witness"] and all(i[3]==0 for i in branch["intervals"])),"nested/double/invalid interval")
    corrected=[]; denominators=[]
    for index,row in enumerate(rows):
        require(row["policy"]==1,"changed benchmark policy")
        require(row["reported_mode"]==row["mode"],"disabled-hook numerator reported as measured")
        require(row["cal_source"]==1,"foreign calibration")
        require(row["witness"]==reference,"mismatched invocation membership/order")
        for key in ["allocations","bytes","cache_hits","cache_misses","cache_contents"]:
            require(row[key]==baseline_rows[index][key],"contaminated allocation/cache state: "+key)
        intervals=row["intervals"]
        require(all(isinstance(i,tuple) and len(i)==4 and 0<=i[1]<=i[2] and i[3]==0 for i in intervals),"nested/double/invalid interval")
        if row["mode"]==0:
            require(not intervals and row["reported_ns"]==0,"disabled timing produced numerator")
        else:
            require([i[0] for i in intervals]==reference,"omitted/double-counted resolution path")
            require(sum(i[2]-i[1] for i in intervals)==row["reported_ns"],"foreign or constant numerator")
        require(row["frame_ns"]>0,"zero denominator")
        require(row["frame_ns"]==row["effective_ns"] and (injected or row["frame_ns"]==row["boundary_ns"]),"forged denominator differs from draw boundary")
        if row["mode"]==2:
            value=row["reported_ns"]-rows[index-1]["reported_ns"]
            require(attest or value>=0,"negative corrected duration")
            corrected.append(value)
            denominators.append(row["frame_ns"] if injected else baseline_rows[index]["frame_ns"])
    low=min(corrected)/max(denominators); high=max(corrected)/min(denominators)
    if injected: require(not low<=0.05<high,"uncertainty crosses threshold")
    quality="INVALID" if min(corrected)<0 else "REJECT" if high>0.05 else "PASS"
    if strict and not attest: require(high<=0.05,f"strict style ratio exceeds threshold: upper={high}, lower={low}")
    return {"raw_numerator_ns":[r["reported_ns"] for r in rows if r["mode"]==2],
            "calibration_ns":[r["reported_ns"] for r in rows if r["mode"]==1],"corrected_ns":corrected,
            "uninstrumented_denominator_ns":None if injected else denominators,
            "arithmetic_denominator_ns":denominators if injected else None,"lower_ratio":low,"upper_ratio":high,
            "quality":quality,"strict_performance_pass":not injected and quality=="PASS",
            "strict_arithmetic_pass":injected and quality=="PASS","clock_kind":"injected-arithmetic-only" if injected else "Instant",
            "census":reference,"measurement_policy":POLICY}


NEGATIVES={
    "omitted-path":"nested/double/invalid interval",
    "double-count":"nested/double/invalid interval",
    "foreign-numerator":"foreign or constant numerator",
    "constant-numerator":"foreign or constant numerator",
    "disabled-as-measured":"branch modes missing",
    "skipped-calibration-resolver":"real style subject failed:",
    "wrong-membership":"frozen source invocation census changed",
    "altered-cells-state":"altered full cells/cursor/semantic state",
    "contaminated-allocation-cache":"contaminated allocation/cache state:",
    "forged-denominator":"forged denominator differs from draw boundary",
    "foreign-calibration":"foreign calibration",
    "negative-corrected":"negative corrected duration",
    "changed-policy":"changed benchmark policy",
    "uncertain-threshold":"uncertainty crosses threshold",
    "omitted-row-bind":"nested/double/invalid interval",
    "omitted-status-bind":"nested/double/invalid interval",
    "omitted-grid-bind":"nested/double/invalid interval",
    "strict-over-budget":"strict style ratio exceeds threshold",
    "strict-injected-forgery":"strict product measurement requires Instant",
}


def inspect_pair(results, injected=False, strict=False, arithmetic=False, expected_witness=None, attest=False):
    baseline,subject=results
    return judge(parse(subject),parse(baseline),injected=injected,strict=strict,arithmetic=arithmetic,expected_witness=expected_witness,attest=attest)


def case_options(name):
    attest=name.startswith("attest-")
    return {"injected":not attest,"strict":attest or name.startswith("strict-"),
        "arithmetic":not attest and name!="strict-injected-forgery","attest":attest}


def prepare(case_names=None):
    original=actual.main_sources(actual.frozen_archive())
    names=case_names or ["valid",*NEGATIVES,"expensive-resolution","paint-only","strict-valid",
        "attest-valid","attest-expensive-resolution","attest-paint-only"]
    prepared=[]; expected_witness=None
    with tempfile.TemporaryDirectory(prefix="tc-architecture-main-style-",dir="/tmp") as directory:
        temp=Path(directory); root=temp/"source"; target=temp/"target"
        baseline=build(root,target,source_fixture(original,baseline=True))
        for name in names:
            control=build(root,target,source_fixture(original,name,baseline=True)) if name in {"expensive-resolution","paint-only","strict-over-budget","attest-expensive-resolution","attest-paint-only"} else baseline
            subject=build(root,target,source_fixture(original,name))
            results=execute_pair(control["executable"],subject["executable"],temp)
            metric=None; failure=None
            try: metric=inspect_pair(results,**case_options(name),expected_witness=expected_witness)
            except ValueError as error: failure=str(error)
            if name in NEGATIVES:
                if name=="skipped-calibration-resolver":
                    require(results[1].returncode==101 and b"three-mode full cells/cursor/state changed" in results[1].stderr,"calibration mutant failed outside actual frame equality")
                require(failure and failure.startswith(NEGATIVES[name]),name+" failed for wrong reason: "+str(failure))
            else: require(failure is None,"actual positive failed: "+str(failure))
            if name=="valid": expected_witness=metric["census"]
            prepared.append({"name":name,"subject":subject,"baseline":control,"metric":metric,"expected_witness":expected_witness,
                "category":"ATTESTATION" if name.startswith("attest-") else "ARCHITECTURE" if name in NEGATIVES else None,"failure":failure})
            print("prepared "+name,file=sys.stderr,flush=True)
        if case_names is None:
            metrics={r["name"]:r["metric"] for r in prepared if r["metric"]}
            ordinary,expensive,paint=(metrics[n] for n in ("attest-valid","attest-expensive-resolution","attest-paint-only"))
            require(min(expensive["raw_numerator_ns"])>max(ordinary["raw_numerator_ns"]),"real resolver cost did not move raw numerator")
            require(min(paint["uninstrumented_denominator_ns"])>max(ordinary["uninstrumented_denominator_ns"]),"real repeated paint did not move production denominator")
            require(max(paint["raw_numerator_ns"])<min(expensive["raw_numerator_ns"]),"paint-only cost contaminated raw resolver numerator")
            actual.write_sources(root,source_fixture(original))
            identity=actual.compiler_identity()
            actual.validate_compiler_identity(identity)
            featureless=subprocess.run([identity["cargo"]["path"],"check","--release","--locked","--offline","--no-default-features","-p","junie-tui","--lib"],cwd=root,env=build_environment(target,identity),capture_output=True,timeout=180,check=False)
            actual.validate_compiler_identity(identity)
            require(featureless.returncode==0,"testing-disabled compilation failed: "+featureless.stderr.decode()[-1500:])
    return prepared


class StyleFixture(runner.Fixture):
    def __init__(self,prepared):
        super().__init__("architecture","valid")
        self.prepared=prepared
        original=actual.main_sources(actual.frozen_archive())
        archive=(HERE/"main-source.tar.gz").read_bytes()
        (self.public/"main-source.tar.gz").write_bytes(archive)
        (self.source/"main-source.tar.gz").write_bytes(archive)
        # Exact archive plus sparse byte overrides is a complete source
        # representation, not a reduced source scope. Avoid creating/deleting
        # thousands of duplicate files for each independently isolated replay.
        directories={}
        for kind in ("subject","baseline"):
            visible=self.public/(kind+"-overrides"); visible.mkdir(); directories[kind]=str(visible)
            for path,data in prepared[kind]["source"].items():
                if original.get(path)==data: continue
                for base in (visible,self.source/(kind+"-overrides")):
                    destination=base/path; destination.parent.mkdir(parents=True,exist_ok=True); destination.write_bytes(data)
        runner.git(self.source,"add","main-source.tar.gz","subject-overrides","baseline-overrides")
        self.source_tree=runner.git(self.source,"write-tree")
        compiler=prepared["subject"]["compilation"]["toolchain"]["rustc"]
        self.context.update(tree=self.source_tree,tool={"path":compiler["path"],"sha256":compiler["sha256"]},architecture_profile={
            "schema":"tc-style-timing-actual-profile/v1","kind":"style-timing",
            "source_representation":{"kind":"pinned-archive-plus-byte-overrides","archive":str(self.public/"main-source.tar.gz"),"overrides":directories,
                "deleted_paths":{kind:sorted(set(original)-set(prepared[kind]["source"])) for kind in ("subject","baseline")}},
            "main_commit":actual.PIN,"main_archive_sha256":actual.ARCHIVE_SHA256,"policy":POLICY,"census":CENSUS,
            "sources":{kind:{p:sha(b) for p,b in prepared[kind]["source"].items()} for kind in ("subject","baseline")},
            "expected_invocations":prepared["expected_witness"],
            "acceptance":"attestation" if prepared["name"].startswith("attest-") else "strict" if prepared["name"]=="strict-injected-forgery" else "strict-arithmetic" if prepared["name"].startswith("strict-") else "protocol-arithmetic",
            "clock_kind":"Instant" if prepared["name"].startswith("attest-") else "injected-arithmetic-only"})
        runner.save(self.context_path,self.context); self.context_hash=sha(self.context_path.read_bytes()); self.before=self.snapshot()

    def launch(self,request):
        require(request=={"schema":"tc-proof-runner-observe/v1","nonce":self.nonce,"operation":"architecture","source_commit":self.source_commit,"tree":self.source_tree},"style observer request binding")
        require(not self.events,"style observer replay")
        p=self.prepared
        results=execute_pair(p["baseline"]["executable"],p["subject"]["executable"],self.private,unreadable=[self.public,self.output])
        failure=None; metric=None
        try: metric=inspect_pair(results,**case_options(p["name"]),expected_witness=p["expected_witness"])
        except ValueError as error: failure=str(error)
        if p["name"]=="skipped-calibration-resolver":
            require(results[1].returncode==101 and b"three-mode full cells/cursor/state changed" in results[1].stderr,"replayed calibration mutant changed cause")
        if p["category"]=="ARCHITECTURE": require(failure and failure.startswith(NEGATIVES[p["name"]]),"replayed negative changed cause: "+str(failure))
        else: require(failure is None,"replayed positive failed: "+str(failure))
        self.actual_category="ARCHITECTURE" if failure or (p["category"]=="ATTESTATION" and metric["quality"]!="PASS") else None
        payload={"schema":"tc-style-timing-actual-observation/v1","policy":POLICY,
            "control":{"exit":results[0].returncode,"stdout":base64.b64encode(results[0].stdout).decode(),"stderr":base64.b64encode(results[0].stderr).decode()},
            "subject":{"exit":results[1].returncode,"stdout":base64.b64encode(results[1].stdout).decode(),"stderr":base64.b64encode(results[1].stderr).decode()}}
        event={"operation":"architecture","run_id":self.run_id,"tree":self.source_tree,"source_commit":self.source_commit,
            "compilation":{k:p[k]["compilation"] for k in ("baseline","subject")},"exit":results[1].returncode,"payload":payload}
        self.events.append(event); self.metric=metric
        return event

    def validate(self,result,category):
        require(len(self.events)==1,"style qualification omitted protected execution")
        if category=="ATTESTATION": category=self.actual_category
        require(category==self.actual_category,"private style expectation changed")
        code,report=result
        if category:
            require(code!=0 and report["status"]=="rejected" and report["category"]==category and report["outputs"]=={},"actual negative did not reject exactly")
        else:
            require(code==0 and report["status"]=="passed" and report["category"] is None and canonical(report["outputs"])==canonical({"observations":self.events}),"actual positive rejected or observations forged")


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--self-test",action="store_true")
    parser.add_argument("--runner",type=Path)
    args=parser.parse_args()
    require(args.self_test or args.runner,"qualification mode required")
    require(args.self_test or args.runner.is_file(),"actual submitted dispatcher required")
    prepared=prepare(); recovery=prepared[0]
    accepted=0; negatives=0; recoveries=0; attacks=0; executions=[]; attestations=[]
    for row in prepared:
        selections=[row]+([recovery] if row["category"] else [])
        if row["category"] and row["name"].startswith("strict-"):
            selections.append(next(p for p in prepared if p["name"]=="strict-valid"))
        for index,selected in enumerate(selections):
            fixture=StyleFixture(selected)
            try:
                executable=args.runner.resolve() if args.runner else fixture.public/"style-transport-candidate"
                if args.self_test: executable.write_text(adapter_source()); executable.chmod(0o755)
                try: fixture.validate(fixture.run(executable),selected["category"])
                except ValueError as error:
                    raise ValueError(str(error)+"; case="+selected["name"]+"; private_observer_violations="+repr(fixture.violations)) from error
                print("qualified "+selected["name"],file=sys.stderr,flush=True)
                executions.append({"case":selected["name"],"event_sha256":sha(canonical(fixture.events[0])),
                    "source_tree":fixture.source_tree,"source_inventory_sha256":selected["subject"]["compilation"]["source_inventory_sha256"],
                    "subject_executable_sha256":selected["subject"]["compilation"]["executable_sha256"],"category":fixture.actual_category})
                if selected["category"]=="ATTESTATION":
                    metric=dict(fixture.metric); metric.pop("census")
                    attestations.append({"case":selected["name"],"measurement":metric,"event_sha256":sha(canonical(fixture.events[0]))})
                accepted+=selected["category"] is None; negatives+=selected["category"]=="ARCHITECTURE"
                recoveries+=index>0
            finally: fixture.cleanup()
    if args.self_test:
        for mode,selected in [("always-pass",prepared[1]),("forged",recovery),("zero",recovery),("typed-bool",recovery),("typed-float",recovery),
                ("always-pass-attestation",next(p for p in prepared if p["name"]=="attest-expensive-resolution"))]:
            fixture=StyleFixture(selected)
            try:
                adapter=fixture.public/"malicious-style-candidate"; adapter.write_text(adapter_source("always-pass" if mode=="always-pass-attestation" else mode)); adapter.chmod(0o755)
                try: fixture.validate(fixture.run(adapter),selected["category"])
                except ValueError as error:
                    expected={"always-pass":{"actual negative did not reject exactly"},"always-pass-attestation":{"actual negative did not reject exactly"},"forged":{"forged observation"},"zero":{"style qualification omitted protected execution"},
                        "typed-bool":{"forged observation bytes"},"typed-float":{"forged observation bytes"}}[mode]
                    require(str(error) in expected,"transport attack failed for unrelated cause: "+str(error)); attacks+=1
                    if selected["category"]=="ATTESTATION":
                        metric=dict(fixture.metric); metric.pop("census")
                        attestations.append({"case":selected["name"]+"/always-pass-attack","measurement":metric,"event_sha256":sha(canonical(fixture.events[0]))})
                else: raise ValueError("external runner forgery accepted")
            finally: fixture.cleanup()
            fixture=StyleFixture(recovery)
            try:
                adapter=fixture.public/"fresh-style-recovery"; adapter.write_text(adapter_source()); adapter.chmod(0o755)
                fixture.validate(fixture.run(adapter),None); recoveries+=1; accepted+=1
            finally: fixture.cleanup()
    metric=dict(recovery["metric"]); metric["census_count"]=len(metric.pop("census"))
    metric["entry_counts"]={name:recovery["metric"]["census"].count(i) for i,name in enumerate(CENSUS)}
    preparation_attestations=[]
    for row in prepared:
        if row["category"]=="ATTESTATION":
            measured=dict(row["metric"]); measured.pop("census")
            preparation_attestations.append({"case":row["name"],"measurement":measured,
                "source_inventory_sha256":row["subject"]["compilation"]["source_inventory_sha256"],"executable_sha256":row["subject"]["compilation"]["executable_sha256"]})
    print(json.dumps({"prepared":len(prepared),"negative_cases":negatives,"required_negative_classes":14,"positive_control_runs":accepted,"fresh_recoveries":recoveries,
        "malicious_external_runner_rejections":attacks,"arithmetic_control_not_performance":metric,"actual_Instant_attestations":attestations,
        "pre_dispatch_Instant_attestations":preparation_attestations,"source_pin":actual.PIN,"archive_sha256":actual.ARCHIVE_SHA256,"qualified_execution_receipts":executions},sort_keys=True))


def adapter_source(mode="valid"):
    """Transport self-test consumer: evaluates raw measurements, never expected labels."""
    prelude="#!"+sys.executable+"\nimport ast,base64,hashlib,json,os,subprocess,sys\n"
    prelude+="def require(condition,message):\n    if not condition: raise ValueError(message)\n"
    prelude+="def sha(data): return hashlib.sha256(data).hexdigest()\n"
    prelude+="CENSUS="+repr(CENSUS)+"\nPOLICY="+repr(POLICY)+"\n"
    prelude+=inspect.getsource(parse)+"\n"+inspect.getsource(judge)+"\n"
    body='''
context=json.load(open(sys.argv[-1]))
events=[]; failure=False
if MODE != "zero":
    request={'schema':'tc-proof-runner-observe/v1','nonce':os.environ['TC_PROOF_OBSERVER_NONCE'],'operation':'architecture','source_commit':os.environ['TC_PROOF_ORACLE_COMMIT'],'tree':os.environ['TC_PROOF_SOURCE_TREE']}
    with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']),'w') as f: f.write(json.dumps(request)+'\\n'); f.flush()
    with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as f: event=json.loads(f.readline())
    events=[event]
    def result(which):
        p=event['payload'][which]
        return subprocess.CompletedProcess([],p['exit'],base64.b64decode(p['stdout']),base64.b64decode(p['stderr']))
    try:
        profile=context['architecture_profile']
        metric=judge(parse(result('subject')),parse(result('control')),injected=profile['clock_kind']=='injected-arithmetic-only',strict=profile['acceptance'] in {'strict','strict-arithmetic','attestation'},arithmetic=profile['acceptance'] in {'strict-arithmetic','protocol-arithmetic'},attest=profile['acceptance']=='attestation',expected_witness=profile['expected_invocations'])
        if profile['acceptance']=='attestation': failure=metric['quality']!='PASS'
    except ValueError: failure=True
    if MODE == 'forged': event['payload']['subject']['stdout']='forged'
if MODE in {'always-pass','zero'}: failure=False
digest=lambda e: hashlib.sha256(json.dumps(e,sort_keys=True,separators=(',',':')).encode()).hexdigest()
observed_digests=[digest(e) for e in events]
if MODE=='typed-bool': events[0]['payload']['subject']['exit']=False
if MODE=='typed-float': events[0]['payload']['subject']['exit']=0.0
report={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],'operation':'architecture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],'status':'rejected' if failure else 'passed','category':'ARCHITECTURE' if failure else None,'observation_digests':observed_digests,'outputs':{} if failure else {'observations':events}}
with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(report,f)
sys.exit(1 if failure else 0)
'''
    return prelude+"MODE="+repr(mode)+"\n"+body


if __name__=="__main__": main()
