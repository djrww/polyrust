# `.poly` DSL 與 LLM 接口契約

Phase 0 把 polyrust 從「內建樣本的驗證器」升級為「可輸入的驗證型工具」。
本文記錄 `.poly` 描述語言格式、兩個新子命令、以及供前端／LLM 呼叫的 JSON 契約。

## 1. `.poly` 描述語言

`.poly` 檔案的內容 = **Mini-Rust 子集源碼** + 以 `#` 開頭的 metadata 行。

```text
# @intent: 計算平方，並用宏重複使用表達式
# @author: llm

fn sqr(x: i32) -> i32 { x * x }
macro_rules! sqr { ($e:expr) => { $e * $e } }
fn main() {
    let a = 5;
    let b = sqr!(a + 1);
    let c = sqr(3) + sqr!(2);
}
```

### metadata 語法

- `# @intent: …`：一句話描述「這段程式想做什麼」。polyrust **不做語義處理**，
  只回傳到 JSON 供下游（前端顯示、LLM 記錄）。可多行（以換行合併）。
- `# @key: value`：任意鍵值對，原樣回傳到 JSON 的 `metadata` 物件（擴充點）。
- `# @import: name`：內聯函式庫（見 §1.1）。
- `# @set key = value`：模板參數（見 §1.2）。
- `# 普通文字`：註釋，忽略。

### 1.1 函式庫 import

```text
# @import: basic      ← 內建 std（basic / math / bool）
# @import: mylib      ← 檔案系統 std/mylib.poly、./mylib.poly 或相對來源目錄
```

內建 std 函式庫（編譯期嵌入二進位，`std/` 目錄）：

| 名稱 | 內容 |
|---|---|
| `basic` | 宏：`sqr!` `dbl!` `cube!` `quad!` |
| `math` | 函式：`square` `twice` `cube`（單參數）、`add` `sub` `min`（多參數） |
| `bool` | 函式：`negate` |

import 支援遞迴（函式庫可再 import）、去重、循環偵測。函式庫只提供 `fn`/`macro_rules!`
定義（`fn main` 會被剝除），內聯進使用者源碼後一起解析。

### 1.2 模板（`@set` + `{{key}}`）

`.poly` 可以是**模板**：用 `{{key}}` 佔位，`@set key = value` 提供值，載入時替換。

```text
# @intent: 計算 (x + K) 的平方
# @import: basic
# @set init = 5
# @set k = 1
fn main() {
    let x = {{init}};
    let y = sqr!(x + {{k}});
}
```

載入後等同：

```text
fn main() {
    let x = 5;
    let y = sqr!(x + 1);
}
```

缺參數（有佔位無 `@set`）會報錯。這是「描述 → 具體程式」的實例化能力——LLM 可產出
含佔位的模板，再以不同 `@set` 組合產生多個變體，全部經 polyrust 驗證。

### 語言子集（現況）

| 特性 | 支援 |
|---|---|
| 型別 | `i32`、`bool`、`()`、`&i32`、`&mut i32`、`&bool`、`&mut bool` |
| 表達式 | 整數/布林/unit 字面量、變數、`let`、`;` 序列、`if`、`&x`、`&mut x`、`*e`、`x = e`、`*lhs = e`、`f(…)`、宏調用 `m!(…)` |
| 運算子 | `+` `-` `*` `<` `<=` `>=` `==` `!=` `&&` `!` `-e`（一元負號） |
| 頂層 | `fn` 定義（**多參數**）、`macro_rules!` 定義、`fn main` |

函式支援多個參數與多個實參（`fn add(a: i32, b: i32) -> i32`、`add(3, 4)`）；
實參個數不符、型別不符、未定義函式皆判定為不可定型（UNSAT）。

## 2. 子命令

```bash
polyrust check <file.poly> [--json]   # 完整管線：驗證 + 求解 + 代碼生成
polyrust expand <file.poly> [--json]  # 純宏展開（描述 → 展開碼，DSL 生成第一層）
polyrust gen <file.poly> [--json]     # 描述 → 生成碼（含 @import / @set 模板）
polyrust check - [--json]             # 從 stdin 讀取（LLM agent 模式）
polyrust nl "<自然語言>" [旗標] [--json] # 自然語言 → .poly（LLM 護欄；見 docs/LLM.md）
polyrust serve [port]                 # 啟動 Web UI（預設 8080）
```

### Web UI / HTTP API

`serve` 啟動內嵌 HTTP server，把 `check` / `expand` 包成網頁（textarea 貼 `.poly` →
顯示判定 + 生成碼）。HTTP 端點：

| 端點 | 方法 | body | 回傳 |
|---|---|---|---|
| `/` | GET | — | 網頁（inline CSS/JS） |
| `/health` | GET | — | `ok` |
| `/api/check` | POST | `.poly` 文本 | `check` JSON 契約（見下） |
| `/api/expand` | POST | `.poly` 文本 | `expand` JSON 契約 |
| `/api/v1/generate` | POST | `.poly` 文本 | `generate` JSON 契約（見下） |
| `/api/nl` | POST | JSON `{description,…}` 或純文字 | `nl` 護欄 JSON 契約（見 `docs/LLM.md`） |

```bash
curl -X POST --data-binary @examples/sqr.poly http://localhost:8080/api/check
```

