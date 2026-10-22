//! Backgrounds 3, transforms, filters, masking, transitions and animations keep
//! authored syntax and reject invalid arguments and cross-family values.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn exact_number(component: Option<&surgeist_css::CssComponentValue>, representation: &str) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Number(number))) if number.representation() == representation)
}

fn checked_number(representation: &str) -> surgeist_css::CssSpecifiedNumber {
    surgeist_css::CssSpecifiedNumber::try_from_component(
        surgeist_css::CssComponentValue::try_number(representation).unwrap(),
    )
    .unwrap()
}

#[test]
fn parses_background_properties_as_authored_syntax() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-image: url(\"hero.png\"), none; }",
                CssKnownProperty::BackgroundImage
            ),
            BackgroundImage,
            images
        ),
        CssImageValueList::try_new(vec![
            CssImageValue::Url(CssUrl::new("hero.png")),
            CssImageValue::None,
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-position: left 10px top 20%; }",
                CssKnownProperty::BackgroundPosition
            ),
            BackgroundPosition,
            positions
        ),
        CssBackgroundPositionList::try_new(vec![
            CssBackgroundPosition::try_new(
                CssHorizontalPosition::LeftOffset(signed_length_percentage("10px"),),
                CssVerticalPosition::TopOffset(signed_length_percentage("20%"),),
            )
            .unwrap()
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-size: cover, 10px auto; }",
                CssKnownProperty::BackgroundSize
            ),
            BackgroundSize,
            sizes
        ),
        CssBackgroundSizeList::try_new(vec![
            CssBackgroundSize::Cover,
            CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Length(nonnegative_length_percentage("10px")),
                height: Some(CssBackgroundSizeComponent::Auto),
            },
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-repeat: repeat-x, no-repeat round; }",
                CssKnownProperty::BackgroundRepeat
            ),
            BackgroundRepeat,
            repeats
        ),
        CssBackgroundRepeatList::try_new(vec![
            CssBackgroundRepeat::RepeatX,
            CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::NoRepeat,
                y: CssBackgroundRepeatStyle::Round,
            },
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-origin: content-box; }",
                CssKnownProperty::BackgroundOrigin
            ),
            BackgroundOrigin,
            boxes
        ),
        CssBackgroundBoxList::try_new(vec![CssBackgroundBox::ContentBox]).unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-clip: padding-box; }",
                CssKnownProperty::BackgroundClip
            ),
            BackgroundClip,
            boxes
        ),
        CssBackgroundBoxList::try_new(vec![CssBackgroundBox::PaddingBox]).unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { background-attachment: fixed, local; }",
                CssKnownProperty::BackgroundAttachment
            ),
            BackgroundAttachment,
            attachments
        ),
        CssBackgroundAttachmentList::try_new(vec![
            CssBackgroundAttachment::Fixed,
            CssBackgroundAttachment::Local,
        ])
        .unwrap()
    );
}

#[test]
fn parses_interaction_and_outline_properties_as_authored_syntax() {
    assert_eq!(
        property_payload!(
            declaration(".panel { cursor: grab; }", CssKnownProperty::Cursor),
            Cursor,
            value
        ),
        CssCursor::Keyword(CssCursorKeyword::Grab)
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { pointer-events: none; }",
                CssKnownProperty::PointerEvents
            ),
            PointerEvents,
            value
        ),
        CssPointerEvents::None
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { user-select: text; }",
                CssKnownProperty::UserSelect
            ),
            UserSelect,
            value
        ),
        CssUserSelect::Text
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { outline: thick dotted white; }",
                CssKnownProperty::Outline
            ),
            Outline,
            value
        ),
        CssOutline::try_new(
            Some(CssOutlineWidth::Thick),
            Some(CssOutlineStyle::Dotted),
            Some(CssOutlineColor::Color(Box::new(CssColor::from_named(
                CssNamedColor::try_new("white").unwrap()
            )))),
        )
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { outline-width: 2px; }",
                CssKnownProperty::OutlineWidth
            ),
            OutlineWidth,
            value
        ),
        CssOutlineWidth::Length(nonnegative_length("2px"))
    );
}

