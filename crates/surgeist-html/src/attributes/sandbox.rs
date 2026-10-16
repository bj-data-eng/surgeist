//! HTML iframe sandbox token authorship and its local directive projection.
//!
//! This does not determine a loaded document's active sandboxing policy or
//! perform navigation, origin changes, script execution or security enforcement.
//! WHATWG source attribution is in `licenses/whatwg/NOTICE.md`.

use super::*;
use crate::{
    HtmlDuplicateTokenError, HtmlSpaceSeparatedTokens, HtmlSpaceToken, HtmlTokenComparison,
    HtmlUnorderedTokenSet, HtmlValueRecoveryState, HtmlValueReport,
};
use std::{error::Error, fmt};

macro_rules! sandbox_flags {
    ($($variant:ident => $anchor:literal),+ $(,)?) => {
        /// The sixteen source-defined restrictions in a sandboxing flag set.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum HtmlSandboxFlag { $($variant),+ }

        const ALL_FLAGS: &[HtmlSandboxFlag] = &[$(HtmlSandboxFlag::$variant),+];

        impl HtmlSandboxFlag {
            const fn bit(self) -> u16 { 1 << (self as u8) }

            pub fn source_clause(self) -> &'static str {
                match self { $(Self::$variant => source!($anchor)),+ }
            }
        }
    };
}

sandbox_flags! {
    Navigation => "sandboxed-navigation-browsing-context-flag",
    AuxiliaryNavigation => "sandboxed-auxiliary-navigation-browsing-context-flag",
    TopNavigationWithoutUserActivation => "sandboxed-top-level-navigation-without-user-activation-browsing-context-flag",
    TopNavigationWithUserActivation => "sandboxed-top-level-navigation-with-user-activation-browsing-context-flag",
    Origin => "sandboxed-origin-browsing-context-flag",
    Forms => "sandboxed-forms-browsing-context-flag",
    PointerLock => "sandboxed-pointer-lock-browsing-context-flag",
    Scripts => "sandboxed-scripts-browsing-context-flag",
    AutomaticFeatures => "sandboxed-automatic-features-browsing-context-flag",
    DocumentDomain => "sandboxed-document.domain-browsing-context-flag",
    PropagatesToAuxiliary => "sandbox-propagates-to-auxiliary-browsing-contexts-flag",
    Modals => "sandboxed-modals-flag",
    OrientationLock => "sandboxed-orientation-lock-browsing-context-flag",
    Presentation => "sandboxed-presentation-browsing-context-flag",
    Downloads => "sandboxed-downloads-browsing-context-flag",
    CustomProtocolsNavigation => "sandboxed-custom-protocols-navigation-browsing-context-flag",
}

/// An immutable, finite directive-derived restriction set.
/// It is not a document's active policy or evidence of host enforcement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSandboxFlags {
    bits: u16,
}

impl HtmlSandboxFlags {
    /// A genuinely empty restriction set, such as an absent element directive.
    /// This is distinct from parsing a present empty sandbox attribute.
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }
    /// Combines actual restriction sets without removing either contribution.
    /// This does not attest a Document policy, attribute presence or enforcement.
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
    /// Parses one present sandbox directive through the same ASCII token,
    /// keyword and flag projection owners as `iframe_sandbox`.
    ///
    /// A present empty value imposes all source-defined restrictions. Absence
    /// is a separate caller observation and must not be supplied as empty text.
    /// Unknown and repeated tokens do not prevent recognized processing; this
    /// flags-only path creates no token strings, reports or author diagnostics.
    ///
    /// `admit` runs before the raw header, each bounded shared token-cursor
    /// step, each emitted token slice, each keyword header and each actual raw
    /// unit/ASCII-keyword-byte comparison, then the fixed final projection.
    /// Refusal or cancellation is returned unchanged before the reached action;
    /// the caller retains its cumulative work and cancellation state.
    pub fn parse_with<E>(
        raw: &DocumentString,
        mut admit: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        admit()?;
        let units = raw.code_units();
        let mut cursor = crate::tokens::SpaceTokenCursor::new(units);
        let mut keywords = 0;
        loop {
            admit()?;
            match cursor.step() {
                crate::tokens::SpaceTokenStep::Continue => {}
                crate::tokens::SpaceTokenStep::Token { start, end } => {
                    admit()?;
                    if let Some(keyword) = keyword_with(&units[start..end], &mut admit)? {
                        keywords |= keyword.bit();
                    }
                }
                crate::tokens::SpaceTokenStep::Done => break,
            }
        }
        admit()?;
        Ok(project_flags(keywords))
    }
    pub fn contains(&self, flag: HtmlSandboxFlag) -> bool {
        self.bits & flag.bit() != 0
    }
    pub fn iter(&self) -> impl Iterator<Item = HtmlSandboxFlag> + '_ {
        ALL_FLAGS
            .iter()
            .copied()
            .filter(|flag| self.contains(*flag))
    }
    pub fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

