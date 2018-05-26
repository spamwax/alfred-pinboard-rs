use super::*;
use std::io::Write;

use super::browser_info;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn post(&mut self, cmd: SubCommand) {
        debug!("Starting in run");
        let input_tags: Vec<String>;
        let input_desc;
        // let conifg = self.config.as_ref().unwrap();
        // let pinboard = self.pinboard.as_ref().unwrap();
        match cmd {
            SubCommand::Post {
                tags,
                description,
                shared,
                toread,
            } => {
                input_tags = tags;
                input_desc = description;
                toread.map(|f| self.config.as_mut().map(|config| config.toread_new_pin = f));
                toread.map(|f| {
                    self.config
                        .as_mut()
                        .map(|config| config.private_new_pin = f)
                });
            }
            _ => unreachable!(),
        }

        let browser_tab_info = browser_info::get().unwrap_or_else(|e| {
            let _ = io::stdout()
                .write(format!("Error: {}", e).as_ref())
                .expect("Couldn't write to stdout");
            process::exit(1);
        });

        let mut pin_builder =
            PinBuilder::new(&browser_tab_info.url, browser_tab_info.title.clone());
        pin_builder = pin_builder
            .tags(input_tags.join(" "))
            .shared(if self.config.as_ref().unwrap().private_new_pin {
                "no"
            } else {
                "yes"
            })
            .toread(if self.config.as_ref().unwrap().toread_new_pin {
                "yes"
            } else {
                "no"
            });

        if let Some(desc) = input_desc {
            pin_builder = pin_builder.description(desc);
        }

        if let Err(e) = self
            .pinboard
            .as_mut()
            .unwrap()
            .add_pin(pin_builder.into_pin())
        {
            if let Err(io_err) = io::stdout().write(format!("Error: {}", e).as_ref()) {
                error!(
                    "Failed to post to Pinboard AND to notify user: {}",
                    io_err.to_string()
                );
            }
        } else {
            if let Err(io_err) = io::stdout()
                .write(format!("Successfully posted: {}\n", browser_tab_info.title).as_ref())
            {
                error!(
                    "Failed to notify user about posting to Pinboard successfully: {}",
                    io_err.to_string()
                );
            }
            if self.config.as_ref().unwrap().auto_update_cache {
                // update::run(config, pinboard);
            }
        }
    }
}
