use std::ops::Range;

use serde::{Deserialize, Serialize};
use surgeist_css::{
    CssDeclarationList, CssFontFaceDescriptorRef, CssKnownProperty, CssPropertyNameRef, CssRule,
    CssSheet,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPoint {
    Sheet,
    StyleAttribute,
}

impl EntryPoint {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sheet => "sheet",
            Self::StyleAttribute => "style_attribute",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Adapter {
    Stylesheet,
    TopLevelRule,
    TopLevelAtRule,
    StyleDeclarationList,
    StyleDeclaration,
    StyleBlock,
    SelectorList,
    Selector,
    NthSelector,
    MozAnySelector,
    WebkitAnySelector,
    DirSelector,
    HasSelector,
    HostContextSelector,
    HostSelector,
    IsSelector,
    LangSelector,
    MatchesSelector,
    NotSelector,
    SlottedSelector,
    WhereSelector,
    MediaQuery,
    MediaAtRulePrelude,
    PropertyValue,
    FontFaceDescriptorValue,
    CustomPropertyContainment,
}

impl Adapter {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Stylesheet => "stylesheet",
            Self::TopLevelRule => "top_level_rule",
            Self::TopLevelAtRule => "top_level_at_rule",
            Self::StyleDeclarationList => "style_declaration_list",
            Self::StyleDeclaration => "style_declaration",
            Self::StyleBlock => "style_block",
            Self::SelectorList => "selector_list",
            Self::Selector => "selector",
            Self::NthSelector => "nth_selector",
            Self::MozAnySelector => "moz_any_selector",
            Self::WebkitAnySelector => "webkit_any_selector",
            Self::DirSelector => "dir_selector",
            Self::HasSelector => "has_selector",
            Self::HostContextSelector => "host_context_selector",
            Self::HostSelector => "host_selector",
            Self::IsSelector => "is_selector",
            Self::LangSelector => "lang_selector",
            Self::MatchesSelector => "matches_selector",
            Self::NotSelector => "not_selector",
            Self::SlottedSelector => "slotted_selector",
            Self::WhereSelector => "where_selector",
            Self::MediaQuery => "media_query",
            Self::MediaAtRulePrelude => "media_at_rule_prelude",
            Self::PropertyValue => "property_value",
            Self::FontFaceDescriptorValue => "font_face_descriptor_value",
            Self::CustomPropertyContainment => "custom_property_containment",
        }
    }

    pub fn wrap(
        self,
        input: &str,
        property_or_descriptor: Option<PropertyOrDescriptor>,
    ) -> Result<CompleteInput, Mismatch> {
        let (prefix, suffix) = match self {
            Self::Stylesheet | Self::TopLevelRule | Self::TopLevelAtRule => ("", ""),
            Self::StyleDeclarationList | Self::StyleDeclaration => (".surgeist-corpus-probe{", "}"),
            Self::StyleBlock => (".surgeist-corpus-probe", ""),
            Self::SelectorList
            | Self::Selector
            | Self::NthSelector
            | Self::MozAnySelector
            | Self::WebkitAnySelector
            | Self::DirSelector
            | Self::HasSelector
            | Self::HostContextSelector
            | Self::HostSelector
            | Self::IsSelector
            | Self::LangSelector
            | Self::MatchesSelector
            | Self::NotSelector
            | Self::SlottedSelector
            | Self::WhereSelector => (
                "@namespace ns \"surgeist-corpus-probe\";",
                "{--surgeist-corpus-probe:0;}",
            ),
            Self::MediaQuery => ("@media ", "{}"),
            Self::MediaAtRulePrelude => ("@media ", "{}"),
            Self::PropertyValue => match property_or_descriptor {
                Some(PropertyOrDescriptor::Property(property)) => {
                    return CompleteInput::from_owned_parts(
                        format!("{}:", property.canonical_name()),
                        input,
                        ";".to_owned(),
                    );
                }
                _ => return Err(Mismatch::MissingPropertyOrDescriptor { adapter: self }),
            },
            Self::FontFaceDescriptorValue => match property_or_descriptor {
                Some(PropertyOrDescriptor::FontFaceDescriptor(descriptor)) => {
                    let prefix = if descriptor == FontFaceDescriptorKind::UnicodeRange {
                        "@FONT-FACE{FONT-FAMILY:X;SRC:URL(X);UNICODE-RANGE:".to_owned()
                    } else {
                        format!(
                            "@font-face{{font-family:surgeist-corpus-probe;src:url(surgeist-corpus-probe);{}:",
                            descriptor.css_name()
                        )
                    };
                    return CompleteInput::from_owned_parts(prefix, input, ";}".to_owned());
                }
                _ => return Err(Mismatch::MissingPropertyOrDescriptor { adapter: self }),
            },
            Self::CustomPropertyContainment => ("--surgeist-corpus-probe:", ";"),
        };
        if property_or_descriptor.is_some() {
            return Err(Mismatch::UnexpectedPropertyOrDescriptor { adapter: self });
        }
        CompleteInput::from_parts(prefix, input, suffix)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopLevelRuleKind {
    Import,
    Namespace,
    CounterStyle,
    Page,
    LayerStatement,
    LayerBlock,
    FontFace,
    Keyframes,
    Style,
    Media,
    Supports,
    Container,
    Scope,
}

impl TopLevelRuleKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Namespace => "namespace",
            Self::CounterStyle => "counter_style",
            Self::Page => "page",
            Self::LayerStatement => "layer_statement",
            Self::LayerBlock => "layer_block",
            Self::FontFace => "font_face",
            Self::Keyframes => "keyframes",
            Self::Style => "style",
            Self::Media => "media",
            Self::Supports => "supports",
            Self::Container => "container",
            Self::Scope => "scope",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FontFaceDescriptorKind {
    FontFamily,
    Src,
    FontWeight,
    FontStyle,
    FontStretch,
    FontDisplay,
    UnicodeRange,
    FontFeatureSettings,
}

impl FontFaceDescriptorKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::FontFamily => "font_family",
            Self::Src => "src",
            Self::FontWeight => "font_weight",
            Self::FontStyle => "font_style",
            Self::FontStretch => "font_stretch",
            Self::FontDisplay => "font_display",
            Self::UnicodeRange => "unicode_range",
            Self::FontFeatureSettings => "font_feature_settings",
        }
    }

    const fn css_name(self) -> &'static str {
        match self {
            Self::FontFamily => "font-family",
            Self::Src => "src",
            Self::FontWeight => "font-weight",
            Self::FontStyle => "font-style",
            Self::FontStretch => "font-stretch",
            Self::FontDisplay => "font-display",
            Self::UnicodeRange => "unicode-range",
            Self::FontFeatureSettings => "font-feature-settings",
        }
    }

    fn matches(self, occurrence: CssFontFaceDescriptorRef<'_>) -> bool {
        matches!(
            (self, occurrence),
            (Self::FontFamily, CssFontFaceDescriptorRef::FontFamily(_))
                | (Self::Src, CssFontFaceDescriptorRef::Src(_))
                | (Self::FontWeight, CssFontFaceDescriptorRef::FontWeight(_))
                | (Self::FontStyle, CssFontFaceDescriptorRef::FontStyle(_))
                | (Self::FontStretch, CssFontFaceDescriptorRef::FontStretch(_))
                | (Self::FontDisplay, CssFontFaceDescriptorRef::FontDisplay(_))
                | (
                    Self::UnicodeRange,
                    CssFontFaceDescriptorRef::UnicodeRange(_)
                )
                | (
                    Self::FontFeatureSettings,
                    CssFontFaceDescriptorRef::FontFeatureSettings(_)
                )
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyOrDescriptor {
    Property(CssKnownProperty),
    FontFaceDescriptor(FontFaceDescriptorKind),
}

impl PropertyOrDescriptor {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Property(property) => property.canonical_name(),
            Self::FontFaceDescriptor(descriptor) => descriptor.css_name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Extractor {
    SheetRules,
    TopLevelRuleKind(TopLevelRuleKind),
    StyleDeclarations,
    StyleSelector,
    DeclarationList,
    MediaQueries,
    MediaChildren,
    SupportsChildren,
    ContainerChildren,
    ScopeChildren,
    LayerBlockChildren,
    FontFaceDescriptor(FontFaceDescriptorKind),
    KnownDeclaration {
        index: usize,
        property: CssKnownProperty,
    },
}

impl Extractor {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SheetRules => "sheet_rules",
            Self::TopLevelRuleKind(_) => "top_level_rule_kind",
            Self::StyleDeclarations => "style_declarations",
            Self::StyleSelector => "style_selector",
            Self::DeclarationList => "declaration_list",
            Self::MediaQueries => "media_queries",
            Self::MediaChildren => "media_children",
            Self::SupportsChildren => "supports_children",
            Self::ContainerChildren => "container_children",
            Self::ScopeChildren => "scope_children",
            Self::LayerBlockChildren => "layer_block_children",
            Self::FontFaceDescriptor(_) => "font_face_descriptor",
            Self::KnownDeclaration { .. } => "known_declaration",
        }
    }

    pub fn extract_sheet(self, sheet: &CssSheet) -> Result<usize, Mismatch> {
        let rules = sheet.rules();
        let count = match self {
            Self::SheetRules => rules.len(),
            Self::TopLevelRuleKind(kind) => {
                rules.iter().filter(|rule| kind.matches_rule(rule)).count()
            }
            Self::StyleDeclarations => rules
                .iter()
                .find_map(|rule| match rule {
                    CssRule::Style(rule) => Some(rule.declarations().len()),
                    _ => None,
                })
                .unwrap_or(0),
            Self::StyleSelector => rules
                .iter()
                .find_map(|rule| match rule {
                    CssRule::Style(rule) => {
                        let _ = rule.selector();
                        Some(1)
                    }
                    _ => None,
                })
                .unwrap_or(0),
            Self::MediaQueries => rules
                .iter()
                .find_map(|rule| match rule {
                    CssRule::Media(rule) => Some(rule.query().queries().len()),
                    _ => None,
                })
                .unwrap_or(0),
            Self::MediaChildren => first_rule_count(rules, |rule| match rule {
                CssRule::Media(rule) => Some(rule.rules().len()),
                _ => None,
            }),
            Self::SupportsChildren => first_rule_count(rules, |rule| match rule {
                CssRule::Supports(rule) => Some(rule.rules().len()),
                _ => None,
            }),
            Self::ContainerChildren => first_rule_count(rules, |rule| match rule {
                CssRule::Container(rule) => Some(rule.rules().len()),
                _ => None,
            }),
            Self::ScopeChildren => first_rule_count(rules, |rule| match rule {
                CssRule::Scope(rule) => Some(rule.rules().rules().len()),
                _ => None,
            }),
            Self::LayerBlockChildren => first_rule_count(rules, |rule| match rule {
                CssRule::LayerBlock(rule) => Some(rule.rules().len()),
                _ => None,
            }),
            Self::FontFaceDescriptor(descriptor) => rules
                .iter()
                .find_map(|rule| match rule {
                    CssRule::FontFace(rule) => Some(
                        rule.descriptors()
                            .occurrences()
                            .filter(|occurrence| descriptor.matches(*occurrence))
                            .count(),
                    ),
                    _ => None,
                })
                .unwrap_or(0),
            Self::KnownDeclaration { index, property } => rules
                .iter()
                .find_map(|rule| match rule {
                    CssRule::Style(rule) => Some(rule.declarations()),
                    _ => None,
                })
                .map_or(0, |declarations| {
                    known_declaration(declarations, index, property)
                }),
            Self::DeclarationList => {
                return Err(Mismatch::ExtractorEntryPoint {
                    extractor: self,
                    entry_point: EntryPoint::Sheet,
                });
            }
        };
        Ok(count)
    }

    pub fn extract_declaration_list(
        self,
        declarations: &CssDeclarationList,
    ) -> Result<usize, Mismatch> {
        match self {
            Self::DeclarationList => Ok(declarations.len()),
            Self::KnownDeclaration { index, property } => {
                Ok(known_declaration(declarations, index, property))
            }
            _ => Err(Mismatch::ExtractorEntryPoint {
                extractor: self,
                entry_point: EntryPoint::StyleAttribute,
            }),
        }
    }
}

impl TopLevelRuleKind {
    fn matches_rule(self, rule: &CssRule) -> bool {
        matches!(
            (self, rule),
            (Self::Import, CssRule::Import(_))
                | (Self::Namespace, CssRule::Namespace(_))
                | (Self::CounterStyle, CssRule::CounterStyle(_))
                | (Self::Page, CssRule::Page(_))
                | (Self::LayerStatement, CssRule::LayerStatement(_))
                | (Self::LayerBlock, CssRule::LayerBlock(_))
                | (Self::FontFace, CssRule::FontFace(_))
                | (Self::Keyframes, CssRule::Keyframes(_))
                | (Self::Style, CssRule::Style(_))
                | (Self::Media, CssRule::Media(_))
                | (Self::Supports, CssRule::Supports(_))
                | (Self::Container, CssRule::Container(_))
                | (Self::Scope, CssRule::Scope(_))
        )
    }
}

fn first_rule_count(rules: &[CssRule], extract: impl Fn(&CssRule) -> Option<usize>) -> usize {
    rules.iter().find_map(extract).unwrap_or(0)
}

fn known_declaration(
    declarations: &CssDeclarationList,
    index: usize,
    property: CssKnownProperty,
) -> usize {
    usize::from(matches!(
        declarations.get(index).map(|declaration| declaration.property_name()),
        Some(CssPropertyNameRef::Known(actual)) if actual == property
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteInput {
    source: String,
    payload_span: Range<usize>,
}

impl CompleteInput {
    fn from_parts(prefix: &str, input: &str, suffix: &str) -> Result<Self, Mismatch> {
        Self::from_owned_parts(prefix.to_owned(), input, suffix.to_owned())
    }

    fn from_owned_parts(prefix: String, input: &str, suffix: String) -> Result<Self, Mismatch> {
        let payload_span = prefix.len()..prefix.len() + input.len();
        let mut source = String::with_capacity(prefix.len() + input.len() + suffix.len());
        source.push_str(&prefix);
        source.push_str(input);
        source.push_str(&suffix);
        if source.get(payload_span.clone()) != Some(input)
            || source.get(..payload_span.start) != Some(prefix.as_str())
            || source.get(payload_span.end..) != Some(suffix.as_str())
            || (!input.is_empty() && source.match_indices(input).count() != 1)
        {
            return Err(Mismatch::PayloadNotContiguous);
        }
        Ok(Self {
            source,
            payload_span,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn payload_span(&self) -> Range<usize> {
        self.payload_span.clone()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionsProfile {
    Empty,
    AtruleMedia,
    ParseAtrulePreludeFalse,
    ParseCustomPropertyTrue,
    ParseRulePreludeFalse,
    ParseValueFalse,
    CustomPropertyVar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedPolicy {
    FullObservation,
    PanicFreedomOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedReason {
    GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor,
}

impl UnsupportedReason {
    pub const fn name(self) -> &'static str {
        match self {
            Self::GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor => {
                "generic_fragment_without_truthful_supported_property_or_descriptor"
            }
        }
    }
}

impl UnsupportedPolicy {
    pub const fn name(self) -> &'static str {
        match self {
            Self::FullObservation => "full_observation",
            Self::PanicFreedomOnly => "panic_freedom_only",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mismatch {
    MissingPropertyOrDescriptor {
        adapter: Adapter,
    },
    UnexpectedPropertyOrDescriptor {
        adapter: Adapter,
    },
    PayloadNotContiguous,
    ExtractorEntryPoint {
        extractor: Extractor,
        entry_point: EntryPoint,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct RegistryEntry {
    fixture_path: &'static str,
    context: &'static str,
    entry_point: EntryPoint,
    adapter: Adapter,
    extractor: Extractor,
    property_or_descriptor: Option<PropertyOrDescriptor>,
    options: &'static [OptionsProfile],
    unsupported_policy: Option<UnsupportedPolicy>,
    unsupported_reason: Option<UnsupportedReason>,
}

impl RegistryEntry {
    pub const fn fixture_path(self) -> &'static str {
        self.fixture_path
    }

    pub const fn context(self) -> &'static str {
        self.context
    }

    pub const fn entry_point(self) -> EntryPoint {
        self.entry_point
    }

    pub const fn adapter(self) -> Adapter {
        self.adapter
    }

    pub const fn extractor(self) -> Extractor {
        self.extractor
    }

    pub const fn property_or_descriptor(self) -> Option<PropertyOrDescriptor> {
        self.property_or_descriptor
    }

    pub const fn accepts_options(self, options: OptionsProfile) -> bool {
        let mut index = 0;
        while index < self.options.len() {
            if self.options[index] as u8 == options as u8 {
                return true;
            }
            index += 1;
        }
        false
    }

    pub const fn unsupported_policy(self) -> Option<UnsupportedPolicy> {
        self.unsupported_policy
    }

    pub const fn unsupported_reason(self) -> Option<UnsupportedReason> {
        self.unsupported_reason
    }

    pub const fn has_truthful_complete_input(self) -> bool {
        matches!(
            self.unsupported_policy,
            Some(UnsupportedPolicy::FullObservation)
        ) && self.unsupported_reason.is_none()
    }

    pub const fn adapter_name(self) -> &'static str {
        self.adapter.name()
    }

    pub const fn entry_point_name(self) -> &'static str {
        self.entry_point.name()
    }

    pub const fn extractor_name(self) -> &'static str {
        self.extractor.name()
    }

    pub const fn unsupported_policy_name(self) -> Option<&'static str> {
        match self.unsupported_policy {
            Some(policy) => Some(policy.name()),
            None => None,
        }
    }

    pub fn has_legal_context_combination(self) -> bool {
        match self.context {
            "stylesheet" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::Stylesheet)
            ),
            "rule" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::TopLevelRule)
            ),
            "atrule" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::TopLevelAtRule)
            ),
            "declarationList" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::StyleDeclarationList)
            ),
            "declaration" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::StyleDeclaration)
            ),
            "block" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::StyleBlock)
            ),
            "selectorList" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::SelectorList)
            ),
            "selector" => matches!(
                (self.entry_point, self.adapter),
                (
                    EntryPoint::Sheet,
                    Adapter::Selector
                        | Adapter::NthSelector
                        | Adapter::MozAnySelector
                        | Adapter::WebkitAnySelector
                        | Adapter::DirSelector
                        | Adapter::HasSelector
                        | Adapter::HostContextSelector
                        | Adapter::HostSelector
                        | Adapter::IsSelector
                        | Adapter::LangSelector
                        | Adapter::MatchesSelector
                        | Adapter::NotSelector
                        | Adapter::SlottedSelector
                        | Adapter::WhereSelector
                ) | (
                    EntryPoint::StyleAttribute,
                    Adapter::CustomPropertyContainment
                )
            ),
            "mediaQuery" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::MediaQuery)
            ),
            "atrulePrelude" => matches!(
                (self.entry_point, self.adapter),
                (EntryPoint::Sheet, Adapter::MediaAtRulePrelude)
            ),
            "value" => matches!(
                (self.entry_point, self.adapter),
                (
                    EntryPoint::StyleAttribute,
                    Adapter::PropertyValue | Adapter::CustomPropertyContainment
                ) | (EntryPoint::Sheet, Adapter::FontFaceDescriptorValue)
            ),
            _ => false,
        }
    }
}

