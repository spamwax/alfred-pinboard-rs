use crate::bookmark::ChromeBookmark;
use crate::tag_manager::TagManager;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// 搜索结果
#[derive(Debug)]
pub struct SearchResult {
    pub bookmark: ChromeBookmark,
    pub tags: Vec<String>,
    pub score: i64,
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

    /// 搜索书签（优化版：批量加载tags，使用预计算lowercase，提前终止）
    pub fn search(
        &self,
        bookmarks: &[ChromeBookmark],
        tag_manager: &TagManager,
        query: &str,
        search_tags: &[String],
        fuzzy: bool,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        // 批量加载所有书签tags（一次SQL查询替代N次）
        let all_tags = tag_manager.get_all_bookmark_tags()
            .unwrap_or_default();

        // 先按tags过滤
        let tag_filtered_ids = if !search_tags.is_empty() {
            Some(
                tag_manager.find_bookmarks_by_tags(search_tags)?
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>(),
            )
        } else {
            None
        };

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for bookmark in bookmarks {
            // tag过滤
            if let Some(ref id_set) = tag_filtered_ids {
                if !id_set.contains(&bookmark.id) {
                    continue;
                }
            }

            // 如果没有查询词，直接收集
            let score = if query.is_empty() {
                0
            } else if fuzzy {
                self.fuzzy_search(bookmark, query)
            } else {
                self.exact_search(bookmark, &query_lower)
            };

            if query.is_empty() || score > 0 {
                let tags = all_tags
                    .get(&bookmark.id)
                    .cloned()
                    .unwrap_or_default();
                results.push(SearchResult {
                    bookmark: bookmark.clone(),
                    tags,
                    score,
                });
            }
        }

        // 按分数排序
        if !query.is_empty() {
            results.sort_unstable_by(|a, b| b.score.cmp(&a.score));
        }

        // 截断到limit
        results.truncate(limit);

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
