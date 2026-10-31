//! One ownership partition of a completed canonical domain transaction.

use crate::engine::{
    OriginalAtom,
    resolve::{self, AnalyzedStyleOwner, AnalyzedStyleRun},
};
use crate::processing::lexical_inventory::{LexicalDisposition, LexicalRecordKind};
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{FlowOriginKey, ParagraphContext, SemanticAnchorItem};
use crate::transform::{
    context::{self, SourceContext, StyleRow},
    relation::{ComponentOwner, RelationEvent, TransformMap},
};
use crate::{Range, Result, SourceContinuityOperation};
use std::sync::Arc;

pub(in crate::transform) struct ProjectTarget {
    pub(in crate::transform) paragraph: ParagraphContext,
    pub(in crate::transform) host_projection: SourceProjection,
    pub(in crate::transform) styles: Vec<AnalyzedStyleRun>,
}

pub(in crate::transform) struct ProjectedOutput {
    pub(in crate::transform) paragraph: ParagraphContext,
    pub(in crate::transform) host_projection: SourceProjection,
    pub(in crate::transform) styles: Vec<AnalyzedStyleRun>,
    pub(in crate::transform) projection: SourceProjection,
    pub(in crate::transform) transform: Arc<TransformMap>,
}

#[derive(Clone, Copy)]
struct Local {
    output: usize,
    index: usize,
}

struct Builder {
    target: ProjectTarget,
    map: TransformMap,
    atoms: Vec<OriginalAtom>,
    kept_atoms: Vec<usize>,
    kept_emissions: Vec<usize>,
    scope_ids: Vec<usize>,
    phrase_ids: Vec<usize>,
    control_styles: Vec<Option<usize>>,
    scope_styles: Vec<(usize, usize)>,
    barriers: Vec<usize>,
    cuts: Vec<usize>,
    anchor_atoms: Vec<Option<usize>>,
}

impl Builder {
    fn new(target: ProjectTarget) -> Result<Self> {
        let mut control_styles = context::filled(target.paragraph.controls.len(), None)?;
        let mut scope_styles = Vec::new();
        for (index, row) in target.styles.iter().enumerate() {
            match row.owner {
                AnalyzedStyleOwner::Source(_) => (),
                AnalyzedStyleOwner::Control(control) => control_styles[control] = Some(index),
                AnalyzedStyleOwner::Scope(scope) => {
                    context::push(&mut scope_styles, (scope, index))?
                }
            }
        }
        scope_styles.sort_unstable_by_key(|&(scope, _)| scope);
        let anchor_atoms = context::filled(target.paragraph.anchor_order.len(), None)?;
        Ok(Self {
            target,
            map: TransformMap::default(),
            atoms: Vec::new(),
            kept_atoms: Vec::new(),
            kept_emissions: Vec::new(),
            scope_ids: Vec::new(),
            phrase_ids: Vec::new(),
            control_styles,
            scope_styles,
            barriers: Vec::new(),
            cuts: Vec::new(),
            anchor_atoms,
        })
    }

    fn style(&self, row: &StyleRow<'_>, source: Range, control_local: &[usize]) -> usize {
        match row.owner {
            AnalyzedStyleOwner::Source(_) => {
                resolve::source_style(&self.target.styles, source.start)
            }
            AnalyzedStyleOwner::Control(control) => self.control_styles[control_local[control]]
                .expect("prevalidated generating control style"),
            AnalyzedStyleOwner::Scope(scope) => {
                let index = self
                    .scope_styles
                    .binary_search_by_key(&scope, |&(scope, _)| scope)
                    .expect("prevalidated demanded generating scope style");
                self.scope_styles[index].1
            }
        }
    }
}

