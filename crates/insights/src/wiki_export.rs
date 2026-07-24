//! Obsidian-compatible zip export for the on-disk wiki tree (WIKI-07).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::{InsightsError, Result};

const OBSIDIAN_STUB_BODY: &[u8] = br#"{"legacyEditor":false}"#;
const OBSIDIAN_STUB_ENTRY: &str = ".obsidian/app.json";

/// True iff any `.md` exists under `sources/`, `entities/`, or `concepts/` (D-10).
/// Root-only `index.md` or missing section dirs → false.
pub fn wiki_has_exportable_notes(wiki_root: &Path) -> bool {
    for dir in ["sources", "entities", "concepts"] {
        let d = wiki_root.join(dir);
        if !d.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&d) {
            for e in entries.flatten() {
                if e.path().extension().is_some_and(|x| x == "md") {
                    return true;
                }
            }
        }
    }
    false
}

/// Preflight for Obsidian zip export: enabled gate then notes scan (D-06/D-07).
pub fn wiki_export_preflight(wiki_root: &Path, wiki_enabled: bool) -> Result<bool> {
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
    }
    Ok(wiki_has_exportable_notes(wiki_root))
}

/// Pack `wiki_root` into `dest_zip` with inject-only `.obsidian` stub.
///
/// Hard-rejects when `wiki_enabled` is false or the tree has no exportable notes.
pub fn export_wiki_zip(wiki_root: &Path, dest_zip: &Path, wiki_enabled: bool) -> Result<()> {
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
    }
    if !wiki_has_exportable_notes(wiki_root) {
        return Err(InsightsError::WikiEmpty);
    }

    let temp = dest_temp_path(dest_zip);
    match write_zip_to_temp(wiki_root, &temp) {
        Ok(()) => {
            if dest_zip.exists() {
                fs::remove_file(dest_zip)?;
            }
            fs::rename(&temp, dest_zip)?;
            Ok(())
        }
        Err(e) => {
            let _ = fs::remove_file(&temp);
            Err(e)
        }
    }
}

fn dest_temp_path(dest_zip: &Path) -> PathBuf {
    match dest_zip.file_name().and_then(|n| n.to_str()) {
        Some(name) => dest_zip.with_file_name(format!("{name}.tmp")),
        None => dest_zip.with_extension("zip.tmp"),
    }
}

fn write_zip_to_temp(wiki_root: &Path, temp: &Path) -> Result<()> {
    let file = File::create(temp)?;
    let mut zip = ZipWriter::new(file);
    let options =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (name, path) in collect_packable_files(wiki_root)? {
        zip.start_file(&name, options)?;
        zip.write_all(&fs::read(&path)?)?;
    }

    zip.start_file(OBSIDIAN_STUB_ENTRY, options)?;
    zip.write_all(OBSIDIAN_STUB_BODY)?;
    zip.finish()?;
    Ok(())
}

fn is_under_ondisk_obsidian(wiki_root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(wiki_root) else {
        return false;
    };
    matches!(
        rel.components().next(),
        Some(Component::Normal(c)) if c == ".obsidian"
    )
}

