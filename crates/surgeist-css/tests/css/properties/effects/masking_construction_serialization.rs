//! Functional public-model contracts accompanying full authored Masking support.
//! Expected strings come from the selected grammar and documented shared numeric
//! policy. Work formulas visit retained semantic nodes, including shortened edges.
use crate::css::support::serialization_limits::assert_limits as bounds;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::numeric::{
    CssCalculation, CssHintedNumber, CssNonNegative, CssNumber, CssNumeric,
};
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::background::CssBackgroundSize;
use surgeist_css::values::border::{
    CssBorderImageOutset, CssBorderImageOutsetComponent, CssBorderImageRepeat,
    CssBorderImageRepeatKeyword, CssBorderImageSlice, CssBorderImageSliceComponent,
    CssBorderImageWidth, CssBorderImageWidthComponent,
};
use surgeist_css::values::common::CssBoxEdgeKeyword;
use surgeist_css::values::effects::{
    CssClipRule, CssMaskBorder, CssMaskBoxList, CssMaskClip, CssMaskClipList, CssMaskComposite,
    CssMaskCompositeList, CssMaskLayer, CssMaskLayerBoxes, CssMaskList, CssMaskMode,
    CssMaskModeList, CssMaskType,
};
use surgeist_css::values::image::CssImageValue;
use surgeist_css::values::transform::CssCoordBox;
type Limits = CssSerializationLimits;

fn number(text: &str) -> CssNumeric<CssNumber, CssNonNegative> {
    CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
        CssComponentValue::try_number(text).unwrap(),
    )
    .unwrap()
}
fn slice() -> CssBorderImageSlice {
    CssBorderImageSlice::try_new(
        vec![CssBorderImageSliceComponent::Number(number("10"))],
        true,
    )
    .unwrap()
}
fn width() -> CssBorderImageWidth {
    CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Auto]).unwrap()
}
fn outset() -> CssBorderImageOutset {
    CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(number("2"))]).unwrap()
}
fn border() -> CssMaskBorder {
    CssMaskBorder::try_new(
        Some(CssImageValue::None),
        Some(slice()),
        Some(width()),
        Some(outset()),
        Some(CssBorderImageRepeat::new(
            CssBorderImageRepeatKeyword::Round,
            CssBorderImageRepeatKeyword::Space,
        )),
        Some(CssMaskType::Luminance),
    )
    .unwrap()
}

#[test]
fn narrowed_mask_boxes_and_layer_slots_are_intrinsically_checked() {
    assert_eq!(
        CssCoordBox::try_from(CssBoxEdgeKeyword::MarginBox).ok(),
        None
    );
    assert_eq!(
        CssCoordBox::try_from(CssBoxEdgeKeyword::ViewBox).ok(),
        Some(CssCoordBox::ViewBox)
    );
    assert_eq!(CssCoordBox::FillBox.as_str(), "fill-box");
    let one = CssMaskLayerBoxes::Box(CssCoordBox::ContentBox);
    let pair = CssMaskLayerBoxes::Pair {
        origin: CssCoordBox::PaddingBox,
        clip: CssMaskClip::NoClip,
    };
    assert_eq!(one.origin(), Some(CssCoordBox::ContentBox));
    assert_eq!(one.clip(), CssMaskClip::Box(CssCoordBox::ContentBox));
    assert_eq!(pair.origin(), Some(CssCoordBox::PaddingBox));
    assert_eq!(pair.clip(), CssMaskClip::NoClip);
    assert_eq!(CssMaskLayerBoxes::NoClip.origin(), None);
    assert_eq!(CssMaskLayerBoxes::NoClip.clip(), CssMaskClip::NoClip);
    assert!(CssMaskLayer::try_new(None, None, None, None, None, None, None).is_none());
    let layer = CssMaskLayer::try_new(
        Some(CssImageValue::None),
        None,
        None,
        None,
        Some(pair),
        Some(CssMaskComposite::Exclude),
        Some(CssMaskMode::Alpha),
    )
    .unwrap();
    assert_eq!(layer.image(), Some(&CssImageValue::None));
    assert!(layer.position().is_none());
    assert!(layer.size().is_none());
    assert_eq!(layer.repeat(), None);
    assert_eq!(layer.boxes(), Some(pair));
    assert_eq!(layer.composite(), Some(CssMaskComposite::Exclude));
    assert_eq!(layer.mode(), Some(CssMaskMode::Alpha));
    let value = CssMaskList::try_new(vec![layer]).unwrap();
    let before = value.clone();
    // list + layer + none + box group/two slots + operator + mode.
    bounds("none padding-box no-clip exclude alpha", 8, 8, |limits| {
        value.serialize_specified(limits)
    });
    assert_eq!(value, before);
    bounds("content-box", 2, 2, |limits| {
        one.serialize_specified(limits)
    });
    bounds("padding-box no-clip", 3, 3, |limits| {
        pair.serialize_specified(limits)
    });
    bounds("no-clip", 2, 2, |limits| {
        CssMaskLayerBoxes::NoClip.serialize_specified(limits)
    });
    let explicit_pair = CssMaskLayerBoxes::Pair {
        origin: CssCoordBox::ContentBox,
        clip: CssMaskClip::Box(CssCoordBox::ContentBox),
    };
    assert_ne!(one, explicit_pair);
    bounds("content-box content-box", 3, 3, |limits| {
        explicit_pair.serialize_specified(limits)
    });
}

