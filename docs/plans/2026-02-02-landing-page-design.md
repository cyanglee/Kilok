# Kilok Landing Page Design

**Date:** 2026-02-02
**Status:** Approved

## Overview

為 Kilok webapp 建立一個混合型 landing page，結合產品介紹與登入入口，面向想使用 Kilok 的 Claude Code 使用者。

### 設計方向

- **風格**：友善現代風 + 台灣在地元素
- **色調**：沿用現有 theme（藍色 primary #2563eb）
- **特色**：「記錄 kì-lo̍k」的命名故事

## 路由架構

| 原路由 | 新路由 | 說明 |
|--------|--------|------|
| `/` | `/dashboard` | 登入後儀表板（需認證） |
| - | `/` | 新 Landing Page（公開） |

### 認證流程

```
訪客進入 / → Landing Page → 點擊「登入」→ /admin/login → 登入成功 → /dashboard
```

## 頁面區塊

### 1. Floating Navbar

- 半透明毛玻璃效果 (`bg-white/80 backdrop-blur`)
- Logo 左側，GitHub + 登入按鈕右側
- `mx-4 mt-4 rounded-xl` 浮動設計

### 2. Hero 區

- 大標題：「為 Claude Code 打造的時間追蹤工具」
- 副標：「零侵入、自動記錄、精準計費」
- CTA：GitHub 開始使用（主）、查看 Demo（次）
- Statusline 動態示意（CSS animation 模擬計時器）

### 3. 功能特色（4 張卡片）

| 功能 | 說明 |
|------|------|
| 零侵入追蹤 | 透過 statusline hook，無需手動操作 |
| 智慧閒置偵測 | 監控 transcript 檔案，精準判斷活躍狀態 |
| 計費報表生成 | 自動計算 1.2x，支援手動覆寫 |
| 雲端同步儀表板 | Turso 資料庫 + Web 介面 |

### 4. Webapp 截圖展示

- Mock Dashboard 使用假資料渲染
- 虛構公司：Acme Corp、Startup Inc
- 展示客戶列表、進度條、統計卡片

### 5. 名稱故事

> 「Kilok」取自台語「記錄」（kì-lo̍k）的諧音，同時也是英文 key log 的組合。
> 為每一次與 AI 的協作，留下時間的足跡。

- 置中引言風格
- 淺色背景區隔

### 6. Footer

- 快速連結：GitHub、登入 Admin、Demo 報告
- Copyright：© 2026 · Made with Claude Code

## 檔案結構

```
src/
├── routes/
│   ├── +page.svelte              # Landing Page
│   ├── +layout.svelte            # 條件式 layout
│   └── dashboard/
│       ├── +page.svelte          # 原儀表板（搬移）
│       └── +page.server.ts       # 原 server load
└── lib/
    └── components/
        └── landing/
            ├── Navbar.svelte
            ├── Hero.svelte
            ├── Features.svelte
            ├── MockDashboard.svelte
            ├── NameStory.svelte
            └── Footer.svelte
```

## 技術細節

### Statusline 動畫

```svelte
<script>
  let minutes = $state(34);
  $effect(() => {
    const interval = setInterval(() => {
      minutes = (minutes + 1) % 60;
    }, 3000);
    return () => clearInterval(interval);
  });
</script>
```

### 登入 Redirect 調整

登入成功後改為 redirect 到 `/dashboard`（原本是 `/admin/clients`）。

## 實作順序

1. 建立 feature branch
2. 搬移現有儀表板到 `/dashboard`
3. 調整登入 redirect
4. 建立 landing page 元件
5. 組裝 landing page
6. 測試與調整
