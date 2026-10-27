//! Unhinted named coordinates; no baseline choice changes shaping or line metrics.

use super::{
    EffectiveFontInstance, Field, Numeric, Phase, Table, TableError, finite, numeric, table,
    table_error,
};
use crate::{Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, Result};
use icu_properties::{PropertyNamesShort, PropertyParser, props::Script};
use skrifa::{
    FontRef, Tag,
    raw::{
        FontRead, TableProvider,
        tables::base::{Axis, Base, BaseCoord},
    },
};

/// Font-design coordinate reference, before page placement or sideways rotation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontBaselineAxis {
    /// Design Y=0; positive coordinates point up.
    HorizontalY,
    /// Design X=0; positive coordinates point right.
    VerticalX,
}
/// Independently requested named baseline.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontBaseline {
    Alphabetic,
    Central,
}
/// Canonical Unicode script, rather than a caller-selected OpenType table tag.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FontBaselineScript {
    iso: [u8; 4],
}
impl FontBaselineScript {
    /// Accepts a recognized Unicode ISO15924 short name and retains its canonical name.
    pub fn from_iso15924(iso: [u8; 4]) -> Option<Self> {
        Self::from_script(PropertyParser::<Script>::new().get_strict_utf8(&iso)?)
    }
    pub(crate) fn from_script(script: Script) -> Option<Self> {
        let name = PropertyNamesShort::<Script>::new().get(script)?;
        Some(Self {
            iso: name.as_bytes().try_into().ok()?,
        })
    }
    #[must_use]
    pub const fn iso15924(self) -> [u8; 4] {
        self.iso
    }
    fn legacy_tag(self) -> [u8; 4] {
        // This pure provider mapping supplies the legacy tag last, without
        // ScriptList::select's GSUB dflt/latn font-selection fallbacks.
        let tags = skrifa::raw::tables::layout::ScriptTags::from_unicode(Tag::new(&self.iso));
        tags.as_slice()
            .last()
            .expect("Unicode script mapping always includes its legacy tag")
            .to_be_bytes()
    }
}
/// Qualified absence, never a malformed present record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontBaselineAbsence {
    BaseTable,
    Axis,
    Script,
    Name,
}
/// Source of the finite value, under the explicitly unhinted CSS-midpoint profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontBaselineProvenance {
    BaseCoordinate,
    BaseEdgesCssMidpoint,
    IdeographicBottomOneEmCssMidpoint,
    IdeographicTopOneEmCssMidpoint,
    VerticalTopZeroOriginCssMidpoint,
    InstanceMetricsCssMidpoint,
    HorizontalAlphabeticZero,
    HorizontalEmBoxCssMidpoint,
}
/// A finite size-scaled design coordinate and its coupled provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontBaselineValue {
    coordinate: f64,
    provenance: FontBaselineProvenance,
    base_script: Option<[u8; 4]>,
    absence: Option<FontBaselineAbsence>,
}
impl FontBaselineValue {
    #[must_use]
    pub const fn coordinate(self) -> f64 {
        self.coordinate
    }
    #[must_use]
    pub const fn provenance(self) -> FontBaselineProvenance {
        self.provenance
    }
    #[must_use]
    pub const fn base_script(self) -> Option<[u8; 4]> {
        self.base_script
    }
    #[must_use]
    pub const fn absence(self) -> Option<FontBaselineAbsence> {
        self.absence
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontBaselineUnavailableReason {
    VerticalAlphabetic,
    VerticalEmBoxReference,
}
/// Missing vertical capability with the actual queried-axis absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontBaselineUnavailable {
    reason: FontBaselineUnavailableReason,
    absence: FontBaselineAbsence,
}
impl FontBaselineUnavailable {
    #[must_use]
    pub const fn reason(self) -> FontBaselineUnavailableReason {
        self.reason
    }
    #[must_use]
    pub const fn absence(self) -> FontBaselineAbsence {
        self.absence
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontBaselineOutcome {
    Available(FontBaselineValue),
    Unavailable(FontBaselineUnavailable),
}
/// One named observation from an actual retained instance, with no glyph rasterization.
///
/// Coordinates use the axis's font-design origin and are unhinted. Static Device
/// payloads are validated but ppem corrections are omitted. A CSS midpoint keeps
/// fractional units; it deliberately does not use OpenType's whole-unit rounding.
#[derive(Debug)]
pub struct FontBaselineMeasurement<'a> {
    instance: EffectiveFontInstance<'a>,
    script: FontBaselineScript,
    axis: FontBaselineAxis,
    name: FontBaseline,
    outcome: FontBaselineOutcome,
}
impl<'a> FontBaselineMeasurement<'a> {
    #[must_use]
    pub const fn instance(&self) -> &EffectiveFontInstance<'a> {
        &self.instance
    }
    #[must_use]
    pub const fn script(&self) -> FontBaselineScript {
        self.script
    }
    #[must_use]
    pub const fn axis(&self) -> FontBaselineAxis {
        self.axis
    }
    #[must_use]
    pub const fn name(&self) -> FontBaseline {
        self.name
    }
    #[must_use]
    pub const fn outcome(&self) -> &FontBaselineOutcome {
        &self.outcome
    }
}

impl<'a> EffectiveFontInstance<'a> {
    /// Measures one unhinted named baseline, independently of other baseline availability.
    ///
    /// Actual BASE legacy-script/DFLT data precedes the bounded synthesis rules.
    /// This neither matches a font nor creates vertical shaping/placement support.
    /// Vertical alphabetic requires its actual romn record. Vertical central
    /// requires vertical em edges or a horizontal ideo em-box witness; no CJK
    /// classifier, vhea reference, advance or ink-box inference is supplied.
    pub fn baseline(
        &self,
        expected: &FontCatalogueIdentity,
        script: FontBaselineScript,
        axis: FontBaselineAxis,
        name: FontBaseline,
    ) -> Result<FontBaselineMeasurement<'a>> {
        if self.face().catalogue_identity() != expected {
            return Err(Error::new(
                ErrorCode::FontCatalogueMismatch,
                "associated font catalogue mismatch",
            )
            .with_detail(ErrorDetail::FontCatalogueMismatch {
                expected: expected.clone(),
                actual: self.face().catalogue_identity().clone(),
            }));
        }
        let reader = Reader::new(self)?;
        let outcome = reader.resolve(script, axis, name, expected)?;
        Ok(FontBaselineMeasurement {
            instance: *self,
            script,
            axis,
            name,
            outcome,
        })
    }
}

