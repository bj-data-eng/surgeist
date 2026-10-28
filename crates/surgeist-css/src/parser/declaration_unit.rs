//! A declaration that the parser crate's driver consumed, admitted by the
//! shell every declaration of a ported grammar shares.
//!
//! The shell is the one of the declarations the native callbacks still admit
//! (`parse_declaration_core`), reading the same parts of the same text: the
//! value is everything between the colon and the first `!` (or the end of the
//! declaration), whitespace and comments included, because the retained
//! components and the origin of a declaration cover it. The importance
//! annotation is the first `!` and what follows it. Both are read from the
//! driver's token tree, so nothing is lexed again but the value, which the
//! property grammars read from a document of its own.
use std::ops::Range;

use cssparser::ParseError;
use surgeist_css_parser::Token;

use super::block_units::{DeclarationUnit, location_at, offset_of};
use super::{
    DeclarationBoundaryContext, DeclarationMode, ParsedDeclaration, declaration_plan,
    invalid_annotation_for_context, parse_property_declaration_body_from_property_input,
};
use crate::declaration::CssImportance;
use crate::diagnostic::CssError;
use crate::diagnostic::error::property::SyntaxErrorExt;
use crate::parser::syntax_types::Site;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::syntax_consumption::DocumentExt;

/// The importance annotation of a declaration, as the shell reads it.
struct Annotation {
    /// Where the `!` is.
    bang: usize,
    /// Whether it is followed by `important` and nothing else.
    complete: bool,
}

/// Where the parts of a declaration are in the source text.
pub(super) struct DeclarationText {
    name: Range<usize>,
    /// Where the declaration ends: at the `;` that ends it, or at the end of its list.
    pub(super) end: usize,
    /// Where the value begins: right after the colon.
    pub(super) value_start: usize,
    /// Where the value ends: at the first `!`, or at the end of the declaration.
    value_end: usize,
    annotation: Option<Annotation>,
}

impl DeclarationText {
    /// Where the declaration's name begins.
    pub(super) fn name_start(&self) -> usize {
        self.name.start
    }

    /// Reads the parts of `declaration` from the token tree it was consumed
    /// from, which is a document of source text.
    pub(super) fn of(declaration: &DeclarationUnit<'_>) -> Self {
        Self::read(declaration.start(), declaration.end().index())
    }

    /// Reads the parts of the declaration that starts at the identifier `start`
    /// and whose value ends before the node `end` of the same list.
    pub(super) fn read(start: Site<'_>, end: usize) -> Self {
        let document = start.document();
        let list = start.list();
        let nodes = document.list(list);
        let name = document
            .range(nodes[start.index()])
            .expect("a source declaration name has its interval");
        // The driver consumed a declaration only after an identifier, optional
        // trivia and a colon.
        let mut colon = start.index() + 1;
        while document.is_trivia(nodes[colon]) {
            colon += 1;
        }
        let first = colon + 1;
        let boundary = |index: usize| {
            offset_of(
                document
                    .site(list, index)
                    .expect("a boundary of the declaration's list"),
            )
        };
        let bang =
            (first..end).find(|&index| matches!(document.token(nodes[index]), Token::Delim('!')));
        let annotation = bang.map(|bang| {
            let mut index = bang + 1;
            while index < end && document.is_trivia(nodes[index]) {
                index += 1;
            }
            let word = index < end
                && matches!(document.token(nodes[index]), Token::Ident(word) if word.eq_ignore_ascii_case("important"));
            index += 1;
            while index < end && document.is_trivia(nodes[index]) {
                index += 1;
            }
            Annotation {
                bang: boundary(bang),
                complete: word && index >= end,
            }
        });
        Self {
            name: name.start.value()..name.end.value(),
            value_start: boundary(first),
            value_end: boundary(bang.unwrap_or(end)),
            end: boundary(end),
            annotation,
        }
    }
}

/// Admits the declaration named `name` that lies at `text` in the way of
/// `mode`: its name selects the grammar of its value, which is read first;
/// then its importance annotation is judged.
pub(super) fn parse_declaration_unit<'i>(
    mode: DeclarationMode,
    name: &str,
    text: &DeclarationText,
    snapshot: &CssSourceSnapshot,
    parser_context: crate::CssParserContext,
) -> Result<ParsedDeclaration, ParseError<'i, CssError>> {
    let name_origin = CssParsedOrigin::from_range(snapshot, text.name.clone())
        .expect("parser name boundaries belong to the original source");
    let plan = declaration_plan(
        mode,
        name,
        location_at(snapshot, text.name.start),
        parser_context,
    )?;
    let numeric = crate::numeric::expression::NumericInputContext::parsed(snapshot);
    let value = text.value_start..text.value_end;
    let document = crate::syntax_consumption::source_window_document(snapshot, value.clone())
        .map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                location_at(snapshot, value.end),
                error,
            )
        })?;
    let mut value_input = document.root();
    let body = parse_property_declaration_body_from_property_input(
        plan.grammar(),
        &mut value_input,
        &numeric,
        plan.value_context,
    )
    .map_err(SyntaxErrorExt::into_native_source)
    .map_err(|error| plan.contextualize(error))?;
    let components =
        crate::component::values::parse::promote_window(&document).map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                location_at(snapshot, value.end),
                error,
            )
        })?;
    let value_origin = CssParsedOrigin::from_range(snapshot, value)
        .expect("parser value boundaries belong to the original source");
    let importance = match &text.annotation {
        None => CssImportance::Normal,
        Some(annotation) => {
            let admitted = matches!(
                plan.context,
                DeclarationBoundaryContext::OrdinaryKnown(_)
                    | DeclarationBoundaryContext::OrdinaryCustom(_)
                    | DeclarationBoundaryContext::OrdinarySvgGlyph
            );
            if !(admitted && annotation.complete) {
                return Err(invalid_annotation_for_context(
                    location_at(snapshot, annotation.bang),
                    &plan.context,
                ));
            }
            CssImportance::Important
        }
    };
    Ok(ParsedDeclaration {
        body,
        importance,
        components,
        name: name_origin,
        value_origin,
        parser_context,
    })
}
