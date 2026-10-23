//! Helpers shared by every module of the CSS integration binary.
//!
//! Each helper is written once here. Variants are named for the stimulus
//! and checks they apply, and modules import them under their local names
//! so a test still states which front, importance and failure report it
//! depends on instead of copying a constructor.

pub mod authored_property;
pub mod authored_syntax;
pub mod color_and_selector_fixtures;
pub mod color_components;
pub mod color_property;
pub mod csstree;
pub mod descriptor_phases;
pub mod digest;
pub mod external_consumer;
pub mod failure_classes;
#[macro_use]
pub mod font_face;
pub mod isolated_stack;
pub mod lab_direct_component_cases;
pub mod math_depth;
pub mod named_property;
pub mod parse_report;
pub mod property_admission;
pub mod property_expectations;
pub mod rules;
#[macro_use]
pub mod sheet_cases;
pub mod serialization_limits;
pub mod value_assertions;

use surgeist_css::*;

/// Whether `error` is the serialization failure of a recovered media member:
/// an unserializable boundary without a lexical component cause.
pub fn is_recovered_media_failure(error: &CssSerializationError) -> bool {
    error.is_unserializable_boundary() && std::error::Error::source(error).is_none()
}

/// Component limits that bound only the serialized byte length, for
/// `serialize_tokens`; the structural limits keep their defaults.
pub fn byte_limits(max_css_bytes: usize) -> CssComponentValueLimits {
    let defaults = CssComponentValueLimits::default();
    CssComponentValueLimits::try_new(
        defaults.max_nesting_depth(),
        defaults.max_components(),
        max_css_bytes,
    )
    .expect("the default nesting depth is valid")
}

/// The sole declaration of the clean style attribute `css`.
#[track_caller]
pub fn sole_declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {css}: {:?}", report.syntax())
    };
    declaration.clone()
}

/// The sole declaration of the clean style attribute `name:value`, written
/// with `importance`.
#[track_caller]
pub fn declaration(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    let priority = match importance {
        CssImportance::Important => "!important",
        CssImportance::Normal => "",
    };
    sole_declaration(&format!("{name}:{value}{priority}"))
}

/// [`declaration`] with `!important`.
#[track_caller]
pub fn important_declaration(name: &str, value: &str) -> CssDeclaration {
    declaration(name, value, CssImportance::Important)
}

/// [`declaration`] with normal importance.
#[track_caller]
pub fn normal_declaration(name: &str, value: &str) -> CssDeclaration {
    declaration(name, value, CssImportance::Normal)
}

/// [`important_declaration`] that also validates, and checks the canonical
/// name, importance and value serialization.
#[track_caller]
pub fn validated_important_declaration(name: &str, value: &str) -> CssDeclaration {
    let source = important_declaration(name, value);
    assert!(
        validate_style_attribute(&format!("{name}:{value}!important")).is_ok(),
        "{name}:{value}!important"
    );
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(
        source
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        value
    );
    source
}

/// [`important_declaration`] of the Speech 1 property `name`.
#[track_caller]
pub fn important_speech_declaration(name: &str, text: &str) -> CssDeclaration {
    let value = important_declaration(name, text);
    assert_eq!(value.known().unwrap().property(), known_property(name));
    assert_eq!(value.importance(), CssImportance::Important);
    value
}

/// [`normal_declaration`] of `color`.
#[track_caller]
pub fn color_declaration(text: &str) -> CssDeclaration {
    normal_declaration("color", text)
}

/// The typed color of a clean, validated `color:text` style attribute whose
/// value keeps its parsed origin.
pub fn parsed_color(text: &str) -> CssColor {
    let source = format!("color:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(validate_style_attribute(&source).unwrap(), *report.syntax());
    let declaration = &report.syntax()[0];
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("parsed function origin: {source}");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color: {source}");
    };
    value.value().clone()
}

/// Component values of input that must tokenize.
pub fn component_values(source: &str) -> CssComponentValues {
    parse_component_values(source).unwrap()
}

/// An important `color` declaration checked from component values.
pub fn checked_color_declaration(source: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        component_values(source),
        CssImportance::Important,
    )
    .unwrap_or_else(|e| panic!("{source}: {e:?}"))
}

/// The typed color carried by a checked `color` declaration.
pub fn declaration_color(declaration: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color: {declaration:?}")
    };
    value.value()
}

/// The typed color of `source` checked from component values by the
/// `color` grammar.
pub fn checked_color(source: &str) -> CssColor {
    declaration_color(&checked_color_declaration(source)).clone()
}

/// The grammar of a selected property name.
pub fn selected_grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"))
}

