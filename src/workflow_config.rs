use chrono::prelude::*;
use dirs::home_dir;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use alfred;
use failure::Error;
use serde_json;

use semver::{Version, VersionReq};

use crate::AlfredError;

pub(crate) const CONFIG_FILE_NAME: &str = "settings.json";
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
    /// Flag to check if browser's page is already bookmarked
    #[serde(default)]
    pub page_is_bookmarked: bool,
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
        debug!("Starting in new");
        let mut cfg = Config {
            alfred_version: get_alfred_version(),
            pins_to_show: 10,
            tags_to_show: 10,
            tag_only_search: false,
            fuzzy_search: false,
            private_new_pin: true,
            toread_new_pin: false,
            suggest_tags: false,
            page_is_bookmarked: true,
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
        debug!("Starting in setup");
        let (data_dir, cache_dir) = Config::get_workflow_dirs();
        let config = Config::read(data_dir, cache_dir)?;
        Ok(config)
    }

    fn read(mut data_dir: PathBuf, cached_dir: PathBuf) -> Result<Config, Error> {
        debug!("Starting in read");
        data_dir.push(CONFIG_FILE_NAME);
        if data_dir.exists() {
            let mut config: Config = File::open(&data_dir)
                .map_err(|e| {
                    let _err: Error = From::from(e);
                    _err
                })
                .and_then(|fp| {
                    let buf_reader = BufReader::with_capacity(FILE_BUF_SIZE, fp);
                    serde_json::from_reader(buf_reader)
                        .map_err(|_| From::from(AlfredError::ConfigFileErr))
                })?;
            assert!(data_dir.pop());
            config.workflow_data_dir = data_dir;
            config.workflow_cache_dir = cached_dir;
            Ok(config)
        } else {
            Err(From::from(AlfredError::MissingConfigFile))
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        debug!("Starting in save");
        create_dir_all(&self.data_dir())?;

        let mut settings_fn = self.workflow_data_dir.clone();
        settings_fn.push(CONFIG_FILE_NAME);

        File::create(settings_fn)
            .map_err(|e| e.into())
            .and_then(|fp| {
                let buf_writer = BufWriter::with_capacity(FILE_BUF_SIZE, fp);
                serde_json::to_writer(buf_writer, self)?;
                Ok(())
            })
    }

    pub fn discover_dirs(&mut self) {
        debug!("Starting in discover_dirs");
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
        debug!("Starting in is_alfred_v3");
        let v3 = VersionReq::parse("~3").expect("Couldn't parse ~3 version string");
        let v4 = VersionReq::parse("~4").expect("Couldn't parse ~4 version string");
        v3.matches(&self.alfred_version) || v4.matches(&self.alfred_version)
    }

    fn get_workflow_dirs() -> (PathBuf, PathBuf) {
        debug!("Starting in get_workflow_dirs");
        let cache_dir = alfred::env::workflow_cache().unwrap_or_else(|| {
            let mut dir = home_dir().unwrap_or_else(|| PathBuf::from(""));
            dir.push(".cache");
            dir.push("alfred-pinboard-rs");
            dir
        });
        let data_dir = alfred::env::workflow_data().unwrap_or_else(|| {
            let mut dir = home_dir().unwrap_or_else(|| PathBuf::from(""));
            dir.push(".config");
            dir.push("alfred-pinboard-rs");
            dir
        });
        (data_dir, cache_dir)
    }
}

fn get_alfred_version() -> Version {
    debug!("Starting in get_alfred_version");
    alfred::env::version().map_or(
        Version::parse("2.0.0").expect("Parsing 2.0.0 shouldn't fail"),
        |ref s| {
            Version::parse(s).unwrap_or_else(|_| {
                if s.starts_with("3.") {
                    Version::parse("3.0.0").expect("Parsing 3.0.0 shouldn't fail")
                } else if s.starts_with("4."){
                    Version::parse("4.0.0").expect("Parsing 4.0.0 shouldn't fail")
                } else {
                    Version::parse("2.0.0").expect("Parsing 2.0.0 shouldn't fail")
                }
            })
        },
    )
}

fn get_epoch() -> DateTime<Utc> {
    debug!("Starting in get_epoch");
    "1970-01-01T23:00:00Z"
        .parse::<DateTime<Utc>>()
        .expect("Couldn't create UTC epoch time")
}
