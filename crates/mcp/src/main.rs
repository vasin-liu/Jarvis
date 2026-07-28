use std::env;
use std::path::PathBuf;
use std::process;

use mcp::{ensure_db_exists, parse_db_flag, resolve_db_path};

fn main() {
    let db_flag = parse_db_flag(env::args());
    let data_dir = env::var_os("JARVIS_DATA_DIR").map(PathBuf::from);
    let db_path = resolve_db_path(db_flag, data_dir);

    if let Err(err) = ensure_db_exists(&db_path) {
        eprintln!("{err}");
        process::exit(1);
    }

    // Plan 17-04 wires Store::open + rmcp stdio serve.
    eprintln!("jarvis-mcp: database found at {}", db_path.display());
}
