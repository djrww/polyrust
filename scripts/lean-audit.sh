#!/usr/bin/env bash
# Lean 4 形式化的可信度審計：
#   1. 建置（零 sorry / 零 admit 的來源掃描）
#   2. 逐定理 `#print axioms`
#
# 判讀方式：
#   * 出現 `sorryAx`      → 該定理的證明有洞（本倉庫不允許）
#   * 出現 `axiom ...`    → 引入了額外公理（本倉庫不允許）
#   * 只出現 propext / Classical.choice / Quot.sound → Lean 標準三公理
#   * 「does not depend on any axioms」→ 純構造性證明
set -euo pipefail
export PATH="${HOME}/.elan/bin:${PATH}"
cd "$(dirname "$0")/../lean"

echo "== 1/3 來源掃描：sorry / admit =="
hits="$(grep -rn --include='*.lean' -E '\b(sorry|admit)\b' Polyrust Polyrust.lean 2>/dev/null || true)"
if [ -n "$hits" ]; then
  echo "以下位置出現 sorry/admit（含註解；若有註解外的出現即為缺口）："
  echo "$hits"
else
  echo "0 個 sorry/admit"
fi

echo
echo "== 2/3 建置 =="
lake build

echo
echo "== 3/3 公理審計（#print axioms）=="
lake env lean Audit.lean

echo
echo "== 摘要 =="
lake env lean Audit.lean 2>&1 | grep -c "does not depend on any axioms" | xargs -I{} echo "無公理依賴（純構造性）的定理數：{}"
lake env lean Audit.lean 2>&1 | grep -c "sorryAx" | xargs -I{} echo "含 sorryAx 的定理數（必須為 0）：{}"
