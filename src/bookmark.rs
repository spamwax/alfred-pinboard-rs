use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Chrome书签项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeBookmark {
    pub id: String,
    pub name: String,
    pub url: String,
    pub date_added: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_path: Option<String>,
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

    fn extract_from_node(&self, node: &BookmarkNode, folder_path: &str, bookmarks: &mut Vec<ChromeBookmark>) {
        if node.node_type == "url" {
            if let (Some(url), Some(date_added)) = (&node.url, &node.date_added) {
                bookmarks.push(ChromeBookmark {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    url: url.clone(),
                    date_added: date_added.clone(),
                    folder_path: Some(folder_path.to_string()),
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

/// 获取Chrome书签文件路径
pub fn get_chrome_bookmarks_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        // macOS Chrome默认路径
        let chrome_path = home.join("Library/Application Support/Google/Chrome/Default/Bookmarks");
        if chrome_path.exists() {
            return Some(chrome_path);
        }

        // 尝试其他Profile
        for profile_num in 1..10 {
            let profile_path = home.join(format!(
                "Library/Application Support/Google/Chrome/Profile {}/Bookmarks",
                profile_num
            ));
            if profile_path.exists() {
                return Some(profile_path);
            }
        }
    }
    None
}
