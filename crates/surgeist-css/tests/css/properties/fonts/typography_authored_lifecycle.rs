#![forbid(unsafe_code)]
//! Functional tests for new typed typography models and intrinsic borrowed views.
//! Independent authorities: pinned Text4 WD20260814 property productions/initials,
//! selected CSS2 vertical-align; Values4 typed roots and deferred math range.
//! Canonical transform role order and transparent emitted-keyword pricing are
//! adopted authored product contracts. Contextual text processing is excluded.
use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

fn checked(p: P, text: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let declaration = if grammar {
        parse_property_value_for_grammar(p.grammar(), components.clone(), CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    declaration
}
fn fronts(p: P, text: &str) -> [CssDeclaration; 3] {
    let css = format!("/*😀*/{}:{text}!important", p.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    [
        declaration.clone(),
        checked(p, text, false),
        checked(p, text, true),
    ]
}
fn declared(source: &CssDeclaration) -> CssLonghandValueRef<'_> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TextTransform(v) => CssLonghandValueRef::TextTransform(v.value()),
        CssKnownPropertyValueRef::WrapInside(v) => CssLonghandValueRef::WrapInside(v.value()),
        CssKnownPropertyValueRef::WrapBefore(v) => CssLonghandValueRef::WrapBefore(v.value()),
        CssKnownPropertyValueRef::WrapAfter(v) => CssLonghandValueRef::WrapAfter(v.value()),
        CssKnownPropertyValueRef::LineBreak(v) => CssLonghandValueRef::LineBreak(v.value()),
        CssKnownPropertyValueRef::WordSpaceTransform(v) => {
            CssLonghandValueRef::WordSpaceTransform(v.value())
        }
        CssKnownPropertyValueRef::TabSize(v) => CssLonghandValueRef::TabSize(v.value()),
        CssKnownPropertyValueRef::TextIndent(v) => CssLonghandValueRef::TextIndent(v.value()),
        CssKnownPropertyValueRef::VerticalAlign(v) => CssLonghandValueRef::VerticalAlign(v.value()),
        _ => panic!("selected typed typography wrapper"),
    }
}
fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete ordinary")
    };
    let [item] = values.items() else {
        panic!("one terminal")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.ordinary_value().unwrap().view(), declared(source));
    values
}
fn primitive(
    expected: &str,
    inputs: usize,
    projections: usize,
    emit: impl Fn(
        CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    let exact = L::new(inputs, projections, expected.len());
    assert_eq!(emit(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            L::new(inputs - 1, projections, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(inputs, projections - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (
            L::new(inputs, projections, expected.len() - 1),
            K::ByteLimit,
        ),
    ] {
        for _ in 0..2 {
            assert_eq!(emit(limits).unwrap_err().kind(), kind);
        }
        assert_eq!(emit(exact).unwrap(), expected);
    }
}
fn transform_sets() -> [(
    Option<CssTextTransformCase>,
    bool,
    bool,
    &'static str,
    usize,
); 15] {
    use CssTextTransformCase as C;
    [
        (None, true, false, "full-width", 1),
        (None, false, true, "full-size-kana", 1),
        (None, true, true, "full-width full-size-kana", 2),
        (Some(C::Capitalize), false, false, "capitalize", 1),
        (Some(C::Capitalize), true, false, "capitalize full-width", 2),
        (
            Some(C::Capitalize),
            false,
            true,
            "capitalize full-size-kana",
            2,
        ),
        (
            Some(C::Capitalize),
            true,
            true,
            "capitalize full-width full-size-kana",
            3,
        ),
        (Some(C::Uppercase), false, false, "uppercase", 1),
        (Some(C::Uppercase), true, false, "uppercase full-width", 2),
        (
            Some(C::Uppercase),
            false,
            true,
            "uppercase full-size-kana",
            2,
        ),
        (
            Some(C::Uppercase),
            true,
            true,
            "uppercase full-width full-size-kana",
            3,
        ),
        (Some(C::Lowercase), false, false, "lowercase", 1),
        (Some(C::Lowercase), true, false, "lowercase full-width", 2),
        (
            Some(C::Lowercase),
            false,
            true,
            "lowercase full-size-kana",
            2,
        ),
        (
            Some(C::Lowercase),
            true,
            true,
            "lowercase full-width full-size-kana",
            3,
        ),
    ]
}

#[test]
fn transform_constructor_rejects_empty_and_all_nonempty_sets_retain_authored_getters_and_typed_views()
 {
    assert!(CssTextTransformSet::try_new(None, false, false).is_none());
    for (case, width, kana, css, work) in transform_sets() {
        let set = CssTextTransformSet::try_new(case, width, kana).unwrap();
        assert_eq!(set.case(), case);
        assert_eq!(set.full_width(), width);
        assert_eq!(set.full_size_kana(), kana);
        let value = CssTextTransform::Transforms(set);
        primitive(css, work, work, |l| {
            value.serialize_specified_with_limits(l)
        });
        for source in fronts(P::TextTransform, css) {
            let CssLonghandValueRef::TextTransform(actual) = declared(&source) else {
                panic!("transform")
            };
            assert_eq!(actual, &value);
            contributions(&source);
        }
    }
}
#[test]
fn transform_exclusive_branches_are_distinct_single_keyword_values() {
    for (value, css) in [
        (CssTextTransform::None, "none"),
        (CssTextTransform::MathAuto, "math-auto"),
    ] {
        primitive(css, 1, 1, |l| value.serialize_specified_with_limits(l));
        for source in fronts(P::TextTransform, css) {
            let CssLonghandValueRef::TextTransform(actual) = declared(&source) else {
                panic!("exclusive typed transform")
            };
            assert_eq!(actual, &value);
            contributions(&source);
        }
    }
    assert_ne!(CssTextTransform::None, CssTextTransform::MathAuto);
    let width =
        CssTextTransform::Transforms(CssTextTransformSet::try_new(None, true, false).unwrap());
    assert_ne!(width, CssTextTransform::None);
}
#[test]
fn every_finite_wrap_inside_state_has_typed_wrapper_projection_and_primitive_cost() {
    for (value, css) in [
        (CssWrapInside::Auto, "auto"),
        (CssWrapInside::Avoid, "avoid"),
    ] {
        primitive(css, 1, 1, |l| value.serialize_specified_with_limits(l));
        for source in fronts(P::WrapInside, css) {
            let CssLonghandValueRef::WrapInside(actual) = declared(&source) else {
                panic!("inside")
            };
            assert_eq!(*actual, value);
            contributions(&source);
        }
    }
}
#[test]
fn shared_boundary_states_preserve_distinct_before_and_after_property_wrappers() {
    for (value, css) in [
        (CssWrapBoundary::Auto, "auto"),
        (CssWrapBoundary::Avoid, "avoid"),
        (CssWrapBoundary::AvoidLine, "avoid-line"),
        (CssWrapBoundary::AvoidFlex, "avoid-flex"),
        (CssWrapBoundary::Line, "line"),
        (CssWrapBoundary::Flex, "flex"),
    ] {
        primitive(css, 1, 1, |l| value.serialize_specified_with_limits(l));
        for p in [P::WrapBefore, P::WrapAfter] {
            for source in fronts(p, css) {
                let actual = match declared(&source) {
                    CssLonghandValueRef::WrapBefore(v) if p == P::WrapBefore => v,
                    CssLonghandValueRef::WrapAfter(v) if p == P::WrapAfter => v,
                    _ => panic!("distinct boundary identity"),
                };
                assert_eq!(*actual, value);
                contributions(&source);
            }
        }
    }
}
#[test]
fn every_line_break_state_has_typed_projection_and_exact_keyword_provider() {
    for (value, css) in [
        (CssLineBreak::Auto, "auto"),
        (CssLineBreak::Loose, "loose"),
        (CssLineBreak::Normal, "normal"),
        (CssLineBreak::Strict, "strict"),
        (CssLineBreak::Anywhere, "anywhere"),
    ] {
        primitive(css, 1, 1, |l| value.serialize_specified_with_limits(l));
        for source in fronts(P::LineBreak, css) {
            let CssLonghandValueRef::LineBreak(actual) = declared(&source) else {
                panic!("line break")
            };
            assert_eq!(*actual, value);
            contributions(&source);
        }
    }
}
#[test]
fn five_word_space_states_are_constructible_without_empty_or_flag_only_payloads() {
    for (value, css, work) in [
        (CssWordSpaceTransform::None, "none", 1),
        (
            CssWordSpaceTransform::Space { auto_phrase: false },
            "space",
            1,
        ),
        (
            CssWordSpaceTransform::Space { auto_phrase: true },
            "space auto-phrase",
            2,
        ),
        (
            CssWordSpaceTransform::IdeographicSpace { auto_phrase: false },
            "ideographic-space",
            1,
        ),
        (
            CssWordSpaceTransform::IdeographicSpace { auto_phrase: true },
            "ideographic-space auto-phrase",
            2,
        ),
    ] {
        primitive(css, work, work, |l| {
            value.serialize_specified_with_limits(l)
        });
        for source in fronts(P::WordSpaceTransform, css) {
            let CssLonghandValueRef::WordSpaceTransform(actual) = declared(&source) else {
                panic!("word space")
            };
            assert_eq!(*actual, value);
            contributions(&source);
        }
    }
}
#[test]
fn all_nine_metadata_initials_have_source_defined_typed_branches_and_inheritance() {
    for (p, css, inherited) in [
        (P::TextTransform, "none", true),
        (P::WrapInside, "auto", false),
        (P::WrapBefore, "auto", false),
        (P::WrapAfter, "auto", false),
        (P::LineBreak, "auto", true),
        (P::WordSpaceTransform, "none", true),
        (P::TabSize, "8", true),
        (P::TextIndent, "0", true),
        (P::VerticalAlign, "baseline", false),
    ] {
        let CssPropertyKindRef::Longhand(meta) = p.metadata().unwrap().kind() else {
            panic!("independent longhand")
        };
        assert_eq!(meta.property().known_property(), p);
        assert_eq!(meta.inherited_by_default(), inherited);
        let initial = meta.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary initial")
        };
        match (p, initial.view()) {
            (P::TextTransform, CssLonghandValueRef::TextTransform(v)) => {
                assert_eq!(*v, CssTextTransform::None)
            }
            (P::WrapInside, CssLonghandValueRef::WrapInside(v)) => {
                assert_eq!(*v, CssWrapInside::Auto)
            }
            (P::WrapBefore, CssLonghandValueRef::WrapBefore(v)) => {
                assert_eq!(*v, CssWrapBoundary::Auto)
            }
            (P::WrapAfter, CssLonghandValueRef::WrapAfter(v)) => {
                assert_eq!(*v, CssWrapBoundary::Auto)
            }
            (P::LineBreak, CssLonghandValueRef::LineBreak(v)) => assert_eq!(*v, CssLineBreak::Auto),
            (P::WordSpaceTransform, CssLonghandValueRef::WordSpaceTransform(v)) => {
                assert_eq!(*v, CssWordSpaceTransform::None)
            }
            (P::TabSize, CssLonghandValueRef::TabSize(CssTabSize::Number(v))) => {
                assert_eq!(v.serialize_specified().unwrap(), "8");
                assert!(v.calculation().is_none());
                assert_eq!(v.origin(), &CssValueOrigin::Programmatic);
            }
            (P::TextIndent, CssLonghandValueRef::TextIndent(v)) => {
                assert!(!v.hanging());
                assert!(!v.each_line());
                assert_eq!(v.length().serialize_specified().unwrap(), "0");
                assert!(v.length().calculation().is_none());
                assert_eq!(v.length().origin(), &CssValueOrigin::Programmatic);
            }
            (P::VerticalAlign, CssLonghandValueRef::VerticalAlign(v)) => {
                assert_eq!(*v, CssVerticalAlign::Baseline)
            }
            _ => panic!("initial/property typed identity"),
        }
        let source = checked(p, css, false);
        let values = contributions(&source);
        assert_eq!(values.items()[0].ordinary_value(), Some(initial));
    }
}
#[test]
fn tab_literal_zero_prefers_number_and_explicit_dimensions_retain_length_and_original_tokens() {
    for (text, is_number, css) in [
        ("0", true, "0"),
        ("-0", true, "0"),
        ("8", true, "8"),
        ("1e-999", true, "0"),
        ("0px", false, "0px"),
        ("-0px", false, "0px"),
        ("2PX", false, "2px"),
    ] {
        for source in fronts(P::TabSize, text) {
            let CssLonghandValueRef::TabSize(tab) = declared(&source) else {
                panic!("tab")
            };
            match tab {
                CssTabSize::Number(v) if is_number => {
                    assert!(v.calculation().is_none());
                    assert_eq!(
                        v.literal_component(),
                        Some(&source.value_components().items()[0])
                    );
                    assert_eq!(v.origin(), source.value_components().items()[0].origin());
                }
                CssTabSize::Length(v) if !is_number => {
                    assert!(v.calculation().is_none());
                    assert_eq!(
                        v.literal_component(),
                        Some(&source.value_components().items()[0])
                    );
                    assert_eq!(v.origin(), source.value_components().items()[0].origin());
                }
                _ => panic!("authored number/length distinction"),
            }
            primitive(css, 1, 1, |l| tab.serialize_specified_with_limits(l));
            contributions(&source);
        }
    }
    let number = CssTabSize::Number(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number("0").unwrap(),
        )
        .unwrap(),
    );
    let length = CssTabSize::Length(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("0", "px").unwrap(),
        )
        .unwrap(),
    );
    assert_ne!(number, length);
    assert_eq!(number.serialize_specified().unwrap(), "0");
    assert_eq!(length.serialize_specified().unwrap(), "0px");
}
#[test]
fn tab_math_retains_pure_root_graph_origins_and_deferred_negative_ranges() {
    for (text, is_number, expected) in [
        ("calc(-1)", true, "calc(-1)"),
        ("calc(-1px)", false, "calc(-1px)"),
        ("calc(1px / 1px)", true, "calc(1)"),
        ("calc(1% / 1%)", true, "calc(1)"),
        ("calc(1px + 2em)", false, "calc(2em + 1px)"),
    ] {
        for source in fronts(P::TabSize, text) {
            let CssLonghandValueRef::TabSize(tab) = declared(&source) else {
                panic!("tab")
            };
            let components = match tab {
                CssTabSize::Number(v) if is_number => {
                    assert!(v.literal_component().is_none());
                    assert_eq!(v.origin(), source.value_components().items()[0].origin());
                    v.calculation().unwrap().components()
                }
                CssTabSize::Length(v) if !is_number => {
                    assert!(v.literal_component().is_none());
                    assert_eq!(v.origin(), source.value_components().items()[0].origin());
                    v.calculation().unwrap().components()
                }
                _ => panic!("pure root typed branch"),
            };
            assert_eq!(components, source.value_components());
            assert_eq!(tab.serialize_specified().unwrap(), expected);
            contributions(&source);
        }
    }
    let number = CssTabSize::Number(
        CssSpecifiedNonNegativeNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(parse_component_values("calc(-1)").unwrap())
                .unwrap(),
        )
        .unwrap(),
    );
    let length = CssTabSize::Length(
        CssSpecifiedNonNegativeLength::try_from_calculation(
            CssLengthCalculation::try_from_components(
                parse_component_values("calc(-1px)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    assert_eq!(number.serialize_specified().unwrap(), "calc(-1)");
    assert_eq!(length.serialize_specified().unwrap(), "calc(-1px)");
    let math = CssTabSize::Length(
        CssSpecifiedNonNegativeLength::try_from_calculation(
            CssLengthCalculation::try_from_components(
                parse_component_values("calc(1px + 2em)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    primitive("calc(2em + 1px)", 4, 5, |l| {
        math.serialize_specified_with_limits(l)
    });
    for text in [
        "-1",
        "-1e-999",
        "-1px",
        "-1e-999px",
        "calc(1%)",
        "calc(0 + 1px)",
    ] {
        assert!(
            parse_property_value_for_grammar(
                P::TabSize.grammar(),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}
#[test]
fn tab_reentry_borrows_the_supplied_numeric_children_and_preserves_original_occurrence() {
    for pending in ["var(--tab)", "env(tab)", "attr(data-tab *)"] {
        let source = checked(P::TabSize, pending, true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for text in ["0", "0PX", "calc(-1)", "CALC(1PX + 2EM)"] {
            let replacement = parse_component_values(text).unwrap();
            let snapshot = replacement.clone();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("reentered")
            };
            let [item] = values.items() else {
                panic!("one terminal")
            };
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let reference = parse_property_value_for_grammar(
                P::TabSize.grammar(),
                replacement.clone(),
                CssImportance::Normal,
            )
            .unwrap();
            assert_eq!(item.ordinary_value().unwrap().view(), declared(&reference));
            let CssLonghandValueRef::TabSize(tab) = item.ordinary_value().unwrap().view() else {
                panic!("typed tab projection")
            };
            let origin = match tab {
                CssTabSize::Number(v) => v.origin(),
                CssTabSize::Length(v) => v.origin(),
                _ => panic!("number or length"),
            };
            assert_eq!(origin, replacement.items()[0].origin());
            assert_eq!(replacement, snapshot);
        }
        assert!(
            handle
                .reenter(parse_component_values("-1e-999").unwrap())
                .is_err()
        );
        assert!(handle.reenter(parse_component_values("8").unwrap()).is_ok());
    }
}
#[test]
fn existing_indent_flags_and_vertical_signed_numeric_models_have_new_typed_projections() {
    for source in fronts(P::TextIndent, "each-line -1e-999% hanging") {
        let CssLonghandValueRef::TextIndent(v) = declared(&source) else {
            panic!("indent")
        };
        assert!(v.hanging());
        assert!(v.each_line());
        assert_eq!(v.length().serialize_specified().unwrap(), "0%");
        contributions(&source);
    }
    for source in fronts(P::VerticalAlign, "-1px") {
        let CssLonghandValueRef::VerticalAlign(CssVerticalAlign::Length(v)) = declared(&source)
        else {
            panic!("vertical signed length")
        };
        assert_eq!(v.serialize_specified().unwrap(), "-1px");
        assert_eq!(v.origin(), source.value_components().items()[0].origin());
        contributions(&source);
    }
}
