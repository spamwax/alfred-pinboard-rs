use crate::bookmark::ChromeBookmark;
use crate::tag_manager::TagManager;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// 搜索结果
#[derive(Debug)]
pub struct SearchResult {
    pub bookmark: ChromeBookmark,
}

/// 搜索器
pub struct BookmarkSearcher {
    fuzzy_matcher: SkimMatcherV2,
}

impl BookmarkSearcher {
    pub fn new() -> Self {
        BookmarkSearcher {
            fuzzy_matcher: SkimMatcherV2::default(),
        }
    }

    /// 搜索书签（优化版：使用预计算lowercase，Top-K选择，提前终止）
    pub fn search(
        &self,
        bookmarks: &[ChromeBookmark],
        tag_manager: &TagManager,
        query: &str,
        search_tags: &[String],
        folder_filters: &[String],
        fuzzy: bool,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        // 先按tags过滤
        let tag_filtered_ids = if !search_tags.is_empty() {
            let ids = tag_manager.find_bookmarks_by_tags(search_tags)?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            Some(ids.into_iter().collect::<std::collections::HashSet<_>>())
        } else {
            None
        };

        let normalized_folder_filters: Vec<Vec<String>> = folder_filters
            .iter()
            .filter_map(|raw| normalize_folder_filter(raw))
            .collect();

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        if query.is_empty() {
            for bookmark in bookmarks {
                if let Some(ref id_set) = tag_filtered_ids {
                    if !id_set.contains(&bookmark.id) {
                        continue;
                    }
                }

                if !matches_folder_filters(bookmark, &normalized_folder_filters) {
                    continue;
                }

                results.push(SearchResult {
                    bookmark: bookmark.clone(),
                });

                if results.len() >= limit {
                    break;
                }
            }

            return Ok(results);
        }

        #[derive(Debug)]
        struct HeapItem {
            score: i64,
            idx: usize,
            bookmark: ChromeBookmark,
        }

        impl PartialEq for HeapItem {
            fn eq(&self, other: &Self) -> bool {
                self.score == other.score && self.idx == other.idx
            }
        }

        impl Eq for HeapItem {}

        impl Ord for HeapItem {
            fn cmp(&self, other: &Self) -> Ordering {
                match self.score.cmp(&other.score) {
                    Ordering::Equal => other.idx.cmp(&self.idx),
                    other => other,
                }
            }
        }

        impl PartialOrd for HeapItem {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap: BinaryHeap<std::cmp::Reverse<HeapItem>> = BinaryHeap::new();

        for (idx, bookmark) in bookmarks.iter().enumerate() {
            // tag过滤
            if let Some(ref id_set) = tag_filtered_ids {
                if !id_set.contains(&bookmark.id) {
                    continue;
                }
            }

            if !matches_folder_filters(bookmark, &normalized_folder_filters) {
                continue;
            }

            let score = if fuzzy {
                self.fuzzy_search(bookmark, query)
            } else {
                self.exact_search(bookmark, &query_lower)
            };

            if score <= 0 {
                continue;
            }

            let candidate = HeapItem {
                score,
                idx,
                bookmark: bookmark.clone(),
            };

            if heap.len() < limit {
                heap.push(std::cmp::Reverse(candidate));
                continue;
            }

            if let Some(smallest) = heap.peek() {
                let better =
                    score > smallest.0.score || (score == smallest.0.score && idx < smallest.0.idx);
                if better {
                    heap.pop();
                    heap.push(std::cmp::Reverse(candidate));
                }
            }
        }

        let mut items: Vec<HeapItem> = heap.into_iter().map(|item| item.0).collect();
        items.sort_unstable_by(|a, b| b.score.cmp(&a.score).then_with(|| a.idx.cmp(&b.idx)));

        for item in items {
            results.push(SearchResult {
                bookmark: item.bookmark,
            });
        }

        Ok(results)
    }

    /// 模糊搜索
    fn fuzzy_search(&self, bookmark: &ChromeBookmark, query: &str) -> i64 {
        let mut max_score = 0i64;

        if let Some(score) = self.fuzzy_matcher.fuzzy_match(&bookmark.name, query) {
            max_score = max_score.max(score * 2);
        }

        if let Some(score) = self.fuzzy_matcher.fuzzy_match(&bookmark.url, query) {
            max_score = max_score.max(score);
        }

        if let Some(ref folder_path) = bookmark.folder_path {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(folder_path, query) {
                max_score = max_score.max(score / 2);
            }
        }

        max_score
    }

    /// 精确搜索（使用预计算的lowercase字段）
    fn exact_search(&self, bookmark: &ChromeBookmark, query_lower: &str) -> i64 {
        let mut score = 0i64;

        if bookmark.name_lower.contains(query_lower) {
            score += 200;
            // 额外加分：完全匹配标题
            if bookmark.name_lower == query_lower {
                score += 100;
            }
            // 额外加分：以查询开头
            if bookmark.name_lower.starts_with(query_lower) {
                score += 50;
            }
        }

        if bookmark.url_lower.contains(query_lower) {
            score += 100;
        }

        if let Some(ref folder_lower) = bookmark.folder_path_lower {
            if folder_lower.contains(query_lower) {
                score += 50;
            }
        }

        score
    }

    /// 获取所有tags的建议
    pub fn get_tag_suggestions(
        &self,
        tag_manager: &TagManager,
        prefix: &str,
    ) -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
        let all_tags = tag_manager.get_all_tags()?;

        let prefix_lower = prefix.to_lowercase();
        let mut suggestions: Vec<(String, usize)> = if prefix_lower.is_empty() {
            all_tags.into_iter().collect()
        } else {
            all_tags
                .into_iter()
                .filter(|(tag, _)| tag.to_lowercase().contains(&prefix_lower))
                .collect()
        };

        suggestions.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        Ok(suggestions)
    }
}

