//! Selected native initialization requirements and fresh common state.
//!
//! HTML877d #elements-in-the-dom defines interface dispatch, including obsolete
//! exceptions and the custom/unknown fallback. The 113 concrete modern normative
//! definitions exactly cover KnownHtmlElement in this pin; 28 additional obsolete
//! names are handled below. The selected profile has no additional HTML-namespace
//! interface provider. A newly selected definition must extend this dispatch.
//! This is neither parser recovery nor WebIDL prototype construction.

use crate::values::{HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI};
use crate::{HtmlNativeFormAssociationKind, KnownHtmlElement, QualifiedName};

/// Required native family initialization in addition to the common baseline.
///
/// These are requirements, not caller-selected construction modes. Consumers
/// derive them from the real validated name and must fulfill applicable state
/// and hooks before publication. KnownHtml preserves an unadmitted built-in's
/// actual identity; it does not claim every such element has another stored slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNativeInitializationRequirement {
    GenericElement,
    CommonHtml,
    CommonSvg,
    CommonMathml,
    NativeForm(HtmlNativeFormAssociationKind),
    KnownHtml(KnownHtmlElement),
    HtmlMarquee,
    HtmlFrame,
    SvgScript,
}

/// Admission immediately before the named actual borrowed operand read.
/// Header includes the length and, for namespace, optional URI observation.
/// Pair reads one actual UTF-16 unit and its exact canonical ASCII operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNativeNameRead {
    NamespaceHeader,
    NamespacePair,
    LocalNameHeader,
    LocalNamePair,
}

/// A selected requirement and the initialized common tuple for a new element.
///
/// This has no public constructor or profile override. It performs no DOM name
/// validation, registry/definition lookup, custom-name scan or allocation. Actual
/// native creation must independently qualify those facts and instantiate every
/// required family. Existing/advanced state must be imported through its owning
/// adapter, never reset by calling this native initializer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNativeInitialization {
    requirement: HtmlNativeInitializationRequirement,
    common: HtmlNativeCommonState,
}
impl HtmlNativeInitialization {
    pub const fn requirement(&self) -> HtmlNativeInitializationRequirement {
        self.requirement
    }
    pub const fn common(&self) -> HtmlNativeCommonState {
        self.common
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommonScope {
    Element,
    Html,
    Mixin,
}

/// Actual fresh common state, with scope selected from the exact namespace.
///
/// A consuming owner may retain this closed tuple while its exposed operations
/// preserve it. A capability that can change these slots requires richer current
/// state before admission. The mutable nonce slot is separately materialized and
/// owned by the consumer; this value carries only its fresh initialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNativeCommonState {
    scope: CommonScope,
}
impl HtmlNativeCommonState {
    pub const fn element(&self) -> HtmlNativeClosedElementState {
        HtmlNativeClosedElementState { closed: () }
    }
    pub const fn html(&self) -> Option<HtmlNativeClosedHtmlState> {
        match self.scope {
            CommonScope::Html => Some(HtmlNativeClosedHtmlState { closed: () }),
            CommonScope::Element | CommonScope::Mixin => None,
        }
    }
    pub const fn nonce(&self) -> Option<HtmlNativeFreshNonce> {
        match self.scope {
            CommonScope::Element => None,
            CommonScope::Html | CommonScope::Mixin => Some(HtmlNativeFreshNonce::Empty),
        }
    }
}

/// HTML877d #custom-element-reactions gives all elements an initially empty
/// per-element reaction queue; #activation gives each element an initially
/// unset click-in-progress flag. This is not an agent queue or activation API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNativeClosedElementState {
    closed: (),
}
impl HtmlNativeClosedElementState {
    pub const fn reaction_queue_is_empty(&self) -> bool {
        true
    }
    pub const fn click_in_progress(&self) -> bool {
        false
    }
}

/// The HTML-only fresh tuple from #previously-focused-element,
/// #attached-internals and #the-popover-attribute. These are actual null, hidden
/// and false values, not missing observations or execution capabilities.
/// Popover runtime visibility is distinct from HtmlPopoverState's attribute rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNativeClosedHtmlState {
    closed: (),
}
impl HtmlNativeClosedHtmlState {
    pub const fn has_previously_focused_element(&self) -> bool {
        false
    }
    pub const fn has_attached_internals(&self) -> bool {
        false
    }
    pub const fn popover_is_hidden(&self) -> bool {
        true
    }
    pub const fn has_popover_trigger(&self) -> bool {
        false
    }
    pub const fn popover_is_hiding(&self) -> bool {
        false
    }
    pub const fn has_popover_toggle_task_tracker(&self) -> bool {
        false
    }
    pub const fn has_popover_close_watcher(&self) -> bool {
        false
    }
    pub const fn has_popover_opened_in_mode(&self) -> bool {
        false
    }
}

