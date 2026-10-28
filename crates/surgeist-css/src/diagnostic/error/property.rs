//! Property grammar errors retain input bookmarks until the public boundary.
use super::ErrorKind;
use super::source_resolution::{ends_input, precedes_input};
use crate::diagnostic::error::{
    CssQualifiedRuleError, CssUnexpectedEndError, CssUnexpectedTokenError, EXPECT_CSS_SYNTAX,
    EXPECT_DECLARATION_VALUE, EXPECT_PROPERTY_VALUE, PropertyValueRejection, QUALIFIED_RULE,
    encountered_mut, is_component_resource_error, optional_encountered_mut, token_kind,
};
use crate::diagnostic::{CssGrammarExpectation, CssTokenKind, CssTokenSummary};
use crate::parser::property_input::Site;
use crate::source::CssSourcePosition;
use crate::validation::property_for_supported_name;
use cssparser::Token;

#[derive(Clone, Debug)]
pub(crate) struct Error {
    pub(crate) kind: ErrorKind,
}
/// The token stream could not supply what the grammar asked for.
#[derive(Clone, Debug)]
pub(crate) struct StreamError<'i> {
    at: Site<'i>,
    kind: StreamErrorKind<'i>,
}
#[derive(Clone, Debug)]
enum StreamErrorKind<'i> {
    UnexpectedToken(Token<'i>),
    EndOfStream,
}
impl<'i> StreamError<'i> {
    /// The stream held `token` where the grammar needed another.
    pub(crate) fn unexpected(at: Site<'i>, token: Token<'i>) -> Self {
        Self {
            at,
            kind: StreamErrorKind::UnexpectedToken(token),
        }
    }
    /// The stream was empty where the grammar needed a token.
    pub(crate) fn end_of_stream(at: Site<'i>) -> Self {
        Self {
            at,
            kind: StreamErrorKind::EndOfStream,
        }
    }
    /// Whether the stream was empty where the grammar needed a token.
    pub(crate) fn is_end_of_stream(&self) -> bool {
        matches!(self.kind, StreamErrorKind::EndOfStream)
    }
}
/// A grammar failure: a stream failure or the grammar's own error `E`.
#[derive(Clone, Debug)]
pub(crate) struct SyntaxError<'i, E> {
    at: Site<'i>,
    kind: SyntaxErrorKind<'i, E>,
}
#[derive(Clone, Debug)]
enum SyntaxErrorKind<'i, E> {
    Stream(StreamErrorKind<'i>),
    Custom(Box<E>),
}
impl<'i, E> From<StreamError<'i>> for SyntaxError<'i, E> {
    fn from(error: StreamError<'i>) -> Self {
        Self {
            at: error.at,
            kind: SyntaxErrorKind::Stream(error.kind),
        }
    }
}
impl<'i, E> SyntaxError<'i, E> {
    /// The boundary the failure is reported at.
    pub(crate) fn at(&self) -> Site<'i> {
        self.at
    }
    /// The grammar's own error `error`, reported at `at`.
    pub(crate) fn custom(at: Site<'i>, error: E) -> Self {
        Self {
            at,
            kind: SyntaxErrorKind::Custom(Box::new(error)),
        }
    }
    /// The grammar's own error, if the failure is one.
    pub(crate) fn custom_value(&self) -> Option<&E> {
        match &self.kind {
            SyntaxErrorKind::Custom(error) => Some(error),
            SyntaxErrorKind::Stream(_) => None,
        }
    }
}
impl<'i> SyntaxError<'i, Error> {
    /// Qualify diagnostics from actual input bookmarks before publishing them.
    pub(crate) fn qualify(self) -> Self {
        if let Some(control) = self.at.control() {
            let location = self.at;
            return self
                .qualify_controlled(control)
                .unwrap_or_else(|()| location.end_of_stream());
        }
        let mut error = match self.kind {
            SyntaxErrorKind::Custom(error) => error,
            SyntaxErrorKind::Stream(StreamErrorKind::UnexpectedToken(token)) => Box::new(Error {
                kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                    expectation: EXPECT_CSS_SYNTAX,
                    encountered: CssTokenSummary::from_token(&token),
                }),
            }),
            SyntaxErrorKind::Stream(_) => Box::new(Error {
                kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                    expectation: EXPECT_CSS_SYNTAX,
                }),
            }),
        };
        let mut location = self.at;
        let (next, previous) = location.diagnostic_candidates();
        if let Some(encountered) = encountered_mut(&mut error.kind) {
            if let Some(site) = next
                && let Some(token) = site.token()
                && token_kind(&token) == encountered.kind
            {
                if let Some(text) = site.lexeme()
                    && encountered.authored != text
                {
                    encountered.authored = text.to_owned();
                }
                location = site;
            }
        } else if let Some(slot) = optional_encountered_mut(&mut error.kind) {
            // The crate's location rule: the next token, including an
            // explicit closing `)`, is responsible; at the end of the value or
            // of an implicitly closed group, the last authored token is.
            let candidate = next
                .and_then(|site| {
                    let token = site.token()?;
                    (!ends_input(token_kind(&token))).then_some((site, token))
                })
                .or_else(|| {
                    previous.and_then(|site| {
                        let token = site.token()?;
                        (!precedes_input(token_kind(&token))).then_some((site, token))
                    })
                });
            if let Some((site, token)) = candidate {
                let mut summary = CssTokenSummary::from_token(&token);
                if let Some(text) = site.lexeme() {
                    summary.authored = text.to_owned();
                }
                *slot = Some(summary);
                location = site;
            }
        }
        Self {
            kind: SyntaxErrorKind::Custom(error),
            at: location,
        }
    }
    pub(crate) fn into_parts_controlled(
        self,
        control: &crate::values::color_adjust::utf16_value::Control<'_>,
    ) -> Result<(ErrorKind, crate::component::CssSerializedOrigin), ()> {
        let error = self.qualify_controlled(control)?;
        control.step()?;
        let SyntaxErrorKind::Custom(detail) = error.kind else {
            unreachable!("admitted qualified property diagnostic")
        };
        let Error { kind } = *detail;
        control.released(std::mem::size_of::<Error>());
        let origin = match &kind {
            ErrorKind::InvalidComponentValue(detail) => {
                crate::component::CssSerializedOrigin::Token(detail.origin().clone())
            }
            _ => error.at.origin(),
        };
        if control.failure().is_some() {
            return Err(());
        }
        Ok((kind, origin))
    }
    fn qualify_controlled(
        self,
        control: &crate::values::color_adjust::utf16_value::Control<'_>,
    ) -> Result<Self, ()> {
        control.step()?;
        let mut error = match self.kind {
            SyntaxErrorKind::Custom(error) => error,
            SyntaxErrorKind::Stream(StreamErrorKind::UnexpectedToken(_token)) => {
                let encountered = summary_at_controlled(self.at, control)?;
                control.add(std::mem::size_of::<Error>())?;
                Box::new(Error {
                    kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                        expectation: EXPECT_CSS_SYNTAX,
                        encountered,
                    }),
                })
            }
            SyntaxErrorKind::Stream(_) => {
                control.add(std::mem::size_of::<Error>())?;
                Box::new(Error {
                    kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                        expectation: EXPECT_CSS_SYNTAX,
                    }),
                })
            }
        };
        let mut location = self.at;
        let (next, previous) = location.diagnostic_candidates();
        control.step()?;
        if let Some(encountered) = encountered_mut(&mut error.kind) {
            if let Some(site) = next
                && checked_kind_at(site) == Some(encountered.kind)
            {
                if let Some(text) = site.lexeme()
                    && !same_text_controlled(&encountered.authored, text, control)?
                {
                    let next = control.string(text)?;
                    let old = std::mem::replace(&mut encountered.authored, next);
                    let bytes = old.capacity();
                    drop(old);
                    control.released(bytes);
                }
                location = site;
            }
        } else if let Some(slot) = optional_encountered_mut(&mut error.kind) {
            // See the uncontrolled path for the location rule.
            let candidate = next
                .filter(|site| {
                    site.token().is_some()
                        && checked_kind_at(*site).is_some_and(|kind| !ends_input(kind))
                })
                .or_else(|| {
                    previous.filter(|site| {
                        checked_kind_at(*site).is_some_and(|kind| !precedes_input(kind))
                    })
                });
            if let Some(site) = candidate {
                let summary = summary_at_controlled(site, control)?;
                let previous = slot.replace(summary);
                let bytes = previous
                    .as_ref()
                    .map_or(0, |value| value.authored.capacity());
                drop(previous);
                control.released(bytes);
                location = site;
            }
        }
        control.step()?;
        Ok(Self {
            kind: SyntaxErrorKind::Custom(error),
            at: location,
        })
    }
    pub(crate) fn into_parts(self) -> (ErrorKind, crate::component::CssSerializedOrigin) {
        let error = self.qualify();
        let SyntaxErrorKind::Custom(detail) = error.kind else {
            unreachable!("qualified property errors own their semantic kind");
        };
        let Error { kind } = *detail;
        let origin = match &kind {
            ErrorKind::InvalidComponentValue(detail) => {
                crate::component::CssSerializedOrigin::Token(detail.origin().clone())
            }
            _ => error.at.origin(),
        };
        (kind, origin)
    }
    pub(crate) fn into_native_at<'s>(
        self,
        source: &str,
        offset: usize,
    ) -> cssparser::ParseError<'s, crate::diagnostic::CssError> {
        let (kind, _) = self.into_parts();
        let position = CssSourcePosition::from_byte_offset_in(source, offset);
        let location = cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value().saturating_add(1),
        };
        cssparser::ParseError {
            kind: cssparser::ParseErrorKind::Custom(crate::diagnostic::CssError { position, kind }),
            location,
        }
    }
    pub(crate) fn into_native_source<'s>(
        self,
    ) -> cssparser::ParseError<'s, crate::diagnostic::CssError> {
        let error = self.into_source_error();
        let position = error.position();
        let location = cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value().saturating_add(1),
        };
        cssparser::ParseError {
            kind: cssparser::ParseErrorKind::Custom(error),
            location,
        }
    }
    /// Only source-owned callers may ask for a single source diagnostic.
    pub(crate) fn into_source_error(self) -> crate::diagnostic::CssError {
        let error = self.qualify();
        let site = error.at;
        let (kind, origin) = error.into_parts();
        let position = match origin {
            crate::component::CssSerializedOrigin::Token(
                crate::component::CssValueOrigin::Parsed(origin),
            ) => origin.span().start(),
            crate::component::CssSerializedOrigin::Token(
                crate::component::CssValueOrigin::ImplicitClosure { at, .. },
            ) => at.span().start(),
            _ => site.source_position().expect("source property input"),
        };
        crate::diagnostic::CssError { position, kind }
    }
}
pub(crate) fn basic<'i>(error: StreamError<'i>) -> SyntaxError<'i, Error> {
    error.into()
}
pub(crate) fn error_at<'i>(location: Site<'i>, kind: ErrorKind) -> SyntaxError<'i, Error> {
    location.grammar_error(Error { kind })
}
pub(crate) fn unexpected_at<'i>(location: Site<'i>) -> SyntaxError<'i, Error> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_DECLARATION_VALUE,
        }),
    )
}
pub(crate) fn invalid_syntax<'i>(location: Site<'i>) -> SyntaxError<'i, Error> {
    error_at(
        location,
        ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
            production: QUALIFIED_RULE,
            expectation: EXPECT_CSS_SYNTAX,
            encountered: None,
        }),
    )
}
pub(crate) fn invalid_component_value<'i>(
    location: Site<'i>,
    detail: crate::component::CssComponentValueError,
) -> SyntaxError<'i, Error> {
    if location.control().is_some_and(|control| {
        control
            .add(std::mem::size_of::<crate::component::CssComponentValueError>())
            .is_err()
    }) {
        return location.end_of_stream();
    }
    error_at(location, ErrorKind::InvalidComponentValue(Box::new(detail)))
}
pub(crate) fn invalid_color<'i>(
    location: Site<'i>,
    component: Option<&str>,
) -> SyntaxError<'i, Error> {
    error_at(location, super::color_syntax_kind(component))
}
fn encountered(kind: &mut SyntaxErrorKind<'_, Error>) -> Option<CssTokenSummary> {
    match kind {
        SyntaxErrorKind::Stream(StreamErrorKind::UnexpectedToken(token)) => {
            Some(CssTokenSummary::from_token(token))
        }
        SyntaxErrorKind::Custom(error) => encountered_mut(&mut error.kind).cloned(),
        _ => None,
    }
}
pub(crate) fn with_property_context<'i>(
    mut error: SyntaxError<'i, Error>,
    property: &str,
) -> SyntaxError<'i, Error> {
    if let Some(control) = error.at.control() {
        let location = error.at;
        return with_property_context_controlled(error, property, control)
            .unwrap_or_else(|()| location.end_of_stream());
    }
    let Some(property) = property_for_supported_name(property) else {
        return error;
    };
    if let Some(detail) = error.custom_value()
        && (matches!(&detail.kind, ErrorKind::InvalidColorSyntax(_))
            || matches!(&detail.kind, ErrorKind::InvalidComponentValue(detail) if is_component_resource_error(detail)))
    {
        return error;
    }
    let encountered = encountered(&mut error.kind);
    let kind = ErrorKind::InvalidPropertyValue(PropertyValueRejection {
        property,
        expectation: EXPECT_PROPERTY_VALUE,
        encountered,
    });
    match &mut error.kind {
        SyntaxErrorKind::Custom(error) => error.kind = kind,
        SyntaxErrorKind::Stream(_) => {
            error.kind = SyntaxErrorKind::Custom(Box::new(Error { kind }))
        }
    }
    error
}

