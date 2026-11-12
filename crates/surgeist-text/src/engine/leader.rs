//! Text preparation for one isolated generated leader pattern.

use std::sync::Arc;

use crate::{
    Affinity, Alignment, AnalysisTypography, AnalyzedSource, ContextualSourceInput,
    DocumentSegmentBreaks, FinalizedFragmentView, FinalizedGlyphView, FinalizedLine,
    FinalizedRunView, FormattedLineContext, LineAlignmentInsets, LineAllocations, LineBoxDirection,
    LineContext, LineEdgeUnitView, LineEnd, LineOrientation, LineSelectionRequest, LineStart,
    ParagraphBaseDirection, ParagraphOrigin, ParagraphOriginKind, ParagraphStrut,
    ParagraphTabReference, ParagraphTerminalEdge, PreparationOptions, PreparedBreakKind,
    PreparedTermination, PreparedText, ProcessedSource, ProtocolError, Range, Result,
    ScopeAlignmentLimits, ScopeAllocations, ScopeFontReference, SelectedMappingView, Source,
    SourceIdentity, SourceProcessingEnvelope, SourceProcessingEvent, SourceProcessingParagraph,
    SourceProcessingScope, SourceProcessingTopology, SourceWritingAxis, SpacingAllocations,
    SpacingPolicy, Style, TabContext, TabPreparation, Text, TextCombineUpright, WhiteSpaceTrim,
    Wrap, WrappingPolicy,
    protocol::{
        paragraph::ParagraphData,
        protocol_error,
        selection::{LineMetricPolicy, select_with_scratch},
    },
};

use super::{analysis::storage, shape::ShapeContext};

/// The decoded source spelling of one generated leader pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaderPattern {
    Dotted,
    Solid,
    Space,
    Literal(Text),
}

/// Used Text context for one isolated leader pattern.
///
/// A literal contains decoded code units. The source writing axis governs
/// combination independently of the physical shaping orientation. A retained
/// nonzero CSS tab uses the supplied reference metrics and the pattern's start.
#[derive(Clone, Debug)]
pub struct LeaderPatternInput {
    pub pattern: LeaderPattern,
    pub source_identity: SourceIdentity,
    pub style: Style,
    pub direction: ParagraphBaseDirection,
    pub writing_axis: SourceWritingAxis,
    pub typography: AnalysisTypography,
    pub orientation: LineOrientation,
    pub scale: f32,
    pub tab_reference: Option<ParagraphTabReference>,
    /// Context consumed when a vertical pattern selects a combined cell.
    pub combination: Option<LeaderCombinationContext>,
}

/// Actual generated inline context for horizontal-in-vertical composition.
#[derive(Clone, Copy, Debug)]
pub struct LeaderCombinationContext {
    pub horizontal_inner_direction: ParagraphBaseDirection,
    pub font_reference: ScopeFontReference,
    pub limits: ScopeAlignmentLimits,
}

/// One finalized repetition unit with its original source and actual fonts.
#[derive(Clone, Debug)]
pub struct FinalizedLeaderPattern {
    line: FinalizedLine,
}

impl FinalizedLeaderPattern {
    /// The decoded source before removal and whitespace processing.
    #[must_use]
    pub fn source(&self) -> &Source {
        self.processed_source().source()
    }

    /// Exact executed Phase I whitespace and deletion correspondence.
    #[must_use]
    pub fn processed_source(&self) -> &ProcessedSource {
        &self.line.inner.selected.prepared.analyzed.processed
    }

    /// Finalized unit geometry in local coordinates, including advance and baseline.
    #[must_use]
    pub fn metrics(&self) -> crate::LineFeedback {
        self.line.feedback()
    }

