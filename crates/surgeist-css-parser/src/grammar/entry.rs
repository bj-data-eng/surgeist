//! The parser entry points of section 5.4 and the drivers of section 5.5, as
//! methods of [`Stream`].

use super::algorithm::{
    Class, Qualified, classify, consume_at_rule, consume_declaration, consume_qualified_rule,
    opening_token,
};
use super::iterator::{qualified_unit, refusal};
use super::{BlockContents, BlockGrammar, Failed, RuleGrammar, StylesheetContents, UnitKind};
use crate::document::Component;
use crate::stream::StreamError;
use crate::{Stream, SyntaxError, Token};

/// The result of each group of a comma-separated list parsed by a grammar.
type GroupResults<'i, T, E, P> = Vec<Result<T, SyntaxError<'i, E, P>>>;

/// Discards whitespace and comments (section 5.3, "discard whitespace"), and
/// returns the first other component, which is the next one.
///
/// Unlike [`Stream::discard_whitespace`], a refusal of the meter is an error and
/// is never mistaken for the end of the stream.
fn skip_trivia<'i, P>(
    stream: &mut Stream<'i, P>,
) -> Result<Option<Component<'i, P>>, StreamError<'i, P>> {
    let mut index = stream.next_index();
    loop {
        match stream.peek_at(index)? {
            Some(Component::Token {
                token: Token::Whitespace | Token::Comment,
                ..
            }) => index += 1,
            other => {
                stream.seek(index);
                return Ok(other);
            }
        }
    }
}

impl<'i, P> Stream<'i, P> {
    // ----- 5.5.1 and 5.5.5: the drivers

