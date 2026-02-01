# Implementation Plan: Statusline Enhancements

## Stage 1: Pure Rust Git (replace subprocess with gix)
**Goal**: Remove all `std::process::Command` calls to `git`, use `gix` for everything
**Success Criteria**: No subprocess spawning for git operations; same output as before
**Tests**: `cargo test` passes; manual statusline output comparison
**Status**: Not Started

### Changes
- `statusline.rs`: Rewrite `get_git_file_counts()` using `gix` status API
- `statusline.rs`: Rewrite `get_git_sync_status()` using `gix` rev-walk to count ahead/behind
- Remove `use std::process::Command` from statusline module

### Notes
- gix 0.66 with `basic` feature includes `gix::status` for file-level dirty tracking
- Use `repo.status()` iterator to count staged/unstaged/untracked
- For ahead/behind: walk from HEAD to upstream merge-base, count commits each way

---

## Stage 2: Git Detailed Status + Ahead/Behind Display
**Goal**: Show `+3 ~2 ?1 ⇡2 ⇣1` instead of just `≢`
**Success Criteria**: Git bubble shows specific counts for each status type
**Tests**: Manual verification with repos in various states
**Status**: Not Started

### Changes
- `statusline.rs`: Update `generate()` and `generate_minimal()` git content formatting
- Format: `+N` staged, `~N` modified, `?N` untracked, `⇡N` ahead, `⇣N` behind
- Only show non-zero counts

---

## Stage 3: Branch Name Truncation
**Goal**: Long branch names auto-truncate to ~25 chars with `…`
**Success Criteria**: `feature/very-long-branch-name-here` → `feature/very-long-bra…`
**Tests**: Test with branches of varying lengths
**Status**: Not Started

### Changes
- `statusline.rs`: Add `truncate_branch()` helper
- Max 25 chars, append `…` if truncated

---

## Stage 4: Python Version Detection
**Goal**: Show Python version from `.tool-versions` or `.python-version`
**Success Criteria**: Python version appears in info bubble when available
**Tests**: Verify with projects that have `.python-version` or `.tool-versions`
**Status**: Not Started

### Changes
- `statusline.rs`: Add `ICON_PYTHON` constant
- `statusline.rs`: Check `.tool-versions` (python key) and `.python-version` fallback
- Add to info bubble between Node and Timer

---

## Stage 5: Git Status Caching
**Goal**: Cache git status for 2 seconds to avoid redundant git operations
**Success Criteria**: Repeated statusline calls within 2s return cached result; <5ms response
**Tests**: Benchmark with `time` command; verify cache invalidation
**Status**: Not Started

### Changes
- `statusline.rs`: Add file-based cache at `~/.cache/claude-time-tracker/git_status_cache`
- Cache format: JSON with timestamp + path + GitStatus
- TTL: 2 seconds
- Read cache → if fresh, use it; otherwise compute and write
