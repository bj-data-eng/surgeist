use super::*;
use crate::{
    AnalysisTypography, AnalyzedSource, ParagraphBaseDirection, Range, Source, Style,
    UprightContentPiece, UprightPresentation, WhiteSpace,
};

fn style(locale: Option<&str>) -> Style {
    Style {
        locale: locale.map(str::to_owned),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn analyze(source: Source, locale: Option<&str>) -> AnalyzedSource {
    AnalyzedSource::new(
        source,
        style(locale),
        AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft,
        },
    )
    .unwrap()
}

fn content(projection: &SourceProjection, atoms: IndexRange<usize>) -> UprightContent {
    // This callback describes these actual original atoms, not a synthetic
    // indication or a claimed Unicode observation for conditional property facts.
    UprightContent::Presentation(UprightPresentation::new(
        projection.atoms[atoms]
            .iter()
            .map(|atom| UprightContentPiece::Source { range: atom.source })
            .collect(),
    ))
}

fn arranged(analyzed: &AnalyzedSource, barriers: &[usize]) -> Result<Vec<UprightUnit>> {
    let inner = &analyzed.inner;
    units(
        &inner.projection,
        &inner.analysis,
        &inner.styles,
        &barriers.iter().copied().collect(),
        |atoms| content(&inner.projection, atoms),
    )
}

fn ranges(units: &[UprightUnit]) -> Vec<IndexRange<usize>> {
    units.iter().map(|unit| unit.atoms.clone()).collect()
}

fn first_properties(analyzed: &AnalyzedSource) -> Properties {
    properties(&analyzed.inner.projection.atoms[analyzed.inner.analysis.graphemes[0].clone()])
        .unwrap()
}

#[test]
fn tibetan_followers_keep_complete_clusters_and_immediate_predecessors() {
    let a = analyze(Source::new("\u{f40}\u{f72}\u{f0b}\u{f0d}\u{f41}"), None);
    let egcs = a.inner.analysis.graphemes.clone();
    assert_eq!(egcs, [0..2, 2..3, 3..4, 4..5]);
    assert_eq!(ranges(&arranged(&a, &[]).unwrap()), [0..4, 4..5]);
    assert_eq!(
        a.inner.analysis.graphemes, egcs,
        "editing EGCs are unchanged"
    );
    let leading = analyze(Source::new("\u{f0d}\u{f0d}\u{f40}"), None);
    assert_eq!(ranges(&arranged(&leading, &[]).unwrap()), [0..2, 2..3]);
    let space = analyze(Source::new("\u{f0d} \u{f0d}"), None);
    assert_eq!(ranges(&arranged(&space, &[]).unwrap()), [0..1, 1..3]);
    assert_eq!(space.source().text().code_units(), [0xf0d, 0x20, 0xf0d]);
}

#[test]
fn exact_tibetan_set_keeps_marks_and_does_not_absorb_head_or_caret_marks() {
    for cp in [
        0xf08, 0xf0b, 0xf0c, 0xf0d, 0xf0e, 0xf0f, 0xf10, 0xf11, 0xf12, 0xf14,
    ] {
        let text = format!("\u{f40}{}\u{301}", char::from_u32(cp).unwrap());
        let a = analyze(Source::new(text), None);
        assert_eq!(a.inner.analysis.graphemes, [0..1, 1..3]);
        assert_eq!(
            ranges(&arranged(&a, &[]).unwrap()),
            std::iter::once(0..3).collect::<Vec<_>>(),
            "U+{cp:04X}"
        );
    }
    for cp in [0xf04, 0xf05, 0xf06, 0xf07, 0xf13] {
        let a = analyze(
            Source::new(format!("\u{f40}{}", char::from_u32(cp).unwrap())),
            None,
        );
        assert_eq!(
            ranges(&arranged(&a, &[]).unwrap()),
            [0..1, 1..2],
            "U+{cp:04X}"
        );
    }
}

#[test]
fn actual_barriers_and_hard_separators_stop_attachment_but_style_edges_do_not() {
    let mut source = Source::new("\u{f40}\u{f72}\u{f0b}\u{f0d}");
    source.span(Range::new(2, 4), style(Some("und-Tibt")));
    let a = analyze(source, None);
    assert_eq!(
        ranges(&arranged(&a, &[]).unwrap()),
        std::iter::once(0..4).collect::<Vec<_>>()
    );
    assert_eq!(ranges(&arranged(&a, &[3]).unwrap()), [0..3, 3..4]);
    let hard = analyze(Source::new("\u{f40}\u{f0b}\n\u{f0d}"), None);
    assert_eq!(ranges(&arranged(&hard, &[]).unwrap()), [0..2, 2..3, 3..4]);
}

#[test]
fn selected_fragment_rebuild_uses_an_orphan_without_dropping_original_custody() {
    let a = analyze(Source::new("\u{f40}\u{f72}\u{f0b}\u{f0d}\u{f41}"), None);
    assert_eq!(ranges(&arranged(&a, &[]).unwrap()), [0..4, 4..5]);
    // The real projection constructor rebuilds EGCs on a selected fragment.
    // Original source associations survive while presentation positions rebase.
    let mut atoms = a.inner.projection.atoms[3..5].to_vec();
    for atom in &mut atoms {
        atom.presentation.start -= 3;
        atom.presentation.end -= 3;
    }
    let projection = SourceProjection::from_atoms_at_barriers(atoms, 2, vec![], vec![]).unwrap();
    let mut analysis = a.inner.analysis.clone();
    analysis.infos = analysis.infos[3..5].to_vec();
    analysis.levels = analysis.levels[3..5].to_vec();
    analysis.graphemes = vec![0..1, 1..2];
    assert_eq!(projection.graphemes, [0, 1, 2]);
    let selected = units(
        &projection,
        &analysis,
        &a.inner.styles,
        &BTreeSet::new(),
        |atoms| content(&projection, atoms),
    )
    .unwrap();
    assert_eq!(ranges(&selected), [0..1, 1..2]);
    assert_eq!(
        projection
            .atoms
            .iter()
            .map(|a| a.source)
            .collect::<Vec<_>>(),
        [Range::new(3, 4), Range::new(4, 5)]
    );
    assert_eq!(
        a.source().text().code_units(),
        [0xf40, 0xf72, 0xf0b, 0xf0d, 0xf41]
    );
}

#[test]
fn first_cluster_base_and_inherited_enclosing_mark_preserve_actual_script_facts() {
    for (text, script) in [
        ("A\u{301}", Script::Latin),
        ("A\u{20dd}", Script::Latin),
        ("ก\u{20dd}", Script::Thai),
    ] {
        let a = analyze(Source::new(text), None);
        let p = first_properties(&a);
        assert_eq!(p.scripts, [script]);
        assert!(!unit_properties::has_common_enclosing_mark(
            &a.inner.projection.atoms
        ));
        assert_eq!(
            ranges(&arranged(&a, &[]).unwrap()),
            std::iter::once(0..2).collect::<Vec<_>>()
        );
    }
    assert_eq!(
        CodePointMapData::<Script>::new().get32(0x20dd),
        Script::Inherited
    );
}

#[test]
fn common_enclosing_property_rule_is_qualified_without_claiming_a_unicode_witness() {
    // Deliberately conditional rule qualification. Bundled Unicode17 does not
    // supply a real Common-Me character, and U+20DD is not such a character.
    let resolved = resolve_properties(scalar_properties(0x41).unwrap(), true, None).unwrap();
    assert_eq!(resolved.category, GeneralCategory::OtherSymbol);
    assert_eq!(resolved.scripts, [Script::Common]);
    assert_eq!(
        resolved.width,
        CodePointMapData::<EastAsianWidth>::new().get32(0xfffd)
    );
}

#[test]
fn zs_combining_sequence_keeps_base_width_and_first_mark_other_properties() {
    let bare = analyze(Source::new("\u{202f}"), None);
    assert!(first_properties(&bare).scripts.contains(&Script::Mongolian));
    let combined = analyze(Source::new("\u{202f}\u{363}"), None);
    let p = first_properties(&combined);
    assert_eq!(
        combined.inner.analysis.graphemes,
        std::iter::once(0..2).collect::<Vec<_>>()
    );
    assert_eq!(p.category, GeneralCategory::ModifierSymbol);
    assert_eq!(p.scripts, [Script::Latin]);
    assert_eq!(
        p.width,
        CodePointMapData::<EastAsianWidth>::new().get32(0x202f)
    );
    assert!(arranged(&combined, &[]).unwrap()[0].isolate());
    let ideographic = analyze(Source::new("\u{3000}\u{301}\u{3099}"), None);
    let p = first_properties(&ideographic);
    assert_eq!(p.width, EastAsianWidth::Fullwidth);
    assert_eq!(p.category, GeneralCategory::ModifierSymbol);
    assert!(p.scripts.contains(&Script::Latin));
    assert!(!p.scripts.contains(&Script::Hiragana));
    assert_eq!(
        arranged(&ideographic, &[]).unwrap()[0].context,
        UprightUnitContext::WidthOverride(UprightUnitRole::NormalVerticalContext)
    );
    let kana = analyze(Source::new(" \u{3099}\u{301}"), None);
    let p = first_properties(&kana);
    assert_eq!(p.width, EastAsianWidth::Narrow);
    assert_eq!(p.scripts, [Script::Hiragana, Script::Katakana]);
    let mong = analyze(Source::new(" \u{180b}"), None);
    assert_eq!(
        mong.inner.analysis.graphemes,
        std::iter::once(0..2).collect::<Vec<_>>()
    );
    assert_eq!(first_properties(&mong).scripts, [Script::Mongolian]);
    assert!(matches!(
        arranged(&mong, &[]).unwrap_err().detail(),
        Some(ErrorDetail::UprightScript {
            reason: UprightScriptFailure::Intrinsic { resolved: None },
            ..
        })
    ));
}

#[test]
fn width_overrides_precede_all_intrinsic_membership_and_locale_context() {
    for (text, role) in [
        ("Ａ", UprightUnitRole::NormalVerticalContext),
        ("〈", UprightUnitRole::NormalVerticalContext),
        ("ｶﾞ", UprightUnitRole::IsolatedUnit),
    ] {
        let a = analyze(Source::new(text), Some("und-Mong"));
        let actual = arranged(&a, &[]).unwrap();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].context, UprightUnitContext::WidthOverride(role));
    }
}

