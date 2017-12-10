use std::fs::File;
use std::io;
use std::{process, env};
use std::path::{Path, PathBuf};
use alfred;
use std::io::{Read, Write};

use commands::{Opt, SubCommand};
use config::Config;

use rusty_pin::pinboard::Pinboard;

pub fn update_cache() {
    let (config, pinboard) = Config::setup().unwrap_or_else(|err| {
        show_error_alfred(&err);
        process::exit(1);
    });

    pinboard.update_cache().unwrap_or_else(|err| {
        show_error_alfred(&err);
    });
    io::stdout().write(b"Successfully update cache files!").unwrap();
}

pub fn config(x: SubCommand) {
    let mut print_config = false;
    let mut config: Config = Config::read().unwrap_or_else(|err| {
        if !err.contains("authorization token") {
            show_error_alfred(&err);
            process::exit(1);
        }
        match &x {
            &SubCommand::Config { auth_token: Some(_), .. } => {
                let mut config = Config::new();
                match &x {
                    &SubCommand::Config {
                        ref display,
                        ref auth_token,
                        ref number_pins,
                        ref number_tags,
                        ref shared,
                        ref toread,
                        ref fuzzy,
                        ref tags_only,
                        ref auto_update,
                        ref suggest_tags,
                    } => {
                        print_config = *display;
                        config.auth_token = auth_token.as_ref().unwrap().clone();
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
            print_config = display;
            auth_token.map(|val| config.auth_token = val);
            number_pins.map(|val| config.pins_to_show = val);
            number_tags.map(|val| config.tags_to_show = val);
            shared.map(|val| config.private_new_pin = !val);
            toread.map(|val| config.toread_new_pin = val);
            fuzzy.map(|val| config.fuzzy_search = val);
            tags_only.map(|val| config.tag_only_search = val);
            auto_update.map(|val| config.auto_update_cache = val);
            suggest_tags.map(|val| config.suggest_tags = val);
        }
        _ => unreachable!(),
    }

    config.save().unwrap();

    if print_config {
        println!("{:?}", config);
    }

}

fn show_error_alfred(s: &str) {
    let item = alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_file("error.png")
        .into_item();
    alfred::json::write_items(io::stdout(), &[item]).unwrap();
}
