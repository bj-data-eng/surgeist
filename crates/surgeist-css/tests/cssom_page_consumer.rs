#![forbid(unsafe_code)]
//! Public-consumer expectations from the adopted Blink Page/margin witness.
use surgeist_css::{CssRule, parse_sheet};

#[test]
fn named_page_is_admitted() {
    let report = parse_sheet("@page invoice {}");
    assert!(report.is_clean(), "named Page must be admitted: {report:?}");
    assert!(matches!(report.syntax().rules()[0], CssRule::Page(_)));
}

#[test]
fn page_size_and_margin_child_are_retained() {
    let report = parse_sheet("@page { size:A4; @top-left { content:\"X\"; } }");
    assert!(
        report.is_clean(),
        "supported Page payload must be retained: {report:?}"
    );
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { size: a4; @top-left { content: \"X\"; } }"
    );
}

#[test]
fn declaration_only_page_has_adopted_cssom_output() {
    let report = parse_sheet("@page { margin:1px; }");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { margin: 1px; }"
    );
    assert_eq!(report, before);
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { margin: 1px; }"
    );
}

use surgeist_css::{
    CssImportance, CssMarginBox, CssMarginDeclarationBlock, CssMarginRule, CssPageBody,
    CssPageDeclaration, CssPageDeclarationBlock, CssPageDescriptor, CssPageDescriptorKind as K,
    CssPageDescriptorValueRef as V, CssPagePseudo as P,
    CssRuleCssomSerializationErrorKind as RuleError,
    CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, parse_margin_block, parse_page_block,
    parse_page_descriptor_value, parse_page_selector_list,
};
#[test]
fn complete_page_selector_getter_preserves_compounds_lists_and_original_coordinates() {
    let source = " /*😀*/ \\69 nvoice:FIRST:left, :blank:recto:verso ";
    let report = parse_page_selector_list(source);
    assert!(report.is_clean(), "{report:?}");
    let value = report.syntax().as_ref().unwrap();
    assert_eq!(
        value.serialize_cssom().unwrap(),
        "invoice:first:left, :blank:recto:verso"
    );
    assert_eq!(value.selectors()[0].name().unwrap().as_str(), "invoice");
    assert_eq!(value.selectors()[0].pseudos(), &[P::First, P::Left]);
    assert_eq!(value.selectors()[0].specificity().named(), 1);
    assert_eq!(value.selectors()[0].specificity().first_or_blank(), 1);
    assert_eq!(
        value.selectors()[0]
            .origin()
            .unwrap()
            .span()
            .start()
            .byte_offset()
            .value(),
        10
    );
    assert_eq!(value.origin().unwrap().source().as_str(), source);
    let empty = parse_page_selector_list(" /*empty*/ ");
    assert!(empty.is_clean());
    assert!(empty.syntax().as_ref().unwrap().is_empty());
    assert_eq!(
        empty.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        ""
    );
}
#[test]
fn foreign_or_incomplete_page_selectors_reject_the_whole_raw_input() {
    for source in [
        "invoice junk",
        ": first",
        "invoice :left",
        ".card",
        ":nth(1)",
        "invoice,",
        ",invoice",
        "invoice > :first",
        "invoice {}",
    ] {
        let report = parse_page_selector_list(source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
    }
}
#[test]
fn selector_limits_charge_lists_names_and_pseudos_and_allow_atomic_retry() {
    let report = parse_page_selector_list("invoice:first:left, :blank:recto");
    let value = report.syntax().as_ref().unwrap();
    let before = value.clone();
    let expected = "invoice:first:left, :blank:recto";
    let adequate = Limits::new(8, 8, expected.len());
    assert_eq!(
        value.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
    for limits in [
        Limits::new(7, 8, 100),
        Limits::new(8, 7, 100),
        Limits::new(8, 8, expected.len() - 1),
    ] {
        assert!(value.serialize_cssom_with_limits(limits).is_err());
        assert_eq!(value, &before);
        assert_eq!(
            value.serialize_cssom_with_limits(adequate).unwrap(),
            expected
        );
    }
}
#[test]
fn page_descriptor_grammars_and_getters_keep_their_distinct_domains() {
    for (kind, input, expected) in [
        (K::Size, "LANDSCAPE A4", "a4 landscape"),
        (K::Size, "10cm 20cm", "10cm 20cm"),
        (K::Size, "portrait", "portrait"),
        (K::Size, "auto", "auto"),
        (K::PageOrientation, "ROTATE-LEFT", "rotate-left"),
        (K::Marks, "cross crop", "crop cross"),
        (K::Bleed, "-2pt", "-2pt"),
        (K::Bleed, "inherit", "inherit"),
    ] {
        let report = parse_page_descriptor_value(input, kind);
        assert!(report.is_clean(), "{input}: {report:?}");
        let value = report.syntax().as_ref().unwrap();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value.origin().source().as_str(), input);
    }
    for (kind, input) in [
        (K::Size, "A4 A3"),
        (K::Size, "-1px"),
        (K::Size, "1%"),
        (K::PageOrientation, "landscape"),
        (K::Marks, "crop crop"),
        (K::Marks, "none crop"),
        (K::Bleed, "1%"),
        (K::Size, "A4 !important"),
    ] {
        assert!(
            parse_page_descriptor_value(input, kind).syntax().is_none(),
            "{input}"
        );
    }
}
#[test]
fn page_domains_retain_applicable_properties_custom_values_and_margin_order() {
    let source = "@page invoice:first { color:red; size:A4; margin:1em; --x: RAW; @TOP-LEFT { content:\"X\"; color:blue; } page-orientation:rotate-right; @bottom-right { content:counter(page); } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!()
    };
    assert_eq!(page.declarations().occurrences().len(), 5);
    assert_eq!(
        page.margin_rules()
            .iter()
            .map(|v| v.name())
            .collect::<Vec<_>>(),
        vec![CssMarginBox::TopLeft, CssMarginBox::BottomRight]
    );
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page invoice:first { color: red; size: a4; margin: 1em; --x: RAW; page-orientation: rotate-right; @top-left { content: \"X\"; color: blue; } @bottom-right { content: counter(page); } }"
    );
}
#[test]
fn page_and_margin_recovery_drop_invalid_owned_units_without_losing_neighbors() {
    let report = parse_sheet(
        "@page { size:bogus; size:A4; @top-left bad {content:'bad'} @top-right {content:'good'; size:A4; width:3px} marks:crop; }",
    );
    assert!(!report.is_clean());
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!()
    };
    assert_eq!(page.margin_rules().len(), 1);
    assert_eq!(page.margin_rules()[0].name(), CssMarginBox::TopRight);
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { size: a4; marks: crop; @top-right { content: \"good\"; width: 3px; } }"
    );
}
#[test]
fn page_typed_reconstruction_retains_occurrences_and_raw_edit_provenance() {
    let report = parse_page_block("{ margin:1px; size:A4; @top-left {content:'X'} }");
    assert!(report.is_clean(), "{report:?}");
    let body = report.syntax().as_ref().unwrap().body();
    let original = body.declarations().properties()[0].clone();
    let raw = parse_page_descriptor_value(" /*edit*/ A3 ", K::Size);
    let raw = raw.syntax().as_ref().unwrap().clone();
    assert!(matches!(raw.view(), V::Size(_)));
    let mut occurrences = body.declarations().occurrences().to_vec();
    occurrences.push(CssPageDeclaration::Descriptor(CssPageDescriptor::new(
        raw,
        CssImportance::Important,
    )));
    let block = CssPageDeclarationBlock::try_new(occurrences).unwrap();
    assert!(block.properties()[0].same_occurrence(&original));
    let selected = block.effective_descriptor(K::Size).unwrap();
    assert!(selected.occurrence().is_none());
    assert_eq!(selected.value().origin().source().as_str(), " /*edit*/ A3 ");
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin: 1px; size: a3 !important;"
    );
    let margin = parse_margin_block("{content:'Y'}");
    let margin = margin.syntax().as_ref().unwrap().body().clone();
    let new_body = CssPageBody::new(
        block,
        vec![CssMarginRule::new(CssMarginBox::TopCenter, margin)],
    );
    let sheet = parse_sheet("@page invoice {}");
    let CssRule::Page(page) = &sheet.syntax().rules()[0] else {
        panic!()
    };
    assert_eq!(
        CssRule::Page(page.with_body(new_body))
            .serialize_cssom()
            .unwrap(),
        "@page invoice { margin: 1px; size: a3 !important; @top-center { content: \"Y\"; } }"
    );
}
#[test]
fn page_margin_projection_does_not_emit_ordinary_size_extension() {
    for source in [
        "@page { width:1px; height:2px; }",
        "@page { @top-left {width:1px;height:2px} }",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{report:?}");
        let output = report.syntax().serialize_cssom().unwrap();
        assert!(output.contains("width: 1px; height: 2px;"), "{output}");
        assert!(!output.contains("size:"));
    }
}
#[test]
fn empty_page_and_margin_whitespace_is_exact() {
    for (source, expected) in [
        ("@page {}", "@page { }"),
        ("@page invoice {}", "@page invoice { }"),
        ("@page { @top-left {} }", "@page { @top-left { } }"),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean());
        assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    }
}
#[test]
fn page_cumulative_budget_reports_margin_child_and_retry_preserves_source() {
    let report = parse_sheet("@page { size:A4; @top-left {content:'X'} @top-right {content:'Y'} }");
    assert!(report.is_clean());
    let before = report.clone();
    let expected = report.syntax().serialize_cssom().unwrap();
    let failure = report
        .syntax()
        .serialize_cssom_with_limits(Limits::new(1000, 1000, expected.len() - 4))
        .unwrap_err();
    assert_eq!(failure.kind(), RuleError::Resource(Resource::ByteLimit));
    assert_eq!(failure.rule_path(), &[0, 1]);
    assert_eq!(report, before);
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    let raw = parse_margin_block("{content:'X'}");
    let raw = raw.syntax().as_ref().unwrap().body();
    assert!(
        CssMarginDeclarationBlock::try_new_with_limits(
            raw.properties().iter().cloned().collect(),
            Limits::new(1, 100, 100)
        )
        .is_err()
    );
    assert!(CssMarginDeclarationBlock::try_new(raw.properties().iter().cloned().collect()).is_ok());
}

