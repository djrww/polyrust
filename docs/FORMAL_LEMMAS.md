# Lean 純構造宣告逐條清冊（970 條）

> 判據與 `AuditAll.lean` 同源：`Lean.collectAxioms`（即 `#print axioms` 背後同一函式）對 `Polyrust.*` 每一條定理／定義計算公理依賴，**依賴集合為空**者列入本表。零 `sorry`、零自訂公理；依賴 `propext`／`Classical.choice`／`Quot.sound` 者不算純構造，未列入。

**構成**：970 條 = 401 條定理 ＋ 569 條定義；其中手寫 197 條、機器衍生 773 條（歸納型自動產生的一致性／遞迴／判定引理）。

## 模組總覽

| 模組 | 條數 | 手寫 | 機器衍生 | 職責一句話 |
|---|---:|---:|---:|---|
| `Polyrust.T9EndToEnd` | 213 | 25 | 188 | 端到端微實例證明族 |
| `Polyrust.MacroExpansion` | 109 | 6 | 103 | 巨集展開的語義 |
| `Polyrust.OpAbstraction` | 107 | 16 | 91 | 運算子抽象層 |
| `Polyrust.SumReduction` | 106 | 6 | 100 | 求和項歸約 |
| `Polyrust.ProductReduction` | 97 | 5 | 92 | 乘積項歸約 |
| `Polyrust.BorrowOwnership` | 90 | 34 | 56 | 借用／所有權類型檢查的健全性（嵌入式語言層） |
| `Polyrust.T9Generalized` | 83 | 16 | 67 | 廣義端到端證明族 |
| `Polyrust.ClauseDuality` | 34 | 11 | 23 | 子句對偶與互補 |
| `Polyrust.UniPoly` | 26 | 8 | 18 | 單變量多項式運算 |
| `Polyrust.Monomial` | 19 | 17 | 2 | 單項式次序與乘法 |
| `Polyrust.Squarefree` | 17 | 7 | 10 | 平方自由化與 allBits 窮舉 |
| `Polyrust.T6Certificate` | 17 | 8 | 9 | 插值憑證層 |
| `Polyrust.MicroInstance` | 13 | 9 | 4 | 微實例語言核心：表達式／詞元／類型系統 |
| `Polyrust.ClauseAlgebra` | 11 | 11 | 0 | 子句代數：滿足性保持的代數變換 |
| `Polyrust.SPoly` | 8 | 6 | 2 | S-多項式與 Gröbner 步 |
| `Polyrust.Canonical` | 7 | 6 | 1 | 正規形與規範化 |
| `Polyrust.WatchMove` | 7 | 1 | 6 | 監視文字不變量：單步／迭代語義守恆 |
| `Polyrust.BoolNullstellensatz` | 2 | 2 | 0 | 布爾立方 Nullstellensatz：無公共零點 ⟹ 顯式逆元乘子（純計算部分） |
| `Polyrust.Embedding` | 2 | 2 | 0 | 表達式嵌入的語義保持 |
| `Polyrust.Tactics` | 2 | 1 | 1 | 戰術助手 |

## `Polyrust.T9EndToEnd`（213 條）

