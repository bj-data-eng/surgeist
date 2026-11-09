//! Checked Pad descriptors and analytic programs carried in the native ramp texture.

use crate::accounting::{Account, AccountedVec, RenderResource, RenderWork};
use crate::{BackendErrorCode, Color, Error, Result};
use peniko::BrushRef;
use vello_encoding::{DrawMonoid, DrawTag, Layout, Monoid, Ramps, StreamOffsets};

use super::bounded;

pub(crate) const RAMP_WIDTH: usize = 512;
const DESCRIPTOR_TEXELS: usize = 8;
const PROGRAM_RECORD_TEXELS: usize = 7;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PadEndpoints {
    first: u32,
    last: u32,
    first_color: u32,
    last_color: u32,
}

impl PadEndpoints {
    pub(crate) const fn first_position_bits(self) -> u32 {
        self.first
    }

    pub(crate) const fn last_position_bits(self) -> u32 {
        self.last
    }

    pub(crate) fn new(first: (f32, Color), last: (f32, Color), account: &Account) -> Result<Self> {
        // One extraction, magnitude mask and zero comparison per scalar. Signed
        // comparison thresholds retain their bits; only actual -0 becomes +0.
        account.work(RenderWork::CoordinateVisits, 1)?;
        let first_bits = first.0.to_bits();
        account.work(RenderWork::CoordinateVisits, 1)?;
        let first_magnitude = first_bits & 0x7fff_ffff;
        account.work(RenderWork::CoordinateVisits, 1)?;
        let first_offset = if first_magnitude == 0 { 0 } else { first_bits };
        account.work(RenderWork::CoordinateVisits, 1)?;
        let last_bits = last.0.to_bits();
        account.work(RenderWork::CoordinateVisits, 1)?;
        let last_magnitude = last_bits & 0x7fff_ffff;
        account.work(RenderWork::CoordinateVisits, 1)?;
        let last_offset = if last_magnitude == 0 { 0 } else { last_bits };
        // Each native color conversion processes exactly four scalar channels.
        account.work(RenderWork::CoordinateVisits, 4)?;
        let first_color = peniko::Color::from(first.1)
            .premultiply()
            .to_rgba8()
            .to_u32();
        account.work(RenderWork::CoordinateVisits, 4)?;
        let last_color = peniko::Color::from(last.1)
            .premultiply()
            .to_rgba8()
            .to_u32();
        Ok(Self {
            first: first_offset,
            last: last_offset,
            first_color,
            last_color,
        })
    }
}

/// A qualified outgoing interval. The numerical owner supplies the Power
/// coefficient only after proving its whole-segment native error bound.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ProgramSegment {
    Linear,
    Power { exponent: f32 },
    HintAtStart,
    HintAtEnd,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProgramRecord {
    pub(crate) position: f32,
    pub(crate) premultiplied: [f32; 4],
    pub(crate) outgoing: ProgramSegment,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ProgramCoordinates {
    Pad,
    Repeat,
    HorizontalDistancePad,
    HorizontalDistanceRepeat { phase: f32 },
}

impl ProgramCoordinates {
    fn horizontal_distance(self) -> bool {
        matches!(
            self,
            Self::HorizontalDistancePad | Self::HorizontalDistanceRepeat { .. }
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct GradientProgram<'a> {
    pub(crate) records: &'a [ProgramRecord],
    pub(crate) endpoints: PadEndpoints,
    pub(crate) coordinates: ProgramCoordinates,
}

#[derive(Clone, Copy)]
pub(crate) enum GradientSampling<'a> {
    PadRamp(PadEndpoints),
    HorizontalDistanceStep(PadEndpoints),
    Program(GradientProgram<'a>),
}

/// A native brush coupled to its one selected sampling interpretation.
#[derive(Clone, Copy)]
pub(crate) struct SceneBrush<'a> {
    field: SceneField<'a>,
    sampling: Option<GradientSampling<'a>>,
}

#[derive(Clone, Copy)]
enum SceneField<'a> {
    Native(BrushRef<'a>),
    Sweep {
        gradient: &'a peniko::Gradient,
        sweep: peniko::SweepGradientPosition,
        target_extent: crate::PhysicalSize,
    },
}

/// Only this receiving transition exposes the brush to native encoding.
pub(crate) struct QualifiedSceneBrush<'a> {
    brush: BrushRef<'a>,
    sampling: Option<GradientSampling<'a>>,
}

impl<'a> QualifiedSceneBrush<'a> {
    pub(crate) fn into_parts(self) -> (BrushRef<'a>, Option<GradientSampling<'a>>) {
        (self.brush, self.sampling)
    }
}

impl<'a> SceneBrush<'a> {
    pub(crate) fn new(
        brush: BrushRef<'a>,
        sampling: Option<GradientSampling<'a>>,
        target_extent: crate::PhysicalSize,
    ) -> Self {
        let field = match brush {
            BrushRef::Gradient(gradient) => match gradient.kind {
                peniko::GradientKind::Sweep(sweep) => SceneField::Sweep {
                    gradient,
                    sweep,
                    target_extent,
                },
                peniko::GradientKind::Linear(_) | peniko::GradientKind::Radial(_) => {
                    SceneField::Native(brush)
                }
            },
            BrushRef::Solid(_) | BrushRef::Image(_) => SceneField::Native(brush),
        };
        Self { field, sampling }
    }

    pub(crate) fn qualify(
        self,
        effective_transform: &super::geometry::QualifiedTransform,
        account: &Account,
    ) -> Result<QualifiedSceneBrush<'a>> {
        let brush = match self.field {
            SceneField::Native(brush) => brush,
            SceneField::Sweep {
                gradient,
                sweep,
                target_extent,
            } => {
                crate::encode::qualify_native_sweep(
                    effective_transform.native(),
                    sweep,
                    target_extent,
                    account,
                )?;
                BrushRef::Gradient(gradient)
            }
        };
        Ok(QualifiedSceneBrush {
            brush,
            sampling: self.sampling,
        })
    }
}

