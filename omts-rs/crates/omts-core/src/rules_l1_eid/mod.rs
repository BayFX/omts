/// L1-EID validation rules for entity identification (SPEC-002 Section 6.1).
///
/// This module implements rules L1-EID-01 through L1-EID-11 as specified in
/// validation.md Section 4.1 and SPEC-002 Section 6.1.  Each rule is a
/// zero-sized struct that implements [`ValidationRule`].
///
/// Rules are registered in [`crate::validation::build_registry`] when
/// `config.run_l1` is `true`.
use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::check_digits::{gs1_mod10, mod97_10};
use crate::file::OmtsFile;
use crate::validation::{Diagnostic, Level, Location, RuleId, Severity, ValidationRule};

/// LEI format: 18 uppercase alphanumeric characters followed by 2 digits.
static LEI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Z0-9]{18}[0-9]{2}$")
        .unwrap_or_else(|_| Regex::new(".").unwrap_or_else(|_| unreachable!("regex engine broken")))
});

/// DUNS format: exactly 9 digits.
static DUNS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9]{9}$")
        .unwrap_or_else(|_| Regex::new(".").unwrap_or_else(|_| unreachable!("regex engine broken")))
});

/// GLN format: exactly 13 digits.
static GLN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9]{13}$")
        .unwrap_or_else(|_| Regex::new(".").unwrap_or_else(|_| unreachable!("regex engine broken")))
});

/// Core scheme codes defined by SPEC-002 Section 5.1, plus the reserved
/// `opaque` scheme used exclusively by `boundary_ref` nodes (SPEC-004 Section 5.1).
static CORE_SCHEMES: &[&str] = &["lei", "duns", "gln", "nat-reg", "vat", "internal", "opaque"];

/// Returns `true` if the scheme string is a valid core scheme or a
/// reverse-domain extension scheme (contains a dot, e.g. `"com.example.id"`).
fn is_valid_scheme(scheme: &str) -> bool {
    CORE_SCHEMES.contains(&scheme) || scheme.contains('.')
}

/// Returns `true` if the `authority` field is required for the given scheme.
fn requires_authority(scheme: &str) -> bool {
    matches!(scheme, "nat-reg" | "vat" | "internal")
}

fn eid_diag(
    rule_id: RuleId,
    node_id: &str,
    index: usize,
    field: Option<&'static str>,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        rule_id,
        Severity::Error,
        Location::Identifier {
            node_id: node_id.to_owned(),
            index,
            field: field.map(ToOwned::to_owned),
        },
        message,
    )
}

/// Every identifier record MUST have a non-empty `scheme` field (SPEC-002 L1-EID-01).
pub struct L1Eid01;

impl ValidationRule for L1Eid01 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid01
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if ident.scheme.is_empty() {
                    diags.push(eid_diag(
                        RuleId::L1Eid01,
                        node.id.as_ref(),
                        idx,
                        Some("scheme"),
                        "identifier `scheme` must not be empty",
                    ));
                }
            }
        }
    }
}

/// Every identifier record MUST have a non-empty `value` field (SPEC-002 L1-EID-02).
pub struct L1Eid02;

impl ValidationRule for L1Eid02 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid02
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if ident.value.is_empty() {
                    diags.push(eid_diag(
                        RuleId::L1Eid02,
                        node.id.as_ref(),
                        idx,
                        Some("value"),
                        "identifier `value` must not be empty",
                    ));
                }
            }
        }
    }
}

/// For schemes requiring `authority` (`nat-reg`, `vat`, `internal`), the
/// `authority` field MUST be present and non-empty (SPEC-002 L1-EID-03).
pub struct L1Eid03;

impl ValidationRule for L1Eid03 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid03
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if requires_authority(&ident.scheme) {
                    let missing = match &ident.authority {
                        None => true,
                        Some(auth) => auth.is_empty(),
                    };
                    if missing {
                        diags.push(eid_diag(
                            RuleId::L1Eid03,
                            node.id.as_ref(),
                            idx,
                            Some("authority"),
                            format!(
                                "scheme `{}` requires a non-empty `authority` field",
                                ident.scheme
                            ),
                        ));
                    }
                }
            }
        }
    }
}

/// `scheme` MUST be either a core scheme code or a valid extension scheme
/// code (reverse-domain notation) (SPEC-002 L1-EID-04).
///
/// Unknown extension schemes (containing a dot) are permitted.  Pure unknown
/// strings without a dot that are not core schemes are rejected.
pub struct L1Eid04;

