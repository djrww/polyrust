#!/bin/bash
# 大需求落地測試：逐條跑，逐條落盤（防超時丟失）
cd /home/user/polyrust
KEY="${OPENROUTER_API_KEY:?請設定環境變數 OPENROUTER_API_KEY}"
BASE="https://openrouter.ai/api/v1"
OUT="/home/user/polyrust/output/bigreq"
mkdir -p "$OUT"

run_one () {
  local slug="$1"; local model="$2"; local prompt="$3"
  echo "### START $slug ($(date +%H:%M:%S)) model=$model" >> "$OUT/progress.log"
  timeout 900 ./target/release/polyrust nl "$prompt" \
    --provider openrouter --base-url "$BASE" --model "$model" --api-key "$KEY" \
    --attempts 4 --json > "$OUT/$slug.json" 2>"$OUT/$slug.err"
  echo "### DONE  $slug exit=$? ($(date +%H:%M:%S))" >> "$OUT/progress.log"
}

run_one "gui"  "nex-agi/nex-n2.5-pro:free"            "用rust語言開發一個反應式渲染GUI開發平台"
run_one "appstore" "inclusionai/ling-3.0-flash-sante:free" "用rust語言編寫一個新app發佈平台"
run_one "video" "inclusionai/ling-3.0-flash-fin:free"  "用rust語言編寫可錄製短影音既程式"
echo "ALL_DONE" >> "$OUT/progress.log"
