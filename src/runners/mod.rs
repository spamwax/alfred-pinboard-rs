use std::fs::File;
use std::io;
use std::{process, env};
use std::path::{Path, PathBuf};
use alfred;
use std::io::{Read, Write};
use semver::Version;
use semver::VersionReq;

use commands::{Opt, SubCommand};
use workflow_config::Config;

use rusty_pin::pinboard::Pinboard;

pub mod config;
pub mod update;

fn show_error_alfred(s: &str) {
    let item = alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_path("erroricon.icns")
        .into_item();
    alfred::json::write_items(io::stdout(), &[item]).unwrap();
}
