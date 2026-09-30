use surgeist_css::{
    CssBackgroundAttachment, CssBackgroundBox, CssBackgroundBoxList, CssBackgroundLayerBoxes,
    CssBackgroundRepeat, CssBackgroundRepeatStyle, CssBackgroundSize, CssBackgroundSizeComponent,
    CssCalcLength, CssErrorCode, CssGlobalKeyword, CssGradient, CssHorizontalPosition,
    CssImageValue, CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLength,
    CssRecoveryAction, CssVerticalPosition, ErrorKind, parse_style_attribute,
};

#[test]
fn c13_background_layers_retain_typed_structure() {
    let report = parse_style_attribute(concat!(
        "background: url(hero.png) left 10px top 20px / 40px auto ",
        "no-repeat fixed padding-box content-box, ",
        "linear-gradient(red, blue) center / cover repeat-y local border-box #123456",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Background),
    );

    let CssKnownPropertyValueRef::Background(value) = report.syntax()[0]
        .known()
        .expect("known background")
        .property_value()
        .expect("ordinary background")
    else {
        panic!("expected background wrapper");
    };
    let [first, second] = value.background().layers() else {
        panic!("expected two authored background layers");
    };
    assert!(matches!(
        first.image(),
        Some(CssImageValue::Url(url)) if url.as_str() == "hero.png"
    ));
    let first_position = first.position().expect("first position");
    assert!(matches!(
        first_position.horizontal(),
        CssHorizontalPosition::LeftOffset(offset)
            if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
    ));
    assert!(matches!(
        first_position.vertical(),
        CssVerticalPosition::TopOffset(offset)
            if matches!(offset.value(), CssLength::Px(value) if value.value() == 20.0)
    ));
    assert!(matches!(
        first.size(),
        Some(CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(CssLength::Px(width)),
            height: Some(CssBackgroundSizeComponent::Auto),
        }) if width.value() == 40.0
    ));
    assert_eq!(
        first.repeat(),
        Some(CssBackgroundRepeat::Axes {
            x: CssBackgroundRepeatStyle::NoRepeat,
            y: CssBackgroundRepeatStyle::NoRepeat,
        })
    );
    assert_eq!(first.attachment(), Some(CssBackgroundAttachment::Fixed));
    assert_eq!(
        first.boxes(),
        Some(CssBackgroundLayerBoxes::OriginAndClip {
            origin: CssBackgroundBox::PaddingBox,
            clip: CssBackgroundBox::ContentBox,
        })
    );
    assert!(first.color().is_none());

    assert!(matches!(second.image(), Some(CssImageValue::Gradient(_))));
    assert!(matches!(
        second.position().map(|position| position.horizontal()),
        Some(CssHorizontalPosition::Center)
    ));
    assert!(matches!(second.size(), Some(CssBackgroundSize::Cover)));
    assert_eq!(second.repeat(), Some(CssBackgroundRepeat::RepeatY));
    assert_eq!(second.attachment(), Some(CssBackgroundAttachment::Local));
    assert_eq!(
        second.boxes(),
        Some(CssBackgroundLayerBoxes::One(CssBackgroundBox::BorderBox))
    );
    assert_eq!(
        second
            .color()
            .and_then(|color| color.hex_value())
            .map(|color| color.digits()),
        Some("123456")
    );
}

