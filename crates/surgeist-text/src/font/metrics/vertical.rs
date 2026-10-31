//! Nominal static TrueType vertical metrics, before shaping or placement.
//!
//! Only the selected stored glyph header is a native yMax reference. This is
//! neither outline drawing/ink qualification nor vertical substitution shaping.

use super::{
    EffectiveFontInstance, Field, FontLineMetricSource, Numeric, Phase, Table, TableError,
    fallback_ad, finite, numeric, table, table_error,
};
use crate::{
    Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, FontOutlineCapability, FontOutlineError,
    Result,
};
use skrifa::{
    FontRef, Tag,
    raw::{FontData, TableProvider},
};

/// Independent source of a nominal vertical metric component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalGlyphMetricProvenance {
    /// Actual vmtx advance height, including legitimate last-long repetition.
    VmtxAdvance,
    /// Half the actual nominal horizontal advance, without integer rounding.
    HorizontalAdvanceHalf,
    /// Stored glyf header yMax plus actual vmtx top side bearing.
    GlyfYMaxTopSideBearing,
    /// Source-proved empty glyf record, whose yMax reference is zero, plus TSB.
    EmptyGlyfTopSideBearing,
    /// Actual selected horizontal ascent minus signed descent.
    HorizontalMetricsHeight(FontLineMetricSource),
    /// Actual selected horizontal ascent.
    HorizontalMetricsAscent(FontLineMetricSource),
}

/// Same-instance nominal vertical geometry in the horizontal font-design frame.
///
/// Origins are positive right/up from the horizontal design origin. Advance is
/// a nonnegative magnitude, corresponding to the downward vector (0, -advance).
/// Values are scaled once to the instance size, without hinting, synthetic
/// bold/slant adjustment, integer rounding, or vertical feature execution.
/// The actual resource/publication and coordinate loan remain retained.
#[derive(Debug)]
pub struct VerticalGlyphMetrics<'a> {
    instance: EffectiveFontInstance<'a>,
    glyph_id: u32,
    advance: f64,
    origin_x: f64,
    origin_y: f64,
    advance_provenance: VerticalGlyphMetricProvenance,
    origin_y_provenance: VerticalGlyphMetricProvenance,
}

/// Same-instance nominal facts in design units, before any size conversion.
///
/// The admitted static TrueType tables supply integral advance and origin Y;
/// origin X is half an integral horizontal advance. Keep these exact facts for
/// the consuming provider's checked coordinate conversion, without rounding.
#[derive(Debug)]
pub(crate) struct UnscaledVerticalGlyphMetrics<'a> {
    instance: EffectiveFontInstance<'a>,
    units_per_em: u16,
    glyph_id: u32,
    advance: f64,
    origin_x: f64,
    origin_y: f64,
    advance_provenance: VerticalGlyphMetricProvenance,
    origin_y_provenance: VerticalGlyphMetricProvenance,
}
impl<'a> UnscaledVerticalGlyphMetrics<'a> {
    pub(crate) const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    pub(crate) const fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    pub(crate) const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }
    pub(crate) const fn advance(&self) -> f64 {
        self.advance
    }
    pub(crate) const fn origin_x(&self) -> f64 {
        self.origin_x
    }
    pub(crate) const fn origin_y(&self) -> f64 {
        self.origin_y
    }
    pub(crate) const fn advance_provenance(&self) -> VerticalGlyphMetricProvenance {
        self.advance_provenance
    }
    pub(crate) const fn origin_y_provenance(&self) -> VerticalGlyphMetricProvenance {
        self.origin_y_provenance
    }
}
impl<'a> VerticalGlyphMetrics<'a> {
    /// Actual retained resource, size and complete coordinate loan.
    #[must_use]
    pub const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    /// Actual nominal glyph request; glyph zero is valid.
    #[must_use]
    pub const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }
    /// Finite nonnegative downward advance magnitude.
    #[must_use]
    pub const fn advance(&self) -> f64 {
        self.advance
    }
    /// Vertical origin relative to horizontal design origin, positive right.
    #[must_use]
    pub const fn origin_x(&self) -> f64 {
        self.origin_x
    }
    /// Vertical origin relative to horizontal design origin, positive up.
    #[must_use]
    pub const fn origin_y(&self) -> f64 {
        self.origin_y
    }
    /// Actual source of advance height.
    #[must_use]
    pub const fn advance_provenance(&self) -> VerticalGlyphMetricProvenance {
        self.advance_provenance
    }
    /// Actual source of horizontal origin.
    #[must_use]
    pub const fn origin_x_provenance(&self) -> VerticalGlyphMetricProvenance {
        VerticalGlyphMetricProvenance::HorizontalAdvanceHalf
    }
    /// Actual source of vertical origin.
    #[must_use]
    pub const fn origin_y_provenance(&self) -> VerticalGlyphMetricProvenance {
        self.origin_y_provenance
    }
}

