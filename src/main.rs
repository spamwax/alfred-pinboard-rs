extern crate dirs;
extern crate serde;

extern crate serde_derive;
extern crate serde_json;

extern crate structopt;

extern crate env_logger;

extern crate alfred;

use std::borrow::Cow;
use std::io;
use std::process;
use structopt::StructOpt;
use thiserror::Error;

mod bookmark;
mod cli;
mod searcher;
mod tag_manager;

use crate::bookmark::{ChromeBookmark, ChromeBookmarks, get_chrome_bookmarks_path};
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
    env_logger::init();

    let opt: Opt = Opt::from_args();

    if let Err(e) = run(opt) {
        show_error_alfred(e.to_string());
        process::exit(1);
    }
}

fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    // 获取数据目录
    let data_dir = if let Ok(dir) = std::env::var("alfred_workflow_data") {
        std::path::PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .ok_or(AppError::Other("无法获取home目录".to_string()))?
            .join(".alfred-chrome-bookmarks")
    };

    // 确保数据目录存在
    std::fs::create_dir_all(&data_dir)?;

    // 初始化tag管理器
    let tag_db_path = data_dir.join("tags.db");
    let tag_manager = TagManager::new(tag_db_path)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
            show_info_alfred("Chrome书签已刷新");
        }
        SubCommand::Stats => {
            handle_stats(&tag_manager)?;
        }
    }

    Ok(())
}

fn load_chrome_bookmarks() -> Result<Vec<ChromeBookmark>, Box<dyn std::error::Error>> {
    let bookmarks_path = get_chrome_bookmarks_path()
        .ok_or(AppError::BookmarksNotFound)?;

    let chrome_bookmarks = ChromeBookmarks::from_file(bookmarks_path)
        .map_err(|e| AppError::BookmarksReadError(e.to_string()))?;

    Ok(chrome_bookmarks.extract_all_bookmarks())
}

fn handle_search(
    query: Vec<String>,
    tags: Option<String>,
    fuzzy: bool,
    limit: usize,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks()?;
    let searcher = BookmarkSearcher::new();

    let query_str = query.join(" ");
    let search_tags: Vec<String> = if let Some(tags_str) = tags {
        tags_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };

    let results = searcher.search(&bookmarks, tag_manager, &query_str, &search_tags, fuzzy)?;

    let mut items = Vec::new();
    let mut subtitles = Vec::new();

    for result in results.iter().take(limit) {
        let tags_str = result.tags.join(", ");
        let subtitle = if tags_str.is_empty() {
            result.bookmark.url.clone()
        } else {
            format!("🏷 {} | {}", tags_str, result.bookmark.url)
        };
        subtitles.push(subtitle);
    }

    for (i, result) in results.iter().take(limit).enumerate() {
        let item = alfred::ItemBuilder::new(&result.bookmark.name)
            .subtitle(&subtitles[i])
            .arg(&result.bookmark.url)
            .valid(true)
            .into_item();

        items.push(item);
    }

    if items.is_empty() {
        items.push(
            alfred::ItemBuilder::new("未找到书签")
                .subtitle("尝试使用不同的关键词")
                .valid(false)
                .into_item(),
        );
    }

    alfred::json::write_items(io::stdout(), &items)?;
    Ok(())
}

fn handle_tag(
    bookmark: String,
    tags: Vec<String>,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks()?;

    // 查找书签
    let found_bookmark = bookmarks
        .iter()
        .find(|b| b.url == bookmark || b.id == bookmark)
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

    // 添加tags
    tag_manager.add_tags(&found_bookmark.id, &tags)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    show_info_alfred(format!("已为书签添加tags: {}", tags.join(", ")));
    Ok(())
}

fn handle_untag(
    bookmark: String,
    tag: String,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks()?;

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

    let mut items = Vec::new();
    let mut subtitles = Vec::new();

    for (_tag, count) in &suggestions {
        subtitles.push(format!("使用次数: {}", count));
    }

    for (i, (tag, _count)) in suggestions.iter().enumerate() {
        let item = alfred::ItemBuilder::new(tag)
            .subtitle(&subtitles[i])
            .arg(tag)
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

    alfred::json::write_items(io::stdout(), &items)?;
    Ok(())
}

fn handle_show_tags(
    bookmark: String,
    tag_manager: &TagManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmarks = load_chrome_bookmarks()?;

    let found_bookmark = bookmarks
        .iter()
        .find(|b| b.url == bookmark || b.id == bookmark)
        .ok_or_else(|| AppError::BookmarkNotFound(bookmark.clone()))?;

    let tags = tag_manager.get_tags(&found_bookmark.id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut items = Vec::new();
    if tags.is_empty() {
        items.push(
            alfred::ItemBuilder::new("此书签没有tags")
                .subtitle(&found_bookmark.name)
                .valid(false)
                .into_item(),
        );
    } else {
        for tag in &tags {
            let item = alfred::ItemBuilder::new(tag)
                .subtitle(&found_bookmark.name)
                .arg(tag)
                .valid(true)
                .into_item();

            items.push(item);
        }
    }

    alfred::json::write_items(io::stdout(), &items)?;
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
    let bookmarks = load_chrome_bookmarks()?;
    let tagged_count = tag_manager.get_bookmark_count()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let all_tags = tag_manager.get_all_tags()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let info = format!(
        "Chrome书签总数: {}\n已标记书签: {}\nTags总数: {}",
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
    let item = alfred::ItemBuilder::new("信息")
        .subtitle(s)
        .valid(false)
        .into_item();
    let _ = alfred::json::write_items(io::stdout(), &[item]);
}
