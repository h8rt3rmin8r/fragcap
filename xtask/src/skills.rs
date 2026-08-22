// SPDX-License-Identifier: Apache-2.0

//! The vendored skill set gate.
//!
//! `cargo xtask skills` holds `.agents/skills/`, `skills-lock.json`, and git's
//! index to agreement with one another. Three assertions: every lock entry has
//! a directory and the `skillPath` file it names; every vendored directory has
//! a lock entry; and the working tree and the index carry the same vendored
//! files, checked in both directions.
//!
//! The third is the one with a scar behind it. `.agents/skills/debug/` sat on
//! disk and in the lock, and uncommitted, from the founding commit until slice
//! S071, because `.gitignore` carried a bare `debug` pattern inherited from a
//! Cargo build-artifact template. Nothing noticed, because nothing read this
//! file. That is why the assertion asks git's index rather than reparsing
//! ignore rules: the index is what actually determines what a clone receives,
//! and it catches exclusion by any mechanism rather than only that one.
//!
//! It is checked both ways because each direction is a different real defect,
//! and the first version of this gate checked only one. A file removed from the
//! working tree without leaving the index is the mirror case: a clone receives
//! a file the author no longer has. That version passed such a tree while
//! reporting a silently smaller file count as agreement. Found in review of
//! pull request #200.
//!
//! The exit contract is the house 0/1/2: 0 the set agrees, 1 it does not, 2 the
//! gate could not run (git absent, the lock unreadable or shaped unexpectedly,
//! the skills directory missing). The last matters more here than usual. This
//! gate exists because an unverified file drifted for the life of the project,
//! so a gate that cannot read the file must say so rather than pass.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// Where vendored skills live, repository-relative. Forward slashes, because
/// this string is compared against `git ls-files` output.
const SKILLS_DIR: &str = ".agents/skills";

/// The provenance file the gate reconciles against.
const LOCK_FILE: &str = "skills-lock.json";

/// Directories the spec-kit CLI owns. It records them in
/// `.specify/integrations/*.manifest.json` and `skills-lock.json` deliberately
/// omits them, which its own `note` field states. Recognized by prefix rather
/// than by an enumerated list, so a new spec-kit command does not fail the gate
/// on the day it is generated.
const CLI_OWNED_PREFIX: &str = "speckit-";

