#![forbid(unsafe_code)]
//! Composed context permissions through public parsing interfaces.
//! Selected Syntax 3 CRD20211224 §§5.3.7–8, 8.1; Nesting WD20260122 §§3–5;
//! catalog's narrow block-contents import and terminal-run reconciliation;
//! Conditional 3 CRD20240815 §3; Animations 1 WD20230302 §3;
//! CSS2 REC20110607 §13.2; Fonts 4 WD20260907 §6.9.1.
//! Shared exhaustive property authority remains common/property_expectations/records.rs.
use surgeist_css::{
    CssDeclarationContextRef, CssDeclarationList, CssErrorCode, CssFontDisplay,
    CssFontFaceDescriptorKind, CssFontFeatureValueKind, CssFontFeatureValuesItem, CssImportance,
    CssKeyframeSelector, CssKnownProperty, CssNamespaceContext, CssRecoveryAction, CssRule,
    ErrorKind, parse_declaration_list_text, parse_sheet, parse_style_block,
};

fn properties(declarations: &CssDeclarationList) -> Vec<CssKnownProperty> {
    declarations
        .iter()
        .map(|declaration| declaration.known().unwrap().property())
        .collect()
}

fn ordered_style_contents(declarations: &CssDeclarationList, rules: &[CssRule]) {
    assert_eq!(properties(declarations), [CssKnownProperty::Color]);
    assert_eq!(declarations[0].importance(), CssImportance::Important);
    let [
        CssRule::NestedDeclarations(blue),
        CssRule::Style(child),
        CssRule::NestedDeclarations(height),
    ] = rules
    else {
        panic!("declaration run after rejected at-rule, nested style, terminal run")
    };
    assert_eq!(properties(blue.declarations()), [CssKnownProperty::Color]);
    assert_eq!(
        properties(child.declarations()),
        [CssKnownProperty::Opacity]
    );
    assert_eq!(
        properties(height.declarations()),
        [CssKnownProperty::Height]
    );
}

#[test]
fn same_authored_contents_have_style_permissions_in_real_brace_front_and_sheet_but_not_raw_list() {
    let body = "color:red!important; @unknown {width:100px;} color:blue; &{opacity:.5}; height:2px";
    let block_report = parse_style_block(&format!("{{{body}}}"), &CssNamespaceContext::default());
    let block = block_report.syntax().as_ref().unwrap();
    ordered_style_contents(block.declarations(), block.rules());
    assert_eq!(
        block_report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::DropAtRule]
    );

    let sheet_report = parse_sheet(&format!(".parent{{{body}}}"));
    let [CssRule::Style(style)] = sheet_report.syntax().rules() else {
        panic!("real parent style")
    };
    ordered_style_contents(style.declarations(), style.rules());
    assert_eq!(
        sheet_report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::DropAtRule]
    );

    let raw_report = parse_declaration_list_text(body);
    assert_eq!(
        properties(raw_report.syntax()),
        [
            CssKnownProperty::Color,
            CssKnownProperty::Color,
            CssKnownProperty::Height
        ]
    );
    assert_eq!(
        raw_report.syntax()[0].importance(),
        CssImportance::Important
    );
    assert_eq!(
        raw_report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropDeclaration,
            CssRecoveryAction::DropDeclaration
        ]
    );
    assert!(block_report.into_validation_result().is_err());
    assert!(sheet_report.into_validation_result().is_err());
    assert!(raw_report.into_validation_result().is_err());
}

