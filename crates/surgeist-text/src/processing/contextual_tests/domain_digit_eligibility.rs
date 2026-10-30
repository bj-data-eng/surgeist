//! Source-only Digits activation through the real completed-domain front.
//! Native issue-784 policy admits ASCII digit EGCs with the specified marks,
//! rejects overlong maximal runs and retains pre-fullwidth ASCII identity.
//! The capability guard observes cell PRESENCE, not exact grouping or payload.

use crate::*;
use std::sync::Arc;

fn capture(text: &str, limit: TextCombineUprightDigits, fullwidth: bool) -> Arc<ProcessedSource> {
    let mut input = super::input(text, WhiteSpace::Preserve);
    input.source = Source::identified(text, Id::from_u64(78474), 15);
    input.origins = vec![super::origin(
        &input.source,
        Range::new(0, text.encode_utf16().count()),
    )];
    let SourceProcessingScope::Block {
        topology,
        base_style,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual one-raw Block fixture")
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    base_style.text_combine_upright = TextCombineUpright::Digits(limit);
    base_style.text_transform = if fullwidth {
        TextTransform::Set(TextTransformSet::none().with_full_width(true))
    } else {
        TextTransform::none()
    };
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        text.encode_utf16().collect::<Vec<_>>()
    );
    assert!(processed.captured_joint_flow(0).is_none());
    let root = processed.captured_scope(0).unwrap();
    assert_eq!(root.domain_topology(), SourceProcessingTopology::Complete);
    assert_eq!(root.writing_axis(), Some(SourceWritingAxis::Vertical));
    assert_eq!(
        root.horizontal_inner_direction(),
        Some(ParagraphBaseDirection::LeftToRight)
    );
    assert_eq!(
        root.text_combine_upright(),
        Some(TextCombineUpright::Digits(limit))
    );
    processed
}

fn assert_activation(
    text: &str,
    limit: TextCombineUprightDigits,
    fullwidth: bool,
    expects_cell: bool,
) {
    let processed = capture(text, limit, fullwidth);
    let raw = processed.paragraphs().next().unwrap();
    let extent = Range::new(0, text.encode_utf16().count());
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [extent]);
    assert_eq!(
        raw.captured_paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    let domain =
        AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.source(), processed.source());
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    let owner = output.captured_owner().unwrap();
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert!(owner.joint_flow().is_none());
    assert_eq!(output.inner.paragraph.origins.len(), 1);
    let origin = &output.inner.paragraph.origins[0];
    assert_eq!(origin.aggregate, extent);
    assert_eq!(origin.local, extent);
    assert_eq!(
        origin.original(output.processed_source()),
        processed.source()
    );
    let map = &output.inner.paragraph.origin_map;
    assert_eq!(
        map.flow_origin(map.view(0).unwrap()).captured_event(),
        Some(processed.captured_origin(0).unwrap().index)
    );
    let identity = output.identity();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    assert_eq!(system.registered_font_faces().count(), 0);
    let stats = system.stats();
    let error = system
        .prepare(output, PreparationOptions::default())
        .unwrap_err();
    if expects_cell {
        assert_eq!(error.code, ErrorCode::UnsupportedFeature, "{text:?}");
        assert!(
            matches!(
                error.detail(),
                Some(ErrorDetail::TextCombine {
                    reason: TextCombineError::UnsupportedCellPreparation
                })
            ),
            "{text:?}: {error:?}"
        );
        assert_eq!(system.stats(), stats);
    } else {
        // A rejected run must not leave even a short accepted prefix cell.
        // The real ordinary path reaches its own unavailable-font outcome.
        assert_eq!(
            error.code,
            ErrorCode::NoUsableFontFace,
            "{text:?}: {error:?}"
        );
    }
    assert_eq!(output.identity(), identity);
    assert_eq!(processed.source().text(), &Text::from(text));
    assert_eq!(processed.processed_text(), &Text::from(text));
}

#[test]
fn native_marked_ascii_digit_cases_activate_but_non_digit_keycap_bases_do_not() {
    // Native policy explicitly includes these literal attached-mark forms.
    // Presence of a selected cell does not establish its precise membership,
    // number of groups, selection count or compression payload cardinality.
    for text in ["9\u{fe0f}\u{20e3}", "9\u{20e3}", "9\u{20dd}"] {
        assert_activation(text, TextCombineUprightDigits::Two, false, true);
    }
    for text in ["#\u{fe0f}\u{20e3}", "*\u{fe0f}\u{20e3}"] {
        assert_activation(text, TextCombineUprightDigits::Two, false, false);
    }
}

#[test]
fn overlong_maximal_digit_runs_leave_no_prefix_cell_at_shorter_limits() {
    // Four consecutive ASCII digit EGCs are one maximal sequence. Digits2/3
    // cannot salvage a short prefix or split it into several accepted cells.
    // NoUsableFontFace detects ANY such unintended cell via the earlier guard.
    for limit in [
        TextCombineUprightDigits::Two,
        TextCombineUprightDigits::Three,
    ] {
        assert_activation("2010", limit, false, false);
    }
    assert_activation("23", TextCombineUprightDigits::Two, false, true);
    assert_activation("2010", TextCombineUprightDigits::Four, false, true);
}

