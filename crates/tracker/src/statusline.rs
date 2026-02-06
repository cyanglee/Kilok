//! Powerlevel10k-style statusline for Claude Code
//!
//! Generates a rich, colorful status bar with git info, project details,
//! and time tracking integration.

use crate::config::EffectiveConfig;
use crate::db::Database;
use crate::git;
use crate::weather;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Write;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

// VCS status cache TTL in seconds
const VCS_CACHE_TTL_SECS: u64 = 5;

// ===== Nerd Font Icons =====
const ICON_APPLE: &str = "\u{f179}";      //
const ICON_FOLDER: &str = "\u{f07b}";     //
const ICON_GIT: &str = "\u{e725}";        //
const ICON_JJ: &str = "◇";               // jj diamond
const ICON_CHECK: &str = "\u{2714}";      // ✔
const ICON_CROSS: &str = "\u{2718}";      // ✘
const ICON_RUBY: &str = "\u{e791}";       //
const ICON_PYTHON: &str = "\u{e73c}";     //
const ICON_NODE: &str = "\u{e718}";       //
const ICON_CHIP: &str = "\u{f2db}";       //  (CPU/AI chip icon)
const ICON_TIMER: &str = "\u{23f1}";      // ⏱
const ICON_BRAIN: &str = "\u{f5dc}";      //  (context/brain icon)

// ===== Oh My Posh Round Separators =====
const ROUND_LEFT: &str = "\u{e0b6}";      //  (left half circle)
const ROUND_RIGHT: &str = "\u{e0b4}";     //  (right half circle)

// ===== ANSI 256-Color Codes (Ayu Mirage) =====
const BG_SURFACE: u8 = 239;   // Dark surface for git
const BG_ACCENT1: u8 = 215;   // Orange/Gold - directory
const BG_ACCENT2: u8 = 74;    // Blue/Cyan - model bubble
const BG_TIMER: u8 = 114;     // Green - timer bubble
const BG_WEATHER: u8 = 183;   // Purple/Lavender - weather bubble
const FG_TEXT: u8 = 252;      // Light text
const FG_DARK: u8 = 235;      // Dark text on light bg