#[test]
fn terminal_priority_is_admitted_for_cascade_properties_and_recovered_locally_for_keyframes_and_descriptors()
 {
    let source = concat!(
        "a{width:1px!important; height:2px}",
        "@keyframes k{from{width:1px!important; height:2px}}",
        "@font-face{font-family:Demo; font-display:swap!important; src:url(demo); font-weight:400}",
        "b{opacity:.5}",
    );
    let report = parse_sheet(source);
    let [
        CssRule::Style(style),
        CssRule::Keyframes(keyframes),
        CssRule::FontFace(face),
        CssRule::Style(after),
    ] = report.syntax().rules()
    else {
        panic!("all owning rules and later sibling retained")
    };
    assert_eq!(
        properties(style.declarations()),
        [CssKnownProperty::Width, CssKnownProperty::Height]
    );
    assert_eq!(
        style.declarations()[0].importance(),
        CssImportance::Important
    );
    assert_eq!(keyframes.blocks().len(), 1);
    let [height] = keyframes.blocks()[0].declarations().as_slice() else {
        panic!("only normal keyframe sibling")
    };
    assert_eq!(height.known().unwrap().property(), CssKnownProperty::Height);
    assert_eq!(height.source().importance(), CssImportance::Normal);
    assert!(
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::FontDisplay)
            .is_none()
    );
    assert_eq!(
        face.descriptors()
            .occurrences()
            .map(|d| d.value().kind())
            .collect::<Vec<_>>(),
        [
            CssFontFaceDescriptorKind::FontFamily,
            CssFontFaceDescriptorKind::Src,
            CssFontFaceDescriptorKind::FontWeight
        ]
    );
    assert_eq!(
        properties(after.declarations()),
        [CssKnownProperty::Opacity]
    );
    let [keyframe_diagnostic, descriptor_diagnostic] = report.diagnostics() else {
        panic!("two context priority failures")
    };
    assert_eq!(
        keyframe_diagnostic.action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_eq!(
        descriptor_diagnostic.action(),
        CssRecoveryAction::DropDescriptor
    );
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDeclarationAnnotation
        );
        assert_eq!(
            &source[diagnostic.error().position().byte_offset().value()..][..1],
            "!"
        );
    }
    let ErrorKind::InvalidDeclarationAnnotation(keyframe) = keyframe_diagnostic.error().kind()
    else {
        panic!("keyframe annotation error")
    };
    assert!(matches!(
        keyframe.context(),
        CssDeclarationContextRef::Keyframe(_)
    ));
    let ErrorKind::InvalidDeclarationAnnotation(descriptor) = descriptor_diagnostic.error().kind()
    else {
        panic!("descriptor annotation error")
    };
    let CssDeclarationContextRef::Descriptor {
        at_rule,
        descriptor,
    } = descriptor.context()
    else {
        panic!("font descriptor context")
    };
    assert_eq!(
        (at_rule.as_str(), descriptor.as_str()),
        ("font-face", "font-display")
    );
}

#[test]
fn keyframe_rule_list_rejects_at_rules_and_arbitrary_qualified_children_between_defined_blocks() {
    let source = "@keyframes k{from{opacity:0} @media all{to{opacity:.5}} .bad{opacity:.5} to{opacity:1}} a{width:1px}";
    let report = parse_sheet(source);
    let [CssRule::Keyframes(keyframes), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("keyframes and later sibling")
    };
    let [from, to] = keyframes.blocks() else {
        panic!("only defined keyframe children")
    };
    assert_eq!(from.selectors().selectors(), [CssKeyframeSelector::From]);
    assert_eq!(to.selectors().selectors(), [CssKeyframeSelector::To]);
    assert_eq!(from.declarations().len(), 1);
    assert_eq!(to.declarations().len(), 1);
    assert_eq!(properties(after.declarations()), [CssKnownProperty::Width]);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropKeyframeBlock,
            CssRecoveryAction::DropKeyframeBlock
        ]
    );
    for (diagnostic, unit) in report
        .diagnostics()
        .iter()
        .zip(["@media all{to{opacity:.5}}", ".bad{opacity:.5}"])
    {
        let start = source.find(unit).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
    }
}

#[test]
fn selected_page_consumer_removes_child_rules_without_losing_margin_priority_or_later_siblings() {
    // CSS2 §13.2 allows generic at-rule candidates but defines no child at-rule.
    // Page3 margin-box semantics are not selected by this repository's catalog.
    let source = "@page{margin-top:1px!important; @top-left{content:'title'} .nested{color:red} margin-bottom:2px} a{width:3px}";
    let report = parse_sheet(source);
    let [CssRule::Page(page), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("page and later style")
    };
    assert_eq!(
        properties(page.declarations()),
        [CssKnownProperty::MarginTop, CssKnownProperty::MarginBottom]
    );
    assert_eq!(
        page.declarations()[0].importance(),
        CssImportance::Important
    );
    assert_eq!(properties(after.declarations()), [CssKnownProperty::Width]);
    let [margin_child, nested_style] = report.diagnostics() else {
        panic!("two disallowed children")
    };
    assert_eq!(margin_child.error().code(), CssErrorCode::UnsupportedAtRule);
    assert_eq!(nested_style.error().code(), CssErrorCode::InvalidAtRuleBody);
    assert_eq!(margin_child.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(nested_style.action(), CssRecoveryAction::DropAtRule);
}

#[test]
fn font_feature_values_mixed_body_retains_defined_blocks_and_display_while_removing_other_children()
{
    // Selected Fonts4 §6.9.1 expressly admits font-display and defined blocks.
    let source = "@font-feature-values Demo{font-display:swap; @stylistic{first:1} @unknown{lost:2} .nested{color:red}; @swash{last:3}} a{width:1px}";
    let report = parse_sheet(source);
    let [CssRule::FontFeatureValues(features), CssRule::Style(after)] = report.syntax().rules()
    else {
        panic!("font owner and later sibling")
    };
    let [
        CssFontFeatureValuesItem::FontDisplay(display),
        CssFontFeatureValuesItem::Block(stylistic),
        CssFontFeatureValuesItem::Block(swash),
    ] = features.items()
    else {
        panic!("display descriptor and two defined subsidiary blocks")
    };
    assert_eq!(display.value(), CssFontDisplay::Swap);
    assert_eq!(stylistic.kind(), CssFontFeatureValueKind::Stylistic);
    assert_eq!(swash.kind(), CssFontFeatureValueKind::Swash);
    assert_eq!(
        stylistic
            .definitions()
            .iter()
            .map(|d| d.name().as_str())
            .collect::<Vec<_>>(),
        ["first"]
    );
    assert_eq!(
        swash
            .definitions()
            .iter()
            .map(|d| d.name().as_str())
            .collect::<Vec<_>>(),
        ["last"]
    );
    assert_eq!(
        stylistic.definitions()[0].indexes()[0].as_decimal_str(),
        "1"
    );
    assert_eq!(swash.definitions()[0].indexes()[0].as_decimal_str(), "3");
    assert_eq!(properties(after.declarations()), [CssKnownProperty::Width]);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropAtRule,
            CssRecoveryAction::DropDescriptor
        ]
    );
}

