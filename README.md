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
| Lean 全环境公理审计 | `lean/AuditAll.lean`：`Polyrust.*` **1115 条宣告**全数受检（不只手列清单），`sorryAx` 0、非标准公理 0、纯构造性 567 条 ✓ |
| 12 样本实测 ↔ Lean 定理对照 | **[docs/EVIDENCE.md](docs/EVIDENCE.md)**（9/9 义务自证、17/17 测试、4/4 demo；原始输出 `docs/evidence/`，脚本 `scripts/lean-evidence.sh`）✓ |

## CI

[`.github/workflows/ci.yml`](.github/workflows/ci.yml)（`main` / `v0.1.0` 分支、`v*` tag、PR 触发）：

| Job | 内容 | 闸门 |
|---|---|---|
| `rust` | `cargo test --release`、最优化 release 编译 | **硬** |
| `rust` | `cargo fmt --check`、`cargo clippy` | 资讯性（基线有 1294 行重排 / 67 条风格建议，待后续清理） |
| `lean` | `lake build`（`leanprover/lean-action`，读 `lean/lean-toolchain`） | **硬** |
| `lean` | 原生 target `Polyrust:static` / `:shared`（`.a` / `.so`） | **硬** |
| `lean` | `AuditAll.lean` 全环境公理审计 | **硬**（`AUDIT_RESULT=CLEAN`） |
| `verify` | 上述两者汇总 | **硬** |
| `obligations` | 九条定理义务自证（约 6 分钟） | 独立，不挡 `verify` |
| `release-assets` | tag `v*` 时交叉编译 linux/macOS/Windows 并上传到 Release | — |

`v0.1.0` 的实测：[CI run #14](https://github.com/djrww/polyrust/actions/runs/34478029730) — **7/7 job success**。

### release profile

`Cargo.toml` 的 `[profile.release]` 采 `lto = "fat"` + `codegen-units = 1` + `panic = "abort"` + `strip = true`。

实测（`obligations` 工作负载，各 3 次；输出与基准**逐位元组一致**）：

| | 二进制 | `obligations` 均值 | 标准差 |
|---|---|---|---|
| 基准（仅 `debug = false`） | 1 223 712 B | 369.25 s | ±16.70 s |
| 最优化 profile | **785 624 B（−35.8%）** | **346.98 s（−6.03%）** | ±10.69 s |

体积缩减是确定收益。执行时间**逐轮配对 3/3 都是最优化版较快**，但两组 ±1σ 区间重叠
（且同一二进制重跑的波动达 8%），故 6% 这个数字**不足以宣称统计显著**——
主要收益应记在体积上。除错用 `[profile.release-debug]`（保留符号、`panic = "unwind"`）。

### Lean 侧的最优化编译设定：只对原生 target 生效

`lean/lakefile.toml` 的 `buildType = "release"` + `-march=native -flto=thin`
**只在建原生 target 时生效**，这点很容易误判：

```bash
lake build                                   # 只建 defaultTargets（Polyrust）
                                             # → 仅 .olean/.ilean/.c，17 jobs，
                                             #   从不调用 C 编译器，旗标完全不参与
lake build Polyrust:static Polyrust:shared   # 33 jobs：15 个模块各编一个 :c.o，
                                             # → ar 成 .a、lld 成 .so，旗标在此生效
```

实测佐证（Lean v4.33.1，AVX2/AVX-512 可用之主机）：

| 旗标 | `.a` | `.so` |
|---|---|---|
| 含 `-march=native -flto=thin` | 874 818 B | 253 944 B |
| 拿掉 `-march=native` | 787 954 B（**−11.0%**） | 254 248 B |

`.a` 相差 11% 证明旗标确实作用于物件码（`.o` 是中间产物、建完即清，所以 `find` 看不到，
但 `.a` 内含 15 个物件）。

两点须留意：

1. 本库零 `@[extern]`、零 `native_decide`，**原生库不改变任何证明的可信度**——
   定理判定一律走 Lean kernel 检查 `.olean`；原生库仅供需要嵌入执行时使用。
2. `-march=native` 的产物**只能在同级 CPU 上执行，不宜作为对外发布的可携 artifact**。
   需可携版请改 `-march=x86-64-v3` 后重建。
