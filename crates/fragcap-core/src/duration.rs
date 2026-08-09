// SPDX-License-Identifier: Apache-2.0

//! Duration literals, as an operator writes them.
//!
//! Specification section 25.2 lists duration parsing as a tier 0 concern
//! without saying which crate owns it, and three consumers are visible:
//! `capture.duration` in a game profile (slice S05), `--duration` and `--wait`
//! on the command line (S14), and the ring window (S16). This module is here so
//! that all three reach one grammar. Section 8.3 forbids a crate below the
//! facade depending on a sibling, so the alternative was two implementations of
//! `30m`, and two implementations that disagree produce a capture of the wrong
//! length, which is a defect an operator cannot see in the output.
//!
//! The grammar is deliberately narrow: one unsigned integer and one required
//! unit. Widening an accepted syntax later keeps every profile written today
//! valid, while narrowing one does not, so the conservative direction is the
//! reversible one. See slice S05 research R-6.
//!
//! No dependency is involved. `cargo xtask deps` asserts that this module did
//! not bring one to `fragcap-core`, which constitution P-2 is about.

use std::fmt;
use std::time::Duration;

/// Why a duration literal was refused.
///
/// One variant per reason rather than a single message, so a caller can tell a
/// missing unit from an unknown one and say something useful about it. The
/// profile schema turns each of these into a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DurationError {
    /// The literal was empty or contained only whitespace.
    Empty,
    /// Digits were present but no unit followed them.
    ///
    /// Refused rather than defaulted. A guessed unit is a guess about how much
    /// of a session the operator loses.
    MissingUnit,
    /// A unit was present but is not one this grammar accepts.
    UnknownUnit(String),
    /// The value was not a run of decimal digits followed by a unit: a sign, a
    /// fractional part, internal whitespace, a digit separator, or anything
    /// else that would have to be interpreted rather than read.
    Malformed,
    /// The value parsed but does not fit the duration representation.
    ///
    /// Refused rather than saturated, because a saturating parse turns a typo
    /// into a capture that runs for a hundred years.
    Overflow,
    /// The literal names a zero duration.
    ///
    /// Refused for the reason slice S08 rejected a zero-capacity buffer: the
    /// only possible meaning is a mistake, and honoring it would produce an
    /// empty capture and a successful exit.
    Zero,
}

impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationError::Empty => write!(f, "empty duration"),
            DurationError::MissingUnit => write!(
                f,
                "duration has no unit; expected one of ms, s, m, h (for example 30m)"
            ),
            DurationError::UnknownUnit(u) => {
                write!(f, "unknown duration unit `{u}`; expected ms, s, m, or h")
            }
            DurationError::Malformed => write!(
                f,
                "malformed duration; expected digits then a unit (for example 30m)"
            ),
            DurationError::Overflow => write!(f, "duration is too large to represent"),
            DurationError::Zero => write!(f, "duration is zero"),
        }
    }
}

impl std::error::Error for DurationError {}

