use rusqlite::Connection;

use crate::error::Result;

pub const SCHEMA_VERSION: i64 = 3;

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
            error        TEXT,
            summary      TEXT
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

        CREATE TABLE IF NOT EXISTS chat_sessions (
            id         TEXT PRIMARY KEY,
            title      TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id     TEXT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
            role           TEXT NOT NULL,
            content        TEXT NOT NULL,
            citations_json TEXT,
            created_at     INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_chat_messages_session ON chat_messages(session_id);

        CREATE TABLE IF NOT EXISTS tasks (
            id          TEXT PRIMARY KEY,
            source_id   TEXT REFERENCES sources(id) ON DELETE SET NULL,
            title       TEXT NOT NULL,
            description TEXT,
            status      TEXT NOT NULL,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_tasks_source ON tasks(source_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
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

    migrate(conn)?;

    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    let mut version: i64 = conn
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    if version < 2 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS chat_sessions (
                id         TEXT PRIMARY KEY,
                title      TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chat_messages (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id     TEXT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
                role           TEXT NOT NULL,
                content        TEXT NOT NULL,
                citations_json TEXT,
                created_at     INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_chat_messages_session ON chat_messages(session_id);
            ",
        )?;
        conn.execute(
            "UPDATE meta SET value = '2' WHERE key = 'schema_version'",
            [],
        )?;
        version = 2;
    }

    if version < 3 {
        conn.execute_batch(
            "
            ALTER TABLE sources ADD COLUMN summary TEXT;
            CREATE TABLE IF NOT EXISTS tasks (
                id          TEXT PRIMARY KEY,
                source_id   TEXT REFERENCES sources(id) ON DELETE SET NULL,
                title       TEXT NOT NULL,
                description TEXT,
                status      TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                updated_at  INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_source ON tasks(source_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            ",
        )?;
        conn.execute(
            "UPDATE meta SET value = '3' WHERE key = 'schema_version'",
            [],
        )?;
    }

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
            "chat_sessions",
            "chat_messages",
            "tasks",
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
        assert_eq!(version, "3");
    }

    #[test]
    fn migrate_v1_to_v2_adds_chat_tables() {
        register_sqlite_vec();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta VALUES ('schema_version', '1');
            CREATE TABLE sources (
                id TEXT PRIMARY KEY, kind TEXT NOT NULL, uri TEXT NOT NULL,
                title TEXT NOT NULL, content_hash TEXT NOT NULL,
                indexed_at INTEGER, status TEXT NOT NULL, error TEXT
            );
            ",
        )
        .unwrap();
        migrate(&conn).unwrap();
        assert!(table_exists(&conn, "chat_sessions"));
        assert!(table_exists(&conn, "chat_messages"));
        let version: String = conn
            .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn migrate_v2_to_v3_adds_summary_and_tasks() {
        register_sqlite_vec();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta VALUES ('schema_version', '2');
            CREATE TABLE sources (
                id TEXT PRIMARY KEY, kind TEXT NOT NULL, uri TEXT NOT NULL,
                title TEXT NOT NULL, content_hash TEXT NOT NULL,
                indexed_at INTEGER, status TEXT NOT NULL, error TEXT
            );
            ",
        )
        .unwrap();
        migrate(&conn).unwrap();
        assert!(table_exists(&conn, "tasks"));
        let version: String = conn
            .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, "3");
    }
}
