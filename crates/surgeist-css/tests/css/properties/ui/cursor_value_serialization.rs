#![forbid(unsafe_code)]
//! Functional coverage for the direct cursor model and its new specified provider.
//! UI 4 WD 2026-01-20 §5.1.1 owns the complete authored Number pair and fallback.
//! Images 4 WD 2025-09-30 §§2.4, 8 and Appendix A own URL/string alternatives,
//! descriptor grammar/order, omission semantics and the standard prefixed alias.
//! Selection and clamping are downstream. New APIs have functional tests, not
//! a missing-symbol RED; the unchanged admission contract owns prior executable RED.

use CssSpecifiedValueSerializationErrorKind as Kind;
use CssSpecifiedValueSerializationLimits as Limits;
use surgeist_css::*;

fn number(value: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_token(value).unwrap()).unwrap()
}

fn resolution(value: &str, unit: CssResolutionUnit) -> CssResolutionValue {
    CssResolutionValue::from_literal(CssResolutionLiteral::try_new(value, unit).unwrap())
}

fn string(value: &str) -> CssContentString {
    CssContentString::try_new(value).unwrap()
}

fn parsed(value: &str) -> (CssDeclaration, CssCursor) {
    let source = format!("/* 🦀 */ cursor:{value} !important; color:blue");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration, neighbor] = report.syntax().as_slice() else {
        panic!("cursor and its neighbor")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        neighbor.known().unwrap().property(),
        CssKnownProperty::Color
    );
    let CssKnownPropertyValueRef::Cursor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed cursor")
    };
    (declaration.clone(), value.value().clone())
}

fn images(value: &CssCursor) -> &CssCursorImages {
    let CssCursor::Images(images) = value else {
        panic!("image alternatives")
    };
    images
}

fn set(value: &CssCursor) -> &CssCursorUrlSet {
    let CssCursorImageSource::UrlSet(set) = images(value).images()[0].source() else {
        panic!("URL image set")
    };
    set
}

fn assert_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed scalar")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find(token).unwrap();
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + token.len()
    );
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
}

#[test]
fn checked_constructors_reject_empty_lists_and_duplicate_descriptor_kinds() {
    assert!(CssCursorImages::try_new(Vec::new(), CssCursorKeyword::Auto).is_none());
    assert!(CssCursor::try_images(Vec::new(), CssCursorKeyword::Auto).is_none());
    assert!(CssCursorUrlSet::try_new(Vec::new()).is_none());
    let reference = CssCursorUrlSetReference::Url(CssUrl::new("cursor.png"));
    let res = CssCursorUrlSetDescriptor::Resolution(resolution("1", CssResolutionUnit::Dppx));
    let ty = CssCursorUrlSetDescriptor::Type(string("image/png"));
    for descriptors in [
        vec![res.clone(), res.clone()],
        vec![ty.clone(), ty.clone()],
        vec![res.clone(), ty.clone(), res.clone()],
    ] {
        assert!(CssCursorUrlSetOption::try_new(reference.clone(), descriptors).is_none());
    }
    for descriptors in [
        vec![],
        vec![res.clone()],
        vec![ty.clone()],
        vec![res.clone(), ty.clone()],
        vec![ty, res],
    ] {
        assert!(CssCursorUrlSetOption::try_new(reference.clone(), descriptors).is_some());
    }
    assert!(CssContentString::try_new("bad\0string").is_none());
}

#[test]
fn programmatic_pair_is_complete_signed_and_unclamped_with_ordinary_url_semantics() {
    let pair = [number("-3.500"), number("1e999")];
    let entry = CssCursorImage::new(
        CssCursorImageSource::Url(CssUrl::new("cursor.cur")),
        Some(pair.clone()),
    );
    assert_eq!(entry.hotspot(), Some(&pair));
    for number in entry.hotspot().unwrap() {
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    }
    let value = CssCursor::try_images(vec![entry], CssCursorKeyword::Pointer).unwrap();
    assert_eq!(images(&value).fallback(), CssCursorKeyword::Pointer);
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(7, 7, 30))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, before);
}

#[test]
fn parsed_hotspots_retain_exact_coefficients_and_original_positions_after_emission() {
    let (declaration, value) = parsed("url(cursor.cur) 3.000e1 -0.25, pointer");
    let source = declaration.parsed_name().unwrap().source().as_str();
    let pair = images(&value).images()[0].hotspot().unwrap();
    for (number, token) in pair.iter().zip(["3.000e1", "-0.25"]) {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(numeric)) =
            number.literal_component().unwrap().view()
        else {
            panic!("ordinary Number")
        };
        assert_eq!(numeric.representation(), token);
        assert_origin(number.origin(), source, token);
    }
    let before = value.clone();
    let components = declaration.value_components().clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "url(\"cursor.cur\") 30 -0.25, pointer"
    );
    assert_eq!(value, before);
    assert_eq!(declaration.value_components(), &components);
}

