use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Chrome书签项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeBookmark {
    pub id: String,
    pub name: String,
    pub url: String,
    pub date_added: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_path: Option<String>,
    /// 预计算的小写名称，用于加速搜索
    #[serde(skip)]
    pub name_lower: String,
    /// 预计算的小写URL
    #[serde(skip)]
    pub url_lower: String,
    /// 预计算的小写文件夹路径
    #[serde(skip)]
    pub folder_path_lower: Option<String>,
}

/// Chrome书签文件的根结构
#[derive(Debug, Deserialize)]
pub struct ChromeBookmarks {
    pub roots: BookmarkRoots,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkRoots {
    pub bookmark_bar: BookmarkNode,
    pub other: BookmarkNode,
    #[serde(default)]
    pub synced: Option<BookmarkNode>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub date_added: Option<String>,
    #[serde(default)]
    pub children: Vec<BookmarkNode>,
}

impl ChromeBookmarks {
    /// 从文件读取Chrome书签
    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let bookmarks: ChromeBookmarks = serde_json::from_str(&content)?;
        Ok(bookmarks)
    }

    /// 提取所有书签（平铺结构）
    pub fn extract_all_bookmarks(&self) -> Vec<ChromeBookmark> {
        let mut bookmarks = Vec::new();

        // 处理书签栏
        self.extract_from_node(&self.roots.bookmark_bar, "书签栏", &mut bookmarks);

        // 处理其他书签
        self.extract_from_node(&self.roots.other, "其他书签", &mut bookmarks);

        // 处理同步书签
        if let Some(ref synced) = self.roots.synced {
            self.extract_from_node(synced, "同步书签", &mut bookmarks);
        }

        bookmarks
    }

    fn extract_from_node(
        &self,
        node: &BookmarkNode,
        folder_path: &str,
        bookmarks: &mut Vec<ChromeBookmark>,
    ) {
        if node.node_type == "url" {
            if let (Some(url), Some(date_added)) = (&node.url, &node.date_added) {
                let folder_path_str = folder_path.to_string();
                bookmarks.push(ChromeBookmark {
                    name_lower: node.name.to_lowercase(),
                    url_lower: url.to_lowercase(),
                    folder_path_lower: Some(folder_path_str.to_lowercase()),
                    id: node.id.clone(),
                    name: node.name.clone(),
                    url: url.clone(),
                    date_added: date_added.clone(),
                    folder_path: Some(folder_path_str),
                });
            }
        } else if node.node_type == "folder" {
            let new_path = format!("{}/{}", folder_path, node.name);
            for child in &node.children {
                self.extract_from_node(child, &new_path, bookmarks);
            }
        }
    }
}

/// 获取受支持浏览器（Chromium系）书签文件路径
pub fn get_chrome_bookmarks_path() -> Option<PathBuf> {
    dirs::home_dir().and_then(|home| get_chrome_bookmarks_path_from_home(&home))
}

fn get_chrome_bookmarks_path_from_home(home: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("ALFRED_CHROME_BOOKMARKS_PATH") {
        let configured = PathBuf::from(path);
        if configured.exists() {
            return Some(configured);
        }
    }

    let app_support_dir = home.join("Library/Application Support");
    let browser_roots = [
        "Google/Chrome",
        "Google/Chrome Beta",
        "Google/Chrome Dev",
        "Google/Chrome Canary",
        "BraveSoftware/Brave-Browser",
        "BraveSoftware/Brave-Browser-Beta",
        "BraveSoftware/Brave-Browser-Nightly",
        "Microsoft Edge",
        "Microsoft Edge Beta",
        "Microsoft Edge Dev",
        "Microsoft Edge Canary",
        "Chromium",
        "Vivaldi",
        "Arc",
        "The Browser Company/Arc",
        "Dia",
        "The Browser Company/Dia",
        "Opera",
        "com.operasoftware.Opera",
        "com.operasoftware.OperaDeveloper",
        "com.operasoftware.OperaNext",
        "com.operasoftware.OperaGX",
        "Sidekick",
    ];

    let mut candidates = Vec::new();
    for browser_root in browser_roots {
        collect_bookmarks_from_browser_root(&app_support_dir.join(browser_root), &mut candidates);
    }

    select_latest_bookmarks(candidates)
}

fn collect_bookmarks_from_browser_root(root: &Path, candidates: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    let root_bookmarks = root.join("Bookmarks");
    if root_bookmarks.exists() {
        candidates.push(root_bookmarks);
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let profile_name = entry.file_name();
        let profile_name = profile_name.to_string_lossy();
        if !is_chromium_profile_dir(&profile_name) {
            continue;
        }

        let profile_bookmarks = entry.path().join("Bookmarks");
        if profile_bookmarks.exists() {
            candidates.push(profile_bookmarks);
        }
    }
}

fn is_chromium_profile_dir(name: &str) -> bool {
    name == "Default"
        || name == "Guest Profile"
        || name == "System Profile"
        || name.starts_with("Profile ")
        || name.starts_with("Person ")
}

