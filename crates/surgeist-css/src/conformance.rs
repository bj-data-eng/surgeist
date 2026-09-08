use crate::{CssErrorCode, CssFeatureId, CssKnownProperty};

#[expect(
    dead_code,
    reason = "private atomic implementation reconciliation view"
)]
#[derive(Clone, Copy, Debug)]
struct CssAtomicImplementationInventoryView {
    properties: &'static [crate::properties::PropertyImplementation],
    parser_owners: &'static [crate::parser::CssAtomicImplementationInventory],
}

#[expect(
    dead_code,
    reason = "reviewed directly against active atomic parser paths"
)]
static ATOMIC_IMPLEMENTATION_INVENTORY: CssAtomicImplementationInventoryView =
    CssAtomicImplementationInventoryView {
        properties: crate::properties::property_implementation_inventory(),
        parser_owners: crate::parser::atomic_implementation_inventories(),
    };

/// The grammar-family role of one support-catalog feature.
///
/// This classification identifies authored syntax; it does not enable parsing,
/// perform contextual resolution, or imply support for sibling productions.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssFeatureKind {
    /// An at-rule, qualified rule, or syntax-owned rule metadata production.
    Rule,
    /// A declaration or declaration-list production.
    Declaration,
    /// A descriptor within an owning at-rule.
    Descriptor,
    /// A shared authored-value production.
    Value,
    /// A recognized non-custom authored property production.
    Property,
    /// An explicitly parsed authored property alias with its own grammar and mapping.
    PropertyAlias,
    /// A selector production or finite selector spelling group.
    Selector,
    /// A media-query production or finite media-query spelling group.
    MediaQuery,
    /// A container-query production or finite container-query spelling group.
    ContainerQuery,
}

/// The bounded implementation state of one exact catalog production.
///
/// The three states are closed for I01: callers can distinguish a complete
/// production, a documented supported subset, and a recognized spelling that
/// is never retained as that production.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssSupportStatus {
    /// The complete identified production is implemented.
    Complete,
    /// Only the record's documented supported subset is implemented.
    Partial,
    /// The spelling is recognized but the production is not implemented.
    RecognizedUnsupported,
}

/// A stable identity for one immutable conformance source.
///
/// Source identities are registry-owned and cannot be forged by downstream
/// callers.
///
/// ```compile_fail
/// use surgeist_css::CssSpecificationSourceId;
///
/// let _ = CssSpecificationSourceId("O-CSS2");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssSpecificationSourceId(&'static str);

impl CssSpecificationSourceId {
    const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Returns the exact stable source identity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// The selected conformance-profile tier of a specification source.
///
/// A tier classifies provenance only. It never implies parser support.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssSpecificationTier {
    /// An official dated source in the Snapshot 2026 profile.
    Snapshot2026Official,
    /// A reliable dated source preserved by the Snapshot 2026 profile.
    Snapshot2026Reliable,
    /// A stable dated source preserved by the Snapshot 2026 profile.
    Snapshot2026Stable,
    /// An interoperability source preserved by the Snapshot 2026 profile.
    Snapshot2026Interop,
    /// A deliberately selected Surgeist extension or repository baseline.
    SurgeistExtension,
    /// A standards-track source outside the selected profile.
    LaterStandard,
}

/// Immutable provenance for one support-catalog record.
///
/// Exactly one of [`Self::url`] and [`Self::repository_provenance`] is present.
/// Values are catalog-owned and cannot be constructed by downstream callers.
///
/// ```compile_fail
/// use surgeist_css::{CssSpecificationSource, CssSpecificationTier};
///
/// let _ = CssSpecificationSource {
///     id: todo!(),
///     module: "CSS",
///     level: "3",
///     tier: CssSpecificationTier::LaterStandard,
///     url: None,
///     repository_provenance: None,
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssSpecificationSource {
    id: CssSpecificationSourceId,
    module: &'static str,
    level: &'static str,
    tier: CssSpecificationTier,
    url: Option<&'static str>,
    repository_provenance: Option<&'static str>,
}

impl CssSpecificationSource {
    const fn from_url(
        id: &'static str,
        module: &'static str,
        level: &'static str,
        tier: CssSpecificationTier,
        value: &'static str,
    ) -> Self {
        Self {
            id: CssSpecificationSourceId::new(id),
            module,
            level,
            tier,
            url: Some(value),
            repository_provenance: None,
        }
    }

    const fn from_repository(
        id: &'static str,
        module: &'static str,
        level: &'static str,
        value: &'static str,
    ) -> Self {
        Self {
            id: CssSpecificationSourceId::new(id),
            module,
            level,
            tier: CssSpecificationTier::SurgeistExtension,
            url: None,
            repository_provenance: Some(value),
        }
    }

    /// Returns the exact stable source identity.
    #[must_use]
    pub const fn id(self) -> CssSpecificationSourceId {
        self.id
    }

    /// Returns the specification module name.
    #[must_use]
    pub const fn module(self) -> &'static str {
        self.module
    }

    /// Returns the module level or exact selected baseline slice.
    #[must_use]
    pub const fn level(self) -> &'static str {
        self.level
    }

    /// Returns the source's conformance-profile tier.
    #[must_use]
    pub const fn tier(self) -> CssSpecificationTier {
        self.tier
    }

    /// Returns the immutable specification URL, when this record cites a specification.
    #[must_use]
    pub const fn url(self) -> Option<&'static str> {
        self.url
    }

    /// Returns the exact baseline revision and path, when this record cites repository source.
    #[must_use]
    pub const fn repository_provenance(self) -> Option<&'static str> {
        self.repository_provenance
    }
}

/// A stable identity for one official conformance exclusion.
///
/// ```compile_fail
/// use surgeist_css::CssConformanceExclusionId;
///
/// let _ = CssConformanceExclusionId("excluded.O-CSS2.informative-audit");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssConformanceExclusionId(&'static str);

impl CssConformanceExclusionId {
    const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Returns the exact stable exclusion identity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Why an official source item is not an authored parser-facing production.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssExclusionReason {
    /// The source item is informative rather than a normative production.
    InformativeOnly,
    /// A selected source replaced the item without leaving a current production.
    SupersededWithoutCurrentProduction,
    /// The item specifies semantics outside strict authored CSS syntax.
    OutsideAuthoredSyntaxBoundary,
}

/// A stable identity for a source or production that supersedes an exclusion.
///
/// This semantic identity can name an active feature, a planned official
/// feature, or a selected source when the exclusion covers a broader source
/// area. Values are registry-owned and cannot be forged downstream.
///
/// ```compile_fail
/// use surgeist_css::CssConformanceSupersedingId;
///
/// let _ = CssConformanceSupersedingId("O-SYNTAX3");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssConformanceSupersedingId(&'static str);

impl CssConformanceSupersedingId {
    const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Returns the exact stable superseding identity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Immutable metadata for one official conformance exclusion.
///
/// Exclusions are audit facts. They do not dispatch the parser, carry a support
/// status, or change how authored input is diagnosed.
///
/// ```compile_fail
/// use surgeist_css::CssExclusionReason;
///
/// let _ = surgeist_css::CssExclusionMetadata {
///     id: todo!(),
///     source: todo!(),
///     production: "example",
///     reason: CssExclusionReason::InformativeOnly,
///     superseding_ids: None,
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssExclusionMetadata {
    id: CssConformanceExclusionId,
    source: CssSpecificationSource,
    production: &'static str,
    reason: CssExclusionReason,
    superseding_ids: Option<&'static [CssConformanceSupersedingId]>,
}

impl CssExclusionMetadata {
    const fn new(
        id: &'static str,
        source: CssSpecificationSource,
        production: &'static str,
        reason: CssExclusionReason,
        superseding_ids: Option<&'static [CssConformanceSupersedingId]>,
    ) -> Self {
        Self {
            id: CssConformanceExclusionId::new(id),
            source,
            production,
            reason,
            superseding_ids,
        }
    }

    /// Returns the globally unique stable exclusion identity.
    #[must_use]
    pub const fn id(&self) -> CssConformanceExclusionId {
        self.id
    }

    /// Returns the immutable official source containing the excluded item.
    #[must_use]
    pub const fn source(&self) -> CssSpecificationSource {
        self.source
    }

    /// Returns the exact source area, production, or fragment.
    #[must_use]
    pub const fn production(&self) -> &'static str {
        self.production
    }

    /// Returns why the item is excluded from authored syntax coverage.
    #[must_use]
    pub const fn reason(&self) -> CssExclusionReason {
        self.reason
    }

    /// Returns stable IDs that supersede the item, when the contract names them.
    #[must_use]
    pub const fn superseding_ids(&self) -> Option<&'static [CssConformanceSupersedingId]> {
        self.superseding_ids
    }
}

/// Immutable support metadata for one CSS production or preserved baseline alias.
///
/// Records describe authored syntax only. They do not dispatch the parser,
/// validate source, perform selector/query matching, or resolve authored values.
/// The four aggregate aliases expose immutable atomic targets and do not own
/// parser dispatch. Fields and construction are catalog-owned so stable identity
/// and status invariants cannot be forged by downstream callers.
///
/// ```compile_fail
/// use surgeist_css::feature_catalog;
///
/// let metadata = &feature_catalog()[0];
/// let _ = metadata.id;
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssFeatureMetadata {
    id: CssFeatureId,
    kind: CssFeatureKind,
    spelling: &'static str,
    source: CssSpecificationSource,
    production: &'static str,
    status: CssSupportStatus,
    supported_subset: Option<&'static str>,
    unsupported_remainder: Option<&'static str>,
    recognized_unsupported_code: Option<CssErrorCode>,
    disposition: CssConformanceDisposition,
    property: Option<CssKnownProperty>,
    property_aliases: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum CssConformanceDisposition {
    Atomic,
    BaselineAlias(&'static [CssFeatureId]),
}

impl CssFeatureMetadata {
    const fn complete(
        id: &'static str,
        kind: CssFeatureKind,
        spelling: &'static str,
        source: CssSpecificationSource,
        production: &'static str,
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind,
            spelling,
            source,
            production,
            status: CssSupportStatus::Complete,
            supported_subset: None,
            unsupported_remainder: None,
            recognized_unsupported_code: None,
            disposition: CssConformanceDisposition::Atomic,
            property: None,
            property_aliases: &[],
        }
    }

    const fn partial(
        id: &'static str,
        kind: CssFeatureKind,
        spelling: &'static str,
        source: CssSpecificationSource,
        production: &'static str,
        supported_subset: &'static str,
        unsupported_remainder: &'static str,
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind,
            spelling,
            source,
            production,
            status: CssSupportStatus::Partial,
            supported_subset: Some(supported_subset),
            unsupported_remainder: Some(unsupported_remainder),
            recognized_unsupported_code: None,
            disposition: CssConformanceDisposition::Atomic,
            property: None,
            property_aliases: &[],
        }
    }

    const fn baseline_alias(
        id: &'static str,
        kind: CssFeatureKind,
        spelling: &'static str,
        source: CssSpecificationSource,
        production: &'static str,
        boundary: (&'static str, &'static str),
        targets: &'static [CssFeatureId],
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind,
            spelling,
            source,
            production,
            status: CssSupportStatus::Partial,
            supported_subset: Some(boundary.0),
            unsupported_remainder: Some(boundary.1),
            recognized_unsupported_code: None,
            disposition: CssConformanceDisposition::BaselineAlias(targets),
            property: None,
            property_aliases: &[],
        }
    }

    const fn recognized_unsupported(
        id: &'static str,
        kind: CssFeatureKind,
        spelling: &'static str,
        source: CssSpecificationSource,
        production: &'static str,
        code: CssErrorCode,
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind,
            spelling,
            source,
            production,
            status: CssSupportStatus::RecognizedUnsupported,
            supported_subset: None,
            unsupported_remainder: None,
            recognized_unsupported_code: Some(code),
            disposition: CssConformanceDisposition::Atomic,
            property: None,
            property_aliases: &[],
        }
    }

    const fn partial_property(
        id: &'static str,
        property: CssKnownProperty,
        canonical_name: &'static str,
        production: &'static str,
        aliases: &'static [&'static str],
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind: CssFeatureKind::Property,
            spelling: canonical_name,
            source: property_source(property),
            production: property_production(property, production),
            status: CssSupportStatus::Partial,
            supported_subset: Some(PROPERTY_SUBSET),
            unsupported_remainder: Some(PROPERTY_REMAINDER),
            recognized_unsupported_code: None,
            disposition: CssConformanceDisposition::Atomic,
            property: Some(property),
            property_aliases: aliases,
        }
    }

    const fn complete_property(
        id: &'static str,
        property: CssKnownProperty,
        canonical_name: &'static str,
        production: &'static str,
        aliases: &'static [&'static str],
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind: CssFeatureKind::Property,
            spelling: canonical_name,
            source: property_source(property),
            production: property_production(property, production),
            status: CssSupportStatus::Complete,
            supported_subset: None,
            unsupported_remainder: None,
            recognized_unsupported_code: None,
            disposition: CssConformanceDisposition::Atomic,
            property: Some(property),
            property_aliases: aliases,
        }
    }

    const fn partial_property_with_boundary(
        id: &'static str,
        property: CssKnownProperty,
        canonical_name: &'static str,
        production: &'static str,
        aliases: &'static [&'static str],
        supported_subset: &'static str,
        unsupported_remainder: &'static str,
    ) -> Self {
        Self {
            id: CssFeatureId::new(id),
            kind: CssFeatureKind::Property,
            spelling: canonical_name,
            source: property_source(property),
            production: property_production(property, production),
            status: CssSupportStatus::Partial,
            supported_subset: Some(supported_subset),
            unsupported_remainder: Some(unsupported_remainder),
            recognized_unsupported_code: None,
            disposition: CssConformanceDisposition::Atomic,
            property: Some(property),
            property_aliases: aliases,
        }
    }

    /// Returns the globally unique stable feature identity.
    #[must_use]
    pub const fn id(&self) -> CssFeatureId {
        self.id
    }

    /// Returns the authored grammar-family role.
    #[must_use]
    pub const fn kind(&self) -> CssFeatureKind {
        self.kind
    }

    /// Returns the exact authored spelling or named grammar production.
    #[must_use]
    pub const fn spelling(&self) -> &'static str {
        self.spelling
    }

    /// Returns the immutable specification or repository provenance.
    #[must_use]
    pub const fn source(&self) -> CssSpecificationSource {
        self.source
    }

    /// Returns the exact cited production or section name.
    #[must_use]
    pub const fn production(&self) -> &'static str {
        self.production
    }

    /// Returns the bounded implementation state.
    #[must_use]
    pub const fn status(&self) -> CssSupportStatus {
        self.status
    }

    /// Returns the non-empty implemented-subset description exactly for partial records.
    #[must_use]
    pub const fn supported_subset(&self) -> Option<&'static str> {
        self.supported_subset
    }

    /// Returns the non-empty valid-but-unsupported description exactly for partial records.
    #[must_use]
    pub const fn unsupported_remainder(&self) -> Option<&'static str> {
        self.unsupported_remainder
    }

    /// Returns the emitted root diagnostic code exactly for recognized-unsupported records.
    ///
    /// The complete diagnostic identity is this code together with [`Self::id`].
    #[must_use]
    pub const fn recognized_unsupported_code(&self) -> Option<CssErrorCode> {
        self.recognized_unsupported_code
    }

    /// Returns the immutable atomic targets of a preserved I01 aggregate alias.
    ///
    /// Atomic parser-facing records return an empty slice.
    #[must_use]
    pub const fn baseline_alias_targets(&self) -> &'static [CssFeatureId] {
        match self.disposition {
            CssConformanceDisposition::Atomic => &[],
            CssConformanceDisposition::BaselineAlias(targets) => targets,
        }
    }
}

/// Immutable catalog metadata for one recognized non-custom authored CSS property.
///
/// The metadata reports parser support; it does not apply cascade, substitute
/// variables, resolve values, or dispatch property parsing. Construction is
/// catalog-owned so callers cannot forge a property-to-feature association.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssPropertyMetadata {
    feature: &'static CssFeatureMetadata,
}

impl CssPropertyMetadata {
    /// Returns the underlying support-catalog record.
    #[must_use]
    pub const fn feature(&self) -> &'static CssFeatureMetadata {
        self.feature
    }

    /// Returns the canonical authored property identity.
    #[must_use]
    pub fn property(&self) -> CssKnownProperty {
        self.feature
            .property
            .expect("property metadata always wraps a property catalog record")
    }

    /// Returns the canonical lowercase authored property spelling.
    #[must_use]
    pub const fn canonical_name(&self) -> &'static str {
        self.feature.spelling
    }

    /// Returns reviewed authored aliases that normalize to this property.
    #[must_use]
    pub const fn aliases(&self) -> &'static [&'static str] {
        self.feature.property_aliases
    }
}

macro_rules! dated_source {
    ($id:literal, $module:literal, $level:literal, $tier:path, $url:literal) => {
        CssSpecificationSource::from_url($id, $module, $level, $tier, $url)
    };
}

const O_CSS2: CssSpecificationSource = dated_source!(
    "O-CSS2",
    "CSS",
    "2.1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2011/REC-CSS2-20110607/"
);
const O_SYNTAX3: CssSpecificationSource = dated_source!(
    "O-SYNTAX3",
    "CSS Syntax",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/"
);
const O_STYLE_ATTR: CssSpecificationSource = dated_source!(
    "O-STYLE-ATTR",
    "CSS Style Attributes",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2013/REC-css-style-attr-20131107/"
);
const O_MEDIA3: CssSpecificationSource = dated_source!(
    "O-MEDIA3",
    "Media Queries",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/REC-mediaqueries-3-20240521/"
);
const O_CONDITIONAL3: CssSpecificationSource = dated_source!(
    "O-CONDITIONAL3",
    "CSS Conditional Rules",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/"
);
const O_SELECTORS3: CssSpecificationSource = dated_source!(
    "O-SELECTORS3",
    "Selectors",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2018/REC-selectors-3-20181106/"
);
const O_NAMESPACES3: CssSpecificationSource = dated_source!(
    "O-NAMESPACES3",
    "CSS Namespaces",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2014/REC-css-namespaces-3-20140320/"
);
const O_CASCADE4: CssSpecificationSource = dated_source!(
    "O-CASCADE4",
    "CSS Cascading and Inheritance",
    "4",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2022/CR-css-cascade-4-20220113/"
);
const O_VALUES3: CssSpecificationSource = dated_source!(
    "O-VALUES3",
    "CSS Values and Units",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/CRD-css-values-3-20240322/"
);
const O_VARIABLES1: CssSpecificationSource = dated_source!(
    "O-VARIABLES1",
    "CSS Custom Properties for Cascading Variables",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2022/CR-css-variables-1-20220616/"
);
const O_BOX3: CssSpecificationSource = dated_source!(
    "O-BOX3",
    "CSS Box Model",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/REC-css-box-3-20240411/"
);
const O_COLOR4: CssSpecificationSource = dated_source!(
    "O-COLOR4",
    "CSS Color",
    "4",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2026/CRD-css-color-4-20260326/"
);
const O_BACKGROUNDS3: CssSpecificationSource = dated_source!(
    "O-BACKGROUNDS3",
    "CSS Backgrounds and Borders",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/"
);
const O_IMAGES3: CssSpecificationSource = dated_source!(
    "O-IMAGES3",
    "CSS Images",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2023/CRD-css-images-3-20231218/"
);
const O_FONTS3: CssSpecificationSource = dated_source!(
    "O-FONTS3",
    "CSS Fonts",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/"
);
const O_WRITING3: CssSpecificationSource = dated_source!(
    "O-WRITING3",
    "CSS Writing Modes",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2019/REC-css-writing-modes-3-20191210/"
);
const O_MULTICOL1: CssSpecificationSource = dated_source!(
    "O-MULTICOL1",
    "CSS Multi-column Layout",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/"
);
const O_FLEXBOX1: CssSpecificationSource = dated_source!(
    "O-FLEXBOX1",
    "CSS Flexible Box Layout",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/"
);
const O_UI3: CssSpecificationSource = dated_source!(
    "O-UI3",
    "CSS Basic User Interface",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2018/REC-css-ui-3-20180621/"
);
const O_CONTAIN1: CssSpecificationSource = dated_source!(
    "O-CONTAIN1",
    "CSS Containment",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/REC-css-contain-1-20240625/"
);
const O_TRANSFORMS1: CssSpecificationSource = dated_source!(
    "O-TRANSFORMS1",
    "CSS Transforms",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2019/CR-css-transforms-1-20190214/"
);
const O_COMPOSITING1: CssSpecificationSource = dated_source!(
    "O-COMPOSITING1",
    "Compositing and Blending",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2024/CRD-compositing-1-20240321/"
);
const O_EASING1: CssSpecificationSource = dated_source!(
    "O-EASING1",
    "CSS Easing Functions",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2023/CRD-css-easing-1-20230213/"
);
const O_COUNTERSTYLES3: CssSpecificationSource = dated_source!(
    "O-COUNTERSTYLES3",
    "CSS Counter Styles",
    "3",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/"
);

