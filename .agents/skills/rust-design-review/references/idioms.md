# Idioms Checklist

Source: https://rust-unofficial.github.io/patterns/idioms/index.html

All of these are community-agreed best practices. Breaking them requires justification.

---

## 1. Use Borrowed Types for Arguments

**Rule**: Accept the widest possible type via deref coercion.

| Instead of | Prefer |
|------------|--------|
| `&String` | `&str` |
| `&Vec<T>` | `&[T]` |
| `&Box<T>` | `&T` |

**Why**: `&String` forces callers to own a `String`; `&str` also accepts `&String`
(via deref coercion), string literals, and string slices. Two layers of indirection
become one.

**Flag if you see**: A function parameter typed `&String`, `&Vec<_>`, or `&Box<_>`.

```rust
// ❌
fn greet(name: &String) { ... }

// ✅
fn greet(name: &str) { ... }
```

---

## 2. String Concatenation with `format!`

**Rule**: Use `format!` to concatenate strings, especially with mixed literals and values.

**Why**: Clearer, more readable. The `+` operator consumes the left operand and requires
`&str` on the right, leading to awkward `&`-ing.

**Exception**: In hot paths, a pre-allocated `String::with_capacity` + `push_str` is faster.

```rust
// ❌ (works but awkward)
let greeting = "Hello ".to_owned() + name + "!";

// ✅
let greeting = format!("Hello {name}!");
```

---

## 3. Constructors

**Rule**: Use an associated function named `new` as the primary constructor.
Implement `Default` whenever a sensible zero-value exists.

**Why**: Rust has no language-level constructors. `new` is the community convention;
users expect it. `Default` allows use in `*or_default()` stdlib methods, struct
update syntax (`..Default::default()`), and derive macros.

**Flag if you see**:
- Struct with all-public fields but no `new()` or `Default` impl (library crate).
- Manually written `Default` impl when `#[derive(Default)]` would work.
- `new` and `default()` that do different things without documentation.

```rust
// ✅
#[derive(Default)]
pub struct Config {
    pub timeout: Duration,
    pub retries: u32,
}

impl Config {
    pub fn new() -> Self { Self::default() }
}
```

---

## 4. The `Default` Trait

**Rule**: Implement `Default` for any type that has a sensible "empty" state.

**Why**: Unlocks `Option::unwrap_or_default()`, `HashMap::entry().or_default()`,
struct update syntax, and generic bounds on `Default`.

**Flag if you see**:
- All fields implement `Default` but `#[derive(Default)]` is absent.
- Manual `Default` impl that just calls `new()` — just derive it instead.

```rust
// ✅ — derive when all fields implement Default
#[derive(Default, Debug)]
struct State {
    count: usize,      // defaults to 0
    name: String,      // defaults to ""
    active: bool,      // defaults to false
}
```

---

## 5. Collections as Smart Pointers (Deref)

**Rule**: Implement `Deref` on owning collections so they coerce to their borrowed view.

**Why**: `Vec<T>` → `&[T]`, `String` → `&str`, `Box<T>` → `&T`. Most methods live
on the borrowed slice/str, so callers get them for free.

**Flag if you see**:
- A custom collection that doesn't implement `Deref` to a slice/borrowed view.
- (Anti-pattern sibling) Using `Deref` to fake inheritance — see anti-patterns.

---

## 6. Finalisation in Destructors

**Rule**: Use `Drop` impl (not explicit cleanup calls) to release resources.

**Why**: Rust has no `finally` block. Destructors run on early return and panic
unwind. Manual cleanup calls can be forgotten.

**Flag if you see**:
- `close()` / `cleanup()` / `release()` methods that must be called by hand.
- Resource types without a `Drop` impl.

```rust
// ✅
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
```

---

## 7. `mem::take` / `mem::replace` in Enum Mutations

**Rule**: When mutating an enum variant in place, use `mem::take` or `mem::replace`
instead of cloning to satisfy the borrow checker.

**Why**: Avoids an unnecessary allocation. `mem::take` swaps out a value with its
`Default`, returning the original owned value.

