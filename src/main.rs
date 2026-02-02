use std::borrow::Cow;
use std::io::{self, BufWriter, Write};
use std::process;
use structopt::StructOpt;
use thiserror::Error;

mod bookmark;
mod cli;
mod searcher;
mod tag_manager;

use crate::bookmark::{BookmarkCache, ChromeBookmark, get_chrome_bookmarks_path};
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

    std::fs::create_dir_all(&data_dir)?;

    let tag_db_path = data_dir.join("tags.db");
    let tag_manager = TagManager::new(tag_db_path)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let bookmark_cache = BookmarkCache::new(&data_dir);

    match opt.cmd {
        SubCommand::Search {
            query,
            tags,
            fuzzy,
            limit,
        } => {
            handle_search(query, tags, fuzzy, limit, &tag_manager, &bookmark_cache)?;
        }
        SubCommand::Tag { bookmark, tags } => {
            handle_tag(bookmark, tags, &tag_manager, &bookmark_cache)?;
        }
        SubCommand::Untag { bookmark, tag } => {
            handle_untag(bookmark, tag, &tag_manager, &bookmark_cache)?;
        }
        SubCommand::ListTags { prefix } => {
            handle_list_tags(prefix, &tag_manager)?;
        }
        SubCommand::ShowTags { bookmark } => {
            handle_show_tags(bookmark, &tag_manager, &bookmark_cache)?;
        }
        SubCommand::RenameTag { old_tag, new_tag } => {
            handle_rename_tag(old_tag, new_tag, &tag_manager)?;
        }
        SubCommand::Refresh => {
            show_info_alfred("Chrome书签已刷新");
        }
        SubCommand::Stats => {
            handle_stats(&tag_manager, &bookmark_cache)?;
        }
    }

    Ok(())
}

fn load_chrome_bookmarks(cache: &BookmarkCache) -> Result<Vec<ChromeBookmark>, Box<dyn std::error::Error>> {
    let bookmarks_path = get_chrome_bookmarks_path()
        .ok_or(AppError::BookmarksNotFound)?;

    cache.load(&bookmarks_path)
        .map_err(|e| AppError::BookmarksReadError(e.to_string()).into())
}

fn handle_search(
    query: Vec<String>,
    tags: Option<String>,
    fuzzy: bool,
    limit: usize,
    tag_manager: &TagManager,
    cache: &BookmarkCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks(cache)?;
    let searcher = BookmarkSearcher::new();

    let query_str = query.join(" ");
    let search_tags: Vec<String> = if let Some(tags_str) = tags {
        tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else {
        Vec::new()
    };

    let results = searcher.search(&bookmarks, tag_manager, &query_str, &search_tags, fuzzy, limit)?;

    // 预计算所有字符串，确保它们的生命周期足够长
    struct PrecomputedStrings {
        subtitle: String,
        cmd_subtitle: String,
        opt_subtitle: String,
    }

    let precomputed: Vec<PrecomputedStrings> = results.iter().take(limit).map(|result| {
        let tags_str = result.tags.join(", ");
        let domain = extract_domain(&result.bookmark.url);
        PrecomputedStrings {
            subtitle: build_subtitle(&tags_str, &result.bookmark.folder_path, &domain),
            cmd_subtitle: format!("复制URL: {}", result.bookmark.url),
            opt_subtitle: format!("📂 {}", result.bookmark.folder_path.as_deref().unwrap_or("未分类")),
        }
    }).collect();

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let mut items = Vec::with_capacity(precomputed.len() + 1);

    for (result, strings) in results.iter().take(limit).zip(precomputed.iter()) {
        let item = alfred::ItemBuilder::new(&result.bookmark.name)
            .subtitle(&strings.subtitle)
            .arg(&result.bookmark.url)
            .uid(&result.bookmark.id)
            .quicklook_url(&result.bookmark.url)
            .valid(true)
            .modifier(
                alfred::Modifier::Command,
                Some(&strings.cmd_subtitle),
                Some(&result.bookmark.url),
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
            .text_copy(&result.bookmark.url)
            .text_large_type(&result.bookmark.name)
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
    cache: &BookmarkCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks(cache)?;

    let found_bookmark = bookmarks
        .iter()
        .find(|b| b.url == bookmark || b.id == bookmark)
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

    tag_manager.add_tags(&found_bookmark.id, &tags)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    show_info_alfred(format!("已为书签添加tags: {}", tags.join(", ")));
    Ok(())
}

fn handle_untag(
    bookmark: String,
    tag: String,
    tag_manager: &TagManager,
    cache: &BookmarkCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks(cache)?;

    let found_bookmark = bookmarks
        .iter()
        .find(|b| b.url == bookmark || b.id == bookmark)
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

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
    cache: &BookmarkCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks(cache)?;

    let found_bookmark = bookmarks
        .iter()
        .find(|b| b.url == bookmark || b.id == bookmark)
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

fn handle_stats(tag_manager: &TagManager, cache: &BookmarkCache) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks(cache)?;
    let tagged_count = tag_manager.get_bookmark_count()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let all_tags = tag_manager.get_all_tags()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let info = format!(
        "Chrome书签: {} | 已标记: {} | Tags: {}",
        bookmarks.len(),
        tagged_count,
        all_tags.len()
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