struct KeywordDefinition {
    keyword: HtmlSandboxKeyword,
    spelling: &'static str,
    source: &'static str,
    removes: u16,
}

macro_rules! sandbox_keywords {
    ($($variant:ident => $spelling:literal [$first:ident $(, $flag:ident)*]),+ $(,)?) => {
        /// The thirteen allowed keywords of the selected HTML sandbox edition.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum HtmlSandboxKeyword { $($variant),+ }

        // One source declaration owns recognition, author spelling and all
        // directive relationships, including keywords that relax several flags.
        const KEYWORDS: &[KeywordDefinition] = &[$(
            KeywordDefinition {
                keyword: HtmlSandboxKeyword::$variant,
                spelling: $spelling,
                source: concat!(source!("attr-iframe-sandbox-"), $spelling),
                removes: HtmlSandboxFlag::$first.bit() $(| HtmlSandboxFlag::$flag.bit())*,
            }
        ),+];

        impl HtmlSandboxKeyword {
            const fn bit(self) -> u16 { 1 << (self as u8) }

            pub fn keyword(self) -> &'static str {
                KEYWORDS[self as usize].spelling
            }
            pub fn source_clause(self) -> &'static str {
                KEYWORDS[self as usize].source
            }
            /// Recognizes only an entire token by the source's ASCII comparison.
            /// The token's spelling, including non-scalar units, is not changed.
            pub fn from_token(token: &HtmlSpaceToken) -> Option<Self> {
                match keyword_with(token.value().code_units(), &mut || Ok::<_, std::convert::Infallible>(())) {
                    Ok(keyword) => keyword,
                    Err(never) => match never {},
                }
            }
            fn token(self) -> HtmlSpaceToken {
                HtmlSpaceToken::new(self.keyword()).expect("sandbox keyword is one nonempty ASCII token")
            }
        }
    };
}

sandbox_keywords! {
    AllowDownloads => "allow-downloads" [Downloads],
    AllowForms => "allow-forms" [Forms],
    AllowModals => "allow-modals" [Modals],
    AllowOrientationLock => "allow-orientation-lock" [OrientationLock],
    AllowPointerLock => "allow-pointer-lock" [PointerLock],
    AllowPopups => "allow-popups" [AuxiliaryNavigation, CustomProtocolsNavigation],
    AllowPopupsToEscapeSandbox => "allow-popups-to-escape-sandbox" [PropagatesToAuxiliary],
    AllowPresentation => "allow-presentation" [Presentation],
    AllowSameOrigin => "allow-same-origin" [Origin],
    AllowScripts => "allow-scripts" [Scripts, AutomaticFeatures],
    AllowTopNavigation => "allow-top-navigation" [TopNavigationWithoutUserActivation, TopNavigationWithUserActivation, CustomProtocolsNavigation],
    AllowTopNavigationByUserActivation => "allow-top-navigation-by-user-activation" [TopNavigationWithUserActivation],
    AllowTopNavigationToCustomProtocols => "allow-top-navigation-to-custom-protocols" [CustomProtocolsNavigation],
}

/// One keyword inventory and ASCII comparison kernel serves ordinary reports,
/// author token recognition and bounded flags-only processing.
fn keyword_with<E>(
    units: &[u16],
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<Option<HtmlSandboxKeyword>, E> {
    let mut entries = KEYWORDS.iter();
    loop {
        admit()?;
        let Some(entry) = entries.next() else {
            return Ok(None);
        };
        let length = units.len();
        if length != entry.spelling.len() {
            continue;
        }
        let mut matches = true;
        let mut index = 0;
        while index < length {
            admit()?;
            if !ascii_keyword_unit(units[index], entry.spelling.as_bytes()[index]) {
                matches = false;
                break;
            }
            index += 1;
        }
        if matches {
            return Ok(Some(entry.keyword));
        }
    }
}

/// An author-only redundancy; recognized processing remains unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSandboxRedundancy {
    TopNavigationAndByUserActivation,
    CustomProtocolsAndTopNavigation,
    CustomProtocolsAndPopups,
}

