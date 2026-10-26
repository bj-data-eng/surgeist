use surgeist_css::*;

fn programmatic(text: &str) -> Vec<CssComponentValue> {
    text.split_whitespace()
        .map(|token| CssComponentValue::try_token(token).unwrap())
        .collect()
}

#[test]
fn parsed_terminator_is_retained_by_constructed_supports_declaration() {
    let parsed = CssComponentValues::parse(
        &CssParser::default(),
        "/* backing belongs only to semicolon */;",
    )
    .unwrap();
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
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        )),
        "retained semicolon source has {} backing bytes",
        source.storage_usage().retained_bytes()
    );
}

#[test]
fn selected_supports_subcondition_retains_its_whole_lexical_root() {
    let parsed =
        CssComponentValues::parse(&CssParser::default(), "/* sibling backing */(x:y)").unwrap();
    let mut components = vec![
        CssComponentValue::try_block(
            CssBlockKind::Parenthesis,
            CssComponentValues::try_new(programmatic("x : y"), Default::default()).unwrap(),
        )
        .unwrap(),
    ];
    components.extend(programmatic("and"));
    components.push(parsed.items().last().unwrap().clone());
    let parent = CssCondition::<CssSupportsLeaf>::try_from_components(
        CssComponentValues::try_new(components, Default::default()).unwrap(),
        &CssNamespaceContext::default(),
        Default::default(),
    )
    .unwrap();
    let CssConditionKind::And(list) = parent.kind() else {
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
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        ))
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
    let declaration = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
        CssComponentValues::try_new(vec![variable], Default::default()).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssExpansion::Pending(pending) = declaration.expand().unwrap() else {
        panic!("pending width");
    };
    let replacement =
        CssComponentValues::parse(&CssParser::default(), "/* independent backing */1px").unwrap();
    let completed = pending.reenter(replacement).unwrap();
    let block = CssSpecifiedDeclarationBlock::try_from_expansions(
        &[CssExpansion::Contributions(completed)],
        Default::default(),
    )
    .unwrap();
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&block),
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        ))
    );
    let mut sources = CssSourceRetention::new(0, 100_000, 100);
    assert_eq!(
        sources.visit(&block.entries()[0]),
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        ))
    );
}

#[test]
fn selected_when_subcondition_retains_its_whole_lexical_root() {
    let parsed =
        CssComponentValues::parse(&CssParser::default(), "/* hidden sibling */media(height)")
            .unwrap();
    let first = CssComponentValue::try_function(
        "media",
        CssComponentValues::try_new(programmatic("width"), Default::default()).unwrap(),
    )
    .unwrap();
    let mut components = vec![first];
    components.extend(programmatic("and"));
    components.push(parsed.items().last().unwrap().clone());
    let parent = CssCondition::<CssWhenLeaf>::try_from_components(
        CssComponentValues::try_new(components, Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssConditionKind::And(list) = parent.kind() else {
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
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        ))
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
        let declaration = CssDeclaration::try_from_components(
            &CssParser::new(
                CssParserContext::default().with_svg_glyph_orientation_vertical(),
                Default::default(),
            ),
            CssPropertyTarget::Property(property),
            CssComponentValues::try_new(vec![variable], Default::default()).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let CssExpansion::Pending(pending) = declaration.expand().unwrap() else {
            panic!("pending");
        };
        let completed = pending
            .reenter(CssComponentValues::parse(&CssParser::default(), replacement).unwrap())
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
                Err(CssSourceRetentionError::limit(
                    CssSourceRetentionResource::RetainedBytes
                ))
            );
        }
    }
}
