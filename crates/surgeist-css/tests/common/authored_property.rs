#![forbid(unsafe_code)]
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
