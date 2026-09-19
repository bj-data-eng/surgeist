//! Ordinary authored container property values and programmatic canonical emission.
use crate::component_values::CssCanonicalBuilder;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueLimits,
    CssComponentValues, CssContainerName, CssSerializedValue, CssValueOrigin,
};

/// The six ordinary container-type states, without computed containment behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssContainerType {
    Normal,
    Size,
    InlineSize,
    ScrollState,
    SizeScrollState,
    InlineSizeScrollState,
}
impl CssContainerType {
    fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Normal => &["normal"],
            Self::Size => &["size"],
            Self::InlineSize => &["inline-size"],
            Self::ScrollState => &["scroll-state"],
            Self::SizeScrollState => &["size", "scroll-state"],
            Self::InlineSizeScrollState => &["inline-size", "scroll-state"],
        }
    }
    fn projection(&self) -> Projection<'_> {
        Projection {
            names: None,
            kind: Some(*self),
        }
    }
}

/// A checked nonempty ordered list; case and repeated names remain significant.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssContainerNameList {
    names: Vec<CssContainerName>,
}
impl CssContainerNameList {
    #[must_use]
    pub fn try_new(names: Vec<CssContainerName>) -> Option<Self> {
        (!names.is_empty()).then_some(Self { names })
    }
    #[must_use]
    pub fn names(&self) -> &[CssContainerName] {
        &self.names
    }
}

/// The ordinary container-name domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssContainerNames {
    None,
    Names(CssContainerNameList),
}
impl CssContainerNames {
    fn projection(&self) -> Projection<'_> {
        Projection {
            names: Some(self),
            kind: None,
        }
    }
}

/// A checked ordinary container shorthand. Omitted type is represented by Normal.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssContainer {
    names: CssContainerNames,
    container_type: CssContainerType,
}
impl CssContainer {
    #[must_use]
    pub const fn new(names: CssContainerNames, container_type: CssContainerType) -> Self {
        Self {
            names,
            container_type,
        }
    }
    #[must_use]
    pub const fn names(&self) -> &CssContainerNames {
        &self.names
    }
    #[must_use]
    pub const fn container_type(&self) -> CssContainerType {
        self.container_type
    }
    fn projection(&self) -> Projection<'_> {
        Projection {
            names: Some(&self.names),
            kind: (self.container_type != CssContainerType::Normal).then_some(self.container_type),
        }
    }
}

#[derive(Clone, Copy)]
enum Piece<'a> {
    Ident(&'a str),
    Space,
    Slash,
}
struct Projection<'a> {
    names: Option<&'a CssContainerNames>,
    kind: Option<CssContainerType>,
}
fn emission_error(kind: CssComponentValueErrorKind) -> CssComponentValueError {
    CssComponentValueError::new(kind, CssValueOrigin::Programmatic)
}
impl Projection<'_> {
    fn visit(
        &self,
        mut push: impl FnMut(Piece<'_>) -> Result<(), CssComponentValueError>,
    ) -> Result<(), CssComponentValueError> {
        if let Some(names) = self.names {
            match names {
                CssContainerNames::None => push(Piece::Ident("none"))?,
                CssContainerNames::Names(list) => {
                    for (index, name) in list.names().iter().enumerate() {
                        if index != 0 {
                            push(Piece::Space)?;
                        }
                        push(Piece::Ident(name.as_str()))?;
                    }
                }
            }
            if self.kind.is_some() {
                push(Piece::Space)?;
                push(Piece::Slash)?;
                push(Piece::Space)?;
            }
        }
        if let Some(kind) = self.kind {
            for (index, keyword) in kind.keywords().iter().enumerate() {
                if index != 0 {
                    push(Piece::Space)?;
                }
                push(Piece::Ident(keyword))?;
            }
        }
        Ok(())
    }
    fn components(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssComponentValues, CssComponentValueError> {
        // Raw decoded bytes plus fixed separators are a lower bound on escaped output.
        // Check it and token count before allocating or escaping caller-sized names.
        let mut count = 0usize;
        let mut bytes = 0usize;
        self.visit(|piece| {
            count = count
                .checked_add(1)
                .ok_or_else(|| emission_error(CssComponentValueErrorKind::CapacityOverflow))?;
            bytes = bytes
                .checked_add(match piece {
                    Piece::Ident(name) => name.len(),
                    Piece::Space | Piece::Slash => 1,
                })
                .ok_or_else(|| emission_error(CssComponentValueErrorKind::CapacityOverflow))?;
            if count > limits.max_components() {
                return Err(emission_error(CssComponentValueErrorKind::ComponentLimit));
            }
            if bytes > limits.max_css_bytes() {
                return Err(emission_error(CssComponentValueErrorKind::ByteLimit));
            }
            Ok(())
        })?;
        let mut items = Vec::new();
        self.visit(|piece| {
            items.push(match piece {
                Piece::Ident(name) => CssComponentValue::try_ident(name)?,
                Piece::Space => CssComponentValue::try_token(" ")?,
                Piece::Slash => CssComponentValue::try_token("/")?,
            });
            Ok(())
        })?;
        CssComponentValues::try_new_with_limits(items, limits)
    }
    fn serialize(&self, max_bytes: usize) -> Result<CssSerializedValue, CssComponentValueError> {
        let limits = CssComponentValueLimits::try_new(0, usize::MAX, max_bytes)
            .expect("leaf depth is valid");
        let components = self.components(limits)?;
        let mut builder = CssCanonicalBuilder::new(max_bytes);
        builder.push_components(components.items())?;
        builder.finish()
    }
}
macro_rules! emission {
    ($($ty:ty),+ $(,)?) => { $(impl $ty {
        /// Emits canonical components with explicitly programmatic token origins.
        pub fn to_components(&self) -> Result<CssComponentValues, CssComponentValueError> {
            self.to_components_with_limits(CssComponentValueLimits::default())
        }
        /// Emits canonical components under checked token and escaped-byte limits.
        pub fn to_components_with_limits(&self, limits: CssComponentValueLimits) -> Result<CssComponentValues, CssComponentValueError> {
            self.projection().components(limits)
        }
        /// Canonical semantic spelling; parsed declaration wrappers retain authored spelling.
        pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> { self.serialize_with_limit(usize::MAX) }
        /// Bounds canonical output, including identifier escapes and separators.
        pub fn serialize_with_limit(&self, max_bytes: usize) -> Result<CssSerializedValue, CssComponentValueError> { self.projection().serialize(max_bytes) }
    })+ };
}
emission!(CssContainerType, CssContainerNames, CssContainer);
