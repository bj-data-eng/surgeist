//! Separate provider-owned source and selected-presentation joining actions.

#[cfg(test)]
#[path = "original_joining_tests.rs"]
mod tests;

use alloc::{sync::Arc, vec, vec::Vec};
use core::{mem::size_of, num::NonZeroU32};

use super::buffer::{hb_buffer_t, GlyphInfo, GlyphPosition};
use super::ot_shaper_arabic::{self, hb_arabic_joining_type_t as Class};
use crate::{Direction, Script, ShapingRefusal, UnicodeBuffer};

/// A failure to analyze a complete source or selected-presentation domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoiningSourceError {
    /// The scalar-indexed ordinary-symbol metadata is invalid.
    OrdinarySymbol(crate::OrdinarySymbolInputError),
    /// The operation currently admits horizontal directions only.
    UnsupportedDirection,
    /// The provider's logical resource budget prevented complete analysis.
    Refused(ShapingRefusal),
}

/// Invalid correspondence between an emitted buffer and either joining domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoiningInputError {
    /// The actual script or direction differs from the analyzed source profile.
    Profile,
    /// There must be one correspondence entry per emitted Unicode scalar.
    Length,
    /// An index is outside its retained domain.
    SourceIndex,
    /// Indices in each domain map must be strictly increasing and unrepeated.
    SourceOrder,
    /// A mapped contribution must retain its domain's actual code point.
    SourceCharacter,
    /// A mapped scalar has different semantic presentation from the body.
    SourcePresentation,
    /// Checked logical allocation accounting rejected the request.
    GrowthLimit,
}

#[derive(Debug)]
pub(crate) struct SourceScalar {
    pub(crate) codepoint: u32,
    ordinary_symbol: bool,
    class: Class,
    action: u8,
}

#[derive(Debug)]
struct SourceData {
    script: Script,
    direction: Direction,
    scalars: Vec<SourceScalar>,
}

/// Immutable, font-independent joining analysis of one complete Unicode domain.
///
/// The caller supplies either genuine unbroken processed source or actual
/// selected-line presentation, bounded by real barriers. Font changes do not
/// bound these Unicode domains. This operation analyzes the whole body rather
/// than clipping it to external pre/post-context capacity. Scalar indices are local to
/// this value, never glyph clusters or authored UTF-16 offsets.
///
/// Actions are derived by the provider, including its script-specific selector
/// handling. Actual glyph normalization must subsequently prove compatible
/// joining-bearing correspondence; this is not all-Unicode normalization support.
#[derive(Clone, Debug)]
pub struct JoiningSource(Arc<SourceData>);

