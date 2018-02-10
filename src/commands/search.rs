use super::*;
use alfred::{Item, ItemBuilder, Modifier};

use rusty_pin::pinboard::SearchType;

// TODO: Investigate why content of text_copy is not used within Alfred when user presses ⌘-C
pub fn run(cmd: SubCommand, config: Config, pinboard: Pinboard) {
    info!("Starting in search::run");
    match cmd {
        SubCommand::Search {
            tags,
            title,
            description,
            url,
            query,
        } => {
            let mut search_fields = vec![];
            if tags {
                search_fields.push(SearchType::TagOnly);
            }
            if title {
                search_fields.push(SearchType::TitleOnly);
            }
            if url {
                search_fields.push(SearchType::UrlOnly);
            }
            if description {
                search_fields.push(SearchType::DescriptionOnly);
            }
            // If user is not asking explicitly for search fields, then search based on
            // configuration set by user
            if search_fields.is_empty() {
                if config.tag_only_search {
                    search_fields.push(SearchType::TagOnly);
                } else {
                    search_fields = vec![
                        SearchType::TagOnly,
                        SearchType::TitleOnly,
                        SearchType::DescriptionOnly,
                    ];
                }
            }

            process(query, &search_fields, config.pins_to_show, pinboard);
        }
        _ => unreachable!(),
    }
}

fn process(query: Vec<String>, search_fields: &[SearchType], pins_to_show: u8, pinboard: Pinboard) {
    info!("Starting in search::process");
    match pinboard.search(&query, search_fields) {
        Err(e) => ::show_error_alfred(&e),
        Ok(r) => {
            let alfred_items = match r {
                // No result was found.
                None => vec![
                    ItemBuilder::new("No bookmarks found!")
                        .icon_path("no_result.png")
                        .into_item(),
                ],
                // Some results were found
                Some(pins) => pins.iter()
                    // Only take pins_to_show of them to show
                    .take(pins_to_show as usize)
                    // Create Alfred items that support:
                    // - quicklook
                    // - opening bookmark in a browser
                    // - showing large text
                    // - holding modifiers to
                    //   show extended text, tags or open the link in https://pinboard.in
                    .map(|pin| {
                        let _none: Option<String> = None;
                        ItemBuilder::new(pin.title.as_ref())
                            .subtitle(pin.url.as_ref())
                            .arg(pin.url.as_ref())
                            .subtitle_mod(Modifier::Command, pin.tags.as_ref())
                            .quicklook_url(pin.url.as_ref())
                            .text_large_type(pin.title.as_ref())
                            .text_copy(pin.url.as_ref())
                            .icon_path("bookmarks.png")
                            // Hold Control: Show extended description of bookmark.
                            .modifier(Modifier::Control,
                                      pin.extended.clone(), _none, true, None)
                            // Hold Option: Pressing Enter opens the bookmark on Pinboard
                            .modifier(
                                Modifier::Option,
                                // subtitle
                                Some("Show bookmark in https://pinboard.in"),
                                // Only show alphanumeric/ascii characters as searching on
                                // Pinboard's website currently doesn't like extra stuff.
                                // arg
                                Some(
                                    pin.title
                                        .split_whitespace()
                                        .filter(|s| s.len() != 1)
                                        .map(|s| {
                                            s.chars()
                                                .map(|c: char| match c {
                                                    ch if ch.is_alphanumeric() || ch.is_ascii() => {
                                                        ch
                                                    }
                                                    _ => ' ',
                                                })
                                                .collect::<String>()
                                        })
                                        .collect::<Vec<String>>()
                                        .join(" "),
                                ),
                                true,
                                None,
                            )
                            .into_item()
                    })
                    .collect::<Vec<Item>>(),
            };
            alfred::json::write_items(io::stdout(), alfred_items.as_ref()).expect("Couldn't write to stdout");
        }
    }
}