impl From<peniko::Color> for SceneBrush<'_> {
    fn from(color: peniko::Color) -> Self {
        Self {
            field: SceneField::Native(BrushRef::Solid(color)),
            sampling: None,
        }
    }
}

impl<'a> From<peniko::ImageBrushRef<'a>> for SceneBrush<'a> {
    fn from(image: peniko::ImageBrushRef<'a>) -> Self {
        Self {
            field: SceneField::Native(BrushRef::Image(image)),
            sampling: None,
        }
    }
}

impl<'a> From<&'a peniko::ImageBrush> for SceneBrush<'a> {
    fn from(image: &'a peniko::ImageBrush) -> Self {
        image.as_ref().into()
    }
}

#[cfg(test)]
mod sweep_receiving_tests {
    use super::*;
    use crate::{GradientRealizationError, PhysicalSize};
    use vello_encoding::Transform;

    fn gradient() -> peniko::Gradient {
        peniko::Gradient::new_sweep((0.0, 0.0), 0.0, std::f32::consts::TAU)
            .with_stops([peniko::Color::BLACK, peniko::Color::WHITE])
    }

    #[test]
    fn sweep_qualification_consumes_actual_native_factors_instead_of_source_inverse() {
        let account = Account::for_test();
        let gradient = gradient();
        for (native, accepted) in [
            (Transform::IDENTITY, true),
            (
                Transform {
                    matrix: [2.0_f32.powi(-80), 0.0, 0.0, 2.0_f32.powi(-80)],
                    translation: [0.0; 2],
                },
                false,
            ),
            (
                Transform {
                    matrix: [2.0_f32.powi(127), 0.0, 0.0, 2.0_f32.powi(-124)],
                    translation: [0.0; 2],
                },
                false,
            ),
        ] {
            // The geometry receiver has accepted both native axes. Its source
            // inverse is identity in every case; only the actual factors vary.
            let transform = super::super::geometry::qualify_native_transform(
                kurbo::Affine::IDENTITY,
                native,
                &account,
            )
            .unwrap();
            let result = SceneBrush::new((&gradient).into(), None, PhysicalSize::new(32, 16))
                .qualify(&transform, &account);
            if accepted {
                assert!(result.is_ok());
            } else {
                let error = result.err().unwrap();
                assert_eq!(
                    error.gradient_realization_diagnostic(),
                    Some(&GradientRealizationError::InsufficientNativeCapacity)
                );
                assert!(error.geometry_realization_diagnostic().is_none());
            }
        }
    }

    #[test]
    fn earlier_geometry_failure_keeps_its_owner_before_invalid_sweep_center() {
        let account = Account::for_test();
        let gradient =
            peniko::Gradient::new_sweep((f64::MAX, f64::MAX), 0.0, std::f32::consts::TAU)
                .with_stops([peniko::Color::BLACK, peniko::Color::WHITE]);
        let mut scene = super::super::scene::VelloScene::default();
        let error = scene
            .fill(
                peniko::Fill::NonZero,
                kurbo::Affine::scale(2.0_f64.powi(-127)),
                SceneBrush::new((&gradient).into(), None, PhysicalSize::new(32, 16)),
                None,
                &kurbo::Rect::new(0.0, 0.0, 32.0, 16.0),
                &account,
            )
            .unwrap_err();
        assert!(error.geometry_realization_diagnostic().is_some());
        assert!(error.gradient_realization_diagnostic().is_none());
    }
}

#[derive(Debug)]
struct OwnedProgram {
    records: AccountedVec<ProgramRecord>,
    endpoints: PadEndpoints,
    coordinates: ProgramCoordinates,
}

#[derive(Debug)]
enum CapturedSampling {
    PadRamp(PadEndpoints),
    HorizontalDistanceStep(PadEndpoints),
    Program(OwnedProgram),
}

#[derive(Debug)]
pub(crate) struct CapturedEndpoint {
    word: usize,
    sampling: CapturedSampling,
}

#[derive(Default)]
pub(crate) struct EndpointRecords {
    records: Option<AccountedVec<CapturedEndpoint>>,
}

