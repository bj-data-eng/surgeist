//! Intrinsic interpretation of selected HTML attributes over exact raw records.
//!
//! This bounded family covers global and selected element-specific enumerations,
//! Boolean attributes, and declarative-shadow template modes/flags. It does not resolve inheritance,
//! editing, directionality, dragging, focus, microdata, or shadow-root creation.
//! The defining HTML clauses, rather than non-normative index summaries, own
//! these rules. WHATWG source attribution is in `licenses/whatwg/NOTICE.md`.

use crate::{
    Attribute, DocumentString, HtmlShadowRootMode, HtmlSlotAssignment, HtmlValueAdmissionError,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};

/// Exact presence of the supplied attribute, independent of validity/defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeObservation {
    Missing,
    PresentEmpty,
    PresentNonempty,
}

/// Whether the selected intrinsic rule applies to the supplied expanded names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeApplicability {
    Applicable,
    ForeignElementNamespace,
    DifferentElement,
    DifferentAttribute,
}

/// Recognition of literal value syntax; a fallback never changes this result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAttributeRecognition {
    Missing,
    Empty,
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
    BooleanSpelling,
    BdoDirectionRequired,
    BdoDirectionMustBeLtrOrRtl,
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
    intrinsic: Option<T>,
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
    /// Returns the attribute's typed intrinsic state, without contextual effects.
    pub fn intrinsic(&self) -> Option<&T> {
        self.intrinsic.as_ref()
    }
    /// Returns the selected standard attribute identity, not raw recognition.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.rule.attribute
    }
    /// Returns existing original index row IDs covered by this defining rule.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.rule.rows
    }
    /// Returns the immutable normative defining clause, not an index summary.
    pub fn source_clause(&self) -> &'static str {
        self.rule.source
    }
    /// Returns the bdo-specific defining clause when that author rule applies.
    pub fn context_source_clause(&self) -> Option<&'static str> {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuleMetadata {
    attribute: KnownHtmlAttribute,
    rows: &'static [u16],
    source: &'static str,
    context: Context,
}

struct Keyword<T> {
    spelling: &'static str,
    canonical: &'static str,
    state: T,
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
     $missing:expr, $invalid:expr, $empty:expr, [$($spelling:literal => ($canonical:literal, $state:expr)),+ $(,)?]) => {
        const $name: EnumRule<$type> = EnumRule {
            metadata: RuleMetadata { attribute: KnownHtmlAttribute::$attribute, rows: &[$($row),+],
                source: source!($source), context: $context },
            keywords: &[$(Keyword { spelling: $spelling, canonical: $canonical, state: $state }),+],
            missing: $missing, invalid: $invalid, empty: $empty, exact_keywords: false,
        };
    };
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

// ol[type] is an exact character table, not an ASCII-insensitive enumeration.
const OL_TYPE: EnumRule<HtmlOlType> = EnumRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Type,
        rows: &[170],
        source: source!("attr-ol-type"),
        context: Context::Elements(&[KnownHtmlElement::Ol]),
    },
    keywords: &[
        Keyword {
            spelling: "1",
            canonical: "1",
            state: HtmlOlType::Decimal,
        },
        Keyword {
            spelling: "a",
            canonical: "a",
            state: HtmlOlType::LowerAlpha,
        },
        Keyword {
            spelling: "A",
            canonical: "A",
            state: HtmlOlType::UpperAlpha,
        },
        Keyword {
            spelling: "i",
            canonical: "i",
            state: HtmlOlType::LowerRoman,
        },
        Keyword {
            spelling: "I",
            canonical: "I",
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
}

impl HtmlBooleanAttribute {
    fn rule(self) -> RuleMetadata {
        // These defining clauses each explicitly say Boolean attribute.
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
        };
        RuleMetadata {
            attribute,
            rows,
            source,
            context,
        }
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
        Context::Elements(elements) => known.is_some_and(|name| elements.contains(&name)),
    };
    if !applicable {
        return HtmlAttributeApplicability::DifferentElement;
    }
    HtmlAttributeApplicability::Applicable
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
    HtmlAttributeReport {
        element: element.clone(),
        raw_attribute: raw.cloned(),
        rule,
        observation,
        applicability: applies(rule, element, raw),
        recognition: HtmlAttributeRecognition::Unapplied,
        author: HtmlAttributeAuthorState::Unapplied,
        intrinsic: None,
    }
}

fn ascii_keyword(raw: &DocumentString, keyword: &str) -> bool {
    raw.code_units().len() == keyword.len()
        && raw
            .code_units()
            .iter()
            .zip(keyword.bytes())
            .all(|(&unit, byte)| {
                u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(&byte))
            })
}

impl HtmlValueInterpreter {
    fn enumerated<T: Copy>(
        &self,
        rule: &EnumRule<T>,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<T>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(rule.metadata, element, raw);
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        match raw {
            None => {
                result.recognition = HtmlAttributeRecognition::Missing;
                result.author = HtmlAttributeAuthorState::Missing;
                result.intrinsic = rule.missing;
            }
            Some(attribute) => {
                let value = attribute.value();
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
                    result.recognition = HtmlAttributeRecognition::Keyword {
                        matched: keyword.spelling,
                        canonical: keyword.canonical,
                    };
                    result.author = HtmlAttributeAuthorState::Conforming;
                    result.intrinsic = Some(keyword.state);
                } else if value.is_empty() {
                    result.recognition = HtmlAttributeRecognition::Empty;
                    result.intrinsic = rule.empty.or(rule.invalid);
                    result.author = if rule.empty.is_some() {
                        HtmlAttributeAuthorState::Conforming
                    } else {
                        HtmlAttributeAuthorState::Nonconforming(
                            HtmlAttributeConformanceError::UnrecognizedKeyword,
                        )
                    };
                } else {
                    result.recognition = HtmlAttributeRecognition::Unrecognized;
                    result.author = HtmlAttributeAuthorState::Nonconforming(
                        HtmlAttributeConformanceError::UnrecognizedKeyword,
                    );
                    result.intrinsic = rule.invalid;
                }
            }
        }
        Ok(result)
    }

    /// Interprets content Boolean presence; this does not compute its effects.
    pub fn boolean_attribute(
        &self,
        selected: HtmlBooleanAttribute,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<bool>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(selected.rule(), element, raw);
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
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
        Ok(result)
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
