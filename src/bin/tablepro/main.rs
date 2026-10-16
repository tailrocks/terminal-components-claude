//! TablePro, terminal edition: the core database workbench built on the
//! Junie-inspired design system.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_colour;
#[cfg(test)]
mod app_tests_context;
#[cfg(test)]
mod app_tests_copy;
#[cfg(test)]
mod app_tests_coverage;
#[cfg(test)]
mod app_tests_fields;
#[cfg(test)]
mod app_tests_mouse;
#[cfg(test)]
mod app_tests_splits;
mod connections;
mod context;
mod db;
mod model;
#[cfg(test)]
mod motion_tests;
#[cfg(test)]
mod perf_tests;
mod sql;
mod tabs;
mod workbench;

use clap::{CommandFactory, Parser, error::ErrorKind};
use junie_tui::cli::CommonArgs;

use crate::app::App;
use crate::db::Connection;

#[derive(Debug, Parser)]
#[command(
    name = "tablepro",
    about = "TablePro terminal database workbench",
    long_about = "TablePro — terminal database workbench on the Junie design system"
)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    /// Connect to the named connection at start
    #[arg(long, value_name = "NAME")]
    connect: Option<String>,
}

fn cli_after_help() -> String {
    "Motion:    --motion paused pins tick-derived surfaces at --frame N for capture\n\n\
     Keys: Ctrl+O open quickly · Ctrl+T new query · Ctrl+R run · Ctrl+Y history · ? help · q quit"
        .to_string()
}

fn connection_index(connections: &[Connection], name: &str) -> Result<usize, clap::Error> {
    connections
        .iter()
        .position(|c| c.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            Cli::command().error(
                ErrorKind::ValueValidation,
                format!("no connection named {name:?}"),
            )
        })
}

fn main() -> std::io::Result<()> {
    let cli: Cli = junie_tui::cli::parse(cli_after_help());
    let mut app = App::new(cli.common.theme());
    if let Some(name) = cli.connect {
        let i = connection_index(&app.connections.connections, &name)
            .unwrap_or_else(|error| error.exit());
        app.connect(i);
    }
    app.set_motion(cli.common.motion.unwrap_or_default(), cli.common.frame);
    junie_tui::runtime::run(&mut app)
}

junie_tui::impl_application!(App);
