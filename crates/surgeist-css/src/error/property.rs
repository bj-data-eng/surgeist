//! Property grammar errors retain input bookmarks until the public boundary.
use super::source_resolution::{ends_input, precedes_input};
use super::*;
pub(crate) use super::{CssFeatureId, ErrorKind};
use crate::parser::property_input::Site;

#[derive(Clone, Debug)]
pub(crate) struct Error {
    pub(crate) kind: ErrorKind,
}
#[derive(Clone, Debug)]
pub(crate) enum ParseErrorKind<'i, E> {
    Basic(BasicParseErrorKind<'i>),
    Custom(Box<E>),
}
#[derive(Clone, Debug)]
pub(crate) struct ParseError<'i, E> {
    pub(crate) kind: ParseErrorKind<'i, E>,
    pub(crate) location: Site<'i>,
}
#[derive(Clone, Debug)]
pub(crate) struct BasicParseError<'i> {
    pub(crate) kind: BasicParseErrorKind<'i>,
    pub(crate) location: Site<'i>,
}
impl<'i, E> From<BasicParseError<'i>> for ParseError<'i, E> {
    fn from(error: BasicParseError<'i>) -> Self {
        Self {
            kind: ParseErrorKind::Basic(error.kind),
            location: error.location,
        }
    }
}
impl<'i> ParseError<'i, Error> {
    /// Qualify diagnostics from actual input bookmarks before publishing them.
    pub(crate) fn qualify(self) -> Self {
        if let Some(control) = self.location.control() {
            let location = self.location;
            return self
                .qualify_controlled(control)
                .unwrap_or_else(|()| location.new_error(BasicParseErrorKind::EndOfInput));
        }
        let mut error = match self.kind {
            ParseErrorKind::Custom(error) => error,
            ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(token)) => Box::new(Error {
                kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                    expectation: EXPECT_CSS_SYNTAX,
                    encountered: CssTokenSummary::from_token(&token),
                }),
            }),
            ParseErrorKind::Basic(_) => Box::new(Error {
                kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                    expectation: EXPECT_CSS_SYNTAX,
                }),
            }),
        };
        let mut location = self.location;
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
            kind: ParseErrorKind::Custom(error),
            location,
        }
    }
    pub(crate) fn into_parts_controlled(
        self,
        control: &crate::utf16_value::Control<'_>,
    ) -> Result<(ErrorKind, crate::CssSerializedOrigin), ()> {
        let error = self.qualify_controlled(control)?;
        control.step()?;
        let ParseErrorKind::Custom(detail) = error.kind else {
            unreachable!("admitted qualified property diagnostic")
        };
        let Error { kind } = *detail;
        control.released(std::mem::size_of::<Error>());
        let origin = match &kind {
            ErrorKind::InvalidComponentValue(detail) => {
                crate::CssSerializedOrigin::Token(detail.origin().clone())
            }
            _ => error.location.origin(),
        };
        if control.failure().is_some() {
            return Err(());
        }
        Ok((kind, origin))
    }
    fn qualify_controlled(self, control: &crate::utf16_value::Control<'_>) -> Result<Self, ()> {
        control.step()?;
        let mut error = match self.kind {
            ParseErrorKind::Custom(error) => error,
            ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(_token)) => {
                let encountered = summary_at_controlled(self.location, control)?;
                control.add(std::mem::size_of::<Error>())?;
                Box::new(Error {
                    kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                        expectation: EXPECT_CSS_SYNTAX,
                        encountered,
                    }),
                })
            }
            ParseErrorKind::Basic(_) => {
                control.add(std::mem::size_of::<Error>())?;
                Box::new(Error {
                    kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                        expectation: EXPECT_CSS_SYNTAX,
                    }),
                })
            }
        };
        let mut location = self.location;
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
            kind: ParseErrorKind::Custom(error),
            location,
        })
    }
    pub(crate) fn into_parts(self) -> (ErrorKind, crate::CssSerializedOrigin) {
        let error = self.qualify();
        let ParseErrorKind::Custom(detail) = error.kind else {
            unreachable!("qualified property errors own their semantic kind");
        };
        let Error { kind } = *detail;
        let origin = match &kind {
            ErrorKind::InvalidComponentValue(detail) => {
                crate::CssSerializedOrigin::Token(detail.origin().clone())
            }
            _ => error.location.origin(),
        };
        (kind, origin)
    }
    pub(crate) fn into_native_at<'s>(
        self,
        source: &str,
        offset: usize,
    ) -> cssparser::ParseError<'s, super::CssError> {
        let (kind, _) = self.into_parts();
        let position = CssSourcePosition::from_byte_offset_in(source, offset);
        let location = cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value().saturating_add(1),
        };
        cssparser::ParseError {
            kind: cssparser::ParseErrorKind::Custom(super::CssError { position, kind }),
            location,
        }
    }
    pub(crate) fn into_native_source<'s>(self) -> cssparser::ParseError<'s, super::CssError> {
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
    pub(crate) fn into_source_error(self) -> super::CssError {
        let error = self.qualify();
        let site = error.location;
        let (kind, origin) = error.into_parts();
        let position = match origin {
            crate::CssSerializedOrigin::Token(crate::CssValueOrigin::Parsed(origin)) => {
                origin.span().start()
            }
            crate::CssSerializedOrigin::Token(crate::CssValueOrigin::ImplicitClosure {
                at,
                ..
            }) => at.span().start(),
            _ => site.source_position().expect("source property input"),
        };
        super::CssError { position, kind }
    }
}
pub(crate) fn basic<'i>(error: BasicParseError<'i>) -> ParseError<'i, Error> {
    error.into()
}
pub(crate) fn error_at<'i>(location: Site<'i>, kind: ErrorKind) -> ParseError<'i, Error> {
    location.new_custom_error(Error { kind })
}
pub(crate) fn unexpected_at<'i>(location: Site<'i>) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_DECLARATION_VALUE,
        }),
    )
}
pub(crate) fn invalid_syntax<'i>(location: Site<'i>) -> ParseError<'i, Error> {
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
    detail: crate::CssComponentValueError,
) -> ParseError<'i, Error> {
    if location.control().is_some_and(|control| {
        control
            .add(std::mem::size_of::<crate::CssComponentValueError>())
            .is_err()
    }) {
        return location.new_error(BasicParseErrorKind::EndOfInput);
    }
    error_at(location, ErrorKind::InvalidComponentValue(Box::new(detail)))
}
pub(crate) fn invalid_color<'i>(
    location: Site<'i>,
    component: Option<&str>,
) -> ParseError<'i, Error> {
    error_at(location, super::color_syntax_kind(component))
}
fn encountered(kind: &mut ParseErrorKind<'_, Error>) -> Option<CssTokenSummary> {
    match kind {
        ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(token)) => {
            Some(CssTokenSummary::from_token(token))
        }
        ParseErrorKind::Custom(error) => encountered_mut(&mut error.kind).cloned(),
        _ => None,
    }
}
pub(crate) fn with_property_context<'i>(
    mut error: ParseError<'i, Error>,
    property: &str,
) -> ParseError<'i, Error> {
    if let Some(control) = error.location.control() {
        let location = error.location;
        return with_property_context_controlled(error, property, control)
            .unwrap_or_else(|()| location.new_error(BasicParseErrorKind::EndOfInput));
    }
    let Some(property) = property_for_supported_name(property) else {
        return error;
    };
    if let ParseErrorKind::Custom(detail) = &error.kind
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
        ParseErrorKind::Custom(error) => error.kind = kind,
        ParseErrorKind::Basic(_) => error.kind = ParseErrorKind::Custom(Box::new(Error { kind })),
    }
    error
}

