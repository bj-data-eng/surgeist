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
/// The three states are closed: callers can distinguish a complete
/// production, a documented supported subset, and a recognized spelling that
/// is never retained as that production. A partial remainder describes scope
/// not claimed as supported; it does not establish that every remainder form
/// is valid CSS. Consult the cited standard for normative validity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssSupportStatus {
    /// The complete identified production is implemented.
    Complete,
    /// Only the record's documented supported subset is implemented. The
    /// remainder may be known-valid unimplemented syntax or unresolved standard
    /// applicability; this status alone does not classify its validity.
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
    /// A named feature selected by Snapshot 2026 section 4, without selecting
    /// its containing pre-CR module as a whole.
    Snapshot2026PreCrException,
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

/// Immutable support metadata for one atomic CSS production.
///
/// Records describe authored syntax only. They do not dispatch the parser,
/// validate source, perform selector/query matching, or resolve authored values.
/// Fields and construction are catalog-owned so stable identity
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
    property: Option<CssKnownProperty>,
    property_aliases: &'static [&'static str],
}

/// The implementation state of one schema-backed property row.
#[derive(Clone, Copy)]
enum PropertyRowStatus {
    Complete,
    Partial {
        supported_subset: &'static str,
        unsupported_remainder: &'static str,
    },
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
            property: None,
            property_aliases: &[],
        }
    }

    /// Builds a property row whose spelling, stable ID and aliases come from the
    /// property schema; only provenance, status and an exceptional production
    /// anchor are row data.
    const fn property(
        property: CssKnownProperty,
        source: CssSpecificationSource,
        status: PropertyRowStatus,
        production: Option<&'static str>,
    ) -> Self {
        let (status, supported_subset, unsupported_remainder) = match status {
            PropertyRowStatus::Complete => (CssSupportStatus::Complete, None, None),
            PropertyRowStatus::Partial {
                supported_subset,
                unsupported_remainder,
            } => (
                CssSupportStatus::Partial,
                Some(supported_subset),
                Some(unsupported_remainder),
            ),
        };
        Self {
            id: CssFeatureId::new(property.stable_id()),
            kind: CssFeatureKind::Property,
            spelling: property.canonical_name(),
            source,
            production: match production {
                Some(production) => production,
                None => property_definition_anchor(property),
            },
            status,
            supported_subset,
            unsupported_remainder,
            recognized_unsupported_code: None,
            property: Some(property),
            property_aliases: property.aliases(),
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

    /// Returns the non-empty scope not claimed as supported exactly for partial
    /// records. The description may identify known-valid unimplemented syntax
    /// or explicitly unresolved standard applicability; neither this accessor
    /// nor [`CssSupportStatus::Partial`] proves that every remainder form is valid.
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
}

/// Immutable support-catalog metadata for one recognized non-custom authored CSS property.
///
/// The metadata reports parser support; it does not apply cascade, substitute
/// variables, resolve values, or dispatch property parsing. Construction is
/// catalog-owned so callers cannot forge a property-to-feature association.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssPropertySupportMetadata {
    feature: &'static CssFeatureMetadata,
}

