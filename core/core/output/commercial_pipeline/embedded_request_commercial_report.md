# Commercial Pipeline — embedded_request — txt

- NL: 嵌入式 ECU 控制，實現 sensor_read 讀傳感器和 control_loop 控制循環，帶安全檢查 safety_check
- Poly len: 729 chars
- Fully complete (4 layers): true
- Functional passed: Some(true)
- Risk: 14.0 | QAP: false
- Duration: 1028ms

## AST — N=7 fns=5 complete=true

ProgramV2 (universe N=7) {
  items: 4 | structs: 0 enums: 0 fns: 5 traits: 0 impls: 0 mods: 0
  ├─ fn sensor_read(0 params) -> ()
  ├─ fn actuator_write(0 params) -> ()
  ├─ fn control_loop(0 params) -> ()
  ├─ fn safety_check(0 params) -> ()
  └─ fn main() { 8 items in body }
}

Stats: structs=0 enums=0 fns=5 traits=0 impls=0 mods=0 consts=0 statics=0 types=0 universe_n=7


## MIR — N=7 products=0 complete=true

Lowered MIR {
  products: 0 ([])
  sums: 0 ([])
  generated: 0 items
  mod_map: 0 entries
  universe N=7
  program items: 4
  stats: Lowered: 0 products, 0 sums, 0 generated, 0 mod_map, universe N=7
Products: []
Sums: []

}


## V3 — SAT — Risk 14.0 — QAP Some(false)

Business: 低风险代码，适合直接上链或嵌入式部署，QAP 证书可作为审计见证

Loss avoided: $10k-50k (避免轻微缺陷)

## Native — compile true — binary 4507824 bytes

## Solana On-Chain

- core/output/commercial_pipeline/solana/embedded_request_solana_payload.json
- core/output/commercial_pipeline/solana/embedded_request_solana_program.rs
- core/output/commercial_pipeline/solana/embedded_request_anchor_idl.json
- core/output/commercial_pipeline/solana/embedded_request_client.ts
- core/output/commercial_pipeline/solana/embedded_request_deploy.sh
- core/output/commercial_pipeline/solana/embedded_request_Cargo.toml

## Output Files

- core/output/commercial_pipeline/embedded_request_commercial.poly
- core/output/commercial_pipeline/embedded_request_commercial.rs
- core/output/commercial_pipeline/embedded_request_audit.md
- core/output/commercial_pipeline/embedded_request_audit.json
- core/output/commercial_pipeline/solana/embedded_request_solana_payload.json
- core/output/commercial_pipeline/solana/embedded_request_solana_program.rs
- core/output/commercial_pipeline/solana/embedded_request_anchor_idl.json
- core/output/commercial_pipeline/solana/embedded_request_client.ts
- core/output/commercial_pipeline/solana/embedded_request_deploy.sh
- core/output/commercial_pipeline/solana/embedded_request_Cargo.toml
