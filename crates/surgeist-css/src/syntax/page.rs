//! Authored Page selectors and distinct Page/margin descriptor domains.
use crate::{
    CssComponentValues, CssDeclaration, CssDeclarationList, CssDescriptorOccurrence,
    CssGlobalKeyword, CssImportance, CssParsedOrigin, CssSourcePosition, CssSpecifiedLength,
    CssSpecifiedNonNegativeLength,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssPagePseudo {
    Left,
    Right,
    First,
    Blank,
    Recto,
    Verso,
}
impl CssPagePseudo {
    #[must_use]
    pub const fn specificity(self) -> CssPageSpecificity {
        match self {
            Self::First | Self::Blank => CssPageSpecificity::new(0, 1, 0),
            _ => CssPageSpecificity::new(0, 0, 1),
        }
    }
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::First => "first",
            Self::Blank => "blank",
            Self::Recto => "recto",
            Self::Verso => "verso",
        }
    }
}
/// Page specificity (named, first/blank, side); distinct from element specificity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssPageSpecificity {
    named: usize,
    first_or_blank: usize,
    side: usize,
}
impl CssPageSpecificity {
    #[must_use]
    pub const fn new(named: usize, first_or_blank: usize, side: usize) -> Self {
        Self {
            named,
            first_or_blank,
            side,
        }
    }
    #[must_use]
    pub const fn named(self) -> usize {
        self.named
    }
    #[must_use]
    pub const fn first_or_blank(self) -> usize {
        self.first_or_blank
    }
    #[must_use]
    pub const fn side(self) -> usize {
        self.side
    }
}
/// One nonempty named/pseudo Page selector, retaining compound order and provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageSelector {
    name: Option<crate::CssIdent>,
    pseudos: Vec<CssPagePseudo>,
    origin: Option<CssParsedOrigin>,
}
impl CssPageSelector {
    #[must_use]
    pub fn try_new(name: Option<crate::CssIdent>, pseudos: Vec<CssPagePseudo>) -> Option<Self> {
        (name.is_some() || !pseudos.is_empty()).then_some(Self {
            name,
            pseudos,
            origin: None,
        })
    }
    pub(crate) fn from_parsed(
        name: Option<crate::CssIdent>,
        pseudos: Vec<CssPagePseudo>,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            name,
            pseudos,
            origin: Some(origin),
        }
    }
    #[must_use]
    pub const fn name(&self) -> Option<&crate::CssIdent> {
        self.name.as_ref()
    }
    #[must_use]
    pub fn pseudos(&self) -> &[CssPagePseudo] {
        &self.pseudos
    }
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
    #[must_use]
    pub fn specificity(&self) -> CssPageSpecificity {
        CssPageSpecificity {
            named: usize::from(self.name.is_some()),
            first_or_blank: self
                .pseudos
                .iter()
                .filter(|p| matches!(p, CssPagePseudo::First | CssPagePseudo::Blank))
                .count(),
            side: self
                .pseudos
                .iter()
                .filter(|p| !matches!(p, CssPagePseudo::First | CssPagePseudo::Blank))
                .count(),
        }
    }
}
/// The Page prelude; empty is distinct from a nonempty selector list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssPageSelectorList {
    selectors: Vec<CssPageSelector>,
    origin: Option<CssParsedOrigin>,
}
impl CssPageSelectorList {
    #[must_use]
    pub fn new(selectors: Vec<CssPageSelector>) -> Self {
        Self {
            selectors,
            origin: None,
        }
    }
    pub(crate) fn from_parsed(selectors: Vec<CssPageSelector>, origin: CssParsedOrigin) -> Self {
        Self {
            selectors,
            origin: Some(origin),
        }
    }
    #[must_use]
    pub fn selectors(&self) -> &[CssPageSelector] {
        &self.selectors
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty()
    }
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssPageDescriptorKind {
    Size,
    PageOrientation,
    Marks,
    Bleed,
}
impl CssPageDescriptorKind {
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::PageOrientation => "page-orientation",
            Self::Marks => "marks",
            Self::Bleed => "bleed",
        }
    }
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        [Self::Size, Self::PageOrientation, Self::Marks, Self::Bleed]
            .into_iter()
            .find(|kind| name.eq_ignore_ascii_case(kind.css_name()))
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageOrientation {
    Portrait,
    Landscape,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageOutputOrientation {
    Upright,
    RotateLeft,
    RotateRight,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageSize {
    A5,
    A4,
    A3,
    B5,
    B4,
    JisB5,
    JisB4,
    Letter,
    Legal,
    Ledger,
}
impl CssPageSize {
    pub(crate) const fn css_name(self) -> &'static str {
        match self {
            Self::A5 => "a5",
            Self::A4 => "a4",
            Self::A3 => "a3",
            Self::B5 => "b5",
            Self::B4 => "b4",
            Self::JisB5 => "jis-b5",
            Self::JisB4 => "jis-b4",
            Self::Letter => "letter",
            Self::Legal => "legal",
            Self::Ledger => "ledger",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageSizeValue {
    Auto,
    Dimensions(
        CssSpecifiedNonNegativeLength,
        Option<CssSpecifiedNonNegativeLength>,
    ),
    Named(CssPageNamedSize),
}
/// A named page size and/or orientation; at least one component is present.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageNamedSize {
    size: Option<CssPageSize>,
    orientation: Option<CssPageOrientation>,
}
impl CssPageNamedSize {
    #[must_use]
    pub fn try_new(
        size: Option<CssPageSize>,
        orientation: Option<CssPageOrientation>,
    ) -> Option<Self> {
        (size.is_some() || orientation.is_some()).then_some(Self { size, orientation })
    }
    #[must_use]
    pub const fn size(&self) -> Option<CssPageSize> {
        self.size
    }
    #[must_use]
    pub const fn orientation(&self) -> Option<CssPageOrientation> {
        self.orientation
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssPageMarks {
    crop: bool,
    cross: bool,
}
impl CssPageMarks {
    pub(crate) const fn new(crop: bool, cross: bool) -> Self {
        Self { crop, cross }
    }
    #[must_use]
    pub const fn crop(self) -> bool {
        self.crop
    }
    #[must_use]
    pub const fn cross(self) -> bool {
        self.cross
    }
}
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageBleed {
    Auto,
    Length(CssSpecifiedLength),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PageValueData {
    Size(CssPageSizeValue),
    PageOrientation(CssPageOutputOrientation),
    Marks(CssPageMarks),
    Bleed(CssPageBleed),
    Global(CssGlobalKeyword),
    Pending(crate::CssSubstitutionDependentValue),
}
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageDescriptorValueRef<'a> {
    Size(&'a CssPageSizeValue),
    PageOrientation(CssPageOutputOrientation),
    Marks(CssPageMarks),
    Bleed(&'a CssPageBleed),
    Global(CssGlobalKeyword),
    Pending(&'a crate::CssSubstitutionDependentValue),
}
/// Checked domain-specific value. Every constructor is parser-owned; raw edits retain value-only origins.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageDescriptorValue {
    kind: CssPageDescriptorKind,
    pub(crate) data: PageValueData,
    components: CssComponentValues,
    origin: CssParsedOrigin,
}
impl CssPageDescriptorValue {
    pub(crate) fn from_parsed(
        kind: CssPageDescriptorKind,
        data: PageValueData,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            kind,
            data,
            components,
            origin,
        }
    }
    #[must_use]
    pub const fn kind(&self) -> CssPageDescriptorKind {
        self.kind
    }
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
    #[must_use]
    pub const fn view(&self) -> CssPageDescriptorValueRef<'_> {
        match &self.data {
            PageValueData::Size(v) => CssPageDescriptorValueRef::Size(v),
            PageValueData::PageOrientation(v) => CssPageDescriptorValueRef::PageOrientation(*v),
            PageValueData::Marks(v) => CssPageDescriptorValueRef::Marks(*v),
            PageValueData::Bleed(v) => CssPageDescriptorValueRef::Bleed(v),
            PageValueData::Global(v) => CssPageDescriptorValueRef::Global(*v),
            PageValueData::Pending(v) => CssPageDescriptorValueRef::Pending(v),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageDescriptor {
    inner: std::sync::Arc<PageDescriptorData>,
}
#[derive(Debug, PartialEq)]
struct PageDescriptorData {
    value: CssPageDescriptorValue,
    importance: CssImportance,
    occurrence: Option<CssDescriptorOccurrence<CssPageDescriptorValue>>,
}
impl CssPageDescriptor {
    #[must_use]
    pub fn new(value: CssPageDescriptorValue, importance: CssImportance) -> Self {
        Self {
            inner: std::sync::Arc::new(PageDescriptorData {
                value,
                importance,
                occurrence: None,
            }),
        }
    }
    pub(crate) fn from_parsed(
        value: CssPageDescriptorValue,
        importance: CssImportance,
        name: CssParsedOrigin,
    ) -> Self {
        let occurrence = CssDescriptorOccurrence::from_parsed(
            value.clone(),
            name,
            value.origin.clone(),
            value.components.clone(),
        );
        Self {
            inner: std::sync::Arc::new(PageDescriptorData {
                value,
                importance,
                occurrence: Some(occurrence),
            }),
        }
    }
    #[must_use]
    pub fn value(&self) -> &CssPageDescriptorValue {
        &self.inner.value
    }
    #[must_use]
    pub fn importance(&self) -> CssImportance {
        self.inner.importance
    }
    /// Changes selected priority while retaining the original named/value occurrence.
    #[must_use]
    pub fn with_importance(&self, importance: CssImportance) -> Self {
        Self {
            inner: std::sync::Arc::new(PageDescriptorData {
                value: self.inner.value.clone(),
                importance,
                occurrence: self.inner.occurrence.clone(),
            }),
        }
    }
    #[must_use]
    pub fn occurrence(&self) -> Option<&CssDescriptorOccurrence<CssPageDescriptorValue>> {
        self.inner.occurrence.as_ref()
    }
}
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageDeclaration {
    Property(CssDeclaration),
    Descriptor(CssPageDescriptor),
}
/// Checked Page declarations; ordered authored inventory and ordinary property projection stay distinct.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageDeclarationBlock {
    occurrences: Vec<CssPageDeclaration>,
    properties: CssDeclarationList,
}
impl CssPageDeclarationBlock {
    pub(crate) fn from_parsed(occurrences: Vec<CssPageDeclaration>) -> Self {
        let properties = CssDeclarationList::new(
            occurrences
                .iter()
                .filter_map(|entry| match entry {
                    CssPageDeclaration::Property(v) => Some(v.clone()),
                    _ => None,
                })
                .collect(),
        );
        Self {
            occurrences,
            properties,
        }
    }
    #[must_use]
    pub fn occurrences(&self) -> &[CssPageDeclaration] {
        &self.occurrences
    }
    #[must_use]
    pub const fn properties(&self) -> &CssDeclarationList {
        &self.properties
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.occurrences.is_empty()
    }
    #[must_use]
    pub fn effective_descriptor(&self, kind: CssPageDescriptorKind) -> Option<&CssPageDescriptor> {
        self.occurrences
            .iter()
            .rev()
            .filter_map(|entry| match entry {
                CssPageDeclaration::Descriptor(v) if v.value().kind() == kind => Some(v),
                _ => None,
            })
            .find(|v| v.importance() == CssImportance::Important)
            .or_else(|| {
                self.occurrences.iter().rev().find_map(|entry| match entry {
                    CssPageDeclaration::Descriptor(v) if v.value().kind() == kind => Some(v),
                    _ => None,
                })
            })
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssMarginBox {
    TopLeftCorner,
    TopLeft,
    TopCenter,
    TopRight,
    TopRightCorner,
    BottomLeftCorner,
    BottomLeft,
    BottomCenter,
    BottomRight,
    BottomRightCorner,
    LeftTop,
    LeftMiddle,
    LeftBottom,
    RightTop,
    RightMiddle,
    RightBottom,
}
impl CssMarginBox {
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::TopLeftCorner => "top-left-corner",
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::TopRightCorner => "top-right-corner",
            Self::BottomLeftCorner => "bottom-left-corner",
            Self::BottomLeft => "bottom-left",
            Self::BottomCenter => "bottom-center",
            Self::BottomRight => "bottom-right",
            Self::BottomRightCorner => "bottom-right-corner",
            Self::LeftTop => "left-top",
            Self::LeftMiddle => "left-middle",
            Self::LeftBottom => "left-bottom",
            Self::RightTop => "right-top",
            Self::RightMiddle => "right-middle",
            Self::RightBottom => "right-bottom",
        }
    }
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        [
            Self::TopLeftCorner,
            Self::TopLeft,
            Self::TopCenter,
            Self::TopRight,
            Self::TopRightCorner,
            Self::BottomLeftCorner,
            Self::BottomLeft,
            Self::BottomCenter,
            Self::BottomRight,
            Self::BottomRightCorner,
            Self::LeftTop,
            Self::LeftMiddle,
            Self::LeftBottom,
            Self::RightTop,
            Self::RightMiddle,
            Self::RightBottom,
        ]
        .into_iter()
        .find(|v| name.eq_ignore_ascii_case(v.css_name()))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssMarginDeclarationBlock {
    properties: CssDeclarationList,
}
impl CssMarginDeclarationBlock {
    pub(crate) fn from_parsed(properties: CssDeclarationList) -> Self {
        Self { properties }
    }
    #[must_use]
    pub const fn properties(&self) -> &CssDeclarationList {
        &self.properties
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssMarginRule {
    name: CssMarginBox,
    declarations: CssMarginDeclarationBlock,
    position: Option<CssSourcePosition>,
}
impl CssMarginRule {
    #[must_use]
    pub const fn new(name: CssMarginBox, declarations: CssMarginDeclarationBlock) -> Self {
        Self {
            name,
            declarations,
            position: None,
        }
    }
    pub(crate) const fn from_parsed(
        name: CssMarginBox,
        declarations: CssMarginDeclarationBlock,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            declarations,
            position: Some(position),
        }
    }
    #[must_use]
    pub const fn name(&self) -> CssMarginBox {
        self.name
    }
    #[must_use]
    pub const fn declarations(&self) -> &CssMarginDeclarationBlock {
        &self.declarations
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageBody {
    declarations: CssPageDeclarationBlock,
    margin_rules: Vec<CssMarginRule>,
}
impl CssPageBody {
    #[must_use]
    pub const fn new(
        declarations: CssPageDeclarationBlock,
        margin_rules: Vec<CssMarginRule>,
    ) -> Self {
        Self {
            declarations,
            margin_rules,
        }
    }
    #[must_use]
    pub const fn declarations(&self) -> &CssPageDeclarationBlock {
        &self.declarations
    }
    #[must_use]
    pub fn margin_rules(&self) -> &[CssMarginRule] {
        &self.margin_rules
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageRule {
    selectors: CssPageSelectorList,
    body: CssPageBody,
    position: CssSourcePosition,
}
impl CssPageRule {
    pub(crate) const fn new(
        selectors: CssPageSelectorList,
        body: CssPageBody,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            body,
            position,
        }
    }
    #[must_use]
    pub const fn selectors(&self) -> &CssPageSelectorList {
        &self.selectors
    }
    #[must_use]
    pub const fn declarations(&self) -> &CssPageDeclarationBlock {
        self.body.declarations()
    }
    #[must_use]
    pub fn margin_rules(&self) -> &[CssMarginRule] {
        self.body.margin_rules()
    }
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
    #[must_use]
    pub fn with_body(&self, body: CssPageBody) -> Self {
        Self {
            body,
            ..self.clone()
        }
    }
    #[must_use]
    pub fn with_selectors(&self, selectors: CssPageSelectorList) -> Self {
        Self {
            selectors,
            ..self.clone()
        }
    }
}

/// Typed failure at the Page/margin domain reconstruction boundary.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssPageBlockErrorKind {
    PropertyNotApplicable,
    InvalidPropertyValue,
    Resource(crate::CssSpecifiedValueSerializationError),
}
#[derive(Clone, Debug)]
pub struct CssPageBlockError {
    kind: CssPageBlockErrorKind,
    ordinal: Option<usize>,
    declaration: Option<CssDeclaration>,
}
impl CssPageBlockError {
    #[must_use]
    pub const fn kind(&self) -> &CssPageBlockErrorKind {
        &self.kind
    }
    #[must_use]
    pub const fn ordinal(&self) -> Option<usize> {
        self.ordinal
    }
    #[must_use]
    pub const fn declaration(&self) -> Option<&CssDeclaration> {
        self.declaration.as_ref()
    }
}
impl std::fmt::Display for CssPageBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid Page/margin block at {:?}: {:?}",
            self.ordinal, self.kind
        )
    }
}
impl std::error::Error for CssPageBlockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            CssPageBlockErrorKind::Resource(v) => Some(v),
            _ => None,
        }
    }
}
impl From<crate::CssSpecifiedValueSerializationError> for CssPageBlockError {
    fn from(error: crate::CssSpecifiedValueSerializationError) -> Self {
        Self {
            kind: CssPageBlockErrorKind::Resource(error),
            ordinal: None,
            declaration: None,
        }
    }
}
fn check_property(
    value: CssDeclaration,
    ordinal: usize,
) -> Result<CssDeclaration, CssPageBlockError> {
    let issue = if value
        .known()
        .is_some_and(|v| !crate::parser::is_page_margin_property(v.property()))
        || value.svg_glyph_orientation_vertical().is_some()
    {
        Some(CssPageBlockErrorKind::PropertyNotApplicable)
    } else if crate::parser::page_declaration_violation(value.body(), value.value_components())
        .is_some()
    {
        Some(CssPageBlockErrorKind::InvalidPropertyValue)
    } else {
        None
    };
    if let Some(kind) = issue {
        return Err(CssPageBlockError {
            kind,
            ordinal: Some(ordinal),
            declaration: Some(value),
        });
    }
    Ok(value.into_page_context())
}
impl CssPageDeclarationBlock {
    pub fn try_new(occurrences: Vec<CssPageDeclaration>) -> Result<Self, CssPageBlockError> {
        Self::try_new_with_limits(
            occurrences,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_new_with_limits(
        occurrences: Vec<CssPageDeclaration>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssPageBlockError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer.node()?;
        let mut admitted = Vec::new();
        for (ordinal, value) in occurrences.into_iter().enumerate() {
            admitted.try_reserve(1).map_err(|_| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            let value = match value {
                CssPageDeclaration::Property(v) => {
                    let v = check_property(v, ordinal)?;
                    crate::declaration_serialization::append_authored_declaration(
                        v.body(),
                        v.value_components(),
                        v.importance(),
                        &mut writer,
                    )?;
                    CssPageDeclaration::Property(v)
                }
                CssPageDeclaration::Descriptor(v) => {
                    writer.node()?;
                    v.value().append_value(&mut writer)?;
                    CssPageDeclaration::Descriptor(v)
                }
            };
            admitted.push(value);
        }
        Ok(Self::from_parsed(admitted))
    }
}
impl CssMarginDeclarationBlock {
    pub fn try_new(properties: Vec<CssDeclaration>) -> Result<Self, CssPageBlockError> {
        Self::try_new_with_limits(
            properties,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_new_with_limits(
        properties: Vec<CssDeclaration>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssPageBlockError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer.node()?;
        let mut admitted = Vec::new();
        for (ordinal, value) in properties.into_iter().enumerate() {
            let value = check_property(value, ordinal)?;
            crate::declaration_serialization::append_authored_declaration(
                value.body(),
                value.value_components(),
                value.importance(),
                &mut writer,
            )?;
            admitted.try_reserve(1).map_err(|_| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            admitted.push(value);
        }
        Ok(Self {
            properties: CssDeclarationList::new(admitted),
        })
    }
    pub fn try_specified_properties(
        &self,
    ) -> Result<crate::CssSpecifiedDeclarationBlock, crate::CssDeclarationBlockError> {
        self.try_specified_properties_with_limits(
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_specified_properties_with_limits(
        &self,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<crate::CssSpecifiedDeclarationBlock, crate::CssDeclarationBlockError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        crate::CssSpecifiedDeclarationBlock::from_margin_sources(
            self.properties.iter(),
            &mut writer,
        )
    }
    pub fn serialize_cssom(&self) -> Result<String, crate::CssDeclarationBlockError> {
        self.serialize_cssom_with_limits(crate::CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_cssom_with_limits(
        &self,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<String, crate::CssDeclarationBlockError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        let block = crate::CssSpecifiedDeclarationBlock::from_margin_sources(
            self.properties.iter(),
            &mut writer,
        )?;
        block.append_cssom(&mut writer)?;
        Ok(writer.css)
    }
}
