<!-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial) -->
# v2/v3 JSON 契約 v0.3（bounded 標記全鏈路）

DEV_PLAN_V03 **P0-C5** 的落地文本。自 v0.3 起，`check-v2`（`api_version: "0.3"`）與
`check-v3`（`api_version: "3.0"`）的 JSON 判決攜帶 **bounded 標記**，且三值判決
（SAT / UNSAT / UNKNOWN）對**默認 fuel=3 路徑**同樣生效。

## 1. `bounded` 欄位

凡判決係喺 **fuel 有界展開模型**內給出，JSON 必帶：

```json
"bounded": { "kind": "loop_fuel", "fuel": 3, "insufficient": false }
```

| 欄位 | 語義 |
|---|---|
| `kind` | 有界種類。現時只有 `"loop_fuel"`（@fuel / 默認 3 展開）；預留 `"unroll_depth"` 等，新增時同步本文件與測試。 |
| `fuel` | 有效界度（整數）：顯式 `@fuel: N` 值，否則默認 `3`。 |
| `insufficient` | `true` ⟺ 估算迭代需求超界（見 §2），判決層對應 UNKNOWN。 |

無 fuel 有界語義時（源無 `loop`/`while`/`for` 關鍵字且無 `@fuel`），`bounded: null`。

觸發條件與約束生成端（`gen_loop_fuel_constraints` 分支）**嚴格一致**：
顯式 `@fuel` 優先，否則源文本含迴圈關鍵字即默認 3（`pipeline_v2::effective_fuel`）。

## 2. 三值判決與默認路徑

| 情形 | `verdict` | `bounded` |
|---|---|---|
| UNSAT（約束系統衝突 / lifetime 環） | `UNSAT` | 有界照標（UNSAT 同樣只在有界模型內成立） |
| fuel 估算不足（超界）且無 `@invariant` | `UNKNOWN` | `insufficient: true` |
| 有界模型內 SAT（界內） | `SAT` | `insufficient: false` |
| 無迴圈 | `SAT`/`UNSAT` | `null` |

**P0-C5 行為變更（兌現 DEV_PLAN_V03 P1-U2 承諾）**：v0.2 的
`estimate_fuel_insufficiency` 只檢查**顯式** `@fuel`；v0.3 起默認 fuel=3 路徑
照樣納入誠實三值——`while x < 100 { x = x + 1; }` 無註解時由 (誤導性) `SAT`
改報 `UNKNOWN`、`bounded:{kind:"loop_fuel",fuel:3,insufficient:true}`。

保守原則不變：估算器只在**可確認不足**時回 UNKNOWN（解析 `while X < N` +
`X += D` 形狀；`loop {}`/`for` 形態不估算，僅標 `bounded`），寧漏報不誤報。
`@invariant` 出現時豁免（既有口徑）。

## 3. CI 閘門語義（fail-closed）

消費本契約的閘門（CI、審批流、商業報告生成）**必須**：

1. `verdict: "UNKNOWN"` 一律視為**非通過**——不得降格為 SAT 或 UNSAT，
   不得計入「已驗證」涵蓋率分母。
2. `verdict: "SAT"` 且 `bounded != null` 時，只在 `bounded.kind`/`fuel`
   所描述的有界模型內申報通過（報告須附該標記）。
3. `insufficient: true` 的樣本若要晉升 SAT，須以足額 `@fuel` 或
   `@invariant` 重新判決——唔准人手改判。

紅線對齊：「任何判定不得偽造證書或偽造判定」。bounded 標記缺失（舊
`api_version`）視為契約違規，閘門拒收。

## 4. 版本記錄

- `check`（v1）：契約 `0.1` 維持不變；統計層另有 `lrat_verified`/`lrat_steps`
  （P0-C2 LRAT 自證 UNSAT 證書）。
- `check-v2`：`0.2` → **`0.3`**（新增 `bounded`；`unknown_reason` 保留兼容）。
- `check-v3`：`3.0` 系列不變；自 v0.3 起新增 `bounded` 穿透欄位與三值判決對齊。
- `llbc-check`（Charon/LLBC 鏈）：`0.3`，bounded/abstract marker 鏈原生攜帶
  （見 C2_LOWERING.md）。

## 5. 驗收測試

- `pipeline_v2::borrowck_v03_tests::p0c5_effective_fuel_triggers`
- `pipeline_v2::borrowck_v03_tests::p0c5_default_fuel_folded_into_unknown`
- `driver::tests::p0c5_bounded_json_contract`（四案橫掃：顯式超界／默認超界／
  默認界內／無迴圈）
- `driver::tests::p0c5_bounded_json_contract_v3_passthrough`（v3 穿透 + 三值對齊）
