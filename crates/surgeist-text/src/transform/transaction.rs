//! One request-local source transformation transaction. Facts are immutable;
//! stage effects and component mappings become ready only on genuine demand.

use super::{
    casing::{self, Input, Locale, Mapper, MappingError, Operation},
    context::{self, SourceContext, StyleRow},
    mapping::{self, ComponentInput, Mapped, MappingPlan, Mappings, PrefixEffect},
    relation::{Component, ComponentOwner, TransformMap},
    separators::{self, Adjacency, automatic::PhraseReads},
};
use crate::{
    CaseTransform, Range, Result, TextTransform,
    engine::{
        OriginalAtom,
        analysis::wrapping::{Event, LogicalEvent},
        resolve::AnalyzedStyleOwner,
    },
    projection::SourceProjection,
    protocol::paragraph::SemanticAnchorItem,
};
use icu_properties::{
    CodePointMapData, CodePointSetData,
    props::{ExtendedPictographic, GraphemeClusterBreak, IndicConjunctBreak},
};
use std::{cell::RefCell, rc::Rc};

enum Stage<T> {
    Unread,
    Ready(T),
}
fn unread<T>(length: usize) -> Result<Vec<Stage<T>>> {
    context::collect_rows((0..length).map(|_| Stage::Unread))
}

#[derive(Clone, Copy)]
struct Value {
    owner: ComponentOwner,
    source: Range,
    processed: Range,
    code_point: u32,
    separator_space: bool,
    style: usize,
}

#[derive(Clone, Copy)]
enum Purpose {
    PreCase,
    Mapped,
    SelectionIdentity,
}

/// Canonical finite-prefix effects translated to actual pre-case recipients.
#[derive(Clone, Copy)]
struct PrefixReceipt {
    preserved: [Option<usize>; 2],
    removed: Option<usize>,
    inserted: Option<usize>,
}

pub(super) struct Completed<'a> {
    pub(super) input: SourceProjection,
    pub(super) styles: Vec<StyleRow<'a>>,
    pub(super) map: TransformMap,
    pub(super) captured: Vec<LogicalEvent>,
    pub(super) mappings: Mappings,
    pub(super) selection_scopes: Option<Vec<bool>>,
}

struct TransformWork<'borrow, 'source> {
    context: &'borrow SourceContext<'source>,
    original: SourceProjection,
    styles: Vec<StyleRow<'source>>,
    map: TransformMap,
    events: Vec<LogicalEvent>,
    adjacency: Vec<Adjacency>,
    phrase_reads: PhraseReads,
    pre_case: Vec<Stage<Option<Value>>>,
    mapped: Vec<Stage<Mapped>>,
    /// Ready word-fact staging cohort. Output authority remains owns_event;
    /// demanded selection dependencies do not become requested source roots.
    cohort: Vec<bool>,
    word_starts: Vec<Stage<mapping::WordStarts>>,
    initials: Vec<Stage<Option<Operation>>>,
    prefixes: Vec<Stage<PrefixReceipt>>,
    hard_gaps: Vec<bool>,
    missing: Vec<(usize, usize, super::context::MissingEdge)>,
    mapper: Option<Rc<Mapper>>,
}

impl<'borrow, 'source> TransformWork<'borrow, 'source> {
    fn new(context: &'borrow SourceContext<'source>, original: SourceProjection) -> Result<Self> {
        let styles = context.styles()?;
        let events = context.separator_events(&original, &styles)?;
        let map = TransformMap::originals(&original, &styles, context)?;
        let seams = separators::connection_seams(&original, &events, context)?;
        // This fold observes original mixed roles only, never separator effects.
        let adjacency = separators::forced_adjacency(
            &original,
            &events,
            context.processed.source(),
            context,
            &seams,
        )?;
        let length = events
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(context::capacity)?;
        let mut hard_gaps = context::filled(events.len() + 1, false)?;
        for seam in &seams {
            // Captured connections retain their exact event cut. A flat
            // declaration has no retained Text inside its interval; the shared
            // separator readers refuse material generation without its missing
            // event-side evidence. Its no-effect scalar cut remains unique.
            hard_gaps[seam.upper] = true;
        }
        for fence in context.fences()? {
            if let context::SourceFence::BlockEvent(order) = fence {
                hard_gaps[context.block_event_cut(order, &original, &events)] = true;
            }
        }
        let mut missing = Vec::new();
        if let Some(topology) = context.scope_topology {
            for &edge in &context.missing {
                let incidence = |input: usize, last: bool| {
                    let order = topology.capture.origin_orders[input];
                    let key = crate::protocol::paragraph::FlowOriginKey::CapturedEvent(order);
                    let origin = context.origin_index(key).expect("actual edge original");
                    let range = context.origins[origin].aggregate;
                    let mut matching = events.iter().enumerate().filter_map(|(index, event)| {
                        let incident = match event.kind {
                            Event::Text(row) => map.original_pieces(row).iter().any(|piece| piece.origin == key),
                            Event::Deleted(source, ..) => source.start < range.end && range.start < source.end,
                            Event::Anchor(anchor) => matches!(anchor.item, SemanticAnchorItem::EmptyOrigin(index) if index == origin),
                        };
                        incident.then_some(index * 2 + 1)
                    });
                    if last {
                        matching.next_back()
                    } else {
                        matching.next()
                    }
                };
                if let (Some(before), Some(after)) = (
                    incidence(edge.before_origin, true),
                    incidence(edge.after_origin, false),
                ) {
                    context::push(&mut missing, (before + 1, after, edge))?;
                }
            }
        }
        let phrase_reads = PhraseReads::new(&styles, &original, context, &map, &events, &seams)?;
        Ok(Self {
            context,
            original,
            styles,
            map,
            events,
            adjacency,
            phrase_reads,
            pre_case: unread(length)?,
            mapped: unread(length)?,
            cohort: context::filled(length, false)?,
            word_starts: unread(length)?,
            initials: unread(length)?,
            prefixes: unread(length)?,
            hard_gaps,
            missing,
            mapper: None,
        })
    }

    fn hard_event(&self, index: usize) -> bool {
        match self.events[index].kind {
            Event::Text(row) => {
                crate::engine::analysis::is_hard_separator(self.original.atoms[row].code_point)
            }
            Event::Deleted(..) => false,
            Event::Anchor(anchor) => self.context.hard_anchor(anchor.item),
        }
    }

    fn stopped(&self, slot: usize) -> bool {
        if slot.is_multiple_of(2) {
            self.hard_gaps[slot / 2]
        } else {
            self.hard_event(slot / 2)
        }
    }

    fn crossing(&self, cut: usize, reverse: bool) -> std::result::Result<bool, MappingError> {
        let target = if reverse {
            cut.checked_sub(1)
        } else {
            Some(cut)
        };
        if target.is_none_or(|target| target >= self.pre_case.len() || self.stopped(target)) {
            return Ok(true);
        }
        if let Some((_, _, edge)) = self
            .missing
            .iter()
            .find(|(first, last, _)| *first <= cut && cut <= *last)
        {
            return Err(MappingError::Context(*edge));
        }
        Ok(false)
    }

    fn source_error(&self, error: crate::Error) -> MappingError {
        if let Some(crate::ErrorDetail::SourceProcessing {
            reason:
                crate::SourceProcessingError::MissingSourceContinuity {
                    before_origin,
                    after_origin,
                    ..
                },
        }) = error.detail()
            && let Some(edge) = self.context.missing.iter().find(|edge| {
                edge.before_origin == *before_origin && edge.after_origin == *after_origin
            })
        {
            return MappingError::Context(*edge);
        }
        MappingError::Source(error)
    }