/// HTML's nonce mixin slot is actually initialized empty on HTML/SVG/MathML
/// elements. The owner admits and creates the separate mutable DocumentString.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNativeFreshNonce {
    Empty,
}

fn initialized(requirement: HtmlNativeInitializationRequirement) -> HtmlNativeInitialization {
    use HtmlNativeInitializationRequirement as R;
    let scope = match requirement {
        R::GenericElement => CommonScope::Element,
        R::CommonSvg | R::CommonMathml | R::SvgScript => CommonScope::Mixin,
        R::CommonHtml | R::NativeForm(_) | R::KnownHtml(_) | R::HtmlMarquee | R::HtmlFrame => {
            CommonScope::Html
        }
    };
    HtmlNativeInitialization {
        requirement,
        common: HtmlNativeCommonState { scope },
    }
}

fn known_requirement(element: KnownHtmlElement) -> HtmlNativeInitializationRequirement {
    use HtmlNativeInitializationRequirement as R;
    use KnownHtmlElement as E;
    if let Some(kind) = HtmlNativeFormAssociationKind::from_element(element) {
        return R::NativeForm(kind);
    }
    // The first 41 cases cover HTMLElement-only definitions plus div/span's
    // reflected interfaces. The next 41 reflected/child-text/tree-query
    // interfaces also require only common native initialization. Common does not
    // equate WebIDL interfaces or grant execution, navigation, rendering or queries.
    match element {
        E::Article
        | E::Section
        | E::Nav
        | E::Aside
        | E::Hgroup
        | E::Header
        | E::Footer
        | E::Address
        | E::Dt
        | E::Dd
        | E::Figure
        | E::Figcaption
        | E::Main
        | E::Search
        | E::Em
        | E::Strong
        | E::Small
        | E::S
        | E::Cite
        | E::Dfn
        | E::Abbr
        | E::Ruby
        | E::Rt
        | E::Rp
        | E::Code
        | E::Var
        | E::Samp
        | E::Kbd
        | E::Sub
        | E::Sup
        | E::I
        | E::B
        | E::U
        | E::Mark
        | E::Bdi
        | E::Bdo
        | E::Wbr
        | E::Summary
        | E::Noscript
        | E::Div
        | E::Span
        | E::Html
        | E::Head
        | E::Title
        | E::Body
        | E::P
        | E::Hr
        | E::Pre
        | E::Blockquote
        | E::Ol
        | E::Ul
        | E::Menu
        | E::Li
        | E::Dl
        | E::Q
        | E::Data
        | E::Time
        | E::Br
        | E::Ins
        | E::Del
        | E::Map
        | E::Datalist
        | E::Label
        | E::Legend
        | E::Progress
        | E::Meter
        | E::Table
        | E::Caption
        | E::Colgroup
        | E::Col
        | E::Tbody
        | E::Thead
        | E::Tfoot
        | E::Tr
        | E::Td
        | E::Th
        | E::H1
        | E::H2
        | E::H3
        | E::H4
        | E::H5
        | E::H6 => R::CommonHtml,
        _ => R::KnownHtml(element),
    }
}

