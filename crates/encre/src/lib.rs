//! encre: the Facile document format.
//!
//! A document is a ProseMirror JSON tree ([`Node`]) that validates against schema v3
//! (`spec/schema.json`). [`markdown`] projects it to Facile Markdown and back without loss.
#![deny(unused, dead_code)]

mod compact;
mod content;
mod escape;
mod ids;
pub mod markdown;
mod migrate;
mod model;
#[cfg(feature = "openui")]
pub mod openui;
pub mod ops;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod schema;
pub mod typst;
mod validate;
#[cfg(feature = "yjs")]
pub mod yjs;

pub use compact::compact;
pub use ids::{assign_ids, is_block_id};
pub use migrate::{MigrationError, migrate};
pub use model::{Mark, Node};
pub use schema::SCHEMA_VERSION;
pub use validate::{ValidationError, validate};
