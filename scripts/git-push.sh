#!/bin/sh
# SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
# 持久 push：PAT 存喺 repo 外嘅 workspace 檔 /home/user/.github_push_token
# （.git/config 唔持久——快照排除；本 script + token 檔持久）。
# 用法：scripts/git-push.sh <branch> [branch2 ...]
set -eu
REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_DIR"
TOKEN="$(cat /home/user/.github_push_token)"
for BR in "$@"; do
  git push "https://djrww:${TOKEN}@github.com/djrww/polyrust.git" "${BR}:${BR}"
done
