use crate::bookmark::ChromeBookmark;
use crate::tag_manager::TagManager;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// 搜索结果
#[derive(Debug, Clone)]
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

    /// 搜索书签
    ///
    /// 参数:
    /// - bookmarks: 所有Chrome书签
    /// - tag_manager: tag管理器
    /// - query: 搜索查询
    /// - search_tags: 要搜索的tags（AND查询）
    /// - fuzzy: 是否使用模糊搜索
    pub fn search(
        &self,
        bookmarks: &[ChromeBookmark],
        tag_manager: &TagManager,
        query: &str,
        search_tags: &[String],
        fuzzy: bool,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // 先按tags过滤
        let filtered_bookmarks = if !search_tags.is_empty() {
            let bookmark_ids = tag_manager.find_bookmarks_by_tags(search_tags)?;
            let id_set: std::collections::HashSet<_> = bookmark_ids.into_iter().collect();
            bookmarks
                .iter()
                .filter(|b| id_set.contains(&b.id))
                .collect::<Vec<_>>()
        } else {
            bookmarks.iter().collect::<Vec<_>>()
        };

        // 如果没有查询词，返回所有过滤后的结果
        if query.is_empty() {
            for bookmark in filtered_bookmarks {
                let tags = tag_manager.get_tags(&bookmark.id).unwrap_or_default();
                results.push(SearchResult {
                    bookmark: bookmark.clone(),
                    tags,
                    score: 0,
                });
            }
            return Ok(results);
        }

        // 按查询词搜索
        for bookmark in filtered_bookmarks {
            let score = if fuzzy {
                self.fuzzy_search(bookmark, query)
            } else {
                self.exact_search(bookmark, query)
            };

            if score > 0 {
                let tags = tag_manager.get_tags(&bookmark.id).unwrap_or_default();
                results.push(SearchResult {
                    bookmark: bookmark.clone(),
                    tags,
                    score,
                });
            }
        }

        // 按分数排序
        results.sort_by(|a, b| b.score.cmp(&a.score));

        Ok(results)
    }

    /// 模糊搜索
    fn fuzzy_search(&self, bookmark: &ChromeBookmark, query: &str) -> i64 {
        let mut max_score = 0i64;

        // 搜索标题
        if let Some(score) = self.fuzzy_matcher.fuzzy_match(&bookmark.name, query) {
            max_score = max_score.max(score * 2); // 标题权重更高
        }

        // 搜索URL
        if let Some(score) = self.fuzzy_matcher.fuzzy_match(&bookmark.url, query) {
            max_score = max_score.max(score);
        }

        // 搜索文件夹路径
        if let Some(ref folder_path) = bookmark.folder_path {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(folder_path, query) {
                max_score = max_score.max(score / 2); // 文件夹权重较低
            }
        }

        max_score
    }

    /// 精确搜索（包含匹配）
    fn exact_search(&self, bookmark: &ChromeBookmark, query: &str) -> i64 {
        let query_lower = query.to_lowercase();
        let mut score = 0i64;

        // 搜索标题
        if bookmark.name.to_lowercase().contains(&query_lower) {
            score += 200;
        }

        // 搜索URL
        if bookmark.url.to_lowercase().contains(&query_lower) {
            score += 100;
        }

        // 搜索文件夹路径
        if let Some(ref folder_path) = bookmark.folder_path {
            if folder_path.to_lowercase().contains(&query_lower) {
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
        let mut suggestions: Vec<(String, usize)> = all_tags
            .into_iter()
            .filter(|(tag, _)| tag.to_lowercase().contains(&prefix_lower))
            .collect();

        // 按使用次数排序
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(suggestions)
    }
}

impl Default for BookmarkSearcher {
    fn default() -> Self {
        Self::new()
    }
}
