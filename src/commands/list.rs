use super::*;
use std::{thread, time};

use alfred::{Item, ItemBuilder, Modifier};
use alfred_rs::Data;
use failure::Error;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn list(&self, cmd: SubCommand) {
        match cmd {
            SubCommand::List {
                tags,
                suggest,
                query,
            } => self.process(tags, suggest, query),
            _ => unreachable!(),
        }
    }

    fn process(&self, tags: bool, suggest: Option<bool>, q: Option<String>) {
        debug!("Starting in list::process");
        let config = self.config.as_ref().unwrap();
        let pinboard = self.pinboard.as_ref().unwrap();

        let private_pin = (!config.private_new_pin).to_string();
        let toread_pin = config.toread_new_pin.to_string();
        let option_subtitle = if config.private_new_pin {
            "Post the bookmark as PUBLIC."
        } else {
            "Post the bookmark as private."
        };
        let control_subtitle = if config.toread_new_pin {
            "Mark the pin as NOT toread."
        } else {
            "Mark the pin as TOREAD."
        };

        if tags {
            // Search the tags using the last 'word' in 'q'
            let queries = q.unwrap_or_else(String::new);

            // Check if user has entered ';' which indicates they are providing a description.
            // So no need to search for tags!
            if queries.contains(';') {
                let trim_chars: &[_] = &['\t', ' ', '\\', '\n'];
                let pin_info = queries
                    .splitn(2, ';')
                    .map(|s| s.trim_matches(trim_chars))
                    .collect::<Vec<&str>>();
                debug!("  pin_info: {:?}", pin_info);
                let mut item = ItemBuilder::new("Hit Return to bookmark the page!")
                    .icon_path("upload.png")
                    .arg(queries.as_str())
                    .variable("shared", private_pin)
                    .variable("toread", toread_pin)
                    .variable("tags", pin_info[0])
                    .subtitle_mod(Modifier::Option, option_subtitle)
                    .subtitle_mod(Modifier::Control, control_subtitle);
                if !pin_info[1].is_empty() {
                    item = item.variable("description", pin_info[1])
                }
                let item = vec![item.into_item()];
                if let Err(e) = self.write_output_items(item) {
                    error!("list: Couldn't write to Alfred: {:?}", e);
                }
                return;
            }

            let query_words: Vec<&str> = queries.split_whitespace().collect();

            let last_query_word_tag;
            let mut popular_tags;
            let mut alfred_items = Vec::with_capacity(config.tags_to_show as usize + 1);

            // First try to get list of popular tags from Pinboard
            let tag_suggestion = suggest.unwrap_or(config.suggest_tags);
            popular_tags = if tag_suggestion {
                // if suggest.unwrap_or(config.suggest_tags) {
                suggest_tags()
            } else {
                vec![]
            };

            let last_query_word = query_words.last().unwrap_or(&"");

            match pinboard.search_list_of_tags(last_query_word) {
                Err(e) => crate::show_error_alfred(e.to_string()),
                Ok(results) => {
                    let prev_tags = if query_words.len() > 1 {
                        // User has already searched for other tags, we should include those in the
                        // 'autocomplete' field of the AlfredItem
                        queries.get(0..=queries.rfind(' ').unwrap()).unwrap()
                    } else {
                        ""
                    };

                    // No result means, we couldn't find a tag using the given query
                    // Some result mean we found tags even though the query was empty as
                    // search_list_of_tags returns all tags for empty queries.
                    let items = match results {
                        Some(i) => {
                            debug!("Found {} tags.", i.len());
                            i
                        }
                        None => {
                            assert!(!query_words.is_empty());
                            debug!("Didn't find any tag for `{}`", last_query_word);
                            last_query_word_tag =
                                Tag::new((*last_query_word).to_string(), 0).set_new();
                            vec![&last_query_word_tag]
                        }
                    };
                    let prev_tags_len = prev_tags.len();
                    // Show the tag with highest frequency matching the last query before popular/suggested tags.
                    popular_tags.insert(0, items[0].clone());
                    alfred_items = popular_tags
                        .iter()
                        // Combine popular tags and returned tags from cache
                        .chain(items.into_iter().skip(1).take(config.tags_to_show as usize))
                        // Remove tags that user has already selected
                        .filter(|tag| {
                            if !query_words.is_empty() {
                                let upper = query_words.len() - 1;
                                !query_words.as_slice()[0..upper]
                                    .iter()
                                    .any(|q| q == &tag.0.as_str())
                            } else {
                                true
                            }
                        })
                        // transform tags to Alfred items
                        .map(|tag| {
                            let mut _args = String::with_capacity(prev_tags_len + tag.0.len());
                            _args.push_str(prev_tags);
                            _args.push_str(&tag.0);
                            ItemBuilder::new(tag.0.as_str())
                                .subtitle(tag.1.to_string())
                                .autocomplete(_args.clone())
                                .subtitle_mod(Modifier::Option, option_subtitle)
                                .subtitle_mod(Modifier::Control, control_subtitle)
                                .variable("tags", _args.clone())
                                .variable("shared", &private_pin)
                                .variable("toread", &toread_pin)
                                .arg(_args)
                                .valid(true)
                                .icon_path("tag.png")
                                .into_item()
                        })
                        .collect::<Vec<Item>>();
                }
            }
            if config.page_is_bookmarked && is_page_bookmarked(&pinboard) {
                let bookmark_present = ItemBuilder::new("You already have the bookmark!")
                    .icon_path("bookmark-delete.png")
                    .into_item();
                alfred_items.insert(0, bookmark_present);
            }
            if let Err(e) = self.write_output_items(alfred_items) {
                error!("list: Couldn't write to Alfred: {:?}", e);
            }
        } else {
            if q.is_some() && !q.unwrap().is_empty() {
                warn!(
                    "Ignoring search query, will spit out {} of bookmarks.",
                    config.pins_to_show
                )
            }
            let items = pinboard
                .list_bookmarks()
                .unwrap_or_else(Vec::new)
                .into_iter()
                .take(config.pins_to_show as usize)
                .map(|pin| {
                    ItemBuilder::new(pin.title.as_ref())
                        .subtitle(pin.url.as_ref())
                        .arg(pin.url.as_ref())
                        .into_item()
                })
                .collect::<Vec<Item>>();
            if let Err(e) = self.write_output_items(items) {
                error!("list: Couldn't write to Alfred: {:?}", e);
            }
        }
    }
}

