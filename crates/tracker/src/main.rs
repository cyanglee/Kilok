mod billable;
mod cli;
mod config;
mod db;
mod git;
mod idle;
mod models;
mod report;
mod statusline;
mod tracker;
mod weather;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

use cli::{Cli, Commands, ConfigAction, ProjectsAction};
use config::EffectiveConfig;
use db::{Database, SyncStats};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { path } => cmd_start(&path).await,
        Commands::Heartbeat { path } => cmd_heartbeat(&path).await,
        Commands::Stop { path } => cmd_stop(&path).await,
        Commands::Report {
            month,
            project,
            format,
            output,
            all_formats,
        } => cmd_report(month, project, format, output, all_formats).await,
        Commands::Status => cmd_status().await,
        Commands::ActiveTime { path } => cmd_active_time(&path).await,
        Commands::Sync { path } => cmd_sync(path).await,
        Commands::PushToRemote { dry_run } => cmd_push_to_remote(dry_run).await,
        Commands::Config { action } => match action {
            ConfigAction::Init => cmd_config_init(),
            ConfigAction::Edit => cmd_config_edit(),
            ConfigAction::Show => cmd_config_show(),
        },
        Commands::Projects { action } => match action {
            ProjectsAction::List => cmd_projects_list().await,
            ProjectsAction::SetName { path, name } => cmd_projects_set_name(&path, &name).await,
        },
        Commands::Statusline => cmd_statusline().await,
    }
}

async fn open_db(config: &EffectiveConfig) -> Result<Database> {
    if config.is_turso_enabled() {
        if config.turso_use_replica {
            // Embedded replica mode - local cache with sync
            // Only use this if you need offline support and don't have multiple writers
            Database::open_replica(
                &config.database_path,
                config.turso_url.as_ref().unwrap(),
                config.turso_auth_token.as_ref().unwrap(),
            ).await
        } else {
            // Pure remote mode (default) - no local cache, safer for concurrent access
            Database::open_remote(
                config.turso_url.as_ref().unwrap(),
                config.turso_auth_token.as_ref().unwrap(),
            ).await
        }
    } else {
        Database::open_local(&config.database_path).await
    }
}

async fn cmd_start(path: &str) -> Result<()> {
    let project_path = PathBuf::from(path).canonicalize()
        .with_context(|| format!("Invalid path: {}", path))?;

    let config = EffectiveConfig::load(Some(&project_path))?;
    let db = open_db(&config).await?;

    tracker::start_session(&db, &project_path, &config).await
}

async fn cmd_heartbeat(path: &str) -> Result<()> {
    let project_path = PathBuf::from(path).canonicalize()
        .with_context(|| format!("Invalid path: {}", path))?;

    let config = EffectiveConfig::load(Some(&project_path))?;

    // Check if project is idle (based on transcript mtime)
    if idle::is_project_idle(&project_path, config.idle_timeout_minutes) {
        // Project is idle, skip heartbeat
        return Ok(());
    }

    // Get current HEAD commit for per-commit time tracking
    let commit_hash = git::get_git_info(&project_path)
        .ok()
        .and_then(|g| g.head_commit);

    // Write heartbeat to local cache (no DB access)
    write_local_heartbeat(&project_path, commit_hash.as_deref())?;

    Ok(())
}

/// Write a heartbeat to local cache file
/// Format: {"timestamp":"...","project_path":"...","unix_ts":...,"commit_hash":"..."}
fn write_local_heartbeat(project_path: &Path, commit_hash: Option<&str>) -> Result<()> {
    use std::io::Write;

    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let cache_dir = home_dir.join(".cache").join("claude-time-tracker");
    fs::create_dir_all(&cache_dir)?;

    let heartbeat_file = cache_dir.join("heartbeats.jsonl");
    let now = Utc::now();

    let entry = serde_json::json!({
        "timestamp": now.to_rfc3339(),
        "project_path": project_path.to_string_lossy(),
        "unix_ts": now.timestamp(),
        "commit_hash": commit_hash
    });

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&heartbeat_file)
        .with_context(|| format!("Failed to open heartbeat cache: {}", heartbeat_file.display()))?;

    writeln!(file, "{}", entry)?;

    Ok(())
}