macro_rules! profile_source {
    ($name:ident, $id:literal, $module:literal, $level:literal, $tier:path, $url:literal) => {
        const $name: CssSpecificationSource = dated_source!($id, $module, $level, $tier, $url);
    };
}

profile_source!(
    R_MEDIA4,
    "R-MEDIA4",
    "Media Queries",
    "4",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/"
);
profile_source!(
    R_SCROLLBARS1,
    "R-SCROLLBARS1",
    "CSS Scrollbars Styling",
    "1",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/"
);
profile_source!(
    R_GRID1,
    "R-GRID1",
    "CSS Grid Layout",
    "1",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2025/CRD-css-grid-1-20250326/"
);
profile_source!(
    R_GRID2,
    "R-GRID2",
    "CSS Grid Layout",
    "2",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/"
);
profile_source!(
    R_CASCADE5,
    "R-CASCADE5",
    "CSS Cascading and Inheritance",
    "5",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/"
);
profile_source!(
    R_CONDITIONAL4,
    "R-CONDITIONAL4",
    "CSS Conditional Rules",
    "4",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2025/CRD-css-conditional-4-20250904/"
);
profile_source!(
    S_DISPLAY3,
    "S-DISPLAY3",
    "CSS Display",
    "3",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2026/CRD-css-display-3-20260605/"
);
profile_source!(
    S_WRITING4,
    "S-WRITING4",
    "CSS Writing Modes",
    "4",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/"
);
profile_source!(
    S_BREAK3,
    "S-BREAK3",
    "CSS Fragmentation",
    "3",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2018/CR-css-break-3-20181204/"
);
profile_source!(
    S_ALIGN3,
    "S-ALIGN3",
    "CSS Box Alignment",
    "3",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2026/WD-css-align-3-20260130/"
);
profile_source!(
    S_SHAPES1,
    "S-SHAPES1",
    "CSS Shapes",
    "1",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/"
);
profile_source!(
    S_TEXT3,
    "S-TEXT3",
    "CSS Text",
    "3",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2026/CRD-css-text-3-20260608/"
);
profile_source!(
    S_TEXTDECOR3,
    "S-TEXTDECOR3",
    "CSS Text Decoration",
    "3",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2022/CRD-css-text-decor-3-20220505/"
);
profile_source!(
    S_MASKING1,
    "S-MASKING1",
    "CSS Masking",
    "1",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/"
);
profile_source!(
    I_TRANSITIONS1,
    "I-TRANSITIONS1",
    "CSS Transitions",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/"
);
profile_source!(
    I_ANIMATIONS1,
    "I-ANIMATIONS1",
    "CSS Animations",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2023/WD-css-animations-1-20230302/"
);
profile_source!(
    I_FILTER1,
    "I-FILTER1",
    "Filter Effects",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/"
);
profile_source!(
    I_SIZING3,
    "I-SIZING3",
    "CSS Box Sizing",
    "3",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/"
);
profile_source!(
    I_TRANSFORMS2,
    "I-TRANSFORMS2",
    "CSS Transforms",
    "2",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/"
);
profile_source!(
    I_LISTS3,
    "I-LISTS3",
    "CSS Lists and Counters",
    "3",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2020/WD-css-lists-3-20201117/"
);
profile_source!(
    I_POSITION3,
    "I-POSITION3",
    "CSS Positioned Layout",
    "3",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2025/WD-css-position-3-20251007/"
);
profile_source!(
    I_FONTS4,
    "I-FONTS4",
    "CSS Fonts",
    "4",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-fonts-4-20260422/"
);
profile_source!(
    I_COLOR5,
    "I-COLOR5",
    "CSS Color",
    "5",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-color-5-20260618/"
);
profile_source!(
    I_SELECTORS4,
    "I-SELECTORS4",
    "Selectors",
    "4",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-selectors-4-20260122/"
);
profile_source!(
    I_CONTAIN2,
    "I-CONTAIN2",
    "CSS Containment",
    "2",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2022/WD-css-contain-2-20220917/"
);
profile_source!(
    I_NESTING1,
    "I-NESTING1",
    "CSS Nesting",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/"
);
profile_source!(
    X_CONTAIN3,
    "X-CONTAIN3",
    "CSS Containment",
    "3",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2022/WD-css-contain-3-20220818/"
);
profile_source!(
    X_CONDITIONAL5,
    "X-CONDITIONAL5",
    "CSS Conditional Rules",
    "5",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/"
);
profile_source!(
    X_CASCADE6,
    "X-CASCADE6",
    "CSS Cascading and Inheritance",
    "6",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/"
);
profile_source!(
    X_PSEUDO4,
    "X-PSEUDO4",
    "CSS Pseudo-Elements",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2025/WD-css-pseudo-4-20250627/"
);
const X_VALUES4: CssSpecificationSource = CssSpecificationSource::from_repository(
    "X-VALUES4",
    "CSS Values and Units",
    "4",
    "720ea2863696971ea6a6744e0f23acbb3e6936bd:css-values-4/Overview.bs",
);
profile_source!(
    X_MEDIA5,
    "X-MEDIA5",
    "Media Queries",
    "5",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/"
);
profile_source!(
    X_OVERFLOW3,
    "X-OVERFLOW3",
    "CSS Overflow",
    "3",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/"
);
profile_source!(
    X_SIZING4,
    "X-SIZING4",
    "CSS Box Sizing",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2021/WD-css-sizing-4-20210520/"
);
profile_source!(
    X_TEXT4,
    "X-TEXT4",
    "CSS Text",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2026/WD-css-text-4-20260608/"
);
profile_source!(
    X_TEXTDECOR4,
    "X-TEXTDECOR4",
    "CSS Text Decoration",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/"
);
profile_source!(
    X_UI4,
    "X-UI4",
    "CSS Basic User Interface",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2026/WD-css-ui-4-20260120/"
);
profile_source!(
    X_CONTENT3,
    "X-CONTENT3",
    "CSS Generated Content",
    "3",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2025/WD-css-content-3-20251204/"
);
profile_source!(
    X_FULLSCREEN,
    "X-FULLSCREEN",
    "Fullscreen",
    "unleveled",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2012/WD-fullscreen-20120703/"
);

const X_FILTER2_BASE: CssSpecificationSource = CssSpecificationSource::from_repository(
    "X-FILTER2-BASE",
    "Filter Effects",
    "2 baseline subset",
    "bc5394f:src/parser/effects.rs",
);
const X_DISPLAY_MODE_BASE: CssSpecificationSource = CssSpecificationSource::from_repository(
    "X-DISPLAY-MODE-BASE",
    "Media Queries",
    "display-mode baseline subset",
    "bc5394f:src/parser/queries.rs",
);
const X_GRID_TOLERANCE_BASE: CssSpecificationSource = CssSpecificationSource::from_repository(
    "X-GRID-TOLERANCE-BASE",
    "CSS Grid Layout",
    "grid-flow-tolerance baseline subset",
    "bc5394f:src/parser/grid.rs",
);

const CSS_SYNTAX_3: CssSpecificationSource = O_SYNTAX3;
const CSS_STYLE_ATTRIBUTES: CssSpecificationSource = O_STYLE_ATTR;
const CSS_CASCADE_4: CssSpecificationSource = O_CASCADE4;
const BASELINE_SELECTORS: CssSpecificationSource = CssSpecificationSource::from_repository(
    "I01-BASE-SELECTORS",
    "Surgeist CSS selectors parser",
    "I01 baseline",
    "bc5394f:src/parser/selectors.rs",
);
const BASELINE_QUERIES: CssSpecificationSource = CssSpecificationSource::from_repository(
    "I01-BASE-QUERIES",
    "Surgeist CSS query parser",
    "I01 baseline",
    "bc5394f:src/parser/queries.rs",
);

static SPECIFICATION_SOURCES: &[CssSpecificationSource] = &[
    O_CSS2,
    O_SYNTAX3,
    O_STYLE_ATTR,
    O_MEDIA3,
    O_CONDITIONAL3,
    O_SELECTORS3,
    O_NAMESPACES3,
    O_CASCADE4,
    O_VALUES3,
    O_VARIABLES1,
    O_BOX3,
    O_COLOR4,
    O_BACKGROUNDS3,
    O_IMAGES3,
    O_FONTS3,
    O_WRITING3,
    O_MULTICOL1,
    O_FLEXBOX1,
    O_UI3,
    O_CONTAIN1,
    O_TRANSFORMS1,
    O_COMPOSITING1,
    O_EASING1,
    O_COUNTERSTYLES3,
    R_MEDIA4,
    R_SCROLLBARS1,
    R_GRID1,
    R_GRID2,
    R_CASCADE5,
    R_CONDITIONAL4,
    S_DISPLAY3,
    S_WRITING4,
    S_BREAK3,
    S_ALIGN3,
    S_SHAPES1,
    S_TEXT3,
    S_TEXTDECOR3,
    S_MASKING1,
    I_TRANSITIONS1,
    I_ANIMATIONS1,
    I_FILTER1,
    I_SIZING3,
    I_TRANSFORMS2,
    I_LISTS3,
    I_POSITION3,
    I_FONTS4,
    I_COLOR5,
    I_SELECTORS4,
    I_CONTAIN2,
    I_NESTING1,
    X_CONTAIN3,
    X_CONDITIONAL5,
    X_CASCADE6,
    X_PSEUDO4,
    X_VALUES4,
    X_MEDIA5,
    X_OVERFLOW3,
    X_SIZING4,
    X_TEXT4,
    X_TEXTDECOR4,
    X_UI4,
    X_CONTENT3,
    X_FULLSCREEN,
    X_FILTER2_BASE,
    X_DISPLAY_MODE_BASE,
    X_GRID_TOLERANCE_BASE,
    BASELINE_SELECTORS,
    BASELINE_QUERIES,
];

macro_rules! exclusion {
    ($id:expr, $source:ident, $production:expr, $reason:path) => {
        CssExclusionMetadata::new($id, $source, $production, $reason, None)
    };
    ($id:expr, $source:ident, $production:expr, $reason:path, [$($owner:literal),+ $(,)?]) => {
        CssExclusionMetadata::new(
            $id,
            $source,
            $production,
            $reason,
            Some(&[$(CssConformanceSupersedingId::new($owner)),+]),
        )
    };
}

macro_rules! superseded_css2_property {
    ($name:literal, $chapter:literal, $owner:literal) => {
        exclusion!(
            concat!("excluded.O-CSS2.property.", $name),
            O_CSS2,
            concat!($chapter, "#propdef-", $name),
            CssExclusionReason::SupersededWithoutCurrentProduction,
            [$owner]
        )
    };
}

macro_rules! informative_css2_property {
    ($name:literal) => {
        exclusion!(
            concat!("excluded.O-CSS2.informative-property.", $name),
            O_CSS2,
            concat!("aural.html#propdef-", $name),
            CssExclusionReason::InformativeOnly
        )
    };
}

const INFORMATIVE_SOURCE_AUDIT: &str = "examples, explicitly non-normative notes, status/TOC, changelogs, acknowledgments, indexes, bibliography, test inventories, and conformance boilerplate";

macro_rules! informative_source_audit {
    ($source_id:literal, $source:ident) => {
        exclusion!(
            concat!("excluded.", $source_id, ".informative-audit"),
            $source,
            INFORMATIVE_SOURCE_AUDIT,
            CssExclusionReason::InformativeOnly
        )
    };
}