impl ValidationRule for L1Eid04 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid04
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if ident.scheme.is_empty() {
                    continue;
                }
                if !is_valid_scheme(&ident.scheme) {
                    diags.push(eid_diag(
                        RuleId::L1Eid04,
                        node.id.as_ref(),
                        idx,
                        Some("scheme"),
                        format!(
                            "scheme `{}` is not a recognised core scheme or reverse-domain extension",
                            ident.scheme
                        ),
                    ));
                }
            }
        }
    }
}

/// For `lei` scheme: `value` MUST match `^[A-Z0-9]{18}[0-9]{2}$` and MUST
/// pass MOD 97-10 check digit verification (SPEC-002 L1-EID-05).
pub struct L1Eid05;

impl ValidationRule for L1Eid05 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid05
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if ident.scheme != "lei" {
                    continue;
                }
                if !LEI_RE.is_match(&ident.value) {
                    diags.push(eid_diag(
                        RuleId::L1Eid05,
                        node.id.as_ref(),
                        idx,
                        Some("value"),
                        format!(
                            "LEI `{}` does not match `^[A-Z0-9]{{18}}[0-9]{{2}}$`",
                            ident.value
                        ),
                    ));
                } else if !mod97_10(&ident.value) {
                    diags.push(eid_diag(
                        RuleId::L1Eid05,
                        node.id.as_ref(),
                        idx,
                        Some("value"),
                        format!(
                            "LEI `{}` fails MOD 97-10 check digit verification",
                            ident.value
                        ),
                    ));
                }
            }
        }
    }
}

/// For `duns` scheme: `value` MUST match `^[0-9]{9}$` (SPEC-002 L1-EID-06).
pub struct L1Eid06;

impl ValidationRule for L1Eid06 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid06
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if ident.scheme != "duns" {
                    continue;
                }
                if !DUNS_RE.is_match(&ident.value) {
                    diags.push(eid_diag(
                        RuleId::L1Eid06,
                        node.id.as_ref(),
                        idx,
                        Some("value"),
                        format!("DUNS `{}` does not match `^[0-9]{{9}}$`", ident.value),
                    ));
                }
            }
        }
    }
}

/// For `gln` scheme: `value` MUST match `^[0-9]{13}$` and MUST pass GS1
/// mod-10 check digit verification (SPEC-002 L1-EID-07).
pub struct L1Eid07;

impl ValidationRule for L1Eid07 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid07
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if ident.scheme != "gln" {
                    continue;
                }
                if !GLN_RE.is_match(&ident.value) {
                    diags.push(eid_diag(
                        RuleId::L1Eid07,
                        node.id.as_ref(),
                        idx,
                        Some("value"),
                        format!("GLN `{}` does not match `^[0-9]{{13}}$`", ident.value),
                    ));
                } else if !gs1_mod10(&ident.value) {
                    diags.push(eid_diag(
                        RuleId::L1Eid07,
                        node.id.as_ref(),
                        idx,
                        Some("value"),
                        format!(
                            "GLN `{}` fails GS1 mod-10 check digit verification",
                            ident.value
                        ),
                    ));
                }
            }
        }
    }
}

/// `valid_from` and `valid_to`, if present, MUST be valid ISO 8601 date
/// strings in `YYYY-MM-DD` format (SPEC-002 L1-EID-08).
///
/// Note: [`crate::newtypes::CalendarDate`] already enforces the `YYYY-MM-DD`
/// shape at deserialization time, so any [`crate::newtypes::CalendarDate`] value in a parsed
/// [`crate::types::Identifier`] is guaranteed to have the correct format.
/// This rule therefore checks the *semantic* calendar validity (e.g. month
/// must be 01–12, day must be within the month's range).
pub struct L1Eid08;

/// Returns `true` if the string `s` (already known to match `YYYY-MM-DD`)
/// represents a semantically valid calendar date.
pub(crate) fn is_calendar_date_valid(s: &str) -> bool {
    // CalendarDate guarantees the regex YYYY-MM-DD matched, so we can index directly.
    let bytes = s.as_bytes();
    let year = parse_u32_fixed(&bytes[0..4]);
    let month = parse_u32_fixed(&bytes[5..7]);
    let day = parse_u32_fixed(&bytes[8..10]);

    if !(1..=12).contains(&month) {
        return false;
    }
    let max_day = days_in_month(year, month);
    day >= 1 && day <= max_day
}

