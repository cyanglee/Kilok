//! Claude Time Tracker MCP Server
//!
//! Provides MCP tools for Claude Code to:
//! - list_work_items: List work items within a time range
//! - get_work_item: Get detailed work item info with sessions and commits
//! - update_work_item: Update work item title, description, or time adjustment

use anyhow::Result;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::ServiceExt,
    tool, tool_handler, tool_router,
    ErrorData as McpError, Json, ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

mod db_wrapper;
use db_wrapper::DbWrapper;

/// MCP Server handler for time tracking
#[derive(Clone)]
pub struct TimeTrackerServer {
    db: Arc<RwLock<Option<DbWrapper>>>,
    tool_router: ToolRouter<Self>,
}

impl TimeTrackerServer {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    async fn get_db(&self) -> Result<DbWrapper, McpError> {
        let guard = self.db.read().await;
        if let Some(db) = guard.as_ref() {
            return Ok(db.clone());
        }
        drop(guard);

        // Initialize database connection
        let db = DbWrapper::new().await.map_err(|e| {
            McpError::internal_error(e.to_string(), None)
        })?;
        let mut write_guard = self.db.write().await;
        *write_guard = Some(db.clone());
        Ok(db)
    }
}

// Tool parameter types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListWorkItemsParams {
    /// Project name or path (optional)
    #[schemars(description = "專案名稱或路徑")]
    pub project: Option<String>,
    /// Month in "YYYY-MM" format (optional)
    #[schemars(description = "月份，格式為 YYYY-MM")]
    pub month: Option<String>,
    /// Include commit list in response
    #[serde(default)]
    #[schemars(description = "是否包含 commit 列表")]
    pub include_commits: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetWorkItemParams {
    /// Work item ID
    #[schemars(description = "工作項目 ID")]
    pub work_item_id: Option<i64>,
    /// Work item identifier (e.g., "ABC-123")
    #[schemars(description = "工作項目識別碼，例如 ABC-123")]
    pub identifier: Option<String>,
    /// Project name or path (required if using identifier)
    #[schemars(description = "專案名稱或路徑（使用 identifier 時必填）")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateWorkItemParams {
    /// Work item identifier (e.g., "ABC-123" or "feature-name")
    #[schemars(description = "工作項目識別碼，例如 ABC-123 或 invoice-enhancements")]
    pub identifier: String,
    /// Project name or path
    #[schemars(description = "專案名稱或路徑")]
    pub project: String,
    /// Optional title
    #[schemars(description = "標題（選填）")]
    pub title: Option<String>,
    /// Optional description
    #[schemars(description = "描述（選填）")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateWorkItemParams {
    /// Work item ID
    #[schemars(description = "工作項目 ID")]
    pub work_item_id: i64,
    /// New title
    #[schemars(description = "新標題")]
    pub title: Option<String>,
    /// New description
    #[schemars(description = "新描述")]
    pub description: Option<String>,
    /// Time adjustment in seconds (positive or negative)
    #[schemars(description = "時間調整秒數（可正可負）")]
    pub time_adjustment_seconds: Option<i64>,
    /// Completion date (YYYY-MM-DD format)
    #[schemars(description = "完成日期，格式為 YYYY-MM-DD")]
    pub completed_date: Option<String>,
}

