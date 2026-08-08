// SPDX-License-Identifier: Apache-2.0

//! Block and option framing.
//!
//! pcapng is a sequence of self-delimiting blocks, and every block and every
//! option is padded to a 32-bit boundary with the padding excluded from the
//! declared length. That rule is the whole of this module, and getting it wrong
//! is the defect a format writer actually produces: not a wrong idea, an
//! off-by-one on a length nobody can see by reading the code.
//!
//! Nothing here knows what a packet is. It writes little-endian regardless of
//! host, per specification section 13.2 as this slice fixed it, so the same
//! input produces the same bytes on any machine.

use std::io::Write;

use crate::error::WriteError;

/// pcapng block type values.
pub(crate) mod block_type {
    pub(crate) const SECTION_HEADER: u32 = 0x0A0D_0D0A;
    pub(crate) const INTERFACE_DESCRIPTION: u32 = 0x0000_0001;
    pub(crate) const INTERFACE_STATISTICS: u32 = 0x0000_0005;
    pub(crate) const ENHANCED_PACKET: u32 = 0x0000_0006;
}

/// pcapng option codes.
///
/// Codes are per-block except `END_OF_OPT` and `COMMENT`, which are universal.
/// That is why `IF_NAME` and `ISB_IFRECV` can share the value 2 without
/// ambiguity: they never appear in the same block.
pub(crate) mod opt {
    pub(crate) const END_OF_OPT: u16 = 0;
    pub(crate) const COMMENT: u16 = 1;

    pub(crate) const SHB_USERAPPL: u16 = 4;

    pub(crate) const IF_NAME: u16 = 2;
    pub(crate) const IF_TSRESOL: u16 = 9;

    pub(crate) const ISB_IFRECV: u16 = 4;
    pub(crate) const ISB_IFDROP: u16 = 5;
    pub(crate) const ISB_OSDROP: u16 = 7;
}

/// The largest value a 16-bit option length can express.
const MAX_OPTION_LEN: usize = u16::MAX as usize;

/// Bytes of padding needed to reach the next 32-bit boundary.
pub(crate) fn padding_for(len: usize) -> usize {
    (4 - len % 4) % 4
}

/// An option list under construction.
///
/// Accumulates into a buffer rather than writing straight through, because a
/// block's total length has to be known before its first byte is emitted and
/// the options are what make it variable.
#[derive(Debug, Default)]
pub(crate) struct Options {
    buf: Vec<u8>,
}

impl Options {
    pub(crate) fn new() -> Self {
        Options { buf: Vec::new() }
    }

    /// Append one option: code, value length before padding, value, padding.
    pub(crate) fn push(&mut self, code: u16, value: &[u8]) -> Result<(), WriteError> {
        if value.len() > MAX_OPTION_LEN {
            return Err(WriteError::OptionTooLong {
                code,
                len: value.len(),
            });
        }
        self.buf.extend_from_slice(&code.to_le_bytes());
        self.buf
            .extend_from_slice(&(value.len() as u16).to_le_bytes());
        self.buf.extend_from_slice(value);
        self.buf
            .extend_from_slice(&vec![0u8; padding_for(value.len())]);
        Ok(())
    }

    pub(crate) fn push_str(&mut self, code: u16, value: &str) -> Result<(), WriteError> {
        self.push(code, value.as_bytes())
    }

    pub(crate) fn push_u64(&mut self, code: u16, value: u64) -> Result<(), WriteError> {
        self.push(code, &value.to_le_bytes())
    }

    pub(crate) fn push_u8(&mut self, code: u16, value: u8) -> Result<(), WriteError> {
        self.push(code, &[value])
    }

    /// Finish the list, appending `opt_endofopt`.
    ///
    /// An empty list stays empty: pcapng permits a block with no options at
    /// all, and writing a lone terminator would add four bytes that say
    /// nothing.
    pub(crate) fn finish(mut self) -> Vec<u8> {
        if !self.buf.is_empty() {
            self.buf.extend_from_slice(&opt::END_OF_OPT.to_le_bytes());
            self.buf.extend_from_slice(&0u16.to_le_bytes());
        }
        self.buf
    }
}