impl JoiningSource {
    /// Analyze one complete Unicode domain for an explicit shaping profile.
    /// The resulting allocation may be shared across fonts and selections.
    /// `ordinary_symbol_positions` contains strictly increasing scalar indices
    /// of genuine nonexempt Cc with the same closed presentation as the emitted
    /// buffer. Cluster values and UTF-16 offsets do not identify these scalars.
    ///
    /// # Errors
    /// Returns `UnsupportedDirection` for a nonhorizontal profile,
    /// `OrdinarySymbol` for invalid presentation input, or the provider's actual
    /// logical growth/work refusal before publishing a source.
    pub fn analyze(
        source: &str,
        script: Script,
        direction: Direction,
        ordinary_symbol_positions: &[u32],
    ) -> Result<Self, JoiningSourceError> {
        if !direction.is_horizontal() {
            return Err(JoiningSourceError::UnsupportedDirection);
        }
        let count = source.chars().count();
        let limit = count
            .checked_mul(hb_buffer_t::MAX_LEN_FACTOR)
            .map(|n| n.max(hb_buffer_t::MAX_LEN_MIN))
            .and_then(|n| n.checked_mul(row_bytes()));
        let overlap = count
            .checked_mul(row_bytes() + size_of::<SourceScalar>())
            .and_then(|n| n.checked_add(source_header_bytes()));
        if count > hb_buffer_t::MAX_LEN_DEFAULT
            || !matches!((limit, overlap), (Some(limit), Some(bytes)) if bytes <= limit)
        {
            return Err(JoiningSourceError::Refused(ShapingRefusal::GrowthLimit));
        }
        // Admission precedes the temporary Unicode allocation as well as its
        // counted work; overflow must not buy an unbounded analysis prefix.
        let operations = i32::try_from(count)
            .ok()
            .and_then(|n| n.checked_mul(hb_buffer_t::MAX_OPS_FACTOR))
            .map_or(hb_buffer_t::MAX_OPS_DEFAULT, |n| {
                n.max(hb_buffer_t::MAX_OPS_MIN)
            });
        if count
            .checked_mul(8)
            .and_then(|n| i32::try_from(n).ok())
            .is_none_or(|n| n > operations)
        {
            return Err(JoiningSourceError::Refused(ShapingRefusal::WorkLimit));
        }
        let mut input = UnicodeBuffer::new();
        if !input.reserve(count) {
            return Err(JoiningSourceError::Refused(ShapingRefusal::GrowthLimit));
        }
        for (i, c) in source.chars().enumerate() {
            input.add(
                c,
                u32::try_from(i)
                    .map_err(|_| JoiningSourceError::Refused(ShapingRefusal::GrowthLimit))?,
            );
        }
        input
            .set_ordinary_symbol_positions(ordinary_symbol_positions)
            .map_err(JoiningSourceError::OrdinarySymbol)?;
        input.set_script(script);
        input.set_direction(direction);
        let mut buffer = input.0;
        buffer.enter();
        if !work(&mut buffer, count.saturating_mul(8)) {
            return Err(JoiningSourceError::Refused(ShapingRefusal::WorkLimit));
        }
        for (i, info) in buffer.info[..buffer.len].iter_mut().enumerate() {
            info.set_joining_lineage(
                u32::try_from(i + 1)
                    .map_err(|_| JoiningSourceError::Refused(ShapingRefusal::GrowthLimit))?,
            );
        }
        super::ot_shape::prepare_joining_source(&mut buffer);
        ot_shaper_arabic::source_joining_actions(script, &mut buffer);
        if let crate::ShapingStatus::Refused(reason) = buffer.shaping_status() {
            return Err(JoiningSourceError::Refused(reason));
        }
        let mut scalars = Vec::with_capacity(count);
        let mut symbols = ordinary_symbol_positions.iter().copied().peekable();
        for (index, c) in source.chars().enumerate() {
            let ordinary_symbol = symbols.peek() == Some(&(index as u32));
            if ordinary_symbol {
                symbols.next();
            }
            scalars.push(SourceScalar {
                codepoint: c as u32,
                ordinary_symbol,
                class: if ordinary_symbol {
                    Class::U
                } else {
                    class(c as u32)
                },
                action: ot_shaper_arabic::no_joining_action(),
            });
        }
        let actual_overlap = storage_bytes(&buffer, buffer.len).and_then(|bytes| {
            scalars
                .capacity()
                .checked_mul(size_of::<SourceScalar>())
                .and_then(|n| n.checked_add(bytes))
                .and_then(|n| n.checked_add(source_header_bytes()))
        });
        if !matches!((limit, actual_overlap), (Some(limit), Some(bytes)) if bytes <= limit) {
            return Err(JoiningSourceError::Refused(ShapingRefusal::GrowthLimit));
        }
        for info in &buffer.info[..buffer.len] {
            let index = info.joining_lineage() as usize - 1;
            scalars[index].action = info.arabic_shaping_action();
        }
        Ok(Self(Arc::new(SourceData {
            script,
            direction,
            scalars,
        })))
    }
}

fn row_bytes() -> usize {
    size_of::<GlyphInfo>() + size_of::<GlyphPosition>()
}

fn storage_bytes(buffer: &hb_buffer_t, length: usize) -> Option<usize> {
    buffer
        .info
        .capacity()
        .max(length)
        .checked_mul(size_of::<GlyphInfo>())
        .and_then(|n| {
            buffer
                .pos
                .capacity()
                .max(length)
                .checked_mul(size_of::<GlyphPosition>())
                .and_then(|m| n.checked_add(m))
        })
}

fn class(cp: u32) -> Class {
    ot_shaper_arabic::joining_class(cp)
}

fn special_selector(cp: u32) -> bool {
    matches!(cp, 0x180b..=0x180d | 0x180f)
}

#[derive(Clone, Copy, Debug)]
struct InputMapping {
    original: Option<NonZeroU32>,
    presentation: NonZeroU32,
}

#[derive(Clone, Debug)]
pub(crate) struct JoiningRequest {
    original: JoiningSource,
    presentation: JoiningSource,
    map: Vec<InputMapping>,
}

