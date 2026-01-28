# Time Report Web Interface 設計文件

## 概述

建立一個 Web 介面，讓業主可以查看每月的維護時數報告。

## 目標

- 業主可透過公開連結查看自己專案的報告
- 你也能產出報告連結分享給業主
- MVP 功能簡潔，之後可擴展

## 技術架構

```
┌─────────────────────────────────────────────────────────┐
│                                                          │
│   ┌─────────────┐                         ┌──────────┐  │
│   │ Rust CLI    │   automatic sync        │ SvelteKit│  │
│   │ (libSQL     │◀───────────────────────▶│ (Web UI) │  │
│   │  embedded   │         Turso           │          │  │
│   │  replica)   │         Cloud           │          │  │
│   └─────────────┘                         └──────────┘  │
│         │                                       │       │
│         ▼                                       ▼       │
│   ┌─────────────┐                         ┌──────────┐  │
│   │ local .db   │ ◀─────sync─────────────▶│  Turso   │  │
│   │ file        │                         │  remote  │  │
│   └─────────────┘                         └──────────┘  │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 元件

1. **Rust CLI（現有）**：改用 `libsql` crate，支援 embedded replica
2. **Turso Cloud**：儲存同步後的資料
3. **SvelteKit Web**：SSR 讀取 Turso，呈現報告
4. **Cloudflare Pages**：部署 SvelteKit

## 資料庫 Schema

### 現有表格（保留）

```sql
-- projects, sessions, heartbeats, commits（維持不變）
```

### 新增表格

```sql
-- 客戶/業主
CREATE TABLE clients (
    id INTEGER PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,        -- URL 用：'metis', 'yourclinic'
    name TEXT NOT NULL,               -- 顯示名稱：'Metis 投資顧問'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 合約時數（年度合約）
CREATE TABLE contracts (
    id INTEGER PRIMARY KEY,
    client_id INTEGER NOT NULL REFERENCES clients(id),
    year INTEGER NOT NULL,            -- 2025, 2026
    total_hours REAL NOT NULL,        -- 合約總時數：144
    carried_over REAL DEFAULT 0,      -- 上年度結轉：8
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(client_id, year)
);

-- 擴展 projects 表
ALTER TABLE projects ADD COLUMN client_id INTEGER REFERENCES clients(id);
```

### 資料關係

```
Client (Metis)
  ├── Contract (2025: 144h, carry: 8h)
  ├── Contract (2026: 120h)
  └── Project (HK site)
      └── Session → Commits
```

## URL 結構

| URL | 用途 |
|-----|------|
| `/` | 首頁（客戶列表，你自己用） |
| `/:clientSlug` | 客戶年度總覽（給業主看） |
| `/:clientSlug/:year-:month` | 單月詳細報告 |

## 頁面設計

### 1. 首頁 `/`

顯示所有客戶卡片，點擊進入客戶總覽。

### 2. 客戶總覽 `/:clientSlug`

**頂部統計（Bento Grid）：**
- 總時數（含結轉）
- 已使用時數
- 剩餘時數

**月份列表：**
- 顯示各月份時數
- 點擊進入月報詳情

### 3. 月報詳情 `/:clientSlug/:year-:month`

**頂部摘要：**
- 本月時數
- 累積時數

**工作明細表格：**
| 日期 | 時數 | 工作項 | 說明 |
|------|------|--------|------|

## 專案結構

```
time-report-web/
├── src/
│   ├── routes/
│   │   ├── +page.svelte              # 首頁
│   │   ├── +layout.svelte            # 共用佈局
│   │   └── [client]/
│   │       ├── +page.svelte          # 客戶總覽
│   │       ├── +page.server.ts       # 資料載入
│   │       └── [period]/
│   │           ├── +page.svelte      # 月報詳情
│   │           └── +page.server.ts
│   ├── lib/
│   │   ├── server/
│   │   │   └── db.ts                 # Turso 連線
│   │   └── components/
│   │       ├── StatCard.svelte
│   │       └── WorkTable.svelte
│   └── app.css
├── package.json
├── svelte.config.js
├── tailwind.config.js
└── wrangler.toml                     # Cloudflare 設定
```

## Tech Stack

| 層級 | 技術 |
|------|------|
| 框架 | SvelteKit |
| 樣式 | TailwindCSS |
| 資料庫 | Turso (libSQL) |
| 部署 | Cloudflare Pages |
| Rust CLI | libsql crate（embedded replica） |

## MVP 功能範圍

### 包含

- [x] 客戶列表頁
- [x] 客戶年度總覽（總時數、使用、剩餘）
- [x] 月份列表
- [x] 月報明細表格（日期、時數、工作項、說明）
- [x] 公開連結存取（無認證）

### 之後再加

- [ ] 圖表視覺化（長條圖、圓餅圖）
- [ ] 篩選/搜尋功能
- [ ] 網站欄位
- [ ] 匯出 PDF
- [ ] 認證機制

## Rust CLI 修改

需要將現有的 `rusqlite` 改為 `libsql`，並設定 embedded replica：

```rust
use libsql::Builder;

let db = Builder::new_remote_replica(
    local_path,                        // 本地 replica 路徑
    "libsql://your-db.turso.io",       // Turso URL
    auth_token,
)
.build()
.await?;

// 定期或每次寫入後同步
db.sync().await?;
```

## 實作順序

1. **Turso 設定**：建立 database、取得 credentials
2. **Schema migration**：新增 clients, contracts 表
3. **SvelteKit 專案**：初始化、設定 Turso 連線
4. **頁面開發**：首頁 → 客戶總覽 → 月報詳情
5. **部署**：Cloudflare Pages
6. **Rust CLI 改造**：遷移到 libsql embedded replica
