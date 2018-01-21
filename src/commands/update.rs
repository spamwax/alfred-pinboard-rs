use super::*;
use chrono::prelude::*;

pub fn run(mut config: Config, mut pinboard: Pinboard) {
    match pinboard.is_cache_outdated(config.update_time) {
        Err(err) => {
            io::stdout().write(format!("Error: {}", err).as_ref());
            process::exit(1);
        }
        Ok(needs_update) => {
            if needs_update {
                pinboard.update_cache().unwrap_or_else(|err| {
                    io::stdout().write(format!("Error: {}", err).as_ref());
                    process::exit(1);
                });
                config.update_time = Utc::now();
                if let Err(_) = config.save() {
                    io::stdout().write(
                        format!("Error: Couldn't save update time to workflow's config file!")
                            .as_ref(),
                    );
                }
                io::stdout()
                    .write(format!("Updated cache files!").as_ref())
                    .unwrap();
            } else {
                io::stdout()
                    .write(format!("Cache is already up-to-date!").as_ref())
                    .unwrap();
            }
        }
    }
}
