//! Unlike `config` and `agent`, this function is NOT a stub: it's pure
//! logic (no network call, no file actually being read), so it makes sense
//! to write it for real right away. It's also a good first example of
//! `Path`/`PathBuf`, which in Rust replace the "bare" strings you'd use in
//! Python/JS for file paths.

use std::path::{Component, Path, PathBuf};

/// Tries to resolve `input` (relative to the sandbox, absolute, or with
/// `~`) while staying inside `root`, without touching disk — useful even
/// for a path that doesn't exist yet. Returns `None` if the result would
/// fall outside the sandbox.
///
/// `Option<PathBuf>` is Rust's equivalent of a value that "might be
/// missing": there's no implicit `null`/`None` like in C#/Java, the type
/// itself declares that the result might not be there, and the compiler
/// forces you to handle both cases before using the value.
pub fn resolve_within(root: &Path, input: &str) -> Option<PathBuf> {
    if input.starts_with('~') {
        // Anything starting with ~ points at the home folder: outside the sandbox by definition.
        return None;
    }

    let raw = if Path::new(input).is_absolute() {
        PathBuf::from(input)
    } else {
        root.join(input)
    };

    // Normalize ".." and "." by hand, component by component, so the file
    // doesn't need to exist yet (canonicalize() would require that instead).
    let mut normalized = PathBuf::new();
    for component in raw.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    if normalized.starts_with(root) {
        Some(normalized)
    } else {
        None
    }
}

/// Looks through the free text typed by the user for the first "token"
/// that looks like a path (contains '/' or starts with '~'), to hand off
/// to `resolve_within`. Returns `None` if it doesn't find one.
pub fn find_path_like_token(input: &str) -> Option<&str> {
    input
        .split_whitespace()
        .find(|token| token.contains('/') || token.starts_with('~'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_escapes_with_double_dot() {
        let root = Path::new("/home/user/project");
        assert!(resolve_within(root, "../secret").is_none());
    }

    #[test]
    fn accepts_internal_paths() {
        let root = Path::new("/home/user/project");
        assert_eq!(
            resolve_within(root, "src/main.rs"),
            Some(PathBuf::from("/home/user/project/src/main.rs"))
        );
    }

    #[test]
    fn blocks_home() {
        let root = Path::new("/home/user/project");
        assert!(resolve_within(root, "~/other").is_none());
    }
}