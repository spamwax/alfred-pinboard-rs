use super::*;
use alfred::{Item, ItemBuilder, Modifier};

use rusty_pin::pinboard::SearchType;

// TODO: Investigate why content of text_copy is not used within Alfred when user presses ⌘-C
pub fn run<'a, 'b>(
    cmd: SubCommand,
    config: &Config,
    pinboard: &'a Pinboard,
) -> impl IntoIterator<Item = Item<'a>> {
    debug!("Starting in search::run");
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
            process(query, &search_fields, config.pins_to_show, pinboard)
        }
        _ => unreachable!(),
    }
}

// TODO: Write this function using From<Iterator> trait. <11-02-18, Hamid> //
fn process<'a, 'b>(
    query: Vec<String>,
    search_fields: &[SearchType],
    pins_to_show: u8,
    pinboard: &'a Pinboard<'a, 'a>,
) -> impl IntoIterator<Item = Item<'a>> {
    debug!("Starting in search::process");
    match pinboard.search(&query, search_fields) {
        Err(e) => vec![::alfred_error(e.to_string())],
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
                            .variable("tags", pin.tags.as_ref())
                            .subtitle_mod(Modifier::Command, pin.tags.as_ref())
                            .quicklook_url(pin.url.as_ref())
                            .text_large_type(pin.title.as_ref())
                            .text_copy(pin.url.as_ref())
                            .icon_path("bookmarks.png")
                            // Hold Control: Show extended description of bookmark.
                            .modifier(Modifier::Control,
                                      pin.extended.clone(),
                                      Some(pin.url.as_ref()), true, None)
                            // Hold Option: Pressing Enter opens the bookmark on Pinboard
                            // FIXME: There should be a better way of locating an item on
                            // Pinboard's website. Pinboard, however currently doesn't
                            // provide a direct way of doing that!
                            .modifier(
                                Modifier::Option,
                                // subtitle
                                Some("Show bookmark in https://pinboard.in"),
                                // Pinboard's website currently doesn't like extra stuff in
                                // search query's string.
                                // Workaround: Search for item's all tags plus strictly
                                // ascii words in its title.
                                // arg
                                Some(
                                    [
                                    pin.tags.as_ref(),
                                    " ",
                                    pin.title
                                        .split_whitespace()
                                        .filter(|s| s.len() != 1)
                                        .filter(|s| s.chars().all(|c| c.is_alphanumeric()))
                                        .collect::<Vec<&str>>()
                                        .join(" ").as_str()
                                    ].concat()
                                ),
                                true,
                                None,
                            )
                            .into_item()
                    })
                    .collect::<Vec<Item>>(),
            };
            alfred_items
            // alfred::json::write_items(io::stdout(), alfred_items.as_ref())
            //     .expect("Couldn't write to stdout");
        }
    }
}
