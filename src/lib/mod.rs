//! The pgrep module allows quickly filtering through discovered source code project roots
//!
//! Just write the following `code.toml` config somewhere:
//! ```toml
//! [general]
//! folders = ['/home/<user>/development']
//! ```
//! 
//! Then run `pgrep '*test*' --config 'code.toml'` to find projects containing `test` either in the path or name.
//! 
//! # Supported project kinds
//! 
//! For now only [`crate::ProjectKind`] are supported but over time, this list will grow.
//! 
//! # Caching
//! 
//! Scanned folders and discovered projects are cached every [`Cache::CACHE_BUST_THRESHOLD`].
//! 
//! You can specify the `--no-cache` comande-line options to disable cache.
//! Or manually bust it using the exclusive `--clean-cache`

pub mod app;
pub mod cache;
pub mod config;
pub mod error;
pub mod project;
pub mod query;
pub mod options;
pub mod fmt;
pub mod ui;

pub use app::*;
pub use cache::*;
pub use config::*;
pub use error::*;
pub use project::*;
pub use query::*;
pub use options::*;
pub use fmt::*;
pub use ui::*;
