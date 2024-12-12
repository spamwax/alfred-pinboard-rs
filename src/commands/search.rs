use super::{io, Pinboard, Runner, SubCommand};
use alfred::{Item, ItemBuilder, Modifier};

use rusty_pin::pinboard::SearchType;
use std::borrow::Cow;
use std::io::Write;
use url::Url;

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
                exacttag,
                query,
            } => {
                info!("query: {:?}", query);
                let pins_to_show = self.config.as_ref().unwrap().pins_to_show;
                let url_vs_tags = self.config.as_ref().unwrap().show_url_vs_tags;
                let pinboard = self.pinboard.as_ref().unwrap();
                let user_query = query.join(" ");
                let variables = vec![("user_query", user_query.as_str())];

                let mut search_fields = vec![];
                // figure out which fields to search if user is not after an exact tag.
                if !exacttag {
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
                }

                let items = process(
                    query,
                    &search_fields,
                    exacttag,
                    pins_to_show,
                    url_vs_tags,
                    pinboard,
                );
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
#[allow(clippy::needless_pass_by_value)]
fn process<'a>(
    query: Vec<String>,
    search_fields: &[SearchType],
    exacttag: bool,
    pins_to_show: u8,
    url_vs_tags: bool,
    pinboard: &'a Pinboard<'a, 'a>,
) -> Vec<Item<'a>> {
    debug!("Starting in search::process");
    assert!(!query.is_empty());

    let r = if exacttag {
        debug!("finding all pins having exact tag: {}", query[0]);
        pinboard.find_tag(query[0].as_str())
    } else {
        debug!("searching for pins containing all of {:?}", &query);
        pinboard.search(&query, search_fields)
    };

    match r {
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
                        let (subtitle, modifier_subtitle) = if url_vs_tags {
                            (pin.tags.as_ref(), pin.url.as_ref())
                        } else {
                            (pin.url.as_ref(), pin.tags.as_ref())
                        };
                        // Convert pin's url and path segments to space delimited string so it can be
                        // used to find the pin on piaboard.in website.
                        // For example, if pin's url is https://pinboard.in/u:hamid/b:1234567890,
                        // it will be converted to "pinboard.in u:hamid b:1234567890".
                        // If pin's url is not a valid http URL, it will be converted using its scheme.
                        // For example, if pin's url is "data:text/plain,Hello?World#", it will be converted to "datd text plain,Hello".
                        // TODO: For URLs that have spaces or special characters that will be encoded to stuff like %20,
                        // query_to_be_used_on_pinboardsite text used to find the pin on pinboard.in website will likely not work!
                        // This is due to the fact that pinboard.in search is weak an cannot handle such queries.
                        // One solution will be using another URL/URI crate such as "uriparse"?
                        debug!("Building search query_to_be_used_on_pinboardsite!");
                        let _url = Url::parse(pin.url.as_ref()).unwrap();
                        let mut _host_str;
                        let _path_segments;
                        let query_to_be_used_on_pinboardsite = if !_url.cannot_be_a_base() {
                            _host_str = if _url.scheme().contains("http") {
                                vec![_url.host_str().unwrap_or_default()]
                            } else {
                                vec![_url.scheme(), _url.host_str().unwrap_or_default()]
                            };
                            _path_segments = _url
                                .path_segments()
                                .map(|path| path.collect::<Vec<_>>())
                                .unwrap_or_default();
                            _host_str.extend(_path_segments);
                            _host_str.join(" ").trim().to_owned()
                        } else {
                            _host_str = vec![_url.scheme()];
                            let _path = _url.path().to_string().replace("/", " ");
                            _path_segments = vec![&_path];
                            _host_str.extend(_path_segments);
                            _host_str.join(" ").trim().to_owned()
                        };
                        debug!(
                            "query_to_be_used_on_pinboardsite: {}",
                            &query_to_be_used_on_pinboardsite
                        );
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
                                Some("Find bookmark in https://pinboard.in"),
                                // Pinboard's website currently doesn't like extra stuff in
                                // search query's string.
                                // Workaround: Search for item's all tags plus strictly
                                // ascii words in its title.
                                // arg
                                Some(query_to_be_used_on_pinboardsite),
                                // Some(pin.url.clone()),
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