static CONFORMANCE_EXCLUSIONS: &[CssExclusionMetadata] = &[
    exclusion!(
        "excluded.O-WRITING3.property.glyph-orientation-horizontal",
        O_WRITING3,
        "#propdef-glyph-orientation-horizontal",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["official.property.text-orientation"]
    ),
    exclusion!(
        "excluded.O-UI3.property.ime-mode",
        O_UI3,
        "#propdef-ime-mode",
        CssExclusionReason::SupersededWithoutCurrentProduction
    ),
    superseded_css2_property!("margin", "box.html", "baseline.property.margin"),
    superseded_css2_property!(
        "margin-bottom",
        "box.html",
        "baseline.property.margin-bottom"
    ),
    superseded_css2_property!("margin-left", "box.html", "baseline.property.margin-left"),
    superseded_css2_property!("margin-right", "box.html", "baseline.property.margin-right"),
    superseded_css2_property!("margin-top", "box.html", "baseline.property.margin-top"),
    superseded_css2_property!("padding", "box.html", "baseline.property.padding"),
    superseded_css2_property!(
        "padding-bottom",
        "box.html",
        "baseline.property.padding-bottom"
    ),
    superseded_css2_property!("padding-left", "box.html", "baseline.property.padding-left"),
    superseded_css2_property!(
        "padding-right",
        "box.html",
        "baseline.property.padding-right"
    ),
    superseded_css2_property!("padding-top", "box.html", "baseline.property.padding-top"),
    superseded_css2_property!("color", "colors.html", "baseline.property.color"),
    superseded_css2_property!("background", "colors.html", "baseline.property.background"),
    superseded_css2_property!(
        "background-attachment",
        "colors.html",
        "baseline.property.background-attachment"
    ),
    superseded_css2_property!(
        "background-color",
        "colors.html",
        "baseline.property.background-color"
    ),
    superseded_css2_property!(
        "background-image",
        "colors.html",
        "baseline.property.background-image"
    ),
    superseded_css2_property!(
        "background-position",
        "colors.html",
        "baseline.property.background-position"
    ),
    superseded_css2_property!(
        "background-repeat",
        "colors.html",
        "baseline.property.background-repeat"
    ),
    superseded_css2_property!("border", "box.html", "baseline.property.border"),
    superseded_css2_property!(
        "border-bottom",
        "box.html",
        "baseline.property.border-bottom"
    ),
    superseded_css2_property!(
        "border-bottom-color",
        "box.html",
        "baseline.property.border-bottom-color"
    ),
    superseded_css2_property!(
        "border-bottom-style",
        "box.html",
        "baseline.property.border-bottom-style"
    ),
    superseded_css2_property!(
        "border-bottom-width",
        "box.html",
        "baseline.property.border-bottom-width"
    ),
    superseded_css2_property!("border-color", "box.html", "baseline.property.border-color"),
    superseded_css2_property!("border-left", "box.html", "baseline.property.border-left"),
    superseded_css2_property!(
        "border-left-color",
        "box.html",
        "baseline.property.border-left-color"
    ),
    superseded_css2_property!(
        "border-left-style",
        "box.html",
        "baseline.property.border-left-style"
    ),
    superseded_css2_property!(
        "border-left-width",
        "box.html",
        "baseline.property.border-left-width"
    ),
    superseded_css2_property!("border-right", "box.html", "baseline.property.border-right"),
    superseded_css2_property!(
        "border-right-color",
        "box.html",
        "baseline.property.border-right-color"
    ),
    superseded_css2_property!(
        "border-right-style",
        "box.html",
        "baseline.property.border-right-style"
    ),
    superseded_css2_property!(
        "border-right-width",
        "box.html",
        "baseline.property.border-right-width"
    ),
    superseded_css2_property!("border-style", "box.html", "baseline.property.border-style"),
    superseded_css2_property!("border-top", "box.html", "baseline.property.border-top"),
    superseded_css2_property!(
        "border-top-color",
        "box.html",
        "baseline.property.border-top-color"
    ),
    superseded_css2_property!(
        "border-top-style",
        "box.html",
        "baseline.property.border-top-style"
    ),
    superseded_css2_property!(
        "border-top-width",
        "box.html",
        "baseline.property.border-top-width"
    ),
    superseded_css2_property!("border-width", "box.html", "baseline.property.border-width"),
    superseded_css2_property!("font", "fonts.html", "baseline.property.font"),
    superseded_css2_property!("font-family", "fonts.html", "baseline.property.font-family"),
    superseded_css2_property!("font-size", "fonts.html", "baseline.property.font-size"),
    superseded_css2_property!("font-style", "fonts.html", "baseline.property.font-style"),
    superseded_css2_property!(
        "font-variant",
        "fonts.html",
        "baseline.property.font-variant"
    ),
    superseded_css2_property!("font-weight", "fonts.html", "baseline.property.font-weight"),
    superseded_css2_property!("direction", "visuren.html", "baseline.property.direction"),
    superseded_css2_property!(
        "unicode-bidi",
        "visuren.html",
        "official.property.unicode-bidi"
    ),
    superseded_css2_property!("cursor", "ui.html", "baseline.property.cursor"),
    superseded_css2_property!("outline", "ui.html", "baseline.property.outline"),
    superseded_css2_property!(
        "outline-color",
        "ui.html",
        "baseline.property.outline-color"
    ),
    superseded_css2_property!(
        "outline-style",
        "ui.html",
        "baseline.property.outline-style"
    ),
    superseded_css2_property!(
        "outline-width",
        "ui.html",
        "baseline.property.outline-width"
    ),
    informative_css2_property!("azimuth"),
    informative_css2_property!("cue"),
    informative_css2_property!("cue-after"),
    informative_css2_property!("cue-before"),
    informative_css2_property!("elevation"),
    informative_css2_property!("pause"),
    informative_css2_property!("pause-after"),
    informative_css2_property!("pause-before"),
    informative_css2_property!("pitch"),
    informative_css2_property!("pitch-range"),
    informative_css2_property!("play-during"),
    informative_css2_property!("richness"),
    informative_css2_property!("speak"),
    informative_css2_property!("speak-header"),
    informative_css2_property!("speak-numeral"),
    informative_css2_property!("speak-punctuation"),
    informative_css2_property!("speech-rate"),
    informative_css2_property!("stress"),
    informative_css2_property!("voice-family"),
    informative_css2_property!("volume"),
    exclusion!(
        "excluded.O-CSS2.superseded-syntax",
        O_CSS2,
        "syndata.html;grammar.html",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["O-SYNTAX3"]
    ),
    exclusion!(
        "excluded.O-CSS2.superseded-media",
        O_CSS2,
        "media.html",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["O-MEDIA3", "O-CONDITIONAL3"]
    ),
    exclusion!(
        "excluded.O-CSS2.superseded-selectors",
        O_CSS2,
        "selector.html",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["O-SELECTORS3", "O-NAMESPACES3"]
    ),
    exclusion!(
        "excluded.O-CSS2.superseded-cascade-values",
        O_CSS2,
        "cascade.html;syndata.html",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["O-CASCADE4", "O-VALUES3", "O-VARIABLES1"]
    ),
    exclusion!(
        "excluded.O-CSS2.non-authored-semantics",
        O_CSS2,
        "visuren.html;visufx.html;tables.html;page.html#outside-page-box,#page-breaks,#page-cascade",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-SYNTAX3.fragment-api",
        O_SYNTAX3,
        "#parse-rule,#parse-declaration,#parse-component-value,#parse-list-of-component-values,#parse-comma-separated-list-of-component-values",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-SYNTAX3.serialization",
        O_SYNTAX3,
        "#serialization",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-STYLE-ATTR.interpretation",
        O_STYLE_ATTR,
        "#interpret",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-MEDIA3.evaluation",
        O_MEDIA3,
        "#media0,#media1 evaluation portions",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-CONDITIONAL3.evaluation-api",
        O_CONDITIONAL3,
        "#processing,#the-cssmediarule-interface,#the-csssupportsrule-interface,#apis",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-SELECTORS3.matching-specificity",
        O_SELECTORS3,
        "#selectors,#specificity,#first-formatted-line,#application-in-css",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-NAMESPACES3.uri-matching",
        O_NAMESPACES3,
        "semantic portions of #scope,#css-qnames",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-CASCADE4.processing",
        O_CASCADE4,
        "#import-processing,#value-stages,#filtering,#cascading,#initial-values,#inheriting",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-VALUES3.metasyntax",
        O_VALUES3,
        "#value-defs",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-VALUES3.computation",
        O_VALUES3,
        "#calc-computed-value,#calc-range,#calc-serialize,#relative-urls",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-VARIABLES1.substitution",
        O_VARIABLES1,
        "#cycles,#invalid-variables,#variables-in-shorthands,#apis",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-BOX3.layout",
        O_BOX3,
        "#box-model,#fragmentation",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-COLOR4.color5-syntax",
        O_COLOR4,
        "Color 5 references",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["I-COLOR5"]
    ),
    exclusion!(
        "excluded.O-COLOR4.quirky-color",
        O_COLOR4,
        "#quirky-color",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-COLOR4.processing",
        O_COLOR4,
        "conversion/interpolation/gamut/resolution/serialization/sample-code sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-BACKGROUNDS3.painting",
        O_BACKGROUNDS3,
        "serialization/painting/corner/border-image/shadow algorithms",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-IMAGES3.processing",
        O_IMAGES3,
        "object negotiation/sizing/interpolation/serialization algorithms",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-FONTS3.processing",
        O_FONTS3,
        "loading/fetching/matching/feature-resolution/object-model sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-FONTS3.font-display",
        O_FONTS3,
        "no Fonts 3 production",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["baseline.descriptor.font-display"]
    ),
    exclusion!(
        "excluded.O-FONTS3.font-feature-values",
        O_FONTS3,
        "no Fonts 3 production",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["later.rule.font-feature-values"]
    ),
    exclusion!(
        "excluded.O-WRITING3.layout",
        O_WRITING3,
        "bidi/inline/abstract/principal-flow/text-combine algorithms",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-MULTICOL1.layout",
        O_MULTICOL1,
        "model/pseudo-algorithm/stacking/overflow sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-FLEXBOX1.layout",
        O_FLEXBOX1,
        "box/items/lines/layout/pagination/axis algorithms",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-FLEXBOX1.webkit-legacy",
        O_FLEXBOX1,
        "#webkit-aliases",
        CssExclusionReason::SupersededWithoutCurrentProduction
    ),
    exclusion!(
        "excluded.O-UI3.behavior",
        O_UI3,
        "ellipsis/input/default-style behavior sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-CONTAIN1.semantics",
        O_CONTAIN1,
        "containment-type/optimization sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-TRANSFORMS1.processing",
        O_TRANSFORMS1,
        "rendering/SVG/animation/interpolation/matrix algorithms",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-COMPOSITING1.processing",
        O_COMPOSITING1,
        "Canvas/formula/backdrop/group/advanced-compositing sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-EASING1.evaluation",
        O_EASING1,
        "easing output/serialization sections",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    exclusion!(
        "excluded.O-COUNTERSTYLES3.processing",
        O_COUNTERSTYLES3,
        "counter algorithms/predefined rendering/APIs/sample sheet",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    ),
    informative_source_audit!("O-CSS2", O_CSS2),
    informative_source_audit!("O-SYNTAX3", O_SYNTAX3),
    informative_source_audit!("O-STYLE-ATTR", O_STYLE_ATTR),
    informative_source_audit!("O-MEDIA3", O_MEDIA3),
    informative_source_audit!("O-CONDITIONAL3", O_CONDITIONAL3),
    informative_source_audit!("O-SELECTORS3", O_SELECTORS3),
    informative_source_audit!("O-NAMESPACES3", O_NAMESPACES3),
    informative_source_audit!("O-CASCADE4", O_CASCADE4),
    informative_source_audit!("O-VALUES3", O_VALUES3),
    informative_source_audit!("O-VARIABLES1", O_VARIABLES1),
    informative_source_audit!("O-BOX3", O_BOX3),
    informative_source_audit!("O-COLOR4", O_COLOR4),
    informative_source_audit!("O-BACKGROUNDS3", O_BACKGROUNDS3),
    informative_source_audit!("O-IMAGES3", O_IMAGES3),
    informative_source_audit!("O-FONTS3", O_FONTS3),
    informative_source_audit!("O-WRITING3", O_WRITING3),
    informative_source_audit!("O-MULTICOL1", O_MULTICOL1),
    informative_source_audit!("O-FLEXBOX1", O_FLEXBOX1),
    informative_source_audit!("O-UI3", O_UI3),
    informative_source_audit!("O-CONTAIN1", O_CONTAIN1),
    informative_source_audit!("O-TRANSFORMS1", O_TRANSFORMS1),
    informative_source_audit!("O-COMPOSITING1", O_COMPOSITING1),
    informative_source_audit!("O-EASING1", O_EASING1),
    informative_source_audit!("O-COUNTERSTYLES3", O_COUNTERSTYLES3),
];

// These private records are a hand-authored reconciliation of the official
// production ledger. They intentionally do not participate in parser dispatch,
// public feature lookup, or support-status calculation. Every selected row is
// now either an active public atomic feature or an explicit exclusion; the
// persisted ledger remains an external review authority.

#[expect(dead_code, reason = "private official-ledger reconciliation metadata")]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum CssOfficialCoverageRecord {
    Active(CssFeatureId),
    Excluded(&'static CssExclusionMetadata),
}

#[expect(dead_code, reason = "private official-ledger reconciliation metadata")]
#[derive(Clone, Copy, Debug)]
struct CssOfficialCoverageGroup {
    records: &'static [CssOfficialCoverageRecord],
    shared_records: &'static [CssSharedCoverageRecord],
}

#[expect(dead_code, reason = "private official-ledger reconciliation metadata")]
#[derive(Clone, Copy, Debug)]
struct CssSharedCoverageRecord {
    ledger_index: usize,
    record: &'static CssOfficialCoverageRecord,
}

macro_rules! active_coverage {
    ($id:literal) => {
        CssOfficialCoverageRecord::Active(CssFeatureId::new($id))
    };
}

static OFFICIAL_CUSTOM_PROPERTY_COVERAGE: CssOfficialCoverageRecord =
    active_coverage!("baseline.declaration.custom-property");

static OFFICIAL_PROPERTY_COVERAGE_ROWS: &[CssOfficialCoverageRecord] = &[
    active_coverage!("official.property.border-collapse"),
    active_coverage!("official.property.border-spacing"),
    active_coverage!("baseline.property.bottom"),
    active_coverage!("official.property.caption-side"),
    active_coverage!("baseline.property.clear"),
    active_coverage!("official.property.clip"),
    active_coverage!("baseline.property.content"),
    active_coverage!("baseline.property.counter-increment"),
    active_coverage!("baseline.property.counter-reset"),
    active_coverage!("baseline.property.display"),
    active_coverage!("official.property.empty-cells"),
    active_coverage!("baseline.property.float"),
    active_coverage!("baseline.property.height"),
    active_coverage!("baseline.property.left"),
    active_coverage!("baseline.property.letter-spacing"),
    active_coverage!("baseline.property.line-height"),
    active_coverage!("baseline.property.list-style"),
    active_coverage!("baseline.property.list-style-image"),
    active_coverage!("baseline.property.list-style-position"),
    active_coverage!("baseline.property.list-style-type"),
    active_coverage!("baseline.property.max-height"),
    active_coverage!("baseline.property.max-width"),
    active_coverage!("baseline.property.min-height"),
    active_coverage!("baseline.property.min-width"),
    active_coverage!("official.property.orphans"),
    active_coverage!("baseline.property.overflow"),
    active_coverage!("official.property.page-break-after"),
    active_coverage!("official.property.page-break-before"),
    active_coverage!("official.property.page-break-inside"),
    active_coverage!("baseline.property.position"),
    active_coverage!("official.property.quotes"),
    active_coverage!("baseline.property.right"),
    active_coverage!("official.property.table-layout"),
    active_coverage!("baseline.property.text-align"),
    active_coverage!("baseline.property.text-decoration"),
    active_coverage!("baseline.property.text-indent"),
    active_coverage!("baseline.property.text-transform"),
    active_coverage!("baseline.property.top"),
    active_coverage!("baseline.property.vertical-align"),
    active_coverage!("baseline.property.visibility"),
    active_coverage!("baseline.property.white-space"),
    active_coverage!("official.property.widows"),
    active_coverage!("baseline.property.width"),
    active_coverage!("official.property.word-spacing"),
    active_coverage!("baseline.property.z-index"),
    active_coverage!("baseline.property.all"),
    active_coverage!("baseline.property.margin"),
    active_coverage!("baseline.property.margin-bottom"),
    active_coverage!("baseline.property.margin-left"),
    active_coverage!("baseline.property.margin-right"),
    active_coverage!("baseline.property.margin-top"),
    active_coverage!("baseline.property.padding"),
    active_coverage!("baseline.property.padding-bottom"),
    active_coverage!("baseline.property.padding-left"),
    active_coverage!("baseline.property.padding-right"),
    active_coverage!("baseline.property.padding-top"),
    active_coverage!("baseline.property.color"),
    active_coverage!("baseline.property.opacity"),
    active_coverage!("baseline.property.background"),
    active_coverage!("baseline.property.background-attachment"),
    active_coverage!("baseline.property.background-clip"),
    active_coverage!("baseline.property.background-color"),
    active_coverage!("baseline.property.background-image"),
    active_coverage!("baseline.property.background-origin"),
    active_coverage!("baseline.property.background-position"),
    active_coverage!("baseline.property.background-repeat"),
    active_coverage!("baseline.property.background-size"),
    active_coverage!("baseline.property.border"),
    active_coverage!("baseline.property.border-bottom"),
    active_coverage!("baseline.property.border-bottom-color"),
    active_coverage!("baseline.property.border-bottom-left-radius"),
    active_coverage!("baseline.property.border-bottom-right-radius"),
    active_coverage!("baseline.property.border-bottom-style"),
    active_coverage!("baseline.property.border-bottom-width"),
    active_coverage!("baseline.property.border-color"),
    active_coverage!("official.property.border-image"),
    active_coverage!("official.property.border-image-outset"),
    active_coverage!("official.property.border-image-repeat"),
    active_coverage!("official.property.border-image-slice"),
    active_coverage!("official.property.border-image-source"),
    active_coverage!("official.property.border-image-width"),
    active_coverage!("baseline.property.border-left"),
    active_coverage!("baseline.property.border-left-color"),
    active_coverage!("baseline.property.border-left-style"),
    active_coverage!("baseline.property.border-left-width"),
    active_coverage!("baseline.property.border-radius"),
    active_coverage!("baseline.property.border-right"),
    active_coverage!("baseline.property.border-right-color"),
    active_coverage!("baseline.property.border-right-style"),
    active_coverage!("baseline.property.border-right-width"),
    active_coverage!("baseline.property.border-style"),
    active_coverage!("baseline.property.border-top"),
    active_coverage!("baseline.property.border-top-color"),
    active_coverage!("baseline.property.border-top-left-radius"),
    active_coverage!("baseline.property.border-top-right-radius"),
    active_coverage!("baseline.property.border-top-style"),
    active_coverage!("baseline.property.border-top-width"),
    active_coverage!("baseline.property.border-width"),
    active_coverage!("baseline.property.box-shadow"),
    active_coverage!("official.property.image-orientation"),
    active_coverage!("official.property.image-rendering"),
    active_coverage!("official.property.object-fit"),
    active_coverage!("official.property.object-position"),
    active_coverage!("baseline.property.font"),
    active_coverage!("baseline.property.font-family"),
    active_coverage!("baseline.property.font-feature-settings"),
    active_coverage!("official.property.font-kerning"),
    active_coverage!("baseline.property.font-size"),
    active_coverage!("official.property.font-size-adjust"),
    active_coverage!("baseline.property.font-stretch"),
    active_coverage!("baseline.property.font-style"),
    active_coverage!("official.property.font-synthesis"),
    active_coverage!("baseline.property.font-variant"),
    active_coverage!("official.property.font-variant-caps"),
    active_coverage!("official.property.font-variant-east-asian"),
    active_coverage!("official.property.font-variant-ligatures"),
    active_coverage!("official.property.font-variant-numeric"),
    active_coverage!("official.property.font-variant-position"),
    active_coverage!("baseline.property.font-weight"),
    active_coverage!("baseline.property.direction"),
    active_coverage!("official.property.text-combine-upright"),
    active_coverage!("official.property.text-orientation"),
    active_coverage!("official.property.unicode-bidi"),
    active_coverage!("baseline.property.writing-mode"),
    active_coverage!("official.property.column-count"),
    active_coverage!("official.property.column-fill"),
    active_coverage!("official.property.column-rule"),
    active_coverage!("official.property.column-rule-color"),
    active_coverage!("official.property.column-rule-style"),
    active_coverage!("official.property.column-rule-width"),
    active_coverage!("official.property.column-span"),
    active_coverage!("official.property.column-width"),
    active_coverage!("official.property.columns"),
    active_coverage!("baseline.property.align-content"),
    active_coverage!("baseline.property.align-items"),
    active_coverage!("baseline.property.align-self"),
    active_coverage!("baseline.property.flex"),
    active_coverage!("baseline.property.flex-basis"),
    active_coverage!("baseline.property.flex-direction"),
    active_coverage!("official.property.flex-flow"),
    active_coverage!("baseline.property.flex-grow"),
    active_coverage!("baseline.property.flex-shrink"),
    active_coverage!("baseline.property.flex-wrap"),
    active_coverage!("baseline.property.justify-content"),
    active_coverage!("baseline.property.box-sizing"),
    active_coverage!("official.property.caret-color"),
    active_coverage!("baseline.property.cursor"),
    active_coverage!("baseline.property.outline"),
    active_coverage!("baseline.property.outline-color"),
    active_coverage!("official.property.outline-offset"),
    active_coverage!("baseline.property.outline-style"),
    active_coverage!("baseline.property.outline-width"),
    active_coverage!("official.property.resize"),
    active_coverage!("baseline.property.text-overflow"),
    active_coverage!("official.property.contain"),
    active_coverage!("baseline.property.transform"),
    active_coverage!("official.property.transform-box"),
    active_coverage!("baseline.property.transform-origin"),
    active_coverage!("official.property.background-blend-mode"),
    active_coverage!("official.property.isolation"),
    active_coverage!("official.property.mix-blend-mode"),
];

static OFFICIAL_NON_PROPERTY_COVERAGE_ROWS: &[CssOfficialCoverageRecord] = &[
    active_coverage!("later.rule.page"),
    active_coverage!("official.selector.page-pseudo"),
    active_coverage!("foundation.encoding.charset"),
    active_coverage!("baseline.rule.style"),
    active_coverage!("official.rule.at-rule"),
    active_coverage!("official.qualified-rule.generic"),
    active_coverage!("official.declaration.generic"),
    active_coverage!("official.value.syntax-token-stream"),
    active_coverage!("official.value.component-value"),
    active_coverage!("official.value.simple-block"),
    active_coverage!("official.value.function"),
    active_coverage!("official.value.stylesheet"),
    active_coverage!("official.value.rule-list"),
    active_coverage!("official.value.declaration-list"),
    active_coverage!("official.value.style-block"),
    active_coverage!("official.value.declaration-value"),
    active_coverage!("official.value.any-value"),
    active_coverage!("official.value.an-plus-b"),
    active_coverage!("official.value.unicode-range"),
    active_coverage!("foundation.declaration-list.style-attribute"),
    active_coverage!("official.media.query-list-core"),
    active_coverage!("baseline.media.type"),
    active_coverage!("official.media.feature.width"),
    active_coverage!("official.media.feature.height"),
    active_coverage!("official.media.feature.device-width"),
    active_coverage!("official.media.feature.device-height"),
    active_coverage!("official.media.feature.orientation"),
    active_coverage!("official.media.feature.aspect-ratio"),
    active_coverage!("official.media.feature.device-aspect-ratio"),
    active_coverage!("official.media.feature.color"),
    active_coverage!("official.media.feature.color-index"),
    active_coverage!("official.media.feature.monochrome"),
    active_coverage!("official.media.feature.resolution"),
    active_coverage!("official.media.feature.scan"),
    active_coverage!("official.media.feature.grid"),
    active_coverage!("baseline.rule.media"),
    active_coverage!("later.rule.supports"),
    active_coverage!("official.rule.conditional-group-context"),
    active_coverage!("official.selector.group"),
    active_coverage!("official.selector.type"),
    active_coverage!("official.selector.universal"),
    active_coverage!("official.selector.attribute-presence-value"),
    active_coverage!("official.selector.attribute-substring"),
    active_coverage!("official.selector.class"),
    active_coverage!("official.selector.id"),
    active_coverage!("official.selector.dynamic"),
    active_coverage!("official.selector.target"),
    active_coverage!("official.selector.lang"),
    active_coverage!("official.selector.ui-state"),
    active_coverage!("official.selector.structural"),
    active_coverage!("official.selector.negation"),
    active_coverage!("official.selector.first-line"),
    active_coverage!("official.selector.first-letter"),
    active_coverage!("official.selector.generated"),
    active_coverage!("official.selector.combinator.descendant"),
    active_coverage!("official.selector.combinator.child"),
    active_coverage!("official.selector.combinator.next-sibling"),
    active_coverage!("official.selector.combinator.subsequent-sibling"),
    active_coverage!("later.rule.namespace"),
    active_coverage!("official.selector.namespace-qualified-name"),
    active_coverage!("baseline.rule.import"),
    active_coverage!("foundation.declaration.importance"),
    active_coverage!("official.value.css-wide-keyword"),
    active_coverage!("official.value.custom-ident"),
    active_coverage!("official.value.ident"),
    active_coverage!("official.value.string"),
    active_coverage!("official.value.url"),
    active_coverage!("official.value.url-modifier"),
    active_coverage!("official.value.integer"),
    active_coverage!("official.value.number"),
    active_coverage!("official.value.dimension"),
    active_coverage!("official.value.percentage"),
    active_coverage!("official.value.length"),
    active_coverage!("official.value.length-percentage"),
    active_coverage!("official.value.angle"),
    active_coverage!("official.value.angle-percentage"),
    active_coverage!("official.value.time"),
    active_coverage!("official.value.time-percentage"),
    active_coverage!("official.value.frequency"),
    active_coverage!("official.value.frequency-percentage"),
    active_coverage!("official.value.resolution"),
    active_coverage!("official.value.position"),
    active_coverage!("official.value.calc"),
    active_coverage!("baseline.value.substitution-dependent"),
    active_coverage!("official.value.box-edge-keywords"),
    active_coverage!("official.value.color"),
    active_coverage!("official.value.alpha"),
    active_coverage!("official.value.hue"),
    active_coverage!("official.value.rgb"),
    active_coverage!("official.value.hex-color"),
    active_coverage!("official.value.named-color"),
    active_coverage!("official.value.system-color"),
    active_coverage!("official.value.deprecated-system-color"),
    active_coverage!("official.value.transparent"),
    active_coverage!("official.value.currentcolor"),
    active_coverage!("official.value.hsl"),
    active_coverage!("official.value.hwb"),
    active_coverage!("official.value.lab"),
    active_coverage!("official.value.lch"),
    active_coverage!("official.value.oklab"),
    active_coverage!("official.value.oklch"),
    active_coverage!("official.value.predefined-color"),
    active_coverage!("official.value.background-layer"),
    active_coverage!("official.value.background-image"),
    active_coverage!("official.value.repeat-style"),
    active_coverage!("official.value.background-attachment"),
    active_coverage!("official.value.background-position"),
    active_coverage!("official.value.background-size"),
    active_coverage!("official.value.line-style"),
    active_coverage!("official.value.line-width"),
    active_coverage!("official.value.shadow"),
    active_coverage!("official.value.image"),
    active_coverage!("official.value.gradient"),
    active_coverage!("official.value.linear-gradient"),
    active_coverage!("official.value.radial-gradient"),
    active_coverage!("official.value.repeating-linear-gradient"),
    active_coverage!("official.value.repeating-radial-gradient"),
    active_coverage!("official.value.color-stop-list"),
    active_coverage!("official.value.side-or-corner"),
    active_coverage!("official.value.radial-shape"),
    active_coverage!("official.value.radial-size"),
    active_coverage!("official.value.radial-extent"),
    active_coverage!("baseline.rule.font-face"),
    active_coverage!("baseline.descriptor.font-family"),
    active_coverage!("baseline.descriptor.src"),
    active_coverage!("baseline.descriptor.font-style"),
    active_coverage!("baseline.descriptor.font-weight"),
    active_coverage!("baseline.descriptor.font-stretch"),
    active_coverage!("baseline.descriptor.unicode-range"),
    active_coverage!("official.descriptor.font-feature-settings"),
    active_coverage!("official.value.font-source"),
    active_coverage!("official.value.opentype-tag"),
    active_coverage!("official.value.transform-list"),
    active_coverage!("official.value.transform-function"),
    active_coverage!("official.value.transform.matrix"),
    active_coverage!("official.value.transform.translate"),
    active_coverage!("official.value.transform.translate-x"),
    active_coverage!("official.value.transform.translate-y"),
    active_coverage!("official.value.transform.scale"),
    active_coverage!("official.value.transform.scale-x"),
    active_coverage!("official.value.transform.scale-y"),
    active_coverage!("official.value.transform.rotate"),
    active_coverage!("official.value.transform.skew"),
    active_coverage!("official.value.transform.skew-x"),
    active_coverage!("official.value.transform.skew-y"),
    active_coverage!("official.value.blend-mode"),
    active_coverage!("official.value.easing-function"),
    active_coverage!("official.value.cubic-bezier-easing"),
    active_coverage!("official.value.step-easing"),
    active_coverage!("official.value.step-position"),
    active_coverage!("later.rule.counter-style"),
    active_coverage!("official.descriptor.counter-style.system"),
    active_coverage!("official.descriptor.counter-style.negative"),
    active_coverage!("official.descriptor.counter-style.prefix"),
    active_coverage!("official.descriptor.counter-style.suffix"),
    active_coverage!("official.descriptor.counter-style.range"),
    active_coverage!("official.descriptor.counter-style.pad"),
    active_coverage!("official.descriptor.counter-style.fallback"),
    active_coverage!("official.descriptor.counter-style.symbols"),
    active_coverage!("official.descriptor.counter-style.additive-symbols"),
    active_coverage!("official.descriptor.counter-style.speak-as"),
    active_coverage!("official.value.counter-style"),
    active_coverage!("official.value.counter-style-name"),
    active_coverage!("official.value.symbol"),
    active_coverage!("official.value.symbols-function"),
    active_coverage!("official.value.symbols-type"),
];

#[expect(
    dead_code,
    reason = "reviewed directly against the 162-row official property ledger"
)]
static OFFICIAL_PROPERTY_COVERAGE: CssOfficialCoverageGroup = CssOfficialCoverageGroup {
    records: OFFICIAL_PROPERTY_COVERAGE_ROWS,
    shared_records: &[CssSharedCoverageRecord {
        ledger_index: 46,
        record: &OFFICIAL_CUSTOM_PROPERTY_COVERAGE,
    }],
};

