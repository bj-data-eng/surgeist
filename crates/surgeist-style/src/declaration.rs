use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::{
    AlignContent, AnimationDirection, AnimationDirectionList, AnimationFillMode,
    AnimationFillModeList, AnimationItem, AnimationIterationCount, AnimationIterationCountList,
    AnimationIterationNumber, AnimationList, AnimationName, AnimationNameList, AnimationPlayState,
    AnimationPlayStateList, AspectRatio, BasicShape, Border, BorderLineStyle, BorderRadii,
    BorderSide, BorderStyles, BoxDecorationBreak, CalcLength, CalcLengthTerm, ClipPath, Color,
    ColorFunction, ColorInterpolationSpace, ColorMix, Content, ContentItem, ContentVisibility,
    CornerRadius, CounterChanges, CounterFunction, CounterStyle, CountersFunction, Cursor,
    DimensionLength, Display, DurationSeconds, EasingFunction, EasingList, Edges, Filter,
    FilterFunction, Flex, FlexFactor, Font, FontFamilyList, FontFeatureSettings, FontStretch,
    FontVariant, FontWeight, GridAreaPlacement, GridAutoFlow, GridDefinition, GridFlowTolerance,
    GridLine, GridPlacement, GridTemplate, GridTemplateAreas, GridTrackComponent, GridTrackList,
    LayoutPosition, Length, LetterSpacing, ListStyle, ListStyleImage, ListStylePosition,
    ListStyleType, MaxTrackSizing, MinTrackSizing, Opacity, Order, Outline, OutlineStyle,
    OutlineWidth, OverflowWrap, PlaceContentAlignment, PlaceItemsAlignment, PointerEvents,
    Property, RelativeColor, Result, Rotate, Scale, ScrollbarWidth, Shadow, Size, StyleColor,
    SubgridLineNameComponent, SymbolicComponentExpression, TextAlignLast, TextDecoration,
    TextDecorationLine, TextDecorationStyle, TextDecorationThickness, TextIndent, TextOverflow,
    TextSlant, TextTransform, TextWrap, TimeList, TrackRepeatCount, TrackSizing, Transform,
    TransitionItem, TransitionList, TransitionPropertyList, TransitionPropertyTarget, Translate,
    UserSelect, Value, VariableExpression, VariableFallback, VariableReference, VerticalAlign,
    Visibility, WhiteSpace, WordBreak, ZIndex,
    value::{
        BackgroundAttachmentList, BackgroundBox, BackgroundRepeat, BackgroundRepeatList,
        BackgroundSize, BackgroundSizeComponent, BackgroundSizeList, ImageLayer, ImageLayerList,
        KeyframesName, MaskLayerList, Position, PositionComponent, PositionList,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub(crate) property: Property,
    pub(crate) value: Value,
}

impl Declaration {
    #[must_use]
    pub(crate) fn new(property: Property, value: Value) -> Self {
        Self { property, value }
    }

    pub fn try_new(property: Property, value: Value) -> Result<Self> {
        property.validate_value(&value)?;
        Ok(Self::new(property, value))
    }

    #[must_use]
    pub const fn property(&self) -> Property {
        self.property
    }

    #[must_use]
    pub const fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedDeclaration(Declaration);

impl TypedDeclaration {
    #[must_use]
    pub fn width(width: DimensionLength) -> Self {
        Self(Declaration::new(
            Property::Width,
            Value::Length(width.into_length()),
        ))
    }

    #[must_use]
    pub fn opacity(opacity: Opacity) -> Self {
        Self(Declaration::new(
            Property::Opacity,
            Value::Number(opacity.get()),
        ))
    }

    pub fn try_text_color(color: StyleColor) -> Result<Self> {
        Ok(Self(Declaration::try_new(
            Property::Color,
            Value::StyleColor(color),
        )?))
    }

    pub fn try_concrete_text_color(color: Color) -> Result<Self> {
        Self::try_text_color(StyleColor::rgba(color))
    }

    #[must_use]
    pub fn transition_duration(durations: TimeList) -> Self {
        Self(Declaration::new(
            Property::TransitionDuration,
            Value::TimeList(durations),
        ))
    }

    fn into_declaration(self) -> Declaration {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Fingerprint(u64);

impl Fingerprint {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Declarations {
    values: Vec<Declaration>,
}

impl Declarations {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_typed(declarations: impl IntoIterator<Item = TypedDeclaration>) -> Result<Self> {
        let mut values = Self::new();
        for declaration in declarations {
            let Declaration { property, value } = declaration.into_declaration();
            values.try_insert(property, value)?;
        }
        Ok(values)
    }

    fn set(mut self, property: Property, value: Value) -> Self {
        self.insert(property, value);
        self
    }

    pub fn try_set(mut self, property: Property, value: Value) -> Result<Self> {
        self.try_insert(property, value)?;
        Ok(self)
    }

    fn insert(&mut self, property: Property, value: Value) -> &mut Self {
        self.insert_validated(canonical_declarations(property, value));
        self
    }

    fn insert_validated(&mut self, declarations: Vec<Declaration>) -> &mut Self {
        for declaration in declarations {
            self.insert_canonical(declaration.property, declaration.value);
        }
        self
    }

    fn insert_canonical(&mut self, property: Property, value: Value) {
        if let Some(existing) = self
            .values
            .iter_mut()
            .find(|declaration| declaration.property == property)
        {
            existing.value = value;
        } else {
            self.values.push(Declaration::new(property, value));
        }
    }

    pub fn try_insert(&mut self, property: Property, value: Value) -> Result<&mut Self> {
        property.validate_value(&value)?;
        let declarations = canonical_declarations(property, value);
        for declaration in &declarations {
            declaration.property.validate_value(&declaration.value)?;
        }
        Ok(self.insert_validated(declarations))
    }

    #[must_use]
    pub fn get(&self, property: Property) -> Option<&Value> {
        self.values
            .iter()
            .find(|declaration| declaration.property == property)
            .map(|declaration| &declaration.value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Declaration> {
        self.values.iter()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn fingerprint(&self) -> Fingerprint {
        let mut hasher = DefaultHasher::new();
        for declaration in &self.values {
            declaration.property.hash(&mut hasher);
            hash_value(&declaration.value, &mut hasher);
        }
        Fingerprint(hasher.finish())
    }

    pub fn try_bg(self, color: StyleColor) -> Result<Self> {
        self.try_background_color(color)
    }

    pub fn try_background_color(self, color: StyleColor) -> Result<Self> {
        self.try_set(Property::Background, Value::StyleColor(color))
    }

    pub fn try_concrete_background_color(self, color: Color) -> Result<Self> {
        self.try_background_color(StyleColor::rgba(color))
    }

    #[must_use]
    pub fn background_image(self, value: ImageLayerList) -> Self {
        self.set(Property::BackgroundImage, Value::ImageLayerList(value))
    }

    #[must_use]
    pub fn background_position(self, value: PositionList) -> Self {
        self.set(Property::BackgroundPosition, Value::PositionList(value))
    }

    #[must_use]
    pub fn background_size(self, value: BackgroundSizeList) -> Self {
        self.set(Property::BackgroundSize, Value::BackgroundSizeList(value))
    }

    #[must_use]
    pub fn background_repeat(self, value: BackgroundRepeatList) -> Self {
        self.set(
            Property::BackgroundRepeat,
            Value::BackgroundRepeatList(value),
        )
    }

    #[must_use]
    pub fn background_origin(self, value: BackgroundBox) -> Self {
        self.set(Property::BackgroundOrigin, Value::BackgroundBox(value))
    }

    #[must_use]
    pub fn background_clip(self, value: BackgroundBox) -> Self {
        self.set(Property::BackgroundClip, Value::BackgroundBox(value))
    }

    #[must_use]
    pub fn background_attachment(self, value: BackgroundAttachmentList) -> Self {
        self.set(
            Property::BackgroundAttachment,
            Value::BackgroundAttachmentList(value),
        )
    }

    pub fn mask(self, value: MaskLayerList) -> Result<Self> {
        self.try_set(Property::Mask, Value::MaskLayerList(value))
    }

    #[must_use]
    pub fn mask_image(self, value: ImageLayerList) -> Self {
        self.set(Property::MaskImage, Value::ImageLayerList(value))
    }

    #[must_use]
    pub fn mask_position(self, value: PositionList) -> Self {
        self.set(Property::MaskPosition, Value::PositionList(value))
    }

    #[must_use]
    pub fn mask_size(self, value: BackgroundSizeList) -> Self {
        self.set(Property::MaskSize, Value::BackgroundSizeList(value))
    }

    #[must_use]
    pub fn mask_repeat(self, value: BackgroundRepeatList) -> Self {
        self.set(Property::MaskRepeat, Value::BackgroundRepeatList(value))
    }

    pub fn try_text_color(self, color: StyleColor) -> Result<Self> {
        self.try_set(Property::Color, Value::StyleColor(color))
    }

    pub fn try_concrete_text_color(self, color: Color) -> Result<Self> {
        self.try_text_color(StyleColor::rgba(color))
    }

    #[must_use]
    pub fn width(self, width: DimensionLength) -> Self {
        self.set(Property::Width, Value::Length(width.into_length()))
    }

    #[must_use]
    pub fn height(self, height: DimensionLength) -> Self {
        self.set(Property::Height, Value::Length(height.into_length()))
    }

    pub fn try_padding(self, edges: Edges) -> Result<Self> {
        self.try_set(Property::Padding, Value::Edges(edges))
    }

    pub fn try_margin(self, edges: Edges) -> Result<Self> {
        self.try_set(Property::Margin, Value::Edges(edges))
    }

    pub fn try_inset(self, edges: Edges) -> Result<Self> {
        self.try_set(Property::Inset, Value::Edges(edges))
    }

    pub fn try_inset_top(self, value: Length) -> Result<Self> {
        self.try_set(Property::Top, Value::Length(value))
    }

    pub fn try_inset_right(self, value: Length) -> Result<Self> {
        self.try_set(Property::Right, Value::Length(value))
    }

    pub fn try_inset_bottom(self, value: Length) -> Result<Self> {
        self.try_set(Property::Bottom, Value::Length(value))
    }

    pub fn try_inset_left(self, value: Length) -> Result<Self> {
        self.try_set(Property::Left, Value::Length(value))
    }

    pub fn try_margin_top(self, value: Length) -> Result<Self> {
        self.try_set(Property::MarginTop, Value::Length(value))
    }

    pub fn try_margin_right(self, value: Length) -> Result<Self> {
        self.try_set(Property::MarginRight, Value::Length(value))
    }

    pub fn try_margin_bottom(self, value: Length) -> Result<Self> {
        self.try_set(Property::MarginBottom, Value::Length(value))
    }

    pub fn try_margin_left(self, value: Length) -> Result<Self> {
        self.try_set(Property::MarginLeft, Value::Length(value))
    }

    pub fn try_padding_top(self, value: Length) -> Result<Self> {
        self.try_set(Property::PaddingTop, Value::Length(value))
    }

    pub fn try_padding_right(self, value: Length) -> Result<Self> {
        self.try_set(Property::PaddingRight, Value::Length(value))
    }

    pub fn try_padding_bottom(self, value: Length) -> Result<Self> {
        self.try_set(Property::PaddingBottom, Value::Length(value))
    }

    pub fn try_padding_left(self, value: Length) -> Result<Self> {
        self.try_set(Property::PaddingLeft, Value::Length(value))
    }

    pub fn try_radius(self, radii: BorderRadii) -> Result<Self> {
        self.try_set(Property::Radius, Value::BorderRadii(radii))
    }

    pub fn try_shadow(self, shadow: Shadow) -> Result<Self> {
        self.try_set(Property::Shadow, Value::ShadowList(vec![shadow]))
    }

    pub fn try_border_width(self, edges: Edges) -> Result<Self> {
        self.try_set(Property::BorderWidth, Value::Edges(edges))
    }

    pub fn try_border_top_width(self, value: Length) -> Result<Self> {
        self.try_set(Property::BorderTopWidth, Value::Length(value))
    }

    pub fn try_border_right_width(self, value: Length) -> Result<Self> {
        self.try_set(Property::BorderRightWidth, Value::Length(value))
    }

    pub fn try_border_bottom_width(self, value: Length) -> Result<Self> {
        self.try_set(Property::BorderBottomWidth, Value::Length(value))
    }

    pub fn try_border_left_width(self, value: Length) -> Result<Self> {
        self.try_set(Property::BorderLeftWidth, Value::Length(value))
    }

    pub fn try_border(self, value: Border) -> Result<Self> {
        self.try_set(Property::Border, Value::Border(value))
    }

    pub fn try_border_top(self, value: Border) -> Result<Self> {
        self.try_set(Property::BorderTop, Value::Border(value))
    }

    pub fn try_border_right(self, value: Border) -> Result<Self> {
        self.try_set(Property::BorderRight, Value::Border(value))
    }

    pub fn try_border_bottom(self, value: Border) -> Result<Self> {
        self.try_set(Property::BorderBottom, Value::Border(value))
    }

    pub fn try_border_left(self, value: Border) -> Result<Self> {
        self.try_set(Property::BorderLeft, Value::Border(value))
    }

    #[must_use]
    pub fn border_style(self, value: BorderStyles) -> Self {
        self.set(Property::BorderStyle, Value::BorderStyles(value))
    }

    #[must_use]
    pub fn border_top_style(self, value: BorderLineStyle) -> Self {
        self.set(Property::BorderTopStyle, Value::BorderLineStyle(value))
    }

    #[must_use]
    pub fn border_right_style(self, value: BorderLineStyle) -> Self {
        self.set(Property::BorderRightStyle, Value::BorderLineStyle(value))
    }

    #[must_use]
    pub fn border_bottom_style(self, value: BorderLineStyle) -> Self {
        self.set(Property::BorderBottomStyle, Value::BorderLineStyle(value))
    }

    #[must_use]
    pub fn border_left_style(self, value: BorderLineStyle) -> Self {
        self.set(Property::BorderLeftStyle, Value::BorderLineStyle(value))
    }

    pub fn try_border_color(self, value: StyleColor) -> Result<Self> {
        self.try_set(Property::BorderColor, Value::StyleColor(value))
    }

    pub fn try_border_top_color(self, value: StyleColor) -> Result<Self> {
        self.try_set(Property::BorderTopColor, Value::StyleColor(value))
    }

    pub fn try_border_right_color(self, value: StyleColor) -> Result<Self> {
        self.try_set(Property::BorderRightColor, Value::StyleColor(value))
    }

    pub fn try_border_bottom_color(self, value: StyleColor) -> Result<Self> {
        self.try_set(Property::BorderBottomColor, Value::StyleColor(value))
    }

    pub fn try_border_left_color(self, value: StyleColor) -> Result<Self> {
        self.try_set(Property::BorderLeftColor, Value::StyleColor(value))
    }

    pub fn try_border_top_left_radius(self, value: CornerRadius) -> Result<Self> {
        self.try_set(Property::BorderTopLeftRadius, Value::CornerRadius(value))
    }

    pub fn try_border_top_right_radius(self, value: CornerRadius) -> Result<Self> {
        self.try_set(Property::BorderTopRightRadius, Value::CornerRadius(value))
    }

    pub fn try_border_bottom_right_radius(self, value: CornerRadius) -> Result<Self> {
        self.try_set(
            Property::BorderBottomRightRadius,
            Value::CornerRadius(value),
        )
    }

    pub fn try_border_bottom_left_radius(self, value: CornerRadius) -> Result<Self> {
        self.try_set(Property::BorderBottomLeftRadius, Value::CornerRadius(value))
    }

    pub fn try_border_radius(self, value: BorderRadii) -> Result<Self> {
        self.try_set(Property::Radius, Value::BorderRadii(value))
    }

    pub fn try_outline(self, value: Outline) -> Result<Self> {
        self.try_set(Property::Outline, Value::Outline(value))
    }

    pub fn try_outline_color(self, value: StyleColor) -> Result<Self> {
        self.try_set(Property::OutlineColor, Value::StyleColor(value))
    }

    #[must_use]
    pub fn outline_style(self, value: OutlineStyle) -> Self {
        self.set(Property::OutlineStyle, Value::OutlineStyle(value))
    }

    pub fn try_outline_width(self, value: OutlineWidth) -> Result<Self> {
        self.try_set(Property::OutlineWidth, Value::OutlineWidth(value))
    }

    #[must_use]
    pub fn opacity(self, opacity: Opacity) -> Self {
        self.set(Property::Opacity, Value::Number(opacity.get()))
    }

    pub fn try_font_size(self, size: Length) -> Result<Self> {
        self.try_set(Property::FontSize, Value::Length(size))
    }

    pub fn try_font_family(self, family: FontFamilyList) -> Result<Self> {
        self.try_set(Property::FontFamily, Value::FontFamilyList(family))
    }

    pub fn try_line_height(self, line_height: Length) -> Result<Self> {
        self.try_set(Property::LineHeight, Value::Length(line_height))
    }

    #[must_use]
    pub fn font_weight(self, weight: FontWeight) -> Self {
        self.set(Property::FontWeight, Value::FontWeight(weight))
    }

    pub fn try_font_style(self, style: TextSlant) -> Result<Self> {
        self.try_set(Property::FontStyle, Value::TextSlant(style))
    }

    #[must_use]
    pub fn font_stretch(self, stretch: FontStretch) -> Self {
        self.set(Property::FontStretch, Value::FontStretch(stretch))
    }

    #[must_use]
    pub fn font_variant(self, variant: FontVariant) -> Self {
        self.set(Property::FontVariant, Value::FontVariant(variant))
    }

    pub fn try_font_feature_settings(self, settings: FontFeatureSettings) -> Result<Self> {
        self.try_set(
            Property::FontFeatureSettings,
            Value::FontFeatureSettings(settings),
        )
    }

    pub fn try_font(self, font: Font) -> Result<Self> {
        self.try_set(Property::Font, Value::Font(font))
    }

    #[must_use]
    pub fn text_align_last(self, value: TextAlignLast) -> Self {
        self.set(Property::TextAlignLast, Value::TextAlignLast(value))
    }

    pub fn try_text_indent(self, value: TextIndent) -> Result<Self> {
        self.try_set(Property::TextIndent, Value::TextIndent(value))
    }

    #[must_use]
    pub fn vertical_align(self, value: VerticalAlign) -> Self {
        self.set(Property::VerticalAlign, Value::VerticalAlign(value))
    }

    pub fn try_vertical_align(self, value: VerticalAlign) -> Result<Self> {
        self.try_set(Property::VerticalAlign, Value::VerticalAlign(value))
    }

    pub fn try_letter_spacing(self, value: LetterSpacing) -> Result<Self> {
        self.try_set(Property::LetterSpacing, Value::LetterSpacing(value))
    }

    #[must_use]
    pub fn text_transform(self, value: TextTransform) -> Self {
        self.set(Property::TextTransform, Value::TextTransform(value))
    }

    #[must_use]
    pub fn text_wrap(self, value: TextWrap) -> Self {
        self.set(Property::TextWrap, Value::TextWrap(value))
    }

    #[must_use]
    pub fn white_space(self, value: WhiteSpace) -> Self {
        self.set(Property::WhiteSpace, Value::WhiteSpace(value))
    }

    #[must_use]
    pub fn word_break(self, value: WordBreak) -> Self {
        self.set(Property::WordBreak, Value::WordBreak(value))
    }

    #[must_use]
    pub fn overflow_wrap(self, value: OverflowWrap) -> Self {
        self.set(Property::OverflowWrap, Value::OverflowWrap(value))
    }

    #[must_use]
    pub fn text_overflow(self, value: TextOverflow) -> Self {
        self.set(Property::TextOverflow, Value::TextOverflow(value))
    }

    pub fn try_text_decoration(self, value: TextDecoration) -> Result<Self> {
        self.try_set(Property::TextDecoration, Value::TextDecoration(value))
    }

    pub fn try_text_decoration_line(self, value: TextDecorationLine) -> Result<Self> {
        self.try_set(
            Property::TextDecorationLine,
            Value::TextDecorationLine(value),
        )
    }

    pub fn try_text_decoration_color(self, color: StyleColor) -> Result<Self> {
        self.try_set(Property::TextDecorationColor, Value::StyleColor(color))
    }

    #[must_use]
    pub fn text_decoration_style(self, value: TextDecorationStyle) -> Self {
        self.set(
            Property::TextDecorationStyle,
            Value::TextDecorationStyle(value),
        )
    }

    pub fn try_text_decoration_thickness(self, value: TextDecorationThickness) -> Result<Self> {
        self.try_set(
            Property::TextDecorationThickness,
            Value::TextDecorationThickness(value),
        )
    }

    #[must_use]
    pub fn cursor(self, cursor: Cursor) -> Self {
        self.set(Property::Cursor, Value::Cursor(cursor))
    }

    #[must_use]
    pub fn pointer_events(self, pointer_events: PointerEvents) -> Self {
        self.set(
            Property::PointerEvents,
            Value::PointerEvents(pointer_events),
        )
    }

    #[must_use]
    pub fn user_select(self, value: UserSelect) -> Self {
        self.set(Property::UserSelect, Value::UserSelect(value))
    }

    #[must_use]
    pub fn visibility(self, visibility: Visibility) -> Self {
        self.set(Property::Visibility, Value::Visibility(visibility))
    }

    #[must_use]
    pub fn position(self, position: LayoutPosition) -> Self {
        self.set(Property::Position, Value::Position(position))
    }

    #[must_use]
    pub fn z_index(self, z_index: ZIndex) -> Self {
        self.set(Property::ZIndex, Value::ZIndex(z_index))
    }

    #[must_use]
    pub fn scrollbar_width(self, value: ScrollbarWidth) -> Self {
        self.set(Property::ScrollbarWidth, Value::ScrollbarWidth(value))
    }

    #[must_use]
    pub fn content_visibility(self, value: ContentVisibility) -> Self {
        self.set(Property::ContentVisibility, Value::ContentVisibility(value))
    }

    pub fn content(self, value: Content) -> Result<Self> {
        self.try_set(Property::Content, Value::Content(value))
    }

    #[must_use]
    pub fn list_style_type(self, value: ListStyleType) -> Self {
        self.set(Property::ListStyleType, Value::ListStyleType(value))
    }

    #[must_use]
    pub fn list_style_position(self, value: ListStylePosition) -> Self {
        self.set(Property::ListStylePosition, Value::ListStylePosition(value))
    }

    #[must_use]
    pub fn list_style_image(self, value: ListStyleImage) -> Self {
        self.set(Property::ListStyleImage, Value::ListStyleImage(value))
    }

    pub fn list_style(self, value: ListStyle) -> Result<Self> {
        self.try_set(Property::ListStyle, Value::ListStyle(value))
    }

    pub fn counter_reset(self, value: CounterChanges) -> Result<Self> {
        self.try_set(Property::CounterReset, Value::CounterChanges(value))
    }

    pub fn counter_increment(self, value: CounterChanges) -> Result<Self> {
        self.try_set(Property::CounterIncrement, Value::CounterChanges(value))
    }

    pub fn counter_set(self, value: CounterChanges) -> Result<Self> {
        self.try_set(Property::CounterSet, Value::CounterChanges(value))
    }

    #[must_use]
    pub fn order(self, order: Order) -> Self {
        self.set(Property::Order, Value::Order(order))
    }

    pub fn try_flex_grow(self, value: FlexFactor) -> Result<Self> {
        self.try_set(Property::FlexGrow, Value::FlexFactor(value))
    }

    pub fn try_flex_shrink(self, value: FlexFactor) -> Result<Self> {
        self.try_set(Property::FlexShrink, Value::FlexFactor(value))
    }

    pub fn try_flex(self, value: Flex) -> Result<Self> {
        self.try_set(Property::Flex, Value::Flex(value))
    }

    pub fn try_aspect_ratio(self, value: AspectRatio) -> Result<Self> {
        self.try_set(Property::AspectRatio, Value::AspectRatio(value))
    }

    pub fn try_transform(self, transform: Transform) -> Result<Self> {
        self.try_set(Property::Transform, Value::Transform(transform))
    }

    pub fn try_transform_origin(self, origin: Size) -> Result<Self> {
        self.try_set(Property::TransformOrigin, Value::Size(origin))
    }

    #[must_use]
    pub fn box_decoration_break(self, value: BoxDecorationBreak) -> Self {
        self.set(
            Property::BoxDecorationBreak,
            Value::BoxDecorationBreak(value),
        )
    }

    #[must_use]
    pub fn filter(self, value: Filter) -> Self {
        self.set(Property::Filter, Value::Filter(value))
    }

    #[must_use]
    pub fn backdrop_filter(self, value: Filter) -> Self {
        self.set(Property::BackdropFilter, Value::Filter(value))
    }

    #[must_use]
    pub fn clip_path(self, value: ClipPath) -> Self {
        self.set(Property::ClipPath, Value::ClipPath(value))
    }

    #[must_use]
    pub fn translate(self, value: Translate) -> Self {
        self.set(Property::Translate, Value::Translate(value))
    }

    #[must_use]
    pub fn rotate(self, value: Rotate) -> Self {
        self.set(Property::Rotate, Value::Rotate(value))
    }

    #[must_use]
    pub fn scale(self, value: Scale) -> Self {
        self.set(Property::Scale, Value::Scale(value))
    }

    pub fn transition_property(self, properties: TransitionPropertyList) -> Result<Self> {
        self.try_set(
            Property::TransitionProperty,
            Value::TransitionPropertyList(properties),
        )
    }

    pub fn transition_duration(self, durations: TimeList) -> Result<Self> {
        self.try_set(Property::TransitionDuration, Value::TimeList(durations))
    }

    pub fn transition_delay(self, delays: TimeList) -> Result<Self> {
        self.try_set(Property::TransitionDelay, Value::TimeList(delays))
    }

    pub fn transition_timing_function(self, easings: EasingList) -> Result<Self> {
        self.try_set(
            Property::TransitionTimingFunction,
            Value::EasingList(easings),
        )
    }

    pub fn transition(self, transitions: TransitionList) -> Result<Self> {
        self.try_set(Property::Transition, Value::TransitionList(transitions))
    }

    pub fn animation_name(self, names: AnimationNameList) -> Result<Self> {
        self.try_set(Property::AnimationName, Value::AnimationNameList(names))
    }

    pub fn animation_duration(self, durations: TimeList) -> Result<Self> {
        self.try_set(Property::AnimationDuration, Value::TimeList(durations))
    }

    pub fn animation_delay(self, delays: TimeList) -> Result<Self> {
        self.try_set(Property::AnimationDelay, Value::TimeList(delays))
    }

    pub fn animation_timing_function(self, easings: EasingList) -> Result<Self> {
        self.try_set(
            Property::AnimationTimingFunction,
            Value::EasingList(easings),
        )
    }

    pub fn animation_iteration_count(self, values: AnimationIterationCountList) -> Result<Self> {
        self.try_set(
            Property::AnimationIterationCount,
            Value::AnimationIterationCountList(values),
        )
    }

    pub fn animation_direction(self, values: AnimationDirectionList) -> Result<Self> {
        self.try_set(
            Property::AnimationDirection,
            Value::AnimationDirectionList(values),
        )
    }

    pub fn animation_fill_mode(self, values: AnimationFillModeList) -> Result<Self> {
        self.try_set(
            Property::AnimationFillMode,
            Value::AnimationFillModeList(values),
        )
    }

    pub fn animation_play_state(self, values: AnimationPlayStateList) -> Result<Self> {
        self.try_set(
            Property::AnimationPlayState,
            Value::AnimationPlayStateList(values),
        )
    }

    pub fn animation(self, animations: AnimationList) -> Result<Self> {
        self.try_set(Property::Animation, Value::AnimationList(animations))
    }

    #[must_use]
    pub fn display(self, display: Display) -> Self {
        self.set(Property::Display, Value::Display(display))
    }

    pub fn try_grid_template_rows(self, tracks: GridTrackList) -> Result<Self> {
        self.try_set(Property::GridTemplateRows, Value::GridTrackList(tracks))
    }

    pub fn try_grid_template_columns(self, tracks: GridTrackList) -> Result<Self> {
        self.try_set(Property::GridTemplateColumns, Value::GridTrackList(tracks))
    }

    pub fn try_grid_template_areas(self, areas: GridTemplateAreas) -> Result<Self> {
        self.try_set(Property::GridTemplateAreas, Value::GridTemplateAreas(areas))
    }

    pub fn try_grid_template(self, template: GridTemplate) -> Result<Self> {
        self.try_set(Property::GridTemplate, Value::GridTemplate(template))
    }

    pub fn try_grid_auto_rows(self, tracks: GridTrackList) -> Result<Self> {
        self.try_set(Property::GridAutoRows, Value::GridTrackList(tracks))
    }

    pub fn try_grid_auto_columns(self, tracks: GridTrackList) -> Result<Self> {
        self.try_set(Property::GridAutoColumns, Value::GridTrackList(tracks))
    }

    #[must_use]
    pub fn grid_auto_flow(self, flow: GridAutoFlow) -> Self {
        self.set(Property::GridAutoFlow, Value::GridAutoFlow(flow))
    }

    pub fn try_grid_flow_tolerance(self, tolerance: GridFlowTolerance) -> Result<Self> {
        self.try_set(
            Property::GridFlowTolerance,
            Value::GridFlowTolerance(tolerance),
        )
    }

    pub fn try_grid(self, grid: GridDefinition) -> Result<Self> {
        self.try_set(Property::Grid, Value::GridDefinition(grid))
    }

    pub fn try_grid_row_start(self, line: GridLine) -> Result<Self> {
        self.try_set(Property::GridRowStart, Value::GridLine(line))
    }

    pub fn try_grid_row_end(self, line: GridLine) -> Result<Self> {
        self.try_set(Property::GridRowEnd, Value::GridLine(line))
    }

    pub fn try_grid_column_start(self, line: GridLine) -> Result<Self> {
        self.try_set(Property::GridColumnStart, Value::GridLine(line))
    }

    pub fn try_grid_column_end(self, line: GridLine) -> Result<Self> {
        self.try_set(Property::GridColumnEnd, Value::GridLine(line))
    }

    pub fn try_grid_row(self, placement: GridPlacement) -> Result<Self> {
        self.try_set(Property::GridRow, Value::GridPlacement(placement))
    }

    pub fn try_grid_column(self, placement: GridPlacement) -> Result<Self> {
        self.try_set(Property::GridColumn, Value::GridPlacement(placement))
    }

    pub fn try_grid_area(self, area: GridAreaPlacement) -> Result<Self> {
        self.try_set(Property::GridArea, Value::GridAreaPlacement(area))
    }

    #[must_use]
    pub fn place_content(self, value: PlaceContentAlignment) -> Self {
        self.set(Property::PlaceContent, Value::PlaceContentAlignment(value))
    }

    #[must_use]
    pub fn place_items(self, value: PlaceItemsAlignment) -> Self {
        self.set(Property::PlaceItems, Value::PlaceItemsAlignment(value))
    }

    #[must_use]
    pub fn place_self(self, value: PlaceItemsAlignment) -> Self {
        self.set(Property::PlaceSelf, Value::PlaceItemsAlignment(value))
    }

    #[must_use]
    pub fn align_tracks(self, value: AlignContent) -> Self {
        self.set(Property::AlignTracks, Value::AlignContent(value))
    }

    #[must_use]
    pub fn justify_tracks(self, value: AlignContent) -> Self {
        self.set(Property::JustifyTracks, Value::AlignContent(value))
    }

    #[must_use]
    pub fn background(&self) -> Option<&StyleColor> {
        match self.get(Property::Background) {
            Some(Value::StyleColor(color)) => Some(color),
            _ => None,
        }
    }

    #[must_use]
    pub fn padding_edges(&self) -> Option<Edges> {
        edge_values(
            self,
            Property::PaddingTop,
            Property::PaddingRight,
            Property::PaddingBottom,
            Property::PaddingLeft,
        )
    }

    #[must_use]
    pub fn margin_edges(&self) -> Option<Edges> {
        edge_values(
            self,
            Property::MarginTop,
            Property::MarginRight,
            Property::MarginBottom,
            Property::MarginLeft,
        )
    }

    #[must_use]
    pub fn opacity_number(&self) -> Option<f32> {
        match self.get(Property::Opacity) {
            Some(Value::Number(opacity)) => Some(*opacity),
            _ => None,
        }
    }

    #[must_use]
    pub fn font_size_length(&self) -> Option<Length> {
        match self.get(Property::FontSize) {
            Some(Value::Length(size)) => Some(size.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn cursor_kind(&self) -> Option<Cursor> {
        match self.get(Property::Cursor) {
            Some(Value::Cursor(cursor)) => Some(*cursor),
            _ => None,
        }
    }

    #[must_use]
    pub fn pointer_events_kind(&self) -> Option<PointerEvents> {
        match self.get(Property::PointerEvents) {
            Some(Value::PointerEvents(pointer_events)) => Some(*pointer_events),
            _ => None,
        }
    }

    #[must_use]
    pub fn width_length(&self) -> Option<Length> {
        match self.get(Property::Width) {
            Some(Value::Length(length)) => Some(length.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn height_length(&self) -> Option<Length> {
        match self.get(Property::Height) {
            Some(Value::Length(length)) => Some(length.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn border_width_edges(&self) -> Option<Edges> {
        edge_values(
            self,
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
        )
    }

    #[must_use]
    pub fn visibility_state(&self) -> Option<Visibility> {
        match self.get(Property::Visibility) {
            Some(Value::Visibility(visibility)) => Some(*visibility),
            _ => None,
        }
    }

    #[must_use]
    pub fn transform_value(&self) -> Option<&Transform> {
        match self.get(Property::Transform) {
            Some(Value::Transform(transform)) => Some(transform),
            _ => None,
        }
    }

    #[must_use]
    pub fn transform_origin_size(&self) -> Option<Size> {
        match self.get(Property::TransformOrigin) {
            Some(Value::Size(origin)) => Some(origin.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn box_decoration_break_value(&self) -> Option<BoxDecorationBreak> {
        match self.get(Property::BoxDecorationBreak) {
            Some(Value::BoxDecorationBreak(value)) => Some(*value),
            _ => None,
        }
    }

    #[must_use]
    pub fn filter_value(&self) -> Option<&Filter> {
        match self.get(Property::Filter) {
            Some(Value::Filter(filter)) => Some(filter),
            _ => None,
        }
    }

    #[must_use]
    pub fn backdrop_filter_value(&self) -> Option<&Filter> {
        match self.get(Property::BackdropFilter) {
            Some(Value::Filter(filter)) => Some(filter),
            _ => None,
        }
    }

    #[must_use]
    pub fn clip_path_value(&self) -> Option<&ClipPath> {
        match self.get(Property::ClipPath) {
            Some(Value::ClipPath(clip_path)) => Some(clip_path),
            _ => None,
        }
    }

    #[must_use]
    pub fn translate_value(&self) -> Option<&Translate> {
        match self.get(Property::Translate) {
            Some(Value::Translate(translate)) => Some(translate),
            _ => None,
        }
    }

    #[must_use]
    pub fn rotate_value(&self) -> Option<&Rotate> {
        match self.get(Property::Rotate) {
            Some(Value::Rotate(rotate)) => Some(rotate),
            _ => None,
        }
    }

    #[must_use]
    pub fn scale_value(&self) -> Option<&Scale> {
        match self.get(Property::Scale) {
            Some(Value::Scale(scale)) => Some(scale),
            _ => None,
        }
    }

    #[must_use]
    pub fn transition_property_list(&self) -> Option<&TransitionPropertyList> {
        match self.get(Property::TransitionProperty) {
            Some(Value::TransitionPropertyList(properties)) => Some(properties),
            _ => None,
        }
    }

    #[must_use]
    pub fn transition_duration_list(&self) -> Option<&TimeList> {
        match self.get(Property::TransitionDuration) {
            Some(Value::TimeList(durations)) => Some(durations),
            _ => None,
        }
    }

    #[must_use]
    pub fn transition_delay_list(&self) -> Option<&TimeList> {
        match self.get(Property::TransitionDelay) {
            Some(Value::TimeList(delays)) => Some(delays),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_name_list(&self) -> Option<&AnimationNameList> {
        match self.get(Property::AnimationName) {
            Some(Value::AnimationNameList(names)) => Some(names),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_duration_list(&self) -> Option<&TimeList> {
        match self.get(Property::AnimationDuration) {
            Some(Value::TimeList(durations)) => Some(durations),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_delay_list(&self) -> Option<&TimeList> {
        match self.get(Property::AnimationDelay) {
            Some(Value::TimeList(delays)) => Some(delays),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_timing_function_list(&self) -> Option<&EasingList> {
        match self.get(Property::AnimationTimingFunction) {
            Some(Value::EasingList(easings)) => Some(easings),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_iteration_count_list(&self) -> Option<&AnimationIterationCountList> {
        match self.get(Property::AnimationIterationCount) {
            Some(Value::AnimationIterationCountList(values)) => Some(values),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_direction_list(&self) -> Option<&AnimationDirectionList> {
        match self.get(Property::AnimationDirection) {
            Some(Value::AnimationDirectionList(values)) => Some(values),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_fill_mode_list(&self) -> Option<&AnimationFillModeList> {
        match self.get(Property::AnimationFillMode) {
            Some(Value::AnimationFillModeList(values)) => Some(values),
            _ => None,
        }
    }

    #[must_use]
    pub fn animation_play_state_list(&self) -> Option<&AnimationPlayStateList> {
        match self.get(Property::AnimationPlayState) {
            Some(Value::AnimationPlayStateList(values)) => Some(values),
            _ => None,
        }
    }
}

pub(crate) fn canonical_properties(property: Property) -> Vec<Property> {
    match property {
        Property::Inset => vec![
            Property::Top,
            Property::Right,
            Property::Bottom,
            Property::Left,
        ],
        Property::Margin => vec![
            Property::MarginTop,
            Property::MarginRight,
            Property::MarginBottom,
            Property::MarginLeft,
        ],
        Property::Padding => vec![
            Property::PaddingTop,
            Property::PaddingRight,
            Property::PaddingBottom,
            Property::PaddingLeft,
        ],
        Property::Border => vec![
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ],
        Property::BorderColor => vec![
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ],
        Property::BorderWidth => vec![
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
        ],
        Property::BorderStyle => vec![
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
        ],
        Property::BorderTop => vec![
            Property::BorderTopWidth,
            Property::BorderTopStyle,
            Property::BorderTopColor,
        ],
        Property::BorderRight => vec![
            Property::BorderRightWidth,
            Property::BorderRightStyle,
            Property::BorderRightColor,
        ],
        Property::BorderBottom => vec![
            Property::BorderBottomWidth,
            Property::BorderBottomStyle,
            Property::BorderBottomColor,
        ],
        Property::BorderLeft => vec![
            Property::BorderLeftWidth,
            Property::BorderLeftStyle,
            Property::BorderLeftColor,
        ],
        Property::Radius => vec![
            Property::BorderTopLeftRadius,
            Property::BorderTopRightRadius,
            Property::BorderBottomRightRadius,
            Property::BorderBottomLeftRadius,
        ],
        Property::Outline => vec![
            Property::OutlineWidth,
            Property::OutlineStyle,
            Property::OutlineColor,
        ],
        Property::MinSize => vec![Property::MinWidth, Property::MinHeight],
        Property::MaxSize => vec![Property::MaxWidth, Property::MaxHeight],
        Property::Overflow => vec![Property::OverflowX, Property::OverflowY],
        Property::Align => vec![Property::AlignItems, Property::AlignSelf],
        Property::Justify => vec![Property::JustifyItems, Property::JustifySelf],
        Property::PlaceContent => vec![Property::AlignContent, Property::JustifyContent],
        Property::PlaceItems => vec![Property::AlignItems, Property::JustifyItems],
        Property::PlaceSelf => vec![Property::AlignSelf, Property::JustifySelf],
        Property::Flex => vec![
            Property::FlexGrow,
            Property::FlexShrink,
            Property::FlexBasis,
        ],
        Property::Font => vec![
            Property::FontStyle,
            Property::FontVariant,
            Property::FontWeight,
            Property::FontStretch,
            Property::FontSize,
            Property::LineHeight,
            Property::FontFamily,
        ],
        Property::TextDecoration => vec![
            Property::TextDecorationLine,
            Property::TextDecorationColor,
            Property::TextDecorationStyle,
            Property::TextDecorationThickness,
        ],
        Property::ListStyle => vec![
            Property::ListStyleType,
            Property::ListStylePosition,
            Property::ListStyleImage,
        ],
        Property::Gap => vec![Property::RowGap, Property::ColumnGap],
        Property::GridTemplate => vec![
            Property::GridTemplateRows,
            Property::GridTemplateColumns,
            Property::GridTemplateAreas,
        ],
        Property::Grid => vec![
            Property::GridTemplateRows,
            Property::GridTemplateColumns,
            Property::GridTemplateAreas,
            Property::GridAutoRows,
            Property::GridAutoColumns,
            Property::GridAutoFlow,
        ],
        Property::GridRow => vec![Property::GridRowStart, Property::GridRowEnd],
        Property::GridColumn => vec![Property::GridColumnStart, Property::GridColumnEnd],
        Property::GridArea => vec![
            Property::GridRowStart,
            Property::GridColumnStart,
            Property::GridRowEnd,
            Property::GridColumnEnd,
        ],
        Property::Mask => vec![
            Property::MaskImage,
            Property::MaskPosition,
            Property::MaskSize,
            Property::MaskRepeat,
        ],
        Property::Transition => vec![
            Property::TransitionProperty,
            Property::TransitionDuration,
            Property::TransitionDelay,
            Property::TransitionTimingFunction,
        ],
        Property::Animation => vec![
            Property::AnimationName,
            Property::AnimationDuration,
            Property::AnimationDelay,
            Property::AnimationTimingFunction,
            Property::AnimationIterationCount,
            Property::AnimationDirection,
            Property::AnimationFillMode,
            Property::AnimationPlayState,
        ],
        property => vec![property],
    }
}

pub(crate) fn canonical_declarations(property: Property, value: Value) -> Vec<Declaration> {
    match (property, value) {
        (Property::Inset, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Inset),
            Value::Keyword(keyword),
        ),
        (Property::Inset, Value::Edges(edges)) => edge_declarations(
            edges,
            Property::Top,
            Property::Right,
            Property::Bottom,
            Property::Left,
        ),
        (Property::Margin, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Margin),
            Value::Keyword(keyword),
        ),
        (Property::Margin, Value::Edges(edges)) => edge_declarations(
            edges,
            Property::MarginTop,
            Property::MarginRight,
            Property::MarginBottom,
            Property::MarginLeft,
        ),
        (Property::Padding, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Padding),
            Value::Keyword(keyword),
        ),
        (Property::Padding, Value::Edges(edges)) => edge_declarations(
            edges,
            Property::PaddingTop,
            Property::PaddingRight,
            Property::PaddingBottom,
            Property::PaddingLeft,
        ),
        (Property::Border, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Border),
            Value::Keyword(keyword),
        ),
        (Property::Border, Value::Border(border)) => border_declarations(border),
        (Property::BorderColor, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderColor),
            Value::Keyword(keyword),
        ),
        (Property::BorderColor, Value::StyleColor(color)) => vec![
            Declaration::new(Property::BorderTopColor, Value::StyleColor(color.clone())),
            Declaration::new(Property::BorderRightColor, Value::StyleColor(color.clone())),
            Declaration::new(
                Property::BorderBottomColor,
                Value::StyleColor(color.clone()),
            ),
            Declaration::new(Property::BorderLeftColor, Value::StyleColor(color)),
        ],
        (Property::BorderWidth, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderWidth),
            Value::Keyword(keyword),
        ),
        (Property::BorderWidth, Value::Edges(edges)) => edge_declarations(
            edges,
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
        ),
        (Property::BorderStyle, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderStyle),
            Value::Keyword(keyword),
        ),
        (Property::BorderStyle, Value::BorderStyles(styles)) => vec![
            Declaration::new(
                Property::BorderTopStyle,
                Value::BorderLineStyle(styles.top()),
            ),
            Declaration::new(
                Property::BorderRightStyle,
                Value::BorderLineStyle(styles.right()),
            ),
            Declaration::new(
                Property::BorderBottomStyle,
                Value::BorderLineStyle(styles.bottom()),
            ),
            Declaration::new(
                Property::BorderLeftStyle,
                Value::BorderLineStyle(styles.left()),
            ),
        ],
        (Property::BorderTop, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderTop),
            Value::Keyword(keyword),
        ),
        (Property::BorderTop, Value::Border(border)) => {
            border_side_declarations(BorderSide::Top, border)
        }
        (Property::BorderRight, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderRight),
            Value::Keyword(keyword),
        ),
        (Property::BorderRight, Value::Border(border)) => {
            border_side_declarations(BorderSide::Right, border)
        }
        (Property::BorderBottom, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderBottom),
            Value::Keyword(keyword),
        ),
        (Property::BorderBottom, Value::Border(border)) => {
            border_side_declarations(BorderSide::Bottom, border)
        }
        (Property::BorderLeft, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::BorderLeft),
            Value::Keyword(keyword),
        ),
        (Property::BorderLeft, Value::Border(border)) => {
            border_side_declarations(BorderSide::Left, border)
        }
        (Property::Radius, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Radius),
            Value::Keyword(keyword),
        ),
        (Property::Radius, Value::BorderRadii(radii)) => vec![
            Declaration::new(
                Property::BorderTopLeftRadius,
                Value::CornerRadius(radii.top_left().clone()),
            ),
            Declaration::new(
                Property::BorderTopRightRadius,
                Value::CornerRadius(radii.top_right().clone()),
            ),
            Declaration::new(
                Property::BorderBottomRightRadius,
                Value::CornerRadius(radii.bottom_right().clone()),
            ),
            Declaration::new(
                Property::BorderBottomLeftRadius,
                Value::CornerRadius(radii.bottom_left().clone()),
            ),
        ],
        (Property::Outline, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Outline),
            Value::Keyword(keyword),
        ),
        (Property::Outline, Value::Outline(outline)) => outline_declarations(outline),
        (Property::MinSize, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::MinSize),
            Value::Keyword(keyword),
        ),
        (Property::MinSize, value) => vec![
            Declaration::new(Property::MinWidth, value.clone()),
            Declaration::new(Property::MinHeight, value),
        ],
        (Property::MaxSize, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::MaxSize),
            Value::Keyword(keyword),
        ),
        (Property::MaxSize, value) => vec![
            Declaration::new(Property::MaxWidth, value.clone()),
            Declaration::new(Property::MaxHeight, value),
        ],
        (Property::Overflow, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Overflow),
            Value::Keyword(keyword),
        ),
        (Property::Overflow, Value::OverflowAxes(axes)) => vec![
            Declaration::new(Property::OverflowX, Value::Overflow(axes.x)),
            Declaration::new(Property::OverflowY, Value::Overflow(axes.y)),
        ],
        (Property::Overflow, value) => vec![
            Declaration::new(Property::OverflowX, value.clone()),
            Declaration::new(Property::OverflowY, value),
        ],
        (Property::Align, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Align),
            Value::Keyword(keyword),
        ),
        (Property::Align, value) => vec![
            Declaration::new(Property::AlignItems, value.clone()),
            Declaration::new(Property::AlignSelf, value),
        ],
        (Property::Justify, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Justify),
            Value::Keyword(keyword),
        ),
        (Property::Justify, value) => vec![
            Declaration::new(Property::JustifyItems, value.clone()),
            Declaration::new(Property::JustifySelf, value),
        ],
        (Property::PlaceContent, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::PlaceContent),
            Value::Keyword(keyword),
        ),
        (Property::PlaceContent, Value::PlaceContentAlignment(value)) => vec![
            Declaration::new(Property::AlignContent, Value::AlignContent(value.first())),
            Declaration::new(
                Property::JustifyContent,
                Value::AlignContent(value.second()),
            ),
        ],
        (Property::PlaceItems, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::PlaceItems),
            Value::Keyword(keyword),
        ),
        (Property::PlaceItems, Value::PlaceItemsAlignment(value)) => vec![
            Declaration::new(Property::AlignItems, Value::AlignItems(value.first())),
            Declaration::new(Property::JustifyItems, Value::AlignItems(value.second())),
        ],
        (Property::PlaceSelf, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::PlaceSelf),
            Value::Keyword(keyword),
        ),
        (Property::PlaceSelf, Value::PlaceItemsAlignment(value)) => vec![
            Declaration::new(Property::AlignSelf, Value::AlignItems(value.first())),
            Declaration::new(Property::JustifySelf, Value::AlignItems(value.second())),
        ],
        (Property::Flex, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Flex),
            Value::Keyword(keyword),
        ),
        (Property::Flex, Value::Flex(value)) => flex_declarations(value),
        (Property::Font, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Font),
            Value::Keyword(keyword),
        ),
        (Property::Font, Value::Font(font)) => font_declarations(font),
        (Property::TextDecoration, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::TextDecoration),
            Value::Keyword(keyword),
        ),
        (Property::TextDecoration, Value::TextDecoration(value)) => {
            text_decoration_declarations(value)
        }
        (Property::ListStyle, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::ListStyle),
            Value::Keyword(keyword),
        ),
        (Property::ListStyle, Value::ListStyle(value)) => list_style_declarations(value),
        (Property::Gap, Value::Keyword(keyword)) => {
            same_value_declarations(canonical_properties(Property::Gap), Value::Keyword(keyword))
        }
        (Property::Gap, value) => vec![
            Declaration::new(Property::RowGap, value.clone()),
            Declaration::new(Property::ColumnGap, value),
        ],
        (Property::GridTemplate, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::GridTemplate),
            Value::Keyword(keyword),
        ),
        (Property::GridTemplate, Value::GridTemplate(template)) => {
            let GridTemplate {
                rows,
                columns,
                areas,
            } = template;
            vec![
                Declaration::new(Property::GridTemplateRows, Value::GridTrackList(rows)),
                Declaration::new(Property::GridTemplateColumns, Value::GridTrackList(columns)),
                Declaration::new(Property::GridTemplateAreas, Value::GridTemplateAreas(areas)),
            ]
        }
        (Property::Grid, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Grid),
            Value::Keyword(keyword),
        ),
        (Property::Grid, Value::GridDefinition(grid)) => {
            let GridDefinition {
                template,
                auto_rows,
                auto_columns,
                auto_flow,
            } = grid;
            let GridTemplate {
                rows,
                columns,
                areas,
            } = template;
            vec![
                Declaration::new(Property::GridTemplateRows, Value::GridTrackList(rows)),
                Declaration::new(Property::GridTemplateColumns, Value::GridTrackList(columns)),
                Declaration::new(Property::GridTemplateAreas, Value::GridTemplateAreas(areas)),
                Declaration::new(Property::GridAutoRows, Value::GridTrackList(auto_rows)),
                Declaration::new(
                    Property::GridAutoColumns,
                    Value::GridTrackList(auto_columns),
                ),
                Declaration::new(Property::GridAutoFlow, Value::GridAutoFlow(auto_flow)),
            ]
        }
        (Property::GridRow, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::GridRow),
            Value::Keyword(keyword),
        ),
        (Property::GridRow, Value::GridPlacement(placement)) => {
            let GridPlacement { start, end } = placement;
            let end = grid_placement_end_for_shorthand(&start, end);
            vec![
                Declaration::new(Property::GridRowStart, Value::GridLine(start)),
                Declaration::new(Property::GridRowEnd, Value::GridLine(end)),
            ]
        }
        (Property::GridColumn, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::GridColumn),
            Value::Keyword(keyword),
        ),
        (Property::GridColumn, Value::GridPlacement(placement)) => {
            let GridPlacement { start, end } = placement;
            let end = grid_placement_end_for_shorthand(&start, end);
            vec![
                Declaration::new(Property::GridColumnStart, Value::GridLine(start)),
                Declaration::new(Property::GridColumnEnd, Value::GridLine(end)),
            ]
        }
        (Property::GridArea, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::GridArea),
            Value::Keyword(keyword),
        ),
        (Property::GridArea, Value::GridAreaPlacement(area)) => {
            let GridAreaPlacement {
                row_start,
                column_start,
                row_end,
                column_end,
            } = area;
            let column_start = if matches!(column_start, GridLine::Auto) {
                grid_area_omitted_line(&row_start)
            } else {
                column_start
            };
            let row_end = if matches!(row_end, GridLine::Auto) {
                grid_area_omitted_line(&row_start)
            } else {
                row_end
            };
            let column_end = if matches!(column_end, GridLine::Auto) {
                grid_area_omitted_line(&column_start)
            } else {
                column_end
            };
            vec![
                Declaration::new(Property::GridRowStart, Value::GridLine(row_start)),
                Declaration::new(Property::GridColumnStart, Value::GridLine(column_start)),
                Declaration::new(Property::GridRowEnd, Value::GridLine(row_end)),
                Declaration::new(Property::GridColumnEnd, Value::GridLine(column_end)),
            ]
        }
        (Property::Mask, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Mask),
            Value::Keyword(keyword),
        ),
        (Property::Mask, Value::MaskLayerList(layers)) => mask_declarations(layers),
        (Property::Transition, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Transition),
            Value::Keyword(keyword),
        ),
        (Property::Transition, Value::TransitionList(value)) => transition_declarations(value),
        (Property::Animation, Value::Keyword(keyword)) => same_value_declarations(
            canonical_properties(Property::Animation),
            Value::Keyword(keyword),
        ),
        (Property::Animation, Value::AnimationList(value)) => animation_declarations(value),
        (property, value) => vec![Declaration::new(property, value)],
    }
}

fn transition_declarations(value: TransitionList) -> Vec<Declaration> {
    let mut properties = Vec::new();
    let mut durations = Vec::new();
    let mut delays = Vec::new();
    let mut easings = Vec::new();

    for item in value.items() {
        properties.push(
            item.property()
                .cloned()
                .unwrap_or(TransitionPropertyTarget::All),
        );
        durations.push(
            item.duration()
                .unwrap_or_else(|| DurationSeconds::new(0.0).unwrap()),
        );
        delays.push(
            item.delay()
                .unwrap_or_else(|| DurationSeconds::new(0.0).unwrap()),
        );
        easings.push(
            item.timing_function()
                .cloned()
                .unwrap_or(EasingFunction::Ease),
        );
    }

    vec![
        Declaration::new(
            Property::TransitionProperty,
            Value::TransitionPropertyList(TransitionPropertyList::try_new(properties).unwrap()),
        ),
        Declaration::new(
            Property::TransitionDuration,
            Value::TimeList(TimeList::try_new(durations).unwrap()),
        ),
        Declaration::new(
            Property::TransitionDelay,
            Value::TimeList(TimeList::try_new(delays).unwrap()),
        ),
        Declaration::new(
            Property::TransitionTimingFunction,
            Value::EasingList(EasingList::try_new(easings).unwrap()),
        ),
    ]
}

fn animation_declarations(value: AnimationList) -> Vec<Declaration> {
    let mut names = Vec::new();
    let mut durations = Vec::new();
    let mut delays = Vec::new();
    let mut easings = Vec::new();
    let mut iteration_counts = Vec::new();
    let mut directions = Vec::new();
    let mut fill_modes = Vec::new();
    let mut play_states = Vec::new();

    for item in value.items() {
        names.push(item.name().cloned().unwrap_or(AnimationName::None));
        durations.push(
            item.duration()
                .unwrap_or_else(|| DurationSeconds::new(0.0).unwrap()),
        );
        delays.push(
            item.delay()
                .unwrap_or_else(|| DurationSeconds::new(0.0).unwrap()),
        );
        easings.push(
            item.timing_function()
                .cloned()
                .unwrap_or(EasingFunction::Ease),
        );
        iteration_counts.push(item.iteration_count().unwrap_or_else(|| {
            AnimationIterationCount::Number(AnimationIterationNumber::try_new(1.0).unwrap())
        }));
        directions.push(item.direction().unwrap_or(AnimationDirection::Normal));
        fill_modes.push(item.fill_mode().unwrap_or(AnimationFillMode::None));
        play_states.push(item.play_state().unwrap_or(AnimationPlayState::Running));
    }

    vec![
        Declaration::new(
            Property::AnimationName,
            Value::AnimationNameList(AnimationNameList::try_new(names).unwrap()),
        ),
        Declaration::new(
            Property::AnimationDuration,
            Value::TimeList(TimeList::try_new(durations).unwrap()),
        ),
        Declaration::new(
            Property::AnimationDelay,
            Value::TimeList(TimeList::try_new(delays).unwrap()),
        ),
        Declaration::new(
            Property::AnimationTimingFunction,
            Value::EasingList(EasingList::try_new(easings).unwrap()),
        ),
        Declaration::new(
            Property::AnimationIterationCount,
            Value::AnimationIterationCountList(
                AnimationIterationCountList::try_new(iteration_counts).unwrap(),
            ),
        ),
        Declaration::new(
            Property::AnimationDirection,
            Value::AnimationDirectionList(AnimationDirectionList::try_new(directions).unwrap()),
        ),
        Declaration::new(
            Property::AnimationFillMode,
            Value::AnimationFillModeList(AnimationFillModeList::try_new(fill_modes).unwrap()),
        ),
        Declaration::new(
            Property::AnimationPlayState,
            Value::AnimationPlayStateList(AnimationPlayStateList::try_new(play_states).unwrap()),
        ),
    ]
}

fn flex_declarations(value: Flex) -> Vec<Declaration> {
    let (grow, shrink, basis) = match value {
        Flex::None => (FlexFactor::zero(), FlexFactor::zero(), Length::Auto),
        Flex::Auto => (FlexFactor::one(), FlexFactor::one(), Length::Auto),
        Flex::Components {
            grow,
            shrink,
            basis,
        } => (
            grow,
            shrink.unwrap_or_else(FlexFactor::one),
            basis.unwrap_or(Length::ZERO),
        ),
    };

    vec![
        Declaration::new(Property::FlexGrow, Value::FlexFactor(grow)),
        Declaration::new(Property::FlexShrink, Value::FlexFactor(shrink)),
        Declaration::new(Property::FlexBasis, Value::Length(basis)),
    ]
}

fn font_declarations(font: Font) -> Vec<Declaration> {
    vec![
        Declaration::new(
            Property::FontStyle,
            Value::TextSlant(font.style().unwrap_or_default()),
        ),
        Declaration::new(
            Property::FontVariant,
            Value::FontVariant(font.variant().unwrap_or_default()),
        ),
        Declaration::new(
            Property::FontWeight,
            Value::FontWeight(font.weight().unwrap_or_default()),
        ),
        Declaration::new(
            Property::FontStretch,
            Value::FontStretch(font.stretch().unwrap_or_default()),
        ),
        Declaration::new(Property::FontSize, Value::Length(font.size().clone())),
        Declaration::new(
            Property::LineHeight,
            Value::Length(
                font.line_height()
                    .cloned()
                    .unwrap_or_else(default_line_height),
            ),
        ),
        Declaration::new(
            Property::FontFamily,
            Value::FontFamilyList(font.family().clone()),
        ),
    ]
}

fn text_decoration_declarations(value: TextDecoration) -> Vec<Declaration> {
    vec![
        Declaration::new(
            Property::TextDecorationLine,
            Value::TextDecorationLine(value.line().cloned().unwrap_or_default()),
        ),
        Declaration::new(
            Property::TextDecorationColor,
            Value::StyleColor(
                value
                    .color()
                    .cloned()
                    .unwrap_or_else(StyleColor::current_color),
            ),
        ),
        Declaration::new(
            Property::TextDecorationStyle,
            Value::TextDecorationStyle(value.style().unwrap_or_default()),
        ),
        Declaration::new(
            Property::TextDecorationThickness,
            Value::TextDecorationThickness(value.thickness().cloned().unwrap_or_default()),
        ),
    ]
}

fn list_style_declarations(value: ListStyle) -> Vec<Declaration> {
    vec![
        Declaration::new(
            Property::ListStyleType,
            Value::ListStyleType(value.style_type().cloned().unwrap_or_default()),
        ),
        Declaration::new(
            Property::ListStylePosition,
            Value::ListStylePosition(value.position().unwrap_or_default()),
        ),
        Declaration::new(
            Property::ListStyleImage,
            Value::ListStyleImage(value.image().cloned().unwrap_or_default()),
        ),
    ]
}

fn mask_declarations(layers: MaskLayerList) -> Vec<Declaration> {
    let mut images = Vec::new();
    let mut positions = Vec::new();
    let mut sizes = Vec::new();
    let mut repeats = Vec::new();

    for layer in layers.layers() {
        images.push(layer.image().cloned().unwrap_or(ImageLayer::None));
        positions.push(
            layer
                .position()
                .cloned()
                .unwrap_or_else(default_layer_position),
        );
        sizes.push(layer.size().cloned().unwrap_or_else(default_layer_size));
        repeats.push(layer.repeat().unwrap_or_else(default_layer_repeat));
    }

    vec![
        Declaration::new(
            Property::MaskImage,
            Value::ImageLayerList(ImageLayerList::try_new(images).unwrap()),
        ),
        Declaration::new(
            Property::MaskPosition,
            Value::PositionList(PositionList::try_new(positions).unwrap()),
        ),
        Declaration::new(
            Property::MaskSize,
            Value::BackgroundSizeList(BackgroundSizeList::try_new(sizes).unwrap()),
        ),
        Declaration::new(
            Property::MaskRepeat,
            Value::BackgroundRepeatList(BackgroundRepeatList::try_new(repeats).unwrap()),
        ),
    ]
}

fn default_line_height() -> Length {
    match Property::LineHeight.metadata().default() {
        Value::Length(length) => length.clone(),
        _ => unreachable!("line-height metadata default is a length"),
    }
}

fn default_layer_position() -> Position {
    match Property::MaskPosition.metadata().default() {
        Value::PositionList(positions) => positions.positions()[0].clone(),
        _ => unreachable!("mask-position metadata default is a position list"),
    }
}

fn default_layer_size() -> BackgroundSize {
    match Property::MaskSize.metadata().default() {
        Value::BackgroundSizeList(sizes) => sizes.sizes()[0].clone(),
        _ => unreachable!("mask-size metadata default is a background size list"),
    }
}

fn default_layer_repeat() -> BackgroundRepeat {
    match Property::MaskRepeat.metadata().default() {
        Value::BackgroundRepeatList(repeats) => repeats.repeats()[0],
        _ => unreachable!("mask-repeat metadata default is a background repeat list"),
    }
}

fn grid_placement_end_for_shorthand(start: &GridLine, end: GridLine) -> GridLine {
    match (&start, end) {
        (GridLine::BareIdent(name), GridLine::Auto) => GridLine::BareIdent(name.clone()),
        (_, end) => end,
    }
}

fn grid_area_omitted_line(reference: &GridLine) -> GridLine {
    match reference {
        GridLine::BareIdent(name) => GridLine::BareIdent(name.clone()),
        _ => GridLine::Auto,
    }
}

fn same_value_declarations(properties: Vec<Property>, value: Value) -> Vec<Declaration> {
    properties
        .into_iter()
        .map(|property| Declaration::new(property, value.clone()))
        .collect()
}

fn edge_declarations(
    edges: Edges,
    top: Property,
    right: Property,
    bottom: Property,
    left: Property,
) -> Vec<Declaration> {
    vec![
        Declaration::new(top, Value::Length(edges.top)),
        Declaration::new(right, Value::Length(edges.right)),
        Declaration::new(bottom, Value::Length(edges.bottom)),
        Declaration::new(left, Value::Length(edges.left)),
    ]
}

fn border_declarations(border: Border) -> Vec<Declaration> {
    let width = border.width().cloned().unwrap_or_else(default_border_width);
    let style = border.style().unwrap_or(BorderLineStyle::None);
    let color = border
        .color()
        .cloned()
        .unwrap_or_else(StyleColor::current_color);

    vec![
        Declaration::new(Property::BorderTopWidth, Value::Length(width.clone())),
        Declaration::new(Property::BorderRightWidth, Value::Length(width.clone())),
        Declaration::new(Property::BorderBottomWidth, Value::Length(width.clone())),
        Declaration::new(Property::BorderLeftWidth, Value::Length(width)),
        Declaration::new(Property::BorderTopStyle, Value::BorderLineStyle(style)),
        Declaration::new(Property::BorderRightStyle, Value::BorderLineStyle(style)),
        Declaration::new(Property::BorderBottomStyle, Value::BorderLineStyle(style)),
        Declaration::new(Property::BorderLeftStyle, Value::BorderLineStyle(style)),
        Declaration::new(Property::BorderTopColor, Value::StyleColor(color.clone())),
        Declaration::new(Property::BorderRightColor, Value::StyleColor(color.clone())),
        Declaration::new(
            Property::BorderBottomColor,
            Value::StyleColor(color.clone()),
        ),
        Declaration::new(Property::BorderLeftColor, Value::StyleColor(color)),
    ]
}

fn border_side_declarations(side: BorderSide, border: Border) -> Vec<Declaration> {
    let width = border.width().cloned().unwrap_or_else(default_border_width);
    let style = border.style().unwrap_or(BorderLineStyle::None);
    let color = border
        .color()
        .cloned()
        .unwrap_or_else(StyleColor::current_color);
    let (width_property, style_property, color_property) = match side {
        BorderSide::Top => (
            Property::BorderTopWidth,
            Property::BorderTopStyle,
            Property::BorderTopColor,
        ),
        BorderSide::Right => (
            Property::BorderRightWidth,
            Property::BorderRightStyle,
            Property::BorderRightColor,
        ),
        BorderSide::Bottom => (
            Property::BorderBottomWidth,
            Property::BorderBottomStyle,
            Property::BorderBottomColor,
        ),
        BorderSide::Left => (
            Property::BorderLeftWidth,
            Property::BorderLeftStyle,
            Property::BorderLeftColor,
        ),
    };

    vec![
        Declaration::new(width_property, Value::Length(width)),
        Declaration::new(style_property, Value::BorderLineStyle(style)),
        Declaration::new(color_property, Value::StyleColor(color)),
    ]
}

fn outline_declarations(outline: Outline) -> Vec<Declaration> {
    let width = outline.width().cloned().unwrap_or(OutlineWidth::Medium);
    let style = outline
        .style()
        .unwrap_or(OutlineStyle::Border(BorderLineStyle::None));
    let color = outline
        .color()
        .cloned()
        .unwrap_or_else(StyleColor::current_color);

    vec![
        Declaration::new(Property::OutlineWidth, Value::OutlineWidth(width)),
        Declaration::new(Property::OutlineStyle, Value::OutlineStyle(style)),
        Declaration::new(Property::OutlineColor, Value::StyleColor(color)),
    ]
}

fn default_border_width() -> Length {
    Length::Px(3.0)
}

fn edge_values(
    declarations: &Declarations,
    top: Property,
    right: Property,
    bottom: Property,
    left: Property,
) -> Option<Edges> {
    Some(Edges::new(
        declaration_length(declarations, top)?,
        declaration_length(declarations, right)?,
        declaration_length(declarations, bottom)?,
        declaration_length(declarations, left)?,
    ))
}

fn declaration_length(declarations: &Declarations, property: Property) -> Option<Length> {
    match declarations.get(property) {
        Some(Value::Length(length)) => Some(length.clone()),
        _ => None,
    }
}

pub(crate) fn hash_value(value: &Value, state: &mut DefaultHasher) {
    match value {
        Value::Keyword(value) => {
            0u8.hash(state);
            value.hash(state);
        }
        Value::Display(value) => {
            20u8.hash(state);
            value.hash(state);
        }
        Value::BoxSizing(value) => {
            26u8.hash(state);
            value.hash(state);
        }
        Value::Position(value) => {
            27u8.hash(state);
            value.hash(state);
        }
        Value::ZIndex(value) => {
            41u8.hash(state);
            match value {
                ZIndex::Auto => {
                    0u8.hash(state);
                }
                ZIndex::Integer(value) => {
                    1u8.hash(state);
                    value.hash(state);
                }
            }
        }
        Value::ScrollbarWidth(value) => {
            45u8.hash(state);
            value.hash(state);
        }
        Value::ContentVisibility(value) => {
            46u8.hash(state);
            value.hash(state);
        }
        Value::Content(value) => {
            93u8.hash(state);
            hash_content(value, state);
        }
        Value::ListStyleType(value) => {
            94u8.hash(state);
            hash_list_style_type(value, state);
        }
        Value::ListStylePosition(value) => {
            95u8.hash(state);
            value.hash(state);
        }
        Value::ListStyleImage(value) => {
            96u8.hash(state);
            hash_list_style_image(value, state);
        }
        Value::ListStyle(value) => {
            97u8.hash(state);
            hash_list_style(value, state);
        }
        Value::CounterChanges(value) => {
            98u8.hash(state);
            hash_counter_changes(value, state);
        }
        Value::TransitionPropertyList(value) => {
            99u8.hash(state);
            hash_transition_property_list(value, state);
        }
        Value::TimeList(value) => {
            100u8.hash(state);
            hash_time_list(value, state);
        }
        Value::EasingList(value) => {
            101u8.hash(state);
            hash_easing_list(value, state);
        }
        Value::TransitionList(value) => {
            102u8.hash(state);
            hash_transition_list(value, state);
        }
        Value::AnimationIterationCountList(value) => {
            103u8.hash(state);
            hash_animation_iteration_count_list(value, state);
        }
        Value::AnimationDirectionList(value) => {
            104u8.hash(state);
            value.hash(state);
        }
        Value::AnimationFillModeList(value) => {
            105u8.hash(state);
            value.hash(state);
        }
        Value::AnimationPlayStateList(value) => {
            106u8.hash(state);
            value.hash(state);
        }
        Value::AnimationList(value) => {
            107u8.hash(state);
            hash_animation_list(value, state);
        }
        Value::Order(value) => {
            42u8.hash(state);
            value.get().hash(state);
        }
        Value::FlexFactor(value) => {
            43u8.hash(state);
            value.get().to_bits().hash(state);
        }
        Value::Flex(value) => {
            47u8.hash(state);
            hash_flex(value, state);
        }
        Value::AspectRatio(value) => {
            44u8.hash(state);
            match value.as_ratio() {
                Some(ratio) => {
                    1u8.hash(state);
                    ratio.to_bits().hash(state);
                }
                None => {
                    0u8.hash(state);
                }
            }
        }
        Value::Direction(value) => {
            28u8.hash(state);
            value.hash(state);
        }
        Value::Overflow(value) => {
            29u8.hash(state);
            value.hash(state);
        }
        Value::OverflowAxes(value) => {
            38u8.hash(state);
            value.hash(state);
        }
        Value::Float(value) => {
            30u8.hash(state);
            value.hash(state);
        }
        Value::Clear(value) => {
            31u8.hash(state);
            value.hash(state);
        }
        Value::TextAlign(value) => {
            32u8.hash(state);
            value.hash(state);
        }
        Value::TextAlignLast(value) => {
            56u8.hash(state);
            value.hash(state);
        }
        Value::TextIndent(value) => {
            57u8.hash(state);
            hash_length(value.length(), state);
            value.hanging().hash(state);
            value.each_line().hash(state);
        }
        Value::VerticalAlign(value) => {
            58u8.hash(state);
            hash_vertical_align(value, state);
        }
        Value::LetterSpacing(value) => {
            59u8.hash(state);
            hash_letter_spacing(value, state);
        }
        Value::TextTransform(value) => {
            60u8.hash(state);
            value.hash(state);
        }
        Value::TextWrap(value) => {
            61u8.hash(state);
            value.hash(state);
        }
        Value::WhiteSpace(value) => {
            62u8.hash(state);
            value.hash(state);
        }
        Value::WordBreak(value) => {
            63u8.hash(state);
            value.hash(state);
        }
        Value::OverflowWrap(value) => {
            64u8.hash(state);
            value.hash(state);
        }
        Value::TextOverflow(value) => {
            65u8.hash(state);
            value.hash(state);
        }
        Value::TextDecoration(value) => {
            66u8.hash(state);
            hash_text_decoration(value, state);
        }
        Value::TextDecorationLine(value) => {
            67u8.hash(state);
            value.hash(state);
        }
        Value::TextDecorationStyle(value) => {
            68u8.hash(state);
            value.hash(state);
        }
        Value::TextDecorationThickness(value) => {
            69u8.hash(state);
            hash_text_decoration_thickness(value, state);
        }
        Value::StyleColor(value) => {
            70u8.hash(state);
            hash_style_color(value, state);
        }
        Value::Border(value) => {
            71u8.hash(state);
            hash_border(value, state);
        }
        Value::BorderStyles(value) => {
            72u8.hash(state);
            value.hash(state);
        }
        Value::BorderLineStyle(value) => {
            73u8.hash(state);
            value.hash(state);
        }
        Value::CornerRadius(value) => {
            74u8.hash(state);
            hash_corner_radius(value, state);
        }
        Value::BorderRadii(value) => {
            75u8.hash(state);
            hash_border_radii(value, state);
        }
        Value::Outline(value) => {
            76u8.hash(state);
            hash_outline(value, state);
        }
        Value::OutlineStyle(value) => {
            77u8.hash(state);
            value.hash(state);
        }
        Value::OutlineWidth(value) => {
            78u8.hash(state);
            hash_outline_width(value, state);
        }
        Value::ImageLayerList(value) => {
            79u8.hash(state);
            hash_image_layer_list(value, state);
        }
        Value::PositionList(value) => {
            80u8.hash(state);
            hash_position_list(value, state);
        }
        Value::BackgroundSizeList(value) => {
            81u8.hash(state);
            hash_background_size_list(value, state);
        }
        Value::BackgroundRepeatList(value) => {
            82u8.hash(state);
            value.hash(state);
        }
        Value::BackgroundBox(value) => {
            83u8.hash(state);
            value.hash(state);
        }
        Value::BackgroundAttachmentList(value) => {
            84u8.hash(state);
            value.hash(state);
        }
        Value::MaskLayerList(value) => {
            85u8.hash(state);
            hash_mask_layer_list(value, state);
        }
        Value::BoxDecorationBreak(value) => {
            86u8.hash(state);
            value.hash(state);
        }
        Value::Filter(value) => {
            87u8.hash(state);
            hash_filter(value, state);
        }
        Value::ClipPath(value) => {
            88u8.hash(state);
            hash_clip_path(value, state);
        }
        Value::WritingMode(value) => {
            33u8.hash(state);
            value.hash(state);
        }
        Value::FlexDirection(value) => {
            34u8.hash(state);
            value.hash(state);
        }
        Value::FlexWrap(value) => {
            35u8.hash(state);
            value.hash(state);
        }
        Value::AlignItems(value) => {
            36u8.hash(state);
            value.hash(state);
        }
        Value::AlignContent(value) => {
            37u8.hash(state);
            value.hash(state);
        }
        Value::PlaceContentAlignment(value) => {
            48u8.hash(state);
            value.hash(state);
        }
        Value::PlaceItemsAlignment(value) => {
            49u8.hash(state);
            value.hash(state);
        }
        Value::Number(value) => {
            1u8.hash(state);
            hash_f32(*value, state);
        }
        Value::Length(value) => {
            2u8.hash(state);
            hash_length(value, state);
        }
        Value::Size(value) => {
            3u8.hash(state);
            hash_length(&value.width, state);
            hash_length(&value.height, state);
        }
        Value::Edges(value) => {
            4u8.hash(state);
            hash_length(&value.top, state);
            hash_length(&value.right, state);
            hash_length(&value.bottom, state);
            hash_length(&value.left, state);
        }
        Value::GridTrackList(value) => {
            16u8.hash(state);
            hash_grid_track_list(value, state);
        }
        Value::GridTemplateAreas(value) => {
            21u8.hash(state);
            hash_grid_template_areas(value, state);
        }
        Value::GridTemplate(value) => {
            23u8.hash(state);
            hash_grid_template(value, state);
        }
        Value::GridDefinition(value) => {
            24u8.hash(state);
            hash_grid_definition(value, state);
        }
        Value::GridLine(value) => {
            22u8.hash(state);
            hash_grid_line(value, state);
        }
        Value::GridPlacement(value) => {
            17u8.hash(state);
            hash_grid_placement(value, state);
        }
        Value::GridAreaPlacement(value) => {
            25u8.hash(state);
            hash_grid_area_placement(value, state);
        }
        Value::GridAutoFlow(value) => {
            18u8.hash(state);
            value.hash(state);
        }
        Value::GridFlowTolerance(value) => {
            39u8.hash(state);
            hash_grid_flow_tolerance(value, state);
        }
        Value::Color(value) => {
            5u8.hash(state);
            hash_color(*value, state);
        }
        Value::Corners(value) => {
            6u8.hash(state);
            hash_length(&value.top_left, state);
            hash_length(&value.top_right, state);
            hash_length(&value.bottom_right, state);
            hash_length(&value.bottom_left, state);
        }
        Value::FontFamilyList(value) => {
            7u8.hash(state);
            value.hash(state);
        }
        Value::FontWeight(value) => {
            50u8.hash(state);
            value.hash(state);
        }
        Value::TextSlant(value) => {
            51u8.hash(state);
            hash_slant(*value, state);
        }
        Value::FontStretch(value) => {
            52u8.hash(state);
            value.hash(state);
        }
        Value::FontVariant(value) => {
            53u8.hash(state);
            value.hash(state);
        }
        Value::FontFeatureSettings(value) => {
            54u8.hash(state);
            value.hash(state);
        }
        Value::Font(value) => {
            55u8.hash(state);
            hash_font(value, state);
        }
        Value::AnimationNameList(value) => {
            40u8.hash(state);
            hash_animation_name_list(value, state);
        }
        Value::ShadowList(value) => {
            9u8.hash(state);
            value.len().hash(state);
            for shadow in value {
                hash_length(&shadow.x, state);
                hash_length(&shadow.y, state);
                hash_length(&shadow.blur, state);
                hash_length(&shadow.spread, state);
                hash_color(shadow.color, state);
                shadow.inset.hash(state);
            }
        }
        Value::Stroke(value) => {
            10u8.hash(state);
            hash_length(&value.width, state);
            hash_color(value.color, state);
            value.style.hash(state);
            value.sides.top.hash(state);
            value.sides.right.hash(state);
            value.sides.bottom.hash(state);
            value.sides.left.hash(state);
            if let Some(dash) = value.dash {
                true.hash(state);
                hash_f32(dash.density, state);
                hash_f32(dash.phase, state);
                dash.rounded.hash(state);
                dash.circular.hash(state);
            } else {
                false.hash(state);
            }
            value.align.hash(state);
        }
        Value::Text(value) => {
            11u8.hash(state);
            value.font_family.hash(state);
            hash_length(&value.font_size, state);
            value.font_weight.hash(state);
            hash_slant(value.font_style, state);
            hash_length(&value.line_height, state);
            hash_color(value.color, state);
            value.alignment.hash(state);
            value.wrap.hash(state);
            value.white_space.hash(state);
            value.word_break.hash(state);
            value.overflow_wrap.hash(state);
            hash_decoration(value.underline, state);
            hash_decoration(value.strikethrough, state);
            hash_color(value.selection_color, state);
        }
        Value::Transform(value) => {
            12u8.hash(state);
            value.operations.len().hash(state);
            for operation in &value.operations {
                hash_transform_op(operation, state);
            }
        }
        Value::Translate(value) => {
            89u8.hash(state);
            hash_translate(value, state);
        }
        Value::Rotate(value) => {
            90u8.hash(state);
            hash_rotate(value, state);
        }
        Value::Scale(value) => {
            91u8.hash(state);
            hash_scale(value, state);
        }
        Value::Cursor(value) => {
            13u8.hash(state);
            value.hash(state);
        }
        Value::PointerEvents(value) => {
            14u8.hash(state);
            value.hash(state);
        }
        Value::UserSelect(value) => {
            92u8.hash(state);
            value.hash(state);
        }
        Value::Visibility(value) => {
            15u8.hash(state);
            value.hash(state);
        }
    }
}

fn hash_transition_property_list(value: &TransitionPropertyList, state: &mut DefaultHasher) {
    value.values().len().hash(state);
    for target in value.values() {
        hash_transition_property_target(target, state);
    }
}

fn hash_transition_property_target(value: &TransitionPropertyTarget, state: &mut DefaultHasher) {
    match value {
        TransitionPropertyTarget::All => {
            0u8.hash(state);
        }
        TransitionPropertyTarget::None => {
            1u8.hash(state);
        }
        TransitionPropertyTarget::Property(property) => {
            2u8.hash(state);
            property.hash(state);
        }
        TransitionPropertyTarget::Custom(name) => {
            3u8.hash(state);
            name.as_str().hash(state);
        }
    }
}

fn hash_time_list(value: &TimeList, state: &mut DefaultHasher) {
    value.seconds().len().hash(state);
    for seconds in value.seconds() {
        hash_f32(seconds.get(), state);
    }
}

fn hash_easing_list(value: &EasingList, state: &mut DefaultHasher) {
    value.values().len().hash(state);
    for easing in value.values() {
        hash_easing_function(easing, state);
    }
}

fn hash_easing_function(value: &EasingFunction, state: &mut DefaultHasher) {
    match value {
        EasingFunction::Ease => 0u8.hash(state),
        EasingFunction::Linear => 1u8.hash(state),
        EasingFunction::EaseIn => 2u8.hash(state),
        EasingFunction::EaseOut => 3u8.hash(state),
        EasingFunction::EaseInOut => 4u8.hash(state),
        EasingFunction::StepStart => 5u8.hash(state),
        EasingFunction::StepEnd => 6u8.hash(state),
        EasingFunction::CubicBezier(arguments) => {
            7u8.hash(state);
            arguments.as_css().hash(state);
        }
        EasingFunction::Steps(arguments) => {
            8u8.hash(state);
            arguments.as_css().hash(state);
        }
    }
}

fn hash_transition_list(value: &TransitionList, state: &mut DefaultHasher) {
    value.items().len().hash(state);
    for item in value.items() {
        hash_transition_item(item, state);
    }
}

fn hash_transition_item(value: &TransitionItem, state: &mut DefaultHasher) {
    match value.property() {
        Some(property) => {
            true.hash(state);
            hash_transition_property_target(property, state);
        }
        None => false.hash(state),
    }
    match value.duration() {
        Some(duration) => {
            true.hash(state);
            hash_f32(duration.get(), state);
        }
        None => false.hash(state),
    }
    match value.delay() {
        Some(delay) => {
            true.hash(state);
            hash_f32(delay.get(), state);
        }
        None => false.hash(state),
    }
    match value.timing_function() {
        Some(easing) => {
            true.hash(state);
            hash_easing_function(easing, state);
        }
        None => false.hash(state),
    }
}

fn hash_animation_name_list(value: &AnimationNameList, state: &mut DefaultHasher) {
    value.names().len().hash(state);
    for name in value.names() {
        hash_animation_name(name, state);
    }
}

fn hash_animation_name(value: &AnimationName, state: &mut DefaultHasher) {
    match value {
        AnimationName::None => {
            0u8.hash(state);
        }
        AnimationName::Keyframes(name) => {
            1u8.hash(state);
            hash_keyframes_name(name, state);
        }
    }
}

fn hash_keyframes_name(value: &KeyframesName, state: &mut DefaultHasher) {
    match value {
        KeyframesName::Ident(ident) => {
            0u8.hash(state);
            ident.as_str().hash(state);
        }
        KeyframesName::String(string) => {
            1u8.hash(state);
            string.as_str().hash(state);
        }
    }
}

fn hash_animation_iteration_count_list(
    value: &AnimationIterationCountList,
    state: &mut DefaultHasher,
) {
    value.values().len().hash(state);
    for count in value.values() {
        hash_animation_iteration_count(*count, state);
    }
}

fn hash_animation_iteration_count(value: AnimationIterationCount, state: &mut DefaultHasher) {
    match value {
        AnimationIterationCount::Infinite => {
            0u8.hash(state);
        }
        AnimationIterationCount::Number(number) => {
            1u8.hash(state);
            hash_f32(number.get(), state);
        }
    }
}

fn hash_animation_list(value: &AnimationList, state: &mut DefaultHasher) {
    value.items().len().hash(state);
    for item in value.items() {
        hash_animation_item(item, state);
    }
}

fn hash_animation_item(value: &AnimationItem, state: &mut DefaultHasher) {
    match value.name() {
        Some(name) => {
            true.hash(state);
            hash_animation_name(name, state);
        }
        None => false.hash(state),
    }
    match value.duration() {
        Some(duration) => {
            true.hash(state);
            hash_f32(duration.get(), state);
        }
        None => false.hash(state),
    }
    match value.delay() {
        Some(delay) => {
            true.hash(state);
            hash_f32(delay.get(), state);
        }
        None => false.hash(state),
    }
    match value.timing_function() {
        Some(easing) => {
            true.hash(state);
            hash_easing_function(easing, state);
        }
        None => false.hash(state),
    }
    match value.iteration_count() {
        Some(count) => {
            true.hash(state);
            hash_animation_iteration_count(count, state);
        }
        None => false.hash(state),
    }
    match value.direction() {
        Some(direction) => {
            true.hash(state);
            hash_animation_direction(direction, state);
        }
        None => false.hash(state),
    }
    match value.fill_mode() {
        Some(fill_mode) => {
            true.hash(state);
            hash_animation_fill_mode(fill_mode, state);
        }
        None => false.hash(state),
    }
    match value.play_state() {
        Some(play_state) => {
            true.hash(state);
            hash_animation_play_state(play_state, state);
        }
        None => false.hash(state),
    }
}

fn hash_animation_direction(value: AnimationDirection, state: &mut DefaultHasher) {
    match value {
        AnimationDirection::Normal => 0u8.hash(state),
        AnimationDirection::Reverse => 1u8.hash(state),
        AnimationDirection::Alternate => 2u8.hash(state),
        AnimationDirection::AlternateReverse => 3u8.hash(state),
    }
}

fn hash_animation_fill_mode(value: AnimationFillMode, state: &mut DefaultHasher) {
    match value {
        AnimationFillMode::None => 0u8.hash(state),
        AnimationFillMode::Forwards => 1u8.hash(state),
        AnimationFillMode::Backwards => 2u8.hash(state),
        AnimationFillMode::Both => 3u8.hash(state),
    }
}

fn hash_animation_play_state(value: AnimationPlayState, state: &mut DefaultHasher) {
    match value {
        AnimationPlayState::Running => 0u8.hash(state),
        AnimationPlayState::Paused => 1u8.hash(state),
    }
}

fn hash_content(value: &Content, state: &mut DefaultHasher) {
    match value {
        Content::Normal => 0u8.hash(state),
        Content::None => 1u8.hash(state),
        Content::Items(items) => {
            2u8.hash(state);
            items.items().len().hash(state);
            for item in items.items() {
                hash_content_item(item, state);
            }
        }
    }
}

fn hash_content_item(value: &ContentItem, state: &mut DefaultHasher) {
    match value {
        ContentItem::String(value) => {
            0u8.hash(state);
            value.as_str().hash(state);
        }
        ContentItem::Url(value) => {
            1u8.hash(state);
            value.hash(state);
        }
        ContentItem::Counter(value) => {
            2u8.hash(state);
            hash_counter_function(value, state);
        }
        ContentItem::Counters(value) => {
            3u8.hash(state);
            hash_counters_function(value, state);
        }
        ContentItem::Attr(value) => {
            4u8.hash(state);
            value.as_str().hash(state);
        }
        ContentItem::OpenQuote => 5u8.hash(state),
        ContentItem::CloseQuote => 6u8.hash(state),
        ContentItem::NoOpenQuote => 7u8.hash(state),
        ContentItem::NoCloseQuote => 8u8.hash(state),
    }
}

fn hash_counter_function(value: &CounterFunction, state: &mut DefaultHasher) {
    value.name().as_str().hash(state);
    if let Some(style) = value.style() {
        true.hash(state);
        hash_counter_style(style, state);
    } else {
        false.hash(state);
    }
}

fn hash_counters_function(value: &CountersFunction, state: &mut DefaultHasher) {
    value.name().as_str().hash(state);
    value.separator().as_str().hash(state);
    if let Some(style) = value.style() {
        true.hash(state);
        hash_counter_style(style, state);
    } else {
        false.hash(state);
    }
}

fn hash_counter_style(value: &CounterStyle, state: &mut DefaultHasher) {
    match value {
        CounterStyle::BuiltIn(value) => {
            0u8.hash(state);
            hash_builtin_counter_style(*value, state);
        }
        CounterStyle::Named(value) => {
            1u8.hash(state);
            value.as_str().hash(state);
        }
    }
}

fn hash_builtin_counter_style(value: super::BuiltInCounterStyle, state: &mut DefaultHasher) {
    match value {
        super::BuiltInCounterStyle::Disc => 0u8.hash(state),
        super::BuiltInCounterStyle::Circle => 1u8.hash(state),
        super::BuiltInCounterStyle::Square => 2u8.hash(state),
        super::BuiltInCounterStyle::Decimal => 3u8.hash(state),
        super::BuiltInCounterStyle::DecimalLeadingZero => 4u8.hash(state),
        super::BuiltInCounterStyle::LowerAlpha => 5u8.hash(state),
        super::BuiltInCounterStyle::UpperAlpha => 6u8.hash(state),
        super::BuiltInCounterStyle::LowerLatin => 7u8.hash(state),
        super::BuiltInCounterStyle::UpperLatin => 8u8.hash(state),
        super::BuiltInCounterStyle::LowerRoman => 9u8.hash(state),
        super::BuiltInCounterStyle::UpperRoman => 10u8.hash(state),
    }
}

fn hash_list_style_type(value: &ListStyleType, state: &mut DefaultHasher) {
    match value {
        ListStyleType::None => 0u8.hash(state),
        ListStyleType::CounterStyle(value) => {
            1u8.hash(state);
            hash_counter_style(value, state);
        }
        ListStyleType::String(value) => {
            2u8.hash(state);
            value.as_str().hash(state);
        }
    }
}

fn hash_list_style_image(value: &ListStyleImage, state: &mut DefaultHasher) {
    match value {
        ListStyleImage::None => 0u8.hash(state),
        ListStyleImage::Url(value) => {
            1u8.hash(state);
            value.hash(state);
        }
    }
}

fn hash_list_style(value: &ListStyle, state: &mut DefaultHasher) {
    if let Some(style_type) = value.style_type() {
        true.hash(state);
        hash_list_style_type(style_type, state);
    } else {
        false.hash(state);
    }
    value.position().hash(state);
    if let Some(image) = value.image() {
        true.hash(state);
        hash_list_style_image(image, state);
    } else {
        false.hash(state);
    }
}

fn hash_counter_changes(value: &CounterChanges, state: &mut DefaultHasher) {
    match value {
        CounterChanges::None => 0u8.hash(state),
        CounterChanges::Changes(changes) => {
            1u8.hash(state);
            changes.changes().len().hash(state);
            for change in changes.changes() {
                change.name().as_str().hash(state);
                change.value().hash(state);
            }
        }
    }
}

fn hash_filter(value: &Filter, state: &mut DefaultHasher) {
    match value {
        Filter::None => 0u8.hash(state),
        Filter::Functions(functions) => {
            1u8.hash(state);
            functions.functions().len().hash(state);
            for function in functions.functions() {
                hash_filter_function(function, state);
            }
        }
    }
}

fn hash_filter_function(value: &FilterFunction, state: &mut DefaultHasher) {
    match value {
        FilterFunction::Blur(value) => {
            0u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Brightness(value) => {
            1u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Contrast(value) => {
            2u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::DropShadow(value) => {
            3u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Grayscale(value) => {
            4u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::HueRotate(value) => {
            5u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Invert(value) => {
            6u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Opacity(value) => {
            7u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Saturate(value) => {
            8u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Sepia(value) => {
            9u8.hash(state);
            value.as_str().hash(state);
        }
        FilterFunction::Url(value) => {
            10u8.hash(state);
            value.as_str().hash(state);
        }
    }
}

fn hash_clip_path(value: &ClipPath, state: &mut DefaultHasher) {
    match value {
        ClipPath::None => 0u8.hash(state),
        ClipPath::Url(value) => {
            1u8.hash(state);
            value.as_str().hash(state);
        }
        ClipPath::BasicShape(value) => {
            2u8.hash(state);
            hash_basic_shape(value, state);
        }
    }
}

fn hash_basic_shape(value: &BasicShape, state: &mut DefaultHasher) {
    match value {
        BasicShape::Inset(value) => {
            0u8.hash(state);
            value.as_str().hash(state);
        }
        BasicShape::Circle(value) => {
            1u8.hash(state);
            value.as_str().hash(state);
        }
        BasicShape::Ellipse(value) => {
            2u8.hash(state);
            value.as_str().hash(state);
        }
        BasicShape::Polygon(value) => {
            3u8.hash(state);
            value.as_str().hash(state);
        }
    }
}

fn hash_translate(value: &Translate, state: &mut DefaultHasher) {
    match value {
        Translate::None => 0u8.hash(state),
        Translate::Values(values) => {
            1u8.hash(state);
            values.values().len().hash(state);
            for value in values.values() {
                hash_length(value, state);
            }
        }
    }
}

fn hash_rotate(value: &Rotate, state: &mut DefaultHasher) {
    match value {
        Rotate::None => 0u8.hash(state),
        Rotate::Value(value) => {
            1u8.hash(state);
            value.as_str().hash(state);
        }
    }
}

fn hash_scale(value: &Scale, state: &mut DefaultHasher) {
    match value {
        Scale::None => 0u8.hash(state),
        Scale::Values(values) => {
            1u8.hash(state);
            values.values().len().hash(state);
            for value in values.values() {
                hash_f32(*value, state);
            }
        }
    }
}

fn hash_flex(value: &Flex, state: &mut DefaultHasher) {
    match value {
        Flex::None => 0u8.hash(state),
        Flex::Auto => 1u8.hash(state),
        Flex::Components {
            grow,
            shrink,
            basis,
        } => {
            2u8.hash(state);
            grow.get().to_bits().hash(state);
            shrink.map(FlexFactor::get).map(f32::to_bits).hash(state);
            if let Some(basis) = basis {
                true.hash(state);
                hash_length(basis, state);
            } else {
                false.hash(state);
            }
        }
    }
}

fn hash_vertical_align(value: &VerticalAlign, state: &mut DefaultHasher) {
    match value {
        VerticalAlign::Baseline => 0u8.hash(state),
        VerticalAlign::Sub => 1u8.hash(state),
        VerticalAlign::Super => 2u8.hash(state),
        VerticalAlign::TextTop => 3u8.hash(state),
        VerticalAlign::TextBottom => 4u8.hash(state),
        VerticalAlign::Middle => 5u8.hash(state),
        VerticalAlign::Top => 6u8.hash(state),
        VerticalAlign::Bottom => 7u8.hash(state),
        VerticalAlign::Length(length) => {
            8u8.hash(state);
            hash_length(length.length(), state);
        }
    }
}

fn hash_letter_spacing(value: &LetterSpacing, state: &mut DefaultHasher) {
    match value {
        LetterSpacing::Normal => 0u8.hash(state),
        LetterSpacing::Length(length) => {
            1u8.hash(state);
            hash_length(length.length(), state);
        }
    }
}

fn hash_text_decoration(value: &TextDecoration, state: &mut DefaultHasher) {
    if let Some(line) = value.line() {
        true.hash(state);
        line.hash(state);
    } else {
        false.hash(state);
    }
    if let Some(color) = value.color() {
        true.hash(state);
        hash_style_color(color, state);
    } else {
        false.hash(state);
    }
    value.style().hash(state);
    if let Some(thickness) = value.thickness() {
        true.hash(state);
        hash_text_decoration_thickness(thickness, state);
    } else {
        false.hash(state);
    }
}

fn hash_text_decoration_thickness(value: &TextDecorationThickness, state: &mut DefaultHasher) {
    match value {
        TextDecorationThickness::Auto => 0u8.hash(state),
        TextDecorationThickness::FromFont => 1u8.hash(state),
        TextDecorationThickness::Length(length) => {
            2u8.hash(state);
            hash_length(length.length(), state);
        }
    }
}

fn hash_border(value: &Border, state: &mut DefaultHasher) {
    if let Some(width) = value.width() {
        true.hash(state);
        hash_length(width, state);
    } else {
        false.hash(state);
    }
    value.style().hash(state);
    if let Some(color) = value.color() {
        true.hash(state);
        hash_style_color(color, state);
    } else {
        false.hash(state);
    }
}

fn hash_corner_radius(value: &CornerRadius, state: &mut DefaultHasher) {
    hash_length(value.horizontal(), state);
    hash_length(value.vertical(), state);
}

fn hash_border_radii(value: &BorderRadii, state: &mut DefaultHasher) {
    hash_corner_radius(value.top_left(), state);
    hash_corner_radius(value.top_right(), state);
    hash_corner_radius(value.bottom_right(), state);
    hash_corner_radius(value.bottom_left(), state);
}

fn hash_outline(value: &Outline, state: &mut DefaultHasher) {
    if let Some(width) = value.width() {
        true.hash(state);
        hash_outline_width(width, state);
    } else {
        false.hash(state);
    }
    value.style().hash(state);
    if let Some(color) = value.color() {
        true.hash(state);
        hash_style_color(color, state);
    } else {
        false.hash(state);
    }
}

fn hash_outline_width(value: &OutlineWidth, state: &mut DefaultHasher) {
    match value {
        OutlineWidth::Thin => 0u8.hash(state),
        OutlineWidth::Medium => 1u8.hash(state),
        OutlineWidth::Thick => 2u8.hash(state),
        OutlineWidth::Length(length) => {
            3u8.hash(state);
            hash_length(length.length(), state);
        }
    }
}

fn hash_image_layer_list(value: &ImageLayerList, state: &mut DefaultHasher) {
    value.layers().len().hash(state);
    for layer in value.layers() {
        hash_image_layer(layer, state);
    }
}

fn hash_image_layer(value: &ImageLayer, state: &mut DefaultHasher) {
    match value {
        ImageLayer::None => 0u8.hash(state),
        ImageLayer::Url(url) => {
            1u8.hash(state);
            url.hash(state);
        }
    }
}

fn hash_position_list(value: &PositionList, state: &mut DefaultHasher) {
    value.positions().len().hash(state);
    for position in value.positions() {
        hash_layer_position(position, state);
    }
}

fn hash_layer_position(value: &Position, state: &mut DefaultHasher) {
    value.components().len().hash(state);
    for component in value.components() {
        match component {
            PositionComponent::Horizontal(keyword) => {
                0u8.hash(state);
                keyword.hash(state);
            }
            PositionComponent::Vertical(keyword) => {
                1u8.hash(state);
                keyword.hash(state);
            }
            PositionComponent::Length(length) => {
                2u8.hash(state);
                hash_length(length, state);
            }
        }
    }
}

fn hash_background_size_list(value: &BackgroundSizeList, state: &mut DefaultHasher) {
    value.sizes().len().hash(state);
    for size in value.sizes() {
        hash_background_size(size, state);
    }
}

fn hash_background_size(value: &BackgroundSize, state: &mut DefaultHasher) {
    match value {
        BackgroundSize::Cover => 0u8.hash(state),
        BackgroundSize::Contain => 1u8.hash(state),
        BackgroundSize::Explicit { width, height } => {
            2u8.hash(state);
            hash_background_size_component(width, state);
            if let Some(height) = height {
                true.hash(state);
                hash_background_size_component(height, state);
            } else {
                false.hash(state);
            }
        }
    }
}

fn hash_background_size_component(value: &BackgroundSizeComponent, state: &mut DefaultHasher) {
    match value {
        BackgroundSizeComponent::Auto => 0u8.hash(state),
        BackgroundSizeComponent::Length(length) => {
            1u8.hash(state);
            hash_length(length, state);
        }
    }
}

fn hash_mask_layer_list(value: &MaskLayerList, state: &mut DefaultHasher) {
    value.layers().len().hash(state);
    for layer in value.layers() {
        if let Some(image) = layer.image() {
            true.hash(state);
            hash_image_layer(image, state);
        } else {
            false.hash(state);
        }
        if let Some(position) = layer.position() {
            true.hash(state);
            hash_layer_position(position, state);
        } else {
            false.hash(state);
        }
        if let Some(size) = layer.size() {
            true.hash(state);
            hash_background_size(size, state);
        } else {
            false.hash(state);
        }
        layer.repeat().hash(state);
    }
}

fn hash_font(value: &Font, state: &mut DefaultHasher) {
    if let Some(style) = value.style() {
        true.hash(state);
        hash_slant(style, state);
    } else {
        false.hash(state);
    }
    value.variant().hash(state);
    value.weight().hash(state);
    value.stretch().hash(state);
    hash_length(value.size(), state);
    if let Some(line_height) = value.line_height() {
        true.hash(state);
        hash_length(line_height, state);
    } else {
        false.hash(state);
    }
    value.family().hash(state);
}

fn hash_grid_track_list(value: &GridTrackList, state: &mut DefaultHasher) {
    value.components.len().hash(state);
    for component in &value.components {
        hash_grid_track_component(component, state);
    }
}

fn hash_grid_template_areas(value: &GridTemplateAreas, state: &mut DefaultHasher) {
    value.rows.len().hash(state);
    for row in &value.rows {
        row.cells.len().hash(state);
        for cell in &row.cells {
            cell.hash(state);
        }
    }
}

fn hash_grid_template(value: &GridTemplate, state: &mut DefaultHasher) {
    hash_grid_track_list(&value.rows, state);
    hash_grid_track_list(&value.columns, state);
    hash_grid_template_areas(&value.areas, state);
}

fn hash_grid_definition(value: &GridDefinition, state: &mut DefaultHasher) {
    hash_grid_template(&value.template, state);
    hash_grid_track_list(&value.auto_rows, state);
    hash_grid_track_list(&value.auto_columns, state);
    value.auto_flow.hash(state);
}

fn hash_grid_track_component(component: &GridTrackComponent, state: &mut DefaultHasher) {
    match component {
        GridTrackComponent::Track(track) => {
            0u8.hash(state);
            hash_track_sizing(track, state);
        }
        GridTrackComponent::Repeat(repeat) => {
            1u8.hash(state);
            match repeat.count {
                TrackRepeatCount::Count(count) => {
                    0u8.hash(state);
                    count.hash(state);
                }
                TrackRepeatCount::AutoFill => 1u8.hash(state),
                TrackRepeatCount::AutoFit => 2u8.hash(state),
            }
            repeat.components.len().hash(state);
            for component in &repeat.components {
                hash_grid_track_component(component, state);
            }
        }
        GridTrackComponent::LineNames(names) => {
            2u8.hash(state);
            names.hash(state);
        }
        GridTrackComponent::Subgrid(subgrid) => {
            3u8.hash(state);
            subgrid.name_components().len().hash(state);
            for component in subgrid.name_components() {
                match component {
                    SubgridLineNameComponent::LineNames(names) => {
                        0u8.hash(state);
                        names.hash(state);
                    }
                    SubgridLineNameComponent::Repeat {
                        count,
                        line_name_sets,
                    } => {
                        1u8.hash(state);
                        count.hash(state);
                        line_name_sets.hash(state);
                    }
                }
            }
        }
    }
}

fn hash_grid_flow_tolerance(value: &GridFlowTolerance, state: &mut DefaultHasher) {
    match value {
        GridFlowTolerance::Normal => 0u8.hash(state),
        GridFlowTolerance::Length(length) => {
            1u8.hash(state);
            hash_length(length, state);
        }
        GridFlowTolerance::Percent(value) => {
            2u8.hash(state);
            hash_f32(*value, state);
        }
        GridFlowTolerance::Infinite => 3u8.hash(state),
    }
}

fn hash_track_sizing(value: &TrackSizing, state: &mut DefaultHasher) {
    hash_min_track_sizing(&value.min, state);
    hash_max_track_sizing(&value.max, state);
}

fn hash_min_track_sizing(value: &MinTrackSizing, state: &mut DefaultHasher) {
    match value {
        MinTrackSizing::Length(length) => {
            0u8.hash(state);
            hash_length(length, state);
        }
        MinTrackSizing::Auto => 1u8.hash(state),
        MinTrackSizing::MinContent => 2u8.hash(state),
        MinTrackSizing::MaxContent => 3u8.hash(state),
    }
}

fn hash_max_track_sizing(value: &MaxTrackSizing, state: &mut DefaultHasher) {
    match value {
        MaxTrackSizing::Length(length) => {
            0u8.hash(state);
            hash_length(length, state);
        }
        MaxTrackSizing::Flex(flex) => {
            1u8.hash(state);
            hash_f32(*flex, state);
        }
        MaxTrackSizing::Auto => 2u8.hash(state),
        MaxTrackSizing::MinContent => 3u8.hash(state),
        MaxTrackSizing::MaxContent => 4u8.hash(state),
        MaxTrackSizing::FitContent(length) => {
            5u8.hash(state);
            hash_length(length, state);
        }
    }
}

fn hash_grid_placement(value: &GridPlacement, state: &mut DefaultHasher) {
    hash_grid_line(&value.start, state);
    hash_grid_line(&value.end, state);
}

fn hash_grid_area_placement(value: &GridAreaPlacement, state: &mut DefaultHasher) {
    hash_grid_line(&value.row_start, state);
    hash_grid_line(&value.column_start, state);
    hash_grid_line(&value.row_end, state);
    hash_grid_line(&value.column_end, state);
}

fn hash_grid_line(value: &GridLine, state: &mut DefaultHasher) {
    match value {
        GridLine::Auto => 0u8.hash(state),
        GridLine::Line(line) => {
            1u8.hash(state);
            line.hash(state);
        }
        GridLine::Span(span) => {
            2u8.hash(state);
            span.hash(state);
        }
        GridLine::BareIdent(name) => {
            3u8.hash(state);
            name.hash(state);
        }
        GridLine::NamedLine { name, index } => {
            4u8.hash(state);
            name.hash(state);
            index.hash(state);
        }
        GridLine::NamedSpan { name, index } => {
            5u8.hash(state);
            name.hash(state);
            index.hash(state);
        }
    }
}

fn hash_length(value: &super::Length, state: &mut DefaultHasher) {
    match value {
        super::Length::Normal => 7u8.hash(state),
        super::Length::Px(value) => {
            0u8.hash(state);
            hash_f32(*value, state);
        }
        super::Length::Percent(value) => {
            1u8.hash(state);
            hash_f32(*value, state);
        }
        super::Length::Calc(value) => {
            8u8.hash(state);
            hash_calc_length(value, state);
        }
        super::Length::Fill => 2u8.hash(state),
        super::Length::Fit => 3u8.hash(state),
        super::Length::MinContent => 4u8.hash(state),
        super::Length::MaxContent => 5u8.hash(state),
        super::Length::Auto => 6u8.hash(state),
    }
}

fn hash_calc_length(value: &CalcLength, state: &mut DefaultHasher) {
    match value {
        CalcLength::Px(value) => {
            0u8.hash(state);
            hash_f32(*value, state);
        }
        CalcLength::Percent(value) => {
            1u8.hash(state);
            hash_f32(*value, state);
        }
        CalcLength::Sum(terms) => {
            2u8.hash(state);
            terms.len().hash(state);
            for term in terms {
                hash_calc_term(term, state);
            }
        }
    }
}

fn hash_calc_term(term: &CalcLengthTerm, state: &mut DefaultHasher) {
    term.operator.hash(state);
    hash_calc_length(&term.value, state);
}

fn hash_transform_op(value: &super::TransformOp, state: &mut DefaultHasher) {
    match value {
        super::TransformOp::Translate { x, y } => {
            0u8.hash(state);
            hash_length(x, state);
            hash_length(y, state);
        }
        super::TransformOp::Scale { x, y } => {
            1u8.hash(state);
            hash_f32(*x, state);
            hash_f32(*y, state);
        }
        super::TransformOp::Rotate { radians } => {
            2u8.hash(state);
            hash_f32(*radians, state);
        }
    }
}

fn hash_color(value: Color, state: &mut DefaultHasher) {
    hash_f32(value.r(), state);
    hash_f32(value.g(), state);
    hash_f32(value.b(), state);
    hash_f32(value.a(), state);
}

pub(crate) fn hash_style_color(value: &StyleColor, state: &mut DefaultHasher) {
    match value {
        StyleColor::CurrentColor => 0u8.hash(state),
        StyleColor::Rgba(color) => {
            1u8.hash(state);
            hash_color(*color, state);
        }
        StyleColor::System(color) => {
            2u8.hash(state);
            color.hash(state);
        }
        StyleColor::Hsl {
            hue,
            saturation,
            lightness,
            alpha,
        } => {
            3u8.hash(state);
            hash_color_component(*hue, state);
            hash_color_component(*saturation, state);
            hash_color_component(*lightness, state);
            hash_alpha(*alpha, state);
        }
        StyleColor::Hwb {
            hue,
            whiteness,
            blackness,
            alpha,
        } => {
            4u8.hash(state);
            hash_color_component(*hue, state);
            hash_color_component(*whiteness, state);
            hash_color_component(*blackness, state);
            hash_alpha(*alpha, state);
        }
        StyleColor::Lab {
            lightness,
            a,
            b,
            alpha,
        } => {
            5u8.hash(state);
            hash_color_component(*lightness, state);
            hash_color_component(*a, state);
            hash_color_component(*b, state);
            hash_alpha(*alpha, state);
        }
        StyleColor::Lch {
            lightness,
            chroma,
            hue,
            alpha,
        } => {
            6u8.hash(state);
            hash_color_component(*lightness, state);
            hash_color_component(*chroma, state);
            hash_color_component(*hue, state);
            hash_alpha(*alpha, state);
        }
        StyleColor::Oklab {
            lightness,
            a,
            b,
            alpha,
        } => {
            7u8.hash(state);
            hash_color_component(*lightness, state);
            hash_color_component(*a, state);
            hash_color_component(*b, state);
            hash_alpha(*alpha, state);
        }
        StyleColor::Oklch {
            lightness,
            chroma,
            hue,
            alpha,
        } => {
            8u8.hash(state);
            hash_color_component(*lightness, state);
            hash_color_component(*chroma, state);
            hash_color_component(*hue, state);
            hash_alpha(*alpha, state);
        }
        StyleColor::ColorFunction(value) => {
            9u8.hash(state);
            hash_color_function(value, state);
        }
        StyleColor::ColorMix(value) => {
            10u8.hash(state);
            hash_color_mix(value, state);
        }
        StyleColor::Relative(value) => {
            11u8.hash(state);
            hash_relative_color(value, state);
        }
    }
}

fn hash_color_component(value: super::ColorComponent, state: &mut DefaultHasher) {
    value.get().map(f32::to_bits).hash(state);
}

fn hash_alpha(value: Option<super::Alpha>, state: &mut DefaultHasher) {
    value.map(super::Alpha::get).map(f32::to_bits).hash(state);
}

fn hash_color_function(value: &ColorFunction, state: &mut DefaultHasher) {
    value.space().hash(state);
    for component in value.components() {
        hash_color_component(*component, state);
    }
    hash_alpha(value.alpha(), state);
}

fn hash_color_interpolation_space(value: ColorInterpolationSpace, state: &mut DefaultHasher) {
    value.hash(state);
}

fn hash_color_mix(value: &ColorMix, state: &mut DefaultHasher) {
    hash_color_interpolation_space(value.interpolation().space(), state);
    value.interpolation().hue().hash(state);
    hash_color_mix_component(value.left(), state);
    hash_color_mix_component(value.right(), state);
}

fn hash_color_mix_component(value: &super::ColorMixComponent, state: &mut DefaultHasher) {
    hash_style_color(value.color(), state);
    value.percentage().map(f32::to_bits).hash(state);
}

fn hash_relative_color(value: &RelativeColor, state: &mut DefaultHasher) {
    value.function().hash(state);
    hash_style_color(value.source(), state);
    value.components().len().hash(state);
    for component in value.components() {
        hash_symbolic_component_expression(component, state);
    }
    if let Some(alpha) = value.alpha() {
        true.hash(state);
        hash_symbolic_component_expression(alpha, state);
    } else {
        false.hash(state);
    }
}

fn hash_symbolic_component_expression(
    value: &SymbolicComponentExpression,
    state: &mut DefaultHasher,
) {
    value.authored().as_css().hash(state);
    value.references().len().hash(state);
    for reference in value.references() {
        hash_variable_reference(reference, state);
    }
}

fn hash_variable_reference(value: &VariableReference, state: &mut DefaultHasher) {
    value.name().hash(state);
    if let Some(fallback) = value.fallback() {
        true.hash(state);
        hash_variable_fallback(fallback, state);
    } else {
        false.hash(state);
    }
}

fn hash_variable_fallback(value: &VariableFallback, state: &mut DefaultHasher) {
    value.authored().as_css().hash(state);
    hash_variable_expression(value.expression(), state);
}

fn hash_variable_expression(value: &VariableExpression, state: &mut DefaultHasher) {
    match value {
        VariableExpression::Value(value) => {
            0u8.hash(state);
            hash_value(value, state);
        }
        VariableExpression::CssWideKeyword(value) => {
            1u8.hash(state);
            value.hash(state);
        }
        VariableExpression::Reference(value) => {
            2u8.hash(state);
            hash_variable_reference(value, state);
        }
    }
}

fn hash_slant(value: TextSlant, state: &mut DefaultHasher) {
    match value {
        TextSlant::Normal => 0u8.hash(state),
        TextSlant::Italic => 1u8.hash(state),
        TextSlant::Oblique(angle) => {
            2u8.hash(state);
            angle.map(f32::to_bits).hash(state);
        }
    }
}

fn hash_decoration(value: super::Decoration, state: &mut DefaultHasher) {
    value.enabled().hash(state);
    value.offset().map(f32::to_bits).hash(state);
    value.size().map(f32::to_bits).hash(state);
    if let Some(brush) = value.brush() {
        true.hash(state);
        hash_f32(brush.r(), state);
        hash_f32(brush.g(), state);
        hash_f32(brush.b(), state);
        hash_f32(brush.a(), state);
    } else {
        false.hash(state);
    }
}

fn hash_f32(value: f32, state: &mut DefaultHasher) {
    value.to_bits().hash(state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authored::AuthoredCascadeValue;
    use crate::{
        AlignItems, Alpha, AnimationItem, AnimationList, AuthoredDeclaration, AuthoredDeclarations,
        AuthoredProperty, AuthoredTokens, BackgroundAttachment, BackgroundRepeatStyle, BoxSizing,
        BuiltInCounterStyle, CalcLength, CalcLengthTerm, ColorComponent, ColorInterpolationMethod,
        ColorInterpolationSpace, ColorMix, ColorMixComponent, Content, ContentItem,
        ContentItemList, ContentString, CounterChange, CounterChangeList, CounterChanges,
        CounterFunction, CounterName, CounterStyle, CssWideKeyword, CustomPropertyName, ErrorCode,
        FilterFunctionList, Font, FontFeature, FontFeatureSettings, FontFeatureTag,
        FontFeatureValue, FontStretch, FontVariant, FontWeight, FontWeightNumber,
        GridFlowTolerance, HorizontalPositionKeyword, KeyframesIdent, LetterSpacing, ListStyle,
        ListStyleImage, ListStylePosition, ListStyleType, MaskLayer, OutlineWidthLength,
        OverflowWrap, StyleColor, StyleUrl, SymbolicComponentExpression, SymbolicFunctionValue,
        SystemColor, TextAlignLast, TextDecoration, TextDecorationLine,
        TextDecorationLineComponent, TextDecorationStyle, TextDecorationThickness, TextIndent,
        TextOverflow, TextTransform, TextWrap, UserSelect, VariableExpression, VariableFallback,
        VariableReference, VerticalAlign, VerticalPositionKeyword, WhiteSpace, WordBreak,
    };

    fn value_hash(value: &Value) -> u64 {
        let mut hasher = DefaultHasher::new();
        hash_value(value, &mut hasher);
        hasher.finish()
    }

    #[test]
    fn value_hash_distinguishes_grid_flow_tolerance_from_box_sizing() {
        assert_ne!(
            value_hash(&Value::GridFlowTolerance(GridFlowTolerance::Normal)),
            value_hash(&Value::BoxSizing(BoxSizing::ContentBox))
        );
    }

    #[test]
    fn value_hash_distinguishes_calc_lengths() {
        let calc_a = CalcLength::sum(
            CalcLengthTerm::add(CalcLength::px(20.0)),
            [CalcLengthTerm::add(CalcLength::percent(10.0))],
        );
        let calc_b = CalcLength::sum(
            CalcLengthTerm::add(CalcLength::px(21.0)),
            [CalcLengthTerm::add(CalcLength::percent(10.0))],
        );

        assert_ne!(
            value_hash(&Value::Length(Length::Calc(calc_a))),
            value_hash(&Value::Length(Length::Calc(calc_b)))
        );
    }

    #[test]
    fn background_layer_properties_accept_symbolic_values() {
        let images =
            ImageLayerList::try_new([ImageLayer::url(StyleUrl::new("hero.png").unwrap())]).unwrap();
        let position = Position::try_new([
            PositionComponent::Horizontal(HorizontalPositionKeyword::Left),
            PositionComponent::Length(Length::Percent(25.0)),
        ])
        .unwrap();
        let positions = PositionList::try_new([position.clone()]).unwrap();
        let size = BackgroundSize::Explicit {
            width: BackgroundSizeComponent::Length(Length::Percent(100.0)),
            height: Some(BackgroundSizeComponent::Auto),
        };
        let sizes = BackgroundSizeList::try_new([size.clone()]).unwrap();

        let declarations = Declarations::new()
            .background_image(images.clone())
            .background_position(positions.clone())
            .background_size(sizes.clone())
            .background_repeat(BackgroundRepeatList::try_new([BackgroundRepeat::RepeatX]).unwrap())
            .background_origin(BackgroundBox::PaddingBox)
            .background_clip(BackgroundBox::ContentBox)
            .background_attachment(
                BackgroundAttachmentList::try_new([BackgroundAttachment::Fixed]).unwrap(),
            );

        assert_eq!(
            declarations.get(Property::BackgroundImage),
            Some(&Value::ImageLayerList(images))
        );
        assert_eq!(
            declarations.get(Property::BackgroundPosition),
            Some(&Value::PositionList(positions))
        );
        assert_eq!(
            declarations.get(Property::BackgroundSize),
            Some(&Value::BackgroundSizeList(sizes))
        );
    }

    #[test]
    fn effect_properties_accept_symbolic_values() {
        let filter = Filter::Functions(
            FilterFunctionList::try_new([FilterFunction::Blur(
                SymbolicFunctionValue::new("4px").unwrap(),
            )])
            .unwrap(),
        );
        let clip = ClipPath::BasicShape(BasicShape::Circle(
            SymbolicFunctionValue::new("50%").unwrap(),
        ));

        let declarations = Declarations::new()
            .box_decoration_break(BoxDecorationBreak::Clone)
            .filter(filter.clone())
            .backdrop_filter(Filter::None)
            .clip_path(clip.clone());

        assert_eq!(
            declarations.get(Property::BoxDecorationBreak),
            Some(&Value::BoxDecorationBreak(BoxDecorationBreak::Clone))
        );
        assert_eq!(
            declarations.get(Property::Filter),
            Some(&Value::Filter(filter))
        );
        assert_eq!(
            declarations.get(Property::BackdropFilter),
            Some(&Value::Filter(Filter::None))
        );
        assert_eq!(
            declarations.get(Property::ClipPath),
            Some(&Value::ClipPath(clip))
        );
    }

    #[test]
    fn individual_transform_properties_accept_symbolic_values() {
        let translate = Translate::try_values([Length::Px(10.0), Length::Percent(5.0)]).unwrap();
        let scale = Scale::try_values([1.0, 2.0]).unwrap();
        let rotate = Rotate::Value(SymbolicFunctionValue::new("45deg").unwrap());

        let declarations = Declarations::new()
            .translate(translate.clone())
            .rotate(rotate.clone())
            .scale(scale.clone());

        assert_eq!(
            declarations.get(Property::Translate),
            Some(&Value::Translate(translate))
        );
        assert_eq!(
            declarations.get(Property::Rotate),
            Some(&Value::Rotate(rotate))
        );
        assert_eq!(
            declarations.get(Property::Scale),
            Some(&Value::Scale(scale))
        );
    }

    #[test]
    fn interaction_properties_accept_typed_values() {
        let declarations = Declarations::new()
            .cursor(Cursor::Pointer)
            .pointer_events(PointerEvents::None)
            .user_select(UserSelect::All);

        assert_eq!(
            declarations.get(Property::Cursor),
            Some(&Value::Cursor(Cursor::Pointer))
        );
        assert_eq!(
            declarations.get(Property::PointerEvents),
            Some(&Value::PointerEvents(PointerEvents::None))
        );
        assert_eq!(
            declarations.get(Property::UserSelect),
            Some(&Value::UserSelect(UserSelect::All))
        );

        let metadata = Property::UserSelect.metadata();
        assert!(!metadata.is_inherited());
        assert!(!metadata.impact_flags().affects_layout());
        assert!(metadata.impact_flags().affects_paint());
        assert!(!metadata.impact_flags().affects_text());
        assert_eq!(metadata.default(), &Value::UserSelect(UserSelect::Auto));
    }

    #[test]
    fn mask_shorthand_lowers_to_layer_longhands() {
        let layer = MaskLayer::try_new(
            Some(ImageLayer::url(StyleUrl::new("mask.svg").unwrap())),
            Some(
                Position::try_new([PositionComponent::Vertical(VerticalPositionKeyword::Top)])
                    .unwrap(),
            ),
            Some(BackgroundSize::Contain),
            Some(BackgroundRepeat::Axes {
                x: BackgroundRepeatStyle::NoRepeat,
                y: BackgroundRepeatStyle::NoRepeat,
            }),
        )
        .unwrap();
        let mask = MaskLayerList::try_new([layer]).unwrap();
        let declarations = Declarations::new().mask(mask.clone()).unwrap();

        assert_eq!(declarations.get(Property::Mask), None);
        assert!(matches!(
            declarations.get(Property::MaskImage),
            Some(Value::ImageLayerList(_))
        ));
        assert!(matches!(
            declarations.get(Property::MaskPosition),
            Some(Value::PositionList(_))
        ));
        assert!(matches!(
            declarations.get(Property::MaskSize),
            Some(Value::BackgroundSizeList(_))
        ));
        assert!(matches!(
            declarations.get(Property::MaskRepeat),
            Some(Value::BackgroundRepeatList(_))
        ));
    }

    #[test]
    fn list_style_shorthand_lowers_to_marker_longhands() {
        let marker_type = ListStyleType::String(ContentString::try_new("->").unwrap());
        let image = ListStyleImage::Url(StyleUrl::new("marker.svg").unwrap());
        let declarations = Declarations::new()
            .list_style(
                ListStyle::try_new(
                    Some(marker_type.clone()),
                    Some(ListStylePosition::Inside),
                    Some(image.clone()),
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(declarations.get(Property::ListStyle), None);
        assert_eq!(
            declarations.get(Property::ListStyleType),
            Some(&Value::ListStyleType(marker_type))
        );
        assert_eq!(
            declarations.get(Property::ListStylePosition),
            Some(&Value::ListStylePosition(ListStylePosition::Inside))
        );
        assert_eq!(
            declarations.get(Property::ListStyleImage),
            Some(&Value::ListStyleImage(image))
        );
    }

    #[test]
    fn list_style_shorthand_resets_omitted_components_to_defaults() {
        let declarations = Declarations::new()
            .list_style(ListStyle::try_new(Some(ListStyleType::None), None, None).unwrap())
            .unwrap();

        assert_eq!(
            declarations.get(Property::ListStyleType),
            Some(&Value::ListStyleType(ListStyleType::None))
        );
        assert_eq!(
            declarations.get(Property::ListStylePosition),
            Some(&Value::ListStylePosition(ListStylePosition::Outside))
        );
        assert_eq!(
            declarations.get(Property::ListStyleImage),
            Some(&Value::ListStyleImage(ListStyleImage::None))
        );
    }

    #[test]
    fn generated_content_and_counters_accept_typed_values() {
        let counter_name = CounterName::try_new("section").unwrap();
        let content = Content::Items(
            ContentItemList::try_new([
                ContentItem::String(ContentString::try_new("Section ").unwrap()),
                ContentItem::Counter(CounterFunction::new(
                    counter_name.clone(),
                    Some(CounterStyle::BuiltIn(BuiltInCounterStyle::Decimal)),
                )),
            ])
            .unwrap(),
        );
        let changes = CounterChanges::Changes(
            CounterChangeList::try_new([CounterChange::new(counter_name, 1)]).unwrap(),
        );

        let declarations = Declarations::new()
            .content(content.clone())
            .unwrap()
            .counter_increment(changes.clone())
            .unwrap();

        assert_eq!(
            declarations.get(Property::Content),
            Some(&Value::Content(content))
        );
        assert_eq!(
            declarations.get(Property::CounterIncrement),
            Some(&Value::CounterChanges(changes))
        );
    }

    #[test]
    fn inline_text_properties_accept_typed_values() {
        let declarations = Declarations::new()
            .text_align_last(TextAlignLast::Justify)
            .try_text_indent(TextIndent::new(Length::Percent(12.5), true, false).unwrap())
            .unwrap()
            .try_vertical_align(VerticalAlign::try_length(Length::Px(-2.0)).unwrap())
            .unwrap()
            .try_letter_spacing(LetterSpacing::try_length(Length::Px(1.5)).unwrap())
            .unwrap()
            .text_transform(TextTransform::Uppercase);

        assert_eq!(
            declarations.get(Property::TextAlignLast),
            Some(&Value::TextAlignLast(TextAlignLast::Justify))
        );
        assert_eq!(
            declarations.get(Property::TextIndent),
            Some(&Value::TextIndent(
                TextIndent::new(Length::Percent(12.5), true, false).unwrap()
            ))
        );
        assert_eq!(
            declarations.get(Property::VerticalAlign),
            Some(&Value::VerticalAlign(
                VerticalAlign::try_length(Length::Px(-2.0)).unwrap()
            ))
        );
        assert_eq!(
            declarations.get(Property::LetterSpacing),
            Some(&Value::LetterSpacing(
                LetterSpacing::try_length(Length::Px(1.5)).unwrap()
            ))
        );
        assert_eq!(
            declarations.get(Property::TextTransform),
            Some(&Value::TextTransform(TextTransform::Uppercase))
        );
    }

    #[test]
    fn text_flow_properties_accept_typed_values() {
        let declarations = Declarations::new()
            .text_wrap(TextWrap::Balance)
            .white_space(WhiteSpace::BreakSpaces)
            .word_break(WordBreak::BreakWord)
            .overflow_wrap(OverflowWrap::Anywhere)
            .text_overflow(TextOverflow::Ellipsis);

        assert_eq!(
            declarations.get(Property::TextWrap),
            Some(&Value::TextWrap(TextWrap::Balance))
        );
        assert_eq!(
            declarations.get(Property::WhiteSpace),
            Some(&Value::WhiteSpace(WhiteSpace::BreakSpaces))
        );
        assert_eq!(
            declarations.get(Property::WordBreak),
            Some(&Value::WordBreak(WordBreak::BreakWord))
        );
        assert_eq!(
            declarations.get(Property::OverflowWrap),
            Some(&Value::OverflowWrap(OverflowWrap::Anywhere))
        );
        assert_eq!(
            declarations.get(Property::TextOverflow),
            Some(&Value::TextOverflow(TextOverflow::Ellipsis))
        );
    }

    #[test]
    fn text_decoration_shorthand_lowers_to_canonical_longhands() {
        let line = TextDecorationLine::try_new([
            TextDecorationLineComponent::Underline,
            TextDecorationLineComponent::LineThrough,
        ])
        .unwrap();
        let thickness = TextDecorationThickness::try_length(Length::Px(2.0)).unwrap();
        let decoration = TextDecoration::try_new(
            Some(line.clone()),
            None,
            Some(TextDecorationStyle::Wavy),
            Some(thickness.clone()),
        )
        .unwrap();

        let declarations = Declarations::new().try_text_decoration(decoration).unwrap();

        assert_eq!(declarations.get(Property::TextDecoration), None);
        assert_eq!(
            declarations.get(Property::TextDecorationLine),
            Some(&Value::TextDecorationLine(line))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationStyle),
            Some(&Value::TextDecorationStyle(TextDecorationStyle::Wavy))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationThickness),
            Some(&Value::TextDecorationThickness(thickness))
        );
    }

    #[test]
    fn transition_shorthand_lowers_to_typed_lists() {
        let declarations = Declarations::new()
            .transition(
                TransitionList::try_new([TransitionItem::try_new(
                    Some(TransitionPropertyTarget::Property(Property::Opacity)),
                    Some(DurationSeconds::new(0.2).unwrap()),
                    Some(DurationSeconds::new(0.05).unwrap()),
                    Some(EasingFunction::EaseOut),
                )
                .unwrap()])
                .unwrap(),
            )
            .unwrap();

        assert_eq!(declarations.get(Property::Transition), None);
        assert!(matches!(
            declarations.get(Property::TransitionProperty),
            Some(Value::TransitionPropertyList(_))
        ));
        assert!(matches!(
            declarations.get(Property::TransitionDuration),
            Some(Value::TimeList(_))
        ));
        assert!(matches!(
            declarations.get(Property::TransitionTimingFunction),
            Some(Value::EasingList(_))
        ));
    }

    #[test]
    fn transition_shorthand_resets_omitted_components_to_defaults() {
        let declarations = Declarations::new()
            .transition(
                TransitionList::try_new([TransitionItem::try_new(
                    Some(TransitionPropertyTarget::Property(Property::Opacity)),
                    None,
                    None,
                    None,
                )
                .unwrap()])
                .unwrap(),
            )
            .unwrap();

        let durations = declarations
            .get(Property::TransitionDuration)
            .and_then(|value| match value {
                Value::TimeList(values) => Some(values),
                _ => None,
            })
            .unwrap();

        assert_eq!(durations.seconds()[0].get(), 0.0);
    }

    #[test]
    fn animation_shorthand_lowers_to_typed_longhand_lists() {
        let fade = AnimationName::Keyframes(KeyframesName::Ident(
            KeyframesIdent::try_new("fade-in").unwrap(),
        ));
        let slide = AnimationName::Keyframes(KeyframesName::Ident(
            KeyframesIdent::try_new("slide-in").unwrap(),
        ));

        let declarations = Declarations::new()
            .animation(
                AnimationList::try_new([
                    AnimationItem::try_new(
                        Some(fade.clone()),
                        Some(DurationSeconds::new(0.3).unwrap()),
                        Some(DurationSeconds::new(0.1).unwrap()),
                        Some(EasingFunction::EaseInOut),
                        Some(AnimationIterationCount::Infinite),
                        Some(AnimationDirection::Alternate),
                        Some(AnimationFillMode::Both),
                        Some(AnimationPlayState::Paused),
                    )
                    .unwrap(),
                    AnimationItem::try_new(
                        Some(slide.clone()),
                        Some(DurationSeconds::new(0.6).unwrap()),
                        Some(DurationSeconds::new(0.2).unwrap()),
                        Some(EasingFunction::Linear),
                        Some(AnimationIterationCount::Number(
                            AnimationIterationNumber::try_new(2.0).unwrap(),
                        )),
                        Some(AnimationDirection::Reverse),
                        Some(AnimationFillMode::Forwards),
                        Some(AnimationPlayState::Running),
                    )
                    .unwrap(),
                ])
                .unwrap(),
            )
            .unwrap();

        assert_eq!(declarations.get(Property::Animation), None);
        assert_eq!(
            declarations
                .iter()
                .map(Declaration::property)
                .collect::<Vec<_>>(),
            vec![
                Property::AnimationName,
                Property::AnimationDuration,
                Property::AnimationDelay,
                Property::AnimationTimingFunction,
                Property::AnimationIterationCount,
                Property::AnimationDirection,
                Property::AnimationFillMode,
                Property::AnimationPlayState,
            ]
        );
        assert!(matches!(
            declarations.get(Property::AnimationName),
            Some(Value::AnimationNameList(values)) if values.names() == [fade, slide]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationDuration),
            Some(Value::TimeList(values))
                if values.seconds().iter().map(|value| value.get()).collect::<Vec<_>>()
                    == vec![0.3, 0.6]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationIterationCount),
            Some(Value::AnimationIterationCountList(values)) if values.values().len() == 2
        ));
        assert!(matches!(
            declarations.get(Property::AnimationDirection),
            Some(Value::AnimationDirectionList(values))
                if values.values() == [AnimationDirection::Alternate, AnimationDirection::Reverse]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationFillMode),
            Some(Value::AnimationFillModeList(values))
                if values.values() == [AnimationFillMode::Both, AnimationFillMode::Forwards]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationPlayState),
            Some(Value::AnimationPlayStateList(values))
                if values.values() == [AnimationPlayState::Paused, AnimationPlayState::Running]
        ));
    }

    #[test]
    fn animation_shorthand_resets_omitted_components_to_defaults() {
        let declarations = Declarations::new()
            .animation(
                AnimationList::try_new([AnimationItem::try_new(
                    Some(AnimationName::Keyframes(KeyframesName::Ident(
                        KeyframesIdent::try_new("fade-in").unwrap(),
                    ))),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap()])
                .unwrap(),
            )
            .unwrap();

        assert!(matches!(
            declarations.get(Property::AnimationDuration),
            Some(Value::TimeList(values)) if values.seconds()[0].get() == 0.0
        ));
        assert!(matches!(
            declarations.get(Property::AnimationDelay),
            Some(Value::TimeList(values)) if values.seconds()[0].get() == 0.0
        ));
        assert!(matches!(
            declarations.get(Property::AnimationTimingFunction),
            Some(Value::EasingList(values)) if values.values() == [EasingFunction::Ease]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationIterationCount),
            Some(Value::AnimationIterationCountList(values))
                if matches!(
                    values.values(),
                    [AnimationIterationCount::Number(number)] if number.get() == 1.0
                )
        ));
        assert!(matches!(
            declarations.get(Property::AnimationDirection),
            Some(Value::AnimationDirectionList(values))
                if values.values() == [AnimationDirection::Normal]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationFillMode),
            Some(Value::AnimationFillModeList(values)) if values.values() == [AnimationFillMode::None]
        ));
        assert!(matches!(
            declarations.get(Property::AnimationPlayState),
            Some(Value::AnimationPlayStateList(values))
                if values.values() == [AnimationPlayState::Running]
        ));
    }

    #[test]
    fn color_properties_accept_symbolic_style_colors() {
        let decoration = StyleColor::current_color();
        let rgba = StyleColor::rgba(Color::try_rgba(0.2, 0.4, 0.6, 1.0).unwrap());

        let declarations = Declarations::new()
            .try_text_color(rgba.clone())
            .unwrap()
            .try_background_color(StyleColor::current_color())
            .unwrap()
            .try_text_decoration_color(decoration.clone())
            .unwrap();

        assert_eq!(
            declarations.get(Property::Color),
            Some(&Value::StyleColor(rgba))
        );
        assert_eq!(
            declarations.get(Property::Background),
            Some(&Value::StyleColor(StyleColor::current_color()))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationColor),
            Some(&Value::StyleColor(decoration))
        );
    }

    #[test]
    fn text_decoration_shorthand_lowers_color_with_existing_components() {
        let line = TextDecorationLine::try_new([TextDecorationLineComponent::Underline]).unwrap();
        let color = StyleColor::system(SystemColor::CanvasText);
        let thickness = TextDecorationThickness::try_length(Length::Px(2.0)).unwrap();
        let decoration = TextDecoration::try_new(
            Some(line.clone()),
            Some(color.clone()),
            Some(TextDecorationStyle::Wavy),
            Some(thickness.clone()),
        )
        .unwrap();

        let declarations = Declarations::new().try_text_decoration(decoration).unwrap();

        assert_eq!(declarations.get(Property::TextDecoration), None);
        assert_eq!(
            declarations.get(Property::TextDecorationLine),
            Some(&Value::TextDecorationLine(line))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationColor),
            Some(&Value::StyleColor(color))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationStyle),
            Some(&Value::TextDecorationStyle(TextDecorationStyle::Wavy))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationThickness),
            Some(&Value::TextDecorationThickness(thickness))
        );
    }

    #[test]
    fn text_decoration_css_wide_expands_to_color_longhand() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::TextDecoration),
            CssWideKeyword::Initial,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::TextDecoration), None);
        for property in [
            Property::TextDecorationLine,
            Property::TextDecorationColor,
            Property::TextDecorationStyle,
            Property::TextDecorationThickness,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Initial
                ))
            );
        }
    }

    #[test]
    fn border_shorthands_lower_to_side_longhands() {
        let color = StyleColor::rgba(Color::BLACK);
        let border = Border::try_new(
            Some(Length::Px(2.0)),
            Some(BorderLineStyle::Dashed),
            Some(color.clone()),
        )
        .unwrap();

        let declarations = Declarations::new().try_border(border).unwrap();

        assert_eq!(declarations.get(Property::Border), None);
        for property in [
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
        ] {
            assert_eq!(
                declarations.get(property),
                Some(&Value::Length(Length::Px(2.0)))
            );
        }
        for property in [
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
        ] {
            assert_eq!(
                declarations.get(property),
                Some(&Value::BorderLineStyle(BorderLineStyle::Dashed))
            );
        }
        for property in [
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ] {
            assert_eq!(
                declarations.get(property),
                Some(&Value::StyleColor(color.clone()))
            );
        }
    }

    #[test]
    fn border_style_shorthand_lowers_to_side_styles() {
        let styles = BorderStyles::new(
            BorderLineStyle::Solid,
            BorderLineStyle::Dashed,
            BorderLineStyle::Dotted,
            BorderLineStyle::Double,
        );

        let declarations = Declarations::new().border_style(styles);

        assert_eq!(declarations.get(Property::BorderStyle), None);
        assert_eq!(
            declarations.get(Property::BorderTopStyle),
            Some(&Value::BorderLineStyle(BorderLineStyle::Solid))
        );
        assert_eq!(
            declarations.get(Property::BorderRightStyle),
            Some(&Value::BorderLineStyle(BorderLineStyle::Dashed))
        );
        assert_eq!(
            declarations.get(Property::BorderBottomStyle),
            Some(&Value::BorderLineStyle(BorderLineStyle::Dotted))
        );
        assert_eq!(
            declarations.get(Property::BorderLeftStyle),
            Some(&Value::BorderLineStyle(BorderLineStyle::Double))
        );
    }

    #[test]
    fn border_css_wide_expands_to_all_border_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Border),
            CssWideKeyword::Unset,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::Border), None);
        for property in [
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
            );
        }
    }

    #[test]
    fn border_side_css_wide_expands_to_side_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::BorderTop),
            CssWideKeyword::Inherit,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::BorderTop), None);
        for property in [
            Property::BorderTopWidth,
            Property::BorderTopStyle,
            Property::BorderTopColor,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Inherit
                ))
            );
        }
        assert_eq!(canonical.get(Property::BorderRightWidth), None);
        assert_eq!(canonical.get(Property::BorderRightStyle), None);
        assert_eq!(canonical.get(Property::BorderRightColor), None);
    }

    #[test]
    fn border_style_css_wide_expands_to_side_styles() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::BorderStyle),
            CssWideKeyword::Inherit,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::BorderStyle), None);
        for property in [
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Inherit
                ))
            );
        }
    }

    #[test]
    fn border_color_shorthand_lowers_to_side_colors() {
        let color = StyleColor::system(SystemColor::CanvasText);

        let declarations = Declarations::new().try_border_color(color.clone()).unwrap();

        assert_eq!(declarations.get(Property::BorderColor), None);
        for property in [
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ] {
            assert_eq!(
                declarations.get(property),
                Some(&Value::StyleColor(color.clone()))
            );
        }
    }

    #[test]
    fn border_color_css_wide_expands_to_side_colors() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::BorderColor),
            CssWideKeyword::RevertLayer,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::BorderColor), None);
        for property in [
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::RevertLayer
                ))
            );
        }
    }

    #[test]
    fn border_radius_shorthand_lowers_to_individual_corners() {
        let top_left = CornerRadius::new(Length::Px(4.0), Length::Percent(40.0)).unwrap();
        let top_right = CornerRadius::new(Length::Px(8.0), Length::Percent(50.0)).unwrap();
        let bottom_right = CornerRadius::new(Length::Px(12.0), Length::Percent(60.0)).unwrap();
        let bottom_left = CornerRadius::new(Length::Px(16.0), Length::Percent(70.0)).unwrap();
        let radii = BorderRadii::new(
            top_left.clone(),
            top_right.clone(),
            bottom_right.clone(),
            bottom_left.clone(),
        );

        let declarations = Declarations::new().try_border_radius(radii).unwrap();

        assert_eq!(declarations.get(Property::Radius), None);
        assert_eq!(
            declarations.get(Property::BorderTopLeftRadius),
            Some(&Value::CornerRadius(top_left))
        );
        assert_eq!(
            declarations.get(Property::BorderTopRightRadius),
            Some(&Value::CornerRadius(top_right))
        );
        assert_eq!(
            declarations.get(Property::BorderBottomRightRadius),
            Some(&Value::CornerRadius(bottom_right))
        );
        assert_eq!(
            declarations.get(Property::BorderBottomLeftRadius),
            Some(&Value::CornerRadius(bottom_left))
        );
    }

    #[test]
    fn border_radius_css_wide_expands_to_corner_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Radius),
            CssWideKeyword::RevertLayer,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::Radius), None);
        for property in [
            Property::BorderTopLeftRadius,
            Property::BorderTopRightRadius,
            Property::BorderBottomRightRadius,
            Property::BorderBottomLeftRadius,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::RevertLayer
                ))
            );
        }
    }

    #[test]
    fn outline_shorthand_lowers_to_longhands() {
        let outline = Outline::try_new(
            Some(OutlineWidth::Length(
                OutlineWidthLength::new(Length::Px(3.0)).unwrap(),
            )),
            Some(OutlineStyle::Border(BorderLineStyle::Dotted)),
            Some(StyleColor::current_color()),
        )
        .unwrap();

        let declarations = Declarations::new().try_outline(outline).unwrap();

        assert_eq!(declarations.get(Property::Outline), None);
        assert!(matches!(
            declarations.get(Property::OutlineWidth),
            Some(Value::OutlineWidth(OutlineWidth::Length(_)))
        ));
        assert_eq!(
            declarations.get(Property::OutlineStyle),
            Some(&Value::OutlineStyle(OutlineStyle::Border(
                BorderLineStyle::Dotted
            )))
        );
        assert_eq!(
            declarations.get(Property::OutlineColor),
            Some(&Value::StyleColor(StyleColor::current_color()))
        );
    }

    #[test]
    fn outline_css_wide_expands_to_outline_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Outline),
            CssWideKeyword::Initial,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::Outline), None);
        for property in [
            Property::OutlineWidth,
            Property::OutlineStyle,
            Property::OutlineColor,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Initial
                ))
            );
        }
    }

    #[test]
    fn border_radius_supports_individual_elliptical_corners() {
        let radius = CornerRadius::new(Length::Px(4.0), Length::Percent(50.0)).unwrap();
        let declarations = Declarations::new()
            .try_border_top_left_radius(radius.clone())
            .unwrap();

        assert_eq!(
            declarations.get(Property::BorderTopLeftRadius),
            Some(&Value::CornerRadius(radius))
        );
    }

    #[test]
    fn border_side_shorthand_resets_omitted_components() {
        let border = Border::try_new(Some(Length::Px(5.0)), None, None).unwrap();
        let declarations = Declarations::new()
            .try_border_top_width(Length::Px(9.0))
            .unwrap()
            .border_top_style(BorderLineStyle::Dashed)
            .try_border_top_color(StyleColor::system(SystemColor::LinkText))
            .unwrap()
            .try_border_top(border)
            .unwrap();

        assert_eq!(
            declarations.get(Property::BorderTopWidth),
            Some(&Value::Length(Length::Px(5.0)))
        );
        assert_eq!(
            declarations.get(Property::BorderTopStyle),
            Some(&Value::BorderLineStyle(BorderLineStyle::None))
        );
        assert_eq!(
            declarations.get(Property::BorderTopColor),
            Some(&Value::StyleColor(StyleColor::current_color()))
        );
        assert_eq!(declarations.get(Property::BorderRightWidth), None);
        assert_eq!(declarations.get(Property::BorderRightStyle), None);
        assert_eq!(declarations.get(Property::BorderRightColor), None);
    }

    #[test]
    fn border_shorthand_resets_omitted_components_to_defaults() {
        let border = Border::try_new(Some(Length::Px(5.0)), None, None).unwrap();
        let declarations = Declarations::new()
            .border_top_style(BorderLineStyle::Dashed)
            .try_border_top_color(StyleColor::system(SystemColor::LinkText))
            .unwrap()
            .try_border(border)
            .unwrap();

        assert_eq!(
            declarations.get(Property::BorderTopWidth),
            Some(&Value::Length(Length::Px(5.0)))
        );
        assert_eq!(
            declarations.get(Property::BorderTopStyle),
            Some(&Value::BorderLineStyle(BorderLineStyle::None))
        );
        assert_eq!(
            declarations.get(Property::BorderTopColor),
            Some(&Value::StyleColor(StyleColor::current_color()))
        );
    }

    #[test]
    fn outline_shorthand_resets_omitted_components_to_defaults() {
        let outline = Outline::try_new(Some(OutlineWidth::Thick), None, None).unwrap();
        let declarations = Declarations::new()
            .outline_style(OutlineStyle::Auto)
            .try_outline_color(StyleColor::system(SystemColor::LinkText))
            .unwrap()
            .try_outline(outline)
            .unwrap();

        assert_eq!(
            declarations.get(Property::OutlineWidth),
            Some(&Value::OutlineWidth(OutlineWidth::Thick))
        );
        assert_eq!(
            declarations.get(Property::OutlineStyle),
            Some(&Value::OutlineStyle(OutlineStyle::Border(
                BorderLineStyle::None
            )))
        );
        assert_eq!(
            declarations.get(Property::OutlineColor),
            Some(&Value::StyleColor(StyleColor::current_color()))
        );
    }

    #[test]
    fn symbolic_color_components_validate_domains() {
        assert!(Alpha::new(0.0).is_ok());
        assert!(Alpha::new(1.0).is_ok());
        assert!(Alpha::new(-0.1).is_err());
        assert!(Alpha::new(1.1).is_err());
        assert!(Alpha::new(f32::NAN).is_err());

        assert!(ColorComponent::new(Some(0.0)).is_ok());
        assert!(ColorComponent::new(Some(f32::NAN)).is_err());

        let left = ColorMixComponent::try_new(StyleColor::current_color(), Some(25.0)).unwrap();
        let right = ColorMixComponent::try_new(StyleColor::rgba(Color::BLACK), None).unwrap();
        let mix = StyleColor::color_mix(ColorMix::new(
            ColorInterpolationMethod::new(ColorInterpolationSpace::Oklab, None),
            left,
            right,
        ));
        assert!(matches!(mix, StyleColor::ColorMix(_)));
        assert!(ColorMixComponent::try_new(StyleColor::current_color(), Some(101.0)).is_err());
    }

    #[test]
    fn relative_color_component_dependencies_include_nested_fallbacks() {
        let primary = CustomPropertyName::try_new("--a").unwrap();
        let fallback = CustomPropertyName::try_new("--b").unwrap();
        let expression = SymbolicComponentExpression::new(
            AuthoredTokens::new("calc(var(--a, var(--b)) + 1)"),
            [VariableReference::new(
                primary.clone(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("var(--b)"),
                    VariableExpression::Reference(VariableReference::new(fallback.clone(), None)),
                )),
            )],
        )
        .unwrap();

        assert_eq!(expression.dependencies(), vec![primary, fallback]);
    }

    #[test]
    fn text_decoration_shorthand_resets_omitted_components_to_defaults() {
        let line = TextDecorationLine::try_new([TextDecorationLineComponent::Underline]).unwrap();
        let decoration = TextDecoration::try_new(Some(line.clone()), None, None, None).unwrap();

        let declarations = Declarations::new()
            .text_decoration_style(TextDecorationStyle::Wavy)
            .try_text_decoration_thickness(TextDecorationThickness::FromFont)
            .unwrap()
            .try_text_decoration(decoration)
            .unwrap();

        assert_eq!(declarations.get(Property::TextDecoration), None);
        assert_eq!(
            declarations.get(Property::TextDecorationLine),
            Some(&Value::TextDecorationLine(line))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationStyle),
            Some(&Value::TextDecorationStyle(TextDecorationStyle::default()))
        );
        assert_eq!(
            declarations.get(Property::TextDecorationThickness),
            Some(&Value::TextDecorationThickness(
                TextDecorationThickness::default()
            ))
        );
    }

    #[test]
    fn text_decoration_models_validate_domains() {
        assert!(TextDecorationLine::none().is_none());
        assert!(
            TextDecorationLine::try_new([
                TextDecorationLineComponent::Underline,
                TextDecorationLineComponent::Underline,
            ])
            .is_err()
        );
        assert!(TextDecoration::try_new(None, None, None, None).is_err());
        assert!(TextDecorationThickness::try_length(Length::Px(0.0)).is_ok());
        assert!(TextDecorationThickness::try_length(Length::Percent(10.0)).is_ok());
        assert!(TextDecorationThickness::try_length(Length::Px(-1.0)).is_err());
        assert!(
            TextDecorationThickness::try_length(Length::Calc(CalcLength::sum(
                CalcLengthTerm::add(CalcLength::px(0.0)),
                [CalcLengthTerm::sub(CalcLength::px(1.0))]
            )))
            .is_err()
        );
        assert!(TextDecorationThickness::try_length(Length::Auto).is_err());
    }

    #[test]
    fn inline_text_values_validate_length_domains() {
        assert!(TextIndent::new(Length::Auto, false, false).is_err());
        assert!(VerticalAlign::try_length(Length::Auto).is_err());
        assert!(LetterSpacing::try_length(Length::Percent(10.0)).is_err());
        assert!(LetterSpacing::try_length(Length::Normal).is_err());
        assert!(
            LetterSpacing::try_length(Length::Calc(CalcLength::sum(
                CalcLengthTerm::add(CalcLength::percent(50.0)),
                [CalcLengthTerm::add(CalcLength::px(1.0))]
            )))
            .is_err()
        );
    }

    #[test]
    fn calc_lengths_validate_through_length_properties() {
        let calc = CalcLength::sum(
            CalcLengthTerm::add(CalcLength::px(20.0)),
            [CalcLengthTerm::add(CalcLength::percent(10.0))],
        );

        Declaration::try_new(Property::Width, Value::Length(Length::Calc(calc))).unwrap();
    }

    #[test]
    fn flex_shorthand_lowers_to_grow_shrink_and_basis() {
        let declarations = Declarations::new().try_flex(Flex::none()).unwrap();
        assert_eq!(declarations.get(Property::Flex), None);
        assert_eq!(
            declarations.get(Property::FlexGrow),
            Some(&Value::FlexFactor(FlexFactor::zero()))
        );
        assert_eq!(
            declarations.get(Property::FlexShrink),
            Some(&Value::FlexFactor(FlexFactor::zero()))
        );
        assert_eq!(
            declarations.get(Property::FlexBasis),
            Some(&Value::Length(Length::Auto))
        );

        let declarations = Declarations::new().try_flex(Flex::auto()).unwrap();
        assert_eq!(
            declarations.get(Property::FlexGrow),
            Some(&Value::FlexFactor(FlexFactor::one()))
        );
        assert_eq!(
            declarations.get(Property::FlexShrink),
            Some(&Value::FlexFactor(FlexFactor::one()))
        );
        assert_eq!(
            declarations.get(Property::FlexBasis),
            Some(&Value::Length(Length::Auto))
        );

        let declarations = Declarations::new()
            .try_flex(Flex::components(FlexFactor::new(2.0).unwrap(), None, None))
            .unwrap();
        assert_eq!(
            declarations.get(Property::FlexGrow),
            Some(&Value::FlexFactor(FlexFactor::new(2.0).unwrap()))
        );
        assert_eq!(
            declarations.get(Property::FlexShrink),
            Some(&Value::FlexFactor(FlexFactor::one()))
        );
        assert_eq!(
            declarations.get(Property::FlexBasis),
            Some(&Value::Length(Length::ZERO))
        );
    }

    #[test]
    fn flex_shorthand_validates_canonical_basis_domain() {
        assert!(
            Declarations::new()
                .try_flex(Flex::components(
                    FlexFactor::one(),
                    None,
                    Some(Length::Normal),
                ))
                .is_err()
        );
        assert!(
            Declarations::new()
                .try_flex(Flex::components(
                    FlexFactor::one(),
                    None,
                    Some(Length::Px(-1.0)),
                ))
                .is_err()
        );
    }

    #[test]
    fn font_shorthand_lowers_to_canonical_font_longhands() {
        let families = FontFamilyList::new(["Inter", "system-ui"]).unwrap();
        let font = Font::try_new(
            Some(TextSlant::Italic),
            Some(FontVariant::SmallCaps),
            Some(FontWeight::Number(FontWeightNumber::new(650).unwrap())),
            Some(FontStretch::Condensed),
            Length::Px(18.0),
            Some(Length::Percent(125.0)),
            families.clone(),
        )
        .unwrap();

        let declarations = Declarations::new().try_font(font).unwrap();

        assert_eq!(declarations.get(Property::Font), None);
        assert_eq!(
            declarations.get(Property::FontStyle),
            Some(&Value::TextSlant(TextSlant::Italic))
        );
        assert_eq!(
            declarations.get(Property::FontVariant),
            Some(&Value::FontVariant(FontVariant::SmallCaps))
        );
        assert_eq!(
            declarations.get(Property::FontWeight),
            Some(&Value::FontWeight(FontWeight::Number(
                FontWeightNumber::new(650).unwrap()
            )))
        );
        assert_eq!(
            declarations.get(Property::FontStretch),
            Some(&Value::FontStretch(FontStretch::Condensed))
        );
        assert_eq!(
            declarations.get(Property::FontSize),
            Some(&Value::Length(Length::Px(18.0)))
        );
        assert_eq!(
            declarations.get(Property::LineHeight),
            Some(&Value::Length(Length::Percent(125.0)))
        );
        assert_eq!(
            declarations.get(Property::FontFamily),
            Some(&Value::FontFamilyList(families))
        );
    }

    #[test]
    fn font_shorthand_resets_omitted_components_to_defaults() {
        let families = FontFamilyList::new(["Inter"]).unwrap();
        let font = Font::try_new(
            None,
            None,
            None,
            None,
            Length::Px(14.0),
            None,
            families.clone(),
        )
        .unwrap();

        let declarations = Declarations::new()
            .try_font_style(TextSlant::Italic)
            .unwrap()
            .font_variant(FontVariant::SmallCaps)
            .font_weight(FontWeight::Bold)
            .font_stretch(FontStretch::Expanded)
            .try_line_height(Length::Percent(140.0))
            .unwrap()
            .try_font(font)
            .unwrap();

        assert_eq!(
            declarations.get(Property::FontStyle),
            Some(&Value::TextSlant(TextSlant::default()))
        );
        assert_eq!(
            declarations.get(Property::FontVariant),
            Some(&Value::FontVariant(FontVariant::default()))
        );
        assert_eq!(
            declarations.get(Property::FontWeight),
            Some(&Value::FontWeight(FontWeight::default()))
        );
        assert_eq!(
            declarations.get(Property::FontStretch),
            Some(&Value::FontStretch(FontStretch::default()))
        );
        assert_eq!(
            declarations.get(Property::FontSize),
            Some(&Value::Length(Length::Px(14.0)))
        );
        assert_eq!(
            declarations.get(Property::LineHeight),
            Some(&Property::LineHeight.metadata().default().clone())
        );
        assert_eq!(
            declarations.get(Property::FontFamily),
            Some(&Value::FontFamilyList(families))
        );
    }

    #[test]
    fn font_values_validate_css_facing_domains() {
        assert!(FontWeightNumber::new(1).is_ok());
        assert!(FontWeightNumber::new(1000).is_ok());
        assert!(FontWeightNumber::new(0).is_err());
        assert!(FontWeightNumber::new(1001).is_err());

        assert!(FontFeatureTag::new("kern").is_ok());
        assert!(FontFeatureTag::new("liga").is_ok());
        assert!(FontFeatureTag::new("abc").is_err());
        assert!(FontFeatureTag::new("abcde").is_err());

        let features = FontFeatureSettings::features([FontFeature::new(
            FontFeatureTag::new("kern").unwrap(),
            Some(FontFeatureValue::On),
        )])
        .unwrap();
        assert_eq!(features.len(), 1);
        assert!(FontFeatureSettings::features([]).is_err());
    }

    #[test]
    fn font_shorthand_rejects_invalid_length_domains() {
        let families = FontFamilyList::new(["Inter"]).unwrap();
        let invalid_size =
            Font::try_new(None, None, None, None, Length::Auto, None, families.clone())
                .unwrap_err();
        assert_eq!(invalid_size.code(), ErrorCode::InvalidValue);

        let invalid_line_height = Font::try_new(
            None,
            None,
            None,
            None,
            Length::Px(16.0),
            Some(Length::Auto),
            families,
        )
        .unwrap_err();
        assert_eq!(invalid_line_height.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn font_style_builder_rejects_invalid_slant() {
        let invalid = Declarations::new()
            .try_font_style(TextSlant::Oblique(Some(f32::NAN)))
            .unwrap_err();

        assert_eq!(invalid.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn place_shorthands_lower_to_axis_longhands() {
        let declarations = Declarations::new().place_content(PlaceContentAlignment::new(
            AlignContent::Center,
            AlignContent::SpaceBetween,
        ));
        assert_eq!(declarations.get(Property::PlaceContent), None);
        assert_eq!(
            declarations.get(Property::AlignContent),
            Some(&Value::AlignContent(AlignContent::Center))
        );
        assert_eq!(
            declarations.get(Property::JustifyContent),
            Some(&Value::AlignContent(AlignContent::SpaceBetween))
        );

        let declarations = Declarations::new()
            .place_items(PlaceItemsAlignment::new(
                AlignItems::Start,
                AlignItems::Stretch,
            ))
            .place_self(PlaceItemsAlignment::new(
                AlignItems::End,
                AlignItems::Center,
            ));
        assert_eq!(
            declarations.get(Property::AlignItems),
            Some(&Value::AlignItems(AlignItems::Start))
        );
        assert_eq!(
            declarations.get(Property::JustifyItems),
            Some(&Value::AlignItems(AlignItems::Stretch))
        );
        assert_eq!(
            declarations.get(Property::AlignSelf),
            Some(&Value::AlignItems(AlignItems::End))
        );
        assert_eq!(
            declarations.get(Property::JustifySelf),
            Some(&Value::AlignItems(AlignItems::Center))
        );
    }

    #[test]
    fn calc_px_only_negative_results_are_rejected_for_non_negative_properties() {
        let calc = CalcLength::sum(
            CalcLengthTerm::add(CalcLength::px(0.0)),
            [CalcLengthTerm::sub(CalcLength::px(1.0))],
        );

        let error =
            Declaration::try_new(Property::Width, Value::Length(Length::Calc(calc))).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn calc_percent_only_negative_results_are_rejected_for_non_negative_properties() {
        let calc = CalcLength::sum(
            CalcLengthTerm::add(CalcLength::percent(0.0)),
            [CalcLengthTerm::sub(CalcLength::percent(1.0))],
        );

        let error =
            Declaration::try_new(Property::Width, Value::Length(Length::Calc(calc))).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn mixed_all_nonpositive_calc_lengths_are_rejected_for_non_negative_properties() {
        let calc = CalcLength::sum(
            CalcLengthTerm::sub(CalcLength::px(1.0)),
            [CalcLengthTerm::sub(CalcLength::percent(1.0))],
        );

        let error =
            Declaration::try_new(Property::Width, Value::Length(Length::Calc(calc))).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn indefinite_mixed_calc_lengths_remain_valid_for_non_negative_properties() {
        let calc = CalcLength::sum(
            CalcLengthTerm::sub(CalcLength::px(1.0)),
            [CalcLengthTerm::add(CalcLength::percent(10.0))],
        );

        Declaration::try_new(Property::Width, Value::Length(Length::Calc(calc))).unwrap();
    }

    #[test]
    fn grid_flow_tolerance_calc_reaches_property_domain_validation() {
        let calc = CalcLength::sum(
            CalcLengthTerm::add(CalcLength::px(8.0)),
            [CalcLengthTerm::add(CalcLength::percent(2.0))],
        );

        let error = Declaration::try_new(
            Property::GridFlowTolerance,
            Value::GridFlowTolerance(GridFlowTolerance::Length(Length::Calc(calc))),
        )
        .unwrap_err();
        assert!(error.to_string().contains("grid flow tolerance length"));
    }

    #[test]
    fn edge_shorthands_lower_to_side_longhands() {
        let edges = Edges::new(
            Length::Px(1.0),
            Length::Px(2.0),
            Length::Px(3.0),
            Length::Px(4.0),
        );

        let declarations = Declarations::new().try_margin(edges.clone()).unwrap();
        assert_eq!(declarations.get(Property::Margin), None);
        assert_eq!(
            declarations.get(Property::MarginTop),
            Some(&Value::Length(edges.top.clone()))
        );
        assert_eq!(
            declarations.get(Property::MarginRight),
            Some(&Value::Length(edges.right.clone()))
        );
        assert_eq!(
            declarations.get(Property::MarginBottom),
            Some(&Value::Length(edges.bottom.clone()))
        );
        assert_eq!(
            declarations.get(Property::MarginLeft),
            Some(&Value::Length(edges.left.clone()))
        );

        let declarations = Declarations::new().try_padding(edges.clone()).unwrap();
        assert_eq!(declarations.get(Property::Padding), None);
        assert_eq!(
            declarations.get(Property::PaddingTop),
            Some(&Value::Length(edges.top.clone()))
        );
        assert_eq!(
            declarations.get(Property::PaddingRight),
            Some(&Value::Length(edges.right.clone()))
        );
        assert_eq!(
            declarations.get(Property::PaddingBottom),
            Some(&Value::Length(edges.bottom.clone()))
        );
        assert_eq!(
            declarations.get(Property::PaddingLeft),
            Some(&Value::Length(edges.left.clone()))
        );

        let declarations = Declarations::new().try_border_width(edges.clone()).unwrap();
        assert_eq!(declarations.get(Property::BorderWidth), None);
        assert_eq!(
            declarations.get(Property::BorderTopWidth),
            Some(&Value::Length(edges.top.clone()))
        );
        assert_eq!(
            declarations.get(Property::BorderRightWidth),
            Some(&Value::Length(edges.right.clone()))
        );
        assert_eq!(
            declarations.get(Property::BorderBottomWidth),
            Some(&Value::Length(edges.bottom.clone()))
        );
        assert_eq!(
            declarations.get(Property::BorderLeftWidth),
            Some(&Value::Length(edges.left.clone()))
        );

        let declarations = Declarations::new().try_inset(edges.clone()).unwrap();
        assert_eq!(declarations.get(Property::Inset), None);
        assert_eq!(
            declarations.get(Property::Top),
            Some(&Value::Length(edges.top))
        );
        assert_eq!(
            declarations.get(Property::Right),
            Some(&Value::Length(edges.right))
        );
        assert_eq!(
            declarations.get(Property::Bottom),
            Some(&Value::Length(edges.bottom))
        );
        assert_eq!(
            declarations.get(Property::Left),
            Some(&Value::Length(edges.left))
        );
    }

    #[test]
    fn edge_shorthands_validate_canonical_longhand_domains() {
        assert!(
            Declarations::new()
                .try_padding(Edges::all(Length::Auto))
                .is_err()
        );
        assert!(
            Declarations::new()
                .try_border_width(Edges::all(Length::Normal))
                .is_err()
        );
        assert!(
            Declarations::new()
                .try_margin(Edges::all(Length::Normal))
                .is_err()
        );
        assert!(
            Declarations::new()
                .try_set(Property::PaddingTop, Value::Color(Color::BLACK))
                .is_err()
        );
    }
}
