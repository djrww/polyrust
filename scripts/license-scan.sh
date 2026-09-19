#!/bin/sh
# SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
# license-scan.sh — 檢查所有軟件源碼檔案帶 SPDX 雙授權標頭（CI 硬閘門）
# 排除：生成物（output/）、基準輸出、第三方、目標目錄。
set -eu
cd "$(git rev-parse --show-toplevel)"

REQUIRED='SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)'
missing=0

for f in $(git ls-files '*.rs' '*.lean'); do
    case "$f" in
        */output/*|core/output/*|*/generated/*|target/*) continue ;;
    esac
    if ! head -15 "$f" | grep -q "$REQUIRED"; then
        echo "MISSING SPDX: $f"
        missing=$((missing + 1))
    fi
done

if [ "$missing" -gt 0 ]; then
    echo "license-scan: $missing 個檔案缺 SPDX 標頭（紅線三：文檔/授權口徑單一化）" >&2
    exit 1
fi
echo "license-scan: 全部源碼檔案帶 AGPL-3.0+Commercial 雙授權標頭 ✓"
