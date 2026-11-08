//! Ordinary Pad side data transported through the existing native ramp texture.

use crate::accounting::{Account, AccountedVec, RenderResource, RenderWork};
use crate::{BackendErrorCode, Color, Error, Result};
use peniko::BrushRef;
use vello_encoding::{DrawMonoid, DrawTag, Layout, Monoid, Ramps, StreamOffsets};

use super::bounded;

pub(crate) const RAMP_WIDTH: usize = 512;
const DESCRIPTOR_TEXELS: usize = 5;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PadEndpoints {
    first: u32,
    last: u32,
    first_color: u32,
    last_color: u32,
}

impl PadEndpoints {
    pub(crate) fn new(first: (f32, Color), last: (f32, Color), account: &Account) -> Result<Self> {
        // One extraction and one canonicalization per scalar. Valid normalized
        // offsets are nonnegative, so clearing the sign only changes -0 to +0.
        account.work(RenderWork::CoordinateVisits, 1)?;
        let first_bits = first.0.to_bits();
        account.work(RenderWork::CoordinateVisits, 1)?;
        let first_offset = first_bits & 0x7fff_ffff;
        account.work(RenderWork::CoordinateVisits, 1)?;
        let last_bits = last.0.to_bits();
        account.work(RenderWork::CoordinateVisits, 1)?;
        let last_offset = last_bits & 0x7fff_ffff;
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

/// A borrowed native brush and the side semantics its native stop list cannot
/// express alone. Internal scene entrypoints consume this coupled value.
#[derive(Clone, Copy)]
pub(crate) struct SceneBrush<'a> {
    pub(crate) brush: BrushRef<'a>,
    pub(crate) endpoints: Option<PadEndpoints>,
}

impl<'a> SceneBrush<'a> {
    pub(crate) const fn new(brush: BrushRef<'a>, endpoints: Option<PadEndpoints>) -> Self {
        Self { brush, endpoints }
    }
}

impl<'a, T: Into<BrushRef<'a>>> From<T> for SceneBrush<'a> {
    fn from(brush: T) -> Self {
        Self::new(brush.into(), None)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CapturedEndpoint {
    word: usize,
    endpoints: PadEndpoints,
}

#[derive(Default)]
pub(crate) struct EndpointRecords {
    records: Option<AccountedVec<CapturedEndpoint>>,
}

impl EndpointRecords {
    pub(crate) fn capture(
        &mut self,
        word: usize,
        endpoints: PadEndpoints,
        tag: DrawTag,
        account: &Account,
    ) -> Result<()> {
        if !gradient_tag(tag) {
            return Ok(());
        }
        if self.records.is_none() {
            self.records = Some(account.retained(0)?);
        }
        let records = self.records.as_mut().expect("endpoint backing");
        if !records.is_empty() {
            account.work(RenderWork::ResourceProbes, 1)?;
            let previous = records.last().expect("nonempty endpoint records");
            if previous.word >= word {
                return Err(correspondence());
            }
        }
        bounded::push_record(records, CapturedEndpoint { word, endpoints }, account)
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
        "locked ordinary-gradient endpoint correspondence changed",
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
    let required = add(
        native_count,
        mul(DESCRIPTOR_TEXELS, records.len(), account)?,
        account,
    )?;
    let rounded = add(required, RAMP_WIDTH - 1, account)?;
    let rows = scalar(account, || rounded / RAMP_WIDTH)?;
    let padded = mul(rows, RAMP_WIDTH, account)?;
    let height = scalar(account, || u32::try_from(rows))?.map_err(|_| correspondence())?;
    let last_index = scalar(account, || records.len() - 1)?;
    let last_descriptor = add(
        native_count,
        mul(DESCRIPTOR_TEXELS, last_index, account)?,
        account,
    )?;
    if !scalar(account, || last_descriptor < (1_usize << 30))? {
        return Err(correspondence());
    }
    let mut texels = account.retained(padded)?;
    for texel in ramps.data {
        account.work(RenderWork::SerializedBytes, 4)?;
        texels.push_reserved(*texel);
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
                let byte = add(data_start, mul(prefix, 4, account)?, account)?;
                account.work(RenderWork::ResourceProbes, 1)?;
                let original = read_word(packed, byte)?;
                let mode = scalar(account, || original & 3)?;
                let ramp = scalar(account, || original >> 2)?;
                if scalar(account, || mode != 0)? {
                    return Err(correspondence());
                }
                if scalar(account, || ramp >= ramps.height)? {
                    return Err(correspondence());
                }
                let flat = scalar(account, || u32::try_from(texels.len()))?
                    .map_err(|_| correspondence())?;
                for word in [
                    record.endpoints.first,
                    record.endpoints.last,
                    ramp,
                    record.endpoints.first_color,
                    record.endpoints.last_color,
                ] {
                    account.work(RenderWork::SerializedBytes, 4)?;
                    // Upload bytes are explicitly little endian, matching the
                    // WGSL pack4x8unorm codec regardless of CPU byte order.
                    texels.push_reserved(u32::from_ne_bytes(word.to_le_bytes()));
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
        account.work(RenderWork::SerializedBytes, 4)?;
        texels.push_reserved(0);
    }
    Ok(Some(EndpointTexture { texels, height }))
}

fn read_record(
    records: &[CapturedEndpoint],
    index: usize,
    account: &Account,
) -> Result<Option<CapturedEndpoint>> {
    if index == records.len() {
        return Ok(None);
    }
    account.work(RenderWork::ResourceProbes, 1)?;
    Ok(records.get(index).copied())
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
            endpoints: endpoints(),
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
                    let descriptor = &texture.texels[512..517];
                    assert_eq!(
                        descriptor
                            .iter()
                            .map(|word| u32::from_le(*word))
                            .collect::<Vec<_>>(),
                        [
                            0.0_f32.to_bits(),
                            1.0_f32.to_bits(),
                            0,
                            0xff00_00ff,
                            0xff00_ff00
                        ]
                    );
                    assert!(texture.texels[517..].iter().all(|word| *word == 0));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn a_descriptor_crosses_rows_without_a_per_draw_row() {
        let mut encoding = Encoding::new();
        let records: Vec<_> = (0..103)
            .map(|_| CapturedEndpoint {
                word: append(&mut encoding, peniko::Extend::Pad),
                endpoints: endpoints(),
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
                    // 512 native texels + 515 descriptor texels, padded only once.
                    assert_eq!(texture.texels.len(), 1536);
                    assert_eq!(texture.height, 3);
                    let start = 512 + 102 * 5;
                    assert_eq!(start, 1022);
                    assert_eq!(
                        layout.draw_data(&packed)[records[102].word],
                        ((start as u32) << 2) | 3
                    );
                    assert_eq!(u32::from_le(texture.texels[start + 2]), 0);
                    assert_eq!(u32::from_le(texture.texels[start + 3]), 0xff00_00ff);
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
            endpoints: endpoints(),
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
                endpoints: endpoints(),
            },
            CapturedEndpoint {
                word: second + 1,
                endpoints: endpoints(),
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
        let usage = context.last_attempt().unwrap().usage();
        assert!(usage.peak(RenderResource::RetainedCpuBytes) >= 1024 * 4);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::SerializedBytes), (512 + 5 + 1) * 4);
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
            endpoints: endpoints(),
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
        let usage = context.last_attempt().unwrap().usage();
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
            endpoints: endpoints(),
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
        let usage = context.last_attempt().unwrap().usage();
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
        let report = context.last_attempt().unwrap();
        assert_eq!(
            report.refusal().unwrap().quantity(),
            crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes)
        );
        assert_eq!(report.usage().peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::SerializedBytes), 0);
        assert_eq!(packed, original);
    }

    #[test]
    fn capture_backing_refusal_leaves_no_retained_record() {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(RenderLimits::new().with_retained_bytes(0)),
        );
        let mut records = EndpointRecords::default();
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                records.capture(0, endpoints(), DrawTag::LINEAR_GRADIENT, account)
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(records.as_slice().is_empty());
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::RasterResourceCopies), 0);
    }
}
