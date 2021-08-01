/// Providing this command with a URL will try to remove the related bookmark from Pinboard.
/// If no URL is provided, this command will fetch browser's tab info and show and Alfred item that
/// can be used for deletion in next step.
///
use super::browser_info;
use super::*;
use crate::AlfredError;
use alfred::ItemBuilder;
use std::io::Write;

// TODO: right now we accept deleting tag & url at the same time. If user asks to delete a tag
// only, this function will automatically grab browser's url and return an Alfred item containing
// it while deleting the given tag as well. I believe these two options should be made exclusively
// mutual.
impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn delete(&mut self, cmd: SubCommand) {
        debug!("Starting in delete");
        let (url, tag) = match cmd {
            SubCommand::Delete { url, tag } => (url, tag),
            _ => unreachable!(),
        };
        match self.perform_delete(url, tag) {
            Ok(s) if !s.is_empty() => {
                io::stdout()
                    .write_all(s.as_bytes())
                    .expect("Couldn't write to stdout");
                if self.config.as_ref().unwrap().auto_update_cache {
                    self.update_cache();
                }
            }
            Err(e) => {
                let msg = ["Error: ", e.to_string().as_str()].concat();
                io::stdout()
                    .write_all(msg.as_bytes())
                    .expect("Couldn't write to stdout");
            }
            Ok(_) => (),
        }
    }

    fn perform_delete(
        &mut self,
        url: Option<String>,
        tag: Option<String>,
    ) -> Result<String, Error> {
        if let Some(tag) = tag {
            debug!("  tag: {}", tag);
            self.pinboard
                .as_ref()
                .unwrap()
                .delete_tag(tag)
                .map_err(|e| {
                    error!("{}", e.to_string());
                    AlfredError::DeleteFailed("couldn't delete tag".to_string())
                })?;
            return Ok("Successfully deleted tag.".to_string());
        }
        if let Some(url) = url {
            debug!("  url: {}", url);
            self.pinboard.as_ref().unwrap().delete(&url).map_err(|e| {
                error!("{}", e.to_string());
                AlfredError::DeleteFailed("couldn't delete url".to_string())
            })?;
            Ok("Successfully deleted bookmark.".to_string())
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
                    error!("Couldn't get browser info: {:?}", e);
                    ItemBuilder::new("Couldn't get browser's info!")
                        .subtitle("Error")
                        .icon_path("erroricon.icns")
                        .into_item()
                }
            };
            if let Err(e) = self.write_output_items(vec![item], Option::<Vec<(&str, &str)>>::None) {
                error!("delete: Couldn't write to Alfred: {:?}", e);
            }
            Ok(String::new())
        }
    }
}
