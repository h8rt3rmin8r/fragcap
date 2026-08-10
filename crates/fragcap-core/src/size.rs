// SPDX-License-Identifier: Apache-2.0

//! Size literals, as an operator writes them.
//!
//! The size counterpart to [`crate::duration`], and deliberately its mirror.
//! Slice S14 needs `--max-bytes` and the size form of the ring window (S16) to
//! parse a byte count, and the same reasoning that keeps the duration grammar
//! in `fragcap-core` keeps this one here: two consumers reach one grammar, and
//! two implementations of `4mb` that disagree would size a capture bound
//! differently in the profile and on the command line, which is a defect an
//! operator cannot see in the output.
//!
//! The grammar is one unsigned integer and one required unit, binary
//! (1024-based). A missing unit is refused rather than defaulted to bytes: the
//! duration grammar set the required-unit precedent for the same reason, that a
//! guessed unit is a guess about how large a bound the operator meant.
//!
//! Binary units, not decimal, because a capture bound is reasoned about beside
//! buffer sizes, which are powers of two. See slice S14 research D-h.
//!
//! No dependency is involved. `cargo xtask deps` asserts that this module did
//! not bring one to `fragcap-core`, which constitution P-2 is about.

use std::fmt;

/// Why a size literal was refused.
///
/// One variant per reason rather than a single message, so a caller can tell a
/// missing unit from an unknown one and say something useful about it. The
/// command line turns each of these into a usage message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SizeError {
    /// The literal was empty or contained only whitespace.
    Empty,
    /// Digits were present but no unit followed them.
    ///
    /// Refused rather than defaulted to bytes. A guessed unit is a guess about
    /// how large a bound the operator meant.
    MissingUnit,
    /// A unit was present but is not one this grammar accepts.
    UnknownUnit(String),
    /// The value was not a run of decimal digits followed by a unit: a sign, a
    /// fractional part, internal whitespace, a digit separator, or anything
    /// else that would have to be interpreted rather than read.
    Malformed,
    /// The value parsed but does not fit a `u64` of bytes.
    ///
    /// Refused rather than saturated, because a saturating parse turns a typo
    /// into a bound no capture ever reaches.
    Overflow,
    /// The literal names a zero size.
    ///
    /// Refused for the reason the duration grammar rejects a zero duration: the
    /// only possible meaning is a mistake, and honoring it would produce a bound
    /// reached before the first packet and an empty capture.
    Zero,
}

impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeError::Empty => write!(f, "empty size"),
            SizeError::MissingUnit => write!(
                f,
                "size has no unit; expected one of b, kb, mb, gb (for example 4mb)"
            ),
            SizeError::UnknownUnit(u) => {
                write!(f, "unknown size unit `{u}`; expected b, kb, mb, or gb")
            }
            SizeError::Malformed => write!(
                f,
                "malformed size; expected digits then a unit (for example 4mb)"
            ),
            SizeError::Overflow => write!(f, "size is too large to represent"),
            SizeError::Zero => write!(f, "size is zero"),
        }
    }
}

impl std::error::Error for SizeError {}

