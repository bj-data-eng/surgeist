//! Functional native policy/group/resource qualification, independent of emitted glyph guesses.
use super::*;
use crate::{
    HyphenateCharacter, HyphenationError, HyphenationPlacement as Placement, Hyphens,
    NativeHyphenationConvention as Convention, ParagraphControl, ParagraphControlKind,
    ParagraphReplacement, ParagraphReplacementDefinition as Definition, Range, Text, Wrap,
};

fn input(text: &str, base_style: Style) -> ParagraphInput {
    let source = Source::new(text);
    let len = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::AllocatedContributions,
        origins: vec![paragraph::ParagraphOrigin {
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if len == 0 {
            vec![crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::Before,
            }]
        } else {
            vec![]
        },
    }
}
fn capture(system: &mut System, value: ParagraphInput) -> crate::Result<PreparedText> {
    let analyzed = AnalyzedSource::from_paragraph(value)?;
    system.prepare(
        &crate::test_wrapping::bind_missing(&analyzed, Wrap::Word),
        PreparationOptions::default(),
    )
}
fn texts(p: &PreparedText) -> Vec<Vec<u16>> {
    p.alternative_views()
        .map(|v| v.text().code_units().to_vec())
        .collect()
}
fn reason(error: &crate::Error) -> HyphenationError {
    match error.detail() {
        Some(crate::ErrorDetail::Hyphenation { reason }) => *reason,
        detail => panic!("wrong error: {detail:?}"),
    }
}
fn convention(text: &str, placement: Placement) -> Convention {
    Convention::language_script("en-Latn-x-native", "Latn", Text::from(text), placement).unwrap()
}
fn bound_style() -> Style {
    Style {
        locale: Some("en-Latn-x-native".into()),
        ..style()
    }
}
fn control(value: &mut ParagraphInput, own: Style) {
    value.controls.push(ParagraphControl {
        at: if value.source.text().len() == 5 {
            2
        } else {
            value.source.text().len()
        },
        kind: ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(own),
        },
    });
    value.anchor_order.push(crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
}
fn finish(p: &PreparedText, alternative: usize) -> FinalizedLine {
    let alt = p.alternative_views().nth(alternative).unwrap();
    let offer = alt.opportunity();
    let mut r = request(p);
    r.end = offer.view().boundary();
    r.termination = PreparedTermination::Ordinary(offer);
    r.alternative = Some(alt.handle());
    r.context.end = LineEnd::Soft;
    r.participants = p
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    r.controls = p
        .control_views()
        .map(|v| ControlAllocation {
            occurrence: v.handle(),
            strut: None,
        })
        .collect();
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

#[test]
fn none_suppresses_both_shy_sources_but_preserves_visible_hyphen_and_optional_control() {
    let mut system = system();
    let mut v = input(
        "A\u{ad}X-Y",
        Style {
            hyphens: Hyphens::None,
            ..style()
        },
    );
    v.replacements.push(
        ParagraphReplacement::resolved_native_logical_before(
            Definition::Source {
                origin: 0,
                source: Range::new(1, 2),
            },
            2,
            vec![Text::from("XX")],
        )
        .unwrap(),
    );
    v.controls.push(ParagraphControl {
        at: 3,
        kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
    });
    v.anchor_order.push(crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
    let p = capture(&mut system, v).unwrap();
    assert!(texts(&p).is_empty());
    let offers: Vec<_> = p
        .opportunity_views()
        .map(|v| (v.boundary().view().source_before().get(), v.kind()))
        .collect();
    assert!(!offers.iter().any(|(at, _)| *at == 2));
    assert!(offers.contains(&(3, PreparedBreakKind::Allowed)));
    assert!(offers.contains(&(4, PreparedBreakKind::Allowed)));
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 173, 88, 45, 89]
    );
}
#[test]
fn manual_auto_and_none_do_not_invent_dictionary_points() {
    let mut system = system();
    for mode in [Hyphens::None, Hyphens::Manual, Hyphens::Auto] {
        let p = capture(
            &mut system,
            input(
                "ABC",
                Style {
                    hyphens: mode,
                    ..style()
                },
            ),
        )
        .unwrap();
        assert_eq!(
            p.opportunity_views().map(|v| v.kind()).collect::<Vec<_>>(),
            [PreparedBreakKind::ParagraphEnd]
        );
    }
}
#[test]
fn exact_style_strings_including_empty_are_real_selected_endings() {
    let mut system = system();
    for text in ["", "XX", "\u{2069}"] {
        let p = capture(
            &mut system,
            input(
                "AX\u{ad}XXX",
                Style {
                    hyphenate_character: HyphenateCharacter::String(Text::from(text)),
                    ..style()
                },
            ),
        )
        .unwrap();
        assert_eq!(texts(&p), [text.encode_utf16().collect::<Vec<_>>()]);
        let line = finish(&p, 0);
        assert_eq!(
            line.feedback().advance,
            if text == "XX" { 80.0 } else { 40.0 }
        );
        assert!(line.selected_line().request().alternative.is_some());
        assert_eq!(line.selected_line().request().context.end, LineEnd::Soft);
    }
}
#[test]
fn convention_binding_preserves_full_locale_and_canonical_script() {
    let c = Convention::language_script(
        "EN_latn_US_x_Foo",
        "lAtN",
        Text::from("~"),
        Placement::LogicalBefore,
    )
    .unwrap();
    assert_eq!(c.locale(), Some("EN_latn_US_x_Foo"));
    assert_eq!(c.script(), "Latn");
    for locale in ["en-Latn-us-x-foo", "EN_LATN_US_X_FOO"] {
        crate::ValidatedStyle::try_from(Style {
            locale: Some(locale.into()),
            native_hyphenation_convention: Some(c.clone()),
            ..style()
        })
        .unwrap();
    }
    for locale in [
        None,
        Some("en-Latn-US-x-bar"),
        Some("en-Latn-US"),
        Some("en-Cyrl-US-x-foo"),
    ] {
        let e = crate::ValidatedStyle::try_from(Style {
            locale: locale.map(str::to_owned),
            native_hyphenation_convention: Some(c.clone()),
            ..style()
        })
        .unwrap_err();
        assert_eq!(reason(&e), HyphenationError::InvalidConventionBinding);
    }
    let declared =
        Convention::declared_script("Latn", Text::from(""), Placement::LogicalBefore).unwrap();
    crate::ValidatedStyle::try_from(Style {
        locale: Some("und-Latn".into()),
        native_hyphenation_convention: Some(declared.clone()),
        ..style()
    })
    .unwrap();
    for locale in ["en", "und-Cyrl"] {
        assert_eq!(
            reason(
                &crate::ValidatedStyle::try_from(Style {
                    locale: Some(locale.into()),
                    native_hyphenation_convention: Some(declared.clone()),
                    ..style()
                })
                .unwrap_err()
            ),
            HyphenationError::InvalidConventionBinding
        );
    }
    for script in ["Zzzz", "Zyyy", "Zinh", "Latin", "Lat1"] {
        assert!(
            Convention::declared_script(script, Text::from(""), Placement::LogicalBefore).is_err()
        );
    }
    assert!(
        Convention::language_script("en-Cyrl", "Latn", Text::from(""), Placement::LogicalBefore)
            .is_err()
    );
}
#[test]
fn convention_text_placement_and_native_group_have_distinct_authority() {
    let mut system = system();
    for text in ["", "~"] {
        let p = capture(
            &mut system,
            input(
                "AX\u{ad}XXX",
                Style {
                    native_hyphenation_convention: Some(convention(text, Placement::LogicalBefore)),
                    ..bound_style()
                },
            ),
        )
        .unwrap();
        assert_eq!(texts(&p), [text.encode_utf16().collect::<Vec<_>>()]);
    }
    for placement in [Placement::LogicalAfter, Placement::BothSides] {
        let base = Style {
            native_hyphenation_convention: Some(convention("~", placement)),
            ..bound_style()
        };
        if placement == Placement::BothSides {
            assert_eq!(
                reason(&capture(&mut system, input("AX\u{ad}XXX", base.clone())).unwrap_err()),
                HyphenationError::UnsupportedPlacement(placement)
            );
        } else {
            // LogicalAfter is now a real continuation choice; it emits nothing
            // on this ending while preserving its actual convention string.
            let p = capture(&mut system, input("AX\u{ad}XXX", base.clone())).unwrap();
            assert_eq!(texts(&p), [vec![126]]);
            let line = finish(&p, 0);
            assert_eq!(line.feedback().advance, 40.0);
            assert!(line.hyphenation_continuation().is_some());
        }
        let empty = Style {
            hyphenate_character: HyphenateCharacter::String(Text::from("")),
            ..base.clone()
        };
        assert_eq!(
            texts(&capture(&mut system, input("AX\u{ad}XXX", empty)).unwrap()),
            [Vec::<u16>::new()]
        );
        let mut native = input("AX\u{ad}XXX", base);
        native.replacements.push(
            ParagraphReplacement::resolved_native_logical_before(
                Definition::Source {
                    origin: 0,
                    source: Range::new(2, 3),
                },
                3,
                vec![Text::from("XX"), Text::from(""), Text::from("X")],
            )
            .unwrap(),
        );
        let p = capture(&mut system, native).unwrap();
        assert_eq!(texts(&p), [vec![88, 88], vec![], vec![88]]);
        assert_eq!(
            [
                finish(&p, 0).feedback().advance,
                finish(&p, 1).feedback().advance,
                finish(&p, 2).feedback().advance
            ],
            [80.0, 40.0, 60.0]
        );
    }
}
#[test]
fn control_uses_own_style_and_auto_primary_is_not_borrowed_from_adjacent_run() {
    let mut system = system();
    let own = Style {
        font: Font::new().family("Missing owning primary"),
        ..style()
    };
    let mut v = input("AXXXX", style());
    control(&mut v, own.clone());
    assert_eq!(
        reason(&capture(&mut system, v).unwrap_err()),
        HyphenationError::MissingPrimaryFont
    );
    let mut v = input("AXXXX", style());
    control(
        &mut v,
        Style {
            hyphenate_character: HyphenateCharacter::String(Text::from("")),
            ..own.clone()
        },
    );
    let p = capture(&mut system, v).unwrap();
    assert_eq!(finish(&p, 0).feedback().advance, 40.0);
    let mut v = input("AXXXX", style());
    control(
        &mut v,
        Style {
            hyphens: Hyphens::None,
            ..own
        },
    );
    assert!(texts(&capture(&mut system, v).unwrap()).is_empty());
    let mut v = input("AXXXX", style());
    control(
        &mut v,
        Style {
            size: 10.0,
            hyphenate_character: HyphenateCharacter::String(Text::from("X")),
            ..style()
        },
    );
    let p = capture(&mut system, v).unwrap();
    let line = finish(&p, 0);
    assert_eq!(line.feedback().advance, 50.0);
    let sizes: Vec<_> = line
        .run_views()
        .filter(|r| r.glyphs().any(|g| g.advance() != 0.0))
        .map(|r| r.font_size())
        .collect();
    assert_eq!(sizes, [20.0, 10.0]);
}
#[test]
fn actual_primary_without_space_glyph_still_selects_its_u2010() {
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[35, 35],
        &[(0x41, 35), (0xad, 3), (0x58, 58), (0x2010, 36), (0x2d, 15)],
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let p = capture(&mut system, input("AX\u{ad}XXX", style())).unwrap();
    assert_eq!(texts(&p), [vec![0x2010]]);
}
#[test]
fn repeated_genuine_owner_groups_and_empty_native_list_are_rejected() {
    assert_eq!(
        reason(
            &ParagraphReplacement::resolved_native_logical_before(
                Definition::Control {
                    control: 0,
                    origin: None
                },
                1,
                vec![]
            )
            .unwrap_err()
        ),
        HyphenationError::EmptyNativeAlternatives
    );
    let mut v = input("A", style());
    control(&mut v, style());
    v.replacements = vec![
        ParagraphReplacement::style_driven(
            Definition::Control {
                control: 0,
                origin: None,
            },
            1,
        ),
        ParagraphReplacement::style_driven(
            Definition::Control {
                control: 0,
                origin: Some(0),
            },
            1,
        ),
    ];
    assert!(matches!(
        super::reason(&AnalyzedSource::from_paragraph(v).unwrap_err()),
        ProtocolError::DuplicateReplacement {
            alternative: 1,
            prior: 0
        }
    ));
    let mut v = input("A\u{ad}X", style());
    let group = ParagraphReplacement::style_driven(
        Definition::Source {
            origin: 0,
            source: Range::new(1, 2),
        },
        2,
    );
    v.replacements = vec![group.clone(), group];
    assert!(matches!(
        super::reason(&AnalyzedSource::from_paragraph(v).unwrap_err()),
        ProtocolError::DuplicateReplacement {
            alternative: 1,
            prior: 0
        }
    ));
}
#[test]
fn full_hyphenation_facts_change_style_cache_identity() {
    let source = crate::ValidatedSource::try_from(Source::new("A")).unwrap();
    let system = system();
    let key = |s: Style| {
        crate::Key::from_validated(
            &source,
            &crate::ValidatedStyle::try_from(s).unwrap(),
            crate::ValidatedOptions::try_from(crate::Options::default()).unwrap(),
            system.font_catalogue_identity(),
        )
    };
    let base = key(bound_style());
    assert_ne!(
        base,
        key(Style {
            hyphens: Hyphens::None,
            ..bound_style()
        })
    );
    assert_ne!(
        base,
        key(Style {
            hyphenate_character: HyphenateCharacter::String(Text::from("")),
            ..bound_style()
        })
    );
    assert_ne!(
        base,
        key(Style {
            native_hyphenation_convention: Some(convention("", Placement::LogicalBefore)),
            ..bound_style()
        })
    );
    assert_ne!(
        key(Style {
            native_hyphenation_convention: Some(convention("", Placement::LogicalBefore)),
            ..bound_style()
        }),
        key(Style {
            native_hyphenation_convention: Some(convention("", Placement::BothSides)),
            ..bound_style()
        })
    );
}

