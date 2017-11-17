use std::path::{PathBuf, Path};

use serde_json;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    //    /// Folder to store volatile data of the workflow
    //    workflow_cache_dir: PathBuf,
    //    /// Folder to store data of the workflow
    //    workflow_data_dir: PathBuf,
    /// Which version of Alfred we are being executed under
    alfred_version: String,
    /// Number of bookmarks to show in Alfred
    pins_to_show: usize,
    /// Number of tags to show in Alfred
    tags_to_show: usize,
    /// Flag to perform search only on `tag` fields of bookmarks
    tag_only_search: bool,
    /// Flag to perform a fuzzy search
    fuzzy_search: bool,
    /// Flag to save bookmarks as private
    private_new_pin: bool,
    /// Flag to save bookmarks as toread
    toread_new_pin: bool,
    /// Flag to update cache after each bookmark saving automatically.
    auto_update_cache: bool,
    /// Authentication Token
    auth_token: String,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read_from<P: AsRef<Path>>(&mut self, p: P) -> Result<Config, String> {
        Ok(Config::new())
    }
}
