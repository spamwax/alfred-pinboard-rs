#![feature(attr_literals)]
extern crate serde;
extern crate serde_json;
extern crate semver;
#[macro_use]
extern crate serde_derive;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate rusty_pin;
extern crate alfred;

use std::io;
use std::env;
use std::fs::File;
use std::process;

use structopt::StructOpt;

use rusty_pin::Pinboard;

mod workflow_config;
mod commands;
mod runners;

use commands::{Opt, SubCommand};
use workflow_config::Config;

use runners::{config, update, list, search};

//TODO: Use 'semver' crate to compare Alfred's version
fn main() {

    let opt: Opt = Opt::from_args();
    //    println!("{:?}\n", opt);

    match opt.cmd {
        SubCommand::Config { .. } =>  config::run(opt.cmd),
        _ => {
            // If user is not configuring, we will abort upon any errors.
            let (config, mut pinboard) = setup().unwrap_or_else(|err| {
                show_error_alfred(&err);
                process::exit(1);
            });
            match opt.cmd {
                SubCommand::Update => update::run(pinboard),
                SubCommand::List { .. } => list::run(opt.cmd, config, pinboard),
                SubCommand::Search {..} => search::run(opt.cmd, config, pinboard),
                _ => unimplemented!(),
            }
        }
    }

}

fn setup<'a>() -> Result<(Config, Pinboard<'a>), String> {
    let config = Config::setup()?;
    let mut pinboard = Pinboard::new(config.auth_token.clone())?;
    pinboard.enable_fuzzy_search(config.fuzzy_search);
    pinboard.enable_tag_only_search(config.tag_only_search);
    pinboard.enable_private_new_pin(config.private_new_pin);
    pinboard.enable_toread_new_pin(config.toread_new_pin);

    Ok((config, pinboard))
}

fn show_error_alfred(s: &str) {
    let item = alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_path("erroricon.icns")
        .into_item();
    alfred::json::write_items(io::stdout(), &[item]).unwrap();
}
