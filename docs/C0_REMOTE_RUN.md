# C0 遠程執行手冊（借 CI runner 避開沙箱 2GB RAM 限制）

> 背景：C0 基建（`charon.pin`/`scripts/c0_spike.py`/`scripts/c0_env_check.sh`）已齊，
> 但沙箱 2GB RAM 構建唔起 Charon（實測硬下限 ≈4GB，見 `docs/C0_CHARON_SPIKE.md`）。
> 手冊：**你推一次 workflow 上 GitHub，之後全部自動，結果兩條通道回流沙箱。**

## 運作原理

- GitHub `ubuntu-24.04` runner：4 vCPU / **16GB RAM** —— 綽綽有餘。
- Runner 係 Debian glibc 2.39，沙箱係 glibc **2.41**；舊 glibc 編嘅 binary 喺新 glibc 上可跑 ✓。
- 編譯先肥（charon_lib 峰值 ~1.25GB RSS），**runtime 好瘦**（小輸入每個幾百 MB 都唔使）——
  所以 binary 搬返沙箱後**喺沙箱跑 spike 都得**（Plan B）。
- CI 用 `secrets.GITHUB_TOKEN` 自己發佈；**沙箱唔使任何憑證**。

## Step 1（你手動，唯一一步）：推 workflow 上 GitHub

沙箱冇憑證推唔到；喺你本機執行（擇一）：

```bash
# A) 直接用成個倉（如果本機有 clone 同本分支同步）
cd polyrust && git fetch <沙箱來源> dev/v0.3-hardening && git checkout <branch>
git push <your-remote> HEAD
# B) 套用補丁檔（最小動作）
cd polyrust && git am < /path/to/polyrust_v0.3_hardening.patch && git push <your-remote> <branch>
```

分支名建議 `dev/v0.3-hardening`（照舊）或專開 `c0-charon-spike`——
workflow 對 `c0-charon*` push 會自動跑，否則下一步手動觸發。

## Step 2：觸發

Actions 頁 → **c0-charon-spike** → Run workflow（選分支）→ 等 ~15–25 分鐘。
（Push 到 `c0-charon*` 分支則免觸發，自動跑。）

## Step 3：結果兩條通道回流沙箱

**通道 ① Release assets（public repo 最簡單）** — 沙箱匿名落載：

```bash
cd /home/user
curl -sSfL "https://github.com/djrww/polyrust/releases/download/c0-charon-spike/c0-spike-results.tar.gz" -o c0-spike-results.tar.gz
curl -sSfL "https://github.com/djrww/polyrust/releases/download/c0-charon-spike/charon-bin-x86_64-ubuntu24.04.tar.gz" -o charon-bin.tar.gz   # 可選（Plan B 用）
```

**通道 ② `c0-results` 分支 commit（public repo 適用、免解 tar）**：

```bash
cd /home/user/polyrust
git fetch https://github.com/djrww/polyrust c0-results
git show FETCH_HEAD:c0/spike_out/summary.md   # 直接睇結果
```

> Private repo：通道 ①② 都要憑證 → 改由 browser 喺 Actions 頁 download
> **Artifacts: c0-charon-spike** zip，放入 workspace 任意路徑話我知，我接手解包+commit。

## Step 4（我接手）：解包 + commit + 判定轉綠

我會：對照 C0 驗收線（≥70/100 出 LLBC）寫入 `docs/C0_CHARON_SPIKE.md` 結果表、
fixtures commit 入 `charon_fixtures/`、C1 parser 即刻對接。

## Plan B（結果唔使你郁）：沙箱跑 spike

如果想 fixtures 由沙箱本地產生：解壓 `charon-bin.tar.gz` 到 `/home/user/charon-bin/`，
執行（runtime 記憶體需求低，沙箱 OK）：

```bash
export RUSTUP_TOOLCHAIN=nightly-2026-09-17   # 已喺沙箱裝好（如失效：rustup toolchain install）
export LD_LIBRARY_PATH="$HOME/.rustup/toolchains/nightly-2026-09-17-x86_64-unknown-linux-gnu/lib:$LD_LIBRARY_PATH"
python3 scripts/c0_spike.py --charon /home/user/charon-bin/charon \
  --out /home/user/spike_out --fixtures /home/user/polyrust/charon_fixtures
```

> 二進制同 pinned toolchain 都係 `nightly-2026-09-17`，rustc_driver dynlib 匹配。
> 若报 `librustc_driver-*.so` not found → 檢查 LD_LIBRARY_PATH 行。
