//! Checked property-value construction and diagnostics mapped to input components.

use std::fmt;

use crate::{
    CssColorComponentName, CssComponentValueError, CssComponentValueErrorKind, CssComponentValues,
    CssDeclaration, CssDeclarationContextRef, CssErrorCode, CssGrammarExpectation, CssImportance,
    CssKnownProperty, CssProductionId, CssPropertyNameRef, CssSerializedOrigin, CssTokenSummary,
    ErrorKind,
};

/// A component invariant or property grammar rejected by checked value construction.
#[derive(Clone, Debug, Eq, PartialEq)]
enum PropertyValueFailure {
    /// Supplied components exceeded an intrinsic syntax or resource bound; the
    /// error is located at the responsible original token.
    Component(CssComponentValueError),
    /// The declaration-value boundary or selected property grammar rejected the input.
    Grammar {
        kind: ErrorKind,
        origin: CssSerializedOrigin,
    },
}

/// A property-value failure located in the original component input.
///
/// Either the supplied components could not be admitted
/// ([`Self::component_error`]) or the selected property grammar rejected them, in which
/// case [`Self::code`] and the other fact accessors report the grammar diagnostic as a
/// [`crate::CssError`] would, without an authored position. Unlike a stylesheet
/// diagnostic, this failure can originate in programmatic or mixed-source tokens, so it is
/// located by [`Self::origin`] and never presents generated serialization coordinates as
/// authored source positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPropertyValueError {
    kind: PropertyValueErrorKind,
}

// The UTF-16 property front retains this error inline in its report, so the
// admission case stays unboxed; the checked pipeline keeps its boxed failure.
#[derive(Clone, Debug, Eq, PartialEq)]
enum PropertyValueErrorKind {
    Admission(CssComponentValueError),
    Value(Box<PropertyValueFailure>),
}

impl CssPropertyValueError {
    /// A component admission failure of UTF-16 property input.
    pub(crate) const fn admission(error: CssComponentValueError) -> Self {
        Self {
            kind: PropertyValueErrorKind::Admission(error),
        }
    }

    const fn failure(&self) -> Option<&PropertyValueFailure> {
        match &self.kind {
            PropertyValueErrorKind::Value(failure) => Some(failure),
            PropertyValueErrorKind::Admission(_) => None,
        }
    }

    const fn grammar(&self) -> Option<&ErrorKind> {
        match self.failure() {
            Some(PropertyValueFailure::Grammar { kind, .. }) => Some(kind),
            _ => None,
        }
    }

    /// The component kind of an admission failure of UTF-16 property input.
    pub(crate) const fn admission_kind(&self) -> Option<CssComponentValueErrorKind> {
        match &self.kind {
            PropertyValueErrorKind::Admission(error) => Some(error.kind),
            PropertyValueErrorKind::Value(_) => None,
        }
    }

    pub(crate) fn from_component_controlled(
        error: CssComponentValueError,
        control: &crate::utf16_value::Control<'_>,
    ) -> Result<Self, ()> {
        control.add(std::mem::size_of::<PropertyValueFailure>())?;
        Ok(Self::from_component(error))
    }

    pub(crate) fn from_direct_grammar_controlled(
        kind: ErrorKind,
        origin: CssSerializedOrigin,
        control: &crate::utf16_value::Control<'_>,
    ) -> Result<Self, ()> {
        control.add(std::mem::size_of::<PropertyValueFailure>())?;
        Ok(Self::from_direct_grammar(kind, origin))
    }

