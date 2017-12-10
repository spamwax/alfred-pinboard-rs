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

mod workflow_config;
mod commands;
mod runners;

use commands::{Opt, SubCommand};
use workflow_config::Config;
use rusty_pin::Pinboard;

use runners::{config, update};

//TODO: Use 'semver' crate to compare Alfred's version
fn main() {

    let opt: Opt = Opt::from_args();
    //    println!("{:?}\n", opt);

    match opt.cmd {
        SubCommand::Config { .. } =>  config::run(opt.cmd),
        SubCommand::Update => update::run(),
        _ => unimplemented!(),
    }

}
