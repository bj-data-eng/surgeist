//! HTML877d link-type interpretation. Resource execution belongs to consumers.

use crate::tokens::{SpaceTokenCursor, SpaceTokenStep};
use crate::{
    Attribute, DocumentString, HtmlLinkDestination, HtmlValueAdmissionError, HtmlValueInterpreter,
    KnownHtmlElement, OrderedAttributes, QualifiedName,
};

/// HTML877d's historical flag, owned by the actual constructed link record.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlLinkFlags {
    explicitly_enabled: bool,
}
impl HtmlLinkFlags {
    pub fn fresh() -> Self {
        Self::default()
    }
    pub fn explicitly_enabled(self) -> bool {
        self.explicitly_enabled
    }
    /// Invoke only for the actual disabled attribute change. Copying attributes
    /// into a new element does not simulate a removal.
    pub fn disabled_attribute_changed(
        &mut self,
        namespace: &crate::Namespace,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
    ) {
        if namespace.uri().is_none() && old.is_some() && new.is_none() {
            self.disabled_removed();
        }
    }
    pub(crate) fn disabled_removed(&mut self) {
        self.explicitly_enabled = true;
    }
}
/// A qualified current-node-document observation, distinct from resource execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlLinkHistory {
    flags: HtmlLinkFlags,
    created_by_node_document_parser: bool,
}
impl HtmlLinkHistory {
    pub fn new(flags: HtmlLinkFlags, created_by_node_document_parser: bool) -> Self {
        Self {
            flags,
            created_by_node_document_parser,
        }
    }
    pub fn flags(self) -> HtmlLinkFlags {
        self.flags
    }
    pub fn created_by_node_document_parser(self) -> bool {
        self.created_by_node_document_parser
    }
}

/// Element roles have distinct applicability; form is not an anchor alias.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelRole {
    Link,
    Anchor,
    Area,
    Form,
}
impl HtmlRelRole {
    pub(crate) fn from_element(element: &QualifiedName) -> Option<Self> {
        Self::from_known(KnownHtmlElement::from_name(element)?)
    }
    pub(crate) fn from_known(element: KnownHtmlElement) -> Option<Self> {
        match element {
            KnownHtmlElement::Link => Some(Self::Link),
            KnownHtmlElement::A => Some(Self::Anchor),
            KnownHtmlElement::Area => Some(Self::Area),
            KnownHtmlElement::Form => Some(Self::Form),
            _ => None,
        }
    }
}

