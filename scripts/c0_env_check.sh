#!/usr/bin/env bash
# SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
# c0_env_check.sh — Charon C0 Spike 一鍵環境檢查 + 執行（CHARON_POLYIR_BLUEPRINT C0）
#
# 用途：任何 ≥4GB RAM、有網絡嘅機器一條命令完成 C0 全量執行：
#   bash scripts/c0_env_check.sh
#
# 步驟：檢查 RAM → clone/pin charon → install pinned nightly → build（2GB 機自動加
# opt-level=1 降峰值記憶體覆寫）→ 執行 c0_spike.py 全量案例 → 輸出驗收判定。

set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo "$PWD")"

CHARON_COMMIT=$(grep '^charon_commit:' charon.pin | awk '{print $2}')
TOOLCHAIN=$(grep '^rust_toolchain:' charon.pin | awk '{print $2}')
WORK=${C0_WORKDIR:-/tmp/polyrust_charon_work}

echo "== C0 環境檢查 =="
echo "charon pin: $CHARON_COMMIT  toolchain: $TOOLCHAIN"

# 1) RAM 檢查（Charon 構建峰值記憶體見 Docs: charon_lib ~1.25GB @cgu32/opt1, deps 高並發會超 2GB）
MEM_MB=$(free -m 2>/dev/null | awk '/^Mem:/{print $2}' || echo 9999)
if [ "$MEM_MB" -lt 3900 ]; then
  echo "⚠  RAM ${MEM_MB}MB < 3900MB —— 套用低記憶體覆寫（-j2 + charon opt-level=1），仍可能失敗；建議 ≥4GB"
  JOBS=2
  LOWMEM=1
else
  JOBS=$(nproc 2>/dev/null || echo 4)
  LOWMEM=0
fi

# 2) clone + pin
if [ ! -d "$WORK/charon-src" ]; then
  git clone --depth 1 https://github.com/AeneasVerif/charon "$WORK/charon-src"
fi
cd "$WORK/charon-src"
git fetch --depth 2 origin "$CHARON_COMMIT" 2>/dev/null || true
git checkout -q "$CHARON_COMMIT" 2>/dev/null || echo "(淺 clone 已為目標 commit 或 checkout 略過)"

# 3) pinned toolchain
rustup toolchain install "$TOOLCHAIN" --profile minimal \
  --component rust-src --component rustc-dev --component llvm-tools
export RUSTUP_TOOLCHAIN="$TOOLCHAIN"

# 4) 收窄 toolchain targets（加速），可選低記憶體覆寫
printf '[toolchain]\nchannel = "%s"\ncomponents = ["rustc-dev","llvm-tools","rust-src"]\ntargets = ["%s"]\n' \
  "$TOOLCHAIN" "$(rustc -vV | sed -n 's/^host: //p')" > rust-toolchain
cp rust-toolchain charon/rust-toolchain
if [ "$LOWMEM" = "1" ] && ! grep -q 'opt-level = 1' charon/Cargo.toml; then
  printf '\n[profile.release.package.charon]\nopt-level = 1\ncodegen-units = 32\n' >> charon/Cargo.toml
fi

# 5) build（CARGO_TARGET_DIR 避免名單內目錄名，快照場景可續跑）
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$WORK/cargo_out}
(cd charon && cargo build --release --locked -j"$JOBS")

CHARON_BIN="$CARGO_TARGET_DIR/release/charon"
[ -x "$CHARON_BIN" ] || { echo "ERROR: charon binary 未產出"; exit 1; }

# 6) 全量 spike
cd -
python3 scripts/c0_spike.py --charon "$CHARON_BIN" \
  --out "$WORK/spike_out" --fixtures charon_fixtures
