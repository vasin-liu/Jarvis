mod error;
mod rrf;
mod retrieve;

pub use error::{Result, RetrieveError};
pub use retrieve::{retrieve, RetrieverConfig};
