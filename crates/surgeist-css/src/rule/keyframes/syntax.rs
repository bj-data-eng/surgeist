use crate::component::CssComponentValues;
use crate::declaration::{CssCustomDeclaration, CssDeclaration, CssDeclarationBody, CssImportance};
use crate::numeric::{CssCalculation, CssNumeric, CssPercentage};
use crate::property::CssKnownDeclaration;
use crate::property::CssPropertyNameRef;
use crate::source::CssSourcePosition;
use crate::values::common::CssCustomIdent;

#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframesRule {
    name: CssKeyframesName,
    blocks: Vec<CssKeyframeBlock>,
    position: CssSourcePosition,
}

impl CssKeyframesRule {
    #[must_use]
    pub(crate) fn new(
        name: CssKeyframesName,
        blocks: Vec<CssKeyframeBlock>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            blocks,
            position,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &CssKeyframesName {
        &self.name
    }

    #[must_use]
    pub fn blocks(&self) -> &[CssKeyframeBlock] {
        &self.blocks
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssKeyframesName {
    Ident(CssKeyframesIdent),
    String(CssKeyframesString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframesString {
    value: String,
}

impl CssKeyframesString {
    /// Preserves an unrestricted decoded animation name, including empty strings.
    /// Unrepresentable string contents fail explicitly during specified serialization.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self { value }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// A parser-produced authored keyframe block with a distinct declaration collection.
///
/// Its private fields couple validated selectors, keyframe-only declarations, and semantic source
/// provenance. It does not apply cascade, resolve substitutions, or interpolate animations.
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeBlock {
    selectors: CssKeyframeSelectorList,
    declarations: CssKeyframeDeclarationList,
    position: CssSourcePosition,
}

impl CssKeyframeBlock {
    #[must_use]
    pub(crate) fn new(
        selectors: CssKeyframeSelectorList,
        declarations: CssKeyframeDeclarationList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            declarations,
            position,
        }
    }

    #[must_use]
    pub const fn selectors(&self) -> &CssKeyframeSelectorList {
        &self.selectors
    }

    #[must_use]
    pub const fn declarations(&self) -> &CssKeyframeDeclarationList {
        &self.declarations
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeSelectorList {
    selectors: Vec<CssKeyframeSelector>,
}

impl CssKeyframeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssKeyframeSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssKeyframeSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssKeyframeSelector] {
        &self.selectors
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssKeyframeSelector {
    From,
    To,
    Percent(CssKeyframePercent),
}

impl CssKeyframeSelector {
    /// Returns the authored percentage domain, without evaluating calculations.
    #[must_use]
    pub fn offset(&self) -> CssKeyframePercent {
        match self {
            Self::From => CssKeyframePercent {
                value: CssKeyframePercentData::Literal(0.0),
            },
            Self::To => CssKeyframePercent {
                value: CssKeyframePercentData::Literal(100.0),
            },
            Self::Percent(percent) => percent.clone(),
        }
    }
}

/// An authored keyframe percentage: a bounded binary64 literal or symbolic percentage math.
///
/// Literal admission converts to binary64 before checking the inclusive range. Calculations
/// remain authored; their range is checked at computed or used value time downstream.
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframePercent {
    value: CssKeyframePercentData,
}

#[derive(Clone, Debug, PartialEq)]
enum CssKeyframePercentData {
    Literal(f64),
    Calculation(CssNumeric<CssPercentage>),
}

impl CssKeyframePercent {
    /// Accepts a finite binary64 percentage within the inclusive range 0 through 100.
    #[must_use]
    pub fn try_new(value: f64) -> Option<Self> {
        if value.is_finite() && (0.0..=100.0).contains(&value) {
            Some(Self {
                value: CssKeyframePercentData::Literal(value),
            })
        } else {
            None
        }
    }

    /// Retains a checked percentage function, or range-checks a bare percentage root.
    ///
    /// The shared percentage owner classifies the already checked root. Bare
    /// roots use the same binary64 literal boundary as [`Self::try_new`]. Function roots retain
    /// exact authored coefficients and provenance without evaluation or clamping.
    #[must_use]
    pub fn try_from_calculation(calculation: CssCalculation<CssPercentage>) -> Option<Self> {
        Self::from_specified(CssNumeric::<CssPercentage>::try_from_calculation(calculation).ok()?)
    }

    /// Parser admission retains recovered syntax, which parsing reports separately.
    pub(crate) fn from_parser_calculation(
        calculation: CssCalculation<CssPercentage>,
    ) -> Option<Self> {
        Self::from_specified(
            CssNumeric::<CssPercentage>::from_parser_calculation(calculation).ok()?,
        )
    }

    fn from_specified(value: CssNumeric<CssPercentage>) -> Option<Self> {
        if let Some(component) = value.literal_component() {
            let crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Percentage(number),
            ) = component.view()
            else {
                return None;
            };
            Self::try_new(number.representation().parse::<f64>().ok()?)
        } else {
            Some(Self {
                value: CssKeyframePercentData::Calculation(value),
            })
        }
    }

    /// Returns the supported literal scalar, or `None` for authored calculations.
    #[must_use]
    pub const fn literal_value(&self) -> Option<f64> {
        match &self.value {
            CssKeyframePercentData::Literal(value) => Some(*value),
            CssKeyframePercentData::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic percentage function without computing its offset.
    #[must_use]
    pub fn calculation(&self) -> Option<&CssCalculation<CssPercentage>> {
        match &self.value {
            CssKeyframePercentData::Literal(_) => None,
            CssKeyframePercentData::Calculation(value) => value.calculation(),
        }
    }

    pub(crate) fn specified_calculation(&self) -> Option<&CssNumeric<CssPercentage>> {
        match &self.value {
            CssKeyframePercentData::Literal(_) => None,
            CssKeyframePercentData::Calculation(value) => Some(value),
        }
    }
}

/// An ordered parser-produced collection of keyframe declarations.
///
/// Its distinct element type makes importance unavailable in keyframe syntax while preserving
/// property coupling and semantic source positions. Construction is parser-owned; this collection
/// does not run animation interpolation, cascade, or substitution.
///
/// ```compile_fail
/// use surgeist_css::rule::keyframes::CssKeyframeDeclarationList;
/// let _ = CssKeyframeDeclarationList { declarations: Vec::new() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeDeclarationList {
    declarations: Vec<CssKeyframeDeclaration>,
}

impl CssKeyframeDeclarationList {
    pub(crate) const fn new(declarations: Vec<CssKeyframeDeclaration>) -> Self {
        Self { declarations }
    }

    /// Returns keyframe declarations in authored order.
    #[must_use]
    pub fn as_slice(&self) -> &[CssKeyframeDeclaration] {
        &self.declarations
    }

    /// Iterates over keyframe declarations in authored order.
    pub fn iter(&self) -> std::slice::Iter<'_, CssKeyframeDeclaration> {
        self.declarations.iter()
    }

    /// Returns the number of keyframe declarations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Returns whether no keyframe declarations were retained.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Emits the CSSOM declaration block under the same closed keyframe grammar.
    ///
    /// Preserves genuine parser sources while construction and output share one budget.
    pub fn serialize_cssom(
        &self,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<String, crate::declaration::CssDeclarationBlockError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer.append_cssom_keyframe_declaration_list(self)?;
        Ok(writer.css)
    }
}

impl std::ops::Deref for CssKeyframeDeclarationList {
    type Target = [CssKeyframeDeclaration];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// A parser-produced declaration in the authored keyframe syntax phase.
///
/// The private fields retain property/value coupling, complete value components and the
/// property-name position. Keyframe grammar omits animation properties except timing-function
/// and rejects declaration importance, so this type intentionally has no importance field or
/// accessor. It does not interpolate, apply, cascade, or resolve the authored value.
///
/// ```compile_fail
/// let report = surgeist_css::CssParser::default().parse_sheet("@keyframes x { from { opacity: 0 } }");
/// assert!(report.is_clean());
/// let surgeist_css::rule::CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
///     unreachable!()
/// };
/// let _ = rule.blocks()[0].declarations().as_slice()[0].importance();
/// ```
#[derive(Clone, Debug)]
pub struct CssKeyframeDeclaration {
    source: NormalParsedKeyframeCssDeclaration,
}

/// Parser-only proof: a genuine parsed, normal, admitted keyframe occurrence.
#[derive(Clone, Debug)]
struct NormalParsedKeyframeCssDeclaration(CssDeclaration);

impl PartialEq for CssKeyframeDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.body() == other.body()
            && self.value_components() == other.value_components()
            && self.position() == other.position()
    }
}

impl CssKeyframeDeclaration {
    /// Borrows the independent SVG authored body from its retained occurrence.
    #[must_use]
    pub fn svg_glyph_orientation_vertical(
        &self,
    ) -> Option<&crate::property::CssSvgGlyphOrientationVerticalDeclaration> {
        self.source().svg_glyph_orientation_vertical()
    }

    pub(crate) fn from_parsed(source: CssDeclaration) -> Self {
        assert_eq!(source.importance(), CssImportance::Normal);
        assert!(source.parsed_name().is_some());
        assert!(
            source.known().is_none_or(|known| {
                crate::parser::keyframe_property_admitted(known.property())
            })
        );
        Self {
            source: NormalParsedKeyframeCssDeclaration(source),
        }
    }

    /// Borrows the original parsed occurrence, including context and token origins.
    #[must_use]
    pub fn source(&self) -> &CssDeclaration {
        &self.source.0
    }

    pub(crate) fn value_components(&self) -> &CssComponentValues {
        self.source().value_components()
    }

    /// Returns the property-coupled authored body.
    #[must_use]
    pub fn body(&self) -> &CssDeclarationBody {
        self.source().body()
    }

    /// Returns the known-property declaration, or `None` for custom and independent SVG bodies.
    #[must_use]
    pub fn known(&self) -> Option<&CssKnownDeclaration> {
        self.source().known()
    }

    /// Returns the custom declaration, or `None` for known and independent SVG bodies.
    #[must_use]
    pub fn custom(&self) -> Option<&CssCustomDeclaration> {
        self.source().custom()
    }

    /// Returns a borrowed semantic property-name view derived from the active body.
    #[must_use]
    pub fn property_name(&self) -> CssPropertyNameRef<'_> {
        self.source().property_name()
    }

    /// Returns the original property-name start.
    #[must_use]
    pub fn position(&self) -> CssSourcePosition {
        self.source()
            .position()
            .expect("parser-only keyframe source has a name origin")
    }
}

/// A checked identifier shared by keyframes definitions and animation names.
/// Animations 1 excludes `none` in addition to the Values 4 reserved words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframesIdent(CssCustomIdent);

impl CssKeyframesIdent {
    #[must_use]
    pub fn try_new(value: CssCustomIdent) -> Option<Self> {
        (!value.as_str().eq_ignore_ascii_case("none")).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
