use super::types::PropertyChange;

/// Floating-point epsilon for numeric field comparisons (diff.md Section 3.1).
pub(super) const NUMERIC_EPSILON: f64 = 1e-9;

/// Normalises a date string to `YYYY-MM-DD` by zero-padding month and day if
/// they are written without leading zeros (e.g. `"2026-2-9"` → `"2026-02-09"`).
///
/// A conformant `CalendarDate` is already zero-padded, but the spec says the
/// diff engine should normalise before comparing to avoid false positives
/// (diff.md Section 3.1).
pub(super) fn normalise_date(s: &str) -> String {
    let parts: Vec<&str> = s.splitn(3, '-').collect();
    if parts.len() == 3 {
        let year = parts[0];
        let month = parts[1];
        let day = parts[2];
        format!(
            "{}-{:0>2}-{:0>2}",
            year,
            month
                .trim_start_matches('0')
                .parse::<u32>()
                .map_or_else(|_| month.to_owned(), |n| format!("{n:02}")),
            day.trim_start_matches('0')
                .parse::<u32>()
                .map_or_else(|_| day.to_owned(), |n| format!("{n:02}"))
        )
    } else {
        s.to_owned()
    }
}

/// Converts a `serde_json::Value` that might represent a date string to its
/// normalised form. Non-string values are returned as-is.
pub(super) fn normalise_date_value(v: &serde_json::Value) -> serde_json::Value {
    if let Some(s) = v.as_str() {
        // Only normalise if it looks like a date (contains hyphens).
        if s.contains('-') {
            return serde_json::Value::String(normalise_date(s));
        }
    }
    v.clone()
}

/// Returns `true` if two `serde_json::Value`s are semantically equal under the
/// diff rules:
/// - For strings that look like dates, normalise before comparing.
/// - For numbers, use epsilon comparison.
/// - Otherwise, use structural equality.
pub(super) fn values_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value;
    match (a, b) {
        // Both numbers: compare with epsilon.
        (Value::Number(na), Value::Number(nb)) => {
            match (na.as_f64(), nb.as_f64()) {
                (Some(fa), Some(fb)) => (fa - fb).abs() < NUMERIC_EPSILON,
                // If one can't be represented as f64 (very rare for valid JSON),
                // fall back to structural equality of the Number tokens.
                _ => na == nb,
            }
        }
        // Both strings that look like dates: normalise before comparing.
        (Value::String(sa), Value::String(sb)) => {
            if sa.contains('-') && sb.contains('-') {
                normalise_date(sa) == normalise_date(sb)
            } else {
                sa == sb
            }
        }
        // Everything else: structural equality.
        _ => a == b,
    }
}

/// Converts an `Option<T>` to `Option<serde_json::Value>` by serialising.
/// Returns `None` if the input is `None` or if serialization fails.
pub(super) fn to_value<T: serde::Serialize>(v: &Option<T>) -> Option<serde_json::Value> {
    let inner = v.as_ref()?;
    serde_json::to_value(inner).ok()
}

/// Emits a `PropertyChange` if `old_value` and `new_value` differ (or if one
/// is `None` and the other is not), using semantic equality.
///
/// Uses `values_equal` for the comparison so that date normalisation and
/// numeric epsilon are applied.
pub(super) fn maybe_change(
    field: &str,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    out: &mut Vec<PropertyChange>,
) {
    let equal = match (&old_value, &new_value) {
        (None, None) => true,
        (Some(a), Some(b)) => values_equal(a, b),
        _ => false,
    };
    if !equal {
        out.push(PropertyChange {
            field: field.to_owned(),
            old_value,
            new_value,
        });
    }
}
