//! Checked authored custom-media definitions and symbolic references.
use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
use crate::*;
use std::fmt;

/// A decoded extension name and its original identifier component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomMediaName {
    component: CssComponentValue,
    name: String,
}
impl CssCustomMediaName {
    /// Constructs from a decoded identifier value, escaping it as necessary.
    pub fn try_new(name: impl Into<String>) -> Result<Self, CssCustomMediaConstructionError> {
        Self::try_from_component(CssComponentValue::try_ident(name)?)
    }
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssCustomMediaConstructionError> {
        let Some(name) = ident(&component).filter(|name| name.starts_with("--")) else {
            return Err(CssCustomMediaConstructionError::InvalidName {
                origin: component.origin().clone(),
            });
        };
        let name = name.to_owned();
        Ok(Self { component, name })
    }
    pub fn as_str(&self) -> &str {
        &self.name
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}
/// An authored definition body, without definition lookup or evaluation.
#[derive(Clone, Debug, PartialEq)]
pub enum CssCustomMediaBody {
    True,
    False,
    Media(CssMediaQueryList),
}
/// A checked symbolic boolean reference. The original parentheses remain available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomMediaReference {
    syntax: Box<CustomMediaReferenceSyntax>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct CustomMediaReferenceSyntax {
    name: CssCustomMediaName,
    component: CssComponentValue,
}
impl CssCustomMediaReference {
    pub(crate) fn new(name: CssCustomMediaName, component: CssComponentValue) -> Self {
        Self {
            syntax: Box::new(CustomMediaReferenceSyntax { name, component }),
        }
    }
    pub const fn name(&self) -> &CssCustomMediaName {
        &self.syntax.name
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.syntax.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.syntax.component.origin()
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}
/// A checked authored statement. Duplicate definitions and cycles remain symbolic.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCustomMediaRule {
    data: Box<CustomMediaRuleData>,
}

#[derive(Clone, Debug, PartialEq)]
struct CustomMediaRuleData {
    name: CssCustomMediaName,
    body: CssCustomMediaBody,
    origin: CssValueOrigin,
    body_origin: CssValueOrigin,
    semicolon: CssValueOrigin,
}
/// A failure to construct an unrecovered custom-media definition.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCustomMediaConstructionError {
    InvalidName { origin: CssValueOrigin },
    InvalidPrelude { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    AmbiguousMediaBody { origin: CssValueOrigin },
    Component(CssComponentValueError),
    Media(CssMediaConstructionError),
}
/// An atomic custom-media serialization failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCustomMediaSerializationError {
    Component(CssComponentValueError),
    Media(CssMediaSerializationError),
}
impl CssCustomMediaConstructionError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidName { origin }
            | Self::InvalidPrelude { origin }
            | Self::RecoveredInput { origin }
            | Self::AmbiguousMediaBody { origin } => origin,
            Self::Component(e) => e.origin(),
            Self::Media(e) => e.origin(),
        }
    }
}
impl CssCustomMediaSerializationError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Component(e) => e.origin(),
            Self::Media(e) => e.origin(),
        }
    }
}
impl fmt::Display for CssCustomMediaConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "custom-media construction failed: {self:?}")
    }
}
impl fmt::Display for CssCustomMediaSerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "custom-media serialization failed: {self:?}")
    }
}
impl std::error::Error for CssCustomMediaConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(e) => Some(e),
            Self::Media(e) => Some(e),
            _ => None,
        }
    }
}
impl std::error::Error for CssCustomMediaSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(e) => Some(e),
            Self::Media(e) => Some(e),
        }
    }
}
impl From<CssComponentValueError> for CssCustomMediaConstructionError {
    fn from(e: CssComponentValueError) -> Self {
        Self::Component(e)
    }
}
impl From<CssMediaConstructionError> for CssCustomMediaConstructionError {
    fn from(e: CssMediaConstructionError) -> Self {
        Self::Media(e)
    }
}
impl From<CssComponentValueError> for CssCustomMediaSerializationError {
    fn from(e: CssComponentValueError) -> Self {
        Self::Component(e)
    }
}
impl From<CssMediaSerializationError> for CssCustomMediaSerializationError {
    fn from(e: CssMediaSerializationError) -> Self {
        Self::Media(e)
    }
}

