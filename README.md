# polyrust — Rust 宏的代数形式化

**CDCL × Buchberger × QAP**：把 Rust（Mini-Rust 子集）的语法规则、类型检查与借用检查转换为布尔多项式方程组，用三方联合求解，并从代数见证合成可编译的 Rust 代码。

## 命题 P

> Rust 宏程序的类型检查与借用检查可完整编码为布尔多项式方程组上的代数问题；该编码可靠且完备，可由 CDCL 布尔求解、Buchberger 演算法（Gröbner 基）判定与二次算术程序（QAP）见证三方联合求解，并能从代数见证合成可再解析、可被 rustc 编译、语义保持的 Rust 代码。

九条定理 T1–T9 联合蕴含命题 P：
数学证明见 **[docs/THEOREMS.md](docs/THEOREMS.md)**；Lean 4 形式化的对照、边界与「错了会怎样」见 **[docs/LEAN.md](docs/LEAN.md)**，代码在 **[lean/](lean/)**。

## 构建与运行

```bash
cargo build --release
./target/release/polyrust demo          # 四个端到端 demo（SAT→codegen+rustc / UNSAT→1∈G）
./target/release/polyrust obligations   # T1–T9 义务自证（12 程序 × 9 定理）
./target/release/polyrust gen A         # 打印指定 demo 生成码
./target/release/polyrust debug <file>  # σ_D 逐约束合法性检查
cargo test --release                    # 17 个单元测试
```

## 管线

```
源码 → 宏展开（卫生转录） → 约束生成（类型位元 one-hot + 规则方程 + 借用子句）
     → CDCL(T) 回圈（SAT 模型 ⇄ Buchberger 理论检查，学习子句以多项式并入）
     → Gröbner 判定（1 ∈ G ⟺ 不可定型） → 见证求解 σ
     → R1CS → QAP（Lagrange 基装配，Z | a·b−c 验证）
     → 代码生成（round-trip 重解析 + rustc 编译）
```

系数域为质域 **𝔽_p，p = 2⁶¹ − 1**（Mersenne 质数）；0/1 判定保真性由嵌入引理 L0 保证（见 THEOREMS.md §2）。

## 验证状态（2026-09-10）

| 项目 | 结果 |
|---|---|
| 单元测试 | 17/17 ✓ |
| demoA–D 端到端一致性 | 4/4 ✓（含 QAP 验证、篡改拒绝、rustc 编译） |
| 九条定理义务自证 | 全部通过 ✓ |
| Lean 4 形式化 | **14 模块 273 条定理全编译通过、零 sorry、零自定义公理**：T3(a) 对偶、T3(b) 消解恒等式、T4 终止界、T5 S-多项式准则、T6 无根证书、T7(a) 简化基唯一、T7(b) 宏展开同态、T8 QAP 忠实、L0 嵌入、T9 判定等价与代码生成 round-trip、**借用/所有权区间冲突** ✓ |
| Lean 4 审计 | `bash scripts/lean-audit.sh`：逐定理 `#print axioms`（仅 Lean 标准三公理）✓ |
| 12 样本实测 ↔ Lean 定理对照 | **[docs/EVIDENCE.md](docs/EVIDENCE.md)**（9/9 义务自证、17/17 测试、4/4 demo；原始输出 `docs/evidence/`，脚本 `scripts/lean-evidence.sh`）✓ |
