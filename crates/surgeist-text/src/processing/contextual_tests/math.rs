use super::*;

fn math(capture: &mut ContextualSourceInput, scope: usize) {
    let SourceProcessingScope::Block { base_style, .. } = &mut capture.scopes[scope] else {
        panic!("actual block")
    };
    base_style.text_transform = TextTransform::MathAuto;
}
fn fact(capture: &mut ContextualSourceInput, index: usize) {
    let row = &mut capture.origins[index];
    row.text_node =
        Some(CompleteTextNodeContribution::new(row.source.text().clone(), row.local).unwrap());
}
fn reason(error: &Error) -> &TextTransformError {
    match error.detail() {
        Some(ErrorDetail::TextTransform { reason }) => reason,
        _ => panic!("{error:?}"),
    }
}

#[test]
fn contextual_math_demands_actual_domain_winners_before_discard_and_trim() {
    for mode in [WhiteSpace::Discard, WhiteSpace::Preserve] {
        let mut i = input(" c ", mode);
        math(&mut i, 0);
        i.scopes.push(SourceProcessingScope::Inline {
            style: None,
            boundaries: None,
            trim: WhiteSpaceTrim {
                discard_inner: true,
                ..WhiteSpaceTrim::default()
            },
        });
        i.events.insert(2, SourceProcessingEvent::EnterScope(1));
        i.events.insert(4, SourceProcessingEvent::ExitScope(1));
        let e = process_contextual_source(i.clone()).unwrap_err();
        assert_eq!(
            reason(&e),
            &TextTransformError::MissingTextNodeContext { origin: 0 }
        );
        fact(&mut i, 0);
        let processed = Arc::new(process_contextual_source(i).unwrap());
        let a = AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
            .unwrap();
        assert_eq!(a.inner.projection.text, "c");
        assert_eq!(a.source().text(), &Text::from(" c "));
    }
    let mut i = input("c", WhiteSpace::Preserve);
    math(&mut i, 0);
    fact(&mut i, 0);
    let processed = Arc::new(process_contextual_source(i).unwrap());
    let a =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
    assert_eq!(a.inner.projection.text, "𝑐");
    assert_eq!(
        a.inner.transform.original_pieces(0)[0].local,
        Range::new(0, 1)
    );
}

#[test]
fn contextual_holes_retain_front_origin_indices_and_do_not_borrow_parent_domain_style() {
    let mut i = hole(InlineBoxKind::OutOfFlow);
    math(&mut i, 0);
    assert_eq!(
        reason(&process_contextual_source(i.clone()).unwrap_err()),
        &TextTransformError::MissingTextNodeContext { origin: 0 }
    );
    fact(&mut i, 0);
    assert_eq!(
        reason(&process_contextual_source(i.clone()).unwrap_err()),
        &TextTransformError::MissingTextNodeContext { origin: 2 }
    );
    fact(&mut i, 2);
    let processed = Arc::new(process_contextual_source(i.clone()).unwrap());
    let views: Vec<_> = processed.paragraphs().collect();
    let parent = AnalyzedSource::from_processed_paragraph(views[0].clone()).unwrap();
    let child = AnalyzedSource::from_processed_paragraph(views[1].clone()).unwrap();
    assert_eq!(parent.inner.projection.text, "A B"); // complete original node is multiscalar
    assert_eq!(child.inner.projection.text, " I "); // no parent math demand on child
    assert_eq!(
        views[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(6, 9)]
    );
    i.source.span(
        Range::new(3, 6),
        Style {
            text_transform: TextTransform::MathAuto,
            ..style(WhiteSpace::Preserve, 20.0)
        },
    );
    assert_eq!(
        reason(&process_contextual_source(i.clone()).unwrap_err()),
        &TextTransformError::MissingTextNodeContext { origin: 1 }
    );
    fact(&mut i, 1);
    assert!(process_contextual_source(i).is_ok());
}

#[test]
fn contextual_node_mismatch_is_distinct_and_empty_origins_have_no_text_demand() {
    let mut i = input("c", WhiteSpace::Preserve);
    i.origins[0].text_node =
        Some(CompleteTextNodeContribution::new(Text::from("A"), Range::new(0, 1)).unwrap());
    let e = process_contextual_source(i).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidSourceProcessing);
    assert_eq!(
        reason(&e),
        &TextTransformError::TextNodeMismatch { origin: 0 }
    );
    let mut i = input("", WhiteSpace::Preserve);
    math(&mut i, 0);
    assert!(process_contextual_source(i.clone()).is_ok());
    i.origins[0].text_node =
        Some(CompleteTextNodeContribution::new(Text::from("c"), Range::new(0, 1)).unwrap());
    assert_eq!(
        reason(&process_contextual_source(i).unwrap_err()),
        &TextTransformError::TextNodeMismatch { origin: 0 }
    );
}

#[test]
fn contextual_aggregate_winner_overrides_both_domain_base_and_resource_styles() {
    let mut i = input("c", WhiteSpace::Preserve);
    math(&mut i, 0);
    i.origins[0].source.span(
        Range::new(0, 1),
        Style {
            text_transform: TextTransform::MathAuto,
            ..style(WhiteSpace::Preserve, 10.0)
        },
    );
    i.source.span(
        Range::new(0, 1),
        Style {
            text_transform: TextTransform::MathAuto,
            ..style(WhiteSpace::Preserve, 10.0)
        },
    );
    i.source
        .span(Range::new(0, 1), style(WhiteSpace::Preserve, 10.0));
    i.source.span(
        Range::new(0, 0),
        Style {
            text_transform: TextTransform::MathAuto,
            ..style(WhiteSpace::Preserve, 10.0)
        },
    );
    let processed = Arc::new(process_contextual_source(i).unwrap());
    let a =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
    assert_eq!(a.inner.projection.text, "c");
}