impl JoiningRequest {
    pub(crate) fn new(
        buffer: &hb_buffer_t,
        original: &JoiningSource,
        original_map: &[Option<usize>],
        presentation: &JoiningSource,
        presentation_map: &[usize],
    ) -> Result<Self, JoiningInputError> {
        if original_map.len() != buffer.len || presentation_map.len() != buffer.len {
            return Err(JoiningInputError::Length);
        }
        if [original, presentation].iter().any(|source| {
            source.0.script != buffer.script.unwrap_or(crate::script::UNKNOWN)
                || source.0.direction != buffer.direction
        }) {
            return Err(JoiningInputError::Profile);
        }
        let retained = retained_sources(&[
            Some(original),
            Some(presentation),
            buffer.joining.request.as_ref().map(|r| &r.original),
            buffer.joining.request.as_ref().map(|r| &r.presentation),
        ])
        .ok_or(JoiningInputError::GrowthLimit)?;
        let limit = buffer
            .len
            .checked_mul(hb_buffer_t::MAX_LEN_FACTOR)
            .map(|n| n.max(hb_buffer_t::MAX_LEN_MIN))
            .and_then(|n| n.checked_mul(row_bytes()));
        let bytes = buffer
            .len
            .checked_mul(
                size_of::<InputMapping>() + size_of::<Option<usize>>() + size_of::<usize>(),
            )
            .and_then(|n| storage_bytes(buffer, buffer.len).and_then(|rows| n.checked_add(rows)))
            .and_then(|n| n.checked_add(retained));
        let bytes = bytes.and_then(|n| {
            buffer.joining.request.as_ref().map_or(Some(n), |request| {
                request
                    .map
                    .capacity()
                    .checked_mul(size_of::<InputMapping>())
                    .and_then(|old| n.checked_add(old))
            })
        });
        if !matches!((limit, bytes), (Some(limit), Some(bytes)) if bytes <= limit) {
            return Err(JoiningInputError::GrowthLimit);
        }
        let mut previous_original = None;
        let mut previous_presentation = None;
        // Validate all input before allocating or replacing the old request.
        for (i, (&original_index, &presentation_index)) in
            original_map.iter().zip(presentation_map).enumerate()
        {
            validate_index(
                presentation,
                presentation_index,
                buffer.info[i].glyph_id,
                buffer.info[i].is_ordinary_symbol(),
                &mut previous_presentation,
            )?;
            if let Some(index) = original_index {
                validate_index(
                    original,
                    index,
                    buffer.info[i].glyph_id,
                    buffer.info[i].is_ordinary_symbol(),
                    &mut previous_original,
                )?;
            }
        }
        let mut map = Vec::with_capacity(buffer.len);
        for (&original, &presentation) in original_map.iter().zip(presentation_map) {
            map.push(InputMapping {
                original: original.and_then(|index| NonZeroU32::new((index + 1) as u32)),
                presentation: NonZeroU32::new((presentation + 1) as u32)
                    .expect("validated scalar index"),
            });
        }
        Ok(Self {
            original: original.clone(),
            presentation: presentation.clone(),
            map,
        })
    }

    fn validate(&self, buffer: &hb_buffer_t) -> Result<(), JoiningInputError> {
        if [self.original.0.as_ref(), self.presentation.0.as_ref()]
            .iter()
            .any(|source| {
                source.script != buffer.script.unwrap_or(crate::script::UNKNOWN)
                    || source.direction != buffer.direction
            })
        {
            return Err(JoiningInputError::Profile);
        }
        if self.map.len() != buffer.len {
            return Err(JoiningInputError::Length);
        }
        for (i, entry) in self.map.iter().enumerate() {
            let cp = buffer.info[i].glyph_id;
            if self.presentation.0.scalars[entry.presentation.get() as usize - 1].codepoint != cp
                || entry.original.is_some_and(|index| {
                    self.original.0.scalars[index.get() as usize - 1].codepoint != cp
                })
            {
                return Err(JoiningInputError::SourceCharacter);
            }
            let symbol = buffer.info[i].is_ordinary_symbol();
            if self.presentation.0.scalars[entry.presentation.get() as usize - 1].ordinary_symbol
                != symbol
                || entry.original.is_some_and(|index| {
                    self.original.0.scalars[index.get() as usize - 1].ordinary_symbol != symbol
                })
            {
                return Err(JoiningInputError::SourcePresentation);
            }
        }
        Ok(())
    }

    fn scalar(&self, entry: InputMapping) -> &SourceScalar {
        entry.original.map_or_else(
            || &self.presentation.0.scalars[entry.presentation.get() as usize - 1],
            |index| &self.original.0.scalars[index.get() as usize - 1],
        )
    }
}

