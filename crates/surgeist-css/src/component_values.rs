//! Owned CSS component values and token-preserving serialization.
//!
//! This syntax layer does not interpret a property, substitute variables, or
//! implement CSSOM property serialization. Original token representations and
//! independently originating replacement tokens remain available to consumers.

mod parse;
mod serialize;
pub(crate) use serialize::{CssCanonicalBuilder, CssCanonicalToken};

use std::fmt;
use std::ops::Range;
use std::sync::Arc;

use cssparser::{ToCss, Token, TokenSerializationType};

use crate::source::{CssSourcePosition, CssSourceSpan};

use crate::STRUCTURAL_NESTING_LIMIT as MAX_COMPONENT_DEPTH;

/// Resource limits for one component-value construction or parse.
///
/// Components include whitespace and comment nodes; opening a function or block
/// adds one node, and its closing delimiter does not add another. Depth zero
/// admits leaf tokens. The byte limit applies to source input and serialized
/// output, including inserted separators and implicit closing delimiters.
/// These are syntax-length limits, not a global allocator or process-memory cap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssComponentValueLimits {
    max_depth: u32,
    max_components: usize,
    max_css_bytes: usize,
}

impl CssComponentValueLimits {
    /// Creates limits without allowing the parser's 256-level ceiling to rise.
    #[must_use]
    pub const fn try_new(
        max_depth: u32,
        max_components: usize,
        max_css_bytes: usize,
    ) -> Option<Self> {
        if max_depth > MAX_COMPONENT_DEPTH {
            None
        } else {
            Some(Self {
                max_depth,
                max_components,
                max_css_bytes,
            })
        }
    }

    /// Returns the maximum number of enclosing component blocks or functions.
    #[must_use]
    pub const fn max_nesting_depth(self) -> u32 {
        self.max_depth
    }

    /// Returns the maximum component count, including descendants and trivia.
    #[must_use]
    pub const fn max_components(self) -> usize {
        self.max_components
    }

    /// Returns the maximum source or serialized UTF-8 length.
    #[must_use]
    pub const fn max_css_bytes(self) -> usize {
        self.max_css_bytes
    }
}

impl Default for CssComponentValueLimits {
    /// Keeps the structural ceiling while imposing no additional length policy.
    fn default() -> Self {
        Self {
            max_depth: MAX_COMPONENT_DEPTH,
            max_components: usize::MAX,
            max_css_bytes: usize::MAX,
        }
    }
}

/// One immutable parse input. Equality compares source text.
///
/// [`Self::same_snapshot`] separately compares parse-input identity. Neither
/// operation represents a mutable stylesheet revision or CSSOM object handle.
#[derive(Clone)]
pub struct CssSourceSnapshot(Arc<SourceSnapshotData>);

struct SourceSnapshotData {
    text: Box<str>,
    checkpoints: Box<[CssSourcePosition]>,
}

impl CssSourceSnapshot {
    pub(crate) fn new(source: &str) -> Self {
        let mut position = CssSourcePosition::from_byte_offset_in("", 0);
        let mut checkpoints = vec![position];
        let mut start = 0;
        let mut characters = source.char_indices().peekable();
        while let Some((offset, character)) = characters.next() {
            let mut end = offset + character.len_utf8();
            if character == '\r' && characters.peek().is_some_and(|(_, next)| *next == '\n') {
                let (offset, character) = characters.next().expect("peeked LF");
                end = offset + character.len_utf8();
            }
            if end - start >= 64 {
                position = position.advanced_by(&source[start..end]);
                checkpoints.push(position);
                start = end;
            }
        }
        Self(Arc::new(SourceSnapshotData {
            text: source.into(),
            checkpoints: checkpoints.into_boxed_slice(),
        }))
    }

    pub(crate) fn position_at(&self, byte_offset: usize) -> Option<CssSourcePosition> {
        if !self.as_str().is_char_boundary(byte_offset) {
            return None;
        }
        let index = self
            .0
            .checkpoints
            .partition_point(|position| position.byte_offset().value() <= byte_offset);
        let checkpoint = self.0.checkpoints[index - 1];
        self.as_str()
            .get(checkpoint.byte_offset().value()..byte_offset)
            .map(|suffix| checkpoint.advanced_by(suffix))
    }

    /// Returns the complete original UTF-8 parse input.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0.text
    }

    /// Reports whether two origins refer to the same immutable parse input.
    #[must_use]
    pub fn same_snapshot(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl fmt::Debug for CssSourceSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CssSourceSnapshot")
            .field(&self.as_str())
            .finish()
    }
}

impl PartialEq for CssSourceSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for CssSourceSnapshot {}

/// A parser-produced span bound to the immutable source that it addresses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssParsedOrigin {
    source: CssSourceSnapshot,
    span: CssSourceSpan,
}

impl CssParsedOrigin {
    pub(crate) fn from_range(source: &CssSourceSnapshot, range: Range<usize>) -> Option<Self> {
        Some(Self {
            source: source.clone(),
            span: CssSourceSpan::new(
                source.position_at(range.start)?,
                source.position_at(range.end)?,
            )?,
        })
    }

