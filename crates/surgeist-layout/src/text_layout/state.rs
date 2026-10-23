//! Off-side Text progress retained by the existing Layout computation session.

use super::TextParagraphBinding;
use crate::{LayoutScalar, NodeOutputOf, Point};
use surgeist_text::{LinePlacement, PlacementCoverage, PlacementRequest};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalFrame<Node, S: LayoutScalar> {
    pub node: Node,
    pub origin: Point<f64>,
    pub output: NodeOutputOf<S>,
}

#[derive(Clone)]
pub(crate) struct PendingParagraph<Node> {
    pub container: Node,
    pub nodes: Vec<Node>,
    pub prepared: surgeist_text::PreparedText,
    pub lines: Vec<LinePlacement>,
    pub bands: Vec<super::fit::LineBands>,
    pub flows: Vec<super::fit::LineFlow>,
    pub amounts: Vec<Vec<f64>>,
    pub encounters: Vec<Vec<super::fit::EncounterBasis>>,
    pub zones: Vec<super::fit::ZoneBasis>,
    pub outflows: Vec<(usize, surgeist_text::PreparedAtomic, Node)>,
    pub indent: crate::text_indent::ResolvedTextIndent<f64>,
    pub indent_start: crate::text_indent::IndentLineState,
    pub indent_end: crate::text_indent::IndentLineState,
    pub hyphenation_start: crate::hyphenation::LineHyphenationState,
    pub hyphenation_end: crate::hyphenation::LineHyphenationState,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LineBasis {
    zone: super::fit::ZoneBasis,
    encounters: Vec<super::fit::EncounterBasis>,
    request: surgeist_text::LineSelectionRequest,
    amounts: Vec<f64>,
    anchor: surgeist_text::LineAnchor,
    atomics: Vec<surgeist_text::AtomicPlacement>,
    bands: super::fit::LineBands,
    flow: super::fit::LineFlow,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PassBasis<Node, S: LayoutScalar> {
    pub frames: Vec<CanonicalFrame<Node, S>>,
    pub lines: Vec<LineBasis>,
    pub absolute_baselines: Vec<(Node, f64)>,
    pub hyphenation: Vec<(
        Node,
        crate::hyphenation::LineHyphenationState,
        crate::hyphenation::LineHyphenationState,
    )>,
    pub indentation: Vec<(
        Node,
        crate::text_indent::ResolvedTextIndent<f64>,
        crate::text_indent::IndentLineState,
        crate::text_indent::IndentLineState,
    )>,
}

/// This invocation-local state is part of the ordinary unpublished session.
/// An initial measurement pass supplies no canonical-frame proof. Later passes
/// use actual preceding canonical outputs and must reproduce their full basis
/// before any placed Text or completed batch can escape.
pub(crate) struct TextComputation<'input, Node, S: LayoutScalar> {
    pub paragraphs: &'input [TextParagraphBinding<Node>],
    pub frames: std::sync::Arc<[CanonicalFrame<Node, S>]>,
    pub pending: Vec<PendingParagraph<Node>>,
    pub reusable: std::sync::Arc<[super::fit::ReusableLine]>,
    pub atomic_origins: Vec<(Node, Point<f64>)>,
    pub absolute_baselines: Vec<(Node, f64)>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> TextComputation<'input, Node, S> {
    pub fn new(paragraphs: &'input [TextParagraphBinding<Node>]) -> Self {
        Self {
            paragraphs,
            frames: std::sync::Arc::from([]),
            pending: Vec::new(),
            reusable: std::sync::Arc::from([]),
            atomic_origins: Vec::new(),
            absolute_baselines: Vec::new(),
        }
    }

    pub fn paragraph(
        &self,
        container: Node,
        nodes: &[Node],
    ) -> Option<&'input TextParagraphBinding<Node>> {
        self.paragraphs
            .iter()
            .find(|binding| binding.container == container && binding.nodes == nodes)
    }

