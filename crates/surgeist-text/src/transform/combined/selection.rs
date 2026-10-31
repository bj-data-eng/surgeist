//! Selection reads one immutable semantic inventory. Box custody is separate
//! from Unicode segmentation; a crossing unit is never split to make a cell.
use super::*;
use crate::processing::lexical_inventory::{LexicalDisposition, LexicalRecordKind, LexicalState};
use crate::transform::relation::{ComponentOwner, RelationEvent};
use crate::{SourceProcessingEvent, TextCombineUpright as Mode};

fn capture<'a>(canonical: &'a Canonical<'_>) -> &'a crate::processing::contextual::ScopeCapture {
    &canonical
        .context
        .scope_topology
        .expect("admitted domain")
        .capture
}
pub(in crate::transform) fn common(
    capture: &crate::processing::contextual::ScopeCapture,
    mut a: usize,
    mut b: usize,
) -> usize {
    if a == b {
        return a;
    }
    let depth = |scope: usize| {
        let mut current = capture.scopes[scope].parent;
        let mut count = 0;
        while let Some(parent) = current {
            count += 1;
            current = capture.scopes[parent].parent;
        }
        count
    };
    let mut ad = depth(a);
    let mut bd = depth(b);
    while ad > bd {
        a = capture.scopes[a].parent.expect("admitted ancestor");
        ad -= 1;
    }
    while bd > ad {
        b = capture.scopes[b].parent.expect("admitted ancestor");
        bd -= 1;
    }
    while a != b {
        a = capture.scopes[a].parent.expect("same domain");
        b = capture.scopes[b].parent.expect("same domain");
    }
    a
}
fn mode(canonical: &Canonical<'_>, scope: usize) -> Result<Mode> {
    capture(canonical).scopes[scope]
        .style
        .as_ref()
        .map(|style| style.text_combine_upright)
        .ok_or_else(|| refused(TextCombineError::MissingMode { scope }))
}
fn original_pieces(canonical: &Canonical<'_>, source: Range) -> Result<Vec<OriginalPiece>> {
    let originals = &canonical.context.origins;
    let first = originals.partition_point(|origin| origin.aggregate.end <= source.start);
    let mut pieces = Vec::new();
    for original in originals[first..]
        .iter()
        .take_while(|origin| origin.aggregate.start < source.end)
    {
        let start = source.start.max(original.aggregate.start);
        let end = source.end.min(original.aggregate.end);
        if start < end {
            context::push(
                &mut pieces,
                OriginalPiece {
                    origin: original.key,
                    aggregate: Range::new(start, end),
                    local: Range::new(
                        original
                            .local
                            .start
                            .checked_add(start - original.aggregate.start)
                            .ok_or_else(context::capacity)?,
                        original
                            .local
                            .start
                            .checked_add(end - original.aggregate.start)
                            .ok_or_else(context::capacity)?,
                    ),
                },
            )?;
        }
    }
    Ok(pieces)
}
fn source_scope(canonical: &Canonical<'_>, pieces: &[OriginalPiece]) -> (usize, usize) {
    let capture = capture(canonical);
    let mut scopes = pieces.iter().map(|piece| {
        let order = piece
            .origin
            .captured_event()
            .expect("captured lexical/emission input");
        (capture.events[order].before.expect("original scope"), order)
    });
    let (mut scope, order) = scopes.next().expect("nonempty actual source incidence");
    for (other, _) in scopes {
        scope = common(capture, scope, other);
    }
    (scope, order)
}