/// Write one complete block: type, total length, body, padding, total length.
///
/// The trailing length is what lets a reader walk the file backwards, and it
/// must equal the leading one. Both include all twelve bytes of framing.
pub(crate) fn write_block<W: Write>(
    out: &mut W,
    block_type: u32,
    body: &[u8],
) -> Result<(), WriteError> {
    let pad = padding_for(body.len());
    let total = (body.len() + pad + 12) as u32;
    out.write_all(&block_type.to_le_bytes())?;
    out.write_all(&total.to_le_bytes())?;
    out.write_all(body)?;
    out.write_all(&vec![0u8; pad])?;
    out.write_all(&total.to_le_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le16(b: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([b[at], b[at + 1]])
    }

    fn le32(b: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }

    #[test]
    fn padding_reaches_the_next_boundary() {
        assert_eq!(padding_for(0), 0);
        assert_eq!(padding_for(1), 3);
        assert_eq!(padding_for(2), 2);
        assert_eq!(padding_for(3), 1);
        assert_eq!(padding_for(4), 0);
        assert_eq!(padding_for(5), 3);
    }

    /// The declared length is the value length. Padding exists to align the
    /// next option and is not part of what was recorded.
    #[test]
    fn option_length_excludes_padding() {
        for len in [0usize, 1, 3, 4, 5] {
            let mut o = Options::new();
            o.push(0x1234, &vec![0xAAu8; len]).expect("within limits");
            let b = o.finish();

            assert_eq!(le16(&b, 0), 0x1234, "code at offset 0");
            assert_eq!(le16(&b, 2) as usize, len, "declared length is unpadded");

            let consumed = 4 + len + padding_for(len);
            assert_eq!(consumed % 4, 0, "option occupies whole 32-bit words");
            assert_eq!(
                le16(&b, consumed),
                opt::END_OF_OPT,
                "terminator follows the padded option"
            );
        }
    }

    #[test]
    fn an_option_list_ends_with_end_of_opt() {
        let mut o = Options::new();
        o.push_str(opt::COMMENT, "hi").expect("short");
        let b = o.finish();
        assert_eq!(le16(&b, b.len() - 4), opt::END_OF_OPT);
        assert_eq!(le16(&b, b.len() - 2), 0, "terminator carries no value");
    }

    #[test]
    fn an_empty_option_list_writes_nothing() {
        assert!(Options::new().finish().is_empty());
    }

    #[test]
    fn integer_options_are_little_endian_on_every_host() {
        let mut o = Options::new();
        o.push_u64(opt::ISB_IFRECV, 1).expect("short");
        let b = o.finish();
        assert_eq!(&b[4..12], &[1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn an_oversized_option_is_refused_not_truncated() {
        let mut o = Options::new();
        let err = o
            .push(opt::COMMENT, &vec![0u8; MAX_OPTION_LEN + 1])
            .expect_err("over the 16-bit limit");
        assert_eq!(
            err,
            WriteError::OptionTooLong {
                code: opt::COMMENT,
                len: MAX_OPTION_LEN + 1
            }
        );
    }

    #[test]
    fn a_maximum_length_option_is_accepted() {
        let mut o = Options::new();
        o.push(opt::COMMENT, &vec![0u8; MAX_OPTION_LEN])
            .expect("exactly at the limit");
    }

    #[test]
    fn block_lengths_agree_and_include_the_framing() {
        for body_len in [0usize, 1, 4, 7] {
            let mut out = Vec::new();
            write_block(
                &mut out,
                block_type::ENHANCED_PACKET,
                &vec![0xBBu8; body_len],
            )
            .expect("in-memory write cannot fail");

            let leading = le32(&out, 4);
            let trailing = le32(&out, out.len() - 4);
            assert_eq!(leading, trailing, "a reader walks this file both ways");
            assert_eq!(leading as usize, out.len(), "length covers the whole block");
            assert_eq!(
                leading as usize,
                body_len + padding_for(body_len) + 12,
                "twelve bytes of framing plus a padded body"
            );
            assert_eq!(leading % 4, 0, "blocks occupy whole 32-bit words");
        }
    }

    #[test]
    fn block_type_is_written_first() {
        let mut out = Vec::new();
        write_block(&mut out, block_type::SECTION_HEADER, &[]).expect("in-memory");
        assert_eq!(le32(&out, 0), 0x0A0D_0D0A);
    }
}