#[test]
fn parses_transform_effect_and_mask_properties_as_authored_syntax() {
    let transform = property_payload!(
        declaration(
            ".panel { transform: translate(10px, 20px) rotate(45deg) scale(1.5); }",
            CssKnownProperty::Transform
        ),
        Transform,
        value
    );
    let CssTransform::Functions(functions) = transform else {
        panic!("expected transform functions");
    };
    assert_eq!(functions.functions().len(), 3);
    assert_eq!(
        functions.functions()[0].kind(),
        CssTransformFunctionKind::Translate
    );

    assert_eq!(
        property_payload!(
            declaration(
                ".panel { transform-origin: center top; }",
                CssKnownProperty::TransformOrigin
            ),
            TransformOrigin,
            origin
        ),
        CssTransformOrigin::try_new(
            CssPhysicalPosition::try_new(CssHorizontalPosition::Center, CssVerticalPosition::Top)
                .unwrap(),
            None,
        )
        .unwrap()
    );
    let CssFilter::Functions(filter) = property_payload!(
        declaration(
            ".panel { filter: blur(4px) opacity(50%); }",
            CssKnownProperty::Filter
        ),
        Filter,
        value
    ) else {
        panic!("filter functions");
    };
    assert_eq!(filter.functions().len(), 2);
    let CssFilterFunction::Blur(blur) = &filter.functions()[0] else {
        panic!("blur");
    };
    assert!(exact_dimension(
        blur.length().literal_component(),
        "4",
        "px"
    ));
    assert!(matches!(
        filter.functions()[1],
        CssFilterFunction::Opacity(CssFilterAmount::Percentage(_))
    ));
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { backdrop-filter: none; }",
                CssKnownProperty::BackdropFilter
            ),
            BackdropFilter,
            value
        ),
        CssFilter::None
    );
    let CssClipPath::BasicShape(clip_shape) = property_payload!(
        declaration(
            ".panel { clip-path: circle(50% at center); }",
            CssKnownProperty::ClipPath
        ),
        ClipPath,
        value
    ) else {
        panic!("typed percentage circle");
    };
    let CssBasicShape::Circle(circle) = clip_shape.shape() else {
        panic!("typed percentage circle");
    };
    assert!(
        matches!(circle.radius(), CssCircleRadius::LengthPercentage(radius)
        if exact_percentage(radius.literal_component(), "50"))
    );
    assert!(circle.position().is_some());
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { mask-image: url(mask.png), none; }",
                CssKnownProperty::MaskImage
            ),
            MaskImage,
            images
        ),
        CssImageValueList::try_new(vec![
            CssImageValue::Url(CssUrl::new("mask.png")),
            CssImageValue::None,
        ])
        .unwrap()
    );
    let mask_layers = property_payload!(
        declaration(
            ".panel { mask: url(mask.png) center / contain no-repeat; }",
            CssKnownProperty::Mask
        ),
        Mask,
        value
    );
    assert_eq!(mask_layers.layers().len(), 1);
}

