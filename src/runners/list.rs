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
        let mut alfred_items = vec![];
        // Search the tags using the last 'word' in 'q'
        let queries = q.unwrap_or(String::new());
        let query_words: Vec<&str> = queries.split_whitespace().collect();

        match pinboard.search_list_of_tags(query_words.last().unwrap_or(&String::new().as_str())) {
            Err(e) => ::show_error_alfred(&e),
            Ok(results) => {
                alfred_items = match results {
                    None => vec![
                        ItemBuilder::new("No bookmarks found!")
                            .icon_path("no_result.icns")
                            .into_item(),
                    ],
                    Some(items) => {
                        let mut prev_tags: &str = "";
                        if query_words.len() > 1 {
                            prev_tags = queries.get(0..queries.rfind(' ').unwrap() + 1).unwrap()
                        }
                        items
                            .into_iter()
                            .map(|tag| {
                                ItemBuilder::new(tag.0.as_ref())
                                    .subtitle(tag.1.to_string())
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
            eprintln!("Ignoring search query, will spit out all bookmakrs.")
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
