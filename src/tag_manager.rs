use rusqlite::{params, Connection, Result, ToSql};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use crate::bookmark::ChromeBookmark;

/// Tag管理器
pub struct TagManager {
    conn: Connection,
    fts_enabled: bool,
}

impl TagManager {
    /// 创建或打开tag数据库
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let _ = conn.busy_timeout(Duration::from_millis(500));

        // 性能优化: WAL模式 + 内存临时表 + 宽松同步
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -2000;
             PRAGMA mmap_size = 268435456;",
        )?;

        // 创建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmark_tags (
                bookmark_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (bookmark_id, tag)
            )",
            [],
        )?;

        // 创建索引以加速查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tag ON bookmark_tags(tag)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmark_id ON bookmark_tags(bookmark_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                date_added TEXT NOT NULL,
                folder_path TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_name ON bookmarks(name)",
            [],
        )?;

        let fts_enabled = match conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
                bookmark_id UNINDEXED,
                name,
                url,
                folder_path,
                tokenize = 'unicode61'
            )",
            [],
        ) {
            Ok(_) => true,
            Err(_) => false,
        };

        Ok(TagManager { conn, fts_enabled })
    }

    pub fn fts_enabled(&self) -> bool {
        self.fts_enabled
    }

    /// 为书签添加多个tags
    pub fn add_tags(&self, bookmark_id: &str, tags: &[String]) -> Result<usize> {
        if tags.is_empty() {
            return Ok(0);
        }

        self.conn.execute_batch("BEGIN IMMEDIATE;")?;

        let result: Result<usize> = (|| {
            let mut stmt = self.conn.prepare(
                "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag) VALUES (?1, ?2)",
            )?;
            let mut inserted = 0usize;
            let mut seen = HashSet::new();

            for tag in tags {
                let trimmed = tag.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if !seen.insert(trimmed.to_string()) {
                    continue;
                }
                inserted += stmt.execute(params![bookmark_id, trimmed])?;
            }

            Ok(inserted)
        })();

        match result {
            Ok(inserted) => {
                self.conn.execute_batch("COMMIT;")?;
                Ok(inserted)
            }
            Err(err) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(err)
            }
        }
    }

    /// 删除书签的某个tag
    pub fn remove_tag(&self, bookmark_id: &str, tag: &str) -> Result<()> {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        self.conn.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND tag = ?2",
            params![bookmark_id, trimmed],
        )?;
        Ok(())
    }

    /// 删除书签的所有tags
    #[allow(dead_code)]
    pub fn remove_all_tags(&self, bookmark_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1",
            params![bookmark_id],
        )?;
        Ok(())
    }

    /// 获取书签的所有tags
    pub fn get_tags(&self, bookmark_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM bookmark_tags WHERE bookmark_id = ?1 ORDER BY tag")?;

        let tags = stmt
            .query_map(params![bookmark_id], |row| row.get(0))?
            .collect::<Result<Vec<String>>>()?;

        Ok(tags)
    }

    /// 获取所有tags及其使用次数
    pub fn get_all_tags(&self) -> Result<HashMap<String, usize>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag, COUNT(*) as count FROM bookmark_tags GROUP BY tag ORDER BY count DESC, tag")?;

        let mut tags = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })?;

        for row in rows {
            let (tag, count) = row?;
            tags.insert(tag, count);
        }

        Ok(tags)
    }

    /// 查找包含指定tags的书签ID（支持多tag AND查询）
    pub fn find_bookmarks_by_tags(&self, tags: &[String]) -> Result<Vec<String>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let mut normalized = Vec::new();
        let mut seen = HashSet::new();
        for tag in tags {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                continue;
            }
            if seen.insert(trimmed.to_string()) {
                normalized.push(trimmed.to_string());
            }
        }

        if normalized.is_empty() {
            return Ok(Vec::new());
        }

        // 构建SQL查询：查找同时包含所有指定tags的书签
        let placeholders: Vec<String> = normalized.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT bookmark_id
             FROM bookmark_tags
             WHERE tag IN ({})
             GROUP BY bookmark_id
             HAVING COUNT(DISTINCT tag) = ?",
            placeholders.join(",")
        );

        let mut stmt = self.conn.prepare(&query)?;

        // 绑定参数
        let tag_count = normalized.len();
        let mut params: Vec<&dyn rusqlite::ToSql> = normalized
            .iter()
            .map(|t| t as &dyn rusqlite::ToSql)
            .collect();
        params.push(&tag_count);

        let bookmark_ids = stmt
            .query_map(params.as_slice(), |row| row.get(0))?
            .collect::<Result<Vec<String>>>()?;

        Ok(bookmark_ids)
    }

    /// 查找包含任一指定tag的书签ID（OR查询）
    #[allow(dead_code)]
    pub fn find_bookmarks_by_any_tag(&self, tags: &[String]) -> Result<Vec<String>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT DISTINCT bookmark_id FROM bookmark_tags WHERE tag IN ({})",
            placeholders.join(",")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> =
            tags.iter().map(|t| t as &dyn rusqlite::ToSql).collect();

        let bookmark_ids = stmt
            .query_map(params.as_slice(), |row| row.get(0))?
            .collect::<Result<Vec<String>>>()?;

        Ok(bookmark_ids)
    }

    /// 重命名tag
    pub fn rename_tag(&self, old_tag: &str, new_tag: &str) -> Result<usize> {
        let old_trimmed = old_tag.trim();
        let new_trimmed = new_tag.trim();
        if old_trimmed.is_empty() || new_trimmed.is_empty() {
            return Ok(0);
        }

        let updated = self.conn.execute(
            "UPDATE bookmark_tags SET tag = ?1 WHERE tag = ?2",
            params![new_trimmed, old_trimmed],
        )?;
        Ok(updated)
    }

    /// 批量获取指定书签的tags
    pub fn get_tags_for_bookmarks(
        &self,
        bookmark_ids: &[String],
    ) -> Result<HashMap<String, Vec<String>>> {
        if bookmark_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<String> = bookmark_ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT bookmark_id, tag
             FROM bookmark_tags
             WHERE bookmark_id IN ({})
             ORDER BY bookmark_id, tag",
            placeholders.join(",")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn ToSql> = bookmark_ids.iter().map(|id| id as &dyn ToSql).collect();

        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut result: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            let (bookmark_id, tag) = row?;
            result.entry(bookmark_id).or_default().push(tag);
        }

        Ok(result)
    }

    fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn bookmarks_need_refresh(&self, fingerprint: &str) -> Result<bool> {
        Ok(self.get_meta("bookmarks_fingerprint")?.as_deref() != Some(fingerprint))
    }

    pub fn clear_bookmarks_index(&self) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        let result: Result<()> = (|| {
            self.conn.execute("DELETE FROM bookmarks", [])?;
            if self.fts_enabled {
                self.conn.execute("DELETE FROM bookmarks_fts", [])?;
            }
            self.conn
                .execute("DELETE FROM meta WHERE key = 'bookmarks_fingerprint'", [])?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT;")?;
                Ok(())
            }
            Err(err) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(err)
            }
        }
    }

    pub fn replace_bookmarks(&self, bookmarks: &[ChromeBookmark], fingerprint: &str) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        let result: Result<()> = (|| {
            self.conn.execute("DELETE FROM bookmarks", [])?;
            if self.fts_enabled {
                self.conn.execute("DELETE FROM bookmarks_fts", [])?;
            }

            let mut stmt = self.conn.prepare(
                "INSERT INTO bookmarks (id, name, url, date_added, folder_path)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            let mut fts_stmt = if self.fts_enabled {
                Some(self.conn.prepare(
                    "INSERT INTO bookmarks_fts (bookmark_id, name, url, folder_path)
                     VALUES (?1, ?2, ?3, ?4)",
                )?)
            } else {
                None
            };

            for bookmark in bookmarks {
                stmt.execute(params![
                    bookmark.id,
                    bookmark.name,
                    bookmark.url,
                    bookmark.date_added,
                    bookmark.folder_path
                ])?;
                if let Some(ref mut fts_stmt) = fts_stmt {
                    fts_stmt.execute(params![
                        bookmark.id,
                        bookmark.name,
                        bookmark.url,
                        bookmark.folder_path
                    ])?;
                }
            }

            self.conn.execute(
                "DELETE FROM bookmark_tags
                 WHERE bookmark_id NOT IN (SELECT id FROM bookmarks)",
                [],
            )?;

            self.set_meta("bookmarks_fingerprint", fingerprint)?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT;")?;
                Ok(())
            }
            Err(err) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(err)
            }
        }
    }

    pub fn get_total_bookmarks(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_bookmark_by_id_or_url(&self, id_or_url: &str) -> Result<Option<ChromeBookmark>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, date_added, folder_path
             FROM bookmarks
             WHERE id = ?1 OR url = ?1
             LIMIT 1",
        )?;
        let mut rows = stmt.query(params![id_or_url])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let url: String = row.get(2)?;
            let date_added: String = row.get(3)?;
            let folder_path: Option<String> = row.get(4)?;
            let name_lower = name.to_lowercase();
            let url_lower = url.to_lowercase();
            let folder_path_lower = folder_path.as_ref().map(|p| p.to_lowercase());
            return Ok(Some(ChromeBookmark {
                id,
                name,
                url,
                date_added,
                folder_path,
                name_lower,
                url_lower,
                folder_path_lower,
            }));
        }
        Ok(None)
    }

    pub fn load_all_bookmarks(&self) -> Result<Vec<ChromeBookmark>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, date_added, folder_path
             FROM bookmarks
             ORDER BY rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let url: String = row.get(2)?;
            let date_added: String = row.get(3)?;
            let folder_path: Option<String> = row.get(4)?;
            let name_lower = name.to_lowercase();
            let url_lower = url.to_lowercase();
            let folder_path_lower = folder_path.as_ref().map(|p| p.to_lowercase());
            Ok(ChromeBookmark {
                id,
                name,
                url,
                date_added,
                folder_path,
                name_lower,
                url_lower,
                folder_path_lower,
            })
        })?;

        rows.collect::<Result<Vec<_>>>()
    }

    pub fn list_bookmarks(&self, limit: usize) -> Result<Vec<ChromeBookmark>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, date_added, folder_path
             FROM bookmarks
             ORDER BY rowid
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let url: String = row.get(2)?;
            let date_added: String = row.get(3)?;
            let folder_path: Option<String> = row.get(4)?;
            let name_lower = name.to_lowercase();
            let url_lower = url.to_lowercase();
            let folder_path_lower = folder_path.as_ref().map(|p| p.to_lowercase());
            Ok(ChromeBookmark {
                id,
                name,
                url,
                date_added,
                folder_path,
                name_lower,
                url_lower,
                folder_path_lower,
            })
        })?;
        rows.collect::<Result<Vec<_>>>()
    }

    pub fn list_bookmarks_by_tags(
        &self,
        tags: &[String],
        limit: usize,
    ) -> Result<Vec<ChromeBookmark>> {
        let normalized = normalize_tags(tags);
        if normalized.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = normalized.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT id, name, url, date_added, folder_path
             FROM bookmarks
             WHERE id IN (
                SELECT bookmark_id
                FROM bookmark_tags
                WHERE tag IN ({})
                GROUP BY bookmark_id
                HAVING COUNT(DISTINCT tag) = ?
             )
             ORDER BY rowid
             LIMIT ?",
            placeholders.join(",")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let mut params: Vec<&dyn ToSql> = normalized.iter().map(|t| t as &dyn ToSql).collect();
        let tag_count = normalized.len() as i64;
        params.push(&tag_count);
        let limit_param = limit as i64;
        params.push(&limit_param);

        let rows = stmt.query_map(params.as_slice(), |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let url: String = row.get(2)?;
            let date_added: String = row.get(3)?;
            let folder_path: Option<String> = row.get(4)?;
            let name_lower = name.to_lowercase();
            let url_lower = url.to_lowercase();
            let folder_path_lower = folder_path.as_ref().map(|p| p.to_lowercase());
            Ok(ChromeBookmark {
                id,
                name,
                url,
                date_added,
                folder_path,
                name_lower,
                url_lower,
                folder_path_lower,
            })
        })?;
        rows.collect::<Result<Vec<_>>>()
    }

    pub fn search_bookmarks_fts(
        &self,
        query: &str,
        tags: &[String],
        limit: usize,
    ) -> Result<Option<Vec<ChromeBookmark>>> {
        if !self.fts_enabled {
            return Ok(None);
        }

        let fts_query = build_fts_query(query);
        let fts_query = match fts_query {
            Some(q) => q,
            None => return Ok(None),
        };

        let normalized_tags = normalize_tags(tags);
        let mut params: Vec<&dyn ToSql> = Vec::new();
        let mut sql = String::from(
            "SELECT b.id, b.name, b.url, b.date_added, b.folder_path
             FROM bookmarks_fts
             JOIN bookmarks b ON b.id = bookmarks_fts.bookmark_id
             WHERE bookmarks_fts MATCH ?",
        );
        params.push(&fts_query);

        let tag_count_storage;
        if !normalized_tags.is_empty() {
            let placeholders: Vec<String> =
                normalized_tags.iter().map(|_| "?".to_string()).collect();
            sql.push_str(" AND b.id IN (SELECT bookmark_id FROM bookmark_tags WHERE tag IN (");
            sql.push_str(&placeholders.join(","));
            sql.push_str(") GROUP BY bookmark_id HAVING COUNT(DISTINCT tag) = ?)");

            for tag in &normalized_tags {
                params.push(tag as &dyn ToSql);
            }
            tag_count_storage = normalized_tags.len() as i64;
            params.push(&tag_count_storage);
        }

        sql.push_str(" ORDER BY bm25(bookmarks_fts) LIMIT ?");
        let limit_param = limit as i64;
        params.push(&limit_param);

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params.as_slice(), |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let url: String = row.get(2)?;
            let date_added: String = row.get(3)?;
            let folder_path: Option<String> = row.get(4)?;
            let name_lower = name.to_lowercase();
            let url_lower = url.to_lowercase();
            let folder_path_lower = folder_path.as_ref().map(|p| p.to_lowercase());
            Ok(ChromeBookmark {
                id,
                name,
                url,
                date_added,
                folder_path,
                name_lower,
                url_lower,
                folder_path_lower,
            })
        })?;
        let results = rows.collect::<Result<Vec<_>>>()?;
        Ok(Some(results))
    }

    /// 获取数据库中的书签总数
    pub fn get_bookmark_count(&self) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(DISTINCT bookmark_id) FROM bookmark_tags",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// 获取数据库中的tag总数
    pub fn get_tag_count(&self) -> Result<usize> {
        let count: usize =
            self.conn
                .query_row("SELECT COUNT(DISTINCT tag) FROM bookmark_tags", [], |row| {
                    row.get(0)
                })?;
        Ok(count)
    }
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn build_fts_query(query: &str) -> Option<String> {
    let mut parts = Vec::new();
    for token in query.split_whitespace() {
        let cleaned: String = token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        parts.push(format!("{}*", cleaned));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_bookmark(id: &str, name: &str, url: &str, folder: Option<&str>) -> ChromeBookmark {
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

    #[test]
    fn add_and_get_tags_normalizes_and_dedupes() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("tags.db");
        let manager = TagManager::new(db_path).expect("manager");

        let inserted = manager
            .add_tags(
                "id-1",
                &vec![" rust ".into(), "rust".into(), "  ".into(), "cli".into()],
            )
            .expect("add tags");
        assert_eq!(inserted, 2);

        let tags = manager.get_tags("id-1").expect("get tags");
        assert_eq!(tags, vec!["cli".to_string(), "rust".to_string()]);
    }

    #[test]
    fn find_bookmarks_by_tags_and_rename() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("tags.db");
        let manager = TagManager::new(db_path).expect("manager");

        manager
            .add_tags("id-1", &vec!["work".into(), "rust".into()])
            .expect("add tags");
        manager
            .add_tags("id-2", &vec!["work".into()])
            .expect("add tags");

        let matches = manager
            .find_bookmarks_by_tags(&vec![" work ".into(), "rust".into(), "work".into()])
            .expect("find");
        assert_eq!(matches, vec!["id-1".to_string()]);

        let updated = manager.rename_tag("work", "office").expect("rename");
        assert_eq!(updated, 2);

        let tags = manager.get_tags("id-2").expect("get tags");
        assert_eq!(tags, vec!["office".to_string()]);
    }

    #[test]
    fn get_tags_for_bookmarks_subset() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("tags.db");
        let manager = TagManager::new(db_path).expect("manager");

        manager
            .add_tags("id-1", &vec!["a".into(), "b".into()])
            .expect("add tags");
        manager
            .add_tags("id-2", &vec!["c".into()])
            .expect("add tags");

        let map = manager
            .get_tags_for_bookmarks(&vec!["id-2".into()])
            .expect("map");
        assert_eq!(map.get("id-2").unwrap(), &vec!["c".to_string()]);
        assert!(map.get("id-1").is_none());
    }

    #[test]
    fn get_tag_count_returns_distinct_total() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("tags.db");
        let manager = TagManager::new(db_path).expect("manager");

        manager
            .add_tags("id-1", &vec!["a".into(), "b".into()])
            .expect("add tags");
        manager
            .add_tags("id-2", &vec!["b".into(), "c".into()])
            .expect("add tags");

        let count = manager.get_tag_count().expect("count");
        assert_eq!(count, 3);
    }

    #[test]
    fn replace_bookmarks_builds_index_and_searches() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("tags.db");
        let manager = TagManager::new(db_path).expect("manager");

        let bookmarks = vec![
            sample_bookmark("1", "Rust Lang", "https://rust-lang.org", Some("Root")),
            sample_bookmark("2", "Example", "https://example.com", None),
        ];

        manager
            .replace_bookmarks(&bookmarks, "fp-1")
            .expect("replace");
        assert!(!manager.bookmarks_need_refresh("fp-1").expect("fingerprint"));
        assert_eq!(manager.get_total_bookmarks().expect("count"), 2);

        if manager.fts_enabled() {
            let results = manager
                .search_bookmarks_fts("rust", &[], 10)
                .expect("fts search")
                .expect("fts results");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, "1");
        }

        let found = manager
            .get_bookmark_by_id_or_url("https://example.com")
            .expect("by url");
        assert_eq!(found.unwrap().id, "2");
    }
}
