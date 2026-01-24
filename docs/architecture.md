# Claude Time Tracker 架構文檔

## 概述

Claude Time Tracker 是一個用於追蹤 Claude Code 使用時間的工具。它透過 Claude Code 的 statusline 機制來可靠地記錄工作時間，並支援本地 SQLite 或 Turso 雲端資料庫。

## 系統架構

```
┌─────────────────────────────────────────────────────────────┐
│                    Statusline (每秒)                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 1. record_local_heartbeat()                         │    │
│  │    - 每 60 秒寫入本地快取                           │    │
│  │    - 每 5 分鐘觸發背景 sync                         │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 2. get_tracker_status()                             │    │
│  │    - 每 10 秒才呼叫 status（使用快取）              │    │
│  │    - 顯示 ⏱ 小時鐘                                  │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              ~/.cache/claude-time-tracker/                  │
│  - heartbeats.jsonl (本地 heartbeat 快取)                   │
│  - status_cache (status 輸出快取)                           │
│  - last_heartbeat (上次 heartbeat 時間戳)                   │
│  - last_sync (上次同步時間戳)                               │
└─────────────────────────────────────────────────────────────┘
                           ↓ (sync 命令)
┌─────────────────────────────────────────────────────────────┐
│              資料庫 (SQLite / Turso)                        │
│  - 本地：~/.local/share/claude-time-tracker/data.db        │
│  - 遠端：Turso (可選，需設定 config.toml)                   │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              Kilok (讀取報告資料)                           │
└─────────────────────────────────────────────────────────────┘
```

## 設計決策

### 為什麼不使用 Claude Code Hooks？

原本的設計使用 Claude Code 的 `UserPromptSubmit` hook 來記錄 heartbeat，但實測發現：

| 問題 | 數據 |
|------|------|
| Hook 觸發率 | 僅 25%（4 次 SessionStart 只有 1 次 heartbeat） |
| Turso 延遲 | 每次 hook 執行需 1-2 秒 |
| 本地 SQLite | 僅需 8ms |

**結論**：`UserPromptSubmit` hook 不可靠，改用 statusline 作為可靠的追蹤點。

### Statusline-First 設計

**核心原則**：Statusline 是 Claude Code 唯一 100% 可靠的追蹤點（每秒呼叫）。

**Hooks 的角色**：
- `SessionStart` / `Stop` hooks 仍然存在，但只是「nice to have」
- 即使 hooks 沒觸發（如 session restore），tracking 依然正常運作
- Sync 命令會自動建立缺少的 session

**優化策略**：
- Heartbeat 記錄：每 60 秒寫入本地快取（避免頻繁寫入）
- 資料庫同步：每 5 分鐘背景執行（批次處理）
- Session 自動建立：sync 偵測到 heartbeat 但無 active session 時自動建立
- Status 查詢：每 10 秒快取（減少資料庫壓力）

## 檔案結構

### 本地快取 (`~/.cache/claude-time-tracker/`)

| 檔案 | 用途 |
|------|------|
| `heartbeats.jsonl` | Heartbeat 記錄（JSONL 格式） |
| `status_cache` | Status 輸出快取 |
| `last_heartbeat` | 上次 heartbeat 的 Unix 時間戳 |
| `last_sync` | 上次同步的 Unix 時間戳 |

### Heartbeat 格式

```json
{"timestamp":"2026-01-24T00:12:34Z","project_path":"/path/to/project","unix_ts":1234567890}
```

## Claude Code 整合

### Hooks 設定 (`~/.claude/settings.json`)

```json
{
  "hooks": {
    "SessionStart": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "/Users/take5/bin/claude-time-tracker start --path $PWD"
      }]
    }],
    "Stop": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "/Users/take5/bin/claude-time-tracker stop --path $PWD"
      }]
    }]
  }
}
```

**注意**：必須使用完整路徑（`/Users/take5/bin/`），不能使用 `~/bin/`，因為在 Claude Code 環境中 tilde expansion 可能失敗。

### Statusline 設定 (`~/.claude/settings.json`)

```json
{
  "statusline": {
    "type": "script",
    "script": "/Users/take5/.claude/statusline-wrapper.sh"
  }
}
```

## 時間計算邏輯

### Active Time 計算

```
active_time = Σ(interval) where interval <= idle_timeout
```

只有連續 heartbeat 之間的間隔小於 `idle_timeout_minutes`（預設 5 分鐘）才會被計入活躍時間。

