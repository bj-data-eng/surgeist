//! Single-origin incidence uses issued preparation and occurrence custody.
fn text_origin_incidence(
    unit: &surgeist_text::PreparedUnit,
    origin: &surgeist_text::PreparedOrigin,
    local: usize,
    funding: Option<crate::text_layout::scope_alignment::EntryFundingRef<'_>>,
) -> Result<Option<super::OriginIncidence>, String> {
    super::text_origin_incidence(unit, origin, local, funding).map_err(|error| {
        format!(
            "{:?}",
            crate::block::inline_run::text_run_error::<u32, f64, ()>(0, error)
        )
    })
}
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use surgeist_text as text;

fn paragraph(parts: Vec<ParagraphPart>) -> PreparedParagraph {
    PreparedParagraph::new(
        parts,
        ahem_style(10.0, 10.0),
        text::ParagraphBaseDirection::LeftToRight,
    )
}

#[test]
fn same_preparation_positive_extent_and_foreign_same_range() {
    let style = ahem_style(10.0, 10.0);
    let actual = paragraph(vec![ParagraphPart::text(
        1,
        text::Source::new("AB"),
        style.clone(),
    )]);
    let foreign = paragraph(vec![ParagraphPart::text(1, text::Source::new("AB"), style)]);
    let origin = actual.origin(1);
    let mut seen = 0;
    for participant in actual.prepared.participants() {
        let text::PreparedParticipant::Text(unit) = participant else {
            continue;
        };
        let range = unit.view().source_range();
        let incidence = text_origin_incidence(&unit, &origin, range.start, None)
            .unwrap()
            .unwrap();
        assert!(incidence.represented);
        assert!(!incidence.inside);
        assert!(
            text_origin_incidence(&unit, &foreign.origin(1), range.start, None)
                .unwrap()
                .is_none()
        );
        seen += 1;
    }
    assert_eq!(seen, 2);
}

#[test]
fn equal_anchor_empty_originals_keep_distinct_occurrences() {
    let style = ahem_style(10.0, 10.0);
    let actual = paragraph(vec![
        ParagraphPart::text(1, text::Source::new(""), style.clone()),
        ParagraphPart::text(2, text::Source::new(""), style),
    ]);
    let mut seen = 0;
    for participant in actual.prepared.participants() {
        let text::PreparedParticipant::Text(unit) = participant else {
            continue;
        };
        let Some(origin) = unit.view().empty_origin() else {
            continue;
        };
        let other = if origin == actual.origin(1) {
            actual.origin(2)
        } else {
            actual.origin(1)
        };
        let incidence = text_origin_incidence(&unit, &origin, 0, None)
            .unwrap()
            .unwrap();
        assert!(incidence.represented);
        assert!(!incidence.inside);
        assert!(
            text_origin_incidence(&unit, &other, 0, None)
                .unwrap()
                .is_none()
        );
        seen += 1;
    }
    assert_eq!(seen, 2);
}

#[test]
fn generated_empty_anchor_does_not_donate_an_original() {
    let mut style = ahem_style(10.0, 10.0);
    style.word_space_transform = text::WordSpaceTransform::Space;
    let actual = paragraph(vec![
        ParagraphPart::text(1, text::Source::new(""), style.clone()),
        ParagraphPart::Control {
            node: 2,
            kind: text::ParagraphControlKind::Wbr {
                style: Box::new(style),
            },
        },
    ]);
    let mut generated = 0;
    for participant in actual.prepared.participants() {
        let text::PreparedParticipant::Text(unit) = participant else {
            continue;
        };
        let view = unit.view();
        if view.empty_origin().is_none()
            && view.source_range().is_empty()
            && view.processed_range().is_empty()
        {
            assert!(
                text_origin_incidence(&unit, &actual.origin(1), 0, None)
                    .unwrap()
                    .is_none()
            );
            generated += 1;
        }
    }
    assert_eq!(generated, 1);
}

#[test]
fn nonzero_local_range_rejects_outside_endpoints_without_losing_membership() {
    let actual = crate::test_support::inline_endpoint::Fixture::new(
        "AB",
        (1, 2),
        text::WhiteSpace::Preserve,
        false,
        false,
    );
    let origin = actual
        .binding
        .origins
        .iter()
        .find(|row| row.origin.view().local_range().start == 1)
        .unwrap()
        .origin
        .clone();
    let mut seen = 0;
    for participant in actual.binding.prepared.participants() {
        let text::PreparedParticipant::Text(unit) = participant else {
            continue;
        };
        let Some(at_start) = text_origin_incidence(&unit, &origin, 1, None).unwrap() else {
            continue;
        };
        assert!(at_start.represented);
        let at_end = text_origin_incidence(&unit, &origin, 2, None)
            .unwrap()
            .unwrap();
        assert!(at_end.represented);
        for outside in [0, 3] {
            let incidence = text_origin_incidence(&unit, &origin, outside, None)
                .unwrap()
                .unwrap();
            assert!(!incidence.represented);
            assert!(!incidence.inside);
        }
        seen += 1;
    }
    assert_eq!(seen, 1);
}
