# Commercial Pipeline — missing_syntax — txt

- NL: 一個簡單函數
- Poly len: 511 chars
- Fully complete (4 layers): true
- Functional passed: Some(true)
- Risk: 12.0 | QAP: false
- Duration: 772ms

## AST — N=7 fns=3 complete=true

ProgramV2 (universe N=7) {
  items: 2 | structs: 0 enums: 0 fns: 3 traits: 0 impls: 0 mods: 0
  ├─ fn process(0 params) -> ()
  ├─ fn validate(0 params) -> ()
  └─ fn main() { 7 items in body }
}

Stats: structs=0 enums=0 fns=3 traits=0 impls=0 mods=0 consts=0 statics=0 types=0 universe_n=7


## MIR — N=7 products=0 complete=true

Lowered MIR {
  products: 0 ([])
  sums: 0 ([])
  generated: 0 items
  mod_map: 0 entries
  universe N=7
  program items: 2
  stats: Lowered: 0 products, 0 sums, 0 generated, 0 mod_map, universe N=7
Products: []
Sums: []

}


## V3 — SAT — Risk 12.0 — QAP Some(false)

Business: 低风险代码，适合直接上链或嵌入式部署，QAP 证书可作为审计见证

Loss avoided: $10k-50k (避免轻微缺陷)

## Native — compile true — binary 4506688 bytes

## Solana On-Chain

- core/output/commercial_pipeline/solana/missing_syntax_solana_payload.json
- core/output/commercial_pipeline/solana/missing_syntax_solana_program.rs
- core/output/commercial_pipeline/solana/missing_syntax_anchor_idl.json
- core/output/commercial_pipeline/solana/missing_syntax_client.ts
- core/output/commercial_pipeline/solana/missing_syntax_deploy.sh
- core/output/commercial_pipeline/solana/missing_syntax_Cargo.toml

## Output Files

- core/output/commercial_pipeline/missing_syntax_commercial.poly
- core/output/commercial_pipeline/missing_syntax_commercial.rs
- core/output/commercial_pipeline/missing_syntax_audit.md
- core/output/commercial_pipeline/missing_syntax_audit.json
- core/output/commercial_pipeline/solana/missing_syntax_solana_payload.json
- core/output/commercial_pipeline/solana/missing_syntax_solana_program.rs
- core/output/commercial_pipeline/solana/missing_syntax_anchor_idl.json
- core/output/commercial_pipeline/solana/missing_syntax_client.ts
- core/output/commercial_pipeline/solana/missing_syntax_deploy.sh
- core/output/commercial_pipeline/solana/missing_syntax_Cargo.toml
