# Commercial Pipeline — defi_audit_request — txt

- NL: Solana DeFi 審計核心，實現 check_balance 檢查餘額和 transfer 轉賬，帶手續費 fee_calc
- Poly len: 793 chars
- Fully complete (4 layers): true
- Functional passed: Some(true)
- Risk: 26.0 | QAP: true
- Duration: 1092ms

## AST — N=7 fns=4 complete=true

ProgramV2 (universe N=7) {
  items: 3 | structs: 0 enums: 0 fns: 4 traits: 0 impls: 0 mods: 0
  ├─ fn check_balance(0 params) -> ()
  ├─ fn transfer(0 params) -> ()
  ├─ fn fee_calc(0 params) -> ()
  └─ fn main() { 9 items in body }
}

Stats: structs=0 enums=0 fns=4 traits=0 impls=0 mods=0 consts=0 statics=0 types=0 universe_n=7


## MIR — N=7 products=0 complete=true

Lowered MIR {
  products: 0 ([])
  sums: 0 ([])
  generated: 0 items
  mod_map: 0 entries
  universe N=7
  program items: 3
  stats: Lowered: 0 products, 0 sums, 0 generated, 0 mod_map, universe N=7
Products: []
Sums: []

}


## V3 — SAT — Risk 26.0 — QAP Some(true)

Business: 低风险代码，适合直接上链或嵌入式部署，QAP 证书可作为审计见证

Loss avoided: $10k-50k (避免轻微缺陷)

## Native — compile true — binary 4507864 bytes

## Solana On-Chain

- core/output/commercial_pipeline/solana/defi_audit_request_solana_payload.json
- core/output/commercial_pipeline/solana/defi_audit_request_solana_program.rs
- core/output/commercial_pipeline/solana/defi_audit_request_anchor_idl.json
- core/output/commercial_pipeline/solana/defi_audit_request_client.ts
- core/output/commercial_pipeline/solana/defi_audit_request_deploy.sh
- core/output/commercial_pipeline/solana/defi_audit_request_Cargo.toml

## Output Files

- core/output/commercial_pipeline/defi_audit_request_commercial.poly
- core/output/commercial_pipeline/defi_audit_request_commercial.rs
- core/output/commercial_pipeline/defi_audit_request_audit.md
- core/output/commercial_pipeline/defi_audit_request_audit.json
- core/output/commercial_pipeline/defi_audit_request_qap.json
- core/output/commercial_pipeline/solana/defi_audit_request_solana_payload.json
- core/output/commercial_pipeline/solana/defi_audit_request_solana_program.rs
- core/output/commercial_pipeline/solana/defi_audit_request_anchor_idl.json
- core/output/commercial_pipeline/solana/defi_audit_request_client.ts
- core/output/commercial_pipeline/solana/defi_audit_request_deploy.sh
- core/output/commercial_pipeline/solana/defi_audit_request_Cargo.toml