fn output_cuts(canonical: &Canonical<'_>, outputs: &[Option<usize>]) -> Result<Vec<Option<usize>>> {
    use crate::analyzed::captured::CapturedOwner;
    let topology = canonical
        .context
        .scope_topology
        .expect("captured selection");
    let capture = &topology.capture;
    let mut seen = context::filled(capture.paragraphs.len(), false)?;
    let mut owners = Vec::new();
    let mut ids = Vec::new();
    for record in &canonical.context.processed.paragraphs {
        let Some(other) = &record.data.scope_topology else {
            continue;
        };
        if other.domain != topology.domain {
            continue;
        }
        let CapturedOwner::Raw(raw) = &other.owner else {
            unreachable!("raw processing owner")
        };
        let Some(output) = outputs[raw.index] else {
            continue;
        };
        if seen[raw.index] {
            continue;
        }
        let joint = capture.paragraphs[raw.index].joint_flow.filter(|&joint| {
            (canonical.context.whole_domain || output != 0)
                && capture.joint_flows[joint]
                    .members
                    .iter()
                    .all(|member| outputs[member.paragraph] == Some(output))
        });
        let owner = joint.map_or_else(
            || CapturedOwner::Raw(raw.clone()),
            |index| {
                CapturedOwner::Joint(crate::CapturedJointFlow {
                    capture: std::sync::Arc::clone(capture),
                    index,
                })
            },
        );
        for ordinal in 0..owner.len() {
            seen[owner.member(ordinal).index] = true;
        }
        context::push(&mut owners, owner)?;
        context::push(&mut ids, output)?;
    }
    let references = context::collect_rows(owners.iter())?;
    let cuts = canonical.context.output_cuts(&references)?;
    context::collect_rows(cuts.into_iter().map(|owner| owner.map(|owner| ids[owner])))
}

fn parts(
    canonical: &Canonical<'_>,
    output_for_raw: &[Option<usize>],
    cut_outputs: &[Option<usize>],
) -> Result<Vec<Part>> {
    let mut parts = Vec::new();
    let ledger_len = match &canonical.context.processed.lexical {
        LexicalState::Executed { records, .. } => records.len(),
        LexicalState::Untransformed => 0,
    };
    let mut seen = context::filled(ledger_len, false)?;
    for event in &canonical.map.events {
        match *event {
            RelationEvent::Text { atom } => {
                let id = canonical.map.atoms[atom];
                let emission = &canonical.map.emissions[id];
                let component = &canonical.map.components[emission.component];
                let (pieces, scope, order) = match component.owner {
                    ComponentOwner::Original(row) => {
                        let pieces = context::collect_rows(
                            canonical.map.original_pieces(row).iter().copied(),
                        )?;
                        let (scope, order) = source_scope(canonical, &pieces);
                        (pieces, scope, order)
                    }
                    ComponentOwner::Wbr(control) => {
                        let item = crate::protocol::paragraph::SemanticAnchorItem::Control(control);
                        let order = canonical.context.capture_order(item);
                        (
                            Vec::new(),
                            capture(canonical).events[order]
                                .before
                                .expect("real Wbr scope"),
                            order,
                        )
                    }
                    ComponentOwner::Virtual(index) => {
                        let row = &canonical.map.virtuals[index];
                        (
                            Vec::new(),
                            canonical.map.scopes[row.scope].ordinal,
                            row.capture_cut,
                        )
                    }
                };
                context::push(
                    &mut parts,
                    Part {
                        reference: SelectionPart::Emission(id),
                        identity: emission.selection_identity,
                        normal: emission.code_point,
                        source: component.source,
                        processed: component.processed,
                        scope,
                        order,
                        output: if matches!(component.owner, ComponentOwner::Virtual(_)) {
                            cut_outputs[order]
                        } else {
                            capture(canonical).events[order]
                                .paragraph
                                .and_then(|raw| output_for_raw[raw])
                        },
                        mode: canonical.source_styles[component.style]
                            .style
                            .authored()
                            .white_space,
                        pieces,
                        emission: Some(*emission),
                    },
                )?;
            }
            RelationEvent::PhaseIDeleted { lexical, .. } => {
                for (index, record) in
                    lexical.indexed_occurrences(&canonical.context.processed.lexical)
                {
                    if seen[index] {
                        continue;
                    }
                    let LexicalRecordKind::Character {
                        source,
                        mode,
                        disposition:
                            LexicalDisposition::Deleted {
                                processed,
                                cause: crate::processing::DeletionCause::CollapsedSpace,
                            },
                        ..
                    } = record.kind
                    else {
                        continue;
                    };
                    if record.domain != canonical.context.scope_topology.expect("domain").domain {
                        continue;
                    }
                    seen[index] = true;
                    let pieces = original_pieces(canonical, source)?;
                    let (scope, order) = source_scope(canonical, &pieces);
                    context::push(
                        &mut parts,
                        Part {
                            reference: SelectionPart::CollapsedSpace(index),
                            identity: 0x20,
                            normal: 0x20,
                            source,
                            processed: Range::new(processed, processed),
                            scope,
                            order,
                            mode,
                            pieces,
                            emission: None,
                            output: capture(canonical).events[order]
                                .paragraph
                                .and_then(|raw| output_for_raw[raw]),
                        },
                    )?;
                }
            }
            _ => (),
        }
    }
    Ok(parts)
}