fn validate_index(
    source: &JoiningSource,
    index: usize,
    cp: u32,
    ordinary_symbol: bool,
    previous: &mut Option<usize>,
) -> Result<(), JoiningInputError> {
    let scalar = source
        .0
        .scalars
        .get(index)
        .ok_or(JoiningInputError::SourceIndex)?;
    if u32::try_from(index + 1).is_err() {
        return Err(JoiningInputError::SourceIndex);
    }
    if previous.is_some_and(|old| index <= old) {
        return Err(JoiningInputError::SourceOrder);
    }
    if scalar.codepoint != cp {
        return Err(JoiningInputError::SourceCharacter);
    }
    if scalar.ordinary_symbol != ordinary_symbol {
        return Err(JoiningInputError::SourcePresentation);
    }
    *previous = Some(index);
    Ok(())
}

fn retained_bytes(original: &JoiningSource, presentation: &JoiningSource) -> Option<usize> {
    retained_sources(&[Some(original), Some(presentation)])
}

fn source_header_bytes() -> usize {
    size_of::<SourceData>() + size_of::<usize>() * 2
}

fn retained_sources(sources: &[Option<&JoiningSource>]) -> Option<usize> {
    let mut bytes = 0usize;
    for (i, source) in sources.iter().enumerate() {
        let Some(source) = source else { continue };
        if sources[..i]
            .iter()
            .flatten()
            .any(|old| Arc::ptr_eq(&source.0, &old.0))
        {
            continue;
        }
        bytes = source
            .0
            .scalars
            .capacity()
            .checked_mul(size_of::<SourceScalar>())
            .and_then(|n| n.checked_add(source_header_bytes()))
            .and_then(|n| bytes.checked_add(n))?;
    }
    Some(bytes)
}

#[derive(Clone, Copy, Debug)]
enum Lineage {
    Input(usize),
    Synthetic(u32),
    Union(u32, u32),
}

#[derive(Default)]
pub(crate) struct JoiningState {
    pub(crate) request: Option<JoiningRequest>,
    nodes: Vec<Lineage>,
    bytes: usize,
    tracking: bool,
    allocation_limit: usize,
}

impl JoiningState {
    pub(crate) fn bound(request: JoiningRequest) -> Result<Self, JoiningInputError> {
        let bytes = request
            .map
            .capacity()
            .checked_mul(size_of::<InputMapping>())
            .and_then(|n| {
                retained_bytes(&request.original, &request.presentation)
                    .and_then(|r| n.checked_add(r))
            })
            .ok_or(JoiningInputError::GrowthLimit)?;
        let allocation_limit = request
            .map
            .len()
            .checked_mul(hb_buffer_t::MAX_LEN_FACTOR)
            .map(|n| n.max(hb_buffer_t::MAX_LEN_MIN))
            .ok_or(JoiningInputError::GrowthLimit)?;
        Ok(Self {
            request: Some(request),
            bytes,
            allocation_limit,
            ..Self::default()
        })
    }
}

pub(crate) fn work(buffer: &mut hb_buffer_t, count: usize) -> bool {
    let Some(remaining) = i32::try_from(count)
        .ok()
        .and_then(|n| buffer.max_ops.checked_sub(n))
    else {
        buffer.refuse(ShapingRefusal::WorkLimit);
        buffer.successful = false;
        return false;
    };
    if remaining < 0 {
        buffer.refuse(ShapingRefusal::WorkLimit);
        buffer.successful = false;
        return false;
    }
    buffer.max_ops = remaining;
    true
}

pub(crate) fn room(buffer: &mut hb_buffer_t, length: usize, extra: usize) -> bool {
    if buffer.joining.request.is_none() {
        return true;
    }
    let limit = buffer
        .max_len
        .min(buffer.joining.allocation_limit)
        .checked_mul(row_bytes());
    let bytes = storage_bytes(buffer, length)
        .and_then(|n| n.checked_add(buffer.joining.bytes))
        .and_then(|n| n.checked_add(extra));
    if !matches!((limit, bytes), (Some(limit), Some(bytes)) if bytes <= limit) {
        buffer.refuse(ShapingRefusal::GrowthLimit);
        buffer.successful = false;
        return false;
    }
    true
}

fn charge(buffer: &mut hb_buffer_t, bytes: usize) -> bool {
    if !room(buffer, buffer.len, bytes) {
        return false;
    }
    buffer.joining.bytes += bytes;
    true
}