#[derive(Clone, Copy)]
struct Lookup {
    coordinate: Option<f64>,
    script: Option<[u8; 4]>,
    absence: Option<FontBaselineAbsence>,
}
impl Lookup {
    fn absent(absence: FontBaselineAbsence, script: Option<[u8; 4]>) -> Self {
        Self {
            coordinate: None,
            script,
            absence: Some(absence),
        }
    }
}
// One unscaled owner for the raw observation and the used upright adapter.
// Derivation provenance identifies which edge is authored or synthesized.
pub(super) struct VerticalEm {
    pub under: f64,
    pub over: f64,
    pub central: f64,
    pub provenance: FontBaselineProvenance,
    pub script: Option<[u8; 4]>,
    pub absence: Option<FontBaselineAbsence>,
}
pub(super) enum VerticalEmOutcome {
    Available(VerticalEm),
    Unavailable {
        value: FontBaselineUnavailable,
        script: Option<[u8; 4]>,
    },
}
pub(super) fn vertical_em(
    instance: &EffectiveFontInstance<'_>,
    script: FontBaselineScript,
) -> Result<(u16, VerticalEmOutcome)> {
    let reader = Reader::new(instance)?;
    Ok((reader.upem, reader.vertical_em(script)?))
}

struct Reader<'instance, 'font> {
    instance: &'instance EffectiveFontInstance<'font>,
    font: FontRef<'instance>,
    base: Option<Base<'instance>>,
    upem: u16,
}
impl<'instance, 'font> Reader<'instance, 'font> {
    fn new(instance: &'instance EffectiveFontInstance<'font>) -> Result<Self> {
        let font =
            FontRef::from_index(instance.face.bytes(), instance.face.index()).map_err(|_| {
                Error::new(ErrorCode::FontLoadFailed, "invalid font face").with_detail(
                    ErrorDetail::InvalidFontFace {
                        index: instance.face.index(),
                    },
                )
            })?;
        table(&font, Table::Head, true)?;
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
        let base = table(&font, Table::Base, false)?
            .map(|data| {
                let major = data
                    .read_at::<u16>(0)
                    .map_err(|_| refuse(TableError::Malformed))?;
                let minor = data
                    .read_at::<u16>(2)
                    .map_err(|_| refuse(TableError::Malformed))?;
                if major != 1 || minor > 1 {
                    return Err(refuse(TableError::UnsupportedVersion));
                }
                Base::read(data).map_err(|_| refuse(TableError::Malformed))
            })
            .transpose()?;
        Ok(Self {
            instance,
            font,
            base,
            upem,
        })
    }
    fn available(
        &self,
        coordinate: f64,
        provenance: FontBaselineProvenance,
        script: Option<[u8; 4]>,
        absence: Option<FontBaselineAbsence>,
    ) -> Result<FontBaselineOutcome> {
        finite(Field::BaselineCoordinate, Phase::Unscaled, coordinate)?;
        let scaled = coordinate * (f64::from(self.instance.font_size()) / f64::from(self.upem));
        finite(Field::BaselineCoordinate, Phase::Scaled, scaled)?;
        Ok(FontBaselineOutcome::Available(FontBaselineValue {
            coordinate: scaled,
            provenance,
            base_script: script,
            absence,
        }))
    }
    fn midpoint(&self, a: f64, b: f64) -> Result<f64> {
        let result = a / 2.0 + b / 2.0;
        finite(Field::BaselineMidpoint, Phase::Unscaled, result)?;
        Ok(result)
    }
    fn resolve(
        &self,
        script: FontBaselineScript,
        axis: FontBaselineAxis,
        name: FontBaseline,
        expected: &FontCatalogueIdentity,
    ) -> Result<FontBaselineOutcome> {
        use FontBaselineAxis::{HorizontalY, VerticalX};
        use FontBaselineProvenance as P;
        if name == FontBaseline::Alphabetic {
            let alpha = self.lookup(script, axis, *b"romn")?;
            if let Some(value) = alpha.coordinate {
                return self.available(value, P::BaseCoordinate, alpha.script, None);
            }
            return if axis == HorizontalY {
                self.available(
                    0.0,
                    P::HorizontalAlphabeticZero,
                    alpha.script,
                    alpha.absence,
                )
            } else {
                Ok(unavailable(
                    FontBaselineUnavailableReason::VerticalAlphabetic,
                    alpha,
                ))
            };
        }
        if axis == VerticalX {
            return match self.vertical_em(script)? {
                VerticalEmOutcome::Available(em) => {
                    self.available(em.central, em.provenance, em.script, em.absence)
                }
                VerticalEmOutcome::Unavailable { value, .. } => {
                    Ok(FontBaselineOutcome::Unavailable(value))
                }
            };
        }
        let bottom = self.lookup(script, axis, *b"ideo")?;
        let top = self.lookup(script, axis, *b"idtp")?;
        match (bottom.coordinate, top.coordinate) {
            (Some(under), Some(over)) => self.available(
                self.midpoint(under, over)?,
                P::BaseEdgesCssMidpoint,
                bottom.script,
                None,
            ),
            (Some(value), None) => self.available(
                self.midpoint(value, value + f64::from(self.upem))?,
                P::IdeographicBottomOneEmCssMidpoint,
                bottom.script,
                top.absence,
            ),
            (None, Some(value)) => self.available(
                self.midpoint(value - f64::from(self.upem), value)?,
                P::IdeographicTopOneEmCssMidpoint,
                bottom.script,
                bottom.absence,
            ),
            (None, None) => {
                let metrics = self.instance.metrics(expected)?;
                let scaled = self.midpoint(metrics.ascent(), metrics.descent())?;
                Ok(FontBaselineOutcome::Available(FontBaselineValue {
                    coordinate: scaled,
                    provenance: P::InstanceMetricsCssMidpoint,
                    base_script: bottom.script,
                    absence: bottom.absence,
                }))
            }
        }
    }
    fn vertical_em(&self, script: FontBaselineScript) -> Result<VerticalEmOutcome> {
        use FontBaselineAxis::{HorizontalY, VerticalX};
        use FontBaselineProvenance as P;
        let bottom = self.lookup(script, VerticalX, *b"ideo")?;
        let top = self.lookup(script, VerticalX, *b"idtp")?;
        if let Some(value) = bottom.coordinate
            && value != 0.0
        {
            return Err(numeric(
                Field::BaselineIdeographicOrigin,
                Phase::Unscaled,
                Numeric::NonZero,
                value,
            ));
        }
        let (under, over, provenance, base_script, absence) =
            match (bottom.coordinate, top.coordinate) {
                (Some(under), Some(over)) => {
                    (under, over, P::BaseEdgesCssMidpoint, bottom.script, None)
                }
                (Some(_), None) => (
                    0.0,
                    f64::from(self.upem),
                    P::IdeographicBottomOneEmCssMidpoint,
                    bottom.script,
                    top.absence,
                ),
                (None, Some(over)) => (
                    0.0,
                    over,
                    P::VerticalTopZeroOriginCssMidpoint,
                    bottom.script,
                    bottom.absence,
                ),
                (None, None) => {
                    let horizontal = self.lookup(script, HorizontalY, *b"ideo")?;
                    if horizontal.coordinate.is_none() {
                        return Ok(VerticalEmOutcome::Unavailable {
                            value: FontBaselineUnavailable {
                                reason: FontBaselineUnavailableReason::VerticalEmBoxReference,
                                absence: bottom
                                    .absence
                                    .expect("unavailable lookup retains its absence"),
                            },
                            script: bottom.script,
                        });
                    }
                    (
                        0.0,
                        f64::from(self.upem),
                        P::HorizontalEmBoxCssMidpoint,
                        horizontal.script,
                        bottom.absence,
                    )
                }
            };
        Ok(VerticalEmOutcome::Available(VerticalEm {
            under,
            over,
            central: self.midpoint(under, over)?,
            provenance,
            script: base_script,
            absence,
        }))
    }
    fn lookup(
        &self,
        script: FontBaselineScript,
        axis: FontBaselineAxis,
        name: [u8; 4],
    ) -> Result<Lookup> {
        let Some(base) = &self.base else {
            return Ok(Lookup::absent(FontBaselineAbsence::BaseTable, None));
        };
        let axis = match axis {
            FontBaselineAxis::HorizontalY => base.horiz_axis(),
            FontBaselineAxis::VerticalX => base.vert_axis(),
        };
        let Some(axis) = axis else {
            return Ok(Lookup::absent(FontBaselineAbsence::Axis, None));
        };
        self.axis_lookup(
            base,
            axis.map_err(|_| refuse(TableError::Malformed))?,
            script,
            name,
        )
    }
    fn axis_lookup(
        &self,
        base: &Base<'_>,
        axis: Axis<'_>,
        script: FontBaselineScript,
        name: [u8; 4],
    ) -> Result<Lookup> {
        if axis.base_script_list_offset().to_u32() == 0 {
            return Err(refuse(TableError::Malformed));
        }
        let scripts = axis
            .base_script_list()
            .map_err(|_| refuse(TableError::Malformed))?;
        let records = scripts.base_script_records();
        if records.len() != usize::from(scripts.base_script_count()) {
            return Err(refuse(TableError::Malformed));
        }
        if records
            .windows(2)
            .any(|pair| pair[0].base_script_tag() >= pair[1].base_script_tag())
        {
            return Err(refuse(TableError::BaselineScriptOrder));
        }
        let tags = axis
            .base_tag_list()
            .transpose()
            .map_err(|_| refuse(TableError::Malformed))?;
        if let Some(tags) = &tags {
            if tags.baseline_tags().len() != usize::from(tags.base_tag_count()) {
                return Err(refuse(TableError::Malformed));
            }
            if tags
                .baseline_tags()
                .windows(2)
                .any(|pair| pair[0].get() >= pair[1].get())
            {
                return Err(refuse(TableError::BaselineTagOrder));
            }
        }
        let legacy = Tag::new(&script.legacy_tag());
        let selected = records
            .iter()
            .find(|row| row.base_script_tag() == legacy)
            .or_else(|| {
                records
                    .iter()
                    .find(|row| row.base_script_tag() == Tag::new(b"DFLT"))
            });
        let Some(selected) = selected else {
            return Ok(Lookup::absent(FontBaselineAbsence::Script, None));
        };
        let selected_tag = selected.base_script_tag().to_be_bytes();
        if selected.base_script_offset().to_u32() == 0 {
            return Err(refuse(TableError::Malformed));
        }
        let script = selected
            .base_script(scripts.offset_data())
            .map_err(|_| refuse(TableError::Malformed))?;
        let values = script
            .base_values()
            .transpose()
            .map_err(|_| refuse(TableError::Malformed))?;
        let Some(values) = values else {
            return Ok(Lookup::absent(
                FontBaselineAbsence::Name,
                Some(selected_tag),
            ));
        };
        let tag_count = tags.as_ref().map_or(0, |tags| tags.base_tag_count());
        if values.base_coord_count() != tag_count {
            return Err(refuse(TableError::BaselineValueCount {
                expected: tag_count,
                actual: values.base_coord_count(),
            }));
        }
        if values.default_baseline_index() >= values.base_coord_count() {
            return Err(refuse(TableError::BaselineDefaultIndex {
                index: values.default_baseline_index(),
                count: values.base_coord_count(),
            }));
        }
        let Some(tags) = tags else {
            return Ok(Lookup::absent(
                FontBaselineAbsence::Name,
                Some(selected_tag),
            ));
        };
        let Some(index) = tags
            .baseline_tags()
            .iter()
            .position(|tag| tag.get() == Tag::new(&name))
        else {
            return Ok(Lookup::absent(
                FontBaselineAbsence::Name,
                Some(selected_tag),
            ));
        };
        let offset = values
            .base_coord_offsets()
            .get(index)
            .ok_or_else(|| refuse(TableError::Malformed))?
            .get()
            .to_u32();
        if offset == 0 {
            return Err(refuse(TableError::Malformed));
        }
        let data = values
            .offset_data()
            .slice(usize::try_from(offset).map_err(|_| refuse(TableError::Malformed))?..)
            .ok_or_else(|| refuse(TableError::Malformed))?;
        let format = data
            .read_at::<u16>(0)
            .map_err(|_| refuse(TableError::Malformed))?;
        if !(1..=3).contains(&format) {
            return Err(refuse(TableError::BaselineCoordinateFormat { format }));
        }
        let coordinate = BaseCoord::read(data).map_err(|_| refuse(TableError::Malformed))?;
        let mut value = f64::from(coordinate.coordinate());
        match coordinate {
            BaseCoord::Format1(_) => {}
            BaseCoord::Format2(record) => {
                let glyph = record.reference_glyph();
                let point = record.base_coord_point();
                let count = self.instance.baseline_point_count(glyph).map_err(|error| {
                    if let Some(ErrorDetail::FontOutline { reason, .. }) = error.detail() {
                        refuse(TableError::BaselineOutline {
                            glyph,
                            point,
                            reason: *reason,
                        })
                    } else {
                        error
                    }
                })?;
                if usize::from(point) >= count {
                    return Err(refuse(TableError::BaselinePointRange {
                        glyph,
                        point,
                        count,
                    }));
                }
            }
            BaseCoord::Format3(record) => {
                let offset = record.device_offset().offset().to_u32();
                if offset != 0 {
                    let data = record
                        .offset_data()
                        .slice(
                            usize::try_from(offset).map_err(|_| refuse(TableError::Malformed))?..,
                        )
                        .ok_or_else(|| refuse(TableError::Malformed))?;
                    let a = data
                        .read_at::<u16>(0)
                        .map_err(|_| refuse(TableError::Malformed))?;
                    let b = data
                        .read_at::<u16>(2)
                        .map_err(|_| refuse(TableError::Malformed))?;
                    let format = data
                        .read_at::<u16>(4)
                        .map_err(|_| refuse(TableError::Malformed))?;
                    if format == 0x8000 {
                        let store = base
                            .item_var_store()
                            .ok_or_else(|| refuse(TableError::BaselineVariationStoreMissing))?
                            .map_err(|_| refuse(TableError::Malformed))?;
                        value += super::variation::baseline_delta(
                            store,
                            self.instance,
                            u16::try_from(index).map_err(|_| refuse(TableError::Malformed))?,
                            a,
                            b,
                            name,
                        )?;
                    } else {
                        let bits = match format {
                            1 => 2,
                            2 => 4,
                            3 => 8,
                            _ => return Err(refuse(TableError::BaselineDeviceFormat { format })),
                        };
                        if a > b {
                            return Err(refuse(TableError::Malformed));
                        }
                        let words = ((usize::from(b - a) + 1) * bits).div_ceil(16);
                        if data.len() < 6 + words * 2 {
                            return Err(refuse(TableError::Malformed));
                        }
                        for at in 0..words {
                            data.read_at::<u16>(6 + at * 2)
                                .map_err(|_| refuse(TableError::Malformed))?;
                        }
                        if self.font.table_data(Tag::new(b"fvar")).is_some() {
                            return Err(refuse(TableError::BaselineVariableDevice));
                        }
                        // Fully admitted static ppem deltas are deliberately not consumed.
                    }
                }
            }
        }
        finite(Field::BaselineCoordinate, Phase::Unscaled, value)?;
        Ok(Lookup {
            coordinate: Some(value),
            script: Some(selected_tag),
            absence: None,
        })
    }
}
fn unavailable(reason: FontBaselineUnavailableReason, lookup: Lookup) -> FontBaselineOutcome {
    FontBaselineOutcome::Unavailable(FontBaselineUnavailable {
        reason,
        absence: lookup
            .absence
            .expect("a missing coordinate retains its absence"),
    })
}
fn refuse(reason: TableError) -> Error {
    table_error(Table::Base, reason)
}

#[cfg(test)]
mod tests;
