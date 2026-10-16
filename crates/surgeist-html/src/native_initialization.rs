//! Shared element classification, closed common values and native freshness.
//!
//! HTML877d #elements-in-the-dom defines interface dispatch, including obsolete
//! exceptions and the custom/unknown fallback. The 113 concrete modern normative
//! definitions exactly cover KnownHtmlElement in this pin; 28 additional obsolete
//! names are handled below. The selected profile has no additional HTML-namespace
//! interface provider. A newly selected definition must extend this dispatch.
//! This is neither parser recovery nor WebIDL prototype construction.

use crate::values::{HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, SVG_NAMESPACE_URI};
use crate::{
    HtmlNativeFormAssociationKind, HtmlTemplateFreshInsertionState, KnownHtmlElement, QualifiedName,
};

/// Name-derived family and namespace applicability, independent of creation cause.
///
/// Native creation must initialize each applicable family; retained installation
/// must transfer its actual state. Classification proves neither operation.
/// KnownHtml preserves an unadmitted built-in's actual identity; it does not
/// claim every such element has another stored slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlElementClass {
    GenericElement,
    CommonHtml,
    CommonSvg,
    CommonMathml,
    FormAssociated(HtmlNativeFormAssociationKind),
    KnownHtml(KnownHtmlElement),
    HtmlMarquee,
    HtmlFrame,
    SvgScript,
}
impl HtmlElementClass {
    /// Applicable closed common fields, not evidence of their current values.
    pub const fn common_scope(self) -> HtmlCommonScope {
        match self {
            Self::GenericElement | Self::CommonSvg | Self::CommonMathml | Self::SvgScript => {
                HtmlCommonScope::Element
            }
            Self::CommonHtml
            | Self::FormAssociated(_)
            | Self::KnownHtml(_)
            | Self::HtmlMarquee
            | Self::HtmlFrame => HtmlCommonScope::Html,
        }
    }
    /// Nonce slot applicability. This neither reads nor initializes that slot.
    pub const fn has_nonce(self) -> bool {
        !matches!(self, Self::GenericElement)
    }
}

/// Admission immediately before the named actual borrowed operand read.
/// Header includes the length and, for namespace, optional URI observation.
/// Pair reads one actual UTF-16 unit and its exact canonical ASCII operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlElementNameRead {
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
    requirement: HtmlElementClass,
}
impl HtmlNativeInitialization {
    pub const fn requirement(&self) -> HtmlElementClass {
        self.requirement
    }
    pub const fn common(&self) -> HtmlCommonState {
        HtmlCommonState::closed(self.requirement.common_scope())
    }
    /// Fresh nonce evidence belongs to native creation, separately from common
    /// current values. Retained nonce readback must come from its actual owner.
    pub const fn nonce(&self) -> Option<HtmlNativeFreshNonce> {
        if self.requirement.has_nonce() {
            Some(HtmlNativeFreshNonce::Empty)
        } else {
            None
        }
    }
    /// Fresh insertion references for the actual HTML template requirement.
    /// The consumer creates the separate contents and owns all reference slots;
    /// this token cannot reset installed or retained insertion history.
    pub const fn template_insertion(&self) -> Option<HtmlTemplateFreshInsertionState> {
        match self.requirement {
            HtmlElementClass::KnownHtml(KnownHtmlElement::Template) => {
                Some(HtmlTemplateFreshInsertionState::NullTargetAndMarkers)
            }
            _ => None,
        }
    }
    /// HTML877d #manually-assigned-nodes starts an actual native slot empty.
    /// The consumer materializes the identity-bearing ordered set; this value
    /// is creation evidence, not a current-empty flag after later assignment.
    pub const fn slot(&self) -> Option<HtmlNativeFreshSlot> {
        match self.requirement {
            HtmlElementClass::KnownHtml(KnownHtmlElement::Slot) => {
                Some(HtmlNativeFreshSlot::EmptyManualAssignments)
            }
            _ => None,
        }
    }
}

