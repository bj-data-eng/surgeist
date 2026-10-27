//! Identity-only CSSOM objects. Live reads explicitly receive their owner;
//! owning captures read one immutable snapshot, including every chained object.

use crate::{model::State, *};
use surgeist_css::rule::CssRule as AuthoredRule;

/// A read-only typed object and its owning immutable snapshot.
///
/// Cloning shares the same snapshot. Child objects and conversions preserve
/// this capture rather than consulting the live owner. The handle alone retains
/// identity, while this capture also keeps the observed object data alive.
///
/// Keyframes has its own list interface and is not a grouping rule:
/// ```compile_fail
/// use surgeist_cssom::CssKeyframesRule;
/// fn invalid(rule: &CssKeyframesRule) { rule.as_grouping_rule(); }
/// ```
/// Object fields cannot be used for unchecked construction:
/// ```compile_fail
/// use surgeist_cssom::{CssRule, CssomRuleId};
/// fn invalid(id: CssomRuleId) { let _ = CssRule { id }; }
/// ```
#[derive(Clone, Debug)]
pub struct CssomCapture<T> {
    pub(crate) handle: T,
    pub(crate) snapshot: CssomSnapshot,
}
impl<T> CssomCapture<T> {
    /// The object's identity-only live handle. Reading it requires an explicit store.
    pub fn handle(&self) -> &T {
        &self.handle
    }
    /// Trusted product inspection of the same owning capture used by all reads.
    pub fn snapshot(&self) -> &CssomSnapshot {
        &self.snapshot
    }
}

#[derive(Clone, Copy)]
enum RuleClass {
    Any,
    Grouping,
    Condition,
    Style,
    Page,
    Margin,
    NestedDeclarations,
    Media,
    Supports,
    Container,
    Import,
    Namespace,
    Scope,
    LayerBlock,
    LayerStatement,
    SupportsCondition,
    Keyframes,
    Keyframe,
    FontFace,
    FontFeatureValues,
    FontPaletteValues,
    CounterStyle,
    CustomMedia,
    ColorProfile,
}
fn rule_matches(data: &CssomRuleData, class: RuleClass) -> bool {
    use CssomRuleData as D;
    use RuleClass as C;
    match class {
        C::Any => true,
        C::Grouping => matches!(
            data,
            D::Style { .. } | D::Page { .. } | D::Group { .. } | D::NamedSupports { .. }
        ),
        C::Condition => matches!(
            data,
            D::Group {
                prelude: CssomGroupPrelude::Media(_)
                    | CssomGroupPrelude::Supports(_)
                    | CssomGroupPrelude::Container(_),
                ..
            }
        ),
        C::Style => matches!(data, D::Style { .. }),
        C::Page => matches!(data, D::Page { .. }),
        C::Margin => matches!(data, D::Margin { .. }),
        C::NestedDeclarations => matches!(data, D::NestedDeclarations { .. }),
        C::Media => matches!(
            data,
            D::Group {
                prelude: CssomGroupPrelude::Media(_),
                ..
            }
        ),
        C::Supports => matches!(
            data,
            D::Group {
                prelude: CssomGroupPrelude::Supports(_),
                ..
            }
        ),
        C::Container => matches!(
            data,
            D::Group {
                prelude: CssomGroupPrelude::Container(_),
                ..
            }
        ),
        C::Import => matches!(data, D::Import { .. }),
        C::Namespace => matches!(
            data,
            D::Leaf {
                current: CssomAuthoredRule::Rule(AuthoredRule::Namespace(_))
            }
        ),
        C::Scope => matches!(
            data,
            D::Group {
                prelude: CssomGroupPrelude::Scope { .. },
                ..
            }
        ),
        C::LayerBlock => matches!(
            data,
            D::Group {
                prelude: CssomGroupPrelude::Layer(_),
                ..
            }
        ),
        C::LayerStatement => matches!(
            data,
            D::Leaf {
                current: CssomAuthoredRule::Rule(AuthoredRule::LayerStatement(_))
            }
        ),
        C::SupportsCondition => matches!(data, D::NamedSupports { .. }),
        C::Keyframes => matches!(data, D::Keyframes { .. }),
        C::Keyframe => matches!(data, D::Keyframe { .. }),
        C::FontFace => matches!(data, D::FontFace { .. }),
        C::FontFeatureValues => matches!(data, D::FontFeatureValues { .. }),
        C::CounterStyle => matches!(data, D::CounterStyle { .. }),
        C::CustomMedia => matches!(data, D::CustomMedia { .. }),
        C::FontPaletteValues => matches!(
            data,
            D::Leaf {
                current: CssomAuthoredRule::Rule(AuthoredRule::FontPaletteValues(_))
            }
        ),
        C::ColorProfile => matches!(
            data,
            D::Leaf {
                current: CssomAuthoredRule::Rule(AuthoredRule::ColorProfile(_))
            }
        ),
    }
}

