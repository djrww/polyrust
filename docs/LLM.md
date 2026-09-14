# LLM 整合層：可插拔 Provider × `.poly` 護欄

> 定位：把「自然語言 → 已驗證的 Rust 代碼」閉環中**唯一未形式化的一環**
> （自然語言理解）外包給任意 LLM，同時用三道護欄閘門保證：**LLM 的產出
> 永遠不被信任——只有通過形式化管線的才算數**。

```
自然語言 ──▶ LLM（任意 API）──▶ .poly 候選 ──▶ 護欄三道閘門 ──▶ SAT ──▶ 生成 Rust（rustc ✓）
                                   ▲                    │
                                   └── 精確錯誤回餵 ◀──┘（結構/語法/語義拒絕原因）
```

## 1. 接入任何 API（Provider 適配）

`src/llm.rs` 定義統一介面 `trait LlmProvider { fn chat(&self, system, msgs) -> Result<String, String> }`。
內建四種適配，**新增供應商 = 實作一個 `chat` 方法**：

| `--provider` | 協定 | 預設端點 | 覆蓋範圍 |
|---|---|---|---|
| `openai` | Chat Completions | `https://api.openai.com/v1` | OpenAI 官方 |
| 任意自訂名（如 `mycorp`） | Chat Completions | 必須 `--base-url` | **一切 OpenAI 相容端點** |
| `ollama` / `vllm` / `lmstudio` | Chat Completions | `http://127.0.0.1:11434/v1` 等 | 本地推理 |
| `anthropic` | Messages API | `https://api.anthropic.com` | Claude 原生 |
| `gemini` | generateContent | `https://generativelanguage.googleapis.com` | Gemini 原生 |
| `mock` | 內建腳本 | — | 離線測試（確定性） |

Groq、Together、OpenRouter、DeepSeek、Moonshot、Azure 相容端點等全部走
「自訂名 + `--base-url`」，零代碼改動。

### 設定（旗標優先，環境變數遞補）

```bash
polyrust nl "把 1 加 2，再乘 3" \
    --provider mycorp --base-url http://localhost:8000/v1 \
    --model qwen2.5-coder --api-key sk-... \
    --attempts 5 --temperature 0.2 --max-tokens 2048
```

| 旗標 | 環境變數（遞補順序） | 預設 |
|---|---|---|
| `--provider` | — | `openai` |
| `--model` | `POLYRUST_LLM_MODEL` → `OPENAI_MODEL`／`ANTHROPIC_MODEL`／`GEMINI_MODEL` | 依 provider（如 `gpt-4o-mini`） |
| `--base-url` | `POLYRUST_LLM_BASE_URL` → `OPENAI_BASE_URL`／`ANTHROPIC_BASE_URL` | 依 provider |
| `--api-key` | `POLYRUST_LLM_API_KEY` → `OPENAI_API_KEY`／`ANTHROPIC_API_KEY`／`GEMINI_API_KEY`／`GOOGLE_API_KEY` | 本地端點免金鑰 |
| `--attempts` | — | 3（上限 16） |

HTTP 傳輸零依賴：`http://` 走裸 TCP（含 chunked 解碼），`https://` 走系統
`curl`。整個 LLM 層無任何新 crate 依賴。

## 2. 護欄（Guardrail）：三道閘門 + 修復回餵

每一輪 LLM 輸出必須**依次通過三道閘門**，任何一道失敗都會被拒絕，
且把**精確錯誤**回餵給 LLM 要求重寫（最多 `--attempts` 輪）：

| 閘門 | 內容 | 失敗時的回餵 |
|---|---|---|
| 0 結構 | 恰好一個 ` ```poly ` 圍欄塊、≤64 KB、含 `# @intent:` 首行 | 格式要求全文 |
| 1 語法 | `.poly` 元資料/模板層（`dsl::resolve`）+ Mini-Rust 程式體解析 | 解析器的**逐字錯誤訊息** |
| 2 語義 | **完整代數管線**：約束生成 → CDCL(T) → Buchberger → QAP | `checker_msg` + 展開日誌（例如「拒絕：main 體不可定型」「invoke#0 臂1 → 1 + true」） |

關鍵性質：

- **零信任**：閘門 2 就是 §THEOREMS T1/T2/T6 的同一條管線；LLM 產出與人手寫
  的 `.poly` 走完全相同的驗證，沒有任何特殊通道。
- **可修復**：UNSAT 回餵包含具體拒絕原因與宏展開日誌，讓 LLM 能定位錯誤臂／
  錯誤型別，而不只是「失敗了」。
