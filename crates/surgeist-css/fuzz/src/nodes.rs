//! Retained-node invariants: nodes are valid by construction, appear in source
//! order, and carry provenance inside the parsed source under its coordinate
//! contract. Nothing here re-derives grammar validity.
use crate::check::{Check, Checker, Violation};
use surgeist_css::{
    declaration::CssDeclaration,
    property::CssPropertyNameRef,
    rule::{CssRule, CssSheet},
    source::CssSourcePosition,
};

pub(crate) const RETAINED: &str = "surgeist-css::retained-nodes";

/// Declarations of one list: consistent identity, strictly increasing name
/// starts, values after their names, and provenance over the parsed source.
pub(crate) fn declarations(
    checker: &Checker,
    text: &str,
    list: &[CssDeclaration],
    what: &str,
) -> Check {
    let mut previous_start = None;
    for (index, declaration) in list.iter().enumerate() {
        match declaration.property_name() {
            CssPropertyNameRef::Known(property) => {
                if declaration.known().map(|known| known.property()) != Some(property) {
                    return Err(Violation::new(
                        RETAINED,
                        "known-name-without-known-body",
                        format!(
                            "{what} declaration {index} names a known property without its body"
                        ),
                    ));
                }
            }
            CssPropertyNameRef::Custom(name) => {
                let named = declaration
                    .custom()
                    .is_some_and(|custom| custom.name().as_str() == name.as_str());
                if !named || !name.as_str().starts_with("--") {
                    return Err(Violation::new(
                        RETAINED,
                        "custom-name-without-custom-body",
                        format!("{what} declaration {index} has an inconsistent custom name"),
                    ));
                }
            }
            _ => {}
        }
        let name = declaration.parsed_name();
        let value = declaration.parsed_value();
        for origin in name.into_iter().chain(value) {
            if origin.source().as_str() != text {
                return Err(Violation::new(
                    RETAINED,
                    "origin-source-differs",
                    format!("{what} declaration {index} origin addresses another source"),
                ));
            }
        }
        if let Some(name) = name {
            checker.span(RETAINED, text, name.span(), "declaration name")?;
            if let Some(start) = previous_start
                && name.span().start().byte_offset() <= start
            {
                return Err(Violation::new(
                    RETAINED,
                    "declarations-out-of-source-order",
                    format!("{what} declaration {index} does not follow its predecessor"),
                ));
            }
            previous_start = Some(name.span().start().byte_offset());
            if declaration.position() != Some(name.span().start()) {
                return Err(Violation::new(
                    RETAINED,
                    "declaration-position-not-name-start",
                    format!("{what} declaration {index} position is not its name start"),
                ));
            }
        }
        if let Some(value) = value {
            checker.span(RETAINED, text, value.span(), "declaration value")?;
            if let Some(name) = name
                && value.span().start().byte_offset() < name.span().end().byte_offset()
            {
                return Err(Violation::new(
                    RETAINED,
                    "value-precedes-name",
                    format!("{what} declaration {index} value starts before its name ends"),
                ));
            }
        }
    }
    Ok(())
}

fn rule_position(rule: &CssRule) -> Option<CssSourcePosition> {
    match rule {
        CssRule::Style(rule) => Some(rule.position()),
        CssRule::Media(rule) => rule.position(),
        CssRule::Supports(rule) => rule.position(),
        CssRule::Container(rule) => rule.position(),
        CssRule::LayerBlock(rule) => rule.position(),
        CssRule::NestedDeclarations(rule) => Some(rule.position()),
        _ => None,
    }
}

fn rules(checker: &Checker, text: &str, list: &[CssRule], what: &str) -> Check {
    let mut previous = None;
    for (index, rule) in list.iter().enumerate() {
        if let Some(position) = rule_position(rule) {
            checker.position(RETAINED, text, position, "rule position")?;
            if previous.is_some_and(|before| position.byte_offset() <= before) {
                return Err(Violation::new(
                    RETAINED,
                    "rules-out-of-source-order",
                    format!("{what} rule {index} does not follow its predecessor"),
                ));
            }
            previous = Some(position.byte_offset());
        }
        match rule {
            CssRule::Style(rule) => {
                declarations(checker, text, rule.declarations(), "style rule")?;
                rules(checker, text, rule.rules(), "style rule child")?;
            }
            CssRule::NestedDeclarations(rule) => {
                declarations(checker, text, rule.declarations(), "nested declarations")?;
            }
            CssRule::Media(rule) => rules(checker, text, rule.rules(), "media rule child")?,
            CssRule::Supports(rule) => rules(checker, text, rule.rules(), "supports rule child")?,
            CssRule::Container(rule) => rules(checker, text, rule.rules(), "container rule child")?,
            CssRule::LayerBlock(rule) => rules(checker, text, rule.rules(), "layer rule child")?,
            _ => {}
        }
    }
    Ok(())
}

/// Every retained rule and declaration of a parsed sheet.
pub(crate) fn sheet(checker: &Checker, text: &str, sheet: &CssSheet) -> Check {
    if let Some(source) = sheet.parse_source()
        && source.source().as_str() != text
    {
        return Err(Violation::new(
            RETAINED,
            "sheet-source-differs",
            "the sheet retains a snapshot other than the parsed source",
        ));
    }
    rules(checker, text, sheet.rules(), "sheet")
}
