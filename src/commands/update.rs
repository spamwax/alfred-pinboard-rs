use super::{io, Runner};
use chrono::prelude::*;
use std::io::Write;

#[allow(clippy::option_map_unit_fn)]
impl Runner<'_, '_> {
    pub fn update_cache(&mut self, force: bool) {
        match self.update(force) {
            Ok(s) => io::stdout()
                .write_all(s.as_bytes())
                .expect("Couldn't write to stdout"),
            Err(e) => {
                let msg = ["Error: ", e.to_string().as_str()].concat();
                io::stdout()
                    .write_all(msg.as_bytes())
                    .expect("Couldn't write to stdout");
            }
        }
    }

    fn update(&mut self, force: bool) -> Result<String, Box<dyn std::error::Error>> {
        info!("Starting in update_cache");
        match self
            .pinboard
            .as_ref()
            .unwrap()
            .is_cache_outdated(self.config.as_ref().unwrap().update_time)
        {
            Err(e) => {
                error!("{}", e.to_string());
                Err(crate::AlfredError::CacheUpdateFailed(
                    "comparing timestamp with server failed".to_string(),
                )
                .into())
            }
            Ok(needs_update) => {
                if needs_update || force {
                    debug!("  cache neeeds updating.");
                    self.pinboard
                        .as_mut()
                        .unwrap()
                        .update_cache()
                        .map_err(|e| {
                            error!("{}", e.to_string());
                            crate::AlfredError::CacheUpdateFailed("update_cache failed".to_string())
                        })?;
                    self.config
                        .as_mut()
                        .map(|config| config.update_time = Utc::now());
                    self.config.as_mut().unwrap().save().map_err(|e| {
                        error!("{}", e.to_string());
                        crate::AlfredError::CacheUpdateFailed(
                            "saving update timestamp failed".to_string(),
                        )
                    })?;
                    Ok("Updated cache files!".to_string())
                } else {
                    debug!("  cache is up-to-date.");
                    Ok("Cache is already up-to-date!".to_string())
                }
            }
        }
    }
}