    /// Paint-order glyph runs with retained font and group associations.
    pub fn run_views(&self) -> impl ExactSizeIterator<Item = LeaderRunView<'_>> {
        let inner = &self.line.inner;
        let atom_count = inner.selected.presentation.projection.atoms.len();
        let groups = &inner.data.leader_paint_groups[atom_count..];
        self.line.run_views().map(|run| LeaderRunView {
            groups: &groups[run.paint_glyph_range()],
            run,
        })
    }

    /// Finalized cluster contributions, including glyphless spacing.
    pub fn cluster_views(&self) -> impl ExactSizeIterator<Item = LeaderClusterView<'_>> {
        let groups = &self.line.inner.data.leader_paint_groups;
        self.line.edge_units().map(|edge| LeaderClusterView {
            group: edge
                .first_presentation_atom()
                .map(|atom| LeaderPaintGroup(groups[atom])),
            edge,
        })
    }

    /// Actual fragment and source coverage in the finalized unit.
    pub fn fragment_views(&self) -> impl ExactSizeIterator<Item = FinalizedFragmentView<'_>> {
        self.line.fragment_views()
    }

    /// Actual original-source correspondence after transformation and shaping.
    pub fn mapping_views(&self) -> impl ExactSizeIterator<Item = SelectedMappingView<'_>> {
        self.line.mapping_views()
    }
}

/// Identity of one indivisible paint group within a finalized pattern.
/// Clones retain these identities. Compare groups within that pattern.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LeaderPaintGroup(usize);

/// A finalized font run with exact paint group membership for each glyph.
pub struct LeaderRunView<'a> {
    run: FinalizedRunView<'a>,
    groups: &'a [usize],
}

impl LeaderRunView<'_> {
    /// Actual retained font instance, basis, and metric frame.
    #[must_use]
    pub fn run(&self) -> &FinalizedRunView<'_> {
        &self.run
    }

    /// Each actual glyph in paint order with its indivisible group identity.
    pub fn glyphs(
        &self,
    ) -> impl ExactSizeIterator<Item = (FinalizedGlyphView<'_>, LeaderPaintGroup)> {
        self.run
            .glyphs()
            .zip(self.groups.iter().copied().map(LeaderPaintGroup))
    }
}

/// A finalized contribution in the isolated pattern's metric frame.
pub struct LeaderClusterView<'a> {
    edge: LineEdgeUnitView<'a>,
    group: Option<LeaderPaintGroup>,
}

impl LeaderClusterView<'_> {
    /// Group of this contribution. Several contributions can share one paint group.
    /// A group can have zero paint glyphs.
    #[must_use]
    pub fn paint_group(&self) -> Option<LeaderPaintGroup> {
        self.group
    }

    /// Signed inline advance after spacing.
    #[must_use]
    pub fn advance(&self) -> f64 {
        self.edge.advance()
    }

    /// Allocated metric bounds, independent of glyph ink.
    #[must_use]
    pub fn bounds(&self) -> crate::Rect {
        self.edge.bounds()
    }

    /// Position in the transformed presentation.
    #[must_use]
    pub fn presentation_range(&self) -> crate::PresentationRange {
        self.edge.presentation_range()
    }

    /// Nonempty original UTF-16 pieces associated with this contribution.
    pub fn source_pieces(&self) -> impl Iterator<Item = crate::Range> + '_ {
        self.edge.source_pieces()
    }
}

pub(crate) fn analyze(input: LeaderPatternInput) -> Result<(AnalyzedSource, PreparationOptions)> {
    let options = PreparationOptions::try_new(input.scale, TabPreparation::CssStops)?
        .with_orientation(input.orientation);
    let text = match &input.pattern {
        LeaderPattern::Dotted => ".".into(),
        LeaderPattern::Solid => "_".into(),
        LeaderPattern::Space => " ".into(),
        LeaderPattern::Literal(text) => text.clone(),
    };
    let mut source = Source::new(text);
    source.set_identity(
        input.source_identity.id(),
        input.source_identity.revision().get(),
    );
    let analyzed = if input.writing_axis == SourceWritingAxis::Vertical
        && input.style.text_combine_upright != TextCombineUpright::None
    {
        analyze_combination(source, input)?
    } else {
        let mut paragraph = ParagraphData::standalone(&source);
        paragraph.context.terminal_edge = ParagraphTerminalEdge::NonBlockBoundary;
        paragraph.context.base_direction = Some(input.direction);
        paragraph.context.tab_reference = input.tab_reference;
        paragraph.context.origins[0].kind = ParagraphOriginKind::Generated;
        let processed = Arc::new(crate::processing::process_leader_pattern(
            source,
            input.style,
        )?);
        AnalyzedSource::from_inner(super::analyze_paragraph(
            processed,
            paragraph,
            input.typography,
        )?)
    };
    let policies =
        storage::collect(
            analyzed
                .wrapping_relations()
                .map(|relation| WrappingPolicy {
                    relation: relation.handle(),
                    ancestor_wrap: Wrap::None,
                }),
        )?;
    Ok((analyzed.with_wrapping_policies(policies)?, options))
}

