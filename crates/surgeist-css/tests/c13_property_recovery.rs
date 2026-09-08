use surgeist_css::{
    CssErrorCode, CssImageValue, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssRecoveryAction, CssRule, ErrorKind, parse_sheet, parse_style_attribute,
};

#[derive(Clone, Copy)]
struct InvalidDeclaration {
    record: &'static str,
    property: &'static str,
    value: &'static str,
    responsible: &'static str,
    use_last: bool,
}

const C13_RECOVERY_CASES: &[InvalidDeclaration] = &[
    // The nine C13 property records.
    case(
        "official.property.border-image",
        "border-image",
        "10 // 1 2 3 4 5",
        "5",
    ),
    case(
        "official.property.border-image-outset",
        "border-image-outset",
        "10%",
        "10%",
    ),
    case(
        "official.property.border-image-repeat",
        "border-image-repeat",
        "round space stretch",
        "stretch",
    ),
    case(
        "official.property.border-image-slice",
        "border-image-slice",
        "1 2 3 4 5",
        "5",
    ),
    case(
        "official.property.border-image-source",
        "border-image-source",
        "none, url(frame.png)",
        ",",
    ),
    case(
        "official.property.border-image-width",
        "border-image-width",
        "1 2 3 4 5",
        "5",
    ),
    case(
        "official.property.image-orientation",
        "image-orientation",
        "flip 90deg",
        "90deg",
    ),
    case(
        "official.property.image-rendering",
        "image-rendering",
        "smooth",
        "smooth",
    ),
    case(
        "official.property.object-fit",
        "object-fit",
        "scale-up",
        "scale-up",
    ),
    // The eighteen C13 shared-value records, each exercised through a public property boundary.
    case(
        "official.value.background-layer",
        "background",
        "/ cover",
        "/",
    ),
    case(
        "official.value.background-image",
        "background-image",
        "none url(hero.png)",
        "url(hero.png)",
    ),
    case(
        "official.value.repeat-style",
        "background-repeat",
        "repeat-x repeat-y",
        "repeat-y",
    ),
    case(
        "official.value.background-attachment",
        "background-attachment",
        "fixed local",
        "local",
    ),
    case(
        "official.value.background-size",
        "background-size",
        "cover contain",
        "contain",
    ),
    case(
        "official.value.line-style",
        "border-style",
        "solid dotted dashed double hidden",
        "hidden",
    ),
    case(
        "official.value.line-width",
        "border-width",
        "thin medium thick 1px 2px",
        "2px",
    ),
    case(
        "official.value.image",
        "background-image",
        "url(one.png), image(two.png)",
        "image(two.png)",
    ),
    case(
        "official.value.gradient",
        "background-image",
        "linear-gradient(red)",
        "red",
    ),
    case(
        "official.value.linear-gradient",
        "background-image",
        "linear-gradient(to left right, red, blue)",
        "right",
    ),
    case(
        "official.value.radial-gradient",
        "background-image",
        "radial-gradient(circle square, red, blue)",
        "square",
    ),
    last_case(
        "official.value.repeating-linear-gradient",
        "background-image",
        "repeating-linear-gradient(red, blue,)",
        ")",
    ),
    last_case(
        "official.value.repeating-radial-gradient",
        "background-image",
        "repeating-radial-gradient(red, blue,)",
        ")",
    ),
    case(
        "official.value.color-stop-list",
        "background-image",
        "linear-gradient(red)",
        "red",
    ),
    case(
        "official.value.side-or-corner",
        "background-image",
        "linear-gradient(to top bottom, red, blue)",
        "bottom",
    ),
    case(
        "official.value.radial-shape",
        "background-image",
        "radial-gradient(circle square, red, blue)",
        "square",
    ),
    case(
        "official.value.radial-size",
        "background-image",
        "radial-gradient(circle 10%, red, blue)",
        "10%",
    ),
    case(
        "official.value.radial-extent",
        "background-image",
        "radial-gradient(circle closest, red, blue)",
        "closest",
    ),
    // Existing Partial Backgrounds/Borders rows that C13 promotes.
    last_case(
        "baseline.property.background",
        "background",
        "left / cover / contain",
        "/",
    ),
    case(
        "baseline.property.background-color",
        "background-color",
        "red blue",
        "blue",
    ),
    last_case(
        "baseline.property.background-image",
        "background-image",
        "none,, url(hero.png)",
        ",",
    ),
    case(
        "baseline.property.background-size",
        "background-size",
        "cover contain",
        "contain",
    ),
    case(
        "baseline.property.background-repeat",
        "background-repeat",
        "repeat-x repeat-y",
        "repeat-y",
    ),
    case(
        "baseline.property.background-origin",
        "background-origin",
        "border-box padding-box",
        "padding-box",
    ),
    case(
        "baseline.property.background-clip",
        "background-clip",
        "content-box border-box",
        "border-box",
    ),
    case(
        "baseline.property.background-attachment",
        "background-attachment",
        "fixed local",
        "local",
    ),
    case(
        "baseline.property.border",
        "border",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-top",
        "border-top",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-right",
        "border-right",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-bottom",
        "border-bottom",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-left",
        "border-left",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-width",
        "border-width",
        "1px 2px 3px 4px 5px",
        "5px",
    ),
    case(
        "baseline.property.border-top-width",
        "border-top-width",
        "10%",
        "10%",
    ),
    case(
        "baseline.property.border-right-width",
        "border-right-width",
        "10%",
        "10%",
    ),
    case(
        "baseline.property.border-bottom-width",
        "border-bottom-width",
        "10%",
        "10%",
    ),
    case(
        "baseline.property.border-left-width",
        "border-left-width",
        "10%",
        "10%",
    ),
    case(
        "baseline.property.border-color",
        "border-color",
        "red blue",
        "blue",
    ),
    case(
        "baseline.property.border-top-color",
        "border-top-color",
        "red blue",
        "blue",
    ),
    case(
        "baseline.property.border-right-color",
        "border-right-color",
        "red blue",
        "blue",
    ),
    case(
        "baseline.property.border-bottom-color",
        "border-bottom-color",
        "red blue",
        "blue",
    ),
    case(
        "baseline.property.border-left-color",
        "border-left-color",
        "red blue",
        "blue",
    ),
    case(
        "baseline.property.border-style",
        "border-style",
        "solid dotted dashed double hidden",
        "hidden",
    ),
    case(
        "baseline.property.border-top-style",
        "border-top-style",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-right-style",
        "border-right-style",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-bottom-style",
        "border-bottom-style",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-left-style",
        "border-left-style",
        "solid dotted",
        "dotted",
    ),
    case(
        "baseline.property.border-radius",
        "border-radius",
        "1px 2px 3px 4px 5px",
        "5px",
    ),
    case(
        "baseline.property.border-top-left-radius",
        "border-top-left-radius",
        "-1px",
        "-1px",
    ),
    case(
        "baseline.property.border-top-right-radius",
        "border-top-right-radius",
        "-1px",
        "-1px",
    ),
    case(
        "baseline.property.border-bottom-right-radius",
        "border-bottom-right-radius",
        "-1px",
        "-1px",
    ),
    case(
        "baseline.property.border-bottom-left-radius",
        "border-bottom-left-radius",
        "-1px",
        "-1px",
    ),
];

