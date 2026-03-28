use super::repository::{list_stale_entry_ids, upsert_entry_analysis};
use super::service::TextAnalysisService;
use super::{ANALYSIS_CONTRACT_VERSION, TEXT_ANALYSIS_VERSION};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};

pub const DEFAULT_REBUILD_BATCH_SIZE: usize = 250;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RebuildEntryAnalysisResult {
    pub scanned: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Debug)]
struct RebuildCandidate {
    id: String,
    content_hash: String,
    content_data: Option<String>,
}

#[derive(Debug, Default)]
pub struct EntryAnalysisRebuilder {
    service: TextAnalysisService,
}

impl EntryAnalysisRebuilder {
    pub fn new() -> Self {
        Self {
            service: TextAnalysisService::new(),
        }
    }

    pub async fn rebuild(
        &self,
        pool: &Pool<Sqlite>,
        batch_size: Option<usize>,
    ) -> Result<RebuildEntryAnalysisResult> {
        let batch_size = batch_size.unwrap_or(DEFAULT_REBUILD_BATCH_SIZE).max(1);
        let skipped =
            count_stale_non_text_entries(pool, ANALYSIS_CONTRACT_VERSION, TEXT_ANALYSIS_VERSION)
                .await?;
        let stale_entry_ids = list_stale_entry_ids(
            pool,
            ANALYSIS_CONTRACT_VERSION,
            TEXT_ANALYSIS_VERSION,
            batch_size,
        )
        .await?;
        let candidates = load_rebuild_candidates(pool, &stale_entry_ids).await?;

        let mut result = RebuildEntryAnalysisResult {
            scanned: candidates.len() + skipped,
            updated: 0,
            skipped,
            failed: 0,
        };

        for candidate in candidates {
            let Some(content_data) = candidate.content_data.as_deref() else {
                result.skipped += 1;
                continue;
            };

            let snapshot = self.service.analyze(content_data);
            match upsert_entry_analysis(pool, &candidate.id, &candidate.content_hash, &snapshot)
                .await
            {
                Ok(_) => {
                    result.updated += 1;
                }
                Err(error) => {
                    log::error!(
                        "[analysis::rebuild] failed to upsert analysis for '{}': {}",
                        candidate.id,
                        error
                    );
                    result.failed += 1;
                }
            }
        }

        Ok(result)
    }
}

async fn count_stale_non_text_entries(
    pool: &Pool<Sqlite>,
    contract_version: i32,
    analysis_version: i32,
) -> Result<usize> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM clipboard_entries e
        LEFT JOIN entry_analysis a ON a.entry_id = e.id
        WHERE (
                a.entry_id IS NULL
                OR a.contract_version < ?
                OR a.analysis_version < ?
            )
          AND (
                e.content_type != 'text'
                OR e.content_data IS NULL
            )
        "#,
    )
    .bind(contract_version)
    .bind(analysis_version)
    .fetch_one(pool)
    .await?;

    Ok(count.max(0) as usize)
}

async fn load_rebuild_candidates(
    pool: &Pool<Sqlite>,
    entry_ids: &[String],
) -> Result<Vec<RebuildCandidate>> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT id, content_hash, content_data FROM clipboard_entries WHERE id IN (",
    );
    let mut separated = builder.separated(", ");
    for entry_id in entry_ids {
        separated.push_bind(entry_id);
    }
    separated.push_unseparated(") ORDER BY created_at DESC");

    let rows = builder.build().fetch_all(pool).await?;

    rows.into_iter()
        .map(|row| {
            Ok(RebuildCandidate {
                id: row.try_get("id")?,
                content_hash: row.try_get("content_hash")?,
                content_data: row.try_get("content_data")?,
            })
        })
        .collect()
}
