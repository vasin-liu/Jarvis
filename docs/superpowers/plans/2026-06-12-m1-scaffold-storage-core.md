# M1 · 项目骨架 + 存储核心 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 搭好 Tauri 2 + React/TS 桌面应用骨架，并实现一个可独立测试的 `store` crate：用 SQLite + sqlite-vec + FTS5 完成来源/分片的建表、增删、向量 KNN 检索与 BM25 全文检索。

**Architecture:** Cargo workspace。核心逻辑放在独立库 crate `crates/store`，与 Tauri 解耦以便单元/集成测试；`src-tauri` 作为 Tauri 应用依赖 `store`。SQLite 单文件持久化，sqlite-vec 提供向量虚拟表，FTS5 提供 BM25 全文表。

**Tech Stack:** Rust, Tauri 2, React + TypeScript (Vite), rusqlite (bundled SQLite + FTS5), sqlite-vec, serde, thiserror；测试用 tempfile。

---

## 范围说明（M1 边界）

- **做**：workspace 脚手架、Tauri 能起一个窗口、`store` crate 的全部存储原语（schema、CRUD、向量检索、BM25 检索、meta 读写）。
- **不做**：解析文档、生成 embedding、RAG、飞书、前端业务界面。这些在 M2–M5。
- M1 完成的可验证产物：`cargo test -p store` 全绿；`npm run tauri dev` 能弹出窗口。

## 文件结构

```
00_Creativity/
├─ Cargo.toml                      # 创建：workspace 定义
├─ package.json                    # 创建：前端 + tauri 脚本（脚手架生成）
├─ index.html                      # 脚手架生成
├─ vite.config.ts                  # 脚手架生成
├─ src/                            # React 前端（脚手架生成，M1 仅保留默认页）
│  ├─ main.tsx
│  └─ App.tsx
├─ src-tauri/
│  ├─ Cargo.toml                   # 修改：依赖 store crate
│  ├─ tauri.conf.json             # 脚手架生成
│  └─ src/
│     ├─ main.rs                  # 修改：注册一个冒烟 command
│     └─ lib.rs                   # 修改：暴露 run()
└─ crates/
   └─ store/
      ├─ Cargo.toml               # 创建
      └─ src/
         ├─ lib.rs                # 创建：导出模块
         ├─ error.rs             # 创建：StoreError
         ├─ types.rs            # 创建：Source / Chunk / 等类型
         ├─ vecext.rs           # 创建：sqlite-vec 扩展注册
         ├─ schema.rs           # 创建：建表迁移
         └─ store.rs            # 创建：Store（CRUD + 检索）
```

每个文件单一职责：`types` 只放数据结构，`error` 只放错误，`schema` 只放 DDL，`vecext` 只管扩展加载，`store` 编排连接与查询。

---

### Task 0: 脚手架 — Tauri 2 + React/TS + workspace

**Files:**
- Create: 整个 Tauri 项目（脚手架生成）
- Create: `Cargo.toml`（workspace）
- Create: `crates/store/Cargo.toml`

- [ ] **Step 1: 用官方脚手架生成 Tauri 2 + React/TS 应用**

在 `D:\Work\99_Code\00_Creativity` 下运行（当前目录已为空，生成到当前目录）：

```bash
npm create tauri-app@latest . -- --template react-ts --manager npm
```

交互选择：前端语言 TypeScript / React，包管理器 npm，UI 模板 React。生成后安装依赖：

```bash
npm install
```

- [ ] **Step 2: 验证脚手架可运行**

Run: `npm run tauri dev`
Expected: 弹出一个 Tauri 桌面窗口，显示默认 React 欢迎页。确认后 Ctrl+C 关闭。

- [ ] **Step 3: 把项目改造成 Cargo workspace**

脚手架默认 `src-tauri` 是独立 crate。在仓库根创建 `Cargo.toml`：

```toml
[workspace]
members = ["src-tauri", "crates/store"]
resolver = "2"

[workspace.dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
sqlite-vec = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
```

- [ ] **Step 4: 创建 store crate 的 Cargo.toml**

Create `crates/store/Cargo.toml`:

```toml
[package]
name = "store"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { workspace = true }
sqlite-vec = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3"
```

Create `crates/store/src/lib.rs`:

```rust
pub mod error;
pub mod types;
mod vecext;
mod schema;
pub mod store;

pub use error::{Result, StoreError};
pub use store::Store;
pub use types::*;
```

