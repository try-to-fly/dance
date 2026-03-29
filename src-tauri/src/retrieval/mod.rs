use crate::analysis::contract::{AnalysisMetadata, UrlMetadata};
use crate::analysis::repository::map_history_row;
use crate::models::{ClipboardEntry, ClipboardRetrievalMatch, ClipboardRetrievalMatchKind};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, QueryBuilder, Row, Sqlite, SqliteConnection};
use std::cmp::Ordering;
use std::collections::HashSet;

const DEFAULT_HISTORY_LIMIT: i32 = 50;
const DEFAULT_HISTORY_OFFSET: i32 = 0;
const SEARCH_CANDIDATE_MULTIPLIER: i32 = 4;
const MIN_SEARCH_CANDIDATES: i32 = 120;
const MAX_SEARCH_CANDIDATES: i32 = 320;
const FALLBACK_FUZZY_CANDIDATES: i32 = 400;
const MAX_SEARCH_TEXT_LENGTH: usize = 4096;
const MAX_JSON_KEYS: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClipboardHistoryQuery {
    pub text: Option<String>,
    pub selected_type: Option<String>,
    pub source_app: Option<String>,
    pub favorites_only: Option<bool>,
    pub recency_days: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SearchStructuredTerm {
    kind: ClipboardRetrievalMatchKind,
    value: String,
}

#[derive(Debug, Clone)]
struct SearchDocument {
    content_hash: String,
    content_type: String,
    content_subtype: Option<String>,
    source_app: Option<String>,
    is_favorite: bool,
    created_at: i64,
    search_text: String,
    structured_terms: Vec<SearchStructuredTerm>,
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    entry: ClipboardEntry,
    document: SearchDocument,
}

#[derive(Debug, Clone)]
struct RankedCandidate {
    entry: ClipboardEntry,
    score: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchIndexRebuildResult {
    pub reindexed: usize,
    pub failed: usize,
}

pub async fn upsert_entry_search_document(
    tx: &mut SqliteConnection,
    entry: &ClipboardEntry,
) -> Result<()> {
    let document = build_search_document(entry);
    let structured_terms_json = serde_json::to_string(&document.structured_terms)?;
    let updated_at = Utc::now().timestamp_millis();

    sqlx::query(
        r#"
        INSERT INTO entry_search_documents (
            entry_id,
            content_hash,
            content_type,
            content_subtype,
            source_app,
            is_favorite,
            created_at,
            search_text,
            structured_terms_json,
            updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(entry_id) DO UPDATE SET
            content_hash = excluded.content_hash,
            content_type = excluded.content_type,
            content_subtype = excluded.content_subtype,
            source_app = excluded.source_app,
            is_favorite = excluded.is_favorite,
            created_at = excluded.created_at,
            search_text = excluded.search_text,
            structured_terms_json = excluded.structured_terms_json,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&entry.id)
    .bind(&document.content_hash)
    .bind(&document.content_type)
    .bind(&document.content_subtype)
    .bind(&document.source_app)
    .bind(document.is_favorite as i32)
    .bind(document.created_at)
    .bind(&document.search_text)
    .bind(&structured_terms_json)
    .bind(updated_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM entry_search_fts WHERE entry_id = ?")
        .bind(&entry.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO entry_search_fts (entry_id, search_text) VALUES (?, ?)")
        .bind(&entry.id)
        .bind(&document.search_text)
        .execute(&mut *tx)
        .await?;

    Ok(())
}

pub async fn refresh_favorite_search_document(pool: &Pool<Sqlite>, entry_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE entry_search_documents
        SET is_favorite = (
                SELECT is_favorite
                FROM clipboard_entries
                WHERE id = ?
            ),
            updated_at = ?
        WHERE entry_id = ?
        "#,
    )
    .bind(entry_id)
    .bind(Utc::now().timestamp_millis())
    .bind(entry_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn search_clipboard_history(
    pool: &Pool<Sqlite>,
    query: ClipboardHistoryQuery,
) -> Result<Vec<ClipboardEntry>> {
    let limit = query.limit.unwrap_or(DEFAULT_HISTORY_LIMIT).max(1);
    let offset = query.offset.unwrap_or(DEFAULT_HISTORY_OFFSET).max(0);
    let selected_type = normalize_optional(query.selected_type.as_deref());
    let source_app = normalize_optional(query.source_app.as_deref());
    let recency_after = query
        .recency_days
        .filter(|days| *days > 0)
        .map(|days| Utc::now().timestamp_millis() - days * 24 * 60 * 60 * 1000);
    let normalized_text = normalize_optional(query.text.as_deref());

    if normalized_text.is_none() {
        return load_recent_history(
            pool,
            limit,
            offset,
            selected_type,
            source_app,
            query.favorites_only,
            recency_after,
        )
        .await;
    }

    let search_text = normalized_text.unwrap_or_default();
    let tokens = tokenize_query(&search_text);
    if tokens.is_empty() {
        return load_recent_history(
            pool,
            limit,
            offset,
            selected_type,
            source_app,
            query.favorites_only,
            recency_after,
        )
        .await;
    }

    let candidate_target = ((limit + offset) * SEARCH_CANDIDATE_MULTIPLIER)
        .clamp(MIN_SEARCH_CANDIDATES, MAX_SEARCH_CANDIDATES);
    let mut candidates = load_fts_candidates(
        pool,
        &tokens,
        candidate_target,
        selected_type.as_deref(),
        source_app.as_deref(),
        query.favorites_only,
        recency_after,
    )
    .await?;

    if candidates.len() < (limit + offset) as usize {
        let mut fallback_candidates = load_recent_candidates(
            pool,
            FALLBACK_FUZZY_CANDIDATES,
            selected_type.as_deref(),
            source_app.as_deref(),
            query.favorites_only,
            recency_after,
        )
        .await?;

        let seen_ids: HashSet<String> = candidates
            .iter()
            .map(|candidate| candidate.entry.id.clone())
            .collect();
        fallback_candidates.retain(|candidate| !seen_ids.contains(&candidate.entry.id));
        candidates.extend(fallback_candidates);
    }

    let mut ranked = candidates
        .into_iter()
        .filter_map(|candidate| rank_candidate(candidate, &search_text, &tokens))
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.entry.created_at.cmp(&left.entry.created_at))
    });

    Ok(ranked
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|candidate| candidate.entry)
        .collect())
}

pub async fn rebuild_search_documents(pool: &Pool<Sqlite>) -> Result<SearchIndexRebuildResult> {
    let rows = sqlx::query(
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
        "#,
    )
    .fetch_all(pool)
    .await?;

    let entries = rows
        .into_iter()
        .map(map_history_row)
        .collect::<Result<Vec<_>>>()?;
    let mut connection = pool.acquire().await?;
    let mut result = SearchIndexRebuildResult::default();

    for entry in entries {
        match upsert_entry_search_document(&mut connection, &entry).await {
            Ok(_) => {
                result.reindexed += 1;
            }
            Err(error) => {
                log::error!(
                    "[retrieval::rebuild] failed to rebuild search document for '{}': {}",
                    entry.id,
                    error
                );
                result.failed += 1;
            }
        }
    }

    Ok(result)
}

async fn load_recent_history(
    pool: &Pool<Sqlite>,
    limit: i32,
    offset: i32,
    selected_type: Option<String>,
    source_app: Option<String>,
    favorites_only: Option<bool>,
    recency_after: Option<i64>,
) -> Result<Vec<ClipboardEntry>> {
    let mut builder = QueryBuilder::new(
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
        WHERE 1 = 1
        "#,
    );

    append_filter_clause(
        &mut builder,
        selected_type.as_deref(),
        source_app.as_deref(),
        favorites_only,
        recency_after,
    );
    builder.push(" ORDER BY e.created_at DESC LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let rows = builder.build().fetch_all(pool).await?;
    rows.into_iter().map(map_history_row).collect()
}

async fn load_fts_candidates(
    pool: &Pool<Sqlite>,
    tokens: &[String],
    limit: i32,
    selected_type: Option<&str>,
    source_app: Option<&str>,
    favorites_only: Option<bool>,
    recency_after: Option<i64>,
) -> Result<Vec<SearchCandidate>> {
    let fts_query = build_fts_query(tokens);
    let mut builder = QueryBuilder::new(
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
            a.analyzed_at AS analysis_analyzed_at,
            d.search_text AS retrieval_search_text,
            d.structured_terms_json AS retrieval_structured_terms_json,
            bm25(entry_search_fts) AS retrieval_bm25
        FROM entry_search_fts
        JOIN entry_search_documents d ON d.entry_id = entry_search_fts.entry_id
        JOIN clipboard_entries e ON e.id = d.entry_id
        LEFT JOIN entry_analysis a ON a.entry_id = e.id
        WHERE entry_search_fts MATCH "#,
    );
    builder.push_bind(fts_query);

    append_filter_clause(
        &mut builder,
        selected_type,
        source_app,
        favorites_only,
        recency_after,
    );
    builder.push(" ORDER BY bm25(entry_search_fts), d.created_at DESC LIMIT ");
    builder.push_bind(limit);

    let rows = builder.build().fetch_all(pool).await?;
    rows.into_iter().map(map_search_candidate_row).collect()
}

async fn load_recent_candidates(
    pool: &Pool<Sqlite>,
    limit: i32,
    selected_type: Option<&str>,
    source_app: Option<&str>,
    favorites_only: Option<bool>,
    recency_after: Option<i64>,
) -> Result<Vec<SearchCandidate>> {
    let mut builder = QueryBuilder::new(
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
            a.analyzed_at AS analysis_analyzed_at,
            d.search_text AS retrieval_search_text,
            d.structured_terms_json AS retrieval_structured_terms_json
        FROM clipboard_entries e
        LEFT JOIN entry_analysis a ON a.entry_id = e.id
        LEFT JOIN entry_search_documents d ON d.entry_id = e.id
        WHERE 1 = 1
        "#,
    );
    append_filter_clause(
        &mut builder,
        selected_type,
        source_app,
        favorites_only,
        recency_after,
    );
    builder.push(" ORDER BY e.created_at DESC LIMIT ");
    builder.push_bind(limit);

    let rows = builder.build().fetch_all(pool).await?;
    rows.into_iter().map(map_search_candidate_row).collect()
}

fn append_filter_clause<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    selected_type: Option<&'a str>,
    source_app: Option<&'a str>,
    favorites_only: Option<bool>,
    recency_after: Option<i64>,
) {
    if let Some(selected_type) = selected_type {
        match selected_type {
            "text" => {
                builder.push(" AND e.content_type = 'text'");
            }
            "image" => {
                builder.push(" AND e.content_type = 'image'");
            }
            "file" => {
                builder.push(" AND e.content_type = 'file'");
            }
            value if value.starts_with("text:") => {
                builder.push(
                    " AND e.content_type = 'text' AND COALESCE(a.subtype, e.content_subtype) = ",
                );
                builder.push_bind(value.trim_start_matches("text:"));
            }
            _ => {}
        }
    }

    if let Some(source_app) = source_app {
        builder.push(" AND LOWER(COALESCE(e.source_app, '')) = ");
        builder.push_bind(source_app);
    }

    if favorites_only.unwrap_or(false) {
        builder.push(" AND e.is_favorite = 1");
    }

    if let Some(recency_after) = recency_after {
        builder.push(" AND e.created_at >= ");
        builder.push_bind(recency_after);
    }
}

fn map_search_candidate_row(row: sqlx::sqlite::SqliteRow) -> Result<SearchCandidate> {
    let search_text = row
        .try_get::<Option<String>, _>("retrieval_search_text")?
        .unwrap_or_default();
    let structured_terms = row
        .try_get::<Option<String>, _>("retrieval_structured_terms_json")?
        .and_then(|value| serde_json::from_str::<Vec<SearchStructuredTerm>>(&value).ok())
        .unwrap_or_default();
    let entry = map_history_row(row)?;

    let document = if search_text.trim().is_empty() {
        build_search_document(&entry)
    } else {
        SearchDocument {
            content_hash: entry.content_hash.clone(),
            content_type: entry.content_type.clone(),
            content_subtype: entry.content_subtype.clone(),
            source_app: entry.source_app.clone(),
            is_favorite: entry.is_favorite,
            created_at: entry.created_at,
            search_text,
            structured_terms,
        }
    };

    Ok(SearchCandidate { entry, document })
}

fn build_search_document(entry: &ClipboardEntry) -> SearchDocument {
    let mut structured_terms = Vec::new();
    let mut structured_seen = HashSet::new();
    let subtype = entry
        .analysis
        .as_ref()
        .map(|analysis| analysis.subtype.as_str().to_string())
        .or_else(|| entry.content_subtype.clone());

    if let Some(source_app) = entry.source_app.as_deref() {
        push_structured_term(
            &mut structured_terms,
            &mut structured_seen,
            ClipboardRetrievalMatchKind::SourceApp,
            source_app,
        );
    }

    if let Some(analysis) = entry.analysis.as_ref() {
        match &analysis.metadata {
            AnalysisMetadata::Url(metadata) => {
                collect_url_terms(metadata, &mut structured_terms, &mut structured_seen)
            }
            AnalysisMetadata::Json(_) => collect_json_terms(
                entry.content_data.as_deref(),
                &mut structured_terms,
                &mut structured_seen,
            ),
            AnalysisMetadata::Command(metadata) => {
                if let Some(command_name) = metadata.command_name.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::CommandName,
                        command_name,
                    );
                }
                if let Some(shell_family) = metadata.shell_family.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::Metadata,
                        shell_family,
                    );
                }
            }
            AnalysisMetadata::Color(metadata) => {
                if let Some(value) = metadata.hex.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::ColorValue,
                        value,
                    );
                }
                if let Some(value) = metadata.rgb.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::ColorValue,
                        value,
                    );
                }
                if let Some(value) = metadata.rgba.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::ColorValue,
                        value,
                    );
                }
                if let Some(value) = metadata.hsl.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::ColorValue,
                        value,
                    );
                }
            }
            AnalysisMetadata::Email(metadata) => {
                push_structured_term(
                    &mut structured_terms,
                    &mut structured_seen,
                    ClipboardRetrievalMatchKind::Metadata,
                    &metadata.domain,
                );
                push_structured_term(
                    &mut structured_terms,
                    &mut structured_seen,
                    ClipboardRetrievalMatchKind::Metadata,
                    &metadata.local_part,
                );
            }
            AnalysisMetadata::Timestamp(metadata) => {
                if let Some(value) = metadata.iso8601.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::Metadata,
                        value,
                    );
                }
                if let Some(value) = metadata.date_string.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::Metadata,
                        value,
                    );
                }
                if let Some(unix_ms) = metadata.unix_ms {
                    let unix_ms_string = unix_ms.to_string();
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::Metadata,
                        &unix_ms_string,
                    );
                }
            }
            AnalysisMetadata::Code(metadata) => {
                if let Some(language) = metadata.detected_language.as_deref() {
                    push_structured_term(
                        &mut structured_terms,
                        &mut structured_seen,
                        ClipboardRetrievalMatchKind::Metadata,
                        language,
                    );
                }
            }
            AnalysisMetadata::IpAddress(_)
            | AnalysisMetadata::PlainText(_)
            | AnalysisMetadata::Markdown(_)
            | AnalysisMetadata::Base64(_) => {}
        }
    } else if matches!(subtype.as_deref(), Some("url")) {
        collect_url_terms_from_text(
            entry.content_data.as_deref(),
            &mut structured_terms,
            &mut structured_seen,
        );
    } else if matches!(subtype.as_deref(), Some("json")) {
        collect_json_terms(
            entry.content_data.as_deref(),
            &mut structured_terms,
            &mut structured_seen,
        );
    }

    let raw_content = normalize_text(entry.content_data.as_deref().unwrap_or_default());
    let structured_text = structured_terms
        .iter()
        .map(|term| term.value.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let search_text = [
        entry.source_app.clone().unwrap_or_default(),
        subtype.clone().unwrap_or_default(),
        structured_text,
        raw_content,
    ]
    .join(" ");

    SearchDocument {
        content_hash: entry.content_hash.clone(),
        content_type: entry.content_type.clone(),
        content_subtype: subtype,
        source_app: entry.source_app.clone(),
        is_favorite: entry.is_favorite,
        created_at: entry.created_at,
        search_text: trim_to_max_length(&search_text, MAX_SEARCH_TEXT_LENGTH),
        structured_terms,
    }
}

