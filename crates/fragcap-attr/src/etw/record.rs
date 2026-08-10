// SPDX-License-Identifier: Apache-2.0

//! The kernel process event layout, and the timestamp conversion.
//!
//! This module is the one place in the slice where a wrong number produces
//! plausible values rather than an error, so it is deliberately dull. Every
//! offset is arithmetic over a byte slice with a bounds check, nothing is
//! transmuted, and a record that does not parse yields `None` rather than a
//! guess.
//!
//! The kernel process provider is a MOF-based provider, not a manifest-based
//! one, so its events are fixed structures rather than a property list. The
//! layout below is the `Process_V3_TypeGroup1` and `Process_V4_TypeGroup1`
//! classes:
//!
//! ```text
//! UniqueProcessKey      pointer-sized
//! ProcessId             u32
//! ParentId              u32
//! SessionId             u32
//! ExitStatus            i32
//! DirectoryTableBase    pointer-sized
//! Flags                 u32            (version 4 only)
//! UserSID               variable, see `sid_len`
//! ImageFileName         null-terminated ANSI
//! CommandLine           null-terminated UTF-16
//! ```
//!
//! Version 4 adds two more UTF-16 strings after the command line that fragcap
//! does not read.

/// Nanoseconds between the Windows epoch (1601-01-01 UTC) and the Unix epoch
/// (1970-01-01 UTC), expressed in the 100 nanosecond units `FILETIME` counts.
///
/// Spelled out rather than written as a literal, because a magic number here is
/// the kind of wrong that yields timestamps which look reasonable and are
/// decades off, and no test over synthetic data catches it.
const FILETIME_TO_UNIX_EPOCH_100NS: i64 = 116_444_736_000_000_000;

/// Convert a `FILETIME`-shaped value into nanoseconds since the Unix epoch.
///
/// The session sets its client context to system time precisely so that the
/// event header timestamp is this and not a performance counter. A performance
/// counter is monotonic and has no relationship to the wall clock, and
/// therefore none to the timestamps a capture driver puts on packets, which
/// would make process events unplaceable against the traffic they explain.
pub fn filetime_to_unix_nanos(filetime: i64) -> i64 {
    filetime
        .saturating_sub(FILETIME_TO_UNIX_EPOCH_100NS)
        .saturating_mul(100)
}

/// What one process start event carried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartFields {
    pub pid: u32,
    pub parent: u32,
    pub image: String,
    pub command_line: Option<String>,
}

/// The pointer width of the process that produced the trace.
///
/// A 32-bit process on a 64-bit system emits 4-byte pointers into the same
/// stream, and the record layout has two pointer-sized fields before anything
/// fragcap reads. The header supplies the width; this is not guessed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerWidth {
    Four,
    Eight,
}

impl PointerWidth {
    fn bytes(self) -> usize {
        match self {
            PointerWidth::Four => 4,
            PointerWidth::Eight => 8,
        }
    }
}

/// The length of the embedded user security identifier, in bytes.
///
/// The MOF encoding is awkward and is the reason this parse is written out
/// rather than expressed as a struct. When the first pointer-sized value is
/// zero there is no identifier and the field is that one value. Otherwise the
/// field is a `TOKEN_USER`, which is two pointers, followed by the identifier
/// itself: a revision byte, a sub-authority count byte, six bytes of authority,
/// and four bytes per sub-authority.
fn sid_len(buf: &[u8], at: usize, width: PointerWidth) -> Option<usize> {
    let p = width.bytes();
    let head = buf.get(at..at + p)?;
    let is_null = head.iter().all(|b| *b == 0);
    if is_null {
        return Some(p);
    }
    let sid_at = at + 2 * p;
    let sub_authorities = *buf.get(sid_at + 1)? as usize;
    Some(2 * p + 8 + 4 * sub_authorities)
}

/// Read a null-terminated ANSI string, returning it and the bytes consumed.
fn ansi_at(buf: &[u8], at: usize) -> Option<(String, usize)> {
    let rest = buf.get(at..)?;
    let end = rest.iter().position(|b| *b == 0)?;
    let s = String::from_utf8_lossy(&rest[..end]).into_owned();
    Some((s, end + 1))
}

/// Read a null-terminated UTF-16 string, returning it and the bytes consumed.
///
/// Lossy on an unpaired surrogate rather than refusing. A command line that
/// Windows accepted is a command line fragcap has to record, and refusing it
/// would discard an observation over an encoding detail, which P-9 does not
/// permit. The lossy form is what the platform itself displays.
fn wide_at(buf: &[u8], at: usize) -> Option<(String, usize)> {
    let rest = buf.get(at..)?;
    let mut units = Vec::new();
    let mut i = 0;
    loop {
        let pair = rest.get(i..i + 2)?;
        let u = u16::from_le_bytes([pair[0], pair[1]]);
        i += 2;
        if u == 0 {
            break;
        }
        units.push(u);
    }
    Some((String::from_utf16_lossy(&units), i))
}

