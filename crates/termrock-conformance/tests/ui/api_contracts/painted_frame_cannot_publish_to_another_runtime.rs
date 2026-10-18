use ratatui::Frame;
use termrock::runtime::{App, Runtime};

fn wrong<A: App>(one: &mut Runtime<A>, two: &mut Runtime<A>, frame: &mut Frame<'_>) {
    let candidate = one.draw(frame);
    candidate.commit_presented(two);
}

fn main() {}