fn cannot_run(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

// ---------------------------------------------------------------------------
// A strict reader for the subset of JSON this gate needs.
//
// `skills-lock.json` is machine written by a tool this repository does not
// contain, so the reader is deliberately strict: anything it does not
// recognize is an error rather than a guess. A guess here would produce either
// a false failure against a correct file or, worse, a false pass. Both are
// exit 2 material, and the caller treats them as such.
// ---------------------------------------------------------------------------

/// The JSON shapes the gate distinguishes. Numbers, booleans, and null are all
/// `Other`: the gate reads two string fields and never inspects anything else.
enum Json {
    Str(String),
    Obj(BTreeMap<String, Json>),
    Other,
}

impl Json {
    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_obj(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Obj(m) => Some(m),
            _ => None,
        }
    }
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Reader<'a> {
    fn new(s: &'a str) -> Self {
        Reader {
            b: s.as_bytes(),
            i: 0,
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.b.get(self.i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.i += 1;
        }
    }

    fn expect(&mut self, c: u8) -> io::Result<()> {
        self.skip_ws();
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(cannot_run(format!(
                "{LOCK_FILE}: expected '{}' at byte {}",
                c as char, self.i
            )))
        }
    }

    fn string(&mut self) -> io::Result<String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let c = *self
                .b
                .get(self.i)
                .ok_or_else(|| cannot_run(format!("{LOCK_FILE}: unterminated string")))?;
            self.i += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let e = *self
                        .b
                        .get(self.i)
                        .ok_or_else(|| cannot_run(format!("{LOCK_FILE}: unterminated escape")))?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = self
                                .b
                                .get(self.i..self.i + 4)
                                .ok_or_else(|| cannot_run(format!("{LOCK_FILE}: short \\u")))?;
                            let hex = std::str::from_utf8(hex)
                                .map_err(|_| cannot_run(format!("{LOCK_FILE}: bad \\u")))?;
                            let n = u32::from_str_radix(hex, 16)
                                .map_err(|_| cannot_run(format!("{LOCK_FILE}: bad \\u")))?;
                            self.i += 4;
                            // Surrogate halves are rejected rather than
                            // reassembled: no key or value the gate reads is
                            // outside the basic multilingual plane, and a
                            // half-decoded name is a wrong answer.
                            out.push(char::from_u32(n).ok_or_else(|| {
                                cannot_run(format!("{LOCK_FILE}: unsupported \\u{hex}"))
                            })?);
                        }
                        other => {
                            return Err(cannot_run(format!(
                                "{LOCK_FILE}: unknown escape \\{}",
                                other as char
                            )))
                        }
                    }
                }
                // Multi-byte UTF-8 arrives here one byte at a time; push the
                // raw bytes and let the final string carry them.
                _ => {
                    let start = self.i - 1;
                    let mut end = self.i;
                    while self.b.get(end).is_some_and(|b| b & 0xC0 == 0x80) {
                        end += 1;
                    }
                    let s = std::str::from_utf8(&self.b[start..end])
                        .map_err(|_| cannot_run(format!("{LOCK_FILE}: invalid UTF-8")))?;
                    out.push_str(s);
                    self.i = end;
                }
            }
        }
    }

    /// Parse one complete document: a value, then end of input.
    ///
    /// The EOF requirement is not pedantry. Without it a lock followed by a
    /// second value, or by trailing garbage, parses to the leading object and
    /// the gate accepts a file no JSON reader would, which is the opposite of
    /// the fail-closed contract this module documents.
    fn document(&mut self) -> io::Result<Json> {
        let v = self.value()?;
        self.skip_ws();
        if self.i != self.b.len() {
            return Err(cannot_run(format!(
                "{LOCK_FILE}: trailing content after the document, at byte {}",
                self.i
            )));
        }
        Ok(v)
    }

    fn value(&mut self) -> io::Result<Json> {
        self.skip_ws();
        match self.b.get(self.i) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(_) => {
                self.scalar()?;
                Ok(Json::Other)
            }
            None => Err(cannot_run(format!("{LOCK_FILE}: unexpected end of input"))),
        }
    }

    fn object(&mut self) -> io::Result<Json> {
        self.expect(b'{')?;
        let mut map = BTreeMap::new();
        self.skip_ws();
        if self.b.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.skip_ws();
            let k = self.string()?;
            self.expect(b':')?;
            let v = self.value()?;
            map.insert(k, v);
            self.skip_ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(map));
                }
                _ => return Err(cannot_run(format!("{LOCK_FILE}: malformed object"))),
            }
        }
    }

    fn array(&mut self) -> io::Result<Json> {
        self.expect(b'[')?;
        self.skip_ws();
        if self.b.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Json::Other);
        }
        loop {
            self.value()?;
            self.skip_ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Other);
                }
                _ => return Err(cannot_run(format!("{LOCK_FILE}: malformed array"))),
            }
        }
    }

    /// Consume a number, `true`, `false`, or `null` without interpreting it.
    fn scalar(&mut self) -> io::Result<()> {
        let start = self.i;
        while self
            .b
            .get(self.i)
            .is_some_and(|c| !matches!(c, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r'))
        {
            self.i += 1;
        }
        if self.i == start {
            return Err(cannot_run(format!("{LOCK_FILE}: empty value")));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The inventory, and the pure check over it.
// ---------------------------------------------------------------------------

/// One lock entry, reduced to what the gate reconciles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub skill_path: String,
}

/// The three views the gate compares. Assembled from the filesystem and git by
/// [`inventory`], and checked by [`check`], which is pure so the assertions are
/// testable without a working tree or a git binary.
#[derive(Debug, Default)]
pub struct Inventory {
    /// Lock entries, in file order.
    pub lock: Vec<Entry>,
    /// Vendored directory names present under `.agents/skills/`, excluding
    /// CLI-owned ones.
    pub dirs: BTreeSet<String>,
    /// Repository-relative paths of every file under a vendored directory.
    pub files: BTreeSet<String>,
    /// Repository-relative paths git tracks under `.agents/skills/`.
    pub tracked: BTreeSet<String>,
}

/// Compare the three views. Returns one message per disagreement, most
/// specific first, or an empty vector when they agree.
///
/// A failure names the entry, directory, or file at fault. A gate that fails
/// without saying what failed is a gate somebody deletes.
pub fn check(inv: &Inventory) -> Vec<String> {
    let mut fails = Vec::new();
    let names: BTreeSet<&str> = inv.lock.iter().map(|e| e.name.as_str()).collect();

    // 1. Every lock entry has a directory, and the file its skillPath names.
    for e in &inv.lock {
        if !inv.dirs.contains(&e.name) {
            fails.push(format!(
                "lock entry '{}' has no directory at {SKILLS_DIR}/{}",
                e.name, e.name
            ));
        } else if !inv.files.contains(&e.skill_path) {
            fails.push(format!(
                "lock entry '{}' names skillPath '{}', which does not exist",
                e.name, e.skill_path
            ));
        }
    }

    // 2. Every vendored directory has a lock entry.
    for d in &inv.dirs {
        if !names.contains(d.as_str()) {
            fails.push(format!(
                "{SKILLS_DIR}/{d} has no entry in {LOCK_FILE}; vendored content carries provenance"
            ));
        }
    }

    // 3. The disk and git's index carry the same vendored files. Checked both
    //    ways, because each direction is a real and different defect.
    //
    //    Present-but-untracked is the one that produced this gate: the debug
    //    skill sat on disk and in the lock, and out of every clone, for the
    //    life of the project. Tracked-but-absent is its mirror, a file removed
    //    from the working tree without leaving the index, where a clone gets a
    //    file the author no longer has. The first version of this gate checked
    //    only the first direction and passed a tree with the second, reporting
    //    a silently smaller file count as agreement. Found in review of pull
    //    request #200.
    for f in &inv.files {
        if !inv.tracked.contains(f) {
            fails.push(format!(
                "{f} is present but not tracked by git; a clone would not receive it"
            ));
        }
    }
    for f in &inv.tracked {
        if !inv.files.contains(f) {
            fails.push(format!(
                "{f} is tracked by git but absent from the working tree; a clone would receive a file this tree does not have"
            ));
        }
    }

    fails
}

/// Whether a directory under `.agents/skills/` belongs to the spec-kit CLI.
fn is_cli_owned(name: &str) -> bool {
    name.starts_with(CLI_OWNED_PREFIX)
}

/// The first path component under `.agents/skills/`, which is the skill
/// directory a file belongs to. `None` for a path outside the skills tree.
///
/// A file sitting directly in `.agents/skills/` yields its own file name. That
/// is deliberate: it is not CLI-owned, so it stays in both the disk view and
/// the index view and the two-way comparison treats it like any other file,
/// rather than being silently exempt from both.
fn skill_of(path: &str) -> Option<&str> {
    path.strip_prefix(SKILLS_DIR)?
        .strip_prefix('/')?
        .split('/')
        .next()
        .filter(|s| !s.is_empty())
}

/// Read `skills-lock.json` into entries, or fail closed.
fn read_lock(root: &Path) -> io::Result<Vec<Entry>> {
    let text = fs::read_to_string(root.join(LOCK_FILE))?;
    let doc = Reader::new(&text).document()?;
    let skills = doc
        .as_obj()
        .and_then(|m| m.get("skills"))
        .and_then(Json::as_obj)
        .ok_or_else(|| cannot_run(format!("{LOCK_FILE}: no top-level 'skills' object")))?;

    let mut out = Vec::new();
    for (name, entry) in skills {
        let skill_path = entry
            .as_obj()
            .and_then(|m| m.get("skillPath"))
            .and_then(Json::as_str)
            .ok_or_else(|| {
                cannot_run(format!(
                    "{LOCK_FILE}: entry '{name}' has no skillPath string"
                ))
            })?;
        out.push(Entry {
            name: name.clone(),
            skill_path: skill_path.to_string(),
        });
    }
    Ok(out)
}

/// Every file beneath `dir`, as repository-relative forward-slash paths.
fn walk(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| cannot_run("path escaped the repository root"))?;
            out.insert(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// What git tracks under `.agents/skills/`, excluding CLI-owned directories.
///
/// The exclusion is what makes the two-way comparison in [`check`] correct.
/// The directory walk never descends into `speckit-*`, so leaving those paths
/// here would report all ten of them as tracked-but-absent on every run.
///
/// `-z` is not decoration: without it git quotes and escapes paths outside a
/// narrow character set, and the escaped form would not compare equal to the
/// path the directory walk produced.
fn tracked(root: &Path) -> io::Result<BTreeSet<String>> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z", "--", SKILLS_DIR])
        .output()
        .map_err(|e| cannot_run(format!("git is required to check tracking: {e}")))?;
    if !out.status.success() {
        return Err(cannot_run(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .filter(|s| skill_of(s).is_some_and(|d| !is_cli_owned(d)))
        .map(str::to_string)
        .collect())
}

/// Assemble the three views from the working tree.
fn inventory(root: &Path) -> io::Result<Inventory> {
    let skills = root.join(SKILLS_DIR);
    if !skills.is_dir() {
        // Reported as could-not-run rather than as drift. A missing skills
        // directory is a louder problem than a stale lock, and calling it lock
        // drift would send the reader to the wrong file.
        return Err(cannot_run(format!("{SKILLS_DIR} does not exist")));
    }

    let mut inv = Inventory {
        lock: read_lock(root)?,
        tracked: tracked(root)?,
        ..Default::default()
    };

    for entry in fs::read_dir(&skills)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| cannot_run("skill directory entry name is not UTF-8"))?
            .to_string();
        if is_cli_owned(&name) {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, &mut inv.files)?;
            inv.dirs.insert(name);
        } else {
            // A loose file directly in .agents/skills/. It belongs to no skill,
            // so no directory assertion covers it, but the index view carries
            // it and the two views must describe the same set or the two-way
            // comparison invents a disagreement.
            let rel = path
                .strip_prefix(root)
                .map_err(|_| cannot_run("path escaped the repository root"))?;
            inv.files.insert(rel.to_string_lossy().replace('\\', "/"));
        }
    }

    Ok(inv)
}

/// Run the gate. Returns the number of disagreements found.
pub fn run(root: &Path) -> io::Result<usize> {
    let inv = inventory(root)?;
    let fails = check(&inv);

    if fails.is_empty() {
        println!(
            "skills: OK  {} lock entr{} resolve to a directory and a SKILL.md",
            inv.lock.len(),
            if inv.lock.len() == 1 { "y" } else { "ies" }
        );
        println!(
            "skills: OK  {} vendored director{} carry provenance",
            inv.dirs.len(),
            if inv.dirs.len() == 1 { "y" } else { "ies" }
        );
        println!(
            "skills: OK  {} vendored file(s) agree between the working tree and git's index",
            inv.files.len()
        );
    } else {
        for f in &fails {
            eprintln!("skills: FAIL {f}");
        }
    }

    Ok(fails.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str) -> Entry {
        Entry {
            name: name.to_string(),
            skill_path: format!("{SKILLS_DIR}/{name}/SKILL.md"),
        }
    }

    /// A tree where all three views agree.
    fn consistent() -> Inventory {
        let mut inv = Inventory {
            lock: vec![entry("shruggie-bash")],
            ..Default::default()
        };
        inv.dirs.insert("shruggie-bash".to_string());
        let f = format!("{SKILLS_DIR}/shruggie-bash/SKILL.md");
        inv.files.insert(f.clone());
        inv.tracked.insert(f);
        inv
    }

    #[test]
    fn a_consistent_tree_passes() {
        assert!(check(&consistent()).is_empty());
    }

    #[test]
    fn a_lock_entry_without_a_directory_fails() {
        let mut inv = consistent();
        inv.lock.push(entry("shruggie-ghost"));
        let fails = check(&inv);
        assert_eq!(fails.len(), 1, "{fails:?}");
        assert!(fails[0].contains("shruggie-ghost"), "{fails:?}");
        assert!(fails[0].contains("no directory"), "{fails:?}");
    }

    #[test]
    fn a_lock_entry_whose_skill_path_is_missing_fails() {
        let mut inv = consistent();
        inv.lock[0].skill_path = format!("{SKILLS_DIR}/shruggie-bash/NOPE.md");
        let fails = check(&inv);
        assert_eq!(fails.len(), 1, "{fails:?}");
        assert!(fails[0].contains("skillPath"), "{fails:?}");
    }

    #[test]
    fn a_directory_without_a_lock_entry_fails() {
        let mut inv = consistent();
        inv.dirs.insert("shruggie-orphan".to_string());
        let fails = check(&inv);
        assert_eq!(fails.len(), 1, "{fails:?}");
        assert!(fails[0].contains("shruggie-orphan"), "{fails:?}");
        assert!(fails[0].contains("provenance"), "{fails:?}");
    }

    /// The regression that produced this gate: a file on disk, named by the
    /// lock, that git does not carry.
    #[test]
    fn an_untracked_vendored_file_fails() {
        let mut inv = consistent();
        inv.files
            .insert(format!("{SKILLS_DIR}/shruggie-bash/assets/extra.md"));
        let fails = check(&inv);
        assert_eq!(fails.len(), 1, "{fails:?}");
        assert!(fails[0].contains("extra.md"), "{fails:?}");
        assert!(fails[0].contains("not tracked"), "{fails:?}");
    }

    /// The mirror of the case above, and the one the first version of this
    /// gate missed: `git rm` was never run, so the index still carries a file
    /// the working tree no longer has.
    #[test]
    fn a_tracked_file_absent_from_the_working_tree_fails() {
        let mut inv = consistent();
        inv.tracked
            .insert(format!("{SKILLS_DIR}/shruggie-bash/assets/fixtures.md"));
        let fails = check(&inv);
        assert_eq!(fails.len(), 1, "{fails:?}");
        assert!(fails[0].contains("fixtures.md"), "{fails:?}");
        assert!(
            fails[0].contains("absent from the working tree"),
            "{fails:?}"
        );
    }

    #[test]
    fn the_skill_directory_of_a_path_is_its_first_component() {
        assert_eq!(
            skill_of(".agents/skills/shruggie-bash/assets/fixtures.md"),
            Some("shruggie-bash")
        );
        assert_eq!(
            skill_of(".agents/skills/speckit-plan/SKILL.md"),
            Some("speckit-plan")
        );
        // A loose file directly in the skills directory yields its own name,
        // so it is excluded from neither view.
        assert_eq!(skill_of(".agents/skills/README.md"), Some("README.md"));
        assert_eq!(skill_of(".agents/skills"), None);
        assert_eq!(skill_of("crates/fragcap/src/lib.rs"), None);
    }

    #[test]
    fn cli_owned_directories_are_recognized_by_prefix() {
        assert!(is_cli_owned("speckit-plan"));
        assert!(is_cli_owned("speckit-anything-added-later"));
        // The vendored autopilot skill is not CLI-owned despite the substring.
        assert!(!is_cli_owned("shruggie-speckit"));
        assert!(!is_cli_owned("shruggie-bash"));
    }

    #[test]
    fn the_reader_extracts_entries() {
        let text = r#"{
          "version": 1,
          "note": "escapes: \" \\ \u00e9",
          "skills": {
            "shruggie-bash": {
              "source": "shruggietech/skills@v1.11.0",
              "sourceType": "vendored",
              "skillPath": ".agents/skills/shruggie-bash/SKILL.md",
              "computedHash": "abc"
            }
          }
        }"#;
        let doc = Reader::new(text).value().expect("parses");
        let skills = doc
            .as_obj()
            .unwrap()
            .get("skills")
            .unwrap()
            .as_obj()
            .unwrap();
        assert_eq!(skills.len(), 1);
        let e = skills.get("shruggie-bash").unwrap().as_obj().unwrap();
        assert_eq!(
            e.get("skillPath").unwrap().as_str().unwrap(),
            ".agents/skills/shruggie-bash/SKILL.md"
        );
        let note = doc.as_obj().unwrap().get("note").unwrap().as_str().unwrap();
        assert_eq!(note, "escapes: \" \\ \u{e9}");
    }

    /// A document is a value and then the end of the file. Without this the
    /// gate accepts a lock followed by a second value or by garbage.
    #[test]
    fn the_reader_refuses_trailing_content() {
        let lock = r#"{"skills": {}}"#;
        assert!(
            Reader::new(lock).document().is_ok(),
            "the bare document parses"
        );
        for trailing in [
            "x", "{}", "  null", "

]",
        ] {
            let text = format!("{lock}{trailing}");
            assert!(
                Reader::new(&text).document().is_err(),
                "should have refused trailing {trailing:?}"
            );
        }
        // Trailing whitespace alone is not trailing content.
        assert!(Reader::new(&format!(
            "{lock}

"
        ))
        .document()
        .is_ok());
    }

    /// The reader fails closed. Every one of these is exit 2 at the caller,
    /// never a pass and never a reported drift.
    #[test]
    fn the_reader_refuses_what_it_does_not_understand() {
        for bad in [
            "{",
            "{\"skills\":}",
            "{\"skills\": {\"a\": {\"skillPath\": }}}",
            "{\"skills\": {\"a\": }}",
            "\"unterminated",
            "{\"a\": \"\\q\"}",
        ] {
            assert!(
                Reader::new(bad).value().is_err(),
                "should have refused: {bad}"
            );
        }
    }

    #[test]
    fn a_lock_without_a_skills_object_is_not_readable() {
        let dir = std::env::temp_dir().join("fragcap-xtask-skills-test");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join(LOCK_FILE), "{\"version\": 1}").expect("write");
        let err = read_lock(&dir).expect_err("must refuse");
        assert!(err.to_string().contains("skills"), "{err}");
        let _ = fs::remove_file(dir.join(LOCK_FILE));
    }
}
