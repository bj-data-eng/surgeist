//! Layout-private exact qualification of captured ordinary box batches.
//! Authored oracles are inputs, never copied from a completed batch.
use super::capture_authority::{CanonicalField, CurrentDomain, CurrentRequirements, Selector};
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub(crate) enum QualificationError {
    IdentityChanged {
        component: String,
    },
    MalformedDeclaration {
        reason: String,
    },
    IncompleteRepresentation {
        key: String,
    },
    IncompleteNativeOracle {
        key: String,
    },
    NativeOracleChanged {
        key: String,
        expected: String,
        actual: String,
    },
    UnsupportedObservation {
        owner: String,
        category: String,
    },
    DuplicateKey {
        key: String,
    },
    UnexpectedPass,
    DifferenceSetChanged {
        missing: Vec<String>,
        extra: Vec<String>,
        changed: Vec<String>,
    },
    ComparisonFailure {
        reason: String,
    },
}
impl std::fmt::Display for QualificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for QualificationError {}
type Result<T> = std::result::Result<T, QualificationError>;
fn malformed(reason: impl Into<String>) -> QualificationError {
    QualificationError::MalformedDeclaration {
        reason: reason.into(),
    }
}

// Narrowing is explicit: the author supplies canonical representable scalars.
// No browser tolerance is borrowed for Native authority or declared pairs.
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(transparent)]
struct Number(Scalar);
impl Number {
    fn new(value: Scalar) -> Result<Self> {
        if !value.is_finite() {
            return Err(malformed("nonfinite scalar"));
        }
        Ok(Self(if value == 0.0 { 0.0 } else { value }))
    }
}
impl<'de> Deserialize<'de> for Number {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let original = f64::deserialize(d)?;
        let value = original as Scalar;
        if !original.is_finite() || !value.is_finite() || f64::from(value) != original {
            return Err(serde::de::Error::custom(
                "expected canonical finite Layout scalar",
            ));
        }
        Ok(Self(if value == 0.0 { 0.0 } else { value }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Point {
    x: Number,
    y: Number,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Extent {
    width: Number,
    height: Number,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Edges {
    top: Number,
    right: Number,
    bottom: Number,
    left: Number,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Rect {
    origin: Point,
    size: Extent,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Interval {
    minimum: Number,
    maximum: Number,
}

macro_rules! semantic_enum {
    ($name:ident, $actual:ident, {$($variant:ident),+ $(,)?}) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
        #[serde(rename_all = "kebab-case")]
        enum $name { $($variant),+ }
        impl From<layout::$actual> for $name {
            fn from(actual: layout::$actual) -> Self {
                match actual { $(layout::$actual::$variant => Self::$variant),+ }
            }
        }
    };
}
semantic_enum!(WritingMode, WritingMode, {HorizontalTb, VerticalRl, VerticalLr, SidewaysRl, SidewaysLr});
semantic_enum!(Direction, Direction, {Ltr, Rtl});
semantic_enum!(Overflow, Overflow, {Visible, Clip, Hidden, Scroll, Auto});
semantic_enum!(Axis, PhysicalAxis, {Horizontal, Vertical});
semantic_enum!(SnapAxis, ScrollSnapAxis, {X, Y, Block, Inline, Both});
semantic_enum!(Strictness, ScrollSnapStrictness, {Proximity, Mandatory});
semantic_enum!(Align, ScrollSnapAlignValue, {None, Start, End, Center});
semantic_enum!(Stop, ScrollSnapStop, {Normal, Always});

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Flow {
    writing_mode: WritingMode,
    direction: Direction,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum Fact {
    Unknown,
    Absent,
    Available { axis: Axis, offset: Number },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Pair {
    first: Fact,
    last: Fact,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct NamedFacts {
    alphabetic: Pair,
    central: Pair,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum Snap {
    None,
    Enabled {
        axis: SnapAxis,
        strictness: Strictness,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Target {
    border_box: Rect,
    scroll_margin: Edges,
    flow_axes: Flow,
    block_align: Align,
    inline_align: Align,
    snap_stop: Stop,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ScrollFacts {
    flow_axes: Flow,
    used_overflow_x: Overflow,
    used_overflow_y: Overflow,
    border_box: Rect,
    padding_box: Rect,
    content_box: Rect,
    scrollport: Rect,
    #[serde(deserialize_with = "explicit_nullable")]
    clip_x: Option<Interval>,
    #[serde(deserialize_with = "explicit_nullable")]
    clip_y: Option<Interval>,
    scrollable_overflow: Rect,
    range_x: Interval,
    range_y: Interval,
    #[serde(deserialize_with = "explicit_nullable")]
    gutter_top: Option<Rect>,
    #[serde(deserialize_with = "explicit_nullable")]
    gutter_right: Option<Rect>,
    #[serde(deserialize_with = "explicit_nullable")]
    gutter_bottom: Option<Rect>,
    #[serde(deserialize_with = "explicit_nullable")]
    gutter_left: Option<Rect>,
    scrollbar_size: Extent,
    resolved_scroll_padding: Edges,
    optimal_viewing_region: Rect,
    scroll_snap_type: Snap,
    target: Target,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum Scroll {
    Absent,
    Present { facts: Box<ScrollFacts> },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct NodeFacts {
    source_index: usize,
    location: Point,
    size: Extent,
    content_size: Extent,
    border: Edges,
    padding: Edges,
    margin: Edges,
    named_baselines: NamedFacts,
    scroll: Scroll,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum Packet {
    Absent,
    Present { facts: Box<NodeFacts> },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum SourceRole {
    OrdinaryBox,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct NativeNode {
    path: String,
    role: SourceRole,
    children: Vec<String>,
    unrounded: Packet,
    final_output: Packet,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeOracle {
    source_derivation: String,
    nodes: Vec<NativeNode>,
}

// The two domains deliberately have distinct field tags and number types.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct ComparedField(Field);
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "domain", rename_all = "kebab-case", deny_unknown_fields)]
enum Field {
    Layout {
        path: String,
        field: LayoutField,
        browser: Number,
        native: Number,
    },
    ElementSize {
        path: String,
        field: ElementField,
        browser: u32,
        native: u32,
    },
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum LayoutField {
    X,
    Y,
    Width,
    Height,
    ScrollWidth,
    ScrollHeight,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ElementField {
    ClientWidth,
    ClientHeight,
    ScrollWidth,
    ScrollHeight,
}

impl ComparedField {
    fn key(&self) -> String {
        match &self.0 {
            Field::Layout { path, field, .. } => format!("layout:{path}:{field:?}"),
            Field::ElementSize { path, field, .. } => format!("element-size:{path}:{field:?}"),
        }
    }
    fn canonical_selector(&self) -> Option<Selector> {
        let field = match &self.0 {
            Field::Layout { field, .. } => match field {
                LayoutField::X => CanonicalField::X,
                LayoutField::Y => CanonicalField::Y,
                LayoutField::Width => CanonicalField::Width,
                LayoutField::Height => CanonicalField::Height,
                LayoutField::ScrollWidth | LayoutField::ScrollHeight => return None,
            },
            Field::ElementSize { field, .. } => match field {
                ElementField::ClientWidth => CanonicalField::ClientWidth,
                ElementField::ClientHeight => CanonicalField::ClientHeight,
                ElementField::ScrollWidth => CanonicalField::ScrollWidth,
                ElementField::ScrollHeight => CanonicalField::ScrollHeight,
            },
        };
        Some(Selector {
            path: self.path().to_owned(),
            field,
        })
    }

    fn path(&self) -> &str {
        match &self.0 {
            Field::Layout { path, .. } | Field::ElementSize { path, .. } => path,
        }
    }
    fn differs(&self) -> bool {
        match &self.0 {
            Field::Layout {
                browser, native, ..
            } => !numeric_values_match(browser.0, native.0),
            Field::ElementSize {
                browser, native, ..
            } => browser != native,
        }
    }
    pub(super) fn layout(
        path: String,
        field: NumericComparisonField,
        browser: Scalar,
        native: Scalar,
    ) -> Result<Self> {
        let field = match field {
            NumericComparisonField::X => LayoutField::X,
            NumericComparisonField::Y => LayoutField::Y,
            NumericComparisonField::Width => LayoutField::Width,
            NumericComparisonField::Height => LayoutField::Height,
            NumericComparisonField::ScrollWidth => LayoutField::ScrollWidth,
            NumericComparisonField::ScrollHeight => LayoutField::ScrollHeight,
        };
        Ok(Self(Field::Layout {
            path,
            field,
            browser: Number::new(browser)?,
            native: Number::new(native)?,
        }))
    }
    pub(super) fn element(
        path: String,
        field: ElementScrollSizeField,
        browser: u32,
        native: u32,
    ) -> Self {
        let field = match field {
            ElementScrollSizeField::ClientWidth => ElementField::ClientWidth,
            ElementScrollSizeField::ClientHeight => ElementField::ClientHeight,
            ElementScrollSizeField::ScrollWidth => ElementField::ScrollWidth,
            ElementScrollSizeField::ScrollHeight => ElementField::ScrollHeight,
        };
        Self(Field::ElementSize {
            path,
            field,
            browser,
            native,
        })
    }
}

/// Created by the inventory consumer only after report/check-corpus and exact-byte
/// validation. The identity is a closed projection, not a portable receipt.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaseIdentity {
    source: String,
    source_sha256: String,
    variant: String,
    #[serde(deserialize_with = "explicit_nullable")]
    historical_sha256: Option<String>,
    capture_sha256: String,
    record_sha256: String,
    generator: String,
    engine_version: String,
    browser_executable: String,
    browser_executable_sha256: String,
    browser_provenance: String,
    profile_sha256: String,
    source_imports_sha256: String,
    inputs: BTreeMap<String, String>,
    linked_resources: BTreeMap<String, String>,
    font_demand: FontDemand,
    policy_source_sha256: String,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum FontDemand {
    None,
    #[serde(rename = "ahem-10px-zwsp")]
    AhemZwsp,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    id: String,
    edition: String,
    source: String,
    source_sha256: String,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
enum CaptureLineage {
    Focused,
    Published,
}
impl CaptureLineage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::Published => "published",
        }
    }
}

/// The fixed Layout-owned edition association; no caller-supplied edition text.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PublishedEdition {
    CanonicalBoxDifferences,
    MeasuredGridDifferences,
}

impl PublishedEdition {
    pub(crate) fn name(self) -> &'static str {
        self.artifact_root()
            .rsplit('/')
            .next()
            .expect("fixed edition path")
    }

    pub(crate) fn artifact_root(self) -> &'static str {
        match self {
            Self::CanonicalBoxDifferences => super::capture_authority::ARTIFACT_ROOT,
            Self::MeasuredGridDifferences => {
                "tests/layout/browser_parity/captured/measured-grid-differences"
            }
        }
    }
    fn domain(self) -> CurrentDomain {
        match self {
            Self::CanonicalBoxDifferences => CurrentDomain::Layout,
            Self::MeasuredGridDifferences => CurrentDomain::LayoutAndElementSizes,
        }
    }
}

/// Select one capture lineage before comparing any observation or Native output.
/// Published authority always carries the explicit fixed edition association.
pub(crate) enum CaptureAuthority<'a> {
    Focused(&'a CaseIdentity),
    Published {
        edition: PublishedEdition,
        identity: &'a CaseIdentity,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Representation {
    #[serde(deserialize_with = "explicit_nullable")]
    published_edition: Option<PublishedEdition>,
    current: Option<super::capture_authority::CurrentRequirements>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpec {
    profile: Profile,
    identities: BTreeMap<CaptureLineage, CaseIdentity>,
    representation: Representation,
    policy: Policy,
    reason: String,
    native: NativeOracle,
    differences: Vec<ComparedField>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Profile {
    CanonicalNoTextBoxes,
    CanonicalMeasuredGrid,
}
impl Profile {
    fn published_edition(&self) -> PublishedEdition {
        match self {
            Self::CanonicalNoTextBoxes => PublishedEdition::CanonicalBoxDifferences,
            Self::CanonicalMeasuredGrid => PublishedEdition::MeasuredGridDifferences,
        }
    }
}
pub(crate) struct ExactDifferenceSpec(RawSpec);
pub(crate) struct ResolvedSpec<'a> {
    spec: &'a RawSpec,
    source: Vec<(String, Vec<String>)>,
    current: Option<&'a CurrentRequirements>,
}
pub(crate) struct CompletedComparison {
    nodes: Vec<NativeNode>,
    observations: Vec<ComparedField>,
}
impl CompletedComparison {
    pub(crate) fn raw_differences(&self) -> impl Iterator<Item = &ComparedField> {
        self.observations.iter().filter(|field| field.differs())
    }
}
#[derive(Debug, PartialEq)]
pub(crate) enum RawVerdict {
    Different { fields: usize },
}
#[derive(Debug, PartialEq)]
pub(crate) enum QualifiedVerdict {
    Intentional { policy: String, edition: String },
}
#[derive(Debug, PartialEq)]
pub(crate) struct QualifiedComparison {
    pub(crate) raw: RawVerdict,
    pub(crate) qualified: QualifiedVerdict,
}

fn text(value: &str) -> Result<()> {
    if value.is_empty() || value.trim() != value {
        Err(malformed("expected nonempty trimmed text"))
    } else {
        Ok(())
    }
}
fn digest(value: &str) -> Result<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(malformed("expected lowercase SHA-256"))
    }
}
fn relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.contains(['\\', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(malformed(format!("invalid relative path {path}")));
    }
    Ok(())
}
fn source_path(path: &str) -> Result<()> {
    if path == "/" {
        return Ok(());
    }
    if !path.starts_with('/')
        || path[1..].split('/').any(|part| {
            part.is_empty()
                || (part.len() > 1 && part.starts_with('0'))
                || !part.bytes().all(|b| b.is_ascii_digit())
                || part.parse::<usize>().is_err()
        })
    {
        return Err(malformed(format!("invalid source path {path}")));
    }
    Ok(())
}
fn unique<T>(values: &[T], key: impl Fn(&T) -> String) -> Result<BTreeMap<String, &T>> {
    let mut map = BTreeMap::new();
    for value in values {
        let key = key(value);
        if map.insert(key.clone(), value).is_some() {
            return Err(QualificationError::DuplicateKey { key });
        }
    }
    Ok(map)
}
impl CaseIdentity {
    pub(crate) fn from_verified_projection(value: serde_json::Value) -> Result<Self> {
        let identity: Self = serde_json::from_value(value).map_err(|e| malformed(e.to_string()))?;
        identity.validate()?;
        Ok(identity)
    }
    fn changed_component(&self, actual: &Self) -> Option<String> {
        macro_rules! fields { ($($field:ident),+ $(,)?) => { $(if self.$field != actual.$field { return Some(stringify!($field).to_owned()); })+ }; }
        fields!(
            source,
            source_sha256,
            variant,
            historical_sha256,
            capture_sha256,
            record_sha256,
            generator,
            engine_version,
            browser_executable,
            browser_executable_sha256,
            browser_provenance,
            profile_sha256,
            source_imports_sha256,
            inputs,
            linked_resources,
            font_demand,
            policy_source_sha256
        );
        None
    }
    fn shared_basis_changed_component(&self, other: &Self) -> Option<String> {
        macro_rules! fields { ($($field:ident),+ $(,)?) => { $(if self.$field != other.$field { return Some(stringify!($field).to_owned()); })+ }; }
        fields!(
            source,
            source_sha256,
            variant,
            record_sha256,
            historical_sha256,
            font_demand,
            policy_source_sha256
        );
        for (path, digest) in self.inputs.iter().chain(&self.linked_resources) {
            for map in [&other.inputs, &other.linked_resources] {
                if map.get(path).is_some_and(|other| other != digest) {
                    return Some(format!("resource:{path}"));
                }
            }
        }
        None
    }
    fn validate_policy_inputs(&self, policy: &Policy) -> Result<()> {
        // The verified caller supplies the complete capture census. Parsing
        // rejects inconsistent authored entries without inventing absent ones;
        // exact runtime map equality remains mandatory at resolution.
        for (path, expected) in [
            (&self.source, &self.source_sha256),
            (&policy.source, &policy.source_sha256),
        ] {
            for map in [&self.inputs, &self.linked_resources] {
                if map.get(path).is_some_and(|actual| actual != expected) {
                    return Err(malformed(format!(
                        "dedicated identity input disagrees: {path}"
                    )));
                }
            }
        }
        for (path, digest) in &self.linked_resources {
            if self.inputs.get(path).is_some_and(|input| input != digest) {
                return Err(malformed(format!(
                    "resource identity disagrees between maps: {path}"
                )));
            }
        }
        Ok(())
    }
    fn validate(&self) -> Result<()> {
        for value in [
            &self.source,
            &self.variant,
            &self.generator,
            &self.engine_version,
            &self.browser_executable,
            &self.browser_provenance,
        ] {
            text(value)?;
        }
        for value in [
            &self.source_sha256,
            &self.capture_sha256,
            &self.record_sha256,
            &self.browser_executable_sha256,
            &self.profile_sha256,
            &self.source_imports_sha256,
            &self.policy_source_sha256,
        ]
        .into_iter()
        .chain(self.historical_sha256.iter())
        .chain(self.inputs.values())
        .chain(self.linked_resources.values())
        {
            digest(value)?;
        }
        relative_path(&self.source)?;
        relative_path(&self.browser_executable)?;
        for path in self.inputs.keys().chain(self.linked_resources.keys()) {
            relative_path(path)?;
        }
        if self.inputs.is_empty() {
            return Err(malformed("missing complete capture inputs"));
        }
        Ok(())
    }
}
impl ExactDifferenceSpec {
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        // Share the already-owned duplicate-key parser; no Value last-key wins.
        let value = prepared_source::unique_json(raw).map_err(|e| malformed(e.to_string()))?;
        let spec: RawSpec = serde_json::from_value(value).map_err(|e| malformed(e.to_string()))?;
        for value in [
            &spec.policy.id,
            &spec.policy.edition,
            &spec.policy.source,
            &spec.reason,
            &spec.native.source_derivation,
        ] {
            text(value)?;
        }
        relative_path(&spec.policy.source)?;
        digest(&spec.policy.source_sha256)?;
        let Some((_, basis)) = spec.identities.first_key_value() else {
            return Err(malformed("capture identities must be nonempty"));
        };
        for (lineage, identity) in &spec.identities {
            identity.validate()?;
            identity.validate_policy_inputs(&spec.policy)?;
            if spec.policy.source_sha256 != identity.policy_source_sha256 {
                return Err(malformed("policy source identity disagrees"));
            }
            if let Some(component) = basis.shared_basis_changed_component(identity) {
                return Err(malformed(format!(
                    "capture lineages disagree on shared semantic basis: {component}"
                )));
            }
            if *lineage == CaptureLineage::Published
                && spec.representation.published_edition != Some(spec.profile.published_edition())
            {
                return Err(malformed(format!(
                    "published identity requires the {} edition association",
                    spec.profile.published_edition().name(),
                )));
            }
            let measured = matches!(spec.profile, Profile::CanonicalMeasuredGrid);
            if measured != (identity.font_demand == FontDemand::AhemZwsp)
                || measured && *lineage != CaptureLineage::Published
            {
                return Err(malformed(
                    "font demand or capture lineage disagrees with the source profile",
                ));
            }
        }
        if spec.native.nodes.is_empty() || spec.differences.is_empty() {
            return Err(malformed(
                "Native census and intentional set must be nonempty",
            ));
        }
        unique(&spec.native.nodes, |n| n.path.clone())?;
        unique(&spec.differences, ComparedField::key)?;
        for node in &spec.native.nodes {
            source_path(&node.path)?;
            unique(&node.children, Clone::clone)?;
            for child in &node.children {
                source_path(child)?;
            }
            validate_extents(
                &serde_json::to_value(node).map_err(|e| malformed(e.to_string()))?,
                "",
            )?;
        }
        for difference in &spec.differences {
            source_path(difference.path())?;
            if let Field::ElementSize {
                browser, native, ..
            } = &difference.0
                && (*browser > i32::MAX as u32 || *native > i32::MAX as u32)
            {
                return Err(malformed("DOM size pair exceeds nonnegative CSSOM long"));
            }
            if !difference.differs() {
                return Err(malformed("declared pair is not a raw difference"));
            }
        }
        Ok(Self(spec))
    }
    pub(crate) fn policy_source(&self) -> (&str, &str) {
        (&self.0.policy.source, &self.0.policy.source_sha256)
    }
    pub(crate) fn published_edition(&self) -> Option<PublishedEdition> {
        self.0.representation.published_edition
    }
    pub(crate) fn require_published_context(&self, edition: PublishedEdition) -> Result<()> {
        if self.published_edition() != Some(edition) {
            return Err(QualificationError::IdentityChanged {
                component: "published_edition".to_owned(),
            });
        }
        if !self.0.identities.contains_key(&CaptureLineage::Published) {
            return Err(QualificationError::IdentityChanged {
                component: "capture_lineage.published".to_owned(),
            });
        }
        if self.0.representation.current.is_none() {
            return Err(QualificationError::IncompleteRepresentation {
                key: "representation.current".to_owned(),
            });
        }
        Ok(())
    }
    /// Freeze the complete current canonical requirement before execution. Parse
    /// may retain an unresolved declaration; it never manufactures this binding.
    pub(crate) fn prebind_current(
        &self,
        edition: PublishedEdition,
        selected: &super::capture_authority::CaseSelection,
    ) -> Result<super::capture_authority::CurrentRequirements> {
        self.require_published_context(edition)?;
        if self.published_edition() != Some(edition) {
            return Err(QualificationError::IdentityChanged {
                component: "published_edition".to_owned(),
            });
        }
        let identity = self
            .0
            .identities
            .get(&CaptureLineage::Published)
            .ok_or_else(|| QualificationError::IdentityChanged {
                component: "capture_lineage.published".to_owned(),
            })?;
        if identity.source != selected.source
            || identity.source_sha256 != selected.source_sha256
            || identity.variant != selected.variant
            || identity.record_sha256 != selected.record_sha256
            || identity.historical_sha256 != selected.historical_sha256
        {
            return Err(QualificationError::IdentityChanged {
                component: "current representative source/variant/history/record".to_owned(),
            });
        }
        let required = self.0.representation.current.as_ref().ok_or_else(|| {
            QualificationError::IncompleteRepresentation {
                key: "representation.current".to_owned(),
            }
        })?;
        relative_path(&required.output)?;
        if required.case_id != selected.name || required.output != selected.output {
            return Err(QualificationError::IncompleteRepresentation {
                key: "current representative case/output".to_owned(),
            });
        }
        let mut nodes = self
            .0
            .native
            .nodes
            .iter()
            .map(|n| n.path.clone())
            .collect::<Vec<_>>();
        nodes.sort();
        if self.0.native.nodes.iter().any(|n| {
            !matches!(n.unrounded, Packet::Present { .. })
                || !matches!(n.final_output, Packet::Present { .. })
        }) {
            return Err(QualificationError::IncompleteNativeOracle {
                key: "Published canonical boxes require both present packet channels".to_owned(),
            });
        }
        let selectors = canonical_selectors(edition.domain(), &nodes);
        if required.domain != edition.domain()
            || required.nodes != nodes
            || required.unrounded_packets != nodes
            || required.final_packets != nodes
            || required.selectors != selectors
        {
            return Err(QualificationError::IncompleteRepresentation {
                key: "complete canonical source/selector/both-packet census".to_owned(),
            });
        }
        Ok(required.clone())
    }
    pub(crate) fn resolve_against<'a>(
        &'a self,
        authority: CaptureAuthority<'_>,
        golden: &Golden,
    ) -> Result<ResolvedSpec<'a>> {
        let (lineage, identity, edition) = match authority {
            CaptureAuthority::Focused(identity) => (CaptureLineage::Focused, identity, None),
            CaptureAuthority::Published { edition, identity } => {
                (CaptureLineage::Published, identity, Some(edition))
            }
        };
        let expected =
            self.0
                .identities
                .get(&lineage)
                .ok_or_else(|| QualificationError::IdentityChanged {
                    component: format!("capture_lineage.{}", lineage.as_str()),
                })?;
        if let Some(edition) = edition
            && self.published_edition() != Some(edition)
        {
            return Err(QualificationError::IdentityChanged {
                component: "published_edition".to_owned(),
            });
        }
        if let Some(component) = expected.changed_component(identity) {
            return Err(QualificationError::IdentityChanged { component });
        }
        let measured_path = match self.0.profile {
            Profile::CanonicalNoTextBoxes => None,
            Profile::CanonicalMeasuredGrid => {
                let leaf = golden.measured_leaf.as_ref().ok_or_else(|| {
                    unsupported(
                        "/",
                        "measured Grid profile requires its retained leaf witness",
                    )
                })?;
                Some(format!("/{}", leaf.child_index))
            }
        };
        let source = source_census(&golden.root, &golden.expectations, measured_path.as_deref())?;
        let current = if let Some(edition) = edition {
            let current = self.0.representation.current.as_ref().ok_or_else(|| {
                QualificationError::IncompleteRepresentation {
                    key: "representation.current".to_owned(),
                }
            })?;
            let selected = super::capture_authority::CaseSelection {
                name: current.case_id.clone(),
                output: current.output.clone(),
                source: identity.source.clone(),
                source_sha256: identity.source_sha256.clone(),
                variant: identity.variant.clone(),
                record_sha256: identity.record_sha256.clone(),
                historical_sha256: identity.historical_sha256.clone(),
            };
            let required = self.prebind_current(edition, &selected)?;
            verify_current_requirements(&required, golden, &source)?;
            Some(current)
        } else {
            None
        };
        let expected = unique(&self.0.native.nodes, |n| n.path.clone())?;
        if expected.len() != source.len() {
            return Err(QualificationError::IncompleteNativeOracle {
                key: "source census".to_owned(),
            });
        }
        for (path, children) in &source {
            let node = expected
                .get(path)
                .ok_or_else(|| QualificationError::IncompleteNativeOracle { key: path.clone() })?;
            if &node.children != children {
                return Err(QualificationError::IncompleteNativeOracle {
                    key: format!("{path}:ordered children"),
                });
            }
        }
        Ok(ResolvedSpec {
            spec: &self.0,
            source,
            current,
        })
    }
}

fn canonical_selectors(domain: CurrentDomain, nodes: &[String]) -> Vec<Selector> {
    nodes
        .iter()
        .flat_map(|path| {
            domain.fields().iter().map(|&field| Selector {
                path: path.clone(),
                field,
            })
        })
        .collect()
}

fn verify_current_requirements(
    required: &CurrentRequirements,
    golden: &Golden,
    source: &[(String, Vec<String>)],
) -> Result<()> {
    let measured = required.domain == CurrentDomain::LayoutAndElementSizes;
    if !matches!(
        &golden.viewport.root_context,
        RootContext::Qualified(root_context::QualifiedRootContext::AbsoluteRootLocal { .. })
    ) || golden.has_element_scroll_sizes() != measured
        || golden.measured_leaf.is_some() != measured
    {
        return Err(QualificationError::IncompleteRepresentation {
            key: "current root-border-local observation frame or domain".to_owned(),
        });
    }
    fn verify_geometry(expected: &Expectation) -> Result<()> {
        if expected.scroll_size.is_some()
            || expected.x.is_none()
            || expected.y.is_none()
            || expected.width.is_none()
            || expected.height.is_none()
        {
            return Err(QualificationError::IncompleteRepresentation {
                key: "complete current canonical box observations".to_owned(),
            });
        }
        for child in &expected.children {
            verify_geometry(child)?;
        }
        Ok(())
    }
    verify_geometry(&golden.expectations)?;
    let mut nodes = source
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    nodes.sort();
    if nodes != required.nodes {
        return Err(QualificationError::IncompleteRepresentation {
            key: "actual current requested source census".to_owned(),
        });
    }
    Ok(())
}

// A nullable input is still a required authored field. Serde must not turn a
// missing key into an invented absence; the owning type establishes the schema.
pub(crate) fn explicit_nullable<'de, D, T>(
    deserializer: D,
) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

fn validate_extents(value: &serde_json::Value, key: &str) -> Result<()> {
    match value {
        serde_json::Value::Object(map) => {
            for (field, value) in map {
                if matches!(field.as_str(), "width" | "height")
                    && value.as_f64().is_some_and(|v| v < 0.0)
                {
                    return Err(malformed(format!("negative extent {key}.{field}")));
                }
                if field == "minimum"
                    && value
                        .as_f64()
                        .zip(map.get("maximum").and_then(serde_json::Value::as_f64))
                        .is_some_and(|(min, max)| min > max)
                {
                    return Err(malformed(format!("inverted interval {key}")));
                }
                validate_extents(value, &format!("{key}.{field}"))?;
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                validate_extents(value, key)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn unsupported(owner: &str, category: &str) -> QualificationError {
    QualificationError::UnsupportedObservation {
        owner: owner.to_owned(),
        category: category.to_owned(),
    }
}
fn source_census(
    root: &Node,
    expectation: &Expectation,
    measured_path: Option<&str>,
) -> Result<Vec<(String, Vec<String>)>> {
    fn walk(
        node: &Node,
        expected: &Expectation,
        path: String,
        measured_path: Option<&str>,
        out: &mut Vec<(String, Vec<String>)>,
    ) -> Result<()> {
        let role_matches = if measured_path == Some(path.as_str()) {
            node.kind == NodeKind::Text && node.text.is_some()
        } else {
            node.kind == NodeKind::Div && node.text.is_none()
        };
        if !role_matches
            || node.inline_text.is_some()
            || node.prepared.is_some()
            || node.prepared_occurrence.is_some()
            || node.prepared_origin.is_some()
            || node.prepared_control.is_some()
            || node.inline_boundary.is_some()
            || node.atomic_inline_participation.is_some()
            || node.anonymous_grid_text_wrapper
            || node.layout_ready_inline_root
            || node.atomic_paragraph_metrics.is_some()
            || node.grid_alignment_reference.is_some()
            || node.shape_bands.is_some()
            || node.atomic_line_strut.is_some()
        {
            return Err(unsupported(
                &path,
                "source does not match its canonical box role",
            ));
        }
        if expected.fragments.is_some()
            || expected.range_rectangles.is_some()
            || expected.browser_control.is_some()
            || expected.browser_br_font_band.is_some()
        {
            return Err(unsupported(&path, "fragment/Range/control request"));
        }
        if node.children.len() != expected.children.len() {
            return Err(QualificationError::IncompleteNativeOracle {
                key: format!("{path}:comparison topology"),
            });
        }
        let children: Vec<_> = (0..node.children.len())
            .map(|i| {
                if path == "/" {
                    format!("/{i}")
                } else {
                    format!("{path}/{i}")
                }
            })
            .collect();
        out.push((path, children.clone()));
        for ((child, expectation), path) in
            node.children.iter().zip(&expected.children).zip(children)
        {
            walk(child, expectation, path, measured_path, out)?;
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, expectation, "/".to_owned(), measured_path, &mut out)?;
    Ok(out)
}

pub(crate) fn complete_captured_comparison(
    golden: &Golden,
    spec: &ResolvedSpec<'_>,
) -> Result<CompletedComparison> {
    // One admitted computation and the existing terminal traversal. A late error
    // discards both collected packets and observations.
    let mut snapshot = None;
    let mut snapshot_error = None;
    let mut sink = NumericComparisonSink::CollectAll {
        root: golden.name.clone(),
        fields: Vec::new(),
    };
    let comparison = compare_captured_with_batch(golden, &mut sink, |tree, batch| {
        match snapshot_batch(tree, batch, &spec.source) {
            Ok(nodes) => snapshot = Some(nodes),
            Err(error) => {
                let message = error.to_string();
                snapshot_error = Some(error);
                return Err(Error::new(message));
            }
        }
        Ok(())
    });
    if let Some(error) = snapshot_error {
        return Err(error);
    }
    let comparison = comparison.map_err(|e| QualificationError::ComparisonFailure {
        reason: e.to_string(),
    })?;
    if !comparison.endpoint_unobservable_fields().is_empty() {
        return Err(unsupported("/", "endpoint-unobservable output census"));
    }
    let NumericComparisonSink::CollectAll { fields, .. } = sink else {
        unreachable!("owned collecting sink")
    };
    if let Some(required) = spec.current {
        let mut actual = fields
            .iter()
            .map(ComparedField::canonical_selector)
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| QualificationError::IncompleteRepresentation {
                key: "comparison produced a field outside the current domain".to_owned(),
            })?;
        actual.sort();
        if actual != required.selectors {
            return Err(QualificationError::IncompleteRepresentation {
                key: "complete executed current comparison selectors".to_owned(),
            });
        }
    }
    unique(&fields, ComparedField::key)?;
    let nodes = snapshot.ok_or_else(|| malformed("missing completed batch"))?;
    Ok(CompletedComparison {
        nodes,
        observations: fields,
    })
}
impl ResolvedSpec<'_> {
    pub(crate) fn qualify(&self, completed: &CompletedComparison) -> Result<QualifiedComparison> {
        let expected = unique(&self.spec.native.nodes, |n| n.path.clone())?;
        if expected.len() != completed.nodes.len() {
            return Err(QualificationError::IncompleteNativeOracle {
                key: "completed source census".to_owned(),
            });
        }
        for node in &completed.nodes {
            let expected = expected.get(&node.path).ok_or_else(|| {
                QualificationError::IncompleteNativeOracle {
                    key: node.path.clone(),
                }
            })?;
            compare_native(expected, node)?;
        }
        // Also resolve requested-field membership: an undeclared or removed row
        // cannot accidentally be accepted by a set with the same count.
        let observed = unique(&completed.observations, ComparedField::key)?;
        let actual: BTreeMap<_, _> = observed.into_iter().filter(|(_, v)| v.differs()).collect();
        if actual.is_empty() {
            return Err(QualificationError::UnexpectedPass);
        }
        let declared = unique(&self.spec.differences, ComparedField::key)?;
        let missing = declared
            .keys()
            .filter(|k| !actual.contains_key(*k))
            .cloned()
            .collect::<Vec<_>>();
        let extra = actual
            .keys()
            .filter(|k| !declared.contains_key(*k))
            .cloned()
            .collect::<Vec<_>>();
        let changed = actual
            .iter()
            .filter_map(|(k, v)| declared.get(k).filter(|d| *d != v).map(|_| k.clone()))
            .collect::<Vec<_>>();
        if !missing.is_empty() || !extra.is_empty() || !changed.is_empty() {
            return Err(QualificationError::DifferenceSetChanged {
                missing,
                extra,
                changed,
            });
        }
        Ok(QualifiedComparison {
            raw: RawVerdict::Different {
                fields: actual.len(),
            },
            qualified: QualifiedVerdict::Intentional {
                policy: self.spec.policy.id.clone(),
                edition: self.spec.policy.edition.clone(),
            },
        })
    }
}
fn compare_native(expected: &NativeNode, actual: &NativeNode) -> Result<()> {
    fn walk(expected: &serde_json::Value, actual: &serde_json::Value, key: &str) -> Result<()> {
        if expected == actual {
            return Ok(());
        }
        match (expected, actual) {
            (serde_json::Value::Object(e), serde_json::Value::Object(a))
                if e.keys().eq(a.keys()) =>
            {
                for (name, value) in e {
                    walk(value, &a[name], &format!("{key}.{name}"))?;
                }
            }
            _ => {
                return Err(QualificationError::NativeOracleChanged {
                    key: key.to_owned(),
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                });
            }
        }
        Ok(())
    }
    // Serialize only the closed private expectation carrier, never production
    // structs or opaque ScrollGeometry internals.
    let expected = serde_json::to_value(expected).map_err(|e| malformed(e.to_string()))?;
    let actual = serde_json::to_value(actual).map_err(|e| malformed(e.to_string()))?;
    walk(&expected, &actual, actual["path"].as_str().unwrap_or("/"))
}
fn snapshot_batch(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
    source: &[(String, Vec<String>)],
) -> Result<Vec<NativeNode>> {
    if !batch.placed_text().is_empty()
        || !batch.unrounded_inline_fragments().is_empty()
        || !batch.final_inline_fragments().is_empty()
        || batch.native_inline_box_families().next().is_some()
        || batch.native_inline_box_pieces().next().is_some()
        || !tree.paragraphs.is_empty()
    {
        return Err(unsupported(
            "/",
            "Text/fragment/native-inline output census",
        ));
    }
    if tree.nodes.len() != source.len()
        || tree.nodes.iter().any(|n| {
            n.synthetic
                || n.layout_input
                    .as_box()
                    .is_none_or(|input| input.display.is_inline_level())
        })
    {
        return Err(unsupported("/", "synthetic/non-box lowered source"));
    }
    fn packets(
        entries: &[layout::LayoutOutputEntry<usize>],
        count: usize,
    ) -> Result<BTreeMap<usize, Packet>> {
        let mut out = BTreeMap::new();
        for entry in entries {
            if entry.node() >= count {
                return Err(unsupported("/", "foreign batch node"));
            }
            if out
                .insert(
                    entry.node(),
                    Packet::Present {
                        facts: Box::new(node_facts(entry.output())?),
                    },
                )
                .is_some()
            {
                return Err(QualificationError::DuplicateKey {
                    key: format!("batch node {}", entry.node()),
                });
            }
        }
        Ok(out)
    }
    let mut unrounded = packets(batch.unrounded_entries(), source.len())?;
    let mut final_output = packets(batch.final_entries(), source.len())?;
    // Associate actual source-child identities, never batch slice ordinals.
    fn associate(
        tree: &TestTree,
        node: usize,
        path: String,
        out: &mut BTreeMap<String, usize>,
    ) -> Result<()> {
        if out.insert(path.clone(), node).is_some() {
            return Err(QualificationError::DuplicateKey { key: path });
        }
        for (i, child) in tree.nodes[node].children.iter().enumerate() {
            associate(
                tree,
                *child,
                if path == "/" {
                    format!("/{i}")
                } else {
                    format!("{path}/{i}")
                },
                out,
            )?;
        }
        Ok(())
    }
    let mut association = BTreeMap::new();
    associate(tree, 0, "/".to_owned(), &mut association)?;
    let mut nodes = Vec::new();
    for (path, children) in source {
        let node = *association
            .get(path)
            .ok_or_else(|| QualificationError::IncompleteNativeOracle { key: path.clone() })?;
        nodes.push(NativeNode {
            path: path.clone(),
            role: SourceRole::OrdinaryBox,
            children: children.clone(),
            unrounded: unrounded.remove(&node).unwrap_or(Packet::Absent),
            final_output: final_output.remove(&node).unwrap_or(Packet::Absent),
        });
    }
    if !unrounded.is_empty() || !final_output.is_empty() || association.len() != source.len() {
        return Err(unsupported("/", "unconsumed batch/source node"));
    }
    Ok(nodes)
}

fn point(value: layout::Point) -> Result<Point> {
    Ok(Point {
        x: Number::new(value.x)?,
        y: Number::new(value.y)?,
    })
}
fn extent(value: layout::Size<Scalar>) -> Result<Extent> {
    Ok(Extent {
        width: Number::new(value.width)?,
        height: Number::new(value.height)?,
    })
}
fn edges(value: layout::Edges<Scalar>) -> Result<Edges> {
    Ok(Edges {
        top: Number::new(value.top)?,
        right: Number::new(value.right)?,
        bottom: Number::new(value.bottom)?,
        left: Number::new(value.left)?,
    })
}
fn rect(value: layout::ScrollRect) -> Result<Rect> {
    Ok(Rect {
        origin: point(value.origin())?,
        size: extent(value.size())?,
    })
}
fn flow(value: layout::FlowAxes) -> Flow {
    Flow {
        writing_mode: value.writing_mode().into(),
        direction: value.direction().into(),
    }
}
fn interval(minimum: Scalar, maximum: Scalar) -> Result<Interval> {
    Ok(Interval {
        minimum: Number::new(minimum)?,
        maximum: Number::new(maximum)?,
    })
}
fn fact(value: layout::BaselineFactOf) -> Result<Fact> {
    Ok(match value {
        layout::BaselineFactOf::Unknown => Fact::Unknown,
        layout::BaselineFactOf::Absent => Fact::Absent,
        layout::BaselineFactOf::Available(c) => Fact::Available {
            axis: c.axis().into(),
            offset: Number::new(c.offset())?,
        },
    })
}
fn pair(value: layout::BaselinePairOf) -> Result<Pair> {
    Ok(Pair {
        first: fact(value.first())?,
        last: fact(value.last())?,
    })
}
fn node_facts(output: layout::NodeOutput) -> Result<NodeFacts> {
    // No `..`: a new public packet field requires an explicit oracle decision.
    let layout::NodeOutput {
        source_index,
        location,
        size,
        content_size,
        border,
        padding,
        margin,
        named_baselines,
        scroll_geometry,
    } = output;
    Ok(NodeFacts {
        source_index: source_index.get(),
        location: point(location)?,
        size: extent(size)?,
        content_size: extent(content_size)?,
        border: edges(border)?,
        padding: edges(padding)?,
        margin: edges(margin)?,
        named_baselines: NamedFacts {
            alphabetic: pair(named_baselines.get(layout::BaselineName::Alphabetic))?,
            central: pair(named_baselines.get(layout::BaselineName::Central))?,
        },
        scroll: match scroll_geometry {
            None => Scroll::Absent,
            Some(g) => Scroll::Present {
                facts: Box::new(scroll_facts(g)?),
            },
        },
    })
}
fn scroll_facts(g: layout::ScrollGeometry) -> Result<ScrollFacts> {
    let clips = g.overflow_clip();
    let range = g.physical_range();
    let gutters = g.gutters();
    let target = g.target();
    let margin = target.scroll_margin();
    Ok(ScrollFacts {
        flow_axes: flow(g.flow_axes()),
        used_overflow_x: g.used_overflow_x().into(),
        used_overflow_y: g.used_overflow_y().into(),
        border_box: rect(g.border_box())?,
        padding_box: rect(g.padding_box())?,
        content_box: rect(g.content_box())?,
        scrollport: rect(g.scrollport())?,
        clip_x: clips
            .x()
            .map(|i| interval(i.minimum(), i.maximum()))
            .transpose()?,
        clip_y: clips
            .y()
            .map(|i| interval(i.minimum(), i.maximum()))
            .transpose()?,
        scrollable_overflow: rect(g.scrollable_overflow())?,
        range_x: interval(range.x().minimum(), range.x().maximum())?,
        range_y: interval(range.y().minimum(), range.y().maximum())?,
        gutter_top: gutters.top().map(rect).transpose()?,
        gutter_right: gutters.right().map(rect).transpose()?,
        gutter_bottom: gutters.bottom().map(rect).transpose()?,
        gutter_left: gutters.left().map(rect).transpose()?,
        scrollbar_size: extent(g.scrollbar_size())?,
        resolved_scroll_padding: edges(g.resolved_scroll_padding())?,
        optimal_viewing_region: rect(g.optimal_viewing_region())?,
        scroll_snap_type: match g.scroll_snap_type() {
            layout::ScrollSnapType::None => Snap::None,
            layout::ScrollSnapType::Enabled { axis, strictness } => Snap::Enabled {
                axis: axis.into(),
                strictness: strictness.into(),
            },
        },
        target: Target {
            border_box: rect(target.border_box())?,
            scroll_margin: Edges {
                top: Number::new(margin.top())?,
                right: Number::new(margin.right())?,
                bottom: Number::new(margin.bottom())?,
                left: Number::new(margin.left())?,
            },
            flow_axes: flow(target.flow_axes()),
            block_align: target.snap_align().block().into(),
            inline_align: target.snap_align().inline().into(),
            snap_stop: target.snap_stop().into(),
        },
    })
}

pub(super) fn source_path_from_diagnostic(
    root: &str,
    path: &str,
) -> std::result::Result<String, Error> {
    let suffix = path
        .strip_prefix(root)
        .ok_or_else(|| Error::new("comparison path lost its root"))?;
    let source = if suffix.is_empty() { "/" } else { suffix };
    source_path(source).map_err(|e| Error::new(e.to_string()))?;
    Ok(source.to_owned())
}

#[cfg(test)]
#[path = "exact_differences_tests.rs"]
mod tests;
