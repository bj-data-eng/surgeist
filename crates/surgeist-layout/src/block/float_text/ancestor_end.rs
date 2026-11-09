//! Source-ordered, immediately closing physical edges for one native float.

use super::{FloatEvent, cell_origin_incidence};
use crate::text_layout::AdmittedRun;
use crate::text_layout::fit::FitError;
use crate::text_layout::scope_alignment::{EntryBacking, EntryFundingRef};
use crate::{LayoutScalar, block::floats::AcceptedFloatPrefix};
use surgeist_text as text;

struct ClosingEnd {
    occurrence: text::PreparedControl,
    allocation: text::InlineBoundaryAllocation,
}

pub(super) struct ClosingEnds {
    rows: Vec<ClosingEnd>,
    backing: EntryBacking,
}

pub(super) struct ClosingDemand {
    pub extra: f64,
    pub participates: bool,
}

fn visit(funding: Option<EntryFundingRef<'_>>) -> Result<(), FitError> {
    if let Some(funding) = funding {
        funding.visit(1)?;
    }
    Ok(())
}

fn find_row<'a, T>(
    rows: &'a [T],
    funding: Option<EntryFundingRef<'_>>,
    predicate: impl Fn(&T) -> bool,
) -> Result<Option<&'a T>, FitError> {
    for row in rows {
        visit(funding)?;
        if predicate(row) {
            return Ok(Some(row));
        }
    }
    Ok(None)
}

pub(in crate::block) fn source_interval<Node: Copy + Eq>(
    run: &AdmittedRun<'_, Node>,
    associations: &crate::text_layout::capture::ScopeAssociations<Node>,
    context: &crate::inline_wrapping::WrappingContext<Node>,
    graph: &crate::engine::validation::ValidationGraph<Node>,
    owner: &text::InlineBoxSource,
    funding: Option<EntryFundingRef<'_>>,
) -> Result<Option<(usize, usize)>, FitError> {
    let inspect = || {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        Ok::<_, text::Error>(())
    };
    let mut pair = context.text_box_source_pair(
        run.binding.container,
        owner,
        &run.binding.prepared,
        inspect,
    )?;
    if pair.is_none()
        && let Some(scope) = owner.captured_scope()
    {
        for (actual, actual_pair) in associations.native_pairs() {
            inspect()?;
            if actual == scope {
                pair = Some(actual_pair);
                break;
            }
        }
    }
    let Some(pair) = pair else {
        return Ok(None);
    };
    let projected = context.native_pair_projection_with(
        graph,
        run.binding.container,
        run.nodes(),
        pair,
        || inspect().map_err(FitError::from),
    )?;
    let position = |endpoint, after| -> Result<usize, FitError> {
        match endpoint {
            crate::inline_wrapping::ScopeEndpoint::BeforeRun => Ok(0),
            crate::inline_wrapping::ScopeEndpoint::AfterRun => Ok(run.nodes().len()),
            crate::inline_wrapping::ScopeEndpoint::Local(node) => {
                for (index, &actual) in run.nodes().iter().enumerate() {
                    inspect()?;
                    if actual == node {
                        return Ok(index + usize::from(after));
                    }
                }
                Err(crate::WrappingScopeError::UnsupportedIncidence.into())
            }
        }
    };
    Ok(Some((
        position(projected.start, true)?,
        position(projected.end, false)?,
    )))
}

fn owner(
    control: &text::PreparedControl,
    allocation: Option<&text::InlineBoundaryAllocation>,
) -> Option<(text::InlineBoxSource, text::InlineBoxSource)> {
    if let Some(context) = allocation.and_then(|allocation| allocation.box_context()) {
        return Some((context.owner().clone(), context.parent().clone()));
    }
    let captured = control.captured_control()?;
    let scope = captured.scope();
    (scope.exit_control().as_ref() == Some(&captured)).then_some(())?;
    let parent = scope.parent()?;
    Some((
        text::InlineBoxSource::captured(scope),
        text::InlineBoxSource::captured(parent),
    ))
}

