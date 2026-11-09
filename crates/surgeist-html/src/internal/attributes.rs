//! Intrinsic interpretation of selected HTML attributes over exact raw records.
//!
//! This bounded family covers global and selected element-specific enumerations,
//! Boolean attributes, exact scalar content values, submitter keyword overrides,
//! and declarative-shadow template modes/flags. It does not resolve inheritance,
//! editing, directionality, dragging, focus, or shadow-root creation. Microdata
//! token observations are local; qualified tree/URL/vocabulary facts belong to
//! the semantic owner.
//! The defining HTML clauses, rather than non-normative index summaries, own
//! these rules. WHATWG source attribution is in `licenses/whatwg/NOTICE.md`.
//! Referrer-policy vocabulary is qualified separately in its report metadata.

use crate::internal::microsyntax::tokens::ascii_case_insensitive_unit;
use crate::internal::values::semantic_input::{capture_attribute, capture_name, find_attribute};
use crate::{
    internal::construction::HtmlShadowRootMode, internal::construction::HtmlSlotAssignment,
    internal::microsyntax::tokens::HtmlTokenError, internal::values::Attribute,
    internal::values::DocumentString, internal::values::Namespace, internal::values::QualifiedName,
    internal::values::custom_names::HtmlCustomElementNameError,
    internal::values::interpretation::HtmlValueAdmissionError,
    internal::values::interpretation::HtmlValueInterpretationError,
    internal::values::interpretation::HtmlValueInterpreter, vocabulary::KnownHtmlAttribute,
    vocabulary::KnownHtmlElement,
};
use crate::{
    internal::values::semantic_input::HtmlAttributeInput,
    internal::values::semantic_input::HtmlNameInput,
};
use encoding_label::HtmlUtf8EncodingLabelError;
use input_colorspace::HtmlInputColorSpace;
use navigable_target::HtmlNavigableTargetError;

/// Exact presence of the supplied attribute, independent of validity/defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeObservation {
    /// The attribute record is absent from the supplied attributes.
    Missing,
    /// The attribute is present and its value has zero UTF-16 code units.
    PresentEmpty,
    /// The attribute is present and its value has at least one UTF-16 code unit.
    PresentNonempty,
}

/// Whether the selected rule applies to the expanded names and fact family.
/// Author-only restrictions do not suppress known intrinsic content presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeApplicability {
    /// The rule applies to the supplied element and attribute names, subject to any further context
    /// fact the rule needs.
    Applicable,
    /// The element is not in the HTML namespace, so no HTML content attribute rule applies to it.
    ForeignElementNamespace,
    /// The element is in the HTML namespace but is not one the selected rule covers.
    DifferentElement,
    /// The supplied attribute has a namespace or a local name other than the one the selected rule
    /// defines.
    DifferentAttribute,
    /// Script kind and external/inline placement were not supplied.
    ScriptContextRequired,
    /// The supplied script kind does not admit the selected content attribute.
    InapplicableScriptKind,
    /// An unknown HTML element needs actual custom-control classification.
    CustomControlContextRequired,
    /// The supplied custom element is not form-associated.
    InapplicableCustomControl,
    /// The supplied facts belong to another rule or element context.
    DifferentContext,
}

/// Explicit script facts from HTML's content-attribute applicability table.
///
/// These are supplied facts, not a script type or MIME parser. No script is
/// fetched or executed, and inline imports are not resolved by this model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptAttributeContext {
    /// A classic script whose source is fetched through its `src` attribute.
    ExternalClassic,
    /// A classic script whose source is the element's inline text.
    InlineClassic,
    /// A module script whose source is fetched through its `src` attribute.
    ExternalModule,
    /// A module script whose source is the element's inline text.
    InlineModule,
    /// An import map; the script attribute table lists none of the attributes this family
    /// interprets for it.
    ImportMap,
    /// A speculation rules block; the script attribute table lists none of the attributes this
    /// family interprets for it.
    SpeculationRules,
    /// A data block, a `script` whose type is neither a script nor a rules type; the script
    /// attribute table lists none of the attributes this family interprets for it.
    DataBlock,
}

/// Explicit a/area href presence, without parsing or resolving the URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkAttributeContext {
    /// The `a` or `area` element has an `href` attribute, even when its value is empty.
    WithHref,
    /// The `a` or `area` element has no `href` attribute.
    WithoutHref,
}

/// Source-established autonomous custom-control classification.
///
/// This is a supplied fact about an actual element and its definition. The
/// interpreter does not infer registry, upgrade or form association from a name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomControlAttributeContext {
    /// The custom element's definition makes it a form-associated custom element.
    FormAssociated,
    /// The custom element is not form-associated, so form-control attributes have no rule here.
    NotFormAssociated,
}

/// Actual button classification, including source-defined Auto-state context.
/// This is not inferred from the type keyword or used to submit a form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlButtonSubmitterContext {
    /// The button is a submit button as the `button` element section defines the term.
    SubmitButton,
    /// The button is not a submit button, for example because its `type` is Reset Button or Button.
    NotSubmitButton,
}

/// Actual ancestor-chain evidence for img's ismap author requirement.
/// An HTML a ancestor with href present qualifies even when href is empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageMapAttributeContext {
    /// The `img` has an HTML `a` ancestor carrying an `href` attribute, which satisfies the `ismap`
    /// requirement.
    AncestorAnchorWithHref,
    /// No ancestor of the `img` is an HTML `a` element with an `href` attribute.
    NoAncestorAnchorWithHref,
    /// The caller could not establish ancestor evidence, so the requirement is reported as
    /// unavailable rather than failed.
    Unavailable,
}

/// Actual alt observation for img controls, without trimming its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageControlsAttributeContext {
    /// The `img` has no `alt` attribute.
    AltMissing,
    /// The `img` has an `alt` attribute whose value is the empty string.
    AltEmpty,
    /// The `img` has an `alt` attribute whose value has at least one code unit; whitespace-only
    /// text counts as nonempty.
    AltNonempty,
    /// The caller could not establish the `alt` observation, so the requirement is reported as
    /// unavailable rather than failed.
    Unavailable,
}

/// Source-derived stylesheet membership with the exact supplied rel value.
/// Construction belongs to HtmlValueInterpreter::stylesheet_link_context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlStylesheetLinkAttributeContext {
    rel: Option<DocumentString>,
    contains_stylesheet: bool,
}

impl HtmlStylesheetLinkAttributeContext {
    /// None is an actual missing rel attribute, not unavailable evidence.
    pub fn rel(&self) -> Option<&DocumentString> {
        self.rel.as_ref()
    }
    /// Returns whether the `rel` value, split on ASCII whitespace, contains the `stylesheet` token
    /// compared ASCII case-insensitively.
    pub fn contains_stylesheet(&self) -> bool {
        self.contains_stylesheet
    }
}

/// A selected extra author constraint, independent of value spelling.
/// This bounded observation does not certify complete element conformance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeContextAuthorState {
    /// This rule/raw observation selects no extra context-author check.
    NotRequired,
    /// No author-context observation was requested for the present attribute.
    Unchecked,
    /// Supplied actual context evidence could not be established.
    Unavailable,
    /// The context evidence was supplied and satisfies the selected requirement.
    Conforming,
    /// The context evidence was supplied and violates the selected requirement; the payload names
    /// which one.
    Nonconforming(HtmlAttributeContextConformanceError),
    /// The selected rule does not apply to these names or operation context.
    Unapplied,
}

/// A context-dependent author requirement that a present attribute violates.
///
/// Each variant names one requirement from the defining clause of the attribute. The value's own
/// spelling is judged separately by [`HtmlAttributeConformanceError`]. The enum is
/// `#[non_exhaustive]`: new requirements can be added without breaking a caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAttributeContextConformanceError {
    /// An `img` with `ismap` has no HTML `a` ancestor carrying an `href` attribute.
    ImageMapRequiresAncestorAnchorWithHref,
    /// An `img` with `controls` has a missing or empty `alt` attribute.
    ImageControlsRequireNonemptyAlt,
    /// A `link` with `disabled` does not have the `stylesheet` relationship in its `rel`.
    StylesheetDisabledRequiresStylesheetRel,
    /// An `a` or `area` with `referrerpolicy` has no `href` attribute.
    ReferrerPolicyRequiresHref,
    /// The normative input-state bookkeeping prohibits authored presence.
    /// This report does not apply the source's separate ignore/effect rule.
    InputStateForbidsAttribute,
    /// The script table prohibits authored presence for this kind/placement.
    ScriptContextForbidsAttribute,
    /// The actual button is not a source-defined submit button.
    FormnovalidateRequiresSubmitButton,
    /// A formmethod or formenctype override was authored on a non-submit button.
    FormOverrideRequiresSubmitButton,
    /// Hard textarea wrap requires a specified cols attribute, independently of its value.
    TextareaHardWrapRequiresCols,
    /// A specified link blocking declaration requires stylesheet or expect rel.
    BlockingRequiresStylesheetOrExpectRel,
    /// Specified link icon sizes require icon or apple-touch-icon rel.
    IconSizesRequireIconRel,
    /// Source dimension attributes require an actual HTML picture parent.
    SourceDimensionsRequirePictureParent,
}

impl std::fmt::Display for HtmlAttributeContextConformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ImageMapRequiresAncestorAnchorWithHref => {
                "image map attribute requires an ancestor a element with an href attribute"
            }
            Self::ImageControlsRequireNonemptyAlt => {
                "image controls attribute requires a nonempty alt attribute"
            }
            Self::StylesheetDisabledRequiresStylesheetRel => {
                "disabled attribute on a link requires the stylesheet relationship"
            }
            Self::ReferrerPolicyRequiresHref => {
                "referrerpolicy attribute requires an href attribute"
            }
            Self::InputStateForbidsAttribute => {
                "attribute is prohibited for the input element's type state"
            }
            Self::ScriptContextForbidsAttribute => {
                "attribute is prohibited for this script type and placement"
            }
            Self::FormnovalidateRequiresSubmitButton => {
                "formnovalidate attribute requires a submit button"
            }
            Self::FormOverrideRequiresSubmitButton => {
                "formmethod and formenctype attributes require a submit button"
            }
            Self::TextareaHardWrapRequiresCols => {
                "textarea with hard wrapping requires a cols attribute"
            }
            Self::BlockingRequiresStylesheetOrExpectRel => {
                "blocking attribute on a link requires the stylesheet or expect relationship"
            }
            Self::IconSizesRequireIconRel => {
                "sizes attribute on a link requires the icon or apple-touch-icon relationship"
            }
            Self::SourceDimensionsRequirePictureParent => {
                "width and height attributes on source require a picture parent"
            }
        })
    }
}
impl std::error::Error for HtmlAttributeContextConformanceError {}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SuppliedContext {
    None,
    Script(HtmlScriptAttributeContext),
    Hyperlink(HtmlHyperlinkAttributeContext),
    CustomControl(HtmlCustomControlAttributeContext),
    Input(HtmlInputType),
    ButtonSubmitter(HtmlButtonSubmitterContext),
    ImageMap(HtmlImageMapAttributeContext),
    ImageControls(HtmlImageControlsAttributeContext),
    StylesheetLink(HtmlStylesheetLinkAttributeContext),
    TextareaCols(Option<Attribute>),
}

/// Recognition of literal value syntax; a fallback never changes this result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeRecognition {
    /// The attribute is absent; nothing was recognised.
    Missing,
    /// The attribute is present with an empty value that matched no keyword. For a Boolean
    /// attribute this is the conforming empty spelling. For an enumerated attribute the rule's
    /// empty value default applies, or the invalid value default when the rule has no empty one.
    Empty,
    /// The selected text rule recognizes an exact nonempty value.
    NonemptyText,
    /// A command keyword begins with two exact ASCII hyphens.
    /// Independent author checks can still reject its boundary spelling.
    CustomCommand,
    /// A keyword matched under the rule's comparison (ASCII case-insensitive for enumerated
    /// attributes, exact for `ol` `type`).
    Keyword {
        /// The keyword spelling from the rule's table that matched the value, which can be an alias
        /// such as `off` for `autocapitalize`.
        matched: &'static str,
        /// The conforming spelling of the state the matched keyword selects, which is what an
        /// emitter writes back.
        canonical: &'static str,
    },
    /// The value is nonempty and matched no keyword of the rule, or is not a Boolean attribute's
    /// own name; the rule's invalid value default, if any, applies.
    Unrecognized,
    /// A Boolean attribute's value equals the attribute's own name, compared ASCII
    /// case-insensitively.
    BooleanName,
    /// The selected rule does not apply to these names, so no value was examined.
    Unapplied,
}

/// A value violates an author requirement of this bounded intrinsic family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAttributeConformanceError {
    /// The value of an enumerated attribute matches no keyword. Every nonempty unmatched value is
    /// refused, whatever the empty value default; an empty value is refused only when the
    /// attribute defines no empty value default.
    UnrecognizedKeyword,
    /// A source-recognized keyword spelling is forbidden in authored markup.
    NonconformingKeyword,
    /// A Boolean attribute's value is neither empty nor the attribute's name compared ASCII
    /// case-insensitively.
    BooleanSpelling,
    /// A `bdo` element has no `dir` attribute.
    BdoDirectionRequired,
    /// A `bdo` element's `dir` attribute is present but is not in the `ltr` or `rtl` state, so
    /// `auto` and invalid values are rejected.
    BdoDirectionMustBeLtrOrRtl,
    /// A specified details name must not be the empty string.
    DetailsNameMustNotBeEmpty,
    /// A specified form name must not be the empty string.
    FormNameMustNotBeEmpty,
    /// A specified direction submission name must not be the empty string.
    DirnameMustNotBeEmpty,
    /// An authored input placeholder contains a forbidden CR or LF code unit.
    InputPlaceholderContainsNewline {
        /// Zero-based UTF-16 code-unit offset of the first CR or LF in the value.
        code_unit_offset: usize,
    },
    /// An HTML id violates its local nonempty/no-ASCII-whitespace author rule.
    InvalidIdToken(HtmlTokenError),
    /// A map name violates its local nonempty/no-ASCII-whitespace author rule.
    InvalidMapNameToken(HtmlTokenError),
    /// A target/name violates HTML's local navigable-target grammar.
    InvalidNavigableTarget(HtmlNavigableTargetError),
    /// The specified encoding label does not match HTML's required UTF-8 spelling.
    InvalidUtf8EncodingLabel(HtmlUtf8EncodingLabelError),
    /// A specified `is` value violates HTML's local custom-element-name predicate.
    InvalidCustomElementName(HtmlCustomElementNameError),
}

impl std::fmt::Display for HtmlAttributeConformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnrecognizedKeyword => f.write_str("attribute value is not a recognized keyword"),
            Self::NonconformingKeyword => {
                f.write_str("attribute keyword is forbidden in authored markup")
            }
            Self::BooleanSpelling => {
                f.write_str("boolean attribute value is neither empty nor the attribute's name")
            }
            Self::BdoDirectionRequired => f.write_str("bdo element requires a dir attribute"),
            Self::BdoDirectionMustBeLtrOrRtl => {
                f.write_str("bdo element dir attribute must be ltr or rtl")
            }
            Self::DetailsNameMustNotBeEmpty => {
                f.write_str("details name attribute must not be empty")
            }
            Self::FormNameMustNotBeEmpty => f.write_str("form name attribute must not be empty"),
            Self::DirnameMustNotBeEmpty => f.write_str("dirname attribute must not be empty"),
            Self::InputPlaceholderContainsNewline { code_unit_offset } => write!(
                f,
                "input placeholder contains a line break at UTF-16 offset {code_unit_offset}"
            ),
            Self::InvalidIdToken(_) => f.write_str("id attribute is not a valid token"),
            Self::InvalidMapNameToken(_) => f.write_str("map name attribute is not a valid token"),
            Self::InvalidNavigableTarget(_) => {
                f.write_str("attribute is not a valid navigable target name")
            }
            Self::InvalidUtf8EncodingLabel(_) => {
                f.write_str("encoding label is not the required UTF-8 spelling")
            }
            Self::InvalidCustomElementName(_) => {
                f.write_str("is attribute is not a valid custom-element name")
            }
        }
    }
}
impl std::error::Error for HtmlAttributeConformanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidIdToken(error) | Self::InvalidMapNameToken(error) => Some(error),
            Self::InvalidNavigableTarget(error) => Some(error),
            Self::InvalidUtf8EncodingLabel(error) => Some(error),
            Self::InvalidCustomElementName(error) => Some(error),
            _ => None,
        }
    }
}

