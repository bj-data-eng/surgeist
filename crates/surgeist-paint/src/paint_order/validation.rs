//! Operation-local incidence proofs and ordered, admitted scratch indices.

use std::{mem::take, ops::Range};

use super::model::*;
use crate::{
    BoxId, ChildOrder, PaintError, Participation, Position, ResourceKind, WorkKind,
    accounting::{PhaseMeter, allocate, charge, release},
    algorithm::{SortBuffers, State},
};
use surgeist_layout::SourceIndex;
use surgeist_ledger::Scope;

#[derive(Clone, Copy)]
struct SourceKey {
    parent: usize,
    order: SourceIndex,
}
#[derive(Clone, Copy)]
struct FragmentKey {
    owner: usize,
    order: usize,
    placement: FragmentPlacement,
    parent_owner: Option<usize>,
    parent_order: Option<usize>,
    line_order: Option<usize>,
}
#[derive(Clone, Copy)]
enum FragmentPlacement {
    Ordinary,
    Promoted { container: FragmentId },
}
#[derive(Clone, Copy)]
struct LineKey {
    container: usize,
    order: usize,
}
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct PrimitiveKey {
    bucket: usize,
    role: u8,
    applicator: usize,
    order: usize,
}

#[derive(Default)]
pub(super) struct PaintIndex {
    pub(super) fragments: Vec<usize>,
    pub(super) lines: Vec<usize>,
    pub(super) primitives: Vec<usize>,
    pub(super) sources: Vec<usize>,
    pub(super) content: Vec<ContentEntry>,
    pub(super) fragment_ranges: Vec<Range<usize>>,
    pub(super) line_ranges: Vec<Range<usize>>,
    pub(super) primitive_ranges: Vec<Range<usize>>,
    pub(super) source_ranges: Vec<Range<usize>>,
    pub(super) content_ranges: Vec<Range<usize>>,
    fragment_keys: Vec<FragmentKey>,
    line_keys: Vec<LineKey>,
    primitive_keys: Vec<PrimitiveKey>,
    source_keys: Vec<SourceKey>,
    content_entries: Vec<ContentEntry>,
    content_keys: Vec<LineKey>,
    content_order: Vec<usize>,
    merge: Vec<usize>,
}

impl PaintIndex {
    pub(super) fn cleanup(
        &mut self,
        p: &mut PhaseMeter<'_, '_>,
        root: Scope,
    ) -> Result<(), PaintError> {
        macro_rules! drop_scratch {
            ($($field:ident),+ $(,)?) => {$(
                release(p, root, take(&mut self.$field), ResourceKind::ScratchBytes)?;
            )+};
        }
        drop_scratch!(
            fragments,
            lines,
            primitives,
            sources,
            fragment_ranges,
            line_ranges,
            primitive_ranges,
            source_ranges,
            fragment_keys,
            line_keys,
            primitive_keys,
            source_keys,
            content,
            content_ranges,
            content_entries,
            content_keys,
            content_order,
            merge,
        );
        Ok(())
    }
}

fn scratch<T>(p: &mut PhaseMeter<'_, '_>, root: Scope, n: usize) -> Result<Vec<T>, PaintError> {
    allocate(p, root, n, ResourceKind::ScratchBytes)
}

fn push<T>(
    values: &mut Vec<T>,
    value: T,
    p: &mut PhaseMeter<'_, '_>,
    phase: Scope,
) -> Result<(), PaintError> {
    charge(p, phase, WorkKind::OrderMove, 1)?;
    values.push(value);
    Ok(())
}

fn replaced(kind: BoxPaintKind) -> bool {
    matches!(
        kind,
        BoxPaintKind::BlockReplaced | BoxPaintKind::InlineReplaced
    )
}

fn source_child(slot: usize, boxes: usize) -> SourceChild {
    if slot < boxes {
        SourceChild::Box(BoxId::new(slot))
    } else {
        SourceChild::Text(TextId::new(slot - boxes))
    }
}