職責：端到端微實例證明族

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 1 | `Polyrust.IsMonoAt` | 手寫 | 定義 | 節點層級的單型性。 | `Polyrust.Sigma → Polyrust.Expr → Prop` |
| 2 | `Polyrust.IsRoot` | 手寫 | 定義 | σ 是方程組的 0/1 根。 | `Polyrust.Expr → Polyrust.Sigma → Prop` |
| 3 | `Polyrust.Sigma` | 手寫 | 定義 | 指名道姓的目標，匿名 lambda 會讓後續 `rw`/`rcases` 無從下手。 型別位元賦值：每個節點、每個型別一個 0/1 位元。 | `Type` |
| 4 | `Polyrust.bit_and` | 手寫 | 定理 | ! ## 七、位元算術引理 | `∀ (a b : Bool), Polyrust.bit (a && b) = Polyrust.bit a * Polyrust.bit b` |
| 5 | `Polyrust.cAddB` | 手寫 | 定義 | `add a b`：`t_{e,bool} = 0`（加法不產生布爾值）。 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 6 | `Polyrust.cAddH` | 手寫 | 定義 | 可計算定義 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 7 | `Polyrust.cAddI` | 手寫 | 定義 | `add a b`：`t_{e,i32} = t_{a,i32}·t_{b,i32}`。 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 8 | `Polyrust.cEqbB` | 手寫 | 定義 | `eqb a b`：`t_{e,bool} = t_{a,i32}·t_{b,i32}`。 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 9 | `Polyrust.cEqbH` | 手寫 | 定義 | 可計算定義 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 10 | `Polyrust.cEqbI` | 手寫 | 定義 | `eqb a b`：`t_{e,i32} = 0`。 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 11 | `Polyrust.cIteB` | 手寫 | 定義 | `ite c t f`：`t_{e,bool} = t_{c,bool}·t_{t,bool}·t_{f,bool}`。 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 12 | `Polyrust.cIteH` | 手寫 | 定義 | 可計算定義 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 13 | `Polyrust.cIteI` | 手寫 | 定義 | `ite c t f`：`t_{e,i32} = t_{c,bool}·t_{t,i32}·t_{f,i32}`。 | `Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Polyrust.Sigma → Int` |
| 14 | `Polyrust.cNumB` | 手寫 | 定義 | `num n`：`t = 0`（型別 bool 被排除）。 | `Int → Polyrust.Sigma → Int` |
| 15 | `Polyrust.cNumH` | 手寫 | 定義 | 各節點的 one-hot 約束（具名）。 | `Int → Polyrust.Sigma → Int` |
| 16 | `Polyrust.cNumI` | 手寫 | 定義 | `num n`：`t = 1`（型別 i32）。 | `Int → Polyrust.Sigma → Int` |
| 17 | `Polyrust.gen` | 手寫 | 定義 | 代碼生成：前序線性化。 | `Polyrust.Expr → List Polyrust.Tok` |
| 18 | `Polyrust.genC` | 手寫 | 定義 | 約束生成：規則方程 + one-hot。 | `Polyrust.Expr → List (Polyrust.Sigma → Int)` |
| 19 | `Polyrust.genC_num_oneHot` | 手寫 | 定理 | ! ## 三、約束表成員關係（顯式構造，不依賴 `simp` 的嵌套順序） | `∀ (n : Int), Polyrust.cNumH n ∈ Polyrust.genC (Polyrust.Expr.num n)` |
| 20 | `Polyrust.oneHot` | 手寫 | 定義 | one-hot 約束：`t_{e,i32} + t_{e,bool} − 1 = 0`。 | `Polyrust.Sigma → Polyrust.Expr → Int` |
| 21 | `Polyrust.oneHotC` | 手寫 | 定義 | one-hot 約束（以節點為參數的形式，便於指名其為約束表成員）。 | `Polyrust.Expr → Polyrust.Sigma → Int` |
| 22 | `Polyrust.parse` | 手寫 | 定義 | 公開解析器：燃料 = 輸入長度 + 1。 | `List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok)` |
| 23 | `Polyrust.parseFuel` | 手寫 | 定義 | 終止檢查，而「跳過一個子樹」本來就不是結構遞歸。 | `Nat → List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok)` |
| 24 | `Polyrust.sizeT` | 手寫 | 定義 | 生成碼長度（結構遞歸，供「燃料夠多」引理使用）。 | `Polyrust.Expr → Nat` |
| 25 | `Polyrust.tb` | 手寫 | 定義 | 節點 e 在型別 τ 上的位元，作為 ℤ 值。 | `Polyrust.Sigma → Polyrust.Expr → Polyrust.Ty → Int` |
| 26 | `Polyrust.Expr.brecOn.go` | 等式引理／助手 | 定義 | `Expr` 定義編譯時產生的遞迴／匹配助手 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → ((t : Polyrust.Expr) → Polyrust.Expr.…` |
| 27 | `Polyrust.cAddB.eq_1` | 等式引理／助手 | 定理 | 定義 `cAddB` 的等式引理（定義的展開方程） | `∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma), Polyrust.cAddB a b σ = Polyrust.tb σ (a.add b) Poly…` |
| 28 | `Polyrust.cAddH.eq_1` | 等式引理／助手 | 定理 | 定義 `cAddH` 的等式引理（定義的展開方程） | `∀ (a b : Polyrust.Expr), Polyrust.cAddH a b = Polyrust.oneHotC (a.add b)` |
| 29 | `Polyrust.cAddI.eq_1` | 等式引理／助手 | 定理 | 定義 `cAddI` 的等式引理（定義的展開方程） | `∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cAddI a b σ =     Polyrust.tb σ (a.add b…` |
| 30 | `Polyrust.cEqbB.eq_1` | 等式引理／助手 | 定理 | 定義 `cEqbB` 的等式引理（定義的展開方程） | `∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cEqbB a b σ =     Polyrust.tb σ (a.eqb b…` |
| 31 | `Polyrust.cEqbH.eq_1` | 等式引理／助手 | 定理 | 定義 `cEqbH` 的等式引理（定義的展開方程） | `∀ (a b : Polyrust.Expr), Polyrust.cEqbH a b = Polyrust.oneHotC (a.eqb b)` |
| 32 | `Polyrust.cEqbI.eq_1` | 等式引理／助手 | 定理 | 定義 `cEqbI` 的等式引理（定義的展開方程） | `∀ (a b : Polyrust.Expr) (σ : Polyrust.Sigma), Polyrust.cEqbI a b σ = Polyrust.tb σ (a.eqb b) Poly…` |
| 33 | `Polyrust.cIteB.eq_1` | 等式引理／助手 | 定理 | 定義 `cIteB` 的等式引理（定義的展開方程） | `∀ (c t f : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cIteB c t f σ =     Polyrust.tb σ (c.i…` |
| 34 | `Polyrust.cIteH.eq_1` | 等式引理／助手 | 定理 | 定義 `cIteH` 的等式引理（定義的展開方程） | `∀ (c t f : Polyrust.Expr), Polyrust.cIteH c t f = Polyrust.oneHotC (c.ite t f)` |
| 35 | `Polyrust.cIteI.eq_1` | 等式引理／助手 | 定理 | 定義 `cIteI` 的等式引理（定義的展開方程） | `∀ (c t f : Polyrust.Expr) (σ : Polyrust.Sigma),   Polyrust.cIteI c t f σ =     Polyrust.tb σ (c.i…` |
| 36 | `Polyrust.cNumB.eq_1` | 等式引理／助手 | 定理 | 定義 `cNumB` 的等式引理（定義的展開方程） | `∀ (n : Int) (σ : Polyrust.Sigma), Polyrust.cNumB n σ = Polyrust.tb σ (Polyrust.Expr.num n) Polyru…` |
| 37 | `Polyrust.cNumH.eq_1` | 等式引理／助手 | 定理 | 定義 `cNumH` 的等式引理（定義的展開方程） | `∀ (n : Int), Polyrust.cNumH n = Polyrust.oneHotC (Polyrust.Expr.num n)` |
| 38 | `Polyrust.cNumI.eq_1` | 等式引理／助手 | 定理 | 定義 `cNumI` 的等式引理（定義的展開方程） | `∀ (n : Int) (σ : Polyrust.Sigma), Polyrust.cNumI n σ = Polyrust.tb σ (Polyrust.Expr.num n) Polyru…` |
| 39 | `Polyrust.gen._f` | 等式引理／助手 | 定義 | `gen` 定義編譯時產生的遞迴／匹配助手 | `(x : Polyrust.Expr) → Polyrust.Expr.below x → List Polyrust.Tok` |
| 40 | `Polyrust.gen._sunfold` | 等式引理／助手 | 定義 | `gen` 的結構遞迴展開引理 | `Polyrust.Expr → List Polyrust.Tok` |
| 41 | `Polyrust.gen._unsafe_rec` | 等式引理／助手 | 定義 | `gen` 定義編譯時產生的遞迴／匹配助手 | `Polyrust.Expr → List Polyrust.Tok` |
| 42 | `Polyrust.gen.eq_1` | 等式引理／助手 | 定理 | 定義 `gen` 的等式引理（定義的展開方程） | `∀ (a : Int), Polyrust.gen (Polyrust.Expr.num a) = [Polyrust.Tok.num a]` |
| 43 | `Polyrust.gen.eq_2` | 等式引理／助手 | 定理 | 定義 `gen` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.Expr), Polyrust.gen (a.add a_1) = Polyrust.Tok.add :: (Polyrust.gen a ++ Poly…` |
| 44 | `Polyrust.gen.eq_3` | 等式引理／助手 | 定理 | 定義 `gen` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.Expr), Polyrust.gen (a.eqb a_1) = Polyrust.Tok.eqb :: (Polyrust.gen a ++ Poly…` |
| 45 | `Polyrust.gen.eq_4` | 等式引理／助手 | 定理 | 定義 `gen` 的等式引理（定義的展開方程） | `∀ (a a_1 a_2 : Polyrust.Expr),   Polyrust.gen (a.ite a_1 a_2) = Polyrust.Tok.ite :: (Polyrust.gen…` |
| 46 | `Polyrust.gen.eq_def` | 等式引理／助手 | 定理 | 定義 `gen` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.Expr),   Polyrust.gen x =     match x with     \| Polyrust.Expr.num n => [Polyrust…` |
| 47 | `Polyrust.genC._f` | 等式引理／助手 | 定義 | `genC` 定義編譯時產生的遞迴／匹配助手 | `(x : Polyrust.Expr) → Polyrust.Expr.below x → List (Polyrust.Sigma → Int)` |
| 48 | `Polyrust.genC._sunfold` | 等式引理／助手 | 定義 | `genC` 的結構遞迴展開引理 | `Polyrust.Expr → List (Polyrust.Sigma → Int)` |
| 49 | `Polyrust.genC._unsafe_rec` | 等式引理／助手 | 定義 | `genC` 定義編譯時產生的遞迴／匹配助手 | `Polyrust.Expr → List (Polyrust.Sigma → Int)` |
| 50 | `Polyrust.genC.eq_1` | 等式引理／助手 | 定理 | 定義 `genC` 的等式引理（定義的展開方程） | `∀ (a : Int), Polyrust.genC (Polyrust.Expr.num a) = [Polyrust.cNumI a, Polyrust.cNumB a, Polyrust.…` |
| 51 | `Polyrust.genC.eq_2` | 等式引理／助手 | 定理 | 定義 `genC` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.Expr),   Polyrust.genC (a.add a_1) =     Polyrust.genC a ++ Polyrust.genC a_1…` |
| 52 | `Polyrust.genC.eq_3` | 等式引理／助手 | 定理 | 定義 `genC` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.Expr),   Polyrust.genC (a.eqb a_1) =     Polyrust.genC a ++ Polyrust.genC a_1…` |
| 53 | `Polyrust.genC.eq_4` | 等式引理／助手 | 定理 | 定義 `genC` 的等式引理（定義的展開方程） | `∀ (a a_1 a_2 : Polyrust.Expr),   Polyrust.genC (a.ite a_1 a_2) =     Polyrust.genC a ++ Polyrust.…` |
| 54 | `Polyrust.genC.eq_def` | 等式引理／助手 | 定理 | 定義 `genC` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.Expr),   Polyrust.genC x =     match x with     \| Polyrust.Expr.num n => [Polyrus…` |
| 55 | `Polyrust.isMonoAt_of_root.match_1_1` | 等式引理／助手 | 定義 | `isMonoAt_of_root` 定義編譯時產生的遞迴／匹配助手 | `∀ {e' : Polyrust.Expr} {σ : Polyrust.Sigma} (τ τ' : Polyrust.Ty) (motive : σ e' τ = true ∧ σ e' τ…` |
| 56 | `Polyrust.occurs_oneHot_mem_genC.match_1_1` | 等式引理／助手 | 定義 | `occurs_oneHot_mem_genC` 定義編譯時產生的遞迴／匹配助手 | `∀ (motive : (x x_1 : Polyrust.Expr) → Polyrust.Occurs x x_1 → Prop) (x x_1 : Polyrust.Expr)   (x_…` |
| 57 | `Polyrust.oneHot.eq_1` | 等式引理／助手 | 定理 | 定義 `oneHot` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Sigma) (e : Polyrust.Expr),   Polyrust.oneHot σ e = Polyrust.tb σ e Polyrust.Ty.i…` |
| 58 | `Polyrust.oneHotC.eq_1` | 等式引理／助手 | 定理 | 定義 `oneHotC` 的等式引理（定義的展開方程） | `∀ (e : Polyrust.Expr) (σ : Polyrust.Sigma), Polyrust.oneHotC e σ = Polyrust.oneHot σ e` |
| 59 | `Polyrust.parseFuel._f` | 等式引理／助手 | 定義 | `parseFuel` 定義編譯時產生的遞迴／匹配助手 | `(x : Nat) →   Nat.below (motive := fun x => List Polyrust.Tok → Option (Polyrust.Expr × List Poly…` |
| 60 | `Polyrust.parseFuel._sunfold` | 等式引理／助手 | 定義 | `parseFuel` 的結構遞迴展開引理 | `Nat → List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok)` |
| 61 | `Polyrust.parseFuel._unsafe_rec` | 等式引理／助手 | 定義 | `parseFuel` 定義編譯時產生的遞迴／匹配助手 | `Nat → List Polyrust.Tok → Option (Polyrust.Expr × List Polyrust.Tok)` |
| 62 | `Polyrust.parseFuel.eq_1` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (x : List Polyrust.Tok), Polyrust.parseFuel 0 x = none` |
| 63 | `Polyrust.parseFuel.eq_2` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (n : Nat), Polyrust.parseFuel n.succ [] = none` |
| 64 | `Polyrust.parseFuel.eq_3` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.unit :: rest) =…` |
| 65 | `Polyrust.parseFuel.eq_4` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (n : Nat) (n_1 : Int) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.num…` |
| 66 | `Polyrust.parseFuel.eq_5` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.add :: rest) = …` |
| 67 | `Polyrust.parseFuel.eq_6` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.eqb :: rest) = …` |
| 68 | `Polyrust.parseFuel.eq_7` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (n : Nat) (rest : List Polyrust.Tok),   Polyrust.parseFuel n.succ (Polyrust.Tok.ite :: rest) = …` |
| 69 | `Polyrust.parseFuel.eq_def` | 等式引理／助手 | 定理 | 定義 `parseFuel` 的等式引理（定義的展開方程） | `∀ (x : Nat) (x_1 : List Polyrust.Tok),   Polyrust.parseFuel x x_1 =     match x, x_1 with     \| 0…` |
| 70 | `Polyrust.parseFuel.match_1` | 等式引理／助手 | 定義 | `parseFuel` 定義編譯時產生的遞迴／匹配助手 | `(motive : Option (Polyrust.Expr × List Polyrust.Tok) → Sort u_1) →   (x : Option (Polyrust.Expr ×…` |
| 71 | `Polyrust.parseFuel.match_3` | 等式引理／助手 | 定義 | `parseFuel` 定義編譯時產生的遞迴／匹配助手 | `(motive : Nat → List Polyrust.Tok → Sort u_1) →   (x : Nat) →     (x_1 : List Polyrust.Tok) →    …` |
| 72 | `Polyrust.sizeT._f` | 等式引理／助手 | 定義 | `sizeT` 定義編譯時產生的遞迴／匹配助手 | `(x : Polyrust.Expr) → Polyrust.Expr.below x → Nat` |
| 73 | `Polyrust.sizeT._sunfold` | 等式引理／助手 | 定義 | `sizeT` 的結構遞迴展開引理 | `Polyrust.Expr → Nat` |
| 74 | `Polyrust.sizeT._unsafe_rec` | 等式引理／助手 | 定義 | `sizeT` 定義編譯時產生的遞迴／匹配助手 | `Polyrust.Expr → Nat` |
| 75 | `Polyrust.sizeT.eq_1` | 等式引理／助手 | 定理 | 定義 `sizeT` 的等式引理（定義的展開方程） | `∀ (a : Int), Polyrust.sizeT (Polyrust.Expr.num a) = 1` |
| 76 | `Polyrust.sizeT.eq_2` | 等式引理／助手 | 定理 | 定義 `sizeT` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.Expr), Polyrust.sizeT (a.add a_1) = 1 + (Polyrust.sizeT a + Polyrust.sizeT a_1)` |
| 77 | `Polyrust.sizeT.eq_3` | 等式引理／助手 | 定理 | 定義 `sizeT` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.Expr), Polyrust.sizeT (a.eqb a_1) = 1 + (Polyrust.sizeT a + Polyrust.sizeT a_1)` |
| 78 | `Polyrust.sizeT.eq_4` | 等式引理／助手 | 定理 | 定義 `sizeT` 的等式引理（定義的展開方程） | `∀ (a a_1 a_2 : Polyrust.Expr),   Polyrust.sizeT (a.ite a_1 a_2) = 1 + (Polyrust.sizeT a + Polyrus…` |
| 79 | `Polyrust.sizeT.eq_def` | 等式引理／助手 | 定理 | 定義 `sizeT` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.Expr),   Polyrust.sizeT x =     match x with     \| Polyrust.Expr.num a => 1     \|…` |
| 80 | `Polyrust.tb.eq_1` | 等式引理／助手 | 定理 | 定義 `tb` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Sigma) (e : Polyrust.Expr) (τ : Polyrust.Ty), Polyrust.tb σ e τ = Polyrust.bit (σ…` |
| 81 | `Polyrust.untypable_iff_no_root.match_1_1` | 等式引理／助手 | 定義 | `untypable_iff_no_root` 定義編譯時產生的遞迴／匹配助手 | `∀ (e : Polyrust.Expr) (motive : (∃ σ, Polyrust.IsRoot e σ) → Prop) (h : ∃ σ, Polyrust.IsRoot e σ)…` |
| 82 | `Polyrust.Expr._sizeOf_1` | 歸納型衍生 | 定義 | `Expr` 的輔助大小函數 | `Polyrust.Expr → Nat` |
| 83 | `Polyrust.Expr._sizeOf_inst` | 歸納型衍生 | 定義 | `Expr` 的輔助大小函數 | `SizeOf Polyrust.Expr` |
| 84 | `Polyrust.Expr.add.elim` | 歸納型衍生 | 定義 | `Expr` 的構造子消去器 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 1 → ((a a_1 : Polyrust.Ex…` |
| 85 | `Polyrust.Expr.add.inj` | 歸納型衍生 | 定理 | `Expr` 的構造子結構助手（機器衍生） | `∀ {a a_1 a_2 a_3 : Polyrust.Expr}, a.add a_1 = a_2.add a_3 → a = a_2 ∧ a_1 = a_3` |
| 86 | `Polyrust.Expr.add.noConfusion` | 歸納型衍生 | 定義 | `Expr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a_1 a' a'_1 : Polyrust.Expr} → a.add a_1 = a'.add a'_1 → (a = a' → a_1 = a'_1 →…` |
| 87 | `Polyrust.Expr.add.sizeOf_spec` | 歸納型衍生 | 定理 | `Expr` 結構大小的規格命題 | `∀ (a a_1 : Polyrust.Expr), sizeOf (a.add a_1) = 1 + sizeOf a + sizeOf a_1` |
| 88 | `Polyrust.Expr.below` | 歸納型衍生 | 定義 | `Expr` 的良基遞迴助手 | `{motive : Polyrust.Expr → Sort u} → Polyrust.Expr → Sort (max 1 u)` |
| 89 | `Polyrust.Expr.brecOn` | 歸納型衍生 | 定義 | `Expr` 的良基遞迴助手 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → ((t : Polyrust.Expr) → Polyrust.Expr.…` |
| 90 | `Polyrust.Expr.casesOn` | 歸納型衍生 | 定義 | `Expr` 的案例分析原則 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) →     ((a : Int) → motive (Polyrust.Exp…` |
| 91 | `Polyrust.Expr.ctorElim` | 歸納型衍生 | 定義 | `Expr` 的構造子結構助手（機器衍生） | `{motive : Polyrust.Expr → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Expr) → ctorIdx = t.ctorIdx…` |
| 92 | `Polyrust.Expr.ctorIdx` | 歸納型衍生 | 定義 | `Expr` 的構造子結構助手（機器衍生） | `Polyrust.Expr → Nat` |
| 93 | `Polyrust.Expr.eqb.elim` | 歸納型衍生 | 定義 | `Expr` 的構造子消去器 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 2 → ((a a_1 : Polyrust.Ex…` |
| 94 | `Polyrust.Expr.eqb.inj` | 歸納型衍生 | 定理 | `Expr` 的構造子結構助手（機器衍生） | `∀ {a a_1 a_2 a_3 : Polyrust.Expr}, a.eqb a_1 = a_2.eqb a_3 → a = a_2 ∧ a_1 = a_3` |
| 95 | `Polyrust.Expr.eqb.noConfusion` | 歸納型衍生 | 定義 | `Expr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a_1 a' a'_1 : Polyrust.Expr} → a.eqb a_1 = a'.eqb a'_1 → (a = a' → a_1 = a'_1 →…` |
| 96 | `Polyrust.Expr.eqb.sizeOf_spec` | 歸納型衍生 | 定理 | `Expr` 結構大小的規格命題 | `∀ (a a_1 : Polyrust.Expr), sizeOf (a.eqb a_1) = 1 + sizeOf a + sizeOf a_1` |
| 97 | `Polyrust.Expr.ite.elim` | 歸納型衍生 | 定義 | `Expr` 的構造子消去器 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 3 → ((a a_1 a_2 : Polyrus…` |
| 98 | `Polyrust.Expr.ite.inj` | 歸納型衍生 | 定理 | `Expr` 的構造子結構助手（機器衍生） | `∀ {a a_1 a_2 a_3 a_4 a_5 : Polyrust.Expr}, a.ite a_1 a_2 = a_3.ite a_4 a_5 → a = a_3 ∧ a_1 = a_4 …` |
| 99 | `Polyrust.Expr.ite.noConfusion` | 歸納型衍生 | 定義 | `Expr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {a a_1 a_2 a' a'_1 a'_2 : Polyrust.Expr} →     a.ite a_1 a_2 = a'.ite a'_1 a'_2 …` |
| 100 | `Polyrust.Expr.ite.sizeOf_spec` | 歸納型衍生 | 定理 | `Expr` 結構大小的規格命題 | `∀ (a a_1 a_2 : Polyrust.Expr), sizeOf (a.ite a_1 a_2) = 1 + sizeOf a + sizeOf a_1 + sizeOf a_2` |
| 101 | `Polyrust.Expr.noConfusion` | 歸納型衍生 | 定義 | `Expr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {t t' : Polyrust.Expr} → t = t' → Polyrust.Expr.noConfusionType P t t'` |
| 102 | `Polyrust.Expr.num.elim` | 歸納型衍生 | 定義 | `Expr` 的構造子消去器 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) → t.ctorIdx = 0 → ((a : Int) → motive (…` |
| 103 | `Polyrust.Expr.num.inj` | 歸納型衍生 | 定理 | `Expr` 的構造子結構助手（機器衍生） | `∀ {a a_1 : Int}, Polyrust.Expr.num a = Polyrust.Expr.num a_1 → a = a_1` |
| 104 | `Polyrust.Expr.num.noConfusion` | 歸納型衍生 | 定義 | `Expr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a' : Int} → Polyrust.Expr.num a = Polyrust.Expr.num a' → (a = a' → P) → P` |
| 105 | `Polyrust.Expr.num.sizeOf_spec` | 歸納型衍生 | 定理 | `Expr` 結構大小的規格命題 | `∀ (a : Int), sizeOf (Polyrust.Expr.num a) = 1 + sizeOf a` |
| 106 | `Polyrust.Expr.recOn` | 歸納型衍生 | 定義 | `Expr` 的結構遞迴原則 | `{motive : Polyrust.Expr → Sort u} →   (t : Polyrust.Expr) →     ((a : Int) → motive (Polyrust.Exp…` |
| 107 | `Polyrust.Occurs.below.casesOn` | 歸納型衍生 | 定義 | `Occurs` 的案例分析原則 | `∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop}   {motive_1…` |
| 108 | `Polyrust.Occurs.brecOn` | 歸納型衍生 | 定理 | `Occurs` 的良基遞迴助手 | `∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop} {a_1 : Poly…` |
| 109 | `Polyrust.Occurs.casesOn` | 歸納型衍生 | 定義 | `Occurs` 的案例分析原則 | `∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop} {a_1 : Poly…` |
| 110 | `Polyrust.Occurs.recOn` | 歸納型衍生 | 定義 | `Occurs` 的結構遞迴原則 | `∀ {a : Polyrust.Expr} {motive : (a_1 : Polyrust.Expr) → Polyrust.Occurs a a_1 → Prop} {a_1 : Poly…` |
| 111 | `Polyrust.Tok._sizeOf_1` | 歸納型衍生 | 定義 | `Tok` 的輔助大小函數 | `Polyrust.Tok → Nat` |
| 112 | `Polyrust.Tok._sizeOf_inst` | 歸納型衍生 | 定義 | `Tok` 的輔助大小函數 | `SizeOf Polyrust.Tok` |
| 113 | `Polyrust.Tok.add.elim` | 歸納型衍生 | 定義 | `Tok` 的構造子消去器 | `{motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 2 → motive Polyrust.Tok.add →…` |
| 114 | `Polyrust.Tok.add.sizeOf_spec` | 歸納型衍生 | 定理 | `Tok` 結構大小的規格命題 | `sizeOf Polyrust.Tok.add = 1` |
| 115 | `Polyrust.Tok.casesOn` | 歸納型衍生 | 定義 | `Tok` 的案例分析原則 | `{motive : Polyrust.Tok → Sort u} →   (t : Polyrust.Tok) →     motive Polyrust.Tok.unit →       ((…` |
| 116 | `Polyrust.Tok.ctorElim` | 歸納型衍生 | 定義 | `Tok` 的構造子結構助手（機器衍生） | `{motive : Polyrust.Tok → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Tok) → ctorIdx = t.ctorIdx →…` |
| 117 | `Polyrust.Tok.ctorIdx` | 歸納型衍生 | 定義 | `Tok` 的構造子結構助手（機器衍生） | `Polyrust.Tok → Nat` |
| 118 | `Polyrust.Tok.eqb.elim` | 歸納型衍生 | 定義 | `Tok` 的構造子消去器 | `{motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 3 → motive Polyrust.Tok.eqb →…` |
| 119 | `Polyrust.Tok.eqb.sizeOf_spec` | 歸納型衍生 | 定理 | `Tok` 結構大小的規格命題 | `sizeOf Polyrust.Tok.eqb = 1` |
| 120 | `Polyrust.Tok.ite.elim` | 歸納型衍生 | 定義 | `Tok` 的構造子消去器 | `{motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 4 → motive Polyrust.Tok.ite →…` |
| 121 | `Polyrust.Tok.ite.sizeOf_spec` | 歸納型衍生 | 定理 | `Tok` 結構大小的規格命題 | `sizeOf Polyrust.Tok.ite = 1` |
| 122 | `Polyrust.Tok.noConfusion` | 歸納型衍生 | 定義 | `Tok` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {t t' : Polyrust.Tok} → t = t' → Polyrust.Tok.noConfusionType P t t'` |
| 123 | `Polyrust.Tok.num.elim` | 歸納型衍生 | 定義 | `Tok` 的構造子消去器 | `{motive : Polyrust.Tok → Sort u} →   (t : Polyrust.Tok) → t.ctorIdx = 1 → ((a : Int) → motive (Po…` |
| 124 | `Polyrust.Tok.num.inj` | 歸納型衍生 | 定理 | `Tok` 的構造子結構助手（機器衍生） | `∀ {a a_1 : Int}, Polyrust.Tok.num a = Polyrust.Tok.num a_1 → a = a_1` |
| 125 | `Polyrust.Tok.num.noConfusion` | 歸納型衍生 | 定義 | `Tok` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a' : Int} → Polyrust.Tok.num a = Polyrust.Tok.num a' → (a = a' → P) → P` |
| 126 | `Polyrust.Tok.num.sizeOf_spec` | 歸納型衍生 | 定理 | `Tok` 結構大小的規格命題 | `∀ (a : Int), sizeOf (Polyrust.Tok.num a) = 1 + sizeOf a` |
| 127 | `Polyrust.Tok.recOn` | 歸納型衍生 | 定義 | `Tok` 的結構遞迴原則 | `{motive : Polyrust.Tok → Sort u} →   (t : Polyrust.Tok) →     motive Polyrust.Tok.unit →       ((…` |
| 128 | `Polyrust.Tok.unit.elim` | 歸納型衍生 | 定義 | `Tok` 的構造子消去器 | `{motive : Polyrust.Tok → Sort u} → (t : Polyrust.Tok) → t.ctorIdx = 0 → motive Polyrust.Tok.unit …` |
| 129 | `Polyrust.Tok.unit.sizeOf_spec` | 歸納型衍生 | 定理 | `Tok` 結構大小的規格命題 | `sizeOf Polyrust.Tok.unit = 1` |
| 130 | `Polyrust.Ty._sizeOf_1` | 歸納型衍生 | 定義 | `Ty` 的輔助大小函數 | `Polyrust.Ty → Nat` |
| 131 | `Polyrust.Ty._sizeOf_inst` | 歸納型衍生 | 定義 | `Ty` 的輔助大小函數 | `SizeOf Polyrust.Ty` |
| 132 | `Polyrust.Ty.boolean.elim` | 歸納型衍生 | 定義 | `Ty` 的構造子消去器 | `{motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → t.ctorIdx = 1 → motive Polyrust.Ty.boolean …` |
| 133 | `Polyrust.Ty.boolean.sizeOf_spec` | 歸納型衍生 | 定理 | `Ty` 結構大小的規格命題 | `sizeOf Polyrust.Ty.boolean = 1` |
| 134 | `Polyrust.Ty.casesOn` | 歸納型衍生 | 定義 | `Ty` 的案例分析原則 | `{motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → motive Polyrust.Ty.i32 → motive Polyrust.Ty…` |
| 135 | `Polyrust.Ty.ctorElim` | 歸納型衍生 | 定義 | `Ty` 的構造子結構助手（機器衍生） | `{motive : Polyrust.Ty → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Ty) → ctorIdx = t.ctorIdx → P…` |
| 136 | `Polyrust.Ty.ctorIdx` | 歸納型衍生 | 定義 | `Ty` 的構造子結構助手（機器衍生） | `Polyrust.Ty → Nat` |
| 137 | `Polyrust.Ty.i32.elim` | 歸納型衍生 | 定義 | `Ty` 的構造子消去器 | `{motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → t.ctorIdx = 0 → motive Polyrust.Ty.i32 → mo…` |
| 138 | `Polyrust.Ty.i32.sizeOf_spec` | 歸納型衍生 | 定理 | `Ty` 結構大小的規格命題 | `sizeOf Polyrust.Ty.i32 = 1` |
| 139 | `Polyrust.Ty.noConfusion` | 歸納型衍生 | 定義 | `Ty` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort v✝} → {x y : Polyrust.Ty} → x = y → Polyrust.Ty.noConfusionType P x y` |
| 140 | `Polyrust.Ty.ofNat` | 歸納型衍生 | 定義 | `Ty` 的自然數字面量實例 | `Nat → Polyrust.Ty` |
| 141 | `Polyrust.Ty.recOn` | 歸納型衍生 | 定義 | `Ty` 的結構遞迴原則 | `{motive : Polyrust.Ty → Sort u} → (t : Polyrust.Ty) → motive Polyrust.Ty.i32 → motive Polyrust.Ty…` |
| 142 | `Polyrust.Ty.toCtorIdx` | 歸納型衍生 | 定義 | `Ty` 的構造子結構助手（機器衍生） | `Polyrust.Ty → Nat` |
| 143 | `Polyrust.instDecidableEqExpr` | 實例衍生 | 定義 | `Expr` 可判定相等（DecidableEq）的判定程序 | `DecidableEq Polyrust.Expr` |
| 144 | `Polyrust.instDecidableEqExpr.decEq` | 實例衍生 | 定義 | `Expr` 可判定相等（DecidableEq）的判定程序 | `(x x_1 : Polyrust.Expr) → Decidable (x = x_1)` |
| 145 | `Polyrust.instDecidableEqExpr.decEq._f` | 實例衍生 | 定義 | `Expr` 相等性判定的遞迴助手（機器衍生） | `(x : Polyrust.Expr) →   Polyrust.Expr.below (motive := fun x => (x_1 : Polyrust.Expr) → Decidable…` |
| 146 | `Polyrust.instDecidableEqExpr.decEq._proof_1` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Expr.num a = Polyrust.Expr.num a` |
| 147 | `Polyrust.instDecidableEqExpr.decEq._proof_10` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 : Polyrust.Expr), a.add a_1 = a_2.eqb a_3 → False` |
| 148 | `Polyrust.instDecidableEqExpr.decEq._proof_11` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.add a_1 = a_2.ite a_3 a_4 → False` |
| 149 | `Polyrust.instDecidableEqExpr.decEq._proof_12` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.Expr) (a_2 : Int), a.eqb a_1 = Polyrust.Expr.num a_2 → False` |
| 150 | `Polyrust.instDecidableEqExpr.decEq._proof_13` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 : Polyrust.Expr), a.eqb a_1 = a_2.add a_3 → False` |
| 151 | `Polyrust.instDecidableEqExpr.decEq._proof_14` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.Expr), a.eqb a_1 = a.eqb a_1` |
| 152 | `Polyrust.instDecidableEqExpr.decEq._proof_15` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b : Polyrust.Expr), ¬a_1 = b → a.eqb a_1 = a.eqb b → False` |
| 153 | `Polyrust.instDecidableEqExpr.decEq._proof_16` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b b_1 : Polyrust.Expr), ¬a = b → a.eqb a_1 = b.eqb b_1 → False` |
| 154 | `Polyrust.instDecidableEqExpr.decEq._proof_17` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.eqb a_1 = a_2.ite a_3 a_4 → False` |
| 155 | `Polyrust.instDecidableEqExpr.decEq._proof_18` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 : Polyrust.Expr) (a_3 : Int), a.ite a_1 a_2 = Polyrust.Expr.num a_3 → False` |
| 156 | `Polyrust.instDecidableEqExpr.decEq._proof_19` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.ite a_1 a_2 = a_3.add a_4 → False` |
| 157 | `Polyrust.instDecidableEqExpr.decEq._proof_2` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a b : Int), ¬a = b → Polyrust.Expr.num a = Polyrust.Expr.num b → False` |
| 158 | `Polyrust.instDecidableEqExpr.decEq._proof_20` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.Expr), a.ite a_1 a_2 = a_3.eqb a_4 → False` |
| 159 | `Polyrust.instDecidableEqExpr.decEq._proof_21` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 : Polyrust.Expr), a.ite a_1 a_2 = a.ite a_1 a_2` |
| 160 | `Polyrust.instDecidableEqExpr.decEq._proof_22` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 b : Polyrust.Expr), ¬a_2 = b → a.ite a_1 a_2 = a.ite a_1 b → False` |
| 161 | `Polyrust.instDecidableEqExpr.decEq._proof_23` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 b b_1 : Polyrust.Expr), ¬a_1 = b → a.ite a_1 a_2 = a.ite b b_1 → False` |
| 162 | `Polyrust.instDecidableEqExpr.decEq._proof_24` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 b b_1 b_2 : Polyrust.Expr), ¬a = b → a.ite a_1 a_2 = b.ite b_1 b_2 → False` |
| 163 | `Polyrust.instDecidableEqExpr.decEq._proof_3` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 a_2 : Polyrust.Expr), Polyrust.Expr.num a = a_1.add a_2 → False` |
| 164 | `Polyrust.instDecidableEqExpr.decEq._proof_4` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 a_2 : Polyrust.Expr), Polyrust.Expr.num a = a_1.eqb a_2 → False` |
| 165 | `Polyrust.instDecidableEqExpr.decEq._proof_5` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 a_2 a_3 : Polyrust.Expr), Polyrust.Expr.num a = a_1.ite a_2 a_3 → False` |
| 166 | `Polyrust.instDecidableEqExpr.decEq._proof_6` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.Expr) (a_2 : Int), a.add a_1 = Polyrust.Expr.num a_2 → False` |
| 167 | `Polyrust.instDecidableEqExpr.decEq._proof_7` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.Expr), a.add a_1 = a.add a_1` |
| 168 | `Polyrust.instDecidableEqExpr.decEq._proof_8` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b : Polyrust.Expr), ¬a_1 = b → a.add a_1 = a.add b → False` |
| 169 | `Polyrust.instDecidableEqExpr.decEq._proof_9` | 實例衍生 | 定理 | `Expr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b b_1 : Polyrust.Expr), ¬a = b → a.add a_1 = b.add b_1 → False` |
| 170 | `Polyrust.instDecidableEqExpr.decEq._sunfold` | 實例衍生 | 定義 | `Expr` 相等性判定的展開引理（機器衍生） | `(x x_1 : Polyrust.Expr) → Decidable (x = x_1)` |
| 171 | `Polyrust.instDecidableEqExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | `Expr` 相等性判定的遞迴助手（機器衍生） | `(x x_1 : Polyrust.Expr) → Decidable (x = x_1)` |
| 172 | `Polyrust.instDecidableEqExpr.decEq.match_1` | 實例衍生 | 定義 | `Expr` 相等性判定的輔助匹配（機器衍生） | `(motive : Polyrust.Expr → Polyrust.Expr → Sort u_1) →   (x x_1 : Polyrust.Expr) →     ((a b : Int…` |
| 173 | `Polyrust.instDecidableEqTok` | 實例衍生 | 定義 | `Tok` 可判定相等（DecidableEq）的判定程序 | `DecidableEq Polyrust.Tok` |
| 174 | `Polyrust.instDecidableEqTok.decEq` | 實例衍生 | 定義 | `Tok` 可判定相等（DecidableEq）的判定程序 | `(x x_1 : Polyrust.Tok) → Decidable (x = x_1)` |
| 175 | `Polyrust.instDecidableEqTok.decEq._proof_1` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.unit = Polyrust.Tok.num a → False` |
| 176 | `Polyrust.instDecidableEqTok.decEq._proof_10` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.ite → False` |
| 177 | `Polyrust.instDecidableEqTok.decEq._proof_11` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.add = Polyrust.Tok.unit → False` |
| 178 | `Polyrust.instDecidableEqTok.decEq._proof_12` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.add = Polyrust.Tok.num a → False` |
| 179 | `Polyrust.instDecidableEqTok.decEq._proof_13` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.add = Polyrust.Tok.eqb → False` |
| 180 | `Polyrust.instDecidableEqTok.decEq._proof_14` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.add = Polyrust.Tok.ite → False` |
| 181 | `Polyrust.instDecidableEqTok.decEq._proof_15` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.eqb = Polyrust.Tok.unit → False` |
| 182 | `Polyrust.instDecidableEqTok.decEq._proof_16` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.eqb = Polyrust.Tok.num a → False` |
| 183 | `Polyrust.instDecidableEqTok.decEq._proof_17` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.eqb = Polyrust.Tok.add → False` |
| 184 | `Polyrust.instDecidableEqTok.decEq._proof_18` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.eqb = Polyrust.Tok.ite → False` |
| 185 | `Polyrust.instDecidableEqTok.decEq._proof_19` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.ite = Polyrust.Tok.unit → False` |
| 186 | `Polyrust.instDecidableEqTok.decEq._proof_2` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.unit = Polyrust.Tok.add → False` |
| 187 | `Polyrust.instDecidableEqTok.decEq._proof_20` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.ite = Polyrust.Tok.num a → False` |
| 188 | `Polyrust.instDecidableEqTok.decEq._proof_21` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.ite = Polyrust.Tok.add → False` |
| 189 | `Polyrust.instDecidableEqTok.decEq._proof_22` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.ite = Polyrust.Tok.eqb → False` |
| 190 | `Polyrust.instDecidableEqTok.decEq._proof_3` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.unit = Polyrust.Tok.eqb → False` |
| 191 | `Polyrust.instDecidableEqTok.decEq._proof_4` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `Polyrust.Tok.unit = Polyrust.Tok.ite → False` |
| 192 | `Polyrust.instDecidableEqTok.decEq._proof_5` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.unit → False` |
| 193 | `Polyrust.instDecidableEqTok.decEq._proof_6` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.num a` |
| 194 | `Polyrust.instDecidableEqTok.decEq._proof_7` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a b : Int), ¬a = b → Polyrust.Tok.num a = Polyrust.Tok.num b → False` |
| 195 | `Polyrust.instDecidableEqTok.decEq._proof_8` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.add → False` |
| 196 | `Polyrust.instDecidableEqTok.decEq._proof_9` | 實例衍生 | 定理 | `Tok` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.Tok.num a = Polyrust.Tok.eqb → False` |
| 197 | `Polyrust.instDecidableEqTok.decEq.match_1` | 實例衍生 | 定義 | `Tok` 相等性判定的輔助匹配（機器衍生） | `(motive : Polyrust.Tok → Polyrust.Tok → Sort u_1) →   (x x_1 : Polyrust.Tok) →     (Unit → motive…` |
| 198 | `Polyrust.instDecidableEqTy` | 實例衍生 | 定義 | `Ty` 可判定相等（DecidableEq）的判定程序 | `DecidableEq Polyrust.Ty` |
| 199 | `Polyrust.instDecidableEqTy._proof_1` | 實例衍生 | 定理 | `Ty` 相等性判定的內部子證明（機器衍生） | `∀ (x y : Polyrust.Ty), x.ctorIdx = y.ctorIdx → x = y` |
| 200 | `Polyrust.instDecidableEqTy._proof_2` | 實例衍生 | 定理 | `Ty` 相等性判定的內部子證明（機器衍生） | `∀ (x y : Polyrust.Ty), ¬x.ctorIdx = y.ctorIdx → x = y → False` |
| 201 | `Polyrust.instReprExpr.repr.match_1` | 實例衍生 | 定義 | `Expr` 顯示函數的機器衍生助手 | `(motive : Polyrust.Expr → Sort u_1) →   (x : Polyrust.Expr) →     ((a : Int) → motive (Polyrust.E…` |
| 202 | `Polyrust.instReprTok.repr.match_1` | 實例衍生 | 定義 | `Tok` 顯示函數的機器衍生助手 | `(motive : Polyrust.Tok → Sort u_1) →   (x : Polyrust.Tok) →     (Unit → motive Polyrust.Tok.unit)…` |
| 203 | `Polyrust.instReprTy` | 實例衍生 | 定義 | `Ty` 顯示函數的機器衍生助手 | `Repr Polyrust.Ty` |
| 204 | `Polyrust.instReprTy.repr` | 實例衍生 | 定義 | `Ty` 的顯示函數（Repr 型別類別實例） | `Polyrust.Ty → Nat → Format` |
| 205 | `Polyrust.instReprTy.repr.match_1` | 實例衍生 | 定義 | `Ty` 顯示函數的機器衍生助手 | `(motive : Polyrust.Ty → Sort u_1) →   (x : Polyrust.Ty) → (Unit → motive Polyrust.Ty.i32) → (Unit…` |
| 206 | `Polyrust.Expr.brecOn.eq` | 衍生 | 定理 | `Expr` 相關助手（機器衍生） | `∀ {motive : Polyrust.Expr → Sort u} (t : Polyrust.Expr) (F_1 : (t : Polyrust.Expr) → Polyrust.Exp…` |
| 207 | `Polyrust.Expr.ctorElimType` | 衍生 | 定義 | `Expr` 相關助手（機器衍生） | `{motive : Polyrust.Expr → Sort u} → Nat → Sort (max 1 u)` |
| 208 | `Polyrust.Expr.noConfusionType` | 衍生 | 定義 | `Expr` 相關助手（機器衍生） | `Sort u → Polyrust.Expr → Polyrust.Expr → Sort u` |
| 209 | `Polyrust.Tok.ctorElimType` | 衍生 | 定義 | `Tok` 相關助手（機器衍生） | `{motive : Polyrust.Tok → Sort u} → Nat → Sort (max 1 u)` |
| 210 | `Polyrust.Tok.noConfusionType` | 衍生 | 定義 | `Tok` 相關助手（機器衍生） | `Sort u → Polyrust.Tok → Polyrust.Tok → Sort u` |
| 211 | `Polyrust.Ty.ctorElimType` | 衍生 | 定義 | `Ty` 相關助手（機器衍生） | `{motive : Polyrust.Ty → Sort u} → Nat → Sort (max 1 u)` |
| 212 | `Polyrust.Ty.noConfusionType` | 衍生 | 定義 | `Ty` 相關助手（機器衍生） | `Sort v✝ → Polyrust.Ty → Polyrust.Ty → Sort v✝` |
| 213 | `Polyrust.Ty.ofNat_ctorIdx` | 衍生 | 定理 | `Ty` 相關助手（機器衍生） | `∀ (x : Polyrust.Ty), Polyrust.Ty.ofNat x.ctorIdx = x` |

## `Polyrust.MacroExpansion`（109 條）

職責：巨集展開的語義

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 214 | `Polyrust.Ctx` | 手寫 | 定義 | 模板上下文：變量 → 型別（`none` = 未宣告）。 | `Type` |
| 215 | `Polyrust.armCtx` | 手寫 | 定義 | 模板臂的宣告上下文：`v` 被宣告為 i32，其餘未宣告。 | `Nat → Polyrust.Ctx` |
| 216 | `Polyrust.armTemplate` | 手寫 | 定義 | ! ## 七、T7(b) 判定面：錯臂必被拒絕 要求 i32 參數的模板臂：`armTemplate v = (v + 1)`。 | `Nat → Polyrust.VExpr` |
| 217 | `Polyrust.expand` | 手寫 | 定義 | **展開**（模板 → 調用點程序）：把模板變量換成 T9 語言中的實際節點。 | `(Nat → Polyrust.Expr) → Polyrust.VExpr → Polyrust.Expr` |
| 218 | `Polyrust.subst` | 手寫 | 定義 | **語法代入**（模板 → 模板）。 | `(Nat → Polyrust.VExpr) → Polyrust.VExpr → Polyrust.VExpr` |
| 219 | `Polyrust.vars` | 手寫 | 定義 | 自由變量出現（列表，可重複）。 | `Polyrust.VExpr → List Nat` |
| 220 | `Polyrust.VExpr.brecOn.go` | 等式引理／助手 | 定義 | `VExpr` 定義編譯時產生的遞迴／匹配助手 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → ((t : Polyrust.VExpr) → Polyrust.VE…` |
| 221 | `Polyrust.armCtx.eq_1` | 等式引理／助手 | 定理 | 定義 `armCtx` 的等式引理（定義的展開方程） | `∀ (v w : Nat), Polyrust.armCtx v w = if w = v then some Polyrust.Ty.i32 else none` |
| 222 | `Polyrust.armTemplate.eq_1` | 等式引理／助手 | 定理 | 定義 `armTemplate` 的等式引理（定義的展開方程） | `∀ (v : Nat), Polyrust.armTemplate v = (Polyrust.VExpr.var v).add (Polyrust.VExpr.num 1)` |
| 223 | `Polyrust.expand._f` | 等式引理／助手 | 定義 | `expand` 定義編譯時產生的遞迴／匹配助手 | `(Nat → Polyrust.Expr) → (x : Polyrust.VExpr) → Polyrust.VExpr.below x → Polyrust.Expr` |
| 224 | `Polyrust.expand._sunfold` | 等式引理／助手 | 定義 | `expand` 的結構遞迴展開引理 | `(Nat → Polyrust.Expr) → Polyrust.VExpr → Polyrust.Expr` |
| 225 | `Polyrust.expand._unsafe_rec` | 等式引理／助手 | 定義 | `expand` 定義編譯時產生的遞迴／匹配助手 | `(Nat → Polyrust.Expr) → Polyrust.VExpr → Polyrust.Expr` |
| 226 | `Polyrust.expand.eq_1` | 等式引理／助手 | 定理 | 定義 `expand` 的等式引理（定義的展開方程） | `∀ (σ : Nat → Polyrust.Expr) (a : Nat), Polyrust.expand σ (Polyrust.VExpr.var a) = σ a` |
| 227 | `Polyrust.expand.eq_2` | 等式引理／助手 | 定理 | 定義 `expand` 的等式引理（定義的展開方程） | `∀ (σ : Nat → Polyrust.Expr) (a : Int), Polyrust.expand σ (Polyrust.VExpr.num a) = Polyrust.Expr.n…` |
| 228 | `Polyrust.expand.eq_3` | 等式引理／助手 | 定理 | 定義 `expand` 的等式引理（定義的展開方程） | `∀ (σ : Nat → Polyrust.Expr) (a a_1 : Polyrust.VExpr),   Polyrust.expand σ (a.add a_1) = (Polyrust…` |
| 229 | `Polyrust.expand.eq_4` | 等式引理／助手 | 定理 | 定義 `expand` 的等式引理（定義的展開方程） | `∀ (σ : Nat → Polyrust.Expr) (a a_1 : Polyrust.VExpr),   Polyrust.expand σ (a.eqb a_1) = (Polyrust…` |
| 230 | `Polyrust.expand.eq_5` | 等式引理／助手 | 定理 | 定義 `expand` 的等式引理（定義的展開方程） | `∀ (σ : Nat → Polyrust.Expr) (a a_1 a_2 : Polyrust.VExpr),   Polyrust.expand σ (a.ite a_1 a_2) = (…` |
| 231 | `Polyrust.expand.eq_def` | 等式引理／助手 | 定理 | 定義 `expand` 的等式引理（定義的展開方程） | `∀ (σ : Nat → Polyrust.Expr) (x : Polyrust.VExpr),   Polyrust.expand σ x =     match x with     \| …` |
| 232 | `Polyrust.subst._f` | 等式引理／助手 | 定義 | `subst` 定義編譯時產生的遞迴／匹配助手 | `(Nat → Polyrust.VExpr) → (x : Polyrust.VExpr) → Polyrust.VExpr.below x → Polyrust.VExpr` |
| 233 | `Polyrust.subst._sunfold` | 等式引理／助手 | 定義 | `subst` 的結構遞迴展開引理 | `(Nat → Polyrust.VExpr) → Polyrust.VExpr → Polyrust.VExpr` |
| 234 | `Polyrust.subst._unsafe_rec` | 等式引理／助手 | 定義 | `subst` 定義編譯時產生的遞迴／匹配助手 | `(Nat → Polyrust.VExpr) → Polyrust.VExpr → Polyrust.VExpr` |
| 235 | `Polyrust.subst.eq_1` | 等式引理／助手 | 定理 | 定義 `subst` 的等式引理（定義的展開方程） | `∀ (ρ : Nat → Polyrust.VExpr) (a : Nat), Polyrust.subst ρ (Polyrust.VExpr.var a) = ρ a` |
| 236 | `Polyrust.subst.eq_2` | 等式引理／助手 | 定理 | 定義 `subst` 的等式引理（定義的展開方程） | `∀ (ρ : Nat → Polyrust.VExpr) (a : Int), Polyrust.subst ρ (Polyrust.VExpr.num a) = Polyrust.VExpr.…` |
| 237 | `Polyrust.subst.eq_3` | 等式引理／助手 | 定理 | 定義 `subst` 的等式引理（定義的展開方程） | `∀ (ρ : Nat → Polyrust.VExpr) (a a_1 : Polyrust.VExpr),   Polyrust.subst ρ (a.add a_1) = (Polyrust…` |
| 238 | `Polyrust.subst.eq_4` | 等式引理／助手 | 定理 | 定義 `subst` 的等式引理（定義的展開方程） | `∀ (ρ : Nat → Polyrust.VExpr) (a a_1 : Polyrust.VExpr),   Polyrust.subst ρ (a.eqb a_1) = (Polyrust…` |
| 239 | `Polyrust.subst.eq_5` | 等式引理／助手 | 定理 | 定義 `subst` 的等式引理（定義的展開方程） | `∀ (ρ : Nat → Polyrust.VExpr) (a a_1 a_2 : Polyrust.VExpr),   Polyrust.subst ρ (a.ite a_1 a_2) = (…` |
| 240 | `Polyrust.subst.eq_def` | 等式引理／助手 | 定理 | 定義 `subst` 的等式引理（定義的展開方程） | `∀ (ρ : Nat → Polyrust.VExpr) (x : Polyrust.VExpr),   Polyrust.subst ρ x =     match x with     \| …` |
| 241 | `Polyrust.vars._f` | 等式引理／助手 | 定義 | `vars` 定義編譯時產生的遞迴／匹配助手 | `(x : Polyrust.VExpr) → Polyrust.VExpr.below x → List Nat` |
| 242 | `Polyrust.vars._sunfold` | 等式引理／助手 | 定義 | `vars` 的結構遞迴展開引理 | `Polyrust.VExpr → List Nat` |
| 243 | `Polyrust.vars._unsafe_rec` | 等式引理／助手 | 定義 | `vars` 定義編譯時產生的遞迴／匹配助手 | `Polyrust.VExpr → List Nat` |
| 244 | `Polyrust.vars.eq_1` | 等式引理／助手 | 定理 | 定義 `vars` 的等式引理（定義的展開方程） | `∀ (a : Nat), Polyrust.vars (Polyrust.VExpr.var a) = [a]` |
| 245 | `Polyrust.vars.eq_2` | 等式引理／助手 | 定理 | 定義 `vars` 的等式引理（定義的展開方程） | `∀ (a : Int), Polyrust.vars (Polyrust.VExpr.num a) = []` |
| 246 | `Polyrust.vars.eq_3` | 等式引理／助手 | 定理 | 定義 `vars` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.VExpr), Polyrust.vars (a.add a_1) = Polyrust.vars a ++ Polyrust.vars a_1` |
| 247 | `Polyrust.vars.eq_4` | 等式引理／助手 | 定理 | 定義 `vars` 的等式引理（定義的展開方程） | `∀ (a a_1 : Polyrust.VExpr), Polyrust.vars (a.eqb a_1) = Polyrust.vars a ++ Polyrust.vars a_1` |
| 248 | `Polyrust.vars.eq_5` | 等式引理／助手 | 定理 | 定義 `vars` 的等式引理（定義的展開方程） | `∀ (a a_1 a_2 : Polyrust.VExpr),   Polyrust.vars (a.ite a_1 a_2) = Polyrust.vars a ++ Polyrust.var…` |
| 249 | `Polyrust.vars.eq_def` | 等式引理／助手 | 定理 | 定義 `vars` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.VExpr),   Polyrust.vars x =     match x with     \| Polyrust.VExpr.var v => [v]   …` |
| 250 | `Polyrust.VExpr._sizeOf_1` | 歸納型衍生 | 定義 | `VExpr` 的輔助大小函數 | `Polyrust.VExpr → Nat` |
| 251 | `Polyrust.VExpr._sizeOf_inst` | 歸納型衍生 | 定義 | `VExpr` 的輔助大小函數 | `SizeOf Polyrust.VExpr` |
| 252 | `Polyrust.VExpr.add.elim` | 歸納型衍生 | 定義 | `VExpr` 的構造子消去器 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 2 → ((a a_1 : Polyrust.…` |
| 253 | `Polyrust.VExpr.add.inj` | 歸納型衍生 | 定理 | `VExpr` 的構造子結構助手（機器衍生） | `∀ {a a_1 a_2 a_3 : Polyrust.VExpr}, a.add a_1 = a_2.add a_3 → a = a_2 ∧ a_1 = a_3` |
| 254 | `Polyrust.VExpr.add.noConfusion` | 歸納型衍生 | 定義 | `VExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a_1 a' a'_1 : Polyrust.VExpr} → a.add a_1 = a'.add a'_1 → (a = a' → a_1 = a'_1 …` |
| 255 | `Polyrust.VExpr.add.sizeOf_spec` | 歸納型衍生 | 定理 | `VExpr` 結構大小的規格命題 | `∀ (a a_1 : Polyrust.VExpr), sizeOf (a.add a_1) = 1 + sizeOf a + sizeOf a_1` |
| 256 | `Polyrust.VExpr.below` | 歸納型衍生 | 定義 | `VExpr` 的良基遞迴助手 | `{motive : Polyrust.VExpr → Sort u} → Polyrust.VExpr → Sort (max 1 u)` |
| 257 | `Polyrust.VExpr.brecOn` | 歸納型衍生 | 定義 | `VExpr` 的良基遞迴助手 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → ((t : Polyrust.VExpr) → Polyrust.VE…` |
| 258 | `Polyrust.VExpr.casesOn` | 歸納型衍生 | 定義 | `VExpr` 的案例分析原則 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) →     ((a : Nat) → motive (Polyrust.V…` |
| 259 | `Polyrust.VExpr.ctorElim` | 歸納型衍生 | 定義 | `VExpr` 的構造子結構助手（機器衍生） | `{motive : Polyrust.VExpr → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.VExpr) → ctorIdx = t.ctorI…` |
| 260 | `Polyrust.VExpr.ctorIdx` | 歸納型衍生 | 定義 | `VExpr` 的構造子結構助手（機器衍生） | `Polyrust.VExpr → Nat` |
| 261 | `Polyrust.VExpr.eqb.elim` | 歸納型衍生 | 定義 | `VExpr` 的構造子消去器 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 3 → ((a a_1 : Polyrust.…` |
| 262 | `Polyrust.VExpr.eqb.inj` | 歸納型衍生 | 定理 | `VExpr` 的構造子結構助手（機器衍生） | `∀ {a a_1 a_2 a_3 : Polyrust.VExpr}, a.eqb a_1 = a_2.eqb a_3 → a = a_2 ∧ a_1 = a_3` |
| 263 | `Polyrust.VExpr.eqb.noConfusion` | 歸納型衍生 | 定義 | `VExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a_1 a' a'_1 : Polyrust.VExpr} → a.eqb a_1 = a'.eqb a'_1 → (a = a' → a_1 = a'_1 …` |
| 264 | `Polyrust.VExpr.eqb.sizeOf_spec` | 歸納型衍生 | 定理 | `VExpr` 結構大小的規格命題 | `∀ (a a_1 : Polyrust.VExpr), sizeOf (a.eqb a_1) = 1 + sizeOf a + sizeOf a_1` |
| 265 | `Polyrust.VExpr.ite.elim` | 歸納型衍生 | 定義 | `VExpr` 的構造子消去器 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 4 → ((a a_1 a_2 : Polyr…` |
| 266 | `Polyrust.VExpr.ite.inj` | 歸納型衍生 | 定理 | `VExpr` 的構造子結構助手（機器衍生） | `∀ {a a_1 a_2 a_3 a_4 a_5 : Polyrust.VExpr}, a.ite a_1 a_2 = a_3.ite a_4 a_5 → a = a_3 ∧ a_1 = a_4…` |
| 267 | `Polyrust.VExpr.ite.noConfusion` | 歸納型衍生 | 定義 | `VExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {a a_1 a_2 a' a'_1 a'_2 : Polyrust.VExpr} →     a.ite a_1 a_2 = a'.ite a'_1 a'_2…` |
| 268 | `Polyrust.VExpr.ite.sizeOf_spec` | 歸納型衍生 | 定理 | `VExpr` 結構大小的規格命題 | `∀ (a a_1 a_2 : Polyrust.VExpr), sizeOf (a.ite a_1 a_2) = 1 + sizeOf a + sizeOf a_1 + sizeOf a_2` |
| 269 | `Polyrust.VExpr.noConfusion` | 歸納型衍生 | 定義 | `VExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {t t' : Polyrust.VExpr} → t = t' → Polyrust.VExpr.noConfusionType P t t'` |
| 270 | `Polyrust.VExpr.num.elim` | 歸納型衍生 | 定義 | `VExpr` 的構造子消去器 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 1 → ((a : Int) → motive…` |
| 271 | `Polyrust.VExpr.num.inj` | 歸納型衍生 | 定理 | `VExpr` 的構造子結構助手（機器衍生） | `∀ {a a_1 : Int}, Polyrust.VExpr.num a = Polyrust.VExpr.num a_1 → a = a_1` |
| 272 | `Polyrust.VExpr.num.noConfusion` | 歸納型衍生 | 定義 | `VExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a' : Int} → Polyrust.VExpr.num a = Polyrust.VExpr.num a' → (a = a' → P) → P` |
| 273 | `Polyrust.VExpr.num.sizeOf_spec` | 歸納型衍生 | 定理 | `VExpr` 結構大小的規格命題 | `∀ (a : Int), sizeOf (Polyrust.VExpr.num a) = 1 + sizeOf a` |
| 274 | `Polyrust.VExpr.recOn` | 歸納型衍生 | 定義 | `VExpr` 的結構遞迴原則 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) →     ((a : Nat) → motive (Polyrust.V…` |
| 275 | `Polyrust.VExpr.var.elim` | 歸納型衍生 | 定義 | `VExpr` 的構造子消去器 | `{motive : Polyrust.VExpr → Sort u} →   (t : Polyrust.VExpr) → t.ctorIdx = 0 → ((a : Nat) → motive…` |
| 276 | `Polyrust.VExpr.var.inj` | 歸納型衍生 | 定理 | `VExpr` 的構造子結構助手（機器衍生） | `∀ {a a_1 : Nat}, Polyrust.VExpr.var a = Polyrust.VExpr.var a_1 → a = a_1` |
| 277 | `Polyrust.VExpr.var.noConfusion` | 歸納型衍生 | 定義 | `VExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {a a' : Nat} → Polyrust.VExpr.var a = Polyrust.VExpr.var a' → (a = a' → P) → P` |
| 278 | `Polyrust.VExpr.var.sizeOf_spec` | 歸納型衍生 | 定理 | `VExpr` 結構大小的規格命題 | `∀ (a : Nat), sizeOf (Polyrust.VExpr.var a) = 1 + sizeOf a` |
| 279 | `Polyrust.instDecidableEqVExpr` | 實例衍生 | 定義 | `VExpr` 可判定相等（DecidableEq）的判定程序 | `DecidableEq Polyrust.VExpr` |
| 280 | `Polyrust.instDecidableEqVExpr.decEq` | 實例衍生 | 定義 | `VExpr` 可判定相等（DecidableEq）的判定程序 | `(x x_1 : Polyrust.VExpr) → Decidable (x = x_1)` |
| 281 | `Polyrust.instDecidableEqVExpr.decEq._f` | 實例衍生 | 定義 | `VExpr` 相等性判定的遞迴助手（機器衍生） | `(x : Polyrust.VExpr) →   Polyrust.VExpr.below (motive := fun x => (x_1 : Polyrust.VExpr) → Decida…` |
| 282 | `Polyrust.instDecidableEqVExpr.decEq._proof_1` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Nat), Polyrust.VExpr.var a = Polyrust.VExpr.var a` |
| 283 | `Polyrust.instDecidableEqVExpr.decEq._proof_10` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.num a = a_1.add a_2 → False` |
| 284 | `Polyrust.instDecidableEqVExpr.decEq._proof_11` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.num a = a_1.eqb a_2 → False` |
| 285 | `Polyrust.instDecidableEqVExpr.decEq._proof_12` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 a_2 a_3 : Polyrust.VExpr), Polyrust.VExpr.num a = a_1.ite a_2 a_3 → False` |
| 286 | `Polyrust.instDecidableEqVExpr.decEq._proof_13` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.VExpr) (a_2 : Nat), a.add a_1 = Polyrust.VExpr.var a_2 → False` |
| 287 | `Polyrust.instDecidableEqVExpr.decEq._proof_14` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.VExpr) (a_2 : Int), a.add a_1 = Polyrust.VExpr.num a_2 → False` |
| 288 | `Polyrust.instDecidableEqVExpr.decEq._proof_15` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.VExpr), a.add a_1 = a.add a_1` |
| 289 | `Polyrust.instDecidableEqVExpr.decEq._proof_16` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b : Polyrust.VExpr), ¬a_1 = b → a.add a_1 = a.add b → False` |
| 290 | `Polyrust.instDecidableEqVExpr.decEq._proof_17` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b b_1 : Polyrust.VExpr), ¬a = b → a.add a_1 = b.add b_1 → False` |
| 291 | `Polyrust.instDecidableEqVExpr.decEq._proof_18` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 : Polyrust.VExpr), a.add a_1 = a_2.eqb a_3 → False` |
| 292 | `Polyrust.instDecidableEqVExpr.decEq._proof_19` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.add a_1 = a_2.ite a_3 a_4 → False` |
| 293 | `Polyrust.instDecidableEqVExpr.decEq._proof_2` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a b : Nat), ¬a = b → Polyrust.VExpr.var a = Polyrust.VExpr.var b → False` |
| 294 | `Polyrust.instDecidableEqVExpr.decEq._proof_20` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.VExpr) (a_2 : Nat), a.eqb a_1 = Polyrust.VExpr.var a_2 → False` |
| 295 | `Polyrust.instDecidableEqVExpr.decEq._proof_21` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.VExpr) (a_2 : Int), a.eqb a_1 = Polyrust.VExpr.num a_2 → False` |
| 296 | `Polyrust.instDecidableEqVExpr.decEq._proof_22` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 : Polyrust.VExpr), a.eqb a_1 = a_2.add a_3 → False` |
| 297 | `Polyrust.instDecidableEqVExpr.decEq._proof_23` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 : Polyrust.VExpr), a.eqb a_1 = a.eqb a_1` |
| 298 | `Polyrust.instDecidableEqVExpr.decEq._proof_24` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b : Polyrust.VExpr), ¬a_1 = b → a.eqb a_1 = a.eqb b → False` |
| 299 | `Polyrust.instDecidableEqVExpr.decEq._proof_25` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 b b_1 : Polyrust.VExpr), ¬a = b → a.eqb a_1 = b.eqb b_1 → False` |
| 300 | `Polyrust.instDecidableEqVExpr.decEq._proof_26` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.eqb a_1 = a_2.ite a_3 a_4 → False` |
| 301 | `Polyrust.instDecidableEqVExpr.decEq._proof_27` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 : Polyrust.VExpr) (a_3 : Nat), a.ite a_1 a_2 = Polyrust.VExpr.var a_3 → False` |
| 302 | `Polyrust.instDecidableEqVExpr.decEq._proof_28` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 : Polyrust.VExpr) (a_3 : Int), a.ite a_1 a_2 = Polyrust.VExpr.num a_3 → False` |
| 303 | `Polyrust.instDecidableEqVExpr.decEq._proof_29` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.ite a_1 a_2 = a_3.add a_4 → False` |
| 304 | `Polyrust.instDecidableEqVExpr.decEq._proof_3` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Nat) (a_1 : Int), Polyrust.VExpr.var a = Polyrust.VExpr.num a_1 → False` |
| 305 | `Polyrust.instDecidableEqVExpr.decEq._proof_30` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 a_4 : Polyrust.VExpr), a.ite a_1 a_2 = a_3.eqb a_4 → False` |
| 306 | `Polyrust.instDecidableEqVExpr.decEq._proof_31` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 : Polyrust.VExpr), a.ite a_1 a_2 = a.ite a_1 a_2` |
| 307 | `Polyrust.instDecidableEqVExpr.decEq._proof_32` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 b : Polyrust.VExpr), ¬a_2 = b → a.ite a_1 a_2 = a.ite a_1 b → False` |
| 308 | `Polyrust.instDecidableEqVExpr.decEq._proof_33` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 b b_1 : Polyrust.VExpr), ¬a_1 = b → a.ite a_1 a_2 = a.ite b b_1 → False` |
| 309 | `Polyrust.instDecidableEqVExpr.decEq._proof_34` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 b b_1 b_2 : Polyrust.VExpr), ¬a = b → a.ite a_1 a_2 = b.ite b_1 b_2 → False` |
| 310 | `Polyrust.instDecidableEqVExpr.decEq._proof_4` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Nat) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.var a = a_1.add a_2 → False` |
| 311 | `Polyrust.instDecidableEqVExpr.decEq._proof_5` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Nat) (a_1 a_2 : Polyrust.VExpr), Polyrust.VExpr.var a = a_1.eqb a_2 → False` |
| 312 | `Polyrust.instDecidableEqVExpr.decEq._proof_6` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Nat) (a_1 a_2 a_3 : Polyrust.VExpr), Polyrust.VExpr.var a = a_1.ite a_2 a_3 → False` |
| 313 | `Polyrust.instDecidableEqVExpr.decEq._proof_7` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int) (a_1 : Nat), Polyrust.VExpr.num a = Polyrust.VExpr.var a_1 → False` |
| 314 | `Polyrust.instDecidableEqVExpr.decEq._proof_8` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a : Int), Polyrust.VExpr.num a = Polyrust.VExpr.num a` |
| 315 | `Polyrust.instDecidableEqVExpr.decEq._proof_9` | 實例衍生 | 定理 | `VExpr` 相等性判定的內部子證明（機器衍生） | `∀ (a b : Int), ¬a = b → Polyrust.VExpr.num a = Polyrust.VExpr.num b → False` |
| 316 | `Polyrust.instDecidableEqVExpr.decEq._sunfold` | 實例衍生 | 定義 | `VExpr` 相等性判定的展開引理（機器衍生） | `(x x_1 : Polyrust.VExpr) → Decidable (x = x_1)` |
| 317 | `Polyrust.instDecidableEqVExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | `VExpr` 相等性判定的遞迴助手（機器衍生） | `(x x_1 : Polyrust.VExpr) → Decidable (x = x_1)` |
| 318 | `Polyrust.instDecidableEqVExpr.decEq.match_1` | 實例衍生 | 定義 | `VExpr` 相等性判定的輔助匹配（機器衍生） | `(motive : Polyrust.VExpr → Polyrust.VExpr → Sort u_1) →   (x x_1 : Polyrust.VExpr) →     ((a b : …` |
| 319 | `Polyrust.instReprVExpr.repr.match_1` | 實例衍生 | 定義 | `VExpr` 顯示函數的機器衍生助手 | `(motive : Polyrust.VExpr → Sort u_1) →   (x : Polyrust.VExpr) →     ((a : Nat) → motive (Polyrust…` |
| 320 | `Polyrust.VExpr.brecOn.eq` | 衍生 | 定理 | `VExpr` 相關助手（機器衍生） | `∀ {motive : Polyrust.VExpr → Sort u} (t : Polyrust.VExpr)   (F_1 : (t : Polyrust.VExpr) → Polyrus…` |
| 321 | `Polyrust.VExpr.ctorElimType` | 衍生 | 定義 | `VExpr` 相關助手（機器衍生） | `{motive : Polyrust.VExpr → Sort u} → Nat → Sort (max 1 u)` |
| 322 | `Polyrust.VExpr.noConfusionType` | 衍生 | 定義 | `VExpr` 相關助手（機器衍生） | `Sort u → Polyrust.VExpr → Polyrust.VExpr → Sort u` |

## `Polyrust.OpAbstraction`（107 條）

職責：運算子抽象層

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 323 | `Polyrust.IsRootG2` | 手寫 | 定義 | 根：滿足全部約束的位元賦值。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Polyrust.SigmaG2 Ty → Prop` |
| 324 | `Polyrust.SigmaG2` | 手寫 | 定義 | ! ## 三、約束編碼（運算子規格參數化） 位元賦值：每個節點、每個型別一個 0/1 位元。 | `Type → Type` |
| 325 | `Polyrust.TypableG2` | 手寫 | 定義 | 可定型（泛化運算子）。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Prop` |
| 326 | `Polyrust.andSpec` | 手寫 | 定義 | 布爾類運算子（And）：`eqbTy × eqbTy → eqbTy`。 | `Polyrust.Lang Polyrust.Ty → Polyrust.BinSpec Polyrust.Ty` |
| 327 | `Polyrust.arithSpec` | 手寫 | 定義 | 算術類運算子（Add/Sub/Mul）：`numTy × numTy → numTy`。 | `Polyrust.Lang Polyrust.Ty → Polyrust.BinSpec Polyrust.Ty` |
| 328 | `Polyrust.cBinop` | 手寫 | 定義 | `binop s a b` 節點的規則方程（逐型別）： 位元 = 輸出標記 · a 在 s.in1 的位元 · b 在 s.in2 的位元（類比 `cAdd` 的 `numMark`）。 | `{Ty : Type} →   [DecidableEq Ty] → Polyrust.BinSpec Ty → Polyrust.ExprG Ty → Polyrust.ExprG Ty → …` |
| 329 | `Polyrust.cIte2` | 手寫 | 定義 | `ite` 節點的規則方程（逐型別）。 | `{Ty : Type} →   Polyrust.Lang Ty → Polyrust.ExprG Ty → Polyrust.ExprG Ty → Polyrust.ExprG Ty → Ty…` |
| 330 | `Polyrust.cNum2` | 手寫 | 定義 | `num` 節點的規則方程（逐型別）：位元 = num 標記。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Int → Ty → Polyrust.SigmaG2 Ty → Int` |
| 331 | `Polyrust.cmpSpec` | 手寫 | 定義 | 比較類運算子（Lt/Le/Ge/Eq/Ne）：`numTy × numTy → eqbTy`。 | `Polyrust.Lang Polyrust.Ty → Polyrust.BinSpec Polyrust.Ty` |
| 332 | `Polyrust.genCG2` | 手寫 | 定義 | 約束生成（泛化運算子）：每個節點的規則方程 + one-hot。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → List (Polyrust.SigmaG2 Ty…` |
| 333 | `Polyrust.oneHotC2` | 手寫 | 定義 | one-hot 約束（以節點為參數的形式）。 | `{Ty : Type} → Polyrust.Lang Ty → Polyrust.ExprG Ty → Polyrust.SigmaG2 Ty → Int` |
| 334 | `Polyrust.oneHotG2` | 手寫 | 定義 | **one-hot（泛化運算子）**：`(Σ_{t ∈ enumAll} bit(σ e t)) − 1 = 0`。 | `{Ty : Type} → Polyrust.Lang Ty → Polyrust.SigmaG2 Ty → Polyrust.ExprG Ty → Int` |
| 335 | `Polyrust.outMark` | 手寫 | 定義 | 規格 `s` 的輸出標記（對偶於 `T9Generalized.numMark`/`eqbMark`）。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.BinSpec Ty → Ty → Int` |
| 336 | `Polyrust.tbG2` | 手寫 | 定義 | 節點 e 在型別 τ 上的位元，作為 ℤ 值。 | `{Ty : Type} → Polyrust.SigmaG2 Ty → Polyrust.ExprG Ty → Ty → Int` |
| 337 | `Polyrust.tycheckG` | 手寫 | 定義 | ! ## 二、泛化檢查器、單型性 泛化檢查器：`binop s a b` 的輸出型別必須是 `s.out`，且 `a : s.in1`、`b : s.in2`。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Ty → Bool` |
| 338 | `Polyrust.witnessG2` | 手寫 | 定義 | 見證賦值：直接取檢查器的答案。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SigmaG2 Ty` |
| 339 | `Polyrust.ExprG.brecOn.go` | 等式引理／助手 | 定義 | `ExprG` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       ((t :…` |
| 340 | `Polyrust.genCG2._f` | 等式引理／助手 | 定義 | `genCG2` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty → (x : Polyrust.ExprG Ty) → Polyrust.Expr…` |
| 341 | `Polyrust.genCG2._sunfold` | 等式引理／助手 | 定義 | `genCG2` 的結構遞迴展開引理 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → List (Polyrust.SigmaG2 Ty…` |
| 342 | `Polyrust.genCG2._unsafe_rec` | 等式引理／助手 | 定義 | `genCG2` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → List (Polyrust.SigmaG2 Ty…` |
| 343 | `Polyrust.genCG2.eq_def` | 等式引理／助手 | 定理 | 定義 `genCG2` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.ExprG Ty),   Polyrust.…` |
| 344 | `Polyrust.isMonoAtG2_of_root.match_1_3` | 等式引理／助手 | 定義 | `isMonoAtG2_of_root` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} {e : Polyrust.ExprG Ty} {σ : Polyrust.SigmaG2 Ty} (τ τ' : Ty)   (motive : σ e τ = t…` |
| 345 | `Polyrust.oneHotC2.eq_1` | 等式引理／助手 | 定理 | 定義 `oneHotC2` 的等式引理（定義的展開方程） | `∀ {Ty : Type} (L : Polyrust.Lang Ty) (e : Polyrust.ExprG Ty) (σ : Polyrust.SigmaG2 Ty),   Polyrus…` |
| 346 | `Polyrust.outMark.eq_1` | 等式引理／助手 | 定理 | 定義 `outMark` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (s : Polyrust.BinSpec Ty) (t : Ty),   Polyrust.outMark s t …` |
| 347 | `Polyrust.tycheckG._f` | 等式引理／助手 | 定義 | `tycheckG` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty → (x : Polyrust.ExprG Ty) → Polyrust.Expr…` |
| 348 | `Polyrust.tycheckG._sunfold` | 等式引理／助手 | 定義 | `tycheckG` 的結構遞迴展開引理 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Ty → Bool` |
| 349 | `Polyrust.tycheckG._unsafe_rec` | 等式引理／助手 | 定義 | `tycheckG` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ExprG Ty → Ty → Bool` |
| 350 | `Polyrust.tycheckG.eq_1` | 等式引理／助手 | 定理 | 定義 `tycheckG` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a : Int),   Polyrust.tyche…` |
| 351 | `Polyrust.tycheckG.eq_2` | 等式引理／助手 | 定理 | 定義 `tycheckG` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (s : Polyrust.BinSpec Ty)  …` |
| 352 | `Polyrust.tycheckG.eq_3` | 等式引理／助手 | 定理 | 定義 `tycheckG` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (c t f : Polyrust.ExprG Ty)…` |
| 353 | `Polyrust.tycheckG.eq_def` | 等式引理／助手 | 定理 | 定義 `tycheckG` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.ExprG Ty) (x_1 : Ty), …` |
| 354 | `Polyrust.tycheckG.match_1` | 等式引理／助手 | 定義 | `tycheckG` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   (motive : Polyrust.ExprG Ty → Ty → Sort u_1) →     (x : Polyrust.ExprG Ty) →     …` |
| 355 | `Polyrust.tycheckG_exclusive.match_1_1` | 等式引理／助手 | 定義 | `tycheckG_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (n : Int) (τ τ' : Ty)   (motive :   …` |
| 356 | `Polyrust.tycheckG_exclusive.match_1_3` | 等式引理／助手 | 定義 | `tycheckG_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (s : Polyrust.BinSpec Ty) (a b : Pol…` |
| 357 | `Polyrust.tycheckG_exclusive.match_1_5` | 等式引理／助手 | 定義 | `tycheckG_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (c t f : Polyrust.ExprG Ty) (τ τ' : …` |
| 358 | `Polyrust.typable_iff_rootG2.match_1_1` | 等式引理／助手 | 定義 | `typable_iff_rootG2` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.ExprG Ty)   (motive : …` |
| 359 | `Polyrust.untypable_iff_no_rootG2.match_1_1` | 等式引理／助手 | 定義 | `untypable_iff_no_rootG2` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.ExprG Ty)   (motive : …` |
| 360 | `Polyrust.BinSpec._sizeOf_1` | 歸納型衍生 | 定義 | `BinSpec` 的輔助大小函數 | `{Ty : Type} → [SizeOf Ty] → Polyrust.BinSpec Ty → Nat` |
| 361 | `Polyrust.BinSpec._sizeOf_inst` | 歸納型衍生 | 定義 | `BinSpec` 的輔助大小函數 | `(Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.BinSpec Ty)` |
| 362 | `Polyrust.BinSpec.casesOn` | 歸納型衍生 | 定義 | `BinSpec` 的案例分析原則 | `{Ty : Type} →   {motive : Polyrust.BinSpec Ty → Sort u} →     (t : Polyrust.BinSpec Ty) → ((in1 i…` |
| 363 | `Polyrust.BinSpec.ctorIdx` | 歸納型衍生 | 定義 | `BinSpec` 的構造子結構助手（機器衍生） | `{Ty : Type} → Polyrust.BinSpec Ty → Nat` |
| 364 | `Polyrust.BinSpec.mk._flat_ctor` | 歸納型衍生 | 定義 | `BinSpec` 的構造子結構助手（機器衍生） | `{Ty : Type} → Ty → Ty → Ty → Polyrust.BinSpec Ty` |
| 365 | `Polyrust.BinSpec.mk.inj` | 歸納型衍生 | 定理 | `BinSpec` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {in1 in2 out in1_1 in2_1 out_1 : Ty},   { in1 := in1, in2 := in2, out := out } = { …` |
| 366 | `Polyrust.BinSpec.mk.noConfusion` | 歸納型衍生 | 定義 | `BinSpec` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} →   {P : Sort u} →     {in1 in2 out in1' in2' out' : Ty} →       { in1 := in1, in2 :=…` |
| 367 | `Polyrust.BinSpec.mk.sizeOf_spec` | 歸納型衍生 | 定理 | `BinSpec` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (in1 in2 out : Ty),   sizeOf { in1 := in1, in2 := in2, out := ou…` |
| 368 | `Polyrust.BinSpec.noConfusion` | 歸納型衍生 | 定義 | `BinSpec` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Ty : Type} →     {t : Polyrust.BinSpec Ty} →       {Ty' : Type} → {t' : Polyrus…` |
| 369 | `Polyrust.BinSpec.recOn` | 歸納型衍生 | 定義 | `BinSpec` 的結構遞迴原則 | `{Ty : Type} →   {motive : Polyrust.BinSpec Ty → Sort u} →     (t : Polyrust.BinSpec Ty) → ((in1 i…` |
| 370 | `Polyrust.ExprG._sizeOf_1` | 歸納型衍生 | 定義 | `ExprG` 的輔助大小函數 | `{Ty : Type} → [SizeOf Ty] → Polyrust.ExprG Ty → Nat` |
| 371 | `Polyrust.ExprG._sizeOf_inst` | 歸納型衍生 | 定義 | `ExprG` 的輔助大小函數 | `(Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.ExprG Ty)` |
| 372 | `Polyrust.ExprG.below` | 歸納型衍生 | 定義 | `ExprG` 的良基遞迴助手 | `{Ty : Type} → {motive : Polyrust.ExprG Ty → Sort u} → Polyrust.ExprG Ty → Sort (max 1 u)` |
| 373 | `Polyrust.ExprG.binop.elim` | 歸納型衍生 | 定義 | `ExprG` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       t.cto…` |
| 374 | `Polyrust.ExprG.binop.inj` | 歸納型衍生 | 定理 | `ExprG` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a : Polyrust.BinSpec Ty} {a_1 a_2 : Polyrust.ExprG Ty} {a_3 : Polyrust.BinSpec Ty}…` |
| 375 | `Polyrust.ExprG.binop.noConfusion` | 歸納型衍生 | 定義 | `ExprG` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} →   {P : Sort u} →     {a : Polyrust.BinSpec Ty} →       {a_1 a_2 : Polyrust.ExprG Ty…` |
| 376 | `Polyrust.ExprG.binop.sizeOf_spec` | 歸納型衍生 | 定理 | `ExprG` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty),   sizeO…` |
| 377 | `Polyrust.ExprG.brecOn` | 歸納型衍生 | 定義 | `ExprG` 的良基遞迴助手 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) → ((t : Polyr…` |
| 378 | `Polyrust.ExprG.casesOn` | 歸納型衍生 | 定義 | `ExprG` 的案例分析原則 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       ((a :…` |
| 379 | `Polyrust.ExprG.ctorElim` | 歸納型衍生 | 定義 | `ExprG` 的構造子結構助手（機器衍生） | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.ExprG…` |
| 380 | `Polyrust.ExprG.ctorIdx` | 歸納型衍生 | 定義 | `ExprG` 的構造子結構助手（機器衍生） | `{Ty : Type} → Polyrust.ExprG Ty → Nat` |
| 381 | `Polyrust.ExprG.ite.elim` | 歸納型衍生 | 定義 | `ExprG` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) → t.ctorIdx =…` |
| 382 | `Polyrust.ExprG.ite.inj` | 歸納型衍生 | 定理 | `ExprG` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a a_1 a_2 a_3 a_4 a_5 : Polyrust.ExprG Ty},   a.ite a_1 a_2 = a_3.ite a_4 a_5 → a …` |
| 383 | `Polyrust.ExprG.ite.noConfusion` | 歸納型衍生 | 定義 | `ExprG` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} →   {P : Sort u} →     {a a_1 a_2 a' a'_1 a'_2 : Polyrust.ExprG Ty} →       a.ite a_1…` |
| 384 | `Polyrust.ExprG.ite.sizeOf_spec` | 歸納型衍生 | 定理 | `ExprG` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 a_2 : Polyrust.ExprG Ty),   sizeOf (a.ite a_1 a_2) = 1 + …` |
| 385 | `Polyrust.ExprG.noConfusion` | 歸納型衍生 | 定義 | `ExprG` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Ty : Type} →     {t : Polyrust.ExprG Ty} →       {Ty' : Type} → {t' : Polyrust.…` |
| 386 | `Polyrust.ExprG.num.elim` | 歸納型衍生 | 定義 | `ExprG` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) → t.ctorIdx =…` |
| 387 | `Polyrust.ExprG.num.inj` | 歸納型衍生 | 定理 | `ExprG` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a a_1 : Int}, Polyrust.ExprG.num a = Polyrust.ExprG.num a_1 → a = a_1` |
| 388 | `Polyrust.ExprG.num.noConfusion` | 歸納型衍生 | 定義 | `ExprG` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} → {P : Sort u} → {a a' : Int} → Polyrust.ExprG.num a = Polyrust.ExprG.num a' → (a = a…` |
| 389 | `Polyrust.ExprG.num.sizeOf_spec` | 歸納型衍生 | 定理 | `ExprG` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a : Int), sizeOf (Polyrust.ExprG.num a) = 1 + sizeOf a` |
| 390 | `Polyrust.ExprG.recOn` | 歸納型衍生 | 定義 | `ExprG` 的結構遞迴原則 | `{Ty : Type} →   {motive : Polyrust.ExprG Ty → Sort u} →     (t : Polyrust.ExprG Ty) →       ((a :…` |
| 391 | `Polyrust.instDecidableEqBinSpec` | 實例衍生 | 定義 | `BinSpec` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.BinSpec Ty)` |
| 392 | `Polyrust.instDecidableEqBinSpec.decEq` | 實例衍生 | 定義 | `BinSpec` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.BinSpec Ty) → Decidable (x = x_1)` |
| 393 | `Polyrust.instDecidableEqBinSpec.decEq._proof_1` | 實例衍生 | 定理 | `BinSpec` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 : Ty), { in1 := a, in2 := a_1, out := a_2 } = { in1 := a, in2 := a_1, ou…` |
| 394 | `Polyrust.instDecidableEqBinSpec.decEq._proof_2` | 實例衍生 | 定理 | `BinSpec` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 b : Ty),   ¬a_2 = b → { in1 := a, in2 := a_1, out := a_2 } = { in1 := a,…` |
| 395 | `Polyrust.instDecidableEqBinSpec.decEq._proof_3` | 實例衍生 | 定理 | `BinSpec` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 b b_1 : Ty),   ¬a_1 = b → { in1 := a, in2 := a_1, out := a_2 } = { in1 :…` |
| 396 | `Polyrust.instDecidableEqBinSpec.decEq._proof_4` | 實例衍生 | 定理 | `BinSpec` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 b b_1 b_2 : Ty),   ¬a = b → { in1 := a, in2 := a_1, out := a_2 } = { in1…` |
| 397 | `Polyrust.instDecidableEqBinSpec.decEq.match_1` | 實例衍生 | 定義 | `BinSpec` 相等性判定的輔助匹配（機器衍生） | `{Ty : Type} →   (motive : Polyrust.BinSpec Ty → Polyrust.BinSpec Ty → Sort u_1) →     (x x_1 : Po…` |
| 398 | `Polyrust.instDecidableEqExprG` | 實例衍生 | 定義 | `ExprG` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.ExprG Ty)` |
| 399 | `Polyrust.instDecidableEqExprG.decEq` | 實例衍生 | 定義 | `ExprG` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ExprG Ty) → Decidable (x = x_1)` |
| 400 | `Polyrust.instDecidableEqExprG.decEq._f` | 實例衍生 | 定義 | `ExprG` 相等性判定的遞迴助手（機器衍生） | `{Ty : Type} →   [DecidableEq Ty] →     (x : Polyrust.ExprG Ty) →       Polyrust.ExprG.below (moti…` |
| 401 | `Polyrust.instDecidableEqExprG.decEq._proof_1` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Int), Polyrust.ExprG.num a = Polyrust.ExprG.num a` |
| 402 | `Polyrust.instDecidableEqExprG.decEq._proof_10` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 a_3 a_4 a_5 : Polyrust.ExprG Ty),   Polyrust.Exp…` |
| 403 | `Polyrust.instDecidableEqExprG.decEq._proof_11` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 : Polyrust.ExprG Ty) (a_3 : Int), a.ite a_1 a_2 = Polyrust.ExprG.num a_3…` |
| 404 | `Polyrust.instDecidableEqExprG.decEq._proof_12` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 : Polyrust.ExprG Ty) (a_3 : Polyrust.BinSpec Ty) (a_4 a_5 : Polyrust.Exp…` |
| 405 | `Polyrust.instDecidableEqExprG.decEq._proof_13` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 : Polyrust.ExprG Ty), a.ite a_1 a_2 = a.ite a_1 a_2` |
| 406 | `Polyrust.instDecidableEqExprG.decEq._proof_14` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 b : Polyrust.ExprG Ty), ¬a_2 = b → a.ite a_1 a_2 = a.ite a_1 b → False` |
| 407 | `Polyrust.instDecidableEqExprG.decEq._proof_15` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 b b_1 : Polyrust.ExprG Ty), ¬a_1 = b → a.ite a_1 a_2 = a.ite b b_1 → False` |
| 408 | `Polyrust.instDecidableEqExprG.decEq._proof_16` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 a_2 b b_1 b_2 : Polyrust.ExprG Ty), ¬a = b → a.ite a_1 a_2 = b.ite b_1 b_2 →…` |
| 409 | `Polyrust.instDecidableEqExprG.decEq._proof_2` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a b : Int), ¬a = b → Polyrust.ExprG.num a = Polyrust.ExprG.num b → False` |
| 410 | `Polyrust.instDecidableEqExprG.decEq._proof_3` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Int) (a_1 : Polyrust.BinSpec Ty) (a_2 a_3 : Polyrust.ExprG Ty),   Polyrust.Exp…` |
| 411 | `Polyrust.instDecidableEqExprG.decEq._proof_4` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Int) (a_1 a_2 a_3 : Polyrust.ExprG Ty), Polyrust.ExprG.num a = a_1.ite a_2 a_3…` |
| 412 | `Polyrust.instDecidableEqExprG.decEq._proof_5` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty) (a_3 : Int),   Polyrust.Exp…` |
| 413 | `Polyrust.instDecidableEqExprG.decEq._proof_6` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty),   Polyrust.ExprG.binop a a…` |
| 414 | `Polyrust.instDecidableEqExprG.decEq._proof_7` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 b : Polyrust.ExprG Ty),   ¬a_2 = b → Polyrust.Ex…` |
| 415 | `Polyrust.instDecidableEqExprG.decEq._proof_8` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 b b_1 : Polyrust.ExprG Ty),   ¬a_1 = b → Polyrus…` |
| 416 | `Polyrust.instDecidableEqExprG.decEq._proof_9` | 實例衍生 | 定理 | `ExprG` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Polyrust.BinSpec Ty) (a_1 a_2 : Polyrust.ExprG Ty) (b : Polyrust.BinSpec Ty)  …` |
| 417 | `Polyrust.instDecidableEqExprG.decEq._sunfold` | 實例衍生 | 定義 | `ExprG` 相等性判定的展開引理（機器衍生） | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ExprG Ty) → Decidable (x = x_1)` |
| 418 | `Polyrust.instDecidableEqExprG.decEq._unsafe_rec` | 實例衍生 | 定義 | `ExprG` 相等性判定的遞迴助手（機器衍生） | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ExprG Ty) → Decidable (x = x_1)` |
| 419 | `Polyrust.instDecidableEqExprG.decEq.match_1` | 實例衍生 | 定義 | `ExprG` 相等性判定的輔助匹配（機器衍生） | `{Ty : Type} →   (motive : Polyrust.ExprG Ty → Polyrust.ExprG Ty → Sort u_1) →     (x x_1 : Polyru…` |
| 420 | `Polyrust.instReprBinSpec` | 實例衍生 | 定義 | `BinSpec` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Repr (Polyrust.BinSpec Ty)` |
| 421 | `Polyrust.instReprBinSpec.repr` | 實例衍生 | 定義 | `BinSpec` 的顯示函數（Repr 型別類別實例） | `{Ty : Type} → [Repr Ty] → Polyrust.BinSpec Ty → Nat → Format` |
| 422 | `Polyrust.instReprExprG.repr.match_1` | 實例衍生 | 定義 | `ExprG` 顯示函數的機器衍生助手 | `{Ty : Type} →   (motive : Polyrust.ExprG Ty → Sort u_1) →     (x : Polyrust.ExprG Ty) →       ((a…` |
| 423 | `Polyrust.BinSpec.in1` | 衍生 | 定義 | `BinSpec` 相關助手（機器衍生） | `{Ty : Type} → Polyrust.BinSpec Ty → Ty` |
| 424 | `Polyrust.BinSpec.in2` | 衍生 | 定義 | `BinSpec` 相關助手（機器衍生） | `{Ty : Type} → Polyrust.BinSpec Ty → Ty` |
| 425 | `Polyrust.BinSpec.noConfusionType` | 衍生 | 定義 | `BinSpec` 相關助手（機器衍生） | `Sort u → {Ty : Type} → Polyrust.BinSpec Ty → {Ty' : Type} → Polyrust.BinSpec Ty' → Sort u` |
| 426 | `Polyrust.BinSpec.out` | 衍生 | 定義 | `BinSpec` 相關助手（機器衍生） | `{Ty : Type} → Polyrust.BinSpec Ty → Ty` |
| 427 | `Polyrust.ExprG.brecOn.eq` | 衍生 | 定理 | `ExprG` 相關助手（機器衍生） | `∀ {Ty : Type} {motive : Polyrust.ExprG Ty → Sort u} (t : Polyrust.ExprG Ty)   (F_1 : (t : Polyrus…` |
| 428 | `Polyrust.ExprG.ctorElimType` | 衍生 | 定義 | `ExprG` 相關助手（機器衍生） | `{Ty : Type} → {motive : Polyrust.ExprG Ty → Sort u} → Nat → Sort (max 1 u)` |
| 429 | `Polyrust.ExprG.noConfusionType` | 衍生 | 定義 | `ExprG` 相關助手（機器衍生） | `Sort u → {Ty : Type} → Polyrust.ExprG Ty → {Ty' : Type} → Polyrust.ExprG Ty' → Sort u` |

## `Polyrust.SumReduction`（106 條）

職責：求和項歸約

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 430 | `Polyrust.TypableSum` | 手寫 | 定義 | 和型可定型：存在某個和型使檢查通過。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Prop` |
| 431 | `Polyrust.TypeTypable` | 手寫 | 定義 | 型別可定型：存在某個表達式被檢查為該型別。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumTy Ty → Prop` |
| 432 | `Polyrust.sumBits` | 手寫 | 定義 | 和型用 `++` 做不相交**和**。 | `{Ty : Type} → Polyrust.Lang Ty → (Ty → Bool) → (Ty → Bool) → List Int` |
| 433 | `Polyrust.tycheckSum` | 手寫 | 定義 | ! ## 二、和型檢查器與檢查歸約 和型檢查器：`lift e` 按基本語言的 `tycheck` 檢查，`inl a` 按 `sum τ₁ τ₂` **只檢查左變體** `a : τ₁`，`inr b` **只檢查右變體** `b : τ | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Polyrust.Sum…` |
| 434 | `Polyrust.tycheck_inl` | 手寫 | 定理 | **檢查歸約（T-c-inl）**：`inl a` 對 `sum τ₁ τ₂` 的檢查就是 `a` 對 `τ₁` 的檢查。 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {a : Polyrust.SumExpr Polyrust.Expr}…` |
| 435 | `Polyrust.tycheck_inr` | 手寫 | 定理 | **檢查歸約（T-c-inr）**：`inr b` 對 `sum τ₁ τ₂` 的檢查就是 `b` 對 `τ₂` 的檢查。 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {b : Polyrust.SumExpr Polyrust.Expr}…` |
| 436 | `Polyrust.SumExpr.brecOn.go` | 等式引理／助手 | 定義 | `SumExpr` 定義編譯時產生的遞迴／匹配助手 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) →  …` |
| 437 | `Polyrust.SumTy.brecOn.go` | 等式引理／助手 | 定義 | `SumTy` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) →       ((t :…` |
| 438 | `Polyrust.tycheckSum._f` | 等式引理／助手 | 定義 | `tycheckSum` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty →       (x : Polyrust.SumExpr Polyrust.Ex…` |
| 439 | `Polyrust.tycheckSum._sunfold` | 等式引理／助手 | 定義 | `tycheckSum` 的結構遞迴展開引理 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Polyrust.Sum…` |
| 440 | `Polyrust.tycheckSum._unsafe_rec` | 等式引理／助手 | 定義 | `tycheckSum` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SumExpr Polyrust.Expr → Polyrust.Sum…` |
| 441 | `Polyrust.tycheckSum.eq_1` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (τ : Ty),   Poly…` |
| 442 | `Polyrust.tycheckSum.eq_2` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.Expr) (a_1 a_2 : Polyr…` |
| 443 | `Polyrust.tycheckSum.eq_3` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.SumExpr Polyrust.Expr)…` |
| 444 | `Polyrust.tycheckSum.eq_4` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.SumExpr Polyrust.Expr)…` |
| 445 | `Polyrust.tycheckSum.eq_5` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (b : Polyrust.SumExpr Polyrust.Expr)…` |
| 446 | `Polyrust.tycheckSum.eq_6` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.SumExpr Polyrust.Expr)…` |
| 447 | `Polyrust.tycheckSum.eq_def` | 等式引理／助手 | 定理 | 定義 `tycheckSum` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.SumExpr Polyrust.Expr)…` |
| 448 | `Polyrust.tycheckSum.match_1` | 等式引理／助手 | 定義 | `tycheckSum` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   (motive : Polyrust.SumExpr Polyrust.Expr → Polyrust.SumTy Ty → Sort u_1) →     (x…` |
| 449 | `Polyrust.SumExpr._sizeOf_1` | 歸納型衍生 | 定義 | `SumExpr` 的輔助大小函數 | `{Expr : Type} → [SizeOf Expr] → Polyrust.SumExpr Expr → Nat` |
| 450 | `Polyrust.SumExpr._sizeOf_inst` | 歸納型衍生 | 定義 | `SumExpr` 的輔助大小函數 | `(Expr : Type) → [SizeOf Expr] → SizeOf (Polyrust.SumExpr Expr)` |
| 451 | `Polyrust.SumExpr.below` | 歸納型衍生 | 定義 | `SumExpr` 的良基遞迴助手 | `{Expr : Type} → {motive : Polyrust.SumExpr Expr → Sort u} → Polyrust.SumExpr Expr → Sort (max 1 u)` |
| 452 | `Polyrust.SumExpr.brecOn` | 歸納型衍生 | 定義 | `SumExpr` 的良基遞迴助手 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → (…` |
| 453 | `Polyrust.SumExpr.casesOn` | 歸納型衍生 | 定義 | `SumExpr` 的案例分析原則 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) →  …` |
| 454 | `Polyrust.SumExpr.ctorElim` | 歸納型衍生 | 定義 | `SumExpr` 的構造子結構助手（機器衍生） | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (ctorIdx : Nat) →       (t : Po…` |
| 455 | `Polyrust.SumExpr.ctorIdx` | 歸納型衍生 | 定義 | `SumExpr` 的構造子結構助手（機器衍生） | `{Expr : Type} → Polyrust.SumExpr Expr → Nat` |
| 456 | `Polyrust.SumExpr.inl.elim` | 歸納型衍生 | 定義 | `SumExpr` 的構造子消去器 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → t…` |
| 457 | `Polyrust.SumExpr.inl.inj` | 歸納型衍生 | 定理 | `SumExpr` 的構造子結構助手（機器衍生） | `∀ {Expr : Type} {a a_1 : Polyrust.SumExpr Expr}, a.inl = a_1.inl → a = a_1` |
| 458 | `Polyrust.SumExpr.inl.noConfusion` | 歸納型衍生 | 定義 | `SumExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Expr : Type} → {P : Sort u} → {a a' : Polyrust.SumExpr Expr} → a.inl = a'.inl → (a ≍ a' → P) → P` |
| 459 | `Polyrust.SumExpr.inl.sizeOf_spec` | 歸納型衍生 | 定理 | `SumExpr` 結構大小的規格命題 | `∀ {Expr : Type} [inst : SizeOf Expr] (a : Polyrust.SumExpr Expr), sizeOf a.inl = 1 + sizeOf a` |
| 460 | `Polyrust.SumExpr.inr.elim` | 歸納型衍生 | 定義 | `SumExpr` 的構造子消去器 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → t…` |
| 461 | `Polyrust.SumExpr.inr.inj` | 歸納型衍生 | 定理 | `SumExpr` 的構造子結構助手（機器衍生） | `∀ {Expr : Type} {a a_1 : Polyrust.SumExpr Expr}, a.inr = a_1.inr → a = a_1` |
| 462 | `Polyrust.SumExpr.inr.noConfusion` | 歸納型衍生 | 定義 | `SumExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Expr : Type} → {P : Sort u} → {a a' : Polyrust.SumExpr Expr} → a.inr = a'.inr → (a ≍ a' → P) → P` |
| 463 | `Polyrust.SumExpr.inr.sizeOf_spec` | 歸納型衍生 | 定理 | `SumExpr` 結構大小的規格命題 | `∀ {Expr : Type} [inst : SizeOf Expr] (a : Polyrust.SumExpr Expr), sizeOf a.inr = 1 + sizeOf a` |
| 464 | `Polyrust.SumExpr.lift.elim` | 歸納型衍生 | 定義 | `SumExpr` 的構造子消去器 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) → t…` |
| 465 | `Polyrust.SumExpr.lift.inj` | 歸納型衍生 | 定理 | `SumExpr` 的構造子結構助手（機器衍生） | `∀ {Expr : Type} {a a_1 : Expr}, Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift a_1 → a = a_1` |
| 466 | `Polyrust.SumExpr.lift.noConfusion` | 歸納型衍生 | 定義 | `SumExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Expr : Type} → {P : Sort u} → {a a' : Expr} → Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift a'…` |
| 467 | `Polyrust.SumExpr.lift.sizeOf_spec` | 歸納型衍生 | 定理 | `SumExpr` 結構大小的規格命題 | `∀ {Expr : Type} [inst : SizeOf Expr] (a : Expr), sizeOf (Polyrust.SumExpr.lift a) = 1 + sizeOf a` |
| 468 | `Polyrust.SumExpr.noConfusion` | 歸納型衍生 | 定義 | `SumExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Expr : Type} →     {t : Polyrust.SumExpr Expr} →       {Expr' : Type} → {t' : P…` |
| 469 | `Polyrust.SumExpr.recOn` | 歸納型衍生 | 定義 | `SumExpr` 的結構遞迴原則 | `{Expr : Type} →   {motive : Polyrust.SumExpr Expr → Sort u} →     (t : Polyrust.SumExpr Expr) →  …` |
| 470 | `Polyrust.SumTy._sizeOf_1` | 歸納型衍生 | 定義 | `SumTy` 的輔助大小函數 | `{Ty : Type} → [SizeOf Ty] → Polyrust.SumTy Ty → Nat` |
| 471 | `Polyrust.SumTy._sizeOf_inst` | 歸納型衍生 | 定義 | `SumTy` 的輔助大小函數 | `(Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.SumTy Ty)` |
| 472 | `Polyrust.SumTy.base.elim` | 歸納型衍生 | 定義 | `SumTy` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) → t.ctorIdx =…` |
| 473 | `Polyrust.SumTy.base.inj` | 歸納型衍生 | 定理 | `SumTy` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a a_1 : Ty}, Polyrust.SumTy.base a = Polyrust.SumTy.base a_1 → a = a_1` |
| 474 | `Polyrust.SumTy.base.noConfusion` | 歸納型衍生 | 定義 | `SumTy` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} → {P : Sort u} → {a a' : Ty} → Polyrust.SumTy.base a = Polyrust.SumTy.base a' → (a ≍ …` |
| 475 | `Polyrust.SumTy.base.sizeOf_spec` | 歸納型衍生 | 定理 | `SumTy` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a : Ty), sizeOf (Polyrust.SumTy.base a) = 1 + sizeOf a` |
| 476 | `Polyrust.SumTy.below` | 歸納型衍生 | 定義 | `SumTy` 的良基遞迴助手 | `{Ty : Type} → {motive : Polyrust.SumTy Ty → Sort u} → Polyrust.SumTy Ty → Sort (max 1 u)` |
| 477 | `Polyrust.SumTy.brecOn` | 歸納型衍生 | 定義 | `SumTy` 的良基遞迴助手 | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) → ((t : Polyr…` |
| 478 | `Polyrust.SumTy.casesOn` | 歸納型衍生 | 定義 | `SumTy` 的案例分析原則 | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) →       ((a :…` |
| 479 | `Polyrust.SumTy.ctorElim` | 歸納型衍生 | 定義 | `SumTy` 的構造子結構助手（機器衍生） | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.SumTy…` |
| 480 | `Polyrust.SumTy.ctorIdx` | 歸納型衍生 | 定義 | `SumTy` 的構造子結構助手（機器衍生） | `{Ty : Type} → Polyrust.SumTy Ty → Nat` |
| 481 | `Polyrust.SumTy.noConfusion` | 歸納型衍生 | 定義 | `SumTy` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Ty : Type} →     {t : Polyrust.SumTy Ty} →       {Ty' : Type} → {t' : Polyrust.…` |
| 482 | `Polyrust.SumTy.recOn` | 歸納型衍生 | 定義 | `SumTy` 的結構遞迴原則 | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) →       ((a :…` |
| 483 | `Polyrust.SumTy.sum.elim` | 歸納型衍生 | 定義 | `SumTy` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.SumTy Ty → Sort u} →     (t : Polyrust.SumTy Ty) → t.ctorIdx =…` |
| 484 | `Polyrust.SumTy.sum.inj` | 歸納型衍生 | 定理 | `SumTy` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a a_1 a_2 a_3 : Polyrust.SumTy Ty}, a.sum a_1 = a_2.sum a_3 → a = a_2 ∧ a_1 = a_3` |
| 485 | `Polyrust.SumTy.sum.noConfusion` | 歸納型衍生 | 定義 | `SumTy` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} →   {P : Sort u} → {a a_1 a' a'_1 : Polyrust.SumTy Ty} → a.sum a_1 = a'.sum a'_1 → (a…` |
| 486 | `Polyrust.SumTy.sum.sizeOf_spec` | 歸納型衍生 | 定理 | `SumTy` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 : Polyrust.SumTy Ty), sizeOf (a.sum a_1) = 1 + sizeOf a +…` |
| 487 | `Polyrust.instDecidableEqSumExpr` | 實例衍生 | 定義 | `SumExpr` 可判定相等（DecidableEq）的判定程序 | `{Expr : Type} → [DecidableEq Expr] → DecidableEq (Polyrust.SumExpr Expr)` |
| 488 | `Polyrust.instDecidableEqSumExpr.decEq` | 實例衍生 | 定義 | `SumExpr` 可判定相等（DecidableEq）的判定程序 | `{Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.SumExpr Expr) → Decidable (x = x_1)` |
| 489 | `Polyrust.instDecidableEqSumExpr.decEq._f` | 實例衍生 | 定義 | `SumExpr` 相等性判定的遞迴助手（機器衍生） | `{Expr : Type} →   [DecidableEq Expr] →     (x : Polyrust.SumExpr Expr) →       Polyrust.SumExpr.b…` |
| 490 | `Polyrust.instDecidableEqSumExpr.decEq._proof_1` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Expr), Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift a` |
| 491 | `Polyrust.instDecidableEqSumExpr.decEq._proof_10` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a a_1 : Polyrust.SumExpr Expr), a.inr = a_1.inl → False` |
| 492 | `Polyrust.instDecidableEqSumExpr.decEq._proof_11` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Polyrust.SumExpr Expr), a.inr = a.inr` |
| 493 | `Polyrust.instDecidableEqSumExpr.decEq._proof_12` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a b : Polyrust.SumExpr Expr), ¬a = b → a.inr = b.inr → False` |
| 494 | `Polyrust.instDecidableEqSumExpr.decEq._proof_2` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a b : Expr), ¬a = b → Polyrust.SumExpr.lift a = Polyrust.SumExpr.lift b → False` |
| 495 | `Polyrust.instDecidableEqSumExpr.decEq._proof_3` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Expr) (a_1 : Polyrust.SumExpr Expr), Polyrust.SumExpr.lift a = a_1.inl → False` |
| 496 | `Polyrust.instDecidableEqSumExpr.decEq._proof_4` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Expr) (a_1 : Polyrust.SumExpr Expr), Polyrust.SumExpr.lift a = a_1.inr → False` |
| 497 | `Polyrust.instDecidableEqSumExpr.decEq._proof_5` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Polyrust.SumExpr Expr) (a_1 : Expr), a.inl = Polyrust.SumExpr.lift a_1 → False` |
| 498 | `Polyrust.instDecidableEqSumExpr.decEq._proof_6` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Polyrust.SumExpr Expr), a.inl = a.inl` |
| 499 | `Polyrust.instDecidableEqSumExpr.decEq._proof_7` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a b : Polyrust.SumExpr Expr), ¬a = b → a.inl = b.inl → False` |
| 500 | `Polyrust.instDecidableEqSumExpr.decEq._proof_8` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a a_1 : Polyrust.SumExpr Expr), a.inl = a_1.inr → False` |
| 501 | `Polyrust.instDecidableEqSumExpr.decEq._proof_9` | 實例衍生 | 定理 | `SumExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Polyrust.SumExpr Expr) (a_1 : Expr), a.inr = Polyrust.SumExpr.lift a_1 → False` |
| 502 | `Polyrust.instDecidableEqSumExpr.decEq._sunfold` | 實例衍生 | 定義 | `SumExpr` 相等性判定的展開引理（機器衍生） | `{Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.SumExpr Expr) → Decidable (x = x_1)` |
| 503 | `Polyrust.instDecidableEqSumExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | `SumExpr` 相等性判定的遞迴助手（機器衍生） | `{Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.SumExpr Expr) → Decidable (x = x_1)` |
| 504 | `Polyrust.instDecidableEqSumExpr.decEq.match_1` | 實例衍生 | 定義 | `SumExpr` 相等性判定的輔助匹配（機器衍生） | `{Expr : Type} →   (motive : Polyrust.SumExpr Expr → Polyrust.SumExpr Expr → Sort u_1) →     (x x_…` |
| 505 | `Polyrust.instDecidableEqSumTy` | 實例衍生 | 定義 | `SumTy` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.SumTy Ty)` |
| 506 | `Polyrust.instDecidableEqSumTy.decEq` | 實例衍生 | 定義 | `SumTy` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.SumTy Ty) → Decidable (x = x_1)` |
| 507 | `Polyrust.instDecidableEqSumTy.decEq._f` | 實例衍生 | 定義 | `SumTy` 相等性判定的遞迴助手（機器衍生） | `{Ty : Type} →   [DecidableEq Ty] →     (x : Polyrust.SumTy Ty) →       Polyrust.SumTy.below (moti…` |
| 508 | `Polyrust.instDecidableEqSumTy.decEq._proof_1` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Ty), Polyrust.SumTy.base a = Polyrust.SumTy.base a` |
| 509 | `Polyrust.instDecidableEqSumTy.decEq._proof_2` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a b : Ty), ¬a = b → Polyrust.SumTy.base a = Polyrust.SumTy.base b → False` |
| 510 | `Polyrust.instDecidableEqSumTy.decEq._proof_3` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Ty) (a_1 a_2 : Polyrust.SumTy Ty), Polyrust.SumTy.base a = a_1.sum a_2 → False` |
| 511 | `Polyrust.instDecidableEqSumTy.decEq._proof_4` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 : Polyrust.SumTy Ty) (a_2 : Ty), a.sum a_1 = Polyrust.SumTy.base a_2 → False` |
| 512 | `Polyrust.instDecidableEqSumTy.decEq._proof_5` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 : Polyrust.SumTy Ty), a.sum a_1 = a.sum a_1` |
| 513 | `Polyrust.instDecidableEqSumTy.decEq._proof_6` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 b : Polyrust.SumTy Ty), ¬a_1 = b → a.sum a_1 = a.sum b → False` |
| 514 | `Polyrust.instDecidableEqSumTy.decEq._proof_7` | 實例衍生 | 定理 | `SumTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 b b_1 : Polyrust.SumTy Ty), ¬a = b → a.sum a_1 = b.sum b_1 → False` |
| 515 | `Polyrust.instDecidableEqSumTy.decEq._sunfold` | 實例衍生 | 定義 | `SumTy` 相等性判定的展開引理（機器衍生） | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.SumTy Ty) → Decidable (x = x_1)` |
| 516 | `Polyrust.instDecidableEqSumTy.decEq._unsafe_rec` | 實例衍生 | 定義 | `SumTy` 相等性判定的遞迴助手（機器衍生） | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.SumTy Ty) → Decidable (x = x_1)` |
| 517 | `Polyrust.instDecidableEqSumTy.decEq.match_1` | 實例衍生 | 定義 | `SumTy` 相等性判定的輔助匹配（機器衍生） | `{Ty : Type} →   (motive : Polyrust.SumTy Ty → Polyrust.SumTy Ty → Sort u_1) →     (x x_1 : Polyru…` |
| 518 | `Polyrust.instReprSumExpr` | 實例衍生 | 定義 | `SumExpr` 顯示函數的機器衍生助手 | `{Expr : Type} → [Repr Expr] → Repr (Polyrust.SumExpr Expr)` |
| 519 | `Polyrust.instReprSumExpr.repr` | 實例衍生 | 定義 | `SumExpr` 的顯示函數（Repr 型別類別實例） | `{Expr : Type} → [Repr Expr] → Polyrust.SumExpr Expr → Nat → Format` |
| 520 | `Polyrust.instReprSumExpr.repr._f` | 實例衍生 | 定義 | `SumExpr` 顯示函數的機器衍生助手 | `{Expr : Type} →   [Repr Expr] → (x : Polyrust.SumExpr Expr) → Polyrust.SumExpr.below (motive := f…` |
| 521 | `Polyrust.instReprSumExpr.repr._sunfold` | 實例衍生 | 定義 | `SumExpr` 顯示函數的機器衍生助手 | `{Expr : Type} → [Repr Expr] → Polyrust.SumExpr Expr → Nat → Format` |
| 522 | `Polyrust.instReprSumExpr.repr._unsafe_rec` | 實例衍生 | 定義 | `SumExpr` 顯示函數的機器衍生助手 | `{Expr : Type} → [Repr Expr] → Polyrust.SumExpr Expr → Nat → Format` |
| 523 | `Polyrust.instReprSumExpr.repr.match_1` | 實例衍生 | 定義 | `SumExpr` 顯示函數的機器衍生助手 | `{Expr : Type} →   (motive : Polyrust.SumExpr Expr → Sort u_1) →     (x : Polyrust.SumExpr Expr) →…` |
| 524 | `Polyrust.instReprSumTy` | 實例衍生 | 定義 | `SumTy` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Repr (Polyrust.SumTy Ty)` |
| 525 | `Polyrust.instReprSumTy.repr` | 實例衍生 | 定義 | `SumTy` 的顯示函數（Repr 型別類別實例） | `{Ty : Type} → [Repr Ty] → Polyrust.SumTy Ty → Nat → Format` |
| 526 | `Polyrust.instReprSumTy.repr._f` | 實例衍生 | 定義 | `SumTy` 顯示函數的機器衍生助手 | `{Ty : Type} →   [Repr Ty] → (x : Polyrust.SumTy Ty) → Polyrust.SumTy.below (motive := fun x => Na…` |
| 527 | `Polyrust.instReprSumTy.repr._sunfold` | 實例衍生 | 定義 | `SumTy` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Polyrust.SumTy Ty → Nat → Format` |
| 528 | `Polyrust.instReprSumTy.repr._unsafe_rec` | 實例衍生 | 定義 | `SumTy` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Polyrust.SumTy Ty → Nat → Format` |
| 529 | `Polyrust.instReprSumTy.repr.match_1` | 實例衍生 | 定義 | `SumTy` 顯示函數的機器衍生助手 | `{Ty : Type} →   (motive : Polyrust.SumTy Ty → Sort u_1) →     (x : Polyrust.SumTy Ty) →       ((a…` |
| 530 | `Polyrust.SumExpr.brecOn.eq` | 衍生 | 定理 | `SumExpr` 相關助手（機器衍生） | `∀ {Expr : Type} {motive : Polyrust.SumExpr Expr → Sort u} (t : Polyrust.SumExpr Expr)   (F_1 : (t…` |
| 531 | `Polyrust.SumExpr.ctorElimType` | 衍生 | 定義 | `SumExpr` 相關助手（機器衍生） | `{Expr : Type} → {motive : Polyrust.SumExpr Expr → Sort u} → Nat → Sort (max 1 u)` |
| 532 | `Polyrust.SumExpr.noConfusionType` | 衍生 | 定義 | `SumExpr` 相關助手（機器衍生） | `Sort u → {Expr : Type} → Polyrust.SumExpr Expr → {Expr' : Type} → Polyrust.SumExpr Expr' → Sort u` |
| 533 | `Polyrust.SumTy.brecOn.eq` | 衍生 | 定理 | `SumTy` 相關助手（機器衍生） | `∀ {Ty : Type} {motive : Polyrust.SumTy Ty → Sort u} (t : Polyrust.SumTy Ty)   (F_1 : (t : Polyrus…` |
| 534 | `Polyrust.SumTy.ctorElimType` | 衍生 | 定義 | `SumTy` 相關助手（機器衍生） | `{Ty : Type} → {motive : Polyrust.SumTy Ty → Sort u} → Nat → Sort (max 1 u)` |
| 535 | `Polyrust.SumTy.noConfusionType` | 衍生 | 定義 | `SumTy` 相關助手（機器衍生） | `Sort u → {Ty : Type} → Polyrust.SumTy Ty → {Ty' : Type} → Polyrust.SumTy Ty' → Sort u` |

## `Polyrust.ProductReduction`（97 條）

職責：乘積項歸約

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 536 | `Polyrust.TypableProd` | 手寫 | 定義 | 積型可定型：存在某個積型使檢查通過。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Prop` |
| 537 | `Polyrust.pairBit` | 手寫 | 定義 | pair 節點對分量型別 `(τ₁, τ₂)` 的位元，由分量位元 `σa τ₁ · σb τ₂` 給出 （積型規則方程 `t_{pair,τ₁τ₂} = t_{a,τ₁} · t_{b,τ₂}` 的位元形式）。 | `{Ty : Type} → (Ty → Bool) → (Ty → Bool) → Ty → Ty → Int` |
| 538 | `Polyrust.pairBitSum` | 手寫 | 定義 | pair 節點的**位元總和**（對所有 `(τ₁, τ₂) ∈ enumAll × enumAll` 求和）。 | `{Ty : Type} → Polyrust.Lang Ty → (Ty → Bool) → (Ty → Bool) → Int` |
| 539 | `Polyrust.tycheckProd` | 手寫 | 定義 | ! ## 二、積型檢查器與檢查歸約 積型檢查器：`lift e` 按基本語言的 `tycheck` 檢查，`pair a b` 按 `prod τ₁ τ₂` **逐欄位**檢查分量。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Polyrust.Pr…` |
| 540 | `Polyrust.tycheck_pair` | 手寫 | 定理 | **檢查歸約（T-b）**：`pair` 的檢查就是分量檢查的逐欄位合取（定義即此）。 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {a b : Polyrust.ProdExpr Polyrust.Ex…` |
| 541 | `Polyrust.ProdExpr.brecOn.go` | 等式引理／助手 | 定義 | `ProdExpr` 定義編譯時產生的遞迴／匹配助手 | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →…` |
| 542 | `Polyrust.ProdTy.brecOn.go` | 等式引理／助手 | 定義 | `ProdTy` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) →       ((t…` |
| 543 | `Polyrust.tycheckProd._f` | 等式引理／助手 | 定義 | `tycheckProd` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty →       (x : Polyrust.ProdExpr Polyrust.E…` |
| 544 | `Polyrust.tycheckProd._sunfold` | 等式引理／助手 | 定義 | `tycheckProd` 的結構遞迴展開引理 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Polyrust.Pr…` |
| 545 | `Polyrust.tycheckProd._unsafe_rec` | 等式引理／助手 | 定義 | `tycheckProd` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.ProdExpr Polyrust.Expr → Polyrust.Pr…` |
| 546 | `Polyrust.tycheckProd.eq_1` | 等式引理／助手 | 定理 | 定義 `tycheckProd` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (τ : Ty),   Poly…` |
| 547 | `Polyrust.tycheckProd.eq_2` | 等式引理／助手 | 定理 | 定義 `tycheckProd` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a : Polyrust.Expr) (a_1 a_2 : Polyr…` |
| 548 | `Polyrust.tycheckProd.eq_3` | 等式引理／助手 | 定理 | 定義 `tycheckProd` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.ProdExpr Polyrust.Ex…` |
| 549 | `Polyrust.tycheckProd.eq_4` | 等式引理／助手 | 定理 | 定義 `tycheckProd` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a a_1 : Polyrust.ProdExpr Polyrust.…` |
| 550 | `Polyrust.tycheckProd.eq_def` | 等式引理／助手 | 定理 | 定義 `tycheckProd` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.ProdExpr Polyrust.Expr…` |
| 551 | `Polyrust.tycheckProd.match_1` | 等式引理／助手 | 定義 | `tycheckProd` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   (motive : Polyrust.ProdExpr Polyrust.Expr → Polyrust.ProdTy Ty → Sort u_1) →     …` |
| 552 | `Polyrust.tycheckProd_exclusive.match_1_1` | 等式引理／助手 | 定義 | `tycheckProd_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (τ τ' : Polyrust…` |
| 553 | `Polyrust.tycheckProd_exclusive.match_1_8` | 等式引理／助手 | 定義 | `tycheckProd_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.ProdExpr Polyrust.Ex…` |
| 554 | `Polyrust.typable_pair_iff.match_1_6` | 等式引理／助手 | 定義 | `typable_pair_iff` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {a b : Polyrust.ProdExpr Polyrust.Ex…` |
| 555 | `Polyrust.ProdExpr._sizeOf_1` | 歸納型衍生 | 定義 | `ProdExpr` 的輔助大小函數 | `{Expr : Type} → [SizeOf Expr] → Polyrust.ProdExpr Expr → Nat` |
| 556 | `Polyrust.ProdExpr._sizeOf_inst` | 歸納型衍生 | 定義 | `ProdExpr` 的輔助大小函數 | `(Expr : Type) → [SizeOf Expr] → SizeOf (Polyrust.ProdExpr Expr)` |
| 557 | `Polyrust.ProdExpr.below` | 歸納型衍生 | 定義 | `ProdExpr` 的良基遞迴助手 | `{Expr : Type} → {motive : Polyrust.ProdExpr Expr → Sort u} → Polyrust.ProdExpr Expr → Sort (max 1 u)` |
| 558 | `Polyrust.ProdExpr.brecOn` | 歸納型衍生 | 定義 | `ProdExpr` 的良基遞迴助手 | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →…` |
| 559 | `Polyrust.ProdExpr.casesOn` | 歸納型衍生 | 定義 | `ProdExpr` 的案例分析原則 | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →…` |
| 560 | `Polyrust.ProdExpr.ctorElim` | 歸納型衍生 | 定義 | `ProdExpr` 的構造子結構助手（機器衍生） | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (ctorIdx : Nat) →       (t : P…` |
| 561 | `Polyrust.ProdExpr.ctorIdx` | 歸納型衍生 | 定義 | `ProdExpr` 的構造子結構助手（機器衍生） | `{Expr : Type} → Polyrust.ProdExpr Expr → Nat` |
| 562 | `Polyrust.ProdExpr.lift.elim` | 歸納型衍生 | 定義 | `ProdExpr` 的構造子消去器 | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →…` |
| 563 | `Polyrust.ProdExpr.lift.inj` | 歸納型衍生 | 定理 | `ProdExpr` 的構造子結構助手（機器衍生） | `∀ {Expr : Type} {a a_1 : Expr}, Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift a_1 → a = a_1` |
| 564 | `Polyrust.ProdExpr.lift.noConfusion` | 歸納型衍生 | 定義 | `ProdExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Expr : Type} → {P : Sort u} → {a a' : Expr} → Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift …` |
| 565 | `Polyrust.ProdExpr.lift.sizeOf_spec` | 歸納型衍生 | 定理 | `ProdExpr` 結構大小的規格命題 | `∀ {Expr : Type} [inst : SizeOf Expr] (a : Expr), sizeOf (Polyrust.ProdExpr.lift a) = 1 + sizeOf a` |
| 566 | `Polyrust.ProdExpr.noConfusion` | 歸納型衍生 | 定義 | `ProdExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Expr : Type} →     {t : Polyrust.ProdExpr Expr} →       {Expr' : Type} → {t' : …` |
| 567 | `Polyrust.ProdExpr.pair.elim` | 歸納型衍生 | 定義 | `ProdExpr` 的構造子消去器 | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →…` |
| 568 | `Polyrust.ProdExpr.pair.inj` | 歸納型衍生 | 定理 | `ProdExpr` 的構造子結構助手（機器衍生） | `∀ {Expr : Type} {a a_1 a_2 a_3 : Polyrust.ProdExpr Expr}, a.pair a_1 = a_2.pair a_3 → a = a_2 ∧ a…` |
| 569 | `Polyrust.ProdExpr.pair.noConfusion` | 歸納型衍生 | 定義 | `ProdExpr` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Expr : Type} →   {P : Sort u} → {a a_1 a' a'_1 : Polyrust.ProdExpr Expr} → a.pair a_1 = a'.pair …` |
| 570 | `Polyrust.ProdExpr.pair.sizeOf_spec` | 歸納型衍生 | 定理 | `ProdExpr` 結構大小的規格命題 | `∀ {Expr : Type} [inst : SizeOf Expr] (a a_1 : Polyrust.ProdExpr Expr), sizeOf (a.pair a_1) = 1 + …` |
| 571 | `Polyrust.ProdExpr.recOn` | 歸納型衍生 | 定義 | `ProdExpr` 的結構遞迴原則 | `{Expr : Type} →   {motive : Polyrust.ProdExpr Expr → Sort u} →     (t : Polyrust.ProdExpr Expr) →…` |
| 572 | `Polyrust.ProdTy._sizeOf_1` | 歸納型衍生 | 定義 | `ProdTy` 的輔助大小函數 | `{Ty : Type} → [SizeOf Ty] → Polyrust.ProdTy Ty → Nat` |
| 573 | `Polyrust.ProdTy._sizeOf_inst` | 歸納型衍生 | 定義 | `ProdTy` 的輔助大小函數 | `(Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.ProdTy Ty)` |
| 574 | `Polyrust.ProdTy.base.elim` | 歸納型衍生 | 定義 | `ProdTy` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) → t.ctorIdx…` |
| 575 | `Polyrust.ProdTy.base.inj` | 歸納型衍生 | 定理 | `ProdTy` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a a_1 : Ty}, Polyrust.ProdTy.base a = Polyrust.ProdTy.base a_1 → a = a_1` |
| 576 | `Polyrust.ProdTy.base.noConfusion` | 歸納型衍生 | 定義 | `ProdTy` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} → {P : Sort u} → {a a' : Ty} → Polyrust.ProdTy.base a = Polyrust.ProdTy.base a' → (a …` |
| 577 | `Polyrust.ProdTy.base.sizeOf_spec` | 歸納型衍生 | 定理 | `ProdTy` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a : Ty), sizeOf (Polyrust.ProdTy.base a) = 1 + sizeOf a` |
| 578 | `Polyrust.ProdTy.below` | 歸納型衍生 | 定義 | `ProdTy` 的良基遞迴助手 | `{Ty : Type} → {motive : Polyrust.ProdTy Ty → Sort u} → Polyrust.ProdTy Ty → Sort (max 1 u)` |
| 579 | `Polyrust.ProdTy.brecOn` | 歸納型衍生 | 定義 | `ProdTy` 的良基遞迴助手 | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) → ((t : Pol…` |
| 580 | `Polyrust.ProdTy.casesOn` | 歸納型衍生 | 定義 | `ProdTy` 的案例分析原則 | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) →       ((a…` |
| 581 | `Polyrust.ProdTy.ctorElim` | 歸納型衍生 | 定義 | `ProdTy` 的構造子結構助手（機器衍生） | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (ctorIdx : Nat) → (t : Polyrust.Prod…` |
| 582 | `Polyrust.ProdTy.ctorIdx` | 歸納型衍生 | 定義 | `ProdTy` 的構造子結構助手（機器衍生） | `{Ty : Type} → Polyrust.ProdTy Ty → Nat` |
| 583 | `Polyrust.ProdTy.noConfusion` | 歸納型衍生 | 定義 | `ProdTy` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Ty : Type} →     {t : Polyrust.ProdTy Ty} →       {Ty' : Type} → {t' : Polyrust…` |
| 584 | `Polyrust.ProdTy.prod.elim` | 歸納型衍生 | 定義 | `ProdTy` 的構造子消去器 | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) → t.ctorIdx…` |
| 585 | `Polyrust.ProdTy.prod.inj` | 歸納型衍生 | 定理 | `ProdTy` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {a a_1 a_2 a_3 : Polyrust.ProdTy Ty}, a.prod a_1 = a_2.prod a_3 → a = a_2 ∧ a_1 = a_3` |
| 586 | `Polyrust.ProdTy.prod.noConfusion` | 歸納型衍生 | 定義 | `ProdTy` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} →   {P : Sort u} → {a a_1 a' a'_1 : Polyrust.ProdTy Ty} → a.prod a_1 = a'.prod a'_1 →…` |
| 587 | `Polyrust.ProdTy.prod.sizeOf_spec` | 歸納型衍生 | 定理 | `ProdTy` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (a a_1 : Polyrust.ProdTy Ty), sizeOf (a.prod a_1) = 1 + sizeOf a…` |
| 588 | `Polyrust.ProdTy.recOn` | 歸納型衍生 | 定義 | `ProdTy` 的結構遞迴原則 | `{Ty : Type} →   {motive : Polyrust.ProdTy Ty → Sort u} →     (t : Polyrust.ProdTy Ty) →       ((a…` |
| 589 | `Polyrust.instDecidableEqProdExpr` | 實例衍生 | 定義 | `ProdExpr` 可判定相等（DecidableEq）的判定程序 | `{Expr : Type} → [DecidableEq Expr] → DecidableEq (Polyrust.ProdExpr Expr)` |
| 590 | `Polyrust.instDecidableEqProdExpr.decEq` | 實例衍生 | 定義 | `ProdExpr` 可判定相等（DecidableEq）的判定程序 | `{Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.ProdExpr Expr) → Decidable (x = x_1)` |
| 591 | `Polyrust.instDecidableEqProdExpr.decEq._f` | 實例衍生 | 定義 | `ProdExpr` 相等性判定的遞迴助手（機器衍生） | `{Expr : Type} →   [DecidableEq Expr] →     (x : Polyrust.ProdExpr Expr) →       Polyrust.ProdExpr…` |
| 592 | `Polyrust.instDecidableEqProdExpr.decEq._proof_1` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Expr), Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift a` |
| 593 | `Polyrust.instDecidableEqProdExpr.decEq._proof_2` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a b : Expr), ¬a = b → Polyrust.ProdExpr.lift a = Polyrust.ProdExpr.lift b → False` |
| 594 | `Polyrust.instDecidableEqProdExpr.decEq._proof_3` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a : Expr) (a_1 a_2 : Polyrust.ProdExpr Expr), Polyrust.ProdExpr.lift a = a_1.pai…` |
| 595 | `Polyrust.instDecidableEqProdExpr.decEq._proof_4` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a a_1 : Polyrust.ProdExpr Expr) (a_2 : Expr), a.pair a_1 = Polyrust.ProdExpr.lif…` |
| 596 | `Polyrust.instDecidableEqProdExpr.decEq._proof_5` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a a_1 : Polyrust.ProdExpr Expr), a.pair a_1 = a.pair a_1` |
| 597 | `Polyrust.instDecidableEqProdExpr.decEq._proof_6` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a a_1 b : Polyrust.ProdExpr Expr), ¬a_1 = b → a.pair a_1 = a.pair b → False` |
| 598 | `Polyrust.instDecidableEqProdExpr.decEq._proof_7` | 實例衍生 | 定理 | `ProdExpr` 相等性判定的內部子證明（機器衍生） | `∀ {Expr : Type} (a a_1 b b_1 : Polyrust.ProdExpr Expr), ¬a = b → a.pair a_1 = b.pair b_1 → False` |
| 599 | `Polyrust.instDecidableEqProdExpr.decEq._sunfold` | 實例衍生 | 定義 | `ProdExpr` 相等性判定的展開引理（機器衍生） | `{Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.ProdExpr Expr) → Decidable (x = x_1)` |
| 600 | `Polyrust.instDecidableEqProdExpr.decEq._unsafe_rec` | 實例衍生 | 定義 | `ProdExpr` 相等性判定的遞迴助手（機器衍生） | `{Expr : Type} → [DecidableEq Expr] → (x x_1 : Polyrust.ProdExpr Expr) → Decidable (x = x_1)` |
| 601 | `Polyrust.instDecidableEqProdExpr.decEq.match_1` | 實例衍生 | 定義 | `ProdExpr` 相等性判定的輔助匹配（機器衍生） | `{Expr : Type} →   (motive : Polyrust.ProdExpr Expr → Polyrust.ProdExpr Expr → Sort u_1) →     (x …` |
| 602 | `Polyrust.instDecidableEqProdTy` | 實例衍生 | 定義 | `ProdTy` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → DecidableEq (Polyrust.ProdTy Ty)` |
| 603 | `Polyrust.instDecidableEqProdTy.decEq` | 實例衍生 | 定義 | `ProdTy` 可判定相等（DecidableEq）的判定程序 | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ProdTy Ty) → Decidable (x = x_1)` |
| 604 | `Polyrust.instDecidableEqProdTy.decEq._f` | 實例衍生 | 定義 | `ProdTy` 相等性判定的遞迴助手（機器衍生） | `{Ty : Type} →   [DecidableEq Ty] →     (x : Polyrust.ProdTy Ty) →       Polyrust.ProdTy.below (mo…` |
| 605 | `Polyrust.instDecidableEqProdTy.decEq._proof_1` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Ty), Polyrust.ProdTy.base a = Polyrust.ProdTy.base a` |
| 606 | `Polyrust.instDecidableEqProdTy.decEq._proof_2` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a b : Ty), ¬a = b → Polyrust.ProdTy.base a = Polyrust.ProdTy.base b → False` |
| 607 | `Polyrust.instDecidableEqProdTy.decEq._proof_3` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a : Ty) (a_1 a_2 : Polyrust.ProdTy Ty), Polyrust.ProdTy.base a = a_1.prod a_2 → False` |
| 608 | `Polyrust.instDecidableEqProdTy.decEq._proof_4` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 : Polyrust.ProdTy Ty) (a_2 : Ty), a.prod a_1 = Polyrust.ProdTy.base a_2 → False` |
| 609 | `Polyrust.instDecidableEqProdTy.decEq._proof_5` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 : Polyrust.ProdTy Ty), a.prod a_1 = a.prod a_1` |
| 610 | `Polyrust.instDecidableEqProdTy.decEq._proof_6` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 b : Polyrust.ProdTy Ty), ¬a_1 = b → a.prod a_1 = a.prod b → False` |
| 611 | `Polyrust.instDecidableEqProdTy.decEq._proof_7` | 實例衍生 | 定理 | `ProdTy` 相等性判定的內部子證明（機器衍生） | `∀ {Ty : Type} (a a_1 b b_1 : Polyrust.ProdTy Ty), ¬a = b → a.prod a_1 = b.prod b_1 → False` |
| 612 | `Polyrust.instDecidableEqProdTy.decEq._sunfold` | 實例衍生 | 定義 | `ProdTy` 相等性判定的展開引理（機器衍生） | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ProdTy Ty) → Decidable (x = x_1)` |
| 613 | `Polyrust.instDecidableEqProdTy.decEq._unsafe_rec` | 實例衍生 | 定義 | `ProdTy` 相等性判定的遞迴助手（機器衍生） | `{Ty : Type} → [DecidableEq Ty] → (x x_1 : Polyrust.ProdTy Ty) → Decidable (x = x_1)` |
| 614 | `Polyrust.instDecidableEqProdTy.decEq.match_1` | 實例衍生 | 定義 | `ProdTy` 相等性判定的輔助匹配（機器衍生） | `{Ty : Type} →   (motive : Polyrust.ProdTy Ty → Polyrust.ProdTy Ty → Sort u_1) →     (x x_1 : Poly…` |
| 615 | `Polyrust.instReprProdExpr` | 實例衍生 | 定義 | `ProdExpr` 顯示函數的機器衍生助手 | `{Expr : Type} → [Repr Expr] → Repr (Polyrust.ProdExpr Expr)` |
| 616 | `Polyrust.instReprProdExpr.repr` | 實例衍生 | 定義 | `ProdExpr` 的顯示函數（Repr 型別類別實例） | `{Expr : Type} → [Repr Expr] → Polyrust.ProdExpr Expr → Nat → Format` |
| 617 | `Polyrust.instReprProdExpr.repr._f` | 實例衍生 | 定義 | `ProdExpr` 顯示函數的機器衍生助手 | `{Expr : Type} →   [Repr Expr] →     (x : Polyrust.ProdExpr Expr) → Polyrust.ProdExpr.below (motiv…` |
| 618 | `Polyrust.instReprProdExpr.repr._sunfold` | 實例衍生 | 定義 | `ProdExpr` 顯示函數的機器衍生助手 | `{Expr : Type} → [Repr Expr] → Polyrust.ProdExpr Expr → Nat → Format` |
| 619 | `Polyrust.instReprProdExpr.repr._unsafe_rec` | 實例衍生 | 定義 | `ProdExpr` 顯示函數的機器衍生助手 | `{Expr : Type} → [Repr Expr] → Polyrust.ProdExpr Expr → Nat → Format` |
| 620 | `Polyrust.instReprProdExpr.repr.match_1` | 實例衍生 | 定義 | `ProdExpr` 顯示函數的機器衍生助手 | `{Expr : Type} →   (motive : Polyrust.ProdExpr Expr → Sort u_1) →     (x : Polyrust.ProdExpr Expr)…` |
| 621 | `Polyrust.instReprProdTy` | 實例衍生 | 定義 | `ProdTy` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Repr (Polyrust.ProdTy Ty)` |
| 622 | `Polyrust.instReprProdTy.repr` | 實例衍生 | 定義 | `ProdTy` 的顯示函數（Repr 型別類別實例） | `{Ty : Type} → [Repr Ty] → Polyrust.ProdTy Ty → Nat → Format` |
| 623 | `Polyrust.instReprProdTy.repr._f` | 實例衍生 | 定義 | `ProdTy` 顯示函數的機器衍生助手 | `{Ty : Type} →   [Repr Ty] → (x : Polyrust.ProdTy Ty) → Polyrust.ProdTy.below (motive := fun x => …` |
| 624 | `Polyrust.instReprProdTy.repr._sunfold` | 實例衍生 | 定義 | `ProdTy` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Polyrust.ProdTy Ty → Nat → Format` |
| 625 | `Polyrust.instReprProdTy.repr._unsafe_rec` | 實例衍生 | 定義 | `ProdTy` 顯示函數的機器衍生助手 | `{Ty : Type} → [Repr Ty] → Polyrust.ProdTy Ty → Nat → Format` |
| 626 | `Polyrust.instReprProdTy.repr.match_1` | 實例衍生 | 定義 | `ProdTy` 顯示函數的機器衍生助手 | `{Ty : Type} →   (motive : Polyrust.ProdTy Ty → Sort u_1) →     (x : Polyrust.ProdTy Ty) →       (…` |
| 627 | `Polyrust.ProdExpr.brecOn.eq` | 衍生 | 定理 | `ProdExpr` 相關助手（機器衍生） | `∀ {Expr : Type} {motive : Polyrust.ProdExpr Expr → Sort u} (t : Polyrust.ProdExpr Expr)   (F_1 : …` |
| 628 | `Polyrust.ProdExpr.ctorElimType` | 衍生 | 定義 | `ProdExpr` 相關助手（機器衍生） | `{Expr : Type} → {motive : Polyrust.ProdExpr Expr → Sort u} → Nat → Sort (max 1 u)` |
| 629 | `Polyrust.ProdExpr.noConfusionType` | 衍生 | 定義 | `ProdExpr` 相關助手（機器衍生） | `Sort u → {Expr : Type} → Polyrust.ProdExpr Expr → {Expr' : Type} → Polyrust.ProdExpr Expr' → Sort u` |
| 630 | `Polyrust.ProdTy.brecOn.eq` | 衍生 | 定理 | `ProdTy` 相關助手（機器衍生） | `∀ {Ty : Type} {motive : Polyrust.ProdTy Ty → Sort u} (t : Polyrust.ProdTy Ty)   (F_1 : (t : Polyr…` |
| 631 | `Polyrust.ProdTy.ctorElimType` | 衍生 | 定義 | `ProdTy` 相關助手（機器衍生） | `{Ty : Type} → {motive : Polyrust.ProdTy Ty → Sort u} → Nat → Sort (max 1 u)` |
| 632 | `Polyrust.ProdTy.noConfusionType` | 衍生 | 定義 | `ProdTy` 相關助手（機器衍生） | `Sort u → {Ty : Type} → Polyrust.ProdTy Ty → {Ty' : Type} → Polyrust.ProdTy Ty' → Sort u` |

## `Polyrust.BorrowOwnership`（90 條）

職責：借用／所有權類型檢查的健全性（嵌入式語言層）

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 633 | `Polyrust.BSign` | 手寫 | 定義 | ! ## 三、約束系統（與 `constraints.rs` 的 `emit` 逐式對應） 借用位元賦值（與 CDCL 的變量賦值同型：`Nat → Bool`）。 | `Type` |
| 634 | `Polyrust.BgenIdeal` | 手寫 | 定義 | 由集合 `S` 生成的理想。 | `((Polyrust.BSign → Int) → Prop) → (Polyrust.BSign → Int) → Prop` |
| 635 | `Polyrust.BgenIdeal_isIdeal` | 手寫 | 定理 | 命題證明 | `∀ (S : (Polyrust.BSign → Int) → Prop), Polyrust.BIsIdeal (Polyrust.BgenIdeal S)` |
| 636 | `Polyrust.BgenIdeal_mono` | 手寫 | 定理 | 命題證明 | `∀ {S T : (Polyrust.BSign → Int) → Prop},   (∀ (q : Polyrust.BSign → Int), S q → T q) →     ∀ (p :…` |
| 637 | `Polyrust.BgenIdeal_subset` | 手寫 | 定理 | 命題證明 | `∀ {S : (Polyrust.BSign → Int) → Prop} {q : Polyrust.BSign → Int}, S q → Polyrust.BgenIdeal S q` |
| 638 | `Polyrust.Clean` | 手寫 | 定義 | **分析結果為乾淨**（對應 `BorrowAnalysis::is_clean`）：無任何衝突。 | `Polyrust.BorrowAnalysis → Prop` |
| 639 | `Polyrust.DistinctConflict` | 手寫 | 定義 | 相異節點的兩個借用才可能構成衝突對（把「相異」說死，避免自重疊的假衝突）。 | `Polyrust.Borrow → Polyrust.Borrow → Prop` |
| 640 | `Polyrust.assignClause` | 手寫 | 定義 | 單元子句：`¬b`。對應 `clauses.push(vec![lit(b,false)])`。 | `Nat → List Polyrust.Lit` |
| 641 | `Polyrust.assignConflict` | 手寫 | 定義 | **賦值衝突**（所有權規則一）：在點 `p` 對變量 `defn` 賦值，而借用 `b` 仍存活 （`b.start ≤ p < b.stop`）。對應 `analysis.rs` 的 `assign_conflicts`。 | `Polyrust.Borrow → Nat → Nat → Prop` |
| 642 | `Polyrust.assignConflict_definitions_agree` | 手寫 | 定理 | 等式／恆等命題 | `∀ (b : Polyrust.Borrow) (defn p : Nat), Polyrust.assignConflict b defn p ↔ b.varDef = defn ∧ b.st…` |
| 643 | `Polyrust.assignPoly` | 手寫 | 定義 | 被排除節點：`b = 0`（借用期間賦值／移動 ⇒ 該借用必須不存在）。 對應 `emit(var_poly(b))`。 | `Nat → Polyrust.BSign → Int` |
| 644 | `Polyrust.assignSet` | 手寫 | 定義 | 被排除的約束集：`{b, b − 1}`。 | `Nat → (Polyrust.BSign → Int) → Prop` |
| 645 | `Polyrust.bb` | 手寫 | 定義 | 位元 → 整數。 | `Polyrust.BSign → Nat → Int` |
| 646 | `Polyrust.borrowSystem` | 手寫 | 定義 | 借用約束系統：活躍方程 ++ 衝突多項式 ++ 被排除方程。 | `List Nat → List (Nat × Nat) → List Nat → List (Polyrust.BSign → Int)` |
| 647 | `Polyrust.clashClause` | 手寫 | 定義 | 衝突子句：`¬b_i ∨ ¬b_j`。對應 `clauses.push(vec![lit(b1,false), lit(b2,false)])`。 | `Nat → Nat → List Polyrust.Lit` |
| 648 | `Polyrust.clashPoly` | 手寫 | 定義 | 衝突多項式：`b_i · b_j = 0`（兩借用不可同時存在）。 對應 `emit(var_poly(b1).mul(&var_poly(b2)))`。 | `Nat → Nat → Polyrust.BSign → Int` |
| 649 | `Polyrust.clashSet` | 手寫 | 定義 | 衝突的約束集：`{x_i x_j, x_i − 1, x_j − 1}`。 | `Nat → Nat → (Polyrust.BSign → Int) → Prop` |
| 650 | `Polyrust.conflictsWith` | 手寫 | 定義 | 借用衝突：同一變量且存活區間重疊。 | `Polyrust.Borrow → Polyrust.Borrow → Prop` |
| 651 | `Polyrust.conflictsWith_comm` | 手寫 | 定理 | 命題證明 | `∀ {b₁ b₂ : Polyrust.Borrow}, Polyrust.conflictsWith b₁ b₂ ↔ Polyrust.conflictsWith b₂ b₁` |
| 652 | `Polyrust.liveEq` | 手寫 | 定義 | 活躍方程：`b − 1 = 0`（借用存在）。對應 `emit(var_poly(bv).sub(&ONE))`。 | `Nat → Polyrust.BSign → Int` |
| 653 | `Polyrust.moveConflict` | 手寫 | 定義 | **移動衝突**（所有權規則二）：在點 `p` 把變量 `defn` 的所有權移出，而借用仍存活。 Mini-Rust 尚無 `move`；此處形式化的是規則本身。 | `Polyrust.Borrow → Nat → Nat → Prop` |
| 654 | `Polyrust.negLit` | 手寫 | 定義 | 負文字 `¬x_n`（活躍借用的位元是「不得同時為真」，故子句取負文字）。 | `Nat → Polyrust.Lit` |
| 655 | `Polyrust.overlaps` | 手寫 | 定義 | **區間重疊**：半開區間 `[s₁,e₁) ∩ [s₂,e₂) ≠ ∅ ⟺ s₁ < e₂ ∧ s₂ < e₁`。 與 `analysis.rs` 的 `a.start < b.end && b.start < a.end` 逐字對應。 | `Polyrust.Borrow → Polyrust.Borrow → Prop` |
| 656 | `Polyrust.overlaps_comm` | 手寫 | 定理 | 命題證明 | `∀ {b₁ b₂ : Polyrust.Borrow}, Polyrust.overlaps b₁ b₂ ↔ Polyrust.overlaps b₂ b₁` |
| 657 | `Polyrust.overlaps_self_iff` | 手寫 | 定理 | 重疊的自反判定：區間非空時自身重疊。 | `∀ (b : Polyrust.Borrow), Polyrust.overlaps b b ↔ b.start < b.stop` |
| 658 | `Polyrust.overlaps_self_of_nonempty` | 手寫 | 定理 | 「同一借用在多處被使用」不是衝突；衝突來自**兩個不同**的借用節點。 | `∀ {b : Polyrust.Borrow}, b.start < b.stop → Polyrust.overlaps b b` |
| 659 | `Polyrust.p5Borrows` | 手寫 | 定義 | ! ## 八、P5 與 P6：唯一差別是存活區間 P5-twice-mut 的借用結構核心：`let r1 = &mut x; let r2 = &mut x; *r1 + *r2` ——`r1` 存活到「最後使用點」，故兩個 `&mut` | `List Polyrust.Borrow` |
| 660 | `Polyrust.p5_conflict` | 手寫 | 定理 | **P5 衝突**：兩個同變量借用區間重疊 ⇒ UNSAT。 | `Polyrust.conflictsWith { node := 1, varDef := 7, start := 1, stop := 4 }   { node := 2, varDef :=…` |
| 661 | `Polyrust.p6Borrows` | 手寫 | 定義 | P6-temp-borrow 的借用結構核心：`*(&mut x) + *(&mut x)` ——兩個暫時借用各只在「自己那一點」存活，區間不相交。 | `List Polyrust.Borrow` |
| 662 | `Polyrust.p6_no_conflict` | 手寫 | 定理 | **P6 無衝突**：區間不相交（`1 + 1 ≤ 3`）⇒ SAT。 | `¬Polyrust.conflictsWith { node := 1, varDef := 7, start := 1, stop := 2 }     { node := 2, varDef…` |
| 663 | `Polyrust.p6_no_distinct_conflict` | 手寫 | 定理 | **相異節點的衝突判定（P6）**：兩個暫時借用的區間 `[1,2)`、`[3,4)` 不相交。 | `¬Polyrust.DistinctConflict { node := 1, varDef := 7, start := 1, stop := 2 }     { node := 2, var…` |
| 664 | `Polyrust.useAfterMove` | 手寫 | 定義 | **移動後使用**（所有權規則三）：先於點 `m` 移出所有權，之後於點 `p > m` 使用。 | `Nat → Nat → Nat → Prop` |
| 665 | `Polyrust.use_after_move_is_clash` | 手寫 | 定理 | 故 `useAfterMove` 不需要新的代數機制。 | `∀ (m u : Nat), Polyrust.borrowSystem [m, u] [(m, u)] [] = [Polyrust.liveEq m, Polyrust.liveEq u, …` |
| 666 | `Polyrust.varLit` | 手寫 | 定義 | ! ## 五、借用子句與 T3(a) 對偶（CDCL 側） 正文字 `x_n`。 | `Nat → Polyrust.Lit` |
| 667 | `Polyrust.assignClause.eq_1` | 等式引理／助手 | 定理 | 定義 `assignClause` 的等式引理（定義的展開方程） | `∀ (n : Nat), Polyrust.assignClause n = [Polyrust.negLit n]` |
| 668 | `Polyrust.assignPoly.eq_1` | 等式引理／助手 | 定理 | 定義 `assignPoly` 的等式引理（定義的展開方程） | `∀ (n : Nat) (β : Polyrust.BSign), Polyrust.assignPoly n β = Polyrust.bb β n` |
| 669 | `Polyrust.bb.eq_1` | 等式引理／助手 | 定理 | 定義 `bb` 的等式引理（定義的展開方程） | `∀ (β : Polyrust.BSign) (n : Nat), Polyrust.bb β n = Polyrust.bit (β n)` |
| 670 | `Polyrust.borrowSystem.eq_1` | 等式引理／助手 | 定理 | 定義 `borrowSystem` 的等式引理（定義的展開方程） | `∀ (live : List Nat) (pairs : List (Nat × Nat)) (assigns : List Nat),   Polyrust.borrowSystem live…` |
| 671 | `Polyrust.clashClause.eq_1` | 等式引理／助手 | 定理 | 定義 `clashClause` 的等式引理（定義的展開方程） | `∀ (i j : Nat), Polyrust.clashClause i j = [Polyrust.negLit i, Polyrust.negLit j]` |
| 672 | `Polyrust.clashPoly.eq_1` | 等式引理／助手 | 定理 | 定義 `clashPoly` 的等式引理（定義的展開方程） | `∀ (i j : Nat) (β : Polyrust.BSign), Polyrust.clashPoly i j β = Polyrust.bb β i * Polyrust.bb β j` |
| 673 | `Polyrust.liveEq.eq_1` | 等式引理／助手 | 定理 | 定義 `liveEq` 的等式引理（定義的展開方程） | `∀ (n : Nat) (β : Polyrust.BSign), Polyrust.liveEq n β = Polyrust.bb β n - 1` |
| 674 | `Polyrust.negLit.eq_1` | 等式引理／助手 | 定理 | 定義 `negLit` 的等式引理（定義的展開方程） | `∀ (n : Nat), Polyrust.negLit n = (Polyrust.varLit n).neg` |
| 675 | `Polyrust.p5Borrows.eq_1` | 等式引理／助手 | 定理 | 定義 `p5Borrows` 的等式引理（定義的展開方程） | `Polyrust.p5Borrows =   [{ node := 1, varDef := 7, start := 1, stop := 4 }, { node := 2, varDef :=…` |
| 676 | `Polyrust.p6Borrows.eq_1` | 等式引理／助手 | 定理 | 定義 `p6Borrows` 的等式引理（定義的展開方程） | `Polyrust.p6Borrows =   [{ node := 1, varDef := 7, start := 1, stop := 2 }, { node := 2, varDef :=…` |
| 677 | `Polyrust.varLit.eq_1` | 等式引理／助手 | 定理 | 定義 `varLit` 的等式引理（定義的展開方程） | `∀ (n : Nat), Polyrust.varLit n = { var := n, pos := true }` |
| 678 | `Polyrust.BIsIdeal.casesOn` | 歸納型衍生 | 定義 | `BIsIdeal` 的案例分析原則 | `{I : (Polyrust.BSign → Int) → Prop} →   {motive : Polyrust.BIsIdeal I → Sort u} →     (t : Polyru…` |
| 679 | `Polyrust.BIsIdeal.mk._flat_ctor` | 歸納型衍生 | 定義 | `BIsIdeal` 的構造子結構助手（機器衍生） | `∀ {I : (Polyrust.BSign → Int) → Prop},   (I fun x => 0) →     (∀ (a b : Polyrust.BSign → Int), I …` |
| 680 | `Polyrust.BIsIdeal.recOn` | 歸納型衍生 | 定義 | `BIsIdeal` 的結構遞迴原則 | `{I : (Polyrust.BSign → Int) → Prop} →   {motive : Polyrust.BIsIdeal I → Sort u} →     (t : Polyru…` |
| 681 | `Polyrust.Borrow._sizeOf_1` | 歸納型衍生 | 定義 | `Borrow` 的輔助大小函數 | `Polyrust.Borrow → Nat` |
| 682 | `Polyrust.Borrow._sizeOf_inst` | 歸納型衍生 | 定義 | `Borrow` 的輔助大小函數 | `SizeOf Polyrust.Borrow` |
| 683 | `Polyrust.Borrow.casesOn` | 歸納型衍生 | 定義 | `Borrow` 的案例分析原則 | `{motive : Polyrust.Borrow → Sort u} →   (t : Polyrust.Borrow) →     ((node varDef start stop : Na…` |
| 684 | `Polyrust.Borrow.ctorIdx` | 歸納型衍生 | 定義 | `Borrow` 的構造子結構助手（機器衍生） | `Polyrust.Borrow → Nat` |
| 685 | `Polyrust.Borrow.mk._flat_ctor` | 歸納型衍生 | 定義 | `Borrow` 的構造子結構助手（機器衍生） | `Nat → Nat → Nat → Nat → Polyrust.Borrow` |
| 686 | `Polyrust.Borrow.mk.inj` | 歸納型衍生 | 定理 | `Borrow` 的構造子結構助手（機器衍生） | `∀ {node varDef start stop node_1 varDef_1 start_1 stop_1 : Nat},   { node := node, varDef := varD…` |
| 687 | `Polyrust.Borrow.mk.noConfusion` | 歸納型衍生 | 定義 | `Borrow` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {node varDef start stop node' varDef' start' stop' : Nat} →     { node := node, …` |
| 688 | `Polyrust.Borrow.mk.sizeOf_spec` | 歸納型衍生 | 定理 | `Borrow` 結構大小的規格命題 | `∀ (node varDef start stop : Nat),   sizeOf { node := node, varDef := varDef, start := start, stop…` |
| 689 | `Polyrust.Borrow.noConfusion` | 歸納型衍生 | 定義 | `Borrow` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {t t' : Polyrust.Borrow} → t = t' → Polyrust.Borrow.noConfusionType P t t'` |
| 690 | `Polyrust.Borrow.recOn` | 歸納型衍生 | 定義 | `Borrow` 的結構遞迴原則 | `{motive : Polyrust.Borrow → Sort u} →   (t : Polyrust.Borrow) →     ((node varDef start stop : Na…` |
| 691 | `Polyrust.BorrowAnalysis._sizeOf_1` | 歸納型衍生 | 定義 | `BorrowAnalysis` 的輔助大小函數 | `Polyrust.BorrowAnalysis → Nat` |
| 692 | `Polyrust.BorrowAnalysis._sizeOf_inst` | 歸納型衍生 | 定義 | `BorrowAnalysis` 的輔助大小函數 | `SizeOf Polyrust.BorrowAnalysis` |
| 693 | `Polyrust.BorrowAnalysis.casesOn` | 歸納型衍生 | 定義 | `BorrowAnalysis` 的案例分析原則 | `{motive : Polyrust.BorrowAnalysis → Sort u} →   (t : Polyrust.BorrowAnalysis) →     ((live : List…` |
| 694 | `Polyrust.BorrowAnalysis.ctorIdx` | 歸納型衍生 | 定義 | `BorrowAnalysis` 的構造子結構助手（機器衍生） | `Polyrust.BorrowAnalysis → Nat` |
| 695 | `Polyrust.BorrowAnalysis.mk._flat_ctor` | 歸納型衍生 | 定義 | `BorrowAnalysis` 的構造子結構助手（機器衍生） | `List Nat → List (Nat × Nat) → List Nat → Polyrust.BorrowAnalysis` |
| 696 | `Polyrust.BorrowAnalysis.mk.inj` | 歸納型衍生 | 定理 | `BorrowAnalysis` 的構造子結構助手（機器衍生） | `∀ {live : List Nat} {pairs : List (Nat × Nat)} {assigns live_1 : List Nat} {pairs_1 : List (Nat ×…` |
| 697 | `Polyrust.BorrowAnalysis.mk.noConfusion` | 歸納型衍生 | 定義 | `BorrowAnalysis` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {live : List Nat} →     {pairs : List (Nat × Nat)} →       {assigns live' : List…` |
| 698 | `Polyrust.BorrowAnalysis.mk.sizeOf_spec` | 歸納型衍生 | 定理 | `BorrowAnalysis` 結構大小的規格命題 | `∀ (live : List Nat) (pairs : List (Nat × Nat)) (assigns : List Nat),   sizeOf { live := live, pai…` |
| 699 | `Polyrust.BorrowAnalysis.noConfusion` | 歸納型衍生 | 定義 | `BorrowAnalysis` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {t t' : Polyrust.BorrowAnalysis} → t = t' → Polyrust.BorrowAnalysis.noConfusionTyp…` |
| 700 | `Polyrust.BorrowAnalysis.recOn` | 歸納型衍生 | 定義 | `BorrowAnalysis` 的結構遞迴原則 | `{motive : Polyrust.BorrowAnalysis → Sort u} →   (t : Polyrust.BorrowAnalysis) →     ((live : List…` |
| 701 | `Polyrust.instDecidableEqBorrow` | 實例衍生 | 定義 | `Borrow` 可判定相等（DecidableEq）的判定程序 | `DecidableEq Polyrust.Borrow` |
| 702 | `Polyrust.instDecidableEqBorrow.decEq` | 實例衍生 | 定義 | `Borrow` 可判定相等（DecidableEq）的判定程序 | `(x x_1 : Polyrust.Borrow) → Decidable (x = x_1)` |
| 703 | `Polyrust.instDecidableEqBorrow.decEq._proof_1` | 實例衍生 | 定理 | `Borrow` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 : Nat),   { node := a, varDef := a_1, start := a_2, stop := a_3 } = { node := a,…` |
| 704 | `Polyrust.instDecidableEqBorrow.decEq._proof_2` | 實例衍生 | 定理 | `Borrow` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 b : Nat),   ¬a_3 = b →     { node := a, varDef := a_1, start := a_2, stop := a_3…` |
| 705 | `Polyrust.instDecidableEqBorrow.decEq._proof_3` | 實例衍生 | 定理 | `Borrow` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 b b_1 : Nat),   ¬a_2 = b →     { node := a, varDef := a_1, start := a_2, stop :=…` |
| 706 | `Polyrust.instDecidableEqBorrow.decEq._proof_4` | 實例衍生 | 定理 | `Borrow` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 b b_1 b_2 : Nat),   ¬a_1 = b →     { node := a, varDef := a_1, start := a_2, sto…` |
| 707 | `Polyrust.instDecidableEqBorrow.decEq._proof_5` | 實例衍生 | 定理 | `Borrow` 相等性判定的內部子證明（機器衍生） | `∀ (a a_1 a_2 a_3 b b_1 b_2 b_3 : Nat),   ¬a = b →     { node := a, varDef := a_1, start := a_2, s…` |
| 708 | `Polyrust.instDecidableEqBorrow.decEq.match_1` | 實例衍生 | 定義 | `Borrow` 相等性判定的輔助匹配（機器衍生） | `(motive : Polyrust.Borrow → Polyrust.Borrow → Sort u_1) →   (x x_1 : Polyrust.Borrow) →     ((a a…` |
| 709 | `Polyrust.instReprBorrow` | 實例衍生 | 定義 | `Borrow` 顯示函數的機器衍生助手 | `Repr Polyrust.Borrow` |
| 710 | `Polyrust.instReprBorrow.repr` | 實例衍生 | 定義 | `Borrow` 的顯示函數（Repr 型別類別實例） | `Polyrust.Borrow → Nat → Format` |
| 711 | `Polyrust.BIsIdeal.mul_mem` | 衍生 | 定理 | `BIsIdeal` 相關助手（機器衍生） | `∀ {I : (Polyrust.BSign → Int) → Prop}, Polyrust.BIsIdeal I → ∀ (g a : Polyrust.BSign → Int), I a …` |
| 712 | `Polyrust.BIsIdeal.sub_mem` | 衍生 | 定理 | `BIsIdeal` 相關助手（機器衍生） | `∀ {I : (Polyrust.BSign → Int) → Prop},   Polyrust.BIsIdeal I → ∀ (a b : Polyrust.BSign → Int), I …` |
| 713 | `Polyrust.BIsIdeal.zero_mem` | 衍生 | 定理 | `BIsIdeal` 相關助手（機器衍生） | `∀ {I : (Polyrust.BSign → Int) → Prop}, Polyrust.BIsIdeal I → I fun x => 0` |
| 714 | `Polyrust.Borrow.noConfusionType` | 衍生 | 定義 | `Borrow` 相關助手（機器衍生） | `Sort u → Polyrust.Borrow → Polyrust.Borrow → Sort u` |
| 715 | `Polyrust.Borrow.node` | 衍生 | 定義 | `Borrow` 相關助手（機器衍生） | `Polyrust.Borrow → Nat` |
| 716 | `Polyrust.Borrow.start` | 衍生 | 定義 | `Borrow` 相關助手（機器衍生） | `Polyrust.Borrow → Nat` |
| 717 | `Polyrust.Borrow.stop` | 衍生 | 定義 | `Borrow` 相關助手（機器衍生） | `Polyrust.Borrow → Nat` |
| 718 | `Polyrust.Borrow.varDef` | 衍生 | 定義 | `Borrow` 相關助手（機器衍生） | `Polyrust.Borrow → Nat` |
| 719 | `Polyrust.BorrowAnalysis.assigns` | 衍生 | 定義 | `BorrowAnalysis` 相關助手（機器衍生） | `Polyrust.BorrowAnalysis → List Nat` |
| 720 | `Polyrust.BorrowAnalysis.live` | 衍生 | 定義 | `BorrowAnalysis` 相關助手（機器衍生） | `Polyrust.BorrowAnalysis → List Nat` |
| 721 | `Polyrust.BorrowAnalysis.noConfusionType` | 衍生 | 定義 | `BorrowAnalysis` 相關助手（機器衍生） | `Sort u → Polyrust.BorrowAnalysis → Polyrust.BorrowAnalysis → Sort u` |
| 722 | `Polyrust.BorrowAnalysis.pairs` | 衍生 | 定義 | `BorrowAnalysis` 相關助手（機器衍生） | `Polyrust.BorrowAnalysis → List (Nat × Nat)` |

## `Polyrust.T9Generalized`（83 條）

職責：廣義端到端證明族

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 723 | `Polyrust.IsRootG` | 手寫 | 定義 | σ 是方程組的 0/1 根。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.SigmaG Ty → Prop` |
| 724 | `Polyrust.SigmaG` | 手寫 | 定義 | 位元賦值：每個節點、每個型別一個 0/1 位元。 | `Type → Type` |
| 725 | `Polyrust.TypableG` | 手寫 | 定義 | 可定型（泛化）：存在宇宙中某個型別 τ 使 `tycheck e τ = true`。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Prop` |
| 726 | `Polyrust.cAdd` | 手寫 | 定義 | `add a b` 的規則方程（對每個 t）：`t_{e,t} = numMark t · t_a,numTy · t_b,numTy`。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.Expr → Ty → Polyrust…` |
| 727 | `Polyrust.cEqb` | 手寫 | 定義 | `eqb a b` 的規則方程（對每個 t）：`t_{e,t} = eqbMark t · t_a,numTy · t_b,numTy`。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.Expr → Ty → Polyrust…` |
| 728 | `Polyrust.cIte` | 手寫 | 定義 | `ite c t f` 的規則方程（對每個 τ）：`t_{e,τ} = t_c,eqbTy · t_t,τ · t_f,τ`。 | `{Ty : Type} → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.Expr → Polyrust.Expr → Ty → Polyrust.Si…` |
| 729 | `Polyrust.cNum` | 手寫 | 定義 | `num n` 的規則方程（對每個 t ∈ enumAll）：`t_{e,t} = numMark t`。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Int → Ty → Polyrust.SigmaG Ty → Int` |
| 730 | `Polyrust.eqbMark` | 手寫 | 定義 | 可計算定義 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Ty → Int` |
| 731 | `Polyrust.genCG` | 手寫 | 定義 | 約束生成（泛化）：對每個子節點、每個型別生成規則方程，加 one-hot。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → List (Polyrust.SigmaG Ty → Int)` |
| 732 | `Polyrust.numMark` | 手寫 | 定義 | 型別 t 的「基元標記」：t = numTy 時為 1，否則 0（作為 ℤ）。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Ty → Int` |
| 733 | `Polyrust.numMark_eq` | 手寫 | 定理 | `numMark` 的值就是 `bit (decide (t = numTy))`（定義展開）。 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) {t : Ty},   Polyrust.numMark L t = P…` |
| 734 | `Polyrust.oneHotCG` | 手寫 | 定義 | one-hot 約束（以節點為參數的形式）。 | `{Ty : Type} → Polyrust.Lang Ty → Polyrust.Expr → Polyrust.SigmaG Ty → Int` |
| 735 | `Polyrust.oneHotG` | 手寫 | 定義 | **one-hot（泛化）**：`(Σ_{t ∈ enumAll} bit(σ e t)) − 1 = 0`。 | `{Ty : Type} → Polyrust.Lang Ty → Polyrust.SigmaG Ty → Polyrust.Expr → Int` |
| 736 | `Polyrust.tbG` | 手寫 | 定義 | 節點 e 在型別 τ 上的位元，作為 ℤ 值。 | `{Ty : Type} → Polyrust.SigmaG Ty → Polyrust.Expr → Ty → Int` |
| 737 | `Polyrust.tycheck` | 手寫 | 定義 | `L.numTy`），`ite` 的條件是 `L.eqbTy`、輸出是分支型別。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Ty → Bool` |
| 738 | `Polyrust.witnessG` | 手寫 | 定義 | 見證賦值 σ_D：節點 e' 在位元 τ 上的值就是檢查器的答案 `tycheck e' τ`。 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.SigmaG Ty` |
| 739 | `Polyrust.eqbMark.eq_1` | 等式引理／助手 | 定理 | 定義 `eqbMark` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (t : Ty),   Polyrust.eqbMark L t = P…` |
| 740 | `Polyrust.genCG._f` | 等式引理／助手 | 定義 | `genCG` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   [DecidableEq Ty] → Polyrust.Lang Ty → (x : Polyrust.Expr) → Polyrust.Expr.below x…` |
| 741 | `Polyrust.genCG._sunfold` | 等式引理／助手 | 定義 | `genCG` 的結構遞迴展開引理 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → List (Polyrust.SigmaG Ty → Int)` |
| 742 | `Polyrust.genCG._unsafe_rec` | 等式引理／助手 | 定義 | `genCG` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → List (Polyrust.SigmaG Ty → Int)` |
| 743 | `Polyrust.genCG.eq_def` | 等式引理／助手 | 定理 | 定義 `genCG` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.Expr),   Polyrust.genC…` |
| 744 | `Polyrust.genCG.match_1` | 等式引理／助手 | 定義 | `genCG` 定義編譯時產生的遞迴／匹配助手 | `(motive : Polyrust.Expr → Sort u_1) →   (x : Polyrust.Expr) →     ((n : Int) → motive (Polyrust.E…` |
| 745 | `Polyrust.isMonoAtG_of_root.match_1_3` | 等式引理／助手 | 定義 | `isMonoAtG_of_root` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} {e : Polyrust.Expr} {σ : Polyrust.SigmaG Ty} (τ τ' : Ty) (motive : σ e τ = true ∧ σ…` |
| 746 | `Polyrust.numMark.eq_1` | 等式引理／助手 | 定理 | 定義 `numMark` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (t : Ty),   Polyrust.numMark L t = P…` |
| 747 | `Polyrust.oneHotCG.eq_1` | 等式引理／助手 | 定理 | 定義 `oneHotCG` 的等式引理（定義的展開方程） | `∀ {Ty : Type} (L : Polyrust.Lang Ty) (e : Polyrust.Expr) (σ : Polyrust.SigmaG Ty),   Polyrust.one…` |
| 748 | `Polyrust.tycheck._f` | 等式引理／助手 | 定義 | `tycheck` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   [DecidableEq Ty] →     Polyrust.Lang Ty → (x : Polyrust.Expr) → Polyrust.Expr.bel…` |
| 749 | `Polyrust.tycheck._sunfold` | 等式引理／助手 | 定義 | `tycheck` 的結構遞迴展開引理 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Ty → Bool` |
| 750 | `Polyrust.tycheck._unsafe_rec` | 等式引理／助手 | 定義 | `tycheck` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} → [DecidableEq Ty] → Polyrust.Lang Ty → Polyrust.Expr → Ty → Bool` |
| 751 | `Polyrust.tycheck.eq_1` | 等式引理／助手 | 定理 | 定義 `tycheck` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a : Int),   Polyrust.tyche…` |
| 752 | `Polyrust.tycheck.eq_2` | 等式引理／助手 | 定理 | 定義 `tycheck` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a b : Polyrust.Expr),   Po…` |
| 753 | `Polyrust.tycheck.eq_3` | 等式引理／助手 | 定理 | 定義 `tycheck` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (a b : Polyrust.Expr),   Po…` |
| 754 | `Polyrust.tycheck.eq_4` | 等式引理／助手 | 定理 | 定義 `tycheck` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Ty) (c t f : Polyrust.Expr),   …` |
| 755 | `Polyrust.tycheck.eq_def` | 等式引理／助手 | 定理 | 定義 `tycheck` 的等式引理（定義的展開方程） | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (x : Polyrust.Expr) (x_1 : Ty),   Po…` |
| 756 | `Polyrust.tycheck.match_1` | 等式引理／助手 | 定義 | `tycheck` 定義編譯時產生的遞迴／匹配助手 | `{Ty : Type} →   (motive : Polyrust.Expr → Ty → Sort u_1) →     (x : Polyrust.Expr) →       (x_1 :…` |
| 757 | `Polyrust.tycheck_exclusive.match_1_1` | 等式引理／助手 | 定義 | `tycheck_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (n : Int) (τ τ' : Ty)   (motive :   …` |
| 758 | `Polyrust.tycheck_exclusive.match_1_3` | 等式引理／助手 | 定義 | `tycheck_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.Expr) (τ τ' : Ty)   …` |
| 759 | `Polyrust.tycheck_exclusive.match_1_5` | 等式引理／助手 | 定義 | `tycheck_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (a b : Polyrust.Expr) (τ τ' : Ty)   …` |
| 760 | `Polyrust.tycheck_exclusive.match_1_7` | 等式引理／助手 | 定義 | `tycheck_exclusive` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (c t f : Polyrust.Expr) (τ τ' : Ty) …` |
| 761 | `Polyrust.typable_iff_rootG.match_1_1` | 等式引理／助手 | 定義 | `typable_iff_rootG` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr)   (motive : (∃ σ…` |
| 762 | `Polyrust.untypable_iff_no_rootG.match_1_1` | 等式引理／助手 | 定義 | `untypable_iff_no_rootG` 定義編譯時產生的遞迴／匹配助手 | `∀ {Ty : Type} [inst : DecidableEq Ty] (L : Polyrust.Lang Ty) (e : Polyrust.Expr)   (motive : (∃ σ…` |
| 763 | `Polyrust.Lang._sizeOf_1` | 歸納型衍生 | 定義 | `Lang` 的輔助大小函數 | `{Ty : Type} → [SizeOf Ty] → Polyrust.Lang Ty → Nat` |
| 764 | `Polyrust.Lang._sizeOf_inst` | 歸納型衍生 | 定義 | `Lang` 的輔助大小函數 | `(Ty : Type) → [SizeOf Ty] → SizeOf (Polyrust.Lang Ty)` |
| 765 | `Polyrust.Lang.casesOn` | 歸納型衍生 | 定義 | `Lang` 的案例分析原則 | `{Ty : Type} →   {motive : Polyrust.Lang Ty → Sort u} →     (t : Polyrust.Lang Ty) →       ((enumA…` |
| 766 | `Polyrust.Lang.ctorIdx` | 歸納型衍生 | 定義 | `Lang` 的構造子結構助手（機器衍生） | `{Ty : Type} → Polyrust.Lang Ty → Nat` |
| 767 | `Polyrust.Lang.mk._flat_ctor` | 歸納型衍生 | 定義 | `Lang` 的構造子結構助手（機器衍生） | `{Ty : Type} →   (enumAll : List Ty) →     enumAll.Nodup → (∀ (t : Ty), t ∈ enumAll) → (numTy eqbT…` |
| 768 | `Polyrust.Lang.mk.inj` | 歸納型衍生 | 定理 | `Lang` 的構造子結構助手（機器衍生） | `∀ {Ty : Type} {enumAll : List Ty} {nodup : enumAll.Nodup} {complete : ∀ (t : Ty), t ∈ enumAll} {n…` |
| 769 | `Polyrust.Lang.mk.noConfusion` | 歸納型衍生 | 定義 | `Lang` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{Ty : Type} →   {P : Sort u} →     {enumAll : List Ty} →       {nodup : enumAll.Nodup} →         …` |
| 770 | `Polyrust.Lang.mk.sizeOf_spec` | 歸納型衍生 | 定理 | `Lang` 結構大小的規格命題 | `∀ {Ty : Type} [inst : SizeOf Ty] (enumAll : List Ty) (nodup : enumAll.Nodup) (complete : ∀ (t : T…` |
| 771 | `Polyrust.Lang.noConfusion` | 歸納型衍生 | 定義 | `Lang` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {Ty : Type} →     {t : Polyrust.Lang Ty} →       {Ty' : Type} → {t' : Polyrust.L…` |
| 772 | `Polyrust.Lang.recOn` | 歸納型衍生 | 定義 | `Lang` 的結構遞迴原則 | `{Ty : Type} →   {motive : Polyrust.Lang Ty → Sort u} →     (t : Polyrust.Lang Ty) →       ((enumA…` |
| 773 | `Polyrust.Ty3._sizeOf_1` | 歸納型衍生 | 定義 | `Ty3` 的輔助大小函數 | `Polyrust.Ty3 → Nat` |
| 774 | `Polyrust.Ty3._sizeOf_inst` | 歸納型衍生 | 定義 | `Ty3` 的輔助大小函數 | `SizeOf Polyrust.Ty3` |
| 775 | `Polyrust.Ty3.boolean.elim` | 歸納型衍生 | 定義 | `Ty3` 的構造子消去器 | `{motive : Polyrust.Ty3 → Sort u} → (t : Polyrust.Ty3) → t.ctorIdx = 2 → motive Polyrust.Ty3.boole…` |
| 776 | `Polyrust.Ty3.boolean.sizeOf_spec` | 歸納型衍生 | 定理 | `Ty3` 結構大小的規格命題 | `sizeOf Polyrust.Ty3.boolean = 1` |
| 777 | `Polyrust.Ty3.casesOn` | 歸納型衍生 | 定義 | `Ty3` 的案例分析原則 | `{motive : Polyrust.Ty3 → Sort u} →   (t : Polyrust.Ty3) → motive Polyrust.Ty3.i32 → motive Polyru…` |
| 778 | `Polyrust.Ty3.ctorElim` | 歸納型衍生 | 定義 | `Ty3` 的構造子結構助手（機器衍生） | `{motive : Polyrust.Ty3 → Sort u} →   (ctorIdx : Nat) → (t : Polyrust.Ty3) → ctorIdx = t.ctorIdx →…` |
| 779 | `Polyrust.Ty3.ctorIdx` | 歸納型衍生 | 定義 | `Ty3` 的構造子結構助手（機器衍生） | `Polyrust.Ty3 → Nat` |
| 780 | `Polyrust.Ty3.i32.elim` | 歸納型衍生 | 定義 | `Ty3` 的構造子消去器 | `{motive : Polyrust.Ty3 → Sort u} → (t : Polyrust.Ty3) → t.ctorIdx = 0 → motive Polyrust.Ty3.i32 →…` |
| 781 | `Polyrust.Ty3.i32.sizeOf_spec` | 歸納型衍生 | 定理 | `Ty3` 結構大小的規格命題 | `sizeOf Polyrust.Ty3.i32 = 1` |
| 782 | `Polyrust.Ty3.i64.elim` | 歸納型衍生 | 定義 | `Ty3` 的構造子消去器 | `{motive : Polyrust.Ty3 → Sort u} → (t : Polyrust.Ty3) → t.ctorIdx = 1 → motive Polyrust.Ty3.i64 →…` |
| 783 | `Polyrust.Ty3.i64.sizeOf_spec` | 歸納型衍生 | 定理 | `Ty3` 結構大小的規格命題 | `sizeOf Polyrust.Ty3.i64 = 1` |
| 784 | `Polyrust.Ty3.noConfusion` | 歸納型衍生 | 定義 | `Ty3` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort v✝} → {x y : Polyrust.Ty3} → x = y → Polyrust.Ty3.noConfusionType P x y` |
| 785 | `Polyrust.Ty3.ofNat` | 歸納型衍生 | 定義 | `Ty3` 的自然數字面量實例 | `Nat → Polyrust.Ty3` |
| 786 | `Polyrust.Ty3.recOn` | 歸納型衍生 | 定義 | `Ty3` 的結構遞迴原則 | `{motive : Polyrust.Ty3 → Sort u} →   (t : Polyrust.Ty3) → motive Polyrust.Ty3.i32 → motive Polyru…` |
| 787 | `Polyrust.Ty3.toCtorIdx` | 歸納型衍生 | 定義 | `Ty3` 的構造子結構助手（機器衍生） | `Polyrust.Ty3 → Nat` |
| 788 | `Polyrust.instDecidableEqTy3` | 實例衍生 | 定義 | `Ty3` 可判定相等（DecidableEq）的判定程序 | `DecidableEq Polyrust.Ty3` |
| 789 | `Polyrust.instDecidableEqTy3._proof_1` | 實例衍生 | 定理 | `Ty3` 相等性判定的內部子證明（機器衍生） | `∀ (x y : Polyrust.Ty3), x.ctorIdx = y.ctorIdx → x = y` |
| 790 | `Polyrust.instDecidableEqTy3._proof_2` | 實例衍生 | 定理 | `Ty3` 相等性判定的內部子證明（機器衍生） | `∀ (x y : Polyrust.Ty3), ¬x.ctorIdx = y.ctorIdx → x = y → False` |
| 791 | `Polyrust.instReprTy3` | 實例衍生 | 定義 | `Ty3` 顯示函數的機器衍生助手 | `Repr Polyrust.Ty3` |
| 792 | `Polyrust.instReprTy3.repr` | 實例衍生 | 定義 | `Ty3` 的顯示函數（Repr 型別類別實例） | `Polyrust.Ty3 → Nat → Format` |
| 793 | `Polyrust.instReprTy3.repr.match_1` | 實例衍生 | 定義 | `Ty3` 顯示函數的機器衍生助手 | `(motive : Polyrust.Ty3 → Sort u_1) →   (x : Polyrust.Ty3) →     (Unit → motive Polyrust.Ty3.i32) …` |
| 794 | `Polyrust.Lang.complete` | 衍生 | 定理 | `Lang` 相關助手（機器衍生） | `∀ {Ty : Type} (self : Polyrust.Lang Ty) (t : Ty), t ∈ self.enumAll` |
| 795 | `Polyrust.Lang.enumAll` | 衍生 | 定義 | `Lang` 相關助手（機器衍生） | `{Ty : Type} → Polyrust.Lang Ty → List Ty` |
| 796 | `Polyrust.Lang.eqbTy` | 衍生 | 定義 | `Lang` 相關助手（機器衍生） | `{Ty : Type} → Polyrust.Lang Ty → Ty` |
| 797 | `Polyrust.Lang.noConfusionType` | 衍生 | 定義 | `Lang` 相關助手（機器衍生） | `Sort u → {Ty : Type} → Polyrust.Lang Ty → {Ty' : Type} → Polyrust.Lang Ty' → Sort u` |
| 798 | `Polyrust.Lang.nodup` | 衍生 | 定理 | `Lang` 相關助手（機器衍生） | `∀ {Ty : Type} (self : Polyrust.Lang Ty), self.enumAll.Nodup` |
| 799 | `Polyrust.Lang.numTy` | 衍生 | 定義 | `Lang` 相關助手（機器衍生） | `{Ty : Type} → Polyrust.Lang Ty → Ty` |
| 800 | `Polyrust.Lang.num_ne_eqb` | 衍生 | 定理 | `Lang` 相關助手（機器衍生） | `∀ {Ty : Type} (self : Polyrust.Lang Ty), self.numTy ≠ self.eqbTy` |
| 801 | `Polyrust.Ty3.ctorElimType` | 衍生 | 定義 | `Ty3` 相關助手（機器衍生） | `{motive : Polyrust.Ty3 → Sort u} → Nat → Sort (max 1 u)` |
| 802 | `Polyrust.Ty3.noConfusionType` | 衍生 | 定義 | `Ty3` 相關助手（機器衍生） | `Sort v✝ → Polyrust.Ty3 → Polyrust.Ty3 → Sort v✝` |
| 803 | `Polyrust.Ty3.ofNat_ctorIdx` | 衍生 | 定理 | `Ty3` 相關助手（機器衍生） | `∀ (x : Polyrust.Ty3), Polyrust.Ty3.ofNat x.ctorIdx = x` |
| 804 | `Polyrust.threeTypeLang._proof_3` | 衍生 | 定理 | `threeTypeLang` 相關助手（機器衍生） | `Polyrust.Ty3.i32 = Polyrust.Ty3.boolean → False` |
| 805 | `Polyrust.twoTypeLang._proof_3` | 衍生 | 定理 | `twoTypeLang` 相關助手（機器衍生） | `Polyrust.Ty.i32 = Polyrust.Ty.boolean → False` |

## `Polyrust.ClauseDuality`（34 條）

職責：子句對偶與互補

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 806 | `Polyrust.Assignment` | 手寫 | 定義 | 布爾賦值 σ : 變量 → {0,1}。 | `Type` |
| 807 | `Polyrust.bit` | 手寫 | 定義 | 布爾值 → 整數（{0,1} 嵌入 ℤ；係數域 𝔽_p 的代表元）。 | `Bool → Int` |
| 808 | `Polyrust.bit_eq_one` | 手寫 | 定理 | 等式／恆等命題 | `∀ (b : Bool), b = true → Polyrust.bit b = 1` |
| 809 | `Polyrust.bit_eq_zero` | 手寫 | 定理 | 等式／恆等命題 | `∀ (b : Bool), b = false → Polyrust.bit b = 0` |
| 810 | `Polyrust.clausePoly` | 手寫 | 定義 | 子句多項式 P_C = ∏ (1 − x̃ᵢ) 在賦值 σ 處的值。 | `Polyrust.Assignment → List Polyrust.Lit → Int` |
| 811 | `Polyrust.clauseSat` | 手寫 | 定義 | σ 滿足子句 C ⟺ 某文字為真（存在量詞的布爾編碼）。 | `Polyrust.Assignment → List Polyrust.Lit → Bool` |
| 812 | `Polyrust.cnfPolys` | 手寫 | 定義 | CNF 公式的多項式組 P_Φ 在 σ 處的值列表。 | `Polyrust.Assignment → List (List Polyrust.Lit) → List Int` |
| 813 | `Polyrust.cnfSat` | 手寫 | 定義 | CNF 公式 Φ 的滿足性。 | `Polyrust.Assignment → List (List Polyrust.Lit) → Bool` |
| 814 | `Polyrust.field_poly_bit` | 手寫 | 定理 | **域多項式引理**：x² − x 在 0/1 賦值下恆為零—— 這正是布爾解空間 {0,1}ⁿ 的代數刻畫（理想 B 的作用）。 | `∀ (b : Bool), Polyrust.bit b * (Polyrust.bit b - 1) = 0` |
| 815 | `Polyrust.litFactor` | 手寫 | 定義 | 文字 ℓ 的多項式因子 (1 − x̃ℓ) 在賦值 σ 處的值。 | `Polyrust.Assignment → Polyrust.Lit → Int` |
| 816 | `Polyrust.litSat` | 手寫 | 定義 | σ 滿足文字 ℓ：正文字看 σᵢ，負文字看 ¬σᵢ。 | `Polyrust.Assignment → Polyrust.Lit → Bool` |
| 817 | `Polyrust.Lit.neg.eq_1` | 等式引理／助手 | 定理 | 定義 `Lit` 的等式引理（定義的展開方程） | `∀ (l : Polyrust.Lit), l.neg = { var := l.var, pos := !l.pos }` |
| 818 | `Polyrust.bit.eq_1` | 等式引理／助手 | 定理 | 定義 `bit` 的等式引理（定義的展開方程） | `∀ (b : Bool), Polyrust.bit b = if b = true then 1 else 0` |
| 819 | `Polyrust.clausePoly.eq_1` | 等式引理／助手 | 定理 | 定義 `clausePoly` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Assignment) (C : List Polyrust.Lit),   Polyrust.clausePoly σ C = List.foldr (fun …` |
| 820 | `Polyrust.clauseSat.eq_1` | 等式引理／助手 | 定理 | 定義 `clauseSat` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Assignment) (C : List Polyrust.Lit), Polyrust.clauseSat σ C = C.any (Polyrust.lit…` |
| 821 | `Polyrust.cnfPolys.eq_1` | 等式引理／助手 | 定理 | 定義 `cnfPolys` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Assignment) (Φ : List (List Polyrust.Lit)), Polyrust.cnfPolys σ Φ = List.map (Pol…` |
| 822 | `Polyrust.cnfSat.eq_1` | 等式引理／助手 | 定理 | 定義 `cnfSat` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Assignment) (Φ : List (List Polyrust.Lit)), Polyrust.cnfSat σ Φ = Φ.all (Polyrust…` |
| 823 | `Polyrust.litFactor.eq_1` | 等式引理／助手 | 定理 | 定義 `litFactor` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit), Polyrust.litFactor σ l = 1 - Polyrust.bit (Polyru…` |
| 824 | `Polyrust.litSat.eq_1` | 等式引理／助手 | 定理 | 定義 `litSat` 的等式引理（定義的展開方程） | `∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit), Polyrust.litSat σ l = if l.pos = true then σ l.va…` |
| 825 | `Polyrust.Lit._sizeOf_1` | 歸納型衍生 | 定義 | `Lit` 的輔助大小函數 | `Polyrust.Lit → Nat` |
| 826 | `Polyrust.Lit._sizeOf_inst` | 歸納型衍生 | 定義 | `Lit` 的輔助大小函數 | `SizeOf Polyrust.Lit` |
| 827 | `Polyrust.Lit.casesOn` | 歸納型衍生 | 定義 | `Lit` 的案例分析原則 | `{motive : Polyrust.Lit → Sort u} →   (t : Polyrust.Lit) → ((var : Nat) → (pos : Bool) → motive { …` |
| 828 | `Polyrust.Lit.ctorIdx` | 歸納型衍生 | 定義 | `Lit` 的構造子結構助手（機器衍生） | `Polyrust.Lit → Nat` |
| 829 | `Polyrust.Lit.mk._flat_ctor` | 歸納型衍生 | 定義 | `Lit` 的構造子結構助手（機器衍生） | `Nat → Bool → Polyrust.Lit` |
| 830 | `Polyrust.Lit.mk.inj` | 歸納型衍生 | 定理 | `Lit` 的構造子結構助手（機器衍生） | `∀ {var : Nat} {pos : Bool} {var_1 : Nat} {pos_1 : Bool},   { var := var, pos := pos } = { var := …` |
| 831 | `Polyrust.Lit.mk.noConfusion` | 歸納型衍生 | 定義 | `Lit` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} →   {var : Nat} →     {pos : Bool} →       {var' : Nat} →         {pos' : Bool} → { …` |
| 832 | `Polyrust.Lit.mk.sizeOf_spec` | 歸納型衍生 | 定理 | `Lit` 結構大小的規格命題 | `∀ (var : Nat) (pos : Bool), sizeOf { var := var, pos := pos } = 1 + sizeOf var + sizeOf pos` |
| 833 | `Polyrust.Lit.noConfusion` | 歸納型衍生 | 定義 | `Lit` 構造子一致性：不同構造子互不相等、同構造子則參數相等 | `{P : Sort u} → {t t' : Polyrust.Lit} → t = t' → Polyrust.Lit.noConfusionType P t t'` |
| 834 | `Polyrust.Lit.recOn` | 歸納型衍生 | 定義 | `Lit` 的結構遞迴原則 | `{motive : Polyrust.Lit → Sort u} →   (t : Polyrust.Lit) → ((var : Nat) → (pos : Bool) → motive { …` |
| 835 | `Polyrust.instReprLit` | 實例衍生 | 定義 | `Lit` 顯示函數的機器衍生助手 | `Repr Polyrust.Lit` |
| 836 | `Polyrust.instReprLit.repr` | 實例衍生 | 定義 | `Lit` 的顯示函數（Repr 型別類別實例） | `Polyrust.Lit → Nat → Format` |
| 837 | `Polyrust.Lit.noConfusionType` | 衍生 | 定義 | `Lit` 相關助手（機器衍生） | `Sort u → Polyrust.Lit → Polyrust.Lit → Sort u` |
| 838 | `Polyrust.Lit.pos` | 衍生 | 定義 | `Lit` 相關助手（機器衍生） | `Polyrust.Lit → Bool` |
| 839 | `Polyrust.Lit.var` | 衍生 | 定義 | `Lit` 相關助手（機器衍生） | `Polyrust.Lit → Nat` |

## `Polyrust.UniPoly`（26 條）

職責：單變量多項式運算

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 840 | `Polyrust.UniPoly` | 手寫 | 定義 | 升冪係數的一元整係數多項式（與 Rust 版 `qap.rs` 的 `UniPoly` 同構設計）。 | `Type` |
| 841 | `Polyrust.UniPoly.XsubC` | 手寫 | 定義 | X − a：monic 線性因式。 | `Int → Polyrust.UniPoly` |
| 842 | `Polyrust.UniPoly.add` | 手寫 | 定義 | 可計算定義 | `Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly` |
| 843 | `Polyrust.UniPoly.eval` | 手寫 | 定義 | 在點 t 處求值（Horner 形式）。 | `Polyrust.UniPoly → Int → Int` |
| 844 | `Polyrust.UniPoly.negP` | 手寫 | 定義 | 可計算定義 | `Polyrust.UniPoly → Polyrust.UniPoly` |
| 845 | `Polyrust.UniPoly.shift` | 手寫 | 定義 | 可計算定義 | `Polyrust.UniPoly → Polyrust.UniPoly` |
| 846 | `Polyrust.UniPoly.smulC` | 手寫 | 定義 | 可計算定義 | `Int → Polyrust.UniPoly → Polyrust.UniPoly` |
| 847 | `Polyrust.UniPoly.sub` | 手寫 | 定義 | 可計算定義 | `Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly` |
| 848 | `Polyrust.UniPoly.add._f` | 等式引理／助手 | 定義 | `UniPoly` 定義編譯時產生的遞迴／匹配助手 | `(x : Polyrust.UniPoly) →   List.below (motive := fun x => Polyrust.UniPoly → Polyrust.UniPoly) x …` |
| 849 | `Polyrust.UniPoly.add._sunfold` | 等式引理／助手 | 定義 | `UniPoly` 的結構遞迴展開引理 | `Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly` |
| 850 | `Polyrust.UniPoly.add._unsafe_rec` | 等式引理／助手 | 定義 | `UniPoly` 定義編譯時產生的遞迴／匹配助手 | `Polyrust.UniPoly → Polyrust.UniPoly → Polyrust.UniPoly` |
| 851 | `Polyrust.UniPoly.add.eq_1` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.UniPoly), Polyrust.UniPoly.add [] x = x` |
| 852 | `Polyrust.UniPoly.add.eq_2` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.UniPoly), (x = [] → False) → x.add [] = x` |
| 853 | `Polyrust.UniPoly.add.eq_3` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (a : Int) (as : List Int) (b : Int) (bs : List Int),   Polyrust.UniPoly.add (a :: as) (b :: bs)…` |
| 854 | `Polyrust.UniPoly.add.eq_def` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (x x_1 : Polyrust.UniPoly),   x.add x_1 =     match x, x_1 with     \| [], q => q     \| p, [] =>…` |
| 855 | `Polyrust.UniPoly.add.match_1` | 等式引理／助手 | 定義 | `UniPoly` 定義編譯時產生的遞迴／匹配助手 | `(motive : Polyrust.UniPoly → Polyrust.UniPoly → Sort u_1) →   (x x_1 : Polyrust.UniPoly) →     ((…` |
| 856 | `Polyrust.UniPoly.eval._f` | 等式引理／助手 | 定義 | `UniPoly` 定義編譯時產生的遞迴／匹配助手 | `(x : Polyrust.UniPoly) → List.below (motive := fun x => Int → Int) x → Int → Int` |
| 857 | `Polyrust.UniPoly.eval._sunfold` | 等式引理／助手 | 定義 | `UniPoly` 的結構遞迴展開引理 | `Polyrust.UniPoly → Int → Int` |
| 858 | `Polyrust.UniPoly.eval._unsafe_rec` | 等式引理／助手 | 定義 | `UniPoly` 定義編譯時產生的遞迴／匹配助手 | `Polyrust.UniPoly → Int → Int` |
| 859 | `Polyrust.UniPoly.eval.eq_1` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (x : Int), Polyrust.UniPoly.eval [] x = 0` |
| 860 | `Polyrust.UniPoly.eval.eq_2` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (x c : Int) (cs : List Int), Polyrust.UniPoly.eval (c :: cs) x = c + x * Polyrust.UniPoly.eval …` |
| 861 | `Polyrust.UniPoly.eval.eq_def` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.UniPoly) (x_1 : Int),   x.eval x_1 =     match x, x_1 with     \| [], x => 0     \|…` |
| 862 | `Polyrust.UniPoly.eval.match_1` | 等式引理／助手 | 定義 | `UniPoly` 定義編譯時產生的遞迴／匹配助手 | `(motive : Polyrust.UniPoly → Int → Sort u_1) →   (x : Polyrust.UniPoly) →     (x_1 : Int) →      …` |
| 863 | `Polyrust.UniPoly.negP.eq_1` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (p : Polyrust.UniPoly), p.negP = List.map (fun c => -c) p` |
| 864 | `Polyrust.UniPoly.smulC.eq_1` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (c : Int) (p : Polyrust.UniPoly), Polyrust.UniPoly.smulC c p = List.map (fun x => c * x) p` |
| 865 | `Polyrust.UniPoly.sub.eq_1` | 等式引理／助手 | 定理 | 定義 `UniPoly` 的等式引理（定義的展開方程） | `∀ (p q : Polyrust.UniPoly), p.sub q = p.add q.negP` |

## `Polyrust.Monomial`（19 條）

職責：單項式次序與乘法

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 866 | `Polyrust.MonoExp` | 手寫 | 定義 | 單項式 = 指數向量（變量索引從 0 起）。 | `Type` |
| 867 | `Polyrust.coprimeM` | 手寫 | 定義 | 互素：任何維度上不同時為正（gcd = 1，含「無公共變量」）。 | `Polyrust.MonoExp → Polyrust.MonoExp → Prop` |
| 868 | `Polyrust.coprimeM_comm` | 手寫 | 定理 | 命題證明 | `∀ {a b : Polyrust.MonoExp}, Polyrust.coprimeM a b → Polyrust.coprimeM b a` |
| 869 | `Polyrust.dividesM` | 手寫 | 定義 | 單項式整除：逐點指數 ≤。 | `Polyrust.MonoExp → Polyrust.MonoExp → Prop` |
| 870 | `Polyrust.dividesM_monoMul_right` | 手寫 | 定理 | 命題證明 | `∀ (a b : Polyrust.MonoExp), Polyrust.dividesM b (Polyrust.monoMul a b)` |
| 871 | `Polyrust.dividesM_monoOne` | 手寫 | 定理 | 命題證明 | `∀ (a : Polyrust.MonoExp), Polyrust.dividesM Polyrust.monoOne a` |
| 872 | `Polyrust.dividesM_refl` | 手寫 | 定理 | ! ## 整除的基本性質 | `∀ (a : Polyrust.MonoExp), Polyrust.dividesM a a` |
| 873 | `Polyrust.dividesM_trans` | 手寫 | 定理 | 命題證明 | `∀ {a b c : Polyrust.MonoExp}, Polyrust.dividesM a b → Polyrust.dividesM b c → Polyrust.dividesM a c` |
| 874 | `Polyrust.gcdM` | 手寫 | 定義 | 最大公因 gcd = 逐點 min。 | `Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp` |
| 875 | `Polyrust.lcmM` | 手寫 | 定義 | 最小公倍 lcm = 逐點 max。 | `Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp` |
| 876 | `Polyrust.monoMul` | 手寫 | 定義 | 單項式乘法 = 指數相加。 | `Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp` |
| 877 | `Polyrust.monoOne` | 手寫 | 定義 | 單位單項式 1（全零指數向量）。 | `Polyrust.MonoExp` |
| 878 | `Polyrust.monoOne_squarefree` | 手寫 | 定理 | 命題證明 | `Polyrust.squarefreeM Polyrust.monoOne` |
| 879 | `Polyrust.quotM` | 手寫 | 定義 | 商：`quotM a b = b / a`（逐點減法；要求 `a ∣ b` 才有意義）。 | `Polyrust.MonoExp → Polyrust.MonoExp → Polyrust.MonoExp` |
| 880 | `Polyrust.squarefreeM` | 手寫 | 定義 | 平方自由單項式：每個指數 ≤ 1。 | `Polyrust.MonoExp → Prop` |
| 881 | `Polyrust.x1` | 手寫 | 定義 | 變量 xᵢ 的指數向量。 | `Nat → Polyrust.MonoExp` |
| 882 | `Polyrust.x2` | 手寫 | 定義 | xᵢ² 的指數向量（域多項式 xᵢ² − xᵢ 的首項）。 | `Nat → Polyrust.MonoExp` |
| 883 | `Polyrust.x1.eq_1` | 等式引理／助手 | 定理 | 定義 `x1` 的等式引理（定義的展開方程） | `∀ (i j : Nat), Polyrust.x1 i j = if j = i then 1 else 0` |
| 884 | `Polyrust.x2.eq_1` | 等式引理／助手 | 定理 | 定義 `x2` 的等式引理（定義的展開方程） | `∀ (i j : Nat), Polyrust.x2 i j = if j = i then 2 else 0` |

## `Polyrust.Squarefree`（17 條）

職責：平方自由化與 allBits 窮舉

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 885 | `Polyrust.allBits` | 手寫 | 定義 | n 維位串的全枚舉（長度 2ⁿ）。 | `Nat → List (Nat → Bool)` |
| 886 | `Polyrust.ofBits` | 手寫 | 定義 | ! ## 環節二：平方自由單項式 ↔ 位串 位串 → 平方自由單項式：出現（1）或不出現（0）。 | `(Nat → Bool) → Polyrust.MonoExp` |
| 887 | `Polyrust.setN` | 手寫 | 定義 | 在位置 i 設定位元。 | `(Nat → Bool) → Nat → Bool → Nat → Bool` |
| 888 | `Polyrust.supportLe` | 手寫 | 定義 | ! ## 環節三：n 維位串的枚舉與計數 位串的 n 維支撐條件：第 n 位以後恆 false。 | `(Nat → Bool) → Nat → Prop` |
| 889 | `Polyrust.supportLeM` | 手寫 | 定義 | 單項式的 n 維支撐條件。 | `Polyrust.MonoExp → Nat → Prop` |
| 890 | `Polyrust.supportLe_nth_false` | 手寫 | 定理 | 支撐 ≤ n 的位串，第 n 位必為假。 | `∀ {f : Nat → Bool} {n : Nat}, Polyrust.supportLe f n → f n = false` |
| 891 | `Polyrust.toBits` | 手寫 | 定義 | 平方自由單項式 → 位串：指數是否為 1。 | `Polyrust.MonoExp → Nat → Bool` |
| 892 | `Polyrust.allBits._f` | 等式引理／助手 | 定義 | `allBits` 定義編譯時產生的遞迴／匹配助手 | `(x : Nat) → Nat.below x → List (Nat → Bool)` |
| 893 | `Polyrust.allBits._sunfold` | 等式引理／助手 | 定義 | `allBits` 的結構遞迴展開引理 | `Nat → List (Nat → Bool)` |
| 894 | `Polyrust.allBits._unsafe_rec` | 等式引理／助手 | 定義 | `allBits` 定義編譯時產生的遞迴／匹配助手 | `Nat → List (Nat → Bool)` |
| 895 | `Polyrust.allBits.eq_1` | 等式引理／助手 | 定理 | 定義 `allBits` 的等式引理（定義的展開方程） | `Polyrust.allBits 0 = [fun x => false]` |
| 896 | `Polyrust.allBits.eq_2` | 等式引理／助手 | 定理 | 定義 `allBits` 的等式引理（定義的展開方程） | `∀ (n : Nat),   Polyrust.allBits n.succ =     List.flatMap (fun f => [Polyrust.setN f n false, Pol…` |
| 897 | `Polyrust.allBits.eq_def` | 等式引理／助手 | 定理 | 定義 `allBits` 的等式引理（定義的展開方程） | `∀ (x : Nat),   Polyrust.allBits x =     match x with     \| 0 => [fun x => false]     \| n.succ => …` |
| 898 | `Polyrust.allBits.match_1` | 等式引理／助手 | 定義 | `allBits` 定義編譯時產生的遞迴／匹配助手 | `(motive : Nat → Sort u_1) → (x : Nat) → (Unit → motive 0) → ((n : Nat) → motive n.succ) → motive x` |
| 899 | `Polyrust.ofBits.eq_1` | 等式引理／助手 | 定理 | 定義 `ofBits` 的等式引理（定義的展開方程） | `∀ (f : Nat → Bool) (j : Nat), Polyrust.ofBits f j = if f j = true then 1 else 0` |
| 900 | `Polyrust.setN.eq_1` | 等式引理／助手 | 定理 | 定義 `setN` 的等式引理（定義的展開方程） | `∀ (f : Nat → Bool) (i : Nat) (b : Bool) (j : Nat), Polyrust.setN f i b j = if j = i then b else f j` |
| 901 | `Polyrust.toBits.eq_1` | 等式引理／助手 | 定理 | 定義 `toBits` 的等式引理（定義的展開方程） | `∀ (m : Polyrust.MonoExp) (j : Nat), Polyrust.toBits m j = (m j == 1)` |

## `Polyrust.T6Certificate`（17 條）

職責：插值憑證層

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 902 | `Polyrust.F` | 手寫 | 定義 | ! ## 二、布爾方體上的多項式函數與插值 布爾點上的 ℤ 值函數。 | `Type` |
| 903 | `Polyrust.NoCommonZero` | 手寫 | 定義 | 「無公共零點」表示每個點 x 都有某個系統元素 f 使 f(x) ≠ 0。 系統 fs 在點集 pts 上無公共零點。 | `List (Nat → Bool) → List Polyrust.F → Prop` |
| 904 | `Polyrust.PolyFn.one` | 手寫 | 定理 | 等式／恆等命題 | `∀ {n : Nat}, Polyrust.PolyFn n fun x => 1` |
| 905 | `Polyrust.PolyFn.smulConst` | 手寫 | 定理 | 等式／恆等命題 | `∀ {n : Nat} {f : Polyrust.F} (c : Int), Polyrust.PolyFn n f → Polyrust.PolyFn n fun x => c * f x` |
| 906 | `Polyrust.PolyFn.zero` | 手寫 | 定理 | 等式／恆等命題 | `∀ {n : Nat}, Polyrust.PolyFn n fun x => 0` |
| 907 | `Polyrust.coord` | 手寫 | 定義 | 座標函數（第 j 個變量的 0/1 取值）。 | `Nat → Polyrust.F` |
| 908 | `Polyrust.delta` | 手寫 | 定義 | ! ## 三、拉格朗日基 δ_σ 及其性質 拉格朗日基（指標函數）：δ n σ 在 σ 上取值 1、在其他（支撐 ≤ n 的）點取值 0。 以低 n 個座標的因子乘積遞歸構造。 | `Nat → (Nat → Bool) → Polyrust.F` |
| 909 | `Polyrust.interp` | 手寫 | 定義 | 以分拆單位把任意函數「插值」為多項式函數。 | `Nat → List (Nat → Bool) → Polyrust.F → Polyrust.F` |
| 910 | `Polyrust.delta._f` | 等式引理／助手 | 定義 | `delta` 定義編譯時產生的遞迴／匹配助手 | `(x : Nat) → Nat.below (motive := fun x => (Nat → Bool) → Polyrust.F) x → (Nat → Bool) → Polyrust.F` |
| 911 | `Polyrust.delta._sunfold` | 等式引理／助手 | 定義 | `delta` 的結構遞迴展開引理 | `Nat → (Nat → Bool) → Polyrust.F` |
| 912 | `Polyrust.delta._unsafe_rec` | 等式引理／助手 | 定義 | `delta` 定義編譯時產生的遞迴／匹配助手 | `Nat → (Nat → Bool) → Polyrust.F` |
| 913 | `Polyrust.delta.match_1` | 等式引理／助手 | 定義 | `delta` 定義編譯時產生的遞迴／匹配助手 | `(motive : Nat → (Nat → Bool) → Sort u_1) →   (x : Nat) →     (x_1 : Nat → Bool) →       ((x : Nat…` |
| 914 | `Polyrust.interp.eq_1` | 等式引理／助手 | 定理 | 定義 `interp` 的等式引理（定義的展開方程） | `∀ (n : Nat) (pts : List (Nat → Bool)) (g : Polyrust.F) (x : Nat → Bool),   Polyrust.interp n pts …` |
| 915 | `Polyrust.PolyFn.below.casesOn` | 歸納型衍生 | 定義 | `PolyFn` 的案例分析原則 | `∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop}   {motive_1 : {a : Polyrust.…` |
| 916 | `Polyrust.PolyFn.brecOn` | 歸納型衍生 | 定理 | `PolyFn` 的良基遞迴助手 | `∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop} {a : Polyrust.F} (t : Polyru…` |
| 917 | `Polyrust.PolyFn.casesOn` | 歸納型衍生 | 定義 | `PolyFn` 的案例分析原則 | `∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop} {a : Polyrust.F} (t : Polyru…` |
| 918 | `Polyrust.PolyFn.recOn` | 歸納型衍生 | 定義 | `PolyFn` 的結構遞迴原則 | `∀ {n : Nat} {motive : (a : Polyrust.F) → Polyrust.PolyFn n a → Prop} {a : Polyrust.F} (t : Polyru…` |

## `Polyrust.MicroInstance`（13 條）

職責：微實例語言核心：表達式／詞元／類型系統

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 919 | `Polyrust.T1_micro` | 手寫 | 定理 | **T1 微實例（可靠性）**：推導 D 的位元賦值 σ_D 是方程組的根。 | `Polyrust.addPoly1 true true true = 0 ∧ Polyrust.addPoly2 true true true = 0` |
| 920 | `Polyrust.T6_micro_sat` | 手寫 | 定理 | **T6 微實例（SAT 側）**：無上下文衝突時系統可解（σ = 全真）。 | `∃ tx ty tr, Polyrust.addPoly1 tx ty tr = 0 ∧ Polyrust.addPoly2 tx ty tr = 0` |
| 921 | `Polyrust.T7_arm_sat` | 手寫 | 定理 | **T7 微實例（exists-arm 可解）**：對任意 x 的類型，總存在臂選擇 使系統可解（宏總能定型）——臂選擇的存在量化語義。 | `∀ (tx : Bool), ∃ a, Polyrust.armPoly a tx true = 0 ∧ Polyrust.arm2PolyA a tx true = 0 ∧ Polyrust.…` |
| 922 | `Polyrust.addPoly1` | 手寫 | 定義 | - t_x − t_y = 0（操作數類型一致） 規則方程 1：t_x + t_y − t_r − 1。 | `Bool → Bool → Bool → Int` |
| 923 | `Polyrust.addPoly2` | 手寫 | 定義 | 規則方程 2：t_x − t_y。 | `Bool → Bool → Bool → Int` |
| 924 | `Polyrust.arm2PolyA` | 手寫 | 定義 | 臂 2 約束 A：(1−a)·t_x。 | `Bool → Bool → Bool → Int` |
| 925 | `Polyrust.arm2PolyB` | 手寫 | 定義 | 臂 2 約束 B：(1−a)·(1−t_r)。 | `Bool → Bool → Bool → Int` |
| 926 | `Polyrust.armPoly` | 手寫 | 定義 | - 臂 2：(1−a)·t_x = 0（x : bool）∧ (1−a)·(1−t_r) = 0（輸出 bool） 臂 1 約束：a·(t_x − t_r)。 | `Bool → Bool → Bool → Int` |
| 927 | `Polyrust.wellTypedAdd` | 手寫 | 定義 | 良構性：推導存在（兩操作數 i32、結果 i32）。 | `Bool → Bool → Bool → Prop` |
| 928 | `Polyrust.addPoly1.eq_1` | 等式引理／助手 | 定理 | 定義 `addPoly1` 的等式引理（定義的展開方程） | `∀ (tx ty tr : Bool), Polyrust.addPoly1 tx ty tr = Polyrust.bit tx + Polyrust.bit ty - Polyrust.bi…` |
| 929 | `Polyrust.addPoly2.eq_1` | 等式引理／助手 | 定理 | 定義 `addPoly2` 的等式引理（定義的展開方程） | `∀ (tx ty _tr : Bool), Polyrust.addPoly2 tx ty _tr = Polyrust.bit tx - Polyrust.bit ty` |
| 930 | `Polyrust.armPoly.eq_1` | 等式引理／助手 | 定理 | 定義 `armPoly` 的等式引理（定義的展開方程） | `∀ (a tx tr : Bool), Polyrust.armPoly a tx tr = Polyrust.bit a * (Polyrust.bit tx - Polyrust.bit tr)` |
| 931 | `Polyrust.wellTypedAdd.eq_1` | 等式引理／助手 | 定理 | 定義 `wellTypedAdd` 的等式引理（定義的展開方程） | `∀ (tx ty tr : Bool), Polyrust.wellTypedAdd tx ty tr = (tx = true ∧ ty = true ∧ tr = true)` |

## `Polyrust.ClauseAlgebra`（11 條）

職責：子句代數：滿足性保持的代數變換

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 932 | `Polyrust.Entails` | 手寫 | 定義 | 子句被 CNF 蘊涵：Φ 的每個模型都滿足 C。 | `List (List Polyrust.Lit) → List Polyrust.Lit → Prop` |
| 933 | `Polyrust.Lit.neg` | 手寫 | 定義 | ! ## 文字的補與因子 文字的否定：同一變量、相反極性。 | `Polyrust.Lit → Polyrust.Lit` |
| 934 | `Polyrust.Lit.neg_neg` | 手寫 | 定理 | 等式／恆等命題 | `∀ (l : Polyrust.Lit), l.neg.neg = l` |
| 935 | `Polyrust.Lit.neg_var` | 手寫 | 定理 | 等式／恆等命題 | `∀ (l : Polyrust.Lit), l.neg.var = l.var` |
| 936 | `Polyrust.Mod` | 手寫 | 定義 | ! ## CNF 層：模型集與學習步的守恆 CNF 模型集：σ 滿足 Φ 的所有子句。 | `Polyrust.Assignment → List (List Polyrust.Lit) → Prop` |
| 937 | `Polyrust.PolyZero` | 手寫 | 定義 | CNF 的多項式零點集：Φ 的全部子句多項式在 σ 處歸零。 | `Polyrust.Assignment → List (List Polyrust.Lit) → Prop` |
| 938 | `Polyrust.bit_add_bit_not` | 手寫 | 定理 | 布爾取值的「0/1 互補」：`bit b + bit (!b) = 1`。 | `∀ (b : Bool), (Polyrust.bit b + Polyrust.bit !b) = 1` |
| 939 | `Polyrust.clausePoly_cons` | 手寫 | 定理 | 等式／恆等命題 | `∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit) (C : List Polyrust.Lit),   Polyrust.clausePoly σ (…` |
| 940 | `Polyrust.clausePoly_nil` | 手寫 | 定理 | 空子句的多項式是常數 1：它「永不歸零」——U N S A T 的算術面貌。 | `∀ {σ : Polyrust.Assignment}, Polyrust.clausePoly σ [] = 1` |
| 941 | `Polyrust.clausePoly_nil_ne_zero` | 手寫 | 定理 | 命題證明 | `∀ (σ : Polyrust.Assignment), Polyrust.clausePoly σ [] ≠ 0` |
| 942 | `Polyrust.litFactor_eq_one_sub` | 手寫 | 定理 | 等式／恆等命題 | `∀ (σ : Polyrust.Assignment) (l : Polyrust.Lit), Polyrust.litFactor σ l = 1 - Polyrust.bit (Polyru…` |

## `Polyrust.SPoly`（8 條）

職責：S-多項式與 Gröbner 步

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 943 | `Polyrust.IsLead` | 手寫 | 定義 | μ 是 p 的首項單項式：係數非零，且所有非零項都被 μ 整除。 | `Polyrust.MonoExp → Polyrust.MPoly → Prop` |
| 944 | `Polyrust.MPoly` | 手寫 | 定義 | ! ## 多項式（係數函數）與理想 多項式：單項式 → 係數（本層只用到支撐、首項與理想封閉性， 故不強制有限支撐；所有定理都對「支撐」逐項陳述）。 | `Type` |
| 945 | `Polyrust.addP` | 手寫 | 定義 | 可計算定義 | `Polyrust.MPoly → Polyrust.MPoly → Polyrust.MPoly` |
| 946 | `Polyrust.negP` | 手寫 | 定義 | 可計算定義 | `Polyrust.MPoly → Polyrust.MPoly` |
| 947 | `Polyrust.subP` | 手寫 | 定義 | 可計算定義 | `Polyrust.MPoly → Polyrust.MPoly → Polyrust.MPoly` |
| 948 | `Polyrust.zeroP` | 手寫 | 定義 | 可計算定義 | `Polyrust.MPoly` |
| 949 | `Polyrust.subP.eq_1` | 等式引理／助手 | 定理 | 定義 `subP` 的等式引理（定義的展開方程） | `∀ (p q : Polyrust.MPoly) (m : Polyrust.MonoExp), Polyrust.subP p q m = p m - q m` |
| 950 | `Polyrust.zeroP.eq_1` | 等式引理／助手 | 定理 | 定義 `zeroP` 的等式引理（定義的展開方程） | `∀ (x : Polyrust.MonoExp), Polyrust.zeroP x = 0` |

## `Polyrust.Canonical`（7 條）

職責：正規形與規範化

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 951 | `Polyrust.FullyReduced` | 手寫 | 定義 | 完全約化：p 的所有非零項都不落在 W 內。 | `(Polyrust.MonoExp → Prop) → Polyrust.MPoly → Prop` |
| 952 | `Polyrust.HasMaxMono` | 手寫 | 定義 | 「極大項存在」：任何非零多項式都有對整除極大的項。 對有限支撐多項式在與整除相容的項序下皆成立（見 `hmax_pureNat`）。 | `Polyrust.MPoly → Prop` |
| 953 | `Polyrust.Interreduced` | 手寫 | 定義 | ! ## 互約化與完全約化 互約化：μ 是 p 的首項，且 p 的所有**非首項**都不落在 W 內。 （約化 Gröbner 基的條件：任一基元素的任何項都不被其他基元素的首項整除。） | `(Polyrust.MonoExp → Prop) → Polyrust.MonoExp → Polyrust.MPoly → Prop` |
| 954 | `Polyrust.leadIdeal` | 手寫 | 定義 | ! ## 首項理想 理想 I 的首項理想 in(I)：m 是 I 中某元素的**首項單項式**。 | `(Polyrust.MPoly → Prop) → Polyrust.MonoExp → Prop` |
| 955 | `Polyrust.mon` | 手寫 | 定義 | 這是有限支撐的單變量多項式，`HasMaxMono` 可由「取最大非零指數」直接證明。 單變量模型的單項式 x₀ⁿ。 | `Nat → Polyrust.MonoExp` |
| 956 | `Polyrust.subP_apply` | 手寫 | 定理 | 等式／恆等命題 | `∀ (p q : Polyrust.MPoly) (m : Polyrust.MonoExp), Polyrust.subP p q m = p m - q m` |
| 957 | `Polyrust.mon.eq_1` | 等式引理／助手 | 定理 | 定義 `mon` 的等式引理（定義的展開方程） | `∀ (n j : Nat), Polyrust.mon n j = if j = 0 then n else 0` |

## `Polyrust.WatchMove`（7 條）

職責：監視文字不變量：單步／迭代語義守恆

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 958 | `Polyrust.watch_overwrite_unsound` | 手寫 | 定理 | （純 3-SAT @brute 對照常態化實測抓到；修復見 `src/cdcl.rs`。） | `∃ σ,   Polyrust.clauseSat σ [{ var := 0, pos := true }, { var := 1, pos := true }, { var := 2, po…` |
| 959 | `Polyrust.WatchMove.casesOn` | 歸納型衍生 | 定義 | `WatchMove` 的案例分析原則 | `∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMove a a_1 → Prop} {a a_1 : List Polyrust…` |
| 960 | `Polyrust.WatchMove.recOn` | 歸納型衍生 | 定義 | `WatchMove` 的結構遞迴原則 | `∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMove a a_1 → Prop} {a a_1 : List Polyrust…` |
| 961 | `Polyrust.WatchMoves.below.casesOn` | 歸納型衍生 | 定義 | `WatchMoves` 的案例分析原則 | `∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop}   {motive_1 : {a a_1 …` |
| 962 | `Polyrust.WatchMoves.brecOn` | 歸納型衍生 | 定理 | `WatchMoves` 的良基遞迴助手 | `∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop} {a a_1 : List Polyrus…` |
| 963 | `Polyrust.WatchMoves.casesOn` | 歸納型衍生 | 定義 | `WatchMoves` 的案例分析原則 | `∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop} {a a_1 : List Polyrus…` |
| 964 | `Polyrust.WatchMoves.recOn` | 歸納型衍生 | 定義 | `WatchMoves` 的結構遞迴原則 | `∀ {motive : (a a_1 : List Polyrust.Lit) → Polyrust.WatchMoves a a_1 → Prop} {a a_1 : List Polyrus…` |

## `Polyrust.BoolNullstellensatz`（2 條）

職責：布爾立方 Nullstellensatz：無公共零點 ⟹ 顯式逆元乘子（純計算部分）

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 965 | `Polyrust.ModP` | 手寫 | 定義 | ! ## 一、模同餘（零依賴定義 `p ∣ a − b` 與極小 API） 模 `p` 同餘（整數層，零依賴：`p ∣ a − b`）。 | `Int → Int → Int → Prop` |
| 966 | `Polyrust.modp_congr` | 手寫 | 定理 | 模同餘命題 | `∀ {p a b c d : Int}, a = c → b = d → Polyrust.ModP p a b → Polyrust.ModP p c d` |

## `Polyrust.Embedding`（2 條）

職責：表達式嵌入的語義保持

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 967 | `Polyrust.P61` | 手寫 | 定義 | Mersenne 質數 p = 2⁶¹ − 1（本項目的係數域 𝔽_p）。 | `Nat` |
| 968 | `Polyrust.P61_eq` | 手寫 | 定理 | 命題證明 | `Polyrust.P61 + 1 = 2 ^ 61` |

## `Polyrust.Tactics`（2 條）

職責：戰術助手

| # | 宣告 | 類別 | 種類 | 功用 | 類型簽名（節錄） |
|---:|---|---|---|---|---|
| 969 | `Polyrust.tacticInt_ring` | 手寫 | 定義 | 整數環戰術巨集（int_ring） | `ParserDescr` |
| 970 | `Polyrust._aux_Polyrust_Tactics___macroRules_Polyrust_tacticInt_ring_1` | 等式引理／助手 | 定義 | 巨集規則助手（機器衍生） | `Macro` |

## 把這 970 條嵌入二進制，實際增添了什麼功能

### 直接回答

**不增添任何新的執行期計算功能。** 命題證明（`Prop`）編譯後是證明項，不含可執行內容；569 條定義雖會編成 Lean 位元碼進入靜態庫，但 Rust 側**沒有任何 C ABI 橋接去呼叫它們**（`src/formal.rs` 只有初始化入口）。

### 實際獲得的四項能力

| # | 能力 | 機制 |
|---|---|---|
| 1 | **啟動自檢**：二進制載入時初始化 Lean 執行期並註冊全部 20 模組的宣告；失敗即無法啟動。`startup_report()` 輸出「20 模組已靜態嵌入並載入 ✓」 | `formal::init()` → `initialize_polyrust_x2dformal_Polyrust()` |
| 2 | **信任錨隨包發行**：二進制本身攜帶著「本程式的演算法性質已被機器檢查過」的證明物件，而非只靠外部文件聲稱 | `.a` 靜態連結進 release 二進制（+1.37 MB，二進制總計約 3.8 MB） |
| 3 | **可再驗證性**：任何人取得源碼＋Lean 工具鏈，可重跑 `lake build` ＋ `AuditAll` 復現「受檢宣告 1,787、純構造 970、零 sorry、零自訂公理 ⇒ CLEAN」 | 判據寫死在 `AuditAll.lean`，CI 以末行 `AUDIT_RESULT=` 判定 |
| 4 | **零動態依賴保證**：嵌入後 `ldd` 僅見 glibc 系，無 Lean 動態庫 runtime 依賴 | 靜態連結 `libleanrt` |

### 這 970 條覆蓋的保證範圍（按職責歸組）

* **表達式／類型系統**（MicroInstance、Embedding、MacroExpansion、BorrowOwnership、OpAbstraction）：求值語義、嵌入保語義、巨集展開、借用檢查健全性。
* **子句層**（ClauseAlgebra、ClauseDuality、WatchMove）：子句代數變換保滿足性、監視文字單步與**任意步數迭代**語義守恆。
* **多項式層**（Monomial、SPoly、Squarefree、SumReduction、ProductReduction、Canonical、UniPoly）：單項式運算、Gröbner 步、平方自由化、歸約正確性。
* **憑證層**（T6Certificate、BoolNullstellensatz）：插值憑證、布爾立方無公共零點 ⟹ 模 p 可逆乘子（純計算部分）。
* **端到端**（T9EndToEnd、T9Generalized、根模組）：微實例級完整管線證明族。

### 邊界（講死）

1. 嵌入的是**宣告與證明項**，不是可呼叫的執行期函式——不會令程式「跑得不一樣」。
2. 依賴 `Classical.choice` 的定理（如 `bool_nullstellensatz`、`watchMoves_preserve_sat`）**不在**這 970 條之內；它們同樣已嵌入（全部 1,787 條宣告都在庫中），只是不屬純構造子集。
3. 本表 970 條由環境掃描自動產生（`Enumerate.lean`，判據同 `AuditAll`），非人工繕寫——與實際編譯內容同源，無漂移空間。
