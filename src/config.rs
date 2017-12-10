use std::fs::{File, create_dir_all};
use std::path::{PathBuf, Path};
use std::io::{Read, Write};
use std::env;

use serde;
use serde_json;
use alfred;

use rusty_pin::Pinboard;

const CONFIG_FILE_NAME: &str = "settings.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Which version of Alfred we are being executed under
    pub alfred_version: String,
    /// Number of bookmarks to show in Alfred
    pub pins_to_show: u8,
    /// Number of tags to show in Alfred
    pub tags_to_show: u8,
    /// Flag to perform search only on `tag` fields of bookmarks
    pub tag_only_search: bool,
    /// Flag to perform a fuzzy search
    pub fuzzy_search: bool,
    /// Flag to save bookmarks as private
    pub private_new_pin: bool,
    /// Flag to save bookmarks as toread
    pub toread_new_pin: bool,
    /// Flag to suggest popular tags for the browser's current url
    pub suggest_tags: bool,
    /// Flag to update cache after each bookmark saving automatically.
    pub auto_update_cache: bool,
    /// Authentication Token
    pub auth_token: String,

    /// Folder to store volatile data of the workflow
    workflow_data_dir: PathBuf,
    /// Folder to store data of the workflow
    workflow_cache_dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        Config {
            alfred_version: String::new(),
            pins_to_show: 10,
            tags_to_show: 10,
            tag_only_search: false,
            fuzzy_search: false,
            private_new_pin: true,
            toread_new_pin: false,
            suggest_tags: true,
            auto_update_cache: true,
            auth_token: String::new(),
            workflow_data_dir: PathBuf::default(),
            workflow_cache_dir: PathBuf::default(),
        }
    }

    pub fn setup() -> Result<(Config, Pinboard), String> {
        let config = Config::read()?;
        let mut pinboard = Pinboard::new(&config.auth_token)?;
        pinboard.set_cache_dir(&config.workflow_cache_dir)?;
        pinboard.enable_fuzzy_search(config.fuzzy_search);
        pinboard.enable_tag_only_search(config.tag_only_search);
        Ok((config, pinboard))
    }

    pub fn read() -> Result<Config, String> {
        // If config file exists read settings
        let mut p = Config::get_workflow_dirs().0;
        p.push(CONFIG_FILE_NAME);
        if p.exists() {
            let mut config: Config = File::open(p)
                .map_err(|e| e.to_string())
                .and_then(|mut f| {
                    let mut content = String::new();
                    f.read_to_string(&mut content)
                        .map_err(|e| e.to_string())
                        .and_then(|_| Ok(content))
                })
                .and_then(|inp| {
                    serde_json::from_str(&inp).map_err(|e| {
                        format!("Bad settings file: {}\n{}", CONFIG_FILE_NAME, e.to_string())
                    })
                })?;
            config.discover_dirs();
            Ok(config)
        } else {
            Err(String::from(format!(
                "Can't find Workflow's setting file:\n{:?}\n\
                    Have you added your authorization token?",
                p
            )))
        }
    }

    pub fn save(&self) -> Result<(), String> {
        create_dir_all(&self.workflow_data_dir).map_err(|e| e.to_string())?;

        let mut settings_fn = self.workflow_data_dir.clone();
        settings_fn.push(CONFIG_FILE_NAME);
        let mut fp = File::create(settings_fn).map_err(|e| e.to_string())?;
        serde_json::to_string(self)
            .map_err(|e| e.to_string())
            .and_then(|content| {
                fp.write_all(content.as_ref()).map_err(|e| e.to_string())
            })
    }

    pub fn discover_dirs(&mut self) {
        let dirs = Config::get_workflow_dirs();
        self.workflow_data_dir = dirs.0;
        self.workflow_cache_dir = dirs.1;
    }

    fn get_workflow_dirs() -> (PathBuf, PathBuf) {
        let cache_dir = alfred::env::workflow_cache().unwrap_or_else(|| {
            let mut dir = env::home_dir().unwrap_or(PathBuf::from(""));
            dir.push(".cache");
            dir.push("alfred-pinboard-rs");
            dir
        });
        let data_dir = alfred::env::workflow_data().unwrap_or_else(|| {
            let mut dir = env::home_dir().unwrap_or(PathBuf::from(""));
            dir.push(".config");
            dir.push("alfred-pinboard-rs");
            dir
        });
        (data_dir, cache_dir)
    }
}
