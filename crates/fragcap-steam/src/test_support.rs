// SPDX-License-Identifier: Apache-2.0

//! A minimal temporary-directory fixture for tests, with no third-party
//! dependency. Each tree is a uniquely named directory under the system temp
//! directory, removed on drop.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A throwaway directory tree for a test.
pub struct TempTree {
    root: PathBuf,
}

impl TempTree {
    /// Create a fresh, empty temporary tree.
    pub fn new() -> TempTree {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("fragcap-steam-test-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&root).expect("create temp tree");
        TempTree { root }
    }

    /// The tree root.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Write a file (creating parent directories) with the given text.
    pub fn write(&self, path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, contents).expect("write file");
    }

    /// Write a placeholder executable of `size` bytes (creating parents).
    pub fn write_exe(&self, path: &Path, size: usize) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, vec![0u8; size]).expect("write exe");
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
