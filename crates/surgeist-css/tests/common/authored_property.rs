#![forbid(unsafe_code)]
#![expect(
    dead_code,
    reason = "each integration target uses a different assertion subset"
)]
//! Shared execution and assertions for authored-property tests. Grammar cases,
//! expected serializations, expansion payloads, and resource costs stay with
//! their owning suites and are never inferred from production metadata.

use surgeist_css::*;

#[derive(Clone, Copy, Debug)]
pub enum ParserFront {
    StyleAttribute,
    CheckedName,
    CheckedGrammar,
    TextName,
    TextGrammar,
}

impl ParserFront {
    #[track_caller]
    pub fn parse(self, property: CssKnownProperty, value: &str) -> CssDeclaration {
        match self {
            Self::StyleAttribute => parsed(property, value),
            Self::CheckedName => checked(property, value, false),
            Self::CheckedGrammar => checked(property, value, true),
            Self::TextName | Self::TextGrammar => {
                let report = text_report(
                    property,
                    value,
                    matches!(self, Self::TextGrammar),
                    CssImportance::Important,
                );
                assert!(
                    report.is_clean(),
                    "{property:?}: {value:?} via {self:?}: {:?}",
                    report.diagnostics()
                );
                let source = report.syntax().as_ref().expect("retained value").clone();
                assert_eq!(source.known().unwrap().property(), property);
                assert_eq!(source.known().unwrap().grammar(), property.grammar());
                assert_eq!(source.parsed_value().unwrap().source().as_str(), value);
                assert!(source.position().is_none());
                assert!(source.parsed_name().is_none());
                assert_eq!(source.importance(), CssImportance::Important);
                source
            }
        }
    }

    #[track_caller]
    pub fn valid(self, property: CssKnownProperty, input: &str, expected: &str) -> CssDeclaration {
        let source = self.parse(property, input);
        let before = source.clone();
        let actual = source.to_specified_css().unwrap_or_else(|error| {
            panic!(
                "{property:?}: {input:?} via {self:?}: specified serialization failed: {error:?}"
            )
        });
        assert_eq!(
            actual,
            format!("{}: {expected} !important;", property.canonical_name()),
            "{property:?}: {input:?} via {self:?}: specified serialization"
        );
        assert_eq!(
            source, before,
            "{property:?}: {input:?} via {self:?}: serialization mutated the declaration"
        );
        source
    }
}

#[track_caller]
pub fn parsed(property: CssKnownProperty, value: &str) -> CssDeclaration {
    // Exercise the canonical name and its uppercase spelling. The owning
    // suites previously used different spellings; neither stimulus is lost.
    parsed_name(property, property.canonical_name(), value);
    parsed_name(
        property,
        &property.canonical_name().to_ascii_uppercase(),
        value,
    )
}

#[track_caller]
fn parsed_name(property: CssKnownProperty, name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{name}:{value}!important");
    let report = parse_style_attribute(&css);
    assert!(
        report.is_clean(),
        "{property:?}: {value:?} via StyleAttribute: {:?}",
        report.diagnostics()
    );
    assert!(
        validate_style_attribute(&css).is_ok(),
        "{css}: clean-report validation"
    );
    let [source] = report.syntax().as_slice() else {
        panic!("{css}: expected one occurrence")
    };
    assert_eq!(source.known().unwrap().property(), property);
    assert_eq!(source.known().unwrap().grammar(), property.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    source.clone()
}

#[track_caller]
pub fn checked(property: CssKnownProperty, value: &str, grammar: bool) -> CssDeclaration {
    let front = if grammar {
        ParserFront::CheckedGrammar
    } else {
        ParserFront::CheckedName
    };
    let components = parse_component_values(value).unwrap_or_else(|error| {
        panic!("{property:?}: {value:?} via {front:?}: component parsing failed: {error:?}")
    });
    let before = components.clone();
    let source = checked_components(
        property,
        components.clone(),
        grammar,
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{property:?}: {value:?} via {front:?}: {error:?}"));
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert_eq!(source.known().unwrap().property(), property);
    assert_eq!(source.known().unwrap().grammar(), property.grammar());
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

pub fn checked_components(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), components, importance)
    } else {
        parse_property_value(CssPropertyNameRef::Known(property), components, importance)
    }
}