// Response types
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListWorkItemsResponse {
    pub work_items: Vec<WorkItemResponse>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WorkItemResponse {
    pub id: i64,
    pub identifier: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub total_seconds: i64,
    pub time_adjustment_seconds: i64,
    pub completed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits: Option<Vec<String>>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WorkItemDetailResponse {
    pub id: i64,
    pub identifier: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub total_seconds: i64,
    pub time_adjustment_seconds: i64,
    pub completed_date: Option<String>,
    pub sessions: Vec<SessionResponse>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SessionResponse {
    pub id: i64,
    pub started_at: String,
    pub active_seconds: Option<i64>,
    pub commits: Vec<CommitResponse>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CommitResponse {
    pub hash: String,
    pub message: String,
}

#[tool_router]
impl TimeTrackerServer {
    /// List work items for a project within a time range
    #[tool(description = "列出工作項目。可依專案和月份篩選。")]
    async fn list_work_items(
        &self,
        params: Parameters<ListWorkItemsParams>,
    ) -> Result<Json<ListWorkItemsResponse>, String> {
        let db = self.get_db().await.map_err(|e| e.to_string())?;

        let work_items = db
            .list_work_items(params.0.project.as_deref(), params.0.month.as_deref())
            .await
            .map_err(|e| e.to_string())?;

        let mut responses = Vec::new();
        for wi in work_items {
            let commits = if params.0.include_commits {
                Some(
                    db.get_work_item_commits(wi.id)
                        .await
                        .unwrap_or_default()
                        .into_iter()
                        .map(|c| c.message.unwrap_or_default())
                        .collect(),
                )
            } else {
                None
            };

            let total_seconds = db.calculate_work_item_total_seconds(wi.id).await.unwrap_or(0);
            let source = db.get_project_source(wi.project_id).await.unwrap_or(None);

            responses.push(WorkItemResponse {
                id: wi.id,
                identifier: wi.identifier,
                title: wi.title,
                description: wi.description,
                total_seconds,
                time_adjustment_seconds: wi.time_adjustment_seconds,
                completed_date: wi.completed_date,
                source,
                commits,
            });
        }

        Ok(Json(ListWorkItemsResponse { work_items: responses }))
    }

    /// Get detailed information about a specific work item
    #[tool(description = "取得工作項目完整資訊，包含所有 sessions 和 commits")]
    async fn get_work_item(
        &self,
        params: Parameters<GetWorkItemParams>,
    ) -> Result<Json<WorkItemDetailResponse>, String> {
        let db = self.get_db().await.map_err(|e| e.to_string())?;

        let detail = db
            .get_work_item_detail(
                params.0.work_item_id,
                params.0.identifier.as_deref(),
                params.0.project.as_deref(),
            )
            .await
            .map_err(|e| e.to_string())?;

        let sessions: Vec<SessionResponse> = detail
            .sessions
            .into_iter()
            .map(|(session, commits)| SessionResponse {
                id: session.id,
                started_at: session.started_at.to_rfc3339(),
                active_seconds: session.active_seconds,
                commits: commits
                    .into_iter()
                    .map(|c| CommitResponse {
                        hash: c.hash,
                        message: c.message.unwrap_or_default(),
                    })
                    .collect(),
            })
            .collect();

        let response = WorkItemDetailResponse {
            id: detail.work_item.id,
            identifier: detail.work_item.identifier,
            title: detail.work_item.title,
            description: detail.work_item.description,
            total_seconds: detail.total_seconds,
            time_adjustment_seconds: detail.work_item.time_adjustment_seconds,
            completed_date: detail.work_item.completed_date,
            sessions,
        };

        Ok(Json(response))
    }

    /// Create a work item or return existing one
    #[tool(description = "建立工作項目（如已存在則回傳現有項目）。用於在更新前先確保工作項目存在。")]
    async fn create_work_item(
        &self,
        params: Parameters<CreateWorkItemParams>,
    ) -> Result<Json<WorkItemResponse>, String> {
        let db = self.get_db().await.map_err(|e| e.to_string())?;

        // Get or create work item
        let work_item = db
            .get_or_create_work_item(&params.0.identifier, &params.0.project)
            .await
            .map_err(|e| e.to_string())?;

        // If title or description provided, update the work item
        if params.0.title.is_some() || params.0.description.is_some() {
            let updated = db
                .update_work_item(
                    work_item.id,
                    params.0.title.as_deref(),
                    params.0.description.as_deref(),
                    None,
                    None,
                )
                .await
                .map_err(|e| e.to_string())?;

            let total_seconds = db
                .calculate_work_item_total_seconds(updated.id)
                .await
                .unwrap_or(0);
            let source = db.get_project_source(updated.project_id).await.unwrap_or(None);

            return Ok(Json(WorkItemResponse {
                id: updated.id,
                identifier: updated.identifier,
                title: updated.title,
                description: updated.description,
                total_seconds,
                time_adjustment_seconds: updated.time_adjustment_seconds,
                completed_date: updated.completed_date,
                source,
                commits: None,
            }));
        }

        let total_seconds = db
            .calculate_work_item_total_seconds(work_item.id)
            .await
            .unwrap_or(0);
        let source = db.get_project_source(work_item.project_id).await.unwrap_or(None);

        Ok(Json(WorkItemResponse {
            id: work_item.id,
            identifier: work_item.identifier,
            title: work_item.title,
            description: work_item.description,
            total_seconds,
            time_adjustment_seconds: work_item.time_adjustment_seconds,
            completed_date: work_item.completed_date,
            source,
            commits: None,
        }))
    }

    /// Update a work item's title, description, or time adjustment
    #[tool(description = "更新工作項目的標題、描述或時間調整")]
    async fn update_work_item(
        &self,
        params: Parameters<UpdateWorkItemParams>,
    ) -> Result<Json<WorkItemResponse>, String> {
        let db = self.get_db().await.map_err(|e| e.to_string())?;

        let updated = db
            .update_work_item(
                params.0.work_item_id,
                params.0.title.as_deref(),
                params.0.description.as_deref(),
                params.0.time_adjustment_seconds,
                params.0.completed_date.as_deref(),
            )
            .await
            .map_err(|e| e.to_string())?;

        let total_seconds = db
            .calculate_work_item_total_seconds(updated.id)
            .await
            .unwrap_or(0);

        let source = db.get_project_source(updated.project_id).await.unwrap_or(None);

        let response = WorkItemResponse {
            id: updated.id,
            identifier: updated.identifier,
            title: updated.title,
            description: updated.description,
            total_seconds,
            time_adjustment_seconds: updated.time_adjustment_seconds,
            completed_date: updated.completed_date,
            source,
            commits: None,
        };

        Ok(Json(response))
    }
}

#[tool_handler]
impl ServerHandler for TimeTrackerServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Claude Time Tracker MCP Server - 追蹤 Claude Code 使用時間並產生報告".into(),
            ),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize MCP server with stdio transport
    let server = TimeTrackerServer::new();

    let service = server
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await?;

    service.waiting().await?;

    Ok(())
}
