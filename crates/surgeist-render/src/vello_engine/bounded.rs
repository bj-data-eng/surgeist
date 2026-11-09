//! Capacity qualification for the locked Vello public stream producers.
//!
//! Resolver's private caches are an opaque dependency boundary. Its caller-owned
//! input streams and packed output are not: every backing is admitted before the
//! dependency can write, including transient PathEncoder writes later truncated.

use crate::Result;
pub(super) use crate::accounting::PackedProducer;
#[cfg(test)]
pub(super) use crate::accounting::packed_size;
use crate::accounting::{
    Account, AccountedVec, Hold, RenderResource, RenderWork, StorageContractError,
    StorageSizing as Sizing, reserve_backing_with,
};
use peniko::BrushRef;
use vello_encoding::{Encoding, StreamOffsets};

pub(super) struct EncodingBacking {
    holds: [Option<Hold>; 11],
    resource: RenderResource,
}
impl Default for EncodingBacking {
    fn default() -> Self {
        Self {
            holds: Default::default(),
            resource: RenderResource::RetainedCpuBytes,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Growth {
    pub(super) path_tags: usize,
    pub(super) path_data: usize,
    pub(super) draw_tags: usize,
    pub(super) draw_data: usize,
    pub(super) transforms: usize,
    pub(super) styles: usize,
    pub(super) patches: usize,
    pub(super) color_stops: usize,
    pub(super) glyphs: usize,
    pub(super) glyph_runs: usize,
    pub(super) normalized_coords: usize,
}

pub(super) fn add(a: usize, b: usize, account: &Account) -> Result<usize> {
    Sizing::new(account, RenderResource::RetainedCpuBytes).add(a, b)
}

impl EncodingBacking {
    pub(super) fn temporary() -> Self {
        Self {
            resource: RenderResource::ScratchCpuBytes,
            ..Self::default()
        }
    }

    pub(super) fn reserve(
        &mut self,
        encoding: &mut Encoding,
        growth: Growth,
        account: &Account,
    ) -> Result<()> {
        let storage = account.storage();
        let resource = self.resource;
        let sizing = Sizing::new(account, resource);
        macro_rules! stream {
            ($index:literal, $vector:expr, $extra:expr) => {{
                let vector = &mut $vector;
                let needed = sizing.add(vector.len(), $extra)?;
                // Keep amortized growth while qualifying the whole replacement
                // before the dependency receives the vector. These are sizing
                // facts; Ledger remains the sole capacity/overlap authority.
                let total = if needed > vector.capacity() {
                    needed.max(sizing.mul(vector.capacity(), 2)?)
                } else {
                    needed
                };
                reserve_backing_with(
                    vector,
                    &mut self.holds[$index],
                    total,
                    &storage,
                    resource,
                    |_| Ok(()),
                )?;
            }};
        }
        stream!(0, encoding.path_tags, growth.path_tags);
        stream!(1, encoding.path_data, growth.path_data);
        stream!(2, encoding.draw_tags, growth.draw_tags);
        stream!(3, encoding.draw_data, growth.draw_data);
        stream!(4, encoding.transforms, growth.transforms);
        stream!(5, encoding.styles, growth.styles);
        stream!(6, encoding.resources.patches, growth.patches);
        stream!(7, encoding.resources.color_stops, growth.color_stops);
        stream!(8, encoding.resources.glyphs, growth.glyphs);
        stream!(9, encoding.resources.glyph_runs, growth.glyph_runs);
        stream!(
            10,
            encoding.resources.normalized_coords,
            growth.normalized_coords
        );
        Ok(())
    }

    pub(super) fn path(
        &mut self,
        encoding: &mut Encoding,
        count: usize,
        account: &Account,
    ) -> Result<()> {
        let sizing = Sizing::new(account, self.resource);
        self.reserve(
            encoding,
            Growth {
                path_tags: sizing.add(sizing.mul(count, 2)?, 2)?,
                path_data: sizing.add(sizing.mul(count, 8)?, 4)?,
                ..Growth::default()
            },
            account,
        )
    }

    pub(super) fn dispose(mut self, mut encoding: Encoding) {
        fn vector<T>(values: Vec<T>, hold: Option<Hold>) {
            if let Some(hold) = hold {
                hold.dispose(|| drop(values));
            } else {
                drop(values);
            }
        }
        vector(
            std::mem::take(&mut encoding.path_tags),
            self.holds[0].take(),
        );
        vector(
            std::mem::take(&mut encoding.path_data),
            self.holds[1].take(),
        );
        vector(
            std::mem::take(&mut encoding.draw_tags),
            self.holds[2].take(),
        );
        vector(
            std::mem::take(&mut encoding.draw_data),
            self.holds[3].take(),
        );
        vector(
            std::mem::take(&mut encoding.transforms),
            self.holds[4].take(),
        );
        vector(std::mem::take(&mut encoding.styles), self.holds[5].take());
        vector(
            std::mem::take(&mut encoding.resources.patches),
            self.holds[6].take(),
        );
        vector(
            std::mem::take(&mut encoding.resources.color_stops),
            self.holds[7].take(),
        );
        vector(
            std::mem::take(&mut encoding.resources.glyphs),
            self.holds[8].take(),
        );
        vector(
            std::mem::take(&mut encoding.resources.glyph_runs),
            self.holds[9].take(),
        );
        vector(
            std::mem::take(&mut encoding.resources.normalized_coords),
            self.holds[10].take(),
        );
    }

    /// Copies all eleven streams without invoking Encoding::clone's infallible
    /// allocating path. The source and destination holds coexist during copying.
    pub(super) fn copy(encoding: &Encoding, account: &Account) -> Result<OwnedEncoding> {
        let mut owned = OwnedEncoding::default();
        let (result, backing) = owned.parts();
        backing.reserve(
            result,
            Growth {
                path_tags: encoding.path_tags.len(),
                path_data: encoding.path_data.len(),
                draw_tags: encoding.draw_tags.len(),
                draw_data: encoding.draw_data.len(),
                transforms: encoding.transforms.len(),
                styles: encoding.styles.len(),
                patches: encoding.resources.patches.len(),
                color_stops: encoding.resources.color_stops.len(),
                glyphs: encoding.resources.glyphs.len(),
                glyph_runs: encoding.resources.glyph_runs.len(),
                normalized_coords: encoding.resources.normalized_coords.len(),
            },
            account,
        )?;
        // This public append is one indivisible nonallocating dependency copy
        // after every destination backing has been admitted.
        let lengths = [
            encoding.path_tags.len(),
            encoding.path_data.len(),
            encoding.draw_tags.len(),
            encoding.draw_data.len(),
            encoding.transforms.len(),
            encoding.styles.len(),
            encoding.resources.patches.len(),
            encoding.resources.color_stops.len(),
            encoding.resources.glyphs.len(),
            encoding.resources.glyph_runs.len(),
            encoding.resources.normalized_coords.len(),
        ];
        let mut changes = [(RenderWork::EncodingStreamCopies, 0); 11];
        for (change, count) in changes.iter_mut().zip(lengths) {
            change.1 = crate::accounting::as_u64(count)?;
        }
        account.work_batch(changes)?;
        // Resource patches share immutable image/font payloads. No nested Vec
        // clone is hidden in this public append.
        result.append(encoding, &None);
        Ok(owned)
    }
}

pub(super) fn brush_growth(brush: BrushRef<'_>) -> Growth {
    use std::mem::size_of;
    use vello_encoding::{
        DrawColor, DrawImage, DrawLinearGradient, DrawRadialGradient, DrawSweepGradient,
    };
    let (draw_data, color_stops, patches) = match brush {
        BrushRef::Solid(_) => (size_of::<DrawColor>() / 4, 0, 0),
        BrushRef::Image(_) => (size_of::<DrawImage>() / 4, 0, 1),
        BrushRef::Gradient(gradient) => {
            let words = match gradient.kind {
                peniko::GradientKind::Linear(_) => size_of::<DrawLinearGradient>() / 4,
                peniko::GradientKind::Radial(_) => size_of::<DrawRadialGradient>() / 4,
                peniko::GradientKind::Sweep(_) => size_of::<DrawSweepGradient>() / 4,
            };
            (words, gradient.stops.len(), 1)
        }
    };
    Growth {
        draw_tags: 1,
        draw_data,
        color_stops,
        patches,
        ..Growth::default()
    }
}

pub(super) fn materialize(
    elements: impl IntoIterator<Item = kurbo::PathEl>,
    account: &Account,
) -> Result<AccountedVec<kurbo::PathEl>> {
    let elements = elements.into_iter();
    // Exact slice/path iterators can reserve once. Unknown provider/dash output
    // grows geometrically through the same fallible old/new backing protocol.
    let mut result = account.temporary(elements.size_hint().0)?;
    for element in elements {
        account.work(RenderWork::PathElementVisits, 1)?;
        push_element(&mut result, element)?;
    }
    Ok(result)
}

pub(super) fn push_element(
    elements: &mut AccountedVec<kurbo::PathEl>,
    element: kurbo::PathEl,
) -> Result<()> {
    if elements.len() == elements.capacity() {
        elements.grow(elements.capacity().max(4))?;
    }
    elements.push(element)
}

pub(super) fn sum_streams(
    target: &mut StreamOffsets,
    source: &StreamOffsets,
    account: &Account,
) -> Result<()> {
    let sizing = Sizing::new(account, RenderResource::RetainedCpuBytes);
    target.path_tags = sizing.add(target.path_tags, source.path_tags)?;
    target.path_data = sizing.add(target.path_data, source.path_data)?;
    target.draw_tags = sizing.add(target.draw_tags, source.draw_tags)?;
    target.draw_data = sizing.add(target.draw_data, source.draw_data)?;
    target.transforms = sizing.add(target.transforms, source.transforms)?;
    target.styles = sizing.add(target.styles, source.styles)?;
    Ok(())
}

/// Imported immutable payload leases outlive the forward attempt without keeping
/// its ledger or callbacks alive. Registry identities deduplicate shared sources.
#[derive(Default)]
pub(super) struct PayloadCustody {
    imports: Option<AccountedVec<crate::accounting::CpuImport>>,
}
impl PayloadCustody {
    pub(super) fn keep(
        &mut self,
        import: Option<crate::accounting::CpuImport>,
        account: &Account,
    ) -> Result<()> {
        if let Some(import) = import {
            let imports = match &mut self.imports {
                Some(imports) => imports,
                slot @ None => slot.insert(account.retained(0)?),
            };
            push_record(imports, import, account)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct OwnedEncoding {
    encoding: Encoding,
    backing: EncodingBacking,
}
impl OwnedEncoding {
    pub(super) fn temporary() -> Self {
        Self {
            encoding: Encoding::default(),
            backing: EncodingBacking::temporary(),
        }
    }
    pub(super) fn parts(&mut self) -> (&mut Encoding, &mut EncodingBacking) {
        (&mut self.encoding, &mut self.backing)
    }
}
impl std::ops::Deref for OwnedEncoding {
    type Target = Encoding;
    fn deref(&self) -> &Encoding {
        &self.encoding
    }
}
impl Drop for OwnedEncoding {
    fn drop(&mut self) {
        std::mem::take(&mut self.backing).dispose(std::mem::take(&mut self.encoding));
    }
}

/// Storage owns and admits actual replacement-backing relocation. This helper
/// only requests the existing record-capacity growth policy.
pub(super) fn reserve_record<T>(values: &mut AccountedVec<T>) -> Result<()> {
    if values.len() == values.capacity() {
        values.grow(values.capacity().max(1))?;
    }
    Ok(())
}
pub(super) fn push_record<T>(
    values: &mut AccountedVec<T>,
    value: T,
    account: &Account,
) -> Result<()> {
    reserve_record(values)?;
    let slot = values.append_slot()?;
    account.work(RenderWork::RasterResourceCopies, 1)?;
    slot.write(value);
    Ok(())
}

pub(super) struct RasterRegistry<K, V> {
    values: Option<AccountedVec<(K, V)>>,
    resource: RenderResource,
}
impl<K, V> Default for RasterRegistry<K, V> {
    fn default() -> Self {
        Self {
            values: None,
            resource: RenderResource::RetainedCpuBytes,
        }
    }
}
impl<K: Ord, V> RasterRegistry<K, V> {
    pub(super) fn temporary() -> Self {
        Self {
            resource: RenderResource::ScratchCpuBytes,
            ..Self::default()
        }
    }
    fn position(
        values: &[(K, V)],
        key: &K,
        account: &Account,
    ) -> Result<std::result::Result<usize, usize>> {
        let (mut lo, mut hi) = (0, values.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            account.work(RenderWork::ResourceProbes, 1)?;
            let record = values
                .get(mid)
                .ok_or(StorageContractError::InvalidPosition {
                    index: mid,
                    len: values.len(),
                })?;
            match record.0.cmp(key) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return Ok(Ok(mid)),
            }
        }
        Ok(Err(lo))
    }
    pub(super) fn get(&self, key: &K, account: &Account) -> Result<Option<&V>> {
        let Some(values) = &self.values else {
            return Ok(None);
        };
        match Self::position(values.as_slice(), key, account)? {
            Ok(index) => {
                let record = values
                    .get(index)
                    .ok_or(StorageContractError::InvalidPosition {
                        index,
                        len: values.len(),
                    })?;
                Ok(Some(&record.1))
            }
            Err(_) => Ok(None),
        }
    }
    pub(super) fn insert(&mut self, key: K, value: V, account: &Account) -> Result<()> {
        let values = match &mut self.values {
            Some(values) => values,
            slot @ None => slot.insert(AccountedVec::new(account, 0, self.resource)?),
        };
        match Self::position(values.as_slice(), &key, account)? {
            Ok(index) => {
                let len = values.len();
                let record = values
                    .get_mut(index)
                    .ok_or(StorageContractError::InvalidPosition { index, len })?;
                account.work(RenderWork::RasterResourceCopies, 1)?;
                *record = (key, value);
            }
            Err(index) => {
                reserve_record(values)?;
                let slot = values.insert_slot(index)?;
                account.work(RenderWork::RasterResourceCopies, slot.movement_count())?;
                slot.write((key, value));
            }
        }
        Ok(())
    }
    pub(super) fn is_empty(&self) -> bool {
        self.values.as_ref().is_none_or(|values| values.is_empty())
    }
    #[cfg(test)]
    pub(super) fn from_records(values: impl IntoIterator<Item = (K, V)>) -> Self {
        let account = Account::for_test();
        let mut result = Self::default();
        for (key, value) in values {
            result
                .insert(key, value, &account)
                .expect("fixture registry");
        }
        result
    }
}

/// Public PathEncoder consumes each already materialized element only after its
/// actual visit is admitted. A refusal can stop this private unpublished scene.
pub(super) fn encode_elements(
    encoding: &mut Encoding,
    elements: &[kurbo::PathEl],
    fill: bool,
    transform: &super::geometry::QualifiedTransform,
    account: &Account,
) -> Result<bool> {
    encode_elements_with_marker(encoding, elements, fill, true, transform, account)
}

/// Marker suppression is used by native outline preflight. It does not suppress
/// PathEncoder's implicit fill closure or change its stroke marker policy.
pub(super) fn encode_elements_with_marker(
    encoding: &mut Encoding,
    elements: &[kurbo::PathEl],
    fill: bool,
    insert_path_marker: bool,
    transform: &super::geometry::QualifiedTransform,
    account: &Account,
) -> Result<bool> {
    use super::geometry::{NativeElement, PathQualifier};
    if !super::geometry::has_segments(elements, account)? {
        return Ok(false);
    }
    let mut qualifier = PathQualifier::new(fill);
    let mut path = encoding.encode_path(fill);
    for (index, element) in elements.iter().enumerate() {
        account.work(RenderWork::PathElementVisits, 1)?;
        match qualifier.element(*element, index, transform, account)? {
            NativeElement::Move([x, y]) => path.move_to(x, y),
            NativeElement::Line([x, y]) => path.line_to(x, y),
            NativeElement::Quad([x, y], [u, v]) => path.quad_to(x, y, u, v),
            NativeElement::Cubic([x, y], [u, v], [s, t]) => path.cubic_to(x, y, u, v, s, t),
            NativeElement::Close => path.close(),
            NativeElement::Empty => (),
        }
    }
    qualifier.finish(account)?;
    Ok(path.finish(insert_path_marker) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{RenderAccountingContext, RenderAttemptKind, StorageAccount};
    use kurbo::PathEl::{ClosePath, CurveTo, LineTo, MoveTo, QuadTo};

    #[test]
    fn refused_indivisible_encoding_copy_commits_no_stream_element_work() {
        use crate::accounting::{
            FrameAttempt, FrameReportSlot, RenderAccountingOptions, RenderAttemptStatus,
            RenderCpuScope, RenderLimits, RendererCpu,
        };
        let image = crate::Image::from_rgba(crate::Size::new(0.0, 3.0), Vec::<u8>::new()).unwrap();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image.data, 1.0);
        assert_eq!(
            (
                encoding.draw_tags.len(),
                encoding.draw_data.len(),
                encoding.resources.patches.len()
            ),
            (1, 3, 1)
        );
        for limit in [1, 4, 5] {
            let options = RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::EncodingStreamCopies, limit)
                    .unwrap(),
            );
            let cpu = RendererCpu::new(options).unwrap();
            let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
            let result = EncodingBacking::copy(&encoding, attempt.account());
            let (status, diagnostic) = match &result {
                Ok(copy) => {
                    assert_eq!(copy.draw_data, encoding.draw_data);
                    assert!(copy.draw_tags == encoding.draw_tags);
                    assert_eq!(copy.resources.patches.len(), 1);
                    (RenderAttemptStatus::Succeeded, None)
                }
                Err(error) => (attempt.status_for(error), error.accounting_diagnostic()),
            };
            assert_eq!(result.is_ok(), limit == 5);
            drop(result);
            let (report, fault) = attempt.finish(status, diagnostic);
            assert_eq!(fault, None);
            let observed = report.attempt().unwrap();
            assert_eq!(
                observed.usage().work(RenderWork::EncodingStreamCopies),
                if limit == 5 { 5 } else { 0 },
                "the public append copies all five elements or is never invoked"
            );
            if limit < 5 {
                assert_eq!(
                    observed.refusal().unwrap().quantity,
                    crate::RenderQuantity::Work(RenderWork::EncodingStreamCopies)
                );
            }
            let closing = report.renderer_cpu().unwrap();
            assert_eq!(
                closing
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
            assert_eq!(encoding.resources.patches.len(), 1);
        }
    }

    fn run_with_cap(
        cap: usize,
        run: impl FnOnce(&Account) -> Result<()>,
    ) -> RenderAccountingContext {
        let mut context = RenderAccountingContext::default();
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                crate::GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, cap),
                #[cfg(feature = "render-text")]
                None,
                run,
            )
            .unwrap();
        context
    }

    #[test]
    fn move_line_fill_has_literal_closed_streams_without_capacity_growth() {
        run_with_cap(usize::MAX, |account| {
            let mut encoding = Encoding::default();
            let mut backing = EncodingBacking::default();
            let input = [MoveTo((1.0, 2.0).into()), LineTo((3.0, 4.0).into())];
            backing.path(&mut encoding, input.len(), account)?;
            let capacities = (encoding.path_tags.capacity(), encoding.path_data.capacity());
            assert!(encode_elements(
                &mut encoding,
                &input,
                true,
                &super::super::geometry::qualify_transform(kurbo::Affine::IDENTITY, account)?,
                account
            )?);
            assert_eq!(
                encoding
                    .path_tags
                    .iter()
                    .map(|tag| tag.0)
                    .collect::<Vec<_>>(),
                [9, 13, 16]
            );
            assert_eq!(
                encoding.path_data,
                [
                    1_f32.to_bits(),
                    2_f32.to_bits(),
                    3_f32.to_bits(),
                    4_f32.to_bits(),
                    1_f32.to_bits(),
                    2_f32.to_bits()
                ]
            );
            assert_eq!(
                capacities,
                (encoding.path_tags.capacity(), encoding.path_data.capacity())
            );
            Ok(())
        });
    }

    #[test]
    fn materialized_path_bounds_cover_truncated_and_stroke_cap_writes() {
        let cases: &[&[kurbo::PathEl]] = &[
            &[],
            &[MoveTo((0.0, 0.0).into())],
            &[LineTo((1.0, 2.0).into())],
            &[MoveTo((0.0, 0.0).into()), MoveTo((1.0, 1.0).into())],
            &[
                MoveTo((0.0, 0.0).into()),
                LineTo((0.0, 0.0).into()),
                ClosePath,
            ],
            &[
                MoveTo((0.0, 0.0).into()),
                QuadTo((1.0, 3.0).into(), (2.0, 0.0).into()),
            ],
            &[
                MoveTo((0.0, 0.0).into()),
                CurveTo((1.0, 3.0).into(), (2.0, 3.0).into(), (3.0, 0.0).into()),
            ],
            &[
                MoveTo((0.0, 0.0).into()),
                LineTo((1.0, 2.0).into()),
                MoveTo((3.0, 4.0).into()),
                LineTo((5.0, 6.0).into()),
                ClosePath,
            ],
        ];
        for input in cases {
            for fill in [false, true] {
                run_with_cap(usize::MAX, |account| {
                    let mut encoding = Encoding::default();
                    let mut backing = EncodingBacking::default();
                    backing.path(&mut encoding, input.len(), account)?;
                    let capacities = (encoding.path_tags.capacity(), encoding.path_data.capacity());
                    encode_elements(
                        &mut encoding,
                        input,
                        fill,
                        &super::super::geometry::qualify_transform(
                            kurbo::Affine::IDENTITY,
                            account,
                        )?,
                        account,
                    )?;
                    assert_eq!(
                        capacities,
                        (encoding.path_tags.capacity(), encoding.path_data.capacity())
                    );
                    assert!(encoding.path_tags.len() <= 2 * input.len() + 2);
                    assert!(encoding.path_data.len() <= 8 * input.len() + 4);
                    Ok(())
                });
            }
        }
    }

    #[test]
    fn one_under_cpu_cap_refuses_before_path_stream_mutation() {
        // Two materialized inputs require 6 tag bytes and 20 data words.
        let context = run_with_cap(85, |account| {
            let mut encoding = Encoding::default();
            let mut backing = EncodingBacking::default();
            assert!(backing.path(&mut encoding, 2, account).is_err());
            assert!(encoding.path_tags.is_empty());
            assert!(encoding.path_data.is_empty());
            assert_eq!(encoding.path_data.capacity(), 0);
            Ok(())
        });
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            6
        );
        let context = run_with_cap(86, |account| {
            let mut encoding = Encoding::default();
            let mut backing = EncodingBacking::default();
            backing.path(&mut encoding, 2, account)?;
            assert_eq!(encoding.path_tags.capacity(), 6);
            assert_eq!(encoding.path_data.capacity(), 20);
            Ok(())
        });
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            86
        );
    }

