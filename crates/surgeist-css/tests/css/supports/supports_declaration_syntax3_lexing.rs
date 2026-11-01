//! The value of a supports declaration whose last token is a lone backslash.
//!
//! Syntax 3 (2026 CRD) §4.3.7 and §4.3.8: a backslash followed by a newline is
//! not a valid escape, so `-\` and a newline is the delimiters `-` and `\`
//! followed by whitespace (§4.3.9). A declaration value is any sequence of
//! tokens (§5.5.6): Conditional 3 `supports()` takes the declaration as
//! written, and whether the property supports the value is its answer, not a
//! failure to parse.

use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValueRef, CssValueTokenRef};
use surgeist_css::query::supports::CssSupportsDeclaration;

#[test]
fn a_value_ending_in_a_backslash_before_a_newline_is_a_declaration_value() {
    for newline in ["\n", "\r\n", "\r", "\u{c}"] {
        let value = format!("-\\{newline}");
        for property in ["color", "width", "--custom"] {
            let declaration =
                CssSupportsDeclaration::parse(&CssParser::default(), property, &value)
                    .unwrap_or_else(|error| panic!("{property}: {value:?}: {error:?}"));
            assert_eq!(declaration.property(), property, "{value:?}");
            // The declaration is its name, the colon, and the tokens of the
            // value: the delimiter `-`, the delimiter `\` and the whitespace
            // that the newline is.
            let views: Vec<_> = declaration
                .components()
                .iter()
                .map(|component| component.view())
                .collect();
            let [
                CssComponentValueRef::Token(CssValueTokenRef::Ident(name)),
                CssComponentValueRef::Token(CssValueTokenRef::Colon),
                CssComponentValueRef::Token(CssValueTokenRef::Delim('-')),
                CssComponentValueRef::Token(CssValueTokenRef::Delim('\\')),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_)),
            ] = views.as_slice()
            else {
                panic!("{property}: {value:?}: {views:?}");
            };
            assert_eq!(*name, property, "{value:?}");
        }
    }
}
