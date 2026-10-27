use std::{fmt, sync::Arc};

use linebender_resource_handle::Blob;
use skrifa::{Tag, raw::TableProvider};

use crate::{Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, NumericRequirement, Result};

/// Strong actual font resource, collection face and associated catalogue publication.
///
/// Resource equality preserves Blob identity plus collection index. Catalogue
/// provenance is qualified separately; equal resources can have distinct publications.
#[derive(Clone, Debug)]
pub struct FontData {
    data: Blob<u8>,
    index: u32,
    catalogue: FontCatalogueIdentity,
}

impl PartialEq for FontData {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.index == other.index
    }
}

impl FontData {
    /// Retains the actual admitted resource without copying its byte backing.
    pub(crate) fn from_blob(data: Blob<u8>, index: u32, catalogue: FontCatalogueIdentity) -> Self {
        Self {
            data,
            index,
            catalogue,
        }
    }

    /// Original whole-resource bytes, without copying.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.data.as_ref()
    }
    /// Collection face index.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
    /// Actual retained publication, independent of resource equality.
    #[must_use]
    pub const fn catalogue_identity(&self) -> &FontCatalogueIdentity {
        &self.catalogue
    }
    /// Borrows this same resource, face and publication.
    #[must_use]
    pub fn face(&self) -> FontFace<'_> {
        FontFace::new(&self.data, self.index, &self.catalogue)
    }
    /// Selects an explicit unmodified face and complete canonical coordinates.
    /// This does not match an unresolved Style or silently discard faux synthesis.
    pub fn unsynthesized_instance<'a>(
        &'a self,
        size: f32,
        normalized_coords: &'a [i16],
    ) -> Result<EffectiveFontInstance<'a>> {
        EffectiveFontInstance::qualify(self.face(), size, normalized_coords)
    }
}

/// A borrowed actual resource, collection face and associated publication.
#[derive(Clone, Copy)]
pub struct FontFace<'a> {
    data: &'a Blob<u8>,
    index: u32,
    catalogue: &'a FontCatalogueIdentity,
}

impl fmt::Debug for FontFace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontFace")
            .field("index", &self.index())
            .field("bytes", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl<'a> FontFace<'a> {
    pub(crate) const fn new(
        data: &'a Blob<u8>,
        index: u32,
        catalogue: &'a FontCatalogueIdentity,
    ) -> Self {
        Self {
            data,
            index,
            catalogue,
        }
    }

    /// Actual catalogue publication under which this face was associated.
    #[must_use]
    pub const fn catalogue_identity(&self) -> &'a FontCatalogueIdentity {
        self.catalogue
    }

    /// Retains the actual strong resource and publication without copying bytes.
    #[must_use]
    pub fn to_owned(&self) -> FontData {
        FontData {
            data: self.data.clone(),
            index: self.index,
            catalogue: self.catalogue.clone(),
        }
    }

    /// Whole original resource bytes, borrowed without copying.
    #[must_use]
    pub fn bytes(&self) -> &'a [u8] {
        self.data.as_ref()
    }

    /// Actual face index within that resource.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Compares the actual strong allocation and collection index, not IDs or content.
    #[must_use]
    pub fn is_same_face(&self, other: &Self) -> bool {
        let (left, _) = self.data.clone().into_raw_parts();
        let (right, _) = other.data.clone().into_raw_parts();
        self.index() == other.index() && Arc::ptr_eq(&left, &right)
    }
}

/// Concrete executable synthesis in canonical glyph-local logical units.
///
/// The x-shear is dimensionless. Embolden is a positive local-x copy distance;
/// zero means no copy. Neither value changes advances or line metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontSynthesis {
    x_shear: f32,
    embolden: f32,
}

impl FontSynthesis {
    /// Resolves actual Fontique degree-angle/selected-bold facts once at shaping size.
    pub(crate) fn resolve(
        size: f32,
        skew_degrees: Option<f32>,
        selected_bold: bool,
    ) -> Result<Self> {
        use crate::engine::text_numeric_error;
        use crate::error::{TextNumericError as Numeric, TextNumericPhase as Phase};
        if !size.is_finite() || size <= 0.0 {
            return Err(text_numeric_error(
                "synthesis font size",
                Phase::FontSynthesis,
                if size.is_finite() {
                    Numeric::OutOfRange
                } else {
                    Numeric::NonFinite
                },
                f64::from(size),
            ));
        }
        let x_shear = match skew_degrees {
            None => 0.0,
            Some(degrees) => {
                if !degrees.is_finite() {
                    return Err(text_numeric_error(
                        "synthesis skew degrees",
                        Phase::FontSynthesis,
                        Numeric::NonFinite,
                        f64::from(degrees),
                    ));
                }
                let shear = degrees.to_radians().tan();
                if !shear.is_finite() {
                    return Err(text_numeric_error(
                        "synthesis x shear",
                        Phase::FontSynthesis,
                        Numeric::NonFinite,
                        f64::from(shear),
                    ));
                }
                if degrees != 0.0 && shear == 0.0 {
                    return Err(text_numeric_error(
                        "synthesis x shear",
                        Phase::FontSynthesis,
                        Numeric::NonZeroLost,
                        f64::from(degrees),
                    ));
                }
                if shear == 0.0 { 0.0 } else { shear }
            }
        };
        let embolden = if selected_bold {
            let amount = size / 36.0_f32;
            if !amount.is_finite() || amount <= 0.0 {
                return Err(text_numeric_error(
                    "synthesis embolden",
                    Phase::FontSynthesis,
                    if amount.is_finite() {
                        Numeric::PositiveLost
                    } else {
                        Numeric::NonFinite
                    },
                    f64::from(amount),
                ));
            }
            amount
        } else {
            0.0
        };
        Ok(Self { x_shear, embolden })
    }