#[test]
fn nonempty_longhand_lists_preserve_independent_arity_and_exact_work() {
    assert!(CssMaskBoxList::try_new(vec![]).is_none());
    assert!(CssMaskClipList::try_new(vec![]).is_none());
    assert!(CssMaskModeList::try_new(vec![]).is_none());
    assert!(CssMaskCompositeList::try_new(vec![]).is_none());
    let boxes =
        CssMaskBoxList::try_new(vec![CssCoordBox::ContentBox, CssCoordBox::ViewBox]).unwrap();
    assert_eq!(
        boxes.boxes(),
        &[CssCoordBox::ContentBox, CssCoordBox::ViewBox]
    );
    bounds("content-box, view-box", 3, 3, |limits| {
        boxes.serialize_specified(limits)
    });
    let clips = CssMaskClipList::try_new(vec![
        CssMaskClip::NoClip,
        CssMaskClip::Box(CssCoordBox::StrokeBox),
    ])
    .unwrap();
    assert_eq!(
        clips.clips(),
        &[
            CssMaskClip::NoClip,
            CssMaskClip::Box(CssCoordBox::StrokeBox)
        ]
    );
    bounds("no-clip, stroke-box", 3, 3, |limits| {
        clips.serialize_specified(limits)
    });
    let modes = CssMaskModeList::try_new(vec![
        CssMaskMode::Alpha,
        CssMaskMode::Luminance,
        CssMaskMode::MatchSource,
    ])
    .unwrap();
    assert_eq!(modes.modes().len(), 3);
    bounds("alpha, luminance, match-source", 4, 4, |limits| {
        modes.serialize_specified(limits)
    });
    let operators = CssMaskCompositeList::try_new(vec![
        CssMaskComposite::Add,
        CssMaskComposite::Subtract,
        CssMaskComposite::Intersect,
        CssMaskComposite::Exclude,
    ])
    .unwrap();
    assert_eq!(operators.operators().len(), 4);
    bounds("add, subtract, intersect, exclude", 5, 5, |limits| {
        operators.serialize_specified(limits)
    });
    bounds("evenodd", 1, 1, |limits| {
        CssClipRule::Evenodd.serialize_specified(limits)
    });
    bounds("nonzero", 1, 1, |limits| {
        CssClipRule::Nonzero.serialize_specified(limits)
    });
    bounds("luminance", 1, 1, |limits| {
        CssMaskType::Luminance.serialize_specified(limits)
    });
    bounds("alpha", 1, 1, |limits| {
        CssMaskType::Alpha.serialize_specified(limits)
    });
}

#[test]
fn border_constructor_reuses_checked_edges_and_rejects_impossible_slash_groups() {
    assert!(CssMaskBorder::try_new(None, None, None, None, None, None).is_none());
    assert!(
        CssMaskBorder::try_new(
            Some(CssImageValue::None),
            None,
            Some(width()),
            None,
            None,
            None
        )
        .is_none()
    );
    assert!(
        CssMaskBorder::try_new(
            None,
            None,
            None,
            Some(outset()),
            None,
            Some(CssMaskType::Alpha)
        )
        .is_none()
    );
    let value = border();
    let before = value.clone();
    assert_eq!(value.source(), Some(&CssImageValue::None));
    assert_eq!(value.slice(), Some(&slice()));
    assert_eq!(value.width(), Some(&width()));
    assert_eq!(value.outset(), Some(&outset()));
    assert_eq!(value.mode(), Some(CssMaskType::Luminance));
    assert_eq!(
        value.repeat().unwrap().vertical(),
        CssBorderImageRepeatKeyword::Space
    );
    for edge in value.slice().unwrap().values() {
        let CssBorderImageSliceComponent::Number(number) = edge else {
            panic!("number slice")
        };
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(
            number.literal().map(|literal| literal.component()),
            Some(&CssComponentValue::try_number("10").unwrap())
        );
    }
    // Aggregate 1 + source 1 + slice (aggregate1/four edges2/fill1)10
    // + width (aggregate1/four auto leaves1)5 + outset (aggregate1/four edges2)9
    // + repeat3 + mode1 = 30. Shortening visits all four expanded edges.
    bounds(
        "none 10 fill / auto / 2 round space luminance",
        30,
        30,
        |limits| value.serialize_specified(limits),
    );
    assert_eq!(value, before);
    let only_source =
        CssMaskBorder::try_new(Some(CssImageValue::None), None, None, None, None, None).unwrap();
    bounds("none", 2, 2, |limits| {
        only_source.serialize_specified(limits)
    });
    let only_mode =
        CssMaskBorder::try_new(None, None, None, None, None, Some(CssMaskType::Alpha)).unwrap();
    bounds("alpha", 2, 2, |limits| {
        only_mode.serialize_specified(limits)
    });
    let skipped_width =
        CssMaskBorder::try_new(None, Some(slice()), None, Some(outset()), None, None).unwrap();
    bounds("10 fill / / 2", 20, 20, |limits| {
        skipped_width.serialize_specified(limits)
    });
}

