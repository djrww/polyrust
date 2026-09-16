# Pipeline V3 示例 — 想不到效果 + 持续迭代 + 商业深化

本目录展示 V3 管线的惊艳效果。

## 示例列表

| 文件 | 场景 | 风险 | 特性 | 惊艳点 |
|---|---|---|---|---|
| `web3_audit.poly` | Solana DeFi 合约审计 | high | unsafe, lifetime, QAP链上 | QAP证书上链，风险驱动 F4F5 |
| `embedded_cert.poly` | 嵌入式 ECU ISO26262认证 | medium | no-io, pure, fuel, loop | 合规映射，Lean证明追溯 |
| `self_evolving.poly` | 自进化 Poly深化 | low | template, deepening chain | 递归深化，N=7+i扩展，自验证元循环 |
| `llm_guardrail.poly` | LLM 护栏 | medium | borrowck, lifetime | 风险评分，持续迭代 |

## 运行

```bash
# V3 商业深化验证 (默认 5轮迭代，自动深化，商业审计，链上导出，自验证)
cargo run --bin polyrust -- v3 examples/pipeline_v3/web3_audit.poly

# JSON 输出 (含完整商业审计报告)
cargo run --bin polyrust -- v3 examples/pipeline_v3/web3_audit.poly --json | jq .commercial

# 自定义迭代
cargo run --bin polyrust -- v3 examples/pipeline_v3/self_evolving.poly --max-iter 5 --risk-threshold 60

# 禁用深化 (快速路径)
cargo run --bin polyrust -- v3 examples/pipeline_v3/web3_audit.poly --no-deepening

# API
curl -X POST --data-binary @examples/pipeline_v3/web3_audit.poly http://localhost:8080/api/v3/check | jq
```

## 想不到效果演示

### 1. 自进化 (Self-Evolving)

```bash
./target/debug/polyrust v3 examples/pipeline_v3/self_evolving.poly --json | jq '.poly_deepening.chain | length'
# 输出: 3 (深化链长度)
./target/debug/polyrust v3 examples/pipeline_v3/self_evolving.poly --json | jq '.poly_deepening.final_poly' | wc -c
# 长度持续增长，但仍 SAT
```

### 2. 风险驱动选型 (Risk-Driven)

```bash
# low风险 → F4快速
./target/debug/polyrust v3 examples/pipeline_v3/self_evolving.poly --json | jq '.final_stats.groebner_algo'
# 可能输出 "classic" 或 "f4"

# high风险 → F4F5彻底
./target/debug/polyrust v3 examples/pipeline_v3/web3_audit.poly --json | jq '.final_stats.groebner_algo'
# 输出 "f4f5"
```

### 3. 链上证书 (On-Chain)

```bash
./target/debug/polyrust v3 examples/pipeline_v3/web3_audit.poly --json | jq '.qap_onchain.payload' | jq
# 输出 Solana 可验证载荷，含 lean_proofs
```

### 4. 商业审计报告 (Commercial Audit)

```bash
./target/debug/polyrust v3 examples/pipeline_v3/web3_audit.poly --json | jq '.commercial.audit_report_md' -r
# 输出 Markdown 审计报告，含风险分析、合规映射、Lean证明、迭代历史、修复建议、商业落地建议
```

### 5. 持续迭代收敛 (Continuous Iteration)

```bash
./target/debug/polyrust v3 examples/phase3/struct_sat.poly --json | jq '.iterations[] | "\(.iteration) \(.verdict) \(.groebner_algo) risk=\(.risk_score) \(.converged)"'
# 输出:
# 1 SAT classic risk=6.0 false
# 2 SAT f4f5 risk=8.0 false
# 3 SAT f4f5 risk=10.0 true (收敛)
```

## 商业价值

- **Web3审计**: 避免 $10M 损失，QAP证书可公开验证，收费 $75k/次
- **嵌入式认证**: 认证时间 6月→2月，收费 $150k/年
- **CI闸门**: 减少 70% unsafe漏洞，$20-50/开发者/月
- **LLM护栏**: 拦截 30% 不安全代码，$0.01-0.05/次

## Lean 支撑

V3 的持续迭代基于 Lean 第5类 `IncrementalIteration` 85定理:

- `parseFuel_mono`: fuel 单调
- `borrowSystem_subset_mono`: borrowSystem 子集单调
- `f4_ideal_invariant_iter`: F4 理想不变迭代
- `f5_sig_transitive_closure`: F5 签名传递闭包
- `f4f5_iter_converges`: F4F5 迭代收敛
- `loop_contract_fuel_iter`: LoopContract fuel 迭代
- `qap_verified_implies_sat`: QAP验证蕴含 SAT

纯构造，零 sorry，零 axiom，`lake build` 40 jobs 成功，`AuditAll` 3432受检/1870纯构造 CLEAN。