#[test]
fn number_math_hotspots_remain_math_in_the_model_and_use_shared_specified_projection() {
    let (_, value) = parsed("url(cursor.cur) calc(1 + 2) calc(-0.25), auto");
    let pair = images(&value).images()[0].hotspot().unwrap();
    assert!(pair[0].calculation().is_some());
    assert!(pair[1].calculation().is_some());
    let before = value.clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "url(\"cursor.cur\") calc(3) calc(-0.25), auto"
    );
    assert_eq!(value, before);
}

#[test]
fn omission_and_explicit_zero_hotspots_remain_distinct_in_model_and_output() {
    let (_, omitted) = parsed("url(cursor.cur), auto");
    let (_, explicit) = parsed("url(cursor.cur) 0 -0, auto");
    assert!(images(&omitted).images()[0].hotspot().is_none());
    assert!(images(&explicit).images()[0].hotspot().is_some());
    assert_eq!(
        omitted.serialize_specified().unwrap(),
        "url(\"cursor.cur\"), auto"
    );
    assert_eq!(
        explicit.serialize_specified().unwrap(),
        "url(\"cursor.cur\") 0 0, auto"
    );
}

#[test]
fn ordered_url_and_string_candidates_keep_authored_forms_and_descriptor_order() {
    let (declaration, value) = parsed(
        "image-set(\"first.png\" type(\"image/png\") 96.000dpi, url(second.png) 2x), pointer",
    );
    let options = set(&value).options();
    assert_eq!(options.len(), 2);
    assert!(
        matches!(options[0].reference(), CssCursorUrlSetReference::String(value) if value.as_str() == "first.png")
    );
    assert!(
        matches!(options[1].reference(), CssCursorUrlSetReference::Url(value) if value.as_str() == "second.png")
    );
    assert!(matches!(
        options[0].descriptors(),
        [
            CssCursorUrlSetDescriptor::Type(_),
            CssCursorUrlSetDescriptor::Resolution(_)
        ]
    ));
    assert_eq!(options[0].image_type().unwrap().as_str(), "image/png");
    let literal = options[0].resolution().unwrap().literal().unwrap();
    assert_eq!(literal.numeric().representation(), "96.000");
    assert_eq!(literal.unit(), CssResolutionUnit::Dpi);
    assert_origin(
        literal.origin(),
        declaration.parsed_name().unwrap().source().as_str(),
        "96.000dpi",
    );
    let before = value.clone();
    let components = declaration.value_components().clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "image-set(\"first.png\" 1dppx type(\"image/png\"), url(\"second.png\") 2dppx), pointer"
    );
    assert_eq!(value, before);
    assert_eq!(declaration.value_components(), &components);
}

#[test]
fn omitted_resolution_is_preserved_and_explicit_one_is_not_discarded() {
    let (_, value) = parsed("image-set(url(first.png), url(second.png) 1x), auto");
    assert!(set(&value).options()[0].resolution().is_none());
    assert!(set(&value).options()[1].resolution().is_some());
    assert_eq!(
        value.serialize_specified().unwrap(),
        "image-set(url(\"first.png\"), url(\"second.png\") 1dppx), auto"
    );
}

#[test]
fn type_only_arbitrary_mime_and_duplicate_resolution_candidates_all_survive_emission() {
    for (source, expected) in [
        (
            "image-set(url(a.png) type(\"\")), auto",
            "image-set(url(\"a.png\") type(\"\")), auto",
        ),
        (
            "image-set(url(a.png) type(\"not a MIME type\")), auto",
            "image-set(url(\"a.png\") type(\"not a MIME type\")), auto",
        ),
        (
            "image-set(url(a.png) 1x, url(b.png) 96dpi), auto",
            "image-set(url(\"a.png\") 1dppx, url(\"b.png\") 1dppx), auto",
        ),
    ] {
        let (_, value) = parsed(source);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
    }
}