fn rank_candidate(
    candidate: SearchCandidate,
    normalized_query: &str,
    query_tokens: &[String],
) -> Option<RankedCandidate> {
    let mut entry = candidate.entry;
    let document = candidate.document;
    let mut best_score = 0.0_f64;
    let mut best_match = None;
    let raw_content = normalize_text(entry.content_data.as_deref().unwrap_or_default());

    if let Some(source_app) = entry.source_app.as_deref() {
        let normalized_app = normalize_text(source_app);
        let score = score_term_against_query(&normalized_app, query_tokens);
        if score > best_score {
            best_score = score + 10.0;
            best_match = Some(ClipboardRetrievalMatch {
                score: best_score,
                match_kind: ClipboardRetrievalMatchKind::SourceApp,
                label: "Source app".to_string(),
                snippet: Some(source_app.to_string()),
                matched_terms: query_tokens.to_vec(),
            });
        }
    }

    for term in &document.structured_terms {
        let normalized_value = normalize_text(&term.value);
        let score = score_term_against_query(&normalized_value, query_tokens);
        if score > best_score {
            best_score = score + 18.0;
            best_match = Some(ClipboardRetrievalMatch {
                score: best_score,
                match_kind: term.kind.clone(),
                label: retrieval_label(&term.kind).to_string(),
                snippet: Some(term.value.clone()),
                matched_terms: query_tokens.to_vec(),
            });
        }
    }

    let content_score = score_term_against_query(&raw_content, query_tokens);
    if content_score > best_score {
        best_score = content_score;
        best_match = Some(ClipboardRetrievalMatch {
            score: best_score,
            match_kind: if content_score < 40.0 {
                ClipboardRetrievalMatchKind::Fuzzy
            } else {
                ClipboardRetrievalMatchKind::Content
            },
            label: if content_score < 40.0 {
                "Fuzzy content".to_string()
            } else {
                "Content".to_string()
            },
            snippet: build_content_snippet(
                entry.content_data.as_deref(),
                normalized_query,
                query_tokens,
            ),
            matched_terms: query_tokens.to_vec(),
        });
    }

    if let Some(mut retrieval) = best_match {
        let final_score =
            best_score + favorite_bonus(entry.is_favorite) + recency_bonus(entry.created_at);
        retrieval.score = final_score;
        entry.retrieval = Some(retrieval);
        return Some(RankedCandidate {
            entry,
            score: final_score,
        });
    }

    None
}