pub fn validate_closed_model() -> bool {
    let sheet = CssSheet::new();
    let rule_kinds = [
        TopLevelRuleKind::Import,
        TopLevelRuleKind::Namespace,
        TopLevelRuleKind::CounterStyle,
        TopLevelRuleKind::Page,
        TopLevelRuleKind::LayerStatement,
        TopLevelRuleKind::LayerBlock,
        TopLevelRuleKind::FontFace,
        TopLevelRuleKind::Keyframes,
        TopLevelRuleKind::Style,
        TopLevelRuleKind::Media,
        TopLevelRuleKind::Supports,
        TopLevelRuleKind::Container,
        TopLevelRuleKind::Scope,
    ];
    let descriptors = [
        FontFaceDescriptorKind::FontFamily,
        FontFaceDescriptorKind::Src,
        FontFaceDescriptorKind::FontWeight,
        FontFaceDescriptorKind::FontStyle,
        FontFaceDescriptorKind::FontStretch,
        FontFaceDescriptorKind::FontDisplay,
        FontFaceDescriptorKind::UnicodeRange,
        FontFaceDescriptorKind::FontFeatureSettings,
    ];
    let sheet_extractors = [
        Extractor::SheetRules,
        Extractor::StyleDeclarations,
        Extractor::StyleSelector,
        Extractor::MediaQueries,
        Extractor::MediaChildren,
        Extractor::SupportsChildren,
        Extractor::ContainerChildren,
        Extractor::ScopeChildren,
        Extractor::LayerBlockChildren,
    ];
    let declaration_extractor: fn(Extractor, &CssDeclarationList) -> Result<usize, Mismatch> =
        Extractor::extract_declaration_list;
    let _ = declaration_extractor;

    sheet_extractors
        .into_iter()
        .chain(rule_kinds.map(Extractor::TopLevelRuleKind))
        .chain(descriptors.map(Extractor::FontFaceDescriptor))
        .all(|extractor| extractor.extract_sheet(&sheet) == Ok(0))
        && Extractor::DeclarationList.extract_sheet(&sheet)
            == Err(Mismatch::ExtractorEntryPoint {
                extractor: Extractor::DeclarationList,
                entry_point: EntryPoint::Sheet,
            })
        && EntryPoint::Sheet.name() == "sheet"
        && UnsupportedPolicy::FullObservation.name() == "full_observation"
        && REGISTRY.iter().all(|entry| {
            !entry.adapter_name().is_empty()
                && !entry.entry_point_name().is_empty()
                && !entry.extractor_name().is_empty()
                && entry.unsupported_policy_name().is_some()
                && match (
                    entry.has_truthful_complete_input(),
                    entry.unsupported_policy(),
                    entry.unsupported_reason(),
                ) {
                    (true, Some(UnsupportedPolicy::FullObservation), None) => true,
                    (false, Some(UnsupportedPolicy::PanicFreedomOnly), Some(reason)) => {
                        !reason.name().is_empty()
                    }
                    _ => false,
                }
        })
}

