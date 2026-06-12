use rusqlite::Connection;

use crate::error::Result;

pub const SCHEMA_VERSION: i64 = 1;

/// Create all tables (idempotent). `dim` is the embedding dimension for vec0.
pub fn init_schema(conn: &Connection, dim: usize) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sources (
            id           TEXT PRIMARY KEY,
            kind         TEXT NOT NULL,
            uri          TEXT NOT NULL,
            title        TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            indexed_at   INTEGER,
            status       TEXT NOT NULL,
            error        TEXT
        );

        CREATE TABLE IF NOT EXISTS chunks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id   TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
            ord         INTEGER NOT NULL,
            text        TEXT NOT NULL,
            loc         TEXT NOT NULL,
            token_count INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_chunks_source ON chunks(source_id);

        CREATE TABLE IF NOT EXISTS embed_cache (
            text_hash TEXT PRIMARY KEY,
            vector    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(text);
        ",
    )?;

    conn.execute(
        &format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vec_chunks USING vec0(embedding float[{dim}])"
        ),
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO meta(key, value) VALUES ('schema_version', ?1)",
        [SCHEMA_VERSION.to_string()],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vecext::register_sqlite_vec;
    use rusqlite::Connection;

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE name = ?1",
            [name],
            |_| Ok(()),
        )
        .is_ok()
    }

    #[test]
    fn init_schema_creates_all_tables() {
        register_sqlite_vec();
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn, 4).unwrap();
        for t in [
            "sources",
            "chunks",
            "embed_cache",
            "meta",
            "chunks_fts",
            "vec_chunks",
        ] {
            assert!(table_exists(&conn, t), "missing table: {t}");
        }
    }

    #[test]
    fn init_schema_is_idempotent() {
        register_sqlite_vec();
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn, 4).unwrap();
        init_schema(&conn, 4).unwrap();
        let version: String = conn
            .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, "1");
    }
}