pub(crate) fn ident(value: &CssComponentValue) -> Option<&str> {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => Some(name),
        _ => None,
    }
}
pub(crate) fn trivia(value: &CssComponentValue) -> bool {
    matches!(
        value.view(),
        CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}
pub(crate) fn boolean_keyword(name: &str) -> Option<bool> {
    if name.eq_ignore_ascii_case("true") {
        Some(true)
    } else if name.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}
pub(crate) fn boolean_body(
    values: &[CssComponentValue],
) -> Option<(CssCustomMediaBody, CssValueOrigin)> {
    let mut meaningful = values.iter().filter(|v| !trivia(v));
    let token = meaningful.next()?;
    if meaningful.next().is_some() {
        return None;
    }
    let name = ident(token)?;
    let body = if boolean_keyword(name)? {
        CssCustomMediaBody::True
    } else {
        CssCustomMediaBody::False
    };
    Some((body, token.origin().clone()))
}
impl CssCustomMediaRule {
    /// Checks a typed body without resolving references. Recovered members and a
    /// singleton unmodified true/false media type cannot construct this rule.
    pub fn try_new(
        name: CssCustomMediaName,
        body: CssCustomMediaBody,
    ) -> Result<Self, CssCustomMediaConstructionError> {
        Self::try_new_with_limits(name, body, CssComponentValueLimits::default())
    }
    /// Checks the complete canonical statement against component and byte limits.
    /// Generated at-keyword and semicolon origins are programmatic.
    pub fn try_new_with_limits(
        name: CssCustomMediaName,
        body: CssCustomMediaBody,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssCustomMediaConstructionError> {
        let deep = matches!(&body,CssCustomMediaBody::Media(list) if list.queries().iter().any(crate::media::query_is_deep));
        crate::media::with_media_stack(deep, || {
            let rule = Self::parsed(
                name,
                body,
                CssValueOrigin::Programmatic,
                CssValueOrigin::Programmatic,
                CssValueOrigin::Programmatic,
            );
            rule.validate(limits)?;
            Ok(rule)
        })
    }
    /// Admits a prelude (name followed by body), preserving original component origins.
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssCustomMediaConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    /// Checks both supplied prelude components and canonical statement output.
    /// The absent at-keyword has no parsed position; supplied tokens keep theirs.
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssCustomMediaConstructionError> {
        crate::media::with_media_stack(values.nesting_depth() >= 64, || {
            values.validate_with_limits(limits)?;
            if let Some(origin) = values.first_implicit_origin() {
                return Err(CssCustomMediaConstructionError::RecoveredInput {
                    origin: origin.clone(),
                });
            }
            let Some(index) = values.items().iter().position(|v| !trivia(v)) else {
                return Err(CssCustomMediaConstructionError::InvalidPrelude {
                    origin: CssValueOrigin::Programmatic,
                });
            };
            let name = CssCustomMediaName::try_from_component(values.items()[index].clone())?;
            let tail = &values.items()[index + 1..];
            let (body, body_origin) = if let Some(boolean) = boolean_body(tail) {
                boolean
            } else {
                let mut queries = Vec::new();
                let mut commas = Vec::new();
                let mut start = 0;
                let empty = tail.iter().all(trivia);
                if !empty {
                    for end in 0..=tail.len() {
                        if end == tail.len()
                            || matches!(
                                tail[end].view(),
                                CssComponentValueRef::Token(CssValueTokenRef::Comma)
                            )
                        {
                            queries.push(CssMediaQuery::try_from_components_with_limits(
                                CssComponentValues::try_new(tail[start..end].to_vec())?,
                                limits,
                            )?);
                            if end < tail.len() {
                                commas.push(tail[end].origin().clone());
                            }
                            start = end + 1;
                        }
                    }
                }
                (
                    CssCustomMediaBody::Media(CssMediaQueryList::with_comma_origins(
                        queries, commas,
                    )),
                    tail.iter()
                        .find(|v| !trivia(v))
                        .map_or(CssValueOrigin::Programmatic, |v| v.origin().clone()),
                )
            };
            let rule = Self::parsed(
                name,
                body,
                CssValueOrigin::Programmatic,
                body_origin,
                CssValueOrigin::Programmatic,
            );
            rule.validate(limits)?;
            Ok(rule)
        })
    }
    pub(crate) fn parsed(
        name: CssCustomMediaName,
        body: CssCustomMediaBody,
        origin: CssValueOrigin,
        body_origin: CssValueOrigin,
        semicolon: CssValueOrigin,
    ) -> Self {
        Self {
            data: Box::new(CustomMediaRuleData {
                name,
                body,
                origin,
                body_origin,
                semicolon,
            }),
        }
    }
    pub const fn name(&self) -> &CssCustomMediaName {
        &self.data.name
    }
    pub const fn body(&self) -> &CssCustomMediaBody {
        &self.data.body
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.data.origin
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.data.origin)
    }
    /// Emits canonical authored syntax with original token origins. Any recovered
    /// media member fails the entire operation before an output is returned.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssCustomMediaSerializationError> {
        let deep = matches!(&self.data.body,CssCustomMediaBody::Media(list) if list.queries().iter().any(crate::media::query_is_deep));
        crate::media::with_media_stack(deep, || self.emit(usize::MAX))
    }
    fn emit(
        &self,
        max_bytes: usize,
    ) -> Result<CssSerializedValue, CssCustomMediaSerializationError> {
        let mut out = CssCanonicalBuilder::new(max_bytes);
        out.push_grammar(
            CssCanonicalToken::AtKeyword("custom-media"),
            &self.data.origin,
        )?;
        out.push_grammar(CssCanonicalToken::Whitespace, self.data.name.origin())?;
        out.push_component(self.data.name.component())?;
        match &self.data.body {
            CssCustomMediaBody::True | CssCustomMediaBody::False => {
                out.push_grammar(CssCanonicalToken::Whitespace, &self.data.body_origin)?;
                out.push_grammar(
                    CssCanonicalToken::Ident(
                        if matches!(self.data.body, CssCustomMediaBody::True) {
                            "true"
                        } else {
                            "false"
                        },
                    ),
                    &self.data.body_origin,
                )?;
            }
            CssCustomMediaBody::Media(list) => {
                for (index, query) in list.queries().iter().enumerate() {
                    let origin = if index == 0 {
                        query.origin()
                    } else {
                        list.comma_origins
                            .get(index - 1)
                            .unwrap_or(&CssValueOrigin::Programmatic)
                    };
                    if index > 0 {
                        out.push_grammar(CssCanonicalToken::Comma, origin)?;
                    }
                    out.push_grammar(CssCanonicalToken::Whitespace, origin)?;
                    crate::media::emit_query(&mut out, query)?;
                }
            }
        }
        out.push_grammar(CssCanonicalToken::Semicolon, &self.data.semicolon)?;
        Ok(out.finish()?)
    }
    fn validate(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<(), CssCustomMediaConstructionError> {
        if let CssCustomMediaBody::Media(list) = &self.data.body {
            if let [CssMediaQuery::Typed(query)] = list.queries()
                && query.modifier().is_none()
                && query.condition().is_none()
                && ident(&query.syntax.media_type).is_some_and(|name| {
                    name.eq_ignore_ascii_case("true") || name.eq_ignore_ascii_case("false")
                })
            {
                return Err(CssCustomMediaConstructionError::AmbiguousMediaBody {
                    origin: query.syntax.media_type.origin().clone(),
                });
            }
            if let Some(CssMediaQuery::Never(value)) = list
                .queries()
                .iter()
                .find(|q| matches!(q, CssMediaQuery::Never(_)))
            {
                return Err(CssCustomMediaConstructionError::RecoveredInput {
                    origin: value.origin().clone(),
                });
            }
        }

        let output = self.emit(limits.max_css_bytes()).map_err(|e| match e {
            CssCustomMediaSerializationError::Component(e) => {
                CssCustomMediaConstructionError::Component(e)
            }
            CssCustomMediaSerializationError::Media(CssMediaSerializationError::Component(e)) => {
                CssCustomMediaConstructionError::Component(e)
            }
            CssCustomMediaSerializationError::Media(
                CssMediaSerializationError::RecoveredNever { origin },
            ) => CssCustomMediaConstructionError::RecoveredInput { origin },
        })?;
        for segment in output.segments() {
            if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) =
                output.value_origin_at(segment.byte_range().start)
            {
                return Err(CssCustomMediaConstructionError::RecoveredInput {
                    origin: origin.clone(),
                });
            }
        }
        // Validate resource consumption only. The original typed graph is never replaced.
        parse_component_values_with_limits(output.as_css(), limits).map_err(|error| {
            let offset = crate::media::parsed_position(error.origin())
                .map_or(0, |p| p.byte_offset().value());
            CssCustomMediaConstructionError::Component(CssComponentValueError::new(
                error.kind(),
                output
                    .value_origin_at(offset)
                    .cloned()
                    .unwrap_or_else(|| self.data.origin.clone()),
            ))
        })?;
        Ok(())
    }
}