/// Shared pinned ICU segmentation over the actual requested scalar identities.
/// Intermediate cuts come only from genuine semantic fences selected by caller.
pub(super) fn units(
    parts: &[Part],
    ids: impl IntoIterator<Item = usize>,
    cuts: &[usize],
) -> Result<Vec<IndexRange<usize>>> {
    segment_units(ids.into_iter().map(|id| parts[id].identity), cuts)
}
pub(super) fn used_units(
    parts: &[Part],
    ids: impl IntoIterator<Item = usize>,
) -> Result<Vec<IndexRange<usize>>> {
    segment_units(ids.into_iter().map(|id| parts[id].normal), &[])
}
fn segment_units(
    values: impl IntoIterator<Item = u32>,
    cuts: &[usize],
) -> Result<Vec<IndexRange<usize>>> {
    let mut text = String::new();
    let mut offsets = Vec::new();
    for cp in values {
        context::push(&mut offsets, text.len())?;
        let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
        text.try_reserve(ch.len_utf8())
            .map_err(|_| context::capacity())?;
        text.push(ch);
    }
    context::push(&mut offsets, text.len())?;
    let mut hard = context::collect_rows(cuts.iter().map(|&cut| offsets[cut]))?;
    context::push(&mut hard, 0)?;
    context::push(&mut hard, text.len())?;
    hard.sort_unstable();
    hard.dedup();
    let mut boundaries = Vec::new();
    crate::projection::visit_grapheme_boundaries(&text, &hard, |byte| {
        let position = offsets
            .binary_search(&byte)
            .map_err(|_| context::capacity())?;
        if boundaries.last() != Some(&position) {
            context::push(&mut boundaries, position)?;
        }
        Ok(())
    })?;
    context::collect_rows(boundaries.windows(2).map(|pair| pair[0]..pair[1]))
}

