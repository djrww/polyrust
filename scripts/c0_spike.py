#!/usr/bin/env python3
# SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
# c0_spike.py — Charon C0 Spike harness（CHARON_POLYIR_BLUEPRINT.md C0）
#
# 將語義矩陣 100 案例 + examples/*.poly + examples/phase3/*.poly
# 脫水（去掉 polyrust DSL `# ...` 行）成純 Rust，逐一餵 Charon，記錄：
#   ok / ok_with_missing / rustc_reject（地真值 UNSAT：型別錯誤，含 E0614 類）/ charon_err
# 輸出 spike_out/results.json + summary.md；並挑 6 個代表作 llbc fixture。
#
# 用法：python3 scripts/c0_spike.py --charon /path/to/charon --out spike_out
#
import argparse, json, os, re, shutil, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def sanitize(src: str) -> str:
    """去掉 polyrust DSL 註釋行（`# ...`）——rustc 視 `#` 為屬性開頭，原樣餵入必 syntax error。"""
    lines = [ln for ln in src.splitlines() if not ln.lstrip().startswith("#")]
    return "\n".join(lines) + "\n"


def corpus():
    items = []  # (name, category, sanitized_rs)
    sm = open(os.path.join(ROOT, "core/src/semantic_matrix.rs"), encoding="utf-8").read()
    for name, body, cat in re.findall(
        r'SemanticCase::new\("(\w+)",\s*"((?:[^"\\]|\\.)*)",\s*(?:true|false),\s*vec!\[[^\]]*\],\s*"(\w+)"\)',
        sm,
    ):
        items.append((name, f"matrix/{cat}", sanitize(body.encode().decode("unicode_escape"))))
    for sub in ("examples", "examples/phase3"):
        d = os.path.join(ROOT, sub)
        if not os.path.isdir(d):
            continue
        for f in sorted(os.listdir(d)):
            if f.endswith(".poly"):
                src = open(os.path.join(d, f), encoding="utf-8").read()
                items.append((f"{sub.split('/')[-1]}/{f[:-5]}", "examples", sanitize(src)))
    return items


