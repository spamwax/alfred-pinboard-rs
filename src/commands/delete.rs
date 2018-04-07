use super::browser_info;
use super::*;
use alfred::ItemBuilder;
use std::io::Write;

pub fn run(cmd: SubCommand, config: &Config, pinboard: &Pinboard) {
    let _ = config; // To silent compiler.
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
                ItemBuilder::new("Couldn't get browrer info!")
                    .subtitle("Error")
                    .icon_path("erroricon.icns")
                    .into_item()
            }
        };
        ::write_to_alfred(vec![item], config).expect("Couldn't write to Alfred");
    }
}
