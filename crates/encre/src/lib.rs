//! encre: the Facile document format.
//!
//! A document is a ProseMirror JSON tree ([`Node`]) that validates against schema v1
//! (`spec/schema.json`). [`markdown`] projects it to Facile Markdown and back without loss.
#![deny(unused, dead_code)]

mod content;
mod ids;
pub mod markdown;
mod migrate;
mod model;
pub mod schema;
mod validate;

pub use ids::{assign_ids, is_block_id};
pub use migrate::{MigrationError, migrate};
pub use model::{Mark, Node};
pub use schema::SCHEMA_VERSION;
pub use validate::{ValidationError, validate};
