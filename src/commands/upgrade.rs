use super::*;
use std::io::Write;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn upgrade(&self, cmd: &SubCommand) {
        debug!("Starting in upgrade");
        match *cmd {
            SubCommand::SelfUpdate { check, download } => {
                if check && download {
                    eprintln!("Cannont check & download at the same time!");
                    process::exit(1);
                }
                let json_format = self.config.as_ref().unwrap().can_use_json();
                if check {
                    let none: Option<Vec<(&str, &str)>> = None;
                    if let Ok(item) = self.get_upgrade_item() {
                        let upgrade_item = if let Some(item) = item {
                            item
                        } else {
                            alfred::ItemBuilder::new("You have the latest version of workflow!")
                                .icon_path("auto_update.png")
                                .variable("workflow_update_ready", "0")
                                .arg("update")
                                .into_item()
                        };
                        crate::write_to_alfred(vec![upgrade_item], json_format, none);
                    } else {
                        let item =
                            alfred::ItemBuilder::new("Error in getting upgrade info!").into_item();
                        crate::write_to_alfred(vec![item], json_format, none);
                    }
                } else if download {
                    let filename = self.updater.as_ref().unwrap().download_latest();
                    if let Ok(filename) = filename {
                        if let Some(p) = filename.to_str() {
                            io::stdout()
                                .write_all(["Download Successful: ", p].concat().as_bytes())
                                .expect("Couldn't write to output!");
                        } else {
                            io::stdout()
                                .write_all(b"Download OK, issue with its file name!")
                                .expect("Couldn't write to output!");
                        }
                    } else {
                        let _ = io::stdout()
                            .write_all(b"Error: Couldn't download the latest workflow.");
                        debug!("Download error: {:?}", filename.unwrap_err());
                        process::exit(1);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
