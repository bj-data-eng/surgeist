#![forbid(unsafe_code)]
//! Functional construction contract from selected parser placement and Fonts4 §4.1/13.2.
use surgeist_css::*;

fn sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}
fn rule(source: &str) -> CssRule {
    sheet(source).rules()[0].clone()
}
fn scoped(source: &str) -> Vec<CssScopedRule> {
    let source = format!("@scope {{ {source} }}");
    let input = sheet(&source);
    let CssRule::Scope(scope) = &input.rules()[0] else {
        panic!("scope")
    };
    scope.rules().rules().to_vec()
}
fn context() -> CssNamespaceContext {
    CssNamespaceContext::default()
}
fn query() -> CssMediaQueryList {
    CssMediaQueryList::new(Vec::new())
}
fn ordinary(source: &str, kind: CssFontFaceDescriptorKind) -> CssFontFaceDescriptorValue {
    let report = parse_font_face_descriptor_value(source, kind);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(value)) = report.syntax() else {
        panic!("ordinary descriptor")
    };
    value.clone()
}
#[test]
fn source_free_face_and_sheet_preserve_ordered_children_and_empty_rules() {
    let empty = CssFontFaceRule::new(CssFontFaceDescriptors::new(Vec::new()));
    assert_eq!(empty.position(), None);
    let parsed = rule("@font-face { font-weight: 300; font-weight: 600; }");
    let CssRule::FontFace(face) = &parsed else {
        panic!("face")
    };
    let positions = face
        .descriptors()
        .occurrences()
        .map(|record| record.position())
        .collect::<Vec<_>>();
    let constructed = CssFontFaceRule::new(face.descriptors().clone());
    assert_eq!(constructed.position(), None);
    assert_eq!(
        constructed
            .descriptors()
            .occurrences()
            .map(|record| record.position())
            .collect::<Vec<_>>(),
        positions
    );
    let input = CssSheet::try_from_rules(vec![
        CssRule::FontFace(empty),
        CssRule::FontFace(constructed),
    ])
    .unwrap();
    assert!(input.encoding().is_none());
    assert_eq!(
        input.to_specified_css().unwrap(),
        "@font-face { }\n@font-face { font-weight: 600; }"
    );
    assert_eq!(face.descriptors().occurrences().len(), 2);
}
#[test]
fn every_ordinary_and_scoped_group_constructor_has_absent_enclosing_position() {
    let children = vec![rule("@font-face {}")];
    let media = CssMediaRule::try_new(query(), children.clone(), &context()).unwrap();
    assert_eq!(media.position(), None);
    assert_eq!(media.rules(), children);
    let source = rule("@supports (display: block) {}");
    let CssRule::Supports(supports) = source else {
        panic!("supports")
    };
    assert_eq!(
        CssSupportsRule::try_new(supports.condition().clone(), children.clone(), &context())
            .unwrap()
            .position(),
        None
    );
    let source = rule("@container (width > 1px) {}");
    let CssRule::Container(container) = source else {
        panic!("container")
    };
    assert_eq!(
        CssContainerRule::try_new(container.prelude().clone(), children.clone(), &context())
            .unwrap()
            .position(),
        None
    );
    assert_eq!(
        CssLayerBlockRule::try_new(None, children, &context())
            .unwrap()
            .position(),
        None
    );
    let children = scoped("@font-face {}");
    assert_eq!(
        CssScopedMediaRule::try_new(query(), children.clone(), &context())
            .unwrap()
            .position(),
        None
    );
    assert_eq!(
        CssScopedSupportsRule::try_new(supports.condition().clone(), children.clone(), &context())
            .unwrap()
            .position(),
        None
    );
    assert_eq!(
        CssScopedContainerRule::try_new(container.prelude().clone(), children.clone(), &context())
            .unwrap()
            .position(),
        None
    );
    assert_eq!(
        CssScopedLayerBlockRule::try_new(None, children.clone(), &context())
            .unwrap()
            .position(),
        None
    );
    assert_eq!(
        CssScopeRule::try_new(
            None,
            None,
            children,
            &context(),
            CssScopeNestingContext::None
        )
        .unwrap()
        .position(),
        None
    );
}
#[test]
fn leading_layer_import_namespace_order_and_duplicate_namespace_admission() {
    for source in [
        "@layer first; @import 'a'; @namespace p 'urn:p'; p|a {}",
        "@namespace p 'urn:p'; @namespace P 'urn:P'; p|a {}",
    ] {
        let parsed = sheet(source);
        assert!(
            CssSheet::try_from_rules(parsed.rules().to_vec()).is_ok(),
            "{source}"
        );
    }
    for (rules, kind, path) in [
        (
            vec![rule("a {}"), rule("@import 'a';")],
            CssRuleConstructionErrorKind::InvalidPreludeOrder,
            vec![1],
        ),
        (
            vec![
                rule("@import 'a';"),
                rule("@layer first;"),
                rule("@import 'b';"),
            ],
            CssRuleConstructionErrorKind::InvalidPreludeOrder,
            vec![2],
        ),
        (
            vec![rule("@namespace p 'urn:p';"), rule("@import 'b';")],
            CssRuleConstructionErrorKind::InvalidPreludeOrder,
            vec![1],
        ),
        (
            vec![rule("@namespace p 'urn:p';"), rule("@namespace p 'urn:p';")],
            CssRuleConstructionErrorKind::DuplicateNamespace,
            vec![1],
        ),
        (
            vec![rule("@namespace 'urn:a';"), rule("@namespace 'urn:a';")],
            CssRuleConstructionErrorKind::DuplicateNamespace,
            vec![1],
        ),
    ] {
        let before = rules.clone();
        let error = CssSheet::try_from_rules(rules).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.path(), path);
        assert!(error.position().is_some());
        assert!(!before.is_empty());
    }
}
#[test]
fn detached_relative_and_nested_declaration_subtrees_recheck_ancestry() {
    let parsed = sheet("a { > b {} color: red; }");
    let CssRule::Style(style) = &parsed.rules()[0] else {
        panic!("style")
    };
    assert_eq!(
        CssSheet::try_from_rules(vec![parsed.rules()[0].clone()])
            .unwrap()
            .rules(),
        parsed.rules()
    );
    for child in style.rules() {
        let error = CssMediaRule::try_new(query(), vec![child.clone()], &context()).unwrap_err();
        assert_eq!(error.kind(), CssRuleConstructionErrorKind::InvalidPlacement);
        assert_eq!(error.path(), &[0]);
        assert!(error.position().is_some());
    }
    assert!(CssSheet::try_from_rules(vec![rule("& {}")]).is_ok());
    assert!(CssScopedMediaRule::try_new(query(), scoped("> b {}"), &context()).is_ok());
    assert_eq!(
        CssMediaRule::try_new(query(), vec![rule("@import 'a';")], &context())
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(
        CssMediaRule::try_new(query(), vec![rule("@namespace p 'urn:p';")], &context())
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
}
#[test]
fn scope_direct_page_role_differs_from_its_ordinary_group_body() {
    let source = sheet("@scope { @media all { @page {} } }");
    let CssRule::Scope(scope) = &source.rules()[0] else {
        panic!("scope")
    };
    let CssScopedRule::Media(media) = &scope.rules().rules()[0] else {
        panic!("media")
    };
    let page = media.rules().rules()[0].clone();
    let list = vec![page];
    assert!(CssScopedMediaRule::try_new(query(), list.clone(), &context()).is_ok());
    let error = CssScopeRule::try_new(None, None, list, &context(), CssScopeNestingContext::None)
        .unwrap_err();
    assert_eq!(error.kind(), CssRuleConstructionErrorKind::InvalidPlacement);
    assert_eq!(error.path(), &[0]);
    assert!(error.position().is_some());
    assert!(
        CssScopeRule::try_new(
            scope.root().cloned(),
            scope.limit().cloned(),
            scope.rules().rules().to_vec(),
            &context(),
            CssScopeNestingContext::None
        )
        .is_ok()
    );
}
#[test]
fn namespace_constraints_check_declared_prefix_and_default_structure_without_uri_identity() {
    let parsed =
        sheet("@namespace p 'urn:old'; p|a {} @supports selector(p|b) {} @scope (p|c) {} ");
    let new_context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("p").unwrap()),
        CssNamespaceName::new("urn:new"),
    )]);
    for child in &parsed.rules()[1..] {
        assert!(CssMediaRule::try_new(query(), vec![child.clone()], &new_context).is_ok());
        assert_eq!(
            CssMediaRule::try_new(query(), vec![child.clone()], &context())
                .unwrap_err()
                .kind(),
            CssRuleConstructionErrorKind::NamespaceMismatch
        );
    }
    let default =
        CssNamespaceContext::from_bindings([(None, CssNamespaceName::new("urn:default"))]);
    assert_eq!(
        CssMediaRule::try_new(query(), vec![rule("a {}")], &default)
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::NamespaceMismatch
    );
    let parsed = sheet("@namespace 'urn:default'; a {}");
    assert_eq!(
        CssMediaRule::try_new(query(), vec![parsed.rules()[1].clone()], &context())
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::NamespaceMismatch
    );
    assert!(CssMediaRule::try_new(query(), vec![rule("*|a {}")], &default).is_ok());
}
#[test]
fn scope_raw_identifiers_and_functional_arguments_are_intrinsically_checked() {
    for selector in [
        CssSelector::Tag(String::new()),
        CssSelector::Key("\0".into()),
        CssSelector::Class(String::new()),
        CssSelector::PseudoClass(CssPseudoClass::Is(
            CssPseudoSelectorList::try_new(vec![CssSelector::Class("\0".into())]).unwrap(),
        )),
    ] {
        let root =
            CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector)]).unwrap();
        let error = CssScopeRule::try_new(
            Some(root),
            None,
            Vec::new(),
            &context(),
            CssScopeNestingContext::None,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleConstructionErrorKind::InvalidSelectorIdentifier
        );
        assert!(error.path().is_empty());
        assert_eq!(error.position(), None);
    }
    for name in ["1lead", "embedded space", "é"] {
        let root = CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(
            CssSelector::Class(name.into()),
        )])
        .unwrap();
        assert!(
            CssScopeRule::try_new(
                None,
                Some(root),
                Vec::new(),
                &context(),
                CssScopeNestingContext::None
            )
            .is_ok()
        );
    }
    let mut selector = CssSelector::Class("ok".into());
    for _ in 0..256 {
        selector = CssSelector::PseudoClass(CssPseudoClass::Is(
            CssPseudoSelectorList::try_new(vec![selector]).unwrap(),
        ));
    }
    let root =
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector.clone())]).unwrap();
    assert!(
        CssScopeRule::try_new(
            Some(root),
            None,
            Vec::new(),
            &context(),
            CssScopeNestingContext::None
        )
        .is_ok()
    );
    selector = CssSelector::PseudoClass(CssPseudoClass::Is(
        CssPseudoSelectorList::try_new(vec![selector]).unwrap(),
    ));
    let root = CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector)]).unwrap();
    assert_eq!(
        CssScopeRule::try_new(
            Some(root),
            None,
            Vec::new(),
            &context(),
            CssScopeNestingContext::None
        )
        .unwrap_err()
        .kind(),
        CssRuleConstructionErrorKind::SelectorNestingLimit
    );
}
#[test]
fn exact_rule_depth_boundary_counts_new_ordinary_and_scoped_wrappers() {
    let mut child = rule("@font-face {}");
    for _ in 1..256 {
        child = CssRule::Media(CssMediaRule::try_new(query(), vec![child], &context()).unwrap());
    }
    assert!(CssSheet::try_from_rules(vec![child.clone()]).is_ok());
    let error = CssMediaRule::try_new(query(), vec![child], &context()).unwrap_err();
    assert_eq!(error.kind(), CssRuleConstructionErrorKind::NestingLimit);
    assert_eq!(error.path().len(), 256);
    assert!(error.position().is_some());
    let input = format!(
        "{}@font-face {{}}{}",
        "@media all {".repeat(255),
        "}".repeat(255)
    );
    let child = rule(&input);
    assert_eq!(
        CssMediaRule::try_new(query(), vec![child], &context())
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::NestingLimit
    );
    let mut child = CssScopedRule::FontFace(CssFontFaceRule::new(CssFontFaceDescriptors::new(
        Vec::new(),
    )));
    for _ in 1..256 {
        child = CssScopedRule::Media(
            CssScopedMediaRule::try_new(query(), vec![child], &context()).unwrap(),
        );
    }
    let error = CssScopedMediaRule::try_new(query(), vec![child], &context()).unwrap_err();
    assert_eq!(error.kind(), CssRuleConstructionErrorKind::NestingLimit);
    assert_eq!(error.position(), None);
}
#[test]
fn aggregate_descriptor_front_door_covers_all_fourteen_kinds() {
    use CssFontFaceDescriptorKind as K;
    for (kind, source, expected) in [
        (K::FontFamily, "'Demo Font'", "Demo Font"),
        (K::Src, "local('Demo')", "local(\"Demo\")"),
        (K::FontWeight, "400 4e2", "400"),
        (K::FontStyle, "oblique 10deg 1e1deg", "oblique 10deg"),
        (K::FontWidth, "75% 75.0%", "75%"),
        (K::FontDisplay, "swap", "swap"),
        (K::UnicodeRange, "U+0-7F,U+A0", "U+0-7F, U+A0"),
        (K::FontFeatureSettings, "'liga' off", "\"liga\" off"),
        (K::FontVariationSettings, "'wght' 625", "\"wght\" 625"),
        (K::FontNamedInstance, "''", "\"\""),
        (K::FontLanguageOverride, "'ENG'", "\"ENG\""),
        (K::AscentOverride, "90%", "90%"),
        (K::DescentOverride, "normal", "normal"),
        (K::LineGapOverride, "0%", "0%"),
    ] {
        let value = ordinary(source, kind);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{kind:?}");
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, before);
    }
}
#[test]
fn suppressed_descriptor_occurrences_and_feature_names_charge_work_without_bytes() {
    let input = rule(&format!(
        "@font-face {{ font-family: '{}'; font-family: Demo; }}",
        "suppressed".repeat(100)
    ));
    let expected = "@font-face { font-family: Demo; }";
    let limits = CssSpecifiedValueSerializationLimits::new(5, 5, expected.len());
    assert_eq!(
        input.to_specified_css_with_limits(limits).unwrap(),
        expected
    );
    let input = rule(
        "@font-feature-values Demo { @swash { pretty: 999999999999999999999999999999; } @swash { pretty: 1; } }",
    );
    let expected = "@font-feature-values Demo { @swash { pretty: 1; } }";
    assert_eq!(
        input
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                10,
                10,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for limits in [
        CssSpecifiedValueSerializationLimits::new(9, 10, expected.len()),
        CssSpecifiedValueSerializationLimits::new(10, 9, expected.len()),
        CssSpecifiedValueSerializationLimits::new(10, 10, expected.len() - 1),
    ] {
        assert!(input.to_specified_css_with_limits(limits).is_err());
    }
}
#[test]
fn pending_descriptor_effective_value_uses_shared_authored_owner() {
    let input = rule("@font-face { font-weight: 400; font-weight: env(weight); }");
    let before = input.clone();
    assert_eq!(
        input.to_specified_css().unwrap(),
        "@font-face { font-weight:  env(weight); }"
    );
    assert_eq!(input, before);
    let report =
        parse_font_face_descriptor_value("env(weight)", CssFontFaceDescriptorKind::FontWeight);
    let value = report.syntax().as_ref().unwrap().clone();
    let face = CssFontFaceRule::new(CssFontFaceDescriptors::new(vec![
        CssFontFaceDescriptor::new(value),
    ]));
    assert_eq!(face.position(), None);
    let record = face.descriptors().occurrences().next().unwrap();
    assert_eq!(record.position(), None);
    assert_eq!(
        CssRule::FontFace(face).to_specified_css().unwrap(),
        "@font-face { font-weight: env(weight); }"
    );
}
#[test]
fn complete_font_metadata_keeps_pinned_source_identity() {
    for (id, production) in [
        ("baseline.rule.font-face", "#font-face-rule"),
        (
            "later.rule.font-feature-values",
            "#font-feature-values-syntax",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap();
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.source().id().as_str(), "I-FONTS4-20260907");
        assert_eq!(metadata.production(), production);
        assert!(metadata.unsupported_remainder().is_none());
    }
}

#[test]
fn malformed_font_grammar_keeps_independent_local_recovery_controls() {
    for source in [
        "@font-face { font-weight: 400 500 600; font-display: swap; }",
        "@font-feature-values Demo { @character-variant { bad: 1 2 3; good: 100; } }",
    ] {
        let report = parse_sheet(source);
        assert_eq!(report.syntax().rules().len(), 1);
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDescriptor
        );
    }
}
#[test]
fn first_authored_failure_precedes_later_prelude_order_errors() {
    let input = sheet("a { > b {} }");
    let CssRule::Style(style) = &input.rules()[0] else {
        panic!("style")
    };
    let error =
        CssSheet::try_from_rules(vec![style.rules()[0].clone(), rule("@import 'a';")]).unwrap_err();
    assert_eq!(error.kind(), CssRuleConstructionErrorKind::InvalidPlacement);
    assert_eq!(error.path(), &[0]);
}
#[test]
fn functional_compound_raw_classes_use_the_same_intrinsic_rejection() {
    assert!(CssCompoundSelectorArgument::try_new(CssSelector::Class("\0".into())).is_none());
    let argument =
        CssCompoundSelectorArgument::try_new(CssSelector::Class("escaped name".into())).unwrap();
    let selector = CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument));
    let root = CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector)]).unwrap();
    let scope = CssScopeRule::try_new(
        Some(root.clone()),
        None,
        Vec::new(),
        &context(),
        CssScopeNestingContext::None,
    )
    .unwrap();
    assert_eq!(scope.root(), Some(&root));
}

