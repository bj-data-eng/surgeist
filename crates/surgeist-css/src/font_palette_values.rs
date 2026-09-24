//! Checked authored `@font-palette-values` syntax without palette lookup or substitution.

use std::fmt;

use crate::{
    CssAbsoluteColorEligibility, CssAbsoluteColorExclusion, CssAuthoredColor, CssComponentValue,
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValueLimits,
    CssComponentValueRef, CssComponentValues, CssFontFaceFamily, CssIntegerValue,
    CssSerializedOrigin, CssSerializedValue, CssSourcePosition, CssSubstitutionDependentValue,
    CssValueOrigin, Error, ErrorKind,
};

/// A rejected semantic palette component or rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteConstructionError {
    InvalidName,
    NegativeIndex,
    ContextualColor(CssAbsoluteColorExclusion),
    MissingFontFamily,
}

impl fmt::Display for CssFontPaletteConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid font-palette-values construction: {self:?}"
        )
    }
}

impl std::error::Error for CssFontPaletteConstructionError {}

/// A decoded, case-sensitive `<dashed-ident>` palette name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontPaletteName(Box<str>);

impl CssFontPaletteName {
    /// Checks the decoded identifier, including the valid bare `--` spelling.
    pub fn try_new(value: &str) -> Result<Self, CssFontPaletteConstructionError> {
        if !value.starts_with("--") || CssComponentValue::try_ident(value).is_err() {
            return Err(CssFontPaletteConstructionError::InvalidName);
        }
        Ok(Self(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The three descriptor grammars in a palette definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteDescriptorKind {
    FontFamily,
    BasePalette,
    OverrideColors,
}

impl CssFontPaletteDescriptorKind {
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::FontFamily => "font-family",
            Self::BasePalette => "base-palette",
            Self::OverrideColors => "override-colors",
        }
    }

    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
        [Self::FontFamily, Self::BasePalette, Self::OverrideColors]
            .into_iter()
            .find(|kind| kind.css_name().eq_ignore_ascii_case(name))
    }
}

/// An exact nonnegative ordinary integer or an unresolved Integer-root calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteIndex {
    value: CssIntegerValue,
}

impl CssFontPaletteIndex {
    pub fn try_new(value: CssIntegerValue) -> Result<Self, CssFontPaletteConstructionError> {
        let negative = match &value {
            CssIntegerValue::Literal(value) => *value < 0,
            CssIntegerValue::ExactLiteral(value) => {
                let spelling = value.numeric().representation();
                let digits = spelling.strip_prefix(['+', '-']).unwrap_or(spelling);
                spelling.starts_with('-') && !digits.trim_start_matches('0').is_empty()
            }
            CssIntegerValue::Calculation(_) => false,
        };
        if negative {
            return Err(CssFontPaletteConstructionError::NegativeIndex);
        }
        Ok(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssIntegerValue {
        &self.value
    }
}

/// The authored base palette selection; absence remains distinct on the rule.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteBase {
    Light,
    Dark,
    Index(CssFontPaletteIndex),
}

/// An authored override pair. Duplicated indices are retained by its owning list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteOverride {
    index: CssFontPaletteIndex,
    color: CssAuthoredColor,
}

impl CssFontPaletteOverride {
    pub fn try_new(
        index: CssFontPaletteIndex,
        color: CssAuthoredColor,
    ) -> Result<Self, CssFontPaletteConstructionError> {
        if let CssAbsoluteColorEligibility::Contextual(reason) = color.absolute_eligibility() {
            return Err(CssFontPaletteConstructionError::ContextualColor(reason));
        }
        Ok(Self { index, color })
    }

    #[must_use]
    pub const fn index(&self) -> &CssFontPaletteIndex {
        &self.index
    }

    #[must_use]
    pub const fn color(&self) -> &CssAuthoredColor {
        &self.color
    }
}

/// A checked component-native descriptor construction or reentry failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteValueErrorKind {
    Component(CssComponentValueErrorKind),
    Grammar(ErrorKind),
    ResidualSubstitution,
    NotPending,
}

