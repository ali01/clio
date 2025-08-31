pub mod config;
pub mod error;
pub mod fetcher;
pub mod source;

// Re-export commonly used types
pub use config::Config;
pub use error::ClioError;
pub use fetcher::{FetchResult, FetchStats, Fetcher};
pub use source::{Item, Source};