// These macros emit concrete methods; no extensible public backend or hierarchy
// participates in dispatch. Each constructor is also usable against staged State
// by the private operation owner before publication.
macro_rules! identity_handle {
    ($name:ident, $id:ty, $factory:ident, $check:expr) => {
        /// An identity-only live CSSOM object. Reads require its explicit owner.
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name {
            id: $id,
        }
        impl $name {
            /// Trusted product identity inspection for host integration and provenance.
            /// This does not construct or validate an object in another state.
            pub fn id(&self) -> &$id {
                &self.id
            }
            pub(crate) fn from_state(state: &State, id: &$id) -> Result<Self, CssomError> {
                ($check)(state, id)?;
                Ok(Self { id: id.clone() })
            }
            /// Validate owner, presence and interface against this snapshot.
            pub fn capture(
                &self,
                snapshot: &CssomSnapshot,
            ) -> Result<CssomCapture<Self>, CssomError> {
                Ok(CssomCapture {
                    handle: Self::from_state(&snapshot.state, self.id())?,
                    snapshot: snapshot.clone(),
                })
            }
        }
        impl CssomStore {
            /// Recover a checked typed object from a trusted product identity.
            pub fn $factory(&self, id: &$id) -> Result<$name, CssomError> {
                $name::from_state(&self.state, id)
            }
        }
    };
}
macro_rules! rule_handle {
    ($name:ident, $factory:ident, $class:ident) => {
        identity_handle!(
            $name,
            CssomRuleId,
            $factory,
            |state: &State, id: &CssomRuleId| {
                if rule_matches(state.rule(id)?.data(), RuleClass::$class) {
                    Ok(())
                } else {
                    Err(CssomError::WrongKind)
                }
            }
        );
    };
}
rule_handle!(CssRule, css_rule, Any);
rule_handle!(CssGroupingRule, css_grouping_rule, Grouping);
rule_handle!(CssConditionRule, css_condition_rule, Condition);
rule_handle!(CssStyleRule, css_style_rule, Style);
rule_handle!(CssPageRule, css_page_rule, Page);
rule_handle!(CssMarginRule, css_margin_rule, Margin);
rule_handle!(
    CssNestedDeclarations,
    css_nested_declarations,
    NestedDeclarations
);
rule_handle!(CssMediaRule, css_media_rule, Media);
rule_handle!(CssSupportsRule, css_supports_rule, Supports);
rule_handle!(CssContainerRule, css_container_rule, Container);
rule_handle!(CssImportRule, css_import_rule, Import);
rule_handle!(CssNamespaceRule, css_namespace_rule, Namespace);
rule_handle!(CssScopeRule, css_scope_rule, Scope);
rule_handle!(CssLayerBlockRule, css_layer_block_rule, LayerBlock);
rule_handle!(
    CssLayerStatementRule,
    css_layer_statement_rule,
    LayerStatement
);
rule_handle!(
    CssSupportsConditionRule,
    css_supports_condition_rule,
    SupportsCondition
);
rule_handle!(CssKeyframesRule, css_keyframes_rule, Keyframes);
rule_handle!(CssKeyframeRule, css_keyframe_rule, Keyframe);
rule_handle!(CssFontFaceRule, css_font_face_rule, FontFace);
rule_handle!(
    CssFontFeatureValuesRule,
    css_font_feature_values_rule,
    FontFeatureValues
);
rule_handle!(
    CssFontPaletteValuesRule,
    css_font_palette_values_rule,
    FontPaletteValues
);
rule_handle!(CssCounterStyleRule, css_counter_style_rule, CounterStyle);
rule_handle!(CssCustomMediaRule, css_custom_media_rule, CustomMedia);
rule_handle!(CssColorProfileRule, css_color_profile_rule, ColorProfile);