    #[test]
    fn replacement_admission_keeps_old_stream_live_until_new_backing_is_admitted() {
        let context = run_with_cap(151, |account| {
            let mut encoding = Encoding::default();
            let mut backing = EncodingBacking::default();
            backing.path(&mut encoding, 1, account)?;
            encoding.path_data.push(17);
            // Amortized replacement is 24 data words (96B). The old 48B
            // and renewed 8 tag bytes stay live: 152B must be admitted before
            // moving either data backing, even though the final 104B fits.
            assert!(backing.path(&mut encoding, 2, account).is_err());
            assert_eq!(encoding.path_data, [17]);
            assert_eq!(encoding.path_data.capacity(), 12);
            Ok(())
        });
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            60
        );
        let context = run_with_cap(152, |account| {
            let mut encoding = Encoding::default();
            let mut backing = EncodingBacking::default();
            backing.path(&mut encoding, 1, account)?;
            encoding.path_data.push(17);
            backing.path(&mut encoding, 2, account)?;
            assert_eq!(encoding.path_tags.capacity(), 8);
            assert_eq!(encoding.path_data.capacity(), 24);
            assert_eq!(encoding.path_data, [17]);
            Ok(())
        });
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            152
        );
    }

    fn solid_serialization_fixture(account: &Account) -> Result<OwnedEncoding> {
        let mut owned = OwnedEncoding::default();
        let (encoding, backing) = owned.parts();
        backing.reserve(
            encoding,
            Growth {
                path_tags: 2,
                transforms: 1,
                styles: 1,
                ..Growth::default()
            },
            account,
        )?;
        encoding.encode_transform(vello_encoding::Transform::IDENTITY);
        encoding.encode_fill_style(peniko::Fill::NonZero);
        let input = [MoveTo((1.0, 2.0).into()), LineTo((3.0, 4.0).into())];
        backing.path(encoding, 2, account)?;
        encode_elements(
            encoding,
            &input,
            true,
            &super::super::geometry::qualify_transform(kurbo::Affine::IDENTITY, account)?,
            account,
        )?;
        backing.reserve(
            encoding,
            brush_growth(peniko::BrushRef::Solid(peniko::Color::BLACK)),
            account,
        )?;
        encoding.encode_color(peniko::Color::BLACK);
        Ok(owned)
    }

    #[test]
    fn packed_solid_streams_match_literal_format_and_locked_resolver() {
        run_with_cap(usize::MAX, |account| {
            let owned = solid_serialization_fixture(account)?;
            let encoding = &owned.encoding;
            let size = packed_size(encoding, &StreamOffsets::default(), account)?;
            assert_eq!(size, 1088); // 1024 + 6*4 + 4 + 4 + 24 + 8
            let packed = PackedProducer::new(encoding, &StreamOffsets::default(), account)?;
            let capacity = packed.capacity();
            let mut resolver = vello_encoding::Resolver::default();
            let (packed, layout, _, _) = packed.resolve(&mut resolver)?.into_parts();
            assert_eq!(packed.len(), 1088);
            assert_eq!(packed.capacity(), capacity);
            assert_eq!(layout.path_data_base, 256);
            let expected_points: Vec<_> = [1.0_f32, 2.0, 3.0, 4.0, 1.0, 2.0]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect();
            assert_eq!(&packed[1024..1048], expected_points);
            assert_eq!(account.work_count(RenderWork::SerializedBytes)?, 1088);
            Ok(())
        });
    }

    #[test]
    fn empty_native_serialization_keeps_admitted_capacity_without_inventing_bytes() {
        let context = run_with_cap(usize::MAX, |account| {
            let encoding = Encoding::default();
            let producer = PackedProducer::new(&encoding, &StreamOffsets::default(), account)?;
            assert!(producer.capacity() >= 4);
            let mut resolver = vello_encoding::Resolver::default();
            let (packed, layout, _, _) = producer.resolve(&mut resolver)?.into_parts();
            assert!(packed.is_empty());
            assert_eq!(layout.n_paths, 0);
            assert_eq!(account.work_count(RenderWork::SerializedBytes)?, 0);
            Ok(())
        });
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
    fn native_serialization_one_short_refusal_drops_intact_owner_and_retries() {
        use crate::accounting::{
            FrameAttempt, FrameReportSlot, RenderAccountingOptions, RenderAttemptStatus,
            RenderCpuScope, RenderLimits, RendererCpu,
        };
        let options = RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::SerializedBytes, 1087)
                .unwrap(),
        );
        let cpu = RendererCpu::new(options).unwrap();
        let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
        let account = attempt.account();
        let owned = solid_serialization_fixture(account).unwrap();
        let producer =
            PackedProducer::new(&owned.encoding, &StreamOffsets::default(), account).unwrap();
        let mut resolver = vello_encoding::Resolver::default();
        let error = producer.resolve(&mut resolver).err().unwrap();
        assert!(error.accounting_diagnostic().is_some());
        drop(owned);
        let (report, fault) =
            attempt.finish(RenderAttemptStatus::Refused, error.accounting_diagnostic());
        assert_eq!(fault, None);
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::SerializedBytes),
            0
        );
        assert_eq!(
            report
                .renderer_cpu()
                .unwrap()
                .scope_usage(RenderCpuScope::FrameBacking)
                .live(RenderResource::RetainedCpuBytes),
            0
        );
        run_with_cap(usize::MAX, |account| {
            let owned = solid_serialization_fixture(account)?;
            let producer =
                PackedProducer::new(&owned.encoding, &StreamOffsets::default(), account)?;
            let mut resolver = vello_encoding::Resolver::default();
            let (packed, _, _, _) = producer.resolve(&mut resolver)?.into_parts();
            assert_eq!(packed.len(), 1088);
            Ok(())
        });
    }

    #[test]
    fn packed_padding_includes_implicit_open_clip_endings() {
        let mut encoding = Encoding::default();
        encoding
            .path_tags
            .resize(1024, vello_encoding::PathTag::PATH);
        assert_eq!(
            packed_size(&encoding, &StreamOffsets::default(), &Account::for_test()).unwrap(),
            1024
        );
        encoding.n_open_clips = 1;
        assert_eq!(
            packed_size(&encoding, &StreamOffsets::default(), &Account::for_test()).unwrap(),
            2052
        );
    }

    fn capacities(encoding: &Encoding) -> [usize; 11] {
        [
            encoding.path_tags.capacity(),
            encoding.path_data.capacity(),
            encoding.draw_tags.capacity(),
            encoding.draw_data.capacity(),
            encoding.transforms.capacity(),
            encoding.styles.capacity(),
            encoding.resources.patches.capacity(),
            encoding.resources.color_stops.capacity(),
            encoding.resources.glyphs.capacity(),
            encoding.resources.glyph_runs.capacity(),
            encoding.resources.normalized_coords.capacity(),
        ]
    }

    #[test]
    fn zero_one_and_many_gradient_stops_stay_within_admitted_brush_streams() {
        for count in [0, 1, 5] {
            run_with_cap(usize::MAX, |account| {
                let colors = [peniko::Color::BLACK; 5];
                let gradient = peniko::Gradient::new_linear((0.0, 0.0), (1.0, 0.0))
                    .with_stops(&colors[..count]);
                let brush = peniko::BrushRef::Gradient(&gradient);
                let mut encoding = Encoding::default();
                let mut backing = EncodingBacking::default();
                backing.reserve(&mut encoding, brush_growth(brush), account)?;
                let before = capacities(&encoding);
                encoding.encode_brush(brush, 1.0);
                assert_eq!(capacities(&encoding), before);
                assert_eq!(encoding.draw_tags.len(), 1);
                assert_eq!(encoding.draw_data.len(), if count < 2 { 1 } else { 5 });
                assert_eq!(
                    encoding.resources.color_stops.len(),
                    if count < 2 { 0 } else { 5 }
                );
                assert_eq!(encoding.resources.patches.len(), usize::from(count >= 2));
                Ok(())
            });
        }
    }

    #[test]
    fn forced_state_blur_and_clip_formats_keep_their_reserved_capacity() {
        run_with_cap(usize::MAX, |account| {
            let mut encoding = Encoding::default();
            let mut backing = EncodingBacking::default();
            for _ in 0..2 {
                backing.reserve(
                    &mut encoding,
                    Growth {
                        path_tags: 2,
                        transforms: 1,
                        styles: 1,
                        ..Growth::default()
                    },
                    account,
                )?;
                let before = capacities(&encoding);
                encoding.encode_transform(vello_encoding::Transform::IDENTITY);
                encoding.encode_fill_style(peniko::Fill::NonZero);
                assert_eq!(capacities(&encoding), before);
                encoding.force_next_transform_and_style();
            }
            assert_eq!(
                encoding
                    .path_tags
                    .iter()
                    .map(|tag| tag.0)
                    .collect::<Vec<_>>(),
                [32, 64, 32, 64]
            );
            backing.reserve(
                &mut encoding,
                Growth {
                    draw_tags: 1,
                    draw_data: 5,
                    ..Growth::default()
                },
                account,
            )?;
            let before = capacities(&encoding);
            encoding.encode_blurred_rounded_rect(peniko::Color::BLACK, 2.0, 3.0, 0.5, 1.0);
            assert_eq!(capacities(&encoding), before);
            assert_eq!(encoding.draw_data.len(), 5);
            backing.reserve(
                &mut encoding,
                Growth {
                    draw_tags: 1,
                    draw_data: 2,
                    ..Growth::default()
                },
                account,
            )?;
            let before = capacities(&encoding);
            encoding.encode_begin_clip(vello_encoding::DrawBeginClip::clip());
            assert_eq!(capacities(&encoding), before);
            backing.reserve(
                &mut encoding,
                Growth {
                    path_tags: 1,
                    draw_tags: 1,
                    ..Growth::default()
                },
                account,
            )?;
            let before = capacities(&encoding);
            encoding.encode_end_clip();
            assert_eq!(capacities(&encoding), before);
            assert_eq!(encoding.n_open_clips, 0);
            assert_eq!(encoding.draw_data.len(), 7);
            Ok(())
        });
    }

    #[test]
    fn append_moves_literal_contents_of_all_eleven_admitted_streams() {
        run_with_cap(usize::MAX, |account| {
            let mut source = Encoding::default();
            source.path_tags.push(vello_encoding::PathTag::PATH);
            source.path_data.push(17);
            source.draw_tags.push(vello_encoding::DrawTag::COLOR);
            source.draw_data.push(23);
            source.transforms.push(vello_encoding::Transform::IDENTITY);
            source
                .styles
                .push(vello_encoding::Style::from_fill(peniko::Fill::NonZero));
            source
                .resources
                .patches
                .push(vello_encoding::Patch::GlyphRun { index: 0 });
            source
                .resources
                .color_stops
                .push(peniko::ColorStop::from((0.0, peniko::Color::BLACK)));
            source.resources.glyphs.push(vello_encoding::Glyph {
                id: 7,
                x: 2.0,
                y: 3.0,
            });
            source.resources.normalized_coords.push(0);
            source.resources.glyph_runs.push(vello_encoding::GlyphRun {
                font: peniko::FontData::new(
                    peniko::Blob::new(std::sync::Arc::new(Vec::<u8>::new())),
                    0,
                ),
                transform: vello_encoding::Transform::IDENTITY,
                glyph_transform: None,
                brush_transform: None,
                font_size: 16.0,
                font_embolden: vello_encoding::FontEmbolden::default(),
                hint: false,
                normalized_coords: 0..1,
                style: peniko::Fill::NonZero.into(),
                glyphs: 0..1,
                stream_offsets: StreamOffsets::default(),
            });
            let copy = EncodingBacking::copy(&source, account)?;
            assert_eq!(capacities(&copy), [1; 11]);
            assert_eq!(copy.path_data, [17]);
            assert_eq!(copy.draw_data, [23]);
            assert_eq!(copy.resources.glyphs[0].id, 7);
            assert_eq!(copy.resources.glyph_runs[0].glyphs, 0..1);
            assert_eq!(copy.resources.glyph_runs[0].normalized_coords, 0..1);
            assert_eq!(copy.resources.normalized_coords, [0]);
            assert!(matches!(
                copy.resources.patches[0],
                vello_encoding::Patch::GlyphRun { index: 0 }
            ));
            assert_eq!(account.work_count(RenderWork::EncodingStreamCopies)?, 11);
            Ok(())
        });
    }

    #[test]
    fn unknown_materialized_output_uses_admitted_geometric_backings() {
        let element_bytes = std::mem::size_of::<kurbo::PathEl>();
        for allowed in [24 * element_bytes - 1, 24 * element_bytes] {
            let mut context = RenderAccountingContext::default();
            context
                .prepare(
                    RenderAttemptKind::GlyphRun,
                    crate::GlyphRunLimits::new(usize::MAX, usize::MAX, allowed, usize::MAX),
                    #[cfg(feature = "render-text")]
                    None,
                    |account| {
                        let mut next = 0;
                        let source = std::iter::from_fn(|| {
                            if next == 10 {
                                return None;
                            }
                            let point = MoveTo((f64::from(next), 0.0).into());
                            next += 1;
                            Some(point)
                        });
                        let result = materialize(source, account);
                        if allowed == 24 * element_bytes {
                            let values = result?;
                            assert_eq!(values.len(), 10);
                            assert_eq!(values.capacity(), 16);
                            assert_eq!(values[9], MoveTo((9.0, 0.0).into()));
                        } else {
                            assert!(result.is_err());
                        }
                        Ok(())
                    },
                )
                .unwrap();
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(
                usage.peak(RenderResource::ScratchCpuBytes),
                if allowed == 24 * element_bytes {
                    (24 * element_bytes) as u64
                } else {
                    (12 * element_bytes) as u64
                }
            );
            assert_eq!(
                usage.work(RenderWork::PathElementVisits),
                if allowed == 24 * element_bytes { 10 } else { 9 }
            );
        }
    }

    #[test]
    fn sorted_raster_registry_preserves_values_and_counts_actual_record_moves() {
        run_with_cap(usize::MAX, |account| {
            let mut registry = RasterRegistry::<usize, u32>::default();
            registry.insert(20, 2, account)?;
            registry.insert(10, 1, account)?;
            registry.insert(20, 3, account)?;
            assert_eq!(registry.get(&10, account)?, Some(&1));
            assert_eq!(registry.get(&20, account)?, Some(&3));
            assert_eq!(account.work_count(RenderWork::RasterResourceCopies)?, 4);
            assert_eq!(account.work_count(RenderWork::StorageRelocations)?, 1);
            Ok(())
        });
    }

    #[test]
    fn registry_replacement_refusal_preserves_old_record_without_claiming_moves() {
        let bytes = std::mem::size_of::<(usize, u32)>();
        let context = run_with_cap(3 * bytes - 1, |account| {
            let mut registry = RasterRegistry::<usize, u32>::default();
            registry.insert(20, 2, account)?;
            assert!(registry.insert(10, 1, account).is_err());
            assert_eq!(
                registry.values.as_ref().expect("old registry").as_slice(),
                &[(20, 2)]
            );
            assert_eq!(account.work_count(RenderWork::RasterResourceCopies)?, 1);
            Ok(())
        });
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            bytes as u64
        );
    }

    #[test]
    fn packed_size_overflow_identifies_renderer_scope_and_retained_backing() {
        let error = packed_size(
            &Encoding::default(),
            &StreamOffsets {
                path_tags: usize::MAX,
                ..StreamOffsets::default()
            },
            &Account::for_test(),
        )
        .unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(crate::RenderAccountingError::CpuCapacityOverflow {
                scope: crate::RenderCpuScope::FrameBacking,
                resource: RenderResource::RetainedCpuBytes
            })
        );
    }

    #[test]
    fn encoding_disposal_times_the_real_glyph_font_destructor_with_backing_live() {
        use crate::accounting::{
            RenderAccountingOptions, RenderClock, RenderCpuScope, RenderProfile, RendererCpu,
        };
        use std::sync::atomic::{AtomicU64, Ordering};
        static TIME: AtomicU64 = AtomicU64::new(0);
        fn now() -> std::time::Duration {
            std::time::Duration::from_millis(TIME.load(Ordering::Relaxed))
        }
        struct FontBytes;
        impl AsRef<[u8]> for FontBytes {
            fn as_ref(&self) -> &[u8] {
                &[]
            }
        }
        impl Drop for FontBytes {
            fn drop(&mut self) {
                TIME.store(17, Ordering::Relaxed);
            }
        }
        TIME.store(0, Ordering::Relaxed);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_clock(RenderClock::Host(now))
                .with_profile(RenderProfile::Summary { timing: true }),
        )
        .unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let mut runs = AccountedVec::new_in(&storage, 1, RenderResource::RetainedCpuBytes).unwrap();
        runs.push(vello_encoding::GlyphRun {
            font: peniko::FontData::new(peniko::Blob::new(std::sync::Arc::new(FontBytes)), 0),
            transform: vello_encoding::Transform::IDENTITY,
            glyph_transform: None,
            brush_transform: None,
            font_size: 16.0,
            font_embolden: vello_encoding::FontEmbolden::default(),
            hint: false,
            normalized_coords: 0..0,
            style: peniko::Fill::NonZero.into(),
            glyphs: 0..0,
            stream_offsets: StreamOffsets::default(),
        })
        .unwrap();
        let (values, hold) = runs.into_parts().unwrap();
        let mut owned = OwnedEncoding::default();
        owned.encoding.resources.glyph_runs = values;
        owned.backing.holds[9] = Some(hold);
        TIME.store(5, Ordering::Relaxed);
        drop(owned);
        let snapshot = cpu.snapshot().unwrap();
        assert_eq!(
            snapshot.usage().live(RenderResource::RetainedCpuBytes),
            baseline
        );
        assert_eq!(
            snapshot.scope_elapsed(RenderCpuScope::FrameBacking),
            Some(std::time::Duration::from_millis(12))
        );
    }
}