async fn cmd_stop(path: &str) -> Result<()> {
    let project_path = PathBuf::from(path).canonicalize()
        .with_context(|| format!("Invalid path: {}", path))?;

    let config = EffectiveConfig::load(Some(&project_path))?;
    let db = open_db(&config).await?;

    tracker::stop_session(&db, &project_path, &config).await
}

async fn cmd_report(
    month: Option<String>,
    project_filter: Option<String>,
    format: String,
    output: Option<String>,
    all_formats: bool,
) -> Result<()> {
    let config = EffectiveConfig::load(None)?;
    let db = open_db(&config).await?;

    // Parse month
    let (year, month_num) = if let Some(ref m) = month {
        report::parse_month(m)?
    } else {
        report::current_month()
    };

    // Generate report data
    let report_data = report::generate_report(
        &db,
        year,
        month_num,
        project_filter.as_deref(),
        config.max_commits_per_item,
    ).await?;

    // Determine formats to output
    let formats: Vec<&str> = if all_formats {
        vec!["md", "csv", "tsv", "json"]
    } else {
        format.split(',').map(|s| s.trim()).collect()
    };

    let multiple_formats = formats.len() > 1;

    // Generate and output reports
    for fmt in formats {
        let content = match fmt {
            "md" | "markdown" => report::markdown::generate(&report_data, config.include_commits),
            "csv" => report::csv::generate_string(&report_data, config.include_commits)?,
            "tsv" => report::tsv::generate_string(&report_data, config.include_commits)?,
            "json" => report::json::generate(&report_data)?,
            _ => {
                eprintln!("Unknown format: {}", fmt);
                continue;
            }
        };

        if let Some(ref base_path) = output {
            let ext = match fmt {
                "md" | "markdown" => "md",
                "csv" => "csv",
                "tsv" => "tsv",
                "json" => "json",
                _ => fmt,
            };
            let file_path = if multiple_formats {
                format!("{}.{}", base_path, ext)
            } else if base_path.ends_with(&format!(".{}", ext)) {
                base_path.clone()
            } else {
                format!("{}.{}", base_path, ext)
            };

            fs::write(&file_path, &content)
                .with_context(|| format!("Failed to write report to {}", file_path))?;
            eprintln!("Report written to: {}", file_path);
        } else {
            println!("{}", content);
        }
    }

    Ok(())
}

async fn cmd_status() -> Result<()> {
    let config = EffectiveConfig::load(None)?;
    let db = open_db(&config).await?;

    let active_sessions = db.get_all_active_sessions().await?;

    if active_sessions.is_empty() {
        println!("No active tracking sessions.");
        return Ok(());
    }

    println!("Active tracking sessions:\n");

    for session in active_sessions {
        let project = db.get_project_by_id(session.project_id).await?;
        let heartbeats = db.get_heartbeats(session.id).await?;

        let elapsed = calculate_active_time_with_current(&heartbeats, config.idle_timeout_minutes);

        println!(
            "  Project: {}",
            project.display_name.as_deref().unwrap_or(&project.path)
        );
        println!("  Branch:  {}", session.branch);
        println!("  Started: {}", session.started_at);
        println!("  Active:  {}", tracker::format_duration(elapsed));
        println!();
    }

    Ok(())
}

/// Calculate active time including time since last heartbeat (for status display)
fn calculate_active_time_with_current(heartbeats: &[models::Heartbeat], idle_timeout_minutes: u32) -> i64 {
    if heartbeats.is_empty() {
        return 0;
    }

    let timeout_seconds = (idle_timeout_minutes as i64) * 60;
    let mut total_seconds: i64 = 0;

    for window in heartbeats.windows(2) {
        let interval = (window[1].timestamp - window[0].timestamp).num_seconds();

        if interval <= timeout_seconds {
            total_seconds += interval;
        }
    }

    // Add time from last heartbeat to now (if within timeout)
    if let Some(last) = heartbeats.last() {
        let since_last = (Utc::now() - last.timestamp).num_seconds();
        if since_last <= timeout_seconds {
            total_seconds += since_last;
        }
    }

    total_seconds
}

