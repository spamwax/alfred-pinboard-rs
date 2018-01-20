use super::*;
use chrono::prelude::*;

pub fn run(mut config: Config, mut pinboard: Pinboard) {
    match pinboard.is_cache_outdated(config.update_time) {
        Err(err) => ::show_error_alfred(&err),
        Ok(flag) => {
            if flag {
                pinboard.update_cache().unwrap_or_else(|err| {
                    ::show_error_alfred(&err);
                });
                config.update_time = Utc::now();
                if let Err(_) = config.save() {
                    ::show_error_alfred("Couldn't save to workflow's config file!");
                }
                io::stdout().write(b"Updated cache files!").unwrap();
            } else {
                io::stdout().write(b"Cache is already up-to-date!").unwrap();
            }
        }
    }
}