/// Intrinsic value author conformance is independent of state/applicability.
///
/// This also includes bdo's required direction value, but does not validate
/// serialized qualified-name syntax or other tree/element author requirements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeAuthorState {
    /// The attribute is absent, so no author value rule applies.
    Missing,
    /// The value satisfies the rule's author requirements.
    Conforming,
    /// The value violates an author requirement; the payload names it.
    Nonconforming(HtmlAttributeConformanceError),
    /// The rule does not apply to these names, so no author judgment was made.
    Unapplied,
}

/// A typed intrinsic result with the complete exact raw input and source rule.
///
/// `intrinsic()` is not an inherited, computed, used, rendering or runtime state.
/// No-state and an unapplied rule both have no intrinsic value; `applicability()`
/// distinguishes them. Unrecognized values remain exact even when they default.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAttributeReport<T> {
    element: QualifiedName,
    raw_attribute: Option<Attribute>,
    rule: RuleMetadata,
    observation: HtmlAttributeObservation,
    applicability: HtmlAttributeApplicability,
    recognition: HtmlAttributeRecognition,
    author: HtmlAttributeAuthorState,
    context_author: HtmlAttributeContextAuthorState,
    intrinsic: Option<T>,
    supplied_context: SuppliedContext,
}

impl<T> HtmlAttributeReport<T> {
    /// Returns the exact element name, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        &self.element
    }
    /// Returns the exact supplied attribute name/prefix/value, if present.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.raw_attribute.as_ref()
    }
    /// Returns the exact supplied raw value, preserving absence versus empty.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw_attribute.as_ref().map(HtmlAttributeInput::value)
    }
    /// Returns the exact presence of the supplied attribute, before any validity or default is
    /// considered.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.observation
    }
    /// Returns whether the selected rule covers the supplied element and attribute names, which
    /// decides whether the other results are meaningful.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.applicability
    }
    /// Returns how the literal value matched the rule, before defaults. A defaulted intrinsic state
    /// never changes this result.
    pub fn recognition(&self) -> HtmlAttributeRecognition {
        self.recognition
    }
    /// Returns the value's author conformance for this rule, or `Unapplied` when the rule does not
    /// cover the names. Context-dependent requirements are reported separately by
    /// [`HtmlAttributeReport::context_author`].
    pub fn author(&self) -> HtmlAttributeAuthorState {
        self.author
    }
    /// Returns the selected extra author-context observation. Boolean value
    /// spelling remains in author(); both violations can therefore coexist.
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        self.context_author
    }
    /// Returns the attribute's typed intrinsic state, without contextual effects.
    pub fn intrinsic(&self) -> Option<&T> {
        self.intrinsic.as_ref()
    }
    /// Returns the selected standard attribute identity, not raw recognition.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.rule.attribute
    }
    /// Returns the exact supplied script facts, even for an unapplied rule.
    pub fn script_context(&self) -> Option<HtmlScriptAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::Script(context) => Some(*context),
            _ => None,
        }
    }
    /// Returns the exact supplied hyperlink facts, even for an unapplied rule.
    pub fn hyperlink_context(&self) -> Option<HtmlHyperlinkAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::Hyperlink(context) => Some(*context),
            _ => None,
        }
    }
    /// Returns the supplied actual custom-control classification, if any.
    pub fn custom_control_context(&self) -> Option<HtmlCustomControlAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::CustomControl(context) => Some(*context),
            _ => None,
        }
    }
    /// Returns the supplied actual input type state, without inferring a type.
    pub fn input_state(&self) -> Option<HtmlInputType> {
        match &self.supplied_context {
            SuppliedContext::Input(state) => Some(*state),
            _ => None,
        }
    }
    /// Returns supplied actual submit-button classification, if any.
    pub fn button_submitter_context(&self) -> Option<HtmlButtonSubmitterContext> {
        match &self.supplied_context {
            SuppliedContext::ButtonSubmitter(context) => Some(*context),
            _ => None,
        }
    }
    /// Returns the supplied `ismap` ancestor evidence, if this report was built with it.
    pub fn image_map_context(&self) -> Option<HtmlImageMapAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::ImageMap(context) => Some(*context),
            _ => None,
        }
    }
    /// Returns the supplied `img` `alt` observation, if this report was built with it.
    pub fn image_controls_context(&self) -> Option<HtmlImageControlsAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::ImageControls(context) => Some(*context),
            _ => None,
        }
    }
    /// Returns the supplied `rel` stylesheet membership, if this report was built with it.
    pub fn stylesheet_link_context(&self) -> Option<&HtmlStylesheetLinkAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::StylesheetLink(context) => Some(context),
            _ => None,
        }
    }
    /// Returns existing original index row IDs covered by this defining rule.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.rule.rows
    }
    /// Returns the immutable normative defining clause, not an index summary.
    pub fn source_clause(&self) -> &'static str {
        self.rule.source
    }
    /// Returns a separately pinned external keyword definition, when adopted.
    /// This does not adopt external network or effective-policy algorithms.
    pub fn vocabulary_source_clause(&self) -> Option<&'static str> {
        self.rule.vocabulary_source
    }
    /// Returns the defining clause for additional element-context facts.
    pub fn context_source_clause(&self) -> Option<&'static str> {
        if matches!(
            self.rule.attribute,
            KnownHtmlAttribute::Dirname | KnownHtmlAttribute::Placeholder
        ) {
            match KnownHtmlElement::from_name(&self.element) {
                Some(KnownHtmlElement::Input) => {
                    return Some(match &self.supplied_context {
                        SuppliedContext::Input(state) => input_state_source(*state),
                        _ => INPUT_SOURCE,
                    });
                }
                Some(KnownHtmlElement::Textarea) => return Some(self.rule.source),
                _ => {}
            }
        }
        if self.rule.context == Context::PopoverTargetAction
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Input)
        {
            return Some(match &self.supplied_context {
                SuppliedContext::Input(state) => input_state_source(*state),
                _ => INPUT_SOURCE,
            });
        }
        if self.rule.source == image_values::ALT_SOURCE
            || self.rule.source == image_values::SRC_SOURCE
            || self.rule.source == image_values::DIMENSIONS_SOURCE
        {
            match KnownHtmlElement::from_name(&self.element) {
                Some(KnownHtmlElement::Input) => {
                    return Some(match &self.supplied_context {
                        SuppliedContext::Input(state) => input_state_source(*state),
                        _ => INPUT_SOURCE,
                    });
                }
                Some(KnownHtmlElement::Source) => return Some(image_values::SOURCE_ELEMENT_SOURCE),
                _ => {}
            }
        }
        if self.rule.attribute == KnownHtmlAttribute::Colorspace
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Input)
        {
            return Some(match &self.supplied_context {
                SuppliedContext::Input(state) => input_state_source(*state),
                _ => INPUT_SOURCE,
            });
        }
        if self.rule.attribute == KnownHtmlAttribute::Charset
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Meta)
        {
            return Some(META_SOURCE);
        }
        if self.rule.attribute == KnownHtmlAttribute::Wrap
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Textarea)
        {
            return Some(self.rule.source);
        }
        if self.rule.attribute == KnownHtmlAttribute::Scope
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Th)
        {
            return Some(self.rule.source);
        }
        if matches!(
            (
                self.rule.context,
                KnownHtmlElement::from_name(&self.element)
            ),
            (
                Context::ImageMap | Context::ImageControls,
                Some(KnownHtmlElement::Img)
            ) | (Context::StylesheetLink, Some(KnownHtmlElement::Link))
        ) {
            return Some(self.rule.source);
        }
        if matches!(
            self.rule.context,
            Context::InputControl(_) | Context::SubmitterOverride
        ) {
            match KnownHtmlElement::from_name(&self.element) {
                Some(KnownHtmlElement::Input) => {
                    return Some(match &self.supplied_context {
                        SuppliedContext::Input(state) => input_state_source(*state),
                        _ => INPUT_SOURCE,
                    });
                }
                Some(KnownHtmlElement::Button) => {
                    return Some(if self.rule.context == Context::SubmitterOverride {
                        BUTTON_SOURCE
                    } else {
                        BUTTON_TYPE_SOURCE
                    });
                }
                None if html_element_namespace(&self.element)
                    && self.rule.context != Context::SubmitterOverride =>
                {
                    return Some(FORM_ASSOCIATED_CUSTOM_SOURCE);
                }
                _ => {}
            }
        }
        if matches!(self.rule.context, Context::ScriptBoolean(_))
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Script)
        {
            return Some(SCRIPT_SOURCE);
        }
        if self.rule.context == Context::DisabledControls
            && html_element_namespace(&self.element)
            && KnownHtmlElement::from_name(&self.element).is_none()
        {
            return Some(FORM_ASSOCIATED_CUSTOM_SOURCE);
        }
        if matches!(
            self.rule.context,
            Context::Fetching {
                hyperlink: true,
                ..
            }
        ) {
            match KnownHtmlElement::from_name(&self.element) {
                Some(KnownHtmlElement::A) => return Some(HYPERLINK_A_SOURCE),
                Some(KnownHtmlElement::Area) => return Some(HYPERLINK_AREA_SOURCE),
                _ => {}
            }
        }
        if matches!(self.rule.context, Context::Fetching { .. })
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Script)
        {
            return Some(SCRIPT_SOURCE);
        }
        (self.applicability == HtmlAttributeApplicability::Applicable
            && self.rule.context == Context::Direction
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Bdo))
        .then_some(BDO_SOURCE)
    }
    /// Returns the raw value units admitted before name/value processing.
    pub fn input_code_units(&self) -> usize {
        self.raw().map_or(0, DocumentString::len)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Context {
    Global,
    Direction,
    Template,
    Elements(&'static [KnownHtmlElement]),
    DisabledControls,
    InputControl(InputBooleanApplicability),
    SubmitterOverride,
    PopoverTargetAction,
    ScriptBoolean(ScriptApplicability),
    ImageMap,
    ImageControls,
    StylesheetLink,
    Fetching {
        elements: &'static [KnownHtmlElement],
        script: ScriptApplicability,
        hyperlink: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptApplicability {
    ClassicOrModule,
    ExternalClassicOrModule,
    Classic,
    ExternalClassic,
    Async,
}

impl ScriptApplicability {
    fn allows(self, context: HtmlScriptAttributeContext) -> bool {
        use HtmlScriptAttributeContext::{
            ExternalClassic, ExternalModule, InlineClassic, InlineModule,
        };
        match self {
            Self::ClassicOrModule => matches!(
                context,
                ExternalClassic | ExternalModule | InlineClassic | InlineModule
            ),
            Self::ExternalClassicOrModule => matches!(context, ExternalClassic | ExternalModule),
            Self::Classic => matches!(context, ExternalClassic | InlineClassic),
            Self::ExternalClassic => context == ExternalClassic,
            Self::Async => matches!(context, ExternalClassic | ExternalModule | InlineModule),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputBooleanApplicability {
    Alpha,
    Checked,
    Multiple,
    Readonly,
    Required,
    Formnovalidate,
}

impl InputBooleanApplicability {
    fn allows(self, state: HtmlInputType) -> bool {
        use HtmlInputType::{
            Checkbox, Color, Date, Email, File, Image, LocalDateTime, Month, Number, Password,
            Radio, Search, Submit, Telephone, Text, Time, Url, Week,
        };
        let editable = matches!(
            state,
            Text | Search
                | Telephone
                | Url
                | Email
                | Password
                | Date
                | Month
                | Week
                | Time
                | LocalDateTime
                | Number
        );
        match self {
            Self::Alpha => state == Color,
            Self::Checked => matches!(state, Checkbox | Radio),
            Self::Multiple => matches!(state, Email | File),
            Self::Readonly => editable,
            Self::Required => editable || matches!(state, Checkbox | Radio | File),
            Self::Formnovalidate => matches!(state, Submit | Image),
        }
    }

    fn applicability(self, element: Option<KnownHtmlElement>) -> HtmlAttributeApplicability {
        use HtmlAttributeApplicability::{
            Applicable, CustomControlContextRequired, DifferentElement,
        };
        match (self, element) {
            (_, Some(KnownHtmlElement::Input)) => Applicable,
            (Self::Multiple | Self::Required, Some(KnownHtmlElement::Select))
            | (Self::Readonly | Self::Required, Some(KnownHtmlElement::Textarea)) => Applicable,
            (Self::Readonly, None) => CustomControlContextRequired,
            (Self::Formnovalidate, Some(KnownHtmlElement::Button)) => Applicable,
            _ => DifferentElement,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuleMetadata {
    attribute: KnownHtmlAttribute,
    rows: &'static [u16],
    source: &'static str,
    context: Context,
    vocabulary_source: Option<&'static str>,
}

struct Keyword<T> {
    spelling: &'static str,
    canonical: &'static str,
    state: T,
    conforming: bool,
}

struct EnumRule<T: 'static> {
    metadata: RuleMetadata,
    keywords: &'static [Keyword<T>],
    missing: Option<T>,
    invalid: Option<T>,
    empty: Option<T>,
    exact_keywords: bool,
}

macro_rules! source {
    ($anchor:literal) => {
        concat!(
            "https://html.spec.whatwg.org/commit-snapshots/",
            "877d6146487f15b15bc8dd0e3c625063a37b456a/#",
            $anchor
        )
    };
}

const BDO_SOURCE: &str = source!("the-bdo-element");
const META_SOURCE: &str = source!("the-meta-element");
const SCRIPT_SOURCE: &str = source!("the-script-element");
const HYPERLINK_A_SOURCE: &str = source!("the-a-element");
const HYPERLINK_AREA_SOURCE: &str = source!("the-area-element");
const FORM_ASSOCIATED_CUSTOM_SOURCE: &str = source!("form-associated-custom-element");
const INPUT_SOURCE: &str = source!("the-input-element");
const BUTTON_TYPE_SOURCE: &str = source!("attr-button-type");
const BUTTON_SOURCE: &str = source!("the-button-element");

pub(crate) fn input_type_keyword(index: usize) -> Option<(&'static str, HtmlInputType)> {
    INPUT_TYPE
        .keywords
        .get(index)
        .map(|row| (row.spelling, row.state))
}
pub(crate) fn input_colorspace_keyword(
    index: usize,
) -> Option<(&'static str, HtmlInputColorSpace)> {
    input_colorspace::colorspace_keyword(index)
}
pub(crate) fn input_colorspace_default() -> HtmlInputColorSpace {
    input_colorspace::colorspace_default()
}
pub(crate) fn input_colorspace_invalid_default() -> HtmlInputColorSpace {
    input_colorspace::colorspace_invalid_default()
}
impl HtmlInputType {
    /// Selects the HTML missing-value default from the authoritative type rows.
    /// No attribute lookup, text inspection or allocation is performed.
    pub fn missing_value_default() -> Self {
        INPUT_TYPE_DEFAULTS.missing
    }
}

pub(crate) fn input_type_invalid_default() -> HtmlInputType {
    INPUT_TYPE_DEFAULTS.invalid
}

fn input_state_source(state: HtmlInputType) -> &'static str {
    match state {
        HtmlInputType::Hidden => source!("hidden-state-(type=hidden)"),
        HtmlInputType::Text | HtmlInputType::Search => {
            source!("text-(type=text)-state-and-search-state-(type=search)")
        }
        HtmlInputType::Telephone => source!("telephone-state-(type=tel)"),
        HtmlInputType::Url => source!("url-state-(type=url)"),
        HtmlInputType::Email => source!("email-state-(type=email)"),
        HtmlInputType::Password => source!("password-state-(type=password)"),
        HtmlInputType::Date => source!("date-state-(type=date)"),
        HtmlInputType::Month => source!("month-state-(type=month)"),
        HtmlInputType::Week => source!("week-state-(type=week)"),
        HtmlInputType::Time => source!("time-state-(type=time)"),
        HtmlInputType::LocalDateTime => source!("local-date-and-time-state-(type=datetime-local)"),
        HtmlInputType::Number => source!("number-state-(type=number)"),
        HtmlInputType::Range => source!("range-state-(type=range)"),
        HtmlInputType::Color => source!("color-state-(type=color)"),
        HtmlInputType::Checkbox => source!("checkbox-state-(type=checkbox)"),
        HtmlInputType::Radio => source!("radio-button-state-(type=radio)"),
        HtmlInputType::File => source!("file-upload-state-(type=file)"),
        HtmlInputType::Submit => source!("submit-button-state-(type=submit)"),
        HtmlInputType::Image => source!("image-button-state-(type=image)"),
        HtmlInputType::Reset => source!("reset-button-state-(type=reset)"),
        HtmlInputType::Button => source!("button-state-(type=button)"),
    }
}
// Vocabulary-only adoption; raw index.src.html SHA-256:
// a277e78e06418ec60afc6f499aee79d4f3a3a067e0fd97e9886a2d9e3de95eb1.
const REFERRER_POLICY_VOCABULARY: &str = concat!(
    "https://github.com/w3c/webappsec-referrer-policy/blob/",
    "cc435b05ca4a94f7f1a139be5074b168d20014db/index.src.html#L216"
);

macro_rules! keyword_type {
    ($name:ident, $attribute:ident, [$($(#[$vmeta:meta])* $variant:ident => $keyword:literal),+ $(,)?]) => {
        #[doc = concat!("Conforming native keywords for the HTML ", stringify!($attribute), " attribute.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name { $($(#[$vmeta])* $variant),+ }
        impl $name {
            /// Returns a conforming canonical author spelling.
            pub fn canonical_keyword(self) -> &'static str {
                match self { $(Self::$variant => $keyword),+ }
            }
            /// Constructs the standard no-namespace attribute without guessing an element context.
            pub fn into_attribute(self) -> Attribute {
                Attribute::new(KnownHtmlAttribute::$attribute.qualified_name(), self.canonical_keyword())
            }
        }
    };
}

macro_rules! default_state {
    ($name:ident, $value:ident, $default:ident, $default_doc:literal) => {
        #[doc = concat!("Intrinsic ", stringify!($value), " state; the default is not a conforming author keyword.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            #[doc = concat!("The value matched a conforming [`", stringify!($value), "`] keyword.")]
            Keyword($value),
            #[doc = $default_doc]
            $default,
        }
    };
}

keyword_type!(HtmlDir, Dir, [
    /// The `ltr` keyword: the element's contents are explicitly directionally isolated
    /// left-to-right text.
    Ltr => "ltr",
    /// The `rtl` keyword: the element's contents are explicitly directionally isolated
    /// right-to-left text.
    Rtl => "rtl",
    /// The `auto` keyword: the contents are explicitly directionally isolated, with the direction
    /// determined programmatically from them.
    Auto => "auto",
]);
default_state!(
    HtmlDirState,
    HtmlDir,
    Undefined,
    "The Undefined state, which a missing or invalid value selects: the attribute states no direction."
);
keyword_type!(HtmlContentEditable, Contenteditable, [
    /// The `true` keyword, also selected by an empty value: the element is editable.
    True => "true",
    /// The `false` keyword: the element is not editable.
    False => "false",
    /// The `plaintext-only` keyword: only the element's raw text content is editable and rich
    /// formatting is disabled.
    PlaintextOnly => "plaintext-only",
]);
default_state!(
    HtmlContentEditableState,
    HtmlContentEditable,
    Inherit,
    "The Inherit state, which a missing or invalid value selects: the element is editable or not according to its parent."
);
keyword_type!(HtmlDraggable, Draggable, [
    /// The `true` keyword: the element is draggable.
    True => "true",
    /// The `false` keyword: the element is not draggable.
    False => "false",
]);
default_state!(
    HtmlDraggableState,
    HtmlDraggable,
    Auto,
    "The Auto state, which a missing or invalid value selects: neither keyword was specified, so the default drag behavior applies."
);
keyword_type!(HtmlHidden, Hidden, [
    /// The `hidden` keyword, also selected by an empty or invalid value: the element is not
    /// rendered.
    Hidden => "hidden",
    /// The `until-found` keyword: the element is not rendered, but its content stays reachable to
    /// find-in-page and fragment navigation.
    UntilFound => "until-found",
]);
default_state!(
    HtmlHiddenState,
    HtmlHidden,
    NotHidden,
    "The Not Hidden state, which a missing value selects: the attribute is absent."
);
keyword_type!(HtmlPopover, Popover, [
    /// The `auto` popover state, also selected by an empty value.
    Auto => "auto",
    /// The `manual` popover state, also selected by an invalid value.
    Manual => "manual",
    /// The `hint` popover state.
    Hint => "hint",
]);
default_state!(
    HtmlPopoverState,
    HtmlPopover,
    NoPopover,
    "The No Popover state, which a missing value selects: the element is not a popover."
);
keyword_type!(HtmlTranslate, Translate, [
    /// The `yes` keyword, also selected by an empty value: the element's translatable text is
    /// translated when the page is localized.
    Yes => "yes",
    /// The `no` keyword: the element's translatable text is left unchanged when the page is
    /// localized.
    No => "no",
]);
default_state!(
    HtmlTranslateState,
    HtmlTranslate,
    Inherit,
    "The Inherit state, which a missing or invalid value selects: the translation mode comes from the parent."
);
keyword_type!(HtmlSpellcheck, Spellcheck, [
    /// The `true` keyword, also selected by an empty value: spelling and grammar are checked.
    True => "true",
    /// The `false` keyword: spelling and grammar are not checked.
    False => "false",
]);
default_state!(
    HtmlSpellcheckState,
    HtmlSpellcheck,
    Default,
    "The Default state, which a missing or invalid value selects: the element acts according to its default behavior, possibly based on its parent's own spellcheck state."
);
keyword_type!(HtmlAutocapitalize, Autocapitalize, [
    /// No automatic capitalization. The `none` keyword and its alias `off` select it.
    None => "none",
    /// Capitalize the first letter of each sentence. The `sentences` keyword and its alias `on`
    /// select it, and so does an invalid value.
    Sentences => "sentences",
    /// The `words` keyword: capitalize the first letter of each word.
    Words => "words",
    /// The `characters` keyword: capitalize every character.
    Characters => "characters",
]);
default_state!(
    HtmlAutocapitalizeState,
    HtmlAutocapitalize,
    Default,
    "The Default state, which a missing value selects: the user agent's own autocapitalization behavior applies."
);
keyword_type!(HtmlAutocorrect, Autocorrect, [
    /// The `on` keyword, also selected by a missing, empty or invalid value: automatic text
    /// correction is enabled.
    On => "on",
    /// The `off` keyword: automatic text correction is disabled.
    Off => "off",
]);
keyword_type!(HtmlWritingSuggestions, Writingsuggestions, [
    /// The `true` keyword, also selected by an empty or invalid value: writing suggestions should
    /// be offered on the element.
    True => "true",
    /// The `false` keyword: writing suggestions should not be offered on the element.
    False => "false",
]);
default_state!(
    HtmlWritingSuggestionsState,
    HtmlWritingSuggestions,
    Default,
    "The Default state, which a missing value selects: the element acts according to its default behavior, possibly based on its parent's own state."
);
keyword_type!(HtmlInputMode, Inputmode, [
    /// The `none` keyword: no virtual keyboard is wanted, for content that supplies its own input
    /// method.
    None => "none",
    /// The `text` keyword: a standard text-entry keyboard for the user's locale.
    Text => "text",
    /// The `tel` keyword: a telephone-number keypad.
    Tel => "tel",
    /// The `url` keyword: a keyboard suited to entering a URL.
    Url => "url",
    /// The `email` keyword: a keyboard suited to entering an email address.
    Email => "email",
    /// The `numeric` keyword: a keyboard for entering digits.
    Numeric => "numeric",
    /// The `decimal` keyword: a keyboard for entering digits and a decimal separator.
    Decimal => "decimal",
    /// The `search` keyword: a keyboard suited to entering a search query.
    Search => "search",
]);
keyword_type!(HtmlEnterKeyHint, Enterkeyhint, [
    /// The `enter` keyword: label the enter key for inserting a new line.
    Enter => "enter",
    /// The `done` keyword: label the enter key as finishing input.
    Done => "done",
    /// The `go` keyword: label the enter key as going to the target the text names.
    Go => "go",
    /// The `next` keyword: label the enter key as moving to the next field.
    Next => "next",
    /// The `previous` keyword: label the enter key as moving to the previous field.
    Previous => "previous",
    /// The `search` keyword: label the enter key as running a search.
    Search => "search",
    /// The `send` keyword: label the enter key as sending the text.
    Send => "send",
]);

keyword_type!(HtmlClosedBy, Closedby, [
    /// The `any` keyword: close requests and clicks outside the dialog close it.
    Any => "any",
    /// The `closerequest` keyword: close requests close the dialog.
    CloseRequest => "closerequest",
    /// The `none` keyword: no user action closes the dialog automatically.
    None => "none",
]);
default_state!(
    HtmlClosedByState,
    HtmlClosedBy,
    Auto,
    "The Auto state, which a missing or invalid value selects: behaves as Close Request when the dialog was shown modally and as None otherwise."
);
keyword_type!(HtmlFormAutocomplete, Autocomplete, [
    /// The `on` keyword, also selected by a missing or invalid value: the user agent may prefill
    /// the form's controls.
    On => "on",
    /// The `off` keyword: the user agent should not prefill the form's controls.
    Off => "off",
]);
keyword_type!(HtmlDecoding, Decoding, [
    /// The `sync` keyword: prefer decoding the image synchronously for atomic presentation with
    /// other content.
    Sync => "sync",
    /// The `async` keyword: prefer decoding the image asynchronously to avoid delaying other
    /// content.
    Async => "async",
    /// The `auto` keyword, also selected by a missing or invalid value: no decoding preference.
    Auto => "auto",
]);
keyword_type!(HtmlLoading, Loading, [
    /// The `lazy` keyword: the resource's fetch may be deferred until it is likely to be needed.
    Lazy => "lazy",
    /// The `eager` keyword, also selected by a missing or invalid value: fetch the resource
    /// immediately.
    Eager => "eager",
]);
keyword_type!(HtmlPreload, Preload, [
    /// The `auto` keyword, also selected by an empty value: the user agent may download the whole
    /// resource optimistically.
    Auto => "auto",
    /// The `none` keyword: the author does not expect the media resource to be needed, or wants to
    /// minimize server traffic.
    None => "none",
    /// The `metadata` keyword: the author expects only metadata such as dimensions, track list and
    /// duration to be needed.
    Metadata => "metadata",
]);

/// Intrinsic preload hint, with no invented implementation-defined choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPreloadState {
    /// The value matched a conforming `preload` keyword.
    Keyword(HtmlPreload),
    /// HTML supplies no selected preload state for missing/invalid values.
    ImplementationDefined,
}

keyword_type!(HtmlFormMethod, Method, [
    /// The `get` keyword, also selected by a missing or invalid value: the form submits with the
    /// HTTP GET method.
    Get => "get",
    /// The `post` keyword: the form submits with the HTTP POST method.
    Post => "post",
    /// The `dialog` keyword: the form closes the dialog that contains it, if any, and otherwise
    /// does not submit.
    Dialog => "dialog",
]);
keyword_type!(HtmlFormEnctype, Enctype, [
    /// The `application/x-www-form-urlencoded` keyword, also selected by a missing or invalid
    /// value.
    UrlEncoded => "application/x-www-form-urlencoded",
    /// The `multipart/form-data` keyword.
    MultipartFormData => "multipart/form-data",
    /// The `text/plain` keyword.
    PlainText => "text/plain",
]);
keyword_type!(HtmlTrackKind, Kind, [
    /// The `subtitles` keyword, also selected by a missing value: a transcription or translation of
    /// the dialogue, overlaid on the video.
    Subtitles => "subtitles",
    /// The `captions` keyword: a transcription of the dialogue, sound effects and other relevant
    /// audio, for when sound is unavailable.
    Captions => "captions",
    /// The `descriptions` keyword: textual descriptions of the video component, intended for audio
    /// synthesis.
    Descriptions => "descriptions",
    /// The `chapters` keyword: chapter titles intended to aid navigation of the media resource.
    Chapters => "chapters",
    /// The `metadata` keyword, also selected by an invalid value: tracks intended for use from
    /// script and not displayed by the user agent.
    Metadata => "metadata",
]);
keyword_type!(HtmlOlType, Type, [
    /// The `1` marker: decimal numbers. A missing or invalid value selects it.
    Decimal => "1",
    /// The `a` marker: the lowercase Latin alphabet.
    LowerAlpha => "a",
    /// The `A` marker: the uppercase Latin alphabet.
    UpperAlpha => "A",
    /// The `i` marker: lowercase Roman numerals.
    LowerRoman => "i",
    /// The `I` marker: uppercase Roman numerals.
    UpperRoman => "I",
]);
keyword_type!(HtmlInputType, Type, [
    /// The Hidden state (`hidden`): an arbitrary string with no control shown to the user.
    Hidden => "hidden",
    /// The Text state (`text`), also selected by a missing or invalid value: text with no line
    /// breaks, shown as a text control.
    Text => "text",
    /// The Search state (`search`): text with no line breaks, shown as a search control.
    Search => "search",
    /// The Telephone state (`tel`): text with no line breaks, shown as a text control.
    Telephone => "tel",
    /// The URL state (`url`): an absolute URL, shown as a text control.
    Url => "url",
    /// The Email state (`email`): an email address or a list of email addresses, shown as a text
    /// control.
    Email => "email",
    /// The Password state (`password`): text with no line breaks that is sensitive information,
    /// shown as a text control that obscures data entry.
    Password => "password",
    /// The Date state (`date`): a date made of a year, month and day with no time zone.
    Date => "date",
    /// The Month state (`month`): a date made of a year and a month with no time zone.
    Month => "month",
    /// The Week state (`week`): a date made of a week-year number and a week number with no time
    /// zone.
    Week => "week",
    /// The Time state (`time`): a time with no time zone, made of hour, minute, seconds and
    /// fractional seconds.
    Time => "time",
    /// The Local Date and Time state (`datetime-local`): a date and time with no time zone.
    LocalDateTime => "datetime-local",
    /// The Number state (`number`): a numerical value, shown as a text control or spinner.
    Number => "number",
    /// The Range state (`range`): a numerical value whose exact value is not important, shown as a
    /// slider or similar.
    Range => "range",
    /// The Color state (`color`): a color value, shown as a color picker.
    Color => "color",
    /// The Checkbox state (`checkbox`): a set of zero or more values from a predefined list, shown
    /// as a checkbox.
    Checkbox => "checkbox",
    /// The Radio Button state (`radio`): an enumerated value, shown as a radio button.
    Radio => "radio",
    /// The File Upload state (`file`): zero or more files, each with a MIME type and optionally a
    /// filename.
    File => "file",
    /// The Submit Button state (`submit`): an enumerated value that must be the last value selected
    /// and that initiates form submission.
    Submit => "submit",
    /// The Image Button state (`image`): a coordinate relative to an image's size that initiates
    /// form submission, shown as a clickable image or button.
    Image => "image",
    /// The Reset Button state (`reset`): a button with no data type that resets the form.
    Reset => "reset",
    /// The Button state (`button`): a button with no data type and no default behavior.
    Button => "button",
]);
keyword_type!(HtmlButtonType, Type, [
    /// The `submit` keyword: the Submit Button state, which submits the form.
    Submit => "submit",
    /// The `reset` keyword: the Reset Button state, which resets the form.
    Reset => "reset",
    /// The `button` keyword: the Button state, which has no default behavior.
    Button => "button",
]);
default_state!(
    HtmlButtonTypeState,
    HtmlButtonType,
    Auto,
    "The Auto state, which a missing or invalid value selects: the button is a submit button only when it has neither a `command` nor a `commandfor` attribute and its parent is not a `select` element."
);
keyword_type!(HtmlCrossOrigin, Crossorigin, [
    /// The `anonymous` keyword, also selected by an empty or invalid value: the fetch uses CORS
    /// mode with the same-origin credentials mode.
    Anonymous => "anonymous",
    /// The `use-credentials` keyword: the fetch uses CORS mode with the include credentials mode.
    UseCredentials => "use-credentials",
]);
default_state!(
    HtmlCrossOriginState,
    HtmlCrossOrigin,
    NoCors,
    "The No CORS state, which a missing value selects: the attribute does not request a CORS fetch."
);
keyword_type!(HtmlFetchPriority, Fetchpriority, [
    /// The `high` keyword: fetch the resource at a higher priority than its default.
    High => "high",
    /// The `low` keyword: fetch the resource at a lower priority than its default.
    Low => "low",
    /// The `auto` keyword, also selected by a missing or invalid value: no preference among
    /// fetches.
    Auto => "auto",
]);
keyword_type!(HtmlReferrerPolicy, Referrerpolicy, [
    /// The empty string: no policy is set, so the fallback policy applies. A missing or invalid
    /// value selects it.
    Empty => "",
    /// The `no-referrer` policy.
    NoReferrer => "no-referrer",
    /// The `no-referrer-when-downgrade` policy.
    NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
    /// The `same-origin` policy.
    SameOrigin => "same-origin",
    /// The `origin` policy.
    Origin => "origin",
    /// The `strict-origin` policy.
    StrictOrigin => "strict-origin",
    /// The `origin-when-cross-origin` policy.
    OriginWhenCrossOrigin => "origin-when-cross-origin",
    /// The `strict-origin-when-cross-origin` policy.
    StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
    /// The `unsafe-url` policy.
    UnsafeUrl => "unsafe-url",
]);

impl HtmlShadowRootMode {
    /// Constructs a conforming template shadowrootmode keyword attribute.
    pub fn into_shadowrootmode_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Shadowrootmode.qualified_name(),
            match self {
                Self::Open => "open",
                Self::Closed => "closed",
            },
        )
    }
}

impl HtmlSlotAssignment {
    /// Constructs a conforming template shadowrootslotassignment keyword attribute.
    pub fn into_shadowrootslotassignment_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Shadowrootslotassignment.qualified_name(),
            match self {
                Self::Named => "named",
                Self::Manual => "manual",
            },
        )
    }
}

// One authoritative authored rules declaration. Its typed records drive both
// metadata and execution; index rows are provenance references, never grammars.
/// The values an enumerated attribute selects when it is missing, empty or invalid, for a rule that
/// defines the missing and invalid values. A rule built with [`total_enum_rule!`] holds exactly these
/// as its defaults, so a caller that needs a value is never handed an absent default.
#[derive(Clone, Copy)]
struct TotalEnumDefaults<T> {
    missing: T,
    invalid: T,
    empty: Option<T>,
}

/// Defines an [`EnumRule`] whose missing and invalid values are defined, together with the
/// [`TotalEnumDefaults`] constant that the rule's own defaults are taken from.
macro_rules! total_enum_rule {
    ($name:ident, $defaults:ident, $type:ty, $attribute:ident, $context:expr, [$($row:literal),+], $source:literal,
     $missing:expr, $invalid:expr, $empty:expr, [$($keyword:tt)+] $(, vocabulary: $vocabulary:expr)?) => {
        const $defaults: TotalEnumDefaults<$type> = TotalEnumDefaults {
            missing: $missing,
            invalid: $invalid,
            empty: $empty,
        };
        enum_rule!($name, $type, $attribute, $context, [$($row),+], $source,
            Some($defaults.missing), Some($defaults.invalid), $defaults.empty, [$($keyword)+]
            $(, vocabulary: $vocabulary)?);
    };
}

macro_rules! enum_rule {
    ($name:ident, $type:ty, $attribute:ident, $context:expr, [$($row:literal),+], $source:literal,
     $missing:expr, $invalid:expr, $empty:expr, [$($spelling:literal => ($canonical:literal, $state:expr)),+ $(,)?]
     $(, vocabulary: $vocabulary:expr)?) => {
        const $name: EnumRule<$type> = EnumRule {
            metadata: RuleMetadata { attribute: KnownHtmlAttribute::$attribute, rows: &[$($row),+],
                source: source!($source), context: $context,
                vocabulary_source: enum_rule!(@vocabulary $($vocabulary)?) },
            keywords: &[$(Keyword { spelling: $spelling, canonical: $canonical, state: $state, conforming: true }),+],
            missing: $missing, invalid: $invalid, empty: $empty, exact_keywords: false,
        };
    };
    (@vocabulary) => { None };
    (@vocabulary $source:expr) => { Some($source) };
}

enum_rule!(DIR, HtmlDirState, Dir, Context::Direction, [41, 42], "attr-dir",
    Some(HtmlDirState::Undefined), Some(HtmlDirState::Undefined), None,
    ["ltr" => ("ltr", HtmlDirState::Keyword(HtmlDir::Ltr)),
     "rtl" => ("rtl", HtmlDirState::Keyword(HtmlDir::Rtl)),
     "auto" => ("auto", HtmlDirState::Keyword(HtmlDir::Auto))]);
enum_rule!(CONTENTEDITABLE, HtmlContentEditableState, Contenteditable, Context::Global, [31], "attr-contenteditable",
    Some(HtmlContentEditableState::Inherit), Some(HtmlContentEditableState::Inherit), Some(HtmlContentEditableState::Keyword(HtmlContentEditable::True)),
    ["true" => ("true", HtmlContentEditableState::Keyword(HtmlContentEditable::True)),
     "false" => ("false", HtmlContentEditableState::Keyword(HtmlContentEditable::False)),
     "plaintext-only" => ("plaintext-only", HtmlContentEditableState::Keyword(HtmlContentEditable::PlaintextOnly))]);
enum_rule!(DRAGGABLE, HtmlDraggableState, Draggable, Context::Global, [48], "attr-draggable",
    Some(HtmlDraggableState::Auto), Some(HtmlDraggableState::Auto), None,
    ["true" => ("true", HtmlDraggableState::Keyword(HtmlDraggable::True)),
     "false" => ("false", HtmlDraggableState::Keyword(HtmlDraggable::False))]);
enum_rule!(HIDDEN, HtmlHiddenState, Hidden, Context::Global, [65], "attr-hidden",
    Some(HtmlHiddenState::NotHidden), Some(HtmlHiddenState::Keyword(HtmlHidden::Hidden)), Some(HtmlHiddenState::Keyword(HtmlHidden::Hidden)),
    ["hidden" => ("hidden", HtmlHiddenState::Keyword(HtmlHidden::Hidden)),
     "until-found" => ("until-found", HtmlHiddenState::Keyword(HtmlHidden::UntilFound))]);
enum_rule!(POPOVER, HtmlPopoverState, Popover, Context::Global, [119], "attr-popover",
    Some(HtmlPopoverState::NoPopover), Some(HtmlPopoverState::Keyword(HtmlPopover::Manual)), Some(HtmlPopoverState::Keyword(HtmlPopover::Auto)),
    ["auto" => ("auto", HtmlPopoverState::Keyword(HtmlPopover::Auto)),
     "manual" => ("manual", HtmlPopoverState::Keyword(HtmlPopover::Manual)),
     "hint" => ("hint", HtmlPopoverState::Keyword(HtmlPopover::Hint))]);
enum_rule!(TRANSLATE, HtmlTranslateState, Translate, Context::Global, [165], "attr-translate",
    Some(HtmlTranslateState::Inherit), Some(HtmlTranslateState::Inherit), Some(HtmlTranslateState::Keyword(HtmlTranslate::Yes)),
    ["yes" => ("yes", HtmlTranslateState::Keyword(HtmlTranslate::Yes)),
     "no" => ("no", HtmlTranslateState::Keyword(HtmlTranslate::No))]);
enum_rule!(SPELLCHECK, HtmlSpellcheckState, Spellcheck, Context::Global, [148], "attr-spellcheck",
    Some(HtmlSpellcheckState::Default), Some(HtmlSpellcheckState::Default), Some(HtmlSpellcheckState::Keyword(HtmlSpellcheck::True)),
    ["true" => ("true", HtmlSpellcheckState::Keyword(HtmlSpellcheck::True)),
     "false" => ("false", HtmlSpellcheckState::Keyword(HtmlSpellcheck::False))]);
enum_rule!(AUTOCAPITALIZE, HtmlAutocapitalizeState, Autocapitalize, Context::Global, [12], "attr-autocapitalize",
    Some(HtmlAutocapitalizeState::Default), Some(HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)), None,
    ["off" => ("none", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::None)),
     "none" => ("none", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::None)),
     "on" => ("sentences", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)),
     "sentences" => ("sentences", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)),
     "words" => ("words", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Words)),
     "characters" => ("characters", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Characters))]);