fn original_output(
    context: &SourceContext<'_>,
    key: FlowOriginKey,
    outputs: &[Option<usize>],
) -> usize {
    let order = key
        .captured_event()
        .expect("domain original has an actual captured event");
    let capture = &context
        .scope_topology
        .expect("captured domain batch")
        .capture;
    let raw = capture.events[order]
        .paragraph
        .expect("actual original paragraph");
    outputs[raw].expect("complete domain output association")
}

#[derive(Clone, Copy, Default)]
struct RawBounds {
    first: Option<usize>,
    last: Option<usize>,
}
impl RawBounds {
    fn include(&mut self, context: &SourceContext<'_>, raw: usize) {
        let capture = &context
            .scope_topology
            .expect("captured domain batch")
            .capture;
        let enter = capture.paragraphs[raw].enter;
        if self
            .first
            .is_none_or(|old| enter < capture.paragraphs[old].enter)
        {
            self.first = Some(raw);
        }
        if self
            .last
            .is_none_or(|old| enter > capture.paragraphs[old].enter)
        {
            self.last = Some(raw);
        }
    }
    fn piece(&mut self, context: &SourceContext<'_>, key: FlowOriginKey) {
        let capture = &context
            .scope_topology
            .expect("captured domain batch")
            .capture;
        let event = key.captured_event().expect("actual original capture event");
        self.include(
            context,
            capture.events[event]
                .paragraph
                .expect("actual original raw owner"),
        );
    }
    fn error(self, source: Range) -> crate::Error {
        crate::transform::domain::missing_joint(
            self.first.expect("required actual first raw member"),
            self.last.expect("required actual last raw member"),
            source,
        )
    }
}

/// Final EGCs are checked once against the actual final owner partition. A
/// domain request does not authorize a unit to straddle independent outputs.
fn check_units(
    context: &SourceContext<'_>,
    map: &TransformMap,
    projection: &SourceProjection,
    components: &[Local],
    builders: &[Builder],
    control_raw: &[Option<usize>],
    output_for_raw: &[Option<usize>],
) -> Result<()> {
    let mut missing =
        context::collect_rows(context.missing.iter().enumerate().map(|(ordinal, edge)| {
            (
                context
                    .origins
                    .partition_point(|origin| origin.aggregate.start < edge.at),
                ordinal,
                *edge,
            )
        }))?;
    missing.sort_unstable_by_key(|&(cut, ordinal, _)| (cut, ordinal));
    let mut seen = context::filled(builders.len(), usize::MAX)?;
    let mut witnesses = context::filled(builders.len(), RawBounds::default())?;
    let capture = &context
        .scope_topology
        .expect("captured domain batch")
        .capture;
    for (unit_index, unit) in super::scalar_units(projection)?.into_iter().enumerate() {
        let mut outputs = Vec::new();
        let mut first = usize::MAX;
        let mut last = 0;
        let mut original_first = usize::MAX;
        let mut original_last = 0;
        for atom in unit {
            let component_index = map.emissions[map.atoms[atom]].component;
            let component = &map.components[component_index];
            let output = components[component_index].output;
            if seen[output] != unit_index {
                seen[output] = unit_index;
                witnesses[output] = RawBounds::default();
                context::push(&mut outputs, output)?;
            }
            match component.owner {
                ComponentOwner::Original(row) => {
                    for piece in map.original_pieces(row) {
                        first = first.min(piece.aggregate.start);
                        last = last.max(piece.aggregate.end);
                        witnesses[output].piece(context, piece.origin);
                        let index = context
                            .origin_index(piece.origin)
                            .expect("actual source original");
                        original_first = original_first.min(index);
                        original_last = original_last.max(index);
                    }
                }
                ComponentOwner::Wbr(control) => {
                    first = first.min(component.source.start);
                    last = last.max(component.source.end);
                    witnesses[output]
                        .include(context, control_raw[control].expect("actual Wbr raw owner"));
                }
                ComponentOwner::Virtual(index) => {
                    first = first.min(component.source.start);
                    last = last.max(component.source.end);
                    if let Some(raw) = capture.events[map.virtuals[index].capture_cut].paragraph {
                        witnesses[output].include(context, raw);
                    }
                }
            }
        }
        if original_first != usize::MAX {
            let next = missing.partition_point(|(cut, _, _)| *cut <= original_first);
            if let Some((cut, _, edge)) = missing.get(next)
                && *cut <= original_last
            {
                return Err(edge.unavailable(SourceContinuityOperation::Segmentation));
            }
        }
        if outputs.len() > 1 {
            let mut required = RawBounds::default();
            for output in outputs {
                let witness = &mut witnesses[output];
                if witness.first.is_none() {
                    // A genuine joint-owned generated seam may have no raw
                    // occurrence. These are the admitted output's ownership
                    // bounds, not fabricated original occurrence provenance.
                    let owner = &builders[output]
                        .target
                        .paragraph
                        .scope_topology
                        .as_ref()
                        .expect("captured output target")
                        .owner;
                    let crate::analyzed::captured::CapturedOwner::Joint(_) = owner else {
                        unreachable!("only an admitted joint owns a raw-less generated cut")
                    };
                    for ordinal in 0..owner.len() {
                        let raw = owner.member(ordinal).index;
                        assert_eq!(
                            output_for_raw[raw],
                            Some(output),
                            "admitted whole-joint output association"
                        );
                        witness.include(context, raw);
                    }
                }
                required.include(
                    context,
                    witness.first.expect("actual output ownership witness"),
                );
                required.include(
                    context,
                    witness.last.expect("actual output ownership witness"),
                );
            }
            return Err(required.error(Range::new(first.min(last), last)));
        }
    }
    Ok(())
}

