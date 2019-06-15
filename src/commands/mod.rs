use alfred;
use std::io;
use std::{env, process};

use crate::cli::SubCommand;
use crate::workflow_config::Config;

use rusty_pin::{PinBuilder, Pinboard, Tag};

pub mod config;
mod delete;
mod list;
mod post;
mod rename;
mod search;
mod update;
mod upgrade;

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

        let update_item = self.get_upgrade_item();
        if let Ok(item) = update_item {
            output_items.push(item);
        }

        let json_format = self.config.as_ref().unwrap().can_use_json();
        crate::write_to_alfred(output_items, json_format);
        Ok(())
    }

    fn get_upgrade_item(&self) -> Result<alfred::Item, Error> {
        debug!("Starting in get_upgrade_item");
        let r = self.updater.as_ref().unwrap().update_ready();

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
                    Ok(alfred::ItemBuilder::new(
                        "New Version Is Available for Rusty Pin Workflow! 🎉",
                    )
                    .subtitle(format!(
                        "Click to download & upgrade {} ⟶ {}",
                        old_version, new_version
                    ))
                    .icon_path("auto_update.png")
                    .variable("workflow_update_ready", "1")
                    .arg("update")
                    .into_item())
                } else {
                    info!("Update *UNAVAILABLE*\n{:#?}", r);
                    Ok(
                        alfred::ItemBuilder::new("You have the latest version of workflow!")
                            .icon_path("auto_update.png")
                            .variable("workflow_update_ready", "0")
                            .arg("update")
                            .into_item(),
                    )
                }
            }
            Err(e) => {
                error!("Error: {}", e.to_string());
                Err(e)
            }
        }
    }
}