#[expect(
    dead_code,
    reason = "reviewed directly against the 167-row official non-property ledger"
)]
static OFFICIAL_NON_PROPERTY_COVERAGE: CssOfficialCoverageGroup = CssOfficialCoverageGroup {
    records: OFFICIAL_NON_PROPERTY_COVERAGE_ROWS,
    shared_records: &[CssSharedCoverageRecord {
        ledger_index: 83,
        record: &OFFICIAL_CUSTOM_PROPERTY_COVERAGE,
    }],
};

#[expect(
    dead_code,
    reason = "reviewed directly against the official legacy shorthand"
)]
static OFFICIAL_LEGACY_PROPERTY_ALIAS_COVERAGE: CssOfficialCoverageRecord =
    active_coverage!("official.property-alias.glyph-orientation-vertical");

macro_rules! excluded_coverage {
    ($($index:literal),* $(,)?) => {
        &[
            $(CssOfficialCoverageRecord::Excluded(&CONFORMANCE_EXCLUSIONS[$index]),)*
        ]
    };
}

// The public exclusion registry remains the single metadata owner. These
// mutually-exclusive coverage records borrow every section 2.3/2.4/5 and
// per-source informative row without copying its identity, reason, production,
// or supersession ownership.
#[expect(
    dead_code,
    reason = "reviewed directly against the official exclusion remainder"
)]
static OFFICIAL_EXCLUDED_COVERAGE: &[CssOfficialCoverageRecord] = excluded_coverage!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
);

const BASELINE_RULE_SUBSET: &str =
    "The baseline parser spelling and the I01 recovery extensions are supported.";
const BASELINE_RULE_REMAINDER: &str =
    "Other valid forms of the cited rule production are outside the I01 subset.";
const SELECTOR_REMAINDER: &str =
    "Other valid forms of the cited Selectors production are outside the I01 subset.";
const SUPPORTS_SELECTOR_SUBSET: &str = "selector() accepts complete Selectors 3 plus the selected I01 extensions: i and s attribute modifiers; :scope, :focus-visible, :focus-within, :required, :optional, :valid, :invalid, :placeholder-shown, :modal, :fullscreen, :popover-open, :default, :indeterminate, :read-only, :read-write, :in-range, and :out-of-range; :is(), :where(), :has(), selector-list :not(), and nth-child of lists; and ::marker, ::selection, ::backdrop, and generated-marker sequences.";
const SUPPORTS_SELECTOR_REMAINDER: &str = "The || combinator, unselected Selectors 4 pseudo-classes and pseudo-elements, and syntax outside those atomic extension rows remain outside the typed subset; balanced content is preserved as general-enclosed authored syntax.";
const QUERY_REMAINDER: &str =
    "Other valid forms of the cited query production are outside the I01 subset.";
const PROPERTY_SUBSET: &str = "The property-specific parser behavior at 4b288d6:src/parser/mod.rs, plus whole-value CSS-wide keywords and syntactically admissible substitution-dependent authored values, is supported.";
const PROPERTY_REMAINDER: &str =
    "Other valid forms of the cited property production are outside the I01 subset.";
const TIMING_SUBSET: &str = "The I01 shorthand components plus C03 duration, signed delay, iteration, and typed calculation syntax and C05 easing functions are supported.";
const TIMING_REMAINDER: &str =
    "Other valid forms of the cited shorthand production remain unsupported.";
const BASIC_SHAPE_SUBSET: &str =
    "Typed inset(), circle(), ellipse(), and polygon() functions are supported.";
const BASIC_SHAPE_REMAINDER: &str = "path(), shape(), rect(), and xywh() remain unsupported.";
const BACKDROP_FILTER_SUBSET: &str = "The exact I01 filter-function-list subset preserved at bc5394f:src/parser/effects.rs is supported with typed current values.";
const BACKDROP_FILTER_REMAINDER: &str = "Every Filter Effects 2 behavior absent from that preserved baseline subset remains unsupported.";
const CLIP_PATH_SUBSET: &str =
    "none, URL, and typed inset(), circle(), ellipse(), and polygon() functions are supported.";
const CLIP_PATH_REMAINDER: &str =
    "Reference-box combinations and path(), shape(), rect(), and xywh() remain unsupported.";
const COLOR5_RELATIVE_SUBSET: &str = "The eight preserved relative-color families are supported: rgb()/rgba(), hsl()/hsla(), hwb(), lab(), lch(), oklab(), oklch(), and color() in a predefined RGB or XYZ space.";
const COLOR5_RELATIVE_REMAINDER: &str = "alpha(), custom-profile parameters, and other unselected CSS Color 5 color functions remain unsupported.";
const COLOR5_MIX_SUBSET: &str = "The preserved color-mix() subset requires an interpolation method, exactly two colors, optional trailing percentages, and a predefined or polar color space.";
const COLOR5_MIX_REMAINDER: &str =
    "Other valid forms of the dated CSS Color 5 color-mix() production remain unsupported.";
const GRID_REPEAT_SUBSET: &str = "Non-recursive integer track and fixed repeats, plus one fixed-size automatic repeat where the consumer permits it, are supported.";
const GRID_REPEAT_REMAINDER: &str =
    "Subgrid name-repeat and other unselected Grid 2 forms remain unsupported.";
const GRID_PROPERTY_SUBSET: &str = "The C07 structural grammar supports non-recursive integer track and fixed repeats, one fixed-size automatic repeat where permitted, and repeat-free automatic track-size lists.";
const GRID_PROPERTY_REMAINDER: &str =
    "Subgrid name-repeat and other unselected Grid 2 property grammar remain unsupported.";
const KEYFRAMES_SUBSET: &str = "Keyframe names, literal selectors, empty rules and blocks, duplicate selectors and blocks in authored order, and supported declarations with recovery are supported.";
const KEYFRAMES_REMAINDER: &str = "Calculation selectors, string names, and declaration-processing grammar not selected by C07 remain unsupported.";
const FONT_WEIGHT_RANGE_SUBSET: &str =
    "Integer font-weight values from 1 through 1000 are supported.";
const FONT_WEIGHT_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-weight property grammar remains unsupported.";
const FONT_FACE_WEIGHT_RANGE_SUBSET: &str = "Font-face font-weight numbers from 1 through 1000 and increasing two-value ranges are supported.";
const FONT_FACE_WEIGHT_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-weight descriptor grammar remains unsupported.";
const FONT_FACE_STYLE_RANGE_SUBSET: &str =
    "Font-face oblique style with one or two increasing -90deg through 90deg angles is supported.";
const FONT_FACE_STYLE_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-style descriptor grammar remains unsupported.";
const FONT_FACE_STRETCH_RANGE_SUBSET: &str = "Font-face non-negative percentage stretch values and increasing two-value ranges are supported.";
const FONT_FACE_STRETCH_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-stretch descriptor grammar remains unsupported.";
const FONT_SOURCE_HINTS_SUBSET: &str = "The woff, woff2, truetype, opentype, collection, embedded-opentype, and svg format() hints and the variations, color-colrv0, color-colrv1, color-svg, color-sbix, color-cbdt, features-opentype, features-aat, features-graphite, and incremental tech() hints are supported.";
const FONT_SOURCE_HINTS_REMAINDER: &str =
    "Other unselected Fonts 4 font source format and technology hints remain unsupported.";

const fn property_source(property: CssKnownProperty) -> CssSpecificationSource {
    match property {
        CssKnownProperty::All => O_CASCADE4,
        CssKnownProperty::Display
        | CssKnownProperty::BorderCollapse
        | CssKnownProperty::BorderSpacing
        | CssKnownProperty::CaptionSide
        | CssKnownProperty::Clip
        | CssKnownProperty::EmptyCells
        | CssKnownProperty::Orphans
        | CssKnownProperty::PageBreakAfter
        | CssKnownProperty::PageBreakBefore
        | CssKnownProperty::PageBreakInside
        | CssKnownProperty::Quotes
        | CssKnownProperty::TableLayout
        | CssKnownProperty::Widows
        | CssKnownProperty::WordSpacing
        | CssKnownProperty::Position
        | CssKnownProperty::Overflow
        | CssKnownProperty::Float
        | CssKnownProperty::Clear
        | CssKnownProperty::Visibility
        | CssKnownProperty::Content
        | CssKnownProperty::ListStyleType
        | CssKnownProperty::ListStylePosition
        | CssKnownProperty::ListStyleImage
        | CssKnownProperty::ListStyle
        | CssKnownProperty::CounterReset
        | CssKnownProperty::CounterIncrement
        | CssKnownProperty::Width
        | CssKnownProperty::Height
        | CssKnownProperty::MinWidth
        | CssKnownProperty::MinHeight
        | CssKnownProperty::MaxWidth
        | CssKnownProperty::MaxHeight
        | CssKnownProperty::LineHeight
        | CssKnownProperty::TextAlign
        | CssKnownProperty::TextIndent
        | CssKnownProperty::VerticalAlign
        | CssKnownProperty::LetterSpacing
        | CssKnownProperty::WhiteSpace
        | CssKnownProperty::TextDecoration
        | CssKnownProperty::TextTransform
        | CssKnownProperty::Top
        | CssKnownProperty::Right
        | CssKnownProperty::Bottom
        | CssKnownProperty::Left
        | CssKnownProperty::ZIndex => O_CSS2,
        CssKnownProperty::BoxSizing
        | CssKnownProperty::CaretColor
        | CssKnownProperty::OutlineOffset
        | CssKnownProperty::Resize
        | CssKnownProperty::TextOverflow
        | CssKnownProperty::Cursor
        | CssKnownProperty::Outline
        | CssKnownProperty::OutlineColor
        | CssKnownProperty::OutlineStyle
        | CssKnownProperty::OutlineWidth => O_UI3,
        CssKnownProperty::Contain => O_CONTAIN1,
        CssKnownProperty::Direction
        | CssKnownProperty::TextCombineUpright
        | CssKnownProperty::TextOrientation
        | CssKnownProperty::UnicodeBidi
        | CssKnownProperty::WritingMode => O_WRITING3,
        CssKnownProperty::ColumnCount
        | CssKnownProperty::ColumnFill
        | CssKnownProperty::ColumnRule
        | CssKnownProperty::ColumnRuleColor
        | CssKnownProperty::ColumnRuleStyle
        | CssKnownProperty::ColumnRuleWidth
        | CssKnownProperty::ColumnSpan
        | CssKnownProperty::ColumnWidth
        | CssKnownProperty::Columns => O_MULTICOL1,
        CssKnownProperty::FlexDirection
        | CssKnownProperty::FlexFlow
        | CssKnownProperty::FlexWrap
        | CssKnownProperty::AlignContent
        | CssKnownProperty::JustifyContent
        | CssKnownProperty::AlignItems
        | CssKnownProperty::AlignSelf
        | CssKnownProperty::FlexBasis
        | CssKnownProperty::FlexGrow
        | CssKnownProperty::FlexShrink
        | CssKnownProperty::Flex => O_FLEXBOX1,
        CssKnownProperty::Margin
        | CssKnownProperty::MarginTop
        | CssKnownProperty::MarginRight
        | CssKnownProperty::MarginBottom
        | CssKnownProperty::MarginLeft
        | CssKnownProperty::Padding
        | CssKnownProperty::PaddingTop
        | CssKnownProperty::PaddingRight
        | CssKnownProperty::PaddingBottom
        | CssKnownProperty::PaddingLeft => O_BOX3,
        CssKnownProperty::Color | CssKnownProperty::Opacity => O_COLOR4,
        CssKnownProperty::Border
        | CssKnownProperty::BorderTop
        | CssKnownProperty::BorderRight
        | CssKnownProperty::BorderBottom
        | CssKnownProperty::BorderLeft
        | CssKnownProperty::BorderWidth
        | CssKnownProperty::BorderTopWidth
        | CssKnownProperty::BorderRightWidth
        | CssKnownProperty::BorderBottomWidth
        | CssKnownProperty::BorderLeftWidth
        | CssKnownProperty::Background
        | CssKnownProperty::BackgroundColor
        | CssKnownProperty::BorderColor
        | CssKnownProperty::BorderTopColor
        | CssKnownProperty::BorderRightColor
        | CssKnownProperty::BorderBottomColor
        | CssKnownProperty::BorderLeftColor
        | CssKnownProperty::BackgroundImage
        | CssKnownProperty::BackgroundPosition
        | CssKnownProperty::BackgroundSize
        | CssKnownProperty::BackgroundRepeat
        | CssKnownProperty::BackgroundOrigin
        | CssKnownProperty::BackgroundClip
        | CssKnownProperty::BackgroundAttachment
        | CssKnownProperty::BorderImage
        | CssKnownProperty::BorderImageOutset
        | CssKnownProperty::BorderImageRepeat
        | CssKnownProperty::BorderImageSlice
        | CssKnownProperty::BorderImageSource
        | CssKnownProperty::BorderImageWidth
        | CssKnownProperty::BorderStyle
        | CssKnownProperty::BorderTopStyle
        | CssKnownProperty::BorderRightStyle
        | CssKnownProperty::BorderBottomStyle
        | CssKnownProperty::BorderLeftStyle
        | CssKnownProperty::BorderRadius
        | CssKnownProperty::BorderTopLeftRadius
        | CssKnownProperty::BorderTopRightRadius
        | CssKnownProperty::BorderBottomRightRadius
        | CssKnownProperty::BorderBottomLeftRadius
        | CssKnownProperty::BoxShadow => O_BACKGROUNDS3,
        CssKnownProperty::ImageOrientation
        | CssKnownProperty::ImageRendering
        | CssKnownProperty::ObjectFit
        | CssKnownProperty::ObjectPosition => O_IMAGES3,
        CssKnownProperty::FontSize
        | CssKnownProperty::FontFamily
        | CssKnownProperty::Font
        | CssKnownProperty::FontWeight
        | CssKnownProperty::FontStyle
        | CssKnownProperty::FontStretch
        | CssKnownProperty::FontVariant
        | CssKnownProperty::FontVariantCaps
        | CssKnownProperty::FontVariantEastAsian
        | CssKnownProperty::FontVariantLigatures
        | CssKnownProperty::FontVariantNumeric
        | CssKnownProperty::FontVariantPosition
        | CssKnownProperty::FontFeatureSettings
        | CssKnownProperty::FontKerning
        | CssKnownProperty::FontSizeAdjust
        | CssKnownProperty::FontSynthesis => O_FONTS3,
        CssKnownProperty::OverflowX | CssKnownProperty::OverflowY => X_OVERFLOW3,
        CssKnownProperty::JustifyItems
        | CssKnownProperty::JustifySelf
        | CssKnownProperty::PlaceContent
        | CssKnownProperty::PlaceItems
        | CssKnownProperty::PlaceSelf
        | CssKnownProperty::Gap
        | CssKnownProperty::RowGap
        | CssKnownProperty::ColumnGap
        | CssKnownProperty::JustifyTracks
        | CssKnownProperty::AlignTracks => S_ALIGN3,
        CssKnownProperty::ContentVisibility => I_CONTAIN2,
        CssKnownProperty::CounterSet => I_LISTS3,
        CssKnownProperty::GridFlowTolerance => X_GRID_TOLERANCE_BASE,
        CssKnownProperty::GridTemplateRows
        | CssKnownProperty::GridTemplateColumns
        | CssKnownProperty::GridTemplateAreas
        | CssKnownProperty::GridTemplate
        | CssKnownProperty::GridAutoRows
        | CssKnownProperty::GridAutoColumns
        | CssKnownProperty::GridAutoFlow
        | CssKnownProperty::GridRowStart
        | CssKnownProperty::GridRowEnd
        | CssKnownProperty::GridColumnStart
        | CssKnownProperty::GridColumnEnd
        | CssKnownProperty::GridRow
        | CssKnownProperty::GridColumn
        | CssKnownProperty::GridArea
        | CssKnownProperty::Grid => R_GRID2,
        CssKnownProperty::TextAlignLast
        | CssKnownProperty::WordBreak
        | CssKnownProperty::OverflowWrap => S_TEXT3,
        CssKnownProperty::TextWrap => X_TEXT4,
        CssKnownProperty::TextDecorationLine
        | CssKnownProperty::TextDecorationColor
        | CssKnownProperty::TextDecorationStyle => S_TEXTDECOR3,
        CssKnownProperty::TextDecorationThickness => X_TEXTDECOR4,
        CssKnownProperty::Inset => I_POSITION3,
        CssKnownProperty::BoxDecorationBreak => S_BREAK3,
        CssKnownProperty::Order => S_DISPLAY3,
        CssKnownProperty::AspectRatio => I_SIZING3,
        CssKnownProperty::ScrollbarWidth => R_SCROLLBARS1,
        CssKnownProperty::PointerEvents | CssKnownProperty::UserSelect => X_UI4,
        CssKnownProperty::Transform
        | CssKnownProperty::TransformBox
        | CssKnownProperty::TransformOrigin => O_TRANSFORMS1,
        CssKnownProperty::BackgroundBlendMode
        | CssKnownProperty::Isolation
        | CssKnownProperty::MixBlendMode => O_COMPOSITING1,
        CssKnownProperty::Translate | CssKnownProperty::Rotate | CssKnownProperty::Scale => {
            I_TRANSFORMS2
        }
        CssKnownProperty::Filter => I_FILTER1,
        CssKnownProperty::BackdropFilter => X_FILTER2_BASE,
        CssKnownProperty::ClipPath
        | CssKnownProperty::Mask
        | CssKnownProperty::MaskImage
        | CssKnownProperty::MaskSize
        | CssKnownProperty::MaskPosition
        | CssKnownProperty::MaskRepeat => S_MASKING1,
        CssKnownProperty::TransitionProperty
        | CssKnownProperty::TransitionDuration
        | CssKnownProperty::TransitionDelay
        | CssKnownProperty::TransitionTimingFunction
        | CssKnownProperty::Transition => I_TRANSITIONS1,
        CssKnownProperty::AnimationName
        | CssKnownProperty::AnimationDuration
        | CssKnownProperty::AnimationDelay
        | CssKnownProperty::AnimationTimingFunction
        | CssKnownProperty::AnimationIterationCount
        | CssKnownProperty::AnimationDirection
        | CssKnownProperty::AnimationFillMode
        | CssKnownProperty::AnimationPlayState
        | CssKnownProperty::Animation => I_ANIMATIONS1,
    }
}