/// Fresh HTML slot manual intent, with node identity/storage owned by DOM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNativeFreshSlot {
    EmptyManualAssignments,
}

/// Applicability of the shared common fields. SVG, MathML and generic elements
/// have only the element tuple; nonce applicability is independent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCommonScope {
    Element,
    Html,
}

/// The closed current common tuple, reusable across construction causes.
///
/// A consuming owner may retain this closed tuple while its exposed operations
/// preserve it. A capability that can change these slots requires richer current
/// state before admission. This value carries no report, subject or creation
/// authority: the actual producer must qualify it. The mutable nonce slot is
/// separately owned and observed, and is not part of this tuple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCommonState {
    scope: HtmlCommonScope,
}
impl HtmlCommonState {
    /// Constructs the literal closed values for the given applicability.
    /// This does not establish that any particular element has these values.
    pub const fn closed(scope: HtmlCommonScope) -> Self {
        Self { scope }
    }
    pub const fn scope(&self) -> HtmlCommonScope {
        self.scope
    }
    pub const fn element(&self) -> HtmlClosedElementState {
        HtmlClosedElementState { closed: () }
    }
    pub const fn html(&self) -> Option<HtmlClosedHtmlState> {
        match self.scope {
            HtmlCommonScope::Html => Some(HtmlClosedHtmlState { closed: () }),
            HtmlCommonScope::Element => None,
        }
    }
}

/// HTML877d #custom-element-reactions gives all elements an initially empty
/// per-element reaction queue; #activation gives each element an initially
/// unset click-in-progress flag. This is not an agent queue or activation API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlClosedElementState {
    closed: (),
}
impl HtmlClosedElementState {
    pub const fn reaction_queue_is_empty(&self) -> bool {
        true
    }
    pub const fn click_in_progress(&self) -> bool {
        false
    }
}

