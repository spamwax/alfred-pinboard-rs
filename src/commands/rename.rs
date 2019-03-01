use super::*;
use std::io::Write;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn rename(&self, cmd: &SubCommand) {
        match cmd {
            SubCommand::Rename { tags } => println!("{:?}", tags),
            _ => unreachable!(),
        }
    }
}
