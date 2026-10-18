//! §16.1 `response.rs` / §21 item 4: `BitOr` folds *dispatch* answers, so it
//! is defined for `Response<()>` only. Folding two action-carrying responses
//! would have to silently drop one action.

use junie_tui::Response;

fn main() {
    let a: Response<u8> = Response::action(1);
    let b: Response<u8> = Response::action(2);
    let _ = a | b;
}
