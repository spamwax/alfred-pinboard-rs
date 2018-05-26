use alfred;
use std::io;
use std::{env, process};

use cli::SubCommand;
use workflow_config::Config;

use rusty_pin::{PinBuilder, Pinboard, Tag};

pub mod config;
// pub mod config;
mod delete;
mod list;
mod post;
mod search;
mod update;

mod browser_info;

use alfred_rs::updater::GithubReleaser;
use alfred_rs::updater::Updater;

pub(super) struct Runner<'api, 'pin> {
    pub config: Option<Config>,
    pub pinboard: Option<Pinboard<'api, 'pin>>,
    pub updater: Option<Updater<GithubReleaser>>,
}