fn text_report(
    property: CssKnownProperty,
    value: &str,
    grammar: bool,
    importance: CssImportance,
) -> CssParseReport<Option<CssDeclaration>> {
    if grammar {
        parse_property_value_text_for_grammar(value, property.grammar(), importance)
    } else {
        parse_property_value_text(value, CssPropertyNameRef::Known(property), importance)
    }
}

#[track_caller]
pub fn invalid(property: CssKnownProperty, value: &str) {
    let css = format!("color:red;{}:{value};color:blue", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}: retained siblings");
    assert!(
        report
            .syntax()
            .iter()
            .all(|d| d.known().unwrap().property() == CssKnownProperty::Color),
        "{css}: sibling identities"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("{css}: expected one atomic grammar failure")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("{css}: expected property grammar diagnostic")
    };
    assert_eq!(detail.property(), property);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );

    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    // The original suites exercise different importance paths. Keep both,
    // rather than letting extraction silently replace Normal with Important.
    for importance in [CssImportance::Normal, CssImportance::Important] {
        for (grammar, front) in [
            (false, ParserFront::CheckedName),
            (true, ParserFront::CheckedGrammar),
        ] {
            let error =
                checked_components(property, components.clone(), grammar, importance).unwrap_err();
            assert!(
                matches!(error.kind(), CssPropertyValueErrorKind::Grammar(_)),
                "{property:?}: {value:?} via {front:?}, {importance:?}: expected grammar error, got {error:?}"
            );
        }
        for (grammar, front) in [
            (false, ParserFront::TextName),
            (true, ParserFront::TextGrammar),
        ] {
            let report = text_report(property, value, grammar, importance);
            assert!(
                report.syntax().is_none(),
                "{property:?}: {value:?} via {front:?}, {importance:?}: invalid value retained"
            );
            assert!(
                !report.is_clean(),
                "{property:?}: {value:?} via {front:?}, {importance:?}: missing diagnostic"
            );
        }
    }
    assert_eq!(
        components, before,
        "{property:?}: {value:?}: rejected input changed"
    );
}

#[track_caller]
pub fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
}

/// Execute every authored admission front; family values remain with callers.
pub fn fronts(property: CssKnownProperty, text: &str) -> [CssDeclaration; 5] {
    [
        ParserFront::StyleAttribute,
        ParserFront::CheckedName,
        ParserFront::CheckedGrammar,
        ParserFront::TextName,
        ParserFront::TextGrammar,
    ]
    .map(|front| front.parse(property, text))
}

#[track_caller]
pub fn pending_longhand_reentry(
    name: &str,
    valid: &[&str],
    invalid: &[&str],
    terminal: &str,
    declaration: fn(&str, &str) -> CssDeclaration,
) {
    let source = declaration(name, "var(--value)");
    assert!(source.known().unwrap().substitution_dependent().is_some());
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("whole-value substitution remains pending")
    };
    assert!(handle.source().same_occurrence(&source));
    for &text in invalid {
        assert!(matches!(
            handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    for text in ["var(--again)", "env(value)", "attr(data-value)"] {
        assert_eq!(
            handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    for text in [valid[0], "inherit"] {
        let replacement = parse_component_values(text).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered terminal")
            };
            let [item] = values.items() else {
                panic!("one replacement contribution")
            };
            assert_eq!(item.property().canonical_name(), terminal);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[track_caller]
pub fn normalized_longhand_order_and_limit(name: &str, valid: &[&str]) {
    let text = format!(
        ".a{{{name}:{}!important;color:red;{name}:{}}}",
        valid[0], valid[1]
    );
    let report = parse_sheet(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (order, expected) in [name, "color", name].into_iter().enumerate() {
        let item = declarations[order];
        assert_eq!(item.order(), order);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            expected
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("one normalized terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert!(values.items()[0].source().same_occurrence(item.source()));
    }
    assert_eq!(
        declarations[0].source().importance(),
        CssImportance::Important
    );
    assert_eq!(declarations[2].source().importance(), CssImportance::Normal);
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2,
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert_eq!(
        error
            .declaration()
            .unwrap()
            .known()
            .unwrap()
            .property()
            .canonical_name(),
        name
    );
    assert_eq!(
        normalize_sheet(report.syntax()).unwrap().items().len(),
        normalized.items().len()
    );
}