impl Default for BookmarkSearcher {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_folder_filter(raw: &str) -> Option<Vec<String>> {
    let cleaned = raw
        .trim()
        .to_lowercase()
        .replace('\\', "/")
        .replace('>', "/")
        .replace('|', "/");

    let segments: Vec<String> = cleaned
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect();

    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

fn matches_folder_filters(bookmark: &ChromeBookmark, folder_filters: &[Vec<String>]) -> bool {
    if folder_filters.is_empty() {
        return true;
    }

    let Some(folder_lower) = bookmark.folder_path_lower.as_deref() else {
        return false;
    };
    let folder_segments: Vec<&str> = folder_lower
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();

    folder_filters
        .iter()
        .all(|filter| folder_matches_hierarchy(&folder_segments, filter))
}

fn folder_matches_hierarchy(folder_segments: &[&str], filter_segments: &[String]) -> bool {
    if filter_segments.is_empty() {
        return true;
    }

    if filter_segments.len() == 1 {
        let wanted = &filter_segments[0];
        return folder_segments
            .iter()
            .any(|segment| segment.contains(wanted));
    }

    let mut cursor = 0usize;
    for wanted in filter_segments {
        let mut found = false;
        while cursor < folder_segments.len() {
            if folder_segments[cursor].contains(wanted) {
                found = true;
                cursor += 1;
                break;
            }
            cursor += 1;
        }
        if !found {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag_manager::TagManager;
    use tempfile::tempdir;

    fn bookmark(id: &str, name: &str, url: &str, folder: Option<&str>) -> ChromeBookmark {
        ChromeBookmark {
            id: id.to_string(),
            name: name.to_string(),
            url: url.to_string(),
            date_added: "0".to_string(),
            folder_path: folder.map(|p| p.to_string()),
            name_lower: name.to_lowercase(),
            url_lower: url.to_lowercase(),
            folder_path_lower: folder.map(|p| p.to_lowercase()),
        }
    }

    fn tag_manager() -> (TagManager, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("tags.db");
        (TagManager::new(db_path).expect("manager"), dir)
    }

    #[test]
    fn exact_search_ranks_full_match_first() {
        let (manager, _dir) = tag_manager();
        let searcher = BookmarkSearcher::new();
        let bookmarks = vec![
            bookmark("1", "rust", "https://rust-lang.org", None),
            bookmark("2", "rust-lang", "https://example.com", None),
            bookmark("3", "other", "https://other.com", None),
        ];

        let results = searcher
            .search(&bookmarks, &manager, "rust", &[], &[], false, 10)
            .expect("search");
        assert_eq!(results.first().unwrap().bookmark.id, "1");
    }

    #[test]
    fn search_filters_by_tags_and_limits() {
        let (manager, _dir) = tag_manager();
        manager
            .add_tags("1", &vec!["work".into()])
            .expect("add tags");
        manager
            .add_tags("2", &vec!["personal".into()])
            .expect("add tags");

        let searcher = BookmarkSearcher::new();
        let bookmarks = vec![
            bookmark("1", "alpha", "https://a.com", None),
            bookmark("2", "beta", "https://b.com", None),
            bookmark("3", "gamma", "https://c.com", None),
        ];

        let results = searcher
            .search(
                &bookmarks,
                &manager,
                "",
                &vec!["work".into()],
                &[],
                false,
                1,
            )
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bookmark.id, "1");
    }

    #[test]
    fn empty_query_returns_first_n_in_order() {
        let (manager, _dir) = tag_manager();
        let searcher = BookmarkSearcher::new();
        let bookmarks = vec![
            bookmark("1", "one", "https://1.com", None),
            bookmark("2", "two", "https://2.com", None),
            bookmark("3", "three", "https://3.com", None),
        ];

        let results = searcher
            .search(&bookmarks, &manager, "", &[], &[], false, 2)
            .expect("search");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].bookmark.id, "1");
        assert_eq!(results[1].bookmark.id, "2");
    }

    #[test]
    fn folder_filter_supports_hierarchy_matching() {
        let (manager, _dir) = tag_manager();
        let searcher = BookmarkSearcher::new();
        let bookmarks = vec![
            bookmark(
                "1",
                "rust doc",
                "https://doc.rust-lang.org",
                Some("书签栏/Work/Project/Rust"),
            ),
            bookmark("2", "music", "https://music.example", Some("书签栏/Play")),
        ];

        let results = searcher
            .search(
                &bookmarks,
                &manager,
                "",
                &[],
                &vec!["work/project".into()],
                false,
                10,
            )
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bookmark.id, "1");
    }

    #[test]
    fn folder_filter_accepts_partial_segment() {
        let (manager, _dir) = tag_manager();
        let searcher = BookmarkSearcher::new();
        let bookmarks = vec![
            bookmark(
                "1",
                "rust doc",
                "https://doc.rust-lang.org",
                Some("书签栏/Work/Project/Rust"),
            ),
            bookmark("2", "music", "https://music.example", Some("书签栏/Play")),
        ];

        let results = searcher
            .search(
                &bookmarks,
                &manager,
                "",
                &[],
                &vec!["proj".into()],
                false,
                10,
            )
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bookmark.id, "1");
    }
}
