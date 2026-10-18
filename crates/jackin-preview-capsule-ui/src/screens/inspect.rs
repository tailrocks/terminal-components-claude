#![doc = r" Read-only instance inspection composition."]

use termrock::Id;

#[doc = r" Inspector root."]
pub const ROOT: Id = Id::root("jackin.inspect");
#[doc = r" Inspector body."]
pub const BODY: Id = ROOT.sub("body");

#[doc = r" Inspection state.  It intentionally stores only an instance id; all"]
#[doc = r" displayed fields are projected from the current `World` on every draw."]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InspectState {
    #[doc = r" Identifier of the instance being inspected, if selected."]
    pub instance: Option<String>,
}