    /// Returns the source snapshot that owns this span.
    #[must_use]
    pub const fn source(&self) -> &CssSourceSnapshot {
        &self.source
    }

    /// Returns the original authored-source coordinates.
    #[must_use]
    pub const fn span(&self) -> CssSourceSpan {
        self.span
    }
}

/// Provenance for a component token or delimiter, without fabricated positions.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssValueOrigin {
    /// Input rejected before tokenization, without copying its source bytes.
    UnretainedInput {
        /// The rejected input's UTF-8 byte length, without an authored span.
        byte_length: usize,
    },
    /// The token or delimiter was consumed from this original source span.
    Parsed(CssParsedOrigin),
    /// Checked Rust construction supplied the syntax, without authored source.
    Programmatic,
    /// EOF supplied syntax that closes an opening token, block, or function.
    ImplicitClosure {
        /// The token or delimiter whose termination was implied.
        opening: CssParsedOrigin,
        /// The zero-width source position at EOF.
        at: CssParsedOrigin,
    },
}

/// A typed component-construction or serialization failure category.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssComponentValueErrorKind {
    /// A tokenizer-level bad string was encountered.
    BadString,
    /// A tokenizer-level bad URL was encountered.
    BadUrl,
    /// A closing delimiter had no matching opener in this component stream.
    UnmatchedClosingDelimiter,
    /// The spelling did not represent exactly the required token.
    InvalidToken,
    /// The decoded identifier cannot retain its identity through CSS tokenization.
    InvalidIdentifier,
    /// The spelling was not exactly one CSS number token.
    InvalidNumber,
    /// The numeric representation or unit cannot form the requested dimension.
    InvalidDimension,
    /// The requested function cannot be represented as a function with these children.
    InvalidFunction,
    /// A requested string or URL contains a code point tokenization would replace.
    InvalidString,
    /// A token boundary cannot be serialized without changing meaningful whitespace.
    UnserializableBoundary,
    /// A nested component exceeded the selected structural limit.
    NestingLimit,
    /// The selected maximum component count was exceeded.
    ComponentLimit,
    /// Source or serialized output exceeded the selected byte limit.
    ByteLimit,
    /// A size calculation could not be represented by the platform.
    CapacityOverflow,
}

/// A typed failure located in original source or explicitly programmatic syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssComponentValueError {
    kind: CssComponentValueErrorKind,
    origin: CssValueOrigin,
}

impl CssComponentValueError {
    /// Returns the violated syntax or resource invariant.
    #[must_use]
    pub const fn kind(&self) -> CssComponentValueErrorKind {
        self.kind
    }

    /// Returns the responsible origin; programmatic errors have no parsed span.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }

    pub(crate) fn new(kind: CssComponentValueErrorKind, origin: CssValueOrigin) -> Self {
        Self { kind, origin }
    }

    fn programmatic(kind: CssComponentValueErrorKind) -> Self {
        Self::new(kind, CssValueOrigin::Programmatic)
    }
}

impl fmt::Display for CssComponentValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS component value {:?}", self.kind)?;
        if let CssValueOrigin::Parsed(origin) = &self.origin {
            write!(formatter, " at {}", origin.span.start())?;
        }
        Ok(())
    }
}

impl std::error::Error for CssComponentValueError {}

/// The lexical number-versus-integer distinction; this does not imply a range.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssNumericTokenKind {
    /// The representation contains no decimal part or exponent.
    Integer,
    /// The representation contains a decimal part or exponent.
    Number,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NumericToken {
    representation: Box<str>,
    kind: CssNumericTokenKind,
    has_sign: bool,
}

/// A borrowed lexical numeric token, preserving precision and sign spelling.
#[derive(Clone, Copy, Debug)]
pub struct CssNumericTokenRef<'a>(&'a NumericToken);

impl<'a> CssNumericTokenRef<'a> {
    /// Returns the exact numeric representation, without any dimension unit or `%`.
    #[must_use]
    pub fn representation(self) -> &'a str {
        &self.0.representation
    }

    /// Returns the token's lexical number type.
    #[must_use]
    pub const fn kind(self) -> CssNumericTokenKind {
        self.0.kind
    }

    /// Returns whether an explicit `+` or `-` was present.
    #[must_use]
    pub const fn has_sign(self) -> bool {
        self.0.has_sign
    }
}

/// The CSS hash-token flag, independent of its source escape spelling.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssHashFlag {
    /// The token's decoded name begins as a CSS identifier.
    Id,
    /// The token has the unrestricted hash type.
    Unrestricted,
}