impl ClosingEnds {
    pub(super) fn project<'a, Node: Copy + Eq, S: LayoutScalar>(
        source: &FloatEvent<Node, S>,
        run: &AdmittedRun<'_, Node>,
        associations: &crate::text_layout::capture::ScopeAssociations<Node>,
        controls: impl Iterator<Item = &'a text::ControlAllocation> + Clone,
        mut interval: impl FnMut(&text::InlineBoxSource) -> Result<Option<(usize, usize)>, FitError>,
        funding: Option<EntryFundingRef<'_>>,
    ) -> Result<Self, FitError> {
        let mut result = Self {
            rows: Vec::new(),
            backing: EntryBacking::default(),
        };
        let mut physical_end_requested = false;
        for row in controls.clone() {
            visit(funding)?;
            if matches!(
                row.occurrence.view().kind(),
                text::ParagraphControlKind::InlineBoundaryEnd
            ) && row
                .boundary
                .as_ref()
                .is_some_and(|boundary| boundary.box_context().is_some())
            {
                physical_end_requested = true;
                break;
            }
        }
        if !physical_end_requested {
            // Complete actual allocations establish that this invocation has
            // no physical closing-edge demand. Generic contributions do not
            // acquire wrapping geometry merely by retaining a source scope.
            return Ok(result);
        }
        let allocation = |control: &text::PreparedControl| -> Result<Option<&'a text::InlineBoundaryAllocation>, FitError> {
            for row in controls.clone() {
                visit(funding)?;
                if row.occurrence == *control {
                    return Ok(row.boundary.as_ref());
                }
            }
            Ok(None)
        };
        // Find the actual innermost source owner open at this native cut. A
        // zero generic contribution may close a source owner, but never lends
        // physical-edge authority to its own metrics or advance.
        let mut open = None;
        let mut inner = None;
        for row in &run.binding.controls {
            visit(funding)?;
            if !matches!(
                row.occurrence.view().kind(),
                text::ParagraphControlKind::InlineBoundaryEnd
            ) {
                continue;
            }
            let actual = allocation(&row.occurrence)?;
            let Some((owner, _)) = owner(&row.occurrence, actual) else {
                continue;
            };
            let Some((start, end)) = interval(&owner)? else {
                if actual.is_some_and(|allocation| allocation.box_context().is_some()) {
                    return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
                }
                continue;
            };
            if start <= source.native_offset
                && source.native_offset < end
                && inner.is_none_or(|(a, b)| a <= start && end <= b)
            {
                inner = Some((start, end));
                open = Some(owner);
            }
        }
        for (index, &node) in run
            .nodes()
            .iter()
            .enumerate()
            .skip(source.native_offset + 1)
        {
            visit(funding)?;
            for (rank, participant) in run.binding.prepared.participants().enumerate() {
                visit(funding)?;
                if rank < run.segment.start().view().participant_offset()
                    || run.segment.end().view().participant_offset() <= rank
                {
                    continue;
                }
                if let text::PreparedParticipant::ComputedLineBoundary(boundary) = participant {
                    let cut = associations
                        .computed_cut(&boundary)
                        .ok_or(FitError::MissingComputedBoundaryIncidence)?;
                    let Some(cut) = cut.relative_to(run.inventory.clone()) else {
                        // The issued cut is proved wholly outside this native
                        // window, not missing receiving authority.
                        continue;
                    };
                    if cut.side_of_node(source.native_offset) == Some(std::cmp::Ordering::Less)
                        && cut.side_of_node(index) == Some(std::cmp::Ordering::Greater)
                    {
                        return Ok(result);
                    }
                }
            }
            if let Some(row) = find_row(&run.binding.origins, funding, |row| row.node == node)? {
                for participant in run.binding.prepared.participants() {
                    visit(funding)?;
                    match participant {
                        text::PreparedParticipant::Text(unit) => {
                            let view = unit.view();
                            if view.empty_origin().as_ref() == Some(&row.origin) {
                                continue;
                            }
                            let mut member = false;
                            for piece in view.origins() {
                                visit(funding)?;
                                member |= piece.origin() == row.origin;
                            }
                            if member
                                && (!view.processed_range().is_empty()
                                    || view.source_range().is_empty())
                            {
                                return Ok(result);
                            }
                        }
                        text::PreparedParticipant::Cell(cell)
                            if cell_origin_incidence(
                                &cell,
                                &row.origin,
                                row.origin.view().local_range().start,
                                funding,
                            )?
                            .is_some() =>
                        {
                            return Ok(result);
                        }
                        _ => {}
                    }
                }
                continue;
            }
            if let Some(row) = find_row(&run.binding.atomics, funding, |row| row.node == node)? {
                if row.occurrence.view().original_box().kind() == text::InlineBoxKind::InFlow {
                    return Ok(result);
                }
                continue;
            }
            let Some(row) = find_row(&run.binding.controls, funding, |row| row.node == node)?
            else {
                // Native out-of-flow occurrences are not Text participants.
                continue;
            };
            if !run
                .binding
                .prepared
                .boundary_consumes_control(&run.segment.end(), &row.occurrence)?
            {
                return Ok(result);
            }
            if Self::virtual_before(source, run, &row.occurrence, funding)? {
                return Ok(result);
            }
            use text::ParagraphControlKind as K;
            match row.occurrence.view().kind() {
                K::InlineBoundaryEnd => {
                    let actual = allocation(&row.occurrence)?;
                    let Some((owner, parent)) = owner(&row.occurrence, actual) else {
                        continue;
                    };
                    if open.as_ref() != Some(&owner) {
                        return Ok(result);
                    }
                    if let Some(actual) =
                        actual.filter(|allocation| allocation.box_context().is_some())
                    {
                        result.backing.reserve_push_ref(&mut result.rows, funding)?;
                        result.rows.push(ClosingEnd {
                            occurrence: row.occurrence.clone(),
                            allocation: actual.clone(),
                        });
                    }
                    open = Some(parent);
                }
                K::LeftToRightEmbedding
                | K::RightToLeftEmbedding
                | K::LeftToRightOverride
                | K::RightToLeftOverride
                | K::PopDirectionalFormat
                | K::LeftToRightIsolate
                | K::RightToLeftIsolate
                | K::FirstStrongIsolate
                | K::PopDirectionalIsolate => {}
                _ => return Ok(result),
            }
        }
        Ok(result)
    }

    fn virtual_before<Node: Copy + Eq, S: LayoutScalar>(
        source: &FloatEvent<Node, S>,
        run: &AdmittedRun<'_, Node>,
        end: &text::PreparedControl,
        funding: Option<EntryFundingRef<'_>>,
    ) -> Result<bool, FitError> {
        for separator in run.binding.prepared.virtual_separators() {
            visit(funding)?;
            let view = separator.view();
            let mut before_end = false;
            for following in view.following_controls() {
                visit(funding)?;
                before_end |= following == *end;
            }
            if !before_end {
                continue;
            }
            for unit in view.units() {
                visit(funding)?;
                for (rank, participant) in run.binding.prepared.participants().enumerate() {
                    visit(funding)?;
                    if participant == text::PreparedParticipant::Text(unit.clone())
                        && (source.position <= rank || source.inside && source.position == rank)
                    {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub(super) fn demand(
        &self,
        line: &text::FinalizedLine,
        observation: &AcceptedFloatPrefix,
        funding: Option<EntryFundingRef<'_>>,
    ) -> Result<ClosingDemand, FitError> {
        if self.rows.is_empty() {
            return Ok(ClosingDemand {
                extra: 0.0,
                participates: false,
            });
        }
        let selected = line.selected_line();
        let request = selected.request();
        let prefix = match observation {
            AcceptedFloatPrefix::Seam(seam) => Some(line.prefix_at(seam)?),
            AcceptedFloatPrefix::EmptyParagraphEnd { .. } => None,
        };
        let mut result = ClosingDemand {
            extra: 0.0,
            participates: false,
        };
        for row in &self.rows {
            visit(funding)?;
            let mut actual = &row.allocation;
            let mut in_line = false;
            for allocation in &request.controls {
                visit(funding)?;
                if allocation.occurrence == row.occurrence {
                    if let Some(boundary) = &allocation.boundary {
                        actual = boundary;
                    }
                    in_line = true;
                    break;
                }
            }
            if !in_line {
                for allocation in &request.boundary_context {
                    visit(funding)?;
                    if allocation.occurrence == row.occurrence {
                        actual = &allocation.boundary;
                        break;
                    }
                }
            }
            result.participates |= actual.participates();
            let mut charged = false;
            if in_line
                && let Some(prefix_group) =
                    prefix.as_ref().and_then(|prefix| prefix.interior_unit())
            {
                visit(funding)?;
                let before = line.prefix_at(&text::TextSeam::BeforeParticipant(
                    text::PreparedParticipant::Control(row.occurrence.clone()),
                ))?;
                charged = before.interior_unit().is_some_and(|end_group| {
                    let range = prefix_group.presentation_range();
                    range.start() < range.end() && end_group.presentation_range() == range
                });
            }
            if !charged {
                result.extra = crate::text_layout::run::add(result.extra, actual.advance())?;
            }
        }
        Ok(result)
    }
}
