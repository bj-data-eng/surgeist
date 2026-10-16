use std::sync::Arc;
use surgeist_css::*;
use surgeist_css::{CssFontFaceRule as AuthoredFontFaceRule, CssRule as AuthoredRule};
use surgeist_cssom::*;

fn inputs(name: &str) -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: name.into(),
            revision: 1,
        },
        true,
    )
}
fn parsed(text: &str) -> CssParseReport<CssSheet> {
    let mut units = Vec::with_capacity(4096);
    units.extend(text.encode_utf16());
    CssParserContext::default()
        .parse_sheet_utf16_with_limits(Arc::new(units), CssStylesheetInputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
}
fn owner(bytes: usize) -> CssomStore {
    CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: bytes,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap()
}
fn create(owner: &mut CssomStore, sheet: CssSheet, name: &str) -> CssomSheetId {
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot()))
        .unwrap();
    let ticket = edit.create_typed_sheet(sheet, inputs(name)).unwrap();
    owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}

#[test]
fn independently_constructed_font_strings_keep_their_actual_semantic_leaf_source() {
    for (property, kind) in [
        (
            "font-named-instance",
            CssFontFaceDescriptorKind::FontNamedInstance,
        ),
        (
            "font-language-override",
            CssFontFaceDescriptorKind::FontLanguageOverride,
        ),
    ] {
        let report = parsed(&format!("@font-face{{{property}:\"Thing\"}}"));
        let bytes = report
            .syntax()
            .parse_source()
            .unwrap()
            .source()
            .storage_usage()
            .retained_bytes();
        let AuthoredRule::FontFace(rule) = &report.syntax().rules()[0] else {
            panic!("font face");
        };
        let value = rule.descriptors().effective(kind).unwrap().value().clone();
        // Fresh descriptor/rule/sheet owners have no enclosing source, so only
        // the actual semantic leaf can retain and charge the original backing.
        let rule = AuthoredFontFaceRule::new(surgeist_css::CssFontFaceDescriptors::new(vec![
            CssFontFaceDescriptor::new(value),
        ]));
        let graph = CssSheet::try_from_rules(vec![AuthoredRule::FontFace(rule)]).unwrap();
        assert!(graph.parse_source().is_none());
        let mut short = owner(bytes - 1);
        let mut edit = short
            .batch(CssomEditGuard::from_snapshot(&short.snapshot()))
            .unwrap();
        assert!(matches!(
            edit.create_typed_sheet(graph.clone(), inputs("font")),
            Err(CssomError::Limit {
                resource: "retained parse source bytes",
                ..
            })
        ));
        assert!(matches!(short.commit(edit), Err(CssomError::BatchAborted)));
        let mut exact = owner(bytes);
        create(&mut exact, graph, "font");
    }
}

#[test]
fn linked_input_retains_the_entire_snapshot_and_deduplicates_repeated_links() {
    let report = parsed("/* source reachable only through a retained foreign state */");
    let bytes = report
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .storage_usage()
        .retained_bytes();
    let mut foreign = owner(bytes);
    let selected = create(&mut foreign, CssSheet::new(), "selected-empty");
    create(&mut foreign, report.into_parts().0, "unselected-source");
    let link = CssomLinkedInput {
        version: CssomInputVersion {
            role: CssomInputRole::Import,
            identity: "foreign".into(),
            revision: 1,
        },
        snapshot: foreign.snapshot(),
        sheet: selected,
    };
    let context = CssomContext {
        inputs: vec![],
        linked: vec![link.clone(), link],
    };
    assert!(matches!(
        CssomStore::new(
            CssomLimits {
                max_retained_source_bytes: bytes - 1,
                ..Default::default()
            },
            context.clone()
        ),
        Err(CssomError::Limit {
            resource: "retained parse source bytes",
            ..
        })
    ));
    CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: bytes,
            ..Default::default()
        },
        context.clone(),
    )
    .unwrap();

    // A current context retains the whole intermediate snapshot, including its
    // links, even though both selected sheets have no parsed rules or source.
    let mut intermediate = CssomStore::new(CssomLimits::default(), context).unwrap();
    let selected = create(&mut intermediate, CssSheet::new(), "intermediate-empty");
    let transitive = CssomContext {
        inputs: vec![],
        linked: vec![CssomLinkedInput {
            version: CssomInputVersion {
                role: CssomInputRole::Import,
                identity: "intermediate".into(),
                revision: 1,
            },
            snapshot: intermediate.snapshot(),
            sheet: selected,
        }],
    };
    assert!(matches!(
        CssomStore::new(
            CssomLimits {
                max_retained_source_bytes: bytes - 1,
                ..Default::default()
            },
            transitive.clone()
        ),
        Err(CssomError::Limit {
            resource: "retained parse source bytes",
            ..
        })
    ));
    CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: bytes,
            ..Default::default()
        },
        transitive,
    )
    .unwrap();
}

