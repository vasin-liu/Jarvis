mod document;
mod error;
mod hash;
mod loader;

pub use document::Document;
pub use error::{IngestError, Result};
pub use loader::load_path;
