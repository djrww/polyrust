# C3 Loop + Fuel + UNKNOWN 記檔（2026-09-20）

## 交付
1. **lexit 出邊棧**：`loops: Vec<(breaks, continues)>`——Break(d)/Continue(d) 精確路由到目標 loop 層；
   nested_loop（depth>0）直通。
2. **法證修正（核心）**：Continue 係 backedge（下一輪入邊）唔係出邊——之前撈亂令 while-loop 一輪
   就「自然退出」，fuel 語義全錯。C3 起：每輪頭入繼續路徑、Break 即出、K=fuel 輪盡 → `loop-fuel-exhausted(K)`。
3. **UNKNOWN 口徑（v0.3 hardening 對齊）**:`V4Opts{loop_fuel, require_full_loops, has_invariant}`；
   `scan_annotations` 掃 `# @fuel N(:)`/`# @invariant`（源行軌；`#[polyrust::fuel(N)]` attr 軌後補）。
   require_full && exhausted && !invariant ⇒ **UNKNOWN(reason)**，唔翻盤 SAT/UNSAT。
4. **triangular presolve**：SSA 定義鏈健全消去（唯一出現+線性先消，迭代至 fixpoint）——
   fib debug GB 48s → 全套 llbc 測試 0.42s；雙單測（健全 SAT 保持 / 矛盾唔匿）。
5. **法證形狀五連**（白名單擴充，全部 C0/C3 實測）：
   - body string `"Opaque"`（外部/trait method）+ null fun 佔位
   - RValue `{Ref}`/`AddressOf`（借用，抽象）、const `{Str}` 等非數值（ScalarOpaque）
   - stmt `Abort:"UndefinedBehavior"`（panic 分支，唔計正常終止）、`Drop{obj}`（no-op）
   - UnaryOp `Cast`（widening passthrough + marker）
   - 裸字符串 Deref projection、`(*r).0`/`*(r.0)` collapse（alias-cell 抽象，C4 先正名）
   - const 定點裸 `{"Value":[id,…]}`（nested_loop）→ crate-wide 表 fallback 掃全 translated

## 驗收截圖
- `loop_unknown`（`# @fuel: 1`，`while x < 100`）→ **Unknown("@fuel 1 不足覆蓋…")**，marker `loop-fuel-exhausted(1)` ✓
- loop_match 15/15：全 SAT（v3↔v4 一致）✓
- `scan_annotations`：`# @fuel 10`/`# @fuel: 7`/`# @invariant` 全捕捉到 ✓
- 118/118 lib 綠（113 +5 新）

## 殘尾（C4 卷走）
- Assert overflow 真約束（panic-freedom）；ADT/ref/borrow 真語義；`#[polyrust::fuel]` attr 軌；
  callee 內 loop 遺留出邊檢查而家硬錯（"dangling loop control in inlined call"）
