//! Shared strict used metric variation storage and checked delta accumulation.
//! MVAR retains whole-unit rounding; BASE retains fractional design units.
//!
//! Parsing and region interpolation use the retained safe read-fonts/Skrifa
//! APIs. Completeness, nullable-index semantics and widening are Text-owned;
//! this does not use their permissive high-level metric-delta defaults.

use skrifa::{
    instance::NormalizedCoord,
    raw::{
        FontData, FontRead,
        tables::{
            mvar::Mvar,
            variations::{ItemVariationData, ItemVariationStore, VariationRegionList},
        },
    },
};

use super::{
    EffectiveFontInstance, Error, ErrorCode, ErrorDetail, Result, Table, TableError, table_error,
};

const TAGS: [[u8; 4]; 7] = [
    *b"hasc", *b"hdsc", *b"hlgp", *b"hcla", *b"hcld", *b"stro", *b"strs",
];

pub(super) struct Deltas([f64; 7]);
impl Deltas {
    pub(super) fn get(&self, tag: &[u8; 4]) -> f64 {
        TAGS.iter()
            .position(|actual| actual == tag)
            .map_or(0.0, |at| self.0[at])
    }
}

struct Request {
    record: u16,
    offset: u32,
    inner: u16,
    tag: [u8; 4],
}
struct Data<'a> {
    offset: u32,
    table: ItemVariationData<'a>,
}

