use std::cell::Cell;
use surgeist_css::{CssParser, CssParserContext, CssPropertyGrammar};
use surgeist_cssom::*;

fn limits() -> CssomDeclarationRequestLimits {
    CssomDeclarationRequestLimits::default()
}

fn version(role: CssomInputRole, identity: &str) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.into(),
        revision: 1,
    }
}

fn properties(
    source: &str,
    readonly: bool,
    independent_svg: bool,
    decision: bool,
) -> (CssomStore, CssStyleProperties) {
    let parser = if independent_svg {
        CssParserContext::default().with_svg_glyph_orientation_vertical()
    } else {
        CssParserContext::default()
    };
    let mut support = CssomDeclarationSupport::try_new(
        [
            "width",
            "margin",
            "float",
            "flex",
            "page-break-before",
            "text-orientation",
            "background",
        ]
        .into_iter()
        .map(|name| {
            (
                CssPropertyGrammar::from_name(name).unwrap(),
                if decision && name == "background" {
                    CssomUsabilityRequirement::WholeDeclarationDecision
                } else {
                    CssomUsabilityRequirement::AllCheckedValues
                },
            )
        })
        .collect(),
    )
    .unwrap();
    if independent_svg {
        support = support
            .with_svg_glyph_orientation_vertical(CssomUsabilityRequirement::AllCheckedValues);
    } else {
        let mut grammars = support.properties().to_vec();
        grammars.push((
            CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap(),
            CssomUsabilityRequirement::AllCheckedValues,
        ));
        support = CssomDeclarationSupport::try_new(grammars).unwrap();
    }
    let context = CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Support, "renderer"),
                data: CssomInputData::Support(support),
            },
            CssomInput {
                version: version(CssomInputRole::ParserMode, "parser"),
                data: CssomInputData::ParserMode(parser),
            },
        ],
        ..Default::default()
    };
    let mut store = CssomStore::new(CssomLimits::default(), context).unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let declarations = CssParser::new(parser, Default::default())
        .parse_block_contents(source)
        .into_parts()
        .0;
    let ticket = batch
        .create_declarations(declarations, false, readonly, None)
        .unwrap();
    let id = store.commit(batch).unwrap().block(&ticket).unwrap().clone();
    let handle = store.css_style_properties(&id).unwrap();
    (store, handle)
}

#[test]
fn ordinary_shorthand_custom_and_float_reflections_use_empty_priority_and_owning_capture() {
    let (mut store, style) = properties(
        "width:1px!important; float:left!important",
        false,
        false,
        false,
    );
    let captured = style.capture(&store.snapshot()).unwrap();
    assert_eq!(captured.width(limits()).unwrap(), "1px");
    assert_eq!(captured.css_float(limits()).unwrap(), "left");
    let outcome = style
        .set_width(&mut store, "3px", limits(), |_| {
            panic!("unconditional grammar must not resolve")
        })
        .unwrap();
    assert_eq!(outcome.value(), &());
    drop(outcome);
    assert_eq!(style.width(&store, limits()).unwrap(), "3px");
    assert_eq!(
        style
            .get_property_priority(&store, "width", limits())
            .unwrap(),
        ""
    );
    drop(
        style
            .set_margin(&mut store, "1px 2px", limits(), |_| {
                panic!("unconditional shorthand must not resolve")
            })
            .unwrap(),
    );
    assert_eq!(style.margin(&store, limits()).unwrap(), "1px 2px");
    assert_eq!(style.margin_top(&store, limits()).unwrap(), "1px");
    assert_eq!(style.margin_right(&store, limits()).unwrap(), "2px");
    drop(
        style
            .set_css_float(&mut store, "right", limits(), |_| {
                panic!("unconditional float must not resolve")
            })
            .unwrap(),
    );
    assert_eq!(style.css_float(&store, limits()).unwrap(), "right");
    assert_eq!(
        style
            .get_property_priority(&store, "float", limits())
            .unwrap(),
        ""
    );
    drop(
        style
            .set_property(&mut store, "--Theme", "token", None, limits(), |_| {
                panic!("custom property must not resolve")
            })
            .unwrap(),
    );
    assert_eq!(
        style
            .get_property_value(&store, "--Theme", limits())
            .unwrap(),
        "token"
    );
    assert_eq!(
        style
            .get_property_value(&store, "--theme", limits())
            .unwrap(),
        ""
    );
    drop(store);
    assert_eq!(captured.width(limits()).unwrap(), "1px");
    assert_eq!(captured.css_float(limits()).unwrap(), "left");
    assert_eq!(
        captured.get_property_priority("width", limits()).unwrap(),
        "important"
    );
}

