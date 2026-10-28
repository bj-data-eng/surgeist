//! Actual registered-font preparation for Layout's owning tests.
//!
//! These helpers construct source/style facts and retain Text-issued handles.
//! They do not accept shaped widths, glyph packets, levels or break receipts.

use surgeist_text::{
    Affinity, AnalyzedSource, DocumentSegmentBreaks, Font, FontCatalogueMode, Id, InlineBox,
    InlineBoxKind, LineHeight, ParagraphAnchor, ParagraphAnchorItem, ParagraphBaseDirection,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnection,
    ParagraphConnectionKind, ParagraphControl, ParagraphControlKind, ParagraphInput,
    ParagraphOrigin, ParagraphOriginKind, PreparationOptions, PreparedText, Source, Style, System,
    SystemOptions,
};

use crate::{
    LayoutInputOf, LayoutScalar, TextAtomicBinding, TextControlBinding, TextOriginBinding,
    TextParagraphBinding, TextParticipation,
};

const AHEM: &[u8] = include_bytes!("../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

/// Licensed Ahem has a one-em advance and ascender/descender 0.8em/0.2em.
/// The absolute line height is supplied explicitly, independently of its size.
pub(crate) fn ahem_style(size: f32, line_height: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size,
        line_height: LineHeight::Absolute(line_height),
        ..Style::default()
    }
}

/// This fixture explicitly authors a flat containing wrapping context. Text
/// derives the actual survivor pairs; the fixture supplies only missing NCA
/// facts and preserves any exact relation policy already captured by the input.
pub(crate) fn in_flat_wrapping_context(
    analyzed: AnalyzedSource,
    ancestor_wrap: surgeist_text::Wrap,
) -> AnalyzedSource {
    let policies = analyzed
        .wrapping_relations()
        .filter(|view| view.ancestor_wrap().is_none())
        .map(|view| surgeist_text::WrappingPolicy {
            relation: view.handle(),
            ancestor_wrap,
        })
        .collect();
    analyzed
        .with_wrapping_policies(policies)
        .expect("fixture supplies its actual flat ancestor wrapping context")
}

/// One actual original tree occurrence in an authored test paragraph.
pub(crate) enum ParagraphPart {
    Text(Box<ParagraphText>),
    Atomic {
        node: u32,
        size: surgeist_text::Size,
    },
    OutOfFlow {
        node: u32,
        size: surgeist_text::Size,
    },
    Control {
        node: u32,
        kind: ParagraphControlKind,
    },
}

pub(crate) struct ParagraphText {
    node: u32,
    source: Source,
    style: Style,
}

impl ParagraphPart {
    pub(crate) fn text(node: u32, source: Source, style: Style) -> Self {
        Self::Text(Box::new(ParagraphText {
            node,
            source,
            style,
        }))
    }
}

/// The sole actual preparation and its tree associations, independent of System.
#[derive(Clone, Debug)]
pub(crate) struct PreparedParagraph {
    pub(crate) prepared: PreparedText,
    nodes: Vec<u32>,
    origins: Vec<TextOriginBinding<u32>>,
    atomics: Vec<TextAtomicBinding<u32>>,
    controls: Vec<TextControlBinding<u32>>,
}

struct ParagraphFacts<'font> {
    strut: surgeist_text::ParagraphStrut,
    terminal: surgeist_text::ParagraphTerminalEdge,
    font: &'font [u8],
}

impl PreparedParagraph {
    /// Explicitly authors the fixture's actual boundary (or truthful absence).
    /// The default constructors below author a complete flat block instead.
    pub(crate) fn at_boundary(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        terminal: surgeist_text::ParagraphTerminalEdge,
    ) -> Self {
        Self::with_preparation(
            parts,
            base_style,
            direction,
            ParagraphFacts {
                strut: surgeist_text::ParagraphStrut::DefaultFont,
                terminal,
                font: AHEM,
            },
            Vec::new(),
            PreparationOptions::default(),
            None,
        )
    }
    pub(crate) fn plain(node: u32, source: Source, style: Style) -> Self {
        Self::new(
            vec![ParagraphPart::text(node, source, style.clone())],
            style,
            ParagraphBaseDirection::LeftToRight,
        )
    }

    pub(crate) fn new(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
    ) -> Self {
        Self::with_replacements(parts, base_style, direction, Vec::new())
    }

