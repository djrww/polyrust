//! 多項式編碼擴展示例：展示 v0.2 新特性如何編為多項式
//! Phase3 擴展：完整 9 特性家族 + 契約、效應、借用、QAP

#[derive(Clone, Debug)]
pub struct TypeBits {
    pub node_id: usize,
    pub bits: Vec<usize>,
    pub names: Vec<String>,
}

pub fn encode_struct_product_example() -> String {
    r#"
# struct Point { x: i32, y: i32 } 的積型編碼
# 設 t_Point, t_x_i32, t_y_i32 為位元
# one-hot: t_Point + t_other =1 仍成立，但額外：
t_Point - t_x_i32 * t_y_i32 = 0   # Point 存在當且僅當兩字段都 i32
t_{Point {x: e1, y:e2}} - t_Point * (t_{e1}=i32) * (t_{e2}=i32) = 0
t_{p.x} - t_Point * t_{x_field} = 0   # 投影
# 多項式次數 ≤3，係數 ∈ {-1,0,1}
"#.to_string()
}

pub fn encode_enum_sum_example() -> String {
    r#"
# enum Option<T> { Some(T), None } 的和型編碼
# tag bits: tag_Some, tag_None, Σ tag =1
# t_Option = t_Some + t_None
# t_Some = t_T  (Some 攜帶 T)
# 構造 Some(e): t_{Some(e)} - tag_Some * t_e =0
# match:
#   match opt { Some(x) => b1, None => b2 }
# 引入 arm bits m1,m2 Σ m=1
#   t_result - m1*t_b1 - m2*t_b2 =0
#   pattern check: m1 → is_Some(opt), m2 → is_None(opt)
"#.to_string()
}

pub fn encode_vec_example() -> String {
    r#"
# Vec<T> 泛型容器編碼（Phase3）
# 類型構造: Vec<T> 位元 = f(T位元)，f 為單射
# 內部表示: ptr + len + cap 三元組
# 約束:
#   len <= cap : cap - len - slack =0
#   push: len' = len +1 : len_next - len -1 =0
#   pop: len' = len -1
# 操作規則:
#   Vec::new() : t = Vec<U> 對任意 U，tv_U 自由
#   push: self: &mut Vec<T>, v: T
#     t_self_VecT * t_v_T - t_self_VecT =0  (T 一致)
#   get: self: &Vec<T>, i: i32 -> Option<T>
#     t_result_OptionT - t_self_VecT =0
# 統一多項式: t_T1 - t_T2 =0
# R1CS: len/cap 作為 witness
"#.to_string()
}

pub fn encode_hashmap_example() -> String {
    r#"
# HashMap<K,V> 編碼（Phase3）
# 表示: Vec<(K,V)> + key 唯一
# 約束:
#   len <= cap 同 Vec
#   key 唯一: ∀ i≠j, k_i != k_j
#   (k_i - k_j) * inv -1 =0  # 逆元見證不等
#   insert: 若 key 存在則替換，否則 push
#   get: 線性搜索語義，多項式為 OR over i
# R1CS: inv 為輔助變量
"#.to_string()
}

pub fn encode_string_example() -> String {
    r#"
# String 編碼（Phase3）
# 視為 Vec<u8> + utf8 不變量
# 約束:
#   同 Vec: len <= cap, push 等
#   utf8: 每個 byte ∈ [0,255] 且有效序列
#     byte - 0 >=0, 255 - byte >=0 (範圍，轉 slack)
#     多字節序列: 起始 byte 模式檢查
#   String::from(s): s: &str -> String, &str 視為 String 的借用
# R1CS: utf8 檢查為位運算約束
"#.to_string()
}