fn score_term_against_query(term: &str, query_tokens: &[String]) -> f64 {
    if term.is_empty() {
        return 0.0;
    }

    let joined_query = query_tokens.join(" ");
    if term == joined_query {
        return 96.0;
    }
    if term.contains(&joined_query) {
        return 82.0;
    }

    let mut total = 0.0;
    for token in query_tokens {
        if term == token {
            total += 40.0;
            continue;
        }
        if term.starts_with(token) {
            total += 30.0;
            continue;
        }
        if term.contains(token) {
            total += 22.0;
            continue;
        }
        if subsequence_score(term, token) > 0.0 {
            total += 14.0;
        }
    }

    total
}

fn subsequence_score(term: &str, token: &str) -> f64 {
    if token.len() < 2 || term.len() < token.len() {
        return 0.0;
    }

    let mut token_chars = token.chars();
    let mut current = token_chars.next();
    if current.is_none() {
        return 0.0;
    }

    let mut matched = 0;
    for ch in term.chars() {
        if Some(ch) == current {
            matched += 1;
            current = token_chars.next();
            if current.is_none() {
                break;
            }
        }
    }

    if matched == token.chars().count() {
        12.0
    } else {
        0.0
    }
}

fn favorite_bonus(is_favorite: bool) -> f64 {
    if is_favorite {
        2.5
    } else {
        0.0
    }
}