impl CssPropertySupportMetadata {
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

// Bounded Page grammar/property domain selected by the Page consumer contract.
const L_PAGE3_20181018: CssSpecificationSource = dated_source!(
    "L-PAGE3-20181018",
    "CSS Paged Media",
    "3",
    CssSpecificationTier::LaterStandard,
    "https://www.w3.org/TR/2018/WD-css-page-3-20181018/"
);
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
pub(crate) const O_VARIABLES1: CssSpecificationSource = dated_source!(
    "O-VARIABLES1",
    "CSS Custom Properties for Cascading Variables",
    "1",
    CssSpecificationTier::Snapshot2026Official,
    "https://www.w3.org/TR/2022/CR-css-variables-1-20220616/"
);
// Required definitions only; Env1 is not a selected whole-module profile member.
const D_ENV1: CssSpecificationSource = dated_source!(
    "D-ENV1",
    "CSS Environment Variables",
    "1",
    CssSpecificationTier::LaterStandard,
    "https://www.w3.org/TR/2025/WD-css-env-1-20250923/"
);
// Required Content 3 attr() definitions only; Values 5 is not selected in full.
const D_VALUES5_ATTR: CssSpecificationSource = dated_source!(
    "D-VALUES5-ATTR",
    "CSS Values and Units",
    "5",
    CssSpecificationTier::LaterStandard,
    "https://www.w3.org/TR/2024/WD-css-values-5-20241111/"
);
// Snapshot 2026 §4 selects request URL modifiers as a feature-scoped pre-CR
// release exception; its bibliography pins Values 5 WD 2024-11-11.
const I_VALUES5_REQUEST: CssSpecificationSource = dated_source!(
    "I-VALUES5-REQUEST",
    "CSS Values and Units",
    "5",
    CssSpecificationTier::Snapshot2026PreCrException,
    "https://www.w3.org/TR/2024/WD-css-values-5-20241111/"
);
// Specific Images4 at-position hyperlinks supersede its general Values3 prose.
// This imports the complete position production, not the whole Values5 module.
const D_VALUES5_GRADIENT_POSITION: CssSpecificationSource = dated_source!(
    "D-VALUES5-GRADIENT-POSITION",
    "CSS Values and Units",
    "5",
    CssSpecificationTier::LaterStandard,
    "https://www.w3.org/TR/2024/WD-css-values-5-20241111/"
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
    "https://www.w3.org/TR/2026/CRD-css-color-4-20260908/"
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
    "https://www.w3.org/TR/2026/REC-css-ui-3-20260407/"
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
// Required Transitions 1 definitions only; Easing 2 is not selected in full.
const D_EASING2: CssSpecificationSource = dated_source!(
    "D-EASING2",
    "CSS Easing Functions",
    "2",
    CssSpecificationTier::LaterStandard,
    "https://www.w3.org/TR/2024/WD-css-easing-2-20240829/"
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
    R_COLORADJUST1,
    "R-COLORADJUST1",
    "CSS Color Adjustment",
    "1",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/"
);
profile_source!(
    R_SCROLLSNAP1,
    "R-SCROLLSNAP1",
    "CSS Scroll Snap",
    "1",
    CssSpecificationTier::Snapshot2026Reliable,
    "https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/"
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
    S_SPEECH1,
    "S-SPEECH1",
    "CSS Speech",
    "1",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/"
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
    S_VIEWTRANSITIONS1,
    "S-VIEWTRANSITIONS1",
    "CSS View Transitions",
    "1",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/"
);
profile_source!(
    I_WILLCHANGE1,
    "I-WILLCHANGE1",
    "CSS Will Change",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2022/CRD-css-will-change-1-20220505/"
);
profile_source!(
    I_SCROLLANCHORING1,
    "I-SCROLLANCHORING1",
    "CSS Scroll Anchoring",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2020/WD-css-scroll-anchoring-1-20201111/"
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
    I_MOTION1,
    "I-MOTION1",
    "Motion Path",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2024/WD-motion-1-20241105/"
);
profile_source!(
    S_TEXT3,
    "S-TEXT3",
    "CSS Text",
    "3",
    CssSpecificationTier::Snapshot2026Stable,
    "https://www.w3.org/TR/2026/CRD-css-text-3-20260814/"
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
    I_SIZING3_20260904,
    "I-SIZING3-20260904",
    "CSS Box Sizing",
    "3",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/"
);
profile_source!(
    I_LOGICAL1_20251204,
    "I-LOGICAL1-20251204",
    "CSS Logical Properties and Values",
    "1",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2025/WD-css-logical-1-20251204/"
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
    I_FONTS4_20260907,
    "I-FONTS4-20260907",
    "CSS Fonts",
    "4",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/"
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
    I_COLOR5_20260908,
    "I-COLOR5-20260908",
    "CSS Color",
    "5",
    CssSpecificationTier::Snapshot2026Interop,
    "https://www.w3.org/TR/2026/WD-css-color-5-20260908/"
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
    I_VALUES4_20240312,
    "I-VALUES4-20240312",
    "CSS Values and Units",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2024/WD-css-values-4-20240312/"
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
    X_SIZING4_20260904,
    "X-SIZING4-20260904",
    "CSS Box Sizing",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/"
);
// Snapshot §4 selects only the text-box family. Imported <text-edge> grammar
// does not select line-fit-edge or the whole Inline Layout module.
profile_source!(
    F_INLINE3_20241218,
    "F-INLINE3-20241218",
    "CSS Inline Layout",
    "3",
    CssSpecificationTier::Snapshot2026PreCrException,
    "https://www.w3.org/TR/2024/WD-css-inline-3-20241218/"
);
profile_source!(
    X_TEXT4,
    "X-TEXT4",
    "CSS Text",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2026/WD-css-text-4-20260814/"
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
// Dated imports for UI4 image-1D definitions.
profile_source!(
    X_IMAGES4_20250930,
    "X-IMAGES4-20250930",
    "CSS Images",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2025/WD-css-images-4-20250930/"
);
// Snapshot 2026 section 4 selects conic gradients and gradient interpolation.
// Their reachable stop-list grammar is selected without the whole Images4 module.
const F_IMAGES4_GRADIENTS_20250930: CssSpecificationSource = dated_source!(
    "F-IMAGES4-GRADIENTS-20250930",
    "CSS Images",
    "4",
    CssSpecificationTier::Snapshot2026PreCrException,
    "https://www.w3.org/TR/2025/WD-css-images-4-20250930/"
);
profile_source!(
    X_BORDERS4_20251216,
    "X-BORDERS4-20251216",
    "CSS Borders and Box Decorations",
    "4",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2025/WD-css-borders-4-20251216/"
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

profile_source!(
    X_BACKDROP_FILTER,
    "X-BACKDROP-FILTER",
    "Filter Effects",
    "2 named property exception",
    CssSpecificationTier::SurgeistExtension,
    "https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs"
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

profile_source!(
    X_GRID3_20260121,
    "X-GRID3-20260121",
    "CSS Grid Layout",
    "3",
    CssSpecificationTier::SurgeistExtension,
    "https://www.w3.org/TR/2026/WD-css-grid-3-20260121/"
);

const CSS_STYLE_ATTRIBUTES: CssSpecificationSource = O_STYLE_ATTR;
const CSS_CASCADE_4: CssSpecificationSource = O_CASCADE4;

profile_source!(
    I_WEBKIT_SVG_GLYPH,
    "I-WEBKIT-SVG-GLYPH",
    "SVG glyph authored compatibility",
    "WebKit73aa6c89e2cb77c46184a81aec944e4ab99d114d",
    CssSpecificationTier::Snapshot2026Interop,
    "https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json#L5350-L5367"
);

static SPECIFICATION_SOURCES: &[CssSpecificationSource] = &[
    L_PAGE3_20181018,
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
    D_ENV1,
    D_VALUES5_ATTR,
    I_VALUES5_REQUEST,
    D_VALUES5_GRADIENT_POSITION,
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
    D_EASING2,
    O_COUNTERSTYLES3,
    R_MEDIA4,
    R_SCROLLBARS1,
    R_SCROLLSNAP1,
    R_COLORADJUST1,
    R_GRID1,
    R_GRID2,
    R_CASCADE5,
    R_CONDITIONAL4,
    S_DISPLAY3,
    S_SPEECH1,
    S_WRITING4,
    S_BREAK3,
    S_VIEWTRANSITIONS1,
    I_WILLCHANGE1,
    I_SCROLLANCHORING1,
    S_ALIGN3,
    S_SHAPES1,
    S_TEXT3,
    I_MOTION1,
    S_TEXTDECOR3,
    S_MASKING1,
    I_TRANSITIONS1,
    I_ANIMATIONS1,
    I_FILTER1,
    I_SIZING3,
    I_SIZING3_20260904,
    I_LOGICAL1_20251204,
    I_TRANSFORMS2,
    I_LISTS3,
    I_POSITION3,
    I_FONTS4,
    I_FONTS4_20260907,
    I_COLOR5,
    I_COLOR5_20260908,
    I_SELECTORS4,
    I_CONTAIN2,
    I_NESTING1,
    X_CONTAIN3,
    X_CONDITIONAL5,
    X_CASCADE6,
    X_PSEUDO4,
    X_VALUES4,
    I_VALUES4_20240312,
    X_MEDIA5,
    X_OVERFLOW3,
    X_SIZING4,
    X_SIZING4_20260904,
    X_TEXT4,
    F_INLINE3_20241218,
    X_TEXTDECOR4,
    X_UI4,
    X_IMAGES4_20250930,
    F_IMAGES4_GRADIENTS_20250930,
    X_BORDERS4_20251216,
    X_CONTENT3,
    X_FULLSCREEN,
    X_BACKDROP_FILTER,
    X_DISPLAY_MODE_BASE,
    X_GRID_TOLERANCE_BASE,
    X_GRID3_20260121,
    I_WEBKIT_SVG_GLYPH,
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
        "#input-method-editor",
        CssExclusionReason::SupersededWithoutCurrentProduction,
        ["X-UI4"]
    ),
    exclusion!(
        "excluded.X-UI4.property.ime-mode",
        X_UI4,
        "#input-method-editor",
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
    // UI3 Appendix E (#default-style-sheet) is informative and covered by
    // excluded.O-UI3.informative-audit, rather than an execution requirement.
    exclusion!(
        "excluded.O-UI3.behavior",
        O_UI3,
        "#resize; #text-overflow; #ellipsing-details; #ellipsis-interaction; #ellipsis-scrolling; #cursor; #canvas_cursor; #caret-color (rendering and user interaction only)",
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
        "easing evaluation and output algorithms",
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
    active_coverage!("official.property.inline-size"),
    active_coverage!("official.property.block-size"),
    active_coverage!("baseline.property.left"),
    active_coverage!("baseline.property.letter-spacing"),
    active_coverage!("baseline.property.line-height"),
    active_coverage!("baseline.property.list-style"),
    active_coverage!("baseline.property.list-style-image"),
    active_coverage!("baseline.property.list-style-position"),
    active_coverage!("baseline.property.list-style-type"),
    active_coverage!("official.property.marker-side"),
    active_coverage!("baseline.property.max-height"),
    active_coverage!("baseline.property.max-width"),
    active_coverage!("official.property.max-inline-size"),
    active_coverage!("official.property.max-block-size"),
    active_coverage!("baseline.property.min-height"),
    active_coverage!("baseline.property.min-width"),
    active_coverage!("official.property.min-inline-size"),
    active_coverage!("official.property.min-block-size"),
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
    active_coverage!("official.property.font-language-override"),
    active_coverage!("official.property.font-optical-sizing"),
    active_coverage!("baseline.property.font-size"),
    active_coverage!("official.property.font-size-adjust"),
    active_coverage!("official.property.font-variation-settings"),
    active_coverage!("baseline.property.font-stretch"),
    active_coverage!("baseline.property.font-style"),
    active_coverage!("official.property.font-synthesis"),
    active_coverage!("official.property.font-synthesis-weight"),
    active_coverage!("official.property.font-synthesis-style"),
    active_coverage!("official.property.font-synthesis-small-caps"),
    active_coverage!("official.property.font-synthesis-position"),
    active_coverage!("baseline.property.font-variant"),
    active_coverage!("official.property.font-variant-alternates"),
    active_coverage!("official.property.font-variant-caps"),
    active_coverage!("official.property.font-variant-east-asian"),
    active_coverage!("official.property.font-variant-emoji"),
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
    active_coverage!("official.property.container-name"),
    active_coverage!("official.property.container-type"),
    active_coverage!("official.property.container"),
    active_coverage!("official.property.flex-flow"),
    active_coverage!("baseline.property.flex-grow"),
    active_coverage!("baseline.property.flex-shrink"),
    active_coverage!("baseline.property.flex-wrap"),
    active_coverage!("baseline.property.justify-content"),
    active_coverage!("baseline.property.box-sizing"),
    active_coverage!("official.property.nav-up"),
    active_coverage!("official.property.nav-right"),
    active_coverage!("official.property.nav-down"),
    active_coverage!("official.property.nav-left"),
    active_coverage!("official.property.caret-animation"),
    active_coverage!("official.property.caret-shape"),
    active_coverage!("official.property.caret"),
    active_coverage!("official.property.interactivity"),
    active_coverage!("official.property.interest-delay-start"),
    active_coverage!("official.property.interest-delay-end"),
    active_coverage!("official.property.interest-delay"),
    active_coverage!("official.property.accent-color"),
    active_coverage!("official.property.appearance"),
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
    active_coverage!("official.selector.logical-page-pseudo"),
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
    active_coverage!("ext.selector.dir"),
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
    active_coverage!("required.value.request-url-modifier"),
    active_coverage!("official.value.integer"),
    active_coverage!("official.value.number"),
    active_coverage!("official.value.dimension"),
    active_coverage!("official.value.percentage"),
    active_coverage!("ext.value.image-1d"),
    active_coverage!("ext.value.stripes"),
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
    active_coverage!("required.value.environment-substitution"),
    active_coverage!("required.value.attribute-substitution"),
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
    active_coverage!("ext.value.filter-image"),
    active_coverage!("official.value.gradient"),
    active_coverage!("official.value.linear-gradient"),
    active_coverage!("official.value.radial-gradient"),
    active_coverage!("official.value.repeating-linear-gradient"),
    active_coverage!("official.value.repeating-radial-gradient"),
    active_coverage!("official.value.color-stop-list"),
    active_coverage!("ext.value.conic-gradient"),
    active_coverage!("ext.value.gradient-interpolation"),
    active_coverage!("ext.value.gradient-stop-list"),
    active_coverage!("required.value.gradient-position"),
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
    active_coverage!("official.descriptor.font-variation-settings"),
    active_coverage!("official.descriptor.font-named-instance"),
    active_coverage!("official.descriptor.font-language-override"),
    active_coverage!("official.descriptor.ascent-override"),
    active_coverage!("official.descriptor.descent-override"),
    active_coverage!("official.descriptor.line-gap-override"),
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
    active_coverage!("official.value.linear-easing"),
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
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129,
);

const BASELINE_RULE_SUBSET: &str =
    "The baseline parser spelling and the selected recovery extensions are supported.";
const BASELINE_RULE_REMAINDER: &str =
    "Other valid forms of the cited rule production are outside the supported subset.";
const SELECTOR_REMAINDER: &str =
    "Other valid forms of the cited Selectors production are outside the supported subset.";
const SUPPORTS_SELECTOR_SUBSET: &str = "selector() accepts complete Selectors 3 plus the selected extensions: the || column combinator; i and s attribute modifiers; :scope, :focus-visible, :focus-within, :required, :optional, :valid, :invalid, :placeholder-shown, :modal, :fullscreen, :popover-open, :default, :indeterminate, :read-only, :read-write, :in-range, and :out-of-range; :is(), :where(), :has(), selector-list :not(), and nth-child of lists; and ::marker, ::selection, ::backdrop, and generated-marker sequences. The selected named states :defined, :any-link, :playing, :paused, :seeking, :buffering, :stalled, :muted, :volume-locked, :open, :picture-in-picture, :unchecked, :user-valid and :user-invalid retain distinct symbolic identities, with decoded ASCII-insensitive admission and canonical lowercase specified output; functional forms remain invalid and :any-link is not lowered. Directionality accepts one identifier and language accepts nonempty comma-separated identifier or string ranges. Required shadow definitions add :host, compound-argument :host() and :host-context(), ::slotted(), and identifier-list ::part(). Autofill and its required -webkit-autofill alias share the standard identity; unknown nonfunctional -webkit- pseudo-elements retain symbolic match-nothing identities with ASCII-lowercase output. Pseudo 4 adds Prefix, Suffix, SearchText, TargetText, SpellingError, GrammarError, Highlight with one checked custom identifier, Placeholder, FileSelectorButton and DetailsContent. Nonfunctional Current is limited to SearchText and element-backed suffix contexts and inherited logical arguments; ordinary Current and functional Current remain unsupported. View Transitions 1 adds its tree-abiding root and Group, ImagePair, Old and New functional pseudos with wildcard or custom-ident arguments, named-descendant OnlyChild suffixes and exact root-to-group-to-image-pair-to-old/new compounds. Ordered pseudo-element suffixes retain contextual pseudo-classes and defined child transitions, resetting permissions at each new segment. All typed selector tests remain symbolic, including optional UA pseudo implementation and matching prohibitions.";
const SUPPORTS_SELECTOR_REMAINDER: &str = "Unselected Selectors 4 pseudo-classes and pseudo-elements, and syntax outside those atomic extension rows remain outside the typed subset; balanced content is preserved as general-enclosed authored syntax.";
const QUERY_REMAINDER: &str =
    "Other valid forms of the cited query production are outside the supported subset.";
const PROPERTY_SUBSET: &str = "The property-specific parser behavior at 4b288d6:src/parser/mod.rs, plus whole-value CSS-wide keywords and syntactically admissible substitution-dependent authored values, is supported.";
const PROPERTY_REMAINDER: &str =
    "Other valid forms of the cited property production are outside the supported subset.";
const COLOR5_RELATIVE_SUBSET: &str = "Relative rgb()/rgba(), hsl()/hsla(), hwb(), lab(), lch(), oklab(), oklch(), predefined and custom-profile color(), and alpha() preserve authored channels and symbolic calculations.";
const COLOR5_RELATIVE_REMAINDER: &str = "Unselected CSS Color 5 source-color functions remain unsupported; profile binding and color evaluation belong to downstream resolution.";
const COLOR5_MIX_SUBSET: &str = "Authored color-mix() supports an optional interpolation method, ordered nonempty color lists, literal or calculated percentage weights before or after colors, and predefined, polar or symbolic custom spaces.";
const COLOR5_MIX_REMAINDER: &str = "Unselected contextual Color 5 alternatives and downstream color evaluation remain outside this authored subset.";
const GRID_AUTO_FLOW_SUBSET: &str = "The authored grid-auto-flow grammar accepts exactly normal, dense, row, row dense, column, and column dense; axis and dense may appear in either order. Normal and bare dense remain symbolic; intrinsic longhand expansion and canonical specified serialization preserve the six states.";
const GRID_AUTO_FLOW_REMAINDER: &str = "Grid 3 §2.3 issue #12803 has not settled whether its context-dependent orientation extends grid-auto-flow or belongs to a separate grid-lanes-direction property. The normal initial and implicit-direction dense choice follow the recorded WebKit behavior; runtime orientation and placement belong downstream.";

macro_rules! define_property_definition_anchors {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty, $all_parser:ident, $all_dispatch:block $(, expansion = $all_kind:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $accessor:ident, $parser:ident, $dispatch:block
        $(, expansion = $expansion:ident { $($metadata:tt)* })?;
    )*) => {
        /// Returns the `#propdef-` anchor that names a schema property's definition.
        const fn property_definition_anchor(property: CssKnownProperty) -> &'static str {
            match property {
                CssKnownProperty::All => concat!("#propdef-", $all_canonical),
                $(CssKnownProperty::$variant => concat!("#propdef-", $canonical),)*
            }
        }
    };
}

crate::properties::property_schema!(define_property_definition_anchors, input, numeric);

/// One schema-backed property row: the variant, its provenance, its status and,
/// when the cited definition is not at `#propdef-<name>`, its production anchor.
macro_rules! property_row {
    (@production) => {
        None
    };
    (@production $production:literal) => {
        Some($production)
    };
    ($variant:ident, $source:expr, complete $(, $production:literal)?) => {
        CssFeatureMetadata::property(
            CssKnownProperty::$variant,
            $source,
            PropertyRowStatus::Complete,
            property_row!(@production $($production)?),
        )
    };
    ($variant:ident, $source:expr, partial $(, $production:literal)?) => {
        property_row!($variant, $source, partial(PROPERTY_SUBSET, PROPERTY_REMAINDER) $(, $production)?)
    };
    ($variant:ident, $source:expr, partial($subset:expr, $remainder:expr) $(, $production:literal)?) => {
        CssFeatureMetadata::property(
            CssKnownProperty::$variant,
            $source,
            PropertyRowStatus::Partial {
                supported_subset: $subset,
                unsupported_remainder: $remainder,
            },
            property_row!(@production $($production)?),
        )
    };
}

static FEATURE_CATALOG: [CssFeatureMetadata; 794] = [
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
        I_FONTS4_20260907,
        "#font-face-rule",
    ),
    CssFeatureMetadata::complete(
        "baseline.rule.keyframes",
        CssFeatureKind::Rule,
        "@keyframes",
        I_ANIMATIONS1,
        "#keyframes",
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
        "#contents-of,#placement",
    ),
    CssFeatureMetadata::partial(
        "baseline.rule.container",
        CssFeatureKind::Rule,
        "@container",
        X_CONDITIONAL5,
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
        "Baseline custom-property names and authored token streams, including their recovery behavior, are supported.",
        "Other valid CSS Variables custom-property declaration forms are outside the supported subset.",
    ),
    CssFeatureMetadata::partial(
        "baseline.value.substitution-dependent",
        CssFeatureKind::Value,
        "preserved known-property and font-palette descriptor values containing substitution functions",
        O_VARIABLES1,
        "#using-variables",
        "Known-property and font-palette descriptor values with syntactically admissible var() references remain authored and symbolic.",
        "Other descriptor contexts outside the selected authored consumers remain unsupported; variable resolution belongs to style.",
    ),
    CssFeatureMetadata::partial(
        "required.value.environment-substitution",
        CssFeatureKind::Value,
        "env() in authored property and selected font-palette, font-face, counter-style, font-feature-values and color-profile descriptor values",
        D_ENV1,
        "#funcdef-env,#env-function,#env-in-shorthands",
        "Known-property values, selected font-palette and recognized font-face descriptors, all ten counter-style descriptors, font-feature-values font-display and named values in all seven subsidiary blocks, and color-profile descriptors qualify for pending substitution through valid env() functions, including exact integer indices, symbolic integer calculations and token-preserving fallbacks. Strict replacement reentry rejects residual env().",
        "Other Env1 contexts and environment lookup/substitution execution remain outside this authored subset; this record does not select the complete Env1 module.",
    ),
    CssFeatureMetadata::partial(
        "required.value.attribute-substitution",
        CssFeatureKind::Value,
        "attr() in authored known-property values",
        D_VALUES5_ATTR,
        "#component-function-commas,#typedef-syntax,#attr-notation,#attr-substitution,#attr-security",
        "Known-property values with syntactically valid attr() references remain whole-value pending, retaining original components and provenance. Strict replacement reentry rejects residual attr() and checks the ordinary property grammar.",
        "Attribute lookup, fallback choice and computed-value substitution remain downstream; descriptor and query admission and the rest of Values 5 are not selected here.",
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
        I_VALUES4_20240312,
        "#urls",
    ),
    CssFeatureMetadata::complete(
        "official.value.url-modifier",
        CssFeatureKind::Value,
        "<url-modifier>",
        I_VALUES4_20240312,
        "#url-modifiers",
    ),
    CssFeatureMetadata::partial(
        "required.value.request-url-modifier",
        CssFeatureKind::Value,
        "recognized request URL modifier functions",
        I_VALUES5_REQUEST,
        "#request-url-modifiers,#typedef-request-url-modifier",
        "Shared quoted url()/src() admission and checked modifier construction validate crossorigin(anonymous | use-credentials), integrity(<string>) and the eight referrerpolicy() keywords. Typed inspection retains decoded request data while original name, argument components, origins and modifier order remain authored; unknown functions and bare identifiers retain generic extension syntax. Strict importing-property and descriptor reentry reuses this grammar.",
        "Fetch request steps, CORS behavior, integrity evaluation, referrer execution and resource resolution remain downstream; this feature-scoped record does not select all of Values 5.",
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
        "ext.value.image-1d",
        CssFeatureKind::Value,
        "<image-1D>",
        X_IMAGES4_20250930,
        "#typedef-image-1d",
    ),
    CssFeatureMetadata::complete(
        "ext.value.stripes",
        CssFeatureKind::Value,
        "stripes()",
        X_IMAGES4_20250930,
        "#stripes",
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
        "official.value.quirky-length",
        CssFeatureKind::Value,
        "<quirky-length> in explicit document contexts",
        I_VALUES4_20240312,
        "#deprecated-quirky-length",
    ),
    CssFeatureMetadata::complete(
        "official.value.quirky-color",
        CssFeatureKind::Value,
        "<quirky-color> in explicit document contexts",
        O_COLOR4,
        "#quirky-color",
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
    CssFeatureMetadata::complete(
        "interop.value.light-dark-color",
        CssFeatureKind::Value,
        "light-dark() color",
        I_COLOR5_20260908,
        "#typedef-light-dark-color",
    ),
    CssFeatureMetadata::complete(
        "interop.value.light-dark-image",
        CssFeatureKind::Value,
        "light-dark() image",
        I_COLOR5_20260908,
        "#typedef-light-dark-image",
    ),
    CssFeatureMetadata::complete(
        "interop.value.contrast-color",
        CssFeatureKind::Value,
        "contrast-color()",
        I_COLOR5_20260908,
        "#funcdef-contrast-color",
    ),
    CssFeatureMetadata::complete(
        "interop.value.device-cmyk",
        CssFeatureKind::Value,
        "device-cmyk()",
        I_COLOR5_20260908,
        "#funcdef-device-cmyk",
    ),
    // The selected Grid3 production adds two standalone display alternatives;
    // it does not extend Display3's syntactic <display-inside> production.
    CssFeatureMetadata::complete(
        "ext.value.grid-lanes-display",
        CssFeatureKind::Value,
        "grid-lanes | inline-grid-lanes",
        X_GRID3_20260121,
        "#grid-lanes-containers",
    ),
    CssFeatureMetadata::complete(
        "ext.value.column-width.box-size",
        CssFeatureKind::Value,
        "<box-size> for column-width",
        X_SIZING4_20260904,
        "#column-sizing",
    ),
    CssFeatureMetadata::complete(
        "ext.value.grid-repeat",
        CssFeatureKind::Value,
        "repeat()",
        X_GRID3_20260121,
        "#intrinsic-auto-repeat",
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
        "ext.value.filter-image",
        CssFeatureKind::Value,
        "filter()",
        I_FILTER1,
        "#FilterCSSImageValue",
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
        "ext.value.conic-gradient",
        CssFeatureKind::Value,
        "conic-gradient()",
        F_IMAGES4_GRADIENTS_20250930,
        "#conic-gradient-syntax",
    ),
    CssFeatureMetadata::complete(
        "ext.value.gradient-interpolation",
        CssFeatureKind::Value,
        "selected gradient color interpolation",
        F_IMAGES4_GRADIENTS_20250930,
        "#linear-gradients",
    ),
    CssFeatureMetadata::complete(
        "ext.value.gradient-stop-list",
        CssFeatureKind::Value,
        "selected linear and angular color-stop lists",
        F_IMAGES4_GRADIENTS_20250930,
        "#color-stop-syntax",
    ),
    CssFeatureMetadata::complete(
        "required.value.gradient-position",
        CssFeatureKind::Value,
        "<position> in selected radial and conic gradient at productions",
        D_VALUES5_GRADIENT_POSITION,
        "#typedef-position",
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
        D_EASING2,
        "#easing-functions",
    ),
    CssFeatureMetadata::complete(
        "official.value.linear-easing",
        CssFeatureKind::Value,
        "linear()",
        D_EASING2,
        "#linear-easing-function-syntax,#create-a-linear-easing-function",
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
        "official.value.ray",
        CssFeatureKind::Value,
        "ray()",
        I_MOTION1,
        "#ray-function",
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
    CssFeatureMetadata::complete(
        "ext.value.basic-shape",
        CssFeatureKind::Value,
        "<basic-shape>",
        S_SHAPES1,
        "#typedef-basic-shape",
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.shape",
        CssFeatureKind::Value,
        "shape()",
        S_SHAPES1,
        "#funcdef-basic-shape-shape",
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
        "ext.value.basic-shape.path",
        CssFeatureKind::Value,
        "path()",
        S_SHAPES1,
        "#funcdef-basic-shape-path",
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.rect",
        CssFeatureKind::Value,
        "rect()",
        S_SHAPES1,
        "#funcdef-basic-shape-rect",
    ),
    CssFeatureMetadata::complete(
        "ext.value.basic-shape.xywh",
        CssFeatureKind::Value,
        "xywh()",
        S_SHAPES1,
        "#funcdef-basic-shape-xywh",
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
        "#terminology,#declaration,#syntax,#scope,#prefixes",
    ),
    CssFeatureMetadata::complete(
        "later.rule.supports",
        CssFeatureKind::Rule,
        "@supports",
        O_CONDITIONAL3,
        "#at-supports",
    ),
    CssFeatureMetadata::partial(
        "ext.rule.when",
        CssFeatureKind::Rule,
        "@when",
        X_CONDITIONAL5,
        "#when-rule",
        "Complete adopted authored profile: exact media-feature and supports-declaration leaves, opaque fallback, homogeneous algebra, checked ordinary/scoped/style bodies and shared specified output; https://github.com/bj-data-eng/surgeist/issues/647",
        "The selected Conditional5 source supplies no fully normative generic when production or literal When/Else CSSOM algorithm. This adopted authored profile is complete within its bound; condition evaluation remains downstream.",
    ),
    CssFeatureMetadata::partial(
        "ext.rule.else",
        CssFeatureKind::Rule,
        "@else",
        X_CONDITIONAL5,
        "#else-rule",
        "Complete adopted authored profile: optional adopted condition, original-token adjacent conditional chains, checked complete-list assembly and shared specified output; https://github.com/bj-data-eng/surgeist/issues/647",
        "The selected Conditional5 source supplies no fully normative generic when production or literal When/Else CSSOM algorithm. This adopted authored profile is complete within its bound; condition evaluation remains downstream.",
    ),
    CssFeatureMetadata::complete(
        "ext.rule.supports-condition",
        CssFeatureKind::Rule,
        "@supports-condition",
        X_CONDITIONAL5,
        "#at-ruledef-supports-condition",
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
        L_PAGE3_20181018,
        "#syntax-page-selector",
    ),
    CssFeatureMetadata::complete(
        "official.selector.page-pseudo",
        CssFeatureKind::Selector,
        ":left|:right|:first|:blank",
        L_PAGE3_20181018,
        "#page-selectors",
    ),
    CssFeatureMetadata::complete(
        "official.selector.logical-page-pseudo",
        CssFeatureKind::Selector,
        ":recto|:verso",
        I_LOGICAL1_20251204,
        "#page",
    ),
    CssFeatureMetadata::complete(
        "later.rule.font-feature-values",
        CssFeatureKind::Rule,
        "@font-feature-values",
        I_FONTS4_20260907,
        "#font-feature-values-syntax",
    ),
    CssFeatureMetadata::complete(
        "interop.rule.color-profile",
        CssFeatureKind::Rule,
        "@color-profile",
        I_COLOR5_20260908,
        "#at-profile",
    ),
    CssFeatureMetadata::complete(
        "interop.descriptor.color-profile.src",
        CssFeatureKind::Descriptor,
        "src in @color-profile",
        I_COLOR5_20260908,
        "#descdef-color-profile-src",
    ),
    CssFeatureMetadata::complete(
        "interop.descriptor.color-profile.rendering-intent",
        CssFeatureKind::Descriptor,
        "rendering-intent in @color-profile",
        I_COLOR5_20260908,
        "#descdef-color-profile-rendering-intent",
    ),
    CssFeatureMetadata::complete(
        "interop.descriptor.color-profile.components",
        CssFeatureKind::Descriptor,
        "components in @color-profile",
        I_COLOR5_20260908,
        "#descdef-color-profile-components",
    ),
    CssFeatureMetadata::complete(
        "later.rule.font-palette-values",
        CssFeatureKind::Rule,
        "@font-palette-values",
        I_FONTS4_20260907,
        "#font-palette-values",
    ),
    CssFeatureMetadata::complete(
        "later.descriptor.font-palette-values.font-family",
        CssFeatureKind::Descriptor,
        "font-family in @font-palette-values",
        I_FONTS4_20260907,
        "#font-family-2-desc",
    ),
    CssFeatureMetadata::complete(
        "later.descriptor.font-palette-values.base-palette",
        CssFeatureKind::Descriptor,
        "base-palette in @font-palette-values",
        I_FONTS4_20260907,
        "#base-palette-desc",
    ),
    CssFeatureMetadata::complete(
        "later.descriptor.font-palette-values.override-colors",
        CssFeatureKind::Descriptor,
        "override-colors in @font-palette-values",
        I_FONTS4_20260907,
        "#override-color",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-family",
        CssFeatureKind::Descriptor,
        "font-family in @font-face",
        I_FONTS4_20260907,
        "#font-family-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.src",
        CssFeatureKind::Descriptor,
        "src in @font-face",
        I_FONTS4_20260907,
        "#font-face-src-parsing",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-weight",
        CssFeatureKind::Descriptor,
        "font-weight in @font-face",
        I_FONTS4_20260907,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-style",
        CssFeatureKind::Descriptor,
        "font-style in @font-face",
        I_FONTS4_20260907,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "baseline.descriptor.font-stretch",
        CssFeatureKind::Descriptor,
        "font-width in @font-face",
        I_FONTS4_20260907,
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
        I_FONTS4_20260907,
        "#font-rend-desc",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.font-variation-settings",
        CssFeatureKind::Descriptor,
        "font-variation-settings in @font-face",
        I_FONTS4_20260907,
        "#descdef-font-face-font-variation-settings",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.font-named-instance",
        CssFeatureKind::Descriptor,
        "font-named-instance in @font-face",
        I_FONTS4_20260907,
        "#font-named-instance",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.font-language-override",
        CssFeatureKind::Descriptor,
        "font-language-override in @font-face",
        I_FONTS4_20260907,
        "#font-language-override-desc",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.ascent-override",
        CssFeatureKind::Descriptor,
        "ascent-override in @font-face",
        I_FONTS4_20260907,
        "#descdef-font-face-ascent-override",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.descent-override",
        CssFeatureKind::Descriptor,
        "descent-override in @font-face",
        I_FONTS4_20260907,
        "#descdef-font-face-descent-override",
    ),
    CssFeatureMetadata::complete(
        "official.descriptor.line-gap-override",
        CssFeatureKind::Descriptor,
        "line-gap-override in @font-face",
        I_FONTS4_20260907,
        "#descdef-font-face-line-gap-override",
    ),
    CssFeatureMetadata::complete(
        "official.value.font-source",
        CssFeatureKind::Value,
        "@font-face source list",
        I_FONTS4_20260907,
        "#font-face-src-parsing",
    ),
    CssFeatureMetadata::complete(
        "official.value.opentype-tag",
        CssFeatureKind::Value,
        "OpenType feature tag",
        I_FONTS4_20260907,
        "#font-rend-desc",
    ),
    CssFeatureMetadata::complete(
        "ext.descriptor.font-weight-range",
        CssFeatureKind::Descriptor,
        "font-weight ranges in @font-face",
        I_FONTS4_20260907,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "ext.descriptor.font-style-oblique-range",
        CssFeatureKind::Descriptor,
        "font-style oblique ranges in @font-face",
        I_FONTS4_20260907,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "ext.descriptor.font-stretch-range",
        CssFeatureKind::Descriptor,
        "ordinary font-width ranges in @font-face",
        I_FONTS4_20260907,
        "#font-prop-desc",
    ),
    CssFeatureMetadata::complete(
        "ext.value.font-source-modern-hints",
        CssFeatureKind::Value,
        "format() keyword and tech() font-source hints",
        I_FONTS4_20260907,
        "#font-face-src-parsing",
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
        ":scope, :focus-visible, :focus-within, :required, :optional, :valid, :invalid, :placeholder-shown, :default, :indeterminate, :read-only, :read-write, :in-range, :out-of-range, :modal, :fullscreen, :popover-open, :defined, :any-link, :playing, :paused, :seeking, :buffering, :stalled, :muted, :volume-locked, :open, :picture-in-picture, :unchecked, :user-valid, :user-invalid",
        I_SELECTORS4,
        "#useraction-pseudos,#input-pseudos,#resource-pseudos,#display-state-pseudos,#the-defined-pseudo,#the-any-link-pseudo,#video-state,#media-loading-state,#sound-state,#open-state,#pip-state,#checked,#user-pseudos",
        "The exact extension-state pseudo-class spelling group is supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.selector.extension-functional",
        CssFeatureKind::Selector,
        ":is(), :where(), complex :not(), :has(), and nth-child of lists",
        I_SELECTORS4,
        "#matches,#zero-matches,#relational,#negation,#the-nth-child-pseudo",
        "The exact extension-functional pseudo-class spelling group is supported.",
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
    CssFeatureMetadata::partial(
        "baseline.selector.nesting",
        CssFeatureKind::Selector,
        "nesting &, scoped selector anchors, and contextual relative selectors",
        I_NESTING1,
        "#nest-selector,#syntax-examples,#nesting-at-scope",
        "Nesting &, counted scope anchors, scoped relative selectors, and contextual relative scope boundaries are supported.",
        SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "baseline.media.type",
        CssFeatureKind::MediaQuery,
        "all, aural, braille, embossed, handheld, print, projection, screen, speech, tty, tv",
        X_MEDIA5,
        "#media-types",
    ),
    CssFeatureMetadata::partial(
        "baseline.container.condition",
        CssFeatureKind::ContainerQuery,
        "and/or/not, size features, authored style and scroll-state queries, and general-enclosed",
        X_CONDITIONAL5,
        "#container-rule",
        "Recursive container, style and scroll-state logic, supported-property and custom-property boolean/plain style features, broad authored style ranges, four discrete scroll-state features with whole-value pending substitution, and original-component general-enclosed operands are retained; query evaluation and scroll snapshots belong downstream.",
        QUERY_REMAINDER,
    ),
    CssFeatureMetadata::partial(
        "baseline.container.size-feature",
        CssFeatureKind::ContainerQuery,
        "width, height, inline-size, block-size, aspect-ratio, orientation and applicable min-/max- names",
        X_CONDITIONAL5,
        "#size-container",
        "The six authored size features retain boolean, plain, prefixed and directional range forms, exact symbolic numeric values, checked var() operands and the scoped tree-counting functions; orientation remains discrete.",
        QUERY_REMAINDER,
    ),
    property_row!(All, O_CASCADE4, partial),
    property_row!(Display, S_DISPLAY3, complete),
    property_row!(BoxSizing, I_SIZING3_20260904, complete),
    property_row!(Position, I_POSITION3, complete),
    property_row!(Direction, O_WRITING3, complete),
    property_row!(Overflow, X_OVERFLOW3, complete),
    property_row!(OverflowX, X_OVERFLOW3, complete),
    property_row!(OverflowY, X_OVERFLOW3, complete),
    property_row!(OverflowBlock, X_OVERFLOW3, complete),
    property_row!(OverflowInline, X_OVERFLOW3, complete),
    property_row!(OverflowClipMargin, X_OVERFLOW3, complete),
    property_row!(ViewTransitionName, S_VIEWTRANSITIONS1, complete),
    property_row!(WillChange, I_WILLCHANGE1, complete),
    property_row!(OverflowAnchor, I_SCROLLANCHORING1, complete),
    property_row!(ScrollBehavior, X_OVERFLOW3, complete),
    property_row!(ScrollbarGutter, X_OVERFLOW3, complete),
    property_row!(FlexDirection, O_FLEXBOX1, complete),
    property_row!(FlexFlow, O_FLEXBOX1, complete),
    property_row!(FlexWrap, O_FLEXBOX1, complete),
    property_row!(ContainerName, X_CONDITIONAL5, complete),
    property_row!(ContainerType, X_CONDITIONAL5, complete),
    property_row!(Container, X_CONDITIONAL5, complete),
    property_row!(Float, I_LOGICAL1_20251204, complete, "#float-clear"),
    property_row!(Clear, I_LOGICAL1_20251204, complete, "#float-clear"),
    property_row!(AlignContent, S_ALIGN3, complete),
    property_row!(JustifyContent, S_ALIGN3, complete),
    property_row!(AlignItems, S_ALIGN3, complete),
    property_row!(AlignSelf, S_ALIGN3, complete),
    property_row!(JustifyItems, S_ALIGN3, complete),
    property_row!(JustifySelf, S_ALIGN3, complete),
    property_row!(PlaceContent, S_ALIGN3, complete),
    property_row!(PlaceItems, S_ALIGN3, complete),
    property_row!(PlaceSelf, S_ALIGN3, complete),
    property_row!(Visibility, S_DISPLAY3, complete),
    property_row!(VoiceDuration, S_SPEECH1, complete),
    property_row!(VoiceBalance, S_SPEECH1, complete),
    property_row!(VoiceVolume, S_SPEECH1, complete),
    property_row!(VoicePitch, S_SPEECH1, complete),
    property_row!(VoiceRange, S_SPEECH1, complete),
    property_row!(VoiceRate, S_SPEECH1, complete),
    property_row!(VoiceFamily, S_SPEECH1, complete),
    property_row!(VoiceStress, S_SPEECH1, complete),
    property_row!(Speak, S_SPEECH1, complete),
    property_row!(SpeakAs, S_SPEECH1, complete),
    property_row!(PauseBefore, S_SPEECH1, complete),
    property_row!(PauseAfter, S_SPEECH1, complete),
    property_row!(Pause, S_SPEECH1, complete),
    property_row!(RestBefore, S_SPEECH1, complete),
    property_row!(RestAfter, S_SPEECH1, complete),
    property_row!(Rest, S_SPEECH1, complete),
    property_row!(CueBefore, S_SPEECH1, complete),
    property_row!(CueAfter, S_SPEECH1, complete),
    property_row!(Cue, S_SPEECH1, complete),
    property_row!(Content, X_CONTENT3, complete),
    property_row!(StringSet, X_CONTENT3, complete),
    property_row!(BookmarkLevel, X_CONTENT3, complete),
    property_row!(BookmarkLabel, X_CONTENT3, complete),
    property_row!(BookmarkState, X_CONTENT3, complete),
    property_row!(ContentVisibility, I_CONTAIN2, complete),
    property_row!(ListStyleType, I_LISTS3, complete),
    property_row!(ListStylePosition, I_LISTS3, complete),
    property_row!(ListStyleImage, I_LISTS3, complete),
    property_row!(ListStyle, I_LISTS3, complete),
    property_row!(MarkerSide, I_LISTS3, complete),
    property_row!(CounterReset, I_LISTS3, complete),
    property_row!(CounterIncrement, I_LISTS3, complete),
    property_row!(CounterSet, I_LISTS3, complete),
    property_row!(Width, I_SIZING3_20260904, complete),
    property_row!(Height, I_SIZING3_20260904, complete),
    property_row!(InlineSize, I_SIZING3_20260904, complete),
    property_row!(BlockSize, I_SIZING3_20260904, complete),
    property_row!(MinWidth, I_SIZING3_20260904, complete),
    property_row!(MinHeight, I_SIZING3_20260904, complete),
    property_row!(MinInlineSize, I_SIZING3_20260904, complete),
    property_row!(MinBlockSize, I_SIZING3_20260904, complete),
    property_row!(MaxWidth, I_SIZING3_20260904, complete),
    property_row!(MaxHeight, I_SIZING3_20260904, complete),
    property_row!(MaxInlineSize, I_SIZING3_20260904, complete),
    property_row!(MaxBlockSize, I_SIZING3_20260904, complete),
    property_row!(Size, X_SIZING4_20260904, complete),
    property_row!(MinSize, X_SIZING4_20260904, complete),
    property_row!(MaxSize, X_SIZING4_20260904, complete),
    property_row!(FrameSizing, X_SIZING4_20260904, complete),
    property_row!(MinIntrinsicSizing, X_SIZING4_20260904, complete),
    property_row!(ContainIntrinsicWidth, X_SIZING4_20260904, complete),
    property_row!(ContainIntrinsicHeight, X_SIZING4_20260904, complete),
    property_row!(ContainIntrinsicInlineSize, X_SIZING4_20260904, complete),
    property_row!(ContainIntrinsicBlockSize, X_SIZING4_20260904, complete),
    property_row!(ContainIntrinsicSize, X_SIZING4_20260904, complete),
    property_row!(FlexBasis, O_FLEXBOX1, complete),
    property_row!(Gap, S_ALIGN3, complete),
    property_row!(RowGap, S_ALIGN3, complete),
    property_row!(ColumnGap, S_ALIGN3, complete),
    property_row!(ColumnCount, O_MULTICOL1, complete),
    property_row!(ColumnFill, O_MULTICOL1, complete),
    property_row!(ColumnRule, O_MULTICOL1, complete),
    property_row!(ColumnRuleColor, O_MULTICOL1, complete),
    property_row!(ColumnRuleStyle, O_MULTICOL1, complete),
    property_row!(ColumnRuleWidth, O_MULTICOL1, complete),
    property_row!(ColumnSpan, O_MULTICOL1, complete),
    property_row!(ColumnWidth, O_MULTICOL1, complete),
    property_row!(Columns, O_MULTICOL1, complete),
    property_row!(ItemDirection, X_GRID3_20260121, complete),
    property_row!(ItemWrap, X_GRID3_20260121, complete),
    property_row!(ItemPack, X_GRID3_20260121, complete),
    property_row!(ItemFlow, X_GRID3_20260121, complete),
    property_row!(FlowTolerance, X_GRID3_20260121, complete),
    property_row!(GridTemplateRows, R_GRID2, complete),
    property_row!(GridTemplateColumns, R_GRID2, complete),
    property_row!(GridTemplateAreas, R_GRID2, complete),
    property_row!(GridTemplate, R_GRID2, complete),
    property_row!(GridAutoRows, R_GRID2, complete),
    property_row!(GridAutoColumns, R_GRID2, complete),
    property_row!(
        GridAutoFlow,
        R_GRID2,
        partial(GRID_AUTO_FLOW_SUBSET, GRID_AUTO_FLOW_REMAINDER)
    ),
    property_row!(GridRowStart, R_GRID2, complete),
    property_row!(GridRowEnd, R_GRID2, complete),
    property_row!(GridColumnStart, R_GRID2, complete),
    property_row!(GridColumnEnd, R_GRID2, complete),
    property_row!(GridRow, R_GRID2, complete),
    property_row!(GridColumn, R_GRID2, complete),
    property_row!(GridArea, R_GRID2, complete),
    property_row!(Grid, R_GRID2, complete),
    property_row!(FontSize, I_FONTS4_20260907, complete),
    property_row!(
        LineHeight,
        O_CSS2,
        complete,
        "visudet.html#propdef-line-height"
    ),
    property_row!(WritingMode, S_WRITING4, complete),
    property_row!(TextAlign, X_TEXT4, complete),
    property_row!(TextAlignAll, X_TEXT4, complete),
    property_row!(TextAlignLast, X_TEXT4, complete),
    property_row!(TextIndent, X_TEXT4, complete),
    property_row!(
        VerticalAlign,
        O_CSS2,
        complete,
        "visudet.html#propdef-vertical-align"
    ),
    property_row!(FontFamily, I_FONTS4_20260907, complete),
    property_row!(Font, I_FONTS4_20260907, complete),
    property_row!(FontWeight, I_FONTS4_20260907, complete),
    property_row!(FontStyle, I_FONTS4_20260907, complete),
    property_row!(FontWidth, I_FONTS4_20260907, complete),
    property_row!(FontVariant, I_FONTS4_20260907, complete),
    property_row!(FontVariantAlternates, I_FONTS4_20260907, complete),
    property_row!(FontVariantCaps, I_FONTS4_20260907, complete),
    property_row!(FontVariantEastAsian, I_FONTS4_20260907, complete),
    property_row!(FontVariantEmoji, I_FONTS4_20260907, complete),
    property_row!(FontVariantLigatures, I_FONTS4_20260907, complete),
    property_row!(FontVariantNumeric, I_FONTS4_20260907, complete),
    property_row!(FontVariantPosition, I_FONTS4_20260907, complete),
    property_row!(FontFeatureSettings, I_FONTS4_20260907, complete),
    property_row!(FontKerning, I_FONTS4_20260907, complete),
    property_row!(FontLanguageOverride, I_FONTS4_20260907, complete),
    property_row!(FontOpticalSizing, I_FONTS4_20260907, complete),
    property_row!(FontSizeAdjust, I_FONTS4_20260907, complete),
    property_row!(FontVariationSettings, I_FONTS4_20260907, complete),
    property_row!(FontSynthesis, I_FONTS4_20260907, complete),
    property_row!(FontSynthesisWeight, I_FONTS4_20260907, complete),
    property_row!(FontSynthesisStyle, I_FONTS4_20260907, complete),
    property_row!(FontSynthesisSmallCaps, I_FONTS4_20260907, complete),
    property_row!(FontSynthesisPosition, I_FONTS4_20260907, complete),
    property_row!(FontPalette, I_FONTS4_20260907, complete),
    CssFeatureMetadata::complete(
        "ext.property.font-weight-range",
        CssFeatureKind::Property,
        "font-weight numeric range",
        I_FONTS4_20260907,
        "#font-weight-absolute-values",
    ),
    property_row!(LetterSpacing, X_TEXT4, complete),
    property_row!(Hyphens, X_TEXT4, complete),
    property_row!(HyphenateCharacter, X_TEXT4, complete),
    property_row!(HyphenateLimitZone, X_TEXT4, complete),
    property_row!(HyphenateLimitChars, X_TEXT4, complete),
    property_row!(HyphenateLimitLines, X_TEXT4, complete),
    property_row!(HyphenateLimitLast, X_TEXT4, complete),
    property_row!(TextJustify, X_TEXT4, complete),
    property_row!(TextGroupAlign, X_TEXT4, complete),
    property_row!(LinePadding, X_TEXT4, complete),
    property_row!(TextAutospace, X_TEXT4, complete),
    property_row!(TextSpacingTrim, X_TEXT4, complete),
    property_row!(TextBoxTrim, F_INLINE3_20241218, complete),
    property_row!(TextBoxEdge, F_INLINE3_20241218, complete),
    property_row!(TextBox, F_INLINE3_20241218, complete),
    property_row!(TextSpacing, X_TEXT4, complete),
    property_row!(HangingPunctuation, X_TEXT4, complete),
    property_row!(WrapInside, X_TEXT4, complete),
    property_row!(WrapBefore, X_TEXT4, complete),
    property_row!(WrapAfter, X_TEXT4, complete),
    property_row!(LineBreak, X_TEXT4, complete),
    property_row!(WordSpaceTransform, X_TEXT4, complete),
    property_row!(TabSize, X_TEXT4, complete),
    property_row!(TextWrapMode, X_TEXT4, complete),
    property_row!(TextWrapStyle, X_TEXT4, complete),
    property_row!(WhiteSpaceCollapse, X_TEXT4, complete),
    property_row!(WhiteSpaceTrim, X_TEXT4, complete),
    property_row!(TextWrap, X_TEXT4, complete),
    property_row!(WhiteSpace, X_TEXT4, complete),
    property_row!(WordBreak, X_TEXT4, complete),
    property_row!(OverflowWrap, S_TEXT3, complete),
    property_row!(TextOverflow, X_OVERFLOW3, complete),
    property_row!(TextDecoration, X_TEXTDECOR4, complete),
    property_row!(TextDecorationLine, X_TEXTDECOR4, complete),
    property_row!(TextDecorationColor, X_TEXTDECOR4, complete),
    property_row!(TextDecorationStyle, X_TEXTDECOR4, complete),
    property_row!(TextDecorationThickness, X_TEXTDECOR4, complete),
    property_row!(TextUnderlinePosition, X_TEXTDECOR4, complete),
    property_row!(TextUnderlineOffset, X_TEXTDECOR4, complete),
    property_row!(TextDecorationSkip, X_TEXTDECOR4, complete),
    property_row!(TextDecorationSkipSelf, X_TEXTDECOR4, complete),
    property_row!(TextDecorationSkipBox, X_TEXTDECOR4, complete),
    property_row!(TextDecorationSkipInset, X_TEXTDECOR4, complete),
    property_row!(TextDecorationSkipSpaces, X_TEXTDECOR4, complete),
    property_row!(TextDecorationSkipInk, X_TEXTDECOR4, complete),
    property_row!(TextEmphasis, X_TEXTDECOR4, complete),
    property_row!(TextEmphasisStyle, X_TEXTDECOR4, complete),
    property_row!(TextEmphasisColor, X_TEXTDECOR4, complete),
    property_row!(TextEmphasisPosition, X_TEXTDECOR4, complete),
    property_row!(TextEmphasisSkip, X_TEXTDECOR4, complete),
    property_row!(TextShadow, X_TEXTDECOR4, complete),
    property_row!(TextTransform, X_TEXT4, complete),
    property_row!(Inset, I_POSITION3, complete),
    property_row!(Top, I_POSITION3, complete),
    property_row!(Right, I_POSITION3, complete),
    property_row!(Bottom, I_POSITION3, complete),
    property_row!(Left, I_POSITION3, complete),
    property_row!(InsetBlockStart, I_POSITION3, complete),
    property_row!(InsetBlockEnd, I_POSITION3, complete),
    property_row!(InsetInlineStart, I_POSITION3, complete),
    property_row!(InsetInlineEnd, I_POSITION3, complete),
    property_row!(InsetBlock, I_POSITION3, complete),
    property_row!(InsetInline, I_POSITION3, complete),
    property_row!(ZIndex, O_CSS2, complete, "visuren.html#propdef-z-index"),
    property_row!(BoxDecorationBreak, S_BREAK3, complete),
    property_row!(Margin, O_BOX3, complete),
    property_row!(MarginTop, O_BOX3, complete),
    property_row!(MarginRight, O_BOX3, complete),
    property_row!(MarginBottom, O_BOX3, complete),
    property_row!(MarginLeft, O_BOX3, complete),
    property_row!(MarginBlockStart, I_LOGICAL1_20251204, complete),
    property_row!(MarginBlockEnd, I_LOGICAL1_20251204, complete),
    property_row!(MarginInlineStart, I_LOGICAL1_20251204, complete),
    property_row!(MarginInlineEnd, I_LOGICAL1_20251204, complete),
    property_row!(MarginBlock, I_LOGICAL1_20251204, complete),
    property_row!(MarginInline, I_LOGICAL1_20251204, complete),
    property_row!(Padding, O_BOX3, complete),
    property_row!(PaddingTop, O_BOX3, complete),
    property_row!(PaddingRight, O_BOX3, complete),
    property_row!(PaddingBottom, O_BOX3, complete),
    property_row!(PaddingLeft, O_BOX3, complete),
    property_row!(PaddingBlockStart, I_LOGICAL1_20251204, complete),
    property_row!(PaddingBlockEnd, I_LOGICAL1_20251204, complete),
    property_row!(PaddingInlineStart, I_LOGICAL1_20251204, complete),
    property_row!(PaddingInlineEnd, I_LOGICAL1_20251204, complete),
    property_row!(PaddingBlock, I_LOGICAL1_20251204, complete),
    property_row!(PaddingInline, I_LOGICAL1_20251204, complete),
    property_row!(Border, O_BACKGROUNDS3, complete),
    property_row!(BorderTop, O_BACKGROUNDS3, complete),
    property_row!(BorderRight, O_BACKGROUNDS3, complete),
    property_row!(BorderBottom, O_BACKGROUNDS3, complete),
    property_row!(BorderLeft, O_BACKGROUNDS3, complete),
    property_row!(BorderBlockStart, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockEnd, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineStart, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineEnd, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlock, I_LOGICAL1_20251204, complete),
    property_row!(BorderInline, I_LOGICAL1_20251204, complete),
    property_row!(BorderWidth, O_BACKGROUNDS3, complete),
    property_row!(BorderTopWidth, O_BACKGROUNDS3, complete),
    property_row!(BorderRightWidth, O_BACKGROUNDS3, complete),
    property_row!(BorderBottomWidth, O_BACKGROUNDS3, complete),
    property_row!(BorderLeftWidth, O_BACKGROUNDS3, complete),
    property_row!(BorderBlockStartWidth, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockEndWidth, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineStartWidth, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineEndWidth, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockWidth, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineWidth, I_LOGICAL1_20251204, complete),
    CssFeatureMetadata::complete(
        "official.value.border-width-logical-values",
        CssFeatureKind::Value,
        "border-width logical switch",
        I_LOGICAL1_20251204,
        "#logical-shorthand-keyword",
    ),
    property_row!(BorderImage, O_BACKGROUNDS3, complete),
    property_row!(BorderImageOutset, O_BACKGROUNDS3, complete),
    property_row!(BorderImageRepeat, O_BACKGROUNDS3, complete),
    property_row!(BorderImageSlice, O_BACKGROUNDS3, complete),
    property_row!(BorderImageSource, O_BACKGROUNDS3, complete),
    property_row!(BorderImageWidth, O_BACKGROUNDS3, complete),
    property_row!(Color, O_COLOR4, complete),
    property_row!(Background, O_BACKGROUNDS3, complete),
    property_row!(BackgroundColor, O_BACKGROUNDS3, complete),
    property_row!(BorderColor, O_BACKGROUNDS3, complete),
    property_row!(BorderTopColor, O_BACKGROUNDS3, complete),
    property_row!(BorderRightColor, O_BACKGROUNDS3, complete),
    property_row!(BorderBottomColor, O_BACKGROUNDS3, complete),
    property_row!(BorderLeftColor, O_BACKGROUNDS3, complete),
    property_row!(BorderBlockStartColor, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockEndColor, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineStartColor, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineEndColor, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockColor, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineColor, I_LOGICAL1_20251204, complete),
    property_row!(BackgroundImage, O_BACKGROUNDS3, complete),
    property_row!(BackgroundPosition, O_BACKGROUNDS3, complete),
    property_row!(ObjectPosition, O_IMAGES3, complete),
    property_row!(ImageOrientation, O_IMAGES3, complete),
    property_row!(ImageRendering, O_IMAGES3, complete),
    property_row!(ObjectFit, O_IMAGES3, complete),
    property_row!(BackgroundSize, O_BACKGROUNDS3, complete),
    property_row!(BackgroundRepeat, O_BACKGROUNDS3, complete),
    property_row!(BackgroundOrigin, O_BACKGROUNDS3, complete),
    property_row!(BackgroundClip, O_BACKGROUNDS3, complete),
    property_row!(BackgroundAttachment, O_BACKGROUNDS3, complete),
    property_row!(BorderStyle, O_BACKGROUNDS3, complete),
    property_row!(BorderTopStyle, O_BACKGROUNDS3, complete),
    property_row!(BorderRightStyle, O_BACKGROUNDS3, complete),
    property_row!(BorderBottomStyle, O_BACKGROUNDS3, complete),
    property_row!(BorderLeftStyle, O_BACKGROUNDS3, complete),
    property_row!(BorderBlockStartStyle, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockEndStyle, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineStartStyle, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineEndStyle, I_LOGICAL1_20251204, complete),
    property_row!(BorderBlockStyle, I_LOGICAL1_20251204, complete),
    property_row!(BorderInlineStyle, I_LOGICAL1_20251204, complete),
    CssFeatureMetadata::complete(
        "official.value.border-style-logical-values",
        CssFeatureKind::Value,
        "border-style logical switch",
        I_LOGICAL1_20251204,
        "#logical-shorthand-keyword",
    ),
    CssFeatureMetadata::complete(
        "official.value.border-color-logical-values",
        CssFeatureKind::Value,
        "border-color logical switch",
        I_LOGICAL1_20251204,
        "#logical-shorthand-keyword",
    ),
    property_row!(BorderRadius, O_BACKGROUNDS3, complete),
    property_row!(BorderTopLeftRadius, O_BACKGROUNDS3, complete),
    property_row!(BorderTopRightRadius, O_BACKGROUNDS3, complete),
    property_row!(BorderBottomRightRadius, O_BACKGROUNDS3, complete),
    property_row!(BorderBottomLeftRadius, O_BACKGROUNDS3, complete),
    property_row!(BorderStartStartRadius, I_LOGICAL1_20251204, complete),
    property_row!(BorderStartEndRadius, I_LOGICAL1_20251204, complete),
    property_row!(BorderEndStartRadius, I_LOGICAL1_20251204, complete),
    property_row!(BorderEndEndRadius, I_LOGICAL1_20251204, complete),
    property_row!(BoxShadow, O_BACKGROUNDS3, complete),
    property_row!(Opacity, O_COLOR4, complete),
    property_row!(FlexGrow, O_FLEXBOX1, complete),
    property_row!(FlexShrink, O_FLEXBOX1, complete),
    property_row!(Order, S_DISPLAY3, complete),
    property_row!(Flex, O_FLEXBOX1, complete),
    property_row!(AspectRatio, X_SIZING4_20260904, complete),
    property_row!(ScrollbarWidth, R_SCROLLBARS1, complete),
    property_row!(ScrollbarColor, R_SCROLLBARS1, complete),
    property_row!(ColorScheme, R_COLORADJUST1, complete),
    property_row!(ForcedColorAdjust, R_COLORADJUST1, complete),
    property_row!(PrintColorAdjust, R_COLORADJUST1, complete),
    property_row!(ColorAdjust, R_COLORADJUST1, complete),
    property_row!(Cursor, X_UI4, complete),
    property_row!(PointerEvents, X_UI4, complete),
    property_row!(UserSelect, X_UI4, complete),
    property_row!(Outline, X_UI4, complete),
    property_row!(OutlineColor, X_UI4, complete),
    property_row!(OutlineStyle, X_UI4, complete),
    property_row!(OutlineWidth, X_UI4, complete),
    property_row!(Transform, O_TRANSFORMS1, complete),
    property_row!(TransformOrigin, O_TRANSFORMS1, complete),
    property_row!(Translate, I_TRANSFORMS2, partial),
    property_row!(Rotate, I_TRANSFORMS2, partial),
    property_row!(Scale, I_TRANSFORMS2, partial),
    property_row!(Perspective, I_TRANSFORMS2, partial),
    property_row!(PerspectiveOrigin, I_TRANSFORMS2, partial),
    property_row!(TransformStyle, I_TRANSFORMS2, partial),
    property_row!(BackfaceVisibility, I_TRANSFORMS2, partial),
    property_row!(Filter, I_FILTER1, complete),
    property_row!(FloodColor, I_FILTER1, complete, "#FloodColorProperty"),
    property_row!(FloodOpacity, I_FILTER1, complete, "#FloodOpacityProperty"),
    property_row!(LightingColor, I_FILTER1, complete, "#LightingColorProperty"),
    property_row!(
        ColorInterpolationFilters,
        I_FILTER1,
        complete,
        "#ColorInterpolationFiltersProperty"
    ),
    property_row!(
        BackdropFilter,
        X_BACKDROP_FILTER,
        complete,
        "#BackdropFilterProperty"
    ),
    property_row!(ClipPath, S_MASKING1, complete),
    property_row!(ShapeOutside, S_SHAPES1, complete),
    property_row!(OffsetPosition, I_MOTION1, complete),
    property_row!(OffsetPath, I_MOTION1, complete),
    property_row!(OffsetDistance, I_MOTION1, complete),
    property_row!(OffsetRotate, I_MOTION1, complete),
    property_row!(OffsetAnchor, I_MOTION1, complete),
    property_row!(Offset, I_MOTION1, complete),
    property_row!(ShapeImageThreshold, S_SHAPES1, complete),
    property_row!(ShapeMargin, S_SHAPES1, complete),
    property_row!(Mask, S_MASKING1, complete),
    property_row!(MaskImage, S_MASKING1, complete),
    property_row!(MaskSize, S_MASKING1, complete),
    property_row!(MaskPosition, S_MASKING1, complete),
    property_row!(MaskRepeat, S_MASKING1, complete),
    property_row!(MaskMode, S_MASKING1, complete),
    property_row!(MaskOrigin, S_MASKING1, complete),
    property_row!(MaskClip, S_MASKING1, complete),
    property_row!(MaskComposite, S_MASKING1, complete),
    property_row!(MaskBorder, S_MASKING1, complete),
    property_row!(MaskBorderSource, S_MASKING1, complete),
    property_row!(MaskBorderSlice, S_MASKING1, complete),
    property_row!(MaskBorderWidth, S_MASKING1, complete),
    property_row!(MaskBorderOutset, S_MASKING1, complete),
    property_row!(MaskBorderRepeat, S_MASKING1, complete),
    property_row!(MaskBorderMode, S_MASKING1, complete),
    property_row!(ClipRule, S_MASKING1, complete),
    property_row!(MaskType, S_MASKING1, complete),
    property_row!(TransitionProperty, I_TRANSITIONS1, complete),
    property_row!(TransitionDuration, I_TRANSITIONS1, complete),
    property_row!(TransitionDelay, I_TRANSITIONS1, complete),
    property_row!(TransitionTimingFunction, I_TRANSITIONS1, complete),
    property_row!(Transition, I_TRANSITIONS1, complete),
    property_row!(AnimationName, I_ANIMATIONS1, complete),
    property_row!(AnimationDuration, I_ANIMATIONS1, complete),
    property_row!(AnimationDelay, I_ANIMATIONS1, complete),
    property_row!(AnimationTimingFunction, I_ANIMATIONS1, complete),
    property_row!(AnimationIterationCount, I_ANIMATIONS1, complete),
    property_row!(AnimationDirection, I_ANIMATIONS1, complete),
    property_row!(AnimationFillMode, I_ANIMATIONS1, complete),
    property_row!(AnimationPlayState, I_ANIMATIONS1, complete),
    property_row!(Animation, I_ANIMATIONS1, complete),
    property_row!(
        BorderCollapse,
        O_CSS2,
        complete,
        "tables.html#propdef-border-collapse"
    ),
    property_row!(
        BorderSpacing,
        O_CSS2,
        complete,
        "tables.html#propdef-border-spacing"
    ),
    property_row!(
        CaptionSide,
        O_CSS2,
        complete,
        "tables.html#propdef-caption-side"
    ),
    property_row!(Clip, S_MASKING1, complete),
    property_row!(
        EmptyCells,
        O_CSS2,
        complete,
        "tables.html#propdef-empty-cells"
    ),
    property_row!(Orphans, S_BREAK3, complete),
    property_row!(BreakAfter, S_BREAK3, complete),
    property_row!(BreakBefore, S_BREAK3, complete),
    property_row!(BreakInside, S_BREAK3, complete),
    property_row!(Quotes, X_CONTENT3, complete),
    property_row!(
        TableLayout,
        O_CSS2,
        complete,
        "tables.html#propdef-table-layout"
    ),
    property_row!(ScrollSnapType, R_SCROLLSNAP1, complete),
    property_row!(ScrollSnapAlign, R_SCROLLSNAP1, complete),
    property_row!(ScrollSnapStop, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingTop, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingRight, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingBottom, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingLeft, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingBlockStart, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingBlockEnd, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingInlineStart, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingInlineEnd, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginTop, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginRight, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginBottom, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginLeft, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginBlockStart, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginBlockEnd, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginInlineStart, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginInlineEnd, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingBlock, R_SCROLLSNAP1, complete),
    property_row!(ScrollPaddingInline, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginBlock, R_SCROLLSNAP1, complete),
    property_row!(ScrollMarginInline, R_SCROLLSNAP1, complete),
    property_row!(ScrollPadding, R_SCROLLSNAP1, complete),
    property_row!(ScrollMargin, R_SCROLLSNAP1, complete),
    property_row!(Widows, S_BREAK3, complete),
    property_row!(WordSpacing, X_TEXT4, complete),
    property_row!(TextCombineUpright, S_WRITING4, complete),
    property_row!(TextOrientation, O_WRITING3, complete),
    property_row!(UnicodeBidi, O_WRITING3, complete),
    property_row!(NavUp, X_UI4, complete),
    property_row!(NavRight, X_UI4, complete),
    property_row!(NavDown, X_UI4, complete),
    property_row!(NavLeft, X_UI4, complete),
    property_row!(CaretAnimation, X_UI4, complete),
    property_row!(CaretShape, X_UI4, complete),
    property_row!(Caret, X_UI4, complete),
    property_row!(Interactivity, X_UI4, complete),
    property_row!(InterestDelayStart, X_UI4, complete),
    property_row!(InterestDelayEnd, X_UI4, complete),
    property_row!(InterestDelay, X_UI4, complete),
    property_row!(AccentColor, X_UI4, complete),
    property_row!(Appearance, X_UI4, complete),
    property_row!(CaretColor, X_UI4, complete),
    property_row!(OutlineOffset, X_UI4, complete),
    property_row!(Resize, X_UI4, complete),
    property_row!(Contain, I_CONTAIN2, complete),
    property_row!(TransformBox, O_TRANSFORMS1, complete),
    property_row!(BackgroundBlendMode, O_COMPOSITING1, complete),
    property_row!(Isolation, O_COMPOSITING1, complete),
    property_row!(MixBlendMode, O_COMPOSITING1, complete),
    CssFeatureMetadata::partial(
        "official.property-alias.glyph-orientation-vertical",
        CssFeatureKind::PropertyAlias,
        "glyph-orientation-vertical",
        O_WRITING3,
        "#propdef-glyph-orientation-vertical",
        "Decoded auto, integer-flag unitless 0 and 90, and exact degree-valued 0deg and 90deg (including equivalent authored degree spellings) map to text-orientation mixed, upright, and sideways.",
        "Unresolved by the selected standard: numeric-terminal spelling, including unitless integer/number distinctions and alternate lexical forms, and math applicability to finite keyword mapping (CSSWG issue 8032).",
    ),
    CssFeatureMetadata::complete(
        "official.property.page-break-before",
        CssFeatureKind::PropertyAlias,
        "page-break-before",
        S_BREAK3,
        "#page-break-properties",
    ),
    CssFeatureMetadata::complete(
        "official.property.page-break-after",
        CssFeatureKind::PropertyAlias,
        "page-break-after",
        S_BREAK3,
        "#page-break-properties",
    ),
    CssFeatureMetadata::complete(
        "official.property.page-break-inside",
        CssFeatureKind::PropertyAlias,
        "page-break-inside",
        S_BREAK3,
        "#page-break-properties",
    ),
    CssFeatureMetadata::complete(
        "official.value.page-break-logical-values",
        CssFeatureKind::Value,
        "flow-relative page-break values",
        I_LOGICAL1_20251204,
        "#page",
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
        I_SELECTORS4,
        "#the-lang-pseudo",
    ),
    CssFeatureMetadata::complete(
        "ext.selector.dir",
        CssFeatureKind::Selector,
        ":dir()",
        I_SELECTORS4,
        "#the-dir-pseudo",
    ),
    CssFeatureMetadata::complete(
        "ext.selector.autofill",
        CssFeatureKind::Selector,
        ":autofill with required -webkit-autofill alias canonicalization",
        I_SELECTORS4,
        "#selectordef-autofill,#legacy-aliasing,#compat",
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
        "authored selector prefixes and default, explicit-none, any, and named namespace constraints",
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
        "ext.pseudo-element.prefix",
        CssFeatureKind::Selector,
        "::prefix, including the defined ::first-letter::prefix child",
        X_PSEUDO4,
        "#selectordef-first-letter-prefix",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.suffix",
        CssFeatureKind::Selector,
        "::suffix, including the defined ::first-letter::suffix child",
        X_PSEUDO4,
        "#selectordef-first-letter-suffix",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.search-text",
        CssFeatureKind::Selector,
        "::search-text authored identity, independent of optional UA implementation",
        X_PSEUDO4,
        "#selectordef-search-text",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.target-text",
        CssFeatureKind::Selector,
        "::target-text",
        X_PSEUDO4,
        "#selectordef-target-text",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.spelling-error",
        CssFeatureKind::Selector,
        "::spelling-error",
        X_PSEUDO4,
        "#selectordef-spelling-error",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.grammar-error",
        CssFeatureKind::Selector,
        "::grammar-error",
        X_PSEUDO4,
        "#selectordef-grammar-error",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.highlight",
        CssFeatureKind::Selector,
        "::highlight() with exactly one checked, case-preserving custom identifier",
        X_PSEUDO4,
        "#selectordef-highlight-custom-ident",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.placeholder",
        CssFeatureKind::Selector,
        "::placeholder tree-abiding authored identity",
        X_PSEUDO4,
        "#selectordef-placeholder",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.file-selector-button",
        CssFeatureKind::Selector,
        "::file-selector-button with element-backed pseudo syntax and tree-abiding transitions",
        X_PSEUDO4,
        "#selectordef-file-selector-button,#element-backed",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.details-content",
        CssFeatureKind::Selector,
        "::details-content with element-backed pseudo syntax and tree-abiding transitions",
        X_PSEUDO4,
        "#selectordef-details-content,#element-backed",
    ),
    CssFeatureMetadata::complete(
        "ext.pseudo-element.search-text-current",
        CssFeatureKind::Selector,
        "nonfunctional :current in SearchText and element-backed suffix contexts, inherited by logical arguments; ordinary :current, current(), :past and :future remain unsupported",
        X_PSEUDO4,
        "#selectordef-search-text,#element-backed",
    ),
    CssFeatureMetadata::complete(
        "official.pseudo-element.view-transition",
        CssFeatureKind::Selector,
        "::view-transition root with generic suffix permissions and group children",
        S_VIEWTRANSITIONS1,
        "#selectordef-view-transition,#pseudo-root",
    ),
    CssFeatureMetadata::complete(
        "official.pseudo-element.view-transition-group",
        CssFeatureKind::Selector,
        "::view-transition-group(* | <custom-ident>), OnlyChild suffix and image-pair children",
        S_VIEWTRANSITIONS1,
        "#selectordef-view-transition-group,#pseudo-root",
    ),
    CssFeatureMetadata::complete(
        "official.pseudo-element.view-transition-image-pair",
        CssFeatureKind::Selector,
        "::view-transition-image-pair(* | <custom-ident>), OnlyChild suffix and old/new children",
        S_VIEWTRANSITIONS1,
        "#selectordef-view-transition-image-pair,#pseudo-root",
    ),
    CssFeatureMetadata::complete(
        "official.pseudo-element.view-transition-old",
        CssFeatureKind::Selector,
        "::view-transition-old(* | <custom-ident>) with OnlyChild suffix",
        S_VIEWTRANSITIONS1,
        "#selectordef-view-transition-old,#pseudo-root",
    ),
    CssFeatureMetadata::complete(
        "official.pseudo-element.view-transition-new",
        CssFeatureKind::Selector,
        "::view-transition-new(* | <custom-ident>) with OnlyChild suffix",
        S_VIEWTRANSITIONS1,
        "#selectordef-view-transition-new,#pseudo-root",
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
        "ext.pseudo-element.unknown-webkit",
        CssFeatureKind::Selector,
        "nonfunctional unknown -webkit- pseudo-elements with symbolic match-nothing identity and ASCII-lowercase output",
        I_SELECTORS4,
        "#compat,#unknown--webkit--pseudo-elements",
    ),
    CssFeatureMetadata::complete(
        "official.media.query-list-core",
        CssFeatureKind::MediaQuery,
        "Media Queries 5 authored query-list grammar",
        X_MEDIA5,
        "#mq-syntax",
    ),
    CssFeatureMetadata::complete(
        "ext.rule.custom-media",
        CssFeatureKind::Rule,
        "authored custom-media definitions with boolean or symbolic query-list bodies and checked construction",
        X_MEDIA5,
        "#at-ruledef-custom-media",
    ),
    CssFeatureMetadata::complete(
        "ext.media.custom-media",
        CssFeatureKind::MediaQuery,
        "symbolic custom-media references in boolean media-feature context",
        X_MEDIA5,
        "#custom-mq",
    ),
    CssFeatureMetadata::complete(
        "ext.media.condition-syntax",
        CssFeatureKind::MediaQuery,
        "not, and, or, explicit groups and general-enclosed condition syntax",
        X_MEDIA5,
        "#mq-syntax",
    ),
    CssFeatureMetadata::complete(
        "ext.media.malformed-member-never",
        CssFeatureKind::MediaQuery,
        "malformed query-list members become Never",
        X_MEDIA5,
        "#error-handling",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.width",
        CssFeatureKind::MediaQuery,
        "width, min-width, max-width, including boolean forms where permitted",
        X_MEDIA5,
        "#width",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.height",
        CssFeatureKind::MediaQuery,
        "height, min-height, max-height, including boolean forms where permitted",
        X_MEDIA5,
        "#height",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.device-width",
        CssFeatureKind::MediaQuery,
        "device-width, min-device-width, max-device-width, including boolean forms where permitted",
        X_MEDIA5,
        "#device-width",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.device-height",
        CssFeatureKind::MediaQuery,
        "device-height, min-device-height, max-device-height, including boolean forms where permitted",
        X_MEDIA5,
        "#device-height",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.aspect-ratio",
        CssFeatureKind::MediaQuery,
        "aspect-ratio, min-aspect-ratio, max-aspect-ratio, including boolean forms where permitted",
        X_MEDIA5,
        "#aspect-ratio",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.device-aspect-ratio",
        CssFeatureKind::MediaQuery,
        "device-aspect-ratio, min-device-aspect-ratio, max-device-aspect-ratio, including boolean forms where permitted",
        X_MEDIA5,
        "#device-aspect-ratio",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.resolution",
        CssFeatureKind::MediaQuery,
        "resolution, min-resolution, max-resolution with dpi and dpcm, including boolean forms where permitted",
        X_MEDIA5,
        "#resolution",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.color",
        CssFeatureKind::MediaQuery,
        "color, min-color, max-color, including boolean forms where permitted",
        X_MEDIA5,
        "#color",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.color-index",
        CssFeatureKind::MediaQuery,
        "color-index, min-color-index, max-color-index, including boolean forms where permitted",
        X_MEDIA5,
        "#color-index",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.monochrome",
        CssFeatureKind::MediaQuery,
        "monochrome, min-monochrome, max-monochrome, including boolean forms where permitted",
        X_MEDIA5,
        "#monochrome",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.scan",
        CssFeatureKind::MediaQuery,
        "scan, including its boolean form",
        X_MEDIA5,
        "#scan",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.grid",
        CssFeatureKind::MediaQuery,
        "grid, including its boolean form",
        X_MEDIA5,
        "#grid",
    ),
    CssFeatureMetadata::complete(
        "ext.media.resolution.dppx",
        CssFeatureKind::MediaQuery,
        "dppx resolution unit",
        X_MEDIA5,
        "#resolution",
    ),
    CssFeatureMetadata::complete(
        "ext.supports.general-enclosed",
        CssFeatureKind::Value,
        "general-enclosed supports condition",
        X_VALUES4,
        "css-values-4/Overview.bs#typedef-general-enclosed",
    ),
    CssFeatureMetadata::complete(
        "ext.supports.named",
        CssFeatureKind::Value,
        "named supports condition reference",
        X_CONDITIONAL5,
        "#typedef-supports-decl,#typedef-supports-condition-name",
    ),
    CssFeatureMetadata::complete(
        "ext.supports.font-tech",
        CssFeatureKind::Value,
        "font-tech() authored single-technology support predicate",
        X_CONDITIONAL5,
        "#typedef-supports-font-tech-fn",
    ),
    CssFeatureMetadata::complete(
        "ext.supports.font-format",
        CssFeatureKind::Value,
        "font-format() authored single keyword or string support predicate",
        X_CONDITIONAL5,
        "#typedef-supports-font-format-fn",
    ),
    CssFeatureMetadata::complete(
        "ext.supports.at-rule",
        CssFeatureKind::Value,
        "at-rule() authored single at-keyword support predicate",
        X_CONDITIONAL5,
        "#typedef-supports-at-rule-fn",
    ),
    CssFeatureMetadata::partial(
        "ext.supports.selector",
        CssFeatureKind::Selector,
        "selector() supports test",
        R_CONDITIONAL4,
        "#typedef-supports-selector-fn",
        SUPPORTS_SELECTOR_SUBSET,
        SUPPORTS_SELECTOR_REMAINDER,
    ),
    CssFeatureMetadata::complete(
        "ext.media.range.width",
        CssFeatureKind::MediaQuery,
        "width: signed symbolic lengths in feature-first, value-first and source-ordered chained comparisons",
        X_MEDIA5,
        "#width",
    ),
    CssFeatureMetadata::complete(
        "ext.media.range.height",
        CssFeatureKind::MediaQuery,
        "height: signed symbolic lengths in feature-first, value-first and source-ordered chained comparisons",
        X_MEDIA5,
        "#height",
    ),
    CssFeatureMetadata::complete(
        "ext.media.range.resolution",
        CssFeatureKind::MediaQuery,
        "resolution: signed symbolic resolutions and infinite in feature-first, value-first and source-ordered chained comparisons",
        X_MEDIA5,
        "#resolution",
    ),
    CssFeatureMetadata::complete(
        "ext.media.range.color",
        CssFeatureKind::MediaQuery,
        "color: signed symbolic integers in feature-first, value-first and source-ordered chained comparisons",
        X_MEDIA5,
        "#color",
    ),
    CssFeatureMetadata::complete(
        "ext.media.range.monochrome",
        CssFeatureKind::MediaQuery,
        "monochrome: signed symbolic integers in feature-first, value-first and source-ordered chained comparisons",
        X_MEDIA5,
        "#monochrome",
    ),
    CssFeatureMetadata::complete(
        "official.media.feature.orientation",
        CssFeatureKind::MediaQuery,
        "orientation",
        X_MEDIA5,
        "#orientation",
    ),
    CssFeatureMetadata::complete(
        "ext.media.hover",
        CssFeatureKind::MediaQuery,
        "hover",
        X_MEDIA5,
        "#hover",
    ),
    CssFeatureMetadata::complete(
        "ext.media.any-hover",
        CssFeatureKind::MediaQuery,
        "any-hover",
        X_MEDIA5,
        "#descdef-media-any-hover",
    ),
    CssFeatureMetadata::complete(
        "ext.media.pointer",
        CssFeatureKind::MediaQuery,
        "pointer",
        X_MEDIA5,
        "#pointer",
    ),
    CssFeatureMetadata::complete(
        "ext.media.any-pointer",
        CssFeatureKind::MediaQuery,
        "any-pointer",
        X_MEDIA5,
        "#descdef-media-any-pointer",
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
        X_MEDIA5,
        "#descdef-media-display-mode",
    ),
    CssFeatureMetadata::complete(
        "ext.media.horizontal-viewport-segments",
        CssFeatureKind::MediaQuery,
        "horizontal-viewport-segments",
        X_MEDIA5,
        "#descdef-media-horizontal-viewport-segments",
    ),
    CssFeatureMetadata::complete(
        "ext.media.vertical-viewport-segments",
        CssFeatureKind::MediaQuery,
        "vertical-viewport-segments",
        X_MEDIA5,
        "#descdef-media-vertical-viewport-segments",
    ),
    CssFeatureMetadata::complete(
        "ext.media.update",
        CssFeatureKind::MediaQuery,
        "update",
        X_MEDIA5,
        "#descdef-media-update",
    ),
    CssFeatureMetadata::complete(
        "ext.media.overflow-block",
        CssFeatureKind::MediaQuery,
        "overflow-block",
        X_MEDIA5,
        "#descdef-media-overflow-block",
    ),
    CssFeatureMetadata::complete(
        "ext.media.overflow-inline",
        CssFeatureKind::MediaQuery,
        "overflow-inline",
        X_MEDIA5,
        "#descdef-media-overflow-inline",
    ),
    CssFeatureMetadata::complete(
        "ext.media.color-gamut",
        CssFeatureKind::MediaQuery,
        "color-gamut",
        X_MEDIA5,
        "#descdef-media-color-gamut",
    ),
    CssFeatureMetadata::complete(
        "ext.media.video-color-gamut",
        CssFeatureKind::MediaQuery,
        "video-color-gamut",
        X_MEDIA5,
        "#descdef-media-video-color-gamut",
    ),
    CssFeatureMetadata::complete(
        "ext.media.dynamic-range",
        CssFeatureKind::MediaQuery,
        "dynamic-range",
        X_MEDIA5,
        "#descdef-media-dynamic-range",
    ),
    CssFeatureMetadata::complete(
        "ext.media.video-dynamic-range",
        CssFeatureKind::MediaQuery,
        "video-dynamic-range",
        X_MEDIA5,
        "#descdef-media-video-dynamic-range",
    ),
    CssFeatureMetadata::complete(
        "ext.media.environment-blending",
        CssFeatureKind::MediaQuery,
        "environment-blending",
        X_MEDIA5,
        "#descdef-media-environment-blending",
    ),
    CssFeatureMetadata::complete(
        "ext.media.inverted-colors",
        CssFeatureKind::MediaQuery,
        "inverted-colors",
        X_MEDIA5,
        "#descdef-media-inverted-colors",
    ),
    CssFeatureMetadata::complete(
        "ext.media.nav-controls",
        CssFeatureKind::MediaQuery,
        "nav-controls",
        X_MEDIA5,
        "#descdef-media-nav-controls",
    ),
    CssFeatureMetadata::complete(
        "ext.media.scripting",
        CssFeatureKind::MediaQuery,
        "scripting",
        X_MEDIA5,
        "#descdef-media-scripting",
    ),
    CssFeatureMetadata::complete(
        "ext.media.prefers-reduced-data",
        CssFeatureKind::MediaQuery,
        "prefers-reduced-data",
        X_MEDIA5,
        "#descdef-media-prefers-reduced-data",
    ),
    CssFeatureMetadata::complete(
        "interop.property.svg-glyph-orientation-vertical",
        CssFeatureKind::Property,
        "glyph-orientation-vertical",
        I_WEBKIT_SVG_GLYPH,
        "#L5350-L5367",
    ),
];

/// Returns the immutable support catalog in stable inventory order.
///
/// Every record is atomic and keeps its exact, source-specific lookup identity.
/// Properties removed by the selected standards are absent, even when historical
/// captures contain them.
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

/// Returns support metadata for a canonical property or name-equivalent alias.
///
/// Matching is ASCII-case-insensitive. Distinct legacy shorthand grammars, custom
/// properties, independent SVG definitions, and unknown spellings return `None`. For legacy grammars, use
/// [`crate::CssPropertyGrammar::from_name`], then [`feature_metadata`] with the
/// grammar's feature ID or its intrinsic metadata accessor. This lookup does not
/// parse a declaration or classify its diagnostics.
#[must_use]
pub fn property_support_metadata(name: &str) -> Option<CssPropertySupportMetadata> {
    FEATURE_CATALOG
        .iter()
        .filter(|feature| feature.kind == CssFeatureKind::Property && feature.property.is_some())
        .find(|feature| {
            feature.spelling.eq_ignore_ascii_case(name)
                || feature
                    .property_aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(name))
        })
        .map(|feature| CssPropertySupportMetadata { feature })
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
