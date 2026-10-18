use ratatui::Frame;
use termrock::runtime::{App, Runtime};

fn stale<A: App>(runtime: &mut Runtime<A>, frame: &mut Frame<'_>) {
    let candidate = runtime.draw(frame);
    let _ = runtime.app_mut();
    candidate.commit_presented();
}

fn main() {}
