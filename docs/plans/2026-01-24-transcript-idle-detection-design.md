# Transcript-Based Idle Detection 設計文件

## 概述

本設計移除對 Claude Code hooks 的依賴，改用 transcript 檔案的修改時間來判斷專案是否處於 idle 狀態。

## 問題背景

### 原有架構的問題

1. **Hooks 不可靠**：`UserPromptSubmit` 觸發率僅 25%
2. **資料庫鎖定**：多個 Claude Code instance 同時觸發 Stop hook → SQLite "database is locked"
3. **複雜度高**：需要維護 hooks 和 statusline 兩套機制

## 新架構

### Statusline-Only 設計

```
┌─────────────────────────────────────────────────────────────┐
│  Statusline 每秒呼叫                                         │
│                                                             │
│  heartbeat 命令（每 60 秒）：                                │
│    1. 檢查 transcript mtime → idle? 跳過                    │
│    2. 寫入本地快取 (~/.cache/claude-time-tracker/)           │
│                                                             │
│  sync 命令（每 5 分鐘）：                                    │
│    1. 檢查 transcript mtime → 自動關閉 idle session         │
│    2. 同步 heartbeats 到資料庫                              │
│    3. 自動建立 session（如果需要）                           │
└─────────────────────────────────────────────────────────────┘
```

### Idle 檢測原理

Claude Code 的 transcript 檔案位於：
```
~/.claude/projects/-<project-path-with-dashes>/<uuid>.jsonl
```

當 Claude 有活動時（用戶輸入、Claude 回應），transcript 檔案會持續更新。
如果 transcript 的 mtime 超過 idle_timeout（預設 5 分鐘），則視為 idle。

### 關鍵優勢

1. **不依賴 hooks**：完全移除 SessionStart/Stop hooks
2. **無資料庫鎖定**：heartbeat 只寫本地快取，sync 背景執行
3. **自動 session 管理**：自動建立和關閉 session，無需手動操作

## 實作細節

### 新增模組：`src/idle.rs`

```rust
pub fn is_project_idle(project_path: &Path, idle_timeout_minutes: u32) -> bool
```

- 找到對應的 Claude transcript 目錄
- 取得最近修改的 .jsonl 檔案的 mtime
- 比較 mtime 與 idle_timeout

### 修改：`heartbeat` 命令

- 不再連接資料庫
- 加入 idle 檢測（如果 idle 則跳過）
- 只寫入本地快取

### 修改：`sync` 命令

- 新增 `--path` 參數
- 自動關閉 idle session
- 同步 heartbeats 到資料庫

### 修改：statusline wrapper

- 呼叫 `heartbeat --path $cwd`（含 idle 檢測）
- 呼叫 `sync --path $cwd`（含 idle session 關閉）

## 移除的項目

- `~/.claude/settings.json` 中的 SessionStart hook
- `~/.claude/settings.json` 中的 Stop hook
- `tracker::record_heartbeat()` 函數（改用本地快取）

## 效能

| 操作 | 耗時 |
|------|------|
| transcript mtime 檢查 | < 3ms |
| heartbeat 寫入（本地快取） | < 1ms |
| sync 背景執行 | 不阻塞 statusline |

## 測試驗證

1. 正常工作時，heartbeats 持續記錄
2. 離開電腦 5 分鐘後，heartbeats 停止記錄
3. Session 在 transcript 超時後自動關閉
4. 無 "database is locked" 錯誤