enum_rule!(AUTOCORRECT, HtmlAutocorrect, Autocorrect, Context::Global, [15], "attr-autocorrect",
    Some(HtmlAutocorrect::On), Some(HtmlAutocorrect::On), Some(HtmlAutocorrect::On),
    ["on" => ("on", HtmlAutocorrect::On), "off" => ("off", HtmlAutocorrect::Off)]);
enum_rule!(WRITINGSUGGESTIONS, HtmlWritingSuggestionsState, Writingsuggestions, Context::Global, [180], "attr-writingsuggestions",
    Some(HtmlWritingSuggestionsState::Default), Some(HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)), Some(HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)),
    ["true" => ("true", HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)),
     "false" => ("false", HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::False))]);
enum_rule!(INPUTMODE, HtmlInputMode, Inputmode, Context::Global, [76], "attr-inputmode",
    None, None, None,
    ["none" => ("none", HtmlInputMode::None), "text" => ("text", HtmlInputMode::Text),
     "tel" => ("tel", HtmlInputMode::Tel), "url" => ("url", HtmlInputMode::Url),
     "email" => ("email", HtmlInputMode::Email), "numeric" => ("numeric", HtmlInputMode::Numeric),
     "decimal" => ("decimal", HtmlInputMode::Decimal), "search" => ("search", HtmlInputMode::Search)]);
