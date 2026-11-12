//! HTML877d link-type interpretation. Resource execution belongs to consumers.

use crate::internal::microsyntax::tokens::{
    DeclaredKeyword, HtmlSpaceToken, HtmlSpaceTokenCursor, HtmlSpaceTokenStep,
    ascii_case_insensitive_unit, ascii_lowercase, declared_keyword, is_ascii_whitespace,
};
pub(crate) mod controlled;
use crate::internal::values::semantic_input::{attributes_iter, find_attribute, view_value};
use crate::{
    internal::attributes::link_destination::HtmlLinkDestination, internal::values::DocumentString,
    internal::values::interpretation::HtmlValueInterpretationError,
    internal::values::interpretation::HtmlValueInterpreter, vocabulary::KnownHtmlElement,
};
use crate::{
    internal::values::semantic_input::HtmlAttributeInput,
    internal::values::semantic_input::HtmlNameInput,
};
use controlled::HtmlRelInputs;
use std::cmp::Ordering;

/// HTML877d's historical flag, owned by the actual constructed link record.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlLinkFlags {
    explicitly_enabled: bool,
}
impl HtmlLinkFlags {
    /// Creates the initial flags, in which the link has not been explicitly enabled (HTML877d
    /// #the-link-element: the boolean is initially false).
    pub fn fresh() -> Self {
        Self::default()
    }
    /// Whether the `disabled` attribute has ever been removed from the link. Once true it stays
    /// true; the stylesheet's alternate flag depends on it (HTML877d #the-link-element).
    pub fn explicitly_enabled(self) -> bool {
        self.explicitly_enabled
    }
    /// Invoke only for the actual disabled attribute change, with the values' UTF-16 code units.
    /// Copying attributes into a new element does not simulate a removal.
    pub fn disabled_attribute_changed(
        &mut self,
        namespace: &crate::internal::values::Namespace,
        old: Option<&[u16]>,
        new: Option<&[u16]>,
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
    /// Pairs the link's flags with whether the link's node document's parser created it. The
    /// constructor checks nothing: both facts come from the owner of the actual link record.
    pub fn new(flags: HtmlLinkFlags, created_by_node_document_parser: bool) -> Self {
        Self {
            flags,
            created_by_node_document_parser,
        }
    }
    /// The link's flags as recorded in this history.
    pub fn flags(self) -> HtmlLinkFlags {
        self.flags
    }
    /// Whether the link's node document's parser created the link element, as recorded by the
    /// history's owner.
    pub fn created_by_node_document_parser(self) -> bool {
        self.created_by_node_document_parser
    }
}

/// Element roles have distinct applicability; form is not an anchor alias.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelRole {
    /// A `link` element, the only role on which keywords such as `canonical`, `icon` and
    /// `stylesheet` import or describe resources.
    Link,
    /// An `a` element.
    Anchor,
    /// An `area` element. Its keywords apply as they do for `a`.
    Area,
    /// A `form` element. It takes only the annotation keywords and the hyperlink keywords that the
    /// standard allows on `form`.
    Form,
}
impl HtmlRelRole {
    /// Maps an element name to its role, or `None` when the name is not an HTML `link`, `a`, `area`
    /// or `form` element, in which case no `rel` interpretation applies.
    pub fn from_element(
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    ) -> Option<Self> {
        Self::from_known(KnownHtmlElement::from_name(element)?)
    }
    /// Maps a known HTML element to its role, or `None` for every element other than `link`, `a`,
    /// `area` and `form`.
    pub fn from_known(element: KnownHtmlElement) -> Option<Self> {
        match element {
            KnownHtmlElement::Link => Some(Self::Link),
            KnownHtmlElement::A => Some(Self::Anchor),
            KnownHtmlElement::Area => Some(Self::Area),
            KnownHtmlElement::Form => Some(Self::Form),
            _ => None,
        }
    }
}

