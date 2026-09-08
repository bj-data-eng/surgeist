use std::{
    collections::{BTreeMap, BTreeSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use super::{
    AlignContent, AnimationDirectionList, AnimationFillModeList, AnimationIterationCountList,
    AnimationNameList, AnimationPlayStateList, AspectRatio, BorderLineStyle, BorderRadii,
    BoxDecorationBreak, ClipPath, Condition, ConditionFacts, ContainerFacts, ContentVisibility,
    CornerRadius, Corners, CssWideKeyword, Cursor, Declarations, Display, Edges, Filter,
    FlexFactor, FontFamilyList, FontFeatureSettings, FontStretch, FontVariant, FontWeight,
    LayoutPosition, Length, LetterSpacing, ListStyleImage, ListStylePosition, ListStyleType,
    MediaEnvironment, Order, OutlineStyle, OutlineWidth, OverflowWrap, PointerEvents, Property,
    QueryLength, QueryLengthBasis, Ratio, Resolution, Result, Rotate, RulePrecedence, RuleScope,
    Scale, ScrollbarWidth, SelectorMatchContext, Sheet, Size, StyleBucket, StyleColor,
    TextAlignLast, TextDecorationLine, TextDecorationStyle, TextDecorationThickness, TextIndent,
    TextOverflow, TextSlant, TextTransform, TextWrap, TimeList, Transform, TransitionPropertyList,
    Translate, Traversal, Tree, UserSelect, Value, Version, VerticalAlign, Visibility, WhiteSpace,
    WordBreak, ZIndex,
    declaration::hash_value,
    value::{
        BackgroundAttachmentList, BackgroundBox, BackgroundRepeatList, BackgroundSizeList, Content,
        CounterChanges, EasingList, ImageLayerList, PositionList,
    },
};
use crate::{
    CustomPropertyDependencies, CustomPropertyName, CustomPropertyResolution, CustomPropertyValue,
    InvalidAtComputedValueReason, StyleDiagnostic, StyleDiagnosticSubject, StyleSourceId,
    VariableDependentValue, VariableExpression,
    authored::{AuthoredCascadeValue, CustomPropertyCascadeValue},
    sheet::{RuleDeclarationOrigin, RuleDeclarationValue},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    values: BTreeMap<Property, Value>,
    custom_properties: BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    custom_property_dependencies: CustomPropertyDependencies,
}

impl Resolved {
    #[must_use]
    pub fn new() -> Self {
        let mut values = BTreeMap::new();
        for property in Property::ALL {
            if property.is_canonical() {
                values.insert(*property, property.metadata().default().clone());
            }
        }
        Self {
            values,
            custom_properties: BTreeMap::new(),
            custom_property_dependencies: CustomPropertyDependencies::default(),
        }
    }

    #[must_use]
    pub fn get(&self, property: Property) -> &Value {
        self.values
            .get(&property)
            .expect("resolved style stores every canonical property")
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Property, &Value)> {
        self.values.iter()
    }

    #[must_use]
    pub fn custom_property(&self, name: &CustomPropertyName) -> Option<&CustomPropertyValue> {
        self.custom_properties
            .get(name)
            .and_then(CustomPropertyResolution::value)
    }

    #[must_use]
    pub fn custom_property_resolution(
        &self,
        name: &CustomPropertyName,
    ) -> Option<&CustomPropertyResolution> {
        self.custom_properties.get(name)
    }

    #[must_use]
    pub const fn custom_property_dependencies(&self) -> &CustomPropertyDependencies {
        &self.custom_property_dependencies
    }

    #[must_use]
    pub fn background(&self) -> &StyleColor {
        match self.get(Property::Background) {
            Value::StyleColor(color) => color,
            _ => unreachable!("resolved background stores a style color"),
        }
    }

    #[must_use]
    pub fn background_image(&self) -> &ImageLayerList {
        match self.get(Property::BackgroundImage) {
            Value::ImageLayerList(value) => value,
            _ => unreachable!("resolved background-image stores an image layer list"),
        }
    }

    #[must_use]
    pub fn background_position(&self) -> &PositionList {
        match self.get(Property::BackgroundPosition) {
            Value::PositionList(value) => value,
            _ => unreachable!("resolved background-position stores a position list"),
        }
    }

    #[must_use]
    pub fn background_size(&self) -> &BackgroundSizeList {
        match self.get(Property::BackgroundSize) {
            Value::BackgroundSizeList(value) => value,
            _ => unreachable!("resolved background-size stores a background size list"),
        }
    }

    #[must_use]
    pub fn background_repeat(&self) -> &BackgroundRepeatList {
        match self.get(Property::BackgroundRepeat) {
            Value::BackgroundRepeatList(value) => value,
            _ => unreachable!("resolved background-repeat stores a background repeat list"),
        }
    }

    #[must_use]
    pub fn background_origin(&self) -> BackgroundBox {
        match self.get(Property::BackgroundOrigin) {
            Value::BackgroundBox(value) => *value,
            _ => BackgroundBox::PaddingBox,
        }
    }

    #[must_use]
    pub fn background_clip(&self) -> BackgroundBox {
        match self.get(Property::BackgroundClip) {
            Value::BackgroundBox(value) => *value,
            _ => BackgroundBox::BorderBox,
        }
    }

    #[must_use]
    pub fn background_attachment(&self) -> &BackgroundAttachmentList {
        match self.get(Property::BackgroundAttachment) {
            Value::BackgroundAttachmentList(value) => value,
            _ => unreachable!("resolved background-attachment stores an attachment list"),
        }
    }

    #[must_use]
    pub fn mask_image(&self) -> &ImageLayerList {
        match self.get(Property::MaskImage) {
            Value::ImageLayerList(value) => value,
            _ => unreachable!("resolved mask-image stores an image layer list"),
        }
    }

    #[must_use]
    pub fn mask_position(&self) -> &PositionList {
        match self.get(Property::MaskPosition) {
            Value::PositionList(value) => value,
            _ => unreachable!("resolved mask-position stores a position list"),
        }
    }

    #[must_use]
    pub fn mask_size(&self) -> &BackgroundSizeList {
        match self.get(Property::MaskSize) {
            Value::BackgroundSizeList(value) => value,
            _ => unreachable!("resolved mask-size stores a background size list"),
        }
    }

    #[must_use]
    pub fn mask_repeat(&self) -> &BackgroundRepeatList {
        match self.get(Property::MaskRepeat) {
            Value::BackgroundRepeatList(value) => value,
            _ => unreachable!("resolved mask-repeat stores a background repeat list"),
        }
    }

    #[must_use]
    pub fn text_color(&self) -> &StyleColor {
        match self.get(Property::Color) {
            Value::StyleColor(color) => color,
            _ => unreachable!("resolved color stores a style color"),
        }
    }

    #[must_use]
    pub fn width(&self) -> Length {
        match self.get(Property::Width) {
            Value::Length(value) => value.clone(),
            _ => Length::Auto,
        }
    }

    #[must_use]
    pub fn height(&self) -> Length {
        match self.get(Property::Height) {
            Value::Length(value) => value.clone(),
            _ => Length::Auto,
        }
    }

    #[must_use]
    pub fn padding_edges(&self) -> Edges {
        Edges::new(
            self.length_or(Property::PaddingTop, Length::ZERO),
            self.length_or(Property::PaddingRight, Length::ZERO),
            self.length_or(Property::PaddingBottom, Length::ZERO),
            self.length_or(Property::PaddingLeft, Length::ZERO),
        )
    }

    #[must_use]
    pub fn margin_edges(&self) -> Edges {
        Edges::new(
            self.length_or(Property::MarginTop, Length::ZERO),
            self.length_or(Property::MarginRight, Length::ZERO),
            self.length_or(Property::MarginBottom, Length::ZERO),
            self.length_or(Property::MarginLeft, Length::ZERO),
        )
    }

    #[must_use]
    pub fn inset_edges(&self) -> Edges {
        Edges::new(
            self.length_or(Property::Top, Length::Auto),
            self.length_or(Property::Right, Length::Auto),
            self.length_or(Property::Bottom, Length::Auto),
            self.length_or(Property::Left, Length::Auto),
        )
    }

    #[must_use]
    pub fn radius_corners(&self) -> Corners {
        let radii = self.border_radii();
        Corners::new(
            radii.top_left().horizontal().clone(),
            radii.top_right().horizontal().clone(),
            radii.bottom_right().horizontal().clone(),
            radii.bottom_left().horizontal().clone(),
        )
    }

    #[must_use]
    pub fn border_radii(&self) -> BorderRadii {
        BorderRadii::new(
            self.border_top_left_radius().clone(),
            self.border_top_right_radius().clone(),
            self.border_bottom_right_radius().clone(),
            self.border_bottom_left_radius().clone(),
        )
    }

    #[must_use]
    pub fn border_top_left_radius(&self) -> &CornerRadius {
        self.corner_radius_or(Property::BorderTopLeftRadius)
    }

    #[must_use]
    pub fn border_top_right_radius(&self) -> &CornerRadius {
        self.corner_radius_or(Property::BorderTopRightRadius)
    }

    #[must_use]
    pub fn border_bottom_right_radius(&self) -> &CornerRadius {
        self.corner_radius_or(Property::BorderBottomRightRadius)
    }

    #[must_use]
    pub fn border_bottom_left_radius(&self) -> &CornerRadius {
        self.corner_radius_or(Property::BorderBottomLeftRadius)
    }

    #[must_use]
    pub fn opacity(&self) -> f32 {
        match self.get(Property::Opacity) {
            Value::Number(value) => *value,
            _ => 1.0,
        }
    }

    #[must_use]
    pub fn font_size(&self) -> Length {
        match self.get(Property::FontSize) {
            Value::Length(value) => value.clone(),
            _ => Length::Px(16.0),
        }
    }

    #[must_use]
    pub fn font_family(&self) -> &FontFamilyList {
        match self.get(Property::FontFamily) {
            Value::FontFamilyList(family) => family,
            _ => unreachable!("resolved font-family stores a font family list"),
        }
    }

    #[must_use]
    pub fn line_height(&self) -> Length {
        match self.get(Property::LineHeight) {
            Value::Length(value) => value.clone(),
            _ => Length::Px(16.0),
        }
    }

    #[must_use]
    pub fn font_weight(&self) -> FontWeight {
        match self.get(Property::FontWeight) {
            Value::FontWeight(value) => *value,
            _ => FontWeight::default(),
        }
    }

    #[must_use]
    pub fn font_style(&self) -> TextSlant {
        match self.get(Property::FontStyle) {
            Value::TextSlant(value) => *value,
            _ => TextSlant::default(),
        }
    }

    #[must_use]
    pub fn font_stretch(&self) -> FontStretch {
        match self.get(Property::FontStretch) {
            Value::FontStretch(value) => *value,
            _ => FontStretch::default(),
        }
    }

    #[must_use]
    pub fn font_variant(&self) -> FontVariant {
        match self.get(Property::FontVariant) {
            Value::FontVariant(value) => *value,
            _ => FontVariant::default(),
        }
    }

    #[must_use]
    pub fn font_feature_settings(&self) -> &FontFeatureSettings {
        match self.get(Property::FontFeatureSettings) {
            Value::FontFeatureSettings(value) => value,
            _ => unreachable!("resolved font-feature-settings stores feature settings"),
        }
    }

    #[must_use]
    pub fn text_align_last(&self) -> TextAlignLast {
        match self.get(Property::TextAlignLast) {
            Value::TextAlignLast(value) => *value,
            _ => TextAlignLast::default(),
        }
    }

    #[must_use]
    pub fn text_indent(&self) -> TextIndent {
        match self.get(Property::TextIndent) {
            Value::TextIndent(value) => value.clone(),
            _ => TextIndent::default(),
        }
    }

    #[must_use]
    pub fn vertical_align(&self) -> VerticalAlign {
        match self.get(Property::VerticalAlign) {
            Value::VerticalAlign(value) => value.clone(),
            _ => VerticalAlign::default(),
        }
    }

    #[must_use]
    pub fn letter_spacing(&self) -> LetterSpacing {
        match self.get(Property::LetterSpacing) {
            Value::LetterSpacing(value) => value.clone(),
            _ => LetterSpacing::default(),
        }
    }

    #[must_use]
    pub fn text_transform(&self) -> TextTransform {
        match self.get(Property::TextTransform) {
            Value::TextTransform(value) => *value,
            _ => TextTransform::default(),
        }
    }

    #[must_use]
    pub fn text_wrap(&self) -> TextWrap {
        match self.get(Property::TextWrap) {
            Value::TextWrap(value) => *value,
            _ => TextWrap::default(),
        }
    }

    #[must_use]
    pub fn white_space(&self) -> WhiteSpace {
        match self.get(Property::WhiteSpace) {
            Value::WhiteSpace(value) => *value,
            _ => WhiteSpace::default(),
        }
    }

    #[must_use]
    pub fn word_break(&self) -> WordBreak {
        match self.get(Property::WordBreak) {
            Value::WordBreak(value) => *value,
            _ => WordBreak::default(),
        }
    }

    #[must_use]
    pub fn overflow_wrap(&self) -> OverflowWrap {
        match self.get(Property::OverflowWrap) {
            Value::OverflowWrap(value) => *value,
            _ => OverflowWrap::default(),
        }
    }

    #[must_use]
    pub fn text_overflow(&self) -> TextOverflow {
        match self.get(Property::TextOverflow) {
            Value::TextOverflow(value) => *value,
            _ => TextOverflow::default(),
        }
    }

    #[must_use]
    pub fn text_decoration_line(&self) -> &TextDecorationLine {
        match self.get(Property::TextDecorationLine) {
            Value::TextDecorationLine(value) => value,
            _ => unreachable!("resolved text-decoration-line stores a text decoration line"),
        }
    }

    #[must_use]
    pub fn text_decoration_style(&self) -> TextDecorationStyle {
        match self.get(Property::TextDecorationStyle) {
            Value::TextDecorationStyle(value) => *value,
            _ => TextDecorationStyle::default(),
        }
    }

    #[must_use]
    pub fn text_decoration_thickness(&self) -> &TextDecorationThickness {
        match self.get(Property::TextDecorationThickness) {
            Value::TextDecorationThickness(value) => value,
            _ => {
                unreachable!(
                    "resolved text-decoration-thickness stores a text decoration thickness"
                )
            }
        }
    }

    #[must_use]
    pub fn text_decoration_color(&self) -> &StyleColor {
        match self.get(Property::TextDecorationColor) {
            Value::StyleColor(value) => value,
            _ => unreachable!("resolved text-decoration-color stores a style color"),
        }
    }

    #[must_use]
    pub fn cursor(&self) -> Cursor {
        match self.get(Property::Cursor) {
            Value::Cursor(cursor) => *cursor,
            _ => Cursor::Default,
        }
    }

    #[must_use]
    pub fn pointer_events(&self) -> PointerEvents {
        match self.get(Property::PointerEvents) {
            Value::PointerEvents(pointer_events) => *pointer_events,
            _ => PointerEvents::Auto,
        }
    }

    #[must_use]
    pub fn user_select(&self) -> UserSelect {
        match self.get(Property::UserSelect) {
            Value::UserSelect(value) => *value,
            _ => UserSelect::default(),
        }
    }

    #[must_use]
    pub fn border_width_edges(&self) -> Edges {
        Edges::new(
            self.length_or(Property::BorderTopWidth, Length::Px(3.0)),
            self.length_or(Property::BorderRightWidth, Length::Px(3.0)),
            self.length_or(Property::BorderBottomWidth, Length::Px(3.0)),
            self.length_or(Property::BorderLeftWidth, Length::Px(3.0)),
        )
    }

    #[must_use]
    pub fn border_top_color(&self) -> &StyleColor {
        self.style_color_or(Property::BorderTopColor)
    }

    #[must_use]
    pub fn border_right_color(&self) -> &StyleColor {
        self.style_color_or(Property::BorderRightColor)
    }

    #[must_use]
    pub fn border_bottom_color(&self) -> &StyleColor {
        self.style_color_or(Property::BorderBottomColor)
    }

    #[must_use]
    pub fn border_left_color(&self) -> &StyleColor {
        self.style_color_or(Property::BorderLeftColor)
    }

    #[must_use]
    pub fn border_top_style(&self) -> BorderLineStyle {
        self.border_line_style_or(Property::BorderTopStyle)
    }

    #[must_use]
    pub fn border_right_style(&self) -> BorderLineStyle {
        self.border_line_style_or(Property::BorderRightStyle)
    }

    #[must_use]
    pub fn border_bottom_style(&self) -> BorderLineStyle {
        self.border_line_style_or(Property::BorderBottomStyle)
    }

    #[must_use]
    pub fn border_left_style(&self) -> BorderLineStyle {
        self.border_line_style_or(Property::BorderLeftStyle)
    }

    #[must_use]
    pub fn outline_width(&self) -> &OutlineWidth {
        match self.get(Property::OutlineWidth) {
            Value::OutlineWidth(value) => value,
            _ => unreachable!("resolved outline-width stores an outline width"),
        }
    }

    #[must_use]
    pub fn outline_style(&self) -> OutlineStyle {
        match self.get(Property::OutlineStyle) {
            Value::OutlineStyle(value) => *value,
            _ => OutlineStyle::Border(BorderLineStyle::None),
        }
    }

    #[must_use]
    pub fn outline_color(&self) -> &StyleColor {
        self.style_color_or(Property::OutlineColor)
    }

    #[must_use]
    pub fn visibility(&self) -> Visibility {
        match self.get(Property::Visibility) {
            Value::Visibility(visibility) => *visibility,
            _ => Visibility::Visible,
        }
    }

    #[must_use]
    pub fn transform(&self) -> &Transform {
        match self.get(Property::Transform) {
            Value::Transform(transform) => transform,
            _ => unreachable!("resolved transform stores a transform value"),
        }
    }

    #[must_use]
    pub fn transform_origin(&self) -> Size {
        match self.get(Property::TransformOrigin) {
            Value::Size(origin) => origin.clone(),
            _ => Size::new(Length::Percent(50.0), Length::Percent(50.0)),
        }
    }

    #[must_use]
    pub fn box_decoration_break(&self) -> BoxDecorationBreak {
        match self.get(Property::BoxDecorationBreak) {
            Value::BoxDecorationBreak(value) => *value,
            _ => BoxDecorationBreak::Slice,
        }
    }

    #[must_use]
    pub fn filter(&self) -> &Filter {
        match self.get(Property::Filter) {
            Value::Filter(filter) => filter,
            _ => unreachable!("resolved filter stores a filter value"),
        }
    }

    #[must_use]
    pub fn backdrop_filter(&self) -> &Filter {
        match self.get(Property::BackdropFilter) {
            Value::Filter(filter) => filter,
            _ => unreachable!("resolved backdrop-filter stores a filter value"),
        }
    }

    #[must_use]
    pub fn clip_path(&self) -> &ClipPath {
        match self.get(Property::ClipPath) {
            Value::ClipPath(clip_path) => clip_path,
            _ => unreachable!("resolved clip-path stores a clip path value"),
        }
    }

    #[must_use]
    pub fn translate(&self) -> &Translate {
        match self.get(Property::Translate) {
            Value::Translate(translate) => translate,
            _ => unreachable!("resolved translate stores a translate value"),
        }
    }

    #[must_use]
    pub fn rotate(&self) -> &Rotate {
        match self.get(Property::Rotate) {
            Value::Rotate(rotate) => rotate,
            _ => unreachable!("resolved rotate stores a rotate value"),
        }
    }

    #[must_use]
    pub fn scale(&self) -> &Scale {
        match self.get(Property::Scale) {
            Value::Scale(scale) => scale,
            _ => unreachable!("resolved scale stores a scale value"),
        }
    }

    #[must_use]
    pub fn transition_properties(&self) -> &TransitionPropertyList {
        match self.get(Property::TransitionProperty) {
            Value::TransitionPropertyList(properties) => properties,
            _ => unreachable!("resolved transition-property stores transition property list"),
        }
    }

    #[must_use]
    pub fn transition_duration(&self) -> &TimeList {
        match self.get(Property::TransitionDuration) {
            Value::TimeList(durations) => durations,
            _ => unreachable!("resolved transition-duration stores time list"),
        }
    }

    #[must_use]
    pub fn transition_delay(&self) -> &TimeList {
        match self.get(Property::TransitionDelay) {
            Value::TimeList(delays) => delays,
            _ => unreachable!("resolved transition-delay stores time list"),
        }
    }

    #[must_use]
    pub fn transition_timing_function(&self) -> &EasingList {
        match self.get(Property::TransitionTimingFunction) {
            Value::EasingList(easings) => easings,
            _ => unreachable!("resolved transition-timing-function stores easing list"),
        }
    }

    #[must_use]
    pub fn animation_name(&self) -> &AnimationNameList {
        match self.get(Property::AnimationName) {
            Value::AnimationNameList(names) => names,
            _ => unreachable!("resolved animation-name stores animation name list"),
        }
    }

    #[must_use]
    pub fn animation_duration(&self) -> &TimeList {
        match self.get(Property::AnimationDuration) {
            Value::TimeList(durations) => durations,
            _ => unreachable!("resolved animation-duration stores time list"),
        }
    }

    #[must_use]
    pub fn animation_delay(&self) -> &TimeList {
        match self.get(Property::AnimationDelay) {
            Value::TimeList(delays) => delays,
            _ => unreachable!("resolved animation-delay stores time list"),
        }
    }

    #[must_use]
    pub fn animation_timing_function(&self) -> &EasingList {
        match self.get(Property::AnimationTimingFunction) {
            Value::EasingList(easings) => easings,
            _ => unreachable!("resolved animation-timing-function stores easing list"),
        }
    }

    #[must_use]
    pub fn animation_iteration_count(&self) -> &AnimationIterationCountList {
        match self.get(Property::AnimationIterationCount) {
            Value::AnimationIterationCountList(values) => values,
            _ => {
                unreachable!("resolved animation-iteration-count stores animation count list")
            }
        }
    }

    #[must_use]
    pub fn animation_direction(&self) -> &AnimationDirectionList {
        match self.get(Property::AnimationDirection) {
            Value::AnimationDirectionList(values) => values,
            _ => unreachable!("resolved animation-direction stores animation direction list"),
        }
    }

    #[must_use]
    pub fn animation_fill_mode(&self) -> &AnimationFillModeList {
        match self.get(Property::AnimationFillMode) {
            Value::AnimationFillModeList(values) => values,
            _ => unreachable!("resolved animation-fill-mode stores animation fill mode list"),
        }
    }

    #[must_use]
    pub fn animation_play_state(&self) -> &AnimationPlayStateList {
        match self.get(Property::AnimationPlayState) {
            Value::AnimationPlayStateList(values) => values,
            _ => unreachable!("resolved animation-play-state stores animation play state list"),
        }
    }

    #[must_use]
    pub fn display(&self) -> Display {
        match self.get(Property::Display) {
            Value::Display(display) => *display,
            _ => Display::default(),
        }
    }

    #[must_use]
    pub fn position(&self) -> LayoutPosition {
        match self.get(Property::Position) {
            Value::Position(value) => *value,
            _ => LayoutPosition::default(),
        }
    }

    #[must_use]
    pub fn z_index(&self) -> ZIndex {
        match self.get(Property::ZIndex) {
            Value::ZIndex(value) => *value,
            _ => ZIndex::default(),
        }
    }

    #[must_use]
    pub fn scrollbar_width(&self) -> ScrollbarWidth {
        match self.get(Property::ScrollbarWidth) {
            Value::ScrollbarWidth(value) => *value,
            _ => ScrollbarWidth::default(),
        }
    }

    #[must_use]
    pub fn content_visibility(&self) -> ContentVisibility {
        match self.get(Property::ContentVisibility) {
            Value::ContentVisibility(value) => *value,
            _ => ContentVisibility::default(),
        }
    }

    #[must_use]
    pub fn content(&self) -> &Content {
        match self.get(Property::Content) {
            Value::Content(value) => value,
            _ => unreachable!("resolved content stores generated content"),
        }
    }

    #[must_use]
    pub fn list_style_type(&self) -> &ListStyleType {
        match self.get(Property::ListStyleType) {
            Value::ListStyleType(value) => value,
            _ => unreachable!("resolved list-style-type stores marker type"),
        }
    }

    #[must_use]
    pub fn list_style_position(&self) -> ListStylePosition {
        match self.get(Property::ListStylePosition) {
            Value::ListStylePosition(value) => *value,
            _ => ListStylePosition::Outside,
        }
    }

    #[must_use]
    pub fn list_style_image(&self) -> &ListStyleImage {
        match self.get(Property::ListStyleImage) {
            Value::ListStyleImage(value) => value,
            _ => unreachable!("resolved list-style-image stores marker image"),
        }
    }

    #[must_use]
    pub fn counter_reset(&self) -> &CounterChanges {
        self.counter_changes(Property::CounterReset)
    }

    #[must_use]
    pub fn counter_increment(&self) -> &CounterChanges {
        self.counter_changes(Property::CounterIncrement)
    }

    #[must_use]
    pub fn counter_set(&self) -> &CounterChanges {
        self.counter_changes(Property::CounterSet)
    }

    #[must_use]
    pub fn order(&self) -> Order {
        match self.get(Property::Order) {
            Value::Order(value) => *value,
            _ => Order::default(),
        }
    }

    #[must_use]
    pub fn flex_grow(&self) -> FlexFactor {
        match self.get(Property::FlexGrow) {
            Value::FlexFactor(value) => *value,
            _ => FlexFactor::zero(),
        }
    }

    #[must_use]
    pub fn flex_shrink(&self) -> FlexFactor {
        match self.get(Property::FlexShrink) {
            Value::FlexFactor(value) => *value,
            _ => FlexFactor::one(),
        }
    }

    #[must_use]
    pub fn align_tracks(&self) -> AlignContent {
        match self.get(Property::AlignTracks) {
            Value::AlignContent(value) => *value,
            _ => AlignContent::default(),
        }
    }

    #[must_use]
    pub fn justify_tracks(&self) -> AlignContent {
        match self.get(Property::JustifyTracks) {
            Value::AlignContent(value) => *value,
            _ => AlignContent::default(),
        }
    }

    #[must_use]
    pub fn aspect_ratio(&self) -> AspectRatio {
        match self.get(Property::AspectRatio) {
            Value::AspectRatio(value) => *value,
            _ => AspectRatio::default(),
        }
    }

    fn length_or(&self, property: Property, fallback: Length) -> Length {
        match self.get(property) {
            Value::Length(value) => value.clone(),
            _ => fallback,
        }
    }

    fn style_color_or(&self, property: Property) -> &StyleColor {
        match self.get(property) {
            Value::StyleColor(value) => value,
            _ => unreachable!("resolved style color property stores a style color"),
        }
    }

    fn border_line_style_or(&self, property: Property) -> BorderLineStyle {
        match self.get(property) {
            Value::BorderLineStyle(value) => *value,
            _ => BorderLineStyle::None,
        }
    }

    fn corner_radius_or(&self, property: Property) -> &CornerRadius {
        match self.get(property) {
            Value::CornerRadius(value) => value,
            _ => unreachable!("resolved corner radius property stores a corner radius"),
        }
    }

    fn counter_changes(&self, property: Property) -> &CounterChanges {
        match self.get(property) {
            Value::CounterChanges(value) => value,
            _ => unreachable!("resolved counter property stores counter changes"),
        }
    }

    fn inherit_from(&mut self, parent: &Self) {
        for property in Property::ALL {
            if property.is_canonical() && property.metadata().is_inherited() {
                self.values.insert(*property, parent.get(*property).clone());
            }
        }
        self.custom_properties = parent.custom_properties.clone();
    }

    fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for (property, value) in &self.values {
            property.hash(&mut hasher);
            hash_value(value, &mut hasher);
        }
        for (name, resolution) in &self.custom_properties {
            name.hash(&mut hasher);
            hash_custom_property_resolution(resolution, &mut hasher);
        }
        for property in Property::ALL {
            for name in self.custom_property_dependencies.for_property(*property) {
                property.hash(&mut hasher);
                name.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    fn apply(&mut self, declarations: &Declarations, parent: Option<&Self>) -> Result<()> {
        for declaration in declarations.iter() {
            declaration.property.validate_value(&declaration.value)?;
            let value = resolve_legacy_value(declaration.property, &declaration.value, parent);
            self.values.insert(declaration.property, value);
        }
        Ok(())
    }
}

impl Default for Resolved {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedWithDiagnostics {
    resolved: Resolved,
    diagnostics: Vec<StyleDiagnostic>,
}

impl ResolvedWithDiagnostics {
    #[must_use]
    pub(crate) fn new(resolved: Resolved, diagnostics: Vec<StyleDiagnostic>) -> Self {
        Self {
            resolved,
            diagnostics,
        }
    }

    #[must_use]
    pub const fn resolved(&self) -> &Resolved {
        &self.resolved
    }

    #[must_use]
    pub fn into_resolved(self) -> Resolved {
        self.resolved
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[StyleDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug)]
pub struct Context<'a, T: Tree> {
    pub tree: &'a T,
    pub node: T::Id,
    pub traversal: Traversal,
    pub parent: Option<&'a Resolved>,
    pub local: Option<&'a Declarations>,
    pub animated: Option<&'a Declarations>,
    condition_facts: ConditionFacts,
    style_bucket: StyleBucket,
    selector_root: Option<T::Id>,
    selector_scope: Option<T::Id>,
}

impl<'a, T: Tree> Context<'a, T> {
    #[must_use]
    pub fn new(tree: &'a T, node: T::Id) -> Self {
        Self {
            tree,
            node,
            traversal: Traversal::Projected,
            parent: None,
            local: None,
            animated: None,
            condition_facts: ConditionFacts::new(),
            style_bucket: StyleBucket::Element,
            selector_root: None,
            selector_scope: None,
        }
    }

    #[must_use]
    pub const fn traversal(mut self, traversal: Traversal) -> Self {
        self.traversal = traversal;
        self
    }

    #[must_use]
    pub fn condition_facts(mut self, facts: ConditionFacts) -> Self {
        self.condition_facts = facts;
        self
    }

    #[must_use]
    pub fn media_environment(mut self, media: MediaEnvironment) -> Self {
        self.condition_facts = self.condition_facts.media(media);
        self
    }

    #[must_use]
    pub fn container_facts(mut self, container: ContainerFacts) -> Self {
        self.condition_facts = self.condition_facts.container(container);
        self
    }

    #[must_use]
    pub const fn parent(mut self, parent: &'a Resolved) -> Self {
        self.parent = Some(parent);
        self
    }

    #[must_use]
    pub const fn local(mut self, local: &'a Declarations) -> Self {
        self.local = Some(local);
        self
    }

    #[must_use]
    pub const fn animated(mut self, animated: &'a Declarations) -> Self {
        self.animated = Some(animated);
        self
    }

    #[must_use]
    pub const fn style_bucket(mut self, bucket: StyleBucket) -> Self {
        self.style_bucket = bucket;
        self
    }

    #[must_use]
    pub const fn selector_root(mut self, root: T::Id) -> Self {
        self.selector_root = Some(root);
        self
    }

    #[must_use]
    pub const fn selector_scope(mut self, scope: T::Id) -> Self {
        self.selector_scope = Some(scope);
        self
    }

    #[must_use]
    pub const fn conditions(&self) -> &ConditionFacts {
        &self.condition_facts
    }
}

#[derive(Clone, Debug)]
pub struct Resolver {
    sheet: Sheet,
    cache: BTreeMap<u64, CacheEntry>,
    cache_by_node: BTreeMap<u64, BTreeSet<u64>>,
    cache_hits: usize,
}

impl Resolver {
    #[must_use]
    pub fn new(sheet: Sheet) -> Self {
        Self {
            sheet,
            cache: BTreeMap::new(),
            cache_by_node: BTreeMap::new(),
            cache_hits: 0,
        }
    }

    #[must_use]
    pub fn sheet(&self) -> &Sheet {
        &self.sheet
    }

    #[must_use]
    pub const fn cache_hits(&self) -> usize {
        self.cache_hits
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_by_node.clear();
        self.cache_hits = 0;
    }

    pub fn clear_cache_for_sheet(&mut self, version: Version) {
        if self.sheet.version() == version {
            self.clear_cache();
        }
    }

    pub fn clear_cache_for_node<T: Hash>(&mut self, node: T) {
        let node_hash = hash_node(&node);
        let Some(keys) = self.cache_by_node.remove(&node_hash) else {
            return;
        };
        for key in keys {
            self.cache.remove(&key);
        }
    }

    pub fn resolve<T: Tree>(&mut self, context: Context<'_, T>) -> Result<Resolved> {
        self.resolve_inner(context, false)
            .map(ResolvedWithDiagnostics::into_resolved)
    }

    pub fn resolve_with_diagnostics<T: Tree>(
        &mut self,
        context: Context<'_, T>,
    ) -> Result<ResolvedWithDiagnostics> {
        self.resolve_inner(context, true)
    }

    fn resolve_inner<T: Tree>(
        &mut self,
        context: Context<'_, T>,
        collect_diagnostics: bool,
    ) -> Result<ResolvedWithDiagnostics> {
        let cache_key = self.cache_key(&context)?;
        if !collect_diagnostics
            && let Some(key) = cache_key
            && let Some(entry) = self.cache.get(&key.value)
        {
            self.cache_hits += 1;
            return Ok(ResolvedWithDiagnostics::new(
                entry.resolved.clone(),
                Vec::new(),
            ));
        }

        let mut resolved = Resolved::new();
        if let Some(parent) = context.parent {
            resolved.inherit_from(parent);
        }

        let mut rule_candidates = BTreeMap::<Property, Vec<RuleCandidate>>::new();
        let mut custom_candidates =
            BTreeMap::<CustomPropertyName, Vec<CustomPropertyCandidate>>::new();
        for rule in self.sheet.candidate_rules(context.tree, context.node)? {
            if rule.style_bucket() != context.style_bucket {
                continue;
            }
            if !Condition::matches_all(rule.conditions(), context.conditions()) {
                continue;
            }
            let mut selector_context = SelectorMatchContext::new(context.node, context.traversal);
            if let Some(root) = context.selector_root {
                selector_context = selector_context.with_root(root);
            }
            if let Some(scope) = context.selector_scope {
                selector_context = selector_context.with_scope(scope);
            }
            if let Some(rule_scope) = rule.scope() {
                let Some(anchor) = matching_scope_anchor(
                    rule_scope,
                    context.tree,
                    context.node,
                    context.traversal,
                    context.selector_root,
                )?
                else {
                    continue;
                };
                selector_context = selector_context.with_scope(anchor);
            }
            if rule
                .selector()
                .matches_with_context(context.tree, selector_context)?
            {
                for declaration in rule.declaration_items() {
                    let candidate = RuleCandidate::try_from_declaration(
                        declaration.property(),
                        rule.precedence(),
                        declaration.origin(),
                        declaration.value(),
                        declaration.source(),
                    )?;
                    rule_candidates
                        .entry(candidate.property)
                        .or_default()
                        .push(candidate);
                }
                for declaration in rule.custom_declaration_items() {
                    let candidate = CustomPropertyCandidate::from_declaration(
                        rule.precedence(),
                        declaration.origin(),
                        declaration.value(),
                        declaration.source(),
                    );
                    custom_candidates
                        .entry(declaration.name().clone())
                        .or_default()
                        .push(candidate);
                }
            }
        }
        resolve_custom_property_candidates(
            &mut resolved.custom_properties,
            &mut custom_candidates,
            context.parent,
        );
        let mut diagnostics = DiagnosticCollector::new(collect_diagnostics);
        for (property, candidates) in &mut rule_candidates {
            candidates.sort_by_key(|candidate| candidate.precedence);
            let value = resolve_rule_candidates(
                *property,
                candidates,
                context.parent,
                &resolved.custom_properties,
                &mut resolved.custom_property_dependencies,
                &mut diagnostics,
            );
            resolved.values.insert(*property, value);
        }

        if let Some(local) = context.local {
            resolved.apply(local, context.parent)?;
        }
        if let Some(animated) = context.animated {
            resolved.apply(animated, context.parent)?;
        }

        if let Some(key) = cache_key {
            self.cache_by_node
                .entry(key.node)
                .or_default()
                .insert(key.value);
            self.cache.insert(
                key.value,
                CacheEntry {
                    resolved: resolved.clone(),
                },
            );
        }
        Ok(ResolvedWithDiagnostics::new(
            resolved,
            diagnostics.into_diagnostics(),
        ))
    }

    fn cache_key<T: Tree>(&self, context: &Context<'_, T>) -> Result<Option<CacheKey>> {
        let Some(tree_version) = context.tree.version_hint() else {
            return Ok(None);
        };
        let node = context.tree.node(context.node)?;
        let node_hash = hash_node(&context.node);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.sheet.version().hash(&mut hasher);
        tree_version.hash(&mut hasher);
        node_hash.hash(&mut hasher);
        context.traversal.hash(&mut hasher);
        hash_state(&node.state, &mut hasher);
        hash_condition_facts(context.conditions(), &mut hasher);
        context.parent.map(Resolved::fingerprint).hash(&mut hasher);
        context
            .local
            .map(Declarations::fingerprint)
            .hash(&mut hasher);
        context
            .animated
            .map(Declarations::fingerprint)
            .hash(&mut hasher);
        context.style_bucket.hash(&mut hasher);
        context.selector_root.hash(&mut hasher);
        context.selector_scope.hash(&mut hasher);
        Ok(Some(CacheKey {
            value: hasher.finish(),
            node: node_hash,
        }))
    }
}

fn matching_scope_anchor<T: Tree>(
    scope: &RuleScope,
    tree: &T,
    subject: T::Id,
    traversal: Traversal,
    explicit_root: Option<T::Id>,
) -> Result<Option<T::Id>>
where
    T::Id: Copy + Eq,
{
    let mut ancestors = Vec::new();
    let mut candidate = subject;
    loop {
        ancestors.push(candidate);
        let Some(parent) = tree.parent(candidate, traversal)? else {
            break;
        };
        candidate = parent;
    }

    for (candidate_index, candidate) in ancestors.iter().copied().enumerate() {
        if !scope_root_matches(scope, tree, candidate, traversal, explicit_root, &ancestors)? {
            continue;
        }
        let limit_candidates = if candidate_index == 0 {
            &ancestors[..=candidate_index]
        } else {
            &ancestors[..candidate_index]
        };
        if scope_limit_blocks(scope, tree, limit_candidates, traversal, explicit_root)? {
            continue;
        }
        return Ok(Some(candidate));
    }
    Ok(None)
}

fn scope_root_matches<T: Tree>(
    scope: &RuleScope,
    tree: &T,
    candidate: T::Id,
    traversal: Traversal,
    explicit_root: Option<T::Id>,
    ancestors: &[T::Id],
) -> Result<bool>
where
    T::Id: Copy + Eq,
{
    let Some(roots) = scope.roots() else {
        if let Some(root) = explicit_root {
            return Ok(candidate == root);
        }
        return Ok(ancestors.last().is_some_and(|root| *root == candidate));
    };

    for selector in roots.selectors() {
        if selector.matches_with_context(
            tree,
            selector_context_for_candidate(candidate, traversal, explicit_root),
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn scope_limit_blocks<T: Tree>(
    scope: &RuleScope,
    tree: &T,
    candidates: &[T::Id],
    traversal: Traversal,
    explicit_root: Option<T::Id>,
) -> Result<bool>
where
    T::Id: Copy + Eq,
{
    let Some(limits) = scope.limits() else {
        return Ok(false);
    };
    for candidate in candidates.iter().copied() {
        for selector in limits.selectors() {
            if selector.matches_with_context(
                tree,
                selector_context_for_candidate(candidate, traversal, explicit_root),
            )? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn selector_context_for_candidate<Id: Copy>(
    candidate: Id,
    traversal: Traversal,
    explicit_root: Option<Id>,
) -> SelectorMatchContext<Id> {
    let context = SelectorMatchContext::new(candidate, traversal);
    if let Some(root) = explicit_root {
        context.with_root(root)
    } else {
        context
    }
}

#[derive(Clone, Debug)]
struct RuleCandidate {
    property: Property,
    precedence: RulePrecedence,
    origin: RuleDeclarationOrigin,
    value: RuleCandidateValue,
    source: Option<StyleSourceId>,
}

impl RuleCandidate {
    fn try_from_declaration(
        property: Property,
        precedence: RulePrecedence,
        origin: RuleDeclarationOrigin,
        value: RuleDeclarationValue<'_>,
        source: Option<StyleSourceId>,
    ) -> Result<Self> {
        let value = match value {
            RuleDeclarationValue::Value(value) => {
                property.validate_value(value)?;
                RuleCandidateValue::Value(value.clone())
            }
            RuleDeclarationValue::Authored(value) => match value {
                AuthoredCascadeValue::Value(value) => {
                    property.validate_value(value)?;
                    RuleCandidateValue::Value(value.clone())
                }
                AuthoredCascadeValue::CssWideKeyword(keyword) => {
                    RuleCandidateValue::CssWideKeyword(*keyword)
                }
                AuthoredCascadeValue::VariableDependent(value) => {
                    debug_assert_eq!(property, value.property());
                    RuleCandidateValue::VariableDependent(value.clone())
                }
            },
        };
        Ok(Self {
            property,
            precedence,
            origin,
            value,
            source,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RuleCandidateValue {
    Value(Value),
    CssWideKeyword(CssWideKeyword),
    VariableDependent(VariableDependentValue),
}

#[derive(Clone, Debug)]
struct CustomPropertyCandidate {
    precedence: RulePrecedence,
    origin: RuleDeclarationOrigin,
    value: CustomPropertyCandidateValue,
    #[allow(dead_code)]
    source: Option<StyleSourceId>,
}

impl CustomPropertyCandidate {
    fn from_declaration(
        precedence: RulePrecedence,
        origin: RuleDeclarationOrigin,
        value: &CustomPropertyCascadeValue,
        source: Option<StyleSourceId>,
    ) -> Self {
        let value = match value {
            CustomPropertyCascadeValue::Value(value) => {
                CustomPropertyCandidateValue::Value(value.clone())
            }
            CustomPropertyCascadeValue::CssWideKeyword(keyword) => {
                CustomPropertyCandidateValue::CssWideKeyword(*keyword)
            }
        };
        Self {
            precedence,
            origin,
            value,
            source,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CustomPropertyCandidateValue {
    Value(CustomPropertyValue),
    CssWideKeyword(CssWideKeyword),
}

fn resolve_custom_property_candidates(
    environment: &mut BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    candidates_by_name: &mut BTreeMap<CustomPropertyName, Vec<CustomPropertyCandidate>>,
    parent: Option<&Resolved>,
) {
    for candidates in candidates_by_name.values_mut() {
        candidates.sort_by_key(|candidate| candidate.precedence);
    }
    let names = candidates_by_name.keys().cloned().collect::<Vec<_>>();
    for name in names {
        let Some(candidates) = candidates_by_name.get(&name) else {
            continue;
        };
        let Some(index) = candidates.len().checked_sub(1) else {
            continue;
        };
        let mut visited = BTreeSet::new();
        match resolve_custom_property_candidate_at(&name, candidates, index, parent, &mut visited) {
            Some(resolution) => {
                environment.insert(name, resolution);
            }
            None => {
                environment.remove(&name);
            }
        }
    }
    mark_custom_property_cycles(environment);
}

fn resolve_custom_property_candidate_at(
    name: &CustomPropertyName,
    candidates: &[CustomPropertyCandidate],
    index: usize,
    parent: Option<&Resolved>,
    visited: &mut BTreeSet<usize>,
) -> Option<CustomPropertyResolution> {
    if !visited.insert(index) {
        return parent
            .and_then(|parent| parent.custom_property_resolution(name))
            .cloned();
    }
    let candidate = &candidates[index];
    let resolution = match &candidate.value {
        CustomPropertyCandidateValue::Value(value) => {
            Some(CustomPropertyResolution::valid(value.clone()))
        }
        CustomPropertyCandidateValue::CssWideKeyword(keyword) => {
            if matches!(keyword, CssWideKeyword::RevertLayer) {
                debug_assert_eq!(candidate.origin, RuleDeclarationOrigin::Authored);
            }
            resolve_custom_property_css_wide_keyword(
                name, *keyword, parent, candidates, index, visited,
            )
        }
    };
    visited.remove(&index);
    resolution
}

fn resolve_custom_property_css_wide_keyword(
    name: &CustomPropertyName,
    keyword: CssWideKeyword,
    parent: Option<&Resolved>,
    candidates: &[CustomPropertyCandidate],
    index: usize,
    visited: &mut BTreeSet<usize>,
) -> Option<CustomPropertyResolution> {
    match keyword {
        CssWideKeyword::Initial => None,
        CssWideKeyword::Inherit | CssWideKeyword::Unset => parent
            .and_then(|parent| parent.custom_property_resolution(name))
            .cloned(),
        CssWideKeyword::RevertLayer => {
            let layer = candidates[index].precedence.layer_order();
            if let Some((lower_index, _)) = candidates
                .iter()
                .enumerate()
                .rev()
                .find(|(_, candidate)| candidate.precedence.layer_order() < layer)
            {
                resolve_custom_property_candidate_at(name, candidates, lower_index, parent, visited)
            } else {
                parent
                    .and_then(|parent| parent.custom_property_resolution(name))
                    .cloned()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Done,
}

fn mark_custom_property_cycles(
    environment: &mut BTreeMap<CustomPropertyName, CustomPropertyResolution>,
) {
    let names = environment.keys().cloned().collect::<Vec<_>>();
    let mut states = BTreeMap::new();
    let mut stack = Vec::new();
    let mut cyclic = BTreeSet::new();
    for name in &names {
        collect_custom_property_cycles(name, environment, &mut states, &mut stack, &mut cyclic);
    }
    for name in cyclic {
        environment.insert(name, CustomPropertyResolution::invalid());
    }
}

fn collect_custom_property_cycles(
    name: &CustomPropertyName,
    environment: &BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    states: &mut BTreeMap<CustomPropertyName, VisitState>,
    stack: &mut Vec<CustomPropertyName>,
    cyclic: &mut BTreeSet<CustomPropertyName>,
) {
    match states.get(name).copied() {
        Some(VisitState::Done) => return,
        Some(VisitState::Visiting) => {
            if let Some(index) = stack.iter().position(|stacked| stacked == name) {
                cyclic.extend(stack[index..].iter().cloned());
            }
            return;
        }
        None => {}
    }
    states.insert(name.clone(), VisitState::Visiting);
    stack.push(name.clone());
    if let Some(resolution) = environment.get(name)
        && let Some(value) = resolution.value()
    {
        for dependency in required_custom_property_edges(value, environment) {
            if environment.contains_key(dependency) {
                collect_custom_property_cycles(dependency, environment, states, stack, cyclic);
            }
        }
    }
    stack.pop();
    states.insert(name.clone(), VisitState::Done);
}

fn required_custom_property_edges<'a>(
    value: &'a CustomPropertyValue,
    environment: &'a BTreeMap<CustomPropertyName, CustomPropertyResolution>,
) -> BTreeSet<&'a CustomPropertyName> {
    let mut edges = BTreeSet::new();
    for reference in value.references() {
        collect_required_reference_edges(reference, environment, &mut edges);
    }
    for property in Property::ALL {
        if let Some(typed_value) = value.typed_value(*property) {
            collect_required_expression_edges(typed_value.expression(), environment, &mut edges);
        }
    }
    edges
}

fn collect_required_expression_edges<'a>(
    expression: &'a VariableExpression,
    environment: &'a BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    edges: &mut BTreeSet<&'a CustomPropertyName>,
) {
    match expression {
        VariableExpression::Value(_) | VariableExpression::CssWideKeyword(_) => {}
        VariableExpression::Reference(reference) => {
            collect_required_reference_edges(reference, environment, edges);
        }
    }
}

fn collect_required_reference_edges<'a>(
    reference: &'a crate::VariableReference,
    environment: &'a BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    edges: &mut BTreeSet<&'a CustomPropertyName>,
) {
    edges.insert(reference.name());
    let needs_fallback = environment
        .get(reference.name())
        .is_none_or(|resolution| resolution.is_invalid() || resolution.value().is_none());
    if needs_fallback && let Some(fallback) = reference.fallback() {
        collect_required_expression_edges(fallback.expression(), environment, edges);
    }
}

fn resolve_legacy_value(property: Property, value: &Value, parent: Option<&Resolved>) -> Value {
    match value {
        Value::Keyword(super::Keyword::Initial) => {
            resolve_contextless_css_wide_keyword(property, CssWideKeyword::Initial, parent)
        }
        Value::Keyword(super::Keyword::Inherit) => {
            resolve_contextless_css_wide_keyword(property, CssWideKeyword::Inherit, parent)
        }
        Value::Keyword(super::Keyword::Unset) => {
            resolve_contextless_css_wide_keyword(property, CssWideKeyword::Unset, parent)
        }
        _ => value.clone(),
    }
}

fn resolve_rule_candidates(
    property: Property,
    candidates: &[RuleCandidate],
    parent: Option<&Resolved>,
    custom_properties: &BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    dependencies: &mut CustomPropertyDependencies,
    diagnostics: &mut DiagnosticCollector,
) -> Value {
    let Some(index) = candidates.len().checked_sub(1) else {
        return property.metadata().default().clone();
    };
    let mut evaluator = RuleEvaluator::new(
        parent,
        candidates,
        custom_properties,
        dependencies,
        diagnostics,
    );
    evaluator.resolve_candidate_at(property, index)
}

fn resolve_contextless_css_wide_keyword(
    property: Property,
    keyword: CssWideKeyword,
    parent: Option<&Resolved>,
) -> Value {
    match keyword {
        CssWideKeyword::Initial => property.metadata().default().clone(),
        CssWideKeyword::Inherit => resolve_inherit(property, parent),
        CssWideKeyword::Unset => resolve_unset(property, parent),
        CssWideKeyword::RevertLayer => resolve_unset(property, parent),
    }
}

struct RuleEvaluator<'a, 'dependencies> {
    parent: Option<&'a Resolved>,
    candidates: &'a [RuleCandidate],
    custom_properties: &'a BTreeMap<CustomPropertyName, CustomPropertyResolution>,
    dependencies: &'dependencies mut CustomPropertyDependencies,
    diagnostics: &'dependencies mut DiagnosticCollector,
    visited_candidates: BTreeSet<usize>,
}

impl<'a, 'dependencies> RuleEvaluator<'a, 'dependencies> {
    fn new(
        parent: Option<&'a Resolved>,
        candidates: &'a [RuleCandidate],
        custom_properties: &'a BTreeMap<CustomPropertyName, CustomPropertyResolution>,
        dependencies: &'dependencies mut CustomPropertyDependencies,
        diagnostics: &'dependencies mut DiagnosticCollector,
    ) -> Self {
        Self {
            parent,
            candidates,
            custom_properties,
            dependencies,
            diagnostics,
            visited_candidates: BTreeSet::new(),
        }
    }

    fn resolve_candidate_at(&mut self, property: Property, index: usize) -> Value {
        if !self.visited_candidates.insert(index) {
            return resolve_unset(property, self.parent);
        }
        let origin = self.candidates[index].origin;
        let candidate_value = self.candidates[index].value.clone();
        let value = match candidate_value {
            RuleCandidateValue::Value(value) => resolve_legacy_value(property, &value, self.parent),
            RuleCandidateValue::CssWideKeyword(keyword) => {
                if matches!(keyword, CssWideKeyword::RevertLayer) {
                    debug_assert_eq!(origin, RuleDeclarationOrigin::Authored);
                }
                self.resolve_css_wide_keyword(property, keyword, Some(index))
            }
            RuleCandidateValue::VariableDependent(value) => {
                let mut variable_stack = Vec::new();
                match self.evaluate_variable_expression(
                    property,
                    value.expression(),
                    index,
                    &mut variable_stack,
                ) {
                    Ok(value) => value,
                    Err(failure) => {
                        self.push_variable_failures(property, index, failure);
                        resolve_unset(property, self.parent)
                    }
                }
            }
        };
        self.visited_candidates.remove(&index);
        value
    }

    // Style-owned CSS-wide keyword resolution over root-supplied layer/source precedence.
    fn resolve_css_wide_keyword(
        &mut self,
        property: Property,
        keyword: CssWideKeyword,
        index: Option<usize>,
    ) -> Value {
        match keyword {
            CssWideKeyword::Initial => property.metadata().default().clone(),
            CssWideKeyword::Inherit => resolve_inherit(property, self.parent),
            CssWideKeyword::Unset => resolve_unset(property, self.parent),
            CssWideKeyword::RevertLayer => {
                let Some(index) = index else {
                    return resolve_unset(property, self.parent);
                };
                let layer = self.candidates[index].precedence.layer_order();
                let lower_index = self
                    .candidates
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, candidate)| candidate.precedence.layer_order() < layer)
                    .map(|(lower_index, _)| lower_index);
                lower_index
                    .map(|lower_index| self.resolve_candidate_at(property, lower_index))
                    .unwrap_or_else(|| resolve_unset(property, self.parent))
            }
        }
    }

    fn evaluate_variable_expression(
        &mut self,
        property: Property,
        expression: &VariableExpression,
        candidate_index: usize,
        variable_stack: &mut Vec<CustomPropertyName>,
    ) -> std::result::Result<Value, VariableResolutionFailure> {
        match expression {
            VariableExpression::Value(value) => property
                .validate_value(value)
                .ok()
                .map(|()| resolve_legacy_value(property, value, self.parent))
                .ok_or_else(VariableResolutionFailure::new),
            VariableExpression::CssWideKeyword(keyword) => {
                Ok(self.resolve_css_wide_keyword(property, *keyword, Some(candidate_index)))
            }
            VariableExpression::Reference(reference) => {
                self.dependencies.insert(property, reference.name().clone());
                if variable_stack.iter().any(|name| name == reference.name()) {
                    let failure = VariableResolutionFailure::from_reason(
                        InvalidAtComputedValueReason::CyclicCustomProperty(
                            reference.name().clone(),
                        ),
                    );
                    return self.evaluate_variable_fallback_or_failure(
                        property,
                        reference.fallback(),
                        candidate_index,
                        variable_stack,
                        failure,
                    );
                }

                let Some(resolution) = self.custom_properties.get(reference.name()) else {
                    let failure = VariableResolutionFailure::from_reason(
                        InvalidAtComputedValueReason::MissingCustomProperty(
                            reference.name().clone(),
                        ),
                    );
                    return self.evaluate_variable_fallback_or_failure(
                        property,
                        reference.fallback(),
                        candidate_index,
                        variable_stack,
                        failure,
                    );
                };
                if resolution.is_invalid() {
                    let failure = VariableResolutionFailure::from_reason(
                        InvalidAtComputedValueReason::CyclicCustomProperty(
                            reference.name().clone(),
                        ),
                    );
                    return self.evaluate_variable_fallback_or_failure(
                        property,
                        reference.fallback(),
                        candidate_index,
                        variable_stack,
                        failure,
                    );
                }
                let Some(value) = resolution.value() else {
                    let failure = VariableResolutionFailure::from_reason(
                        InvalidAtComputedValueReason::MissingCustomProperty(
                            reference.name().clone(),
                        ),
                    );
                    return self.evaluate_variable_fallback_or_failure(
                        property,
                        reference.fallback(),
                        candidate_index,
                        variable_stack,
                        failure,
                    );
                };
                let Some(typed_value) = value.typed_value(property) else {
                    let failure = VariableResolutionFailure::from_reason(
                        InvalidAtComputedValueReason::MissingTypedCustomPropertyValue(
                            reference.name().clone(),
                            property,
                        ),
                    );
                    return self.evaluate_variable_fallback_or_failure(
                        property,
                        reference.fallback(),
                        candidate_index,
                        variable_stack,
                        failure,
                    );
                };

                variable_stack.push(reference.name().clone());
                let result = self.evaluate_variable_expression(
                    property,
                    typed_value.expression(),
                    candidate_index,
                    variable_stack,
                );
                variable_stack.pop();
                result.or_else(|failure| {
                    self.evaluate_variable_fallback_or_failure(
                        property,
                        reference.fallback(),
                        candidate_index,
                        variable_stack,
                        failure,
                    )
                })
            }
        }
    }

    fn evaluate_variable_fallback(
        &mut self,
        property: Property,
        fallback: Option<&crate::VariableFallback>,
        candidate_index: usize,
        variable_stack: &mut Vec<CustomPropertyName>,
    ) -> std::result::Result<Value, VariableResolutionFailure> {
        fallback.map_or_else(
            || Err(VariableResolutionFailure::new()),
            |fallback| {
                self.evaluate_variable_expression(
                    property,
                    fallback.expression(),
                    candidate_index,
                    variable_stack,
                )
            },
        )
    }

    fn evaluate_variable_fallback_or_failure(
        &mut self,
        property: Property,
        fallback: Option<&crate::VariableFallback>,
        candidate_index: usize,
        variable_stack: &mut Vec<CustomPropertyName>,
        failure: VariableResolutionFailure,
    ) -> std::result::Result<Value, VariableResolutionFailure> {
        self.evaluate_variable_fallback(property, fallback, candidate_index, variable_stack)
            .map_err(|fallback_failure| failure.with_appended(fallback_failure))
    }

    fn push_variable_failures(
        &mut self,
        property: Property,
        candidate_index: usize,
        failure: VariableResolutionFailure,
    ) {
        let source = self.candidates[candidate_index].source;
        for reason in failure.into_reasons() {
            self.diagnostics.push_invalid_at_computed_value(
                StyleDiagnosticSubject::Property(property),
                source,
                reason,
            );
        }
    }
}

struct DiagnosticCollector {
    enabled: bool,
    diagnostics: Vec<StyleDiagnostic>,
}

impl DiagnosticCollector {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            diagnostics: Vec::new(),
        }
    }

    fn push_invalid_at_computed_value(
        &mut self,
        subject: StyleDiagnosticSubject,
        source: Option<StyleSourceId>,
        reason: InvalidAtComputedValueReason,
    ) {
        if self.enabled {
            self.diagnostics
                .push(StyleDiagnostic::invalid_at_computed_value(
                    subject, source, reason,
                ));
        }
    }

    fn into_diagnostics(self) -> Vec<StyleDiagnostic> {
        self.diagnostics
    }
}

struct VariableResolutionFailure {
    reasons: Vec<InvalidAtComputedValueReason>,
}

impl VariableResolutionFailure {
    fn new() -> Self {
        Self {
            reasons: Vec::new(),
        }
    }

    fn from_reason(reason: InvalidAtComputedValueReason) -> Self {
        Self {
            reasons: vec![reason],
        }
    }

    fn with_appended(mut self, mut fallback: Self) -> Self {
        self.reasons.append(&mut fallback.reasons);
        self
    }

    fn into_reasons(self) -> Vec<InvalidAtComputedValueReason> {
        self.reasons
    }
}

fn resolve_inherit(property: Property, parent: Option<&Resolved>) -> Value {
    parent
        .map(|parent| parent.get(property).clone())
        .unwrap_or_else(|| property.metadata().default().clone())
}

fn resolve_unset(property: Property, parent: Option<&Resolved>) -> Value {
    if property.metadata().is_inherited() {
        resolve_inherit(property, parent)
    } else {
        property.metadata().default().clone()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CacheKey {
    value: u64,
    node: u64,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    resolved: Resolved,
}

fn hash_state(state: &super::StyleState, hasher: &mut impl Hasher) {
    state.hovered().hash(hasher);
    state.active().hash(hasher);
    state.focused().hash(hasher);
    state.focus_visible().hash(hasher);
    state.focus_within().hash(hasher);
    state.pointer_captured().hash(hasher);
    state.enabled().hash(hasher);
    state.selected().hash(hasher);
    state.pressed().hash(hasher);
    state.checked().hash(hasher);
    state.expanded().hash(hasher);
    state.required().hash(hasher);
    state.valid().hash(hasher);
    state.placeholder_shown().hash(hasher);
    state.modal().hash(hasher);
    state.fullscreen().hash(hasher);
    state.popover_open().hash(hasher);
    state.default_state().hash(hasher);
    state.indeterminate().hash(hasher);
    state.read_write().hash(hasher);
    state.range_state().hash(hasher);
}

fn hash_condition_facts(facts: &ConditionFacts, hasher: &mut impl Hasher) {
    hash_media_environment(facts.media_environment(), hasher);
    facts.container_facts().is_some().hash(hasher);
    if let Some(container) = facts.container_facts() {
        hash_container_facts(container, hasher);
    }
}

fn hash_media_environment(environment: &MediaEnvironment, hasher: &mut impl Hasher) {
    environment.media_type_fact().hash(hasher);
    hash_query_length_option(environment.width_fact(), hasher);
    hash_query_length_option(environment.height_fact(), hasher);
    hash_query_length_basis(environment.length_basis(), hasher);
    hash_resolution_option(environment.resolution_fact(), hasher);
    environment.color_fact().hash(hasher);
    environment.monochrome_fact().hash(hasher);
    environment.orientation_fact().hash(hasher);
    environment.prefers_color_scheme_fact().hash(hasher);
    environment.prefers_reduced_motion_fact().hash(hasher);
    environment.prefers_reduced_transparency_fact().hash(hasher);
    environment.prefers_contrast_fact().hash(hasher);
    environment.forced_colors_fact().hash(hasher);
    environment.hover_fact().hash(hasher);
    environment.any_hover_fact().hash(hasher);
    environment.pointer_fact().hash(hasher);
    environment.any_pointer_fact().hash(hasher);
    environment.display_mode_fact().hash(hasher);
}

fn hash_container_facts(facts: &ContainerFacts, hasher: &mut impl Hasher) {
    facts.name_fact().hash(hasher);
    hash_query_length_option(facts.width_fact(), hasher);
    hash_query_length_option(facts.height_fact(), hasher);
    hash_query_length_option(facts.inline_size_fact(), hasher);
    hash_query_length_option(facts.block_size_fact(), hasher);
    hash_query_length_basis(facts.length_basis(), hasher);
    hash_ratio_option(facts.aspect_ratio_fact(), hasher);
    facts.orientation_fact().hash(hasher);
    for (name, value) in facts.custom_properties() {
        name.hash(hasher);
        value.hash(hasher);
    }
}

fn hash_query_length_basis(basis: &QueryLengthBasis, hasher: &mut impl Hasher) {
    hash_query_length_option(basis.font_size_basis(), hasher);
    hash_query_length_option(basis.root_font_size_basis(), hasher);
    hash_query_length_option(basis.ex_size_basis(), hasher);
    hash_query_length_option(basis.cap_size_basis(), hasher);
    hash_query_length_option(basis.ch_size_basis(), hasher);
    hash_query_length_option(basis.ic_size_basis(), hasher);
    hash_query_length_option(basis.line_height_basis(), hasher);
    hash_query_length_option(basis.root_ex_size_basis(), hasher);
    hash_query_length_option(basis.root_cap_size_basis(), hasher);
    hash_query_length_option(basis.root_ch_size_basis(), hasher);
    hash_query_length_option(basis.root_ic_size_basis(), hasher);
    hash_query_length_option(basis.root_line_height_basis(), hasher);
    hash_query_length_option(basis.viewport_width_basis(), hasher);
    hash_query_length_option(basis.viewport_height_basis(), hasher);
    hash_query_length_option(basis.small_viewport_width_basis(), hasher);
    hash_query_length_option(basis.small_viewport_height_basis(), hasher);
    hash_query_length_option(basis.large_viewport_width_basis(), hasher);
    hash_query_length_option(basis.large_viewport_height_basis(), hasher);
    hash_query_length_option(basis.dynamic_viewport_width_basis(), hasher);
    hash_query_length_option(basis.dynamic_viewport_height_basis(), hasher);
    hash_query_length_option(basis.container_width_basis(), hasher);
    hash_query_length_option(basis.container_height_basis(), hasher);
    hash_query_length_option(basis.container_inline_size_basis(), hasher);
    hash_query_length_option(basis.container_block_size_basis(), hasher);
}

fn hash_query_length_option(value: Option<QueryLength>, hasher: &mut impl Hasher) {
    value.is_some().hash(hasher);
    if let Some(value) = value {
        value.value().to_bits().hash(hasher);
        value.unit().hash(hasher);
    }
}

fn hash_resolution_option(value: Option<Resolution>, hasher: &mut impl Hasher) {
    value.is_some().hash(hasher);
    if let Some(value) = value {
        value.value().to_bits().hash(hasher);
        value.unit().hash(hasher);
    }
}

fn hash_ratio_option(value: Option<Ratio>, hasher: &mut impl Hasher) {
    value.is_some().hash(hasher);
    if let Some(value) = value {
        value.numerator().to_bits().hash(hasher);
        value.denominator().to_bits().hash(hasher);
    }
}

fn hash_node<T: Hash>(node: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}

fn hash_custom_property_resolution(
    resolution: &CustomPropertyResolution,
    hasher: &mut DefaultHasher,
) {
    resolution.is_invalid().hash(hasher);
    if let Some(value) = resolution.value() {
        hash_custom_property_value(value, hasher);
    }
}

fn hash_custom_property_value(value: &CustomPropertyValue, hasher: &mut DefaultHasher) {
    value.authored().as_css().hash(hasher);
    for dependency in value.dependencies() {
        dependency.hash(hasher);
    }
    for property in Property::ALL {
        if let Some(typed_value) = value.typed_value(*property) {
            property.hash(hasher);
            hash_variable_expression(typed_value.expression(), hasher);
        }
    }
}

fn hash_variable_expression(expression: &VariableExpression, hasher: &mut DefaultHasher) {
    match expression {
        VariableExpression::Value(value) => {
            0_u8.hash(hasher);
            hash_value(value, hasher);
        }
        VariableExpression::CssWideKeyword(keyword) => {
            1_u8.hash(hasher);
            keyword.hash(hasher);
        }
        VariableExpression::Reference(reference) => {
            2_u8.hash(hasher);
            reference.name().hash(hasher);
            if let Some(fallback) = reference.fallback() {
                true.hash(hasher);
                fallback.authored().as_css().hash(hasher);
                hash_variable_expression(fallback.expression(), hasher);
            } else {
                false.hash(hasher);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AnimationDirection, AnimationDirectionList, AnimationFillMode, AnimationFillModeList,
        AnimationIterationCount, AnimationIterationCountList, AnimationIterationNumber,
        AnimationName, AnimationNameList, AnimationPlayState, AnimationPlayStateList, AspectRatio,
        AuthoredDeclaration, AuthoredDeclarations, AuthoredProperty, AuthoredTokens, AuthoredValue,
        BuiltInCounterStyle, Color, Combinator, ComplexSelectorPart, ContainerCondition,
        ContainerFacts, ContainerFeatureQuery, Content, ContentItem, ContentItemList,
        ContentString, ContentVisibility, CounterChange, CounterChangeList, CounterChanges,
        CounterName, CounterStyle, CssWideKeyword, CustomPropertyName, CustomPropertyValue,
        Declarations, DurationSeconds, EasingFunction, EasingList, Error, ErrorCode,
        FilterFunction, FilterFunctionList, Flex, FontFamilyList, FontFeature, FontFeatureSettings,
        FontFeatureTag, FontFeatureValue, FontStretch, FontVariant, FontWeight, FontWeightNumber,
        ImageLayer, InvalidAtComputedValueReason, KeyframesIdent, KeyframesName, LayerOrder,
        LayoutPosition, LetterSpacing, ListStyle, ListStyleImage, ListStylePosition, ListStyleType,
        MediaCondition, MediaEnvironment, MediaFeatureQuery, MediaQuery, MediaQueryList, Node,
        Order, OverflowWrap, PlaceContentAlignment, QueryComparison, QueryLength, QueryLengthUnit,
        RangeFeature, RulePrecedence, RuleScope, RuleTarget, ScopeSelectorList, ScrollbarWidth,
        Selector, SelectorSpecificity, SourceOrder, StyleBucket, StyleClass, StyleColor, StyleRole,
        StyleSourceId, StyleState, StyleTag, StyleUrl, SymbolicFunctionValue, SystemColor,
        TextAlignLast, TextDecorationLine, TextDecorationLineComponent, TextDecorationStyle,
        TextDecorationThickness, TextIndent, TextOverflow, TextSlant, TextTransform, TextWrap,
        TimeList, UserSelect, VariableDependentValue, VariableExpression, VariableFallback,
        VariableReference, VerticalAlign, WhiteSpace, WordBreak, ZIndex,
    };

    fn precedence(layer: u32, source: u32) -> RulePrecedence {
        RulePrecedence::new(LayerOrder::new(layer), SourceOrder::new(source))
    }

    fn authored_color(value: Color) -> AuthoredDeclarations {
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Color),
                    AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(value))),
                )
                .unwrap(),
            )
            .unwrap();
        declarations
    }

    fn authored_width(value: Length) -> AuthoredDeclarations {
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Width),
                    AuthoredValue::Value(Value::Length(value)),
                )
                .unwrap(),
            )
            .unwrap();
        declarations
    }

    fn authored_keyword(property: Property, keyword: CssWideKeyword) -> AuthoredDeclarations {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(property),
            keyword,
        ));
        declarations
    }

    fn custom_name(name: &str) -> CustomPropertyName {
        CustomPropertyName::try_new(name).unwrap()
    }

    fn custom_color_declarations(name: &str, color: Color) -> AuthoredDeclarations {
        custom_value_declarations(
            custom_name(name),
            CustomPropertyValue::new(AuthoredTokens::new(format!("{color:?}")), [])
                .try_with_typed_value(
                    Property::Color,
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(color))),
                )
                .unwrap(),
        )
    }

    fn custom_value_declarations(
        name: CustomPropertyName,
        value: CustomPropertyValue,
    ) -> AuthoredDeclarations {
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Custom(name),
                    AuthoredValue::CustomProperty(value),
                )
                .unwrap(),
            )
            .unwrap();
        declarations
    }

    fn custom_keyword_declarations(
        name: CustomPropertyName,
        keyword: CssWideKeyword,
    ) -> AuthoredDeclarations {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Custom(name),
            keyword,
        ));
        declarations
    }

    fn variable_color_declarations(
        name: CustomPropertyName,
        fallback: Option<VariableExpression>,
    ) -> AuthoredDeclarations {
        let fallback = fallback
            .map(|expression| VariableFallback::new(AuthoredTokens::new("fallback"), expression));
        let variable = VariableDependentValue::try_new(
            Property::Color,
            AuthoredTokens::new(format!("var({})", name.as_str())),
            VariableExpression::Reference(VariableReference::new(name, fallback)),
        )
        .unwrap();
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Color),
                    AuthoredValue::VariableDependent(variable),
                )
                .unwrap(),
            )
            .unwrap();
        declarations
    }

    fn push_custom_color(sheet: &mut Sheet, name: &str, color: Color, precedence: RulePrecedence) {
        push_authored(sheet, custom_color_declarations(name, color), precedence);
    }

    fn dependency_names_for_property(resolved: &Resolved, property: Property) -> Vec<String> {
        resolved
            .custom_property_dependencies()
            .for_property(property)
            .map(|name| name.as_str().to_owned())
            .collect()
    }

    fn resolve_child(sheet: Sheet, parent: Option<&Resolved>) -> Resolved {
        let tree = TestTree::new(vec![
            TestNode::new(0, "stack").children([1]),
            TestNode::new(1, "button"),
        ]);
        let mut resolver = Resolver::new(sheet);
        let context = Context::new(&tree, 1);
        let context = if let Some(parent) = parent {
            context.parent(parent)
        } else {
            context
        };

        resolver.resolve(context).unwrap()
    }

    fn resolve_single(declarations: Declarations) -> Resolved {
        resolve_child(
            Sheet::new().rule(Selector::tag("button").unwrap(), declarations),
            None,
        )
    }

    fn parent_color(color: Color) -> Resolved {
        let mut parent = Resolved::new();
        parent
            .apply(
                &Declarations::new().try_concrete_text_color(color).unwrap(),
                None,
            )
            .unwrap();
        parent
    }

    fn parent_custom_color(name: &str, color: Color) -> Resolved {
        let mut sheet = Sheet::new();
        push_custom_color(&mut sheet, name, color, precedence(1, 0));
        resolve_child(sheet, None)
    }

    fn push_authored(
        sheet: &mut Sheet,
        declarations: AuthoredDeclarations,
        precedence: RulePrecedence,
    ) {
        sheet
            .push_authored_rule(Selector::tag("button").unwrap(), declarations, precedence)
            .unwrap();
    }

    #[test]
    fn media_condition_rules_match_environment_facts() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let query = MediaQueryList::try_new([MediaQuery::Condition(MediaCondition::Feature(
            MediaFeatureQuery::Width(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(640.0, QueryLengthUnit::Px).unwrap(),
            )),
        ))])
        .unwrap();
        let sheet = Sheet::new().conditional_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                .unwrap(),
            [Condition::media(query)],
        );
        let mut resolver = Resolver::new(sheet);

        let matching = resolver
            .resolve(
                Context::new(&tree, 0).media_environment(
                    MediaEnvironment::new()
                        .width(QueryLength::try_new(800.0, QueryLengthUnit::Px).unwrap()),
                ),
            )
            .unwrap();
        let non_matching = resolver
            .resolve(
                Context::new(&tree, 0).media_environment(
                    MediaEnvironment::new()
                        .width(QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap()),
                ),
            )
            .unwrap();

        assert_eq!(
            matching.text_color(),
            &StyleColor::rgba(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(non_matching.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn container_condition_rules_match_container_facts() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let condition =
            ContainerCondition::Feature(ContainerFeatureQuery::Width(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(300.0, QueryLengthUnit::Px).unwrap(),
            )));
        let sheet = Sheet::new().conditional_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
                .unwrap(),
            [Condition::container(condition)],
        );
        let mut resolver = Resolver::new(sheet);

        let matching = resolver
            .resolve(
                Context::new(&tree, 0).container_facts(
                    ContainerFacts::new()
                        .width(QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap()),
                ),
            )
            .unwrap();
        let missing = resolver.resolve(Context::new(&tree, 0)).unwrap();

        assert_eq!(
            matching.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
        );
        assert_eq!(missing.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn animation_longhand_declarations_resolve_typed_lists() {
        let names = AnimationNameList::try_new([AnimationName::Keyframes(KeyframesName::Ident(
            KeyframesIdent::try_new("fade-in").unwrap(),
        ))])
        .unwrap();
        let durations = TimeList::try_new([DurationSeconds::new(0.25).unwrap()]).unwrap();
        let delays = TimeList::try_new([DurationSeconds::new(0.05).unwrap()]).unwrap();
        let timing = EasingList::try_new([EasingFunction::EaseInOut]).unwrap();
        let iterations = AnimationIterationCountList::try_new([AnimationIterationCount::Number(
            AnimationIterationNumber::try_new(2.0).unwrap(),
        )])
        .unwrap();
        let directions = AnimationDirectionList::try_new([AnimationDirection::Alternate]).unwrap();
        let fill_modes = AnimationFillModeList::try_new([AnimationFillMode::Both]).unwrap();
        let play_states = AnimationPlayStateList::try_new([AnimationPlayState::Paused]).unwrap();

        let style = resolve_single(
            Declarations::new()
                .animation_name(names.clone())
                .unwrap()
                .animation_duration(durations.clone())
                .unwrap()
                .animation_delay(delays.clone())
                .unwrap()
                .animation_timing_function(timing.clone())
                .unwrap()
                .animation_iteration_count(iterations.clone())
                .unwrap()
                .animation_direction(directions.clone())
                .unwrap()
                .animation_fill_mode(fill_modes.clone())
                .unwrap()
                .animation_play_state(play_states.clone())
                .unwrap(),
        );

        assert_eq!(style.animation_name(), &names);
        assert_eq!(style.animation_duration(), &durations);
        assert_eq!(style.animation_delay(), &delays);
        assert_eq!(style.animation_timing_function(), &timing);
        assert_eq!(style.animation_iteration_count(), &iterations);
        assert_eq!(style.animation_direction(), &directions);
        assert_eq!(style.animation_fill_mode(), &fill_modes);
        assert_eq!(style.animation_play_state(), &play_states);
    }

    #[test]
    fn resolved_edge_getters_assemble_side_longhands() {
        let style = resolve_single(
            Declarations::new()
                .try_margin_top(Length::Px(2.0))
                .unwrap()
                .try_margin_right(Length::Px(4.0))
                .unwrap()
                .try_padding_bottom(Length::Px(6.0))
                .unwrap()
                .try_border_left_width(Length::Px(8.0))
                .unwrap(),
        );

        assert_eq!(style.margin_edges().top, Length::Px(2.0));
        assert_eq!(style.margin_edges().right, Length::Px(4.0));
        assert_eq!(style.margin_edges().bottom, Length::ZERO);
        assert_eq!(style.margin_edges().left, Length::ZERO);

        assert_eq!(style.padding_edges().top, Length::ZERO);
        assert_eq!(style.padding_edges().bottom, Length::Px(6.0));

        assert_eq!(style.border_width_edges().left, Length::Px(8.0));
        assert_eq!(style.inset_edges().top, Length::Auto);
    }

    #[test]
    fn resolved_core_layout_getters_return_typed_values() {
        let style = resolve_single(
            Declarations::new()
                .position(LayoutPosition::Sticky)
                .z_index(ZIndex::integer(3))
                .scrollbar_width(ScrollbarWidth::None)
                .content_visibility(ContentVisibility::Hidden)
                .order(Order::new(-1))
                .try_aspect_ratio(AspectRatio::ratio(2.0).unwrap())
                .unwrap(),
        );

        assert_eq!(style.position(), LayoutPosition::Sticky);
        assert_eq!(style.z_index(), ZIndex::integer(3));
        assert_eq!(style.scrollbar_width(), ScrollbarWidth::None);
        assert_eq!(style.content_visibility(), ContentVisibility::Hidden);
        assert_eq!(style.order(), Order::new(-1));
        assert_eq!(style.aspect_ratio(), AspectRatio::ratio(2.0).unwrap());
    }

    #[test]
    fn layout_operation_eight_values_resolve_together() {
        let style = resolve_single(
            Declarations::new()
                .display(Display::Grid)
                .position(LayoutPosition::Fixed)
                .try_inset(Edges::all(Length::Px(3.0)))
                .unwrap()
                .try_margin_left(Length::Px(-2.0))
                .unwrap()
                .try_padding(Edges::all(Length::Px(4.0)))
                .unwrap()
                .try_border_width(Edges::all(Length::Px(1.0)))
                .unwrap()
                .z_index(ZIndex::integer(7))
                .scrollbar_width(ScrollbarWidth::Thin)
                .content_visibility(ContentVisibility::Auto)
                .order(Order::new(5))
                .try_flex(Flex::auto())
                .unwrap()
                .place_content(PlaceContentAlignment::all(AlignContent::Center))
                .align_tracks(AlignContent::SpaceEvenly),
        );

        assert_eq!(style.display(), Display::Grid);
        assert_eq!(style.position(), LayoutPosition::Fixed);
        assert_eq!(style.inset_edges().left, Length::Px(3.0));
        assert_eq!(style.margin_edges().left, Length::Px(-2.0));
        assert_eq!(style.padding_edges().right, Length::Px(4.0));
        assert_eq!(style.border_width_edges().top, Length::Px(1.0));
        assert_eq!(style.z_index(), ZIndex::integer(7));
        assert_eq!(style.scrollbar_width(), ScrollbarWidth::Thin);
        assert_eq!(style.content_visibility(), ContentVisibility::Auto);
        assert_eq!(style.order(), Order::new(5));
        assert_eq!(style.flex_grow(), FlexFactor::one());
        assert_eq!(style.flex_shrink(), FlexFactor::one());
        assert_eq!(style.align_tracks(), AlignContent::SpaceEvenly);
    }

    #[test]
    fn generated_content_values_resolve_on_pseudo_buckets_without_tree_materialization() {
        let tree = TestTree::new(vec![TestNode::new(0, "root")]);
        let content = Content::Items(
            ContentItemList::try_new([ContentItem::String(ContentString::try_new("New").unwrap())])
                .unwrap(),
        );
        let sheet = Sheet::new().targeted_rule(
            RuleTarget::new(Selector::tag("root").unwrap(), StyleBucket::Before),
            Declarations::new().content(content.clone()).unwrap(),
        );

        let style = Resolver::new(sheet.clone())
            .resolve(Context::new(&tree, 0).style_bucket(StyleBucket::Before))
            .unwrap();
        let element_style = Resolver::new(sheet)
            .resolve(Context::new(&tree, 0))
            .unwrap();

        assert_eq!(style.content(), &content);
        assert_eq!(element_style.content(), &Content::Normal);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn list_marker_and_counter_values_resolve_together() {
        let tree = TestTree::new(vec![TestNode::new(0, "item")]);
        let counter_name = CounterName::try_new("item").unwrap();
        let changes = CounterChanges::Changes(
            CounterChangeList::try_new([CounterChange::new(counter_name, 1)]).unwrap(),
        );
        let marker_image = ListStyleImage::Url(StyleUrl::new("marker.svg").unwrap());
        let sheet = Sheet::new().targeted_rule(
            RuleTarget::element(Selector::tag("item").unwrap()),
            Declarations::new()
                .list_style(
                    ListStyle::try_new(
                        Some(ListStyleType::CounterStyle(CounterStyle::BuiltIn(
                            BuiltInCounterStyle::Decimal,
                        ))),
                        Some(ListStylePosition::Inside),
                        Some(marker_image.clone()),
                    )
                    .unwrap(),
                )
                .unwrap()
                .counter_increment(changes.clone())
                .unwrap(),
        );

        let style = Resolver::new(sheet)
            .resolve(Context::new(&tree, 0))
            .unwrap();

        assert!(matches!(
            style.list_style_type(),
            ListStyleType::CounterStyle(CounterStyle::BuiltIn(BuiltInCounterStyle::Decimal))
        ));
        assert_eq!(style.list_style_position(), ListStylePosition::Inside);
        assert_eq!(style.list_style_image(), &marker_image);
        assert_eq!(style.counter_increment(), &changes);
    }

    #[test]
    fn text_operation_nine_values_resolve_together() {
        let feature_settings = FontFeatureSettings::features([FontFeature::new(
            FontFeatureTag::new("liga").unwrap(),
            Some(FontFeatureValue::Off),
        )])
        .unwrap();
        let indent = TextIndent::new(Length::Px(10.0), true, true).unwrap();
        let decoration_line =
            TextDecorationLine::try_new([TextDecorationLineComponent::Underline]).unwrap();
        let decoration_thickness =
            TextDecorationThickness::try_length(Length::Percent(12.0)).unwrap();

        let style = resolve_single(
            Declarations::new()
                .try_font_family(FontFamilyList::new(["Inter", "serif"]).unwrap())
                .unwrap()
                .try_font_size(Length::Px(18.0))
                .unwrap()
                .try_line_height(Length::Percent(130.0))
                .unwrap()
                .font_weight(FontWeight::Number(FontWeightNumber::new(625).unwrap()))
                .try_font_style(TextSlant::Italic)
                .unwrap()
                .font_stretch(FontStretch::SemiExpanded)
                .font_variant(FontVariant::SmallCaps)
                .try_font_feature_settings(feature_settings.clone())
                .unwrap()
                .text_align_last(TextAlignLast::End)
                .try_text_indent(indent.clone())
                .unwrap()
                .vertical_align(VerticalAlign::TextTop)
                .try_letter_spacing(LetterSpacing::try_length(Length::Px(0.5)).unwrap())
                .unwrap()
                .text_wrap(TextWrap::Stable)
                .white_space(WhiteSpace::PreLine)
                .word_break(WordBreak::BreakAll)
                .overflow_wrap(OverflowWrap::Anywhere)
                .text_overflow(TextOverflow::Ellipsis)
                .try_text_decoration_line(decoration_line.clone())
                .unwrap()
                .text_decoration_style(TextDecorationStyle::Dotted)
                .try_text_decoration_thickness(decoration_thickness.clone())
                .unwrap()
                .text_transform(TextTransform::Lowercase),
        );

        assert_eq!(style.font_size(), Length::Px(18.0));
        assert_eq!(style.line_height(), Length::Percent(130.0));
        assert_eq!(
            style.font_weight(),
            FontWeight::Number(FontWeightNumber::new(625).unwrap())
        );
        assert_eq!(style.font_style(), TextSlant::Italic);
        assert_eq!(style.font_stretch(), FontStretch::SemiExpanded);
        assert_eq!(style.font_variant(), FontVariant::SmallCaps);
        assert_eq!(style.font_feature_settings(), &feature_settings);
        assert_eq!(style.text_align_last(), TextAlignLast::End);
        assert_eq!(style.text_indent(), indent);
        assert_eq!(style.vertical_align(), VerticalAlign::TextTop);
        assert_eq!(
            style.letter_spacing(),
            LetterSpacing::try_length(Length::Px(0.5)).unwrap()
        );
        assert_eq!(style.text_wrap(), TextWrap::Stable);
        assert_eq!(style.white_space(), WhiteSpace::PreLine);
        assert_eq!(style.word_break(), WordBreak::BreakAll);
        assert_eq!(style.overflow_wrap(), OverflowWrap::Anywhere);
        assert_eq!(style.text_overflow(), TextOverflow::Ellipsis);
        assert_eq!(style.text_decoration_line(), &decoration_line);
        assert_eq!(style.text_decoration_style(), TextDecorationStyle::Dotted);
        assert_eq!(style.text_decoration_thickness(), &decoration_thickness);
        assert_eq!(style.text_transform(), TextTransform::Lowercase);
    }

    #[test]
    fn resolved_font_getters_return_typed_values() {
        let features = FontFeatureSettings::features([FontFeature::new(
            FontFeatureTag::new("kern").unwrap(),
            Some(FontFeatureValue::On),
        )])
        .unwrap();

        let style = resolve_single(
            Declarations::new()
                .try_font_family(FontFamilyList::new(["Inter", "serif"]).unwrap())
                .unwrap()
                .try_font_size(Length::Px(17.0))
                .unwrap()
                .try_line_height(Length::Percent(140.0))
                .unwrap()
                .font_weight(FontWeight::Bold)
                .try_font_style(TextSlant::Oblique(None))
                .unwrap()
                .font_stretch(FontStretch::Expanded)
                .font_variant(FontVariant::SmallCaps)
                .try_font_feature_settings(features.clone())
                .unwrap(),
        );

        assert_eq!(
            style.font_family().as_slice(),
            &["Inter".to_string(), "serif".to_string()]
        );
        assert_eq!(style.font_size(), Length::Px(17.0));
        assert_eq!(style.line_height(), Length::Percent(140.0));
        assert_eq!(style.font_weight(), FontWeight::Bold);
        assert_eq!(style.font_style(), TextSlant::Oblique(None));
        assert_eq!(style.font_stretch(), FontStretch::Expanded);
        assert_eq!(style.font_variant(), FontVariant::SmallCaps);
        assert_eq!(style.font_feature_settings(), &features);
    }

    #[test]
    fn resolved_inline_text_getters_return_typed_values() {
        let indent = TextIndent::new(Length::Px(8.0), false, true).unwrap();
        let style = resolve_single(
            Declarations::new()
                .text_align_last(TextAlignLast::Center)
                .try_text_indent(indent.clone())
                .unwrap()
                .vertical_align(VerticalAlign::Middle)
                .try_letter_spacing(LetterSpacing::try_length(Length::Px(1.0)).unwrap())
                .unwrap()
                .text_transform(TextTransform::Capitalize),
        );

        assert_eq!(style.text_align_last(), TextAlignLast::Center);
        assert_eq!(style.text_indent(), indent);
        assert_eq!(style.vertical_align(), VerticalAlign::Middle);
        assert_eq!(
            style.letter_spacing(),
            LetterSpacing::try_length(Length::Px(1.0)).unwrap()
        );
        assert_eq!(style.text_transform(), TextTransform::Capitalize);
    }

    #[test]
    fn resolved_text_flow_getters_return_typed_values() {
        let style = resolve_single(
            Declarations::new()
                .text_wrap(TextWrap::Pretty)
                .white_space(WhiteSpace::PreWrap)
                .word_break(WordBreak::KeepAll)
                .overflow_wrap(OverflowWrap::BreakWord)
                .text_overflow(TextOverflow::Ellipsis),
        );

        assert_eq!(style.text_wrap(), TextWrap::Pretty);
        assert_eq!(style.white_space(), WhiteSpace::PreWrap);
        assert_eq!(style.word_break(), WordBreak::KeepAll);
        assert_eq!(style.overflow_wrap(), OverflowWrap::BreakWord);
        assert_eq!(style.text_overflow(), TextOverflow::Ellipsis);
    }

    #[test]
    fn resolved_text_decoration_getters_return_typed_values() {
        let line = TextDecorationLine::try_new([TextDecorationLineComponent::Overline]).unwrap();
        let thickness = TextDecorationThickness::FromFont;
        let style = resolve_single(
            Declarations::new()
                .try_text_decoration_line(line.clone())
                .unwrap()
                .text_decoration_style(TextDecorationStyle::Dashed)
                .try_text_decoration_thickness(thickness.clone())
                .unwrap(),
        );

        assert_eq!(style.text_decoration_line(), &line);
        assert_eq!(style.text_decoration_style(), TextDecorationStyle::Dashed);
        assert_eq!(style.text_decoration_thickness(), &thickness);
    }

    #[test]
    fn resolved_color_getters_preserve_symbolic_colors() {
        let decoration = StyleColor::current_color();
        let background = StyleColor::system(SystemColor::Canvas);
        let style = resolve_single(
            Declarations::new()
                .try_text_color(StyleColor::rgba(Color::BLACK))
                .unwrap()
                .try_background_color(background.clone())
                .unwrap()
                .try_text_decoration_color(decoration.clone())
                .unwrap(),
        );

        assert_eq!(style.text_color(), &StyleColor::rgba(Color::BLACK));
        assert_eq!(style.background(), &background);
        assert_eq!(style.text_decoration_color(), &decoration);
    }

    #[test]
    fn paint_operation_ten_values_resolve_together() {
        let background = StyleColor::system(SystemColor::Canvas);
        let border_color = StyleColor::current_color();
        let corner = CornerRadius::new(Length::Px(8.0), Length::Px(12.0)).unwrap();
        let images =
            ImageLayerList::try_new([ImageLayer::url(StyleUrl::new("bg.png").unwrap())]).unwrap();
        let filter = Filter::Functions(
            FilterFunctionList::try_new([FilterFunction::Brightness(
                SymbolicFunctionValue::new("120%").unwrap(),
            )])
            .unwrap(),
        );
        let translate = Translate::try_values([Length::Px(2.0)]).unwrap();
        let scale = Scale::try_values([1.0, 1.2]).unwrap();

        let style = resolve_single(
            Declarations::new()
                .try_background_color(background.clone())
                .unwrap()
                .background_image(images.clone())
                .try_border_top_color(border_color.clone())
                .unwrap()
                .border_top_style(BorderLineStyle::Solid)
                .try_border_top_left_radius(corner.clone())
                .unwrap()
                .box_decoration_break(BoxDecorationBreak::Clone)
                .filter(filter.clone())
                .clip_path(ClipPath::None)
                .translate(translate.clone())
                .scale(scale.clone())
                .user_select(UserSelect::Text),
        );

        assert_eq!(style.background(), &background);
        assert_eq!(style.background_image(), &images);
        assert_eq!(style.border_top_color(), &border_color);
        assert_eq!(style.border_top_style(), BorderLineStyle::Solid);
        assert_eq!(style.border_top_left_radius(), &corner);
        assert_eq!(style.box_decoration_break(), BoxDecorationBreak::Clone);
        assert_eq!(style.filter(), &filter);
        assert_eq!(style.clip_path(), &ClipPath::None);
        assert_eq!(style.translate(), &translate);
        assert_eq!(style.scale(), &scale);
        assert_eq!(style.user_select(), UserSelect::Text);
    }

    #[test]
    fn higher_layer_wins_over_later_source_order() {
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0)),
            precedence(1, 100),
        );
        push_authored(&mut sheet, authored_color(Color::BLACK), precedence(2, 0));
        let parent = parent_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0));

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn source_order_wins_within_same_layer() {
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0)),
            precedence(7, 0),
        );
        push_authored(&mut sheet, authored_color(Color::BLACK), precedence(7, 1));
        let parent = parent_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0));

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn authored_specificity_wins_within_same_layer_before_source_order() {
        let mut sheet = Sheet::new();
        sheet
            .push_authored_rule(
                Selector::tag("button").unwrap(),
                authored_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0)),
                precedence(7, 10).with_specificity(SelectorSpecificity::new(0, 0, 1)),
            )
            .unwrap();
        sheet
            .push_authored_rule(
                Selector::tag("button").unwrap(),
                authored_color(Color::BLACK),
                precedence(7, 1).with_specificity(SelectorSpecificity::new(0, 1, 0)),
            )
            .unwrap();

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn inherit_uses_parent_value() {
        let parent = parent_color(Color::BLACK);
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_keyword(Property::Color, CssWideKeyword::Inherit),
            precedence(1, 0),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn initial_uses_property_default() {
        let mut parent = Resolved::new();
        parent
            .apply(
                &Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                    .unwrap(),
                None,
            )
            .unwrap();
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_keyword(Property::Color, CssWideKeyword::Initial),
            precedence(1, 0),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn unset_inherits_inherited_properties_and_initializes_non_inherited_properties() {
        let mut parent = Resolved::new();
        parent
            .apply(
                &Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                    .unwrap()
                    .try_set(Property::Width, Value::Length(Length::Px(88.0)))
                    .unwrap(),
                None,
            )
            .unwrap();
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_keyword(Property::Color, CssWideKeyword::Unset),
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            authored_keyword(Property::Width, CssWideKeyword::Unset),
            precedence(1, 1),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(
            resolved.text_color(),
            &StyleColor::rgba(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(resolved.width(), Length::Auto);
    }

    #[test]
    fn revert_layer_uses_lower_layer_candidate() {
        let mut sheet = Sheet::new();
        push_authored(&mut sheet, authored_color(Color::BLACK), precedence(1, 0));
        push_authored(
            &mut sheet,
            authored_keyword(Property::Color, CssWideKeyword::RevertLayer),
            precedence(2, 0),
        );
        let parent = parent_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0));

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn revert_layer_ignores_same_layer_earlier_source_order() {
        let mut sheet = Sheet::new();
        push_authored(&mut sheet, authored_color(Color::BLACK), precedence(1, 0));
        push_authored(
            &mut sheet,
            authored_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0)),
            precedence(2, 0),
        );
        push_authored(
            &mut sheet,
            authored_keyword(Property::Color, CssWideKeyword::RevertLayer),
            precedence(2, 1),
        );
        let parent = parent_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0));

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn revert_layer_resolves_as_unset_without_lower_layer() {
        let mut parent = Resolved::new();
        parent
            .apply(
                &Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                    .unwrap()
                    .try_set(Property::Width, Value::Length(Length::Px(88.0)))
                    .unwrap(),
                None,
            )
            .unwrap();
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_keyword(Property::Color, CssWideKeyword::RevertLayer),
            precedence(2, 0),
        );
        push_authored(
            &mut sheet,
            authored_keyword(Property::Width, CssWideKeyword::RevertLayer),
            precedence(2, 1),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(
            resolved.text_color(),
            &StyleColor::rgba(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(resolved.width(), Length::Auto);
    }

    #[test]
    fn child_styles_inherit_custom_properties_from_parent_resolved() {
        let name = custom_name("--brand");
        let parent = parent_custom_color("--brand", Color::raw_rgba(0.2, 0.3, 0.4, 1.0));
        let sheet = Sheet::new();

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(
            resolved.custom_property(&name).unwrap().authored().as_css(),
            "Color { r: 0.2, g: 0.3, b: 0.4, a: 1.0 }"
        );
    }

    #[test]
    fn authored_custom_property_overrides_parent_custom_property() {
        let name = custom_name("--brand");
        let parent = parent_custom_color("--brand", Color::BLACK);
        let mut sheet = Sheet::new();
        push_custom_color(
            &mut sheet,
            "--brand",
            Color::raw_rgba(0.8, 0.1, 0.1, 1.0),
            precedence(1, 0),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(
            resolved.custom_property(&name).unwrap().authored().as_css(),
            "Color { r: 0.8, g: 0.1, b: 0.1, a: 1.0 }"
        );
    }

    #[test]
    fn initial_clears_custom_property_so_variable_uses_fallback() {
        let name = custom_name("--brand");
        let parent = parent_custom_color("--brand", Color::BLACK);
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_keyword_declarations(name.clone(), CssWideKeyword::Initial),
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                name.clone(),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(1, 1),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.custom_property(&name), None);
        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
    }

    #[test]
    fn unset_inherits_custom_property_from_parent() {
        let name = custom_name("--brand");
        let parent = parent_custom_color("--brand", Color::raw_rgba(0.2, 0.3, 0.4, 1.0));
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_keyword_declarations(name.clone(), CssWideKeyword::Unset),
            precedence(1, 0),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(
            resolved.custom_property(&name).unwrap().authored().as_css(),
            "Color { r: 0.2, g: 0.3, b: 0.4, a: 1.0 }"
        );
    }

    #[test]
    fn revert_layer_on_custom_property_uses_lower_layer_candidate() {
        let name = custom_name("--brand");
        let mut sheet = Sheet::new();
        push_custom_color(&mut sheet, "--brand", Color::BLACK, precedence(1, 0));
        push_authored(
            &mut sheet,
            custom_keyword_declarations(name.clone(), CssWideKeyword::RevertLayer),
            precedence(2, 0),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(
            resolved.custom_property(&name).unwrap().authored().as_css(),
            "Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }"
        );
    }

    #[test]
    fn revert_layer_on_custom_property_without_lower_layer_resolves_as_unset() {
        let name = custom_name("--brand");
        let parent = parent_custom_color("--brand", Color::raw_rgba(0.3, 0.4, 0.5, 1.0));
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_keyword_declarations(name.clone(), CssWideKeyword::RevertLayer),
            precedence(2, 0),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(
            resolved.custom_property(&name).unwrap().authored().as_css(),
            "Color { r: 0.3, g: 0.4, b: 0.5, a: 1.0 }"
        );
    }

    #[test]
    fn revert_layer_on_custom_property_honors_lower_layer_clear_result() {
        let name = custom_name("--brand");
        let parent = parent_custom_color("--brand", Color::BLACK);
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_keyword_declarations(name.clone(), CssWideKeyword::Initial),
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            custom_keyword_declarations(name.clone(), CssWideKeyword::RevertLayer),
            precedence(2, 0),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                name.clone(),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(2, 1),
        );

        let resolved = resolve_child(sheet, Some(&parent));

        assert_eq!(resolved.custom_property(&name), None);
        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
    }

    #[test]
    fn variable_dependent_color_resolves_through_typed_custom_property() {
        let name = custom_name("--brand");
        let mut sheet = Sheet::new();
        push_custom_color(
            &mut sheet,
            "--brand",
            Color::raw_rgba(0.9, 0.2, 0.1, 1.0),
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                name,
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(1, 1),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(
            resolved.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.9, 0.2, 0.1, 1.0))
        );
    }

    #[test]
    fn custom_property_dependencies_expose_only_valid_primary_reference_when_fallback_is_unused() {
        let mut sheet = Sheet::new();
        push_custom_color(&mut sheet, "--brand", Color::BLACK, precedence(1, 0));
        push_custom_color(
            &mut sheet,
            "--fallback",
            Color::TRANSPARENT,
            precedence(1, 1),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                custom_name("--brand"),
                Some(VariableExpression::Reference(VariableReference::new(
                    custom_name("--fallback"),
                    None,
                ))),
            ),
            precedence(1, 2),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
        assert_eq!(
            dependency_names_for_property(&resolved, Property::Color),
            ["--brand"]
        );
        assert_eq!(
            resolved
                .custom_property_dependencies()
                .properties_for_custom_property(&custom_name("--brand"))
                .collect::<Vec<_>>(),
            [Property::Color]
        );
    }

    #[test]
    fn custom_property_dependencies_include_fallback_reference_when_primary_is_untyped() {
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_value_declarations(
                custom_name("--brand"),
                CustomPropertyValue::new(AuthoredTokens::new("not typed"), []),
            ),
            precedence(1, 0),
        );
        push_custom_color(
            &mut sheet,
            "--fallback",
            Color::TRANSPARENT,
            precedence(1, 1),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                custom_name("--brand"),
                Some(VariableExpression::Reference(VariableReference::new(
                    custom_name("--fallback"),
                    None,
                ))),
            ),
            precedence(1, 2),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
        assert_eq!(
            dependency_names_for_property(&resolved, Property::Color),
            ["--brand", "--fallback"]
        );
    }

    #[test]
    fn custom_property_dependencies_include_fallback_reference_when_primary_is_missing() {
        let mut sheet = Sheet::new();
        push_custom_color(
            &mut sheet,
            "--fallback",
            Color::TRANSPARENT,
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                custom_name("--brand"),
                Some(VariableExpression::Reference(VariableReference::new(
                    custom_name("--fallback"),
                    None,
                ))),
            ),
            precedence(1, 1),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
        assert_eq!(
            dependency_names_for_property(&resolved, Property::Color),
            ["--brand", "--fallback"]
        );
    }

    #[test]
    fn variable_dependent_color_uses_fallback_when_custom_property_is_missing() {
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            variable_color_declarations(
                custom_name("--brand"),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(1, 0),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
    }

    #[test]
    fn variable_dependent_color_without_fallback_resolves_as_unset_when_missing() {
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            variable_color_declarations(custom_name("--brand"), None),
            precedence(1, 0),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn resolve_with_diagnostics_reports_missing_custom_property() {
        let source = StyleSourceId::try_new(11).unwrap();
        let missing = CustomPropertyName::try_new("--missing").unwrap();
        let declarations = variable_color_declarations(missing.clone(), None).with_source(source);
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        sheet
            .push_authored_rule(
                Selector::tag("button").unwrap(),
                declarations,
                precedence(0, 0),
            )
            .unwrap();
        let mut resolver = Resolver::new(sheet);

        let output = resolver
            .resolve_with_diagnostics(Context::new(&tree, 0))
            .unwrap();

        assert_eq!(
            output.resolved().text_color(),
            &StyleColor::rgba(Color::BLACK)
        );
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].reason(),
            &InvalidAtComputedValueReason::MissingCustomProperty(missing)
        );
        assert_eq!(output.diagnostics()[0].source(), Some(source));
    }

    #[test]
    fn resolve_with_diagnostics_reports_missing_typed_custom_property_value() {
        let source = StyleSourceId::try_new(12).unwrap();
        let name = custom_name("--brand");
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_value_declarations(
                name.clone(),
                CustomPropertyValue::new(AuthoredTokens::new("not typed"), []),
            ),
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(name.clone(), None).with_source(source),
            precedence(1, 1),
        );
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut resolver = Resolver::new(sheet);

        let output = resolver
            .resolve_with_diagnostics(Context::new(&tree, 0))
            .unwrap();

        assert_eq!(
            output.resolved().text_color(),
            &StyleColor::rgba(Color::BLACK)
        );
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].reason(),
            &InvalidAtComputedValueReason::MissingTypedCustomPropertyValue(name, Property::Color)
        );
        assert_eq!(output.diagnostics()[0].source(), Some(source));
    }

    #[test]
    fn resolve_with_diagnostics_suppresses_missing_custom_property_when_fallback_is_valid() {
        let source = StyleSourceId::try_new(13).unwrap();
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            variable_color_declarations(
                custom_name("--brand"),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            )
            .with_source(source),
            precedence(1, 0),
        );
        let mut resolver = Resolver::new(sheet);

        let output = resolver
            .resolve_with_diagnostics(Context::new(&tree, 0))
            .unwrap();

        assert_eq!(
            output.resolved().text_color(),
            &StyleColor::rgba(Color::TRANSPARENT)
        );
        assert!(output.diagnostics().is_empty());
    }

    #[test]
    fn resolve_with_diagnostics_recomputes_diagnostics_when_resolved_style_is_cached() {
        let source = StyleSourceId::try_new(14).unwrap();
        let missing = custom_name("--missing");
        let declarations = variable_color_declarations(missing.clone(), None).with_source(source);
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        sheet
            .push_authored_rule(
                Selector::tag("button").unwrap(),
                declarations,
                precedence(0, 0),
            )
            .unwrap();
        let mut resolver = Resolver::new(sheet);

        let cached = resolver.resolve(Context::new(&tree, 0)).unwrap();
        let output = resolver
            .resolve_with_diagnostics(Context::new(&tree, 0))
            .unwrap();

        assert_eq!(cached.text_color(), &StyleColor::rgba(Color::BLACK));
        assert_eq!(
            output.resolved().text_color(),
            &StyleColor::rgba(Color::BLACK)
        );
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].reason(),
            &InvalidAtComputedValueReason::MissingCustomProperty(missing)
        );
        assert_eq!(resolver.cache_hits(), 0);
    }

    #[test]
    fn existing_resolve_api_still_returns_only_resolved_style() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut resolver = Resolver::new(Sheet::new());

        let resolved = resolver.resolve(Context::new(&tree, 0)).unwrap();

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn untyped_custom_property_uses_variable_fallback_or_unset_for_target_property() {
        let name = custom_name("--brand");
        let mut fallback_sheet = Sheet::new();
        push_authored(
            &mut fallback_sheet,
            custom_value_declarations(
                name.clone(),
                CustomPropertyValue::new(AuthoredTokens::new("not typed"), []),
            ),
            precedence(1, 0),
        );
        push_authored(
            &mut fallback_sheet,
            variable_color_declarations(
                name.clone(),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(1, 1),
        );

        let fallback_resolved = resolve_child(fallback_sheet, None);

        assert_eq!(
            fallback_resolved.text_color(),
            &StyleColor::rgba(Color::TRANSPARENT)
        );

        let mut unset_sheet = Sheet::new();
        push_authored(
            &mut unset_sheet,
            custom_value_declarations(
                name.clone(),
                CustomPropertyValue::new(AuthoredTokens::new("not typed"), []),
            ),
            precedence(1, 0),
        );
        push_authored(
            &mut unset_sheet,
            variable_color_declarations(name, None),
            precedence(1, 1),
        );

        let unset_resolved = resolve_child(unset_sheet, None);

        assert_eq!(unset_resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn fallback_only_custom_property_self_reference_does_not_create_cycle() {
        let a = custom_name("--a");
        let b = custom_name("--b");
        let a_color = Color::raw_rgba(0.1, 0.6, 0.3, 1.0);
        let b_fallback_reference = VariableReference::new(b.clone(), None);
        let b_expression = VariableExpression::Reference(VariableReference::new(
            a.clone(),
            Some(VariableFallback::new(
                AuthoredTokens::new("var(--b)"),
                VariableExpression::Reference(b_fallback_reference),
            )),
        ));
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            custom_value_declarations(
                a,
                CustomPropertyValue::new(AuthoredTokens::new("green"), [])
                    .try_with_typed_value(
                        Property::Color,
                        VariableExpression::Value(Value::StyleColor(StyleColor::rgba(a_color))),
                    )
                    .unwrap(),
            ),
            precedence(1, 0),
        );
        push_authored(
            &mut sheet,
            custom_value_declarations(
                b.clone(),
                CustomPropertyValue::new(
                    AuthoredTokens::new("var(--a, var(--b))"),
                    [match &b_expression {
                        VariableExpression::Reference(reference) => reference.clone(),
                        _ => unreachable!("test expression is a reference"),
                    }],
                )
                .try_with_typed_value(Property::Color, b_expression)
                .unwrap(),
            ),
            precedence(1, 1),
        );
        push_authored(
            &mut sheet,
            variable_color_declarations(
                b.clone(),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(1, 2),
        );

        let resolved = resolve_child(sheet, None);

        assert_eq!(resolved.text_color(), &StyleColor::rgba(a_color));
        assert!(
            !resolved
                .custom_property_resolution(&b)
                .unwrap()
                .is_invalid()
        );
    }

    #[test]
    fn custom_property_cycle_uses_fallback_or_unset_without_unbounded_recursion() {
        let a = custom_name("--a");
        let b = custom_name("--b");
        let a_reference = VariableReference::new(b.clone(), None);
        let b_reference = VariableReference::new(a.clone(), None);
        let mut fallback_sheet = Sheet::new();
        push_authored(
            &mut fallback_sheet,
            custom_value_declarations(
                a.clone(),
                CustomPropertyValue::new(AuthoredTokens::new("var(--b)"), [a_reference.clone()])
                    .try_with_typed_value(
                        Property::Color,
                        VariableExpression::Reference(a_reference),
                    )
                    .unwrap(),
            ),
            precedence(1, 0),
        );
        push_authored(
            &mut fallback_sheet,
            custom_value_declarations(
                b.clone(),
                CustomPropertyValue::new(AuthoredTokens::new("var(--a)"), [b_reference.clone()])
                    .try_with_typed_value(
                        Property::Color,
                        VariableExpression::Reference(b_reference),
                    )
                    .unwrap(),
            ),
            precedence(1, 1),
        );
        push_authored(
            &mut fallback_sheet,
            variable_color_declarations(
                a.clone(),
                Some(VariableExpression::Value(Value::StyleColor(
                    StyleColor::rgba(Color::TRANSPARENT),
                ))),
            ),
            precedence(1, 2),
        );

        let fallback_resolved = resolve_child(fallback_sheet, None);

        assert_eq!(
            fallback_resolved.text_color(),
            &StyleColor::rgba(Color::TRANSPARENT)
        );
        assert!(
            fallback_resolved
                .custom_property_resolution(&a)
                .unwrap()
                .is_invalid()
        );

        let mut unset_sheet = Sheet::new();
        let a_reference = VariableReference::new(b.clone(), None);
        let b_reference = VariableReference::new(a.clone(), None);
        push_authored(
            &mut unset_sheet,
            custom_value_declarations(
                a.clone(),
                CustomPropertyValue::new(AuthoredTokens::new("var(--b)"), [a_reference.clone()])
                    .try_with_typed_value(
                        Property::Color,
                        VariableExpression::Reference(a_reference),
                    )
                    .unwrap(),
            ),
            precedence(1, 0),
        );
        push_authored(
            &mut unset_sheet,
            custom_value_declarations(
                b,
                CustomPropertyValue::new(AuthoredTokens::new("var(--a)"), [b_reference.clone()])
                    .try_with_typed_value(
                        Property::Color,
                        VariableExpression::Reference(b_reference),
                    )
                    .unwrap(),
            ),
            precedence(1, 1),
        );
        push_authored(
            &mut unset_sheet,
            variable_color_declarations(a, None),
            precedence(1, 2),
        );

        let unset_resolved = resolve_child(unset_sheet, None);

        assert_eq!(unset_resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn local_declarations_still_override_sheet_rules() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            authored_width(Length::Px(24.0)),
            precedence(2, 0),
        );
        let local = Declarations::new()
            .try_set(Property::Width, Value::Length(Length::Px(48.0)))
            .unwrap();
        let mut resolver = Resolver::new(sheet);

        let resolved = resolver
            .resolve(Context::new(&tree, 0).local(&local))
            .unwrap();

        assert_eq!(resolved.width(), Length::Px(48.0));
    }

    #[test]
    fn local_and_animated_declarations_override_variable_dependent_sheet_resolution() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        push_custom_color(&mut sheet, "--brand", Color::BLACK, precedence(1, 0));
        push_authored(
            &mut sheet,
            variable_color_declarations(custom_name("--brand"), None),
            precedence(1, 1),
        );
        let local = Declarations::new()
            .try_concrete_text_color(Color::raw_rgba(0.4, 0.4, 0.4, 1.0))
            .unwrap();
        let animated = Declarations::new()
            .try_concrete_text_color(Color::TRANSPARENT)
            .unwrap();
        let mut resolver = Resolver::new(sheet);

        let resolved = resolver
            .resolve(Context::new(&tree, 0).local(&local).animated(&animated))
            .unwrap();

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
    }

    #[test]
    fn resolver_defaults_to_element_bucket() {
        let tree = TestTree::new(vec![TestNode::new(0, "button").class("badge")]);
        let sheet = Sheet::new()
            .rule(
                Selector::class("badge").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
                    .unwrap(),
            )
            .targeted_rule(
                RuleTarget::new(Selector::class("badge").unwrap(), StyleBucket::Before),
                Declarations::new()
                    .try_concrete_text_color(Color::BLACK)
                    .unwrap(),
            );
        let mut resolver = Resolver::new(sheet);

        let resolved = resolver.resolve(Context::new(&tree, 0)).unwrap();

        assert_eq!(
            resolved.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
        );
    }

    #[test]
    fn resolver_applies_only_requested_style_bucket() {
        let tree = TestTree::new(vec![TestNode::new(0, "button").class("badge")]);
        let sheet = Sheet::new()
            .rule(
                Selector::class("badge").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
                    .unwrap(),
            )
            .targeted_rule(
                RuleTarget::new(Selector::class("badge").unwrap(), StyleBucket::Before),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.8, 0.7, 0.6, 1.0))
                    .unwrap(),
            );
        let mut resolver = Resolver::new(sheet);

        let element = resolver.resolve(Context::new(&tree, 0)).unwrap();
        let before = resolver
            .resolve(Context::new(&tree, 0).style_bucket(StyleBucket::Before))
            .unwrap();

        assert_eq!(
            element.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
        );
        assert_eq!(
            before.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.8, 0.7, 0.6, 1.0))
        );
    }

    #[test]
    fn resolver_cache_key_includes_style_bucket() {
        let tree = TestTree::new(vec![TestNode::new(0, "button").class("badge")]);
        let sheet = Sheet::new()
            .rule(
                Selector::class("badge").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
                    .unwrap(),
            )
            .targeted_rule(
                RuleTarget::new(Selector::class("badge").unwrap(), StyleBucket::Before),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.8, 0.7, 0.6, 1.0))
                    .unwrap(),
            );
        let mut resolver = Resolver::new(sheet);

        let element = resolver.resolve(Context::new(&tree, 0)).unwrap();
        let before = resolver
            .resolve(Context::new(&tree, 0).style_bucket(StyleBucket::Before))
            .unwrap();
        let before_again = resolver
            .resolve(Context::new(&tree, 0).style_bucket(StyleBucket::Before))
            .unwrap();

        assert_eq!(
            element.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
        );
        assert_eq!(
            before.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.8, 0.7, 0.6, 1.0))
        );
        assert_eq!(
            before_again.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.8, 0.7, 0.6, 1.0))
        );
        assert_eq!(resolver.cache_hits(), 1);
    }

    #[test]
    fn resolver_cache_key_distinguishes_condition_facts() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let query = MediaQueryList::try_new([MediaQuery::Condition(MediaCondition::Feature(
            MediaFeatureQuery::Width(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(640.0, QueryLengthUnit::Px).unwrap(),
            )),
        ))])
        .unwrap();
        let sheet = Sheet::new().conditional_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                .unwrap(),
            [Condition::media(query)],
        );
        let mut resolver = Resolver::new(sheet);

        let wide = resolver
            .resolve(
                Context::new(&tree, 0).media_environment(
                    MediaEnvironment::new()
                        .width(QueryLength::try_new(800.0, QueryLengthUnit::Px).unwrap()),
                ),
            )
            .unwrap();
        let narrow = resolver
            .resolve(
                Context::new(&tree, 0).media_environment(
                    MediaEnvironment::new()
                        .width(QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap()),
                ),
            )
            .unwrap();

        assert_ne!(wide.text_color(), narrow.text_color());
        assert_eq!(resolver.cache_hits(), 0);
    }

    #[test]
    fn resolver_cache_key_distinguishes_local_and_animated_overlays() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut resolver = Resolver::new(Sheet::new());
        let local = Declarations::new()
            .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
            .unwrap();
        let animated = Declarations::new()
            .try_concrete_text_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
            .unwrap();

        let local_resolved = resolver
            .resolve(Context::new(&tree, 0).local(&local))
            .unwrap();
        let animated_resolved = resolver
            .resolve(Context::new(&tree, 0).animated(&animated))
            .unwrap();

        assert_ne!(local_resolved.text_color(), animated_resolved.text_color());
        assert_eq!(resolver.cache_hits(), 0);
    }

    #[test]
    fn pseudo_bucket_inherits_from_supplied_parent_style() {
        let tree = TestTree::new(vec![TestNode::new(0, "button").class("badge")]);
        let sheet = Sheet::new().targeted_rule(
            RuleTarget::new(Selector::class("badge").unwrap(), StyleBucket::Before),
            Declarations::new()
                .try_set(Property::Width, Value::Length(Length::Px(32.0)))
                .unwrap(),
        );
        let parent = parent_color(Color::raw_rgba(0.4, 0.5, 0.6, 1.0));
        let mut resolver = Resolver::new(sheet);

        let before = resolver
            .resolve(
                Context::new(&tree, 0)
                    .style_bucket(StyleBucket::Before)
                    .parent(&parent),
            )
            .unwrap();

        assert_eq!(
            before.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.4, 0.5, 0.6, 1.0))
        );
        assert_eq!(before.width(), Length::Px(32.0));
    }

    #[test]
    fn local_and_animated_overlays_apply_to_requested_bucket() {
        let tree = TestTree::new(vec![TestNode::new(0, "button").class("badge")]);
        let sheet = Sheet::new()
            .rule(
                Selector::class("badge").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.1, 0.2, 0.3, 1.0))
                    .unwrap(),
            )
            .targeted_rule(
                RuleTarget::new(Selector::class("badge").unwrap(), StyleBucket::Before),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.4, 0.5, 0.6, 1.0))
                    .unwrap(),
            );
        let local = Declarations::new()
            .try_concrete_text_color(Color::raw_rgba(0.7, 0.8, 0.9, 1.0))
            .unwrap();
        let animated = Declarations::new()
            .try_concrete_text_color(Color::TRANSPARENT)
            .unwrap();
        let mut resolver = Resolver::new(sheet);

        let before = resolver
            .resolve(
                Context::new(&tree, 0)
                    .style_bucket(StyleBucket::Before)
                    .local(&local)
                    .animated(&animated),
            )
            .unwrap();

        assert_eq!(before.text_color(), &StyleColor::rgba(Color::TRANSPARENT));
    }

    #[test]
    fn parent_custom_property_changes_affect_cache_keys() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        push_authored(
            &mut sheet,
            variable_color_declarations(custom_name("--brand"), None),
            precedence(1, 0),
        );
        let black_parent = parent_custom_color("--brand", Color::BLACK);
        let transparent_parent = parent_custom_color("--brand", Color::TRANSPARENT);
        let mut resolver = Resolver::new(sheet);

        let black_resolved = resolver
            .resolve(Context::new(&tree, 0).parent(&black_parent))
            .unwrap();
        let transparent_resolved = resolver
            .resolve(Context::new(&tree, 0).parent(&transparent_parent))
            .unwrap();

        assert_eq!(black_resolved.text_color(), &StyleColor::rgba(Color::BLACK));
        assert_eq!(
            transparent_resolved.text_color(),
            &StyleColor::rgba(Color::TRANSPARENT)
        );
        assert_eq!(resolver.cache_hits(), 0);
    }

    #[test]
    fn legacy_sheet_rules_keep_flat_source_order() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        sheet.push_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                .unwrap(),
        );
        sheet.push_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );
        let mut resolver = Resolver::new(sheet);

        let resolved = resolver.resolve(Context::new(&tree, 0)).unwrap();

        assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    #[test]
    fn resolver_cache_key_distinguishes_selector_scope_anchor() {
        let tree = TestTree::new(vec![
            TestNode::new(0, "root").children([1, 2]),
            TestNode::new(1, "section").class("scope").children([3]),
            TestNode::new(2, "section").class("other").children([4]),
            TestNode::new(3, "button"),
            TestNode::new(4, "button"),
        ]);
        let selector = Selector::complex([
            ComplexSelectorPart::root(Selector::compound().scope_anchor().class("scope").unwrap()),
            ComplexSelectorPart::related(
                Combinator::Descendant,
                Selector::compound().tag("button").unwrap(),
            ),
        ])
        .unwrap();
        let sheet = Sheet::new().rule(
            selector,
            Declarations::new()
                .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                .unwrap(),
        );
        let mut resolver = Resolver::new(sheet);

        let scoped = resolver
            .resolve(Context::new(&tree, 3).selector_root(0).selector_scope(1))
            .unwrap();
        let wrong_scope = resolver
            .resolve(Context::new(&tree, 3).selector_root(0).selector_scope(2))
            .unwrap();

        assert_ne!(scoped.text_color(), wrong_scope.text_color());
        assert_eq!(resolver.cache_hits(), 0);
    }

    #[test]
    fn resolver_applies_scope_anchor_rules_only_with_matching_selector_scope() {
        let tree = TestTree::new(vec![
            TestNode::new(0, "root").children([1, 2]),
            TestNode::new(1, "section").class("scope").children([3]),
            TestNode::new(2, "section").class("other").children([4]),
            TestNode::new(3, "button"),
            TestNode::new(4, "button"),
        ]);
        let selector = Selector::complex([
            ComplexSelectorPart::root(Selector::compound().scope_anchor().class("scope").unwrap()),
            ComplexSelectorPart::related(
                Combinator::Descendant,
                Selector::compound().tag("button").unwrap(),
            ),
        ])
        .unwrap();
        let sheet = Sheet::new().rule(
            selector,
            Declarations::new()
                .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                .unwrap(),
        );
        let mut resolver = Resolver::new(sheet);

        let scoped = resolver
            .resolve(Context::new(&tree, 3).selector_root(0).selector_scope(1))
            .unwrap();
        let unscoped = resolver
            .resolve(Context::new(&tree, 4).selector_root(0).selector_scope(1))
            .unwrap();

        assert_eq!(
            scoped.text_color(),
            &StyleColor::rgba(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(unscoped.text_color(), &StyleColor::rgba(Color::BLACK));
    }

    mod scoped_rules {
        use super::*;

        #[test]
        fn scoped_rules_apply_inside_root_until_limit() {
            let tree = TestTree::new(vec![
                TestNode::new(0, "root").children([1]),
                TestNode::new(1, "section").class("card").children([2, 3]),
                TestNode::new(2, "button"),
                TestNode::new(3, "footer").class("limit").children([4]),
                TestNode::new(4, "button"),
            ]);
            let scope = RuleScope::try_new(
                Some(ScopeSelectorList::try_new([Selector::class("card").unwrap()]).unwrap()),
                Some(ScopeSelectorList::try_new([Selector::class("limit").unwrap()]).unwrap()),
            )
            .unwrap();
            let mut sheet = Sheet::new();
            sheet
                .push_scoped_rule(
                    scope,
                    Selector::tag("button").unwrap(),
                    Declarations::new()
                        .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                        .unwrap(),
                )
                .unwrap();
            let mut resolver = Resolver::new(sheet);

            let inside = resolver.resolve(Context::new(&tree, 2)).unwrap();
            let outside_limit = resolver.resolve(Context::new(&tree, 4)).unwrap();

            assert_eq!(
                inside.text_color(),
                &StyleColor::rgba(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
            );
            assert_eq!(outside_limit.text_color(), &StyleColor::rgba(Color::BLACK));
        }

        #[test]
        fn scope_anchor_pseudo_class_uses_matched_rule_scope_root() {
            let tree = TestTree::new(vec![
                TestNode::new(0, "root").children([1]),
                TestNode::new(1, "section").class("card").children([2]),
                TestNode::new(2, "button"),
            ]);
            let scope = RuleScope::try_new(
                Some(ScopeSelectorList::try_new([Selector::class("card").unwrap()]).unwrap()),
                None,
            )
            .unwrap();
            let selector = Selector::complex([
                ComplexSelectorPart::root(Selector::compound().scope_anchor()),
                ComplexSelectorPart::related(
                    Combinator::Descendant,
                    Selector::compound().tag("button").unwrap(),
                ),
            ])
            .unwrap();
            let mut sheet = Sheet::new();
            sheet
                .push_scoped_rule(
                    scope,
                    selector,
                    Declarations::new()
                        .try_concrete_text_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
                        .unwrap(),
                )
                .unwrap();
            let mut resolver = Resolver::new(sheet);

            let resolved = resolver.resolve(Context::new(&tree, 2)).unwrap();

            assert_eq!(
                resolved.text_color(),
                &StyleColor::rgba(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
            );
        }

        #[test]
        fn scope_limit_blocks_subject_that_is_also_scope_root() {
            let tree = TestTree::new(vec![
                TestNode::new(0, "root").children([1]),
                TestNode::new(1, "button").class("card").class("limit"),
            ]);
            let scope = RuleScope::try_new(
                Some(ScopeSelectorList::try_new([Selector::class("card").unwrap()]).unwrap()),
                Some(ScopeSelectorList::try_new([Selector::class("limit").unwrap()]).unwrap()),
            )
            .unwrap();
            let mut sheet = Sheet::new();
            sheet
                .push_scoped_rule(
                    scope,
                    Selector::tag("button").unwrap(),
                    Declarations::new()
                        .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                        .unwrap(),
                )
                .unwrap();
            let mut resolver = Resolver::new(sheet);

            let resolved = resolver.resolve(Context::new(&tree, 1)).unwrap();

            assert_eq!(resolved.text_color(), &StyleColor::rgba(Color::BLACK));
        }
    }

    #[derive(Clone, Debug)]
    struct TestNode {
        id: usize,
        tag: StyleTag,
        classes: Vec<StyleClass>,
        children: Vec<usize>,
    }

    impl TestNode {
        fn new(id: usize, tag: &str) -> Self {
            Self {
                id,
                tag: StyleTag::new(tag).unwrap(),
                classes: Vec::new(),
                children: Vec::new(),
            }
        }

        fn class(mut self, class: &str) -> Self {
            self.classes.push(StyleClass::new(class).unwrap());
            self
        }

        fn children(mut self, children: impl IntoIterator<Item = usize>) -> Self {
            self.children = children.into_iter().collect();
            self
        }
    }

    struct TestTree {
        nodes: Vec<TestNode>,
    }

    impl TestTree {
        fn new(nodes: Vec<TestNode>) -> Self {
            Self { nodes }
        }
    }

    impl Tree for TestTree {
        type Id = usize;

        fn version_hint(&self) -> Option<u64> {
            Some(1)
        }

        fn node(&self, id: Self::Id) -> Result<Node<Self::Id>> {
            let node = self
                .nodes
                .get(id)
                .ok_or_else(|| Error::new(ErrorCode::MissingNode, "missing test node"))?;
            Ok(Node {
                id: node.id,
                tag: Some(node.tag.clone()),
                key: None,
                classes: node.classes.clone(),
                attributes: Vec::new(),
                role: StyleRole::default(),
                state: StyleState::default(),
                text: false,
            })
        }

        fn parent(&self, id: Self::Id, _traversal: Traversal) -> Result<Option<Self::Id>> {
            Ok(self
                .nodes
                .iter()
                .find(|node| node.children.contains(&id))
                .map(|node| node.id))
        }

        fn children(
            &self,
            id: Self::Id,
            _traversal: Traversal,
        ) -> Result<impl Iterator<Item = Self::Id> + '_> {
            Ok(self.nodes[id].children.iter().copied())
        }

        fn previous_sibling(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>> {
            let Some(parent) = self.parent(id, traversal)? else {
                return Ok(None);
            };
            let siblings = &self.nodes[parent].children;
            Ok(siblings
                .iter()
                .position(|sibling| *sibling == id)
                .and_then(|index| index.checked_sub(1))
                .map(|index| siblings[index]))
        }
    }
}