pub(crate) fn is_resource_parse_error(error: &SyntaxError<'_, Error>) -> bool {
    matches!(error.custom_value(), Some(error)
        if matches!(&error.kind, ErrorKind::NestingLimit(_))
            || matches!(&error.kind, ErrorKind::InvalidComponentValue(detail) if is_component_resource_error(detail)))
}

pub(crate) fn implicit_eof_kind() -> ErrorKind {
    ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
        expectation: EXPECT_CSS_SYNTAX,
    })
}

/// Construct a grammar rejection without inventing a source coordinate.
pub(crate) fn property_rejection_kind(
    name: &str,
    component: Option<&crate::component::CssComponentValue>,
    expectation: &'static str,
) -> ErrorKind {
    let encountered = component.map(CssTokenSummary::from_component);
    if let Some(property) = property_for_supported_name(name) {
        return ErrorKind::InvalidPropertyValue(PropertyValueRejection {
            property,
            expectation: EXPECT_PROPERTY_VALUE,
            encountered,
        });
    }
    match encountered {
        Some(encountered) => ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
            expectation: CssGrammarExpectation::new(expectation),
            encountered,
        }),
        None => ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: CssGrammarExpectation::new(expectation),
        }),
    }
}

fn same_text_controlled(
    a: &str,
    b: &str,
    control: &crate::values::color_adjust::utf16_value::Control<'_>,
) -> Result<bool, ()> {
    control.step()?;
    if a.len() != b.len() {
        return Ok(false);
    }
    for index in 0..a.len() {
        control.step()?;
        let left = a.as_bytes()[index];
        control.step()?;
        if left != b.as_bytes()[index] {
            return Ok(false);
        }
    }
    Ok(true)
}
fn with_property_context_controlled<'i>(
    mut error: SyntaxError<'i, Error>,
    property: &str,
    control: &crate::values::color_adjust::utf16_value::Control<'_>,
) -> Result<SyntaxError<'i, Error>, ()> {
    control.step()?;
    let Some(property) = property_for_supported_name(property) else {
        return Ok(error);
    };
    if let Some(detail) = error.custom_value()
        && (matches!(&detail.kind, ErrorKind::InvalidColorSyntax(_))
            || matches!(&detail.kind, ErrorKind::InvalidComponentValue(detail) if is_component_resource_error(detail)))
    {
        return Ok(error);
    }
    let encountered = match &mut error.kind {
        SyntaxErrorKind::Stream(StreamErrorKind::UnexpectedToken(_token)) => {
            Some(summary_at_controlled(error.at, control)?)
        }
        SyntaxErrorKind::Custom(detail) => encountered_mut(&mut detail.kind)
            .map(|value| value.clone_controlled(control))
            .transpose()?,
        _ => None,
    };
    let kind = ErrorKind::InvalidPropertyValue(PropertyValueRejection {
        property,
        expectation: EXPECT_PROPERTY_VALUE,
        encountered,
    });
    match &mut error.kind {
        SyntaxErrorKind::Custom(detail) => {
            let previous_bytes = detail
                .kind
                .retained_backing_bytes_with(&mut || control.step())
                .map_err(|error| {
                    if error.is_capacity_overflow() {
                        let _ = control.fail::<()>(
                            crate::values::color_adjust::utf16_value::Failure::CapacityOverflow,
                        );
                    }
                })?;
            let previous = std::mem::replace(&mut detail.kind, kind);
            drop(previous);
            control.released(previous_bytes);
        }
        SyntaxErrorKind::Stream(_) => {
            control.add(std::mem::size_of::<Error>())?;
            error.kind = SyntaxErrorKind::Custom(Box::new(Error { kind }));
        }
    }
    Ok(error)
}

