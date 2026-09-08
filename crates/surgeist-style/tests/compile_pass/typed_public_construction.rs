use surgeist_style::{
    AlignContent, AlignItems, Alpha, AnimationDirection, AnimationDirectionList, AnimationFillMode,
    AnimationFillModeList, AnimationItem, AnimationIterationCount, AnimationIterationCountList,
    AnimationIterationNumber, AnimationList, AnimationName, AnimationNameList, AnimationPlayState,
    AnimationPlayStateList, AspectRatio, AttributeCaseSensitivity, AttributeMatcher,
    AttributeSelector, AuthoredDeclaration, AuthoredDeclarations, AuthoredProperty, AuthoredTokens,
    AuthoredValue, BackgroundAttachment,
    BackgroundAttachmentList, BackgroundBox, BackgroundRepeat, BackgroundRepeatList,
    BackgroundRepeatStyle, BackgroundSize, BackgroundSizeComponent, BackgroundSizeList,
    BasicShape, Border, BorderLineStyle, BorderRadii, BorderStyles, BoxDecorationBreak,
    BuiltInCounterStyle, CascadeChange, Change, ClipPath, Color, ColorComponent,
    ColorSchemePreference, Combinator, Condition, ConditionFacts, ConditionFactChange,
    ContainerCondition, ContainerConditionList, ContainerFacts, ContainerFeatureQuery,
    ContainerName, ContainerStyleQuery, Content, ContentItem, ContentItemList, ContentString,
    ContentVisibility, Context, ContrastPreference, CornerRadius, CounterChange,
    CounterChangeList, CounterChanges, CounterFunction, CounterName, CounterStyle,
    CounterStyleName, CountersFunction, CssPx, CssWideKeyword, DisplayMode,
    CustomPropertyName, CustomPropertyTypedValue, CustomPropertyValue, Declarations,
    DimensionLength, DurationSeconds, EasingArguments, EasingFunction, EasingList, Filter,
    FilterFunction, FilterFunctionList, Flex, FlexFactor, Font, FontFamilyList, FontFeature,
    FontFeatureSettings, FontFeatureTag, FontFeatureValue, FontStretch, FontVariant, FontWeight,
    ForcedColorsMode, GridTrackList, HoverCapability, HorizontalPositionKeyword, ImageLayer,
    ImageLayerList, InvalidAtComputedValueReason, KeyframeBlock, KeyframeOffset,
    KeyframeSelectorList, KeyframesIdent, KeyframesName, KeyframesRule, KeyframesString,
    LayerBlock, LayerOrder, LayerRegistry, LayerStatement, LayoutPosition, Length,
    LetterSpacing, LetterSpacingLength, ListStyle, ListStyleImage, ListStylePosition,
    ListStyleType, MaskLayer, MaskLayerList, MediaCondition, MediaConditionList,
    MediaEnvironment, MediaFeatureQuery, MediaQuery, MediaQueryList, MediaQueryModifier,
    MediaType, Node, NonNegativeInteger, NthPattern, NthSelector, Opacity, Order, Orientation,
    Outline, OutlineStyle, OutlineWidth, OutlineWidthLength, OverflowWrap, PointerCapability,
    PlaceContentAlignment, PlaceItemsAlignment, Position, PositionComponent, PositionList,
    Property, PseudoClassSelector, PseudoElement, QueryComparison, QueryLength, QueryLengthBasis,
    QueryLengthUnit, RangeFeature, RangeState, Ratio, ReducedMotionPreference,
    ReducedTransparencyPreference, RelativeSelector, RelativeSelectorList, Resolution,
    ResolutionUnit, ResolvedWithDiagnostics, Rotate, RulePrecedence, RuleScope, RuleTarget,
    RuntimePseudoClass, Scale, ScaleValues, ScopeSelectorList, ScrollbarWidth, Selector, SelectorList,
    SelectorListPseudoClass, SelectorSpecificity, SelectorFactChange, Sheet, SourceOrder,
    StateFlag, StructuralSelector, StyleAttributeName, StyleAttributeValue, StyleBucket,
    StyleBucketPolicy, StyleColor, StyleDiagnostic, StyleDiagnosticKind, StyleDiagnosticSubject,
    StyleLayerName, StyleLayerNameList, StyleRole, StyleSourceId, StyleState, StyleTag, StyleUrl,
    SymbolicFunctionValue, TextAlignLast, TextDecoration, TextDecorationLine, TypedMediaQuery,
    TextDecorationLineComponent, TextDecorationStyle, TextDecorationThickness,
    TextDecorationThicknessLength, TextIndent, TextOverflow, TextSlant, TextTransform, TextWrap,
    TimeList, TransitionItem, TransitionList, TransitionPropertyList, TransitionPropertyName,
    TransitionPropertyTarget, Translate, TranslateValues, Traversal, Tree, TypedDeclaration,
    UserSelect, Value, VariableDependentValue, VariableExpression, VariableFallback,
    VariableReference, VerticalAlign, VerticalAlignLength, VerticalPositionKeyword, WhiteSpace,
    WordBreak, ZIndex,
};