fn relative_zip_entry_name(wiki_root: &Path, abs: &Path) -> Result<String> {
    let rel = abs.strip_prefix(wiki_root).map_err(|_| {
        InsightsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path outside wiki_root",
        ))
    })?;
    if rel.as_os_str().is_empty() {
        return Err(InsightsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty relative path",
        )));
    }
    if rel.is_absolute()
        || rel.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(InsightsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unsafe zip entry path: {}", rel.display()),
        )));
    }
    Ok(rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn collect_packable_files(wiki_root: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut out = Vec::new();
    if !wiki_root.is_dir() {
        return Ok(out);
    }
    let mut stack = vec![wiki_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            let path = entry.path();
            // Do not follow symlinks out of wiki_root (T-12-02).
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                if is_under_ondisk_obsidian(wiki_root, &path) {
                    continue;
                }
                stack.push(path);
            } else if ft.is_file() {
                if is_under_ondisk_obsidian(wiki_root, &path) {
                    continue;
                }
                let name = relative_zip_entry_name(wiki_root, &path)?;
                out.push((name, path));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::path::Component;

    use zip::ZipArchive;

    fn write_md(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn archive_names(zip_path: &Path) -> Vec<String> {
        let f = fs::File::open(zip_path).unwrap();
        let mut archive = ZipArchive::new(f).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }

    fn assert_path_safe_names(names: &[String]) {
        for n in names {
            assert!(
                !n.contains(".."),
                "entry must not contain ParentDir segments: {n}"
            );
            assert!(
                !n.starts_with('/') && !n.starts_with('\\'),
                "entry must not be absolute: {n}"
            );
            assert!(
                !n.contains('\\'),
                "entry must use / separators only: {n}"
            );
            let p = Path::new(n);
            assert!(
                !p.is_absolute(),
                "entry path must not be absolute: {n}"
            );
            assert!(
                p.components().all(|c| matches!(c, Component::Normal(_))),
                "entry must have only Normal components: {n}"
            );
            // Drive / prefix style (e.g. C:)
            assert!(
                !n.chars().nth(1).is_some_and(|c| c == ':'),
                "entry must not be drive-prefixed: {n}"
            );
        }
    }

    #[test]
    fn export_wiki_zip_includes_stub_and_pages() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("sources/foo.md"), "# Foo\n");
        write_md(&wiki_root.join("index.md"), "# Index\n");

        let dest = dir.path().join("out.zip");
        export_wiki_zip(&wiki_root, &dest, true).expect("export should succeed");

        assert!(dest.is_file(), "dest zip must exist");
        assert!(
            !wiki_root.join(".obsidian").exists(),
            "stub must not be written on disk (D-14)"
        );

        let names = archive_names(&dest);
        assert!(
            names.iter().any(|n| n == "sources/foo.md" || n.ends_with("sources/foo.md")),
            "zip must include sources page, got {names:?}"
        );
        let obsidian: Vec<_> = names
            .iter()
            .filter(|n| n.starts_with(".obsidian"))
            .collect();
        assert_eq!(
            obsidian,
            vec![".obsidian/app.json"],
            "exactly one .obsidian entry expected, got {obsidian:?}"
        );

        let f = fs::File::open(&dest).unwrap();
        let mut archive = ZipArchive::new(f).unwrap();
        let mut stub = archive.by_name(OBSIDIAN_STUB_ENTRY).unwrap();
        let mut body = String::new();
        stub.read_to_string(&mut body).unwrap();
        assert_eq!(body, r#"{"legacyEditor":false}"#);

        assert_path_safe_names(&names);
    }

    #[test]
    fn export_wiki_zip_rejects_empty_wiki() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("index.md"), "# Index only\n");

        let dest = dir.path().join("empty.zip");
        let err = export_wiki_zip(&wiki_root, &dest, true).unwrap_err();
        assert!(
            matches!(err, InsightsError::WikiEmpty),
            "expected WikiEmpty, got {err:?}"
        );
        assert!(
            !dest.exists(),
            "dest must not be written for empty wiki"
        );
    }

    #[test]
    fn export_wiki_rejects_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("sources/foo.md"), "# Foo\n");

        let dest = dir.path().join("disabled.zip");
        let err = export_wiki_zip(&wiki_root, &dest, false).unwrap_err();
        assert!(
            matches!(err, InsightsError::WikiDisabled),
            "expected WikiDisabled, got {err:?}"
        );
        assert!(!dest.exists(), "dest must not be written when disabled");
    }

    #[test]
    fn wiki_export_preflight_rejects_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("sources/foo.md"), "# Foo\n");

        let err = wiki_export_preflight(&wiki_root, false).unwrap_err();
        assert!(
            matches!(err, InsightsError::WikiDisabled),
            "expected WikiDisabled, got {err:?}"
        );
        assert_eq!(
            err.to_string(),
            InsightsError::WikiDisabled.to_string(),
            "Display must match export WikiDisabled IPC string"
        );
    }

    #[test]
    fn wiki_export_preflight_enabled_reports_notes() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("index.md"), "# Index only\n");
        assert_eq!(wiki_export_preflight(&wiki_root, true).unwrap(), false);

        write_md(&wiki_root.join("sources/foo.md"), "# Foo\n");
        assert_eq!(wiki_export_preflight(&wiki_root, true).unwrap(), true);
    }

    #[test]
    fn export_wiki_skips_ondisk_obsidian_injects_stub() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("sources/foo.md"), "# Foo\n");
        let ondisk = wiki_root.join(".obsidian/app.json");
        write_md(&ondisk, r#"{"legacyEditor":true,"userTheme":"dark"}"#);

        let dest = dir.path().join("skip-obsidian.zip");
        export_wiki_zip(&wiki_root, &dest, true).expect("export should succeed");

        // On-disk .obsidian still present (we did not delete it), but zip uses stub.
        assert!(ondisk.is_file());

        let f = fs::File::open(&dest).unwrap();
        let mut archive = ZipArchive::new(f).unwrap();
        let names: Vec<_> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        let obsidian: Vec<_> = names
            .iter()
            .filter(|n| n.starts_with(".obsidian"))
            .cloned()
            .collect();
        assert_eq!(obsidian, vec![".obsidian/app.json".to_string()]);

        let mut stub = archive.by_name(OBSIDIAN_STUB_ENTRY).unwrap();
        let mut body = String::new();
        stub.read_to_string(&mut body).unwrap();
        assert_eq!(body, r#"{"legacyEditor":false}"#);
        assert!(!body.contains("userTheme"));
    }

    #[test]
    fn export_wiki_has_exportable_notes_matches_d10() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");

        assert!(
            !wiki_has_exportable_notes(&wiki_root),
            "missing dirs → false"
        );

        write_md(&wiki_root.join("index.md"), "# Index\n");
        assert!(
            !wiki_has_exportable_notes(&wiki_root),
            "index-only → false"
        );

        write_md(&wiki_root.join("sources/a.md"), "# A\n");
        assert!(
            wiki_has_exportable_notes(&wiki_root),
            "sources .md → true"
        );

        let dir2 = tempfile::tempdir().unwrap();
        let root2 = dir2.path().join("wiki");
        write_md(&root2.join("entities/e.md"), "# E\n");
        assert!(wiki_has_exportable_notes(&root2));

        let dir3 = tempfile::tempdir().unwrap();
        let root3 = dir3.path().join("wiki");
        write_md(&root3.join("concepts/c.md"), "# C\n");
        assert!(wiki_has_exportable_notes(&root3));
    }

    #[test]
    fn export_wiki_zip_path_safe_entry_names() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("sources/nested/page.md"), "# Nested\n");
        write_md(&wiki_root.join("entities/e.md"), "# E\n");

        let dest = dir.path().join("safe.zip");
        export_wiki_zip(&wiki_root, &dest, true).expect("export should succeed");

        let names = archive_names(&dest);
        assert_path_safe_names(&names);
        assert!(names.iter().any(|n| n.contains("sources/") && n.ends_with(".md")));
    }
}
