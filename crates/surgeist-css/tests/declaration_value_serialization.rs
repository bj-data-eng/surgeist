#![forbid(unsafe_code)]

//! Functional tests accompany the new single-declaration API; no existing
//! callable boundary could express this complete output before implementation.
//! Pinned CSSOM1 §6.6 supplies punctuation and importance; each owning property
//! grammar and its focused serialization fixtures supply the typed value text.
//! Syntax3 §10 requires reparsing, while Variables1 §4.1 preserves custom text.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn parsed(source: &str) -> CssDeclaration {
    let report = parse_declaration(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report
        .syntax()
        .as_ref()
        .expect("retained declaration")
        .clone()
}

fn checked(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).expect("known grammar"),
        parse_component_values(value).unwrap(),
        importance,
    )
    .unwrap()
}

#[test]
fn every_represented_ordinary_property_dispatches_its_typed_payload() {
    // These canonical witnesses cover each ordinary owning payload, including
    // every property sharing it. They are independent grammar expectations,
    // not expected strings obtained from a child serializer at runtime.
    let cases: &[(&[&str], &str, &str)] = &[
        // CssContainerNames
        (&["container-name"], "Foo", "Foo"),
        // CssContainerType
        (&["container-type"], "inline-size", "inline-size"),
        // CssContainer
        (&["container"], "Foo / inline-size", "Foo / inline-size"),
        // CssDisplayValue
        (&["display"], "inline flex", "inline flex"),
        // CssBoxSizing
        (&["box-sizing"], "border-box", "border-box"),
        // CssBorderCollapse
        (&["border-collapse"], "collapse", "collapse"),
        // CssBorderSpacing
        (&["border-spacing"], "10px", "10px 10px"),
        // CssCaptionSide
        (&["caption-side"], "bottom", "bottom"),
        // CssClip
        (&["clip"], "auto", "auto"),
        // CssEmptyCells
        (&["empty-cells"], "hide", "hide"),
        // CssPageLineMinimum
        (&["orphans", "widows"], "2", "2"),
        // CssBreakBetween
        (&["break-after", "break-before"], "page", "page"),
        // CssBreakInside
        (&["break-inside"], "avoid", "avoid"),
        // CssQuotes
        (&["quotes"], "auto", "auto"),
        // CssTableLayout
        (&["table-layout"], "fixed", "fixed"),
        // CssScrollSnapType
        (&["scroll-snap-type"], "none", "none"),
        // CssScrollSnapAlign
        (&["scroll-snap-align"], "none", "none"),
        // CssScrollSnapStop
        (&["scroll-snap-stop"], "normal", "normal"),
        // CssScrollPaddingValue
        (
            &[
                "scroll-padding-top",
                "scroll-padding-right",
                "scroll-padding-bottom",
                "scroll-padding-left",
                "scroll-padding-block-start",
                "scroll-padding-block-end",
                "scroll-padding-inline-start",
                "scroll-padding-inline-end",
            ],
            "auto",
            "auto",
        ),
        // CssSpecifiedLength
        (
            &[
                "scroll-margin-top",
                "scroll-margin-right",
                "scroll-margin-bottom",
                "scroll-margin-left",
                "scroll-margin-block-start",
                "scroll-margin-block-end",
                "scroll-margin-inline-start",
                "scroll-margin-inline-end",
                "outline-offset",
            ],
            "10px",
            "10px",
        ),
        // CssScrollPaddingPair
        (
            &["scroll-padding-block", "scroll-padding-inline"],
            "10px",
            "10px",
        ),
        // CssScrollMarginPair
        (
            &["scroll-margin-block", "scroll-margin-inline"],
            "10px",
            "10px",
        ),
        // CssScrollPaddingShorthand
        (&["scroll-padding"], "10px", "10px"),
        // CssScrollMarginShorthand
        (&["scroll-margin"], "10px", "10px"),
        // CssTextSpacingAdjustment
        (&["word-spacing", "letter-spacing"], "normal", "normal"),
        // CssLayoutPosition
        (&["position"], "static", "static"),
        // CssDirection
        (&["direction"], "rtl", "rtl"),
        // CssOverflowValue
        (&["overflow"], "visible", "visible"),
        // CssOverflow
        (
            &[
                "overflow-x",
                "overflow-y",
                "overflow-block",
                "overflow-inline",
            ],
            "visible",
            "visible",
        ),
        // CssOverflowClipMargin
        (&["overflow-clip-margin"], "10px", "10px"),
        // CssWillChange
        (&["will-change"], "opacity", "opacity"),
        // CssOverflowAnchor
        (&["overflow-anchor"], "auto", "auto"),
        // CssScrollBehavior
        (&["scroll-behavior"], "smooth", "smooth"),
        // CssScrollbarGutter
        (&["scrollbar-gutter"], "stable", "stable"),
        // CssVoiceDuration
        (&["voice-duration"], "auto", "auto"),
        // CssVoiceBalance
        (&["voice-balance"], "center", "center"),
        // CssVoiceVolume
        (&["voice-volume"], "medium", "medium"),
        // CssVoicePitchRange
        (&["voice-pitch", "voice-range"], "medium", "medium"),
        // CssVoiceRate
        (&["voice-rate"], "normal", "normal"),
        // CssVoiceFamily
        (&["voice-family"], "male", "male"),
        // CssVoiceStress
        (&["voice-stress"], "normal", "normal"),
        // CssSpeak
        (&["speak"], "auto", "auto"),
        // CssSpeakAs
        (&["speak-as"], "normal", "normal"),
        // CssSpeechBreak
        (
            &["pause-before", "pause-after", "rest-before", "rest-after"],
            "none",
            "none",
        ),
        // CssSpeechBreakPair
        (&["pause", "rest"], "none", "none"),
        // CssCue
        (&["cue-before", "cue-after"], "none", "none"),
        // CssCuePair
        (&["cue"], "none", "none"),
        // CssFlexDirection
        (&["flex-direction"], "row", "row"),
        // CssFlexFlow
        (&["flex-flow"], "row nowrap", "row nowrap"),
        // CssFlexWrap
        (&["flex-wrap"], "nowrap", "nowrap"),
        // CssFloat
        (&["float"], "left", "left"),
        // CssClear
        (&["clear"], "both", "both"),
        // CssAlignContentValue
        (&["align-content"], "center", "center"),
        // CssJustifyContentValue
        (&["justify-content"], "center", "center"),
        // CssAlignItemsValue
        (&["align-items"], "center", "center"),
        // CssAlignSelfValue
        (&["align-self"], "center", "center"),
        // CssJustifyItemsValue
        (&["justify-items"], "center", "center"),
        // CssJustifySelfValue
        (&["justify-self"], "center", "center"),
        // The authored alignment provider contract emits both effective roles,
        // in Align3's align/justify order, including the copied omitted role.
        // This declaration API does not add CSSOM shorthand minimization.
        // CssPlaceContentValue
        (&["place-content"], "center", "center center"),
        // CssPlaceItemsValue
        (&["place-items"], "center", "center center"),
        // CssPlaceSelfValue
        (&["place-self"], "center", "center center"),
        // CssVisibility
        (&["visibility"], "visible", "visible"),
        // CssContentValue
        (&["content"], "none", "none"),
        // CssContentVisibility
        (&["content-visibility"], "auto", "auto"),
        // CssListStyleTypeValue
        (&["list-style-type"], "none", "none"),
        // CssListStylePosition
        (&["list-style-position"], "inside", "inside"),
        // CssImageValue
        (&["list-style-image", "border-image-source"], "none", "none"),
        // CssListStyleValue
        (&["list-style"], "none", "none"),
        // CssMarkerSide
        (&["marker-side"], "match-self", "match-self"),
        // CssCounterChangesValue
        (&["counter-reset", "counter-set"], "Item", "Item 0"),
        // CssCounterChangesValue
        (&["counter-increment"], "Item", "Item 1"),
        // CssSizeValue
        (
            &[
                "width",
                "height",
                "inline-size",
                "block-size",
                "min-width",
                "min-height",
                "min-inline-size",
                "min-block-size",
            ],
            "auto",
            "auto",
        ),
        // CssMaxSizeValue
        (
            &[
                "max-width",
                "max-height",
                "max-inline-size",
                "max-block-size",
            ],
            "none",
            "none",
        ),
        // CssSizePair
        (&["size", "min-size"], "auto", "auto"),
        // CssMaxSizePair
        (&["max-size"], "none", "none"),
        // CssFrameSizing
        (&["frame-sizing"], "auto", "auto"),
        // CssMinIntrinsicSizing
        (&["min-intrinsic-sizing"], "legacy", "legacy"),
        // CssContainIntrinsicSizeValue
        (
            &[
                "contain-intrinsic-width",
                "contain-intrinsic-height",
                "contain-intrinsic-inline-size",
                "contain-intrinsic-block-size",
            ],
            "none",
            "none",
        ),
        // CssContainIntrinsicSize
        (&["contain-intrinsic-size"], "none", "none"),
        // CssFlexBasisValue
        (&["flex-basis"], "auto", "auto"),
        // CssGapShorthand
        (&["gap"], "10px", "10px"),
        // CssGapValue
        (&["row-gap", "column-gap"], "10px", "10px"),
        // CssColumnCount
        (&["column-count"], "auto", "auto"),
        // CssColumnFill: Multicol1 §7.1 admits auto, balance and balance-all.
        (&["column-fill"], "balance", "balance"),
        // CssColumnRule
        (&["column-rule"], "10px", "10px"),
        // CssColor
        (
            &[
                "column-rule-color",
                "text-decoration-color",
                "color",
                "background-color",
                "border-top-color",
                "border-right-color",
                "border-bottom-color",
                "border-left-color",
                "border-block-start-color",
                "border-block-end-color",
                "border-inline-start-color",
                "border-inline-end-color",
            ],
            "purple",
            "purple",
        ),
        // CssLineStyle
        (&["column-rule-style"], "solid", "solid"),
        // CssLineWidth
        (&["column-rule-width"], "medium", "medium"),
        // CssColumnSpan
        (&["column-span"], "all", "all"),
        // CssColumnWidth
        (&["column-width"], "auto", "auto"),
        // CssColumns
        (&["columns"], "auto", "auto auto"),
        // CssFlowTolerance
        (&["flow-tolerance"], "normal", "normal"),
        // CssGridTrackList
        (
            &["grid-template-rows", "grid-template-columns"],
            "10px",
            "10px",
        ),
        // CssGridTemplateAreas
        (&["grid-template-areas"], "none", "none"),
        // CssGridTemplate
        (&["grid-template"], "none", "none"),
        // CssGridTrackSizeList
        (&["grid-auto-rows", "grid-auto-columns"], "10px", "10px"),
        // CssGridAutoFlow
        (&["grid-auto-flow"], "row", "row"),
        // CssGridLine
        (
            &[
                "grid-row-start",
                "grid-row-end",
                "grid-column-start",
                "grid-column-end",
            ],
            "auto",
            "auto",
        ),
        // CssGridLineRange
        (&["grid-row", "grid-column"], "auto", "auto"),
        // CssGridArea
        (&["grid-area"], "auto", "auto"),
        // CssGrid
        (&["grid"], "none", "none"),
        // CssFontSize
        (&["font-size"], "medium", "medium"),
        // CssLineHeight
        (&["line-height"], "normal", "normal"),
        // CssTextCombineUpright
        (&["text-combine-upright"], "none", "none"),
        // CssTextOrientation
        (&["text-orientation"], "upright", "upright"),
        // CssUnicodeBidi: WritingModes3 §2.2 defines normal, not none.
        (&["unicode-bidi"], "normal", "normal"),
        // CssWritingMode
        (&["writing-mode"], "vertical-rl", "vertical-rl"),
        // CssTextAlignValue
        (&["text-align"], "start", "start"),
        // CssTextAlignAllValue
        (&["text-align-all"], "start", "start"),
        // CssTextAlignLastValue
        (&["text-align-last"], "auto", "auto"),
        // CssTextIndent
        (&["text-indent"], "10px", "10px"),
        // CssVerticalAlign
        (&["vertical-align"], "baseline", "baseline"),
        // CssFontFamilyList
        (&["font-family"], "serif", "serif"),
        // CssFontValue
        (&["font"], "caption", "caption"),
        // CssFontWeight
        (&["font-weight"], "normal", "normal"),
        // CssFontStyle
        (&["font-style"], "normal", "normal"),
        // CssFontWidth
        (&["font-width"], "normal", "normal"),
        // CssFontVariantValue
        (&["font-variant"], "normal", "normal"),
        // CssFontVariantCaps
        (&["font-variant-caps"], "normal", "normal"),
        // CssFontVariantEastAsian
        (&["font-variant-east-asian"], "normal", "normal"),
        // CssFontVariantLigatures
        (&["font-variant-ligatures"], "normal", "normal"),
        // CssFontVariantNumeric
        (&["font-variant-numeric"], "normal", "normal"),
        // CssFontVariantPosition
        (&["font-variant-position"], "normal", "normal"),
        // CssFontVariantAlternates
        (&["font-variant-alternates"], "normal", "normal"),
        // CssFontVariantEmoji
        (&["font-variant-emoji"], "normal", "normal"),
        // CssAuthoredFontFeatureSettings
        (&["font-feature-settings"], "normal", "normal"),
        // CssFontKerning
        (&["font-kerning"], "normal", "normal"),
        // CssFontSizeAdjust
        (&["font-size-adjust"], "none", "none"),
        // CssFontLanguageOverride
        (&["font-language-override"], "normal", "normal"),
        // CssFontOpticalSizing
        (&["font-optical-sizing"], "none", "none"),
        // CssFontVariationSettings
        (&["font-variation-settings"], "normal", "normal"),
        // CssFontSynthesis
        (&["font-synthesis"], "none", "none"),
        // CssFontSynthesisWeight
        (&["font-synthesis-weight"], "none", "none"),
        // CssFontSynthesisStyle
        (&["font-synthesis-style"], "none", "none"),
        // CssFontSynthesisSmallCaps
        (&["font-synthesis-small-caps"], "none", "none"),
        // CssFontSynthesisPosition
        (&["font-synthesis-position"], "none", "none"),
        // CssFontPalette
        (&["font-palette"], "normal", "normal"),
        // CssTextWrap
        (&["text-wrap"], "wrap", "wrap"),
        // CssWhiteSpace
        (&["white-space"], "normal", "normal"),
        // CssWordBreak
        (&["word-break"], "normal", "normal"),
        // CssOverflowWrap
        (&["overflow-wrap"], "normal", "normal"),
        // CssTextOverflow
        (&["text-overflow"], "clip", "clip"),
        // CssTextDecoration
        (&["text-decoration"], "none", "none"),
        // CssTextDecorationLine
        (&["text-decoration-line"], "none", "none"),
        // CssTextDecorationStyle
        (&["text-decoration-style"], "dashed", "dashed"),
        // CssTextDecorationThickness
        (&["text-decoration-thickness"], "auto", "auto"),
        // CssTextTransform
        (&["text-transform"], "none", "none"),
        // CssInsetShorthand
        (&["inset"], "10px", "10px"),
        // CssInsetValue
        (
            &[
                "top",
                "right",
                "bottom",
                "left",
                "inset-block-start",
                "inset-block-end",
                "inset-inline-start",
                "inset-inline-end",
            ],
            "10px",
            "10px",
        ),
        // CssInsetPair
        (&["inset-block", "inset-inline"], "10px", "10px"),
        // CssZIndexValue
        (&["z-index"], "auto", "auto"),
        // CssBoxDecorationBreak
        (&["box-decoration-break"], "clone", "clone"),
        // CssMarginShorthand
        (&["margin"], "10px", "10px"),
        // CssMarginValue
        (
            &[
                "margin-top",
                "margin-right",
                "margin-bottom",
                "margin-left",
                "margin-block-start",
                "margin-block-end",
                "margin-inline-start",
                "margin-inline-end",
            ],
            "10px",
            "10px",
        ),
        // CssMarginPair
        (&["margin-block", "margin-inline"], "10px", "10px"),
        // CssPaddingShorthand
        (&["padding"], "10px", "10px"),
        // CssPaddingValue
        (
            &[
                "padding-top",
                "padding-right",
                "padding-bottom",
                "padding-left",
                "padding-block-start",
                "padding-block-end",
                "padding-inline-start",
                "padding-inline-end",
            ],
            "10px",
            "10px",
        ),
        // CssPaddingPair
        (&["padding-block", "padding-inline"], "10px", "10px"),
        // CssBorder
        (
            &[
                "border",
                "border-top",
                "border-right",
                "border-bottom",
                "border-left",
                "border-block-start",
                "border-block-end",
                "border-inline-start",
                "border-inline-end",
                "border-block",
                "border-inline",
            ],
            "solid",
            "solid",
        ),
        // CssBorderWidthShorthand
        (&["border-width"], "10px", "10px"),
        // CssBorderWidth
        (
            &[
                "border-top-width",
                "border-right-width",
                "border-bottom-width",
                "border-left-width",
                "border-block-start-width",
                "border-block-end-width",
                "border-inline-start-width",
                "border-inline-end-width",
            ],
            "medium",
            "medium",
        ),
        // CssBorderWidthPair
        (
            &["border-block-width", "border-inline-width"],
            "10px",
            "10px",
        ),
        // CssBackground
        (&["background"], "purple", "purple"),
        // CssBorderColorShorthand
        (&["border-color"], "purple", "purple"),
        // CssBorderColorPair
        (
            &["border-block-color", "border-inline-color"],
            "purple",
            "purple",
        ),
        // CssImageValueList
        (&["background-image", "mask-image"], "none", "none"),
        // CssBackgroundPositionList
        (&["background-position"], "10px 20px", "10px 20px"),
        // CssPhysicalPosition
        (&["object-position"], "10px 20px", "10px 20px"),
        // CssBackgroundSizeList
        (&["background-size", "mask-size"], "auto", "auto"),
        // CssBackgroundRepeatList
        (&["background-repeat", "mask-repeat"], "repeat", "repeat"),
        // CssBackgroundBoxList
        (
            &["background-origin", "background-clip"],
            "border-box",
            "border-box",
        ),
        // CssBackgroundAttachmentList
        (&["background-attachment"], "fixed", "fixed"),
        // CssBorderImage
        (&["border-image"], "none", "none"),
        // CssBorderImageOutset
        (&["border-image-outset"], "0", "0"),
        // CssBorderImageRepeat
        (&["border-image-repeat"], "stretch", "stretch"),
        // CssBorderImageSlice
        (&["border-image-slice"], "100%", "100%"),
        // CssBorderImageWidth
        (&["border-image-width"], "1", "1"),
        // CssImageOrientation
        (&["image-orientation"], "none", "none"),
        // CssImageRendering
        (&["image-rendering"], "auto", "auto"),
        // CssObjectFit
        (&["object-fit"], "cover", "cover"),
        // CssBorderStyleShorthand
        (&["border-style"], "solid", "solid"),
        // CssBorderStyle
        (
            &[
                "border-top-style",
                "border-right-style",
                "border-bottom-style",
                "border-left-style",
                "border-block-start-style",
                "border-block-end-style",
                "border-inline-start-style",
                "border-inline-end-style",
            ],
            "solid",
            "solid",
        ),
        // CssBorderStylePair
        (
            &["border-block-style", "border-inline-style"],
            "solid",
            "solid",
        ),
        // CssBorderRadiusShorthand
        (&["border-radius"], "10px", "10px"),
        // CssCornerRadiusValue
        (
            &[
                "border-top-left-radius",
                "border-top-right-radius",
                "border-bottom-right-radius",
                "border-bottom-left-radius",
                "border-start-start-radius",
                "border-start-end-radius",
                "border-end-start-radius",
                "border-end-end-radius",
            ],
            "10px",
            "10px",
        ),
        // CssBoxShadow
        (&["box-shadow"], "none", "none"),
        // CssOpacityValue
        (&["opacity"], "0.5", "0.5"),
        // CssSpecifiedNonNegativeNumber
        (&["flex-grow", "flex-shrink"], "2", "2"),
        // CssIntegerValue
        (&["order"], "-1", "-1"),
        // CssFlexValue
        (&["flex"], "none", "none"),
        // CssAspectRatioValue
        (&["aspect-ratio"], "2 / 1", "2 / 1"),
        // CssScrollbarWidth
        (&["scrollbar-width"], "thin", "thin"),
        // CssScrollbarColor
        (&["scrollbar-color"], "auto", "auto"),
        // CssColorScheme
        (&["color-scheme"], "normal", "normal"),
        // CssForcedColorAdjust
        (&["forced-color-adjust"], "auto", "auto"),
        // CssPrintColorAdjust
        (&["print-color-adjust", "color-adjust"], "exact", "exact"),
        // CssCursor
        (&["cursor"], "auto", "auto"),
        // CssCaretColor
        (&["caret-color"], "auto", "auto"),
        // CssPointerEvents
        (&["pointer-events"], "none", "none"),
        // CssUserSelect
        (&["user-select"], "none", "none"),
        // CssResize
        (&["resize"], "none", "none"),
        // CssContain
        (&["contain"], "none", "none"),
        // CssOutline
        (&["outline"], "none", "none"),
        // CssOutlineColor
        (&["outline-color"], "auto", "auto"),
        // CssOutlineStyle
        (&["outline-style"], "none", "none"),
        // CssOutlineWidth
        (&["outline-width"], "medium", "medium"),
        // CssTransform
        (&["transform"], "none", "none"),
        // CssTransformBox
        (&["transform-box"], "border-box", "border-box"),
        // CssTransformOrigin
        (&["transform-origin"], "10px 20px", "10px 20px"),
        // CssTranslate
        (&["translate"], "none", "none"),
        // CssRotate
        (&["rotate"], "none", "none"),
        // CssScale
        (&["scale"], "none", "none"),
        // CssFilter
        (&["filter", "backdrop-filter"], "none", "none"),
        // CssClipPath
        (&["clip-path"], "none", "none"),
        // CssBlendModeList
        (&["background-blend-mode"], "multiply", "multiply"),
        // CssIsolation
        (&["isolation"], "auto", "auto"),
        // CssBlendMode
        (&["mix-blend-mode"], "multiply", "multiply"),
        // CssMaskList
        (&["mask"], "none", "none"),
        // CssPhysicalPositionList
        (&["mask-position"], "10px 20px", "10px 20px"),
        // CssTransitionPropertyList: Transitions1 §2.1 defines the all keyword.
        (&["transition-property"], "all", "all"),
        // CssDurationList
        (&["transition-duration", "animation-duration"], "1s", "1s"),
        // CssDelayList
        (&["transition-delay", "animation-delay"], "-1s", "-1s"),
        // CssEasingList
        (
            &["transition-timing-function", "animation-timing-function"],
            "ease",
            "ease",
        ),
        // CssTransitionList
        (&["transition"], "none", "none"),
        // CssAnimationNameList
        (&["animation-name"], "none", "none"),
        // CssAnimationIterationCountList
        (&["animation-iteration-count"], "2", "2"),
        // CssAnimationDirectionList
        (&["animation-direction"], "normal", "normal"),
        // CssAnimationFillModeList
        (&["animation-fill-mode"], "none", "none"),
        // CssAnimationPlayStateList
        (&["animation-play-state"], "paused", "paused"),
        // CssAnimationList
        (&["animation"], "none", "none"),
    ];
    let mut failures = Vec::new();
    for &(names, input, value_css) in cases {
        for &name in names {
            let expected = format!("{name}: {value_css} !important;");
            let source = format!("{name}:{input}!IMPORTANT");
            let report = parse_declaration(&source);
            if !report.is_clean() {
                failures.push(format!("{source}: {:?}", report.diagnostics()));
                continue;
            }
            let parsed = report.syntax().as_ref().unwrap();
            let checked = checked(name, input, CssImportance::Important);
            for declaration in [parsed, &checked] {
                assert!(
                    declaration.known().unwrap().property_value().is_some(),
                    "ordinary {name}"
                );
                let before = declaration.clone();
                let output = declaration.to_specified_css();
                if output.as_deref() != Ok(expected.as_str()) {
                    failures.push(format!("{source}: expected {expected:?}, got {output:?}"));
                    continue;
                }
                let reparse = parse_style_attribute(&expected);
                assert!(
                    reparse.is_clean(),
                    "{expected}: {:?}",
                    reparse.diagnostics()
                );
                assert_eq!(reparse.syntax()[0].to_specified_css().unwrap(), expected);
                assert_eq!(declaration, &before);
                assert!(declaration.same_occurrence(&before));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn exact_limits(declaration: &CssDeclaration, expected: &str, input: usize, projection: usize) {
    let before = declaration.clone();
    let components = declaration.value_components().clone();
    let name_origin = declaration.parsed_name().cloned();
    let value_origin = declaration.parsed_value().cloned();
    assert_eq!(
        declaration
            .to_specified_css_with_limits(Limits::new(input, projection, expected.len()))
            .unwrap(),
        expected,
    );
    for (limits, kind) in [
        (
            Limits::new(0, projection, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(input - 1, projection, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(input, 0, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(input, projection - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(input, projection, 0), Kind::ByteLimit),
        (
            Limits::new(input, projection, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(
            declaration
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(declaration.to_specified_css().unwrap(), expected);
        assert_eq!(declaration, &before);
        assert!(declaration.same_occurrence(&before));
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(declaration.parsed_name(), name_origin.as_ref());
        assert_eq!(declaration.parsed_value(), value_origin.as_ref());
    }
}

#[test]
fn single_declaration_punctuation_and_css_wide_keywords_have_exact_limits() {
    for (name, input, value) in [
        ("color", "PuRpLe", "purple"),
        ("width", "INHERIT", "inherit"),
        ("all", "INITIAL", "initial"),
        ("padding", "UNSET", "unset"),
        ("border", "REVERT", "revert"),
        ("display", "REVERT-LAYER", "revert-layer"),
    ] {
        for importance in [CssImportance::Normal, CssImportance::Important] {
            let annotation = if importance == CssImportance::Important {
                " !important"
            } else {
                ""
            };
            let source = format!("{name}:{input}{annotation}");
            let expected = format!("{name}: {value}{annotation};");
            for declaration in [parsed(&source), checked(name, input, importance)] {
                exact_limits(&declaration, &expected, 3, 3);
            }
        }
    }
}

#[test]
fn name_equivalent_aliases_use_canonical_owners_and_preserve_original_names() {
    for (name, input, expected) in [
        (
            "-WEBKIT-FLEX-DIRECTION",
            "COLUMN",
            "flex-direction: column;",
        ),
        ("word-wrap", "BREAK-WORD", "overflow-wrap: break-word;"),
        ("font-stretch", "CONDENSED", "font-width: condensed;"),
        ("COLOR", "RED", "color: red;"),
    ] {
        for declaration in [
            parsed(&format!("{name}:{input}")),
            checked(name, input, CssImportance::Normal),
        ] {
            let original = declaration.parsed_name().cloned();
            assert_eq!(declaration.to_specified_css().unwrap(), expected);
            assert_eq!(declaration.parsed_name(), original.as_ref());
            if let Some(origin) = original {
                assert_eq!(origin.source().as_str(), format!("{name}:{input}"));
            }
        }
    }
}

#[test]
fn legacy_grammar_text_inverts_only_its_source_defined_semantic_mapping() {
    for (name, input, value, projection) in [
        ("page-break-before", "ALWAYS", "always", 3),
        ("page-break-after", "always", "always", 3),
        ("page-break-before", "avoid", "avoid", 3),
        ("page-break-after", "left", "left", 3),
        ("page-break-inside", "avoid", "avoid", 3),
        ("glyph-orientation-vertical", "auto", "auto", 3),
        ("glyph-orientation-vertical", "0", "0deg", 4),
        ("glyph-orientation-vertical", "90", "90deg", 4),
        ("glyph-orientation-vertical", "0deg", "0deg", 4),
        ("glyph-orientation-vertical", "90deg", "90deg", 4),
    ] {
        let expected = format!("{name}: {value} !important;");
        for declaration in [
            parsed(&format!("{name}:{input}!important")),
            checked(name, input, CssImportance::Important),
        ] {
            assert_eq!(declaration.known().unwrap().grammar().name(), name);
            exact_limits(&declaration, &expected, 3, projection);
            let reparse = parse_style_attribute(&expected);
            assert!(
                reparse.is_clean(),
                "{expected}: {:?}",
                reparse.diagnostics()
            );
            assert_eq!(
                reparse.syntax()[0].known().unwrap().grammar(),
                declaration.known().unwrap().grammar()
            );
            assert_eq!(reparse.syntax()[0].to_specified_css().unwrap(), expected);
        }
    }
}

#[test]
fn counter_defaults_and_math_share_the_declaration_budget_without_resolving_integers() {
    for (name, value) in [
        ("counter-reset", "0"),
        ("counter-increment", "1"),
        ("counter-set", "0"),
    ] {
        for declaration in [
            parsed(&format!("{name}:Item")),
            checked(name, "Item", CssImportance::Normal),
        ] {
            let expected = format!("{name}: Item {value};");
            // Declaration + name, then list + entry + name; generated default
            // adds one projection node and no authored input node.
            exact_limits(&declaration, &expected, 5, 6);
        }
        for declaration in [
            parsed(&format!("{name}:Item calc(2.5)")),
            checked(name, "Item calc(2.5)", CssImportance::Normal),
        ] {
            // Numeric math preserves the integer-domain calc wrapper. Its
            // owner costs 5 input / 4 projection nodes including the list.
            exact_limits(&declaration, &format!("{name}: Item calc(2.5);"), 7, 6);
        }
    }
}

#[test]
fn semantic_children_canonicalize_numeric_math_and_new_aggregate_owners() {
    for (name, input, value) in [
        ("width", "+02.500PX", "2.5px"),
        ("width", "calc(2px + 3px)", "calc(5px)"),
        ("opacity", "calc(0.2 + 0.3)", "calc(0.5)"),
        ("border-top-width", "THICK", "thick"),
        ("column-rule-width", "THIN", "thin"),
        (
            "color",
            "rgb(calc(100 * 4) 127 calc(20 - 35))",
            "rgb(255, 127, 0)",
        ),
        (
            "cursor",
            "url(cursor.cur) 3.000e1 -0.25, POINTER",
            "url(\"cursor.cur\") 30 -0.25, pointer",
        ),
        ("outline", "RED SOLID THIN", "thin solid red"),
        ("mask", "url(mask.png)", "url(\"mask.png\")"),
        (
            "transform",
            "translateX(+02.500PX) rotate(90DEG)",
            "translateX(2.5px) rotate(90deg)",
        ),
        ("translate", "10px 0px 0px", "10px"),
        ("scale", "2 2 1", "2"),
        ("rotate", "z 90deg", "90deg"),
        ("transition", "opacity 1s ease 0s", "opacity 1s ease 0s"),
    ] {
        let expected = format!("{name}: {value} !important;");
        for declaration in [
            parsed(&format!("{name}:{input}!important")),
            checked(name, input, CssImportance::Important),
        ] {
            let before = declaration.clone();
            let components = declaration.value_components().clone();
            assert_eq!(declaration.to_specified_css().unwrap(), expected);
            assert_eq!(
                declaration
                    .to_specified_css_with_limits(Limits::new(65_536, 262_144, expected.len() - 1))
                    .unwrap_err()
                    .kind(),
                Kind::ByteLimit
            );
            assert_eq!(declaration.value_components(), &components);
            assert_eq!(declaration, before);
            assert!(declaration.same_occurrence(&before));
        }
    }
}

fn custom_checked(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    let name = CssCustomPropertyName::try_new(name).unwrap();
    parse_property_value(
        CssPropertyNameRef::Custom(&name),
        parse_component_values(value).unwrap(),
        importance,
    )
    .unwrap()
}

#[test]
fn custom_values_and_custom_css_wide_keywords_preserve_retained_case_and_tokens() {
    for (name, input, name_css, nodes) in [
        ("--Case", "MiXeD", "--Case", 4),
        ("--Case", "INITIAL", "--Case", 4),
        ("--Case", "rEvErT-LaYeR", "--Case", 4),
        (r"--\41", "01.00PX", "--A", 4),
        (r"--A\ B", "A/**/B", r"--A\ B", 6),
        ("--空", "/*Head*/MiXeD/*Tail*/", "--空", 6),
        ("--Empty", "", "--Empty", 3),
    ] {
        for importance in [CssImportance::Normal, CssImportance::Important] {
            let annotation = if importance == CssImportance::Important {
                " !important"
            } else {
                ""
            };
            // No annotation-separating whitespace is included in the authored
            // value stimulus; annotation output is owned by the declaration.
            let bang = if importance == CssImportance::Important {
                "!important"
            } else {
                ""
            };
            let expected = format!("{name_css}: {input}{annotation};");
            for declaration in [
                parsed(&format!("{name}:{input}{bang}")),
                custom_checked(name, input, importance),
            ] {
                exact_limits(&declaration, &expected, nodes, nodes);
                assert!(declaration.custom().is_some());
                if input == "INITIAL" || input == "rEvErT-LaYeR" {
                    assert!(declaration.custom().unwrap().value().global().is_some());
                }
            }
        }
    }
}

#[test]
fn pending_known_values_and_all_keep_original_function_case_and_symbolic_text() {
    for (name, input, expected, nodes) in [
        ("width", "VaR(--Case)", "width: VaR(--Case);", 5),
        ("all", "var(--Reset)", "all: var(--Reset);", 5),
    ] {
        for declaration in [
            parsed(&format!("{name}:{input}")),
            checked(name, input, CssImportance::Normal),
        ] {
            assert!(
                declaration
                    .known()
                    .unwrap()
                    .substitution_dependent()
                    .is_some()
            );
            exact_limits(&declaration, expected, nodes, nodes);
        }
    }
    for (name, input) in [
        ("padding", "VaR(--Case, +01.00PX)"),
        ("width", "ENV(future, calc(1PX + 2PX))"),
        ("all", "var(--Reset,/**/INITIAL)"),
    ] {
        let expected = format!("{name}: {input};");
        let declaration = parsed(&format!("{name}:{input}"));
        let components = declaration.value_components().clone();
        assert_eq!(declaration.to_specified_css().unwrap(), expected);
        assert_eq!(declaration.value_components(), &components);
    }
}

#[test]
fn constructed_custom_adjacency_uses_the_shared_component_boundary_owner() {
    let name = CssCustomPropertyName::try_new("--Tokens").unwrap();
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("A").unwrap(),
        CssComponentValue::try_ident("B").unwrap(),
    ])
    .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    exact_limits(&declaration, "--Tokens: A/**/B;", 5, 5);
    assert_eq!(declaration.value_components(), &components);
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
}

#[test]
fn owning_precision_failure_returns_no_declaration_text_and_preserves_occurrence() {
    for (name, input) in [
        ("rotate", "calc(.00000001) 1 0 30deg"),
        (
            "transform",
            "translateX(2px) rotate3d(1e-999, 1, 0, 30deg) scale(2)",
        ),
    ] {
        for declaration in [
            parsed(&format!("{name}:{input}!important")),
            checked(name, input, CssImportance::Important),
        ] {
            let before = declaration.clone();
            let components = declaration.value_components().clone();
            let value_origin = declaration.parsed_value().cloned();
            for _ in 0..2 {
                assert_eq!(
                    declaration.to_specified_css().unwrap_err().kind(),
                    Kind::UnrepresentableValue
                );
                assert_eq!(declaration.value_components(), &components);
                assert_eq!(declaration.parsed_value(), value_origin.as_ref());
                assert_eq!(declaration, before);
                assert!(declaration.same_occurrence(&before));
            }
        }
    }
}

#[test]
fn symbolic_edge_whitespace_is_trimmed_without_dropping_comments_or_refunding_work() {
    for (name, input, value, nodes) in [
        ("--X", " \tINITIAL \n", "INITIAL", 6),
        ("--X", " \tMiXeD \n", "MiXeD", 6),
        (
            "--X",
            " \t/*Head*/  MiXeD /*Tail*/\n",
            "/*Head*/  MiXeD /*Tail*/",
            10,
        ),
        ("--X", " \t \n", "", 4),
        ("width", " \tVaR(--Case) \n", "VaR(--Case)", 7),
        ("all", " \tvar(--Reset) \n", "var(--Reset)", 7),
    ] {
        let expected = format!("{name}: {value} !important;");
        let checked = if name.starts_with("--") {
            custom_checked(name, input, CssImportance::Important)
        } else {
            checked(name, input, CssImportance::Important)
        };
        for declaration in [parsed(&format!("{name}:{input}!important")), checked] {
            exact_limits(&declaration, &expected, nodes, nodes);
            let reparse = parse_style_attribute(&expected);
            assert!(
                reparse.is_clean(),
                "{expected}: {:?}",
                reparse.diagnostics()
            );
            assert_eq!(reparse.syntax()[0].to_specified_css().unwrap(), expected);
        }
    }
}

#[test]
fn nested_symbolic_trivia_and_recovered_closures_keep_their_original_origins() {
    let input = " \tvar(--Case, /*inside*/  01.00PX ) \n";
    let expected = "--X: var(--Case, /*inside*/  01.00PX );";
    for declaration in [
        parsed(&format!("--X:{input}")),
        custom_checked("--X", input, CssImportance::Normal),
    ] {
        let components = declaration.value_components().clone();
        assert_eq!(declaration.to_specified_css().unwrap(), expected);
        assert_eq!(declaration.value_components(), &components);
    }
    let source = "--X:var(--Case";
    let report = parse_style_attribute(source);
    assert!(!report.is_clean());
    assert_eq!(report.syntax().len(), 1);
    let declaration = &report.syntax()[0];
    let components = declaration.value_components().clone();
    let origins: Vec<_> = components
        .items()
        .iter()
        .map(|item| item.origin().clone())
        .collect();
    exact_limits(declaration, "--X: var(--Case);", 5, 5);
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(
        declaration
            .value_components()
            .items()
            .iter()
            .map(|item| item.origin().clone())
            .collect::<Vec<_>>(),
        origins
    );
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        source
    );
}