const EMPTY: &[OptionsProfile] = &[OptionsProfile::Empty];
const EMPTY_OR_NO_ATRULE_PRELUDE: &[OptionsProfile] = &[
    OptionsProfile::Empty,
    OptionsProfile::ParseAtrulePreludeFalse,
];
const EMPTY_OR_NO_RULE_PRELUDE: &[OptionsProfile] =
    &[OptionsProfile::Empty, OptionsProfile::ParseRulePreludeFalse];
const EMPTY_OR_NO_VALUE: &[OptionsProfile] =
    &[OptionsProfile::Empty, OptionsProfile::ParseValueFalse];
const EMPTY_OR_CUSTOM: &[OptionsProfile] = &[
    OptionsProfile::Empty,
    OptionsProfile::ParseCustomPropertyTrue,
];
const EMPTY_OR_CUSTOM_VAR: &[OptionsProfile] =
    &[OptionsProfile::Empty, OptionsProfile::CustomPropertyVar];
const ATRULE_MEDIA: &[OptionsProfile] = &[OptionsProfile::Empty, OptionsProfile::AtruleMedia];

const FULL_OBSERVATION: Option<UnsupportedPolicy> = Some(UnsupportedPolicy::FullObservation);
const PANIC_FREEDOM_ONLY: Option<UnsupportedPolicy> = Some(UnsupportedPolicy::PanicFreedomOnly);
const GENERIC_FRAGMENT: Option<UnsupportedReason> =
    Some(UnsupportedReason::GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor);

