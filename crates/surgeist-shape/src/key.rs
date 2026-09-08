use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(u128);

impl Key {
    #[must_use]
    pub fn from_shape(shape: &Shape) -> Self {
        shape.key()
    }

    #[must_use]
    pub fn from_path(path: &Path, fill_rule: FillRule) -> Self {
        path.key(fill_rule)
    }

    #[must_use]
    pub const fn from_parts(high: u64, low: u64) -> Self {
        Self(((high as u128) << 64) | low as u128)
    }

    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }
}
pub(crate) fn hash_point(state: &mut StableHasher, point: Point) {
    state.write_f64(point.x());
    state.write_f64(point.y());
}

pub(crate) fn hash_rect(state: &mut StableHasher, rect: Rect) {
    hash_point(state, rect.origin());
    state.write_f64(rect.size().width());
    state.write_f64(rect.size().height());
}

pub(crate) fn hash_radii(state: &mut StableHasher, radii: Radii) {
    state.write_f64(radii.top_left());
    state.write_f64(radii.top_right());
    state.write_f64(radii.bottom_right());
    state.write_f64(radii.bottom_left());
}

#[derive(Clone, Copy)]
pub(crate) struct StableHasher {
    state: u64,
}

impl StableHasher {
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn write_u8(&mut self, value: u8) {
        self.state ^= u64::from(value);
        self.state = self.state.wrapping_mul(Self::PRIME);
    }

    pub(crate) fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    pub(crate) fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    pub(crate) fn write_f64(&mut self, value: f64) {
        self.write_u64(value.to_bits());
    }

    pub(crate) fn finish(self) -> u64 {
        self.state
    }

    pub(crate) fn finish_with_seed(self, seed: u64) -> u64 {
        self.state ^ seed.rotate_left(17)
    }
}