#[test]
fn source_shy_uses_own_primary_before_an_adjacent_face_with_u2010() {
    let primary = crate::shaping_test_fonts::range_ligature_ahem(
        &[35, 35],
        &[(0x41, 35), (0xad, 3), (0x58, 58), (0x2d, 15)],
    );
    let secondary = crate::shaping_test_fonts::range_ligature_ahem(
        &[35, 35],
        &[(0x41, 35), (0xad, 3), (0x58, 58), (0x2010, 36), (0x2d, 15)],
    );
    let font = skrifa::FontRef::new(&secondary).unwrap();
    let mut name = font
        .table_data(skrifa::Tag::new(b"name"))
        .unwrap()
        .as_bytes()
        .to_vec();
    for i in 0..name.len().saturating_sub(7) {
        if &name[i..i + 8] == b"\0A\0h\0e\0m" {
            name[i + 1] = b'B';
        }
    }
    let secondary = crate::test_fonts::replace_tables(&secondary, vec![(*b"name", name)]);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(primary).unwrap();
    system.register_fonts(secondary).unwrap();
    let mut v = input(
        "AX\u{ad}XXX",
        Style {
            font: Font::new().family("Bhem"),
            ..style()
        },
    );
    v.source.span(Range::new(2, 3), style());
    v.origins[0].source = v.source.clone();
    let p = capture(&mut system, v).unwrap();
    assert_eq!(texts(&p), [vec![45]]);
    let before = p.font_catalogue_identity().clone();
    system.refresh_fonts().unwrap();
    assert_ne!(system.font_catalogue_identity(), &before);
    let line = finish(&p, 0);
    assert_eq!(line.feedback().advance, 60.0);
    assert!(
        line.run_views()
            .all(|r| r.face().catalogue_identity() == &before)
    );
}