pub(super) fn resolve(
    data: Option<FontData<'_>>,
    instance: &EffectiveFontInstance<'_>,
    used: &[[u8; 4]],
) -> Result<Deltas> {
    let mut result = Deltas([0.0; 7]);
    let Some(data) = data else { return Ok(result) };
    if instance.axis_count() == 0 {
        return Err(refuse(TableError::VariationWithoutAxes));
    }
    let mvar = Mvar::read(data).map_err(|_| refuse(TableError::Malformed))?;
    if mvar.version().major != 1 || mvar.version().minor != 0 {
        return Err(refuse(TableError::UnsupportedVersion));
    }
    if data
        .read_at::<u16>(4)
        .map_err(|_| refuse(TableError::Malformed))?
        != 0
        || mvar.value_record_size() < 8
    {
        return Err(refuse(TableError::Malformed));
    }
    if mvar.value_record_size() != 8 {
        return Err(refuse(TableError::UnsupportedRecordSize {
            size: mvar.value_record_size(),
        }));
    }
    let count = usize::from(mvar.value_record_count());
    require_extent(data, add(12, mul(count, 8)?)?)?;
    let records = mvar.value_records();
    if records.len() != count {
        return Err(refuse(TableError::Malformed));
    }
    for (at, pair) in records.windows(2).enumerate() {
        if pair[0].value_tag() >= pair[1].value_tag() {
            return Err(refuse(TableError::VariationRecordOrder {
                record: ordinal(at + 1)?,
            }));
        }
    }
    if count == 0 {
        if !mvar.item_variation_store_offset().is_null() {
            return Err(refuse(TableError::Malformed));
        }
        return Ok(result);
    }
    let store_offset = mvar.item_variation_store_offset().offset().to_u32();
    let store = mvar
        .item_variation_store()
        .ok_or_else(|| {
            refuse(TableError::VariationOffset {
                offset: store_offset,
            })
        })?
        .map_err(|_| {
            refuse(TableError::VariationOffset {
                offset: store_offset,
            })
        })?;
    let strict = StrictStore::new(store, instance, Table::Mvar)?;
    let store_data = strict.source.offset_data();
    let offsets = strict.source.item_variation_data_offsets();
    let mut requests = Vec::new();
    reserve(&mut requests, count)?;
    for (at, record) in records.iter().enumerate() {
        let record_at = ordinal(at)?;
        let outer = record.delta_set_outer_index();
        let inner = record.delta_set_inner_index();
        if outer == u16::MAX && inner == u16::MAX {
            continue;
        }
        let slot = offsets
            .get(usize::from(outer))
            .ok_or_else(|| {
                refuse(TableError::VariationDeltaIndex {
                    record: record_at,
                    outer,
                    inner,
                })
            })?
            .get();
        // A declared null slot is specified zero, independently of inner.
        if slot.is_null() {
            continue;
        }
        requests.push(Request {
            record: record_at,
            offset: slot.offset().to_u32(),
            inner,
            tag: record.value_tag().to_be_bytes(),
        });
    }
    requests.sort_unstable_by_key(|request| (request.offset, request.inner));
    let mut tables = Vec::new();
    reserve(&mut tables, requests.len())?;
    let mut next = 0;
    // Each actual shared data offset and referenced row is qualified once,
    // even if many tags/outer slots alias it. No variation executes yet.
    while next < requests.len() {
        let offset = requests[next].offset;
        let table = read_data(
            store_data,
            offset,
            strict.regions.region_count(),
            Table::Mvar,
        )?;
        while next < requests.len() && requests[next].offset == offset {
            let inner = requests[next].inner;
            if inner >= table.item_count() {
                let request = &requests[next];
                let outer = records[usize::from(request.record)].delta_set_outer_index();
                return Err(refuse(TableError::VariationDeltaIndex {
                    record: request.record,
                    outer,
                    inner,
                }));
            }
            let actual = table.delta_set(inner).count();
            if actual != usize::from(table.region_index_count()) {
                return Err(refuse(TableError::VariationRow {
                    offset,
                    inner,
                    expected: table.region_index_count(),
                    actual,
                }));
            }
            while next < requests.len()
                && requests[next].offset == offset
                && requests[next].inner == inner
            {
                next += 1;
            }
        }
        tables.push(Data { offset, table });
    }

    let scalars = strict.scalars(instance)?;

    next = 0;
    let mut table_at = 0;
    while next < requests.len() {
        let first = next;
        let request = &requests[first];
        while next < requests.len()
            && requests[next].offset == request.offset
            && requests[next].inner == request.inner
        {
            next += 1;
        }
        while tables[table_at].offset < request.offset {
            table_at += 1;
        }
        if !requests[first..next]
            .iter()
            .any(|request| used.contains(&request.tag))
        {
            continue;
        }
        let table = &tables[table_at].table;
        let sum = weighted_sum(table, request.inner, &scalars, request.tag)?;
        let rounded = sum
            .checked_add(0x8000)
            .ok_or_else(|| arithmetic(request.tag))?
            >> 16;
        // At most u16::MAX signed32 terms, each weighted in [0,1]. The
        // integer is below2^47 in magnitude, so its f64 conversion is exact.
        if !(-(1i64 << 47)..(1i64 << 47)).contains(&rounded) {
            return Err(arithmetic(request.tag));
        }
        for request in &requests[first..next] {
            if used.contains(&request.tag)
                && let Some(at) = TAGS.iter().position(|tag| *tag == request.tag)
            {
                result.0[at] = rounded as f64;
            }
        }
    }
    Ok(result)
}