#[test]
fn authored_transform_filter_and_basic_shape_values_preserve_family_context() {
    let CssTransform::Functions(functions) = property_payload!(
        declaration(
            ".panel { transform: translate(10px, 20px) rotate(45deg); }",
            CssKnownProperty::Transform
        ),
        Transform,
        value
    ) else {
        panic!("expected transform functions");
    };
    let CssTransformFunction::Translate(translation) = &functions.functions()[0] else {
        panic!("typed translate function");
    };
    assert!(exact_dimension(
        translation.x().literal_component(),
        "10",
        "px"
    ));
    assert!(exact_dimension(
        translation.y().unwrap().literal_component(),
        "20",
        "px"
    ));
    assert!(matches!(
        functions.functions()[1],
        CssTransformFunction::Rotate(CssAngleOrZero::Angle(ref value)) if value.literal().is_some()
    ));

    let CssFilter::Functions(functions) = property_payload!(
        declaration(
            ".panel { filter: blur(4px) opacity(50%); }",
            CssKnownProperty::Filter
        ),
        Filter,
        value
    ) else {
        panic!("expected filter functions");
    };
    let CssFilterFunction::Opacity(amount) = &functions.functions()[1] else {
        panic!("expected opacity filter");
    };
    assert!(matches!(amount, CssFilterAmount::Percentage(_)));

    let CssClipPath::BasicShape(clip_shape) = property_payload!(
        declaration(
            ".panel { clip-path: circle(50% at center); }",
            CssKnownProperty::ClipPath
        ),
        ClipPath,
        value
    ) else {
        panic!("expected basic shape clip-path");
    };
    let CssBasicShape::Circle(circle) = clip_shape.shape() else {
        panic!("expected basic shape clip-path");
    };
    assert!(
        matches!(circle.radius(), CssCircleRadius::LengthPercentage(radius)
        if exact_percentage(radius.literal_component(), "50"))
    );
    assert!(circle.position().is_some());

    let easings = property_payload!(
        declaration(
            ".panel { transition-timing-function: cubic-bezier(0.1, 0.2, 0.3, 1); }",
            CssKnownProperty::TransitionTimingFunction
        ),
        TransitionTimingFunction,
        timing_functions
    );
    let CssEasing::CubicBezier(bezier) = &easings.values()[0] else {
        panic!("expected cubic-bezier easing");
    };
    assert!(exact_number(
        (bezier.x1().value()).literal_component(),
        "0.1"
    ));
    assert!(exact_number((bezier.y2()).literal_component(), "1"));
}

#[test]
fn parses_transition_properties_and_preserves_comma_lists() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { transition-property: opacity, transform; }",
                CssKnownProperty::TransitionProperty
            ),
            TransitionProperty,
            properties
        ),
        CssTransitionPropertyList::try_new(vec![
            CssTransitionProperty::Custom(
                CssTransitionPropertyName::try_new(CssCustomIdent::try_new("opacity").unwrap())
                    .unwrap()
            ),
            CssTransitionProperty::Custom(
                CssTransitionPropertyName::try_new(CssCustomIdent::try_new("transform").unwrap())
                    .unwrap()
            ),
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { transition-duration: 150ms, 2s; }",
                CssKnownProperty::TransitionDuration
            ),
            TransitionDuration,
            durations
        ),
        CssDurationList::try_new(vec![
            CssDuration::try_new(CssTimeValue::from_literal(
                CssTimeLiteral::try_new("150", CssTimeUnit::Milliseconds).unwrap()
            ))
            .unwrap(),
            CssDuration::try_new(CssTimeValue::from_literal(
                CssTimeLiteral::try_new("2", CssTimeUnit::Seconds).unwrap()
            ))
            .unwrap(),
        ])
        .unwrap()
    );
    let easings = property_payload!(
        declaration(
            ".panel { transition-timing-function: ease-in, cubic-bezier(0.1, 0.2, 0.3, 1); }",
            CssKnownProperty::TransitionTimingFunction
        ),
        TransitionTimingFunction,
        timing_functions
    );
    assert!(matches!(
        easings.values(),
        [
            CssEasing::Keyword(CssEasingKeyword::EaseIn),
            CssEasing::CubicBezier(_)
        ]
    ));

    let transitions = property_payload!(
        declaration(
            ".panel { transition: opacity 150ms ease-in 20ms, transform 2s linear; }",
            CssKnownProperty::Transition
        ),
        Transition,
        transitions
    );
    assert_eq!(transitions.values().len(), 2);
}