#[test]
fn negative_resolution_math_remains_unclamped_while_zero_literal_uses_ordinary_projection() {
    let (_, value) = parsed("image-set(url(a.png) -0x, url(b.png) calc(-1x)), auto");
    assert_eq!(
        set(&value).options()[0]
            .resolution()
            .unwrap()
            .literal()
            .unwrap()
            .numeric()
            .representation(),
        "-0"
    );
    assert!(
        set(&value).options()[1]
            .resolution()
            .unwrap()
            .calculation()
            .is_some()
    );
    let before = value.clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "image-set(url(\"a.png\") 0dppx, url(\"b.png\") calc(-1dppx)), auto"
    );
    assert_eq!(value, before);
}

#[test]
fn standard_alias_emits_canonical_function_without_rewriting_original_components() {
    let (declaration, value) = parsed("-WEBKIT-IMAGE-SET(\"cursor.png\" 1x), POINTER");
    let before = declaration.value_components().clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "image-set(\"cursor.png\" 1dppx), pointer"
    );
    assert_eq!(declaration.value_components(), &before);
    let function = before
        .items()
        .iter()
        .find_map(|component| match component.view() {
            CssComponentValueRef::Function(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(function.name(), "-WEBKIT-IMAGE-SET");
}

#[test]
fn checked_value_construction_keeps_supplied_numeric_origins_and_no_fabricated_declaration_position()
 {
    let input = "image-set(url(cursor.png) type(\"image/png\") 96.000dpi) -3.500 +4.250, auto";
    let supplied = parse_component_values(input).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Cursor),
        supplied.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), &supplied);
    let CssKnownPropertyValueRef::Cursor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed cursor")
    };
    let value = value.value();
    let pair = images(value).images()[0].hotspot().unwrap();
    assert_origin(pair[0].origin(), input, "-3.500");
    assert_origin(pair[1].origin(), input, "+4.250");
    assert_origin(
        set(value).options()[0].resolution().unwrap().origin(),
        input,
        "96.000dpi",
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "image-set(url(\"cursor.png\") 1dppx type(\"image/png\")) -3.5 4.25, auto"
    );
    assert_eq!(declaration.value_components(), &supplied);
}

#[test]
fn escaped_string_reference_and_type_use_existing_css_string_emission() {
    let option = CssCursorUrlSetOption::try_new(
        CssCursorUrlSetReference::String(string("a\"b\\c")),
        vec![CssCursorUrlSetDescriptor::Type(string("x\"y\\z"))],
    )
    .unwrap();
    let set = CssCursorUrlSet::try_new(vec![option]).unwrap();
    let value = CssCursor::try_images(
        vec![CssCursorImage::new(CssCursorImageSource::UrlSet(set), None)],
        CssCursorKeyword::Auto,
    )
    .unwrap();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "image-set(\"a\\\"b\\\\c\" type(\"x\\\"y\\\\z\")), auto"
    );
}

#[test]
fn every_selected_keyword_has_canonical_specified_output_and_one_node_cost() {
    for name in [
        "auto",
        "default",
        "none",
        "context-menu",
        "help",
        "pointer",
        "progress",
        "wait",
        "cell",
        "crosshair",
        "text",
        "vertical-text",
        "alias",
        "copy",
        "move",
        "no-drop",
        "not-allowed",
        "grab",
        "grabbing",
        "all-scroll",
        "col-resize",
        "row-resize",
        "n-resize",
        "e-resize",
        "s-resize",
        "w-resize",
        "ne-resize",
        "nw-resize",
        "se-resize",
        "sw-resize",
        "ew-resize",
        "ns-resize",
        "nesw-resize",
        "nwse-resize",
        "zoom-in",
        "zoom-out",
    ] {
        let (_, value) = parsed(&name.to_ascii_uppercase());
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 1, name.len()))
                .unwrap(),
            name
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(0, 1, name.len()))
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 0, name.len()))
                .unwrap_err()
                .kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 1, name.len() - 1))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
    }
}

#[test]
fn exact_public_node_projection_and_byte_boundaries_follow_the_documented_model_cost() {
    for (source, expected, nodes) in [
        (
            "url(cursor.cur) 30 -0.25, pointer",
            "url(\"cursor.cur\") 30 -0.25, pointer",
            7,
        ),
        (
            "image-set(url(cursor.png) type(\"image/png\") 1dppx), auto",
            "image-set(url(\"cursor.png\") 1dppx type(\"image/png\")), auto",
            10,
        ),
        (
            "image-set(\"cursor.png\" type(\"image/png\") 1dppx), auto",
            "image-set(\"cursor.png\" 1dppx type(\"image/png\")), auto",
            9,
        ),
    ] {
        let (declaration, value) = parsed(source);
        let before = value.clone();
        let components = declaration.value_components().clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(value, before);
        assert_eq!(declaration.value_components(), &components);
    }
}
