# Claude Time Tracker MCP Server 設計文件

## 概述

將 claude-time-tracker 做成 MCP Server，讓 Claude Code 可以：
1. 讀取時間追蹤資料
2. 分析 commits 並產生工作項描述
3. 寫入描述到資料庫

前端 kilok 只負責呈現資料，不負責智慧分析。

## 架構

```
┌─────────────────────────────────────────────────────────────┐
│  Claude Code                                                │
│    ↓ MCP 協議                                               │
│  claude-time-tracker (MCP Server)                           │
│    - list_work_items                                        │
│    - get_work_item                                          │
│    - update_work_item                                       │
│    ↓                                                        │
│  Turso DB ←────────────────────────→ kilok (前端)           │
│                                       - 顯示報告             │
│                                       - 編輯時間調整         │
│                                       - 合併/拆分 work items │
└─────────────────────────────────────────────────────────────┘
```

## 分工原則

| 操作類型 | 負責者 | 原因 |
|----------|--------|------|
| 分析 commits → 產生描述 | Claude Code (MCP) | AI 擅長 |
| 調整時間、合併拆分 | kilok 前端 | UI 操作更直覺 |
| 顯示報告 | kilok 前端 | 視覺化呈現 |

## Schema 變更

### 新增 work_items 表

```sql
CREATE TABLE work_items (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    identifier TEXT NOT NULL,              -- "ABC-123" (從 branch 提取)
    title TEXT,                            -- Claude 產生的標題
    description TEXT,                      -- Claude 產生的完整描述
    time_adjustment_seconds INTEGER DEFAULT 0,  -- 手動時間調整（正負秒數）
    completed_date TEXT,                   -- 完成日期
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT,
    UNIQUE(project_id, identifier)
);

CREATE INDEX idx_work_items_project_id ON work_items(project_id);
```

### 修改 sessions 表

```sql
ALTER TABLE sessions ADD COLUMN work_item_id INTEGER REFERENCES work_items(id);
CREATE INDEX idx_sessions_work_item_id ON sessions(work_item_id);
```

### 時間計算

```sql
-- 顯示時間 = 計算值 + 調整值
SELECT
  w.*,
  (SELECT COALESCE(SUM(s.active_seconds), 0)
   FROM sessions s
   WHERE s.work_item_id = w.id) + w.time_adjustment_seconds AS total_seconds
FROM work_items w;
```

## 資料關係

```
Project (linpong)
└── Work Item (ABC-123)
    ├── title: "發票功能強化"
    ├── description: "優化發票版面樣式、改用 LXGW WenKai..."
    ├── time_adjustment_seconds: 0
    ├── total_seconds: 29700 (計算值)
    └── Sessions
        ├── Session 1 (7200s) → Commits [c1, c2, c3]
        └── Session 2 (22500s) → Commits [c4, c5]
```

## MCP 工具設計

### list_work_items

列出指定時間範圍的工作項目。

```typescript
{
  name: "list_work_items",
  description: "列出工作項目",
  parameters: {
    project?: string,       // 專案名稱或路徑
    month?: string,         // "2026-01" 格式
    include_commits?: bool  // 是否包含 commit 列表
  }
}
```

**回傳範例**：
```json
[
  {
    "id": 1,
    "identifier": "ABC-123",
    "title": "發票功能強化",
    "description": "優化發票版面樣式...",
    "total_seconds": 29700,
    "time_adjustment_seconds": 0,
    "completed_date": "2026-01-19",
    "commits": [
      "feat(pdf): enhance invoice layout",
      "feat(pdf): switch to LXGW WenKai font"
    ]
  }
]
```

### get_work_item

取得單一工作項目的完整資訊。

```typescript
{
  name: "get_work_item",
  description: "取得工作項目完整資訊，包含所有 sessions 和 commits",
  parameters: {
    work_item_id?: number,
    identifier?: string,    // "ABC-123"
    project?: string
  }
}
```

**回傳範例**：
```json
{
  "id": 1,
  "identifier": "ABC-123",
  "title": "發票功能強化",
  "description": null,
  "total_seconds": 29700,
  "sessions": [
    {
      "id": 10,
      "started_at": "2026-01-18T10:00:00Z",
      "active_seconds": 7200,
      "commits": [
        {"hash": "abc123", "message": "feat(pdf): enhance invoice layout"},
        {"hash": "def456", "message": "feat(pdf): switch to LXGW WenKai font"}
      ]
    }
  ]
}
```

### update_work_item

更新工作項目的標題、描述或時間調整。

```typescript
{
  name: "update_work_item",
  description: "更新工作項目",
  parameters: {
    work_item_id: number,
    title?: string,
    description?: string,
    time_adjustment_seconds?: number,  // 正負秒數調整
    completed_date?: string
  }
}
```

## 使用流程

### Claude Code 產生報告

```
用戶：「幫我整理這個月的工作報告」
          ↓
Claude Code：呼叫 list_work_items(month: "2026-01", include_commits: true)
          ↓
MCP Server 回傳 work items 列表（含 commits）
          ↓
Claude Code：發現某些 work items 沒有 title/description
          ↓
Claude Code：分析 commits，產生標題和描述
          ↓
Claude Code：呼叫 update_work_item(work_item_id: 1, title: "...", description: "...")
          ↓
MCP Server：寫入資料庫
          ↓
Claude Code：輸出完整報告給用戶
```

### 調整時間

```
用戶：「ABC-123 那個工作項實際花了多 30 分鐘，幫我調整」
          ↓
Claude Code：呼叫 update_work_item(work_item_id: 1, time_adjustment_seconds: 1800)
          ↓
MCP Server：更新 time_adjustment_seconds
          ↓
下次查詢時，total_seconds = 計算值 + 1800
```

## 實作順序

1. **Schema migration**：新增 work_items 表，修改 sessions 表
2. **MCP Server**：實作三個工具
3. **CLI 整合**：現有 CLI 也能使用新的 work_items 功能
4. **kilok 前端**：讀取 work_items 顯示報告

## 與 kilok 的整合

kilok 前端直接讀取 Turso DB：

```typescript
// kilok 查詢
const workItems = await db.execute(`
  SELECT
    w.*,
    (SELECT COALESCE(SUM(s.active_seconds), 0)
     FROM sessions s
     WHERE s.work_item_id = w.id) + w.time_adjustment_seconds AS total_seconds
  FROM work_items w
  JOIN projects p ON w.project_id = p.id
  WHERE p.client_id = ?
  ORDER BY w.completed_date DESC
`, [clientId]);
```

kilok 負責：
- 顯示報告
- 直接編輯 time_adjustment_seconds
- 合併/拆分 work items（操作 work_item_id 關聯）