    /// Backing outside the error's inline representation: the boxed failure
    /// and its grammar payload. An admission failure owns no extra backing.
    pub(crate) fn retained_backing_bytes_with<E>(
        &self,
        admit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<usize, crate::CssRetainedPayloadError<E>> {
        let Some(failure) = self.failure() else {
            return Ok(0);
        };
        admit().map_err(crate::CssRetainedPayloadError::admission)?;
        let extra = match failure {
            PropertyValueFailure::Grammar { kind, .. } => {
                kind.retained_backing_bytes_with(admit)?
            }
            PropertyValueFailure::Component(_) => 0,
        };
        std::mem::size_of::<PropertyValueFailure>()
            .checked_add(extra)
            .ok_or_else(crate::CssRetainedPayloadError::capacity_overflow)
    }

    /// Returns the responsible original token, generated boundary, or EOF origin.
    ///
    /// The origin is cloned from the retained failure.
    #[must_use]
    pub fn origin(&self) -> CssSerializedOrigin {
        match &self.kind {
            PropertyValueErrorKind::Admission(error) => {
                CssSerializedOrigin::Token(error.origin().clone())
            }
            PropertyValueErrorKind::Value(failure) => match &**failure {
                PropertyValueFailure::Component(error) => {
                    CssSerializedOrigin::Token(error.origin().clone())
                }
                PropertyValueFailure::Grammar { origin, .. } => origin.clone(),
            },
        }
    }

    /// Returns the component failure when the components could not be admitted or a
    /// component invariant or resource refused them, located at the original token.
    ///
    /// A grammar diagnostic whose code is `InvalidComponentValue` also reports its
    /// component failure here, as [`crate::CssError::component_error`] does.
    #[must_use]
    pub fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            PropertyValueErrorKind::Admission(error) => Some(error),
            PropertyValueErrorKind::Value(failure) => match &**failure {
                PropertyValueFailure::Component(error) => Some(error),
                PropertyValueFailure::Grammar {
                    kind: ErrorKind::InvalidComponentValue(error),
                    ..
                } => Some(error),
                PropertyValueFailure::Grammar { .. } => None,
            },
        }
    }

    /// Returns the code of the grammar diagnostic, or `None` for a component failure.
    #[must_use]
    pub fn code(&self) -> Option<CssErrorCode> {
        self.grammar().map(ErrorKind::code)
    }

    /// Returns the grammar expectation of the diagnostic, as [`crate::CssError::expectation`].
    #[must_use]
    pub fn expectation(&self) -> Option<CssGrammarExpectation> {
        self.grammar().and_then(ErrorKind::expectation)
    }

    /// Returns the responsible authored token of the diagnostic, as
    /// [`crate::CssError::encountered`].
    #[must_use]
    pub fn encountered(&self) -> Option<&CssTokenSummary> {
        self.grammar().and_then(ErrorKind::encountered)
    }

    /// Returns the property whose grammar rejected an `InvalidPropertyValue`.
    #[must_use]
    pub fn known_property(&self) -> Option<CssKnownProperty> {
        self.grammar().and_then(ErrorKind::known_property)
    }

    /// Returns the colour component of an `InvalidColorSyntax`, when known.
    #[must_use]
    pub fn color_component(&self) -> Option<&CssColorComponentName> {
        self.grammar().and_then(ErrorKind::color_component)
    }

    /// Returns the production of the diagnostic, as [`crate::CssError::production`].
    #[must_use]
    pub fn production(&self) -> Option<CssProductionId> {
        self.grammar().and_then(ErrorKind::production)
    }

    /// Returns the depth limit of a `NestingLimit`, as [`crate::CssError::nesting_limit`].
    #[must_use]
    pub fn nesting_limit(&self) -> Option<u32> {
        match self.grammar()? {
            ErrorKind::NestingLimit(detail) | ErrorKind::ParserUnavailable(detail) => {
                Some(detail.limit())
            }
            _ => None,
        }
    }

    /// Returns the grammar context owning an `InvalidDeclarationAnnotation`.
    #[must_use]
    pub fn declaration_context(&self) -> Option<CssDeclarationContextRef<'_>> {
        match self.grammar()? {
            ErrorKind::InvalidDeclarationAnnotation(detail) => Some(detail.context()),
            _ => None,
        }
    }

    pub(crate) fn from_component(error: CssComponentValueError) -> Self {
        Self {
            kind: PropertyValueErrorKind::Value(Box::new(PropertyValueFailure::Component(error))),
        }
    }

    pub(crate) fn from_direct_grammar(kind: ErrorKind, origin: CssSerializedOrigin) -> Self {
        if let ErrorKind::InvalidComponentValue(detail) = &kind
            && crate::error::is_component_resource_error(detail)
        {
            return Self::from_component((**detail).clone());
        }
        Self::grammar_failure(kind, origin)
    }

    fn grammar_failure(kind: ErrorKind, origin: CssSerializedOrigin) -> Self {
        Self {
            kind: PropertyValueErrorKind::Value(Box::new(PropertyValueFailure::Grammar {
                kind,
                origin,
            })),
        }
    }
}

impl fmt::Display for CssPropertyValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            PropertyValueErrorKind::Admission(error) => {
                formatter.write_str("invalid CSS property input: ")?;
                error.describe(formatter)
            }
            PropertyValueErrorKind::Value(failure) => match &**failure {
                PropertyValueFailure::Component(error) => {
                    formatter.write_str("invalid CSS property value: ")?;
                    error.describe(formatter)
                }
                PropertyValueFailure::Grammar { kind, .. } => kind.describe(formatter),
            },
        }
    }
}