impl EndpointRecords {
    pub(crate) fn capture(
        &mut self,
        word: usize,
        sampling: GradientSampling<'_>,
        tag: DrawTag,
        account: &Account,
    ) -> Result<()> {
        if !gradient_tag(tag) {
            return match sampling {
                GradientSampling::PadRamp(_) => Ok(()),
                GradientSampling::HorizontalDistanceStep(_) | GradientSampling::Program(_) => {
                    Err(correspondence())
                }
            };
        }
        let distance = scalar(account, || match sampling {
            GradientSampling::PadRamp(_) => false,
            GradientSampling::HorizontalDistanceStep(_) => true,
            GradientSampling::Program(program) => program.coordinates.horizontal_distance(),
        })?;
        if distance && !scalar(account, || tag == DrawTag::LINEAR_GRADIENT)? {
            return Err(correspondence());
        }
        let sampling = match sampling {
            GradientSampling::PadRamp(endpoints) => CapturedSampling::PadRamp(endpoints),
            GradientSampling::HorizontalDistanceStep(endpoints) => {
                if !scalar(account, || endpoints.first == endpoints.last)? {
                    return Err(correspondence());
                }
                CapturedSampling::HorizontalDistanceStep(endpoints)
            }
            GradientSampling::Program(program) => {
                if let ProgramCoordinates::HorizontalDistanceRepeat { phase } = program.coordinates
                    && (!scalar(account, || phase.is_finite())?
                        || scalar(account, || phase < 0.0)?
                        || scalar(account, || phase > 1.0)?
                        || (!scalar(account, || phase == 0.0)?
                            && !scalar(account, || phase.is_normal())?)
                        || !scalar(account, || program.endpoints.first == 0)?
                        || !scalar(account, || program.endpoints.last == 1.0_f32.to_bits())?)
                {
                    return Err(correspondence());
                }
                let count = scalar(account, || program.records.len())?;
                if scalar(account, || count < 2)? {
                    return Err(correspondence());
                }
                let words = mul(count, PROGRAM_RECORD_TEXELS, account)?;
                if scalar(account, || words >= (1_usize << 30))? {
                    return Err(correspondence());
                }
                account.work(RenderWork::ResourceProbes, 2)?;
                let first = program.records.first().expect("nonempty program");
                let last = program.records.last().expect("nonempty program");
                let first_control = scalar(account, || f32::from_bits(program.endpoints.first))?;
                let last_control = scalar(account, || f32::from_bits(program.endpoints.last))?;
                if !scalar(account, || first.position == first_control)?
                    || !scalar(account, || last.position == last_control)?
                    || !scalar(account, || first.position < last.position)?
                    || !matches!(last.outgoing, ProgramSegment::Linear)
                {
                    return Err(correspondence());
                }
                // Own an independent copy before the borrowed FrameBrush
                // can drop. Partial failures release this exact real capacity.
                let mut records = account.retained(count)?;
                let mut previous = None;
                for record in program.records {
                    account.work(RenderWork::GradientStopVisits, 1)?;
                    if !scalar(account, || record.position.is_finite())? {
                        return Err(correspondence());
                    }
                    if let Some(previous) = previous
                        && scalar(account, || record.position < previous)?
                    {
                        return Err(correspondence());
                    }
                    previous = Some(record.position);
                    for channel in record.premultiplied {
                        if !scalar(account, || channel.is_finite())?
                            || scalar(account, || channel < 0.0)?
                            || scalar(account, || channel > 1.0)?
                        {
                            return Err(correspondence());
                        }
                    }
                    for channel in &record.premultiplied[..3] {
                        if scalar(account, || *channel > record.premultiplied[3])? {
                            return Err(correspondence());
                        }
                    }
                    if let ProgramSegment::Power { exponent } = record.outgoing
                        && (!scalar(account, || exponent.is_normal())?
                            || scalar(account, || exponent <= 0.0)?)
                    {
                        return Err(correspondence());
                    }
                    let slot = records.append_slot()?;
                    account.work(RenderWork::RasterResourceCopies, 1)?;
                    slot.write(*record);
                }
                CapturedSampling::Program(OwnedProgram {
                    records,
                    endpoints: program.endpoints,
                    coordinates: program.coordinates,
                })
            }
        };
        if self.records.is_none() {
            self.records = Some(account.retained(0)?);
        }
        let records = self.records.as_mut().expect("endpoint backing");
        if !records.is_empty() {
            account.work(RenderWork::ResourceProbes, 1)?;
            let previous = records.last().expect("nonempty endpoint records");
            if scalar(account, || previous.word >= word)? {
                return Err(correspondence());
            }
        }
        bounded::push_record(records, CapturedEndpoint { word, sampling }, account)
    }

    pub(crate) fn as_slice(&self) -> &[CapturedEndpoint] {
        self.records
            .as_ref()
            .map_or(&[], |records| records.as_slice())
    }
}

fn gradient_tag(tag: DrawTag) -> bool {
    matches!(
        tag,
        DrawTag::LINEAR_GRADIENT | DrawTag::RADIAL_GRADIENT | DrawTag::SWEEP_GRADIENT
    )
}

fn correspondence() -> Error {
    Error::new(
        BackendErrorCode::RenderFailed,
        "locked native-gradient sampling correspondence changed",
    )
}

fn scalar<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

fn add(a: usize, b: usize, account: &Account) -> Result<usize> {
    scalar(account, || a.checked_add(b))?.ok_or_else(|| {
        account
            .storage()
            .capacity_error(RenderResource::RetainedCpuBytes)
    })
}

fn mul(a: usize, b: usize, account: &Account) -> Result<usize> {
    scalar(account, || a.checked_mul(b))?.ok_or_else(|| {
        account
            .storage()
            .capacity_error(RenderResource::RetainedCpuBytes)
    })
}

/// A complete guarded texture payload. All rows before the descriptors are the
/// dependency's unchanged ramps. Padding is initialized real upload storage.
pub(crate) struct EndpointTexture {
    pub(crate) texels: AccountedVec<u32>,
    pub(crate) height: u32,
}

