use crate::transform::relation::RelationEvent;
use crate::{AnalyzedSource, CaseTransform, Range, Source, Style, Text, TextTransform};

fn style(case: CaseTransform, locale: Option<&str>) -> Style {
    Style {
        text_transform: TextTransform::Set(crate::TextTransformSet::none().with_case(case)),
        locale: locale.map(str::to_owned),
        ..Style::default()
    }
}

#[test]
fn actual_transform_phase_executes_full_contextual_and_initial_unit_literals() {
    use CaseTransform::{Capitalize, Lowercase, Uppercase};
    for (input, case, locale, expected) in [
        ("ABc", Lowercase, None, "abc"),
        ("ABc", Capitalize, None, "ABc"),
        ("abc DEF", Capitalize, None, "Abc DEF"),
        ("\u{600}abc def", Capitalize, None, "\u{600}Abc Def"),
        ("Straße", Uppercase, None, "STRASSE"),
        ("ßeta", Capitalize, None, "Sseta"),
        ("ΐ", Uppercase, None, "Ι\u{308}\u{301}"),
        ("İ", Lowercase, None, "i\u{307}"),
        ("ΟΣ", Lowercase, None, "ος"),
        ("ΟΣΑ", Lowercase, None, "οσα"),
        ("ΟΣ\u{301}Α", Lowercase, None, "οσ\u{301}α"),
        ("ΟΣ\u{301}", Lowercase, None, "ος\u{301}"),
        ("iı", Uppercase, Some("tr-Latn"), "İI"),
        ("Iİ", Lowercase, Some("az-Latn"), "ıi"),
        ("I\u{307}", Lowercase, Some("tr"), "i"),
        ("i", Uppercase, Some("tr-Arab"), "I"),
        ("I\u{301}", Lowercase, Some("lt"), "i\u{307}\u{301}"),
        ("i\u{307}\u{301}", Uppercase, Some("lt"), "I\u{301}"),
        ("άδικος", Uppercase, Some("el"), "ΑΔΙΚΟΣ"),
        ("άι", Uppercase, Some("el"), "ΑΪ"),
        ("ή", Uppercase, Some("el"), "Ή"),
        ("ijSSEL", Capitalize, Some("nl"), "IJSSEL"),
        ("3abc \u{2bb}abc", Capitalize, None, "3abc \u{2bb}abc"),
        ("α\u{313}\u{345}", Capitalize, None, "Α\u{313}\u{345}"),
        ("\u{1f80}", Capitalize, None, "\u{1f88}"),
        ("i\u{307}", Capitalize, Some("lt"), "I"),
        (
            "an t-uisce ar aon tslí i bhfad",
            Uppercase,
            Some("ga"),
            "AN tUISCE AR AON tSLÍ I bhFAD",
        ),
        ("nÓg NÓG", Lowercase, Some("ga"), "n-óg nóg"),
    ] {
        let source = Source::new(input);
        let analyzed = AnalyzedSource::new(source.clone(), style(case, locale)).unwrap();
        assert_eq!(
            analyzed.inner.projection.text, expected,
            "{input:?}/{case:?}/{locale:?}"
        );
        assert_eq!(analyzed.source(), &source);
        assert_eq!(analyzed.processed_source().processed_text(), source.text());
        assert_eq!(analyzed.inner.host_projection.text, input);
    }
}

