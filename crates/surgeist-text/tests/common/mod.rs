/// Finite admission policy for the checked-in and authored test font fixtures.
pub const FONT_LIMITS: surgeist_font::DecodeLimits =
    surgeist_font::DecodeLimits::new(2 << 20, 2 << 20, 2 << 20, 256, 16, 1 << 20, 5 << 20);