const fn case(
    record: &'static str,
    property: &'static str,
    value: &'static str,
    responsible: &'static str,
) -> InvalidDeclaration {
    InvalidDeclaration {
        record,
        property,
        value,
        responsible,
        use_last: false,
    }
}

const fn last_case(
    record: &'static str,
    property: &'static str,
    value: &'static str,
    responsible: &'static str,
) -> InvalidDeclaration {
    InvalidDeclaration {
        use_last: true,
        ..case(record, property, value, responsible)
    }
}

fn property_names(declarations: &surgeist_css::CssDeclarationList) -> Vec<&str> {
    declarations
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("unexpected future property-name kind"),
        })
        .collect()
}

fn assert_invalid_case(case: InvalidDeclaration) {
    let invalid = format!("{}: {};", case.property, case.value);
    let source = format!("--😀: kept; {invalid} color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(
        property_names(report.syntax()),
        ["--😀", "color"],
        "{}",
        case.record
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{}: expected exactly one diagnostic: {:?}",
            case.record,
            report.diagnostics()
        );
    };
    let unit_start = source.find(&invalid).expect("invalid declaration");
    let value_start = unit_start + case.property.len() + 2;
    let relative = if case.use_last {
        case.value.rfind(case.responsible)
    } else {
        case.value.find(case.responsible)
    }
    .expect("responsible token");
    let responsible = value_start + relative;

    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue,
        "{}",
        case.record
    );
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{}",
        case.record
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        unit_start,
        "{}",
        case.record
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        unit_start + invalid.len(),
        "{}",
        case.record
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible,
        "{}",
        case.record
    );
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..responsible].encode_utf16().count(),
        "{}",
        case.record,
    );
    assert!(
        matches!(
            diagnostic.error().kind(),
            ErrorKind::InvalidPropertyValue(detail)
                if detail.property().canonical_name() == case.property
        ),
        "{}: {diagnostic:#?}",
        case.record
    );

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_style_attribute(&source)
            .expect_err("strict validation rejects recovered C13 syntax")
            .diagnostics(),
        report.diagnostics(),
        "{}",
        case.record,
    );
}