fn recency_bonus(created_at: i64) -> f64 {
    let age_ms = (Utc::now().timestamp_millis() - created_at).max(0);
    let age_days = age_ms as f64 / (24.0 * 60.0 * 60.0 * 1000.0);
    (30.0 - age_days).clamp(0.0, 30.0) / 10.0
}

fn build_content_snippet(
    content: Option<&str>,
    normalized_query: &str,
    query_tokens: &[String],
) -> Option<String> {
    let content = content?.trim();
    if content.is_empty() {
        return None;
    }

    let normalized = normalize_text(content);
    let needle = query_tokens
        .iter()
        .find_map(|token| normalized.find(token).map(|index| (token, index)))
        .or_else(|| {
            normalized
                .find(normalized_query)
                .map(|index| (&query_tokens[0], index))
        })?;

    let start_chars = normalized[..needle.1].chars().count();
    let start = start_chars.saturating_sub(24);
    let end = (start_chars + needle.0.chars().count() + 48).min(content.chars().count());
    let snippet = content
        .chars()
        .skip(start)
        .take(end - start)
        .collect::<String>();
    Some(trim_to_max_length(&snippet, 96))
}

fn retrieval_label(kind: &ClipboardRetrievalMatchKind) -> &'static str {
    match kind {
        ClipboardRetrievalMatchKind::Content => "Content",
        ClipboardRetrievalMatchKind::SourceApp => "Source app",
        ClipboardRetrievalMatchKind::UrlHost => "URL host",
        ClipboardRetrievalMatchKind::UrlPath => "URL path",
        ClipboardRetrievalMatchKind::UrlQuery => "URL query",
        ClipboardRetrievalMatchKind::JsonKey => "JSON key",
        ClipboardRetrievalMatchKind::CommandName => "Command",
        ClipboardRetrievalMatchKind::ColorValue => "Color",
        ClipboardRetrievalMatchKind::Metadata => "Metadata",
        ClipboardRetrievalMatchKind::Fuzzy => "Fuzzy",
    }
}

