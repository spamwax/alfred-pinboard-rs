use std::borrow::Cow;
use std::collections::HashSet;
use std::io::{self, BufWriter, Write};
use std::process;
use structopt::StructOpt;
use thiserror::Error;

mod bookmark;
mod cli;
mod searcher;
mod tag_manager;

use crate::bookmark::{BookmarkCache, compute_bookmarks_fingerprint, get_chrome_bookmarks_path};
use crate::cli::{Opt, SubCommand};
use crate::searcher::BookmarkSearcher;
use crate::tag_manager::TagManager;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Chrome书签文件未找到")]
    BookmarksNotFound,
    #[error("读取书签失败: {0}")]
    BookmarksReadError(String),
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    #[error("书签未找到: {0}")]
    BookmarkNotFound(String),
    #[error("其他错误: {0}")]
    Other(String),
}

fn main() {
    let opt: Opt = Opt::from_args();

    if let Err(e) = run(opt) {
        show_error_alfred(e.to_string());
        process::exit(1);
    }
}

fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = if let Ok(dir) = std::env::var("alfred_workflow_data") {
        std::path::PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .ok_or(AppError::Other("无法获取home目录".to_string()))?
            .join(".alfred-chrome-bookmarks")
    };

    let cache_dir = if let Ok(dir) = std::env::var("alfred_workflow_cache") {
        std::path::PathBuf::from(dir)
    } else {
        data_dir.clone()
    };

    std::fs::create_dir_all(&data_dir)?;
    if cache_dir != data_dir {
        std::fs::create_dir_all(&cache_dir)?;
    }

    let tag_db_path = data_dir.join("tags.db");
    let tag_manager = TagManager::new(tag_db_path)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let bookmark_cache = BookmarkCache::new(&cache_dir);

    let needs_index = matches!(
        opt.cmd,
        SubCommand::Search { .. }
            | SubCommand::Tag { .. }
            | SubCommand::Untag { .. }
            | SubCommand::ShowTags { .. }
            | SubCommand::Stats
    );

    if needs_index {
        let bookmarks_path = get_chrome_bookmarks_path()
            .ok_or(AppError::BookmarksNotFound)?;
        ensure_bookmark_index(&tag_manager, &bookmark_cache, &bookmarks_path)?;
    }

    match opt.cmd {
        SubCommand::Search {
            query,
            tags,
            fuzzy,
            limit,
        } => {
            handle_search(query, tags, fuzzy, limit, &tag_manager)?;
        }
        SubCommand::Tag { bookmark, tags } => {
            handle_tag(bookmark, tags, &tag_manager)?;
        }
        SubCommand::Untag { bookmark, tag } => {
            handle_untag(bookmark, tag, &tag_manager)?;
        }
        SubCommand::ListTags { prefix } => {
            handle_list_tags(prefix, &tag_manager)?;
        }
        SubCommand::ShowTags { bookmark } => {
            handle_show_tags(bookmark, &tag_manager)?;
        }
        SubCommand::RenameTag { old_tag, new_tag } => {
            handle_rename_tag(old_tag, new_tag, &tag_manager)?;
        }
        SubCommand::Refresh => {
            let bookmarks_path = get_chrome_bookmarks_path()
                .ok_or(AppError::BookmarksNotFound)?;
            bookmark_cache.invalidate();
            tag_manager
                .clear_bookmarks_index()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            ensure_bookmark_index(&tag_manager, &bookmark_cache, &bookmarks_path)?;
            show_info_alfred("Chrome书签已刷新");
        }
        SubCommand::Stats => {
            handle_stats(&tag_manager)?;
        }
    }

    Ok(())
}