/// Get active time for current session (for statusline integration)
/// Outputs only the seconds number, or nothing if no active session
async fn cmd_active_time(path: &str) -> Result<()> {
    let project_path = PathBuf::from(path).canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path));

    let config = EffectiveConfig::load(Some(&project_path))?;
    let db = open_db(&config).await?;

    let path_str = project_path.to_str().context("Invalid project path")?;

    // Find project
    let project = match db.get_project_by_path(path_str).await? {
        Some(p) => p,
        None => return Ok(()), // No output if no project
    };

    // Find active session
    let session = match db.get_active_session(project.id).await? {
        Some(s) => s,
        None => return Ok(()), // No output if no active session
    };

    // Get heartbeats and calculate active time
    let heartbeats = db.get_heartbeats(session.id).await?;
    let active_seconds = calculate_active_time_with_current(&heartbeats, config.idle_timeout_minutes);

    // Output only the number (for easy parsing by statusline)
    println!("{}", active_seconds);

    Ok(())
}

fn cmd_config_init() -> Result<()> {
    let path = config::init_global_config()?;
    println!("Configuration initialized at: {}", path.display());
    Ok(())
}

fn cmd_config_edit() -> Result<()> {
    let path = config::global_config_path()?;

    if !path.exists() {
        config::init_global_config()?;
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;

    Ok(())
}

fn cmd_config_show() -> Result<()> {
    let config = config::load_global_config()?;
    let toml = toml::to_string_pretty(&config)?;
    println!("{}", toml);
    Ok(())
}

async fn cmd_projects_list() -> Result<()> {
    let config = EffectiveConfig::load(None)?;
    let db = open_db(&config).await?;
    let projects = db.list_projects().await?;

    if projects.is_empty() {
        println!("No tracked projects yet.");
        return Ok(());
    }

    println!("Tracked projects:\n");

    for project in projects {
        let name = project.display_name.as_deref().unwrap_or("-");
        println!("  Path: {}", project.path);
        println!("  Name: {}", name);
        if let Some(ref remote) = project.git_remote {
            println!("  Remote: {}", remote);
        }
        println!();
    }

    Ok(())
}

async fn cmd_projects_set_name(path: &str, name: &str) -> Result<()> {
    let config = EffectiveConfig::load(None)?;
    let db = open_db(&config).await?;

    let project_path = PathBuf::from(path).canonicalize()
        .with_context(|| format!("Invalid path: {}", path))?;

    let path_str = project_path.to_str().context("Invalid path")?;

    db.get_or_create_project(path_str, None, Some(name), None, None).await?;

    println!("Set display name for {} to: {}", path_str, name);
    Ok(())
}

/// Sync local heartbeat cache to database
/// Called periodically by statusline to ensure reliable time tracking
/// Auto-creates sessions if heartbeats exist but no active session (statusline-first design)
/// Auto-closes idle sessions based on transcript activity
async fn cmd_sync(path: Option<String>) -> Result<()> {
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader};

    // Use ~/.cache/ (Linux-style) to match statusline-wrapper.sh
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let cache_dir = home_dir.join(".cache").join("claude-time-tracker");
    let heartbeat_file = cache_dir.join("heartbeats.jsonl");

    // Open database
    let config = EffectiveConfig::load(None)?;
    let db = open_db(&config).await?;

    // === Phase 1: Check for idle sessions and close them ===
    if let Some(ref path_str) = path {
        let project_path = PathBuf::from(path_str).canonicalize()
            .with_context(|| format!("Invalid path: {}", path_str))?;

        // Check if this project is idle
        if idle::is_project_idle(&project_path, config.idle_timeout_minutes) {
            // Close any active session for this project
            if let Some(project) = db.get_project_by_path(project_path.to_str().unwrap_or("")).await? {
                if let Some(session) = db.get_active_session(project.id).await? {
                    // Calculate active time and close
                    let heartbeats = db.get_heartbeats(session.id).await?;
                    let active_seconds = calculate_active_time_simple(&heartbeats, config.idle_timeout_minutes);

                    // Collect commits if possible
                    let git_info = git::get_git_info(&project_path).ok();
                    let end_commit = git_info.as_ref().and_then(|g| g.head_commit.clone());

                    if let Some(ref start) = session.start_commit {
                        if let Ok(commits) = git::get_commits_between(&project_path, Some(start), end_commit.as_deref()) {
                            if !commits.is_empty() {
                                let _ = db.record_commits(session.id, &commits).await;
                            }
                        }
                    }

                    db.complete_session(
                        session.id,
                        end_commit.as_deref(),
                        active_seconds,
                        models::SessionStatus::Completed,
                    ).await?;

                    eprintln!(
                        "Auto-closed idle session for: {} (active time: {})",
                        project.display_name.as_deref().unwrap_or(&project.path),
                        tracker::format_duration(active_seconds)
                    );
                }
            }
        }
    }

    // === Phase 2: Sync heartbeats from local cache ===

    // Check if cache file exists
    if !heartbeat_file.exists() {
        return Ok(());
    }

    // Read and parse heartbeats
    let file = fs::File::open(&heartbeat_file)
        .with_context(|| format!("Failed to open heartbeat cache: {}", heartbeat_file.display()))?;
    let reader = BufReader::new(file);

    // Heartbeat entry with commit hash
    #[derive(Clone)]
    struct HeartbeatEntry {
        timestamp: chrono::DateTime<Utc>,
        commit_hash: Option<String>,
    }

    // Group heartbeats by project path
    let mut heartbeats_by_project: HashMap<String, Vec<HeartbeatEntry>> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON: {"timestamp":"...","project_path":"...","unix_ts":...,"commit_hash":"..."}
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            if let (Some(timestamp_str), Some(project_path)) = (
                json.get("timestamp").and_then(|v| v.as_str()),
                json.get("project_path").and_then(|v| v.as_str()),
            ) {
                if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                    let commit_hash = json.get("commit_hash")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    heartbeats_by_project
                        .entry(project_path.to_string())
                        .or_default()
                        .push(HeartbeatEntry {
                            timestamp: timestamp.with_timezone(&Utc),
                            commit_hash,
                        });
                }
            }
        }
    }

    if heartbeats_by_project.is_empty() {
        // No heartbeats to sync, clean up empty file
        let _ = fs::remove_file(&heartbeat_file);
        return Ok(());
    }

    let mut synced_count = 0;
    let mut settled_commits = 0;

    for (project_path, mut entries) in heartbeats_by_project {
        // Sort entries chronologically
        entries.sort_by_key(|e| e.timestamp);

        // Get git info for session creation
        let project_path_buf = PathBuf::from(&project_path);
        let git_info = git::get_git_info(&project_path_buf).ok();

        // Load project-specific config for work_item_pattern
        let project_config = EffectiveConfig::load(Some(&project_path_buf)).unwrap_or(config.clone());

        // Get or create project
        let project = db.get_or_create_project(
            &project_path,
            git_info.as_ref().and_then(|g| g.remote_url.as_deref()),
            project_config.project_name.as_deref(),
            Some(&project_config.work_item_pattern),
            project_config.work_item_source.map(|s| s.as_str()),
        ).await?;

        // Get or create active session for this project
        let session = match db.get_active_session(project.id).await? {
            Some(s) => s,
            None => {
                // Auto-create session (statusline-first design)
                let branch = git_info
                    .as_ref()
                    .map(|g| g.branch.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let start_commit = git_info.as_ref().and_then(|g| g.head_commit.clone());

                // Extract work item from branch name
                let work_item = regex::Regex::new(&project_config.work_item_pattern)
                    .ok()
                    .and_then(|re| {
                        re.captures(&branch).and_then(|caps| {
                            caps.get(1).or_else(|| caps.get(0)).map(|m| m.as_str().to_string())
                        })
                    });

                // Use earliest heartbeat timestamp as session start time
                let earliest_ts = entries.first().map(|e| e.timestamp);

                let session = db.create_session_at(
                    project.id,
                    &branch,
                    work_item.as_deref(),
                    start_commit.as_deref(),
                    earliest_ts,
                ).await?;

                eprintln!(
                    "Auto-created session for: {} (branch: {}{})",
                    project.display_name.as_deref().unwrap_or(&project_path),
                    branch,
                    work_item.map(|w| format!(", work_item: {}", w)).unwrap_or_default()
                );

                session
            }
        };

        // Get last heartbeat from DB to check for commit changes
        let last_db_heartbeat = db.get_last_heartbeat(session.id).await?;
        let mut prev_commit = last_db_heartbeat.and_then(|h| h.commit_hash);

        // Record each heartbeat and check for commit changes
        for entry in &entries {
            // Check if commit changed
            if let (Some(ref prev), Some(ref current)) = (&prev_commit, &entry.commit_hash) {
                if prev != current {
                    // Commit changed! Settle time for the previous commit
                    let heartbeats = db.get_heartbeats_for_commit(session.id, prev).await?;
                    if !heartbeats.is_empty() {
                        let active_seconds = calculate_active_time_simple(&heartbeats, project_config.idle_timeout_minutes);
                        db.upsert_commit_time(session.id, prev, None, None, active_seconds).await?;
                        settled_commits += 1;
                    }
                }
            }

            // Record heartbeat with commit hash
            db.record_heartbeat_with_commit(session.id, entry.timestamp, entry.commit_hash.as_deref()).await?;
            synced_count += 1;

            // Update prev_commit for next iteration
            prev_commit = entry.commit_hash.clone();
        }
    }

    // Clear the cache file after successful sync
    fs::remove_file(&heartbeat_file)
        .with_context(|| "Failed to remove heartbeat cache after sync")?;

    if synced_count > 0 {
        if settled_commits > 0 {
            eprintln!("Synced {} heartbeats, settled {} commits", synced_count, settled_commits);
        } else {
            eprintln!("Synced {} heartbeats to database", synced_count);
        }
    }

    Ok(())
}