#[test]
fn source_free_owner_has_exact_independent_traversal_work_and_no_byte_charge() {
    CssomStore::new(
        CssomLimits {
            max_retained_source_bytes: 0,
            max_source_traversal_steps: 3,
            max_source_traversal_depth: 0,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    assert!(matches!(
        CssomStore::new(
            CssomLimits {
                max_retained_source_bytes: 0,
                max_source_traversal_steps: 2,
                ..Default::default()
            },
            CssomContext::default()
        ),
        Err(CssomError::Limit {
            resource: "retained source traversal steps",
            maximum: 2
        })
    ));
}

#[test]
fn hidden_lexical_roots_and_terminators_cannot_bypass_atomic_typed_admission() {
    fn tokens(text: &str) -> Vec<CssComponentValue> {
        text.split_whitespace()
            .map(|s| CssComponentValue::try_token(s).unwrap())
            .collect()
    }
    let parsed = parse_component_values("/* terminator-only source */;").unwrap();
    let mut components = tokens("x : y");
    components.push(parsed.items().last().unwrap().clone());
    let body =
        CssSupportsTestBody::try_from_components(CssComponentValues::try_new(components).unwrap())
            .unwrap();
    let named = surgeist_css::CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--test").unwrap(),
        body,
    )
    .unwrap();

    let parsed = parse_component_values("/* hidden supports sibling */(x:y)").unwrap();
    let first = CssComponentValue::try_block(
        CssBlockKind::Parenthesis,
        CssComponentValues::try_new(tokens("x : y")).unwrap(),
    )
    .unwrap();
    let mut components = vec![first];
    components.extend(tokens("and"));
    components.push(parsed.items().last().unwrap().clone());
    let parent = CssSupportsCondition::try_from_components(
        CssComponentValues::try_new(components).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let CssSupportsConditionKind::And(list) = parent.kind() else {
        panic!("and");
    };
    let supports = surgeist_css::CssSupportsRule::try_new(
        list.conditions()[0].clone(),
        vec![],
        &CssNamespaceContext::default(),
    )
    .unwrap();

    let parsed = parse_component_values("/* hidden when sibling */media(height)").unwrap();
    let first = CssComponentValue::try_function(
        "media",
        CssComponentValues::try_new(tokens("width")).unwrap(),
    )
    .unwrap();
    let mut components = vec![first];
    components.extend(tokens("and"));
    components.push(parsed.items().last().unwrap().clone());
    let parent =
        CssWhenCondition::try_from_components(CssComponentValues::try_new(components).unwrap())
            .unwrap();
    let CssWhenConditionKind::And(list) = parent.kind() else {
        panic!("and");
    };
    let when = surgeist_css::CssWhenRule::try_new(
        list.conditions()[0].clone(),
        vec![],
        &CssNamespaceContext::default(),
    )
    .unwrap();

    for rule in [
        AuthoredRule::SupportsCondition(named),
        AuthoredRule::Supports(supports),
        AuthoredRule::When(when),
    ] {
        let sheet = CssSheet::try_from_rules(vec![rule]).unwrap();
        assert!(sheet.parse_source().is_none());
        let mut high = owner(1024 * 1024);
        create(&mut high, sheet.clone(), "positive");
        let mut zero = owner(0);
        let before = zero.snapshot();
        let mut edit = zero.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
        assert!(matches!(
            edit.create_typed_sheet(sheet, inputs("refused")),
            Err(CssomError::Limit {
                resource: "retained parse source bytes",
                maximum: 0
            })
        ));
        assert!(matches!(zero.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(zero.snapshot().revision(), before.revision());
    }
}
