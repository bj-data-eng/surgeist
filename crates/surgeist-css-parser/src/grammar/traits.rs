//! The traits a grammar implements to be driven by the rule algorithms.

use super::{AtRule, Declaration, QualifiedRule};
use crate::SyntaxError;

/// The grammar of a list of rules: what the specification calls "valid in the
/// current context" for an at-rule and a qualified rule (sections 5.5.2 and
/// 5.5.3).
///
/// The driver hands over a whole rule, prelude and block already delimited, and
/// the grammar answers with the item it makes of the rule or with the error
/// that says the rule is not valid. Interpreting the prelude and the block is
/// the grammar's, with the [`Stream`](crate::Stream) methods and, for a block
/// of rules or declarations, [`stylesheet_contents`] and [`block_contents`]
/// again.
///
/// The shape of these traits is provisional: they are proven on the grammars of
/// `surgeist-css` before they are relied on.
///
/// [`stylesheet_contents`]: crate::Stream::stylesheet_contents
/// [`block_contents`]: crate::Stream::block_contents
pub trait RuleGrammar<'i, P = ()> {
    /// What the grammar makes of a valid rule or declaration.
    type Item;
    /// The grammar's own error, carried by
    /// [`SyntaxError::custom`](crate::SyntaxError::custom).
    type Error;

    /// Decides whether `rule` is a valid at-rule in the grammar's context and
    /// returns its item.
    ///
    /// # Errors
    ///
    /// An error says the rule is not valid ("return nothing", section 5.5.2).
    fn at_rule(
        &mut self,
        rule: AtRule<'i, P>,
    ) -> Result<Self::Item, SyntaxError<'i, Self::Error, P>>;

    /// Decides whether `rule` is a valid qualified rule in the grammar's
    /// context and returns its item.
    ///
    /// # Errors
    ///
    /// An error says the rule is not valid (an invalid rule error, section
    /// 5.5.3).
    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, P>,
    ) -> Result<Self::Item, SyntaxError<'i, Self::Error, P>>;
}

/// The grammar of a block: a [`RuleGrammar`] that also takes declarations
/// (section 5.5.5).
///
/// A stylesheet's root grammar cannot take a declaration (section 5.5.1), so
/// the method is not part of [`RuleGrammar`].
pub trait BlockGrammar<'i, P = ()>: RuleGrammar<'i, P> {
    /// Decides whether `declaration` is valid in the grammar's context and
    /// returns its item.
    ///
    /// # Errors
    ///
    /// An error says the declaration is not valid ("return nothing", section
    /// 5.5.6 step 9). In a block the driver then reads the same text again as a
    /// qualified rule (section 5.5.5).
    fn declaration(
        &mut self,
        declaration: Declaration<'i, P>,
    ) -> Result<Self::Item, SyntaxError<'i, Self::Error, P>>;
}