pub fn encode_loop_example() -> String {
    r#"
# loop / while / for 有界展開 + 不變量（Phase3）
# @fuel: 10, @invariant: x >=0, @requires, @ensures
# while cond { body }  fuel=K
# 降維:
#   let mut __fuel = K;
#   loop {
#     if __fuel <=0 { break; }
#     if !cond { break; }
#     assert invariant;
#     body;
#     __fuel -=1;
#   }
# 編碼:
#   fuel counter: c_{i+1} - c_i +1 =0  (遞減)
#   invariant: t_inv -1 =0  (bool 為 true)
#   保持: inv ∧ cond ∧ body ⇒ inv'
#   結束: inv ∧ ¬cond ⇒ post
# for:
#   for x in iter { body }
#   → let mut __iter = iter.into_iter();
#     loop { match __iter.next() { Some(x) => body, None => break } }
#   同 while + iterator 協議
"#.to_string()
}

pub fn encode_match_example() -> String {
    r#"
# match 編譯為決策樹（Phase3 完整）
# match e {
#   Pat1 => b1,
#   Pat2 => b2,
# }
# 引入:
#   tmp = e
#   is_pat1(tmp) = tag check polynomial
#   m1,m2 arm bits Σ m=1, m_i*(m_i-1)=0
#   t_result = m1*t_b1 + m2*t_b2
#   m1*(1 - is_pat1) =0, m2*(1 - is_pat2)=0
# 窮舉性: Σ is_pat_i -1 =0 若無 _
# 存在量化: 模式綁定 let x = destructure(tmp)
# R1CS: arm bits 為 one-hot，結果為線性組合
"#.to_string()
}

pub fn encode_module_example() -> String {
    r#"
# mod 樹扁平化（Phase3 完整）
# mod a { pub struct B { x: i32 } }
# mod b { use crate::a::B; }
# 解析:
#   ModuleTree { name: crate, children: [a,b] }
#   a::B -> fully qualified path
#   b 中 use -> alias table
# 可見性:
#   vis_{path} bit, 若私有訪問非法 → 強制矛盾
#   t_{illegal} =0 對所有 t，破壞 one-hot → UNSAT
# Lean: ModuleFlatten 模組證明扁平化保持可定型性
"#.to_string()
}

pub fn encode_async_example() -> String {
    r#"
# async fn -> Future 狀態機（Phase3 QAP）
# async fn fetch() -> i32 { 42 }
# 降維:
#   enum FetchState { Start, Poll(i32), Done(i32) }
#   struct FetchFuture { state: FetchState }
#   impl Future for FetchFuture { type Output=i32; fn poll(...) }
# 編碼:
#   Ty Future<T> = generic
#   await: e: Future<T> => await e: T
#   t_{await e} - t_T * t_{e: Future<T>} =0
#   引入 Poll enum sum type
# 狀態機 QAP:
#   one-hot: Σ s_i -1 =0, s_i*(s_i-1)=0
#   轉換: s_from * poll - s_to =0
#   輪詢: poll_i*(poll_i-1)=0
#   3 states for 1 await: Pending, Polling(0), Ready
# R1CS: 狀態變量為 witness，轉換為約束
# Lean: AsyncStateMachine 模組
"#.to_string()
}

pub fn encode_unsafe_example() -> String {
    r#"
# unsafe 上下文位元（Phase3）
# in_unsafe bit, 初始 0
# unsafe { ... } 內 in_unsafe=1
# *mut T, *const T 新類型
# 規則:
#   raw ptr deref: *p 要求 in_unsafe=1
#   (1 - in_unsafe) * t_{*p} =0 → 若不在 unsafe 則 t 無解 → UNSAT
#   &mut T as *mut T 允許 (safe)
#   *mut T as &mut T 要求 unsafe
# 借用檢查: raw ptr 不追蹤，但 safe 引用仍追蹤
# 效應系統:
#   # @unsafe-allowed 允許全局 unsafe
#   否則需在 unsafe 塊內
# Lean: UnsafeContext 模組，定理 unsafe_allowed_passes
"#.to_string()
}

pub fn encode_io_example() -> String {
    r#"
# I/O 效應系統（Phase3）
# 建模: I/O 操作視為返回 Result<T,E> 的普通函式，加上效應標記
#   fn File::open(path: String) -> Result<File, IoError>  // 效應: io
#   println! : (T) -> ()  // 效應: io
# 約束:
#   has_io bit, io_usages: List Nat
#   # @no-io: has_io 必須為 0，否則 UNSAT
#   # @pure true: has_io 必須為 0
#   has_io*(has_io-1)=0 布爾
# QAP: I/O 見證不進 QAP（非純計算），QAP 僅驗證純部分
# Lean: UnsafeContext 含 I/O 檢查
"#.to_string()
}

