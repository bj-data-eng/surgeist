#![forbid(unsafe_code)]
//! Public consumer composition: checked CSS payloads without semantic text round trips.

use surgeist_css::{
    CssAdmittedRule, CssDeclaration, CssDeclarationBlockErrorKind, CssEditedGroupPreludeRef,
    CssEditedRuleView, CssImportance, CssInvalidKeyframeSourceReason, CssKeyframeRuleView,
    CssKeyframeRuleViewError, CssKnownProperty, CssNamespaceContext, CssNamespaceName,
    CssNamespacePrefix, CssPropertyNameRef, CssRecoveryAction, CssRule, CssRuleAdmissionContext,
    CssRuleCssomSerializationErrorKind, CssSpecifiedDeclarationBlock, CssSpecifiedDeclarationEntry,
    CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, classify_rule_syntax, parse_component_values,
    parse_declaration_block_contents, parse_keyframe_rule, parse_keyframe_selector_list,
    parse_property_value,
};

fn terminal(
    block: &CssSpecifiedDeclarationBlock,
    property: CssKnownProperty,
) -> &CssSpecifiedDeclarationEntry {
    block
        .entries()
        .iter()
        .find(|entry| matches!(entry.property_name(), CssPropertyNameRef::Known(name) if name == property))
        .expect("requested checked terminal")
}

fn custom<'a>(
    block: &'a CssSpecifiedDeclarationBlock,
    name: &str,
) -> &'a CssSpecifiedDeclarationEntry {
    block
        .entries()
        .iter()
        .find(|entry| matches!(entry.property_name(), CssPropertyNameRef::Custom(found) if found.as_str() == name))
        .expect("requested case-sensitive custom terminal")
}

fn original_value(declaration: &CssDeclaration) -> &str {
    let parsed = declaration.parsed_value().expect("authored value origin");
    let span = parsed.span();
    &parsed.source().as_str()[span.start().byte_offset().value()..span.end().byte_offset().value()]
}

