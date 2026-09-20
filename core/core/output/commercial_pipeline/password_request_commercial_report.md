# Commercial Pipeline — password_request — txt

- NL: 密碼生成器，帶強度檢查 Strength 和熵計算 entropy，配置 PasswordConfig 需驗證長度
- Poly len: 1616 chars
- Fully complete (4 layers): true
- Functional passed: Some(true)
- Risk: 15.5 | QAP: false
- Duration: 1107ms

## AST — N=11 fns=3 complete=true

ProgramV2 (universe N=11) {
  items: 8 | structs: 2 enums: 1 fns: 3 traits: 0 impls: 3 mods: 0
  ├─ struct PasswordConfig {
  │   ├─ length: i32
  │   ├─ use_upper: bool
  │   ├─ use_lower: bool
  │   ├─ use_digits: bool
  │   ├─ use_symbols: bool
  │   }
  ├─ impl  for PasswordConfig
  ├─ enum Strength {
  │   ├─ Weak(..)
  │   ├─ Medium(..)
  │   ├─ Strong(..)
  │   }
  ├─ impl  for Strength
  ├─ fn entropy(0 params) -> ()
  ├─ struct PasswordGenerator {
  │   ├─ config: PasswordConfig
  │   }

## MIR — N=11 products=2 complete=true

Lowered MIR {
  products: 2 (["PasswordConfig", "PasswordGenerator"])
  sums: 1 (["Strength"])
  generated: 0 items
  mod_map: 0 entries
  universe N=11
  program items: 8
  product PasswordConfig: 5 fields
    - length: i32
    - use_upper: bool
    - use_lower: bool
    - use_digits: bool
    - use_symbols: bool
  product PasswordGenerator: 1 fields
    - config: PasswordConfig
  sum Strength: 3 variants
  stats: Lowered: 2 products, 1 sums, 0 generated, 0 mod_map, universe N=11
Products: ["Pa

## V3 — SAT — Risk 15.5 — QAP Some(false)

Business: 低风险代码，适合直接上链或嵌入式部署，QAP 证书可作为审计见证

Loss avoided: $10k-50k (避免轻微缺陷)

## Native — compile true — binary 4506720 bytes

## Solana On-Chain

- core/output/commercial_pipeline/solana/password_request_solana_payload.json
- core/output/commercial_pipeline/solana/password_request_solana_program.rs
- core/output/commercial_pipeline/solana/password_request_anchor_idl.json
- core/output/commercial_pipeline/solana/password_request_client.ts
- core/output/commercial_pipeline/solana/password_request_deploy.sh
- core/output/commercial_pipeline/solana/password_request_Cargo.toml

## Output Files

- core/output/commercial_pipeline/password_request_commercial.poly
- core/output/commercial_pipeline/password_request_commercial.rs
- core/output/commercial_pipeline/password_request_audit.md
- core/output/commercial_pipeline/password_request_audit.json
- core/output/commercial_pipeline/solana/password_request_solana_payload.json
- core/output/commercial_pipeline/solana/password_request_solana_program.rs
- core/output/commercial_pipeline/solana/password_request_anchor_idl.json
- core/output/commercial_pipeline/solana/password_request_client.ts
- core/output/commercial_pipeline/solana/password_request_deploy.sh
- core/output/commercial_pipeline/solana/password_request_Cargo.toml
