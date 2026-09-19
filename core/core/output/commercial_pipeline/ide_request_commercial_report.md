# Commercial Pipeline — ide_request — txt

- NL: 實現一個帶 LSP 的 Enterprise IDE，支持文件樹 FileTree 和文本緩衝 TextBuffer，帶光標移動和診斷
- Poly len: 2163 chars
- Fully complete (4 layers): true
- Functional passed: Some(true)
- Risk: 26.0 | QAP: true
- Duration: 3018ms

## AST — N=18 fns=1 complete=true

ProgramV2 (universe N=18) {
  items: 14 | structs: 8 enums: 0 fns: 1 traits: 0 impls: 6 mods: 0
  ├─ struct Position {
  │   ├─ line: i32
  │   ├─ col: i32
  │   }
  ├─ struct Range {
  │   ├─ start: Position
  │   ├─ end: Position
  │   }
  ├─ struct FileNode {
  │   ├─ path: String
  │   ├─ content: String
  │   }
  ├─ impl  for FileNode
  ├─ struct FileTree {
  │   ├─ nodes: Vec<FileNode>
  │   }
  ├─ impl  for FileTree
  ├─ struct TextBuffer {
  │   ├─ text: String
  │   }
  ├─ impl  for Tex

## MIR — N=18 products=8 complete=true

Lowered MIR {
  products: 8 (["RustAnalyzer", "Range", "FileNode", "Position", "FileTree", "TextBuffer", "EnterpriseIDE", "Editor"])
  sums: 0 ([])
  generated: 0 items
  mod_map: 0 entries
  universe N=18
  program items: 14
  product RustAnalyzer: 1 fields
    - cache: i32
  product Range: 2 fields
    - start: Position
    - end: Position
  product FileNode: 2 fields
    - path: String
    - content: String
  product Position: 2 fields
    - line: i32
    - col: i32
  product FileTree: 1 fiel

## V3 — SAT — Risk 26.0 — QAP Some(true)

Business: 低风险代码，适合直接上链或嵌入式部署，QAP 证书可作为审计见证

Loss avoided: $10k-50k (避免轻微缺陷)

## Native — compile true — binary 4679320 bytes

## Solana On-Chain

- core/output/commercial_pipeline/solana/ide_request_solana_payload.json
- core/output/commercial_pipeline/solana/ide_request_solana_program.rs
- core/output/commercial_pipeline/solana/ide_request_anchor_idl.json
- core/output/commercial_pipeline/solana/ide_request_client.ts
- core/output/commercial_pipeline/solana/ide_request_deploy.sh
- core/output/commercial_pipeline/solana/ide_request_Cargo.toml

## Output Files

- core/output/commercial_pipeline/ide_request_commercial.poly
- core/output/commercial_pipeline/ide_request_commercial.rs
- core/output/commercial_pipeline/ide_request_audit.md
- core/output/commercial_pipeline/ide_request_audit.json
- core/output/commercial_pipeline/ide_request_qap.json
- core/output/commercial_pipeline/solana/ide_request_solana_payload.json
- core/output/commercial_pipeline/solana/ide_request_solana_program.rs
- core/output/commercial_pipeline/solana/ide_request_anchor_idl.json
- core/output/commercial_pipeline/solana/ide_request_client.ts
- core/output/commercial_pipeline/solana/ide_request_deploy.sh
- core/output/commercial_pipeline/solana/ide_request_Cargo.toml
