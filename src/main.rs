#![feature(attr_literals)]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate rusty_pin as rustypin;
extern crate alfred;

use std::io;
use std::path::{Path, PathBuf};
use std::env;
use std::fs::File;
use std::process;

use structopt::StructOpt;

use rustypin::pinboard;

mod config;
mod commands;

use commands::Commands;
use config::Config;


//TODO: Use 'semver' crate to compare Alfred's version
fn main() {
    let opt = Commands::from_args();
    println!("{:?}", opt);

    let mut cache_dir = alfred::env::workflow_cache().unwrap_or_else(|| {
        let mut dir = env::home_dir().unwrap_or(PathBuf::from(""));
        dir.push(".cache");
        dir.push("alfred-pinboard-rs");
        dir
    });
    let mut data_dir = alfred::env::workflow_data().unwrap_or_else(|| {
        let mut dir = env::home_dir().unwrap_or(PathBuf::from(""));
        dir.push(".config");
        dir.push("alfred-pinboard-rs");
        dir
    });
    println!("{:?}", cache_dir);
    data_dir.push("settings.json");

    let mut config = Config::new().read_from(data_dir);
    if let Err(_) = config {
        process::exit(1);
    } else {
        println!("{:?}", config);
    }
}
