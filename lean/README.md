# polyrust-formal — polyrust 的 Lean 4 形式化

[docs/THEOREMS.md](../docs/THEOREMS.md) 中九條定理證明骨幹的機械化。
**無 Mathlib 依賴**——僅用 Lean 4 core（自包含、可復現、`lake build` 數秒完成）。

## 模組與定理對應

| 模組 | 定理 | 核心結果 |
|---|---|---|
| `Polyrust.ClauseDuality` | **T3(a)** | `clause_duality`：σ ⊨ C ⟺ P_C(σ) = 0（子句–多項式對偶）；`field_poly_bit`：x²−x 刻畫布爾域；`cnf_duality`：CNF 公式 ↔ 多項式組 |
| `Polyrust.UniPoly` | **T8** | `div_linear`（構造性 Horner 餘式定理）、`vanishing_prod_dvd`（互異根 ⇒ vanishing 多項式整除）、`dvd_prod_vanishing`，合成 `qap_duality`：∀j a_j·b_j = c_j ⟺ Z ∣ A·B−C |
| `Polyrust.Squarefree` | **T4** | `standard_implies_squarefree`（標準單項式平方自由——域多項式入基 ⇒ 首項理想含 xᵢ²）、`squarefree_count`（平方自由單項式恰 2ⁿ 個）⇒ 基擴充 ≤ 2ⁿ ⇒ Buchberger 必終止 |
| `Polyrust.Embedding` | **L0** | `L0_mod_faithful`（\|n\| < p=2⁶¹−1 ⇒ 模零 ⟺ 整數零）+ `eval_abs_bound`（小係數多項式 0/1 點求值 ≤ 2²⁸）⇒ `L0_eval_faithful`：𝔽_p 判定 ⟺ ℤ 判定 |
| `Polyrust.MicroInstance` | **T1/T2/T6/T7** | 三個微型系統（加法規則、上下文矛盾、宏 exists-arm 選臂）的 2^k 全枚舉：σ_D 是根（T1）、根解碼推導（T2）、UNSAT ⟺ 不可定型（T6）、選臂可解/錯臂矛盾（T7） |

## 構建與驗證

```bash
lake build        # 無外部依賴；需要 Lean 4.33+
```

`Polyrust.lean` 為根模組，`import Polyrust` 即可得全部定理。

### 兩項審計的差別（重要）

* `Audit.lean` 檢查的是**手列清單**（84 條）——若新增定理忘了加進清單，缺口不會被發現。
* `AuditAll.lean` 直接掃描環境裡 `Polyrust.*` 的**每一條宣告**（1115 條），底層用 Lean 內建的
  `Lean.collectAxioms`（即 `#print axioms` 背後的同一個函式），因此新增定理也會被自動覆蓋。
  末行輸出 `AUDIT_RESULT=CLEAN` / `DIRTY`，CI 以此作為硬閘門。

已做過反向驗證：注入一條 `theorem ... := sorry` 後，`AuditAll.lean` 報 `DIRTY` 並點名該宣告，
而 `Audit.lean` 完全沒發現（仍報 0 個 `sorryAx`）。

```bash
lake env lean Audit.lean      # 逐定理 #print axioms（84 條手列清單）
lake env lean AuditAll.lean   # 全環境掃描（Polyrust.* 每一條宣告）
bash scripts/lean-audit.sh    # 一次跑完：來源掃描 + 建置 + 兩項審計
```

## 設計說明

- **求值層面陳述**：多項式以升冪係數列表表示（與 Rust 版 `qap.rs` 的
  `UniPoly` 同構），定理陳述在「求值函數」層面（∀t, p(t) = q(t)·…），
  避免表示不唯一帶來的雜訊；`div_linear` 的見證 q 是構造性的。
- **單項式 = 指數向量函數**（`Nat → ℕ`）：整除/平方自由是逐點性質，
  無需 List 索引引理；計數經位串枚舉器 `allBits`（flatMap 構造）。
- **嵌入引理的界**：|c| ≤ 2¹⁶、項數 ≤ 2¹²、0/1 點求值 ⇒ \|f(σ)\| ≤ 2²⁸ < p，
  與 docs/THEOREMS.md §2 的參數一致（Lean 側取具體安全值 65536/4096）。