fn redundancies(bits: u16) -> [Option<HtmlSandboxRedundancy>; 3] {
    use HtmlSandboxKeyword::{
        AllowPopups, AllowTopNavigation, AllowTopNavigationByUserActivation,
        AllowTopNavigationToCustomProtocols,
    };
    let has = |keyword: HtmlSandboxKeyword| bits & keyword.bit() != 0;
    [
        (has(AllowTopNavigation) && has(AllowTopNavigationByUserActivation))
            .then_some(HtmlSandboxRedundancy::TopNavigationAndByUserActivation),
        (has(AllowTopNavigationToCustomProtocols) && has(AllowTopNavigation))
            .then_some(HtmlSandboxRedundancy::CustomProtocolsAndTopNavigation),
        (has(AllowTopNavigationToCustomProtocols) && has(AllowPopups))
            .then_some(HtmlSandboxRedundancy::CustomProtocolsAndPopups),
    ]
}

fn project_flags(bits: u16) -> HtmlSandboxFlags {
    let removed = KEYWORDS
        .iter()
        .filter(|entry| bits & entry.keyword.bit() != 0)
        .fold(0, |removed, entry| removed | entry.removes);
    // Exactly sixteen flags occupy this finite representation. Navigation and
    // DocumentDomain are never removed by any directive keyword.
    HtmlSandboxFlags { bits: !removed }
}

/// A conforming native sandbox value, without any active-policy claim.
/// Equality is unordered; serialization retains the chosen keyword order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSandboxValue {
    tokens: HtmlUnorderedTokenSet,
    keywords: u16,
}

impl HtmlSandboxValue {
    /// Rejects duplicate keywords and the source's redundant combinations.
    /// An empty sequence authors a present empty sandbox (all restrictions).
    pub fn new(
        keywords: impl IntoIterator<Item = HtmlSandboxKeyword>,
    ) -> Result<Self, HtmlSandboxValueError> {
        let mut bits = 0;
        let tokens = HtmlUnorderedTokenSet::new(
            HtmlTokenComparison::AsciiCaseInsensitive,
            keywords.into_iter().map(|keyword| {
                bits |= keyword.bit();
                keyword.token()
            }),
        )
        .map_err(HtmlSandboxValueError::Duplicate)?;
        if let Some(redundancy) = redundancies(bits).into_iter().flatten().next() {
            return Err(HtmlSandboxValueError::Redundant(redundancy));
        }
        Ok(Self {
            tokens,
            keywords: bits,
        })
    }
    pub fn tokens(&self) -> &HtmlUnorderedTokenSet {
        &self.tokens
    }
    pub fn flags(&self) -> HtmlSandboxFlags {
        project_flags(self.keywords)
    }
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Sandbox.qualified_name(),
            self.tokens.as_document_string(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSandboxValueError {
    Duplicate(HtmlDuplicateTokenError),
    Redundant(HtmlSandboxRedundancy),
}

impl fmt::Display for HtmlSandboxValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(error) => error.fmt(f),
            Self::Redundant(error) => write!(f, "redundant sandbox keywords: {error:?}"),
        }
    }
}
impl Error for HtmlSandboxValueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Duplicate(error) => Some(error),
            Self::Redundant(_) => None,
        }
    }
}

/// An unrecognized whole token, preserving exact spelling and token position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSandboxUnknownToken {
    token_index: usize,
    value: DocumentString,
}

impl HtmlSandboxUnknownToken {
    pub fn token_index(&self) -> usize {
        self.token_index
    }
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// Independent author violations; none suppresses the parsed flag projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSandboxAuthorDiagnostics {
    duplicate: Option<HtmlDuplicateTokenError>,
    unknown: Vec<HtmlSandboxUnknownToken>,
    redundant: [Option<HtmlSandboxRedundancy>; 3],
}

