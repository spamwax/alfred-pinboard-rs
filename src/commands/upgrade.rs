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
                let is_alfred_v3 = self.config.as_ref().unwrap().is_alfred_v3();
                if check {
                    if let Ok(item) = self.get_upgrade_item() {
                        crate::write_to_alfred(vec![item], is_alfred_v3);
                    } else {
                        let item =
                            alfred::ItemBuilder::new("Error in getting upgrade info!").into_item();
                        crate::write_to_alfred(vec![item], is_alfred_v3);
                    }
                } else if download {
                    let filename = self.updater.as_ref().unwrap().download_latest();
                    if let Ok(filename) = filename {
                        if let Some(p) = filename.to_str() {
                            let _ = io::stdout()
                                .write(format!("Download successful: {}", p).as_bytes());
                        } else {
                            let _ = io::stdout().write(b"Download OK, issue with its file name!");
                        }
                    } else {
                        let _ =
                            io::stdout().write(b"Error: Couldn't download the latest workflow.");
                        process::exit(1);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
