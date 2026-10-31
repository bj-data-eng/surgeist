//! Domain publication from one canonical source transaction. Raw declarations
//! remain immutable; output content is lowered from actual global incidence.
use super::context::{self, SourceContext};
use crate::analyzed::captured::CapturedOwner;
use crate::processing::contextual::ParagraphTopology;
use crate::protocol::paragraph::{
    FlowOriginKey, OriginMap, ParagraphBoundaryDefinition, ParagraphContent, ParagraphContext,
    ParagraphData, SemanticAnchor, SemanticAnchorItem,
};
use crate::{
    AnalysisTypography, AnalyzedDomain, AnalyzedSource, CapturedJointFlow, ProcessedSource, Range,
    Result,
};
use std::sync::Arc;

fn raw_owner(context: &SourceContext<'_>, key: FlowOriginKey) -> usize {
    context
        .scope_topology
        .expect("admitted domain")
        .capture
        .events[key.captured_event().expect("captured domain original")]
    .paragraph
    .expect("original source member")
}

pub(crate) fn missing_joint(first: usize, last: usize, source: Range) -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::UnsupportedFeature,
        "indivisible source content needs actual joint output authority",
    )
    .with_detail(crate::ErrorDetail::SourceProcessing {
        reason: crate::SourceProcessingError::MissingJointAnalysisAuthority {
            first_paragraph: first,
            last_paragraph: last,
            source,
        },
    })
}

struct Lowering {
    owner: CapturedOwner,
    content: ParagraphContent,
    control_orders: Vec<usize>,
    anchors: Vec<usize>,
}
impl Lowering {
    fn new(owner: CapturedOwner) -> Result<Self> {
        Ok(Self {
            owner,
            content: ParagraphContent {
                origins: Vec::new(),
                origin_map: OriginMap::captured(Vec::new(), 0)?,
                connections: Vec::new(),
                connection_ends: Some(Vec::new()),
                connection_anchor_cuts: Some(Vec::new()),
                controls: Vec::new(),
                boundary_policies: Vec::new(),
                replacements: Vec::new(),
                anchor_order: Vec::new(),
                computed_boundaries: Vec::new(),
                coverage: Some(Vec::new()),
                envelope: None,
            },
            control_orders: Vec::new(),
            anchors: Vec::new(),
        })
    }
}