#[test]
fn recovered_raw_terminals_form_an_edited_media_child_after_inputs_drop_and_budget_retry() {
    const RAW: &str =
        "/*😀*/\r\nmargin:1px 2px 3px 4px; --Case:日本; .skip{width:9px} opacity:.5!important";
    let (edited, margin_origin, opacity_origin) = {
        let input = RAW.to_owned();
        let report = parse_declaration_block_contents(&input);
        assert!(
            !report.is_clean(),
            "a nested rule is recovered out of the declaration block"
        );
        let projected =
            CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
        let right = terminal(&projected, CssKnownProperty::MarginRight);
        let opacity = terminal(&projected, CssKnownProperty::Opacity);
        let selected = [
            right.with_importance(CssImportance::Important),
            custom(&projected, "--Case").clone(),
            opacity.with_importance(CssImportance::Normal),
        ];
        (
            CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap(),
            right.source().clone(),
            opacity.source().clone(),
        )
    };
    let selectors = {
        let input = ".composed {}".to_owned();
        let classified = classify_rule_syntax(&input);
        assert!(classified.is_clean());
        let admitted = classified.syntax().as_ref().unwrap().admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Stylesheet,
        );
        assert!(admitted.is_clean());
        let Some(CssAdmittedRule::Ordinary(CssRule::Style(style))) = admitted.syntax() else {
            panic!("admitted style")
        };
        style.selectors().clone()
    };
    let query = {
        let input = "@media print {}".to_owned();
        let classified = classify_rule_syntax(&input);
        let admitted = classified.syntax().as_ref().unwrap().admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Stylesheet,
        );
        assert!(admitted.is_clean());
        let Some(CssAdmittedRule::Ordinary(CssRule::Media(media))) = admitted.syntax() else {
            panic!("admitted media prelude")
        };
        media.query().clone()
    };

    // CSS2 four-side assignment gives the second supplied margin value to right.
    // Current CSSOM serializes selected declarations in order and cannot recover
    // a four-sided shorthand from this incomplete selection.
    assert_eq!(
        edited
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::Margin))
            .unwrap(),
        None
    );
    let style = CssEditedRuleView::try_style(&selectors, &edited, &[]).unwrap();
    assert_eq!(
        style.serialize_cssom().unwrap(),
        ".composed { margin-right: 2px !important; --Case: 日本; opacity: 0.5; }"
    );
    let children = [style];
    let media =
        CssEditedRuleView::try_group(CssEditedGroupPreludeRef::Media(&query), &children).unwrap();
    const EXPECTED: &str = "@media print {\n  .composed { margin-right: 2px !important; --Case: 日本; opacity: 0.5; }\n}";
    let error = media
        .serialize_cssom_with_limits(Limits::new(100_000, 100_000, EXPECTED.len() - 1))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssRuleCssomSerializationErrorKind::Resource(Resource::ByteLimit)
    );
    assert_eq!(
        media
            .serialize_cssom_with_limits(Limits::new(100_000, 100_000, EXPECTED.len()))
            .unwrap(),
        EXPECTED
    );
    assert_eq!(media.serialize_cssom().unwrap(), EXPECTED);

    // The typed edited wrappers are programmatic; surviving occurrences are authored.
    assert!(style.parsed_rule().is_none());
    assert!(media.parsed_rule().is_none());
    let right = terminal(&edited, CssKnownProperty::MarginRight);
    let opacity = terminal(&edited, CssKnownProperty::Opacity);
    assert!(right.source().same_occurrence(&margin_origin));
    assert!(opacity.source().same_occurrence(&opacity_origin));
    assert_eq!(right.importance(), CssImportance::Important);
    assert_eq!(right.source().importance(), CssImportance::Normal);
    assert_eq!(opacity.importance(), CssImportance::Normal);
    assert_eq!(opacity.source().importance(), CssImportance::Important);
    assert_eq!(original_value(right.source()), "1px 2px 3px 4px");
    assert_eq!(original_value(opacity.source()), ".5");
    assert_eq!(
        right.source().parsed_value().unwrap().source().as_str(),
        RAW
    );
    assert!(
        right
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(opacity.source().parsed_value().unwrap().source())
    );
}

