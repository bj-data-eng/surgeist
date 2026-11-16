//! A deterministic entropy cursor. An exhausted slice reads as zero bytes, so a
//! given slice always yields the same recipe; generation never fails on length.

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    pub(crate) fn byte(&mut self) -> u8 {
        let byte = self.bytes.get(self.at).copied().unwrap_or(0);
        self.at = self.at.saturating_add(1);
        byte
    }

    pub(crate) fn below(&mut self, count: usize) -> usize {
        usize::from(self.byte()) % count.max(1)
    }

    pub(crate) fn flag(&mut self) -> bool {
        self.byte() % 2 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_entropy_reads_zero_deterministically() {
        let mut cursor = Cursor::new(&[0xff, 0x03]);
        assert_eq!(cursor.byte(), 0xff);
        assert!(cursor.flag() && cursor.below(0) == 0);
        assert_eq!(cursor.byte(), 0);
        assert_eq!(cursor.below(5), 0);
    }
}