impl<'a> EffectiveFontInstance<'a> {
    /// Resolves unhinted nominal vertical geometry for a static TrueType glyph.
    ///
    /// A coherent vhea/vmtx pair supplies native advance and TSB; absent tables
    /// synthesize height=A-D and origin=(W/2,A) from the actual horizontal A/D
    /// owner. Native origin consumes only the selected glyf header, not painted
    /// bounds. TrueType VORG is irrelevant, even when malformed. Variable faces
    /// (including default coordinates), CFF/CFF2 and VARC are precisely refused.
    /// This operation does not authorize vertical paragraph shaping/placement.
    pub fn vertical_glyph_metrics(
        &self,
        glyph_id: u32,
        expected: &FontCatalogueIdentity,
    ) -> Result<VerticalGlyphMetrics<'a>> {
        let raw = self.unscaled_vertical_glyph_metrics(glyph_id, expected)?;
        let scale = f64::from(self.size) / f64::from(raw.units_per_em());
        let (advance, origin_x, origin_y) = (
            raw.advance() * scale,
            raw.origin_x() * scale,
            raw.origin_y() * scale,
        );
        check(advance, origin_x, origin_y, Phase::Scaled)?;
        Ok(VerticalGlyphMetrics {
            instance: *raw.instance(),
            glyph_id: raw.glyph_id(),
            advance,
            origin_x,
            origin_y,
            advance_provenance: raw.advance_provenance(),
            origin_y_provenance: raw.origin_y_provenance(),
        })
    }

    pub(crate) fn unscaled_vertical_glyph_metrics(
        &self,
        glyph_id: u32,
        expected: &FontCatalogueIdentity,
    ) -> Result<UnscaledVerticalGlyphMetrics<'a>> {
        if self.face.catalogue_identity() != expected {
            return Err(Error::new(
                ErrorCode::FontCatalogueMismatch,
                "associated font catalogue mismatch",
            )
            .with_detail(ErrorDetail::FontCatalogueMismatch {
                expected: expected.clone(),
                actual: self.face.catalogue_identity().clone(),
            }));
        }
        let font = FontRef::from_index(self.face.bytes(), self.face.index()).map_err(|_| {
            Error::new(ErrorCode::FontLoadFailed, "invalid font face").with_detail(
                ErrorDetail::InvalidFontFace {
                    index: self.face.index(),
                },
            )
        })?;
        table(&font, Table::Head, true)?;
        let head = font.head().map_err(|_| malformed(Table::Head))?;
        if head.version().major != 1 || head.version().minor != 0 {
            return Err(table_error(Table::Head, TableError::UnsupportedVersion));
        }
        if head.magic_number() != 0x5f0f3cf5 {
            return Err(malformed(Table::Head));
        }
        let upem = head.units_per_em();
        if !(16..=16384).contains(&upem) {
            return Err(table_error(
                Table::Head,
                TableError::InvalidUnitsPerEm { value: upem },
            ));
        }
        let maxp = required(&font, Table::Maxp)?;
        let version = u32_at(maxp, 0, Table::Maxp)?;
        if !matches!(version, 0x0000_5000 | 0x0001_0000) {
            return Err(table_error(Table::Maxp, TableError::UnsupportedVersion));
        }
        let count = u32::from(u16_at(maxp, 4, Table::Maxp)?);
        if glyph_id >= count {
            return Err(Error::new(
                ErrorCode::InvalidRange,
                "glyph outside retained face inventory",
            )
            .with_detail(ErrorDetail::FontGlyphRange {
                glyph: glyph_id,
                count,
            }));
        }
        if self.axis_count() > 0 {
            return Err(Error::new(
                ErrorCode::UnsupportedFeature,
                "variable vertical glyph metrics are not qualified",
            )
            .with_detail(ErrorDetail::UnsupportedVerticalGlyphVariation {
                glyph: glyph_id,
                axis_count: self.axis_count(),
            }));
        }
        if has(&font, b"VARC") {
            return Err(outline_refused(
                glyph_id,
                FontOutlineError::Unsupported {
                    capability: FontOutlineCapability::VariableComposite,
                },
            ));
        }
        if !has(&font, b"glyf") {
            for (tag, capability) in [
                (b"CFF2", FontOutlineCapability::PostScriptVariation),
                (b"CFF ", FontOutlineCapability::PostScript),
            ] {
                if has(&font, tag) {
                    return Err(outline_refused(
                        glyph_id,
                        FontOutlineError::Unsupported { capability },
                    ));
                }
            }
            return Err(outline_refused(
                glyph_id,
                FontOutlineError::UnavailableOutline,
            ));
        }
        if version != 0x0001_0000 {
            return Err(table_error(Table::Maxp, TableError::UnsupportedVersion));
        }
        if maxp.len() < 32 {
            return Err(malformed(Table::Maxp));
        }
        let hhea = required(&font, Table::Hhea)?;
        let horizontal_count = metric_count(hhea, Table::Hhea, count)?;
        let hmtx = required(&font, Table::Hmtx)?;
        let (width, _) = metric(hmtx, Table::Hmtx, horizontal_count, count, glyph_id)?;
        let vhea = table(&font, Table::Vhea, false)?;
        let vmtx = table(&font, Table::Vmtx, false)?;
        let (advance, origin_y, advance_provenance, origin_y_provenance) = match (vhea, vmtx) {
            (Some(_), None) => {
                return Err(table_error(
                    Table::Vhea,
                    TableError::MissingCompanion {
                        required: Table::Vmtx,
                    },
                ));
            }
            (None, Some(_)) => {
                return Err(table_error(
                    Table::Vmtx,
                    TableError::MissingCompanion {
                        required: Table::Vhea,
                    },
                ));
            }
            (Some(vhea), Some(vmtx)) => {
                if table(&font, Table::Vvar, false)?.is_some() {
                    return Err(table_error(Table::Vvar, TableError::VariationWithoutAxes));
                }
                let long_count = metric_count(vhea, Table::Vhea, count)?;
                let (height, tsb) = metric(vmtx, Table::Vmtx, long_count, count, glyph_id)?;
                let (y_max, empty) = glyph_y_max(
                    &font,
                    head.index_to_loc_format(),
                    head.glyph_data_format(),
                    count,
                    glyph_id,
                )?;
                (
                    height,
                    y_max + tsb,
                    VerticalGlyphMetricProvenance::VmtxAdvance,
                    if empty {
                        VerticalGlyphMetricProvenance::EmptyGlyfTopSideBearing
                    } else {
                        VerticalGlyphMetricProvenance::GlyfYMaxTopSideBearing
                    },
                )
            }
            (None, None) => {
                if table(&font, Table::Vvar, false)?.is_some() {
                    return Err(table_error(
                        Table::Vvar,
                        TableError::MissingCompanion {
                            required: Table::Vhea,
                        },
                    ));
                }
                let (ascent, descent, source) = fallback_ad(&font, self)?;
                (
                    ascent - descent,
                    ascent,
                    VerticalGlyphMetricProvenance::HorizontalMetricsHeight(source),
                    VerticalGlyphMetricProvenance::HorizontalMetricsAscent(source),
                )
            }
        };
        let origin_x = width / 2.0;
        finite(Field::HorizontalAdvance, Phase::Unscaled, width)?;
        check(advance, origin_x, origin_y, Phase::Unscaled)?;
        Ok(UnscaledVerticalGlyphMetrics {
            instance: *self,
            units_per_em: upem,
            glyph_id,
            advance,
            origin_x,
            origin_y,
            advance_provenance,
            origin_y_provenance,
        })
    }
}