#[test]
fn contextual_retry_and_recovered_rule_publish_a_selected_snapshot_with_original_origins() {
    const INPUT: &str = "/*😀*/\r\n& > svg|leaf { padding:1px 2px; opacity:.25; rubbish; }";
    let (style, snapshot) = {
        let input = INPUT.to_owned();
        let preliminary = classify_rule_syntax(&input);
        assert!(preliminary.is_clean());
        let syntax = preliminary.syntax().as_ref().unwrap();
        let snapshot = syntax.origin().source().clone();
        let rejected = syntax.admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Style,
        );
        assert!(rejected.syntax().is_none());
        let bindings = CssNamespaceContext::from_bindings([(
            Some(CssNamespacePrefix::try_new("svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        )]);
        let accepted = syntax.admit(&bindings, CssRuleAdmissionContext::Style);
        assert!(!accepted.is_clean());
        assert!(
            accepted
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDeclaration })
        );
        assert!(accepted.clone().into_validation_result().is_err());
        let Some(CssAdmittedRule::Ordinary(CssRule::Style(style))) = accepted.into_parts().0 else {
            panic!("recovered nested style retained after contextual admission")
        };
        (style, snapshot)
    };

    let projected =
        CssSpecifiedDeclarationBlock::try_from_declarations(style.declarations()).unwrap();
    let opacity = terminal(&projected, CssKnownProperty::Opacity);
    let selected = [opacity.with_importance(CssImportance::Important)];
    let edited = CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap();
    let view = CssEditedRuleView::try_style(style.selectors(), &edited, &[]).unwrap();
    // The explicit nesting anchor remains in the supplied receiving context.
    // No enclosing stylesheet or reparsed formatted text is needed.
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "& > svg|leaf { opacity: 0.25 !important; }"
    );
    assert!(view.parsed_rule().is_none());
    assert_eq!(snapshot.as_str(), INPUT);
    assert_eq!(
        style.position().byte_offset().value(),
        INPUT.find('&').unwrap()
    );
    let retained = &edited.entries()[0];
    assert!(retained.source().same_occurrence(opacity.source()));
    assert_eq!(retained.source().position(), opacity.source().position());
    assert!(snapshot.same_snapshot(retained.source().parsed_value().unwrap().source()));
    assert_eq!(original_value(retained.source()), ".25");
    assert_eq!(retained.source().importance(), CssImportance::Normal);
    assert_eq!(retained.importance(), CssImportance::Important);
    assert_eq!(style.declarations().len(), 2);
}

#[test]
fn detached_keyframe_accepts_checked_replacements_and_preserves_authored_and_programmatic_origins()
{
    const INPUT: &str = "/*😀*/\r\nFROM,25%,to { margin:1px 2px; opacity:.4; --Tone:日本 }";
    let (authored, original_opacity, selectors) = {
        let input = INPUT.to_owned();
        let parsed = parse_keyframe_rule(&input);
        assert!(parsed.is_clean(), "{parsed:?}");
        let block = parsed.syntax().as_ref().unwrap().block();
        let authored =
            CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(block.declarations())
                .unwrap();
        let original_opacity = terminal(&authored, CssKnownProperty::Opacity)
            .source()
            .clone();
        let replacement_input = "to, from, 25%, 25%".to_owned();
        let replacement = parse_keyframe_selector_list(&replacement_input);
        assert!(replacement.is_clean());
        (
            authored,
            original_opacity,
            replacement.syntax().as_ref().unwrap().selectors().clone(),
        )
    };
    let programmatic = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Opacity),
        parse_component_values(".75").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let supplied = surgeist_css::CssDeclarationList::try_new(vec![programmatic.clone()]).unwrap();
    let supplied = CssSpecifiedDeclarationBlock::try_from_declarations(&supplied).unwrap();
    let replacement = terminal(&supplied, CssKnownProperty::Opacity).clone();
    let right = terminal(&authored, CssKnownProperty::MarginRight).clone();
    let selected = [right, replacement, custom(&authored, "--Tone").clone()];

    let mut invalid = selected.clone();
    invalid[1] = invalid[1].with_importance(CssImportance::Important);
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&invalid)
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::InvalidKeyframeSource {
            reason: CssInvalidKeyframeSourceReason::Importance
        }
    ));
    let edited = CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&selected).unwrap();
    drop(authored);
    drop(supplied);
    let view = CssKeyframeRuleView::try_new(&selectors, &edited).unwrap();
    assert_eq!(
        selectors.serialize_key_text().unwrap(),
        "100%, 0%, 25%, 25%"
    );
    const EXPECTED: &str = "100%, 0%, 25%, 25% { margin-right: 2px; opacity: 0.75; --Tone: 日本; }";
    assert!(matches!(
        view.serialize_cssom_with_limits(Limits::new(100_000, 100_000, EXPECTED.len() - 1)),
        Err(CssKeyframeRuleViewError::Serialization(error)) if error.kind() == Resource::ByteLimit
    ));
    assert_eq!(
        view.serialize_cssom_with_limits(Limits::new(100_000, 100_000, EXPECTED.len()))
            .unwrap(),
        EXPECTED
    );
    assert_eq!(view.serialize_cssom().unwrap(), EXPECTED);

    let right = terminal(&edited, CssKnownProperty::MarginRight);
    let opacity = terminal(&edited, CssKnownProperty::Opacity);
    assert_eq!(
        right.source().parsed_value().unwrap().source().as_str(),
        INPUT
    );
    assert_eq!(original_value(right.source()), "1px 2px");
    assert!(
        right
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(
                custom(&edited, "--Tone")
                    .source()
                    .parsed_value()
                    .unwrap()
                    .source()
            )
    );
    assert_eq!(original_value(&original_opacity), ".4");
    assert!(opacity.source().same_occurrence(&programmatic));
    assert!(opacity.source().position().is_none());
    assert!(opacity.source().parsed_value().is_none());
    assert_eq!(programmatic.importance(), CssImportance::Normal);
}