const fn property_production(property: CssKnownProperty, default: &'static str) -> &'static str {
    match property {
        CssKnownProperty::BorderCollapse
        | CssKnownProperty::BorderSpacing
        | CssKnownProperty::CaptionSide
        | CssKnownProperty::EmptyCells
        | CssKnownProperty::TableLayout => match property {
            CssKnownProperty::BorderCollapse => "tables.html#propdef-border-collapse",
            CssKnownProperty::BorderSpacing => "tables.html#propdef-border-spacing",
            CssKnownProperty::CaptionSide => "tables.html#propdef-caption-side",
            CssKnownProperty::EmptyCells => "tables.html#propdef-empty-cells",
            CssKnownProperty::TableLayout => "tables.html#propdef-table-layout",
            _ => default,
        },
        CssKnownProperty::Clip => "visufx.html#propdef-clip",
        CssKnownProperty::Orphans
        | CssKnownProperty::PageBreakAfter
        | CssKnownProperty::PageBreakBefore
        | CssKnownProperty::PageBreakInside
        | CssKnownProperty::Widows => match property {
            CssKnownProperty::Orphans => "page.html#propdef-orphans",
            CssKnownProperty::PageBreakAfter => "page.html#propdef-page-break-after",
            CssKnownProperty::PageBreakBefore => "page.html#propdef-page-break-before",
            CssKnownProperty::PageBreakInside => "page.html#propdef-page-break-inside",
            CssKnownProperty::Widows => "page.html#propdef-widows",
            _ => default,
        },
        CssKnownProperty::Quotes => "generate.html#propdef-quotes",
        CssKnownProperty::WordSpacing => "text.html#propdef-word-spacing",
        CssKnownProperty::Display
        | CssKnownProperty::Position
        | CssKnownProperty::Float
        | CssKnownProperty::Clear
        | CssKnownProperty::Top
        | CssKnownProperty::Right
        | CssKnownProperty::Bottom
        | CssKnownProperty::Left
        | CssKnownProperty::ZIndex => match property {
            CssKnownProperty::Display => "visuren.html#propdef-display",
            CssKnownProperty::Position => "visuren.html#propdef-position",
            CssKnownProperty::Float => "visuren.html#propdef-float",
            CssKnownProperty::Clear => "visuren.html#propdef-clear",
            CssKnownProperty::Top => "visuren.html#propdef-top",
            CssKnownProperty::Right => "visuren.html#propdef-right",
            CssKnownProperty::Bottom => "visuren.html#propdef-bottom",
            CssKnownProperty::Left => "visuren.html#propdef-left",
            CssKnownProperty::ZIndex => "visuren.html#propdef-z-index",
            _ => default,
        },
        CssKnownProperty::Overflow | CssKnownProperty::Visibility => match property {
            CssKnownProperty::Overflow => "visufx.html#propdef-overflow",
            CssKnownProperty::Visibility => "visufx.html#propdef-visibility",
            _ => default,
        },
        CssKnownProperty::Content
        | CssKnownProperty::CounterIncrement
        | CssKnownProperty::CounterReset
        | CssKnownProperty::ListStyle
        | CssKnownProperty::ListStyleImage
        | CssKnownProperty::ListStylePosition
        | CssKnownProperty::ListStyleType => match property {
            CssKnownProperty::Content => "generate.html#propdef-content",
            CssKnownProperty::CounterIncrement => "generate.html#propdef-counter-increment",
            CssKnownProperty::CounterReset => "generate.html#propdef-counter-reset",
            CssKnownProperty::ListStyle => "generate.html#propdef-list-style",
            CssKnownProperty::ListStyleImage => "generate.html#propdef-list-style-image",
            CssKnownProperty::ListStylePosition => "generate.html#propdef-list-style-position",
            CssKnownProperty::ListStyleType => "generate.html#propdef-list-style-type",
            _ => default,
        },
        CssKnownProperty::Width
        | CssKnownProperty::Height
        | CssKnownProperty::MinWidth
        | CssKnownProperty::MinHeight
        | CssKnownProperty::MaxWidth
        | CssKnownProperty::MaxHeight
        | CssKnownProperty::LineHeight
        | CssKnownProperty::VerticalAlign => match property {
            CssKnownProperty::Width => "visudet.html#propdef-width",
            CssKnownProperty::Height => "visudet.html#propdef-height",
            CssKnownProperty::MinWidth => "visudet.html#propdef-min-width",
            CssKnownProperty::MinHeight => "visudet.html#propdef-min-height",
            CssKnownProperty::MaxWidth => "visudet.html#propdef-max-width",
            CssKnownProperty::MaxHeight => "visudet.html#propdef-max-height",
            CssKnownProperty::LineHeight => "visudet.html#propdef-line-height",
            CssKnownProperty::VerticalAlign => "visudet.html#propdef-vertical-align",
            _ => default,
        },
        CssKnownProperty::LetterSpacing
        | CssKnownProperty::TextAlign
        | CssKnownProperty::TextDecoration
        | CssKnownProperty::TextIndent
        | CssKnownProperty::TextTransform
        | CssKnownProperty::WhiteSpace => match property {
            CssKnownProperty::LetterSpacing => "text.html#propdef-letter-spacing",
            CssKnownProperty::TextAlign => "text.html#propdef-text-align",
            CssKnownProperty::TextDecoration => "text.html#propdef-text-decoration",
            CssKnownProperty::TextIndent => "text.html#propdef-text-indent",
            CssKnownProperty::TextTransform => "text.html#propdef-text-transform",
            CssKnownProperty::WhiteSpace => "text.html#propdef-white-space",
            _ => default,
        },
        _ => default,
    }
}

macro_rules! property_feature {
    ($property:path, $canonical_name:literal, $stable_id:literal) => {
        CssFeatureMetadata::partial_property(
            $stable_id,
            $property,
            $canonical_name,
            concat!("#propdef-", $canonical_name),
            &[],
        )
    };
}

macro_rules! complete_property_feature {
    ($property:path, $canonical_name:literal, $stable_id:literal) => {
        CssFeatureMetadata::complete_property(
            $stable_id,
            $property,
            $canonical_name,
            concat!("#propdef-", $canonical_name),
            &[],
        )
    };
}

const PSEUDO_ELEMENT_ALIAS_TARGETS: &[CssFeatureId] = &[
    CssFeatureId::new("official.selector.generated"),
    CssFeatureId::new("ext.pseudo-element.marker"),
    CssFeatureId::new("ext.pseudo-element.selection"),
    CssFeatureId::new("ext.pseudo-element.backdrop"),
    CssFeatureId::new("ext.pseudo-element.generated-marker"),
];

const MEDIA_QUERY_LIST_ALIAS_TARGETS: &[CssFeatureId] = &[
    CssFeatureId::new("official.media.query-list-core"),
    CssFeatureId::new("ext.media.condition-syntax"),
    CssFeatureId::new("ext.media.malformed-member-never"),
];

const MEDIA_RANGE_ALIAS_TARGETS: &[CssFeatureId] = &[
    CssFeatureId::new("official.media.feature.width"),
    CssFeatureId::new("official.media.feature.height"),
    CssFeatureId::new("official.media.feature.resolution"),
    CssFeatureId::new("official.media.feature.color"),
    CssFeatureId::new("official.media.feature.monochrome"),
    CssFeatureId::new("ext.media.resolution.dppx"),
    CssFeatureId::new("ext.media.range.width"),
    CssFeatureId::new("ext.media.range.height"),
    CssFeatureId::new("ext.media.range.resolution"),
    CssFeatureId::new("ext.media.range.color"),
    CssFeatureId::new("ext.media.range.monochrome"),
];

const MEDIA_DISCRETE_ALIAS_TARGETS: &[CssFeatureId] = &[
    CssFeatureId::new("official.media.feature.orientation"),
    CssFeatureId::new("ext.media.hover"),
    CssFeatureId::new("ext.media.any-hover"),
    CssFeatureId::new("ext.media.pointer"),
    CssFeatureId::new("ext.media.any-pointer"),
    CssFeatureId::new("ext.media.prefers-color-scheme"),
    CssFeatureId::new("ext.media.prefers-reduced-motion"),
    CssFeatureId::new("ext.media.prefers-reduced-transparency"),
    CssFeatureId::new("ext.media.prefers-contrast"),
    CssFeatureId::new("ext.media.forced-colors"),
    CssFeatureId::new("ext.media.display-mode"),
];

static FEATURE_CATALOG: [CssFeatureMetadata; 487] = [
    CssFeatureMetadata::complete(
        "baseline.rule.import",
        CssFeatureKind::Rule,
        "@import",
        O_CASCADE4,
        "#at-import",
    ),
    CssFeatureMetadata::complete(
        "ext.import.layer",
        CssFeatureKind::Rule,
        "@import layer or layer() clause",
        R_CASCADE5,
        "#at-import",
    ),
    CssFeatureMetadata::complete(
        "ext.stylesheet.prelude-order",
        CssFeatureKind::Rule,
        "initial layer statements, imports, namespaces, and body-rule ordering",
        R_CASCADE5,
        "#at-import",
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.layer-statement",
        CssFeatureKind::Rule,
        "@layer ...;",
        R_CASCADE5,
        "#layering",
        BASELINE_RULE_SUBSET,
        BASELINE_RULE_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.layer-block",
        CssFeatureKind::Rule,
        "@layer {...}",
        R_CASCADE5,
        "#layering",
        BASELINE_RULE_SUBSET,
        BASELINE_RULE_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "baseline.rule.font-face",
        CssFeatureKind::Rule,
        "@font-face",
        O_FONTS3,
        "#font-face-rule",
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.keyframes",
        CssFeatureKind::Rule,
        "@keyframes",
        I_ANIMATIONS1,
        "#keyframes",
        KEYFRAMES_SUBSET,
        KEYFRAMES_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.style",
        CssFeatureKind::Rule,
        "style and nested qualified rules",
        O_SYNTAX3,
        "#style-rules",
        BASELINE_RULE_SUBSET,
        BASELINE_RULE_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "official.rule.at-rule",
        CssFeatureKind::Rule,
        "generic at-rule",
        O_SYNTAX3,
        "#at-rules,#consume-at-rule",
    ),
    CssFeatureMetadata::complete(
        "official.qualified-rule.generic",
        CssFeatureKind::Rule,
        "generic qualified rule",
        O_SYNTAX3,
        "#consume-qualified-rule",
    ),
    CssFeatureMetadata::complete(
        "baseline.rule.media",
        CssFeatureKind::Rule,
        "@media",
        O_CONDITIONAL3,
        "#at-media",
    ),
    CssFeatureMetadata::complete(
        "official.rule.conditional-group-context",
        CssFeatureKind::Rule,
        "conditional group rule contents and placement",
        O_CONDITIONAL3,
        "#contents,#placement",
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.container",
        CssFeatureKind::Rule,
        "@container",
        X_CONTAIN3,
        "#container-rule",
        BASELINE_RULE_SUBSET,
        BASELINE_RULE_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.scope",
        CssFeatureKind::Rule,
        "@scope",
        X_CASCADE6,
        "#scope-atrule",
        BASELINE_RULE_SUBSET,
        BASELINE_RULE_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "foundation.encoding.charset",
        CssFeatureKind::Rule,
        "optional leading legacy @charset metadata",
        CSS_SYNTAX_3,
        "#charset-rule",
    ),
    CssFeatureMetadata::complete(
        "foundation.declaration-list.style-attribute",
        CssFeatureKind::Declaration,
        "style-attribute declaration-list structure",
        CSS_STYLE_ATTRIBUTES,
        "#syntax",
    ),
    CssFeatureMetadata::complete(
        "foundation.declaration.importance",
        CssFeatureKind::Declaration,
        "terminal declaration !important annotation",
        CSS_CASCADE_4,
        "#importance",
    ),
    CssFeatureMetadata::complete(
        "official.declaration.generic",
        CssFeatureKind::Declaration,
        "generic declaration",
        O_SYNTAX3,
        "#consume-declaration",
    ),
    CssFeatureMetadata::complete(
        "official.value.syntax-token-stream",
        CssFeatureKind::Value,
        "syntax token stream",
        O_SYNTAX3,
        "#tokenization",
    ),
    CssFeatureMetadata::complete(
        "official.value.component-value",
        CssFeatureKind::Value,
        "<component-value>",
        O_SYNTAX3,
        "#consume-component-value",
    ),
    CssFeatureMetadata::complete(
        "official.value.simple-block",
        CssFeatureKind::Value,
        "simple block",
        O_SYNTAX3,
        "#consume-simple-block",
    ),
    CssFeatureMetadata::complete(
        "official.value.function",
        CssFeatureKind::Value,
        "function",
        O_SYNTAX3,
        "#consume-function",
    ),
    CssFeatureMetadata::complete(
        "official.value.stylesheet",
        CssFeatureKind::Value,
        "<stylesheet>",
        O_SYNTAX3,
        "#parser-entry-points",
    ),
    CssFeatureMetadata::complete(
        "official.value.rule-list",
        CssFeatureKind::Value,
        "<rule-list>",
        O_SYNTAX3,
        "#declaration-rule-list",
    ),
    CssFeatureMetadata::complete(
        "official.value.declaration-list",
        CssFeatureKind::Value,
        "<declaration-list>",
        O_SYNTAX3,
        "#declaration-rule-list",
    ),
    CssFeatureMetadata::complete(
        "official.value.style-block",
        CssFeatureKind::Value,
        "<style-block>",
        O_SYNTAX3,
        "#declaration-rule-list",
    ),
    CssFeatureMetadata::complete(
        "official.value.declaration-value",
        CssFeatureKind::Value,
        "<declaration-value>",
        O_SYNTAX3,
        "#any-value",
    ),
    CssFeatureMetadata::complete(
        "official.value.any-value",
        CssFeatureKind::Value,
        "<any-value>",
        O_SYNTAX3,
        "#any-value",
    ),
    CssFeatureMetadata::complete(
        "official.value.an-plus-b",
        CssFeatureKind::Value,
        "<an+b>",
        O_SYNTAX3,
        "#the-anb-type",
    ),
    CssFeatureMetadata::complete(
        "official.value.unicode-range",
        CssFeatureKind::Value,
        "<urange>",
        O_SYNTAX3,
        "#urange-syntax",
    ),
    CssFeatureMetadata::partial(
        "baseline.declaration.custom-property",
        CssFeatureKind::Declaration,
        "custom-property names and authored token streams",
        O_VARIABLES1,
        "#defining-variables,#syntax",
        "Baseline custom-property names and authored token streams, including I01 recovery behavior, are supported.",
        "Other valid CSS Variables custom-property declaration forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::partial(
        "baseline.value.substitution-dependent",
        CssFeatureKind::Value,
        "preserved known-property values containing substitution functions",
        O_VARIABLES1,
        "#using-variables",
        "Known-property values with syntactically admissible var() references remain authored and symbolic.",
        "Other valid CSS Variables substitution functions and post-substitution forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::complete(
        "official.value.css-wide-keyword",
        CssFeatureKind::Value,
        "CSS-wide keyword",
        O_CASCADE4,
        "#defaulting-keywords",
    ),
    CssFeatureMetadata::complete(
        "official.value.custom-ident",
        CssFeatureKind::Value,
        "<custom-ident>",
        O_VALUES3,
        "#custom-idents",
    ),
    CssFeatureMetadata::complete(
        "official.value.ident",
        CssFeatureKind::Value,
        "<ident>",
        O_VALUES3,
        "#custom-idents",
    ),
    CssFeatureMetadata::complete(
        "official.value.string",
        CssFeatureKind::Value,
        "<string>",
        O_VALUES3,
        "#strings",
    ),
    CssFeatureMetadata::complete(
        "official.value.url",
        CssFeatureKind::Value,
        "<url>",
        O_VALUES3,
        "#urls",
    ),
    CssFeatureMetadata::complete(
        "official.value.url-modifier",
        CssFeatureKind::Value,
        "<url-modifier>",
        O_VALUES3,
        "#url-modifiers",
    ),
    CssFeatureMetadata::complete(
        "official.value.integer",
        CssFeatureKind::Value,
        "<integer>",
        O_VALUES3,
        "#integers",
    ),
    CssFeatureMetadata::complete(
        "official.value.number",
        CssFeatureKind::Value,
        "<number>",
        O_VALUES3,
        "#numbers",
    ),
    CssFeatureMetadata::complete(
        "official.value.dimension",
        CssFeatureKind::Value,
        "<dimension>",
        O_VALUES3,
        "#dimensions",
    ),
    CssFeatureMetadata::complete(
        "official.value.percentage",
        CssFeatureKind::Value,
        "<percentage>",
        O_VALUES3,
        "#percentages",
    ),
    CssFeatureMetadata::complete(
        "official.value.length",
        CssFeatureKind::Value,
        "<length>",
        O_VALUES3,
        "#lengths",
    ),
    CssFeatureMetadata::complete(
        "official.value.length-percentage",
        CssFeatureKind::Value,
        "<length-percentage>",
        O_VALUES3,
        "#mixed-percentages",
    ),
    CssFeatureMetadata::complete(
        "official.value.angle",
        CssFeatureKind::Value,
        "<angle>",
        O_VALUES3,
        "#angles",
    ),
    CssFeatureMetadata::complete(
        "official.value.angle-percentage",
        CssFeatureKind::Value,
        "<angle-percentage>",
        O_VALUES3,
        "#mixed-percentages",
    ),
    CssFeatureMetadata::complete(
        "official.value.time",
        CssFeatureKind::Value,
        "<time>",
        O_VALUES3,
        "#time",
    ),
    CssFeatureMetadata::complete(
        "official.value.time-percentage",
        CssFeatureKind::Value,
        "<time-percentage>",
        O_VALUES3,
        "#mixed-percentages",
    ),
    CssFeatureMetadata::complete(
        "official.value.frequency",
        CssFeatureKind::Value,
        "<frequency>",
        O_VALUES3,
        "#frequency",
    ),
    CssFeatureMetadata::complete(
        "official.value.frequency-percentage",
        CssFeatureKind::Value,
        "<frequency-percentage>",
        O_VALUES3,
        "#mixed-percentages",
    ),
    CssFeatureMetadata::complete(
        "official.value.resolution",
        CssFeatureKind::Value,
        "<resolution>",
        O_VALUES3,
        "#resolution",
    ),
    CssFeatureMetadata::complete(
        "official.value.color",
        CssFeatureKind::Value,
        "<color>",
        O_COLOR4,
        "#color-type",
    ),
    CssFeatureMetadata::complete(
        "official.value.alpha",
        CssFeatureKind::Value,
        "<alpha-value>",
        O_COLOR4,
        "#alpha-syntax",
    ),
    CssFeatureMetadata::complete(
        "official.value.hue",
        CssFeatureKind::Value,
        "<hue>",
        O_COLOR4,
        "#hue-syntax",
    ),
    CssFeatureMetadata::complete(
        "official.value.rgb",
        CssFeatureKind::Value,
        "rgb()/rgba()",
        O_COLOR4,
        "#rgb-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.hex-color",
        CssFeatureKind::Value,
        "<hex-color>",
        O_COLOR4,
        "#hex-notation",
    ),
    CssFeatureMetadata::complete(
        "official.value.named-color",
        CssFeatureKind::Value,
        "<named-color>",
        O_COLOR4,
        "#named-colors",
    ),
    CssFeatureMetadata::complete(
        "official.value.system-color",
        CssFeatureKind::Value,
        "<system-color>",
        O_COLOR4,
        "#css-system-colors",
    ),
    CssFeatureMetadata::complete(
        "official.value.deprecated-system-color",
        CssFeatureKind::Value,
        "<deprecated-system-color>",
        O_COLOR4,
        "#css-system-colors",
    ),
    CssFeatureMetadata::complete(
        "official.value.transparent",
        CssFeatureKind::Value,
        "transparent",
        O_COLOR4,
        "#transparent-color",
    ),
    CssFeatureMetadata::complete(
        "official.value.currentcolor",
        CssFeatureKind::Value,
        "currentColor",
        O_COLOR4,
        "#currentcolor-color",
    ),
    CssFeatureMetadata::complete(
        "official.value.hsl",
        CssFeatureKind::Value,
        "hsl()/hsla()",
        O_COLOR4,
        "#the-hsl-notation",
    ),
    CssFeatureMetadata::complete(
        "official.value.hwb",
        CssFeatureKind::Value,
        "hwb()",
        O_COLOR4,
        "#the-hwb-notation",
    ),
    CssFeatureMetadata::complete(
        "official.value.lab",
        CssFeatureKind::Value,
        "lab()",
        O_COLOR4,
        "#specifying-lab-lch",
    ),
    CssFeatureMetadata::complete(
        "official.value.lch",
        CssFeatureKind::Value,
        "lch()",
        O_COLOR4,
        "#specifying-lab-lch",
    ),
    CssFeatureMetadata::complete(
        "official.value.oklab",
        CssFeatureKind::Value,
        "oklab()",
        O_COLOR4,
        "#specifying-oklab-oklch",
    ),
    CssFeatureMetadata::complete(
        "official.value.oklch",
        CssFeatureKind::Value,
        "oklch()",
        O_COLOR4,
        "#specifying-oklab-oklch",
    ),
    CssFeatureMetadata::complete(
        "official.value.predefined-color",
        CssFeatureKind::Value,
        "color()",
        O_COLOR4,
        "#color-function",
    ),
    CssFeatureMetadata::partial(
        "ext.value.relative-color",
        CssFeatureKind::Value,
        "relative color syntax",
        I_COLOR5,
        "#relative-colors,#relative-syntax",
        COLOR5_RELATIVE_SUBSET,
        COLOR5_RELATIVE_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.rgb",
        CssFeatureKind::Value,
        "relative rgb()/rgba()",
        I_COLOR5,
        "#relative-RGB",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.hsl",
        CssFeatureKind::Value,
        "relative hsl()/hsla()",
        I_COLOR5,
        "#relative-HSL",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.hwb",
        CssFeatureKind::Value,
        "relative hwb()",
        I_COLOR5,
        "#relative-HWB",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.lab",
        CssFeatureKind::Value,
        "relative lab()",
        I_COLOR5,
        "#relative-Lab",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.oklab",
        CssFeatureKind::Value,
        "relative oklab()",
        I_COLOR5,
        "#relative-Oklab",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.lch",
        CssFeatureKind::Value,
        "relative lch()",
        I_COLOR5,
        "#relative-LCH",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.oklch",
        CssFeatureKind::Value,
        "relative oklch()",
        I_COLOR5,
        "#relative-OkLCh",
    ),
    CssFeatureMetadata::complete(
        "ext.value.relative-color.predefined",
        CssFeatureKind::Value,
        "relative color()",
        I_COLOR5,
        "#relative-color-function",
    ),
    CssFeatureMetadata::partial(
        "ext.value.color-mix",
        CssFeatureKind::Value,
        "color-mix()",
        I_COLOR5,
        "#funcdef-color-mix",
        COLOR5_MIX_SUBSET,
        COLOR5_MIX_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "ext.value.grid-repeat",
        CssFeatureKind::Value,
        "repeat()",
        R_GRID2,
        "#repeat-notation",
        GRID_REPEAT_SUBSET,
        GRID_REPEAT_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "official.value.position",
        CssFeatureKind::Value,
        "<position>",
        O_VALUES3,
        "#position",
    ),
    CssFeatureMetadata::complete(
        "official.value.box-edge-keywords",
        CssFeatureKind::Value,
        "content-box|padding-box|border-box|margin-box|fill-box|stroke-box|view-box",
        O_BOX3,
        "#keywords",
    ),
    CssFeatureMetadata::complete(
        "official.value.background-layer",
        CssFeatureKind::Value,
        "<bg-layer>",
        O_BACKGROUNDS3,
        "#layering",
    ),
    CssFeatureMetadata::complete(
        "official.value.background-image",
        CssFeatureKind::Value,
        "<bg-image>",
        O_BACKGROUNDS3,
        "#background-image",
    ),
    CssFeatureMetadata::complete(
        "official.value.repeat-style",
        CssFeatureKind::Value,
        "<repeat-style>",
        O_BACKGROUNDS3,
        "#background-repeat",
    ),
    CssFeatureMetadata::complete(
        "official.value.background-attachment",
        CssFeatureKind::Value,
        "<attachment>",
        O_BACKGROUNDS3,
        "#background-attachment",
    ),
    CssFeatureMetadata::complete(
        "official.value.background-position",
        CssFeatureKind::Value,
        "<bg-position>#",
        O_BACKGROUNDS3,
        "#background-position",
    ),
    CssFeatureMetadata::complete(
        "official.value.background-size",
        CssFeatureKind::Value,
        "<bg-size>",
        O_BACKGROUNDS3,
        "#background-size",
    ),
    CssFeatureMetadata::complete(
        "official.value.line-style",
        CssFeatureKind::Value,
        "<line-style>",
        O_BACKGROUNDS3,
        "#border-style",
    ),
    CssFeatureMetadata::complete(
        "official.value.line-width",
        CssFeatureKind::Value,
        "<line-width>",
        O_BACKGROUNDS3,
        "#border-width",
    ),
    CssFeatureMetadata::complete(
        "official.value.image",
        CssFeatureKind::Value,
        "<image>",
        O_IMAGES3,
        "#image-values",
    ),
    CssFeatureMetadata::complete(
        "official.value.gradient",
        CssFeatureKind::Value,
        "<gradient>",
        O_IMAGES3,
        "#gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.linear-gradient",
        CssFeatureKind::Value,
        "linear-gradient()",
        O_IMAGES3,
        "#linear-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.radial-gradient",
        CssFeatureKind::Value,
        "radial-gradient()",
        O_IMAGES3,
        "#radial-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.repeating-linear-gradient",
        CssFeatureKind::Value,
        "repeating-linear-gradient()",
        O_IMAGES3,
        "#repeating-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.repeating-radial-gradient",
        CssFeatureKind::Value,
        "repeating-radial-gradient()",
        O_IMAGES3,
        "#repeating-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.color-stop-list",
        CssFeatureKind::Value,
        "<color-stop-list>",
        O_IMAGES3,
        "#color-stop-syntax",
    ),
    CssFeatureMetadata::complete(
        "official.value.side-or-corner",
        CssFeatureKind::Value,
        "<side-or-corner>",
        O_IMAGES3,
        "#linear-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.radial-shape",
        CssFeatureKind::Value,
        "<radial-shape>",
        O_IMAGES3,
        "#radial-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.radial-size",
        CssFeatureKind::Value,
        "<radial-size>",
        O_IMAGES3,
        "#radial-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.radial-extent",
        CssFeatureKind::Value,
        "<radial-extent>",
        O_IMAGES3,
        "#radial-gradients",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform-list",
        CssFeatureKind::Value,
        "<transform-list>",
        O_TRANSFORMS1,
        "#transform-function-lists",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform-function",
        CssFeatureKind::Value,
        "<transform-function>",
        O_TRANSFORMS1,
        "#transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.matrix",
        CssFeatureKind::Value,
        "matrix()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.translate",
        CssFeatureKind::Value,
        "translate()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.translate-x",
        CssFeatureKind::Value,
        "translateX()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.translate-y",
        CssFeatureKind::Value,
        "translateY()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.scale",
        CssFeatureKind::Value,
        "scale()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.scale-x",
        CssFeatureKind::Value,
        "scaleX()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.scale-y",
        CssFeatureKind::Value,
        "scaleY()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.rotate",
        CssFeatureKind::Value,
        "rotate()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.skew",
        CssFeatureKind::Value,
        "skew()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.skew-x",
        CssFeatureKind::Value,
        "skewX()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.transform.skew-y",
        CssFeatureKind::Value,
        "skewY()",
        O_TRANSFORMS1,
        "#two-d-transform-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.blend-mode",
        CssFeatureKind::Value,
        "normal|multiply|screen|overlay|darken|lighten|color-dodge|color-burn|hard-light|soft-light|difference|exclusion|hue|saturation|color|luminosity",
        O_COMPOSITING1,
        "#blending,#blendingseparable,#blendingnonseparable",
    ),
    CssFeatureMetadata::complete(
        "official.value.easing-function",
        CssFeatureKind::Value,
        "<easing-function>",
        O_EASING1,
        "#easing-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.cubic-bezier-easing",
        CssFeatureKind::Value,
        "cubic-bezier()",
        O_EASING1,
        "#cubic-bezier-easing-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.step-easing",
        CssFeatureKind::Value,
        "steps()",
        O_EASING1,
        "#step-easing-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.step-position",
        CssFeatureKind::Value,
        "<step-position>",
        O_EASING1,
        "#step-easing-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.shadow",
        CssFeatureKind::Value,
        "<shadow>",
        O_BACKGROUNDS3,
        "#box-shadow",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.matrix3d",
        CssFeatureKind::Value,
        "matrix3d()",
        I_TRANSFORMS2,
        "#funcdef-matrix3d",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.perspective",
        CssFeatureKind::Value,
        "perspective()",
        I_TRANSFORMS2,
        "#funcdef-perspective",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.rotate3d",
        CssFeatureKind::Value,
        "rotate3d()",
        I_TRANSFORMS2,
        "#funcdef-rotate3d",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.rotate-x",
        CssFeatureKind::Value,
        "rotateX()",
        I_TRANSFORMS2,
        "#funcdef-rotatex",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.rotate-y",
        CssFeatureKind::Value,
        "rotateY()",
        I_TRANSFORMS2,
        "#funcdef-rotatey",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.rotate-z",
        CssFeatureKind::Value,
        "rotateZ()",
        I_TRANSFORMS2,
        "#funcdef-rotatez",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.scale3d",
        CssFeatureKind::Value,
        "scale3d()",
        I_TRANSFORMS2,
        "#funcdef-scale3d",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.scale-z",
        CssFeatureKind::Value,
        "scaleZ()",
        I_TRANSFORMS2,
        "#funcdef-scalez",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.translate3d",
        CssFeatureKind::Value,
        "translate3d()",
        I_TRANSFORMS2,
        "#funcdef-translate3d",
    ),
    CssFeatureMetadata::complete(
        "ext.value.transform.translate-z",
        CssFeatureKind::Value,
        "translateZ()",
        I_TRANSFORMS2,
        "#funcdef-translatez",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter-function-list",
        CssFeatureKind::Value,
        "<filter-function-list>",
        I_FILTER1,
        "#FilterProperty",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.blur",
        CssFeatureKind::Value,
        "blur()",
        I_FILTER1,
        "#funcdef-filter-blur",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.brightness",
        CssFeatureKind::Value,
        "brightness()",
        I_FILTER1,
        "#funcdef-filter-brightness",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.contrast",
        CssFeatureKind::Value,
        "contrast()",
        I_FILTER1,
        "#funcdef-filter-contrast",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.grayscale",
        CssFeatureKind::Value,
        "grayscale()",
        I_FILTER1,
        "#funcdef-filter-grayscale",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.hue-rotate",
        CssFeatureKind::Value,
        "hue-rotate()",
        I_FILTER1,
        "#funcdef-filter-hue-rotate",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.invert",
        CssFeatureKind::Value,
        "invert()",
        I_FILTER1,
        "#funcdef-filter-invert",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.opacity",
        CssFeatureKind::Value,
        "opacity()",
        I_FILTER1,
        "#funcdef-filter-opacity",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.saturate",
        CssFeatureKind::Value,
        "saturate()",
        I_FILTER1,
        "#funcdef-filter-saturate",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.sepia",
        CssFeatureKind::Value,
        "sepia()",
        I_FILTER1,
        "#funcdef-filter-sepia",
    ),
    CssFeatureMetadata::complete(
        "ext.value.filter.drop-shadow",
        CssFeatureKind::Value,
        "drop-shadow()",
        I_FILTER1,
        "#funcdef-filter-drop-shadow",
    ),
    CssFeatureMetadata::partial(
        "ext.value.basic-shape",
        CssFeatureKind::Value,
        "<basic-shape>",
        S_SHAPES1,
        "#typedef-basic-shape",
        BASIC_SHAPE_SUBSET,
        BASIC_SHAPE_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.inset",
        CssFeatureKind::Value,
        "inset()",
        S_SHAPES1,
        "#funcdef-basic-shape-inset",
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.circle",
        CssFeatureKind::Value,
        "circle()",
        S_SHAPES1,
        "#funcdef-basic-shape-circle",
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.ellipse",
        CssFeatureKind::Value,
        "ellipse()",
        S_SHAPES1,
        "#funcdef-basic-shape-ellipse",
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.polygon",
        CssFeatureKind::Value,
        "polygon()",
        S_SHAPES1,
        "#funcdef-basic-shape-polygon",
    ),
    CssFeatureMetadata::complete(
        "official.value.calc",
        CssFeatureKind::Value,
        "calc()",
        O_VALUES3,
        "#calc-notation,#calc-syntax,#calc-type-checking",
    ),
    CssFeatureMetadata::complete(
        "later.rule.namespace",
        CssFeatureKind::Rule,
        "@namespace",
        O_NAMESPACES3,
        "#declaration,#syntax",
    ),
    CssFeatureMetadata::complete(
        "later.rule.supports",
        CssFeatureKind::Rule,
        "@supports",
        O_CONDITIONAL3,
        "#at-supports",
    ),
    CssFeatureMetadata::complete(
        "later.rule.counter-style",
        CssFeatureKind::Rule,
        "@counter-style",
        O_COUNTERSTYLES3,
        "#the-counter-style-rule",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.system",
        CssFeatureKind::Descriptor,
        "system in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-system",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.negative",
        CssFeatureKind::Descriptor,
        "negative in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-negative",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.prefix",
        CssFeatureKind::Descriptor,
        "prefix in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-prefix",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.suffix",
        CssFeatureKind::Descriptor,
        "suffix in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-suffix",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.range",
        CssFeatureKind::Descriptor,
        "range in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-range",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.pad",
        CssFeatureKind::Descriptor,
        "pad in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-pad",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.fallback",
        CssFeatureKind::Descriptor,
        "fallback in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-fallback",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.symbols",
        CssFeatureKind::Descriptor,
        "symbols in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-symbols",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.additive-symbols",
        CssFeatureKind::Descriptor,
        "additive-symbols in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-symbols",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.counter-style.speak-as",
        CssFeatureKind::Descriptor,
        "speak-as in @counter-style",
        O_COUNTERSTYLES3,
        "#counter-style-speak-as",
    ),
    CssFeatureMetadata::complete(
        "official.value.counter-style",
        CssFeatureKind::Value,
        "<counter-style>",
        O_COUNTERSTYLES3,
        "#the-counter-style-rule",
    ),
    CssFeatureMetadata::complete(
        "official.value.counter-style-name",
        CssFeatureKind::Value,
        "<counter-style-name>",
        O_COUNTERSTYLES3,
        "#the-counter-style-rule",
    ),
    CssFeatureMetadata::complete(
        "official.value.symbol",
        CssFeatureKind::Value,
        "<symbol>",
        O_COUNTERSTYLES3,
        "#counter-style-symbols",
    ),
    CssFeatureMetadata::complete(
        "official.value.symbols-function",
        CssFeatureKind::Value,
        "symbols()",
        O_COUNTERSTYLES3,
        "#symbols-function",
    ),
    CssFeatureMetadata::complete(
        "official.value.symbols-type",
        CssFeatureKind::Value,
        "cyclic|numeric|alphabetic|symbolic|fixed",
        O_COUNTERSTYLES3,
        "#symbols-function",
    ),
    CssFeatureMetadata::complete(
        "later.rule.page",
        CssFeatureKind::Rule,
        "@page",
        O_CSS2,
        "page.html#page-box",
    ),
    CssFeatureMetadata::complete(
        "official.selector.page-pseudo",
        CssFeatureKind::Selector,
        ":left|:right|:first",
        O_CSS2,
        "page.html#page-selectors",
    ),
    CssFeatureMetadata::recognized_unsupported(
        "later.rule.font-feature-values",
        CssFeatureKind::Rule,
        "@font-feature-values",
        I_FONTS4,
        "#font-feature-values-rule",
        CssErrorCode::UnsupportedAtRule,
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-family",
        CssFeatureKind::Descriptor,
        "font-family in @font-face",
        O_FONTS3,
        "#font-family-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.src",
        CssFeatureKind::Descriptor,
        "src in @font-face",
        O_FONTS3,
        "#src-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-weight",
        CssFeatureKind::Descriptor,
        "font-weight in @font-face",
        O_FONTS3,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-style",
        CssFeatureKind::Descriptor,
        "font-style in @font-face",
        O_FONTS3,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-stretch",
        CssFeatureKind::Descriptor,
        "font-stretch in @font-face",
        O_FONTS3,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-display",
        CssFeatureKind::Descriptor,
        "font-display in @font-face",
        I_FONTS4,
        "#font-display-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.unicode-range",
        CssFeatureKind::Descriptor,
        "unicode-range in @font-face",
        O_FONTS3,
        "#unicode-range-desc",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.font-feature-settings",
        CssFeatureKind::Descriptor,
        "font-feature-settings in @font-face",
        O_FONTS3,
        "#font-rend-desc",
    ),
    CssFeatureMetadata::complete(
        "official.value.font-source",
        CssFeatureKind::Value,
        "@font-face source list",
        O_FONTS3,
        "#src-desc",
    ),
    CssFeatureMetadata::complete(
        "official.value.opentype-tag",
        CssFeatureKind::Value,
        "OpenType feature tag",
        O_FONTS3,
        "#font-rend-desc",
    ),
    CssFeatureMetadata::partial(
        "ext.descriptor.font-weight-range",
        CssFeatureKind::Descriptor,
        "font-weight ranges in @font-face",
        I_FONTS4,
        "#font-weight-desc",
        FONT_FACE_WEIGHT_RANGE_SUBSET,
        FONT_FACE_WEIGHT_RANGE_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "ext.descriptor.font-style-oblique-range",
        CssFeatureKind::Descriptor,
        "font-style oblique ranges in @font-face",
        I_FONTS4,
        "#font-style-desc",
        FONT_FACE_STYLE_RANGE_SUBSET,
        FONT_FACE_STYLE_RANGE_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "ext.descriptor.font-stretch-range",
        CssFeatureKind::Descriptor,
        "font-stretch percentage ranges in @font-face",
        I_FONTS4,
        "#font-stretch-desc",
        FONT_FACE_STRETCH_RANGE_SUBSET,
        FONT_FACE_STRETCH_RANGE_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "ext.value.font-source-modern-hints",
        CssFeatureKind::Value,
        "format() keyword and tech() font-source hints",
        I_FONTS4,
        "#font-face-src-parsing",
        FONT_SOURCE_HINTS_SUBSET,
        FONT_SOURCE_HINTS_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.complex",
        CssFeatureKind::Selector,
        "type, universal, ID, class; presence and six valued attribute matchers; descendant, child, next-sibling, subsequent-sibling combinators",
        O_SELECTORS3,
        "#type-selectors,#universal-selector,#attribute-representation,#attribute-substrings,#class-html,#id-selectors,#descendant-combinators,#child-combinators,#adjacent-sibling-combinators,#general-sibling-combinators",
        "The exact baseline-recognized complex-selector spelling group is supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.pseudo-class",
        CssFeatureKind::Selector,
        ":root, :hover, :active, :focus, :disabled, :enabled, :checked, :first-child, :last-child, :only-child, :empty, :first-of-type, :last-of-type, :only-of-type",
        O_SELECTORS3,
        "#dynamic-pseudos,#UIstates,#structural-pseudos",
        "The exact baseline-recognized pseudo-class spelling group is supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.functional",
        CssFeatureKind::Selector,
        ":nth-child(), :nth-last-child(), :nth-of-type(), :nth-last-of-type(), :not()",
        O_SELECTORS3,
        "#structural-pseudos,#negation",
        "The exact baseline-recognized functional pseudo-class spelling group is supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.extension-state",
        CssFeatureKind::Selector,
        ":scope, :focus-visible, :focus-within, :required, :optional, :valid, :invalid, :placeholder-shown, :default, :indeterminate, :read-only, :read-write, :in-range, :out-of-range, :modal, :fullscreen, :popover-open",
        I_SELECTORS4,
        "#useraction-pseudos,#input-pseudos,#resource-pseudos,#display-state-pseudos",
        "The exact I01 extension-state pseudo-class spelling group is supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.extension-functional",
        CssFeatureKind::Selector,
        ":is(), :where(), complex :not(), :has(), and nth-child of lists",
        I_SELECTORS4,
        "#matches,#zero-matches,#relational,#negation,#the-nth-child-pseudo",
        "The exact I01 extension-functional pseudo-class spelling group is supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.attribute-case",
        CssFeatureKind::Selector,
        "i and s attribute-selector modifiers",
        I_SELECTORS4,
        "#attribute-case",
        "The i and s attribute-selector case modifiers are supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::baseline_alias(
        "baseline.selector.pseudo-element",
        CssFeatureKind::Selector,
        "::before, ::after, ::marker, ::selection, ::backdrop, and generated ::marker sequences",
        BASELINE_SELECTORS,
        "pseudo-element selector",
        (
            "The exact baseline-recognized pseudo-element spelling group is supported.",
            SELECTOR_REMAINDER,
        ),
        PSEUDO_ELEMENT_ALIAS_TARGETS,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.nesting",
        CssFeatureKind::Selector,
        "nesting &, scoped selector anchors, and scoped relative selectors",
        I_NESTING1,
        "#nest-selector",
        "Nesting &, scoped selector anchors, and scoped relative selectors are supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::baseline_alias(
        "baseline.media.query-list",
        CssFeatureKind::MediaQuery,
        "typed/condition query lists, not/only, and/or/not, range and colon forms, and malformed-member Never recovery",
        BASELINE_QUERIES,
        "media query list",
        (
            "The exact baseline-recognized media query-list spelling group and malformed-member Never recovery are supported.",
            QUERY_REMAINDER,
        ),
        MEDIA_QUERY_LIST_ALIAS_TARGETS,
    ),
    CssFeatureMetadata::complete(
        "baseline.media.type",
        CssFeatureKind::MediaQuery,
        "all, aural, braille, embossed, handheld, print, projection, screen, speech, tty, tv",
        O_MEDIA3,
        "#media1",
    ),
    CssFeatureMetadata::baseline_alias(
        "baseline.media.range-feature",
        CssFeatureKind::MediaQuery,
        "width, height, resolution, color, monochrome and their min-/max- names",
        BASELINE_QUERIES,
        "media range feature",
        (
            "The exact baseline-recognized media range-feature spelling group is supported.",
            QUERY_REMAINDER,
        ),
        MEDIA_RANGE_ALIAS_TARGETS,
    ),
    CssFeatureMetadata::baseline_alias(
        "baseline.media.discrete-feature",
        CssFeatureKind::MediaQuery,
        "orientation, prefers-color-scheme, prefers-reduced-motion, prefers-reduced-transparency, prefers-contrast, forced-colors, hover, any-hover, pointer, any-pointer, display-mode",
        BASELINE_QUERIES,
        "media discrete feature",
        (
            "The exact baseline-recognized media discrete-feature spelling group is supported.",
            QUERY_REMAINDER,
        ),
        MEDIA_DISCRETE_ALIAS_TARGETS,
    ),
    CssFeatureMetadata::partial(
        "baseline.container.condition",
        CssFeatureKind::ContainerQuery,
        "and/or/not, size features, and custom-property style existence/equality",
        X_CONTAIN3,
        "#container-rule",
        "The exact baseline-recognized container-condition spelling group is supported.",
        QUERY_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.container.size-feature",
        CssFeatureKind::ContainerQuery,
        "width, height, inline-size, block-size, aspect-ratio, orientation and applicable min-/max- names",
        X_CONTAIN3,
        "#size-container",
        "The exact baseline-recognized container size-feature spelling group is supported.",
        QUERY_REMAINDER,
    ),
    property_feature!(CssKnownProperty::All, "all", "baseline.property.all"),
    property_feature!(
        CssKnownProperty::Display,
        "display",
        "baseline.property.display"
    ),
    property_feature!(
        CssKnownProperty::BoxSizing,
        "box-sizing",
        "baseline.property.box-sizing"
    ),
    property_feature!(
        CssKnownProperty::Position,
        "position",
        "baseline.property.position"
    ),
    property_feature!(
        CssKnownProperty::Direction,
        "direction",
        "baseline.property.direction"
    ),
    property_feature!(
        CssKnownProperty::Overflow,
        "overflow",
        "baseline.property.overflow"
    ),
    property_feature!(
        CssKnownProperty::OverflowX,
        "overflow-x",
        "baseline.property.overflow-x"
    ),
    property_feature!(
        CssKnownProperty::OverflowY,
        "overflow-y",
        "baseline.property.overflow-y"
    ),
    property_feature!(
        CssKnownProperty::FlexDirection,
        "flex-direction",
        "baseline.property.flex-direction"
    ),
    complete_property_feature!(
        CssKnownProperty::FlexFlow,
        "flex-flow",
        "official.property.flex-flow"
    ),
    property_feature!(
        CssKnownProperty::FlexWrap,
        "flex-wrap",
        "baseline.property.flex-wrap"
    ),
    property_feature!(CssKnownProperty::Float, "float", "baseline.property.float"),
    property_feature!(CssKnownProperty::Clear, "clear", "baseline.property.clear"),
    property_feature!(
        CssKnownProperty::AlignContent,
        "align-content",
        "baseline.property.align-content"
    ),
    property_feature!(
        CssKnownProperty::JustifyContent,
        "justify-content",
        "baseline.property.justify-content"
    ),
    property_feature!(
        CssKnownProperty::AlignItems,
        "align-items",
        "baseline.property.align-items"
    ),
    property_feature!(
        CssKnownProperty::AlignSelf,
        "align-self",
        "baseline.property.align-self"
    ),
    property_feature!(
        CssKnownProperty::JustifyItems,
        "justify-items",
        "baseline.property.justify-items"
    ),
    property_feature!(
        CssKnownProperty::JustifySelf,
        "justify-self",
        "baseline.property.justify-self"
    ),
    property_feature!(
        CssKnownProperty::PlaceContent,
        "place-content",
        "baseline.property.place-content"
    ),
    property_feature!(
        CssKnownProperty::PlaceItems,
        "place-items",
        "baseline.property.place-items"
    ),
    property_feature!(
        CssKnownProperty::PlaceSelf,
        "place-self",
        "baseline.property.place-self"
    ),
    property_feature!(
        CssKnownProperty::Visibility,
        "visibility",
        "baseline.property.visibility"
    ),
    property_feature!(
        CssKnownProperty::Content,
        "content",
        "baseline.property.content"
    ),
    property_feature!(
        CssKnownProperty::ContentVisibility,
        "content-visibility",
        "baseline.property.content-visibility"
    ),
    property_feature!(
        CssKnownProperty::ListStyleType,
        "list-style-type",
        "baseline.property.list-style-type"
    ),
    property_feature!(
        CssKnownProperty::ListStylePosition,
        "list-style-position",
        "baseline.property.list-style-position"
    ),
    property_feature!(
        CssKnownProperty::ListStyleImage,
        "list-style-image",
        "baseline.property.list-style-image"
    ),
    property_feature!(
        CssKnownProperty::ListStyle,
        "list-style",
        "baseline.property.list-style"
    ),
    property_feature!(
        CssKnownProperty::CounterReset,
        "counter-reset",
        "baseline.property.counter-reset"
    ),
    property_feature!(
        CssKnownProperty::CounterIncrement,
        "counter-increment",
        "baseline.property.counter-increment"
    ),
    property_feature!(
        CssKnownProperty::CounterSet,
        "counter-set",
        "baseline.property.counter-set"
    ),
    property_feature!(CssKnownProperty::Width, "width", "baseline.property.width"),
    property_feature!(
        CssKnownProperty::Height,
        "height",
        "baseline.property.height"
    ),
    property_feature!(
        CssKnownProperty::MinWidth,
        "min-width",
        "baseline.property.min-width"
    ),
    property_feature!(
        CssKnownProperty::MinHeight,
        "min-height",
        "baseline.property.min-height"
    ),
    property_feature!(
        CssKnownProperty::MaxWidth,
        "max-width",
        "baseline.property.max-width"
    ),
    property_feature!(
        CssKnownProperty::MaxHeight,
        "max-height",
        "baseline.property.max-height"
    ),
    property_feature!(
        CssKnownProperty::FlexBasis,
        "flex-basis",
        "baseline.property.flex-basis"
    ),
    property_feature!(CssKnownProperty::Gap, "gap", "baseline.property.gap"),
    property_feature!(
        CssKnownProperty::RowGap,
        "row-gap",
        "baseline.property.row-gap"
    ),
    property_feature!(
        CssKnownProperty::ColumnGap,
        "column-gap",
        "baseline.property.column-gap"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnCount,
        "column-count",
        "official.property.column-count"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnFill,
        "column-fill",
        "official.property.column-fill"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnRule,
        "column-rule",
        "official.property.column-rule"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnRuleColor,
        "column-rule-color",
        "official.property.column-rule-color"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnRuleStyle,
        "column-rule-style",
        "official.property.column-rule-style"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnRuleWidth,
        "column-rule-width",
        "official.property.column-rule-width"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnSpan,
        "column-span",
        "official.property.column-span"
    ),
    complete_property_feature!(
        CssKnownProperty::ColumnWidth,
        "column-width",
        "official.property.column-width"
    ),
    complete_property_feature!(
        CssKnownProperty::Columns,
        "columns",
        "official.property.columns"
    ),
    property_feature!(
        CssKnownProperty::GridFlowTolerance,
        "grid-flow-tolerance",
        "baseline.property.grid-flow-tolerance"
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.grid-template-rows",
        CssKnownProperty::GridTemplateRows,
        "grid-template-rows",
        "#propdef-grid-template-rows",
        &[],
        GRID_PROPERTY_SUBSET,
        GRID_PROPERTY_REMAINDER,
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.grid-template-columns",
        CssKnownProperty::GridTemplateColumns,
        "grid-template-columns",
        "#propdef-grid-template-columns",
        &[],
        GRID_PROPERTY_SUBSET,
        GRID_PROPERTY_REMAINDER,
    ),
    property_feature!(
        CssKnownProperty::GridTemplateAreas,
        "grid-template-areas",
        "baseline.property.grid-template-areas"
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.grid-template",
        CssKnownProperty::GridTemplate,
        "grid-template",
        "#propdef-grid-template",
        &[],
        GRID_PROPERTY_SUBSET,
        GRID_PROPERTY_REMAINDER,
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.grid-auto-rows",
        CssKnownProperty::GridAutoRows,
        "grid-auto-rows",
        "#propdef-grid-auto-rows",
        &[],
        GRID_PROPERTY_SUBSET,
        GRID_PROPERTY_REMAINDER,
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.grid-auto-columns",
        CssKnownProperty::GridAutoColumns,
        "grid-auto-columns",
        "#propdef-grid-auto-columns",
        &[],
        GRID_PROPERTY_SUBSET,
        GRID_PROPERTY_REMAINDER,
    ),
    property_feature!(
        CssKnownProperty::GridAutoFlow,
        "grid-auto-flow",
        "baseline.property.grid-auto-flow"
    ),
    property_feature!(
        CssKnownProperty::GridRowStart,
        "grid-row-start",
        "baseline.property.grid-row-start"
    ),
    property_feature!(
        CssKnownProperty::GridRowEnd,
        "grid-row-end",
        "baseline.property.grid-row-end"
    ),
    property_feature!(
        CssKnownProperty::GridColumnStart,
        "grid-column-start",
        "baseline.property.grid-column-start"
    ),
    property_feature!(
        CssKnownProperty::GridColumnEnd,
        "grid-column-end",
        "baseline.property.grid-column-end"
    ),
    property_feature!(
        CssKnownProperty::GridRow,
        "grid-row",
        "baseline.property.grid-row"
    ),
    property_feature!(
        CssKnownProperty::GridColumn,
        "grid-column",
        "baseline.property.grid-column"
    ),
    property_feature!(
        CssKnownProperty::GridArea,
        "grid-area",
        "baseline.property.grid-area"
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.grid",
        CssKnownProperty::Grid,
        "grid",
        "#propdef-grid",
        &[],
        GRID_PROPERTY_SUBSET,
        GRID_PROPERTY_REMAINDER,
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-size",
        CssKnownProperty::FontSize,
        "font-size",
        "#propdef-font-size",
        &[],
    ),
    property_feature!(
        CssKnownProperty::LineHeight,
        "line-height",
        "baseline.property.line-height"
    ),
    property_feature!(
        CssKnownProperty::WritingMode,
        "writing-mode",
        "baseline.property.writing-mode"
    ),
    property_feature!(
        CssKnownProperty::TextAlign,
        "text-align",
        "baseline.property.text-align"
    ),
    property_feature!(
        CssKnownProperty::TextAlignLast,
        "text-align-last",
        "baseline.property.text-align-last"
    ),
    property_feature!(
        CssKnownProperty::TextIndent,
        "text-indent",
        "baseline.property.text-indent"
    ),
    property_feature!(
        CssKnownProperty::VerticalAlign,
        "vertical-align",
        "baseline.property.vertical-align"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-family",
        CssKnownProperty::FontFamily,
        "font-family",
        "#propdef-font-family",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font",
        CssKnownProperty::Font,
        "font",
        "#propdef-font",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-weight",
        CssKnownProperty::FontWeight,
        "font-weight",
        "#propdef-font-weight",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-style",
        CssKnownProperty::FontStyle,
        "font-style",
        "#propdef-font-style",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-stretch",
        CssKnownProperty::FontStretch,
        "font-stretch",
        "#propdef-font-stretch",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-variant",
        CssKnownProperty::FontVariant,
        "font-variant",
        "#propdef-font-variant",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-variant-caps",
        CssKnownProperty::FontVariantCaps,
        "font-variant-caps",
        "#propdef-font-variant-caps",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-variant-east-asian",
        CssKnownProperty::FontVariantEastAsian,
        "font-variant-east-asian",
        "#propdef-font-variant-east-asian",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-variant-ligatures",
        CssKnownProperty::FontVariantLigatures,
        "font-variant-ligatures",
        "#propdef-font-variant-ligatures",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-variant-numeric",
        CssKnownProperty::FontVariantNumeric,
        "font-variant-numeric",
        "#propdef-font-variant-numeric",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-variant-position",
        CssKnownProperty::FontVariantPosition,
        "font-variant-position",
        "#propdef-font-variant-position",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.font-feature-settings",
        CssKnownProperty::FontFeatureSettings,
        "font-feature-settings",
        "#propdef-font-feature-settings",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-kerning",
        CssKnownProperty::FontKerning,
        "font-kerning",
        "#propdef-font-kerning",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-size-adjust",
        CssKnownProperty::FontSizeAdjust,
        "font-size-adjust",
        "#propdef-font-size-adjust",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.font-synthesis",
        CssKnownProperty::FontSynthesis,
        "font-synthesis",
        "#propdef-font-synthesis",
        &[],
    ),
    CssFeatureMetadata::partial(
        "ext.property.font-weight-range",
        CssFeatureKind::Property,
        "font-weight numeric range",
        I_FONTS4,
        "#font-weight-prop",
        FONT_WEIGHT_RANGE_SUBSET,
        FONT_WEIGHT_RANGE_REMAINDER,
    ),
    property_feature!(
        CssKnownProperty::LetterSpacing,
        "letter-spacing",
        "baseline.property.letter-spacing"
    ),
    property_feature!(
        CssKnownProperty::TextWrap,
        "text-wrap",
        "baseline.property.text-wrap"
    ),
    property_feature!(
        CssKnownProperty::WhiteSpace,
        "white-space",
        "baseline.property.white-space"
    ),
    property_feature!(
        CssKnownProperty::WordBreak,
        "word-break",
        "baseline.property.word-break"
    ),
    property_feature!(
        CssKnownProperty::OverflowWrap,
        "overflow-wrap",
        "baseline.property.overflow-wrap"
    ),
    property_feature!(
        CssKnownProperty::TextOverflow,
        "text-overflow",
        "baseline.property.text-overflow"
    ),
    property_feature!(
        CssKnownProperty::TextDecoration,
        "text-decoration",
        "baseline.property.text-decoration"
    ),
    property_feature!(
        CssKnownProperty::TextDecorationLine,
        "text-decoration-line",
        "baseline.property.text-decoration-line"
    ),
    property_feature!(
        CssKnownProperty::TextDecorationColor,
        "text-decoration-color",
        "baseline.property.text-decoration-color"
    ),
    property_feature!(
        CssKnownProperty::TextDecorationStyle,
        "text-decoration-style",
        "baseline.property.text-decoration-style"
    ),
    property_feature!(
        CssKnownProperty::TextDecorationThickness,
        "text-decoration-thickness",
        "baseline.property.text-decoration-thickness"
    ),
    property_feature!(
        CssKnownProperty::TextTransform,
        "text-transform",
        "baseline.property.text-transform"
    ),
    property_feature!(CssKnownProperty::Inset, "inset", "baseline.property.inset"),
    property_feature!(CssKnownProperty::Top, "top", "baseline.property.top"),
    property_feature!(CssKnownProperty::Right, "right", "baseline.property.right"),
    property_feature!(
        CssKnownProperty::Bottom,
        "bottom",
        "baseline.property.bottom"
    ),
    property_feature!(CssKnownProperty::Left, "left", "baseline.property.left"),
    property_feature!(
        CssKnownProperty::ZIndex,
        "z-index",
        "baseline.property.z-index"
    ),
    property_feature!(
        CssKnownProperty::BoxDecorationBreak,
        "box-decoration-break",
        "baseline.property.box-decoration-break"
    ),
    property_feature!(
        CssKnownProperty::Margin,
        "margin",
        "baseline.property.margin"
    ),
    property_feature!(
        CssKnownProperty::MarginTop,
        "margin-top",
        "baseline.property.margin-top"
    ),
    property_feature!(
        CssKnownProperty::MarginRight,
        "margin-right",
        "baseline.property.margin-right"
    ),
    property_feature!(
        CssKnownProperty::MarginBottom,
        "margin-bottom",
        "baseline.property.margin-bottom"
    ),
    property_feature!(
        CssKnownProperty::MarginLeft,
        "margin-left",
        "baseline.property.margin-left"
    ),
    property_feature!(
        CssKnownProperty::Padding,
        "padding",
        "baseline.property.padding"
    ),
    property_feature!(
        CssKnownProperty::PaddingTop,
        "padding-top",
        "baseline.property.padding-top"
    ),
    property_feature!(
        CssKnownProperty::PaddingRight,
        "padding-right",
        "baseline.property.padding-right"
    ),
    property_feature!(
        CssKnownProperty::PaddingBottom,
        "padding-bottom",
        "baseline.property.padding-bottom"
    ),
    property_feature!(
        CssKnownProperty::PaddingLeft,
        "padding-left",
        "baseline.property.padding-left"
    ),
    complete_property_feature!(
        CssKnownProperty::Border,
        "border",
        "baseline.property.border"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderTop,
        "border-top",
        "baseline.property.border-top"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderRight,
        "border-right",
        "baseline.property.border-right"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderBottom,
        "border-bottom",
        "baseline.property.border-bottom"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderLeft,
        "border-left",
        "baseline.property.border-left"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderWidth,
        "border-width",
        "baseline.property.border-width"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderTopWidth,
        "border-top-width",
        "baseline.property.border-top-width"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderRightWidth,
        "border-right-width",
        "baseline.property.border-right-width"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderBottomWidth,
        "border-bottom-width",
        "baseline.property.border-bottom-width"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderLeftWidth,
        "border-left-width",
        "baseline.property.border-left-width"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderImage,
        "border-image",
        "official.property.border-image"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderImageOutset,
        "border-image-outset",
        "official.property.border-image-outset"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderImageRepeat,
        "border-image-repeat",
        "official.property.border-image-repeat"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderImageSlice,
        "border-image-slice",
        "official.property.border-image-slice"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderImageSource,
        "border-image-source",
        "official.property.border-image-source"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderImageWidth,
        "border-image-width",
        "official.property.border-image-width"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.color",
        CssKnownProperty::Color,
        "color",
        "#propdef-color",
        &[],
    ),
    complete_property_feature!(
        CssKnownProperty::Background,
        "background",
        "baseline.property.background"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundColor,
        "background-color",
        "baseline.property.background-color"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderColor,
        "border-color",
        "baseline.property.border-color"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderTopColor,
        "border-top-color",
        "baseline.property.border-top-color"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderRightColor,
        "border-right-color",
        "baseline.property.border-right-color"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderBottomColor,
        "border-bottom-color",
        "baseline.property.border-bottom-color"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderLeftColor,
        "border-left-color",
        "baseline.property.border-left-color"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundImage,
        "background-image",
        "baseline.property.background-image"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.background-position",
        CssKnownProperty::BackgroundPosition,
        "background-position",
        "#propdef-background-position",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.object-position",
        CssKnownProperty::ObjectPosition,
        "object-position",
        "#propdef-object-position",
        &[],
    ),
    complete_property_feature!(
        CssKnownProperty::ImageOrientation,
        "image-orientation",
        "official.property.image-orientation"
    ),
    complete_property_feature!(
        CssKnownProperty::ImageRendering,
        "image-rendering",
        "official.property.image-rendering"
    ),
    complete_property_feature!(
        CssKnownProperty::ObjectFit,
        "object-fit",
        "official.property.object-fit"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundSize,
        "background-size",
        "baseline.property.background-size"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundRepeat,
        "background-repeat",
        "baseline.property.background-repeat"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundOrigin,
        "background-origin",
        "baseline.property.background-origin"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundClip,
        "background-clip",
        "baseline.property.background-clip"
    ),
    complete_property_feature!(
        CssKnownProperty::BackgroundAttachment,
        "background-attachment",
        "baseline.property.background-attachment"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderStyle,
        "border-style",
        "baseline.property.border-style"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderTopStyle,
        "border-top-style",
        "baseline.property.border-top-style"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderRightStyle,
        "border-right-style",
        "baseline.property.border-right-style"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderBottomStyle,
        "border-bottom-style",
        "baseline.property.border-bottom-style"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderLeftStyle,
        "border-left-style",
        "baseline.property.border-left-style"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderRadius,
        "border-radius",
        "baseline.property.border-radius"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderTopLeftRadius,
        "border-top-left-radius",
        "baseline.property.border-top-left-radius"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderTopRightRadius,
        "border-top-right-radius",
        "baseline.property.border-top-right-radius"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderBottomRightRadius,
        "border-bottom-right-radius",
        "baseline.property.border-bottom-right-radius"
    ),
    complete_property_feature!(
        CssKnownProperty::BorderBottomLeftRadius,
        "border-bottom-left-radius",
        "baseline.property.border-bottom-left-radius"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.box-shadow",
        CssKnownProperty::BoxShadow,
        "box-shadow",
        "#propdef-box-shadow",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.opacity",
        CssKnownProperty::Opacity,
        "opacity",
        "#propdef-opacity",
        &[],
    ),
    property_feature!(
        CssKnownProperty::FlexGrow,
        "flex-grow",
        "baseline.property.flex-grow"
    ),
    property_feature!(
        CssKnownProperty::FlexShrink,
        "flex-shrink",
        "baseline.property.flex-shrink"
    ),
    property_feature!(CssKnownProperty::Order, "order", "baseline.property.order"),
    property_feature!(CssKnownProperty::Flex, "flex", "baseline.property.flex"),
    property_feature!(
        CssKnownProperty::JustifyTracks,
        "justify-tracks",
        "baseline.property.justify-tracks"
    ),
    property_feature!(
        CssKnownProperty::AlignTracks,
        "align-tracks",
        "baseline.property.align-tracks"
    ),
    property_feature!(
        CssKnownProperty::AspectRatio,
        "aspect-ratio",
        "baseline.property.aspect-ratio"
    ),
    property_feature!(
        CssKnownProperty::ScrollbarWidth,
        "scrollbar-width",
        "baseline.property.scrollbar-width"
    ),
    property_feature!(
        CssKnownProperty::Cursor,
        "cursor",
        "baseline.property.cursor"
    ),
    property_feature!(
        CssKnownProperty::PointerEvents,
        "pointer-events",
        "baseline.property.pointer-events"
    ),
    property_feature!(
        CssKnownProperty::UserSelect,
        "user-select",
        "baseline.property.user-select"
    ),
    property_feature!(
        CssKnownProperty::Outline,
        "outline",
        "baseline.property.outline"
    ),
    property_feature!(
        CssKnownProperty::OutlineColor,
        "outline-color",
        "baseline.property.outline-color"
    ),
    property_feature!(
        CssKnownProperty::OutlineStyle,
        "outline-style",
        "baseline.property.outline-style"
    ),
    property_feature!(
        CssKnownProperty::OutlineWidth,
        "outline-width",
        "baseline.property.outline-width"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.transform",
        CssKnownProperty::Transform,
        "transform",
        "#propdef-transform",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.transform-origin",
        CssKnownProperty::TransformOrigin,
        "transform-origin",
        "#propdef-transform-origin",
        &[],
    ),
    property_feature!(
        CssKnownProperty::Translate,
        "translate",
        "baseline.property.translate"
    ),
    property_feature!(
        CssKnownProperty::Rotate,
        "rotate",
        "baseline.property.rotate"
    ),
    property_feature!(CssKnownProperty::Scale, "scale", "baseline.property.scale"),
    CssFeatureMetadata::complete_property(
        "baseline.property.filter",
        CssKnownProperty::Filter,
        "filter",
        "#propdef-filter",
        &[],
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.backdrop-filter",
        CssKnownProperty::BackdropFilter,
        "backdrop-filter",
        "#propdef-backdrop-filter",
        &[],
        BACKDROP_FILTER_SUBSET,
        BACKDROP_FILTER_REMAINDER,
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.clip-path",
        CssKnownProperty::ClipPath,
        "clip-path",
        "#propdef-clip-path",
        &[],
        CLIP_PATH_SUBSET,
        CLIP_PATH_REMAINDER,
    ),
    property_feature!(CssKnownProperty::Mask, "mask", "baseline.property.mask"),
    property_feature!(
        CssKnownProperty::MaskImage,
        "mask-image",
        "baseline.property.mask-image"
    ),
    property_feature!(
        CssKnownProperty::MaskSize,
        "mask-size",
        "baseline.property.mask-size"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.mask-position",
        CssKnownProperty::MaskPosition,
        "mask-position",
        "#propdef-mask-position",
        &[],
    ),
    property_feature!(
        CssKnownProperty::MaskRepeat,
        "mask-repeat",
        "baseline.property.mask-repeat"
    ),
    property_feature!(
        CssKnownProperty::TransitionProperty,
        "transition-property",
        "baseline.property.transition-property"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.transition-duration",
        CssKnownProperty::TransitionDuration,
        "transition-duration",
        "#propdef-transition-duration",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.transition-delay",
        CssKnownProperty::TransitionDelay,
        "transition-delay",
        "#propdef-transition-delay",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.transition-timing-function",
        CssKnownProperty::TransitionTimingFunction,
        "transition-timing-function",
        "#propdef-transition-timing-function",
        &[],
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.transition",
        CssKnownProperty::Transition,
        "transition",
        "#propdef-transition",
        &[],
        TIMING_SUBSET,
        TIMING_REMAINDER,
    ),
    property_feature!(
        CssKnownProperty::AnimationName,
        "animation-name",
        "baseline.property.animation-name"
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.animation-duration",
        CssKnownProperty::AnimationDuration,
        "animation-duration",
        "#propdef-animation-duration",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.animation-delay",
        CssKnownProperty::AnimationDelay,
        "animation-delay",
        "#propdef-animation-delay",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.animation-timing-function",
        CssKnownProperty::AnimationTimingFunction,
        "animation-timing-function",
        "#propdef-animation-timing-function",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "baseline.property.animation-iteration-count",
        CssKnownProperty::AnimationIterationCount,
        "animation-iteration-count",
        "#propdef-animation-iteration-count",
        &[],
    ),
    property_feature!(
        CssKnownProperty::AnimationDirection,
        "animation-direction",
        "baseline.property.animation-direction"
    ),
    property_feature!(
        CssKnownProperty::AnimationFillMode,
        "animation-fill-mode",
        "baseline.property.animation-fill-mode"
    ),
    property_feature!(
        CssKnownProperty::AnimationPlayState,
        "animation-play-state",
        "baseline.property.animation-play-state"
    ),
    CssFeatureMetadata::partial_property_with_boundary(
        "baseline.property.animation",
        CssKnownProperty::Animation,
        "animation",
        "#propdef-animation",
        &[],
        TIMING_SUBSET,
        TIMING_REMAINDER,
    ),
    CssFeatureMetadata::complete_property(
        "official.property.border-collapse",
        CssKnownProperty::BorderCollapse,
        "border-collapse",
        "tables.html#propdef-border-collapse",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.border-spacing",
        CssKnownProperty::BorderSpacing,
        "border-spacing",
        "tables.html#propdef-border-spacing",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.caption-side",
        CssKnownProperty::CaptionSide,
        "caption-side",
        "tables.html#propdef-caption-side",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.clip",
        CssKnownProperty::Clip,
        "clip",
        "visufx.html#propdef-clip",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.empty-cells",
        CssKnownProperty::EmptyCells,
        "empty-cells",
        "tables.html#propdef-empty-cells",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.orphans",
        CssKnownProperty::Orphans,
        "orphans",
        "page.html#propdef-orphans",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.page-break-after",
        CssKnownProperty::PageBreakAfter,
        "page-break-after",
        "page.html#propdef-page-break-after",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.page-break-before",
        CssKnownProperty::PageBreakBefore,
        "page-break-before",
        "page.html#propdef-page-break-before",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.page-break-inside",
        CssKnownProperty::PageBreakInside,
        "page-break-inside",
        "page.html#propdef-page-break-inside",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.quotes",
        CssKnownProperty::Quotes,
        "quotes",
        "generate.html#propdef-quotes",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.table-layout",
        CssKnownProperty::TableLayout,
        "table-layout",
        "tables.html#propdef-table-layout",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.widows",
        CssKnownProperty::Widows,
        "widows",
        "page.html#propdef-widows",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.word-spacing",
        CssKnownProperty::WordSpacing,
        "word-spacing",
        "text.html#propdef-word-spacing",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.text-combine-upright",
        CssKnownProperty::TextCombineUpright,
        "text-combine-upright",
        "#propdef-text-combine-upright",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.text-orientation",
        CssKnownProperty::TextOrientation,
        "text-orientation",
        "#propdef-text-orientation",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.unicode-bidi",
        CssKnownProperty::UnicodeBidi,
        "unicode-bidi",
        "#propdef-unicode-bidi",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.caret-color",
        CssKnownProperty::CaretColor,
        "caret-color",
        "#propdef-caret-color",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.outline-offset",
        CssKnownProperty::OutlineOffset,
        "outline-offset",
        "#propdef-outline-offset",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.resize",
        CssKnownProperty::Resize,
        "resize",
        "#propdef-resize",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.contain",
        CssKnownProperty::Contain,
        "contain",
        "#propdef-contain",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.transform-box",
        CssKnownProperty::TransformBox,
        "transform-box",
        "#propdef-transform-box",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.background-blend-mode",
        CssKnownProperty::BackgroundBlendMode,
        "background-blend-mode",
        "#propdef-background-blend-mode",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.isolation",
        CssKnownProperty::Isolation,
        "isolation",
        "#propdef-isolation",
        &[],
    ),
    CssFeatureMetadata::complete_property(
        "official.property.mix-blend-mode",
        CssKnownProperty::MixBlendMode,
        "mix-blend-mode",
        "#propdef-mix-blend-mode",
        &[],
    ),
    CssFeatureMetadata::complete(
        "official.property-alias.glyph-orientation-vertical",
        CssFeatureKind::PropertyAlias,
        "glyph-orientation-vertical",
        O_WRITING3,
        "#propdef-glyph-orientation-vertical",
    ),
    CssFeatureMetadata::complete(
        "official.selector.group",
        CssFeatureKind::Selector,
        "comma-separated selector groups",
        O_SELECTORS3,
        "#grouping",
    ),
    CssFeatureMetadata::complete(
        "official.selector.type",
        CssFeatureKind::Selector,
        "type selectors",
        O_SELECTORS3,
        "#type-selectors",
    ),
    CssFeatureMetadata::complete(
        "official.selector.universal",
        CssFeatureKind::Selector,
        "universal selector *",
        O_SELECTORS3,
        "#universal-selector",
    ),
    CssFeatureMetadata::complete(
        "official.selector.attribute-presence-value",
        CssFeatureKind::Selector,
        "[attr], [attr=value], [attr~=value], [attr|=value]",
        O_SELECTORS3,
        "#attribute-representation",
    ),
    CssFeatureMetadata::complete(
        "official.selector.attribute-substring",
        CssFeatureKind::Selector,
        "[attr^=value], [attr$=value], [attr*=value]",
        O_SELECTORS3,
        "#attribute-substrings",
    ),
    CssFeatureMetadata::complete(
        "official.selector.class",
        CssFeatureKind::Selector,
        ".class",
        O_SELECTORS3,
        "#class-html",
    ),
    CssFeatureMetadata::complete(
        "official.selector.id",
        CssFeatureKind::Selector,
        "#id",
        O_SELECTORS3,
        "#id-selectors",
    ),
    CssFeatureMetadata::complete(
        "official.selector.dynamic",
        CssFeatureKind::Selector,
        ":link, :visited, :hover, :active, :focus",
        O_SELECTORS3,
        "#dynamic-pseudos",
    ),
    CssFeatureMetadata::complete(
        "official.selector.target",
        CssFeatureKind::Selector,
        ":target",
        O_SELECTORS3,
        "#target-pseudo",
    ),
    CssFeatureMetadata::complete(
        "official.selector.lang",
        CssFeatureKind::Selector,
        ":lang()",
        O_SELECTORS3,
        "#lang-pseudo",
    ),
    CssFeatureMetadata::complete(
        "official.selector.ui-state",
        CssFeatureKind::Selector,
        ":enabled, :disabled, :checked, :indeterminate",
        O_SELECTORS3,
        "#UIstates",
    ),
    CssFeatureMetadata::complete(
        "official.selector.structural",
        CssFeatureKind::Selector,
        "Selectors 3 structural pseudo-classes",
        O_SELECTORS3,
        "#structural-pseudos",
    ),
    CssFeatureMetadata::complete(
        "official.selector.negation",
        CssFeatureKind::Selector,
        ":not()",
        O_SELECTORS3,
        "#negation",
    ),
    CssFeatureMetadata::complete(
        "official.selector.first-line",
        CssFeatureKind::Selector,
        ":first-line, ::first-line",
        O_SELECTORS3,
        "#first-line",
    ),
    CssFeatureMetadata::complete(
        "official.selector.first-letter",
        CssFeatureKind::Selector,
        ":first-letter, ::first-letter",
        O_SELECTORS3,
        "#first-letter",
    ),
    CssFeatureMetadata::complete(
        "official.selector.generated",
        CssFeatureKind::Selector,
        "::before, ::after",
        O_SELECTORS3,
        "#gen-content",
    ),
    CssFeatureMetadata::complete(
        "official.selector.combinator.descendant",
        CssFeatureKind::Selector,
        "descendant combinator",
        O_SELECTORS3,
        "#descendant-combinators",
    ),
    CssFeatureMetadata::complete(
        "official.selector.combinator.child",
        CssFeatureKind::Selector,
        "child combinator >",
        O_SELECTORS3,
        "#child-combinators",
    ),
    CssFeatureMetadata::complete(
        "official.selector.combinator.next-sibling",
        CssFeatureKind::Selector,
        "next-sibling combinator +",
        O_SELECTORS3,
        "#adjacent-sibling-combinators",
    ),
    CssFeatureMetadata::complete(
        "official.selector.combinator.subsequent-sibling",
        CssFeatureKind::Selector,
        "subsequent-sibling combinator ~",
        O_SELECTORS3,
        "#general-sibling-combinators",
    ),
    CssFeatureMetadata::complete(
        "official.selector.namespace-qualified-name",
        CssFeatureKind::Selector,
        "default, explicit-none, any, and named selector namespace constraints",
        O_NAMESPACES3,
        "#scope,#prefixes,#css-qnames",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.marker",
        CssFeatureKind::Selector,
        "::marker",
        X_PSEUDO4,
        "#marker-pseudo",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.selection",
        CssFeatureKind::Selector,
        "::selection",
        X_PSEUDO4,
        "#selectordef-selection",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.backdrop",
        CssFeatureKind::Selector,
        "::backdrop",
        X_PSEUDO4,
        "#selectordef-backdrop",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.generated-marker",
        CssFeatureKind::Selector,
        "::before::marker, ::after::marker",
        X_PSEUDO4,
        "#marker-pseudo",
    ),
    CssFeatureMetadata::complete(
        "official.media.query-list-core",
        CssFeatureKind::MediaQuery,
        "Media Queries 3 query-list core",
        O_MEDIA3,
        "#syntax",
    ),
    CssFeatureMetadata::complete(
        "ext.media.condition-syntax",
        CssFeatureKind::MediaQuery,
        "not, and, and comma-separated condition syntax",
        R_MEDIA4,
        "#mq-syntax",
    ),
    CssFeatureMetadata::complete(
        "ext.media.malformed-member-never",
        CssFeatureKind::MediaQuery,
        "malformed query-list members become Never",
        R_MEDIA4,
        "#mq-invalid",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.width",
        CssFeatureKind::MediaQuery,
        "width, min-width, max-width, including boolean forms where permitted",
        O_MEDIA3,
        "#width",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.height",
        CssFeatureKind::MediaQuery,
        "height, min-height, max-height, including boolean forms where permitted",
        O_MEDIA3,
        "#height",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.device-width",
        CssFeatureKind::MediaQuery,
        "device-width, min-device-width, max-device-width, including boolean forms where permitted",
        O_MEDIA3,
        "#device-width",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.device-height",
        CssFeatureKind::MediaQuery,
        "device-height, min-device-height, max-device-height, including boolean forms where permitted",
        O_MEDIA3,
        "#device-height",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.aspect-ratio",
        CssFeatureKind::MediaQuery,
        "aspect-ratio, min-aspect-ratio, max-aspect-ratio, including boolean forms where permitted",
        O_MEDIA3,
        "#aspect-ratio",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.device-aspect-ratio",
        CssFeatureKind::MediaQuery,
        "device-aspect-ratio, min-device-aspect-ratio, max-device-aspect-ratio, including boolean forms where permitted",
        O_MEDIA3,
        "#device-aspect-ratio",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.resolution",
        CssFeatureKind::MediaQuery,
        "resolution, min-resolution, max-resolution with dpi and dpcm, including boolean forms where permitted",
        O_MEDIA3,
        "#resolution",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.color",
        CssFeatureKind::MediaQuery,
        "color, min-color, max-color, including boolean forms where permitted",
        O_MEDIA3,
        "#color",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.color-index",
        CssFeatureKind::MediaQuery,
        "color-index, min-color-index, max-color-index, including boolean forms where permitted",
        O_MEDIA3,
        "#color-index",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.monochrome",
        CssFeatureKind::MediaQuery,
        "monochrome, min-monochrome, max-monochrome, including boolean forms where permitted",
        O_MEDIA3,
        "#monochrome",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.scan",
        CssFeatureKind::MediaQuery,
        "scan, including its boolean form",
        O_MEDIA3,
        "#scan",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.grid",
        CssFeatureKind::MediaQuery,
        "grid, including its boolean form",
        O_MEDIA3,
        "#grid",
    ),
    CssFeatureMetadata::complete(
        "ext.media.resolution.dppx",
        CssFeatureKind::MediaQuery,
        "dppx resolution unit",
        R_MEDIA4,
        "#resolution",
    ),
    CssFeatureMetadata::complete(
        "ext.supports.general-enclosed",
        CssFeatureKind::Value,
        "general-enclosed supports condition",
        X_VALUES4,
        "css-values-4/Overview.bs#general-enclosed",
    ),
    CssFeatureMetadata::partial(
        "ext.supports.selector",
        CssFeatureKind::Selector,
        "selector() supports test",
        R_CONDITIONAL4,
        "#at-supports",
        SUPPORTS_SELECTOR_SUBSET,
        SUPPORTS_SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "ext.media.range.width",
        CssFeatureKind::MediaQuery,
        "width comparison form",
        R_MEDIA4,
        "#width",
        "One-sided finite authored length comparisons are supported.",
        "Other valid Media Queries 4 width range forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::partial(
        "ext.media.range.height",
        CssFeatureKind::MediaQuery,
        "height comparison form",
        R_MEDIA4,
        "#height",
        "One-sided finite authored length comparisons are supported.",
        "Other valid Media Queries 4 height range forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::partial(
        "ext.media.range.resolution",
        CssFeatureKind::MediaQuery,
        "resolution comparison form",
        R_MEDIA4,
        "#resolution",
        "One-sided finite positive resolution comparisons are supported.",
        "Other valid Media Queries 4 resolution range forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::partial(
        "ext.media.range.color",
        CssFeatureKind::MediaQuery,
        "color comparison form",
        R_MEDIA4,
        "#color",
        "One-sided non-negative integer color comparisons are supported.",
        "Other valid Media Queries 4 color range forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::partial(
        "ext.media.range.monochrome",
        CssFeatureKind::MediaQuery,
        "monochrome comparison form",
        R_MEDIA4,
        "#monochrome",
        "One-sided non-negative integer monochrome comparisons are supported.",
        "Other valid Media Queries 4 monochrome range forms are outside the I01 subset.",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.orientation",
        CssFeatureKind::MediaQuery,
        "orientation",
        O_MEDIA3,
        "#orientation",
    ),
    CssFeatureMetadata::complete(
        "ext.media.hover",
        CssFeatureKind::MediaQuery,
        "hover",
        R_MEDIA4,
        "#hover",
    ),
    CssFeatureMetadata::complete(
        "ext.media.any-hover",
        CssFeatureKind::MediaQuery,
        "any-hover",
        R_MEDIA4,
        "#any-hover",
    ),
    CssFeatureMetadata::complete(
        "ext.media.pointer",
        CssFeatureKind::MediaQuery,
        "pointer",
        R_MEDIA4,
        "#pointer",
    ),
    CssFeatureMetadata::complete(
        "ext.media.any-pointer",
        CssFeatureKind::MediaQuery,
        "any-pointer",
        R_MEDIA4,
        "#any-pointer",
    ),
    CssFeatureMetadata::complete(
        "ext.media.prefers-color-scheme",
        CssFeatureKind::MediaQuery,
        "prefers-color-scheme",
        X_MEDIA5,
        "#prefers-color-scheme",
    ),
    CssFeatureMetadata::complete(
        "ext.media.prefers-reduced-motion",
        CssFeatureKind::MediaQuery,
        "prefers-reduced-motion",
        X_MEDIA5,
        "#prefers-reduced-motion",
    ),
    CssFeatureMetadata::complete(
        "ext.media.prefers-reduced-transparency",
        CssFeatureKind::MediaQuery,
        "prefers-reduced-transparency",
        X_MEDIA5,
        "#prefers-reduced-transparency",
    ),
    CssFeatureMetadata::complete(
        "ext.media.prefers-contrast",
        CssFeatureKind::MediaQuery,
        "prefers-contrast",
        X_MEDIA5,
        "#prefers-contrast",
    ),
    CssFeatureMetadata::complete(
        "ext.media.forced-colors",
        CssFeatureKind::MediaQuery,
        "forced-colors",
        X_MEDIA5,
        "#forced-colors",
    ),
    CssFeatureMetadata::complete(
        "ext.media.display-mode",
        CssFeatureKind::MediaQuery,
        "display-mode",
        X_DISPLAY_MODE_BASE,
        "display-mode baseline subset",
    ),
];

/// Returns the immutable support catalog in stable inventory order.
///
/// Preserved I01 records retain their exact lookup identities, and later atomic
/// records retain their source-specific identities.
#[must_use]
pub fn feature_catalog() -> &'static [CssFeatureMetadata] {
    &FEATURE_CATALOG
}

