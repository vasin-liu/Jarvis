use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{Result, StoreError};
use crate::schema::init_schema;
use crate::types::{ChunkHit, IndexStatus, NewChunk, Source, SourceKind};
use crate::vecext::register_sqlite_vec;

pub struct Store {
    conn: Mutex<Connection>,
    dim: usize,
}

impl Store {
    pub fn open(path: impl AsRef<Path>, dim: usize) -> Result<Self> {
        register_sqlite_vec();
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        init_schema(&conn, dim)?;
        Ok(Self {
            conn: Mutex::new(conn),
            dim,
        })
    }

    pub fn open_in_memory(dim: usize) -> Result<Self> {
        register_sqlite_vec();
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        init_schema(&conn, dim)?;
        Ok(Self {
            conn: Mutex::new(conn),
            dim,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn upsert_source(&self, s: &Source) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sources(id, kind, uri, title, content_hash, indexed_at, status, error)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
               kind=excluded.kind, uri=excluded.uri, title=excluded.title,
               content_hash=excluded.content_hash, indexed_at=excluded.indexed_at,
               status=excluded.status, error=excluded.error",
            rusqlite::params![
                s.id,
                s.kind.as_str(),
                s.uri,
                s.title,
                s.content_hash,
                s.indexed_at,
                s.status.as_str(),
                s.error,
            ],
        )?;
        Ok(())
    }

    pub fn get_source(&self, id: &str) -> Result<Source> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, kind, uri, title, content_hash, indexed_at, status, error
             FROM sources WHERE id = ?1",
            [id],
            Self::row_to_source,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_string()),
            other => StoreError::Sqlite(other),
        })
    }

    pub fn list_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, kind, uri, title, content_hash, indexed_at, status, error
             FROM sources ORDER BY id",
        )?;
        let rows = stmt.query_map([], Self::row_to_source)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete_source(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sources WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Write chunks into chunks, vec_chunks, and chunks_fts tables.
    pub fn insert_chunks(&self, source_id: &str, chunks: &[NewChunk]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        for c in chunks {
            if c.embedding.len() != self.dim {
                return Err(StoreError::DimMismatch {
                    expected: self.dim,
                    got: c.embedding.len(),
                });
            }
            tx.execute(
                "INSERT INTO chunks(source_id, ord, text, loc, token_count)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![source_id, c.ord, c.text, c.loc, c.token_count],
            )?;
            let chunk_id = tx.last_insert_rowid();

            let vec_json = serde_json::to_string(&c.embedding)?;
            tx.execute(
                "INSERT INTO vec_chunks(rowid, embedding) VALUES(?1, ?2)",
                rusqlite::params![chunk_id, vec_json],
            )?;
            tx.execute(
                "INSERT INTO chunks_fts(rowid, text) VALUES(?1, ?2)",
                rusqlite::params![chunk_id, c.text],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Delete all chunks for a source (vector + FTS rows included).
    pub fn delete_chunks_for_source(&self, source_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        let ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT id FROM chunks WHERE source_id = ?1")?;
            let rows = stmt.query_map([source_id], |r| r.get::<_, i64>(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        for id in &ids {
            tx.execute("DELETE FROM vec_chunks WHERE rowid = ?1", [id])?;
            tx.execute("DELETE FROM chunks_fts WHERE rowid = ?1", [id])?;
        }
        tx.execute("DELETE FROM chunks WHERE source_id = ?1", [source_id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn count_chunks(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))?)
    }

    /// KNN vector search (lower score = closer).
    pub fn search_vector(&self, query: &[f32], k: usize) -> Result<Vec<ChunkHit>> {
        if query.len() != self.dim {
            return Err(StoreError::DimMismatch {
                expected: self.dim,
                got: query.len(),
            });
        }
        let conn = self.conn.lock().unwrap();
        let q_json = serde_json::to_string(query)?;
        let mut stmt = conn.prepare(
            "SELECT c.id, c.source_id, c.text, c.loc, v.distance
             FROM vec_chunks v
             JOIN chunks c ON c.id = v.rowid
             WHERE v.embedding MATCH ?1 AND k = ?2
             ORDER BY v.distance",
        )?;
        let rows = stmt.query_map(rusqlite::params![q_json, k as i64], |r| {
            Ok(ChunkHit {
                chunk_id: r.get(0)?,
                source_id: r.get(1)?,
                text: r.get(2)?,
                loc: r.get(3)?,
                score: r.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// FTS5 BM25 search (lower score = more relevant).
    pub fn search_fts(&self, query: &str, k: usize) -> Result<Vec<ChunkHit>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT c.id, c.source_id, c.text, c.loc, bm25(chunks_fts) AS score
             FROM chunks_fts
             JOIN chunks c ON c.id = chunks_fts.rowid
             WHERE chunks_fts MATCH ?1
             ORDER BY score
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![query, k as i64], |r| {
            Ok(ChunkHit {
                chunk_id: r.get(0)?,
                source_id: r.get(1)?,
                text: r.get(2)?,
                loc: r.get(3)?,
                score: r.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO meta(key, value) VALUES(?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let r = conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
            row.get::<_, String>(0)
        });
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StoreError::Sqlite(e)),
        }
    }

    fn row_to_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
        let kind_s: String = row.get(1)?;
        let status_s: String = row.get(6)?;
        Ok(Source {
            id: row.get(0)?,
            kind: SourceKind::parse(&kind_s).unwrap_or(SourceKind::LocalFile),
            uri: row.get(2)?,
            title: row.get(3)?,
            content_hash: row.get(4)?,
            indexed_at: row.get(5)?,
            status: IndexStatus::parse(&status_s).unwrap_or(IndexStatus::Pending),
            error: row.get(7)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NewChunk;

    fn sample_source(id: &str) -> Source {
        Source {
            id: id.to_string(),
            kind: SourceKind::LocalFile,
            uri: format!("/tmp/{id}.md"),
            title: format!("title {id}"),
            content_hash: "h0".to_string(),
            indexed_at: None,
            status: IndexStatus::Pending,
            error: None,
        }
    }

    fn chunk(ord: i64, text: &str, emb: [f32; 4]) -> NewChunk {
        NewChunk {
            ord,
            text: text.to_string(),
            loc: format!("L{ord}"),
            token_count: text.split_whitespace().count() as i64,
            embedding: emb.to_vec(),
        }
    }

    #[test]
    fn upsert_get_list_delete_source() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store.upsert_source(&sample_source("b")).unwrap();

        let got = store.get_source("a").unwrap();
        assert_eq!(got.title, "title a");
        assert_eq!(store.list_sources().unwrap().len(), 2);

        store.delete_source("a").unwrap();
        assert_eq!(store.list_sources().unwrap().len(), 1);
        assert!(matches!(
            store.get_source("a"),
            Err(StoreError::NotFound(_))
        ));
    }

    #[test]
    fn upsert_updates_existing() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        let mut s = sample_source("a");
        s.title = "updated".to_string();
        s.status = IndexStatus::Indexed;
        store.upsert_source(&s).unwrap();

        let got = store.get_source("a").unwrap();
        assert_eq!(got.title, "updated");
        assert_eq!(got.status, IndexStatus::Indexed);
        assert_eq!(store.list_sources().unwrap().len(), 1);
    }

    #[test]
    fn insert_and_count_chunks() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks(
                "a",
                &[
                    chunk(0, "hello world", [1.0, 0.0, 0.0, 0.0]),
                    chunk(1, "rust sqlite", [0.0, 1.0, 0.0, 0.0]),
                ],
            )
            .unwrap();
        assert_eq!(store.count_chunks().unwrap(), 2);
    }

    #[test]
    fn insert_rejects_wrong_dim() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        let bad = NewChunk {
            ord: 0,
            text: "x".into(),
            loc: "L0".into(),
            token_count: 1,
            embedding: vec![1.0, 2.0],
        };
        assert!(matches!(
            store.insert_chunks("a", &[bad]),
            Err(StoreError::DimMismatch {
                expected: 4,
                got: 2
            })
        ));
    }

    #[test]
    fn delete_chunks_for_source_clears_all_tables() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks("a", &[chunk(0, "hello world", [1.0, 0.0, 0.0, 0.0])])
            .unwrap();
        store.delete_chunks_for_source("a").unwrap();
        assert_eq!(store.count_chunks().unwrap(), 0);
    }

    #[test]
    fn search_vector_returns_nearest_first() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks(
                "a",
                &[
                    chunk(0, "x axis", [1.0, 0.0, 0.0, 0.0]),
                    chunk(1, "y axis", [0.0, 1.0, 0.0, 0.0]),
                    chunk(2, "z axis", [0.0, 0.0, 1.0, 0.0]),
                ],
            )
            .unwrap();

        let hits = store.search_vector(&[0.9, 0.1, 0.0, 0.0], 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].text, "x axis");
        assert!(hits[0].score <= hits[1].score);
    }

    #[test]
    fn search_vector_rejects_wrong_dim() {
        let store = Store::open_in_memory(4).unwrap();
        assert!(matches!(
            store.search_vector(&[1.0, 2.0], 5),
            Err(StoreError::DimMismatch { .. })
        ));
    }

    #[test]
    fn search_fts_matches_keyword() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks(
                "a",
                &[
                    chunk(0, "the quick brown fox", [1.0, 0.0, 0.0, 0.0]),
                    chunk(1, "lazy dog sleeps", [0.0, 1.0, 0.0, 0.0]),
                    chunk(2, "fox and hound", [0.0, 0.0, 1.0, 0.0]),
                ],
            )
            .unwrap();

        let hits = store.search_fts("fox", 10).unwrap();
        assert_eq!(hits.len(), 2);
        for h in &hits {
            assert!(h.text.contains("fox"));
        }
    }

    #[test]
    fn search_fts_no_match_returns_empty() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks("a", &[chunk(0, "hello world", [1.0, 0.0, 0.0, 0.0])])
            .unwrap();
        let hits = store.search_fts("nonexistentterm", 10).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn meta_set_and_get() {
        let store = Store::open_in_memory(4).unwrap();
        assert_eq!(store.get_meta("embedder_id").unwrap(), None);
        store.set_meta("embedder_id", "fastembed:bge-small").unwrap();
        assert_eq!(
            store.get_meta("embedder_id").unwrap(),
            Some("fastembed:bge-small".to_string())
        );
        store.set_meta("embedder_id", "ollama:nomic").unwrap();
        assert_eq!(
            store.get_meta("embedder_id").unwrap(),
            Some("ollama:nomic".to_string())
        );
    }
}
