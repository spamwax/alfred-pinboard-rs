#![feature(attr_literals)]
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate alfred;
extern crate rusty_pin;

use std::io;
use std::env;
use std::process;
use std::borrow::Cow;

use failure::Error;
use structopt::StructOpt;
use rusty_pin::Pinboard;

mod workflow_config;
mod cli;
mod commands;

use cli::{Opt, SubCommand};
use workflow_config::Config;

use commands::{config, delete, list, post, search, update};

#[derive(Debug, Fail)]
pub enum AlfredError {
    #[fail(display = "Config file may be corrupted")]
    ConfigFileErr,
    #[fail(display = "Missing config file (did you set API token?)")]
    MissingConfigFile,
    #[fail(display = "What did you do?")]
    Other,
}

// TODO: Use 'semver' crate to compare Alfred's version
// TODO: Improve performance, maybe use toml format for saving config. Look into how manytimes when
// read cache files when initiating the binary.
fn main() {
    env_logger::init();

    info!("Parsing input arguments.");
    let opt: Opt = Opt::from_args();
    //    println!("{:?}\n", opt);

    info!("Deciding on which command branch");
    match opt.cmd {
        SubCommand::Config { .. } => config::run(opt.cmd),
        _ => {
            // If user is not configuring, we will abort upon any errors.
            let (config, mut pinboard) = setup().unwrap_or_else(|err| {
                show_error_alfred(err.to_string());
                process::exit(1);
            });
            match opt.cmd {
                SubCommand::Update => update::run(config, pinboard),
                SubCommand::List { .. } => list::run(opt.cmd, config, pinboard),
                SubCommand::Search { .. } => search::run(opt.cmd, config, pinboard),
                SubCommand::Post { .. } => post::run(opt.cmd, config, pinboard),
                SubCommand::Delete { .. } => delete::run(opt.cmd, config, pinboard),
                _ => unimplemented!(),
            }
        }
    }
}

fn setup<'a>() -> Result<(Config, Pinboard<'a>), Error> {
    info!("Starting in setup");
    let config = Config::setup()?;
    let mut pinboard = Pinboard::new(config.auth_token.clone(), alfred::env::workflow_cache())?;
    pinboard.enable_fuzzy_search(config.fuzzy_search);
    pinboard.enable_tag_only_search(config.tag_only_search);
    pinboard.enable_private_new_pin(config.private_new_pin);
    pinboard.enable_toread_new_pin(config.toread_new_pin);

    Ok((config, pinboard))
}

fn show_error_alfred<'a, T: Into<Cow<'a, str>>>(s: T) {
    info!("Starting in show_error_alfred");
    let item = alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_path("erroricon.icns")
        .into_item();
    alfred::json::write_items(io::stdout(), &[item]).expect("Can't write to stdout");
}

fn write_to_alfred<'a, I>(items: I, config: Config) -> Result<(), String>
where
    I: IntoIterator<Item = alfred::Item<'a>>,
{
    info!("Starting in write_to_alfred");
    let output_items = items.into_iter().collect::<Vec<alfred::Item>>();

    let exec_counter = env::var("apr_execution_counter").unwrap_or("1".to_string());

    // Depending on alfred version use either json or xml output.
    if config.is_alfred_v3() {
        alfred::json::Builder::with_items(output_items.as_slice())
            .variable("apr_execution_counter", exec_counter.as_str())
            .write(io::stdout())
            .map_err(|e| e.to_string())
    } else {
        alfred::xml::write_items(io::stdout(), &output_items).map_err(|e| e.to_string())
    }
}