**Flag if you see**:
- `.clone()` called on a value inside `&mut MyEnum` arm just to move it into a
  new variant.

```rust
// ❌ unnecessary clone
*e = MyEnum::B { name: name.clone() };

// ✅
*e = MyEnum::B { name: std::mem::take(name) };
```

---

## 8. On-Stack Dynamic Dispatch

**Rule**: Use `&dyn Trait` bound to a stack variable instead of `Box<dyn Trait>`
when heap allocation is unnecessary.

**Why**: Since Rust 1.79 the compiler extends temporary lifetimes automatically.
No heap allocation needed for conditional dispatch over short-lived values.

```rust
// ✅ — no Box needed since Rust 1.79
let readable: &mut dyn io::Read = if flag { &mut stdin() } else { &mut file };
```

---

## 9. `Option` as an Iterator

**Rule**: Use `Option`'s `IntoIterator` impl to integrate with iterator chains
instead of manual `if let Some`.

**Why**: Cleaner composition with `.extend()`, `.chain()`, `.filter_map()`.

```rust
// ❌ verbose
if let Some(v) = maybe_val { vec.push(v); }

// ✅
vec.extend(maybe_val);
```

---

## 10. Selective Closure Captures

**Rule**: When only some variables need to be moved / cloned / borrowed into a
closure, use a rebinding block inside `{ }` before the `move` closure.

**Why**: Makes capture intent explicit; the narrowly-scoped copies are dropped
immediately when the closure runs.

```rust
// ✅
let closure = {
    let num2 = num2.clone();   // only num2 is cloned
    let num3 = num3.as_ref();  // num3 is borrowed, not moved
    move || *num1 + *num2 + *num3
};
```

---

## 11. `#[non_exhaustive]` for Forward Compatibility

**Rule**: Apply `#[non_exhaustive]` to public structs and enums in library crates
when new fields/variants may be added in minor versions.

**Why**: Prevents downstream code from exhaustively matching or constructing the
type, giving you room to grow without a major version bump.

**Trade-off**: Forces `..` in patterns and removes direct construction — adds
ergonomic friction. Use deliberately; prefer major versions for frequent changes.

```rust
#[non_exhaustive]
pub enum Status {
    Ok,
    Pending,
    // future variants won't break callers
}
```

---

## 12. Temporary Mutability

**Rule**: Rebind a mutable variable as immutable once mutation is complete.

**Why**: Documents intent; compiler enforces that the value is not accidentally
mutated later.

```rust
// ✅
let mut data = fetch();
data.sort();
let data = data; // now immutable
```

---

## 13. Return Consumed Argument on Error

**Rule**: If a fallible function takes ownership of a value, return that value
inside the `Err` variant so callers can retry without re-creating it.

**Why**: Avoids forcing callers to clone an argument just to enable retry logic.
Mirrors `String::from_utf8` → `FromUtf8Error::into_bytes`.

```rust
// ✅
pub struct SendError(pub String);

pub fn send(value: String) -> Result<(), SendError> {
    if failed() { return Err(SendError(value)); }
    Ok(())
}
```

---

## 14. Easy Doc Initialization (Doc Boilerplate)

**Rule**: In doc examples that need complex setup, wrap the real assertion in a
helper function (`# fn example(obj: MyThing)`) to avoid repeating boilerplate
across every method's example.

**Why**: Reduces copy-paste in docs; the code still compiles and type-checks.

---

## 15. FFI Idioms (when working with C interop)

1. **Accepting strings**: Use `&CStr` (borrowed), not a copied `CString`. Minimises `unsafe`.
2. **Passing strings**: Bind `CString` to a named variable before calling the C function;
   a temporary `CString` drops before the pointer is used (dangling pointer bug).
3. **Error codes**: Convert flat enums to integer codes via `impl From<MyError> for libc::c_int`.
4. **Minimise unsafe scope**: Wrap the unsafe pointer op in the smallest possible block; convert
   to safe Rust types immediately.
