//! Authored scroll-state queries. Snapshots, direction resolution and truth are external.
use crate::component_values::CssCanonicalBuilder;
use crate::supports::SupportsLexical;
use crate::*;

/// One admitted scroll-state query node with its original lexical region.
/// Construct through [`CssContainerCondition::try_from_components`] or stylesheet parsing.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerScrollQuery {
    kind: Box<CssContainerScrollQueryKind>,
    lexical: SupportsLexical,
}

/// Inspectable authored logic; a kind alone cannot construct an admitted query.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerScrollQueryKind {
    Feature(CssContainerScrollFeature),
    Parenthesized(Box<CssContainerScrollQuery>),
    Not(Box<CssContainerScrollQuery>),
    And(CssContainerScrollQueryList),
    Or(CssContainerScrollQueryList),
    GeneralEnclosed(CssContainerGeneralEnclosed),
}

/// At least two operands of one homogeneous logical production.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerScrollQueryList {
    queries: Vec<CssContainerScrollQuery>,
}
impl CssContainerScrollQueryList {
    pub(crate) fn new(queries: Vec<CssContainerScrollQuery>) -> Self {
        debug_assert!(queries.len() >= 2);
        Self { queries }
    }
    pub fn queries(&self) -> &[CssContainerScrollQuery] {
        &self.queries
    }
}

/// The four discrete features of a scroll-state query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainerScrollFeatureKind {
    Stuck,
    Snapped,
    Scrollable,
    Scrolled,
}
impl CssContainerScrollFeatureKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Stuck => "stuck",
            Self::Snapped => "snapped",
            Self::Scrollable => "scrollable",
            Self::Scrolled => "scrolled",
        }
    }
    pub(crate) fn parse(name: &str) -> Option<Self> {
        [Self::Stuck, Self::Snapped, Self::Scrollable, Self::Scrolled]
            .into_iter()
            .find(|kind| name.eq_ignore_ascii_case(kind.name()))
    }
}

/// Coupled feature/value domains; scrollable and scrolled share direction syntax.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerScrollFeature {
    Boolean(CssContainerScrollFeatureKind),
    Stuck(CssContainerStuckValue),
    Snapped(CssContainerSnappedValue),
    Scrollable(CssContainerScrollDirectionValue),
    Scrolled(CssContainerScrollDirectionValue),
}

macro_rules! keywords {
    ($(#[$doc:meta])* $name:ident { $($variant:ident => $spelling:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $spelling),+ }
            }
            pub(crate) fn parse(name: &str) -> Option<Self> {
                $(if name.eq_ignore_ascii_case($spelling) { return Some(Self::$variant); })+
                None
            }
        }
    };
}
keywords!(
    /// Authored stuck edges. Logical edges remain unresolved.
    CssContainerStuckKeyword {
        None => "none", Top => "top", Right => "right", Bottom => "bottom", Left => "left",
        BlockStart => "block-start", InlineStart => "inline-start", BlockEnd => "block-end", InlineEnd => "inline-end",
    }
);
keywords!(
    /// Authored snapping axes, including simultaneous snapping on both axes.
    CssContainerSnappedKeyword {
        None => "none", X => "x", Y => "y", Block => "block", Inline => "inline", Both => "both",
    }
);
keywords!(
    /// Authored directions for scrollable and scrolled, without history or axis resolution.
    CssContainerScrollDirectionKeyword {
        None => "none", Top => "top", Right => "right", Bottom => "bottom", Left => "left",
        BlockStart => "block-start", InlineStart => "inline-start", BlockEnd => "block-end", InlineEnd => "inline-end",
        X => "x", Y => "y", Block => "block", Inline => "inline",
    }
);

#[derive(Clone, Debug, PartialEq)]
enum ScrollValue<K> {
    Keyword {
        keyword: K,
        components: CssComponentValues,
    },
    Pending(CssContainerPendingValue),
}
macro_rules! operand {
    ($(#[$doc:meta])* $name:ident, $view:ident, $keyword:ty, $domain:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name { value: ScrollValue<$keyword> }
        #[derive(Clone, Copy, Debug)]
        #[non_exhaustive]
        pub enum $view<'a> {
            Keyword($keyword),
            Pending(&'a CssContainerPendingValue),
        }
        impl $name {
            pub(crate) fn typed(keyword: $keyword, components: CssComponentValues) -> Self {
                Self { value: ScrollValue::Keyword { keyword, components } }
            }
            pub(crate) fn pending(components: CssComponentValues) -> Self {
                Self { value: ScrollValue::Pending(CssContainerPendingValue::new(CssContainerValueDomain::$domain, components)) }
            }
            pub fn view(&self) -> $view<'_> {
                match &self.value {
                    ScrollValue::Keyword { keyword, .. } => $view::Keyword(*keyword),
                    ScrollValue::Pending(value) => $view::Pending(value),
                }
            }
            pub fn components(&self) -> &CssComponentValues {
                match &self.value {
                    ScrollValue::Keyword { components, .. } => components,
                    ScrollValue::Pending(value) => value.components(),
                }
            }
            pub fn origin(&self) -> &CssValueOrigin {
                self.components().items().iter().find(|item| !crate::supports::trivia(item))
                    .expect("admitted keyword or pending operand").origin()
            }
            pub fn position(&self) -> Option<CssSourcePosition> {
                crate::media::parsed_position(self.origin())
            }
            pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
                self.serialize_with_limit(usize::MAX)
            }
            pub fn serialize_with_limit(&self, max_css_bytes: usize) -> Result<CssSerializedValue, CssComponentValueError> {
                let mut out = CssCanonicalBuilder::new(max_css_bytes);
                out.push_components(self.components().items())?;
                out.finish()
            }
        }
    };
}
operand!(
    /// A stuck keyword or a whole value awaiting substitution into that domain.
    CssContainerStuckValue, CssContainerStuckValueRef, CssContainerStuckKeyword, Stuck
);
operand!(
    /// A snapped keyword or a whole value awaiting substitution into that domain.
    CssContainerSnappedValue, CssContainerSnappedValueRef, CssContainerSnappedKeyword, Snapped
);
operand!(
    /// A direction keyword or a whole value awaiting substitution into that domain.
    CssContainerScrollDirectionValue, CssContainerScrollDirectionValueRef, CssContainerScrollDirectionKeyword, ScrollDirection
);

impl CssContainerScrollQuery {
    pub(crate) fn new(kind: CssContainerScrollQueryKind, lexical: SupportsLexical) -> Self {
        Self {
            kind: Box::new(kind),
            lexical,
        }
    }
    pub fn kind(&self) -> &CssContainerScrollQueryKind {
        &self.kind
    }
    pub fn components(&self) -> &[CssComponentValue] {
        self.lexical.items()
    }
    pub fn origin(&self) -> &CssValueOrigin {
        self.lexical.first_origin()
    }
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components())?;
        out.finish()
    }
}
