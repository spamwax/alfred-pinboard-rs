use super::{browser_info, env, Config, Pinboard, Runner, SubCommand, Tag};
use crate::cli::Opt;
use std::{thread, time};

use alfred::{Item, ItemBuilder, Modifier};
use alfred_rs::Data;

impl Runner<'_, '_> {
    pub fn list(&self, opt: Opt) {
        let cmd = opt.cmd;
        let query_item = opt.query_as_item;
        let _ = opt.debug_level;
        match cmd {
            SubCommand::List {
                tags,
                suggest,
                no_existing_page,
                query,
            } => self.process(tags, suggest, no_existing_page, query_item, query),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn process(
        &self,
        tags: bool,
        suggest: Option<bool>,
        no_existing_page: bool,
        query_item: bool,
        q: Option<String>,
    ) {
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
            "Mark the bookmark as NOT toread."
        } else {
            "Mark the bookmark as TOREAD."
        };

        if tags {
            // Search the tags using the last 'word' in 'q'
            let queries = q.unwrap_or_default();

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
                    item = item.variable("description", pin_info[1]);
                }
                let item = vec![item.into_item()];
                if let Err(e) = self.write_output_items(item, Option::<Vec<(&str, &str)>>::None) {
                    error!("list: Couldn't write to Alfred: {:?}", e);
                }
                return;
            }

            let query_words: Vec<&str> = queries.split_whitespace().collect();

            let last_query_word_tag;
            let mut alfred_items = Vec::with_capacity(config.tags_to_show as usize + 1);

            // First try to get list of popular tags from Pinboard
            let tag_suggestion = suggest.unwrap_or(config.suggest_tags);
            let popular_tags = if tag_suggestion {
                // if suggest.unwrap_or(config.suggest_tags) {
                suggest_tags()
            } else {
                vec![]
            };

            let last_query_word = query_words.last().unwrap_or(&"");
            let mut top_items: Vec<Tag> = Vec::with_capacity(2usize);

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
                    last_query_word_tag = Tag::new((*last_query_word).to_string(), 0).set_new();
                    let mut filter_idx = None;
                    #[allow(clippy::single_match_else)]
                    let items = match results {
                        Some(i) => {
                            debug!("Found {} tags.", i.len());
                            top_items.insert(0, i[0].clone());
                            // If user input matches a tag, we will show it as first item
                            if let Some(idx) = i.iter().position(|t| &t.0 == last_query_word) {
                                if idx != 0 {
                                    top_items.insert(0, i[idx].clone());
                                    filter_idx = Some(idx - 1); // Substract one since we will be
                                                                // always skipping the firt tag
                                                                // from results (i)
                                    debug!("Found exact tag: {:?}, {}", i[idx], idx);
                                }
                            } else {
                                // Otherwise we will show the tag with highest frequency matching user
                                // input before popular/suggested tags, unless --query-as-item is set
                                // in which case we create an alfred item with what user has typed
                                top_items.insert(0, last_query_word_tag);
                                if query_item {
                                    filter_idx = None;
                                }
                            }
                            i
                        }
                        None => {
                            assert!(!query_words.is_empty());
                            debug!("Didn't find any tag for `{}`", last_query_word);
                            filter_idx = None;
                            top_items.insert(0, last_query_word_tag);
                            vec![]
                        }
                    };
                    let prev_tags_len = prev_tags.len();
                    alfred_items = top_items // Start with top_items, and
                        .iter()
                        .enumerate()
                        .chain(popular_tags.iter().enumerate()) // then add popular_tags
                        // and finally add all tags returned from search, ensuring to remove
                        // items that are in top_results
                        .chain(
                            items.into_iter().skip(1).enumerate().filter(|&(idx, _)| {
                                filter_idx.is_none() || idx != filter_idx.unwrap()
                            }),
                        )
                        // Remove all tags that are already present in user query list
                        .filter(|&(_, tag)| {
                            if query_words.is_empty() {
                                true
                            } else {
                                let upper = query_words.len() - 1;
                                !query_words.as_slice()[0..upper]
                                    .iter()
                                    .any(|q| q == &tag.0.as_str())
                            }
                        })
                        .take(config.tags_to_show as usize)
                        .map(|(_, tag)| {
                            let mut item_args = String::with_capacity(prev_tags_len + tag.0.len());
                            item_args.push_str(prev_tags);
                            item_args.push_str(&tag.0);
                            ItemBuilder::new(tag.0.as_str())
                                .subtitle(tag.1.to_string())
                                .autocomplete(item_args.clone())
                                .subtitle_mod(Modifier::Option, option_subtitle)
                                .subtitle_mod(Modifier::Control, control_subtitle)
                                .variable("tags", item_args.clone())
                                .variable("shared", &private_pin)
                                .variable("toread", &toread_pin)
                                .arg(item_args)
                                .valid(true)
                                .icon_path("tag.png")
                                .into_item()
                        })
                        .collect::<Vec<Item>>();
                }
            }
            debug!("alfred_items hast {} entities", alfred_items.len());
            if !query_words.is_empty() && alfred_items.is_empty() {
                alfred_items = vec![ItemBuilder::new("Hit Return to bookmark the page!")
                    .icon_path("upload.png")
                    .arg(queries.as_str())
                    .subtitle("You may have entered duplicate tags!")
                    .variable("shared", &private_pin)
                    .variable("toread", &toread_pin)
                    .variable("tags", queries.as_str())
                    .subtitle_mod(Modifier::Option, option_subtitle)
                    .subtitle_mod(Modifier::Control, control_subtitle)
                    .into_item()];
            }
            if !no_existing_page && config.page_is_bookmarked && is_page_bookmarked(pinboard) {
                let bookmark_present = ItemBuilder::new("You already have the bookmark!")
                    .icon_path("bookmark-delete.png")
                    .into_item();
                alfred_items.insert(0, bookmark_present);
            }
            if let Err(e) = self.write_output_items(alfred_items, Option::<Vec<(&str, &str)>>::None)
            {
                error!("list: Couldn't write to Alfred: {:?}", e);
            }
        } else {
            if q.is_some() && !q.unwrap().is_empty() {
                warn!(
                    "Ignoring search query, will spit out {} of bookmarks.",
                    config.pins_to_show
                );
            }
            let items = pinboard
                .list_bookmarks()
                .unwrap_or_default()
                .into_iter()
                .take(config.pins_to_show as usize)
                .map(|pin| {
                    ItemBuilder::new(pin.title.as_ref())
                        .subtitle(pin.url.as_ref())
                        .arg(pin.url.as_ref())
                        .into_item()
                })
                .collect::<Vec<Item>>();
            if let Err(e) = self.write_output_items(items, Option::<Vec<(&str, &str)>>::None) {
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
                debug!("looking for {:?}", &tab_info.url);
                pinboard
                    .find_url(&tab_info.url)
                    .map(|op| {
                        debug!("pinboard.find_url says {:?}", &op);
                        if let Some(vp) = op {
                            assert!(!vp.is_empty());
                            !vp.is_empty()
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false)
            }
            Err(_) => false,
        };
        let _r = Data::save_to_file("bookmark_exists.json", &found);
        debug!("bookmark found from browser info: {}", found);
    } else {
        found = Data::load_from_file("bookmark_exists.json").unwrap_or(false);
        debug!("bookmark found from cache info: {}", found);
    }
    found
}

