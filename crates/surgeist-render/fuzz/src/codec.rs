//! Strict bounded transport; semantic product rejection remains target-owned.
use crate::error;
use surgeist_fuzz::Result;

pub(crate) const MAX_CASE_BYTES: usize = 128 * 1024;
const REVISION: u8 = 1;

pub(crate) struct Reader<'a> {
    input: &'a [u8],
    cursor: usize,
}
impl<'a> Reader<'a> {
    pub(crate) fn new(input: &'a [u8], magic: [u8; 4]) -> Result<Self> {
        if input.len() > MAX_CASE_BYTES
            || input.len() < 5
            || input[..4] != magic
            || input[4] != REVISION
        {
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
    pub(crate) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }
    pub(crate) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }
    pub(crate) fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }
    pub(crate) fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    pub(crate) fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.u64()?))
    }
    pub(crate) fn count(&mut self, max: usize) -> Result<usize> {
        let count = usize::try_from(self.u32()?).map_err(|_| error("count does not fit host"))?;
        if count > max {
            return Err(error("case count ceiling"));
        }
        Ok(count)
    }
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
    pub(crate) fn finish(self) -> Result<()> {
        if self.cursor != self.input.len() {
            return Err(error("trailing case bytes"));
        }
        Ok(())
    }
}

pub(crate) struct Writer {
    bytes: Vec<u8>,
}
impl Writer {
    pub(crate) fn new(magic: [u8; 4]) -> Result<Self> {
        let mut writer = Self { bytes: Vec::new() };
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
        if total > MAX_CASE_BYTES {
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
    pub(crate) fn u16(&mut self, value: u16) -> Result<()> {
        self.append(&value.to_le_bytes())
    }
    pub(crate) fn u32(&mut self, value: u32) -> Result<()> {
        self.append(&value.to_le_bytes())
    }
    pub(crate) fn u64(&mut self, value: u64) -> Result<()> {
        self.append(&value.to_le_bytes())
    }
    pub(crate) fn f32(&mut self, value: f32) -> Result<()> {
        self.u32(value.to_bits())
    }
    pub(crate) fn f64(&mut self, value: f64) -> Result<()> {
        self.u64(value.to_bits())
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
    pub(crate) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_transport_preserves_exceptional_ieee_bits_and_signed_zero() {
        let mut writer = Writer::new(*b"TEST").unwrap();
        let singles = [0, 0x8000_0000, 0x7f80_0000, 0xff80_0000, 0x7fc0_1234];
        let doubles = [
            0,
            0x8000_0000_0000_0000,
            0x7ff0_0000_0000_0000,
            0x7ff8_0000_0000_1234,
        ];
        writer.u16(0x9876).unwrap();
        for bits in singles {
            writer.f32(f32::from_bits(bits)).unwrap();
        }
        for bits in doubles {
            writer.f64(f64::from_bits(bits)).unwrap();
        }
        writer.bytes(&[17, 23, 41], 3).unwrap();
        let bytes = writer.finish();
        let mut reader = Reader::new(&bytes, *b"TEST").unwrap();
        assert_eq!(reader.u16().unwrap(), 0x9876);
        for bits in singles {
            assert_eq!(reader.f32().unwrap().to_bits(), bits);
        }
        for bits in doubles {
            assert_eq!(reader.f64().unwrap().to_bits(), bits);
        }
        assert_eq!(reader.bytes(3).unwrap(), [17, 23, 41]);
        reader.finish().unwrap();
    }
    #[test]
    fn wrong_target_revision_truncation_and_trailing_bytes_are_rejected() {
        assert!(Reader::new(b"TEST\x02", *b"TEST").is_err());
        assert!(Reader::new(b"ELSE\x01", *b"TEST").is_err());
        assert!(Reader::new(b"TEST", *b"TEST").is_err());
        assert!(
            Reader::new(b"TEST\x01\x00", *b"TEST")
                .unwrap()
                .finish()
                .is_err()
        );
        assert!(
            Reader::new(b"TEST\x01\x00", *b"TEST")
                .unwrap()
                .u32()
                .is_err()
        );
    }
    #[test]
    fn declared_and_actual_byte_bounds_reject_before_copying() {
        let mut reader = Reader::new(b"TEST\x01\xff\xff\xff\xff", *b"TEST").unwrap();
        assert!(reader.bytes(32 * 1024).is_err());
        let mut reader = Reader::new(b"TEST\x01\x03\x00\x00\x00\x01", *b"TEST").unwrap();
        assert!(reader.bytes(3).is_err());
        let mut writer = Writer::new(*b"TEST").unwrap();
        assert!(writer.bytes(&[0; 4], 3).is_err());
        let mut oversized = vec![0; MAX_CASE_BYTES + 1];
        oversized[..5].copy_from_slice(b"TEST\x01");
        assert!(Reader::new(&oversized, *b"TEST").is_err());
    }
}
