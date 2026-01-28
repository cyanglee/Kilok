use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use libsql::{params, Builder, Connection};
use std::path::Path;

use crate::models::{Commit, Heartbeat, Project, Session, SessionStatus, WorkItem, WorkItemDetail, WorkItemSource};

/// Database wrapper supporting both local and Turso remote
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open database in local-only mode
    pub async fn open_local(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {}", parent.display()))?;
        }

        let db = Builder::new_local(path)
            .build()
            .await
            .with_context(|| format!("Failed to open database: {}", path.display()))?;

        let conn = db.connect()?;

        // Enable WAL mode for better concurrent access
        // Also set busy timeout to wait instead of failing immediately on lock
        // Note: PRAGMA journal_mode returns a row, so we use query() and discard the result
        let _ = conn.query("PRAGMA journal_mode=WAL", ()).await
            .context("Failed to enable WAL mode")?;
        let _ = conn.query("PRAGMA busy_timeout=5000", ()).await
            .context("Failed to set busy timeout")?;

        let database = Self { conn };
        database.initialize().await?;
        Ok(database)
    }

    /// Open database with embedded replica (local + Turso sync)
    pub async fn open_replica(local_path: &Path, turso_url: &str, auth_token: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {}", parent.display()))?;
        }

        // Helper to check if error is related to corrupted local DB
        fn is_corruption_error(e: &anyhow::Error) -> bool {
            let msg = e.to_string().to_lowercase();
            msg.contains("malformed") || msg.contains("corrupt") || msg.contains("disk image")
        }

        // Helper to clean up local cache files
        fn cleanup_local_cache(path: &Path) {
            eprintln!("Local cache corrupted, removing and re-syncing from Turso...");
            let _ = std::fs::remove_file(path);
            // Also remove -info and -wal files
            let info_path = path.with_extension("db-info");
            let _ = std::fs::remove_file(&info_path);
            let wal_path = path.with_extension("db-wal");
            let _ = std::fs::remove_file(&wal_path);
            let shm_path = path.with_extension("db-shm");
            let _ = std::fs::remove_file(&shm_path);
            // Remove any tmp files
            if let Some(parent) = path.parent() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().starts_with(".tmp.") {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }

        // First attempt
        let result = Self::try_open_replica(local_path, turso_url, auth_token).await;

        match result {
            Ok(db) => Ok(db),
            Err(e) if is_corruption_error(&e) => {
                // Clean up and retry
                cleanup_local_cache(local_path);
                Self::try_open_replica(local_path, turso_url, auth_token)
                    .await
                    .with_context(|| "Failed to connect to Turso after cache cleanup")
            }
            Err(e) => Err(e),
        }
    }

    /// Internal helper to attempt opening replica
    async fn try_open_replica(local_path: &Path, turso_url: &str, auth_token: &str) -> Result<Self> {
        let db = Builder::new_remote_replica(
            local_path,
            turso_url.to_string(),
            auth_token.to_string(),
        )
        .build()
        .await
        .with_context(|| "Failed to build Turso connection")?;

        let conn = db.connect()
            .with_context(|| "Failed to connect to database")?;

        // Enable WAL mode and busy timeout for better concurrent access
        // Note: PRAGMA journal_mode returns a row, so we use query() and discard the result
        let _ = conn.query("PRAGMA journal_mode=WAL", ()).await
            .context("Failed to enable WAL mode")?;
        let _ = conn.query("PRAGMA busy_timeout=5000", ()).await
            .context("Failed to set busy timeout")?;

        // Initial sync from remote
        db.sync().await.with_context(|| "Failed to sync from Turso")?;

        let database = Self { conn };
        database.initialize().await?;
        Ok(database)
    }

    /// Open database in pure remote mode (no local cache)
    /// This is safer for concurrent access from multiple processes
    pub async fn open_remote(turso_url: &str, auth_token: &str) -> Result<Self> {
        let db = Builder::new_remote(turso_url.to_string(), auth_token.to_string())
            .build()
            .await
            .with_context(|| "Failed to connect to Turso (remote mode)")?;

        let conn = db.connect()
            .with_context(|| "Failed to connect to database")?;

        let database = Self { conn };
        database.initialize().await?;
        Ok(database)
    }

    /// Initialize database schema
    async fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY,
                slug TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS contracts (
                id INTEGER PRIMARY KEY,
                client_id INTEGER NOT NULL REFERENCES clients(id),
                year INTEGER NOT NULL,
                total_hours REAL NOT NULL,
                carried_over REAL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(client_id, year)
            );

            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                git_remote TEXT,
                display_name TEXT,
                work_item_pattern TEXT,
                client_id INTEGER REFERENCES clients(id),
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id),
                branch TEXT NOT NULL,
                work_item TEXT,
                start_commit TEXT,
                end_commit TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                active_seconds INTEGER,
                status TEXT NOT NULL DEFAULT 'active'
            );

            CREATE TABLE IF NOT EXISTS heartbeats (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL REFERENCES sessions(id),
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS commits (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL REFERENCES sessions(id),
                hash TEXT NOT NULL,
                message TEXT,
                committed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS work_items (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id),
                identifier TEXT NOT NULL,
                title TEXT,
                description TEXT,
                time_adjustment_seconds INTEGER DEFAULT 0,
                completed_date TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT,
                UNIQUE(project_id, identifier)
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions(project_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
            CREATE INDEX IF NOT EXISTS idx_heartbeats_session_id ON heartbeats(session_id);
            CREATE INDEX IF NOT EXISTS idx_commits_session_id ON commits(session_id);
            CREATE INDEX IF NOT EXISTS idx_work_items_project_id ON work_items(project_id);
            "#,
        )
        .await
        .context("Failed to initialize database schema")?;

        // Run migrations for existing databases
        self.run_migrations().await?;

        Ok(())
    }

    /// Run database migrations for schema updates
    async fn run_migrations(&self) -> Result<()> {
        // Migration: Add work_item_id to sessions table
        // Check if column exists first
        let session_columns: Vec<String> = {
            let mut rows = self.conn.query(
                "PRAGMA table_info(sessions)",
                (),
            ).await?;
            let mut cols = Vec::new();
            while let Some(row) = rows.next().await? {
                cols.push(row.get::<String>(1)?);
            }
            cols
        };

        if !session_columns.contains(&"work_item_id".to_string()) {
            self.conn.execute(
                "ALTER TABLE sessions ADD COLUMN work_item_id INTEGER REFERENCES work_items(id)",
                (),
            ).await.context("Failed to add work_item_id column to sessions")?;

            // Create index after adding column
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_sessions_work_item_id ON sessions(work_item_id)",
                (),
            ).await.context("Failed to create work_item_id index")?;
        }

        // Migration: Add work_item_source to projects table
        let project_columns: Vec<String> = {
            let mut rows = self.conn.query(
                "PRAGMA table_info(projects)",
                (),
            ).await?;
            let mut cols = Vec::new();
            while let Some(row) = rows.next().await? {
                cols.push(row.get::<String>(1)?);
            }
            cols
        };

        if !project_columns.contains(&"work_item_source".to_string()) {
            self.conn.execute(
                "ALTER TABLE projects ADD COLUMN work_item_source TEXT",
                (),
            ).await.context("Failed to add work_item_source column to projects")?;
        }

        Ok(())
    }

    // ==================== Projects ====================

    /// Get or create a project by path
    pub async fn get_or_create_project(
        &self,
        path: &str,
        git_remote: Option<&str>,
        display_name: Option<&str>,
        work_item_pattern: Option<&str>,
        work_item_source: Option<&str>,
    ) -> Result<Project> {
        // Try to find existing project
        if let Some(project) = self.get_project_by_path(path).await? {
            // Update if new info provided
            if git_remote.is_some() || display_name.is_some() || work_item_pattern.is_some() || work_item_source.is_some() {
                self.conn.execute(
                    "UPDATE projects SET
                        git_remote = COALESCE(?1, git_remote),
                        display_name = COALESCE(?2, display_name),
                        work_item_pattern = COALESCE(?3, work_item_pattern),
                        work_item_source = COALESCE(?4, work_item_source)
                    WHERE id = ?5",
                    params![git_remote, display_name, work_item_pattern, work_item_source, project.id],
                ).await?;
                return self.get_project_by_id(project.id).await;
            }
            return Ok(project);
        }

        // Create new project
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO projects (path, git_remote, display_name, work_item_pattern, work_item_source, client_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                path,
                git_remote,
                display_name,
                work_item_pattern,
                work_item_source,
                Option::<i64>::None,
                now.to_rfc3339()
            ],
        ).await?;

        let id = self.conn.last_insert_rowid();
        self.get_project_by_id(id).await
    }

    /// Get project by ID
    pub async fn get_project_by_id(&self, id: i64) -> Result<Project> {
        let mut rows = self.conn
            .query(
                "SELECT id, path, git_remote, display_name, work_item_pattern, work_item_source, client_id, created_at
                 FROM projects WHERE id = ?1",
                params![id],
            )
            .await?;

        let row = rows.next().await?.context("Project not found")?;
        row_to_project(&row)
    }

    /// Get project by path
    pub async fn get_project_by_path(&self, path: &str) -> Result<Option<Project>> {
        let mut rows = self.conn
            .query(
                "SELECT id, path, git_remote, display_name, work_item_pattern, work_item_source, client_id, created_at
                 FROM projects WHERE path = ?1",
                params![path],
            )
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let mut rows = self.conn.query(
            "SELECT id, path, git_remote, display_name, work_item_pattern, work_item_source, client_id, created_at
             FROM projects ORDER BY path",
            (),
        ).await?;

        let mut projects = Vec::new();
        while let Some(row) = rows.next().await? {
            projects.push(row_to_project(&row)?);
        }

        Ok(projects)
    }

    // ==================== Sessions ====================

    /// Create a new session
    pub async fn create_session(
        &self,
        project_id: i64,
        branch: &str,
        work_item: Option<&str>,
        start_commit: Option<&str>,
    ) -> Result<Session> {
        self.create_session_at(project_id, branch, work_item, start_commit, None).await
    }

    /// Create a new session with optional custom start time
    /// Used by sync command to create sessions starting at the first heartbeat time
    pub async fn create_session_at(
        &self,
        project_id: i64,
        branch: &str,
        work_item: Option<&str>,
        start_commit: Option<&str>,
        started_at: Option<DateTime<Utc>>,
    ) -> Result<Session> {
        let start_time = started_at.unwrap_or_else(Utc::now);
        self.conn.execute(
            "INSERT INTO sessions (project_id, branch, work_item, start_commit, started_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project_id,
                branch,
                work_item,
                start_commit,
                start_time.to_rfc3339(),
                SessionStatus::Active.as_str()
            ],
        ).await?;

        let id = self.conn.last_insert_rowid();
        self.get_session_by_id(id).await
    }

    /// Get session by ID
    pub async fn get_session_by_id(&self, id: i64) -> Result<Session> {
        let mut rows = self.conn
            .query(
                "SELECT id, project_id, branch, work_item, start_commit, end_commit,
                        started_at, ended_at, active_seconds, status
                 FROM sessions WHERE id = ?1",
                params![id],
            )
            .await?;

        let row = rows.next().await?.context("Session not found")?;
        row_to_session(&row)
    }

    /// Get active session for a project
    pub async fn get_active_session(&self, project_id: i64) -> Result<Option<Session>> {
        let mut rows = self.conn
            .query(
                "SELECT id, project_id, branch, work_item, start_commit, end_commit,
                        started_at, ended_at, active_seconds, status
                 FROM sessions WHERE project_id = ?1 AND status = 'active'
                 ORDER BY started_at DESC LIMIT 1",
                params![project_id],
            )
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(row_to_session(&row)?)),
            None => Ok(None),
        }
    }

    /// Get all active sessions (for cleanup)
    pub async fn get_all_active_sessions(&self) -> Result<Vec<Session>> {
        let mut rows = self.conn.query(
            "SELECT id, project_id, branch, work_item, start_commit, end_commit,
                    started_at, ended_at, active_seconds, status
             FROM sessions WHERE status = 'active'",
            (),
        ).await?;

        let mut sessions = Vec::new();
        while let Some(row) = rows.next().await? {
            sessions.push(row_to_session(&row)?);
        }

        Ok(sessions)
    }

    /// Update session end state
    pub async fn complete_session(
        &self,
        session_id: i64,
        end_commit: Option<&str>,
        active_seconds: i64,
        status: SessionStatus,
    ) -> Result<()> {
        let now = Utc::now();
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?1, end_commit = ?2, active_seconds = ?3, status = ?4
             WHERE id = ?5",
            params![
                now.to_rfc3339(),
                end_commit,
                active_seconds,
                status.as_str(),
                session_id
            ],
        ).await?;
        Ok(())
    }

    /// Get sessions within a time range
    pub async fn get_sessions_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        project_id: Option<i64>,
    ) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();

        if let Some(pid) = project_id {
            let mut rows = self.conn.query(
                "SELECT id, project_id, branch, work_item, start_commit, end_commit,
                        started_at, ended_at, active_seconds, status
                 FROM sessions
                 WHERE started_at >= ?1 AND started_at < ?2 AND project_id = ?3 AND status != 'active'
                 ORDER BY started_at",
                params![start.to_rfc3339(), end.to_rfc3339(), pid],
            ).await?;

            while let Some(row) = rows.next().await? {
                sessions.push(row_to_session(&row)?);
            }
        } else {
            let mut rows = self.conn.query(
                "SELECT id, project_id, branch, work_item, start_commit, end_commit,
                        started_at, ended_at, active_seconds, status
                 FROM sessions
                 WHERE started_at >= ?1 AND started_at < ?2 AND status != 'active'
                 ORDER BY started_at",
                params![start.to_rfc3339(), end.to_rfc3339()],
            ).await?;

            while let Some(row) = rows.next().await? {
                sessions.push(row_to_session(&row)?);
            }
        }

        Ok(sessions)
    }

    // ==================== Heartbeats ====================

    /// Record a heartbeat
    pub async fn record_heartbeat(&self, session_id: i64) -> Result<Heartbeat> {
        let now = Utc::now();
        self.record_heartbeat_at(session_id, now).await
    }

    /// Record a heartbeat at a specific timestamp (for syncing from local cache)
    pub async fn record_heartbeat_at(&self, session_id: i64, timestamp: DateTime<Utc>) -> Result<Heartbeat> {
        self.conn.execute(
            "INSERT INTO heartbeats (session_id, timestamp) VALUES (?1, ?2)",
            params![session_id, timestamp.to_rfc3339()],
        ).await?;

        Ok(Heartbeat {
            id: self.conn.last_insert_rowid(),
            session_id,
            timestamp,
        })
    }

    /// Get heartbeats for a session
    pub async fn get_heartbeats(&self, session_id: i64) -> Result<Vec<Heartbeat>> {
        let mut rows = self.conn.query(
            "SELECT id, session_id, timestamp FROM heartbeats
             WHERE session_id = ?1 ORDER BY timestamp",
            params![session_id],
        ).await?;

        let mut heartbeats = Vec::new();
        while let Some(row) = rows.next().await? {
            heartbeats.push(Heartbeat {
                id: row.get::<i64>(0)?,
                session_id: row.get::<i64>(1)?,
                timestamp: parse_datetime(row.get::<String>(2)?),
            });
        }

        Ok(heartbeats)
    }

    // ==================== Work Items ====================

    /// Get or create a work item by identifier
    pub async fn get_or_create_work_item(
        &self,
        project_id: i64,
        identifier: &str,
    ) -> Result<WorkItem> {
        // Try to find existing work item
        if let Some(work_item) = self.get_work_item_by_identifier(project_id, identifier).await? {
            return Ok(work_item);
        }

        // Create new work item
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO work_items (project_id, identifier, created_at)
             VALUES (?1, ?2, ?3)",
            params![project_id, identifier, now.to_rfc3339()],
        ).await?;

        let id = self.conn.last_insert_rowid();
        self.get_work_item_by_id(id).await
    }

    /// Get work item by ID
    pub async fn get_work_item_by_id(&self, id: i64) -> Result<WorkItem> {
        let mut rows = self.conn
            .query(
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items WHERE id = ?1",
                params![id],
            )
            .await?;

        let row = rows.next().await?.context("Work item not found")?;
        row_to_work_item(&row)
    }

    /// Get work item by identifier within a project
    pub async fn get_work_item_by_identifier(
        &self,
        project_id: i64,
        identifier: &str,
    ) -> Result<Option<WorkItem>> {
        let mut rows = self.conn
            .query(
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items WHERE project_id = ?1 AND identifier = ?2",
                params![project_id, identifier],
            )
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(row_to_work_item(&row)?)),
            None => Ok(None),
        }
    }

    /// List work items for a project with optional time range filter
    pub async fn list_work_items(
        &self,
        project_id: Option<i64>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<WorkItem>> {
        let mut work_items = Vec::new();

        // Build query based on filters
        let (query, params_vec): (String, Vec<libsql::Value>) = match (project_id, start, end) {
            (Some(pid), Some(s), Some(e)) => (
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items
                 WHERE project_id = ?1 AND created_at >= ?2 AND created_at < ?3
                 ORDER BY created_at DESC".to_string(),
                vec![pid.into(), s.to_rfc3339().into(), e.to_rfc3339().into()],
            ),
            (Some(pid), None, None) => (
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items WHERE project_id = ?1
                 ORDER BY created_at DESC".to_string(),
                vec![pid.into()],
            ),
            (None, Some(s), Some(e)) => (
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items
                 WHERE created_at >= ?1 AND created_at < ?2
                 ORDER BY created_at DESC".to_string(),
                vec![s.to_rfc3339().into(), e.to_rfc3339().into()],
            ),
            _ => (
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items ORDER BY created_at DESC".to_string(),
                vec![],
            ),
        };

        let mut rows = self.conn.query(&query, params_vec).await?;
        while let Some(row) = rows.next().await? {
            work_items.push(row_to_work_item(&row)?);
        }

        Ok(work_items)
    }

    /// Get work item detail with sessions and commits
    pub async fn get_work_item_detail(&self, work_item_id: i64) -> Result<WorkItemDetail> {
        let work_item = self.get_work_item_by_id(work_item_id).await?;

        // Get sessions for this work item
        let sessions = self.get_sessions_by_work_item_id(work_item_id).await?;

        // Get commits for all sessions
        let mut commits = Vec::new();
        for session in &sessions {
            let session_commits = self.get_commits(session.id).await?;
            commits.extend(session_commits);
        }

        // Calculate total seconds from sessions
        let session_seconds: i64 = sessions
            .iter()
            .filter_map(|s| s.active_seconds)
            .sum();

        let total_seconds = session_seconds + work_item.time_adjustment_seconds;

        Ok(WorkItemDetail {
            work_item,
            total_seconds,
            sessions,
            commits,
        })
    }

    /// Get sessions associated with a work item
    pub async fn get_sessions_by_work_item_id(&self, work_item_id: i64) -> Result<Vec<Session>> {
        let mut rows = self.conn.query(
            "SELECT id, project_id, branch, work_item, start_commit, end_commit,
                    started_at, ended_at, active_seconds, status
             FROM sessions WHERE work_item_id = ?1
             ORDER BY started_at",
            params![work_item_id],
        ).await?;

        let mut sessions = Vec::new();
        while let Some(row) = rows.next().await? {
            sessions.push(row_to_session(&row)?);
        }

        Ok(sessions)
    }

    /// Update work item fields
    pub async fn update_work_item(
        &self,
        work_item_id: i64,
        title: Option<&str>,
        description: Option<&str>,
        time_adjustment_seconds: Option<i64>,
        completed_date: Option<&str>,
    ) -> Result<WorkItem> {
        let now = Utc::now();

        // Build dynamic update query based on provided fields
        let mut updates = vec!["updated_at = ?1".to_string()];
        let mut params: Vec<libsql::Value> = vec![now.to_rfc3339().into()];
        let mut param_idx = 2;

        if let Some(t) = title {
            updates.push(format!("title = ?{}", param_idx));
            params.push(t.into());
            param_idx += 1;
        }

        if let Some(d) = description {
            updates.push(format!("description = ?{}", param_idx));
            params.push(d.into());
            param_idx += 1;
        }

        if let Some(adj) = time_adjustment_seconds {
            updates.push(format!("time_adjustment_seconds = ?{}", param_idx));
            params.push(adj.into());
            param_idx += 1;
        }

        if let Some(cd) = completed_date {
            updates.push(format!("completed_date = ?{}", param_idx));
            params.push(cd.into());
            param_idx += 1;
        }

        params.push(work_item_id.into());

        let query = format!(
            "UPDATE work_items SET {} WHERE id = ?{}",
            updates.join(", "),
            param_idx
        );

        self.conn.execute(&query, params).await?;
        self.get_work_item_by_id(work_item_id).await
    }

    /// Link a session to a work item
    pub async fn link_session_to_work_item(
        &self,
        session_id: i64,
        work_item_id: i64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET work_item_id = ?1 WHERE id = ?2",
            params![work_item_id, session_id],
        ).await?;
        Ok(())
    }

    // ==================== Commits ====================

    /// Record commits for a session
    pub async fn record_commits(&self, session_id: i64, commits: &[(String, String, Option<DateTime<Utc>>)]) -> Result<()> {
        for (hash, message, committed_at) in commits {
            self.conn.execute(
                "INSERT INTO commits (session_id, hash, message, committed_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    session_id,
                    hash.as_str(),
                    message.as_str(),
                    committed_at.map(|dt| dt.to_rfc3339())
                ],
            ).await?;
        }
        Ok(())
    }

    /// Get commits for a session
    pub async fn get_commits(&self, session_id: i64) -> Result<Vec<Commit>> {
        let mut rows = self.conn.query(
            "SELECT id, session_id, hash, message, committed_at FROM commits
             WHERE session_id = ?1 ORDER BY committed_at",
            params![session_id],
        ).await?;

        let mut commits = Vec::new();
        while let Some(row) = rows.next().await? {
            commits.push(Commit {
                id: row.get::<i64>(0)?,
                session_id: row.get::<i64>(1)?,
                hash: row.get::<String>(2)?,
                message: row.get::<Option<String>>(3)?,
                committed_at: row.get::<Option<String>>(4)?.map(parse_datetime),
            });
        }

        Ok(commits)
    }
}