pub(crate) fn prepare(
    layout: &Layout,
    packed: &mut [u8],
    ramps: Ramps<'_>,
    glyph_sizes: &StreamOffsets,
    records: &[CapturedEndpoint],
    account: &Account,
) -> Result<Option<EndpointTexture>> {
    if records.is_empty() {
        return Ok(None);
    }
    let glyph_tags_empty = scalar(account, || glyph_sizes.draw_tags == 0)?;
    let glyph_data_empty = scalar(account, || glyph_sizes.draw_data == 0)?;
    if !glyph_tags_empty || !glyph_data_empty {
        return Err(correspondence());
    }

    // Qualify every public Layout slice before using an accessor or writing the
    // private command. These are word indices, converted to byte indices once.
    let mut previous = 0;
    account.work(RenderWork::CoordinateVisits, 6)?;
    let bases = [
        layout.path_tag_base,
        layout.path_data_base,
        layout.draw_tag_base,
        layout.draw_data_base,
        layout.transform_base,
        layout.style_base,
    ];
    let mut bytes = [0; 6];
    for (index, word) in bases.into_iter().enumerate() {
        let byte = mul(word as usize, 4, account)?;
        if scalar(account, || byte < previous)? {
            return Err(correspondence());
        }
        if scalar(account, || byte > packed.len())? {
            return Err(correspondence());
        }
        bytes[index] = byte;
        previous = byte;
    }
    let tags_start = bytes[2];
    let data_start = bytes[3];
    let data_end = bytes[4];
    let tag_bytes = scalar(account, || data_start - tags_start)?;
    let tag_count = scalar(account, || tag_bytes / 4)?;
    if !scalar(account, || tag_count == layout.n_draw_objects as usize)? {
        return Err(correspondence());
    }
    let native_count = mul(RAMP_WIDTH, ramps.height as usize, account)?;
    if scalar(account, || ramps.width as usize != RAMP_WIDTH)? {
        return Err(correspondence());
    }
    if scalar(account, || ramps.data.len() != native_count)? {
        return Err(correspondence());
    }
    let mut required = native_count;
    for record in records {
        account.work(RenderWork::ResourceProbes, 1)?;
        required = add(required, DESCRIPTOR_TEXELS, account)?;
        if let CapturedSampling::Program(program) = &record.sampling {
            required = add(
                required,
                mul(PROGRAM_RECORD_TEXELS, program.records.len(), account)?,
                account,
            )?;
        }
    }
    // A command reserves two low bits. Qualify every possible descriptor and
    // record address, including the last word, before allocating upload backing.
    if scalar(account, || required > (1_usize << 30))? {
        return Err(correspondence());
    }
    let rounded = add(required, RAMP_WIDTH - 1, account)?;
    let rows = scalar(account, || rounded / RAMP_WIDTH)?;
    let padded = mul(rows, RAMP_WIDTH, account)?;
    let height = scalar(account, || u32::try_from(rows))?.map_err(|_| correspondence())?;
    let mut texels = account.retained(padded)?;
    for texel in ramps.data {
        let slot = texels.append_slot()?;
        account.work(RenderWork::SerializedBytes, 4)?;
        slot.write(*texel);
    }

    let mut prefix = 0_usize;
    let mut record_index = 0;
    let mut next = read_record(records, record_index, account)?;
    for index in 0..tag_count {
        account.work(RenderWork::ResourceProbes, 1)?;
        let tag_byte = add(tags_start, mul(index, 4, account)?, account)?;
        let tag = DrawTag(read_word(packed, tag_byte)?);
        if !matches!(
            tag,
            DrawTag::NOP
                | DrawTag::COLOR
                | DrawTag::LINEAR_GRADIENT
                | DrawTag::RADIAL_GRADIENT
                | DrawTag::SWEEP_GRADIENT
                | DrawTag::IMAGE
                | DrawTag::BLUR_RECT
                | DrawTag::BEGIN_CLIP
                | DrawTag::END_CLIP
        ) {
            return Err(correspondence());
        }
        let size = scalar(account, || DrawMonoid::new(tag).scene_offset as usize)?;
        let end = add(prefix, size, account)?;
        let byte_end = add(data_start, mul(end, 4, account)?, account)?;
        if !scalar(account, || byte_end <= data_end)? {
            return Err(correspondence());
        }
        if let Some(record) = next {
            if scalar(account, || record.word < prefix)? {
                return Err(correspondence());
            }
            if scalar(account, || record.word == prefix)? && size != 0 {
                if !gradient_tag(tag) {
                    return Err(correspondence());
                }
                let distance = scalar(account, || match &record.sampling {
                    CapturedSampling::PadRamp(_) => false,
                    CapturedSampling::HorizontalDistanceStep(_) => true,
                    CapturedSampling::Program(program) => program.coordinates.horizontal_distance(),
                })?;
                if distance && !scalar(account, || tag == DrawTag::LINEAR_GRADIENT)? {
                    return Err(correspondence());
                }
                let byte = add(data_start, mul(prefix, 4, account)?, account)?;
                account.work(RenderWork::ResourceProbes, 1)?;
                let original = read_word(packed, byte)?;
                let mode = scalar(account, || original & 3)?;
                let ramp = scalar(account, || original >> 2)?;
                let expected_mode = match &record.sampling {
                    CapturedSampling::Program(OwnedProgram {
                        coordinates:
                            ProgramCoordinates::Repeat
                            | ProgramCoordinates::HorizontalDistanceRepeat { .. },
                        ..
                    }) => 1,
                    _ => 0,
                };
                if scalar(account, || mode != expected_mode)? {
                    return Err(correspondence());
                }
                if scalar(account, || ramp >= ramps.height)? {
                    return Err(correspondence());
                }
                let flat = scalar(account, || u32::try_from(texels.len()))?
                    .map_err(|_| correspondence())?;
                let (kind, payload, count, endpoints, phase) = match &record.sampling {
                    CapturedSampling::PadRamp(endpoints) => (0, ramp, 0, *endpoints, 0),
                    CapturedSampling::HorizontalDistanceStep(endpoints) => {
                        (3, ramp, 0, *endpoints, 0)
                    }
                    CapturedSampling::Program(program) => {
                        let payload = add(texels.len(), DESCRIPTOR_TEXELS, account)?;
                        let payload = scalar(account, || u32::try_from(payload))?
                            .map_err(|_| correspondence())?;
                        let count = scalar(account, || u32::try_from(program.records.len()))?
                            .map_err(|_| correspondence())?;
                        let (kind, phase) = match program.coordinates {
                            ProgramCoordinates::Pad => (1, 0),
                            ProgramCoordinates::Repeat => (2, 0),
                            ProgramCoordinates::HorizontalDistancePad => (4, 0),
                            ProgramCoordinates::HorizontalDistanceRepeat { phase } => {
                                (5, scalar(account, || phase.to_bits())?)
                            }
                        };
                        (kind, payload, count, program.endpoints, phase)
                    }
                };
                for word in [
                    kind,
                    payload,
                    count,
                    endpoints.first,
                    endpoints.last,
                    endpoints.first_color,
                    endpoints.last_color,
                    phase,
                ] {
                    serialize_word(&mut texels, word, account)?;
                }
                if let CapturedSampling::Program(program) = &record.sampling {
                    for record in &program.records {
                        account.work(RenderWork::GradientStopVisits, 1)?;
                        let position = scalar(account, || record.position.to_bits())?;
                        serialize_word(&mut texels, position, account)?;
                        for channel in record.premultiplied {
                            let word = scalar(account, || channel.to_bits())?;
                            serialize_word(&mut texels, word, account)?;
                        }
                        let (kind, exponent) = match record.outgoing {
                            ProgramSegment::Linear => (0, 0),
                            ProgramSegment::Power { exponent } => {
                                (1, scalar(account, || exponent.to_bits())?)
                            }
                            ProgramSegment::HintAtStart => (2, 0),
                            ProgramSegment::HintAtEnd => (3, 0),
                        };
                        serialize_word(&mut texels, kind, account)?;
                        serialize_word(&mut texels, exponent, account)?;
                    }
                }
                let shifted = scalar(account, || flat << 2)?;
                let command = scalar(account, || shifted | 3)?;
                account.work(RenderWork::SerializedBytes, 4)?;
                packed[byte..byte + 4].copy_from_slice(&command.to_ne_bytes());
                record_index += 1;
                next = read_record(records, record_index, account)?;
            }
        }
        prefix = end;
    }
    let data_bytes = scalar(account, || data_end - data_start)?;
    let consumed = mul(prefix, 4, account)?;
    if scalar(account, || next.is_some())? {
        return Err(correspondence());
    }
    if scalar(account, || consumed != data_bytes)? {
        return Err(correspondence());
    }
    while texels.len() < padded {
        let slot = texels.append_slot()?;
        account.work(RenderWork::SerializedBytes, 4)?;
        slot.write(0);
    }
    Ok(Some(EndpointTexture { texels, height }))
}

