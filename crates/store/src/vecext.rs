use std::sync::Once;

static INIT: Once = Once::new();

/// Register sqlite-vec as a SQLite auto-extension.
/// Must be called before opening any business connection.
pub fn register_sqlite_vec() {
    INIT.call_once(|| {
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
    });
}

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
        assert!(
            version.starts_with('v'),
            "unexpected vec version: {version}"
        );
    }
}
