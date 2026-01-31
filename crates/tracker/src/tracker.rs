use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use regex::Regex;
use std::path::Path;

use crate::config::EffectiveConfig;
use crate::db::Database;
use crate::git;
use crate::models::SessionStatus;

/// Start a new tracking session
pub async fn start_session(db: &Database, project_path: &Path, config: &EffectiveConfig) -> Result<()> {
    let path_str = project_path
        .to_str()
        .context("Invalid project path")?;

    // Get git information
    let git_info = git::get_git_info(project_path).ok();

    // Check for abandoned sessions and close them
    close_abandoned_sessions(db, config).await?;

    // Get or create project
    let project = db.get_or_create_project(
        path_str,
        git_info.as_ref().and_then(|g| g.remote_url.as_deref()),
        config.project_name.as_deref(),
        Some(&config.work_item_pattern),
        config.work_item_source.map(|s| s.as_str()),
    ).await?;

    // Check if there's already an active session for this project
    if let Some(existing) = db.get_active_session(project.id).await? {
        eprintln!(
            "Session already active for project (started at {})",
            existing.started_at
        );
        return Ok(());
    }

    // Extract work item from branch name
    let branch = git_info
        .as_ref()
        .map(|g| g.branch.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let work_item = extract_work_item(&branch, Some(&config.work_item_pattern));

    // Create new session
    let session = db.create_session(
        project.id,
        &branch,
        work_item.as_deref(),
        git_info.as_ref().and_then(|g| g.head_commit.as_deref()),
    ).await?;

    // Record initial heartbeat
    db.record_heartbeat(session.id).await?;

    eprintln!(
        "Started tracking: {} (branch: {}, work_item: {})",
        config.project_name.as_deref().unwrap_or(path_str),
        branch,
        work_item.as_deref().unwrap_or(&branch)
    );

    Ok(())
}

/// Record a heartbeat for the current session
/// If no active session exists, silently succeeds (session will be created on next start)
/// Also records the current HEAD commit for per-commit time tracking
/// When HEAD commit changes, immediately settles time for the previous commit
pub async fn record_heartbeat(db: &Database, project_path: &Path, config: &EffectiveConfig) -> Result<()> {
    let path_str = project_path
        .to_str()
        .context("Invalid project path")?;

    // If project doesn't exist, just return Ok (no session to track)
    let project = match db.get_project_by_path(path_str).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    // If no active session, just return Ok (session might have been stopped)
    let session = match db.get_active_session(project.id).await? {
        Some(s) => s,
        None => return Ok(()),
    };

    // Get current HEAD commit for per-commit time tracking
    let git_info = git::get_git_info(project_path).ok();
    let current_commit = git_info.as_ref().and_then(|g| g.head_commit.clone());

    // Get last heartbeat to check if commit changed
    let last_heartbeat = db.get_last_heartbeat(session.id).await?;
    let now = chrono::Utc::now();

    // Check if commit changed (new commit was made)
    if let (Some(ref last_hb), Some(ref current)) = (&last_heartbeat, &current_commit) {
        if let Some(ref prev_commit) = last_hb.commit_hash {
            if prev_commit != current {
                // Commit changed! Settle time for the previous commit
                settle_commit_time(db, session.id, prev_commit, config.idle_timeout_minutes).await?;
                eprintln!(
                    "Commit changed: {} -> {}, settled time for previous commit",
                    &prev_commit[..8.min(prev_commit.len())],
                    &current[..8.min(current.len())]
                );
            }
        }
    }

    // Record this heartbeat
    db.record_heartbeat_with_commit(session.id, now, current_commit.as_deref()).await?;

    Ok(())
}

/// Settle (calculate and save) time for a specific commit
async fn settle_commit_time(
    db: &Database,
    session_id: i64,
    commit_hash: &str,
    idle_timeout_minutes: u32,
) -> Result<()> {
    // Get all heartbeats for this commit
    let heartbeats = db.get_heartbeats_for_commit(session_id, commit_hash).await?;

    if heartbeats.is_empty() {
        return Ok(());
    }

    // Calculate active time for this commit
    let active_seconds = calculate_active_time(&heartbeats, idle_timeout_minutes);

    // Get commit message from git (if available)
    // For now, we'll leave message as None - it will be filled in when session ends

    // Save or update commit time
    db.upsert_commit_time(
        session_id,
        commit_hash,
        None,  // message will be filled later
        None,  // committed_at will be filled later
        active_seconds,
    ).await?;

    Ok(())
}

/// Stop the current tracking session
pub async fn stop_session(db: &Database, project_path: &Path, config: &EffectiveConfig) -> Result<()> {
    let path_str = project_path
        .to_str()
        .context("Invalid project path")?;

    let project = db
        .get_project_by_path(path_str).await?
        .context("Project not found")?;

    let session = match db.get_active_session(project.id).await? {
        Some(s) => s,
        None => {
            eprintln!("No active session to stop");
            return Ok(());
        }
    };

    // Get current git state
    let git_info = git::get_git_info(project_path).ok();
    let end_commit = git_info.as_ref().and_then(|g| g.head_commit.clone());

    // Calculate active time from heartbeats
    let heartbeats = db.get_heartbeats(session.id).await?;
    let active_seconds = calculate_active_time(&heartbeats, config.idle_timeout_minutes);

    // Calculate per-commit time from heartbeats
    let commit_times = calculate_per_commit_time(&heartbeats, config.idle_timeout_minutes);

    // Collect commits made during this session
    if let Some(ref start) = session.start_commit {
        match git::get_commits_between(
            project_path,
            Some(start),
            end_commit.as_deref(),
        ) {
            Ok(commits) => {
                if !commits.is_empty() {
                    // Enrich commits with their tracked time
                    let commits_with_time: Vec<_> = commits
                        .into_iter()
                        .map(|(hash, message, committed_at)| {
                            let time = commit_times.get(&hash).copied();
                            (hash, message, committed_at, time)
                        })
                        .collect();

                    db.record_commits_with_time(session.id, &commits_with_time).await?;

                    // Log commits with time
                    let tracked_count = commits_with_time.iter().filter(|(_, _, _, t)| t.is_some()).count();
                    eprintln!(
                        "Recorded {} commits for session ({} with time tracking)",
                        commits_with_time.len(),
                        tracked_count
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to collect commits for session {}: {}",
                    session.id, e
                );
            }
        }
    }

    // Complete the session
    db.complete_session(
        session.id,
        end_commit.as_deref(),
        active_seconds,
        SessionStatus::Completed,
    ).await?;

    let duration = format_duration(active_seconds);
    eprintln!(
        "Stopped tracking: {} (active time: {})",
        config.project_name.as_deref().unwrap_or(path_str),
        duration
    );

    Ok(())
}

/// Close any abandoned sessions (from previous runs that didn't properly stop)
async fn close_abandoned_sessions(db: &Database, config: &EffectiveConfig) -> Result<()> {
    let active_sessions = db.get_all_active_sessions().await?;

    for session in active_sessions {
        let heartbeats = db.get_heartbeats(session.id).await?;

        if let Some(last_heartbeat) = heartbeats.last() {
            let timeout = Duration::minutes(config.idle_timeout_minutes as i64);
            let cutoff = last_heartbeat.timestamp + timeout;

            if Utc::now() > cutoff {
                // Session is abandoned - close it
                let active_seconds = calculate_active_time(&heartbeats, config.idle_timeout_minutes);

                db.complete_session(session.id, None, active_seconds, SessionStatus::Abandoned).await?;

                eprintln!(
                    "Closed abandoned session {} (was active for {})",
                    session.id,
                    format_duration(active_seconds)
                );
            }
        }
    }

    Ok(())
}

/// Calculate active time from heartbeats
///
/// Active time is calculated by summing intervals between consecutive heartbeats,
/// but only counting intervals shorter than the idle timeout.
fn calculate_active_time(heartbeats: &[crate::models::Heartbeat], idle_timeout_minutes: u32) -> i64 {
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
        // If interval > timeout, we assume user was away, don't count it
    }

    total_seconds
}

/// Calculate per-commit time from heartbeats
///
/// Returns a map of commit_hash -> active_seconds for each commit.
/// Time is attributed to the commit that was HEAD during each heartbeat interval.
fn calculate_per_commit_time(
    heartbeats: &[crate::models::Heartbeat],
    idle_timeout_minutes: u32,
) -> std::collections::HashMap<String, i64> {
    use std::collections::HashMap;

    let mut commit_times: HashMap<String, i64> = HashMap::new();

    if heartbeats.is_empty() {
        return commit_times;
    }

    let timeout_seconds = (idle_timeout_minutes as i64) * 60;

    for window in heartbeats.windows(2) {
        let interval = (window[1].timestamp - window[0].timestamp).num_seconds();

        // Only count intervals within the idle timeout
        if interval <= timeout_seconds {
            // Attribute time to the commit at the START of the interval
            // (the commit we were working on before the next heartbeat)
            if let Some(ref commit_hash) = window[0].commit_hash {
                *commit_times.entry(commit_hash.clone()).or_insert(0) += interval;
            }
        }
    }

    commit_times
}

/// Extract work item ID from branch name using regex pattern
fn extract_work_item(branch: &str, pattern: Option<&str>) -> Option<String> {
    let pattern = pattern?;

    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let caps = re.captures(branch)?;

    // Return first capture group, or entire match if no groups
    caps.get(1)
        .or_else(|| caps.get(0))
        .map(|m| m.as_str().to_string())
}

/// Format duration in human-readable format
pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_work_item() {
        // Linear-style pattern
        let pattern = r"^(?:feature|fix|chore)/([A-Z]+-\d+)";

        assert_eq!(
            extract_work_item("feature/ABC-123-some-description", Some(pattern)),
            Some("ABC-123".to_string())
        );

        assert_eq!(
            extract_work_item("fix/XYZ-456-bug-fix", Some(pattern)),
            Some("XYZ-456".to_string())
        );

        assert_eq!(
            extract_work_item("main", Some(pattern)),
            None
        );

        // No pattern - should return None
        assert_eq!(
            extract_work_item("feature/ABC-123", None),
            None
        );
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0m");
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(3660), "1h 1m");
        assert_eq!(format_duration(7260), "2h 1m");
    }

    #[test]
    fn test_calculate_active_time() {
        use chrono::Duration;

        let base = Utc::now();
        let heartbeats = vec![
            crate::models::Heartbeat {
                id: 1,
                session_id: 1,
                timestamp: base,
                commit_hash: None,
            },
            crate::models::Heartbeat {
                id: 2,
                session_id: 1,
                timestamp: base + Duration::minutes(5),
                commit_hash: None,
            },
            crate::models::Heartbeat {
                id: 3,
                session_id: 1,
                timestamp: base + Duration::minutes(10),
                commit_hash: None,
            },
            // 20 minute gap (user was away)
            crate::models::Heartbeat {
                id: 4,
                session_id: 1,
                timestamp: base + Duration::minutes(30),
                commit_hash: None,
            },
            crate::models::Heartbeat {
                id: 5,
                session_id: 1,
                timestamp: base + Duration::minutes(35),
                commit_hash: None,
            },
        ];

        // With 10 minute timeout:
        // 5 min + 5 min (counted) + 20 min (not counted, > 10) + 5 min (counted) = 15 min = 900 seconds
        let active = calculate_active_time(&heartbeats, 10);
        assert_eq!(active, 900);
    }

    #[test]
    fn test_calculate_per_commit_time() {
        use chrono::Duration;

        let base = Utc::now();
        let heartbeats = vec![
            crate::models::Heartbeat {
                id: 1,
                session_id: 1,
                timestamp: base,
                commit_hash: Some("abc123".to_string()),
            },
            crate::models::Heartbeat {
                id: 2,
                session_id: 1,
                timestamp: base + Duration::minutes(5),
                commit_hash: Some("abc123".to_string()),
            },
            crate::models::Heartbeat {
                id: 3,
                session_id: 1,
                timestamp: base + Duration::minutes(10),
                commit_hash: Some("def456".to_string()), // New commit
            },
            crate::models::Heartbeat {
                id: 4,
                session_id: 1,
                timestamp: base + Duration::minutes(15),
                commit_hash: Some("def456".to_string()),
            },
            crate::models::Heartbeat {
                id: 5,
                session_id: 1,
                timestamp: base + Duration::minutes(18),
                commit_hash: Some("ghi789".to_string()), // Another commit
            },
        ];

        let commit_times = calculate_per_commit_time(&heartbeats, 10);

        // abc123: 5 min (base to +5) + 5 min (+5 to +10) = 10 min = 600 seconds
        assert_eq!(commit_times.get("abc123"), Some(&600));

        // def456: 5 min (+10 to +15) + 3 min (+15 to +18) = 8 min = 480 seconds
        assert_eq!(commit_times.get("def456"), Some(&480));

        // ghi789: no time (it's the last commit, no interval after it)
        assert_eq!(commit_times.get("ghi789"), None);
    }
}
