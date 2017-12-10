use std::fs::File;
use std::io;
use std::{process, env};
use std::path::{Path, PathBuf};
use alfred;

use commands::{Opt, SubCommand};
use config::Config;


pub fn config(x: SubCommand) {
    let config: Config = Config::setup().unwrap_or_else(|err| {
        if !err.contains("authorization token") {
            show_error_alfred(&err);
            process::exit(1);
        }
        match x {
            SubCommand::Config { auth_token: Some(_), .. } => {
                let mut config = Config::new();
                match x {
                    SubCommand::Config {
                        display,
                        auth_token,
                        number_pins,
                        number_tags,
                        shared,
                        toread,
                        fuzzy,
                        tags_only,
                        auto_update,
                        suggest_tags,
                    } => {
                        config.auth_token = auth_token.unwrap();
                        number_pins.map(|val| config.pins_to_show = val);
                        number_tags.map(|val| config.tags_to_show = val);
                        shared.map(|val| config.private_new_pin = !val);
                        toread.map(|val| config.toread_new_pin = val);
                        fuzzy.map(|val| config.fuzzy_search = val);
                        tags_only.map(|val| config.tag_only_search = val);
                        auto_update.map(|val| config.auto_update_cache = val);
                        suggest_tags.map(|val| config.suggest_tags = val);
                        config.discover_dirs();
                    }
                    _ => (),
                }
                config
            }
            _ => {
                show_error_alfred("First-time config command should provide authorization token!");
                process::exit(1);
            }
        }
    });

    config.save().unwrap();

}

fn show_error_alfred(s: &str) {
    let item = alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_file("error.png")
        .into_item();
    alfred::json::write_items(io::stdout(), &[item]);
}

//let config = setup().unwrap_or_else(|err| {
//eprintln!("Problem setting up: {}", err);
//process::exit(1);
//});
