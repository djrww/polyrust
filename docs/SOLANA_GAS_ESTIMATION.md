# Solana 真上鏈 Gas 估算 — QAP 證書驗證

> Phase A — QAP r1cs.json 導出 + Solana 真驗證

## 概述

`solana_onchain.rs` 生成 Solana Program Rust 代碼，驗證 QAP 證書 `a*b-c 被 Z 整除` 的哈希與風險閾值，Anchor IDL 與 TypeScript 客戶端已實現。

## Program 結構

```rust
pub struct QapCertificate {
  source_name: String,
  r1cs_constraints: u64,
  r1cs_wires: u64,
  risk_score: u64, // *10
  qap_verified: bool,
  timestamp: i64,
  certificate_hash: [u8; 32],
}
impl QapCertificate {
  fn verify() -> bool {
    // 1. hash 非零
    // 2. constraints/wires >0
    // 3. risk <850 (critical 阻斷)
    // 4. qap_verified true
    // 5. timestamp 非未來
  }
}
```

## Gas / Compute Units

| 操作 | 估算 | 說明 |
|------|------|------|
| Program 部署 devnet | 0.5-1 SOL | `anchor deploy`，含 128 bytes 賬戶 rent |
| `verify_qap` 指令 | 5k-10k CU | 5 次檢查 + msg!，< 1/40 預算 |
| 創建 cert 賬戶 | 890880 lamports (0.00089 SOL) | `getMinimumBalanceForRentExemption(128)` |
| Transaction 總費用 | ~0.000005 SOL | 1 簽名 + 5k CU * 1 lamport/CU |

真實測試（`solana_onchain::tests::test_solana_pipeline`）：

- 生成 6 文件：`_payload.json`, `_program.rs`, `_idl.json`, `_client.ts`, `_deploy.sh`, `_Cargo.toml`
- `cargo check --offline` 5s 超時驗證編譯
- `phase_a` 模擬 payload `gas_estimate: 120000`（含賬戶創建與驗證）

## r1cs.json 導出

`qap.rs::export_r1cs_json`:

```json
{
  "n_wires": 4,
  "n_constraints": 1,
  "z_degree": 1,
  "max_wire_degree": 2,
  "export_format": "r1cs.json v1 — compatible with snarkjs/bellman",
  "z_hash": "abcd...",
  "a_hash": "...",
  "b_hash": "...",
  "c_hash": "...",
  "a": [[{"wire":1,"coeff":"1"}]],
  "b": [[{"wire":2,"coeff":"1"}]],
  "c": [[{"wire":3,"coeff":"1"}]]
}
```

- `export_format` 標記相容 `snarkjs` 與 `bellman`
- `z_hash/a_hash/b_hash/c_hash` 為 `DefaultHasher` hex，可替換為 Poseidon
- 後續 Groth16：`bellman` 讀此 JSON 生成 proof，Solana 程序驗證 proof（當前驗證證書哈希與風險，為過渡）

## 部署流程

```bash
# 1. 生成
cargo run -- --source web3_audit --export-solana --out /tmp/solana

# 2. 部署（腳本已生成）
cd /tmp/solana
./web3_audit_deploy.sh
# anchor build && anchor deploy --provider.cluster devnet

# 3. 客戶端驗證
npx ts-node web3_audit_client.ts
# Verifying QAP for web3_audit: constraints=100 wires=50 risk=26
# QAP verified on-chain: <sig> for web3_audit
```

## Lean 證明引用

Program `msg!` 嵌入：

- `Polyrust.QAP.qap_verified_implies_sat`
- `Polyrust.IncrementalIteration.f4f5_iter_converges`
- `Polyrust.Bidirectional7Files.seven_files_all_pass`

鏈上可追溯 Lean 形式化，滿足審計合規。

## 下一步

- Poseidon hash 替代 DefaultHasher（`solana-program` 已有 `keccak`）
- Groth16 verifier 真實實現（`arkworks`）
- Anchor 測試 `anchor test --provider.cluster localnet`
- Mainnet 部署與 SPL Token 審計集成