fn digit(parts: &[Part], range: IndexRange<usize>) -> bool {
    digit_code_points(parts[range].iter().map(|part| part.identity))
}
pub(in crate::transform) fn digit_code_points(values: impl Iterator<Item = u32> + Clone) -> bool {
    use icu_properties::{
        CodePointMapData,
        props::{GeneralCategory, GeneralCategoryGroup},
    };
    let mut remaining = values.clone();
    let Some(first) = remaining.next() else {
        return false;
    };
    if !(0x30..=0x39).contains(&first) {
        return false;
    }
    let categories = CodePointMapData::<GeneralCategory>::new();
    remaining.all(|cp| GeneralCategoryGroup::Mark.contains(categories.get32(cp)))
        && !crate::engine::analysis::unit_properties::has_common_enclosing_mark_in(values)
}
fn predicate(mode: Mode, unit: &SelectionUnit) -> bool {
    match mode {
        Mode::None => false,
        Mode::All => true,
        Mode::Digits(_) => unit.digit,
    }
}
fn owns_part(canonical: &Canonical<'_>, part: &Part) -> bool {
    match part.reference {
        SelectionPart::Emission(id) => crate::transform::project::owns_component(
            &canonical.context,
            &canonical.map,
            canonical.map.emissions[id].component,
        ),
        SelectionPart::CollapsedSpace(_) => part.pieces.iter().any(|piece| {
            canonical
                .context
                .requested
                .origin_map
                .view_origin(piece.origin)
                .is_some()
        }),
    }
}
fn require_joint(
    canonical: &Canonical<'_>,
    parts: &[Part],
    range: IndexRange<usize>,
    outputs: &[Option<usize>],
) -> Result<()> {
    let capture = capture(canonical);
    let mut owner = None;
    let mut different = false;
    let mut unowned = false;
    let mut first = None::<(usize, usize)>;
    let mut last = None::<(usize, usize)>;
    let mut start = usize::MAX;
    let mut end = 0;
    let owned = parts[range.clone()]
        .iter()
        .any(|part| owns_part(canonical, part));
    for part in &parts[range] {
        let mut add = |order: usize| {
            if let Some(raw) = capture.events[order].paragraph {
                let output = outputs[raw].expect("domain output");
                if let Some(earlier) = owner {
                    different |= earlier != output;
                } else {
                    owner = Some(output);
                }
                if first.is_none_or(|(earlier, _)| order < earlier) {
                    first = Some((order, raw));
                }
                if last.is_none_or(|(later, _)| later < order) {
                    last = Some((order, raw));
                }
            }
        };
        add(part.order);
        for piece in &part.pieces {
            add(piece.origin.captured_event().expect("source member"));
        }
        if part.output.is_none() {
            unowned = true;
            if let Some(emission) = part.emission
                && let ComponentOwner::Virtual(index) =
                    canonical.map.components[emission.component].owner
            {
                for key in canonical.map.virtuals[index].context_origins {
                    add(key.captured_event().expect("captured generated context"));
                }
            }
        }
        start = start.min(part.source.start);
        end = end.max(part.source.end);
    }
    if different || unowned {
        if !canonical.context.whole_domain {
            return if owned {
                Err(crate::transform::project::refused(Range::new(start, end)))
            } else {
                Ok(())
            };
        }
        return Err(super::super::domain::missing_joint(
            first.expect("first actual member").1,
            last.expect("last actual member").1,
            Range::new(start, end),
        ));
    }
    Ok(())
}
fn qualify(canonical: &Canonical<'_>, left: &Part, right: &Part) -> Result<()> {
    if let Some(edge) = canonical.context.missing.iter().find(|edge| {
        let capture = capture(canonical);
        let before = capture.origin_orders[edge.before_origin];
        let after = capture.origin_orders[edge.after_origin];
        left.order <= before && after <= right.order
    }) {
        return Err(edge.unavailable(crate::SourceContinuityOperation::Combination));
    }
    Ok(())
}

