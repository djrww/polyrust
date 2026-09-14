#!/bin/sh
# 完整建置管線：Lean 4 形式化 → 靜態庫 .a → 嵌入 polyrust 二進位檔
#
#   1. lake build Polyrust:static        # 把 lean/Polyrust 14 個模組編成 .a
#   2. cargo build --release             # build.rs 偵測到 .a 後靜態連結嵌入
#
# 產物：target/release/polyrust（.a 已「編譯嵌入」，ldd 僅依賴 glibc）。
set -euo pipefail

cd "$(dirname "$0")/.."

# 定位 elan / lake（優先 ELAN_HOME，其次 ~/.elan，再來 PATH）
if [ -z "${ELAN_HOME:-}" ] && [ -d "$HOME/.elan/bin" ]; then
  export PATH="$HOME/.elan/bin:$PATH"
fi

command -v lake >/dev/null 2>&1 || { echo "❌ 找不到 lake（Lean 工具鏈未安裝）；請先跑 scripts/setup-lean.sh"; exit 1; }

echo "== [1/2] lake build Polyrust:static =="
(cd lean && lake build Polyrust:static)

A=$(find lean/.lake/build/lib -name '*.a' | head -1)
test -s "$A" || { echo "❌ 未產生靜態庫"; exit 1; }
echo "   靜態庫：$A = $(stat -c %s "$A") bytes（$(ar t "$A" | wc -l) 個物件）"

echo "== [2/2] cargo build --release（內嵌 .a）=="
cargo build --release

echo
echo "== 驗證 =="
ldd target/release/polyrust | grep -iE 'lean|gmp|uv|ssl|crypto|libc\+\+' \
  && { echo "❌ 仍有 Lean 動態依賴"; exit 1; } \
  || echo "✓ 無 Lean/gmp/uv/ssl/crypto/libc++ 動態依賴（僅 glibc）"
./target/release/polyrust demo 2>&1 | sed -n '4p'   # 印內嵌狀態列
ls -l target/release/polyrust | awk '{print "✓ 二進位：" $5 " bytes"}'
