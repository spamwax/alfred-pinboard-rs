use super::*;
use chrono::prelude::*;
use std::io::Write;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn update_cache(&mut self) {
        info!("Starting in run");
        match self
            .pinboard
            .as_ref()
            .unwrap()
            .is_cache_outdated(self.config.as_ref().unwrap().update_time)
        {
            Err(err) => {
                let _ = io::stdout()
                    .write(format!("Error: {}", err).as_ref())
                    .expect("Couldn't write to stdout");
                process::exit(1);
            }
            Ok(needs_update) => {
                if needs_update {
                    debug!("  cache neeeds updating.");
                    self.pinboard
                        .as_mut()
                        .unwrap()
                        .update_cache()
                        .unwrap_or_else(|err| {
                            let _ = io::stdout()
                                .write(format!("Error: {}", err).as_ref())
                                .expect("Couldn't write to stdout");
                            process::exit(1);
                        });
                    self.config
                        .as_mut()
                        .map(|config| config.update_time = Utc::now());
                    if self.config.as_mut().unwrap().save().is_err() {
                        let _ = io::stdout()
                            .write(b"Error: Couldn't save update time to workflow's config file!")
                            .expect("Couldn't write to stdout");
                    }
                    let _ = io::stdout()
                        .write(b"Updated cache files!")
                        .expect("Couldn't write to stdout");
                } else {
                    debug!("  cache is up-to-date.");
                    let _ = io::stdout()
                        .write(b"Cache is already up-to-date!")
                        .expect("Couldn't write to stdout");
                }
            }
        }
    }
}
