use crate::model::Node;
use crate::schema::SCHEMA_VERSION;

/// Why a stored document cannot be upgraded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MigrationError {
    /// The stored version is not one this crate knows.
    #[error("unknown schema version {0}; this build reads 1 to {SCHEMA_VERSION}")]
    UnknownVersion(u32),
}

type Migration = fn(Node) -> Node;

/// Schema 2 only adds node types, a mark and an optional attribute, so a version 1 document is unchanged.
fn v1_to_v2(doc: Node) -> Node {
    doc
}

const MIGRATIONS: &[Migration] = &[v1_to_v2];

/// Upgrades `doc`, stored at schema version `from`, to [`SCHEMA_VERSION`].
pub fn migrate(doc: Node, from: u32) -> Result<Node, MigrationError> {
    if from == 0 || from > SCHEMA_VERSION {
        return Err(MigrationError::UnknownVersion(from));
    }
    Ok(MIGRATIONS[(from - 1) as usize..].iter().fold(doc, |doc, step| step(doc)))
}
