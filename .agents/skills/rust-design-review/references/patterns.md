# Patterns & Anti-patterns Catalogue

Source: https://rust-unofficial.github.io/patterns/

---

## ANTI-PATTERNS (always 🔴 Must Fix)

### AP-1: Clone to Satisfy the Borrow Checker

**Symptom**: `.clone()` called on data inside a `&mut` reference, or anywhere the
comment could be "just to make the borrow checker happy".

**Why bad**: Unnecessary heap allocation; obscures intent; masks a design that needs
`mem::take`, `mem::replace`, or struct decomposition.

**Fix**: Use `mem::take(field)` (requires `Default`), `mem::replace(field, fallback)`,
or restructure so ownership flows naturally.

```rust
// ❌
fn transition(e: &mut MyEnum) {
    if let MyEnum::A { name, .. } = e {
        *e = MyEnum::B { name: name.clone() }; // clone only to move
    }
}

// ✅
fn transition(e: &mut MyEnum) {
    if let MyEnum::A { name, .. } = e {
        *e = MyEnum::B { name: std::mem::take(name) };
    }
}
```

---

### AP-2: Deref Polymorphism

**Symptom**: Using `Deref<Target = Base>` to simulate OOP inheritance — the "child"
type delegates all methods to `Base` through automatic deref.

**Why bad**:
- Confuses ownership with subtyping (they are not the same in Rust).
- Leaks internal methods of `Base` into the `Child` public API.
- `self`-consuming methods on `Base` consume the `Child` (surprising).
- Breaks because `impl Trait for Child` doesn't automatically give `impl Trait for Base`.

**Fix**: Prefer composition (hold `Base` as a field, delegate explicitly) or traits
(define shared behaviour in a trait; implement it for both).

```rust
// ❌ — "inheritance" via Deref
struct Child(Base);
impl Deref for Child {
    type Target = Base;
    fn deref(&self) -> &Base { &self.0 }
}

// ✅ — explicit delegation or trait abstraction
trait Animal { fn name(&self) -> &str; }
impl Animal for Child { fn name(&self) -> &str { self.0.name() } }
```

---

### AP-3: `#[deny(warnings)]` in Library Code

**Symptom**: `#![deny(warnings)]` in `lib.rs` or anywhere in a published crate.

**Why bad**: Rustc adds new warnings over time. A new Rust release can break your
downstream users who didn't change any code.

**Fix**: Use `#![warn(...)]` for the specific lints you care about in source.
Run `RUSTFLAGS="-D warnings" cargo check` (or equivalent) in CI only.

```toml
# In CI (e.g. GitHub Actions):
env:
  RUSTFLAGS: "-D warnings"
```

---

## BEHAVIOURAL PATTERNS

### BP-1: Command Pattern

**When to use**: You need to represent actions as first-class objects that can be
queued, logged, undone, or passed across boundaries.

**Rust approaches** (choose based on context):

| Approach | Best when |
|----------|-----------|
| Trait objects (`Box<dyn Command>`) | Commands are structs with state/multiple methods |
| Function pointers `fn() -> T` | Commands are pure, stateless functions |
| `Box<dyn Fn()>` | Commands are closures that capture state |

```rust
// Trait objects approach
pub trait Migration {
    fn execute(&self) -> &str;
    fn rollback(&self) -> &str;
}
```

---

### BP-2: RAII with Guards

**When to use**: A resource (lock, file, network connection, temp file) must be
released exactly once, even on early return or panic.

**How it works**: The guard struct holds the resource reference. `Drop` releases it.
The borrow checker ensures the guard outlives any reference derived from it.

```rust
// Classic example: MutexGuard
let guard = mutex.lock().unwrap();
// guard.deref() gives &T; dropped automatically at end of scope
```

**Key subtleties**:
- Assign the guard to a `_named` variable, never just `_` (which drops immediately).
- Don't store the guard in an `Rc`/`Arc` — then it outlives the scope.

---

### BP-3: Strategy Pattern → Just Use Traits

**When OOP would use Strategy**: You want interchangeable algorithms.

**In Rust**: Simply pass a generic `impl Fn(...)` or `impl MyTrait`. No strategy
object wrapper needed.

```rust
// ✅ — pass behaviour directly
fn sort_by<T, F: Fn(&T, &T) -> std::cmp::Ordering>(data: &mut [T], cmp: F) {
    data.sort_by(cmp);
}
```

---

### BP-4: Visitor Pattern

**When to use**: You need to perform many different operations over a heterogeneous
tree/graph of types without modifying those types.

**In Rust**: Works well with enums + match (the enum is the element hierarchy,
`match` is the dispatch). For open extensibility, use trait objects.

```rust
// Enum-based visitor
enum Expr { Num(f64), Add(Box<Expr>, Box<Expr>) }

fn eval(e: &Expr) -> f64 {
    match e {
        Expr::Num(n) => *n,
        Expr::Add(l, r) => eval(l) + eval(r),
    }
}
```

---

## CREATIONAL PATTERNS

### CP-1: Builder Pattern

**When to use**: A struct has many optional parameters, or construction has multiple
steps / validation that shouldn't live in a giant `new(a, b, c, d, ...)`.

**Rust idioms for builders**:
- The builder owns its data (no lifetimes needed in simple cases).
- `build()` returns `Result<T, E>` when validation can fail.
- Setter methods take `self` (consuming) or `&mut self` (non-consuming/fluent).
- Consider `derive_builder` crate for boilerplate-heavy cases.