/// Input data for statusline generation
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatuslineInput {
    pub cwd: Option<String>,
    pub model: Option<ModelInfo>,
    pub context_window: Option<ContextWindow>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelInfo {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContextWindow {
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
    pub context_window_size: Option<u64>,
}

/// Detected VCS presence in a directory (both can be true for jj+git repos)
#[derive(Debug, Default)]
struct VcsPresence {
    has_jj: bool,
    has_git: bool,
}

/// Detect all VCS systems present by walking up directories.
/// In jj's git-compatible mode, both .jj/ and .git/ coexist.
fn detect_vcs(path: &Path) -> VcsPresence {
    let mut presence = VcsPresence::default();
    let mut current = Some(path);
    while let Some(dir) = current {
        if !presence.has_jj && dir.join(".jj").is_dir() {
            presence.has_jj = true;
        }
        if !presence.has_git && dir.join(".git").exists() {
            presence.has_git = true;
        }
        if presence.has_jj && presence.has_git {
            break;
        }
        current = dir.parent();
    }
    presence
}

/// Git status information
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct GitStatus {
    branch: String,
    is_dirty: bool,
    staged: usize,
    unstaged: usize,
    untracked: usize,
    ahead: usize,
    behind: usize,
}

/// jj (Jujutsu) status information
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct JjStatus {
    change_id: String,
    bookmarks: Vec<String>,
    is_empty: bool,
    has_conflict: bool,
    modified_count: usize,
}

/// Cached VCS status with timestamp (tagged enum for git/jj)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum CachedVcsStatus {
    Git(GitStatus),
    Jj(JjStatus),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct VcsCache {
    status: CachedVcsStatus,
    #[serde(with = "chrono::serde::ts_seconds")]
    cached_at: chrono::DateTime<chrono::Utc>,
}

/// Generate minimal statusline when config is unavailable
pub fn generate_minimal(input: &StatuslineInput) -> String {
    let cwd = input.cwd.as_deref().unwrap_or(".");
    let cwd_path = PathBuf::from(cwd);

    let mut out = String::new();

    // === Bubble 1: Directory (Blue) ===
    let short_dir = shorten_path(&cwd_path);
    write_bubble(&mut out, BG_ACCENT1, FG_DARK, &format!("{} {}", ICON_FOLDER, short_dir));

    out.push(' ');

    // === Bubble 2+3: VCS Status (separate bubbles for jj and git) ===
    let vcs = detect_vcs(&cwd_path);

    if vcs.has_jj {
        if let Some(ref js) = get_jj_status(&cwd_path) {
            let mut jj_content = format!("{} {}", ICON_JJ, js.change_id);
            if let Some(first_bm) = js.bookmarks.first() {
                jj_content.push_str(&format!(" {}", first_bm));
            }
            if js.modified_count > 0 {
                jj_content.push_str(" ≢");
            }
            if js.has_conflict {
                jj_content.push_str(" ⚡");
            }
            write_bubble(&mut out, BG_SURFACE, FG_TEXT, &jj_content);
            out.push(' ');
        }
    }

    if vcs.has_git {
        if let Some(ref gs) = get_git_status(&cwd_path) {
            let mut git_content = format!("{} {}", ICON_GIT, gs.branch);
            if gs.staged > 0 || gs.unstaged > 0 || gs.untracked > 0 {
                git_content.push_str(" ≢");
            }
            write_bubble(&mut out, BG_SURFACE, FG_TEXT, &git_content);
            out.push(' ');
        }
    }

    // === Model + Context (Mauve/Purple) ===
    let model_name = extract_model_name(input);
    let mut info_content = format!("{} {}", ICON_CHIP, model_name);

    if let Some(ref ctx) = input.context_window {
        let percent = ctx.used_percentage.unwrap_or(0.0);
        info_content.push_str(&format!(" {}", format_progress_bar(percent)));
    }

    write_bubble(&mut out, BG_ACCENT2, FG_DARK, &info_content);

    out
}

/// Generate the statusline output
pub async fn generate(input: &StatuslineInput, db: Option<&Database>, config: &EffectiveConfig) -> String {
    let cwd = input.cwd.as_deref().unwrap_or(".");
    let cwd_path = PathBuf::from(cwd);

    let mut out = String::new();

    // === Bubble 1: Directory (Blue) ===
    let short_dir = shorten_path(&cwd_path);
    write_bubble(&mut out, BG_ACCENT1, FG_DARK, &format!("{} {}", ICON_FOLDER, short_dir));

    out.push(' ');

    // === Bubble 2+3: VCS Status (separate bubbles for jj and git) ===
    let vcs = detect_vcs(&cwd_path);

    if vcs.has_jj {
        if let Some(ref js) = get_jj_status(&cwd_path) {
            let mut jj_content = format!("{} {}", ICON_JJ, js.change_id);

            // Show bookmarks (space-separated)
            for bm in &js.bookmarks {
                jj_content.push_str(&format!(" {}", truncate_branch(bm, 25)));
            }

            // Modified file count (jj has no staging area)
            if js.modified_count > 0 {
                jj_content.push_str(&format!(" ✎{}", js.modified_count));
            }

            // Conflict indicator
            if js.has_conflict {
                jj_content.push_str(" ⚡");
            }

            write_bubble(&mut out, BG_SURFACE, FG_TEXT, &jj_content);
            out.push(' ');
        }
    }

    if vcs.has_git {
        if let Some(ref gs) = get_git_status(&cwd_path) {
            // Truncate branch name if too long (max 35 chars)
            let branch_display = truncate_branch(&gs.branch, 35);
            let mut git_content = format!("{} {}", ICON_GIT, branch_display);

            // Add detailed status indicators with clear labels
            // ✚ staged (ready to commit), ✎ modified (not staged), ★ new files
            let mut status_parts: Vec<String> = Vec::new();
            if gs.staged > 0 {
                status_parts.push(format!("✚{}", gs.staged));  // staged/ready
            }
            if gs.unstaged > 0 {
                status_parts.push(format!("✎{}", gs.unstaged));  // modified/edited
            }
            if gs.untracked > 0 {
                status_parts.push(format!("★{}", gs.untracked));  // untracked/new
            }
            if !status_parts.is_empty() {
                git_content.push_str(&format!(" {}", status_parts.join(" ")));
            }

            // Add ahead/behind indicators: ⇡ahead ⇣behind
            if gs.ahead > 0 || gs.behind > 0 {
                let mut sync_parts: Vec<String> = Vec::new();
                if gs.ahead > 0 {
                    sync_parts.push(format!("⇡{}", gs.ahead));
                }
                if gs.behind > 0 {
                    sync_parts.push(format!("⇣{}", gs.behind));
                }
                git_content.push_str(&format!(" {}", sync_parts.join("")));
            }

            write_bubble(&mut out, BG_SURFACE, FG_TEXT, &git_content);
            out.push(' ');
        }
    }

    // === Model + Tools (Blue) ===
    let model_name = extract_model_name(input);
    let mut model_parts: Vec<String> = Vec::new();

    // Model
    model_parts.push(format!("{} {}", ICON_CHIP, model_name));

    // Ruby version
    if let Some(version) = get_tool_version(&cwd_path, "ruby") {
        model_parts.push(format!("{} {}", ICON_RUBY, version));
    }

    // Python version
    if let Some(version) = get_tool_version(&cwd_path, "python") {
        model_parts.push(format!("{} {}", ICON_PYTHON, version));
    }

    // Node version
    if let Some(version) = get_tool_version(&cwd_path, "nodejs") {
        model_parts.push(format!("{} {}", ICON_NODE, version));
    }

    // Context usage (stays with model info)
    if let Some(ref ctx) = input.context_window {
        let percent = ctx.used_percentage.unwrap_or(0.0);
        model_parts.push(format_progress_bar(percent));
    }

    write_bubble(&mut out, BG_ACCENT2, FG_DARK, &model_parts.join(" "));

    // === Bubble 4: Timer (Green) ===
    if let Some(tracker_time) = get_tracker_time(&cwd_path, db, config).await {
        out.push(' ');
        write_bubble(&mut out, BG_TIMER, FG_DARK, &format!("{} {}", ICON_TIMER, tracker_time));
    }

    // === Bubble 5: Weather (Purple) ===
    if let Some(weather_info) = weather::get_weather(&config.weather).await {
        out.push(' ');
        write_bubble(&mut out, BG_WEATHER, FG_DARK, &weather::format_weather(&weather_info));
    }

    out
}

/// Write a rounded bubble segment (Oh My Posh style)
fn write_bubble(out: &mut String, bg: u8, fg: u8, content: &str) {
    // Left round cap (foreground color = bg color, no background)
    write!(out, "\x1b[38;5;{}m{}\x1b[0m", bg, ROUND_LEFT).unwrap();
    // Content with background
    write!(out, "\x1b[48;5;{}m\x1b[38;5;{}m {} \x1b[0m", bg, fg, content).unwrap();
    // Right round cap
    write!(out, "\x1b[38;5;{}m{}\x1b[0m", bg, ROUND_RIGHT).unwrap();
}

/// Format context usage progress bar (returns string, no ANSI in bubble)
fn format_progress_bar(percent: f64) -> String {
    const BAR_WIDTH: usize = 5;
    const BAR_FILLED: &str = "━";  // Smoother bar character
    const BAR_EMPTY: &str = "┈";   // Lighter empty character

    let filled = ((percent / 100.0) * BAR_WIDTH as f64).round() as usize;
    let empty = BAR_WIDTH.saturating_sub(filled);

    let bar: String = BAR_FILLED.repeat(filled) + &BAR_EMPTY.repeat(empty);

    format!("{:.0}% [{}]", percent, bar)  // Added space and brackets
}

/// Shorten a path for display (like ~/w/r/landtop)
fn shorten_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    // Replace home dir with ~
    let home = dirs::home_dir()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    let path_str = if !home.is_empty() && path_str.starts_with(&home) {
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str.to_string()
    };

    // Shorten intermediate directories to first letter
    let parts: Vec<&str> = path_str.split('/').collect();
    if parts.len() <= 2 {
        return path_str;
    }

    let mut result = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Keep last part full
            result.push(part.to_string());
        } else if part.is_empty() || *part == "~" {
            result.push(part.to_string());
        } else {
            // First character only
            result.push(part.chars().next().map(|c| c.to_string()).unwrap_or_default());
        }
    }

    result.join("/")
}