#[test]
fn background_longhands_preserve_ordered_semantic_lists() {
    assert!(CssBackgroundBoxList::try_new(Vec::new()).is_none());

    let report = parse_style_attribute(concat!(
        "background-image: none, linear-gradient(red, blue); ",
        "background-position: left top, right 10px bottom; ",
        "background-size: cover, 10px auto; ",
        "background-repeat: repeat-x, no-repeat round; ",
        "background-origin: border-box, content-box; ",
        "background-clip: padding-box; ",
        "background-attachment: fixed, local",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let values = report
        .syntax()
        .iter()
        .map(|declaration| {
            declaration
                .known()
                .expect("known background longhand")
                .property_value()
                .expect("ordinary background longhand")
        })
        .collect::<Vec<_>>();

    let CssKnownPropertyValueRef::BackgroundImage(images) = values[0] else {
        panic!("expected background-image");
    };
    assert_eq!(images.images().images().len(), 2);
    assert!(matches!(
        images.images().images(),
        [
            CssImageValue::None,
            CssImageValue::Gradient(CssGradient::Linear(_))
        ]
    ));

    let CssKnownPropertyValueRef::BackgroundPosition(positions) = values[1] else {
        panic!("expected background-position");
    };
    assert_eq!(positions.positions().positions().len(), 2);
    assert!(matches!(
        positions.positions().positions()[0].horizontal(),
        CssHorizontalPosition::Left
    ));
    assert!(matches!(
        positions.positions().positions()[0].vertical(),
        CssVerticalPosition::Top
    ));
    assert!(matches!(positions.positions().positions()[1].horizontal(),
        CssHorizontalPosition::RightOffset(offset)
            if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)));
    assert!(matches!(
        positions.positions().positions()[1].vertical(),
        CssVerticalPosition::Bottom
    ));

    let CssKnownPropertyValueRef::BackgroundSize(sizes) = values[2] else {
        panic!("expected background-size");
    };
    assert!(matches!(sizes.sizes().sizes(), [
        CssBackgroundSize::Cover,
        CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(CssLength::Px(length)),
            height: Some(CssBackgroundSizeComponent::Auto),
        },
    ] if length.value() == 10.0));

    let CssKnownPropertyValueRef::BackgroundRepeat(repeats) = values[3] else {
        panic!("expected background-repeat");
    };
    assert!(matches!(
        repeats.repeats().repeats(),
        [
            CssBackgroundRepeat::RepeatX,
            CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::NoRepeat,
                y: CssBackgroundRepeatStyle::Round,
            },
        ]
    ));

    let CssKnownPropertyValueRef::BackgroundOrigin(origin) = values[4] else {
        panic!("expected background-origin");
    };
    assert_eq!(
        origin.boxes().boxes(),
        [CssBackgroundBox::BorderBox, CssBackgroundBox::ContentBox]
    );

    let CssKnownPropertyValueRef::BackgroundClip(clip) = values[5] else {
        panic!("expected background-clip");
    };
    assert_eq!(clip.boxes().boxes(), [CssBackgroundBox::PaddingBox]);

    let CssKnownPropertyValueRef::BackgroundAttachment(attachments) = values[6] else {
        panic!("expected background-attachment");
    };
    assert_eq!(
        attachments.attachments().attachments(),
        [
            CssBackgroundAttachment::Fixed,
            CssBackgroundAttachment::Local,
        ]
    );
}