#[test]
fn parsed_border_children_retain_span_and_origin_through_failure_and_retry() {
    let authored = "/*😀*/mask-border:10 20% fill / auto / 2 round luminance!important";
    let report = CssParser::default().parse_block_contents(authored);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let CssKnownPropertyValueRef::MaskBorder(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("mask border")
    };
    let value = wrapper.value();
    let before = value.clone();
    let CssBorderImageSliceComponent::Number(top) = &value.slice().unwrap().values()[0] else {
        panic!("number")
    };
    let CssValueOrigin::Parsed(origin) = top.origin() else {
        panic!("parsed number")
    };
    let start = authored.find("10").unwrap();
    assert_eq!(origin.source().as_str(), authored);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), start + 2);
    assert!(
        value
            .serialize_specified(Limits::new(100, 100, 1))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(value, &before);
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "10 20% fill / auto / 2 round luminance"
    );
    assert_eq!(top.origin(), &CssValueOrigin::Parsed(origin.clone()));
}

#[test]
fn source_retained_size_only_projection_and_all_new_children_compose() {
    let layer = CssMaskLayer::try_new(
        None,
        None,
        Some(CssBackgroundSize::Contain),
        None,
        Some(CssMaskLayerBoxes::Box(CssCoordBox::ViewBox)),
        Some(CssMaskComposite::Intersect),
        Some(CssMaskMode::MatchSource),
    )
    .unwrap();
    let value = CssMaskList::try_new(vec![layer]).unwrap();
    let before = value.clone();
    // Original size-only3/5 + box group/scalar2 + composite1 + mode1.
    bounds(
        "0% 0% / contain view-box intersect match-source",
        7,
        9,
        |limits| value.serialize_specified(limits),
    );
    assert!(value.layers()[0].position().is_none());
    assert_eq!(value, before);
    let report = CssParser::default()
        .parse_block_contents("mask:0% 0% / contain view-box intersect match-source");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::Mask(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("mask")
    };
    assert!(wrapper.value().layers()[0].position().is_some());
    assert_ne!(*wrapper.value(), value);
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "0% 0% / contain view-box intersect match-source"
    );
}

#[test]
fn mask_border_math_retains_hinted_and_dimensioned_domains_with_shared_projection() {
    let components =
        CssComponentValues::parse(&CssParser::default(), "calc((1px + 1%) / 1px)").unwrap();
    let hinted = CssCalculation::<CssHintedNumber>::try_from_components(
        components.clone(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(hinted.origin(), components.items()[0].origin());
    assert!(hinted.position().is_some());
    assert_eq!(hinted.components(), &components);
    let slice = CssBorderImageSlice::try_new(
        vec![CssBorderImageSliceComponent::HintedNumberCalculation(
            hinted.clone(),
        )],
        false,
    )
    .unwrap();
    let width =
        CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::HintedNumberCalculation(
            hinted,
        )])
        .unwrap();
    let value = CssMaskBorder::try_new(
        None,
        Some(slice),
        Some(width),
        None,
        None,
        Some(CssMaskType::Alpha),
    )
    .unwrap();
    let before = value.clone();
    // Values 4 orders Percentage before dimensioned terms; cancellation leaves
    // a hinted Number without evaluating a contextual percentage basis.
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "calc((1% + 1px) / 1px) / calc((1% + 1px) / 1px) alpha"
    );
    assert!(
        value
            .serialize_specified(Limits::new(100, 100, 1))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(value, before);
    assert!(matches!(
        &value.slice().unwrap().values()[0],
        CssBorderImageSliceComponent::HintedNumberCalculation(_)
    ));
    assert!(matches!(
        &value.width().unwrap().values()[0],
        CssBorderImageWidthComponent::HintedNumberCalculation(_)
    ));
    let report = CssParser::default().parse_block_contents(
        "mask-border:calc((1px + 1%) / 1px) / calc(1px + 2%) / calc(2px + 1em)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::MaskBorder(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("mask border")
    };
    assert!(
        matches!(&wrapper.value().width().unwrap().values()[0],CssBorderImageWidthComponent::LengthPercentage(v) if v.calculation().is_some())
    );
    assert!(
        matches!(&wrapper.value().outset().unwrap().values()[0],CssBorderImageOutsetComponent::Length(v) if v.calculation().is_some())
    );
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "calc((1% + 1px) / 1px) / calc(2% + 1px) / calc(1em + 2px)"
    );
}