#[test]
fn all_supported_mixed_roles_fallback_without_neighbor_or_language_inference() {
    for (text, atom) in [("·", 0), ("A·B", 1), ("漢·字", 1), ("ا·ب", 1)] {
        for locale in [
            None,
            Some("fr"),
            Some("zh"),
            Some("und-Jpan"),
            Some("und-Mong"),
        ] {
            let a = analyze(Source::new(text), locale);
            let actual = arranged(&a, &[]).unwrap();
            let dot = actual.iter().find(|u| u.atoms.start == atom).unwrap();
            assert_eq!(
                dot.context,
                UprightUnitContext::MixedRoleFallback,
                "{text:?} {locale:?}"
            );
            assert!(dot.isolate());
        }
    }
    for (locale, role) in [
        ("und-Latn", UprightUnitRole::IsolatedUnit),
        ("und-Hani", UprightUnitRole::NormalVerticalContext),
    ] {
        let a = analyze(Source::new("·"), Some(locale));
        assert_eq!(
            arranged(&a, &[]).unwrap()[0].context,
            UprightUnitContext::ExplicitScript(role)
        );
    }
}

#[test]
fn complete_unit_context_coherence_is_independent_of_actual_style_intersections() {
    for (left, right, expected) in [
        (
            Some("und-Hani"),
            Some("en-Hani"),
            UprightUnitContext::ExplicitScript(UprightUnitRole::NormalVerticalContext),
        ),
        (
            Some("und-Latn"),
            Some("en-Latn"),
            UprightUnitContext::ExplicitScript(UprightUnitRole::IsolatedUnit),
        ),
        (
            Some("und-Hani"),
            Some("und-Latn"),
            UprightUnitContext::MixedRoleFallback,
        ),
        (
            Some("und-Hani"),
            None,
            UprightUnitContext::MixedRoleFallback,
        ),
    ] {
        let mut source = Source::new("·\u{301}");
        source.span(Range::new(0, 1), style(left));
        source.span(Range::new(1, 2), style(right));
        let a = analyze(source, None);
        assert_eq!(
            a.inner.analysis.graphemes,
            std::iter::once(0..2).collect::<Vec<_>>()
        );
        let actual = arranged(&a, &[]).unwrap();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].context, expected);
    }
}

