//! Genuine contextual scope endpoints and actual registered Ahem for Layout tests.

use super::layout_tree::PublicLayoutTreeOf;
use super::text::ahem_style;
use crate::*;
use std::sync::Arc;
use surgeist_text as text;
use text::{
    ParagraphBaseDirection, ParagraphControlKind as K, Source, SourceProcessingEvent as E, Wrap,
};

const AHEM: &[u8] = include_bytes!("../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

pub(crate) struct Fixture {
    pub(crate) binding: TextParagraphBinding<u32>,
}

impl Fixture {
    pub(crate) fn new(
        value: &str,
        scope: (usize, usize),
        mode: text::WhiteSpace,
        rtl: bool,
        tab: bool,
    ) -> Self {
        Self::with_generated_separators(value, scope, mode, rtl, tab, false)
    }

    pub(crate) fn with_generated_separators(
        value: &str,
        scope: (usize, usize),
        mode: text::WhiteSpace,
        rtl: bool,
        tab: bool,
        generated: bool,
    ) -> Self {
        Self::capture(value, scope, mode, rtl, tab, generated, false)
    }

    pub(crate) fn at_soft_seam() -> Self {
        Self::capture(
            "AB",
            (1, 2),
            text::WhiteSpace::Collapse,
            false,
            false,
            false,
            true,
        )
    }

    fn capture(
        value: &str,
        scope: (usize, usize),
        mode: text::WhiteSpace,
        rtl: bool,
        tab: bool,
        generated: bool,
        soft_seam: bool,
    ) -> Self {
        Self::prepare_input(
            Self::capture_input(value, scope, mode, rtl, generated, soft_seam),
            tab,
        )
    }

    /// Genuine complete P/C capture; existing single-Inline constructors retain
    /// their exact source, font, preparation and native binding behavior.
    /// A nonnominated C is declared separately by the consuming NativePair input.
    pub(crate) fn nested_parent(value: &str, rtl: bool, tab: bool, captured_child: bool) -> Self {
        let end = value.encode_utf16().count();
        let mut input = Self::capture_input(
            value,
            (0, end),
            text::WhiteSpace::Preserve,
            rtl,
            false,
            false,
        );
        input.controls.extend([
            text::ParagraphControl {
                at: 0,
                kind: K::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: end,
                kind: K::InlineBoundaryEnd,
            },
        ]);
        if captured_child {
            let mut child = input.scopes[1].clone();
            let text::SourceProcessingScope::Inline { boundaries, .. } = &mut child else {
                unreachable!("the canonical endpoint input owns an Inline")
            };
            *boundaries = Some(text::SourceProcessingScopeBoundaries { start: 2, end: 3 });
            input.scopes.push(child);
        }
        // P's genuine nominations remain 0/1. A nominated C owns scope2;
        // the NativePair alternative is explicitly declared INSIDE actual P,
        // without inventing an additional unrepresented captured Inline parent.
        let mut events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::EnterScope(1),
            E::Control {
                control: 0,
                affinity: text::Affinity::After,
            },
        ];
        if captured_child {
            events.push(E::EnterScope(2));
        }
        events.extend([
            E::Control {
                control: 2,
                affinity: text::Affinity::After,
            },
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::Control {
                control: 3,
                affinity: text::Affinity::After,
            },
        ]);
        if captured_child {
            events.push(E::ExitScope(2));
        }
        events.extend([
            E::Control {
                control: 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ]);
        input.events = events;
        Self::prepare_input(input, tab)
    }

    fn capture_input(
        value: &str,
        scope: (usize, usize),
        mode: text::WhiteSpace,
        rtl: bool,
        generated: bool,
        soft_seam: bool,
    ) -> text::ContextualSourceInput {
        let mut style = ahem_style(10.0, 10.0);
        style.white_space = mode;
        style.tab_size = text::TabSize::length(20.0).unwrap();
        if generated {
            style.locale = Some("ja".into());
        }
        let direction = if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        };
        let mut source = Source::new(value);
        let end = source.text().len();
        if generated {
            source.span(
                text::Range::new(0, end),
                text::Style {
                    word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
                    ..style.clone()
                },
            );
        }
        let owner = text::Style {
            word_space_transform: if generated {
                text::WordSpaceTransform::Space
            } else {
                text::WordSpaceTransform::None
            },
            ..style.clone()
        };
        assert!(scope.0 < scope.1 && scope.1 <= end);
        let mut cuts = vec![0, scope.0, scope.1, end];
        cuts.sort_unstable();
        cuts.dedup();
        let ranges = cuts
            .windows(2)
            .map(|pair| text::Range::new(pair[0], pair[1]))
            .collect::<Vec<_>>();
        let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
        for (index, range) in ranges.iter().enumerate() {
            if index > 0 {
                events.push(E::Connection(index - 1));
            }
            if range.start == scope.0 {
                if soft_seam {
                    events.push(E::Control {
                        control: 2,
                        affinity: text::Affinity::After,
                    });
                }
                events.push(E::EnterScope(1));
                events.push(E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                });
            }
            events.push(E::Origin {
                origin: index,
                affinity: text::Affinity::After,
            });
            if range.end == scope.1 {
                events.push(E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                });
                events.push(E::ExitScope(1));
            }
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let mut controls = vec![
            text::ParagraphControl {
                at: scope.0,
                kind: K::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: scope.1,
                kind: K::InlineBoundaryEnd,
            },
        ];
        if soft_seam {
            controls.push(text::ParagraphControl {
                at: scope.0,
                kind: K::OptionalBreak { wrap: Wrap::Word },
            });
        }
        text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: ranges
                .iter()
                .map(|range| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: *range,
                    aggregate: *range,
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (1..ranges.len())
                .map(|after_origin| text::SourceProcessingConnection {
                    before_origin: after_origin - 1,
                    after_origin,
                    kind: text::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls,
            boundary_policies: (0..ranges.len().saturating_sub(1))
                .map(|connection| text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(connection),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style.clone(),
                    used_direction: direction,
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: Some(owner),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: direction,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events,
        }
    }

    fn prepare_input(input: text::ContextualSourceInput, tab: bool) -> Self {
        let control_count = input.controls.len();
        let nodes = input
            .events
            .iter()
            .filter_map(|event| match *event {
                E::Origin { origin, .. } => Some(10 + u32::try_from(origin).unwrap()),
                E::Control { control, .. } => Some(20 + u32::try_from(control).unwrap()),
                _ => None,
            })
            .collect();
        let processed = Arc::new(text::process_contextual_source(input).unwrap());
        let analyzed = text::AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let options = text::PreparationOptions::try_new(
            1.0,
            if tab {
                text::TabPreparation::CssStops
            } else {
                text::TabPreparation::FontAdvance
            },
        )
        .unwrap();
        let prepared = system.prepare(&analyzed, options).unwrap();
        let origins = prepared
            .origin_views()
            .enumerate()
            .map(|(index, view)| TextOriginBinding {
                origin: view.handle(),
                node: 10 + u32::try_from(index).unwrap(),
            })
            .collect();
        let controls = prepared
            .control_views()
            .map(|view| TextControlBinding {
                node: 20
                    + u32::try_from(
                        (0..control_count)
                            .find(|&index| {
                                processed.captured_control(index).as_ref()
                                    == view.handle().captured_control().as_ref()
                            })
                            .expect("issued captured control retains actual input identity"),
                    )
                    .unwrap(),
                occurrence: view.handle(),
            })
            .collect();
        Self {
            binding: TextParagraphBinding {
                inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
                prepared,
                container: 0,
                nodes,
                origins,
                atomics: vec![],
                controls,
                line_strut: None,
            },
        }
    }

    pub(crate) fn tree<S: LayoutScalar>(
        &self,
        start: InlineBoundaryEdgeOf<S>,
        end: InlineBoundaryEdgeOf<S>,
        rtl: bool,
        width: Option<f64>,
        float: bool,
    ) -> PublicLayoutTreeOf<S> {
        let mut children = self.binding.nodes.clone();
        if float {
            children.insert(0, 2);
        }
        let mut tree = PublicLayoutTreeOf::new().children(0, children).style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                size: Size::new(
                    width.map_or(PreferredSizeOf::AUTO, |width| {
                        PreferredSizeOf::px(S::from_f64(width))
                    }),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
        for row in &self.binding.origins {
            tree = tree.children(row.node, []).input(
                row.node,
                LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
            );
        }
        for row in &self.binding.controls {
            if row.node == 22 {
                tree = tree.children(row.node, []).input(
                    row.node,
                    LayoutInputOf::text_control(TextControlParticipation::new(
                        row.occurrence.clone(),
                    )),
                );
                continue;
            }
            let (kind, edge) = if row.node == 20 {
                (InlineBoundaryKind::Start, start)
            } else {
                (InlineBoundaryKind::End, end)
            };
            tree = tree.children(row.node, []).input(
                row.node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        kind,
                        InlineMetricsOf::try_new(
                            crate::InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                        edge,
                    )
                    .unwrap()
                    .with_direction(if rtl {
                        Direction::Rtl
                    } else {
                        Direction::Ltr
                    }),
                ),
            );
        }
        if float {
            tree = tree.children(2, []).style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(25.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
        tree
    }

    pub(crate) fn compute<S: LayoutScalar>(
        &self,
        start: InlineBoundaryEdgeOf<S>,
        end: InlineBoundaryEdgeOf<S>,
        rtl: bool,
        width: Option<f64>,
        float: bool,
        available: AvailableOf<S>,
    ) -> CompletedLayoutBatchOf<u32, S> {
        let tree = self.tree(start, end, rtl, width, float);
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![self.binding.clone()],
            crate::WrappingInput {
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
        )
        .unwrap()
    }
}
