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

### Step 0: Detect Current Project (Auto)

**Unless the user explicitly specifies a project**, automatically detect the current Git project:

```bash
git rev-parse --show-toplevel
```

Use the output path as the `project` parameter for MCP calls. This ensures reports are scoped to the current repository.

**Examples:**
- User: "產生這個月報告" → Detect current project, filter by it
- User: "產生 yourclinic 這個月報告" → Use "yourclinic" as project filter
- User: "產生所有專案的報告" → Don't pass project filter (return all)

---

### Step 0.5: Verify Project Exists (If No Match)

If `list_sessions` returns empty results or the detected path doesn't match any known project:

1. **List available projects** to show the user:
   ```
   mcp__time-tracker__list_projects({})
   ```

2. **Ask user** which project to use with `AskUserQuestion`:
   ```
   找不到符合「{detected_path}」的專案。

   選項：
   1. 使用「{closest_match}」（最相似的專案）
   2. 為此路徑設定新的顯示名稱
   3. 顯示所有專案讓我選擇
   ```

3. **If user wants to set a display name**, call:
   ```
   mcp__time-tracker__update_project({
     project_id: <id>,
     display_name: "使用者提供的名稱"
   })
   ```

This ensures proper project identification and allows users to configure display names for better recognition in future reports.

---

### Step 1: Fetch Session Data via MCP

Use the `list_sessions` MCP tool to get all sessions (including main branch work):

```
mcp__time-tracker__list_sessions({
  project: "<detected-project-path>",  // from Step 0, or user-specified
  month: "2026-01",                    // optional, format: YYYY-MM
  include_commits: true                // default: true
})
```

**Response structure:**
```json
{
  "sessions": [
    {
      "id": 1,
      "project_id": 2,
      "project_name": "YourClinic 診所管理系統",
      "branch": "feature/ABC-123-login",
      "work_item": "ABC-123",
      "started_at": "2026-01-15T10:00:00Z",
      "active_seconds": 3600,
      "commits": [
        {"hash": "abc123", "message": "feat: add login form"}
      ]
    },
    {
      "id": 2,
      "project_id": 2,
      "project_name": "YourClinic 診所管理系統",
      "branch": "main",
      "work_item": null,
      "started_at": "2026-01-16T14:00:00Z",
      "active_seconds": 5400,
      "commits": [
        {"hash": "def456", "message": "fix: typo in readme"},
        {"hash": "ghi789", "message": "feat: add CSV export [Branch: feature/export]"}
      ]
    }
  ],
  "total_seconds": 9000
}
```

### Step 2: Group Commits into Work Items

**核心原則：每個獨立的工作都應該是一個獨立的工作項目。**

#### 2a. Sessions with `work_item` (feature branches)

直接使用 `work_item` 值作為工作項目 ID，例如 "ABC-123"。

#### 2b. Sessions without `work_item` (main/master branch)

對於 main/master branch 上的 commits，需要**逐一分析**：

1. **檢查 `[Branch: xxx]` 標籤**：若 commit message 包含此標籤，歸類到該 branch
2. **語意分析相關性**：
   - 同一個功能的多個 commits → 合併成一個工作項目
   - 不相關的 commits → **各自成為獨立的工作項目**
3. **時間分配**：Session 時間按 commit 數量比例分配

**判斷 commits 是否相關的標準：**
- 相同的 scope（如 `fix(coupon): ...` 和 `test(coupon): ...`）
- 明顯的因果關係（如 bug fix 後的 revert 再 fix）
- 同一個 issue 的不同部分

**不應該合併的情況：**
- 不同功能模組的修改
- 獨立的 bug fixes
- 完全不相關的 chores

**Example:**
```
輸入 commits:
  1. "fix(views): refine contact page text"
  2. "feat(security): improve rate limit config"
  3. "fix(coupon): 修復限定商品優惠券無法折扣的問題"
  4. "Revert fix(coupon)..."
  5. "fix(coupon): 修復限定商品優惠券條件邏輯錯誤"
  6. "test(order_price_calculator): 新增折扣計算邏輯的單元測試"

分組結果:
  - 工作項 1: "聯絡頁面文字優化" (commit 1)
  - 工作項 2: "安全性速率限制改善" (commit 2)
  - 工作項 3: "優惠券折扣邏輯修復" (commits 3, 4, 5 - 相關)
  - 工作項 4: "折扣計算測試" (commit 6)

時間分配 (假設 session 總共 6 小時):
  - 工作項 1: 1h (1/6)
  - 工作項 2: 1h (1/6)
  - 工作項 3: 3h (3/6)
  - 工作項 4: 1h (1/6)
```

### Step 3: Transform Work Items

For each grouped work item, perform two transformations:

#### 3a. Translate Work Item ID to Human-Readable Title

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

#### 3b. Summarize Commits into a Description

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

### Step 4: Save Translations (REQUIRED - DO NOT SKIP)

<CRITICAL>
**THIS STEP IS MANDATORY.** You MUST call `mcp__time-tracker__create_work_item` for EVERY work item before outputting the report. Do NOT skip this step. Do NOT output the report until all work items have been saved.
</CRITICAL>

**For each work item**, call `mcp__time-tracker__create_work_item` with:
- `identifier`: The work item identifier (e.g., "invoice-enhancements")
- `project`: The project name or path (e.g., "yourclinic")
- `title`: The translated Chinese title (e.g., "發票功能強化")
- `description`: The summarized Chinese description

**Example - you MUST make these calls:**

