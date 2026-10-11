//! Intrinsic interpretation of selected HTML attributes over exact raw records.
//!
//! This bounded family covers global enumerations, global Boolean attributes,
//! and declarative-shadow template modes/flags. It does not resolve inheritance,
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
    ($name:ident, $type:ty, $attribute:ident, $context:ident, [$($row:literal),+], $source:literal,
     $missing:expr, $invalid:expr, $empty:expr, [$($spelling:literal => ($canonical:literal, $state:expr)),+ $(,)?]) => {
        const $name: EnumRule<$type> = EnumRule {
            metadata: RuleMetadata { attribute: KnownHtmlAttribute::$attribute, rows: &[$($row),+],
                source: source!($source), context: Context::$context },
            keywords: &[$(Keyword { spelling: $spelling, canonical: $canonical, state: $state }),+],
            missing: $missing, invalid: $invalid, empty: $empty,
        };
    };
}

enum_rule!(DIR, HtmlDirState, Dir, Direction, [41, 42], "attr-dir",
    Some(HtmlDirState::Undefined), Some(HtmlDirState::Undefined), None,
    ["ltr" => ("ltr", HtmlDirState::Keyword(HtmlDir::Ltr)),
     "rtl" => ("rtl", HtmlDirState::Keyword(HtmlDir::Rtl)),
     "auto" => ("auto", HtmlDirState::Keyword(HtmlDir::Auto))]);
enum_rule!(CONTENTEDITABLE, HtmlContentEditableState, Contenteditable, Global, [31], "attr-contenteditable",
    Some(HtmlContentEditableState::Inherit), Some(HtmlContentEditableState::Inherit), Some(HtmlContentEditableState::Keyword(HtmlContentEditable::True)),
    ["true" => ("true", HtmlContentEditableState::Keyword(HtmlContentEditable::True)),
     "false" => ("false", HtmlContentEditableState::Keyword(HtmlContentEditable::False)),
     "plaintext-only" => ("plaintext-only", HtmlContentEditableState::Keyword(HtmlContentEditable::PlaintextOnly))]);
enum_rule!(DRAGGABLE, HtmlDraggableState, Draggable, Global, [48], "attr-draggable",
    Some(HtmlDraggableState::Auto), Some(HtmlDraggableState::Auto), None,
    ["true" => ("true", HtmlDraggableState::Keyword(HtmlDraggable::True)),
     "false" => ("false", HtmlDraggableState::Keyword(HtmlDraggable::False))]);
enum_rule!(HIDDEN, HtmlHiddenState, Hidden, Global, [65], "attr-hidden",
    Some(HtmlHiddenState::NotHidden), Some(HtmlHiddenState::Keyword(HtmlHidden::Hidden)), Some(HtmlHiddenState::Keyword(HtmlHidden::Hidden)),
    ["hidden" => ("hidden", HtmlHiddenState::Keyword(HtmlHidden::Hidden)),
     "until-found" => ("until-found", HtmlHiddenState::Keyword(HtmlHidden::UntilFound))]);
enum_rule!(POPOVER, HtmlPopoverState, Popover, Global, [119], "attr-popover",
    Some(HtmlPopoverState::NoPopover), Some(HtmlPopoverState::Keyword(HtmlPopover::Manual)), Some(HtmlPopoverState::Keyword(HtmlPopover::Auto)),
    ["auto" => ("auto", HtmlPopoverState::Keyword(HtmlPopover::Auto)),
     "manual" => ("manual", HtmlPopoverState::Keyword(HtmlPopover::Manual)),
     "hint" => ("hint", HtmlPopoverState::Keyword(HtmlPopover::Hint))]);
enum_rule!(TRANSLATE, HtmlTranslateState, Translate, Global, [165], "attr-translate",
    Some(HtmlTranslateState::Inherit), Some(HtmlTranslateState::Inherit), Some(HtmlTranslateState::Keyword(HtmlTranslate::Yes)),
    ["yes" => ("yes", HtmlTranslateState::Keyword(HtmlTranslate::Yes)),
     "no" => ("no", HtmlTranslateState::Keyword(HtmlTranslate::No))]);