#[test]
fn actual_component_operations_and_languages_never_borrow_the_neighbors_policy() {
    for (left, right, expected) in [
        (
            style(CaseTransform::Capitalize, Some("nl")),
            style(CaseTransform::None, Some("nl")),
            "Ij",
        ),
        (
            style(CaseTransform::None, Some("nl")),
            style(CaseTransform::Capitalize, Some("nl")),
            "iJ",
        ),
        (
            style(CaseTransform::Capitalize, None),
            style(CaseTransform::Capitalize, Some("nl")),
            "IJ",
        ),
        (
            style(CaseTransform::Capitalize, Some("nl")),
            style(CaseTransform::Capitalize, None),
            "Ij",
        ),
    ] {
        let mut source = Source::new("ij");
        source.span(Range::new(1, 2), right);
        assert_eq!(
            AnalyzedSource::new(source, left)
                .unwrap()
                .inner
                .projection
                .text,
            expected
        );
    }
    for (dot, expected) in [
        (style(CaseTransform::Capitalize, Some("lt")), "I"),
        (style(CaseTransform::None, Some("lt")), "I\u{307}"),
        (style(CaseTransform::Capitalize, None), "I\u{307}"),
    ] {
        let mut source = Source::new("i\u{307}");
        source.span(Range::new(1, 2), dot);
        assert_eq!(
            AnalyzedSource::new(source, style(CaseTransform::Capitalize, Some("lt")))
                .unwrap()
                .inner
                .projection
                .text,
            expected
        );
    }
    for (left, right, expected) in [
        (
            style(CaseTransform::None, None),
            style(CaseTransform::Lowercase, Some("ga")),
            "n-ó",
        ),
        (
            style(CaseTransform::Lowercase, Some("ga")),
            style(CaseTransform::None, None),
            "nÓ",
        ),
        (
            style(CaseTransform::Lowercase, Some("ga")),
            style(CaseTransform::Lowercase, None),
            "nó",
        ),
    ] {
        let mut source = Source::new("nÓ");
        source.span(Range::new(1, 2), right);
        assert_eq!(
            AnalyzedSource::new(source, left)
                .unwrap()
                .inner
                .projection
                .text,
            expected
        );
    }
    let mut source = Source::new("ΟΣΑ");
    source.span(Range::new(1, 2), style(CaseTransform::Lowercase, None));
    assert_eq!(
        AnalyzedSource::new(source, Style::default())
            .unwrap()
            .inner
            .projection
            .text,
        "ΟσΑ"
    );
    let mut source = Source::new("foobar");
    source.span(Range::new(3, 6), style(CaseTransform::Capitalize, None));
    assert_eq!(
        AnalyzedSource::new(source, Style::default())
            .unwrap()
            .inner
            .projection
            .text,
        "foobar"
    );
}

#[test]
fn actual_relation_keeps_reordered_noncontiguous_emissions_and_a_real_removal() {
    let mut source = Source::new("\u{1f80}\u{301}\u{313}");
    source.span(
        Range::new(2, 3),
        style(CaseTransform::Uppercase, Some("el")),
    );
    let analyzed =
        AnalyzedSource::new(source.clone(), style(CaseTransform::Uppercase, None)).unwrap();
    assert_eq!(analyzed.inner.projection.text, "\u{1f08}\u{301}Ι");
    let map = &analyzed.inner.transform;
    assert_eq!(
        map.rows.iter().map(|row| row.source).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    assert_eq!(
        map.atoms
            .iter()
            .map(|&id| (
                map.original_row_for_atom(map.emissions[id].atom)
                    .expect("original casing contribution"),
                map.emissions[id].subposition
            ))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0), (0, 1)]
    );
    assert!(matches!(
        map.events(),
        [
            RelationEvent::Text { atom: 0 },
            RelationEvent::Text { atom: 1 },
            RelationEvent::Removal { row: 2 },
            RelationEvent::Text { atom: 2 }
        ]
    ));
    assert_eq!(map.original_pieces(0)[0].aggregate, Range::new(0, 1));
    assert_eq!(map.original_pieces(2)[0].aggregate, Range::new(2, 3));
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.processed_source().processed_text(), source.text());
}

#[test]
fn opaque_original_units_and_kana_composition_keep_independent_phase_custody() {
    let source = Source::new(Text::from_code_units(vec![0xd800, 0xfffd, 0xdc00, 0xdf]));
    let analyzed =
        AnalyzedSource::new(source.clone(), style(CaseTransform::Uppercase, None)).unwrap();
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd800, 0xfffd, 0xdc00, 0x53, 0x53]
    );
    assert_eq!(analyzed.source(), &source);
    let source = Source::new("ß\u{1b132}");
    let analyzed = AnalyzedSource::new(
        source.clone(),
        Style {
            text_transform: TextTransform::Set(
                crate::TextTransformSet::none()
                    .with_case(CaseTransform::Uppercase)
                    .with_full_size_kana(true),
            ),
            ..Style::default()
        },
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.text, "SSこ");
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.processed)
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(0, 1), Range::new(1, 3)]
    );
    assert_eq!(analyzed.inner.projection.presentation_end(), 3);
    assert_eq!(analyzed.processed_source().processed_text(), source.text());
}

