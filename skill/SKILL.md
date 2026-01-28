---
name: kilok
description: Use when user asks about time tracking, generating work reports, setting up project billing configs, or checking current session status
---

# Kilok - Claude Time Tracker

## Language Requirement

**All text output MUST be in Traditional Chinese (繁體中文) using terms and expressions commonly used in Taiwan.**

This applies to:
- Translated work item titles
- Summarized descriptions
- Report headers and labels
- Any user-facing text in the generated report

---

## Interaction Behavior

When the user invokes `/kilok`:

1. **If the request is clear** (e.g., "generate this month's report", "check status"), execute the corresponding action directly.
2. **If the request is vague or just the command name**, use `AskUserQuestion` to clarify:

```
你想要做什麼？

選項：
1. 產生時間報告（Markdown 格式）
2. 產生 Google Sheets 格式（TSV）
3. 查看目前追蹤狀態
4. 管理專案設定
```

---

## Overview

CLI tool for tracking Claude Code usage time per project. Integrates via hooks to automatically track sessions.

## Quick Reference

| Command | Purpose |
|---------|---------|
| `claude-time-tracker status` | Show active sessions |
| `claude-time-tracker report` | Generate current month report |
| `claude-time-tracker report --month 2025-01` | Specific month report |
| `claude-time-tracker report --all-formats -o ~/reports/jan` | Export md, csv, tsv, json |
| `claude-time-tracker report --format tsv` | TSV format for Google Sheets |
| `claude-time-tracker projects list` | List tracked projects |
| `claude-time-tracker config show` | Show current settings |

---

## Report Generation Workflow

When the user requests a time report, follow this workflow to produce a **human-readable, translated report**.

### Step 1: Fetch Raw JSON Data

```bash
claude-time-tracker report --format json [other options]
```

### Step 2: Transform Work Items

For each `work_item` in the JSON output, perform two transformations:

#### 2a. Translate Work Item ID to Human-Readable Title

Convert the technical `id` field (typically a kebab-case branch name fragment) into a descriptive title in **Traditional Chinese (Taiwan)**.

| Raw ID | Translated Title |
|--------|------------------|
| `invoice-enhancements` | 發票功能強化 |
| `pdf-engine-migration` | PDF 引擎遷移 |
| `fix-login-bug` | 登入問題修復 |
| `user-profile-redesign` | 使用者個人資料重新設計 |
| `ABC-123` | (Keep issue tracker IDs as-is) |

**Guidelines:**
- Expand abbreviations (e.g., `pdf` → `PDF`, `auth` → `認證`)
- Convert kebab-case to natural Traditional Chinese
- Preserve issue tracker IDs (e.g., `ABC-123`, `PROJ-456`) without translation
- Use Taiwanese terminology (e.g., use 「使用者」 not 「用户」, 「資料」 not 「数据」)

#### 2b. Summarize Commits into a Description

Synthesize all commit messages from the `commits` array into a **concise 1-2 sentence summary in Traditional Chinese (Taiwan)** that captures the work accomplished.

**Input:** Array of commit messages
```json
{
  "commits": [
    {"message": "feat(pdf): enhance invoice layout and improve styling"},
    {"message": "feat(pdf): switch to LXGW WenKai font for better CJK coverage"},
    {"message": "feat(invoices): add invoice deletion for admin users"}
  ]
}
```

**Output:** Single summarized description in Traditional Chinese
```
優化發票 PDF 版面與樣式、改用 LXGW WenKai 字型以支援中日韓字元、新增管理員刪除發票功能
```

**Summarization Guidelines:**

1. **Consolidate related changes**: Group similar commits (e.g., multiple styling commits → 「改善樣式」)
2. **Remove redundant prefixes**: Strip conventional commit prefixes like `feat:`, `fix:`, `chore:`
3. **Preserve technical specifics**: Keep important details like library names, feature names
4. **Use natural Taiwanese phrasing**: Write as a native Taiwanese speaker would
5. **Prioritize user-facing changes**: Lead with features, then fixes, then internal changes
6. **Keep it brief**: Target 1-2 sentences, use 「、」 to separate items

**Taiwanese Terminology Examples:**

| English Term | Use (Taiwan) | Avoid (China) |
|--------------|--------------|---------------|
| User | 使用者 | 用户 |
| Data | 資料 | 数据 |
| File | 檔案 | 文件 |
| Video | 影片 | 视频 |
| Software | 軟體 | 软件 |
| Information | 資訊 | 信息 |
| Network | 網路 | 网络 |
| Program | 程式 | 程序 |

**Anti-patterns to avoid:**
- ❌ Listing every commit verbatim
- ❌ Including commit hashes or dates
- ❌ Generic summaries like 「各項改進」
- ❌ Using Simplified Chinese or mainland terminology

### Step 3: Save Translations (REQUIRED - DO NOT SKIP)

<CRITICAL>
**THIS STEP IS MANDATORY.** You MUST call `mcp__time-tracker__create_work_item` for EVERY work item before outputting the report. Do NOT skip this step. Do NOT output the report until all work items have been saved.
</CRITICAL>

**For each work item**, call `mcp__time-tracker__create_work_item` with:
- `identifier`: The work item identifier (e.g., "invoice-enhancements")
- `project`: The project name or path (e.g., "yourclinic")
- `title`: The translated Chinese title (e.g., "發票功能強化")
- `description`: The summarized Chinese description