    pub fn paragraph_starting(
        &self,
        container: Node,
        nodes: &[Node],
    ) -> Option<&'input TextParagraphBinding<Node>> {
        self.paragraphs
            .iter()
            .find(|binding| binding.container == container && nodes.starts_with(&binding.nodes))
    }

    pub fn frame(&self, node: Node) -> Option<CanonicalFrame<Node, S>> {
        self.frames.iter().find(|frame| frame.node == node).copied()
    }

    pub fn record(&mut self, paragraph: PendingParagraph<Node>) {
        if let Some(old) = self
            .pending
            .iter_mut()
            .find(|old| old.container == paragraph.container && old.nodes == paragraph.nodes)
        {
            *old = paragraph;
        } else {
            self.pending.push(paragraph);
        }
    }

    pub fn record_atomic_origins(&mut self, origins: Vec<(Node, Point<f64>)>) {
        for (node, origin) in origins {
            if let Some((_, current)) = self
                .atomic_origins
                .iter_mut()
                .find(|(subject, _)| *subject == node)
            {
                *current = origin;
            } else {
                self.atomic_origins.push((node, origin));
            }
        }
    }

    pub fn record_absolute_baseline(&mut self, node: Node, baseline: f64) {
        if let Some((_, current)) = self
            .absolute_baselines
            .iter_mut()
            .find(|(subject, _)| *subject == node)
        {
            *current = baseline;
        } else {
            self.absolute_baselines.push((node, baseline));
        }
    }

    pub fn basis(&self, frames: Vec<CanonicalFrame<Node, S>>) -> PassBasis<Node, S> {
        PassBasis {
            frames,
            lines: self
                .pending
                .iter()
                .flat_map(|paragraph| {
                    paragraph
                        .lines
                        .iter()
                        .zip(&paragraph.bands)
                        .zip(&paragraph.flows)
                        .zip(&paragraph.amounts)
                        .zip(&paragraph.encounters)
                        .zip(&paragraph.zones)
                })
                .map(
                    |(((((placement, band), flow), amounts), encounters), zone)| LineBasis {
                        zone: zone.clone(),
                        encounters: encounters.clone(),
                        amounts: amounts.clone(),
                        request: placement.line.selected_line().request().clone(),
                        anchor: placement.anchor,
                        atomics: placement.atomics.clone(),
                        bands: *band,
                        flow: *flow,
                    },
                )
                .collect(),
            absolute_baselines: self.absolute_baselines.clone(),
            hyphenation: self
                .pending
                .iter()
                .map(|paragraph| {
                    (
                        paragraph.container,
                        paragraph.hyphenation_start,
                        paragraph.hyphenation_end,
                    )
                })
                .collect(),
            indentation: self
                .pending
                .iter()
                .map(|paragraph| {
                    (
                        paragraph.container,
                        paragraph.indent,
                        paragraph.indent_start,
                        paragraph.indent_end,
                    )
                })
                .collect(),
        }
    }

    pub fn finish<M>(
        &mut self,
    ) -> crate::LayoutResultOf<Node, Vec<crate::PlacedTextOutput<Node>>, S, M> {
        let frames = &self.frames;
        let baselines = &self.absolute_baselines;
        self.pending
            .drain(..)
            .map(|mut paragraph| {
                for (line_index, occurrence, node) in paragraph.outflows {
                    let frame =
                        frames
                            .iter()
                            .find(|frame| frame.node == node)
                            .ok_or_else(|| {
                                crate::LayoutErrorOf::new(
                                    crate::LayoutErrorSiteOf::Node(node),
                                    crate::LayoutOperation::TextPlacement,
                                    crate::LayoutErrorKindOf::MissingContext(
                                        crate::LayoutMissingContext::TextOutOfFlowGeometry,
                                    ),
                                )
                            })?;
                    let baseline = baselines
                        .iter()
                        .find(|(subject, _)| *subject == node)
                        .map(|(_, value)| *value)
                        .ok_or_else(|| {
                            crate::LayoutErrorOf::new(
                                crate::LayoutErrorSiteOf::Node(node),
                                crate::LayoutOperation::TextPlacement,
                                crate::LayoutErrorKindOf::MissingContext(
                                    crate::LayoutMissingContext::TextOutOfFlowGeometry,
                                ),
                            )
                        })?;
                    let content = frame
                        .output
                        .scroll_geometry
                        .map(|geometry| geometry.content_box())
                        .ok_or_else(|| {
                            crate::LayoutErrorOf::new(
                                crate::LayoutErrorSiteOf::Node(node),
                                crate::LayoutOperation::TextPlacement,
                                crate::LayoutErrorKindOf::MissingContext(
                                    crate::LayoutMissingContext::TextOutOfFlowGeometry,
                                ),
                            )
                        })?;
                    let border = surgeist_text::Rect::new(
                        frame.origin.x,
                        frame.origin.y,
                        frame.output.size.width.to_f64(),
                        frame.output.size.height.to_f64(),
                    );
                    let content = surgeist_text::Rect::new(
                        placement_add(node, frame.origin.x, content.origin().x.to_f64())?,
                        placement_add(node, frame.origin.y, content.origin().y.to_f64())?,
                        content.size().width.to_f64(),
                        content.size().height.to_f64(),
                    );
                    let baseline = placement_add(node, frame.origin.y, baseline)?;
                    let baseline = (baseline + 0.5).floor();
                    paragraph.lines[line_index]
                        .atomics
                        .push(surgeist_text::AtomicPlacement {
                            occurrence,
                            border,
                            content,
                            baseline,
                        });
                }
                let layout = surgeist_text::Layout::from_finalized_lines(PlacementRequest {
                    prepared: paragraph.prepared,
                    coverage: PlacementCoverage::WholeParagraph,
                    lines: paragraph.lines,
                })
                .map_err(|error| {
                    crate::LayoutErrorOf::new(
                        crate::LayoutErrorSiteOf::Node(paragraph.container),
                        crate::LayoutOperation::TextPlacement,
                        crate::LayoutErrorKindOf::Text(error),
                    )
                })?;
                Ok(crate::PlacedTextOutput::new(
                    paragraph.container,
                    paragraph.nodes,
                    layout,
                ))
            })
            .collect()
    }
}

fn placement_add<Node, S: LayoutScalar, M>(
    node: Node,
    base: f64,
    delta: f64,
) -> crate::LayoutResultOf<Node, f64, S, M> {
    let value = base + delta;
    let reason = if !value.is_finite() {
        Some(crate::LayoutTextNumericError::NonFinite)
    } else if delta != 0.0 && value == base {
        Some(crate::LayoutTextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::TextPlacement,
            crate::LayoutErrorKindOf::TextNumeric {
                value: if value.is_finite() { delta } else { value },
                reason,
            },
        ));
    }
    Ok(value)
}
