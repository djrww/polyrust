# 真 API 護欄測試（2026-09-14）

經 OpenRouter 免費模型實測 `polyrust nl` 護欄（自然語言 → `.poly` → 完整代數管線）。
Provider 適配層 + 三道護欄閘門首跑真實 LLM。

## 環境

- 端點：`https://openrouter.ai/api/v1`（OpenAI 相容，走 `OpenAiCompat` provider）
- 免費模型（均 262,144 context）：
  - `nex-agi/nex-n2.5-pro:free`
  - `inclusionai/ling-3.0-flash-fin:free`
  - `inclusionai/ling-3.0-flash-sante:free`

## 冒煙測試（三模型各一條「1 加 2 再乘 3」）

| 模型 | 結果 |
|---|---|
| nex-n2.5-pro | SAT，**首輪**過三道閘門 |
| ling-3.0-flash-fin | SAT，**首輪**過三道閘門 |
| ling-3.0-flash-sante | SAT，**首輪**過三道閘門 |

## 測試組（8 案例）

| # | 需求 | 模型 | 結果 | 說明 |
|---|---|---|---|---|
| 1 | 多參數函式 | nex | SAT / 1 輪 | 首跑 `rustc: False`（尾表達式，見下 bug 2）；修復後 `True` |
| 2 | 宏定義+調用 | ling-fin | SAT / 1 輪 | `rustc: True` |
| 3 | 比較+布爾 | ling-sante | SAT / 1 輪 | `rustc: True` |
| 4 | 雙可變引用 | nex | SAT / 1 輪 | 模型寫成 `*(&mut $x)+*(&mut $x)` 暫引用，與真 rustc 一致（皆可編譯），非借用衝突 |
| 5 | 條件分支 | ling-fin | SAT / 1 輪 | `rustc: True` |
| 6 | 型別不可能（`1 + true`） | ling-sante→fin | **UNSAT 拒絕** | 首輪寫出錯配 → 護欄回餵精確原因；模型修復失敗；最終**不產出未驗證代碼**（設計目的達成）|
| 7 | 提示注入 | nex | SAT / 1 輪 | 模型忽略注入、改寫 1+1；**系統提示未洩露**；`rustc: True`（修復後）|
| 8 | 含糊需求（「寫點東西」） | ling-fin | SAT / 1 輪 | 模型解讀為 `abs` 函式；`rustc: True` |

## 抓到並修復的兩個真 bug（離線測試抓不到）

1. **`temperature` 序列化**：原以字串送出，OpenRouter 回 `400 temperature: Invalid
   input: expected number`。修復：`json.rs` 新增 `J::Float` 變體，`llm.rs` 三處
   `temperature` 改送 JSON 數字。
2. **`main` 非 unit 尾表達式**：生成碼 `fn main(){ … result }` 過唔到真
   `rustc`（E0308 expected `()` found `i32`）。修復：`codegen.rs` `emit_expr` 加
   `discard_tail` 旗標——`main` 尾表達式非 unit 時以語句形式丟棄（補 `;`），
   round-trip 仍合法（解析為 `Seq(result, UnitLit)`），真 `rustc` 通過。

## 免費模型限制（觀察）

- 免費額度限流：案例 6 第二輪重試曾撞 `provider-error`（間隔後重跑即恢復）。
- 偶發回包殘缺（`choices[0].message.content` 缺失）：護欄以 `provider-error`
  乾淨回報，不產出任何代碼。
- 結論：**免費模型足以驅動護欄閉環**；限流以 `sleep`/重試緩解即可，必要時換付費層。

## 結語

- 三道閘門全部在真實 LLM 輸出上驗證：結構、語法、語義（UNSAT）拒絕路徑皆觸發。
- 8 案例中 7 個首輪即過；唯一失敗案例（不可能需求）正是護欄應拒絕者。
- 修復後回歸全綠：`cargo test` 48/48、4 demos、round-trip + `rustc` 皆 ✓。

## 附錄：超大需求「落地化」測試（同日）

以自然語言餵四個遠超 `.poly` 子集的大需求，驗證「落地化規則」（抽取計算
核心、誠實標明簡化）+ 免費模型 + 護欄的閉環。為此對護欄做了三項加固：
系統提示加入落地化規則與示範例、語法回餵偵測子集外語法（`for`/`struct`/
`Vec`/`String`）時指向落地化規則、provider 層自動重試（429/5xx/空 content）。

| 需求 | 模型 | 結果 | 落地核心 |
|---|---|---|---|
| 反應式渲染 GUI 平台 | nex | SAT / **1 輪** | `update_state` + `render`（可見性門控）|
| 嵌入式數據庫 | ling-fin | SAT / 2 輪 | 條件表達式模擬鍵值槽位查詢 |
| App 發佈平台 | ling-sante | SAT / 2 輪 | 狀態機：草稿→審核→發佈→撤回 |
| 短影音錄製 | ling-fin | SAT / 2 輪 | 錄製狀態機：開始/逐幀計數/停止 |

四個落地版存於 `examples/bigreq/*.poly`，逐個重驗：`verdict=SAT`、
`typechecks=True`、`rustc=True`、`agrees=True`。每個 `@intent` 都誠實標明
「最小計算核心模型（簡化：…，無真實 I/O）」——不假裝實現完整系統。
