use super::*;
use std::io::Write;
use std::error::Error;

use super::browser_info;

// TODO: Honor auto_update flag.

pub fn run(cmd: SubCommand, mut config: Config, pinboard: Pinboard) {
    info!("Starting in run");
    let input_tags: Vec<String>;
    let input_desc;
    match cmd {
        SubCommand::Post {
            tags,
            description,
            shared,
            toread,
        } => {
            input_tags = tags;
            input_desc = description;
            toread.map(|f| config.toread_new_pin = f);
            shared.map(|f| config.private_new_pin = !f);
        }
        _ => unreachable!(),
    }

    let browser_tab_info = browser_info::get().unwrap_or_else(|e| {
        io::stdout().write(format!("Error: {}", e).as_ref()).expect("Couldn't write to stdout");
        process::exit(1);
    });

    let mut pin_builder = PinBuilder::new(&browser_tab_info.url, browser_tab_info.title.clone());
    pin_builder = pin_builder
        .tags(input_tags.join(" "))
        .shared(if config.private_new_pin { "no" } else { "yes" })
        .toread(if config.toread_new_pin { "yes" } else { "no" });

    if let Some(desc) = input_desc {
        pin_builder = pin_builder.description(desc);
    }

    if let Err(e) = pinboard.add_pin(pin_builder.into_pin()) {
        if let Err(io_err) = io::stdout().write(format!("Error: {}", e).as_ref()) {
            eprintln!(
                "Failed to post to Pinboard AND to notify user: {}",
                io_err.description()
            );
        }
    } else {
        if let Err(io_err) = io::stdout()
            .write(format!("Successfully posted: {}\n", browser_tab_info.title).as_ref())
        {
            eprintln!(
                "Failed to notify user about posting to Pinboard successfully: {}",
                io_err.description()
            );
        }
        if config.auto_update_cache {
            update::run(config, pinboard);
        }
    }
}