def llbc_stats(path: str):
    try:
        data = open(path, encoding="utf-8", errors="replace").read()
        doc = json.loads(data)
    except Exception as e:
        return {"parse": f"llbc-not-json: {e}", "bytes": os.path.getsize(path)}
    missing = data.count('"Missing"') + len(re.findall(r'"error"\s*:', data))
    if doc.get("has_errors"):  # Charon 自帶旗號：有 decl 抽取失敗 → 唔算全綠，至少升 ok_with_missing
        missing = max(missing, 1)
    top = {}
    for k, v in doc.items():
        if isinstance(v, list):
            top[k] = len(v)
        elif isinstance(v, dict):
            for k2, v2 in v.items():
                if isinstance(v2, list):
                    top[f"{k}.{k2}"] = len(v2)
    return {"bytes": os.path.getsize(path), "missing_marks": missing, "top_lists": top}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--charon", required=True)
    ap.add_argument("--out", default="spike_out")
    ap.add_argument("--fixtures", default=None, help="複製代表 llbc 到呢個目錄")
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)

    results = []
    for name, cat, rs in corpus():
        if args.limit and len(results) >= args.limit:
            break
        safe = re.sub(r"[^\w/-]", "_", name).replace("/", "__")
        with tempfile.TemporaryDirectory() as td:
            inp = os.path.join(td, "input.rs")
            open(inp, "w", encoding="utf-8").write(rs)
            try:
                # 新 CLI（ca501af6+）：`charon rustc -- <rustc args>`，檔名亦屬 rustc 參數。
                # 舊用法 `charon --crate-type=rlib file.rs` 會被 clap 拒絕（C0 首跑法證：152/152
                # 全錯分為 rustc_reject）。
                # --edition=2021 必需：rustc 單檔編譯預設 edition 2015，async/await 直接 E0670。
                p = subprocess.run(
                    [args.charon, "rustc", "--", inp, "--crate-type=rlib", "--edition=2021"],
                    cwd=td, capture_output=True, text=True, timeout=120,
                )
            except subprocess.TimeoutExpired:
                results.append({"name": name, "cat": cat, "status": "timeout"})
                continue
            err = p.stderr or ""
            llbcs = [f for f in os.listdir(td) if f.endswith(".llbc") or f.endswith(".ullbc")]
            if p.returncode != 0:
                # rustc 真拒絕必帶[E-code]；clap/CLI 錯（"error: unexpected argument"等）一律歸 charon_err，
                # 唔會再誤分。rustc_reject 口徑 = UNSAT(E-code)。
                cls = "rustc_reject" if re.search(r"error\[E\d+\]", err) else "charon_err"
                rec = {"name": name, "cat": cat, "status": cls,
                       "first_err": (err.strip().splitlines() or ["?"])[0][:160]}
                results.append(rec)
                continue
            rec = {"name": name, "cat": cat, "status": "ok"}
            if llbcs:
                dst = os.path.join(args.out, safe + ".llbc")
                shutil.copy(os.path.join(td, llbcs[0]), dst)
                st = llbc_stats(dst)
                rec.update(st)
                if st.get("missing_marks", 0) > 0:
                    rec["status"] = "ok_with_missing"
            results.append(rec)

    os.path.exists(args.out) or os.makedirs(args.out)
    with open(os.path.join(args.out, "results.json"), "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=1)

    # 聚合
    from collections import Counter
    agg = Counter(r["status"] for r in results)
    bycat = {}
    for r in results:
        bycat.setdefault(r["cat"], Counter())[r["status"]] += 1
    n_ok = agg["ok"] + agg["ok_with_missing"]
    # 設計上 UNSAT（rustc-invalid）案例一口徑：明細 status==rustc_reject 的佔位實質係正確判定，
    # 但 DSL 語料（缺 use 行、`?` 非 Result 返回等）屬語料-vs-Rust 誠實邊界，唔係 Charon 能力缺口。
    # 設計上 UNSAT（phase3/*_unsat、examples/bad）：rustc 拒絕係正確判定，唔應拖低指標。
    # 其餘 reject（*_sat/*_unknown 入 rustc_reject、matrix 缺 derive/外部 crate 等）
    # 屬語料-vs-真 Rust 嘅誠實邊界 → C1 語料修剪清單，唔係 Charon 能力缺口。
    n_rej = agg["rustc_reject"]
    n_design_unsat = sum(
        1 for r in results
        if r["status"] == "rustc_reject" and (r["name"].endswith("_unsat") or r["name"] == "examples/bad")
    )
    denom_eff = max(len(results) - n_design_unsat, 1)
    eff = len(results)
    lines = ["# C0 Charon Spike — 執行結果", "",
             f"## 有效轉換率：**{n_ok}/{denom_eff} = {100*n_ok/denom_eff:.1f}%**（已排除設計 UNSAT {n_design_unsat} 例）", "",
             f"總案例 {len(results)}；出 LLBC **{n_ok}**（{100*n_ok/eff:.1f}%）；"
             f"ok {agg['ok']} ｜ ok_with_missing {agg['ok_with_missing']} ｜ "
             f"rustc_reject {agg['rustc_reject']} ｜ charon_err {agg['charon_err']} ｜ timeout {agg['timeout']}", "",
             "## 按類別", "", "| 類別 | ok | ok+missing | rustc_reject | charon_err | timeout |", "|---|---|---|---|---|---|"]
    for c in sorted(bycat):
        cc = bycat[c]
        lines.append(f"| {c} | {cc['ok']} | {cc['ok_with_missing']} | {cc['rustc_reject']} | {cc['charon_err']} | {cc['timeout']} |")
    lines += ["", "## 明細（非 ok）", ""]
    for r in results:
        if r["status"] not in ("ok",):
            lines.append(f"- `{r['name']}` [{r['cat']}] → **{r['status']}**  {r.get('first_err','')}")
    with open(os.path.join(args.out, "summary.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")

    # fixtures：每類挑 1 個 ok 代表
    if args.fixtures:
        os.makedirs(args.fixtures, exist_ok=True)
        picked = {}
        for r in results:
            if r["status"].startswith("ok") and r["cat"] not in picked:
                picked[r["cat"]] = r
        for r in list(picked.values())[:6]:
            safe = re.sub(r"[^\w/-]", "_", r["name"]).replace("/", "__")
            src = os.path.join(args.out, safe + ".llbc")
            if os.path.exists(src):
                shutil.copy(src, os.path.join(args.fixtures, safe + ".llbc"))

    print("\n".join(lines[:8]))
    ok_threshold = 70 if len(results) >= 100 else 0.7 * len(results)
    print(f"\n== C0 驗收：出 LLBC {n_ok}/{len(results)}（門檻 ≥{int(ok_threshold)}）→ " +
          ("PASS" if n_ok >= ok_threshold else "FAIL"))


if __name__ == "__main__":
    sys.exit(main())