#[test]
fn parses_animation_properties_and_preserves_comma_lists() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { animation-name: fade, none; }",
                CssKnownProperty::AnimationName
            ),
            AnimationName,
            names
        ),
        CssAnimationNameList::try_new(vec![
            CssAnimationName::Custom(
                CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap()
            ),
            CssAnimationName::None,
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { animation-iteration-count: 2, infinite; }",
                CssKnownProperty::AnimationIterationCount
            ),
            AnimationIterationCount,
            iteration_counts
        ),
        CssAnimationIterationCountList::try_new(vec![
            CssAnimationIterationCount::Number(nonnegative_number("2")),
            CssAnimationIterationCount::Infinite,
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { animation-play-state: running, paused; }",
                CssKnownProperty::AnimationPlayState
            ),
            AnimationPlayState,
            play_states
        ),
        CssAnimationPlayStateList::try_new(vec![
            CssAnimationPlayState::Running,
            CssAnimationPlayState::Paused,
        ])
        .unwrap()
    );

    // CSS Animations 1 §3.11: in each <single-animation> the first <time> is
    // the duration and the second the delay; omitted components stay absent.
    let animations = property_payload!(
        declaration(
            ".panel { animation: fade 1s ease-in 200ms 3 alternate both running, slide 2s linear; }",
            CssKnownProperty::Animation
        ),
        Animation,
        animations
    );
    let [first, second] = animations.values() else {
        panic!("two authored animations");
    };
    let name = |ident: &str| {
        CssAnimationName::Custom(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new(ident).unwrap()).unwrap(),
        )
    };
    let duration = |number: &str, unit| {
        CssDuration::try_new(CssTimeValue::from_literal(
            CssTimeLiteral::try_new(number, unit).unwrap(),
        ))
        .unwrap()
    };
    assert_eq!(first.name(), Some(&name("fade")));
    assert_eq!(first.duration(), Some(&duration("1", CssTimeUnit::Seconds)));
    assert!(matches!(
        first.timing_function(),
        Some(CssEasing::Keyword(CssEasingKeyword::EaseIn))
    ));
    // Compared structurally, without the parsed origin.
    assert_eq!(
        first
            .delay()
            .map(|delay| CssDuration::try_new(delay.clone()).unwrap()),
        Some(duration("200", CssTimeUnit::Milliseconds))
    );
    assert_eq!(
        first.iteration_count(),
        Some(&CssAnimationIterationCount::Number(nonnegative_number("3")))
    );
    assert_eq!(first.direction(), Some(CssAnimationDirection::Alternate));
    assert_eq!(first.fill_mode(), Some(CssAnimationFillMode::Both));
    assert_eq!(first.play_state(), Some(CssAnimationPlayState::Running));
    assert_eq!(second.name(), Some(&name("slide")));
    assert_eq!(
        second.duration(),
        Some(&duration("2", CssTimeUnit::Seconds))
    );
    assert!(matches!(
        second.timing_function(),
        Some(CssEasing::Keyword(CssEasingKeyword::Linear))
    ));
    assert!(second.delay().is_none());
    assert!(second.iteration_count().is_none());
    assert!(second.direction().is_none());
    assert!(second.fill_mode().is_none());
    assert!(second.play_state().is_none());
}

/// The cursor row of the retired acceptance matrix, with its typed
/// components: CSS UI 4 §5.2 `[<cursor-image>,]* <cursor-predefined>`, where a
/// `<url>` image without a hotspot keeps no coordinates.
#[test]
fn cursor_url_alternative_keeps_its_image_and_predefined_fallback() {
    let cursor = property_payload!(
        declaration(
            ".panel { cursor: url(cursor.png), pointer; }",
            CssKnownProperty::Cursor
        ),
        Cursor,
        value
    );
    let CssCursor::Images(images) = &cursor else {
        panic!("image alternatives: {cursor:?}");
    };
    let [image] = images.images() else {
        panic!("one cursor image");
    };
    let CssCursorImageSource::Url(url) = image.source() else {
        panic!("plain url() image");
    };
    assert_eq!(url.as_str(), "cursor.png");
    assert!(image.hotspot().is_none());
    assert_eq!(images.fallback(), CssCursorKeyword::Pointer);
}

