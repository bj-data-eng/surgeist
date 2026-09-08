use surgeist_css::{
    CssColorStopListItem, CssErrorCode, CssGlobalKeyword, CssGradient, CssHorizontalGradientSide,
    CssImageValue, CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLength,
    CssLinearGradientDirection, CssRadialExtent, CssRadialShape, CssRadialSize, CssRecoveryAction,
    CssVerticalGradientSide, ErrorKind, parse_style_attribute,
};

#[test]
fn c13_images_and_gradients_retain_typed_structure() {
    let report = parse_style_attribute(concat!(
        "background-image: url(hero.png), none, ",
        "linear-gradient(to right top, red 0%, 25%, blue), ",
        "radial-gradient(circle closest-side at left 10px top 20%, red, blue 75%), ",
        "repeating-linear-gradient(45deg, #000 10px, #fff 30px), ",
        "repeating-radial-gradient(ellipse 20% 30% at center, red, blue 40px)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::BackgroundImage(value) = report.syntax()[0]
        .known()
        .expect("known background-image")
        .property_value()
        .expect("ordinary background-image")
    else {
        panic!("expected background-image wrapper");
    };
    assert_eq!(value.images().images().len(), 6);
    assert!(matches!(
        &value.images().images()[0],
        CssImageValue::Url(url) if url.as_str() == "hero.png"
    ));
    assert!(matches!(value.images().images()[1], CssImageValue::None));

    let CssImageValue::Gradient(CssGradient::Linear(linear)) = &value.images().images()[2] else {
        panic!("expected linear gradient");
    };
    assert!(matches!(
        linear.direction(),
        Some(CssLinearGradientDirection::SideOrCorner(direction))
            if direction.horizontal() == Some(CssHorizontalGradientSide::Right)
                && direction.vertical() == Some(CssVerticalGradientSide::Top)
    ));
    assert!(matches!(
        linear.stops().items(),
        [
            CssColorStopListItem::Stop(first),
            CssColorStopListItem::Hint(hint),
            CssColorStopListItem::Stop(last),
        ] if first.position().is_some()
            && matches!(hint.value(), CssLength::Percent(value) if value.value() == 25.0)
            && last.position().is_none()
    ));

    let CssImageValue::Gradient(CssGradient::Radial(radial)) = &value.images().images()[3] else {
        panic!("expected radial gradient");
    };
    assert_eq!(radial.shape(), Some(CssRadialShape::Circle));
    assert!(matches!(
        radial.size(),
        Some(CssRadialSize::Extent(CssRadialExtent::ClosestSide))
    ));
    assert!(radial.position().is_some());
    assert_eq!(radial.stops().items().len(), 2);

    assert!(matches!(
        value.images().images()[4],
        CssImageValue::Gradient(CssGradient::RepeatingLinear(_))
    ));
    assert!(matches!(
        value.images().images()[5],
        CssImageValue::Gradient(CssGradient::RepeatingRadial(_))
    ));
    assert!(value.i01_subset().is_none());
}

#[test]
fn c13_radial_extent_rules_accept_only_shape_compatible_sizes() {
    for valid in [
        "radial-gradient(circle 20px, red, blue)",
        "radial-gradient(20px circle, red, blue)",
        "radial-gradient(ellipse 20% 30%, red, blue)",
        "radial-gradient(20% 30% ellipse, red, blue)",
        "radial-gradient(20px, red, blue)",
        "radial-gradient(20% 30%, red, blue)",
        "radial-gradient(circle farthest-corner, red, blue)",
        "radial-gradient(ellipse closest-side, red, blue)",
    ] {
        let source = format!("background-image: {valid}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{valid}: {:?}", report.diagnostics());
    }

    for invalid in [
        "radial-gradient(circle 20%, red, blue)",
        "radial-gradient(circle 10px 20px, red, blue)",
        "radial-gradient(ellipse 10px, red, blue)",
        "radial-gradient(20%, red, blue)",
        "radial-gradient(circle -1px, red, blue)",
        "radial-gradient(ellipse 10px -2px, red, blue)",
    ] {
        let source = format!("background-image: {invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "retained {invalid}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Color),
            "{invalid}",
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
    }
}

#[test]
fn c13_color_stop_hints_require_interleaved_stop_order() {
    for invalid in [
        "linear-gradient(red)",
        "linear-gradient(20%, red, blue)",
        "linear-gradient(red, 20%)",
        "linear-gradient(red, 20%, 30%, blue)",
        "linear-gradient(red,, blue)",
        "linear-gradient(red 20% blue)",
    ] {
        let source = format!("background-image: {invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "retained {invalid}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Color),
            "{invalid}",
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
    }
}

#[test]
fn c13_images_keep_globals_substitutions_and_exact_invalid_recovery_distinct() {
    let report = parse_style_attribute(concat!(
        "background-image: inherit; ",
        "background-image: var(--hero, linear-gradient(red, blue)); ",
        "background-image: url(hero.png)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        report.syntax()[0].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
    assert!(matches!(
        report.syntax()[1].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::BackgroundImage(url)) =
        report.syntax()[2].known().unwrap().declared_value()
    else {
        panic!("expected ordinary URL image");
    };
    assert!(matches!(url.images().images(), [CssImageValue::Url(_)]));
    assert!(url.i01_subset().is_some());

    let source = concat!(
        "--😀: kept; ",
        "background-image: radial-gradient(circle 10% at center, red, blue); ",
        "color: red"
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one invalid background-image diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let declaration_start = source.find("background-image").unwrap();
    let responsible = source.find("10%").unwrap();
    let declaration_end = source[declaration_start..].find(';').unwrap() + declaration_start + 1;
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        declaration_start
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..responsible].encode_utf16().count(),
    );
    assert!(matches!(
        diagnostic.error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::BackgroundImage
    ));

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects invalid radial size")
            .diagnostics(),
        report.diagnostics(),
    );
}