enum_rule!(ENTERKEYHINT, HtmlEnterKeyHint, Enterkeyhint, Context::Global, [50], "attr-enterkeyhint",
    None, None, None,
    ["enter" => ("enter", HtmlEnterKeyHint::Enter), "done" => ("done", HtmlEnterKeyHint::Done),
     "go" => ("go", HtmlEnterKeyHint::Go), "next" => ("next", HtmlEnterKeyHint::Next),
     "previous" => ("previous", HtmlEnterKeyHint::Previous), "search" => ("search", HtmlEnterKeyHint::Search),
     "send" => ("send", HtmlEnterKeyHint::Send)]);
enum_rule!(SHADOWROOTMODE, HtmlShadowRootMode, Shadowrootmode, Context::Template, [139], "attr-template-shadowrootmode",
    None, None, None,
    ["open" => ("open", HtmlShadowRootMode::Open), "closed" => ("closed", HtmlShadowRootMode::Closed)]);
enum_rule!(SHADOWROOTSLOTASSIGNMENT, HtmlSlotAssignment, Shadowrootslotassignment, Context::Template, [141], "attr-template-shadowrootslotassignment",
    Some(HtmlSlotAssignment::Named), Some(HtmlSlotAssignment::Named), None,
    ["named" => ("named", HtmlSlotAssignment::Named), "manual" => ("manual", HtmlSlotAssignment::Manual)]);

enum_rule!(CLOSEDBY, HtmlClosedByState, Closedby, Context::Elements(&[KnownHtmlElement::Dialog]), [23], "attr-dialog-closedby",
    Some(HtmlClosedByState::Auto), Some(HtmlClosedByState::Auto), None,
    ["any" => ("any", HtmlClosedByState::Keyword(HtmlClosedBy::Any)),
     "closerequest" => ("closerequest", HtmlClosedByState::Keyword(HtmlClosedBy::CloseRequest)),
     "none" => ("none", HtmlClosedByState::Keyword(HtmlClosedBy::None))]);
enum_rule!(FORM_AUTOCOMPLETE, HtmlFormAutocomplete, Autocomplete, Context::Elements(&[KnownHtmlElement::Form]), [13], "attr-form-autocomplete",
    Some(HtmlFormAutocomplete::On), Some(HtmlFormAutocomplete::On), None,
    ["on" => ("on", HtmlFormAutocomplete::On), "off" => ("off", HtmlFormAutocomplete::Off)]);
enum_rule!(DECODING, HtmlDecoding, Decoding, Context::Elements(&[KnownHtmlElement::Img]), [38], "attr-img-decoding",
    Some(HtmlDecoding::Auto), Some(HtmlDecoding::Auto), None,
    ["sync" => ("sync", HtmlDecoding::Sync), "async" => ("async", HtmlDecoding::Async), "auto" => ("auto", HtmlDecoding::Auto)]);
// The shared defining grammar is applied by attr-img-loading,
// attr-iframe-loading and attr-media-loading (audio/video).
enum_rule!(LOADING, HtmlLoading, Loading, Context::Elements(&[KnownHtmlElement::Img, KnownHtmlElement::Iframe, KnownHtmlElement::Audio, KnownHtmlElement::Video]), [89], "lazy-loading-attributes",
    Some(HtmlLoading::Eager), Some(HtmlLoading::Eager), None,
    ["lazy" => ("lazy", HtmlLoading::Lazy), "eager" => ("eager", HtmlLoading::Eager)]);
enum_rule!(PRELOAD, HtmlPreloadState, Preload, Context::Elements(&[KnownHtmlElement::Audio, KnownHtmlElement::Video]), [123], "attr-media-preload",
    Some(HtmlPreloadState::ImplementationDefined), Some(HtmlPreloadState::ImplementationDefined), Some(HtmlPreloadState::Keyword(HtmlPreload::Auto)),
    ["auto" => ("auto", HtmlPreloadState::Keyword(HtmlPreload::Auto)),
     "none" => ("none", HtmlPreloadState::Keyword(HtmlPreload::None)),
     "metadata" => ("metadata", HtmlPreloadState::Keyword(HtmlPreload::Metadata))]);
enum_rule!(FORM_METHOD, HtmlFormMethod, Method, Context::Elements(&[KnownHtmlElement::Form]), [96], "attr-fs-method",
    Some(HtmlFormMethod::Get), Some(HtmlFormMethod::Get), None,
    ["get" => ("get", HtmlFormMethod::Get), "post" => ("post", HtmlFormMethod::Post), "dialog" => ("dialog", HtmlFormMethod::Dialog)]);
enum_rule!(FORM_ENCTYPE, HtmlFormEnctype, Enctype, Context::Elements(&[KnownHtmlElement::Form]), [49], "attr-fs-enctype",
    Some(HtmlFormEnctype::UrlEncoded), Some(HtmlFormEnctype::UrlEncoded), None,
    ["application/x-www-form-urlencoded" => ("application/x-www-form-urlencoded", HtmlFormEnctype::UrlEncoded),
     "multipart/form-data" => ("multipart/form-data", HtmlFormEnctype::MultipartFormData), "text/plain" => ("text/plain", HtmlFormEnctype::PlainText)]);
enum_rule!(TRACK_KIND, HtmlTrackKind, Kind, Context::Elements(&[KnownHtmlElement::Track]), [85], "attr-track-kind",
    Some(HtmlTrackKind::Subtitles), Some(HtmlTrackKind::Metadata), None,
    ["subtitles" => ("subtitles", HtmlTrackKind::Subtitles), "captions" => ("captions", HtmlTrackKind::Captions),
     "descriptions" => ("descriptions", HtmlTrackKind::Descriptions), "chapters" => ("chapters", HtmlTrackKind::Chapters), "metadata" => ("metadata", HtmlTrackKind::Metadata)]);
total_enum_rule!(INPUT_TYPE, INPUT_TYPE_DEFAULTS, HtmlInputType, Type, Context::Elements(&[KnownHtmlElement::Input]), [169], "attr-input-type",
    HtmlInputType::Text, HtmlInputType::Text, None,
    ["hidden" => ("hidden", HtmlInputType::Hidden), "text" => ("text", HtmlInputType::Text),
     "search" => ("search", HtmlInputType::Search), "tel" => ("tel", HtmlInputType::Telephone),
     "url" => ("url", HtmlInputType::Url), "email" => ("email", HtmlInputType::Email),
     "password" => ("password", HtmlInputType::Password), "date" => ("date", HtmlInputType::Date),
     "month" => ("month", HtmlInputType::Month), "week" => ("week", HtmlInputType::Week),
     "time" => ("time", HtmlInputType::Time), "datetime-local" => ("datetime-local", HtmlInputType::LocalDateTime),
     "number" => ("number", HtmlInputType::Number), "range" => ("range", HtmlInputType::Range),
     "color" => ("color", HtmlInputType::Color), "checkbox" => ("checkbox", HtmlInputType::Checkbox),
     "radio" => ("radio", HtmlInputType::Radio), "file" => ("file", HtmlInputType::File),
     "submit" => ("submit", HtmlInputType::Submit), "image" => ("image", HtmlInputType::Image),
     "reset" => ("reset", HtmlInputType::Reset), "button" => ("button", HtmlInputType::Button)]);
enum_rule!(BUTTON_TYPE, HtmlButtonTypeState, Type, Context::Elements(&[KnownHtmlElement::Button]), [167], "attr-button-type",
    Some(HtmlButtonTypeState::Auto), Some(HtmlButtonTypeState::Auto), None,
    ["submit" => ("submit", HtmlButtonTypeState::Keyword(HtmlButtonType::Submit)),
     "reset" => ("reset", HtmlButtonTypeState::Keyword(HtmlButtonType::Reset)),
     "button" => ("button", HtmlButtonTypeState::Keyword(HtmlButtonType::Button))]);
