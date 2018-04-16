use super::*;
use AlfredError;
use chrono::prelude::Local;

pub fn run(x: SubCommand) {
    debug!("Starting in run");
    let print_config;
    let mut config: Config;

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
            config = Config::setup().unwrap_or_else(|err| {
                if_chain!{
                    if auth_token.is_some();
                    if let Some(t) = err.cause().downcast_ref::<AlfredError>();
                    if let AlfredError::MissingConfigFile = *t;
                    then {
                        let mut config = Config::new();
                        config.auth_token = auth_token.as_ref().unwrap().clone();
                        config
                    } else {
                        ::show_error_alfred(err.to_string());
                        process::exit(1);
                    }
                }
            });
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

    if let Err(e) = config.save() {
        error!("Couldn't save config file: {:?}", e);
    };

    if print_config {
        show_config(&config);
    }
}

fn show_config(config: &Config) {
    debug!("Starting in show_config");
    // TODO: Add support for Alfred 2 by returning XML <09-02-18, Hamid> //
    // If Using Alfred Version >=3
    if config.is_alfred_v3() {
        use alfred::ItemBuilder;
        alfred::json::Builder::with_items(&[
            ItemBuilder::new("Only search tags")
                .subtitle(format!("{:?}", config.tag_only_search))
                .arg("pset tagonly")
                .icon_path("tagonly.png")
                .into_item(),
            ItemBuilder::new("Use fuzzy search")
                .subtitle(format!("{:?}", config.fuzzy_search))
                .arg("pset fuzzy")
                .icon_path("fuzzy.png")
                .into_item(),
            ItemBuilder::new("Automatically update cache")
                .subtitle(format!("{:?}", config.auto_update_cache))
                .arg("pset auto")
                .icon_path("auto_update_cache.png")
                .into_item(),
            ItemBuilder::new("Suggest popular tags for open browser tab")
                .subtitle(format!("{:?}", config.suggest_tags))
                .arg("pset suggest_tags")
                .icon_path("suggest.png")
                .into_item(),
            ItemBuilder::new("Mark new bookmarks as toread")
                .subtitle(format!("{:?}", config.toread_new_pin))
                .arg("pset toread")
                .icon_path("toread.png")
                .into_item(),
            ItemBuilder::new("Mark new bookmarks as private")
                .subtitle(format!("{:?}", config.private_new_pin))
                .arg("pset shared")
                .icon_path("private.png")
                .into_item(),
            ItemBuilder::new("Number of tags to show")
                .subtitle(format!("{:?}", config.tags_to_show))
                .arg("pset tags")
                .icon_path("no_of_tags.png")
                .into_item(),
            ItemBuilder::new("Number of bookmarks to show")
                .subtitle(format!("{:?}", config.pins_to_show))
                .arg("pset bookmarks")
                .icon_path("no_of_pins.png")
                .into_item(),
            ItemBuilder::new(
                config
                    .update_time
                    .with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ).subtitle("Latest cache update")
                .icon_path("auto_update.png")
                .into_item(),
        ]).write(io::stdout())
            .expect("Couldn't build alfred items");
    }
}