/// A borrowed semantic view of an owned token. Closing delimiters are structural.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum CssValueTokenRef<'a> {
    /// A decoded identifier.
    Ident(&'a str),
    /// A decoded at-keyword without `@`.
    AtKeyword(&'a str),
    /// A decoded hash without `#`, retaining the type flag.
    Hash { value: &'a str, flag: CssHashFlag },
    /// A decoded quoted string.
    String(&'a str),
    /// A decoded unquoted URL token; quoted URLs are functions.
    Url(&'a str),
    /// A delimiter character.
    Delim(char),
    /// A lexical number.
    Number(CssNumericTokenRef<'a>),
    /// A lexical percentage, with its numeric spelling before `%`.
    Percentage(CssNumericTokenRef<'a>),
    /// A lexical dimension and its decoded unit.
    Dimension {
        number: CssNumericTokenRef<'a>,
        unit: &'a str,
    },
    /// Actual CSS whitespace, distinct from comments.
    Whitespace(&'a str),
    /// A colon token.
    Colon,
    /// A semicolon token.
    Semicolon,
    /// A comma token.
    Comma,
    /// An include-match token.
    IncludeMatch,
    /// A dash-match token.
    DashMatch,
    /// A prefix-match token.
    PrefixMatch,
    /// A suffix-match token.
    SuffixMatch,
    /// A substring-match token.
    SubstringMatch,
    /// A CDO token.
    Cdo,
    /// A CDC token.
    Cdc,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Lexeme {
    text: Box<str>,
    origin: CssValueOrigin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValueToken {
    data: TokenData,
    spelling: Lexeme,
    implicit_end: Option<Lexeme>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenData {
    Ident(Box<str>),
    AtKeyword(Box<str>),
    Hash {
        value: Box<str>,
        flag: CssHashFlag,
    },
    String(Box<str>),
    Url(Box<str>),
    Delim(char),
    Number(NumericToken),
    Percentage(NumericToken),
    Dimension {
        number: NumericToken,
        unit: Box<str>,
    },
    Whitespace,
    Colon,
    Semicolon,
    Comma,
    IncludeMatch,
    DashMatch,
    PrefixMatch,
    SuffixMatch,
    SubstringMatch,
    Cdo,
    Cdc,
}

impl ValueToken {
    fn view(&self) -> CssValueTokenRef<'_> {
        match &self.data {
            TokenData::Ident(value) => CssValueTokenRef::Ident(value),
            TokenData::AtKeyword(value) => CssValueTokenRef::AtKeyword(value),
            TokenData::Hash { value, flag } => CssValueTokenRef::Hash { value, flag: *flag },
            TokenData::String(value) => CssValueTokenRef::String(value),
            TokenData::Url(value) => CssValueTokenRef::Url(value),
            TokenData::Delim(value) => CssValueTokenRef::Delim(*value),
            TokenData::Number(value) => CssValueTokenRef::Number(CssNumericTokenRef(value)),
            TokenData::Percentage(value) => CssValueTokenRef::Percentage(CssNumericTokenRef(value)),
            TokenData::Dimension { number, unit } => CssValueTokenRef::Dimension {
                number: CssNumericTokenRef(number),
                unit,
            },
            TokenData::Whitespace => CssValueTokenRef::Whitespace(&self.spelling.text),
            TokenData::Colon => CssValueTokenRef::Colon,
            TokenData::Semicolon => CssValueTokenRef::Semicolon,
            TokenData::Comma => CssValueTokenRef::Comma,
            TokenData::IncludeMatch => CssValueTokenRef::IncludeMatch,
            TokenData::DashMatch => CssValueTokenRef::DashMatch,
            TokenData::PrefixMatch => CssValueTokenRef::PrefixMatch,
            TokenData::SuffixMatch => CssValueTokenRef::SuffixMatch,
            TokenData::SubstringMatch => CssValueTokenRef::SubstringMatch,
            TokenData::Cdo => CssValueTokenRef::Cdo,
            TokenData::Cdc => CssValueTokenRef::Cdc,
        }
    }

    fn serialization_type(&self) -> TokenSerializationType {
        use TokenSerializationType as Kind;
        match &self.data {
            TokenData::Ident(_) => Kind::Ident,
            TokenData::AtKeyword(_) | TokenData::Hash { .. } => Kind::AtKeywordOrHash,
            TokenData::String(_) => Kind::Other,
            TokenData::Url(_) => Kind::UrlOrBadUrl,
            TokenData::Delim(character) => Token::Delim(*character).serialization_type(),
            TokenData::Number(_) => Kind::Number,
            TokenData::Percentage(_) => Kind::Percentage,
            TokenData::Dimension { .. } => Kind::Dimension,
            TokenData::Whitespace => Kind::WhiteSpace,
            TokenData::DashMatch => Kind::DashMatch,
            TokenData::SubstringMatch => Kind::SubstringMatch,
            TokenData::Cdc => Kind::CDC,
            TokenData::Colon
            | TokenData::Semicolon
            | TokenData::Comma
            | TokenData::IncludeMatch
            | TokenData::PrefixMatch
            | TokenData::SuffixMatch
            | TokenData::Cdo => Kind::Other,
        }
    }
}

/// A simple block's matched delimiter kind.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssBlockKind {
    /// A parenthesis block.
    Parenthesis,
    /// A square-bracket block.
    SquareBracket,
    /// A curly-bracket block.
    CurlyBracket,
}

impl CssBlockKind {
    const fn delimiters(self) -> (&'static str, &'static str) {
        match self {
            Self::Parenthesis => ("(", ")"),
            Self::SquareBracket => ("[", "]"),
            Self::CurlyBracket => ("{", "}"),
        }
    }
}

/// An immutable function, preserving its decoded name and component arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFunctionValue {
    name: Box<str>,
    opening: Lexeme,
    values: CssComponentValues,
    closing: Lexeme,
}

impl CssFunctionValue {
    /// Returns the decoded, case-preserving function name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the immutable arguments, including whitespace and comments.
    #[must_use]
    pub const fn values(&self) -> &CssComponentValues {
        &self.values
    }

    /// Returns the closing delimiter's authored, implicit, or programmatic origin.
    #[must_use]
    pub const fn closing_origin(&self) -> &CssValueOrigin {
        &self.closing.origin
    }
}

/// An immutable simple block with a matched or EOF-implied closing delimiter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSimpleBlock {
    kind: CssBlockKind,
    opening: Lexeme,
    values: CssComponentValues,
    closing: Lexeme,
}

impl CssSimpleBlock {
    /// Returns the matched delimiter kind.
    #[must_use]
    pub const fn kind(&self) -> CssBlockKind {
        self.kind
    }

    /// Returns the immutable block contents.
    #[must_use]
    pub const fn values(&self) -> &CssComponentValues {
        &self.values
    }

    /// Returns the closing delimiter's origin.
    #[must_use]
    pub const fn closing_origin(&self) -> &CssValueOrigin {
        &self.closing.origin
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ComponentData {
    Token(ValueToken),
    Function(CssFunctionValue),
    Block(CssSimpleBlock),
    Comment {
        content: Box<str>,
        spelling: Lexeme,
        implicit_end: Option<Lexeme>,
    },
}

/// One checked, owned component value. Private fields protect token identity.
///
/// Equality includes exact token spelling, child order, delimiter origins and
/// the complete parsed span. Source text and span equality does not require
/// snapshot identity; parsed and programmatic components remain distinct.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssComponentValue {
    data: ComponentData,
    parsed: Option<CssParsedOrigin>,
}

/// A borrowed view of one immutable component value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum CssComponentValueRef<'a> {
    /// A token that is not a structural opening or closing delimiter.
    Token(CssValueTokenRef<'a>),
    /// A function and its arguments.
    Function(&'a CssFunctionValue),
    /// A simple block and its contents.
    Block(&'a CssSimpleBlock),
    /// Preserved comment contents, without the comment delimiters.
    Comment(&'a str),
}

impl CssComponentValue {
    pub(crate) fn collect_from_parser(
        input: &mut cssparser::Parser<'_, '_>,
        source: &CssSourceSnapshot,
    ) -> Result<Self, CssComponentValueError> {
        parse::collect_one(input, source)
    }

    pub(crate) const fn parsed_origin(&self) -> Option<&CssParsedOrigin> {
        self.parsed.as_ref()
    }

    /// Inspects the semantic token or component without permitting mutation.
    #[must_use]
    pub fn view(&self) -> CssComponentValueRef<'_> {
        match &self.data {
            ComponentData::Token(token) => CssComponentValueRef::Token(token.view()),
            ComponentData::Function(function) => CssComponentValueRef::Function(function),
            ComponentData::Block(block) => CssComponentValueRef::Block(block),
            ComponentData::Comment { content, .. } => CssComponentValueRef::Comment(content),
        }
    }

    /// Returns this token's or component opener's origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match &self.data {
            ComponentData::Token(token) => &token.spelling.origin,
            ComponentData::Function(function) => &function.opening.origin,
            ComponentData::Block(block) => &block.opening.origin,
            ComponentData::Comment { spelling, .. } => &spelling.origin,
        }
    }

    /// Constructs exactly one complete non-structural token from CSS spelling.
    ///
    /// This is programmatic input: it cannot create a parsed-source origin.
    pub fn try_token(spelling: &str) -> Result<Self, CssComponentValueError> {
        parse::programmatic_token(spelling, CssComponentValueErrorKind::InvalidToken)
    }

    /// Constructs an identifier from its decoded value, escaping it as needed.
    pub fn try_ident(value: impl Into<String>) -> Result<Self, CssComponentValueError> {
        let value = value.into();
        if value.is_empty() || value.contains('\0') {
            return Err(CssComponentValueError::programmatic(
                CssComponentValueErrorKind::InvalidIdentifier,
            ));
        }
        let token = Token::Ident(value.as_str().into()).to_css_string();
        let result =
            parse::programmatic_token(&token, CssComponentValueErrorKind::InvalidIdentifier)?;
        if matches!(result.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(decoded)) if decoded == value)
        {
            Ok(result)
        } else {
            Err(CssComponentValueError::programmatic(
                CssComponentValueErrorKind::InvalidIdentifier,
            ))
        }
    }

    /// Constructs a lexical number without rounding or imposing property ranges.
    pub fn try_number(representation: &str) -> Result<Self, CssComponentValueError> {
        let result =
            parse::programmatic_token(representation, CssComponentValueErrorKind::InvalidNumber)?;
        if matches!(
            result.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Number(_))
        ) {
            Ok(result)
        } else {
            Err(CssComponentValueError::programmatic(
                CssComponentValueErrorKind::InvalidNumber,
            ))
        }
    }

    /// Constructs a dimension from an exact number representation and decoded unit.
    pub fn try_dimension(
        number: &str,
        unit: impl Into<String>,
    ) -> Result<Self, CssComponentValueError> {
        let unit = unit.into();
        Self::try_number(number).map_err(|_| {
            CssComponentValueError::programmatic(CssComponentValueErrorKind::InvalidDimension)
        })?;
        Self::try_ident(unit.clone()).map_err(|_| {
            CssComponentValueError::programmatic(CssComponentValueErrorKind::InvalidDimension)
        })?;
        let escaped_unit = if let Some(rest) = unit.strip_prefix('e') {
            format!("\\65 {}", Token::Ident(rest.into()).to_css_string())
        } else if let Some(rest) = unit.strip_prefix('E') {
            format!("\\45 {}", Token::Ident(rest.into()).to_css_string())
        } else {
            Token::Ident(unit.as_str().into()).to_css_string()
        };
        let result = parse::programmatic_token(
            &format!("{number}{escaped_unit}"),
            CssComponentValueErrorKind::InvalidDimension,
        )?;
        if matches!(result.view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { number: actual, unit: actual_unit })
            if actual.representation() == number && actual_unit == unit)
        {
            Ok(result)
        } else {
            Err(CssComponentValueError::programmatic(
                CssComponentValueErrorKind::InvalidDimension,
            ))
        }
    }

    /// Constructs a decoded string without replacing NUL code points.
    pub fn try_string(value: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_text_token(value.into(), false)
    }

    /// Constructs an unquoted URL token, escaping whitespace and delimiters.
    pub fn try_url(value: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_text_token(value.into(), true)
    }

    fn try_text_token(value: String, url: bool) -> Result<Self, CssComponentValueError> {
        if value.contains('\0') {
            return Err(CssComponentValueError::programmatic(
                CssComponentValueErrorKind::InvalidString,
            ));
        }
        let spelling = if url {
            Token::UnquotedUrl(value.as_str().into())
        } else {
            Token::QuotedString(value.as_str().into())
        }
        .to_css_string();
        let result =
            parse::programmatic_token(&spelling, CssComponentValueErrorKind::InvalidString)?;
        let same = match result.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Url(decoded)) if url => decoded == value,
            CssComponentValueRef::Token(CssValueTokenRef::String(decoded)) if !url => {
                decoded == value
            }
            _ => false,
        };
        if same {
            Ok(result)
        } else {
            Err(CssComponentValueError::programmatic(
                CssComponentValueErrorKind::InvalidString,
            ))
        }
    }

    /// Constructs a function while preserving the origins of its argument tokens.
    pub fn try_function(
        name: impl Into<String>,
        values: CssComponentValues,
    ) -> Result<Self, CssComponentValueError> {
        let name = name.into();
        Self::try_ident(name.clone()).map_err(|_| {
            CssComponentValueError::programmatic(CssComponentValueErrorKind::InvalidFunction)
        })?;
        check_child_depth(&values)?;
        if name.eq_ignore_ascii_case("url") {
            let first = values.items.iter().find(|value| {
                !matches!(
                    value.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                )
            });
            if !matches!(
                first.map(Self::view),
                Some(CssComponentValueRef::Token(CssValueTokenRef::String(_)))
            ) {
                return Err(CssComponentValueError::programmatic(
                    CssComponentValueErrorKind::InvalidFunction,
                ));
            }
        }
        let opening = programmatic_lexeme(Token::Function(name.as_str().into()).to_css_string());
        Ok(Self {
            parsed: None,
            data: ComponentData::Function(CssFunctionValue {
                opening,
                name: name.into_boxed_str(),
                values,
                closing: programmatic_lexeme(")"),
            }),
        })
    }

    /// Constructs a matched simple block, preserving child origins and depth bounds.
    pub fn try_block(
        kind: CssBlockKind,
        values: CssComponentValues,
    ) -> Result<Self, CssComponentValueError> {
        check_child_depth(&values)?;
        let (opening, closing) = kind.delimiters();
        Ok(Self {
            parsed: None,
            data: ComponentData::Block(CssSimpleBlock {
                kind,
                opening: programmatic_lexeme(opening),
                values,
                closing: programmatic_lexeme(closing),
            }),
        })
    }

    fn child_values(&self) -> Option<&CssComponentValues> {
        match &self.data {
            ComponentData::Function(function) => Some(&function.values),
            ComponentData::Block(block) => Some(&block.values),
            ComponentData::Token(_) | ComponentData::Comment { .. } => None,
        }
    }
}

fn check_child_depth(values: &CssComponentValues) -> Result<(), CssComponentValueError> {
    if values.depth >= MAX_COMPONENT_DEPTH {
        Err(CssComponentValueError::programmatic(
            CssComponentValueErrorKind::NestingLimit,
        ))
    } else {
        Ok(())
    }
}

fn programmatic_lexeme(text: impl Into<Box<str>>) -> Lexeme {
    Lexeme {
        text: text.into(),
        origin: CssValueOrigin::Programmatic,
    }
}

/// A checked immutable sequence, including its whitespace and preserved comments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssComponentValues {
    items: Box<[CssComponentValue]>,
    count: usize,
    depth: u32,
}

impl CssComponentValues {
    pub(crate) fn collect_from_parser(
        input: &mut cssparser::Parser<'_, '_>,
        source: &CssSourceSnapshot,
    ) -> Result<Self, CssComponentValueError> {
        parse::collect(input, source, CssComponentValueLimits::default())
    }

    /// Joins components without merging tokens or changing their origins.
    pub fn try_new(items: Vec<CssComponentValue>) -> Result<Self, CssComponentValueError> {
        Self::try_new_with_limits(items, CssComponentValueLimits::default())
    }

    /// Joins components under explicit resource limits, including output bytes.
    pub fn try_new_with_limits(
        items: Vec<CssComponentValue>,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssComponentValueError> {
        let values = Self::from_items(items, limits)?;
        serialize::validate(&values, limits.max_css_bytes)?;
        Ok(values)
    }

    fn from_items(
        items: Vec<CssComponentValue>,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssComponentValueError> {
        let mut count = items.len();
        let mut depth = 0;
        for item in &items {
            if let Some(children) = item.child_values() {
                count = count.checked_add(children.count).ok_or_else(|| {
                    CssComponentValueError::new(
                        CssComponentValueErrorKind::CapacityOverflow,
                        item.origin().clone(),
                    )
                })?;
                depth = depth.max(children.depth + 1);
            }
            if count > limits.max_components {
                return Err(CssComponentValueError::new(
                    CssComponentValueErrorKind::ComponentLimit,
                    item.origin().clone(),
                ));
            }
            if depth > limits.max_depth {
                return Err(CssComponentValueError::new(
                    CssComponentValueErrorKind::NestingLimit,
                    item.origin().clone(),
                ));
            }
        }
        Ok(Self {
            items: items.into_boxed_slice(),
            count,
            depth,
        })
    }

    /// Returns the complete immutable sequence, including trivia nodes.
    #[must_use]
    pub fn items(&self) -> &[CssComponentValue] {
        &self.items
    }

    pub(crate) fn component_at_path(&self, path: &[usize]) -> Option<&CssComponentValue> {
        let (first, rest) = path.split_first()?;
        let mut component = self.items.get(*first)?;
        for index in rest {
            component = component.child_values()?.items.get(*index)?;
        }
        Some(component)
    }

    pub(crate) fn validate_with_limits(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<(), CssComponentValueError> {
        let mut pending: Vec<_> = self
            .items
            .iter()
            .rev()
            .map(|component| (component, 0))
            .collect();
        let mut count = 0usize;
        while let Some((component, depth)) = pending.pop() {
            count = count.checked_add(1).ok_or_else(|| {
                CssComponentValueError::new(
                    CssComponentValueErrorKind::CapacityOverflow,
                    component.origin().clone(),
                )
            })?;
            if count > limits.max_components {
                return Err(CssComponentValueError::new(
                    CssComponentValueErrorKind::ComponentLimit,
                    component.origin().clone(),
                ));
            }
            if let Some(children) = component.child_values() {
                if depth >= limits.max_depth {
                    return Err(CssComponentValueError::new(
                        CssComponentValueErrorKind::NestingLimit,
                        component.origin().clone(),
                    ));
                }
                pending.extend(children.items.iter().rev().map(|child| (child, depth + 1)));
            }
        }
        serialize::validate(self, limits.max_css_bytes)
    }

    pub(crate) fn first_implicit_origin(&self) -> Option<&CssValueOrigin> {
        let mut pending: Vec<_> = self.items.iter().rev().collect();
        while let Some(component) = pending.pop() {
            let implicit = match &component.data {
                ComponentData::Token(token) => {
                    token.implicit_end.as_ref().map(|lexeme| &lexeme.origin)
                }
                ComponentData::Comment { implicit_end, .. } => {
                    implicit_end.as_ref().map(|lexeme| &lexeme.origin)
                }
                ComponentData::Function(function) => Some(&function.closing.origin),
                ComponentData::Block(block) => Some(&block.closing.origin),
            };
            if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = implicit {
                return Some(origin);
            }
            if let Some(children) = component.child_values() {
                pending.extend(children.items.iter().rev());
            }
        }
        None
    }

    /// Returns the total component count including descendants and trivia.
    #[must_use]
    pub const fn component_count(&self) -> usize {
        self.count
    }

    /// Returns the maximum number of enclosing functions and blocks.
    #[must_use]
    pub const fn nesting_depth(&self) -> u32 {
        self.depth
    }

    /// Serializes complete token representations with only required boundary comments.
    ///
    /// Actual whitespace and comments remain intact. Syntax implied at EOF is
    /// emitted explicitly with its own origin. This is not canonical CSSOM
    /// property serialization and does not validate any property grammar.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }

    /// Serializes while bounding generated UTF-8 bytes, including separators.
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        serialize::serialize(self, max_css_bytes)
    }
}

/// The origin of a generated token, inserted separator, or end-of-input cursor.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssSerializedOrigin {
    /// Bytes emitted from one token or delimiter.
    Token(CssValueOrigin),
    /// A generated comment separating the two adjacent token origins.
    Separator {
        /// Origin of the token before the inserted separator.
        before: CssValueOrigin,
        /// Origin of the token after the inserted separator.
        after: CssValueOrigin,
    },
    /// The generated EOF cursor, anchored to the last emitted origin when any.
    End(Option<CssValueOrigin>),
}

/// One generated UTF-8 byte range and its original provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSerializedOriginSegment {
    range: Range<usize>,
    origin: CssSerializedOrigin,
}

impl CssSerializedOriginSegment {
    /// Returns generated byte offsets, not positions in authored source.
    #[must_use]
    pub fn byte_range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the source or generated-boundary origin of this range.
    #[must_use]
    pub const fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }
}

