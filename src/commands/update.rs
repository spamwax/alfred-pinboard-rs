use super::*;
use chrono::prelude::*;
use std::io::Write;

pub fn run<'a>(mut config: Config, mut pinboard: Pinboard<'a, 'a>) {
    info!("Starting in run");
    match pinboard.is_cache_outdated(config.update_time) {
        Err(err) => {
            let _ = io::stdout()
                .write(format!("Error: {}", err).as_ref())
                .expect("Couldn't write to stdout");
            process::exit(1);
        }
        Ok(needs_update) => {
            if needs_update {
                info!("cache is outdated");
                pinboard.update_cache().unwrap_or_else(|err| {
                    let _ = io::stdout()
                        .write(format!("Error: {}", err).as_ref())
                        .expect("Couldn't write to stdout");
                    process::exit(1);
                });
                config.update_time = Utc::now();
                if config.save().is_err() {
                    let _ = io::stdout()
                        .write(b"Error: Couldn't save update time to workflow's config file!")
                        .expect("Couldn't write to stdout");
                }
                let _ = io::stdout()
                    .write(b"Updated cache files!")
                    .expect("Couldn't write to stdout");
            } else {
                info!("cache is up-to-date");
                let _ = io::stdout()
                    .write(b"Cache is already up-to-date!")
                    .expect("Couldn't write to stdout");
            }
        }
    }
}
