use crate::span::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedEof,
    UnexpectedToken { expected: &'static str },
    UnclosedElement { name: String },
    MismatchedCloseTag { expected: String, found: String },
    InvalidName { name: String },
    InvalidExpression { reason: &'static str },
    StrayTemplateTag { tag: String },
    UnsupportedTemplateTag { tag: String },
}

impl core::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected end of template source"),
            Self::UnexpectedToken { expected } => {
                write!(f, "unexpected token, expected {expected}")
            }
            Self::UnclosedElement { name } => {
                write!(f, "element \"{}\" is not closed", name.escape_debug())
            }
            Self::MismatchedCloseTag { expected, found } => write!(
                f,
                "closing tag \"{}\" does not match open element \"{}\"",
                found.escape_debug(),
                expected.escape_debug()
            ),
            Self::InvalidName { name } => write!(
                f,
                "\"{}\" is not a valid element or attribute name",
                name.escape_debug()
            ),
            Self::InvalidExpression { reason } => write!(f, "invalid expression: {reason}"),
            Self::StrayTemplateTag { tag } => write!(
                f,
                "template tag \"{}\" has no matching opening tag",
                tag.escape_debug()
            ),
            Self::UnsupportedTemplateTag { tag } => {
                write!(
                    f,
                    "template tag \"{}\" is not supported",
                    tag.escape_debug()
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: SourceSpan,
}

impl ParseError {
    #[allow(dead_code)]
    pub(crate) const fn new(kind: ParseErrorKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} at ", self.kind)?;
        self.span.write_start(f)
    }
}

impl core::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    UnknownNativeElement { name: String },
    UnknownComponent { name: String },
    DuplicateAttribute { name: String },
    InvalidAttribute { element: String, attribute: String },
    InvalidAttributeValue { element: String, attribute: String },
}

impl core::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownNativeElement { name } => {
                write!(f, "unknown native element \"{}\"", name.escape_debug())
            }
            Self::UnknownComponent { name } => {
                write!(f, "unknown component \"{}\"", name.escape_debug())
            }
            Self::DuplicateAttribute { name } => write!(
                f,
                "attribute \"{}\" appears more than once on one element",
                name.escape_debug()
            ),
            Self::InvalidAttribute { element, attribute } => write!(
                f,
                "attribute \"{}\" is not allowed on \"{}\"",
                attribute.escape_debug(),
                element.escape_debug()
            ),
            Self::InvalidAttributeValue { element, attribute } => write!(
                f,
                "attribute \"{}\" on \"{}\" does not accept this value form",
                attribute.escape_debug(),
                element.escape_debug()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    kind: ValidationErrorKind,
    span: SourceSpan,
}

impl ValidationError {
    #[allow(dead_code)]
    pub(crate) const fn new(kind: ValidationErrorKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &ValidationErrorKind {
        &self.kind
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} at ", self.kind)?;
        self.span.write_start(f)
    }
}

impl core::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use crate::span::{SourcePos, SourceSpan};

    use super::{ParseError, ParseErrorKind};

    #[test]
    fn parse_error_carries_kind_and_span() {
        let span = SourceSpan::new_unchecked(
            SourcePos::new_unchecked(1, 1, 0),
            SourcePos::new_unchecked(1, 5, 4),
        );
        let error = ParseError::new(
            ParseErrorKind::UnexpectedToken {
                expected: "tag name",
            },
            span,
        );

        assert!(matches!(
            error.kind(),
            ParseErrorKind::UnexpectedToken {
                expected: "tag name"
            }
        ));
        assert_eq!(error.span(), span);
    }
}
