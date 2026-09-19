#!/usr/bin/env python3
"""漏斗量測報告：由護欄批跑產出量化各閘門攔截率、收斂輪次、修復有效率。

用法：
    python3 scripts/funnel_report.py \
        --summary output/suite/summary.json \
        --funnel-log output/guardrail-funnel.ndjson \
        --suite docs/evidence/guardrail-suite.json [--md output/guardrail-report.md]

判據與 `polyrust funnel` 同源（同一份 NDJSON 契約）；本腳本另加：
逐閘門攔截率、收斂輪次分佈、修復有效率、案例預期對照（護欄判別力）。
"""
import argparse, json
from collections import Counter, defaultdict

BLOCK = ["structure-error", "syntax-error", "checker-reject", "unsat", "unresolved"]


def load_funnel(path):
    runs = []
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        try:
            runs.append(json.loads(line))
        except json.JSONDecodeError:
            pass
    return runs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--summary", default="output/suite/summary.json")
    ap.add_argument("--funnel-log", default="output/guardrail-funnel.ndjson")
    ap.add_argument("--suite", default="docs/evidence/guardrail-suite.json")
    ap.add_argument("--md", default="")
    args = ap.parse_args()

    runs = load_funnel(args.funnel_log)
    summary = json.load(open(args.summary))
    expect = {c["id"]: set(c["expect"]) for c in json.load(open(args.suite))["cases"]}

    L = []
    P = L.append

    # ---- 1. 漏斗逐層 ----
    rounds = [a for r in runs for a in (r.get("attempt_log") or [])]
    oc = Counter(a.get("outcome") for a in rounds)
    reached = sum(1 for a in rounds if a.get("outcome") != "provider-error")
    g0 = sum(1 for a in rounds if a.get("outcome") not in ("provider-error", "structure-error"))
    g1 = sum(1 for a in rounds if a.get("outcome") not in ("provider-error", "structure-error", "syntax-error"))
    t0 = sum(1 for a in rounds if a.get("outcome") in ("checker-reject", "unsat", "unresolved", "sat"))
    sat = oc.get("sat", 0)

    def pct(x, y):
        return f"{100.0 * x / y:.1f}%" if y else "—"

    P(f"# 護欄漏斗量測報告（{len(runs)} 次運行）\n")
    P("## 1. 逐閘門到達／攔截（以輪次計）\n")
    P("| 層 | 到達 | 通過 | 攔截 | 攔截率（佔到達） |")
    P("|---|---:|---:|---:|---:|")
    P(f"| 到達護欄（非 provider 錯誤輪次） | {reached} | — | — | — |")
    P(f"| 閘門 0 結構 | {reached} | {g0} | {reached - g0} | {pct(reached - g0, reached)} |")
    P(f"| 閘門 1 語法 | {g0} | {g1} | {g0 - g1} | {pct(g0 - g1, g0)} |")
    P(f"| Tier-0 checker 快篩 | {g1} | {t0} | {g1 - t0} | {pct(g1 - t0, g1)} |")
    P(f"| 閘門 2 完整管線 | {t0} | {sat} | {t0 - sat} | {pct(t0 - sat, t0)} |")
    P("")
    P(f"provider-error 輪次：{oc.get('provider-error', 0)}（未進入任何閘門）")
    P("")
    P("### 輪次結果分佈")
    P("")
    P("| outcome | 輪次 | 佔比 |")
    P("|---|---:|---:|")
    for k, v in oc.most_common():
        P(f"| {k} | {v} | {pct(v, len(rounds))} |")
    P("")

    # ---- 2. 收斂輪次 ----
    sat_runs = [r for r in runs if r.get("verdict") == "SAT"]
    P("## 2. 收斂輪次（verdict=SAT 的運行）\n")
    if sat_runs:
        rounds_used = [r.get("attempts") or len(r.get("attempt_log") or []) for r in sat_runs]
        dist = Counter(rounds_used)
        avg = sum(rounds_used) / len(rounds_used)
        P(f"成功運行：{len(sat_runs)}／{len(runs)}（成功率 {pct(len(sat_runs), len(runs))}）")
        P(f"平均收斂輪次：**{avg:.2f}**；最多 {max(rounds_used)} 輪")
        P("")
        P("| 輪次 | 運行數 |")
        P("|---:|---:|")
        for k in sorted(dist):
            P(f"| {k} | {dist[k]} |")
    else:
        P("（無 SAT 運行）")
    P("")

    # ---- 3. 修復有效率 ----
    P("## 3. 修復回餵有效性（收斂動力）\n")
    repaired = 0
    rejected_then_next = 0
    for r in runs:
        log = r.get("attempt_log") or []
        for i, a in enumerate(log[:-1]):
            if a.get("outcome") in BLOCK:
                rejected_then_next += 1
                if log[i + 1].get("outcome") == "sat":
                    repaired += 1
    P(f"被拒絕後有下一輪的機會數：{rejected_then_next}；下一輪即過：{repaired}"
      f"（修復有效率 {pct(repaired, rejected_then_next)}）")
    P("（修復有效率 = 拒絕回餵後緊接一輪通過的比例；越高代表回餵嘅精確錯誤信息越有用）")
    P("")

    # ---- 4. 逐模型 ----
    P("## 4. 逐模型\n")
    P("| 模型 | 運行 | SAT | 成功率 | 平均收斂輪次 | provider 錯誤 |")
    P("|---|---:|---:|---:|---:|---:|")
    by_model = defaultdict(list)
    for r in runs:
        by_model[r.get("model", "?")].append(r)
    for m, rs in sorted(by_model.items()):
        sr = [r for r in rs if r.get("verdict") == "SAT"]
        ru = [r.get("attempts") or len(r.get("attempt_log") or []) for r in sr]
        pe = sum(1 for r in rs for a in (r.get("attempt_log") or [])
                 if a.get("outcome") == "provider-error")
        P(f"| `{m}` | {len(rs)} | {len(sr)} | {pct(len(sr), len(rs))} | "
          f"{sum(ru) / len(ru) if ru else '—':.2f} | {pe} |")
    P("")

    # ---- 5. 案例預期對照 ----
    P("## 5. 護欄判別力（實際結果 × 案例預期）\n")
    P("| 案例 | 類別 | 模型 | 結果 | 最終 outcome | 預期符合 |")
    P("|---|---|---|---|---|---|")
    ok, tot = 0, 0
    for s in summary:
        exp = expect.get(s["id"], {"any"})
        if exp == {"any"}:
            mark = "（記錄）"
        elif s["outcome"] in exp or (s["verdict"] == "SAT" and "sat" in exp):
            mark = "✓"; ok += 1; tot += 1
        else:
            mark = "✗"; tot += 1
        P(f"| {s['id']} | {s['cat']} | `{s['model']}` | {s['verdict']} | {s['outcome']} | {mark} |")
    P("")
    P(f"判別力：{ok}／{tot} 符合預期（含糊／注入案例不計入）")
    P("")

    txt = "\n".join(L)
    print(txt)
    if args.md:
        open(args.md, "w").write(txt + "\n")
        print(f"[已寫入 {args.md}]")


if __name__ == "__main__":
    main()