fn present(state: &State, owner: &CssomOwnerId, exists: bool) -> Result<(), CssomError> {
    if owner != &state.owner {
        Err(CssomError::ForeignOwner)
    } else if !exists {
        Err(CssomError::MissingObject)
    } else {
        Ok(())
    }
}
identity_handle!(
    StyleSheet,
    CssomSheetId,
    style_sheet,
    |state: &State, id: &CssomSheetId| state.sheet(id).map(|_| ())
);
identity_handle!(
    CssStyleSheet,
    CssomSheetId,
    css_style_sheet,
    |state: &State, id: &CssomSheetId| state.sheet(id).map(|_| ())
);
identity_handle!(
    StyleSheetList,
    CssomSheetListId,
    style_sheet_list,
    |state: &State, id: &CssomSheetListId| present(state, id.owner(), id == &state.sheet_list)
);
identity_handle!(
    CssRuleList,
    CssomRuleListId,
    css_rule_list,
    |state: &State, id: &CssomRuleListId| present(state, id.owner(), state.lists.contains_key(id))
);
identity_handle!(
    MediaList,
    CssomMediaListId,
    media_list,
    |state: &State, id: &CssomMediaListId| state.media(id).map(|_| ())
);
identity_handle!(
    CssFontFeatureValuesMap,
    CssomFeatureMapId,
    css_font_feature_values_map,
    |state: &State, id: &CssomFeatureMapId| state.feature_map(id).map(|_| ())
);

#[derive(Clone, Copy)]
enum BlockClass {
    Declaration,
    Properties,
    Page,
    Margin,
    FontFace,
}
fn block_matches(data: &CssomBlockData, class: BlockClass) -> bool {
    use CssomBlockData as D;
    match class {
        BlockClass::Declaration => !matches!(data, D::CounterStyle(_)),
        BlockClass::Properties => matches!(
            data,
            D::Properties {
                domain: CssomPropertyDomain::Ordinary,
                ..
            }
        ),
        BlockClass::Page => matches!(data, D::Page { .. }),
        BlockClass::Margin => matches!(
            data,
            D::Properties {
                domain: CssomPropertyDomain::Margin,
                ..
            }
        ),
        BlockClass::FontFace => matches!(data, D::FontFace(_)),
    }
}
macro_rules! block_handle {
    ($name:ident, $factory:ident, $class:ident) => {
        identity_handle!(
            $name,
            CssomBlockId,
            $factory,
            |state: &State, id: &CssomBlockId| {
                if block_matches(state.block(id)?.data(), BlockClass::$class) {
                    Ok(())
                } else {
                    Err(CssomError::WrongKind)
                }
            }
        );
    };
}
block_handle!(CssStyleDeclaration, css_style_declaration, Declaration);
block_handle!(CssStyleProperties, css_style_properties, Properties);
block_handle!(CssPageDescriptors, css_page_descriptors, Page);
block_handle!(CssMarginDescriptors, css_margin_descriptors, Margin);
block_handle!(CssFontFaceDescriptors, css_font_face_descriptors, FontFace);

macro_rules! upcast {
    ($from:ident, $to:ident, $method:ident) => {
        impl $from {
            /// Infallible inherited-interface conversion retaining the same identity.
            pub fn $method(&self) -> $to {
                $to {
                    id: self.id.clone(),
                }
            }
        }
        impl CssomCapture<$from> {
            /// Inherited interface in the same owning snapshot.
            pub fn $method(&self) -> CssomCapture<$to> {
                CssomCapture {
                    handle: self.handle.$method(),
                    snapshot: self.snapshot.clone(),
                }
            }
        }
    };
}
upcast!(CssStyleSheet, StyleSheet, as_style_sheet);
upcast!(
    CssStyleProperties,
    CssStyleDeclaration,
    as_style_declaration
);
upcast!(
    CssPageDescriptors,
    CssStyleDeclaration,
    as_style_declaration
);
upcast!(
    CssMarginDescriptors,
    CssStyleDeclaration,
    as_style_declaration
);
upcast!(
    CssFontFaceDescriptors,
    CssStyleDeclaration,
    as_style_declaration
);