/// Truncate branch name intelligently
/// - If branch starts with common prefixes (feature/, bugfix/, etc.), keep prefix + truncated name
/// - Otherwise truncate from the end with ellipsis
fn truncate_branch(branch: &str, max_len: usize) -> String {
    if branch.len() <= max_len {
        return branch.to_string();
    }

    // Common branch prefixes to preserve
    let prefixes = ["feature/", "bugfix/", "hotfix/", "release/", "fix/", "feat/"];

    for prefix in prefixes {
        if branch.starts_with(prefix) {
            let suffix = &branch[prefix.len()..];
            let remaining = max_len.saturating_sub(prefix.len() + 2); // 2 for ".."
            if remaining > 3 {
                return format!("{}{}…", prefix, &suffix[..remaining.min(suffix.len())]);
            }
        }
    }

    // Default: truncate with ellipsis
    let truncate_at = max_len.saturating_sub(1);
    format!("{}…", &branch[..truncate_at.min(branch.len())])
}

/// Get cache file path for a repository
fn get_cache_path(path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hash = hasher.finish();

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("claude-time-tracker")
        .join("vcs-status");

    // Create cache directory if it doesn't exist
    let _ = fs::create_dir_all(&cache_dir);

    cache_dir.join(format!("{:x}.json", hash))
}

