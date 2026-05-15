use crate::app_paths::AppPaths;
use crate::presence;
use crate::retrieval::search_clipboard_history;
use crate::state::AppState;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub use crate::models::{ClipboardEntry, ClipboardRetrievalMatch, ClipboardRetrievalMatchKind};
pub use crate::presence::DaemonStatus;
pub use crate::retrieval::ClipboardHistoryQuery;

pub const DEFAULT_STORAGE_OWNER: &str = "com.dance.app";

#[derive(Clone)]
pub struct HeadlessApp {
    owner: String,
    paths: Arc<AppPaths>,
    state: Arc<AppState>,
}

impl HeadlessApp {
    pub async fn new_default() -> Result<Self> {
        Self::new(DEFAULT_STORAGE_OWNER).await
    }

    pub async fn new(owner: impl Into<String>) -> Result<Self> {
        let owner = owner.into();
        let paths = Arc::new(AppPaths::from_storage_owner_identifier(&owner)?);
        paths.migrate_legacy_roots()?;
        let state = Arc::new(AppState::new(paths.clone()).await?);

        Ok(Self {
            owner,
            paths,
            state,
        })
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub async fn search_clipboard_history(
        &self,
        query: ClipboardHistoryQuery,
    ) -> Result<Vec<ClipboardEntry>> {
        search_clipboard_history(self.state.db.pool(), query).await
    }

    pub async fn start_capture(&self) -> Result<()> {
        self.state.start_monitoring().await
    }

    pub async fn stop_capture(&self) -> Result<()> {
        self.state.stop_monitoring().await
    }

    pub async fn is_capture_running(&self) -> bool {
        self.state.is_monitoring().await
    }

    pub fn is_tauri_active(&self) -> bool {
        presence::is_tauri_presence_fresh(self.paths.as_ref())
    }

    pub fn write_daemon_status(&self, state: &str) -> Result<()> {
        presence::write_daemon_status(self.paths.as_ref(), state, &self.owner)
    }

    pub fn read_daemon_status(&self) -> Option<DaemonStatus> {
        presence::read_daemon_status(self.paths.as_ref())
    }

    pub fn resolve_file_path(&self, file_path: &str) -> Result<PathBuf> {
        if file_path.starts_with("imgs/") {
            return self.paths.resolve_relative_asset_path(file_path);
        }
        Ok(PathBuf::from(file_path))
    }
}