- [ ] **Step 5: 验证 workspace 编译**

Run: `cargo build`
Expected: workspace 两个成员都编译通过（store 暂为空模块会因缺文件报错——先建空文件占位）。先创建空文件：`error.rs`、`types.rs`、`vecext.rs`、`schema.rs`、`store.rs` 内容暂为空。

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore: scaffold tauri2 react-ts app and store crate workspace"
```

---

### Task 1: store 错误类型与数据结构

**Files:**
- Modify: `crates/store/src/error.rs`
- Modify: `crates/store/src/types.rs`
- Test: 内联在 `types.rs`

- [ ] **Step 1: 写错误类型**

`crates/store/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimMismatch { expected: usize, got: usize },
}

pub type Result<T> = std::result::Result<T, StoreError>;
```

- [ ] **Step 2: 写数据结构 + 失败测试**

`crates/store/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    LocalFile,
    LarkDoc,
    LarkMsg,
    LarkSheet,
    LarkMail,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::LocalFile => "local_file",
            SourceKind::LarkDoc => "lark_doc",
            SourceKind::LarkMsg => "lark_msg",
            SourceKind::LarkSheet => "lark_sheet",
            SourceKind::LarkMail => "lark_mail",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "local_file" => SourceKind::LocalFile,
            "lark_doc" => SourceKind::LarkDoc,
            "lark_msg" => SourceKind::LarkMsg,
            "lark_sheet" => SourceKind::LarkSheet,
            "lark_mail" => SourceKind::LarkMail,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStatus {
    Pending,
    Indexed,
    Failed,
}

