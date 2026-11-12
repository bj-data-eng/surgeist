//! Text preparation for one isolated generated leader pattern.

use std::sync::Arc;

use crate::{
    Alignment, AnalysisTypography, AnalyzedSource, FinalizedFragmentView, FinalizedLine,
    FinalizedRunView, FormattedLineContext, LineAlignmentInsets, LineAllocations, LineBoxDirection,
    LineContext, LineEnd, LineOrientation, LineSelectionRequest, LineStart, ParagraphBaseDirection,
    ParagraphOriginKind, ParagraphTabReference, ParagraphTerminalEdge, PreparationOptions,
    PreparedBreakKind, PreparedTermination, PreparedText, ProcessedSource, ProtocolError, Result,
    ScopeAllocations, SelectedMappingView, Source, SourceIdentity, SpacingAllocations,
    SpacingPolicy, Style, TabContext, TabPreparation, Text, Wrap, WrappingPolicy,
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
/// A literal contains decoded code units. The caller supplies the used bidi
/// base and typography for the pattern. A retained nonzero CSS tab consumes
/// the supplied reference metrics and uses the pattern's start as its origin.
#[derive(Clone, Debug)]
pub struct LeaderPatternInput {
    pub pattern: LeaderPattern,
    pub source_identity: SourceIdentity,
    pub style: Style,
    pub direction: ParagraphBaseDirection,
    pub typography: AnalysisTypography,
    pub orientation: LineOrientation,
    pub scale: f32,
    pub tab_reference: Option<ParagraphTabReference>,
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

    /// Actual glyph runs in the finalized unit, with their retained fonts.
    pub fn run_views(&self) -> impl ExactSizeIterator<Item = FinalizedRunView<'_>> {
        self.line.run_views()
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

pub(crate) fn analyze(input: LeaderPatternInput) -> Result<(AnalyzedSource, PreparationOptions)> {
    let options = PreparationOptions::try_new(input.scale, TabPreparation::CssStops)?
        .with_orientation(input.orientation);
    let text = match input.pattern {
        LeaderPattern::Dotted => ".".into(),
        LeaderPattern::Solid => "_".into(),
        LeaderPattern::Space => " ".into(),
        LeaderPattern::Literal(text) => text,
    };
    let mut source = Source::new(text);
    source.set_identity(
        input.source_identity.id(),
        input.source_identity.revision().get(),
    );
    let mut paragraph = ParagraphData::standalone(&source);
    paragraph.context.terminal_edge = ParagraphTerminalEdge::NonBlockBoundary;
    paragraph.context.base_direction = Some(input.direction);
    paragraph.context.tab_reference = input.tab_reference;
    paragraph.context.origins[0].kind = ParagraphOriginKind::Generated;
    let processed = Arc::new(crate::processing::process_leader_pattern(
        source,
        input.style,
    )?);
    let analyzed = AnalyzedSource::from_inner(super::analyze_paragraph(
        processed,
        paragraph,
        input.typography,
    )?);
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
        Affinity, Font, Id, PresentationMappingKind, Range, SourceRevision, TabSize, WhiteSpace,
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
            typography: AnalysisTypography::HorizontalComposition,
            orientation: LineOrientation::Horizontal,
            scale: 1.0,
        }
    }

    fn glyphs(pattern: &FinalizedLeaderPattern) -> Vec<(u32, f32, f64)> {
        pattern
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance(), glyph.pen()))
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
                    let instance = run.effective_instance().unwrap();
                    for glyph in run.glyphs() {
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
                LeaderPattern::Literal("A\tB".into()),
                WhiteSpace::Preserve,
            ))
            .unwrap();
        assert_eq!(pattern.metrics().advance, 100.0);
        assert_eq!(glyphs(&pattern), [(35, 20.0, 0.0), (36, 20.0, 80.0)]);
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
        assert_eq!(
            pattern
                .run_views()
                .next()
                .unwrap()
                .glyphs()
                .next()
                .unwrap()
                .range(),
            Range::new(0, 4)
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
        let mark = run.glyphs().find(|glyph| glyph.id() == 100).unwrap();
        assert_eq!(mark.advance(), 0.0);
        assert_eq!(mark.range(), Range::new(0, 3));
        let instance = run.effective_instance().unwrap();
        let outline = instance
            .glyph_outline(mark.id(), instance.face().catalogue_identity())
            .unwrap();
        assert!(outline.bounds().is_some());
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
            original_run.glyph_basis(),
            crate::GlyphBasis::Counterclockwise
        );
        assert_eq!(original_run.effective_instance().unwrap().font_size(), 20.0);
        request.style.size = 25.0;
        let changed = system.prepare_leader_pattern(request).unwrap();
        assert_eq!(changed.metrics().advance, 25.0);
        assert_eq!(
            changed
                .run_views()
                .next()
                .unwrap()
                .effective_instance()
                .unwrap()
                .font_size(),
            25.0
        );
        assert_eq!(original.metrics().advance, 20.0);
    }
}