#[test]
fn fullwidth_transform_keeps_ascii_digit_activation_without_admitting_literal_fullwidth() {
    // Same normal width mapping, different authentic source identities: the
    // ASCII digits remain eligible; literal U+FF19 acquires no ASCII identity.
    assert_activation("99", TextCombineUprightDigits::Two, true, true);
    assert_activation("９９", TextCombineUprightDigits::Two, false, false);
    assert_activation("９９", TextCombineUprightDigits::Two, true, false);
}

// The adopted source-stage model admits established ineligible Digits through
// the exact front. Four ASCII digit EGCs already exceed Digits2/3: extending
// that maximal sequence cannot make it eligible or require a cell's context.
fn overlimit_without_cell_context(
    limit: TextCombineUprightDigits,
    remote_digit: bool,
) -> Arc<ProcessedSource> {
    let text = if remote_digit { "20109" } else { "2010" };
    let mut input = super::input(text, WhiteSpace::Preserve);
    let SourceProcessingScope::Block {
        topology,
        base_style,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual complete root Block")
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = None;
    *horizontal_inner_direction = None;
    base_style.text_combine_upright = TextCombineUpright::Digits(limit);
    base_style.text_transform = TextTransform::none();
    if remote_digit {
        let mut unused = super::style(WhiteSpace::Preserve, 10.0);
        unused.size = -1.0;
        unused.text_combine_upright = TextCombineUpright::Digits(limit);
        unused.text_transform = TextTransform::none();
        input.source.span(Range::new(4, 5), unused);
        input.paragraphs.push(SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::RightToLeft,
            paragraph_strut: ParagraphStrut::AllocatedContributions,
        });
        input.origins = [Range::new(0, 4), Range::new(4, 5)]
            .into_iter()
            .map(|range| super::origin(&input.source, range))
            .collect();
        input.events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::EnterParagraph(1),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(0),
        ];
    }
    assert!(input.connections.is_empty());
    assert!(input.joint_flows.is_empty());
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        text.encode_utf16().collect::<Vec<_>>()
    );
    let root = processed.captured_scope(0).unwrap();
    assert_eq!(root.domain_topology(), SourceProcessingTopology::Complete);
    assert_eq!(root.writing_axis(), None);
    assert_eq!(root.horizontal_inner_direction(), None);
    assert_eq!(
        root.text_combine_upright(),
        Some(TextCombineUpright::Digits(limit))
    );
    assert!(processed.captured_connection(0).is_none());
    assert!(processed.captured_joint_flow(0).is_none());
    processed
}

fn assert_ordinary_overlimit_view(output: &AnalyzedSource, processed: &Arc<ProcessedSource>) {
    assert_eq!(output.inner.projection.text, "2010");
    assert_eq!(output.combined_cells().len(), 0);
    assert_eq!(output.source(), processed.source());
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [
            (0x32, Range::new(0, 1)),
            (0x30, Range::new(1, 2)),
            (0x31, Range::new(2, 3)),
            (0x30, Range::new(3, 4)),
        ]
    );
    let owner = output.captured_owner().unwrap();
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert!(owner.joint_flow().is_none());
    assert_eq!(output.inner.paragraph.origins.len(), 1);
    let original = &output.inner.paragraph.origins[0];
    assert_eq!(original.aggregate, Range::new(0, 4));
    assert_eq!(original.local, Range::new(0, 4));
    assert_eq!(
        original.original(output.processed_source()),
        processed.source()
    );
    let map = &output.inner.paragraph.origin_map;
    assert_eq!(
        map.flow_origin(map.view(0).unwrap()).captured_event(),
        Some(processed.captured_origin(0).unwrap().index)
    );
}

#[test]
fn an_already_overlimit_digit_run_needs_neither_writing_axis_nor_inner_direction() {
    for limit in [
        TextCombineUprightDigits::Two,
        TextCombineUprightDigits::Three,
    ] {
        let processed = overlimit_without_cell_context(limit, false);
        let raw = processed.paragraphs().next().unwrap();
        assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(0, 4)]);
        let exact = AnalyzedSource::from_processed_paragraph(
            raw.clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_ordinary_overlimit_view(&exact, &processed);
        let identity = exact.identity();
        let domain = AnalyzedDomain::from_processed_paragraph(
            raw,
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(Arc::ptr_eq(domain.processed_source(), &processed));
        assert_eq!(domain.outputs().len(), 1);
        assert_ordinary_overlimit_view(domain.outputs().next().unwrap(), &processed);
        assert_eq!(exact.identity(), identity);
        assert_ordinary_overlimit_view(&exact, &processed);
    }
}

#[test]
fn an_exact_overlimit_digit_run_does_not_demand_unknown_remote_growth_or_unused_style() {
    for limit in [
        TextCombineUprightDigits::Two,
        TextCombineUprightDigits::Three,
    ] {
        let processed = overlimit_without_cell_context(limit, true);
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(raw.len(), 2);
        assert_eq!(
            raw[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 4)]
        );
        assert_eq!(
            raw[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(4, 5)]
        );
        let exact = AnalyzedSource::from_processed_paragraph(
            raw[0].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_ordinary_overlimit_view(&exact, &processed);
        let identity = exact.identity();
        // Only this real remote request consumes its invalid finite font size;
        // the requested four-digit view has no reason to validate that field.
        let error = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidStyle);
        assert_eq!(exact.identity(), identity);
        assert_ordinary_overlimit_view(&exact, &processed);
        assert_eq!(processed.source().text(), &Text::from("20109"));
        assert_eq!(processed.processed_text(), &Text::from("20109"));
    }
}
