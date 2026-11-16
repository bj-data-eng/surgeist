//! Strict bounded transport. Semantic product rejection stays with each target.
use crate::error;
use surgeist_fuzz::Result;

const REVISION: u8 = 1;

pub(crate) struct Reader<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    /// Checks the whole-case ceiling, target magic and revision before any field.
    pub(crate) fn new(input: &'a [u8], magic: [u8; 4], max: usize) -> Result<Self> {
        if input.len() > max || input.len() < 5 || input[..4] != magic || input[4] != REVISION {
            return Err(error("case size, magic or revision"));
        }
        Ok(Self { input, cursor: 5 })
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .cursor
            .checked_add(count)
            .ok_or_else(|| error("case length overflow"))?;
        let bytes = self
            .input
            .get(self.cursor..end)
            .ok_or_else(|| error("truncated case"))?;
        self.cursor = end;
        Ok(bytes)
    }

    pub(crate) fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    pub(crate) fn count(&mut self, max: usize) -> Result<usize> {
        let count = usize::try_from(self.u32()?).map_err(|_| error("count does not fit host"))?;
        if count > max {
            return Err(error("case count ceiling"));
        }
        Ok(count)
    }

    /// A length-prefixed byte string, bounded before it is copied.
    pub(crate) fn bytes(&mut self, max: usize) -> Result<Vec<u8>> {
        let count = self.count(max)?;
        let bytes = self.take(count)?;
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(count)
            .map_err(|_| error("case byte allocation"))?;
        owned.extend_from_slice(bytes);
        Ok(owned)
    }

    /// A length-prefixed strict UTF-8 string; malformed bytes are rejected.
    pub(crate) fn text(&mut self, max: usize) -> Result<String> {
        String::from_utf8(self.bytes(max)?).map_err(|_| error("text is not valid UTF-8"))
    }

    pub(crate) fn finish(self) -> Result<()> {
        if self.cursor != self.input.len() {
            return Err(error("trailing case bytes"));
        }
        Ok(())
    }
}

pub(crate) struct Writer {
    bytes: Vec<u8>,
    max: usize,
}

impl Writer {
    pub(crate) fn new(magic: [u8; 4], max: usize) -> Result<Self> {
        let mut writer = Self {
            bytes: Vec::new(),
            max,
        };
        writer.append(&magic)?;
        writer.u8(REVISION)?;
        Ok(writer)
    }

    fn append(&mut self, bytes: &[u8]) -> Result<()> {
        let total = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or_else(|| error("case length overflow"))?;
        if total > self.max {
            return Err(error("canonical byte ceiling"));
        }
        self.bytes
            .try_reserve_exact(bytes.len())
            .map_err(|_| error("canonical byte allocation"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    pub(crate) fn u8(&mut self, value: u8) -> Result<()> {
        self.append(&[value])
    }

    pub(crate) fn u32(&mut self, value: u32) -> Result<()> {
        self.append(&value.to_le_bytes())
    }

    pub(crate) fn count(&mut self, value: usize, max: usize) -> Result<()> {
        if value > max {
            return Err(error("case count ceiling"));
        }
        self.u32(u32::try_from(value).map_err(|_| error("case count exceeds encoding"))?)
    }

    pub(crate) fn bytes(&mut self, bytes: &[u8], max: usize) -> Result<()> {
        self.count(bytes.len(), max)?;
        self.append(bytes)
    }

    pub(crate) fn text(&mut self, text: &str, max: usize) -> Result<()> {
        self.bytes(text.as_bytes(), max)
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip_and_strict_text_rejects_malformed_utf8() {
        let mut writer = Writer::new(*b"TEST", 64).unwrap();
        writer.u8(7).unwrap();
        writer.u32(0x0102_0304).unwrap();
        writer.bytes(&[0xff, 0x00, 0xfe], 3).unwrap();
        writer.text("a\u{0}\u{10ffff}", 16).unwrap();
        let bytes = writer.finish();
        let mut reader = Reader::new(&bytes, *b"TEST", 64).unwrap();
        assert_eq!(reader.u8().unwrap(), 7);
        assert_eq!(reader.u32().unwrap(), 0x0102_0304);
        assert_eq!(reader.bytes(3).unwrap(), [0xff, 0x00, 0xfe]);
        assert_eq!(reader.text(16).unwrap(), "a\u{0}\u{10ffff}");
        reader.finish().unwrap();

        let mut malformed = Writer::new(*b"TEST", 64).unwrap();
        malformed.bytes(&[0xc0, 0x80], 2).unwrap();
        let malformed = malformed.finish();
        assert!(
            Reader::new(&malformed, *b"TEST", 64)
                .unwrap()
                .text(2)
                .is_err()
        );
    }

    #[test]
    fn wrong_magic_revision_truncation_trailing_and_oversize_are_rejected() {
        assert!(Reader::new(b"TEST\x02", *b"TEST", 64).is_err());
        assert!(Reader::new(b"ELSE\x01", *b"TEST", 64).is_err());
        assert!(Reader::new(b"TEST", *b"TEST", 64).is_err());
        assert!(
            Reader::new(b"TEST\x01\x00", *b"TEST", 64)
                .unwrap()
                .finish()
                .is_err()
        );
        assert!(
            Reader::new(b"TEST\x01\x00", *b"TEST", 64)
                .unwrap()
                .u32()
                .is_err()
        );
        assert!(Reader::new(b"TEST\x01\x00", *b"TEST", 5).is_err());
    }

    #[test]
    fn declared_counts_are_bounded_before_copying() {
        let mut reader = Reader::new(b"TEST\x01\xff\xff\xff\xff", *b"TEST", 64).unwrap();
        assert!(reader.bytes(1024).is_err());
        let mut reader = Reader::new(b"TEST\x01\x03\x00\x00\x00\x01", *b"TEST", 64).unwrap();
        assert!(reader.bytes(3).is_err());
        let mut writer = Writer::new(*b"TEST", 64).unwrap();
        assert!(writer.bytes(&[0; 4], 3).is_err());
        let mut small = Writer::new(*b"TEST", 8).unwrap();
        assert!(small.bytes(&[0; 4], 8).is_err());
    }
}
