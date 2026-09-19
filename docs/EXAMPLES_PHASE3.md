# Phase3 示例集 — 9 特性 × 3 場景 = 27 個 .poly 文件

## 1. struct/enum/impl/trait

### SAT
```poly
# @intent: struct + enum + impl + trait SAT
struct Point { x: i32, y: i32 }
enum Option<T> { Some(T), None }
trait Display { fn fmt(&self) -> String; }
impl Point { fn new(x: i32, y: i32) -> Point { Point { x: x, y: y } } fn norm(&self) -> i32 { self.x * self.x + self.y * self.y } }
impl Display for Point { fn fmt(&self) -> String { String::from("Point") } }
fn main() { let p = Point { x: 3, y: 4 }; let n = p.norm(); }
```

### UNSAT（字段類型不匹配）
```poly
# @intent: struct UNSAT — 字段類型錯誤
struct Point { x: i32, y: i32 }
fn main() { let p = Point { x: true, y: 4 }; }
```

### UNKNOWN（trait 未實現）
```poly
# @intent: trait UNKNOWN — 未實現 Display
struct Point { x: i32, y: i32 }
trait Display { fn fmt(&self) -> String; }
fn main() { let p = Point { x: 1, y: 2 }; let s = p.fmt(); }
```

## 2. Vec/String/HashMap

### SAT
```poly
# @intent: Vec String HashMap SAT
# @import: vec, string, hashmap
# @type-universe: Vec<i32>, String, HashMap<String,i32>
fn main() {
  let v: Vec<i32> = Vec_new();
  Vec_push(&mut v, 1);
  let s = String_from("hi");
  let m: HashMap<String,i32> = HashMap_new();
  HashMap_insert(&mut m, s, 42);
}
```

### UNSAT（類型不統一）
```poly
# @intent: Vec UNSAT — push 類型不匹配
fn main() {
  let v: Vec<i32> = Vec_new();
  Vec_push(&mut v, true);
}
```

### UNKNOWN（HashMap key 重複未知）
```poly
# @intent: HashMap UNKNOWN — key 唯一性需證明
fn main() {
  let m: HashMap<String,i32> = HashMap_new();
  HashMap_insert(&mut m, String_from("a"), 1);
  HashMap_insert(&mut m, String_from("a"), 2);
}
```

## 3. loop/while/for + 契約

### SAT
```poly
# @intent: loop SAT
# @fuel: 5
# @invariant: x >= 0
fn main() {
  let mut x = 0;
  while x < 10 { x = x + 1; }
}
```

### UNSAT（invariant 違反）
```poly
# @intent: loop UNSAT — invariant 違反
# @fuel: 3
# @invariant: x < 5
fn main() {
  let mut x = 0;
  while x < 10 { x = x + 10; }
}
```

### UNKNOWN（fuel 不足）
```poly
# @intent: loop UNKNOWN — fuel 不足
# @fuel: 1
fn main() {
  let mut x = 0;
  while x < 100 { x = x + 1; }
}
```

## 4. match

### SAT
```poly
# @intent: match SAT
enum Option<T> { Some(T), None }
fn main() {
  let opt = Option::Some(5);
  let y = match opt { Some(v) => v, None => 0 };
}
```

### UNSAT（非窮舉）
```poly
# @intent: match UNSAT — 非窮舉
enum Option<T> { Some(T), None }
fn main() {
  let opt = Option::Some(5);
  let y = match opt { Some(v) => v };
}
```

### UNKNOWN（複雜模式）
```poly
# @intent: match UNKNOWN — 嵌套模式
enum Result<T,E> { Ok(T), Err(E) }
enum Option<T> { Some(T), None }
fn main() {
  let r: Result<Option<i32>, String> = Result::Ok(Option::Some(5));
  let y = match r { Result::Ok(Option::Some(v)) => v, _ => 0 };
}
```

## 5. mod 樹

### SAT
```poly
# @intent: mod SAT
mod geometry { pub struct Point { pub x: i32, pub y: i32 } pub fn origin() -> Point { Point { x: 0, y: 0 } } }
fn main() { let p = geometry::Point { x: 1, y: 2 }; }
```

### UNSAT（私有訪問）
```poly
# @intent: mod UNSAT — 私有訪問
mod geometry { struct Point { x: i32, y: i32 } }
fn main() { let p = geometry::Point { x: 1, y: 2 }; }
```

### UNKNOWN（複雜路徑）
```poly
# @intent: mod UNKNOWN — 複雜路徑解析
mod a { pub mod b { pub struct C { pub x: i32 } } }
mod d { use crate::a::b::C; pub fn make() -> C { C { x: 1 } } }
fn main() { let c = d::make(); }
```

## 6. async/await

### SAT
```poly
# @intent: async SAT
async fn fetch() -> i32 { 42 }
fn main() { let f = fetch(); }
```

### UNSAT（await 非 Future）
```poly
# @intent: async UNSAT — await 非 Future
fn main() { let x = 5.await; }
```

### UNKNOWN（多重 await）
```poly
# @intent: async UNKNOWN — 多重 await 狀態機
async fn fetch() -> i32 { 42 }
async fn fetch2() -> i32 { 100 }
fn main() { let x = fetch().await + fetch2().await; }
```

## 7. I/O

### SAT
```poly
# @intent: I/O SAT
fn main() { println("hello"); }
```

### UNSAT（no-io 違反）
```poly
# @intent: I/O UNSAT — no-io 違反
# @no-io
fn main() { println("should fail"); }
```

### UNKNOWN（pure 與 I/O）
```poly
# @intent: I/O UNKNOWN — pure 函數含 I/O
# @pure: true
fn main() { println("pure violation?"); }
```

## 8. unsafe

### SAT
```poly
# @intent: unsafe SAT
# @unsafe-allowed
fn main() { unsafe { let p: *mut i32 = 0 as *mut i32; } }
```

### UNSAT（raw ptr 無 unsafe）
```poly
# @intent: unsafe UNSAT — raw ptr 無 unsafe 塊
fn main() { let p: *mut i32 = 0 as *mut i32; unsafe { *p = 5; } }
```

### UNKNOWN（複雜 raw ptr）
```poly
# @intent: unsafe UNKNOWN — 複雜 raw ptr 鏈
# @unsafe-allowed
fn main() {
  let mut x = 5;
  let p: *mut i32 = &mut x as *mut i32;
  unsafe { *p = 10; }
  let y = x;
}
```

## 9. lifetime

### SAT
```poly
# @intent: lifetime SAT
# @lifetime 'a: 'b
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { x }
fn main() { let s = longest("a", "b"); }
```

### UNSAT（lifetime 環）
```poly
# @intent: lifetime UNSAT — 環
# @lifetime 'a: 'b
# @lifetime 'b: 'a
fn main() {}
```

### UNKNOWN（複雜 outlives）
```poly
# @intent: lifetime UNKNOWN — 複雜 outlives
# @lifetime 'a: 'b
# @lifetime 'b: 'c
# @lifetime 'c: 'd
fn longest<'a, 'b, 'c>(x: &'a str, y: &'b str, z: &'c str) -> &'a str where 'a: 'b, 'b: 'c { x }
fn main() {}
```

所有示例符合 .poly v0.2 語法，編碼見 docs/PHASE3_REPORT.md
