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
    ($($(#[$meta:meta])* $variant:ident => $anchor:literal),+ $(,)?) => {
        /// The sixteen source-defined restrictions in a sandboxing flag set (HTML877d #sandboxing).
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum HtmlSandboxFlag { $($(#[$meta])* $variant),+ }

        const ALL_FLAGS: &[HtmlSandboxFlag] = &[$(HtmlSandboxFlag::$variant),+];

        impl HtmlSandboxFlag {
            const fn bit(self) -> u16 { 1 << (self as u8) }

            /// The URL of this flag's definition in the pinned HTML Standard snapshot.
            pub fn source_clause(self) -> &'static str {
                match self { $(Self::$variant => source!($anchor)),+ }
            }
        }
    };
}

sandbox_flags! {
    /// Prevents content from navigating browsing contexts other than the sandboxed one and those
    /// nested inside it. Auxiliary and top-level contexts are also covered by the next three flags.
    Navigation => "sandboxed-navigation-browsing-context-flag",
    /// Prevents content from creating auxiliary browsing contexts, for example through `target` or
    /// `window.open()`.
    AuxiliaryNavigation => "sandboxed-auxiliary-navigation-browsing-context-flag",
    /// Prevents content from navigating or closing its top-level browsing context. It is consulted
    /// only when the active window lacks transient activation.
    TopNavigationWithoutUserActivation => "sandboxed-top-level-navigation-without-user-activation-browsing-context-flag",
    /// Prevents content from navigating or closing its top-level browsing context. It is consulted
    /// only when the active window has transient activation.
    TopNavigationWithUserActivation => "sandboxed-top-level-navigation-with-user-activation-browsing-context-flag",
    /// Forces content into an opaque origin, which also blocks `document.cookie` and
    /// `localStorage`.
    Origin => "sandboxed-origin-browsing-context-flag",
    /// Blocks form submission.
    Forms => "sandboxed-forms-browsing-context-flag",
    /// Disables the Pointer Lock API.
    PointerLock => "sandboxed-pointer-lock-browsing-context-flag",
    /// Blocks script execution.
    Scripts => "sandboxed-scripts-browsing-context-flag",
    /// Blocks features that trigger automatically, such as autoplaying video or autofocusing a form
    /// control.
    AutomaticFeatures => "sandboxed-automatic-features-browsing-context-flag",
    /// Prevents content from using the `document.domain` setter. No sandbox keyword relaxes it.
    DocumentDomain => "sandboxed-document.domain-browsing-context-flag",
    /// Makes every auxiliary browsing context that the content creates inherit the content's active
    /// sandboxing flag set.
    PropagatesToAuxiliary => "sandbox-propagates-to-auxiliary-browsing-contexts-flag",
    /// Prevents modal dialogs from `alert()`, `confirm()`, `print()`, `prompt()` and the
    /// `beforeunload` event.
    Modals => "sandboxed-modals-flag",
    /// Disables locking the screen orientation.
    OrientationLock => "sandboxed-orientation-lock-browsing-context-flag",
    /// Disables the Presentation API.
    Presentation => "sandboxed-presentation-browsing-context-flag",
    /// Prevents content from initiating downloads, through hyperlinks or through navigation handled
    /// as a download.
    Downloads => "sandboxed-downloads-browsing-context-flag",
    /// Prevents navigations to non-fetch schemes from being handed to external software.
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
        let mut cursor = crate::internal::microsyntax::tokens::SpaceTokenCursor::new(units);
        let mut keywords = 0;
        loop {
            admit()?;
            match cursor.step() {
                crate::internal::microsyntax::tokens::SpaceTokenStep::Continue => {}
                crate::internal::microsyntax::tokens::SpaceTokenStep::Token { start, end } => {
                    admit()?;
                    if let Some(keyword) = keyword_with(&units[start..end], &mut admit)? {
                        keywords |= keyword.bit();
                    }
                }
                crate::internal::microsyntax::tokens::SpaceTokenStep::Done => break,
            }
        }
        admit()?;
        Ok(project_flags(keywords))
    }
    /// Whether `flag` is in the set.
    pub fn contains(&self, flag: HtmlSandboxFlag) -> bool {
        self.bits & flag.bit() != 0
    }
    /// The flags in the set, in `HtmlSandboxFlag` declaration order.
    pub fn iter(&self) -> impl Iterator<Item = HtmlSandboxFlag> + '_ {
        ALL_FLAGS
            .iter()
            .copied()
            .filter(|flag| self.contains(*flag))
    }
    /// The number of flags in the set, from 0 to 16.
    pub fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
    /// Whether the set holds no restriction.
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
    ($($(#[$meta:meta])* $variant:ident => $spelling:literal [$first:ident $(, $flag:ident)*]),+ $(,)?) => {
        /// The thirteen allowed keywords of the selected HTML sandbox edition (HTML877d
        /// #the-iframe-element).
        ///
        /// Each keyword relaxes the restriction flags listed in its variant documentation.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum HtmlSandboxKeyword { $($(#[$meta])* $variant),+ }

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

            /// The keyword's exact lowercase spelling.
            pub fn keyword(self) -> &'static str {
                KEYWORDS[self as usize].spelling
            }
            /// The URL of this keyword's definition in the pinned HTML Standard snapshot.
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
    /// Relaxes `Downloads`.
    AllowDownloads => "allow-downloads" [Downloads],
    /// Relaxes `Forms`.
    AllowForms => "allow-forms" [Forms],
    /// Relaxes `Modals`.
    AllowModals => "allow-modals" [Modals],
    /// Relaxes `OrientationLock`.
    AllowOrientationLock => "allow-orientation-lock" [OrientationLock],
    /// Relaxes `PointerLock`.
    AllowPointerLock => "allow-pointer-lock" [PointerLock],
    /// Relaxes `AuxiliaryNavigation` and `CustomProtocolsNavigation`.
    AllowPopups => "allow-popups" [AuxiliaryNavigation, CustomProtocolsNavigation],
    /// Relaxes `PropagatesToAuxiliary`, so popups do not inherit the sandbox.
    AllowPopupsToEscapeSandbox => "allow-popups-to-escape-sandbox" [PropagatesToAuxiliary],
    /// Relaxes `Presentation`.
    AllowPresentation => "allow-presentation" [Presentation],
    /// Relaxes `Origin`, keeping the content's real origin.
    AllowSameOrigin => "allow-same-origin" [Origin],
    /// Relaxes `Scripts` and `AutomaticFeatures`.
    AllowScripts => "allow-scripts" [Scripts, AutomaticFeatures],
    /// Relaxes `TopNavigationWithoutUserActivation`, `TopNavigationWithUserActivation` and
    /// `CustomProtocolsNavigation`.
    AllowTopNavigation => "allow-top-navigation" [TopNavigationWithoutUserActivation, TopNavigationWithUserActivation, CustomProtocolsNavigation],
    /// Relaxes `TopNavigationWithUserActivation`.
    AllowTopNavigationByUserActivation => "allow-top-navigation-by-user-activation" [TopNavigationWithUserActivation],
    /// Relaxes `CustomProtocolsNavigation`.
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
            if !ascii_case_insensitive_unit(units[index], entry.spelling.as_bytes()[index]) {
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
    /// Both `allow-top-navigation` and `allow-top-navigation-by-user-activation` are listed. The
    /// standard forbids the pair because the first already implies the second (HTML877d
    /// #the-iframe-element).
    TopNavigationAndByUserActivation,
    /// Both `allow-top-navigation-to-custom-protocols` and `allow-top-navigation` are listed. The
    /// standard forbids the pair because the second already relaxes the custom-protocols
    /// restriction.
    CustomProtocolsAndTopNavigation,
    /// Both `allow-top-navigation-to-custom-protocols` and `allow-popups` are listed. The standard
    /// forbids the pair because the second already relaxes the custom-protocols restriction.
    CustomProtocolsAndPopups,
}

impl fmt::Display for HtmlSandboxRedundancy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TopNavigationAndByUserActivation => {
                "sandbox lists both allow-top-navigation and allow-top-navigation-by-user-activation, which is redundant"
            }
            Self::CustomProtocolsAndTopNavigation => {
                "sandbox lists both allow-top-navigation-to-custom-protocols and allow-top-navigation, which is redundant"
            }
            Self::CustomProtocolsAndPopups => {
                "sandbox lists both allow-top-navigation-to-custom-protocols and allow-popups, which is redundant"
            }
        })
    }
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
        )?;
        if let Some(redundancy) = redundancies(bits).into_iter().flatten().next() {
            return Err(HtmlSandboxValueError::Redundant(redundancy));
        }
        Ok(Self {
            tokens,
            keywords: bits,
        })
    }
    /// The keywords in their authored order. Uniqueness is judged ASCII case-insensitively, and
    /// each token is the canonical lowercase keyword spelling.
    pub fn tokens(&self) -> &HtmlUnorderedTokenSet {
        &self.tokens
    }
    /// The restrictions this value imposes: every flag except those that the listed keywords relax.
    pub fn flags(&self) -> HtmlSandboxFlags {
        project_flags(self.keywords)
    }
    /// Emits a no-namespace `sandbox` attribute whose value is the keyword tokens in authored
    /// order, separated by single spaces. An empty keyword list emits a present empty value.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Sandbox.qualified_name(),
            self.tokens.as_document_string(),
        )
    }
}