/// Parse a size literal into a count of bytes.
///
/// The accepted form is one unsigned decimal integer followed immediately by
/// one unit from `b`, `kb`, `mb`, or `gb`, interpreted as binary (1024-based).
/// `512b`, `64kb`, `4mb`, and `2gb` are accepted. Everything else is refused,
/// including a bare integer, a sign, a fraction, internal whitespace, a digit
/// separator, and zero.
///
/// Surrounding whitespace is not accepted either, matching the duration
/// grammar.
///
/// # Errors
///
/// Returns the [`SizeError`] naming the reason. Every refusal is one of them;
/// there is no catch-all.
pub fn parse(literal: &str) -> Result<u64, SizeError> {
    if literal.is_empty() {
        return Err(SizeError::Empty);
    }

    let digits_end = literal
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(literal.len());
    let (digits, unit) = literal.split_at(digits_end);

    if digits.is_empty() {
        // Nothing numeric at all. A leading sign, a leading dot, or a bare
        // unit all land here, and none of them is a size.
        return Err(SizeError::Malformed);
    }
    if unit.is_empty() {
        return Err(SizeError::MissingUnit);
    }

    // The unit runs to the end of the literal. Anything after it is a form this
    // grammar does not have.
    let bytes_per_unit: u64 = match unit {
        "b" => 1,
        "kb" => 1024,
        "mb" => 1024 * 1024,
        "gb" => 1024 * 1024 * 1024,
        _ if unit.chars().all(|c| c.is_ascii_alphabetic()) => {
            return Err(SizeError::UnknownUnit(unit.to_string()))
        }
        _ => return Err(SizeError::Malformed),
    };

    let value: u64 = digits.parse().map_err(|_| SizeError::Overflow)?;
    let bytes = value
        .checked_mul(bytes_per_unit)
        .ok_or(SizeError::Overflow)?;

    if bytes == 0 {
        return Err(SizeError::Zero);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_integer_and_one_unit() {
        let cases: &[(&str, u64)] = &[
            ("1b", 1),
            ("512b", 512),
            ("1kb", 1024),
            ("64kb", 64 * 1024),
            ("4mb", 4 * 1024 * 1024),
            ("2gb", 2 * 1024 * 1024 * 1024),
            // Leading zeros are digits, not a numeric format to interpret.
            ("064kb", 64 * 1024),
        ];
        for (literal, expected) in cases {
            assert_eq!(parse(literal), Ok(*expected), "literal {literal}");
        }
    }

    #[test]
    fn refuses_a_bare_integer_rather_than_guessing_bytes() {
        assert_eq!(parse("4096"), Err(SizeError::MissingUnit));
        assert_eq!(parse("0"), Err(SizeError::MissingUnit));
    }

    #[test]
    fn refuses_zero_in_every_unit() {
        for literal in ["0b", "0kb", "0mb", "0gb", "000mb"] {
            assert_eq!(parse(literal), Err(SizeError::Zero), "literal {literal}");
        }
    }

    #[test]
    fn refuses_an_unknown_unit_and_names_it() {
        assert_eq!(parse("4tb"), Err(SizeError::UnknownUnit("tb".to_string())));
        assert_eq!(parse("4k"), Err(SizeError::UnknownUnit("k".to_string())));
        assert_eq!(
            parse("4KB"),
            Err(SizeError::UnknownUnit("KB".to_string())),
            "units are lowercase; an uppercase one is unknown rather than equivalent"
        );
    }

    #[test]
    fn refuses_every_form_that_would_have_to_be_interpreted() {
        let cases: &[(&str, SizeError)] = &[
            ("", SizeError::Empty),
            ("kb", SizeError::Malformed),
            ("-5mb", SizeError::Malformed),
            ("+5mb", SizeError::Malformed),
            (".5gb", SizeError::Malformed),
            ("1.5gb", SizeError::Malformed),
            ("1_000kb", SizeError::Malformed),
            ("4 mb", SizeError::Malformed),
            (" 4mb", SizeError::Malformed),
            ("4mb ", SizeError::Malformed),
            ("4mb4", SizeError::Malformed),
        ];
        for (literal, expected) in cases {
            assert_eq!(parse(literal), Err(expected.clone()), "literal {literal:?}");
        }
    }

    #[test]
    fn refuses_overflow_rather_than_saturating() {
        // Beyond u64 in the digits themselves.
        assert_eq!(parse("99999999999999999999999b"), Err(SizeError::Overflow));
        // Fits u64 as a count of gibibytes, does not fit as bytes.
        assert_eq!(parse("17179869184gb"), Err(SizeError::Overflow));
    }

    #[test]
    fn the_largest_representable_value_is_accepted() {
        // One below the point where the multiply overflows, so the boundary is
        // asserted from both sides rather than only from the failing one.
        let literal = format!("{}b", u64::MAX);
        assert_eq!(
            parse(&literal),
            Ok(u64::MAX),
            "the overflow check must not reject a value that fits"
        );
    }
}
