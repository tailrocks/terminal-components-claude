use termrock::runtime::{Cx, Moment};

fn synchronize(cx: &mut Cx<'_>, elapsed: Moment) {
    let _ = cx.sync_feedback_time(elapsed);
}

fn main() {}
