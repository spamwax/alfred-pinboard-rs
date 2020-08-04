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
    fn write_output_items<'a, 'b, I, J>(&self, items: I, vars: Option<J>) -> Result<(), Error>
    where
        I: IntoIterator<Item = alfred::Item<'a>>,
        J: IntoIterator<Item = (&'b str, &'b str)>,
    {
        debug!("Starting in write_output_items");
        let mut output_items = items.into_iter().collect::<Vec<alfred::Item>>();

        let update_item = self.get_upgrade_item();
        if let Ok(item) = update_item {
            output_items.push(item);
        } else {
            error!(
                "Error checking for workflow updates: {:?}",
                update_item.unwrap_err()
            );
        }

        let json_format = self.config.as_ref().unwrap().can_use_json();
        // Get username from auth_token
        let idx = self
            .config
            .as_ref()
            .unwrap()
            .auth_token
            .find(':')
            .ok_or_else(|| failure::err_msg("Bad Auth. Token!"))?;
        let username = &self.config.as_ref().unwrap().auth_token.as_str()[..idx];
        let mut variables = vec![("username", username)];
        if let Some(items) = vars {
            items.into_iter().for_each(|(k, v)| {
                variables.push((k, v));
            });
        }
        crate::write_to_alfred(output_items, json_format, Some(variables));
        Ok(())
    }

    fn get_upgrade_item(&self) -> Result<alfred::Item, Error> {
        debug!("Starting in get_upgrade_item");
        self.updater.as_ref().unwrap().update_ready().map(|update| {
            if update {
                info!("Update is available");
                // Since an update is available `latest_version().unwrap()` will not fail
                let new_version = self
                    .updater
                    .as_ref()
                    .unwrap()
                    .latest_avail_version()
                    .unwrap();
                let old_version = self.updater.as_ref().unwrap().current_version();
                alfred::ItemBuilder::new("New Version Is Available for Rusty Pin Workflow! 🎉")
                    .subtitle(format!(
                        "Click to download & upgrade {} ⟶ {}",
                        old_version, new_version
                    ))
                    .icon_path("auto_update.png")
                    .variable("workflow_update_ready", "1")
                    .arg("update")
                    .into_item()
            } else {
                info!("Update *UNAVAILABLE*");
                alfred::ItemBuilder::new("You have the latest version of workflow!")
                    .icon_path("auto_update.png")
                    .variable("workflow_update_ready", "0")
                    .arg("update")
                    .into_item()
            }
        })
    }
}