fn analyze_combination(source: Source, input: LeaderPatternInput) -> Result<AnalyzedSource> {
    let range = Range::new(0, source.text().len());
    let capture = ContextualSourceInput {
        scope_alignment_limits: input.combination.map(|context| context.limits),
        computed_line_boundaries: Vec::new(),
        origins: vec![ParagraphOrigin {
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Generated,
            text_node: None,
        }],
        source,
        segment_breaks: DocumentSegmentBreaks::Defined(Vec::new()),
        envelope: SourceProcessingEnvelope::Complete,
        connections: Vec::new(),
        controls: Vec::new(),
        boundary_policies: Vec::new(),
        replacements: Vec::new(),
        scopes: vec![
            SourceProcessingScope::Block {
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: input.style.clone(),
                used_direction: input.direction,
                writing_axis: Some(input.writing_axis),
                horizontal_inner_direction: None,
                font_reference: None,
            },
            SourceProcessingScope::Inline {
                trim: WhiteSpaceTrim::default(),
                style: Some(input.style),
                boundaries: None,
                alignment_boundary: None,
                font_reference: input.combination.map(|context| context.font_reference),
                horizontal_inner_direction: input
                    .combination
                    .map(|context| context.horizontal_inner_direction),
            },
        ],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: input.direction,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        joint_flows: Vec::new(),
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    };
    let processed = Arc::new(crate::processing::process_leader_capture(
        capture,
        input.tab_reference.as_ref(),
    )?);
    let paragraph = processed
        .paragraphs()
        .next()
        .expect("one isolated paragraph");
    AnalyzedSource::from_processed_paragraph(paragraph, input.typography)
}