fn select_latest_bookmarks(mut candidates: Vec<PathBuf>) -> Option<PathBuf> {
    if candidates.is_empty() {
        return None;
    }

    let mut selected: Option<(std::time::SystemTime, u64, PathBuf)> = None;

    for path in candidates.drain(..) {
        let metadata = match std::fs::metadata(&path) {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let modified = metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let size = metadata.len();

        match selected {
            None => selected = Some((modified, size, path)),
            Some((best_time, best_size, _)) => {
                if modified > best_time || (modified == best_time && size > best_size) {
                    selected = Some((modified, size, path));
                }
            }
        }
    }

    selected.map(|(_, _, path)| path)
}

pub fn compute_bookmarks_fingerprint(
    bookmarks_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(bookmarks_path)?;
    let mtime_nanos = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let canonical = bookmarks_path
        .canonicalize()
        .unwrap_or_else(|_| bookmarks_path.to_path_buf());
    Ok(format!(
        "{}-{}-{}",
        mtime_nanos,
        metadata.len(),
        canonical.to_string_lossy()
    ))
}

/// 带mtime缓存的书签加载器
/// 将解析后的书签序列化到本地缓存文件，只有Chrome书签文件变化时才重新解析
pub struct BookmarkCache {
    cache_path: PathBuf,
    mtime_path: PathBuf,
}

impl BookmarkCache {
    pub fn new(data_dir: &PathBuf) -> Self {
        BookmarkCache {
            cache_path: data_dir.join("bookmarks_cache.json"),
            mtime_path: data_dir.join("bookmarks_mtime"),
        }
    }

    pub fn invalidate(&self) {
        let _ = std::fs::remove_file(&self.cache_path);
        let _ = std::fs::remove_file(&self.mtime_path);
    }

    /// 加载书签，使用缓存（如果Chrome书签文件未变化）
    pub fn load(
        &self,
        bookmarks_path: &PathBuf,
    ) -> Result<Vec<ChromeBookmark>, Box<dyn std::error::Error>> {
        let source_fingerprint = Self::fingerprint(bookmarks_path)?;

        // 检查缓存是否仍然有效
        if let Ok(cached_fingerprint) = std::fs::read_to_string(&self.mtime_path) {
            if cached_fingerprint.trim() == source_fingerprint {
                if let Some(bookmarks) = self.load_cached() {
                    return Ok(bookmarks);
                }
            }
        }

        // 缓存失效，重新解析
        let chrome_bookmarks = match ChromeBookmarks::from_file(bookmarks_path.clone()) {
            Ok(parsed) => parsed,
            Err(err) => {
                if let Some(bookmarks) = self.load_cached() {
                    return Ok(bookmarks);
                }
                return Err(err);
            }
        };
        let bookmarks = chrome_bookmarks.extract_all_bookmarks();

        // 写入缓存（忽略写入失败，不影响功能）
        if let Ok(json) = serde_json::to_vec(&bookmarks) {
            let _ = Self::write_atomic(&self.cache_path, &json);
        }
        let _ = Self::write_atomic(&self.mtime_path, source_fingerprint.as_bytes());

        Ok(bookmarks)
    }

    fn load_cached(&self) -> Option<Vec<ChromeBookmark>> {
        let cached_data = std::fs::read(&self.cache_path).ok()?;
        let bookmarks = serde_json::from_slice::<Vec<ChromeBookmark>>(&cached_data).ok()?;

        let bookmarks = bookmarks
            .into_iter()
            .map(|mut b| {
                b.name_lower = b.name.to_lowercase();
                b.url_lower = b.url.to_lowercase();
                b.folder_path_lower = b.folder_path.as_ref().map(|p| p.to_lowercase());
                b
            })
            .collect();

        Some(bookmarks)
    }

    fn fingerprint(bookmarks_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        compute_bookmarks_fingerprint(bookmarks_path)
    }

    fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
        let tmp_path = path.with_extension("tmp");
        if let Ok(file) = std::fs::File::create(&tmp_path) {
            let mut writer = BufWriter::new(file);
            writer.write_all(data)?;
            writer.flush()?;
            let _ = writer.get_ref().sync_all();
            std::fs::rename(tmp_path, path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_bookmarks(path: &Path, include_other: bool) {
        let other_section = if include_other {
            r#"
    "other": {
      "type": "folder",
      "id": "2",
      "name": "Other Bookmarks",
      "children": [
        {
          "type": "url",
          "id": "20",
          "name": "Other",
          "url": "https://other.com",
          "date_added": "3"
        }
      ]
    }"#
        } else {
            r#"
    "other": {
      "type": "folder",
      "id": "2",
      "name": "Other Bookmarks",
      "children": []
    }"#
        };

        let json = format!(
            r#"{{
  "roots": {{
    "bookmark_bar": {{
      "type": "folder",
      "id": "1",
      "name": "Bookmark Bar",
      "children": [
        {{
          "type": "url",
          "id": "10",
          "name": "Rust",
          "url": "https://rust-lang.org",
          "date_added": "1"
        }},
        {{
          "type": "folder",
          "id": "11",
          "name": "Sub",
          "children": [
            {{
              "type": "url",
              "id": "12",
              "name": "Example",
              "url": "https://example.com",
              "date_added": "2"
            }}
          ]
        }}
      ]
    }},{}
  }}
}}"#,
            other_section
        );
        fs::write(path, json).expect("write bookmarks");
    }

    #[test]
    fn extract_all_bookmarks_builds_paths_and_lowercase() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("Bookmarks");
        write_bookmarks(&path, true);

        let parsed = ChromeBookmarks::from_file(path).expect("parse bookmarks");
        let bookmarks = parsed.extract_all_bookmarks();
        assert_eq!(bookmarks.len(), 3);

        let rust = bookmarks.iter().find(|b| b.id == "10").unwrap();
        assert_eq!(rust.name_lower, "rust");
        assert_eq!(rust.url_lower, "https://rust-lang.org");

        let nested = bookmarks.iter().find(|b| b.id == "12").unwrap();
        assert_eq!(
            nested.folder_path.as_deref(),
            Some("书签栏/Bookmark Bar/Sub")
        );
        assert_eq!(
            nested.folder_path_lower.as_deref(),
            Some("书签栏/bookmark bar/sub")
        );
    }

    #[test]
    fn get_chrome_bookmarks_path_from_home_selects_latest_profile() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();

        let default_path = home.join("Library/Application Support/Google/Chrome/Default");
        fs::create_dir_all(&default_path).expect("create default");
        fs::write(default_path.join("Bookmarks"), "{}").expect("write default");

        let profile_path = home.join("Library/Application Support/Google/Chrome/Profile 1");
        fs::create_dir_all(&profile_path).expect("create profile");
        fs::write(profile_path.join("Bookmarks"), "{\"bigger\":true}").expect("write profile");

        let found_profile = get_chrome_bookmarks_path_from_home(home).expect("find latest");
        assert!(found_profile.ends_with("Profile 1/Bookmarks"));

        fs::remove_file(profile_path.join("Bookmarks")).expect("remove profile");
        let found_default = get_chrome_bookmarks_path_from_home(home).expect("find default");
        assert!(found_default.ends_with("Default/Bookmarks"));
    }

    #[test]
    fn get_chrome_bookmarks_path_supports_arc_and_dia_roots() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();

        let arc_profile = home.join("Library/Application Support/Arc/Default");
        fs::create_dir_all(&arc_profile).expect("create arc default");
        fs::write(arc_profile.join("Bookmarks"), "{}").expect("write arc bookmarks");

        let dia_profile =
            home.join("Library/Application Support/The Browser Company/Dia/Profile 2");
        fs::create_dir_all(&dia_profile).expect("create dia profile");
        fs::write(dia_profile.join("Bookmarks"), "{\"size\":1}").expect("write dia bookmarks");

        let found = get_chrome_bookmarks_path_from_home(home).expect("find bookmarks");
        let found_str = found.to_string_lossy();
        assert!(found_str.contains("/Arc/") || found_str.contains("/Dia/"));
    }

    #[test]
    fn bookmark_cache_invalidates_on_change_and_can_be_cleared() {
        let dir = tempdir().expect("tempdir");
        let data_dir = dir.path().to_path_buf();
        let bookmarks_path = dir.path().join("Bookmarks");

        write_bookmarks(&bookmarks_path, true);
        let cache = BookmarkCache::new(&data_dir);
        let first = cache.load(&bookmarks_path).expect("first load");
        assert_eq!(first.len(), 3);

        write_bookmarks(&bookmarks_path, false);

        let second = cache.load(&bookmarks_path).expect("second load");
        assert_eq!(second.len(), 2);

        let cache_file = data_dir.join("bookmarks_cache.json");
        let mtime_file = data_dir.join("bookmarks_mtime");
        assert!(cache_file.exists());
        assert!(mtime_file.exists());

        cache.invalidate();
        assert!(!cache_file.exists());
        assert!(!mtime_file.exists());
    }

    #[test]
    fn bookmark_cache_falls_back_when_parse_fails() {
        let dir = tempdir().expect("tempdir");
        let data_dir = dir.path().to_path_buf();
        let bookmarks_path = dir.path().join("Bookmarks");

        write_bookmarks(&bookmarks_path, true);
        let cache = BookmarkCache::new(&data_dir);
        let first = cache.load(&bookmarks_path).expect("first load");
        assert_eq!(first.len(), 3);

        fs::write(&bookmarks_path, "{ invalid json").expect("write invalid");

        let fallback = cache.load(&bookmarks_path).expect("fallback load");
        assert_eq!(fallback.len(), 3);
    }
}
