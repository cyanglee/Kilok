//! Idle detection based on Claude transcript file activity
//!
//! This module checks the modification time of Claude's transcript files
//! to determine if the user is actively working or idle.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Get the Claude projects directory for a given project path
///
/// Claude stores transcripts in ~/.claude/projects/-<path-with-dashes>/
/// e.g., /Users/take5/workspace/rust/foo -> -Users-take5-workspace-rust-foo
fn get_claude_project_dir(project_path: &Path) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let claude_projects = home.join(".claude/projects");

    // Convert path to Claude's directory naming format
    // /Users/take5/foo -> -Users-take5-foo
    let path_str = project_path.to_string_lossy();
    let dir_name = format!("-{}", path_str.replace('/', "-"));

    let project_dir = claude_projects.join(dir_name);

    if project_dir.exists() {
        Some(project_dir)
    } else {
        None
    }
}

/// Get the modification time of the most recently modified transcript file
fn get_latest_transcript_mtime(dir: &Path) -> Option<SystemTime> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        // Exclude .wakatime state files
        .filter(|e| {
            !e.path()
                .file_name()
                .map(|n| n.to_string_lossy().contains(".wakatime"))
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .max()
}

/// Check if a project is idle based on transcript file activity
///
/// Returns true if the project is idle (no transcript activity for longer than timeout),
/// false if the project is active or if we can't determine the state.
///
/// When we can't find transcript files, we assume the project is active
/// (fail-open to avoid missing heartbeats).
pub fn is_project_idle(project_path: &Path, idle_timeout_minutes: u32) -> bool {
    // Find Claude project directory
    let transcript_dir = match get_claude_project_dir(project_path) {
        Some(dir) => dir,
        None => return false, // No transcript dir = assume active
    };

    // Get latest transcript modification time
    let latest_mtime = match get_latest_transcript_mtime(&transcript_dir) {
        Some(mtime) => mtime,
        None => return false, // No transcripts = assume active
    };

    // Check if mtime is older than timeout
    let timeout = Duration::from_secs((idle_timeout_minutes as u64) * 60);
    let elapsed = SystemTime::now()
        .duration_since(latest_mtime)
        .unwrap_or(Duration::ZERO);

    elapsed > timeout
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_get_latest_transcript_mtime() {
        let dir = tempdir().unwrap();

        // Create some test files
        File::create(dir.path().join("session1.jsonl")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        File::create(dir.path().join("session2.jsonl")).unwrap();
        File::create(dir.path().join("session1.jsonl.wakatime")).unwrap();

        let mtime = get_latest_transcript_mtime(dir.path());
        assert!(mtime.is_some());
    }
}