/// Parse a duration literal.
///
/// The accepted form is one unsigned decimal integer followed immediately by
/// one unit from `ms`, `s`, `m`, or `h`. `500ms`, `30s`, `30m`, and `2h` are
/// accepted. Everything else is refused, including a bare integer, a sign, a
/// fraction, internal whitespace, a digit separator, a compound form such as
/// `1h30m`, and zero.
///
/// Surrounding whitespace is not accepted either. A profile's value comes from
/// a TOML string, where whitespace inside the quotes is something the author
/// typed, and trimming it would be deciding what they meant.
///
/// # Errors
///
/// Returns the [`DurationError`] naming the reason. Every refusal is one of
/// them; there is no catch-all.
pub fn parse(literal: &str) -> Result<Duration, DurationError> {
    if literal.is_empty() {
        return Err(DurationError::Empty);
    }

    let digits_end = literal
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(literal.len());
    let (digits, unit) = literal.split_at(digits_end);

    if digits.is_empty() {
        // Nothing numeric at all. A leading sign, a leading dot, or a bare
        // unit all land here, and none of them is a duration.
        return Err(DurationError::Malformed);
    }
    if unit.is_empty() {
        return Err(DurationError::MissingUnit);
    }

    // The unit runs to the end of the literal. Anything after it, including a
    // second component, is a form this grammar does not have.
    let millis_per_unit: u64 = match unit {
        "ms" => 1,
        "s" => 1_000,
        "m" => 60 * 1_000,
        "h" => 60 * 60 * 1_000,
        _ if unit.chars().all(|c| c.is_ascii_alphabetic()) => {
            return Err(DurationError::UnknownUnit(unit.to_string()))
        }
        _ => return Err(DurationError::Malformed),
    };

    let value: u64 = digits.parse().map_err(|_| DurationError::Overflow)?;
    let millis = value
        .checked_mul(millis_per_unit)
        .ok_or(DurationError::Overflow)?;

    if millis == 0 {
        return Err(DurationError::Zero);
    }

    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_integer_and_one_unit() {
        let cases: &[(&str, Duration)] = &[
            ("1ms", Duration::from_millis(1)),
            ("500ms", Duration::from_millis(500)),
            ("1s", Duration::from_secs(1)),
            ("30s", Duration::from_secs(30)),
            ("30m", Duration::from_secs(30 * 60)),
            ("2h", Duration::from_secs(2 * 60 * 60)),
            // The literal from specification section 15.2.
            ("30m", Duration::from_secs(1_800)),
            // Leading zeros are digits, not a numeric format to interpret.
            ("030s", Duration::from_secs(30)),
        ];
        for (literal, expected) in cases {
            assert_eq!(parse(literal), Ok(*expected), "literal {literal}");
        }
    }

    #[test]
    fn refuses_a_bare_integer_rather_than_guessing_a_unit() {
        assert_eq!(parse("30"), Err(DurationError::MissingUnit));
        assert_eq!(parse("0"), Err(DurationError::MissingUnit));
    }

    #[test]
    fn refuses_zero_in_every_unit() {
        for literal in ["0ms", "0s", "0m", "0h", "000s"] {
            assert_eq!(
                parse(literal),
                Err(DurationError::Zero),
                "literal {literal}"
            );
        }
    }

    #[test]
    fn refuses_an_unknown_unit_and_names_it() {
        assert_eq!(
            parse("30d"),
            Err(DurationError::UnknownUnit("d".to_string()))
        );
        assert_eq!(
            parse("30us"),
            Err(DurationError::UnknownUnit("us".to_string()))
        );
        assert_eq!(
            parse("30S"),
            Err(DurationError::UnknownUnit("S".to_string())),
            "units are lowercase; an uppercase one is unknown rather than equivalent"
        );
    }

    #[test]
    fn refuses_every_form_that_would_have_to_be_interpreted() {
        let cases: &[(&str, DurationError)] = &[
            ("", DurationError::Empty),
            ("m", DurationError::Malformed),
            ("-5m", DurationError::Malformed),
            ("+5m", DurationError::Malformed),
            (".5h", DurationError::Malformed),
            ("1.5h", DurationError::Malformed),
            ("1_000s", DurationError::Malformed),
            ("30 m", DurationError::Malformed),
            (" 30m", DurationError::Malformed),
            ("30m ", DurationError::Malformed),
            ("1h30m", DurationError::Malformed),
            ("30m30", DurationError::Malformed),
        ];
        for (literal, expected) in cases {
            assert_eq!(parse(literal), Err(expected.clone()), "literal {literal:?}");
        }
    }

    #[test]
    fn refuses_overflow_rather_than_saturating() {
        // Beyond u64 in the digits themselves.
        assert_eq!(
            parse("99999999999999999999999s"),
            Err(DurationError::Overflow)
        );
        // Fits u64 as a count of hours, does not fit as milliseconds.
        assert_eq!(parse("18446744073709551h"), Err(DurationError::Overflow));
    }

    #[test]
    fn the_largest_representable_value_is_accepted() {
        // One below the point where the multiply overflows, so the boundary is
        // asserted from both sides rather than only from the failing one.
        let literal = format!("{}ms", u64::MAX);
        assert_eq!(
            parse(&literal),
            Ok(Duration::from_millis(u64::MAX)),
            "the overflow check must not reject a value that fits"
        );
    }
}
