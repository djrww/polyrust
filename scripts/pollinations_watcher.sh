#!/bin/bash
# Pollinations 匿名配額池恢復監視（本沙箱用）：
# 每 5 分鐘探一次；一旦恢復（回包非 budget 錯誤）即自動跑標準案例集＋出漏斗報告。
# 日誌：output/watcher.log。最多守 8 小時。
export PATH="$HOME/.cargo/bin:$HOME/.elan/bin:$PATH"
cd /home/user/polyrust || exit 1
mkdir -p output
LOG=output/watcher.log

probe() {
  printf '{"model":"openai-fast","messages":[{"role":"user","content":"reply OK"}],"max_tokens":10}' \
  | timeout 40 curl -sS -m 35 -X POST -H "Content-Type: application/json" \
      --data-binary @- https://text.pollinations.ai/v1/chat/completions 2>/dev/null \
  | python3 -c "
import json,sys
try:
    d=json.load(sys.stdin)
    c=(d.get('choices') or [{}])[0].get('message',{}).get('content','')
    print('BUDGET' if 'budget' in c else ('RECOVERED' if c.strip() else 'EMPTY'))
except Exception:
    print('ERR')"
}

for i in $(seq 1 96); do
  R=$(probe)
  if [ "$R" = "RECOVERED" ]; then
    echo "$(date '+%F %T') probe#$i 配額池恢復，開跑批測" >> "$LOG"
    python3 scripts/run_guardrail_suite.py \
      --funnel-log output/guardrail-funnel.ndjson >> "$LOG" 2>&1
    python3 scripts/funnel_report.py \
      --md output/guardrail-report.md >> "$LOG" 2>&1
    echo "$(date '+%F %T') 批測＋報告完成" >> "$LOG"
    exit 0
  fi
  echo "$(date '+%F %T') probe#$i -> $R" >> "$LOG"
  sleep 300
done
echo "$(date '+%F %T') 8 小時內未恢復，監視結束" >> "$LOG"