**範例 1：Feature branch 工作項目**
```
mcp__time-tracker__create_work_item({
  identifier: "invoice-enhancements",
  project: "yourclinic",
  title: "發票功能強化",
  description: "優化發票 PDF 版面與樣式、改用 LXGW WenKai 字型支援中日韓字元、新增管理員刪除發票功能"
})
```

**範例 2：Main branch 上的複合工作項目（含子項目）**

當 main branch 有多個 commits 時，建立一個複合工作項目，並在描述中列出**帶日期的子項目**：

```
// 建立 identifier="master" 或 identifier="main" 的複合工作項目
// identifier 必須與 session 的 branch 名稱相符，web app 才能正確顯示
mcp__time-tracker__create_work_item({
  identifier: "master",  // 或 "main"，取決於實際分支名稱
  project: "landtop",
  title: "一般維護與修復",
  description: "2026/01/15 | 頁面文字優化 | 改善聯絡頁面與錯誤頁面的文字內容\n2026/01/20 | 安全性速率限制 | 改進速率限制設定並新增品牌化錯誤頁面\n2026/01/25 | 價格表格修復 | 修復價格表格在手機版無法滑動的問題\n2026/01/28 | 優惠券折扣修復 | 修復限定商品優惠券無法正確套用折扣的問題"
})
```

**重要：** `identifier` 必須是 `master` 或 `main`（與 session 的 branch 欄位相符），這樣 web app 才能透過 `project_id:branch` 的方式找到對應的工作項目。

**描述格式規範（子項目帶日期和時間）：**
- 每行一個子項目
- 格式：`YYYY/MM/DD | 子項目標題 | 時間 | 簡短說明`
- 日期從 commit 的日期取得
- 時間按 commit 數量比例分配（總時間 / commit 數量）
- Web app 會解析這個格式並以清單方式顯示

**Checklist before proceeding to Step 5:**
- [ ] Called `create_work_item` for work item 1
- [ ] Called `create_work_item` for work item 2
- [ ] ... (all work items)
- [ ] All MCP calls returned successfully

**Note:** `create_work_item` is idempotent - safe to call multiple times.

**If MCP call fails:** Report the error to the user and ask them to run `/mcp` to reconnect, then retry.

---

### Step 5: Output Formatted Report

Generate the final report using this structure (all labels in Traditional Chinese):

```markdown
# 工作時間報告

**期間：** YYYY 年 M 月
**總時數：** Xh Ym

---

## [專案名稱]

**小計：** Xh Ym

### [工作項目標題]
- **日期：** YYYY-MM-DD
- **時間：** Xh Ym
- **說明：** [彙整後的中文摘要]

### [複合工作項目標題]（含子項目）
- **日期：** YYYY-MM-DD
- **時間：** Xh Ym（總計）
- **說明：** [彙整後的中文摘要]
- **子項目：**
  - 子工作 1 - 簡短說明
  - 子工作 2 - 簡短說明
  - 子工作 3 - 簡短說明
```

**報告格式規則：**
1. 每個工作項目都要顯示**日期**（完成日期或最後活動日期）
2. 如果工作項目是由多個不相關的 commits 組成（main branch 雜項），需列出**子項目清單**
3. Feature branch 的單一工作項目不需要子項目清單

### Complete Transformation Example

**範例 1：Feature Branch 單一工作項目**

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

### 發票功能強化
- **日期：** 2026-01-19
- **時間：** 5h 0m
- **說明：** 優化發票 PDF 版面與樣式、改用 LXGW WenKai 字型以支援中日韓字元、新增管理員刪除發票功能

---

**範例 2：Main Branch 複合工作項目（含子項目及日期）**

**Raw JSON Input:**
```json
{
  "branch": "master",
  "work_item": null,
  "total_seconds": 19860,
  "completed_date": "2026-01-28",
  "commits": [
    {"message": "fix(views): refine contact page text", "created_at": "2026-01-15T10:00:00Z"},
    {"message": "feat(security): improve rate limit config", "created_at": "2026-01-20T14:00:00Z"},
    {"message": "fix(mobile): 修復價格表格手機版無法滑動", "created_at": "2026-01-25T09:00:00Z"},
    {"message": "fix(coupon): 修復限定商品優惠券無法折扣的問題", "created_at": "2026-01-28T16:00:00Z"}
  ]
}
```

**Transformed Output（Markdown 報告）：**

### 一般維護與修復
- **總時間：** 5h 32m
- **子項目：**
  - 2026/01/15 頁面文字優化 (1h 23m) - 改善聯絡頁面文字清晰度
  - 2026/01/20 安全性速率限制 (1h 23m) - 改進速率限制設定
  - 2026/01/25 價格表格修復 (1h 23m) - 修復手機版無法滑動的問題
  - 2026/01/28 優惠券折扣修復 (1h 23m) - 修復限定商品優惠券無法正確折扣

**Web App 儲存格式（description 欄位）：**
```
2026/01/15 | 頁面文字優化 | 1h 23m | 改善聯絡頁面文字清晰度
2026/01/20 | 安全性速率限制 | 1h 23m | 改進速率限制設定
2026/01/25 | 價格表格修復 | 1h 23m | 修復手機版無法滑動的問題
2026/01/28 | 優惠券折扣修復 | 1h 23m | 修復限定商品優惠券無法正確折扣
```

**時間分配計算：**
- 取得 session 的 `active_seconds` 總時間
- 按 commit 數量等比例分配給每個子項目
- 例如：4 個 commits 共 5h 32m → 每個約 1h 23m

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
