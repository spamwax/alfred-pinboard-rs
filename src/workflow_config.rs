use chrono::prelude::*;
use dirs::home_dir;
use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::PathBuf;

use alfred_rs::Data;

use semver::{Version, VersionReq};

use crate::AlfredError;

pub(crate) const CONFIG_FILE_NAME: &str = "settings.json";
pub(crate) const CONFIG_KEY_NAME: &str = "settings";

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
    /// Flag to show either url or tags of current bookmark
    #[serde(default)]
    pub show_url_vs_tags: bool,
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

impl Config {
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
            show_url_vs_tags: true,
            auth_token: String::new(),
            update_time: get_epoch(),
            workflow_data_dir: PathBuf::default(),
            workflow_cache_dir: PathBuf::default(),
        };
        cfg.discover_dirs();
        cfg
    }

    pub fn setup() -> Result<Config, Box<dyn std::error::Error>> {
        debug!("Starting in setup");
        let (data_dir, cache_dir) = Config::get_workflow_dirs();
        let config = Config::read(data_dir, cache_dir)?;
        Ok(config)
    }

    fn read(
        mut data_dir: PathBuf,
        cached_dir: PathBuf,
    ) -> Result<Config, Box<dyn std::error::Error>> {
        debug!("Starting in read");
        data_dir.push(CONFIG_FILE_NAME);
        if data_dir.exists() {
            let config = Data::load(&data_dir)?;
            let config: Result<Config, Box<dyn std::error::Error>> = config
                .get(CONFIG_KEY_NAME)
                .map_or_else(
                    || {
                        // println!("--> Resorting to old config file format.");
                        File::open(&data_dir).map_err(|e| e.into()).and_then(|fp| {
                            let buf_reader = BufReader::with_capacity(FILE_BUF_SIZE, fp);
                            serde_json::from_reader(buf_reader)
                                .map_err(|_| From::from(AlfredError::ConfigFileErr))
                        })
                    },
                    Ok,
                )
                .map(|mut c: Config| {
                    assert!(data_dir.pop());
                    c.workflow_data_dir = data_dir;
                    c.workflow_cache_dir = cached_dir;
                    c
                });
            config
        } else {
            Err(From::from(AlfredError::MissingConfigFile))
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting in save");
        create_dir_all(&self.data_dir())?;

        let mut mydata = Data::load(CONFIG_FILE_NAME)?;
        mydata.clear();
        mydata.set(CONFIG_KEY_NAME, self).map_err(|e| e.into())
    }

    pub fn discover_dirs(&mut self) {
        debug!("Starting in discover_dirs");
        let dirs = Config::get_workflow_dirs();
        self.workflow_data_dir = dirs.0;
        self.workflow_cache_dir = dirs.1;
    }

    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &PathBuf {
        &self.workflow_cache_dir
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.workflow_data_dir
    }

    pub fn can_use_json(&self) -> bool {
        // Alfred v3 & above support reading/writing Items in json format
        debug!("Starting in can_use_json");
        let required_version =
            VersionReq::parse(">= 3").expect("Couldn't parse >= 3 version string");
        required_version.matches(&self.alfred_version)
    }

    fn get_workflow_dirs() -> (PathBuf, PathBuf) {
        debug!("Starting in get_workflow_dirs");
        let cache_dir = alfred::env::workflow_cache().unwrap_or_else(|| {
            let mut dir = home_dir().unwrap_or_else(|| PathBuf::from(""));
            dir.push(".cache");
            dir.push("alfred-pinboard-rs");
            dir
        });
        debug!("cache_dir: {}", cache_dir.to_string_lossy());
        if !cache_dir.exists() {
            // If we can't create directories, workflow won't be able to work, so we panic!
            debug!("creating cache_dir");
            create_dir_all(&cache_dir).unwrap();
        }
        let data_dir = alfred::env::workflow_data().unwrap_or_else(|| {
            let mut dir = home_dir().unwrap_or_else(|| PathBuf::from(""));
            dir.push(".config");
            dir.push("alfred-pinboard-rs");
            dir
        });
        debug!("data_dir: {}", data_dir.to_string_lossy());
        if !data_dir.exists() {
            // If we can't create directories, workflow won't be able to work, so we panic!
            debug!("creating data_dir");
            create_dir_all(&data_dir).unwrap();
        }

        (data_dir, cache_dir)
    }
}

fn get_alfred_version() -> Version {
    debug!("Starting in get_alfred_version");
    let min_version = 2;
    let v2 = Version::new(min_version, 0, 0); // If alfred_version env. cannot be found or parsed according to
                                              // semver.org, we will return this version.
    alfred::env::version().map_or(v2.clone(), |ref s| {
        Version::parse(s).unwrap_or_else(|_| {
            // Alfred version is not semver compliant, thus
            s.find('.') // find first dot
                .map_or(v2, |idx| {
                    let m = s[..idx].parse::<u64>().unwrap_or(min_version); // and parse the number before it
                    Version::new(m, 0, 0)
                })
        })
    })
}

fn get_epoch() -> DateTime<Utc> {
    debug!("Starting in get_epoch");
    "1970-01-01T23:00:00Z"
        .parse::<DateTime<Utc>>()
        .expect("Couldn't create UTC epoch time")
}