fn checked_kind_at(site: Site<'_>) -> Option<CssTokenKind> {
    let (component, closing) = site.checked_component()?;
    Some(if closing {
        match component.view() {
            crate::component::CssComponentValueRef::Function(_) => CssTokenKind::CloseParenthesis,
            crate::component::CssComponentValueRef::Block(block) => match block.kind() {
                crate::component::CssBlockKind::Parenthesis => CssTokenKind::CloseParenthesis,
                crate::component::CssBlockKind::SquareBracket => CssTokenKind::CloseSquareBracket,
                crate::component::CssBlockKind::CurlyBracket => CssTokenKind::CloseCurlyBracket,
            },
            _ => return None,
        }
    } else {
        CssTokenSummary::kind_from_component(component)
    })
}
fn summary_at_controlled(
    site: Site<'_>,
    control: &crate::values::color_adjust::utf16_value::Control<'_>,
) -> Result<CssTokenSummary, ()> {
    control.step()?;
    let Some((component, closing)) = site.checked_component() else {
        unreachable!("controlled diagnostic borrows actual checked component")
    };
    let Some(text) = site.lexeme() else {
        unreachable!("controlled diagnostic retains exact lexical source")
    };
    CssTokenSummary::from_checked_controlled(component, closing, text, control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{
        CssComponentValue, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
    };
    use crate::parser::property_input::Stream;
    use crate::source::CssSourceSnapshot;
    use crate::syntax_consumption::{SyntaxInput, SyntaxInputLimits, normalize, source_document};

    #[test]
    fn checked_numeric_error_retains_exact_lexeme_without_float_rounding() {
        let spelling = "123456789012345678901234567890123456789";
        let values = CssComponentValues::try_new(
            vec![CssComponentValue::try_number(spelling).unwrap()],
            Default::default(),
        )
        .unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut input = Stream::new(&document);
        let (kind, origin) = basic(input.consume_keyword("auto").unwrap_err()).into_parts();
        let ErrorKind::UnexpectedToken(detail) = kind else {
            panic!("unexpected lexical number");
        };
        assert_eq!(detail.encountered().authored(), spelling);
        assert_eq!(
            origin,
            CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
        );
    }

    #[test]
    fn source_error_uses_original_scalar_and_utf16_position() {
        let snapshot = CssSourceSnapshot::new("ok\r\n  😀");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Stream::new(&document);
        input.consume_keyword("ok").unwrap();
        let error = basic(input.consume_keyword("other").unwrap_err()).into_source_error();
        assert_eq!(error.position().byte_offset().value(), 6);
        assert_eq!(error.position().line().value(), 1);
        assert_eq!(error.position().column().value(), 2);
    }

    #[test]
    fn checked_origins_never_enable_source_reverse_lookup() {
        let values = crate::component::values::parse_component_values("a b").unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let input = Stream::new(&document);
        assert!(
            input
                .site()
                .source_origin(values.items()[2].origin())
                .is_none()
        );
    }
}
