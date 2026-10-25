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
        let mut nodes = Vec::new();
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
                    nodes.push(22);
                }
                events.push(E::EnterScope(1));
                events.push(E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                });
                nodes.push(20);
            }
            events.push(E::Origin {
                origin: index,
                affinity: text::Affinity::After,
            });
            nodes.push(10 + u32::try_from(index).unwrap());
            if range.end == scope.1 {
                events.push(E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                });
                events.push(E::ExitScope(1));
                nodes.push(21);
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
        let processed = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
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
                        topology: text::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: style.clone(),
                        used_direction: direction,
                    },
                    text::SourceProcessingScope::Inline {
                        trim: Default::default(),
                        style: Some(owner),
                        boundaries: Some(text::SourceProcessingScopeBoundaries {
                            start: 0,
                            end: 1,
                        }),
                    },
                ],
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: direction,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let analyzed =
            text::AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
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
                node: match view.kind() {
                    K::InlineBoundaryStart => 20,
                    K::InlineBoundaryEnd => 21,
                    K::OptionalBreak { .. } => 22,
                    _ => panic!("fixture contains only actual scope endpoints"),
                },
                occurrence: view.handle(),
            })
            .collect();
        Self {
            binding: TextParagraphBinding {
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
                    InlineBoundaryInputOf::new(
                        kind,
                        InlineMetricsOf::try_new(S::from_f64(8.0), S::from_f64(10.0)).unwrap(),
                        edge,
                    )
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
        let capture =
            LayoutFlowInput::try_new(&tree, 0, vec![self.binding.clone()], Vec::new()).unwrap();
        compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
        )
        .unwrap()
    }
}
