#![allow(dead_code)]

use crate::analysis::contract::{
    AnalysisDiagnostic, AnalysisMetadata, AnalysisSnapshot, AnalysisStatus, AnalysisSubtype,
};
use crate::models::ClipboardEntry;
use anyhow::{anyhow, Context, Result};
use sqlx::{sqlite::SqliteRow, Executor, Pool, Row, Sqlite};

pub async fn upsert_entry_analysis<'e, E>(
    executor: E,
    entry_id: &str,
    content_hash: &str,
    snapshot: &AnalysisSnapshot,
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let metadata_json = serde_json::to_string(&snapshot.metadata)?;
    let diagnostics_json = serde_json::to_string(&snapshot.diagnostics)?;

    sqlx::query(
        r#"
        INSERT INTO entry_analysis (
            entry_id,
            content_hash,
            contract_version,
            analysis_version,
            status,
            subtype,
            metadata_json,
            diagnostics_json,
            analyzed_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(entry_id) DO UPDATE SET
            content_hash = excluded.content_hash,
            contract_version = excluded.contract_version,
            analysis_version = excluded.analysis_version,
            status = excluded.status,
            subtype = excluded.subtype,
            metadata_json = excluded.metadata_json,
            diagnostics_json = excluded.diagnostics_json,
            analyzed_at = excluded.analyzed_at
        "#,
    )
    .bind(entry_id)
    .bind(content_hash)
    .bind(snapshot.contract_version)
    .bind(snapshot.analysis_version)
    .bind(snapshot.status.as_str())
    .bind(snapshot.subtype.as_str())
    .bind(metadata_json)
    .bind(diagnostics_json)
    .bind(snapshot.analyzed_at)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn load_entry_analysis_for_history(
    pool: &Pool<Sqlite>,
    limit: i32,
    offset: i32,
    search: Option<&str>,
) -> Result<Vec<ClipboardEntry>> {
    let rows = if let Some(search_term) = search {
        let pattern = format!("%{}%", search_term);
        sqlx::query(
            r#"
            SELECT
                e.id,
                e.content_hash,
                e.content_type,
                e.content_data,
                e.source_app,
                e.created_at,
                e.copy_count,
                e.file_path,
                e.is_favorite,
                e.content_subtype,
                e.metadata,
                e.app_bundle_id,
                a.contract_version AS analysis_contract_version,
                a.analysis_version AS analysis_analysis_version,
                a.status AS analysis_status,
                a.subtype AS analysis_subtype,
                a.metadata_json AS analysis_metadata_json,
                a.diagnostics_json AS analysis_diagnostics_json,
                a.analyzed_at AS analysis_analyzed_at
            FROM clipboard_entries e
            LEFT JOIN entry_analysis a ON a.entry_id = e.id
            WHERE e.content_data LIKE ? OR e.source_app LIKE ?
            ORDER BY e.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                e.id,
                e.content_hash,
                e.content_type,
                e.content_data,
                e.source_app,
                e.created_at,
                e.copy_count,
                e.file_path,
                e.is_favorite,
                e.content_subtype,
                e.metadata,
                e.app_bundle_id,
                a.contract_version AS analysis_contract_version,
                a.analysis_version AS analysis_analysis_version,
                a.status AS analysis_status,
                a.subtype AS analysis_subtype,
                a.metadata_json AS analysis_metadata_json,
                a.diagnostics_json AS analysis_diagnostics_json,
                a.analyzed_at AS analysis_analyzed_at
            FROM clipboard_entries e
            LEFT JOIN entry_analysis a ON a.entry_id = e.id
            ORDER BY e.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    rows.into_iter().map(map_history_row).collect()
}

pub async fn list_stale_entry_ids(
    pool: &Pool<Sqlite>,
    contract_version: i32,
    analysis_version: i32,
    batch_size: usize,
) -> Result<Vec<String>> {
    let rows = sqlx::query(
        r#"
        SELECT e.id
        FROM clipboard_entries e
        LEFT JOIN entry_analysis a ON a.entry_id = e.id
        WHERE e.content_type = 'text'
          AND e.content_data IS NOT NULL
          AND (
              a.entry_id IS NULL
              OR a.contract_version < ?
              OR a.analysis_version < ?
          )
        ORDER BY e.created_at DESC
        LIMIT ?
        "#,
    )
    .bind(contract_version)
    .bind(analysis_version)
    .bind(batch_size as i64)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| row.try_get("id").map_err(Into::into))
        .collect()
}

pub(crate) fn map_history_row(row: SqliteRow) -> Result<ClipboardEntry> {
    let mut entry = ClipboardEntry {
        id: row.try_get("id")?,
        content_hash: row.try_get("content_hash")?,
        content_type: row.try_get("content_type")?,
        content_data: row.try_get("content_data")?,
        source_app: row.try_get("source_app")?,
        created_at: row.try_get("created_at")?,
        copy_count: row.try_get("copy_count")?,
        file_path: row.try_get("file_path")?,
        is_favorite: row.try_get("is_favorite")?,
        content_subtype: row.try_get("content_subtype")?,
        metadata: row.try_get("metadata")?,
        app_bundle_id: row.try_get("app_bundle_id")?,
        analysis: None,
        retrieval: None,
    };

    if let Some(snapshot) = parse_analysis_snapshot(&row)? {
        entry.attach_analysis(snapshot);
    }

    Ok(entry)
}

pub(crate) fn parse_analysis_snapshot(row: &SqliteRow) -> Result<Option<AnalysisSnapshot>> {
    let subtype_raw: Option<String> = row.try_get("analysis_subtype")?;
    let status_raw: Option<String> = row.try_get("analysis_status")?;
    let metadata_json: Option<String> = row.try_get("analysis_metadata_json")?;
    let diagnostics_json: Option<String> = row.try_get("analysis_diagnostics_json")?;
    let contract_version: Option<i32> = row.try_get("analysis_contract_version")?;
    let analysis_version: Option<i32> = row.try_get("analysis_analysis_version")?;
    let analyzed_at: Option<i64> = row.try_get("analysis_analyzed_at")?;

    let Some(subtype_raw) = subtype_raw else {
        return Ok(None);
    };

    let status_raw = status_raw.context("analysis_status missing for joined analysis row")?;
    let metadata_json =
        metadata_json.context("analysis_metadata_json missing for joined analysis row")?;
    let diagnostics_json =
        diagnostics_json.context("analysis_diagnostics_json missing for joined analysis row")?;

    let subtype = AnalysisSubtype::from_str(&subtype_raw)
        .ok_or_else(|| anyhow!("unknown analysis subtype '{}'", subtype_raw))?;
    let status = AnalysisStatus::from_str(&status_raw)
        .ok_or_else(|| anyhow!("unknown analysis status '{}'", status_raw))?;
    let metadata: AnalysisMetadata = serde_json::from_str(&metadata_json)
        .with_context(|| format!("failed to parse analysis metadata for '{}'", subtype_raw))?;
    let diagnostics: Vec<AnalysisDiagnostic> =
        serde_json::from_str(&diagnostics_json).context("failed to parse analysis diagnostics")?;

    Ok(Some(AnalysisSnapshot {
        contract_version: contract_version
            .context("analysis_contract_version missing for joined analysis row")?,
        analysis_version: analysis_version
            .context("analysis_analysis_version missing for joined analysis row")?,
        status,
        subtype,
        metadata,
        diagnostics,
        analyzed_at: analyzed_at.context("analysis_analyzed_at missing for joined analysis row")?,
    }))
}