pub fn encode_lifetime_example() -> String {
    r#"
# lifetime 參數 outlives 圖 + NLL（Phase3）
# fn longest<'a>(x: &'a str, y: &'a str) -> &'a str where 'a: 'b
# 語法: # @lifetime 'a: 'b
# 引入:
#   Lifetime: 'a, 'b, 'static
#   Outlives: 'a: 'b 表示 'a 存活至少與 'b 一樣長
#   Graph: 有向圖，邊 longer -> shorter
#   檢查: has_cycle? 有環 → UNSAT
#   傳遞閉包: 'a: 'b, 'b: 'c ⇒ 'a: 'c
#   'static outlives all
# NLL:
#   Region { lifetime, start, fin, borrowNode }
#   overlap: start < other.fin && other.start < fin
#   conflict: overlap && !outlives_holds
# 多項式:
#   區間包含: [ls,le) 包含 [ss,se) ⇒ ss - ls >=0, le - se >=0
#   編碼為差值多項式 + slack
#   borrow bit: b_{&'a x} = b_x * lt_a
# Lean: LifetimeRegion 模組，定理 static_outlives_all, outlives_refl
"#.to_string()
}

pub fn encode_contract_example() -> String {
    r#"
# 契約註解（Phase3）
# # @fuel: 10, # @invariant: x >=0, # @requires, # @ensures, # @pure, # @no-io, # @qap, # @type-universe, # @mode
# 解析: parse_at_key 避免 = 在 >= 中被誤分割
#   key = 首 token 直到 whitespace/:/=, rest 去掉一次前導 :/=
#   比較運算符 >= <= == != 保留
# LoopContract:
#   fuel_or_default = 3
#   invariant_poly_text: t_inv -1 =0
#   fuel_poly_text: c_{i+1} - c_i -1 =0
# requires/ensures:
#   前置/後置條件 bool 表達式，生成 t_req -1 =0
# pure:
#   # @pure true 要求無 I/O
# Lean: LoopContract 模組
"#.to_string()
}

pub fn encode_trait_impl_example() -> String {
    r#"
# trait/impl 方法表與存在量化（Phase3）
# trait Display { fn fmt(&self) -> String; }
# impl Display for Point { fn fmt(&self) -> String { ... } }
# 方法表:
#   traits: HashMap<String, TraitDef>
#   impls: Vec<ImplDef>
#   impl_map: (for_ty, trait_name) -> impl idx
#   inherent_map: for_ty -> Vec<impl idx>
# 解析:
#   resolve_method(recv_ty, method): 先 inherent 後 trait
#   types_implementing(trait): 所有實現類型
# 存在量化:
#   ∃T: Display. body(T) → 引入 impl_exists_{T,Display} bit
#   method call obj.fmt() 要求 impl_bit=1
# 多項式:
#   method_call_poly: // Type.method -> ret : body
#   existential_poly: exists T: Trait { body }
# Lean: TraitImpl 模組
"#.to_string()
}

pub fn all_examples() -> Vec<(&'static str, String)> {
    vec![
        ("struct (積型)", encode_struct_product_example()),
        ("enum (和型)", encode_enum_sum_example()),
        ("Vec<T>", encode_vec_example()),
        ("HashMap<K,V>", encode_hashmap_example()),
        ("String", encode_string_example()),
        ("loop/while/for (契約)", encode_loop_example()),
        ("match (決策樹)", encode_match_example()),
        ("mod 樹", encode_module_example()),
        ("async/await (QAP)", encode_async_example()),
        ("unsafe/raw ptr", encode_unsafe_example()),
        ("I/O 效應", encode_io_example()),
        ("lifetime 'a (outlives/NLL)", encode_lifetime_example()),
        ("契約 @fuel/@invariant", encode_contract_example()),
        ("trait/impl 方法表", encode_trait_impl_example()),
    ]
}
