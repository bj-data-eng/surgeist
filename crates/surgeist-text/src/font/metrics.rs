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

/// Static font metrics retaining the actual resource and coordinate loan.
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
    /// Resolves glyph-free static metrics against the captured catalogue basis.
    ///
    /// Historical resources work under their retained original publication.
    /// fvar/MVAR metrics refuse, including all-zero defaults. Broader variation
    /// readback remains available. A usable provider strike top is normalized to
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
        if self.axes != 0 || mvar.is_some() {
            return Err(Error::new(
                ErrorCode::UnsupportedFeature,
                "variable font metrics are not supported",
            )
            .with_detail(ErrorDetail::UnsupportedVariableFontMetrics {
                fvar: self.axes != 0,
                mvar: mvar.is_some(),
            }));
        }
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
            if data.len() < required {
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
        let raw = skrifa::metrics::Metrics::new(&font, Size::unscaled(), LocationRef::default());
        let raw_ascent = f64::from(raw.ascent);
        let raw_descent = f64::from(raw.descent);
        let raw_leading = f64::from(raw.leading);
        check_line(raw_ascent, raw_descent, raw_leading, Phase::Unscaled)?;
        let scale = f64::from(self.size) / f64::from(upem);
        let ascent = raw_ascent * scale;
        let descent = raw_descent * scale;
        let leading = raw_leading * scale;
        check_line(ascent, descent, leading, Phase::Scaled)?;
        let strike = raw.strikeout.map(|value| {
            (
                f64::from(value.offset) * scale,
                f64::from(value.thickness) * scale,
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
        })
    }
}

fn table<'a>(font: &FontRef<'a>, which: Table, required: bool) -> Result<Option<TableData<'a>>> {
    let tag = Tag::new(match which {
        Table::Head => b"head",
        Table::Hhea => b"hhea",
        Table::Os2 => b"OS/2",
        Table::Mvar => b"MVAR",
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