fn row_to_project(row: &libsql::Row) -> Result<Project> {
    Ok(Project {
        id: row.get::<i64>(0)?,
        path: row.get::<String>(1)?,
        git_remote: row.get::<Option<String>>(2)?,
        display_name: row.get::<Option<String>>(3)?,
        work_item_pattern: row.get::<Option<String>>(4)?,
        work_item_source: row.get::<Option<String>>(5)?.and_then(|s| WorkItemSource::from_str(&s)),
        client_id: row.get::<Option<i64>>(6)?,
        created_at: parse_datetime(row.get::<String>(7)?),
    })
}

fn row_to_session(row: &libsql::Row) -> Result<Session> {
    Ok(Session {
        id: row.get::<i64>(0)?,
        project_id: row.get::<i64>(1)?,
        branch: row.get::<String>(2)?,
        work_item: row.get::<Option<String>>(3)?,
        start_commit: row.get::<Option<String>>(4)?,
        end_commit: row.get::<Option<String>>(5)?,
        started_at: parse_datetime(row.get::<String>(6)?),
        ended_at: row.get::<Option<String>>(7)?.map(parse_datetime),
        active_seconds: row.get::<Option<i64>>(8)?,
        status: SessionStatus::from_str(&row.get::<String>(9)?).unwrap_or(SessionStatus::Active),
    })
}

