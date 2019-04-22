use super::*;
use alfred::ItemBuilder;
use std::io::Write;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn rename(&mut self, cmd: &SubCommand) {
        match cmd {
            SubCommand::Rename { tags } => self.run(tags),
            _ => unreachable!(),
        }
    }

    // fn run(&mut self, tags: &Vec<String>) {
    fn run(&mut self, tags: &[String]) {
        debug!("running rename::run");
        if tags.len() != 2 {
            debug!("  tags: {:?}", tags);
            let item = ItemBuilder::new("pr old_tag new_tag")
                .subtitle("Rename old -> new")
                .icon_path("bookmark-delete.png")
                .into_item();
            if let Err(e) = self.write_output_items(vec![item]) {
                error!("delete: Couldn't write to Alfred: {:?}", e);
            }
            return;
        }

        debug!("  calling rename API");
        let r = self
            .pinboard
            .as_ref()
            .unwrap()
            .rename_tag(&tags[0], &tags[1]);
        debug!("  matching result: {:?}", &r);
        match r {
            Err(e) => {
                let _ = io::stdout()
                    .write(format!("Error: {}", e).as_ref())
                    .expect("Couldn't write to stdout");
                process::exit(1);
            }
            Ok(_) => {
                let _ = io::stdout()
                    .write(b"Successfully renamed tag.")
                    .expect("Couldn't write to stdout");
                if self.config.as_ref().unwrap().auto_update_cache {
                    self.update_cache();
                }
            }
        }
    }
}