#[test]
fn c13_layer_separator_recovery_preserves_siblings_and_boundaries() {
    for &case in C13_RECOVERY_CASES {
        assert_invalid_case(case);
    }

    let repeated = concat!(
        "background: red, url(hero.png); ",
        "background-image: linear-gradient(red); ",
        "border-image: 10 // 1 2 3 4 5; ",
        "color: red",
    );
    let repeated_report = parse_style_attribute(repeated);
    assert_eq!(property_names(repeated_report.syntax()), ["color"]);
    assert_eq!(repeated_report.diagnostics().len(), 3);
    for (diagnostic, property) in repeated_report.diagnostics().iter().zip([
        CssKnownProperty::Background,
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::BorderImage,
    ]) {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(matches!(
            diagnostic.error().kind(),
            ErrorKind::InvalidPropertyValue(detail) if detail.property() == property
        ));
    }

    let nested = concat!(
        ".parent { background: red, url(hero.png); color: red; ",
        "& .child { border-image: 10 // 1 2 3 4 5; width: 1px; } ",
        "background-image: url(first.png), linear-gradient(red, blue), none; } ",
        ".after { height: 2px; }",
    );
    let nested_report = parse_sheet(nested);
    assert_eq!(nested_report.diagnostics().len(), 2);
    assert!(nested_report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidPropertyValue
            && diagnostic.action() == CssRecoveryAction::DropDeclaration
    }));
    let retained = nested_report
        .syntax()
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Style(style) => Some(property_names(style.declarations())),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(retained, ["color", "width", "background-image", "height"]);

    let image_declaration = nested_report
        .syntax()
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Style(style) => style.declarations().iter().find(|declaration| {
                declaration
                    .known()
                    .is_some_and(|known| known.property() == CssKnownProperty::BackgroundImage)
            }),
            _ => None,
        })
        .next()
        .expect("retained background-image declaration");
    let CssKnownPropertyValueRef::BackgroundImage(images) =
        image_declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected background-image value");
    };
    assert!(matches!(images.images().images(), [
        CssImageValue::Url(first),
        CssImageValue::Gradient(_),
        CssImageValue::None,
    ] if first.as_str() == "first.png"));

    let eof_source = "background-image: linear-gradient(red, blue),";
    let eof_report = parse_style_attribute(eof_source);
    let [eof] = eof_report.diagnostics() else {
        panic!("trailing background layer at EOF must recover exactly once");
    };
    assert!(eof_report.syntax().is_empty());
    assert_eq!(eof.error().code(), CssErrorCode::InvalidPropertyValue);
    assert_eq!(eof.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(eof.span().start().byte_offset().value(), 0);
    assert_eq!(eof.span().end().byte_offset().value(), eof_source.len());
    assert_eq!(
        eof.error().position().byte_offset().value(),
        eof_source.len()
    );

    let implicit_source = "background-image: linear-gradient(red, blue";
    let implicit_report = parse_style_attribute(implicit_source);
    assert_eq!(
        property_names(implicit_report.syntax()),
        ["background-image"]
    );
    let [implicit] = implicit_report.diagnostics() else {
        panic!("unclosed gradient at EOF must retain one implicit closure");
    };
    assert_eq!(implicit.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        implicit.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        implicit.span().start().byte_offset().value(),
        implicit_source.len()
    );
    assert_eq!(
        implicit.span().end().byte_offset().value(),
        implicit_source.len()
    );

    for depth in [255_usize, 256] {
        let value = format!(
            "var(--image, {}none{})",
            "f(".repeat(depth - 1),
            ")".repeat(depth - 1),
        );
        let source = format!("background-image: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(
            property_names(report.syntax()),
            ["background-image", "color"]
        );
    }

    let depth = 257_usize;
    let value = format!(
        "var(--image, {}none{})",
        "f(".repeat(depth - 1),
        ")".repeat(depth - 1),
    );
    let invalid = format!("background-image: {value};");
    let source = format!("{invalid} color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(property_names(report.syntax()), ["color"]);
    let [diagnostic] = report.diagnostics() else {
        panic!("depth 257 must produce one bounded diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), invalid.len());
    let first_over_limit = source.match_indices("f(").nth(255).unwrap().0;
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        first_over_limit
    );

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_style_attribute(&source)
            .expect_err("strict validation rejects C13 nesting overflow")
            .diagnostics(),
        report.diagnostics(),
    );
}