#[test]
fn mask_size_and_repeat_expose_ordered_symbolic_lists() {
    let report = parse_style_attribute(concat!(
        "mask-size: cover, calc(10% + 2px) auto; ",
        "mask-repeat: repeat-x, no-repeat round; ",
        "mask-size: inherit; mask-repeat: var(--repeat)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::MaskSize(sizes)) =
        report.syntax()[0].known().unwrap().declared_value()
    else {
        panic!("ordinary mask-size list");
    };
    let [
        CssBackgroundSize::Cover,
        CssBackgroundSize::Explicit { width, height },
    ] = sizes.sizes().sizes()
    else {
        panic!("ordered cover and symbolic explicit mask sizes");
    };
    assert!(
        matches!(width, CssBackgroundSizeComponent::Length(CssLength::Calc(CssCalcLength::Typed(calc)))
        if calc.components().serialize().unwrap().as_css() == "calc(10% + 2px)")
    );
    assert!(matches!(height, Some(CssBackgroundSizeComponent::Auto)));

    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::MaskRepeat(repeats)) =
        report.syntax()[1].known().unwrap().declared_value()
    else {
        panic!("ordinary mask-repeat list");
    };
    assert!(matches!(
        repeats.repeats().repeats(),
        [
            CssBackgroundRepeat::RepeatX,
            CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::NoRepeat,
                y: CssBackgroundRepeatStyle::Round,
            },
        ]
    ));
    assert!(matches!(
        report.syntax()[2].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
    assert!(matches!(
        report.syntax()[3].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

#[test]
fn background_globals_substitutions_and_layer_colors_remain_distinct() {
    let report = parse_style_attribute(concat!(
        "background: inherit; ",
        "background: var(--surface, url(hero.png) center / cover); ",
        "background: red; ",
        "background: url(hero.png)",
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

    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::Background(color_only)) =
        report.syntax()[2].known().unwrap().declared_value()
    else {
        panic!("expected color-only background");
    };
    assert_eq!(
        color_only.background().layers()[0]
            .color()
            .unwrap()
            .named()
            .unwrap()
            .name(),
        "red"
    );

    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::Background(image_only)) =
        report.syntax()[3].known().unwrap().declared_value()
    else {
        panic!("expected image-only background");
    };
    assert!(image_only.background().layers()[0].image().is_some());
    assert!(image_only.background().layers()[0].color().is_none());
}

#[test]
fn background_shared_longhands_keep_global_and_substitution_branches() {
    for (property, global, substitution) in [
        ("background-image", "inherit", "var(--image)"),
        ("background-size", "initial", "var(--size)"),
        ("background-repeat", "unset", "var(--repeat)"),
        ("background-attachment", "revert", "var(--attachment)"),
        ("background-origin", "revert-layer", "var(--origin)"),
        ("background-clip", "inherit", "var(--clip)"),
    ] {
        let source = format!("{property}: {global}; {property}: {substitution}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [global_value, substitution_value] = report.syntax().as_slice() else {
            panic!("{source}: expected two declarations");
        };
        assert!(matches!(
            global_value.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::Global(_)
        ));
        assert!(matches!(
            substitution_value.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(_)
        ));
    }
}

#[test]
fn background_layer_mutations_drop_exact_declaration_and_keep_siblings() {
    let cases = [
        ("background", "/ cover", "/"),
        ("background", "cover", "cover"),
        ("background", "left no-repeat / cover", "/"),
        ("background", "red, url(hero.png)", "red"),
        (
            "background",
            "url(hero.png) padding-box content-box border-box",
            "border-box",
        ),
        ("background", "url(hero.png) left / cover / contain", "/"),
        ("background-image", "none url(hero.png)", "url(hero.png)"),
        ("background-size", "cover contain", "contain"),
        ("background-repeat", "repeat-x repeat-y", "repeat-y"),
        ("background-attachment", "fixed local", "local"),
        ("background-origin", "border-box padding-box", "padding-box"),
        ("background-clip", "content-box border-box", "border-box"),
    ];

    for (property, invalid, responsible_text) in cases {
        let declaration = format!("{property}: {invalid};");
        let source = format!("--😀: kept; {declaration} color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        let declaration_start = source.find(&declaration).unwrap();
        let declaration_end = declaration_start + declaration.len();
        let value_start = declaration_start + property.len() + 2;
        let responsible = if responsible_text == "/" && invalid.matches('/').count() > 1 {
            value_start + invalid.rfind(responsible_text).unwrap()
        } else {
            value_start + invalid.find(responsible_text).unwrap()
        };
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            declaration_start,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            declaration_end,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[..responsible].encode_utf16().count(),
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-specific diagnostic");
        };
        assert_eq!(detail.property().canonical_name(), property, "{source}");

        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects invalid background syntax")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

#[test]
fn background_empty_comma_items_and_eof_report_exact_recovery() {
    for (invalid, responsible) in [
        (
            "background: url(hero.png),;",
            "background: url(hero.png),".len(),
        ),
        (
            "background: url(hero.png),, none;",
            "background: url(hero.png),".len(),
        ),
        (
            "background-origin: border-box,;",
            "background-origin: border-box,".len(),
        ),
        ("background-image: none,;", "background-image: none,".len()),
    ] {
        let source = format!("{invalid} color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), invalid.len());
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[..responsible].encode_utf16().count(),
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(_) = diagnostic.error().kind() else {
            panic!("{source}: expected property-specific diagnostic");
        };
    }
}
