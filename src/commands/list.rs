use super::*;
use std::io::Write;
use std::{thread, time};

use alfred::{Item, ItemBuilder};
use failure::Error;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn list(&self, cmd: SubCommand) {
        match cmd {
            SubCommand::List {
                tags,
                suggest,
                query,
            } => process(
                self.config.as_ref().unwrap(),
                self.pinboard.as_ref().unwrap(),
                tags,
                suggest,
                query,
            ),
            _ => unreachable!(),
        }
    }
}

fn process<'a>(
    config: &Config,
    pinboard: &Pinboard<'a, 'a>,
    tags: bool,
    suggest: Option<bool>,
    q: Option<String>,
) {
    debug!("Starting in list::process");
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
                .arg(queries.as_ref())
                .variable("tags", pin_info[0]);
            if !pin_info[1].is_empty() {
                item = item.variable("description", pin_info[1])
            }
            let item = item.into_item();
            ::write_to_alfred(vec![item], config).expect("Couldn't write to Alfred");
            return;
        }

        let query_words: Vec<&str> = queries.split_whitespace().collect();

        let last_query_word_tag;
        let mut popular_tags;
        let mut alfred_items = vec![];

        // First try to get list of popular tags from Pinboard
        popular_tags = if suggest.unwrap_or(config.suggest_tags) {
            suggest_tags()
        } else {
            vec![]
        };

        let last_query_word = query_words.last().unwrap_or(&"");

        match pinboard.search_list_of_tags(last_query_word) {
            Err(e) => ::show_error_alfred(e.to_string()),
            Ok(results) => {
                let prev_tags = if query_words.len() > 1 {
                    // User has already searched for other tags, we should include those in the
                    // 'autocomplete' field of the AlfredItem
                    queries.get(0..queries.rfind(' ').unwrap() + 1).unwrap()
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
                        last_query_word_tag = Tag::new(last_query_word.to_string(), 0).set_new();
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
                            // Remove tags that user has aleady selected
                            .filter(|tag| {
                                if !query_words.is_empty() {
                                    let upper = query_words.len() - 1;
                                    !query_words
                                        .as_slice()[0..upper]
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
                            ItemBuilder::new(tag.0.as_ref())
                                .subtitle(tag.1.to_string())
                                .autocomplete(_args.clone())
                                .variable("tags", _args.clone())
                                .arg(_args)
                                .valid(true)
                                .icon_path("tag.png")
                                .into_item()
                        })
                        .collect::<Vec<Item>>();
            }
        }
        ::write_to_alfred(alfred_items, config).expect("Couldn't write to Alfred");
    } else {
        if q.is_some() && !q.unwrap().is_empty() {
            warn!(
                "Ignoring search query, will spit out {} of bookmarks.",
                config.pins_to_show
            )
        }
        let items = pinboard
            .list_bookmarks()
            .unwrap_or_else(|| vec![])
            .into_iter()
            .take(config.pins_to_show as usize)
            .map(|pin| {
                ItemBuilder::new(pin.title.as_ref())
                    .subtitle(pin.url.as_ref())
                    .arg(pin.url.as_ref())
                    .into_item()
            });
        ::write_to_alfred(items, config).expect("Couldn't write to Alfred");
    }
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
            if tx_result.is_ok() {
                warn!("Sent the popular tags from child thread");
            } else {
                warn!("Failed to send popular tags: {:?}", tx_result.unwrap_err());
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
    use std::fs;
    use std::io::{BufRead, BufReader, BufWriter};

    // TODO: Don't create another pinboard instance. use the one list.rs receives to shared with
    // the thread that runs this function.
    // FIXME: If run from outside Alfred (say terminal),
    // the cache folder for 'config' and 'pinboard' will be different.
    let config = Config::setup()?;
    let pinboard = Pinboard::new(config.auth_token.clone(), alfred::env::workflow_cache())?;

    let ptags_fn = config.cache_dir().join("popular.tags.cache");
    let popular_tags = if exec_counter == 1 {
        let tab_info = browser_info::get()?;
        warn!("tab_info.url: {:?}", tab_info.url);
        let tags = match pinboard.popular_tags(&tab_info.url) {
            Err(e) => vec![format!("ERROR: fetching popular tags: {:?}", e)],
            Ok(tags) => tags,
        };
        warn!("tags: {:?}", ptags_fn);
        fs::File::create(ptags_fn).and_then(|fp| {
            let mut writer = BufWriter::with_capacity(1024, fp);
            writer.write_all(tags.join("\n").as_bytes())
        })?;
        tags.into_iter()
            .map(|t| Tag::new(t, 0).set_popular())
            .collect::<Vec<Tag>>()
    } else {
        warn!("reading suggested tags from cache file: {:?}", ptags_fn);
        fs::File::open(ptags_fn).and_then(|fp| {
            let reader = BufReader::with_capacity(1024, fp);
            Ok(reader
                .lines()
                .map(|l| Tag::new(l.expect("bad popular tags cache file?"), 0).set_popular())
                .collect::<Vec<Tag>>())
        })?
    };
    Ok(popular_tags)
}