fn is_page_bookmarked(pinboard: &Pinboard) -> bool {
    let found;

    let exec_counter = env::var("apr_execution_counter")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<usize>()
        .unwrap_or(1);
    debug!("exec_counter: {}", exec_counter);

    if exec_counter == 1 {
        let tab_info = browser_info::get();
        found = match tab_info {
            Ok(tab_info) => {
                debug!("tab_info: {:?}", tab_info);
                pinboard
                    .find_url(&tab_info.url)
                    .and_then(|op| {
                        if let Some(vp) = op {
                            assert!(!vp.is_empty());
                            Ok(!vp.is_empty())
                        } else {
                            Ok(false)
                        }
                    })
                    .unwrap_or(false)
            }
            Err(_) => false,
        };
        let _ = Data::save_to_file("bookmark_exists.json", &found);
        debug!("bookmark found from browser info: {}", found);
    } else {
        found = Data::load_from_file("bookmark_exists.json").unwrap_or(false);
        debug!("bookmark found from cache info: {}", found);
    }
    found
}

fn suggest_tags() -> Vec<Tag> {
    let mut popular_tags = vec![];
    let exec_counter = env::var("apr_execution_counter")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    let thread_handle = thread::spawn(move || {
        warn!("inside spawn thread, about to call get_suggested_tags");
        let r = retrieve_popular_tags(exec_counter);
        if let Ok(pt) = r {
            let tx_result = tx.send(pt);
            match tx_result {
                Ok(_) => warn!("Sent the popular tags from child thread"),
                Err(e) => warn!("Failed to send popular tags: {:?}", e),
            }
        } else {
            warn!("get_suggested_tags: {:?}", r);
        }
    });
    if exec_counter == 1 {
        thread::sleep(time::Duration::from_millis(1000));
    } else {
        thread_handle.join().expect("Child thread terminated");
    }

    if let Ok(pt) = rx.try_recv() {
        warn!("* received popular tags from child: {:?}", pt);
        popular_tags = pt;
    } else {
        warn!("child didn't produce usable result.");
    }

    popular_tags
}
/// Retrieves popular tags from a Web API call for first run and caches them for subsequent runs.
fn retrieve_popular_tags(exec_counter: usize) -> Result<Vec<Tag>, Error> {
    debug!("Starting in get_suggested_tags");

    // TODO: Don't create another pinboard instance. use the one list.rs receives to be shared with
    // the thread that runs this function.
    // FIXME: If run from outside Alfred (say terminal),
    // the cache folder for 'config' and 'pinboard' will be different.
    let config = Config::setup()?;
    let pinboard = Pinboard::new(config.auth_token, alfred::env::workflow_cache())?;

    // let ptags_fn = config.cache_dir().join("popular.tags.cache");
    let ptags_fn = "popular.tags.cache";
    let tags;
    // let metadata = fs::metadata("foo.txt")?;

    // if let Ok(time) = metadata.created() {
    //     println!("{:?}", time);
    // } else {
    //     println!("Not supported on this platform");
    // }
    if exec_counter == 1 {
        let tab_info = browser_info::get()?;
        warn!("tab_info.url: {:?}", tab_info.url);
        tags = match pinboard.popular_tags(&tab_info.url) {
            Err(e) => vec![format!("ERROR: fetching popular tags: {:?}", e)],
            Ok(tags) => tags,
        };
        info!("popular tags: {:?}", tags);
        Data::save_to_file(&ptags_fn, &tags)?;
    } else {
        warn!(
            "**** reading suggested tags from cache file: {:?}",
            ptags_fn
        );
        use failure::err_msg;
        tags = Data::load_from_file(ptags_fn)
            .map_or(Err(err_msg("bad popular tags cache file")), Ok)?;
    }
    Ok(tags
        .into_iter()
        .map(|t| Tag::new(t, 0).set_popular())
        .collect::<Vec<Tag>>())
}
