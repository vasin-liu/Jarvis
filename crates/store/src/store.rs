use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{Result, StoreError};
use crate::schema::init_schema;
use crate::types::{
    ChatMessage, ChatRole, ChatSession, ChunkHit, IndexStatus, NewChunk, Source, SourceKind,
    Task, TaskStatus,
};
use crate::vecext::register_sqlite_vec;

pub struct Store {
    conn: Mutex<Connection>,
    dim: Mutex<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexHealth {
    pub store_dim: usize,
    pub stored_embedder_id: Option<String>,
    pub source_count: usize,
    pub chunk_count: usize,
}

impl Store {
    pub fn open(path: impl AsRef<Path>, dim: usize) -> Result<Self> {
        register_sqlite_vec();
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode=WAL;")?;
        init_schema(&conn, dim)?;
        Ok(Self {
            conn: Mutex::new(conn),
            dim: Mutex::new(dim),
        })
    }

    pub fn open_in_memory(dim: usize) -> Result<Self> {
        register_sqlite_vec();
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode=WAL;")?;
        init_schema(&conn, dim)?;
        Ok(Self {
            conn: Mutex::new(conn),
            dim: Mutex::new(dim),
        })
    }

    pub fn dim(&self) -> usize {
        *self.dim.lock().unwrap()
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
            "SELECT id, kind, uri, title, content_hash, indexed_at, status, error, summary
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
            "SELECT id, kind, uri, title, content_hash, indexed_at, status, error, summary
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

    /// Rekey a source row and update chunk/task foreign keys (used for memory URI migration).
    pub fn rename_source_id(&self, old_id: &str, new_id: &str) -> Result<()> {
        if old_id == new_id {
            return Ok(());
        }
        let old = self.get_source(old_id)?;
        if self.get_source(new_id).is_ok() {
            return Err(StoreError::NotFound(format!(
                "target id already exists: {new_id}"
            )));
        }

        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO sources(id, kind, uri, title, content_hash, indexed_at, status, error, summary)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                new_id,
                old.kind.as_str(),
                new_id,
                old.title,
                old.content_hash,
                old.indexed_at,
                old.status.as_str(),
                old.error,
                old.summary,
            ],
        )?;
        tx.execute(
            "UPDATE chunks SET source_id = ?1 WHERE source_id = ?2",
            rusqlite::params![new_id, old_id],
        )?;
        tx.execute(
            "UPDATE tasks SET source_id = ?1 WHERE source_id = ?2",
            rusqlite::params![new_id, old_id],
        )?;
        tx.execute("DELETE FROM sources WHERE id = ?1", [old_id])?;
        tx.commit()?;
        Ok(())
    }

    /// Write chunks into chunks, vec_chunks, and chunks_fts tables.
    pub fn insert_chunks(&self, source_id: &str, chunks: &[NewChunk]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let dim = *self.dim.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        for c in chunks {
            if c.embedding.len() != dim {
                return Err(StoreError::DimMismatch {
                    expected: dim,
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

    pub fn source_chunk_text(&self, source_id: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT text FROM chunks WHERE source_id = ?1 ORDER BY ord ASC",
        )?;
        let rows = stmt.query_map([source_id], |r| r.get::<_, String>(0))?;
        let mut parts = Vec::new();
        for row in rows {
            parts.push(row?);
        }
        Ok(parts.join("\n\n"))
    }

    pub fn set_source_summary(&self, source_id: &str, summary: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "UPDATE sources SET summary = ?1 WHERE id = ?2",
            rusqlite::params![summary, source_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(source_id.to_string()));
        }
        Ok(())
    }

    pub fn delete_tasks_for_source(&self, source_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tasks WHERE source_id = ?1", [source_id])?;
        Ok(())
    }

    pub fn insert_task(
        &self,
        source_id: Option<&str>,
        title: &str,
        description: Option<&str>,
    ) -> Result<Task> {
        let now = unix_now();
        let id = format!("task-{now}-{}", title.len());
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO tasks(id, source_id, title, description, status, created_at, updated_at)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    id,
                    source_id,
                    title,
                    description,
                    TaskStatus::Pending.as_str(),
                    now,
                    now,
                ],
            )?;
        }
        self.get_task(&id)
    }