/// A failure mapped to the supplied component origins, never fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontPaletteValueError {
    kind: Box<CssFontPaletteValueErrorKind>,
    origin: Box<CssSerializedOrigin>,
    source: Option<Box<PaletteValueErrorSource>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PaletteValueErrorSource {
    Component(CssComponentValueError),
    Grammar(Error),
}

impl CssFontPaletteValueError {
    #[must_use]
    pub fn kind(&self) -> &CssFontPaletteValueErrorKind {
        &self.kind
    }

    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }

    fn from_component(error: CssComponentValueError) -> Self {
        Self {
            kind: Box::new(CssFontPaletteValueErrorKind::Component(error.kind())),
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            source: Some(Box::new(PaletteValueErrorSource::Component(error))),
        }
    }

    fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("palette parser reports a cursor inside its serialized input or at EOF")
            .clone();
        Self {
            kind: Box::new(CssFontPaletteValueErrorKind::Grammar(error.kind().clone())),
            origin: Box::new(origin),
            source: Some(Box::new(PaletteValueErrorSource::Grammar(error))),
        }
    }

    fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: Box::new(CssFontPaletteValueErrorKind::ResidualSubstitution),
            origin: Box::new(CssSerializedOrigin::Token(origin)),
            source: None,
        }
    }

    fn not_pending() -> Self {
        Self {
            kind: Box::new(CssFontPaletteValueErrorKind::NotPending),
            origin: Box::new(CssSerializedOrigin::End(None)),
            source: None,
        }
    }
}

impl fmt::Display for CssFontPaletteValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid font-palette descriptor value: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for CssFontPaletteValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_deref()? {
            PaletteValueErrorSource::Component(error) => Some(error),
            PaletteValueErrorSource::Grammar(error) => Some(error),
        }
    }
}

