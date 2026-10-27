//! Shared execution and assertions for authored-property tests. Grammar cases,
//! expected serializations, expansion payloads, and resource costs stay with
//! their owning suites and are never inferred from production metadata.

use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssLonghandContribution, CssPropertyNameRef,
    CssPropertyTarget, CssPropertyTextTarget, CssPropertyValueError,
};
use surgeist_css::{CssParseReport, CssParser};

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
        let actual = source.serialize_specified(Default::default()).unwrap_or_else(|error| {
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
    let report = CssParser::default().parse_block_contents(&css);
    assert!(
        report.is_clean(),
        "{property:?}: {value:?} via StyleAttribute: {:?}",
        report.diagnostics()
    );
    assert!(
        CssParser::default()
            .parse_block_contents(&css)
            .into_validation_result()
            .is_ok(),
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
    let components =
        CssComponentValues::parse(&CssParser::default(), value).unwrap_or_else(|error| {
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
) -> Result<CssDeclaration, CssPropertyValueError> {
    if grammar {
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(property.grammar()),
            components,
            importance,
        )
    } else {
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            components,
            importance,
        )
    }
}

fn text_report(
    property: CssKnownProperty,
    value: &str,
    grammar: bool,
    importance: CssImportance,
) -> CssParseReport<Option<CssDeclaration>> {
    if grammar {
        CssParser::default().parse_property_value(
            value,
            CssPropertyTextTarget::Grammar(property.grammar()),
            importance,
        )
    } else {
        CssParser::default().parse_property_value(
            value,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(property)),
            importance,
        )
    }
}

#[track_caller]
pub fn invalid(property: CssKnownProperty, value: &str) {
    let css = format!("color:red;{}:{value};color:blue", property.canonical_name());
    let report = CssParser::default().parse_block_contents(&css);
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
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "{css}: expected property grammar diagnostic"
    );
    assert_eq!(detail.known_property().unwrap(), property);
    assert_eq!(
        CssParser::default()
            .parse_block_contents(&css)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );

    let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
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
                error.code().is_some(),
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
    let CssExpansion::Pending(handle) = source.expand().unwrap() else {
        panic!("whole-value substitution remains pending")
    };
    assert!(handle.source().same_occurrence(&source));
    for &text in invalid {
        assert!(
            handle
                .reenter(CssComponentValues::parse(&CssParser::default(), text).unwrap())
                .unwrap_err()
                .replacement_error()
                .is_some()
        );
    }
    for text in ["var(--again)", "env(value)", "attr(data-value)"] {
        assert!(
            handle
                .reenter(CssComponentValues::parse(&CssParser::default(), text).unwrap())
                .unwrap_err()
                .is_residual_substitution()
        );
    }
    for text in [valid[0], "inherit"] {
        let replacement = CssComponentValues::parse(&CssParser::default(), text).unwrap();
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
    let report = CssParser::default().parse_sheet(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
    let error = report.syntax().normalize(limits).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 2))
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
        report
            .syntax()
            .normalize(Default::default())
            .unwrap()
            .items()
            .len(),
        normalized.items().len()
    );
}
