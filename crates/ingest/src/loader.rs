use std::fs;
use std::path::{Path, PathBuf};

use calamine::{open_workbook_auto, Data, Reader};

use crate::document::Document;
use crate::error::{IngestError, Result};
use crate::hash::hash_text;

/// Resolve a user-supplied path to an existing file.
/// Relative paths are tried against cwd and parent dirs (Tauri dev cwd is often `src-tauri`).
pub fn resolve_existing_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();

    if path.is_file() {
        return Ok(path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf()));
    }

    if path.is_absolute() {
        return Err(IngestError::NotFound(path.display().to_string()));
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut base = cwd.as_path();
        for _ in 0..4 {
            let candidate = base.join(path);
            if candidate.is_file() {
                return Ok(candidate
                    .canonicalize()
                    .unwrap_or(candidate));
            }
            base = match base.parent() {
                Some(p) => p,
                None => break,
            };
        }
    }

    Err(IngestError::NotFound(path.display().to_string()))
}

pub fn load_path(path: impl AsRef<Path>) -> Result<Document> {
    let path = resolve_existing_path(path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    let text = match ext.as_str() {
        "txt" | "md" | "markdown" => fs::read_to_string(&path)?,
        "csv" => fs::read_to_string(&path)?,
        "xls" | "xlsx" | "xlsm" | "ods" => load_spreadsheet_text(&path)?,
        other => return Err(IngestError::UnsupportedType(other.to_string())),
    };

    if text.trim().is_empty() {
        return Err(IngestError::Empty(path.display().to_string()));
    }

    let uri = canonical_uri(&path);
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    Ok(Document {
        uri,
        title,
        content_hash: hash_text(&text),
        text,
    })
}

fn canonical_uri(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn load_spreadsheet_text(path: &Path) -> Result<String> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| IngestError::Spreadsheet(e.to_string()))?;
    let mut out = String::new();
    for sheet_name in workbook.sheet_names().to_vec() {
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| IngestError::Spreadsheet(e.to_string()))?;
        out.push_str(&format!("# {sheet_name}\n"));
        for row in range.rows() {
            let cells: Vec<String> = row.iter().map(cell_to_string).collect();
            if cells.iter().any(|c| !c.is_empty()) {
                out.push_str(&cells.join("\t"));
                out.push('\n');
            }
        }
        out.push('\n');
    }
    if out.trim().is_empty() {
        return Err(IngestError::Empty(path.display().to_string()));
    }
    Ok(out)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(f) => f.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{e:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_txt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.txt");
        {
            let mut f = fs::File::create(&path).unwrap();
            writeln!(f, "line one").unwrap();
            writeln!(f, "line two").unwrap();
        }

        let doc = load_path(&path).unwrap();
        assert!(doc.text.contains("line one"));
        assert_eq!(doc.title, "note");
        assert_eq!(doc.content_hash.len(), 64);
        assert!(doc.uri.ends_with("note.txt"));
    }

    #[test]
    fn loads_md_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("readme.md");
        fs::write(&path, "# Title\n\nBody text.").unwrap();

        let doc = load_path(&path).unwrap();
        assert!(doc.text.contains("Body text."));
    }

    #[test]
    fn rejects_unsupported_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.pdf");
        fs::write(&path, "%PDF").unwrap();
        assert!(matches!(
            load_path(&path),
            Err(IngestError::UnsupportedType(_))
        ));
    }

    #[test]
    fn rejects_missing_file() {
        assert!(matches!(
            load_path("/nonexistent/jarvis-test-file.md"),
            Err(IngestError::NotFound(_))
        ));
    }

    #[test]
    fn loads_xls_file_when_present() {
        let path = std::path::Path::new(".lark-cache/heartbeat.xls");
        if !path.is_file() {
            return;
        }
        let doc = load_path(path).unwrap();
        assert!(!doc.text.trim().is_empty());
        assert!(doc.title.contains("heartbeat") || doc.title.contains("xls"));
    }
}