    /// Executable dimensionless x-shear in the font's y-up coordinate system.
    #[must_use]
    pub const fn x_shear(self) -> f32 {
        self.x_shear
    }
    /// Original-plus-positive-local-x copy distance at actual canonical size.
    #[must_use]
    pub const fn embolden(self) -> f32 {
        self.embolden
    }
}

/// A borrowed complete canonical font instance with executable synthesis.
#[derive(Clone, Copy, Debug)]
pub struct EffectiveFontInstance<'a> {
    face: FontFace<'a>,
    size: f32,
    axes: usize,
    stored: &'a [i16],
    synthesis: FontSynthesis,
}

impl<'a> EffectiveFontInstance<'a> {
    pub(crate) fn qualify(face: FontFace<'a>, size: f32, stored: &'a [i16]) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(
                Error::new(ErrorCode::InvalidFontInstance, "invalid shaping size").with_detail(
                    ErrorDetail::InvalidNumericField {
                        field: "shaping font size",
                        value: f64::from(size),
                        requirement: NumericRequirement::FiniteGreaterThanZero,
                    },
                ),
            );
        }
        let axes = axis_count(face)?;
        if !stored.is_empty() && stored.len() != axes {
            return Err(Error::new(
                ErrorCode::InvalidFontInstance,
                "normalized coordinates do not cover the actual axes",
            )
            .with_detail(ErrorDetail::FontCoordinateCount {
                expected: axes,
                actual: stored.len(),
            }));
        }
        for (axis, &bits) in stored.iter().enumerate() {
            if !(-16384..=16384).contains(&bits) {
                return Err(Error::new(
                    ErrorCode::InvalidFontInstance,
                    "noncanonical normalized font coordinate",
                )
                .with_detail(ErrorDetail::FontCoordinate { axis, bits }));
            }
        }
        Ok(Self {
            face,
            size,
            axes,
            stored,
            synthesis: FontSynthesis::default(),
        })
    }

    pub(crate) fn qualify_synthesized(
        face: FontFace<'a>,
        size: f32,
        stored: &'a [i16],
        synthesis: FontSynthesis,
    ) -> Result<Self> {
        let mut instance = Self::qualify(face, size, stored)?;
        instance.synthesis = synthesis;
        Ok(instance)
    }

    /// Concrete synthesis retained by this actual effective instance.
    #[must_use]
    pub const fn synthesis(&self) -> FontSynthesis {
        self.synthesis
    }
    /// Executable dimensionless x-shear.
    #[must_use]
    pub const fn x_shear(&self) -> f32 {
        self.synthesis.x_shear()
    }
    /// Actual local-positive-x emboldening displacement.
    #[must_use]
    pub const fn embolden(&self) -> f32 {
        self.synthesis.embolden()
    }

    /// The actual borrowed face used by this instance.
    #[must_use]
    pub const fn face(&self) -> FontFace<'a> {
        self.face
    }

    /// Actual canonical instance size.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.size
    }

    /// Complete actual variation-axis count; zero only for a proven static face.
    #[must_use]
    pub const fn axis_count(&self) -> usize {
        self.axes
    }

    /// Complete normalized F2Dot14 bits in actual axis order, without allocation.
    pub fn normalized_coords(&self) -> impl ExactSizeIterator<Item = i16> + Clone + '_ {
        (0..self.axes).map(|axis| {
            if self.stored.is_empty() {
                0
            } else {
                self.stored[axis]
            }
        })
    }
}