pub(crate) fn finalize(
    prepared: PreparedText,
    direction: ParagraphBaseDirection,
    scratch: &mut ShapeContext,
) -> Result<FinalizedLeaderPattern> {
    let invalid = || protocol_error(ProtocolError::InvalidSourceSegment { segment: 0 });
    let mut segments = prepared.source_segments();
    if segments.len() != 1 {
        return Err(invalid());
    }
    let segment = segments.next().ok_or_else(invalid)?;
    let termination = segment.end_opportunity();
    if termination.view().kind() != PreparedBreakKind::ParagraphEnd {
        return Err(invalid());
    }
    let request = LineSelectionRequest {
        start: segment.start(),
        end: segment.end(),
        segment,
        participants: storage::collect(prepared.participants())?,
        termination: PreparedTermination::Ordinary(termination),
        alternative: None,
        incoming: None,
        context: LineContext {
            formatted_line: FormattedLineContext::default(),
            base_direction: direction,
            line_box_direction: match direction {
                ParagraphBaseDirection::LeftToRight => LineBoxDirection::LeftToRight,
                ParagraphBaseDirection::RightToLeft => LineBoxDirection::RightToLeft,
            },
            orientation: prepared.inner.options.orientation(),
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            alignment_insets: LineAlignmentInsets::ZERO,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::CssStops { origin: Some(0.0) },
            spacing: SpacingPolicy::None,
        },
        atomics: Vec::new(),
        controls: Vec::new(),
        boundary_context: Vec::new(),
        inline_boxes: None,
        line_strut: None,
        line_box_struts: Vec::new(),
    };
    let selected = select_with_scratch(&prepared, request, LineMetricPolicy::Native, scratch)?;
    let line = selected.finalize(LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::None,
    })?;
    Ok(FinalizedLeaderPattern { line })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Font, Id, PresentationMappingKind, SourceRevision, TabSize, WhiteSpace,
        test_fonts::{AHEM, FONT_LIMITS},
    };

    fn system() -> crate::System {
        let mut system = crate::System::default();
        system.register_fonts(AHEM.to_vec(), FONT_LIMITS).unwrap();
        system
    }

    fn input(pattern: LeaderPattern, white_space: WhiteSpace) -> LeaderPatternInput {
        let direction = ParagraphBaseDirection::LeftToRight;
        let style = Style {
            font: Font::new().family("Ahem"),
            size: 20.0,
            white_space,
            tab_size: TabSize::spaces(4.0).unwrap(),
            ..Style::default()
        };
        LeaderPatternInput {
            pattern,
            source_identity: SourceIdentity::new(Some(Id::from_u64(17)), SourceRevision::new(3)),
            tab_reference: Some(ParagraphTabReference::new(style.clone(), direction).unwrap()),
            style,
            direction,
            writing_axis: SourceWritingAxis::Horizontal,
            typography: AnalysisTypography::HorizontalComposition,
            orientation: LineOrientation::Horizontal,
            scale: 1.0,
            combination: None,
        }
    }

    fn glyphs(pattern: &FinalizedLeaderPattern) -> Vec<(u32, f32, f64)> {
        pattern
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|(glyph, _)| (glyph.id(), glyph.advance(), glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[test]
    fn keyword_patterns_match_decoded_literals_and_space_keeps_advance() {
        let mut system = system();
        for (keyword, literal) in [
            (LeaderPattern::Dotted, "."),
            (LeaderPattern::Solid, "_"),
            (LeaderPattern::Space, " "),
        ] {
            let pattern = system
                .prepare_leader_pattern(input(keyword, WhiteSpace::Collapse))
                .unwrap();
            let explicit = system
                .prepare_leader_pattern(input(
                    LeaderPattern::Literal(literal.into()),
                    WhiteSpace::Collapse,
                ))
                .unwrap();
            assert_eq!(
                pattern.source().text().code_units(),
                explicit.source().text().code_units()
            );
            assert_eq!(pattern.metrics().advance, 20.0);
            assert_eq!(glyphs(&pattern), glyphs(&explicit));
            if literal == " " {
                for run in pattern.run_views() {
                    let instance = run.run().effective_instance().unwrap();
                    for (glyph, _) in run.glyphs() {
                        let outline = instance
                            .glyph_outline(glyph.id(), instance.face().catalogue_identity())
                            .unwrap();
                        assert!(outline.bounds().is_none());
                    }
                }
            }
            assert!(
                pattern
                    .mapping_views()
                    .any(|piece| piece.source_range() == Range::new(0, 1))
            );
        }
    }

    #[test]
    fn removes_mandatory_breaks_before_collapse_and_retains_boundary_spaces() {
        let raw = " A \r\n \u{2028} B ";
        let mut system = system();
        let request = input(LeaderPattern::Literal(raw.into()), WhiteSpace::Collapse);
        let pattern = system.prepare_leader_pattern(request).unwrap();
        assert_eq!(
            pattern.source().text().code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            pattern.processed_source().processed_text().code_units(),
            [32, 65, 32, 66, 32]
        );
        assert_eq!(pattern.metrics().advance, 100.0);
        assert_eq!(pattern.source().id(), Some(Id::from_u64(17)));
        assert_eq!(pattern.source().revision(), 3);
        let deleted: Vec<_> = pattern
            .mapping_views()
            .filter(|piece| piece.kind() == PresentationMappingKind::Deleted)
            .flat_map(|piece| piece.source_range().start..piece.source_range().end)
            .collect();
        assert!(
            [3, 4, 6].iter().all(|position| deleted.contains(position)),
            "{deleted:?}"
        );
    }

    #[test]
    fn uses_unit_start_for_tabs_and_keeps_preserved_edge_advance() {
        let mut system = system();
        let pattern = system
            .prepare_leader_pattern(input(
                LeaderPattern::Literal("A\t\tB".into()),
                WhiteSpace::Preserve,
            ))
            .unwrap();
        assert_eq!(pattern.metrics().advance, 180.0);
        assert_eq!(glyphs(&pattern), [(35, 20.0, 0.0), (36, 20.0, 160.0)]);
        let run = pattern.run_views().next().unwrap();
        let painted: Vec<_> = run.glyphs().map(|(_, group)| group).collect();
        assert_eq!(pattern.cluster_views().len(), 4);
        assert_eq!(
            pattern.cluster_views().next().unwrap().paint_group(),
            Some(painted[0])
        );
        assert_eq!(
            pattern.cluster_views().last().unwrap().paint_group(),
            Some(painted[1])
        );
        for (index, (position, advance)) in [(20.0, 60.0), (80.0, 80.0)].into_iter().enumerate() {
            let tab = pattern.cluster_views().nth(index + 1).unwrap();
            assert_eq!(tab.advance(), advance);
            assert_eq!(tab.bounds().origin.x, position);
            assert_eq!(tab.bounds().size.width, advance);
            let presentation = tab.presentation_range();
            assert_eq!(
                (presentation.start(), presentation.end()),
                (index + 1, index + 2)
            );
            assert!(tab.source_pieces().eq([Range::new(index + 1, index + 2)]));
            assert!(!painted.contains(&tab.paint_group().unwrap()));
        }
        let edge_spaces = system
            .prepare_leader_pattern(input(
                LeaderPattern::Literal(" . ".into()),
                WhiteSpace::Preserve,
            ))
            .unwrap();
        assert_eq!(edge_spaces.metrics().advance, 60.0);
        assert_eq!(glyphs(&edge_spaces)[1], (16, 20.0, 20.0));
    }

    #[test]
    fn reads_each_expanded_cluster_with_its_own_geometry_and_shared_source() {
        let mut system = system();
        let mut request = input(LeaderPattern::Literal("ß".into()), WhiteSpace::Collapse);
        request.style.text_transform = crate::TextTransform::Set(
            crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
        );
        let pattern = system.prepare_leader_pattern(request).unwrap();
        assert_eq!(pattern.metrics().advance, 40.0);
        assert_eq!(glyphs(&pattern), [(53, 20.0, 0.0), (53, 20.0, 20.0)]);
        assert_eq!(pattern.cluster_views().len(), 2);
        let run = pattern.run_views().next().unwrap();
        let painted: Vec<_> = run.glyphs().map(|(_, group)| group).collect();
        assert_ne!(painted[0], painted[1]);
        for (index, (cluster, position)) in pattern.cluster_views().zip([0.0, 20.0]).enumerate() {
            assert_eq!(cluster.advance(), 20.0);
            assert_eq!(cluster.bounds().origin.x, position);
            assert_eq!(cluster.bounds().size.width, 20.0);
            let presentation = cluster.presentation_range();
            assert_eq!(
                (presentation.start(), presentation.end()),
                (index, index + 1)
            );
            assert!(cluster.source_pieces().eq([Range::new(0, 1)]));
            assert_eq!(cluster.paint_group(), Some(painted[index]));
        }
    }

    #[test]
    fn applies_ordinary_case_transform_and_word_spacing_after_break_removal() {
        let mut system = system();
        let mut request = input(
            LeaderPattern::Literal("a\r\n b".into()),
            WhiteSpace::Collapse,
        );
        request.style.text_transform = crate::TextTransform::Set(
            crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
        );
        request.style.word_spacing = 4.0;
        let pattern = system.prepare_leader_pattern(request).unwrap();
        assert_eq!(pattern.metrics().advance, 64.0);
        assert_eq!(
            glyphs(&pattern),
            [(35, 20.0, 0.0), (3, 20.0, 22.0), (36, 20.0, 44.0)]
        );
        assert_eq!(pattern.source().text().code_units(), [97, 13, 10, 32, 98]);
    }

    #[test]
    fn horizontal_pattern_keeps_normal_shaping_with_inherited_combination() {
        let mut system = system();
        let request = input(LeaderPattern::Literal("12".into()), WhiteSpace::Collapse);
        let ordinary = system.prepare_leader_pattern(request.clone()).unwrap();
        for combination in [
            crate::TextCombineUpright::All,
            crate::TextCombineUpright::Digits(crate::TextCombineUprightDigits::Two),
        ] {
            let mut request = request.clone();
            request.style.text_combine_upright = combination;
            let pattern = system.prepare_leader_pattern(request).unwrap();
            assert_eq!(pattern.metrics().advance, 40.0);
            assert_eq!(glyphs(&pattern), glyphs(&ordinary));
            assert_eq!(
                pattern.processed_source().base_style().text_combine_upright,
                combination
            );
            assert_eq!(pattern.cluster_views().len(), 2);
            for (index, cluster) in pattern.cluster_views().enumerate() {
                assert!(cluster.source_pieces().eq([Range::new(index, index + 1)]));
            }
        }
    }

    fn combined_input(
        text: &str,
        orientation: LineOrientation,
        mode: TextCombineUpright,
    ) -> LeaderPatternInput {
        let mut request = input(LeaderPattern::Literal(text.into()), WhiteSpace::Collapse);
        request.writing_axis = SourceWritingAxis::Vertical;
        request.orientation = orientation;
        request.style.text_combine_upright = mode;
        let (axis, baseline) = if orientation == LineOrientation::Vertical {
            request.typography = AnalysisTypography::Mixed;
            (
                crate::FontBaselineAxis::VerticalX,
                crate::FontBaseline::Central,
            )
        } else {
            (
                crate::FontBaselineAxis::HorizontalY,
                crate::FontBaseline::Alphabetic,
            )
        };
        request.combination = Some(LeaderCombinationContext {
            horizontal_inner_direction: ParagraphBaseDirection::LeftToRight,
            font_reference: ScopeFontReference {
                script: crate::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
                axis,
                baseline,
            },
            limits: ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216),
        });
        request
    }

    #[test]
    fn isolated_digits_combine_on_the_vertical_axis_and_retain_one_paint_group() {
        let mut system = crate::System::default();
        system
            .register_fonts(crate::vertical_test_fonts::native(), FONT_LIMITS)
            .unwrap();
        let digits = TextCombineUpright::Digits(crate::TextCombineUprightDigits::Two);
        for orientation in [
            LineOrientation::Vertical,
            LineOrientation::SidewaysCounterclockwise,
        ] {
            let pattern = system
                .prepare_leader_pattern(combined_input("1\r\n2", orientation, digits))
                .unwrap();
            assert_eq!(pattern.metrics().advance, 20.0);
            assert_eq!(pattern.cluster_views().len(), 1);
            let cluster = pattern.cluster_views().next().unwrap();
            let group = cluster.paint_group().unwrap();
            assert_eq!(cluster.advance(), 20.0);
            let run = pattern.run_views().next().unwrap();
            let cell = run.run().cell().unwrap();
            assert_eq!(cell.source_cell().mode(), digits);
            assert!(cell.source_cell().used_code_points().eq([49, 50]));
            assert_eq!(cell.square().size, crate::Size::new(20.0, 20.0));
            assert_eq!(run.glyphs().len(), 2);
            for ((glyph, painted), original) in
                run.glyphs().zip([Range::new(0, 1), Range::new(3, 4)])
            {
                assert_eq!(glyph.range(), original);
                assert_eq!(painted, group);
            }
            let processed = pattern.processed_source();
            assert_eq!(processed.processed_text().code_units(), [49, 50]);
            let cut = processed.position(1).unwrap();
            assert_eq!(
                processed
                    .processed_to_source(cut, Affinity::Before)
                    .unwrap()
                    .get(),
                1
            );
            assert_eq!(
                processed
                    .processed_to_source(cut, Affinity::After)
                    .unwrap()
                    .get(),
                3
            );

            let over_limit = system
                .prepare_leader_pattern(combined_input("123", orientation, digits))
                .unwrap();
            assert!(over_limit.run_views().all(|run| run.run().cell().is_none()));
            assert_eq!(over_limit.cluster_views().len(), 3);

            let mixed = system
                .prepare_leader_pattern(combined_input("A12B", orientation, digits))
                .unwrap();
            let groups: Vec<_> = mixed
                .cluster_views()
                .map(|cluster| cluster.paint_group().unwrap())
                .collect();
            assert_eq!(groups.len(), 3);
            assert_ne!(groups[0], groups[1]);
            assert_ne!(groups[1], groups[2]);
            let mut glyph_groups = Vec::new();
            for run in mixed.run_views() {
                glyph_groups.extend(run.glyphs().map(|(_, group)| group));
            }
            assert_eq!(glyph_groups, [groups[0], groups[1], groups[1], groups[2]]);
        }
    }

    #[test]
    fn combined_pattern_applies_inner_whitespace_to_outer_phase_one_survivors() {
        let mut system = system();
        let pattern = system
            .prepare_leader_pattern(combined_input(
                " 12 ",
                LineOrientation::SidewaysClockwise,
                TextCombineUpright::All,
            ))
            .unwrap();
        assert_eq!(
            pattern.processed_source().processed_text().code_units(),
            [32, 49, 50, 32]
        );
        assert_eq!(pattern.metrics().advance, 20.0);
        let run = pattern.run_views().next().unwrap();
        let cell = run.run().cell().unwrap();
        let source = cell.source_cell();
        assert!(source.used_code_points().eq([49, 50]));
        assert!(
            source
                .source_pieces()
                .filter(|piece| !piece.is_used())
                .map(|piece| piece.aggregate())
                .eq([Range::new(0, 1), Range::new(3, 4)])
        );

        let mut space = combined_input(
            " ",
            LineOrientation::SidewaysClockwise,
            TextCombineUpright::All,
        );
        space.pattern = LeaderPattern::Space;
        let space = system.prepare_leader_pattern(space).unwrap();
        assert_eq!(space.processed_source().processed_text().code_units(), [32]);
        assert_eq!(space.metrics().advance, 20.0);
        assert!(space.run_views().all(|run| run.glyphs().next().is_none()));
    }

    #[test]
    fn punctuation_at_pattern_edges_retains_its_complete_advance() {
        let mut system = system();
        let mut request = input(LeaderPattern::Literal("\"A.".into()), WhiteSpace::Collapse);
        request.style.hanging_punctuation = crate::HangingPunctuation::new(
            true,
            Some(crate::HangingPunctuationEnd::ForceEnd),
            false,
        );
        let pattern = system.prepare_leader_pattern(request).unwrap();
        let metrics = pattern.metrics();
        assert_eq!(glyphs(&pattern).len(), 3);
        assert_eq!(metrics.advance, 60.0);
        assert_eq!(metrics.hanging_start, 0.0);
        assert_eq!(metrics.hanging_end, 0.0);
    }

    #[test]
    fn empty_and_all_deleted_patterns_retain_source_anchors() {
        let mut system = system();
        for text in ["", "\r\n\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}"] {
            let pattern = system
                .prepare_leader_pattern(input(
                    LeaderPattern::Literal(text.into()),
                    WhiteSpace::Preserve,
                ))
                .unwrap();
            let processed = pattern.processed_source();
            assert_eq!(pattern.metrics().advance, 0.0);
            assert!(glyphs(&pattern).is_empty());
            assert!(processed.processed_text().is_empty());
            assert_eq!(
                pattern.source().text().code_units(),
                text.encode_utf16().collect::<Vec<_>>()
            );
            let zero = processed.position(0).unwrap();
            assert_eq!(
                processed
                    .processed_to_source(zero, Affinity::Before)
                    .unwrap()
                    .get(),
                0
            );
            assert_eq!(
                processed
                    .processed_to_source(zero, Affinity::After)
                    .unwrap()
                    .get(),
                text.encode_utf16().count()
            );
        }
    }

    #[test]
    fn deleted_breaks_preserve_joint_ligature_and_combining_coverage() {
        let mut ligature = crate::System::default();
        let font = crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(0x41, 35)]);
        ligature.register_fonts(font, FONT_LIMITS).unwrap();
        let pattern = ligature
            .prepare_leader_pattern(input(
                LeaderPattern::Literal("A\r\nA".into()),
                WhiteSpace::Preserve,
            ))
            .unwrap();
        assert_eq!(glyphs(&pattern), [(58, 20.0, 0.0)]);
        let run = pattern.run_views().next().unwrap();
        let (glyph, group) = run.glyphs().next().unwrap();
        assert_eq!(glyph.range(), Range::new(0, 4));
        assert!(
            pattern
                .cluster_views()
                .all(|cluster| cluster.paint_group() == Some(group))
        );
        assert!(
            pattern
                .cluster_views()
                .flat_map(|cluster| cluster.source_pieces().collect::<Vec<_>>())
                .eq([Range::new(0, 1), Range::new(3, 4)])
        );
        assert!(pattern.mapping_views().any(|piece| {
            piece.kind() == PresentationMappingKind::Deleted
                && piece.source_range() == Range::new(1, 3)
        }));

        let mut combining = crate::System::default();
        combining
            .register_fonts(crate::shaping_test_fonts::mark_ahem(), FONT_LIMITS)
            .unwrap();
        let pattern = combining
            .prepare_leader_pattern(input(
                LeaderPattern::Literal("A\u{2028}\u{0301}".into()),
                WhiteSpace::Preserve,
            ))
            .unwrap();
        assert_eq!(pattern.metrics().advance, 20.0);
        let run = pattern.run_views().next().unwrap();
        let (mark, group) = run.glyphs().find(|(glyph, _)| glyph.id() == 100).unwrap();
        assert!(run.glyphs().all(|(_, painted)| painted == group));
        assert!(
            pattern
                .cluster_views()
                .all(|cluster| cluster.paint_group() == Some(group))
        );
        assert_eq!(mark.advance(), 0.0);
        assert_eq!(mark.range(), Range::new(0, 3));
        let instance = run.run().effective_instance().unwrap();
        let outline = instance
            .glyph_outline(mark.id(), instance.face().catalogue_identity())
            .unwrap();
        assert!(outline.bounds().is_some());
    }

    #[test]
    fn zero_advance_pattern_retains_outline_and_source() {
        let mut hmtx = crate::test_fonts::table(AHEM, *b"hmtx").to_vec();
        let ahem_a_advance = 35 * 4;
        hmtx[ahem_a_advance..ahem_a_advance + 2].fill(0);
        let font = crate::test_fonts::replace_tables(AHEM, vec![(*b"hmtx", hmtx)]);
        let mut system = crate::System::default();
        system.register_fonts(font, FONT_LIMITS).unwrap();

        let pattern = system
            .prepare_leader_pattern(input(
                LeaderPattern::Literal("A".into()),
                WhiteSpace::Preserve,
            ))
            .unwrap();
        assert_eq!(pattern.metrics().advance, 0.0);
        let run = pattern.run_views().next().unwrap();
        let (glyph, group) = run.glyphs().next().unwrap();
        assert_eq!(
            (glyph.id(), glyph.advance(), glyph.range()),
            (35, 0.0, Range::new(0, 1))
        );
        let cluster = pattern.cluster_views().next().unwrap();
        assert_eq!(cluster.advance(), 0.0);
        assert_eq!(cluster.paint_group(), Some(group));
        assert!(cluster.source_pieces().eq([Range::new(0, 1)]));
        let instance = run.run().effective_instance().unwrap();
        let outline = instance
            .glyph_outline(glyph.id(), instance.face().catalogue_identity())
            .unwrap();
        assert!(
            outline
                .bounds()
                .is_some_and(|rect| rect.size.width > 0.0 && rect.size.height > 0.0)
        );
    }

    #[test]
    fn finalizes_rtl_and_sideways_with_actual_glyph_and_font_identity() {
        let mut rtl = crate::System::default();
        rtl.register_fonts(crate::shaping_test_fonts::arabic_ahem(), FONT_LIMITS)
            .unwrap();
        let mut request = input(
            LeaderPattern::Literal("ب\r\nب".into()),
            WhiteSpace::Preserve,
        );
        request.direction = ParagraphBaseDirection::RightToLeft;
        request.tab_reference = None;
        let pattern = rtl.prepare_leader_pattern(request).unwrap();
        let glyph_ids: Vec<_> = glyphs(&pattern).iter().map(|glyph| glyph.0).collect();
        assert_eq!(glyph_ids, [82, 100]);
        assert_eq!(pattern.source().text().code_units(), [0x628, 13, 10, 0x628]);

        let mut system = system();
        let mut request = input(LeaderPattern::Dotted, WhiteSpace::Collapse);
        request.orientation = LineOrientation::SidewaysCounterclockwise;
        let original = system.prepare_leader_pattern(request.clone()).unwrap();
        let repeat = original.clone();
        assert!(Arc::ptr_eq(&original.line.inner, &repeat.line.inner));
        let original_run = original.run_views().next().unwrap();
        assert_eq!(
            original_run.run().glyph_basis(),
            crate::GlyphBasis::Counterclockwise
        );
        assert_eq!(
            original_run.run().effective_instance().unwrap().font_size(),
            20.0
        );
        request.style.size = 25.0;
        let changed = system.prepare_leader_pattern(request).unwrap();
        assert_eq!(changed.metrics().advance, 25.0);
        assert_eq!(
            changed
                .run_views()
                .next()
                .unwrap()
                .run()
                .effective_instance()
                .unwrap()
                .font_size(),
            25.0
        );
        assert_eq!(original.metrics().advance, 20.0);
    }
}
