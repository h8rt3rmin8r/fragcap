# Quickstart: verifying S066

No game, no capture driver, and no live Steam install are needed for any of this;
every scenario below runs against a fixture tree or an in-memory store, the same
posture the rest of the target-store test suite already uses.

## 1. Music-type install path (User Story 1, #166)

```bash
cargo test -p fragcap-steam --lib library:: -- --nocapture
```

Expected: a new fixture test builds a Steam root with one `Music`-typed app
(`appcache/appinfo.vdf` carrying `common/type "Music"`, manifest installed under
`steamapps/music/<installdir>`) alongside an ordinary game under
`steamapps/common/<installdir>`. `discover_in` resolves the music app's
`install_dir` to the real `music/` path with no warning, and the ordinary game's
resolution is unchanged.

## 2. Music apps are not registered (User Story 1, #166, R-6)

```bash
cargo test -p fragcap --features targets --test steam_source -- --nocapture
```

Expected: `SteamSource::discover` over the same fixture root produces one
candidate (the game), and the `Discovery.account` shows the music app counted
under `considered_not_a_game` with `is_conserved()` still true.

## 3. A missing install root renders as a warning (User Story 2, #167)

```bash
cargo test -p fragcap-cli --test cli_targets -- missing_install
```

Expected: a fixture store with one target whose `install_root` points at a
nonexistent path renders that row's SENSITIVITIES cell prefixed with `install
folder not found` (plain text with `NO_COLOR=1`, ANSI-wrapped without it), while
every other row in the same fixture is byte-identical to the pre-feature golden.
The hero listing's trailing `fragcap capture <n>` line does not name the missing
row.

## 4. A renamed title is findable by any of its names (User Story 3, #173)

```bash
cargo test -p fragcap-targets --test selector -- --nocapture
cargo test -p fragcap-targets --lib register:: -- --nocapture
```

Expected: registering a candidate with `name = "Trapped with Ivy & Piper"`,
`folder_name = Some("Escape from Ivy & Piper")`, `executable_hint =
Some("TrappedWithIvyAndPiper-EA.exe")` resolves via `resolve_positional` for the
tokens `"trapped"`, `"escape"`, `"ivy"`, and `"TrappedWithIvyAndPiper"`, each to
the same single target. `targets show` against it prints a divergence note
naming both names; a second fixture target whose `folder_name` is only a casing
variant of `name` prints no note.

## 5. The `&` handle decision

```bash
cargo test -p fragcap-targets --test handle_vectors
```

Expected: `derive_handle("Trapped with Ivy & Piper", None, 1)` returns
`"trapped_with_ivy_and_piper"`.

## 6. Full gate

```bash
cargo xtask ci
```

Runs `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D
warnings`, `cargo test --workspace --locked`, `cargo xtask lint`, `cargo xtask
deps`, and `cargo xtask license`, in that order, in the foreground.