/// Parse a process start event.
///
/// `version` is the event descriptor's version, which selects whether the
/// `Flags` field is present. Anything that does not parse yields `None`, and
/// the caller counts it rather than inventing a process.
pub fn parse_start(buf: &[u8], version: u8, width: PointerWidth) -> Option<StartFields> {
    let p = width.bytes();

    let mut at = p; // UniqueProcessKey
    let pid = u32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?);
    at += 4;
    let parent = u32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?);
    at += 4;
    at += 4; // SessionId
    at += 4; // ExitStatus
    at += p; // DirectoryTableBase
    if version >= 4 {
        at += 4; // Flags
    }

    at += sid_len(buf, at, width)?;

    let (image, used) = ansi_at(buf, at)?;
    at += used;

    // A version 2 record has no command line, and a truncated one has no
    // readable command line. Both are recorded as absent rather than failing
    // the whole parse, and the asymmetry with the fields above is deliberate:
    // failing here would discard a process, which costs its entire subtree,
    // to avoid losing one field that the type already permits to be absent.
    // The fields above have no such fallback, so a record too short to hold
    // them yields nothing at all.
    let command_line = if version >= 3 {
        wide_at(buf, at).map(|(s, _)| s)
    } else {
        None
    };

    Some(StartFields {
        pid,
        parent,
        image,
        command_line,
    })
}

