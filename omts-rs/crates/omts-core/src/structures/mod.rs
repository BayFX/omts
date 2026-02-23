/// Node, Edge, and `EdgeProperties` structs for the OMTS graph data model.
///
/// This module defines the primary graph entity types as specified in
/// data-model.md Sections 5.1–5.3 and 6.1–6.3.
///
/// Key design decisions:
/// - All fields beyond `id` and `node_type`/`edge_type` are `Option<T>` so a
///   single struct covers all node/edge subtypes without enum overhead.
/// - `valid_to` uses `Option<Option<CalendarDate>>` to distinguish absent (field
///   omitted) from explicit `null` (open-ended validity). See
///   [`crate::serde_helpers::deserialize_optional_nullable`].
/// - `#[serde(flatten)] pub extra` on all three structs preserves unknown JSON
///   fields across round trips (SPEC-001 Section 2.2).
mod edge;
mod node;

pub use edge::{Edge, EdgeProperties};
pub use node::Node;

#[cfg(test)]
mod tests;