macro_rules! entry {
    ($path:literal, $context:literal, $entry:ident, $adapter:ident, $extractor:expr, $options:ident) => {
        RegistryEntry {
            fixture_path: $path,
            context: $context,
            entry_point: EntryPoint::$entry,
            adapter: Adapter::$adapter,
            extractor: $extractor,
            property_or_descriptor: None,
            options: $options,
            unsupported_policy: FULL_OBSERVATION,
            unsupported_reason: None,
        }
    };
}

macro_rules! adapterless_entry {
    ($path:literal, $context:literal, $options:ident) => {
        RegistryEntry {
            fixture_path: $path,
            context: $context,
            entry_point: EntryPoint::StyleAttribute,
            adapter: Adapter::CustomPropertyContainment,
            extractor: Extractor::DeclarationList,
            property_or_descriptor: None,
            options: $options,
            unsupported_policy: PANIC_FREEDOM_ONLY,
            unsupported_reason: GENERIC_FRAGMENT,
        }
    };
}

macro_rules! value_entry {
    ($path:literal, $adapter:ident, $extractor:expr, $owner:expr, $options:ident) => {
        RegistryEntry {
            fixture_path: $path,
            context: "value",
            entry_point: EntryPoint::StyleAttribute,
            adapter: Adapter::$adapter,
            extractor: $extractor,
            property_or_descriptor: $owner,
            options: $options,
            unsupported_policy: FULL_OBSERVATION,
            unsupported_reason: None,
        }
    };
}