struct StrictStore<'a> {
    source: ItemVariationStore<'a>,
    regions: VariationRegionList<'a>,
    which: Table,
}
impl<'a> StrictStore<'a> {
    fn new(
        store: ItemVariationStore<'a>,
        instance: &EffectiveFontInstance<'_>,
        which: Table,
    ) -> Result<Self> {
        let refuse = |reason| table_error(which, reason);
        if store.format() != 1 {
            return Err(refuse(TableError::VariationStoreFormat {
                format: store.format(),
            }));
        }
        let store_data = store.offset_data();
        let data_count = usize::from(store.item_variation_data_count());
        require_extent_for(which, store_data, add(8, mul(data_count, 4)?)?)?;
        let offsets = store.item_variation_data_offsets();
        if offsets.len() != data_count {
            return Err(refuse(TableError::Malformed));
        }
        let region_offset = store.variation_region_list_offset().to_u32();
        if region_offset == 0 {
            return Err(refuse(TableError::VariationOffset {
                offset: region_offset,
            }));
        }
        let list = store.variation_region_list().map_err(|_| {
            refuse(TableError::VariationOffset {
                offset: region_offset,
            })
        })?;
        let axes = usize::from(list.axis_count());
        if axes != instance.axis_count() {
            return Err(refuse(TableError::VariationAxisCount {
                expected: instance.axis_count(),
                actual: list.axis_count(),
            }));
        }
        let region_count = usize::from(list.region_count());
        if region_count >= 32768 {
            return Err(refuse(TableError::VariationRegionCount {
                count: list.region_count(),
            }));
        }
        require_extent_for(
            which,
            list.offset_data(),
            add(4, mul(mul(region_count, axes)?, 6)?)?,
        )?;
        let regions = list.variation_regions();
        if regions.len() != region_count {
            return Err(refuse(TableError::Malformed));
        }
        for at in 0..region_count {
            let region = regions.get(at).map_err(|_| refuse(TableError::Malformed))?;
            if region.region_axes().len() != axes {
                return Err(refuse(TableError::Malformed));
            }
            for (axis, coordinate) in region.region_axes().iter().enumerate() {
                for value in [
                    coordinate.start_coord(),
                    coordinate.peak_coord(),
                    coordinate.end_coord(),
                ] {
                    let bits = value.to_bits();
                    if !(-16384..=16384).contains(&bits) {
                        return Err(refuse(TableError::VariationRegionCoordinate {
                            region: ordinal(at)?,
                            axis: ordinal(axis)?,
                            value: bits,
                        }));
                    }
                }
            }
        }

        Ok(Self {
            source: store,
            regions: list,
            which,
        })
    }
    fn scalars(&self, instance: &EffectiveFontInstance<'_>) -> Result<Vec<i32>> {
        let refuse = |reason| table_error(self.which, reason);
        let axes = usize::from(self.regions.axis_count());
        let region_count = usize::from(self.regions.region_count());
        let regions = self.regions.variation_regions();
        let mut coords = Vec::new();
        reserve(&mut coords, axes)?;
        coords.extend(instance.normalized_coords().map(NormalizedCoord::from_bits));
        if coords.len() != axes {
            return Err(refuse(TableError::VariationAxisCount {
                expected: axes,
                actual: ordinal(coords.len())?,
            }));
        }
        let mut scalars = Vec::new();
        reserve(&mut scalars, region_count)?;
        for at in 0..region_count {
            let scalar = regions
                .get(at)
                .map_err(|_| refuse(TableError::Malformed))?
                .compute_scalar(&coords)
                .to_bits();
            if !(0..=65536).contains(&scalar) {
                return Err(refuse(TableError::VariationScalar {
                    region: ordinal(at)?,
                    value: scalar,
                }));
            }
            scalars.push(scalar);
        }

        Ok(scalars)
    }
}

/// BASE retains fractional design-unit deltas rather than MVAR's existing
/// whole-unit rounding. Storage and scalar admission have one common owner.
pub(super) fn baseline_delta(
    store: ItemVariationStore<'_>,
    instance: &EffectiveFontInstance<'_>,
    record: u16,
    outer: u16,
    inner: u16,
    tag: [u8; 4],
) -> Result<f64> {
    let refuse = |reason| table_error(Table::Base, reason);
    if instance.axis_count() == 0 {
        return Err(refuse(TableError::VariationWithoutAxes));
    }
    let strict = StrictStore::new(store, instance, Table::Base)?;
    if outer == u16::MAX && inner == u16::MAX {
        return Ok(0.0);
    }
    let slot = strict
        .source
        .item_variation_data_offsets()
        .get(usize::from(outer))
        .ok_or_else(|| {
            refuse(TableError::VariationDeltaIndex {
                record,
                outer,
                inner,
            })
        })?
        .get();
    if slot.is_null() {
        return Ok(0.0);
    }
    let offset = slot.offset().to_u32();
    let data = read_data(
        strict.source.offset_data(),
        offset,
        strict.regions.region_count(),
        Table::Base,
    )?;
    if inner >= data.item_count() {
        return Err(refuse(TableError::VariationDeltaIndex {
            record,
            outer,
            inner,
        }));
    }
    let actual = data.delta_set(inner).count();
    if actual != usize::from(data.region_index_count()) {
        return Err(refuse(TableError::VariationRow {
            offset,
            inner,
            expected: data.region_index_count(),
            actual,
        }));
    }
    let scalars = strict.scalars(instance)?;
    let result = weighted_sum(&data, inner, &scalars, tag)? as f64 / 65536.0;
    if !result.is_finite() {
        return Err(arithmetic(tag));
    }
    Ok(result)
}

