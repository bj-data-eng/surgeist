//! Deterministic prepared Text paragraphs bound to a generated host.
//!
//! The paragraphs use the Ahem test font, whose advances are one em for every
//! glyph, so a word's width is its letter count times the font size and the line
//! breaks depend only on the generated words and the container width. The font
//! file is the one `surgeist-text` keeps as a test fixture, read by path;
//! the `surgeist-test` dev-dependency supplies its decoding limits.

use std::sync::Arc;

use surgeist_layout::{
    Float, FloatExclusion, InlineBoxFragmentationDemand, InlineMetrics, LayoutFlowInput,
    LayoutInput, LineBreakInput, TextControlBinding, TextOriginBinding, TextParagraphBinding,
    TextParticipation, TextWrapStyle, WrappingInput,
};
use surgeist_text as t;

use super::host::{Host, boxed};

const AHEM: &[u8] = include_bytes!("../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

/// Font size and line height of every generated paragraph, in px.
pub const EM: f32 = 10.0;

/// `count` pseudo-random words of one to nine letters, separated by U+0020 and
/// ending in a space. The sequence depends only on `seed`.
pub fn words(count: usize, seed: u64) -> String {
    let mut state = seed;
    let mut text = String::new();
    for _ in 0..count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let length = 1 + (state >> 59) % 9;
        for letter in 0..length {
            let glyph = b'A' + u8::try_from((state >> (8 + letter)) % 26).expect("a letter");
            text.push(char::from(glyph));
        }
        text.push(' ');
    }
    text
}

/// A block container of `width` px holding forced-break groups of prepared
/// Text, optionally preceded by shape floats.
pub struct Paragraphs {
    pub host: Host,
    binding: TextParagraphBinding<u32>,
}

impl Paragraphs {
    pub fn new(
        groups: &[String],
        width: f32,
        wrap_style: TextWrapStyle,
        floats: usize,
        shape: bool,
    ) -> Self {
        use t::SourceProcessingEvent as E;
        let raw = groups.concat();
        let base = t::Style {
            font: t::Font::new().family("Ahem"),
            size: EM,
            line_height: t::LineHeight::Absolute(EM.into()),
            white_space: t::WhiteSpace::Collapse,
            wrap: t::Wrap::Word,
            word_break: t::WordBreak::Normal,
            overflow_wrap: t::OverflowWrap::Normal,
            hyphens: t::Hyphens::None,
            ..t::Style::default()
        };
        let mut source = t::Source::identified(raw.as_str(), t::Id::from_u64(2245), 1);
        source.span(t::Range::new(0, raw.len()), base.clone());
        let mut at = 0;
        let ranges: Vec<_> = groups
            .iter()
            .map(|group| {
                let range = t::Range::new(at, at + group.len());
                at = range.end;
                range
            })
            .collect();
        let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
        for index in 0..groups.len() {
            if index > 0 {
                events.extend([
                    E::Control {
                        control: index - 1,
                        affinity: t::Affinity::After,
                    },
                    E::Connection(index - 1),
                ]);
            }
            events.push(E::Origin {
                origin: index,
                affinity: t::Affinity::After,
            });
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source: source.clone(),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: ranges
                    .iter()
                    .map(|&range| t::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: t::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                controls: ranges
                    .iter()
                    .take(groups.len() - 1)
                    .map(|range| t::ParagraphControl {
                        at: range.end,
                        kind: t::ParagraphControlKind::ForcedBreak,
                    })
                    .collect(),
                connections: (1..groups.len())
                    .map(|index| t::SourceProcessingConnection {
                        before_origin: index - 1,
                        after_origin: index,
                        kind: t::ParagraphConnectionKind::Continuous,
                    })
                    .collect(),
                boundary_policies: (1..groups.len())
                    .map(|index| t::ParagraphBoundaryPolicy {
                        definition: t::ParagraphBoundaryDefinition::Connection(index - 1),
                        preceding_wrap: t::Wrap::Word,
                        following_wrap: t::Wrap::Word,
                        ancestor_wrap: t::Wrap::Word,
                    })
                    .collect(),
                scopes: vec![t::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: t::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: base,
                    used_direction: t::ParagraphBaseDirection::LeftToRight,
                }],
                paragraphs: vec![t::SourceProcessingParagraph {
                    base_direction: t::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                }],
                replacements: vec![],
                computed_line_boundaries: vec![],
                events,
            })
            .expect("the generated source is admitted"),
        );
        let analyzed = t::AnalyzedSource::from_processed_paragraph(
            capture.paragraphs().next().expect("one paragraph"),
            t::AnalysisTypography::HorizontalComposition,
        )
        .expect("the paragraph analyzes");
        let policies = analyzed
            .wrapping_relations()
            .filter(|relation| relation.ancestor_wrap().is_none())
            .map(|relation| t::WrappingPolicy {
                relation: relation.handle(),
                ancestor_wrap: t::Wrap::Word,
            })
            .collect();
        let mut fonts = t::System::default();
        fonts
            .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
            .expect("the Ahem fixture decodes");
        let prepared = fonts
            .prepare(
                &analyzed
                    .with_wrapping_policies(policies)
                    .expect("the wrapping policies apply"),
                t::PreparationOptions::default(),
            )
            .expect("the paragraph prepares");
        let origins: Vec<_> = prepared.origin_views().map(|view| view.handle()).collect();
        let controls: Vec<_> = prepared.control_views().map(|view| view.handle()).collect();

        let mut root = boxed(surgeist_layout::Display::Block, Some(width), None);
        root.text_wrap_style = wrap_style;
        let mut host = Host::new(root);
        host.shape_floats = shape;
        for _ in 0..floats {
            let mut float = boxed(
                surgeist_layout::Display::Block,
                Some(4.0 * EM),
                Some(4.0 * EM),
            );
            float.float = Float::Left;
            if shape {
                float.float_exclusion = FloatExclusion::Shape;
            }
            host.add(0, float);
        }
        let mut nodes = Vec::new();
        let mut origin_bindings = Vec::new();
        let mut control_bindings = Vec::new();
        for (index, origin) in origins.iter().enumerate() {
            if index > 0 {
                let node = host.push(LayoutInput::line_break(
                    LineBreakInput::new().with_metrics(
                        InlineMetrics::try_new(
                            surgeist_layout::InlineBaselineContext::Physical,
                            0.8 * EM,
                            EM,
                        )
                        .expect("finite metrics"),
                    ),
                ));
                host.children[0].push(node);
                nodes.push(node);
                control_bindings.push(TextControlBinding {
                    occurrence: controls[index - 1].clone(),
                    node,
                });
            }
            let node = host.push(LayoutInput::inline_text(TextParticipation::new(
                origin.clone(),
            )));
            host.children[0].push(node);
            nodes.push(node);
            origin_bindings.push(TextOriginBinding {
                origin: origin.clone(),
                node,
            });
        }
        Self {
            host,
            binding: TextParagraphBinding {
                inline_box_fragments: InlineBoxFragmentationDemand::NotRequested,
                prepared,
                container: 0,
                nodes,
                origins: origin_bindings,
                controls: control_bindings,
                atomics: vec![],
                line_strut: None,
            },
        }
    }

    /// One full flow computation over a fresh checked flow input.
    pub fn compute(&self, width: f32) -> surgeist_layout::CompletedLayoutBatch<u32> {
        let flow = LayoutFlowInput::try_new(
            &self.host,
            0,
            vec![self.binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )
        .expect("the generated flow binds");
        surgeist_layout::compute_layout_with_flow(&flow, 0, super::host::root_request(width))
            .expect("the generated paragraph lays out")
    }
}