fn has(font: &FontRef<'_>, tag: &[u8; 4]) -> bool {
    font.table_directory()
        .table_records()
        .iter()
        .any(|record| record.tag() == Tag::new(tag))
}
fn required<'a>(font: &FontRef<'a>, which: Table) -> Result<FontData<'a>> {
    table(font, which, true)?.ok_or_else(|| table_error(which, TableError::Missing))
}
fn malformed(which: Table) -> Error {
    table_error(which, TableError::Malformed)
}
fn u16_at(data: FontData<'_>, at: usize, which: Table) -> Result<u16> {
    data.read_at(at).map_err(|_| malformed(which))
}
fn i16_at(data: FontData<'_>, at: usize, which: Table) -> Result<i16> {
    data.read_at(at).map_err(|_| malformed(which))
}
fn u32_at(data: FontData<'_>, at: usize, which: Table) -> Result<u32> {
    data.read_at(at).map_err(|_| malformed(which))
}
fn metric_count(data: FontData<'_>, which: Table, glyph_count: u32) -> Result<u16> {
    let version = u32_at(data, 0, which)?;
    if version != 0x0001_0000 && !(which == Table::Vhea && version == 0x0001_1000) {
        return Err(table_error(which, TableError::UnsupportedVersion));
    }
    if data.len() < 36 || i16_at(data, 32, which)? != 0 {
        return Err(malformed(which));
    }
    let count = u16_at(data, 34, which)?;
    if count == 0 || u32::from(count) > glyph_count {
        return Err(table_error(
            which,
            TableError::LongMetricCount { count, glyph_count },
        ));
    }
    Ok(count)
}
fn metric(
    data: FontData<'_>,
    which: Table,
    long_count: u16,
    glyph_count: u32,
    glyph: u32,
) -> Result<(f64, f64)> {
    // Counts are bounded by maxp's u16 inventory, so this complete required
    // extent and selected entry arithmetic cannot overflow usize.
    let long_count = usize::from(long_count);
    let glyph_count = usize::try_from(glyph_count).map_err(|_| malformed(which))?;
    let glyph = usize::try_from(glyph).map_err(|_| malformed(which))?;
    if data.len() < long_count * 4 + (glyph_count - long_count) * 2 {
        return Err(malformed(which));
    }
    let advance = u16_at(data, glyph.min(long_count - 1) * 4, which)?;
    let bearing_at = if glyph < long_count {
        glyph * 4 + 2
    } else {
        long_count * 4 + (glyph - long_count) * 2
    };
    Ok((
        f64::from(advance),
        f64::from(i16_at(data, bearing_at, which)?),
    ))
}
fn glyph_y_max(
    font: &FontRef<'_>,
    format: i16,
    glyph_format: i16,
    count: u32,
    glyph: u32,
) -> Result<(f64, bool)> {
    if !matches!(format, 0 | 1) || glyph_format != 0 {
        return Err(malformed(Table::Head));
    }
    let loca = required(font, Table::Loca)?;
    let glyf = required(font, Table::Glyf)?;
    let stride = if format == 0 { 2 } else { 4 };
    let count = usize::try_from(count).map_err(|_| malformed(Table::Loca))?;
    let glyph = usize::try_from(glyph).map_err(|_| malformed(Table::Loca))?;
    if loca.len() < (count + 1) * stride {
        return Err(malformed(Table::Loca));
    }
    let offset = |at| -> Result<usize> {
        let value = if format == 0 {
            u32::from(u16_at(loca, at * stride, Table::Loca)?) * 2
        } else {
            u32_at(loca, at * stride, Table::Loca)?
        };
        usize::try_from(value).map_err(|_| malformed(Table::Loca))
    };
    let start = offset(glyph)?;
    let end = offset(glyph + 1)?;
    if start > end || end > glyf.len() {
        return Err(malformed(Table::Loca));
    }
    if start == end {
        return Ok((0.0, true));
    }
    if end - start < 10 {
        return Err(malformed(Table::Glyf));
    }
    let x_min = i16_at(glyf, start + 2, Table::Glyf)?;
    let y_min = i16_at(glyf, start + 4, Table::Glyf)?;
    let x_max = i16_at(glyf, start + 6, Table::Glyf)?;
    let y_max = i16_at(glyf, start + 8, Table::Glyf)?;
    if x_min > x_max || y_min > y_max {
        return Err(malformed(Table::Glyf));
    }
    Ok((f64::from(y_max), false))
}
fn check(advance: f64, x: f64, y: f64, phase: Phase) -> Result<()> {
    finite(Field::VerticalAdvance, phase, advance)?;
    if advance < 0.0 {
        return Err(numeric(
            Field::VerticalAdvance,
            phase,
            Numeric::Negative,
            advance,
        ));
    }
    finite(Field::VerticalOriginX, phase, x)?;
    finite(Field::VerticalOriginY, phase, y)
}
fn outline_refused(glyph: u32, reason: FontOutlineError) -> Error {
    Error::new(
        ErrorCode::FontOutlineRefused,
        "unsupported nominal vertical outline profile",
    )
    .with_detail(ErrorDetail::FontOutline { glyph, reason })
}

#[cfg(test)]
mod tests;
