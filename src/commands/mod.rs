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

use alfred_rs::updater::GithubReleaser;
use alfred_rs::updater::Updater;

pub(super) struct Runner<'api, 'pin> {
    pub config: Option<Config>,
    pub pinboard: Option<Pinboard<'api, 'pin>>,
    pub updater: Option<Updater<GithubReleaser>>,
}

impl<'api, 'pin> Runner<'api, 'pin> {
    fn write_output_items<'a, 'b, I, J>(
        &self,
        items: I,
        vars: Option<J>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = alfred::Item<'a>>,
        J: IntoIterator<Item = (&'b str, &'b str)>,
    {
        debug!("Starting in write_output_items");
        let mut output_items = items.into_iter().collect::<Vec<alfred::Item>>();

        match self.get_upgrade_item() {
            Ok(Some(item)) => output_items.push(item),
            Ok(None) => {}
            Err(e) => error!("Error checking for workflow updates: {:?}", e),
        }

        let json_format = self.config.as_ref().unwrap().can_use_json();
        // Get username from auth_token
        let idx = self
            .config
            .as_ref()
            .unwrap()
            .auth_token
            .find(':')
            .ok_or_else(|| "Bad Auth. Token!".to_string())?;
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

    fn get_upgrade_item(&self) -> Result<Option<alfred::Item>, Box<dyn std::error::Error>> {
        debug!("Starting in get_upgrade_item");
        self.updater
            .as_ref()
            .unwrap()
            .update_ready()
            .map(|update| {
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
                    Some(
                        alfred::ItemBuilder::new(
                            "New Version Is Available for Rusty Pin Workflow! 🎉",
                        )
                        .subtitle(format!(
                            "Click to download & upgrade {old_version} ⟶ {new_version}"
                        ))
                        .icon_path("auto_update.png")
                        .variable("workflow_update_ready", "1")
                        .arg("update")
                        .into_item(),
                    )
                } else {
                    info!("Update UNAVAILABLE: You have the latest version of workflow!");
                    None
                }
            })
            .map_err(Into::into)
    }
}