macro_rules! downcast {
    ($base:ident, $specific:ident, $method:ident) => {
        impl $base {
            /// Checked optional conversion. Foreign and missing identities remain errors.
            pub fn $method(&self, store: &CssomStore) -> Result<Option<$specific>, CssomError> {
                match $specific::from_state(&store.state, self.id()) {
                    Ok(value) => Ok(Some(value)),
                    Err(CssomError::WrongKind) => Ok(None),
                    Err(error) => Err(error),
                }
            }
        }
        impl CssomCapture<$base> {
            /// Checked optional conversion within this capture, never current state.
            pub fn $method(&self) -> Result<Option<CssomCapture<$specific>>, CssomError> {
                match $specific::from_state(&self.snapshot.state, self.handle.id()) {
                    Ok(handle) => Ok(Some(CssomCapture {
                        handle,
                        snapshot: self.snapshot.clone(),
                    })),
                    Err(CssomError::WrongKind) => Ok(None),
                    Err(error) => Err(error),
                }
            }
        }
    };
}
downcast!(StyleSheet, CssStyleSheet, as_css_style_sheet);
downcast!(CssStyleDeclaration, CssStyleProperties, as_style_properties);
downcast!(CssStyleDeclaration, CssPageDescriptors, as_page_descriptors);
downcast!(
    CssStyleDeclaration,
    CssMarginDescriptors,
    as_margin_descriptors
);
downcast!(
    CssStyleDeclaration,
    CssFontFaceDescriptors,
    as_font_face_descriptors
);