macro_rules! adapterless_value_entry {
    ($path:literal, $options:ident) => {
        RegistryEntry {
            fixture_path: $path,
            context: "value",
            entry_point: EntryPoint::StyleAttribute,
            adapter: Adapter::CustomPropertyContainment,
            extractor: Extractor::DeclarationList,
            property_or_descriptor: None,
            options: $options,
            unsupported_policy: PANIC_FREEDOM_ONLY,
            unsupported_reason: GENERIC_FRAGMENT,
        }
    };
}

pub const REGISTRY: &[RegistryEntry] = &[
    entry!(
        "expectations/atrule/atrule/container.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::TopLevelRuleKind(TopLevelRuleKind::Container),
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/font-face.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::TopLevelRuleKind(TopLevelRuleKind::FontFace),
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/font-feature-values.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/import.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::TopLevelRuleKind(TopLevelRuleKind::Import),
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/layer.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/media.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::TopLevelRuleKind(TopLevelRuleKind::Media),
        EMPTY_OR_NO_ATRULE_PRELUDE
    ),
    entry!(
        "expectations/atrule/atrule/nest.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/scope.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::ScopeChildren,
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/starting-style.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/atrule/atrule/supports.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SupportsChildren,
        EMPTY
    ),
    entry!(
        "expectations/atrule/block.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY_OR_NO_ATRULE_PRELUDE
    ),
    entry!(
        "expectations/atrule/no-block.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY_OR_NO_ATRULE_PRELUDE
    ),
    entry!(
        "expectations/atrule/stylesheet.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/atrule/tolerant.json",
        "atrule",
        Sheet,
        TopLevelAtRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/atrulePrelude/index.json",
        "atrulePrelude",
        Sheet,
        MediaAtRulePrelude,
        Extractor::MediaQueries,
        ATRULE_MEDIA
    ),
    entry!(
        "expectations/block/Block.json",
        "block",
        Sheet,
        StyleBlock,
        Extractor::StyleDeclarations,
        EMPTY
    ),
    entry!(
        "expectations/declaration/Declaration.json",
        "declaration",
        Sheet,
        StyleDeclaration,
        Extractor::StyleDeclarations,
        EMPTY_OR_NO_VALUE
    ),
    entry!(
        "expectations/declaration/Important.json",
        "declaration",
        Sheet,
        StyleDeclaration,
        Extractor::StyleDeclarations,
        EMPTY
    ),
    entry!(
        "expectations/declaration/custom-property.json",
        "declaration",
        Sheet,
        StyleDeclaration,
        Extractor::StyleDeclarations,
        EMPTY_OR_CUSTOM
    ),
    entry!(
        "expectations/declaration/filter.json",
        "declaration",
        Sheet,
        StyleDeclaration,
        Extractor::StyleDeclarations,
        EMPTY
    ),
    entry!(
        "expectations/declarationList/DeclarationList.json",
        "declarationList",
        Sheet,
        StyleDeclarationList,
        Extractor::StyleDeclarations,
        EMPTY
    ),
    entry!(
        "expectations/declarationList/nesting.json",
        "declarationList",
        Sheet,
        StyleDeclarationList,
        Extractor::StyleDeclarations,
        EMPTY
    ),
    entry!(
        "expectations/declarationList/tolerant.json",
        "declarationList",
        Sheet,
        StyleDeclarationList,
        Extractor::StyleDeclarations,
        EMPTY
    ),
    entry!(
        "expectations/mediaQuery/FeatureRange.json",
        "mediaQuery",
        Sheet,
        MediaQuery,
        Extractor::MediaQueries,
        EMPTY
    ),
    entry!(
        "expectations/mediaQuery/GeneralEnclosed.json",
        "mediaQuery",
        Sheet,
        MediaQuery,
        Extractor::MediaQueries,
        EMPTY
    ),
    entry!(
        "expectations/mediaQuery/MediaQuery.json",
        "mediaQuery",
        Sheet,
        MediaQuery,
        Extractor::MediaQueries,
        EMPTY
    ),
    entry!(
        "expectations/mediaQuery/Ratio.json",
        "mediaQuery",
        Sheet,
        MediaQuery,
        Extractor::MediaQueries,
        EMPTY
    ),
    entry!(
        "expectations/rule/Rule.json",
        "rule",
        Sheet,
        TopLevelRule,
        Extractor::SheetRules,
        EMPTY_OR_NO_RULE_PRELUDE
    ),
    entry!(
        "expectations/rule/legacy.json",
        "rule",
        Sheet,
        TopLevelRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/rule/nested-atrule.json",
        "rule",
        Sheet,
        TopLevelRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/rule/nesting.json",
        "rule",
        Sheet,
        TopLevelRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/rule/tolerant.json",
        "rule",
        Sheet,
        TopLevelRule,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/selector/AttributeSelector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/ClassSelector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    adapterless_entry!("expectations/selector/Combinator.json", "selector", EMPTY),
    entry!(
        "expectations/selector/IdSelector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/Nth.json",
        "selector",
        Sheet,
        NthSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/PseudoClassSelector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/PseudoElementSelector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/Selector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/TypeSelector.json",
        "selector",
        Sheet,
        Selector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/-moz-any.json",
        "selector",
        Sheet,
        MozAnySelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/-webkit-any.json",
        "selector",
        Sheet,
        WebkitAnySelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/dir.json",
        "selector",
        Sheet,
        DirSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/has.json",
        "selector",
        Sheet,
        HasSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/host-context.json",
        "selector",
        Sheet,
        HostContextSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/host.json",
        "selector",
        Sheet,
        HostSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/is.json",
        "selector",
        Sheet,
        IsSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/lang.json",
        "selector",
        Sheet,
        LangSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/matches.json",
        "selector",
        Sheet,
        MatchesSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/not.json",
        "selector",
        Sheet,
        NotSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/slotted.json",
        "selector",
        Sheet,
        SlottedSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selector/functional-pseudo/where.json",
        "selector",
        Sheet,
        WhereSelector,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/selectorList/Selector.json",
        "selectorList",
        Sheet,
        SelectorList,
        Extractor::StyleSelector,
        EMPTY
    ),
    entry!(
        "expectations/stylesheet/StyleSheet.json",
        "stylesheet",
        Sheet,
        Stylesheet,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/stylesheet/comment.json",
        "stylesheet",
        Sheet,
        Stylesheet,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/stylesheet/errors.json",
        "stylesheet",
        Sheet,
        Stylesheet,
        Extractor::SheetRules,
        EMPTY
    ),
    entry!(
        "expectations/stylesheet/tolerant.json",
        "stylesheet",
        Sheet,
        Stylesheet,
        Extractor::SheetRules,
        EMPTY_OR_NO_ATRULE_PRELUDE
    ),
    adapterless_value_entry!("expectations/value/Brackets.json", EMPTY),
    value_entry!(
        "expectations/value/Dimension.json",
        PropertyValue,
        Extractor::KnownDeclaration {
            index: 0,
            property: CssKnownProperty::Width
        },
        Some(PropertyOrDescriptor::Property(CssKnownProperty::Width)),
        EMPTY
    ),
    adapterless_value_entry!("expectations/value/Function.json", EMPTY),
    value_entry!(
        "expectations/value/HexColor.json",
        PropertyValue,
        Extractor::KnownDeclaration {
            index: 0,
            property: CssKnownProperty::Color
        },
        Some(PropertyOrDescriptor::Property(CssKnownProperty::Color)),
        EMPTY
    ),
    adapterless_value_entry!("expectations/value/Identifier.json", EMPTY),
    adapterless_value_entry!("expectations/value/Number.json", EMPTY),
    adapterless_value_entry!("expectations/value/Parentheses.json", EMPTY),
    value_entry!(
        "expectations/value/Percentage.json",
        PropertyValue,
        Extractor::KnownDeclaration {
            index: 0,
            property: CssKnownProperty::Width
        },
        Some(PropertyOrDescriptor::Property(CssKnownProperty::Width)),
        EMPTY
    ),
    adapterless_value_entry!("expectations/value/String.json", EMPTY),
    RegistryEntry {
        fixture_path: "expectations/value/UnicodeRange.json",
        context: "value",
        entry_point: EntryPoint::Sheet,
        adapter: Adapter::FontFaceDescriptorValue,
        extractor: Extractor::FontFaceDescriptor(FontFaceDescriptorKind::UnicodeRange),
        property_or_descriptor: Some(PropertyOrDescriptor::FontFaceDescriptor(
            FontFaceDescriptorKind::UnicodeRange,
        )),
        options: EMPTY,
        unsupported_policy: FULL_OBSERVATION,
        unsupported_reason: None,
    },
    value_entry!(
        "expectations/value/Url.json",
        PropertyValue,
        Extractor::KnownDeclaration {
            index: 0,
            property: CssKnownProperty::BackgroundImage
        },
        Some(PropertyOrDescriptor::Property(
            CssKnownProperty::BackgroundImage
        )),
        EMPTY
    ),
    adapterless_value_entry!("expectations/value/Value.json", EMPTY_OR_CUSTOM_VAR),
    value_entry!(
        "expectations/value/function/calc.json",
        PropertyValue,
        Extractor::KnownDeclaration {
            index: 0,
            property: CssKnownProperty::Width
        },
        Some(PropertyOrDescriptor::Property(CssKnownProperty::Width)),
        EMPTY
    ),
    adapterless_value_entry!("expectations/value/function/element.json", EMPTY),
    adapterless_value_entry!("expectations/value/function/expression.json", EMPTY),
    value_entry!(
        "expectations/value/function/var.json",
        PropertyValue,
        Extractor::KnownDeclaration {
            index: 0,
            property: CssKnownProperty::Width
        },
        Some(PropertyOrDescriptor::Property(CssKnownProperty::Width)),
        EMPTY_OR_CUSTOM
    ),
];