#[test]
fn selected_page_terminal_edits_preserve_sources_and_do_not_expand_the_original_shorthand() {
    use surgeist_css::{
        CssPageRuleView, CssSpecifiedDeclarationValueRef, CssSpecifiedPageDeclarationBlock as B,
        CssSpecifiedPageDeclarationEntry as E,
    };
    let source = "@page invoice:first { margin:1px 2px; size:A4; margin-left:3px; }";
    let report = parse_sheet(source);
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("Page")
    };
    let selected = page.declarations().try_specified().unwrap();
    let top = selected
        .entries()
        .iter()
        .find_map(|v| match v {
            E::Property(v)
                if v.property_name()
                    == surgeist_css::CssPropertyNameRef::Known(
                        surgeist_css::CssKnownProperty::MarginTop,
                    ) =>
            {
                Some(v.clone())
            }
            _ => None,
        })
        .unwrap();
    let descriptor = selected
        .entries()
        .iter()
        .find_map(|v| match v {
            E::Descriptor(v) => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    let named = descriptor.occurrence().unwrap();
    assert_eq!(named.parsed_name().unwrap().source().as_str(), source);
    assert!(
        named
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(page.selectors().origin().unwrap().source())
    );
    let edited = B::try_from_entries(&[
        E::Descriptor(descriptor.with_importance(CssImportance::Important)),
        E::Property(top.with_importance(CssImportance::Important)),
    ])
    .unwrap();
    let view = CssPageRuleView::new(page.selectors(), &edited, &[]);
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "@page invoice:first { size: a4 !important; margin-top: 1px !important; }"
    );
    let E::Descriptor(retained_descriptor) = &edited.entries()[0] else {
        panic!("descriptor")
    };
    assert!(
        retained_descriptor
            .occurrence()
            .unwrap()
            .same_occurrence(named)
    );
    assert_eq!(descriptor.importance(), CssImportance::Normal);
    let E::Property(retained) = &edited.entries()[1] else {
        panic!("terminal")
    };
    assert!(retained.source().same_occurrence(top.source()));
    assert_eq!(retained.source().importance(), CssImportance::Normal);
    assert!(matches!(
        retained.value(),
        CssSpecifiedDeclarationValueRef::Completed(_)
    ));
    assert!(B::try_from_entries(&[E::Property(top.clone()), E::Property(top)]).is_err());
    assert!(
        B::try_from_entries(&[E::Descriptor(descriptor.clone()), E::Descriptor(descriptor)])
            .is_err()
    );
}
#[test]
fn selected_page_and_margin_views_preserve_pending_occurrence_and_order_and_reject_wrong_domains() {
    use surgeist_css::{
        CssMarginRuleView, CssPageRuleView, CssSpecifiedDeclarationBlock,
        CssSpecifiedPageDeclarationBlock as B, CssSpecifiedPageDeclarationEntry as E,
    };
    let report = parse_sheet(
        "@page { margin-top:var(--m); @top-left { content:'X'; color:red } @bottom-center{} }",
    );
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("Page")
    };
    let projected = page.declarations().try_specified().unwrap();
    let E::Property(entry) = &projected.entries()[0] else {
        panic!("pending terminal")
    };
    let edited = B::try_from_entries(&[E::Property(entry.clone())]).unwrap();
    let E::Property(retained) = &edited.entries()[0] else {
        panic!("pending terminal")
    };
    assert!(
        retained
            .source()
            .same_occurrence(&page.declarations().properties()[0])
    );
    let first = page.margin_rules()[0]
        .declarations()
        .try_specified_properties()
        .unwrap();
    let empty = CssSpecifiedDeclarationBlock::try_from_margin_entries(&[]).unwrap();
    let margins = [
        CssMarginRuleView::try_new(CssMarginBox::BottomCenter, &empty).unwrap(),
        CssMarginRuleView::try_new(CssMarginBox::TopLeft, &first).unwrap(),
    ];
    let margin_text = "@top-left { content: \"X\"; color: red; }";
    assert_eq!(margins[1].serialize_cssom().unwrap(), margin_text);
    assert!(
        margins[1]
            .serialize_cssom_with_limits(Limits::new(1000, 1000, margin_text.len() - 1))
            .is_err()
    );
    assert_eq!(margins[1].serialize_cssom().unwrap(), margin_text);
    assert_eq!(
        CssPageRuleView::new(page.selectors(), &edited, &margins)
            .serialize_cssom()
            .unwrap(),
        "@page { margin-top: var(--m); @bottom-center { } @top-left { content: \"X\"; color: red; } }"
    );
    let ordinary = CssSpecifiedDeclarationBlock::try_from_entries(first.entries()).unwrap();
    assert!(CssMarginRuleView::try_new(CssMarginBox::TopLeft, &ordinary).is_none());
    let invalid = parse_sheet(".x { transform:none }");
    let CssRule::Style(style) = &invalid.syntax().rules()[0] else {
        panic!("style")
    };
    let ordinary =
        CssSpecifiedDeclarationBlock::try_from_declarations(style.declarations()).unwrap();
    assert!(CssSpecifiedDeclarationBlock::try_from_page_entries(ordinary.entries()).is_err());
    assert!(CssSpecifiedDeclarationBlock::try_from_margin_entries(ordinary.entries()).is_err());
}
#[test]
fn edited_page_and_margin_projection_share_atomic_node_projection_and_byte_limits() {
    use surgeist_css::{CssMarginRuleView, CssPageRuleView};
    let report = parse_sheet("@page named { size:A4; @top-left {content:'X'} }");
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("Page")
    };
    let selected = page.declarations().try_specified().unwrap();
    let margin = page.margin_rules()[0]
        .declarations()
        .try_specified_properties()
        .unwrap();
    let margins = [CssMarginRuleView::try_new(CssMarginBox::TopLeft, &margin).unwrap()];
    let view = CssPageRuleView::new(page.selectors(), &selected, &margins);
    let expected = "@page named { size: a4; @top-left { content: \"X\"; } }";
    assert_eq!(view.serialize_cssom().unwrap(), expected);
    for limits in [
        Limits::new(0, 1000, 1000),
        Limits::new(1000, 0, 1000),
        Limits::new(1000, 1000, expected.len() - 1),
    ] {
        assert!(view.serialize_cssom_with_limits(limits).is_err());
        assert_eq!(view.serialize_cssom().unwrap(), expected);
    }
}

