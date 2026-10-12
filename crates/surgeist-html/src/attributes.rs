//! Intrinsic interpretation of selected HTML attributes over exact raw records.
//!
//! This bounded family covers global and selected element-specific enumerations,
//! Boolean attributes, exact scalar content values, submitter keyword overrides,
//! and declarative-shadow template modes/flags. It does not resolve inheritance,
//! editing, directionality, dragging, focus, microdata, or shadow-root creation.
//! The defining HTML clauses, rather than non-normative index summaries, own
//! these rules. WHATWG source attribution is in `licenses/whatwg/NOTICE.md`.
//! Referrer-policy vocabulary is qualified separately in its report metadata.

use crate::{
    Attribute, DocumentString, HtmlShadowRootMode, HtmlSlotAssignment, HtmlTokenError,
    HtmlValueAdmissionError, HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, Namespace,
    QualifiedName,
};

/// Exact presence of the supplied attribute, independent of validity/defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeObservation {
    Missing,
    PresentEmpty,
    PresentNonempty,
}

/// Whether the selected rule applies to the expanded names and fact family.
/// Author-only restrictions do not suppress known intrinsic content presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeApplicability {
    Applicable,
    ForeignElementNamespace,
    DifferentElement,
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
    ExternalClassic,
    InlineClassic,
    ExternalModule,
    InlineModule,
    ImportMap,
    SpeculationRules,
    DataBlock,
}

/// Explicit a/area href presence, without parsing or resolving the URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkAttributeContext {
    WithHref,
    WithoutHref,
}

/// Source-established autonomous custom-control classification.
///
/// This is a supplied fact about an actual element and its definition. The
/// interpreter does not infer registry, upgrade or form association from a name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomControlAttributeContext {
    FormAssociated,
    NotFormAssociated,
}

/// Actual button classification, including source-defined Auto-state context.
/// This is not inferred from the type keyword or used to submit a form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlButtonSubmitterContext {
    SubmitButton,
    NotSubmitButton,
}

/// Actual ancestor-chain evidence for img's ismap author requirement.
/// An HTML a ancestor with href present qualifies even when href is empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageMapAttributeContext {
    AncestorAnchorWithHref,
    NoAncestorAnchorWithHref,
    Unavailable,
}

/// Actual alt observation for img controls, without trimming its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageControlsAttributeContext {
    AltMissing,
    AltEmpty,
    AltNonempty,
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
    Conforming,
    Nonconforming(HtmlAttributeContextConformanceError),
    /// The selected rule does not apply to these names or operation context.
    Unapplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeContextConformanceError {
    ImageMapRequiresAncestorAnchorWithHref,
    ImageControlsRequireNonemptyAlt,
    StylesheetDisabledRequiresStylesheetRel,
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
}

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
    Missing,
    Empty,
    /// The selected text rule recognizes an exact nonempty value.
    NonemptyText,
    /// A command keyword begins with two exact ASCII hyphens.
    /// Independent author checks can still reject its boundary spelling.
    CustomCommand,
    Keyword {
        matched: &'static str,
        canonical: &'static str,
    },
    Unrecognized,
    BooleanName,
    Unapplied,
}

/// A value violates an author requirement of this bounded intrinsic family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeConformanceError {
    UnrecognizedKeyword,
    /// A source-recognized keyword spelling is forbidden in authored markup.
    NonconformingKeyword,
    BooleanSpelling,
    BdoDirectionRequired,
    BdoDirectionMustBeLtrOrRtl,
    /// A specified details name must not be the empty string.
    DetailsNameMustNotBeEmpty,
    /// An HTML id violates its local nonempty/no-ASCII-whitespace author rule.
    InvalidIdToken(HtmlTokenError),
    /// A target/name violates HTML's local navigable-target grammar.
    InvalidNavigableTarget(HtmlNavigableTargetError),
    /// The specified encoding label does not match HTML's required UTF-8 spelling.
    InvalidUtf8EncodingLabel(HtmlUtf8EncodingLabelError),
}

