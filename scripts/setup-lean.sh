#!/bin/sh
# Lean 4 環境安裝（沙盒/開發機通用；裝到 /opt/elan 不污染 HOME）
set -e
mkdir -p /opt/elan 2>/dev/null || sudo mkdir -p /opt/elan
sudo chown -R "$(id -u):$(id -g)" /opt/elan 2>/dev/null || true
export ELAN_HOME=/opt/elan
curl -sSf https://elan.lean-lang.org/elan-init.sh | sh -s -- -y --default-toolchain none
/opt/elan/bin/elan default stable
/opt/elan/bin/lean --version
echo "完成。之後用：export ELAN_HOME=/opt/lan PATH=/opt/elan/bin:\$PATH && cd lean && lake build"