/// Token-preserving CSS text and a complete map back to originating syntax.
#[derive(Clone, Debug)]
pub struct CssSerializedValue {
    css: String,
    segments: Box<[CssSerializedOriginSegment]>,
    end: CssSerializedOrigin,
    component_paths: Box<[(Range<usize>, Vec<usize>)]>,
}

impl CssSerializedValue {
    pub(crate) fn from_numeric_tokens(
        tokens: Vec<(String, CssValueOrigin)>,
    ) -> Result<Self, CssComponentValueError> {
        let mut css = String::new();
        let mut segments = Vec::new();
        let mut last = None;
        for (text, origin) in tokens {
            let start = css.len();
            let end = start.checked_add(text.len()).ok_or_else(|| {
                CssComponentValueError::new(
                    CssComponentValueErrorKind::CapacityOverflow,
                    origin.clone(),
                )
            })?;
            css.push_str(&text);
            if start != end {
                segments.push(CssSerializedOriginSegment {
                    range: start..end,
                    origin: CssSerializedOrigin::Token(origin.clone()),
                });
            }
            last = Some(origin);
        }
        Ok(Self {
            css,
            segments: segments.into_boxed_slice(),
            end: CssSerializedOrigin::End(last),
            component_paths: Box::new([]),
        })
    }
    pub(crate) fn component_path_at(&self, byte: usize) -> Option<&[usize]> {
        self.component_paths
            .iter()
            .find(|(range, _)| range.start == byte)
            .map(|(_, path)| path.as_slice())
    }
    pub(crate) fn component_offset_for_path(&self, path: &[usize]) -> Option<usize> {
        self.component_paths
            .iter()
            .find(|(_, candidate)| candidate.as_slice() == path)
            .map(|(range, _)| range.start)
    }
    /// Finds complete sibling components, never a token fragment or a reconstructed graph.
    pub(crate) fn component_paths_in_range(&self, range: Range<usize>) -> Option<Vec<&[usize]>> {
        if range.start > range.end || range.end > self.css.len() {
            return None;
        }
        let mut candidates: Vec<_> = self
            .component_paths
            .iter()
            .filter(|(candidate, _)| candidate.start >= range.start && candidate.end <= range.end)
            .collect();
        candidates
            .sort_by_key(|(candidate, _)| (candidate.start, std::cmp::Reverse(candidate.end)));
        let mut selected: Vec<&(Range<usize>, Vec<usize>)> = Vec::new();
        for candidate in candidates {
            if selected
                .last()
                .is_some_and(|previous| candidate.0.end <= previous.0.end)
            {
                continue;
            }
            selected.push(candidate);
        }
        if let Some(first) = selected.first() {
            let parent = &first.1[..first.1.len() - 1];
            for (index, candidate) in selected.iter().enumerate() {
                if candidate.1.len() != first.1.len()
                    || &candidate.1[..candidate.1.len() - 1] != parent
                    || candidate.1.last().copied()?
                        != first.1.last().copied()?.checked_add(index)?
                {
                    return None;
                }
            }
        }
        let mut selected_index = 0;
        for segment in self
            .segments
            .iter()
            .filter(|segment| segment.range.start < range.end && segment.range.end > range.start)
        {
            if matches!(segment.origin, CssSerializedOrigin::Separator { .. }) {
                continue;
            }
            while selected
                .get(selected_index)
                .is_some_and(|(candidate, _)| candidate.end <= segment.range.start)
            {
                selected_index += 1;
            }
            let (candidate, _) = selected.get(selected_index)?;
            if candidate.start > segment.range.start || candidate.end < segment.range.end {
                return None;
            }
        }
        Some(
            selected
                .into_iter()
                .map(|(_, path)| path.as_slice())
                .collect(),
        )
    }

