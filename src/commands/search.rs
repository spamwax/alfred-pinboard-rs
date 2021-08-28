use super::*;
use alfred::{Item, ItemBuilder, Modifier};

use rusty_pin::pinboard::SearchType;

use std::borrow::Cow;
use std::io::Write;

// TODO: Investigate why content of text_copy is not used within Alfred when user presses ⌘-C
impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn search(&mut self, cmd: SubCommand) {
        debug!("Starting in search::run");
        match cmd {
            SubCommand::Search {
                tags,
                title,
                description,
                url,
                showonlyurl,
                query,
            } => {
                info!("query: {:?}", query);
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
                    if self.config.as_ref().unwrap().tag_only_search {
                        search_fields.push(SearchType::TagOnly);
                    } else {
                        search_fields = vec![
                            SearchType::TagOnly,
                            SearchType::TitleOnly,
                            SearchType::DescriptionOnly,
                            SearchType::UrlOnly,
                        ];
                    }
                }
                debug!("search fields: {:?}", search_fields);
                let pins_to_show = self.config.as_ref().unwrap().pins_to_show;
                let url_vs_tags = self.config.as_ref().unwrap().show_url_vs_tags;
                let pinboard = self.pinboard.as_ref().unwrap();

                let user_query = query.join(" ");
                let variables = vec![("user_query", user_query.as_str())];

                let items = process(query, &search_fields, pins_to_show, url_vs_tags, pinboard);
                if showonlyurl {
                    for item in items {
                        io::stdout()
                            .write_all(
                                item.quicklook_url
                                    .unwrap_or_else(|| Cow::from(""))
                                    .as_bytes(),
                            )
                            .expect("Couldn't write to stdout");
                        io::stdout()
                            .write_all(b"\n")
                            .expect("Couldn't write to stdout");
                    }
                } else if let Err(e) = self.write_output_items(items, Some(variables)) {
                    error!("search: Couldn't write to Alfred: {:?}", e);
                }
            }
            _ => unreachable!(),
        }
    }
}

// TODO: Write this function using From<Iterator> trait. <11-02-18, Hamid> //
fn process<'a>(
    query: Vec<String>,
    search_fields: &[SearchType],
    pins_to_show: u8,
    url_vs_tags: bool,
    pinboard: &'a Pinboard<'a, 'a>,
) -> Vec<Item<'a>> {
    debug!("Starting in search::process");
    match pinboard.search(&query, search_fields) {
        Err(e) => vec![crate::alfred_error_item(e.to_string())],
        Ok(r) => {
            match r {
                // No result was found.
                None => vec![ItemBuilder::new("No bookmarks found!")
                    .icon_path("no_result.png")
                    .into_item()],
                // Some results were found
                Some(pins) => pins
                    .iter()
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
                        let (subtitle, modifier_subtitle) = if !url_vs_tags {
                            (pin.url.as_ref(), pin.tags.as_ref())
                        } else {
                            (pin.tags.as_ref(), pin.url.as_ref())
                        };
                        ItemBuilder::new(pin.title.as_ref())
                            .subtitle(subtitle)
                            .arg(pin.url.as_ref())
                            .variable("tags", pin.tags.as_ref())
                            .subtitle_mod(Modifier::Command, modifier_subtitle)
                            .quicklook_url(pin.url.as_ref())
                            .text_large_type(pin.title.as_ref())
                            .text_copy(pin.url.as_ref())
                            .icon_path("bookmarks.png")
                            // Hold Control: Show extended description of bookmark.
                            .modifier(
                                Modifier::Control,
                                pin.extended.clone(),
                                Some(pin.url.as_ref()),
                                true,
                                None,
                            )
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
                                Some(pin.url.clone()),
                                // Some(
                                //     [
                                //         pin.tags.as_ref(),
                                //         " ",
                                //         pin.title
                                //             .split_whitespace()
                                //             .filter(|s| s.len() != 1)
                                //             .filter(|s| s.chars().all(char::is_alphanumeric))
                                //             .collect::<Vec<&str>>()
                                //             .join(" ")
                                //             .as_str(),
                                //     ]
                                //     .concat()
                                //     .trim()
                                //     .to_string(),
                                // ),
                                true,
                                None,
                            )
                            .into_item()
                    })
                    .collect::<Vec<Item>>(),
            }
        }
    }
}
