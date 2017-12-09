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
use std::env;
use std::fs::File;
use std::process;

use structopt::StructOpt;

use rustypin::pinboard;

mod config;
mod commands;
mod runners;

use commands::{Opt, SubCommand};
use config::Config;


//TODO: Use 'semver' crate to compare Alfred's version
fn main() {

    let opt: Opt = Opt::from_args();
    //    println!("{:?}\n", opt);

    match opt.cmd {
        SubCommand::Config { .. } => runners::config(opt.cmd),
        _ => println!("<<>> EMPTY?!"),
    }

}