    pub(crate) fn with_replacements(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        replacements: Vec<surgeist_text::ParagraphReplacement>,
    ) -> Self {
        Self::with_strut(
            parts,
            base_style,
            direction,
            surgeist_text::ParagraphStrut::DefaultFont,
            replacements,
        )
    }

    /// Supplies the actual paragraph strut policy without changing font metrics.
    pub(crate) fn with_strut(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        paragraph_strut: surgeist_text::ParagraphStrut,
        replacements: Vec<surgeist_text::ParagraphReplacement>,
    ) -> Self {
        Self::with_strut_at_boundary(
            parts,
            base_style,
            direction,
            paragraph_strut,
            replacements,
            surgeist_text::ParagraphTerminalEdge::BlockEnd,
        )
    }

    /// Authors both actual strut participation and the containing end role.
    pub(crate) fn with_strut_at_boundary(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        paragraph_strut: surgeist_text::ParagraphStrut,
        replacements: Vec<surgeist_text::ParagraphReplacement>,
        terminal: surgeist_text::ParagraphTerminalEdge,
    ) -> Self {
        Self::with_preparation(
            parts,
            base_style,
            direction,
            ParagraphFacts {
                strut: paragraph_strut,
                terminal,
                font: AHEM,
            },
            replacements,
            PreparationOptions::default(),
            None,
        )
    }

    /// Authors the fixture's real flat block reference and CSS metric demand.
    pub(crate) fn with_css_tabs(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        block_direction: ParagraphBaseDirection,
    ) -> Self {
        let reference =
            surgeist_text::ParagraphTabReference::new(base_style.clone(), block_direction).unwrap();
        Self::with_preparation(
            parts,
            base_style,
            direction,
            ParagraphFacts {
                strut: surgeist_text::ParagraphStrut::DefaultFont,
                terminal: surgeist_text::ParagraphTerminalEdge::BlockEnd,
                font: AHEM,
            },
            Vec::new(),
            PreparationOptions::try_new(1.0, surgeist_text::TabPreparation::CssStops).unwrap(),
            Some(reference),
        )
    }

    /// Selects a qualified registered test face without changing preparation.
    pub(crate) fn with_registered_font(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        font: &[u8],
    ) -> Self {
        Self::with_preparation(
            parts,
            base_style,
            direction,
            ParagraphFacts {
                strut: surgeist_text::ParagraphStrut::DefaultFont,
                terminal: surgeist_text::ParagraphTerminalEdge::BlockEnd,
                font,
            },
            Vec::new(),
            PreparationOptions::default(),
            None,
        )
    }