#[test]
fn semantic_endpoint_shortening_does_not_require_identical_units_or_keyword_branches() {
    for (kind, source, expected) in [
        (
            CssFontFaceDescriptorKind::FontWeight,
            "normal 400",
            "normal",
        ),
        (CssFontFaceDescriptorKind::FontWeight, "bold 700", "bold"),
        (
            CssFontFaceDescriptorKind::FontWidth,
            "condensed 75%",
            "condensed",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "oblique 90deg 0.25turn",
            "oblique 90deg",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "oblique 90deg 100grad",
            "oblique 90deg",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "oblique 0rad 0deg",
            "oblique 0rad",
        ),
    ] {
        assert_eq!(
            ordinary(source, kind).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn extreme_exact_angle_exponents_do_not_become_a_machine_integer_admission_limit() {
    let exponent = "99999999999999999999999999999999999999999999999999";
    let source = format!("oblique 9e-{exponent}deg 10e-{exponent}grad");
    let value = ordinary(&source, CssFontFaceDescriptorKind::FontStyle);
    assert_eq!(value.serialize_specified().unwrap(), "oblique 0deg");
}

#[test]
fn descriptor_rule_order_uses_selected_cssom_relative_order_and_fonts4_append_policy() {
    let input = rule(
        "@font-face {
        line-gap-override: normal; descent-override: normal; ascent-override: normal;
        font-language-override: normal; font-named-instance: auto; font-variation-settings: normal;
        font-display: auto; font-style: italic; font-weight: 400; font-width: 100%;
        font-feature-settings: normal; unicode-range: U+0-7F; src: local(Demo); font-family: Demo;
    }",
    );
    assert_eq!(
        input.to_specified_css().unwrap(),
        "@font-face { font-family: Demo; src: local(\"Demo\"); unicode-range: U+0-7F; font-feature-settings: normal; font-width: 100%; font-weight: 400; font-style: italic; font-display: auto; font-variation-settings: normal; font-named-instance: auto; font-language-override: normal; ascent-override: normal; descent-override: normal; line-gap-override: normal; }"
    );
}

#[test]
fn parsed_scoped_near_limit_subtrees_count_new_group_and_scope_wrappers() {
    // Scope1 + media254 + face1 = the parser's 256 block-bearing levels.
    let source = format!(
        "@scope {{ {}@font-face {{}}{} }}",
        "@media all {".repeat(254),
        "}".repeat(254)
    );
    let parsed = sheet(&source);
    let CssRule::Scope(scope) = &parsed.rules()[0] else {
        panic!("parsed scope")
    };
    let children = scope.rules().rules().to_vec();
    let CssScopedRule::Media(original) = &children[0] else {
        panic!("parsed scoped group")
    };
    let original_position = original.position().expect("parsed child position");
    let wrapped = CssScopedMediaRule::try_new(query(), children.clone(), &context()).unwrap();
    assert_eq!(wrapped.position(), None);
    let CssScopedRule::Media(retained) = &wrapped.rules().rules()[0] else {
        panic!("retained child")
    };
    assert_eq!(retained.position(), Some(original_position));
    assert_eq!(wrapped.rules().rules(), children);
    let constructed_scope = CssScopeRule::try_new(
        None,
        None,
        children,
        &context(),
        CssScopeNestingContext::None,
    )
    .unwrap();
    assert_eq!(constructed_scope.position(), None);
    for error in [
        CssScopedMediaRule::try_new(
            query(),
            vec![CssScopedRule::Scope(scope.clone())],
            &context(),
        )
        .unwrap_err(),
        CssScopeRule::try_new(
            None,
            None,
            vec![CssScopedRule::Media(wrapped)],
            &context(),
            CssScopeNestingContext::None,
        )
        .unwrap_err(),
    ] {
        assert_eq!(error.kind(), CssRuleConstructionErrorKind::NestingLimit);
        assert_eq!(error.path().len(), 256);
        assert_eq!(
            error.position().unwrap().byte_offset().value(),
            source.find("@font-face").unwrap()
        );
    }
}

#[test]
fn equal_symbolic_ranges_discard_bytes_but_retain_both_projection_visits() {
    for (kind, single, repeated, expected) in [
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(400 + 0)",
            "calc(400 + 0) calc(400 + 0)",
            "calc(400)",
        ),
        (
            CssFontFaceDescriptorKind::FontWidth,
            "calc(75% + 0%)",
            "calc(75% + 0%) calc(75% + 0%)",
            "calc(75%)",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "oblique calc(10deg + 0deg)",
            "oblique calc(10deg + 0deg) calc(10deg + 0deg)",
            "oblique calc(10deg)",
        ),
    ] {
        let single = ordinary(single, kind);
        let repeated = ordinary(repeated, kind);
        let before = repeated.clone();
        assert_eq!(single.serialize_specified().unwrap(), expected);
        assert_eq!(
            repeated
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        // Discover each independent single-endpoint boundary through the public
        // limit contract. The suppressed second endpoint must exceed both.
        for input_budget in [true, false] {
            let single_boundary = (0..100)
                .find(|&limit| {
                    let limits = if input_budget {
                        CssSpecifiedValueSerializationLimits::new(limit, 100, expected.len())
                    } else {
                        CssSpecifiedValueSerializationLimits::new(100, limit, expected.len())
                    };
                    single.serialize_specified_with_limits(limits).is_ok()
                })
                .expect("bounded single endpoint");
            let limits = if input_budget {
                CssSpecifiedValueSerializationLimits::new(single_boundary, 100, expected.len())
            } else {
                CssSpecifiedValueSerializationLimits::new(100, single_boundary, expected.len())
            };
            assert_eq!(
                repeated
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                if input_budget {
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                } else {
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
                }
            );
            assert_eq!(repeated, before);
        }
        assert_eq!(
            repeated
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(repeated, before);
    }
}

#[test]
fn overwritten_symbolic_metrics_use_shared_numeric_visits_without_formatting() {
    for name in ["ascent-override", "descent-override", "line-gap-override"] {
        let surviving = rule(&format!("@font-face {{ {name}: 40%; }}"));
        let input = rule(&format!(
            "@font-face {{ {name}: calc(20% + 10%); {name}: 40%; }}"
        ));
        let before = input.clone();
        let expected = format!("@font-face {{ {name}: 40%; }}");
        assert_eq!(
            input
                .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for input_budget in [true, false] {
            let boundary = (0..100)
                .find(|&limit| {
                    let limits = if input_budget {
                        CssSpecifiedValueSerializationLimits::new(limit, 100, expected.len())
                    } else {
                        CssSpecifiedValueSerializationLimits::new(100, limit, expected.len())
                    };
                    surviving.to_specified_css_with_limits(limits).is_ok()
                })
                .expect("bounded surviving metric");
            let limits = if input_budget {
                CssSpecifiedValueSerializationLimits::new(boundary, 100, expected.len())
            } else {
                CssSpecifiedValueSerializationLimits::new(100, boundary, expected.len())
            };
            assert_eq!(
                input
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                CssSpecifiedRuleSerializationErrorKind::Resource(if input_budget {
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                } else {
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
                })
            );
            assert_eq!(input, before);
        }
        assert_eq!(input, before);
    }
}

#[test]
fn overwritten_pending_streams_charge_visits_without_discarded_bytes() {
    let input = rule(&format!(
        "@font-face {{ font-weight: env({}); font-weight: 600; }}",
        "discarded".repeat(100)
    ));
    let surviving = rule("@font-face { font-weight: 600; }");
    let before = input.clone();
    let expected = "@font-face { font-weight: 600; }";
    assert_eq!(
        input
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for input_budget in [true, false] {
        let boundary = (0..100)
            .find(|&limit| {
                let limits = if input_budget {
                    CssSpecifiedValueSerializationLimits::new(limit, 100, expected.len())
                } else {
                    CssSpecifiedValueSerializationLimits::new(100, limit, expected.len())
                };
                surviving.to_specified_css_with_limits(limits).is_ok()
            })
            .expect("bounded surviving descriptor");
        let limits = if input_budget {
            CssSpecifiedValueSerializationLimits::new(boundary, 100, expected.len())
        } else {
            CssSpecifiedValueSerializationLimits::new(100, boundary, expected.len())
        };
        assert_eq!(
            input
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(if input_budget {
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            } else {
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            })
        );
        assert_eq!(input, before);
    }
    assert_eq!(
        input
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedRuleSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        )
    );
    assert_eq!(input, before);
}

#[test]
fn exact_spelling_identity_in_matching_calculations_shortens_ranges() {
    for (kind, first, second, expected) in [
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(400)",
            "calc(4e2)",
            "calc(400)",
        ),
        (
            CssFontFaceDescriptorKind::FontWidth,
            "calc(75%)",
            "calc(75.0%)",
            "calc(75%)",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "calc(10deg)",
            "calc(1e1deg)",
            "oblique calc(10deg)",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "calc(90deg)",
            "calc(.25turn)",
            "oblique calc(90deg)",
        ),
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(1e-999999999999999999999999)",
            "calc(1.0e-999999999999999999999999)",
            "calc(0)",
        ),
    ] {
        let prefix = if kind == CssFontFaceDescriptorKind::FontStyle {
            "oblique "
        } else {
            ""
        };
        let single = ordinary(&format!("{prefix}{first}"), kind);
        let repeated = ordinary(&format!("{prefix}{first} {second}"), kind);
        let before = repeated.clone();
        assert_eq!(single.serialize_specified().unwrap(), expected);
        assert_eq!(
            repeated
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for input_budget in [true, false] {
            let boundary = (0..100)
                .find(|&limit| {
                    let limits = if input_budget {
                        CssSpecifiedValueSerializationLimits::new(limit, 100, expected.len())
                    } else {
                        CssSpecifiedValueSerializationLimits::new(100, limit, expected.len())
                    };
                    single.serialize_specified_with_limits(limits).is_ok()
                })
                .expect("bounded first endpoint");
            let limits = if input_budget {
                CssSpecifiedValueSerializationLimits::new(boundary, 100, expected.len())
            } else {
                CssSpecifiedValueSerializationLimits::new(100, boundary, expected.len())
            };
            assert_eq!(
                repeated
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                if input_budget {
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                } else {
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
                }
            );
            assert_eq!(repeated, before);
        }
        assert_eq!(
            repeated
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(repeated, before);
    }
}

#[test]
fn matching_calculation_structure_does_not_equate_distinct_rounded_leaves() {
    for (kind, source, expected) in [
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(400.0000001) calc(400.0000002)",
            "calc(400) calc(400)",
        ),
        (
            CssFontFaceDescriptorKind::FontWidth,
            "calc(75.0000001%) calc(75.0000002%)",
            "calc(75%) calc(75%)",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "oblique calc(10.0000001deg) calc(10.0000002deg)",
            "oblique calc(10deg) calc(10deg)",
        ),
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(300 + 100) calc(400)",
            "calc(400) calc(400)",
        ),
    ] {
        let value = ordinary(source, kind);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(value, before);
    }
}

#[test]
fn exact_absolute_units_inside_number_cancellation_share_retained_identity() {
    // Values4 §6.3/7 fixes these ratios independently of floating projection.
    for (first, second, expected) in [
        ("calc(96px / 1px)", "calc(1in / 1px)", "calc(96)"),
        ("calc(1s / 1s)", "calc(1000ms / 1s)", "calc(1)"),
        ("calc(1000Hz / 1Hz)", "calc(1kHz / 1Hz)", "calc(1000)"),
        ("calc(1dppx / 1dppx)", "calc(96dpi / 1dppx)", "calc(1)"),
        ("calc(1em / 1px)", "calc(1.0em / 1px)", "calc(1em / 1px)"),
    ] {
        let kind = CssFontFaceDescriptorKind::FontWeight;
        let single = ordinary(first, kind);
        let repeated = ordinary(&format!("{first} {second}"), kind);
        let before = repeated.clone();
        assert_eq!(single.serialize_specified().unwrap(), expected);
        assert_eq!(
            repeated
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for input_budget in [true, false] {
            let boundary = (0..100)
                .find(|&limit| {
                    let limits = if input_budget {
                        CssSpecifiedValueSerializationLimits::new(limit, 100, expected.len())
                    } else {
                        CssSpecifiedValueSerializationLimits::new(100, limit, expected.len())
                    };
                    single.serialize_specified_with_limits(limits).is_ok()
                })
                .expect("bounded cancellation expression");
            let limits = if input_budget {
                CssSpecifiedValueSerializationLimits::new(boundary, 100, expected.len())
            } else {
                CssSpecifiedValueSerializationLimits::new(100, boundary, expected.len())
            };
            assert_eq!(
                repeated
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                if input_budget {
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                } else {
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
                }
            );
            assert_eq!(repeated, before);
        }
        assert_eq!(repeated, before);
    }
}

#[test]
fn numeric_identity_preserves_different_dimensions_context_and_irrational_angles() {
    for (kind, source, expected) in [
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(1px / 1px) calc(1s / 1s)",
            "calc(1) calc(1)",
        ),
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(1em / 1px) calc(1rem / 1px)",
            "calc(1em / 1px) calc(1rem / 1px)",
        ),
        (
            CssFontFaceDescriptorKind::FontWeight,
            "calc(96.0000001dpi / 1dppx) calc(96.0000002dpi / 1dppx)",
            "calc(1) calc(1)",
        ),
        (
            CssFontFaceDescriptorKind::FontStyle,
            "oblique calc(180deg) calc(3.141592653589793rad)",
            "oblique calc(180deg) calc(180deg)",
        ),
    ] {
        let value = ordinary(source, kind);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(value, before);
    }
}

#[test]
fn typed_constant_and_round_strategy_identity_ignores_case_and_escape_spelling() {
    for (first, second, expected) in [
        ("calc(pi)", "calc(PI)", "calc(3.141593)"),
        ("calc(pi)", r"calc(p\69)", "calc(3.141593)"),
        (
            "round(nearest, 400, 10)",
            "round(NEAREST, 400, 10)",
            "calc(400)",
        ),
        (
            "round(nearest, 400, 10)",
            r"round(n\65 arest, 400, 10)",
            "calc(400)",
        ),
    ] {
        let kind = CssFontFaceDescriptorKind::FontWeight;
        let single = ordinary(first, kind);
        let repeated = ordinary(&format!("{first} {second}"), kind);
        let before = repeated.clone();
        assert_eq!(single.serialize_specified().unwrap(), expected);
        assert_eq!(
            repeated
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for input_budget in [true, false] {
            let boundary = (0..100)
                .find(|&limit| {
                    let limits = if input_budget {
                        CssSpecifiedValueSerializationLimits::new(limit, 100, expected.len())
                    } else {
                        CssSpecifiedValueSerializationLimits::new(100, limit, expected.len())
                    };
                    single.serialize_specified_with_limits(limits).is_ok()
                })
                .expect("bounded typed expression");
            let limits = if input_budget {
                CssSpecifiedValueSerializationLimits::new(boundary, 100, expected.len())
            } else {
                CssSpecifiedValueSerializationLimits::new(100, boundary, expected.len())
            };
            assert_eq!(
                repeated
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                if input_budget {
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                } else {
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
                }
            );
            assert_eq!(repeated, before);
        }
        assert_eq!(repeated, before);
    }
}

#[test]
fn typed_expression_identity_preserves_different_constants_strategies_and_rounded_values() {
    for (source, expected) in [
        ("calc(pi) calc(e)", "calc(3.141593) calc(2.718282)"),
        (
            "calc(pi) calc(3.141592653589793)",
            "calc(3.141593) calc(3.141593)",
        ),
        (
            "round(up, 401, 10) round(down, 401, 10)",
            "calc(410) calc(400)",
        ),
        (
            "round(nearest, 400.1, 1) round(down, 400.1, 1)",
            "calc(400) calc(400)",
        ),
        ("calc(400.0000001) calc(400.0000002)", "calc(400) calc(400)"),
    ] {
        let value = ordinary(source, CssFontFaceDescriptorKind::FontWeight);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    100,
                    100,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(value, before);
    }
}
