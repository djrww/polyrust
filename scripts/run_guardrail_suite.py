#!/usr/bin/env python3
"""護欄標準案例批跑器（真 API，OpenRouter 免費模型）。

用法：
    POLYRUST_LLM_API_KEY=sk-... python3 scripts/run_guardrail_suite.py \
        [--suite docs/evidence/guardrail-suite.json] [--out output/suite] \
        [--funnel-log output/guardrail-funnel.ndjson] [--only s1,u1] [--sleep 6]

設計：
- 金鑰只從環境變數讀取、只經子程序環境傳遞——**不寫入任何檔案**。
- 每次呼叫間隔 ≥ sleep 秒（免費額度限流）；provider-error 自動退避重試一次。
- 每個案例的完整 JSON 契約落盤 output/suite/<model>/<id>.json；
  漏斗 NDJSON 由 --funnel-log 累積（與 `polyrust funnel` 同源）。
"""
import argparse, json, os, subprocess, sys, time

# 端點：OpenRouter 喺本沙箱被出站規則剝 Authorization（對照實驗證實，見
# docs/evidence/guardrail-funnel-2026-09-15.md），故改用 Pollinations 匿名免費
# 端點（OpenAI 相容，/v1 別名，免金鑰）。模型：openai-fast（gpt-oss-20b 後端）。
# 預設：Pollinations 匿名免費端點（本沙箱可用）。
# 喺自己機器跑 OpenRouter 時：
#   python3 scripts/run_guardrail_suite.py --provider openrouter \
#       --base-url https://openrouter.ai/api/v1 \
#       --models "nex-agi/nex-n2.5-pro:free,inclusionai/ling-3.0-flash-fin:free,inclusionai/ling-3.0-flash-sante:free"
DEFAULT_PROVIDER = "pollinations"
DEFAULT_BASE_URL = "https://text.pollinations.ai/v1"
DEFAULT_MODELS = [
    # (model, case filter) —— 全量
    ("openai-fast", None),
]


def run_case(binary, model, case, attempts, out_dir, funnel_log,
             provider, base_url, timeout=360):
    env = dict(os.environ)
    cmd = [binary, "nl", case["prompt"],
           "--provider", provider, "--base-url", base_url,
           "--model", model, "--attempts", str(attempts),
           "--funnel-log", funnel_log, "--json"]
    try:
        p = subprocess.run(cmd, env=env, capture_output=True, text=True,
                           timeout=timeout)
    except subprocess.TimeoutExpired:
        return {"status": "error", "verdict": "ERROR",
                "final_reason": "runner timeout", "attempts": attempts}
    txt = p.stdout.strip()
    i = txt.find("{")
    if i < 0:
        return {"status": "error", "verdict": "ERROR",
                "final_reason": f"no json (rc={p.returncode}): {txt[:200]} {p.stderr[:200]}"}
    try:
        return json.loads(txt[i:])
    except json.JSONDecodeError as e:
        return {"status": "error", "verdict": "ERROR",
                "final_reason": f"bad json: {e}", "raw": txt[i:i + 400]}


def last_outcome(res):
    log = res.get("attempt_log") or []
    return log[-1]["outcome"] if log else res.get("outcome", "?")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--binary", default="target/release/polyrust")
    ap.add_argument("--suite", default="docs/evidence/guardrail-suite.json")
    ap.add_argument("--out", default="output/suite")
    ap.add_argument("--funnel-log", default="output/guardrail-funnel.ndjson")
    ap.add_argument("--attempts", type=int, default=3)
    ap.add_argument("--sleep", type=float, default=6.0)
    ap.add_argument("--only", default="", help="逗號分隔的案例 id，只跑呢啲")
    ap.add_argument("--provider", default=DEFAULT_PROVIDER)
    ap.add_argument("--base-url", default=DEFAULT_BASE_URL)
    ap.add_argument("--models", default="",
                    help="逗號分隔模型清單（覆蓋內建清單，全部案例全量跑）")
    args = ap.parse_args()

    # Pollinations 匿名端點免金鑰；若環境有 POLYRUST_LLM_API_KEY 會照傳（唔落盤）。
    suite = json.load(open(args.suite))["cases"]
    only = {x.strip() for x in args.only.split(",") if x.strip()}
    os.makedirs(args.out, exist_ok=True)
    os.makedirs(os.path.dirname(args.funnel_log) or ".", exist_ok=True)

    models = ([(m.strip(), None) for m in args.models.split(",") if m.strip()]
              if args.models else DEFAULT_MODELS)
    plan = []
    for model, subset in models:
        for case in suite:
            if only and case["id"] not in only:
                continue
            if subset is not None and case["id"] not in subset and not only:
                continue
            plan.append((model, case))

    print(f"批跑：{len(plan)} 次（{len({m for m, _ in plan})} 模型）",
          f"間隔 ≥{args.sleep}s，attempts={args.attempts}", flush=True)
    summary = []
    for k, (model, case) in enumerate(plan, 1):
        slug = model.split("/")[1].split(":")[0]
        d = os.path.join(args.out, slug)
        os.makedirs(d, exist_ok=True)
        t0 = time.time()
        res = run_case(args.binary, model, case, args.attempts, d, args.funnel_log,
                       args.provider, args.base_url)
        # provider-error 退避重試一次
        if last_outcome(res) == "provider-error" or res.get("status") == "error":
            print(f"  [{k}/{len(plan)}] {case['id']} provider 端問題，退避 20s 重試", flush=True)
            time.sleep(20)
            res = run_case(args.binary, model, case, args.attempts, d, args.funnel_log,
                           args.provider, args.base_url)
        res["_case"] = case["id"]
        res["_cat"] = case["cat"]
        res["_model"] = model
        res["_seconds"] = round(time.time() - t0, 1)
        json.dump(res, open(os.path.join(d, case["id"] + ".json"), "w"),
                  ensure_ascii=False, indent=1)
        outcome = last_outcome(res)
        summary.append({"model": slug, "id": case["id"], "cat": case["cat"],
                        "verdict": res.get("verdict"), "outcome": outcome,
                        "attempts_used": res.get("attempts"),
                        "seconds": res["_seconds"]})
        print(f"  [{k}/{len(plan)}] {slug:22s} {case['id']:4s} "
              f"verdict={res.get('verdict'):6s} outcome={outcome:16s} "
              f"attempts={res.get('attempts')} ({res['_seconds']}s)", flush=True)
        if k < len(plan):
            time.sleep(args.sleep)

    json.dump(summary, open(os.path.join(args.out, "summary.json"), "w"),
              ensure_ascii=False, indent=1)
    print(f"\n完成：{len(summary)} 次，明細 {args.out}/，漏斗 {args.funnel_log}")


if __name__ == "__main__":
    main()