    fn with_preparation(
        parts: Vec<ParagraphPart>,
        base_style: Style,
        direction: ParagraphBaseDirection,
        facts: ParagraphFacts<'_>,
        replacements: Vec<surgeist_text::ParagraphReplacement>,
        options: PreparationOptions,
        tab_reference: Option<surgeist_text::ParagraphTabReference>,
    ) -> Self {
        let ancestor_wrap = base_style.wrap;
        let mut aggregate = Source::new("");
        let mut origins = Vec::new();
        let mut connections = Vec::new();
        let mut controls = Vec::new();
        let mut boundaries = Vec::new();
        let mut anchors = Vec::new();
        let mut nodes = Vec::new();
        let mut origin_nodes = Vec::new();
        let mut atomic_nodes = Vec::new();
        let mut control_nodes = Vec::new();
        let mut preceding_wrap = base_style.wrap;
        for part in parts {
            match part {
                ParagraphPart::Text(text) => {
                    let ParagraphText {
                        node,
                        source,
                        style,
                    } = *text;
                    let start = aggregate.text().len();
                    let range = aggregate.push(source.text().clone());
                    if !origins.is_empty() {
                        let connection = connections.len();
                        connections.push(ParagraphConnection {
                            at: start,
                            kind: ParagraphConnectionKind::Continuous,
                        });
                        boundaries.push(ParagraphBoundaryPolicy {
                            definition: ParagraphBoundaryDefinition::Connection(connection),
                            preceding_wrap,
                            following_wrap: style.wrap,
                            ancestor_wrap: base_style.wrap,
                        });
                    }
                    preceding_wrap = style.wrap;
                    aggregate.span(range, style);
                    for span in source.spans() {
                        let local = span.range();
                        aggregate.span(
                            surgeist_text::Range::new(start + local.start, start + local.end),
                            span.style().clone(),
                        );
                    }
                    if range.is_empty() {
                        anchors.push(ParagraphAnchorItem::EmptyOrigin(origins.len()));
                    }
                    let local = surgeist_text::Range::new(0, source.text().len());
                    origins.push(ParagraphOrigin {
                        text_node: None,
                        source,
                        local,
                        aggregate: range,
                        kind: ParagraphOriginKind::Authored,
                    });
                    origin_nodes.push(node);
                    nodes.push(node);
                }
                ParagraphPart::Atomic { node, size } => {
                    let at = aggregate.text().len();
                    let occurrence = atomic_nodes.len();
                    anchors.push(ParagraphAnchorItem::Atomic(occurrence));
                    for definition in [
                        ParagraphBoundaryDefinition::AtomicBefore(occurrence),
                        ParagraphBoundaryDefinition::AtomicAfter(occurrence),
                    ] {
                        boundaries.push(ParagraphBoundaryPolicy {
                            definition,
                            preceding_wrap,
                            following_wrap: base_style.wrap,
                            ancestor_wrap: base_style.wrap,
                        });
                    }
                    aggregate.inline_box(InlineBox::new(
                        Id::from_u64(u64::from(node)),
                        InlineBoxKind::InFlow,
                        at,
                        size,
                    ));
                    atomic_nodes.push(node);
                    nodes.push(node);
                }
                ParagraphPart::OutOfFlow { node, size } => {
                    let at = aggregate.text().len();
                    anchors.push(ParagraphAnchorItem::Atomic(atomic_nodes.len()));
                    aggregate.inline_box(InlineBox::new(
                        Id::from_u64(u64::from(node)),
                        InlineBoxKind::OutOfFlow,
                        at,
                        size,
                    ));
                    atomic_nodes.push(node);
                    nodes.push(node);
                }
                ParagraphPart::Control { node, kind } => {
                    let at = aggregate.text().len();
                    anchors.push(ParagraphAnchorItem::Control(controls.len()));
                    controls.push(ParagraphControl { at, kind });
                    control_nodes.push(node);
                    nodes.push(node);
                }
            }
        }
        let analyzed = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: facts.terminal,
                source: aggregate,
                base_style,
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction: direction,
                paragraph_strut: facts.strut,
                tab_reference,
                origins,
                connections,
                controls,
                boundary_policies: boundaries,
                replacements,
                anchor_order: anchors
                    .into_iter()
                    .map(|item| ParagraphAnchor {
                        item,
                        affinity: Affinity::Before,
                    })
                    .collect(),
            },
            surgeist_text::AnalysisTypography::HorizontalComposition,
        )
        .expect("test supplies complete original paragraph facts");
        let analyzed = in_flat_wrapping_context(analyzed, ancestor_wrap);
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(facts.font.to_vec()).unwrap();
        let prepared = system.prepare(&analyzed, options).unwrap();
        let origins = prepared
            .origin_views()
            .zip(origin_nodes)
            .map(|(view, node)| TextOriginBinding {
                origin: view.handle(),
                node,
            })
            .collect();
        let atomics = prepared
            .atomic_views()
            .zip(atomic_nodes)
            .map(|(view, node)| TextAtomicBinding {
                occurrence: view.handle(),
                node,
            })
            .collect();
        let controls = prepared
            .control_views()
            .zip(control_nodes)
            .map(|(view, node)| TextControlBinding {
                occurrence: view.handle(),
                node,
            })
            .collect();
        Self {
            prepared,
            nodes,
            origins,
            atomics,
            controls,
        }
    }

    pub(crate) fn text_input<S: LayoutScalar>(&self, node: u32) -> LayoutInputOf<S> {
        let origin = self
            .origins
            .iter()
            .find(|binding| binding.node == node)
            .expect("actual source node belongs to this preparation")
            .origin
            .clone();
        LayoutInputOf::inline_text(TextParticipation::new(origin))
    }

    pub(crate) fn origin(&self, node: u32) -> surgeist_text::PreparedOrigin {
        self.origins
            .iter()
            .find(|binding| binding.node == node)
            .expect("actual original occurrence for this tree node")
            .origin
            .clone()
    }

    pub(crate) fn binding(&self, container: u32) -> TextParagraphBinding<u32> {
        TextParagraphBinding {
            prepared: self.prepared.clone(),
            container,
            nodes: self.nodes.clone(),
            origins: self.origins.clone(),
            atomics: self.atomics.clone(),
            controls: self.controls.clone(),
            line_strut: None,
        }
    }
}