/// Why a keyword list cannot form a conforming sandbox value. The enum is `#[non_exhaustive]`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSandboxValueError {
    /// Two keywords were identical when compared ASCII case-insensitively; the payload names the
    /// repeat.
    Duplicate(HtmlDuplicateTokenError),
    /// The list contains a combination that the standard forbids as redundant; the payload names
    /// the first such combination.
    Redundant(HtmlSandboxRedundancy),
}

impl fmt::Display for HtmlSandboxValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(_) => f.write_str("sandbox value contains a duplicate keyword"),
            Self::Redundant(error) => error.fmt(f),
        }
    }
}
impl From<HtmlDuplicateTokenError> for HtmlSandboxValueError {
    fn from(error: HtmlDuplicateTokenError) -> Self {
        Self::Duplicate(error)
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
    /// The zero-based position of this token among all whitespace-separated tokens of the
    /// attribute, recognized keywords included.
    pub fn token_index(&self) -> usize {
        self.token_index
    }
    /// The token's exact spelling in UTF-16 code units, without case folding.
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
    /// The tokens that match no sandbox keyword, in source order. Their presence makes the value
    /// nonconforming but does not change the parsed flags.
    pub fn unknown_tokens(&self) -> &[HtmlSandboxUnknownToken] {
        &self.unknown
    }
    /// Each prohibited pair is retained independently, including overlapping pairs.
    pub fn redundancies(&self) -> impl Iterator<Item = HtmlSandboxRedundancy> + '_ {
        self.redundant.iter().flatten().copied()
    }
}

