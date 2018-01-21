use super::*;
use chrono::prelude::*;

pub fn run(mut config: Config, mut pinboard: Pinboard) {
    match pinboard.is_cache_outdated(config.update_time) {
        Err(err) => {
            io::stdout().write(format!("Error: {}", err).as_ref());
            process::exit(1);
        },
        Ok(needs_update) => {
            if needs_update {
                pinboard.update_cache().unwrap_or_else(|err| {
                    io::stdout().write(format!("Error: {}", err).as_ref());
                    process::exit(1);
                });
                config.update_time = Utc::now();
                if let Err(_) = config.save() {
                    io::stdout().write(b"Error: Couldn't save update time to workflow's config file!");
                }
                io::stdout().write(b"Updated cache files!").unwrap();
            } else {
                io::stdout().write(b"Cache is already up-to-date!").unwrap();
            }
        }
    }
}
