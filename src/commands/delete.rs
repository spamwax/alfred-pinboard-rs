use super::browser_info;
use super::*;
use alfred::ItemBuilder;
use std::io::Write;

/// Providing this command with a URL will try to remove the related bookmark from Pinboard.
/// If no URL is provided, this command will fetch browser's tab info and show and Alfred item that
/// can be used for deletion in next step.
pub fn run(cmd: SubCommand, config: Config, pinboard: Pinboard) {
    debug!("Starting in run");
    let url = match cmd {
        SubCommand::Delete { url } => url,
        _ => unreachable!(),
    };

    if let Some(url) = url {
        if let Err(e) = pinboard.delete(&url) {
            let _ = io::stdout()
                .write(format!("Error: {}", e).as_ref())
                .expect("Couldn't write to stdout");
            process::exit(1);
        } else {
            let _ = io::stdout()
                .write(b"Successfully deleted bookmark.")
                .expect("Couldn't write to stdout");
            if config.auto_update_cache {
                update::run(config, pinboard);
            }
        }
    } else {
        let tab_info;
        let item = match browser_info::get() {
            Ok(browser_tab_info) => {
                tab_info = browser_tab_info;
                ItemBuilder::new(tab_info.title.as_ref())
                    .subtitle(tab_info.url.as_ref())
                    .arg(tab_info.url.as_ref())
                    .quicklook_url(tab_info.url.as_ref())
                    .text_large_type(tab_info.title.as_ref())
                    .text_copy(tab_info.url.as_ref())
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
        ::write_to_alfred(vec![item], &config).expect("Couldn't write to Alfred");
    }
}
