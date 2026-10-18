use termrock::Id;

pub const ROOT: Id = Id::root("jackin.inspect");
pub const BODY: Id = ROOT.sub("body");

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InspectState {
    pub instance: Option<String>,
}