```rust
// ✅ consuming builder
pub struct ServerBuilder {
    host: String,
    port: u16,
    max_conn: usize,
}

impl ServerBuilder {
    pub fn new(host: impl Into<String>) -> Self { ... }
    pub fn port(mut self, p: u16) -> Self { self.port = p; self }
    pub fn max_connections(mut self, n: usize) -> Self { self.max_conn = n; self }
    pub fn build(self) -> Result<Server, ConfigError> { ... }
}

let server = ServerBuilder::new("localhost").port(8080).build()?;
```

**Flag**: A public struct with 5+ fields all passed to `new()` — suggest Builder.

---

### CP-2: Fold (Recursive Data → Single Value)

**When to use**: Collapsing a recursive / tree data structure into a result
(e.g., evaluating an AST).

**In Rust**: Implement a `fold` method on the enum / trait. Works like
`Iterator::fold` but for tree structures.

---

## STRUCTURAL PATTERNS

### SP-1: Newtype Pattern

**When to use**:
- Give a primitive (`u32`, `String`, `f64`) domain meaning and type safety.
- Implement a foreign trait on a foreign type (orphan rule workaround).
- Hide implementation details of a type from the public API.

**How**: Wrap in a single-field tuple struct. Derive or delegate traits as needed.

```rust
// ✅ — type-safe meters vs seconds
struct Metres(f64);
struct Seconds(f64);

// Can't accidentally pass Seconds where Metres expected
fn speed(distance: Metres, time: Seconds) -> f64 {
    distance.0 / time.0
}
```

**Extras**:
- `impl Deref<Target = InnerType>` gives transparent access (but see Deref anti-pattern —
  only safe here because the newtype IS the inner type, not a subtype).
- `impl From<Inner> for Newtype` + `impl From<Newtype> for Inner` for cheap conversion.

**Flag**: Bare `u32`/`String`/`f64` used as a domain concept passed around between
multiple functions — suggest Newtype.

---

### SP-2: Typestate Pattern

**When to use**: An object has distinct states and only certain operations are valid
in each state. Encode this in the type system so invalid transitions are
compile errors.

```rust
// ✅ — can only send on an Open connection
struct Connection<S> { state: S, stream: TcpStream }
struct Open;
struct Closed;

impl Connection<Open> {
    pub fn send(&mut self, data: &[u8]) { ... }
    pub fn close(self) -> Connection<Closed> { ... }
}
// Connection<Closed> has no send() — compile error if misused
```

**Flag**: Methods that `panic!` or return an error because the object is "in the wrong
state" — suggest Typestate.

---

### SP-3: Struct Decomposition for Independent Borrowing

**When to use**: A method needs to mutably borrow two disjoint fields of a struct
simultaneously, which the borrow checker blocks on `self`.

**Fix**: Decompose the struct into sub-structs so each sub-struct can be borrowed
independently via `&mut self.subsA` and `&mut self.subsB`.

```rust
// ❌ — can't borrow self.a and self.b mutably at once through self
struct Big { a: Vec<u8>, b: Vec<u8> }

// ✅
struct AData(Vec<u8>);
struct BData(Vec<u8>);
struct Big { a: AData, b: BData }
// now &mut big.a and &mut big.b are disjoint borrows
```

---

### SP-4: Custom Traits to Avoid Complex Type Bounds (New 2025)

**When to use**: A generic function accumulates many trait bounds that are always
used together (`T: Debug + Clone + Serialize + Send + 'static`).

**Fix**: Define a "supertrait" that bundles them. Callers implement one trait;
generic code uses one bound.

```rust
pub trait Component: Debug + Clone + Send + 'static {}
impl<T: Debug + Clone + Send + 'static> Component for T {}

fn process<T: Component>(item: T) { ... }
```

---

### SP-5: Contain Unsafety in Small Modules

**Rule**: Every `unsafe` block must be the smallest possible scope. Convert raw
pointers to safe Rust types immediately after the `unsafe` block ends.

**Why**: Easier to audit; limits the blast radius of UB.

```rust
// ✅
let s: &str = unsafe {
    // SAFETY: `ptr` is guaranteed valid UTF-8 by the caller
    std::str::from_utf8_unchecked(slice)
};
// `s` is now safe &str — no unsafe needed from here
```

---

## FUNCTIONAL PATTERNS

### FP-1: Iterator Chains over Manual Loops

Prefer chaining iterator adaptors (`map`, `filter`, `flat_map`, `fold`, `any`,
`all`) over explicit `for` loops when the intent is a transformation or reduction.

```rust
// ❌ verbose
let mut result = Vec::new();
for item in items {
    if item.active { result.push(item.id); }
}

// ✅
let result: Vec<_> = items.iter()
    .filter(|i| i.active)
    .map(|i| i.id)
    .collect();
```

---

### FP-2: `Option` and `Result` Combinators

Prefer combinators over nested `match` / `if let`.

| Instead of | Prefer |
|------------|--------|
| `match opt { Some(v) => f(v), None => default }` | `opt.map_or(default, f)` |
| `if let Some(v) = opt { Some(f(v)) } else { None }` | `opt.map(f)` |
| `match res { Ok(v) => Ok(f(v)), Err(e) => Err(e) }` | `res.map(f)` |
| `res.unwrap_or_else(\|_\| panic!("msg"))` | `res.expect("msg")` |
| chained `?` with type mismatch | `res.map_err(Into::into)?` |