#[test]
fn page_descriptors_retain_pending_phases_and_select_effective_priority_without_losing_occurrences()
{
    for (kind, text) in [
        (K::Size, "var(--sheet)"),
        (K::PageOrientation, "env(print-orientation)"),
        (K::Marks, "var(--marks, crop)"),
        (K::Bleed, "calc(env(bleed) + 1px)"),
    ] {
        let report = parse_page_descriptor_value(text, kind);
        assert!(report.is_clean(), "{report:?}");
        let value = report.syntax().as_ref().unwrap();
        assert!(matches!(value.view(), V::Pending(_)));
        assert_eq!(value.serialize_specified().unwrap(), text);
    }
    let report = parse_sheet("@page {size:A4;size:letter!important;size:legal;marks:cross crop;}");
    assert!(report.is_clean());
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("Page")
    };
    assert_eq!(page.declarations().occurrences().len(), 4);
    assert_eq!(
        page.declarations()
            .effective_descriptor(K::Size)
            .unwrap()
            .importance(),
        CssImportance::Important
    );
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { size: letter !important; marks: crop cross; }"
    );
}
#[test]
fn every_canonical_margin_name_and_shared_shorthand_grammar_survives_owning_domains() {
    let names = [
        "top-left-corner",
        "top-left",
        "top-center",
        "top-right",
        "top-right-corner",
        "bottom-left-corner",
        "bottom-left",
        "bottom-center",
        "bottom-right",
        "bottom-right-corner",
        "left-top",
        "left-middle",
        "left-bottom",
        "right-top",
        "right-middle",
        "right-bottom",
    ];
    let mut source = String::from("@page {");
    for name in names {
        source.push_str(&format!("@{name}{{}}"));
    }
    source.push('}');
    let report = parse_sheet(&source);
    assert!(report.is_clean());
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("Page")
    };
    assert_eq!(page.margin_rules().len(), 16);
    let expected = format!(
        "@page {{ {} }}",
        names
            .into_iter()
            .map(|name| format!("@{name} {{ }}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    let report = parse_sheet(
        "@page { font:italic 12px serif; border-style:solid; @TOP-LEFT { overflow:hidden; background:red; } }",
    );
    assert!(report.is_clean(), "{report:?}");
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("Page")
    };
    let selected = page.declarations().try_specified().unwrap();
    assert!(selected.serialize_cssom().unwrap().contains("font:"));
    let child = page.margin_rules()[0].serialize_cssom().unwrap();
    assert!(child.starts_with("@top-left { "));
    assert!(child.contains("overflow: hidden;"));
    assert!(child.contains("background: red;"));
}
