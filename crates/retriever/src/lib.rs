mod error;
mod kb_readonly;
mod related;
mod rrf;
mod retrieve;

pub use error::{Result, RetrieveError};
pub use kb_readonly::{
    list_indexed_sources, search_kb, truncate_excerpt, KbHit, KbSourceEntry, KbSourceList,
    EXCERPT_MAX_CHARS, LIST_SOURCES_MAX,
};
pub use related::{related_sources, RelatedSource};
pub use retrieve::{retrieve, RetrieverConfig};