fn axis_count(face: FontFace<'_>) -> Result<usize> {
    let font = skrifa::FontRef::from_index(face.bytes(), face.index()).map_err(|_| malformed())?;
    let tag = Tag::new(b"fvar");
    let mut records = font
        .table_directory()
        .table_records()
        .iter()
        .filter(|record| record.tag() == tag);
    if records.next().is_none() {
        return Ok(0);
    }
    if records.next().is_some() {
        return Err(malformed());
    }
    // Table presence is established separately: inaccessible/truncated data must
    // never be reclassified as a static font by a permissive metadata accessor.
    let data = font.table_data(tag).ok_or_else(malformed)?;
    let fvar = font.fvar().map_err(|_| malformed())?;
    let count = usize::from(fvar.axis_count());
    let offset =
        usize::try_from(fvar.axis_instance_arrays_offset().to_u32()).map_err(|_| malformed())?;
    let instance_size = usize::from(fvar.instance_size());
    let required_instance = count.checked_mul(4).and_then(|n| n.checked_add(4));
    let axes_end = count.checked_mul(20).and_then(|n| offset.checked_add(n));
    let table_end = axes_end.and_then(|n| {
        usize::from(fvar.instance_count())
            .checked_mul(instance_size)
            .and_then(|instances| n.checked_add(instances))
    });
    if fvar.version().major != 1
        || fvar.version().minor != 0
        || data.as_bytes().get(6..8) != Some(&[0, 2])
        || count == 0
        || fvar.axis_size() != 20
        || offset < 16
        || !required_instance.is_some_and(|n| instance_size == n || instance_size == n + 2)
        || !table_end.is_some_and(|end| end <= data.len())
    {
        return Err(malformed());
    }
    let axes = fvar.axes().map_err(|_| malformed())?;
    if axes.len() != count {
        return Err(malformed());
    }
    for (index, axis) in axes.iter().enumerate() {
        if axis.min_value() > axis.default_value()
            || axis.default_value() > axis.max_value()
            || axes[..index]
                .iter()
                .any(|previous| previous.axis_tag() == axis.axis_tag())
        {
            return Err(malformed());
        }
    }
    Ok(count)
}

fn malformed() -> Error {
    Error::new(
        ErrorCode::InvalidFontInstance,
        "malformed actual font variation metadata",
    )
    .with_detail(ErrorDetail::MalformedFontVariations)
}

mod metrics;
mod outline;
pub use metrics::{
    FontBaseline, FontBaselineAbsence, FontBaselineAxis, FontBaselineMeasurement,
    FontBaselineOutcome, FontBaselineProvenance, FontBaselineScript, FontBaselineUnavailable,
    FontBaselineUnavailableReason, FontBaselineValue, FontLineMetricSource, FontMetrics,
    StrikeoutMetrics, VerticalGlyphMetricProvenance, VerticalGlyphMetrics,
};
pub use outline::{FontOutlineCapability, FontOutlineError, GlyphOutlineGeometry};

#[cfg(test)]
mod synthesis_tests {
    use super::*;

    #[test]
    fn executable_synthesis_resolves_once_in_canonical_font_units() {
        assert_eq!(
            FontSynthesis::resolve(36.0, None, true).unwrap().embolden(),
            1.0
        );
        assert_eq!(
            FontSynthesis::resolve(18.0, None, true).unwrap().embolden(),
            0.5
        );
        for (degrees, expected) in [(45.0, 1.0), (-45.0, -1.0)] {
            let synthesis = FontSynthesis::resolve(36.0, Some(degrees), true).unwrap();
            assert!((synthesis.x_shear() - expected).abs() < 0.000001);
            assert_eq!(synthesis.embolden(), 1.0);
        }
        assert_eq!(
            FontSynthesis::resolve(20.0, None, false).unwrap(),
            FontSynthesis::default()
        );
        let error = FontSynthesis::resolve(f32::from_bits(1), None, true).unwrap_err();
        assert!(matches!(
            error.detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                phase: crate::TextNumericPhase::FontSynthesis,
                category: crate::TextNumericError::PositiveLost,
                ..
            })
        ));
        for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(FontSynthesis::resolve(size, None, false).is_err());
        }
        assert!(FontSynthesis::resolve(20.0, Some(f32::NAN), false).is_err());
    }

    #[test]
    fn synthesis_keeps_actual_resource_and_metric_operation_unchanged() {
        let mut system = crate::System::new(crate::SystemOptions {
            font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        let bytes = include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf");
        system.register_fonts(bytes.to_vec()).unwrap();
        let layout = system
            .layout(
                crate::Source::new("X"),
                crate::Style {
                    font: crate::Font::new().family("Ahem"),
                    size: 36.0,
                    ..crate::Style::default()
                },
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        let font = layout.run_views().next().unwrap().face().to_owned();
        drop(layout);
        let plain = font.unsynthesized_instance(36.0, &[]).unwrap();
        let synthesized = EffectiveFontInstance::qualify_synthesized(
            font.face(),
            36.0,
            &[],
            FontSynthesis::resolve(36.0, Some(45.0), true).unwrap(),
        )
        .unwrap();
        let plain_metrics = plain.metrics(system.font_catalogue_identity()).unwrap();
        let synthesized_metrics = synthesized
            .metrics(system.font_catalogue_identity())
            .unwrap();
        assert_eq!(
            (
                plain_metrics.ascent(),
                plain_metrics.descent(),
                plain_metrics.leading(),
                plain_metrics.strikeout()
            ),
            (
                synthesized_metrics.ascent(),
                synthesized_metrics.descent(),
                synthesized_metrics.leading(),
                synthesized_metrics.strikeout()
            )
        );
        assert!(plain.face().is_same_face(&synthesized.face()));
        assert_eq!(plain.embolden(), 0.0);
        assert_eq!(synthesized.embolden(), 1.0);
        drop(system);
        assert_eq!(synthesized.face().bytes(), bytes);
    }
}
