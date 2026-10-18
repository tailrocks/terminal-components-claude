use termrock::navigation::{FilterList, ItemRow};

struct PaintedOnly;

fn main() {
    let _ = FilterList::<PaintedOnly, ItemRow>::semantic_width(&[]);
}
