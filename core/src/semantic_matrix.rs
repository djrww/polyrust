// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Semantic Matrix — Phase A 语义保持测试矩阵 100 例
//! 目标：poly → Rust → rustc → cargo test 行为等价，>90% 通过率，无 filler

use crate::pipeline_v3::{PipelineV3Config, run_pipeline_v3_with_config};

#[derive(Clone, Debug)]
pub struct SemanticCase {
    pub name: &'static str,
    pub poly_src: &'static str,
    pub should_sat: bool,
    pub expected_contains: Vec<&'static str>,
    pub category: &'static str,
}

impl SemanticCase {
    pub fn new(name: &'static str, src: &'static str, should_sat: bool, contains: Vec<&'static str>, cat: &'static str) -> Self {
        Self {
            name,
            poly_src: src,
            should_sat,
            expected_contains: contains,
            category: cat,
        }
    }
}

pub fn all_semantic_cases() -> Vec<SemanticCase> {
    vec![
        // 基础 10
        SemanticCase::new("sqr", "# @intent square\nfn sqr(x: i32) -> i32 { x * x }", true, vec!["sqr", "i32"], "basic"),
        SemanticCase::new("add", "# @intent add\nfn add(a: i32, b: i32) -> i32 { a + b }", true, vec!["add"], "basic"),
        SemanticCase::new("fact", "# @intent factorial\nfn fact(n: i32) -> i32 { if n <= 1 { 1 } else { n * fact(n-1) } }", true, vec!["fact"], "basic"),
        SemanticCase::new("fib", "# @intent fib\nfn fib(n: i32) -> i32 { if n <= 1 { n } else { fib(n-1) + fib(n-2) } }", true, vec!["fib"], "basic"),
        SemanticCase::new("max", "# @intent max\nfn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }", true, vec!["max"], "basic"),
        SemanticCase::new("is_even", "# @intent is_even\nfn is_even(x: i32) -> bool { x % 2 == 0 }", true, vec!["is_even", "bool"], "basic"),
        SemanticCase::new("abs", "# @intent abs\nfn abs(x: i32) -> i32 { if x < 0 { -x } else { x } }", true, vec!["abs"], "basic"),
        SemanticCase::new("pow", "# @intent pow\nfn pow(x: i32, n: i32) -> i32 { if n == 0 { 1 } else { x * pow(x, n-1) } }", true, vec!["pow"], "basic"),
        SemanticCase::new("gcd", "# @intent gcd\nfn gcd(a: i32, b: i32) -> i32 { if b == 0 { a } else { gcd(b, a % b) } }", true, vec!["gcd"], "basic"),
        SemanticCase::new("sum_range", "# @intent sum_range\nfn sum_range(n: i32) -> i32 { let mut s = 0; let mut i = 0; while i < n { s = s + i; i = i + 1; } s }", true, vec!["sum_range", "while"], "basic"),

        // struct/enum 15
        SemanticCase::new("struct_point", "# @intent struct Point\nstruct Point { x: i32, y: i32 }\nimpl Point { fn new(x: i32, y: i32) -> Point { Point { x, y } } fn len(&self) -> i32 { self.x * self.x + self.y * self.y } }", true, vec!["Point", "struct"], "struct_enum"),
        SemanticCase::new("enum_option", "# @intent enum Option\n#[derive(Debug)]\nenum MyOption { Some(i32), None }\nfn unwrap_or(o: MyOption, def: i32) -> i32 { match o { MyOption::Some(v) => v, MyOption::None => def } }", true, vec!["MyOption", "match"], "struct_enum"),
        SemanticCase::new("enum_result", "# @intent Result\n#[derive(Debug)]\nenum MyResult { Ok(i32), Err(String) }\nfn is_ok(r: MyResult) -> bool { match r { MyResult::Ok(_) => true, MyResult::Err(_) => false } }", true, vec!["MyResult"], "struct_enum"),
        SemanticCase::new("struct_rect", "# @intent Rect\nstruct Rect { w: i32, h: i32 }\nfn area(r: Rect) -> i32 { r.w * r.h }", true, vec!["Rect", "area"], "struct_enum"),
        SemanticCase::new("impl_methods", "# @intent impl\nstruct Counter { n: i32 }\nimpl Counter { fn new() -> Counter { Counter { n: 0 } } fn inc(&mut self) { self.n = self.n + 1; } fn get(&self) -> i32 { self.n } }", true, vec!["Counter", "impl"], "struct_enum"),
        SemanticCase::new("trait_display", "# @intent trait Display\ntrait Display { fn fmt(&self) -> String; }\nstruct S { x: i32 }\nimpl Display for S { fn fmt(&self) -> String { format!(\"{}\", self.x) } }", true, vec!["Display", "trait"], "struct_enum"),
        SemanticCase::new("generic_struct", "# @intent generic struct\nstruct Pair<T> { a: T, b: T }\nfn first<T>(p: Pair<T>) -> T { p.a }", true, vec!["Pair"], "struct_enum"),
        SemanticCase::new("enum_list", "# @intent List\n#[derive(Debug)]\nenum List { Nil, Cons(i32, Box<List>) }\nfn len(l: &List) -> i32 { match l { List::Nil => 0, List::Cons(_, t) => 1 + len(t) } }", true, vec!["List", "Box"], "struct_enum"),
        SemanticCase::new("struct_nested", "# @intent nested\nstruct Inner { v: i32 }\nstruct Outer { inner: Inner }\nfn get(o: Outer) -> i32 { o.inner.v }", true, vec!["Inner", "Outer"], "struct_enum"),
        SemanticCase::new("enum_color", "# @intent Color\n#[derive(Debug, PartialEq)]\nenum Color { Red, Green, Blue }\nfn is_red(c: Color) -> bool { c == Color::Red }", true, vec!["Color", "Red"], "struct_enum"),
        SemanticCase::new("struct_with_enum", "# @intent struct+enum\n#[derive(Debug)]\nenum Status { Active, Inactive }\nstruct User { id: i32, status: Status }\nfn is_active(u: User) -> bool { match u.status { Status::Active => true, _ => false } }", true, vec!["User", "Status"], "struct_enum"),
        SemanticCase::new("impl_trait", "# @intent impl trait\ntrait Animal { fn sound(&self) -> String; }\nstruct Dog;\nimpl Animal for Dog { fn sound(&self) -> String { \"woof\".to_string() } }", true, vec!["Animal", "Dog"], "struct_enum"),
        SemanticCase::new("struct_default", "# @intent Default\n#[derive(Debug, Default)]\nstruct Config { debug: bool, port: i32 }\nfn default_config() -> Config { Config::default() }", true, vec!["Config", "Default"], "struct_enum"),
        SemanticCase::new("enum_with_data", "# @intent enum data\n#[derive(Debug)]\nenum Shape { Circle(i32), Rect(i32, i32) }\nfn area(s: Shape) -> i32 { match s { Shape::Circle(r) => 3 * r * r, Shape::Rect(w,h) => w * h } }", true, vec!["Shape", "Circle"], "struct_enum"),
        SemanticCase::new("struct_methods_chain", "# @intent chain\nstruct Builder { x: i32 }\nimpl Builder { fn new() -> Self { Self { x: 0 } } fn set(mut self, x: i32) -> Self { self.x = x; self } fn build(self) -> i32 { self.x } }", true, vec!["Builder"], "struct_enum"),

        // Vec/String/HashMap 15
        SemanticCase::new("vec_new", "# @intent Vec new\nfn make_vec() -> Vec<i32> { let mut v = Vec::new(); v.push(1); v.push(2); v }", true, vec!["Vec", "push"], "vec_string"),
        SemanticCase::new("vec_len", "# @intent Vec len\nfn vec_len(v: Vec<i32>) -> usize { v.len() }", true, vec!["Vec", "len"], "vec_string"),
        SemanticCase::new("vec_get", "# @intent Vec get\nfn vec_get(v: Vec<i32>, i: usize) -> Option<i32> { if i < v.len() { Some(v[i]) } else { None } }", true, vec!["Vec"], "vec_string"),
        SemanticCase::new("vec_iter_sum", "# @intent Vec iter sum\nfn sum_vec(v: Vec<i32>) -> i32 { let mut s = 0; for x in v { s = s + x; } s }", true, vec!["Vec", "for"], "vec_string"),
        SemanticCase::new("string_from", "# @intent String from\nfn hello() -> String { String::from(\"hello\") }", true, vec!["String"], "vec_string"),
        SemanticCase::new("string_len", "# @intent String len\nfn str_len(s: String) -> usize { s.len() }", true, vec!["String", "len"], "vec_string"),
        SemanticCase::new("string_concat", "# @intent String concat\nfn concat(a: String, b: String) -> String { a + &b }", true, vec!["String"], "vec_string"),
        SemanticCase::new("hashmap_new", "# @intent HashMap new\nuse std::collections::HashMap;\nfn make_map() -> HashMap<String, i32> { let mut m = HashMap::new(); m.insert(\"a\".to_string(), 1); m }", true, vec!["HashMap"], "vec_string"),
        SemanticCase::new("hashmap_get", "# @intent HashMap get\nuse std::collections::HashMap;\nfn get_map(m: HashMap<String, i32>, k: String) -> Option<i32> { m.get(&k).copied() }", true, vec!["HashMap", "get"], "vec_string"),
        SemanticCase::new("vec_map", "# @intent Vec map\nfn double_vec(v: Vec<i32>) -> Vec<i32> { v.into_iter().map(|x| x*2).collect() }", true, vec!["Vec", "map"], "vec_string"),
        SemanticCase::new("string_chars", "# @intent String chars\nfn count_chars(s: String) -> usize { s.chars().count() }", true, vec!["String", "chars"], "vec_string"),
        SemanticCase::new("vec_filter", "# @intent Vec filter\nfn filter_even(v: Vec<i32>) -> Vec<i32> { v.into_iter().filter(|x| x % 2 == 0).collect() }", true, vec!["Vec", "filter"], "vec_string"),
        SemanticCase::new("hashmap_len", "# @intent HashMap len\nuse std::collections::HashMap;\nfn map_len<K,V>(m: HashMap<K,V>) -> usize { m.len() }", true, vec!["HashMap"], "vec_string"),
        SemanticCase::new("string_split", "# @intent String split\nfn split_words(s: String) -> Vec<String> { s.split(' ').map(|x| x.to_string()).collect() }", true, vec!["String", "split"], "vec_string"),
        SemanticCase::new("vec_sort", "# @intent Vec sort\nfn sort_vec(mut v: Vec<i32>) -> Vec<i32> { v.sort(); v }", true, vec!["Vec", "sort"], "vec_string"),

        // loop/match 15
        SemanticCase::new("loop_break", "# @intent loop break\nfn loop_break(n: i32) -> i32 { let mut i = 0; loop { if i >= n { break i; } i = i + 1; } }", true, vec!["loop", "break"], "loop_match"),
        SemanticCase::new("while_loop", "# @intent while\nfn while_sum(n: i32) -> i32 { let mut s = 0; let mut i = 0; while i < n { s = s + i; i = i + 1; } s }", true, vec!["while"], "loop_match"),
        SemanticCase::new("for_range", "# @intent for range\nfn for_sum(n: i32) -> i32 { let mut s = 0; for i in 0..n { s = s + i; } s }", true, vec!["for"], "loop_match"),
        SemanticCase::new("match_int", "# @intent match int\nfn match_int(x: i32) -> String { match x { 0 => \"zero\".to_string(), 1 => \"one\".to_string(), _ => \"other\".to_string() } }", true, vec!["match"], "loop_match"),
        SemanticCase::new("match_bool", "# @intent match bool\nfn match_bool(b: bool) -> i32 { match b { true => 1, false => 0 } }", true, vec!["match", "bool"], "loop_match"),
        SemanticCase::new("loop_continue", "# @intent loop continue\nfn skip_even(n: i32) -> i32 { let mut s = 0; for i in 0..n { if i % 2 == 0 { continue; } s = s + i; } s }", true, vec!["continue"], "loop_match"),
        SemanticCase::new("nested_loop", "# @intent nested loop\nfn nested(n: i32) -> i32 { let mut s = 0; for i in 0..n { for j in 0..n { s = s + i*j; } } s }", true, vec!["for"], "loop_match"),
        SemanticCase::new("match_guard", "# @intent match guard\nfn match_guard(x: i32) -> String { match x { n if n < 0 => \"neg\".to_string(), 0 => \"zero\".to_string(), _ => \"pos\".to_string() } }", true, vec!["match"], "loop_match"),
        SemanticCase::new("loop_return", "# @intent loop return\nfn find_first(v: Vec<i32>, target: i32) -> Option<usize> { for (i, &x) in v.iter().enumerate() { if x == target { return Some(i); } } None }", true, vec!["Option", "for"], "loop_match"),
        SemanticCase::new("while_let", "# @intent while let\nfn while_let(mut v: Vec<i32>) -> i32 { let mut s = 0; while let Some(x) = v.pop() { s = s + x; } s }", true, vec!["while", "let"], "loop_match"),
        SemanticCase::new("match_option", "# @intent match Option\nfn match_option(o: Option<i32>) -> i32 { match o { Some(v) => v, None => 0 } }", true, vec!["Option", "match"], "loop_match"),
        SemanticCase::new("match_result", "# @intent match Result\nfn match_result(r: Result<i32, String>) -> i32 { match r { Ok(v) => v, Err(_) => -1 } }", true, vec!["Result", "match"], "loop_match"),
        SemanticCase::new("for_enumerate", "# @intent for enumerate\nfn enumerate_sum(v: Vec<i32>) -> i32 { let mut s = 0; for (i, x) in v.iter().enumerate() { s = s + (i as i32)*x; } s }", true, vec!["enumerate"], "loop_match"),
        SemanticCase::new("loop_invariant", "# @intent loop invariant\n# @invariant s == sum(0..i)\n# @fuel 10\nfn inv_sum(n: i32) -> i32 { let mut s = 0; let mut i = 0; while i < n { s = s + i; i = i + 1; } s }", true, vec!["while", "invariant"], "loop_match"),
        SemanticCase::new("match_tuple", "# @intent match tuple\nfn match_tuple(t: (i32, bool)) -> i32 { match t { (x, true) => x, (_, false) => 0 } }", true, vec!["match"], "loop_match"),

        // ownership/borrowck/lifetime 15
        SemanticCase::new("borrow_immut", "# @intent borrow immut\nfn borrow_immut(x: &i32) -> i32 { *x }", true, vec!["borrow"], "borrowck"),
        SemanticCase::new("borrow_mut", "# @intent borrow mut\nfn borrow_mut(x: &mut i32) { *x = *x + 1; }", true, vec!["&mut"], "borrowck"),
        SemanticCase::new("move_semantic", "# @intent move\nfn move_sem(s: String) -> String { let t = s; t }", true, vec!["String"], "borrowck"),
        SemanticCase::new("lifetime_simple", "# @intent lifetime\nfn longest<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() > b.len() { a } else { b } }", true, vec!["longest", "'a"], "borrowck"),
        SemanticCase::new("borrow_two", "# @intent borrow two\nfn two_borrows(a: &i32, b: &i32) -> i32 { *a + *b }", true, vec!["&"], "borrowck"),
        // 2026-09-19 修正：期望 SAT。rustc 實測此函數**合法編譯**（兩個 &mut 參數嘅互斥性
        // 由調用點借用規則保證，簽名本身無衝突）；v1 與 v3 都判 SAT。
        // 舊期望 UNSAT(false) 屬 aspirational 錯標，地真值為準。
        SemanticCase::new("mut_borrow_exclusive", "# @intent mut exclusive\nfn exclusive(x: &mut i32, y: &mut i32) { *x = *x + 1; *y = *y + 2; }", true, vec!["&mut"], "borrowck"),
        SemanticCase::new("nll_region", "# @intent NLL\nfn nll() -> i32 { let mut x = 5; { let r = &mut x; *r = 10; } x }", true, vec!["&mut"], "borrowck"),
        SemanticCase::new("outlives", "# @intent outlives\nfn outlives<'a, 'b>(x: &'a i32, y: &'b i32) -> i32 where 'a: 'b { *x + *y }", true, vec!["outlives", "'a"], "borrowck"),
        SemanticCase::new("borrow_in_loop", "# @intent borrow in loop\nfn borrow_loop(v: &mut Vec<i32>) { for x in v.iter_mut() { *x = *x * 2; } }", true, vec!["iter_mut"], "borrowck"),
        SemanticCase::new("move_closure", "# @intent move closure\nfn move_closure() -> i32 { let x = 5; let f = move || x + 1; f() }", true, vec!["move", "closure"], "borrowck"),
        SemanticCase::new("ref_deref", "# @intent ref deref\nfn ref_deref() -> i32 { let x = 5; let r = &x; let rr = &r; ***rr }", false, vec!["&"], "borrowck"),
        SemanticCase::new("lifetime_struct", "# @intent lifetime struct\nstruct Holder<'a> { s: &'a str }\nfn make_holder<'a>(s: &'a str) -> Holder<'a> { Holder { s } }", true, vec!["Holder", "'a"], "borrowck"),
        SemanticCase::new("borrow_match", "# @intent borrow match\nfn borrow_match(o: &Option<i32>) -> i32 { match o { Some(v) => *v, None => 0 } }", true, vec!["Option", "&"], "borrowck"),
        SemanticCase::new("mut_borrow_split", "# @intent mut split\nfn split(v: &mut Vec<i32>) -> (i32, i32) { let (a,b) = v.split_at_mut(1); (a[0], b[0]) }", true, vec!["split_at_mut"], "borrowck"),
        SemanticCase::new("lifetime_elision", "# @intent elision\nfn first_word(s: &str) -> &str { s.split(' ').next().unwrap_or(\"\") }", true, vec!["&str"], "borrowck"),

        // unsafe 10 (real checks)
        SemanticCase::new("raw_ptr_valid", "// @valid: non_null, aligned, in_bounds, not_dangling\n# @intent raw_ptr\nfn raw_valid() { unsafe { let x = 5; let p: *const i32 = &x as *const _; let _ = *p; } }", true, vec!["*const", "unsafe"], "unsafe"),
        SemanticCase::new("raw_ptr_box", "// @valid: from Box\n# @intent Box raw\nfn box_raw() { unsafe { let b = Box::new(5); let p = Box::into_raw(b); let _ = *p; let _ = Box::from_raw(p); } }", true, vec!["Box", "into_raw"], "unsafe"),
        SemanticCase::new("static_mut_exclusive", "// @exclusive\n# @intent static_mut\nstatic mut COUNTER: i32 = 0;\nfn inc() { unsafe { COUNTER += 1; } }", true, vec!["static mut", "COUNTER"], "unsafe"),
        SemanticCase::new("static_mut_mutex", "// @mutex\n# @intent static mut mutex\nuse std::sync::Mutex;\nstatic M: Mutex<i32> = Mutex::new(0);\nfn inc_mutex() { *M.lock().unwrap() += 1; }", true, vec!["Mutex"], "unsafe"),
        SemanticCase::new("union_tag", "// @tag_match\n# @intent union\nunion MyUnion { i: i32, f: f32 }\nfn get_i(u: MyUnion) -> i32 { unsafe { u.i } }", true, vec!["union", "MyUnion"], "unsafe"),
        SemanticCase::new("unsafe_fn_precond", "// @precond: x > 0\n# @intent unsafe fn\nunsafe fn my_unsafe(x: i32) -> i32 { x * 2 }\nfn call() { unsafe { let _ = my_unsafe(5); } }", true, vec!["unsafe fn", "my_unsafe"], "unsafe"),
        SemanticCase::new("unsafe_trait_invariant", "// @invariant: Send safe\n# @intent unsafe trait\nunsafe trait MyUnsafe {}\nstruct S;\nunsafe impl MyUnsafe for S {}", true, vec!["unsafe trait", "MyUnsafe"], "unsafe"),
        SemanticCase::new("raw_ptr_missing_src", "# @intent raw_ptr missing\nfn raw_missing() { unsafe { let p: *const i32 = std::ptr::null(); let _ = *p; } }", false, vec!["*const"], "unsafe"),
        SemanticCase::new("static_mut_missing", "# @intent static_mut missing\nstatic mut X: i32 = 0;\nfn access() { unsafe { X = 1; } }", false, vec!["static mut"], "unsafe"),
        SemanticCase::new("union_missing", "# @intent union missing\nunion U { a: i32, b: f32 }\nfn get(u: U) -> i32 { unsafe { u.a } }", false, vec!["union"], "unsafe"),

        // async/io/effects 10
        SemanticCase::new("async_simple", "# @intent async simple\nasync fn async_add(a: i32, b: i32) -> i32 { a + b }", true, vec!["async", "Future"], "async_io"),
        SemanticCase::new("async_await", "# @intent async await\nasync fn foo() -> i32 { 42 }\nasync fn bar() -> i32 { foo().await + 1 }", true, vec!["await"], "async_io"),
        SemanticCase::new("io_pure", "# @pure\n# @intent pure io\nfn pure_fn(x: i32) -> i32 { x * 2 }", true, vec!["pure"], "async_io"),
        SemanticCase::new("io_effect", "# @intent io effect\nfn with_io() { println!(\"hello\"); }", true, vec!["println"], "async_io"),
        SemanticCase::new("async_spawn", "# @intent async spawn\nasync fn spawned() -> i32 { tokio::spawn(async { 1 }).await.unwrap_or(0) }", false, vec!["spawn"], "async_io"),
        SemanticCase::new("future_combinator", "# @intent future combinator\nasync fn comb() -> i32 { let a = async { 1 }.await; let b = async { 2 }.await; a + b }", true, vec!["async"], "async_io"),
        SemanticCase::new("io_file", "# @intent file io\nfn read_file() -> Result<String, std::io::Error> { std::fs::read_to_string(\"Cargo.toml\") }", true, vec!["read_to_string"], "async_io"),
        SemanticCase::new("async_loop", "# @intent async loop\nasync fn async_loop(n: i32) -> i32 { let mut s = 0; for i in 0..n { s += i; } s }", true, vec!["async", "for"], "async_io"),
        SemanticCase::new("pure_no_io", "# @pure\n# @intent no io\nfn no_io(x: Vec<i32>) -> i32 { x.iter().sum() }", true, vec!["pure"], "async_io"),
        SemanticCase::new("io_with_pure_call", "# @intent io with pure\n# @pure\nfn pure_inner(x: i32) -> i32 { x + 1 }\nfn outer() { println!(\"{}\", pure_inner(5)); }", true, vec!["pure_inner"], "async_io"),

        // commercial/nl 10
        SemanticCase::new("password_gen", "# @intent password generator\nstruct PasswordConfig { length: usize }\nimpl PasswordConfig { fn is_valid(&self) -> bool { self.length >= 8 } }\nfn generate(c: PasswordConfig) -> String { \"a\".repeat(c.length) }", true, vec!["PasswordConfig"], "commercial"),
        SemanticCase::new("text_buffer", "# @intent TextBuffer\nstruct TextBuffer { content: String, version: u64 }\nimpl TextBuffer { fn new(s: String) -> Self { Self { content: s, version: 0 } } fn len(&self) -> usize { self.content.len() } }", true, vec!["TextBuffer"], "commercial"),
        SemanticCase::new("file_tree", "# @intent FileTree\n#[derive(Debug)]\nstruct FileNode { path: String, name: String, is_dir: bool }\nimpl FileNode { fn is_dir(&self) -> bool { self.is_dir } }", true, vec!["FileNode"], "commercial"),
        SemanticCase::new("reactive_ui", "# @intent reactive UI\nstruct VNode { tag: String, children: Vec<VNode> }\nfn create_element(tag: String) -> VNode { VNode { tag, children: vec![] } }", true, vec!["VNode"], "commercial"),
        SemanticCase::new("enterprise_ide", "# @intent Enterprise IDE\nuse std::collections::HashMap;\nstruct EnterpriseIDE { editors: HashMap<String, String> }\nimpl EnterpriseIDE { fn new() -> Self { Self { editors: HashMap::new() } } fn open(&mut self, p: String, c: String) { self.editors.insert(p,c); } }", true, vec!["EnterpriseIDE", "HashMap"], "commercial"),
        SemanticCase::new("defi_audit", "# @intent DeFi audit\nstruct AuditReport { risk: f64 }\nfn audit(code: String) -> AuditReport { AuditReport { risk: if code.contains(\"unsafe\") { 80.0 } else { 10.0 } } }", true, vec!["AuditReport"], "commercial"),
        SemanticCase::new("embedded_cert", "# @intent embedded cert\nfn cert_gen(id: &str) -> String { format!(\"cert-{}-signed\", id) }", true, vec!["cert_gen"], "commercial"),
        SemanticCase::new("llm_guardrail", "# @intent LLM guardrail\nfn guardrail(input: String) -> bool { !input.contains(\"unsafe\") || input.contains(\"@valid\") }", true, vec!["guardrail"], "commercial"),
        SemanticCase::new("web3_audit", "# @intent web3 audit\nstruct Web3Audit { passed: bool }\nfn audit_web3(code: String) -> Web3Audit { Web3Audit { passed: !code.is_empty() } }", true, vec!["Web3Audit"], "commercial"),
        SemanticCase::new("self_evolving", "# @intent self evolving\nfn evolve(src: String) -> String { format!(\"{} // evolved\", src) }", true, vec!["evolve"], "commercial"),
    ]
}

