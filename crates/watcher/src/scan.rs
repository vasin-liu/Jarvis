use std::path::{Path, PathBuf};

use crate::error::{Result, WatchError};

const INDEXABLE: [&str; 3] = ["txt", "md", "markdown"];

pub fn is_indexable(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| INDEXABLE.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn scan_folder(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Err(WatchError::NotFound(root.display().to_string()));
    }
    let mut out = Vec::new();
    walk(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else if is_indexable(&path) {
            out.push(path);
        }
    }
    Ok(())
}

pub fn uri_for_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

pub fn unindex_path(store: &store::Store, path: &Path) -> store::Result<()> {
    let id = uri_for_path(path);
    store.delete_chunks_for_source(&id)?;
    store.delete_source(&id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collects_indexable_files_recursively() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("a.txt"), "a").unwrap();
        fs::write(dir.path().join("nested/b.md"), "b").unwrap();
        fs::write(dir.path().join("skip.pdf"), "%PDF").unwrap();

        let files = scan_folder(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }
}