fn serialize_word(texels: &mut AccountedVec<u32>, word: u32, account: &Account) -> Result<()> {
    let slot = texels.append_slot()?;
    account.work(RenderWork::SerializedBytes, 4)?;
    // Exact little-endian words reconstructed by WGSL pack4x8unorm.
    slot.write(u32::from_ne_bytes(word.to_le_bytes()));
    Ok(())
}

fn read_record<'a>(
    records: &'a [CapturedEndpoint],
    index: usize,
    account: &Account,
) -> Result<Option<&'a CapturedEndpoint>> {
    if index == records.len() {
        return Ok(None);
    }
    account.work(RenderWork::ResourceProbes, 1)?;
    Ok(records.get(index))
}

fn read_word(bytes: &[u8], offset: usize) -> Result<u32> {
    let word: [u8; 4] = bytes
        .get(offset..offset.checked_add(4).ok_or_else(correspondence)?)
        .ok_or_else(correspondence)?
        .try_into()
        .map_err(|_| correspondence())?;
    Ok(u32::from_ne_bytes(word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{RenderAttemptKind, RenderPhase};
    use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
    use vello_encoding::{Encoding, Resolver};

    fn endpoints() -> PadEndpoints {
        PadEndpoints {
            first: 0.0_f32.to_bits(),
            last: 1.0_f32.to_bits(),
            first_color: 0xff00_00ff,
            last_color: 0xff00_ff00,
        }
    }

    #[test]
    fn comparison_thresholds_preserve_signed_nonzero_bits_and_canonicalize_zero() {
        for threshold in [
            -f32::INFINITY,
            -1.0,
            -f32::from_bits(1),
            -0.0,
            0.0,
            f32::from_bits(1),
            1.0,
            f32::INFINITY,
        ] {
            let actual = PadEndpoints::new(
                (threshold, Color::BLACK),
                (threshold, Color::TRANSPARENT),
                &Account::for_test(),
            )
            .unwrap();
            let expected = if threshold == 0.0 {
                0
            } else {
                threshold.to_bits()
            };
            assert_eq!(actual.first, expected);
            assert_eq!(actual.last, expected);
        }
    }

    fn append(encoding: &mut Encoding, extend: peniko::Extend) -> usize {
        let word = encoding.draw_data.len();
        let mut path = encoding.encode_path(true);
        path.move_to(0.0, 0.0);
        path.line_to(8.0, 0.0);
        path.line_to(8.0, 8.0);
        path.close();
        path.finish(true);
        let gradient = peniko::Gradient::new_linear((0.0, 0.0), (8.0, 0.0))
            .with_stops([(0.0, peniko::Color::BLACK), (1.0, peniko::Color::WHITE)])
            .with_extend(extend);
        encoding.encode_brush(&gradient, 1.0);
        word
    }

    #[test]
    fn pad_descriptor_keeps_the_shared_repeat_ramp_immutable() {
        let mut encoding = Encoding::new();
        let pad = append(&mut encoding, peniko::Extend::Pad);
        let repeat = append(&mut encoding, peniko::Extend::Repeat);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        assert_eq!(
            ramps.height, 1,
            "the native cache shares these identical stops"
        );
        let repeat_before = layout.draw_data(&packed)[repeat];
        let records = [CapturedEndpoint {
            word: pad,
            sampling: CapturedSampling::PadRamp(endpoints()),
        }];
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    let texture = prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets::default(),
                        &records,
                        account,
                    )?
                    .unwrap();
                    assert_eq!(&texture.texels[..RAMP_WIDTH], ramps.data);
                    assert_eq!(texture.height, 2);
                    assert_eq!(texture.texels.len(), 1024);
                    assert_eq!(layout.draw_data(&packed)[pad], (512 << 2) | 3);
                    assert_eq!(layout.draw_data(&packed)[repeat], repeat_before);
                    assert_eq!(repeat_before & 3, 1);
                    let descriptor = &texture.texels[512..520];
                    assert_eq!(
                        descriptor
                            .iter()
                            .map(|word| u32::from_le(*word))
                            .collect::<Vec<_>>(),
                        [
                            0,
                            0,
                            0,
                            0.0_f32.to_bits(),
                            1.0_f32.to_bits(),
                            0xff00_00ff,
                            0xff00_ff00,
                            0
                        ]
                    );
                    assert!(texture.texels[520..].iter().all(|word| *word == 0));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn descriptors_share_rows_and_roll_over_at_the_eighth_word_boundary() {
        let mut encoding = Encoding::new();
        let records: Vec<_> = (0..65)
            .map(|_| CapturedEndpoint {
                word: append(&mut encoding, peniko::Extend::Pad),
                sampling: CapturedSampling::PadRamp(endpoints()),
            })
            .collect();
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    let texture = prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets::default(),
                        &records,
                        account,
                    )?
                    .unwrap();
                    // 512 native texels + 520 descriptor texels, padded only once.
                    assert_eq!(texture.texels.len(), 1536);
                    assert_eq!(texture.height, 3);
                    let start = 512 + 64 * 8;
                    assert_eq!(start, 1024);
                    assert_eq!(
                        layout.draw_data(&packed)[records[64].word],
                        ((start as u32) << 2) | 3
                    );
                    assert_eq!(u32::from_le(texture.texels[start + 2]), 0);
                    assert_eq!(u32::from_le(texture.texels[start + 5]), 0xff00_00ff);
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn changed_glyph_draw_streams_refuse_before_descriptor_allocation() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Pad);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let original = packed.clone();
        let records = [CapturedEndpoint {
            word,
            sampling: CapturedSampling::PadRamp(endpoints()),
        }];
        let mut context = RenderAccountingContext::default();
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets {
                            draw_data: 1,
                            ..StreamOffsets::default()
                        },
                        &records,
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(packed, original);
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn invalid_later_association_releases_the_real_serialized_prefix() {
        let mut encoding = Encoding::new();
        let first = append(&mut encoding, peniko::Extend::Pad);
        let second = append(&mut encoding, peniko::Extend::Pad);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let records = [
            CapturedEndpoint {
                word: first,
                sampling: CapturedSampling::PadRamp(endpoints()),
            },
            CapturedEndpoint {
                word: second + 1,
                sampling: CapturedSampling::PadRamp(endpoints()),
            },
        ];
        let mut context = RenderAccountingContext::default();
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets::default(),
                        &records,
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            layout.draw_data(&packed)[first] & 3,
            3,
            "one unpublished command was rewritten"
        );
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert!(usage.peak(RenderResource::RetainedCpuBytes) >= 1024 * 4);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::SerializedBytes), (512 + 8 + 1) * 4);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    }

    #[test]
    fn refused_descriptor_serialization_disposes_actual_capacity() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Pad);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let records = [CapturedEndpoint {
            word,
            sampling: CapturedSampling::PadRamp(endpoints()),
        }];
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::SerializedBytes, 512 * 4)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets::default(),
                        &records,
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::SerializedBytes), 512 * 4);
        assert!(usage.peak(RenderResource::RetainedCpuBytes) >= 1024 * 4);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(layout.draw_data(&packed)[word] & 3, 0);
    }

    #[test]
    fn actual_texture_capacity_and_zero_resource_cap_have_distinct_prefixes() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Pad);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let records = [CapturedEndpoint {
            word,
            sampling: CapturedSampling::PadRamp(endpoints()),
        }];
        let original = packed.clone();
        let mut context = RenderAccountingContext::default();
        let bytes = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    let texture = prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets::default(),
                        &records,
                        account,
                    )?
                    .unwrap();
                    Ok((texture.texels.capacity() * std::mem::size_of::<u32>()) as u64)
                },
            )
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), bytes);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);

        let mut packed = original.clone();
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(RenderLimits::new().with_retained_bytes(0)),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &layout,
                        &mut packed,
                        ramps,
                        &StreamOffsets::default(),
                        &records,
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(
            report.refusal().unwrap().quantity(),
            crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes)
        );
        assert_eq!(report.usage().peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::SerializedBytes), 0);
        assert_eq!(packed, original);
    }

    fn program_records() -> [ProgramRecord; 2] {
        [
            ProgramRecord {
                position: 0.0,
                premultiplied: [1.0, 0.0, 0.0, 1.0],
                outgoing: ProgramSegment::Power { exponent: 0.5 },
            },
            ProgramRecord {
                position: 1.0,
                premultiplied: [0.0, 1.0, 0.0, 1.0],
                outgoing: ProgramSegment::Linear,
            },
        ]
    }

    #[test]
    fn program_capture_survives_source_drop_and_serializes_exact_words() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Repeat);
        let neighbor = append(&mut encoding, peniko::Extend::Repeat);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let neighbor_before = layout.draw_data(&packed)[neighbor];
        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let mut captured = EndpointRecords::default();
                {
                    let source = program_records();
                    captured.capture(
                        word,
                        GradientSampling::Program(GradientProgram {
                            records: &source,
                            endpoints: endpoints(),
                            coordinates: ProgramCoordinates::Repeat,
                        }),
                        DrawTag::LINEAR_GRADIENT,
                        account,
                    )?;
                }
                let texture = prepare(
                    &layout,
                    &mut packed,
                    ramps,
                    &StreamOffsets::default(),
                    captured.as_slice(),
                    account,
                )?
                .unwrap();
                assert_eq!(&texture.texels[..512], ramps.data);
                assert_eq!(layout.draw_data(&packed)[neighbor], neighbor_before);
                assert_eq!(layout.draw_data(&packed)[word], (512 << 2) | 3);
                let words: Vec<_> = texture.texels[512..534]
                    .iter()
                    .map(|word| u32::from_le(*word))
                    .collect();
                assert_eq!(
                    words,
                    [
                        2,
                        520,
                        2,
                        0,
                        1.0_f32.to_bits(),
                        0xff00_00ff,
                        0xff00_ff00,
                        0,
                        0,
                        1.0_f32.to_bits(),
                        0,
                        0,
                        1.0_f32.to_bits(),
                        1,
                        0.5_f32.to_bits(),
                        1.0_f32.to_bits(),
                        0,
                        1.0_f32.to_bits(),
                        0,
                        1.0_f32.to_bits(),
                        0,
                        0,
                    ]
                );
                assert!(texture.texels[534..].iter().all(|word| *word == 0));
                Ok(())
            })
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 4);
        assert_eq!(usage.work(RenderWork::SerializedBytes), (1024 + 1) * 4);
    }

    #[test]
    fn program_copy_refusal_releases_actual_prefix_and_fresh_attempt_retries() {
        let source = program_records();
        let mut captured = EndpointRecords::default();
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::RasterResourceCopies, 1)
                    .unwrap(),
            ),
        );
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                captured.capture(
                    0,
                    GradientSampling::Program(GradientProgram {
                        records: &source,
                        endpoints: endpoints(),
                        coordinates: ProgramCoordinates::Pad,
                    }),
                    DrawTag::LINEAR_GRADIENT,
                    account,
                )
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(captured.as_slice().is_empty());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::RasterResourceCopies), 1);
        assert!(
            usage.peak(RenderResource::RetainedCpuBytes) >= (2 * size_of::<ProgramRecord>()) as u64
        );
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 0);

        let mut retry = RenderAccountingContext::default();
        retry
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                captured.capture(
                    0,
                    GradientSampling::Program(GradientProgram {
                        records: &source,
                        endpoints: endpoints(),
                        coordinates: ProgramCoordinates::Pad,
                    }),
                    DrawTag::LINEAR_GRADIENT,
                    account,
                )?;
                assert_eq!(captured.as_slice().len(), 1);
                drop(std::mem::take(&mut captured));
                Ok(())
            })
            .unwrap();
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn malformed_program_controls_refuse_without_retaining_a_descriptor() {
        for positions in [[0.0, 0.0], [1.0, 0.0], [0.0, f32::NAN]] {
            let mut source = program_records();
            source[0].position = positions[0];
            source[1].position = positions[1];
            let mut captured = EndpointRecords::default();
            let mut context = RenderAccountingContext::default();
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    captured.capture(
                        0,
                        GradientSampling::Program(GradientProgram {
                            records: &source,
                            endpoints: endpoints(),
                            coordinates: ProgramCoordinates::Pad,
                        }),
                        DrawTag::LINEAR_GRADIENT,
                        account,
                    )
                })
                .unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
            assert!(captured.as_slice().is_empty());
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
        }
    }

    #[test]
    fn distance_coordinates_require_linear_carriers_before_copying() {
        let source = program_records();
        let step = PadEndpoints {
            first: 1,
            last: 1,
            ..endpoints()
        };
        for sampling in [
            GradientSampling::HorizontalDistanceStep(step),
            GradientSampling::Program(GradientProgram {
                records: &source,
                endpoints: endpoints(),
                coordinates: ProgramCoordinates::HorizontalDistancePad,
            }),
            GradientSampling::Program(GradientProgram {
                records: &source,
                endpoints: endpoints(),
                coordinates: ProgramCoordinates::HorizontalDistanceRepeat { phase: 0.25 },
            }),
        ] {
            for tag in [
                DrawTag::COLOR,
                DrawTag::RADIAL_GRADIENT,
                DrawTag::SWEEP_GRADIENT,
            ] {
                let mut captured = EndpointRecords::default();
                let mut context = RenderAccountingContext::default();
                let error = context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        captured.capture(0, sampling, tag, account)
                    })
                    .unwrap_err();
                assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
                assert!(captured.as_slice().is_empty());
                let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
                assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
                assert_eq!(usage.work(RenderWork::RasterResourceCopies), 0);
            }
        }
    }

    #[test]
    fn distance_step_packs_equal_directed_controls_without_program_records() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Pad);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let mut captured = EndpointRecords::default();
                captured.capture(
                    word,
                    GradientSampling::HorizontalDistanceStep(PadEndpoints {
                        first: 1,
                        last: 1,
                        ..endpoints()
                    }),
                    DrawTag::LINEAR_GRADIENT,
                    account,
                )?;
                let texture = prepare(
                    &layout,
                    &mut packed,
                    ramps,
                    &StreamOffsets::default(),
                    captured.as_slice(),
                    account,
                )?
                .unwrap();
                let header: Vec<_> = texture.texels[512..520]
                    .iter()
                    .map(|word| u32::from_le(*word))
                    .collect();
                assert_eq!(header, [3, 0, 0, 1, 1, 0xff00_00ff, 0xff00_ff00, 0]);
                assert!(texture.texels[520..].iter().all(|word| *word == 0));
                Ok(())
            })
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn distance_step_resolved_radial_association_refuses_after_real_ramp_prefix() {
        let mut encoding = Encoding::new();
        let mut path = encoding.encode_path(true);
        path.move_to(0.0, 0.0);
        path.line_to(8.0, 0.0);
        path.line_to(8.0, 8.0);
        path.close();
        path.finish(true);
        let word = encoding.draw_data.len();
        let gradient = peniko::Gradient::new_radial((0.0, 0.0), 8.0)
            .with_stops([(0.0, peniko::Color::BLACK), (1.0, peniko::Color::WHITE)]);
        encoding.encode_brush(&gradient, 1.0);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let original = packed.clone();
        let records = [CapturedEndpoint {
            word,
            sampling: CapturedSampling::HorizontalDistanceStep(PadEndpoints {
                first: 1,
                last: 1,
                ..endpoints()
            }),
        }];
        let mut context = RenderAccountingContext::default();
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                prepare(
                    &layout,
                    &mut packed,
                    ramps,
                    &StreamOffsets::default(),
                    &records,
                    account,
                )
                .map(drop)
            })
            .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(packed, original);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::SerializedBytes), 512 * 4);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn distance_repeat_phase_is_owned_and_packed_as_an_exact_eighth_word() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Repeat);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let mut captured = EndpointRecords::default();
                {
                    let source = program_records();
                    captured.capture(
                        word,
                        GradientSampling::Program(GradientProgram {
                            records: &source,
                            endpoints: endpoints(),
                            coordinates: ProgramCoordinates::HorizontalDistanceRepeat {
                                phase: 0.25,
                            },
                        }),
                        DrawTag::LINEAR_GRADIENT,
                        account,
                    )?;
                }
                let texture = prepare(
                    &layout,
                    &mut packed,
                    ramps,
                    &StreamOffsets::default(),
                    captured.as_slice(),
                    account,
                )?
                .unwrap();
                let header: Vec<_> = texture.texels[512..520]
                    .iter()
                    .map(|word| u32::from_le(*word))
                    .collect();
                assert_eq!(
                    header,
                    [
                        5,
                        520,
                        2,
                        0,
                        1.0_f32.to_bits(),
                        0xff00_00ff,
                        0xff00_ff00,
                        0.25_f32.to_bits()
                    ]
                );
                assert_eq!(&texture.texels[..512], ramps.data);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn distance_repeat_invalid_phase_or_nonunit_controls_refuse_before_copying() {
        let source = program_records();
        for (phase, endpoints) in [
            (f32::NAN, endpoints()),
            (f32::INFINITY, endpoints()),
            (-0.25, endpoints()),
            (1.25, endpoints()),
            (f32::from_bits(1), endpoints()),
            (
                0.25,
                PadEndpoints {
                    first: 1,
                    ..endpoints()
                },
            ),
            (
                0.25,
                PadEndpoints {
                    last: 0.5_f32.to_bits(),
                    ..endpoints()
                },
            ),
        ] {
            let mut captured = EndpointRecords::default();
            let mut context = RenderAccountingContext::default();
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    captured.capture(
                        0,
                        GradientSampling::Program(GradientProgram {
                            records: &source,
                            endpoints,
                            coordinates: ProgramCoordinates::HorizontalDistanceRepeat { phase },
                        }),
                        DrawTag::LINEAR_GRADIENT,
                        account,
                    )
                })
                .unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
            assert!(captured.as_slice().is_empty());
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .peak(RenderResource::RetainedCpuBytes),
                0
            );
        }
    }

    #[test]
    fn an_eighth_header_word_refusal_preserves_the_exact_real_prefix_and_retry() {
        let mut encoding = Encoding::new();
        let word = append(&mut encoding, peniko::Extend::Pad);
        let mut resolver = Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&encoding, &mut packed);
        let records = [CapturedEndpoint {
            word,
            sampling: CapturedSampling::PadRamp(endpoints()),
        }];
        // The native 512-word ramp and the first seven actual header words fit;
        // the required phase word is the first refused serialization action.
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::SerializedBytes, (512 + 7) * 4)
                    .unwrap(),
            ),
        );
        let original = packed.clone();
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                prepare(
                    &layout,
                    &mut packed,
                    ramps,
                    &StreamOffsets::default(),
                    &records,
                    account,
                )
                .map(drop)
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert_eq!(packed, original);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::SerializedBytes), (512 + 7) * 4);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert!(usage.peak(RenderResource::RetainedCpuBytes) >= 1024 * 4);
        let mut retry = RenderAccountingContext::default();
        retry
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let texture = prepare(
                    &layout,
                    &mut packed,
                    ramps,
                    &StreamOffsets::default(),
                    &records,
                    account,
                )?
                .unwrap();
                assert_eq!(u32::from_le(texture.texels[519]), 0);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn capture_backing_refusal_leaves_no_retained_record() {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(RenderLimits::new().with_retained_bytes(0)),
        );
        let mut records = EndpointRecords::default();
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                records.capture(
                    0,
                    GradientSampling::PadRamp(endpoints()),
                    DrawTag::LINEAR_GRADIENT,
                    account,
                )
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(records.as_slice().is_empty());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::RasterResourceCopies), 0);
    }
}
