#!/usr/bin/env python3
# SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
# rustc_align_check.py — RSAP R2：corpus → rustc oracle vs PolyIR 對準聚合
#
# 用法（無 Charon 亦可，僅跑 rustc 量尺）：
#   python3 scripts/rustc_align_check.py [--rustc-only] [--json] [--limit N]
#
# 預設走 `polyrust align-check --json`（需先 cargo build）；--rustc-only 則僅用 c0_spike 分類口徑
# 輸出：results.json + summary.md（與 c0_spike 同目錄風格）

import argparse, json, os, re, subprocess, sys, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]

def sanitize(src: str) -> str:
    return "\n".join([ln for ln in src.splitlines() if not ln.lstrip().startswith("#")]) + "\n"

def corpus():
    items = []
    sm = open(ROOT/"core/src/semantic_matrix.rs", encoding="utf-8").read()
    for name, body, cat in re.findall(r'SemanticCase::new\("(\w+)",\s*"((?:[^"\\]|\\.)*)",\s*(?:true|false),\s*vec!\[[^\]]*\],\s*"(\w+)"\)', sm):
        items.append((name, f"matrix/{cat}", sanitize(body.encode().decode("unicode_escape"))))
    for sub in ("examples", "examples/phase3"):
        d = ROOT/sub
        if not d.is_dir():
            continue
        for f in sorted(os.listdir(d)):
            if f.endswith(".poly"):
                src = open(d/f, encoding="utf-8").read()
                items.append((f"{sub.split('/')[-1]}/{f[:-5]}", "examples", sanitize(src)))
    return items

def rustc_oracle(src: str, timeout=10):
    # 用 polyrust align-check --json 若二進制存在，否則直調 rustc
    pol = ROOT/"target/debug/polyrust"
    if pol.exists():
        with tempfile.NamedTemporaryFile(mode="w", suffix=".rs", delete=False) as tf:
            tf.write(src); tf.flush()
            tf_name = tf.name
        try:
            p = subprocess.run([str(pol), "align-check", tf_name, "--json"], capture_output=True, text=True, timeout=timeout)
            try:
                # 取最後一行 JSON（前面可能有 banner）
                lines = [l for l in p.stdout.strip().splitlines() if l.strip().startswith("{")]
                j = json.loads(lines[-1]) if lines else {}
                return j.get("rustc","?"), j.get("rustc_code"), j.get("aligned",True), j.get("violation"), j.get("rustc_detail","")
            except Exception as e:
                return "?", None, True, None, p.stdout+p.stderr
        finally:
            try: os.unlink(tf_name)
            except: pass
    # fallback：直調 rustc
    candidates = ["/home/user/.cargo/bin/rustc","/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc","rustc"]
    with tempfile.TemporaryDirectory() as td:
        inp = os.path.join(td,"input.rs"); out=os.path.join(td,"out.o")
        open(inp,"w").write(src)
        for cand in candidates:
            if cand!="rustc" and not os.path.exists(cand): continue
            try:
                p=subprocess.run([cand,"--crate-type","lib","--edition","2021","-o",out,inp], capture_output=True, text=True, timeout=timeout)
                combined=(p.stderr or "")+"\n"+(p.stdout or "")
                if p.returncode==0:
                    return "Accepted", None, True, None, ""
                m=re.search(r"error\[E(\d+)\]", combined)
                if m:
                    return "Rejected", f"E{m.group(1)}", True, None, combined[:400]
                if "error:" in combined:
                    return "ExternalDep", None, True, None, combined[:400]
                return "Rejected", None, True, None, combined[:200]
            except subprocess.TimeoutExpired:
                return "MissingToolchain", None, True, None, "timeout"
    return "MissingToolchain", None, True, None, "no rustc"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rustc-only", action="store_true", help="僅跑 rustc 分類，不調用 align-check")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--out", default="spike_out_rsap")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    items = corpus()
    if args.limit: items = items[:args.limit]
    results=[]
    for name, cat, rs in items:
        rv, code, aligned, violation, detail = rustc_oracle(rs)
        results.append({
            "name": name, "cat": cat,
            "rustc": rv, "rustc_code": code,
            "aligned": aligned, "violation": violation,
            "detail": detail[:200] if detail else ""
        })
    # 聚合
    from collections import Counter
    cnt = Counter(r["rustc"] for r in results)
    viol = [r for r in results if not r["aligned"]]
    with open(os.path.join(args.out,"results.json"),"w") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)
    lines = [
        "# RSAP — Rustc 對準檢查（align-check）",
        "",
        f"總樣本 {len(results)}；rustc Accepted {cnt['Accepted']} | Rejected {cnt['Rejected']} | ExternalDep {cnt['ExternalDep']} | MissingToolchain {cnt['MissingToolchain']}",
        f"對準違規 **{len(viol)}**（必須 0） → {'PASS ✓' if len(viol)==0 else 'FAIL ✗'}",
        "",
        "## 按類別",
        "| 類別 | Accepted | Rejected | ExternalDep | Missing |",
        "|---|---|---|---|---|",
    ]
    from collections import defaultdict
    bycat=defaultdict(Counter)
    for r in results: bycat[r["cat"]][r["rustc"]]+=1
    for c in sorted(bycat):
        cc=bycat[c]
        lines.append(f"| {c} | {cc['Accepted']} | {cc['Rejected']} | {cc['ExternalDep']} | {cc['MissingToolchain']} |")
    if viol:
        lines += ["", "## 違規明細", ""]
        for v in viol: lines.append(f"- `{v['name']}` {v['rustc']} → {v['violation']}")
    else:
        lines += ["", "## 違規明細", "", "（無）"]
    with open(os.path.join(args.out,"summary.md"),"w") as f: f.write("\n".join(lines)+"\n")
    if args.json:
        print(json.dumps({"total": len(results), "violations": len(viol), "by_rustc": dict(cnt), "pass": len(viol)==0}, ensure_ascii=False))
    else:
        print("\n".join(lines[:12]))
        print(f"\n== RSAP 驗收：違規 {len(viol)}/ {len(results)} → {'PASS' if len(viol)==0 else 'FAIL'}")
    return 0 if len(viol)==0 else 1

if __name__=="__main__": sys.exit(main())