/// An important declaration checked against a named property grammar.
pub fn checked_grammar_declaration(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        selected_grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

/// An important declaration checked against a property's grammar.
pub fn checked_property_grammar_declaration(
    property: CssKnownProperty,
    css: &str,
) -> CssDeclaration {
    parse_property_value_for_grammar(
        property.grammar(),
        parse_component_values(css).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}

/// Checks important component values by property name or, when `grammar`
/// is set, by the property's grammar.
pub fn check_property_components(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    }
}

/// A property that public discovery finds by canonical and uppercase name.
pub fn discoverable_property(name: &str) -> CssKnownProperty {
    let property = CssKnownProperty::from_name(name).unwrap_or_else(|| {
        panic!("selected authored property {name} must accept public discovery")
    });
    assert_eq!(property.canonical_name(), name);
    assert_eq!(
        CssKnownProperty::from_name(&name.to_ascii_uppercase()),
        Some(property)
    );
    assert_eq!(
        CssPropertyGrammar::from_name(name).unwrap(),
        property.grammar()
    );
    property
}

/// An important declaration admitted through one of five public fronts:
/// style attribute (0), checked by name (1) or grammar (2), or text by name
/// (3) or grammar (4).
pub fn authored_declaration_through_front(name: &str, input: &str, front: usize) -> CssDeclaration {
    let p = discoverable_property(name);
    let source = match front {
        0 => {
            let css = format!("/*😀*/{name}:{input}!important");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
            let [source] = report.syntax().as_slice() else {
                panic!("one authored occurrence")
            };
            assert!(source.position().is_some());
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            source.clone()
        }
        1 | 2 => {
            let components = parse_component_values(input).unwrap();
            let before = components.clone();
            let source = check_property_components(p, components.clone(), front == 2).unwrap();
            assert_eq!(components, before);
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_value().is_none());
            source
        }
        3 | 4 => {
            let report = if front == 3 {
                parse_property_value_text(
                    input,
                    CssPropertyNameRef::Known(p),
                    CssImportance::Important,
                )
            } else {
                parse_property_value_text_for_grammar(input, p.grammar(), CssImportance::Important)
            };
            assert!(
                report.is_clean(),
                "{name}:{input}: {:?}",
                report.diagnostics()
            );
            let source = report
                .syntax()
                .as_ref()
                .expect("retained direct value")
                .clone();
            assert!(source.position().is_none());
            assert_eq!(source.parsed_value().unwrap().source().as_str(), input);
            source
        }
        _ => unreachable!(),
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

/// The shared authored-property checked declaration for a property name.
pub fn authored_checked_declaration(name: &str, value: &str, grammar: bool) -> CssDeclaration {
    authored_property::checked(known_property(name), value, grammar)
}

/// The shared authored-property parsed declaration for a property name.
pub fn authored_parsed_declaration(name: &str, value: &str) -> CssDeclaration {
    authored_property::parsed(known_property(name), value)
}

/// A normal-importance declaration checked by known property name.
pub fn normal_checked_declaration(property: CssKnownProperty, text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap()
}

/// An important declaration checked from component values by known property.
pub fn important_checked_components(
    property: CssKnownProperty,
    components: CssComponentValues,
) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        components,
        CssImportance::Important,
    )
    .unwrap()
}

/// An important declaration checked by known property that retains its
/// component values.
pub fn retained_checked_declaration(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let source = parse_property_value(
        CssPropertyNameRef::Known(property),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(source.value_components(), &components);
    source
}

/// Checks important tokenized text by known property name.
pub fn check_property_text(
    property: CssKnownProperty,
    text: &str,
) -> Result<CssDeclaration, CssPropertyValueError> {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(text).unwrap(),
        CssImportance::Important,
    )
}

/// A known property found by name.
#[track_caller]
pub fn known_property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap_or_else(|| panic!("known property: {name}"))
}

/// An important declaration checked in a parser context by a property's
/// grammar.
pub fn contextual_checked_declaration(
    context: CssParserContext,
    name: &str,
    text: &str,
) -> CssDeclaration {
    context
        .parse_property_value_for_grammar(
            known_property(name).grammar(),
            parse_component_values(text).unwrap(),
            CssImportance::Important,
        )
        .unwrap()
}

/// Checks important component values by property name or, when `grammar`
/// is set, by the named property's grammar.
pub fn check_named_components(
    name: &str,
    values: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueError> {
    let p = known_property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), values, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            values,
            CssImportance::Important,
        )
    }
}

/// An important checked declaration that keeps its component values and
/// carries no source position, parsed name or parsed value.
pub fn detached_checked_declaration(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    let source = check_named_components(name, values.clone(), grammar).unwrap();
    assert_eq!(source.value_components(), &values);
    assert_eq!(values, before);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

/// An important Speech 1 declaration checked from component values.
pub fn important_speech_components(name: &str, components: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(known_property(name)),
        components,
        CssImportance::Important,
    )
    .unwrap()
}

/// A clean selector validated with the default namespace context.
pub fn validated_selector(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

/// A clean selector validated in a namespace context.
pub fn validated_selector_in(source: &str, context: &CssNamespaceContext) -> CssSelector {
    let report = parse_selector(source, context);
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

/// The selector admitted by a clean default-namespace parse.
pub fn admitted_selector(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().as_ref().expect("admitted selector").clone()
}