#[test]
fn specified_text_wins_and_suppressed_points_do_not_consume_known_placement() {
    let mut system = system();
    let convention_style = Style {
        native_hyphenation_convention: Some(convention("~", Placement::LogicalBefore)),
        hyphenate_character: HyphenateCharacter::String(Text::from("XX")),
        ..bound_style()
    };
    assert_eq!(
        texts(&capture(&mut system, input("AX\u{ad}XXX", convention_style)).unwrap()),
        [vec![88, 88]]
    );
    for (hyphens, wrap) in [(Hyphens::None, Wrap::Word), (Hyphens::Manual, Wrap::None)] {
        let base = Style {
            hyphens,
            wrap,
            native_hyphenation_convention: Some(convention("~", Placement::BothSides)),
            ..bound_style()
        };
        assert!(texts(&capture(&mut system, input("AX\u{ad}XXX", base)).unwrap()).is_empty());
    }
    let base = Style {
        native_hyphenation_convention: Some(convention("~", Placement::BothSides)),
        ..bound_style()
    };
    let p = capture(&mut system, input("A\u{ad}\nX", base)).unwrap();
    assert!(texts(&p).is_empty());
    assert!(
        p.opportunity_views()
            .any(|v| v.kind() == PreparedBreakKind::Forced)
    );
}

