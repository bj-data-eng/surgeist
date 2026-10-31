//! Glyph-free used upright em metrics, with an explicit coordinate reference.

use super::{
    EffectiveFontInstance, Field, FontBaselineAbsence, FontBaselineProvenance, FontBaselineScript,
    Phase, Table, TableError, baseline, finite, table, table_error, variation,
};
use crate::{Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, Result};
use skrifa::{FontRef, raw::FontData};

/// Coordinate reference of the used upright em edges and Central value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UprightMetricReference {
    /// Absolute horizontal font-design coordinates, positive right.
    FontDesignX,
    /// Signed distances from the declared nominal vertical pen, positive right.
    NominalVerticalPenX,
}

/// Actual source of the coupled used upright em metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UprightMetricProvenance {
    /// Existing qualified raw BASE/em resolution, retaining its derivation.
    RawBaseline(FontBaselineProvenance),
    /// Same-instance vhea1.1 em sides, including qualified MVAR effects.
    VheaTypographic,
    /// One em centered on the nominal vertical pen; no horizontal A/D transfer.
    SynthesizedUprightEmAtNominalPen,
}

/// Finite, same-instance used upright font metrics, scaled once to actual size.
///
/// This font-wide observation does not admit upright paragraph shaping. Qualified
/// fractional BASE values remain available independently of a positioning
/// provider's coordinate grid. No glyph zero, ink or average width is queried.
#[derive(Debug)]
pub struct UprightFontMetrics<'a> {
    instance: EffectiveFontInstance<'a>,
    script: FontBaselineScript,
    units_per_em: u16,
    reference: UprightMetricReference,
    central: f64,
    under: f64,
    over: f64,
    used_under: f64,
    used_over: f64,
    provenance: UprightMetricProvenance,
    base_script: Option<[u8; 4]>,
    base_absence: Option<FontBaselineAbsence>,
}

/// Glyph-free resolved facts retaining their actual design-unit reference.
#[derive(Debug)]
pub(crate) struct UnscaledUprightMetrics<'a> {
    instance: EffectiveFontInstance<'a>,
    script: FontBaselineScript,
    units_per_em: u16,
    reference: UprightMetricReference,
    central: f64,
    under: f64,
    over: f64,
    used_under: f64,
    used_over: f64,
    provenance: UprightMetricProvenance,
    base_script: Option<[u8; 4]>,
    base_absence: Option<FontBaselineAbsence>,
}
impl<'a> UnscaledUprightMetrics<'a> {
    pub(crate) const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    pub(crate) const fn script(&self) -> FontBaselineScript {
        self.script
    }
    pub(crate) const fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    pub(crate) const fn reference(&self) -> UprightMetricReference {
        self.reference
    }
    pub(crate) const fn central_coordinate_or_distance(&self) -> f64 {
        self.central
    }
    pub(crate) const fn em_under(&self) -> f64 {
        self.under
    }
    pub(crate) const fn em_over(&self) -> f64 {
        self.over
    }
    pub(crate) const fn used_under(&self) -> f64 {
        self.used_under
    }
    pub(crate) const fn used_over(&self) -> f64 {
        self.used_over
    }
    pub(crate) const fn provenance(&self) -> UprightMetricProvenance {
        self.provenance
    }
    pub(crate) const fn base_script(&self) -> Option<[u8; 4]> {
        self.base_script
    }
    pub(crate) const fn base_absence(&self) -> Option<FontBaselineAbsence> {
        self.base_absence
    }
}
impl<'a> UprightFontMetrics<'a> {
    /// Actual retained face, canonical size and complete coordinate loan.
    #[must_use]
    pub const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    /// Actual requested canonical Unicode script.
    #[must_use]
    pub const fn script(&self) -> FontBaselineScript {
        self.script
    }
    /// Actual design units per em.
    #[must_use]
    pub const fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    /// Reference shared by Central and both em sides.
    #[must_use]
    pub const fn reference(&self) -> UprightMetricReference {
        self.reference
    }
    /// Absolute Central coordinate or nominal-pen distance, as `reference` states.
    #[must_use]
    pub const fn central_coordinate_or_distance(&self) -> f64 {
        self.central
    }
    /// Signed em-under coordinate or distance, positive right.
    #[must_use]
    pub const fn em_under(&self) -> f64 {
        self.under
    }
    /// Signed em-over coordinate or distance, positive right.
    #[must_use]
    pub const fn em_over(&self) -> f64 {
        self.over
    }
    /// Signed em-under distance from the resolved Central.
    #[must_use]
    pub const fn used_under(&self) -> f64 {
        self.used_under
    }
    /// Signed em-over distance from the resolved Central.
    #[must_use]
    pub const fn used_over(&self) -> f64 {
        self.used_over
    }
    /// Actual native, derived or synthesized source of this coupled result.
    #[must_use]
    pub const fn provenance(&self) -> UprightMetricProvenance {
        self.provenance
    }
    /// Actual selected BASE script, including an authentic DFLT fallback.
    #[must_use]
    pub const fn base_script(&self) -> Option<[u8; 4]> {
        self.base_script
    }
    /// Qualified BASE absence retained through edge synthesis or native fallback.
    #[must_use]
    pub const fn base_absence(&self) -> Option<FontBaselineAbsence> {
        self.base_absence
    }
}

