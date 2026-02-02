use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Tag管理器
pub struct TagManager {
    conn: Connection,
}

impl TagManager {
    /// 创建或打开tag数据库
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

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

        Ok(TagManager { conn })
    }

    /// 为书签添加tag
    pub fn add_tag(&self, bookmark_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag) VALUES (?1, ?2)",
            params![bookmark_id, tag.trim()],
        )?;
        Ok(())
    }

    /// 为书签添加多个tags
    pub fn add_tags(&self, bookmark_id: &str, tags: &[String]) -> Result<()> {
        for tag in tags {
            self.add_tag(bookmark_id, tag)?;
        }
        Ok(())
    }

    /// 删除书签的某个tag
    pub fn remove_tag(&self, bookmark_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND tag = ?2",
            params![bookmark_id, tag],
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

        // 构建SQL查询：查找同时包含所有指定tags的书签
        let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
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
        let tag_count = tags.len();
        let mut params: Vec<&dyn rusqlite::ToSql> = tags.iter().map(|t| t as &dyn rusqlite::ToSql).collect();
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
        let params: Vec<&dyn rusqlite::ToSql> = tags.iter().map(|t| t as &dyn rusqlite::ToSql).collect();

        let bookmark_ids = stmt
            .query_map(params.as_slice(), |row| row.get(0))?
            .collect::<Result<Vec<String>>>()?;

        Ok(bookmark_ids)
    }

    /// 重命名tag
    pub fn rename_tag(&self, old_tag: &str, new_tag: &str) -> Result<usize> {
        let updated = self.conn.execute(
            "UPDATE bookmark_tags SET tag = ?1 WHERE tag = ?2",
            params![new_tag, old_tag],
        )?;
        Ok(updated)
    }

    /// 批量获取所有书签的tags（避免N+1查询）
    pub fn get_all_bookmark_tags(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut stmt = self.conn.prepare(
            "SELECT bookmark_id, tag FROM bookmark_tags ORDER BY bookmark_id, tag",
        )?;

        let mut result: HashMap<String, Vec<String>> = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (bookmark_id, tag) = row?;
            result.entry(bookmark_id).or_default().push(tag);
        }

        Ok(result)
    }

    /// 获取数据库中的书签总数
    pub fn get_bookmark_count(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(DISTINCT bookmark_id) FROM bookmark_tags", [], |row| {
                row.get(0)
            })?;
        Ok(count)
    }
}
