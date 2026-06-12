use std::fs;
use std::path::Path;

use crate::document::Document;
use crate::error::{IngestError, Result};
use crate::hash::hash_text;

pub fn load_path(path: impl AsRef<Path>) -> Result<Document> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    let text = match ext.as_str() {
        "txt" | "md" | "markdown" => fs::read_to_string(path)?,
        other => return Err(IngestError::UnsupportedType(other.to_string())),
    };

    if text.trim().is_empty() {
        return Err(IngestError::Empty(path.display().to_string()));
    }

    let uri = canonical_uri(path);
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
    fn rejects_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        fs::write(&path, "   ").unwrap();
        assert!(matches!(load_path(&path), Err(IngestError::Empty(_))));
    }
}