/// Parses a fixed-width ASCII decimal slice into a `u32`.
fn parse_u32_fixed(bytes: &[u8]) -> u32 {
    let mut n: u32 = 0;
    for &b in bytes {
        n = n * 10 + u32::from(b - b'0');
    }
    n
}

/// Returns the number of days in a given month of a given year.
pub(crate) fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Returns `true` if `year` is a Gregorian leap year.
pub(crate) fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

impl ValidationRule for L1Eid08 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid08
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                if let Some(vf) = &ident.valid_from {
                    if !is_calendar_date_valid(vf.as_ref()) {
                        diags.push(eid_diag(
                            RuleId::L1Eid08,
                            node.id.as_ref(),
                            idx,
                            Some("valid_from"),
                            format!("`valid_from` `{vf}` is not a valid ISO 8601 date"),
                        ));
                    }
                }
                if let Some(Some(vt)) = &ident.valid_to {
                    if !is_calendar_date_valid(vt.as_ref()) {
                        diags.push(eid_diag(
                            RuleId::L1Eid08,
                            node.id.as_ref(),
                            idx,
                            Some("valid_to"),
                            format!("`valid_to` `{vt}` is not a valid ISO 8601 date"),
                        ));
                    }
                }
            }
        }
    }
}

/// If both `valid_from` and `valid_to` are present, `valid_from` MUST be
/// less than or equal to `valid_to` (SPEC-002 L1-EID-09).
///
/// Since [`crate::newtypes::CalendarDate`] derives `PartialOrd` / `Ord` on
/// its inner `String`, and ISO 8601 `YYYY-MM-DD` strings sort lexicographically
/// the same as chronologically, `a <= b` on `CalendarDate` gives the correct
/// temporal ordering.
pub struct L1Eid09;

impl ValidationRule for L1Eid09 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid09
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };
            for (idx, ident) in identifiers.iter().enumerate() {
                let Some(vf) = &ident.valid_from else {
                    continue;
                };
                // valid_to must be Some(Some(date)) — if it's None or Some(None) we skip.
                let Some(Some(vt)) = &ident.valid_to else {
                    continue;
                };
                if vf > vt {
                    diags.push(eid_diag(
                        RuleId::L1Eid09,
                        node.id.as_ref(),
                        idx,
                        None,
                        format!("`valid_from` `{vf}` is after `valid_to` `{vt}`"),
                    ));
                }
            }
        }
    }
}

/// `sensitivity`, if present, MUST be one of `public`, `restricted`, or
/// `confidential` (SPEC-002 L1-EID-10).
///
/// Since [`crate::enums::Sensitivity`] is the concrete type for the
/// `sensitivity` field and serde rejects unknown variants at deserialization
/// time, any `Identifier` that was successfully parsed already has a valid
/// `Sensitivity` value.  This rule is therefore always satisfied for
/// deserialized data — it is included to satisfy the spec's requirement and
/// to cover identifiers constructed programmatically after the fact if the
/// type system is ever relaxed.
///
/// In practice, serde already enforces this invariant for JSON input.
pub struct L1Eid10;

impl ValidationRule for L1Eid10 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid10
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        // The `Sensitivity` enum is exhaustively validated by serde at
        // deserialization; any parsed `Identifier` with a non-None sensitivity
        // field already holds a valid variant.  No additional runtime check is
        // needed for deserialized data.
        //
        // This rule is a no-op for correctly typed data.  It is registered in
        // the validator to document that the rule is covered and to provide a
        // hook if the type definition changes in the future.
        let _ = file;
        let _ = diags;
    }
}

/// No two identifier records on the same node may have identical `scheme`,
/// `value`, and `authority` (SPEC-002 L1-EID-11).
pub struct L1Eid11;

impl ValidationRule for L1Eid11 {
    fn id(&self) -> RuleId {
        RuleId::L1Eid11
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn crate::validation::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let Some(identifiers) = &node.identifiers else {
                continue;
            };

            let mut seen: HashSet<(&str, &str, Option<&str>)> = HashSet::new();

            for (idx, ident) in identifiers.iter().enumerate() {
                let key = (
                    ident.scheme.as_str(),
                    ident.value.as_str(),
                    ident.authority.as_deref(),
                );
                if !seen.insert(key) {
                    diags.push(eid_diag(
                        RuleId::L1Eid11,
                        node.id.as_ref(),
                        idx,
                        None,
                        format!(
                            "duplicate identifier tuple (scheme=`{}`, value=`{}`, authority={:?})",
                            ident.scheme, ident.value, ident.authority
                        ),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