total_enum_rule!(CROSSORIGIN, CROSSORIGIN_DEFAULTS, HtmlCrossOriginState, Crossorigin,
    Context::Fetching { elements: &[KnownHtmlElement::Audio, KnownHtmlElement::Video, KnownHtmlElement::Img, KnownHtmlElement::Link, KnownHtmlElement::Script], script: ScriptApplicability::ClassicOrModule, hyperlink: false },
    [34], "cors-settings-attributes",
    HtmlCrossOriginState::NoCors, HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous), Some(HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)),
    ["anonymous" => ("anonymous", HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)),
     "use-credentials" => ("use-credentials", HtmlCrossOriginState::Keyword(HtmlCrossOrigin::UseCredentials))]);
enum_rule!(FETCHPRIORITY, HtmlFetchPriority, Fetchpriority,
    Context::Fetching { elements: &[KnownHtmlElement::Img, KnownHtmlElement::Link, KnownHtmlElement::Script], script: ScriptApplicability::ExternalClassicOrModule, hyperlink: false },
    [51], "fetch-priority-attributes",
    Some(HtmlFetchPriority::Auto), Some(HtmlFetchPriority::Auto), None,
    ["high" => ("high", HtmlFetchPriority::High), "low" => ("low", HtmlFetchPriority::Low), "auto" => ("auto", HtmlFetchPriority::Auto)]);
total_enum_rule!(REFERRERPOLICY, REFERRERPOLICY_DEFAULTS, HtmlReferrerPolicy, Referrerpolicy,
    Context::Fetching { elements: &[KnownHtmlElement::A, KnownHtmlElement::Area, KnownHtmlElement::Iframe, KnownHtmlElement::Img, KnownHtmlElement::Link, KnownHtmlElement::Script], script: ScriptApplicability::ClassicOrModule, hyperlink: true },
    [126], "referrer-policy-attributes",
    HtmlReferrerPolicy::Empty, HtmlReferrerPolicy::Empty, None,
    ["" => ("", HtmlReferrerPolicy::Empty),
     "no-referrer" => ("no-referrer", HtmlReferrerPolicy::NoReferrer),
     "no-referrer-when-downgrade" => ("no-referrer-when-downgrade", HtmlReferrerPolicy::NoReferrerWhenDowngrade),
     "same-origin" => ("same-origin", HtmlReferrerPolicy::SameOrigin),
     "origin" => ("origin", HtmlReferrerPolicy::Origin),
     "strict-origin" => ("strict-origin", HtmlReferrerPolicy::StrictOrigin),
     "origin-when-cross-origin" => ("origin-when-cross-origin", HtmlReferrerPolicy::OriginWhenCrossOrigin),
     "strict-origin-when-cross-origin" => ("strict-origin-when-cross-origin", HtmlReferrerPolicy::StrictOriginWhenCrossOrigin),
     "unsafe-url" => ("unsafe-url", HtmlReferrerPolicy::UnsafeUrl)],
    vocabulary: REFERRER_POLICY_VOCABULARY);

// ol[type] is an exact character table, not an ASCII-insensitive enumeration.
const OL_TYPE: EnumRule<HtmlOlType> = EnumRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Type,
        rows: &[170],
        source: source!("attr-ol-type"),
        context: Context::Elements(&[KnownHtmlElement::Ol]),
        vocabulary_source: None,
    },
    keywords: &[
        Keyword {
            spelling: "1",
            canonical: "1",
            conforming: true,
            state: HtmlOlType::Decimal,
        },
        Keyword {
            spelling: "a",
            canonical: "a",
            conforming: true,
            state: HtmlOlType::LowerAlpha,
        },
        Keyword {
            spelling: "A",
            canonical: "A",
            conforming: true,
            state: HtmlOlType::UpperAlpha,
        },
        Keyword {
            spelling: "i",
            canonical: "i",
            conforming: true,
            state: HtmlOlType::LowerRoman,
        },
        Keyword {
            spelling: "I",
            canonical: "I",
            conforming: true,
            state: HtmlOlType::UpperRoman,
        },
    ],
    missing: Some(HtmlOlType::Decimal),
    invalid: Some(HtmlOlType::Decimal),
    empty: None,
    exact_keywords: true,
};

/// The Boolean attribute contexts admitted by this bounded intrinsic overlay.
///
/// This is not the complete HTML Boolean attribute inventory. Presence describes
/// only the content attribute; it never claims focus, inertness, an item model,
/// heading computation or a created shadow root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBooleanAttribute {
    /// The global `autofocus` attribute.
    Autofocus,
    /// The global `inert` attribute.
    Inert,
    /// The global `itemscope` attribute that creates a microdata item.
    Itemscope,
    /// The global `headingreset` attribute.
    HeadingReset,
    /// The `template` `shadowrootclonable` attribute.
    ShadowRootClonable,
    /// The `template` `shadowrootcustomelementregistry` attribute.
    ShadowRootCustomElementRegistry,
    /// The `template` `shadowrootdelegatesfocus` attribute.
    ShadowRootDelegatesFocus,
    /// The `template` `shadowrootserializable` attribute.
    ShadowRootSerializable,
    /// The `iframe` `allowfullscreen` attribute.
    AllowFullscreen,
    /// The `audio` and `video` `autoplay` attribute.
    Autoplay,
    /// The audio/video controls rule; this does not select img's controls rule.
    MediaControls,
    /// The `track` `default` attribute.
    TrackDefault,
    /// The `audio` and `video` `loop` attribute.
    Loop,
    /// The `audio` and `video` `muted` attribute.
    Muted,
    /// The `form` `novalidate` attribute.
    Novalidate,
    /// The `details` `open` attribute.
    DetailsOpen,
    /// The `dialog` `open` attribute.
    DialogOpen,
    /// The `video` `playsinline` attribute.
    PlaysInline,
    /// The `ol` `reversed` attribute.
    Reversed,
    /// The `option` `selected` attribute.
    Selected,
    /// Local disabled content presence, with defining source chosen by context.
    /// This does not compute inherited or actual disabledness.
    Disabled,
    /// Color-input alpha content presence, not a CSS color interpretation.
    Alpha,
    /// Default checkedness content presence, not current checkedness.
    Checked,
    /// The `multiple` attribute of `input` in the Email or File state and of `select`.
    Multiple,
    /// The `readonly` attribute of editable `input` states, of `textarea` and of a classified
    /// form-associated custom element (see `readonly_for_custom_control`).
    Readonly,
    /// The `required` attribute of `input`, `select` and `textarea`.
    Required,
    /// Content presence on source-qualified submitters, not no-validate state.
    Formnovalidate,
    /// Script content presence, not scheduling, force-async or execution state.
    Async,
    /// The external classic `script` `defer` attribute.
    Defer,
    /// The classic `script` `nomodule` attribute.
    Nomodule,
    /// img controls content presence, independent of the alt author constraint.
    ImageControls,
    /// ismap content presence, independent of hyperlink activation behavior.
    Ismap,
    /// link disabled presence, not an associated stylesheet's runtime state.
    StylesheetDisabled,
}

impl HtmlBooleanAttribute {
    fn rule(self) -> RuleMetadata {
        // Defining clauses select presence semantics and element contexts;
        // contextual readonly also uses the common Boolean attribute grammar.
        let (attribute, rows, source, context): (_, &[u16], _, _) = match self {
            Self::Autofocus => (
                KnownHtmlAttribute::Autofocus,
                &[16],
                source!("attr-fe-autofocus"),
                Context::Global,
            ),
            Self::Inert => (
                KnownHtmlAttribute::Inert,
                &[75],
                source!("the-inert-attribute"),
                Context::Global,
            ),
            Self::Itemscope => (
                KnownHtmlAttribute::Itemscope,
                &[83],
                source!("attr-itemscope"),
                Context::Global,
            ),
            Self::HeadingReset => (
                KnownHtmlAttribute::Headingreset,
                &[63],
                source!("attr-headingreset"),
                Context::Global,
            ),
            Self::ShadowRootClonable => (
                KnownHtmlAttribute::Shadowrootclonable,
                &[136],
                source!("attr-template-shadowrootclonable"),
                Context::Template,
            ),
            Self::ShadowRootCustomElementRegistry => (
                KnownHtmlAttribute::Shadowrootcustomelementregistry,
                &[137],
                source!("attr-template-shadowrootcustomelementregistry"),
                Context::Template,
            ),
            Self::ShadowRootDelegatesFocus => (
                KnownHtmlAttribute::Shadowrootdelegatesfocus,
                &[138],
                source!("attr-template-shadowrootdelegatesfocus"),
                Context::Template,
            ),
            Self::ShadowRootSerializable => (
                KnownHtmlAttribute::Shadowrootserializable,
                &[140],
                source!("attr-template-shadowrootserializable"),
                Context::Template,
            ),
            Self::AllowFullscreen => (
                KnownHtmlAttribute::Allowfullscreen,
                &[7],
                source!("attr-iframe-allowfullscreen"),
                Context::Elements(&[KnownHtmlElement::Iframe]),
            ),
            Self::Autoplay => (
                KnownHtmlAttribute::Autoplay,
                &[17],
                source!("attr-media-autoplay"),
                Context::Elements(&[KnownHtmlElement::Audio, KnownHtmlElement::Video]),
            ),
            Self::MediaControls => (
                KnownHtmlAttribute::Controls,
                &[32],
                source!("attr-media-controls"),
                Context::Elements(&[KnownHtmlElement::Audio, KnownHtmlElement::Video]),
            ),
            Self::TrackDefault => (
                KnownHtmlAttribute::Default,
                &[39],
                source!("attr-track-default"),
                Context::Elements(&[KnownHtmlElement::Track]),
            ),
            Self::Loop => (
                KnownHtmlAttribute::Loop,
                &[90],
                source!("attr-media-loop"),
                Context::Elements(&[KnownHtmlElement::Audio, KnownHtmlElement::Video]),
            ),
            Self::Muted => (
                KnownHtmlAttribute::Muted,
                &[101],
                source!("attr-media-muted"),
                Context::Elements(&[KnownHtmlElement::Audio, KnownHtmlElement::Video]),
            ),
            Self::Novalidate => (
                KnownHtmlAttribute::Novalidate,
                &[111],
                source!("attr-fs-novalidate"),
                Context::Elements(&[KnownHtmlElement::Form]),
            ),
            Self::DetailsOpen => (
                KnownHtmlAttribute::Open,
                &[112],
                source!("attr-details-open"),
                Context::Elements(&[KnownHtmlElement::Details]),
            ),
            Self::DialogOpen => (
                KnownHtmlAttribute::Open,
                &[113],
                source!("attr-dialog-open"),
                Context::Elements(&[KnownHtmlElement::Dialog]),
            ),
            Self::PlaysInline => (
                KnownHtmlAttribute::Playsinline,
                &[118],
                source!("attr-video-playsinline"),
                Context::Elements(&[KnownHtmlElement::Video]),
            ),
            Self::Reversed => (
                KnownHtmlAttribute::Reversed,
                &[130],
                source!("attr-ol-reversed"),
                Context::Elements(&[KnownHtmlElement::Ol]),
            ),
            Self::Selected => (
                KnownHtmlAttribute::Selected,
                &[135],
                source!("attr-option-selected"),
                Context::Elements(&[KnownHtmlElement::Option]),
            ),
            Self::Disabled => (
                KnownHtmlAttribute::Disabled,
                &[44],
                source!("attr-fe-disabled"),
                Context::DisabledControls,
            ),
            Self::Alpha => (
                KnownHtmlAttribute::Alpha,
                &[8],
                source!("attr-input-alpha"),
                Context::InputControl(InputBooleanApplicability::Alpha),
            ),
            Self::Checked => (
                KnownHtmlAttribute::Checked,
                &[20],
                source!("attr-input-checked"),
                Context::InputControl(InputBooleanApplicability::Checked),
            ),
            Self::Multiple => (
                KnownHtmlAttribute::Multiple,
                &[100],
                source!("attr-input-multiple"),
                Context::InputControl(InputBooleanApplicability::Multiple),
            ),
            Self::Readonly => (
                KnownHtmlAttribute::Readonly,
                &[124],
                source!("attr-input-readonly"),
                Context::InputControl(InputBooleanApplicability::Readonly),
            ),
            Self::Required => (
                KnownHtmlAttribute::Required,
                &[129],
                source!("attr-input-required"),
                Context::InputControl(InputBooleanApplicability::Required),
            ),
            Self::Formnovalidate => (
                KnownHtmlAttribute::Formnovalidate,
                &[59],
                source!("attr-fs-formnovalidate"),
                Context::InputControl(InputBooleanApplicability::Formnovalidate),
            ),
            Self::Async => (
                KnownHtmlAttribute::Async,
                &[11],
                source!("attr-script-async"),
                Context::ScriptBoolean(ScriptApplicability::Async),
            ),
            Self::Defer => (
                KnownHtmlAttribute::Defer,
                &[40],
                source!("attr-script-defer"),
                Context::ScriptBoolean(ScriptApplicability::ExternalClassic),
            ),
            Self::Nomodule => (
                KnownHtmlAttribute::Nomodule,
                &[109],
                source!("attr-script-nomodule"),
                Context::ScriptBoolean(ScriptApplicability::Classic),
            ),
            Self::ImageControls => (
                KnownHtmlAttribute::Controls,
                &[32],
                source!("attr-img-controls"),
                Context::ImageControls,
            ),
            Self::Ismap => (
                KnownHtmlAttribute::Ismap,
                &[79],
                source!("attr-img-ismap"),
                Context::ImageMap,
            ),
            Self::StylesheetDisabled => (
                KnownHtmlAttribute::Disabled,
                &[46],
                source!("attr-link-disabled"),
                Context::StylesheetLink,
            ),
        };
        RuleMetadata {
            attribute,
            rows,
            source,
            context,
            vocabulary_source: None,
        }
    }

    fn rule_for(
        self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    ) -> RuleMetadata {
        let mut rule = self.rule();
        if self == Self::Disabled {
            match KnownHtmlElement::from_name(element) {
                Some(KnownHtmlElement::Fieldset) => {
                    rule.source = source!("attr-fieldset-disabled");
                    rule.rows = &[45];
                }
                Some(KnownHtmlElement::Optgroup) => rule.source = source!("attr-optgroup-disabled"),
                Some(KnownHtmlElement::Option) => rule.source = source!("attr-option-disabled"),
                _ => {}
            }
        }
        match (self, KnownHtmlElement::from_name(element)) {
            (Self::Multiple, Some(KnownHtmlElement::Select)) => {
                rule.source = source!("attr-select-multiple");
            }
            (Self::Readonly, Some(KnownHtmlElement::Textarea)) => {
                rule.source = source!("attr-textarea-readonly");
            }
            (Self::Readonly, None) if html_element_namespace(element) => {
                rule.source = source!("attr-face-readonly");
                rule.rows = &[125];
            }
            (Self::Required, Some(KnownHtmlElement::Select)) => {
                rule.source = source!("attr-select-required");
            }
            (Self::Required, Some(KnownHtmlElement::Textarea)) => {
                rule.source = source!("attr-textarea-required");
            }
            _ => {}
        }
        rule
    }

    /// Returns the standard expanded-name identity for this Boolean rule.
    pub fn attribute(self) -> KnownHtmlAttribute {
        self.rule().attribute
    }

    /// Constructs presence with conforming empty syntax, or omission for false.
    pub fn into_attribute(self, present: bool) -> Option<Attribute> {
        present.then(|| Attribute::new(self.attribute().qualified_name(), ""))
    }
}

