//! Authored UI4 values. Contextual caret, widget, focus and timing behavior is downstream.
use crate::{
    CssCaretColor, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef, CssHashFlag, CssTimeValue, CssValueOrigin, CssValueTokenRef,
};

macro_rules! keywords {
    ($name:ident { $initial:ident, $($variant:ident),+ }) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum $name { #[default] $initial, $($variant),+ }
    };
}
keywords!(CssCaretAnimation { Auto, Manual });
keywords!(CssCaretShape {
    Auto,
    Bar,
    Block,
    Underscore
});
keywords!(CssInteractivity { Auto, Inert });
keywords!(CssAppearance {
    None,
    Auto,
    Base,
    BaseSelect,
    Searchfield,
    Textarea,
    Checkbox,
    Radio,
    Menulist,
    Listbox,
    Meter,
    ProgressBar,
    Button,
    Textfield,
    MenulistButton
});

/// Accent selection stays symbolic until the style owner has context.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum CssAccentColor {
    #[default]
    Auto,
    Color(Box<crate::CssColor>),
}
impl CssAccentColor {
    pub fn color(&self) -> Option<&crate::CssColor> {
        match self {
            Self::Auto => None,
            Self::Color(color) => Some(color),
        }
    }
}

/// A nonempty unordered Caret value retaining the supplied roles and omissions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCaret {
    color: Option<CssCaretColor>,
    animation: Option<CssCaretAnimation>,
    shape: Option<CssCaretShape>,
}
impl CssCaret {
    pub fn try_new(
        color: Option<CssCaretColor>,
        animation: Option<CssCaretAnimation>,
        shape: Option<CssCaretShape>,
    ) -> Option<Self> {
        if color.is_none() && animation.is_none() && shape.is_none() {
            return None;
        }
        Some(Self {
            color,
            animation,
            shape,
        })
    }
    pub const fn color(&self) -> Option<&CssCaretColor> {
        self.color.as_ref()
    }
    pub const fn animation(&self) -> Option<CssCaretAnimation> {
        self.animation
    }
    pub const fn shape(&self) -> Option<CssCaretShape> {
        self.shape
    }
}

/// Normal is symbolic; ordinary times are signed and calculations remain authored.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub enum CssInterestDelayValue {
    #[default]
    Normal,
    Time(CssTimeValue),
}
impl PartialEq for CssInterestDelayValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::Time(left), Self::Time(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}
impl Eq for CssInterestDelayValue {}
impl CssInterestDelayValue {
    pub fn time(&self) -> Option<&CssTimeValue> {
        match self {
            Self::Normal => None,
            Self::Time(value) => Some(value),
        }
    }
    pub(crate) fn specified_value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::Time(left), Self::Time(right)) => left.specified_value_eq(right),
            _ => false,
        }
    }
}
/// One or two retained values; an omitted end repeats start without erasing presence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssInterestDelay {
    start: CssInterestDelayValue,
    end: Option<CssInterestDelayValue>,
}
impl CssInterestDelay {
    pub fn try_new(
        start: CssInterestDelayValue,
        end: Option<CssInterestDelayValue>,
    ) -> Result<Self, crate::CssNumericConstructionError> {
        for value in std::iter::once(&start).chain(end.iter()) {
            if let Some(time) = value.time() {
                time.ensure_closed()?;
            }
        }
        Ok(Self { start, end })
    }
    pub(crate) fn from_parser(
        start: CssInterestDelayValue,
        end: Option<CssInterestDelayValue>,
    ) -> Self {
        Self { start, end }
    }
    pub const fn start(&self) -> &CssInterestDelayValue {
        &self.start
    }
    pub const fn authored_end(&self) -> Option<&CssInterestDelayValue> {
        self.end.as_ref()
    }
    pub fn end(&self) -> &CssInterestDelayValue {
        self.end.as_ref().unwrap_or(&self.start)
    }
}

