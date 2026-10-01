//! Filesystem browsing screen, used in two different places:
//! - `base` with no arguments shows it in `Directory` mode, to pick the
//!   sandbox (see `main.rs`).
//! - `Ctrl+A` inside the sandbox shows it in `Attach` mode, to pick an
//!   external file *or folder* to "attach" (see `tui::mod::run`).
//!
//! Like `sandbox::fs`, this is real code, not a stub: it really reads the
//! filesystem with `std::fs::read_dir`. What's still fake, and deliberately
//! kept out of this file, is what happens *after* something is chosen
//! (copying it read-only into `.base/attachments/`): this stops at "which
//! path did the user choose".

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    /// Only folders are listed; → opens one, Enter uses the one being browsed.
    Directory,
    /// Folders and files are listed; → opens a folder, Enter picks
    /// whatever is highlighted — a file, or a folder, without opening it.
    Attach,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct PickerState {
    pub kind: PickerKind,
    /// The folder whose contents we're currently showing.
    pub current: PathBuf,
    /// What it contains (hidden entries excluded), folders first, then
    /// alphabetical order.
    pub entries: Vec<Entry>,
    /// Index into `entries` currently highlighted.
    pub selected: usize,
    /// Error message from the last `refresh`, if `read_dir` failed (e.g.
    /// permission denied) — shown instead of the list.
    pub message: Option<String>,
}

impl PickerState {
    pub fn new(start: PathBuf, kind: PickerKind) -> Self {
        let mut state = Self {
            kind,
            current: start,
            entries: Vec::new(),
            selected: 0,
            message: None,
        };
        state.refresh();
        state
    }

    fn refresh(&mut self) {
        self.selected = 0;
        self.message = None;

        match std::fs::read_dir(&self.current) {
            Ok(read) => {
                let mut items: Vec<Entry> = read
                    .filter_map(|entry| entry.ok()) // drop entries that error out, one at a time
                    .map(|entry| {
                        let path = entry.path();
                        let is_dir = path.is_dir();
                        Entry { path, is_dir }
                    })
                    .filter(|e| {
                        // no hidden entries in the list, to keep it tidy
                        let hidden = e
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().starts_with('.'))
                            .unwrap_or(false);
                        if hidden {
                            return false;
                        }
                        // in Directory mode, files aren't relevant at all
                        match self.kind {
                            PickerKind::Directory => e.is_dir,
                            PickerKind::Attach => true,
                        }
                    })
                    .collect();
                // folders first, then alphabetical: easier to browse
                items.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.path.cmp(&b.path)));
                self.entries = items;
            }
            Err(e) => {
                self.entries = Vec::new();
                self.message = Some(format!("Could not read {}: {e}", self.current.display()));
            }
        }
    }

    /// Moves the selection by `delta` positions, with wraparound: from
    /// above the first it goes to the last, from below the last it goes
    /// back to the first.
    ///
    /// `rem_euclid` instead of plain `%`: in Rust (as in C/C#/JS) the
    /// remainder of a negative number can be negative (`-1 % 5 == -1`),
    /// which here would give an invalid index. `rem_euclid` always returns
    /// a non-negative result (`(-1i32).rem_euclid(5) == 4`).
    pub fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let len = self.entries.len() as isize;
        let idx = (self.selected as isize + delta).rem_euclid(len);
        self.selected = idx as usize;
    }

    /// If the highlighted entry is a folder, goes into it (used by →). If
    /// it's a file, does nothing — opening a file makes no sense, and
    /// *picking* one isn't "navigating" either: that's `highlighted_path`
    /// and `events::handle_picker` below, triggered by Enter instead.
    pub fn enter_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                self.current = entry.path.clone();
                self.refresh();
            }
        }
    }

    /// The path of the highlighted entry, whatever it is — file or folder.
    /// Used by Enter in `Attach` mode: unlike `enter_selected` (→, folders
    /// only), picking something here never looks inside it.
    pub fn highlighted_path(&self) -> Option<PathBuf> {
        self.entries.get(self.selected).map(|e| e.path.clone())
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current.parent() {
            self.current = parent.to_path_buf();
            self.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Creates a temp folder with two subfolders ("a", "b"), one file
    /// ("c.txt") and one hidden subfolder (".hidden"), shared by several
    /// tests: `label` makes it unique per test, since `cargo test` runs
    /// them in parallel by default.
    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!("base-picker-test-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("b")).unwrap();
        fs::create_dir_all(base.join("a")).unwrap();
        fs::create_dir_all(base.join(".hidden")).unwrap();
        fs::write(base.join("c.txt"), b"fake content").unwrap();
        base
    }

    #[test]
    fn directory_mode_lists_folders_only() {
        let root = tempdir("dir-mode");
        let state = PickerState::new(root.clone(), PickerKind::Directory);
        let path: Vec<PathBuf> = state.entries.iter().map(|e| e.path.clone()).collect();
        assert_eq!(path, vec![root.join("a"), root.join("b")]);
    }

    #[test]
    fn attach_mode_lists_folders_and_files_folders_first() {
        let root = tempdir("attach-mode");
        let state = PickerState::new(root.clone(), PickerKind::Attach);
        let path: Vec<PathBuf> = state.entries.iter().map(|e| e.path.clone()).collect();
        assert_eq!(path, vec![root.join("a"), root.join("b"), root.join("c.txt")]);
        assert!(state.entries[0].is_dir);
        assert!(!state.entries[2].is_dir);
    }

    #[test]
    fn enter_then_go_up_navigates() {
        let root = tempdir("navigate");
        let mut state = PickerState::new(root.clone(), PickerKind::Directory);
        state.enter_selected(); // "a" is first alphabetically
        assert_eq!(state.current, root.join("a"));
        state.go_up();
        assert_eq!(state.current, root);
    }

    #[test]
    fn entering_a_file_does_not_navigate() {
        let root = tempdir("file-no-nav");
        let mut state = PickerState::new(root.clone(), PickerKind::Attach);
        state.selected = 2; // "c.txt", the only file
        state.enter_selected();
        assert_eq!(state.current, root); // unchanged: a file doesn't "open"
    }

    #[test]
    fn highlighted_path_works_for_both_files_and_folders() {
        let root = tempdir("highlight-any");
        let mut state = PickerState::new(root.clone(), PickerKind::Attach);
        state.selected = 0; // "a", a folder
        assert_eq!(state.highlighted_path(), Some(root.join("a")));
        state.selected = 2; // "c.txt", a file
        assert_eq!(state.highlighted_path(), Some(root.join("c.txt")));
    }

    #[test]
    fn selection_wraps_around() {
        let root = tempdir("wrap");
        let mut state = PickerState::new(root, PickerKind::Directory); // 2 subfolders: a, b
        assert_eq!(state.selected, 0);
        state.move_selection(-1);
        assert_eq!(state.selected, 1); // from above the first, wraps to the last
        state.move_selection(1);
        assert_eq!(state.selected, 0); // and from below the last, wraps to the first
    }
}