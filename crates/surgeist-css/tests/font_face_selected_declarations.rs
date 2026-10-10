#![forbid(unsafe_code)]
//! Independent public consumer oracles for #1086; no preimplementation RED.
use CssFontFaceDescriptorKind as K;
use CssSpecifiedValueSerializationErrorKind as E;
use CssSpecifiedValueSerializationLimits as L;
use surgeist_css::*;

fn entries(source: &str) -> Vec<CssFontFaceDescriptor> {
    parse_font_face_declaration_block_contents(source)
        .syntax()
        .as_ref()
        .unwrap()
        .occurrences()
        .cloned()
        .collect()
}
fn cause(error: CssFontFaceDeclarationBlockError, expected: E) {
    let CssFontFaceDeclarationBlockError::Serialization(error) = error else {
        panic!("typed scalar error")
    };
    assert_eq!(error.kind(), expected);
}

#[test]
fn public_descriptor_lookup_uses_exact_decoded_names_case_and_existing_alias() {
    use CssPageDescriptorKind as P;
    for (name, kind) in [
        ("SIZE", P::Size),
        ("page-orientation", P::PageOrientation),
        ("Marks", P::Marks),
        ("bleed", P::Bleed),
    ] {
        assert_eq!(P::from_name(name), Some(kind));
    }
    for (name, kind) in [
        ("FONT-FAMILY", K::FontFamily),
        ("src", K::Src),
        ("font-weight", K::FontWeight),
        ("font-style", K::FontStyle),
        ("font-width", K::FontWidth),
        ("FoNt-StReTcH", K::FontWidth),
        ("font-display", K::FontDisplay),
        ("unicode-range", K::UnicodeRange),
        ("font-feature-settings", K::FontFeatureSettings),
        ("font-variation-settings", K::FontVariationSettings),
        ("font-named-instance", K::FontNamedInstance),
        ("font-language-override", K::FontLanguageOverride),
        ("ascent-override", K::AscentOverride),
        ("descent-override", K::DescentOverride),
        ("line-gap-override", K::LineGapOverride),
    ] {
        assert_eq!(K::from_css_name(name), Some(kind));
    }
    for name in [
        "",
        " size",
        "size ",
        "size:A4",
        "\\73 ize",
        "--size",
        "width",
        "font-family",
    ] {
        assert_eq!(P::from_name(name), None, "{name}");
    }
    for name in [
        "",
        " src",
        "src ",
        "src:local(X)",
        "\\73 rc",
        "--src",
        "size",
        "font-size",
    ] {
        assert_eq!(K::from_css_name(name), None, "{name}");
    }
}

#[test]
fn selected_order_and_edits_are_distinct_from_whole_font_rule_order() {
    let authored = entries("font-family:A; src:local(A); font-display:swap");
    let raw = parse_font_face_descriptor_value("B", K::FontFamily);
    let family = CssFontFaceDescriptor::new(raw.syntax().as_ref().unwrap().clone());
    assert!(family.position().is_none());
    let selected = CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&[
        authored[2].clone(),
        family,
        authored[1].clone(),
    ])
    .unwrap();
    assert_eq!(
        selected
            .entries()
            .iter()
            .map(|d| d.value().kind())
            .collect::<Vec<_>>(),
        [K::FontDisplay, K::FontFamily, K::Src]
    );
    assert_eq!(
        selected.serialize_cssom().unwrap(),
        "font-display: swap; font-family: B; src: local(\"A\");"
    );
    assert_eq!(
        CssRule::FontFace(CssFontFaceRule::new(CssFontFaceDescriptors::new(
            selected.entries().to_vec()
        )))
        .to_specified_css()
        .unwrap(),
        "@font-face { font-family: B; src: local(\"A\"); font-display: swap; }"
    );
    assert_eq!(authored[0].value().serialize_specified().unwrap(), "A");
}

#[test]
fn selected_membership_rejects_raw_duplicates_and_alias_duplicates() {
    for (source, kind) in [
        ("font-display:swap; font-display:optional", K::FontDisplay),
        ("font-stretch:75%; font-width:125%", K::FontWidth),
    ] {
        let raw = entries(source);
        assert!(
            matches!(CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&raw),
            Err(CssFontFaceDeclarationBlockError::DuplicateDescriptor { kind: found }) if found == kind)
        );
        let selected = CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&raw[1..]).unwrap();
        assert_eq!(selected.entries().len(), 1);
    }
    let empty = CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&[]).unwrap();
    assert!(empty.entries().is_empty());
    assert_eq!(
        empty.serialize_cssom_with_limits(L::new(1, 1, 0)).unwrap(),
        ""
    );
    cause(
        empty
            .serialize_cssom_with_limits(L::new(0, 1, 0))
            .unwrap_err(),
        E::InputNodeLimit,
    );
}

