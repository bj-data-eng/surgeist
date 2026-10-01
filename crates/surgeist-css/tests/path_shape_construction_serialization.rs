#![forbid(unsafe_code)]
//! Functional new API evidence, without a claimed preimplementation API RED.
//! SVG 1.1 §8.3.9, adopted Appendix F.6.2 admission, and the path resource
//! contract independently define these expectations; geometry stays unresolved.

use surgeist_css::{
    CssPathDataConstructionErrorKind as E, CssSpecifiedValueSerializationErrorKind as K,
    CssSpecifiedValueSerializationLimits as L, *,
};

#[test]
fn decoded_grammar_errors_locate_the_first_failure_or_incomplete_end() {
    for (data, offset) in [
        ("L0 0", 0),
        ("M", 1),
        ("M0", 2),
        ("M0 0L1", 6),
        ("M0 0L1e+", 8),
        ("M0 0L1e+?", 8),
        ("M0 0X1 2", 4),
        ("M0 0,", 4),
        ("M0 0,L1 2", 4),
        ("M0,,0", 3),
        ("M0 0A1 2 0+0 1 3 4", 10),
        ("M0 0A1 2 0 2 1 3 4", 11),
        ("M0 0L١ 2", 5),
        ("M0 0\u{a0}", 4),
    ] {
        let error = CssPathData::try_new(data).unwrap_err();
        assert_eq!(error.kind(), E::InvalidPathData, "{data}");
        assert_eq!(error.decoded_byte_offset(), Some(offset), "{data}");
        assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
        assert!(error.to_string().contains(&offset.to_string()));
    }
    for data in ["", " \t\r\n "] {
        let error = CssPathData::try_new(data).unwrap_err();
        assert_eq!(error.kind(), E::EmptyPath);
        assert_eq!(error.decoded_byte_offset(), None);
    }
}

#[test]
fn decoded_identity_ignores_token_provenance_but_preserves_svg_spelling() {
    let source = "'M\\30  0'";
    let components = parse_component_values(source).unwrap();
    let component = components.items()[0].clone();
    let parsed = CssPathData::try_from_component(component.clone()).unwrap();
    assert_eq!(parsed.as_str(), "M0 0");
    assert_eq!(parsed.origin(), component.origin());
    let programmatic = CssPathData::try_new("M0 0").unwrap();
    assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(parsed, programmatic);
    for spelling in ["M0,0", "m0 0", "M0.0 0", " M0 0 "] {
        assert_ne!(parsed, CssPathData::try_new(spelling).unwrap());
    }
    let quote =
        CssPathData::try_from_component(CssComponentValue::try_token("\"M0 0\"").unwrap()).unwrap();
    assert_eq!(parsed, quote);
}