/// Parse a process end event, which needs only the identifier.
pub fn parse_end(buf: &[u8], width: PointerWidth) -> Option<u32> {
    let at = width.bytes();
    Some(u32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a version 4 record the way the kernel lays one out, so that the
    /// parse is checked against a known-good encoding rather than against
    /// itself.
    fn v4_record(pid: u32, parent: u32, image: &str, cmdline: &str, sid_subs: u8) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&[0u8; 8]); // UniqueProcessKey
        b.extend_from_slice(&pid.to_le_bytes());
        b.extend_from_slice(&parent.to_le_bytes());
        b.extend_from_slice(&7u32.to_le_bytes()); // SessionId
        b.extend_from_slice(&0i32.to_le_bytes()); // ExitStatus
        b.extend_from_slice(&[0u8; 8]); // DirectoryTableBase
        b.extend_from_slice(&0u32.to_le_bytes()); // Flags, version 4 only

        // TOKEN_USER: two non-null pointers, then the identifier.
        b.extend_from_slice(&[1u8; 8]);
        b.extend_from_slice(&[1u8; 8]);
        b.push(1); // Revision
        b.push(sid_subs); // SubAuthorityCount
        b.extend_from_slice(&[0, 0, 0, 0, 0, 5]); // IdentifierAuthority
        for i in 0..sid_subs {
            b.extend_from_slice(&(i as u32).to_le_bytes());
        }

        b.extend_from_slice(image.as_bytes());
        b.push(0);
        for u in cmdline.encode_utf16() {
            b.extend_from_slice(&u.to_le_bytes());
        }
        b.extend_from_slice(&[0, 0]);
        b
    }

    #[test]
    fn the_epoch_offset_places_a_known_instant_correctly() {
        // 1970-01-01T00:00:00Z is exactly the offset, and must land on zero.
        assert_eq!(filetime_to_unix_nanos(FILETIME_TO_UNIX_EPOCH_100NS), 0);
        // One second later.
        assert_eq!(
            filetime_to_unix_nanos(FILETIME_TO_UNIX_EPOCH_100NS + 10_000_000),
            1_000_000_000
        );
        // And a time before the Unix epoch is negative rather than wrapping.
        assert!(filetime_to_unix_nanos(0) < 0);
    }

    #[test]
    fn the_epoch_offset_saturates_rather_than_wrapping() {
        // A wrapped timestamp would be a silently wrong observation, which is
        // the failure P-9 exists to prevent.
        assert_eq!(filetime_to_unix_nanos(i64::MAX), i64::MAX);
        assert_eq!(filetime_to_unix_nanos(i64::MIN), i64::MIN);
    }

    #[test]
    fn a_version_four_record_parses() {
        let rec = v4_record(4242, 1000, "eso64.exe", "eso64.exe -viewer_id 0", 5);
        let got = parse_start(&rec, 4, PointerWidth::Eight).unwrap();
        assert_eq!(got.pid, 4242);
        assert_eq!(got.parent, 1000);
        assert_eq!(got.image, "eso64.exe");
        assert_eq!(got.command_line.as_deref(), Some("eso64.exe -viewer_id 0"));
    }

    #[test]
    fn a_command_line_with_non_ascii_survives_the_parse() {
        let odd = "app.exe --path \"C:\\Users\\Ünïcødé\\Games\" --tag=日本語";
        let rec = v4_record(1, 2, "app.exe", odd, 5);
        let got = parse_start(&rec, 4, PointerWidth::Eight).unwrap();
        assert_eq!(got.command_line.as_deref(), Some(odd));
    }

    #[test]
    fn a_long_command_line_survives_the_parse() {
        let long = format!("app.exe {}", "-x ".repeat(20_000));
        let rec = v4_record(1, 2, "app.exe", &long, 5);
        let got = parse_start(&rec, 4, PointerWidth::Eight).unwrap();
        assert_eq!(got.command_line.as_deref(), Some(long.as_str()));
    }

    #[test]
    fn the_sub_authority_count_moves_the_strings() {
        // The identifier is variable length, and getting this wrong is exactly
        // the failure that yields a plausible but wrong image name.
        for subs in [0u8, 1, 5, 15] {
            let rec = v4_record(7, 8, "a.exe", "a.exe -q", subs);
            let got = parse_start(&rec, 4, PointerWidth::Eight).unwrap();
            assert_eq!(got.image, "a.exe", "sub-authority count {subs}");
            assert_eq!(got.command_line.as_deref(), Some("a.exe -q"));
        }
    }

    #[test]
    fn a_null_identifier_is_one_pointer_rather_than_a_token_user() {
        let mut b = Vec::new();
        b.extend_from_slice(&[0u8; 8]);
        b.extend_from_slice(&11u32.to_le_bytes());
        b.extend_from_slice(&22u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&[0u8; 8]);
        b.extend_from_slice(&0u32.to_le_bytes()); // Flags
        b.extend_from_slice(&[0u8; 8]); // null SID pointer
        b.extend_from_slice(b"x.exe\0");
        b.extend_from_slice(&[0, 0]); // empty command line

        let got = parse_start(&b, 4, PointerWidth::Eight).unwrap();
        assert_eq!(got.pid, 11);
        assert_eq!(got.parent, 22);
        assert_eq!(got.image, "x.exe");
        assert_eq!(got.command_line.as_deref(), Some(""));
    }

    #[test]
    fn a_version_three_record_has_no_flags_field() {
        let mut b = Vec::new();
        b.extend_from_slice(&[0u8; 8]);
        b.extend_from_slice(&33u32.to_le_bytes());
        b.extend_from_slice(&44u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&[0u8; 8]);
        // No Flags here.
        b.extend_from_slice(&[0u8; 8]); // null SID
        b.extend_from_slice(b"v3.exe\0");
        for u in "v3.exe -a".encode_utf16() {
            b.extend_from_slice(&u.to_le_bytes());
        }
        b.extend_from_slice(&[0, 0]);

        let got = parse_start(&b, 3, PointerWidth::Eight).unwrap();
        assert_eq!(got.pid, 33);
        assert_eq!(got.image, "v3.exe");
        assert_eq!(got.command_line.as_deref(), Some("v3.exe -a"));
    }

    #[test]
    fn a_record_too_short_for_its_fixed_fields_yields_nothing() {
        let rec = v4_record(1, 2, "a.exe", "a.exe", 5);
        // Everything up to and including the image name has no fallback, so a
        // record that cannot supply it is not a process at all.
        for cut in [0, 4, 12, 24, 40, 56, 60] {
            assert!(
                parse_start(&rec[..cut], 4, PointerWidth::Eight).is_none(),
                "a record truncated to {cut} bytes must not parse"
            );
        }
    }

    #[test]
    fn a_record_truncated_only_in_its_command_line_still_names_the_process() {
        let rec = v4_record(77, 88, "a.exe", "a.exe -flag", 5);
        // Losing the command line costs one field the type already permits to
        // be absent. Refusing the record would cost the process, and with it
        // every descendant's ancestry, which is the more expensive of the two.
        let got = parse_start(&rec[..rec.len() - 3], 4, PointerWidth::Eight)
            .expect("the process is still identified");
        assert_eq!(got.pid, 77);
        assert_eq!(got.parent, 88);
        assert_eq!(got.image, "a.exe");
        assert_eq!(got.command_line, None);
    }

    #[test]
    fn an_end_record_yields_its_identifier() {
        let rec = v4_record(999, 1, "a.exe", "a.exe", 5);
        assert_eq!(parse_end(&rec, PointerWidth::Eight), Some(999));
        assert_eq!(parse_end(&rec[..4], PointerWidth::Eight), None);
    }

    #[test]
    fn a_thirty_two_bit_producer_uses_narrower_pointers() {
        let mut b = Vec::new();
        b.extend_from_slice(&[0u8; 4]); // UniqueProcessKey, 4 bytes here
        b.extend_from_slice(&55u32.to_le_bytes());
        b.extend_from_slice(&66u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&[0u8; 4]); // DirectoryTableBase
        b.extend_from_slice(&0u32.to_le_bytes()); // Flags
        b.extend_from_slice(&[0u8; 4]); // null SID
        b.extend_from_slice(b"w32.exe\0");
        b.extend_from_slice(&[0, 0]);

        let got = parse_start(&b, 4, PointerWidth::Four).unwrap();
        assert_eq!(got.pid, 55);
        assert_eq!(got.parent, 66);
        assert_eq!(got.image, "w32.exe");
    }
}
