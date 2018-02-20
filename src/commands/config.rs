use super::*;
use AlfredError;

pub fn run(x: SubCommand) {
    debug!("Starting in run");
    let mut print_config = false;
    let mut config: Config = Config::setup().unwrap_or_else(|err| {
        if_chain!{
            if let Some(t) = err.cause().downcast_ref::<AlfredError>();
            if let AlfredError::MissingConfigFile = *t;
            if let SubCommand::Config { auth_token: Some(_), .. } = x;
            then {
                let mut config = Config::new();
                if let SubCommand::Config {
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
                } = x
                {
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
                }
                config
            } else {
                ::show_error_alfred(err.to_string());
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

    if let Err(e) = config.save() {
        error!("Couldn't save config file: {:?}", e);
    };

    if print_config {
        show_config(&config);
    }
}
pub fn run1(x: SubCommand) {
    debug!("Starting in run");
    let mut print_config = false;
    let mut config: Config = Config::setup().unwrap_or_else(|err| {
        // Check if error in setting up Config is related to missing file.
        // If the file missing see if user is trying to run 'config' command to do
        // initial setup. If user is trying to do that, create a new config, return it
        // and conintue with rest of 'config' command.
        // If user is not doing initial config setup, show appropriate error
        // If Config set up error is not related to missign file (maybe file is there
        // but corrupted) show appropriate message and exit.
        if let Some(t) = err.cause().downcast_ref::<AlfredError>() {
            match *t {
                AlfredError::MissingConfigFile => match x {
                    // Is user setting up auth_token?
                    SubCommand::Config {
                        auth_token: Some(_),
                        ..
                    } => {
                        let mut config = Config::new();
                        if let SubCommand::Config {
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
                        } = x
                        {
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
                        }
                        config
                    }
                    // Not setting up auth_token, show error & exit.
                    _ => {
                        ::show_error_alfred(err.to_string());
                        process::exit(1);
                    }
                },
                _ => {
                    ::show_error_alfred(err.to_string());
                    process::exit(1);
                }
            }
        } else {
            // Other general error, just exit
            ::show_error_alfred(err.to_string());
            process::exit(1);
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
            ItemBuilder::new(format!("{:?}", config.update_time))
                .subtitle("Latest cache update")
                .icon_path("auto_update.png")
                .into_item(),
        ]).write(io::stdout())
            .expect("Couldn't build alfred items");
    }
}