pub(crate) fn analyze(
    processed: Arc<ProcessedSource>,
    member: &ParagraphData,
    typography: AnalysisTypography,
) -> Result<AnalyzedDomain> {
    let topology = member.scope_topology.as_ref().expect("contextual member");
    let capture = Arc::clone(&topology.capture);
    let domain = topology.domain;
    let context = SourceContext::domain(&processed, member)?;
    let input = crate::projection::SourceProjection::from_processed(&processed, member)?;
    let mut canonical = super::execute::canonical(context, input)?;
    let mut lowers = Vec::new();
    let mut output_for_raw = context::filled(capture.paragraphs.len(), None)?;
    // Processed record order is the admitted event order, independently of the
    // caller's raw input IDs and of the designated used-facts member.
    for record in &processed.paragraphs {
        let Some(raw) = &record.data.scope_topology else {
            continue;
        };
        if raw.domain != domain {
            continue;
        }
        let CapturedOwner::Raw(raw) = &raw.owner else {
            unreachable!("raw producer")
        };
        if output_for_raw[raw.index].is_some() {
            continue;
        }
        let owner = if let Some(joint) = capture.paragraphs[raw.index].joint_flow {
            CapturedOwner::Joint(CapturedJointFlow {
                capture: Arc::clone(&capture),
                index: joint,
            })
        } else {
            CapturedOwner::Raw(raw.clone())
        };
        let index = lowers.len();
        for ordinal in 0..owner.len() {
            output_for_raw[owner.member(ordinal).index] = Some(index);
        }
        context::push(&mut lowers, Lowering::new(owner)?)?;
    }
    let mut raw_data = context::filled(capture.paragraphs.len(), None)?;
    for record in &processed.paragraphs {
        if let Some(topology) = &record.data.scope_topology
            && let CapturedOwner::Raw(raw) = &topology.owner
        {
            raw_data[raw.index] = Some(&record.data.context);
        }
    }
    let mut original_local = context::filled(canonical.context.origins.len(), 0)?;
    let mut keys: Vec<Vec<usize>> = context::collect_rows(lowers.iter().map(|_| Vec::new()))?;
    for (index, original) in canonical.context.origins.iter().enumerate() {
        let output =
            output_for_raw[raw_owner(&canonical.context, original.key)].expect("domain owner");
        original_local[index] = lowers[output].content.origins.len();
        context::push(&mut lowers[output].content.origins, original.data.clone())?;
        context::push(
            &mut keys[output],
            original.key.captured_event().expect("captured key"),
        )?;
    }
    let mut control_local = context::filled(canonical.context.controls.len(), 0)?;
    let mut anchor_local = context::filled(canonical.context.anchor_order.len(), 0)?;
    for (order, anchor) in canonical.context.anchor_order.iter().enumerate() {
        let raw = canonical
            .context
            .anchor_paragraph(order)
            .expect("actual raw anchor owner");
        let output = output_for_raw[raw].expect("domain anchor owner");
        let lower = &mut lowers[output];
        let item = match anchor.item {
            SemanticAnchorItem::EmptyOrigin(origin) => {
                SemanticAnchorItem::EmptyOrigin(original_local[origin])
            }
            SemanticAnchorItem::Atomic(atomic) => SemanticAnchorItem::Atomic(atomic),
            SemanticAnchorItem::Control(control) => {
                let index = lower.content.controls.len();
                control_local[control] = index;
                context::push(
                    &mut lower.content.controls,
                    canonical.context.controls[control].clone(),
                )?;
                context::push(
                    &mut lower.control_orders,
                    canonical.context.capture_order(anchor.item),
                )?;
                SemanticAnchorItem::Control(index)
            }
            SemanticAnchorItem::ComputedLineBoundary(computed) => {
                let index = lower.content.computed_boundaries.len();
                context::push(
                    &mut lower.content.computed_boundaries,
                    canonical.context.computed_boundaries[computed].clone(),
                )?;
                SemanticAnchorItem::ComputedLineBoundary(index)
            }
        };
        anchor_local[order] = lower.content.anchor_order.len();
        context::push(&mut lower.anchors, order)?;
        context::push(
            &mut lower.content.anchor_order,
            SemanticAnchor { item, ..*anchor },
        )?;
    }
    // Each connection still derives from a real supplied source edge. Filtering
    // by output membership cannot create continuity across an unknown seam.
    let mut edges = context::collect_rows(canonical.context.connections.iter().enumerate())?;
    edges.sort_by_key(|(_, edge)| edge.before);
    for (global, edge) in edges {
        let before = output_for_raw[raw_owner(
            &canonical.context,
            canonical.context.origins[edge.before].key,
        )]
        .expect("domain");
        let after = output_for_raw[raw_owner(
            &canonical.context,
            canonical.context.origins[edge.after].key,
        )]
        .expect("domain");
        if before != after {
            continue;
        }
        let lower = &mut lowers[before];
        let local = lower.content.connections.len();
        context::push(
            &mut lower.content.connections,
            crate::ParagraphConnection {
                at: edge.at,
                kind: edge.kind,
            },
        )?;
        context::push(
            lower
                .content
                .connection_ends
                .as_mut()
                .expect("declared edges"),
            canonical
                .context
                .connection_ends
                .as_ref()
                .expect("captured edges")[global],
        )?;
        let cut = canonical
            .context
            .connection_anchor_cuts
            .as_ref()
            .expect("captured cut")[global];
        let local_cut = lower.anchors.partition_point(|&order| order < cut);
        context::push(
            lower
                .content
                .connection_anchor_cuts
                .as_mut()
                .expect("captured cuts"),
            local_cut,
        )?;
        if let Some(mut policy) = capture.connections[edge.diagnostic].boundary_policy {
            policy.definition = ParagraphBoundaryDefinition::Connection(local);
            context::push(&mut lower.content.boundary_policies, policy)?;
        }
    }
    let combined = super::combined::complete(&mut canonical, &output_for_raw)?;
    let mut targets = Vec::new();
    for (output, mut lower) in lowers.into_iter().enumerate() {
        lower.content.origin_map = OriginMap::captured(
            std::mem::take(&mut keys[output]),
            lower.content.origins.len(),
        )?;
        for ordinal in 0..lower.owner.len() {
            let raw = lower.owner.member(ordinal).index;
            let data = raw_data[raw].expect("actual admitted member");
            for range in data.coverage.as_ref().expect("captured coverage") {
                context::push(
                    lower.content.coverage.as_mut().expect("captured coverage"),
                    *range,
                )?;
            }
            for replacement in &data.replacements {
                context::push(&mut lower.content.replacements, replacement.clone())?;
            }
            for policy in &data.boundary_policies {
                if !matches!(
                    policy.definition,
                    ParagraphBoundaryDefinition::Connection(_)
                ) {
                    context::push(&mut lower.content.boundary_policies, *policy)?;
                }
            }
        }
        let coverage = lower.content.coverage.as_mut().expect("captured coverage");
        coverage.sort_by_key(|range| (range.start, range.end));
        let first = raw_data[lower.owner.member(0).index].expect("first member");
        let last = raw_data[lower.owner.member(lower.owner.len() - 1).index].expect("last member");
        lower.content.envelope = Some(Range::new(
            first.extent(&processed).start,
            last.extent(&processed).end,
        ));
        let facts = raw_data[lower.owner.facts_owner().index].expect("designated original facts");
        let paragraph = ParagraphContext {
            content: lower.content,
            scope_topology: Some(ParagraphTopology {
                capture: Arc::clone(&capture),
                owner: lower.owner,
                domain,
                control_orders: lower.control_orders,
            }),
            terminal_edge: last.terminal_edge,
            base_direction: facts.base_direction,
            paragraph_strut: facts.paragraph_strut,
            tab_reference: facts.tab_reference.clone(),
            domain_style: facts.domain_style.clone(),
        };
        if typography.is_upright()
            && paragraph.base_direction == Some(crate::ParagraphBaseDirection::RightToLeft)
        {
            return Err(crate::protocol::protocol_error(
                crate::ProtocolError::MismatchedContext {
                    field: crate::ProtocolContextField::BaseDirection,
                },
            ));
        }
        let (spans, default) = crate::engine::admitted_output_styles(&processed, &paragraph)?;
        let host_projection =
            crate::projection::SourceProjection::host(processed.source(), &paragraph)?;
        let styles = crate::engine::output_style_runs(&processed, &paragraph, &spans, &default)?;
        context::push(
            &mut targets,
            super::project::ProjectTarget {
                paragraph,
                host_projection,
                styles,
            },
        )?;
    }
    let owners = context::collect_rows(targets.iter().map(|target| {
        &target
            .paragraph
            .scope_topology
            .as_ref()
            .expect("captured output")
            .owner
    }))?;
    let cut_owners = canonical.context.output_cuts(&owners)?;
    drop(owners);
    let mut scope_styles = Vec::new();
    for component in &canonical.map.components {
        if let super::relation::ComponentOwner::Virtual(index) = component.owner
            && let Some(output) = cut_owners[canonical.map.virtuals[index].capture_cut]
            && matches!(
                canonical.source_styles[component.style].owner,
                crate::engine::resolve::AnalyzedStyleOwner::Scope(_)
            )
        {
            context::push(&mut scope_styles, (output, component.style))?;
        }
    }
    scope_styles.sort_unstable();
    scope_styles.dedup();
    for (output, style) in scope_styles {
        let row = &canonical.source_styles[style];
        context::push(
            &mut targets[output].styles,
            crate::engine::resolve::AnalyzedStyleRun {
                owner: row.owner,
                style: crate::ValidatedStyle::for_processed(row.style.authored().clone())?,
            },
        )?;
    }
    let lowered = super::project::batch(
        canonical,
        targets,
        &output_for_raw,
        &control_local,
        &anchor_local,
        &cut_owners,
    )?;
    let mut outputs = Vec::new();
    for (output, lower) in lowered.into_iter().enumerate() {
        let super::project::ProjectedOutput {
            paragraph,
            host_projection,
            projection,
            transform,
            styles,
        } = lower;
        let combination = Arc::new(combined[output].bind(&transform, &processed.lexical)?);
        let analyzed = crate::engine::complete_analysis(
            Arc::clone(&processed),
            typography,
            crate::engine::CompletedPresentation {
                paragraph,
                host_projection,
                projection,
                transform,
                styles,
                combination: Some(combination),
            },
        )?;
        context::push(&mut outputs, AnalyzedSource::from_inner(analyzed))?;
    }
    Ok(AnalyzedDomain {
        processed: Arc::clone(&processed),
        domain: crate::CapturedScope {
            capture: Arc::clone(&capture),
            index: capture.domain_scopes[domain],
        },
        outputs,
    })
}