pub(crate) fn begin(buffer: &mut hb_buffer_t) {
    let Some(request) = buffer.joining.request.as_ref() else {
        return;
    };
    if request.validate(buffer).is_err() {
        buffer.refuse(ShapingRefusal::JoiningInput);
        buffer.successful = false;
        return;
    }
    if buffer
        .flags
        .contains(crate::BufferFlags::PRODUCE_SAFE_TO_INSERT_TATWEEL)
    {
        buffer.refuse(ShapingRefusal::JoiningProfile);
        buffer.successful = false;
        return;
    }
    let bytes = buffer.len.checked_mul(size_of::<Lineage>());
    let Some(bytes) = bytes.filter(|_| u32::try_from(buffer.len).is_ok()) else {
        buffer.refuse(ShapingRefusal::GrowthLimit);
        buffer.successful = false;
        return;
    };
    if !charge(buffer, bytes) || !work(buffer, buffer.len.saturating_mul(4)) {
        return;
    }
    buffer.joining.nodes.reserve_exact(buffer.len);
    for i in 0..buffer.len {
        buffer.joining.nodes.push(Lineage::Input(i));
        buffer.info[i].set_joining_lineage((i + 1) as u32);
    }
    buffer.joining.tracking = true;
}

fn add_node(buffer: &mut hb_buffer_t, node: Lineage) -> u32 {
    if !buffer.joining.tracking {
        return 0;
    }
    if !work(buffer, 1) {
        return 0;
    }
    let Ok(id) = u32::try_from(buffer.joining.nodes.len() + 1) else {
        buffer.refuse(ShapingRefusal::GrowthLimit);
        buffer.successful = false;
        return 0;
    };
    if id > 0x7fff_ffff {
        buffer.refuse(ShapingRefusal::GrowthLimit);
        buffer.successful = false;
        return 0;
    }
    if buffer.joining.nodes.len() == buffer.joining.nodes.capacity() {
        let old = buffer.joining.nodes.capacity();
        let target = old.max(2).checked_mul(2);
        // During relocation both old and new capacities can be live. The old
        // capacity is already charged; admit the full new allocation first.
        let bytes = target.and_then(|target| target.checked_mul(size_of::<Lineage>()));
        let Some((target, bytes)) = target.zip(bytes) else {
            buffer.refuse(ShapingRefusal::GrowthLimit);
            return 0;
        };
        if !charge(buffer, bytes) || !work(buffer, buffer.joining.nodes.len()) {
            return 0;
        }
        buffer
            .joining
            .nodes
            .reserve_exact(target - buffer.joining.nodes.len());
        buffer.joining.bytes -= old * size_of::<Lineage>();
    }
    buffer.joining.nodes.push(node);
    id
}

pub(crate) fn synthetic(buffer: &mut hb_buffer_t, cp: u32) -> u32 {
    add_node(buffer, Lineage::Synthetic(cp))
}

pub(crate) fn union(buffer: &mut hb_buffer_t, a: u32, b: u32) -> u32 {
    if !buffer.joining.tracking || a == b {
        return a;
    }
    if a == 0 || b == 0 {
        buffer.refuse(ShapingRefusal::JoiningCorrespondence);
        buffer.successful = false;
        return 0;
    }
    add_node(buffer, Lineage::Union(a, b))
}

pub(crate) fn apply(buffer: &mut hb_buffer_t) {
    resolve(buffer, true);
}