    /// Consumes a stylesheet's contents (sections 5.4.4 and 5.5.1): the rules of
    /// the stream, handed to `grammar` one at a time.
    ///
    /// The result is an iterator of units; see [`StylesheetContents`].
    ///
    /// ```
    /// use surgeist_css_parser::grammar::{AtRule, QualifiedRule, RuleGrammar};
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, SyntaxError};
    ///
    /// struct Count;
    /// impl<'i> RuleGrammar<'i> for Count {
    ///     type Item = usize;
    ///     type Error = ();
    ///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<usize, SyntaxError<'i, ()>> {
    ///         Ok(rule.prelude().only_whitespace_remains() as usize)
    ///     }
    ///     fn qualified_rule(&mut self, _: QualifiedRule<'i>) -> Result<usize, SyntaxError<'i, ()>> {
    ///         Ok(2)
    ///     }
    /// }
    /// let document: Document =
    ///     Document::parse("@a; b {}", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let mut grammar = Count;
    /// let items: Vec<_> = document
    ///     .root()
    ///     .stylesheet_contents(&mut grammar)
    ///     .map(Result::unwrap)
    ///     .collect();
    /// assert_eq!(items, [1, 2]);
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub fn stylesheet_contents<'g, G: RuleGrammar<'i, P>>(
        self,
        grammar: &'g mut G,
    ) -> StylesheetContents<'i, 'g, G, P> {
        StylesheetContents {
            stream: self,
            grammar,
            done: false,
            attempt: UnitKind::QualifiedRule,
        }
    }

    /// Consumes a block's contents (sections 5.4.5 and 5.5.5): the
    /// declarations and rules of the stream, handed to `grammar` one at a time.
    ///
    /// The result is an iterator of units; see [`BlockContents`].
    #[must_use]
    pub fn block_contents<'g, G: BlockGrammar<'i, P>>(
        self,
        grammar: &'g mut G,
    ) -> BlockContents<'i, 'g, G, P> {
        BlockContents {
            stream: self,
            grammar,
            done: false,
            attempt: UnitKind::Declaration,
        }
    }

    // ----- 5.4.6 and 5.4.7: one rule, one declaration

    /// Parses a rule (section 5.4.6): whitespace, then one at-rule or qualified
    /// rule, then only whitespace.
    ///
    /// # Errors
    ///
    /// Section 5.4.6 returns a syntax error for an empty stream, for a rule
    /// that is nothing or an invalid rule error, and for anything after the rule
    /// but whitespace; each is a [`Failed`] whose error says which, and which a
    /// refusal of the meter is too.
    ///
    /// ```
    /// use surgeist_css_parser::grammar::{AtRule, QualifiedRule, RuleGrammar};
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, SyntaxError};
    ///
    /// struct Names;
    /// impl<'i> RuleGrammar<'i> for Names {
    ///     type Item = String;
    ///     type Error = ();
    ///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
    ///         Ok(rule.name().to_owned())
    ///     }
    ///     fn qualified_rule(&mut self, _: QualifiedRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
    ///         Ok(String::new())
    ///     }
    /// }
    /// let parse = |text: &str| -> bool {
    ///     let document: Document =
    ///         Document::parse(text, &Options::default(), Limits::default(), DepthExcess::Fail).unwrap();
    ///     document.root().parse_rule(&mut Names).is_ok()
    /// };
    /// assert!(parse("  @media a {}  "));
    /// assert!(!parse("@media a {} b {}")); // two rules
    /// assert!(!parse("   ")); // none
    /// ```
    pub fn parse_rule<G: RuleGrammar<'i, P>>(
        mut self,
        grammar: &mut G,
    ) -> Result<G::Item, Failed<'i, G::Error, P>> {
        let stream = &mut self;
        let first = skip_trivia(stream).map_err(|error| refusal(error, UnitKind::QualifiedRule))?;
        let at = stream.site();
        let Some(first) = first else {
            return Err(Failed::new(
                UnitKind::QualifiedRule,
                stream.end_of_stream(),
                at,
                at,
                None,
            ));
        };
        let (item, kind, end, rule_end) = if let Class::AtKeyword(name) = classify(&first) {
            let outcome = consume_at_rule(stream, name, false, grammar)
                .map_err(|error| refusal(error, UnitKind::AtRule))?;
            (
                outcome.unit?,
                UnitKind::AtRule,
                outcome.end,
                outcome.rule_end,
            )
        } else {
            match consume_qualified_rule(stream, Some(first), false, false, grammar)
                .map_err(|error| refusal(error, UnitKind::QualifiedRule))?
            {
                Qualified::Rule { result, start, end } => (
                    qualified_unit(result, start, end)?,
                    UnitKind::QualifiedRule,
                    end,
                    super::RuleEnd::Block,
                ),
                Qualified::Nothing {
                    reason,
                    start,
                    end,
                    rule_end,
                } => {
                    return Err(Failed::new(
                        UnitKind::QualifiedRule,
                        reason.into(),
                        start,
                        end,
                        Some(rule_end),
                    ));
                }
                Qualified::Quiet {
                    start,
                    end,
                    rule_end,
                } => {
                    return Err(Failed::new(
                        UnitKind::QualifiedRule,
                        SyntaxError::invalid_qualified_rule_error(start),
                        start,
                        end,
                        Some(rule_end),
                    ));
                }
            }
        };
        let start = at;
        match skip_trivia(stream) {
            Ok(None) => Ok(item),
            Ok(Some(component)) => Err(Failed::new(
                kind,
                StreamError::unexpected(stream.site(), opening_token(&component)).into(),
                start,
                end,
                Some(rule_end),
            )),
            Err(error) => Err(refusal(error, kind)),
        }
    }

    /// Parses a declaration (section 5.4.7): whitespace, then one declaration.
    /// Unlike a block's contents, this reads a declaration and not an at-rule,
    /// and nothing is required of what follows it.
    ///
    /// # Errors
    ///
    /// Section 5.4.7 returns a syntax error when no declaration is returned;
    /// the [`Failed`] says why: the stream error of the walk, or the grammar's
    /// own error.
    ///
    /// ```
    /// use surgeist_css_parser::grammar::{AtRule, BlockGrammar, Declaration, QualifiedRule, RuleGrammar};
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, SyntaxError};
    ///
    /// struct Names;
    /// impl<'i> RuleGrammar<'i> for Names {
    ///     type Item = String;
    ///     type Error = ();
    ///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
    ///         Err(SyntaxError::unknown_at_rule_error(rule.start(), rule.name()))
    ///     }
    ///     fn qualified_rule(&mut self, rule: QualifiedRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
    ///         Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    ///     }
    /// }
    /// impl<'i> BlockGrammar<'i> for Names {
    ///     fn declaration(&mut self, d: Declaration<'i>) -> Result<String, SyntaxError<'i, ()>> {
    ///         Ok(d.name().to_owned())
    ///     }
    /// }
    /// let document: Document =
    ///     Document::parse(" color: red", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// assert_eq!(document.root().parse_declaration(&mut Names).unwrap(), "color");
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn parse_declaration<G: BlockGrammar<'i, P>>(
        mut self,
        grammar: &mut G,
    ) -> Result<G::Item, Failed<'i, G::Error, P>> {
        let stream = &mut self;
        let first = skip_trivia(stream).map_err(|error| refusal(error, UnitKind::Declaration))?;
        let start = stream.site();
        let declaration = consume_declaration(stream, first, false).map_err(|error| {
            Failed::new(
                UnitKind::Declaration,
                SyntaxError::from(error),
                start,
                stream.site(),
                None,
            )
        })?;
        let (end, rule_end) = (declaration.end(), declaration.rule_end());
        grammar
            .declaration(declaration)
            .map_err(|error| Failed::new(UnitKind::Declaration, error, start, end, Some(rule_end)))
    }

    // ----- 5.4.8, 5.4.9, 5.5.8: component values

    /// Consumes a component value (section 5.5.8): the next token, or the whole
    /// function or simple block that it opens. Comments are skipped.
    ///
    /// # Errors
    ///
    /// Fails with `is_end_of_stream()` if the stream is empty, and with
    /// `is_refused()` if the meter refuses. The stream does not move on failure.
    ///
    /// ```
    /// use surgeist_css_parser::document::Component;
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let document: Document =
    ///     Document::parse("f(a b) c", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let mut stream = document.root();
    /// // The function and its contents are one component value.
    /// assert!(matches!(stream.consume_component_value().unwrap(), Component::Function { .. }));
    /// assert!(matches!(stream.consume_component_value().unwrap(), Component::Token { .. })); // the whitespace
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn consume_component_value(&mut self) -> Result<Component<'i, P>, StreamError<'i, P>> {
        let mut index = self.next_index();
        loop {
            match self.peek_at(index)? {
                Some(Component::Token {
                    token: Token::Comment,
                    ..
                }) => index += 1,
                Some(component) => {
                    self.seek(index + 1);
                    return Ok(component);
                }
                None => return Err(StreamError::end_of_stream(self.site_at(index))),
            }
        }
    }

    /// Parses a component value (section 5.4.8): whitespace, one component
    /// value, whitespace, and then the end of the stream.
    ///
    /// # Errors
    ///
    /// Section 5.4.8 returns a syntax error for an empty stream and for
    /// anything after the value but whitespace: `is_end_of_stream()` and
    /// `is_unexpected_token()` say which, and `is_refused()` a refusal. On
    /// failure the stream is where it was.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let value = |text: &str| {
    ///     let document: Document =
    ///         Document::parse(text, &Options::default(), Limits::default(), DepthExcess::Fail).unwrap();
    ///     document.root().parse_component_value().is_ok()
    /// };
    /// assert!(value("  f(a b)  "));
    /// assert!(!value("a b"));
    /// assert!(!value(" "));
    /// ```
    pub fn parse_component_value(&mut self) -> Result<Component<'i, P>, StreamError<'i, P>> {
        let mark = self.mark();
        let result = skip_trivia(self).and_then(|_| {
            let value = self.consume_component_value()?;
            match skip_trivia(self)? {
                None => Ok(value),
                Some(component) => Err(StreamError::unexpected(
                    self.site(),
                    opening_token(&component),
                )),
            }
        });
        if result.is_err() {
            self.restore(mark);
        }
        result
    }

    /// Parses a list of component values (section 5.4.9): every component
    /// value of the rest of the stream, a `}` included (the specification's
    /// "consume a list of component values" with no stop token and not
    /// nested). Comments are skipped.
    ///
    /// # Errors
    ///
    /// Fails with `is_refused()` when the meter refuses a step or the growth of
    /// the result; a refusal never returns a shorter list.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let document: Document =
    ///     Document::parse("a f(b) }", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let values = document.root().parse_list_of_component_values().unwrap();
    /// assert_eq!(values.len(), 5); // a, whitespace, f(b), whitespace, }
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn parse_list_of_component_values(
        &mut self,
    ) -> Result<Vec<Component<'i, P>>, StreamError<'i, P>> {
        let mut values = Vec::new();
        loop {
            match self.consume_component_value() {
                Ok(value) => self.push_charged(&mut values, value)?,
                Err(error) if error.is_end_of_stream() => return Ok(values),
                Err(error) => return Err(error),
            }
        }
    }

    // ----- 5.4.10 and 5.4.2: comma-separated lists

    /// Parses a comma-separated list of component values (section 5.4.10): the
    /// stream split at each top-level comma, as one stream for each group.
    ///
    /// The algorithm is literal. While the stream is not empty it takes a list
    /// up to the next comma and discards the comma. So `a,` is one group, `a,,b`
    /// is three (the middle one empty), a leading comma gives an empty first
    /// group, and an input that is only whitespace is one group. A comma
    /// inside a function or block is not a top-level comma. This is not
    /// [`comma_segments`](Self::comma_segments), which hands a grammar every
    /// segment, empty ones and a final one after a trailing comma included.
    ///
    /// The stream is at its end afterwards.
    ///
    /// # Errors
    ///
    /// Fails with `is_refused()` when the meter refuses a step or the growth of
    /// the result; a refusal never returns fewer groups.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let document: Document =
    ///     Document::parse("a, f(b, c),", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let groups = document.root().parse_comma_separated_component_values().unwrap();
    /// assert_eq!(groups.len(), 2); // `a` and ` f(b, c)`; the final comma adds none
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn parse_comma_separated_component_values(
        &mut self,
    ) -> Result<Vec<Stream<'i, P>>, StreamError<'i, P>> {
        let mut groups = Vec::new();
        loop {
            let start = self.next_index();
            let mut index = start;
            let mut significant = false;
            let stop = loop {
                match self.peek_at(index)? {
                    None => break None,
                    Some(Component::Token {
                        token: Token::Comma,
                        ..
                    }) => break Some(index),
                    Some(Component::Token {
                        token: Token::Comment,
                        ..
                    }) => index += 1,
                    Some(_) => {
                        significant = true;
                        index += 1;
                    }
                }
            };
            match stop {
                Some(comma) => {
                    let group = self.span(start, comma);
                    self.push_charged(&mut groups, group)?;
                    self.seek(comma + 1);
                }
                None => {
                    if significant {
                        let group = self.span(start, index);
                        self.push_charged(&mut groups, group)?;
                    }
                    self.seek(index);
                    return Ok(groups);
                }
            }
        }
    }

    /// Parses a comma-separated list according to a CSS grammar (section
    /// 5.4.2): an empty list for a stream of only whitespace, and otherwise the
    /// groups of [`parse_comma_separated_component_values`], each parsed by `f`
    /// as section 5.4.1 parses something according to a grammar: `f` runs over
    /// the group and the group must be empty afterwards.
    ///
    /// The result is the list of results, in order: a group that `f` rejects is
    /// an `Err` in its place and does not fail the others, which is the point
    /// of the algorithm.
    ///
    /// The stream is at its end afterwards.
    ///
    /// # Errors
    ///
    /// The outer error is a refusal of the meter (`is_refused()`): a stream that
    /// was refused anywhere in the work never returns a list.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, SyntaxError};
    ///
    /// let document: Document =
    ///     Document::parse("a, 1, b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let list = document.root().parse_comma_separated_list(|group| {
    ///     Ok::<_, SyntaxError<'_, ()>>(group.consume_ident()?)
    /// }).unwrap();
    /// assert_eq!(list.len(), 3);
    /// assert_eq!(list[0].as_ref().ok(), Some(&"a"));
    /// assert!(list[1].is_err()); // `1` is not an ident, and `a` and `b` are still there
    /// assert_eq!(list[2].as_ref().ok(), Some(&"b"));
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    ///
    /// [`parse_comma_separated_component_values`]: Self::parse_comma_separated_component_values
    pub fn parse_comma_separated_list<T, E>(
        &mut self,
        mut f: impl FnMut(&mut Stream<'i, P>) -> Result<T, SyntaxError<'i, E, P>>,
    ) -> Result<GroupResults<'i, T, E, P>, StreamError<'i, P>> {
        if self.whitespace_left()? {
            self.seek(self.end_index());
            return Ok(Vec::new());
        }
        let groups = self.parse_comma_separated_component_values()?;
        let mut results = Vec::new();
        for mut group in groups {
            let result = group.parse_complete(&mut f);
            self.push_charged(&mut results, result)?;
        }
        if self.is_refused() {
            return Err(StreamError::refused(self.site()));
        }
        Ok(results)
    }
}