/// Returns the immutable specification-source registry in stable declaration order.
#[must_use]
pub const fn specification_sources() -> &'static [CssSpecificationSource] {
    SPECIFICATION_SOURCES
}

/// Returns the specification source for an exact stable source ID.
///
/// Lookup is case-sensitive and performs no trimming or aliasing.
#[must_use]
pub fn specification_source(id: &str) -> Option<&'static CssSpecificationSource> {
    SPECIFICATION_SOURCES
        .iter()
        .find(|source| source.id.as_str() == id)
}

/// Returns the immutable official conformance-exclusion registry.
#[must_use]
pub const fn conformance_exclusions() -> &'static [CssExclusionMetadata] {
    CONFORMANCE_EXCLUSIONS
}

/// Returns exclusion metadata for an exact stable exclusion ID.
///
/// Lookup is case-sensitive and performs no trimming or aliasing.
#[must_use]
pub fn conformance_exclusion(id: &str) -> Option<&'static CssExclusionMetadata> {
    CONFORMANCE_EXCLUSIONS
        .iter()
        .find(|exclusion| exclusion.id.as_str() == id)
}

/// Returns metadata for an exact stable feature ID.
///
/// Lookup is case-sensitive and performs no trimming, aliasing, parser dispatch,
/// or property-name classification.
#[must_use]
pub fn feature_metadata(id: &str) -> Option<&'static CssFeatureMetadata> {
    FEATURE_CATALOG
        .iter()
        .find(|feature| feature.id.as_str() == id)
}