macro_rules! rule_interface {
    ($name:ident) => {
        impl $name {
            pub const STYLE_RULE: u16 = 1;
            pub const CHARSET_RULE: u16 = 2;
            pub const IMPORT_RULE: u16 = 3;
            pub const MEDIA_RULE: u16 = 4;
            pub const FONT_FACE_RULE: u16 = 5;
            pub const PAGE_RULE: u16 = 6;
            pub const KEYFRAMES_RULE: u16 = 7;
            pub const KEYFRAME_RULE: u16 = 8;
            pub const MARGIN_RULE: u16 = 9;
            pub const NAMESPACE_RULE: u16 = 10;
            pub const COUNTER_STYLE_RULE: u16 = 11;
            pub const SUPPORTS_RULE: u16 = 12;
            pub const FONT_FEATURE_VALUES_RULE: u16 = 14;

            /// Deprecated numeric CSSRule type, independent of interface capability.
            pub fn r#type(&self, store: &CssomStore) -> Result<u16, CssomError> {
                store.snapshot().rule_type(self.id())
            }
            /// Serialize current state through the native bounded graph owner.
            pub fn css_text(
                &self,
                store: &CssomStore,
                limits: &CssomRuleFormatLimits,
            ) -> Result<String, CssomError> {
                store.snapshot().rule_css_text(self.id(), limits)
            }
            pub fn parent_rule(&self, store: &CssomStore) -> Result<Option<CssRule>, CssomError> {
                rule_parent(&store.snapshot(), self.id())
            }
            pub fn parent_style_sheet(
                &self,
                store: &CssomStore,
            ) -> Result<Option<CssStyleSheet>, CssomError> {
                rule_sheet(&store.snapshot(), self.id())
            }
        }
        impl CssomCapture<$name> {
            pub fn r#type(&self) -> Result<u16, CssomError> {
                self.snapshot.rule_type(self.handle.id())
            }
            pub fn css_text(&self, limits: &CssomRuleFormatLimits) -> Result<String, CssomError> {
                self.snapshot.rule_css_text(self.handle.id(), limits)
            }
            pub fn parent_rule(&self) -> Result<Option<CssomCapture<CssRule>>, CssomError> {
                rule_parent(&self.snapshot, self.handle.id())?
                    .map(|handle| handle.capture(&self.snapshot))
                    .transpose()
            }
            pub fn parent_style_sheet(
                &self,
            ) -> Result<Option<CssomCapture<CssStyleSheet>>, CssomError> {
                rule_sheet(&self.snapshot, self.handle.id())?
                    .map(|handle| handle.capture(&self.snapshot))
                    .transpose()
            }
        }
    };
}
fn rule_parent(snapshot: &CssomSnapshot, id: &CssomRuleId) -> Result<Option<CssRule>, CssomError> {
    snapshot
        .parent_rule(id)?
        .map(|id| CssRule::from_state(&snapshot.state, id))
        .transpose()
}
fn rule_sheet(
    snapshot: &CssomSnapshot,
    id: &CssomRuleId,
) -> Result<Option<CssStyleSheet>, CssomError> {
    snapshot
        .parent_style_sheet(id)?
        .map(|id| CssStyleSheet::from_state(&snapshot.state, id))
        .transpose()
}
// Concrete inherited wrappers in family owners share this closed inventory.
macro_rules! for_rule_handles {
    ($emit:ident) => {
        $emit!(CssRule);
        $emit!(CssGroupingRule);
        $emit!(CssConditionRule);
        $emit!(CssStyleRule);
        $emit!(CssPageRule);
        $emit!(CssMarginRule);
        $emit!(CssNestedDeclarations);
        $emit!(CssMediaRule);
        $emit!(CssSupportsRule);
        $emit!(CssContainerRule);
        $emit!(CssImportRule);
        $emit!(CssNamespaceRule);
        $emit!(CssScopeRule);
        $emit!(CssLayerBlockRule);
        $emit!(CssLayerStatementRule);
        $emit!(CssSupportsConditionRule);
        $emit!(CssKeyframesRule);
        $emit!(CssKeyframeRule);
        $emit!(CssFontFaceRule);
        $emit!(CssFontFeatureValuesRule);
        $emit!(CssFontPaletteValuesRule);
        $emit!(CssCounterStyleRule);
        $emit!(CssCustomMediaRule);
        $emit!(CssColorProfileRule);
    };
}
pub(crate) use for_rule_handles;
self::for_rule_handles!(rule_interface);
macro_rules! concrete_rule {
    ($name:ident, $method:ident) => {
        upcast!($name, CssRule, as_rule);
        downcast!(CssRule, $name, $method);
    };
}
concrete_rule!(CssGroupingRule, as_grouping_rule);
concrete_rule!(CssConditionRule, as_condition_rule);
concrete_rule!(CssStyleRule, as_style_rule);
concrete_rule!(CssPageRule, as_page_rule);
concrete_rule!(CssMarginRule, as_margin_rule);
concrete_rule!(CssNestedDeclarations, as_nested_declarations);
concrete_rule!(CssMediaRule, as_media_rule);
concrete_rule!(CssSupportsRule, as_supports_rule);
concrete_rule!(CssContainerRule, as_container_rule);
concrete_rule!(CssImportRule, as_import_rule);
concrete_rule!(CssNamespaceRule, as_namespace_rule);
concrete_rule!(CssScopeRule, as_scope_rule);
concrete_rule!(CssLayerBlockRule, as_layer_block_rule);
concrete_rule!(CssLayerStatementRule, as_layer_statement_rule);
concrete_rule!(CssSupportsConditionRule, as_supports_condition_rule);
concrete_rule!(CssKeyframesRule, as_keyframes_rule);
concrete_rule!(CssKeyframeRule, as_keyframe_rule);
concrete_rule!(CssFontFaceRule, as_font_face_rule);
concrete_rule!(CssFontFeatureValuesRule, as_font_feature_values_rule);
concrete_rule!(CssFontPaletteValuesRule, as_font_palette_values_rule);
concrete_rule!(CssCounterStyleRule, as_counter_style_rule);
concrete_rule!(CssCustomMediaRule, as_custom_media_rule);
concrete_rule!(CssColorProfileRule, as_color_profile_rule);

upcast!(CssConditionRule, CssGroupingRule, as_grouping_rule);
upcast!(CssStyleRule, CssGroupingRule, as_grouping_rule);
upcast!(CssPageRule, CssGroupingRule, as_grouping_rule);
upcast!(CssMediaRule, CssGroupingRule, as_grouping_rule);
upcast!(CssMediaRule, CssConditionRule, as_condition_rule);
upcast!(CssSupportsRule, CssGroupingRule, as_grouping_rule);
upcast!(CssSupportsRule, CssConditionRule, as_condition_rule);
upcast!(CssContainerRule, CssGroupingRule, as_grouping_rule);
upcast!(CssContainerRule, CssConditionRule, as_condition_rule);
upcast!(CssScopeRule, CssGroupingRule, as_grouping_rule);
upcast!(CssLayerBlockRule, CssGroupingRule, as_grouping_rule);
upcast!(CssSupportsConditionRule, CssGroupingRule, as_grouping_rule);