**Example - you MUST make these calls:**
```
// For EACH work item, call MCP:
mcp__time-tracker__create_work_item({
  identifier: "invoice-enhancements",
  project: "yourclinic",
  title: "發票功能強化",
  description: "優化發票 PDF 版面與樣式、改用 LXGW WenKai 字型支援中日韓字元、新增管理員刪除發票功能"
})

mcp__time-tracker__create_work_item({
  identifier: "pdf-prawn-migration",
  project: "yourclinic",
  title: "PDF 引擎遷移",
  description: "將發票 PDF 產生器從 ferrum_pdf 遷移至 Prawn 引擎"
})

// ... repeat for ALL work items
```

**Checklist before proceeding to Step 4:**
- [ ] Called `create_work_item` for work item 1
- [ ] Called `create_work_item` for work item 2
- [ ] ... (all work items)
- [ ] All MCP calls returned successfully

**Note:** `create_work_item` is idempotent - safe to call multiple times.

**If MCP call fails:** Report the error to the user and ask them to run `/mcp` to reconnect, then retry.

---

### Step 4: Output Formatted Report

Generate the final report using this structure (all labels in Traditional Chinese):

```markdown
# 工作時間報告

**期間：** YYYY 年 M 月
**總時數：** Xh Ym

---

## [專案名稱]

**小計：** Xh Ym

| 工作項 | 完成日期 | 時間 | 說明 |
|--------|----------|------|------|
| [翻譯後的標題] | YYYY-MM-DD | Xh Ym | [彙整後的中文摘要] |
```

### Complete Transformation Example

**Raw JSON Input:**
```json
{
  "id": "invoice-enhancements",
  "branch": "feature/invoice-enhancements",
  "total_seconds": 18000,
  "completed_date": "2026-01-19",
  "commits": [
    {"message": "feat(pdf): enhance invoice layout and improve styling"},
    {"message": "feat(pdf): switch to LXGW WenKai font for better CJK coverage"},
    {"message": "feat(invoices): add invoice deletion for admin users"}
  ]
}
```

**Transformed Output:**

| 工作項 | 完成日期 | 時間 | 說明 |
|--------|----------|------|------|
| 發票功能強化 | 2026-01-19 | 5h 0m | 優化發票 PDF 版面與樣式、改用 LXGW WenKai 字型以支援中日韓字元、新增管理員刪除發票功能 |

---

## Google Sheets Format (TSV)

When the user requests a format suitable for pasting into Google Sheets, output TSV (Tab-Separated Values):

```bash
claude-time-tracker report --format tsv [other options]
```

Apply the same translation and summarization transformations, preserving the TSV format.

**Example TSV Output:**
```
工作項	完成日期	時間	說明
發票功能強化	2026-01-19	5h 0m	優化發票 PDF 版面與樣式、改用 LXGW WenKai 字型以支援中日韓字元、新增管理員刪除發票功能
PDF 引擎遷移	2026-01-12	2h 37m	將發票 PDF 產生器從 ferrum_pdf 遷移至 Prawn 引擎
```

---

## CLI Commands

### Generate Reports

```bash
# Current month, markdown to stdout
claude-time-tracker report

# Specific month
claude-time-tracker report --month 2025-01

# Filter by project
claude-time-tracker report --project "Client-A"

# All formats to files
claude-time-tracker report --all-formats --output ~/reports/january

# Single format to file
claude-time-tracker report --format csv --output ~/reports/time.csv

# TSV format for Google Sheets (copy-paste ready)
claude-time-tracker report --format tsv
```

### Check Status

```bash
# Show active tracking sessions
claude-time-tracker status
```

### Manage Projects

```bash
# List all tracked projects
claude-time-tracker projects list

# Set friendly display name
claude-time-tracker projects set-name /path/to/project "Client A - E-commerce"
```

### Configuration

```bash
# Initialize global config
claude-time-tracker config init

# Edit config in $EDITOR
claude-time-tracker config edit

# Show current config
claude-time-tracker config show
```

---

## Project Setup

To enable time tracking for a new project, create a config file at `<project>/.claude-time-tracker.toml`:

```toml
name = "Client A - Project Name"

# Optional: Extract work item ID from branch names
# Examples for common issue trackers:
# Linear: "^(?:feature|fix|chore)/([A-Z]+-\\d+)"
# Jira:   "^(?:feature|fix)/([A-Z]+-\\d+)"
# GitHub: "#(\\d+)"
work_item_pattern = "^(?:feature|fix|chore)/([A-Z]+-\\d+)"

[report]
include_commits = true
max_commits_per_item = 10
```

**Common `work_item_pattern` examples:**

| Tracker | Pattern | Matches |
|---------|---------|---------|
| Linear/Jira | `^(?:feature\|fix\|chore)/([A-Z]+-\d+)` | `feature/ABC-123-description` |
| GitHub Issues | `#(\d+)` | `fix-#42-bug` |
| Trello | `([a-zA-Z0-9]{8})` | Short card IDs |

---

## Global Configuration

Location: `~/.config/claude-time-tracker/config.toml`

```toml
[settings]
idle_timeout_minutes = 10
database_path = "~/.local/share/claude-time-tracker/data.db"

# Optional: Turso cloud sync (embedded replica)
[settings.turso]
url = "libsql://your-db.turso.io"
# Use environment variable TURSO_AUTH_TOKEN for auth_token

[report]
default_format = "markdown"
include_commits = true
max_commits_per_item = 5
```

---

## Raw Report Output (Without Transformation)

For reference, the raw CLI output (before transformation) groups time by project and work item:

```markdown
# Time Tracking Report: 2025-01

## Client A - E-commerce
**Total: 12h 30m**

### ABC-123
- Time: 4h 15m
- Branch: feature/ABC-123-checkout-flow
- Commits:
  - Add shopping cart component
  - Implement checkout validation
```

The transformation workflow (Step 2) converts this into the polished, summarized format shown above.
