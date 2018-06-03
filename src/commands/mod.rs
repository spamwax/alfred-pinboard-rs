use alfred;
use std::io;
use std::{env, process};

use cli::SubCommand;
use workflow_config::Config;

use rusty_pin::{PinBuilder, Pinboard, Tag};

pub mod config;
// pub mod config;
mod delete;
mod list;
mod post;
mod search;
mod update;

mod browser_info;

use super::Error;
use alfred_rs::updater::GithubReleaser;
use alfred_rs::updater::Updater;

pub(super) struct Runner<'api, 'pin> {
    pub config: Option<Config>,
    pub pinboard: Option<Pinboard<'api, 'pin>>,
    pub updater: Option<Updater<GithubReleaser>>,
}

impl<'api, 'pin> Runner<'api, 'pin> {
    fn write_output_items<'a, I>(&self, items: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = alfred::Item<'a>>,
    {
        debug!("Starting in write_output_items");
        let mut output_items = items.into_iter().collect::<Vec<alfred::Item>>();

        let exec_counter = env::var("apr_execution_counter").unwrap_or_else(|_| "1".to_string());

        // use std::thread;
        // thread::sleep_ms(1);
        let r = self.updater.as_ref().unwrap().update_ready(); //.expect("couldn't spawn thread");

        match r {
            Ok(update) => {
                if update {
                    info!("update is available");
                    // Since an update is available `latest_version().unwrap()` will not fail
                    let new_version = self
                        .updater
                        .as_ref()
                        .unwrap()
                        .latest_avail_version()
                        .unwrap();
                    let old_version = self.updater.as_ref().unwrap().current_version();
                    let update_item = alfred::ItemBuilder::new(
                        "🎉 New Version Is Available for Rusty Pin Workflow! 🎉",
                    ).subtitle(format!(
                        "Click to download & upgrade {} ⟶ {}",
                        old_version, new_version
                    ))
                        .icon_path("auto_update.png")
                        .variable("update_workflow", "true")
                        .valid(true)
                        .into_item();
                    // Add a new item to previous list of Items
                    output_items.push(update_item);
                // let filename = self.updater.as_ref().unwrap().download_latest();
                // info!("got downloading result");
                // let filename = filename.unwrap();
                // info!("saved file to {:#?}", filename);
                } else if !update {
                    info!("Update *UNAVAILABLE*\n{:#?}", r);
                }
            }
            Err(e) => error!("problem: {:#?}", e),
        }
        // Depending on alfred version use either json or xml output.
        if self.config.as_ref().unwrap().is_alfred_v3() {
            alfred::json::Builder::with_items(output_items.as_slice())
                .variable("apr_execution_counter", exec_counter.as_str())
                .write(io::stdout())?
        } else {
            alfred::xml::write_items(io::stdout(), &output_items)?
        }
        Ok(())
    }
}