impl IndexStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexStatus::Pending => "pending",
            IndexStatus::Indexed => "indexed",
            IndexStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => IndexStatus::Pending,
            "indexed" => IndexStatus::Indexed,
            "failed" => IndexStatus::Failed,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub kind: SourceKind,
    pub uri: String,
    pub title: String,
    pub content_hash: String,
    pub indexed_at: Option<i64>,
    pub status: IndexStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewChunk {
    pub ord: i64,
    pub text: String,
    pub loc: String,
    pub token_count: i64,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkHit {
    pub chunk_id: i64,
    pub source_id: String,
    pub text: String,
    pub loc: String,
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_roundtrips() {
        for k in [
            SourceKind::LocalFile,
            SourceKind::LarkDoc,
            SourceKind::LarkMsg,
            SourceKind::LarkSheet,
            SourceKind::LarkMail,
        ] {
            assert_eq!(SourceKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(SourceKind::parse("bogus"), None);
    }

    #[test]
    fn index_status_roundtrips() {
        for s in [IndexStatus::Pending, IndexStatus::Indexed, IndexStatus::Failed] {
            assert_eq!(IndexStatus::parse(s.as_str()), Some(s));
        }
    }
}
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store types::`
Expected: PASS（`source_kind_roundtrips`、`index_status_roundtrips` 两个用例通过）

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/error.rs crates/store/src/types.rs
git commit -m "feat(store): add error type and core data structures"
```

---

### Task 2: sqlite-vec 扩展注册

**Files:**
- Modify: `crates/store/src/vecext.rs`

- [ ] **Step 1: 写扩展注册函数**

sqlite-vec 必须在创建任何连接之前注册为 auto-extension（注册后所有新连接自动加载）。用 `std::sync::Once` 保证只注册一次。

`crates/store/src/vecext.rs`:

```rust
use std::sync::Once;

static INIT: Once = Once::new();

/// 把 sqlite-vec 注册为 SQLite auto-extension。
/// 注册后，之后用 rusqlite 打开的每个连接都会自动加载 vec0 虚拟表能力。
/// 必须在打开任何业务连接前调用。
pub fn register_sqlite_vec() {
    INIT.call_once(|| {
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
    });
}
```

- [ ] **Step 2: 写冒烟测试 — 确认 vec0 可用**

在 `vecext.rs` 末尾追加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn vec_extension_loads_and_queries_version() {
        register_sqlite_vec();
        let conn = Connection::open_in_memory().unwrap();
        let version: String = conn
            .query_row("SELECT vec_version()", [], |r| r.get(0))
            .unwrap();
        assert!(version.starts_with('v'), "unexpected vec version: {version}");
    }
}
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store vecext::`
Expected: PASS（`vec_version()` 返回形如 `v0.x.x` 的字符串）。若报链接错误，确认 `sqlite-vec` 依赖已在 `crates/store/Cargo.toml`。

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/vecext.rs
git commit -m "feat(store): register sqlite-vec auto extension with smoke test"
```

---

### Task 3: schema 建表迁移

**Files:**
- Modify: `crates/store/src/schema.rs`

- [ ] **Step 1: 写迁移函数**

向量维度在建表时确定（vec0 需要固定维度），所以 `init_schema` 接收 `dim`。

`crates/store/src/schema.rs`:

```rust
use rusqlite::Connection;
use crate::error::Result;

pub const SCHEMA_VERSION: i64 = 1;

/// 在连接上建好全部表（幂等）。dim 为 embedding 维度。
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
```

- [ ] **Step 2: 写测试 — 确认表都建好**

在 `schema.rs` 末尾追加：

```rust
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
        for t in ["sources", "chunks", "embed_cache", "meta", "chunks_fts", "vec_chunks"] {
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
            .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, "1");
    }
}
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store schema::`
Expected: PASS（两个用例：建表齐全、可重复执行）

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/schema.rs
git commit -m "feat(store): add schema migration with all tables"
```

---

### Task 4: Store 打开 + sources 的增删查

**Files:**
- Modify: `crates/store/src/store.rs`

- [ ] **Step 1: 写 Store::open 与 source CRUD**

连接用 `Mutex` 包裹，保证单写者。开启外键约束以支持级联删除。

`crates/store/src/store.rs`:

```rust
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{Result, StoreError};
use crate::schema::init_schema;
use crate::types::{IndexStatus, Source, SourceKind};
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
        Ok(Self { conn: Mutex::new(conn), dim })
    }

    pub fn open_in_memory(dim: usize) -> Result<Self> {
        register_sqlite_vec();
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        init_schema(&conn, dim)?;
        Ok(Self { conn: Mutex::new(conn), dim })
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
```

- [ ] **Step 2: 写 source CRUD 测试**

在 `store.rs` 末尾追加（后续 Task 会往这个 `tests` 模块继续加；先建立它）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(store.get_source("a"), Err(StoreError::NotFound(_))));
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
}
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store store::tests::upsert`
Expected: PASS（`upsert_get_list_delete_source`、`upsert_updates_existing`）

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/store.rs
git commit -m "feat(store): add Store open and source CRUD"
```

---

### Task 5: 插入 chunk（向量 + FTS）与按来源删除

**Files:**
- Modify: `crates/store/src/store.rs`

- [ ] **Step 1: 在 impl Store 中新增 chunk 写入方法**

把以下方法加入 `impl Store`（放在 `delete_source` 之后、`row_to_source` 之前）：

```rust
    /// 为某来源写入一批 chunk：同时写 chunks 表、vec_chunks 向量表、chunks_fts 全文表。
    /// 三张表用 chunk 的自增 id 作为对齐键（vec_chunks.rowid / chunks_fts.rowid）。
    pub fn insert_chunks(&self, source_id: &str, chunks: &[crate::types::NewChunk]) -> Result<()> {
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

    /// 删除某来源下的所有 chunk（含向量与全文行）。
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
```

- [ ] **Step 2: 写测试 — 插入、维度校验、按来源删除**

在 `store.rs` 的 `tests` 模块里追加一个辅助构造器和测试：

```rust
    use crate::types::NewChunk;

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
            Err(StoreError::DimMismatch { expected: 4, got: 2 })
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
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store store::tests::insert store::tests::delete_chunks`
Expected: PASS（`insert_and_count_chunks`、`insert_rejects_wrong_dim`、`delete_chunks_for_source_clears_all_tables`）

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/store.rs
git commit -m "feat(store): insert chunks into vector+fts and delete by source"
```

---

### Task 6: 向量 KNN 检索

**Files:**
- Modify: `crates/store/src/store.rs`

- [ ] **Step 1: 新增向量检索方法**

加入 `impl Store`：

```rust
    /// 用查询向量做 KNN，返回最相近的 k 个 chunk（score = 距离，越小越相近）。
    pub fn search_vector(&self, query: &[f32], k: usize) -> Result<Vec<crate::types::ChunkHit>> {
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
            Ok(crate::types::ChunkHit {
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
```

- [ ] **Step 2: 写测试 — 最近邻顺序正确**

在 `tests` 模块追加：

```rust
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
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store store::tests::search_vector`
Expected: PASS（最近邻是 `x axis`，距离升序）

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/store.rs
git commit -m "feat(store): add vector knn search"
```

---

### Task 7: BM25 全文检索

**Files:**
- Modify: `crates/store/src/store.rs`

- [ ] **Step 1: 新增全文检索方法**

FTS5 的 `bm25()` 返回值越小越相关。为统一"score 越小越靠前"的语义，直接返回 bm25 原值。

加入 `impl Store`：

```rust
    /// FTS5 BM25 全文检索，返回最相关的 k 个 chunk（score = bm25，越小越相关）。
    pub fn search_fts(&self, query: &str, k: usize) -> Result<Vec<crate::types::ChunkHit>> {
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
            Ok(crate::types::ChunkHit {
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
```

- [ ] **Step 2: 写测试 — 关键词命中**

在 `tests` 模块追加：

```rust
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
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p store store::tests::search_fts`
Expected: PASS（"fox" 命中 2 条；无关词返回空）

- [ ] **Step 4: Commit**

```bash
git add crates/store/src/store.rs
git commit -m "feat(store): add fts5 bm25 search"
```

---

### Task 8: meta 读写 + 持久化集成测试

**Files:**
- Modify: `crates/store/src/store.rs`
- Test: `crates/store/tests/persistence.rs`

- [ ] **Step 1: 新增 meta 读写方法**

加入 `impl Store`：

```rust
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
```

- [ ] **Step 2: 写 meta 单元测试**

在 `tests` 模块追加：

```rust
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
```

- [ ] **Step 3: 写落盘持久化集成测试**

集成测试验证"写到真实文件 → 重新打开 → 数据还在 + 检索仍可用"。

Create `crates/store/tests/persistence.rs`:

```rust
use store::{IndexStatus, NewChunk, Source, SourceKind, Store};

fn src(id: &str) -> Source {
    Source {
        id: id.to_string(),
        kind: SourceKind::LocalFile,
        uri: format!("/tmp/{id}.md"),
        title: format!("t {id}"),
        content_hash: "h".to_string(),
        indexed_at: Some(123),
        status: IndexStatus::Indexed,
        error: None,
    }
}

#[test]
fn data_survives_reopen_and_is_searchable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kb.sqlite");

    {
        let store = Store::open(&path, 4).unwrap();
        store.upsert_source(&src("a")).unwrap();
        store
            .insert_chunks(
                "a",
                &[NewChunk {
                    ord: 0,
                    text: "rust vector search".into(),
                    loc: "L0".into(),
                    token_count: 3,
                    embedding: vec![1.0, 0.0, 0.0, 0.0],
                }],
            )
            .unwrap();
        store.set_meta("embedder_id", "fastembed:bge-small").unwrap();
    }

    // 重新打开同一个文件
    let store = Store::open(&path, 4).unwrap();
    assert_eq!(store.list_sources().unwrap().len(), 1);
    assert_eq!(store.count_chunks().unwrap(), 1);
    assert_eq!(
        store.get_meta("embedder_id").unwrap(),
        Some("fastembed:bge-small".to_string())
    );

    let vhits = store.search_vector(&[0.9, 0.1, 0.0, 0.0], 1).unwrap();
    assert_eq!(vhits[0].text, "rust vector search");

    let fhits = store.search_fts("vector", 5).unwrap();
    assert_eq!(fhits.len(), 1);
}

#[test]
fn cascade_delete_source_removes_chunks() {
    let store = Store::open_in_memory(4).unwrap();
    store.upsert_source(&src("a")).unwrap();
    store
        .insert_chunks(
            "a",
            &[NewChunk {
                ord: 0,
                text: "to be deleted".into(),
                loc: "L0".into(),
                token_count: 3,
                embedding: vec![0.0, 1.0, 0.0, 0.0],
            }],
        )
        .unwrap();

    // 先删 chunk 再删 source（FTS/vec 是非外键虚拟表，需显式清）
    store.delete_chunks_for_source("a").unwrap();
    store.delete_source("a").unwrap();

    assert_eq!(store.list_sources().unwrap().len(), 0);
    assert_eq!(store.count_chunks().unwrap(), 0);
}
```

- [ ] **Step 4: 运行全部 store 测试确认通过**

Run: `cargo test -p store`
Expected: PASS（全部单元测试 + `persistence.rs` 两个集成用例）

- [ ] **Step 5: Commit**

```bash
git add crates/store/src/store.rs crates/store/tests/persistence.rs
git commit -m "feat(store): add meta kv and persistence integration tests"
```

---

### Task 9: Tauri 接通 store 的冒烟 command

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/App.tsx`

- [ ] **Step 1: 让 src-tauri 依赖 store**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 增加：

```toml
store = { path = "../crates/store" }
```

- [ ] **Step 2: 写一个返回来源数量的 command**

把 `src-tauri/src/lib.rs` 改为（保留脚手架的 `run` 结构，加入 state 与 command）：

```rust
use std::sync::Arc;
use store::Store;

struct AppState {
    store: Arc<Store>,
}

#[tauri::command]
fn source_count(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    state
        .store
        .list_sources()
        .map(|v| v.len() as i64)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // M1 用内存库做冒烟；M5 会改为 app data 目录的真实文件。
    let store = Arc::new(Store::open_in_memory(4).expect("open store"));

    tauri::Builder::default()
        .manage(AppState { store })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![source_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

注意：保留脚手架已有的插件注册（如 `tauri_plugin_opener`）；若脚手架版本不同，按其原 `run()` 内已有的 `.plugin(...)` 链调整，仅新增 `.manage(...)` 与 `.invoke_handler(...)`。

- [ ] **Step 3: 前端调用该 command 验证打通**

把 `src/App.tsx` 的默认内容替换为最小验证页：

```tsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [count, setCount] = useState<number | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    invoke<number>("source_count")
      .then(setCount)
      .catch((e) => setErr(String(e)));
  }, []);

  return (
    <main style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1>知识中枢 · M1 冒烟</h1>
      {err ? <p>错误：{err}</p> : <p>来源数量：{count ?? "加载中…"}</p>}
    </main>
  );
}

export default App;
```

- [ ] **Step 4: 运行应用确认打通**

Run: `npm run tauri dev`
Expected: 窗口显示「来源数量：0」（内存库初始为空，证明前端→Tauri command→store 链路打通）。确认后 Ctrl+C。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src/App.tsx
git commit -m "feat: wire tauri command to store with frontend smoke check"
```

---

## Self-Review

**1. Spec 覆盖**：M1 对应 spec 第 5 节（数据模型）与第 3 节中的 `store` 模块。本计划任务覆盖：建表（Task 3）、sources CRUD（Task 4）、chunks 写入与级联清理（Task 5）、向量检索（Task 6）、BM25（Task 7）、meta（Task 8）、Tauri 骨架（Task 0/9）。混合检索的 RRF 融合属 M3（retriever），不在 M1，符合里程碑划分。

**2. 占位符扫描**：无 TBD/TODO；每个代码步骤均给出完整代码与可运行命令、预期结果。

**3. 类型一致性**：`Source`/`NewChunk`/`ChunkHit`/`SourceKind`/`IndexStatus` 在 Task 1 定义，后续 Task 4–9 使用的字段名（`id/kind/uri/title/content_hash/indexed_at/status/error`、`ord/text/loc/token_count/embedding`、`chunk_id/source_id/text/loc/score`）与定义一致。方法名 `upsert_source/get_source/list_sources/delete_source/insert_chunks/delete_chunks_for_source/count_chunks/search_vector/search_fts/set_meta/get_meta` 在各任务间保持一致。`Store::open`/`open_in_memory` 签名 `(path, dim)` 一致。

**已知实现注意点（执行时留意，非计划缺陷）**：
- `sqlite-vec`/`rusqlite` 的具体小版本以执行时 `cargo add` 解析为准；若 `sqlite3_auto_extension` 的 transmute 签名因版本变化，按 sqlite-vec 文档当时示例调整。
- vec0 的 KNN 语法用 `embedding MATCH ?1 AND k = ?2`；若该版本要求 `ORDER BY distance LIMIT k` 形式，按其文档改写 `search_vector` 的 SQL。
- 脚手架生成的 `lib.rs`/插件链可能随 Tauri 小版本不同，Task 9 以"在既有 `run()` 上新增 manage + handler"为准。

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-12-m1-scaffold-storage-core.md`. 两种执行方式：

1. **Subagent-Driven（推荐）** — 每个 Task 派发一个全新 subagent，任务间我来审查，迭代快。
2. **Inline Execution** — 在当前会话用 executing-plans 批量执行，带检查点审查。

选哪种？
