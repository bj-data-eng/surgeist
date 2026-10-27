mod baseline;
mod variation;
mod vertical;
pub use baseline::{
    FontBaseline, FontBaselineAbsence, FontBaselineAxis, FontBaselineMeasurement,
    FontBaselineOutcome, FontBaselineProvenance, FontBaselineScript, FontBaselineUnavailable,
    FontBaselineUnavailableReason, FontBaselineValue,
};
pub use vertical::{VerticalGlyphMetricProvenance, VerticalGlyphMetrics};

use skrifa::{
    FontRef, Tag,
    instance::{LocationRef, Size},
    raw::{FontData as TableData, TableProvider},
};

use crate::{
    Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, FontMetricField as Field,
    FontMetricNumericError as Numeric, FontMetricPhase as Phase, FontMetricTable as Table,
    FontMetricTableError as TableError, Result,
};

use super::EffectiveFontInstance;

/// Same-instance font metrics retaining the actual resource and coordinate loan.
///
/// Geometry is f64, independent of glyphs, line boxes, advances or ink. A result
/// can outlive a temporary instance view, but not its actual resource/coordinate
/// storage. This operation does not change current paragraph metrics.
#[derive(Debug)]
pub struct FontMetrics<'a> {
    instance: EffectiveFontInstance<'a>,
    units_per_em: u16,
    ascent: f64,
    descent: f64,
    leading: f64,
    strikeout: StrikeoutMetrics,
    line_metric_source: FontLineMetricSource,
}

impl<'a> FontMetrics<'a> {
    /// Actual face, canonical size and complete coordinates behind these metrics.
    #[must_use]
    pub const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    /// Actual design units per em.
    #[must_use]
    pub const fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    /// Actual stored source selected for ascent/descent; leading may differ.
    #[must_use]
    pub const fn line_metric_source(&self) -> FontLineMetricSource {
        self.line_metric_source
    }
    /// Nonnegative distance above the alphabetic baseline.
    #[must_use]
    pub const fn ascent(&self) -> f64 {
        self.ascent
    }
    /// Signed nonpositive descent below the baseline.
    #[must_use]
    pub const fn descent(&self) -> f64 {
        self.descent
    }
    /// Finite signed additional line spacing.
    #[must_use]
    pub const fn leading(&self) -> f64 {
        self.leading
    }
    /// Normalized center and thickness for this actual instance.
    #[must_use]
    pub const fn strikeout(&self) -> StrikeoutMetrics {
        self.strikeout
    }
}

/// Signed baseline-relative strike center and strictly positive thickness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrikeoutMetrics {
    center_offset: f64,
    thickness: f64,
}
impl StrikeoutMetrics {
    /// Signed center offset, positive above the alphabetic baseline.
    /// At horizontal baseline b, top is b - center_offset - thickness/2.
    #[must_use]
    pub const fn center_offset(self) -> f64 {
        self.center_offset
    }
    /// Strictly positive same-instance thickness.
    #[must_use]
    pub const fn thickness(self) -> f64 {
        self.thickness
    }
}