#[test]
fn malformed_font_mixed_list_unit_consumes_an_unseparated_tail_only_to_child_eof() {
    // Syntax 2021 §5.4.5 discards a nonident component unit through the next
    // root semicolon or bounded EOF. A nested brace does not end that unit.
    let source = "@font-feature-values Demo{font-display:swap; @stylistic{first:1} @unknown{lost:2} .nested{color:red} @swash{last:3}} a{width:1px}";
    let report = parse_sheet(source);
    let [CssRule::FontFeatureValues(features), CssRule::Style(after)] = report.syntax().rules()
    else {
        panic!("font owner and outer later sibling")
    };
    let [
        CssFontFeatureValuesItem::FontDisplay(display),
        CssFontFeatureValuesItem::Block(stylistic),
    ] = features.items()
    else {
        panic!("only the two members before the malformed unit")
    };
    assert_eq!(display.value(), CssFontDisplay::Swap);
    assert_eq!(stylistic.kind(), CssFontFeatureValueKind::Stylistic);
    let [first] = stylistic.definitions() else {
        panic!("retained stylistic definition")
    };
    assert_eq!(first.name().as_str(), "first");
    assert_eq!(first.indexes()[0].as_decimal_str(), "1");
    assert_eq!(properties(after.declarations()), [CssKnownProperty::Width]);
    let [unknown, malformed] = report.diagnostics() else {
        panic!("one unknown child and one complete malformed unit")
    };
    assert_eq!(unknown.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(malformed.action(), CssRecoveryAction::DropDescriptor);
    let unit = ".nested{color:red} @swash{last:3}";
    let start = source.find(unit).unwrap();
    assert_eq!(malformed.span().start().byte_offset().value(), start);
    assert_eq!(
        malformed.span().end().byte_offset().value(),
        start + unit.len()
    );
    assert_eq!(malformed.error().position().byte_offset().value(), start);
}

#[test]
fn same_conditional_body_rejects_top_level_declarations_and_retains_style_ancestor_ordered_runs() {
    let body = "height:2px; a{opacity:.5} width:1px;";
    let top = parse_sheet(&format!("@media all{{{body}}} b{{color:red}}"));
    let [CssRule::Media(media), CssRule::Style(after)] = top.syntax().rules() else {
        panic!("top-level group and sibling")
    };
    assert!(media.rules().is_empty());
    assert_eq!(properties(after.declarations()), [CssKnownProperty::Color]);
    // §5.4.1's qualified prelude includes the root semicolon: the first
    // declaration-looking text owns a{}; the final candidate ends at child EOF.
    assert_eq!(
        top.diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropQualifiedRule,
            CssRecoveryAction::DropQualifiedRule
        ]
    );

    let nested = parse_sheet(&format!(".parent{{@media all{{{body}}}}}"));
    assert!(nested.is_clean(), "{nested:?}");
    let [CssRule::Style(parent)] = nested.syntax().rules() else {
        panic!("actual style ancestor")
    };
    assert!(parent.declarations().is_empty());
    let [CssRule::Media(media)] = parent.rules() else {
        panic!("nested conditional group")
    };
    let [
        CssRule::NestedDeclarations(height),
        CssRule::Style(child),
        CssRule::NestedDeclarations(width),
    ] = media.rules()
    else {
        panic!("declaration, nested style, terminal declaration run")
    };
    assert_eq!(
        properties(height.declarations()),
        [CssKnownProperty::Height]
    );
    assert_eq!(
        properties(child.declarations()),
        [CssKnownProperty::Opacity]
    );
    assert_eq!(properties(width.declarations()), [CssKnownProperty::Width]);
}