    pub fn get_task(&self, id: &str) -> Result<Task> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT t.id, t.source_id, s.title, t.title, t.description, t.status, t.created_at, t.updated_at
             FROM tasks t
             LEFT JOIN sources s ON s.id = t.source_id
             WHERE t.id = ?1",
            [id],
            Self::row_to_task,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_string()),
            other => StoreError::Sqlite(other),
        })
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.source_id, s.title, t.title, t.description, t.status, t.created_at, t.updated_at
             FROM tasks t
             LEFT JOIN sources s ON s.id = t.source_id
             ORDER BY t.updated_at DESC",
        )?;
        let rows = stmt.query_map([], Self::row_to_task)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn update_task_status(&self, id: &str, status: TaskStatus) -> Result<()> {
        let now = unix_now();
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![status.as_str(), now, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    /// KNN vector search (lower score = closer).
    pub fn search_vector(&self, query: &[f32], k: usize) -> Result<Vec<ChunkHit>> {
        let dim = *self.dim.lock().unwrap();
        if query.len() != dim {
            return Err(StoreError::DimMismatch {
                expected: dim,
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

    pub fn create_chat_session(&self, title: &str) -> Result<ChatSession> {
        let now = unix_now();
        let id = format!("sess-{now}");
        let session = ChatSession {
            id: id.clone(),
            title: title.to_string(),
            created_at: now,
            updated_at: now,
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_sessions(id, title, created_at, updated_at) VALUES(?1, ?2, ?3, ?4)",
            rusqlite::params![session.id, session.title, session.created_at, session.updated_at],
        )?;
        Ok(session)
    }

    pub fn list_chat_sessions(&self) -> Result<Vec<ChatSession>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at FROM chat_sessions ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(ChatSession {
                id: r.get(0)?,
                title: r.get(1)?,
                created_at: r.get(2)?,
                updated_at: r.get(3)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(StoreError::from)
    }

    pub fn get_chat_session(&self, id: &str) -> Result<ChatSession> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, title, created_at, updated_at FROM chat_sessions WHERE id = ?1",
            [id],
            |r| {
                Ok(ChatSession {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    created_at: r.get(2)?,
                    updated_at: r.get(3)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_string()),
            other => StoreError::Sqlite(other),
        })
    }

    pub fn delete_chat_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM chat_sessions WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn touch_chat_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE chat_sessions SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![unix_now(), id],
        )?;
        Ok(())
    }

    pub fn rename_chat_session(&self, id: &str, title: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE chat_sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![title, unix_now(), id],
        )?;
        if updated == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn append_chat_message(
        &self,
        session_id: &str,
        role: ChatRole,
        content: &str,
        citations_json: Option<&str>,
    ) -> Result<ChatMessage> {
        let now = unix_now();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_messages(session_id, role, content, citations_json, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![session_id, role.as_str(), content, citations_json, now],
        )?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "UPDATE chat_sessions SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        )?;
        Ok(ChatMessage {
            id,
            session_id: session_id.to_string(),
            role,
            content: content.to_string(),
            citations_json: citations_json.map(str::to_string),
            created_at: now,
        })
    }

    pub fn list_chat_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, citations_json, created_at
             FROM chat_messages WHERE session_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map([session_id], |r| {
            let role_s: String = r.get(2)?;
            Ok(ChatMessage {
                id: r.get(0)?,
                session_id: r.get(1)?,
                role: ChatRole::parse(&role_s).unwrap_or(ChatRole::User),
                content: r.get(3)?,
                citations_json: r.get(4)?,
                created_at: r.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(StoreError::from)
    }

    /// Drop and recreate the vector table, clear all chunks/FTS, mark sources pending.
    pub fn reinit_vectors(&self, new_dim: usize) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let ids: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT id FROM chunks")?;
            let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for id in &ids {
            conn.execute("DELETE FROM vec_chunks WHERE rowid = ?1", [id])?;
            conn.execute("DELETE FROM chunks_fts WHERE rowid = ?1", [id])?;
        }
        conn.execute("DELETE FROM chunks", [])?;
        conn.execute("DELETE FROM embed_cache", [])?;
        conn.execute(
            "UPDATE sources SET status = 'pending', indexed_at = NULL, error = NULL",
            [],
        )?;
        conn.execute("DROP TABLE IF EXISTS vec_chunks", [])?;
        conn.execute(
            &format!("CREATE VIRTUAL TABLE vec_chunks USING vec0(embedding float[{new_dim}])"),
            [],
        )?;
        drop(conn);
        *self.dim.lock().unwrap() = new_dim;
        self.set_meta("vector_dim", &new_dim.to_string())?;
        Ok(())
    }

    pub fn get_embed_cache(&self, text_hash: &str) -> Result<Option<Vec<f32>>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT vector FROM embed_cache WHERE text_hash = ?1",
            [text_hash],
            |row| {
                let raw: String = row.get(0)?;
                Ok(raw)
            },
        );

        match result {
            Ok(raw) => {
                let vec: Vec<f32> = serde_json::from_str(&raw)?;
                Ok(Some(vec))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StoreError::Sqlite(e)),
        }
    }

    pub fn put_embed_cache(&self, text_hash: &str, vector: &[f32]) -> Result<()> {
        let raw = serde_json::to_string(vector)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO embed_cache(text_hash, vector) VALUES(?1, ?2)
             ON CONFLICT(text_hash) DO UPDATE SET vector = excluded.vector",
            rusqlite::params![text_hash, raw],
        )?;
        Ok(())
    }

    pub fn index_health(&self) -> Result<IndexHealth> {
        Ok(IndexHealth {
            store_dim: self.dim(),
            stored_embedder_id: self.get_meta("embedder_id")?,
            source_count: self.list_sources()?.len(),
            chunk_count: self.count_chunks()? as usize,
        })
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
            summary: row.get(8)?,
        })
    }

    fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
        let status_s: String = row.get(5)?;
        Ok(Task {
            id: row.get(0)?,
            source_id: row.get(1)?,
            source_title: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            status: TaskStatus::parse(&status_s).unwrap_or(TaskStatus::Pending),
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NewChunk;

    fn journal_mode(store: &Store) -> String {
        let conn = store.conn.lock().unwrap();
        conn.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .unwrap()
    }

    #[test]
    fn store_open_enables_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kb.sqlite");
        let store = Store::open(&path, 4).unwrap();
        assert_eq!(journal_mode(&store).to_lowercase(), "wal");
    }

    #[test]
    fn open_in_memory_initializes_after_pragma_batch() {
        let store = Store::open_in_memory(4).unwrap();
        // In-memory DBs may report "memory", not "wal" — only require open + schema work.
        let _mode = journal_mode(&store);
        store.upsert_source(&sample_source("a")).unwrap();
        assert_eq!(store.list_sources().unwrap().len(), 1);
    }

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
            summary: None,
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
    fn rename_source_id_preserves_chunks() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("old-id")).unwrap();
        store
            .insert_chunks(
                "old-id",
                &[chunk(0, "memory body text", [1.0, 0.0, 0.0, 0.0])],
            )
            .unwrap();

        store.rename_source_id("old-id", "new-id").unwrap();

        let got = store.get_source("new-id").unwrap();
        assert_eq!(got.uri, "new-id");
        assert!(store
            .source_chunk_text("new-id")
            .unwrap()
            .contains("memory body text"));
        assert!(matches!(
            store.get_source("old-id"),
            Err(StoreError::NotFound(_))
        ));
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
    fn embed_cache_roundtrips_vector() {
        let store = Store::open_in_memory(4).unwrap();
        let vec = vec![0.1, 0.2, 0.3, 0.4];
        store.put_embed_cache("abc123", &vec).unwrap();
        let loaded = store.get_embed_cache("abc123").unwrap().unwrap();
        assert_eq!(loaded, vec);
        assert!(store.get_embed_cache("missing").unwrap().is_none());
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

    #[test]
    fn summary_and_tasks_crud() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks(
                "a",
                &[chunk(0, "task alpha and beta work", [0.1; 4])],
            )
            .unwrap();

        store.set_source_summary("a", "short summary").unwrap();
        assert_eq!(
            store.get_source("a").unwrap().summary.as_deref(),
            Some("short summary")
        );

        let task = store
            .insert_task(Some("a"), "Follow up", Some("details"))
            .unwrap();
        assert_eq!(store.list_tasks().unwrap().len(), 1);

        store
            .update_task_status(&task.id, TaskStatus::Done)
            .unwrap();
        assert_eq!(
            store.get_task(&task.id).unwrap().status,
            TaskStatus::Done
        );

        store.delete_tasks_for_source("a").unwrap();
        assert!(store.list_tasks().unwrap().is_empty());
    }

    #[test]
    fn chat_session_crud() {
        let store = Store::open_in_memory(4).unwrap();
        let s = store.create_chat_session("测试对话").unwrap();
        assert_eq!(store.list_chat_sessions().unwrap().len(), 1);

        store
            .append_chat_message(&s.id, ChatRole::User, "hello", None)
            .unwrap();
        let msgs = store.list_chat_messages(&s.id).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "hello");

        store.rename_chat_session(&s.id, "重命名").unwrap();
        assert_eq!(store.get_chat_session(&s.id).unwrap().title, "重命名");

        store.delete_chat_session(&s.id).unwrap();
        assert!(store.list_chat_sessions().unwrap().is_empty());
    }

    #[test]
    fn reinit_vectors_clears_chunks_and_updates_dim() {
        let store = Store::open_in_memory(4).unwrap();
        store.upsert_source(&sample_source("a")).unwrap();
        store
            .insert_chunks("a", &[chunk(0, "hello world", [1.0, 0.0, 0.0, 0.0])])
            .unwrap();
        assert_eq!(store.count_chunks().unwrap(), 1);

        store.reinit_vectors(8).unwrap();
        assert_eq!(store.dim(), 8);
        assert_eq!(store.count_chunks().unwrap(), 0);
        assert_eq!(
            store.get_source("a").unwrap().status,
            IndexStatus::Pending
        );
    }
}