#[test]
fn background_effect_and_animation_constructors_reject_invalid_states() {
    assert_eq!(CssImageValueList::try_new(Vec::new()), None);
    assert_eq!(
        CssCursorImages::try_new(Vec::new(), CssCursorKeyword::Pointer),
        None
    );
    assert!(CssCursor::try_images(Vec::new(), CssCursorKeyword::Pointer).is_none());
    let horizontal = signed_length_percentage("1px");
    assert!(
        CssPhysicalPosition::try_new(
            CssHorizontalPosition::LeftOffset(horizontal),
            CssVerticalPosition::Top
        )
        .is_err()
    );
    let vertical = signed_length_percentage("5%");
    assert!(
        CssPhysicalPosition::try_new(
            CssHorizontalPosition::Left,
            CssVerticalPosition::BottomOffset(vertical)
        )
        .is_err()
    );
    assert!(
        CssTranslateValues::try_new(
            signed_length_percentage("1px"),
            None,
            Some(signed_length("3px"))
        )
        .is_none()
    );
    for css in ["translate()", "translate3d(1px, 2px, 3px, 4px)"] {
        assert!(!parse_style_attribute(&format!("transform: {css}")).is_clean());
    }
    assert_eq!(CssScaleValues::try_new(Vec::new()), None);
    assert_eq!(
        CssScaleValues::try_new(vec![
            CssTransformScaleComponent::Number(checked_number("1")),
            CssTransformScaleComponent::Number(checked_number("2")),
            CssTransformScaleComponent::Number(checked_number("3")),
            CssTransformScaleComponent::Number(checked_number("4"))
        ]),
        None
    );
    assert_eq!(CssMaskList::try_new(Vec::new()), None);
    assert_eq!(CssTransitionList::try_new(Vec::new()), None);
    assert_eq!(CssTransition::try_new(None, None, None, None), None);
    assert_eq!(CssAnimationList::try_new(Vec::new()), None);
    assert_eq!(
        CssAnimation::try_new(CssAnimationComponents::default()),
        None
    );
    assert!(
        CssDuration::try_new(CssTimeValue::from_literal(
            CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap()
        ))
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number("-1").unwrap()
        )
        .is_err()
    );
    assert_eq!(CssOutline::try_new(None, None, None), None);
}