fn weighted_sum(
    table: &ItemVariationData<'_>,
    inner: u16,
    scalars: &[i32],
    tag: [u8; 4],
) -> Result<i64> {
    let mut sum = 0i64;
    for (delta, index) in table.delta_set(inner).zip(table.region_indexes()) {
        let scalar = i64::from(scalars[usize::from(index.get())]);
        sum = i64::from(delta)
            .checked_mul(scalar)
            .and_then(|term| sum.checked_add(term))
            .ok_or_else(|| arithmetic(tag))?;
    }
    Ok(sum)
}

fn require_extent_for(which: Table, data: FontData<'_>, len: usize) -> Result<()> {
    if len > data.len() {
        return Err(table_error(which, TableError::Malformed));
    }
    Ok(())
}

fn read_data(
    store: FontData<'_>,
    offset: u32,
    region_count: u16,
    which: Table,
) -> Result<ItemVariationData<'_>> {
    let refuse = |reason| table_error(which, reason);
    let at = usize::try_from(offset).map_err(|_| capacity())?;
    let data = store
        .slice(at..)
        .ok_or_else(|| refuse(TableError::VariationOffset { offset }))?;
    let table = ItemVariationData::read(data)
        .map_err(|_| refuse(TableError::VariationOffset { offset }))?;
    let regions = table.region_index_count();
    let words = table.word_delta_count() & 0x7fff;
    if words > regions {
        return Err(refuse(TableError::VariationWordCount {
            offset,
            words,
            regions,
        }));
    }
    let (wide, short) = if table.word_delta_count() & 0x8000 == 0 {
        (2, 1)
    } else {
        (4, 2)
    };
    let row_bytes = add(
        mul(usize::from(words), wide)?,
        mul(usize::from(regions - words), short)?,
    )?;
    let delta_bytes = mul(usize::from(table.item_count()), row_bytes)?;
    let header_bytes = add(6, mul(usize::from(regions), 2)?)?;
    require_extent_for(which, data, add(header_bytes, delta_bytes)?)?;
    if table.region_indexes().len() != usize::from(regions)
        || table.delta_sets().len() != delta_bytes
    {
        return Err(refuse(TableError::Malformed));
    }
    for index in table.region_indexes() {
        if index.get() >= region_count {
            return Err(refuse(TableError::VariationRegionIndex {
                offset,
                index: index.get(),
            }));
        }
    }
    Ok(table)
}

fn require_extent(data: FontData<'_>, len: usize) -> Result<()> {
    if len > data.len() {
        return Err(refuse(TableError::Malformed));
    }
    Ok(())
}
fn ordinal(at: usize) -> Result<u16> {
    u16::try_from(at).map_err(|_| capacity())
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(capacity)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(capacity)
}
fn reserve<T>(buffer: &mut Vec<T>, additional: usize) -> Result<()> {
    let count = add(buffer.len(), additional)?;
    if mul(count, std::mem::size_of::<T>())? > isize::MAX as usize {
        return Err(capacity());
    }
    buffer.try_reserve(additional).map_err(|_| {
        Error::new(
            ErrorCode::FontMetricCapacity,
            "font metric allocation refused",
        )
        .with_detail(ErrorDetail::FontMetricAllocationRefused)
    })
}
fn capacity() -> Error {
    Error::new(
        ErrorCode::FontMetricCapacity,
        "font metric capacity overflow",
    )
    .with_detail(ErrorDetail::FontMetricCapacityOverflow)
}
fn arithmetic(tag: [u8; 4]) -> Error {
    Error::new(
        ErrorCode::InvalidFontMetrics,
        "font metric variation arithmetic refused",
    )
    .with_detail(ErrorDetail::FontMetricVariationArithmetic { tag })
}
fn refuse(reason: TableError) -> Error {
    table_error(Table::Mvar, reason)
}

#[cfg(test)]
mod tests;