/// Intrinsic value author conformance is independent of state/applicability.
///
/// This also includes bdo's required direction value, but does not validate
/// serialized qualified-name syntax or other tree/element author requirements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeAuthorState {
    Missing,
    Conforming,
    Nonconforming(HtmlAttributeConformanceError),
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
        self.raw_attribute.as_ref().map(Attribute::value)
    }
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.observation
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.applicability
    }
    pub fn recognition(&self) -> HtmlAttributeRecognition {
        self.recognition
    }
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
    pub fn image_map_context(&self) -> Option<HtmlImageMapAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::ImageMap(context) => Some(*context),
            _ => None,
        }
    }
    pub fn image_controls_context(&self) -> Option<HtmlImageControlsAttributeContext> {
        match &self.supplied_context {
            SuppliedContext::ImageControls(context) => Some(*context),
            _ => None,
        }
    }
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
        if self.rule.context == Context::PopoverTargetAction
            && KnownHtmlElement::from_name(&self.element) == Some(KnownHtmlElement::Input)
        {
            return Some(match &self.supplied_context {
                SuppliedContext::Input(state) => input_state_source(*state),
                _ => INPUT_SOURCE,
            });
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
                None if self.element.namespace() == &Namespace::html()
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
            && self.element.namespace() == &Namespace::html()
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

/// Shared source applicability for the captured candidacy consumer.
/// Content Boolean presence and author-context reports remain separate.
pub(crate) fn input_readonly_applies(state: HtmlInputType) -> bool {
    InputBooleanApplicability::Readonly.allows(state)
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
    ($name:ident, $attribute:ident, [$($variant:ident => $keyword:literal),+ $(,)?]) => {
        #[doc = concat!("Conforming native keywords for the HTML ", stringify!($attribute), " attribute.")]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name { $($variant),+ }
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
    ($name:ident, $value:ident, $default:ident) => {
        #[doc = concat!("Intrinsic ", stringify!($value), " state; the default is not a conforming author keyword.")]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            Keyword($value),
            $default,
        }
    };
}

keyword_type!(HtmlDir, Dir, [Ltr => "ltr", Rtl => "rtl", Auto => "auto"]);
default_state!(HtmlDirState, HtmlDir, Undefined);
keyword_type!(HtmlContentEditable, Contenteditable, [True => "true", False => "false", PlaintextOnly => "plaintext-only"]);
default_state!(HtmlContentEditableState, HtmlContentEditable, Inherit);
keyword_type!(HtmlDraggable, Draggable, [True => "true", False => "false"]);
default_state!(HtmlDraggableState, HtmlDraggable, Auto);
keyword_type!(HtmlHidden, Hidden, [Hidden => "hidden", UntilFound => "until-found"]);
default_state!(HtmlHiddenState, HtmlHidden, NotHidden);
keyword_type!(HtmlPopover, Popover, [Auto => "auto", Manual => "manual", Hint => "hint"]);
default_state!(HtmlPopoverState, HtmlPopover, NoPopover);
keyword_type!(HtmlTranslate, Translate, [Yes => "yes", No => "no"]);
default_state!(HtmlTranslateState, HtmlTranslate, Inherit);
keyword_type!(HtmlSpellcheck, Spellcheck, [True => "true", False => "false"]);
default_state!(HtmlSpellcheckState, HtmlSpellcheck, Default);
keyword_type!(HtmlAutocapitalize, Autocapitalize, [None => "none", Sentences => "sentences", Words => "words", Characters => "characters"]);
default_state!(HtmlAutocapitalizeState, HtmlAutocapitalize, Default);
keyword_type!(HtmlAutocorrect, Autocorrect, [On => "on", Off => "off"]);
keyword_type!(HtmlWritingSuggestions, Writingsuggestions, [True => "true", False => "false"]);
default_state!(HtmlWritingSuggestionsState, HtmlWritingSuggestions, Default);
keyword_type!(HtmlInputMode, Inputmode, [None => "none", Text => "text", Tel => "tel", Url => "url", Email => "email", Numeric => "numeric", Decimal => "decimal", Search => "search"]);
keyword_type!(HtmlEnterKeyHint, Enterkeyhint, [Enter => "enter", Done => "done", Go => "go", Next => "next", Previous => "previous", Search => "search", Send => "send"]);

keyword_type!(HtmlClosedBy, Closedby, [Any => "any", CloseRequest => "closerequest", None => "none"]);
default_state!(HtmlClosedByState, HtmlClosedBy, Auto);
keyword_type!(HtmlFormAutocomplete, Autocomplete, [On => "on", Off => "off"]);
keyword_type!(HtmlDecoding, Decoding, [Sync => "sync", Async => "async", Auto => "auto"]);
keyword_type!(HtmlLoading, Loading, [Lazy => "lazy", Eager => "eager"]);
keyword_type!(HtmlPreload, Preload, [Auto => "auto", None => "none", Metadata => "metadata"]);

/// Intrinsic preload hint, with no invented implementation-defined choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlPreloadState {
    Keyword(HtmlPreload),
    /// HTML supplies no selected preload state for missing/invalid values.
    ImplementationDefined,
}