/// Borrowed checked descriptor semantics. Components retain the exact authored syntax.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteDescriptorValueRef<'a> {
    FontFamily(&'a [CssFontFaceFamily]),
    BasePalette(&'a CssFontPaletteBase),
    OverrideColors(&'a [CssFontPaletteOverride]),
    Pending(&'a CssSubstitutionDependentValue),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssFontPaletteDescriptorData {
    FontFamily(Vec<CssFontFaceFamily>),
    BasePalette(CssFontPaletteBase),
    OverrideColors(Vec<CssFontPaletteOverride>),
    Pending(CssSubstitutionDependentValue),
}

/// One checked whole descriptor value with original component provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteDescriptorValue {
    kind: CssFontPaletteDescriptorKind,
    components: CssComponentValues,
    data: CssFontPaletteDescriptorData,
}

impl CssFontPaletteDescriptorValue {
    /// Checks the selected grammar while retaining mixed or programmatic origins.
    pub fn try_new(
        kind: CssFontPaletteDescriptorKind,
        components: CssComponentValues,
    ) -> Result<Self, CssFontPaletteValueError> {
        Self::try_new_with_limits(kind, components, CssComponentValueLimits::default())
    }

    /// Uses the same component and output-byte limits as shared component serialization.
    pub fn try_new_with_limits(
        kind: CssFontPaletteDescriptorKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontPaletteValueError> {
        components
            .validate_with_limits(limits)
            .map_err(CssFontPaletteValueError::from_component)?;
        let serialized = components
            .serialize_with_limit(limits.max_css_bytes())
            .map_err(CssFontPaletteValueError::from_component)?;
        let data = crate::parser::font_palette_values::construct_descriptor_value(
            kind,
            &components,
            &serialized,
        )
        .map_err(|error| CssFontPaletteValueError::from_grammar(error, &serialized))?;
        Ok(Self {
            kind,
            components,
            data,
        })
    }

    /// Rechecks caller-supplied replacement components without performing substitution.
    /// Residual decoded `var()` or `env()` anywhere in the tree is rejected first.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
    ) -> Result<Self, CssFontPaletteValueError> {
        self.reparse_after_substitution_with_limits(replacement, CssComponentValueLimits::default())
    }

    pub fn reparse_after_substitution_with_limits(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontPaletteValueError> {
        if !matches!(self.data, CssFontPaletteDescriptorData::Pending(_)) {
            return Err(CssFontPaletteValueError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontPaletteValueError::from_component)?;
        if let Some(origin) = first_substitution_origin(&replacement) {
            return Err(CssFontPaletteValueError::residual(origin));
        }
        Self::try_new_with_limits(self.kind, replacement, limits)
    }

    pub(crate) fn from_parts(
        kind: CssFontPaletteDescriptorKind,
        components: CssComponentValues,
        data: CssFontPaletteDescriptorData,
    ) -> Self {
        let matching_kind = match &data {
            CssFontPaletteDescriptorData::FontFamily(_) => {
                kind == CssFontPaletteDescriptorKind::FontFamily
            }
            CssFontPaletteDescriptorData::BasePalette(_) => {
                kind == CssFontPaletteDescriptorKind::BasePalette
            }
            CssFontPaletteDescriptorData::OverrideColors(_) => {
                kind == CssFontPaletteDescriptorKind::OverrideColors
            }
            CssFontPaletteDescriptorData::Pending(_) => true,
        };
        debug_assert!(matching_kind);
        Self {
            kind,
            components,
            data,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssFontPaletteDescriptorKind {
        self.kind
    }

    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    #[must_use]
    pub fn view(&self) -> CssFontPaletteDescriptorValueRef<'_> {
        match &self.data {
            CssFontPaletteDescriptorData::FontFamily(value) => {
                CssFontPaletteDescriptorValueRef::FontFamily(value)
            }
            CssFontPaletteDescriptorData::BasePalette(value) => {
                CssFontPaletteDescriptorValueRef::BasePalette(value)
            }
            CssFontPaletteDescriptorData::OverrideColors(value) => {
                CssFontPaletteDescriptorValueRef::OverrideColors(value)
            }
            CssFontPaletteDescriptorData::Pending(value) => {
                CssFontPaletteDescriptorValueRef::Pending(value)
            }
        }
    }
}

fn first_substitution_origin(values: &CssComponentValues) -> Option<CssValueOrigin> {
    let mut stack = vec![values.items().iter()];
    while let Some(items) = stack.last_mut() {
        if let Some(item) = items.next() {
            match item.view() {
                CssComponentValueRef::Function(function) => {
                    if function.name().eq_ignore_ascii_case("var")
                        || function.name().eq_ignore_ascii_case("env")
                    {
                        return Some(item.origin().clone());
                    }
                    stack.push(function.values().items().iter());
                }
                CssComponentValueRef::Block(block) => stack.push(block.values().items().iter()),
                _ => {}
            }
        } else {
            stack.pop();
        }
    }
    None
}

/// One ordered descriptor occurrence; checked construction invents no source position.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteDescriptor {
    value: CssFontPaletteDescriptorValue,
    position: Option<CssSourcePosition>,
}

impl CssFontPaletteDescriptor {
    #[must_use]
    pub const fn new(value: CssFontPaletteDescriptorValue) -> Self {
        Self {
            value,
            position: None,
        }
    }

    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub const fn value(&self) -> &CssFontPaletteDescriptorValue {
        &self.value
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// An authored palette definition retaining every valid descriptor occurrence in order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteValuesRule {
    name: CssFontPaletteName,
    descriptors: Vec<CssFontPaletteDescriptor>,
    position: Option<CssSourcePosition>,
}

impl CssFontPaletteValuesRule {
    pub fn try_new(
        name: CssFontPaletteName,
        descriptors: Vec<CssFontPaletteDescriptor>,
    ) -> Result<Self, CssFontPaletteConstructionError> {
        if !descriptors
            .iter()
            .any(|descriptor| descriptor.value.kind() == CssFontPaletteDescriptorKind::FontFamily)
        {
            return Err(CssFontPaletteConstructionError::MissingFontFamily);
        }
        Ok(Self {
            name,
            descriptors,
            position: None,
        })
    }

    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub const fn name(&self) -> &CssFontPaletteName {
        &self.name
    }

    #[must_use]
    pub fn descriptors(&self) -> &[CssFontPaletteDescriptor] {
        &self.descriptors
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}