/// Try to read VCS status from cache
fn read_vcs_cache(path: &Path) -> Option<CachedVcsStatus> {
    let cache_path = get_cache_path(path);
    let content = fs::read_to_string(&cache_path).ok()?;
    let cached: VcsCache = serde_json::from_str(&content).ok()?;

    // Check if cache is still valid
    let age = (Utc::now() - cached.cached_at).num_seconds();
    if age >= 0 && age < VCS_CACHE_TTL_SECS as i64 {
        Some(cached.status)
    } else {
        None
    }
}

/// Write VCS status to cache
fn write_vcs_cache(path: &Path, status: CachedVcsStatus) {
    let cache_path = get_cache_path(path);
    let cached = VcsCache {
        status,
        cached_at: Utc::now(),
    };

    if let Ok(json) = serde_json::to_string(&cached) {
        let _ = fs::write(&cache_path, json);
    }
}

/// Get git status for a directory (with caching)
fn get_git_status(path: &Path) -> Option<GitStatus> {
    // Try cache first
    if let Some(CachedVcsStatus::Git(cached)) = read_vcs_cache(path) {
        return Some(cached);
    }

    // Cache miss - fetch fresh status
    // Use gix for branch info
    let repo = gix::open(path).ok()?;
    let head = repo.head().ok()?;

    let branch = if let Some(name) = head.referent_name() {
        let full_name = name.as_bstr().to_string();
        full_name.strip_prefix("refs/heads/").unwrap_or(&full_name).to_string()
    } else if let Some(id) = head.id() {
        format!(":{}", &id.to_string()[..7])
    } else {
        return None;
    };

    // For status counts, use git command (gix status is complex)
    let (staged, unstaged, untracked) = get_git_file_counts(path);
    let (ahead, behind) = get_git_sync_status(path);

    let is_dirty = staged > 0 || unstaged > 0 || untracked > 0;

    let status = GitStatus {
        branch,
        is_dirty,
        staged,
        unstaged,
        untracked,
        ahead,
        behind,
    };

    // Write to cache
    write_vcs_cache(path, CachedVcsStatus::Git(status.clone()));

    Some(status)
}

/// Get jj (Jujutsu) status for a directory (with caching)
fn get_jj_status(path: &Path) -> Option<JjStatus> {
    // Try cache first
    if let Some(CachedVcsStatus::Jj(cached)) = read_vcs_cache(path) {
        return Some(cached);
    }

    // Cache miss - fetch fresh status via jj CLI
    let status = fetch_jj_status(path)?;

    // Write to cache
    write_vcs_cache(path, CachedVcsStatus::Jj(status.clone()));

    Some(status)
}

/// Fetch jj status from CLI
fn fetch_jj_status(path: &Path) -> Option<JjStatus> {
    use std::process::Command;

    // Get change_id, bookmarks, empty/conflict status in one call
    let template = r#"concat(change_id.short(8), "\n", bookmarks.join(","), "\n", if(empty, "empty", "modified"), "\n", if(conflict, "conflict", "clean"))"#;
    let log_output = Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", template, "-R", &path.to_string_lossy()])
        .output()
        .ok()?;

    if !log_output.status.success() {
        return None;
    }

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    let lines: Vec<&str> = log_str.trim().lines().collect();
    if lines.len() < 4 {
        return None;
    }

    let change_id = lines[0].to_string();
    let bookmarks: Vec<String> = lines[1]
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    let is_empty = lines[2] == "empty";
    let has_conflict = lines[3] == "conflict";

    // Get modified file count from jj diff --summary
    let diff_output = Command::new("jj")
        .args(["diff", "--summary", "-R", &path.to_string_lossy()])
        .output()
        .ok()?;

    let modified_count = if diff_output.status.success() {
        String::from_utf8_lossy(&diff_output.stdout)
            .trim()
            .lines()
            .count()
    } else {
        0
    };

    Some(JjStatus {
        change_id,
        bookmarks,
        is_empty,
        has_conflict,
        modified_count,
    })
}