### Session 生命週期

**主要流程（statusline-first）**：
```
Statusline → 每 60 秒記錄 heartbeat（本地快取）
     ↓
Sync 命令 → 每 5 分鐘同步到資料庫
     ↓         ↓（如果沒有 active session）
     ↓         └→ 自動建立 session（用第一個 heartbeat 時間戳）
     ↓
Stop hook → 計算 active_time + 關閉 session（可選）
```

**輔助流程（hooks，可選）**：
```
SessionStart hook → 建立 session（如果 statusline 還沒建立）
Stop hook → 正常關閉 session
```

### 棄置 Session 處理

如果 Claude Code 異常退出（沒有觸發 Stop hook）：
1. 下次 `start` 時會自動關閉超過 idle_timeout 的棄置 session
2. 或者下次 `sync` 時會將 heartbeats 記錄到現有 session

## 資料庫設定

### 純本地模式

預設使用本地 SQLite，無需額外設定。

### Turso 雲端模式

建立 `~/.config/claude-time-tracker/config.toml`：

```toml
[settings]
idle_timeout_minutes = 5

[settings.turso]
url = "libsql://your-db.turso.io"
# auth_token 可從 TURSO_AUTH_TOKEN 環境變數讀取
```

## CLI 命令

| 命令 | 用途 |
|------|------|
| `start --path <PATH>` | 開始追蹤（SessionStart hook 呼叫） |
| `heartbeat --path <PATH>` | 記錄活動（已棄用，改用 statusline） |
| `stop --path <PATH>` | 停止追蹤（Stop hook 呼叫） |
| `sync` | 同步本地快取到資料庫（statusline 呼叫） |
| `status` | 顯示目前追蹤狀態 |
| `report` | 產生時間報告 |

## 效能數據

| 項目 | 數值 |
|------|------|
| 本地 SQLite 查詢 | ~8ms |
| Turso 查詢 | ~1-2 秒 |
| Heartbeat 寫入（本地快取） | <1ms |
| Sync 批次處理 | 每 5 分鐘一次 |
| Status 快取 TTL | 10 秒 |

## Heartbeat 頻率與資料量分析

### Heartbeat 產生頻率

| 項目 | 數值 | 說明 |
|------|------|------|
| 本地寫入間隔 | 60 秒 | 寫到 JSONL 檔，< 1ms |
| DB 同步間隔 | 5 分鐘 | 批次處理所有 heartbeats |
| 每小時 heartbeats | ~60 筆 | 每 60 秒 1 筆 |
| 每日 (8hr) | ~480 筆 | 一個 project |
| 每月 (22 天) | ~10,560 筆 | 一個 project |

### 資料庫壓力分析

**寫入頻率**：
```
本地快取寫入：每 60 秒 1 次（append to JSONL）→ 幾乎無感
DB 同步：每 5 分鐘 1 次 batch insert → 約 5 筆 heartbeats/次
```

**實際 DB trips**：
- 每小時只有 **12 次** DB 寫入（60 min / 5 min interval）
- 每次寫入 ~5 筆 heartbeats（batch）
- Turso 或本地 SQLite 都能輕鬆處理

### 資料量估算

| 時間範圍 | 單專案資料量 | 10 專案資料量 |
|----------|-------------|--------------|
| 每月 | ~500 KB | ~5 MB |
| 每年 | ~6 MB | ~60 MB |

**結論**：資料量極小，SQLite/Turso 都能輕鬆處理數百萬筆 heartbeats。

### 為什麼不會有效能問題

1. **寫入批次化**：不是每秒寫 DB，而是本地快取 → 5 分鐘批次
2. **讀取快取**：status 有 10 秒 TTL，statusline 每秒呼叫但實際 DB 查詢很少
3. **資料量小**：heartbeat 只有時間戳，不是大型 payload

### 調整建議

如果想進一步減少 DB 壓力，可在 `statusline-wrapper.sh` 中調整 `TRACKER_HB_INTERVAL`：

| 間隔 | 每日 heartbeats | 時間精度 |
|------|-----------------|---------|
| 60 秒（預設）| 480 | 誤差 ±1 分鐘 |
| 120 秒 | 240 | 誤差 ±2 分鐘 |
| 300 秒 | 96 | 誤差 ±5 分鐘 |

對於計費用途，60 秒間隔提供足夠精度，同時保持極低的系統負擔。
