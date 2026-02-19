/// Identity predicates for the merge engine.
///
/// Implements the node identity predicate and temporal compatibility check
/// described in merge.md Sections 2.2 and 3.1.
///
/// All functions in this module are pure (no side-effects, no I/O).
use crate::newtypes::CalendarDate;
use crate::types::Identifier;

// ---------------------------------------------------------------------------
// identifiers_match
// ---------------------------------------------------------------------------

/// Returns `true` when two [`Identifier`] records should be considered the
/// same identifier for merge purposes.
///
/// The predicate is symmetric by construction; every comparison is symmetric
/// (string equality, case-insensitive equality, interval overlap), so
/// `identifiers_match(a, b) == identifiers_match(b, a)` always holds.
///
/// # Rules (applied in order)
///
/// 1. **Internal scheme excluded** — if either identifier uses the `"internal"`
///    scheme, return `false`. Internal identifiers are private to each
///    reporting entity and must never trigger a merge.
/// 2. **Schemes must match** — `a.scheme != b.scheme` → `false`.
/// 3. **Values must match (whitespace-trimmed)** — leading/trailing whitespace
///    in a stored value is normalised away before comparison.
/// 4. **Authority check** — if either record carries an `authority` field,
///    *both* must carry it and it must match case-insensitively. If one has
///    authority and the other does not, return `false`.
/// 5. **Temporal compatibility** — the validity intervals must overlap; see
///    [`temporal_compatible`] for the detailed rules.
pub fn identifiers_match(a: &Identifier, b: &Identifier) -> bool {
    // Rule 1: Exclude internal scheme.
    if a.scheme == "internal" || b.scheme == "internal" {
        return false;
    }

    // Rule 2: Schemes must match.
    if a.scheme != b.scheme {
        return false;
    }

    // Rule 3: Values must match (whitespace-trimmed).
    if a.value.trim() != b.value.trim() {
        return false;
    }

    // Rule 4: Authority check.
    if a.authority.is_some() || b.authority.is_some() {
        match (&a.authority, &b.authority) {
            (Some(aa), Some(ba)) => {
                if !aa.eq_ignore_ascii_case(ba) {
                    return false;
                }
            }
            // One has authority, the other does not.
            (Some(_), None) | (None, Some(_)) => return false,
            // Both None is handled by the outer `is_some()` guard above and
            // can never reach this arm, but the match must be exhaustive.
            (None, None) => {}
        }
    }

    // Rule 5: Temporal compatibility.
    temporal_compatible(a, b)
}

// ---------------------------------------------------------------------------
// temporal_compatible
// ---------------------------------------------------------------------------

/// Returns `true` when two identifier records' validity intervals overlap.
///
/// The full three-state semantics of `valid_to` on [`Identifier`] are:
/// - `None` — field absent (temporal bounds not supplied at all).
/// - `Some(None)` — explicit JSON `null` (identifier has no expiry; open-ended
///   into the future).
/// - `Some(Some(date))` — expires on the given date.
///
/// # Rules
///
/// - If *either* record omits **both** `valid_from` and `valid_to` entirely
///   (i.e. both fields are `None`), temporal compatibility is assumed.
/// - Two intervals overlap when it is *not* the case that one ends strictly
///   before the other begins. Specifically, incompatibility is declared only
///   when both records have a concrete `valid_to` date, one `valid_to` is
///   strictly less than the other's `valid_from`, and that `valid_from` is
///   present. An explicit `valid_to: null` (no-expiry) never causes
///   incompatibility.
pub fn temporal_compatible(a: &Identifier, b: &Identifier) -> bool {
    // If either record has no temporal information at all, assume compatible.
    let a_has_temporal = a.valid_from.is_some() || a.valid_to.is_some();
    let b_has_temporal = b.valid_from.is_some() || b.valid_to.is_some();
    if !a_has_temporal || !b_has_temporal {
        return true;
    }

    // Check whether interval A ends before interval B starts.
    if intervals_disjoint(a.valid_to.as_ref(), b.valid_from.as_ref()) {
        return false;
    }

    // Check whether interval B ends before interval A starts.
    if intervals_disjoint(b.valid_to.as_ref(), a.valid_from.as_ref()) {
        return false;
    }

    true
}