impl<'a> EffectiveFontInstance<'a> {
    /// Resolves same-instance used upright em sides and Central, without glyphs.
    ///
    /// Qualified raw VerticalX BASE/em evidence has precedence, including its
    /// CFF and fractional variation profiles. Otherwise vhea1.1 supplies its
    /// same-instance em sides with MVAR `vasc`/`vdsc`; true absence or valid
    /// vhea1.0 supplies a one-em interval centered on the nominal vertical pen.
    /// Horizontal A/D, line gap, strikeout and unrelated outlines are not read.
    /// This does not change raw named-baseline or nominal glyph observations.
    ///
    /// # Errors
    /// A foreign catalogue returns the original publication mismatch. Malformed
    /// consumed tables and unsupported consumed versions/references retain their
    /// precise metric/BASE errors and never select absence synthesis. Numeric
    /// failures retain the actual unscaled or scaled metric phase. A valid
    /// fractional result is not rejected for a separate provider's half-unit grid.
    pub fn upright_metrics(
        &self,
        expected_catalogue: &FontCatalogueIdentity,
        script: FontBaselineScript,
    ) -> Result<UprightFontMetrics<'a>> {
        let raw = self.unscaled_upright_metrics(expected_catalogue, script)?;
        let scale = f64::from(self.size) / f64::from(raw.units_per_em());
        let values = [
            raw.em_under(),
            raw.em_over(),
            raw.central_coordinate_or_distance(),
            raw.used_under(),
            raw.used_over(),
        ]
        .map(|value| value * scale);
        for (field, value) in upright_fields().into_iter().zip(values) {
            finite(field, Phase::Scaled, value)?;
        }
        let [under, over, central, used_under, used_over] = values;
        Ok(UprightFontMetrics {
            instance: *raw.instance(),
            script: raw.script(),
            units_per_em: raw.units_per_em(),
            reference: raw.reference(),
            central,
            under,
            over,
            used_under,
            used_over,
            provenance: raw.provenance(),
            base_script: raw.base_script(),
            base_absence: raw.base_absence(),
        })
    }

    pub(crate) fn unscaled_upright_metrics(
        &self,
        expected_catalogue: &FontCatalogueIdentity,
        script: FontBaselineScript,
    ) -> Result<UnscaledUprightMetrics<'a>> {
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
        let (upem, raw) = baseline::vertical_em(self, script)?;
        let (under, over, central, reference, provenance, base_script, base_absence) = match raw {
            baseline::VerticalEmOutcome::Available(em) => (
                em.under,
                em.over,
                em.central,
                UprightMetricReference::FontDesignX,
                UprightMetricProvenance::RawBaseline(em.provenance),
                em.script,
                em.absence,
            ),
            baseline::VerticalEmOutcome::Unavailable {
                value: absence,
                script: base_script,
            } => {
                let font =
                    FontRef::from_index(self.face.bytes(), self.face.index()).map_err(|_| {
                        Error::new(ErrorCode::FontLoadFailed, "invalid font face").with_detail(
                            ErrorDetail::InvalidFontFace {
                                index: self.face.index(),
                            },
                        )
                    })?;
                let native = vhea_em(&font, self)?;
                let (under, over, provenance) = native.map_or_else(
                    || {
                        (
                            -f64::from(upem) / 2.0,
                            f64::from(upem) / 2.0,
                            UprightMetricProvenance::SynthesizedUprightEmAtNominalPen,
                        )
                    },
                    |(under, over)| (under, over, UprightMetricProvenance::VheaTypographic),
                );
                (
                    under,
                    over,
                    under / 2.0 + over / 2.0,
                    UprightMetricReference::NominalVerticalPenX,
                    provenance,
                    base_script,
                    Some(absence.absence()),
                )
            }
        };
        let used_under = under - central;
        let used_over = over - central;
        let values = [under, over, central, used_under, used_over];
        for (field, value) in upright_fields().into_iter().zip(values) {
            finite(field, Phase::Unscaled, value)?;
        }
        Ok(UnscaledUprightMetrics {
            instance: *self,
            script,
            units_per_em: upem,
            reference,
            central,
            under,
            over,
            used_under,
            used_over,
            provenance,
            base_script,
            base_absence,
        })
    }

    /// Independently demanded signed upright line spacing in design units.
    /// No glyph inventory, outline, vmtx, BASE or horizontal metric is needed.
    pub(crate) fn upright_line_gap(&self, expected: &FontCatalogueIdentity) -> Result<f64> {
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
        let Some(data) = vhea_typographic(&font)? else {
            return Ok(0.0);
        };
        let deltas = variation::resolve(table(&font, Table::Mvar, false)?, self, &[*b"vlgp"])?;
        let gap = f64::from(
            data.read_at::<i16>(8)
                .map_err(|_| table_error(Table::Vhea, TableError::Malformed))?,
        ) + deltas.get(b"vlgp");
        finite(Field::Leading, Phase::Unscaled, gap)?;
        Ok(gap)
    }
}