/// Declares the pinned standard keywords from one table. The enum, `ALL`, the spelling and the
/// space token each come from this table, so no keyword can be missing from one of them, and the
/// compiler checks that every spelling is one space token.
macro_rules! rel_keywords {
    ($(#[$meta:meta])* enum $name:ident {
        $($(#[$doc:meta])* $variant:ident => $spelling:literal,)+
    }) => {
        $(#[$meta])*
        pub enum $name {
            $($(#[$doc])* $variant,)+
        }
        impl $name {
            /// Every standard keyword exactly once, in declaration order, which is alphabetical by
            /// spelling.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];
            /// The keyword's lowercase spelling. Documents compare keywords ASCII case-insensitively, so
            /// this is the canonical spelling, not the only accepted one (HTML877d #linkTypes).
            pub fn keyword(self) -> &'static str {
                self.declared().as_str()
            }
            /// The keyword's canonical spelling as a space token.
            pub fn space_token(self) -> HtmlSpaceToken {
                HtmlSpaceToken::declared(self.declared())
            }
            fn declared(self) -> DeclaredKeyword {
                match self {
                    $(Self::$variant => declared_keyword!($spelling),)+
                }
            }
        }
    };
}

rel_keywords! {
    /// The pinned standard keywords; registry extensions do not extend this enum.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlRelKeyword {
        /// `alternate` (HTML877d #rel-alternate): alternate representations of the current document,
        /// or, with `stylesheet` on a `link`, an alternative style sheet.
        Alternate => "alternate",
        /// `author` (HTML877d #link-type-author): a link to the author of the current document or of
        /// the nearest `article` ancestor. A `rev` attribute value of `made` counts as this keyword.
        Author => "author",
        /// `bookmark` (HTML877d #link-type-bookmark): the permalink for the nearest ancestor section;
        /// allowed on `a` and `area` only.
        Bookmark => "bookmark",
        /// `canonical` (HTML877d #link-type-canonical): the preferred URL for the current document;
        /// allowed on `link` only.
        Canonical => "canonical",
        /// `dns-prefetch` (HTML877d #link-type-dns-prefetch): preemptive DNS resolution for the target
        /// resource's origin; body-ok.
        DnsPrefetch => "dns-prefetch",
        /// `expect` (HTML877d #link-type-expect): expects an element with the target ID to appear in
        /// the current document; an internal resource link on `link` only.
        Expect => "expect",
        /// `external` (HTML877d #link-type-external): the referenced document is not part of the same
        /// site; an annotation on `a`, `area` and `form`.
        External => "external",
        /// `help` (HTML877d #link-type-help): context-sensitive help.
        Help => "help",
        /// `icon` (HTML877d #rel-icon): an icon that represents the current document; `link` only. The
        /// historical `shortcut icon` form is classified separately.
        Icon => "icon",
        /// `license` (HTML877d #link-type-license): the main content of the current document is covered
        /// by the license the referenced document describes. `copyright` is a forbidden synonym.
        License => "license",
        /// `manifest` (HTML877d #link-type-manifest): an application manifest; `link` only.
        Manifest => "manifest",
        /// `modulepreload` (HTML877d #link-type-modulepreload): preemptively fetches a module script
        /// into the document's module map; body-ok.
        ModulePreload => "modulepreload",
        /// `next` (HTML877d #link-type-next): the referenced document is the next one in a series that
        /// includes the current document.
        Next => "next",
        /// `nofollow` (HTML877d #link-type-nofollow): the current document's author or publisher does
        /// not endorse the referenced document; an annotation.
        NoFollow => "nofollow",
        /// `noopener` (HTML877d #link-type-noopener): creates a top-level traversable with a
        /// non-auxiliary browsing context when the hyperlink would otherwise create an auxiliary one;
        /// an annotation.
        NoOpener => "noopener",
        /// `noreferrer` (HTML877d #link-type-noreferrer): sends no `Referer` header and has the effect
        /// of `noopener`; an annotation.
        NoReferrer => "noreferrer",
        /// `opener` (HTML877d #link-type-opener): creates an auxiliary browsing context when the
        /// hyperlink would otherwise create a non-auxiliary top-level traversable; an annotation.
        Opener => "opener",
        /// `pingback` (HTML877d #link-type-pingback): the address of the pingback server for the
        /// current document; body-ok.
        Pingback => "pingback",
        /// `preconnect` (HTML877d #link-type-preconnect): preemptively connects to the target
        /// resource's origin; body-ok.
        Preconnect => "preconnect",
        /// `prefetch` (HTML877d #link-type-prefetch): preemptively fetches and caches the target for a
        /// likely follow-up navigation; body-ok.
        Prefetch => "prefetch",
        /// `preload` (HTML877d #link-type-preload): preemptively fetches and caches the target for the
        /// current navigation according to the preload destination of the `as` attribute; body-ok.
        Preload => "preload",
        /// `prev` (HTML877d #link-type-prev): the referenced document is the previous one in a series
        /// that includes the current document. `previous` is a forbidden synonym.
        Prev => "prev",
        /// `privacy-policy` (HTML877d #link-type-privacy-policy): information about the data collection
        /// and usage practices that apply to the current document.
        PrivacyPolicy => "privacy-policy",
        /// `search` (HTML877d #link-type-search): a resource that can search the current document and
        /// its related pages.
        Search => "search",
        /// `stylesheet` (HTML877d #link-type-stylesheet): imports a style sheet; body-ok.
        Stylesheet => "stylesheet",
        /// `tag` (HTML877d #link-type-tag): a tag, identified by the given address, that applies to the
        /// current document; allowed on `a` and `area` only.
        Tag => "tag",
        /// `terms-of-service` (HTML877d #link-type-terms-of-service): the agreements between the
        /// current document's provider and its users.
        TermsOfService => "terms-of-service",
    }
}
impl HtmlRelKeyword {
    /// Whether the keyword is body-ok: the standard lists exactly `dns-prefetch`, `modulepreload`,
    /// `pingback`, `preconnect`, `prefetch`, `preload` and `stylesheet`, and a `link` carrying only
    /// such keywords is allowed in the body (HTML877d #linkTypes).
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
    /// The effect the link-type table of HTML877d #linkTypes assigns this keyword on an element of
    /// `role`, or `None` where the table says the keyword is not allowed there. This never returns
    /// `StylesheetModifier`, which interpretation assigns to `alternate` beside `stylesheet`; it
    /// also does not consult the registry for extensions.
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
/// What a recognised link type does to the element that carries it (HTML877d #introduction-2 and
/// the table in HTML877d #linkTypes). The type names the effect and leaves fetching, navigation and
/// annotation processing to consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelEffect {
    /// The keyword creates a hyperlink to the referenced document.
    Hyperlink,
    /// The keyword creates a link to an external resource that the document imports or prepares,
    /// such as a style sheet or an icon.
    ExternalResource,
    /// The keyword creates a link to an internal resource, a part of the current document; only
    /// `expect` does.
    InternalResource,
    /// The keyword annotates a hyperlink, changing how it is processed, instead of creating a link
    /// of its own.
    Annotation,
    /// `alternate` beside `stylesheet` on a `link` modifies that style sheet to be an alternative
    /// style sheet and does not create a hyperlink (HTML877d #link-type-stylesheet).
    StylesheetModifier,
}
/// How a `rel` token (or the historical `rev` value) is spelled relative to the standard's keyword
/// list. Classification is ASCII case-insensitive and never allocates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelSpelling {
    /// The token matches a standard keyword.
    Standard,
    /// The token is `copyright` or `previous`, synonyms the standard requires user agents to handle
    /// as `license` and `prev` but forbids authors to use (HTML877d #linkTypes).
    ForbiddenSynonym,
    /// The token is `shortcut` on a `link` whose whole `rel` value is `shortcut icon`, the
    /// historical spelling that the standard allows before `icon` (HTML877d #rel-icon).
    ShortcutException,
    /// The token is `shortcut` but the exception does not hold, because the element is not a `link`
    /// or the whole value is not `shortcut icon`.
    InvalidShortcut,
    /// The token is neither a standard keyword nor a listed synonym. It may be a registered
    /// extension; resolving that needs a registry, which this crate does not hold.
    Unknown,
    /// The occurrence is not a token: it stands for a `rev` attribute whose value is exactly
    /// `made`, which the standard treats as the `author` keyword (HTML877d #link-type-author).
    HistoricalRev,
}
/// The attribute an occurrence came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelOccurrenceSource {
    /// A token of the `rel` attribute.
    Rel,
    /// The historical `rev` attribute, recorded only for the value `made`; the occurrence spans the
    /// entire value.
    Rev,
}
/// Whether a token's keyword may be used on the element's role, as the table of HTML877d #linkTypes
/// decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelTokenApplicability {
    /// The token has an effect on this role.
    Allowed,
    /// The token is a standard keyword the standard does not allow on this role, an invalid
    /// `shortcut`, or an unknown token that a registry resolved as having no effect. Authors must
    /// not use it.
    NotAllowed,
    /// The token is the permitted `shortcut` that precedes `icon`; it has no effect of its own.
    ShortcutModifier,
    /// The token is unknown and no registry has resolved it, so whether it is an allowed extension
    /// is undetermined.
    UncheckedExtension,
}
/// One original authored occurrence; duplicate and spelling findings coexist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelOccurrence {
    source: HtmlRelOccurrenceSource,
    start: usize,
    end: usize,
    keyword: Option<HtmlRelKeyword>,
    spelling: HtmlRelSpelling,
    duplicate_of: Option<usize>,
    effect: Option<HtmlRelEffect>,
    registry_resolved: bool,
}
impl HtmlRelOccurrence {
    /// Whether a registry refinement has resolved this token. It is false until refinement and
    /// stays false for a refinement of `Unavailable`.
    pub fn registry_resolved(&self) -> bool {
        self.registry_resolved
    }
    /// The attribute that supplied this occurrence.
    pub fn source(&self) -> HtmlRelOccurrenceSource {
        self.source
    }
    /// The half-open range of UTF-16 code-unit offsets of the token within the source attribute's
    /// exact value, suitable for `HtmlRelReport::occurrence_units`. A `rev` occurrence spans the
    /// whole value.
    pub fn code_unit_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
    /// The standard keyword this token denotes, including through a forbidden synonym, or `None`
    /// when it is unknown, a `shortcut` (valid or invalid), or resolved as an extension.
    pub fn keyword(&self) -> Option<HtmlRelKeyword> {
        self.keyword
    }
    /// How the token is spelled relative to the standard's keywords.
    pub fn spelling(&self) -> HtmlRelSpelling {
        self.spelling
    }
    /// The index of the earlier occurrence that this token repeats under ASCII case-insensitive
    /// comparison, or `None` for the first spelling. The standard forbids repeating a keyword
    /// within one `rel` (HTML877d #linkTypes), and this finding coexists with the spelling and
    /// effect findings.
    pub fn duplicate_of(&self) -> Option<usize> {
        self.duplicate_of
    }
    /// The effect of the token on its element's role, or `None` when the token has none.
    /// `alternate` beside `stylesheet` on a `link` reports `StylesheetModifier`.
    pub fn effect(&self) -> Option<HtmlRelEffect> {
        self.effect
    }
    /// Whether the token may be used on its element's role: `Allowed` when it has an effect,
    /// `ShortcutModifier` for the permitted `shortcut`, `UncheckedExtension` for an unresolved
    /// unknown token, and `NotAllowed` otherwise.
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
/// The author requirement that a `link` has exactly one of `rel` and `itemprop` (HTML877d
/// #the-link-element). Elements other than `link` have no such requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelPresenceAuthor {
    /// The element is not a `link`.
    NotRequired,
    /// The `link` has exactly one of `rel` and `itemprop`.
    Satisfied,
    /// The `link` has neither attribute, which the standard does not allow.
    NeitherRelNorItemprop,
    /// The `link` has both attributes, which the standard does not allow.
    BothRelAndItemprop,
}
/// The link address-presence author requirement, independent of URL grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelAddressAuthor {
    /// The element is not a `link`.
    NotRequired,
    /// The `link` has an `href` or an `imagesrcset` attribute.
    Satisfied,
    /// The `link` has neither attribute, so it has no address.
    MissingHrefAndImagesrcset,
}
/// Local part of link's nonempty URL author requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelHrefAuthor {
    /// The element is not a `link` or has no `href`.
    NotRequired,
    /// The `link`'s `href` is empty or contains only ASCII whitespace, where the standard requires
    /// a nonempty URL.
    EmptyOrOnlyAsciiSpaces,
    /// The `link`'s `href` has a non-whitespace code unit. Whether it is a valid URL is not checked
    /// here.
    GrammarUnchecked,
}
/// Whether the element creates a link, summarised from its role, address attributes and tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelLinks {
    /// The element creates no link: it has no known role, it is an `a` or `area` without `href`,
    /// it is a `link` with neither `href` nor `imagesrcset`, or it is a `link` with an address but
    /// no token that has an effect and no unresolved unknown token.
    None,
    /// On a `link`, some token has an effect. On any other role, some token has the `Hyperlink`
    /// effect.
    Declared,
    /// The role is not `link`, no token has the `Hyperlink` effect and no unresolved unknown token
    /// exists. An `a` or `area` with an `href` is then a hyperlink by default.
    ImpliedHyperlink,
    /// No token creates the link and an unresolved unknown token exists, so a registered extension
    /// might still create it. On a `link` no token has an effect. On an `a`, `area` or `form`
    /// no token has the hyperlink effect, though tokens with other effects, such as `nofollow`,
    /// may be present.
    ExtensionEffectsUnchecked,
}
/// How the `alternate` keyword is used, which selects the meaning the standard gives it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelAlternate {
    /// No `alternate` token with an effect exists.
    NotSelected,
    /// On a `link` beside `stylesheet`: the link is an alternative style sheet (HTML877d
    /// #link-type-stylesheet).
    StylesheetModifier,
    /// The `type` attribute is exactly `application/rss+xml` or `application/atom+xml`, so the link
    /// refers to a feed (HTML877d #rel-alternate).
    Feed,
    /// Any other use: an alternate representation of the current document.
    Representation,
}
/// The author requirement that an alternative style sheet has a nonempty `title` (HTML877d
/// #link-type-stylesheet).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelTitleAuthor {
    /// The link is not an alternative style sheet.
    NotRequired,
    /// The alternative style sheet has a nonempty `title`.
    Satisfied,
    /// The alternative style sheet's `title` is absent or empty.
    MissingOrEmpty,
}
/// The reason a `link` is or is not allowed in the body (HTML877d #the-link-element): it is when it
/// has `itemprop` or a `rel` containing only body-ok keywords (HTML877d #linkTypes). The variants
/// `Itemprop` and `AllKeywordsBodyOk` allow it, `RelAbsent` and `ContainsNonBodyOkKeyword` do not,
/// and `NotLink` marks an element the rule does not govern.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelBodyAllowance {
    /// The element is not a `link`, so the rule does not apply.
    NotLink,
    /// The `link` has an `itemprop` attribute.
    Itemprop,
    /// The `link` has no `rel`, so no keyword makes it body-ok.
    RelAbsent,
    /// Every `rel` token is a body-ok keyword.
    AllKeywordsBodyOk,
    /// At least one `rel` token is not a body-ok keyword.
    ContainsNonBodyOkKeyword,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RelData {
    role: Option<HtmlRelRole>,
    rel: Option<DocumentString>,
    rev: Option<DocumentString>,
    occurrences: Vec<HtmlRelOccurrence>,
    presence_author: HtmlRelPresenceAuthor,
    links: HtmlRelLinks,
    alternate: HtmlRelAlternate,
    title_author: HtmlRelTitleAuthor,
    body: HtmlRelBodyAllowance,
    href_present: bool,
    imagesrcset_present: bool,
    address_author: HtmlRelAddressAuthor,
    href_author: HtmlRelHrefAuthor,
    as_destination: Option<HtmlLinkDestination>,
    image_source_preload: bool,
}
/// Local interpretation alone does not certify external grammar, registry or execution.
/// Its single owning record keeps semantic enum payloads bounded without boxing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelReport {
    data: Vec<RelData>,
}
impl HtmlRelReport {
    /// The element's role, or `None` for an element that takes no `rel`.
    pub fn role(&self) -> Option<HtmlRelRole> {
        self.data[0].role
    }
    /// The exact `rel` value that was interpreted, or `None` when the attribute is absent.
    pub fn rel(&self) -> Option<&DocumentString> {
        self.data[0].rel.as_ref()
    }
    /// The exact `rev` value that was considered, or `None` when the attribute is absent.
    pub fn rev(&self) -> Option<&DocumentString> {
        self.data[0].rev.as_ref()
    }
    /// Every occurrence in source order: the `rel` tokens, then at most one historical `rev`
    /// occurrence. Repeats are kept and flagged, never removed.
    pub fn occurrences(&self) -> &[HtmlRelOccurrence] {
        &self.data[0].occurrences
    }
    /// The exact UTF-16 code units of the occurrence at `index`, or `None` when `index` is out of
    /// range.
    pub fn occurrence_units(&self, index: usize) -> Option<&[u16]> {
        let token = self.occurrences().get(index)?;
        let raw = match token.source {
            HtmlRelOccurrenceSource::Rel => self.rel(),
            HtmlRelOccurrenceSource::Rev => self.rev(),
        }?;
        Some(&raw.code_units()[token.start..token.end])
    }
    /// The author finding for the `rel`/`itemprop` presence rule.
    pub fn presence_author(&self) -> HtmlRelPresenceAuthor {
        self.data[0].presence_author
    }
    /// Whether the element creates a link.
    pub fn links(&self) -> HtmlRelLinks {
        self.data[0].links
    }
    /// How the `alternate` keyword is used.
    pub fn alternate(&self) -> HtmlRelAlternate {
        self.data[0].alternate
    }
    /// The author finding for an alternative style sheet's `title`.
    pub fn alternate_stylesheet_title_author(&self) -> HtmlRelTitleAuthor {
        self.data[0].title_author
    }
    /// Why the element is or is not allowed in the body.
    pub fn body_allowance(&self) -> HtmlRelBodyAllowance {
        self.data[0].body
    }
    /// Whether the element has an `href` attribute, whatever its value.
    pub fn href_present(&self) -> bool {
        self.data[0].href_present
    }
    /// Whether the element has an `imagesrcset` attribute, whatever its value.
    pub fn imagesrcset_present(&self) -> bool {
        self.data[0].imagesrcset_present
    }
    /// The author finding that a `link` has an address.
    pub fn address_presence_author(&self) -> HtmlRelAddressAuthor {
        self.data[0].address_author
    }
    /// The author finding for a `link`'s `href` being nonempty.
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
    /// The URL of the pinned standard's link-types clause, which defines every finding in this
    /// report.
    pub fn source_clause(&self) -> &'static str {
        concat!(
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/",
            "#linkTypes"
        )
    }
}

