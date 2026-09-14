#!/bin/bash
# 用法: bigreq-one.sh <slug> <model> <prompt>
cd /home/user/polyrust
KEY="${OPENROUTER_API_KEY:?請設定環境變數 OPENROUTER_API_KEY}"
BASE="https://openrouter.ai/api/v1"
slug="$1"; model="$2"; prompt="$3"
mkdir -p output/bigreq
echo "START $slug $(date +%H:%M:%S)" 
timeout 1200 ./target/release/polyrust nl "$prompt" \
  --provider openrouter --base-url "$BASE" --model "$model" --api-key "$KEY" \
  --attempts 5 --json > "output/bigreq/$slug.json" 2>"output/bigreq/$slug.err"
echo "EXIT=$?"
echo "DONE $slug $(date +%H:%M:%S)"