fn resolve(buffer: &mut hb_buffer_t, write_actions: bool) {
    if !buffer.joining.tracking || !buffer.successful {
        return;
    }
    let nodes = buffer.joining.nodes.len();
    let Some(bytes) = nodes.checked_mul(size_of::<usize>() * 3).and_then(|n| {
        buffer
            .len
            .checked_mul(size_of::<u8>())
            .and_then(|m| n.checked_add(m))
    }) else {
        buffer.refuse(ShapingRefusal::GrowthLimit);
        return;
    };
    if !charge(buffer, bytes)
        || !work(
            buffer,
            nodes
                .saturating_mul(3)
                .saturating_add(buffer.len.saturating_mul(3)),
        )
    {
        return;
    }
    let mut counts = vec![0usize; nodes];
    let mut seen = vec![0usize; nodes];
    let mut stack = Vec::with_capacity(nodes);
    let mut actions = Vec::with_capacity(buffer.len);
    for i in 0..buffer.len {
        let cp = buffer.info[i].as_codepoint();
        let actual_class = if buffer.info[i].is_ordinary_symbol() {
            Class::U
        } else {
            class(cp)
        };
        let mut desired = None;
        let mut original_bearing = false;
        let mut computed_bearing = false;
        let mut synthetic_bearing = false;
        stack.push(buffer.info[i].joining_lineage());
        while let Some(id) = stack.pop() {
            if !work(buffer, 1) {
                return;
            }
            let Some(index) = id
                .checked_sub(1)
                .map(|id| id as usize)
                .filter(|id| *id < nodes)
            else {
                buffer.refuse(ShapingRefusal::JoiningCorrespondence);
                buffer.successful = false;
                return;
            };
            if seen[index] == i + 1 {
                continue;
            }
            seen[index] = i + 1;
            match buffer.joining.nodes[index] {
                Lineage::Union(a, b) => {
                    if !work(buffer, 2) {
                        return;
                    }
                    if stack.len() + 2 > nodes {
                        buffer.refuse(ShapingRefusal::GrowthLimit);
                        return;
                    }
                    stack.extend([a, b]);
                }
                Lineage::Synthetic(codepoint) => {
                    if class(codepoint) != Class::U || actual_class != Class::U {
                        buffer.refuse(ShapingRefusal::JoiningCorrespondence);
                        buffer.successful = false;
                        return;
                    }
                    synthetic_bearing = true;
                }
                Lineage::Input(input) => {
                    let request = buffer.joining.request.as_ref().unwrap();
                    let entry = request.map[input];
                    let scalar = request.scalar(entry);
                    if scalar.ordinary_symbol != buffer.info[i].is_ordinary_symbol() {
                        buffer.refuse(ShapingRefusal::JoiningCorrespondence);
                        buffer.successful = false;
                        return;
                    }
                    if special_selector(scalar.codepoint) {
                        if cp != scalar.codepoint {
                            buffer.refuse(ShapingRefusal::JoiningCorrespondence);
                            buffer.successful = false;
                            return;
                        }
                    } else if actual_class == Class::T || scalar.class == Class::T {
                        continue;
                    }
                    let value = (scalar.class, scalar.action);
                    if scalar.class != actual_class || desired.is_some_and(|old| old != value) {
                        buffer.refuse(ShapingRefusal::JoiningCorrespondence);
                        buffer.successful = false;
                        return;
                    }
                    desired = Some(value);
                    original_bearing |= entry.original.is_some();
                    computed_bearing |= entry.original.is_none();
                    counts[index] += 1;
                }
            }
        }
        if original_bearing && computed_bearing {
            buffer.refuse(ShapingRefusal::JoiningCorrespondence);
            buffer.successful = false;
            return;
        }
        if actual_class != Class::T && desired.is_none() && !synthetic_bearing {
            buffer.refuse(ShapingRefusal::JoiningCorrespondence);
            buffer.successful = false;
            return;
        }
        actions.push(desired.map_or(ot_shaper_arabic::no_joining_action(), |(_, action)| action));
    }
    for (index, node) in buffer.joining.nodes.iter().enumerate() {
        if let Lineage::Input(input) = *node {
            let request = buffer.joining.request.as_ref().unwrap();
            let scalar = request.scalar(request.map[input]);
            if (scalar.class != Class::T || special_selector(scalar.codepoint))
                && counts[index] != 1
            {
                buffer.refuse(ShapingRefusal::JoiningCorrespondence);
                buffer.successful = false;
                return;
            }
        }
    }
    if write_actions {
        for (info, action) in buffer.info[..buffer.len].iter_mut().zip(actions) {
            info.set_arabic_shaping_action(action);
        }
    }
    // No source-domain positive safety is projected onto different adjacency.
    // Whole-buffer unsafe flags conservatively retain both dependencies.
    buffer.joining_dependencies();
    buffer.joining.bytes -= bytes;
    buffer.joining.tracking = false;
}

pub(crate) fn finish_masks(buffer: &mut hb_buffer_t) {
    if !buffer.joining.tracking || !buffer.successful {
        return;
    }
    let required = buffer.joining.request.as_ref().is_some_and(|request| {
        request
            .map
            .iter()
            .any(|entry| request.scalar(*entry).action != ot_shaper_arabic::no_joining_action())
    });
    if required {
        buffer.refuse(ShapingRefusal::JoiningProfile);
        buffer.successful = false;
        buffer.joining.tracking = false;
    } else {
        resolve(buffer, false);
    }
}
