//! Default-feature downstream use of semantic rows with application-owned data.

use termrock::blocks::AppShell;
use termrock::composition::{self, Group};
use termrock::navigation::{AsItem, Item, SemanticItemRow, Tree, TreeNode, TreeState};
use termrock::{Id, ItemKey, Ui};

const FILES: Id = Id::root("example.documents.files");

struct Document {
    id: u64,
    name: &'static str,
    location: &'static str,
    status: &'static str,
    is_folder: bool,
}

impl AsItem for Document {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::num(self.id), self.name)
            .glyph(if self.is_folder { "D" } else { "F" })
            .detail(self.location)
            .tag(self.status)
    }
}

fn document_node(document: &Document) -> TreeNode {
    if document.is_folder {
        TreeNode::parent(0)
    } else {
        TreeNode::leaf(1)
    }
}

#[allow(dead_code)]
fn draw_documents(ui: &mut Ui<'_>, documents: &[Document], state: &TreeState) {
    let tree = Tree::new(FILES)
        .key(|document: &Document| ItemKey::num(document.id))
        .node(&document_node)
        .row(SemanticItemRow::new());
    let files = Group::column([composition::tree(&tree, state, documents)]);
    AppShell::new()
        .body_slot(&files)
        .show(ui)
        .expect("document composition resolves");
}

fn main() {}
