






use junie_tui::shell::Router;

use crate::app::Route;
use crate::screens::Screen;
use crate::screens::accounts::AccountsScreen;
use crate::screens::capsule::CapsuleScreen;
use crate::screens::cockpit::CockpitScreen;
use crate::screens::editor::EditorScreen;
use crate::screens::manager::ManagerScreen;
use crate::screens::prelude::PreludeScreen;
use crate::screens::settings::SettingsScreen;
use crate::screens::usage::UsageScreen;


pub const TAB_MANAGER: usize = 0;

pub const TAB_ACCOUNTS: usize = 1;

pub const TAB_USAGE: usize = 2;


pub enum Page {
    Manager(Box<ManagerScreen>),
    Accounts(Box<AccountsScreen>),
    Usage(Box<UsageScreen>),
    Settings(Box<SettingsScreen>),
    Editor(Box<EditorScreen>),
    Prelude(Box<PreludeScreen>),
    Cockpit(Box<CockpitScreen>),
    Capsule(Box<CapsuleScreen>),
}

impl std::fmt::Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Page({:?})", self.route())
    }
}

impl Page {

    pub fn route(&self) -> Route {
        match self {
            Page::Manager(_) => Route::Manager,
            Page::Accounts(_) => Route::Accounts,
            Page::Usage(_) => Route::Usage,
            Page::Settings(_) => Route::Settings,
            Page::Editor(_) => Route::Editor,
            Page::Prelude(_) => Route::Prelude,
            Page::Cockpit(_) => Route::Cockpit,
            Page::Capsule(_) => Route::Capsule,
        }
    }

    pub fn screen(&self) -> &dyn Screen {
        match self {
            Page::Manager(s) => s.as_ref(),
            Page::Accounts(s) => s.as_ref(),
            Page::Usage(s) => s.as_ref(),
            Page::Settings(s) => s.as_ref(),
            Page::Editor(s) => s.as_ref(),
            Page::Prelude(s) => s.as_ref(),
            Page::Cockpit(s) => s.as_ref(),
            Page::Capsule(s) => s.as_ref(),
        }
    }

    pub fn screen_mut(&mut self) -> &mut dyn Screen {
        match self {
            Page::Manager(s) => s.as_mut(),
            Page::Accounts(s) => s.as_mut(),
            Page::Usage(s) => s.as_mut(),
            Page::Settings(s) => s.as_mut(),
            Page::Editor(s) => s.as_mut(),
            Page::Prelude(s) => s.as_mut(),
            Page::Cockpit(s) => s.as_mut(),
            Page::Capsule(s) => s.as_mut(),
        }
    }
}


pub fn router() -> Router<Page> {
    Router::with_tabs(vec![
        Page::Manager(Box::default()),
        Page::Accounts(Box::default()),
        Page::Usage(Box::default()),
    ])
}



pub trait Pages {

    fn screen(&self, route: Route) -> Option<&dyn Screen>;

    fn screen_mut(&mut self, route: Route) -> Option<&mut dyn Screen>;

    fn routes(&self) -> Vec<Route>;
    fn manager(&self) -> &ManagerScreen;
    fn manager_mut(&mut self) -> &mut ManagerScreen;
    #[cfg(test)]
    fn accounts(&self) -> &AccountsScreen;
    fn accounts_mut(&mut self) -> &mut AccountsScreen;
    #[cfg(test)]
    fn usage(&self) -> &UsageScreen;
    fn usage_mut(&mut self) -> &mut UsageScreen;
    #[cfg(test)]
    fn settings(&self) -> Option<&SettingsScreen>;
    #[cfg(test)]
    fn editor(&self) -> Option<&EditorScreen>;
    #[cfg(test)]
    fn cockpit(&self) -> Option<&CockpitScreen>;
    fn cockpit_mut(&mut self) -> Option<&mut CockpitScreen>;
    #[cfg(test)]
    fn editor_mut(&mut self) -> Option<&mut EditorScreen>;
    fn capsule(&self) -> Option<&CapsuleScreen>;
    fn capsule_mut(&mut self) -> Option<&mut CapsuleScreen>;
}

macro_rules! find {
    ($pages:expr, $v:ident) => {
        $pages.find_map(|p| match p {
            Page::$v(s) => Some(&mut **s),
            _ => None,
        })
    };
    (ref $pages:expr, $v:ident) => {
        $pages.find_map(|p| match p {
            Page::$v(s) => Some(&**s),
            _ => None,
        })
    };
}

impl Pages for Router<Page> {
    fn screen(&self, route: Route) -> Option<&dyn Screen> {
        self.pages().find(|p| p.route() == route).map(Page::screen)
    }
    fn screen_mut(&mut self, route: Route) -> Option<&mut dyn Screen> {
        self.pages_mut()
            .find(|p| p.route() == route)
            .map(Page::screen_mut)
    }
    fn routes(&self) -> Vec<Route> {
        self.pages().map(Page::route).collect()
    }
    fn manager(&self) -> &ManagerScreen {
        find!(ref self.pages(), Manager).expect("the manager is the root of its tab")
    }
    fn manager_mut(&mut self) -> &mut ManagerScreen {
        find!(self.pages_mut(), Manager).expect("the manager is the root of its tab")
    }
    #[cfg(test)]
    fn accounts(&self) -> &AccountsScreen {
        find!(ref self.pages(), Accounts).expect("accounts is the root of its tab")
    }
    fn accounts_mut(&mut self) -> &mut AccountsScreen {
        find!(self.pages_mut(), Accounts).expect("accounts is the root of its tab")
    }
    #[cfg(test)]
    fn usage(&self) -> &UsageScreen {
        find!(ref self.pages(), Usage).expect("usage is the root of its tab")
    }
    fn usage_mut(&mut self) -> &mut UsageScreen {
        find!(self.pages_mut(), Usage).expect("usage is the root of its tab")
    }
    #[cfg(test)]
    fn settings(&self) -> Option<&SettingsScreen> {
        find!(ref self.pages(), Settings)
    }
    #[cfg(test)]
    fn editor(&self) -> Option<&EditorScreen> {
        find!(ref self.pages(), Editor)
    }
    #[cfg(test)]
    fn cockpit(&self) -> Option<&CockpitScreen> {
        find!(ref self.pages(), Cockpit)
    }
    #[cfg(test)]
    fn editor_mut(&mut self) -> Option<&mut EditorScreen> {
        find!(self.pages_mut(), Editor)
    }
    fn cockpit_mut(&mut self) -> Option<&mut CockpitScreen> {
        find!(self.pages_mut(), Cockpit)
    }
    fn capsule(&self) -> Option<&CapsuleScreen> {
        find!(ref self.pages(), Capsule)
    }
    fn capsule_mut(&mut self) -> Option<&mut CapsuleScreen> {
        find!(self.pages_mut(), Capsule)
    }
}