/// Returns `true` when an interval that ends at `end` is strictly before an
/// interval that starts at `start`.
///
/// - `end = None` — field absent; treated as open-ended (never disjoint on
///   this end).
/// - `end = Some(None)` — explicit no-expiry; open-ended (never disjoint).
/// - `end = Some(Some(date))` — concrete end date.
/// - `start = None` — field absent; treated as open-ended from the beginning.
///
/// Disjoint only when `end < start` with both values concrete.
fn intervals_disjoint(end: Option<&Option<CalendarDate>>, start: Option<&CalendarDate>) -> bool {
    // If start is absent, the interval is open-ended at the left; never
    // disjoint on that end.
    let Some(start_date) = start else {
        return false;
    };

    // Resolve the end value.
    match end {
        // end field absent → open-ended; not disjoint.
        None => false,
        // explicit null → no-expiry; not disjoint.
        Some(None) => false,
        // concrete end date: disjoint iff end < start
        Some(Some(end_date)) => end_date < start_date,
    }
}

// ---------------------------------------------------------------------------
// is_lei_annulled
// ---------------------------------------------------------------------------

/// Returns `true` when an LEI identifier is known to be in ANNULLED status.
///
/// The `VerificationStatus` enum does not include an `Annulled` variant; GLEIF
/// ANNULLED status is typically carried as enrichment data outside the core
/// schema. This function inspects the identifier's `extra` extension fields for
/// a best-effort detection of annulled LEIs.
///
/// # Detection strategy
///
/// The function checks:
/// 1. `id.scheme == "lei"`.
/// 2. The `extra` map contains `"entity_status"` with the string value
///    `"ANNULLED"` (case-sensitive, as GLEIF uses all-caps status codes).
///
/// If L2 enrichment data is unavailable, this function returns `false` (no
/// false-positive exclusions). Callers that have richer LEI data should apply
/// their own filtering before index construction.
pub fn is_lei_annulled(id: &Identifier) -> bool {
    if id.scheme != "lei" {
        return false;
    }
    matches!(
        id.extra.get("entity_status").and_then(|v| v.as_str()),
        Some("ANNULLED")
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use serde_json::json;

    use crate::newtypes::CalendarDate;

    use super::*;

    // --- helpers ------------------------------------------------------------

    fn make_id(scheme: &str, value: &str) -> Identifier {
        Identifier {
            scheme: scheme.to_owned(),
            value: value.to_owned(),
            authority: None,
            valid_from: None,
            valid_to: None,
            sensitivity: None,
            verification_status: None,
            verification_date: None,
            extra: serde_json::Map::new(),
        }
    }

    fn with_authority(mut id: Identifier, authority: &str) -> Identifier {
        id.authority = Some(authority.to_owned());
        id
    }

    fn with_valid_from(mut id: Identifier, date: &str) -> Identifier {
        id.valid_from = Some(CalendarDate::try_from(date).expect("valid date"));
        id
    }

    fn with_valid_to_date(mut id: Identifier, date: &str) -> Identifier {
        id.valid_to = Some(Some(CalendarDate::try_from(date).expect("valid date")));
        id
    }

    fn with_valid_to_null(mut id: Identifier) -> Identifier {
        // Explicit no-expiry (JSON null)
        id.valid_to = Some(None);
        id
    }

    // --- identifiers_match --------------------------------------------------

    #[test]
    fn same_scheme_and_value_matches() {
        let a = make_id("lei", "LEI_ACME");
        let b = make_id("lei", "LEI_ACME");
        assert!(identifiers_match(&a, &b));
    }

    #[test]
    fn different_scheme_rejects() {
        let a = make_id("lei", "VALUE");
        let b = make_id("duns", "VALUE");
        assert!(!identifiers_match(&a, &b));
    }

    #[test]
    fn internal_scheme_on_a_rejects() {
        let a = make_id("internal", "sap:1234");
        let b = make_id("lei", "sap:1234");
        assert!(!identifiers_match(&a, &b));
    }

    #[test]
    fn internal_scheme_on_b_rejects() {
        let a = make_id("lei", "VAL");
        let b = make_id("internal", "VAL");
        assert!(!identifiers_match(&a, &b));
    }

    #[test]
    fn both_internal_rejects() {
        let a = make_id("internal", "X");
        let b = make_id("internal", "X");
        assert!(!identifiers_match(&a, &b));
    }

    #[test]
    fn whitespace_trimmed_values_match() {
        let a = make_id("lei", " LEI_ACME ");
        let b = make_id("lei", "LEI_ACME");
        assert!(identifiers_match(&a, &b));
    }

    #[test]
    fn whitespace_trimmed_values_both_padded_match() {
        let a = make_id("duns", "  123456789  ");
        let b = make_id("duns", "  123456789  ");
        assert!(identifiers_match(&a, &b));
    }

    #[test]
    fn different_values_rejects() {
        let a = make_id("lei", "LEI_A");
        let b = make_id("lei", "LEI_B");
        assert!(!identifiers_match(&a, &b));
    }

    #[test]
    fn authority_case_insensitive_match() {
        let a = with_authority(make_id("nat-reg", "HRB12345"), "DE");
        let b = with_authority(make_id("nat-reg", "HRB12345"), "de");
        assert!(identifiers_match(&a, &b));
    }

    #[test]
    fn authority_case_insensitive_mixed_case() {
        let a = with_authority(make_id("nat-reg", "HRB12345"), "GLEIF");
        let b = with_authority(make_id("nat-reg", "HRB12345"), "gleif");
        assert!(identifiers_match(&a, &b));
    }

    #[test]
    fn authority_mismatch_rejects() {
        let a = with_authority(make_id("nat-reg", "HRB12345"), "DE");
        let b = with_authority(make_id("nat-reg", "HRB12345"), "FR");
        assert!(!identifiers_match(&a, &b));
    }

    #[test]
    fn one_has_authority_other_does_not_rejects() {
        let a = with_authority(make_id("nat-reg", "HRB12345"), "DE");
        let b = make_id("nat-reg", "HRB12345");
        assert!(!identifiers_match(&a, &b));
        // Symmetric
        assert!(!identifiers_match(&b, &a));
    }

    #[test]
    fn no_authority_on_either_matches_without_authority_check() {
        // Both lack authority → authority check is skipped.
        let a = make_id("lei", "SAME_VAL");
        let b = make_id("lei", "SAME_VAL");
        assert!(identifiers_match(&a, &b));
    }

    // --- temporal_compatible ------------------------------------------------

    #[test]
    fn both_missing_temporal_is_compatible() {
        let a = make_id("lei", "X");
        let b = make_id("lei", "X");
        // Both have no valid_from and no valid_to → compatible by default.
        assert!(temporal_compatible(&a, &b));
    }

    #[test]
    fn one_missing_temporal_is_compatible() {
        // One record has temporal info, the other does not → compatible.
        let a = with_valid_from(make_id("lei", "X"), "2020-01-01");
        let b = make_id("lei", "X");
        assert!(temporal_compatible(&a, &b));
        assert!(temporal_compatible(&b, &a));
    }

    #[test]
    fn overlapping_intervals_are_compatible() {
        // a: [2020-01-01, 2022-12-31], b: [2021-01-01, 2023-12-31] — overlap in 2021-2022
        let a = with_valid_to_date(
            with_valid_from(make_id("lei", "X"), "2020-01-01"),
            "2022-12-31",
        );
        let b = with_valid_to_date(
            with_valid_from(make_id("lei", "X"), "2021-01-01"),
            "2023-12-31",
        );
        assert!(temporal_compatible(&a, &b));
    }

    #[test]
    fn non_overlapping_intervals_are_incompatible() {
        // a ends 2019-12-31, b starts 2020-01-01 → disjoint
        let a = with_valid_to_date(
            with_valid_from(make_id("lei", "X"), "2018-01-01"),
            "2019-12-31",
        );
        let b = with_valid_from(make_id("lei", "X"), "2020-01-01");
        assert!(!temporal_compatible(&a, &b));
        assert!(!temporal_compatible(&b, &a));
    }

    #[test]
    fn adjacent_intervals_on_same_date_are_compatible() {
        // a: valid_to 2020-12-31, b: valid_from 2020-12-31 — same date → not strictly less than
        let a = with_valid_to_date(make_id("lei", "X"), "2020-12-31");
        let b = with_valid_from(make_id("lei", "X"), "2020-12-31");
        assert!(temporal_compatible(&a, &b));
    }

    #[test]
    fn valid_to_null_no_expiry_is_compatible() {
        // Explicit no-expiry on one side, dated start on the other
        let a = with_valid_to_null(with_valid_from(make_id("lei", "X"), "2020-01-01"));
        let b = with_valid_from(make_id("lei", "X"), "2025-01-01");
        assert!(temporal_compatible(&a, &b));
    }

    #[test]
    fn valid_to_null_both_sides_are_compatible() {
        let a = with_valid_to_null(with_valid_from(make_id("lei", "X"), "2020-01-01"));
        let b = with_valid_to_null(with_valid_from(make_id("lei", "X"), "2021-01-01"));
        assert!(temporal_compatible(&a, &b));
    }

    #[test]
    fn identifiers_match_temporal_incompatibility_rejects() {
        // Same scheme/value/authority but non-overlapping temporal windows → reject
        let a = with_valid_to_date(make_id("lei", "LEI_ACME"), "2019-12-31");
        let b = with_valid_from(make_id("lei", "LEI_ACME"), "2020-06-01");
        assert!(!identifiers_match(&a, &b));
    }

    // --- is_lei_annulled ----------------------------------------------------

    #[test]
    fn non_lei_scheme_not_annulled() {
        let id = make_id("duns", "123");
        assert!(!is_lei_annulled(&id));
    }

    #[test]
    fn lei_without_entity_status_not_annulled() {
        let id = make_id("lei", "SOME_LEI");
        assert!(!is_lei_annulled(&id));
    }

    #[test]
    fn lei_with_annulled_status_is_annulled() {
        let mut id = make_id("lei", "SOME_LEI");
        id.extra
            .insert("entity_status".to_owned(), json!("ANNULLED"));
        assert!(is_lei_annulled(&id));
    }

    #[test]
    fn lei_with_active_status_not_annulled() {
        let mut id = make_id("lei", "SOME_LEI");
        id.extra.insert("entity_status".to_owned(), json!("ACTIVE"));
        assert!(!is_lei_annulled(&id));
    }

    #[test]
    fn lei_with_lowercase_annulled_not_annulled() {
        // GLEIF uses uppercase; lowercase is not a match (case-sensitive).
        let mut id = make_id("lei", "SOME_LEI");
        id.extra
            .insert("entity_status".to_owned(), json!("annulled"));
        assert!(!is_lei_annulled(&id));
    }

    #[test]
    fn internal_scheme_is_not_annulled_check() {
        let mut id = make_id("internal", "LEI_VAL");
        id.extra
            .insert("entity_status".to_owned(), json!("ANNULLED"));
        assert!(!is_lei_annulled(&id), "non-lei scheme must return false");
    }
}
