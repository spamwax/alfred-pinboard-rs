use super::*;

pub fn run<'a>(cmd: SubCommand, config: Config, pinboard: Pinboard<'a>) {
    match cmd {
        SubCommand::List {bookmarks, tags} => process(config, pinboard, bookmarks, tags),
        _ => unreachable!(),
    }
}


fn process<'a>(config: Config, pinboard: Pinboard<'a>, bookmarks: bool, tags: bool) {
    let mut items: Vec<alfred::Item> = Vec::new();

    items = pinboard.list_bookmarks()
        .unwrap_or(vec![])
        .into_iter()
        .take(config.pins_to_show as usize)
        .map(|pin| {
            ItemBuilder::new(pin.title.as_ref())
                .arg(pin.url.as_ref())
                .into_item()
        }).collect::<Vec<Item>>();

    alfred::json::write_items(io::stdout(), items.as_ref());

}

use alfred::{ItemBuilder, Item};
pub struct MyItem<'a>(Item<'a>);
use std::iter::FromIterator;

//impl<'a> FromIterator<Pin> for MyItem<'a> {
//    fn from_iter(p: Pin) -> Self {
//        MyItem(alfred::ItemBuilder::new(p.title)
//            .subtitle(p.url.as_ref())
//            .into_item())
//    }
//}
