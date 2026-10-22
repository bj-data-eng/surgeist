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
    let lines = crate::text_layout::fit::fit(
        crate::text_layout::fit::FitInput {
            prepared,
            available: crate::text_layout::fit::Available::Maximum,
            base_direction: surgeist_text::ParagraphBaseDirection::LeftToRight,
            alignment: crate::TextAlign::Start,
            last_alignment: crate::TextAlignLast::Auto,
            justification: crate::TextJustification::None,
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
                next_transition: None,
            })
        },
        |position, _| {
            Ok(position
                .minimum
                .map_or(position.proposed, |minimum| position.proposed.max(minimum)))
        },
        |_, _, _| {
            Ok(crate::text_layout::fit::LineAllocations {
                atomics: Vec::new(),
                controls: Vec::new(),
                line_strut: None,
            })
        },
        |_, _| Ok(None),
    )
    .unwrap_or_else(|_| panic!("registered Ahem paragraph finalizes"));
    lines[0]
        .finalized
        .fragments()
        .next()
        .expect("actual source fragment")
}
