#!/usr/bin/env bash
# 產生 Lean 側與 Rust 側的實測證據（可重現）
#
#   bash scripts/lean-evidence.sh
#
# 產出：
#   lean/Audit.out            # 逐定理 #print axioms（零 sorry / 零自訂公理）
#   docs/evidence/env.txt     # 工具鏈版本與時間戳
#   docs/evidence/tests.raw.txt        # cargo test --release（單元測試）
#   docs/evidence/obligations.raw.txt  # 九條定理的義務自證（12 樣本）
#   docs/evidence/demo.raw.txt         # 四個 demo 的完整管線報告
#
# 註：Rust 側 obligations 需約 7 分鐘（全量 Buchberger 統計）。
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="${HOME}/.elan/bin:${HOME}/.cargo/bin:${PATH}"
mkdir -p docs/evidence
# 用法：bash scripts/lean-evidence.sh [lean-only]
#   lean-only：只跑 Lean 側（數秒）；完整重跑（含 obligations，約 7 分鐘）不加參數。
LEAN_ONLY="${1:-}"

echo "== Lean 側 =="
(cd lean && lake build)
(cd lean && lake env lean Audit.lean) | tee lean/Audit.out >/dev/null
echo "Audit: $(grep -c 'depends on axioms\|does not depend' lean/Audit.out) 條定理受檢；sorryAx：$(grep -c sorryAx lean/Audit.out || true)"

{
  echo "產生時間（UTC）：$(date -u +%FT%TZ)"
  echo "lean  ：$(lean --version 2>/dev/null || echo '未安裝')"
  echo "lake  ：$(lake --version 2>/dev/null || echo '未安裝')"
  echo "rustc ：$(rustc --version 2>/dev/null || echo '未安裝')"
  echo "cargo ：$(cargo --version 2>/dev/null || echo '未安裝')"
} > docs/evidence/env.txt

if [ "${LEAN_ONLY}" = "lean-only" ]; then
  echo "（lean-only：跳過 Rust 側實測；完整重跑請執行 bash scripts/lean-evidence.sh）"
  exit 0
fi

echo "== Rust 側 =="
cargo build --release
./target/release/polyrust obligations > docs/evidence/obligations.raw.txt 2>&1 || true
./target/release/polyrust demo        > docs/evidence/demo.raw.txt 2>&1 || true
cargo test --release                  > docs/evidence/tests.raw.txt 2>&1 || true
echo "完成：docs/evidence/ 已更新"