    pub(crate) fn value_origin_at(&self, byte: usize) -> Option<&CssValueOrigin> {
        match self.origin_at(byte)? {
            CssSerializedOrigin::Token(origin)
            | CssSerializedOrigin::Separator { after: origin, .. } => Some(origin),
            CssSerializedOrigin::End(origin) => origin.as_ref(),
        }
    }

    /// Returns the generated token-preserving CSS.
    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.css
    }

    /// Returns contiguous generated byte ranges with their original provenance.
    #[must_use]
    pub fn segments(&self) -> &[CssSerializedOriginSegment] {
        &self.segments
    }

    /// Maps a generated byte offset, including EOF, to its responsible origin.
    #[must_use]
    pub fn origin_at(&self, byte_offset: usize) -> Option<&CssSerializedOrigin> {
        if byte_offset == self.css.len() {
            return Some(&self.end);
        }
        let index = self
            .segments
            .partition_point(|segment| segment.range.end <= byte_offset);
        self.segments
            .get(index)
            .filter(|segment| segment.range.contains(&byte_offset))
            .map(|segment| &segment.origin)
    }
}

/// Parses CSS component values, retaining trivia and closing blocks at EOF.
///
/// Bad string/URL tokens and unmatched closing delimiters return typed failures.
/// This does not apply declaration-value or property-specific restrictions.
pub fn parse_component_values(source: &str) -> Result<CssComponentValues, CssComponentValueError> {
    parse_component_values_with_limits(source, CssComponentValueLimits::default())
}

