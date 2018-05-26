use super::browser_info;
use super::*;
use alfred::ItemBuilder;
use std::io::Write;

/// Providing this command with a URL will try to remove the related bookmark from Pinboard.
/// If no URL is provided, this command will fetch browser's tab info and show and Alfred item that
/// can be used for deletion in next step.
///

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn delete(&mut self, cmd: SubCommand) {
        debug!("Starting in run");
        let url = match cmd {
            SubCommand::Delete { url } => url,
            _ => unreachable!(),
        };

        if let Some(url) = url {
            if let Err(e) = self.pinboard.as_ref().unwrap().delete(&url) {
                let _ = io::stdout()
                    .write(format!("Error: {}", e).as_ref())
                    .expect("Couldn't write to stdout");
                process::exit(1);
            } else {
                let _ = io::stdout()
                    .write(b"Successfully deleted bookmark.")
                    .expect("Couldn't write to stdout");
                if self.config.as_ref().unwrap().auto_update_cache {
                    let config = self.config.take().unwrap();
                    let pinboard = self.pinboard.take().unwrap();
                    self.update_cache();
                    // update::run(config, pinboard);
                }
            }
        } else {
            let tab_info;
            let item = match browser_info::get() {
                Ok(browser_tab_info) => {
                    tab_info = browser_tab_info;
                    ItemBuilder::new(tab_info.title.as_str())
                        .subtitle(tab_info.url.as_str())
                        .arg(tab_info.url.as_str())
                        .quicklook_url(tab_info.url.as_str())
                        .text_large_type(tab_info.title.as_str())
                        .text_copy(tab_info.url.as_str())
                        .icon_path("bookmark-delete.png")
                        .into_item()
                }
                Err(e) => {
                    warn!("Couldn't get browser info: {:?}", e);
                    ItemBuilder::new("Couldn't get browser's info!")
                        .subtitle("Error")
                        .icon_path("erroricon.icns")
                        .into_item()
                }
            };
            ::write_to_alfred(vec![item], self.config.as_ref().unwrap())
                .expect("Couldn't write to Alfred");
        }
    }
}