// HTML #elements-in-the-dom's 8 Unknown/12 HTMLElement/2 Pre exceptions,
// plus the six additional obsolete interfaces. Names remain exact identities;
// this is not a parser alias table. Only marquee/frame need separate state.
const OBSOLETE: &[(&str, HtmlNativeInitializationRequirement)] = &[
    ("applet", HtmlNativeInitializationRequirement::CommonHtml),
    ("bgsound", HtmlNativeInitializationRequirement::CommonHtml),
    ("blink", HtmlNativeInitializationRequirement::CommonHtml),
    ("isindex", HtmlNativeInitializationRequirement::CommonHtml),
    ("keygen", HtmlNativeInitializationRequirement::CommonHtml),
    ("multicol", HtmlNativeInitializationRequirement::CommonHtml),
    ("nextid", HtmlNativeInitializationRequirement::CommonHtml),
    ("spacer", HtmlNativeInitializationRequirement::CommonHtml),
    ("acronym", HtmlNativeInitializationRequirement::CommonHtml),
    ("basefont", HtmlNativeInitializationRequirement::CommonHtml),
    ("big", HtmlNativeInitializationRequirement::CommonHtml),
    ("center", HtmlNativeInitializationRequirement::CommonHtml),
    ("nobr", HtmlNativeInitializationRequirement::CommonHtml),
    ("noembed", HtmlNativeInitializationRequirement::CommonHtml),
    ("noframes", HtmlNativeInitializationRequirement::CommonHtml),
    ("plaintext", HtmlNativeInitializationRequirement::CommonHtml),
    ("rb", HtmlNativeInitializationRequirement::CommonHtml),
    ("rtc", HtmlNativeInitializationRequirement::CommonHtml),
    ("strike", HtmlNativeInitializationRequirement::CommonHtml),
    ("tt", HtmlNativeInitializationRequirement::CommonHtml),
    ("listing", HtmlNativeInitializationRequirement::CommonHtml),
    ("xmp", HtmlNativeInitializationRequirement::CommonHtml),
    ("dir", HtmlNativeInitializationRequirement::CommonHtml),
    ("font", HtmlNativeInitializationRequirement::CommonHtml),
    ("param", HtmlNativeInitializationRequirement::CommonHtml),
    ("frameset", HtmlNativeInitializationRequirement::CommonHtml),
    ("marquee", HtmlNativeInitializationRequirement::HtmlMarquee),
    ("frame", HtmlNativeInitializationRequirement::HtmlFrame),
];

fn namespace_matches<E>(
    name: &QualifiedName,
    expected: &str,
    admit: &mut impl FnMut(HtmlNativeNameRead) -> Result<(), E>,
) -> Result<Option<bool>, E> {
    admit(HtmlNativeNameRead::NamespaceHeader)?;
    let Some(value) = name.namespace().uri() else {
        return Ok(None);
    };
    if value.len() != expected.len() {
        return Ok(Some(false));
    }
    for offset in 0..expected.len() {
        admit(HtmlNativeNameRead::NamespacePair)?;
        if value.code_units()[offset] != u16::from(expected.as_bytes()[offset]) {
            return Ok(Some(false));
        }
    }
    Ok(Some(true))
}

fn local_matches<E>(
    name: &QualifiedName,
    expected: &str,
    admit: &mut impl FnMut(HtmlNativeNameRead) -> Result<(), E>,
) -> Result<bool, E> {
    admit(HtmlNativeNameRead::LocalNameHeader)?;
    let value = name.local_name();
    if value.len() != expected.len() {
        return Ok(false);
    }
    for offset in 0..expected.len() {
        admit(HtmlNativeNameRead::LocalNamePair)?;
        if value.code_units()[offset] != u16::from(expected.as_bytes()[offset]) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Selects native creation requirements and common initial values from the
/// actual expanded name. Name validity belongs to the preceding DOM operation.
/// Every Header/Pair is admitted before its actual read; errors stop immediately.
/// No prefix, attribute, registry, custom grammar or external fact is inspected.
///
/// Exact namespace dispatch precedes local-name access. Modern names use the
/// canonical vocabulary; all obsolete cases are exhausted before this selected
/// profile's custom/unknown fallback. Both final prototype branches need the
/// same common tuple; actual custom state still uses its separate shared rule.
/// This function never folds names or treats an unhandled known family as Common.
pub fn native_element_initialization_with<E>(
    name: &QualifiedName,
    mut admit: impl FnMut(HtmlNativeNameRead) -> Result<(), E>,
) -> Result<HtmlNativeInitialization, E> {
    use HtmlNativeInitializationRequirement as R;
    match namespace_matches(name, HTML_NAMESPACE_URI, &mut admit)? {
        None => return Ok(initialized(R::GenericElement)),
        Some(true) => {
            for &element in KnownHtmlElement::ALL {
                if local_matches(name, element.local_name(), &mut admit)? {
                    return Ok(initialized(known_requirement(element)));
                }
            }
            for &(local, requirement) in OBSOLETE {
                if local_matches(name, local, &mut admit)? {
                    return Ok(initialized(requirement));
                }
            }
            return Ok(initialized(R::CommonHtml));
        }
        Some(false) => {}
    }
    if namespace_matches(name, SVG_NAMESPACE_URI, &mut admit)? == Some(true) {
        return Ok(initialized(
            if local_matches(name, KnownHtmlElement::Script.local_name(), &mut admit)? {
                R::SvgScript
            } else {
                R::CommonSvg
            },
        ));
    }
    if namespace_matches(name, MATHML_NAMESPACE_URI, &mut admit)? == Some(true) {
        return Ok(initialized(R::CommonMathml));
    }
    Ok(initialized(R::GenericElement))
}