#[test]
fn background_effect_and_animation_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            background-image: none;
            background-position: center;
            background-size: contain;
            background-repeat: no-repeat;
            background-origin: border-box;
            background-clip: content-box;
            background-attachment: scroll;
            cursor: pointer;
            pointer-events: auto;
            user-select: all;
            outline: 1px solid black;
            outline-color: white;
            outline-style: dashed;
            outline-width: thin;
            transform: none;
            transform-origin: left top;
            translate: 10px 20px;
            rotate: 45deg;
            scale: 1.5 2;
            filter: none;
            backdrop-filter: blur(4px);
            clip-path: none;
            mask: none;
            mask-image: none;
            mask-size: auto;
            mask-position: center;
            mask-repeat: repeat;
            transition-property: opacity;
            transition-duration: 1s;
            transition-delay: 20ms;
            transition-timing-function: ease;
            transition: opacity 1s ease;
            animation-name: fade;
            animation-duration: 1s;
            animation-delay: 20ms;
            animation-timing-function: ease-out;
            animation-iteration-count: infinite;
            animation-direction: alternate;
            animation-fill-mode: both;
            animation-play-state: paused;
            animation: fade 1s ease-in-out infinite alternate both running;
        }",
    )
    .expect("clean parse report");
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::BackgroundPosition,
        CssKnownProperty::BackgroundSize,
        CssKnownProperty::BackgroundRepeat,
        CssKnownProperty::BackgroundOrigin,
        CssKnownProperty::BackgroundClip,
        CssKnownProperty::BackgroundAttachment,
        CssKnownProperty::Cursor,
        CssKnownProperty::PointerEvents,
        CssKnownProperty::UserSelect,
        CssKnownProperty::Outline,
        CssKnownProperty::OutlineColor,
        CssKnownProperty::OutlineStyle,
        CssKnownProperty::OutlineWidth,
        CssKnownProperty::Transform,
        CssKnownProperty::TransformOrigin,
        CssKnownProperty::Translate,
        CssKnownProperty::Rotate,
        CssKnownProperty::Scale,
        CssKnownProperty::Filter,
        CssKnownProperty::BackdropFilter,
        CssKnownProperty::ClipPath,
        CssKnownProperty::Mask,
        CssKnownProperty::MaskImage,
        CssKnownProperty::MaskSize,
        CssKnownProperty::MaskPosition,
        CssKnownProperty::MaskRepeat,
        CssKnownProperty::TransitionProperty,
        CssKnownProperty::TransitionDuration,
        CssKnownProperty::TransitionDelay,
        CssKnownProperty::TransitionTimingFunction,
        CssKnownProperty::Transition,
        CssKnownProperty::AnimationName,
        CssKnownProperty::AnimationDuration,
        CssKnownProperty::AnimationDelay,
        CssKnownProperty::AnimationTimingFunction,
        CssKnownProperty::AnimationIterationCount,
        CssKnownProperty::AnimationDirection,
        CssKnownProperty::AnimationFillMode,
        CssKnownProperty::AnimationPlayState,
        CssKnownProperty::Animation,
    ] {
        assert!(
            declarations.iter().any(|declaration| declaration
                .known()
                .map(|known| known.property())
                == Some(property)),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn background_effect_and_animation_properties_reject_cross_family_values_and_empty_lists() {
    for input in [
        ".panel { background-size: solid; }",
        ".panel { cursor: 10px; }",
        ".panel { pointer-events: grab; }",
        ".panel { outline-width: 10%; }",
        ".panel { transform: red; }",
        ".panel { filter: 10px; }",
        ".panel { transition-duration: 10px; }",
        ".panel { animation-iteration-count: -1; }",
        ".panel { animation-play-state: visible; }",
        ".panel { transition: opacity 1s, ; }",
        ".panel { animation: fade 1s, ; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}

#[test]
fn rejects_duplicate_axis_position_keywords_across_shared_position_properties() {
    for input in [
        ".panel { background-position: left right; }",
        ".panel { background-position: right left; }",
        ".panel { background-position: top bottom; }",
        ".panel { background-position: bottom top; }",
        ".panel { mask-position: left right; }",
        ".panel { mask-position: top bottom; }",
        ".panel { transform-origin: left right; }",
        ".panel { transform-origin: top bottom; }",
        ".panel { mask: url(mask.png) left right / contain no-repeat; }",
        ".panel { mask: url(mask.png) top bottom / contain no-repeat; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}

#[test]
fn preserves_valid_position_keyword_forms_after_duplicate_axis_rejection() {
    for input in [
        ".panel { background-position: left top; }",
        ".panel { background-position: right bottom; }",
        ".panel { background-position: center center; }",
        ".panel { background-position: left 10px top 20%; }",
        ".panel { mask-position: center center; }",
        ".panel { transform-origin: right bottom; }",
        ".panel { mask: url(mask.png) left top / contain no-repeat; }",
    ] {
        parse_sheet(input).unwrap_or_else(|error| panic!("{input} should parse: {error}"));
    }
}

#[test]
fn transform_filter_shape_and_easing_functions_reject_invalid_arguments() {
    for input in [
        ".panel { transform: translate(red); }",
        ".panel { filter: opacity(red); }",
        ".panel { clip-path: circle(red); }",
        ".panel { transition-timing-function: cubic-bezier(red); }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}
