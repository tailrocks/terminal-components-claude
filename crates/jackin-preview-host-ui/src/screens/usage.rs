use termrock::Id;

pub const ROOT: Id = Id::root("jackin.usage");
pub const TABS: Id = ROOT.sub("tabs");
pub const LIST: Id = ROOT.sub("list");
pub const DETAIL: Id = ROOT.sub("detail");
pub const CLOSE: Id = DETAIL.sub("close");
pub const MANAGE: Id = DETAIL.sub("manage");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Overview,
    Registration,
    Quota,
}

impl Tab {
    pub const ALL: [Self; 3] = [Self::Overview, Self::Registration, Self::Quota];

    pub const fn next(self) -> Self {
        match self {
            Self::Overview => Self::Registration,
            Self::Registration => Self::Quota,
            Self::Quota => Self::Overview,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Quota,
            Self::Registration => Self::Overview,
            Self::Quota => Self::Registration,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsageState {
    pub tab: Tab,
    selected: Option<String>,
    detail_open: bool,
}

impl UsageState {
    pub fn select(&mut self, id: Option<impl Into<String>>) {
        self.selected = id.map(Into::into).filter(|id| !id.is_empty());
        if self.selected.is_none() {
            self.detail_open = false;
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub const fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    pub const fn previous_tab(&mut self) {
        self.tab = self.tab.previous();
    }

    pub const fn open_detail(&mut self) -> bool {
        if self.selected.is_some() {
            self.detail_open = true;
        }
        self.detail_open
    }

    pub const fn close_detail(&mut self) {
        self.detail_open = false;
    }

    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }

    pub fn manage_target(&self) -> Option<&str> {
        self.selected()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn crumb(&self, account: Option<(&str, &str)>) -> String {
        if self.detail_open()
            && let Some((surface, name)) = account
        {
            return format!("Usage › {surface} › {name}");
        }
        match self.tab {
            Tab::Overview => "Usage › Overview".to_owned(),
            Tab::Registration => "Usage › Registration".to_owned(),
            Tab::Quota => "Usage › Quota".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_detail_is_read_only_and_requires_selection() {
        let mut state = UsageState::default();
        assert!(!state.open_detail());
        state.select(Some("acct-claude"));
        assert!(state.open_detail());
        assert_eq!(state.manage_target(), Some("acct-claude"));
        state.close_detail();
        assert_eq!(state.selected(), Some("acct-claude"));
        assert!(!state.detail_open());
    }

    #[test]
    fn usage_tabs_wrap_without_mutating_selection() {
        let mut state = UsageState::default();
        state.select(Some("acct"));
        state.previous_tab();
        assert_eq!(state.tab, Tab::Quota);
        state.next_tab();
        assert_eq!(state.tab, Tab::Overview);
        assert_eq!(state.selected(), Some("acct"));
    }

    #[test]
    fn usage_crumb_follows_the_tab_until_detail_is_open() {
        let mut state = UsageState::default();
        assert_eq!(state.crumb(None), "Usage › Overview");
        state.next_tab();
        assert_eq!(state.crumb(None), "Usage › Registration");
        state.next_tab();
        assert_eq!(state.crumb(Some(("Codex", "Experiments"))), "Usage › Quota");
        state.select(Some("acct"));
        state.open_detail();
        assert_eq!(
            state.crumb(Some(("Codex", "Experiments"))),
            "Usage › Codex › Experiments"
        );
        state.close_detail();
        assert_eq!(state.crumb(Some(("Codex", "Experiments"))), "Usage › Quota");
    }
}