#[test]
fn mixed_intrinsic_sets_require_an_actual_coherent_member() {
    for text in ["\u{202f}", "\u{205a}", "\u{2e30}"] {
        for locale in [None, Some("fr"), Some("mn"), Some("und-Jpan")] {
            let a = analyze(Source::new(text), locale);
            let error = arranged(&a, &[]).unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert!(matches!(
                error.detail(),
                Some(ErrorDetail::UprightScript {
                    reason: UprightScriptFailure::UnresolvedContext { .. },
                    ..
                })
            ));
        }
    }
    for (text, locale) in [
        ("\u{202f}", "und-Latn"),
        ("\u{205a}", "und-Geor"),
        ("\u{2e30}", "und-Avst"),
    ] {
        let a = analyze(Source::new(text), Some(locale));
        assert_eq!(
            arranged(&a, &[]).unwrap()[0].context,
            UprightUnitContext::ExplicitScript(UprightUnitRole::IsolatedUnit)
        );
    }
    let nonmember = analyze(Source::new("\u{2e30}"), Some("und-Latn"));
    assert!(matches!(
        arranged(&nonmember, &[]).unwrap_err().detail(),
        Some(ErrorDetail::UprightScript {
            reason: UprightScriptFailure::UnresolvedContext {
                reason: UprightScriptContextFailure::NonMember
            },
            ..
        })
    ));
    for (text, locale, selected) in [
        ("\u{202f}", "und-Mong", *b"Mong"),
        ("\u{202f}", "und-Phag", *b"Phag"),
        ("\u{205a}", "und-Orkh", *b"Orkh"),
    ] {
        let a = analyze(Source::new(text), Some(locale));
        let error = arranged(&a, &[]).unwrap_err();
        let Some(ErrorDetail::UprightScript {
            reason:
                UprightScriptFailure::Intrinsic {
                    resolved: Some(script),
                },
            ..
        }) = error.detail()
        else {
            panic!("expected actual explicit intrinsic refusal")
        };
        assert_eq!(script.iso15924(), selected);
    }
}