fn fragment_owner(slot: usize, boxes: usize) -> FragmentOwner {
    if slot < boxes {
        FragmentOwner::Box(BoxId::new(slot))
    } else {
        FragmentOwner::Text(TextId::new(slot - boxes))
    }
}

/// This consumes the shared core's proved topology, never reclassifies contexts.
pub(super) fn prepare(
    request: PaintRequest<'_>,
    core: &State,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
) -> Result<PaintIndex, PaintError> {
    let mut index = PaintIndex::default();
    let result = (|| {
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        if request.boxes().len() != request.box_paint().len() {
            return Err(PaintInputError::BoxPaintCount {
                boxes: request.boxes().len(),
                box_paint: request.box_paint().len(),
            }
            .into());
        }
        validate_categories(request, core, p, phase)?;
        let owners = request
            .boxes()
            .len()
            .checked_add(request.texts().len())
            .ok_or(PaintError::CapacityOverflow)?;
        let primitive_buckets = request
            .fragments()
            .len()
            .checked_add(1)
            .ok_or(PaintError::CapacityOverflow)?;
        let content_capacity = request
            .lines()
            .len()
            .checked_add(request.fragments().len())
            .ok_or(PaintError::CapacityOverflow)?;
        let longest = owners
            .max(request.fragments().len())
            .max(request.lines().len())
            .max(request.primitives().len());
        let longest = longest.max(content_capacity);
        index.merge = scratch(p, root, longest)?;
        charge(p, phase, WorkKind::OrderMove, longest)?;
        index.merge.resize(longest, 0);
        sources(request, &mut index, p, root, phase, owners)?;
        fragments(request, &mut index, p, root, phase)?;
        lines(request, &mut index, p, root, phase)?;
        incidences(request, p, phase)?;
        content(request, &mut index, p, root, phase, content_capacity)?;
        primitives(request, core, &mut index, p, root, phase)?;
        index.fragment_ranges = ranges(p, root, phase, owners, &index.fragments, |slot| {
            index.fragment_keys[slot].owner
        })?;
        index.line_ranges = ranges(
            p,
            root,
            phase,
            request.fragments().len(),
            &index.lines,
            |slot| index.line_keys[slot].container,
        )?;
        index.primitive_ranges = ranges(
            p,
            root,
            phase,
            primitive_buckets,
            &index.primitives,
            |slot| index.primitive_keys[slot].bucket,
        )?;
        index.source_ranges = ranges(
            p,
            root,
            phase,
            request.boxes().len(),
            &index.sources,
            |slot| index.source_keys[slot].parent,
        )?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(index),
        Err(error) => {
            index.cleanup(p, root)?;
            Err(error)
        }
    }
}

fn validate_categories(
    request: PaintRequest<'_>,
    core: &State,
    p: &mut PhaseMeter<'_, '_>,
    phase: Scope,
) -> Result<(), PaintError> {
    for (slot, record) in request.box_paint().iter().enumerate() {
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let paint = *record;
        let box_id = BoxId::new(slot);
        let facts = request.boxes()[slot];
        if matches!(
            paint.kind(),
            BoxPaintKind::Table | BoxPaintKind::InlineTable
        ) {
            return Err(PaintInputError::UnsupportedBoxKind {
                box_id,
                kind: paint.kind(),
            }
            .into());
        }
        let inline = matches!(
            paint.kind(),
            BoxPaintKind::Inline | BoxPaintKind::InlineBlock | BoxPaintKind::InlineReplaced
        );
        let reason = if box_id == request.root() && paint.kind() != BoxPaintKind::Block {
            Some(BoxCategoryError::RootMustBeBlock)
        } else if box_id == request.root() && paint.float() == FloatParticipation::Floating {
            Some(BoxCategoryError::FloatingRoot)
        } else if inline && paint.float() == FloatParticipation::Floating {
            Some(BoxCategoryError::FloatingInline)
        } else if inline && facts.position().out_of_flow() {
            Some(BoxCategoryError::OutOfFlowInline)
        } else if inline && facts.participation() != Participation::Ordinary {
            Some(BoxCategoryError::InlineItem)
        } else if paint.float() == FloatParticipation::Floating
            && facts.participation() != Participation::Ordinary
        {
            Some(BoxCategoryError::FloatingItem)
        } else if replaced(paint.kind()) && core.nodes[slot].start != core.nodes[slot].end {
            Some(BoxCategoryError::ReplacedHasChildren)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(PaintInputError::InvalidBoxCategory { box_id, reason }.into());
        }
    }
    Ok(())
}

fn sources(
    request: PaintRequest<'_>,
    index: &mut PaintIndex,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
    owners: usize,
) -> Result<(), PaintError> {
    let boxes = request.boxes().len();
    index.sources = scratch(p, root, owners - 1)?;
    index.source_keys = scratch(p, root, owners)?;
    for (slot, facts) in request.boxes().iter().enumerate() {
        charge(p, phase, WorkKind::InlineMemberVisit, 1)?;
        push(
            &mut index.source_keys,
            SourceKey {
                parent: facts.parent().map_or(usize::MAX, BoxId::get),
                order: facts.source_index(),
            },
            p,
            phase,
        )?;
        if facts.parent().is_some() {
            push(&mut index.sources, slot, p, phase)?;
        }
    }
    for (slot, record) in request.texts().iter().enumerate() {
        charge(p, phase, WorkKind::InlineMemberVisit, 1)?;
        let text = *record;
        let parent = text.parent();
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        if parent.get() >= boxes {
            return Err(PaintInputError::MissingTextParent {
                text: TextId::new(slot),
                parent,
            }
            .into());
        }
        if replaced(request.box_paint()[parent.get()].kind())
            || request.boxes()[parent.get()].child_order() != ChildOrder::Source
        {
            return Err(PaintInputError::InvalidTextParent {
                text: TextId::new(slot),
                parent,
            }
            .into());
        }
        push(
            &mut index.source_keys,
            SourceKey {
                parent: parent.get(),
                order: text.source_index(),
            },
            p,
            phase,
        )?;
        push(&mut index.sources, boxes + slot, p, phase)?;
    }
    SortBuffers::new(&mut index.sources, &mut index.merge)?.sort(
        p,
        phase,
        WorkKind::PaintOrderComparison,
        |a, b| {
            let a = index.source_keys[a];
            let b = index.source_keys[b];
            (a.parent, a.order).cmp(&(b.parent, b.order))
        },
    )?;
    for pair in index.sources.windows(2) {
        charge(p, phase, WorkKind::PaintOrderComparison, 1)?;
        let a = index.source_keys[pair[0]];
        let b = index.source_keys[pair[1]];
        if a.parent == b.parent && a.order == b.order {
            return Err(PaintInputError::DuplicateMixedSourceIndex {
                parent: BoxId::new(a.parent),
                source_index: a.order,
                first: source_child(pair[0], boxes),
                other: source_child(pair[1], boxes),
            }
            .into());
        }
    }
    Ok(())
}

fn fragments(
    request: PaintRequest<'_>,
    index: &mut PaintIndex,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
) -> Result<(), PaintError> {
    let n = request.fragments().len();
    index.fragments = scratch(p, root, n)?;
    index.fragment_keys = scratch(p, root, n)?;
    for (slot, record) in request.fragments().iter().enumerate() {
        charge(p, phase, WorkKind::FragmentVisit, 1)?;
        let fragment = *record;
        let id = FragmentId::new(slot);
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let (owner, semantic_parent) = match fragment.owner() {
            FragmentOwner::Box(owner) if owner.get() < request.boxes().len() => {
                (owner.get(), request.boxes()[owner.get()].parent())
            }
            FragmentOwner::Text(owner) if owner.get() < request.texts().len() => (
                request.boxes().len() + owner.get(),
                Some(request.texts()[owner.get()].parent()),
            ),
            owner => {
                return Err(PaintInputError::MissingFragmentOwner {
                    fragment: id,
                    owner,
                }
                .into());
            }
        };
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let mut parent_owner = None;
        let (placement, parent_order) = match fragment.parent() {
            Some(parent) => {
                let Some(parent_record) = request.fragments().get(parent.get()) else {
                    return Err(PaintInputError::MissingFragmentParent {
                        fragment: id,
                        parent,
                    }
                    .into());
                };
                let placement = match fragment.data() {
                    FragmentData::BlockInInline { owner, placement } => {
                        promotion(request, id, owner, placement, *parent_record, p, phase)?;
                        FragmentPlacement::Promoted {
                            container: placement.container(),
                        }
                    }
                    FragmentData::Box { .. } | FragmentData::Text { .. } => {
                        if semantic_parent.map(FragmentOwner::Box) != Some(parent_record.owner()) {
                            return Err(PaintInputError::InvalidFragmentParent {
                                fragment: id,
                                parent: Some(parent),
                            }
                            .into());
                        }
                        FragmentPlacement::Ordinary
                    }
                };
                if let FragmentOwner::Box(parent_box) = parent_record.owner() {
                    parent_owner = Some(parent_box.get());
                }
                if matches!(placement, FragmentPlacement::Ordinary)
                    && let FragmentOwner::Box(box_id) = fragment.owner()
                {
                    charge(p, phase, WorkKind::PaintValidation, 1)?;
                    let paint = request.box_paint()[box_id.get()];
                    let facts = request.boxes()[box_id.get()];
                    if matches!(
                        paint.kind(),
                        BoxPaintKind::Block | BoxPaintKind::BlockReplaced
                    ) && paint.float() == FloatParticipation::None
                        && facts.position() == Position::Static
                        && semantic_parent.is_some_and(|owner| {
                            request.box_paint()[owner.get()].kind() == BoxPaintKind::Inline
                        })
                    {
                        return Err(PaintInputError::InvalidBlockInInlinePlacement {
                            fragment: id,
                            container: parent,
                        }
                        .into());
                    }
                }
                (placement, Some(parent_record.logical_order()))
            }
            None => {
                if semantic_parent.is_some() {
                    return Err(PaintInputError::InvalidFragmentParent {
                        fragment: id,
                        parent: None,
                    }
                    .into());
                }
                (FragmentPlacement::Ordinary, None)
            }
        };
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let line_order = match fragment.line() {
            Some(line) => {
                let Some(record) = request.lines().get(line.get()) else {
                    return Err(PaintInputError::MissingFragmentLine { fragment: id, line }.into());
                };
                Some(record.logical_order())
            }
            None => fragment
                .block_in_inline_placement()
                .map(|placement| placement.content_order()),
        };
        push(
            &mut index.fragment_keys,
            FragmentKey {
                owner,
                order: fragment.logical_order(),
                placement,
                parent_owner,
                parent_order,
                line_order,
            },
            p,
            phase,
        )?;
        push(&mut index.fragments, slot, p, phase)?;
    }
    SortBuffers::new(&mut index.fragments, &mut index.merge)?.sort(
        p,
        phase,
        WorkKind::PaintOrderComparison,
        |a, b| {
            let a = index.fragment_keys[a];
            let b = index.fragment_keys[b];
            (a.owner, a.order).cmp(&(b.owner, b.order))
        },
    )?;
    for pair in index.fragments.windows(2) {
        charge(p, phase, WorkKind::PaintOrderComparison, 1)?;
        let a = index.fragment_keys[pair[0]];
        let b = index.fragment_keys[pair[1]];
        if a.owner != b.owner {
            continue;
        }
        let first = FragmentId::new(pair[0]);
        let other = FragmentId::new(pair[1]);
        if a.order == b.order {
            return Err(PaintInputError::DuplicateFragmentOrder {
                owner: fragment_owner(a.owner, request.boxes().len()),
                logical_order: a.order,
                first,
                other,
            }
            .into());
        }
        match (a.placement, b.placement) {
            (FragmentPlacement::Promoted { container }, FragmentPlacement::Ordinary) => {
                return Err(PaintInputError::InvalidBlockInInlinePlacement {
                    fragment: first,
                    container,
                }
                .into());
            }
            (FragmentPlacement::Ordinary, FragmentPlacement::Promoted { container }) => {
                return Err(PaintInputError::InvalidBlockInInlinePlacement {
                    fragment: other,
                    container,
                }
                .into());
            }
            (FragmentPlacement::Ordinary, FragmentPlacement::Ordinary)
            | (FragmentPlacement::Promoted { .. }, FragmentPlacement::Promoted { .. }) => {}
        }
        if a.parent_owner > b.parent_owner
            || (a.parent_owner == b.parent_owner
                && (a.parent_order > b.parent_order
                    || (a.parent_order == b.parent_order && a.line_order > b.line_order)))
        {
            return Err(PaintInputError::InconsistentFragmentOrder { first, other }.into());
        }
    }
    Ok(())
}

fn lines(
    request: PaintRequest<'_>,
    index: &mut PaintIndex,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
) -> Result<(), PaintError> {
    let n = request.lines().len();
    index.lines = scratch(p, root, n)?;
    index.line_keys = scratch(p, root, n)?;
    for (slot, record) in request.lines().iter().enumerate() {
        charge(p, phase, WorkKind::LineVisit, 1)?;
        let line = *record;
        let id = LineId::new(slot);
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let Some(container) = request.fragments().get(line.container().get()) else {
            return Err(PaintInputError::MissingLineContainer {
                line: id,
                container: line.container(),
            }
            .into());
        };
        let valid = match container.owner() {
            FragmentOwner::Box(owner) => {
                matches!(
                    request.box_paint()[owner.get()].kind(),
                    BoxPaintKind::Block | BoxPaintKind::InlineBlock
                ) && request.boxes()[owner.get()].child_order() == ChildOrder::Source
            }
            FragmentOwner::Text(_) => false,
        };
        if !valid {
            return Err(PaintInputError::InvalidLineContainer {
                line: id,
                container: line.container(),
            }
            .into());
        }
        push(
            &mut index.line_keys,
            LineKey {
                container: line.container().get(),
                order: line.logical_order(),
            },
            p,
            phase,
        )?;
        push(&mut index.lines, slot, p, phase)?;
    }
    SortBuffers::new(&mut index.lines, &mut index.merge)?.sort(
        p,
        phase,
        WorkKind::PaintOrderComparison,
        |a, b| {
            let a = index.line_keys[a];
            let b = index.line_keys[b];
            (a.container, a.order).cmp(&(b.container, b.order))
        },
    )?;
    for pair in index.lines.windows(2) {
        charge(p, phase, WorkKind::PaintOrderComparison, 1)?;
        let a = index.line_keys[pair[0]];
        let b = index.line_keys[pair[1]];
        if a.container == b.container && a.order == b.order {
            return Err(PaintInputError::DuplicateLineOrder {
                container: FragmentId::new(a.container),
                logical_order: a.order,
                first: LineId::new(pair[0]),
                other: LineId::new(pair[1]),
            }
            .into());
        }
    }
    Ok(())
}

fn promotion(
    request: PaintRequest<'_>,
    id: FragmentId,
    owner: BoxId,
    placement: BlockInInlinePlacement,
    parent: PaintFragment,
    p: &mut PhaseMeter<'_, '_>,
    phase: Scope,
) -> Result<(), PaintError> {
    let invalid = || {
        PaintError::Input(PaintInputError::InvalidBlockInInlinePlacement {
            fragment: id,
            container: placement.container(),
        })
    };
    charge(p, phase, WorkKind::PaintValidation, 1)?;
    let facts = request.boxes()[owner.get()];
    let appearance = request.box_paint()[owner.get()];
    if !matches!(
        appearance.kind(),
        BoxPaintKind::Block | BoxPaintKind::BlockReplaced
    ) || appearance.float() != FloatParticipation::None
        || facts.position() != Position::Static
    {
        return Err(invalid());
    }
    charge(p, phase, WorkKind::PaintValidation, 1)?;
    let FragmentOwner::Box(container) = parent.owner() else {
        return Err(invalid());
    };
    let Some(container_facts) = request.boxes().get(container.get()) else {
        return Err(invalid());
    };
    if !matches!(
        request.box_paint()[container.get()].kind(),
        BoxPaintKind::Block | BoxPaintKind::InlineBlock
    ) || container_facts.child_order() != ChildOrder::Source
    {
        return Err(invalid());
    }
    let mut ancestor = facts.parent();
    let mut skipped_inline = false;
    let mut unsupported = None;
    loop {
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        match ancestor {
            Some(ancestor_id) if ancestor_id == container => {
                if let Some(through) = unsupported {
                    return Err(PaintInputError::UnsupportedFragmentTopology {
                        fragment: id,
                        container: placement.container(),
                        through,
                    }
                    .into());
                }
                return if skipped_inline {
                    Ok(())
                } else {
                    Err(invalid())
                };
            }
            Some(ancestor_id) => {
                if request.box_paint()[ancestor_id.get()].kind() == BoxPaintKind::Inline {
                    skipped_inline = true;
                } else if unsupported.is_none() {
                    unsupported = Some(ancestor_id);
                }
                ancestor = request.boxes()[ancestor_id.get()].parent();
            }
            None => return Err(invalid()),
        }
    }
}

fn content(
    request: PaintRequest<'_>,
    index: &mut PaintIndex,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
    capacity: usize,
) -> Result<(), PaintError> {
    index.content_entries = scratch(p, root, capacity)?;
    index.content_keys = scratch(p, root, capacity)?;
    index.content_order = scratch(p, root, capacity)?;
    for (slot, record) in request.lines().iter().enumerate() {
        charge(p, phase, WorkKind::LineVisit, 1)?;
        push(
            &mut index.content_order,
            index.content_entries.len(),
            p,
            phase,
        )?;
        push(
            &mut index.content_keys,
            LineKey {
                container: record.container().get(),
                order: record.logical_order(),
            },
            p,
            phase,
        )?;
        push(
            &mut index.content_entries,
            ContentEntry::Line(LineId::new(slot)),
            p,
            phase,
        )?;
    }
    for (slot, record) in request.fragments().iter().enumerate() {
        charge(p, phase, WorkKind::FragmentVisit, 1)?;
        if let Some(placement) = record.block_in_inline_placement() {
            push(
                &mut index.content_order,
                index.content_entries.len(),
                p,
                phase,
            )?;
            push(
                &mut index.content_keys,
                LineKey {
                    container: placement.container().get(),
                    order: placement.content_order(),
                },
                p,
                phase,
            )?;
            push(
                &mut index.content_entries,
                ContentEntry::PromotedBlock(FragmentId::new(slot)),
                p,
                phase,
            )?;
        }
    }
    SortBuffers::new(&mut index.content_order, &mut index.merge)?.sort(
        p,
        phase,
        WorkKind::PaintOrderComparison,
        |a, b| {
            let a = index.content_keys[a];
            let b = index.content_keys[b];
            (a.container, a.order).cmp(&(b.container, b.order))
        },
    )?;
    for pair in index.content_order.windows(2) {
        charge(p, phase, WorkKind::PaintOrderComparison, 1)?;
        let a = index.content_keys[pair[0]];
        let b = index.content_keys[pair[1]];
        if a.container == b.container && a.order == b.order {
            return Err(PaintInputError::DuplicateContentOrder {
                container: FragmentId::new(a.container),
                logical_order: a.order,
                first: index.content_entries[pair[0]],
                other: index.content_entries[pair[1]],
            }
            .into());
        }
    }
    index.content = scratch(p, root, index.content_order.len())?;
    for &slot in &index.content_order {
        charge(p, phase, WorkKind::InlineMemberVisit, 1)?;
        push(&mut index.content, index.content_entries[slot], p, phase)?;
    }
    index.content_ranges = ranges(
        p,
        root,
        phase,
        request.fragments().len(),
        &index.content_order,
        |slot| index.content_keys[slot].container,
    )?;
    Ok(())
}

fn incidences(
    request: PaintRequest<'_>,
    p: &mut PhaseMeter<'_, '_>,
    phase: Scope,
) -> Result<(), PaintError> {
    for (slot, record) in request.fragments().iter().enumerate() {
        charge(p, phase, WorkKind::FragmentVisit, 1)?;
        let fragment = *record;
        let id = FragmentId::new(slot);
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let required = match fragment.owner() {
            FragmentOwner::Text(_) => true,
            FragmentOwner::Box(owner) => {
                let facts = request.boxes()[owner.get()];
                let paint = request.box_paint()[owner.get()];
                matches!(
                    paint.kind(),
                    BoxPaintKind::Inline | BoxPaintKind::InlineBlock | BoxPaintKind::InlineReplaced
                ) && paint.float() == FloatParticipation::None
                    && facts.participation() == Participation::Ordinary
                    && !facts.position().out_of_flow()
            }
        };
        match (required, fragment.line()) {
            (true, None) => {
                return Err(PaintInputError::MissingLineIncidence { fragment: id }.into());
            }
            (false, Some(line)) => {
                return Err(PaintInputError::UnexpectedLineIncidence { fragment: id, line }.into());
            }
            (false, None) => continue,
            (true, Some(line)) => {
                let container = request.lines()[line.get()].container();
                let valid = fragment.parent().is_some_and(|parent| {
                    if parent == container {
                        return true;
                    }
                    let record = request.fragments()[parent.get()];
                    match record.owner() {
                        FragmentOwner::Box(owner) => {
                            request.box_paint()[owner.get()].kind() == BoxPaintKind::Inline
                                && record.line() == Some(line)
                        }
                        FragmentOwner::Text(_) => false,
                    }
                });
                if !valid {
                    return Err(PaintInputError::InvalidLineIncidence { fragment: id, line }.into());
                }
            }
        }
    }
    Ok(())
}

fn primitive_target(
    request: PaintRequest<'_>,
    primitive: PrimitiveId,
    fragment: FragmentId,
) -> Result<PaintFragment, PaintError> {
    request
        .fragments()
        .get(fragment.get())
        .copied()
        .ok_or_else(|| {
            PaintInputError::MissingPrimitiveFragment {
                primitive,
                fragment,
            }
            .into()
        })
}

fn primitives(
    request: PaintRequest<'_>,
    core: &State,
    index: &mut PaintIndex,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
) -> Result<(), PaintError> {
    let n = request.primitives().len();
    index.primitives = scratch(p, root, n)?;
    index.primitive_keys = scratch(p, root, n)?;
    for (slot, record) in request.primitives().iter().enumerate() {
        charge(p, phase, WorkKind::PrimitiveVisit, 1)?;
        let primitive = *record;
        let id = PrimitiveId::new(slot);
        charge(p, phase, WorkKind::PaintValidation, 1)?;
        let (bucket, role, applicator) = match primitive.data() {
            PrimitiveData::Canvas(kind) => (
                0,
                match kind {
                    CanvasPaintKind::BackgroundColor => 0,
                    CanvasPaintKind::BackgroundImages => 1,
                },
                0,
            ),
            PrimitiveData::Fragment { fragment, kind } => {
                let record = primitive_target(request, id, fragment)?;
                let valid = match (record.owner(), kind) {
                    (FragmentOwner::Text(_), FragmentPaintKind::Text) => true,
                    (FragmentOwner::Box(owner), FragmentPaintKind::ReplacedContent) => {
                        replaced(request.box_paint()[owner.get()].kind())
                    }
                    (
                        FragmentOwner::Box(_),
                        FragmentPaintKind::BackgroundColor
                        | FragmentPaintKind::BackgroundImages
                        | FragmentPaintKind::Border
                        | FragmentPaintKind::Outline,
                    ) => true,
                    _ => false,
                };
                if !valid {
                    return Err(PaintInputError::InvalidPrimitiveTarget {
                        primitive: id,
                        fragment,
                    }
                    .into());
                }
                if record.owner() == FragmentOwner::Box(request.root())
                    && matches!(
                        kind,
                        FragmentPaintKind::BackgroundColor | FragmentPaintKind::BackgroundImages
                    )
                {
                    return Err(PaintInputError::RootFragmentBackground {
                        primitive: id,
                        fragment,
                    }
                    .into());
                }
                let role = match kind {
                    FragmentPaintKind::BackgroundColor => 0,
                    FragmentPaintKind::BackgroundImages => 1,
                    FragmentPaintKind::Border => 2,
                    FragmentPaintKind::ReplacedContent | FragmentPaintKind::Text => 5,
                    FragmentPaintKind::Outline => 7,
                };
                (fragment.get() + 1, role, 0)
            }
            PrimitiveData::Decoration {
                text_fragment: fragment,
                applying_box,
                kind,
            } => {
                let record = primitive_target(request, id, fragment)?;
                let FragmentOwner::Text(text) = record.owner() else {
                    return Err(PaintInputError::InvalidPrimitiveTarget {
                        primitive: id,
                        fragment,
                    }
                    .into());
                };
                charge(p, phase, WorkKind::PaintValidation, 1)?;
                if applying_box.get() >= request.boxes().len() {
                    return Err(PaintInputError::MissingDecorationApplicator {
                        primitive: id,
                        applying_box,
                    }
                    .into());
                }
                let mut ancestor = Some(request.texts()[text.get()].parent());
                loop {
                    charge(p, phase, WorkKind::DecorationComparison, 1)?;
                    match ancestor {
                        Some(owner) if owner == applying_box => break,
                        Some(owner) => ancestor = request.boxes()[owner.get()].parent(),
                        None => {
                            return Err(PaintInputError::InvalidDecorationApplicator {
                                primitive: id,
                                text_fragment: fragment,
                                applying_box,
                            }
                            .into());
                        }
                    }
                }
                let role = match kind {
                    DecorationKind::Underline => 3,
                    DecorationKind::Overline => 4,
                    DecorationKind::LineThrough => 6,
                };
                (
                    fragment.get() + 1,
                    role,
                    core.nodes[applying_box.get()].rank,
                )
            }
        };
        let key = PrimitiveKey {
            bucket,
            role,
            applicator,
            order: primitive.record_order(),
        };
        push(&mut index.primitive_keys, key, p, phase)?;
        push(&mut index.primitives, slot, p, phase)?;
    }
    SortBuffers::new(&mut index.primitives, &mut index.merge)?.sort(
        p,
        phase,
        WorkKind::PaintOrderComparison,
        |a, b| index.primitive_keys[a].cmp(&index.primitive_keys[b]),
    )?;
    for pair in index.primitives.windows(2) {
        charge(p, phase, WorkKind::PaintOrderComparison, 1)?;
        if index.primitive_keys[pair[0]] == index.primitive_keys[pair[1]] {
            return Err(PaintInputError::DuplicatePrimitiveOrder {
                first: PrimitiveId::new(pair[0]),
                other: PrimitiveId::new(pair[1]),
                record_order: index.primitive_keys[pair[0]].order,
            }
            .into());
        }
    }
    Ok(())
}

fn ranges(
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
    buckets: usize,
    values: &[usize],
    bucket: impl Fn(usize) -> usize,
) -> Result<Vec<Range<usize>>, PaintError> {
    let mut result = scratch(p, root, buckets)?;
    let fill = (|| {
        let mut cursor = 0;
        for current in 0..buckets {
            charge(p, phase, WorkKind::InlineMemberVisit, 1)?;
            let start = cursor;
            while cursor < values.len() {
                charge(p, phase, WorkKind::InlineMemberVisit, 1)?;
                if bucket(values[cursor]) != current {
                    break;
                }
                charge(p, phase, WorkKind::OrderMove, 1)?;
                cursor += 1;
            }
            push(&mut result, start..cursor, p, phase)?;
        }
        Ok(())
    })();
    match fill {
        Ok(()) => Ok(result),
        Err(error) => {
            release(p, root, result, ResourceKind::ScratchBytes)?;
            Err(error)
        }
    }
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