/// Returns metadata for a recognized non-custom authored property name.
///
/// Canonical names and reviewed aliases use ASCII-case-insensitive matching.
/// Custom-property names and unknown spellings return `None`; this lookup does
/// not parse a declaration or classify its diagnostics.
#[must_use]
pub fn property_metadata(name: &str) -> Option<CssPropertyMetadata> {
    FEATURE_CATALOG
        .iter()
        .filter(|feature| feature.kind == CssFeatureKind::Property)
        .find(|feature| {
            feature.spelling.eq_ignore_ascii_case(name)
                || feature
                    .property_aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(name))
        })
        .map(|feature| CssPropertyMetadata { feature })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_lookup_exposes_named_status_metadata() {
        let importance =
            feature_metadata("foundation.declaration.importance").expect("importance metadata");
        assert_eq!(importance.status(), CssSupportStatus::Complete);
        assert_eq!(importance.supported_subset(), None);
        assert_eq!(importance.unsupported_remainder(), None);
        assert_eq!(importance.recognized_unsupported_code(), None);

        let import = feature_metadata("baseline.rule.import").expect("import metadata");
        assert_eq!(import.status(), CssSupportStatus::Complete);
        assert_eq!(import.supported_subset(), None);
        assert_eq!(import.unsupported_remainder(), None);
        assert_eq!(import.recognized_unsupported_code(), None);

        let namespace = feature_metadata("later.rule.namespace").expect("namespace metadata");
        assert_eq!(namespace.status(), CssSupportStatus::Complete);
        assert_eq!(namespace.supported_subset(), None);
        assert_eq!(namespace.unsupported_remainder(), None);
        assert_eq!(namespace.recognized_unsupported_code(), None);
    }
}