const fn upright_fields() -> [Field; 5] {
    [
        Field::UprightEmUnder,
        Field::UprightEmOver,
        Field::UprightCentral,
        Field::UprightUsedUnder,
        Field::UprightUsedOver,
    ]
}

// Font-wide em sides consume the vhea header, not a glyph's vmtx/outline path.
// MVAR is demanded only when its actual vhea1.1 fields supply the result.
fn vhea_em(font: &FontRef<'_>, instance: &EffectiveFontInstance<'_>) -> Result<Option<(f64, f64)>> {
    let Some(data) = vhea_typographic(font)? else {
        return Ok(None);
    };
    let malformed = || table_error(Table::Vhea, TableError::Malformed);
    let deltas = variation::resolve(
        table(font, Table::Mvar, false)?,
        instance,
        &[*b"vasc", *b"vdsc"],
    )?;
    let over = f64::from(data.read_at::<i16>(4).map_err(|_| malformed())?) + deltas.get(b"vasc");
    let under = f64::from(data.read_at::<i16>(6).map_err(|_| malformed())?) + deltas.get(b"vdsc");
    Ok(Some((under, over)))
}

// Share the exact existing header qualification without demanding glyph tables.
fn vhea_typographic<'a>(font: &FontRef<'a>) -> Result<Option<FontData<'a>>> {
    let Some(data) = table(font, Table::Vhea, false)? else {
        return Ok(None);
    };
    let malformed = || table_error(Table::Vhea, TableError::Malformed);
    let version = data.read_at::<u32>(0).map_err(|_| malformed())?;
    if !matches!(version, 0x0001_0000 | 0x0001_1000) {
        return Err(table_error(Table::Vhea, TableError::UnsupportedVersion));
    }
    if data.len() < 36 || data.read_at::<i16>(32).map_err(|_| malformed())? != 0 {
        return Err(malformed());
    }
    if version == 0x0001_0000 {
        return Ok(None);
    }
    Ok(Some(data))
}
