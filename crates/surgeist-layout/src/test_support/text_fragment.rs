//! Authentic finalized handles for carrier and transaction tests.

/// Finalize one registered Ahem origin through the actual consuming fitter.
/// Tests may vary their Layout phase geometry, but never invent a Text handle.
pub(crate) fn fragment() -> surgeist_text::FinalizedFragment {
    let paragraph = super::text::PreparedParagraph::plain(
        1,
        surgeist_text::Source::new("X"),
        super::text::ahem_style(10.0, 10.0),
    );
    fragment_for(&paragraph.prepared)
}

pub(crate) fn fragment_for(
    prepared: &surgeist_text::PreparedText,
) -> surgeist_text::FinalizedFragment {
    let scope_alignments = crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
        crate::text_layout::scope_alignment::ScopeAlignmentSnapshot::<f64>::admit::<()>(
            Vec::new(),
            &[],
        )
        .unwrap()
        .freeze(),
    );
    let lines = crate::text_layout::fit::fit(
        crate::text_layout::fit::FitInput {
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            formatted: &crate::text_layout::formatted::FormattedTailSnapshot::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                crate::Direction::Ltr,
            ),
            prepared,
            scope_alignments: &scope_alignments,
            available: crate::text_layout::fit::Available::Maximum,
            base_direction: surgeist_text::ParagraphBaseDirection::LeftToRight,
            block_direction: surgeist_text::ParagraphBaseDirection::LeftToRight,
            alignment: crate::TextAlign::Start,
            last_alignment: crate::TextAlignLast::Auto,
            justification: crate::TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: crate::text_layout::fit::TailContext::Unrestricted,
            orientation: surgeist_text::LineOrientation::Horizontal,
            atomics: &[],
            controls: &[],
            line_strut: None,
            reusable: &[],
        },
        |_, _| {
            Ok(crate::text_layout::fit::Band {
                start: 0.0,
                end: 100.0,
                intrinsic_end_occupation: None,
                next_transition: None,
            })
        },
        |position, _| {
            Ok(position
                .minimum
                .map_or(position.proposed, |minimum| position.proposed.max(minimum)))
        },
        |_, _, _, box_struts| {
            Ok(crate::text_layout::fit::LineAllocations {
                line_box_struts: box_struts.to_vec(),
                atomics: Vec::new(),
                controls: Vec::new(),
                line_strut: None,
            })
        },
        |_, _| Ok(None),
        crate::text_layout::fit::EncounterCallbacks {
            checkpoint_group: || {},
            replay: |_: &surgeist_text::FinalizedLine,
                     _: crate::text_layout::fit::EncounterReplay| Ok(Vec::new()),
            accept: || Ok(()),
            reset: |_| {},
        },
    )
    .unwrap_or_else(|_| panic!("registered Ahem paragraph finalizes"));
    lines[0]
        .finalized
        .fragments()
        .next()
        .expect("actual source fragment")
}
