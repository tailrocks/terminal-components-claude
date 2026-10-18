use ratatui::Frame;
use termrock::runtime::{App, Runtime};

fn twice<A: App>(runtime: &mut Runtime<A>, frame: &mut Frame<'_>) {
    let candidate = runtime.draw(frame);
    candidate.commit_presented();
    candidate.commit_presented();
}

fn main() {}