fn invalid(
    component: &CssComponentValue,
    kind: CssComponentValueErrorKind,
) -> CssComponentValueError {
    CssComponentValueError::new(kind, component.origin().clone())
}
/// A checked ID-selector token, preserving decoded case, escapes and original provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNavigationId {
    component: CssComponentValue,
}
impl CssNavigationId {
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if !matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Hash {
                flag: CssHashFlag::Id,
                ..
            })
        ) {
            return Err(invalid(
                &component,
                CssComponentValueErrorKind::InvalidToken,
            ));
        }
        Ok(Self { component })
    }
    pub fn try_new(decoded: &str) -> Result<Self, CssComponentValueError> {
        let ident = CssComponentValue::try_ident(decoded)?;
        let values = crate::CssComponentValues::try_new(vec![ident])?;
        let css = values.serialize()?;
        Self::try_from_component(CssComponentValue::try_token(&format!("#{}", css.as_css()))?)
    }
    pub fn as_str(&self) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::Hash { value, .. }) =
            self.component.view()
        else {
            unreachable!("checked ID")
        };
        value
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}
/// A conforming target string, including the empty string, never underscore-leading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNavigationTargetName {
    component: CssComponentValue,
}
impl CssNavigationTargetName {
    pub fn try_new(decoded: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_string(decoded)?)
    }
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        let CssComponentValueRef::Token(CssValueTokenRef::String(value)) = component.view() else {
            return Err(invalid(
                &component,
                CssComponentValueErrorKind::InvalidToken,
            ));
        };
        if value.starts_with('_') {
            return Err(invalid(
                &component,
                CssComponentValueErrorKind::InvalidString,
            ));
        }
        if component.implicit_termination_origin().is_some() {
            return Err(invalid(
                &component,
                CssComponentValueErrorKind::InvalidToken,
            ));
        }
        Ok(Self { component })
    }
    pub fn as_str(&self) -> &str {
        string(&self.component)
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    pub(crate) fn from_parser(component: CssComponentValue) -> Self {
        Self { component }
    }
}
fn string(component: &CssComponentValue) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::String(value)) = component.view() else {
        unreachable!("checked string")
    };
    value
}
/// Parser-retained nonconforming target. No public constructor manufactures legacy repair syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssLegacyNavigationTargetName {
    component: CssComponentValue,
}
impl CssLegacyNavigationTargetName {
    pub(crate) fn from_parser(component: CssComponentValue) -> Self {
        Self { component }
    }
    pub fn as_str(&self) -> &str {
        string(&self.component)
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNavigationTarget {
    Current,
    Root,
    Name(CssNavigationTargetName),
    LegacyName(CssLegacyNavigationTargetName),
}
/// A symbolic directional focus input; this model performs no frame navigation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNavigation {
    #[default]
    Auto,
    Id(Box<CssNavigationReference>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNavigationReference {
    id: CssNavigationId,
    target: Option<CssNavigationTarget>,
}
impl CssNavigationReference {
    /// Checked construction excludes browser-retained legacy names and recovered strings.
    pub fn try_new(id: CssNavigationId, target: Option<CssNavigationTarget>) -> Option<Self> {
        if matches!(target, Some(CssNavigationTarget::LegacyName(_))) {
            return None;
        }
        if let Some(CssNavigationTarget::Name(name)) = &target
            && name.component.implicit_termination_origin().is_some()
        {
            return None;
        }
        Some(Self { id, target })
    }
    pub(crate) fn from_parser(id: CssNavigationId, target: Option<CssNavigationTarget>) -> Self {
        Self { id, target }
    }
    pub const fn id(&self) -> &CssNavigationId {
        &self.id
    }
    pub const fn target(&self) -> Option<&CssNavigationTarget> {
        self.target.as_ref()
    }
}
impl CssNavigation {
    pub fn reference(&self) -> Option<&CssNavigationReference> {
        match self {
            Self::Auto => None,
            Self::Id(value) => Some(value),
        }
    }
    pub(crate) fn legacy_target(&self) -> Option<&CssLegacyNavigationTargetName> {
        match self.reference()?.target()? {
            CssNavigationTarget::LegacyName(value) => Some(value),
            _ => None,
        }
    }
}