fn row_to_work_item(row: &libsql::Row) -> Result<WorkItem> {
    Ok(WorkItem {
        id: row.get::<i64>(0)?,
        project_id: row.get::<i64>(1)?,
        identifier: row.get::<String>(2)?,
        title: row.get::<Option<String>>(3)?,
        description: row.get::<Option<String>>(4)?,
        time_adjustment_seconds: row.get::<i64>(5)?,
        completed_date: row.get::<Option<String>>(6)?,
        created_at: parse_datetime(row.get::<String>(7)?),
        updated_at: row.get::<Option<String>>(8)?.map(parse_datetime),
    })
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_local(&db_path).await.unwrap();

        // Should be able to create a project
        let project = db.get_or_create_project("/test/path", None, None, None, None).await.unwrap();
        assert_eq!(project.path, "/test/path");
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_local(&db_path).await.unwrap();

        let project = db.get_or_create_project("/test/path", None, None, None, None).await.unwrap();

        // Create session
        let session = db.create_session(project.id, "main", None, None).await.unwrap();
        assert_eq!(session.status, SessionStatus::Active);

        // Record heartbeats
        db.record_heartbeat(session.id).await.unwrap();
        db.record_heartbeat(session.id).await.unwrap();

        let heartbeats = db.get_heartbeats(session.id).await.unwrap();
        assert_eq!(heartbeats.len(), 2);

        // Complete session
        db.complete_session(session.id, None, 3600, SessionStatus::Completed).await.unwrap();

        let completed = db.get_session_by_id(session.id).await.unwrap();
        assert_eq!(completed.status, SessionStatus::Completed);
        assert_eq!(completed.active_seconds, Some(3600));
    }
}
