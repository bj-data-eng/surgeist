//! Authored UI4 values. Contextual caret, widget, focus and timing behavior is downstream.
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueRef, CssHashFlag, CssValueOrigin,
    CssValueTokenRef,
};
use crate::numeric::{CssNumeric, CssTime};
use crate::values::common::CssSidePair;
use crate::values::ui::CssCaretColor;

macro_rules! keywords {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(#[$imeta:meta])* $initial:ident,
            $($(#[$vmeta:meta])* $variant:ident),+
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum $name {
            $(#[$imeta])*
            #[default]
            $initial,
            $($(#[$vmeta])* $variant),+
        }
    };
}
keywords!(
    /// The authored `caret-animation` keyword.
    CssCaretAnimation {
        /// `auto`: the user agent decides whether and how the caret animates.
        Auto,
        /// `manual`: the user agent does not animate the caret.
        Manual
    }
);
keywords!(
    /// The authored `caret-shape` keyword.
    CssCaretShape {
        /// `auto`: the user agent chooses the caret shape.
        Auto,
        /// `bar`: the caret is a thin bar at the insertion point.
        Bar,
        /// `block`: the caret is a rectangle overlapping the next visible character.
        Block,
        /// `underscore`: the caret is a thin line under the next visible character.
        Underscore
    }
);
keywords!(
    /// The authored `interactivity` keyword.
    CssInteractivity {
        /// `auto`: inertness is determined by the host language.
        Auto,
        /// `inert`: the element is inert.
        Inert
    }
);
keywords!(
    /// The authored `appearance` keyword.
    ///
    /// The compatibility keywords of the specification's `<compat-auto>` and
    /// `<compat-special>` lists remain distinct specified values.
    CssAppearance {
        /// `none`: the element is rendered by the usual CSS rules and widgets use their primitive appearance.
        None,
        /// `auto`: widgets are rendered with their native appearance and other elements as for `none`.
        Auto,
        /// `base`: widgets that have a base appearance use it; other elements are rendered as for `none` or `auto`.
        Base,
        /// `base-select`: a select element or its picker uses the base appearance; other elements are rendered as for `auto`.
        BaseSelect,
        /// `searchfield`: compatibility keyword with the same effect as `auto`.
        Searchfield,
        /// `textarea`: compatibility keyword with the same effect as `auto`.
        Textarea,
        /// `checkbox`: compatibility keyword with the same effect as `auto`.
        Checkbox,
        /// `radio`: compatibility keyword with the same effect as `auto`.
        Radio,
        /// `menulist`: compatibility keyword with the same effect as `auto`.
        Menulist,
        /// `listbox`: compatibility keyword with the same effect as `auto`.
        Listbox,
        /// `meter`: compatibility keyword with the same effect as `auto`.
        Meter,
        /// `progress-bar`: compatibility keyword with the same effect as `auto`.
        ProgressBar,
        /// `button`: compatibility keyword with the same effect as `auto`.
        Button,
        /// `textfield`: special compatibility keyword with the same effect as `auto`, which the host language may also use for the element's native appearance.
        Textfield,
        /// `menulist-button`: special compatibility keyword with the same effect as `auto`, which the host language may also use for the element's native appearance.
        MenulistButton
    }
);

/// Accent selection stays symbolic until the style owner has context.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum CssAccentColor {
    /// `auto`: the user agent chooses the accent color.
    #[default]
    Auto,
    /// An authored accent `<color>`.
    Color(Box<crate::color::CssColor>),
}
impl CssAccentColor {
    /// Returns the authored accent color, or `None` for `auto`.
    pub fn color(&self) -> Option<&crate::color::CssColor> {
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
    /// Builds a `caret` value from its optional parts.
    ///
    /// Returns `None` when all three parts are omitted.
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
    /// Returns the authored caret color, or `None` when it was omitted.
    pub const fn color(&self) -> Option<&CssCaretColor> {
        self.color.as_ref()
    }
    /// Returns the authored caret animation, or `None` when it was omitted.
    pub const fn animation(&self) -> Option<CssCaretAnimation> {
        self.animation
    }
    /// Returns the authored caret shape, or `None` when it was omitted.
    pub const fn shape(&self) -> Option<CssCaretShape> {
        self.shape
    }
}

/// Normal is symbolic; ordinary times are signed and calculations remain authored.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub enum CssInterestDelayValue {
    /// `normal`: the delay is determined by the user agent.
    #[default]
    Normal,
    /// A signed `<time>` delay, or an unresolved calculation.
    Time(CssNumeric<CssTime>),
}
impl PartialEq for CssInterestDelayValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::Time(left), Self::Time(right)) => left == right,
            _ => false,
        }
    }
}
impl Eq for CssInterestDelayValue {}
impl CssInterestDelayValue {
    /// Returns the authored time, or `None` for `normal`.
    pub fn time(&self) -> Option<&CssNumeric<CssTime>> {
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
crate::values::common::common_values::side_element!(
    CssInterestDelayValue,
    same = |a, b, _context| Ok(a.specified_value_eq(b))
);
crate::values::common::common_values::side_pair_serialization!(CssInterestDelayValue);

impl CssSidePair<CssInterestDelayValue> {
    /// Composes `interest-delay` values, rejecting a time that an implicit close recovered, and
    /// retains whether the second value was authored.
    ///
    /// # Errors
    ///
    /// Returns [`crate::numeric::CssNumericConstructionError`] when a time was closed implicitly.
    pub fn try_new(
        start: CssInterestDelayValue,
        end: Option<CssInterestDelayValue>,
    ) -> Result<Self, crate::numeric::CssNumericConstructionError> {
        for value in std::iter::once(&start).chain(end.iter()) {
            if let Some(time) = value.time() {
                time.ensure_closed()?;
            }
        }
        Ok(Self::from_parts(start, end))
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
    /// Checks one component value as an ID-selector token.
    ///
    /// # Errors
    ///
    /// Returns an error when the component is not a hash token with the ID flag.
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
    /// Builds an ID from decoded identifier text, re-escaping it as needed.
    ///
    /// # Errors
    ///
    /// Returns an error when `decoded` is not a valid identifier.
    pub fn try_new(decoded: &str) -> Result<Self, CssComponentValueError> {
        let ident = CssComponentValue::try_ident(decoded)?;
        let values =
            crate::component::CssComponentValues::try_new(vec![ident], Default::default())?;
        let css = values.serialize_tokens(Default::default())?;
        Self::try_from_component(CssComponentValue::try_token(&format!("#{}", css.as_css()))?)
    }
    /// Returns the decoded ID text without the leading `#`.
    pub fn as_str(&self) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::Hash { value, .. }) =
            self.component.view()
        else {
            unreachable!("checked ID")
        };
        value
    }
    /// Borrows the retained ID-selector token component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Returns the original provenance of the ID token.
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
    /// Builds a target name from decoded text.
    ///
    /// # Errors
    ///
    /// Returns an error when the string cannot form a string token, starts with an underscore, or was closed implicitly.
    pub fn try_new(decoded: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_string(decoded)?)
    }
    /// Checks one component value as a conforming target name.
    ///
    /// # Errors
    ///
    /// Returns an error when the component is not a string token, its text starts with an underscore, or the string was terminated implicitly.
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
    /// Returns the decoded target name.
    pub fn as_str(&self) -> &str {
        string(&self.component)
    }
    /// Borrows the retained string token component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Returns the original provenance of the string token.
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
    /// Returns the decoded target name.
    pub fn as_str(&self) -> &str {
        string(&self.component)
    }
    /// Borrows the retained string token component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Returns the original provenance of the string token.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}
/// A directional focus navigation target frame.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNavigationTarget {
    /// `current`: the frame that the element is in.
    Current,
    /// `root`: the user agent targets the full window.
    Root,
    /// A conforming target name.
    Name(CssNavigationTargetName),
    /// A nonconforming target name retained from parser recovery.
    LegacyName(CssLegacyNavigationTargetName),
}
/// A symbolic directional focus input; this model performs no frame navigation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNavigation {
    /// `auto`: the user agent automatically determines which element to navigate the focus to.
    #[default]
    Auto,
    /// An element ID with an optional target frame.
    Id(Box<CssNavigationReference>),
}
/// An element ID and optional target frame for directional focus navigation.
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
    /// Returns the referenced element ID.
    pub const fn id(&self) -> &CssNavigationId {
        &self.id
    }
    /// Returns the authored target frame, or `None` when it was omitted.
    pub const fn target(&self) -> Option<&CssNavigationTarget> {
        self.target.as_ref()
    }
}
impl CssNavigation {
    /// Returns the reference, or `None` for `auto`.
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