trait RelMeter {
    type Error;
    fn work(&mut self, n: usize) -> Result<(), Self::Error>;
    fn reserve_occurrences(
        &mut self,
        values: &mut Vec<HtmlRelOccurrence>,
    ) -> Result<(), Self::Error>;
    fn reserve_data(&mut self, values: &mut Vec<RelData>) -> Result<(), Self::Error>;
}
pub(crate) fn ascii_keyword(units: &[u16], keyword: &str) -> bool {
    units.len() == keyword.len()
        && units
            .iter()
            .zip(keyword.bytes())
            .all(|(&u, b)| ascii_case_insensitive_unit(u, b))
}
/// Narrow membership shares the token owner; no full author certificate is implied.
pub(crate) fn contains_ascii_keyword(raw: &DocumentString, keyword: &str) -> bool {
    crate::internal::microsyntax::tokens::space_token_slices(raw)
        .any(|token| ascii_keyword(token, keyword))
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
        if !ascii_case_insensitive_unit(*unit, byte) {
            return Ok(false);
        }
    }
    Ok(true)
}
/// Orders two token spans of `units` by their ASCII-case-folded content. The charge is made
/// before each unit pair is examined, so it is exactly the work done.
fn compare_folded<C: RelMeter>(
    units: &[u16],
    a: &HtmlRelOccurrence,
    b: &HtmlRelOccurrence,
    m: &mut C,
) -> Result<Ordering, C::Error> {
    m.work(1)?;
    let (a, b) = (&units[a.start..a.end], &units[b.start..b.end]);
    for (a, b) in a.iter().zip(b) {
        m.work(3)?;
        let order = ascii_lowercase(*a).cmp(&ascii_lowercase(*b));
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(a.len().cmp(&b.len()))
}
/// Restores the heap property below `root` among the first `end` items.
fn sift_down<C: RelMeter>(
    items: &mut [HtmlRelOccurrence],
    mut root: usize,
    end: usize,
    m: &mut C,
    order: &mut impl FnMut(&HtmlRelOccurrence, &HtmlRelOccurrence, &mut C) -> Result<Ordering, C::Error>,
) -> Result<(), C::Error> {
    loop {
        m.work(1)?;
        let Some(mut child) = root.checked_mul(2).and_then(|n| n.checked_add(1)) else {
            return Ok(());
        };
        if child >= end {
            return Ok(());
        }
        if child + 1 < end && order(&items[child], &items[child + 1], m)? == Ordering::Less {
            child += 1;
        }
        if order(&items[root], &items[child], m)? != Ordering::Less {
            return Ok(());
        }
        items.swap(root, child);
        root = child;
    }
}
/// Sorts in place under `order`, which charges its own work, without allocating. Every
/// comparison moves its larger operand one level up the heap or ends a sift, so a comparison of
/// two tokens is paid for by one of their lengths, and the total is O(total token units *
/// log(tokens)) however the tokens are arranged.
fn heap_sort<C: RelMeter>(
    items: &mut [HtmlRelOccurrence],
    m: &mut C,
    mut order: impl FnMut(&HtmlRelOccurrence, &HtmlRelOccurrence, &mut C) -> Result<Ordering, C::Error>,
) -> Result<(), C::Error> {
    let len = items.len();
    for root in (0..len / 2).rev() {
        sift_down(items, root, len, m, &mut order)?;
    }
    for end in (1..len).rev() {
        items.swap(0, end);
        sift_down(items, 0, end, m, &mut order)?;
    }
    Ok(())
}
/// Sets `duplicate_of` of every `rel` token that an earlier token equals under ASCII case
/// folding to the position of the first such token. Each token enters with its own position in
/// `duplicate_of`. The tokens are ordered by (folded content, position), so equal tokens are
/// adjacent and the first of a run is the earliest: it keeps no twin, and the others take its
/// position. The positions are then restored by the (strictly increasing) offsets of the tokens.
fn mark_duplicates<C: RelMeter>(
    units: &[u16],
    occurrences: &mut [HtmlRelOccurrence],
    m: &mut C,
) -> Result<(), C::Error> {
    heap_sort(occurrences, m, |a, b, m| {
        Ok(compare_folded(units, a, b, m)?.then(a.start.cmp(&b.start)))
    })?;
    let mut first = None;
    for index in 0..occurrences.len() {
        m.work(1)?;
        let same_run = index > 0
            && compare_folded(units, &occurrences[index - 1], &occurrences[index], m)?
                == Ordering::Equal;
        if same_run {
            occurrences[index].duplicate_of = first;
        } else {
            first = occurrences[index].duplicate_of.take();
        }
    }
    heap_sort(occurrences, m, |a, b, m| {
        m.work(1)?;
        Ok(a.start.cmp(&b.start))
    })
}
fn classify<C: RelMeter>(
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
fn interpret<C: RelMeter>(
    input: &HtmlRelInputs<'_>,
    occurrences: &mut Vec<HtmlRelOccurrence>,
    data: &mut Vec<RelData>,
    m: &mut C,
) -> Result<(), C::Error> {
    m.work(1)?;
    if input.role.is_some()
        && let Some(raw) = input.rel
    {
        m.work(HtmlSpaceTokenCursor::BEGIN_WORK)?;
        let mut cursor = HtmlSpaceTokenCursor::new(raw.code_units());
        loop {
            m.work(1)?;
            if cursor.inspects_unit() {
                m.work(1)?;
            }
            m.work(HtmlSpaceTokenCursor::STEP_WORK)?;
            match cursor.step() {
                HtmlSpaceTokenStep::Continue => {}
                HtmlSpaceTokenStep::Done => break,
                HtmlSpaceTokenStep::Token { start, end } => {
                    let units = &raw.code_units()[start..end];
                    let (keyword, mut spelling) = classify(units, m)?;
                    if spelling == HtmlRelSpelling::InvalidShortcut
                        && input.role == Some(HtmlRelRole::Link)
                        && keyword_at(raw.code_units(), "shortcut icon", m)?
                    {
                        spelling = HtmlRelSpelling::ShortcutException;
                    }
                    m.reserve_occurrences(occurrences)?;
                    m.work(1)?;
                    occurrences.push(HtmlRelOccurrence {
                        source: HtmlRelOccurrenceSource::Rel,
                        start,
                        end,
                        keyword,
                        spelling,
                        // The token's own position until `mark_duplicates` assigns its twin.
                        duplicate_of: Some(occurrences.len()),
                        registry_resolved: false,
                        effect: keyword.and_then(|k| input.role.and_then(|r| k.effect(r))),
                    });
                }
            }
        }
        mark_duplicates(raw.code_units(), occurrences, m)?;
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
        m.reserve_occurrences(occurrences)?;
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
    let (links, alternate, presence_author, title_author, body) = summarize(input, occurrences, m)?;
    let address_author = if input.role != Some(HtmlRelRole::Link) {
        HtmlRelAddressAuthor::NotRequired
    } else if input.href.is_some() || input.imagesrcset.is_some() {
        HtmlRelAddressAuthor::Satisfied
    } else {
        HtmlRelAddressAuthor::MissingHrefAndImagesrcset
    };
    let href_author = if let (Some(HtmlRelRole::Link), Some(href)) = (input.role, input.href) {
        if controlled::rel_href_is_empty_or_ascii_whitespace_with(href.code_units(), |n| m.work(n))?
        {
            HtmlRelHrefAuthor::EmptyOrOnlyAsciiSpaces
        } else {
            HtmlRelHrefAuthor::GrammarUnchecked
        }
    } else {
        HtmlRelHrefAuthor::NotRequired
    };
    let image_source_preload = image_source_preload(input, occurrences, m)?;
    m.reserve_data(data)?;
    m.work(3)?;
    data.push(RelData {
        role: input.role,
        rel: input.rel.cloned(),
        rev: input.rev.cloned(),
        occurrences: std::mem::take(occurrences),
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
    Ok(())
}
fn refresh<C: RelMeter>(
    report: &mut HtmlRelReport,
    input: &HtmlRelInputs<'_>,
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
    input: &HtmlRelInputs<'_>,
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
    input: &HtmlRelInputs<'_>,
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
    /// registry, aggregate memory/work or cancellation certificate is supplied. Storage is reserved
    /// with the fallible allocator: a refusal returns
    /// [`HtmlValueInterpretationError::AllocationFailed`].
    pub fn rel_with_attributes(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
    ) -> Result<HtmlRelReport, HtmlValueInterpretationError> {
        // The `as` value is admitted before any other attribute is read. A view lends `value`
        // only while the view is held, so each selected slot keeps its view.
        let as_name = crate::vocabulary::KnownHtmlAttribute::As.qualified_name();
        let as_attribute = find_attribute(attributes, as_name.namespace(), as_name.local_name());
        self.admit_attribute_raw(view_value(&as_attribute))?;
        let get = |name: &str| {
            attributes_iter(attributes).find(|a| {
                a.name().namespace_uri().is_none()
                    && a.name()
                        .local_name()
                        .code_units()
                        .iter()
                        .copied()
                        .eq(name.bytes().map(u16::from))
            })
        };
        let (rel, rev, href) = (get("rel"), get("rev"), get("href"));
        let (imagesrcset, itemprop) = (get("imagesrcset"), get("itemprop"));
        let (title, type_hint) = (get("title"), get("type"));
        let mut input = HtmlRelInputs {
            role: None,
            rel: view_value(&rel),
            rev: view_value(&rev),
            href: view_value(&href),
            imagesrcset: view_value(&imagesrcset),
            itemprop: view_value(&itemprop),
            title: view_value(&title),
            type_hint: view_value(&type_hint),
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
        input.as_destination = self
            .link_as(element, as_attribute.as_ref())?
            .intrinsic()
            .copied();
        controlled::HtmlRelInterpretation::new(input).run_unmetered()
    }
}

#[cfg(test)]
mod ordinary_allocation_tests {
    use crate::internal::amortized::fault;
    use crate::values::HtmlValueInterpretationError;
    use crate::values::{Attribute, HtmlValueInterpreter, OrderedAttributes};
    use crate::vocabulary::{KnownHtmlAttribute, KnownHtmlElement};
    use std::panic::{AssertUnwindSafe, catch_unwind};

    /// A refused allocation is an expected condition of a fallible allocator, so the standalone
    /// operation reports it as its error. It must neither abort the caller by panicking nor claim
    /// a report for storage it could not reserve.
    #[test]
    fn a_refused_reservation_is_an_error_and_never_a_panic() {
        let interpreter = HtmlValueInterpreter::new(1_000);
        let element = KnownHtmlElement::Link.qualified_name();
        let attributes = OrderedAttributes::new([Attribute::new(
            KnownHtmlAttribute::Rel.qualified_name(),
            "alternate stylesheet author bookmark canonical help icon license next",
        )])
        .unwrap();
        let counting = fault::count_only();
        let baseline = interpreter
            .rel_with_attributes(&element, &attributes)
            .unwrap();
        let requests = fault::requests();
        drop(counting);
        assert_eq!(baseline.occurrences().len(), 9);
        assert!(requests >= 2, "{requests} reservation requests");
        for refuse_from in 1..=requests {
            let _armed = fault::arm(refuse_from);
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                interpreter.rel_with_attributes(&element, &attributes)
            }));
            let verdict = match &outcome {
                Err(_) => "panicked",
                Ok(Ok(_)) => "reported success",
                Ok(Err(HtmlValueInterpretationError::AllocationFailed)) => {
                    "returned the allocation error"
                }
                Ok(Err(_)) => "returned another error",
            };
            assert_eq!(
                verdict, "returned the allocation error",
                "reservation {refuse_from} of {requests} refused"
            );
        }
        let _armed = fault::arm(requests + 1);
        let report = interpreter
            .rel_with_attributes(&element, &attributes)
            .unwrap();
        assert_eq!(report, baseline);
    }
}

#[cfg(test)]
mod keyword_token_tests {
    use super::*;

    /// The keyword table is the closed set of the standard's link types. Each is a valid space
    /// token under the checked constructor, which is what the compile-time check of the table
    /// promises, and `ALL` lists each exactly once in alphabetical order, as it documents.
    #[test]
    fn every_standard_keyword_is_a_valid_space_token_with_its_spelling() {
        assert!(!HtmlRelKeyword::ALL.is_empty());
        for keyword in HtmlRelKeyword::ALL {
            let checked = HtmlSpaceToken::new(keyword.keyword())
                .unwrap_or_else(|error| panic!("{keyword:?}: {error:?}"));
            assert_eq!(keyword.space_token(), checked, "{keyword:?}");
        }
        assert!(
            HtmlRelKeyword::ALL
                .windows(2)
                .all(|pair| pair[0].keyword() < pair[1].keyword()),
            "ALL is strictly alphabetical by spelling"
        );
    }
}