fn suggest_tags() -> Vec<Tag> {
    use std::sync::mpsc;
    let mut popular_tags = vec![];
    let exec_counter = env::var("apr_execution_counter")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let (tx, rx) = mpsc::channel();
    let thread_handle = thread::spawn(move || {
        warn!("inside spawn thread, about to call get_suggested_tags");
        let r = retrieve_popular_tags(exec_counter);
        if let Ok(pt) = r {
            let tx_result = tx.send(pt);
            match tx_result {
                Ok(()) => warn!("Sent the popular tags from child thread"),
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
fn retrieve_popular_tags(exec_counter: usize) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
    debug!("Starting in get_suggested_tags");

    // TODO: Don't create another pinboard instance. use the one list.rs receives to be shared with
    // the thread that runs this function.
    // FIXME: If run from outside Alfred (say terminal),
    // the cache folder for 'config' and 'pinboard' will be different.
    let config = Config::setup()?;
    let pinboard = Pinboard::new(config.auth_token, alfred::env::workflow_cache())?;

    let ptags_fn = "popular.tags.cache";
    let tags;

    if exec_counter == 1 {
        let tab_info = browser_info::get()?;
        warn!("tab_info.url: {:?}", tab_info.url);
        tags = match pinboard.pinboard.popular_tags(&tab_info.url) {
            Err(e) => vec![format!("ERROR: fetching popular tags: {:?}", e)],
            Ok(tags) => tags,
        };
        info!("popular tags: {:?}", tags);
        Data::save_to_file(ptags_fn, &tags)?;
    } else {
        warn!(
            "**** reading suggested tags from cache file: {:?}",
            ptags_fn
        );
        tags = Data::load_from_file(ptags_fn)
            .ok_or_else(|| "bad popular tags cache file".to_string())?;
    }
    Ok(tags
        .into_iter()
        .map(|t| Tag::new(t, 0).set_popular())
        .collect::<Vec<Tag>>())
}
