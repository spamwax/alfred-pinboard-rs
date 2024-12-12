use super::{io, PinBuilder, Runner, SubCommand};
use crate::AlfredError;
use std::io::Write;

use super::browser_info;

impl Runner<'_, '_> {
    pub fn post(&mut self, cmd: SubCommand) {
        match self.perform_post(cmd) {
            Ok(s) => {
                io::stdout()
                    .write_all(s.as_bytes())
                    .expect("Couldn't write to stdout");
                if self.config.as_ref().unwrap().auto_update_cache {
                    self.update_cache(false);
                }
            }
            Err(e) => {
                let msg = ["Error: ", e.to_string().as_str()].concat();
                io::stdout()
                    .write_all(msg.as_bytes())
                    .expect("Couldn't write to stdout");
            }
        }
    }

    fn perform_post(&mut self, cmd: SubCommand) -> Result<String, Box<dyn std::error::Error>> {
        debug!("Starting in perform_post");
        let input_tags: Vec<String>;
        let input_desc;
        let toread;
        let shared;
        match cmd {
            SubCommand::Post {
                tags,
                description,
                shared: shared_flag,
                toread: toread_flag,
            } => {
                debug!(
                    "tags: {:?}, description: {:?}, toread_flag: {:?}, shared_flag: {:?}",
                    tags, description, toread_flag, shared_flag
                );
                input_tags = tags;
                input_desc = description;
                toread =
                    toread_flag.unwrap_or_else(|| self.config.as_ref().unwrap().toread_new_pin);
                shared =
                    shared_flag.unwrap_or_else(|| !self.config.as_ref().unwrap().private_new_pin);
            }
            _ => unreachable!(),
        }

        let browser_tab_info = browser_info::get().map_err(|e| {
            error!("{}", e.to_string());
            AlfredError::Post2PinboardFailed("cannot get browser's info".to_string())
        })?;

        let mut pin_builder = PinBuilder::new(&browser_tab_info.url, &browser_tab_info.title);
        pin_builder = pin_builder
            .tags(input_tags.join(" "))
            .shared(if shared { "yes" } else { "no" })
            .toread(if toread { "yes" } else { "no" });

        if let Some(desc) = input_desc {
            pin_builder = pin_builder.description(desc);
        }

        self.pinboard
            .as_mut()
            .unwrap()
            .add_pin(pin_builder.into_pin())
            .map_err(|e| {
                error!("{}", e.to_string());
                AlfredError::Post2PinboardFailed("Could not post to Pinboard.".to_string())
            })?;
        Ok(format!("Successfully posted: {}\n", browser_tab_info.title))
    }
}
