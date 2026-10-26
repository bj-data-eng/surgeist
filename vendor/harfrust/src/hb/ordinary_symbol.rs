//! Closed native presentation for genuine nonexempt control scalars.

use super::buffer::{hb_buffer_t, GlyphInfo, ShapingRefusal};

/// Invalid scalar-indexed ordinary-symbol presentation metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinarySymbolInputError {
    /// An index is outside the complete Unicode input.
    SourceIndex,
    /// Scalar indices must be strictly increasing and unrepeated.
    SourceOrder,
    /// A selected scalar is not Cc other than TAB, LF or CR.
    SourceCharacter,
    /// Input storage exceeds the provider's representable scalar bound.
    GrowthLimit,
    /// Installing the metadata exceeds the actual input-operation allowance.
    WorkLimit,
}

pub(crate) fn eligible(cp: u32) -> bool {
    matches!(cp, 0..=8 | 11..=12 | 14..=31 | 127..=159)
}

pub(crate) fn install(
    buffer: &mut hb_buffer_t,
    positions: &[u32],
) -> Result<(), OrdinarySymbolInputError> {
    // No metadata allocation: the scalar field is part of the existing bounded
    // row buffers. Check caller spans before scanning or changing any row.
    if buffer.len > hb_buffer_t::MAX_LEN_DEFAULT {
        return Err(OrdinarySymbolInputError::GrowthLimit);
    }
    // Validation can stop after at most len+1 entries: a longer sequence must
    // repeat, reverse or exceed the input. Bound that work before inspection.
    let inspected = positions.len().min(buffer.len.saturating_add(1));
    let remaining = buffer
        .len
        .checked_add(inspected.saturating_mul(2))
        .and_then(|n| i32::try_from(n).ok())
        .and_then(|n| buffer.max_ops.checked_sub(n))
        .filter(|&n| n >= 0)
        .ok_or(OrdinarySymbolInputError::WorkLimit)?;
    let mut previous = None;
    for &position in positions {
        if previous.is_some_and(|prior| position <= prior) {
            return Err(OrdinarySymbolInputError::SourceOrder);
        }
        let info = buffer
            .info
            .get(position as usize)
            .filter(|_| (position as usize) < buffer.len)
            .ok_or(OrdinarySymbolInputError::SourceIndex)?;
        if !eligible(info.glyph_id) {
            return Err(OrdinarySymbolInputError::SourceCharacter);
        }
        previous = Some(position);
    }
    for info in &mut buffer.info[..buffer.len] {
        info.joining_lineage &= 0x7fff_ffff;
    }
    for &position in positions {
        buffer.info[position as usize].joining_lineage |= 0x8000_0000;
    }
    buffer.ordinary_symbol_input_len = (!positions.is_empty()).then_some(buffer.len);
    buffer.max_ops = remaining;
    Ok(())
}

pub(crate) fn begin(buffer: &mut hb_buffer_t) -> bool {
    let Some(length) = buffer.ordinary_symbol_input_len else {
        return true;
    };
    if length != buffer.len {
        buffer.refuse(ShapingRefusal::OrdinarySymbolInput);
        buffer.successful = false;
        return false;
    }
    super::original_joining::work(buffer, length)
}

impl GlyphInfo {
    pub(crate) fn is_ordinary_symbol(&self) -> bool {
        self.joining_lineage & 0x8000_0000 != 0
    }

    pub(crate) fn joining_lineage(&self) -> u32 {
        self.joining_lineage & 0x7fff_ffff
    }

    pub(crate) fn set_joining_lineage(&mut self, lineage: u32) {
        // Node admission proves the low-bit bound before issuance.
        self.joining_lineage = (self.joining_lineage & 0x8000_0000) | lineage;
    }
}

#[cfg(test)]
#[path = "ordinary_symbol_tests.rs"]
mod tests;
