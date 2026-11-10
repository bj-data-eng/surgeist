//! A declaration that the parser crate's driver consumed, admitted by the
//! shell every declaration grammar shares.
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

use crate::diagnostic::error::GrammarFailure;

use super::block_units::{DeclarationUnit, location_at, offset_of};
use super::{
    DeclarationBoundaryContext, DeclarationMode, ParsedDeclaration, declaration_plan,
    invalid_annotation_for_context, parse_property_declaration_body_from_property_input,
};
use crate::declaration::CssImportance;
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
        // The name is the declaration's first node; a declaration of a source
        // document always has one, and the empty interval at its start stands
        // in for a boundary that does not.
        let name = nodes
            .get(start.index())
            .and_then(|&node| document.range(node))
            .map_or_else(
                || {
                    let at = offset_of(start);
                    at..at
                },
                |interval| interval.start.value()..interval.end.value(),
            );
        let is_trivia = |index: usize| {
            nodes
                .get(index)
                .is_some_and(|&node| document.is_trivia(node))
        };
        // The driver consumed a declaration only after an identifier, optional
        // trivia and a colon.
        let mut colon = start.index() + 1;
        while is_trivia(colon) {
            colon += 1;
        }
        let first = colon + 1;
        let name_end = name.end;
        let boundary = |index: usize| document.site(list, index).map_or(name_end, offset_of);
        // Step 6: the last two tokens that are not whitespace are `!` and
        // `important`.
        let bang = declaration
            .is_important()
            .then(|| {
                let significant = |upto: usize| {
                    (first..upto)
                        .rev()
                        .find(|&index| nodes.get(index).is_some() && !is_trivia(index))
                };
                let important = significant(end)?;
                significant(important)
            })
            .flatten();
        Self {
            name,
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
pub(super) fn parse_declaration_unit(
    mode: DeclarationMode,
    name: &str,
    text: &DeclarationText,
    snapshot: &CssSourceSnapshot,
    parser_context: crate::CssParserContext,
) -> Result<ParsedDeclaration, GrammarFailure> {
    let name_origin = CssParsedOrigin::covering(snapshot, text.name.clone());
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
    .map_err(SyntaxErrorExt::into_grammar_failure_source)
    .map_err(|error| plan.contextualize(error))?;
    let components =
        crate::component::values::parse::promote_window(&document).map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                location_at(snapshot, value.end),
                error,
            )
        })?;
    let value_origin = CssParsedOrigin::covering(snapshot, value);
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
