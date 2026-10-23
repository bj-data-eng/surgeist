use surgeist_css::*;

fn programmatic(text: &str) -> Vec<CssComponentValue> {
    text.split_whitespace()
        .map(|token| CssComponentValue::try_token(token).unwrap())
        .collect()
}

#[test]
fn parsed_terminator_is_retained_by_constructed_supports_declaration() {
    let parsed = parse_component_values("/* backing belongs only to semicolon */;").unwrap();
    let semicolon = parsed.items().last().unwrap().clone();
    let CssValueOrigin::Parsed(origin) = semicolon.origin() else {
        panic!("parsed semicolon");
    };
    let source = origin.source().clone();
    let mut components = programmatic("x : y");
    components.push(semicolon);
    let body = CssSupportsTestBody::try_from_components(
        CssComponentValues::try_new(components, Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let rule = CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--test").unwrap(),
        body,
        Default::default(),
    )
    .unwrap();
    let sheet = CssSheet::try_from_rules(vec![CssRule::SupportsCondition(rule)]).unwrap();
    drop(parsed);
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&sheet),
        Err(CssSourceRetentionError::Limit("retained source bytes")),
        "retained semicolon source has {} backing bytes",
        source.storage_usage().retained_bytes()
    );
}

#[test]
fn selected_supports_subcondition_retains_its_whole_lexical_root() {
    let parsed = parse_component_values("/* sibling backing */(x:y)").unwrap();
    let mut components = vec![
        CssComponentValue::try_block(
            CssBlockKind::Parenthesis,
            CssComponentValues::try_new(programmatic("x : y"), Default::default()).unwrap(),
        )
        .unwrap(),
    ];
    components.extend(programmatic("and"));
    components.push(parsed.items().last().unwrap().clone());
    let parent = CssSupportsCondition::try_from_components(
        CssComponentValues::try_new(components, Default::default()).unwrap(),
        &CssNamespaceContext::default(),
        Default::default(),
    )
    .unwrap();
    let CssSupportsConditionKind::And(list) = parent.kind() else {
        panic!("and");
    };
    let child = list.conditions()[0].clone();
    let rule = CssSupportsRule::try_new(child, vec![], &CssNamespaceContext::default()).unwrap();
    let sheet = CssSheet::try_from_rules(vec![CssRule::Supports(rule)]).unwrap();
    drop(parent);
    drop(parsed);
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&sheet),
        Err(CssSourceRetentionError::Limit("retained source bytes"))
    );
}

#[test]
fn completed_specified_entry_retains_independently_parsed_replacement() {
    let variable = CssComponentValue::try_function(
        "var",
        CssComponentValues::try_new(
            vec![CssComponentValue::try_ident("--width").unwrap()],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        CssComponentValues::try_new(vec![variable], Default::default()).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
        panic!("pending width");
    };
    let replacement = parse_component_values("/* independent backing */1px").unwrap();
    let completed = pending.reenter(replacement).unwrap();
    let block = CssSpecifiedDeclarationBlock::try_from_expansions(
        &[CssExpansion::Contributions(completed)],
        Default::default(),
    )
    .unwrap();
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&block),
        Err(CssSourceRetentionError::Limit("retained source bytes"))
    );
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&block.entries()[0]),
        Err(CssSourceRetentionError::Limit("retained source bytes"))
    );
}

#[test]
fn selected_when_subcondition_retains_its_whole_lexical_root() {
    let parsed = parse_component_values("/* hidden sibling */media(height)").unwrap();
    let first = CssComponentValue::try_function(
        "media",
        CssComponentValues::try_new(programmatic("width"), Default::default()).unwrap(),
    )
    .unwrap();
    let mut components = vec![first];
    components.extend(programmatic("and"));
    components.push(parsed.items().last().unwrap().clone());
    let parent = CssWhenCondition::try_from_components(
        CssComponentValues::try_new(components, Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssWhenConditionKind::And(list) = parent.kind() else {
        panic!("and");
    };
    let child = list.conditions()[0].clone();
    let rule = CssWhenRule::try_new(child, vec![], &CssNamespaceContext::default()).unwrap();
    let sheet = CssSheet::try_from_rules(vec![CssRule::When(rule)]).unwrap();
    drop(parent);
    drop(parsed);
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&sheet),
        Err(CssSourceRetentionError::Limit("retained source bytes"))
    );
}

#[test]
fn universal_and_independent_svg_entries_retain_completed_replacement_contexts() {
    for (property, replacement) in [
        (CssPropertyNameRef::Known(CssKnownProperty::All), "inherit"),
        (CssPropertyNameRef::SvgGlyphOrientationVertical, "90deg"),
    ] {
        let variable = CssComponentValue::try_function(
            "var",
            CssComponentValues::try_new(
                vec![CssComponentValue::try_ident("--value").unwrap()],
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let declaration = CssParserContext::default()
            .with_svg_glyph_orientation_vertical()
            .parse_property_value(
                property,
                CssComponentValues::try_new(vec![variable], Default::default()).unwrap(),
                CssImportance::Normal,
            )
            .unwrap();
        let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
            panic!("pending");
        };
        let completed = pending
            .reenter(parse_component_values(replacement).unwrap())
            .unwrap();
        let block = CssSpecifiedDeclarationBlock::try_from_expansions(
            &[CssExpansion::Contributions(completed)],
            Default::default(),
        )
        .unwrap();
        assert!(!block.entries().is_empty());
        for entry in block.entries() {
            let mut sources = CssSourceRetention::new(0, 100_000, 100);
            assert_eq!(
                sources.visit(entry),
                Err(CssSourceRetentionError::Limit("retained source bytes"))
            );
        }
    }
}