enum_rule!(SPELLCHECK, HtmlSpellcheckState, Spellcheck, Global, [148], "attr-spellcheck",
    Some(HtmlSpellcheckState::Default), Some(HtmlSpellcheckState::Default), Some(HtmlSpellcheckState::Keyword(HtmlSpellcheck::True)),
    ["true" => ("true", HtmlSpellcheckState::Keyword(HtmlSpellcheck::True)),
     "false" => ("false", HtmlSpellcheckState::Keyword(HtmlSpellcheck::False))]);
enum_rule!(AUTOCAPITALIZE, HtmlAutocapitalizeState, Autocapitalize, Global, [12], "attr-autocapitalize",
    Some(HtmlAutocapitalizeState::Default), Some(HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)), None,
    ["off" => ("none", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::None)),
     "none" => ("none", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::None)),
     "on" => ("sentences", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)),
     "sentences" => ("sentences", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)),
     "words" => ("words", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Words)),
     "characters" => ("characters", HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Characters))]);
enum_rule!(AUTOCORRECT, HtmlAutocorrect, Autocorrect, Global, [15], "attr-autocorrect",
    Some(HtmlAutocorrect::On), Some(HtmlAutocorrect::On), Some(HtmlAutocorrect::On),
    ["on" => ("on", HtmlAutocorrect::On), "off" => ("off", HtmlAutocorrect::Off)]);
enum_rule!(WRITINGSUGGESTIONS, HtmlWritingSuggestionsState, Writingsuggestions, Global, [180], "attr-writingsuggestions",
    Some(HtmlWritingSuggestionsState::Default), Some(HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)), Some(HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)),
    ["true" => ("true", HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)),
     "false" => ("false", HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::False))]);
enum_rule!(INPUTMODE, HtmlInputMode, Inputmode, Global, [76], "attr-inputmode",
    None, None, None,
    ["none" => ("none", HtmlInputMode::None), "text" => ("text", HtmlInputMode::Text),
     "tel" => ("tel", HtmlInputMode::Tel), "url" => ("url", HtmlInputMode::Url),
     "email" => ("email", HtmlInputMode::Email), "numeric" => ("numeric", HtmlInputMode::Numeric),
     "decimal" => ("decimal", HtmlInputMode::Decimal), "search" => ("search", HtmlInputMode::Search)]);
enum_rule!(ENTERKEYHINT, HtmlEnterKeyHint, Enterkeyhint, Global, [50], "attr-enterkeyhint",
    None, None, None,
    ["enter" => ("enter", HtmlEnterKeyHint::Enter), "done" => ("done", HtmlEnterKeyHint::Done),
     "go" => ("go", HtmlEnterKeyHint::Go), "next" => ("next", HtmlEnterKeyHint::Next),
     "previous" => ("previous", HtmlEnterKeyHint::Previous), "search" => ("search", HtmlEnterKeyHint::Search),
     "send" => ("send", HtmlEnterKeyHint::Send)]);
enum_rule!(SHADOWROOTMODE, HtmlShadowRootMode, Shadowrootmode, Template, [139], "attr-template-shadowrootmode",
    None, None, None,
    ["open" => ("open", HtmlShadowRootMode::Open), "closed" => ("closed", HtmlShadowRootMode::Closed)]);
enum_rule!(SHADOWROOTSLOTASSIGNMENT, HtmlSlotAssignment, Shadowrootslotassignment, Template, [141], "attr-template-shadowrootslotassignment",
    Some(HtmlSlotAssignment::Named), Some(HtmlSlotAssignment::Named), None,
    ["named" => ("named", HtmlSlotAssignment::Named), "manual" => ("manual", HtmlSlotAssignment::Manual)]);

/// The Boolean attributes admitted by this first intrinsic overlay family.
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
    if rule.context == Context::Template
        && KnownHtmlElement::from_name(element) != Some(KnownHtmlElement::Template)
    {
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
                if let Some(keyword) = rule
                    .keywords
                    .iter()
                    .find(|keyword| ascii_keyword(value, keyword.spelling))
                {
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