#[test]
fn ordinary_and_supported_pending_scalars_keep_their_owned_grammar_and_text() {
    let values = entries("font-weight:400 700; font-display:optional; font-width:env(width, 75%)");
    assert_eq!(values[0].value().serialize_specified().unwrap(), "400 700");
    assert_eq!(values[1].value().serialize_specified().unwrap(), "optional");
    assert_eq!(
        values[2].value().serialize_specified().unwrap(),
        "env(width, 75%)"
    );
    let CssAuthoredFontFaceDescriptorValue::Pending(pending) = values[2].value() else {
        panic!()
    };
    assert_eq!(pending.serialize_specified().unwrap(), "env(width, 75%)");
    assert_eq!(
        pending
            .serialize_specified_with_limits(L::new(100, 100, 15))
            .unwrap(),
        "env(width, 75%)"
    );
    assert_eq!(
        pending
            .serialize_specified_with_limits(L::new(100, 100, 14))
            .unwrap_err()
            .kind(),
        E::ByteLimit
    );
}

#[test]
fn selected_occurrence_positions_and_pending_origins_survive_input_drop() {
    let source = "/*😀*/\r\nfont-weight:env(weight); font-display:swap";
    let raw = {
        let input = source.to_owned();
        entries(&input)
    };
    let selected =
        CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&[raw[1].clone(), raw[0].clone()])
            .unwrap();
    assert_eq!(
        selected.entries()[1]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        10
    );
    assert_eq!(selected.entries()[1].position().unwrap().line().value(), 1);
    let CssAuthoredFontFaceDescriptorValue::Pending(pending) = selected.entries()[1].value() else {
        panic!()
    };
    let function = pending
        .components()
        .items()
        .iter()
        .find(|v| matches!(v.view(), CssComponentValueRef::Function(_)))
        .unwrap();
    let CssValueOrigin::Parsed(origin) = function.origin() else {
        panic!()
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 22);
    assert_eq!(
        selected.serialize_cssom().unwrap(),
        "font-display: swap; font-weight: env(weight);"
    );
    assert_eq!(raw[0].value().serialize_specified().unwrap(), "env(weight)");
}

#[test]
fn selected_values_and_punctuation_share_exact_cumulative_allowances_and_retry() {
    let raw = entries("font-display:swap; font-weight:env(weight)");
    let expected = "font-display: swap; font-weight: env(weight);";
    // Block + two entries + display scalar + pending list/function/ident = seven.
    let adequate = L::new(7, 7, expected.len());
    let selected =
        CssSpecifiedFontFaceDeclarationBlock::try_from_entries_with_limits(&raw, adequate).unwrap();
    assert_eq!(
        selected.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(6, 7, expected.len()), E::InputNodeLimit),
        (L::new(7, 6, expected.len()), E::ProjectionNodeLimit),
        (L::new(7, 7, expected.len() - 1), E::ByteLimit),
    ] {
        cause(
            selected.serialize_cssom_with_limits(limits).unwrap_err(),
            kind,
        );
        assert_eq!(
            selected.serialize_cssom_with_limits(adequate).unwrap(),
            expected
        );
        assert_eq!(
            selected.entries()[1].value().serialize_specified().unwrap(),
            "env(weight)"
        );
    }
    for (limits, kind) in [
        (L::new(6, 7, 0), E::InputNodeLimit),
        (L::new(7, 6, 0), E::ProjectionNodeLimit),
    ] {
        cause(
            CssSpecifiedFontFaceDeclarationBlock::try_from_entries_with_limits(&raw, limits)
                .unwrap_err(),
            kind,
        );
        assert_eq!(
            CssSpecifiedFontFaceDeclarationBlock::try_from_entries_with_limits(&raw, adequate)
                .unwrap()
                .serialize_cssom()
                .unwrap(),
            expected
        );
    }
    let value = raw[1].value();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(3, 3, 11))
            .unwrap(),
        "env(weight)"
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(2, 3, 11))
            .unwrap_err()
            .kind(),
        E::InputNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(3, 2, 11))
            .unwrap_err()
            .kind(),
        E::ProjectionNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(3, 3, 10))
            .unwrap_err()
            .kind(),
        E::ByteLimit
    );
    assert_eq!(value.serialize_specified().unwrap(), "env(weight)");
}
