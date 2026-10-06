// Independently authored facts migrated from the existing test inventories.
// Records are maintained directly; there is no production-schema generator.
property_records! {
    CaretAnimation, "caret-animation" {
        metadata: longhand(true, |v| assert_eq!(*v, CssCaretAnimation::Auto)),
        catalog: grammar_catalog!("official.property.caret-animation", "manual", rejected("running")),
        source: "X-UI4",
        dispatch: "manual",
        wrapper: yes,
    }
    CaretShape, "caret-shape" {
        metadata: longhand(true, |v| assert_eq!(*v, CssCaretShape::Auto)),
        catalog: grammar_catalog!("official.property.caret-shape", "underscore", rejected("circle")),
        source: "X-UI4",
        dispatch: "underscore",
        wrapper: yes,
    }
    Interactivity, "interactivity" {
        metadata: longhand(true, |v| assert_eq!(*v, CssInteractivity::Auto)),
        catalog: grammar_catalog!("official.property.interactivity", "inert", rejected("none")),
        source: "X-UI4",
        dispatch: "inert",
        wrapper: yes,
    }
    Appearance, "appearance" {
        metadata: longhand(false, |v| assert_eq!(*v, CssAppearance::None)),
        catalog: grammar_catalog!("official.property.appearance", "base-select", rejected("slider")),
        source: "X-UI4",
        aliases: ["-webkit-appearance"],
        dispatch: "base-select",
        wrapper: yes,
    }
    NavUp, "nav-up" {
        metadata: longhand(false, |v| assert!(matches!(v, CssNavigation::Auto))),
        catalog: grammar_catalog!("official.property.nav-up", "#Next root", rejected("#123")),
        source: "X-UI4",
        dispatch: "#Next root",
        wrapper: yes,
    }
    NavRight, "nav-right" {
        metadata: longhand(false, |v| assert!(matches!(v, CssNavigation::Auto))),
        catalog: grammar_catalog!("official.property.nav-right", "#Next root", rejected("#123")),
        source: "X-UI4",
        dispatch: "#Next root",
        wrapper: yes,
    }
    NavDown, "nav-down" {
        metadata: longhand(false, |v| assert!(matches!(v, CssNavigation::Auto))),
        catalog: grammar_catalog!("official.property.nav-down", "#Next root", rejected("#123")),
        source: "X-UI4",
        dispatch: "#Next root",
        wrapper: yes,
    }
    NavLeft, "nav-left" {
        metadata: longhand(false, |v| assert!(matches!(v, CssNavigation::Auto))),
        catalog: grammar_catalog!("official.property.nav-left", "#Next root", rejected("#123")),
        source: "X-UI4",
        dispatch: "#Next root",
        wrapper: yes,
    }
    InterestDelayStart, "interest-delay-start" {
        metadata: longhand(true, |v| assert!(matches!(v, CssInterestDelayValue::Normal))),
        catalog: grammar_catalog!("official.property.interest-delay-start", "-1s", rejected("1")),
        source: "X-UI4",
        dispatch: "-1s",
        wrapper: yes,
    }
    InterestDelayEnd, "interest-delay-end" {
        metadata: longhand(true, |v| assert!(matches!(v, CssInterestDelayValue::Normal))),
        catalog: grammar_catalog!("official.property.interest-delay-end", "-1s", rejected("1")),
        source: "X-UI4",
        dispatch: "-1s",
        wrapper: yes,
    }
    Caret, "caret" {
        metadata: shorthand([CaretColor, CaretAnimation, CaretShape], []),
        catalog: grammar_catalog!("official.property.caret", "red manual block", rejected("manual manual")),
        source: "X-UI4",
        dispatch: "red manual block",
        wrapper: yes,
    }
    InterestDelay, "interest-delay" {
        metadata: shorthand([InterestDelayStart, InterestDelayEnd], []),
        catalog: grammar_catalog!("official.property.interest-delay", "normal -1s", rejected("1s 2s 3s")),
        source: "X-UI4",
        dispatch: "normal -1s",
        wrapper: yes,
    }
    AccentColor, "accent-color" {
        metadata: longhand(true, |v| assert!(matches!(v, CssAccentColor::Auto))),
        catalog: grammar_catalog!("official.property.accent-color", "light-dark(red, blue)", rejected("auto auto")),
        source: "X-UI4",
        dispatch: "light-dark(red, blue)",
        wrapper: yes,
    }

    AlignContent, "align-content" {
        metadata: longhand(false, |v| {
            assert_eq!(v.value(), CssAlignmentValue::Normal { overflow: None })
        }),
        catalog: grammar_catalog!("baseline.property.align-content", "space-between", rejected("auto")),
        source: "S-ALIGN3",
        aliases: ["-webkit-align-content"],
        dispatch: "space-between",
        wrapper: yes,
    }
    AlignItems, "align-items" {
        metadata: longhand(false, |v| {
            assert_eq!(v.value(), CssAlignmentValue::Normal { overflow: None })
        }),
        catalog: grammar_catalog!("baseline.property.align-items", "first baseline", rejected("space-between")),
        source: "S-ALIGN3",
        aliases: ["-webkit-align-items"],
        dispatch: "first baseline",
        wrapper: yes,
    }
    AlignSelf, "align-self" {
        metadata: longhand(false, |v| assert_eq!(v.value(), CssAlignmentValue::Auto)),
        catalog: grammar_catalog!("baseline.property.align-self", "safe flex-end", rejected("space-between")),
        source: "S-ALIGN3",
        aliases: ["-webkit-align-self"],
        dispatch: "safe flex-end",
        wrapper: yes,
    }
    All, "all" {
        metadata: universal(),
        catalog: grammar_catalog!("baseline.property.all", "inherit", rejected("block")),
        source: "O-CASCADE4",
        dispatch: "block" => "inherit",
    }
    Animation, "animation" {
        metadata: shorthand([AnimationDuration, AnimationTimingFunction, AnimationDelay, AnimationIterationCount, AnimationDirection, AnimationFillMode, AnimationPlayState, AnimationName], []),
        catalog: grammar_catalog!("baseline.property.animation", "fade 1s ease-in 200ms 3 alternate both running", rejected("fade 1s 2s 3s")),
        source: "I-ANIMATIONS1",
        dispatch: "fade 1s ease-in 200ms 3 alternate both running",
        wrapper: yes,
    }
    AnimationDelay, "animation-delay" {
        metadata: longhand(false, |v| {
            assert_eq!(v.values().len(), 1);
            let literal = v.values()[0].literal().unwrap();
            assert_eq!(literal.numeric().representation(), "0");
            assert_eq!(literal.unit(), CssTimeUnit::Seconds);
        }),
        catalog: grammar_catalog!("baseline.property.animation-delay", "200ms", rejected("10px")),
        dispatch: "200ms",
        wrapper: yes,
    }
    AnimationDirection, "animation-direction" {
        metadata: longhand(false, |v| {
            assert_eq!(v.directions(), &[CssAnimationDirection::Normal])
        }),
        catalog: grammar_catalog!("baseline.property.animation-direction", "alternate", rejected("running")),
        dispatch: "alternate",
        wrapper: yes,
    }
    AnimationDuration, "animation-duration" {
        metadata: longhand(false, |v| {
            assert_eq!(v.values().len(), 1);
            let literal = v.values()[0].time().literal().unwrap();
            assert_eq!(literal.numeric().representation(), "0");
            assert_eq!(literal.unit(), CssTimeUnit::Seconds);
        }),
        catalog: grammar_catalog!("baseline.property.animation-duration", "1s", rejected("10px")),
        dispatch: "1s",
        wrapper: yes,
    }
    AnimationFillMode, "animation-fill-mode" {
        metadata: longhand(false, |v| {
            assert_eq!(v.modes(), &[CssAnimationFillMode::None])
        }),
        catalog: grammar_catalog!("baseline.property.animation-fill-mode", "both", rejected("running")),
        dispatch: "both",
        wrapper: yes,
    }
    AnimationIterationCount, "animation-iteration-count" {
        metadata: longhand(false, |v| {
            let [CssAnimationIterationCount::Number(value)] = v.values() else {
                panic!("one ordinary initial count")
            };
            assert_eq!(value.serialize_specified().unwrap(), "1");
            assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        }),
        catalog: grammar_catalog!("baseline.property.animation-iteration-count", "2, infinite", rejected("-1")),
        dispatch: "2, infinite",
        wrapper: yes,
    }
    AnimationName, "animation-name" {
        metadata: longhand(false, |v| assert_eq!(v.names(), &[CssAnimationName::None])),
        catalog: grammar_catalog!("baseline.property.animation-name", "fade, none", rejected("default")),
        dispatch: "fade, none",
        wrapper: yes,
    }
    AnimationPlayState, "animation-play-state" {
        metadata: longhand(false, |v| {
            assert_eq!(v.states(), &[CssAnimationPlayState::Running])
        }),
        catalog: grammar_catalog!("baseline.property.animation-play-state", "running, paused", rejected("alternate")),
        dispatch: "running, paused",
        wrapper: yes,
    }
    AnimationTimingFunction, "animation-timing-function" {
        metadata: longhand(false, |v| {
            assert_eq!(v.values(), &[CssEasing::Keyword(CssEasingKeyword::Ease)])
        }),
        catalog: grammar_catalog!("baseline.property.animation-timing-function", "ease-out", rejected("bounce")),
        dispatch: "ease-out",
        wrapper: yes,
    }
    AspectRatio, "aspect-ratio" {
        metadata: longhand(false, |v| {
            assert!(matches!(v, CssAspectRatioValue::Auto));
        }),
        catalog: grammar_catalog!("baseline.property.aspect-ratio", "1.5", rejected("solid")),
        source: "X-SIZING4-20260904",
        dispatch: "1.5",
        wrapper: yes,
    }
    BackdropFilter, "backdrop-filter" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssFilter::None)
        }),
        catalog: grammar_catalog!("baseline.property.backdrop-filter", "none", rejected("opacity(red)"), "#BackdropFilterProperty"),
        source: "X-BACKDROP-FILTER",
        dispatch: "none",
        wrapper: yes,
    }
    BackfaceVisibility, "backface-visibility" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBackfaceVisibility::Visible)),
        catalog: grammar_catalog!("official.property.backface-visibility", "hidden", rejected("auto")),
        source: "I-TRANSFORMS2",
        dispatch: "hidden",
        wrapper: yes,
    }
    Background, "background" {
        metadata: shorthand([BackgroundImage, BackgroundPosition, BackgroundSize, BackgroundRepeat, BackgroundAttachment, BackgroundOrigin, BackgroundClip, BackgroundColor], [BackgroundBlendMode]),
        catalog: grammar_catalog!("baseline.property.background", "#fff", rejected("#fff #000")),
        source: "O-BACKGROUNDS3",
        dispatch: "#fff",
        wrapper: yes,
    }
    BackgroundAttachment, "background-attachment" {
        metadata: longhand(false, |v| {
            assert_eq!(v.attachments(), &[CssBackgroundAttachment::Scroll])
        }),
        catalog: grammar_catalog!("baseline.property.background-attachment", "fixed, local", rejected("sticky")),
        dispatch: "fixed, local",
        wrapper: yes,
    }
    BackgroundBlendMode, "background-blend-mode" {
        metadata: longhand(false, |v| {
            assert_eq!(v.modes(), &[CssBlendMode::Normal])
        }),
        catalog: grammar_catalog!("official.property.background-blend-mode", "multiply, luminosity", rejected("multiply luminosity")),
    }
    BackgroundClip, "background-clip" {
        metadata: longhand(false, |v| {
            assert_eq!(v.boxes(), &[CssBackgroundBox::BorderBox])
        }),
        catalog: grammar_catalog!("baseline.property.background-clip", "padding-box", rejected("margin-box")),
        dispatch: "padding-box",
        wrapper: yes,
    }
    BackgroundColor, "background-color" {
        metadata: longhand(false, |v| assert_eq!(v, &CssColor::transparent())),
        catalog: grammar_catalog!("baseline.property.background-color", "transparent", rejected("black white")),
        dispatch: "transparent",
        wrapper: yes,
    }
    BackgroundImage, "background-image" {
        metadata: longhand(false, |v| {
            assert!(matches!(v.images(), [CssImageValue::None]));
        }),
        catalog: grammar_catalog!("baseline.property.background-image", "url(\"hero.png\"), none", rejected("url(foo bar)")),
        dispatch: "url(\"hero.png\"), none",
        wrapper: yes,
    }
    BackgroundOrigin, "background-origin" {
        metadata: longhand(false, |v| {
            assert_eq!(v.boxes(), &[CssBackgroundBox::PaddingBox])
        }),
        catalog: grammar_catalog!("baseline.property.background-origin", "content-box", rejected("margin-box")),
        dispatch: "content-box",
        wrapper: yes,
    }
    BackgroundPosition, "background-position" {
        metadata: longhand(false, |v| {
            let [position] = v.positions() else {
                panic!("one initial background position");
            };
            let (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) =
                (position.horizontal(), position.vertical())
            else {
                panic!("initial background position has two free offsets");
            };
            for offset in [x, y] {
                assert!(matches!(offset.literal_component().unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
                        if number.representation() == "0"));
            }
        }),
        catalog: grammar_catalog!("baseline.property.background-position", "left 10px top 20%", rejected("left right")),
        dispatch: "left 10px top 20%",
        wrapper: yes,
    }
    BackgroundRepeat, "background-repeat" {
        metadata: longhand(false, |v| assert_eq!(
            v.repeats(),
            &[CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::Repeat,
                y: CssBackgroundRepeatStyle::Repeat
            },]
        )),
        catalog: grammar_catalog!("baseline.property.background-repeat", "repeat-x, no-repeat round", rejected("solid")),
        dispatch: "repeat-x, no-repeat round",
        wrapper: yes,
    }
    BackgroundSize, "background-size" {
        metadata: longhand(false, |v| assert!(matches!(
            v.sizes(),
            [CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Auto,
                height: None
            }]
        ))),
        catalog: grammar_catalog!("baseline.property.background-size", "cover, 10px auto", rejected("solid")),
        dispatch: "cover, 10px auto",
        wrapper: yes,
    }
    BlockSize, "block-size" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
    }
    Border, "border" {
        metadata: shorthand([BorderTopWidth, BorderRightWidth, BorderBottomWidth, BorderLeftWidth, BorderTopStyle, BorderRightStyle, BorderBottomStyle, BorderLeftStyle, BorderTopColor, BorderRightColor, BorderBottomColor, BorderLeftColor], [BorderImageSource, BorderImageSlice, BorderImageWidth, BorderImageOutset, BorderImageRepeat]),
        catalog: grammar_catalog!("baseline.property.border", "solid 2px #fff", rejected("solid dotted")),
        dispatch: "solid 2px #fff",
        wrapper: yes,
    }
    BorderBlock, "border-block" {
        metadata: shorthand([BorderBlockStartWidth, BorderBlockEndWidth, BorderBlockStartStyle, BorderBlockEndStyle, BorderBlockStartColor, BorderBlockEndColor], []),
    }
    BorderBlockColor, "border-block-color" {
        metadata: shorthand([BorderBlockStartColor, BorderBlockEndColor], []),
    }
    BorderBlockEnd, "border-block-end" {
        metadata: shorthand([BorderBlockEndWidth, BorderBlockEndStyle, BorderBlockEndColor], []),
    }
    BorderBlockEndColor, "border-block-end-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
    }
    BorderBlockEndStyle, "border-block-end-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
    }
    BorderBlockEndWidth, "border-block-end-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
    }
    BorderBlockStart, "border-block-start" {
        metadata: shorthand([BorderBlockStartWidth, BorderBlockStartStyle, BorderBlockStartColor], []),
    }
    BorderBlockStartColor, "border-block-start-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
    }
    BorderBlockStartStyle, "border-block-start-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
    }
    BorderBlockStartWidth, "border-block-start-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
    }
    BorderBlockStyle, "border-block-style" {
        metadata: shorthand([BorderBlockStartStyle, BorderBlockEndStyle], []),
    }
    BorderBlockWidth, "border-block-width" {
        metadata: shorthand([BorderBlockStartWidth, BorderBlockEndWidth], []),
    }
    BorderBottom, "border-bottom" {
        metadata: shorthand([BorderBottomWidth, BorderBottomStyle, BorderBottomColor], []),
        catalog: grammar_catalog!("baseline.property.border-bottom", "#fff", rejected("solid dotted")),
        dispatch: "#fff",
        wrapper: yes,
    }
    BorderBottomColor, "border-bottom-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
        catalog: grammar_catalog!("baseline.property.border-bottom-color", "transparent", rejected("black white")),
        dispatch: "transparent",
        wrapper: yes,
    }
    BorderBottomLeftRadius, "border-bottom-left-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.border-bottom-left-radius", "calc(1px + 2%)", rejected("-1px")),
        dispatch: "calc(1px + 2%)",
        wrapper: yes,
    }
    BorderBottomRightRadius, "border-bottom-right-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.border-bottom-right-radius", "10%", rejected("-1px")),
        dispatch: "10%",
        wrapper: yes,
    }
    BorderBottomStyle, "border-bottom-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
        catalog: grammar_catalog!("baseline.property.border-bottom-style", "ridge", rejected("auto")),
        dispatch: "ridge",
        wrapper: yes,
    }
    BorderBottomWidth, "border-bottom-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
        catalog: grammar_catalog!("baseline.property.border-bottom-width", "3px", rejected("10%")),
        dispatch: "3px",
        wrapper: yes,
    }
    BorderCollapse, "border-collapse" {
        metadata: longhand(true, |v| assert_eq!(*v, CssBorderCollapse::Separate)),
        catalog: grammar_catalog!("official.property.border-collapse", "collapse", rejected("auto")),
    }
    BorderColor, "border-color" {
        metadata: four_side(),
        catalog: grammar_catalog!("baseline.property.border-color", "black", rejected("black white black white black")),
        dispatch: "black",
        wrapper: yes,
    }
    BorderEndEndRadius, "border-end-end-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
    }
    BorderEndStartRadius, "border-end-start-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
    }
    BorderImage, "border-image" {
        metadata: shorthand([BorderImageSource, BorderImageSlice, BorderImageWidth, BorderImageOutset, BorderImageRepeat], []),
        catalog: grammar_catalog!("official.property.border-image", "url(frame.png) 10 fill / 2 / 1 round", rejected("url(frame.png) / 2")),
    }
    BorderImageOutset, "border-image-outset" {
        metadata: longhand(false, |v| {
            assert!(!v.values().is_empty());
            assert!(v.values().iter().all(
                |v| matches!(v, CssBorderImageOutsetComponent::Number(n) if exact_literal(n.literal_component(), "0"))
            ));
        }),
        catalog: grammar_catalog!("official.property.border-image-outset", "1 2px 3 4px", rejected("-1")),
    }
    BorderImageRepeat, "border-image-repeat" {
        metadata: longhand(false, |v| {
            assert_eq!(v.horizontal(), CssBorderImageRepeatKeyword::Stretch);
            assert_eq!(v.vertical(), CssBorderImageRepeatKeyword::Stretch);
        }),
        catalog: grammar_catalog!("official.property.border-image-repeat", "round space", rejected("repeat round stretch")),
    }
    BorderImageSlice, "border-image-slice" {
        metadata: longhand(false, |v| {
            assert!(!v.fill());
            assert!(!v.values().is_empty());
            assert!(v.values().iter().all(
                |v| matches!(v, CssBorderImageSliceComponent::Percentage(n) if exact_literal(n.literal_component(), "100%"))
            ));
        }),
        catalog: grammar_catalog!("official.property.border-image-slice", "10 fill", rejected("10 20 30 40 50")),
    }
    BorderImageSource, "border-image-source" {
        metadata: longhand(false, |v| assert!(matches!(v, CssImageValue::None))),
        catalog: grammar_catalog!("official.property.border-image-source", "linear-gradient(red, blue)", rejected("cover")),
    }
    BorderImageWidth, "border-image-width" {
        metadata: longhand(false, |v| {
            assert!(!v.values().is_empty());
            assert!(
                v.values().iter().all(
                    |v| matches!(v, CssBorderImageWidthComponent::Number(n) if exact_literal(n.literal_component(), "1"))
                )
            );
        }),
        catalog: grammar_catalog!("official.property.border-image-width", "1 auto 25% 4px", rejected("-1")),
    }
    BorderInline, "border-inline" {
        metadata: shorthand([BorderInlineStartWidth, BorderInlineEndWidth, BorderInlineStartStyle, BorderInlineEndStyle, BorderInlineStartColor, BorderInlineEndColor], []),
    }
    BorderInlineColor, "border-inline-color" {
        metadata: shorthand([BorderInlineStartColor, BorderInlineEndColor], []),
    }
    BorderInlineEnd, "border-inline-end" {
        metadata: shorthand([BorderInlineEndWidth, BorderInlineEndStyle, BorderInlineEndColor], []),
    }
    BorderInlineEndColor, "border-inline-end-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
    }
    BorderInlineEndStyle, "border-inline-end-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
    }
    BorderInlineEndWidth, "border-inline-end-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
    }
    BorderInlineStart, "border-inline-start" {
        metadata: shorthand([BorderInlineStartWidth, BorderInlineStartStyle, BorderInlineStartColor], []),
    }
    BorderInlineStartColor, "border-inline-start-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
    }
    BorderInlineStartStyle, "border-inline-start-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
    }
    BorderInlineStartWidth, "border-inline-start-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
    }
    BorderInlineStyle, "border-inline-style" {
        metadata: shorthand([BorderInlineStartStyle, BorderInlineEndStyle], []),
    }
    BorderInlineWidth, "border-inline-width" {
        metadata: shorthand([BorderInlineStartWidth, BorderInlineEndWidth], []),
    }
    BorderLeft, "border-left" {
        metadata: shorthand([BorderLeftWidth, BorderLeftStyle, BorderLeftColor], []),
        catalog: grammar_catalog!("baseline.property.border-left", "dashed black", rejected("solid dotted")),
        dispatch: "dashed black",
        wrapper: yes,
    }
    BorderLeftColor, "border-left-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
        catalog: grammar_catalog!("baseline.property.border-left-color", "#fff", rejected("black white")),
        dispatch: "#fff",
        wrapper: yes,
    }
    BorderLeftStyle, "border-left-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
        catalog: grammar_catalog!("baseline.property.border-left-style", "outset", rejected("auto")),
        dispatch: "outset",
        wrapper: yes,
    }
    BorderLeftWidth, "border-left-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
        catalog: grammar_catalog!("baseline.property.border-left-width", "4px", rejected("10%")),
        dispatch: "4px",
        wrapper: yes,
    }
    BorderRadius, "border-radius" {
        metadata: shorthand([BorderTopLeftRadius, BorderTopRightRadius, BorderBottomRightRadius, BorderBottomLeftRadius], []),
        catalog: grammar_catalog!("baseline.property.border-radius", "1px 2px 3px / 4px 5px", rejected("-1px")),
        dispatch: "1px 2px 3px / 4px 5px",
        wrapper: yes,
    }
    BorderRight, "border-right" {
        metadata: shorthand([BorderRightWidth, BorderRightStyle, BorderRightColor], []),
        catalog: grammar_catalog!("baseline.property.border-right", "1px", rejected("solid dotted")),
        dispatch: "1px",
        wrapper: yes,
    }
    BorderRightColor, "border-right-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
        catalog: grammar_catalog!("baseline.property.border-right-color", "white", rejected("black white")),
        dispatch: "white",
        wrapper: yes,
    }
    BorderRightStyle, "border-right-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
        catalog: grammar_catalog!("baseline.property.border-right-style", "double", rejected("auto")),
        dispatch: "double",
        wrapper: yes,
    }
    BorderRightWidth, "border-right-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
        catalog: grammar_catalog!("baseline.property.border-right-width", "2px", rejected("10%")),
        dispatch: "2px",
        wrapper: yes,
    }
    BorderSpacing, "border-spacing" {
        metadata: longhand(true, |v| {
            assert!(exact_literal(v.horizontal().literal_component(), "0"));
            assert!(exact_literal(v.vertical().literal_component(), "0"));
        }),
        catalog: grammar_catalog!("official.property.border-spacing", "2px 3px", rejected("-1px")),
    }
    BorderStartEndRadius, "border-start-end-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
    }
    BorderStartStartRadius, "border-start-start-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
    }
    BorderStyle, "border-style" {
        metadata: four_side(),
        catalog: grammar_catalog!("baseline.property.border-style", "none hidden dotted dashed", rejected("auto")),
        dispatch: "none hidden dotted dashed",
        wrapper: yes,
    }
    BorderTop, "border-top" {
        metadata: shorthand([BorderTopWidth, BorderTopStyle, BorderTopColor], []),
        catalog: grammar_catalog!("baseline.property.border-top", "black dotted", rejected("solid dotted")),
        dispatch: "black dotted",
        wrapper: yes,
    }
    BorderTopColor, "border-top-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
        catalog: grammar_catalog!("baseline.property.border-top-color", "black", rejected("black white")),
        dispatch: "black",
        wrapper: yes,
    }
    BorderTopLeftRadius, "border-top-left-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.border-top-left-radius", "4px 10%", rejected("-1px")),
        dispatch: "4px 10%",
        wrapper: yes,
    }
    BorderTopRightRadius, "border-top-right-radius" {
        metadata: longhand(false, |v| {
            assert_eq!(
                v,
                &CssCornerRadiusValue::new(CssSpecifiedNonNegativeLengthPercentage::zero(), None)
            );
            assert_eq!(v.horizontal(), v.vertical());
            assert!(v.authored_vertical().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.border-top-right-radius", "1px", rejected("-1px")),
        dispatch: "1px",
        wrapper: yes,
    }
    BorderTopStyle, "border-top-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
        catalog: grammar_catalog!("baseline.property.border-top-style", "solid", rejected("auto")),
        dispatch: "solid",
        wrapper: yes,
    }
    BorderTopWidth, "border-top-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
        catalog: grammar_catalog!("baseline.property.border-top-width", "1px", rejected("10%")),
        dispatch: "1px",
        wrapper: yes,
    }
    BorderWidth, "border-width" {
        metadata: four_side(),
        catalog: grammar_catalog!("baseline.property.border-width", "1px 2px 3px 4px", rejected("10%")),
        dispatch: "1px 2px 3px 4px",
        wrapper: yes,
    }
    Bottom, "bottom" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
        catalog: grammar_catalog!("baseline.property.bottom", "5%", rejected("solid")),
        dispatch: "5%",
        wrapper: yes,
    }
    BoxDecorationBreak, "box-decoration-break" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssBoxDecorationBreak::Slice)
        }),
        catalog: grammar_catalog!("baseline.property.box-decoration-break", "clone", rejected("auto")),
        source: "S-BREAK3",
        dispatch: "clone",
        wrapper: yes,
    }
    BoxShadow, "box-shadow" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBoxShadow::None)),
        catalog: grammar_catalog!("baseline.property.box-shadow", "inset 1px 2px 3px 4px black", rejected("1px 2px -3px")),
        dispatch: "inset 1px 2px 3px 4px black",
        wrapper: yes,
    }
    BoxSizing, "box-sizing" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBoxSizing::ContentBox)),
        catalog: grammar_catalog!("baseline.property.box-sizing", "border-box", rejected("padding-box")),
        source: "I-SIZING3-20260904",
        dispatch: "border-box",
        wrapper: yes,
    }
    BreakAfter, "break-after" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssBreakBetween::Auto)
        }),
        catalog: grammar_catalog!("official.property.break-after", "right", rejected("always")),
    }
    BreakBefore, "break-before" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssBreakBetween::Auto)
        }),
        catalog: grammar_catalog!("official.property.break-before", "page", rejected("always")),
    }
    BreakInside, "break-inside" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBreakInside::Auto)),
        catalog: grammar_catalog!("official.property.break-inside", "avoid", rejected("left")),
    }
    CaptionSide, "caption-side" {
        metadata: longhand(true, |v| assert_eq!(*v, CssCaptionSide::Top)),
        catalog: grammar_catalog!("official.property.caption-side", "bottom", rejected("left")),
    }
    CaretColor, "caret-color" {
        metadata: longhand(true, |v| assert!(matches!(v, CssCaretColor::Auto))),
        catalog: grammar_catalog!("official.property.caret-color", "rebeccapurple", rejected("auto auto")),
        source: "X-UI4",
        dispatch: "rebeccapurple",
        wrapper: yes,
    }
    Clear, "clear" {
        metadata: longhand(false, |v| assert_eq!(*v, CssClear::None)),
        catalog: grammar_catalog!("baseline.property.clear", "both", rejected("start"), "#float-clear"),
        dispatch: "both",
        wrapper: yes,
    }
    Clip, "clip" {
        metadata: longhand(false, |v| assert_eq!(*v, CssClip::Auto)),
        catalog: grammar_catalog!("official.property.clip", "rect(auto, 10px, 20px, -1px)", rejected("rect(1px, 2px, 3px)")),
    }
    ClipPath, "clip-path" {
        metadata: longhand(false, |v| assert_eq!(*v, CssClipPath::None)),
        catalog: grammar_catalog!("baseline.property.clip-path", "circle(50% at center)", rejected("circle(red)")),
        dispatch: "circle(50% at center)",
        wrapper: yes,
    }
    ShapeOutside, "shape-outside" {
        metadata: longhand(false, |v| assert_eq!(*v, CssShapeOutside::None)),
        catalog: complete_grammar_catalog!("official.property.shape-outside", "margin-box circle(50%)", rejected("circle() fill-box")),
        source: "S-SHAPES1",
        dispatch: "margin-box circle(50%)",
        wrapper: yes,
    }
    ShapeImageThreshold, "shape-image-threshold" {
        metadata: longhand(false, |v| {
            let CssOpacityValue::Scalar(value) = v else { panic!("initial threshold number") };
            assert_eq!(value.kind(), CssOpacityScalarKind::Number);
            assert_eq!(value.numeric().representation(), "0");
            assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        }),
        catalog: complete_grammar_catalog!("official.property.shape-image-threshold", "150%", rejected("1px")),
        source: "S-SHAPES1",
        dispatch: "150%",
        wrapper: yes,
    }
    ShapeMargin, "shape-margin" {
        metadata: longhand(false, |v| {
            let component = v.literal_component().expect("initial margin literal");
            assert!(matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(value)) if value.representation() == "0"));
            assert_eq!(v.origin(), &CssValueOrigin::Programmatic);
            assert!(v.calculation().is_none());
        }),
        catalog: complete_grammar_catalog!("official.property.shape-margin", "calc(1px - 2px)", rejected("-1px")),
        source: "S-SHAPES1",
        dispatch: "calc(1px - 2px)",
        wrapper: yes,
    }
    Color, "color" {
        metadata: longhand(true, |v| {
            assert_eq!(v.system(), Some(CssSystemColor::CanvasText))
        }),
        catalog: grammar_catalog!("baseline.property.color", "black", rejected("black white")),
        source: "O-COLOR4",
        dispatch: "black",
        wrapper: yes,
    }
    ColorInterpolationFilters, "color-interpolation-filters" {
        metadata: longhand(true, |v| assert_eq!(*v, CssColorInterpolationFilters::LinearRgb)),
        catalog: grammar_catalog!("official.property.color-interpolation-filters", "sRGB", rejected("rgb"), "#ColorInterpolationFiltersProperty"),
        source: "I-FILTER1",
        dispatch: "sRGB",
        wrapper: yes,
    }
    ColorAdjust, "color-adjust" {
        metadata: shorthand([PrintColorAdjust], []),
        source: "R-COLORADJUST1",
        dispatch: "economy",
        wrapper: yes,
    }
    ColorScheme, "color-scheme" {
        metadata: longhand(true, |v| assert_eq!(*v, CssColorScheme::normal())),
        source: "R-COLORADJUST1",
        dispatch: "only light Future light",
        wrapper: yes,
    }
    ColumnCount, "column-count" {
        metadata: longhand(false, |v| assert_eq!(*v, CssColumnCount::Auto)),
        dispatch: "3",
        wrapper: yes,
    }
    ColumnFill, "column-fill" {
        metadata: longhand(false, |v| assert_eq!(*v, CssColumnFill::Balance)),
        dispatch: "balance-all",
        wrapper: yes,
    }
    ColumnGap, "column-gap" {
        metadata: longhand(false, |v| {
            assert_eq!(v, &CssGapValue::Normal)
        }),
        catalog: grammar_catalog!("baseline.property.column-gap", "5%", rejected("auto")),
        source: "S-ALIGN3",
        aliases: ["grid-column-gap"],
        dispatch: "5%",
        wrapper: yes,
    }
    ColumnRule, "column-rule" {
        metadata: shorthand([ColumnRuleWidth, ColumnRuleStyle, ColumnRuleColor], []),
        dispatch: "thick dashed rebeccapurple",
        wrapper: yes,
    }
    ColumnRuleColor, "column-rule-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
        dispatch: "currentcolor",
        wrapper: yes,
    }
    ColumnRuleStyle, "column-rule-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderStyle::None)),
        dispatch: "double",
        wrapper: yes,
    }
    ColumnRuleWidth, "column-rule-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBorderWidth::Medium)),
        dispatch: "2px",
        wrapper: yes,
    }
    ColumnSpan, "column-span" {
        metadata: longhand(false, |v| assert_eq!(*v, CssColumnSpan::None)),
        dispatch: "all",
        wrapper: yes,
    }
    ColumnWidth, "column-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
        dispatch: "12em",
        wrapper: yes,
    }
    Columns, "columns" {
        metadata: shorthand([ColumnWidth, ColumnCount], []),
        dispatch: "4 10rem",
        wrapper: yes,
    }
    Contain, "contain" {
        metadata: longhand(false, |v| assert_eq!(*v, CssContain::None)),
        catalog: grammar_catalog!("official.property.contain", "paint style size", rejected("size size")),
        source: "I-CONTAIN2",
        dispatch: "paint style size",
        wrapper: yes,
    }
    ContainIntrinsicBlockSize, "contain-intrinsic-block-size" {
        metadata: longhand(false, |v| {
            assert_eq!(
                *v,
                CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None)
            );
            assert!(!v.uses_auto());
            assert!(matches!(
                v.fallback(),
                CssContainIntrinsicSizeFallback::None
            ));
        }),
    }
    ContainIntrinsicHeight, "contain-intrinsic-height" {
        metadata: longhand(false, |v| {
            assert_eq!(
                *v,
                CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None)
            );
            assert!(!v.uses_auto());
            assert!(matches!(
                v.fallback(),
                CssContainIntrinsicSizeFallback::None
            ));
        }),
    }
    ContainIntrinsicInlineSize, "contain-intrinsic-inline-size" {
        metadata: longhand(false, |v| {
            assert_eq!(
                *v,
                CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None)
            );
            assert!(!v.uses_auto());
            assert!(matches!(
                v.fallback(),
                CssContainIntrinsicSizeFallback::None
            ));
        }),
    }
    ContainIntrinsicSize, "contain-intrinsic-size" {
        metadata: shorthand([ContainIntrinsicWidth, ContainIntrinsicHeight], []),
    }
    ContainIntrinsicWidth, "contain-intrinsic-width" {
        metadata: longhand(false, |v| {
            assert_eq!(
                *v,
                CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None)
            );
            assert!(!v.uses_auto());
            assert!(matches!(
                v.fallback(),
                CssContainIntrinsicSizeFallback::None
            ));
        }),
    }
    Container, "container" {
        metadata: shorthand([ContainerName, ContainerType], []),
    }
    ContainerName, "container-name" {
        metadata: longhand(false, |v| assert_eq!(*v, CssContainerNames::None)),
    }
    ContainerType, "container-type" {
        metadata: longhand(false, |v| assert_eq!(*v, CssContainerType::Normal)),
    }
    Content, "content" {
        metadata: longhand(false, |v| assert_eq!(v, &CssContentValue::Normal)),
        catalog: grammar_catalog!("baseline.property.content", "\"Chapter \"", rejected("contents normal")),
        dispatch: "\"Chapter \"",
        wrapper: yes,
    }
    ContentVisibility, "content-visibility" {
        metadata: longhand(false, |v| {
            assert_eq!(v, &CssContentVisibility::Visible)
        }),
        catalog: grammar_catalog!("baseline.property.content-visibility", "auto", rejected("collapse")),
        source: "I-CONTAIN2",
        dispatch: "auto",
        wrapper: yes,
    }
    CounterIncrement, "counter-increment" {
        metadata: longhand(false, |v| assert!(v.changes().is_none())),
        catalog: grammar_catalog!("baseline.property.counter-increment", "section 1", rejected("1")),
        dispatch: "section 1",
        wrapper: yes,
    }
    CounterReset, "counter-reset" {
        metadata: longhand(false, |v| assert!(v.changes().is_none())),
        catalog: grammar_catalog!("baseline.property.counter-reset", "section 2", rejected("none item")),
        dispatch: "section 2",
        wrapper: yes,
    }
    CounterSet, "counter-set" {
        metadata: longhand(false, |v| assert!(v.changes().is_none())),
        catalog: grammar_catalog!("baseline.property.counter-set", "section 3", rejected("none item")),
        source: "I-LISTS3",
        dispatch: "section 3",
        wrapper: yes,
    }
    Cue, "cue" {
        metadata: shorthand([CueBefore, CueAfter], []),
    }
    CueAfter, "cue-after" {
        metadata: longhand(false, |value| {
            assert_eq!(*value, CssCue::None)
        }),
    }
    CueBefore, "cue-before" {
        metadata: longhand(false, |value| {
            assert_eq!(*value, CssCue::None)
        }),
    }
    Cursor, "cursor" {
        metadata: longhand(true, |v| assert_eq!(v, &CssCursor::Keyword(CssCursorKeyword::Auto))),
        catalog: grammar_catalog!("baseline.property.cursor", "grab", rejected("10px")),
        source: "X-UI4",
        dispatch: "grab",
        wrapper: yes,
    }
    Direction, "direction" {
        metadata: longhand(true, |v| assert_eq!(v, &CssDirection::Ltr)),
        catalog: grammar_catalog!("baseline.property.direction", "rtl", rejected("block")),
        source: "O-WRITING3",
        dispatch: "rtl",
        wrapper: yes,
    }
    Display, "display" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssDisplayValue::OutsideInside {
                outside: CssDisplayOutside::Inline,
                inside: CssDisplayInside::Flow,
            },
        )),
        catalog: grammar_catalog!("baseline.property.display", "block", rejected("block inline")),
        source: "S-DISPLAY3",
        dispatch: "block",
        wrapper: yes,
    }
    EmptyCells, "empty-cells" {
        metadata: longhand(true, |v| assert_eq!(*v, CssEmptyCells::Show)),
        catalog: grammar_catalog!("official.property.empty-cells", "hide", rejected("auto")),
    }
    Filter, "filter" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssFilter::None)
        }),
        catalog: grammar_catalog!("baseline.property.filter", "blur(4px) opacity(50%)", rejected("opacity(red)")),
        source: "I-FILTER1",
        dispatch: "blur(4px) opacity(50%)",
        wrapper: yes,
    }
    FloodColor, "flood-color" {
        metadata: longhand(false, |v| assert_eq!(v.named().unwrap().name(), "black")),
        catalog: grammar_catalog!("official.property.flood-color", "color(display-p3 1 0 0)", rejected("black white"), "#FloodColorProperty"),
        source: "I-FILTER1",
        dispatch: "currentColor",
        wrapper: yes,
    }
    FloodOpacity, "flood-opacity" {
        metadata: longhand(false, |v| {
            let CssOpacityValue::Scalar(value) = v else { panic!("initial alpha is a number") };
            assert_eq!(value.kind(), CssOpacityScalarKind::Number);
            assert_eq!(value.numeric().representation(), "1");
            assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        }),
        catalog: grammar_catalog!("official.property.flood-opacity", "150%", rejected("none"), "#FloodOpacityProperty"),
        source: "I-FILTER1",
        dispatch: "-25%",
        wrapper: yes,
    }
    Flex, "flex" {
        metadata: shorthand([FlexGrow, FlexShrink, FlexBasis], []),
        catalog: grammar_catalog!("baseline.property.flex", "2 0 10rem", rejected("-1")),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex"],
        dispatch: "2 0 10rem",
        wrapper: yes,
    }
    FlexBasis, "flex-basis" {
        metadata: longhand(false, |v| {
            assert!(matches!(
                v.view(),
                CssFlexBasisRef::Size(CssSizeValue::Auto)
            ))
        }),
        catalog: grammar_catalog!("baseline.property.flex-basis", "10rem", rejected("solid")),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex-basis"],
        dispatch: "10rem",
        wrapper: yes,
    }
    FlexDirection, "flex-direction" {
        metadata: longhand(false, |v| assert_eq!(*v, CssFlexDirection::Row)),
        catalog: grammar_catalog!("baseline.property.flex-direction", "column-reverse", rejected("wrap")),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex-direction"],
        dispatch: "column-reverse",
        wrapper: yes,
    }
    FlexFlow, "flex-flow" {
        metadata: shorthand([FlexDirection, FlexWrap], []),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex-flow"],
        dispatch: "wrap-reverse column",
        wrapper: yes,
    }
    FlexGrow, "flex-grow" {
        metadata: longhand(false, |v| {
            assert_eq!(v.serialize_specified().unwrap(), "0")
        }),
        catalog: grammar_catalog!("baseline.property.flex-grow", "2", rejected("solid")),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex-grow"],
        dispatch: "2",
        wrapper: yes,
    }
    FlexShrink, "flex-shrink" {
        metadata: longhand(false, |v| {
            assert_eq!(v.serialize_specified().unwrap(), "1")
        }),
        catalog: grammar_catalog!("baseline.property.flex-shrink", "0", rejected("solid")),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex-shrink"],
        dispatch: "0",
        wrapper: yes,
    }
    FlexWrap, "flex-wrap" {
        metadata: longhand(false, |v| assert_eq!(*v, CssFlexWrap::NoWrap)),
        catalog: grammar_catalog!("baseline.property.flex-wrap", "wrap-reverse", rejected("column")),
        source: "O-FLEXBOX1",
        aliases: ["-webkit-flex-wrap"],
        dispatch: "wrap-reverse",
        wrapper: yes,
    }
    Float, "float" {
        metadata: longhand(false, |v| assert_eq!(*v, CssFloat::None)),
        catalog: grammar_catalog!("baseline.property.float", "left", rejected("center"), "#float-clear"),
        dispatch: "left",
        wrapper: yes,
    }
    FlowTolerance, "flow-tolerance" {
        metadata: longhand(false, |v| assert_eq!(v, &CssFlowTolerance::normal())),
        catalog: grammar_catalog!("ext.property.flow-tolerance", "infinite", rejected("solid")),
        source: "X-GRID3-20260121",
        dispatch: "infinite",
        wrapper: yes,
    }
    Font, "font" {
        metadata: shorthand([FontFamily, FontSize, FontWidth, FontStyle, FontVariantCaps, FontWeight, LineHeight], [FontFeatureSettings, FontKerning, FontLanguageOverride, FontOpticalSizing, FontSizeAdjust, FontVariantAlternates, FontVariantEastAsian, FontVariantEmoji, FontVariantLigatures, FontVariantNumeric, FontVariantPosition, FontVariationSettings]),
        catalog: grammar_catalog!("baseline.property.font", "italic small-caps 700 condensed 16px/normal \"Avenir Next\", sans-serif", rejected("bold sans-serif")),
        source: "I-FONTS4-20260907",
        dispatch: "italic small-caps 700 condensed 16px/normal \"Avenir Next\", sans-serif",
        wrapper: yes,
    }
    FontFamily, "font-family" {
        metadata: longhand(true, ua(CssUserAgentInitial::FontFamily)),
        catalog: grammar_catalog!("baseline.property.font-family", "\"Avenir Next\", sans-serif", rejected("sans-serif,")),
        dispatch: "\"Avenir Next\", sans-serif",
        wrapper: yes,
    }
    FontFeatureSettings, "font-feature-settings" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssAuthoredFontFeatureSettings::Normal)
        }),
        catalog: grammar_catalog!("baseline.property.font-feature-settings", "\"kern\" on, \"liga\" 0", rejected("\"abc\" on")),
        dispatch: "\"kern\" on, \"liga\" 0",
        wrapper: yes,
    }
    FontKerning, "font-kerning" {
        metadata: longhand(true, |v| assert_eq!(v, &CssFontKerning::Auto)),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-kerning", authored: "normal", production: "#propdef-font-kerning" },
        source: "I-FONTS4-20260907",
        dispatch: "normal",
        wrapper: yes,
    }
    FontLanguageOverride, "font-language-override" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontLanguageOverride::Normal)
        }),
    }
    FontOpticalSizing, "font-optical-sizing" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontOpticalSizing::Auto)
        }),
    }
    FontPalette, "font-palette" {
        metadata: longhand(true, |v| assert_eq!(v, &CssFontPalette::Normal)),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-palette", authored: "palette-mix(light, dark)", production: "#propdef-font-palette" },
        source: "I-FONTS4-20260907",
        wrapper: yes,
    }
    FontSize, "font-size" {
        metadata: longhand(true, |v| assert_eq!(v, &CssFontSize::Medium)),
        catalog: grammar_catalog!("baseline.property.font-size", "16px", rejected("auto")),
        dispatch: "16px",
        wrapper: yes,
    }
    FontSizeAdjust, "font-size-adjust" {
        metadata: longhand(true, |v| assert_eq!(v, &CssFontSizeAdjust::None)),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-size-adjust", authored: "0.5", production: "#propdef-font-size-adjust" },
        source: "I-FONTS4-20260907",
        dispatch: "0.5",
        wrapper: yes,
    }
    FontStyle, "font-style" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontStyle::Keyword(CssFontStyleKeyword::Normal))
        }),
        catalog: grammar_catalog!("baseline.property.font-style", "italic", rejected("bold")),
        dispatch: "italic",
        wrapper: yes,
    }
    FontSynthesis, "font-synthesis" {
        metadata: shorthand([FontSynthesisWeight, FontSynthesisStyle, FontSynthesisSmallCaps, FontSynthesisPosition], []),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-synthesis", authored: "style weight", production: "#propdef-font-synthesis" },
        source: "I-FONTS4-20260907",
        dispatch: "position small-caps style weight",
        wrapper: yes,
    }
    FontSynthesisPosition, "font-synthesis-position" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontSynthesisPosition::Auto)
        }),
        dispatch: "auto",
        wrapper: yes,
    }
    FontSynthesisSmallCaps, "font-synthesis-small-caps" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontSynthesisSmallCaps::Auto)
        }),
        dispatch: "none",
        wrapper: yes,
    }
    FontSynthesisStyle, "font-synthesis-style" {
        metadata: longhand(true, |v| assert_eq!(v, &CssFontSynthesisStyle::Auto)),
        dispatch: "oblique-only",
        wrapper: yes,
    }
    FontSynthesisWeight, "font-synthesis-weight" {
        metadata: longhand(true, |v| assert_eq!(v, &CssFontSynthesisWeight::Auto)),
        dispatch: "auto",
        wrapper: yes,
    }
    FontVariant, "font-variant" {
        metadata: shorthand([FontVariantLigatures, FontVariantCaps, FontVariantAlternates, FontVariantNumeric, FontVariantEastAsian, FontVariantPosition, FontVariantEmoji], []),
        catalog: grammar_catalog!("baseline.property.font-variant", "small-caps", rejected("italic")),
        dispatch: "small-caps",
        wrapper: yes,
    }
    FontVariantAlternates, "font-variant-alternates" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantAlternates::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-alternates", authored: "styleset(Alpha, beta)", production: "#propdef-font-variant-alternates" },
        source: "I-FONTS4-20260907",
    }
    FontVariantCaps, "font-variant-caps" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantCaps::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-caps", authored: "all-small-caps", production: "#propdef-font-variant-caps" },
        source: "I-FONTS4-20260907",
    }
    FontVariantEastAsian, "font-variant-east-asian" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantEastAsian::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-east-asian", authored: "jis04 ruby", production: "#propdef-font-variant-east-asian" },
        source: "I-FONTS4-20260907",
    }
    FontVariantEmoji, "font-variant-emoji" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantEmoji::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-emoji", authored: "emoji", production: "#propdef-font-variant-emoji" },
        source: "I-FONTS4-20260907",
    }
    FontVariantLigatures, "font-variant-ligatures" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantLigatures::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-ligatures", authored: "common-ligatures no-discretionary-ligatures", production: "#propdef-font-variant-ligatures" },
        source: "I-FONTS4-20260907",
    }
    FontVariantNumeric, "font-variant-numeric" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantNumeric::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-numeric", authored: "lining-nums tabular-nums slashed-zero", production: "#propdef-font-variant-numeric" },
        source: "I-FONTS4-20260907",
    }
    FontVariantPosition, "font-variant-position" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariantPosition::Normal)
        }),
        catalog: CatalogExpectation::Complete { feature_id: "official.property.font-variant-position", authored: "super", production: "#propdef-font-variant-position" },
        source: "I-FONTS4-20260907",
    }
    FontVariationSettings, "font-variation-settings" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontVariationSettings::Normal)
        }),
    }
    FontWeight, "font-weight" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal))
        }),
        catalog: grammar_catalog!("baseline.property.font-weight", "725", rejected("1001")),
        dispatch: "725",
        wrapper: yes,
    }
    FontWidth, "font-width" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssFontWidth::Keyword(CssFontWidthKeyword::Normal))
        }),
        catalog: grammar_catalog!("baseline.property.font-stretch", "semi-condensed", rejected("wide")),
        source: "I-FONTS4-20260907",
        aliases: ["font-stretch"],
        dispatch: "semi-condensed",
        wrapper: yes,
    }
    ForcedColorAdjust, "forced-color-adjust" {
        metadata: longhand(true, |v| assert_eq!(*v, CssForcedColorAdjust::Auto)),
        source: "R-COLORADJUST1",
        dispatch: "preserve-parent-color",
        wrapper: yes,
    }
    FrameSizing, "frame-sizing" {
        metadata: longhand(false, |v| assert_eq!(*v, CssFrameSizing::Auto)),
    }
    Gap, "gap" {
        metadata: shorthand([RowGap, ColumnGap], []),
        catalog: grammar_catalog!("baseline.property.gap", "12px", rejected("auto")),
        source: "S-ALIGN3",
        aliases: ["grid-gap"],
        dispatch: "12px",
        wrapper: yes,
    }
    Grid, "grid" {
        metadata: shorthand([GridTemplateRows, GridTemplateColumns, GridTemplateAreas, GridAutoRows, GridAutoColumns, GridAutoFlow], []),
        catalog: grammar_catalog!("baseline.property.grid", "auto-flow dense 12px / repeat(auto-fit, 10px)", rejected("auto-flow")),
        source: "R-GRID2",
        dispatch: "auto-flow dense 12px / repeat(auto-fit, 10px)",
        wrapper: yes,
    }
    GridArea, "grid-area" {
        metadata: shorthand([GridRowStart, GridColumnStart, GridRowEnd, GridColumnEnd], []),
        catalog: grammar_catalog!("baseline.property.grid-area", "header / 1 / span 2 / main", rejected("0")),
        dispatch: "header / 1 / span 2 / main",
        wrapper: yes,
    }
    GridAutoColumns, "grid-auto-columns" {
        metadata: longhand(false, |v| {
            let [size] = v.sizes() else {
                panic!("exactly one implicit track initial")
            };
            assert_eq!(size.kind(), CssGridTrackSizeKind::Breadth);
            assert_eq!(
                size.breadth().unwrap().kind(),
                CssGridTrackBreadthKind::Auto
            );
        }),
        catalog: grammar_catalog!("baseline.property.grid-auto-columns", "fit-content(20%)", rejected("solid")),
        dispatch: "fit-content(20%)",
        wrapper: yes,
    }
    GridAutoFlow, "grid-auto-flow" {
        metadata: longhand(false, |v| assert_eq!(*v, CssGridAutoFlow::Normal)),
        catalog: grammar_catalog!("baseline.property.grid-auto-flow", "column dense", rejected("left")),
        dispatch: "column dense",
        wrapper: yes,
    }
    GridAutoRows, "grid-auto-rows" {
        metadata: longhand(false, |v| {
            let [size] = v.sizes() else {
                panic!("exactly one implicit track initial")
            };
            assert_eq!(size.kind(), CssGridTrackSizeKind::Breadth);
            assert_eq!(
                size.breadth().unwrap().kind(),
                CssGridTrackBreadthKind::Auto
            );
        }),
        catalog: grammar_catalog!("baseline.property.grid-auto-rows", "minmax(10px, auto)", rejected("solid")),
        dispatch: "minmax(10px, auto)",
        wrapper: yes,
    }
    GridColumn, "grid-column" {
        metadata: shorthand([GridColumnStart, GridColumnEnd], []),
        catalog: grammar_catalog!("baseline.property.grid-column", "nav / main", rejected("0")),
        dispatch: "nav / main",
        wrapper: yes,
    }
    GridColumnEnd, "grid-column-end" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssGridLine::Auto)
        }),
        catalog: grammar_catalog!("baseline.property.grid-column-end", "4", rejected("0")),
        dispatch: "4",
        wrapper: yes,
    }
    GridColumnStart, "grid-column-start" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssGridLine::Auto)
        }),
        catalog: grammar_catalog!("baseline.property.grid-column-start", "nav", rejected("0")),
        dispatch: "nav",
        wrapper: yes,
    }
    GridRow, "grid-row" {
        metadata: shorthand([GridRowStart, GridRowEnd], []),
        catalog: grammar_catalog!("baseline.property.grid-row", "1 / span 2", rejected("0")),
        dispatch: "1 / span 2",
        wrapper: yes,
    }
    GridRowEnd, "grid-row-end" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssGridLine::Auto)
        }),
        catalog: grammar_catalog!("baseline.property.grid-row-end", "auto", rejected("0")),
        dispatch: "auto",
        wrapper: yes,
    }
    GridRowStart, "grid-row-start" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssGridLine::Auto)
        }),
        catalog: grammar_catalog!("baseline.property.grid-row-start", "span 2 main", rejected("0")),
        dispatch: "span 2 main",
        wrapper: yes,
    }
    GridTemplate, "grid-template" {
        metadata: shorthand([GridTemplateRows, GridTemplateColumns, GridTemplateAreas], []),
        catalog: grammar_catalog!("baseline.property.grid-template", "100px 1fr / repeat(2, minmax(10px, 1fr))", rejected("solid")),
        dispatch: "100px 1fr / repeat(2, minmax(10px, 1fr))",
        wrapper: yes,
    }
    GridTemplateAreas, "grid-template-areas" {
        metadata: longhand(false, |v| {
            assert_eq!(v, &CssGridTemplateAreas::None)
        }),
        catalog: grammar_catalog!("baseline.property.grid-template-areas", "\"header header\" \"nav main\"", rejected("\"a a\" \"a .\"")),
        dispatch: "\"header header\" \"nav main\"",
        wrapper: yes,
    }
    GridTemplateColumns, "grid-template-columns" {
        metadata: longhand(false, |v| {
            assert!(v.is_none());
            assert!(v.general_list().is_none());
            assert!(v.auto_list().is_none());
            assert!(v.subgrid_components().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.grid-template-columns", "repeat(2, minmax(10px, 1fr))", rejected("solid")),
        dispatch: "repeat(2, minmax(10px, 1fr))",
        wrapper: yes,
    }
    GridTemplateRows, "grid-template-rows" {
        metadata: longhand(false, |v| {
            assert!(v.is_none());
            assert!(v.general_list().is_none());
            assert!(v.auto_list().is_none());
            assert!(v.subgrid_components().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.grid-template-rows", "[top] 100px 1fr", rejected("solid")),
        dispatch: "[top] 100px 1fr",
        wrapper: yes,
    }
    HangingPunctuation, "hanging-punctuation" {
        metadata: longhand(true, |v| assert_eq!(*v, CssHangingPunctuation::None)),
        catalog: grammar_catalog!("ext.property.hanging-punctuation", "last allow-end first", rejected("force-end allow-end")),
        source: "X-TEXT4",
        dispatch: "last allow-end first",
        wrapper: yes,
    }
    Height, "height" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
        catalog: grammar_catalog!("baseline.property.height", "auto", rejected("solid")),
        dispatch: "auto",
        wrapper: yes,
    }
    HyphenateCharacter, "hyphenate-character" {
        metadata: longhand(true, |v| assert_eq!(*v, CssHyphenateCharacter::Auto)),
        catalog: grammar_catalog!("ext.property.hyphenate-character", "\"\"", rejected("none")),
        source: "X-TEXT4",
        dispatch: "\"\"",
        wrapper: yes,
    }
    HyphenateLimitChars, "hyphenate-limit-chars" {
        metadata: longhand(true, |v| assert_eq!(*v, CssHyphenateLimitChars::auto())),
        catalog: grammar_catalog!("ext.property.hyphenate-limit-chars", "auto 2 3", rejected("2 -1")),
        source: "X-TEXT4",
        dispatch: "auto 2 3",
        wrapper: yes,
    }
    HyphenateLimitLast, "hyphenate-limit-last" {
        metadata: longhand(true, |v| assert_eq!(*v, CssHyphenateLimitLast::None)),
        catalog: grammar_catalog!("ext.property.hyphenate-limit-last", "spread", rejected("auto")),
        source: "X-TEXT4",
        dispatch: "spread",
        wrapper: yes,
    }
    HyphenateLimitLines, "hyphenate-limit-lines" {
        metadata: longhand(true, |v| assert_eq!(*v, CssHyphenateLimitLines::NoLimit)),
        catalog: grammar_catalog!("ext.property.hyphenate-limit-lines", "calc(3 / 2)", rejected("-1")),
        source: "X-TEXT4",
        dispatch: "calc(3 / 2)",
        wrapper: yes,
    }
    HyphenateLimitZone, "hyphenate-limit-zone" {
        metadata: longhand(true, |v| assert_eq!(*v, CssSpecifiedLengthPercentage::zero())),
        catalog: grammar_catalog!("ext.property.hyphenate-limit-zone", "calc(-2px + 3%)", rejected("auto")),
        source: "X-TEXT4",
        dispatch: "calc(-2px + 3%)",
        wrapper: yes,
    }
    Hyphens, "hyphens" {
        metadata: longhand(true, |v| assert_eq!(*v, CssHyphens::Manual)),
        catalog: grammar_catalog!("ext.property.hyphens", "manual", rejected("none auto")),
        source: "X-TEXT4",
        dispatch: "manual",
        wrapper: yes,
    }
    ImageOrientation, "image-orientation" {
        metadata: longhand(true, |v| assert_eq!(*v, CssImageOrientation::FromImage)),
        catalog: grammar_catalog!("official.property.image-orientation", "90deg flip", rejected("flip 90deg flip"), "#propdef-image-orientation"),
        source: "O-IMAGES3",
        dispatch: "flip 30deg",
        wrapper: yes,
    }
    ImageRendering, "image-rendering" {
        metadata: longhand(true, |v| assert_eq!(*v, CssImageRendering::Auto)),
        catalog: grammar_catalog!("official.property.image-rendering", "high-quality", rejected("smooth high-quality"), "#propdef-image-rendering"),
        source: "O-IMAGES3",
        dispatch: "smooth",
        wrapper: yes,
    }
    InlineSize, "inline-size" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
    }
    Inset, "inset" {
        metadata: four_side(),
        catalog: grammar_catalog!("baseline.property.inset", "auto 10px 5%", rejected("solid")),
        source: "I-POSITION3",
        dispatch: "auto 10px 5%",
        wrapper: yes,
    }
    InsetBlock, "inset-block" {
        metadata: shorthand([InsetBlockStart, InsetBlockEnd], []),
    }
    InsetBlockEnd, "inset-block-end" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
    }
    InsetBlockStart, "inset-block-start" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
    }
    InsetInline, "inset-inline" {
        metadata: shorthand([InsetInlineStart, InsetInlineEnd], []),
    }
    InsetInlineEnd, "inset-inline-end" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
    }
    InsetInlineStart, "inset-inline-start" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
    }
    Isolation, "isolation" {
        metadata: longhand(false, |v| assert_eq!(*v, CssIsolation::Auto)),
        catalog: grammar_catalog!("official.property.isolation", "isolate", rejected("none")),
    }
    ItemDirection, "item-direction" {
        metadata: longhand(false, |v| assert_eq!(*v, CssItemDirection::Auto)),
        catalog: grammar_catalog!("ext.property.item-direction", "column-reverse", rejected("reverse")),
        source: "X-GRID3-20260121",
        dispatch: "column-reverse",
        wrapper: yes,
    }
    ItemFlow, "item-flow" {
        metadata: shorthand([ItemDirection, ItemWrap, ItemPack, FlowTolerance], []),
        catalog: grammar_catalog!("ext.property.item-flow", "balance dense reverse -2px row", rejected("wrap row reverse")),
        source: "X-GRID3-20260121",
        dispatch: "balance dense reverse -2px row",
        wrapper: yes,
    }
    ItemPack, "item-pack" {
        metadata: longhand(false, |v| assert_eq!(*v, CssItemPack::Normal)),
        catalog: grammar_catalog!("ext.property.item-pack", "balance dense", rejected("normal dense")),
        source: "X-GRID3-20260121",
        dispatch: "balance dense",
        wrapper: yes,
    }
    ItemWrap, "item-wrap" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssItemWrap::Mode(CssItemWrapMode::Auto))
        }),
        catalog: grammar_catalog!("ext.property.item-wrap", "reverse wrap", rejected("normal reverse")),
        source: "X-GRID3-20260121",
        dispatch: "reverse wrap",
        wrapper: yes,
    }
    JustifyContent, "justify-content" {
        metadata: longhand(false, |v| {
            assert_eq!(v.value(), CssAlignmentValue::Normal { overflow: None })
        }),
        catalog: grammar_catalog!("baseline.property.justify-content", "safe center", rejected("auto")),
        source: "S-ALIGN3",
        aliases: ["-webkit-justify-content"],
        dispatch: "safe center",
        wrapper: yes,
    }
    JustifyItems, "justify-items" {
        metadata: longhand(false, |v| {
            assert_eq!(v.value(), CssAlignmentValue::Legacy(None))
        }),
        catalog: grammar_catalog!("baseline.property.justify-items", "stretch", rejected("space-between")),
        dispatch: "stretch",
        wrapper: yes,
    }
    JustifySelf, "justify-self" {
        metadata: longhand(false, |v| assert_eq!(v.value(), CssAlignmentValue::Auto)),
        catalog: grammar_catalog!("baseline.property.justify-self", "center", rejected("space-between")),
        dispatch: "center",
        wrapper: yes,
    }
    Left, "left" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
        catalog: grammar_catalog!("baseline.property.left", "calc(3px + 4%)", rejected("solid")),
        dispatch: "calc(3px + 4%)",
        wrapper: yes,
    }
    LetterSpacing, "letter-spacing" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssTextSpacingAdjustment::Normal)
        }),
        catalog: grammar_catalog!("baseline.property.letter-spacing", "0.1em", rejected("auto")),
        source: "X-TEXT4",
        dispatch: "0.1em",
        wrapper: yes,
    }
    LightingColor, "lighting-color" {
        metadata: longhand(false, |v| assert_eq!(v.named().unwrap().name(), "white")),
        catalog: grammar_catalog!("official.property.lighting-color", "rgb(from red r g b)", rejected("white black"), "#LightingColorProperty"),
        source: "I-FILTER1",
        dispatch: "light-dark(red, blue)",
        wrapper: yes,
    }
    LineBreak, "line-break" {
        metadata: longhand(true, |v| assert_eq!(*v, CssLineBreak::Auto)),
        catalog: grammar_catalog!("ext.property.line-break", "anywhere", rejected("avoid")),
        source: "X-TEXT4",
        wrapper: yes,
    }
    LineHeight, "line-height" {
        metadata: longhand(true, |v| assert_eq!(v, &CssLineHeight::Normal)),
        catalog: grammar_catalog!("baseline.property.line-height", "normal", rejected("auto")),
        dispatch: "normal",
        wrapper: yes,
    }
    LinePadding, "line-padding" {
        metadata: longhand(true, |v| assert_eq!(*v, CssSpecifiedLength::zero())),
        catalog: grammar_catalog!("ext.property.line-padding", "-2px", rejected("2%")),
        source: "X-TEXT4",
        dispatch: "-2px",
        wrapper: yes,
    }
    ListStyle, "list-style" {
        metadata: shorthand([ListStylePosition, ListStyleImage, ListStyleType], []),
        catalog: grammar_catalog!("baseline.property.list-style", "url(marker.svg) inside square", rejected("inside outside square")),
        dispatch: "url(marker.svg) inside square",
        wrapper: yes,
    }
    ListStyleImage, "list-style-image" {
        metadata: longhand(true, |v| assert_eq!(v, &CssImageValue::None)),
        catalog: grammar_catalog!("baseline.property.list-style-image", "url(marker.svg)", rejected("red")),
        dispatch: "url(marker.svg)",
        wrapper: yes,
    }
    ListStylePosition, "list-style-position" {
        metadata: longhand(true, |v| {
            assert_eq!(*v, CssListStylePosition::Outside)
        }),
        catalog: grammar_catalog!("baseline.property.list-style-position", "inside", rejected("center")),
        dispatch: "inside",
        wrapper: yes,
    }
    ListStyleType, "list-style-type" {
        metadata: longhand(true, |v| {
            assert_eq!(v.serialize_specified().unwrap(), "disc")
        }),
        catalog: grammar_catalog!("baseline.property.list-style-type", "square", rejected("symbols(numeric \"*\")")),
        dispatch: "square",
        wrapper: yes,
    }
    Margin, "margin" {
        metadata: four_side(),
        catalog: grammar_catalog!("baseline.property.margin", "auto 10px 5%", rejected("solid")),
        source: "O-BOX3",
        dispatch: "auto 10px 5%",
        wrapper: yes,
    }
    MarginBlock, "margin-block" {
        metadata: shorthand([MarginBlockStart, MarginBlockEnd], []),
    }
    MarginBlockEnd, "margin-block-end" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
    }
    MarginBlockStart, "margin-block-start" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
    }
    MarginBottom, "margin-bottom" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.margin-bottom", "5%", rejected("solid")),
        dispatch: "5%",
        wrapper: yes,
    }
    MarginInline, "margin-inline" {
        metadata: shorthand([MarginInlineStart, MarginInlineEnd], []),
    }
    MarginInlineEnd, "margin-inline-end" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
    }
    MarginInlineStart, "margin-inline-start" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
    }
    MarginLeft, "margin-left" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.margin-left", "calc(3px + 4%)", rejected("solid")),
        dispatch: "calc(3px + 4%)",
        wrapper: yes,
    }
    MarginRight, "margin-right" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.margin-right", "10px", rejected("solid")),
        dispatch: "10px",
        wrapper: yes,
    }
    MarginTop, "margin-top" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssMarginValue::LengthPercentage(CssSpecifiedLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.margin-top", "auto", rejected("solid")),
        dispatch: "auto",
        wrapper: yes,
    }
    MarkerSide, "marker-side" {
        metadata: longhand(true, |v| assert_eq!(*v, CssMarkerSide::MatchSelf)),
    }
    Mask, "mask" {
        metadata: shorthand([MaskImage, MaskPosition, MaskSize, MaskRepeat, MaskOrigin, MaskClip, MaskComposite, MaskMode], [MaskBorderSource, MaskBorderSlice, MaskBorderWidth, MaskBorderOutset, MaskBorderRepeat, MaskBorderMode]),
        catalog: complete_grammar_catalog!("baseline.property.mask", "url(mask.png) center / contain no-repeat padding-box no-clip exclude alpha", rejected("margin-box")),
        source: "S-MASKING1",
        dispatch: "url(mask.png) center / contain no-repeat",
        wrapper: yes,
    }
    MaskImage, "mask-image" {
        metadata: longhand(false, |v| assert!(matches!(v.images(), [CssImageValue::None]))),
        catalog: complete_grammar_catalog!("baseline.property.mask-image", "url(mask.png), none", rejected("url(foo bar)")),
        source: "S-MASKING1",
        dispatch: "url(mask.png), none",
        wrapper: yes,
    }
    MaskPosition, "mask-position" {
        metadata: longhand(false, |v| {
            let [position] = v.positions() else { panic!("one initial position") };
            let CssHorizontalPosition::Offset(x) = position.horizontal() else { panic!("horizontal percentage") };
            let CssVerticalPosition::Offset(y) = position.vertical() else { panic!("vertical percentage") };
            assert_eq!(x.literal_component(), Some(&CssComponentValue::try_token("0%").unwrap()));
            assert_eq!(y.literal_component(), Some(&CssComponentValue::try_token("0%").unwrap()));
        }),
        catalog: complete_grammar_catalog!("baseline.property.mask-position", "center", rejected("left right")),
        source: "S-MASKING1",
        dispatch: "center",
        wrapper: yes,
    }
    MaskRepeat, "mask-repeat" {
        metadata: longhand(false, |v| assert!(matches!(v.repeats(), [CssBackgroundRepeat::Axes { x: CssBackgroundRepeatStyle::Repeat, y: CssBackgroundRepeatStyle::Repeat }]))),
        catalog: complete_grammar_catalog!("baseline.property.mask-repeat", "repeat", rejected("solid")),
        source: "S-MASKING1",
        dispatch: "repeat",
        wrapper: yes,
    }
    MaskSize, "mask-size" {
        metadata: longhand(false, |v| assert!(matches!(v.sizes(), [CssBackgroundSize::Explicit { width: CssBackgroundSizeComponent::Auto, height: None }]))),
        catalog: complete_grammar_catalog!("baseline.property.mask-size", "contain", rejected("solid")),
        source: "S-MASKING1",
        dispatch: "contain",
        wrapper: yes,
    }
    MaskMode, "mask-mode" {
        metadata: longhand(false, |v| assert_eq!(v.modes(), &[CssMaskMode::MatchSource])),
        catalog: complete_grammar_catalog!("official.property.mask-mode", "alpha, luminance, match-source", rejected("auto")),
        source: "S-MASKING1", dispatch: "alpha, luminance", wrapper: yes,
    }
    MaskOrigin, "mask-origin" {
        metadata: longhand(false, |v| assert_eq!(v.boxes(), &[CssMaskBox::BorderBox])),
        catalog: complete_grammar_catalog!("official.property.mask-origin", "padding-box, fill-box", rejected("margin-box")),
        source: "S-MASKING1", dispatch: "padding-box, fill-box", wrapper: yes,
    }
    MaskClip, "mask-clip" {
        metadata: longhand(false, |v| assert_eq!(v.clips(), &[CssMaskClip::Box(CssMaskBox::BorderBox)])),
        catalog: complete_grammar_catalog!("official.property.mask-clip", "no-clip, stroke-box", rejected("margin-box")),
        source: "S-MASKING1", dispatch: "no-clip, stroke-box", wrapper: yes,
    }
    MaskComposite, "mask-composite" {
        metadata: longhand(false, |v| assert_eq!(v.operators(), &[CssMaskComposite::Add])),
        catalog: complete_grammar_catalog!("official.property.mask-composite", "subtract, intersect, exclude", rejected("xor")),
        source: "S-MASKING1", dispatch: "subtract, intersect, exclude", wrapper: yes,
    }
    MaskBorder, "mask-border" {
        metadata: shorthand([MaskBorderSource, MaskBorderSlice, MaskBorderWidth, MaskBorderOutset, MaskBorderRepeat, MaskBorderMode], []),
        catalog: complete_grammar_catalog!("official.property.mask-border", "url(a) 1 2 fill / auto / 2 round alpha", rejected("fill 1")),
        source: "S-MASKING1", dispatch: "url(a) 1 2 fill / auto / 2 round alpha", wrapper: yes,
    }
    MaskBorderSource, "mask-border-source" {
        metadata: longhand(false, |v| assert!(matches!(v, CssImageValue::None))),
        catalog: complete_grammar_catalog!("official.property.mask-border-source", "url(a)", rejected("none, url(a)")),
        source: "S-MASKING1", dispatch: "url(a)", wrapper: yes,
    }
    MaskBorderSlice, "mask-border-slice" {
        metadata: longhand(false, |v| {
            assert!(!v.fill());
            for edge in v.values() {
                let CssBorderImageSliceComponent::Number(number) = edge else { panic!("number initial slice") };
                assert_eq!(number.literal_component(), Some(&CssComponentValue::try_number("0").unwrap()));
            }
        }),
        catalog: complete_grammar_catalog!("official.property.mask-border-slice", "1 2% fill", rejected("fill 1")),
        source: "S-MASKING1", dispatch: "1 2% fill", wrapper: yes,
    }
    MaskBorderWidth, "mask-border-width" {
        metadata: longhand(false, |v| assert!(v.values().iter().all(|edge| matches!(edge, CssBorderImageWidthComponent::Auto)))),
        catalog: complete_grammar_catalog!("official.property.mask-border-width", "auto 2 30% 4px", rejected("-1%")),
        source: "S-MASKING1", dispatch: "auto 2 30% 4px", wrapper: yes,
    }
    MaskBorderOutset, "mask-border-outset" {
        metadata: longhand(false, |v| {
            for edge in v.values() {
                let CssBorderImageOutsetComponent::Number(number) = edge else { panic!("number initial outset") };
                assert_eq!(number.literal_component(), Some(&CssComponentValue::try_number("0").unwrap()));
            }
        }),
        catalog: complete_grammar_catalog!("official.property.mask-border-outset", "1 2px 3 4em", rejected("1%")),
        source: "S-MASKING1", dispatch: "1 2px 3 4em", wrapper: yes,
    }
    MaskBorderRepeat, "mask-border-repeat" {
        metadata: longhand(false, |v| {
            assert_eq!(v.horizontal(), CssBorderImageRepeatKeyword::Stretch);
            assert_eq!(v.vertical(), CssBorderImageRepeatKeyword::Stretch);
        }),
        catalog: complete_grammar_catalog!("official.property.mask-border-repeat", "round space", rejected("no-repeat")),
        source: "S-MASKING1", dispatch: "round space", wrapper: yes,
    }
    MaskBorderMode, "mask-border-mode" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMaskType::Alpha)),
        catalog: complete_grammar_catalog!("official.property.mask-border-mode", "luminance", rejected("match-source")),
        source: "S-MASKING1", dispatch: "luminance", wrapper: yes,
    }
    ClipRule, "clip-rule" {
        metadata: longhand(true, |v| assert_eq!(*v, CssClipRule::Nonzero)),
        catalog: complete_grammar_catalog!("official.property.clip-rule", "evenodd", rejected("positive")),
        source: "S-MASKING1", dispatch: "evenodd", wrapper: yes,
    }
    MaskType, "mask-type" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMaskType::Luminance)),
        catalog: complete_grammar_catalog!("official.property.mask-type", "alpha", rejected("match-source")),
        source: "S-MASKING1", dispatch: "alpha", wrapper: yes,
    }
    MaxBlockSize, "max-block-size" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMaxSizeValue::NONE)),
    }
    MaxHeight, "max-height" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMaxSizeValue::NONE)),
        catalog: grammar_catalog!("baseline.property.max-height", "fit-content", rejected("solid")),
        dispatch: "fit-content",
        wrapper: yes,
    }
    MaxInlineSize, "max-inline-size" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMaxSizeValue::NONE)),
    }
    MaxSize, "max-size" {
        metadata: shorthand([MaxWidth, MaxHeight], []),
    }
    MaxWidth, "max-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMaxSizeValue::NONE)),
        catalog: grammar_catalog!("baseline.property.max-width", "max-content", rejected("solid")),
        dispatch: "max-content",
        wrapper: yes,
    }
    MinBlockSize, "min-block-size" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
    }
    MinHeight, "min-height" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
        catalog: grammar_catalog!("baseline.property.min-height", "min-content", rejected("solid")),
        dispatch: "min-content",
        wrapper: yes,
    }
    MinInlineSize, "min-inline-size" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
    }
    MinIntrinsicSizing, "min-intrinsic-sizing" {
        metadata: longhand(false, |v| assert_eq!(*v, CssMinIntrinsicSizing::Legacy)),
    }
    MinSize, "min-size" {
        metadata: shorthand([MinWidth, MinHeight], []),
    }
    MinWidth, "min-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
        catalog: grammar_catalog!("baseline.property.min-width", "0", rejected("solid")),
        dispatch: "0",
        wrapper: yes,
    }
    MixBlendMode, "mix-blend-mode" {
        metadata: longhand(false, |v| assert_eq!(*v, CssBlendMode::Normal)),
        catalog: grammar_catalog!("official.property.mix-blend-mode", "soft-light", rejected("multiply, screen")),
    }
    ObjectFit, "object-fit" {
        metadata: longhand(false, |v| assert_eq!(*v, CssObjectFit::Fill)),
        catalog: grammar_catalog!("official.property.object-fit", "scale-down", rejected("cover contain"), "#propdef-object-fit"),
        source: "O-IMAGES3",
        dispatch: "scale-down",
        wrapper: yes,
    }
    ObjectPosition, "object-position" {
        metadata: longhand(false, |v| {
            let (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) = (v.horizontal(), v.vertical()) else {
                panic!("object-position initial has two free percentage offsets");
            };
            for offset in [x, y] {
                assert!(exact_literal(offset.literal_component(), "50%"));
                assert_eq!(offset.origin(), &CssValueOrigin::Programmatic);
            }
        }),
        catalog: grammar_catalog!("official.property.object-position", "bottom 2px right 5%", rejected("left 1px top"), "#propdef-object-position"),
        source: "O-IMAGES3",
        dispatch: "right 5% bottom 2px",
        wrapper: yes,
    }
    Opacity, "opacity" {
        metadata: longhand(false, |v| {
            assert!(
                matches!(v, CssOpacityValue::Scalar(value) if value.numeric().representation() == "1")
            );
        }),
        catalog: grammar_catalog!("baseline.property.opacity", "0.5", rejected("solid")),
        dispatch: "150%",
        wrapper: yes,
    }
    Order, "order" {
        metadata: longhand(false, |v| assert_eq!(v.serialize_specified().unwrap(), "0")),
        catalog: grammar_catalog!("baseline.property.order", "-2", rejected("1.5")),
        source: "S-DISPLAY3",
        aliases: ["-webkit-order"],
        dispatch: "-2",
        wrapper: yes,
    }
    Orphans, "orphans" {
        metadata: longhand(true, |v| {
            assert_eq!(v.literal(), Some(2))
        }),
        catalog: grammar_catalog!("official.property.orphans", "3", rejected("0")),
    }
    Outline, "outline" {
        metadata: shorthand([OutlineWidth, OutlineStyle, OutlineColor], []),
        catalog: grammar_catalog!("baseline.property.outline", "thick dotted white", rejected("solid dotted")),
        source: "X-UI4",
        dispatch: "thick dotted white",
        wrapper: yes,
    }
    OutlineColor, "outline-color" {
        metadata: longhand(false, |v| assert!(matches!(v, CssOutlineColor::Auto))),
        catalog: grammar_catalog!("baseline.property.outline-color", "black", rejected("black white")),
        source: "X-UI4",
        dispatch: "black",
        wrapper: yes,
    }
    OutlineOffset, "outline-offset" {
        metadata: longhand(false, |v| assert_eq!(v.serialize_specified().unwrap(), "0")),
        catalog: grammar_catalog!("official.property.outline-offset", "-2px", rejected("auto")),
        source: "X-UI4",
    }
    OutlineStyle, "outline-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssOutlineStyle::None)),
        catalog: grammar_catalog!("baseline.property.outline-style", "auto", rejected("10px")),
        source: "X-UI4",
        dispatch: "auto",
        wrapper: yes,
    }
    OutlineWidth, "outline-width" {
        metadata: longhand(false, |v| assert!(matches!(v, CssOutlineWidth::Medium))),
        catalog: grammar_catalog!("baseline.property.outline-width", "2px", rejected("10%")),
        source: "X-UI4",
        dispatch: "2px",
        wrapper: yes,
    }
    Overflow, "overflow" {
        metadata: shorthand([OverflowX, OverflowY], []),
        catalog: grammar_catalog!("baseline.property.overflow", "hidden scroll", accepted("auto", |known| {
                let CssKnownPropertyValueRef::Overflow(value) = known.property_value().expect("typed current auto") else {
                    panic!("overflow current auto has the wrong typed value");
                };
                assert_eq!(value.value().x(), CssOverflow::Auto);
                    assert_eq!(value.value().authored_y(), None);
                    assert_eq!(value.value().y(), CssOverflow::Auto);
            })),
        dispatch: "hidden scroll",
        wrapper: yes,
    }
    OverflowAnchor, "overflow-anchor" {
        metadata: longhand(false, |v| assert_eq!(*v, CssOverflowAnchor::Auto)),
        catalog: grammar_catalog!("ext.property.overflow-anchor", "none", rejected("smooth")),
        source: "I-SCROLLANCHORING1",
        dispatch: "none",
        wrapper: yes,
    }
    OverflowBlock, "overflow-block" {
        metadata: longhand(false, |v| assert_eq!(*v, CssOverflow::Visible)),
    }
    OverflowClipMargin, "overflow-clip-margin" {
        metadata: longhand(false, |v| {
            assert_eq!(v.authored_box_edge(), None);
            assert_eq!(v.box_edge(), CssBoxEdgeKeyword::PaddingBox);
            assert_eq!(
                v.authored_offset().unwrap().serialize_specified().unwrap(),
                "0px"
            );
        }),
    }
    OverflowInline, "overflow-inline" {
        metadata: longhand(false, |v| assert_eq!(*v, CssOverflow::Visible)),
    }
    OverflowWrap, "overflow-wrap" {
        metadata: longhand(true, |v| assert_eq!(v, &CssOverflowWrap::Normal)),
        catalog: grammar_catalog!("baseline.property.overflow-wrap", "anywhere", rejected("ellipsis")),
        source: "S-TEXT3",
        aliases: ["word-wrap"],
        dispatch: "anywhere",
        wrapper: yes,
    }
    OverflowX, "overflow-x" {
        metadata: longhand(false, |v| assert_eq!(*v, CssOverflow::Visible)),
        catalog: grammar_catalog!("baseline.property.overflow-x", "clip", accepted("auto", |known| {
                let CssKnownPropertyValueRef::OverflowX(value) = known.property_value().expect("typed current auto") else {
                    panic!("overflow-x current auto has the wrong typed value");
                };
                assert_eq!(*value.value(), CssOverflow::Auto);
            })),
        source: "X-OVERFLOW3",
        dispatch: "clip",
        wrapper: yes,
    }
    OverflowY, "overflow-y" {
        metadata: longhand(false, |v| assert_eq!(*v, CssOverflow::Visible)),
        catalog: grammar_catalog!("baseline.property.overflow-y", "visible", accepted("auto", |known| {
                let CssKnownPropertyValueRef::OverflowY(value) = known.property_value().expect("typed current auto") else {
                    panic!("overflow-y current auto has the wrong typed value");
                };
                assert_eq!(*value.value(), CssOverflow::Auto);
            })),
        dispatch: "visible",
        wrapper: yes,
    }
    Padding, "padding" {
        metadata: four_side(),
        catalog: grammar_catalog!("baseline.property.padding", "1px 2% calc(3px + 4%) 0", rejected("auto")),
        dispatch: "1px 2% calc(3px + 4%) 0",
        wrapper: yes,
    }
    PaddingBlock, "padding-block" {
        metadata: shorthand([PaddingBlockStart, PaddingBlockEnd], []),
    }
    PaddingBlockEnd, "padding-block-end" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
    }
    PaddingBlockStart, "padding-block-start" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
    }
    PaddingBottom, "padding-bottom" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.padding-bottom", "calc(3px + 4%)", rejected("auto")),
        dispatch: "calc(3px + 4%)",
        wrapper: yes,
    }
    PaddingInline, "padding-inline" {
        metadata: shorthand([PaddingInlineStart, PaddingInlineEnd], []),
    }
    PaddingInlineEnd, "padding-inline-end" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
    }
    PaddingInlineStart, "padding-inline-start" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
    }
    PaddingLeft, "padding-left" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.padding-left", "0", rejected("auto")),
        dispatch: "0",
        wrapper: yes,
    }
    PaddingRight, "padding-right" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.padding-right", "2%", rejected("auto")),
        dispatch: "2%",
        wrapper: yes,
    }
    PaddingTop, "padding-top" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssPaddingValue::new(CssSpecifiedNonNegativeLengthPercentage::zero())
        )),
        catalog: grammar_catalog!("baseline.property.padding-top", "12px", rejected("auto")),
        dispatch: "12px",
        wrapper: yes,
    }
    Pause, "pause" {
        metadata: shorthand([PauseBefore, PauseAfter], []),
        wrapper: yes,
    }
    PauseAfter, "pause-after" {
        metadata: longhand(false, |value| assert_eq!(*value, CssSpeechBreak::None)),
        wrapper: yes,
    }
    PauseBefore, "pause-before" {
        metadata: longhand(false, |value| assert_eq!(*value, CssSpeechBreak::None)),
        wrapper: yes,
    }
    Perspective, "perspective" {
        metadata: longhand(false, |v| assert_eq!(*v, CssPerspective::None)),
        catalog: grammar_catalog!("official.property.perspective", ".25px", rejected("-1e-999px")),
        source: "I-TRANSFORMS2",
        dispatch: "calc(1px - 2px)",
        wrapper: yes,
    }
    PerspectiveOrigin, "perspective-origin" {
        metadata: longhand(false, |v| {
            let (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) = (v.horizontal(), v.vertical()) else { panic!("initial physical free offsets") };
            for offset in [x, y] {
                assert!(exact_literal(offset.literal_component(), "50%"));
                assert_eq!(offset.origin(), &CssValueOrigin::Programmatic);
            }
        }),
        catalog: grammar_catalog!("official.property.perspective-origin", "right 2px bottom 3%", rejected("left 2px top")),
        source: "I-TRANSFORMS2",
        dispatch: "right 2px bottom 3%",
        wrapper: yes,
    }
    PlaceContent, "place-content" {
        metadata: shorthand([AlignContent, JustifyContent], []),
        catalog: grammar_catalog!("baseline.property.place-content", "center end", rejected("auto")),
        dispatch: "center end",
        wrapper: yes,
    }
    PlaceItems, "place-items" {
        metadata: shorthand([AlignItems, JustifyItems], []),
        catalog: grammar_catalog!("baseline.property.place-items", "stretch", rejected("space-between")),
        dispatch: "stretch",
        wrapper: yes,
    }
    PlaceSelf, "place-self" {
        metadata: shorthand([AlignSelf, JustifySelf], []),
        catalog: grammar_catalog!("baseline.property.place-self", "end center", rejected("space-between")),
        dispatch: "end center",
        wrapper: yes,
    }
    PointerEvents, "pointer-events" {
        metadata: longhand(true, |v| assert_eq!(*v, CssPointerEvents::Auto)),
        catalog: grammar_catalog!("baseline.property.pointer-events", "none", rejected("grab")),
        source: "X-UI4",
        dispatch: "none",
        wrapper: yes,
    }
    Position, "position" {
        metadata: longhand(false, |v| assert_eq!(*v, CssLayoutPosition::Static)),
        catalog: grammar_catalog!("baseline.property.position", "sticky", rejected("running")),
        dispatch: "sticky",
        wrapper: yes,
    }
    PrintColorAdjust, "print-color-adjust" {
        metadata: longhand(true, |v| assert_eq!(*v, CssPrintColorAdjust::Economy)),
        source: "R-COLORADJUST1",
        dispatch: "exact",
        wrapper: yes,
    }
    Quotes, "quotes" {
        metadata: longhand(true, |v| assert_eq!(v, &CssQuotes::Auto)),
        catalog: grammar_catalog!("official.property.quotes", "\"open\" \"close\"", rejected("\"open\"")),
        source: "X-CONTENT3",
    }
    Resize, "resize" {
        metadata: longhand(false, |v| assert_eq!(*v, CssResize::None)),
        catalog: grammar_catalog!("official.property.resize", "block", accepted("inline", |known| {
            let CssKnownPropertyValueRef::Resize(value) = known.property_value().unwrap() else {
                panic!("typed logical resize")
            };
            assert_eq!(*value.resize(), CssResize::Inline);
        })),
        source: "X-UI4",
        dispatch: "block",
        wrapper: yes,
    }
    Rest, "rest" {
        metadata: shorthand([RestBefore, RestAfter], []),
        wrapper: yes,
    }
    RestAfter, "rest-after" {
        metadata: longhand(false, |value| assert_eq!(*value, CssSpeechBreak::None)),
        wrapper: yes,
    }
    RestBefore, "rest-before" {
        metadata: longhand(false, |value| assert_eq!(*value, CssSpeechBreak::None)),
        wrapper: yes,
    }
    Right, "right" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
        catalog: grammar_catalog!("baseline.property.right", "10px", rejected("solid")),
        dispatch: "10px",
        wrapper: yes,
    }
    Rotate, "rotate" {
        metadata: longhand(false, |v| assert_eq!(*v, CssRotate::None)),
        catalog: grammar_catalog!("baseline.property.rotate", "45deg", rejected("45px")),
        source: "I-TRANSFORMS2",
        dispatch: "45deg",
        wrapper: yes,
    }
    RowGap, "row-gap" {
        metadata: longhand(false, |v| {
            assert_eq!(v, &CssGapValue::Normal)
        }),
        catalog: grammar_catalog!("baseline.property.row-gap", "normal", rejected("auto")),
        source: "S-ALIGN3",
        aliases: ["grid-row-gap"],
        dispatch: "normal",
        wrapper: yes,
    }
    Scale, "scale" {
        metadata: longhand(false, |v| assert_eq!(*v, CssScale::None)),
        catalog: grammar_catalog!("baseline.property.scale", "50% 2", rejected("solid")),
        source: "I-TRANSFORMS2",
        dispatch: "1.5 2",
        wrapper: yes,
    }
    ScrollBehavior, "scroll-behavior" {
        metadata: longhand(false, |v| assert_eq!(*v, CssScrollBehavior::Auto)),
    }
    ScrollMargin, "scroll-margin" {
        metadata: four_side(),
    }
    ScrollMarginBlock, "scroll-margin-block" {
        metadata: shorthand([ScrollMarginBlockStart, ScrollMarginBlockEnd], []),
    }
    ScrollMarginBlockEnd, "scroll-margin-block-end" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginBlockStart, "scroll-margin-block-start" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginBottom, "scroll-margin-bottom" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginInline, "scroll-margin-inline" {
        metadata: shorthand([ScrollMarginInlineStart, ScrollMarginInlineEnd], []),
    }
    ScrollMarginInlineEnd, "scroll-margin-inline-end" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginInlineStart, "scroll-margin-inline-start" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginLeft, "scroll-margin-left" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginRight, "scroll-margin-right" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollMarginTop, "scroll-margin-top" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssSpecifiedLength::zero())
        }),
    }
    ScrollPadding, "scroll-padding" {
        metadata: four_side(),
    }
    ScrollPaddingBlock, "scroll-padding-block" {
        metadata: shorthand([ScrollPaddingBlockStart, ScrollPaddingBlockEnd], []),
    }
    ScrollPaddingBlockEnd, "scroll-padding-block-end" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingBlockStart, "scroll-padding-block-start" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingBottom, "scroll-padding-bottom" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingInline, "scroll-padding-inline" {
        metadata: shorthand([ScrollPaddingInlineStart, ScrollPaddingInlineEnd], []),
    }
    ScrollPaddingInlineEnd, "scroll-padding-inline-end" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingInlineStart, "scroll-padding-inline-start" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingLeft, "scroll-padding-left" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingRight, "scroll-padding-right" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollPaddingTop, "scroll-padding-top" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssScrollPaddingValue::Auto)
        }),
    }
    ScrollSnapAlign, "scroll-snap-align" {
        metadata: longhand(false, |v| assert_eq!(
            *v,
            CssScrollSnapAlign::new(CssScrollSnapAlignment::None, None)
        )),
    }
    ScrollSnapStop, "scroll-snap-stop" {
        metadata: longhand(false, |v| assert_eq!(*v, CssScrollSnapStop::Normal)),
    }
    ScrollSnapType, "scroll-snap-type" {
        metadata: longhand(false, |v| assert_eq!(*v, CssScrollSnapType::None)),
    }
    ScrollbarColor, "scrollbar-color" {
        metadata: longhand(true, |v| assert_eq!(*v, CssScrollbarColor::auto())),
        source: "R-SCROLLBARS1",
        dispatch: "red blue",
        wrapper: yes,
    }
    ScrollbarGutter, "scrollbar-gutter" {
        metadata: longhand(false, |v| assert_eq!(*v, CssScrollbarGutter::Auto)),
    }
    ScrollbarWidth, "scrollbar-width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssScrollbarWidth::Auto)),
        catalog: grammar_catalog!("baseline.property.scrollbar-width", "thin", rejected("solid")),
        source: "R-SCROLLBARS1",
        dispatch: "thin",
        wrapper: yes,
    }
    Size, "size" {
        metadata: shorthand([Width, Height], []),
    }
    Speak, "speak" {
        metadata: longhand(true, |value| assert_eq!(*value, CssSpeak::Auto)),
        dispatch: "always",
        wrapper: yes,
    }
    SpeakAs, "speak-as" {
        metadata: longhand(true, |value| assert_eq!(value, &CssSpeakAs::normal())),
        dispatch: "digits spell-out no-punctuation",
        wrapper: yes,
    }
    TabSize, "tab-size" {
        metadata: longhand(true, |v| {
            let CssTabSize::Number(number) = v else {
                panic!("number tab initial")
            };
            assert!(exact_literal(number.literal_component(), "8"));
        }),
        catalog: grammar_catalog!("ext.property.tab-size", "calc(2 + 3)", rejected("-1e-999")),
        source: "X-TEXT4",
        wrapper: yes,
    }
    TableLayout, "table-layout" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTableLayout::Auto)),
        catalog: grammar_catalog!("official.property.table-layout", "fixed", rejected("collapse")),
    }
    TextAlign, "text-align" {
        metadata: shorthand([TextAlignAll, TextAlignLast], []),
        catalog: grammar_catalog!("baseline.property.text-align", "start", rejected("auto")),
        dispatch: "start",
        wrapper: yes,
    }
    TextAlignAll, "text-align-all" {
        metadata: longhand(true, |v| {
            assert_eq!(*v, CssTextAlignAllValue::Keyword(CssTextAlign::Start))
        }),
    }
    TextAlignLast, "text-align-last" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextAlignLastValue::Auto)),
        catalog: grammar_catalog!("baseline.property.text-align-last", "justify", rejected("justify-all")),
        dispatch: "justify",
        wrapper: yes,
    }
    TextAutospace, "text-autospace" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextAutospace::Normal)),
        catalog: grammar_catalog!("ext.property.text-autospace", "replace punctuation ideograph-alpha", rejected("ideograph-alpha insert punctuation")),
        source: "X-TEXT4",
        dispatch: "replace punctuation ideograph-alpha",
        wrapper: yes,
    }
    TextCombineUpright, "text-combine-upright" {
        metadata: longhand(true, |v| assert_eq!(v, &CssTextCombineUpright::None)),
        catalog: grammar_catalog!("official.property.text-combine-upright", "all", rejected("sideways")),
        source: "S-WRITING4",
    }
    TextDecoration, "text-decoration" {
        metadata: shorthand([TextDecorationLine, TextDecorationThickness, TextDecorationStyle, TextDecorationColor], []),
        catalog: grammar_catalog!("baseline.property.text-decoration", "underline dotted white 3px", rejected("underline underline")),
        source: "X-TEXTDECOR4",
        dispatch: "underline dotted white 3px",
        wrapper: yes,
    }
    TextDecorationColor, "text-decoration-color" {
        metadata: longhand(false, |v| assert!(v.is_current_color())),
        catalog: grammar_catalog!("baseline.property.text-decoration-color", "black", rejected("black white")),
        source: "X-TEXTDECOR4",
        dispatch: "black",
        wrapper: yes,
    }
    TextDecorationLine, "text-decoration-line" {
        metadata: longhand(false, |v| { assert!(v.is_none()); assert!(v.components().is_empty()); assert_eq!(v.error_kind(), None); }),
        catalog: grammar_catalog!("baseline.property.text-decoration-line", "spelling-error", rejected("spelling-error underline")),
        source: "X-TEXTDECOR4",
        dispatch: "underline overline",
        wrapper: yes,
    }
    TextDecorationStyle, "text-decoration-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTextDecorationStyle::Solid)),
        catalog: grammar_catalog!("baseline.property.text-decoration-style", "wavy", rejected("auto")),
        source: "X-TEXTDECOR4",
        dispatch: "wavy",
        wrapper: yes,
    }
    TextDecorationThickness, "text-decoration-thickness" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTextDecorationThickness::Auto)),
        catalog: grammar_catalog!("baseline.property.text-decoration-thickness", "-1px", rejected("1")),
        source: "X-TEXTDECOR4",
        dispatch: "2px",
        wrapper: yes,
    }
    TextDecorationSkip, "text-decoration-skip" {
        metadata: shorthand([TextDecorationSkipSelf, TextDecorationSkipBox, TextDecorationSkipInset, TextDecorationSkipSpaces, TextDecorationSkipInk], []),
        catalog: grammar_catalog!("ext.property.text-decoration-skip", "auto", rejected("objects")),
        source: "X-TEXTDECOR4",
        dispatch: "none",
        wrapper: yes,
    }
    TextDecorationSkipSelf, "text-decoration-skip-self" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextDecorationSkipSelf::Objects)),
        catalog: grammar_catalog!("ext.property.text-decoration-skip-self", "objects", rejected("auto")),
        source: "X-TEXTDECOR4",
        dispatch: "none",
        wrapper: yes,
    }
    TextDecorationSkipBox, "text-decoration-skip-box" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextDecorationSkipBox::None)),
        catalog: grammar_catalog!("ext.property.text-decoration-skip-box", "all", rejected("objects")),
        source: "X-TEXTDECOR4",
        dispatch: "all",
        wrapper: yes,
    }
    TextDecorationSkipInset, "text-decoration-skip-inset" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextDecorationSkipInset::None)),
        catalog: grammar_catalog!("ext.property.text-decoration-skip-inset", "auto", rejected("all")),
        source: "X-TEXTDECOR4",
        dispatch: "auto",
        wrapper: yes,
    }
    TextDecorationSkipSpaces, "text-decoration-skip-spaces" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextDecorationSkipSpaces::StartEnd)),
        catalog: grammar_catalog!("ext.property.text-decoration-skip-spaces", "end start", rejected("start start")),
        source: "X-TEXTDECOR4",
        dispatch: "end start",
        wrapper: yes,
    }
    TextDecorationSkipInk, "text-decoration-skip-ink" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextDecorationSkipInk::Auto)),
        catalog: grammar_catalog!("ext.property.text-decoration-skip-ink", "all", rejected("objects")),
        source: "X-TEXTDECOR4",
        dispatch: "all",
        wrapper: yes,
    }
    TextEmphasis, "text-emphasis" {
        metadata: shorthand([TextEmphasisStyle, TextEmphasisColor], []),
        catalog: grammar_catalog!("ext.property.text-emphasis", "red open triangle", rejected("red blue")),
        source: "X-TEXTDECOR4",
        dispatch: "red open triangle",
        wrapper: yes,
    }
    TextEmphasisStyle, "text-emphasis-style" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextEmphasisStyle::None)),
        catalog: grammar_catalog!("ext.property.text-emphasis-style", "'hello'", rejected("dot circle")),
        source: "X-TEXTDECOR4",
        dispatch: "filled dot",
        wrapper: yes,
    }
    TextEmphasisColor, "text-emphasis-color" {
        metadata: longhand(true, |v| assert!(v.is_current_color())),
        catalog: grammar_catalog!("ext.property.text-emphasis-color", "light-dark(red,blue)", rejected("red blue")),
        source: "X-TEXTDECOR4",
        dispatch: "light-dark(red,blue)",
        wrapper: yes,
    }
    TextEmphasisPosition, "text-emphasis-position" {
        metadata: longhand(true, |v| { assert_eq!(v.vertical(), CssTextEmphasisVertical::Over); assert_eq!(v.side(), Some(CssTextSide::Right)); assert_eq!(v.effective_side(), CssTextSide::Right); }),
        catalog: grammar_catalog!("ext.property.text-emphasis-position", "left under", rejected("right")),
        source: "X-TEXTDECOR4",
        dispatch: "left under",
        wrapper: yes,
    }
    TextEmphasisSkip, "text-emphasis-skip" {
        metadata: longhand(true, |v| { assert!(v.spaces()); assert!(v.punctuation()); assert!(!v.symbols()); assert!(!v.narrow()); }),
        catalog: grammar_catalog!("ext.property.text-emphasis-skip", "narrow symbols spaces", rejected("spaces spaces")),
        source: "X-TEXTDECOR4",
        dispatch: "narrow symbols spaces",
        wrapper: yes,
    }
    TextShadow, "text-shadow" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextShadow::None)),
        catalog: grammar_catalog!("ext.property.text-shadow", "inset red 1px 2px 3px 4px", rejected("1px 2px 0px -1px")),
        source: "X-TEXTDECOR4",
        dispatch: "inset red 1px 2px 3px 4px",
        wrapper: yes,
    }
    TextUnderlinePosition, "text-underline-position" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextUnderlinePosition::Auto)),
        catalog: grammar_catalog!("ext.property.text-underline-position", "left from-font", rejected("auto left")),
        source: "X-TEXTDECOR4",
        dispatch: "left from-font",
        wrapper: yes,
    }
    TextUnderlineOffset, "text-underline-offset" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextUnderlineOffset::Auto)),
        catalog: grammar_catalog!("ext.property.text-underline-offset", "-25%", rejected("from-font")),
        source: "X-TEXTDECOR4",
        dispatch: "-25%",
        wrapper: yes,
    }
    TextGroupAlign, "text-group-align" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTextGroupAlign::None)),
        catalog: grammar_catalog!("ext.property.text-group-align", "center", rejected("justify")),
        source: "X-TEXT4",
        dispatch: "center",
        wrapper: yes,
    }
    TextIndent, "text-indent" {
        metadata: longhand(true, |v| {
            assert!(exact_literal(v.length().literal_component(), "0"));
            assert!(!v.hanging());
            assert!(!v.each_line());
        }),
        catalog: grammar_catalog!("baseline.property.text-indent", "1rem hanging each-line", rejected("auto")),
        source: "X-TEXT4",
        dispatch: "1rem hanging each-line",
        wrapper: yes,
    }
    TextJustify, "text-justify" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextJustify::auto())),
        catalog: grammar_catalog!("ext.property.text-justify", "no-compress distribute", rejected("ruby auto")),
        source: "X-TEXT4",
        dispatch: "no-compress distribute",
        wrapper: yes,
    }
    TextOrientation, "text-orientation" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextOrientation::Mixed)),
        catalog: grammar_catalog!("official.property.text-orientation", "sideways", rejected("auto")),
    }
    TextOverflow, "text-overflow" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTextOverflow::Clip)),
        catalog: grammar_catalog!("baseline.property.text-overflow", "ellipsis", rejected("wrap")),
        dispatch: "ellipsis",
        wrapper: yes,
    }
    TextSpacing, "text-spacing" {
        metadata: shorthand([TextSpacingTrim, TextAutospace], []),
        catalog: grammar_catalog!("ext.property.text-spacing", "replace punctuation ideograph-alpha trim-both", rejected("ideograph-alpha trim-start punctuation")),
        source: "X-TEXT4",
        dispatch: "replace punctuation ideograph-alpha trim-both",
        wrapper: yes,
    }
    TextSpacingTrim, "text-spacing-trim" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextSpacingTrim::Trim(CssSpacingTrim::Normal))),
        catalog: grammar_catalog!("ext.property.text-spacing-trim", "auto", rejected("trim-start trim-both")),
        source: "X-TEXT4",
        dispatch: "auto",
        wrapper: yes,
    }
    TextTransform, "text-transform" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextTransform::None)),
        catalog: grammar_catalog!("baseline.property.text-transform", "uppercase", rejected("wrap")),
        source: "X-TEXT4",
        dispatch: "uppercase",
        wrapper: yes,
    }
    TextWrap, "text-wrap" {
        metadata: shorthand([TextWrapMode, TextWrapStyle], []),
        catalog: grammar_catalog!("baseline.property.text-wrap", "balance", rejected("nowrap wrap")),
        source: "X-TEXT4",
        dispatch: "balance",
        wrapper: yes,
    }
    TextWrapMode, "text-wrap-mode" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextWrapMode::Wrap)),
        catalog: grammar_catalog!("ext.property.text-wrap-mode", "nowrap", rejected("balance")),
        source: "X-TEXT4",
        dispatch: "nowrap",
        wrapper: yes,
    }
    TextWrapStyle, "text-wrap-style" {
        metadata: longhand(true, |v| assert_eq!(*v, CssTextWrapStyle::Auto)),
        catalog: grammar_catalog!("ext.property.text-wrap-style", "avoid-short-last-line", rejected("nowrap")),
        source: "X-TEXT4",
        dispatch: "avoid-short-last-line",
        wrapper: yes,
    }
    Top, "top" {
        metadata: longhand(false, |v| assert_eq!(*v, CssInsetValue::Auto)),
        catalog: grammar_catalog!("baseline.property.top", "auto", rejected("solid")),
        dispatch: "auto",
        wrapper: yes,
    }
    Transform, "transform" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTransform::None)),
        catalog: grammar_catalog!("baseline.property.transform", "translate(10px, 20px) rotate(45deg) scale(1.5)", rejected("translate(red)")),
        source: "O-TRANSFORMS1",
        dispatch: "translate(10px, 20px) rotate(45deg) scale(1.5)",
        wrapper: yes,
    }
    TransformBox, "transform-box" {
        metadata: longhand(false, |v| assert_eq!(v.edge(), CssBoxEdgeKeyword::ViewBox)),
        catalog: grammar_catalog!("official.property.transform-box", "view-box", rejected("padding-box")),
        source: "O-TRANSFORMS1",
        dispatch: "stroke-box",
        wrapper: yes,
    }
    TransformOrigin, "transform-origin" {
        metadata: longhand(false, |v| {
            let (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) = (v.horizontal(), v.vertical()) else { panic!("initial physical free offsets") };
            for offset in [x, y] {
                assert!(exact_literal(offset.literal_component(), "50%"));
                assert_eq!(offset.origin(), &CssValueOrigin::Programmatic);
            }
            assert!(v.z().is_none());
        }),
        catalog: grammar_catalog!("baseline.property.transform-origin", "center top", rejected("left right")),
        source: "O-TRANSFORMS1",
        dispatch: "center top",
        wrapper: yes,
    }
    TransformStyle, "transform-style" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTransformStyle::Flat)),
        catalog: grammar_catalog!("official.property.transform-style", "preserve-3d", rejected("auto")),
        source: "I-TRANSFORMS2",
        dispatch: "preserve-3d",
        wrapper: yes,
    }
    Transition, "transition" {
        metadata: shorthand([TransitionProperty, TransitionDuration, TransitionTimingFunction, TransitionDelay], []),
        catalog: grammar_catalog!("baseline.property.transition", "opacity 150ms ease-in 20ms, transform 2s linear", rejected("opacity 1s 2s 3s")),
        source: "I-TRANSITIONS1",
        dispatch: "opacity 150ms ease-in 20ms, transform 2s linear",
        wrapper: yes,
    }
    TransitionDelay, "transition-delay" {
        metadata: longhand(false, |v| {
            assert_eq!(v.values().len(), 1);
            let literal = v.values()[0].literal().unwrap();
            assert_eq!(literal.numeric().representation(), "0");
            assert_eq!(literal.unit(), CssTimeUnit::Seconds);
        }),
        catalog: grammar_catalog!("baseline.property.transition-delay", "20ms", rejected("10px")),
        dispatch: "20ms",
        wrapper: yes,
    }
    TransitionDuration, "transition-duration" {
        metadata: longhand(false, |v| {
            assert_eq!(v.values().len(), 1);
            let literal = v.values()[0].time().literal().unwrap();
            assert_eq!(literal.numeric().representation(), "0");
            assert_eq!(literal.unit(), CssTimeUnit::Seconds);
        }),
        catalog: grammar_catalog!("baseline.property.transition-duration", "150ms, 2s", rejected("10px")),
        dispatch: "150ms, 2s",
        wrapper: yes,
    }
    TransitionProperty, "transition-property" {
        metadata: longhand(false, |v| {
            assert_eq!(v.properties(), &[CssTransitionProperty::All])
        }),
        catalog: grammar_catalog!("baseline.property.transition-property", "opacity, transform", rejected("default")),
        dispatch: "opacity, transform",
        wrapper: yes,
    }
    TransitionTimingFunction, "transition-timing-function" {
        metadata: longhand(false, |v| {
            assert_eq!(v.values(), &[CssEasing::Keyword(CssEasingKeyword::Ease)])
        }),
        catalog: grammar_catalog!("baseline.property.transition-timing-function", "ease-in, cubic-bezier(0.1, 0.2, 0.3, 1)", rejected("bounce")),
        dispatch: "ease-in, cubic-bezier(0.1, 0.2, 0.3, 1)",
        wrapper: yes,
    }
    Translate, "translate" {
        metadata: longhand(false, |v| assert_eq!(*v, CssTranslate::None)),
        catalog: grammar_catalog!("baseline.property.translate", "10px 20px", rejected("red")),
        source: "I-TRANSFORMS2",
        dispatch: "10px 20px",
        wrapper: yes,
    }
    UnicodeBidi, "unicode-bidi" {
        metadata: longhand(false, |v| assert_eq!(v, &CssUnicodeBidi::Normal)),
        catalog: grammar_catalog!("official.property.unicode-bidi", "isolate-override", rejected("isolate isolate")),
    }
    UserSelect, "user-select" {
        metadata: longhand(false, |v| assert_eq!(*v, CssUserSelect::Auto)),
        catalog: grammar_catalog!("baseline.property.user-select", "text", rejected("grab")),
        source: "X-UI4",
        aliases: ["-webkit-user-select"],
        dispatch: "text",
        wrapper: yes,
    }
    VerticalAlign, "vertical-align" {
        metadata: longhand(false, |v| assert_eq!(*v, CssVerticalAlign::Baseline)),
        catalog: grammar_catalog!("baseline.property.vertical-align", "super", rejected("auto")),
        source: "O-CSS2",
        dispatch: "super",
        wrapper: yes,
    }
    Visibility, "visibility" {
        metadata: longhand(true, |v| assert_eq!(v, &CssVisibility::Visible)),
        catalog: grammar_catalog!("baseline.property.visibility", "collapse", rejected("auto")),
        source: "S-DISPLAY3",
        dispatch: "collapse",
        wrapper: yes,
    }
    VoiceBalance, "voice-balance" {
        metadata: longhand(true, |value| {
            assert_eq!(value.keyword(), Some(CssVoiceBalanceKeyword::Center));
            assert!(value.number().is_none());
        }),
    }
    VoiceDuration, "voice-duration" {
        metadata: longhand(false, |value| {
            assert!(matches!(value, CssVoiceDuration::Auto))
        }),
        wrapper: yes,
    }
    VoiceFamily, "voice-family" {
        metadata: longhand(true, ua(CssUserAgentInitial::VoiceFamily)),
        wrapper: yes,
    }
    VoicePitch, "voice-pitch" {
        metadata: longhand(true, |value| {
            assert_eq!(
                value,
                &CssVoicePitchRange::level_only(CssVoiceLevel::Medium)
            )
        }),
        wrapper: yes,
    }
    VoiceRange, "voice-range" {
        metadata: longhand(true, |value| {
            assert_eq!(
                value,
                &CssVoicePitchRange::level_only(CssVoiceLevel::Medium)
            )
        }),
        wrapper: yes,
    }
    VoiceRate, "voice-rate" {
        metadata: longhand(true, |value| {
            assert_eq!(value.keyword(), Some(CssVoiceRateKeyword::Normal));
            assert!(value.percentage().is_none());
        }),
        wrapper: yes,
    }
    VoiceStress, "voice-stress" {
        metadata: longhand(true, |value| assert_eq!(*value, CssVoiceStress::Normal)),
        wrapper: yes,
    }
    VoiceVolume, "voice-volume" {
        metadata: longhand(true, |value| assert!(matches!(
            value,
            CssVoiceVolume::Level {
                level: CssVoiceVolumeLevel::Medium,
                decibel: None
            }
        ))),
    }
    WhiteSpace, "white-space" {
        metadata: shorthand([WhiteSpaceCollapse, TextWrapMode, WhiteSpaceTrim], []),
        catalog: grammar_catalog!("baseline.property.white-space", "pre-wrap", rejected("balance")),
        source: "X-TEXT4",
        dispatch: "pre-wrap",
        wrapper: yes,
    }
    WhiteSpaceCollapse, "white-space-collapse" {
        metadata: longhand(true, |v| {
            assert_eq!(*v, CssWhiteSpaceCollapse::Collapse)
        }),
        catalog: grammar_catalog!("ext.property.white-space-collapse", "discard", rejected("pre")),
        source: "X-TEXT4",
        dispatch: "discard",
        wrapper: yes,
    }
    WhiteSpaceTrim, "white-space-trim" {
        metadata: longhand(false, |v| assert_eq!(*v, CssWhiteSpaceTrim::none())),
        catalog: grammar_catalog!("ext.property.white-space-trim", "discard-after discard-before", rejected("discard-before discard-before")),
        source: "X-TEXT4",
        dispatch: "discard-inner discard-before",
        wrapper: yes,
    }
    Widows, "widows" {
        metadata: longhand(true, |v| {
            assert_eq!(v.literal(), Some(2))
        }),
        catalog: grammar_catalog!("official.property.widows", "4", rejected("0")),
    }
    Width, "width" {
        metadata: longhand(false, |v| assert_eq!(*v, CssSizeValue::Auto)),
        catalog: grammar_catalog!("baseline.property.width", "calc(100% - 12px)", rejected("solid")),
        dispatch: "calc(100% - 12px)",
        wrapper: yes,
    }
    WillChange, "will-change" {
        metadata: longhand(false, |v| assert_eq!(*v, CssWillChange::Auto)),
        catalog: grammar_catalog!("ext.property.will-change", "span, contents, --custom", rejected("opacity, none")),
        source: "I-WILLCHANGE1",
        dispatch: "span, contents, --custom",
        wrapper: yes,
    }
    WordBreak, "word-break" {
        metadata: longhand(true, |v| assert_eq!(*v, CssWordBreak::Normal)),
        catalog: grammar_catalog!("baseline.property.word-break", "keep-all", rejected("nowrap")),
        source: "X-TEXT4",
        dispatch: "keep-all",
        wrapper: yes,
    }
    WordSpaceTransform, "word-space-transform" {
        metadata: longhand(true, |v| assert_eq!(*v, CssWordSpaceTransform::None)),
        catalog: grammar_catalog!("ext.property.word-space-transform", "auto-phrase ideographic-space", rejected("auto-phrase")),
        source: "X-TEXT4",
        wrapper: yes,
    }
    WordSpacing, "word-spacing" {
        metadata: longhand(true, |v| {
            assert_eq!(v, &CssTextSpacingAdjustment::Normal)
        }),
        catalog: grammar_catalog!("official.property.word-spacing", "-0.25em", rejected("auto")),
        source: "X-TEXT4",
    }
    WrapAfter, "wrap-after" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssWrapBoundary::Auto);
        }),
        catalog: grammar_catalog!("ext.property.wrap-after", "flex", rejected("anywhere")),
        source: "X-TEXT4",
        wrapper: yes,
    }
    WrapBefore, "wrap-before" {
        metadata: longhand(false, |v| {
            assert_eq!(*v, CssWrapBoundary::Auto);
        }),
        catalog: grammar_catalog!("ext.property.wrap-before", "avoid-line", rejected("loose")),
        source: "X-TEXT4",
        wrapper: yes,
    }
    WrapInside, "wrap-inside" {
        metadata: longhand(false, |v| assert_eq!(*v, CssWrapInside::Auto)),
        catalog: grammar_catalog!("ext.property.wrap-inside", "avoid", rejected("avoid-flex")),
        source: "X-TEXT4",
        wrapper: yes,
    }
    WritingMode, "writing-mode" {
        metadata: longhand(true, |v| assert_eq!(v, &CssWritingMode::HorizontalTb)),
        catalog: grammar_catalog!("baseline.property.writing-mode", "vertical-rl", rejected("lr")),
        source: "S-WRITING4",
        dispatch: "vertical-rl",
        wrapper: yes,
    }
    ZIndex, "z-index" {
        metadata: longhand(false, |v| assert_eq!(v, &CssZIndexValue::Auto)),
        catalog: grammar_catalog!("baseline.property.z-index", "-2", rejected("1.5")),
        dispatch: "-2",
        wrapper: yes,
    }
}