/// Parses component values with explicit structural, count, and byte limits.
pub fn parse_component_values_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> Result<CssComponentValues, CssComponentValueError> {
    parse::parse(source, limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_input_reports_its_byte_length_without_a_parsed_origin() {
        let limits = CssComponentValueLimits::try_new(1, 3, 3).expect("valid limits");
        let error = parse_component_values_with_limits("😀", limits).expect_err("four UTF-8 bytes");
        assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
        assert_eq!(
            error.origin(),
            &CssValueOrigin::UnretainedInput { byte_length: 4 }
        );
    }

    #[test]
    fn three_adjacent_components_do_not_form_a_cdo_token() {
        let items = ["<", "!", "--"]
            .into_iter()
            .map(|css| CssComponentValue::try_token(css).expect("individual token"))
            .collect();
        let css = CssComponentValues::try_new(items)
            .expect("three tokens")
            .serialize()
            .expect("token-preserving output");
        assert_eq!(css.as_css(), "<!/**/--");
        let parsed = parse_component_values(css.as_css()).expect("three separate tokens");
        assert!(matches!(
            parsed.items()[0].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Delim('<'))
        ));
        assert!(matches!(
            parsed.items()[1].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Delim('!'))
        ));
        assert!(matches!(
            parsed.items()[3].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Ident("--"))
        ));
    }

    #[test]
    fn escaped_eof_string_can_be_followed_by_another_component() {
        let mut items = parse_component_values("\"abc\\")
            .expect("EOF-terminated string")
            .items()
            .to_vec();
        items.push(CssComponentValue::try_ident("x").expect("following identifier"));
        let css = CssComponentValues::try_new(items)
            .expect("joined components")
            .serialize()
            .expect("close string without escaping its new quote");
        assert_eq!(css.as_css(), "\"abc\\\n\"x");
        let parsed = parse_component_values(css.as_css()).expect("closed string and identifier");
        assert!(matches!(
            parsed.items()[0].view(),
            CssComponentValueRef::Token(CssValueTokenRef::String("abc"))
        ));
        assert!(matches!(
            parsed.items()[1].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Ident("x"))
        ));
        assert!(
            matches!(css.origin_at(5), Some(CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { at, .. }))
            if at.span().start().byte_offset().value() == 5)
        );
    }

    #[test]
    fn escaped_eof_identifier_and_url_keep_the_replacement_code_point() {
        for (source, expected) in [("a\\", "a\\fffd "), ("url(a\\", "url(a\\fffd )")] {
            let css = parse_component_values(source)
                .expect("EOF escape")
                .serialize()
                .expect("explicit escape termination");
            assert_eq!(css.as_css(), expected);
            let parsed = parse_component_values(css.as_css()).expect("terminated token");
            assert!(matches!(
                parsed.items()[0].view(),
                CssComponentValueRef::Token(
                    CssValueTokenRef::Ident("a\u{fffd}") | CssValueTokenRef::Url("a\u{fffd}")
                )
            ));
        }
    }

    #[test]
    fn eof_comment_closure_preserves_its_last_asterisk() {
        let css = parse_component_values("/*a*")
            .expect("EOF-terminated comment")
            .serialize()
            .expect("closed comment");
        assert_eq!(css.as_css(), "/*a**/");
        let parsed = parse_component_values(css.as_css()).expect("complete comment");
        assert!(matches!(
            parsed.items()[0].view(),
            CssComponentValueRef::Comment("a*")
        ));
        assert!(
            matches!(css.origin_at(4), Some(CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { at, .. }))
            if at.span().start().byte_offset().value() == 4)
        );
    }

    #[test]
    fn source_equality_compares_text_while_snapshot_identity_remains_explicit() {
        let first = parse_component_values("a").expect("first parse");
        let second = parse_component_values("a").expect("independent second parse");
        let CssValueOrigin::Parsed(first) = first.items()[0].origin() else {
            panic!("parsed origin")
        };
        let CssValueOrigin::Parsed(second) = second.items()[0].origin() else {
            panic!("parsed origin")
        };
        assert_eq!(first.source(), second.source());
        assert_eq!(first, second);
        assert!(!first.source().same_snapshot(second.source()));
        assert!(first.source().same_snapshot(&first.source().clone()));
    }
}