fn html_element_namespace(
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> bool {
    crate::internal::values::is_html_namespace_uri(element.namespace_uri())
}

fn applies(
    rule: RuleMetadata,
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
) -> HtmlAttributeApplicability {
    // Compare only the selected expanded name; prefixes have no role here.
    // Avoid a vocabulary scan so specialized callers retain their work bounds.
    if raw.is_some_and(|a| {
        a.name().namespace_uri().is_some()
            || !a
                .name()
                .local_name()
                .code_units()
                .iter()
                .copied()
                .eq(rule.attribute.local_name().encode_utf16())
    }) {
        return HtmlAttributeApplicability::DifferentAttribute;
    }
    if !html_element_namespace(element) {
        return HtmlAttributeApplicability::ForeignElementNamespace;
    }
    // Global rules need no element-name lookup, including id and is reports.
    if matches!(rule.context, Context::Global | Context::Direction) {
        return HtmlAttributeApplicability::Applicable;
    }
    let known = KnownHtmlElement::from_name(element);
    let applicable = match rule.context {
        Context::Global | Context::Direction => true,
        Context::Template => known == Some(KnownHtmlElement::Template),
        Context::ImageMap | Context::ImageControls => known == Some(KnownHtmlElement::Img),
        Context::StylesheetLink => known == Some(KnownHtmlElement::Link),
        Context::DisabledControls => matches!(
            known,
            None | Some(
                KnownHtmlElement::Button
                    | KnownHtmlElement::Input
                    | KnownHtmlElement::Select
                    | KnownHtmlElement::Textarea
                    | KnownHtmlElement::Fieldset
                    | KnownHtmlElement::Optgroup
                    | KnownHtmlElement::Option
            )
        ),
        Context::Elements(elements) | Context::Fetching { elements, .. } => {
            known.is_some_and(|name| elements.contains(&name))
        }
        Context::InputControl(condition) => return condition.applicability(known),
        Context::SubmitterOverride | Context::PopoverTargetAction => {
            matches!(
                known,
                Some(KnownHtmlElement::Input | KnownHtmlElement::Button)
            )
        }
        Context::ScriptBoolean(_) => {
            return if known == Some(KnownHtmlElement::Script) {
                HtmlAttributeApplicability::Applicable
            } else {
                HtmlAttributeApplicability::DifferentElement
            };
        }
    };
    if !applicable {
        return HtmlAttributeApplicability::DifferentElement;
    }
    if rule.context == Context::DisabledControls && known.is_none() {
        return HtmlAttributeApplicability::CustomControlContextRequired;
    }
    if matches!(rule.context, Context::Fetching { .. }) && known == Some(KnownHtmlElement::Script) {
        return HtmlAttributeApplicability::ScriptContextRequired;
    }
    HtmlAttributeApplicability::Applicable
}

fn has_hyperlink_author_constraint(
    rule: RuleMetadata,
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> bool {
    matches!(
        rule.context,
        Context::Fetching {
            hyperlink: true,
            ..
        }
    ) && matches!(
        KnownHtmlElement::from_name(element),
        Some(KnownHtmlElement::A | KnownHtmlElement::Area)
    )
}

fn has_boolean_author_constraint(
    rule: RuleMetadata,
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> bool {
    match rule.context {
        Context::ImageMap | Context::ImageControls | Context::StylesheetLink => true,
        Context::InputControl(_) => matches!(
            KnownHtmlElement::from_name(element),
            Some(KnownHtmlElement::Input | KnownHtmlElement::Button)
        ),
        Context::ScriptBoolean(_) => {
            KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Script)
        }
        _ => false,
    }
}

// Seed exact observations and applicability without temporary owned names or
// value buffers. Value grammars, missing-value states and additional author
// context remain with the selected attribute owner.
fn report<T>(
    rule: RuleMetadata,
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
) -> HtmlAttributeReport<T> {
    let observation = match raw {
        None => HtmlAttributeObservation::Missing,
        Some(a) if a.value().is_empty() => HtmlAttributeObservation::PresentEmpty,
        Some(_) => HtmlAttributeObservation::PresentNonempty,
    };
    let applicability = applies(rule, element, raw);
    let context_author = if applicability != HtmlAttributeApplicability::Applicable {
        HtmlAttributeContextAuthorState::Unapplied
    } else if raw.is_some()
        && (has_boolean_author_constraint(rule, element)
            || has_hyperlink_author_constraint(rule, element)
            || rule.context == Context::SubmitterOverride
            || (rule.context == Context::PopoverTargetAction
                && KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Input)))
    {
        HtmlAttributeContextAuthorState::Unchecked
    } else {
        HtmlAttributeContextAuthorState::NotRequired
    };
    HtmlAttributeReport {
        element: capture_name(element),
        raw_attribute: raw.map(capture_attribute),
        rule,
        observation,
        applicability,
        recognition: HtmlAttributeRecognition::Unapplied,
        author: HtmlAttributeAuthorState::Unapplied,
        context_author,
        intrinsic: None,
        supplied_context: SuppliedContext::None,
    }
}

fn ascii_keyword(raw: &DocumentString, keyword: &str) -> bool {
    ascii_keyword_units(raw.code_units(), keyword)
}

fn ascii_keyword_units(units: &[u16], keyword: &str) -> bool {
    units.len() == keyword.len()
        && units
            .iter()
            .zip(keyword.bytes())
            .all(|(&unit, byte)| ascii_case_insensitive_unit(unit, byte))
}

struct EnumValueSelection<T> {
    recognition: HtmlAttributeRecognition,
    author: HtmlAttributeAuthorState,
    intrinsic: Option<T>,
}

fn enum_value_selection<T: Copy + 'static>(
    rule: &EnumRule<T>,
    raw: Option<&DocumentString>,
) -> EnumValueSelection<T> {
    match enum_value_selection_with(rule, raw, |_| Ok::<_, std::convert::Infallible>(())) {
        Ok(selection) => selection,
        Err(never) => match never {},
    }
}

fn enum_keyword_matches_with<T, E>(
    value: &DocumentString,
    keyword: &Keyword<T>,
    exact: bool,
    admit: &mut impl FnMut(bounded_rules::HtmlAttributeKeywordRead) -> Result<(), E>,
) -> Result<bool, E> {
    use bounded_rules::HtmlAttributeKeywordRead as Read;
    admit(Read::KeywordHeader)?;
    let spelling = keyword.spelling;
    if exact {
        let mut actual = value.code_units().iter();
        let mut expected = spelling.encode_utf16();
        loop {
            admit(Read::KeywordPair)?;
            match (actual.next(), expected.next()) {
                (None, None) => return Ok(true),
                (Some(&actual), Some(expected)) if actual == expected => {}
                _ => return Ok(false),
            }
        }
    }
    let units = value.code_units();
    let bytes = spelling.as_bytes();
    if units.len() != bytes.len() {
        return Ok(false);
    }
    for offset in 0..units.len() {
        admit(Read::KeywordPair)?;
        if !ascii_case_insensitive_unit(units[offset], bytes[offset]) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// How a raw value matched an enumerated attribute's keywords, before any default is chosen.
enum EnumRecognition<'rule, T> {
    Missing,
    Keyword(&'rule Keyword<T>),
    Empty,
    Unrecognized,
}

/// Admits the value header, then each keyword's header and pairs until one matches.
fn enum_recognition_with<'rule, T, E>(
    rule: &'rule EnumRule<T>,
    raw: Option<&DocumentString>,
    mut admit: impl FnMut(bounded_rules::HtmlAttributeKeywordRead) -> Result<(), E>,
) -> Result<EnumRecognition<'rule, T>, E> {
    admit(bounded_rules::HtmlAttributeKeywordRead::ValueHeader)?;
    let Some(value) = raw else {
        return Ok(EnumRecognition::Missing);
    };
    for keyword in rule.keywords {
        if enum_keyword_matches_with(value, keyword, rule.exact_keywords, &mut admit)? {
            return Ok(EnumRecognition::Keyword(keyword));
        }
    }
    Ok(if value.is_empty() {
        EnumRecognition::Empty
    } else {
        EnumRecognition::Unrecognized
    })
}

fn enum_value_selection_with<T: Copy + 'static, E>(
    rule: &EnumRule<T>,
    raw: Option<&DocumentString>,
    admit: impl FnMut(bounded_rules::HtmlAttributeKeywordRead) -> Result<(), E>,
) -> Result<EnumValueSelection<T>, E> {
    let mut selection = EnumValueSelection {
        recognition: HtmlAttributeRecognition::Unapplied,
        author: HtmlAttributeAuthorState::Unapplied,
        intrinsic: None,
    };
    match enum_recognition_with(rule, raw, admit)? {
        EnumRecognition::Missing => {
            selection.recognition = HtmlAttributeRecognition::Missing;
            selection.author = HtmlAttributeAuthorState::Missing;
            selection.intrinsic = rule.missing;
        }
        EnumRecognition::Keyword(keyword) => {
            selection.recognition = HtmlAttributeRecognition::Keyword {
                matched: keyword.spelling,
                canonical: keyword.canonical,
            };
            selection.author = if keyword.conforming {
                HtmlAttributeAuthorState::Conforming
            } else {
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::NonconformingKeyword,
                )
            };
            selection.intrinsic = Some(keyword.state);
        }
        EnumRecognition::Empty => {
            selection.recognition = HtmlAttributeRecognition::Empty;
            selection.intrinsic = rule.empty.or(rule.invalid);
            selection.author = if rule.empty.is_some() {
                HtmlAttributeAuthorState::Conforming
            } else {
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::UnrecognizedKeyword,
                )
            };
        }
        EnumRecognition::Unrecognized => {
            selection.recognition = HtmlAttributeRecognition::Unrecognized;
            selection.author = HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword,
            );
            selection.intrinsic = rule.invalid;
        }
    }
    Ok(selection)
}

/// The value a rule with defined defaults selects, by the same recognition as
/// [`enum_value_selection_with`]. Every outcome has a value, so none is optional.
fn total_enum_value_with<T: Copy + 'static, E>(
    rule: &EnumRule<T>,
    defaults: &TotalEnumDefaults<T>,
    raw: Option<&DocumentString>,
    admit: impl FnMut(bounded_rules::HtmlAttributeKeywordRead) -> Result<(), E>,
) -> Result<T, E> {
    Ok(match enum_recognition_with(rule, raw, admit)? {
        EnumRecognition::Missing => defaults.missing,
        EnumRecognition::Keyword(keyword) => keyword.state,
        EnumRecognition::Empty => defaults.empty.unwrap_or(defaults.invalid),
        EnumRecognition::Unrecognized => defaults.invalid,
    })
}