    fn read(&mut self, slot: usize) -> Result<Option<Value>> {
        if let Stage::Ready(value) = self.pre_case[slot] {
            return Ok(value);
        }
        let value = if slot.is_multiple_of(2) {
            let cut = slot / 2;
            if let Some(proposal) = separators::automatic::proposal_at(
                &self.original,
                &mut self.styles,
                self.context,
                &mut self.map,
                cut,
                &mut self.phrase_reads,
            )? {
                let data = &self.map.virtuals[proposal.occurrence];
                Some(Value {
                    owner: ComponentOwner::Virtual(proposal.occurrence),
                    source: Range::new(data.source, data.source),
                    processed: Range::new(data.processed, data.processed),
                    code_point: proposal.code_point,
                    separator_space: proposal.code_point == 0x20,
                    style: proposal.style,
                })
            } else {
                None
            }
        } else {
            let index = slot / 2;
            let event = self.events[index];
            match event.kind {
                Event::Text(row) => {
                    let effect = separators::original_value(
                        &self.original,
                        &self.map,
                        &self.styles,
                        self.context,
                        row,
                        self.adjacency[index],
                    )?;
                    let atom = &self.original.atoms[row];
                    Some(Value {
                        owner: ComponentOwner::Original(row),
                        source: atom.source,
                        processed: atom.processed,
                        code_point: effect.code_point,
                        separator_space: effect.separator_space,
                        style: self.map.rows[row].style,
                    })
                }
                Event::Anchor(anchor) => {
                    if let SemanticAnchorItem::Control(control) = anchor.item {
                        // Original replacements preserve every forced/nonforced mixed
                        // role. A qualified virtual proposal cannot intervene before
                        // a nearest forced role: its canonical suppression excludes
                        // that very interval. Thus this is the post-original role
                        // proof, without recursively reading the final pre-case stage.
                        if let Some(effect) = separators::wbr_value(
                            &self.original,
                            self.context,
                            control,
                            event.atom,
                            self.adjacency[index],
                        )? {
                            let at = self.context.controls[control].at;
                            let p = self.context.processed.project(at, crate::Affinity::After);
                            let style = self.styles.iter().position(|row| matches!(row.owner, AnalyzedStyleOwner::Control(own) if own == control))
                            .expect("actual generating-control style");
                            Some(Value {
                                owner: ComponentOwner::Wbr(control),
                                source: Range::new(at, at),
                                processed: Range::new(p, p),
                                code_point: effect.code_point,
                                separator_space: effect.separator_space,
                                style,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Event::Deleted(..) => None,
            }
        };
        self.pre_case[slot] = Stage::Ready(value);
        Ok(value)
    }

    fn ensure_mapper(&mut self) -> Result<Rc<Mapper>> {
        if let Some(mapper) = &self.mapper {
            return Ok(Rc::clone(mapper));
        }
        let mapper = Rc::new(Mapper::new().map_err(mapping::failed)?);
        self.mapper = Some(Rc::clone(&mapper));
        Ok(mapper)
    }

    fn owns_event(&self, index: usize) -> bool {
        if self.context.whole_domain {
            return true;
        }
        match self.events[index].kind {
            Event::Text(row) => self.map.original_pieces(row).iter().any(|piece| {
                self.context
                    .requested
                    .origin_map
                    .view_origin(piece.origin)
                    .is_some()
            }),
            Event::Deleted(source, ..) => self
                .context
                .requested
                .contains(self.context.processed.source(), source),
            Event::Anchor(anchor) => self
                .context
                .anchor_order
                .iter()
                .position(|own| *own == anchor)
                .and_then(|order| self.context.local_anchor(order))
                .is_some(),
        }
    }

    fn peek(
        &mut self,
        from: usize,
        reverse: bool,
    ) -> Result<Option<(usize, Value, Option<super::context::MissingEdge>)>> {
        let mut position = from;
        let mut missing = None;
        loop {
            let next = if reverse {
                position.checked_sub(1)
            } else {
                position
                    .checked_add(1)
                    .filter(|&next| next < self.pre_case.len())
            };
            let Some(next) = next else { return Ok(None) };
            if self.stopped(next) {
                return Ok(None);
            }
            let cut = if reverse { position } else { next };
            if let Some((_, _, edge)) = self
                .missing
                .iter()
                .find(|(first, last, _)| *first <= cut && cut <= *last)
            {
                missing = missing.or(Some(*edge));
            }
            position = next;
            if let Some(value) = self.read(next)? {
                return Ok(Some((next, value, missing)));
            }
        }
    }

    /// A proof over complete native outcome families, never an identity value.
    /// Optional separator cells may be absent or emit SPACE/ideographic SPACE;
    /// all their mapped outcomes have ordinary leading class and non-Prepend tail.
    fn family_boundary(
        &self,
        from: usize,
        reverse: bool,
        known: u32,
        purpose: Purpose,
    ) -> Result<bool> {
        let ordinary_known = ordinary(known);
        let non_prepend = gcb(known) != GraphemeClusterBreak::Prepend;
        if reverse && !ordinary_known || !reverse && !non_prepend {
            return Ok(false);
        }
        let mut position = from;
        loop {
            let next = if reverse {
                position.checked_sub(1)
            } else {
                position
                    .checked_add(1)
                    .filter(|&next| next < self.pre_case.len())
            };
            let Some(next) = next else { return Ok(true) };
            if self.stopped(next) {
                return Ok(true);
            }
            position = next;
            if next.is_multiple_of(2) {
                continue;
            }
            match self.events[next / 2].kind {
                Event::Text(row) => {
                    let cp = self.original.atoms[row].code_point;
                    if matches!(purpose, Purpose::PreCase) {
                        return Ok(if reverse {
                            gcb(cp) != GraphemeClusterBreak::Prepend
                        } else {
                            ordinary(cp) || cp == 0x200b
                        });
                    }
                    // Irish uppercasing may remove a prefix hyphen; every
                    // nonempty alternative remains ordinary-class punctuation.
                    // Continue past the nullable family rather than inventing
                    // either a hyphen emission or an absent component.
                    if matches!(cp, 0x2d | 0x2010 | 0x2011) {
                        continue;
                    }
                    if cp <= 0x7f && (cp as u8).is_ascii_alphanumeric()
                        || matches!(cp, 0x20 | 0x3000 | 0x200b)
                    {
                        return Ok(true);
                    }
                    let transform = self.styles[self.map.rows[row].style].style.text_transform();
                    if let TextTransform::Set(set) = transform
                        && cp == casing::CAPITAL_SIGMA
                        && set.case() == CaseTransform::Lowercase
                    {
                        let family = casing::lower_sigma_family().map_err(mapping::failed)?;
                        return Ok(family.into_iter().all(|mut outcome| {
                            if !matches!(purpose, Purpose::SelectionIdentity)
                                && set.full_width()
                                && let Some(mapped) = super::width::map(outcome)
                            {
                                outcome = mapped;
                            }
                            if !matches!(purpose, Purpose::SelectionIdentity)
                                && set.full_size_kana()
                                && let Some(mapped) = super::full_size_kana(outcome)
                            {
                                outcome = u32::from(mapped);
                            }
                            if reverse {
                                gcb(outcome) != GraphemeClusterBreak::Prepend
                            } else {
                                ordinary(outcome)
                            }
                        }));
                    }
                    if let TextTransform::Set(set) = transform
                        && set.case() == CaseTransform::None
                    {
                        let mut cp = cp;
                        if !matches!(purpose, Purpose::SelectionIdentity)
                            && set.full_width()
                            && let Some(mapped) = super::width::map(cp)
                        {
                            cp = mapped;
                        }
                        if !matches!(purpose, Purpose::SelectionIdentity)
                            && set.full_size_kana()
                            && let Some(mapped) = super::full_size_kana(cp)
                        {
                            cp = u32::from(mapped);
                        }
                        return Ok(if reverse {
                            gcb(cp) != GraphemeClusterBreak::Prepend
                        } else {
                            ordinary(cp)
                        });
                    }
                    return Ok(false);
                }
                Event::Anchor(_) | Event::Deleted(..) => (),
            }
        }
    }
}

fn gcb(cp: u32) -> GraphemeClusterBreak {
    CodePointMapData::<GraphemeClusterBreak>::new().get32(cp)
}
fn ordinary(cp: u32) -> bool {
    gcb(cp) == GraphemeClusterBreak::Other
        && !CodePointSetData::new::<ExtendedPictographic>().contains32(cp)
        && CodePointMapData::<IndicConjunctBreak>::new().get32(cp) != IndicConjunctBreak::Consonant
}

fn atom(value: Value, at: usize) -> Result<OriginalAtom> {
    let glyph = if (0xd800..=0xdfff).contains(&value.code_point) {
        '\u{fffd}'
    } else {
        char::from_u32(value.code_point).ok_or_else(context::capacity)?
    };
    let length = if (0xd800..=0xdfff).contains(&value.code_point) {
        1
    } else {
        glyph.len_utf16()
    };
    Ok(OriginalAtom {
        source: value.source,
        processed: value.processed,
        presentation: Range::new(at, at.checked_add(length).ok_or_else(context::capacity)?),
        code_point: value.code_point,
        glyph,
        glyph_bytes: 0..0,
    })
}

fn window(values: &[(usize, Value)]) -> Result<SourceProjection> {
    let mut atoms = Vec::new();
    let mut at = 0;
    for &(_, value) in values {
        let next = atom(value, at)?;
        at = next.presentation.end;
        context::push(&mut atoms, next)?;
    }
    SourceProjection::from_atoms_at_barriers(atoms, at, Vec::new(), Vec::new())
}

fn mapping_input<'a>(
    length: usize,
    value: &'a dyn Fn(usize) -> std::result::Result<Option<u32>, MappingError>,
    crossing: &'a dyn Fn(usize, bool) -> std::result::Result<bool, MappingError>,
) -> Input<'a> {
    Input::Source {
        length,
        value,
        crossing,
    }
}

fn prefix_window(
    shared: &RefCell<TransformWork<'_, '_>>,
    slot: usize,
) -> Result<Vec<(usize, Value)>> {
    let mut values = Vec::new();
    let mut position = slot;
    while let Some((next, value, missing)) = shared.borrow_mut().peek(position, true)? {
        // A genuine SPACE is a sufficient word reset; it is not a source fence.
        if matches!(value.code_point, 0x20 | 0x3000) {
            break;
        }
        if let Some(edge) = missing {
            return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
        }
        context::push(&mut values, (next, value))?;
        position = next;
    }
    values.reverse();
    Ok(values)
}

/// Publish facts for a genuinely requested ready stretch and its qualified
/// scalar prefix. Borrowed input is never subjected to its own casing here.
fn word_facts(
    shared: &RefCell<TransformWork<'_, '_>>,
    target: usize,
    need_initial: bool,
) -> Result<()> {
    let (first, last) = {
        let work = shared.borrow();
        let mut first = target;
        let mut last = target;
        while first > 0 && work.cohort[first - 1] && !work.stopped(first - 1) {
            first -= 1;
        }
        while last + 1 < work.pre_case.len() && work.cohort[last + 1] && !work.stopped(last + 1) {
            last += 1;
        }
        (first, last)
    };
    let mut body = Vec::new();
    let mut previous = None;
    for slot in first..=last {
        let value = { shared.borrow_mut().read(slot)? };
        if let Some(value) = value {
            // Neutral leading/trailing sites do not extend a word fact's
            // context. Interior traversal still includes every actual cut,
            // including missing relationships around empty occurrences.
            if let Some(before) = previous {
                for cut in before + 1..=slot {
                    if let Err(error) = shared.borrow().crossing(cut, false) {
                        return Err(mapping::failed(error));
                    }
                }
            }
            context::push(&mut body, (slot, value))?;
            previous = Some(slot);
        }
    }
    let first_value = body.first().expect("demanded word component").0;
    let mut values = prefix_window(shared, first_value)?;
    for value in body {
        context::push(&mut values, value)?;
    }
    let at = values
        .iter()
        .position(|(slot, _)| *slot == target)
        .expect("target input");
    let mut input = window(&values)?;
    let mut units = super::project::scalar_units(&input)?;
    let mut tail_closed = {
        let tail = values.last().expect("target input");
        shared
            .borrow()
            .family_boundary(tail.0, false, tail.1.code_point, Purpose::PreCase)?
    };
    // Only an actual initial-role demand in the open edge EGC completes that
    // EGC. Word-start-only/Irish reads do not force an unused unit or word tail.
    if need_initial && units.last().is_some_and(|unit| unit.contains(&at)) {
        while !tail_closed {
            let tail = *values.last().expect("target input");
            let next = { shared.borrow_mut().peek(tail.0, false)? };
            let Some((slot, value, missing)) = next else {
                tail_closed = true;
                break;
            };
            let guaranteed = !matches!(tail.1.code_point, 0xa | 0xd)
                && gcb(tail.1.code_point) != GraphemeClusterBreak::Control
                && extender(value.code_point);
            if guaranteed {
                if let Some(edge) = missing {
                    return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
                }
                context::push(&mut values, (slot, value))?;
            } else {
                let mut extended = context::collect_rows(values.iter().copied())?;
                context::push(&mut extended, (slot, value))?;
                let candidate = window(&extended)?;
                let candidate_units = super::project::scalar_units(&candidate)?;
                let joins = candidate_units
                    .iter()
                    .any(|unit| unit.contains(&at) && unit.contains(&(values.len())));
                if !joins {
                    tail_closed = true;
                    break;
                }
                if let Some(edge) = missing {
                    return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
                }
                values = extended;
            }
            // GB9/GB9a runs need no repeated segmentation. Every contributing
            // gap was actually staged by peek; possible state transitions return
            // to the canonical segmenter above before reading another value.
            let tail = values.last().expect("completed pre-case value");
            tail_closed = shared.borrow().family_boundary(
                tail.0,
                false,
                tail.1.code_point,
                Purpose::PreCase,
            )?;
        }
        input = window(&values)?;
        units = super::project::scalar_units(&input)?;
    }
    let mapper = shared.borrow_mut().ensure_mapper()?;
    let facts = mapping::word_facts(&input, &mapper)?;
    let open_from = if tail_closed {
        values.len()
    } else {
        units.last().map_or(0, |unit| unit.start)
    };
    let mut work = shared.borrow_mut();
    for (index, &(slot, _)) in values.iter().enumerate() {
        if matches!(work.word_starts[slot], Stage::Unread) {
            let starts = facts.starts[index];
            work.word_starts[slot] = Stage::Ready(mapping::WordStarts {
                ordinary: values[starts.ordinary].0,
                hyphen: values[starts.hyphen].0,
            });
        }
        if index < open_from && matches!(work.initials[slot], Stage::Unread) {
            work.initials[slot] = Stage::Ready(facts.initial[index]);
        }
    }
    Ok(())
}

fn initial_operation(
    shared: &RefCell<TransformWork<'_, '_>>,
    slot: usize,
) -> Result<Option<Operation>> {
    if let Stage::Ready(role) = shared.borrow().initials[slot] {
        return Ok(role);
    }
    if shared.borrow().stopped(slot) {
        return Ok(None);
    }
    word_facts(shared, slot, true)?;
    let work = shared.borrow();
    let Stage::Ready(role) = work.initials[slot] else {
        unreachable!("demanded initial unit was completed")
    };
    Ok(role)
}

fn irish_effect(
    shared: &RefCell<TransformWork<'_, '_>>,
    slot: usize,
    value: Value,
    operation: Operation,
) -> Result<PrefixEffect> {
    if !super::irish::recipient(value.code_point, operation) {
        return Ok(PrefixEffect::default());
    }
    if matches!(shared.borrow().word_starts[slot], Stage::Unread) {
        word_facts(shared, slot, false)?;
    }
    let start = {
        let work = shared.borrow();
        let Stage::Ready(starts) = work.word_starts[slot] else {
            unreachable!("qualified word start")
        };
        if matches!(value.code_point, 0x2d | 0x2010 | 0x2011) {
            starts.hyphen
        } else {
            starts.ordinary
        }
    };
    let cached = match shared.borrow().prefixes[start] {
        Stage::Ready(receipt) => Some(receipt),
        Stage::Unread => None,
    };
    let receipt = if let Some(receipt) = cached {
        receipt
    } else {
        let value = shared
            .borrow_mut()
            .read(start)?
            .expect("canonical word start");
        let mut prefix = context::collect_rows([(start, value)])?;
        let facts = super::irish::prefix_with(|index| {
            while prefix.len() <= index {
                let position = prefix.last().expect("actual word start").0;
                let result = { shared.borrow_mut().peek(position, false) };
                let next = result.map_err(|error| shared.borrow().source_error(error))?;
                let Some((next, value, missing)) = next else {
                    return Ok(None);
                };
                if let Some(edge) = missing {
                    return Err(MappingError::Context(edge));
                }
                context::push(&mut prefix, (next, value)).map_err(MappingError::Source)?;
            }
            Ok(Some(prefix[index].1.code_point))
        })
        .map_err(mapping::failed)?;
        let mut preserved = [None; 2];
        for (recipient, value) in preserved.iter_mut().zip(prefix.iter()).take(facts.preserve) {
            *recipient = Some(value.0);
        }
        let receipt = PrefixReceipt {
            preserved,
            removed: facts.remove.map(|index| prefix[index].0),
            inserted: facts.insert_before.map(|index| prefix[index].0),
        };
        // Only the completed canonical finite predicate fills this receipt.
        // A recipient-local exclusion above never proves other recipients inert.
        shared.borrow_mut().prefixes[start] = Stage::Ready(receipt);
        receipt
    };
    Ok(PrefixEffect {
        preserve: operation == Operation::Upper && receipt.preserved.contains(&Some(slot)),
        remove: operation == Operation::Upper && receipt.removed == Some(slot),
        insert: operation == Operation::Lower && receipt.inserted == Some(slot),
    })
}

fn map_slot(shared: &RefCell<TransformWork<'_, '_>>, slot: usize) -> Result<bool> {
    if matches!(shared.borrow().mapped[slot], Stage::Ready(_)) {
        return Ok(true);
    }
    let Some(value) = shared.borrow_mut().read(slot)? else {
        return Ok(false);
    };
    let style = shared.borrow().styles[value.style].style;
    let operation = match style.text_transform() {
        TextTransform::MathAuto => None,
        TextTransform::Set(set) => match set.case() {
            CaseTransform::None => None,
            CaseTransform::Uppercase => Some(Operation::Upper),
            CaseTransform::Lowercase => Some(Operation::Lower),
            CaseTransform::Capitalize => initial_operation(shared, slot)?,
        },
    };
    let locale = if operation.is_some() {
        Locale::from_tag(style.locale_tag()).map_err(mapping::failed)?
    } else {
        Locale::Root
    };
    let prefix = if locale == Locale::Irish
        && let Some(operation) = operation
    {
        irish_effect(shared, slot, value, operation)?
    } else {
        PrefixEffect::default()
    };
    let mapper = if operation.is_some() && !prefix.preserve && !prefix.remove {
        Some(shared.borrow_mut().ensure_mapper()?)
    } else {
        None
    };
    let (math_eligible, preserves_space, length) = {
        let work = shared.borrow();
        let math = if let ComponentOwner::Original(row) = value.owner {
            let pieces = work.map.original_pieces(row);
            !pieces.is_empty()
                && pieces.iter().all(|piece| {
                    work.context.origins[work
                        .context
                        .origin_index(piece.origin)
                        .expect("actual original")]
                    .text_node
                    .as_ref()
                    .is_some_and(|node| super::math::eligible(node.text()))
                })
        } else {
            false
        };
        (
            math,
            work.context
                .processed
                .preserves_space_at(value.processed.start),
            work.pre_case.len(),
        )
    };
    let read = |index| {
        let result = shared.borrow_mut().read(index);
        result
            .map(|value| value.map(|value| value.code_point))
            .map_err(|error| shared.borrow().source_error(error))
    };
    let crossing = |cut, reverse| shared.borrow().crossing(cut, reverse);
    let input = mapping_input(length, &read, &crossing);
    let mapped = mapping::component(
        &input,
        slot,
        ComponentInput {
            code_point: value.code_point,
            style,
            math_eligible,
            separator_space: value.separator_space,
            preserves_space,
        },
        MappingPlan {
            operation,
            locale,
            prefix,
        },
        mapper.as_deref(),
    )?;
    shared.borrow_mut().mapped[slot] = Stage::Ready(mapped);
    Ok(true)
}

struct SemanticWindow {
    projection: SourceProjection,
    owners: Vec<usize>,
    delayed: bool,
    pending_tail: bool,
}

#[derive(Clone, Copy)]
enum SemanticBasis {
    Presentation,
    SelectionIdentity,
}

fn semantic_window(
    shared: &RefCell<TransformWork<'_, '_>>,
    first: usize,
    last: usize,
    basis: SemanticBasis,
) -> Result<SemanticWindow> {
    let work = shared.borrow();
    let mut values = Vec::new();
    let mut pending = Vec::new();
    let mut cuts = Vec::new();
    let mut delayed = false;
    for slot in first..=last {
        if work.stopped(slot) {
            for value in pending.drain(..) {
                context::push(&mut values, value)?;
            }
            context::push(&mut cuts, values.len())?;
        }
        let Stage::Ready(mapped) = &work.mapped[slot] else {
            continue;
        };
        let Stage::Ready(Some(value)) = work.pre_case[slot] else {
            unreachable!("mapped component has completed input")
        };
        if icu_normalizer::properties::CanonicalCombiningClassMap::new().get32_u8(value.code_point)
            == 0
        {
            for value in pending.drain(..) {
                context::push(&mut values, value)?;
            }
        }
        let postpone = casing::postpones_iota(
            value.code_point,
            mapped.operation,
            mapped.identities.last().copied(),
        );
        delayed |= postpone;
        let emitted = match basis {
            SemanticBasis::Presentation => &mapped.output,
            SemanticBasis::SelectionIdentity => &mapped.identities,
        };
        let immediate = emitted.len().saturating_sub(usize::from(postpone));
        if immediate > 0 {
            for &(before, _) in &pending {
                if let Some((_, _, edge)) = work
                    .missing
                    .iter()
                    .find(|(start, end, _)| before < *end && *start <= slot)
                {
                    return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
                }
            }
        }
        for (index, &cp) in emitted.iter().enumerate() {
            let next = (
                slot,
                Value {
                    code_point: cp,
                    ..value
                },
            );
            if postpone && index + 1 == emitted.len() {
                context::push(&mut pending, next)?;
            } else {
                context::push(&mut values, next)?;
            }
        }
    }
    let pending_tail = !pending.is_empty();
    for value in pending {
        context::push(&mut values, value)?;
    }
    let owners = context::collect_rows(values.iter().map(|(slot, _)| *slot))?;
    let base = window(&values)?;
    let end = base.presentation_end();
    let projection = SourceProjection::from_atoms_at_barriers(base.atoms, end, Vec::new(), cuts)?;
    Ok(SemanticWindow {
        projection,
        owners,
        delayed,
        pending_tail,
    })
}

fn reset(cp: u32) -> bool {
    !matches!(
        gcb(cp),
        GraphemeClusterBreak::Extend
            | GraphemeClusterBreak::ZWJ
            | GraphemeClusterBreak::RegionalIndicator
    ) && !matches!(
        CodePointMapData::<IndicConjunctBreak>::new().get32(cp),
        IndicConjunctBreak::Extend | IndicConjunctBreak::Linker
    )
}

/// A contiguous interval of genuinely completed stage cells. Unread cells are
/// never omitted from its semantic stream.
struct CompletedInterval {
    first: usize,
    last: usize,
}

impl CompletedInterval {
    fn expand(&mut self, work: &TransformWork<'_, '_>) {
        let completed = |slot| match &work.pre_case[slot] {
            Stage::Ready(None) => true,
            Stage::Ready(Some(_)) => matches!(work.mapped[slot], Stage::Ready(_)),
            Stage::Unread => false,
        };
        while self.first > 0 && !work.stopped(self.first - 1) && completed(self.first - 1) {
            self.first -= 1;
        }
        while self.last + 1 < work.pre_case.len()
            && !work.stopped(self.last + 1)
            && completed(self.last + 1)
        {
            self.last += 1;
        }
    }

    fn include(&mut self, slot: usize, work: &TransformWork<'_, '_>) {
        self.first = self.first.min(slot);
        self.last = self.last.max(slot);
        self.expand(work);
    }
}

/// One canonical segmentation and its actual source-slot incidence, shared by
/// every requested component in this completed interval.
struct ClosureWindow {
    semantic: SemanticWindow,
    units: Vec<std::ops::Range<usize>>,
    requested: Vec<bool>,
    first_requested: usize,
    last_requested: usize,
}

impl ClosureWindow {
    fn new(
        shared: &RefCell<TransformWork<'_, '_>>,
        interval: &CompletedInterval,
        owned: &[usize],
    ) -> Result<Option<Self>> {
        let semantic = semantic_window(
            shared,
            interval.first,
            interval.last,
            SemanticBasis::Presentation,
        )?;
        let units = super::project::scalar_units(&semantic.projection)?;
        let requested = context::collect_rows(units.iter().map(|unit| {
            unit.clone()
                .any(|atom| owned.binary_search(&semantic.owners[atom]).is_ok())
        }))?;
        let Some(first_requested) = requested.iter().position(|requested| *requested) else {
            return Ok(None);
        };
        let last_requested = requested
            .iter()
            .rposition(|requested| *requested)
            .expect("first requested unit");
        Ok(Some(Self {
            semantic,
            units,
            requested,
            first_requested,
            last_requested,
        }))
    }

    fn crosses(&self, first: usize, last: usize) -> bool {
        self.units
            .iter()
            .zip(&self.requested)
            .any(|(unit, requested)| {
                if !requested {
                    return false;
                }
                let mut minimum = usize::MAX;
                let mut maximum = 0;
                for atom in unit.clone() {
                    minimum = minimum.min(self.semantic.owners[atom]);
                    maximum = maximum.max(self.semantic.owners[atom]);
                }
                minimum < maximum && minimum < last && first <= maximum
            })
    }

    /// Unknown lookbehind is discharged only by an actual sufficient reset
    /// after that gap, or strong completed boundaries at all its possible cuts.
    /// RI/GB11/GB9c state is never certified from a truncated prefix.
    fn left_independent(&self, first: usize, last: usize, reached_start: bool) -> bool {
        let own_start = self.units[self.first_requested].start;
        if (0..own_start).any(|atom| {
            self.semantic.owners[atom] >= last
                && reset(self.semantic.projection.atoms[atom].code_point)
        }) {
            return true;
        }
        let mut saw_cut = false;
        for atom in 1..self.semantic.owners.len() {
            let before = self.semantic.owners[atom - 1];
            let after = self.semantic.owners[atom];
            if before == after || before.min(after) >= last || first > before.max(after) {
                continue;
            }
            saw_cut = true;
            if !ordinary(self.semantic.projection.atoms[atom].code_point)
                || gcb(self.semantic.projection.atoms[atom - 1].code_point)
                    == GraphemeClusterBreak::Prepend
            {
                return false;
            }
        }
        saw_cut || reached_start
    }
}

fn remember(
    witnesses: &mut Vec<super::context::MissingEdge>,
    missing: Option<super::context::MissingEdge>,
) -> Result<()> {
    if let Some(edge) = missing
        && !witnesses.contains(&edge)
    {
        context::push(witnesses, edge)?;
    }
    Ok(())
}

fn extender(cp: u32) -> bool {
    matches!(
        gcb(cp),
        GraphemeClusterBreak::Extend
            | GraphemeClusterBreak::ZWJ
            | GraphemeClusterBreak::SpacingMark
    )
}

/// Grow only while actual mapped output proves that the pending unit/state
/// cannot yet be closed. This is a sufficient GB9/GB9a continuation proof;
/// canonical segmentation remains the sole completed-unit authority.
fn grow_boundary(
    shared: &RefCell<TransformWork<'_, '_>>,
    interval: &mut CompletedInterval,
    known: &SemanticWindow,
    reverse: bool,
    witnesses: &mut Vec<super::context::MissingEdge>,
) -> Result<bool> {
    let head = known
        .projection
        .atoms
        .first()
        .expect("requested output")
        .code_point;
    let tail = known
        .projection
        .atoms
        .last()
        .expect("requested output")
        .code_point;
    let order_clear = if reverse {
        !known.delayed
    } else {
        !known.pending_tail
    };
    // Prepending extenders is proved only when the existing leading scalar is
    // itself an extender. GB9 is not reversed for an arbitrary leading class.
    let permits_values = if reverse {
        extender(head)
    } else {
        !matches!(tail, 0xa | 0xd) && gcb(tail) != GraphemeClusterBreak::Control
    };
    loop {
        let from = if reverse {
            interval.first
        } else {
            interval.last
        };
        let next = { shared.borrow_mut().peek(from, reverse)? };
        let Some((next, _, missing)) = next else {
            return Ok(true);
        };
        let old_first = interval.first;
        let old_last = interval.last;
        map_slot(shared, next)?;
        remember(witnesses, missing)?;
        interval.include(next, &shared.borrow());
        if !order_clear || !witnesses.is_empty() {
            return Ok(false);
        }
        let added = if reverse {
            interval.first..old_first
        } else {
            old_last + 1..interval.last + 1
        };
        let work = shared.borrow();
        for slot in added {
            let Stage::Ready(mapped) = &work.mapped[slot] else {
                continue;
            };
            let Stage::Ready(Some(value)) = work.pre_case[slot] else {
                unreachable!("completed mapping input")
            };
            if casing::postpones_iota(
                value.code_point,
                mapped.operation,
                mapped.identities.last().copied(),
            ) {
                return Ok(false);
            }
            // Empty is an actual completed removal, not unread input. Otherwise
            // every new scalar must continue the still-open edge unit. A SPACE
            // or any possible state/boundary transition returns to ICU before
            // another component or its style fields are demanded.
            if !mapped.output.is_empty()
                && (!permits_values || !mapped.output.iter().copied().all(extender))
            {
                return Ok(false);
            }
        }
    }
}

fn close_units(shared: &RefCell<TransformWork<'_, '_>>, owned: &[usize]) -> Result<()> {
    let mut cursor = 0;
    while let Some(&seed) = owned.get(cursor) {
        let needs_output = {
            let work = shared.borrow();
            matches!(&work.mapped[seed], Stage::Ready(mapped) if !mapped.output.is_empty())
                && !(seed % 2 == 1 && work.hard_event(seed / 2))
        };
        if !needs_output {
            cursor += 1;
            continue;
        }
        let mut interval = CompletedInterval {
            first: seed,
            last: seed,
        };
        interval.expand(&shared.borrow());
        let mut left_missing = Vec::new();
        let mut right_missing = Vec::new();
        let mut reached_start = false;
        let mut reached_end = false;
        loop {
            // A newly mapped dependency changes this interval. Otherwise the
            // same segmentation serves both boundaries and every owned unit.
            let Some(known) = ClosureWindow::new(shared, &interval, owned)? else {
                break;
            };
            let head = known
                .semantic
                .projection
                .atoms
                .first()
                .expect("requested output")
                .code_point;
            let tail = known
                .semantic
                .projection
                .atoms
                .last()
                .expect("requested output")
                .code_point;
            let unresolved = {
                let work = shared.borrow();
                left_missing.iter().copied().find(|edge| {
                    work.missing.iter().any(|(first, last, own)| {
                        own == edge && !known.left_independent(*first, *last, reached_start)
                    })
                })
            };
            let internal_left = known.units[known.first_requested].start > 0 && reset(head);
            let left_proven = reached_start
                || internal_left
                || shared
                    .borrow()
                    .family_boundary(interval.first, true, head, Purpose::Mapped)?;
            if !left_proven || unresolved.is_some() {
                if reached_start {
                    return Err(unresolved
                        .expect("unqualified source state")
                        .unavailable(crate::SourceContinuityOperation::Segmentation));
                }
                reached_start = grow_boundary(
                    shared,
                    &mut interval,
                    &known.semantic,
                    true,
                    &mut left_missing,
                )?;
                continue;
            }
            {
                let work = shared.borrow();
                if let Some(edge) = right_missing.iter().find(|edge| {
                    work.missing
                        .iter()
                        .any(|(first, last, own)| own == *edge && known.crosses(*first, *last))
                }) {
                    return Err(edge.unavailable(crate::SourceContinuityOperation::Segmentation));
                }
            }
            let internal_right =
                known.units[known.last_requested].end < known.semantic.owners.len();
            if reached_end
                || internal_right
                || shared
                    .borrow()
                    .family_boundary(interval.last, false, tail, Purpose::Mapped)?
            {
                break;
            }
            reached_end = grow_boundary(
                shared,
                &mut interval,
                &known.semantic,
                false,
                &mut right_missing,
            )?;
        }
        // All requested components in the proved interval are discharged by
        // these same units; no component restarts the interval's segmentation.
        while owned.get(cursor).is_some_and(|slot| *slot <= interval.last) {
            cursor += 1;
        }
    }
    Ok(())
}

impl<'borrow, 'source> TransformWork<'borrow, 'source> {
    fn finish(mut self, selection_scopes: Option<Vec<bool>>) -> Result<Completed<'source>> {
        self.map.components.clear();
        self.map.emissions.clear();
        self.map.atoms.clear();
        self.map.events.clear();
        let mut atoms = Vec::new();
        let mut captured = Vec::new();
        let mut cuts = Vec::new();
        let mut spans = Vec::new();
        let mut output = Vec::new();
        let mut identities = Vec::new();
        let mut operations = Vec::new();
        let mut at = 0;
        let mut anchors = context::filled(self.context.anchor_order.len(), None)?;
        let mut expanded = false;
        for slot in 0..self.pre_case.len() {
            if self.stopped(slot) {
                context::push(&mut cuts, atoms.len())?;
            }
            let mapped = std::mem::replace(&mut self.mapped[slot], Stage::Unread);
            let mut emitted = false;
            if let Stage::Ready(mapped) = mapped {
                let Stage::Ready(Some(value)) = self.pre_case[slot] else {
                    unreachable!("completed mapping input")
                };
                let row = self.map.components.len();
                expanded |= !matches!(value.owner, ComponentOwner::Original(_));
                let next = atom(value, at)?;
                at = next.presentation.end;
                context::push(&mut atoms, next)?;
                context::push(
                    &mut self.map.components,
                    Component {
                        owner: value.owner,
                        source: value.source,
                        processed: value.processed,
                        style: value.style,
                        outputs: 0..0,
                    },
                )?;
                if let ComponentOwner::Original(original) = value.owner {
                    self.map.rows[original].separator_space = value.separator_space;
                }
                let start = output.len();
                for cp in mapped.output {
                    context::push(&mut output, cp)?;
                }
                for cp in mapped.identities {
                    context::push(&mut identities, cp)?;
                }
                context::push(&mut spans, start..output.len())?;
                context::push(&mut operations, mapped.operation)?;
                context::push(
                    &mut captured,
                    LogicalEvent {
                        atom: row,
                        source: value.source.start,
                        kind: Event::Text(row),
                    },
                )?;
                emitted = true;
            }
            if slot % 2 == 1 {
                let event = self.events[slot / 2];
                match event.kind {
                    Event::Text(_) => (),
                    Event::Deleted(..) => context::push(
                        &mut captured,
                        LogicalEvent {
                            atom: atoms.len(),
                            ..event
                        },
                    )?,
                    Event::Anchor(anchor) => {
                        let order = self
                            .context
                            .anchor_order
                            .iter()
                            .position(|own| *own == anchor)
                            .expect("actual anchor");
                        anchors[order] = Some(if emitted {
                            atoms.len() - 1
                        } else {
                            atoms.len()
                        });
                        if !emitted {
                            context::push(
                                &mut captured,
                                LogicalEvent {
                                    atom: atoms.len(),
                                    ..event
                                },
                            )?;
                        }
                    }
                }
            }
        }
        // Original-only staging retains its authentic processed coordinates.
        // Generated participants use the staged presentation coordinates.
        if !expanded {
            for (component, atom) in self.map.components.iter().zip(&mut atoms) {
                if let ComponentOwner::Original(row) = component.owner {
                    atom.presentation = self.original.atoms[row].presentation;
                }
            }
            at = atoms.last().map_or(0, |atom| atom.presentation.end);
        }
        cuts.sort_unstable();
        cuts.dedup();
        let mut input =
            SourceProjection::from_atoms_at_barriers(atoms, at, self.original.barriers, cuts)?;
        for (order, cut) in anchors.iter_mut().enumerate() {
            if let Some(at) = *cut {
                let generating = self.map.components.get(at).is_some_and(|component|
                    matches!(component.owner, ComponentOwner::Wbr(control) if self.context.anchor_order[order].item == SemanticAnchorItem::Control(control)));
                if !generating {
                    *cut = Some(self.context.project_anchor(order, &input, at));
                }
            }
        }
        input.anchor_atoms = anchors;
        input.style_rows = context::collect_rows(
            self.map
                .components
                .iter()
                .map(|component| Some(component.style)),
        )?;
        Ok(Completed {
            input,
            styles: self.styles,
            map: self.map,
            captured,
            mappings: Mappings {
                output,
                identities,
                spans,
                operations,
            },
            selection_scopes,
        })
    }
}

/// A genuine same-scope plain-text stretch. Storage-member markers do not
/// delimit it; actual boxes, atomics and source disconnections do.
struct SelectionRun {
    first: usize,
    last: usize,
    scope: usize,
}

/// A completed digit prefix with enough closed eligible units cannot become
/// eligible through later growth. An open last EGC is not counted: appending
/// ZWJ or a disqualifying mark can change that unit's digit eligibility.
/// This proof requires a real left reset, definite mixed gaps, actual non-width
/// EGCs, and no delayed-order ambiguity; unread is never completed absence.
fn overlimit_ready_prefix(
    shared: &RefCell<TransformWork<'_, '_>>,
    run: &SelectionRun,
    last_root: usize,
    limit: usize,
) -> Result<bool> {
    let work = shared.borrow();
    let topology = work.context.scope_topology.expect("captured selection");
    if !topology.capture.complete[topology.domain]
        || (run.first != 1 && !work.stopped(run.first - 1))
    {
        return Ok(false);
    }
    let mut first = None;
    let mut last = None;
    for slot in run.first..=run.last {
        if !slot.is_multiple_of(2) {
            if work.hard_event(slot / 2) {
                return Ok(false);
            }
            if let Event::Deleted(_, _, lexical) = work.events[slot / 2].kind
                && lexical.occurrences(&work.context.processed.lexical).any(|record| matches!(record.kind,
                    crate::processing::lexical_inventory::LexicalRecordKind::Character {
                        disposition: crate::processing::lexical_inventory::LexicalDisposition::Deleted { cause: crate::processing::DeletionCause::CollapsedSpace, .. }, ..
                    })) { return Ok(false); }
        }
        match &work.pre_case[slot] {
            Stage::Ready(None) => (),
            Stage::Ready(Some(_)) => match &work.mapped[slot] {
                Stage::Ready(mapped) => {
                    if !mapped.identities.is_empty() {
                        first.get_or_insert(slot);
                        last = Some(slot);
                    }
                }
                Stage::Unread => break,
            },
            Stage::Unread => break,
        }
    }
    let (Some(first), Some(last)) = (first, last) else {
        return Ok(false);
    };
    if last < last_root
        || work
            .missing
            .iter()
            .any(|(before, after, _)| *before <= last && first <= *after)
    {
        return Ok(false);
    }
    drop(work);
    let semantic = semantic_window(shared, run.first, last, SemanticBasis::SelectionIdentity)?;
    if semantic.delayed || semantic.pending_tail {
        return Ok(false);
    }
    let units = super::project::scalar_units(&semantic.projection)?;
    if units.len() <= limit {
        return Ok(false);
    }
    if !units.iter().all(|unit| {
        super::combined::selection::digit_code_points(
            semantic.projection.atoms[unit.clone()]
                .iter()
                .map(|atom| atom.code_point),
        )
    }) {
        return Ok(false);
    }
    let tail = semantic
        .projection
        .atoms
        .last()
        .expect("actual digit units")
        .code_point;
    let right_closed =
        shared
            .borrow()
            .family_boundary(last, false, tail, Purpose::SelectionIdentity)?;
    if units.len() - usize::from(!right_closed) <= limit {
        return Ok(false);
    }
    Ok(true)
}

fn selection_dependencies(
    shared: &RefCell<TransformWork<'_, '_>>,
    roots: &[usize],
) -> Result<Option<Vec<bool>>> {
    use super::combined::{self, TextCombineError};
    use crate::{
        TextCombineUpright as Mode,
        processing::lexical_inventory::{LexicalDisposition, LexicalRecordKind},
    };
    let work = shared.borrow();
    if !work.context.combination_requested() || work.context.whole_domain {
        return Ok(None);
    }
    let source_context = work.context;
    let zero = |event: &LogicalEvent| match event.kind {
        Event::Deleted(_, _, lexical) => lexical
            .occurrences(&source_context.processed.lexical)
            .any(|record| {
                matches!(
                    record.kind,
                    LexicalRecordKind::Character {
                        disposition: LexicalDisposition::Deleted {
                            cause: crate::processing::DeletionCause::CollapsedSpace,
                            ..
                        },
                        ..
                    }
                )
            }),
        _ => false,
    };
    let semantic = roots.iter().any(|&slot| {
        matches!(&work.mapped[slot], Stage::Ready(mapped) if !mapped.identities.is_empty())
            || slot % 2 == 1 && zero(&work.events[slot / 2])
    });
    if !semantic {
        return Ok(None);
    }
    let topology = work
        .context
        .scope_topology
        .ok_or_else(|| combined::refused(TextCombineError::MissingCapture))?;
    let capture = &topology.capture;
    let mut active_scopes = context::filled(capture.scopes.len(), false)?;
    if capture.scopes[capture.domain_scopes[topology.domain]].writing_axis
        == Some(crate::SourceWritingAxis::Horizontal)
    {
        return Ok(Some(active_scopes));
    }
    let mut event_scopes = Vec::new();
    let mut event_orders = Vec::new();
    let mut atomic = Vec::new();
    for event in &work.events {
        let order = work
            .context
            .event_order(event, &work.original)
            .expect("captured source occurrence");
        let mut scope = capture.events[order].before.expect("source scope");
        if let Event::Text(row) = event.kind {
            for piece in work.map.original_pieces(row) {
                let other = capture.events
                    [piece.origin.captured_event().expect("captured original")]
                .before
                .expect("source original scope");
                scope = combined::selection::common(capture, scope, other);
            }
        }
        context::push(&mut event_scopes, scope)?;
        context::push(&mut event_orders, order)?;
        context::push(
            &mut atomic,
            matches!(event.kind, Event::Anchor(anchor) if matches!(anchor.item, SemanticAnchorItem::Atomic(_))),
        )?;
    }
    let mut boxes = Vec::new();
    for &order in &capture.flow_orders[topology.domain] {
        if let crate::SourceProcessingEvent::EnterScope(scope)
        | crate::SourceProcessingEvent::ExitScope(scope) = capture.events[order].event
            && capture.scopes[scope].definition.is_none()
        {
            context::push(&mut boxes, order)?;
        }
    }
    let mut runs: Vec<SelectionRun> = Vec::new();
    let mut run_for_event = Vec::new();
    for index in 0..work.events.len() {
        let boxed = index > 0
            && boxes
                .get(boxes.partition_point(|&order| order <= event_orders[index - 1]))
                .is_some_and(|&order| order <= event_orders[index]);
        let new = index == 0
            || event_scopes[index - 1] != event_scopes[index]
            || work.hard_gaps[index]
            || boxed
            || atomic[index]
            || atomic[index - 1];
        if new {
            context::push(
                &mut runs,
                SelectionRun {
                    first: index * 2 + 1,
                    last: index * 2 + 1,
                    scope: event_scopes[index],
                },
            )?;
        } else {
            runs.last_mut().expect("source run").last = index * 2 + 1;
        }
        context::push(&mut run_for_event, runs.len() - 1)?;
    }
    let mut wanted = context::filled(runs.len(), false)?;
    let mut visited = context::filled(work.pre_case.len(), false)?;
    let mut seeds: Vec<Vec<usize>> = context::collect_rows(runs.iter().map(|_| Vec::new()))?;
    for &slot in roots {
        let values = match &work.mapped[slot] {
            Stage::Ready(mapped) => mapped.identities.as_slice(),
            Stage::Unread => &[],
        };
        let zero_space = slot % 2 == 1 && zero(&work.events[slot / 2]);
        if values.is_empty() && !zero_space {
            continue;
        }
        let scope = if let Stage::Ready(Some(value)) = work.pre_case[slot] {
            match value.owner {
                ComponentOwner::Original(_) | ComponentOwner::Wbr(_) => event_scopes[slot / 2],
                ComponentOwner::Virtual(index) => {
                    work.map.scopes[work.map.virtuals[index].scope].ordinal
                }
            }
        } else {
            event_scopes[slot / 2]
        };
        let mode = capture.scopes[scope]
            .style
            .as_ref()
            .map(|style| style.text_combine_upright)
            .ok_or_else(|| combined::refused(TextCombineError::MissingMode { scope }))?;
        let eligible = match mode {
            Mode::None => false,
            Mode::All => true,
            Mode::Digits(_) => values.iter().any(|cp| (0x30..=0x39).contains(cp)),
        };
        if !eligible {
            continue;
        }
        active_scopes[scope] = true;
        let event = (slot / 2).min(work.events.len().saturating_sub(1));
        if let Some(&run) = run_for_event.get(event)
            && runs[run].scope == scope
        {
            wanted[run] = true;
            context::push(&mut seeds[run], slot)?;
        }
    }
    drop(work);
    for (index, run) in runs.iter().enumerate() {
        if !wanted[index] {
            continue;
        }
        let mode = capture.scopes[run.scope]
            .style
            .as_ref()
            .expect("admitted mode")
            .text_combine_upright;
        if let Mode::Digits(limit) = mode
            && overlimit_ready_prefix(
                shared,
                run,
                *seeds[index].last().expect("actual requested seed"),
                limit.max_units(),
            )?
        {
            wanted[index] = false;
        }
    }
    active_scopes.fill(false);
    for (index, run) in runs.iter().enumerate() {
        if wanted[index] {
            active_scopes[run.scope] = true;
        }
    }
    // Each actual All stretch is staged as one canonical word-fact cohort.
    // Digits grows only to its first actual non-digit/mark outcome per side.
    for (run_index, run) in runs.iter().enumerate() {
        if !wanted[run_index] {
            continue;
        }
        let mode = shared
            .borrow()
            .context
            .scope_topology
            .expect("capture")
            .capture
            .scopes[run.scope]
            .style
            .as_ref()
            .expect("admitted mode")
            .text_combine_upright;
        let mut wanted_slots = Vec::new();
        if mode == Mode::All {
            for slot in run.first..=run.last {
                if slot.is_multiple_of(2)
                    && !shared
                        .borrow()
                        .phrase_reads
                        .may_stage_scope(slot / 2, run.scope)
                {
                    continue;
                }
                context::push(&mut wanted_slots, slot)?;
            }
        } else {
            for &seed in &seeds[run_index] {
                if visited[seed] {
                    continue;
                }
                visited[seed] = true;
                context::push(&mut wanted_slots, seed)?;
                for reverse in [true, false] {
                    let mut at = seed;
                    loop {
                        let next = if reverse {
                            at.checked_sub(1).filter(|&next| run.first <= next)
                        } else {
                            at.checked_add(1).filter(|&next| next <= run.last)
                        };
                        let Some(next) = next else { break };
                        if visited[next] {
                            break;
                        }
                        visited[next] = true;
                        let missing = {
                            shared
                                .borrow()
                                .missing
                                .iter()
                                .find(|(first, last, _)| *first <= next && next <= *last)
                                .map(|(_, _, edge)| *edge)
                        };
                        if let Some(edge) = missing {
                            return Err(
                                edge.unavailable(crate::SourceContinuityOperation::Combination)
                            );
                        }
                        if next.is_multiple_of(2)
                            && !shared
                                .borrow()
                                .phrase_reads
                                .may_stage_scope(next / 2, run.scope)
                        {
                            break;
                        }
                        shared.borrow_mut().cohort[next] = true;
                        shared.borrow_mut().read(next)?;
                        map_slot(shared, next)?;
                        context::push(&mut wanted_slots, next)?;
                        let continues = {
                            let work = shared.borrow();
                            match &work.mapped[next] {
                                Stage::Ready(mapped) => mapped.identities.iter().all(|&cp| {
                                    (0x30..=0x39).contains(&cp)
                                        || icu_properties::props::GeneralCategoryGroup::Mark
                                            .contains(
                                                CodePointMapData::<
                                                    icu_properties::props::GeneralCategory,
                                                >::new(
                                                )
                                                .get32(cp),
                                            )
                                }),
                                Stage::Unread => {
                                    !matches!(work.events[next / 2].kind, Event::Deleted(..))
                                        || !zero(&work.events[next / 2])
                                }
                            }
                        };
                        at = next;
                        if !continues {
                            break;
                        }
                    }
                }
            }
        }
        wanted_slots.sort_unstable();
        wanted_slots.dedup();
        for &slot in &wanted_slots {
            if let Some(edge) = shared
                .borrow()
                .missing
                .iter()
                .find(|(first, last, _)| *first <= slot && slot <= *last)
                .map(|(_, _, edge)| *edge)
            {
                return Err(edge.unavailable(crate::SourceContinuityOperation::Combination));
            }
            shared.borrow_mut().cohort[slot] = true;
            shared.borrow_mut().read(slot)?;
        }
        for slot in wanted_slots {
            map_slot(shared, slot)?;
        }
    }
    // A real box can end a plain-text run without ending hypothetical NCA
    // lookaround. Demand only the nearest completed semantic unit on each
    // relevant side; unrelated scopes whose NCA is None stay unread.
    let mut neighbor_units = Vec::new();
    for (index, run) in runs.iter().enumerate() {
        if !wanted[index] {
            continue;
        }
        for reverse in [true, false] {
            let other = if reverse {
                index.checked_sub(1)
            } else {
                index.checked_add(1).filter(|&next| next < runs.len())
            };
            let Some(other) = other else { continue };
            let neighbor = &runs[other];
            let boundary = if reverse {
                run.first / 2
            } else {
                neighbor.first / 2
            };
            if shared.borrow().hard_gaps[boundary]
                || atomic[run.first / 2]
                || atomic[neighbor.first / 2]
            {
                continue;
            }
            let ancestor = combined::selection::common(capture, run.scope, neighbor.scope);
            let hypothetical = capture.scopes[ancestor]
                .style
                .as_ref()
                .map(|style| style.text_combine_upright)
                .ok_or_else(|| {
                    combined::refused(TextCombineError::MissingMode { scope: ancestor })
                })?;
            if hypothetical == Mode::None {
                continue;
            }
            let first = if reverse {
                neighbor.last + 1
            } else {
                run.last + 1
            };
            let mut at = first;
            loop {
                if let Some(edge) = shared
                    .borrow()
                    .missing
                    .iter()
                    .find(|(start, end, _)| *start <= at && at <= *end)
                    .map(|(_, _, edge)| *edge)
                {
                    return Err(edge.unavailable(crate::SourceContinuityOperation::Combination));
                }
                if !at.is_multiple_of(2)
                    || shared
                        .borrow()
                        .phrase_reads
                        .may_stage_scope(at / 2, ancestor)
                {
                    shared.borrow_mut().cohort[at] = true;
                    shared.borrow_mut().read(at)?;
                    if map_slot(shared, at)? {
                        let contributes = matches!(&shared.borrow().mapped[at], Stage::Ready(mapped) if !mapped.identities.is_empty());
                        if contributes {
                            context::push(&mut neighbor_units, at)?;
                            break;
                        }
                    } else if !at.is_multiple_of(2) && zero(&shared.borrow().events[at / 2]) {
                        break;
                    }
                }
                let next = if reverse {
                    at.checked_sub(1).filter(|&next| neighbor.first <= next)
                } else {
                    at.checked_add(1).filter(|&next| next <= neighbor.last)
                };
                let Some(next) = next else { break };
                at = next;
            }
        }
    }
    neighbor_units.sort_unstable();
    neighbor_units.dedup();
    close_units(shared, &neighbor_units)?;
    Ok(Some(active_scopes))
}

pub(super) fn run<'source>(
    context: &SourceContext<'source>,
    input: SourceProjection,
) -> Result<Completed<'source>> {
    let shared = RefCell::new(TransformWork::new(context, input)?);
    let length = shared.borrow().pre_case.len();
    let mut requested = Vec::new();
    for slot in 0..length {
        let wanted = {
            let work = shared.borrow();
            if slot % 2 == 1 {
                work.owns_event(slot / 2)
            } else {
                let cut = slot / 2;
                (cut.checked_sub(1)
                    .is_some_and(|index| work.owns_event(index))
                    || work.events.get(cut).is_some_and(|_| work.owns_event(cut)))
                    && work.phrase_reads.root_gap(work.context, cut)
            }
        };
        if wanted {
            shared.borrow_mut().cohort[slot] = true;
            context::push(&mut requested, slot)?;
        }
    }
    // All these sites belong to the actual request. Stage their immutable
    // pre-case values once before any per-component casing or locale execution.
    for &slot in &requested {
        shared.borrow_mut().read(slot)?;
    }
    let mut owned = Vec::new();
    for &slot in &requested {
        if map_slot(&shared, slot)? {
            context::push(&mut owned, slot)?;
        }
    }
    close_units(&shared, &owned)?;
    let selection_scopes = selection_dependencies(&shared, &requested)?;
    close_units(&shared, &owned)?;
    shared.into_inner().finish(selection_scopes)
}
