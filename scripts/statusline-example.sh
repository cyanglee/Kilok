#!/bin/bash

# Kilok Statusline Script
# 顯示：目錄 [branch] ⏱ 追蹤時間 💎 ruby ⬢ node [model] [context bar] 時間

# 從 Claude Code 讀取 JSON 輸入
input=$(cat)

# Debug: 啟用以查看 JSON 欄位
# echo "$input" > /tmp/claude-statusline-debug.json

# ===== Time Tracker - 本機心跳 =====
TRACKER_CACHE_DIR="$HOME/.cache/claude-time-tracker"
TRACKER_LAST_HB_FILE="$TRACKER_CACHE_DIR/last_heartbeat"
TRACKER_LAST_SYNC_FILE="$TRACKER_CACHE_DIR/last_sync"
TRACKER_HB_INTERVAL=60      # 每 60 秒記錄一次心跳
TRACKER_SYNC_INTERVAL=300   # 每 5 分鐘同步到 Turso

record_local_heartbeat() {
    local project_path="$1"
    local now=$(date +%s)

    # 確保快取目錄存在
    mkdir -p "$TRACKER_CACHE_DIR"

    # 節流：檢查是否該記錄心跳
    local last_hb=0
    if [ -f "$TRACKER_LAST_HB_FILE" ]; then
        last_hb=$(cat "$TRACKER_LAST_HB_FILE" 2>/dev/null || echo 0)
    fi

    if [ $((now - last_hb)) -ge $TRACKER_HB_INTERVAL ]; then
        # 呼叫 heartbeat 命令（包含基於 transcript mtime 的閒置偵測）
        # 在背景執行以避免阻塞 statusline
        ($HOME/.local/bin/claude-time-tracker heartbeat --path "$project_path" 2>/dev/null &)
        echo "$now" > "$TRACKER_LAST_HB_FILE"
    fi

    # 檢查是否該同步到資料庫（背景執行）
    local last_sync=0
    if [ -f "$TRACKER_LAST_SYNC_FILE" ]; then
        last_sync=$(cat "$TRACKER_LAST_SYNC_FILE" 2>/dev/null || echo 0)
    fi

    if [ $((now - last_sync)) -ge $TRACKER_SYNC_INTERVAL ]; then
        # 在背景同步，帶入路徑以偵測閒置 session
        ($HOME/.local/bin/claude-time-tracker sync --path "$project_path" 2>/dev/null &)
        echo "$now" > "$TRACKER_LAST_SYNC_FILE"
    fi
}

# ===== 從 JSON 提取資料 =====
cwd=$(echo "$input" | jq -r '.cwd // .workspace.current_dir // "."')

# 記錄本機心跳（節流、非阻塞）
if [ -n "$cwd" ] && [ "$cwd" != "." ]; then
    record_local_heartbeat "$cwd"
fi

# Model 資訊
model=$(echo "$input" | jq -r '.model.name // .model.display_name // .model.id // "unknown"')

# Context window 資訊
CONTEXT_SIZE=$(echo "$input" | jq -r '.context_window.context_window_size // 200000')

# 處理除以零的情況
if [ "$CONTEXT_SIZE" -eq 0 ] 2>/dev/null; then
    CONTEXT_SIZE=200000
fi

# v2.0.70+: 使用 current_usage 計算精確的 context 使用量
CURRENT_USAGE=$(echo "$input" | jq -r '.context_window.current_usage // null')

if [ "$CURRENT_USAGE" != "null" ]; then
    CURRENT_INPUT=$(echo "$CURRENT_USAGE" | jq -r '.input_tokens // 0')
    CACHE_CREATION=$(echo "$CURRENT_USAGE" | jq -r '.cache_creation_input_tokens // 0')
    CACHE_READ=$(echo "$CURRENT_USAGE" | jq -r '.cache_read_input_tokens // 0')
    CURRENT_OUTPUT=$(echo "$CURRENT_USAGE" | jq -r '.output_tokens // 0')
    TOTAL_TOKENS=$((CURRENT_INPUT + CACHE_CREATION + CACHE_READ + CURRENT_OUTPUT))
else
    INPUT_TOKENS=$(echo "$input" | jq -r '.context_window.total_input_tokens // 0')
    OUTPUT_TOKENS=$(echo "$input" | jq -r '.context_window.total_output_tokens // 0')
    TOTAL_TOKENS=$((INPUT_TOKENS + OUTPUT_TOKENS))
fi

PERCENT_USED=$((TOTAL_TOKENS * 100 / CONTEXT_SIZE))
if [ "$PERCENT_USED" -gt 100 ]; then
    PERCENT_USED=100
fi

# 產生進度條（10 字元寬）
BAR_WIDTH=10
FILLED=$((PERCENT_USED * BAR_WIDTH / 100))
EMPTY=$((BAR_WIDTH - FILLED))

BAR_FILLED=""
BAR_EMPTY=""
for ((i=0; i<FILLED; i++)); do BAR_FILLED+="█"; done
for ((i=0; i<EMPTY; i++)); do BAR_EMPTY+="░"; done

