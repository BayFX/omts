use serde::Serialize;

/// Serializes a type-tag enum (which implements `Serialize` to a JSON string)
/// and returns the unquoted string value.
///
/// Falls back to the `Debug` representation if serialization fails, which
/// should never happen for the well-defined enums in this crate.
pub(super) fn tag_to_string<T: Serialize>(tag: &T) -> String {
    match serde_json::to_value(tag) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(other) => format!("{other:?}"),
        Err(_) => "<unknown>".to_owned(),
    }
}
