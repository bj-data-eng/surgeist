//! Immutable canonical body facts, independent of selected-line decoration.

use super::{
    PreparedBoundary, PreparedText, ProtocolError, ProtocolHandleKind,
    paragraph::ParagraphBaseDirection, protocol_error, selection::require_prepared,
};
use crate::Result;

/// Direction observations from one actual body bidi paragraph and queried range.
/// These facts grant no permission to break, select, measure or place text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineDirectionFacts {
    paragraph_strong_direction: Option<ParagraphBaseDirection>,
    has_eligible_strong: bool,
}

impl LineDirectionFacts {
    /// First P2/P3-eligible strong direction in the full actual body context.
    /// None affirmatively means that context contains no eligible strong text.
    #[must_use]
    pub fn paragraph_strong_direction(&self) -> Option<ParagraphBaseDirection> {
        self.paragraph_strong_direction
    }

    /// Whether this exact range contains strong text eligible under its full
    /// body-context isolate state; a suffix never starts a fresh isolate scan.
    #[must_use]
    pub fn has_eligible_strong(&self) -> bool {
        self.has_eligible_strong
    }
}

impl PreparedText {
    /// Observes canonical post-processing body facts at issued mixed boundaries.
    /// A terminating B belongs to its preceding nonempty range. The genuine
    /// source segment supplies empty-range context; crossing bodies is refused.
    pub fn line_direction_facts(
        &self,
        segment: &super::PreparedSourceSegment,
        start: &PreparedBoundary,
        end: &PreparedBoundary,
    ) -> Result<LineDirectionFacts> {
        super::segments::admit(self, segment)?;
        require_prepared(
            self,
            &start.prepared,
            ProtocolHandleKind::Boundary,
            start.index,
        )?;
        require_prepared(self, &end.prepared, ProtocolHandleKind::Boundary, end.index)?;
        let invalid = || protocol_error(ProtocolError::InvalidLineDirectionQuery);
        if start.index > end.index {
            return Err(invalid());
        }
        super::segments::admit_points(self, segment, start, end)?;
        let start = self
            .inner
            .tables
            .boundaries
            .get(start.index)
            .ok_or_else(invalid)?;
        let end = self
            .inner
            .tables
            .boundaries
            .get(end.index)
            .ok_or_else(invalid)?;
        let (strong, has_eligible_strong) = self
            .inner
            .analyzed
            .direction
            .query_in(segment.index, start.participant, end.participant)
            .ok_or_else(invalid)?;
        Ok(LineDirectionFacts {
            paragraph_strong_direction: strong.map(|level| {
                if level == 0 {
                    ParagraphBaseDirection::LeftToRight
                } else {
                    ParagraphBaseDirection::RightToLeft
                }
            }),
            has_eligible_strong,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AnalyzedSource, Font, ParagraphInput, ParagraphOrigin, ParagraphOriginKind, ParagraphStrut,
        PreparationOptions, Source, Style, System, SystemOptions,
    };

    fn prepared(text: &str, base_direction: ParagraphBaseDirection) -> PreparedText {
        prepare_input(input(text, base_direction))
    }

    fn prepare_input(input: ParagraphInput) -> PreparedText {
        let mut system = System::new(SystemOptions {
            font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let analyzed =
            AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
                .unwrap();
        system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap()
    }

    fn input(text: &str, base_direction: ParagraphBaseDirection) -> ParagraphInput {
        let source = Source::new(text);
        let end = source.text().len();
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                ..Style::default()
            },
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![ParagraphOrigin {
                source,
                local: crate::Range::new(0, end),
                aggregate: crate::Range::new(0, end),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        }
    }

    fn boundaries(prepared: &PreparedText) -> Vec<PreparedBoundary> {
        prepared
            .boundary_views()
            .map(|view| view.handle())
            .collect()
    }

    #[test]
    fn prepared_facts_distinguish_forced_body_base_full_strong_and_weak_suffixes() {
        let p = prepared("123A", ParagraphBaseDirection::RightToLeft);
        assert_eq!(
            p.analyzed_source().direction(),
            crate::Direction::RightToLeft
        );
        let b = boundaries(&p);
        let weak = p
            .line_direction_facts(&p.source_segments().next().unwrap(), &b[0], &b[3])
            .unwrap();
        assert_eq!(
            weak.paragraph_strong_direction(),
            Some(ParagraphBaseDirection::LeftToRight)
        );
        assert!(!weak.has_eligible_strong());
        assert!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[3], &b[4])
                .unwrap()
                .has_eligible_strong()
        );
        assert!(
            !p.line_direction_facts(&p.source_segments().next().unwrap(), &b[4], &b[4])
                .unwrap()
                .has_eligible_strong()
        );
        let rtl = prepared("\u{200f}A", ParagraphBaseDirection::LeftToRight);
        let b = boundaries(&rtl);
        let suffix = rtl
            .line_direction_facts(&rtl.source_segments().next().unwrap(), &b[1], &b[2])
            .unwrap();
        assert_eq!(
            suffix.paragraph_strong_direction(),
            Some(ParagraphBaseDirection::RightToLeft)
        );
        assert!(suffix.has_eligible_strong());
    }

    #[test]
    fn prepared_contexts_include_terminating_b_and_validate_empty_seams_and_ranges() {
        let p = prepared("A\nB\n", ParagraphBaseDirection::LeftToRight);
        let b = boundaries(&p);
        assert!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[0], &b[2])
                .unwrap()
                .has_eligible_strong()
        );
        assert!(
            !p.line_direction_facts(&p.source_segments().next().unwrap(), &b[2], &b[2])
                .unwrap()
                .has_eligible_strong()
        );
        assert_eq!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[4], &b[4])
                .unwrap()
                .paragraph_strong_direction(),
            None
        );
        assert!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[0], &b[3])
                .is_err()
        );
        assert!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[3], &b[2])
                .is_err()
        );
        let foreign = prepared("AB", ParagraphBaseDirection::LeftToRight);
        let other = boundaries(&foreign);
        let error = p
            .line_direction_facts(&p.source_segments().next().unwrap(), &b[0], &other[1])
            .unwrap_err();
        assert!(matches!(
            error.detail(),
            Some(crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::ForeignPreparation { .. }
            })
        ));
    }

    #[test]
    fn phase_one_collapse_and_actual_generated_origin_count_in_the_canonical_body() {
        let mut value = input("A\n..", ParagraphBaseDirection::RightToLeft);
        value.base_style.white_space = crate::WhiteSpace::Collapse;
        let p = prepare_input(value);
        assert_eq!(
            p.analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [65, 32, 46, 46]
        );
        let b = boundaries(&p);
        let weak = p
            .line_direction_facts(
                &p.source_segments().next().unwrap(),
                &b[2],
                b.last().unwrap(),
            )
            .unwrap();
        assert_eq!(
            weak.paragraph_strong_direction(),
            Some(ParagraphBaseDirection::LeftToRight)
        );
        assert!(!weak.has_eligible_strong());
        let mut value = input("\u{200f}A", ParagraphBaseDirection::LeftToRight);
        value.origins[0].kind = ParagraphOriginKind::Generated;
        let p = prepare_input(value);
        assert_eq!(
            p.origin_views().next().unwrap().kind(),
            ParagraphOriginKind::Generated
        );
        let b = boundaries(&p);
        let facts = p
            .line_direction_facts(&p.source_segments().next().unwrap(), &b[1], &b[2])
            .unwrap();
        assert_eq!(
            facts.paragraph_strong_direction(),
            Some(ParagraphBaseDirection::RightToLeft)
        );
        assert!(facts.has_eligible_strong());
    }

    #[test]
    fn actual_uppercase_expansion_has_two_issued_strong_ranges_over_one_original() {
        let mut value = input("ß", ParagraphBaseDirection::RightToLeft);
        value.base_style.text_transform = crate::TextTransform::Set(
            crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
        );
        let p = prepare_input(value);
        assert_eq!(
            p.inner
                .analyzed
                .projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [83, 83]
        );
        let b = boundaries(&p);
        assert_eq!(b.len(), 3);
        for pair in b.windows(2) {
            let facts = p
                .line_direction_facts(&p.source_segments().next().unwrap(), &pair[0], &pair[1])
                .unwrap();
            assert!(facts.has_eligible_strong());
            assert_eq!(
                facts.paragraph_strong_direction(),
                Some(ParagraphBaseDirection::LeftToRight)
            );
        }
        assert_eq!(
            p.origin_views().next().unwrap().aggregate_range(),
            crate::Range::new(0, 1)
        );
    }

    #[test]
    fn actual_isolate_forced_empty_and_deleted_participants_keep_their_issued_cuts() {
        use super::super::paragraph::{
            ParagraphAnchor, ParagraphAnchorItem, ParagraphConnection, ParagraphConnectionKind,
            ParagraphControl, ParagraphControlKind,
        };
        let mut value = input("A1B", ParagraphBaseDirection::RightToLeft);
        value.controls = vec![
            ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::RightToLeftIsolate,
            },
            ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::PopDirectionalIsolate,
            },
            ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::ForcedBreak,
            },
        ];
        value.anchor_order = (0..3)
            .map(|i| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(i),
                affinity: crate::Affinity::Before,
            })
            .collect();
        let p = prepare_input(value);
        let b = boundaries(&p);
        let isolated = p
            .line_direction_facts(&p.source_segments().next().unwrap(), &b[1], &b[2])
            .unwrap();
        assert_eq!(isolated.paragraph_strong_direction(), None);
        assert!(!isolated.has_eligible_strong());
        assert!(
            !p.line_direction_facts(&p.source_segments().next().unwrap(), &b[0], &b[5])
                .unwrap()
                .has_eligible_strong()
        );
        assert_eq!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[5], &b[5])
                .unwrap()
                .paragraph_strong_direction(),
            Some(ParagraphBaseDirection::LeftToRight)
        );
        assert!(
            p.line_direction_facts(&p.source_segments().next().unwrap(), &b[5], &b[6])
                .unwrap()
                .has_eligible_strong()
        );
        // Phase I collapses the duplicate SPACE; the retained first SPACE is
        // only trimmed later by Phase II and is not a deleted participant.
        let mut value = input("  A", ParagraphBaseDirection::RightToLeft);
        value.base_style.white_space = crate::WhiteSpace::Collapse;
        value.origins.insert(
            0,
            ParagraphOrigin {
                source: Source::new(""),
                local: crate::Range::new(0, 0),
                aggregate: crate::Range::new(0, 0),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            },
        );
        value.connections = vec![ParagraphConnection {
            at: 0,
            kind: ParagraphConnectionKind::Continuous,
        }];
        // Real producer-declared connection policy; the empty occurrence is
        // metadata and cannot by itself establish an influencing NCA pair.
        value.boundary_policies = vec![crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        }];
        value.anchor_order = vec![ParagraphAnchor {
            item: ParagraphAnchorItem::EmptyOrigin(0),
            affinity: crate::Affinity::Before,
        }];
        let p = prepare_input(value);
        assert_eq!(
            p.analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [32, 65]
        );
        let b = boundaries(&p);
        let mut empty_seen = false;
        let mut deleted_seen = false;
        for (at, participant) in p.participants().enumerate() {
            let super::super::PreparedParticipant::Text(unit) = participant else {
                continue;
            };
            let view = unit.view();
            if view.empty_origin().is_some() || view.processed_range().is_empty() {
                let facts = p
                    .line_direction_facts(&p.source_segments().next().unwrap(), &b[at], &b[at + 1])
                    .unwrap();
                assert_eq!(
                    facts.paragraph_strong_direction(),
                    Some(ParagraphBaseDirection::LeftToRight)
                );
                assert!(!facts.has_eligible_strong());
                empty_seen |= view.empty_origin().is_some();
                deleted_seen |=
                    !view.source_range().is_empty() && view.processed_range().is_empty();
            }
        }
        assert!(empty_seen && deleted_seen);
    }

    #[test]
    fn selected_strong_indication_does_not_become_canonical_body_authority() {
        use skrifa::MetadataProvider;
        let mut value = input("12345\u{ad}67", ParagraphBaseDirection::RightToLeft);
        value.base_style.hyphens = crate::Hyphens::Manual;
        value.base_style.hyphenate_character = crate::HyphenateCharacter::String("A".into());
        let p = prepare_input(value);
        let b = boundaries(&p);
        let before = p
            .line_direction_facts(
                &p.source_segments().next().unwrap(),
                &b[0],
                b.last().unwrap(),
            )
            .unwrap();
        assert_eq!(before.paragraph_strong_direction(), None);
        assert!(!before.has_eligible_strong());
        let offer = p
            .opportunity_views()
            .find(|offer| {
                offer
                    .alternatives()
                    .any(|row| row.text().code_units() == [65])
            })
            .unwrap()
            .handle();
        let alternative = offer
            .view()
            .alternatives()
            .find(|row| row.text().code_units() == [65])
            .unwrap()
            .handle();
        let end = offer.view().boundary();
        let to = end.view().participant_offset();
        let request = crate::LineSelectionRequest {
            segment: p.source_segments().next().unwrap(),
            start: b[0].clone(),
            end,
            termination: crate::PreparedTermination::Ordinary(offer),
            alternative: Some(alternative),
            incoming: None,
            participants: p.participants().take(to).collect(),
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            context: crate::LineContext {
                alignment_insets: crate::LineAlignmentInsets::ZERO,
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::RightToLeft,
                line_box_direction: crate::LineBoxDirection::RightToLeft,
                orientation: crate::LineOrientation::Horizontal,
                start: crate::LineStart::Paragraph,
                end: crate::LineEnd::Soft,
                available_inline: Some(100.0),
                indent: 0.0,
                alignment: crate::Alignment::Start,
                tabs: crate::TabContext::NoTabs,
                spacing: crate::SpacingPolicy::Spaces,
            },
        };
        let finalized = p
            .select(request)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: super::super::SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let font = skrifa::FontRef::new(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf"))
            .unwrap();
        let a = font.charmap().map('A').unwrap().to_u32();
        assert!(
            finalized
                .run_views()
                .any(|run| run.glyphs().any(|glyph| glyph.id() == a))
        );
        assert_eq!(
            p.line_direction_facts(
                &p.source_segments().next().unwrap(),
                &b[0],
                b.last().unwrap()
            )
            .unwrap(),
            before
        );
    }
}
