//! Cursor implementation for the parser.
mod buffered_lexer;

use crate::{
    Error,
    diagnostic::Spelling,
    frontend::{FrontendLimit, FrontendWork},
    lexer::{Error as LexError, InputElement, Lexer, Token, TokenKind},
    parser::{OrAbrupt, ParseResult},
    source::ReadChar,
};
use buffered_lexer::BufferedLexer;
use rustc_hash::FxHashSet;
use surgeist_js_ast::{
    LinearPosition, ModuleItemList, Position, PositionGroup, Punctuator, Span, Spanned,
    function::{FormalParameter, FormalParameterList, FunctionBody},
    operations::{
        CheckLabelsFailure, ContainsSymbol, DirectEvalAdmission, LabelCheckWork, NameAdmission,
        check_labels, contains, contains_arguments, has_direct_super,
        try_all_private_identifiers_valid, try_bound_names, try_contains, try_contains_arguments,
        try_has_direct_super, try_lexically_declared_names, try_lexically_declared_names_legacy,
        try_var_declared_names,
    },
    visitor::{NodeRef, VisitWith},
};
use surgeist_js_interner::{Interner, JStrRef, Sym};

pub(super) struct TokenDiagnostic {
    spelling: Spelling,
    span: Span,
}

impl Token {
    pub(super) fn diagnostic(&self) -> TokenDiagnostic {
        TokenDiagnostic {
            spelling: self.kind().diagnostic_spelling(),
            span: self.span(),
        }
    }
}

impl TokenDiagnostic {
    #[cold]
    pub(super) fn expected<R: ReadChar>(
        self,
        expected: impl Into<Box<[String]>>,
        context: &'static str,
        cursor: &mut Cursor<'_, R>,
        interner: &Interner,
    ) -> Error {
        match cursor.render_diagnostic(self.spelling, "", "", interner) {
            Ok(found) => Error::expected(expected, found, self.span, context),
            Err(limit) => limit.into(),
        }
    }

    #[cold]
    pub(super) fn unexpected<R: ReadChar>(
        self,
        message: &'static str,
        cursor: &mut Cursor<'_, R>,
        interner: &Interner,
    ) -> Error {
        match cursor.render_diagnostic(self.spelling, "", "", interner) {
            Ok(found) => Error::unexpected(found, self.span, message),
            Err(limit) => limit.into(),
        }
    }
}

/// The result of a peek for a semicolon.
#[derive(Debug)]
pub(super) enum SemicolonResult<'s> {
    Found(Option<&'s Token>),
    NotFound(&'s Token),
}

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<'control, R> {
    buffered_lexer: BufferedLexer<'control, R>,

    // The lexer owns the control for the cursor's lifetime.
    controlled: bool,

    /// Tracks if the cursor is in a arrow function declaration.
    arrow: bool,

    /// Indicate if the cursor is used in `JSON.parse`.
    json_parse: bool,

    /// A unique identifier for each parser instance.
    /// This is used to generate unique identifiers tagged template literals.
    identifier: u32,

    /// Tracks the number of tagged templates that are currently being parsed.
    tagged_templates_count: u32,
}

