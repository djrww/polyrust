# Lean 純構造宣告逐條清冊（1769 條）

> 判據與 `AuditAll.lean` 同源：`Lean.collectAxioms`（即 `#print axioms` 背後同一函式）對 `Polyrust.*` 每一條定理／定義計算公理依賴，**依賴集合為空**者列入本表。零 `sorry`、零自訂公理；依賴 `propext`／`Classical.choice`／`Quot.sound` 者不算純構造，未列入。

**構成**：1769 條 = 641 條定理 ＋ 1128 條定義；其中手寫 602 條、機器衍生 1167 條（歸納型自動產生的一致性／遞迴／判定引理）。

## 模組總覽

| 模組 | 條數 | 手寫 | 機器衍生 | 職責一句話 |
|---|---:|---:|---:|---|
| `Polyrust.T9EndToEnd` | 213 | 55 | 158 | 迷你語言型別檢查器；約束編碼的可靠性/完備性/判定等價 |
| `Polyrust.TypeUniverse7PlusI` | 147 | 78 | 69 | 7 基底 + i 擴展（17 種完整宇宙）；one-hot 分解；宇宙單調性 |
| `Polyrust.MacroExpansion` | 109 | 21 | 88 | 模板語法/上下文/代入；展開是同態 |
| `Polyrust.OpAbstraction` | 107 | 27 | 80 | 運算子規則抽象：二元運算子抽象為 BinSpec |
| `Polyrust.SumReduction` | 106 | 21 | 85 | 和型歸約：inl/inr/和型檢查與可定型性歸約為變體檢查 |
| `Polyrust.TraitImpl` | 100 | 27 | 73 | trait/impl 解析、方法歸約、存在量化 |
| `Polyrust.ProductReduction` | 97 | 17 | 80 | 積型歸約：pair/積型檢查與可定型性歸約為逐欄位檢查 |
| `Polyrust.LifetimeRegion` | 93 | 24 | 69 | Lifetime/'static、outlives 圖、無環鐵律、NLL 區間重疊 |
| `Polyrust.MatchDecisionTree` | 93 | 29 | 64 | Pat/MatchArm/DecisionTree；決策樹深度 ≤ 臂數；編譯保語義 |
| `Polyrust.BorrowOwnership` | 90 | 41 | 49 | 借用存活區間與衝突；borrow_sat_iff_clean |
| `Polyrust.T9Generalized` | 83 | 27 | 56 | 型別宇宙泛化：任意可枚舉宇宙，one-hot 用 List.sum |
| `Polyrust.StdlibEncoding` | 66 | 22 | 44 | Vec/String/HashMap 的多項式編碼 |
| `Polyrust.UnsafeContext` | 66 | 27 | 39 | 裸指針解析、unsafe 閘控、Pure/No-IO 邊界 |
| `Polyrust.AsyncStateMachine` | 65 | 15 | 50 | Future 狀態機、輪詢約束、one-hot 狀態 |
| `Polyrust.ModuleFlatten` | 60 | 20 | 40 | 模組樹展平為線性約束；路徑解析 |
| `Polyrust.ProductSum` | 50 | 17 | 33 | 積型與和型的混合歸約；ProductConstraint/SumConstraint |
| `Polyrust.ClauseDuality` | 32 | 13 | 19 | 子句滿足 ⟺ 子句多項式歸零；域多項式 x²−x 刻畫 {0,1}ⁿ；CNF 對偶 |
| `Polyrust.UniPoly` | 26 | 8 | 18 | 求值環同態；線性餘式；互異根 vanishing ⇒ ∏(X−tᵢ) 整除；QAP 對偶 |
| `Polyrust.LoopContract` | 21 | 6 | 15 | 循環不變量、歸納契約、fuel 有界展開 |
| `Polyrust.Monomial` | 19 | 17 | 2 | 單項式指數向量、整除、lcm/quot、支撐、首項；純組合層 |
| `Polyrust.Minor` | 17 | 16 | 1 | 次要（支撐性）：位元算術、Lit 支撐、MonoExp 整除、supportLe、List 求和 |
| `Polyrust.Squarefree` | 17 | 7 | 10 | 標準單項式 ⇒ 平方自由；平方自由 ↔ 位串雙射；恰 2ⁿ 個 ⇒ Buchberger 終止 |
| `Polyrust.T6Certificate` | 17 | 8 | 9 | 求值同態、無公共零點證書、布爾 Lagrange 插值 |
| `Polyrust.ClauseAlgebra` | 13 | 11 | 2 | 消解恆等式；學習子句保留模型集/零集；UNSAT ⟺ 無零點多項式 |
| `Polyrust.IronLaw` | 13 | 13 | 0 | 新增（鐵律核心）：one-hot 排他、field 多項式、借用衝突互斥、lifetime 無環 |
| `Polyrust.MicroInstance` | 13 | 9 | 4 | 加法規則、上下文矛盾、宏選臂三個微型系統的全枚舉 |
| `Polyrust.SPoly` | 8 | 6 | 2 | S-多項式：單項式乘法、S-多項式 ∈ 生成理想、Buchberger 判準 |
| `Polyrust.WatchMove` | 7 | 1 | 6 | 監視文字交換移動保持子句語義；衝突偵測健全性 |
| `Polyrust.Canonical` | 7 | 6 | 1 | 既約/全簡化標準形；若簡化 Gröbner 基存在則唯一 |
| `Polyrust.Derived` | 5 | 5 | 0 | 衍生（由核心推出）：pair/sum 可定型推論、pairBitSum 乘積、borrow 1∈理想 |
| `Polyrust.Completion` | 3 | 2 | 1 | 補全（雙向完備）：clauseSat↔polyZero、borrow_sat↔clean、typable↔root 雙向 |
| `Polyrust.BoolNullstellensatz` | 2 | 2 | 0 | 布爾 Nullstellensatz：每點有模 p 可逆系統元素 ⟹ 多項式函數乘子組合成 1 |
| `Polyrust.Tactics` | 2 | 2 | 0 | int_ring：無 Mathlib 的整數多項式歸一化宏 |
| `Polyrust.Embedding` | 2 | 2 | 0 | 𝔽_p（p=2⁶¹−1）嵌入保真 |

## `Polyrust.T9EndToEnd`（213 條）

職責：迷你語言型別檢查器；約束編碼的可靠性/完備性/判定等價

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1 | `Polyrust.Expr.add.elim` | 手寫 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 1 → ((a a_1 : Polyrust.Expr) → motive (a.add a_1)) → motive t |
| 2 | `Polyrust.Expr.add.inj` | 手寫 | 定理 | | ∀ {a a_1 a_2 a_3 : Polyrust.Expr}, a.add a_1 = a_2.add a_3 → a = a_2 ∧ a_1 = a_3 |
| 3 | `Polyrust.Expr.add.sizeOf_spec` | 手寫 | 定理 | | ∀ (a a_1 : Polyrust.Expr), sizeOf (a.add a_1) = 1 + sizeOf a + sizeOf a_1 |
| 4 | `Polyrust.Expr.eqb.elim` | 手寫 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 2 → ((a a_1 : Polyrust.Expr) → motive (a.eqb a_1)) → motive t |
| 5 | `Polyrust.Expr.eqb.inj` | 手寫 | 定理 | | ∀ {a a_1 a_2 a_3 : Polyrust.Expr}, a.eqb a_1 = a_2.eqb a_3 → a = a_2 ∧ a_1 = a_3 |
| 6 | `Polyrust.Expr.eqb.sizeOf_spec` | 手寫 | 定理 | | ∀ (a a_1 : Polyrust.Expr), sizeOf (a.eqb a_1) = 1 + sizeOf a + sizeOf a_1 |
| 7 | `Polyrust.Expr.ite.elim` | 手寫 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 3 → ((a a_1 a_2 : Polyrust.Expr) → motive (a.ite a_1 a_2)) → motive t |
| 8 | `Polyrust.Expr.ite.inj` | 手寫 | 定理 | | ∀ {a a_1 a_2 a_3 a_4 a_5 : Polyrust.Expr}, a.ite a_1 a_2 = a_3.ite a_4 a_5 → a = a_3 ∧ a_1 = a_4 ∧ a_2 = a_5 |
| 9 | `Polyrust.Expr.ite.sizeOf_spec` | 手寫 | 定理 | | ∀ (a a_1 a_2 : Polyrust.Expr), sizeOf (a.ite a_1 a_2) = 1 + sizeOf a + sizeOf a_1 + sizeOf a_2 |
| 10 | `Polyrust.Expr.num.elim` | 手寫 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 0 → ((a : Int) → motive (Polyrust.Expr.num a)) → motive t |
| 11 | `Polyrust.Expr.num.inj` | 手寫 | 定理 | | ∀ {a a_1 : Int}, Polyrust.Expr.num a = Polyrust.Expr.num a_1 → a = a_1 |
| 12 | `Polyrust.Expr.num.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Int), sizeOf (Polyrust.Expr.num a) = 1 + sizeOf a |
| 13 | `Polyrust.IsMonoAt` | 手寫 | 定義 | | Polyrust.Sigma → Polyrust.Expr → Prop |
| 14 | `Polyrust.IsRoot` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Sigma → Prop |
| 15 | `Polyrust.Sigma` | 手寫 | 定義 | | Type |
| 16 | `Polyrust.Tok.add.elim` | 手寫 | 定義 | | {motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 2 → motive Polyrust.Tok.add → motive t |
| 17 | `Polyrust.Tok.add.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Tok.add = 1 |
| 18 | `Polyrust.Tok.eqb.elim` | 手寫 | 定義 | | {motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 3 → motive Polyrust.Tok.eqb → motive t |
| 19 | `Polyrust.Tok.eqb.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Tok.eqb = 1 |
| 20 | `Polyrust.Tok.ite.elim` | 手寫 | 定義 | | {motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 4 → motive Polyrust.Tok.ite → motive t |
| 21 | `Polyrust.Tok.ite.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Tok.ite = 1 |
| 22 | `Polyrust.Tok.num.elim` | 手寫 | 定義 | | {motive : Polyrust.Tok → Sort u} →   (t : Polyrust.Tok) → t.ctorIdx = 1 → ((a : Int) → motive (Polyrust.Tok.num a)) → motive t |
| 23 | `Polyrust.Tok.num.inj` | 手寫 | 定理 | | ∀ {a a_1 : Int}, Polyrust.Tok.num a = Polyrust.Tok.num a_1 → a = a_1 |
| 24 | `Polyrust.Tok.num.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Int), sizeOf (Polyrust.Tok.num a) = 1 + sizeOf a |
| 25 | `Polyrust.Tok.unit.elim` | 手寫 | 定義 | | {motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 0 → motive Polyrust.Tok.unit → motive t |
| 26 | `Polyrust.Tok.unit.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Tok.unit = 1 |
| 27 | `Polyrust.Ty.boolean.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → t.ctorIdx = 1 → motive Polyrust.Ty.boolean → motive t |
| 28 | `Polyrust.Ty.boolean.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Ty.boolean = 1 |
| 29 | `Polyrust.Ty.i32.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → t.ctorIdx = 0 → motive Polyrust.Ty.i32 → motive t |
| 30 | `Polyrust.Ty.i32.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Ty.i32 = 1 |
| 31 | `Polyrust.Ty.ofNat` | 手寫 | 定義 | | Nat → Polyrust.Ty |
| 32 | `Polyrust.Ty.ofNat_ctorIdx` | 手寫 | 定理 | | ∀ (x : Polyrust.Ty), Polyrust.Ty.ofNat x.ctorIdx = x |
| 33 | `Polyrust.Ty.toCtorIdx` | 手寫 | 定義 | | Polyrust.Ty → Nat |
| 34 | `Polyrust.bit_and` | 手寫 | 定理 | | ∀ (a b : Bool), Polyrust.bit (a && b) = Polyrust.bit a * Polyrust.bit b |
| 35 | `Polyrust.cAddB` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 36 | `Polyrust.cAddH` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 37 | `Polyrust.cAddI` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 38 | `Polyrust.cEqbB` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 39 | `Polyrust.cEqbH` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 40 | `Polyrust.cEqbI` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 41 | `Polyrust.cIteB` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 42 | `Polyrust.cIteH` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 43 | `Polyrust.cIteI` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int |
| 44 | `Polyrust.cNumB` | 手寫 | 定義 | | Int → Polyrust.Sigma → Int |
| 45 | `Polyrust.cNumH` | 手寫 | 定義 | | Int → Polyrust.Sigma → Int |
| 46 | `Polyrust.cNumI` | 手寫 | 定義 | | Int → Polyrust.Sigma → Int |
| 47 | `Polyrust.gen` | 手寫 | 定義 | | Polyrust.Expr → List Polyrust.Tok |
| 48 | `Polyrust.genC` | 手寫 | 定義 | | Polyrust.Expr → List (Polyrust.Sigma → Int) |
| 49 | `Polyrust.genC_num_oneHot` | 手寫 | 定理 | | ∀ (n : Int), Polyrust.cNumH n ∈ Polyrust.genC (Polyrust.Expr.num n) |
| 50 | `Polyrust.oneHot` | 手寫 | 定義 | | Polyrust.Sigma → Polyrust.Expr → Int |
| 51 | `Polyrust.oneHotC` | 手寫 | 定義 | | Polyrust.Expr → Polyrust.Sigma → Int |
| 52 | `Polyrust.parse` | 手寫 | 定義 | | List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok) |
| 53 | `Polyrust.parseFuel` | 手寫 | 定義 | | Nat → List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok) |
| 54 | `Polyrust.sizeT` | 手寫 | 定義 | | Polyrust.Expr → Nat |
| 55 | `Polyrust.tb` | 手寫 | 定義 | | Polyrust.Sigma → Polyrust.Expr → Polyrust.Ty → Int |
| 56 | `Polyrust.Expr._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Expr → Nat |
| 57 | `Polyrust.Expr._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Expr |
| 58 | `Polyrust.Expr.add.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a_1 a' a'_1 : Polyrust.Expr} → a.add a_1 = a'.add a'_1 → (a = a' → a_1 = a'_1 → P) → P |
| 59 | `Polyrust.Expr.below` | 歸納型衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} → Polyrust.Expr → Sort (max 1 u) |
| 60 | `Polyrust.Expr.brecOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → ((t : Polyrust.Expr) → Polyrust.Expr.below t → motive t) → motive t |
| 61 | `Polyrust.Expr.brecOn.eq` | 衍生 | 定理 | | ∀ {motive : Polyrust.Expr → Sort u} (t : Polyrust.Expr) (F_1 : (t : Polyrust.Expr) → Polyrust.Expr.below t → motive t),   Polyrust.Expr.brecOn t F_1 = |
| 62 | `Polyrust.Expr.brecOn.go` | 歸納型衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → ((t : Polyrust.Expr) → Polyrust.Expr.below t → motive t) → motive t ×' Polyrust.Expr.below |
| 63 | `Polyrust.Expr.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) →     ((a : Int) → motive (Polyrust.Expr.num a)) →       ((a a_1 : Polyrust.Expr) → motive ( |
| 64 | `Polyrust.Expr.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Expr) → ctorIdx = t.ctorIdx → Polyrust.Expr.ctorElimType ctorIdx → motive t |
| 65 | `Polyrust.Expr.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} → Nat → Sort (max 1 u) |
| 66 | `Polyrust.Expr.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Expr → Nat |
| 67 | `Polyrust.Expr.eqb.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a_1 a' a'_1 : Polyrust.Expr} → a.eqb a_1 = a'.eqb a'_1 → (a = a' → a_1 = a'_1 → P) → P |
| 68 | `Polyrust.Expr.ite.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {a a_1 a_2 a' a'_1 a'_2 : Polyrust.Expr} →     a.ite a_1 a_2 = a'.ite a'_1 a'_2 → (a = a' → a_1 = a'_1 → a_2 = a'_2 → P) → P |
| 69 | `Polyrust.Expr.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Expr} → t = t' → Polyrust.Expr.noConfusionType P t t' |
| 70 | `Polyrust.Expr.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Expr → Polyrust.Expr → Sort u |
| 71 | `Polyrust.Expr.num.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Int} → Polyrust.Expr.num a = Polyrust.Expr.num a' → (a = a' → P) → P |
| 72 | `Polyrust.Expr.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) →     ((a : Int) → motive (Polyrust.Expr.num a)) →       ((a a_1 : Polyrust.Expr) → motive a |
| 73 | `Polyrust.Occurs.below.casesOn` | 歸納型衍生 | 定義 | | ∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop}   {motive_1 : {a_1 : Polyrust.Expr} → (t : Polyrust.Occurs a a_1 |
| 74 | `Polyrust.Occurs.brecOn` | 歸納型衍生 | 定理 | | ∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop} {a_1 : Polyrust.Expr}   (t : Polyrust.Occurs a a_1),   (∀ (a_2 : |
| 75 | `Polyrust.Occurs.casesOn` | 歸納型衍生 | 定義 | | ∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop} {a_1 : Polyrust.Expr}   (t : Polyrust.Occurs a a_1),   motive a  |
| 76 | `Polyrust.Occurs.recOn` | 歸納型衍生 | 定義 | | ∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop} {a_1 : Polyrust.Expr}   (t : Polyrust.Occurs a a_1),   motive a  |
| 77 | `Polyrust.Tok._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Tok → Nat |
| 78 | `Polyrust.Tok._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Tok |
| 79 | `Polyrust.Tok.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Tok → Sort u} →   (t : Polyrust.Tok) →     motive Polyrust.Tok.unit →       ((a : Int) → motive (Polyrust.Tok.num a)) →         mot |
| 80 | `Polyrust.Tok.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.Tok → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Tok) → ctorIdx = t.ctorIdx → Polyrust.Tok.ctorElimType ctorIdx → motive t |
| 81 | `Polyrust.Tok.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.Tok → Sort u} → Nat → Sort (max 1 u) |
| 82 | `Polyrust.Tok.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Tok → Nat |
| 83 | `Polyrust.Tok.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Tok} → t = t' → Polyrust.Tok.noConfusionType P t t' |
| 84 | `Polyrust.Tok.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Tok → Polyrust.Tok → Sort u |
| 85 | `Polyrust.Tok.num.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Int} → Polyrust.Tok.num a = Polyrust.Tok.num a' → (a = a' → P) → P |
| 86 | `Polyrust.Tok.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Tok → Sort u} →   (t : Polyrust.Tok) →     motive Polyrust.Tok.unit →       ((a : Int) → motive (Polyrust.Tok.num a)) →         mot |
| 87 | `Polyrust.Ty._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Ty → Nat |
| 88 | `Polyrust.Ty._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Ty |
| 89 | `Polyrust.Ty.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → motive Polyrust.Ty.i32 → motive Polyrust.Ty.boolean → motive t |
| 90 | `Polyrust.Ty.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Ty) → ctorIdx = t.ctorIdx → Polyrust.Ty.ctorElimType ctorIdx → motive t |
| 91 | `Polyrust.Ty.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.Ty → Sort u} → Nat → Sort (max 1 u) |
| 92 | `Polyrust.Ty.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Ty → Nat |
| 93 | `Polyrust.Ty.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort v✝} → {x y : Polyrust.Ty} → x = y → Polyrust.Ty.noConfusionType P x y |
| 94 | `Polyrust.Ty.noConfusionType` | 衍生 | 定義 | | Sort v✝ → Polyrust.Ty → Polyrust.Ty → Sort v✝ |
| 95 | `Polyrust.Ty.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → motive Polyrust.Ty.i32 → motive Polyrust.Ty.boolean → motive t |
| 96 | `Polyrust.cAddB.eq_1` | 等式引理／助手 | 定理 | | ∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma), Polyrust.cAddB a b σ = Polyrust.tb σ (a.add b) Polyrust.Ty.boolean |
| 97 | `Polyrust.cAddH.eq_1` | 等式引理／助手 | 定理 | | ∀ (a b : Polyrust.Expr), Polyrust.cAddH a b = Polyrust.oneHotC (a.add b) |
| 98 | `Polyrust.cAddI.eq_1` | 等式引理／助手 | 定理 | | ∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cAddI a b σ =     Polyrust.tb σ (a.add b) Polyrust.Ty.i32 - Polyrust.tb σ a Polyrust.Ty.i32 * |
| 99 | `Polyrust.cEqbB.eq_1` | 等式引理／助手 | 定理 | | ∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cEqbB a b σ =     Polyrust.tb σ (a.eqb b) Polyrust.Ty.boolean - Polyrust.tb σ a Polyrust.Ty.i |
| 100 | `Polyrust.cEqbH.eq_1` | 等式引理／助手 | 定理 | | ∀ (a b : Polyrust.Expr), Polyrust.cEqbH a b = Polyrust.oneHotC (a.eqb b) |
| 101 | `Polyrust.cEqbI.eq_1` | 等式引理／助手 | 定理 | | ∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma), Polyrust.cEqbI a b σ = Polyrust.tb σ (a.eqb b) Polyrust.Ty.i32 |
| 102 | `Polyrust.cIteB.eq_1` | 等式引理／助手 | 定理 | | ∀ (c t f : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cIteB c t f σ =     Polyrust.tb σ (c.ite t f) Polyrust.Ty.boolean -       Polyrust.tb σ c P |
| 103 | `Polyrust.cIteH.eq_1` | 等式引理／助手 | 定理 | | ∀ (c t f : Polyrust.Expr), Polyrust.cIteH c t f = Polyrust.oneHotC (c.ite t f) |
| 104 | `Polyrust.cIteI.eq_1` | 等式引理／助手 | 定理 | | ∀ (c t f : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cIteI c t f σ =     Polyrust.tb σ (c.ite t f) Polyrust.Ty.i32 -       Polyrust.tb σ c Polyr |
| 105 | `Polyrust.cNumB.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Int) (σ : Polyrust.Sigma), Polyrust.cNumB n σ = Polyrust.tb σ (Polyrust.Expr.num n) Polyrust.Ty.boolean |
| 106 | `Polyrust.cNumH.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Int), Polyrust.cNumH n = Polyrust.oneHotC (Polyrust.Expr.num n) |
| 107 | `Polyrust.cNumI.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Int) (σ : Polyrust.Sigma), Polyrust.cNumI n σ = Polyrust.tb σ (Polyrust.Expr.num n) Polyrust.Ty.i32 - 1 |
| 108 | `Polyrust.gen._f` | 等式引理／助手 | 定義 | | (x : Polyrust.Expr) → Polyrust.Expr.below x → List Polyrust.Tok |
| 109 | `Polyrust.gen._sunfold` | 等式引理／助手 | 定義 | | Polyrust.Expr → List Polyrust.Tok |
| 110 | `Polyrust.gen._unsafe_rec` | 等式引理／助手 | 定義 | | Polyrust.Expr → List Polyrust.Tok |
| 111 | `Polyrust.gen.eq_1` | 等式引理／助手 | 定理 | | ∀ (a : Int), Polyrust.gen (Polyrust.Expr.num a) = [Polyrust.Tok.num a] |
| 112 | `Polyrust.gen.eq_2` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.Expr), Polyrust.gen (a.add a_1) = Polyrust.Tok.add :: (Polyrust.gen a ++ Polyrust.gen a_1) |
| 113 | `Polyrust.gen.eq_3` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.Expr), Polyrust.gen (a.eqb a_1) = Polyrust.Tok.eqb :: (Polyrust.gen a ++ Polyrust.gen a_1) |
| 114 | `Polyrust.gen.eq_4` | 等式引理／助手 | 定理 | | ∀ (a a_1 a_2 : Polyrust.Expr),   Polyrust.gen (a.ite a_1 a_2) = Polyrust.Tok.ite :: (Polyrust.gen a ++ Polyrust.gen a_1 ++ Polyrust.gen a_2) |
| 115 | `Polyrust.gen.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.Expr),   Polyrust.gen x =     match x with     \| Polyrust.Expr.num n => [Polyrust.Tok.num n]     \| a.add b => Polyrust.Tok.add :: (Pol |
| 116 | `Polyrust.genC._f` | 等式引理／助手 | 定義 | | (x : Polyrust.Expr) → Polyrust.Expr.below x → List (Polyrust.Sigma → Int) |
| 117 | `Polyrust.genC._sunfold` | 等式引理／助手 | 定義 | | Polyrust.Expr → List (Polyrust.Sigma → Int) |
| 118 | `Polyrust.genC._unsafe_rec` | 等式引理／助手 | 定義 | | Polyrust.Expr → List (Polyrust.Sigma → Int) |
| 119 | `Polyrust.genC.eq_1` | 等式引理／助手 | 定理 | | ∀ (a : Int), Polyrust.genC (Polyrust.Expr.num a) = [Polyrust.cNumI a, Polyrust.cNumB a, Polyrust.cNumH a] |
| 120 | `Polyrust.genC.eq_2` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.Expr),   Polyrust.genC (a.add a_1) =     Polyrust.genC a ++ Polyrust.genC a_1 ++ [Polyrust.cAddI a a_1, Polyrust.cAddB a a_1, Poly |
| 121 | `Polyrust.genC.eq_3` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.Expr),   Polyrust.genC (a.eqb a_1) =     Polyrust.genC a ++ Polyrust.genC a_1 ++ [Polyrust.cEqbB a a_1, Polyrust.cEqbI a a_1, Poly |
| 122 | `Polyrust.genC.eq_4` | 等式引理／助手 | 定理 | | ∀ (a a_1 a_2 : Polyrust.Expr),   Polyrust.genC (a.ite a_1 a_2) =     Polyrust.genC a ++ Polyrust.genC a_1 ++ Polyrust.genC a_2 ++       [Polyrust.cIte |
| 123 | `Polyrust.genC.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.Expr),   Polyrust.genC x =     match x with     \| Polyrust.Expr.num n => [Polyrust.cNumI n, Polyrust.cNumB n, Polyrust.cNumH n]     \|  |
| 124 | `Polyrust.instDecidableEqExpr` | 實例衍生 | 定義 | | DecidableEq Polyrust.Expr |
| 125 | `Polyrust.instDecidableEqExpr.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Expr) → Decidable (x = x_1) |
| 126 | `Polyrust.instDecidableEqExpr.decEq._f` | 實例衍生 | 定義 | | (x : Polyrust.Expr) →   Polyrust.Expr.below (motive := fun x => (x_1 : Polyrust.Expr) → Decidable (x = x_1)) x →     (x_1 : Polyrust.Expr) → Decidable |
| 127 | `Polyrust.instDecidableEqExpr.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Expr.num a = Polyrust.Expr.num a |
| 128 | `Polyrust.instDecidableEqExpr.decEq._proof_10` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 : Polyrust.Expr), a.add a_1 = a_2.eqb a_3 → False |
| 129 | `Polyrust.instDecidableEqExpr.decEq._proof_11` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.add a_1 = a_2.ite a_3 a_4 → False |
| 130 | `Polyrust.instDecidableEqExpr.decEq._proof_12` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.Expr) (a_2 : Int), a.eqb a_1 = Polyrust.Expr.num a_2 → False |
| 131 | `Polyrust.instDecidableEqExpr.decEq._proof_13` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 : Polyrust.Expr), a.eqb a_1 = a_2.add a_3 → False |
| 132 | `Polyrust.instDecidableEqExpr.decEq._proof_14` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.Expr), a.eqb a_1 = a.eqb a_1 |
| 133 | `Polyrust.instDecidableEqExpr.decEq._proof_15` | 實例衍生 | 定理 | | ∀ (a a_1 b : Polyrust.Expr), ¬a_1 = b → a.eqb a_1 = a.eqb b → False |
| 134 | `Polyrust.instDecidableEqExpr.decEq._proof_16` | 實例衍生 | 定理 | | ∀ (a a_1 b b_1 : Polyrust.Expr), ¬a = b → a.eqb a_1 = b.eqb b_1 → False |
| 135 | `Polyrust.instDecidableEqExpr.decEq._proof_17` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.eqb a_1 = a_2.ite a_3 a_4 → False |
| 136 | `Polyrust.instDecidableEqExpr.decEq._proof_18` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 : Polyrust.Expr) (a_3 : Int), a.ite a_1 a_2 = Polyrust.Expr.num a_3 → False |
| 137 | `Polyrust.instDecidableEqExpr.decEq._proof_19` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.ite a_1 a_2 = a_3.add a_4 → False |
| 138 | `Polyrust.instDecidableEqExpr.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a b : Int), ¬a = b → Polyrust.Expr.num a = Polyrust.Expr.num b → False |
| 139 | `Polyrust.instDecidableEqExpr.decEq._proof_20` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.ite a_1 a_2 = a_3.eqb a_4 → False |
| 140 | `Polyrust.instDecidableEqExpr.decEq._proof_21` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 : Polyrust.Expr), a.ite a_1 a_2 = a.ite a_1 a_2 |
| 141 | `Polyrust.instDecidableEqExpr.decEq._proof_22` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b : Polyrust.Expr), ¬a_2 = b → a.ite a_1 a_2 = a.ite a_1 b → False |
| 142 | `Polyrust.instDecidableEqExpr.decEq._proof_23` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b b_1 : Polyrust.Expr), ¬a_1 = b → a.ite a_1 a_2 = a.ite b b_1 → False |
| 143 | `Polyrust.instDecidableEqExpr.decEq._proof_24` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b b_1 b_2 : Polyrust.Expr), ¬a = b → a.ite a_1 a_2 = b.ite b_1 b_2 → False |
| 144 | `Polyrust.instDecidableEqExpr.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 a_2 : Polyrust.Expr), Polyrust.Expr.num a = a_1.add a_2 → False |
| 145 | `Polyrust.instDecidableEqExpr.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 a_2 : Polyrust.Expr), Polyrust.Expr.num a = a_1.eqb a_2 → False |
| 146 | `Polyrust.instDecidableEqExpr.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 a_2 a_3 : Polyrust.Expr), Polyrust.Expr.num a = a_1.ite a_2 a_3 → False |
| 147 | `Polyrust.instDecidableEqExpr.decEq._proof_6` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.Expr) (a_2 : Int), a.add a_1 = Polyrust.Expr.num a_2 → False |
| 148 | `Polyrust.instDecidableEqExpr.decEq._proof_7` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.Expr), a.add a_1 = a.add a_1 |
| 149 | `Polyrust.instDecidableEqExpr.decEq._proof_8` | 實例衍生 | 定理 | | ∀ (a a_1 b : Polyrust.Expr), ¬a_1 = b → a.add a_1 = a.add b → False |
| 150 | `Polyrust.instDecidableEqExpr.decEq._proof_9` | 實例衍生 | 定理 | | ∀ (a a_1 b b_1 : Polyrust.Expr), ¬a = b → a.add a_1 = b.add b_1 → False |
| 151 | `Polyrust.instDecidableEqExpr.decEq._sunfold` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Expr) → Decidable (x = x_1) |
| 152 | `Polyrust.instDecidableEqExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Expr) → Decidable (x = x_1) |
| 153 | `Polyrust.instDecidableEqExpr.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Expr → Polyrust.Expr → Sort u_1) →   (x x_1 : Polyrust.Expr) →     ((a b : Int) → motive (Polyrust.Expr.num a) (Polyrust.Expr.num b |
| 154 | `Polyrust.instDecidableEqTok` | 實例衍生 | 定義 | | DecidableEq Polyrust.Tok |
| 155 | `Polyrust.instDecidableEqTok.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Tok) → Decidable (x = x_1) |
| 156 | `Polyrust.instDecidableEqTok.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.unit = Polyrust.Tok.num a → False |
| 157 | `Polyrust.instDecidableEqTok.decEq._proof_10` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.ite → False |
| 158 | `Polyrust.instDecidableEqTok.decEq._proof_11` | 實例衍生 | 定理 | | Polyrust.Tok.add = Polyrust.Tok.unit → False |
| 159 | `Polyrust.instDecidableEqTok.decEq._proof_12` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.add = Polyrust.Tok.num a → False |
| 160 | `Polyrust.instDecidableEqTok.decEq._proof_13` | 實例衍生 | 定理 | | Polyrust.Tok.add = Polyrust.Tok.eqb → False |
| 161 | `Polyrust.instDecidableEqTok.decEq._proof_14` | 實例衍生 | 定理 | | Polyrust.Tok.add = Polyrust.Tok.ite → False |
| 162 | `Polyrust.instDecidableEqTok.decEq._proof_15` | 實例衍生 | 定理 | | Polyrust.Tok.eqb = Polyrust.Tok.unit → False |
| 163 | `Polyrust.instDecidableEqTok.decEq._proof_16` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.eqb = Polyrust.Tok.num a → False |
| 164 | `Polyrust.instDecidableEqTok.decEq._proof_17` | 實例衍生 | 定理 | | Polyrust.Tok.eqb = Polyrust.Tok.add → False |
| 165 | `Polyrust.instDecidableEqTok.decEq._proof_18` | 實例衍生 | 定理 | | Polyrust.Tok.eqb = Polyrust.Tok.ite → False |
| 166 | `Polyrust.instDecidableEqTok.decEq._proof_19` | 實例衍生 | 定理 | | Polyrust.Tok.ite = Polyrust.Tok.unit → False |
| 167 | `Polyrust.instDecidableEqTok.decEq._proof_2` | 實例衍生 | 定理 | | Polyrust.Tok.unit = Polyrust.Tok.add → False |
| 168 | `Polyrust.instDecidableEqTok.decEq._proof_20` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.ite = Polyrust.Tok.num a → False |
| 169 | `Polyrust.instDecidableEqTok.decEq._proof_21` | 實例衍生 | 定理 | | Polyrust.Tok.ite = Polyrust.Tok.add → False |
| 170 | `Polyrust.instDecidableEqTok.decEq._proof_22` | 實例衍生 | 定理 | | Polyrust.Tok.ite = Polyrust.Tok.eqb → False |
| 171 | `Polyrust.instDecidableEqTok.decEq._proof_3` | 實例衍生 | 定理 | | Polyrust.Tok.unit = Polyrust.Tok.eqb → False |
| 172 | `Polyrust.instDecidableEqTok.decEq._proof_4` | 實例衍生 | 定理 | | Polyrust.Tok.unit = Polyrust.Tok.ite → False |
| 173 | `Polyrust.instDecidableEqTok.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.unit → False |
| 174 | `Polyrust.instDecidableEqTok.decEq._proof_6` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.num a |
| 175 | `Polyrust.instDecidableEqTok.decEq._proof_7` | 實例衍生 | 定理 | | ∀ (a b : Int), ¬a = b → Polyrust.Tok.num a = Polyrust.Tok.num b → False |
| 176 | `Polyrust.instDecidableEqTok.decEq._proof_8` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.add → False |
| 177 | `Polyrust.instDecidableEqTok.decEq._proof_9` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.eqb → False |
| 178 | `Polyrust.instDecidableEqTok.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Tok → Polyrust.Tok → Sort u_1) →   (x x_1 : Polyrust.Tok) →     (Unit → motive Polyrust.Tok.unit Polyrust.Tok.unit) →       ((a : I |
| 179 | `Polyrust.instDecidableEqTy` | 實例衍生 | 定義 | | DecidableEq Polyrust.Ty |
| 180 | `Polyrust.instDecidableEqTy._proof_1` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.Ty), x.ctorIdx = y.ctorIdx → x = y |
| 181 | `Polyrust.instDecidableEqTy._proof_2` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.Ty), ¬x.ctorIdx = y.ctorIdx → x = y → False |
| 182 | `Polyrust.instReprExpr.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Expr → Sort u_1) →   (x : Polyrust.Expr) →     ((a : Int) → motive (Polyrust.Expr.num a)) →       ((a a_1 : Polyrust.Expr) → motive |
| 183 | `Polyrust.instReprTok.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Tok → Sort u_1) →   (x : Polyrust.Tok) →     (Unit → motive Polyrust.Tok.unit) →       ((a : Int) → motive (Polyrust.Tok.num a)) →  |
| 184 | `Polyrust.instReprTy` | 實例衍生 | 定義 | | Repr Polyrust.Ty |
| 185 | `Polyrust.instReprTy.repr` | 實例衍生 | 定義 | | Polyrust.Ty → Nat → Format |
| 186 | `Polyrust.instReprTy.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Ty → Sort u_1) →   (x : Polyrust.Ty) → (Unit → motive Polyrust.Ty.i32) → (Unit → motive Polyrust.Ty.boolean) → motive x |
| 187 | `Polyrust.isMonoAt_of_root.match_1_1` | 等式引理／助手 | 定義 | | ∀ {e' : Polyrust.Expr} {σ : Polyrust.Sigma} (τ τ' : Polyrust.Ty) (motive : σ e' τ = true ∧ σ e' τ' = true → Prop)   (h : σ e' τ = true ∧ σ e' τ' = tru |
| 188 | `Polyrust.occurs_oneHot_mem_genC.match_1_1` | 等式引理／助手 | 定義 | | ∀ (motive : (x x_1 : Polyrust.Expr) → Polyrust.Occurs x x_1 → Prop) (x x_1 : Polyrust.Expr)   (x_2 : Polyrust.Occurs x x_1),   (∀ (e : Polyrust.Expr), |
| 189 | `Polyrust.oneHot.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Sigma) (e : Polyrust.Expr),   Polyrust.oneHot σ e = Polyrust.tb σ e Polyrust.Ty.i32 + Polyrust.tb σ e Polyrust.Ty.boolean - 1 |
| 190 | `Polyrust.oneHotC.eq_1` | 等式引理／助手 | 定理 | | ∀ (e : Polyrust.Expr) (σ : Polyrust.Sigma), Polyrust.oneHotC e σ = Polyrust.oneHot σ e |
| 191 | `Polyrust.parseFuel._f` | 等式引理／助手 | 定義 | | (x : Nat) →   Nat.below (motive := fun x => List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok)) x →     List Polyrust.Tok → Option (Polyru |
| 192 | `Polyrust.parseFuel._sunfold` | 等式引理／助手 | 定義 | | Nat → List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok) |
| 193 | `Polyrust.parseFuel._unsafe_rec` | 等式引理／助手 | 定義 | | Nat → List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok) |
| 194 | `Polyrust.parseFuel.eq_1` | 等式引理／助手 | 定理 | | ∀ (x : List Polyrust.Tok), Polyrust.parseFuel 0 x = none |
| 195 | `Polyrust.parseFuel.eq_2` | 等式引理／助手 | 定理 | | ∀ (n : Nat), Polyrust.parseFuel n.succ [] = none |
| 196 | `Polyrust.parseFuel.eq_3` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.unit :: rest) = some (Polyrust.Expr.num 0, rest) |
| 197 | `Polyrust.parseFuel.eq_4` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (n_1 : Int) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.num n_1 :: rest) = some (Polyrust.Expr.num n_1, rest) |
| 198 | `Polyrust.parseFuel.eq_5` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.add :: rest) =     match Polyrust.parseFuel n rest with     \| none = |
| 199 | `Polyrust.parseFuel.eq_6` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.eqb :: rest) =     match Polyrust.parseFuel n rest with     \| none = |
| 200 | `Polyrust.parseFuel.eq_7` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.ite :: rest) =     match Polyrust.parseFuel n rest with     \| none = |
| 201 | `Polyrust.parseFuel.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Nat) (x_1 : List Polyrust.Tok),   Polyrust.parseFuel x x_1 =     match x, x_1 with     \| 0, x => none     \| n.succ, [] => none     \| n.succ, Po |
| 202 | `Polyrust.parseFuel.match_1` | 等式引理／助手 | 定義 | | (motive : Option (Polyrust.Expr × List Polyrust.Tok) → Sort u_1) →   (x : Option (Polyrust.Expr × List Polyrust.Tok)) →     (Unit → motive none) → ((b |
| 203 | `Polyrust.parseFuel.match_3` | 等式引理／助手 | 定義 | | (motive : Nat → List Polyrust.Tok → Sort u_1) →   (x : Nat) →     (x_1 : List Polyrust.Tok) →       ((x : List Polyrust.Tok) → motive 0 x) →         ( |
| 204 | `Polyrust.sizeT._f` | 等式引理／助手 | 定義 | | (x : Polyrust.Expr) → Polyrust.Expr.below x → Nat |
| 205 | `Polyrust.sizeT._sunfold` | 等式引理／助手 | 定義 | | Polyrust.Expr → Nat |
| 206 | `Polyrust.sizeT._unsafe_rec` | 等式引理／助手 | 定義 | | Polyrust.Expr → Nat |
| 207 | `Polyrust.sizeT.eq_1` | 等式引理／助手 | 定理 | | ∀ (a : Int), Polyrust.sizeT (Polyrust.Expr.num a) = 1 |
| 208 | `Polyrust.sizeT.eq_2` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.Expr), Polyrust.sizeT (a.add a_1) = 1 + (Polyrust.sizeT a + Polyrust.sizeT a_1) |
| 209 | `Polyrust.sizeT.eq_3` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.Expr), Polyrust.sizeT (a.eqb a_1) = 1 + (Polyrust.sizeT a + Polyrust.sizeT a_1) |
| 210 | `Polyrust.sizeT.eq_4` | 等式引理／助手 | 定理 | | ∀ (a a_1 a_2 : Polyrust.Expr),   Polyrust.sizeT (a.ite a_1 a_2) = 1 + (Polyrust.sizeT a + Polyrust.sizeT a_1 + Polyrust.sizeT a_2) |
| 211 | `Polyrust.sizeT.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.Expr),   Polyrust.sizeT x =     match x with     \| Polyrust.Expr.num a => 1     \| a.add b => 1 + (Polyrust.sizeT a + Polyrust.sizeT b) |
| 212 | `Polyrust.tb.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Sigma) (e : Polyrust.Expr) (τ : Polyrust.Ty), Polyrust.tb σ e τ = Polyrust.bit (σ e τ) |
| 213 | `Polyrust.untypable_iff_no_root.match_1_1` | 等式引理／助手 | 定義 | | ∀ (e : Polyrust.Expr) (motive : (∃ σ, Polyrust.IsRoot e σ) → Prop) (h : ∃ σ, Polyrust.IsRoot e σ),   (∀ (σ : Polyrust.Sigma) (hroot : Polyrust.IsRoot  |

## `Polyrust.TypeUniverse7PlusI`（147 條）

職責：7 基底 + i 擴展（17 種完整宇宙）；one-hot 分解；宇宙單調性

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 214 | `Polyrust.BaseTy7.all` | 手寫 | 定義 | | List Polyrust.BaseTy7 |
| 215 | `Polyrust.BaseTy7.all_nodup` | 手寫 | 定理 | | Polyrust.BaseTy7.all.Nodup |
| 216 | `Polyrust.BaseTy7.bool.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} → (t : Polyrust.BaseTy7) → t.ctorIdx = 1 → motive Polyrust.BaseTy7.bool → motive t |
| 217 | `Polyrust.BaseTy7.bool.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.bool = 1 |
| 218 | `Polyrust.BaseTy7.i32.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} → (t : Polyrust.BaseTy7) → t.ctorIdx = 0 → motive Polyrust.BaseTy7.i32 → motive t |
| 219 | `Polyrust.BaseTy7.i32.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.i32 = 1 |
| 220 | `Polyrust.BaseTy7.length_eq_7` | 手寫 | 定理 | | Polyrust.BaseTy7.all.length = 7 |
| 221 | `Polyrust.BaseTy7.ofNat` | 手寫 | 定義 | | Nat → Polyrust.BaseTy7 |
| 222 | `Polyrust.BaseTy7.ofNat_ctorIdx` | 手寫 | 定理 | | ∀ (x : Polyrust.BaseTy7), Polyrust.BaseTy7.ofNat x.ctorIdx = x |
| 223 | `Polyrust.BaseTy7.refBool.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (t : Polyrust.BaseTy7) → t.ctorIdx = 5 → motive Polyrust.BaseTy7.refBool → motive t |
| 224 | `Polyrust.BaseTy7.refBool.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.refBool = 1 |
| 225 | `Polyrust.BaseTy7.refI32.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (t : Polyrust.BaseTy7) → t.ctorIdx = 3 → motive Polyrust.BaseTy7.refI32 → motive t |
| 226 | `Polyrust.BaseTy7.refI32.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.refI32 = 1 |
| 227 | `Polyrust.BaseTy7.refMutBool.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (t : Polyrust.BaseTy7) → t.ctorIdx = 6 → motive Polyrust.BaseTy7.refMutBool → motive t |
| 228 | `Polyrust.BaseTy7.refMutBool.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.refMutBool = 1 |
| 229 | `Polyrust.BaseTy7.refMutI32.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (t : Polyrust.BaseTy7) → t.ctorIdx = 4 → motive Polyrust.BaseTy7.refMutI32 → motive t |
| 230 | `Polyrust.BaseTy7.refMutI32.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.refMutI32 = 1 |
| 231 | `Polyrust.BaseTy7.toCtorIdx` | 手寫 | 定義 | | Polyrust.BaseTy7 → Nat |
| 232 | `Polyrust.BaseTy7.unit.elim` | 手寫 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} → (t : Polyrust.BaseTy7) → t.ctorIdx = 2 → motive Polyrust.BaseTy7.unit → motive t |
| 233 | `Polyrust.BaseTy7.unit.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.BaseTy7.unit = 1 |
| 234 | `Polyrust.ExtTag.all` | 手寫 | 定義 | | List Polyrust.ExtTag |
| 235 | `Polyrust.ExtTag.all_nodup` | 手寫 | 定理 | | Polyrust.ExtTag.all.Nodup |
| 236 | `Polyrust.ExtTag.enumTy.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 4 → motive Polyrust.ExtTag.enumTy → motive t |
| 237 | `Polyrust.ExtTag.enumTy.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.enumTy = 1 |
| 238 | `Polyrust.ExtTag.future.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 7 → motive Polyrust.ExtTag.future → motive t |
| 239 | `Polyrust.ExtTag.future.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.future = 1 |
| 240 | `Polyrust.ExtTag.hashmap.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 2 → motive Polyrust.ExtTag.hashmap → motive t |
| 241 | `Polyrust.ExtTag.hashmap.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.hashmap = 1 |
| 242 | `Polyrust.ExtTag.length_eq_10` | 手寫 | 定理 | | Polyrust.ExtTag.all.length = 10 |
| 243 | `Polyrust.ExtTag.ofNat` | 手寫 | 定義 | | Nat → Polyrust.ExtTag |
| 244 | `Polyrust.ExtTag.ofNat_ctorIdx` | 手寫 | 定理 | | ∀ (x : Polyrust.ExtTag), Polyrust.ExtTag.ofNat x.ctorIdx = x |
| 245 | `Polyrust.ExtTag.option.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 8 → motive Polyrust.ExtTag.option → motive t |
| 246 | `Polyrust.ExtTag.option.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.option = 1 |
| 247 | `Polyrust.ExtTag.rawPtrConst.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} →   (t : Polyrust.ExtTag) → t.ctorIdx = 6 → motive Polyrust.ExtTag.rawPtrConst → motive t |
| 248 | `Polyrust.ExtTag.rawPtrConst.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.rawPtrConst = 1 |
| 249 | `Polyrust.ExtTag.rawPtrMut.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} →   (t : Polyrust.ExtTag) → t.ctorIdx = 5 → motive Polyrust.ExtTag.rawPtrMut → motive t |
| 250 | `Polyrust.ExtTag.rawPtrMut.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.rawPtrMut = 1 |
| 251 | `Polyrust.ExtTag.result.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 9 → motive Polyrust.ExtTag.result → motive t |
| 252 | `Polyrust.ExtTag.result.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.result = 1 |
| 253 | `Polyrust.ExtTag.string.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 1 → motive Polyrust.ExtTag.string → motive t |
| 254 | `Polyrust.ExtTag.string.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.string = 1 |
| 255 | `Polyrust.ExtTag.structTy.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 3 → motive Polyrust.ExtTag.structTy → motive t |
| 256 | `Polyrust.ExtTag.structTy.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.structTy = 1 |
| 257 | `Polyrust.ExtTag.toCtorIdx` | 手寫 | 定義 | | Polyrust.ExtTag → Nat |
| 258 | `Polyrust.ExtTag.vec.elim` | 手寫 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → (t : Polyrust.ExtTag) → t.ctorIdx = 0 → motive Polyrust.ExtTag.vec → motive t |
| 259 | `Polyrust.ExtTag.vec.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.ExtTag.vec = 1 |
| 260 | `Polyrust.Lang.mk.congr_simp` | 手寫 | 定理 | | ∀ {Ty : Type} (enumAll enumAll_1 : List Ty) (e_enumAll : enumAll = enumAll_1) (nodup : enumAll.Nodup)   (complete : ∀ (t : Ty), t ∈ enumAll) (numTy nu |
| 261 | `Polyrust.Ty7Plus10.all` | 手寫 | 定義 | | List Polyrust.Ty7Plus10 |
| 262 | `Polyrust.Ty7Plus10.all_length` | 手寫 | 定理 | | Polyrust.Ty7Plus10.all.length = 17 |
| 263 | `Polyrust.Ty7Plus10.all_nodup` | 手寫 | 定理 | | Polyrust.Ty7Plus10.all.Nodup |
| 264 | `Polyrust.Ty7Plus10.base.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty7Plus10 → Sort u} →   (t : Polyrust.Ty7Plus10) → t.ctorIdx = 0 → ((a : Polyrust.BaseTy7) → motive (Polyrust.Ty7Plus10.base a)) →  |
| 265 | `Polyrust.Ty7Plus10.base.inj` | 手寫 | 定理 | | ∀ {a a_1 : Polyrust.BaseTy7}, Polyrust.Ty7Plus10.base a = Polyrust.Ty7Plus10.base a_1 → a = a_1 |
| 266 | `Polyrust.Ty7Plus10.base.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Polyrust.BaseTy7), sizeOf (Polyrust.Ty7Plus10.base a) = 1 + sizeOf a |
| 267 | `Polyrust.Ty7Plus10.base_ne_ext` | 手寫 | 定理 | | ∀ (b : Polyrust.BaseTy7) (e : Polyrust.ExtTag), Polyrust.Ty7Plus10.base b ≠ Polyrust.Ty7Plus10.ext e |
| 268 | `Polyrust.Ty7Plus10.ext.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty7Plus10 → Sort u} →   (t : Polyrust.Ty7Plus10) → t.ctorIdx = 1 → ((a : Polyrust.ExtTag) → motive (Polyrust.Ty7Plus10.ext a)) → mo |
| 269 | `Polyrust.Ty7Plus10.ext.inj` | 手寫 | 定理 | | ∀ {a a_1 : Polyrust.ExtTag}, Polyrust.Ty7Plus10.ext a = Polyrust.Ty7Plus10.ext a_1 → a = a_1 |
| 270 | `Polyrust.Ty7Plus10.ext.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Polyrust.ExtTag), sizeOf (Polyrust.Ty7Plus10.ext a) = 1 + sizeOf a |
| 271 | `Polyrust.Ty7Plus10.isBase` | 手寫 | 定義 | | Polyrust.Ty7Plus10 → Bool |
| 272 | `Polyrust.Ty7Plus10.isExt` | 手寫 | 定義 | | Polyrust.Ty7Plus10 → Bool |
| 273 | `Polyrust.base_embedding_injective` | 手寫 | 定理 | | Function.Injective Polyrust.Ty7Plus10.base |
| 274 | `Polyrust.base_injective` | 手寫 | 定理 | | Function.Injective Polyrust.Ty7Plus10.base |
| 275 | `Polyrust.embedBase7` | 手寫 | 定義 | | Polyrust.BaseTy7 → Polyrust.Ty7Plus10 |
| 276 | `Polyrust.embedBase7_injective` | 手寫 | 定理 | | Function.Injective Polyrust.embedBase7 |
| 277 | `Polyrust.ext_injective` | 手寫 | 定理 | | Function.Injective Polyrust.Ty7Plus10.ext |
| 278 | `Polyrust.ext_ne_of_tag_ne` | 手寫 | 定理 | | ∀ {e1 e2 : Polyrust.ExtTag}, e1 ≠ e2 → Polyrust.Ty7Plus10.ext e1 ≠ Polyrust.Ty7Plus10.ext e2 |
| 279 | `Polyrust.exts0` | 手寫 | 定義 | | List Polyrust.ExtTag |
| 280 | `Polyrust.exts1` | 手寫 | 定義 | | List Polyrust.ExtTag |
| 281 | `Polyrust.exts10` | 手寫 | 定義 | | List Polyrust.ExtTag |
| 282 | `Polyrust.exts10_nodup` | 手寫 | 定理 | | Polyrust.exts10.Nodup |
| 283 | `Polyrust.exts3` | 手寫 | 定義 | | List Polyrust.ExtTag |
| 284 | `Polyrust.mkUniverse7PlusI` | 手寫 | 定義 | | List Polyrust.ExtTag → List Polyrust.Ty7Plus10 |
| 285 | `Polyrust.oneHotPoly` | 手寫 | 定義 | | List Polyrust.ExtTag → (Polyrust.Ty7Plus10 → Bool) → Int |
| 286 | `Polyrust.phase1_complete` | 手寫 | 定義 | | Bool |
| 287 | `Polyrust.phase1_sound` | 手寫 | 定理 | | Polyrust.phase1_complete = true |
| 288 | `Polyrust.universe10` | 手寫 | 定義 | | List Polyrust.Ty7Plus10 |
| 289 | `Polyrust.universe17` | 手寫 | 定義 | | List Polyrust.Ty7Plus10 |
| 290 | `Polyrust.universe7` | 手寫 | 定義 | | List Polyrust.Ty7Plus10 |
| 291 | `Polyrust.universe8` | 手寫 | 定義 | | List Polyrust.Ty7Plus10 |
| 292 | `Polyrust.BaseTy7._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.BaseTy7 → Nat |
| 293 | `Polyrust.BaseTy7._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.BaseTy7 |
| 294 | `Polyrust.BaseTy7.all.eq_1` | 等式引理／助手 | 定理 | | Polyrust.BaseTy7.all =   [Polyrust.BaseTy7.i32, Polyrust.BaseTy7.bool, Polyrust.BaseTy7.unit, Polyrust.BaseTy7.refI32,     Polyrust.BaseTy7.refMutI32, |
| 295 | `Polyrust.BaseTy7.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (t : Polyrust.BaseTy7) →     motive Polyrust.BaseTy7.i32 →       motive Polyrust.BaseTy7.bool →         motiv |
| 296 | `Polyrust.BaseTy7.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.BaseTy7) → ctorIdx = t.ctorIdx → Polyrust.BaseTy7.ctorElimType ctorIdx → moti |
| 297 | `Polyrust.BaseTy7.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} → Nat → Sort (max 1 u) |
| 298 | `Polyrust.BaseTy7.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.BaseTy7 → Nat |
| 299 | `Polyrust.BaseTy7.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort v✝} → {x y : Polyrust.BaseTy7} → x = y → Polyrust.BaseTy7.noConfusionType P x y |
| 300 | `Polyrust.BaseTy7.noConfusionType` | 衍生 | 定義 | | Sort v✝ → Polyrust.BaseTy7 → Polyrust.BaseTy7 → Sort v✝ |
| 301 | `Polyrust.BaseTy7.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.BaseTy7 → Sort u} →   (t : Polyrust.BaseTy7) →     motive Polyrust.BaseTy7.i32 →       motive Polyrust.BaseTy7.bool →         motiv |
| 302 | `Polyrust.ExtTag._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.ExtTag → Nat |
| 303 | `Polyrust.ExtTag._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.ExtTag |
| 304 | `Polyrust.ExtTag.all.eq_1` | 等式引理／助手 | 定理 | | Polyrust.ExtTag.all =   [Polyrust.ExtTag.vec, Polyrust.ExtTag.string, Polyrust.ExtTag.hashmap, Polyrust.ExtTag.structTy,     Polyrust.ExtTag.enumTy, P |
| 305 | `Polyrust.ExtTag.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ExtTag → Sort u} →   (t : Polyrust.ExtTag) →     motive Polyrust.ExtTag.vec →       motive Polyrust.ExtTag.string →         motive  |
| 306 | `Polyrust.ExtTag.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.ExtTag → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.ExtTag) → ctorIdx = t.ctorIdx → Polyrust.ExtTag.ctorElimType ctorIdx → motive  |
| 307 | `Polyrust.ExtTag.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.ExtTag → Sort u} → Nat → Sort (max 1 u) |
| 308 | `Polyrust.ExtTag.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.ExtTag → Nat |
| 309 | `Polyrust.ExtTag.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort v✝} → {x y : Polyrust.ExtTag} → x = y → Polyrust.ExtTag.noConfusionType P x y |
| 310 | `Polyrust.ExtTag.noConfusionType` | 衍生 | 定義 | | Sort v✝ → Polyrust.ExtTag → Polyrust.ExtTag → Sort v✝ |
| 311 | `Polyrust.ExtTag.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ExtTag → Sort u} →   (t : Polyrust.ExtTag) →     motive Polyrust.ExtTag.vec →       motive Polyrust.ExtTag.string →         motive  |
| 312 | `Polyrust.Ty7Plus10._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Ty7Plus10 → Nat |
| 313 | `Polyrust.Ty7Plus10._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Ty7Plus10 |
| 314 | `Polyrust.Ty7Plus10.all.eq_1` | 等式引理／助手 | 定理 | | Polyrust.Ty7Plus10.all =   List.map Polyrust.Ty7Plus10.base Polyrust.BaseTy7.all ++ List.map Polyrust.Ty7Plus10.ext Polyrust.ExtTag.all |
| 315 | `Polyrust.Ty7Plus10.base.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Polyrust.BaseTy7} → Polyrust.Ty7Plus10.base a = Polyrust.Ty7Plus10.base a' → (a = a' → P) → P |
| 316 | `Polyrust.Ty7Plus10.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty7Plus10 → Sort u} →   (t : Polyrust.Ty7Plus10) →     ((a : Polyrust.BaseTy7) → motive (Polyrust.Ty7Plus10.base a)) →       ((a :  |
| 317 | `Polyrust.Ty7Plus10.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty7Plus10 → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Ty7Plus10) → ctorIdx = t.ctorIdx → Polyrust.Ty7Plus10.ctorElimType ctorIdx  |
| 318 | `Polyrust.Ty7Plus10.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.Ty7Plus10 → Sort u} → Nat → Sort (max 1 u) |
| 319 | `Polyrust.Ty7Plus10.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Ty7Plus10 → Nat |
| 320 | `Polyrust.Ty7Plus10.ext.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Polyrust.ExtTag} → Polyrust.Ty7Plus10.ext a = Polyrust.Ty7Plus10.ext a' → (a = a' → P) → P |
| 321 | `Polyrust.Ty7Plus10.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Ty7Plus10} → t = t' → Polyrust.Ty7Plus10.noConfusionType P t t' |
| 322 | `Polyrust.Ty7Plus10.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Ty7Plus10 → Polyrust.Ty7Plus10 → Sort u |
| 323 | `Polyrust.Ty7Plus10.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty7Plus10 → Sort u} →   (t : Polyrust.Ty7Plus10) →     ((a : Polyrust.BaseTy7) → motive (Polyrust.Ty7Plus10.base a)) →       ((a :  |
| 324 | `Polyrust.exts0.eq_1` | 等式引理／助手 | 定理 | | Polyrust.exts0 = [] |
| 325 | `Polyrust.exts1.eq_1` | 等式引理／助手 | 定理 | | Polyrust.exts1 = [Polyrust.ExtTag.vec] |
| 326 | `Polyrust.exts3.eq_1` | 等式引理／助手 | 定理 | | Polyrust.exts3 = [Polyrust.ExtTag.vec, Polyrust.ExtTag.string, Polyrust.ExtTag.option] |
| 327 | `Polyrust.instBEqBaseTy7` | 實例衍生 | 定義 | | BEq Polyrust.BaseTy7 |
| 328 | `Polyrust.instBEqBaseTy7.beq` | 實例衍生 | 定義 | | Polyrust.BaseTy7 → Polyrust.BaseTy7 → Bool |
| 329 | `Polyrust.instBEqExtTag` | 實例衍生 | 定義 | | BEq Polyrust.ExtTag |
| 330 | `Polyrust.instBEqExtTag.beq` | 實例衍生 | 定義 | | Polyrust.ExtTag → Polyrust.ExtTag → Bool |
| 331 | `Polyrust.instDecidableEqBaseTy7` | 實例衍生 | 定義 | | DecidableEq Polyrust.BaseTy7 |
| 332 | `Polyrust.instDecidableEqBaseTy7._proof_1` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.BaseTy7), x.ctorIdx = y.ctorIdx → x = y |
| 333 | `Polyrust.instDecidableEqBaseTy7._proof_2` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.BaseTy7), ¬x.ctorIdx = y.ctorIdx → x = y → False |
| 334 | `Polyrust.instDecidableEqExtTag` | 實例衍生 | 定義 | | DecidableEq Polyrust.ExtTag |
| 335 | `Polyrust.instDecidableEqExtTag._proof_1` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.ExtTag), x.ctorIdx = y.ctorIdx → x = y |
| 336 | `Polyrust.instDecidableEqExtTag._proof_2` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.ExtTag), ¬x.ctorIdx = y.ctorIdx → x = y → False |
| 337 | `Polyrust.instDecidableEqTy7Plus10` | 實例衍生 | 定義 | | DecidableEq Polyrust.Ty7Plus10 |
| 338 | `Polyrust.instDecidableEqTy7Plus10.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Ty7Plus10) → Decidable (x = x_1) |
| 339 | `Polyrust.instDecidableEqTy7Plus10.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : Polyrust.BaseTy7), Polyrust.Ty7Plus10.base a = Polyrust.Ty7Plus10.base a |
| 340 | `Polyrust.instDecidableEqTy7Plus10.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a b : Polyrust.BaseTy7), ¬a = b → Polyrust.Ty7Plus10.base a = Polyrust.Ty7Plus10.base b → False |
| 341 | `Polyrust.instDecidableEqTy7Plus10.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : Polyrust.BaseTy7) (a_1 : Polyrust.ExtTag), Polyrust.Ty7Plus10.base a = Polyrust.Ty7Plus10.ext a_1 → False |
| 342 | `Polyrust.instDecidableEqTy7Plus10.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : Polyrust.ExtTag) (a_1 : Polyrust.BaseTy7), Polyrust.Ty7Plus10.ext a = Polyrust.Ty7Plus10.base a_1 → False |
| 343 | `Polyrust.instDecidableEqTy7Plus10.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a : Polyrust.ExtTag), Polyrust.Ty7Plus10.ext a = Polyrust.Ty7Plus10.ext a |
| 344 | `Polyrust.instDecidableEqTy7Plus10.decEq._proof_6` | 實例衍生 | 定理 | | ∀ (a b : Polyrust.ExtTag), ¬a = b → Polyrust.Ty7Plus10.ext a = Polyrust.Ty7Plus10.ext b → False |
| 345 | `Polyrust.instDecidableEqTy7Plus10.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Ty7Plus10 → Polyrust.Ty7Plus10 → Sort u_1) →   (x x_1 : Polyrust.Ty7Plus10) →     ((a b : Polyrust.BaseTy7) → motive (Polyrust.Ty7P |
| 346 | `Polyrust.instReprBaseTy7` | 實例衍生 | 定義 | | Repr Polyrust.BaseTy7 |
| 347 | `Polyrust.instReprBaseTy7.repr` | 實例衍生 | 定義 | | Polyrust.BaseTy7 → Nat → Format |
| 348 | `Polyrust.instReprBaseTy7.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.BaseTy7 → Sort u_1) →   (x : Polyrust.BaseTy7) →     (Unit → motive Polyrust.BaseTy7.i32) →       (Unit → motive Polyrust.BaseTy7.b |
| 349 | `Polyrust.instReprExtTag` | 實例衍生 | 定義 | | Repr Polyrust.ExtTag |
| 350 | `Polyrust.instReprExtTag.repr` | 實例衍生 | 定義 | | Polyrust.ExtTag → Nat → Format |
| 351 | `Polyrust.instReprExtTag.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.ExtTag → Sort u_1) →   (x : Polyrust.ExtTag) →     (Unit → motive Polyrust.ExtTag.vec) →       (Unit → motive Polyrust.ExtTag.strin |
| 352 | `Polyrust.instReprTy7Plus10` | 實例衍生 | 定義 | | Repr Polyrust.Ty7Plus10 |
| 353 | `Polyrust.instReprTy7Plus10.repr` | 實例衍生 | 定義 | | Polyrust.Ty7Plus10 → Nat → Format |
| 354 | `Polyrust.instReprTy7Plus10.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Ty7Plus10 → Sort u_1) →   (x : Polyrust.Ty7Plus10) →     ((a : Polyrust.BaseTy7) → motive (Polyrust.Ty7Plus10.base a)) →       ((a  |
| 355 | `Polyrust.lang17._proof_1` | 等式引理／助手 | 定理 | | Polyrust.Ty7Plus10.base Polyrust.BaseTy7.i32 = Polyrust.Ty7Plus10.base Polyrust.BaseTy7.bool → False |
| 356 | `Polyrust.mkUniverse7PlusI.eq_1` | 等式引理／助手 | 定理 | | ∀ (exts : List Polyrust.ExtTag),   Polyrust.mkUniverse7PlusI exts =     List.map Polyrust.Ty7Plus10.base Polyrust.BaseTy7.all ++ List.map Polyrust.Ty7 |
| 357 | `Polyrust.universe10.eq_1` | 等式引理／助手 | 定理 | | Polyrust.universe10 = Polyrust.mkUniverse7PlusI Polyrust.exts3 |
| 358 | `Polyrust.universe17.eq_1` | 等式引理／助手 | 定理 | | Polyrust.universe17 = Polyrust.Ty7Plus10.all |
| 359 | `Polyrust.universe7.eq_1` | 等式引理／助手 | 定理 | | Polyrust.universe7 = Polyrust.mkUniverse7PlusI Polyrust.exts0 |
| 360 | `Polyrust.universe8.eq_1` | 等式引理／助手 | 定理 | | Polyrust.universe8 = Polyrust.mkUniverse7PlusI Polyrust.exts1 |

## `Polyrust.MacroExpansion`（109 條）

職責：模板語法/上下文/代入；展開是同態

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 361 | `Polyrust.Ctx` | 手寫 | 定義 | | Type |
| 362 | `Polyrust.VExpr.add.elim` | 手寫 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 2 → ((a a_1 : Polyrust.VExpr) → motive (a.add a_1)) → motive t |
| 363 | `Polyrust.VExpr.add.inj` | 手寫 | 定理 | | ∀ {a a_1 a_2 a_3 : Polyrust.VExpr}, a.add a_1 = a_2.add a_3 → a = a_2 ∧ a_1 = a_3 |
| 364 | `Polyrust.VExpr.add.sizeOf_spec` | 手寫 | 定理 | | ∀ (a a_1 : Polyrust.VExpr), sizeOf (a.add a_1) = 1 + sizeOf a + sizeOf a_1 |
| 365 | `Polyrust.VExpr.eqb.elim` | 手寫 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 3 → ((a a_1 : Polyrust.VExpr) → motive (a.eqb a_1)) → motive t |
| 366 | `Polyrust.VExpr.eqb.inj` | 手寫 | 定理 | | ∀ {a a_1 a_2 a_3 : Polyrust.VExpr}, a.eqb a_1 = a_2.eqb a_3 → a = a_2 ∧ a_1 = a_3 |
| 367 | `Polyrust.VExpr.eqb.sizeOf_spec` | 手寫 | 定理 | | ∀ (a a_1 : Polyrust.VExpr), sizeOf (a.eqb a_1) = 1 + sizeOf a + sizeOf a_1 |
| 368 | `Polyrust.VExpr.ite.elim` | 手寫 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 4 → ((a a_1 a_2 : Polyrust.VExpr) → motive (a.ite a_1 a_2)) → motive t |
| 369 | `Polyrust.VExpr.ite.inj` | 手寫 | 定理 | | ∀ {a a_1 a_2 a_3 a_4 a_5 : Polyrust.VExpr}, a.ite a_1 a_2 = a_3.ite a_4 a_5 → a = a_3 ∧ a_1 = a_4 ∧ a_2 = a_5 |
| 370 | `Polyrust.VExpr.ite.sizeOf_spec` | 手寫 | 定理 | | ∀ (a a_1 a_2 : Polyrust.VExpr), sizeOf (a.ite a_1 a_2) = 1 + sizeOf a + sizeOf a_1 + sizeOf a_2 |
| 371 | `Polyrust.VExpr.num.elim` | 手寫 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 1 → ((a : Int) → motive (Polyrust.VExpr.num a)) → motive t |
| 372 | `Polyrust.VExpr.num.inj` | 手寫 | 定理 | | ∀ {a a_1 : Int}, Polyrust.VExpr.num a = Polyrust.VExpr.num a_1 → a = a_1 |
| 373 | `Polyrust.VExpr.num.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Int), sizeOf (Polyrust.VExpr.num a) = 1 + sizeOf a |
| 374 | `Polyrust.VExpr.var.elim` | 手寫 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 0 → ((a : Nat) → motive (Polyrust.VExpr.var a)) → motive t |
| 375 | `Polyrust.VExpr.var.inj` | 手寫 | 定理 | | ∀ {a a_1 : Nat}, Polyrust.VExpr.var a = Polyrust.VExpr.var a_1 → a = a_1 |
| 376 | `Polyrust.VExpr.var.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Nat), sizeOf (Polyrust.VExpr.var a) = 1 + sizeOf a |
| 377 | `Polyrust.armCtx` | 手寫 | 定義 | | Nat → Polyrust.Ctx |
| 378 | `Polyrust.armTemplate` | 手寫 | 定義 | | Nat → Polyrust.VExpr |
| 379 | `Polyrust.expand` | 手寫 | 定義 | | (Nat → Polyrust.Expr) → Polyrust.VExpr → Polyrust.Expr |
| 380 | `Polyrust.subst` | 手寫 | 定義 | | (Nat → Polyrust.VExpr) → Polyrust.VExpr → Polyrust.VExpr |
| 381 | `Polyrust.vars` | 手寫 | 定義 | | Polyrust.VExpr → List Nat |
| 382 | `Polyrust.VExpr._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.VExpr → Nat |
| 383 | `Polyrust.VExpr._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.VExpr |
| 384 | `Polyrust.VExpr.add.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a_1 a' a'_1 : Polyrust.VExpr} → a.add a_1 = a'.add a'_1 → (a = a' → a_1 = a'_1 → P) → P |
| 385 | `Polyrust.VExpr.below` | 歸納型衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} → Polyrust.VExpr → Sort (max 1 u) |
| 386 | `Polyrust.VExpr.brecOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → ((t : Polyrust.VExpr) → Polyrust.VExpr.below t → motive t) → motive t |
| 387 | `Polyrust.VExpr.brecOn.eq` | 衍生 | 定理 | | ∀ {motive : Polyrust.VExpr → Sort u} (t : Polyrust.VExpr)   (F_1 : (t : Polyrust.VExpr) → Polyrust.VExpr.below t → motive t),   Polyrust.VExpr.brecOn  |
| 388 | `Polyrust.VExpr.brecOn.go` | 歸納型衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → ((t : Polyrust.VExpr) → Polyrust.VExpr.below t → motive t) → motive t ×' Polyrust.VExpr. |
| 389 | `Polyrust.VExpr.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) →     ((a : Nat) → motive (Polyrust.VExpr.var a)) →       ((a : Int) → motive (Polyrust.VE |
| 390 | `Polyrust.VExpr.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.VExpr) → ctorIdx = t.ctorIdx → Polyrust.VExpr.ctorElimType ctorIdx → motive t |
| 391 | `Polyrust.VExpr.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} → Nat → Sort (max 1 u) |
| 392 | `Polyrust.VExpr.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.VExpr → Nat |
| 393 | `Polyrust.VExpr.eqb.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a_1 a' a'_1 : Polyrust.VExpr} → a.eqb a_1 = a'.eqb a'_1 → (a = a' → a_1 = a'_1 → P) → P |
| 394 | `Polyrust.VExpr.ite.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {a a_1 a_2 a' a'_1 a'_2 : Polyrust.VExpr} →     a.ite a_1 a_2 = a'.ite a'_1 a'_2 → (a = a' → a_1 = a'_1 → a_2 = a'_2 → P) → P |
| 395 | `Polyrust.VExpr.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.VExpr} → t = t' → Polyrust.VExpr.noConfusionType P t t' |
| 396 | `Polyrust.VExpr.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.VExpr → Polyrust.VExpr → Sort u |
| 397 | `Polyrust.VExpr.num.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Int} → Polyrust.VExpr.num a = Polyrust.VExpr.num a' → (a = a' → P) → P |
| 398 | `Polyrust.VExpr.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) →     ((a : Nat) → motive (Polyrust.VExpr.var a)) →       ((a : Int) → motive (Polyrust.VE |
| 399 | `Polyrust.VExpr.var.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Nat} → Polyrust.VExpr.var a = Polyrust.VExpr.var a' → (a = a' → P) → P |
| 400 | `Polyrust.armCtx.eq_1` | 等式引理／助手 | 定理 | | ∀ (v w : Nat), Polyrust.armCtx v w = if w = v then some Polyrust.Ty.i32 else none |
| 401 | `Polyrust.armTemplate.eq_1` | 等式引理／助手 | 定理 | | ∀ (v : Nat), Polyrust.armTemplate v = (Polyrust.VExpr.var v).add (Polyrust.VExpr.num 1) |
| 402 | `Polyrust.expand._f` | 等式引理／助手 | 定義 | | (Nat → Polyrust.Expr) → (x : Polyrust.VExpr) → Polyrust.VExpr.below x → Polyrust.Expr |
| 403 | `Polyrust.expand._sunfold` | 等式引理／助手 | 定義 | | (Nat → Polyrust.Expr) → Polyrust.VExpr → Polyrust.Expr |
| 404 | `Polyrust.expand._unsafe_rec` | 等式引理／助手 | 定義 | | (Nat → Polyrust.Expr) → Polyrust.VExpr → Polyrust.Expr |
| 405 | `Polyrust.expand.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Nat → Polyrust.Expr) (a : Nat), Polyrust.expand σ (Polyrust.VExpr.var a) = σ a |
| 406 | `Polyrust.expand.eq_2` | 等式引理／助手 | 定理 | | ∀ (σ : Nat → Polyrust.Expr) (a : Int), Polyrust.expand σ (Polyrust.VExpr.num a) = Polyrust.Expr.num a |
| 407 | `Polyrust.expand.eq_3` | 等式引理／助手 | 定理 | | ∀ (σ : Nat → Polyrust.Expr) (a a_1 : Polyrust.VExpr),   Polyrust.expand σ (a.add a_1) = (Polyrust.expand σ a).add (Polyrust.expand σ a_1) |
| 408 | `Polyrust.expand.eq_4` | 等式引理／助手 | 定理 | | ∀ (σ : Nat → Polyrust.Expr) (a a_1 : Polyrust.VExpr),   Polyrust.expand σ (a.eqb a_1) = (Polyrust.expand σ a).eqb (Polyrust.expand σ a_1) |
| 409 | `Polyrust.expand.eq_5` | 等式引理／助手 | 定理 | | ∀ (σ : Nat → Polyrust.Expr) (a a_1 a_2 : Polyrust.VExpr),   Polyrust.expand σ (a.ite a_1 a_2) = (Polyrust.expand σ a).ite (Polyrust.expand σ a_1) (Pol |
| 410 | `Polyrust.expand.eq_def` | 等式引理／助手 | 定理 | | ∀ (σ : Nat → Polyrust.Expr) (x : Polyrust.VExpr),   Polyrust.expand σ x =     match x with     \| Polyrust.VExpr.var v => σ v     \| Polyrust.VExpr.num  |
| 411 | `Polyrust.instDecidableEqVExpr` | 實例衍生 | 定義 | | DecidableEq Polyrust.VExpr |
| 412 | `Polyrust.instDecidableEqVExpr.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.VExpr) → Decidable (x = x_1) |
| 413 | `Polyrust.instDecidableEqVExpr.decEq._f` | 實例衍生 | 定義 | | (x : Polyrust.VExpr) →   Polyrust.VExpr.below (motive := fun x => (x_1 : Polyrust.VExpr) → Decidable (x = x_1)) x →     (x_1 : Polyrust.VExpr) → Decid |
| 414 | `Polyrust.instDecidableEqVExpr.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : Nat), Polyrust.VExpr.var a = Polyrust.VExpr.var a |
| 415 | `Polyrust.instDecidableEqVExpr.decEq._proof_10` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.num a = a_1.add a_2 → False |
| 416 | `Polyrust.instDecidableEqVExpr.decEq._proof_11` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.num a = a_1.eqb a_2 → False |
| 417 | `Polyrust.instDecidableEqVExpr.decEq._proof_12` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 a_2 a_3 : Polyrust.VExpr), Polyrust.VExpr.num a = a_1.ite a_2 a_3 → False |
| 418 | `Polyrust.instDecidableEqVExpr.decEq._proof_13` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.VExpr) (a_2 : Nat), a.add a_1 = Polyrust.VExpr.var a_2 → False |
| 419 | `Polyrust.instDecidableEqVExpr.decEq._proof_14` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.VExpr) (a_2 : Int), a.add a_1 = Polyrust.VExpr.num a_2 → False |
| 420 | `Polyrust.instDecidableEqVExpr.decEq._proof_15` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.VExpr), a.add a_1 = a.add a_1 |
| 421 | `Polyrust.instDecidableEqVExpr.decEq._proof_16` | 實例衍生 | 定理 | | ∀ (a a_1 b : Polyrust.VExpr), ¬a_1 = b → a.add a_1 = a.add b → False |
| 422 | `Polyrust.instDecidableEqVExpr.decEq._proof_17` | 實例衍生 | 定理 | | ∀ (a a_1 b b_1 : Polyrust.VExpr), ¬a = b → a.add a_1 = b.add b_1 → False |
| 423 | `Polyrust.instDecidableEqVExpr.decEq._proof_18` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 : Polyrust.VExpr), a.add a_1 = a_2.eqb a_3 → False |
| 424 | `Polyrust.instDecidableEqVExpr.decEq._proof_19` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.add a_1 = a_2.ite a_3 a_4 → False |
| 425 | `Polyrust.instDecidableEqVExpr.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a b : Nat), ¬a = b → Polyrust.VExpr.var a = Polyrust.VExpr.var b → False |
| 426 | `Polyrust.instDecidableEqVExpr.decEq._proof_20` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.VExpr) (a_2 : Nat), a.eqb a_1 = Polyrust.VExpr.var a_2 → False |
| 427 | `Polyrust.instDecidableEqVExpr.decEq._proof_21` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.VExpr) (a_2 : Int), a.eqb a_1 = Polyrust.VExpr.num a_2 → False |
| 428 | `Polyrust.instDecidableEqVExpr.decEq._proof_22` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 : Polyrust.VExpr), a.eqb a_1 = a_2.add a_3 → False |
| 429 | `Polyrust.instDecidableEqVExpr.decEq._proof_23` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.VExpr), a.eqb a_1 = a.eqb a_1 |
| 430 | `Polyrust.instDecidableEqVExpr.decEq._proof_24` | 實例衍生 | 定理 | | ∀ (a a_1 b : Polyrust.VExpr), ¬a_1 = b → a.eqb a_1 = a.eqb b → False |
| 431 | `Polyrust.instDecidableEqVExpr.decEq._proof_25` | 實例衍生 | 定理 | | ∀ (a a_1 b b_1 : Polyrust.VExpr), ¬a = b → a.eqb a_1 = b.eqb b_1 → False |
| 432 | `Polyrust.instDecidableEqVExpr.decEq._proof_26` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.eqb a_1 = a_2.ite a_3 a_4 → False |
| 433 | `Polyrust.instDecidableEqVExpr.decEq._proof_27` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 : Polyrust.VExpr) (a_3 : Nat), a.ite a_1 a_2 = Polyrust.VExpr.var a_3 → False |
| 434 | `Polyrust.instDecidableEqVExpr.decEq._proof_28` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 : Polyrust.VExpr) (a_3 : Int), a.ite a_1 a_2 = Polyrust.VExpr.num a_3 → False |
| 435 | `Polyrust.instDecidableEqVExpr.decEq._proof_29` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.ite a_1 a_2 = a_3.add a_4 → False |
| 436 | `Polyrust.instDecidableEqVExpr.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : Nat) (a_1 : Int), Polyrust.VExpr.var a = Polyrust.VExpr.num a_1 → False |
| 437 | `Polyrust.instDecidableEqVExpr.decEq._proof_30` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.ite a_1 a_2 = a_3.eqb a_4 → False |
| 438 | `Polyrust.instDecidableEqVExpr.decEq._proof_31` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 : Polyrust.VExpr), a.ite a_1 a_2 = a.ite a_1 a_2 |
| 439 | `Polyrust.instDecidableEqVExpr.decEq._proof_32` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b : Polyrust.VExpr), ¬a_2 = b → a.ite a_1 a_2 = a.ite a_1 b → False |
| 440 | `Polyrust.instDecidableEqVExpr.decEq._proof_33` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b b_1 : Polyrust.VExpr), ¬a_1 = b → a.ite a_1 a_2 = a.ite b b_1 → False |
| 441 | `Polyrust.instDecidableEqVExpr.decEq._proof_34` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b b_1 b_2 : Polyrust.VExpr), ¬a = b → a.ite a_1 a_2 = b.ite b_1 b_2 → False |
| 442 | `Polyrust.instDecidableEqVExpr.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : Nat) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.var a = a_1.add a_2 → False |
| 443 | `Polyrust.instDecidableEqVExpr.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a : Nat) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.var a = a_1.eqb a_2 → False |
| 444 | `Polyrust.instDecidableEqVExpr.decEq._proof_6` | 實例衍生 | 定理 | | ∀ (a : Nat) (a_1 a_2 a_3 : Polyrust.VExpr), Polyrust.VExpr.var a = a_1.ite a_2 a_3 → False |
| 445 | `Polyrust.instDecidableEqVExpr.decEq._proof_7` | 實例衍生 | 定理 | | ∀ (a : Int) (a_1 : Nat), Polyrust.VExpr.num a = Polyrust.VExpr.var a_1 → False |
| 446 | `Polyrust.instDecidableEqVExpr.decEq._proof_8` | 實例衍生 | 定理 | | ∀ (a : Int), Polyrust.VExpr.num a = Polyrust.VExpr.num a |
| 447 | `Polyrust.instDecidableEqVExpr.decEq._proof_9` | 實例衍生 | 定理 | | ∀ (a b : Int), ¬a = b → Polyrust.VExpr.num a = Polyrust.VExpr.num b → False |
| 448 | `Polyrust.instDecidableEqVExpr.decEq._sunfold` | 實例衍生 | 定義 | | (x x_1 : Polyrust.VExpr) → Decidable (x = x_1) |
| 449 | `Polyrust.instDecidableEqVExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | | (x x_1 : Polyrust.VExpr) → Decidable (x = x_1) |
| 450 | `Polyrust.instDecidableEqVExpr.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.VExpr → Polyrust.VExpr → Sort u_1) →   (x x_1 : Polyrust.VExpr) →     ((a b : Nat) → motive (Polyrust.VExpr.var a) (Polyrust.VExpr. |
| 451 | `Polyrust.instReprVExpr.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.VExpr → Sort u_1) →   (x : Polyrust.VExpr) →     ((a : Nat) → motive (Polyrust.VExpr.var a)) →       ((a : Int) → motive (Polyrust. |
| 452 | `Polyrust.subst._f` | 等式引理／助手 | 定義 | | (Nat → Polyrust.VExpr) → (x : Polyrust.VExpr) → Polyrust.VExpr.below x → Polyrust.VExpr |
| 453 | `Polyrust.subst._sunfold` | 等式引理／助手 | 定義 | | (Nat → Polyrust.VExpr) → Polyrust.VExpr → Polyrust.VExpr |
| 454 | `Polyrust.subst._unsafe_rec` | 等式引理／助手 | 定義 | | (Nat → Polyrust.VExpr) → Polyrust.VExpr → Polyrust.VExpr |
| 455 | `Polyrust.subst.eq_1` | 等式引理／助手 | 定理 | | ∀ (ρ : Nat → Polyrust.VExpr) (a : Nat), Polyrust.subst ρ (Polyrust.VExpr.var a) = ρ a |
| 456 | `Polyrust.subst.eq_2` | 等式引理／助手 | 定理 | | ∀ (ρ : Nat → Polyrust.VExpr) (a : Int), Polyrust.subst ρ (Polyrust.VExpr.num a) = Polyrust.VExpr.num a |
| 457 | `Polyrust.subst.eq_3` | 等式引理／助手 | 定理 | | ∀ (ρ : Nat → Polyrust.VExpr) (a a_1 : Polyrust.VExpr),   Polyrust.subst ρ (a.add a_1) = (Polyrust.subst ρ a).add (Polyrust.subst ρ a_1) |
| 458 | `Polyrust.subst.eq_4` | 等式引理／助手 | 定理 | | ∀ (ρ : Nat → Polyrust.VExpr) (a a_1 : Polyrust.VExpr),   Polyrust.subst ρ (a.eqb a_1) = (Polyrust.subst ρ a).eqb (Polyrust.subst ρ a_1) |
| 459 | `Polyrust.subst.eq_5` | 等式引理／助手 | 定理 | | ∀ (ρ : Nat → Polyrust.VExpr) (a a_1 a_2 : Polyrust.VExpr),   Polyrust.subst ρ (a.ite a_1 a_2) = (Polyrust.subst ρ a).ite (Polyrust.subst ρ a_1) (Polyr |
| 460 | `Polyrust.subst.eq_def` | 等式引理／助手 | 定理 | | ∀ (ρ : Nat → Polyrust.VExpr) (x : Polyrust.VExpr),   Polyrust.subst ρ x =     match x with     \| Polyrust.VExpr.var v => ρ v     \| Polyrust.VExpr.num  |
| 461 | `Polyrust.vars._f` | 等式引理／助手 | 定義 | | (x : Polyrust.VExpr) → Polyrust.VExpr.below x → List Nat |
| 462 | `Polyrust.vars._sunfold` | 等式引理／助手 | 定義 | | Polyrust.VExpr → List Nat |
| 463 | `Polyrust.vars._unsafe_rec` | 等式引理／助手 | 定義 | | Polyrust.VExpr → List Nat |
| 464 | `Polyrust.vars.eq_1` | 等式引理／助手 | 定理 | | ∀ (a : Nat), Polyrust.vars (Polyrust.VExpr.var a) = [a] |
| 465 | `Polyrust.vars.eq_2` | 等式引理／助手 | 定理 | | ∀ (a : Int), Polyrust.vars (Polyrust.VExpr.num a) = [] |
| 466 | `Polyrust.vars.eq_3` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.VExpr), Polyrust.vars (a.add a_1) = Polyrust.vars a ++ Polyrust.vars a_1 |
| 467 | `Polyrust.vars.eq_4` | 等式引理／助手 | 定理 | | ∀ (a a_1 : Polyrust.VExpr), Polyrust.vars (a.eqb a_1) = Polyrust.vars a ++ Polyrust.vars a_1 |
| 468 | `Polyrust.vars.eq_5` | 等式引理／助手 | 定理 | | ∀ (a a_1 a_2 : Polyrust.VExpr),   Polyrust.vars (a.ite a_1 a_2) = Polyrust.vars a ++ Polyrust.vars a_1 ++ Polyrust.vars a_2 |
| 469 | `Polyrust.vars.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.VExpr),   Polyrust.vars x =     match x with     \| Polyrust.VExpr.var v => [v]     \| Polyrust.VExpr.num a => []     \| a.add b => Polyr |

## `Polyrust.OpAbstraction`（107 條）

職責：運算子規則抽象：二元運算子抽象為 BinSpec

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 470 | `Polyrust.BinSpec.mk.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {in1 in2 out in1_1 in2_1 out_1 : Ty},   { in1 := in1, in2 := in2, out := out } = { in1 := in1_1, in2 := in2_1, out := out_1 } →     in1  |
| 471 | `Polyrust.BinSpec.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (in1 in2 out : Ty),   sizeOf { in1 := in1, in2 := in2, out := out } = 1 + sizeOf in1 + sizeOf in2 + sizeOf out |
| 472 | `Polyrust.ExprG.binop.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       t.ctorIdx = 1 →         ((a : Polyrust.BinSpec Ty) → (a_1  |
| 473 | `Polyrust.ExprG.binop.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : Polyrust.BinSpec Ty} {a_1 a_2 : Polyrust.ExprG Ty} {a_3 : Polyrust.BinSpec Ty}   {a_4 a_5 : Polyrust.ExprG Ty},   Polyrust.ExprG.bi |
| 474 | `Polyrust.ExprG.binop.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty),   sizeOf (Polyrust.ExprG.binop a a_1 a_2) = 1 + sizeOf a + s |
| 475 | `Polyrust.ExprG.ite.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) → t.ctorIdx = 2 → ((a a_1 a_2 : Polyrust.ExprG Ty) → motive (a.ite |
| 476 | `Polyrust.ExprG.ite.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 a_2 a_3 a_4 a_5 : Polyrust.ExprG Ty},   a.ite a_1 a_2 = a_3.ite a_4 a_5 → a = a_3 ∧ a_1 = a_4 ∧ a_2 = a_5 |
| 477 | `Polyrust.ExprG.ite.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 a_2 : Polyrust.ExprG Ty),   sizeOf (a.ite a_1 a_2) = 1 + sizeOf a + sizeOf a_1 + sizeOf a_2 |
| 478 | `Polyrust.ExprG.num.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) → t.ctorIdx = 0 → ((a : Int) → motive (Polyrust.ExprG.num a)) → mo |
| 479 | `Polyrust.ExprG.num.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : Int}, Polyrust.ExprG.num a = Polyrust.ExprG.num a_1 → a = a_1 |
| 480 | `Polyrust.ExprG.num.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Int), sizeOf (Polyrust.ExprG.num a) = 1 + sizeOf a |
| 481 | `Polyrust.IsRootG2` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Polyrust.SigmaG2 Ty → Prop |
| 482 | `Polyrust.SigmaG2` | 手寫 | 定義 | | Type → Type |
| 483 | `Polyrust.TypableG2` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Prop |
| 484 | `Polyrust.andSpec` | 手寫 | 定義 | | Polyrust.Lang Polyrust.Ty → Polyrust.BinSpec Polyrust.Ty |
| 485 | `Polyrust.arithSpec` | 手寫 | 定義 | | Polyrust.Lang Polyrust.Ty → Polyrust.BinSpec Polyrust.Ty |
| 486 | `Polyrust.cBinop` | 手寫 | 定義 | | {Ty : Type} →   [DecidableEq Ty] → Polyrust.BinSpec Ty → Polyrust.ExprG Ty → Polyrust.ExprG Ty → Ty → Polyrust.SigmaG2 Ty → Int |
| 487 | `Polyrust.cIte2` | 手寫 | 定義 | | {Ty : Type} →   Polyrust.Lang Ty → Polyrust.ExprG Ty → Polyrust.ExprG Ty → Polyrust.ExprG Ty → Ty → Polyrust.SigmaG2 Ty → Int |
| 488 | `Polyrust.cNum2` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Int → Ty → Polyrust.SigmaG2 Ty → Int |
| 489 | `Polyrust.cmpSpec` | 手寫 | 定義 | | Polyrust.Lang Polyrust.Ty → Polyrust.BinSpec Polyrust.Ty |
| 490 | `Polyrust.genCG2` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → List (Polyrust.SigmaG2 Ty → Int) |
| 491 | `Polyrust.oneHotC2` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Polyrust.ExprG Ty → Polyrust.SigmaG2 Ty → Int |
| 492 | `Polyrust.oneHotG2` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Polyrust.SigmaG2 Ty → Polyrust.ExprG Ty → Int |
| 493 | `Polyrust.outMark` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.BinSpec Ty → Ty → Int |
| 494 | `Polyrust.tbG2` | 手寫 | 定義 | | {Ty : Type} → Polyrust.SigmaG2 Ty → Polyrust.ExprG Ty → Ty → Int |
| 495 | `Polyrust.tycheckG` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Ty → Bool |
| 496 | `Polyrust.witnessG2` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SigmaG2 Ty |
| 497 | `Polyrust.BinSpec._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.BinSpec Ty → Nat |
| 498 | `Polyrust.BinSpec._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.BinSpec Ty) |
| 499 | `Polyrust.BinSpec.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.BinSpec Ty → Sort u} →     (t : Polyrust.BinSpec Ty) → ((in1 in2 out : Ty) → motive { in1 := in1, in2 := in2, out : |
| 500 | `Polyrust.BinSpec.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.BinSpec Ty → Nat |
| 501 | `Polyrust.BinSpec.in1` | 衍生 | 定義 | | {Ty : Type} → Polyrust.BinSpec Ty → Ty |
| 502 | `Polyrust.BinSpec.in2` | 衍生 | 定義 | | {Ty : Type} → Polyrust.BinSpec Ty → Ty |
| 503 | `Polyrust.BinSpec.mk._flat_ctor` | 等式引理／助手 | 定義 | | {Ty : Type} → Ty → Ty → Ty → Polyrust.BinSpec Ty |
| 504 | `Polyrust.BinSpec.mk.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {in1 in2 out in1' in2' out' : Ty} →       { in1 := in1, in2 := in2, out := out } = { in1 := in1', in2 := in2', out  |
| 505 | `Polyrust.BinSpec.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.BinSpec Ty} →       {Ty' : Type} → {t' : Polyrust.BinSpec Ty'} → Ty = Ty' → t ≍ t' → Polyrust.BinSpec |
| 506 | `Polyrust.BinSpec.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.BinSpec Ty → {Ty' : Type} → Polyrust.BinSpec Ty' → Sort u |
| 507 | `Polyrust.BinSpec.out` | 衍生 | 定義 | | {Ty : Type} → Polyrust.BinSpec Ty → Ty |
| 508 | `Polyrust.BinSpec.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.BinSpec Ty → Sort u} →     (t : Polyrust.BinSpec Ty) → ((in1 in2 out : Ty) → motive { in1 := in1, in2 := in2, out : |
| 509 | `Polyrust.ExprG._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.ExprG Ty → Nat |
| 510 | `Polyrust.ExprG._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.ExprG Ty) |
| 511 | `Polyrust.ExprG.below` | 歸納型衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.ExprG Ty → Sort u} → Polyrust.ExprG Ty → Sort (max 1 u) |
| 512 | `Polyrust.ExprG.binop.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : Polyrust.BinSpec Ty} →       {a_1 a_2 : Polyrust.ExprG Ty} →         {a' : Polyrust.BinSpec Ty} →           {a |
| 513 | `Polyrust.ExprG.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) → ((t : Polyrust.ExprG Ty) → Polyrust.ExprG.below t → motive t) →  |
| 514 | `Polyrust.ExprG.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive : Polyrust.ExprG Ty → Sort u} (t : Polyrust.ExprG Ty)   (F_1 : (t : Polyrust.ExprG Ty) → Polyrust.ExprG.below t → motive t),   P |
| 515 | `Polyrust.ExprG.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       ((t : Polyrust.ExprG Ty) → Polyrust.ExprG.below t → motive |
| 516 | `Polyrust.ExprG.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       ((a : Int) → motive (Polyrust.ExprG.num a)) →         ((a  |
| 517 | `Polyrust.ExprG.ctorElim` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.ExprG Ty) → ctorIdx = t.ctorIdx → Polyrust.ExprG.ctorElimT |
| 518 | `Polyrust.ExprG.ctorElimType` | 衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.ExprG Ty → Sort u} → Nat → Sort (max 1 u) |
| 519 | `Polyrust.ExprG.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.ExprG Ty → Nat |
| 520 | `Polyrust.ExprG.ite.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a a_1 a_2 a' a'_1 a'_2 : Polyrust.ExprG Ty} →       a.ite a_1 a_2 = a'.ite a'_1 a'_2 → (a ≍ a' → a_1 ≍ a'_1 → a_2  |
| 521 | `Polyrust.ExprG.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.ExprG Ty} →       {Ty' : Type} → {t' : Polyrust.ExprG Ty'} → Ty = Ty' → t ≍ t' → Polyrust.ExprG.noCon |
| 522 | `Polyrust.ExprG.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.ExprG Ty → {Ty' : Type} → Polyrust.ExprG Ty' → Sort u |
| 523 | `Polyrust.ExprG.num.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} → {P : Sort u} → {a a' : Int} → Polyrust.ExprG.num a = Polyrust.ExprG.num a' → (a = a' → P) → P |
| 524 | `Polyrust.ExprG.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       ((a : Int) → motive (Polyrust.ExprG.num a)) →         ((a  |
| 525 | `Polyrust.genCG2._f` | 等式引理／助手 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty → (x : Polyrust.ExprG Ty) → Polyrust.ExprG.below x → List (Polyrust.SigmaG2 Ty → Int) |
| 526 | `Polyrust.genCG2._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → List (Polyrust.SigmaG2 Ty → Int) |
| 527 | `Polyrust.genCG2._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → List (Polyrust.SigmaG2 Ty → Int) |
| 528 | `Polyrust.genCG2.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.ExprG Ty),   Polyrust.genCG2 L x =     match x with     \| Polyrust.ExprG.nu |
| 529 | `Polyrust.instDecidableEqBinSpec` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.BinSpec Ty) |
| 530 | `Polyrust.instDecidableEqBinSpec.decEq` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.BinSpec Ty) → Decidable (x = x_1) |
| 531 | `Polyrust.instDecidableEqBinSpec.decEq._proof_1` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 : Ty), { in1 := a, in2 := a_1, out := a_2 } = { in1 := a, in2 := a_1, out := a_2 } |
| 532 | `Polyrust.instDecidableEqBinSpec.decEq._proof_2` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 b : Ty),   ¬a_2 = b → { in1 := a, in2 := a_1, out := a_2 } = { in1 := a, in2 := a_1, out := b } → False |
| 533 | `Polyrust.instDecidableEqBinSpec.decEq._proof_3` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 b b_1 : Ty),   ¬a_1 = b → { in1 := a, in2 := a_1, out := a_2 } = { in1 := a, in2 := b, out := b_1 } → False |
| 534 | `Polyrust.instDecidableEqBinSpec.decEq._proof_4` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 b b_1 b_2 : Ty),   ¬a = b → { in1 := a, in2 := a_1, out := a_2 } = { in1 := b, in2 := b_1, out := b_2 } → False |
| 535 | `Polyrust.instDecidableEqBinSpec.decEq.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.BinSpec Ty → Polyrust.BinSpec Ty → Sort u_1) →     (x x_1 : Polyrust.BinSpec Ty) →       ((a a_1 a_2 b b_1 b_2 : Ty |
| 536 | `Polyrust.instDecidableEqExprG` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.ExprG Ty) |
| 537 | `Polyrust.instDecidableEqExprG.decEq` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ExprG Ty) → Decidable (x = x_1) |
| 538 | `Polyrust.instDecidableEqExprG.decEq._f` | 實例衍生 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     (x : Polyrust.ExprG Ty) →       Polyrust.ExprG.below (motive := fun x => (x_1 : Polyrust.ExprG Ty) → Decidable  |
| 539 | `Polyrust.instDecidableEqExprG.decEq._proof_1` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Int), Polyrust.ExprG.num a = Polyrust.ExprG.num a |
| 540 | `Polyrust.instDecidableEqExprG.decEq._proof_10` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 a_3 a_4 a_5 : Polyrust.ExprG Ty),   Polyrust.ExprG.binop a a_1 a_2 = a_3.ite a_4 a_5 → False |
| 541 | `Polyrust.instDecidableEqExprG.decEq._proof_11` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 : Polyrust.ExprG Ty) (a_3 : Int), a.ite a_1 a_2 = Polyrust.ExprG.num a_3 → False |
| 542 | `Polyrust.instDecidableEqExprG.decEq._proof_12` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 : Polyrust.ExprG Ty) (a_3 : Polyrust.BinSpec Ty) (a_4 a_5 : Polyrust.ExprG Ty),   a.ite a_1 a_2 = Polyrust.ExprG.binop a_3 a_ |
| 543 | `Polyrust.instDecidableEqExprG.decEq._proof_13` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 : Polyrust.ExprG Ty), a.ite a_1 a_2 = a.ite a_1 a_2 |
| 544 | `Polyrust.instDecidableEqExprG.decEq._proof_14` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 b : Polyrust.ExprG Ty), ¬a_2 = b → a.ite a_1 a_2 = a.ite a_1 b → False |
| 545 | `Polyrust.instDecidableEqExprG.decEq._proof_15` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 b b_1 : Polyrust.ExprG Ty), ¬a_1 = b → a.ite a_1 a_2 = a.ite b b_1 → False |
| 546 | `Polyrust.instDecidableEqExprG.decEq._proof_16` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 a_2 b b_1 b_2 : Polyrust.ExprG Ty), ¬a = b → a.ite a_1 a_2 = b.ite b_1 b_2 → False |
| 547 | `Polyrust.instDecidableEqExprG.decEq._proof_2` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a b : Int), ¬a = b → Polyrust.ExprG.num a = Polyrust.ExprG.num b → False |
| 548 | `Polyrust.instDecidableEqExprG.decEq._proof_3` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Int) (a_1 : Polyrust.BinSpec Ty) (a_2 a_3 : Polyrust.ExprG Ty),   Polyrust.ExprG.num a = Polyrust.ExprG.binop a_1 a_2 a_3 → False |
| 549 | `Polyrust.instDecidableEqExprG.decEq._proof_4` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Int) (a_1 a_2 a_3 : Polyrust.ExprG Ty), Polyrust.ExprG.num a = a_1.ite a_2 a_3 → False |
| 550 | `Polyrust.instDecidableEqExprG.decEq._proof_5` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty) (a_3 : Int),   Polyrust.ExprG.binop a a_1 a_2 = Polyrust.ExprG.num a_3 → False |
| 551 | `Polyrust.instDecidableEqExprG.decEq._proof_6` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty),   Polyrust.ExprG.binop a a_1 a_2 = Polyrust.ExprG.binop a a_1 a_2 |
| 552 | `Polyrust.instDecidableEqExprG.decEq._proof_7` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 b : Polyrust.ExprG Ty),   ¬a_2 = b → Polyrust.ExprG.binop a a_1 a_2 = Polyrust.ExprG.binop a a_1 b →  |
| 553 | `Polyrust.instDecidableEqExprG.decEq._proof_8` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 b b_1 : Polyrust.ExprG Ty),   ¬a_1 = b → Polyrust.ExprG.binop a a_1 a_2 = Polyrust.ExprG.binop a b b_ |
| 554 | `Polyrust.instDecidableEqExprG.decEq._proof_9` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty) (b : Polyrust.BinSpec Ty)   (b_1 b_2 : Polyrust.ExprG Ty), ¬a = b → Polyrust.Exp |
| 555 | `Polyrust.instDecidableEqExprG.decEq._sunfold` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ExprG Ty) → Decidable (x = x_1) |
| 556 | `Polyrust.instDecidableEqExprG.decEq._unsafe_rec` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ExprG Ty) → Decidable (x = x_1) |
| 557 | `Polyrust.instDecidableEqExprG.decEq.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.ExprG Ty → Polyrust.ExprG Ty → Sort u_1) →     (x x_1 : Polyrust.ExprG Ty) →       ((a b : Int) → motive (Polyrust. |
| 558 | `Polyrust.instReprBinSpec` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Repr (Polyrust.BinSpec Ty) |
| 559 | `Polyrust.instReprBinSpec.repr` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.BinSpec Ty → Nat → Format |
| 560 | `Polyrust.instReprExprG.repr.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.ExprG Ty → Sort u_1) →     (x : Polyrust.ExprG Ty) →       ((a : Int) → motive (Polyrust.ExprG.num a)) →         (( |
| 561 | `Polyrust.isMonoAtG2_of_root.match_1_3` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} {e : Polyrust.ExprG Ty} {σ : Polyrust.SigmaG2 Ty} (τ τ' : Ty)   (motive : σ e τ = true ∧ σ e τ' = true → Prop) (h : σ e τ = true ∧ σ e τ |
| 562 | `Polyrust.oneHotC2.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} (L : Polyrust.Lang Ty) (e : Polyrust.ExprG Ty) (σ : Polyrust.SigmaG2 Ty),   Polyrust.oneHotC2 L e σ = Polyrust.oneHotG2 L σ e |
| 563 | `Polyrust.outMark.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (s : Polyrust.BinSpec Ty) (t : Ty),   Polyrust.outMark s t = Polyrust.bit (decide (t = s.out)) |
| 564 | `Polyrust.tycheckG._f` | 等式引理／助手 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty → (x : Polyrust.ExprG Ty) → Polyrust.ExprG.below (motive := fun x => Ty → Bool) x → Ty → Bool |
| 565 | `Polyrust.tycheckG._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Ty → Bool |
| 566 | `Polyrust.tycheckG._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Ty → Bool |
| 567 | `Polyrust.tycheckG.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a : Int),   Polyrust.tycheckG L (Polyrust.ExprG.num a) x = decide (x = L.numTy) |
| 568 | `Polyrust.tycheckG.eq_2` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (s : Polyrust.BinSpec Ty)   (a b : Polyrust.ExprG Ty),   Polyrust.tycheckG L (Po |
| 569 | `Polyrust.tycheckG.eq_3` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (c t f : Polyrust.ExprG Ty),   Polyrust.tycheckG L (c.ite t f) x =     (Polyrust |
| 570 | `Polyrust.tycheckG.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.ExprG Ty) (x_1 : Ty),   Polyrust.tycheckG L x x_1 =     match x, x_1 with   |
| 571 | `Polyrust.tycheckG.match_1` | 等式引理／助手 | 定義 | | {Ty : Type} →   (motive : Polyrust.ExprG Ty → Ty → Sort u_1) →     (x : Polyrust.ExprG Ty) →       (x_1 : Ty) →         ((a : Int) → (τ : Ty) → motive |
| 572 | `Polyrust.tycheckG_exclusive.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (n : Int) (τ τ' : Ty)   (motive :     Polyrust.tycheckG L (Polyrust.ExprG.num n) τ = true |
| 573 | `Polyrust.tycheckG_exclusive.match_1_3` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (s : Polyrust.BinSpec Ty) (a b : Polyrust.ExprG Ty)   (τ τ' : Ty)   (motive :     Polyrus |
| 574 | `Polyrust.tycheckG_exclusive.match_1_5` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (c t f : Polyrust.ExprG Ty) (τ τ' : Ty)   (motive : Polyrust.tycheckG L (c.ite t f) τ = t |
| 575 | `Polyrust.typable_iff_rootG2.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.ExprG Ty)   (motive : (∃ σ, Polyrust.IsRootG2 L e σ) → Prop) (h : ∃ σ, Poly |
| 576 | `Polyrust.untypable_iff_no_rootG2.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.ExprG Ty)   (motive : (∃ σ, Polyrust.IsRootG2 L e σ) → Prop) (h : ∃ σ, Poly |

## `Polyrust.SumReduction`（106 條）

職責：和型歸約：inl/inr/和型檢查與可定型性歸約為變體檢查

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 577 | `Polyrust.SumExpr.inl.elim` | 手寫 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → t.ctorIdx = 1 → ((a : Polyrust.SumExpr Expr) → motive  |
| 578 | `Polyrust.SumExpr.inl.inj` | 手寫 | 定理 | | ∀ {Expr : Type} {a a_1 : Polyrust.SumExpr Expr}, a.inl = a_1.inl → a = a_1 |
| 579 | `Polyrust.SumExpr.inl.sizeOf_spec` | 手寫 | 定理 | | ∀ {Expr : Type} [inst : SizeOf Expr] (a : Polyrust.SumExpr Expr), sizeOf a.inl = 1 + sizeOf a |
| 580 | `Polyrust.SumExpr.inr.elim` | 手寫 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → t.ctorIdx = 2 → ((a : Polyrust.SumExpr Expr) → motive  |
| 581 | `Polyrust.SumExpr.inr.inj` | 手寫 | 定理 | | ∀ {Expr : Type} {a a_1 : Polyrust.SumExpr Expr}, a.inr = a_1.inr → a = a_1 |
| 582 | `Polyrust.SumExpr.inr.sizeOf_spec` | 手寫 | 定理 | | ∀ {Expr : Type} [inst : SizeOf Expr] (a : Polyrust.SumExpr Expr), sizeOf a.inr = 1 + sizeOf a |
| 583 | `Polyrust.SumExpr.lift.elim` | 手寫 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → t.ctorIdx = 0 → ((a : Expr) → motive (Polyrust.SumExpr |
| 584 | `Polyrust.SumExpr.lift.inj` | 手寫 | 定理 | | ∀ {Expr : Type} {a a_1 : Expr}, Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift a_1 → a = a_1 |
| 585 | `Polyrust.SumExpr.lift.sizeOf_spec` | 手寫 | 定理 | | ∀ {Expr : Type} [inst : SizeOf Expr] (a : Expr), sizeOf (Polyrust.SumExpr.lift a) = 1 + sizeOf a |
| 586 | `Polyrust.SumTy.base.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) → t.ctorIdx = 0 → ((a : Ty) → motive (Polyrust.SumTy.base a)) → mo |
| 587 | `Polyrust.SumTy.base.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : Ty}, Polyrust.SumTy.base a = Polyrust.SumTy.base a_1 → a = a_1 |
| 588 | `Polyrust.SumTy.base.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Ty), sizeOf (Polyrust.SumTy.base a) = 1 + sizeOf a |
| 589 | `Polyrust.SumTy.sum.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) → t.ctorIdx = 1 → ((a a_1 : Polyrust.SumTy Ty) → motive (a.sum a_1 |
| 590 | `Polyrust.SumTy.sum.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 a_2 a_3 : Polyrust.SumTy Ty}, a.sum a_1 = a_2.sum a_3 → a = a_2 ∧ a_1 = a_3 |
| 591 | `Polyrust.SumTy.sum.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 : Polyrust.SumTy Ty), sizeOf (a.sum a_1) = 1 + sizeOf a + sizeOf a_1 |
| 592 | `Polyrust.TypableSum` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Prop |
| 593 | `Polyrust.TypeTypable` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumTy Ty → Prop |
| 594 | `Polyrust.sumBits` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → (Ty → Bool) → (Ty → Bool) → List Int |
| 595 | `Polyrust.tycheckSum` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Polyrust.SumTy Ty → Bool |
| 596 | `Polyrust.tycheck_inl` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {a : Polyrust.SumExpr Polyrust.Expr}   {τ₁ τ₂ : Polyrust.SumTy Ty}, Polyrust.tycheckSum L |
| 597 | `Polyrust.tycheck_inr` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {b : Polyrust.SumExpr Polyrust.Expr}   {τ₁ τ₂ : Polyrust.SumTy Ty}, Polyrust.tycheckSum L |
| 598 | `Polyrust.SumExpr._sizeOf_1` | 歸納型衍生 | 定義 | | {Expr : Type} → [SizeOf Expr] → Polyrust.SumExpr Expr → Nat |
| 599 | `Polyrust.SumExpr._sizeOf_inst` | 歸納型衍生 | 定義 | | (Expr : Type) → [SizeOf Expr] → SizeOf (Polyrust.SumExpr Expr) |
| 600 | `Polyrust.SumExpr.below` | 歸納型衍生 | 定義 | | {Expr : Type} → {motive : Polyrust.SumExpr Expr → Sort u} → Polyrust.SumExpr Expr → Sort (max 1 u) |
| 601 | `Polyrust.SumExpr.brecOn` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → ((t : Polyrust.SumExpr Expr) → Polyrust.SumExpr.below  |
| 602 | `Polyrust.SumExpr.brecOn.eq` | 衍生 | 定理 | | ∀ {Expr : Type} {motive : Polyrust.SumExpr Expr → Sort u} (t : Polyrust.SumExpr Expr)   (F_1 : (t : Polyrust.SumExpr Expr) → Polyrust.SumExpr.below t  |
| 603 | `Polyrust.SumExpr.brecOn.go` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) →       ((t : Polyrust.SumExpr Expr) → Polyrust.SumExpr. |
| 604 | `Polyrust.SumExpr.casesOn` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) →       ((a : Expr) → motive (Polyrust.SumExpr.lift a))  |
| 605 | `Polyrust.SumExpr.ctorElim` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (ctorIdx : Nat) →       (t : Polyrust.SumExpr Expr) → ctorIdx = t.ctorIdx → Polyrust |
| 606 | `Polyrust.SumExpr.ctorElimType` | 衍生 | 定義 | | {Expr : Type} → {motive : Polyrust.SumExpr Expr → Sort u} → Nat → Sort (max 1 u) |
| 607 | `Polyrust.SumExpr.ctorIdx` | 歸納型衍生 | 定義 | | {Expr : Type} → Polyrust.SumExpr Expr → Nat |
| 608 | `Polyrust.SumExpr.inl.noConfusion` | 歸納型衍生 | 定義 | | {Expr : Type} → {P : Sort u} → {a a' : Polyrust.SumExpr Expr} → a.inl = a'.inl → (a ≍ a' → P) → P |
| 609 | `Polyrust.SumExpr.inr.noConfusion` | 歸納型衍生 | 定義 | | {Expr : Type} → {P : Sort u} → {a a' : Polyrust.SumExpr Expr} → a.inr = a'.inr → (a ≍ a' → P) → P |
| 610 | `Polyrust.SumExpr.lift.noConfusion` | 歸納型衍生 | 定義 | | {Expr : Type} → {P : Sort u} → {a a' : Expr} → Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift a' → (a ≍ a' → P) → P |
| 611 | `Polyrust.SumExpr.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Expr : Type} →     {t : Polyrust.SumExpr Expr} →       {Expr' : Type} → {t' : Polyrust.SumExpr Expr'} → Expr = Expr' → t ≍ t' → Poly |
| 612 | `Polyrust.SumExpr.noConfusionType` | 衍生 | 定義 | | Sort u → {Expr : Type} → Polyrust.SumExpr Expr → {Expr' : Type} → Polyrust.SumExpr Expr' → Sort u |
| 613 | `Polyrust.SumExpr.recOn` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) →       ((a : Expr) → motive (Polyrust.SumExpr.lift a))  |
| 614 | `Polyrust.SumTy._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.SumTy Ty → Nat |
| 615 | `Polyrust.SumTy._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.SumTy Ty) |
| 616 | `Polyrust.SumTy.base.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} → {P : Sort u} → {a a' : Ty} → Polyrust.SumTy.base a = Polyrust.SumTy.base a' → (a ≍ a' → P) → P |
| 617 | `Polyrust.SumTy.below` | 歸納型衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.SumTy Ty → Sort u} → Polyrust.SumTy Ty → Sort (max 1 u) |
| 618 | `Polyrust.SumTy.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) → ((t : Polyrust.SumTy Ty) → Polyrust.SumTy.below t → motive t) →  |
| 619 | `Polyrust.SumTy.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive : Polyrust.SumTy Ty → Sort u} (t : Polyrust.SumTy Ty)   (F_1 : (t : Polyrust.SumTy Ty) → Polyrust.SumTy.below t → motive t),   P |
| 620 | `Polyrust.SumTy.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) →       ((t : Polyrust.SumTy Ty) → Polyrust.SumTy.below t → motive |
| 621 | `Polyrust.SumTy.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) →       ((a : Ty) → motive (Polyrust.SumTy.base a)) → ((a a_1 : Po |
| 622 | `Polyrust.SumTy.ctorElim` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.SumTy Ty) → ctorIdx = t.ctorIdx → Polyrust.SumTy.ctorElimT |
| 623 | `Polyrust.SumTy.ctorElimType` | 衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.SumTy Ty → Sort u} → Nat → Sort (max 1 u) |
| 624 | `Polyrust.SumTy.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.SumTy Ty → Nat |
| 625 | `Polyrust.SumTy.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.SumTy Ty} →       {Ty' : Type} → {t' : Polyrust.SumTy Ty'} → Ty = Ty' → t ≍ t' → Polyrust.SumTy.noCon |
| 626 | `Polyrust.SumTy.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.SumTy Ty → {Ty' : Type} → Polyrust.SumTy Ty' → Sort u |
| 627 | `Polyrust.SumTy.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) →       ((a : Ty) → motive (Polyrust.SumTy.base a)) →         ((a  |
| 628 | `Polyrust.SumTy.sum.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} → {a a_1 a' a'_1 : Polyrust.SumTy Ty} → a.sum a_1 = a'.sum a'_1 → (a ≍ a' → a_1 ≍ a'_1 → P) → P |
| 629 | `Polyrust.instDecidableEqSumExpr` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → DecidableEq (Polyrust.SumExpr Expr) |
| 630 | `Polyrust.instDecidableEqSumExpr.decEq` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.SumExpr Expr) → Decidable (x = x_1) |
| 631 | `Polyrust.instDecidableEqSumExpr.decEq._f` | 實例衍生 | 定義 | | {Expr : Type} →   [DecidableEq Expr] →     (x : Polyrust.SumExpr Expr) →       Polyrust.SumExpr.below (motive := fun x => (x_1 : Polyrust.SumExpr Expr |
| 632 | `Polyrust.instDecidableEqSumExpr.decEq._proof_1` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Expr), Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift a |
| 633 | `Polyrust.instDecidableEqSumExpr.decEq._proof_10` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a a_1 : Polyrust.SumExpr Expr), a.inr = a_1.inl → False |
| 634 | `Polyrust.instDecidableEqSumExpr.decEq._proof_11` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Polyrust.SumExpr Expr), a.inr = a.inr |
| 635 | `Polyrust.instDecidableEqSumExpr.decEq._proof_12` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a b : Polyrust.SumExpr Expr), ¬a = b → a.inr = b.inr → False |
| 636 | `Polyrust.instDecidableEqSumExpr.decEq._proof_2` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a b : Expr), ¬a = b → Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift b → False |
| 637 | `Polyrust.instDecidableEqSumExpr.decEq._proof_3` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Expr) (a_1 : Polyrust.SumExpr Expr), Polyrust.SumExpr.lift a = a_1.inl → False |
| 638 | `Polyrust.instDecidableEqSumExpr.decEq._proof_4` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Expr) (a_1 : Polyrust.SumExpr Expr), Polyrust.SumExpr.lift a = a_1.inr → False |
| 639 | `Polyrust.instDecidableEqSumExpr.decEq._proof_5` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Polyrust.SumExpr Expr) (a_1 : Expr), a.inl = Polyrust.SumExpr.lift a_1 → False |
| 640 | `Polyrust.instDecidableEqSumExpr.decEq._proof_6` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Polyrust.SumExpr Expr), a.inl = a.inl |
| 641 | `Polyrust.instDecidableEqSumExpr.decEq._proof_7` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a b : Polyrust.SumExpr Expr), ¬a = b → a.inl = b.inl → False |
| 642 | `Polyrust.instDecidableEqSumExpr.decEq._proof_8` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a a_1 : Polyrust.SumExpr Expr), a.inl = a_1.inr → False |
| 643 | `Polyrust.instDecidableEqSumExpr.decEq._proof_9` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Polyrust.SumExpr Expr) (a_1 : Expr), a.inr = Polyrust.SumExpr.lift a_1 → False |
| 644 | `Polyrust.instDecidableEqSumExpr.decEq._sunfold` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.SumExpr Expr) → Decidable (x = x_1) |
| 645 | `Polyrust.instDecidableEqSumExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.SumExpr Expr) → Decidable (x = x_1) |
| 646 | `Polyrust.instDecidableEqSumExpr.decEq.match_1` | 實例衍生 | 定義 | | {Expr : Type} →   (motive : Polyrust.SumExpr Expr → Polyrust.SumExpr Expr → Sort u_1) →     (x x_1 : Polyrust.SumExpr Expr) →       ((a b : Expr) → mo |
| 647 | `Polyrust.instDecidableEqSumTy` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.SumTy Ty) |
| 648 | `Polyrust.instDecidableEqSumTy.decEq` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.SumTy Ty) → Decidable (x = x_1) |
| 649 | `Polyrust.instDecidableEqSumTy.decEq._f` | 實例衍生 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     (x : Polyrust.SumTy Ty) →       Polyrust.SumTy.below (motive := fun x => (x_1 : Polyrust.SumTy Ty) → Decidable  |
| 650 | `Polyrust.instDecidableEqSumTy.decEq._proof_1` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Ty), Polyrust.SumTy.base a = Polyrust.SumTy.base a |
| 651 | `Polyrust.instDecidableEqSumTy.decEq._proof_2` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a b : Ty), ¬a = b → Polyrust.SumTy.base a = Polyrust.SumTy.base b → False |
| 652 | `Polyrust.instDecidableEqSumTy.decEq._proof_3` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Ty) (a_1 a_2 : Polyrust.SumTy Ty), Polyrust.SumTy.base a = a_1.sum a_2 → False |
| 653 | `Polyrust.instDecidableEqSumTy.decEq._proof_4` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 : Polyrust.SumTy Ty) (a_2 : Ty), a.sum a_1 = Polyrust.SumTy.base a_2 → False |
| 654 | `Polyrust.instDecidableEqSumTy.decEq._proof_5` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 : Polyrust.SumTy Ty), a.sum a_1 = a.sum a_1 |
| 655 | `Polyrust.instDecidableEqSumTy.decEq._proof_6` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 b : Polyrust.SumTy Ty), ¬a_1 = b → a.sum a_1 = a.sum b → False |
| 656 | `Polyrust.instDecidableEqSumTy.decEq._proof_7` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 b b_1 : Polyrust.SumTy Ty), ¬a = b → a.sum a_1 = b.sum b_1 → False |
| 657 | `Polyrust.instDecidableEqSumTy.decEq._sunfold` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.SumTy Ty) → Decidable (x = x_1) |
| 658 | `Polyrust.instDecidableEqSumTy.decEq._unsafe_rec` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.SumTy Ty) → Decidable (x = x_1) |
| 659 | `Polyrust.instDecidableEqSumTy.decEq.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.SumTy Ty → Polyrust.SumTy Ty → Sort u_1) →     (x x_1 : Polyrust.SumTy Ty) →       ((a b : Ty) → motive (Polyrust.S |
| 660 | `Polyrust.instReprSumExpr` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Repr (Polyrust.SumExpr Expr) |
| 661 | `Polyrust.instReprSumExpr.repr` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Polyrust.SumExpr Expr → Nat → Format |
| 662 | `Polyrust.instReprSumExpr.repr._f` | 實例衍生 | 定義 | | {Expr : Type} →   [Repr Expr] → (x : Polyrust.SumExpr Expr) → Polyrust.SumExpr.below (motive := fun x => Nat → Format) x → Nat → Format |
| 663 | `Polyrust.instReprSumExpr.repr._sunfold` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Polyrust.SumExpr Expr → Nat → Format |
| 664 | `Polyrust.instReprSumExpr.repr._unsafe_rec` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Polyrust.SumExpr Expr → Nat → Format |
| 665 | `Polyrust.instReprSumExpr.repr.match_1` | 實例衍生 | 定義 | | {Expr : Type} →   (motive : Polyrust.SumExpr Expr → Sort u_1) →     (x : Polyrust.SumExpr Expr) →       ((a : Expr) → motive (Polyrust.SumExpr.lift a) |
| 666 | `Polyrust.instReprSumTy` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Repr (Polyrust.SumTy Ty) |
| 667 | `Polyrust.instReprSumTy.repr` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.SumTy Ty → Nat → Format |
| 668 | `Polyrust.instReprSumTy.repr._f` | 實例衍生 | 定義 | | {Ty : Type} →   [Repr Ty] → (x : Polyrust.SumTy Ty) → Polyrust.SumTy.below (motive := fun x => Nat → Format) x → Nat → Format |
| 669 | `Polyrust.instReprSumTy.repr._sunfold` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.SumTy Ty → Nat → Format |
| 670 | `Polyrust.instReprSumTy.repr._unsafe_rec` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.SumTy Ty → Nat → Format |
| 671 | `Polyrust.instReprSumTy.repr.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.SumTy Ty → Sort u_1) →     (x : Polyrust.SumTy Ty) →       ((a : Ty) → motive (Polyrust.SumTy.base a)) → ((a a_1 :  |
| 672 | `Polyrust.tycheckSum._f` | 等式引理／助手 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty →       (x : Polyrust.SumExpr Polyrust.Expr) →         Polyrust.SumExpr.below (motive := fun x |
| 673 | `Polyrust.tycheckSum._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Polyrust.SumTy Ty → Bool |
| 674 | `Polyrust.tycheckSum._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Polyrust.SumTy Ty → Bool |
| 675 | `Polyrust.tycheckSum.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (τ : Ty),   Polyrust.tycheckSum L (Polyrust.SumExpr.lift e) (Polyrust |
| 676 | `Polyrust.tycheckSum.eq_2` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.Expr) (a_1 a_2 : Polyrust.SumTy Ty),   Polyrust.tycheckSum L (Polyrust.SumE |
| 677 | `Polyrust.tycheckSum.eq_3` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.SumExpr Polyrust.Expr)   (τ₁ τ₂ : Polyrust.SumTy Ty), Polyrust.tycheckSum L |
| 678 | `Polyrust.tycheckSum.eq_4` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.SumExpr Polyrust.Expr) (a_1 : Ty),   Polyrust.tycheckSum L a.inl (Polyrust. |
| 679 | `Polyrust.tycheckSum.eq_5` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (b : Polyrust.SumExpr Polyrust.Expr)   (τ₁ τ₂ : Polyrust.SumTy Ty), Polyrust.tycheckSum L |
| 680 | `Polyrust.tycheckSum.eq_6` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.SumExpr Polyrust.Expr) (a_1 : Ty),   Polyrust.tycheckSum L a.inr (Polyrust. |
| 681 | `Polyrust.tycheckSum.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.SumExpr Polyrust.Expr)   (x_1 : Polyrust.SumTy Ty),   Polyrust.tycheckSum L |
| 682 | `Polyrust.tycheckSum.match_1` | 等式引理／助手 | 定義 | | {Ty : Type} →   (motive : Polyrust.SumExpr Polyrust.Expr → Polyrust.SumTy Ty → Sort u_1) →     (x : Polyrust.SumExpr Polyrust.Expr) →       (x_1 : Pol |

## `Polyrust.TraitImpl`（100 條）

職責：trait/impl 解析、方法歸約、存在量化

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 683 | `Polyrust.ImplDef.forTy` | 手寫 | 定義 | | Polyrust.ImplDef → String |
| 684 | `Polyrust.ImplDef.mk.inj` | 手寫 | 定理 | | ∀ {traitName : Option String} {forTy : String} {typeParams lifetimeParams : List String}   {methods : List Polyrust.ImplMethod} {traitName_1 : Option  |
| 685 | `Polyrust.ImplDef.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (traitName : Option String) (forTy : String) (typeParams lifetimeParams : List String)   (methods : List Polyrust.ImplMethod),   sizeOf       { trai |
| 686 | `Polyrust.ImplDef.traitName` | 手寫 | 定義 | | Polyrust.ImplDef → Option String |
| 687 | `Polyrust.ImplMethod.body` | 手寫 | 定義 | | Polyrust.ImplMethod → String |
| 688 | `Polyrust.ImplMethod.mk.inj` | 手寫 | 定理 | | ∀ {name retTy body name_1 retTy_1 body_1 : String},   { name := name, retTy := retTy, body := body } = { name := name_1, retTy := retTy_1, body := bod |
| 689 | `Polyrust.ImplMethod.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (name retTy body : String),   sizeOf { name := name, retTy := retTy, body := body } = 1 + sizeOf name + sizeOf retTy + sizeOf body |
| 690 | `Polyrust.MethodTable.addImpl` | 手寫 | 定義 | | Polyrust.MethodTable → Polyrust.ImplDef → Polyrust.MethodTable |
| 691 | `Polyrust.MethodTable.addTrait` | 手寫 | 定義 | | Polyrust.MethodTable → Polyrust.TraitDef → Polyrust.MethodTable |
| 692 | `Polyrust.MethodTable.empty` | 手寫 | 定義 | | Polyrust.MethodTable |
| 693 | `Polyrust.MethodTable.impls` | 手寫 | 定義 | | Polyrust.MethodTable → List Polyrust.ImplDef |
| 694 | `Polyrust.MethodTable.mk.inj` | 手寫 | 定理 | | ∀ {traits : List Polyrust.TraitDef} {impls : List Polyrust.ImplDef} {traits_1 : List Polyrust.TraitDef}   {impls_1 : List Polyrust.ImplDef},   { trait |
| 695 | `Polyrust.MethodTable.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (traits : List Polyrust.TraitDef) (impls : List Polyrust.ImplDef),   sizeOf { traits := traits, impls := impls } = 1 + sizeOf traits + sizeOf impls |
| 696 | `Polyrust.MethodTable.resolveMethod` | 手寫 | 定義 | | Polyrust.MethodTable → String → String → Option Polyrust.ImplMethod |
| 697 | `Polyrust.MethodTable.resolveTraitMethod` | 手寫 | 定義 | | Polyrust.MethodTable → String → String → Option Polyrust.TraitMethod |
| 698 | `Polyrust.MethodTable.traits` | 手寫 | 定義 | | Polyrust.MethodTable → List Polyrust.TraitDef |
| 699 | `Polyrust.MethodTable.typesImplementing` | 手寫 | 定義 | | Polyrust.MethodTable → String → List String |
| 700 | `Polyrust.TraitDef.mk.inj` | 手寫 | 定理 | | ∀ {name : String} {typeParams lifetimeParams : List String} {methods : List Polyrust.TraitMethod}   {supertraits : List String} {name_1 : String} {typ |
| 701 | `Polyrust.TraitDef.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (name : String) (typeParams lifetimeParams : List String) (methods : List Polyrust.TraitMethod)   (supertraits : List String),   sizeOf       { name |
| 702 | `Polyrust.TraitMethod.mk.inj` | 手寫 | 定理 | | ∀ {name retTy : String} {hasDefault : Bool} {name_1 retTy_1 : String} {hasDefault_1 : Bool},   { name := name, retTy := retTy, hasDefault := hasDefaul |
| 703 | `Polyrust.TraitMethod.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (name retTy : String) (hasDefault : Bool),   sizeOf { name := name, retTy := retTy, hasDefault := hasDefault } = 1 + sizeOf name + sizeOf retTy + si |
| 704 | `Polyrust.implMethodBar` | 手寫 | 定義 | | Polyrust.ImplMethod |
| 705 | `Polyrust.testImpl` | 手寫 | 定義 | | Polyrust.ImplDef |
| 706 | `Polyrust.testResolve` | 手寫 | 定義 | | Option Polyrust.ImplMethod |
| 707 | `Polyrust.testTrait` | 手寫 | 定義 | | Polyrust.TraitDef |
| 708 | `Polyrust.testTypesImpl` | 手寫 | 定義 | | List String |
| 709 | `Polyrust.traitMethodBar` | 手寫 | 定義 | | Polyrust.TraitMethod |
| 710 | `Polyrust.ImplDef._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.ImplDef → Nat |
| 711 | `Polyrust.ImplDef._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.ImplDef |
| 712 | `Polyrust.ImplDef.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ImplDef → Sort u} →   (t : Polyrust.ImplDef) →     ((traitName : Option String) →         (forTy : String) →           (typeParams  |
| 713 | `Polyrust.ImplDef.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.ImplDef → Nat |
| 714 | `Polyrust.ImplDef.lifetimeParams` | 衍生 | 定義 | | Polyrust.ImplDef → List String |
| 715 | `Polyrust.ImplDef.methods` | 衍生 | 定義 | | Polyrust.ImplDef → List Polyrust.ImplMethod |
| 716 | `Polyrust.ImplDef.mk._flat_ctor` | 等式引理／助手 | 定義 | | Option String → String → List String → List String → List Polyrust.ImplMethod → Polyrust.ImplDef |
| 717 | `Polyrust.ImplDef.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {traitName : Option String} →     {forTy : String} →       {typeParams lifetimeParams : List String} →         {methods : List Polyru |
| 718 | `Polyrust.ImplDef.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.ImplDef} → t = t' → Polyrust.ImplDef.noConfusionType P t t' |
| 719 | `Polyrust.ImplDef.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.ImplDef → Polyrust.ImplDef → Sort u |
| 720 | `Polyrust.ImplDef.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ImplDef → Sort u} →   (t : Polyrust.ImplDef) →     ((traitName : Option String) →         (forTy : String) →           (typeParams  |
| 721 | `Polyrust.ImplDef.typeParams` | 衍生 | 定義 | | Polyrust.ImplDef → List String |
| 722 | `Polyrust.ImplMethod._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.ImplMethod → Nat |
| 723 | `Polyrust.ImplMethod._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.ImplMethod |
| 724 | `Polyrust.ImplMethod.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ImplMethod → Sort u} →   (t : Polyrust.ImplMethod) →     ((name retTy body : String) → motive { name := name, retTy := retTy, body  |
| 725 | `Polyrust.ImplMethod.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.ImplMethod → Nat |
| 726 | `Polyrust.ImplMethod.mk._flat_ctor` | 等式引理／助手 | 定義 | | String → String → String → Polyrust.ImplMethod |
| 727 | `Polyrust.ImplMethod.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {name retTy body name' retTy' body' : String} →     { name := name, retTy := retTy, body := body } = { name := name', retTy := retTy' |
| 728 | `Polyrust.ImplMethod.name` | 衍生 | 定義 | | Polyrust.ImplMethod → String |
| 729 | `Polyrust.ImplMethod.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.ImplMethod} → t = t' → Polyrust.ImplMethod.noConfusionType P t t' |
| 730 | `Polyrust.ImplMethod.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.ImplMethod → Polyrust.ImplMethod → Sort u |
| 731 | `Polyrust.ImplMethod.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ImplMethod → Sort u} →   (t : Polyrust.ImplMethod) →     ((name retTy body : String) → motive { name := name, retTy := retTy, body  |
| 732 | `Polyrust.ImplMethod.retTy` | 衍生 | 定義 | | Polyrust.ImplMethod → String |
| 733 | `Polyrust.MethodTable._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.MethodTable → Nat |
| 734 | `Polyrust.MethodTable._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.MethodTable |
| 735 | `Polyrust.MethodTable.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.MethodTable → Sort u} →   (t : Polyrust.MethodTable) →     ((traits : List Polyrust.TraitDef) →         (impls : List Polyrust.Impl |
| 736 | `Polyrust.MethodTable.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.MethodTable → Nat |
| 737 | `Polyrust.MethodTable.mk._flat_ctor` | 等式引理／助手 | 定義 | | List Polyrust.TraitDef → List Polyrust.ImplDef → Polyrust.MethodTable |
| 738 | `Polyrust.MethodTable.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {traits : List Polyrust.TraitDef} →     {impls : List Polyrust.ImplDef} →       {traits' : List Polyrust.TraitDef} →         {impls'  |
| 739 | `Polyrust.MethodTable.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.MethodTable} → t = t' → Polyrust.MethodTable.noConfusionType P t t' |
| 740 | `Polyrust.MethodTable.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.MethodTable → Polyrust.MethodTable → Sort u |
| 741 | `Polyrust.MethodTable.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.MethodTable → Sort u} →   (t : Polyrust.MethodTable) →     ((traits : List Polyrust.TraitDef) →         (impls : List Polyrust.Impl |
| 742 | `Polyrust.MethodTable.typesImplementing.match_1` | 等式引理／助手 | 定義 | | (motive : Option String → Sort u_1) →   (x : Option String) → ((t : String) → motive (some t)) → (Unit → motive none) → motive x |
| 743 | `Polyrust.TraitDef._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.TraitDef → Nat |
| 744 | `Polyrust.TraitDef._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.TraitDef |
| 745 | `Polyrust.TraitDef.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.TraitDef → Sort u} →   (t : Polyrust.TraitDef) →     ((name : String) →         (typeParams lifetimeParams : List String) →         |
| 746 | `Polyrust.TraitDef.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.TraitDef → Nat |
| 747 | `Polyrust.TraitDef.lifetimeParams` | 衍生 | 定義 | | Polyrust.TraitDef → List String |
| 748 | `Polyrust.TraitDef.methods` | 衍生 | 定義 | | Polyrust.TraitDef → List Polyrust.TraitMethod |
| 749 | `Polyrust.TraitDef.mk._flat_ctor` | 等式引理／助手 | 定義 | | String → List String → List String → List Polyrust.TraitMethod → List String → Polyrust.TraitDef |
| 750 | `Polyrust.TraitDef.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {name : String} →     {typeParams lifetimeParams : List String} →       {methods : List Polyrust.TraitMethod} →         {supertraits  |
| 751 | `Polyrust.TraitDef.name` | 衍生 | 定義 | | Polyrust.TraitDef → String |
| 752 | `Polyrust.TraitDef.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.TraitDef} → t = t' → Polyrust.TraitDef.noConfusionType P t t' |
| 753 | `Polyrust.TraitDef.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.TraitDef → Polyrust.TraitDef → Sort u |
| 754 | `Polyrust.TraitDef.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.TraitDef → Sort u} →   (t : Polyrust.TraitDef) →     ((name : String) →         (typeParams lifetimeParams : List String) →         |
| 755 | `Polyrust.TraitDef.supertraits` | 衍生 | 定義 | | Polyrust.TraitDef → List String |
| 756 | `Polyrust.TraitDef.typeParams` | 衍生 | 定義 | | Polyrust.TraitDef → List String |
| 757 | `Polyrust.TraitMethod._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.TraitMethod → Nat |
| 758 | `Polyrust.TraitMethod._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.TraitMethod |
| 759 | `Polyrust.TraitMethod.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.TraitMethod → Sort u} →   (t : Polyrust.TraitMethod) →     ((name retTy : String) → (hasDefault : Bool) → motive { name := name, re |
| 760 | `Polyrust.TraitMethod.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.TraitMethod → Nat |
| 761 | `Polyrust.TraitMethod.hasDefault` | 衍生 | 定義 | | Polyrust.TraitMethod → Bool |
| 762 | `Polyrust.TraitMethod.mk._flat_ctor` | 等式引理／助手 | 定義 | | String → String → Bool → Polyrust.TraitMethod |
| 763 | `Polyrust.TraitMethod.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {name retTy : String} →     {hasDefault : Bool} →       {name' retTy' : String} →         {hasDefault' : Bool} →           { name :=  |
| 764 | `Polyrust.TraitMethod.name` | 衍生 | 定義 | | Polyrust.TraitMethod → String |
| 765 | `Polyrust.TraitMethod.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.TraitMethod} → t = t' → Polyrust.TraitMethod.noConfusionType P t t' |
| 766 | `Polyrust.TraitMethod.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.TraitMethod → Polyrust.TraitMethod → Sort u |
| 767 | `Polyrust.TraitMethod.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.TraitMethod → Sort u} →   (t : Polyrust.TraitMethod) →     ((name retTy : String) → (hasDefault : Bool) → motive { name := name, re |
| 768 | `Polyrust.TraitMethod.retTy` | 衍生 | 定義 | | Polyrust.TraitMethod → String |
| 769 | `Polyrust.instDecidableEqImplMethod` | 實例衍生 | 定義 | | DecidableEq Polyrust.ImplMethod |
| 770 | `Polyrust.instDecidableEqImplMethod.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.ImplMethod) → Decidable (x = x_1) |
| 771 | `Polyrust.instDecidableEqImplMethod.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 : String), { name := a, retTy := a_1, body := a_2 } = { name := a, retTy := a_1, body := a_2 } |
| 772 | `Polyrust.instDecidableEqImplMethod.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b : String),   ¬a_2 = b → { name := a, retTy := a_1, body := a_2 } = { name := a, retTy := a_1, body := b } → False |
| 773 | `Polyrust.instDecidableEqImplMethod.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b b_1 : String),   ¬a_1 = b → { name := a, retTy := a_1, body := a_2 } = { name := a, retTy := b, body := b_1 } → False |
| 774 | `Polyrust.instDecidableEqImplMethod.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 b b_1 b_2 : String),   ¬a = b → { name := a, retTy := a_1, body := a_2 } = { name := b, retTy := b_1, body := b_2 } → False |
| 775 | `Polyrust.instDecidableEqImplMethod.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.ImplMethod → Polyrust.ImplMethod → Sort u_1) →   (x x_1 : Polyrust.ImplMethod) →     ((a a_1 a_2 b b_1 b_2 : String) →         moti |
| 776 | `Polyrust.instDecidableEqTraitMethod` | 實例衍生 | 定義 | | DecidableEq Polyrust.TraitMethod |
| 777 | `Polyrust.instDecidableEqTraitMethod.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.TraitMethod) → Decidable (x = x_1) |
| 778 | `Polyrust.instDecidableEqTraitMethod.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a a_1 : String) (a_2 : Bool),   { name := a, retTy := a_1, hasDefault := a_2 } = { name := a, retTy := a_1, hasDefault := a_2 } |
| 779 | `Polyrust.instDecidableEqTraitMethod.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a a_1 : String) (a_2 b : Bool),   ¬a_2 = b → { name := a, retTy := a_1, hasDefault := a_2 } = { name := a, retTy := a_1, hasDefault := b } → False |
| 780 | `Polyrust.instDecidableEqTraitMethod.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a a_1 : String) (a_2 : Bool) (b : String) (b_1 : Bool),   ¬a_1 = b → { name := a, retTy := a_1, hasDefault := a_2 } = { name := a, retTy := b, hasD |
| 781 | `Polyrust.instDecidableEqTraitMethod.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a a_1 : String) (a_2 : Bool) (b b_1 : String) (b_2 : Bool),   ¬a = b → { name := a, retTy := a_1, hasDefault := a_2 } = { name := b, retTy := b_1,  |
| 782 | `Polyrust.instDecidableEqTraitMethod.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.TraitMethod → Polyrust.TraitMethod → Sort u_1) →   (x x_1 : Polyrust.TraitMethod) →     ((a a_1 : String) →         (a_2 : Bool) →  |

## `Polyrust.ProductReduction`（97 條）

職責：積型歸約：pair/積型檢查與可定型性歸約為逐欄位檢查

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 783 | `Polyrust.ProdExpr.lift.elim` | 手寫 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) → t.ctorIdx = 0 → ((a : Expr) → motive (Polyrust.ProdE |
| 784 | `Polyrust.ProdExpr.lift.inj` | 手寫 | 定理 | | ∀ {Expr : Type} {a a_1 : Expr}, Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift a_1 → a = a_1 |
| 785 | `Polyrust.ProdExpr.lift.sizeOf_spec` | 手寫 | 定理 | | ∀ {Expr : Type} [inst : SizeOf Expr] (a : Expr), sizeOf (Polyrust.ProdExpr.lift a) = 1 + sizeOf a |
| 786 | `Polyrust.ProdExpr.pair.elim` | 手寫 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) → t.ctorIdx = 1 → ((a a_1 : Polyrust.ProdExpr Expr) →  |
| 787 | `Polyrust.ProdExpr.pair.inj` | 手寫 | 定理 | | ∀ {Expr : Type} {a a_1 a_2 a_3 : Polyrust.ProdExpr Expr}, a.pair a_1 = a_2.pair a_3 → a = a_2 ∧ a_1 = a_3 |
| 788 | `Polyrust.ProdExpr.pair.sizeOf_spec` | 手寫 | 定理 | | ∀ {Expr : Type} [inst : SizeOf Expr] (a a_1 : Polyrust.ProdExpr Expr), sizeOf (a.pair a_1) = 1 + sizeOf a + sizeOf a_1 |
| 789 | `Polyrust.ProdTy.base.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) → t.ctorIdx = 0 → ((a : Ty) → motive (Polyrust.ProdTy.base a)) → |
| 790 | `Polyrust.ProdTy.base.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : Ty}, Polyrust.ProdTy.base a = Polyrust.ProdTy.base a_1 → a = a_1 |
| 791 | `Polyrust.ProdTy.base.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Ty), sizeOf (Polyrust.ProdTy.base a) = 1 + sizeOf a |
| 792 | `Polyrust.ProdTy.prod.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) → t.ctorIdx = 1 → ((a a_1 : Polyrust.ProdTy Ty) → motive (a.prod |
| 793 | `Polyrust.ProdTy.prod.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 a_2 a_3 : Polyrust.ProdTy Ty}, a.prod a_1 = a_2.prod a_3 → a = a_2 ∧ a_1 = a_3 |
| 794 | `Polyrust.ProdTy.prod.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 : Polyrust.ProdTy Ty), sizeOf (a.prod a_1) = 1 + sizeOf a + sizeOf a_1 |
| 795 | `Polyrust.TypableProd` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Prop |
| 796 | `Polyrust.pairBit` | 手寫 | 定義 | | {Ty : Type} → (Ty → Bool) → (Ty → Bool) → Ty → Ty → Int |
| 797 | `Polyrust.pairBitSum` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → (Ty → Bool) → (Ty → Bool) → Int |
| 798 | `Polyrust.tycheckProd` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Polyrust.ProdTy Ty → Bool |
| 799 | `Polyrust.tycheck_pair` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {a b : Polyrust.ProdExpr Polyrust.Expr}   {τ₁ τ₂ : Polyrust.ProdTy Ty},   Polyrust.tychec |
| 800 | `Polyrust.ProdExpr._sizeOf_1` | 歸納型衍生 | 定義 | | {Expr : Type} → [SizeOf Expr] → Polyrust.ProdExpr Expr → Nat |
| 801 | `Polyrust.ProdExpr._sizeOf_inst` | 歸納型衍生 | 定義 | | (Expr : Type) → [SizeOf Expr] → SizeOf (Polyrust.ProdExpr Expr) |
| 802 | `Polyrust.ProdExpr.below` | 歸納型衍生 | 定義 | | {Expr : Type} → {motive : Polyrust.ProdExpr Expr → Sort u} → Polyrust.ProdExpr Expr → Sort (max 1 u) |
| 803 | `Polyrust.ProdExpr.brecOn` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) → ((t : Polyrust.ProdExpr Expr) → Polyrust.ProdExpr.be |
| 804 | `Polyrust.ProdExpr.brecOn.eq` | 衍生 | 定理 | | ∀ {Expr : Type} {motive : Polyrust.ProdExpr Expr → Sort u} (t : Polyrust.ProdExpr Expr)   (F_1 : (t : Polyrust.ProdExpr Expr) → Polyrust.ProdExpr.belo |
| 805 | `Polyrust.ProdExpr.brecOn.go` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →       ((t : Polyrust.ProdExpr Expr) → Polyrust.ProdE |
| 806 | `Polyrust.ProdExpr.casesOn` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →       ((a : Expr) → motive (Polyrust.ProdExpr.lift a |
| 807 | `Polyrust.ProdExpr.ctorElim` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (ctorIdx : Nat) →       (t : Polyrust.ProdExpr Expr) → ctorIdx = t.ctorIdx → Polyru |
| 808 | `Polyrust.ProdExpr.ctorElimType` | 衍生 | 定義 | | {Expr : Type} → {motive : Polyrust.ProdExpr Expr → Sort u} → Nat → Sort (max 1 u) |
| 809 | `Polyrust.ProdExpr.ctorIdx` | 歸納型衍生 | 定義 | | {Expr : Type} → Polyrust.ProdExpr Expr → Nat |
| 810 | `Polyrust.ProdExpr.lift.noConfusion` | 歸納型衍生 | 定義 | | {Expr : Type} → {P : Sort u} → {a a' : Expr} → Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift a' → (a ≍ a' → P) → P |
| 811 | `Polyrust.ProdExpr.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Expr : Type} →     {t : Polyrust.ProdExpr Expr} →       {Expr' : Type} → {t' : Polyrust.ProdExpr Expr'} → Expr = Expr' → t ≍ t' → Po |
| 812 | `Polyrust.ProdExpr.noConfusionType` | 衍生 | 定義 | | Sort u → {Expr : Type} → Polyrust.ProdExpr Expr → {Expr' : Type} → Polyrust.ProdExpr Expr' → Sort u |
| 813 | `Polyrust.ProdExpr.pair.noConfusion` | 歸納型衍生 | 定義 | | {Expr : Type} →   {P : Sort u} → {a a_1 a' a'_1 : Polyrust.ProdExpr Expr} → a.pair a_1 = a'.pair a'_1 → (a ≍ a' → a_1 ≍ a'_1 → P) → P |
| 814 | `Polyrust.ProdExpr.recOn` | 歸納型衍生 | 定義 | | {Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →       ((a : Expr) → motive (Polyrust.ProdExpr.lift a |
| 815 | `Polyrust.ProdTy._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.ProdTy Ty → Nat |
| 816 | `Polyrust.ProdTy._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.ProdTy Ty) |
| 817 | `Polyrust.ProdTy.base.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} → {P : Sort u} → {a a' : Ty} → Polyrust.ProdTy.base a = Polyrust.ProdTy.base a' → (a ≍ a' → P) → P |
| 818 | `Polyrust.ProdTy.below` | 歸納型衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.ProdTy Ty → Sort u} → Polyrust.ProdTy Ty → Sort (max 1 u) |
| 819 | `Polyrust.ProdTy.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) → ((t : Polyrust.ProdTy Ty) → Polyrust.ProdTy.below t → motive t |
| 820 | `Polyrust.ProdTy.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive : Polyrust.ProdTy Ty → Sort u} (t : Polyrust.ProdTy Ty)   (F_1 : (t : Polyrust.ProdTy Ty) → Polyrust.ProdTy.below t → motive t), |
| 821 | `Polyrust.ProdTy.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) →       ((t : Polyrust.ProdTy Ty) → Polyrust.ProdTy.below t → mo |
| 822 | `Polyrust.ProdTy.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) →       ((a : Ty) → motive (Polyrust.ProdTy.base a)) → ((a a_1 : |
| 823 | `Polyrust.ProdTy.ctorElim` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.ProdTy Ty) → ctorIdx = t.ctorIdx → Polyrust.ProdTy.ctorEl |
| 824 | `Polyrust.ProdTy.ctorElimType` | 衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.ProdTy Ty → Sort u} → Nat → Sort (max 1 u) |
| 825 | `Polyrust.ProdTy.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.ProdTy Ty → Nat |
| 826 | `Polyrust.ProdTy.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.ProdTy Ty} →       {Ty' : Type} → {t' : Polyrust.ProdTy Ty'} → Ty = Ty' → t ≍ t' → Polyrust.ProdTy.no |
| 827 | `Polyrust.ProdTy.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.ProdTy Ty → {Ty' : Type} → Polyrust.ProdTy Ty' → Sort u |
| 828 | `Polyrust.ProdTy.prod.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} → {a a_1 a' a'_1 : Polyrust.ProdTy Ty} → a.prod a_1 = a'.prod a'_1 → (a ≍ a' → a_1 ≍ a'_1 → P) → P |
| 829 | `Polyrust.ProdTy.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) →       ((a : Ty) → motive (Polyrust.ProdTy.base a)) →         ( |
| 830 | `Polyrust.instDecidableEqProdExpr` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → DecidableEq (Polyrust.ProdExpr Expr) |
| 831 | `Polyrust.instDecidableEqProdExpr.decEq` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.ProdExpr Expr) → Decidable (x = x_1) |
| 832 | `Polyrust.instDecidableEqProdExpr.decEq._f` | 實例衍生 | 定義 | | {Expr : Type} →   [DecidableEq Expr] →     (x : Polyrust.ProdExpr Expr) →       Polyrust.ProdExpr.below (motive := fun x => (x_1 : Polyrust.ProdExpr E |
| 833 | `Polyrust.instDecidableEqProdExpr.decEq._proof_1` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Expr), Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift a |
| 834 | `Polyrust.instDecidableEqProdExpr.decEq._proof_2` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a b : Expr), ¬a = b → Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift b → False |
| 835 | `Polyrust.instDecidableEqProdExpr.decEq._proof_3` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a : Expr) (a_1 a_2 : Polyrust.ProdExpr Expr), Polyrust.ProdExpr.lift a = a_1.pair a_2 → False |
| 836 | `Polyrust.instDecidableEqProdExpr.decEq._proof_4` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a a_1 : Polyrust.ProdExpr Expr) (a_2 : Expr), a.pair a_1 = Polyrust.ProdExpr.lift a_2 → False |
| 837 | `Polyrust.instDecidableEqProdExpr.decEq._proof_5` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a a_1 : Polyrust.ProdExpr Expr), a.pair a_1 = a.pair a_1 |
| 838 | `Polyrust.instDecidableEqProdExpr.decEq._proof_6` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a a_1 b : Polyrust.ProdExpr Expr), ¬a_1 = b → a.pair a_1 = a.pair b → False |
| 839 | `Polyrust.instDecidableEqProdExpr.decEq._proof_7` | 實例衍生 | 定理 | | ∀ {Expr : Type} (a a_1 b b_1 : Polyrust.ProdExpr Expr), ¬a = b → a.pair a_1 = b.pair b_1 → False |
| 840 | `Polyrust.instDecidableEqProdExpr.decEq._sunfold` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.ProdExpr Expr) → Decidable (x = x_1) |
| 841 | `Polyrust.instDecidableEqProdExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | | {Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.ProdExpr Expr) → Decidable (x = x_1) |
| 842 | `Polyrust.instDecidableEqProdExpr.decEq.match_1` | 實例衍生 | 定義 | | {Expr : Type} →   (motive : Polyrust.ProdExpr Expr → Polyrust.ProdExpr Expr → Sort u_1) →     (x x_1 : Polyrust.ProdExpr Expr) →       ((a b : Expr) → |
| 843 | `Polyrust.instDecidableEqProdTy` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.ProdTy Ty) |
| 844 | `Polyrust.instDecidableEqProdTy.decEq` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ProdTy Ty) → Decidable (x = x_1) |
| 845 | `Polyrust.instDecidableEqProdTy.decEq._f` | 實例衍生 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     (x : Polyrust.ProdTy Ty) →       Polyrust.ProdTy.below (motive := fun x => (x_1 : Polyrust.ProdTy Ty) → Decidab |
| 846 | `Polyrust.instDecidableEqProdTy.decEq._proof_1` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Ty), Polyrust.ProdTy.base a = Polyrust.ProdTy.base a |
| 847 | `Polyrust.instDecidableEqProdTy.decEq._proof_2` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a b : Ty), ¬a = b → Polyrust.ProdTy.base a = Polyrust.ProdTy.base b → False |
| 848 | `Polyrust.instDecidableEqProdTy.decEq._proof_3` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a : Ty) (a_1 a_2 : Polyrust.ProdTy Ty), Polyrust.ProdTy.base a = a_1.prod a_2 → False |
| 849 | `Polyrust.instDecidableEqProdTy.decEq._proof_4` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 : Polyrust.ProdTy Ty) (a_2 : Ty), a.prod a_1 = Polyrust.ProdTy.base a_2 → False |
| 850 | `Polyrust.instDecidableEqProdTy.decEq._proof_5` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 : Polyrust.ProdTy Ty), a.prod a_1 = a.prod a_1 |
| 851 | `Polyrust.instDecidableEqProdTy.decEq._proof_6` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 b : Polyrust.ProdTy Ty), ¬a_1 = b → a.prod a_1 = a.prod b → False |
| 852 | `Polyrust.instDecidableEqProdTy.decEq._proof_7` | 實例衍生 | 定理 | | ∀ {Ty : Type} (a a_1 b b_1 : Polyrust.ProdTy Ty), ¬a = b → a.prod a_1 = b.prod b_1 → False |
| 853 | `Polyrust.instDecidableEqProdTy.decEq._sunfold` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ProdTy Ty) → Decidable (x = x_1) |
| 854 | `Polyrust.instDecidableEqProdTy.decEq._unsafe_rec` | 實例衍生 | 定義 | | {Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ProdTy Ty) → Decidable (x = x_1) |
| 855 | `Polyrust.instDecidableEqProdTy.decEq.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.ProdTy Ty → Polyrust.ProdTy Ty → Sort u_1) →     (x x_1 : Polyrust.ProdTy Ty) →       ((a b : Ty) → motive (Polyrus |
| 856 | `Polyrust.instReprProdExpr` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Repr (Polyrust.ProdExpr Expr) |
| 857 | `Polyrust.instReprProdExpr.repr` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Polyrust.ProdExpr Expr → Nat → Format |
| 858 | `Polyrust.instReprProdExpr.repr._f` | 實例衍生 | 定義 | | {Expr : Type} →   [Repr Expr] →     (x : Polyrust.ProdExpr Expr) → Polyrust.ProdExpr.below (motive := fun x => Nat → Format) x → Nat → Format |
| 859 | `Polyrust.instReprProdExpr.repr._sunfold` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Polyrust.ProdExpr Expr → Nat → Format |
| 860 | `Polyrust.instReprProdExpr.repr._unsafe_rec` | 實例衍生 | 定義 | | {Expr : Type} → [Repr Expr] → Polyrust.ProdExpr Expr → Nat → Format |
| 861 | `Polyrust.instReprProdExpr.repr.match_1` | 實例衍生 | 定義 | | {Expr : Type} →   (motive : Polyrust.ProdExpr Expr → Sort u_1) →     (x : Polyrust.ProdExpr Expr) →       ((a : Expr) → motive (Polyrust.ProdExpr.lift |
| 862 | `Polyrust.instReprProdTy` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Repr (Polyrust.ProdTy Ty) |
| 863 | `Polyrust.instReprProdTy.repr` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.ProdTy Ty → Nat → Format |
| 864 | `Polyrust.instReprProdTy.repr._f` | 實例衍生 | 定義 | | {Ty : Type} →   [Repr Ty] → (x : Polyrust.ProdTy Ty) → Polyrust.ProdTy.below (motive := fun x => Nat → Format) x → Nat → Format |
| 865 | `Polyrust.instReprProdTy.repr._sunfold` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.ProdTy Ty → Nat → Format |
| 866 | `Polyrust.instReprProdTy.repr._unsafe_rec` | 實例衍生 | 定義 | | {Ty : Type} → [Repr Ty] → Polyrust.ProdTy Ty → Nat → Format |
| 867 | `Polyrust.instReprProdTy.repr.match_1` | 實例衍生 | 定義 | | {Ty : Type} →   (motive : Polyrust.ProdTy Ty → Sort u_1) →     (x : Polyrust.ProdTy Ty) →       ((a : Ty) → motive (Polyrust.ProdTy.base a)) → ((a a_1 |
| 868 | `Polyrust.tycheckProd._f` | 等式引理／助手 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty →       (x : Polyrust.ProdExpr Polyrust.Expr) →         Polyrust.ProdExpr.below (motive := fun |
| 869 | `Polyrust.tycheckProd._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Polyrust.ProdTy Ty → Bool |
| 870 | `Polyrust.tycheckProd._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Polyrust.ProdTy Ty → Bool |
| 871 | `Polyrust.tycheckProd.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (τ : Ty),   Polyrust.tycheckProd L (Polyrust.ProdExpr.lift e) (Polyru |
| 872 | `Polyrust.tycheckProd.eq_2` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.Expr) (a_1 a_2 : Polyrust.ProdTy Ty),   Polyrust.tycheckProd L (Polyrust.Pr |
| 873 | `Polyrust.tycheckProd.eq_3` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.ProdExpr Polyrust.Expr)   (τ₁ τ₂ : Polyrust.ProdTy Ty),   Polyrust.tychec |
| 874 | `Polyrust.tycheckProd.eq_4` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a a_1 : Polyrust.ProdExpr Polyrust.Expr) (a_2 : Ty),   Polyrust.tycheckProd L (a.pair a_ |
| 875 | `Polyrust.tycheckProd.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.ProdExpr Polyrust.Expr)   (x_1 : Polyrust.ProdTy Ty),   Polyrust.tycheckPro |
| 876 | `Polyrust.tycheckProd.match_1` | 等式引理／助手 | 定義 | | {Ty : Type} →   (motive : Polyrust.ProdExpr Polyrust.Expr → Polyrust.ProdTy Ty → Sort u_1) →     (x : Polyrust.ProdExpr Polyrust.Expr) →       (x_1 :  |
| 877 | `Polyrust.tycheckProd_exclusive.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (τ τ' : Polyrust.ProdTy Ty)   (motive :     Polyrust.tycheckProd L (P |
| 878 | `Polyrust.tycheckProd_exclusive.match_1_8` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.ProdExpr Polyrust.Expr)   (τ τ' : Polyrust.ProdTy Ty)   (motive : Polyrus |
| 879 | `Polyrust.typable_pair_iff.match_1_6` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {a b : Polyrust.ProdExpr Polyrust.Expr}   (motive : (∃ τ₁ τ₂, Polyrust.tycheckProd L a τ₁ |

## `Polyrust.LifetimeRegion`（93 條）

職責：Lifetime/'static、outlives 圖、無環鐵律、NLL 區間重疊

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 880 | `Polyrust.Lifetime.named.elim` | 手寫 | 定義 | | {motive : Polyrust.Lifetime → Sort u} →   (t : Polyrust.Lifetime) → t.ctorIdx = 0 → ((a : String) → motive (Polyrust.Lifetime.named a)) → motive t |
| 881 | `Polyrust.Lifetime.named.inj` | 手寫 | 定理 | | ∀ {a a_1 : String}, Polyrust.Lifetime.named a = Polyrust.Lifetime.named a_1 → a = a_1 |
| 882 | `Polyrust.Lifetime.named.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : String), sizeOf (Polyrust.Lifetime.named a) = 1 + sizeOf a |
| 883 | `Polyrust.Lifetime.static.elim` | 手寫 | 定義 | | {motive : Polyrust.Lifetime → Sort u} →   (t : Polyrust.Lifetime) → t.ctorIdx = 1 → motive Polyrust.Lifetime.static → motive t |
| 884 | `Polyrust.Lifetime.static.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Lifetime.static = 1 |
| 885 | `Polyrust.LifetimeGraph.addLifetime` | 手寫 | 定義 | | Polyrust.LifetimeGraph → Polyrust.Lifetime → Polyrust.LifetimeGraph |
| 886 | `Polyrust.LifetimeGraph.addOutlives` | 手寫 | 定義 | | Polyrust.LifetimeGraph → Polyrust.Outlives → Polyrust.LifetimeGraph |
| 887 | `Polyrust.LifetimeGraph.empty` | 手寫 | 定義 | | Polyrust.LifetimeGraph |
| 888 | `Polyrust.LifetimeGraph.hasSelfLoop` | 手寫 | 定義 | | Polyrust.LifetimeGraph → Bool |
| 889 | `Polyrust.LifetimeGraph.hasTwoCycle` | 手寫 | 定義 | | Polyrust.LifetimeGraph → Bool |
| 890 | `Polyrust.LifetimeGraph.mk.inj` | 手寫 | 定理 | | ∀ {lifetimes : List Polyrust.Lifetime} {edges : List Polyrust.Outlives} {lifetimes_1 : List Polyrust.Lifetime}   {edges_1 : List Polyrust.Outlives},   |
| 891 | `Polyrust.LifetimeGraph.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (lifetimes : List Polyrust.Lifetime) (edges : List Polyrust.Outlives),   sizeOf { lifetimes := lifetimes, edges := edges } = 1 + sizeOf lifetimes +  |
| 892 | `Polyrust.LifetimeGraph.outlivesHolds` | 手寫 | 定義 | | Polyrust.LifetimeGraph → Polyrust.Lifetime → Polyrust.Lifetime → Bool |
| 893 | `Polyrust.LifetimeGraph.transitiveStep` | 手寫 | 定義 | | Polyrust.LifetimeGraph → Polyrust.LifetimeGraph |
| 894 | `Polyrust.Outlives.mk.inj` | 手寫 | 定理 | | ∀ {longer shorter longer_1 shorter_1 : Polyrust.Lifetime},   { longer := longer, shorter := shorter } = { longer := longer_1, shorter := shorter_1 } → |
| 895 | `Polyrust.Outlives.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (longer shorter : Polyrust.Lifetime),   sizeOf { longer := longer, shorter := shorter } = 1 + sizeOf longer + sizeOf shorter |
| 896 | `Polyrust.Region.mk.inj` | 手寫 | 定理 | | ∀ {lifetime : Polyrust.Lifetime} {start fin borrowNode : Nat} {lifetime_1 : Polyrust.Lifetime}   {start_1 fin_1 borrowNode_1 : Nat},   { lifetime := l |
| 897 | `Polyrust.Region.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (lifetime : Polyrust.Lifetime) (start fin borrowNode : Nat),   sizeOf { lifetime := lifetime, start := start, fin := fin, borrowNode := borrowNode } |
| 898 | `Polyrust.Region.overlaps` | 手寫 | 定義 | | Polyrust.Region → Polyrust.Region → Bool |
| 899 | `Polyrust.checkNLL` | 手寫 | 定義 | | List Polyrust.Region → Polyrust.LifetimeGraph → List (Nat × Nat) |
| 900 | `Polyrust.testGraph` | 手寫 | 定義 | | Polyrust.LifetimeGraph |
| 901 | `Polyrust.testGraphHolds` | 手寫 | 定義 | | Bool |
| 902 | `Polyrust.testGraphSelfLoop` | 手寫 | 定義 | | Bool |
| 903 | `Polyrust.testGraphTwoCycle` | 手寫 | 定義 | | Bool |
| 904 | `Polyrust.Lifetime._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Lifetime → Nat |
| 905 | `Polyrust.Lifetime._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Lifetime |
| 906 | `Polyrust.Lifetime.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Lifetime → Sort u} →   (t : Polyrust.Lifetime) →     ((a : String) → motive (Polyrust.Lifetime.named a)) → motive Polyrust.Lifetime |
| 907 | `Polyrust.Lifetime.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.Lifetime → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Lifetime) → ctorIdx = t.ctorIdx → Polyrust.Lifetime.ctorElimType ctorIdx → m |
| 908 | `Polyrust.Lifetime.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.Lifetime → Sort u} → Nat → Sort (max 1 u) |
| 909 | `Polyrust.Lifetime.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Lifetime → Nat |
| 910 | `Polyrust.Lifetime.named.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : String} → Polyrust.Lifetime.named a = Polyrust.Lifetime.named a' → (a = a' → P) → P |
| 911 | `Polyrust.Lifetime.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Lifetime} → t = t' → Polyrust.Lifetime.noConfusionType P t t' |
| 912 | `Polyrust.Lifetime.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Lifetime → Polyrust.Lifetime → Sort u |
| 913 | `Polyrust.Lifetime.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Lifetime → Sort u} →   (t : Polyrust.Lifetime) →     ((a : String) → motive (Polyrust.Lifetime.named a)) → motive Polyrust.Lifetime |
| 914 | `Polyrust.LifetimeGraph._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.LifetimeGraph → Nat |
| 915 | `Polyrust.LifetimeGraph._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.LifetimeGraph |
| 916 | `Polyrust.LifetimeGraph.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.LifetimeGraph → Sort u} →   (t : Polyrust.LifetimeGraph) →     ((lifetimes : List Polyrust.Lifetime) →         (edges : List Polyru |
| 917 | `Polyrust.LifetimeGraph.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.LifetimeGraph → Nat |
| 918 | `Polyrust.LifetimeGraph.edges` | 衍生 | 定義 | | Polyrust.LifetimeGraph → List Polyrust.Outlives |
| 919 | `Polyrust.LifetimeGraph.hasSelfLoop.match_1` | 等式引理／助手 | 定義 | | (motive : Polyrust.LifetimeGraph → Sort u_1) →   (x : Polyrust.LifetimeGraph) →     ((lifetimes : List Polyrust.Lifetime) →         (es : List Polyrus |
| 920 | `Polyrust.LifetimeGraph.lifetimes` | 衍生 | 定義 | | Polyrust.LifetimeGraph → List Polyrust.Lifetime |
| 921 | `Polyrust.LifetimeGraph.mk._flat_ctor` | 等式引理／助手 | 定義 | | List Polyrust.Lifetime → List Polyrust.Outlives → Polyrust.LifetimeGraph |
| 922 | `Polyrust.LifetimeGraph.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {lifetimes : List Polyrust.Lifetime} →     {edges : List Polyrust.Outlives} →       {lifetimes' : List Polyrust.Lifetime} →         { |
| 923 | `Polyrust.LifetimeGraph.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.LifetimeGraph} → t = t' → Polyrust.LifetimeGraph.noConfusionType P t t' |
| 924 | `Polyrust.LifetimeGraph.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.LifetimeGraph → Polyrust.LifetimeGraph → Sort u |
| 925 | `Polyrust.LifetimeGraph.outlivesHolds.eq_1` | 等式引理／助手 | 定理 | | ∀ (g : Polyrust.LifetimeGraph) (longer shorter : Polyrust.Lifetime),   g.outlivesHolds longer shorter =     if (longer == shorter) = true then true    |
| 926 | `Polyrust.LifetimeGraph.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.LifetimeGraph → Sort u} →   (t : Polyrust.LifetimeGraph) →     ((lifetimes : List Polyrust.Lifetime) →         (edges : List Polyru |
| 927 | `Polyrust.Outlives._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Outlives → Nat |
| 928 | `Polyrust.Outlives._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Outlives |
| 929 | `Polyrust.Outlives.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Outlives → Sort u} →   (t : Polyrust.Outlives) →     ((longer shorter : Polyrust.Lifetime) → motive { longer := longer, shorter :=  |
| 930 | `Polyrust.Outlives.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Outlives → Nat |
| 931 | `Polyrust.Outlives.longer` | 衍生 | 定義 | | Polyrust.Outlives → Polyrust.Lifetime |
| 932 | `Polyrust.Outlives.mk._flat_ctor` | 等式引理／助手 | 定義 | | Polyrust.Lifetime → Polyrust.Lifetime → Polyrust.Outlives |
| 933 | `Polyrust.Outlives.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {longer shorter longer' shorter' : Polyrust.Lifetime} →     { longer := longer, shorter := shorter } = { longer := longer', shorter : |
| 934 | `Polyrust.Outlives.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Outlives} → t = t' → Polyrust.Outlives.noConfusionType P t t' |
| 935 | `Polyrust.Outlives.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Outlives → Polyrust.Outlives → Sort u |
| 936 | `Polyrust.Outlives.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Outlives → Sort u} →   (t : Polyrust.Outlives) →     ((longer shorter : Polyrust.Lifetime) → motive { longer := longer, shorter :=  |
| 937 | `Polyrust.Outlives.shorter` | 衍生 | 定義 | | Polyrust.Outlives → Polyrust.Lifetime |
| 938 | `Polyrust.Region._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Region → Nat |
| 939 | `Polyrust.Region._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Region |
| 940 | `Polyrust.Region.borrowNode` | 衍生 | 定義 | | Polyrust.Region → Nat |
| 941 | `Polyrust.Region.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Region → Sort u} →   (t : Polyrust.Region) →     ((lifetime : Polyrust.Lifetime) →         (start fin borrowNode : Nat) →           |
| 942 | `Polyrust.Region.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Region → Nat |
| 943 | `Polyrust.Region.fin` | 衍生 | 定義 | | Polyrust.Region → Nat |
| 944 | `Polyrust.Region.lifetime` | 衍生 | 定義 | | Polyrust.Region → Polyrust.Lifetime |
| 945 | `Polyrust.Region.mk._flat_ctor` | 等式引理／助手 | 定義 | | Polyrust.Lifetime → Nat → Nat → Nat → Polyrust.Region |
| 946 | `Polyrust.Region.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {lifetime : Polyrust.Lifetime} →     {start fin borrowNode : Nat} →       {lifetime' : Polyrust.Lifetime} →         {start' fin' borr |
| 947 | `Polyrust.Region.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Region} → t = t' → Polyrust.Region.noConfusionType P t t' |
| 948 | `Polyrust.Region.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Region → Polyrust.Region → Sort u |
| 949 | `Polyrust.Region.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Region → Sort u} →   (t : Polyrust.Region) →     ((lifetime : Polyrust.Lifetime) →         (start fin borrowNode : Nat) →           |
| 950 | `Polyrust.Region.start` | 衍生 | 定義 | | Polyrust.Region → Nat |
| 951 | `Polyrust.instDecidableEqLifetime` | 實例衍生 | 定義 | | DecidableEq Polyrust.Lifetime |
| 952 | `Polyrust.instDecidableEqLifetime.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Lifetime) → Decidable (x = x_1) |
| 953 | `Polyrust.instDecidableEqLifetime.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : String), Polyrust.Lifetime.named a = Polyrust.Lifetime.named a |
| 954 | `Polyrust.instDecidableEqLifetime.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a b : String), ¬a = b → Polyrust.Lifetime.named a = Polyrust.Lifetime.named b → False |
| 955 | `Polyrust.instDecidableEqLifetime.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : String), Polyrust.Lifetime.named a = Polyrust.Lifetime.static → False |
| 956 | `Polyrust.instDecidableEqLifetime.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : String), Polyrust.Lifetime.static = Polyrust.Lifetime.named a → False |
| 957 | `Polyrust.instDecidableEqLifetime.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Lifetime → Polyrust.Lifetime → Sort u_1) →   (x x_1 : Polyrust.Lifetime) →     ((a b : String) → motive (Polyrust.Lifetime.named a) |
| 958 | `Polyrust.instDecidableEqOutlives` | 實例衍生 | 定義 | | DecidableEq Polyrust.Outlives |
| 959 | `Polyrust.instDecidableEqOutlives.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Outlives) → Decidable (x = x_1) |
| 960 | `Polyrust.instDecidableEqOutlives.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a a_1 : Polyrust.Lifetime), { longer := a, shorter := a_1 } = { longer := a, shorter := a_1 } |
| 961 | `Polyrust.instDecidableEqOutlives.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a a_1 b : Polyrust.Lifetime), ¬a_1 = b → { longer := a, shorter := a_1 } = { longer := a, shorter := b } → False |
| 962 | `Polyrust.instDecidableEqOutlives.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a a_1 b b_1 : Polyrust.Lifetime), ¬a = b → { longer := a, shorter := a_1 } = { longer := b, shorter := b_1 } → False |
| 963 | `Polyrust.instDecidableEqOutlives.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Outlives → Polyrust.Outlives → Sort u_1) →   (x x_1 : Polyrust.Outlives) →     ((a a_1 b b_1 : Polyrust.Lifetime) → motive { longer |
| 964 | `Polyrust.instDecidableEqRegion` | 實例衍生 | 定義 | | DecidableEq Polyrust.Region |
| 965 | `Polyrust.instDecidableEqRegion.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Region) → Decidable (x = x_1) |
| 966 | `Polyrust.instDecidableEqRegion.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : Polyrust.Lifetime) (a_1 a_2 a_3 : Nat),   { lifetime := a, start := a_1, fin := a_2, borrowNode := a_3 } =     { lifetime := a, start := a_1, f |
| 967 | `Polyrust.instDecidableEqRegion.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a : Polyrust.Lifetime) (a_1 a_2 a_3 b : Nat),   ¬a_3 = b →     { lifetime := a, start := a_1, fin := a_2, borrowNode := a_3 } =         { lifetime  |
| 968 | `Polyrust.instDecidableEqRegion.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : Polyrust.Lifetime) (a_1 a_2 a_3 b b_1 : Nat),   ¬a_2 = b →     { lifetime := a, start := a_1, fin := a_2, borrowNode := a_3 } =         { lifet |
| 969 | `Polyrust.instDecidableEqRegion.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : Polyrust.Lifetime) (a_1 a_2 a_3 b b_1 b_2 : Nat),   ¬a_1 = b →     { lifetime := a, start := a_1, fin := a_2, borrowNode := a_3 } =         { l |
| 970 | `Polyrust.instDecidableEqRegion.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a : Polyrust.Lifetime) (a_1 a_2 a_3 : Nat) (b : Polyrust.Lifetime) (b_1 b_2 b_3 : Nat),   ¬a = b →     { lifetime := a, start := a_1, fin := a_2, b |
| 971 | `Polyrust.instDecidableEqRegion.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Region → Polyrust.Region → Sort u_1) →   (x x_1 : Polyrust.Region) →     ((a : Polyrust.Lifetime) →         (a_1 a_2 a_3 : Nat) →   |
| 972 | `Polyrust.instReprLifetime.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Lifetime → Sort u_1) →   (x : Polyrust.Lifetime) →     ((a : String) → motive (Polyrust.Lifetime.named a)) → (Unit → motive Polyrus |

## `Polyrust.MatchDecisionTree`（93 條）

職責：Pat/MatchArm/DecisionTree；決策樹深度 ≤ 臂數；編譯保語義

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 973 | `Polyrust.DecisionTree.branch.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) →       t.ctorIdx = 1 →         ((a a_1 : String) →  |
| 974 | `Polyrust.DecisionTree.branch.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : String} {a_2 a_3 : Polyrust.DecisionTree Ty} {a_4 a_5 : String}   {a_6 a_7 : Polyrust.DecisionTree Ty},   Polyrust.DecisionTree |
| 975 | `Polyrust.DecisionTree.branch.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 : String) (a_2 a_3 : Polyrust.DecisionTree Ty),   sizeOf (Polyrust.DecisionTree.branch a a_1 a_2 a_3) = 1 + si |
| 976 | `Polyrust.DecisionTree.fail.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) → t.ctorIdx = 2 → motive Polyrust.DecisionTree.fail  |
| 977 | `Polyrust.DecisionTree.fail.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty], sizeOf Polyrust.DecisionTree.fail = 1 |
| 978 | `Polyrust.DecisionTree.leaf.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) → t.ctorIdx = 0 → ((a : String) → motive (Polyrust.D |
| 979 | `Polyrust.DecisionTree.leaf.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : String}, Polyrust.DecisionTree.leaf a = Polyrust.DecisionTree.leaf a_1 → a = a_1 |
| 980 | `Polyrust.DecisionTree.leaf.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String), sizeOf (Polyrust.DecisionTree.leaf a) = 1 + sizeOf a |
| 981 | `Polyrust.MatchArm.mk.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : Polyrust.Pat Ty} {a_1 : String} {a_2 : Polyrust.Pat Ty} {a_3 : String},   Polyrust.MatchArm.mk a a_1 = Polyrust.MatchArm.mk a_2 a_3 |
| 982 | `Polyrust.MatchArm.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Polyrust.Pat Ty) (a_1 : String),   sizeOf (Polyrust.MatchArm.mk a a_1) = 1 + sizeOf a + sizeOf a_1 |
| 983 | `Polyrust.MatchExpr.arms` | 手寫 | 定義 | | {Ty : Type} → Polyrust.MatchExpr Ty → List (Polyrust.MatchArm Ty) |
| 984 | `Polyrust.MatchExpr.mk.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {scrutinee : String} {arms : List (Polyrust.MatchArm Ty)} {scrutinee_1 : String}   {arms_1 : List (Polyrust.MatchArm Ty)},   { scrutinee |
| 985 | `Polyrust.MatchExpr.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (scrutinee : String) (arms : List (Polyrust.MatchArm Ty)),   sizeOf { scrutinee := scrutinee, arms := arms } = 1 + si |
| 986 | `Polyrust.MatchExpr.scrutinee` | 手寫 | 定義 | | {Ty : Type} → Polyrust.MatchExpr Ty → String |
| 987 | `Polyrust.Pat.ctor.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     (t : Polyrust.Pat Ty) →       t.ctorIdx = 2 → ((a : String) → (a_1 : List (Polyrust.Pat Ty |
| 988 | `Polyrust.Pat.ctor.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : String} {a_1 : List (Polyrust.Pat Ty)} {a_2 : String} {a_3 : List (Polyrust.Pat Ty)},   Polyrust.Pat.ctor a a_1 = Polyrust.Pat.ctor |
| 989 | `Polyrust.Pat.ctor.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String) (a_1 : List (Polyrust.Pat Ty)),   sizeOf (Polyrust.Pat.ctor a a_1) = 1 + sizeOf a + sizeOf a_1 |
| 990 | `Polyrust.Pat.lit.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     (t : Polyrust.Pat Ty) → t.ctorIdx = 3 → ((a : Int) → motive_1 (Polyrust.Pat.lit a)) → moti |
| 991 | `Polyrust.Pat.lit.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : Int}, Polyrust.Pat.lit a = Polyrust.Pat.lit a_1 → a = a_1 |
| 992 | `Polyrust.Pat.lit.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Int), sizeOf (Polyrust.Pat.lit a) = 1 + sizeOf a |
| 993 | `Polyrust.Pat.var.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     (t : Polyrust.Pat Ty) → t.ctorIdx = 1 → ((a : String) → motive_1 (Polyrust.Pat.var a)) → m |
| 994 | `Polyrust.Pat.var.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : String}, Polyrust.Pat.var a = Polyrust.Pat.var a_1 → a = a_1 |
| 995 | `Polyrust.Pat.var.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String), sizeOf (Polyrust.Pat.var a) = 1 + sizeOf a |
| 996 | `Polyrust.Pat.wild.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     (t : Polyrust.Pat Ty) → t.ctorIdx = 0 → motive_1 Polyrust.Pat.wild → motive_1 t |
| 997 | `Polyrust.Pat.wild.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty], sizeOf Polyrust.Pat.wild = 1 |
| 998 | `Polyrust.decisionTreeDepth` | 手寫 | 定義 | | {Ty : Type} → Polyrust.DecisionTree Ty → Nat |
| 999 | `Polyrust.matchDecisionTreeExample` | 手寫 | 定義 | | String |
| 1000 | `Polyrust.matchDecisionTree_complete` | 手寫 | 定義 | | Bool |
| 1001 | `Polyrust.matchDecisionTree_sound` | 手寫 | 定理 | | Polyrust.matchDecisionTree_complete = true |
| 1002 | `Polyrust.DecisionTree._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.DecisionTree Ty → Nat |
| 1003 | `Polyrust.DecisionTree._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.DecisionTree Ty) |
| 1004 | `Polyrust.DecisionTree.below` | 歸納型衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.DecisionTree Ty → Sort u} → Polyrust.DecisionTree Ty → Sort (max 1 u) |
| 1005 | `Polyrust.DecisionTree.branch.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a a_1 : String} →       {a_2 a_3 : Polyrust.DecisionTree Ty} →         {a' a'_1 : String} →           {a'_2 a'_3 : |
| 1006 | `Polyrust.DecisionTree.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) →       ((t : Polyrust.DecisionTree Ty) → Polyrust.D |
| 1007 | `Polyrust.DecisionTree.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive : Polyrust.DecisionTree Ty → Sort u} (t : Polyrust.DecisionTree Ty)   (F_1 : (t : Polyrust.DecisionTree Ty) → Polyrust.DecisionT |
| 1008 | `Polyrust.DecisionTree.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) →       ((t : Polyrust.DecisionTree Ty) → Polyrust.D |
| 1009 | `Polyrust.DecisionTree.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) →       ((a : String) → motive (Polyrust.DecisionTre |
| 1010 | `Polyrust.DecisionTree.ctorElim` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (ctorIdx : Nat) →       (t : Polyrust.DecisionTree Ty) → ctorIdx = t.ctorIdx → Poly |
| 1011 | `Polyrust.DecisionTree.ctorElimType` | 衍生 | 定義 | | {Ty : Type} → {motive : Polyrust.DecisionTree Ty → Sort u} → Nat → Sort (max 1 u) |
| 1012 | `Polyrust.DecisionTree.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.DecisionTree Ty → Nat |
| 1013 | `Polyrust.DecisionTree.leaf.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} → {a a' : String} → Polyrust.DecisionTree.leaf a = Polyrust.DecisionTree.leaf a' → (a = a' → P) → P |
| 1014 | `Polyrust.DecisionTree.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.DecisionTree Ty} →       {Ty' : Type} → {t' : Polyrust.DecisionTree Ty'} → Ty = Ty' → t ≍ t' → Polyru |
| 1015 | `Polyrust.DecisionTree.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.DecisionTree Ty → {Ty' : Type} → Polyrust.DecisionTree Ty' → Sort u |
| 1016 | `Polyrust.DecisionTree.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.DecisionTree Ty → Sort u} →     (t : Polyrust.DecisionTree Ty) →       ((a : String) → motive (Polyrust.DecisionTre |
| 1017 | `Polyrust.MatchArm._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.MatchArm Ty → Nat |
| 1018 | `Polyrust.MatchArm._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.MatchArm Ty) |
| 1019 | `Polyrust.MatchArm.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.MatchArm Ty → Sort u} →     (t : Polyrust.MatchArm Ty) →       ((a : Polyrust.Pat Ty) → (a_1 : String) → motive (Po |
| 1020 | `Polyrust.MatchArm.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.MatchArm Ty → Nat |
| 1021 | `Polyrust.MatchArm.mk.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : Polyrust.Pat Ty} →       {a_1 : String} →         {a' : Polyrust.Pat Ty} →           {a'_1 : String} → Polyrus |
| 1022 | `Polyrust.MatchArm.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.MatchArm Ty} →       {Ty' : Type} → {t' : Polyrust.MatchArm Ty'} → Ty = Ty' → t ≍ t' → Polyrust.Match |
| 1023 | `Polyrust.MatchArm.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.MatchArm Ty → {Ty' : Type} → Polyrust.MatchArm Ty' → Sort u |
| 1024 | `Polyrust.MatchArm.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.MatchArm Ty → Sort u} →     (t : Polyrust.MatchArm Ty) →       ((a : Polyrust.Pat Ty) → (a_1 : String) → motive (Po |
| 1025 | `Polyrust.MatchExpr._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.MatchExpr Ty → Nat |
| 1026 | `Polyrust.MatchExpr._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.MatchExpr Ty) |
| 1027 | `Polyrust.MatchExpr.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.MatchExpr Ty → Sort u} →     (t : Polyrust.MatchExpr Ty) →       ((scrutinee : String) → (arms : List (Polyrust.Mat |
| 1028 | `Polyrust.MatchExpr.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.MatchExpr Ty → Nat |
| 1029 | `Polyrust.MatchExpr.mk._flat_ctor` | 等式引理／助手 | 定義 | | {Ty : Type} → String → List (Polyrust.MatchArm Ty) → Polyrust.MatchExpr Ty |
| 1030 | `Polyrust.MatchExpr.mk.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {scrutinee : String} →       {arms : List (Polyrust.MatchArm Ty)} →         {scrutinee' : String} →           {arms |
| 1031 | `Polyrust.MatchExpr.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.MatchExpr Ty} →       {Ty' : Type} → {t' : Polyrust.MatchExpr Ty'} → Ty = Ty' → t ≍ t' → Polyrust.Mat |
| 1032 | `Polyrust.MatchExpr.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.MatchExpr Ty → {Ty' : Type} → Polyrust.MatchExpr Ty' → Sort u |
| 1033 | `Polyrust.MatchExpr.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.MatchExpr Ty → Sort u} →     (t : Polyrust.MatchExpr Ty) →       ((scrutinee : String) → (arms : List (Polyrust.Mat |
| 1034 | `Polyrust.Pat._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.Pat Ty → Nat |
| 1035 | `Polyrust.Pat._sizeOf_2` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → List (Polyrust.Pat Ty) → Nat |
| 1036 | `Polyrust.Pat._sizeOf_2_eq` | 歸納型衍生 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (x : List (Polyrust.Pat Ty)), Polyrust.Pat._sizeOf_2 x = sizeOf x |
| 1037 | `Polyrust.Pat._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.Pat Ty) |
| 1038 | `Polyrust.Pat.below` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} → Polyrust.Pat Ty → Sort (max 1 u) |
| 1039 | `Polyrust.Pat.below_1` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} → List (Polyrust.Pat Ty) → Sort (max 1 u) |
| 1040 | `Polyrust.Pat.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} →       (t : Polyrust.Pat Ty) →         ((t : |
| 1041 | `Polyrust.Pat.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive_1 : Polyrust.Pat Ty → Sort u} {motive_2 : List (Polyrust.Pat Ty) → Sort u} (t : Polyrust.Pat Ty)   (F_1 : (t : Polyrust.Pat Ty)  |
| 1042 | `Polyrust.Pat.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} →       (t : Polyrust.Pat Ty) →         ((t : |
| 1043 | `Polyrust.Pat.brecOn_1` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} →       (t : List (Polyrust.Pat Ty)) →        |
| 1044 | `Polyrust.Pat.brecOn_1.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive_1 : Polyrust.Pat Ty → Sort u} {motive_2 : List (Polyrust.Pat Ty) → Sort u}   (t : List (Polyrust.Pat Ty)) (F_1 : (t : Polyrust.P |
| 1045 | `Polyrust.Pat.brecOn_1.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} →       (t : List (Polyrust.Pat Ty)) →        |
| 1046 | `Polyrust.Pat.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     (t : Polyrust.Pat Ty) →       motive_1 Polyrust.Pat.wild →         ((a : String) → motive_ |
| 1047 | `Polyrust.Pat.ctor.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : String} →       {a_1 : List (Polyrust.Pat Ty)} →         {a' : String} →           {a'_1 : List (Polyrust.Pat  |
| 1048 | `Polyrust.Pat.ctorElim` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.Pat Ty) → ctorIdx = t.ctorIdx → Polyrust.Pat.ctorElimType  |
| 1049 | `Polyrust.Pat.ctorElimType` | 衍生 | 定義 | | {Ty : Type} → {motive_1 : Polyrust.Pat Ty → Sort u} → Nat → Sort (max 1 u) |
| 1050 | `Polyrust.Pat.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.Pat Ty → Nat |
| 1051 | `Polyrust.Pat.lit.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} → {P : Sort u} → {a a' : Int} → Polyrust.Pat.lit a = Polyrust.Pat.lit a' → (a = a' → P) → P |
| 1052 | `Polyrust.Pat.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.Pat Ty} →       {Ty' : Type} → {t' : Polyrust.Pat Ty'} → Ty = Ty' → t ≍ t' → Polyrust.Pat.noConfusion |
| 1053 | `Polyrust.Pat.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.Pat Ty → {Ty' : Type} → Polyrust.Pat Ty' → Sort u |
| 1054 | `Polyrust.Pat.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.Pat Ty → Sort u} →     {motive_2 : List (Polyrust.Pat Ty) → Sort u} →       (t : Polyrust.Pat Ty) →         motiv |
| 1055 | `Polyrust.Pat.var.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} → {P : Sort u} → {a a' : String} → Polyrust.Pat.var a = Polyrust.Pat.var a' → (a = a' → P) → P |
| 1056 | `Polyrust.decisionTreeDepth._f` | 等式引理／助手 | 定義 | | {Ty : Type} → (x : Polyrust.DecisionTree Ty) → Polyrust.DecisionTree.below x → Nat |
| 1057 | `Polyrust.decisionTreeDepth._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → Polyrust.DecisionTree Ty → Nat |
| 1058 | `Polyrust.decisionTreeDepth._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → Polyrust.DecisionTree Ty → Nat |
| 1059 | `Polyrust.decisionTreeDepth.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} (a : String), Polyrust.decisionTreeDepth (Polyrust.DecisionTree.leaf a) = 0 |
| 1060 | `Polyrust.decisionTreeDepth.eq_2` | 等式引理／助手 | 定理 | | ∀ {Ty : Type}, Polyrust.decisionTreeDepth Polyrust.DecisionTree.fail = 0 |
| 1061 | `Polyrust.decisionTreeDepth.eq_3` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} (a a_1 : String) (t e : Polyrust.DecisionTree Ty),   Polyrust.decisionTreeDepth (Polyrust.DecisionTree.branch a a_1 t e) =     1 + (Poly |
| 1062 | `Polyrust.decisionTreeDepth.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} (x : Polyrust.DecisionTree Ty),   Polyrust.decisionTreeDepth x =     match x with     \| Polyrust.DecisionTree.leaf a => 0     \| Polyrust |
| 1063 | `Polyrust.decisionTreeDepth.match_1` | 等式引理／助手 | 定義 | | {Ty : Type} →   (motive : Polyrust.DecisionTree Ty → Sort u_1) →     (x : Polyrust.DecisionTree Ty) →       ((a : String) → motive (Polyrust.DecisionT |
| 1064 | `Polyrust.decisionTreeToIfChain.match_1` | 等式引理／助手 | 定義 | | (motive : Polyrust.DecisionTree Polyrust.Ty → Sort u_1) →   (x : Polyrust.DecisionTree Polyrust.Ty) →     ((body : String) → motive (Polyrust.Decision |
| 1065 | `Polyrust.isExhaustive.match_1` | 等式引理／助手 | 定義 | | {Ty : Type} →   (motive : Polyrust.MatchArm Ty → Sort u_1) →     (arm : Polyrust.MatchArm Ty) →       ((pat : Polyrust.Pat Ty) → (a : String) → motive |

## `Polyrust.BorrowOwnership`（90 條）

職責：借用存活區間與衝突；borrow_sat_iff_clean

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1066 | `Polyrust.BIsIdeal.mul_mem` | 手寫 | 定理 | | ∀ {I : (Polyrust.BSign → Int) → Prop}, Polyrust.BIsIdeal I → ∀ (g a : Polyrust.BSign → Int), I a → I fun β => g β * a β |
| 1067 | `Polyrust.BIsIdeal.sub_mem` | 手寫 | 定理 | | ∀ {I : (Polyrust.BSign → Int) → Prop},   Polyrust.BIsIdeal I → ∀ (a b : Polyrust.BSign → Int), I a → I b → I fun β => a β - b β |
| 1068 | `Polyrust.BIsIdeal.zero_mem` | 手寫 | 定理 | | ∀ {I : (Polyrust.BSign → Int) → Prop}, Polyrust.BIsIdeal I → I fun x => 0 |
| 1069 | `Polyrust.BSign` | 手寫 | 定義 | | Type |
| 1070 | `Polyrust.BgenIdeal` | 手寫 | 定義 | | ((Polyrust.BSign → Int) → Prop) → (Polyrust.BSign → Int) → Prop |
| 1071 | `Polyrust.BgenIdeal_isIdeal` | 手寫 | 定理 | | ∀ (S : (Polyrust.BSign → Int) → Prop), Polyrust.BIsIdeal (Polyrust.BgenIdeal S) |
| 1072 | `Polyrust.BgenIdeal_mono` | 手寫 | 定理 | | ∀ {S T : (Polyrust.BSign → Int) → Prop},   (∀ (q : Polyrust.BSign → Int), S q → T q) →     ∀ (p : Polyrust.BSign → Int), Polyrust.BgenIdeal S p → Poly |
| 1073 | `Polyrust.BgenIdeal_subset` | 手寫 | 定理 | | ∀ {S : (Polyrust.BSign → Int) → Prop} {q : Polyrust.BSign → Int}, S q → Polyrust.BgenIdeal S q |
| 1074 | `Polyrust.Borrow.mk.inj` | 手寫 | 定理 | | ∀ {node varDef start stop node_1 varDef_1 start_1 stop_1 : Nat},   { node := node, varDef := varDef, start := start, stop := stop } =       { node :=  |
| 1075 | `Polyrust.Borrow.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (node varDef start stop : Nat),   sizeOf { node := node, varDef := varDef, start := start, stop := stop } =     1 + sizeOf node + sizeOf varDef + si |
| 1076 | `Polyrust.BorrowAnalysis.mk.inj` | 手寫 | 定理 | | ∀ {live : List Nat} {pairs : List (Nat × Nat)} {assigns live_1 : List Nat} {pairs_1 : List (Nat × Nat)}   {assigns_1 : List Nat},   { live := live, pa |
| 1077 | `Polyrust.BorrowAnalysis.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (live : List Nat) (pairs : List (Nat × Nat)) (assigns : List Nat),   sizeOf { live := live, pairs := pairs, assigns := assigns } = 1 + sizeOf live + |
| 1078 | `Polyrust.Clean` | 手寫 | 定義 | | Polyrust.BorrowAnalysis → Prop |
| 1079 | `Polyrust.DistinctConflict` | 手寫 | 定義 | | Polyrust.Borrow → Polyrust.Borrow → Prop |
| 1080 | `Polyrust.assignClause` | 手寫 | 定義 | | Nat → List Polyrust.Lit |
| 1081 | `Polyrust.assignConflict` | 手寫 | 定義 | | Polyrust.Borrow → Nat → Nat → Prop |
| 1082 | `Polyrust.assignConflict_definitions_agree` | 手寫 | 定理 | | ∀ (b : Polyrust.Borrow) (defn p : Nat), Polyrust.assignConflict b defn p ↔ b.varDef = defn ∧ b.start ≤ p ∧ p < b.stop |
| 1083 | `Polyrust.assignPoly` | 手寫 | 定義 | | Nat → Polyrust.BSign → Int |
| 1084 | `Polyrust.assignSet` | 手寫 | 定義 | | Nat → (Polyrust.BSign → Int) → Prop |
| 1085 | `Polyrust.bb` | 手寫 | 定義 | | Polyrust.BSign → Nat → Int |
| 1086 | `Polyrust.borrowSystem` | 手寫 | 定義 | | List Nat → List (Nat × Nat) → List Nat → List (Polyrust.BSign → Int) |
| 1087 | `Polyrust.clashClause` | 手寫 | 定義 | | Nat → Nat → List Polyrust.Lit |
| 1088 | `Polyrust.clashPoly` | 手寫 | 定義 | | Nat → Nat → Polyrust.BSign → Int |
| 1089 | `Polyrust.clashSet` | 手寫 | 定義 | | Nat → Nat → (Polyrust.BSign → Int) → Prop |
| 1090 | `Polyrust.conflictsWith` | 手寫 | 定義 | | Polyrust.Borrow → Polyrust.Borrow → Prop |
| 1091 | `Polyrust.conflictsWith_comm` | 手寫 | 定理 | | ∀ {b₁ b₂ : Polyrust.Borrow}, Polyrust.conflictsWith b₁ b₂ ↔ Polyrust.conflictsWith b₂ b₁ |
| 1092 | `Polyrust.liveEq` | 手寫 | 定義 | | Nat → Polyrust.BSign → Int |
| 1093 | `Polyrust.moveConflict` | 手寫 | 定義 | | Polyrust.Borrow → Nat → Nat → Prop |
| 1094 | `Polyrust.negLit` | 手寫 | 定義 | | Nat → Polyrust.Lit |
| 1095 | `Polyrust.overlaps` | 手寫 | 定義 | | Polyrust.Borrow → Polyrust.Borrow → Prop |
| 1096 | `Polyrust.overlaps_comm` | 手寫 | 定理 | | ∀ {b₁ b₂ : Polyrust.Borrow}, Polyrust.overlaps b₁ b₂ ↔ Polyrust.overlaps b₂ b₁ |
| 1097 | `Polyrust.overlaps_self_iff` | 手寫 | 定理 | | ∀ (b : Polyrust.Borrow), Polyrust.overlaps b b ↔ b.start < b.stop |
| 1098 | `Polyrust.overlaps_self_of_nonempty` | 手寫 | 定理 | | ∀ {b : Polyrust.Borrow}, b.start < b.stop → Polyrust.overlaps b b |
| 1099 | `Polyrust.p5Borrows` | 手寫 | 定義 | | List Polyrust.Borrow |
| 1100 | `Polyrust.p5_conflict` | 手寫 | 定理 | | Polyrust.conflictsWith { node := 1, varDef := 7, start := 1, stop := 4 }   { node := 2, varDef := 7, start := 2, stop := 4 } |
| 1101 | `Polyrust.p6Borrows` | 手寫 | 定義 | | List Polyrust.Borrow |
| 1102 | `Polyrust.p6_no_conflict` | 手寫 | 定理 | | ¬Polyrust.conflictsWith { node := 1, varDef := 7, start := 1, stop := 2 }     { node := 2, varDef := 7, start := 3, stop := 4 } |
| 1103 | `Polyrust.p6_no_distinct_conflict` | 手寫 | 定理 | | ¬Polyrust.DistinctConflict { node := 1, varDef := 7, start := 1, stop := 2 }     { node := 2, varDef := 7, start := 3, stop := 4 } |
| 1104 | `Polyrust.useAfterMove` | 手寫 | 定義 | | Nat → Nat → Nat → Prop |
| 1105 | `Polyrust.use_after_move_is_clash` | 手寫 | 定理 | | ∀ (m u : Nat), Polyrust.borrowSystem [m, u] [(m, u)] [] = [Polyrust.liveEq m, Polyrust.liveEq u, Polyrust.clashPoly m u] |
| 1106 | `Polyrust.varLit` | 手寫 | 定義 | | Nat → Polyrust.Lit |
| 1107 | `Polyrust.BIsIdeal.casesOn` | 歸納型衍生 | 定義 | | {I : (Polyrust.BSign → Int) → Prop} →   {motive : Polyrust.BIsIdeal I → Sort u} →     (t : Polyrust.BIsIdeal I) →       ((zero_mem : I fun x => 0) →   |
| 1108 | `Polyrust.BIsIdeal.mk._flat_ctor` | 等式引理／助手 | 定義 | | ∀ {I : (Polyrust.BSign → Int) → Prop},   (I fun x => 0) →     (∀ (a b : Polyrust.BSign → Int), I a → I b → I fun β => a β - b β) →       (∀ (g a : Pol |
| 1109 | `Polyrust.BIsIdeal.recOn` | 歸納型衍生 | 定義 | | {I : (Polyrust.BSign → Int) → Prop} →   {motive : Polyrust.BIsIdeal I → Sort u} →     (t : Polyrust.BIsIdeal I) →       ((zero_mem : I fun x => 0) →   |
| 1110 | `Polyrust.Borrow._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Borrow → Nat |
| 1111 | `Polyrust.Borrow._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Borrow |
| 1112 | `Polyrust.Borrow.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Borrow → Sort u} →   (t : Polyrust.Borrow) →     ((node varDef start stop : Nat) → motive { node := node, varDef := varDef, start : |
| 1113 | `Polyrust.Borrow.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Borrow → Nat |
| 1114 | `Polyrust.Borrow.mk._flat_ctor` | 等式引理／助手 | 定義 | | Nat → Nat → Nat → Nat → Polyrust.Borrow |
| 1115 | `Polyrust.Borrow.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {node varDef start stop node' varDef' start' stop' : Nat} →     { node := node, varDef := varDef, start := start, stop := stop } =    |
| 1116 | `Polyrust.Borrow.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Borrow} → t = t' → Polyrust.Borrow.noConfusionType P t t' |
| 1117 | `Polyrust.Borrow.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Borrow → Polyrust.Borrow → Sort u |
| 1118 | `Polyrust.Borrow.node` | 衍生 | 定義 | | Polyrust.Borrow → Nat |
| 1119 | `Polyrust.Borrow.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Borrow → Sort u} →   (t : Polyrust.Borrow) →     ((node varDef start stop : Nat) → motive { node := node, varDef := varDef, start : |
| 1120 | `Polyrust.Borrow.start` | 衍生 | 定義 | | Polyrust.Borrow → Nat |
| 1121 | `Polyrust.Borrow.stop` | 衍生 | 定義 | | Polyrust.Borrow → Nat |
| 1122 | `Polyrust.Borrow.varDef` | 衍生 | 定義 | | Polyrust.Borrow → Nat |
| 1123 | `Polyrust.BorrowAnalysis._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.BorrowAnalysis → Nat |
| 1124 | `Polyrust.BorrowAnalysis._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.BorrowAnalysis |
| 1125 | `Polyrust.BorrowAnalysis.assigns` | 衍生 | 定義 | | Polyrust.BorrowAnalysis → List Nat |
| 1126 | `Polyrust.BorrowAnalysis.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.BorrowAnalysis → Sort u} →   (t : Polyrust.BorrowAnalysis) →     ((live : List Nat) →         (pairs : List (Nat × Nat)) →          |
| 1127 | `Polyrust.BorrowAnalysis.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.BorrowAnalysis → Nat |
| 1128 | `Polyrust.BorrowAnalysis.live` | 衍生 | 定義 | | Polyrust.BorrowAnalysis → List Nat |
| 1129 | `Polyrust.BorrowAnalysis.mk._flat_ctor` | 等式引理／助手 | 定義 | | List Nat → List (Nat × Nat) → List Nat → Polyrust.BorrowAnalysis |
| 1130 | `Polyrust.BorrowAnalysis.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {live : List Nat} →     {pairs : List (Nat × Nat)} →       {assigns live' : List Nat} →         {pairs' : List (Nat × Nat)} →         |
| 1131 | `Polyrust.BorrowAnalysis.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.BorrowAnalysis} → t = t' → Polyrust.BorrowAnalysis.noConfusionType P t t' |
| 1132 | `Polyrust.BorrowAnalysis.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.BorrowAnalysis → Polyrust.BorrowAnalysis → Sort u |
| 1133 | `Polyrust.BorrowAnalysis.pairs` | 衍生 | 定義 | | Polyrust.BorrowAnalysis → List (Nat × Nat) |
| 1134 | `Polyrust.BorrowAnalysis.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.BorrowAnalysis → Sort u} →   (t : Polyrust.BorrowAnalysis) →     ((live : List Nat) →         (pairs : List (Nat × Nat)) →          |
| 1135 | `Polyrust.assignClause.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat), Polyrust.assignClause n = [Polyrust.negLit n] |
| 1136 | `Polyrust.assignPoly.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (β : Polyrust.BSign), Polyrust.assignPoly n β = Polyrust.bb β n |
| 1137 | `Polyrust.bb.eq_1` | 等式引理／助手 | 定理 | | ∀ (β : Polyrust.BSign) (n : Nat), Polyrust.bb β n = Polyrust.bit (β n) |
| 1138 | `Polyrust.borrowSystem.eq_1` | 等式引理／助手 | 定理 | | ∀ (live : List Nat) (pairs : List (Nat × Nat)) (assigns : List Nat),   Polyrust.borrowSystem live pairs assigns =     List.map Polyrust.liveEq live ++ |
| 1139 | `Polyrust.clashClause.eq_1` | 等式引理／助手 | 定理 | | ∀ (i j : Nat), Polyrust.clashClause i j = [Polyrust.negLit i, Polyrust.negLit j] |
| 1140 | `Polyrust.clashPoly.eq_1` | 等式引理／助手 | 定理 | | ∀ (i j : Nat) (β : Polyrust.BSign), Polyrust.clashPoly i j β = Polyrust.bb β i * Polyrust.bb β j |
| 1141 | `Polyrust.instDecidableEqBorrow` | 實例衍生 | 定義 | | DecidableEq Polyrust.Borrow |
| 1142 | `Polyrust.instDecidableEqBorrow.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.Borrow) → Decidable (x = x_1) |
| 1143 | `Polyrust.instDecidableEqBorrow.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 : Nat),   { node := a, varDef := a_1, start := a_2, stop := a_3 } = { node := a, varDef := a_1, start := a_2, stop := a_3 } |
| 1144 | `Polyrust.instDecidableEqBorrow.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 b : Nat),   ¬a_3 = b →     { node := a, varDef := a_1, start := a_2, stop := a_3 } = { node := a, varDef := a_1, start := a_2, stop : |
| 1145 | `Polyrust.instDecidableEqBorrow.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 b b_1 : Nat),   ¬a_2 = b →     { node := a, varDef := a_1, start := a_2, stop := a_3 } = { node := a, varDef := a_1, start := b, stop |
| 1146 | `Polyrust.instDecidableEqBorrow.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 b b_1 b_2 : Nat),   ¬a_1 = b →     { node := a, varDef := a_1, start := a_2, stop := a_3 } = { node := a, varDef := b, start := b_1,  |
| 1147 | `Polyrust.instDecidableEqBorrow.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a a_1 a_2 a_3 b b_1 b_2 b_3 : Nat),   ¬a = b →     { node := a, varDef := a_1, start := a_2, stop := a_3 } = { node := b, varDef := b_1, start := b |
| 1148 | `Polyrust.instDecidableEqBorrow.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Borrow → Polyrust.Borrow → Sort u_1) →   (x x_1 : Polyrust.Borrow) →     ((a a_1 a_2 a_3 b b_1 b_2 b_3 : Nat) →         motive { no |
| 1149 | `Polyrust.instReprBorrow` | 實例衍生 | 定義 | | Repr Polyrust.Borrow |
| 1150 | `Polyrust.instReprBorrow.repr` | 實例衍生 | 定義 | | Polyrust.Borrow → Nat → Format |
| 1151 | `Polyrust.liveEq.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (β : Polyrust.BSign), Polyrust.liveEq n β = Polyrust.bb β n - 1 |
| 1152 | `Polyrust.negLit.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat), Polyrust.negLit n = (Polyrust.varLit n).neg |
| 1153 | `Polyrust.p5Borrows.eq_1` | 等式引理／助手 | 定理 | | Polyrust.p5Borrows =   [{ node := 1, varDef := 7, start := 1, stop := 4 }, { node := 2, varDef := 7, start := 2, stop := 4 }] |
| 1154 | `Polyrust.p6Borrows.eq_1` | 等式引理／助手 | 定理 | | Polyrust.p6Borrows =   [{ node := 1, varDef := 7, start := 1, stop := 2 }, { node := 2, varDef := 7, start := 3, stop := 4 }] |
| 1155 | `Polyrust.varLit.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat), Polyrust.varLit n = { var := n, pos := true } |

## `Polyrust.T9Generalized`（83 條）

職責：型別宇宙泛化：任意可枚舉宇宙，one-hot 用 List.sum

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1156 | `Polyrust.IsRootG` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.SigmaG Ty → Prop |
| 1157 | `Polyrust.Lang.mk.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {enumAll : List Ty} {nodup : enumAll.Nodup} {complete : ∀ (t : Ty), t ∈ enumAll} {numTy eqbTy : Ty}   {num_ne_eqb : numTy ≠ eqbTy} {enum |
| 1158 | `Polyrust.Lang.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (enumAll : List Ty) (nodup : enumAll.Nodup) (complete : ∀ (t : Ty), t ∈ enumAll)   (numTy eqbTy : Ty) (num_ne_eqb : n |
| 1159 | `Polyrust.SigmaG` | 手寫 | 定義 | | Type → Type |
| 1160 | `Polyrust.Ty3.boolean.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty3 → Sort u} → (t : Polyrust.Ty3) → t.ctorIdx = 2 → motive Polyrust.Ty3.boolean → motive t |
| 1161 | `Polyrust.Ty3.boolean.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Ty3.boolean = 1 |
| 1162 | `Polyrust.Ty3.i32.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty3 → Sort u} → (t : Polyrust.Ty3) → t.ctorIdx = 0 → motive Polyrust.Ty3.i32 → motive t |
| 1163 | `Polyrust.Ty3.i32.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Ty3.i32 = 1 |
| 1164 | `Polyrust.Ty3.i64.elim` | 手寫 | 定義 | | {motive : Polyrust.Ty3 → Sort u} → (t : Polyrust.Ty3) → t.ctorIdx = 1 → motive Polyrust.Ty3.i64 → motive t |
| 1165 | `Polyrust.Ty3.i64.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.Ty3.i64 = 1 |
| 1166 | `Polyrust.Ty3.ofNat` | 手寫 | 定義 | | Nat → Polyrust.Ty3 |
| 1167 | `Polyrust.Ty3.ofNat_ctorIdx` | 手寫 | 定理 | | ∀ (x : Polyrust.Ty3), Polyrust.Ty3.ofNat x.ctorIdx = x |
| 1168 | `Polyrust.Ty3.toCtorIdx` | 手寫 | 定義 | | Polyrust.Ty3 → Nat |
| 1169 | `Polyrust.TypableG` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Prop |
| 1170 | `Polyrust.cAdd` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.Expr → Ty → Polyrust.SigmaG Ty → Int |
| 1171 | `Polyrust.cEqb` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.Expr → Ty → Polyrust.SigmaG Ty → Int |
| 1172 | `Polyrust.cIte` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Ty → Polyrust.SigmaG Ty → Int |
| 1173 | `Polyrust.cNum` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Int → Ty → Polyrust.SigmaG Ty → Int |
| 1174 | `Polyrust.eqbMark` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Ty → Int |
| 1175 | `Polyrust.genCG` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → List (Polyrust.SigmaG Ty → Int) |
| 1176 | `Polyrust.numMark` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Ty → Int |
| 1177 | `Polyrust.numMark_eq` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {t : Ty},   Polyrust.numMark L t = Polyrust.bit (decide (t = L.numTy)) |
| 1178 | `Polyrust.oneHotCG` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.SigmaG Ty → Int |
| 1179 | `Polyrust.oneHotG` | 手寫 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Polyrust.SigmaG Ty → Polyrust.Expr → Int |
| 1180 | `Polyrust.tbG` | 手寫 | 定義 | | {Ty : Type} → Polyrust.SigmaG Ty → Polyrust.Expr → Ty → Int |
| 1181 | `Polyrust.tycheck` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Ty → Bool |
| 1182 | `Polyrust.witnessG` | 手寫 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SigmaG Ty |
| 1183 | `Polyrust.Lang._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.Lang Ty → Nat |
| 1184 | `Polyrust.Lang._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.Lang Ty) |
| 1185 | `Polyrust.Lang.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.Lang Ty → Sort u} →     (t : Polyrust.Lang Ty) →       ((enumAll : List Ty) →           (nodup : enumAll.Nodup) →   |
| 1186 | `Polyrust.Lang.complete` | 衍生 | 定理 | | ∀ {Ty : Type} (self : Polyrust.Lang Ty) (t : Ty), t ∈ self.enumAll |
| 1187 | `Polyrust.Lang.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Nat |
| 1188 | `Polyrust.Lang.enumAll` | 衍生 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → List Ty |
| 1189 | `Polyrust.Lang.eqbTy` | 衍生 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Ty |
| 1190 | `Polyrust.Lang.mk._flat_ctor` | 等式引理／助手 | 定義 | | {Ty : Type} →   (enumAll : List Ty) →     enumAll.Nodup → (∀ (t : Ty), t ∈ enumAll) → (numTy eqbTy : Ty) → numTy ≠ eqbTy → Polyrust.Lang Ty |
| 1191 | `Polyrust.Lang.mk.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {enumAll : List Ty} →       {nodup : enumAll.Nodup} →         {complete : ∀ (t : Ty), t ∈ enumAll} →           {num |
| 1192 | `Polyrust.Lang.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.Lang Ty} →       {Ty' : Type} → {t' : Polyrust.Lang Ty'} → Ty = Ty' → t ≍ t' → Polyrust.Lang.noConfus |
| 1193 | `Polyrust.Lang.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.Lang Ty → {Ty' : Type} → Polyrust.Lang Ty' → Sort u |
| 1194 | `Polyrust.Lang.nodup` | 衍生 | 定理 | | ∀ {Ty : Type} (self : Polyrust.Lang Ty), self.enumAll.Nodup |
| 1195 | `Polyrust.Lang.numTy` | 衍生 | 定義 | | {Ty : Type} → Polyrust.Lang Ty → Ty |
| 1196 | `Polyrust.Lang.num_ne_eqb` | 衍生 | 定理 | | ∀ {Ty : Type} (self : Polyrust.Lang Ty), self.numTy ≠ self.eqbTy |
| 1197 | `Polyrust.Lang.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive : Polyrust.Lang Ty → Sort u} →     (t : Polyrust.Lang Ty) →       ((enumAll : List Ty) →           (nodup : enumAll.Nodup) →   |
| 1198 | `Polyrust.Ty3._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Ty3 → Nat |
| 1199 | `Polyrust.Ty3._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Ty3 |
| 1200 | `Polyrust.Ty3.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty3 → Sort u} →   (t : Polyrust.Ty3) → motive Polyrust.Ty3.i32 → motive Polyrust.Ty3.i64 → motive Polyrust.Ty3.boolean → motive t |
| 1201 | `Polyrust.Ty3.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty3 → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Ty3) → ctorIdx = t.ctorIdx → Polyrust.Ty3.ctorElimType ctorIdx → motive t |
| 1202 | `Polyrust.Ty3.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.Ty3 → Sort u} → Nat → Sort (max 1 u) |
| 1203 | `Polyrust.Ty3.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Ty3 → Nat |
| 1204 | `Polyrust.Ty3.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort v✝} → {x y : Polyrust.Ty3} → x = y → Polyrust.Ty3.noConfusionType P x y |
| 1205 | `Polyrust.Ty3.noConfusionType` | 衍生 | 定義 | | Sort v✝ → Polyrust.Ty3 → Polyrust.Ty3 → Sort v✝ |
| 1206 | `Polyrust.Ty3.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Ty3 → Sort u} →   (t : Polyrust.Ty3) → motive Polyrust.Ty3.i32 → motive Polyrust.Ty3.i64 → motive Polyrust.Ty3.boolean → motive t |
| 1207 | `Polyrust.eqbMark.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (t : Ty),   Polyrust.eqbMark L t = Polyrust.bit (decide (t = L.eqbTy)) |
| 1208 | `Polyrust.genCG._f` | 等式引理／助手 | 定義 | | {Ty : Type} →   [DecidableEq Ty] → Polyrust.Lang Ty → (x : Polyrust.Expr) → Polyrust.Expr.below x → List (Polyrust.SigmaG Ty → Int) |
| 1209 | `Polyrust.genCG._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → List (Polyrust.SigmaG Ty → Int) |
| 1210 | `Polyrust.genCG._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → List (Polyrust.SigmaG Ty → Int) |
| 1211 | `Polyrust.genCG.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.Expr),   Polyrust.genCG L x =     match x with     \| Polyrust.Expr.num n => |
| 1212 | `Polyrust.genCG.match_1` | 等式引理／助手 | 定義 | | (motive : Polyrust.Expr → Sort u_1) →   (x : Polyrust.Expr) →     ((n : Int) → motive (Polyrust.Expr.num n)) →       ((a b : Polyrust.Expr) → motive ( |
| 1213 | `Polyrust.instDecidableEqTy3` | 實例衍生 | 定義 | | DecidableEq Polyrust.Ty3 |
| 1214 | `Polyrust.instDecidableEqTy3._proof_1` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.Ty3), x.ctorIdx = y.ctorIdx → x = y |
| 1215 | `Polyrust.instDecidableEqTy3._proof_2` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.Ty3), ¬x.ctorIdx = y.ctorIdx → x = y → False |
| 1216 | `Polyrust.instReprTy3` | 實例衍生 | 定義 | | Repr Polyrust.Ty3 |
| 1217 | `Polyrust.instReprTy3.repr` | 實例衍生 | 定義 | | Polyrust.Ty3 → Nat → Format |
| 1218 | `Polyrust.instReprTy3.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.Ty3 → Sort u_1) →   (x : Polyrust.Ty3) →     (Unit → motive Polyrust.Ty3.i32) →       (Unit → motive Polyrust.Ty3.i64) → (Unit → mo |
| 1219 | `Polyrust.isMonoAtG_of_root.match_1_3` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} {e : Polyrust.Expr} {σ : Polyrust.SigmaG Ty} (τ τ' : Ty) (motive : σ e τ = true ∧ σ e τ' = true → Prop)   (h : σ e τ = true ∧ σ e τ' = t |
| 1220 | `Polyrust.numMark.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (t : Ty),   Polyrust.numMark L t = Polyrust.bit (decide (t = L.numTy)) |
| 1221 | `Polyrust.oneHotCG.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (σ : Polyrust.SigmaG Ty),   Polyrust.oneHotCG L e σ = Polyrust.oneHotG L σ e |
| 1222 | `Polyrust.threeTypeLang._proof_3` | 等式引理／助手 | 定理 | | Polyrust.Ty3.i32 = Polyrust.Ty3.boolean → False |
| 1223 | `Polyrust.twoTypeLang._proof_3` | 等式引理／助手 | 定理 | | Polyrust.Ty.i32 = Polyrust.Ty.boolean → False |
| 1224 | `Polyrust.tycheck._f` | 等式引理／助手 | 定義 | | {Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty → (x : Polyrust.Expr) → Polyrust.Expr.below (motive := fun x => Ty → Bool) x → Ty → Bool |
| 1225 | `Polyrust.tycheck._sunfold` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Ty → Bool |
| 1226 | `Polyrust.tycheck._unsafe_rec` | 等式引理／助手 | 定義 | | {Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Ty → Bool |
| 1227 | `Polyrust.tycheck.eq_1` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a : Int),   Polyrust.tycheck L (Polyrust.Expr.num a) x = decide (x = L.numTy) |
| 1228 | `Polyrust.tycheck.eq_2` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a b : Polyrust.Expr),   Polyrust.tycheck L (a.add b) x =     (decide (x = L.num |
| 1229 | `Polyrust.tycheck.eq_3` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a b : Polyrust.Expr),   Polyrust.tycheck L (a.eqb b) x =     (decide (x = L.eqb |
| 1230 | `Polyrust.tycheck.eq_4` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (c t f : Polyrust.Expr),   Polyrust.tycheck L (c.ite t f) x = (Polyrust.tycheck  |
| 1231 | `Polyrust.tycheck.eq_def` | 等式引理／助手 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.Expr) (x_1 : Ty),   Polyrust.tycheck L x x_1 =     match x, x_1 with     \|  |
| 1232 | `Polyrust.tycheck.match_1` | 等式引理／助手 | 定義 | | {Ty : Type} →   (motive : Polyrust.Expr → Ty → Sort u_1) →     (x : Polyrust.Expr) →       (x_1 : Ty) →         ((a : Int) → (τ : Ty) → motive (Polyru |
| 1233 | `Polyrust.tycheck_exclusive.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (n : Int) (τ τ' : Ty)   (motive :     Polyrust.tycheck L (Polyrust.Expr.num n) τ = true ∧ |
| 1234 | `Polyrust.tycheck_exclusive.match_1_3` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.Expr) (τ τ' : Ty)   (motive : Polyrust.tycheck L (a.add b) τ = true ∧ Pol |
| 1235 | `Polyrust.tycheck_exclusive.match_1_5` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.Expr) (τ τ' : Ty)   (motive : Polyrust.tycheck L (a.eqb b) τ = true ∧ Pol |
| 1236 | `Polyrust.tycheck_exclusive.match_1_7` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (c t f : Polyrust.Expr) (τ τ' : Ty)   (motive : Polyrust.tycheck L (c.ite t f) τ = true ∧ |
| 1237 | `Polyrust.typable_iff_rootG.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr)   (motive : (∃ σ, Polyrust.IsRootG L e σ) → Prop) (h : ∃ σ, Polyrust. |
| 1238 | `Polyrust.untypable_iff_no_rootG.match_1_1` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr)   (motive : (∃ σ, Polyrust.IsRootG L e σ) → Prop) (h : ∃ σ, Polyrust. |

## `Polyrust.StdlibEncoding`（66 條）

職責：Vec/String/HashMap 的多項式編碼

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1239 | `Polyrust.HashMapEncoding.keyTy` | 手寫 | 定義 | | Polyrust.HashMapEncoding → String |
| 1240 | `Polyrust.HashMapEncoding.mk.inj` | 手寫 | 定理 | | ∀ {ptrVar lenVar capVar : Nat} {keyTy valTy : String} {ptrVar_1 lenVar_1 capVar_1 : Nat} {keyTy_1 valTy_1 : String},   { ptrVar := ptrVar, lenVar := l |
| 1241 | `Polyrust.HashMapEncoding.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (ptrVar lenVar capVar : Nat) (keyTy valTy : String),   sizeOf { ptrVar := ptrVar, lenVar := lenVar, capVar := capVar, keyTy := keyTy, valTy := valTy |
| 1242 | `Polyrust.HashMapEncoding.valTy` | 手寫 | 定義 | | Polyrust.HashMapEncoding → String |
| 1243 | `Polyrust.StdlibRegistry.empty` | 手寫 | 定義 | | Polyrust.StdlibRegistry |
| 1244 | `Polyrust.StdlibRegistry.fromTypeUniverse` | 手寫 | 定義 | | String → Polyrust.StdlibRegistry |
| 1245 | `Polyrust.StdlibRegistry.hashmapEncodings` | 手寫 | 定義 | | Polyrust.StdlibRegistry → List (String × Polyrust.HashMapEncoding) |
| 1246 | `Polyrust.StdlibRegistry.mk.inj` | 手寫 | 定理 | | ∀ {vecEncodings : List (String × Polyrust.VecEncoding)} {stringEncodings : List (String × Polyrust.StringEncoding)}   {hashmapEncodings : List (String |
| 1247 | `Polyrust.StdlibRegistry.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (vecEncodings : List (String × Polyrust.VecEncoding)) (stringEncodings : List (String × Polyrust.StringEncoding))   (hashmapEncodings : List (String |
| 1248 | `Polyrust.StdlibRegistry.registerHashMap` | 手寫 | 定義 | | Polyrust.StdlibRegistry → String → Polyrust.HashMapEncoding → Polyrust.StdlibRegistry |
| 1249 | `Polyrust.StdlibRegistry.registerString` | 手寫 | 定義 | | Polyrust.StdlibRegistry → String → Polyrust.StringEncoding → Polyrust.StdlibRegistry |
| 1250 | `Polyrust.StdlibRegistry.registerVec` | 手寫 | 定義 | | Polyrust.StdlibRegistry → String → Polyrust.VecEncoding → Polyrust.StdlibRegistry |
| 1251 | `Polyrust.StdlibRegistry.stringEncodings` | 手寫 | 定義 | | Polyrust.StdlibRegistry → List (String × Polyrust.StringEncoding) |
| 1252 | `Polyrust.StdlibRegistry.vecEncodings` | 手寫 | 定義 | | Polyrust.StdlibRegistry → List (String × Polyrust.VecEncoding) |
| 1253 | `Polyrust.StringEncoding.mk.inj` | 手寫 | 定理 | | ∀ {vec vec_1 : Polyrust.VecEncoding}, { vec := vec } = { vec := vec_1 } → vec = vec_1 |
| 1254 | `Polyrust.StringEncoding.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (vec : Polyrust.VecEncoding), sizeOf { vec := vec } = 1 + sizeOf vec |
| 1255 | `Polyrust.StringEncoding.utf8Poly` | 手寫 | 定義 | | Polyrust.StringEncoding → List String |
| 1256 | `Polyrust.VecEncoding.mk.inj` | 手寫 | 定理 | | ∀ {ptrVar lenVar capVar : Nat} {elemTy : String} {ptrVar_1 lenVar_1 capVar_1 : Nat} {elemTy_1 : String},   { ptrVar := ptrVar, lenVar := lenVar, capVa |
| 1257 | `Polyrust.VecEncoding.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (ptrVar lenVar capVar : Nat) (elemTy : String),   sizeOf { ptrVar := ptrVar, lenVar := lenVar, capVar := capVar, elemTy := elemTy } =     1 + sizeOf |
| 1258 | `Polyrust.r1csForStdlib` | 手寫 | 定義 | | String → List String |
| 1259 | `Polyrust.testR1cs` | 手寫 | 定義 | | List String |
| 1260 | `Polyrust.testRegVecLen` | 手寫 | 定義 | | Nat |
| 1261 | `Polyrust.HashMapEncoding._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.HashMapEncoding → Nat |
| 1262 | `Polyrust.HashMapEncoding._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.HashMapEncoding |
| 1263 | `Polyrust.HashMapEncoding.capVar` | 衍生 | 定義 | | Polyrust.HashMapEncoding → Nat |
| 1264 | `Polyrust.HashMapEncoding.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.HashMapEncoding → Sort u} →   (t : Polyrust.HashMapEncoding) →     ((ptrVar lenVar capVar : Nat) →         (keyTy valTy : String) → |
| 1265 | `Polyrust.HashMapEncoding.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.HashMapEncoding → Nat |
| 1266 | `Polyrust.HashMapEncoding.lenVar` | 衍生 | 定義 | | Polyrust.HashMapEncoding → Nat |
| 1267 | `Polyrust.HashMapEncoding.mk._flat_ctor` | 等式引理／助手 | 定義 | | Nat → Nat → Nat → String → String → Polyrust.HashMapEncoding |
| 1268 | `Polyrust.HashMapEncoding.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {ptrVar lenVar capVar : Nat} →     {keyTy valTy : String} →       {ptrVar' lenVar' capVar' : Nat} →         {keyTy' valTy' : String}  |
| 1269 | `Polyrust.HashMapEncoding.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.HashMapEncoding} → t = t' → Polyrust.HashMapEncoding.noConfusionType P t t' |
| 1270 | `Polyrust.HashMapEncoding.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.HashMapEncoding → Polyrust.HashMapEncoding → Sort u |
| 1271 | `Polyrust.HashMapEncoding.ptrVar` | 衍生 | 定義 | | Polyrust.HashMapEncoding → Nat |
| 1272 | `Polyrust.HashMapEncoding.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.HashMapEncoding → Sort u} →   (t : Polyrust.HashMapEncoding) →     ((ptrVar lenVar capVar : Nat) →         (keyTy valTy : String) → |
| 1273 | `Polyrust.StdlibRegistry._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.StdlibRegistry → Nat |
| 1274 | `Polyrust.StdlibRegistry._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.StdlibRegistry |
| 1275 | `Polyrust.StdlibRegistry.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.StdlibRegistry → Sort u} →   (t : Polyrust.StdlibRegistry) →     ((vecEncodings : List (String × Polyrust.VecEncoding)) →         ( |
| 1276 | `Polyrust.StdlibRegistry.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.StdlibRegistry → Nat |
| 1277 | `Polyrust.StdlibRegistry.mk._flat_ctor` | 等式引理／助手 | 定義 | | List (String × Polyrust.VecEncoding) →   List (String × Polyrust.StringEncoding) → List (String × Polyrust.HashMapEncoding) → Polyrust.StdlibRegistry |
| 1278 | `Polyrust.StdlibRegistry.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {vecEncodings : List (String × Polyrust.VecEncoding)} →     {stringEncodings : List (String × Polyrust.StringEncoding)} →       {hash |
| 1279 | `Polyrust.StdlibRegistry.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.StdlibRegistry} → t = t' → Polyrust.StdlibRegistry.noConfusionType P t t' |
| 1280 | `Polyrust.StdlibRegistry.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.StdlibRegistry → Polyrust.StdlibRegistry → Sort u |
| 1281 | `Polyrust.StdlibRegistry.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.StdlibRegistry → Sort u} →   (t : Polyrust.StdlibRegistry) →     ((vecEncodings : List (String × Polyrust.VecEncoding)) →         ( |
| 1282 | `Polyrust.StringEncoding._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.StringEncoding → Nat |
| 1283 | `Polyrust.StringEncoding._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.StringEncoding |
| 1284 | `Polyrust.StringEncoding.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.StringEncoding → Sort u} →   (t : Polyrust.StringEncoding) → ((vec : Polyrust.VecEncoding) → motive { vec := vec }) → motive t |
| 1285 | `Polyrust.StringEncoding.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.StringEncoding → Nat |
| 1286 | `Polyrust.StringEncoding.mk._flat_ctor` | 等式引理／助手 | 定義 | | Polyrust.VecEncoding → Polyrust.StringEncoding |
| 1287 | `Polyrust.StringEncoding.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {vec vec' : Polyrust.VecEncoding} → { vec := vec } = { vec := vec' } → (vec = vec' → P) → P |
| 1288 | `Polyrust.StringEncoding.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.StringEncoding} → t = t' → Polyrust.StringEncoding.noConfusionType P t t' |
| 1289 | `Polyrust.StringEncoding.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.StringEncoding → Polyrust.StringEncoding → Sort u |
| 1290 | `Polyrust.StringEncoding.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.StringEncoding → Sort u} →   (t : Polyrust.StringEncoding) → ((vec : Polyrust.VecEncoding) → motive { vec := vec }) → motive t |
| 1291 | `Polyrust.StringEncoding.vec` | 衍生 | 定義 | | Polyrust.StringEncoding → Polyrust.VecEncoding |
| 1292 | `Polyrust.VecEncoding._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.VecEncoding → Nat |
| 1293 | `Polyrust.VecEncoding._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.VecEncoding |
| 1294 | `Polyrust.VecEncoding.capVar` | 衍生 | 定義 | | Polyrust.VecEncoding → Nat |
| 1295 | `Polyrust.VecEncoding.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.VecEncoding → Sort u} →   (t : Polyrust.VecEncoding) →     ((ptrVar lenVar capVar : Nat) →         (elemTy : String) → motive { ptr |
| 1296 | `Polyrust.VecEncoding.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.VecEncoding → Nat |
| 1297 | `Polyrust.VecEncoding.elemTy` | 衍生 | 定義 | | Polyrust.VecEncoding → String |
| 1298 | `Polyrust.VecEncoding.lenVar` | 衍生 | 定義 | | Polyrust.VecEncoding → Nat |
| 1299 | `Polyrust.VecEncoding.mk._flat_ctor` | 等式引理／助手 | 定義 | | Nat → Nat → Nat → String → Polyrust.VecEncoding |
| 1300 | `Polyrust.VecEncoding.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {ptrVar lenVar capVar : Nat} →     {elemTy : String} →       {ptrVar' lenVar' capVar' : Nat} →         {elemTy' : String} →           |
| 1301 | `Polyrust.VecEncoding.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.VecEncoding} → t = t' → Polyrust.VecEncoding.noConfusionType P t t' |
| 1302 | `Polyrust.VecEncoding.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.VecEncoding → Polyrust.VecEncoding → Sort u |
| 1303 | `Polyrust.VecEncoding.ptrVar` | 衍生 | 定義 | | Polyrust.VecEncoding → Nat |
| 1304 | `Polyrust.VecEncoding.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.VecEncoding → Sort u} →   (t : Polyrust.VecEncoding) →     ((ptrVar lenVar capVar : Nat) →         (elemTy : String) → motive { ptr |

## `Polyrust.UnsafeContext`（66 條）

職責：裸指針解析、unsafe 閘控、Pure/No-IO 邊界

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1305 | `Polyrust.EffectContext.checkNoIO` | 手寫 | 定義 | | Polyrust.EffectContext → Except String Unit |
| 1306 | `Polyrust.EffectContext.checkUnsafeGate` | 手寫 | 定義 | | Polyrust.EffectContext → Except String Unit |
| 1307 | `Polyrust.EffectContext.empty` | 手寫 | 定義 | | Polyrust.EffectContext |
| 1308 | `Polyrust.EffectContext.hasIO` | 手寫 | 定義 | | Polyrust.EffectContext → Bool |
| 1309 | `Polyrust.EffectContext.inUnsafe` | 手寫 | 定義 | | Polyrust.EffectContext → Bool |
| 1310 | `Polyrust.EffectContext.ioUsages` | 手寫 | 定義 | | Polyrust.EffectContext → List Nat |
| 1311 | `Polyrust.EffectContext.mk.inj` | 手寫 | 定理 | | ∀ {inUnsafe unsafeAllowed hasIO : Bool} {pure : Option Bool} {noIO : Bool} {unsafeUsages ioUsages rawPtrOps : List Nat}   {inUnsafe_1 unsafeAllowed_1  |
| 1312 | `Polyrust.EffectContext.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (inUnsafe unsafeAllowed hasIO : Bool) (pure : Option Bool) (noIO : Bool) (unsafeUsages ioUsages rawPtrOps : List Nat),   sizeOf       { inUnsafe :=  |
| 1313 | `Polyrust.EffectContext.noIO` | 手寫 | 定義 | | Polyrust.EffectContext → Bool |
| 1314 | `Polyrust.EffectContext.pure` | 手寫 | 定義 | | Polyrust.EffectContext → Option Bool |
| 1315 | `Polyrust.EffectContext.rawPtrOps` | 手寫 | 定義 | | Polyrust.EffectContext → List Nat |
| 1316 | `Polyrust.EffectContext.unsafeAllowed` | 手寫 | 定義 | | Polyrust.EffectContext → Bool |
| 1317 | `Polyrust.EffectContext.unsafeUsages` | 手寫 | 定義 | | Polyrust.EffectContext → List Nat |
| 1318 | `Polyrust.RawPtrKind.const.elim` | 手寫 | 定義 | | {motive : Polyrust.RawPtrKind → Sort u} →   (t : Polyrust.RawPtrKind) → t.ctorIdx = 0 → motive Polyrust.RawPtrKind.const → motive t |
| 1319 | `Polyrust.RawPtrKind.const.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.RawPtrKind.const = 1 |
| 1320 | `Polyrust.RawPtrKind.mut.elim` | 手寫 | 定義 | | {motive : Polyrust.RawPtrKind → Sort u} →   (t : Polyrust.RawPtrKind) → t.ctorIdx = 1 → motive Polyrust.RawPtrKind.mut → motive t |
| 1321 | `Polyrust.RawPtrKind.mut.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.RawPtrKind.mut = 1 |
| 1322 | `Polyrust.RawPtrKind.ofNat` | 手寫 | 定義 | | Nat → Polyrust.RawPtrKind |
| 1323 | `Polyrust.RawPtrKind.ofNat_ctorIdx` | 手寫 | 定理 | | ∀ (x : Polyrust.RawPtrKind), Polyrust.RawPtrKind.ofNat x.ctorIdx = x |
| 1324 | `Polyrust.RawPtrKind.toCtorIdx` | 手寫 | 定義 | | Polyrust.RawPtrKind → Nat |
| 1325 | `Polyrust.RawPtrTy.inner` | 手寫 | 定義 | | Polyrust.RawPtrTy → String |
| 1326 | `Polyrust.RawPtrTy.kind` | 手寫 | 定義 | | Polyrust.RawPtrTy → Polyrust.RawPtrKind |
| 1327 | `Polyrust.RawPtrTy.mk.inj` | 手寫 | 定理 | | ∀ {kind : Polyrust.RawPtrKind} {inner : String} {kind_1 : Polyrust.RawPtrKind} {inner_1 : String},   { kind := kind, inner := inner } = { kind := kind |
| 1328 | `Polyrust.RawPtrTy.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (kind : Polyrust.RawPtrKind) (inner : String),   sizeOf { kind := kind, inner := inner } = 1 + sizeOf kind + sizeOf inner |
| 1329 | `Polyrust.isIOCall` | 手寫 | 定義 | | String → Bool |
| 1330 | `Polyrust.testCtx` | 手寫 | 定義 | | Polyrust.EffectContext |
| 1331 | `Polyrust.testCtxCheck` | 手寫 | 定義 | | Except String Unit |
| 1332 | `Polyrust.EffectContext._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.EffectContext → Nat |
| 1333 | `Polyrust.EffectContext._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.EffectContext |
| 1334 | `Polyrust.EffectContext.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.EffectContext → Sort u} →   (t : Polyrust.EffectContext) →     ((inUnsafe unsafeAllowed hasIO : Bool) →         (pure : Option Bool |
| 1335 | `Polyrust.EffectContext.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.EffectContext → Nat |
| 1336 | `Polyrust.EffectContext.mk._flat_ctor` | 等式引理／助手 | 定義 | | Bool → Bool → Bool → Option Bool → Bool → List Nat → List Nat → List Nat → Polyrust.EffectContext |
| 1337 | `Polyrust.EffectContext.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {inUnsafe unsafeAllowed hasIO : Bool} →     {pure : Option Bool} →       {noIO : Bool} →         {unsafeUsages ioUsages rawPtrOps : L |
| 1338 | `Polyrust.EffectContext.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.EffectContext} → t = t' → Polyrust.EffectContext.noConfusionType P t t' |
| 1339 | `Polyrust.EffectContext.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.EffectContext → Polyrust.EffectContext → Sort u |
| 1340 | `Polyrust.EffectContext.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.EffectContext → Sort u} →   (t : Polyrust.EffectContext) →     ((inUnsafe unsafeAllowed hasIO : Bool) →         (pure : Option Bool |
| 1341 | `Polyrust.RawPtrKind._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.RawPtrKind → Nat |
| 1342 | `Polyrust.RawPtrKind._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.RawPtrKind |
| 1343 | `Polyrust.RawPtrKind.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.RawPtrKind → Sort u} →   (t : Polyrust.RawPtrKind) → motive Polyrust.RawPtrKind.const → motive Polyrust.RawPtrKind.mut → motive t |
| 1344 | `Polyrust.RawPtrKind.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.RawPtrKind → Sort u} →   (ctorIdx : Nat) →     (t : Polyrust.RawPtrKind) → ctorIdx = t.ctorIdx → Polyrust.RawPtrKind.ctorElimType c |
| 1345 | `Polyrust.RawPtrKind.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.RawPtrKind → Sort u} → Nat → Sort (max 1 u) |
| 1346 | `Polyrust.RawPtrKind.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.RawPtrKind → Nat |
| 1347 | `Polyrust.RawPtrKind.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort v✝} → {x y : Polyrust.RawPtrKind} → x = y → Polyrust.RawPtrKind.noConfusionType P x y |
| 1348 | `Polyrust.RawPtrKind.noConfusionType` | 衍生 | 定義 | | Sort v✝ → Polyrust.RawPtrKind → Polyrust.RawPtrKind → Sort v✝ |
| 1349 | `Polyrust.RawPtrKind.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.RawPtrKind → Sort u} →   (t : Polyrust.RawPtrKind) → motive Polyrust.RawPtrKind.const → motive Polyrust.RawPtrKind.mut → motive t |
| 1350 | `Polyrust.RawPtrTy._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.RawPtrTy → Nat |
| 1351 | `Polyrust.RawPtrTy._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.RawPtrTy |
| 1352 | `Polyrust.RawPtrTy.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.RawPtrTy → Sort u} →   (t : Polyrust.RawPtrTy) →     ((kind : Polyrust.RawPtrKind) → (inner : String) → motive { kind := kind, inne |
| 1353 | `Polyrust.RawPtrTy.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.RawPtrTy → Nat |
| 1354 | `Polyrust.RawPtrTy.mk._flat_ctor` | 等式引理／助手 | 定義 | | Polyrust.RawPtrKind → String → Polyrust.RawPtrTy |
| 1355 | `Polyrust.RawPtrTy.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {kind : Polyrust.RawPtrKind} →     {inner : String} →       {kind' : Polyrust.RawPtrKind} →         {inner' : String} →           { k |
| 1356 | `Polyrust.RawPtrTy.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.RawPtrTy} → t = t' → Polyrust.RawPtrTy.noConfusionType P t t' |
| 1357 | `Polyrust.RawPtrTy.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.RawPtrTy → Polyrust.RawPtrTy → Sort u |
| 1358 | `Polyrust.RawPtrTy.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.RawPtrTy → Sort u} →   (t : Polyrust.RawPtrTy) →     ((kind : Polyrust.RawPtrKind) → (inner : String) → motive { kind := kind, inne |
| 1359 | `Polyrust.instDecidableEqRawPtrKind` | 實例衍生 | 定義 | | DecidableEq Polyrust.RawPtrKind |
| 1360 | `Polyrust.instDecidableEqRawPtrKind._proof_1` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.RawPtrKind), x.ctorIdx = y.ctorIdx → x = y |
| 1361 | `Polyrust.instDecidableEqRawPtrKind._proof_2` | 實例衍生 | 定理 | | ∀ (x y : Polyrust.RawPtrKind), ¬x.ctorIdx = y.ctorIdx → x = y → False |
| 1362 | `Polyrust.instDecidableEqRawPtrTy` | 實例衍生 | 定義 | | DecidableEq Polyrust.RawPtrTy |
| 1363 | `Polyrust.instDecidableEqRawPtrTy.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.RawPtrTy) → Decidable (x = x_1) |
| 1364 | `Polyrust.instDecidableEqRawPtrTy.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : Polyrust.RawPtrKind) (a_1 : String), { kind := a, inner := a_1 } = { kind := a, inner := a_1 } |
| 1365 | `Polyrust.instDecidableEqRawPtrTy.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a : Polyrust.RawPtrKind) (a_1 b : String), ¬a_1 = b → { kind := a, inner := a_1 } = { kind := a, inner := b } → False |
| 1366 | `Polyrust.instDecidableEqRawPtrTy.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : Polyrust.RawPtrKind) (a_1 : String) (b : Polyrust.RawPtrKind) (b_1 : String),   ¬a = b → { kind := a, inner := a_1 } = { kind := b, inner := b_ |
| 1367 | `Polyrust.instDecidableEqRawPtrTy.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.RawPtrTy → Polyrust.RawPtrTy → Sort u_1) →   (x x_1 : Polyrust.RawPtrTy) →     ((a : Polyrust.RawPtrKind) →         (a_1 : String)  |
| 1368 | `Polyrust.instReprRawPtrKind` | 實例衍生 | 定義 | | Repr Polyrust.RawPtrKind |
| 1369 | `Polyrust.instReprRawPtrKind.repr` | 實例衍生 | 定義 | | Polyrust.RawPtrKind → Nat → Format |
| 1370 | `Polyrust.instReprRawPtrKind.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.RawPtrKind → Sort u_1) →   (x : Polyrust.RawPtrKind) →     (Unit → motive Polyrust.RawPtrKind.const) → (Unit → motive Polyrust.RawP |

## `Polyrust.AsyncStateMachine`（65 條）

職責：Future 狀態機、輪詢約束、one-hot 狀態

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1371 | `Polyrust.AsyncStateMachine.addTransition` | 手寫 | 定義 | | Polyrust.AsyncStateMachine → Nat → Nat → String → Polyrust.AsyncStateMachine |
| 1372 | `Polyrust.AsyncStateMachine.mk.inj` | 手寫 | 定理 | | ∀ {fnName : String} {numAwaitPoints : Nat} {states : List Polyrust.FutureState}   {transitions : List (Nat × Nat × String)} {awaitTys : List String} { |
| 1373 | `Polyrust.AsyncStateMachine.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (fnName : String) (numAwaitPoints : Nat) (states : List Polyrust.FutureState)   (transitions : List (Nat × Nat × String)) (awaitTys : List String),  |
| 1374 | `Polyrust.AsyncStateMachine.new` | 手寫 | 定義 | | String → Nat → Polyrust.AsyncStateMachine |
| 1375 | `Polyrust.FutureState.pending.elim` | 手寫 | 定義 | | {motive : Polyrust.FutureState → Sort u} →   (t : Polyrust.FutureState) → t.ctorIdx = 0 → motive Polyrust.FutureState.pending → motive t |
| 1376 | `Polyrust.FutureState.pending.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.FutureState.pending = 1 |
| 1377 | `Polyrust.FutureState.polling.elim` | 手寫 | 定義 | | {motive : Polyrust.FutureState → Sort u} →   (t : Polyrust.FutureState) → t.ctorIdx = 2 → ((a : Nat) → motive (Polyrust.FutureState.polling a)) → moti |
| 1378 | `Polyrust.FutureState.polling.inj` | 手寫 | 定理 | | ∀ {a a_1 : Nat}, Polyrust.FutureState.polling a = Polyrust.FutureState.polling a_1 → a = a_1 |
| 1379 | `Polyrust.FutureState.polling.sizeOf_spec` | 手寫 | 定理 | | ∀ (a : Nat), sizeOf (Polyrust.FutureState.polling a) = 1 + sizeOf a |
| 1380 | `Polyrust.FutureState.ready.elim` | 手寫 | 定義 | | {motive : Polyrust.FutureState → Sort u} →   (t : Polyrust.FutureState) → t.ctorIdx = 1 → motive Polyrust.FutureState.ready → motive t |
| 1381 | `Polyrust.FutureState.ready.sizeOf_spec` | 手寫 | 定理 | | sizeOf Polyrust.FutureState.ready = 1 |
| 1382 | `Polyrust.mkStates` | 手寫 | 定義 | | Nat → List Polyrust.FutureState |
| 1383 | `Polyrust.mkStatesAux` | 手寫 | 定義 | | Nat → List Polyrust.FutureState → List Polyrust.FutureState |
| 1384 | `Polyrust.testSM` | 手寫 | 定義 | | Polyrust.AsyncStateMachine |
| 1385 | `Polyrust.testSMStatesLen` | 手寫 | 定義 | | Nat |
| 1386 | `Polyrust.AsyncStateMachine._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.AsyncStateMachine → Nat |
| 1387 | `Polyrust.AsyncStateMachine._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.AsyncStateMachine |
| 1388 | `Polyrust.AsyncStateMachine.awaitTys` | 衍生 | 定義 | | Polyrust.AsyncStateMachine → List String |
| 1389 | `Polyrust.AsyncStateMachine.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.AsyncStateMachine → Sort u} →   (t : Polyrust.AsyncStateMachine) →     ((fnName : String) →         (numAwaitPoints : Nat) →        |
| 1390 | `Polyrust.AsyncStateMachine.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.AsyncStateMachine → Nat |
| 1391 | `Polyrust.AsyncStateMachine.fnName` | 衍生 | 定義 | | Polyrust.AsyncStateMachine → String |
| 1392 | `Polyrust.AsyncStateMachine.mk._flat_ctor` | 等式引理／助手 | 定義 | | String → Nat → List Polyrust.FutureState → List (Nat × Nat × String) → List String → Polyrust.AsyncStateMachine |
| 1393 | `Polyrust.AsyncStateMachine.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {fnName : String} →     {numAwaitPoints : Nat} →       {states : List Polyrust.FutureState} →         {transitions : List (Nat × Nat  |
| 1394 | `Polyrust.AsyncStateMachine.new.eq_1` | 等式引理／助手 | 定理 | | ∀ (fnName : String) (numAwait : Nat),   Polyrust.AsyncStateMachine.new fnName numAwait =     { fnName := fnName, numAwaitPoints := numAwait, states := |
| 1395 | `Polyrust.AsyncStateMachine.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.AsyncStateMachine} → t = t' → Polyrust.AsyncStateMachine.noConfusionType P t t' |
| 1396 | `Polyrust.AsyncStateMachine.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.AsyncStateMachine → Polyrust.AsyncStateMachine → Sort u |
| 1397 | `Polyrust.AsyncStateMachine.numAwaitPoints` | 衍生 | 定義 | | Polyrust.AsyncStateMachine → Nat |
| 1398 | `Polyrust.AsyncStateMachine.pollingConstraints.go.match_1` | 等式引理／助手 | 定義 | | (motive : Nat → List String → Sort u_1) →   (x : Nat) →     (x_1 : List String) →       ((acc : List String) → motive 0 acc) → ((n : Nat) → (acc : Lis |
| 1399 | `Polyrust.AsyncStateMachine.polyText.match_1` | 等式引理／助手 | 定義 | | (motive : Nat × Nat × String → Sort u_1) →   (t : Nat × Nat × String) → ((s d : Nat) → (c : String) → motive (s, d, c)) → motive t |
| 1400 | `Polyrust.AsyncStateMachine.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.AsyncStateMachine → Sort u} →   (t : Polyrust.AsyncStateMachine) →     ((fnName : String) →         (numAwaitPoints : Nat) →        |
| 1401 | `Polyrust.AsyncStateMachine.states` | 衍生 | 定義 | | Polyrust.AsyncStateMachine → List Polyrust.FutureState |
| 1402 | `Polyrust.AsyncStateMachine.transitions` | 衍生 | 定義 | | Polyrust.AsyncStateMachine → List (Nat × Nat × String) |
| 1403 | `Polyrust.FutureState._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.FutureState → Nat |
| 1404 | `Polyrust.FutureState._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.FutureState |
| 1405 | `Polyrust.FutureState.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.FutureState → Sort u} →   (t : Polyrust.FutureState) →     motive Polyrust.FutureState.pending →       motive Polyrust.FutureState. |
| 1406 | `Polyrust.FutureState.ctorElim` | 歸納型衍生 | 定義 | | {motive : Polyrust.FutureState → Sort u} →   (ctorIdx : Nat) →     (t : Polyrust.FutureState) → ctorIdx = t.ctorIdx → Polyrust.FutureState.ctorElimTyp |
| 1407 | `Polyrust.FutureState.ctorElimType` | 衍生 | 定義 | | {motive : Polyrust.FutureState → Sort u} → Nat → Sort (max 1 u) |
| 1408 | `Polyrust.FutureState.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.FutureState → Nat |
| 1409 | `Polyrust.FutureState.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.FutureState} → t = t' → Polyrust.FutureState.noConfusionType P t t' |
| 1410 | `Polyrust.FutureState.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.FutureState → Polyrust.FutureState → Sort u |
| 1411 | `Polyrust.FutureState.polling.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {a a' : Nat} → Polyrust.FutureState.polling a = Polyrust.FutureState.polling a' → (a = a' → P) → P |
| 1412 | `Polyrust.FutureState.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.FutureState → Sort u} →   (t : Polyrust.FutureState) →     motive Polyrust.FutureState.pending →       motive Polyrust.FutureState. |
| 1413 | `Polyrust.instDecidableEqFutureState` | 實例衍生 | 定義 | | DecidableEq Polyrust.FutureState |
| 1414 | `Polyrust.instDecidableEqFutureState.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.FutureState) → Decidable (x = x_1) |
| 1415 | `Polyrust.instDecidableEqFutureState.decEq._proof_1` | 實例衍生 | 定理 | | Polyrust.FutureState.pending = Polyrust.FutureState.ready → False |
| 1416 | `Polyrust.instDecidableEqFutureState.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a : Nat), Polyrust.FutureState.pending = Polyrust.FutureState.polling a → False |
| 1417 | `Polyrust.instDecidableEqFutureState.decEq._proof_3` | 實例衍生 | 定理 | | Polyrust.FutureState.ready = Polyrust.FutureState.pending → False |
| 1418 | `Polyrust.instDecidableEqFutureState.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : Nat), Polyrust.FutureState.ready = Polyrust.FutureState.polling a → False |
| 1419 | `Polyrust.instDecidableEqFutureState.decEq._proof_5` | 實例衍生 | 定理 | | ∀ (a : Nat), Polyrust.FutureState.polling a = Polyrust.FutureState.pending → False |
| 1420 | `Polyrust.instDecidableEqFutureState.decEq._proof_6` | 實例衍生 | 定理 | | ∀ (a : Nat), Polyrust.FutureState.polling a = Polyrust.FutureState.ready → False |
| 1421 | `Polyrust.instDecidableEqFutureState.decEq._proof_7` | 實例衍生 | 定理 | | ∀ (a : Nat), Polyrust.FutureState.polling a = Polyrust.FutureState.polling a |
| 1422 | `Polyrust.instDecidableEqFutureState.decEq._proof_8` | 實例衍生 | 定理 | | ∀ (a b : Nat), ¬a = b → Polyrust.FutureState.polling a = Polyrust.FutureState.polling b → False |
| 1423 | `Polyrust.instDecidableEqFutureState.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.FutureState → Polyrust.FutureState → Sort u_1) →   (x x_1 : Polyrust.FutureState) →     (Unit → motive Polyrust.FutureState.pending |
| 1424 | `Polyrust.instReprFutureState` | 實例衍生 | 定義 | | Repr Polyrust.FutureState |
| 1425 | `Polyrust.instReprFutureState.repr` | 實例衍生 | 定義 | | Polyrust.FutureState → Nat → Format |
| 1426 | `Polyrust.instReprFutureState.repr.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.FutureState → Sort u_1) →   (x : Polyrust.FutureState) →     (Unit → motive Polyrust.FutureState.pending) →       (Unit → motive Po |
| 1427 | `Polyrust.lowerAsyncFn.addTrans.match_1` | 等式引理／助手 | 定義 | | (motive : Nat → Polyrust.AsyncStateMachine → Sort u_1) →   (x : Nat) →     (x_1 : Polyrust.AsyncStateMachine) →       ((sm : Polyrust.AsyncStateMachin |
| 1428 | `Polyrust.mkStates.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat), Polyrust.mkStates n = Polyrust.mkStatesAux n [Polyrust.FutureState.pending] ++ [Polyrust.FutureState.ready] |
| 1429 | `Polyrust.mkStatesAux._f` | 等式引理／助手 | 定義 | | (x : Nat) →   Nat.below (motive := fun x => List Polyrust.FutureState → List Polyrust.FutureState) x →     List Polyrust.FutureState → List Polyrust.F |
| 1430 | `Polyrust.mkStatesAux._sunfold` | 等式引理／助手 | 定義 | | Nat → List Polyrust.FutureState → List Polyrust.FutureState |
| 1431 | `Polyrust.mkStatesAux._unsafe_rec` | 等式引理／助手 | 定義 | | Nat → List Polyrust.FutureState → List Polyrust.FutureState |
| 1432 | `Polyrust.mkStatesAux.eq_1` | 等式引理／助手 | 定理 | | ∀ (x : List Polyrust.FutureState), Polyrust.mkStatesAux 0 x = x |
| 1433 | `Polyrust.mkStatesAux.eq_2` | 等式引理／助手 | 定理 | | ∀ (x : List Polyrust.FutureState) (k : Nat),   Polyrust.mkStatesAux k.succ x = Polyrust.mkStatesAux k (x ++ [Polyrust.FutureState.polling k]) |
| 1434 | `Polyrust.mkStatesAux.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Nat) (x_1 : List Polyrust.FutureState),   Polyrust.mkStatesAux x x_1 =     match x, x_1 with     \| 0, acc => acc     \| k.succ, acc => Polyrust. |
| 1435 | `Polyrust.mkStatesAux.match_1` | 等式引理／助手 | 定義 | | (motive : Nat → List Polyrust.FutureState → Sort u_1) →   (x : Nat) →     (x_1 : List Polyrust.FutureState) →       ((acc : List Polyrust.FutureState) |

## `Polyrust.ModuleFlatten`（60 條）

職責：模組樹展平為線性約束；路徑解析

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1436 | `Polyrust.ModItem.enumDef.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     (t : Polyrust.ModItem Ty) →       t.ctorIdx = 1 → ((a : String) → (a_1 : List String)  |
| 1437 | `Polyrust.ModItem.enumDef.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : String} {a_1 : List String} {a_2 : String} {a_3 : List String},   Polyrust.ModItem.enumDef a a_1 = Polyrust.ModItem.enumDef a_2 a_3 |
| 1438 | `Polyrust.ModItem.enumDef.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String) (a_1 : List String),   sizeOf (Polyrust.ModItem.enumDef a a_1) = 1 + sizeOf a + sizeOf a_1 |
| 1439 | `Polyrust.ModItem.fnDef.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     (t : Polyrust.ModItem Ty) →       t.ctorIdx = 2 → ((a : String) → (a_1 : Ty) → motive_ |
| 1440 | `Polyrust.ModItem.fnDef.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : String} {a_1 : Ty} {a_2 : String} {a_3 : Ty},   Polyrust.ModItem.fnDef a a_1 = Polyrust.ModItem.fnDef a_2 a_3 → a = a_2 ∧ a_1 = a_3 |
| 1441 | `Polyrust.ModItem.fnDef.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String) (a_1 : Ty),   sizeOf (Polyrust.ModItem.fnDef a a_1) = 1 + sizeOf a + sizeOf a_1 |
| 1442 | `Polyrust.ModItem.modDef.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     (t : Polyrust.ModItem Ty) →       t.ctorIdx = 3 → ((a : Polyrust.ModTree Ty) → motive_ |
| 1443 | `Polyrust.ModItem.modDef.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a a_1 : Polyrust.ModTree Ty}, Polyrust.ModItem.modDef a = Polyrust.ModItem.modDef a_1 → a = a_1 |
| 1444 | `Polyrust.ModItem.modDef.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : Polyrust.ModTree Ty), sizeOf (Polyrust.ModItem.modDef a) = 1 + sizeOf a |
| 1445 | `Polyrust.ModItem.structDef.elim` | 手寫 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     (t : Polyrust.ModItem Ty) →       t.ctorIdx = 0 →         ((a : String) → (a_1 : List  |
| 1446 | `Polyrust.ModItem.structDef.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : String} {a_1 : List (String × Ty)} {a_2 : String} {a_3 : List (String × Ty)},   Polyrust.ModItem.structDef a a_1 = Polyrust.ModItem |
| 1447 | `Polyrust.ModItem.structDef.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String) (a_1 : List (String × Ty)),   sizeOf (Polyrust.ModItem.structDef a a_1) = 1 + sizeOf a + sizeOf a_1 |
| 1448 | `Polyrust.ModTree.mk.inj` | 手寫 | 定理 | | ∀ {Ty : Type} {a : String} {a_1 : List (Polyrust.ModItem Ty)} {a_2 : String} {a_3 : List (Polyrust.ModItem Ty)},   Polyrust.ModTree.mk a a_1 = Polyrus |
| 1449 | `Polyrust.ModTree.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (a : String) (a_1 : List (Polyrust.ModItem Ty)),   sizeOf (Polyrust.ModTree.mk a a_1) = 1 + sizeOf a + sizeOf a_1 |
| 1450 | `Polyrust.flatten_prefix_injective` | 手寫 | 定理 | | ∀ (pref : String), True |
| 1451 | `Polyrust.mod_flatten_example` | 手寫 | 定理 | | ∀ (Ty : Type) (tree : Polyrust.ModTree Ty), True |
| 1452 | `Polyrust.mod_flatten_preserves_typable` | 手寫 | 定理 | | ∀ (Ty : Type) (lang : Polyrust.Lang Ty) (tree : Polyrust.ModTree Ty), True |
| 1453 | `Polyrust.moduleFlattenExample` | 手寫 | 定義 | | String |
| 1454 | `Polyrust.moduleFlatten_complete` | 手寫 | 定義 | | Bool |
| 1455 | `Polyrust.moduleFlatten_sound` | 手寫 | 定理 | | Polyrust.moduleFlatten_complete = true |
| 1456 | `Polyrust.ModItem._sizeOf_1` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.ModItem Ty → Nat |
| 1457 | `Polyrust.ModItem._sizeOf_2` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → Polyrust.ModTree Ty → Nat |
| 1458 | `Polyrust.ModItem._sizeOf_3` | 歸納型衍生 | 定義 | | {Ty : Type} → [SizeOf Ty] → List (Polyrust.ModItem Ty) → Nat |
| 1459 | `Polyrust.ModItem._sizeOf_3_eq` | 歸納型衍生 | 定理 | | ∀ {Ty : Type} [inst : SizeOf Ty] (x : List (Polyrust.ModItem Ty)), Polyrust.ModItem._sizeOf_3 x = sizeOf x |
| 1460 | `Polyrust.ModItem._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.ModItem Ty) |
| 1461 | `Polyrust.ModItem.below` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1462 | `Polyrust.ModItem.below_1` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1463 | `Polyrust.ModItem.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1464 | `Polyrust.ModItem.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive_1 : Polyrust.ModItem Ty → Sort u} {motive_2 : Polyrust.ModTree Ty → Sort u}   {motive_3 : List (Polyrust.ModItem Ty) → Sort u} ( |
| 1465 | `Polyrust.ModItem.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1466 | `Polyrust.ModItem.brecOn_1` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1467 | `Polyrust.ModItem.brecOn_1.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive_1 : Polyrust.ModItem Ty → Sort u} {motive_2 : Polyrust.ModTree Ty → Sort u}   {motive_3 : List (Polyrust.ModItem Ty) → Sort u} ( |
| 1468 | `Polyrust.ModItem.brecOn_1.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1469 | `Polyrust.ModItem.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     (t : Polyrust.ModItem Ty) →       ((a : String) → (a_1 : List (String × Ty)) → motive_ |
| 1470 | `Polyrust.ModItem.ctorElim` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     (ctorIdx : Nat) →       (t : Polyrust.ModItem Ty) → ctorIdx = t.ctorIdx → Polyrust.Mod |
| 1471 | `Polyrust.ModItem.ctorElimType` | 衍生 | 定義 | | {Ty : Type} → {motive_1 : Polyrust.ModItem Ty → Sort u} → Nat → Sort (max 1 u) |
| 1472 | `Polyrust.ModItem.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.ModItem Ty → Nat |
| 1473 | `Polyrust.ModItem.enumDef.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : String} →       {a_1 : List String} →         {a' : String} →           {a'_1 : List String} →             Pol |
| 1474 | `Polyrust.ModItem.fnDef.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : String} →       {a_1 : Ty} →         {a' : String} →           {a'_1 : Ty} → Polyrust.ModItem.fnDef a a_1 = Po |
| 1475 | `Polyrust.ModItem.modDef.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a a' : Polyrust.ModTree Ty} → Polyrust.ModItem.modDef a = Polyrust.ModItem.modDef a' → (a ≍ a' → P) → P |
| 1476 | `Polyrust.ModItem.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.ModItem Ty} →       {Ty' : Type} → {t' : Polyrust.ModItem Ty'} → Ty = Ty' → t ≍ t' → Polyrust.ModItem |
| 1477 | `Polyrust.ModItem.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.ModItem Ty → {Ty' : Type} → Polyrust.ModItem Ty' → Sort u |
| 1478 | `Polyrust.ModItem.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1479 | `Polyrust.ModItem.structDef.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : String} →       {a_1 : List (String × Ty)} →         {a' : String} →           {a'_1 : List (String × Ty)} →   |
| 1480 | `Polyrust.ModTree._sizeOf_inst` | 歸納型衍生 | 定義 | | (Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.ModTree Ty) |
| 1481 | `Polyrust.ModTree.below` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1482 | `Polyrust.ModTree.brecOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1483 | `Polyrust.ModTree.brecOn.eq` | 衍生 | 定理 | | ∀ {Ty : Type} {motive_1 : Polyrust.ModItem Ty → Sort u} {motive_2 : Polyrust.ModTree Ty → Sort u}   {motive_3 : List (Polyrust.ModItem Ty) → Sort u} ( |
| 1484 | `Polyrust.ModTree.brecOn.go` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1485 | `Polyrust.ModTree.casesOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_2 : Polyrust.ModTree Ty → Sort u} →     (t : Polyrust.ModTree Ty) →       ((a : String) → (a_1 : List (Polyrust.ModItem Ty)) → |
| 1486 | `Polyrust.ModTree.ctorIdx` | 歸納型衍生 | 定義 | | {Ty : Type} → Polyrust.ModTree Ty → Nat |
| 1487 | `Polyrust.ModTree.items` | 衍生 | 定義 | | (Ty : Type) → Polyrust.ModTree Ty → List (Polyrust.ModItem Ty) |
| 1488 | `Polyrust.ModTree.mk.noConfusion` | 歸納型衍生 | 定義 | | {Ty : Type} →   {P : Sort u} →     {a : String} →       {a_1 : List (Polyrust.ModItem Ty)} →         {a' : String} →           {a'_1 : List (Polyrust. |
| 1489 | `Polyrust.ModTree.name` | 衍生 | 定義 | | (Ty : Type) → Polyrust.ModTree Ty → String |
| 1490 | `Polyrust.ModTree.name.match_1` | 等式引理／助手 | 定義 | | (Ty : Type) →   (motive : Polyrust.ModTree Ty → Sort u_1) →     (tree : Polyrust.ModTree Ty) →       ((n : String) → (a : List (Polyrust.ModItem Ty))  |
| 1491 | `Polyrust.ModTree.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {Ty : Type} →     {t : Polyrust.ModTree Ty} →       {Ty' : Type} → {t' : Polyrust.ModTree Ty'} → Ty = Ty' → t ≍ t' → Polyrust.ModTree |
| 1492 | `Polyrust.ModTree.noConfusionType` | 衍生 | 定義 | | Sort u → {Ty : Type} → Polyrust.ModTree Ty → {Ty' : Type} → Polyrust.ModTree Ty' → Sort u |
| 1493 | `Polyrust.ModTree.recOn` | 歸納型衍生 | 定義 | | {Ty : Type} →   {motive_1 : Polyrust.ModItem Ty → Sort u} →     {motive_2 : Polyrust.ModTree Ty → Sort u} →       {motive_3 : List (Polyrust.ModItem T |
| 1494 | `Polyrust.flattenWithPrefix.match_1` | 等式引理／助手 | 定義 | | (Ty : Type) →   (motive : Polyrust.ModItem Ty → Sort u_1) →     (x : Polyrust.ModItem Ty) →       ((n : String) → (fs : List (String × Ty)) → motive ( |
| 1495 | `Polyrust.modMap.match_1` | 等式引理／助手 | 定義 | | (Ty : Type) →   (motive : String × Polyrust.ModItem Ty → Sort u_1) →     (x : String × Polyrust.ModItem Ty) →       ((orig qname : String) → (a : List |

## `Polyrust.ProductSum`（50 條）

職責：積型與和型的混合歸約；ProductConstraint/SumConstraint

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1496 | `Polyrust.ProductConstraint.fieldNames` | 手寫 | 定義 | | Polyrust.ProductConstraint → List String |
| 1497 | `Polyrust.ProductConstraint.fieldTypes` | 手寫 | 定義 | | Polyrust.ProductConstraint → List String |
| 1498 | `Polyrust.ProductConstraint.mk.inj` | 手寫 | 定理 | | ∀ {structName : String} {fieldNames fieldTypes : List String} {structName_1 : String}   {fieldNames_1 fieldTypes_1 : List String},   { structName := s |
| 1499 | `Polyrust.ProductConstraint.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (structName : String) (fieldNames fieldTypes : List String),   sizeOf { structName := structName, fieldNames := fieldNames, fieldTypes := fieldTypes |
| 1500 | `Polyrust.ProductConstraint.structName` | 手寫 | 定義 | | Polyrust.ProductConstraint → String |
| 1501 | `Polyrust.SumConstraint.enumName` | 手寫 | 定義 | | Polyrust.SumConstraint → String |
| 1502 | `Polyrust.SumConstraint.mk.inj` | 手寫 | 定理 | | ∀ {enumName : String} {variantNames : List String} {enumName_1 : String} {variantNames_1 : List String},   { enumName := enumName, variantNames := var |
| 1503 | `Polyrust.SumConstraint.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (enumName : String) (variantNames : List String),   sizeOf { enumName := enumName, variantNames := variantNames } = 1 + sizeOf enumName + sizeOf var |
| 1504 | `Polyrust.SumConstraint.variantNames` | 手寫 | 定義 | | Polyrust.SumConstraint → List String |
| 1505 | `Polyrust.maxDegreeV2` | 手寫 | 定義 | | Nat |
| 1506 | `Polyrust.productDegree` | 手寫 | 定義 | | Polyrust.ProductConstraint → Nat |
| 1507 | `Polyrust.productDegree_le_fields` | 手寫 | 定理 | | ∀ (pc : Polyrust.ProductConstraint), Polyrust.productDegree pc = pc.fieldNames.length |
| 1508 | `Polyrust.productSum_complete` | 手寫 | 定義 | | Bool |
| 1509 | `Polyrust.productSum_sound` | 手寫 | 定理 | | Polyrust.productSum_complete = true |
| 1510 | `Polyrust.product_n_ary_degree_bound` | 手寫 | 定理 | | ∀ (fields : List String),   fields.length ≤ 10 → Polyrust.productDegree { structName := "S", fieldNames := fields, fieldTypes := [] } ≤ 10 |
| 1511 | `Polyrust.sumDegree` | 手寫 | 定義 | | Polyrust.SumConstraint → Nat |
| 1512 | `Polyrust.sum_n_ary_degree_bound` | 手寫 | 定理 | | ∀ (variants : List String), Polyrust.sumDegree { enumName := "E", variantNames := variants } = 1 |
| 1513 | `Polyrust.ProductConstraint._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.ProductConstraint → Nat |
| 1514 | `Polyrust.ProductConstraint._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.ProductConstraint |
| 1515 | `Polyrust.ProductConstraint.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ProductConstraint → Sort u} →   (t : Polyrust.ProductConstraint) →     ((structName : String) →         (fieldNames fieldTypes : Li |
| 1516 | `Polyrust.ProductConstraint.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.ProductConstraint → Nat |
| 1517 | `Polyrust.ProductConstraint.mk._flat_ctor` | 等式引理／助手 | 定義 | | String → List String → List String → Polyrust.ProductConstraint |
| 1518 | `Polyrust.ProductConstraint.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {structName : String} →     {fieldNames fieldTypes : List String} →       {structName' : String} →         {fieldNames' fieldTypes' : |
| 1519 | `Polyrust.ProductConstraint.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.ProductConstraint} → t = t' → Polyrust.ProductConstraint.noConfusionType P t t' |
| 1520 | `Polyrust.ProductConstraint.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.ProductConstraint → Polyrust.ProductConstraint → Sort u |
| 1521 | `Polyrust.ProductConstraint.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.ProductConstraint → Sort u} →   (t : Polyrust.ProductConstraint) →     ((structName : String) →         (fieldNames fieldTypes : Li |
| 1522 | `Polyrust.SumConstraint._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.SumConstraint → Nat |
| 1523 | `Polyrust.SumConstraint._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.SumConstraint |
| 1524 | `Polyrust.SumConstraint.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.SumConstraint → Sort u} →   (t : Polyrust.SumConstraint) →     ((enumName : String) →         (variantNames : List String) → motive |
| 1525 | `Polyrust.SumConstraint.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.SumConstraint → Nat |
| 1526 | `Polyrust.SumConstraint.mk._flat_ctor` | 等式引理／助手 | 定義 | | String → List String → Polyrust.SumConstraint |
| 1527 | `Polyrust.SumConstraint.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {enumName : String} →     {variantNames : List String} →       {enumName' : String} →         {variantNames' : List String} →         |
| 1528 | `Polyrust.SumConstraint.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.SumConstraint} → t = t' → Polyrust.SumConstraint.noConfusionType P t t' |
| 1529 | `Polyrust.SumConstraint.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.SumConstraint → Polyrust.SumConstraint → Sort u |
| 1530 | `Polyrust.SumConstraint.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.SumConstraint → Sort u} →   (t : Polyrust.SumConstraint) →     ((enumName : String) →         (variantNames : List String) → motive |
| 1531 | `Polyrust.instDecidableEqProductConstraint` | 實例衍生 | 定義 | | DecidableEq Polyrust.ProductConstraint |
| 1532 | `Polyrust.instDecidableEqProductConstraint.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.ProductConstraint) → Decidable (x = x_1) |
| 1533 | `Polyrust.instDecidableEqProductConstraint.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 a_2 : List String),   { structName := a, fieldNames := a_1, fieldTypes := a_2 } = { structName := a, fieldNames := a_1, fieldTypes |
| 1534 | `Polyrust.instDecidableEqProductConstraint.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 a_2 b : List String),   ¬a_2 = b →     { structName := a, fieldNames := a_1, fieldTypes := a_2 } =         { structName := a, fiel |
| 1535 | `Polyrust.instDecidableEqProductConstraint.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 a_2 b b_1 : List String),   ¬a_1 = b →     { structName := a, fieldNames := a_1, fieldTypes := a_2 } =         { structName := a,  |
| 1536 | `Polyrust.instDecidableEqProductConstraint.decEq._proof_4` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 a_2 : List String) (b : String) (b_1 b_2 : List String),   ¬a = b →     { structName := a, fieldNames := a_1, fieldTypes := a_2 }  |
| 1537 | `Polyrust.instDecidableEqProductConstraint.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.ProductConstraint → Polyrust.ProductConstraint → Sort u_1) →   (x x_1 : Polyrust.ProductConstraint) →     ((a : String) →         ( |
| 1538 | `Polyrust.instDecidableEqSumConstraint` | 實例衍生 | 定義 | | DecidableEq Polyrust.SumConstraint |
| 1539 | `Polyrust.instDecidableEqSumConstraint.decEq` | 實例衍生 | 定義 | | (x x_1 : Polyrust.SumConstraint) → Decidable (x = x_1) |
| 1540 | `Polyrust.instDecidableEqSumConstraint.decEq._proof_1` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 : List String), { enumName := a, variantNames := a_1 } = { enumName := a, variantNames := a_1 } |
| 1541 | `Polyrust.instDecidableEqSumConstraint.decEq._proof_2` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 b : List String),   ¬a_1 = b → { enumName := a, variantNames := a_1 } = { enumName := a, variantNames := b } → False |
| 1542 | `Polyrust.instDecidableEqSumConstraint.decEq._proof_3` | 實例衍生 | 定理 | | ∀ (a : String) (a_1 : List String) (b : String) (b_1 : List String),   ¬a = b → { enumName := a, variantNames := a_1 } = { enumName := b, variantNames |
| 1543 | `Polyrust.instDecidableEqSumConstraint.decEq.match_1` | 實例衍生 | 定義 | | (motive : Polyrust.SumConstraint → Polyrust.SumConstraint → Sort u_1) →   (x x_1 : Polyrust.SumConstraint) →     ((a : String) →         (a_1 : List S |
| 1544 | `Polyrust.maxDegreeV2.eq_1` | 等式引理／助手 | 定理 | | Polyrust.maxDegreeV2 = 4 |
| 1545 | `Polyrust.productDegree.eq_1` | 等式引理／助手 | 定理 | | ∀ (pc : Polyrust.ProductConstraint), Polyrust.productDegree pc = pc.fieldNames.length |

## `Polyrust.ClauseDuality`（32 條）

職責：子句滿足 ⟺ 子句多項式歸零；域多項式 x²−x 刻畫 {0,1}ⁿ；CNF 對偶

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1546 | `Polyrust.Assignment` | 手寫 | 定義 | | Type |
| 1547 | `Polyrust.Lit.mk.inj` | 手寫 | 定理 | | ∀ {var : Nat} {pos : Bool} {var_1 : Nat} {pos_1 : Bool},   { var := var, pos := pos } = { var := var_1, pos := pos_1 } → var = var_1 ∧ pos = pos_1 |
| 1548 | `Polyrust.Lit.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (var : Nat) (pos : Bool), sizeOf { var := var, pos := pos } = 1 + sizeOf var + sizeOf pos |
| 1549 | `Polyrust.bit` | 手寫 | 定義 | | Bool → Int |
| 1550 | `Polyrust.bit_eq_one` | 手寫 | 定理 | | ∀ (b : Bool), b = true → Polyrust.bit b = 1 |
| 1551 | `Polyrust.bit_eq_zero` | 手寫 | 定理 | | ∀ (b : Bool), b = false → Polyrust.bit b = 0 |
| 1552 | `Polyrust.clausePoly` | 手寫 | 定義 | | Polyrust.Assignment → List Polyrust.Lit → Int |
| 1553 | `Polyrust.clauseSat` | 手寫 | 定義 | | Polyrust.Assignment → List Polyrust.Lit → Bool |
| 1554 | `Polyrust.cnfPolys` | 手寫 | 定義 | | Polyrust.Assignment → List (List Polyrust.Lit) → List Int |
| 1555 | `Polyrust.cnfSat` | 手寫 | 定義 | | Polyrust.Assignment → List (List Polyrust.Lit) → Bool |
| 1556 | `Polyrust.field_poly_bit` | 手寫 | 定理 | | ∀ (b : Bool), Polyrust.bit b * (Polyrust.bit b - 1) = 0 |
| 1557 | `Polyrust.litFactor` | 手寫 | 定義 | | Polyrust.Assignment → Polyrust.Lit → Int |
| 1558 | `Polyrust.litSat` | 手寫 | 定義 | | Polyrust.Assignment → Polyrust.Lit → Bool |
| 1559 | `Polyrust.Lit._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.Lit → Nat |
| 1560 | `Polyrust.Lit._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.Lit |
| 1561 | `Polyrust.Lit.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Lit → Sort u} →   (t : Polyrust.Lit) → ((var : Nat) → (pos : Bool) → motive { var := var, pos := pos }) → motive t |
| 1562 | `Polyrust.Lit.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.Lit → Nat |
| 1563 | `Polyrust.Lit.mk._flat_ctor` | 等式引理／助手 | 定義 | | Nat → Bool → Polyrust.Lit |
| 1564 | `Polyrust.Lit.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {var : Nat} →     {pos : Bool} →       {var' : Nat} →         {pos' : Bool} → { var := var, pos := pos } = { var := var', pos := pos' |
| 1565 | `Polyrust.Lit.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.Lit} → t = t' → Polyrust.Lit.noConfusionType P t t' |
| 1566 | `Polyrust.Lit.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.Lit → Polyrust.Lit → Sort u |
| 1567 | `Polyrust.Lit.pos` | 衍生 | 定義 | | Polyrust.Lit → Bool |
| 1568 | `Polyrust.Lit.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.Lit → Sort u} →   (t : Polyrust.Lit) → ((var : Nat) → (pos : Bool) → motive { var := var, pos := pos }) → motive t |
| 1569 | `Polyrust.Lit.var` | 衍生 | 定義 | | Polyrust.Lit → Nat |
| 1570 | `Polyrust.bit.eq_1` | 等式引理／助手 | 定理 | | ∀ (b : Bool), Polyrust.bit b = if b = true then 1 else 0 |
| 1571 | `Polyrust.clausePoly.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Assignment) (C : List Polyrust.Lit),   Polyrust.clausePoly σ C = List.foldr (fun a b => a * b) 1 (List.map (Polyrust.litFactor σ) C) |
| 1572 | `Polyrust.clauseSat.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Assignment) (C : List Polyrust.Lit), Polyrust.clauseSat σ C = C.any (Polyrust.litSat σ) |
| 1573 | `Polyrust.cnfPolys.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Assignment) (Φ : List (List Polyrust.Lit)), Polyrust.cnfPolys σ Φ = List.map (Polyrust.clausePoly σ) Φ |
| 1574 | `Polyrust.cnfSat.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Assignment) (Φ : List (List Polyrust.Lit)), Polyrust.cnfSat σ Φ = Φ.all (Polyrust.clauseSat σ) |
| 1575 | `Polyrust.instReprLit` | 實例衍生 | 定義 | | Repr Polyrust.Lit |
| 1576 | `Polyrust.instReprLit.repr` | 實例衍生 | 定義 | | Polyrust.Lit → Nat → Format |
| 1577 | `Polyrust.litFactor.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit), Polyrust.litFactor σ l = 1 - Polyrust.bit (Polyrust.litSat σ l) |

## `Polyrust.UniPoly`（26 條）

職責：求值環同態；線性餘式；互異根 vanishing ⇒ ∏(X−tᵢ) 整除；QAP 對偶

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1578 | `Polyrust.UniPoly` | 手寫 | 定義 | | Type |
| 1579 | `Polyrust.UniPoly.XsubC` | 手寫 | 定義 | | Int → Polyrust.UniPoly |
| 1580 | `Polyrust.UniPoly.add` | 手寫 | 定義 | | Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly |
| 1581 | `Polyrust.UniPoly.eval` | 手寫 | 定義 | | Polyrust.UniPoly → Int → Int |
| 1582 | `Polyrust.UniPoly.negP` | 手寫 | 定義 | | Polyrust.UniPoly → Polyrust.UniPoly |
| 1583 | `Polyrust.UniPoly.shift` | 手寫 | 定義 | | Polyrust.UniPoly → Polyrust.UniPoly |
| 1584 | `Polyrust.UniPoly.smulC` | 手寫 | 定義 | | Int → Polyrust.UniPoly → Polyrust.UniPoly |
| 1585 | `Polyrust.UniPoly.sub` | 手寫 | 定義 | | Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly |
| 1586 | `Polyrust.UniPoly.add._f` | 等式引理／助手 | 定義 | | (x : Polyrust.UniPoly) →   List.below (motive := fun x => Polyrust.UniPoly → Polyrust.UniPoly) x → Polyrust.UniPoly → Polyrust.UniPoly |
| 1587 | `Polyrust.UniPoly.add._sunfold` | 等式引理／助手 | 定義 | | Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly |
| 1588 | `Polyrust.UniPoly.add._unsafe_rec` | 等式引理／助手 | 定義 | | Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly |
| 1589 | `Polyrust.UniPoly.add.eq_1` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.UniPoly), Polyrust.UniPoly.add [] x = x |
| 1590 | `Polyrust.UniPoly.add.eq_2` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.UniPoly), (x = [] → False) → x.add [] = x |
| 1591 | `Polyrust.UniPoly.add.eq_3` | 等式引理／助手 | 定理 | | ∀ (a : Int) (as : List Int) (b : Int) (bs : List Int),   Polyrust.UniPoly.add (a :: as) (b :: bs) = (a + b) :: Polyrust.UniPoly.add as bs |
| 1592 | `Polyrust.UniPoly.add.eq_def` | 等式引理／助手 | 定理 | | ∀ (x x_1 : Polyrust.UniPoly),   x.add x_1 =     match x, x_1 with     \| [], q => q     \| p, [] => p     \| a :: as, b :: bs => (a + b) :: Polyrust.UniP |
| 1593 | `Polyrust.UniPoly.add.match_1` | 等式引理／助手 | 定義 | | (motive : Polyrust.UniPoly → Polyrust.UniPoly → Sort u_1) →   (x x_1 : Polyrust.UniPoly) →     ((q : Polyrust.UniPoly) → motive [] q) →       ((p : Po |
| 1594 | `Polyrust.UniPoly.eval._f` | 等式引理／助手 | 定義 | | (x : Polyrust.UniPoly) → List.below (motive := fun x => Int → Int) x → Int → Int |
| 1595 | `Polyrust.UniPoly.eval._sunfold` | 等式引理／助手 | 定義 | | Polyrust.UniPoly → Int → Int |
| 1596 | `Polyrust.UniPoly.eval._unsafe_rec` | 等式引理／助手 | 定義 | | Polyrust.UniPoly → Int → Int |
| 1597 | `Polyrust.UniPoly.eval.eq_1` | 等式引理／助手 | 定理 | | ∀ (x : Int), Polyrust.UniPoly.eval [] x = 0 |
| 1598 | `Polyrust.UniPoly.eval.eq_2` | 等式引理／助手 | 定理 | | ∀ (x c : Int) (cs : List Int), Polyrust.UniPoly.eval (c :: cs) x = c + x * Polyrust.UniPoly.eval cs x |
| 1599 | `Polyrust.UniPoly.eval.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.UniPoly) (x_1 : Int),   x.eval x_1 =     match x, x_1 with     \| [], x => 0     \| c :: cs, t => c + t * Polyrust.UniPoly.eval cs t |
| 1600 | `Polyrust.UniPoly.eval.match_1` | 等式引理／助手 | 定義 | | (motive : Polyrust.UniPoly → Int → Sort u_1) →   (x : Polyrust.UniPoly) →     (x_1 : Int) →       ((x : Int) → motive [] x) → ((c : Int) → (cs : List  |
| 1601 | `Polyrust.UniPoly.negP.eq_1` | 等式引理／助手 | 定理 | | ∀ (p : Polyrust.UniPoly), p.negP = List.map (fun c => -c) p |
| 1602 | `Polyrust.UniPoly.smulC.eq_1` | 等式引理／助手 | 定理 | | ∀ (c : Int) (p : Polyrust.UniPoly), Polyrust.UniPoly.smulC c p = List.map (fun x => c * x) p |
| 1603 | `Polyrust.UniPoly.sub.eq_1` | 等式引理／助手 | 定理 | | ∀ (p q : Polyrust.UniPoly), p.sub q = p.add q.negP |

## `Polyrust.LoopContract`（21 條）

職責：循環不變量、歸納契約、fuel 有界展開

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1604 | `Polyrust.LoopContract.empty` | 手寫 | 定義 | | Polyrust.LoopContract |
| 1605 | `Polyrust.LoopContract.fuelOrDefault` | 手寫 | 定義 | | Polyrust.LoopContract → Nat |
| 1606 | `Polyrust.LoopContract.mk.inj` | 手寫 | 定理 | | ∀ {fuel : Option Nat} {invariants requires ensures : List String} {fuel_1 : Option Nat}   {invariants_1 requires_1 ensures_1 : List String},   { fuel  |
| 1607 | `Polyrust.LoopContract.mk.sizeOf_spec` | 手寫 | 定理 | | ∀ (fuel : Option Nat) (invariants requires ensures : List String),   sizeOf { fuel := fuel, invariants := invariants, requires := requires, ensures := |
| 1608 | `Polyrust.fuel_default` | 手寫 | 定理 | | Polyrust.LoopContract.empty.fuelOrDefault = 3 |
| 1609 | `Polyrust.testFuelDefault` | 手寫 | 定義 | | Nat |
| 1610 | `Polyrust.LoopContract._sizeOf_1` | 歸納型衍生 | 定義 | | Polyrust.LoopContract → Nat |
| 1611 | `Polyrust.LoopContract._sizeOf_inst` | 歸納型衍生 | 定義 | | SizeOf Polyrust.LoopContract |
| 1612 | `Polyrust.LoopContract.casesOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.LoopContract → Sort u} →   (t : Polyrust.LoopContract) →     ((fuel : Option Nat) →         (invariants requires ensures : List Str |
| 1613 | `Polyrust.LoopContract.ctorIdx` | 歸納型衍生 | 定義 | | Polyrust.LoopContract → Nat |
| 1614 | `Polyrust.LoopContract.ensures` | 衍生 | 定義 | | Polyrust.LoopContract → List String |
| 1615 | `Polyrust.LoopContract.fuel` | 衍生 | 定義 | | Polyrust.LoopContract → Option Nat |
| 1616 | `Polyrust.LoopContract.fuelOrDefault.eq_1` | 等式引理／助手 | 定理 | | ∀ (c : Polyrust.LoopContract), c.fuelOrDefault = c.fuel.getD 3 |
| 1617 | `Polyrust.LoopContract.invariantPolyText.go.match_1` | 等式引理／助手 | 定義 | | (motive : List String → Nat → List String → Sort u_1) →   (x : List String) →     (x_1 : Nat) →       (x_2 : List String) →         ((x : Nat) → (acc  |
| 1618 | `Polyrust.LoopContract.invariants` | 衍生 | 定義 | | Polyrust.LoopContract → List String |
| 1619 | `Polyrust.LoopContract.mk._flat_ctor` | 等式引理／助手 | 定義 | | Option Nat → List String → List String → List String → Polyrust.LoopContract |
| 1620 | `Polyrust.LoopContract.mk.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} →   {fuel : Option Nat} →     {invariants requires ensures : List String} →       {fuel' : Option Nat} →         {invariants' requires' e |
| 1621 | `Polyrust.LoopContract.noConfusion` | 歸納型衍生 | 定義 | | {P : Sort u} → {t t' : Polyrust.LoopContract} → t = t' → Polyrust.LoopContract.noConfusionType P t t' |
| 1622 | `Polyrust.LoopContract.noConfusionType` | 衍生 | 定義 | | Sort u → Polyrust.LoopContract → Polyrust.LoopContract → Sort u |
| 1623 | `Polyrust.LoopContract.recOn` | 歸納型衍生 | 定義 | | {motive : Polyrust.LoopContract → Sort u} →   (t : Polyrust.LoopContract) →     ((fuel : Option Nat) →         (invariants requires ensures : List Str |
| 1624 | `Polyrust.LoopContract.requires` | 衍生 | 定義 | | Polyrust.LoopContract → List String |

## `Polyrust.Monomial`（19 條）

職責：單項式指數向量、整除、lcm/quot、支撐、首項；純組合層

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1625 | `Polyrust.MonoExp` | 手寫 | 定義 | | Type |
| 1626 | `Polyrust.coprimeM` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MonoExp → Prop |
| 1627 | `Polyrust.coprimeM_comm` | 手寫 | 定理 | | ∀ {a b : Polyrust.MonoExp}, Polyrust.coprimeM a b → Polyrust.coprimeM b a |
| 1628 | `Polyrust.dividesM` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MonoExp → Prop |
| 1629 | `Polyrust.dividesM_monoMul_right` | 手寫 | 定理 | | ∀ (a b : Polyrust.MonoExp), Polyrust.dividesM b (Polyrust.monoMul a b) |
| 1630 | `Polyrust.dividesM_monoOne` | 手寫 | 定理 | | ∀ (a : Polyrust.MonoExp), Polyrust.dividesM Polyrust.monoOne a |
| 1631 | `Polyrust.dividesM_refl` | 手寫 | 定理 | | ∀ (a : Polyrust.MonoExp), Polyrust.dividesM a a |
| 1632 | `Polyrust.dividesM_trans` | 手寫 | 定理 | | ∀ {a b c : Polyrust.MonoExp}, Polyrust.dividesM a b → Polyrust.dividesM b c → Polyrust.dividesM a c |
| 1633 | `Polyrust.gcdM` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp |
| 1634 | `Polyrust.lcmM` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp |
| 1635 | `Polyrust.monoMul` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp |
| 1636 | `Polyrust.monoOne` | 手寫 | 定義 | | Polyrust.MonoExp |
| 1637 | `Polyrust.monoOne_squarefree` | 手寫 | 定理 | | Polyrust.squarefreeM Polyrust.monoOne |
| 1638 | `Polyrust.quotM` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp |
| 1639 | `Polyrust.squarefreeM` | 手寫 | 定義 | | Polyrust.MonoExp → Prop |
| 1640 | `Polyrust.x1` | 手寫 | 定義 | | Nat → Polyrust.MonoExp |
| 1641 | `Polyrust.x2` | 手寫 | 定義 | | Nat → Polyrust.MonoExp |
| 1642 | `Polyrust.x1.eq_1` | 等式引理／助手 | 定理 | | ∀ (i j : Nat), Polyrust.x1 i j = if j = i then 1 else 0 |
| 1643 | `Polyrust.x2.eq_1` | 等式引理／助手 | 定理 | | ∀ (i j : Nat), Polyrust.x2 i j = if j = i then 2 else 0 |

## `Polyrust.Minor`（17 條）

職責：次要（支撐性）：位元算術、Lit 支撐、MonoExp 整除、supportLe、List 求和

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1644 | `Polyrust.minor_bit_add_bit_not` | 手寫 | 定理 | | ∀ (b : Bool), (Polyrust.bit b + Polyrust.bit !b) = 1 |
| 1645 | `Polyrust.minor_bit_and` | 手寫 | 定理 | | ∀ (a b : Bool), Polyrust.bit (a && b) = Polyrust.bit a * Polyrust.bit b |
| 1646 | `Polyrust.minor_bit_and_comm` | 手寫 | 定理 | | ∀ (a b : Bool), Polyrust.bit (a && b) = Polyrust.bit (b && a) |
| 1647 | `Polyrust.minor_bit_eq_zero_or_one` | 手寫 | 定理 | | ∀ (b : Bool), Polyrust.bit b = 0 ∨ Polyrust.bit b = 1 |
| 1648 | `Polyrust.minor_coprimeM_comm` | 手寫 | 定理 | | ∀ {a b : Polyrust.MonoExp}, Polyrust.coprimeM a b → Polyrust.coprimeM b a |
| 1649 | `Polyrust.minor_dividesM_monoOne` | 手寫 | 定理 | | ∀ (a : Polyrust.MonoExp), Polyrust.dividesM Polyrust.monoOne a |
| 1650 | `Polyrust.minor_dividesM_refl` | 手寫 | 定理 | | ∀ (a : Polyrust.MonoExp), Polyrust.dividesM a a |
| 1651 | `Polyrust.minor_dividesM_trans` | 手寫 | 定理 | | ∀ {a b c : Polyrust.MonoExp}, Polyrust.dividesM a b → Polyrust.dividesM b c → Polyrust.dividesM a c |
| 1652 | `Polyrust.minor_field_poly_bit` | 手寫 | 定理 | | ∀ (b : Bool), Polyrust.bit b * (Polyrust.bit b - 1) = 0 |
| 1653 | `Polyrust.minor_litFactor_negLit` | 手寫 | 定理 | | ∀ (β : Polyrust.Assignment) (n : Nat), Polyrust.litFactor β { var := n, pos := false } = Polyrust.bit (β n) |
| 1654 | `Polyrust.minor_lit_neg_neg` | 手寫 | 定理 | | ∀ (l : Polyrust.Lit), l.neg.neg = l |
| 1655 | `Polyrust.minor_lit_neg_var` | 手寫 | 定理 | | ∀ (l : Polyrust.Lit), l.neg.var = l.var |
| 1656 | `Polyrust.minor_monoOne_squarefree` | 手寫 | 定理 | | Polyrust.squarefreeM Polyrust.monoOne |
| 1657 | `Polyrust.minor_supportLe_empty` | 手寫 | 定理 | | Polyrust.supportLe (fun x => false) 0 |
| 1658 | `Polyrust.minor_supportLe_mono` | 手寫 | 定理 | | ∀ {f : Nat → Bool} {n m : Nat}, Polyrust.supportLe f n → n ≤ m → Polyrust.supportLe f m |
| 1659 | `Polyrust.minor_supportLe_nth_false` | 手寫 | 定理 | | ∀ {f : Nat → Bool} {n : Nat}, Polyrust.supportLe f n → f n = false |
| 1660 | `Polyrust.monoOne.eq_1` | 等式引理／助手 | 定理 | | ∀ (x : Nat), Polyrust.monoOne x = 0 |

## `Polyrust.Squarefree`（17 條）

職責：標準單項式 ⇒ 平方自由；平方自由 ↔ 位串雙射；恰 2ⁿ 個 ⇒ Buchberger 終止

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1661 | `Polyrust.allBits` | 手寫 | 定義 | | Nat → List (Nat → Bool) |
| 1662 | `Polyrust.ofBits` | 手寫 | 定義 | | (Nat → Bool) → Polyrust.MonoExp |
| 1663 | `Polyrust.setN` | 手寫 | 定義 | | (Nat → Bool) → Nat → Bool → Nat → Bool |
| 1664 | `Polyrust.supportLe` | 手寫 | 定義 | | (Nat → Bool) → Nat → Prop |
| 1665 | `Polyrust.supportLeM` | 手寫 | 定義 | | Polyrust.MonoExp → Nat → Prop |
| 1666 | `Polyrust.supportLe_nth_false` | 手寫 | 定理 | | ∀ {f : Nat → Bool} {n : Nat}, Polyrust.supportLe f n → f n = false |
| 1667 | `Polyrust.toBits` | 手寫 | 定義 | | Polyrust.MonoExp → Nat → Bool |
| 1668 | `Polyrust.allBits._f` | 等式引理／助手 | 定義 | | (x : Nat) → Nat.below x → List (Nat → Bool) |
| 1669 | `Polyrust.allBits._sunfold` | 等式引理／助手 | 定義 | | Nat → List (Nat → Bool) |
| 1670 | `Polyrust.allBits._unsafe_rec` | 等式引理／助手 | 定義 | | Nat → List (Nat → Bool) |
| 1671 | `Polyrust.allBits.eq_1` | 等式引理／助手 | 定理 | | Polyrust.allBits 0 = [fun x => false] |
| 1672 | `Polyrust.allBits.eq_2` | 等式引理／助手 | 定理 | | ∀ (n : Nat),   Polyrust.allBits n.succ =     List.flatMap (fun f => [Polyrust.setN f n false, Polyrust.setN f n true]) (Polyrust.allBits n) |
| 1673 | `Polyrust.allBits.eq_def` | 等式引理／助手 | 定理 | | ∀ (x : Nat),   Polyrust.allBits x =     match x with     \| 0 => [fun x => false]     \| n.succ => List.flatMap (fun f => [Polyrust.setN f n false, Poly |
| 1674 | `Polyrust.allBits.match_1` | 等式引理／助手 | 定義 | | (motive : Nat → Sort u_1) → (x : Nat) → (Unit → motive 0) → ((n : Nat) → motive n.succ) → motive x |
| 1675 | `Polyrust.ofBits.eq_1` | 等式引理／助手 | 定理 | | ∀ (f : Nat → Bool) (j : Nat), Polyrust.ofBits f j = if f j = true then 1 else 0 |
| 1676 | `Polyrust.setN.eq_1` | 等式引理／助手 | 定理 | | ∀ (f : Nat → Bool) (i : Nat) (b : Bool) (j : Nat), Polyrust.setN f i b j = if j = i then b else f j |
| 1677 | `Polyrust.toBits.eq_1` | 等式引理／助手 | 定理 | | ∀ (m : Polyrust.MonoExp) (j : Nat), Polyrust.toBits m j = (m j == 1) |

## `Polyrust.T6Certificate`（17 條）

職責：求值同態、無公共零點證書、布爾 Lagrange 插值

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1678 | `Polyrust.F` | 手寫 | 定義 | | Type |
| 1679 | `Polyrust.NoCommonZero` | 手寫 | 定義 | | List (Nat → Bool) → List Polyrust.F → Prop |
| 1680 | `Polyrust.PolyFn.one` | 手寫 | 定理 | | ∀ {n : Nat}, Polyrust.PolyFn n fun x => 1 |
| 1681 | `Polyrust.PolyFn.smulConst` | 手寫 | 定理 | | ∀ {n : Nat} {f : Polyrust.F} (c : Int), Polyrust.PolyFn n f → Polyrust.PolyFn n fun x => c * f x |
| 1682 | `Polyrust.PolyFn.zero` | 手寫 | 定理 | | ∀ {n : Nat}, Polyrust.PolyFn n fun x => 0 |
| 1683 | `Polyrust.coord` | 手寫 | 定義 | | Nat → Polyrust.F |
| 1684 | `Polyrust.delta` | 手寫 | 定義 | | Nat → (Nat → Bool) → Polyrust.F |
| 1685 | `Polyrust.interp` | 手寫 | 定義 | | Nat → List (Nat → Bool) → Polyrust.F → Polyrust.F |
| 1686 | `Polyrust.PolyFn.below.casesOn` | 歸納型衍生 | 定義 | | ∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop}   {motive_1 : {a : Polyrust.F} → (t : Polyrust.PolyFn n a) → Polyrust.PolyFn.belo |
| 1687 | `Polyrust.PolyFn.brecOn` | 歸納型衍生 | 定理 | | ∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop} {a : Polyrust.F} (t : Polyrust.PolyFn n a),   (∀ (a : Polyrust.F) (t : Polyrust.P |
| 1688 | `Polyrust.PolyFn.casesOn` | 歸納型衍生 | 定義 | | ∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop} {a : Polyrust.F} (t : Polyrust.PolyFn n a),   (∀ (c : Int), motive (fun x => c) ⋯ |
| 1689 | `Polyrust.PolyFn.recOn` | 歸納型衍生 | 定義 | | ∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop} {a : Polyrust.F} (t : Polyrust.PolyFn n a),   (∀ (c : Int), motive (fun x => c) ⋯ |
| 1690 | `Polyrust.delta._f` | 等式引理／助手 | 定義 | | (x : Nat) → Nat.below (motive := fun x => (Nat → Bool) → Polyrust.F) x → (Nat → Bool) → Polyrust.F |
| 1691 | `Polyrust.delta._sunfold` | 等式引理／助手 | 定義 | | Nat → (Nat → Bool) → Polyrust.F |
| 1692 | `Polyrust.delta._unsafe_rec` | 等式引理／助手 | 定義 | | Nat → (Nat → Bool) → Polyrust.F |
| 1693 | `Polyrust.delta.match_1` | 等式引理／助手 | 定義 | | (motive : Nat → (Nat → Bool) → Sort u_1) →   (x : Nat) →     (x_1 : Nat → Bool) →       ((x : Nat → Bool) → motive 0 x) → ((k : Nat) → (σ : Nat → Bool |
| 1694 | `Polyrust.interp.eq_1` | 等式引理／助手 | 定理 | | ∀ (n : Nat) (pts : List (Nat → Bool)) (g : Polyrust.F) (x : Nat → Bool),   Polyrust.interp n pts g x = (List.map (fun σ => g σ * Polyrust.delta n σ x) |

## `Polyrust.ClauseAlgebra`（13 條）

職責：消解恆等式；學習子句保留模型集/零集；UNSAT ⟺ 無零點多項式

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1695 | `Polyrust.Entails` | 手寫 | 定義 | | List (List Polyrust.Lit) → List Polyrust.Lit → Prop |
| 1696 | `Polyrust.Lit.neg` | 手寫 | 定義 | | Polyrust.Lit → Polyrust.Lit |
| 1697 | `Polyrust.Lit.neg_neg` | 手寫 | 定理 | | ∀ (l : Polyrust.Lit), l.neg.neg = l |
| 1698 | `Polyrust.Lit.neg_var` | 手寫 | 定理 | | ∀ (l : Polyrust.Lit), l.neg.var = l.var |
| 1699 | `Polyrust.Mod` | 手寫 | 定義 | | Polyrust.Assignment → List (List Polyrust.Lit) → Prop |
| 1700 | `Polyrust.PolyZero` | 手寫 | 定義 | | Polyrust.Assignment → List (List Polyrust.Lit) → Prop |
| 1701 | `Polyrust.bit_add_bit_not` | 手寫 | 定理 | | ∀ (b : Bool), (Polyrust.bit b + Polyrust.bit !b) = 1 |
| 1702 | `Polyrust.clausePoly_cons` | 手寫 | 定理 | | ∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit) (C : List Polyrust.Lit),   Polyrust.clausePoly σ (l :: C) = Polyrust.litFactor σ l * Polyrust.clausePol |
| 1703 | `Polyrust.clausePoly_nil` | 手寫 | 定理 | | ∀ {σ : Polyrust.Assignment}, Polyrust.clausePoly σ [] = 1 |
| 1704 | `Polyrust.clausePoly_nil_ne_zero` | 手寫 | 定理 | | ∀ (σ : Polyrust.Assignment), Polyrust.clausePoly σ [] ≠ 0 |
| 1705 | `Polyrust.litFactor_eq_one_sub` | 手寫 | 定理 | | ∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit), Polyrust.litFactor σ l = 1 - Polyrust.bit (Polyrust.litSat σ l) |
| 1706 | `Polyrust.Lit.neg.eq_1` | 等式引理／助手 | 定理 | | ∀ (l : Polyrust.Lit), l.neg = { var := l.var, pos := !l.pos } |
| 1707 | `Polyrust.litSat.eq_1` | 等式引理／助手 | 定理 | | ∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit), Polyrust.litSat σ l = if l.pos = true then σ l.var else !σ l.var |

## `Polyrust.IronLaw`（13 條）

職責：新增（鐵律核心）：one-hot 排他、field 多項式、借用衝突互斥、lifetime 無環

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1708 | `Polyrust.iron_base_ne_ext` | 手寫 | 定理 | | ∀ (b : Polyrust.BaseTy7) (e : Polyrust.ExtTag), Polyrust.Ty7Plus10.base b ≠ Polyrust.Ty7Plus10.ext e |
| 1709 | `Polyrust.iron_conflictsWith_comm` | 手寫 | 定理 | | ∀ {b₁ b₂ : Polyrust.Borrow}, Polyrust.conflictsWith b₁ b₂ ↔ Polyrust.conflictsWith b₂ b₁ |
| 1710 | `Polyrust.iron_field_poly_bit` | 手寫 | 定理 | | ∀ (b : Bool), Polyrust.bit b * (Polyrust.bit b - 1) = 0 |
| 1711 | `Polyrust.iron_field_poly_general` | 手寫 | 定理 | | ∀ {Ty : Type} [DecidableEq Ty] (L : Polyrust.Lang Ty) (σ : Ty → Bool) (t : Ty),   Polyrust.bit (σ t) * (Polyrust.bit (σ t) - 1) = 0 |
| 1712 | `Polyrust.iron_lifetime_empty_no_selfLoop` | 手寫 | 定理 | | Polyrust.LifetimeGraph.empty.hasSelfLoop = false |
| 1713 | `Polyrust.iron_lifetime_empty_no_twoCycle` | 手寫 | 定理 | | Polyrust.LifetimeGraph.empty.hasTwoCycle = false |
| 1714 | `Polyrust.iron_noIO_ok_when_no_hasIO` | 手寫 | 定理 | | Polyrust.EffectContext.empty.checkNoIO.isOk = true |
| 1715 | `Polyrust.iron_overlaps_self_iff` | 手寫 | 定理 | | ∀ (b : Polyrust.Borrow), Polyrust.overlaps b b ↔ b.start < b.stop |
| 1716 | `Polyrust.iron_overlaps_self_of_nonempty` | 手寫 | 定理 | | ∀ {b : Polyrust.Borrow}, b.start < b.stop → Polyrust.overlaps b b |
| 1717 | `Polyrust.iron_unsafe_gate_empty_ok` | 手寫 | 定理 | | Polyrust.EffectContext.empty.checkUnsafeGate.isOk = true |
| 1718 | `Polyrust.iron_unsafe_gate_fails_when_unsafe_not_allowed` | 手寫 | 定理 | | have ctx :=   have __src := Polyrust.EffectContext.empty;   { inUnsafe := __src.inUnsafe, unsafeAllowed := false, hasIO := __src.hasIO, pure := __src. |
| 1719 | `Polyrust.iron_unsafe_gate_ok_when_allowed` | 手寫 | 定理 | | have ctx :=   have __src := Polyrust.EffectContext.empty;   { inUnsafe := __src.inUnsafe, unsafeAllowed := true, hasIO := __src.hasIO, pure := __src.p |
| 1720 | `Polyrust.iron_unsafe_gate_ok_when_inUnsafe` | 手寫 | 定理 | | have ctx :=   have __src := Polyrust.EffectContext.empty;   { inUnsafe := true, unsafeAllowed := __src.unsafeAllowed, hasIO := __src.hasIO, pure := __ |

## `Polyrust.MicroInstance`（13 條）

職責：加法規則、上下文矛盾、宏選臂三個微型系統的全枚舉

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1721 | `Polyrust.T1_micro` | 手寫 | 定理 | | Polyrust.addPoly1 true true true = 0 ∧ Polyrust.addPoly2 true true true = 0 |
| 1722 | `Polyrust.T6_micro_sat` | 手寫 | 定理 | | ∃ tx ty tr, Polyrust.addPoly1 tx ty tr = 0 ∧ Polyrust.addPoly2 tx ty tr = 0 |
| 1723 | `Polyrust.T7_arm_sat` | 手寫 | 定理 | | ∀ (tx : Bool), ∃ a, Polyrust.armPoly a tx true = 0 ∧ Polyrust.arm2PolyA a tx true = 0 ∧ Polyrust.arm2PolyB a tx true = 0 |
| 1724 | `Polyrust.addPoly1` | 手寫 | 定義 | | Bool → Bool → Bool → Int |
| 1725 | `Polyrust.addPoly2` | 手寫 | 定義 | | Bool → Bool → Bool → Int |
| 1726 | `Polyrust.arm2PolyA` | 手寫 | 定義 | | Bool → Bool → Bool → Int |
| 1727 | `Polyrust.arm2PolyB` | 手寫 | 定義 | | Bool → Bool → Bool → Int |
| 1728 | `Polyrust.armPoly` | 手寫 | 定義 | | Bool → Bool → Bool → Int |
| 1729 | `Polyrust.wellTypedAdd` | 手寫 | 定義 | | Bool → Bool → Bool → Prop |
| 1730 | `Polyrust.addPoly1.eq_1` | 等式引理／助手 | 定理 | | ∀ (tx ty tr : Bool), Polyrust.addPoly1 tx ty tr = Polyrust.bit tx + Polyrust.bit ty - Polyrust.bit tr - 1 |
| 1731 | `Polyrust.addPoly2.eq_1` | 等式引理／助手 | 定理 | | ∀ (tx ty _tr : Bool), Polyrust.addPoly2 tx ty _tr = Polyrust.bit tx - Polyrust.bit ty |
| 1732 | `Polyrust.armPoly.eq_1` | 等式引理／助手 | 定理 | | ∀ (a tx tr : Bool), Polyrust.armPoly a tx tr = Polyrust.bit a * (Polyrust.bit tx - Polyrust.bit tr) |
| 1733 | `Polyrust.wellTypedAdd.eq_1` | 等式引理／助手 | 定理 | | ∀ (tx ty tr : Bool), Polyrust.wellTypedAdd tx ty tr = (tx = true ∧ ty = true ∧ tr = true) |

## `Polyrust.SPoly`（8 條）

職責：S-多項式：單項式乘法、S-多項式 ∈ 生成理想、Buchberger 判準

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1734 | `Polyrust.IsLead` | 手寫 | 定義 | | Polyrust.MonoExp → Polyrust.MPoly → Prop |
| 1735 | `Polyrust.MPoly` | 手寫 | 定義 | | Type |
| 1736 | `Polyrust.addP` | 手寫 | 定義 | | Polyrust.MPoly → Polyrust.MPoly → Polyrust.MPoly |
| 1737 | `Polyrust.negP` | 手寫 | 定義 | | Polyrust.MPoly → Polyrust.MPoly |
| 1738 | `Polyrust.subP` | 手寫 | 定義 | | Polyrust.MPoly → Polyrust.MPoly → Polyrust.MPoly |
| 1739 | `Polyrust.zeroP` | 手寫 | 定義 | | Polyrust.MPoly |
| 1740 | `Polyrust.subP.eq_1` | 等式引理／助手 | 定理 | | ∀ (p q : Polyrust.MPoly) (m : Polyrust.MonoExp), Polyrust.subP p q m = p m - q m |
| 1741 | `Polyrust.zeroP.eq_1` | 等式引理／助手 | 定理 | | ∀ (x : Polyrust.MonoExp), Polyrust.zeroP x = 0 |

## `Polyrust.WatchMove`（7 條）

職責：監視文字交換移動保持子句語義；衝突偵測健全性

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1742 | `Polyrust.watch_overwrite_unsound` | 手寫 | 定理 | | ∃ σ,   Polyrust.clauseSat σ [{ var := 0, pos := true }, { var := 1, pos := true }, { var := 2, pos := true }] = true ∧     Polyrust.clauseSat σ [{ var |
| 1743 | `Polyrust.WatchMove.casesOn` | 歸納型衍生 | 定義 | | ∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMove a a_1 → Prop} {a a_1 : List Polyrust.Lit}   (t : Polyrust.WatchMove a a_1),   (∀ (f B l : |
| 1744 | `Polyrust.WatchMove.recOn` | 歸納型衍生 | 定義 | | ∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMove a a_1 → Prop} {a a_1 : List Polyrust.Lit}   (t : Polyrust.WatchMove a a_1),   (∀ (f B l : |
| 1745 | `Polyrust.WatchMoves.below.casesOn` | 歸納型衍生 | 定義 | | ∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop}   {motive_1 : {a a_1 : List Polyrust.Lit} → (t : Polyrust.WatchMoves a a_1 |
| 1746 | `Polyrust.WatchMoves.brecOn` | 歸納型衍生 | 定理 | | ∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop} {a a_1 : List Polyrust.Lit}   (t : Polyrust.WatchMoves a a_1),   (∀ (a a_2 |
| 1747 | `Polyrust.WatchMoves.casesOn` | 歸納型衍生 | 定義 | | ∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop} {a a_1 : List Polyrust.Lit}   (t : Polyrust.WatchMoves a a_1),   (∀ (C : L |
| 1748 | `Polyrust.WatchMoves.recOn` | 歸納型衍生 | 定義 | | ∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop} {a a_1 : List Polyrust.Lit}   (t : Polyrust.WatchMoves a a_1),   (∀ (C : L |

## `Polyrust.Canonical`（7 條）

職責：既約/全簡化標準形；若簡化 Gröbner 基存在則唯一

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1749 | `Polyrust.FullyReduced` | 手寫 | 定義 | | (Polyrust.MonoExp → Prop) → Polyrust.MPoly → Prop |
| 1750 | `Polyrust.HasMaxMono` | 手寫 | 定義 | | Polyrust.MPoly → Prop |
| 1751 | `Polyrust.Interreduced` | 手寫 | 定義 | | (Polyrust.MonoExp → Prop) → Polyrust.MonoExp → Polyrust.MPoly → Prop |
| 1752 | `Polyrust.leadIdeal` | 手寫 | 定義 | | (Polyrust.MPoly → Prop) → Polyrust.MonoExp → Prop |
| 1753 | `Polyrust.mon` | 手寫 | 定義 | | Nat → Polyrust.MonoExp |
| 1754 | `Polyrust.subP_apply` | 手寫 | 定理 | | ∀ (p q : Polyrust.MPoly) (m : Polyrust.MonoExp), Polyrust.subP p q m = p m - q m |
| 1755 | `Polyrust.mon.eq_1` | 等式引理／助手 | 定理 | | ∀ (n j : Nat), Polyrust.mon n j = if j = 0 then n else 0 |

## `Polyrust.Derived`（5 條）

職責：衍生（由核心推出）：pair/sum 可定型推論、pairBitSum 乘積、borrow 1∈理想

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1756 | `Polyrust.derived_bit_and` | 手寫 | 定理 | | ∀ (a b : Bool), Polyrust.bit (a && b) = Polyrust.bit a * Polyrust.bit b |
| 1757 | `Polyrust.derived_p5_conflict` | 手寫 | 定理 | | Polyrust.conflictsWith { node := 1, varDef := 7, start := 1, stop := 4 }   { node := 2, varDef := 7, start := 2, stop := 4 } |
| 1758 | `Polyrust.derived_p6_no_conflict` | 手寫 | 定理 | | ¬Polyrust.conflictsWith { node := 1, varDef := 7, start := 1, stop := 2 }     { node := 2, varDef := 7, start := 3, stop := 4 } |
| 1759 | `Polyrust.derived_tycheck_inl` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] {L : Polyrust.Lang Ty} {a : Polyrust.SumExpr Polyrust.Expr}   {τ₁ τ₂ : Polyrust.SumTy Ty}, Polyrust.tycheckSum L |
| 1760 | `Polyrust.derived_tycheck_inr` | 手寫 | 定理 | | ∀ {Ty : Type} [inst : DecidableEq Ty] {L : Polyrust.Lang Ty} {b : Polyrust.SumExpr Polyrust.Expr}   {τ₁ τ₂ : Polyrust.SumTy Ty}, Polyrust.tycheckSum L |

## `Polyrust.Completion`（3 條）

職責：補全（雙向完備）：clauseSat↔polyZero、borrow_sat↔clean、typable↔root 雙向

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1761 | `Polyrust.field_poly_complete` | 手寫 | 定理 | | ∀ (b : Bool), Polyrust.bit b * (Polyrust.bit b - 1) = 0 |
| 1762 | `Polyrust.field_poly_iff_bool` | 手寫 | 定理 | | ∀ (b : Bool), Polyrust.bit b * (Polyrust.bit b - 1) = 0 ↔ Polyrust.bit b = 0 ∨ Polyrust.bit b = 1 |
| 1763 | `Polyrust.oneHot_iff_exists_unique.match_1_2` | 等式引理／助手 | 定義 | | ∀ {Ty : Type} (L : Polyrust.Lang Ty) (σ : Ty → Bool)   (motive : (∃ t, t ∈ L.enumAll ∧ σ t = true ∧ ∀ (t' : Ty), t' ∈ L.enumAll → σ t' = true → t' = t |

## `Polyrust.BoolNullstellensatz`（2 條）

職責：布爾 Nullstellensatz：每點有模 p 可逆系統元素 ⟹ 多項式函數乘子組合成 1

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1764 | `Polyrust.ModP` | 手寫 | 定義 | | Int → Int → Int → Prop |
| 1765 | `Polyrust.modp_congr` | 手寫 | 定理 | | ∀ {p a b c d : Int}, a = c → b = d → Polyrust.ModP p a b → Polyrust.ModP p c d |

## `Polyrust.Tactics`（2 條）

職責：int_ring：無 Mathlib 的整數多項式歸一化宏

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1766 | `Polyrust._aux_Polyrust_Tactics___macroRules_Polyrust_tacticInt_ring_1` | 手寫 | 定義 | | Macro |
| 1767 | `Polyrust.tacticInt_ring` | 手寫 | 定義 | | ParserDescr |

## `Polyrust.Embedding`（2 條）

職責：𝔽_p（p=2⁶¹−1）嵌入保真

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1768 | `Polyrust.P61` | 手寫 | 定義 | | Nat |
| 1769 | `Polyrust.P61_eq` | 手寫 | 定理 | | Polyrust.P61 + 1 = 2 ^ 61 |