impl HtmlSandboxAuthorDiagnostics {
    /// The shared unique-token owner reports the first repeated token.
    pub fn duplicate(&self) -> Option<&HtmlDuplicateTokenError> {
        self.duplicate.as_ref()
    }
    pub fn unknown_tokens(&self) -> &[HtmlSandboxUnknownToken] {
        &self.unknown
    }
    /// Each prohibited pair is retained independently, including overlapping pairs.
    pub fn redundancies(&self) -> impl Iterator<Item = HtmlSandboxRedundancy> + '_ {
        self.redundant.iter().flatten().copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSandboxAuthorState {
    Missing,
    Conforming,
    Nonconforming(HtmlSandboxAuthorDiagnostics),
    Unapplied,
}

/// Local content contribution, independent of ancestor/CSP/navigation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSandboxDirectiveState {
    /// No attribute directive; distinct from a parsed empty directive.
    Absent,
    Present(HtmlSandboxFlags),
}

/// Source-qualified sandbox interpretation over an exact attribute record.
///
/// The standalone raw-unit cap applies before names or token scans. It does not
/// promise allocation-failure recovery, work credits or cancellation. Parsed
/// local restrictions are not a loaded document's active sandboxing flag set.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlSandboxReport {
    attribute: HtmlAttributeReport<HtmlSandboxDirectiveState>,
    common: Option<HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>>,
    author: HtmlSandboxAuthorState,
}

impl HtmlSandboxReport {
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    pub fn processing_source_clause(&self) -> &'static str {
        source!("parse-a-sandboxing-directive")
    }
    pub fn flag_set_source_clause(&self) -> &'static str {
        source!("iframe-sandboxing-flag-set")
    }
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>> {
        self.common.as_ref()
    }
    pub fn tokens(&self) -> Option<&HtmlSpaceSeparatedTokens> {
        match self.common.as_ref()?.recovery() {
            HtmlValueRecoveryState::Value(tokens) => Some(tokens),
            _ => None,
        }
    }
    pub fn author(&self) -> &HtmlSandboxAuthorState {
        &self.author
    }
    /// None means genuinely unapplied names, not missing/invalid content.
    pub fn intrinsic(&self) -> Option<&HtmlSandboxDirectiveState> {
        self.attribute.intrinsic()
    }
}

const SANDBOX: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Sandbox,
    rows: &[133],
    source: source!("attr-iframe-sandbox"),
    context: Context::Elements(&[KnownHtmlElement::Iframe]),
    vocabulary_source: None,
};

impl HtmlValueInterpreter {
    /// Interprets iframe sandbox tokens and local restrictions, preserving all
    /// authored spellings and independent duplicate/unknown/redundancy errors.
    /// No navigation, origin fact, active policy or host enforcement is inferred.
    pub fn iframe_sandbox(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlSandboxReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut attribute = report(SANDBOX, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlSandboxReport {
                attribute,
                common: None,
                author: HtmlSandboxAuthorState::Unapplied,
            });
        }
        let common = self.space_separated_tokens(raw.map(Attribute::value))?;
        let tokens = match common.recovery() {
            HtmlValueRecoveryState::Value(tokens) => tokens,
            HtmlValueRecoveryState::Missing => {
                attribute.intrinsic = Some(HtmlSandboxDirectiveState::Absent);
                return Ok(HtmlSandboxReport {
                    attribute,
                    common: Some(common),
                    author: HtmlSandboxAuthorState::Missing,
                });
            }
            HtmlValueRecoveryState::Failed(_) => {
                unreachable!("ASCII whitespace splitting has no failure after raw admission")
            }
        };
        let duplicate = HtmlUnorderedTokenSet::new(
            HtmlTokenComparison::AsciiCaseInsensitive,
            tokens.tokens().iter().cloned(),
        )
        .err();
        let mut keywords = 0;
        let mut unknown = Vec::new();
        for (token_index, token) in tokens.tokens().iter().enumerate() {
            if let Some(keyword) = HtmlSandboxKeyword::from_token(token) {
                keywords |= keyword.bit();
            } else {
                unknown.push(HtmlSandboxUnknownToken {
                    token_index,
                    value: token.value().clone(),
                });
            }
        }
        let redundant = redundancies(keywords);
        let author = if duplicate.is_some()
            || !unknown.is_empty()
            || redundant.iter().any(Option::is_some)
        {
            HtmlSandboxAuthorState::Nonconforming(HtmlSandboxAuthorDiagnostics {
                duplicate,
                unknown,
                redundant,
            })
        } else {
            HtmlSandboxAuthorState::Conforming
        };
        attribute.intrinsic = Some(HtmlSandboxDirectiveState::Present(project_flags(keywords)));
        Ok(HtmlSandboxReport {
            attribute,
            common: Some(common),
            author,
        })
    }
}