/// Simple active time calculation for sync command
fn calculate_active_time_simple(heartbeats: &[models::Heartbeat], idle_timeout_minutes: u32) -> i64 {
    if heartbeats.is_empty() {
        return 0;
    }

    let timeout_seconds = (idle_timeout_minutes as i64) * 60;
    let mut total_seconds: i64 = 0;

    for window in heartbeats.windows(2) {
        let interval = (window[1].timestamp - window[0].timestamp).num_seconds();

        if interval <= timeout_seconds {
            total_seconds += interval;
        }
    }

    total_seconds
}

/// Push local database data to Turso (one-time migration)
/// Uses SQL export/import for simplicity
async fn cmd_push_to_remote(dry_run: bool) -> Result<()> {
    let config = EffectiveConfig::load(None)?;

    if !config.is_turso_enabled() {
        println!("Turso is not configured. Nothing to push.");
        return Ok(());
    }

    println!("Connecting to local database...");
    let local_db = Database::open_local(&config.database_path).await?;

    // Get summary of local data
    let local_projects = local_db.list_projects().await?;
    let sessions_count = local_db.count_sessions().await?;

    println!("\nLocal database contains:");
    println!("  {} projects", local_projects.len());
    println!("  {} sessions", sessions_count);

    if dry_run {
        println!("\n[Dry run] Would sync the following projects to Turso:");
        for project in &local_projects {
            println!("  - {} ({})",
                project.display_name.as_deref().unwrap_or("-"),
                project.path
            );
        }
        println!("\nRun without --dry-run to perform the sync.");
        return Ok(());
    }

    println!("\nConnecting to Turso...");
    let remote_db = Database::open_remote(
        config.turso_url.as_ref().unwrap(),
        config.turso_auth_token.as_ref().unwrap(),
    ).await?;

    // Sync using INSERT OR IGNORE for idempotency
    println!("\nSyncing data to Turso...");

    let synced = local_db.export_and_import_to(&remote_db).await?;

    println!("\nSync complete!");
    println!("  Projects: {}", synced.projects);
    println!("  Sessions: {}", synced.sessions);
    println!("  Heartbeats: {}", synced.heartbeats);
    println!("  Commits: {}", synced.commits);

    Ok(())
}

/// Generate powerlevel10k-style statusline for Claude Code
async fn cmd_statusline() -> Result<()> {
    use std::io::{self, Read};

    // Read JSON input from stdin
    let mut input_str = String::new();
    io::stdin().read_to_string(&mut input_str)?;

    let input: statusline::StatuslineInput = serde_json::from_str(&input_str)
        .context("Failed to parse statusline input JSON")?;

    // Get cwd for config and db
    let cwd = input.cwd.as_deref().map(PathBuf::from);

    // Load config (silently, no errors in statusline)
    let config = match cwd.as_ref() {
        Some(p) => EffectiveConfig::load(Some(p)).ok(),
        None => EffectiveConfig::load(None).ok(),
    };

    // Generate and output statusline
    let output = if let Some(ref cfg) = config {
        // Try to open database for time tracking (optional, don't fail if unavailable)
        let db = open_db(cfg).await.ok();
        statusline::generate(&input, db.as_ref(), cfg).await
    } else {
        // No config available, generate minimal statusline
        statusline::generate_minimal(&input)
    };

    print!("{}", output);

    Ok(())
}