fn main() -> surgeist_style::Result<()> {
    let width = TypedDeclaration::width(DimensionLength::px(CssPx::new(120.0)?)?);
    let opacity = TypedDeclaration::opacity(Opacity::new(0.75)?);
    let color = TypedDeclaration::try_text_color(StyleColor::rgba(Color::try_rgba(
        0.0, 0.0, 0.0, 1.0,
    )?))?;

    let declarations = Declarations::from_typed([width, opacity, color])?;
    assert_eq!(declarations.len(), 3);

    let declarations = Declarations::new()
        .width(DimensionLength::px(CssPx::new(240.0)?)?)
        .opacity(Opacity::new(0.5)?)
        .transition_duration(TimeList::try_new([DurationSeconds::new(0.2)?])?)?
        .try_grid_template_rows(GridTrackList::new(Vec::new()))?;
    assert_eq!(declarations.len(), 4);

    let declarations = Declarations::new()
        .try_set(
            Property::FontFamily,
            Value::FontFamilyList(FontFamilyList::new(["Inter", "system-ui"])?),
        )?
        .try_set(
            Property::AnimationName,
            Value::AnimationNameList(AnimationNameList::try_new([AnimationName::Keyframes(
                KeyframesName::Ident(KeyframesIdent::try_new("fade-in")?),
            )])?),
        )?;
    assert_eq!(declarations.len(), 2);

    let font_features = FontFeatureSettings::features([FontFeature::new(
        FontFeatureTag::new("kern")?,
        Some(FontFeatureValue::On),
    )])?;
    let font = Font::try_new(
        Some(TextSlant::Italic),
        Some(FontVariant::SmallCaps),
        Some(FontWeight::Bold),
        Some(FontStretch::Expanded),
        Length::Px(16.0),
        Some(Length::Percent(120.0)),
        FontFamilyList::new(["Inter", "serif"])?,
    )?;
    let declarations = Declarations::new()
        .try_font_family(FontFamilyList::new(["Inter", "serif"])?)?
        .try_font_size(Length::Px(16.0))?
        .try_line_height(Length::Percent(120.0))?
        .font_weight(FontWeight::Bold)
        .try_font_style(TextSlant::Italic)?
        .font_stretch(FontStretch::Expanded)
        .font_variant(FontVariant::SmallCaps)
        .try_font_feature_settings(font_features)?
        .try_font(font)?;
    assert_eq!(declarations.len(), 8);

    let text_indent = TextIndent::new(Length::Percent(10.0), true, false)?;
    assert_eq!(text_indent.length(), &Length::Percent(10.0));
    assert!(text_indent.hanging());
    assert!(!text_indent.each_line());
    let vertical_align_length = VerticalAlignLength::new(Length::Px(-1.0))?;
    assert_eq!(vertical_align_length.length(), &Length::Px(-1.0));
    let letter_spacing_length = LetterSpacingLength::new(Length::Px(0.5))?;
    assert_eq!(letter_spacing_length.length(), &Length::Px(0.5));
    let declarations = Declarations::new()
        .text_align_last(TextAlignLast::End)
        .try_text_indent(text_indent)?
        .try_vertical_align(VerticalAlign::Length(vertical_align_length))?
        .try_letter_spacing(LetterSpacing::Length(letter_spacing_length))?
        .text_transform(TextTransform::Lowercase);
    assert_eq!(declarations.len(), 5);

    let declarations = Declarations::new()
        .text_wrap(TextWrap::Balance)
        .white_space(WhiteSpace::BreakSpaces)
        .word_break(WordBreak::BreakWord)
        .overflow_wrap(OverflowWrap::Anywhere)
        .text_overflow(TextOverflow::Ellipsis);
    assert_eq!(declarations.len(), 5);

    let time_list = TimeList::try_new([
        DurationSeconds::new(0.2)?,
        DurationSeconds::new(0.4)?,
    ])?;
    let easing_list = EasingList::try_new([
        EasingFunction::EaseInOut,
        EasingFunction::CubicBezier(EasingArguments::try_new("0.4, 0, 0.2, 1")?),
    ])?;
    let transition_properties = TransitionPropertyList::try_new([
        TransitionPropertyTarget::All,
        TransitionPropertyTarget::Custom(TransitionPropertyName::try_new("opacity")?),
    ])?;
    let transition = TransitionList::try_new([TransitionItem::try_new(
        Some(TransitionPropertyTarget::Property(Property::Opacity)),
        Some(DurationSeconds::new(0.2)?),
        Some(DurationSeconds::new(0.05)?),
        Some(EasingFunction::EaseOut),
    )?])?;
    let declarations = Declarations::new()
        .transition_property(transition_properties.clone())?
        .transition_duration(time_list.clone())?
        .transition_delay(TimeList::try_new([DurationSeconds::new(0.05)?])?)?
        .transition_timing_function(easing_list.clone())?
        .transition(transition.clone())?;
    assert_eq!(declarations.len(), 4);
    let _ = (time_list, easing_list, transition_properties, transition);

    let animation_names = AnimationNameList::try_new([
        AnimationName::None,
        AnimationName::Keyframes(KeyframesName::Ident(KeyframesIdent::try_new("fade-in")?)),
        AnimationName::Keyframes(KeyframesName::String(KeyframesString::try_new("fade in")?)),
    ])?;
    let animation_iterations = AnimationIterationCountList::try_new([
        AnimationIterationCount::Number(AnimationIterationNumber::try_new(2.0)?),
        AnimationIterationCount::Infinite,
    ])?;
    let animation_directions = AnimationDirectionList::try_new([
        AnimationDirection::Normal,
        AnimationDirection::AlternateReverse,
    ])?;
    let animation_fill_modes =
        AnimationFillModeList::try_new([AnimationFillMode::None, AnimationFillMode::Forwards])?;
    let animation_play_states =
        AnimationPlayStateList::try_new([AnimationPlayState::Running, AnimationPlayState::Paused])?;
    let declarations = Declarations::new()
        .animation_name(animation_names.clone())?
        .animation_duration(TimeList::try_new([DurationSeconds::new(0.2)?])?)?
        .animation_delay(TimeList::try_new([DurationSeconds::new(0.05)?])?)?
        .animation_timing_function(EasingList::try_new([EasingFunction::EaseIn])?)?
        .animation_iteration_count(animation_iterations.clone())?
        .animation_direction(animation_directions.clone())?
        .animation_fill_mode(animation_fill_modes.clone())?
        .animation_play_state(animation_play_states.clone())?
        .animation(AnimationList::try_new([AnimationItem::try_new(
            Some(AnimationName::Keyframes(KeyframesName::Ident(
                KeyframesIdent::try_new("fade-in")?,
            ))),
            Some(DurationSeconds::new(0.3)?),
            Some(DurationSeconds::new(0.05)?),
            Some(EasingFunction::EaseInOut),
            Some(AnimationIterationCount::Infinite),
            Some(AnimationDirection::Alternate),
            Some(AnimationFillMode::Both),
            Some(AnimationPlayState::Paused),
        )?])?)?;
    assert_eq!(declarations.len(), 8);
    let _ = (
        animation_names,
        animation_iterations,
        animation_directions,
        animation_fill_modes,
        animation_play_states,
    );

    let mut keyframe_declarations = AuthoredDeclarations::new();
    keyframe_declarations.try_push(AuthoredDeclaration::try_new(
        AuthoredProperty::Property(Property::Opacity),
        AuthoredValue::Value(Value::Number(1.0)),
    )?)?;
    let keyframes = KeyframesRule::try_new(
        KeyframesName::Ident(KeyframesIdent::try_new("fade-in")?),
        [KeyframeBlock::try_new(
            KeyframeSelectorList::try_new([KeyframeOffset::from(), KeyframeOffset::to()])?,
            keyframe_declarations,
        )?],
    )?;
    let sheet = Sheet::new().keyframes_rule(keyframes);
    assert_eq!(sheet.keyframes_rule_count(), 1);

    let base_layer = StyleLayerName::try_new(["base"])?;
    let theme_layer = StyleLayerName::try_new(["theme", "buttons"])?;
    let layers = StyleLayerNameList::try_new([base_layer.clone(), theme_layer.clone()])?;
    let statement = LayerStatement::new(layers.clone());
    let mut registry = LayerRegistry::new();
    let registered = registry.declare(&statement);
    assert_eq!(registered.len(), 2);
    assert_eq!(registry.order(&base_layer), Some(LayerOrder::new(1)));
    assert!(matches!(
        LayerBlock::Named(theme_layer.clone()),
        LayerBlock::Named(_)
    ));
    assert_eq!(LayerBlock::Anonymous, LayerBlock::Anonymous);
    let mut sheet = Sheet::new();
    sheet.declare_layers(layers);
    sheet.push_layer_rule(base_layer, Selector::tag("button")?, Declarations::new())?;
    assert!(sheet.layer_order(&theme_layer).is_some());

    let scope = RuleScope::try_new(
        Some(ScopeSelectorList::try_new([Selector::class("card")?])?),
        Some(ScopeSelectorList::try_new([Selector::class("limit")?])?),
    )?;
    let mut sheet = Sheet::new();
    sheet.push_scoped_rule(scope, Selector::tag("button")?, Declarations::new())?;

    let text_decoration_line = TextDecorationLine::try_new([
        TextDecorationLineComponent::Underline,
        TextDecorationLineComponent::Overline,
    ])?;
    assert!(!text_decoration_line.is_none());
    assert_eq!(text_decoration_line.components().len(), 2);
    assert!(TextDecorationLine::none().components().is_empty());
    let text_decoration_thickness_length =
        TextDecorationThicknessLength::new(Length::Px(2.0))?;
    assert_eq!(
        text_decoration_thickness_length.length(),
        &Length::Px(2.0)
    );
    let text_decoration_thickness =
        TextDecorationThickness::Length(text_decoration_thickness_length);
    let text_decoration = TextDecoration::try_new(
        Some(text_decoration_line.clone()),
        None,
        Some(TextDecorationStyle::Double),
        Some(text_decoration_thickness.clone()),
    )?;
    assert!(text_decoration.line().is_some());
    assert_eq!(text_decoration.style(), Some(TextDecorationStyle::Double));
    assert!(text_decoration.thickness().is_some());
    let declarations = Declarations::new()
        .try_text_decoration(text_decoration)?
        .try_text_decoration_line(text_decoration_line)?
        .text_decoration_style(TextDecorationStyle::Wavy)
        .try_text_decoration_thickness(text_decoration_thickness)?;
    assert_eq!(declarations.len(), 4);

    let content_string = ContentString::try_new("Section ")?;
    assert_eq!(content_string.as_str(), "Section ");
    let counter_name = CounterName::try_new("section")?;
    let counter_style_name = CounterStyleName::try_new("legal")?;
    assert_eq!(counter_name.as_str(), "section");
    assert_eq!(counter_style_name.as_str(), "legal");
    let counter_style = CounterStyle::Named(counter_style_name);
    let counter = CounterFunction::new(counter_name.clone(), Some(counter_style.clone()));
    assert_eq!(counter.name().as_str(), "section");
    assert_eq!(counter.style(), Some(&counter_style));
    let counters = CountersFunction::new(
        counter_name,
        ContentString::try_new(".")?,
        Some(CounterStyle::BuiltIn(BuiltInCounterStyle::LowerRoman)),
    );
    assert_eq!(counters.separator().as_str(), ".");
    assert_eq!(
        counters.style(),
        Some(&CounterStyle::BuiltIn(BuiltInCounterStyle::LowerRoman))
    );
    let content_items = ContentItemList::try_new([
        ContentItem::String(content_string),
        ContentItem::Url(StyleUrl::new("section.svg")?),
        ContentItem::Counter(counter),
        ContentItem::Counters(counters),
        ContentItem::Attr(StyleAttributeName::new("data-title")?),
        ContentItem::OpenQuote,
        ContentItem::CloseQuote,
        ContentItem::NoOpenQuote,
        ContentItem::NoCloseQuote,
    ])?;
    assert_eq!(content_items.items().len(), 9);
    let content = Content::Items(content_items);
    assert!(matches!(content, Content::Items(_)));
    assert!(matches!(Content::Normal, Content::Normal));
    assert!(matches!(Content::None, Content::None));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::Disc),
        CounterStyle::BuiltIn(BuiltInCounterStyle::Disc)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::Circle),
        CounterStyle::BuiltIn(BuiltInCounterStyle::Circle)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::Square),
        CounterStyle::BuiltIn(BuiltInCounterStyle::Square)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::Decimal),
        CounterStyle::BuiltIn(BuiltInCounterStyle::Decimal)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::DecimalLeadingZero),
        CounterStyle::BuiltIn(BuiltInCounterStyle::DecimalLeadingZero)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::LowerAlpha),
        CounterStyle::BuiltIn(BuiltInCounterStyle::LowerAlpha)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::UpperAlpha),
        CounterStyle::BuiltIn(BuiltInCounterStyle::UpperAlpha)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::LowerLatin),
        CounterStyle::BuiltIn(BuiltInCounterStyle::LowerLatin)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::UpperLatin),
        CounterStyle::BuiltIn(BuiltInCounterStyle::UpperLatin)
    ));
    assert!(matches!(
        CounterStyle::BuiltIn(BuiltInCounterStyle::UpperRoman),
        CounterStyle::BuiltIn(BuiltInCounterStyle::UpperRoman)
    ));
    let list_style = ListStyle::try_new(
        Some(ListStyleType::CounterStyle(CounterStyle::BuiltIn(
            BuiltInCounterStyle::Disc,
        ))),
        Some(ListStylePosition::Outside),
        Some(ListStyleImage::Url(StyleUrl::new("marker.svg")?)),
    )?;
    assert!(matches!(
        list_style.style_type(),
        Some(ListStyleType::CounterStyle(_))
    ));
    assert_eq!(list_style.position(), Some(ListStylePosition::Outside));
    assert!(matches!(list_style.image(), Some(ListStyleImage::Url(_))));
    assert!(matches!(ListStyleType::None, ListStyleType::None));
    assert!(matches!(
        ListStyleType::String(ContentString::try_new("*")?),
        ListStyleType::String(_)
    ));
    assert!(matches!(ListStyleImage::None, ListStyleImage::None));
    assert!(matches!(ListStylePosition::Inside, ListStylePosition::Inside));
    let counter_change = CounterChange::new(CounterName::try_new("section")?, 1);
    assert_eq!(counter_change.value(), 1);
    let counter_changes = CounterChangeList::try_new([counter_change])?;
    assert_eq!(counter_changes.changes().len(), 1);
    assert!(matches!(
        CounterChanges::Changes(counter_changes),
        CounterChanges::Changes(_)
    ));
    assert!(matches!(CounterChanges::default(), CounterChanges::None));
    let generated_content = Content::Items(ContentItemList::try_new([
        ContentItem::String(ContentString::try_new("Item ")?),
        ContentItem::Counter(CounterFunction::new(
            CounterName::try_new("item")?,
            Some(CounterStyle::BuiltIn(BuiltInCounterStyle::Decimal)),
        )),
    ])?);
    let declarations = Declarations::new()
        .content(generated_content)?
        .list_style(ListStyle::try_new(
            Some(ListStyleType::CounterStyle(CounterStyle::BuiltIn(
                BuiltInCounterStyle::Disc,
            ))),
            Some(ListStylePosition::Outside),
            Some(ListStyleImage::None),
        )?)?
        .counter_reset(CounterChanges::None)?
        .counter_increment(CounterChanges::Changes(CounterChangeList::try_new([
            CounterChange::new(CounterName::try_new("item")?, 1),
        ])?))?
        .counter_set(CounterChanges::None)?;
    let _ = declarations;

    let filter_functions =
        FilterFunctionList::try_new([FilterFunction::Blur(SymbolicFunctionValue::new("4px")?)])?;
    assert_eq!(filter_functions.functions().len(), 1);
    let clip_path = ClipPath::BasicShape(BasicShape::Circle(SymbolicFunctionValue::new("50%")?));
    let translate_values = TranslateValues::try_new([Length::Px(10.0), Length::Percent(5.0)])?;
    assert_eq!(translate_values.values().len(), 2);
    let scale_values = ScaleValues::try_new([1.0, 2.0])?;
    assert_eq!(scale_values.values().len(), 2);
    let declarations = Declarations::new()
        .box_decoration_break(BoxDecorationBreak::Clone)
        .filter(Filter::Functions(filter_functions))
        .backdrop_filter(Filter::None)
        .clip_path(clip_path)
        .translate(Translate::try_values([Length::Px(10.0)])?)
        .translate(Translate::Values(translate_values))
        .rotate(Rotate::Value(SymbolicFunctionValue::new("45deg")?))
        .scale(Scale::try_values([1.0, 2.0])?)
        .scale(Scale::Values(scale_values));
    assert_eq!(declarations.len(), 7);

    let declarations = Declarations::new().user_select(UserSelect::Text);
    assert_eq!(
        declarations.get(Property::UserSelect),
        Some(&Value::UserSelect(UserSelect::Text))
    );

    let alpha = Alpha::new(0.5)?;
    let color = StyleColor::Hsl {
        hue: ColorComponent::new(Some(210.0))?,
        saturation: ColorComponent::new(Some(60.0))?,
        lightness: ColorComponent::new(Some(40.0))?,
        alpha: Some(alpha),
    };
    let declarations = Declarations::new()
        .try_text_color(color.clone())?
        .try_text_decoration_color(StyleColor::current_color())?;
    assert_eq!(declarations.len(), 2);

    let hero_url = StyleUrl::new("hero.png")?;
    assert_eq!(hero_url.as_str(), "hero.png");
    let image_layers = ImageLayerList::try_new([ImageLayer::url(hero_url)])?;
    assert_eq!(image_layers.layers().len(), 1);
    assert!(ImageLayerList::try_new([]).is_err());
    let position = Position::try_new([
        PositionComponent::Horizontal(HorizontalPositionKeyword::Left),
        PositionComponent::Length(Length::Percent(25.0)),
    ])?;
    assert_eq!(position.components().len(), 2);
    assert_eq!(
        Position::origin().components(),
        &[
            PositionComponent::Length(Length::Percent(0.0)),
            PositionComponent::Length(Length::Percent(0.0)),
        ]
    );
    assert!(Position::try_new([]).is_err());
    assert!(
        Position::try_new([
            PositionComponent::Horizontal(HorizontalPositionKeyword::Left),
            PositionComponent::Horizontal(HorizontalPositionKeyword::Right),
        ])
        .is_err()
    );
    assert!(
        Position::try_new([
            PositionComponent::Vertical(VerticalPositionKeyword::Top),
            PositionComponent::Vertical(VerticalPositionKeyword::Bottom),
        ])
        .is_err()
    );
    let position_layers = PositionList::try_new([position.clone()])?;
    let size = BackgroundSize::Explicit {
        width: BackgroundSizeComponent::Length(Length::Percent(100.0)),
        height: Some(BackgroundSizeComponent::Auto),
    };
    let size_layers = BackgroundSizeList::try_new([size.clone()])?;
    let repeat = BackgroundRepeat::Axes {
        x: BackgroundRepeatStyle::NoRepeat,
        y: BackgroundRepeatStyle::NoRepeat,
    };
    let repeat_layers = BackgroundRepeatList::try_new([repeat])?;
    let attachments = BackgroundAttachmentList::try_new([BackgroundAttachment::Fixed])?;
    let mask_layer = MaskLayer::try_new(
        Some(ImageLayer::None),
        Some(Position::try_new([PositionComponent::Vertical(
            VerticalPositionKeyword::Top,
        )])?),
        Some(BackgroundSize::Contain),
        Some(BackgroundRepeat::RepeatX),
    )?;
    assert!(MaskLayer::try_new(None, None, None, None).is_err());
    let mask_layers = MaskLayerList::try_new([mask_layer])?;
    let declarations = Declarations::new()
        .background_image(image_layers.clone())
        .background_position(position_layers.clone())
        .background_size(size_layers.clone())
        .background_repeat(repeat_layers.clone())
        .background_origin(BackgroundBox::PaddingBox)
        .background_clip(BackgroundBox::ContentBox)
        .background_attachment(attachments)
        .mask(mask_layers)?
        .mask_image(image_layers)
        .mask_position(position_layers)
        .mask_size(size_layers)
        .mask_repeat(repeat_layers);
    assert_eq!(declarations.get(Property::Mask), None);
    assert!(matches!(
        declarations.get(Property::BackgroundImage),
        Some(Value::ImageLayerList(_))
    ));
    assert!(matches!(
        declarations.get(Property::MaskPosition),
        Some(Value::PositionList(_))
    ));

    let declarations = Declarations::new()
        .try_inset_top(Length::Auto)?
        .try_margin_left(Length::Px(-4.0))?
        .try_padding_right(Length::Px(8.0))?
        .try_border_bottom_width(Length::Px(2.0))?;
    assert_eq!(declarations.len(), 4);

    let border = Border::try_new(
        Some(Length::Px(2.0)),
        Some(BorderLineStyle::Solid),
        Some(StyleColor::current_color()),
    )?;
    let border_styles = BorderStyles::new(
        BorderLineStyle::Solid,
        BorderLineStyle::Dashed,
        BorderLineStyle::Dotted,
        BorderLineStyle::Double,
    );
    let corner = CornerRadius::new(Length::Px(4.0), Length::Percent(50.0))?;
    let radii = BorderRadii::all(corner.clone());
    let outline = Outline::try_new(
        Some(OutlineWidth::Length(OutlineWidthLength::new(
            Length::Px(3.0),
        )?)),
        Some(OutlineStyle::Border(BorderLineStyle::Dotted)),
        Some(StyleColor::current_color()),
    )?;
    let declarations = Declarations::new()
        .try_border(border)?
        .border_style(border_styles)
        .try_border_color(StyleColor::current_color())?
        .try_border_top_left_radius(corner)?
        .try_border_radius(radii)?
        .try_outline(outline)?;
    assert_eq!(declarations.len(), 19);

    let declarations = Declarations::new()
        .position(LayoutPosition::Fixed)
        .z_index(ZIndex::integer(-1))
        .scrollbar_width(ScrollbarWidth::Thin)
        .content_visibility(ContentVisibility::Auto)
        .order(Order::new(2))
        .try_flex_grow(FlexFactor::new(2.0)?)?
        .try_flex_shrink(FlexFactor::new(0.0)?)?
        .try_aspect_ratio(AspectRatio::ratio(16.0 / 9.0)?)?;
    assert_eq!(declarations.len(), 8);

    let declarations = Declarations::new()
        .try_flex(Flex::components(
            FlexFactor::new(2.0)?,
            None,
            Some(Length::Px(10.0)),
        ))?
        .place_content(PlaceContentAlignment::all(AlignContent::Center))
        .place_items(PlaceItemsAlignment::new(
            AlignItems::Start,
            AlignItems::Stretch,
        ))
        .place_self(PlaceItemsAlignment::all(AlignItems::Center))
        .align_tracks(AlignContent::SpaceBetween)
        .justify_tracks(AlignContent::SpaceAround);
    assert_eq!(declarations.len(), 11);

    let precedence = RulePrecedence::new(LayerOrder::new(2), SourceOrder::new(8));
    assert_eq!(precedence.layer_order(), LayerOrder::new(2));

    let specificity = SelectorSpecificity::new(0, 1, 0);
    assert!(specificity > SelectorSpecificity::new(0, 0, 1));
    let selector_list = SelectorList::try_new([
        Selector::tag("button")?,
        Selector::class("primary")?,
    ])?;
    assert_eq!(selector_list.selectors().len(), 2);
    let attribute_selector = AttributeSelector::equals_with_case(
        "data-id",
        "submit",
        AttributeCaseSensitivity::AsciiCaseInsensitive,
    )?;
    let matcher_selector = AttributeSelector::matcher_with_case(
        "data-tags",
        AttributeMatcher::Includes(StyleAttributeValue::new("primary")?),
        AttributeCaseSensitivity::DocumentDefault,
    )?;
    assert!(matches!(attribute_selector, AttributeSelector::Matcher { .. }));
    assert!(matches!(matcher_selector, AttributeSelector::Matcher { .. }));
    let pseudo_selector = Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Hover));
    assert_eq!(pseudo_selector.specificity(), SelectorSpecificity::new(0, 1, 0));
    let nth_selector = NthSelector::new(NthPattern::odd(), Some(selector_list.clone()));
    assert_eq!(nth_selector.pattern(), NthPattern::odd());
    assert!(nth_selector.filter().is_some());
    let structural_selector = Selector::pseudo(PseudoClassSelector::structural(
        StructuralSelector::NthChild(nth_selector),
    ));
    assert_eq!(
        structural_selector.specificity(),
        SelectorSpecificity::new(0, 2, 0)
    );
    let has_child = Selector::pseudo(PseudoClassSelector::has(RelativeSelectorList::try_new(
        [RelativeSelector::new(
            Combinator::Child,
            Selector::class("child")?,
        )],
    )?));
    assert_eq!(has_child.specificity(), SelectorSpecificity::new(0, 1, 0));
    let not_disabled = Selector::pseudo(PseudoClassSelector::selector_list(
        SelectorListPseudoClass::Not(SelectorList::try_new([Selector::class("disabled")?])?),
    ));
    assert_eq!(
        not_disabled.specificity(),
        SelectorSpecificity::new(0, 1, 0)
    );
    let state = StyleState::default().with_range_state(Some(RangeState::InRange));
    assert!(state.has_flag(StateFlag::InRange));

    let before_bucket = StyleBucket::Before;
    assert_eq!(before_bucket.policy(), StyleBucketPolicy::GeneratedContentBox);
    let nested_marker_bucket =
        StyleBucket::from_pseudo_elements([PseudoElement::Before, PseudoElement::Marker])?;
    assert_eq!(nested_marker_bucket, StyleBucket::BeforeMarker);
    assert_eq!(
        StyleBucket::BeforeMarker.policy(),
        StyleBucketPolicy::GeneratedContentMarker
    );
    let target = RuleTarget::new(Selector::class("badge")?, StyleBucket::Before);
    assert_eq!(target.bucket(), StyleBucket::Before);
    let mut targeted_sheet = Sheet::new().targeted_rule(target, Declarations::new());
    targeted_sheet.push_targeted_rule(
        RuleTarget::new(Selector::tag("button")?, StyleBucket::After),
        Declarations::new(),
    );
    assert_eq!(targeted_sheet.rule_count(), 2);
    let tree = PublicTree;
    let _before_context = Context::new(&tree, 0).style_bucket(StyleBucket::Before);

    let selector_change = Change::from_selector_fact_change(SelectorFactChange::Class);
    assert!(selector_change.rematch);
    assert!(selector_change.scope.whole_tree);
    let condition_change = Change::from_condition_fact_change(ConditionFactChange::Media);
    assert!(condition_change.rematch);
    let cascade_change = Change::from_cascade_change(CascadeChange::LayerOrder);
    assert!(cascade_change.scope.whole_tree);
    let bucket_change = Change::from_style_bucket_change(StyleBucket::Before);
    assert!(bucket_change.rematch);

    let rem = QueryLength::try_new(1.0, QueryLengthUnit::Rem)?;
    let media_basis = QueryLengthBasis::new()
        .root_font_size(QueryLength::try_new(16.0, QueryLengthUnit::Px)?)
        .viewport_width(QueryLength::try_new(1024.0, QueryLengthUnit::Px)?)
        .viewport_height(QueryLength::try_new(768.0, QueryLengthUnit::Px)?);
    assert_eq!(rem.unit(), QueryLengthUnit::Rem);
    assert_eq!(rem.to_css_px(&media_basis), Some(16.0));
    assert_eq!(media_basis.viewport_width_basis().unwrap().value(), 1024.0);

    let resolution = Resolution::try_new(192.0, ResolutionUnit::Dpi)?;
    assert_eq!(resolution.to_dppx(), 2.0);
    let ratio = Ratio::try_new(16.0, 9.0)?;
    assert!(ratio.value() > 1.0);
    let color_bits = NonNegativeInteger::new(8);
    assert_eq!(color_bits.value(), 8);

    let media_conditions = MediaConditionList::try_new([
        MediaCondition::Feature(MediaFeatureQuery::Hover(HoverCapability::Hover)),
        MediaCondition::Not(Box::new(MediaCondition::Feature(
            MediaFeatureQuery::ForcedColors(ForcedColorsMode::Active),
        ))),
    ])?;
    assert_eq!(media_conditions.conditions().len(), 2);

    let typed_query = TypedMediaQuery::new(
        Some(MediaQueryModifier::Only),
        MediaType::Screen,
        Some(MediaCondition::And(media_conditions)),
    );
    assert_eq!(typed_query.media_type(), MediaType::Screen);
    assert_eq!(typed_query.modifier(), Some(MediaQueryModifier::Only));
    assert!(typed_query.condition().is_some());

    let media_query_list = MediaQueryList::try_new([
        MediaQuery::Condition(MediaCondition::Feature(MediaFeatureQuery::Width(
            RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(40.0, QueryLengthUnit::Rem)?,
            ),
        ))),
        MediaQuery::Typed(typed_query),
    ])?;
    assert_eq!(media_query_list.queries().len(), 2);
    let media_environment = MediaEnvironment::new()
        .media_type(MediaType::Screen)
        .width(QueryLength::try_new(800.0, QueryLengthUnit::Px)?)
        .height(QueryLength::try_new(400.0, QueryLengthUnit::Px)?)
        .with_length_basis(media_basis)
        .resolution(resolution)
        .color(color_bits)
        .monochrome(NonNegativeInteger::new(0))
        .with_orientation(Orientation::Landscape)
        .prefers_color_scheme(ColorSchemePreference::Dark)
        .prefers_reduced_motion(ReducedMotionPreference::NoPreference)
        .prefers_reduced_transparency(ReducedTransparencyPreference::NoPreference)
        .prefers_contrast(ContrastPreference::NoPreference)
        .forced_colors(ForcedColorsMode::None)
        .hover(HoverCapability::Hover)
        .any_hover(HoverCapability::Hover)
        .pointer(PointerCapability::Fine)
        .any_pointer(PointerCapability::Fine)
        .display_mode(DisplayMode::Browser);
    assert!(media_query_list.matches(&media_environment));
    assert_eq!(
        media_environment.orientation_fact(),
        Some(Orientation::Landscape)
    );
    assert_eq!(media_environment.orientation(), Some(Orientation::Landscape));
    assert!(Condition::media(media_query_list).is_media());

    let container_name = ContainerName::try_new("sidebar")?;
    let container_condition = ContainerCondition::And(ContainerConditionList::try_new([
        ContainerCondition::Feature(ContainerFeatureQuery::InlineSize(RangeFeature::new(
            Some(QueryComparison::GreaterThanOrEqual),
            QueryLength::try_new(320.0, QueryLengthUnit::Px)?,
        ))),
        ContainerCondition::Style(ContainerStyleQuery::CustomPropertyPresence(
            CustomPropertyName::try_new("--theme")?,
        )),
    ])?);
    let container_facts = ContainerFacts::new()
        .name(container_name.clone())
        .inline_size(QueryLength::try_new(400.0, QueryLengthUnit::Px)?)
        .with_length_basis(QueryLengthBasis::new().container_inline_size(
            QueryLength::try_new(400.0, QueryLengthUnit::Px)?,
        ))
        .custom_property(
            CustomPropertyName::try_new("--theme")?,
            AuthoredTokens::new("dark"),
        );
    assert!(container_condition.matches(&container_facts));
    assert_eq!(container_name.as_str(), "sidebar");
    assert_eq!(container_facts.name_fact(), Some(&container_name));
    assert_eq!(
        container_facts
            .length_basis()
            .container_inline_size_basis()
            .unwrap()
            .value(),
        400.0
    );
    assert!(Condition::container(container_condition).is_container());
    let condition_facts = ConditionFacts::new()
        .media(MediaEnvironment::new().width(QueryLength::try_new(800.0, QueryLengthUnit::Px)?))
        .container(ContainerFacts::new().width(QueryLength::try_new(320.0, QueryLengthUnit::Px)?));
    assert!(condition_facts.container_facts().is_some());

    let custom_name = CustomPropertyName::try_new("--brand")?;
    let authored_tokens = AuthoredTokens::new("var(--brand, #000)");
    assert_eq!(custom_name.as_str(), "--brand");
    assert_eq!(authored_tokens.as_css(), "var(--brand, #000)");

    let style_source = StyleSourceId::try_new(12)?;
    assert_eq!(style_source.get(), 12);
    let sourced_declaration = AuthoredDeclaration::try_new(
        AuthoredProperty::Property(Property::Color),
        AuthoredValue::Value(Value::StyleColor(StyleColor::current_color())),
    )?
    .with_source(StyleSourceId::try_new(100)?);
    assert_eq!(sourced_declaration.source().unwrap().get(), 100);
    let diagnostic = StyleDiagnostic::invalid_at_computed_value(
        StyleDiagnosticSubject::Property(Property::Color),
        Some(style_source),
        InvalidAtComputedValueReason::MissingCustomProperty(custom_name.clone()),
    );
    assert_eq!(diagnostic.kind(), StyleDiagnosticKind::InvalidAtComputedValue);
    assert_eq!(diagnostic.source(), Some(style_source));
    assert_eq!(
        diagnostic.subject(),
        &StyleDiagnosticSubject::Property(Property::Color)
    );
    assert_eq!(
        diagnostic.reason(),
        &InvalidAtComputedValueReason::MissingCustomProperty(custom_name.clone())
    );
    let custom_property_subject = StyleDiagnosticSubject::CustomProperty(custom_name.clone());
    assert!(matches!(
        custom_property_subject,
        StyleDiagnosticSubject::CustomProperty(_)
    ));

    let space_name = CustomPropertyName::try_new("--space")?;
    let variable_gap = VariableDependentValue::try_new(
        Property::Gap,
        AuthoredTokens::new("var(--space, 8px)"),
        VariableExpression::Reference(VariableReference::new(
            space_name,
            Some(VariableFallback::new(
                AuthoredTokens::new("8px"),
                VariableExpression::Value(Value::Length(Length::px(8.0))),
            )),
        )),
    )?;
    assert_eq!(variable_gap.property(), Property::Gap);
    assert_eq!(variable_gap.dependencies().len(), 1);

    let nested_fallback_width = VariableDependentValue::try_new(
        Property::Width,
        AuthoredTokens::new("var(--space, var(--fallback, 4px))"),
        VariableExpression::Reference(VariableReference::new(
            CustomPropertyName::try_new("--space")?,
            Some(VariableFallback::new(
                AuthoredTokens::new("var(--fallback, 4px)"),
                VariableExpression::Reference(VariableReference::new(
                    CustomPropertyName::try_new("--fallback")?,
                    Some(VariableFallback::new(
                        AuthoredTokens::new("4px"),
                        VariableExpression::Value(Value::Length(Length::px(4.0))),
                    )),
                )),
            )),
        )),
    )?;
    assert_eq!(nested_fallback_width.dependencies().len(), 2);

    let mut custom_value = CustomPropertyValue::new(
        AuthoredTokens::new("var(--brand, var(--fallback-brand, black))"),
        [VariableReference::new(
            CustomPropertyName::try_new("--brand")?,
            Some(VariableFallback::new(
                AuthoredTokens::new("var(--fallback-brand, black)"),
                VariableExpression::Reference(VariableReference::new(
                    CustomPropertyName::try_new("--fallback-brand")?,
                    Some(VariableFallback::new(
                        AuthoredTokens::new("black"),
                        VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                    )),
                )),
            )),
        )],
    );
    assert_eq!(custom_value.references()[0].name().as_str(), "--brand");
    assert_eq!(custom_value.dependencies().len(), 2);
    custom_value.try_push_typed_value(CustomPropertyTypedValue::try_new(
        Property::Color,
        VariableExpression::Reference(VariableReference::new(
            CustomPropertyName::try_new("--brand")?,
            Some(VariableFallback::new(
                AuthoredTokens::new("black"),
                VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
            )),
        )),
    )?)?;
    assert!(custom_value.typed_value(Property::Color).is_some());

    let literal_custom_value = CustomPropertyValue::new(AuthoredTokens::new("8px"), [])
        .try_with_typed_value(
            Property::Width,
            VariableExpression::Value(Value::Length(Length::px(8.0))),
        )?;
    assert_eq!(literal_custom_value.authored().as_css(), "8px");

    let mut authored = AuthoredDeclarations::new();
    authored.push(AuthoredDeclaration::css_wide(
        AuthoredProperty::All,
        CssWideKeyword::Initial,
    ));
    authored.try_push(AuthoredDeclaration::try_new(
        AuthoredProperty::Custom(CustomPropertyName::try_new("--brand")?),
        AuthoredValue::CustomProperty(custom_value),
    )?)?;
    authored.try_push(AuthoredDeclaration::try_new(
        AuthoredProperty::Property(Property::Color),
        AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
    )?)?;
    assert!(authored.len() >= 2);
    Ok(())
}

fn _accepts_resolved_with_diagnostics(_value: Option<ResolvedWithDiagnostics>) {}

struct PublicTree;

impl Tree for PublicTree {
    type Id = usize;

    fn version_hint(&self) -> Option<u64> {
        None
    }

    fn node(&self, id: Self::Id) -> surgeist_style::Result<Node<Self::Id>> {
        Ok(Node {
            id,
            tag: Some(StyleTag::new("button")?),
            key: None,
            classes: Vec::new(),
            attributes: Vec::new(),
            role: StyleRole::default(),
            state: StyleState::default(),
            text: false,
        })
    }

    fn parent(
        &self,
        _id: Self::Id,
        _traversal: Traversal,
    ) -> surgeist_style::Result<Option<Self::Id>> {
        Ok(None)
    }

    fn children(
        &self,
        _id: Self::Id,
        _traversal: Traversal,
    ) -> surgeist_style::Result<impl Iterator<Item = Self::Id> + '_> {
        Ok(std::iter::empty())
    }

    fn previous_sibling(
        &self,
        _id: Self::Id,
        _traversal: Traversal,
    ) -> surgeist_style::Result<Option<Self::Id>> {
        Ok(None)
    }
}
