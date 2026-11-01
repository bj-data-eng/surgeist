//! A declaration that the parser crate's driver consumed, admitted by the
//! shell every declaration of a ported grammar shares.
//!
//! The driver has applied Syntax 3 section 5.5.6 steps 6 to 8 by the time a
//! grammar sees the declaration: the `!important` that ends it is recognised
//! (the last two tokens that are not whitespace) and a non-custom value with a
//! `{}`-block beside other values is refused. The text records where the parts
//! of the declaration are in the source: the value runs from the colon to that
//! `!` (or to the end of the declaration), whitespace and comments included,
//! because the retained components and the origin of a declaration cover it.
//! Nothing is lexed again but the value, which the property grammars read from
//! a document of its own.
use std::ops::Range;

use cssparser::ParseError;

use super::block_units::{DeclarationUnit, location_at, offset_of};
use super::{
    DeclarationBoundaryContext, DeclarationMode, ParsedDeclaration, declaration_plan,
    invalid_annotation_for_context, parse_property_declaration_body_from_property_input,
};
use crate::declaration::CssImportance;
use crate::diagnostic::CssError;
use crate::diagnostic::error::property::SyntaxErrorExt;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::syntax_consumption::DocumentExt;

/// Where the parts of a declaration are in the source text.
pub(crate) struct DeclarationText {
    name: Range<usize>,
    /// Where the declaration ends: at the `;` that ends it, or at the end of its list.
    pub(super) end: usize,
    /// Where the value begins: right after the colon.
    pub(super) value_start: usize,
    /// Where the value ends: at the `!` of the `!important` that ends the
    /// declaration, or at the end of the declaration.
    value_end: usize,
    /// Where the `!` of the `!important` that ends the declaration is.
    bang: Option<usize>,
}

impl DeclarationText {
    /// Where the declaration's name begins.
    pub(super) fn name_start(&self) -> usize {
        self.name.start
    }

    /// Where the value ends: at the `!` of the `!important` that ends the
    /// declaration, or at the end of the declaration.
    pub(super) const fn value_end(&self) -> usize {
        self.value_end
    }

    /// Where the `!` of the `!important` that ends the declaration is, if there
    /// is one (step 6).
    pub(super) const fn bang(&self) -> Option<usize> {
        self.bang
    }

    /// The bytes of the declaration's name.
    pub(super) fn name_range(&self) -> Range<usize> {
        self.name.clone()
    }

    /// Reads the parts of `declaration` from the token tree it was consumed
    /// from, which is a document of source text.
    pub(super) fn of(declaration: &DeclarationUnit<'_>) -> Self {
        let start = declaration.start();
        let end = declaration.end().index();
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
        // Step 6: the last two tokens that are not whitespace are `!` and
        // `important`.
        let bang = declaration
            .is_important()
            .then(|| {
                let significant = |upto: usize| {
                    (first..upto)
                        .rev()
                        .find(|&index| !document.is_trivia(nodes[index]))
                };
                let important = significant(end)?;
                significant(important)
            })
            .flatten();
        Self {
            name: name.start.value()..name.end.value(),
            value_start: boundary(first),
            value_end: boundary(bang.unwrap_or(end)),
            end: boundary(end),
            bang: bang.map(boundary),
        }
    }
}

/// Admits the declaration named `name` that lies at `text` in the way of
/// `mode`: its name selects the grammar of its value, which is read first;
/// then its importance is judged.
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
    let importance = match text.bang() {
        None => CssImportance::Normal,
        Some(bang) => {
            let admitted = matches!(
                plan.context,
                DeclarationBoundaryContext::OrdinaryKnown(_)
                    | DeclarationBoundaryContext::OrdinaryCustom(_)
                    | DeclarationBoundaryContext::OrdinarySvgGlyph
            );
            if !admitted {
                return Err(invalid_annotation_for_context(
                    location_at(snapshot, bang),
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