pub(super) fn select(
    canonical: &Canonical<'_>,
    output_for_raw: &[Option<usize>],
) -> Result<CombinedSourcePlan> {
    let cut_outputs = output_cuts(canonical, output_for_raw)?;
    let parts = parts(canonical, output_for_raw, &cut_outputs)?;
    let capture = capture(canonical);
    let domain = canonical.context.scope_topology.expect("domain").domain;
    let mut fences = Vec::new();
    let mut boxes = Vec::new();
    let mut box_orders = Vec::new();
    let mut hard = Vec::new();
    // Project actual event prefixes through real emission order. A delayed
    // descendant may follow a later input; source-order binary search over the
    // emissions would lose that custody and is not a valid cut projection.
    let mut ends = context::filled(
        capture
            .events
            .len()
            .checked_add(1)
            .ok_or_else(context::capacity)?,
        0usize,
    )?;
    for (index, part) in parts.iter().enumerate() {
        ends[part.order + 1] = ends[part.order + 1].max(index + 1);
    }
    for index in 1..ends.len() {
        ends[index] = ends[index].max(ends[index - 1]);
    }
    for fence in canonical.context.fences()? {
        use crate::transform::context::SourceFence;
        let (order, stop) = match fence {
            SourceFence::Disconnected(index) => (
                capture.connections[canonical.context.connections[index].diagnostic].order,
                true,
            ),
            SourceFence::BlockEvent(order) => (order, true),
            SourceFence::Anchor(item) => (
                canonical.context.capture_order(item),
                matches!(
                    item,
                    crate::protocol::paragraph::SemanticAnchorItem::Atomic(_)
                ),
            ),
        };
        let cut = ends[order];
        context::push(&mut hard, cut)?;
        if stop {
            context::push(&mut fences, cut)?;
        }
    }
    for &order in &capture.flow_orders[domain] {
        if let SourceProcessingEvent::EnterScope(scope) | SourceProcessingEvent::ExitScope(scope) =
            capture.events[order].event
        {
            if capture.scopes[scope].definition.is_none() {
                context::push(&mut boxes, ends[order])?;
                context::push(&mut box_orders, order)?;
            } else if capture.scopes[scope].domain != domain {
                context::push(&mut fences, ends[order])?;
            }
        }
    }
    fences.sort_unstable();
    fences.dedup();
    boxes.sort_unstable();
    boxes.dedup();
    hard.sort_unstable();
    hard.dedup();
    let mut selected_units = Vec::new();
    for range in units(&parts, 0..parts.len(), &hard)? {
        if range.is_empty() {
            continue;
        }
        require_joint(canonical, &parts, range.clone(), output_for_raw)?;
        let mut scope = parts[range.start].scope;
        for part in &parts[range.clone()] {
            scope = common(capture, scope, part.scope);
        }
        let first_order = parts[range.clone()]
            .iter()
            .map(|part| part.order)
            .min()
            .expect("nonempty unit");
        let last_order = parts[range.clone()]
            .iter()
            .map(|part| part.order)
            .max()
            .expect("nonempty unit");
        let crosses_box = parts[range.clone()].iter().any(|part| part.scope != scope)
            || box_orders
                .get(box_orders.partition_point(|&order| order <= first_order))
                .is_some_and(|&order| order <= last_order);
        context::push(
            &mut selected_units,
            SelectionUnit {
                digit: digit(&parts, range.clone()),
                parts: range,
                scope,
                crosses_box,
            },
        )?;
    }
    let axis = capture.scopes[capture.domain_scopes[domain]].writing_axis;
    let mut groups = Vec::new();
    let mut next = 0;
    while next < selected_units.len() {
        let first = next;
        let unit = &selected_units[first];
        if axis == Some(crate::SourceWritingAxis::Horizontal) {
            next += 1;
            continue;
        }
        if canonical
            .selection_scopes
            .as_ref()
            .is_some_and(|scopes| !scopes[unit.scope])
        {
            next += 1;
            continue;
        }
        let current_mode = mode(canonical, unit.scope)?;
        if unit.crosses_box || !predicate(current_mode, unit) {
            next += 1;
            continue;
        }
        if !capture.complete[domain] {
            return Err(refused(TextCombineError::MissingScopeContext {
                scope: unit.scope,
            }));
        }
        next += 1;
        while next < selected_units.len() {
            let previous = &selected_units[next - 1];
            let following = &selected_units[next];
            let cut = previous.parts.end;
            if fences.binary_search(&cut).is_ok()
                || boxes.binary_search(&cut).is_ok()
                || following.scope != unit.scope
                || following.crosses_box
                || !predicate(current_mode, following)
            {
                break;
            }
            qualify(
                canonical,
                &parts[previous.parts.end - 1],
                &parts[following.parts.start],
            )?;
            next += 1;
        }
        let mut reject =
            matches!(current_mode, Mode::Digits(limit) if next - first > limit.max_units());
        if reject {
            continue;
        }
        for (left, right) in first
            .checked_sub(1)
            .map(|left| (left, first))
            .into_iter()
            .chain((next < selected_units.len()).then_some((next - 1, next)))
        {
            let a = &selected_units[left];
            let b = &selected_units[right];
            if fences.binary_search(&a.parts.end).is_ok() {
                continue;
            }
            let ancestor = common(capture, a.scope, b.scope);
            let hypothetical = mode(canonical, ancestor)?;
            if predicate(hypothetical, a) && predicate(hypothetical, b) {
                qualify(canonical, &parts[a.parts.end - 1], &parts[b.parts.start])?;
                reject = true;
            }
        }
        if reject {
            continue;
        }
        if axis.is_none() {
            return Err(refused(TextCombineError::MissingWritingAxis { domain }));
        }
        let group_parts = selected_units[first].parts.start..selected_units[next - 1].parts.end;
        if !canonical.context.whole_domain
            && !parts[group_parts.clone()]
                .iter()
                .any(|part| owns_part(canonical, part))
        {
            continue;
        }
        let range = selected_units[first].parts.start..selected_units[next - 1].parts.end;
        require_joint(canonical, &parts, range.clone(), output_for_raw)?;
        let inner_direction = capture.scopes[unit.scope]
            .horizontal_inner_direction
            .ok_or_else(|| {
                refused(TextCombineError::MissingInnerDirection { scope: unit.scope })
            })?;
        context::push(
            &mut groups,
            FixedGroup {
                units: first..next,
                parts: range,
                scope: unit.scope,
                parent: capture.scopes[unit.scope].parent,
                mode: current_mode,
                inner_direction,
            },
        )?;
    }
    let visible = context::collect_rows(
        parts
            .iter()
            .map(|part| matches!(part.reference, SelectionPart::Emission(_))),
    )?;
    let mut replay_stops = Vec::new();
    if let LexicalState::Executed { records, .. } = &canonical.context.processed.lexical {
        for record in records {
            if record.domain != domain {
                continue;
            }
            if let LexicalRecordKind::Barrier {
                origin:
                    crate::processing::lexical_inventory::LexicalBarrierOrigin::CapturedEvent(order),
            } = record.kind
            {
                let suppressible = match capture.events[order].event {
                    SourceProcessingEvent::Control { control, .. } => {
                        capture.controls[control].fact.kind
                            == crate::ParagraphControlKind::ForcedBreak
                    }
                    _ => false,
                };
                context::push(&mut replay_stops, (ends[order], suppressible))?;
            }
        }
    }
    replay_stops.sort_by_key(|stop| stop.0);
    let mut replay_unknown = Vec::new();
    for edge in &canonical.context.missing {
        let order = capture.origin_orders[edge.after_origin];
        context::push(
            &mut replay_unknown,
            (ends[order], edge.before_origin, edge.after_origin),
        )?;
    }
    replay_unknown.sort_by_key(|edge| edge.0);
    let retained = context::filled(parts.len(), true)?;
    let mut group_for_part = context::filled(parts.len(), None)?;
    for (index, group) in groups.iter().enumerate() {
        for part in group.parts.clone() {
            group_for_part[part] = Some(index);
        }
    }
    let lexical_count = match &canonical.context.processed.lexical {
        LexicalState::Executed { records, .. } => records.len(),
        LexicalState::Untransformed => 0,
    };
    let mut lexical_parts = context::filled(lexical_count, None)?;
    for (index, part) in parts.iter().enumerate() {
        if let SelectionPart::CollapsedSpace(record) = part.reference {
            lexical_parts[record] = Some(index);
        }
    }
    Ok(CombinedSourcePlan {
        parts,
        units: selected_units,
        groups,
        used: Vec::new(),
        fences,
        semantic_fences: hard,
        visible,
        retained,
        replay_stops,
        replay_unknown,
        group_for_part,
        lexical_parts,
    })
}
