use crate::app_paths::AppPaths;
use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite, Transaction};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Database {
    pool: Pool<Sqlite>,
    #[cfg_attr(not(test), allow(dead_code))]
    db_path: PathBuf,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ExistingClipboardEntry {
    id: String,
    content_type: String,
    content_data: Option<String>,
    source_app: Option<String>,
    created_at: i64,
    copy_count: i32,
    file_path: Option<String>,
    is_favorite: bool,
    content_subtype: Option<String>,
    metadata: Option<String>,
    app_bundle_id: Option<String>,
}

impl Database {
    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn new() -> Result<Self> {
        Self::new_in(Arc::new(AppPaths::from_default_roots()?)).await
    }

    pub async fn new_in(paths: Arc<AppPaths>) -> Result<Self> {
        let db_path = paths.history_db_path();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;

        let db = Self { pool, db_path };
        db.init().await?;

        Ok(db)
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    #[cfg(test)]
    pub fn from_pool(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            db_path: PathBuf::new(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard_entries (
                id TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_data TEXT,
                source_app TEXT,
                created_at INTEGER NOT NULL,
                copy_count INTEGER DEFAULT 1,
                file_path TEXT,
                is_favorite INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON clipboard_entries(created_at DESC)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_content_hash ON clipboard_entries(content_hash)",
        )
        .execute(&self.pool)
        .await?;

        // 执行数据库迁移
        self.migrate().await?;

        let mut tx = self.pool.begin().await?;
        self.merge_existing_content_hash_duplicates(&mut tx).await?;
        tx.commit().await?;

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_clipboard_entries_content_hash_unique ON clipboard_entries(content_hash)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn migrate(&self) -> Result<()> {
        // 添加 content_subtype 字段（如果不存在）
        let _ = sqlx::query("ALTER TABLE clipboard_entries ADD COLUMN content_subtype TEXT")
            .execute(&self.pool)
            .await;

        // 添加 metadata 字段（如果不存在）
        let _ = sqlx::query("ALTER TABLE clipboard_entries ADD COLUMN metadata TEXT")
            .execute(&self.pool)
            .await;

        // 添加 app_bundle_id 字段（如果不存在）
        let _ = sqlx::query("ALTER TABLE clipboard_entries ADD COLUMN app_bundle_id TEXT")
            .execute(&self.pool)
            .await;

        // 为新字段创建索引
        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_content_subtype ON clipboard_entries(content_subtype)",
        )
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_app_bundle_id ON clipboard_entries(app_bundle_id)",
        )
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS entry_analysis (
                entry_id TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                contract_version INTEGER NOT NULL,
                analysis_version INTEGER NOT NULL,
                status TEXT NOT NULL,
                subtype TEXT NOT NULL,
                metadata_json TEXT NOT NULL CHECK (json_valid(metadata_json)),
                diagnostics_json TEXT NOT NULL CHECK (json_valid(diagnostics_json)),
                analyzed_at INTEGER NOT NULL,
                FOREIGN KEY (entry_id) REFERENCES clipboard_entries(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_entry_analysis_content_hash ON entry_analysis(content_hash)",
        )
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_entry_analysis_versions ON entry_analysis(analysis_version, contract_version)",
        )
        .execute(&self.pool)
        .await;

        Ok(())
    }

    async fn merge_existing_content_hash_duplicates(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<()> {
        let duplicate_hashes = sqlx::query_scalar::<_, String>(
            r#"
            SELECT content_hash
            FROM clipboard_entries
            GROUP BY content_hash
            HAVING COUNT(*) > 1
            "#,
        )
        .fetch_all(&mut **tx)
        .await?;

        for content_hash in duplicate_hashes {
            let rows = sqlx::query_as::<_, ExistingClipboardEntry>(
                r#"
                SELECT
                    id,
                    content_type,
                    content_data,
                    source_app,
                    created_at,
                    copy_count,
                    file_path,
                    is_favorite,
                    content_subtype,
                    metadata,
                    app_bundle_id
                FROM clipboard_entries
                WHERE content_hash = ?
                ORDER BY created_at DESC, id DESC
                "#,
            )
            .bind(&content_hash)
            .fetch_all(&mut **tx)
            .await?;

            let Some(survivor) = rows.first().cloned() else {
                continue;
            };

            if rows.len() < 2 {
                continue;
            }

            let copy_count = rows.iter().map(|row| row.copy_count).sum::<i32>();
            let is_favorite = rows.iter().any(|row| row.is_favorite);
            let content_data = preferred_value(survivor.content_data.clone(), &rows, |row| {
                row.content_data.clone()
            });
            let source_app = preferred_value(survivor.source_app.clone(), &rows, |row| {
                row.source_app.clone()
            });
            let content_subtype = preferred_value(survivor.content_subtype.clone(), &rows, |row| {
                row.content_subtype.clone()
            });
            let metadata =
                preferred_value(survivor.metadata.clone(), &rows, |row| row.metadata.clone());
            let app_bundle_id = preferred_value(survivor.app_bundle_id.clone(), &rows, |row| {
                row.app_bundle_id.clone()
            });
            let file_path = preferred_value(survivor.file_path.clone(), &rows, |row| {
                row.file_path.clone()
            });

            sqlx::query(
                r#"
                UPDATE clipboard_entries
                SET
                    content_type = ?,
                    content_data = ?,
                    source_app = ?,
                    created_at = ?,
                    copy_count = ?,
                    file_path = ?,
                    is_favorite = ?,
                    content_subtype = ?,
                    metadata = ?,
                    app_bundle_id = ?
                WHERE id = ?
                "#,
            )
            .bind(&survivor.content_type)
            .bind(&content_data)
            .bind(&source_app)
            .bind(survivor.created_at)
            .bind(copy_count)
            .bind(&file_path)
            .bind(is_favorite as i32)
            .bind(&content_subtype)
            .bind(&metadata)
            .bind(&app_bundle_id)
            .bind(&survivor.id)
            .execute(&mut **tx)
            .await?;

            sqlx::query("DELETE FROM clipboard_entries WHERE content_hash = ? AND id != ?")
                .bind(&content_hash)
                .bind(&survivor.id)
                .execute(&mut **tx)
                .await?;
        }

        Ok(())
    }
}

fn preferred_value<T, F>(
    survivor_value: Option<T>,
    ordered_rows: &[ExistingClipboardEntry],
    selector: F,
) -> Option<T>
where
    T: Clone,
    F: Fn(&ExistingClipboardEntry) -> Option<T>,
{
    survivor_value.or_else(|| ordered_rows.iter().skip(1).find_map(selector))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{
        load_entry_analysis_for_history, upsert_entry_analysis, TextAnalysisService,
    };
    use crate::models::{ClipboardEntry, ContentType};
    use sqlx::Row;
    use tempfile::TempDir;

    async fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_clipboard.db");
        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await.unwrap();
        let db = Database::from_pool(pool);
        db.init().await.unwrap();
        (db, temp_dir)
    }

    async fn assert_analysis_join_read_model(db: &Database) {
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("https://example.com/path?debug=true".to_string()),
            "analysis_merge_hash".to_string(),
            Some("Browser".to_string()),
            None,
        );
        entry.content_subtype = Some("plain_text".to_string());
        entry.metadata = Some(r#"{"legacy":true}"#.to_string());

        sqlx::query(
            r#"
            INSERT INTO clipboard_entries
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .bind(&entry.content_subtype)
        .bind(&entry.metadata)
        .execute(db.pool())
        .await
        .unwrap();

        let snapshot = TextAnalysisService::new().analyze("https://example.com/path?debug=true");
        upsert_entry_analysis(db.pool(), &entry.id, &entry.content_hash, &snapshot)
            .await
            .unwrap();

        let history = load_entry_analysis_for_history(db.pool(), 50, 0, None)
            .await
            .unwrap();
        let joined_entry = history
            .into_iter()
            .find(|candidate| candidate.id == entry.id)
            .unwrap();

        assert_eq!(joined_entry.content_subtype, Some("url".to_string()));
        assert!(joined_entry
            .metadata
            .as_ref()
            .is_some_and(|value| value.contains("url_parts")));
        assert_eq!(
            joined_entry
                .analysis
                .as_ref()
                .map(|value| value.subtype.as_str()),
            Some("url")
        );
    }

    #[tokio::test]
    async fn test_database_creation() {
        let (db, _temp_dir) = create_test_db().await;

        // Test that the table was created
        let result = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='clipboard_entries'",
        )
        .fetch_one(db.pool())
        .await;

        assert!(result.is_ok());
        let row = result.unwrap();
        assert_eq!(row.get::<String, _>("name"), "clipboard_entries");
    }

    #[tokio::test]
    async fn test_insert_text_entry() {
        let (db, _temp_dir) = create_test_db().await;

        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("Hello, world!".to_string()),
            "test_hash_123".to_string(),
            Some("TestApp".to_string()),
            None,
        );

        // Insert entry
        let result = sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .execute(db.pool())
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows_affected(), 1);

        // Verify entry exists
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(db.pool())
                .await
                .unwrap();

        assert_eq!(stored_entry.id, entry.id);
        assert_eq!(stored_entry.content_hash, entry.content_hash);
        assert_eq!(stored_entry.content_data, entry.content_data);
        assert_eq!(stored_entry.source_app, entry.source_app);
    }

    #[tokio::test]
    async fn test_insert_multiple_entries() {
        let (db, _temp_dir) = create_test_db().await;

        let entries: Vec<ClipboardEntry> = (0..10)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Content {}", i)),
                    format!("hash_{}", i),
                    Some(format!("App{}", i)),
                    None,
                )
            })
            .collect();

        // Insert all entries with different timestamps
        for (i, entry) in entries.iter().enumerate() {
            let result = sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at + i as i64) // Different timestamps
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(db.pool())
            .await;

            assert!(result.is_ok());
        }

        // Verify count
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(count, 10);

        // Verify all entries exist
        let stored_entries = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY created_at",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(stored_entries.len(), 10);
        for (i, stored_entry) in stored_entries.iter().enumerate() {
            assert_eq!(stored_entry.content_data, Some(format!("Content {}", i)));
        }
    }

    #[tokio::test]
    async fn test_update_entry() {
        let (db, _temp_dir) = create_test_db().await;

        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("Original content".to_string()),
            "update_hash".to_string(),
            Some("OriginalApp".to_string()),
            None,
        );

        // Insert entry
        sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .execute(db.pool())
        .await
        .unwrap();

        // Update entry
        entry.copy_count += 5;
        entry.is_favorite = true;

        let result = sqlx::query(
            "UPDATE clipboard_entries SET copy_count = ?, is_favorite = ? WHERE id = ?",
        )
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .bind(&entry.id)
        .execute(db.pool())
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows_affected(), 1);

        // Verify updates
        let updated_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(db.pool())
                .await
                .unwrap();

        assert_eq!(updated_entry.copy_count, 6);
        assert!(updated_entry.is_favorite);
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let (db, _temp_dir) = create_test_db().await;

        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("To be deleted".to_string()),
            "delete_hash".to_string(),
            Some("DeleteApp".to_string()),
            None,
        );

        // Insert entry
        sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .execute(db.pool())
        .await
        .unwrap();

        // Delete entry
        let result = sqlx::query("DELETE FROM clipboard_entries WHERE id = ?")
            .bind(&entry.id)
            .execute(db.pool())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows_affected(), 1);

        // Verify entry is gone
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries WHERE id = ?")
            .bind(&entry.id)
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_query_by_content_type() {
        let (db, _temp_dir) = create_test_db().await;

        let text_entries: Vec<ClipboardEntry> = (0..5)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Text content {}", i)),
                    format!("text_hash_{}", i),
                    Some("TextApp".to_string()),
                    None,
                )
            })
            .collect();

        let image_entries: Vec<ClipboardEntry> = (0..3)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Image,
                    None,
                    format!("image_hash_{}", i),
                    Some("ImageApp".to_string()),
                    Some(format!("imgs/image_{}.png", i)),
                )
            })
            .collect();

        // Insert all entries
        for entry in text_entries.iter().chain(image_entries.iter()) {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, file_path)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.file_path)
            .execute(db.pool())
            .await
            .unwrap();
        }

        // Query text entries
        let stored_text_entries = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_type = 'text'",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(stored_text_entries.len(), 5);
        for entry in &stored_text_entries {
            assert_eq!(entry.content_type, "text");
            assert!(entry.content_data.is_some());
        }

        // Query image entries
        let stored_image_entries = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_type = 'image'",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(stored_image_entries.len(), 3);
        for entry in &stored_image_entries {
            assert_eq!(entry.content_type, "image");
            assert!(entry.file_path.is_some());
        }
    }

    #[tokio::test]
    async fn test_query_with_pagination() {
        let (db, _temp_dir) = create_test_db().await;

        let entries: Vec<ClipboardEntry> = (0..20)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Paginated content {}", i)),
                    format!("page_hash_{}", i),
                    Some("PageApp".to_string()),
                    None,
                )
            })
            .collect();

        // Insert entries with different timestamps to ensure ordering
        for (i, entry) in entries.iter().enumerate() {
            let mut entry_with_timestamp = entry.clone();
            entry_with_timestamp.created_at += i as i64 * 1000; // Different timestamps

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry_with_timestamp.id)
            .bind(&entry_with_timestamp.content_hash)
            .bind(&entry_with_timestamp.content_type)
            .bind(&entry_with_timestamp.content_data)
            .bind(&entry_with_timestamp.source_app)
            .bind(entry_with_timestamp.created_at)
            .bind(entry_with_timestamp.copy_count)
            .bind(entry_with_timestamp.is_favorite)
            .execute(db.pool())
            .await
            .unwrap();
        }

        // Test pagination - first page
        let first_page = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY created_at DESC LIMIT 10 OFFSET 0",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(first_page.len(), 10);

        // Test pagination - second page
        let second_page = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY created_at DESC LIMIT 10 OFFSET 10",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(second_page.len(), 10);

        // Verify no overlap
        let first_page_ids: std::collections::HashSet<String> =
            first_page.iter().map(|e| e.id.clone()).collect();
        let second_page_ids: std::collections::HashSet<String> =
            second_page.iter().map(|e| e.id.clone()).collect();

        assert!(first_page_ids.is_disjoint(&second_page_ids));
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let (db, _temp_dir) = create_test_db().await;

        let entries = vec![
            ClipboardEntry::new(
                ContentType::Text,
                Some("The quick brown fox jumps".to_string()),
                "search_hash_1".to_string(),
                Some("SearchApp".to_string()),
                None,
            ),
            ClipboardEntry::new(
                ContentType::Text,
                Some("Python programming language".to_string()),
                "search_hash_2".to_string(),
                Some("SearchApp".to_string()),
                None,
            ),
            ClipboardEntry::new(
                ContentType::Text,
                Some("JavaScript and TypeScript".to_string()),
                "search_hash_3".to_string(),
                Some("SearchApp".to_string()),
                None,
            ),
        ];

        // Insert entries with different timestamps
        for (i, entry) in entries.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at + i as i64) // Different timestamps
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(db.pool())
            .await
            .unwrap();
        }

        // Search for "fox"
        let fox_results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_data LIKE ?",
        )
        .bind("%fox%")
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(fox_results.len(), 1);
        assert!(fox_results[0]
            .content_data
            .as_ref()
            .unwrap()
            .contains("fox"));

        // Search for "Script" (case insensitive)
        let script_results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_data LIKE ? COLLATE NOCASE",
        )
        .bind("%script%")
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(script_results.len(), 1); // JavaScript and TypeScript (one row)

        // Search with no results
        let no_results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_data LIKE ?",
        )
        .bind("%nonexistent%")
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(no_results.len(), 0);
    }

    #[tokio::test]
    async fn test_unicode_content_storage() {
        let (db, _temp_dir) = create_test_db().await;

        let unicode_contents = [
            "Hello 世界! 🌍",
            "Привет мир! 🇷🇺",
            "مرحبا بالعالم! 🇸🇦",
            "こんにちは世界! 🇯🇵",
            "🎉🎊🎈🎁🎂🎀🎃",
            "Émojis: 👨‍💻👩‍💻🧑‍💻",
        ];

        for (i, content) in unicode_contents.iter().enumerate() {
            let entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("unicode_hash_{}", i),
                Some("UnicodeApp".to_string()),
                None,
            );

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(db.pool())
            .await
            .unwrap();

            // Verify storage and retrieval
            let stored_entry =
                sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                    .bind(&entry.id)
                    .fetch_one(db.pool())
                    .await
                    .unwrap();

            assert_eq!(stored_entry.content_data.as_ref().unwrap(), content);
        }
    }

    #[tokio::test]
    async fn test_large_content_storage() {
        let (db, _temp_dir) = create_test_db().await;

        let large_content = "A".repeat(1_000_000); // 1MB of text
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some(large_content.clone()),
            "large_content_hash".to_string(),
            Some("LargeApp".to_string()),
            None,
        );

        let result = sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .execute(db.pool())
        .await;

        assert!(result.is_ok());

        // Verify retrieval
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(db.pool())
                .await
                .unwrap();

        assert_eq!(stored_entry.content_data.as_ref().unwrap().len(), 1_000_000);
        assert_eq!(*stored_entry.content_data.as_ref().unwrap(), large_content);
    }

    #[tokio::test]
    async fn test_index_performance() {
        let (db, _temp_dir) = create_test_db().await;

        // Insert many entries to test index effectiveness
        for i in 0..1000 {
            let entry = ClipboardEntry::new(
                ContentType::Text,
                Some(format!("Performance test content {}", i)),
                format!("perf_hash_{}", i),
                Some("PerfApp".to_string()),
                None,
            );

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(db.pool())
            .await
            .unwrap();
        }

        // Test indexed queries
        let start = std::time::Instant::now();

        // Query by created_at (should use idx_created_at)
        let _recent_entries = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        let created_at_duration = start.elapsed();

        let start = std::time::Instant::now();

        // Query by content_hash (should use idx_content_hash)
        let _hash_entry = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_hash = ?",
        )
        .bind("perf_hash_500")
        .fetch_all(db.pool())
        .await
        .unwrap();

        let content_hash_duration = start.elapsed();

        // These should complete quickly with proper indexing
        assert!(created_at_duration.as_millis() < 100);
        assert!(content_hash_duration.as_millis() < 50);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let (db, _temp_dir) = create_test_db().await;
        let db = std::sync::Arc::new(db);

        let mut handles = vec![];

        // Spawn multiple tasks that insert entries concurrently
        for thread_id in 0..10 {
            let db_clone = db.clone();
            let handle = tokio::spawn(async move {
                for i in 0..10 {
                    let entry = ClipboardEntry::new(
                        ContentType::Text,
                        Some(format!("Concurrent content {}-{}", thread_id, i)),
                        format!("concurrent_hash_{}_{}", thread_id, i),
                        Some("ConcurrentApp".to_string()),
                        None,
                    );

                    sqlx::query(
                        r#"
                        INSERT INTO clipboard_entries 
                        (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                        "#
                    )
                    .bind(&entry.id)
                    .bind(&entry.content_hash)
                    .bind(&entry.content_type)
                    .bind(&entry.content_data)
                    .bind(&entry.source_app)
                    .bind(entry.created_at)
                    .bind(entry.copy_count)
                    .bind(entry.is_favorite)
                    .execute(db_clone.pool())
                    .await
                    .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all entries were inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(count, 100); // 10 threads × 10 entries each
    }

    #[tokio::test]
    async fn test_migration_fields() {
        let (db, _temp_dir) = create_test_db().await;

        // Test that migration fields exist
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("Migration test".to_string()),
            "migration_hash".to_string(),
            Some("MigrationApp".to_string()),
            None,
        );

        // Set new fields
        entry.content_subtype = Some("url".to_string());
        entry.metadata = Some(r#"{"test": "metadata"}"#.to_string());
        entry.app_bundle_id = Some("com.test.migration".to_string());

        let result = sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata, app_bundle_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .bind(&entry.content_subtype)
        .bind(&entry.metadata)
        .bind(&entry.app_bundle_id)
        .execute(db.pool())
        .await;

        assert!(result.is_ok());

        // Verify retrieval
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(db.pool())
                .await
                .unwrap();

        assert_eq!(stored_entry.content_subtype, Some("url".to_string()));
        assert_eq!(
            stored_entry.metadata,
            Some(r#"{"test": "metadata"}"#.to_string())
        );
        assert_eq!(
            stored_entry.app_bundle_id,
            Some("com.test.migration".to_string())
        );
    }

    #[tokio::test]
    async fn test_database_reads_analysis_columns_after_migration() {
        let (db, _temp_dir) = create_test_db().await;

        let columns = sqlx::query("PRAGMA table_info(entry_analysis)")
            .fetch_all(db.pool())
            .await
            .unwrap();
        let column_names = columns
            .iter()
            .map(|row| row.get::<String, _>("name"))
            .collect::<Vec<_>>();

        assert!(column_names.contains(&"entry_id".to_string()));
        assert!(column_names.contains(&"content_hash".to_string()));
        assert!(column_names.contains(&"contract_version".to_string()));
        assert!(column_names.contains(&"analysis_version".to_string()));
        assert!(column_names.contains(&"status".to_string()));
        assert!(column_names.contains(&"subtype".to_string()));
        assert!(column_names.contains(&"metadata_json".to_string()));
        assert!(column_names.contains(&"diagnostics_json".to_string()));
        assert!(column_names.contains(&"analyzed_at".to_string()));

        let foreign_keys = sqlx::query("PRAGMA foreign_key_list(entry_analysis)")
            .fetch_all(db.pool())
            .await
            .unwrap();
        assert_eq!(foreign_keys.len(), 1);
        assert_eq!(
            foreign_keys[0].get::<String, _>("table"),
            "clipboard_entries"
        );
        assert_eq!(foreign_keys[0].get::<String, _>("on_delete"), "CASCADE");

        let indexes = sqlx::query("PRAGMA index_list(entry_analysis)")
            .fetch_all(db.pool())
            .await
            .unwrap();
        let index_names = indexes
            .iter()
            .map(|row| row.get::<String, _>("name"))
            .collect::<Vec<_>>();

        assert!(index_names.contains(&"idx_entry_analysis_content_hash".to_string()));
        assert!(index_names.contains(&"idx_entry_analysis_versions".to_string()));
    }

    #[tokio::test]
    async fn test_database_upserts_entry_analysis_rows() {
        let (db, _temp_dir) = create_test_db().await;

        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("https://example.com/docs".to_string()),
            "analysis_upsert_hash".to_string(),
            Some("Browser".to_string()),
            None,
        );

        sqlx::query(
            r#"
            INSERT INTO clipboard_entries
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.content_hash)
        .bind(&entry.content_type)
        .bind(&entry.content_data)
        .bind(&entry.source_app)
        .bind(entry.created_at)
        .bind(entry.copy_count)
        .bind(entry.is_favorite)
        .execute(db.pool())
        .await
        .unwrap();

        let snapshot = TextAnalysisService::new().analyze("https://example.com/docs");
        upsert_entry_analysis(db.pool(), &entry.id, &entry.content_hash, &snapshot)
            .await
            .unwrap();

        let row = sqlx::query(
            r#"
            SELECT
                content_hash,
                contract_version,
                analysis_version,
                status,
                subtype,
                json_valid(metadata_json) AS metadata_valid,
                json_valid(diagnostics_json) AS diagnostics_valid
            FROM entry_analysis
            WHERE entry_id = ?
            "#,
        )
        .bind(&entry.id)
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(row.get::<String, _>("content_hash"), entry.content_hash);
        assert_eq!(
            row.get::<String, _>("status"),
            snapshot.status.as_str().to_string()
        );
        assert_eq!(
            row.get::<String, _>("subtype"),
            snapshot.subtype.as_str().to_string()
        );
        assert_eq!(
            row.get::<i32, _>("contract_version"),
            snapshot.contract_version
        );
        assert_eq!(
            row.get::<i32, _>("analysis_version"),
            snapshot.analysis_version
        );
        assert_eq!(row.get::<i64, _>("metadata_valid"), 1);
        assert_eq!(row.get::<i64, _>("diagnostics_valid"), 1);
    }

    #[tokio::test]
    async fn test_database_merges_analysis_fields() {
        let (db, _temp_dir) = create_test_db().await;
        assert_analysis_join_read_model(&db).await;
    }

    #[tokio::test]
    async fn test_entry_analysis_repository_upsert_and_join_read_model() {
        let (db, _temp_dir) = create_test_db().await;
        assert_analysis_join_read_model(&db).await;
    }
}