`--json`（或 `-j`）時只輸出單行 JSON 到 stdout（無 banner），退出碼 `0`=成功、
`1`=錯誤（parse 失敗、管線錯誤）。人類可讀模式則印出判定、統計、型別解碼與生成碼。

## 3. JSON 契約（`api_version: "0.1"`）

### `check` 成功

```jsonc
{
  "api_version": "0.1",
  "mode": "check",
  "source": "sqr",                 // 檔名（stdin 時為 "stdin"）
  "intent": "計算平方…",            // @intent（無則 null）
  "metadata": {"author": "llm"},    // 其餘 @key: value
  "status": "ok",
  "verdict": "SAT" | "UNSAT",       // 可定型 / 不可定型
  "typechecks": true,               // ground-truth 型別檢查是否接受
  "checker_msg": "接受（…）",
  "agrees": true,                   // 管線判定 ⟺ checker（應恆為 true）
  "stats": {                        // 代數求解統計
    "n_vars": 170, "n_polys": 313, "n_clauses": 2,
    "cdcl_rounds": 1, "cdcl_decisions": 0, "cdcl_propagations": 0,
    "cdcl_conflicts": 0, "cdcl_learned": 0,
    "gb_generators": 313, "gb_pairs_considered": 142311,
    "gb_s_polys": 1496, "gb_basis_adds": 222, "reduced_basis_size": 170,
    "r1cs_constraints": 581, "r1cs_wires": 439,
    "qap_max_degree": 580, "qap_verified": true, "qap_tamper_rejected": true
  },
  "result": {
    "node_types": {"node0": "i32", …},   // 型別解碼（SAT 時）
    "arm_choice": {"invoke#5": 0, …}     // 宏臂選擇
  },
  "generated": {
    "code": "…",                    // 生成的 Rust 源碼（SAT 時）
    "file": "output/generated/sqr.rs",
    "rustc_compiles": true          // 生成碼是否通過 rustc 編譯
  },
  "expansion_log": ["invoke#5 臂1 → …"]
}
```

### `expand` 成功

```jsonc
{
  "api_version": "0.1", "mode": "expand", "source": "sqr",
  "intent": "…", "metadata": {…}, "status": "ok",
  "macros": [{"name": "sqr", "arms": 1}],
  "expansions": [{"node": 5, "arm": 0, "text": "( a + 1 ) * ( a + 1 )"}]
}
```

### `generate` 成功（Phase 2 的「描述 → 生成碼」出口）

```jsonc
{
  "api_version": "0.1", "mode": "generate", "source": "template",
  "intent": "計算 (x + K) 的平方", "status": "ok",
  "verdict": "SAT", "typechecks": true,
  "generated_code": "…",       // 生成 Rust 源碼（含 import 內聯 + 模板替換）
  "generated_file": "output/generated/template.rs",
  "rustc_compiles": true
}
```

### 錯誤（各命令通用）

```jsonc
{
  "api_version": "0.1", "mode": "check", "source": "…",
  "intent": null, "status": "error", "error": "讀取失敗／解析失敗／缺 @set 參數／找不到函式庫"
}
```

## 4. LLM 接口（已實作：`nl` 護欄層）

`.poly` 是 LLM 與 polyrust 之間的契約格式：**LLM 產生 `.poly` 文本，polyrust
負責驗證與還原生成碼**，形成「LLM 負責描述邏輯 → polyrust 形式化保證正確」的閉環。
此接口已由 `nl` 子命令 + 護欄層完整實作（**可接入任何 LLM API**），細節見
**[`docs/LLM.md`](LLM.md)**。

兩種使用方式：

**（甲）agent 模式**——外部 LLM 自己寫 `.poly`，polyrust 只當驗證器：

```
外部 LLM（寫 .poly） → polyrust check - --json → { verdict, typechecks, generated.code, rustc_compiles }
```

```bash
cat <<'EOF' | polyrust check - --json
# @intent: 傳回兩個整數之和
fn add(a: i32, b: i32) -> i32 { a + b }
fn main() { let r = add(2, 3); }
EOF
```

**（乙）護欄模式**——自然語言直接進，polyrust 內部呼叫任意 LLM 並用三道閘門
（結構 → 語法 → 完整代數管線語義）層層把關，失敗原因回餵修復：

```bash
polyrust nl "把 1 加 2，再乘 3" \
    --provider mycorp --base-url http://localhost:8000/v1 --model qwen2.5-coder
```

LLM 可依 `verdict` / `typechecks` / `rustc_compiles` 判斷生成碼是否正確，
必要時修正描述後重試；護欄模式則把這個重試迴圈內建化，且**絕不產出未經驗證的代碼**。

## 5. 範例

見 [`examples/`](../examples/)：`sqr.poly`（SAT，良構）、`bad.poly`（UNSAT，型別錯誤）、
`template.poly`（`@import` + `@set` 模板）。函式庫見 [`std/`](../std/)。

```bash
polyrust check examples/sqr.poly          # 判定 SAT，輸出生成碼
polyrust check examples/sqr.poly --json   # 結構化結果
polyrust expand examples/sqr.poly         # 純宏展開
polyrust check examples/bad.poly          # 判定 UNSAT（1∈G ⇒ 不可定型）
polyrust gen examples/template.poly       # 描述 → 生成碼（import + 模板實例化）
```