/// Targets and index vectors come from the driver's admitted same-capture
/// output partition. Styles are already validated for those actual owners.
/// The insertion-cut table is shared with that partition's scope-style admission.
pub(in crate::transform) fn batch(
    canonical: crate::transform::execute::Canonical<'_>,
    targets: Vec<ProjectTarget>,
    output_for_raw: &[Option<usize>],
    control_local: &[usize],
    anchor_local: &[usize],
    cut_owners: &[Option<usize>],
) -> Result<Vec<ProjectedOutput>> {
    let crate::transform::execute::Canonical {
        context,
        source_styles,
        projection,
        map,
        ..
    } = canonical;
    let context = &context;
    let mut builders = Vec::new();
    for target in targets {
        context::push(&mut builders, Builder::new(target)?)?;
    }
    let mut anchor_outputs = Vec::new();
    let mut control_outputs = context::filled(context.controls.len(), None)?;
    let mut control_raw = context::filled(context.controls.len(), None)?;
    for (order, anchor) in context.anchor_order.iter().enumerate() {
        let raw = context
            .anchor_paragraph(order)
            .expect("actual raw anchor owner");
        let output = output_for_raw[raw].expect("domain anchor output");
        context::push(&mut anchor_outputs, output)?;
        if let SemanticAnchorItem::Control(control) = anchor.item {
            control_outputs[control] = Some(output);
            control_raw[control] = Some(raw);
        }
    }

    let mut live_rows = context::filled(map.rows.len(), false)?;
    for component in &map.components {
        if let ComponentOwner::Original(row) = component.owner
            && !component.outputs.is_empty()
        {
            live_rows[row] = true;
        }
    }
    let mut rows = Vec::new();
    for (index, row) in map.rows.iter().enumerate() {
        let mut output = None;
        let mut crossing = false;
        let mut required = RawBounds::default();
        let mut first = usize::MAX;
        let mut last = 0;
        for piece in map.original_pieces(index) {
            let next = original_output(context, piece.origin, output_for_raw);
            crossing |= output.is_some_and(|old| old != next);
            output = Some(next);
            required.piece(context, piece.origin);
            first = first.min(piece.aggregate.start);
            last = last.max(piece.aggregate.end);
        }
        if crossing {
            if live_rows[index] {
                return Err(required.error(Range::new(first.min(last), last)));
            }
            // Removed mappings need actual per-owner metadata partitioning;
            // they cannot manufacture a final-unit joint requirement.
            return Err(super::refused(row.source));
        }
        let output = output.expect("original row retains genuine source pieces");
        let builder = &mut builders[output];
        let mut own = row.clone();
        let first = builder.map.pieces.len();
        for &piece in map.original_pieces(index) {
            context::push(&mut builder.map.pieces, piece)?;
        }
        own.pieces = first..builder.map.pieces.len();
        own.style = builder.style(&source_styles[row.style], own.source, control_local);
        context::push(
            &mut rows,
            Local {
                output,
                index: builder.map.rows.len(),
            },
        )?;
        context::push(&mut builder.map.rows, own)?;
    }

    let mut virtuals = Vec::new();
    for row in &map.virtuals {
        let output = cut_owners[row.capture_cut]
            .ok_or_else(|| super::refused(Range::new(row.source, row.source)))?;
        let builder = &mut builders[output];
        let mut own = row.clone();
        own.control_cut = builder
            .target
            .paragraph
            .scope_topology
            .as_ref()
            .expect("captured target")
            .control_orders
            .partition_point(|&cut| cut < row.capture_cut);
        context::push(&mut builder.scope_ids, row.scope)?;
        context::push(&mut builder.phrase_ids, row.phrase)?;
        context::push(
            &mut virtuals,
            Local {
                output,
                index: builder.map.virtuals.len(),
            },
        )?;
        context::push(&mut builder.map.virtuals, own)?;
    }
    for builder in &mut builders {
        builder.scope_ids.sort_unstable();
        builder.scope_ids.dedup();
        for &scope in &builder.scope_ids {
            context::push(&mut builder.map.scopes, map.scopes[scope].clone())?;
        }
        for row in &mut builder.map.virtuals {
            row.scope = builder
                .scope_ids
                .binary_search(&row.scope)
                .expect("actual used scope");
        }
    }
    let mut components = Vec::new();
    for component in &map.components {
        let (location, owner) = match component.owner {
            ComponentOwner::Original(row) => (rows[row], ComponentOwner::Original(rows[row].index)),
            ComponentOwner::Virtual(row) => {
                (virtuals[row], ComponentOwner::Virtual(virtuals[row].index))
            }
            ComponentOwner::Wbr(control) => (
                Local {
                    output: control_outputs[control].expect("actual generating control output"),
                    index: control_local[control],
                },
                ComponentOwner::Wbr(control_local[control]),
            ),
        };
        let builder = &mut builders[location.output];
        let mut own = component.clone();
        own.owner = owner;
        own.style = builder.style(
            &source_styles[component.style],
            component.source,
            control_local,
        );
        context::push(
            &mut components,
            Local {
                output: location.output,
                index: builder.map.components.len(),
            },
        )?;
        context::push(&mut builder.map.components, own)?;
    }
    check_units(
        context,
        &map,
        &projection,
        &components,
        &builders,
        &control_raw,
        output_for_raw,
    )?;

    let mut emissions = Vec::new();
    for (index, emission) in map.emissions.iter().enumerate() {
        let component = components[emission.component];
        let builder = &mut builders[component.output];
        let mut own = *emission;
        own.component = component.index;
        own.atom = 0;
        own.event = 0;
        context::push(
            &mut emissions,
            Local {
                output: component.output,
                index: builder.map.emissions.len(),
            },
        )?;
        context::push(&mut builder.kept_emissions, index)?;
        context::push(&mut builder.map.emissions, own)?;
    }
    for builder in &mut builders {
        for component in &mut builder.map.components {
            component.outputs = builder
                .kept_emissions
                .partition_point(|&old| old < component.outputs.start)
                ..builder
                    .kept_emissions
                    .partition_point(|&old| old < component.outputs.end);
            if let ComponentOwner::Original(row) = component.owner {
                builder.map.rows[row].outputs = component.outputs.clone();
            }
        }
    }
    let mut atom_locations = context::filled(projection.atoms.len(), None)?;
    for event in &map.events {
        match *event {
            RelationEvent::Text { atom } => {
                let emission = emissions[map.atoms[atom]];
                let builder = &mut builders[emission.output];
                let local = builder.atoms.len();
                builder.map.emissions[emission.index].atom = local;
                builder.map.emissions[emission.index].event = builder.map.events.len();
                atom_locations[atom] = Some(Local {
                    output: emission.output,
                    index: local,
                });
                context::push(&mut builder.map.atoms, emission.index)?;
                context::push(&mut builder.kept_atoms, atom)?;
                context::push(&mut builder.atoms, projection.atoms[atom].clone())?;
                context::push(&mut builder.map.events, RelationEvent::Text { atom: local })?;
            }
            RelationEvent::Removal {
                row,
                selection_part,
            } => {
                let row = rows[row];
                context::push(
                    &mut builders[row.output].map.events,
                    RelationEvent::Removal {
                        row: row.index,
                        selection_part,
                    },
                )?;
            }
            RelationEvent::GeneratedSuppression {
                component,
                selection_part,
            } => {
                let component = components[component];
                context::push(
                    &mut builders[component.output].map.events,
                    RelationEvent::GeneratedSuppression {
                        component: component.index,
                        selection_part,
                    },
                )?;
            }
            RelationEvent::Anchor { order } => context::push(
                &mut builders[anchor_outputs[order]].map.events,
                RelationEvent::Anchor {
                    order: anchor_local[order],
                },
            )?,
            RelationEvent::PhaseIDeleted {
                source,
                processed,
                lexical,
            } => {
                let (_, occurrence) = lexical
                    .indexed_occurrences(&context.processed.lexical)
                    .next()
                    .expect("executed deletion retains its real occurrence");
                let LexicalRecordKind::Character {
                    disposition: LexicalDisposition::Deleted { cause, .. },
                    ..
                } = occurrence.kind
                else {
                    unreachable!("deleted character occurrence")
                };
                let first = context
                    .origins
                    .partition_point(|origin| origin.aggregate.end <= source.start);
                for original in context.origins[first..]
                    .iter()
                    .take_while(|origin| origin.aggregate.start < source.end)
                {
                    let fragment = Range::new(
                        source.start.max(original.aggregate.start),
                        source.end.min(original.aggregate.end),
                    );
                    if fragment.is_empty() {
                        continue;
                    }
                    let output = original_output(context, original.key, output_for_raw);
                    let lexical = context.processed.lexical.deletion(fragment, cause);
                    context::push(
                        &mut builders[output].map.events,
                        RelationEvent::PhaseIDeleted {
                            source: lexical.original_range(&context.processed.lexical),
                            processed,
                            lexical,
                        },
                    )?;
                }
            }
        }
    }

    // The completed native phrase receipt is shared, never rerun or relabeled.
    // Its canonical rows identify actual original custody even across outputs.
    let mut seen_phrase = context::filled(builders.len(), None)?;
    for (receipt, phrase) in map.phrases.iter().enumerate() {
        for row in phrase.canonical_rows() {
            let output = rows[row].output;
            if seen_phrase[output] != Some(receipt) {
                context::push(&mut builders[output].phrase_ids, receipt)?;
                seen_phrase[output] = Some(receipt);
            }
        }
    }
    for builder in &mut builders {
        builder.phrase_ids.sort_unstable();
        builder.phrase_ids.dedup();
        for &receipt in &builder.phrase_ids {
            context::push(&mut builder.map.phrases, Arc::clone(&map.phrases[receipt]))?;
        }
        for row in &mut builder.map.virtuals {
            row.phrase = builder
                .phrase_ids
                .binary_search(&row.phrase)
                .expect("actual retained phrase receipt");
        }
    }

    // Source barriers also belong to genuine zero-source members and anchors.
    // Index those incidences once; numeric envelopes do not grant membership.
    let capture = &context
        .scope_topology
        .expect("captured domain batch")
        .capture;
    let mut points = Vec::new();
    for (output, builder) in builders.iter().enumerate() {
        let owner = &builder
            .target
            .paragraph
            .scope_topology
            .as_ref()
            .expect("captured target")
            .owner;
        for member in 0..owner.len() {
            let raw = capture.paragraphs[owner.member(member).index];
            context::push(&mut points, (capture.events[raw.enter].at, output))?;
            context::push(&mut points, (capture.events[raw.exit].at, output))?;
        }
    }
    for (order, anchor) in context.anchor_order.iter().enumerate() {
        context::push(
            &mut points,
            (context.anchor_position(anchor.item), anchor_outputs[order]),
        )?;
    }
    for (index, row) in map.virtuals.iter().enumerate() {
        context::push(&mut points, (row.source, virtuals[index].output))?;
    }
    points.sort_unstable();
    points.dedup();
    let mut barrier_seen = context::filled(builders.len(), None)?;
    for (index, &at) in projection.barriers.iter().enumerate() {
        let first = context
            .origins
            .partition_point(|origin| origin.aggregate.end < at);
        for original in context.origins[first..]
            .iter()
            .take_while(|origin| origin.aggregate.start <= at)
        {
            let output = original_output(context, original.key, output_for_raw);
            if barrier_seen[output] != Some(index) {
                context::push(&mut builders[output].barriers, at)?;
                barrier_seen[output] = Some(index);
            }
        }
        let first = points.partition_point(|&(source, _)| source < at);
        for &(_, output) in points[first..]
            .iter()
            .take_while(|&&(source, _)| source == at)
        {
            if barrier_seen[output] != Some(index) {
                context::push(&mut builders[output].barriers, at)?;
                barrier_seen[output] = Some(index);
            }
        }
    }
    for &cut in &projection.barrier_atoms {
        let left = cut.checked_sub(1).and_then(|atom| atom_locations[atom]);
        let right = atom_locations.get(cut).copied().flatten();
        if let Some(left) = left {
            context::push(&mut builders[left.output].cuts, left.index + 1)?;
        }
        if let Some(right) = right
            && left.is_none_or(|left| left.output != right.output)
        {
            context::push(&mut builders[right.output].cuts, right.index)?;
        }
    }
    for (order, atom) in projection.anchor_atoms.iter().enumerate() {
        if let Some(atom) = atom {
            let builder = &mut builders[anchor_outputs[order]];
            builder.anchor_atoms[anchor_local[order]] =
                Some(builder.kept_atoms.partition_point(|&old| old < *atom));
        }
    }
    let mut outputs = Vec::new();
    for mut builder in builders {
        builder.map.finish_cuts()?;
        builder.cuts.sort_unstable();
        builder.cuts.dedup();
        let end = builder.atoms.last().map_or(0, |atom| atom.presentation.end);
        let mut projection = SourceProjection::from_atoms_at_barriers(
            builder.atoms,
            end,
            builder.barriers,
            builder.cuts,
        )?;
        projection.style_rows = context::collect_rows(
            (0..projection.atoms.len())
                .map(|atom| Some(builder.map.component_for_atom(atom).style)),
        )?;
        projection.anchor_atoms = builder.anchor_atoms;
        let transform = Arc::new(builder.map);
        projection.transform = Some(Arc::clone(&transform));
        context::push(
            &mut outputs,
            ProjectedOutput {
                paragraph: builder.target.paragraph,
                host_projection: builder.target.host_projection,
                styles: builder.target.styles,
                projection,
                transform,
            },
        )?;
    }
    Ok(outputs)
}