#[test]
fn raw_exact_empty_long_and_none_use_actual_finalized_indication_advance() {
    let mut system = system();
    let source = Source::new("X\u{ad}XX");
    let options = crate::Options {
        width: Some(40.0),
        quantize: false,
        ..crate::Options::default()
    };
    for (text, expected) in [
        ("", vec![20.0, 40.0]),
        ("X", vec![40.0, 40.0]),
        ("XXX", vec![80.0, 40.0]),
    ] {
        let own = Style {
            hyphenate_character: HyphenateCharacter::String(Text::from(text)),
            ..style()
        };
        let p = system.layout(source.clone(), own.clone(), options).unwrap();
        assert_eq!(
            p.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 4)]
        );
        assert_eq!(raw_advances(&p), expected);
        assert_eq!(p.metrics().overflow(), text == "XXX");
        let whole = system
            .layout(
                source.clone(),
                own,
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(whole.lines().len(), 1);
        assert_eq!(whole.metrics().full_width(), 60.0);
    }
    let specified = Style {
        hyphenate_character: HyphenateCharacter::String(Text::from("X")),
        ..style()
    };
    let allowed = system
        .layout(source.clone(), specified.clone(), options)
        .unwrap();
    let none = system
        .layout(
            source.clone(),
            Style {
                hyphens: Hyphens::None,
                ..specified.clone()
            },
            options,
        )
        .unwrap();
    assert_eq!(none.lines().len(), 1);
    assert_eq!(none.metrics().full_width(), 60.0);
    assert!(none.metrics().overflow());
    assert_ne!(none.key(), allowed.key());
    let again = system.layout(source, specified, options).unwrap();
    assert_eq!(again.key(), allowed.key());
    assert_eq!(again.lines().len(), 2);
}
#[test]
fn raw_candidate_retreat_refinalizes_the_real_earlier_shy_without_losing_continuation_custody() {
    let mut system = system();
    let p = system
        .layout(
            Source::new("X\u{ad}XX\u{ad}XX"),
            Style {
                hyphenate_character: HyphenateCharacter::String(Text::from("X")),
                ..style()
            },
            crate::Options {
                width: Some(60.0),
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    // The nominal first prefix through the second SHY is 60, but its actual
    // selected indication makes 80. The first SHY really fits at 40 instead.
    assert_eq!(
        p.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 5), Range::new(5, 7)]
    );
    assert_eq!(raw_advances(&p), [40.0, 60.0, 40.0]);
    assert_eq!(
        p.source().text().code_units(),
        [88, 173, 88, 88, 173, 88, 88]
    );
    assert!(!p.metrics().overflow());
}