#[test]
fn distinct_legacy_grammar_and_vendor_names_preserve_native_semantics() {
    let (mut store, style) = properties("", false, false, false);
    drop(
        style
            .set_page_break_before(&mut store, "always", limits(), |_| panic!("no resolver"))
            .unwrap(),
    );
    // CSSOM serializes the selected longhand; authored legacy spelling remains
    // grammar provenance rather than changing the selected declaration value.
    assert_eq!(style.page_break_before(&store, limits()).unwrap(), "page");
    assert_eq!(style.break_before(&store, limits()).unwrap(), "page");
    let revision = store.snapshot().revision().clone();
    // 'column' is in the canonical grammar, but not this legacy shorthand.
    drop(
        style
            .set_page_break_before(&mut store, "column", limits(), |_| {
                panic!("invalid grammar must not resolve")
            })
            .unwrap(),
    );
    assert_eq!(store.snapshot().revision(), &revision);
    assert_eq!(style.page_break_before(&store, limits()).unwrap(), "page");
    drop(
        style
            .set_webkit_flex(&mut store, "2 3 10px", limits(), |_| panic!("no resolver"))
            .unwrap(),
    );
    assert_eq!(style.webkit_flex(&store, limits()).unwrap(), "2 3 10px");
    assert_eq!(style.flex(&store, limits()).unwrap(), "2 3 10px");
}

#[test]
fn one_svg_spelling_preserves_independent_and_legacy_grammar_contexts() {
    let (mut store, style) = properties("", false, true, false);
    drop(
        style
            .set_glyph_orientation_vertical(&mut store, "45deg", limits(), |_| {
                panic!("independent SVG support is unconditional")
            })
            .unwrap(),
    );
    assert_eq!(
        style.glyph_orientation_vertical(&store, limits()).unwrap(),
        "45deg"
    );
    let snapshot = store.snapshot();
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(entries),
        ..
    } = snapshot.block(style.id()).unwrap().data()
    else {
        panic!("checked SVG projection")
    };
    assert_eq!(
        entries.entries()[0].property_name(),
        surgeist_css::CssSvgGlyphOrientationVerticalDeclaration::metadata().property()
    );
    let (mut legacy_store, legacy_style) = properties("", false, false, false);
    drop(
        legacy_style
            .set_glyph_orientation_vertical(&mut legacy_store, "90deg", limits(), |_| {
                panic!("finite legacy support is unconditional")
            })
            .unwrap(),
    );
    assert_eq!(
        legacy_style
            .glyph_orientation_vertical(&legacy_store, limits())
            .unwrap(),
        "sideways"
    );
    assert_eq!(
        legacy_style
            .text_orientation(&legacy_store, limits())
            .unwrap(),
        "sideways"
    );
    let revision = legacy_store.snapshot().revision().clone();
    drop(
        legacy_style
            .set_glyph_orientation_vertical(&mut legacy_store, "45deg", limits(), |_| {
                panic!("finite alias rejects before resolution")
            })
            .unwrap(),
    );
    assert_eq!(legacy_store.snapshot().revision(), &revision);
    assert_eq!(
        legacy_style
            .glyph_orientation_vertical(&legacy_store, limits())
            .unwrap(),
        "sideways"
    );
}

#[test]
fn generated_members_do_not_grant_support_and_admission_errors_do_not_publish() {
    let (mut store, style) = properties("", false, false, true);
    let before = store.snapshot();
    drop(
        style
            .set_color(&mut store, "red", limits(), |_| {
                panic!("unsupported name must not resolve")
            })
            .unwrap(),
    );
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(style.color(&store, limits()).unwrap(), "");
    let calls = Cell::new(0);
    assert!(matches!(
        style.set_background(&mut store, "red", limits(), |_| {
            calls.set(calls.get() + 1);
            Err(CssomError::InvalidInput("independent provider declined"))
        }),
        Err(CssomError::InvalidInput("independent provider declined"))
    ));
    assert_eq!(calls.get(), 1);
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(style.background(&store, limits()).unwrap(), "");
    let (mut readonly, style) = properties("width:1px", true, false, false);
    let revision = readonly.snapshot().revision().clone();
    assert!(matches!(
        style.set_width(&mut readonly, "2px", limits(), |_| panic!(
            "readonly must not resolve"
        )),
        Err(CssomError::Source(CssomException::NoModificationAllowed))
    ));
    assert_eq!(readonly.snapshot().revision(), &revision);
    assert_eq!(style.width(&readonly, limits()).unwrap(), "1px");
}
