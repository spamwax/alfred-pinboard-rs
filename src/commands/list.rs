use super::*;
use std::io::Write;
use std::{thread, time};

use failure::Error;
use alfred::{Item, ItemBuilder};

pub fn run<'a>(cmd: SubCommand, config: &Config, pinboard: &Pinboard<'a>) {
    match cmd {
        SubCommand::List { tags, query } => process(config, pinboard, tags, query),
        _ => unreachable!(),
    }
}

fn process<'a>(config: &Config, pinboard: &Pinboard<'a>, tags: bool, q: Option<String>) {
    debug!("Starting in list::process");
    if tags {
        // Search the tags using the last 'word' in 'q'
        let queries = q.unwrap_or_else(String::new);

        // Check if user has entered ';' which indicates they are providing a description.
        // So no need to search for tags!
        if queries.contains(';') {
            let pin_info = queries
                .splitn(2, ';')
                .map(|s| s.trim())
                .collect::<Vec<&str>>();
            let item = ItemBuilder::new("Hit Return to bookmark the page!")
                .icon_path("upload.png")
                .arg(queries.as_ref())
                .variable("tags", pin_info[0])
                .variable("description", pin_info[1])
                .into_item();
            ::write_to_alfred(vec![item], config).expect("Couldn't write to Alfred");
            return;
        }

        let query_words: Vec<&str> = queries.split_whitespace().collect();

        let mut popular_tags = vec![];
        let mut alfred_items = vec![];

        let exec_counter;
        // First try to get list of popular tags from Pinboard
        // TODO: Run popular_tag fetching in a different thread <21-02-18, Hamid> //
        if config.suggest_tags {
            exec_counter = env::var("apr_execution_counter")
                .unwrap_or_else(|_| "1".to_string())
                .parse::<usize>()
                .unwrap_or(1);

            use std::sync::mpsc;
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
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
            thread::sleep(time::Duration::from_millis(800));
            warn!("outside: waited 200");
            if let Ok(pt) = rx.try_recv() {
                    warn!("received popular tags from child: {:?}", pt);
                    popular_tags = pt;
            }
        }

        match pinboard.search_list_of_tags(query_words.last().unwrap_or(&String::new().as_str())) {
            Err(e) => ::show_error_alfred(e.to_string()),
            Ok(results) => {
                alfred_items = match results {
                    None => {
                        assert!(!query_words.is_empty());
                        let last_query_word = *query_words.last().unwrap();
                        vec![
                            ItemBuilder::new(last_query_word)
                                .subtitle("NEW TAG")
                                .autocomplete(last_query_word)
                                .icon_path("tag.png")
                                .into_item(),
                        ]
                    }
                    Some(items) => {
                        debug!("Found {} tags.", items.len());
                        let prev_tags = if query_words.len() > 1 {
                            // User has already searched for other tags, we should include those in the
                            // 'autocomplete' field of the AlfredItem
                            queries.get(0..queries.rfind(' ').unwrap() + 1).unwrap()
                        } else {
                            ""
                        };
                        let prev_tags_len = prev_tags.len();
                        // Show the tag with highest frequency matching the last query before popular/suggested tags.
                        popular_tags.insert(0, items[0].clone());
                        popular_tags
                            .iter()
                            // Combine popular tags and returned tags from cache
                            .chain(items.into_iter().take(config.tags_to_show as usize))
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
                                .subtitle(if tag.1 != 0 {
                                    tag.1.to_string()
                                } else {
                                    String::from("Popular")
                                })
                                .autocomplete(_args.clone())
                                .variable("tags", _args.clone())
                                .arg(_args)
                                .valid(true)
                                .icon_path("tag.png")
                                .into_item()
                        })
                        .collect::<Vec<Item>>()
                    }
                };
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

/// Retrieves popular tags from a Web API call for first run and caches them for subsequent runs.
fn retrieve_popular_tags(exec_counter: usize) -> Result<Vec<Tag>, Error> {
    debug!("Starting in get_suggested_tags");
    use std::fs;
    use std::io::{BufRead, BufReader, BufWriter};

    // let wait_time = time::Duration::from_millis(300);
    // thread::sleep(wait_time);
    // return Ok(vec![Tag("Hamid".to_string(), 42)]);
    // // return Err("fake error".to_string());

    // FIXME: If run from outside Alfred (say terminal), the cache folder for 'config' and 'pinboard' will be different.
    let config = Config::setup()?;
    let pinboard = Pinboard::new(config.auth_token.clone(), alfred::env::workflow_cache())?;

    let ptags_fn = config.cache_dir().join("popular.tags.cache");
    let mut popular_tags = vec![];

    if exec_counter == 1 {
        warn!("Retrieving popular tags.");
        if let Ok(tab_info) = browser_info::get() {
            warn!("tab_info.url: {:?}", tab_info.url);
            let tags = match pinboard.popular_tags(&tab_info.url) {
                Err(e) => vec![format!("ERROR: fetching popular tags: {:?}", e)],
                Ok(tags) => tags,
            };
            warn!("tags: {:?}", tags);
            warn!("tags: {:?}", ptags_fn);
            fs::File::create(ptags_fn)
                .and_then(|fp| {
                    let mut writer = BufWriter::with_capacity(1024, fp);
                    writer.write_all(tags.join("\n").as_bytes())
                })?;
            popular_tags = tags.into_iter().map(|t| Tag(t, 0)).collect::<Vec<Tag>>();
        }
    } else {
        warn!("reading suggested tags from cache file: {:?}", ptags_fn);
        fs::File::open(ptags_fn)
            .and_then(|fp| {
                let reader = BufReader::with_capacity(1024, fp);
                popular_tags = reader
                    .lines()
                    .map(|l| Tag(l.expect("bad popular tags cache file?"), 0))
                    .collect::<Vec<Tag>>();
                Ok(())
            })?;
    }
    Ok(popular_tags)
}
