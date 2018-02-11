use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::io::{Read, Write};
use std::io::{BufReader, BufWriter};
use std::env;
use chrono::prelude::*;

use failure::Error;
use serde_json;
use alfred;

use semver::{Version, VersionReq};

use AlfredError;

const CONFIG_FILE_NAME: &str = "settings.json";
const FILE_BUF_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Which version of Alfred we are being executed under
    #[serde(skip, default = "get_alfred_version")]
    pub alfred_version: Version,
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
    /// Last time cache was updated
    #[serde(default = "get_epoch")]
    pub update_time: DateTime<Utc>,

    /// Folder to store volatile data of the workflow
    workflow_data_dir: PathBuf,
    /// Folder to store data of the workflow
    workflow_cache_dir: PathBuf,
}

impl<'a> Config {
    pub fn new() -> Self {
        info!("Starting in new");
        let mut cfg = Config {
            alfred_version: get_alfred_version(),
            pins_to_show: 10,
            tags_to_show: 10,
            tag_only_search: false,
            fuzzy_search: false,
            private_new_pin: true,
            toread_new_pin: false,
            suggest_tags: true,
            auto_update_cache: true,
            auth_token: String::new(),
            update_time: get_epoch(),
            workflow_data_dir: PathBuf::default(),
            workflow_cache_dir: PathBuf::default(),
        };
        cfg.discover_dirs();
        cfg
    }

    pub fn setup() -> Result<Config, Error> {
        info!("Starting in setup");
        let config = Config::read()?;
        Ok(config)
    }

    fn read() -> Result<Config, Error> {
        info!("Starting in read");
        // If config file exists read settings
        let mut p = Config::get_workflow_dirs().0;
        p.push(CONFIG_FILE_NAME);
        if p.exists() {
            let mut config: Config = File::open(p)
                .map_err(|e| {
                    let _err: Error = From::from(e);
                    _err
                })
                .and_then(|f| {
                    let mut content = String::new();
                    let mut reader = BufReader::with_capacity(FILE_BUF_SIZE, f);
                    reader.read_to_string(&mut content).map_err(|e| {
                        let _err: Error = From::from(e);
                        _err
                    })?;
                    Ok(content)
                })
                .and_then(|inp| {
                    serde_json::from_str(&inp).map_err(|_| {
                        let workflow_err: Error = From::from(AlfredError::ConfigFileErr);
                        workflow_err
                    })
                })?;
            config.discover_dirs();
            Ok(config)
        } else {
            Err(From::from(AlfredError::MissingConfigFile))
        }
    }

    pub fn save(&self) -> Result<(), String> {
        info!("Starting in save");
        create_dir_all(&self.workflow_data_dir).map_err(|e| e.to_string())?;

        let mut settings_fn = self.workflow_data_dir.clone();
        settings_fn.push(CONFIG_FILE_NAME);
        let fp = File::create(settings_fn).map_err(|e| e.to_string())?;
        serde_json::to_string(self)
            .map_err(|e| e.to_string())
            .and_then(|content| {
                let mut writer = BufWriter::with_capacity(FILE_BUF_SIZE, fp);
                writer
                    .write_all(content.as_ref())
                    .map_err(|e| e.to_string())
            })
    }

    pub fn discover_dirs(&mut self) {
        info!("Starting in discover_dirs");
        let dirs = Config::get_workflow_dirs();
        self.workflow_data_dir = dirs.0;
        self.workflow_cache_dir = dirs.1;
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.workflow_cache_dir
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.workflow_data_dir
    }

    pub fn is_alfred_v3(&self) -> bool {
        info!("Starting in is_alfred_v3");
        let r = VersionReq::parse("~3").unwrap();
        if r.matches(&self.alfred_version) {
            true
        } else {
            false
        }
    }

    fn get_workflow_dirs() -> (PathBuf, PathBuf) {
        info!("Starting in get_workflow_dirs");
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

fn get_alfred_version() -> Version {
    info!("Starting in get_alfred_version");
    alfred::env::version().map_or(
        Version::parse("2.0.0").expect("Parsing 2.0.0 shouldn't fail"),
        |ref s| {
            Version::parse(s).unwrap_or_else(|_| {
                if s.starts_with("3.") {
                    Version::parse("3.0.0").expect("Parsing 3.0.0 shouldn't fail")
                } else {
                    Version::parse("2.0.0").expect("Parsing 2.0.0 shouldn't fail")
                }
            })
        },
    )
}

fn get_epoch() -> DateTime<Utc> {
    info!("Starting in get_epoch");
    "1970-01-01T23:00:00Z".parse::<DateTime<Utc>>().unwrap()
}