fn raw_advances(layout: &crate::Layout) -> Vec<f64> {
    let mut values = vec![0.0; layout.lines().len()];
    for run in layout.inline_metric_facts().runs() {
        values[run.line()] += run.advance();
    }
    values
}

#[test]
fn incident_empty_origin_cannot_own_a_positive_source_shy_native_group() {
    let mut system = system();
    for leading_empty in [false, true] {
        let mut v = input("AX\u{ad}XXX", style());
        v.paragraph_strut = ParagraphStrut::DefaultFont;
        let positive = usize::from(leading_empty);
        if leading_empty {
            v.origins.insert(
                0,
                paragraph::ParagraphOrigin {
                    source: Source::new(""),
                    local: Range::new(0, 0),
                    aggregate: Range::new(0, 0),
                    kind: paragraph::ParagraphOriginKind::Authored,
                },
            );
            v.connections.push(paragraph::ParagraphConnection {
                at: 0,
                kind: paragraph::ParagraphConnectionKind::Continuous,
            });
            v.anchor_order.push(crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::Before,
            });
        }
        v.boundary_policies = (0..v.connections.len())
            .map(|i| crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .collect();
        v.replacements.push(
            ParagraphReplacement::resolved_native_logical_before(
                Definition::Source {
                    origin: positive,
                    source: Range::new(2, 3),
                },
                3,
                vec![Text::from("XX"), Text::from("")],
            )
            .unwrap(),
        );
        let p = capture(&mut system, v).unwrap();
        // This is the exact supplied native list, never an inferred auto string.
        assert_eq!(
            texts(&p),
            [vec![88, 88], vec![]],
            "leading empty: {leading_empty}"
        );
        let origin = p.origin_views().nth(positive).unwrap().handle();
        assert!(p.alternative_views().all(|a| a.definition()
            == crate::PreparedAlternativeDefinition::Source {
                origin: origin.clone(),
                source: Range::new(2, 3)
            }));
        assert_eq!(finish(&p, 0).feedback().advance, 80.0);
        assert_eq!(finish(&p, 1).feedback().advance, 40.0);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [65, 88, 173, 88, 88, 88]
        );
    }
}
fn empty_after_shy_input(mode: Hyphens) -> ParagraphInput {
    let mut v = input(
        "AX\u{ad}XXX",
        Style {
            hyphens: mode,
            hyphenate_character: HyphenateCharacter::String(Text::from("XX")),
            ..style()
        },
    );
    let source = v.source.clone();
    v.origins = vec![
        paragraph::ParagraphOrigin {
            source: source.clone(),
            local: Range::new(0, 3),
            aggregate: Range::new(0, 3),
            kind: paragraph::ParagraphOriginKind::Authored,
        },
        paragraph::ParagraphOrigin {
            source: Source::new(""),
            local: Range::new(0, 0),
            aggregate: Range::new(3, 3),
            kind: paragraph::ParagraphOriginKind::Authored,
        },
        paragraph::ParagraphOrigin {
            source,
            local: Range::new(3, 6),
            aggregate: Range::new(3, 6),
            kind: paragraph::ParagraphOriginKind::Authored,
        },
    ];
    v.connections = vec![
        paragraph::ParagraphConnection {
            at: 3,
            kind: paragraph::ParagraphConnectionKind::Continuous
        };
        2
    ];
    v.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::EmptyOrigin(1),
        affinity: crate::Affinity::Before,
    }];
    v.boundary_policies = (0..v.connections.len())
        .map(|i| crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    v
}
fn empty_after_shy(mode: Hyphens) -> PreparedText {
    capture(&mut system(), empty_after_shy_input(mode)).unwrap()
}
#[test]
fn none_cannot_leak_shy_icu_permission_after_an_empty_origin() {
    let p = empty_after_shy(Hyphens::None);
    let offers = p
        .opportunity_views()
        .filter(|v| v.boundary().view().source_before().get() == 3)
        .map(|v| v.kind())
        .collect::<Vec<_>>();
    assert!(offers.is_empty(), "none emitted {offers:?} at the SHY seam");
}
#[test]
fn manual_cannot_turn_a_shy_into_an_unindicated_break_after_an_empty_origin() {
    let p = empty_after_shy(Hyphens::Manual);
    let offers = p
        .opportunity_views()
        .filter(|v| v.boundary().view().source_before().get() == 3)
        .collect::<Vec<_>>();
    assert!(!offers.is_empty());
    for offer in offers {
        assert_eq!(
            offer.kind(),
            PreparedBreakKind::Discretionary,
            "an incidental empty origin cannot own generic ICU permission"
        );
        assert_eq!(
            offer
                .alternatives()
                .map(|a| a.text().code_units().to_vec())
                .collect::<Vec<_>>(),
            [vec![88, 88]]
        );
    }
    let empty = PreparedParticipant::Text(
        p.unit_views()
            .find(|unit| unit.empty_origin().is_some())
            .unwrap()
            .handle(),
    );
    let first = finish(&p, 0);
    assert_eq!(first.feedback().advance, 80.0);
    assert!(
        !first
            .selected_line()
            .request()
            .participants
            .contains(&empty)
    );
    let mut r = request(&p);
    r.start = first.selected_line().request().end.clone();
    r.context.start = LineStart::Continuation;
    r.participants = p
        .participants()
        .skip(r.start.view().participant_offset())
        .collect();
    assert!(r.participants.contains(&empty));
    let second = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(second.feedback().advance, 60.0);
    assert_eq!(
        second.selected_line().request().start,
        first.selected_line().request().end
    );
}

