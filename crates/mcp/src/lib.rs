mod error;
mod paths;
mod server;

pub use error::{McpError, Result};
pub use paths::{
    default_app_data_dir, ensure_db_exists, parse_db_flag, resolve_db_path,
};
pub use server::{JarvisMcp, SearchParams};
