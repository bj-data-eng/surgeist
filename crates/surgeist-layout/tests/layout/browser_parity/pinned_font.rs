//! One fixture resource, shared as bytes; producer and reader schemas stay independent.

pub const ID: &str = "ahem-ttf-f0a92cd0";
pub const SHA256: &str = "f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc";
pub const BYTES: &[u8] = include_bytes!("fonts/Ahem.ttf");

fn word(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}
fn long(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

/// Actual nominal coverage of the pinned resource's Microsoft Unicode format-4
/// cmap. This is only fixture resource metadata, not a font or shaping provider.
pub fn nominal_glyph(code_point: u32) -> Option<u16> {
    let code_point = u16::try_from(code_point).ok()?;
    let count = usize::from(word(BYTES, 4)?);
    let mut cmap = None;
    for index in 0..count {
        let row = 12_usize.checked_add(index.checked_mul(16)?)?;
        if BYTES.get(row..row.checked_add(4)?)? == b"cmap" {
            let start = usize::try_from(long(BYTES, row + 8)?).ok()?;
            let length = usize::try_from(long(BYTES, row + 12)?).ok()?;
            cmap = Some(BYTES.get(start..start.checked_add(length)?)?);
        }
    }
    let cmap = cmap?;
    for record in 0..usize::from(word(cmap, 2)?) {
        let record = 4 + record * 8;
        if word(cmap, record)? != 3 || word(cmap, record + 2)? != 1 {
            continue;
        }
        let start = usize::try_from(long(cmap, record + 4)?).ok()?;
        if word(cmap, start)? != 4 {
            return None;
        }
        let length = usize::from(word(cmap, start + 2)?);
        let table = cmap.get(start..start.checked_add(length)?)?;
        let count = usize::from(word(table, 6)?) / 2;
        for segment in 0..count {
            let end = word(table, 14 + segment * 2)?;
            let start = word(table, 16 + count * 2 + segment * 2)?;
            if !(start..=end).contains(&code_point) {
                continue;
            }
            let delta = word(table, 16 + count * 4 + segment * 2)?;
            let range_position = 16 + count * 6 + segment * 2;
            let range_offset = usize::from(word(table, range_position)?);
            let glyph = if range_offset == 0 {
                code_point.wrapping_add(delta)
            } else {
                let glyph = word(
                    table,
                    range_position
                        .checked_add(range_offset)?
                        .checked_add(usize::from(code_point - start) * 2)?,
                )?;
                if glyph == 0 {
                    0
                } else {
                    glyph.wrapping_add(delta)
                }
            };
            return (glyph != 0).then_some(glyph);
        }
        return None;
    }
    None
}

/// These are source controls interpreted without requesting a font glyph by the
/// selected Text profile. Tabs are deliberately not admitted as space surrogates.
pub const fn source_control(code_point: u32) -> bool {
    matches!(code_point, 0x0a | 0x0d | 0x061c | 0x200e | 0x200f | 0x2028 | 0x2029 | 0x202a..=0x202e | 0x2066..=0x2069)
}

pub fn covers(units: &[u16]) -> bool {
    char::decode_utf16(units.iter().copied()).all(|value| {
        value.is_ok_and(|value| {
            let value = u32::from(value);
            source_control(value) || nominal_glyph(value).is_some()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_resource_has_independent_positive_and_negative_nominal_coverage() {
        assert_eq!(BYTES.len(), 22_572);
        assert_eq!(nominal_glyph(u32::from('X')), Some(58));
        assert!(covers(&[0x0058, 0x2066, 0x0058, 0x2069]));
        assert!(!covers(&[0x05d0]));
        assert!(!covers(&[0xd800]));
        assert!(!covers(&[0x0009]));
    }
}