impl<'a> EffectiveFontInstance<'a> {
    /// Resolves glyph-free same-instance metrics against the captured catalogue basis.
    ///
    /// Historical resources work under their retained original publication.
    /// Static and qualified MVAR metrics use the exact canonical coordinates.
    /// Concrete synthesis leaves these line metrics unchanged. A usable strike top is normalized to
    /// center; fallback is ascent/2 and actual size/18, not raw UPEM/18.
    pub fn metrics(&self, expected_catalogue: &FontCatalogueIdentity) -> Result<FontMetrics<'a>> {
        if self.face.catalogue_identity() != expected_catalogue {
            return Err(Error::new(
                ErrorCode::FontCatalogueMismatch,
                "associated font catalogue mismatch",
            )
            .with_detail(ErrorDetail::FontCatalogueMismatch {
                expected: expected_catalogue.clone(),
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
        // Directory admission is separate from permissive provider accessors.
        table(&font, Table::Head, true)?;
        let hhea = table(&font, Table::Hhea, false)?;
        let os2 = table(&font, Table::Os2, false)?;
        let mvar = table(&font, Table::Mvar, false)?;
        let head = font
            .head()
            .map_err(|_| table_error(Table::Head, TableError::Malformed))?;
        if head.version().major != 1 || head.version().minor != 0 {
            return Err(table_error(Table::Head, TableError::UnsupportedVersion));
        }
        if head.magic_number() != 0x5f0f3cf5 {
            return Err(table_error(Table::Head, TableError::Malformed));
        }
        let upem = head.units_per_em();
        if !(16..=16384).contains(&upem) {
            return Err(table_error(
                Table::Head,
                TableError::InvalidUnitsPerEm { value: upem },
            ));
        }
        validate_line_tables(&font, hhea, os2, true)?;
        let raw = skrifa::metrics::Metrics::new(&font, Size::unscaled(), LocationRef::default());
        let origin = line_origin(&font);
        let (mut used, mut used_count) = match origin {
            FontLineMetricSource::Os2Typographic => {
                ([*b"hasc", *b"hdsc", *b"hlgp", [0; 4], [0; 4]], 3)
            }
            FontLineMetricSource::Os2Windows => ([*b"hcla", *b"hcld", [0; 4], [0; 4], [0; 4]], 2),
            FontLineMetricSource::Hhea => ([[0; 4]; 5], 0),
        };
        if raw.strikeout.is_some() {
            used[used_count] = *b"stro";
            used[used_count + 1] = *b"strs";
            used_count += 2;
        }
        let deltas = variation::resolve(mvar, self, &used[..used_count])?;
        let mut raw_ascent = f64::from(raw.ascent);
        let mut raw_descent = f64::from(raw.descent);
        let mut raw_leading = f64::from(raw.leading);
        // Required signs belong to the resolved requested instance, not an
        // intermediate base which a valid variation can bring into range.
        finite(Field::Ascent, Phase::Unscaled, raw_ascent)?;
        finite(Field::Descent, Phase::Unscaled, raw_descent)?;
        finite(Field::Leading, Phase::Unscaled, raw_leading)?;
        (raw_ascent, raw_descent) = resolve_ad(raw_ascent, raw_descent, origin, &deltas)?;
        if origin == FontLineMetricSource::Os2Typographic {
            raw_leading += deltas.get(b"hlgp");
        }
        check_line(raw_ascent, raw_descent, raw_leading, Phase::Unscaled)?;
        let scale = f64::from(self.size) / f64::from(upem);
        let ascent = raw_ascent * scale;
        let descent = raw_descent * scale;
        let leading = raw_leading * scale;
        check_line(ascent, descent, leading, Phase::Scaled)?;
        let strike = raw.strikeout.map(|value| {
            (
                (f64::from(value.offset) + deltas.get(b"stro")) * scale,
                (f64::from(value.thickness) + deltas.get(b"strs")) * scale,
            )
        });
        let strikeout = normalize_strike(f64::from(self.size), ascent, strike)?;
        Ok(FontMetrics {
            instance: *self,
            units_per_em: upem,
            ascent,
            descent,
            leading,
            strikeout,
            line_metric_source: origin,
        })
    }
}

// Associate adjustments with the actual stored fields chosen by the retained
// Skrifa0.42.1 static selector (MIT OR Apache-2.0); see the tracked
// docs/font-metrics-provenance.md. Only provenance predicates are adapted; base
// numbers still come from the provider, and selection never reruns after delta.
/// Actual horizontal ascent/descent selection, independent of leading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontLineMetricSource {
    /// OS/2 typographic ascent/descent.
    Os2Typographic,
    /// hhea ascent/descent.
    Hhea,
    /// OS/2 Windows clipping ascent/descent.
    Os2Windows,
}
fn line_origin(font: &FontRef<'_>) -> FontLineMetricSource {
    use skrifa::raw::tables::os2::SelectionFlags;
    let os2 = font.os2().ok(); // Present tables were validated above.
    if os2.as_ref().is_some_and(|table| {
        table
            .fs_selection()
            .contains(SelectionFlags::USE_TYPO_METRICS)
    }) {
        return FontLineMetricSource::Os2Typographic;
    }
    if font
        .hhea()
        .ok()
        .is_some_and(|table| table.ascender().to_i16() != 0 || table.descender().to_i16() != 0)
    {
        return FontLineMetricSource::Hhea;
    }
    match os2 {
        Some(table) if table.s_typo_ascender() != 0 || table.s_typo_descender() != 0 => {
            FontLineMetricSource::Os2Typographic
        }
        Some(_) => FontLineMetricSource::Os2Windows,
        None => FontLineMetricSource::Hhea,
    }
}

// Both operations use this actual selected-field arithmetic; no inverse scaling.
fn resolve_ad(
    mut raw_ascent: f64,
    mut raw_descent: f64,
    origin: FontLineMetricSource,
    deltas: &variation::Deltas,
) -> Result<(f64, f64)> {
    match origin {
        FontLineMetricSource::Os2Typographic => {
            raw_ascent += deltas.get(b"hasc");
            raw_descent += deltas.get(b"hdsc");
        }
        FontLineMetricSource::Os2Windows => {
            raw_ascent += deltas.get(b"hcla");
            let stored_descent = -raw_descent + deltas.get(b"hcld");
            finite(Field::Descent, Phase::Unscaled, stored_descent)?;
            if stored_descent < 0.0 {
                return Err(numeric(
                    Field::Descent,
                    Phase::Unscaled,
                    Numeric::Positive,
                    -stored_descent,
                ));
            }
            raw_descent = -stored_descent;
            // The provider's hhea gap (or initial zero) stays unchanged.
        }
        FontLineMetricSource::Hhea => {}
    }
    Ok((raw_ascent, raw_descent))
}

// Only A/D are consumed here. Neither gap nor strike validation/variation is
// demanded, and Hhea selection does not consume unrelated MVAR storage.
fn fallback_ad(
    font: &FontRef<'_>,
    instance: &EffectiveFontInstance<'_>,
) -> Result<(f64, f64, FontLineMetricSource)> {
    let hhea = table(font, Table::Hhea, false)?;
    let os2 = table(font, Table::Os2, false)?;
    validate_line_tables(font, hhea, os2, false)?;
    let raw = skrifa::metrics::Metrics::new(font, Size::unscaled(), LocationRef::default());
    let origin = line_origin(font);
    let used: &[[u8; 4]] = match origin {
        FontLineMetricSource::Os2Typographic => &[*b"hasc", *b"hdsc"],
        FontLineMetricSource::Os2Windows => &[*b"hcla", *b"hcld"],
        FontLineMetricSource::Hhea => &[],
    };
    let mvar = if used.is_empty() {
        None
    } else {
        table(font, Table::Mvar, false)?
    };
    let deltas = variation::resolve(mvar, instance, used)?;
    let ascent = f64::from(raw.ascent);
    let descent = f64::from(raw.descent);
    finite(Field::Ascent, Phase::Unscaled, ascent)?;
    finite(Field::Descent, Phase::Unscaled, descent)?;
    let (ascent, descent) = resolve_ad(ascent, descent, origin, &deltas)?;
    check_line(ascent, descent, 0.0, Phase::Unscaled)?;
    Ok((ascent, descent, origin))
}

fn validate_line_tables(
    font: &FontRef<'_>,
    hhea: Option<TableData<'_>>,
    os2: Option<TableData<'_>>,
    complete: bool,
) -> Result<()> {
    if hhea.is_some() {
        let hhea = font
            .hhea()
            .map_err(|_| table_error(Table::Hhea, TableError::Malformed))?;
        if hhea.version().major != 1 || hhea.version().minor != 0 {
            return Err(table_error(Table::Hhea, TableError::UnsupportedVersion));
        }
    }
    if let Some(data) = os2 {
        let os2 = font
            .os2()
            .map_err(|_| table_error(Table::Os2, TableError::Malformed))?;
        let required = match os2.version() {
            0 => 78,
            1 => 86,
            2..=4 => 96,
            5 => 100,
            _ => return Err(table_error(Table::Os2, TableError::UnsupportedVersion)),
        };
        if data.len() < if complete { required } else { 78 } {
            return Err(table_error(Table::Os2, TableError::Malformed));
        }
    }
    if hhea.is_none() && os2.is_none() {
        return Err(Error::new(
            ErrorCode::InvalidFontMetrics,
            "missing usable line metric source",
        )
        .with_detail(ErrorDetail::MissingFontLineMetrics));
    }
    Ok(())
}

fn table<'a>(font: &FontRef<'a>, which: Table, required: bool) -> Result<Option<TableData<'a>>> {
    let tag = Tag::new(match which {
        Table::Head => b"head",
        Table::Hhea => b"hhea",
        Table::Os2 => b"OS/2",
        Table::Mvar => b"MVAR",
        Table::Base => b"BASE",
        Table::Maxp => b"maxp",
        Table::Hmtx => b"hmtx",
        Table::Vhea => b"vhea",
        Table::Vmtx => b"vmtx",
        Table::Loca => b"loca",
        Table::Glyf => b"glyf",
        Table::Vvar => b"VVAR",
    });
    let mut entries = font
        .table_directory()
        .table_records()
        .iter()
        .filter(|entry| entry.tag() == tag);
    if entries.next().is_none() {
        return if required {
            Err(table_error(which, TableError::Missing))
        } else {
            Ok(None)
        };
    }
    if entries.next().is_some() {
        return Err(table_error(which, TableError::Duplicate));
    }
    font.table_data(tag)
        .map(Some)
        .ok_or_else(|| table_error(which, TableError::Inaccessible))
}
fn table_error(table: Table, reason: TableError) -> Error {
    Error::new(
        ErrorCode::InvalidFontMetrics,
        "invalid used font metric table",
    )
    .with_detail(ErrorDetail::FontMetricTable { table, reason })
}
fn finite(field: Field, phase: Phase, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(numeric(field, phase, Numeric::NonFinite, value));
    }
    Ok(())
}
fn check_line(ascent: f64, descent: f64, leading: f64, phase: Phase) -> Result<()> {
    finite(Field::Ascent, phase, ascent)?;
    if ascent < 0.0 {
        return Err(numeric(Field::Ascent, phase, Numeric::Negative, ascent));
    }
    finite(Field::Descent, phase, descent)?;
    if descent > 0.0 {
        return Err(numeric(Field::Descent, phase, Numeric::Positive, descent));
    }
    finite(Field::Leading, phase, leading)
}
fn normalize_strike(
    size: f64,
    ascent: f64,
    strike: Option<(f64, f64)>,
) -> Result<StrikeoutMetrics> {
    let phase = Phase::StrikeoutNormalization;
    finite(Field::Size, phase, size)?;
    if size <= 0.0 {
        return Err(numeric(Field::Size, phase, Numeric::NonPositive, size));
    }
    finite(Field::Ascent, phase, ascent)?;
    if ascent < 0.0 {
        return Err(numeric(Field::Ascent, phase, Numeric::Negative, ascent));
    }
    let (center_offset, thickness) = match strike {
        Some((top, thickness)) if top.is_finite() && thickness.is_finite() && thickness > 0.0 => {
            (top - thickness / 2.0, thickness)
        }
        _ => (ascent / 2.0, size / 18.0),
    };
    finite(Field::StrikeoutCenter, phase, center_offset)?;
    finite(Field::StrikeoutThickness, phase, thickness)?;
    if thickness <= 0.0 {
        return Err(numeric(
            Field::StrikeoutThickness,
            phase,
            Numeric::NonPositive,
            thickness,
        ));
    }
    Ok(StrikeoutMetrics {
        center_offset,
        thickness,
    })
}
fn numeric(field: Field, phase: Phase, category: Numeric, value: f64) -> Error {
    Error::new(
        ErrorCode::InvalidFontMetrics,
        "invalid font metric numeric value",
    )
    .with_detail(ErrorDetail::InvalidFontMetric {
        field,
        phase,
        category,
        value,
    })
}

#[cfg(test)]
mod tests;
