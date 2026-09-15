# polyrust — Rust 宏的代数形式化

**CDCL × Buchberger × QAP**：把 Rust（Mini-Rust 子集）的语法规则、类型检查与借用检查转换为布尔多项式方程组，用三方联合求解，并从代数见证合成可编译的 Rust 代码。

## 仓库结构（Cargo workspace）

```
core/            polyrust-core —— 形式化管线核心【零第三方依赖，只用 std】
                 · lib（供前端依赖）+ 二进制 `polyrust`（CLI，向后兼容）
                 · Lean 4 形式化库静态嵌入（有工具链时自动，无则优雅降级）
frontends/
  http/          polyrust-http —— axum/tokio HTTP API 前端（/health、/api/nl、/api/funnel）
  llm/           polyrust-nl  —— ureq（纯 Rust TLS）传输的 LLM 护栏前端
lean/            Lean 4 形式化（零依赖；20 模块、408 定理，见 docs/LEAN.md）
docs/            THEOREMS / LEAN / EVIDENCE / LLM / POLY_DSL / FORMAL_LEMMAS
scripts/         审计与批测脚本
```

**核心承诺**：`core` 永远零第三方依赖；一切第三方依赖只出现在 `frontends/*`。

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
cargo test --release                    # 67 个核心单元测试
```

### 可输入模式（Phase 0）：`.poly` DSL 验证型工具

polyrust 已从「内建样本的验证器」升级为「可输入的验证型工具」。用 `.poly` 描述语言
（Mini-Rust 子集源碼 + `# @intent` metadata）输入任意程序，验证其型别/借用、求解、
并还原一份可编译的 Rust 代码：

```bash
./target/release/polyrust check examples/sqr.poly          # 完整管線：判定 + 生成碼
./target/release/polyrust check examples/sqr.poly --json   # 結構化 JSON（供前端/LLM）
./target/release/polyrust expand examples/sqr.poly         # 純宏展開（描述→展開碼）
./target/release/polyrust gen examples/template.poly       # 描述→生成碼（@import + @set 模板）
cat foo.poly | ./target/release/polyrust check - --json    # stdin 模式（LLM agent）
./target/release/polyrust serve 8080                       # 啟動 Web UI（瀏覽器操作）
```

`.poly` 支援三層描述能力（Phase 2）：**`@import`** 內聯函式庫（內建 `std/`：`basic`/`math`/`bool`）、
**`@set` + `{{key}}`** 模板實例化、**`gen`** 描述→生成碼。详见
**[docs/POLY_DSL.md](docs/POLY_DSL.md)**。

`.poly` 格式、子命令、JSON 契约详见 **[docs/POLY_DSL.md](docs/POLY_DSL.md)**。
这是「LLM 描述逻辑 → polyrust 形式化保证正确」闭环的接口层。

### 前端（有第三方依赖；可选用）

```bash
./target/release/polyrust-http 8090      # axum HTTP API：/health、/api/nl、/api/funnel
./target/release/polyrust-nl "把 1 加 2" --base-url https://openrouter.ai/api/v1 \
    --model <slug>                       # ureq 原生 TLS 传输跑 LLM 护栏（无需系统 curl）
```

两者与核心自带 `serve`（std-only）平行存在：要生态整合/正式路由用前端，要零依赖部署用核心。

### Web UI（Phase 1）

`polyrust serve [port]` 启动一个内嵌的极简 HTTP server（std-only，零依赖），
提供网页界面：贴上 `.poly` 描述 → 撳「驗證 + 生成」即显示判定、统计与生成的 Rust 代码
（含 rustc 编译结果），或撳「純展開」看宏展开。API 端点：

| 端点 | 方法 | 说明 |
|---|---|---|
| `/` | GET | 网页界面（inline CSS/JS，无外部资源） |
| `/health` | GET | 存活探针 |
| `/api/check` | POST | body = `.poly` 文本，回 `check` JSON 契约 |
| `/api/expand` | POST | body = `.poly` 文本，回 `expand` JSON 契约 |
| `/api/v1/generate` | POST | body = `.poly` 文本，回 `generate` JSON 契约（描述→生成碼） |

### 最优化 release 编译（含「排除执行期无需要的文件」）

`Cargo.toml` 的 `[profile.release]` 采用执行期最优化设定：

| 设定 | 值 | 作用 |
|---|---|---|
| `opt-level` | `3` | 最高一般优化 |
| `lto` | `"fat"` | 全程式链接期优化（跨模组内联 / 去死码） |
| `codegen-units` | `1` | 单一 codegen unit，给 LLVM 最大优化视野 |
| `panic` | `"abort"` | 移除 landing pad，缩小体积、减少展开开销 |
| `debug` | `false` | 不带除错符号 |
| `strip` | `true` | 剥除符号表 |
| `overflow-checks` | `false` | 关闭整数溢位检查 |

再加上 build.rs 以 `--gc-sections` 剔除未使用区段，产出的二进位**不含除错符号、符号表与任何执行期用不到的区段**（约 3.5 MB，`ldd` 仅依赖 glibc）。

### 内嵌 Lean 4 形式化库（.lean → .a → 编入二进位）

`lean/Polyrust` 的 14 个定理模组会被编译成静态库并**静态嵌入**二进位档内：

```bash
bash scripts/build-embedded.sh   # 一键：lake build Polyrust:static → cargo build --release
```

机制（[`build.rs`](build.rs)）：

1. 定位 `lean/.lake/build/lib/libpolyrust_x2dformal_Polyrust.a`；不存在时自动
   `lake build Polyrust:static`（需要 elan，见 `scripts/setup-lean.sh`）。
2. 以**静态链接**把 `.a` 与 Lean 执行期（`libleancpp`/`libLean`/`libStd`/`libInit`/
   `libleanrt` + `libc++`/`libc++abi`/`libunwind`/`libgmp`/`libuv`/`libssl`/`libcrypto`）
   一并编入二进位，并设 `cfg(has_lean_embed)`。
3. Rust 侧（[`src/formal.rs`](src/formal.rs)）在启动时呼叫 Lean 执行期入口
   `initialize_polyrust_x2dformal_Polyrust` 载入全部定理；头部会打印内嵌状态：
   `〔Lean 4 形式化库（Polyrust.*，14 模组）已静态嵌入并载入 ✓〕`。
4. 无 Lean 工具链（或 macOS/Windows 交叉编译）时**优雅降级**：跳过内嵌，
   polyrust 仍正常编译运作，仅该状态列改为「未内嵌」。

验证：`ldd target/release/polyrust` 不含 `lean`/`gmp`/`uv`/`ssl`/`crypto`/`libc++`，
仅剩 glibc —— 即「.lean 编成 .a、最终编入二进位」且无执行期多余依赖。

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

v0.1.0 的实测结果记录在 [Release 说明](https://github.com/djrww/polyrust/releases/tag/v0.1.0) 中，
CI 历史见 [Actions](https://github.com/djrww/polyrust/actions)（本表列的每一项都是硬闸门，
除了标注「资讯性」的 fmt/clippy）。

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