impl std::error::Error for CssPropertyValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PropertyValueErrorKind::Admission(error) => Some(error),
            PropertyValueErrorKind::Value(_) => None,
        }
    }
}

fn recovered_component_error(value: &CssComponentValues) -> Option<CssPropertyValueError> {
    let origin = value.first_implicit_origin()?;
    Some(CssPropertyValueError::grammar_failure(
        crate::error::property::implicit_eof_kind(),
        CssSerializedOrigin::End(Some(origin.clone())),
    ))
}

pub(crate) fn parse_property_value_with_context(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclaration, CssPropertyValueError> {
    let body = checked_property_value_body(property, &value, parser_context)?;
    Ok(CssDeclaration::new_constructed(
        parser_context,
        body,
        importance,
        value,
    ))
}

pub(crate) fn parse_page_property_value_with_context(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclaration, CssPropertyValueError> {
    let body = checked_page_property_value_body(property, &value, parser_context)?;
    Ok(CssDeclaration::new_constructed_in_context(
        parser_context,
        crate::syntax::DeclarationContext::Page,
        body,
        importance,
        value,
    ))
}

pub(crate) fn checked_page_property_value_body(
    property: CssPropertyNameRef<'_>,
    value: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueError> {
    let body = checked_property_value_body(property, value, parser_context)?;
    let admitted = match property {
        CssPropertyNameRef::Known(property) => crate::parser::is_page_margin_property(property),
        CssPropertyNameRef::Custom(_) => true,
        CssPropertyNameRef::SvgGlyphOrientationVertical => false,
    };
    let violation = crate::parser::page_declaration_violation(&body, value);
    if !admitted || violation.is_some() {
        let component = violation
            .as_ref()
            .and_then(|violation| violation.component())
            .or_else(|| crate::parser::first_page_component(value));
        let name = match property {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            CssPropertyNameRef::SvgGlyphOrientationVertical => "glyph-orientation-vertical",
        };
        let kind = crate::error::property::property_rejection_kind(
            name,
            component,
            "an admitted Page declaration",
        );
        return Err(CssPropertyValueError::grammar_failure(
            kind,
            component.map_or(CssSerializedOrigin::End(None), |component| {
                CssSerializedOrigin::Token(component.origin().clone())
            }),
        ));
    }
    Ok(body)
}

// Declaration construction and strict expansion reentry share both the checked
// grammar and its original-component error mapping. Reentry retains the original
// occurrence and therefore must not manufacture a replacement declaration.

pub(crate) fn checked_property_value_body(
    property: CssPropertyNameRef<'_>,
    value: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueError> {
    if !matches!(property, CssPropertyNameRef::Custom(_))
        && let Some(error) = recovered_component_error(value)
    {
        return Err(error);
    }
    crate::parser::parse_property_value_body(property, value, parser_context)
}

pub(crate) fn checked_svg_glyph_value_body(
    admission: crate::svg_glyph::SvgGlyphAdmission,
    value: &CssComponentValues,
    context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueError> {
    if let Some(error) = recovered_component_error(value) {
        return Err(error);
    }
    crate::parser::parse_svg_glyph_value_body(admission, value, context)
}

pub(crate) fn parse_property_value_for_grammar_with_context(
    grammar: crate::CssPropertyGrammar,
    value: CssComponentValues,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclaration, CssPropertyValueError> {
    let body = checked_grammar_value_body(grammar, &value, parser_context)?;
    Ok(CssDeclaration::new_constructed(
        parser_context,
        body,
        importance,
        value,
    ))
}

pub(crate) fn checked_grammar_value_body(
    grammar: crate::CssPropertyGrammar,
    value: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueError> {
    if let Some(error) = recovered_component_error(value) {
        return Err(error);
    }
    crate::parser::parse_property_value_body_for_grammar(grammar, value, parser_context)
}
/// Which grammar a declaration value is checked against.
///
/// Chosen by the calling code, never by CSS input.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssPropertyTarget<'a> {
    /// The ordinary style-rule grammar of a named property.
    Property(CssPropertyNameRef<'a>),
    /// An explicit grammar handle, including legacy shorthand grammars.
    Grammar(crate::CssPropertyGrammar),
    /// A property in `@page` context.
    PageProperty(CssPropertyNameRef<'a>),
    /// A property in a margin-box context.
    MarginProperty(CssPropertyNameRef<'a>),
}

/// Which grammar the source-text front [`CssParser::parse_property_value`](crate::CssParser::parse_property_value)
/// checks a value against.
///
/// A page or margin value is checked only from owned components, through
/// [`CssDeclaration::try_from_components`] and [`CssPropertyTarget`], so this type
/// admits just the targets the text front can serve. Chosen by the calling code,
/// never by CSS input.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssPropertyTextTarget<'a> {
    /// The ordinary style-rule grammar of a named property.
    Property(CssPropertyNameRef<'a>),
    /// An explicit grammar handle, including legacy shorthand grammars.
    Grammar(crate::CssPropertyGrammar),
}

impl CssDeclaration {
    /// Checks owned components against a target grammar and constructs an authored
    /// declaration occurrence.
    ///
    /// A [`CssPropertyTarget::Property`] selects the same grammar as stylesheet
    /// declarations; a [`CssPropertyTarget::Grammar`] an explicit canonical or
    /// legacy grammar, with the same direct token admission, limits and
    /// original-origin mapping. Whole-value CSS-wide keywords and valid
    /// substitution-dependent values remain symbolic. Custom names preserve case
    /// and allow empty values. Known grammars reject implicit closures in the
    /// original components, including CSS-wide and substitution-dependent values.
    /// Top-level annotations and declaration delimiters are rejected: importance
    /// comes only from the separate argument.
    ///
    /// A [`CssPropertyTarget::PageProperty`] or [`CssPropertyTarget::MarginProperty`]
    /// checks the components as one authored Page declaration. It admits
    /// represented Page/margin-applicable shared properties and custom values.
    /// CSS-wide, calculation, length-percentage and substitution grammar remains
    /// symbolic, including font-relative operands. Page physical margins exclude
    /// the crate's logical-margin extension and nonzero unitless values. Custom
    /// tokens are retained without margin restrictions or variable lookup. The
    /// occurrence retains Page semantic context independently of document mode,
    /// so strict pending reentry cannot silently use ordinary element grammar. A
    /// margin-box target shares the Page producer boundary. Page-only descriptors
    /// are parsed by [`CssPageDescriptorValue::parse`](crate::CssPageDescriptorValue::parse).
    ///
    /// Supplied token origins remain intact. The returned declaration has no
    /// parsed property-name position or single parsed declaration-value span;
    /// inspect its components for their origins. Admission reads the supplied
    /// tokens directly and obeys the immutable component tree's previously
    /// validated structural and serialization constraints. This operation does
    /// not substitute variables, expand shorthands, resolve values, or apply
    /// cascade. The parser supplies the document mode.
    ///
    /// ```
    /// use surgeist_css::{
    ///     CssComponentValues, CssDeclaration, CssExpansion, CssImportance, CssKnownProperty,
    ///     CssParser, CssPropertyNameRef, CssPropertyTarget,
    /// };
    /// let parser = CssParser::default();
    /// let margin = CssDeclaration::try_from_components(
    ///     &parser,
    ///     CssPropertyTarget::PageProperty(CssPropertyNameRef::Known(CssKnownProperty::Margin)),
    ///     CssComponentValues::parse(&parser, "var(--m, 1px)").unwrap(),
    ///     CssImportance::Normal,
    /// ).unwrap();
    /// let CssExpansion::Pending(pending) = margin.expand().unwrap() else {
    ///     panic!("pending authored Page margin");
    /// };
    /// assert!(pending.reenter(CssComponentValues::parse(&parser, "1em").unwrap()).is_ok());
    /// assert!(pending
    ///     .reenter(CssComponentValues::parse(&parser, "calc(1Q + 2%)").unwrap())
    ///     .is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the component or grammar failure located at the responsible
    /// original token.
    pub fn try_from_components(
        parser: &crate::CssParser,
        target: CssPropertyTarget<'_>,
        values: CssComponentValues,
        importance: CssImportance,
    ) -> Result<CssDeclaration, CssPropertyValueError> {
        let context = parser.context();
        match target {
            CssPropertyTarget::Property(property) => {
                parse_property_value_with_context(property, values, importance, context)
            }
            CssPropertyTarget::Grammar(grammar) => {
                parse_property_value_for_grammar_with_context(grammar, values, importance, context)
            }
            CssPropertyTarget::PageProperty(property)
            | CssPropertyTarget::MarginProperty(property) => {
                parse_page_property_value_with_context(property, values, importance, context)
            }
        }
    }
}