fn ensure_bookmark_index(
    tag_manager: &TagManager,
    cache: &BookmarkCache,
    bookmarks_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let fingerprint = compute_bookmarks_fingerprint(bookmarks_path)?;
    if !tag_manager
        .bookmarks_need_refresh(&fingerprint)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    {
        return Ok(());
    }

    let bookmarks = cache
        .load(&bookmarks_path.to_path_buf())
        .map_err(|e| AppError::BookmarksReadError(e.to_string()))?;
    tag_manager
        .replace_bookmarks(&bookmarks, &fingerprint)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

fn handle_search(
    query: Vec<String>,
    tags: Option<String>,
    fuzzy: bool,
    limit: usize,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let searcher = BookmarkSearcher::new();

    let query_str = query.join(" ");
    let search_tags: Vec<String> = if let Some(tags_str) = tags {
        normalize_tags(tags_str.split(','))
    } else {
        Vec::new()
    };

    let fallback_search = || -> Result<Vec<crate::bookmark::ChromeBookmark>, Box<dyn std::error::Error>> {
        let bookmarks = tag_manager
            .load_all_bookmarks()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let results = searcher.search(&bookmarks, tag_manager, &query_str, &search_tags, fuzzy, limit)?;
        Ok(results.into_iter().map(|result| result.bookmark).collect())
    };

    let bookmarks = if query_str.is_empty() {
        if search_tags.is_empty() {
            tag_manager
                .list_bookmarks(limit)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?
        } else {
            tag_manager
                .list_bookmarks_by_tags(&search_tags, limit)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?
        }
    } else if !fuzzy {
        match tag_manager
            .search_bookmarks_fts(&query_str, &search_tags, limit)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        {
            Some(results) => results,
            None => fallback_search()?,
        }
    } else {
        fallback_search()?
    };

    // 预计算所有字符串，确保它们的生命周期足够长
    struct PrecomputedStrings {
        subtitle: String,
        cmd_subtitle: String,
        opt_subtitle: String,
    }

    let result_ids: Vec<String> = bookmarks
        .iter()
        .map(|bookmark| bookmark.id.clone())
        .collect();

    let tags_map = tag_manager
        .get_tags_for_bookmarks(&result_ids)
        .unwrap_or_default();

    let precomputed: Vec<PrecomputedStrings> = bookmarks.iter().take(limit).map(|bookmark| {
        let tags_str = tags_map
            .get(&bookmark.id)
            .cloned()
            .unwrap_or_default()
            .join(", ");
        let domain = extract_domain(&bookmark.url);
        PrecomputedStrings {
            subtitle: build_subtitle(&tags_str, &bookmark.folder_path, &domain),
            cmd_subtitle: format!("复制URL: {}", bookmark.url),
            opt_subtitle: format!("📂 {}", bookmark.folder_path.as_deref().unwrap_or("未分类")),
        }
    }).collect();

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let mut items = Vec::with_capacity(precomputed.len() + 1);

    for (bookmark, strings) in bookmarks.iter().take(limit).zip(precomputed.iter()) {
        let item = alfred::ItemBuilder::new(&bookmark.name)
            .subtitle(&strings.subtitle)
            .arg(&bookmark.url)
            .uid(&bookmark.id)
            .quicklook_url(&bookmark.url)
            .valid(true)
            .modifier(
                alfred::Modifier::Command,
                Some(&strings.cmd_subtitle),
                Some(&bookmark.url),
                true,
                None,
            )
            .modifier(
                alfred::Modifier::Option,
                Some(&strings.opt_subtitle),
                None::<&str>,
                false,
                None,
            )
            .text_copy(&bookmark.url)
            .text_large_type(&bookmark.name)
            .into_item();

        items.push(item);
    }

    let empty_subtitle = if !search_tags.is_empty() {
        format!("当前搜索tags: {} | 尝试使用不同的关键词", search_tags.join(", "))
    } else {
        "尝试使用不同的关键词".to_string()
    };

    if items.is_empty() {
        items.push(
            alfred::ItemBuilder::new("未找到书签")
                .subtitle(&empty_subtitle)
                .valid(false)
                .into_item(),
        );
    }

    alfred::json::write_items(&mut writer, &items)?;
    writer.flush()?;
    Ok(())
}

/// 从URL提取域名
fn extract_domain(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}

fn normalize_tags<I, S>(tags: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for tag in tags {
        let trimmed = tag.as_ref().trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }

    normalized
}

/// 构建改进的subtitle
fn build_subtitle(tags_str: &str, folder_path: &Option<String>, domain: &str) -> String {
    let mut parts = Vec::new();

    if !tags_str.is_empty() {
        parts.push(format!("🏷 {}", tags_str));
    }

    if let Some(ref path) = folder_path {
        // 简化文件夹路径，只显示最后两级
        let short_path: String = path
            .split('/')
            .rev()
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("/");
        if !short_path.is_empty() {
            parts.push(format!("📂 {}", short_path));
        }
    }

    parts.push(domain.to_string());
    parts.join("  ·  ")
}

fn handle_tag(
    bookmark: String,
    tags: Vec<String>,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let found_bookmark = tag_manager
        .get_bookmark_by_id_or_url(&bookmark)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

    let tags = normalize_tags(tags);
    if tags.is_empty() {
        return Err(AppError::Other("未提供有效的tag".to_string()).into());
    }

    let added = tag_manager.add_tags(&found_bookmark.id, &tags)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    show_info_alfred(format!(
        "已为书签添加{}个tags: {}",
        added,
        tags.join(", ")
    ));
    Ok(())
}

fn handle_untag(
    bookmark: String,
    tag: String,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let found_bookmark = tag_manager
        .get_bookmark_by_id_or_url(&bookmark)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

    let tag = tag.trim().to_string();
    if tag.is_empty() {
        return Err(AppError::Other("未提供有效的tag".to_string()).into());
    }

    tag_manager.remove_tag(&found_bookmark.id, &tag)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    show_info_alfred(format!("已删除tag: {}", tag));
    Ok(())
}

fn handle_list_tags(
    prefix: Option<String>,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let searcher = BookmarkSearcher::new();
    let suggestions = searcher.get_tag_suggestions(tag_manager, &prefix.unwrap_or_default())?;

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let mut items = Vec::with_capacity(suggestions.len() + 1);

    let subtitles: Vec<String> = suggestions.iter()
        .map(|(_, count)| format!("使用次数: {} | 按回车筛选此tag", count))
        .collect();

    for (i, (tag, _)) in suggestions.iter().enumerate() {
        let item = alfred::ItemBuilder::new(tag)
            .subtitle(&subtitles[i])
            .arg(tag)
            .uid(tag)
            .autocomplete(tag)
            .valid(true)
            .into_item();

        items.push(item);
    }

    if items.is_empty() {
        items.push(
            alfred::ItemBuilder::new("没有tags")
                .subtitle("使用 'tag' 命令添加tags")
                .valid(false)
                .into_item(),
        );
    }

    alfred::json::write_items(&mut writer, &items)?;
    writer.flush()?;
    Ok(())
}

fn handle_show_tags(
    bookmark: String,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let found_bookmark = tag_manager
        .get_bookmark_by_id_or_url(&bookmark)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

    let tags = tag_manager.get_tags(&found_bookmark.id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let uids: Vec<String> = tags.iter()
        .map(|tag| format!("{}:{}", found_bookmark.id, tag))
        .collect();

    let mut items = Vec::new();
    if tags.is_empty() {
        items.push(
            alfred::ItemBuilder::new("此书签没有tags")
                .subtitle(&found_bookmark.name)
                .valid(false)
                .into_item(),
        );
    } else {
        for (i, tag) in tags.iter().enumerate() {
            let item = alfred::ItemBuilder::new(tag)
                .subtitle(&found_bookmark.name)
                .arg(tag)
                .uid(&uids[i])
                .valid(true)
                .into_item();

            items.push(item);
        }
    }

    alfred::json::write_items(&mut writer, &items)?;
    writer.flush()?;
    Ok(())
}

fn handle_rename_tag(
    old_tag: String,
    new_tag: String,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let updated = tag_manager.rename_tag(&old_tag, &new_tag)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    show_info_alfred(format!("已重命名tag，影响{}个书签", updated));
    Ok(())
}

fn handle_stats(tag_manager: &TagManager) -> Result<(), Box<dyn std::error::Error>> {
    let total_bookmarks = tag_manager.get_total_bookmarks()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let tagged_count = tag_manager.get_bookmark_count()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let tag_count = tag_manager.get_tag_count()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let info = format!(
        "Chrome书签: {} | 已标记: {} | Tags: {}",
        total_bookmarks,
        tagged_count,
        tag_count
    );

    show_info_alfred(info);
    Ok(())
}

fn show_error_alfred<'a, T: Into<Cow<'a, str>>>(s: T) {
    let item = alfred::ItemBuilder::new("错误")
        .subtitle(s)
        .valid(false)
        .into_item();
    let _ = alfred::json::write_items(io::stdout(), &[item]);
}

fn show_info_alfred<'a, T: Into<Cow<'a, str>>>(s: T) {
    let item = alfred::ItemBuilder::new("✓ 操作完成")
        .subtitle(s)
        .valid(false)
        .into_item();
    let _ = alfred::json::write_items(io::stdout(), &[item]);
}