fn build_fts_query(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| format!("\"{}\"*", escape_fts_token(token)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn escape_fts_token(token: &str) -> String {
    token
        .chars()
        .map(|ch| if ch == '"' { ' ' } else { ch })
        .collect::<String>()
}

fn collect_url_terms(
    metadata: &UrlMetadata,
    terms: &mut Vec<SearchStructuredTerm>,
    seen: &mut HashSet<String>,
) {
    push_structured_term(
        terms,
        seen,
        ClipboardRetrievalMatchKind::UrlHost,
        &metadata.host,
    );
    push_structured_term(
        terms,
        seen,
        ClipboardRetrievalMatchKind::UrlPath,
        &metadata.path,
    );
    push_structured_term(
        terms,
        seen,
        ClipboardRetrievalMatchKind::Metadata,
        &metadata.protocol,
    );

    for param in &metadata.query_params {
        push_structured_term(
            terms,
            seen,
            ClipboardRetrievalMatchKind::UrlQuery,
            &param.key,
        );
        push_structured_term(
            terms,
            seen,
            ClipboardRetrievalMatchKind::UrlQuery,
            &param.value,
        );
    }
}

fn collect_url_terms_from_text(
    content: Option<&str>,
    terms: &mut Vec<SearchStructuredTerm>,
    seen: &mut HashSet<String>,
) {
    let Some(content) = content else {
        return;
    };
    let Ok(parsed) = url::Url::parse(content.trim()) else {
        return;
    };

    let metadata = UrlMetadata {
        protocol: parsed.scheme().to_string(),
        host: parsed.host_str().unwrap_or_default().to_string(),
        path: parsed.path().to_string(),
        query_params: parsed
            .query_pairs()
            .map(|(key, value)| crate::analysis::contract::UrlQueryParam {
                key: key.to_string(),
                value: value.to_string(),
            })
            .collect(),
    };
    collect_url_terms(&metadata, terms, seen);
}

fn collect_json_terms(
    content: Option<&str>,
    terms: &mut Vec<SearchStructuredTerm>,
    seen: &mut HashSet<String>,
) {
    let Some(content) = content else {
        return;
    };
    let Ok(value) = serde_json::from_str::<Value>(content) else {
        return;
    };

    let mut keys = Vec::new();
    collect_json_keys(&value, String::new(), &mut keys);
    for key in keys.into_iter().take(MAX_JSON_KEYS) {
        push_structured_term(terms, seen, ClipboardRetrievalMatchKind::JsonKey, &key);
    }
}

fn collect_json_keys(value: &Value, prefix: String, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let next_prefix = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{}.{}", prefix, key)
                };
                out.push(next_prefix.clone());
                collect_json_keys(child, next_prefix, out);
            }
        }
        Value::Array(items) => {
            for item in items.iter().take(8) {
                collect_json_keys(item, prefix.clone(), out);
            }
        }
        _ => {}
    }
}

fn push_structured_term(
    terms: &mut Vec<SearchStructuredTerm>,
    seen: &mut HashSet<String>,
    kind: ClipboardRetrievalMatchKind,
    value: &str,
) {
    let normalized = normalize_text(value);
    if normalized.is_empty() {
        return;
    }

    let key = format!("{:?}:{}", kind, normalized);
    if seen.insert(key) {
        terms.push(SearchStructuredTerm {
            kind,
            value: trim_to_max_length(value.trim(), 256),
        });
    }
}

fn tokenize_query(value: &str) -> Vec<String> {
    normalize_text(value)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(normalize_text)
        .filter(|value| !value.trim().is_empty())
}

fn normalize_text(value: &str) -> String {
    trim_to_max_length(
        &value
            .to_lowercase()
            .replace(['\n', '\r'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        MAX_SEARCH_TEXT_LENGTH,
    )
}

fn trim_to_max_length(value: &str, max_length: usize) -> String {
    if value.chars().count() <= max_length {
        return value.trim().to_string();
    }

    value
        .chars()
        .take(max_length)
        .collect::<String>()
        .trim()
        .to_string()
}
