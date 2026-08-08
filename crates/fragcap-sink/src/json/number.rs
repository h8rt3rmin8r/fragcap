// SPDX-License-Identifier: Apache-2.0

//! Exact decimal timestamps, without floating point.
//!
//! Section 13.5 shows a timestamp as a JSON number with six fractional digits.
//! The obvious way to produce one is to divide into an `f64` and print it.
//!
//! That path is wrong, and not in the distant or exotic way one might assume.
//! It was measured. For a whole-microsecond timestamp in the present era it
//! agrees with this one, so a casual test would not catch it. But a capture
//! driver reports nanoseconds, and for any sub-microsecond remainder of 500 ns
//! or more the two disagree by a microsecond: this floors, as the declared
//! resolution and the pcapng writer do, while dividing into an `f64` and
//! printing to six places rounds. That is one packet described two ways by the
//! two output formats of the same capture, today, on ordinary input.
//!
//! Precision is the secondary problem: an `f64` holds microseconds-since-epoch
//! exactly only to a 53-bit significand. Integer arithmetic needs neither
//! argument, costs the same four lines, and can be checked by reading them.
//! The JSON stream exists for correlation against other logs, where a wrong
//! microsecond is worse than a missing one because it looks right.

use crate::error::WriteError;

/// Nanoseconds in one microsecond.
const NANOS_PER_MICRO: i64 = 1_000;

/// Render a nanosecond timestamp as seconds with exactly six fractional digits.
///
/// Floors toward negative infinity, consistently with the pcapng writer, so the
/// two outputs order a pair of timestamps the same way.
///
/// Refuses a pre-epoch value rather than emitting a negative number. The
/// pcapng writer refuses the same input because the format cannot represent it;
/// JSON could represent it, and refusing anyway keeps one capture from being
/// describable in one output and not the other.
pub(crate) fn render_timestamp(nanos: i64) -> Result<String, WriteError> {
    if nanos < 0 {
        return Err(WriteError::TimestampBeforeEpoch { nanos });
    }
    let micros = nanos / NANOS_PER_MICRO;
    let secs = micros / 1_000_000;
    let frac = micros % 1_000_000;
    // Six digits always, so a whole second is not narrower than a fractional
    // one and a consumer reading fixed-width text is not surprised.
    Ok(format!("{secs}.{frac:06}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_specification_example_renders_exactly() {
        // Section 13.5 prints this number. If this fails, the documentation is
        // wrong or the formatter is.
        let nanos = 1_754_500_000_123_456_000;
        assert_eq!(render_timestamp(nanos).unwrap(), "1754500000.123456");
    }

    #[test]
    fn a_whole_second_still_carries_six_digits() {
        assert_eq!(
            render_timestamp(1_700_000_000 * 1_000_000_000).unwrap(),
            "1700000000.000000"
        );
    }

    #[test]
    fn the_epoch_renders_as_zero() {
        assert_eq!(render_timestamp(0).unwrap(), "0.000000");
    }

    #[test]
    fn a_leading_zero_fraction_is_padded_not_truncated() {
        // 1 microsecond past the second. Formatting the remainder without
        // padding would render this as `1.1`, which is 100 milliseconds.
        assert_eq!(
            render_timestamp(1_000_000_000 + NANOS_PER_MICRO).unwrap(),
            "1.000001"
        );
    }

    #[test]
    fn sub_microsecond_components_floor() {
        assert_eq!(render_timestamp(999).unwrap(), "0.000000");
        assert_eq!(render_timestamp(1_999).unwrap(), "0.000001");
    }

    #[test]
    fn the_conversion_preserves_ordering() {
        let mut prev = String::new();
        for n in [0i64, 1, 999, 1_000, 1_001, 1_000_000, 1_000_000_000] {
            let s = render_timestamp(n).unwrap();
            assert!(s >= prev, "flooring must not reorder observations");
            prev = s;
        }
    }

    /// The regression guard.
    ///
    /// The divergence is not exotic and not far in the future. A capture
    /// driver reports nanoseconds, and any timestamp with a sub-microsecond
    /// remainder of 500 ns or more is rendered differently by the two paths:
    /// this one floors, as the declared resolution and the pcapng writer do,
    /// while dividing into an `f64` and printing to six places rounds. One
    /// microsecond of disagreement between the two output formats for the same
    /// packet, today, on ordinary input.
    #[test]
    fn the_integer_path_floors_where_a_float_path_would_round() {
        let nanos = 1_754_500_000_123_456_789;
        let exact = render_timestamp(nanos).unwrap();
        assert_eq!(
            exact, "1754500000.123456",
            "floored, per the declared resolution"
        );

        let via_float = format!("{:.6}", nanos as f64 / 1e9);
        assert_eq!(
            via_float, "1754500000.123457",
            "the rejected path rounds up"
        );
        assert_ne!(
            exact, via_float,
            "a float path disagrees with the pcapng writer about this packet"
        );
    }

    #[test]
    fn present_era_values_are_exact_across_the_whole_fractional_range() {
        for micros in [0u64, 1, 999_999, 500_000, 123_456] {
            let nanos = 1_754_500_000_000_000_000 + (micros as i64 * 1_000);
            assert_eq!(
                render_timestamp(nanos).unwrap(),
                format!("1754500000.{micros:06}")
            );
        }
    }

    #[test]
    fn a_pre_epoch_timestamp_is_refused() {
        // Consistent with the pcapng writer, so one capture cannot be
        // describable in one output format and not the other.
        assert_eq!(
            render_timestamp(-1),
            Err(WriteError::TimestampBeforeEpoch { nanos: -1 })
        );
    }

    #[test]
    fn large_timestamps_stay_exact() {
        // Year 2100. Still inside the range where a float would also be right;
        // the magnitude where it is not is covered above.
        let nanos = 4_102_444_800_654_321_000;
        assert_eq!(render_timestamp(nanos).unwrap(), "4102444800.654321");
    }
}