/// The author-conformance verdict for the `sandbox` attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSandboxAuthorState {
    /// The attribute is absent, which is conforming and imposes no directive.
    Missing,
    /// The attribute is present and every token is a recognized keyword, with no repeats and no
    /// forbidden combination.
    Conforming,
    /// The attribute is present but breaks one or more author rules; the payload lists every
    /// violation independently.
    Nonconforming(HtmlSandboxAuthorDiagnostics),
    /// The rule does not apply because the element is not an HTML `iframe` or the supplied
    /// attribute is not `sandbox`.
    Unapplied,
}

/// Local content contribution, independent of ancestor/CSP/navigation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSandboxDirectiveState {
    /// No attribute directive; distinct from a parsed empty directive.
    Absent,
    /// A directive is present; the payload is its flag set. A present empty value yields every
    /// flag.
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
    /// The exact element name that was supplied, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    /// The exact supplied attribute (name, prefix and value), or `None` when it was absent.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    /// The exact raw value in UTF-16 code units, distinguishing an absent attribute from an empty
    /// one.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    /// Whether the attribute was absent, present and empty, or present and nonempty.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    /// Whether the attribute's rule applies to the supplied element.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    /// The standard attribute this report is about, `sandbox`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    /// The rows of the pinned specification's attribute index that this rule reproduces.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    /// The URL of the attribute's definition in the pinned HTML Standard snapshot.
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    /// The URL of the pinned standard's algorithm for parsing a sandboxing directive.
    pub fn processing_source_clause(&self) -> &'static str {
        source!("parse-a-sandboxing-directive")
    }
    /// The URL of the pinned standard's definition of the iframe sandboxing flag set.
    pub fn flag_set_source_clause(&self) -> &'static str {
        source!("iframe-sandboxing-flag-set")
    }
    /// The length of the raw value in UTF-16 code units, as admitted against the per-value cap.
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// The shared space-separated token report for the raw value, or `None` when the rule does not
    /// apply.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>> {
        self.common.as_ref()
    }
    /// The recovered tokens, split on ASCII whitespace with every spelling preserved, or `None`
    /// when the rule does not apply or the attribute is absent.
    pub fn tokens(&self) -> Option<&HtmlSpaceSeparatedTokens> {
        match self.common.as_ref()?.recovery() {
            HtmlValueRecoveryState::Value(tokens) => Some(tokens),
            _ => None,
        }
    }
    /// The author verdict. It is independent of the parsed flag projection.
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlSandboxReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut attribute = report(SANDBOX, element, raw);
        if attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlSandboxReport {
                attribute,
                common: None,
                author: HtmlSandboxAuthorState::Unapplied,
            });
        }
        let common = self.space_separated_tokens(raw.map(HtmlAttributeInput::value))?;
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