#[test]
fn independent_optional_and_forced_controls_after_a_shy_keep_their_own_ending() {
    for (kind, expected) in [
        (
            ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            PreparedBreakKind::Allowed,
        ),
        (ParagraphControlKind::ForcedBreak, PreparedBreakKind::Forced),
    ] {
        let mut v = empty_after_shy_input(Hyphens::None);
        v.controls.push(ParagraphControl { at: 3, kind });
        v.anchor_order.push(crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(0),
            affinity: crate::Affinity::After,
        });
        let p = capture(&mut system(), v).unwrap();
        let control = PreparedParticipant::Control(p.control_views().next().unwrap().handle());
        let offers = p
            .opportunity_views()
            .filter(|v| v.boundary().view().source_before().get() == 3)
            .collect::<Vec<_>>();
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].kind(), expected);
        assert_eq!(
            p.participants()
                .nth(offers[0].boundary().view().participant_offset() - 1),
            Some(control)
        );
    }
}

#[test]
fn raw_atomic_after_a_shy_keeps_its_independent_none_and_manual_permission() {
    for mode in [Hyphens::None, Hyphens::Manual] {
        let mut source = Source::new("A\u{ad}XX");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(86072),
            crate::InlineBoxKind::InFlow,
            2,
            crate::Size::new(10.0, 10.0),
        ));
        let p = system()
            .layout(
                source,
                Style {
                    hyphens: mode,
                    hyphenate_character: HyphenateCharacter::String(Text::from("XXX")),
                    ..style()
                },
                crate::Options {
                    width: Some(40.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(raw_advances(&p), [20.0, 40.0]); // Font rows exclude the actual10px atomic.
        assert_eq!(
            p.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 4)]
        );
        assert_eq!(p.metrics().full_width(), 40.0);
        assert!(!p.metrics().overflow());
        assert!(
            !p.glyph_runs()
                .iter()
                .flat_map(|r| r.glyphs())
                .any(|g| g.range() == Range::new(2, 2) && g.advance() != 0.0)
        );
    }
}
#[test]
fn native_atomic_permission_after_empty_shy_custody_stays_independent() {
    for mode in [Hyphens::None, Hyphens::Manual] {
        let mut v = empty_after_shy_input(mode);
        v.source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(86073),
            crate::InlineBoxKind::InFlow,
            3,
            crate::Size::new(10.0, 10.0),
        ));
        v.anchor_order.push(crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Atomic(0),
            affinity: crate::Affinity::After,
        });
        for definition in [
            crate::ParagraphBoundaryDefinition::AtomicBefore(0),
            crate::ParagraphBoundaryDefinition::AtomicAfter(0),
        ] {
            v.boundary_policies.push(crate::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            });
        }
        let p = capture(&mut system(), v).unwrap();
        let atomic = PreparedParticipant::Atomic(p.atomic_views().next().unwrap().handle());
        let position = p.participants().position(|item| item == atomic).unwrap();
        for cut in [position, position + 1] {
            assert!(
                p.opportunity_views()
                    .any(|v| v.boundary().view().participant_offset() == cut
                        && v.kind() == PreparedBreakKind::Allowed),
                "actual atomic side retains its own permission"
            );
        }
    }
}