impl<'control, R> Cursor<'control, R>
where
    R: ReadChar,
{
    #[cold]
    fn render_diagnostic(
        &mut self,
        spelling: Spelling,
        prefix: &'static str,
        suffix: &'static str,
        interner: &Interner,
    ) -> Result<String, FrontendLimit> {
        let controlled = self.controlled;
        let mut admit = |work| self.buffered_lexer.admit_frontend(work);
        spelling.render(interner, prefix, suffix, controlled.then_some(&mut admit))
    }

    pub(super) fn diagnostic_symbol(
        &mut self,
        prefix: &'static str,
        symbol: Sym,
        suffix: &'static str,
        interner: &Interner,
    ) -> ParseResult<String> {
        Ok(self.render_diagnostic(Spelling::Symbol(symbol), prefix, suffix, interner)?)
    }

    /// Creates a new cursor with the given reader.
    pub(super) fn new(
        reader: R,
        control: Option<&'control mut crate::frontend::FrontendControl<'control>>,
    ) -> Self {
        Self {
            controlled: control.is_some(),
            buffered_lexer: Lexer::with_control(reader, control).into(),
            arrow: false,
            json_parse: false,
            identifier: 0,
            tagged_templates_count: 0,
        }
    }

    /// Sets the goal symbol of the cursor to `Module`.
    pub(super) fn set_module(&mut self) {
        self.buffered_lexer.set_module(true);
    }

    /// Returns `true` if the cursor is currently parsing a `Module`.
    pub(super) const fn module(&self) -> bool {
        self.buffered_lexer.module()
    }

    pub(super) fn set_goal(&mut self, elm: InputElement) {
        self.buffered_lexer.set_goal(elm);
    }

    #[inline]
    pub(super) fn admit_grammar(&mut self) -> ParseResult<()> {
        self.buffered_lexer
            .admit_frontend(FrontendWork::GrammarDispatch)
            .map_err(Error::from)
    }

    #[inline]
    pub(super) fn admit_early_error_visit(&mut self) -> ParseResult<()> {
        self.buffered_lexer
            .admit_frontend(FrontendWork::EarlyErrorVisit)
            .map_err(Error::from)
    }

    #[inline]
    pub(super) fn admit_name_comparison(&mut self) -> ParseResult<()> {
        self.buffered_lexer
            .admit_frontend(FrontendWork::NameComparison)
            .map_err(Error::from)
    }

    pub(super) fn intern<'a>(
        &mut self,
        interner: &mut Interner,
        spelling: impl Into<JStrRef<'a>>,
    ) -> ParseResult<Sym> {
        self.buffered_lexer
            .intern(interner, spelling)
            .map_err(Error::from)
    }

    pub(super) fn admit_interner_name_preparation(&mut self, units: usize) -> ParseResult<()> {
        self.buffered_lexer
            .admit_frontend(FrontendWork::InternerNamePreparation(units))
            .map_err(Error::from)
    }

    fn with_name_admission<T>(
        &mut self,
        collect: impl FnOnce(NameAdmission<'_, FrontendLimit>) -> Result<T, FrontendLimit>,
    ) -> ParseResult<T> {
        let result = if self.controlled {
            collect(Some(&mut |work| {
                self.buffered_lexer.admit_frontend(work.into())
            }))
        } else {
            collect(None)
        };
        result.map_err(Error::from)
    }

    pub(super) fn formal_parameter_list(
        &mut self,
        parameters: Vec<FormalParameter>,
    ) -> ParseResult<FormalParameterList> {
        self.with_name_admission(|admission| {
            FormalParameterList::try_from_parameters(parameters, admission)
        })
    }

    pub(super) fn bound_names<'a, N>(&mut self, node: &'a N) -> ParseResult<Vec<Sym>>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        self.with_name_admission(|admission| try_bound_names(node, admission))
    }

    pub(super) fn all_private_identifiers_valid<'a, N>(&mut self, node: &'a N) -> ParseResult<bool>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        self.with_name_admission(|admission| {
            try_all_private_identifiers_valid(node, Vec::new(), admission)
        })
    }

    pub(super) fn lexically_declared_names<'a, N>(&mut self, node: &'a N) -> ParseResult<Vec<Sym>>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        self.with_name_admission(|admission| try_lexically_declared_names(node, admission))
    }

    pub(super) fn lexically_declared_names_legacy<'a, N>(
        &mut self,
        node: &'a N,
    ) -> ParseResult<Vec<(Sym, bool)>>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        self.with_name_admission(|admission| try_lexically_declared_names_legacy(node, admission))
    }

    pub(super) fn var_declared_names<'a, N>(&mut self, node: &'a N) -> ParseResult<FxHashSet<Sym>>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        self.with_name_admission(|admission| try_var_declared_names(node, admission))
    }

    pub(super) fn exported_names(&mut self, items: &ModuleItemList) -> ParseResult<Vec<Sym>> {
        self.with_name_admission(|admission| items.try_exported_names(admission))
    }

    pub(super) fn exported_bindings(
        &mut self,
        items: &ModuleItemList,
    ) -> ParseResult<FxHashSet<Sym>> {
        self.with_name_admission(|admission| items.try_exported_bindings(admission))
    }

    pub(super) fn with_direct_eval_admission<T>(
        &mut self,
        construct: impl FnOnce(DirectEvalAdmission<'_, FrontendLimit>) -> Result<T, FrontendLimit>,
    ) -> ParseResult<T> {
        let result = if self.controlled {
            construct(Some(&mut || {
                self.buffered_lexer
                    .admit_frontend(FrontendWork::DirectEvalVisit)
            }))
        } else {
            construct(None)
        };
        result.map_err(Error::from)
    }

    pub(super) fn contains(
        &mut self,
        node: &impl VisitWith,
        symbol: ContainsSymbol,
    ) -> ParseResult<bool> {
        if self.controlled {
            try_contains(node, symbol, || self.admit_early_error_visit())
        } else {
            Ok(contains(node, symbol))
        }
    }

    pub(super) fn contains_arguments(&mut self, node: &impl VisitWith) -> ParseResult<bool> {
        if self.controlled {
            try_contains_arguments(node, || self.admit_early_error_visit())
        } else {
            Ok(contains_arguments(node))
        }
    }

    pub(super) fn has_direct_super(
        &mut self,
        params: &FormalParameterList,
        body: &FunctionBody,
    ) -> ParseResult<bool> {
        if self.controlled {
            try_has_direct_super(params, body, || self.admit_early_error_visit())
        } else {
            Ok(has_direct_super(params, body))
        }
    }

    pub(super) fn check_labels(
        &mut self,
        node: &impl VisitWith,
        interner: &Interner,
        position: Position,
    ) -> ParseResult<()> {
        check_labels(node, |work| {
            let work = match work {
                LabelCheckWork::Visit => FrontendWork::EarlyErrorVisit,
                LabelCheckWork::Lookup => FrontendWork::LabelLookup,
            };
            self.buffered_lexer.admit_frontend(work)
        })
        .or_else(|error| match error {
            CheckLabelsFailure::Syntax(error) => {
                let (prefix, label) = error.message_parts();
                let message = if let Some(label) = label {
                    self.diagnostic_symbol(prefix, label, "", interner)?.into()
                } else {
                    prefix.into()
                };
                Err(Error::lex(LexError::Syntax(message, position)))
            }
            CheckLabelsFailure::Refused(limit) => Err(limit.into()),
        })
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    /// If `init_with_eq` is `true`, then assuming that the starting '/=' has already been consumed.
    pub(super) fn lex_regex(
        &mut self,
        start: PositionGroup,
        interner: &mut Interner,
        init_with_eq: bool,
    ) -> ParseResult<Token> {
        self.buffered_lexer.lex_regex(start, interner, init_with_eq)
    }

    pub(super) fn lex_template(
        &mut self,
        start: PositionGroup,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        self.buffered_lexer.lex_template(start, interner)
    }

    /// Advances the cursor and returns the next token.
    pub(super) fn next(&mut self, interner: &mut Interner) -> ParseResult<Option<Token>> {
        self.buffered_lexer.next(true, interner)
    }

    /// Advances the cursor without returning the next token.
    pub(super) fn advance(&mut self, interner: &mut Interner) -> ParseResult<()> {
        self.next(interner).map(drop)
    }

    /// Peeks a future token, without consuming it or advancing the cursor.
    /// This peeking **skips** line terminators.
    ///
    /// You can skip some tokens with the `skip_n` option.
    pub(super) fn peek(
        &mut self,
        skip_n: usize,
        interner: &mut Interner,
    ) -> ParseResult<Option<&Token>> {
        self.buffered_lexer.peek(skip_n, true, interner)
    }

    /// Peeks a future token, without consuming it or advancing the cursor.
    /// This peeking **does not skips** line terminators.
    ///
    /// You can skip some tokens with the `skip_n` option.
    pub(super) fn peek_no_skip_line_term(
        &mut self,
        skip_n: usize,
        interner: &mut Interner,
    ) -> ParseResult<Option<&Token>> {
        self.buffered_lexer.peek(skip_n, false, interner)
    }

    /// Gets the current strict mode for the cursor.
    pub(super) const fn strict(&self) -> bool {
        self.buffered_lexer.strict()
    }

    /// Sets the strict mode to strict or non-strict.
    pub(super) fn set_strict(&mut self, strict: bool) {
        self.buffered_lexer.set_strict(strict);
    }

    /// Returns if the cursor is currently in an arrow function declaration.
    pub(super) const fn arrow(&self) -> bool {
        self.arrow
    }

    /// Set if the cursor is currently in a arrow function declaration.
    pub(super) fn set_arrow(&mut self, arrow: bool) {
        self.arrow = arrow;
    }

    /// Returns if the cursor is currently used in `JSON.parse`.
    pub(super) const fn json_parse(&self) -> bool {
        self.json_parse
    }

    /// Set if the cursor is currently used in `JSON.parse`.
    pub(super) fn set_json_parse(&mut self, json_parse: bool) {
        self.json_parse = json_parse;
    }

    /// Set the identifier of the cursor.
    #[inline]
    pub(super) fn set_identifier(&mut self, identifier: u32) {
        self.identifier = identifier;
    }

    /// Get the identifier for a tagged template.
    #[inline]
    pub(super) fn tagged_template_identifier(&mut self) -> Option<u64> {
        self.tagged_templates_count = self.tagged_templates_count.checked_add(1)?;

        let identifier = u64::from(self.identifier);
        let count = u64::from(self.tagged_templates_count);

        Some((count << 32) | identifier)
    }

    /// Returns an error if the next token is not of kind `kind`.
    pub(super) fn expect<K>(
        &mut self,
        kind: K,
        context: &'static str,
        interner: &mut Interner,
    ) -> ParseResult<Token>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.next(interner).or_abrupt()?;
        let kind = kind.into();

        if next_token.kind() == &kind {
            Ok(next_token)
        } else {
            let expected = self.render_diagnostic(kind.diagnostic_spelling(), "", "", interner)?;
            Err(next_token
                .diagnostic()
                .expected([expected], context, self, interner))
        }
    }

    /// It will peek for the next token, to see if it's a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn peek_semicolon(
        &mut self,
        interner: &mut Interner,
    ) -> ParseResult<SemicolonResult<'_>> {
        self.peek_no_skip_line_term(0, interner)?
            .map_or(Ok(SemicolonResult::Found(None)), |tk| match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon | Punctuator::CloseBlock)
                | TokenKind::LineTerminator => Ok(SemicolonResult::Found(Some(tk))),
                _ => Ok(SemicolonResult::NotFound(tk)),
            })
    }

    /// Consumes the next token if it is a semicolon, or returns a `Error` if it's not.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn expect_semicolon(
        &mut self,
        context: &'static str,
        interner: &mut Interner,
    ) -> ParseResult<()> {
        match self.peek_semicolon(interner)? {
            SemicolonResult::Found(Some(tk)) => match *tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    let _next = self.buffered_lexer.next(false, interner)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            SemicolonResult::Found(None) => Ok(()),
            SemicolonResult::NotFound(tk) => {
                Err(tk
                    .diagnostic()
                    .expected([";".to_owned()], context, self, interner))
            }
        }
    }

    /// It will make sure that the peeked token (skipping n tokens) is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// This is just syntactic sugar for a `.peek(skip_n)` call followed by a check that the result
    /// is not a line terminator or `None`.
    pub(super) fn peek_expect_no_lineterminator(
        &mut self,
        skip_n: usize,
        context: &'static str,
        interner: &mut Interner,
    ) -> ParseResult<&Token> {
        let tok = self.peek_no_skip_line_term(skip_n, interner).or_abrupt()?;

        if tok.kind() == &TokenKind::LineTerminator {
            Err(Error::unexpected("line terminator", tok.span(), context))
        } else {
            Ok(tok)
        }
    }

    /// Check if the peeked token is a line terminator.
    pub(super) fn peek_is_line_terminator(
        &mut self,
        skip_n: usize,
        interner: &mut Interner,
    ) -> ParseResult<Option<bool>> {
        self.peek_no_skip_line_term(skip_n, interner)?
            .map_or(Ok(None), |t| {
                Ok(Some(t.kind() == &TokenKind::LineTerminator))
            })
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    ///
    /// No next token also returns None.
    pub(super) fn next_if<K>(
        &mut self,
        kind: K,
        interner: &mut Interner,
    ) -> ParseResult<Option<Token>>
    where
        K: Into<TokenKind>,
    {
        if let Some(token) = self.peek(0, interner)?
            && token.kind() == &kind.into()
        {
            self.next(interner)
        } else {
            Ok(None)
        }
    }

    /// Gets current linear position in the source code.
    #[inline]
    pub(super) fn linear_pos(&self) -> LinearPosition {
        self.buffered_lexer.linear_pos()
    }

    pub(super) fn take_source(&mut self) -> surgeist_js_ast::SourceText {
        self.buffered_lexer.take_source()
    }
}