keyword_type!(HtmlFormMethod, Method, [Get => "get", Post => "post", Dialog => "dialog"]);
keyword_type!(HtmlFormEnctype, Enctype, [UrlEncoded => "application/x-www-form-urlencoded", MultipartFormData => "multipart/form-data", PlainText => "text/plain"]);
keyword_type!(HtmlTrackKind, Kind, [Subtitles => "subtitles", Captions => "captions", Descriptions => "descriptions", Chapters => "chapters", Metadata => "metadata"]);
keyword_type!(HtmlOlType, Type, [Decimal => "1", LowerAlpha => "a", UpperAlpha => "A", LowerRoman => "i", UpperRoman => "I"]);
keyword_type!(HtmlInputType, Type, [
    Hidden => "hidden", Text => "text", Search => "search", Telephone => "tel",
    Url => "url", Email => "email", Password => "password", Date => "date",
    Month => "month", Week => "week", Time => "time", LocalDateTime => "datetime-local",
    Number => "number", Range => "range", Color => "color", Checkbox => "checkbox",
    Radio => "radio", File => "file", Submit => "submit", Image => "image",
    Reset => "reset", Button => "button"
]);
keyword_type!(HtmlButtonType, Type, [Submit => "submit", Reset => "reset", Button => "button"]);
default_state!(HtmlButtonTypeState, HtmlButtonType, Auto);
keyword_type!(HtmlCrossOrigin, Crossorigin, [Anonymous => "anonymous", UseCredentials => "use-credentials"]);
default_state!(HtmlCrossOriginState, HtmlCrossOrigin, NoCors);
keyword_type!(HtmlFetchPriority, Fetchpriority, [High => "high", Low => "low", Auto => "auto"]);
keyword_type!(HtmlReferrerPolicy, Referrerpolicy, [
    Empty => "", NoReferrer => "no-referrer", NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
    SameOrigin => "same-origin", Origin => "origin", StrictOrigin => "strict-origin",
    OriginWhenCrossOrigin => "origin-when-cross-origin",
    StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin", UnsafeUrl => "unsafe-url"
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
enum_rule!(INPUT_TYPE, HtmlInputType, Type, Context::Elements(&[KnownHtmlElement::Input]), [169], "attr-input-type",
    Some(HtmlInputType::Text), Some(HtmlInputType::Text), None,
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
enum_rule!(CROSSORIGIN, HtmlCrossOriginState, Crossorigin,
    Context::Fetching { elements: &[KnownHtmlElement::Audio, KnownHtmlElement::Video, KnownHtmlElement::Img, KnownHtmlElement::Link, KnownHtmlElement::Script], script: ScriptApplicability::ClassicOrModule, hyperlink: false },
    [34], "cors-settings-attributes",
    Some(HtmlCrossOriginState::NoCors), Some(HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)), Some(HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)),
    ["anonymous" => ("anonymous", HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous)),
     "use-credentials" => ("use-credentials", HtmlCrossOriginState::Keyword(HtmlCrossOrigin::UseCredentials))]);