#[test]
fn component_construction_rejects_non_string_and_recovered_string_with_original_origins() {
    for component in [
        CssComponentValue::try_ident("M0").unwrap(),
        CssComponentValue::try_token("1").unwrap(),
        CssComponentValue::try_function("path", CssComponentValues::try_new(vec![]).unwrap())
            .unwrap(),
    ] {
        let origin = component.origin().clone();
        let error = CssPathData::try_from_component(component).unwrap_err();
        assert_eq!(error.kind(), E::ExpectedString);
        assert_eq!(error.origin(), &origin);
        assert_eq!(error.decoded_byte_offset(), None);
    }
    for source in ["'M0 0", "'M0 0\\"] {
        let values = parse_component_values(source).unwrap();
        let error = CssPathData::try_from_component(values.items()[0].clone()).unwrap_err();
        assert_eq!(error.kind(), E::RecoveredInput);
        assert_eq!(error.decoded_byte_offset(), None);
        let CssValueOrigin::ImplicitClosure { at, .. } = error.origin() else {
            panic!("implicit quote");
        };
        assert_eq!(at.source().as_str(), source);
        assert_eq!(at.span().start().byte_offset().value(), source.len());
        assert_eq!(at.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn escaped_component_error_offsets_remain_decoded_and_do_not_shift_source_spans() {
    let source = "'M\\30  0L1e+'";
    let component = parse_component_values(source).unwrap().items()[0].clone();
    let origin = component.origin().clone();
    let error = CssPathData::try_from_component(component).unwrap_err();
    assert_eq!(error.kind(), E::InvalidPathData);
    assert_eq!(error.decoded_byte_offset(), Some(8));
    assert_eq!(error.origin(), &origin);
    let CssValueOrigin::Parsed(parsed) = error.origin() else {
        panic!("parsed quote");
    };
    assert_eq!(parsed.span().start().byte_offset().value(), 0);
    assert_eq!(parsed.span().end().byte_offset().value(), source.len());
    let empty = parse_component_values("''").unwrap().items()[0].clone();
    let origin = empty.origin().clone();
    let error = CssPathData::try_from_component(empty).unwrap_err();
    assert_eq!(error.kind(), E::EmptyPath);
    assert_eq!(error.origin(), &origin);
    assert_eq!(error.decoded_byte_offset(), None);
}

fn budget(
    expected: &str,
    nodes: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(nodes, nodes, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(nodes - 1, nodes, expected.len()), K::InputNodeLimit),
        (
            L::new(nodes, nodes - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(nodes, nodes, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(serialize(L::default()).unwrap(), expected);
}

#[test]
fn path_composition_and_css_escaping_obey_one_atomic_cumulative_budget() {
    let component = parse_component_values("' M0\\9 0\\d L1\\a 2 '")
        .unwrap()
        .items()[0]
        .clone();
    let data = CssPathData::try_from_component(component).unwrap();
    let origin = data.origin().clone();
    let omitted = CssPathShape::new(None, data.clone());
    assert_eq!(omitted.fill_rule(), None);
    assert_eq!(omitted.data(), &data);
    for (fill, prefix, nodes) in [
        (None, "", 2),
        (Some(CssFillRule::Nonzero), "nonzero, ", 3),
        (Some(CssFillRule::Evenodd), "evenodd, ", 3),
    ] {
        let shape = CssPathShape::new(fill, data.clone());
        let expected = format!("path({prefix}\" M0\\9 0\\d L1\\a 2 \")");
        assert_eq!(shape.fill_rule(), fill);
        if fill.is_some() {
            assert_ne!(shape, omitted);
        }
        budget(&expected, nodes, |limits| {
            shape.serialize_specified_with_limits(limits)
        });
        let basic = CssBasicShape::Path(shape.clone());
        budget(&expected, nodes, |limits| {
            basic.serialize_specified_with_limits(limits)
        });
        let clip = CssClipPathShape::new(basic.clone(), None);
        budget(&expected, nodes + 1, |limits| {
            clip.serialize_specified_with_limits(limits)
        });
        let clip = CssClipPath::BasicShape(CssClipPathShape::new(
            basic,
            Some(CssBoxEdgeKeyword::BorderBox),
        ));
        budget(&format!("{expected} border-box"), nodes + 2, |limits| {
            clip.serialize_specified_with_limits(limits)
        });
        assert_eq!(shape.data().origin(), &origin);
        assert_eq!(shape.data().as_str(), " M0\t0\rL1\n2 ");
    }
}

#[test]
fn arbitrarily_large_valid_decimal_spellings_have_no_constructor_float_or_byte_ceiling() {
    let huge = format!(
        "M{}e{} -1e-{}",
        "9".repeat(100_000),
        "9".repeat(100_000),
        "9".repeat(100_000)
    );
    let data = CssPathData::try_new(huge.clone()).unwrap();
    assert_eq!(data.as_str(), huge);
    for spelling in ["M0 0A-1 +2 0 0110-20", "M1.2.3L1. 2.", "M0 0ZzM1 2"] {
        assert_eq!(CssPathData::try_new(spelling).unwrap().as_str(), spelling);
    }
}

#[test]
fn required_svg_imports_pin_only_path_definitions_without_changing_selected_modules() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../specs/catalog.json")).unwrap();
    assert_eq!(catalog["modules"].as_array().unwrap().len(), 79);
    for (id, url, hash, bytes, anchor) in [
        (
            "svg11-path-data-grammar",
            "https://www.w3.org/TR/2011/REC-SVG11-20110816/paths.html",
            "bd8e7b9789962497a6b2ca2b8012a4f824905a89b40beaa3b29cc9eabf5d6fc7",
            180140,
            "PathDataBNF",
        ),
        (
            "svg11-path-parsing-interpretation",
            "https://www.w3.org/TR/2011/REC-SVG11-20110816/implnote.html",
            "8c6edf7d7d52aa3d8c11f38276d090105f2504fd557edc2f5d4348e58d58e3e5",
            34617,
            "ArcOutOfRangeParameters",
        ),
    ] {
        let sources = catalog["normative_definitions"]["sources"]
            .as_array()
            .unwrap();
        let source = sources.iter().find(|source| source["id"] == id).unwrap();
        assert_eq!(source["url"], url);
        assert_eq!(source["sha256"], hash);
        assert_eq!(source["bytes"], bytes);
        let definitions = catalog["normative_definitions"]["definitions"]
            .as_array()
            .unwrap();
        let definition = definitions
            .iter()
            .find(|definition| definition["source_id"] == id)
            .unwrap();
        assert_eq!(definition["authored_owner"], "css");
        assert!(
            definition["sections"]
                .as_array()
                .unwrap()
                .iter()
                .any(|section| section["anchor"] == anchor)
        );
        assert_eq!(definition["required_by"][0]["module_id"], "css-shapes-1");
    }
}
