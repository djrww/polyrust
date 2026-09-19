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
# 相容三種安裝位置：使用者 HOME、/opt（見 scripts/setup-lean.sh）、CI setup-action
for d in "${HOME}/.elan/bin" /opt/elan/bin "${HOME}/.local/bin"; do
  [ -d "$d" ] && PATH="$d:${PATH}"
done
export PATH
command -v lake >/dev/null 2>&1 || { echo "找不到 lake；請先執行 scripts/setup-lean.sh 或安裝 elan" >&2; exit 127; }
cd "$(dirname "$0")/../lean"

echo "== 1/4 來源掃描：sorry / admit =="
hits="$(grep -rn --include='*.lean' -E '\b(sorry|admit)\b' Polyrust Polyrust.lean 2>/dev/null || true)"
if [ -n "$hits" ]; then
  echo "以下位置出現 sorry/admit（含註解；若有註解外的出現即為缺口）："
  echo "$hits"
else
  echo "0 個 sorry/admit"
fi

echo
echo "== 2/4 建置 =="
lake build

echo
echo "== 3/4 公理審計（#print axioms）=="
lake env lean Audit.lean

echo
echo "== 4/4 全環境公理審計（涵蓋每一條宣告，不只手列清單）=="
lake env lean AuditAll.lean | tee AuditAll.out
grep -qx 'AUDIT_RESULT=CLEAN' AuditAll.out \
  || { echo "❌ 全環境公理審計未通過（見上）" >&2; exit 1; }

echo
echo "== 摘要 =="
# 注意：grep -c 在「零符合」時回傳 exit 1，配 set -e/pipefail 會讓腳本誤報失敗，
# 故一律 `|| true` 兜住（計數本身仍由 grep -c 正確輸出）。
n_constructive="$(lake env lean Audit.lean 2>&1 | grep -c 'does not depend on any axioms' || true)"
n_sorry="$(lake env lean Audit.lean 2>&1 | grep -c 'sorryAx' || true)"
echo "無公理依賴（純構造性）的定理數：${n_constructive}"
echo "含 sorryAx 的定理數（必須為 0）：${n_sorry}"
grep -E '受檢宣告數|純構造性' AuditAll.out || true
[ "${n_sorry}" = "0" ] || { echo "❌ 發現 sorryAx" >&2; exit 1; }
echo "✅ 審計通過"