- **會收斂也會放棄**：`--attempts` 用盡仍不過 ⇒ 回傳 `verdict: UNSAT/ERROR`
  與完整 `attempt_log`（每輪結果 + 回餵內容），**絕不產出未經驗證的代碼**。
- 系統提示（`guardrail_system_prompt`）把 LLM 限定在「自然語言 → .poly」
  單一職責：完整語法表、七種型別限制、典型拒絕原因、正反範例。

## 3. 三個入口（同一契約）

| 入口 | 用法 |
|---|---|
| CLI | `polyrust nl "<需求>" [旗標] [--json]` |
| HTTP | `POST /api/nl`，body = JSON `{"description","provider"?,"model"?,"base_url"?,"api_key"?,"attempts"?}` 或純文字 |
| 程式庫 | `llm::run_guardrail(provider, nl, attempts, do_gen)` → `GuardrailResult` |

### JSON 契約（`--json` / `/api/nl`）

```json
{
  "api_version": "0.1", "mode": "nl", "description": "…", "provider": "…",
  "status": "ok|error", "verdict": "SAT|UNSAT|ERROR",
  "poly": "通過護欄的 .poly 全文（或 null）",
  "checker_msg": "…", "attempts": 3,
  "attempt_log": [ {"n":1,"outcome":"syntax-error","feedback":"…"}, … ],
  "generated": { "code": "…", "file": "output/generated/nl.rs", "rustc_compiles": true },
  "final_reason": "第 3 輪通過三道閘門"
}
```

`outcome ∈ {sat, unsat, syntax-error, unresolved, provider-error}`。

## 4. 實測記錄（2026-09-14）

### 4.1 離線（確定性）

以本地 OpenAI 相容假伺服器（腳本：語法錯 → 型別錯配 → 正確）實測：

```
第 1 輪：✗ 結構/語法拒絕  ↳ 管線錯誤：無法解析的 token Some(Semi)
第 2 輪：✗ 語義拒絕（UNSAT）↳ 形式化管線判定 UNSAT（拒絕）：拒絕：main 體不可定型
第 3 輪：✓ 通過
判定：SAT — 生成 output/generated/nl.rs（rustc 編譯 ✓）
```

伺服器側日誌確認修復回饋確實進入對話史（第 2 輪 `messages` 長度 4、
第 3 輪長度 6，末條為回餵的錯誤訊息）。

單元測試 10 條（`cargo test llm`）：圍欄抽取（優先 ` ```poly `、退路 intent 塊、
無圍欄拒絕）、**多塊拒絕**、修復迴圈（拒絕→修復→通過）、UNSAT 回餵、用盡放棄、
`@intent` 結構閘門、**對抗性語料庫**（拒答／空塊／幻覺語法 `for`+`struct`／
雙可變借用／散文包裹）、JSON 讀取器（含代理對解碼）。

### 4.2 真 API（OpenRouter 免費模型）

經 `https://openrouter.ai/api/v1`（OpenAI 相容）實測三個免費模型（各 262k
context），**8 案例測試組、7 個首輪即過**；唯一失敗案例（「把 `true` 加到 `1`」）
正是護欄應拒絕者——首輪回餵精確原因，最終**不產出任何未驗證代碼**。

| 案例 | 結果 |
|---|---|
| 多參數函式／宏／比較布爾／條件分支／含糊需求 | SAT，首輪，`rustc` ✓ |
| 雙可變引用 | SAT（模型寫成暫引用變體，與真 `rustc` 判定一致） |
| 型別不可能（`1 + true`） | **UNSAT 拒絕** → 修復失敗 → 放棄（零未驗證產出）|
| 提示注入 | 模型忽略注入、系統提示未洩露；輸出仍經完整管線 |

抓到並修復兩個**離線測試抓不到**的真 bug：(1) `temperature` 原以字串送出，
現送 JSON 數字（`J::Float`）；(2) `main` 非 unit 尾表達式過唔到真 `rustc`，
現以語句形式丟棄（`discard_tail`）。完整記錄見
[`docs/evidence/llm-real-api-2026-09-14.md`](evidence/llm-real-api-2026-09-14.md)。

## 5. 邊界（這層不做什麼）

- 不做語義理解：需求是否「合理」由管線判定（UNSAT 即拒絕），護欄不猜意圖。
- 不信任任何 LLM 產出：含「看起来对」的代碼在內，一律過完整管線。
- 目標語言仍是 Mini-Rust 可判定子集（七型別、無循環/結構體）——與
  `docs/POLY_DSL.md` 相同邊界。