/// Get git file status counts using single git status --porcelain call
/// This is 4x faster than the original 4 separate subprocess calls
fn get_git_file_counts(path: &Path) -> (usize, usize, usize) {
    use std::process::Command;

    let output = match Command::new("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain=v1"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return (0, 0, 0),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut staged = 0usize;
    let mut unstaged = 0usize;
    let mut untracked = 0usize;

    for line in stdout.lines() {
        if line.len() < 2 {
            continue;
        }

        let index_status = line.chars().next().unwrap_or(' ');
        let worktree_status = line.chars().nth(1).unwrap_or(' ');

        // Index status (staged changes)
        match index_status {
            'A' | 'M' | 'D' | 'R' | 'C' => staged += 1,
            _ => {}
        }

        // Worktree status (unstaged changes)
        match worktree_status {
            'M' | 'D' => unstaged += 1,
            _ => {}
        }

        // Untracked files
        if index_status == '?' && worktree_status == '?' {
            untracked += 1;
        }
    }

    (staged, unstaged, untracked)
}

/// Get ahead/behind counts relative to upstream using single git command
fn get_git_sync_status(path: &Path) -> (usize, usize) {
    use std::process::Command;

    // First check if upstream exists
    let upstream_check = Command::new("git")
        .args(["-C", &path.to_string_lossy(), "rev-parse", "--abbrev-ref", "@{upstream}"])
        .output();

    if !matches!(upstream_check, Ok(ref o) if o.status.success()) {
        return (0, 0);
    }

    // Get ahead/behind in single command
    let output = match Command::new("git")
        .args(["-C", &path.to_string_lossy(), "rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return (0, 0),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split_whitespace().collect();

    if parts.len() == 2 {
        let ahead = parts[0].parse().unwrap_or(0);
        let behind = parts[1].parse().unwrap_or(0);
        (ahead, behind)
    } else {
        (0, 0)
    }
}

/// Get tool version from .tool-versions file
fn get_tool_version(path: &Path, tool: &str) -> Option<String> {
    let tool_versions = path.join(".tool-versions");
    if !tool_versions.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&tool_versions).ok()?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == tool {
            return Some(parts[1].to_string());
        }
    }

    None
}

/// Extract model name from input
fn extract_model_name(input: &StatuslineInput) -> String {
    input.model.as_ref().and_then(|m| {
        m.name.as_ref()
            .or(m.display_name.as_ref())
            .or(m.id.as_ref())
    })
    .map(|name| {
        // Shorten model name
        let name = name.strip_prefix("Claude ").unwrap_or(name);
        let name = name.strip_prefix("claude-").unwrap_or(name);
        // Remove date suffix like -20251101
        if let Some(pos) = name.rfind("-20") {
            &name[..pos]
        } else {
            name
        }
    })
    .unwrap_or("unknown")
    .to_string()
}

/// Get active time from tracker
async fn get_tracker_time(path: &Path, db: Option<&Database>, config: &EffectiveConfig) -> Option<String> {
    let db = db?;

    let path_str = path.to_str()?;
    let project = db.get_project_by_path(path_str).await.ok()??;
    let session = db.get_active_session(project.id).await.ok()??;
    let heartbeats = db.get_heartbeats(session.id).await.ok()?;

    let active_seconds = calculate_active_time(&heartbeats, config.idle_timeout_minutes);

    if active_seconds <= 0 {
        return None;
    }

    let hours = active_seconds / 3600;
    let mins = (active_seconds % 3600) / 60;

    if hours > 0 {
        Some(format!("{}h{}m", hours, mins))
    } else {
        Some(format!("{}m", mins))
    }
}

/// Calculate active time from heartbeats
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