impl HtmlValueInterpreter {
    /// Content-state processing for the HTML-defined generic parent algorithm.
    /// This makes no element applicability or author-conformance assertion.
    pub fn writingsuggestions_processing(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlWritingSuggestionsProcessing, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw)?;
        let selection = enum_value_selection(&WRITINGSUGGESTIONS, raw);
        Ok(HtmlWritingSuggestionsProcessing {
            recognition: selection.recognition,
            intrinsic: selection.intrinsic,
        })
    }

    fn enumerated<T: Copy>(
        &self,
        rule: &EnumRule<T>,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<T>, HtmlValueAdmissionError> {
        self.enumerated_with_context(rule, element, raw, SuppliedContext::None)
    }

    fn enumerated_with_context<T: Copy>(
        &self,
        rule: &EnumRule<T>,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        supplied_context: SuppliedContext,
    ) -> Result<HtmlAttributeReport<T>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut result = report(rule.metadata, element, raw);
        result.supplied_context = supplied_context.clone();
        match supplied_context {
            SuppliedContext::None => {}
            SuppliedContext::Script(context) => {
                if result.applicability == HtmlAttributeApplicability::ScriptContextRequired {
                    if let Context::Fetching { script, .. } = rule.metadata.context {
                        result.applicability = if script.allows(context) {
                            HtmlAttributeApplicability::Applicable
                        } else {
                            HtmlAttributeApplicability::InapplicableScriptKind
                        };
                    }
                } else if result.applicability == HtmlAttributeApplicability::Applicable {
                    result.applicability = HtmlAttributeApplicability::DifferentElement;
                }
            }
            SuppliedContext::Hyperlink(_) => {
                if !has_hyperlink_author_constraint(rule.metadata, element)
                    && matches!(
                        result.applicability,
                        HtmlAttributeApplicability::Applicable
                            | HtmlAttributeApplicability::ScriptContextRequired
                    )
                {
                    result.applicability = HtmlAttributeApplicability::DifferentElement;
                }
            }
            SuppliedContext::CustomControl(_) => {
                result.applicability = HtmlAttributeApplicability::DifferentElement;
            }
            SuppliedContext::Input(_) | SuppliedContext::ButtonSubmitter(_) => {
                let matches_element = matches!(
                    (&supplied_context, KnownHtmlElement::from_name(element)),
                    (SuppliedContext::Input(_), Some(KnownHtmlElement::Input))
                        | (
                            SuppliedContext::ButtonSubmitter(_),
                            Some(KnownHtmlElement::Button)
                        )
                );
                if result.applicability == HtmlAttributeApplicability::Applicable
                    && (!matches_element
                        || !(rule.metadata.context == Context::SubmitterOverride
                            || (rule.metadata.context == Context::PopoverTargetAction
                                && matches!(supplied_context, SuppliedContext::Input(_)))))
                {
                    result.applicability = HtmlAttributeApplicability::DifferentContext;
                }
            }
            SuppliedContext::ImageMap(_)
            | SuppliedContext::ImageControls(_)
            | SuppliedContext::StylesheetLink(_)
            | SuppliedContext::TextareaCols(_) => {
                result.applicability = HtmlAttributeApplicability::DifferentContext;
            }
        }
        if result.applicability != HtmlAttributeApplicability::Applicable {
            result.context_author = HtmlAttributeContextAuthorState::Unapplied;
            return Ok(result);
        }
        // The href requirement constrains authorship only. Without supplied
        // evidence, intrinsic interpretation retains an unchecked author fact.
        if !has_hyperlink_author_constraint(rule.metadata, element)
            && rule.metadata.context != Context::SubmitterOverride
            && !(rule.metadata.context == Context::PopoverTargetAction
                && KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Input))
        {
            result.context_author = HtmlAttributeContextAuthorState::NotRequired;
        }
        if raw.is_some()
            && let SuppliedContext::Hyperlink(context) = &result.supplied_context
        {
            result.context_author = match context {
                HtmlHyperlinkAttributeContext::WithHref => {
                    HtmlAttributeContextAuthorState::Conforming
                }
                HtmlHyperlinkAttributeContext::WithoutHref => {
                    HtmlAttributeContextAuthorState::Nonconforming(
                        HtmlAttributeContextConformanceError::ReferrerPolicyRequiresHref,
                    )
                }
            };
        }
        if raw.is_some() && rule.metadata.context == Context::SubmitterOverride {
            result.context_author = match &result.supplied_context {
                SuppliedContext::Input(HtmlInputType::Submit | HtmlInputType::Image)
                | SuppliedContext::ButtonSubmitter(HtmlButtonSubmitterContext::SubmitButton) => {
                    HtmlAttributeContextAuthorState::Conforming
                }
                SuppliedContext::Input(_) => HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                ),
                SuppliedContext::ButtonSubmitter(_) => {
                    HtmlAttributeContextAuthorState::Nonconforming(
                        HtmlAttributeContextConformanceError::FormOverrideRequiresSubmitButton,
                    )
                }
                _ => HtmlAttributeContextAuthorState::Unchecked,
            };
        }
        if raw.is_some() && rule.metadata.context == Context::PopoverTargetAction {
            result.context_author = match &result.supplied_context {
                SuppliedContext::Input(
                    HtmlInputType::Submit
                    | HtmlInputType::Image
                    | HtmlInputType::Reset
                    | HtmlInputType::Button,
                ) => HtmlAttributeContextAuthorState::Conforming,
                SuppliedContext::Input(_) => HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                ),
                _ => result.context_author,
            };
        }
        let selection = enum_value_selection(rule, raw.map(HtmlAttributeInput::value));
        result.recognition = selection.recognition;
        result.author = selection.author;
        result.intrinsic = selection.intrinsic;
        Ok(result)
    }

    /// Interprets content Boolean presence; this does not compute its effects.
    pub fn boolean_attribute(
        &self,
        selected: HtmlBooleanAttribute,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(selected, element, raw, SuppliedContext::None)
    }

    fn boolean_with_context(
        &self,
        selected: HtmlBooleanAttribute,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        supplied_context: SuppliedContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        if let SuppliedContext::StylesheetLink(context) = &supplied_context {
            self.admit_attribute_raw(context.rel())?;
        }
        let mut result = report(selected.rule_for(element), element, raw);
        result.supplied_context = supplied_context.clone();
        if let SuppliedContext::CustomControl(context) = &supplied_context {
            if result.applicability == HtmlAttributeApplicability::CustomControlContextRequired {
                result.applicability = match context {
                    HtmlCustomControlAttributeContext::FormAssociated => {
                        HtmlAttributeApplicability::Applicable
                    }
                    HtmlCustomControlAttributeContext::NotFormAssociated => {
                        HtmlAttributeApplicability::InapplicableCustomControl
                    }
                };
            } else if matches!(
                result.applicability,
                HtmlAttributeApplicability::Applicable
                    | HtmlAttributeApplicability::ScriptContextRequired
            ) {
                result.applicability = HtmlAttributeApplicability::DifferentElement;
            }
        }
        match supplied_context {
            SuppliedContext::TextareaCols(_) => {
                if matches!(
                    result.applicability,
                    HtmlAttributeApplicability::Applicable
                        | HtmlAttributeApplicability::ScriptContextRequired
                        | HtmlAttributeApplicability::CustomControlContextRequired
                ) {
                    result.applicability = HtmlAttributeApplicability::DifferentContext;
                }
            }
            SuppliedContext::Input(_) => {
                if !matches!(
                    (result.rule.context, KnownHtmlElement::from_name(element)),
                    (Context::InputControl(_), Some(KnownHtmlElement::Input))
                ) && matches!(
                    result.applicability,
                    HtmlAttributeApplicability::Applicable
                        | HtmlAttributeApplicability::ScriptContextRequired
                        | HtmlAttributeApplicability::CustomControlContextRequired
                ) {
                    result.applicability = HtmlAttributeApplicability::DifferentContext;
                }
            }
            SuppliedContext::Script(_) => {
                if !matches!(
                    (result.rule.context, KnownHtmlElement::from_name(element)),
                    (Context::ScriptBoolean(_), Some(KnownHtmlElement::Script))
                ) && matches!(
                    result.applicability,
                    HtmlAttributeApplicability::Applicable
                        | HtmlAttributeApplicability::ScriptContextRequired
                        | HtmlAttributeApplicability::CustomControlContextRequired
                ) {
                    result.applicability = HtmlAttributeApplicability::DifferentContext;
                }
            }
            SuppliedContext::ButtonSubmitter(_)
                if !matches!(
                    (result.rule.context, KnownHtmlElement::from_name(element)),
                    (
                        Context::InputControl(InputBooleanApplicability::Formnovalidate),
                        Some(KnownHtmlElement::Button)
                    )
                ) && matches!(
                    result.applicability,
                    HtmlAttributeApplicability::Applicable
                        | HtmlAttributeApplicability::ScriptContextRequired
                        | HtmlAttributeApplicability::CustomControlContextRequired
                ) =>
            {
                result.applicability = HtmlAttributeApplicability::DifferentContext;
            }
            _ => {}
        }
        if result.applicability != HtmlAttributeApplicability::Applicable {
            result.context_author = HtmlAttributeContextAuthorState::Unapplied;
            return Ok(result);
        }
        if !has_boolean_author_constraint(result.rule, element) {
            result.context_author = HtmlAttributeContextAuthorState::NotRequired;
        }
        result.intrinsic = Some(raw.is_some());
        match raw {
            None => {
                result.recognition = HtmlAttributeRecognition::Missing;
                result.author = HtmlAttributeAuthorState::Missing;
            }
            Some(a) if a.value().is_empty() => {
                result.recognition = HtmlAttributeRecognition::Empty;
                result.author = HtmlAttributeAuthorState::Conforming;
            }
            Some(a) if ascii_keyword(a.value(), selected.attribute().local_name()) => {
                result.recognition = HtmlAttributeRecognition::BooleanName;
                result.author = HtmlAttributeAuthorState::Conforming;
            }
            Some(_) => {
                result.recognition = HtmlAttributeRecognition::Unrecognized;
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::BooleanSpelling,
                );
            }
        }
        if raw.is_some() {
            result.context_author = match &result.supplied_context {
                SuppliedContext::Input(state) => {
                    if let Context::InputControl(condition) = result.rule.context {
                        if condition.allows(*state) {
                            HtmlAttributeContextAuthorState::Conforming
                        } else {
                            HtmlAttributeContextAuthorState::Nonconforming(
                                HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                            )
                        }
                    } else {
                        result.context_author
                    }
                }
                SuppliedContext::Script(context) => {
                    if let Context::ScriptBoolean(condition) = result.rule.context {
                        if condition.allows(*context) {
                            HtmlAttributeContextAuthorState::Conforming
                        } else {
                            HtmlAttributeContextAuthorState::Nonconforming(
                                HtmlAttributeContextConformanceError::ScriptContextForbidsAttribute,
                            )
                        }
                    } else {
                        result.context_author
                    }
                }
                SuppliedContext::ButtonSubmitter(context) => match context {
                    HtmlButtonSubmitterContext::SubmitButton => HtmlAttributeContextAuthorState::Conforming,
                    HtmlButtonSubmitterContext::NotSubmitButton => HtmlAttributeContextAuthorState::Nonconforming(
                        HtmlAttributeContextConformanceError::FormnovalidateRequiresSubmitButton,
                    ),
                },
                SuppliedContext::ImageMap(context) if result.rule.context == Context::ImageMap => {
                    match context {
                        HtmlImageMapAttributeContext::AncestorAnchorWithHref => {
                            HtmlAttributeContextAuthorState::Conforming
                        }
                        HtmlImageMapAttributeContext::NoAncestorAnchorWithHref => {
                            HtmlAttributeContextAuthorState::Nonconforming(
                                HtmlAttributeContextConformanceError::ImageMapRequiresAncestorAnchorWithHref,
                            )
                        }
                        HtmlImageMapAttributeContext::Unavailable => HtmlAttributeContextAuthorState::Unavailable,
                    }
                }
                SuppliedContext::ImageControls(context) if result.rule.context == Context::ImageControls => {
                    match context {
                        HtmlImageControlsAttributeContext::AltNonempty => HtmlAttributeContextAuthorState::Conforming,
                        HtmlImageControlsAttributeContext::AltMissing | HtmlImageControlsAttributeContext::AltEmpty => {
                            HtmlAttributeContextAuthorState::Nonconforming(
                                HtmlAttributeContextConformanceError::ImageControlsRequireNonemptyAlt,
                            )
                        }
                        HtmlImageControlsAttributeContext::Unavailable => HtmlAttributeContextAuthorState::Unavailable,
                    }
                }
                SuppliedContext::StylesheetLink(context) if result.rule.context == Context::StylesheetLink => {
                    if context.contains_stylesheet() {
                        HtmlAttributeContextAuthorState::Conforming
                    } else {
                        HtmlAttributeContextAuthorState::Nonconforming(
                            HtmlAttributeContextConformanceError::StylesheetDisabledRequiresStylesheetRel,
                        )
                    }
                }
                _ => result.context_author,
            };
        }
        Ok(result)
    }

    /// Interprets ismap presence while observing its actual ancestor author fact.
    /// URL resolution, activation and server-side map behavior remain external.
    pub fn ismap_with_context(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlImageMapAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(
            HtmlBooleanAttribute::Ismap,
            element,
            raw,
            SuppliedContext::ImageMap(context),
        )
    }

    /// Interprets img controls presence independently of the actual alt fact.
    /// A host may expose UI; this operation does not create or suppress controls.
    pub fn image_controls_with_context(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlImageControlsAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(
            HtmlBooleanAttribute::ImageControls,
            element,
            raw,
            SuppliedContext::ImageControls(context),
        )
    }

    /// Admits an actual rel value and derives only stylesheet token membership.
    /// ASCII whitespace and case apply; no full rel vocabulary is certified.
    pub fn stylesheet_link_context(
        &self,
        rel: Option<&DocumentString>,
    ) -> Result<HtmlStylesheetLinkAttributeContext, HtmlValueAdmissionError> {
        self.admit_attribute_raw(rel)?;
        let contains_stylesheet = rel.is_some_and(|raw| {
            crate::internal::elements::rel::contains(
                raw,
                crate::internal::elements::rel::HtmlRelKeyword::Stylesheet,
            )
        });
        Ok(HtmlStylesheetLinkAttributeContext {
            rel: rel.cloned(),
            contains_stylesheet,
        })
    }

    /// Interprets link disabled content presence and its rel author constraint.
    /// The supplied context is re-admitted under this interpreter's own cap.
    pub fn stylesheet_disabled(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: &HtmlStylesheetLinkAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(
            HtmlBooleanAttribute::StylesheetDisabled,
            element,
            raw,
            SuppliedContext::StylesheetLink(context.clone()),
        )
    }

    /// Interprets Boolean content and its actual input-state author constraint.
    /// The state can come from input_type interpretation; it is never guessed.
    /// Ignored effects, current checkedness and other live control state are separate.
    pub fn boolean_attribute_for_input(
        &self,
        selected: HtmlBooleanAttribute,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(selected, element, raw, SuppliedContext::Input(state))
    }

    /// Interprets script Boolean content presence with actual kind/src facts.
    /// Scheduling, force-async, module support and execution remain external.
    pub fn boolean_attribute_for_script(
        &self,
        selected: HtmlBooleanAttribute,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlScriptAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(selected, element, raw, SuppliedContext::Script(context))
    }

    /// Interprets formnovalidate presence on an actually classified button.
    /// This does not compute form ownership or the effective no-validate state.
    pub fn formnovalidate_for_button(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlButtonSubmitterContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(
            HtmlBooleanAttribute::Formnovalidate,
            element,
            raw,
            SuppliedContext::ButtonSubmitter(context),
        )
    }

    /// Interprets readonly for an explicitly classified autonomous custom control.
    /// Presence does not describe current editability or validation results.
    pub fn readonly_for_custom_control(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlCustomControlAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(
            HtmlBooleanAttribute::Readonly,
            element,
            raw,
            SuppliedContext::CustomControl(context),
        )
    }

    /// Interprets local disabled content presence using its defining element clause.
    /// Ancestors, legend exceptions and actual disabledness are separate queries.
    pub fn disabled(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_attribute(HtmlBooleanAttribute::Disabled, element, raw)
    }

    /// Interprets disabled for an explicitly classified autonomous custom control.
    pub fn disabled_for_custom_control(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlCustomControlAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(
            HtmlBooleanAttribute::Disabled,
            element,
            raw,
            SuppliedContext::CustomControl(context),
        )
    }

    /// Interprets dir's intrinsic state and the bdo-specific author requirement.
    /// Computed directionality and auto-directionality need separate context.
    pub fn dir(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlDirState>, HtmlValueAdmissionError> {
        let mut result = self.enumerated(&DIR, element, raw)?;
        if result.applicability == HtmlAttributeApplicability::Applicable
            && KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Bdo)
        {
            if raw.is_none() {
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::BdoDirectionRequired,
                );
            } else if !matches!(
                result.intrinsic,
                Some(HtmlDirState::Keyword(HtmlDir::Ltr | HtmlDir::Rtl))
            ) {
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::BdoDirectionMustBeLtrOrRtl,
                );
            }
        }
        Ok(result)
    }
}

macro_rules! interpret_method {
    ($method:ident, $rule:ident, $type:ty) => {
        impl HtmlValueInterpreter {
            #[doc = concat!("Interprets the intrinsic ", stringify!($method), " attribute state; no contextual or runtime state is computed.")]
            pub fn $method(&self, element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized), raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated(&$rule, element, raw)
            }
        }
    };
}

interpret_method!(contenteditable, CONTENTEDITABLE, HtmlContentEditableState);
interpret_method!(draggable, DRAGGABLE, HtmlDraggableState);
interpret_method!(hidden, HIDDEN, HtmlHiddenState);
interpret_method!(popover, POPOVER, HtmlPopoverState);
interpret_method!(translate, TRANSLATE, HtmlTranslateState);
interpret_method!(spellcheck, SPELLCHECK, HtmlSpellcheckState);
interpret_method!(autocapitalize, AUTOCAPITALIZE, HtmlAutocapitalizeState);
interpret_method!(autocorrect, AUTOCORRECT, HtmlAutocorrect);
interpret_method!(
    writingsuggestions,
    WRITINGSUGGESTIONS,
    HtmlWritingSuggestionsState
);
interpret_method!(inputmode, INPUTMODE, HtmlInputMode);
interpret_method!(enterkeyhint, ENTERKEYHINT, HtmlEnterKeyHint);
interpret_method!(shadowrootmode, SHADOWROOTMODE, HtmlShadowRootMode);
interpret_method!(
    shadowrootslotassignment,
    SHADOWROOTSLOTASSIGNMENT,
    HtmlSlotAssignment
);
interpret_method!(closedby, CLOSEDBY, HtmlClosedByState);
interpret_method!(form_autocomplete, FORM_AUTOCOMPLETE, HtmlFormAutocomplete);
interpret_method!(decoding, DECODING, HtmlDecoding);
interpret_method!(loading, LOADING, HtmlLoading);
interpret_method!(preload, PRELOAD, HtmlPreloadState);
interpret_method!(form_method, FORM_METHOD, HtmlFormMethod);
interpret_method!(form_enctype, FORM_ENCTYPE, HtmlFormEnctype);
interpret_method!(track_kind, TRACK_KIND, HtmlTrackKind);
interpret_method!(ol_type, OL_TYPE, HtmlOlType);
interpret_method!(input_type, INPUT_TYPE, HtmlInputType);
interpret_method!(button_type, BUTTON_TYPE, HtmlButtonTypeState);
interpret_method!(crossorigin, CROSSORIGIN, HtmlCrossOriginState);
interpret_method!(fetchpriority, FETCHPRIORITY, HtmlFetchPriority);
interpret_method!(referrerpolicy, REFERRERPOLICY, HtmlReferrerPolicy);

macro_rules! script_interpret_method {
    ($method:ident, $rule:ident, $type:ty) => {
        impl HtmlValueInterpreter {
            /// Interprets an intrinsic keyword attribute with explicitly supplied script facts.
            pub fn $method(
                &self,
                element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
                raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
                context: HtmlScriptAttributeContext,
            ) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
                self.enumerated_with_context(&$rule, element, raw, SuppliedContext::Script(context))
            }
        }
    };
}

script_interpret_method!(crossorigin_for_script, CROSSORIGIN, HtmlCrossOriginState);
script_interpret_method!(fetchpriority_for_script, FETCHPRIORITY, HtmlFetchPriority);
script_interpret_method!(
    referrerpolicy_for_script,
    REFERRERPOLICY,
    HtmlReferrerPolicy
);

impl HtmlValueInterpreter {
    /// Interprets referrerpolicy for a/area over explicitly supplied href presence.
    /// The href value and effective request policy are not resolved.
    pub fn referrerpolicy_for_hyperlink(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlHyperlinkAttributeContext,
    ) -> Result<HtmlAttributeReport<HtmlReferrerPolicy>, HtmlValueAdmissionError> {
        self.enumerated_with_context(
            &REFERRERPOLICY,
            element,
            raw,
            SuppliedContext::Hyperlink(context),
        )
    }
}

// Local processing has a deliberate bounded front door, separate from the
// ordinary attribute value/report reexports.
pub(crate) mod bounded_rules {
    use super::*;

