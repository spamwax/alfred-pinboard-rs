use super::*;
use alfred::{Item, ItemBuilder};

// TODO: implement me!
pub fn run<'a>(cmd: SubCommand, config: Config, pinboard: Pinboard<'a>) {
    match cmd {
        SubCommand::List { tags, query } => process(config, pinboard, tags, query),
        _ => unreachable!(),
    }
}

fn process<'a>(config: Config, pinboard: Pinboard<'a>, tags: bool, q: Option<String>) {
    if tags {
        let mut popular_tags = vec![];
        let mut alfred_items = vec![];

        // First try to get list of popular tags from Pinboard
        if config.suggest_tags {
            if let Ok(tab_info) = browser_info::get() {
                let tags = match pinboard.popular_tags(&tab_info.url) {
                    Err(e) => vec![String::from("ERROR: fetching popular tags!")],
                    Ok(tags) => tags,
                };
                popular_tags = tags.into_iter().map(|t| Tag(t, 0)).collect::<Vec<Tag>>();
            }
        }

        // Search the tags using the last 'word' in 'q'
        let queries = q.unwrap_or(String::new());
        let query_words: Vec<&str> = queries.split_whitespace().collect();

        match pinboard.search_list_of_tags(query_words.last().unwrap_or(&String::new().as_str())) {
            Err(e) => ::show_error_alfred(&e),
            Ok(results) => {
                alfred_items = match results {
                    None => {
                        assert!(!query_words.is_empty());
                        let last_query_word = query_words.last().unwrap().to_string();
                        vec![
                            ItemBuilder::new(last_query_word.clone())
                                .subtitle("NEW TAG")
                                .autocomplete(last_query_word)
                                .icon_path("tag.png")
                                .into_item(),
                        ]
                    }
                    Some(items) => {
                        let mut prev_tags: &str = "";
                        if query_words.len() > 1 {
                            // User has already searched for other tags, we should include those in the
                            // 'autocomplete' field of the AlfredItem
                            prev_tags = queries.get(0..queries.rfind(' ').unwrap() + 1).unwrap()
                        }
                        popular_tags
                            .iter()
                            .chain(items.into_iter().take(config.tags_to_show as usize))
                            .map(|tag| {
                                ItemBuilder::new(tag.0.as_ref())
                                    .subtitle(if tag.1 != 0 {
                                        tag.1.to_string()
                                    } else {
                                        String::from("Popular")
                                    })
                                    .autocomplete([prev_tags, &tag.0].concat())
                                    .arg(String::from(prev_tags) + &tag.0)
                                    .into_item()
                            })
                            .collect::<Vec<Item>>()
                    }
                };
            }
        }
        ::write_to_alfred(alfred_items, config);
    } else {
        if q.is_some() && !q.unwrap().is_empty() {
            eprintln!("Ignoring search query, will spit out all bookmarks.")
        }
        let items = pinboard
            .list_bookmarks()
            .unwrap_or(vec![])
            .into_iter()
            .take(config.pins_to_show as usize)
            .map(|pin| {
                ItemBuilder::new(pin.title.as_ref())
                    .subtitle(pin.url.as_ref())
                    .arg(pin.url.as_ref())
                    .into_item()
            });
        ::write_to_alfred(items, config);
    }
}

pub struct MyItem<'a>(Item<'a>);
use std::iter::FromIterator;

//impl<'a> FromIterator<Pin> for MyItem<'a> {
//    fn from_iter(p: Pin) -> Self {
//        MyItem(alfred::ItemBuilder::new(p.title)
//            .subtitle(p.url.as_ref())
//            .into_item())
//    }
//}
