use super::*;

pub fn run(cmd: SubCommand) {
    let config = Config::setup().unwrap_or_else(|err| {
        show_error_alfred(&err);
        process::exit(1);
    });

    match cmd {
        SubCommand::List {bookmarks, tags} => process(config, bookmarks, tags),
        _ => unreachable!(),
    }
//    pinboard.update_cache().unwrap_or_else(|err| {
//        show_error_alfred(&err);
//    });
//    io::stdout().write(b"Successfully listed all shit!").unwrap();
}


fn process(config: Config, bookmarks: bool, tags: bool) {
    let mut pinboard = Pinboard::new(config.auth_token.as_ref()).unwrap_or_else(|err| {
        show_error_alfred(&err);
        process::exit(1);
    });

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