/// The HTML-only closed tuple from #previously-focused-element,
/// #attached-internals and #the-popover-attribute. These are actual null, hidden
/// and false values, not missing observations or execution capabilities.
/// Popover runtime visibility is distinct from HtmlPopoverState's attribute rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlClosedHtmlState {
    closed: (),
}
impl HtmlClosedHtmlState {
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

fn known_class(element: KnownHtmlElement) -> HtmlElementClass {
    use HtmlElementClass as R;
    use KnownHtmlElement as E;
    if let Some(kind) = HtmlNativeFormAssociationKind::from_element(element) {
        return R::FormAssociated(kind);
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
const OBSOLETE: &[(&str, HtmlElementClass)] = &[
    ("applet", HtmlElementClass::CommonHtml),
    ("bgsound", HtmlElementClass::CommonHtml),
    ("blink", HtmlElementClass::CommonHtml),
    ("isindex", HtmlElementClass::CommonHtml),
    ("keygen", HtmlElementClass::CommonHtml),
    ("multicol", HtmlElementClass::CommonHtml),
    ("nextid", HtmlElementClass::CommonHtml),
    ("spacer", HtmlElementClass::CommonHtml),
    ("acronym", HtmlElementClass::CommonHtml),
    ("basefont", HtmlElementClass::CommonHtml),
    ("big", HtmlElementClass::CommonHtml),
    ("center", HtmlElementClass::CommonHtml),
    ("nobr", HtmlElementClass::CommonHtml),
    ("noembed", HtmlElementClass::CommonHtml),
    ("noframes", HtmlElementClass::CommonHtml),
    ("plaintext", HtmlElementClass::CommonHtml),
    ("rb", HtmlElementClass::CommonHtml),
    ("rtc", HtmlElementClass::CommonHtml),
    ("strike", HtmlElementClass::CommonHtml),
    ("tt", HtmlElementClass::CommonHtml),
    ("listing", HtmlElementClass::CommonHtml),
    ("xmp", HtmlElementClass::CommonHtml),
    ("dir", HtmlElementClass::CommonHtml),
    ("font", HtmlElementClass::CommonHtml),
    ("param", HtmlElementClass::CommonHtml),
    ("frameset", HtmlElementClass::CommonHtml),
    ("marquee", HtmlElementClass::HtmlMarquee),
    ("frame", HtmlElementClass::HtmlFrame),
];

fn namespace_matches<E>(
    name: &QualifiedName,
    expected: &str,
    admit: &mut impl FnMut(HtmlElementNameRead) -> Result<(), E>,
) -> Result<Option<bool>, E> {
    admit(HtmlElementNameRead::NamespaceHeader)?;
    let Some(value) = name.namespace().uri() else {
        return Ok(None);
    };
    if value.len() != expected.len() {
        return Ok(Some(false));
    }
    for offset in 0..expected.len() {
        admit(HtmlElementNameRead::NamespacePair)?;
        if value.code_units()[offset] != u16::from(expected.as_bytes()[offset]) {
            return Ok(Some(false));
        }
    }
    Ok(Some(true))
}

fn local_matches<E>(
    name: &QualifiedName,
    expected: &str,
    admit: &mut impl FnMut(HtmlElementNameRead) -> Result<(), E>,
) -> Result<bool, E> {
    admit(HtmlElementNameRead::LocalNameHeader)?;
    let value = name.local_name();
    if value.len() != expected.len() {
        return Ok(false);
    }
    for offset in 0..expected.len() {
        admit(HtmlElementNameRead::LocalNamePair)?;
        if value.code_units()[offset] != u16::from(expected.as_bytes()[offset]) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Classifies family and common/nonce applicability from the actual expanded
/// name, independently of construction cause. Name validity belongs to the
/// preceding operation. No current state or fresh evidence is supplied.
/// Every Header/Pair is admitted before its actual read; errors stop immediately.
/// No prefix, attribute, registry, custom grammar or external fact is inspected.
///
/// Exact namespace dispatch precedes local-name access. Modern names use the
/// canonical vocabulary; all obsolete cases are exhausted before this selected
/// profile's custom/unknown fallback. Both final prototype branches need the
/// same family classification; actual custom state uses its separate shared rule.
/// This function never folds names or treats an unhandled known family as Common.
pub fn element_classification_with<E>(
    name: &QualifiedName,
    mut admit: impl FnMut(HtmlElementNameRead) -> Result<(), E>,
) -> Result<HtmlElementClass, E> {
    use HtmlElementClass as R;
    match namespace_matches(name, HTML_NAMESPACE_URI, &mut admit)? {
        None => return Ok(R::GenericElement),
        Some(true) => {
            for &element in KnownHtmlElement::ALL {
                if local_matches(name, element.local_name(), &mut admit)? {
                    return Ok(known_class(element));
                }
            }
            for &(local, requirement) in OBSOLETE {
                if local_matches(name, local, &mut admit)? {
                    return Ok(requirement);
                }
            }
            return Ok(R::CommonHtml);
        }
        Some(false) => {}
    }
    if namespace_matches(name, SVG_NAMESPACE_URI, &mut admit)? == Some(true) {
        return Ok(
            if local_matches(name, KnownHtmlElement::Script.local_name(), &mut admit)? {
                R::SvgScript
            } else {
                R::CommonSvg
            },
        );
    }
    if namespace_matches(name, MATHML_NAMESPACE_URI, &mut admit)? == Some(true) {
        return Ok(R::CommonMathml);
    }
    Ok(R::GenericElement)
}

/// Selects and fulfills the shared common initial tuple for native creation.
/// Applicable family state and hooks remain the consuming owner's obligation.
/// Existing/parsed current state cannot be established by this fresh operation.
/// Classification's exact name reads are admitted once, before each operand read.
pub fn native_element_initialization_with<E>(
    name: &QualifiedName,
    admit: impl FnMut(HtmlElementNameRead) -> Result<(), E>,
) -> Result<HtmlNativeInitialization, E> {
    element_classification_with(name, admit)
        .map(|requirement| HtmlNativeInitialization { requirement })
}