#[test]
fn intrinsic_mixed_set_checks_every_style_in_the_complete_arrangement_unit() {
    let mut source = Source::new("\u{202f}\u{f0d}");
    source.span(Range::new(0, 1), style(Some("und-Latn")));
    source.span(Range::new(1, 2), style(Some("und-Mong")));
    let a = analyze(source, None);
    assert_eq!(
        arrangement_ranges(
            &a.inner.projection,
            &a.inner.analysis.graphemes,
            &BTreeSet::new()
        )
        .unwrap(),
        std::iter::once(0..2).collect::<Vec<_>>()
    );
    let error = arranged(&a, &[]).unwrap_err();
    let Some(ErrorDetail::UprightScript {
        reason:
            UprightScriptFailure::UnresolvedContext {
                reason: UprightScriptContextFailure::ConflictingScripts,
            },
        contexts,
        ..
    }) = error.detail()
    else {
        panic!("actual complete-unit contexts conflict")
    };
    assert_eq!(
        contexts.iter().map(|c| c.locale()).collect::<Vec<_>>(),
        [Some("und-Latn"), Some("und-Mong")]
    );
    assert_eq!(
        contexts[0].owner(),
        TextStyleOwner::Source {
            range: Range::new(0, 1)
        }
    );
    assert_eq!(
        contexts[1].owner(),
        TextStyleOwner::Source {
            range: Range::new(1, 2)
        }
    );

    let mut source = Source::new("\u{202f}\u{f0d}\u{f0d}");
    source.span(Range::new(0, 1), style(Some("und-Latn")));
    source.span(Range::new(1, 2), style(Some("und-Mong")));
    let a = analyze(source, None);
    assert!(
        matches!(
            arranged(&a, &[]).unwrap_err().detail(),
            Some(ErrorDetail::UprightScript {
                reason: UprightScriptFailure::UnresolvedContext {
                    reason: UprightScriptContextFailure::MissingExplicitScript
                },
                ..
            })
        ),
        "a genuine later missing context is retained instead of masking it with an earlier conflict"
    );
}

#[test]
fn uniform_supported_roles_do_not_require_an_explicit_resolver() {
    let horizontal = analyze(Source::new("א"), Some("und-Mong"));
    assert_eq!(
        arranged(&horizontal, &[]).unwrap()[0].context,
        UprightUnitContext::UniformScripts(UprightUnitRole::IsolatedUnit)
    );
    let vertical = analyze(Source::new("\u{13000}"), Some("und-Latn"));
    assert_eq!(
        arranged(&vertical, &[]).unwrap()[0].context,
        UprightUnitContext::UniformScripts(UprightUnitRole::NormalVerticalContext)
    );
}

#[test]
fn intrinsic_candidates_and_failure_content_preserve_the_actual_source() {
    for text in ["\u{1820}", "\u{a840}", "\u{1681}", "\u{10c00}"] {
        let a = analyze(Source::new(text), None);
        let error = arranged(&a, &[]).unwrap_err();
        assert!(matches!(
            error.detail(),
            Some(ErrorDetail::UprightScript {
                reason: UprightScriptFailure::Intrinsic { resolved: None },
                ..
            })
        ));
    }
    let a = analyze(Source::new("A\u{202f}"), Some("fr"));
    let error = arranged(&a, &[]).unwrap_err();
    let Some(ErrorDetail::UprightScript {
        content,
        candidates,
        contexts,
        ..
    }) = error.detail()
    else {
        panic!("expected actual source-script error")
    };
    let UprightContent::Presentation(presentation) = content.as_ref() else {
        panic!("actual source refusal retains presentation provenance")
    };
    assert_eq!(
        candidates.iter().map(|s| s.iso15924()).collect::<Vec<_>>(),
        [*b"Latn", *b"Mong", *b"Phag"]
    );
    assert_eq!(contexts.len(), 1);
    assert_eq!(
        presentation.pieces(),
        [UprightContentPiece::Source {
            range: Range::new(1, 2)
        }]
    );
    assert_eq!(a.source().text().code_units(), [0x41, 0x202f]);
}