#[test]
fn immutable_context_crosses_connected_origins_and_neutral_oof_but_not_real_domains() {
    for (connection, box_kind, expected) in [
        (crate::ParagraphConnectionKind::Continuous, None, "οσα"),
        (
            crate::ParagraphConnectionKind::Continuous,
            Some(crate::InlineBoxKind::OutOfFlow),
            "οσα",
        ),
        (crate::ParagraphConnectionKind::Disconnected, None, "οςα"),
        (
            crate::ParagraphConnectionKind::Continuous,
            Some(crate::InlineBoxKind::InFlow),
            "οςα",
        ),
    ] {
        let mut source = Source::new("ΟΣΑ");
        if let Some(kind) = box_kind {
            source.inline_box(crate::InlineBox::new(
                crate::Id::from_u64(6305),
                kind,
                2,
                crate::Size::new(0.0, 0.0),
            ));
        }
        let analyzed = AnalyzedSource::from_paragraph(crate::ParagraphInput {
            source: source.clone(),
            base_style: style(CaseTransform::Lowercase, None),
            paragraph_strut: crate::ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: crate::ParagraphBaseDirection::LeftToRight,
            origins: [(0, 2), (2, 3)]
                .into_iter()
                .map(|(start, end)| crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(start, end),
                    aggregate: Range::new(start, end),
                    kind: crate::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![crate::ParagraphConnection {
                at: 2,
                kind: connection,
            }],
            controls: vec![],
            replacements: vec![],
            boundary_policies: (if connection == crate::ParagraphConnectionKind::Continuous {
                vec![crate::ParagraphBoundaryDefinition::Connection(0)]
            } else {
                vec![]
            })
            .into_iter()
            .chain(if box_kind == Some(crate::InlineBoxKind::InFlow) {
                vec![
                    crate::ParagraphBoundaryDefinition::AtomicBefore(0),
                    crate::ParagraphBoundaryDefinition::AtomicAfter(0),
                ]
            } else {
                vec![]
            })
            .map(|definition| crate::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
            anchor_order: if box_kind.is_some() {
                vec![crate::ParagraphAnchor {
                    item: crate::ParagraphAnchorItem::Atomic(0),
                    affinity: crate::Affinity::After,
                }]
            } else {
                vec![]
            },
        })
        .unwrap();
        assert_eq!(analyzed.inner.projection.text, expected);
        assert_eq!(analyzed.source(), &source);
        assert_eq!(analyzed.processed_source().processed_text(), source.text());
    }
    let analyzed =
        AnalyzedSource::new(Source::new("ΟΣ\nΑ"), style(CaseTransform::Lowercase, None)).unwrap();
    assert_eq!(analyzed.inner.projection.text, "ος\nα");
}

#[test]
fn split_origin_dot_and_acute_use_actual_context_through_neutral_oof() {
    for (mark, locale, expected, rows) in [
        ("\u{307}", "tr", "i", vec![0]),
        ("\u{301}", "lt", "i\u{307}\u{301}", vec![0, 0, 1]),
    ] {
        for oof in [false, true] {
            let mut source = Source::new(format!("I{mark}"));
            if oof {
                source.inline_box(crate::InlineBox::new(
                    crate::Id::from_u64(6320),
                    crate::InlineBoxKind::OutOfFlow,
                    1,
                    crate::Size::new(0.0, 0.0),
                ));
            }
            let analyzed = AnalyzedSource::from_paragraph(crate::ParagraphInput {
                source: source.clone(),
                base_style: style(CaseTransform::Lowercase, Some(locale)),
                paragraph_strut: crate::ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: crate::ParagraphBaseDirection::LeftToRight,
                origins: [Source::new("I"), Source::new(mark)]
                    .into_iter()
                    .enumerate()
                    .map(|(index, original)| crate::ParagraphOrigin {
                        text_node: None,
                        source: original,
                        local: Range::new(0, 1),
                        aggregate: Range::new(index, index + 1),
                        kind: crate::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: vec![crate::ParagraphConnection {
                    at: 1,
                    kind: crate::ParagraphConnectionKind::Continuous,
                }],
                controls: vec![],
                replacements: vec![],
                boundary_policies: vec![crate::ParagraphBoundaryPolicy {
                    definition: crate::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                }],
                anchor_order: if oof {
                    vec![crate::ParagraphAnchor {
                        item: crate::ParagraphAnchorItem::Atomic(0),
                        affinity: crate::Affinity::After,
                    }]
                } else {
                    vec![]
                },
            })
            .unwrap();
            assert_eq!(analyzed.inner.projection.text, expected);
            let map = &analyzed.inner.transform;
            assert_eq!(
                map.atoms
                    .iter()
                    .map(|&id| map
                        .original_row_for_atom(map.emissions[id].atom)
                        .expect("original casing contribution"))
                    .collect::<Vec<_>>(),
                rows
            );
            for row in 0..2 {
                let piece = &map.original_pieces(row)[0];
                assert_eq!(
                    (piece.origin, piece.local, piece.aggregate),
                    (row, Range::new(0, 1), Range::new(row, row + 1))
                );
            }
            assert_eq!(analyzed.source(), &source);
            assert_eq!(analyzed.processed_source().processed_text(), source.text());
            if locale == "tr" {
                assert!(
                    map.events()
                        .iter()
                        .any(|event| matches!(event, RelationEvent::Removal { row: 1 }))
                );
            }
        }
    }
}
