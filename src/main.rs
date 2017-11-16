extern crate rusty_pin as rustypin;
extern crate alfred;

use std::io;

fn main() {
    alfred::json::write_items(
        io::stdout(),
        &[
            alfred::Item::new("Item 1"),
            alfred::ItemBuilder::new("Item 2")
                .subtitle("Subtitle")
                .into_item(),
            alfred::ItemBuilder::new("Item 3")
                .arg("Argument")
                .subtitle("Subtitle")
                .icon_filetype("public.folder")
                .into_item(),
        ],
    );
}