# 格式化 token 數量
format_tokens() {
    local tokens=$1
    if [ "$tokens" -ge 1000 ]; then
        echo "$((tokens / 1000))k"
    else
        echo "$tokens"
    fi
}
TOKENS_DISPLAY="$(format_tokens $TOTAL_TOKENS)/$(format_tokens $CONTEXT_SIZE)"

# 取得目前目錄名稱
dir=$(basename "$cwd")

# 取得 git 資訊
git_branch=""
git_status=""
if git -C "$cwd" rev-parse --git-dir > /dev/null 2>&1; then
    git_branch=$(git -C "$cwd" branch --show-current 2>/dev/null)

    if ! git -C "$cwd" diff --quiet 2>/dev/null || ! git -C "$cwd" diff --cached --quiet 2>/dev/null; then
        git_status="*"
    fi
    if [ -n "$(git -C "$cwd" ls-files --others --exclude-standard 2>/dev/null | head -1)" ]; then
        git_status="*"
    fi

    upstream=$(git -C "$cwd" rev-parse --abbrev-ref '@{upstream}' 2>/dev/null)
    if [ -n "$upstream" ]; then
        ahead=$(git -C "$cwd" rev-list --count '@{upstream}..HEAD' 2>/dev/null || echo 0)
        behind=$(git -C "$cwd" rev-list --count 'HEAD..@{upstream}' 2>/dev/null || echo 0)

        if [ "$ahead" -gt 0 ] && [ "$behind" -gt 0 ]; then
            git_status="${git_status} ↕${ahead}/${behind}"
        elif [ "$ahead" -gt 0 ]; then
            git_status="${git_status} ↑${ahead}"
        elif [ "$behind" -gt 0 ]; then
            git_status="${git_status} ↓${behind}"
        fi
    fi
fi

# 取得目前專案的 time tracker 狀態（從心跳計算的實際活躍時間）
tracker_status=""

if [ -n "$cwd" ] && [ "$cwd" != "." ]; then
    # 使用 CLI 取得實際活躍時間（考慮閒置超時）
    active_seconds=$($HOME/.local/bin/claude-time-tracker active-time --path "$cwd" 2>/dev/null)

    if [ -n "$active_seconds" ] && [ "$active_seconds" -gt 0 ] 2>/dev/null; then
        # 格式化經過時間
        hours=$((active_seconds / 3600))
        mins=$(((active_seconds % 3600) / 60))

        if [ "$hours" -gt 0 ]; then
            tracker_status="⏱ ${hours}h${mins}m"
        else
            tracker_status="⏱ ${mins}m"
        fi
    fi
fi

# 從 .tool-versions 取得 Ruby 版本
ruby_version=""
if [ -f "$cwd/.tool-versions" ]; then
    ruby_version=$(grep "^ruby " "$cwd/.tool-versions" 2>/dev/null | awk '{print $2}' | tr -d '\n')
fi

# 從 .tool-versions 取得 Node 版本
node_version=""
if [ -f "$cwd/.tool-versions" ]; then
    node_version=$(grep "^nodejs " "$cwd/.tool-versions" 2>/dev/null | awk '{print $2}' | tr -d '\n')
fi

# 目前時間
time=$(date +%H:%M:%S)

# ===== 建構輸出 =====
output=""

# 目錄（藍色）
output="\033[34m${dir}\033[0m"

# Git branch 含狀態（黃色，有變更時紅色）
if [ -n "$git_branch" ]; then
    if [[ "$git_status" == *"*"* ]]; then
        dirty_indicator="\033[31m*\033[33m"
        git_status_display="${git_status/\*/$dirty_indicator}"
    else
        git_status_display="$git_status"
    fi
    git_status_display=$(echo "$git_status_display" | sed 's/↓/\\033[31m↓/g' | sed 's/↑/\\033[32m↑/g' | sed 's/↕/\\033[35m↕/g')
    output="$output \033[33m[${git_branch}${git_status_display}\033[33m]\033[0m"
fi

# Time tracker 狀態（青色）- 只在追蹤目前專案時顯示
if [ -n "$tracker_status" ]; then
    output="$output \033[36m${tracker_status}\033[0m"
fi

# Ruby 版本（紅色）
if [ -n "$ruby_version" ]; then
    output="$output \033[31m💎 ${ruby_version}\033[0m"
fi

# Node 版本（綠色）
if [ -n "$node_version" ]; then
    output="$output \033[32m⬢ ${node_version}\033[0m"
fi

# Model 名稱（亮青色）
model_short=$(echo "$model" | sed 's/^Claude //' | sed 's/^claude-//')
output="$output \033[96m[${model_short}]\033[0m"

# Context 使用量：進度條含顏色
if [ "$PERCENT_USED" -gt 80 ] 2>/dev/null; then
    BAR_COLOR="\033[31m"
elif [ "$PERCENT_USED" -gt 50 ] 2>/dev/null; then
    BAR_COLOR="\033[33m"
else
    BAR_COLOR="\033[32m"
fi
output="$output ${BAR_COLOR}[${BAR_FILLED}\033[2m${BAR_EMPTY}\033[0m${BAR_COLOR}]\033[0m \033[2m${TOKENS_DISPLAY}\033[0m"

# 時間（淡化）
output="$output \033[2m${time}\033[0m"

# 輸出
printf "%b" "$output"