enum_rule!(FETCHPRIORITY, HtmlFetchPriority, Fetchpriority,
    Context::Fetching { elements: &[KnownHtmlElement::Img, KnownHtmlElement::Link, KnownHtmlElement::Script], script: ScriptApplicability::ExternalClassicOrModule, hyperlink: false },
    [51], "fetch-priority-attributes",
    Some(HtmlFetchPriority::Auto), Some(HtmlFetchPriority::Auto), None,
    ["high" => ("high", HtmlFetchPriority::High), "low" => ("low", HtmlFetchPriority::Low), "auto" => ("auto", HtmlFetchPriority::Auto)]);
enum_rule!(REFERRERPOLICY, HtmlReferrerPolicy, Referrerpolicy,
    Context::Fetching { elements: &[KnownHtmlElement::A, KnownHtmlElement::Area, KnownHtmlElement::Iframe, KnownHtmlElement::Img, KnownHtmlElement::Link, KnownHtmlElement::Script], script: ScriptApplicability::ClassicOrModule, hyperlink: true },
    [126], "referrer-policy-attributes",
    Some(HtmlReferrerPolicy::Empty), Some(HtmlReferrerPolicy::Empty), None,
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
    Autofocus,
    Inert,
    Itemscope,
    HeadingReset,
    ShadowRootClonable,
    ShadowRootCustomElementRegistry,
    ShadowRootDelegatesFocus,
    ShadowRootSerializable,
    AllowFullscreen,
    Autoplay,
    /// The audio/video controls rule; this does not select img's controls rule.
    MediaControls,
    TrackDefault,
    Loop,
    Muted,
    Novalidate,
    DetailsOpen,
    DialogOpen,
    PlaysInline,
    Reversed,
    Selected,
    /// Local disabled content presence, with defining source chosen by context.
    /// This does not compute inherited or actual disabledness.
    Disabled,
    /// Color-input alpha content presence, not a CSS color interpretation.
    Alpha,
    /// Default checkedness content presence, not current checkedness.
    Checked,
    Multiple,
    Readonly,
    Required,
    /// Content presence on source-qualified submitters, not no-validate state.
    Formnovalidate,
    /// Script content presence, not scheduling, force-async or execution state.
    Async,
    Defer,
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

    fn rule_for(self, element: &QualifiedName) -> RuleMetadata {
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
            (Self::Readonly, None) if element.namespace() == &Namespace::html() => {
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

fn applies(
    rule: RuleMetadata,
    element: &QualifiedName,
    raw: Option<&Attribute>,
) -> HtmlAttributeApplicability {
    if raw.is_some_and(|a| KnownHtmlAttribute::from_name(a.name()) != Some(rule.attribute)) {
        return HtmlAttributeApplicability::DifferentAttribute;
    }
    if element.namespace() != &Namespace::html() {
        return HtmlAttributeApplicability::ForeignElementNamespace;
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

fn has_hyperlink_author_constraint(rule: RuleMetadata, element: &QualifiedName) -> bool {
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

fn has_boolean_author_constraint(rule: RuleMetadata, element: &QualifiedName) -> bool {
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

fn report<T>(
    rule: RuleMetadata,
    element: &QualifiedName,
    raw: Option<&Attribute>,
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
        element: element.clone(),
        raw_attribute: raw.cloned(),
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
        && units.iter().zip(keyword.bytes()).all(|(&unit, byte)| {
            u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(&byte))
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
pub(crate) fn stylesheet_link_context_work_bound(input_code_units: usize) -> Option<usize> {
    input_code_units.checked_mul(8)?.checked_add(13)
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
    let mut selection = EnumValueSelection {
        recognition: HtmlAttributeRecognition::Unapplied,
        author: HtmlAttributeAuthorState::Unapplied,
        intrinsic: None,
    };
    match raw {
        None => {
            selection.recognition = HtmlAttributeRecognition::Missing;
            selection.author = HtmlAttributeAuthorState::Missing;
            selection.intrinsic = rule.missing;
        }
        Some(value) => {
            if let Some(keyword) = rule.keywords.iter().find(|keyword| {
                if rule.exact_keywords {
                    value
                        .code_units()
                        .iter()
                        .copied()
                        .eq(keyword.spelling.encode_utf16())
                } else {
                    ascii_keyword(value, keyword.spelling)
                }
            }) {
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
            } else if value.is_empty() {
                selection.recognition = HtmlAttributeRecognition::Empty;
                selection.intrinsic = rule.empty.or(rule.invalid);
                selection.author = if rule.empty.is_some() {
                    HtmlAttributeAuthorState::Conforming
                } else {
                    HtmlAttributeAuthorState::Nonconforming(
                        HtmlAttributeConformanceError::UnrecognizedKeyword,
                    )
                };
            } else {
                selection.recognition = HtmlAttributeRecognition::Unrecognized;
                selection.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::UnrecognizedKeyword,
                );
                selection.intrinsic = rule.invalid;
            }
        }
    }
    selection
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WritingSuggestionsProcessing {
    pub(crate) recognition: HtmlAttributeRecognition,
    pub(crate) intrinsic: Option<HtmlWritingSuggestionsState>,
}

impl HtmlValueInterpreter {
    /// Content-state processing for the HTML-defined generic parent algorithm.
    /// This makes no element applicability or author-conformance assertion.
    pub(crate) fn writingsuggestions_processing(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<WritingSuggestionsProcessing, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw)?;
        let selection = enum_value_selection(&WRITINGSUGGESTIONS, raw);
        Ok(WritingSuggestionsProcessing {
            recognition: selection.recognition,
            intrinsic: selection.intrinsic,
        })
    }

    fn enumerated<T: Copy>(
        &self,
        rule: &EnumRule<T>,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<T>, HtmlValueAdmissionError> {
        self.enumerated_with_context(rule, element, raw, SuppliedContext::None)
    }

    fn enumerated_with_context<T: Copy>(
        &self,
        rule: &EnumRule<T>,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        supplied_context: SuppliedContext,
    ) -> Result<HtmlAttributeReport<T>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
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
        let selection = enum_value_selection(rule, raw.map(Attribute::value));
        result.recognition = selection.recognition;
        result.author = selection.author;
        result.intrinsic = selection.intrinsic;
        Ok(result)
    }

    /// Interprets content Boolean presence; this does not compute its effects.
    pub fn boolean_attribute(
        &self,
        selected: HtmlBooleanAttribute,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(selected, element, raw, SuppliedContext::None)
    }

    fn boolean_with_context(
        &self,
        selected: HtmlBooleanAttribute,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        supplied_context: SuppliedContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
            crate::tokens::space_token_slices(raw)
                .any(|token| ascii_keyword_units(token, "stylesheet"))
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(selected, element, raw, SuppliedContext::Input(state))
    }

    /// Interprets script Boolean content presence with actual kind/src facts.
    /// Scheduling, force-async, module support and execution remain external.
    pub fn boolean_attribute_for_script(
        &self,
        selected: HtmlBooleanAttribute,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        context: HtmlScriptAttributeContext,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_with_context(selected, element, raw, SuppliedContext::Script(context))
    }

    /// Interprets formnovalidate presence on an actually classified button.
    /// This does not compute form ownership or the effective no-validate state.
    pub fn formnovalidate_for_button(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.boolean_attribute(HtmlBooleanAttribute::Disabled, element, raw)
    }

    /// Interprets disabled for an explicitly classified autonomous custom control.
    pub fn disabled_for_custom_control(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
            pub fn $method(&self, element: &QualifiedName, raw: Option<&Attribute>) -> Result<HtmlAttributeReport<$type>, HtmlValueAdmissionError> {
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
                element: &QualifiedName,
                raw: Option<&Attribute>,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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

mod scalar;
pub use scalar::*;

mod autocomplete;
pub use autocomplete::*;

mod accept;
pub use accept::*;

mod control_length;
pub use control_length::*;

mod sandbox;
pub use sandbox::*;

mod input_bounds;
pub use input_bounds::*;

mod srcset;
pub use srcset::*;

mod command;
pub use command::*;

mod modification;
pub use modification::*;

mod area;
pub use area::*;

mod progress;
pub use progress::*;

mod meter;
pub use meter::*;

mod details_name;
pub use details_name::*;

mod id;

mod navigable_target;
pub use navigable_target::*;

mod textarea_wrap;
pub use textarea_wrap::*;

mod hash_name;
pub use hash_name::*;

mod input_colorspace;
pub use input_colorspace::*;

mod encoding_label;
pub use encoding_label::*;

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