    /// Admission before one intrinsic enumeration observation. No name,
    /// attribute applicability or captured-tree evidence is examined.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HtmlAttributeKeywordRead {
        /// Raw presence/length, missing default and fixed selection headers.
        /// This occurs even when the raw attribute is missing.
        ValueHeader,
        /// One keyword's spelling/length and the raw comparison headers.
        KeywordHeader,
        /// One raw UTF-16 unit and canonical ASCII byte comparison. The shared
        /// exact-keyword path instead compares UTF-16 units and admits its
        /// optional next-unit/end observation through this same action.
        KeywordPair,
    }

    /// CORS settings content-state processing for an already qualified caller.
    /// Missing, empty and invalid defaults use the same rule as the ordinary
    /// interpreter. This grants no element applicability or resource authority.
    /// Each header/pair is admitted before its operand read; refusal stops
    /// immediately, without allocation or mutation.
    pub fn crossorigin_processing_with<E>(
        raw: Option<&DocumentString>,
        admit: impl FnMut(HtmlAttributeKeywordRead) -> Result<(), E>,
    ) -> Result<HtmlCrossOriginState, E> {
        total_enum_value_with(&CROSSORIGIN, &CROSSORIGIN_DEFAULTS, raw, admit)
    }

    /// Referrer policy content-state processing, independently of element,
    /// hyperlink or script applicability. The empty policy remains unresolved;
    /// this does not choose a Document policy or parse an HTTP header.
    /// Uses the shared vocabulary/defaults and admits each header/pair before
    /// its operand read. No name or attribute record is constructed or scanned.
    pub fn referrerpolicy_processing_with<E>(
        raw: Option<&DocumentString>,
        admit: impl FnMut(HtmlAttributeKeywordRead) -> Result<(), E>,
    ) -> Result<HtmlReferrerPolicy, E> {
        total_enum_value_with(&REFERRERPOLICY, &REFERRERPOLICY_DEFAULTS, raw, admit)
    }

    /// Shared source applicability for the captured candidacy consumer.
    /// Content Boolean presence and author-context reports remain separate.
    pub fn input_readonly_applies(state: HtmlInputType) -> bool {
        InputBooleanApplicability::Readonly.allows(state)
    }

    /// Whether the `list` attribute applies to an `input` of this type state.
    ///
    /// `list` applies to the Text, Search, Telephone, URL, Email, Date, Month, Week, Time, Local
    /// Date and Time, Number, Range and Color states (HTML877d `#input-type-attr-summary`, the
    /// `list` row). When it does not apply there is no suggestions source element (HTML877d
    /// `#the-list-attribute`, "If the list attribute does not apply, there is no suggestions
    /// source element"). Retained capture:
    /// `references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`, the
    /// `input-type-attr-summary` table and the `list` attribute prose. This is applicability
    /// alone; it does not resolve the attribute's ID reference.
    pub fn input_list_applies(state: HtmlInputType) -> bool {
        use HtmlInputType::{
            Color, Date, Email, LocalDateTime, Month, Number, Range, Search, Telephone, Text, Time,
            Url, Week,
        };
        matches!(
            state,
            Text | Search
                | Telephone
                | Url
                | Email
                | Date
                | Month
                | Week
                | Time
                | LocalDateTime
                | Number
                | Range
                | Color
        )
    }

    /// Whether an `input` of this type state is a submit button.
    ///
    /// The Submit Button state and the Image Button state both describe an `input` element that
    /// "is a button, specifically a submit button" (HTML877d `#submit-button-state-(type=submit)`
    /// and `#image-button-state-(type=image)`). Every other state is not a submit button. Retained
    /// capture: `references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`,
    /// the opening prose of those two states. This classifies the state alone, not the form owner
    /// or any activation.
    pub fn input_submitter_context(state: HtmlInputType) -> HtmlButtonSubmitterContext {
        match state {
            HtmlInputType::Submit | HtmlInputType::Image => {
                HtmlButtonSubmitterContext::SubmitButton
            }
            _ => HtmlButtonSubmitterContext::NotSubmitButton,
        }
    }

    /// A fact about an actual `button` element that its Auto type state consults.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HtmlButtonSubmitterFact {
        /// The element has a `command` content attribute, whatever its value.
        CommandAttributePresent,
        /// The element has a `commandfor` content attribute, whatever its value.
        CommandforAttributePresent,
        /// The element's parent node is a `select` element. A parent that is absent or is not an
        /// element is not a `select`, so the fact is false there.
        ParentIsSelect,
    }

    /// Whether a `button` element is a submit button, from its type state and the facts its Auto
    /// state consults.
    ///
    /// A `button` is a submit button when its `type` attribute is in the Submit Button state, or
    /// when it is in the Auto state, neither `command` nor `commandfor` is present and the parent
    /// node is not a `select` element (HTML877d `#the-button-element`, "is said to be a submit
    /// button"; `#attr-button-type` for the Auto state). Retained capture:
    /// `references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md`, the list
    /// after "is said to be a submit button". The Reset Button and Button states are never
    /// submit buttons.
    ///
    /// `facts` is asked lazily, in the standard's order, and only in the Auto state: the
    /// `command` attribute first, then `commandfor` only when `command` is absent, then the parent
    /// only when both are absent. Its first error ends the classification and is returned
    /// unchanged, so a caller's charges, cancellation and first reached failure are exactly those
    /// of the facts it was asked for.
    pub fn button_submitter_context_with<E>(
        state: HtmlButtonTypeState,
        mut facts: impl FnMut(HtmlButtonSubmitterFact) -> Result<bool, E>,
    ) -> Result<HtmlButtonSubmitterContext, E> {
        let submit = match state {
            HtmlButtonTypeState::Keyword(HtmlButtonType::Submit) => true,
            HtmlButtonTypeState::Keyword(HtmlButtonType::Reset | HtmlButtonType::Button) => false,
            HtmlButtonTypeState::Auto => {
                !facts(HtmlButtonSubmitterFact::CommandAttributePresent)?
                    && !facts(HtmlButtonSubmitterFact::CommandforAttributePresent)?
                    && !facts(HtmlButtonSubmitterFact::ParentIsSelect)?
            }
        };
        Ok(if submit {
            HtmlButtonSubmitterContext::SubmitButton
        } else {
            HtmlButtonSubmitterContext::NotSubmitButton
        })
    }

    /// Finite work envelope for stylesheet membership and fixed context creation.
    ///
    /// For n input units, the borrowed Infra splitter inspects at most n boundary
    /// units and visits at most n + 1 fields. At most n nonempty tokens reach the
    /// membership search, with two length reads each. Compared tokens are disjoint:
    /// at most n unit/keyword-byte pairs are read and ASCII-compared. These account
    /// for n + (n + 1) + n + 2n + 2n + n = 8n + 1 actions. The additional twelve
    /// cover raw presence/length/cap admission, borrowed slice access, search result,
    /// shared-handle retention, and fixed result construction. Early matches only
    /// reduce work. No token or code-unit buffer is materialized by this operation.
    pub fn stylesheet_link_context_work_bound(input_code_units: usize) -> Option<usize> {
        input_code_units.checked_mul(8)?.checked_add(13)
    }

    /// The recognition and intrinsic state of one `writingsuggestions` value, without a name or
    /// attribute record.
    ///
    /// The bounded front door returns it after admitting the value under the caller's work
    /// accounting.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct HtmlWritingSuggestionsProcessing {
        pub(crate) recognition: HtmlAttributeRecognition,
        pub(crate) intrinsic: Option<HtmlWritingSuggestionsState>,
    }
    impl HtmlWritingSuggestionsProcessing {
        /// Returns how the literal value matched the `true` and `false` keywords, before defaults.
        pub const fn recognition(&self) -> HtmlAttributeRecognition {
            self.recognition
        }
        /// Returns the selected intrinsic state, which is never absent for this attribute because
        /// every value has a default.
        pub const fn intrinsic(&self) -> Option<HtmlWritingSuggestionsState> {
            self.intrinsic
        }
    }
}
pub(crate) use bounded_rules::HtmlWritingSuggestionsProcessing;
#[cfg(test)]
use bounded_rules::stylesheet_link_context_work_bound;

pub(crate) mod accept;
pub(crate) mod area;
pub(crate) mod autocomplete;
pub(crate) mod command;
pub(crate) mod control_length;
pub(crate) mod control_text;
pub(crate) mod custom_name;
pub(crate) mod details_name;
pub(crate) mod encoding_label;
pub(crate) mod form_name;
pub(crate) mod hash_name;
pub(crate) mod id;
pub(crate) mod image_values;
pub(crate) mod input_bounds;
pub(crate) mod input_colorspace;
pub(crate) mod link_destination;
pub(crate) mod map_name;
pub(crate) mod meta_pragma;
pub(crate) mod meta_processing;
pub(crate) mod meter;
pub(crate) mod microdata;
pub(crate) mod modification;
pub(crate) mod navigable_target;
pub(crate) mod progress;
pub(crate) mod sandbox;
pub(crate) mod scalar;
pub(crate) mod srcset;
pub(crate) mod table_header_scope;
pub(crate) mod textarea_wrap;
pub(crate) mod token_declarations;
#[cfg(test)]
mod stylesheet_context_tests {
    use super::*;

    #[test]
    fn stylesheet_membership_keeps_exact_shared_rel_and_ascii_only_boundaries() {
        let interpreter = HtmlValueInterpreter::new(100);
        for (units, expected) in [
            (vec![], false),
            (" \t\n\r\u{c}".encode_utf16().collect(), false),
            (
                "alternate\tStYlEsHeEt\nstylesheet\r\u{c}preload "
                    .encode_utf16()
                    .collect(),
                true,
            ),
            ("stylesheet\u{a0}alternate".encode_utf16().collect(), false),
            ("stylesheet,preload".encode_utf16().collect(), false),
            ("xstylesheet stylesheetx".encode_utf16().collect(), false),
            (vec![0xD800, 0x20, 0xDC00, 0x20, 0xD83D, 0xDE00], false),
            (
                [vec![0xD800, 0x20], "STYLESHEET".encode_utf16().collect()].concat(),
                true,
            ),
        ] {
            let raw = DocumentString::from_utf16(units.clone());
            let context = interpreter.stylesheet_link_context(Some(&raw)).unwrap();
            assert_eq!(context.contains_stylesheet(), expected);
            assert_eq!(context.rel().unwrap().code_units(), units);
            assert_eq!(
                context.rel().unwrap().code_units().as_ptr(),
                raw.code_units().as_ptr()
            );
        }
        let missing = interpreter.stylesheet_link_context(None).unwrap();
        assert_eq!(missing.rel(), None);
        assert!(!missing.contains_stylesheet());
    }

    #[test]
    fn borrowed_keyword_comparison_has_the_same_exact_ascii_language() {
        for (text, expected) in [
            ("stylesheet", true),
            ("STyLeShEeT", true),
            (" stylesheet", false),
            ("stylesheet ", false),
            ("\u{17f}tylesheet", false),
            ("styleſheet", false),
            ("stylesheetx", false),
        ] {
            let raw = DocumentString::from(text);
            assert_eq!(ascii_keyword(&raw, "stylesheet"), expected);
            assert_eq!(
                ascii_keyword_units(raw.code_units(), "stylesheet"),
                expected
            );
        }
        let isolated = DocumentString::from_utf16([0xD800; 10].to_vec());
        assert!(!ascii_keyword_units(isolated.code_units(), "stylesheet"));
    }

    #[test]
    fn stylesheet_work_envelope_is_finite_for_empty_wide_and_representation_edges() {
        // n splitter reads + n+1 field visits + n token visits + 2n length
        // reads + 2n paired reads + n comparisons, plus 12 fixed actions.
        assert_eq!(stylesheet_link_context_work_bound(0), Some(13));
        assert_eq!(stylesheet_link_context_work_bound(10), Some(93));
        assert_eq!(stylesheet_link_context_work_bound(10_000), Some(80_013));
        let maximum_input = (usize::MAX - 13) / 8;
        assert_eq!(
            stylesheet_link_context_work_bound(maximum_input),
            Some(maximum_input * 8 + 13)
        );
        assert_eq!(stylesheet_link_context_work_bound(maximum_input + 1), None);
        assert_eq!(stylesheet_link_context_work_bound(usize::MAX), None);

        let raw = DocumentString::from("x ".repeat(5_000));
        let context = HtmlValueInterpreter::new(10_000)
            .stylesheet_link_context(Some(&raw))
            .unwrap();
        assert!(!context.contains_stylesheet());
        assert_eq!(context.rel().unwrap().code_units(), raw.code_units());
    }

    #[test]
    fn whole_rel_cap_precedes_an_early_stylesheet_match() {
        let raw = DocumentString::from("stylesheet ignored");
        let error = HtmlValueInterpreter::new(raw.len() - 1)
            .stylesheet_link_context(Some(&raw))
            .unwrap_err();
        assert_eq!(error.capacity(), raw.len() - 1);
        assert_eq!(error.required(), raw.len());
        let context = HtmlValueInterpreter::new(raw.len())
            .stylesheet_link_context(Some(&raw))
            .unwrap();
        assert!(context.contains_stylesheet());
        assert_eq!(context.rel().unwrap().code_units(), raw.code_units());
    }
}

pub(crate) mod media;
pub(crate) mod script_type;

#[cfg(test)]
mod total_enum_tests {
    use super::*;

    /// The state-determination steps of HTML §2.3.2.2, "Keywords and enumerated attributes"
    /// (references/html-language-foundations--877d6146487f--b55e73c31978.md, lines 2497-2512),
    /// over a keyword table written out here: not specified selects the missing value default; an
    /// ASCII case-insensitive keyword match selects its state; the empty string selects the empty
    /// value default when one is defined; everything else selects the invalid value default.
    fn oracle<T: Copy>(
        keywords: &[(&str, T)],
        missing: T,
        invalid: T,
        empty: Option<T>,
        raw: Option<&str>,
    ) -> T {
        let Some(raw) = raw else { return missing };
        if let Some((_, state)) = keywords
            .iter()
            .find(|(keyword, _)| keyword.eq_ignore_ascii_case(raw))
        {
            return *state;
        }
        if raw.is_empty()
            && let Some(state) = empty
        {
            return state;
        }
        invalid
    }

    const RAW: [Option<&str>; 14] = [
        None,
        Some(""),
        Some("anonymous"),
        Some("ANONYMOUS"),
        Some("Use-Credentials"),
        Some("use-credentials "),
        Some("no-referrer"),
        Some("Origin"),
        Some("origin-when-cross-origin"),
        Some("unsafe-url"),
        Some("text"),
        Some("DATETIME-LOCAL"),
        Some("anonymou"),
        Some("é"),
    ];

    #[test]
    fn crossorigin_processing_follows_the_enumerated_attribute_steps() {
        use HtmlCrossOrigin::{Anonymous, UseCredentials};
        let keywords = [
            ("anonymous", HtmlCrossOriginState::Keyword(Anonymous)),
            (
                "use-credentials",
                HtmlCrossOriginState::Keyword(UseCredentials),
            ),
        ];
        for raw in RAW {
            assert_eq!(
                bounded_rules::crossorigin_processing_with(
                    raw.map(DocumentString::from).as_ref(),
                    |_| Ok::<(), std::convert::Infallible>(())
                )
                .unwrap_or_else(|never| match never {}),
                oracle(
                    &keywords,
                    HtmlCrossOriginState::NoCors,
                    HtmlCrossOriginState::Keyword(Anonymous),
                    Some(HtmlCrossOriginState::Keyword(Anonymous)),
                    raw
                ),
                "{raw:?}"
            );
        }
    }

    #[test]
    fn referrer_policy_processing_follows_the_enumerated_attribute_steps() {
        let keywords = [
            ("", HtmlReferrerPolicy::Empty),
            ("no-referrer", HtmlReferrerPolicy::NoReferrer),
            (
                "no-referrer-when-downgrade",
                HtmlReferrerPolicy::NoReferrerWhenDowngrade,
            ),
            ("same-origin", HtmlReferrerPolicy::SameOrigin),
            ("origin", HtmlReferrerPolicy::Origin),
            ("strict-origin", HtmlReferrerPolicy::StrictOrigin),
            (
                "origin-when-cross-origin",
                HtmlReferrerPolicy::OriginWhenCrossOrigin,
            ),
            (
                "strict-origin-when-cross-origin",
                HtmlReferrerPolicy::StrictOriginWhenCrossOrigin,
            ),
            ("unsafe-url", HtmlReferrerPolicy::UnsafeUrl),
        ];
        for raw in RAW {
            assert_eq!(
                bounded_rules::referrerpolicy_processing_with(
                    raw.map(DocumentString::from).as_ref(),
                    |_| Ok::<(), std::convert::Infallible>(())
                )
                .unwrap_or_else(|never| match never {}),
                oracle(
                    &keywords,
                    HtmlReferrerPolicy::Empty,
                    HtmlReferrerPolicy::Empty,
                    None,
                    raw
                ),
                "{raw:?}"
            );
        }
    }

    #[test]
    fn the_input_type_and_colorspace_defaults_are_the_rules_own() {
        assert_eq!(HtmlInputType::missing_value_default(), HtmlInputType::Text);
        assert_eq!(input_type_invalid_default(), HtmlInputType::Text);
        assert_eq!(input_colorspace_default(), HtmlInputColorSpace::LimitedSrgb);
        assert_eq!(
            input_colorspace_invalid_default(),
            HtmlInputColorSpace::LimitedSrgb
        );
        // The rules select these same defaults for a missing and an unrecognized value.
        let interpreter = HtmlValueInterpreter::new(64);
        let input = QualifiedName::new(Namespace::html(), None, "input");
        let invalid = Attribute::new(KnownHtmlAttribute::Type.qualified_name(), "bogus");
        assert_eq!(
            interpreter
                .input_type(&input, Some(&invalid))
                .unwrap()
                .intrinsic(),
            Some(&input_type_invalid_default())
        );
        assert_eq!(
            interpreter
                .input_type(&input, None::<&Attribute>)
                .unwrap()
                .intrinsic(),
            Some(&HtmlInputType::missing_value_default())
        );
    }
}