pub(crate) fn is_resource_parse_error(error: &ParseError<'_, Error>) -> bool {
    matches!(&error.kind, ParseErrorKind::Custom(error)
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
    component: Option<&crate::CssComponentValue>,
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
    control: &crate::utf16_value::Control<'_>,
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
    mut error: ParseError<'i, Error>,
    property: &str,
    control: &crate::utf16_value::Control<'_>,
) -> Result<ParseError<'i, Error>, ()> {
    control.step()?;
    let Some(property) = property_for_supported_name(property) else {
        return Ok(error);
    };
    if let ParseErrorKind::Custom(detail) = &error.kind
        && (matches!(&detail.kind, ErrorKind::InvalidColorSyntax(_))
            || matches!(&detail.kind, ErrorKind::InvalidComponentValue(detail) if is_component_resource_error(detail)))
    {
        return Ok(error);
    }
    let encountered = match &mut error.kind {
        ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(_token)) => {
            Some(summary_at_controlled(error.location, control)?)
        }
        ParseErrorKind::Custom(detail) => encountered_mut(&mut detail.kind)
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
        ParseErrorKind::Custom(detail) => {
            let previous_bytes = detail
                .kind
                .retained_backing_bytes_with(&mut || control.step())
                .map_err(|error| {
                    if error.is_capacity_overflow() {
                        let _ = control.fail::<()>(crate::utf16_value::Failure::CapacityOverflow);
                    }
                })?;
            let previous = std::mem::replace(&mut detail.kind, kind);
            drop(previous);
            control.released(previous_bytes);
        }
        ParseErrorKind::Basic(_) => {
            control.add(std::mem::size_of::<Error>())?;
            error.kind = ParseErrorKind::Custom(Box::new(Error { kind }));
        }
    }
    Ok(error)
}

fn checked_kind_at(site: Site<'_>) -> Option<CssTokenKind> {
    let (component, closing) = site.checked_component()?;
    Some(if closing {
        match component.view() {
            crate::CssComponentValueRef::Function(_) => CssTokenKind::CloseParenthesis,
            crate::CssComponentValueRef::Block(block) => match block.kind() {
                crate::CssBlockKind::Parenthesis => CssTokenKind::CloseParenthesis,
                crate::CssBlockKind::SquareBracket => CssTokenKind::CloseSquareBracket,
                crate::CssBlockKind::CurlyBracket => CssTokenKind::CloseCurlyBracket,
            },
            _ => return None,
        }
    } else {
        CssTokenSummary::kind_from_component(component)
    })
}
fn summary_at_controlled(
    site: Site<'_>,
    control: &crate::utf16_value::Control<'_>,
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
    use crate::parser::property_input::Parser;
    use crate::syntax_consumption::{SyntaxInput, SyntaxInputLimits, normalize, source_document};
    use crate::{
        CssComponentValue, CssComponentValues, CssSerializedOrigin, CssSourceSnapshot,
        CssValueOrigin,
    };

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
        let mut input = Parser::new(&document);
        let (kind, origin) = basic(input.expect_ident_matching("auto").unwrap_err()).into_parts();
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
        let mut input = Parser::new(&document);
        input.expect_ident_matching("ok").unwrap();
        let error = basic(input.expect_ident_matching("other").unwrap_err()).into_source_error();
        assert_eq!(error.position().byte_offset().value(), 6);
        assert_eq!(error.position().line().value(), 1);
        assert_eq!(error.position().column().value(), 2);
    }

    #[test]
    fn checked_origins_never_enable_source_reverse_lookup() {
        let values = crate::parse_component_values("a b").unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let input = Parser::new(&document);
        assert!(
            input
                .position()
                .source_origin(values.items()[2].origin())
                .is_none()
        );
    }
}