/// The pinned standard keywords; registry extensions do not extend this enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelKeyword {
    Alternate,
    Author,
    Bookmark,
    Canonical,
    DnsPrefetch,
    Expect,
    External,
    Help,
    Icon,
    License,
    Manifest,
    ModulePreload,
    Next,
    NoFollow,
    NoOpener,
    NoReferrer,
    Opener,
    Pingback,
    Preconnect,
    Prefetch,
    Preload,
    Prev,
    PrivacyPolicy,
    Search,
    Stylesheet,
    Tag,
    TermsOfService,
}
impl HtmlRelKeyword {
    pub const ALL: &'static [Self] = &[
        Self::Alternate,
        Self::Author,
        Self::Bookmark,
        Self::Canonical,
        Self::DnsPrefetch,
        Self::Expect,
        Self::External,
        Self::Help,
        Self::Icon,
        Self::License,
        Self::Manifest,
        Self::ModulePreload,
        Self::Next,
        Self::NoFollow,
        Self::NoOpener,
        Self::NoReferrer,
        Self::Opener,
        Self::Pingback,
        Self::Preconnect,
        Self::Prefetch,
        Self::Preload,
        Self::Prev,
        Self::PrivacyPolicy,
        Self::Search,
        Self::Stylesheet,
        Self::Tag,
        Self::TermsOfService,
    ];
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Alternate => "alternate",
            Self::Author => "author",
            Self::Bookmark => "bookmark",
            Self::Canonical => "canonical",
            Self::DnsPrefetch => "dns-prefetch",
            Self::Expect => "expect",
            Self::External => "external",
            Self::Help => "help",
            Self::Icon => "icon",
            Self::License => "license",
            Self::Manifest => "manifest",
            Self::ModulePreload => "modulepreload",
            Self::Next => "next",
            Self::NoFollow => "nofollow",
            Self::NoOpener => "noopener",
            Self::NoReferrer => "noreferrer",
            Self::Opener => "opener",
            Self::Pingback => "pingback",
            Self::Preconnect => "preconnect",
            Self::Prefetch => "prefetch",
            Self::Preload => "preload",
            Self::Prev => "prev",
            Self::PrivacyPolicy => "privacy-policy",
            Self::Search => "search",
            Self::Stylesheet => "stylesheet",
            Self::Tag => "tag",
            Self::TermsOfService => "terms-of-service",
        }
    }
    pub fn body_ok(self) -> bool {
        matches!(
            self,
            Self::DnsPrefetch
                | Self::ModulePreload
                | Self::Pingback
                | Self::Preconnect
                | Self::Prefetch
                | Self::Preload
                | Self::Stylesheet
        )
    }
    /// Possible DOM supported-token set, not an actual host capability assertion.
    pub fn possible_supported_token(self, role: HtmlRelRole) -> bool {
        if role == HtmlRelRole::Link {
            matches!(
                self,
                Self::Alternate
                    | Self::DnsPrefetch
                    | Self::Expect
                    | Self::Icon
                    | Self::Manifest
                    | Self::ModulePreload
                    | Self::Next
                    | Self::Pingback
                    | Self::Preconnect
                    | Self::Prefetch
                    | Self::Preload
                    | Self::Search
                    | Self::Stylesheet
            )
        } else {
            matches!(self, Self::NoOpener | Self::NoReferrer | Self::Opener)
        }
    }
    pub fn effect(self, role: HtmlRelRole) -> Option<HtmlRelEffect> {
        use HtmlRelEffect as E;
        use HtmlRelKeyword as K;
        let anchor = matches!(role, HtmlRelRole::Anchor | HtmlRelRole::Area);
        match self {
            K::Alternate | K::Author | K::PrivacyPolicy | K::TermsOfService
                if role == HtmlRelRole::Link || anchor =>
            {
                Some(E::Hyperlink)
            }
            K::Canonical if role == HtmlRelRole::Link => Some(E::Hyperlink),
            K::Bookmark | K::Tag if anchor => Some(E::Hyperlink),
            K::Help | K::License | K::Next | K::Prev | K::Search => Some(E::Hyperlink),
            K::External | K::NoFollow | K::NoOpener | K::NoReferrer | K::Opener
                if role != HtmlRelRole::Link =>
            {
                Some(E::Annotation)
            }
            K::DnsPrefetch
            | K::Icon
            | K::Manifest
            | K::ModulePreload
            | K::Pingback
            | K::Preconnect
            | K::Prefetch
            | K::Preload
            | K::Stylesheet
                if role == HtmlRelRole::Link =>
            {
                Some(E::ExternalResource)
            }
            K::Expect if role == HtmlRelRole::Link => Some(E::InternalResource),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelEffect {
    Hyperlink,
    ExternalResource,
    InternalResource,
    Annotation,
    StylesheetModifier,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelSpelling {
    Standard,
    ForbiddenSynonym,
    ShortcutException,
    InvalidShortcut,
    Unknown,
    HistoricalRev,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelOccurrenceSource {
    Rel,
    Rev,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelTokenApplicability {
    Allowed,
    NotAllowed,
    ShortcutModifier,
    UncheckedExtension,
}
/// One original authored occurrence; duplicate and spelling findings coexist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelOccurrence {
    pub(crate) source: HtmlRelOccurrenceSource,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) keyword: Option<HtmlRelKeyword>,
    pub(crate) spelling: HtmlRelSpelling,
    pub(crate) duplicate_of: Option<usize>,
    pub(crate) effect: Option<HtmlRelEffect>,
    pub(crate) registry_resolved: bool,
}
impl HtmlRelOccurrence {
    pub fn source(&self) -> HtmlRelOccurrenceSource {
        self.source
    }
    pub fn code_unit_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
    pub fn keyword(&self) -> Option<HtmlRelKeyword> {
        self.keyword
    }
    pub fn spelling(&self) -> HtmlRelSpelling {
        self.spelling
    }
    pub fn duplicate_of(&self) -> Option<usize> {
        self.duplicate_of
    }
    pub fn effect(&self) -> Option<HtmlRelEffect> {
        self.effect
    }
    pub fn applicability(&self) -> HtmlRelTokenApplicability {
        if self.effect.is_some() {
            HtmlRelTokenApplicability::Allowed
        } else if self.spelling == HtmlRelSpelling::ShortcutException {
            HtmlRelTokenApplicability::ShortcutModifier
        } else if self.spelling == HtmlRelSpelling::Unknown && !self.registry_resolved {
            HtmlRelTokenApplicability::UncheckedExtension
        } else {
            HtmlRelTokenApplicability::NotAllowed
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelPresenceAuthor {
    NotRequired,
    Satisfied,
    NeitherRelNorItemprop,
    BothRelAndItemprop,
}
/// The link address-presence author requirement, independent of URL grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelAddressAuthor {
    NotRequired,
    Satisfied,
    MissingHrefAndImagesrcset,
}
/// Local part of link's nonempty URL author requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelHrefAuthor {
    NotRequired,
    EmptyOrOnlyAsciiSpaces,
    GrammarUnchecked,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelLinks {
    None,
    Declared,
    ImpliedHyperlink,
    ExtensionEffectsUnchecked,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelAlternate {
    NotSelected,
    StylesheetModifier,
    Feed,
    Representation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelTitleAuthor {
    NotRequired,
    Satisfied,
    MissingOrEmpty,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelBodyAllowance {
    NotLink,
    Itemprop,
    RelAbsent,
    AllKeywordsBodyOk,
    ContainsNonBodyOkKeyword,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RelData {
    pub(crate) role: Option<HtmlRelRole>,
    pub(crate) rel: Option<DocumentString>,
    pub(crate) rev: Option<DocumentString>,
    pub(crate) occurrences: Vec<HtmlRelOccurrence>,
    pub(crate) presence_author: HtmlRelPresenceAuthor,
    pub(crate) links: HtmlRelLinks,
    pub(crate) alternate: HtmlRelAlternate,
    pub(crate) title_author: HtmlRelTitleAuthor,
    pub(crate) body: HtmlRelBodyAllowance,
    pub(crate) href_present: bool,
    pub(crate) imagesrcset_present: bool,
    pub(crate) address_author: HtmlRelAddressAuthor,
    pub(crate) href_author: HtmlRelHrefAuthor,
    pub(crate) as_destination: Option<HtmlLinkDestination>,
    pub(crate) image_source_preload: bool,
}
/// Local interpretation alone does not certify external grammar, registry or execution.
/// Its single owning record keeps semantic enum payloads bounded without boxing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelReport {
    pub(crate) data: Vec<RelData>,
}
impl HtmlRelReport {
    pub fn role(&self) -> Option<HtmlRelRole> {
        self.data[0].role
    }
    pub fn rel(&self) -> Option<&DocumentString> {
        self.data[0].rel.as_ref()
    }
    pub fn rev(&self) -> Option<&DocumentString> {
        self.data[0].rev.as_ref()
    }
    pub fn occurrences(&self) -> &[HtmlRelOccurrence] {
        &self.data[0].occurrences
    }
    pub fn occurrence_units(&self, index: usize) -> Option<&[u16]> {
        let token = self.occurrences().get(index)?;
        let raw = match token.source {
            HtmlRelOccurrenceSource::Rel => self.rel(),
            HtmlRelOccurrenceSource::Rev => self.rev(),
        }?;
        Some(&raw.code_units()[token.start..token.end])
    }
    pub fn presence_author(&self) -> HtmlRelPresenceAuthor {
        self.data[0].presence_author
    }
    pub fn links(&self) -> HtmlRelLinks {
        self.data[0].links
    }
    pub fn alternate(&self) -> HtmlRelAlternate {
        self.data[0].alternate
    }
    pub fn alternate_stylesheet_title_author(&self) -> HtmlRelTitleAuthor {
        self.data[0].title_author
    }
    pub fn body_allowance(&self) -> HtmlRelBodyAllowance {
        self.data[0].body
    }
    pub fn href_present(&self) -> bool {
        self.data[0].href_present
    }
    pub fn imagesrcset_present(&self) -> bool {
        self.data[0].imagesrcset_present
    }
    pub fn address_presence_author(&self) -> HtmlRelAddressAuthor {
        self.data[0].address_author
    }
    pub fn href_author(&self) -> HtmlRelHrefAuthor {
        self.data[0].href_author
    }
    /// State from the existing canonical as interpreter; no fetch destination.
    pub fn as_destination(&self) -> Option<HtmlLinkDestination> {
        self.data[0].as_destination
    }
    /// Whether an actual imagesrcset slot satisfies its preload/image context.
    /// This does not validate its source-set grammar or select an image resource.
    pub fn image_source_preload_context(&self) -> bool {
        self.data[0].image_source_preload
    }
    pub fn source_clause(&self) -> &'static str {
        concat!(
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/",
            "#linkTypes"
        )
    }
}

pub(crate) trait RelMeter {
    type Error;
    fn work(&mut self, n: usize) -> Result<(), Self::Error>;
    fn reserve<T>(&mut self, values: &mut Vec<T>) -> Result<(), Self::Error>;
}
struct OrdinaryRelMeter;
impl RelMeter for OrdinaryRelMeter {
    type Error = std::convert::Infallible;
    fn work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn reserve<T>(&mut self, values: &mut Vec<T>) -> Result<(), Self::Error> {
        values.reserve_exact(1);
        Ok(())
    }
}
pub(crate) fn fold(unit: u16) -> u16 {
    if (65..=90).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}
pub(crate) fn ascii_keyword(units: &[u16], keyword: &str) -> bool {
    units.len() == keyword.len()
        && units
            .iter()
            .zip(keyword.bytes())
            .all(|(u, b)| fold(*u) == u16::from(b))
}
/// Narrow membership shares the token owner; no full author certificate is implied.
pub(crate) fn contains_ascii_keyword(raw: &DocumentString, keyword: &str) -> bool {
    crate::tokens::space_token_slices(raw).any(|token| ascii_keyword(token, keyword))
}
pub(crate) fn contains(raw: &DocumentString, keyword: HtmlRelKeyword) -> bool {
    contains_ascii_keyword(raw, keyword.keyword())
}
fn keyword_at<C: RelMeter>(units: &[u16], keyword: &str, m: &mut C) -> Result<bool, C::Error> {
    m.work(1)?;
    if units.len() != keyword.len() {
        return Ok(false);
    }
    for (unit, byte) in units.iter().zip(keyword.bytes()) {
        m.work(3)?;
        if fold(*unit) != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn equal_at<C: RelMeter>(a: &[u16], b: &[u16], m: &mut C) -> Result<bool, C::Error> {
    m.work(1)?;
    if a.len() != b.len() {
        return Ok(false);
    }
    for (a, b) in a.iter().zip(b) {
        m.work(3)?;
        if fold(*a) != fold(*b) {
            return Ok(false);
        }
    }
    Ok(true)
}
pub(crate) fn classify<C: RelMeter>(
    units: &[u16],
    m: &mut C,
) -> Result<(Option<HtmlRelKeyword>, HtmlRelSpelling), C::Error> {
    for &keyword in HtmlRelKeyword::ALL {
        m.work(1)?;
        if keyword_at(units, keyword.keyword(), m)? {
            return Ok((Some(keyword), HtmlRelSpelling::Standard));
        }
    }
    for (alias, keyword) in [
        ("copyright", HtmlRelKeyword::License),
        ("previous", HtmlRelKeyword::Prev),
    ] {
        if keyword_at(units, alias, m)? {
            return Ok((Some(keyword), HtmlRelSpelling::ForbiddenSynonym));
        }
    }
    Ok((
        None,
        if keyword_at(units, "shortcut", m)? {
            HtmlRelSpelling::InvalidShortcut
        } else {
            HtmlRelSpelling::Unknown
        },
    ))
}
pub(crate) struct RelInputs<'a> {
    pub(crate) role: Option<HtmlRelRole>,
    pub(crate) rel: Option<&'a DocumentString>,
    pub(crate) rev: Option<&'a DocumentString>,
    pub(crate) href: Option<&'a DocumentString>,
    pub(crate) imagesrcset: Option<&'a DocumentString>,
    pub(crate) itemprop: Option<&'a DocumentString>,
    pub(crate) title: Option<&'a DocumentString>,
    pub(crate) type_hint: Option<&'a DocumentString>,
    pub(crate) as_destination: Option<HtmlLinkDestination>,
}
pub(crate) fn interpret<C: RelMeter>(
    input: RelInputs<'_>,
    m: &mut C,
) -> Result<HtmlRelReport, C::Error> {
    m.work(1)?;
    let mut occurrences: Vec<HtmlRelOccurrence> = Vec::new();
    if input.role.is_some()
        && let Some(raw) = input.rel
    {
        m.work(SpaceTokenCursor::BEGIN_WORK)?;
        let mut cursor = SpaceTokenCursor::new(raw.code_units());
        loop {
            m.work(1)?;
            if cursor.inspects_unit() {
                m.work(1)?;
            }
            m.work(SpaceTokenCursor::STEP_WORK)?;
            match cursor.step() {
                SpaceTokenStep::Continue => {}
                SpaceTokenStep::Done => break,
                SpaceTokenStep::Token { start, end } => {
                    let units = &raw.code_units()[start..end];
                    let (keyword, mut spelling) = classify(units, m)?;
                    if spelling == HtmlRelSpelling::InvalidShortcut
                        && input.role == Some(HtmlRelRole::Link)
                        && keyword_at(raw.code_units(), "shortcut icon", m)?
                    {
                        spelling = HtmlRelSpelling::ShortcutException;
                    }
                    let mut duplicate_of = None;
                    for (index, prior) in occurrences.iter().enumerate() {
                        m.work(1)?;
                        if equal_at(units, &raw.code_units()[prior.start..prior.end], m)? {
                            duplicate_of = Some(index);
                            break;
                        }
                    }
                    m.reserve(&mut occurrences)?;
                    m.work(1)?;
                    occurrences.push(HtmlRelOccurrence {
                        source: HtmlRelOccurrenceSource::Rel,
                        start,
                        end,
                        keyword,
                        spelling,
                        duplicate_of,
                        registry_resolved: false,
                        effect: keyword.and_then(|k| input.role.and_then(|r| k.effect(r))),
                    });
                }
            }
        }
    }
    if matches!(
        input.role,
        Some(HtmlRelRole::Link | HtmlRelRole::Anchor | HtmlRelRole::Area)
    ) && let Some(raw) = input.rev
        && {
            m.work(10)?;
            raw.code_units() == [109, 97, 100, 101]
        }
    {
        m.reserve(&mut occurrences)?;
        m.work(1)?;
        occurrences.push(HtmlRelOccurrence {
            source: HtmlRelOccurrenceSource::Rev,
            start: 0,
            end: raw.len(),
            keyword: Some(HtmlRelKeyword::Author),
            spelling: HtmlRelSpelling::HistoricalRev,
            duplicate_of: None,
            registry_resolved: false,
            effect: Some(HtmlRelEffect::Hyperlink),
        });
    }
    let (links, alternate, presence_author, title_author, body) =
        summarize(&input, &mut occurrences, m)?;
    let address_author = if input.role != Some(HtmlRelRole::Link) {
        HtmlRelAddressAuthor::NotRequired
    } else if input.href.is_some() || input.imagesrcset.is_some() {
        HtmlRelAddressAuthor::Satisfied
    } else {
        HtmlRelAddressAuthor::MissingHrefAndImagesrcset
    };
    let href_author = if let (Some(HtmlRelRole::Link), Some(href)) = (input.role, input.href) {
        let mut nonspace = false;
        for &u in href.code_units() {
            m.work(6)?;
            nonspace |= !matches!(u, 9 | 10 | 12 | 13 | 32);
        }
        if nonspace {
            HtmlRelHrefAuthor::GrammarUnchecked
        } else {
            HtmlRelHrefAuthor::EmptyOrOnlyAsciiSpaces
        }
    } else {
        HtmlRelHrefAuthor::NotRequired
    };
    let image_source_preload = image_source_preload(&input, &occurrences, m)?;
    let mut data = Vec::new();
    m.reserve(&mut data)?;
    m.work(3)?;
    data.push(RelData {
        role: input.role,
        rel: input.rel.cloned(),
        rev: input.rev.cloned(),
        occurrences,
        presence_author,
        links,
        alternate,
        title_author,
        body,
        href_present: input.href.is_some(),
        imagesrcset_present: input.imagesrcset.is_some(),
        address_author,
        href_author,
        as_destination: input.as_destination,
        image_source_preload,
    });
    Ok(HtmlRelReport { data })
}
/// Reuses the native aggregate algorithm after capture-qualified canonical effects.
/// Original raw spelling, occurrence boundaries and author observations are retained.
pub(crate) fn resolve_occurrence(
    report: &mut HtmlRelReport,
    index: usize,
    keyword: Option<HtmlRelKeyword>,
    effect: Option<HtmlRelEffect>,
    resolved: bool,
) {
    let token = &mut report.data[0].occurrences[index];
    token.keyword = keyword;
    token.effect = effect;
    token.registry_resolved = resolved;
}
pub(crate) fn refresh<C: RelMeter>(
    report: &mut HtmlRelReport,
    input: &RelInputs<'_>,
    m: &mut C,
) -> Result<(), C::Error> {
    let data = &mut report.data[0];
    let (links, alternate, presence_author, title_author, body) =
        summarize(input, &mut data.occurrences, m)?;
    m.work(5)?;
    data.links = links;
    data.alternate = alternate;
    data.presence_author = presence_author;
    data.title_author = title_author;
    data.body = body;
    data.image_source_preload = image_source_preload(input, &data.occurrences, m)?;
    Ok(())
}
fn image_source_preload<C: RelMeter>(
    input: &RelInputs<'_>,
    occurrences: &[HtmlRelOccurrence],
    m: &mut C,
) -> Result<bool, C::Error> {
    m.work(3)?;
    if input.role != Some(HtmlRelRole::Link)
        || input.as_destination != Some(HtmlLinkDestination::Image)
        || input.imagesrcset.is_none()
    {
        return Ok(false);
    }
    for token in occurrences {
        m.work(3)?;
        if token.keyword == Some(HtmlRelKeyword::Preload) && token.effect.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}
fn summarize<C: RelMeter>(
    input: &RelInputs<'_>,
    occurrences: &mut [HtmlRelOccurrence],
    m: &mut C,
) -> Result<
    (
        HtmlRelLinks,
        HtmlRelAlternate,
        HtmlRelPresenceAuthor,
        HtmlRelTitleAuthor,
        HtmlRelBodyAllowance,
    ),
    C::Error,
> {
    let mut has_alternate = false;
    let mut has_stylesheet = false;
    let mut has_hyperlink = false;
    let mut unknown = false;
    let mut allowed = false;
    let mut all_body_ok = true;
    for token in occurrences.iter() {
        m.work(12)?; // Fixed predicates over this admitted occurrence; no raw scan.
        has_alternate |= token.keyword == Some(HtmlRelKeyword::Alternate) && token.effect.is_some();
        has_stylesheet |=
            token.keyword == Some(HtmlRelKeyword::Stylesheet) && token.effect.is_some();
        unknown |= token.spelling == HtmlRelSpelling::Unknown && !token.registry_resolved;
        if token.source == HtmlRelOccurrenceSource::Rel {
            all_body_ok &= token.keyword.is_some_and(HtmlRelKeyword::body_ok);
        }
        allowed |= token.effect.is_some();
    }
    m.work(44)?; // Both fixed feed spellings, including their terminating comparisons.
    let alternate = if !has_alternate {
        HtmlRelAlternate::NotSelected
    } else if input.role == Some(HtmlRelRole::Link) && has_stylesheet {
        HtmlRelAlternate::StylesheetModifier
    } else if input.type_hint.is_some_and(is_exact_feed_type) {
        HtmlRelAlternate::Feed
    } else {
        HtmlRelAlternate::Representation
    };
    for token in occurrences.iter_mut() {
        m.work(5)?;
        if token.keyword == Some(HtmlRelKeyword::Alternate)
            && alternate == HtmlRelAlternate::StylesheetModifier
        {
            token.effect = Some(HtmlRelEffect::StylesheetModifier)
        }
        has_hyperlink |= token.effect == Some(HtmlRelEffect::Hyperlink);
    }
    let links = match input.role {
        None => HtmlRelLinks::None,
        Some(HtmlRelRole::Link) if input.href.is_none() && input.imagesrcset.is_none() => {
            HtmlRelLinks::None
        }
        Some(HtmlRelRole::Anchor | HtmlRelRole::Area) if input.href.is_none() => HtmlRelLinks::None,
        Some(HtmlRelRole::Link) if !allowed => {
            if unknown {
                HtmlRelLinks::ExtensionEffectsUnchecked
            } else {
                HtmlRelLinks::None
            }
        }
        Some(HtmlRelRole::Link) => HtmlRelLinks::Declared,
        Some(_) if has_hyperlink => HtmlRelLinks::Declared,
        Some(_) if unknown => HtmlRelLinks::ExtensionEffectsUnchecked,
        Some(_) => HtmlRelLinks::ImpliedHyperlink,
    };
    let presence_author = if input.role != Some(HtmlRelRole::Link) {
        HtmlRelPresenceAuthor::NotRequired
    } else {
        match (input.rel.is_some(), input.itemprop.is_some()) {
            (false, false) => HtmlRelPresenceAuthor::NeitherRelNorItemprop,
            (true, true) => HtmlRelPresenceAuthor::BothRelAndItemprop,
            _ => HtmlRelPresenceAuthor::Satisfied,
        }
    };
    let title_author = if alternate != HtmlRelAlternate::StylesheetModifier {
        HtmlRelTitleAuthor::NotRequired
    } else if input.title.is_some_and(|t| !t.is_empty()) {
        HtmlRelTitleAuthor::Satisfied
    } else {
        HtmlRelTitleAuthor::MissingOrEmpty
    };
    let body = if input.role != Some(HtmlRelRole::Link) {
        HtmlRelBodyAllowance::NotLink
    } else if input.itemprop.is_some() {
        HtmlRelBodyAllowance::Itemprop
    } else if input.rel.is_none() {
        HtmlRelBodyAllowance::RelAbsent
    } else if all_body_ok {
        HtmlRelBodyAllowance::AllKeywordsBodyOk
    } else {
        HtmlRelBodyAllowance::ContainsNonBodyOkKeyword
    };
    Ok((links, alternate, presence_author, title_author, body))
}
pub(crate) fn is_exact_feed_type(raw: &DocumentString) -> bool {
    raw.code_units()
        .iter()
        .copied()
        .eq("application/rss+xml".encode_utf16())
        || raw
            .code_units()
            .iter()
            .copied()
            .eq("application/atom+xml".encode_utf16())
}
impl HtmlValueInterpreter {
    /// Ordinary local values with per-original-attribute admission. No contextual,
    /// registry, aggregate memory/work or cancellation certificate is supplied.
    pub fn rel_with_attributes(
        &self,
        element: &QualifiedName,
        attributes: &OrderedAttributes,
    ) -> Result<HtmlRelReport, HtmlValueAdmissionError> {
        let get = |name: &str| {
            attributes
                .iter()
                .find(|a| {
                    a.name().namespace().uri().is_none()
                        && a.name()
                            .local_name()
                            .code_units()
                            .iter()
                            .copied()
                            .eq(name.bytes().map(u16::from))
                })
                .map(Attribute::value)
        };
        let as_name = crate::KnownHtmlAttribute::As.qualified_name();
        let as_attribute = attributes.get(as_name.namespace(), as_name.local_name());
        self.admit_attribute_raw(as_attribute.map(Attribute::value))?;
        let mut input = RelInputs {
            role: None,
            rel: get("rel"),
            rev: get("rev"),
            href: get("href"),
            imagesrcset: get("imagesrcset"),
            itemprop: get("itemprop"),
            title: get("title"),
            type_hint: get("type"),
            as_destination: None,
        };
        for raw in [
            input.rel,
            input.rev,
            input.href,
            input.imagesrcset,
            input.itemprop,
            input.title,
            input.type_hint,
        ] {
            self.admit_attribute_raw(raw)?;
        }
        input.role = HtmlRelRole::from_element(element);
        input.as_destination = self.link_as(element, as_attribute)?.intrinsic().copied();
        match interpret(input, &mut OrdinaryRelMeter) {
            Ok(value) => Ok(value),
            Err(never) => match never {},
        }
    }
}