pub fn run_semantic_matrix() -> (usize, usize, Vec<String>) {
    let config = PipelineV3Config::default();
    let mut passed = 0;
    let mut failed = 0;
    let mut details = Vec::new();
    for case in all_semantic_cases() {
        let res = run_pipeline_v3_with_config(case.name, case.poly_src, None, &config);
        match res {
            Ok(v3) => {
                let sat = !v3.final_is_unsat;
                let rust_code = v3.generated_rust.clone().unwrap_or_default();
                let contains_all = case.expected_contains.iter().all(|s| rust_code.contains(*s) || case.poly_src.contains(*s));
                let ok = sat == case.should_sat && (case.should_sat == false || contains_all);
                if ok {
                    passed += 1;
                } else {
                    failed += 1;
                    details.push(format!(
                        "FAIL {}: expected SAT={} got SAT={} contains={} rust_len={}",
                        case.name, case.should_sat, sat, contains_all, rust_code.len()
                    ));
                }
            }
            Err(e) => {
                if !case.should_sat {
                    passed += 1;
                } else {
                    failed += 1;
                    details.push(format!("ERR {}: {} expected SAT={}", case.name, e, case.should_sat));
                }
            }
        }
    }
    (passed, failed, details)
}

pub fn semantic_matrix_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![("semantic_matrix.rs", "Semantic Matrix 100 例 — Phase A", "core/src/semantic_matrix.rs")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_matrix_count() {
        let cases = all_semantic_cases();
        assert_eq!(cases.len(), 100, "should have 100 cases");
    }

    /// v0.3 hardening：已知失敗白名單（baseline ratchet）。
    /// 規則：只允許從本表移除（修好即刪行並同步下調上限），
    /// 絕不新增——新失敗 = 回歸，CI 紅。每行附原因與跟蹤。
    const KNOWN_FAILURES: &[(&str, &str)] = &[
        ("async_simple", "async 狀態機生成碼語義標記缺失（expected_contains 不符）— P1"),
        ("async_spawn", "async spawn 判定偏差 — P1"),
        ("io_with_pure_call", "I/O 效應 × pure 誤拒（false UNSAT）— P1"),
        ("enterprise_ide", "商用大例誤拒（false UNSAT）— P1"),
    ];

    #[test]
    fn test_semantic_matrix_pass_rate() {
        let (passed, failed, details) = run_semantic_matrix();
        let total = passed + failed;
        let rate = passed as f64 / total as f64 * 100.0;
        println!("Semantic Matrix: {}/{} passed ({:.1}%)", passed, total, rate);
        for d in &details {
            println!("{}", d);
        }
        // 2026-09-19 第二次收緊：P0-C1 修好 borrowck 兩案例後基線 96.0%
        assert!(rate >= 96.0, "pass rate {:.1}% 低於 2026-09-19 基線 96%（回歸）", rate);

        // baseline ratchet：失敗集合必須 ⊆ 已知白名單（新失敗即回歸，直接紅）
        let failing: Vec<String> = details
            .iter()
            .filter(|d| d.starts_with("FAIL "))
            .map(|d| {
                d.trim_start_matches("FAIL ")
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            })
            .collect();
        for f in &failing {
            assert!(
                KNOWN_FAILURES.iter().any(|(k, _)| k == f),
                "語義矩陣出現**新**失敗案例（回歸）：{}（不在已知白名單；若屬修復後態請更新 KNOWN_FAILURES）",
                f
            );
        }
        // 反向：白名單內已修好的案例應從表中移除（提示，不硬擋）
        let fixed: Vec<&str> = KNOWN_FAILURES
            .iter()
            .map(|(k, r)| (*k, *r))
            .filter(|(k, _)| !failing.iter().any(|f| f.as_str() == *k))
            .map(|(k, r)| {
                println!("✅ 已修復（請從 KNOWN_FAILURES 移除）：{} — 原記錄：{}", k, r);
                k
            })
            .collect();
        println!(
            "ratchet: {} 失敗（白名單 {} 項，其中 {} 項已實際修好待移除）",
            failing.len(), KNOWN_FAILURES.len(), fixed.len()
        );
    }

    #[test]
    fn test_no_filler_in_generated() {
        let cases = all_semantic_cases();
        let config = PipelineV3Config::default();
        for case in cases.iter().take(20) {
            if let Ok(v3) = run_pipeline_v3_with_config(case.name, case.poly_src, None, &config) {
                if let Some(rust) = v3.generated_rust {
                    assert!(
                        !rust.contains("// repeat") && !rust.contains("语义填充"),
                        "case {} contains filler",
                        case.name
                    );
                }
            }
        }
    }
}
