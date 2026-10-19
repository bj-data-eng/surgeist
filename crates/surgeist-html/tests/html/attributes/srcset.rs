use crate::html::support::{html_element as element, unqualified_attribute as attr};
use surgeist_html::*;

fn attributes(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(values.iter().map(|&(name, value)| attr(name, value))).unwrap()
}
fn img(value: &str) -> HtmlSrcsetReport {
    HtmlValueInterpreter::new(100_000)
        .img_srcset(&element("img"), Some(&attr("srcset", value)))
        .unwrap()
}
fn urls(report: &HtmlSrcsetReport) -> Vec<DocumentString> {
    report
        .candidates()
        .unwrap()
        .iter()
        .map(|candidate| candidate.url().clone())
        .collect()
}
fn strings(report: &HtmlSrcsetReport) -> Vec<String> {
    urls(report)
        .iter()
        .map(|url| url.to_utf8().unwrap())
        .collect()
}
fn errors(report: &HtmlSrcsetReport) -> Vec<HtmlSrcsetParseError> {
    report
        .diagnostics()
        .iter()
        .map(HtmlSrcsetParseDiagnostic::error)
        .collect()
}
fn author_errors(report: &HtmlSrcsetReport) -> Vec<HtmlSrcsetAuthorViolation> {
    report
        .author_diagnostics()
        .iter()
        .map(HtmlSrcsetAuthorDiagnostic::violation)
        .collect()
}
fn density(report: &HtmlSrcsetReport, index: usize) -> f64 {
    match report.candidates().unwrap()[index].descriptor() {
        HtmlParsedSrcsetDescriptor::Density(density) => density.value(),
        other => panic!("expected density, got {other:?}"),
    }
}
fn native(url: &str, descriptor: HtmlSrcsetDescriptor) -> HtmlSrcsetCandidate {
    HtmlSrcsetCandidate::new(HtmlSrcsetUrlToken::new(url).unwrap(), descriptor)
}
fn native_density(value: &str) -> HtmlSrcsetDescriptor {
    HtmlSrcsetDescriptor::Density(HtmlSrcsetDensity::from_decimal_syntax(value).unwrap())
}
fn native_width(value: u128) -> HtmlSrcsetDescriptor {
    HtmlSrcsetDescriptor::Width(HtmlSrcsetWidth::new(HtmlNonNegativeInteger::from(value)).unwrap())
}

#[test]
fn native_constructors_preserve_exact_symbols_and_reject_intrinsic_invalid_values() {
    for (value, error) in [
        ("", HtmlSrcsetUrlTokenError::Empty),
        (",a", HtmlSrcsetUrlTokenError::EdgeComma),
        ("a,", HtmlSrcsetUrlTokenError::EdgeComma),
        ("a b", HtmlSrcsetUrlTokenError::AsciiWhitespace),
    ] {
        assert_eq!(HtmlSrcsetUrlToken::new(value), Err(error));
    }
    assert!(HtmlSrcsetUrlToken::new("../relative,a").is_ok());
    assert!(HtmlSrcsetUrlToken::new(DocumentString::from_utf16(vec![0, 0xd800])).is_ok());
    assert_eq!(
        HtmlSrcsetWidth::new(HtmlNonNegativeInteger::from(0)),
        Err(HtmlSrcsetWidthError)
    );
    for value in ["0", "-0", "-1", "-1e-999"] {
        assert_eq!(
            HtmlSrcsetDensity::from_decimal_syntax(value),
            Err(HtmlSrcsetDensityError::Nonpositive)
        );
    }
    for value in ["+1", "1.", "1e+", " 1", "1 ", "NaN", "Infinity"] {
        assert_eq!(
            HtmlSrcsetDensity::from_decimal_syntax(value),
            Err(HtmlSrcsetDensityError::InvalidSyntax)
        );
    }
    let tiny = HtmlSrcsetDensity::from_decimal_syntax("1e-999").unwrap();
    assert_eq!(tiny.syntax(), &DocumentString::from("1e-999"));
    assert_eq!(
        tiny.to_floating_point_number().unwrap().value().to_bits(),
        0
    );
    assert_eq!(
        HtmlSrcsetDensity::from_decimal_syntax("1e999")
            .unwrap()
            .to_floating_point_number(),
        Err(HtmlSrcsetDensityProjectionError)
    );
    let value = HtmlSrcsetValue::new(vec![
        native("data:image/png;base64,AAAA", native_density("02.0")),
        native("../next", native_density("3")),
    ])
    .unwrap();
    assert_eq!(
        value.as_document_string(),
        DocumentString::from("data:image/png;base64,AAAA 02.0x, ../next 3x")
    );
    assert_eq!(
        value.external_conformance(),
        HtmlSrcsetExternalConformance::Unchecked
    );
    let raw = value.clone().into_imagesrcset_attribute();
    assert_eq!(
        raw.name(),
        &KnownHtmlAttribute::Imagesrcset.qualified_name()
    );
    assert_eq!(raw.value(), &value.as_document_string());
    assert_eq!(
        img(&value.as_document_string().to_utf8().unwrap()).author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
    assert_eq!(
        HtmlSrcsetValue::new(vec![]),
        Err(HtmlSrcsetAuthorViolation::EmptyList)
    );
    assert!(HtmlSrcsetDensity::try_from(HtmlFloatingPointNumber::try_from(2.5).unwrap()).is_ok());
}

#[test]
fn author_uniqueness_compares_mathematical_values_before_binary64_rounding() {
    for value in ["a 1x, b 1.0x", "a 1x, b 10e-1x", "a, b 1x"] {
        let report = img(value);
        assert!(
            author_errors(&report).contains(&HtmlSrcsetAuthorViolation::DuplicateDensity {
                first: 0,
                duplicate: 1
            })
        );
        assert_eq!(strings(&report), ["a", "b"]);
        assert!(report.diagnostics().is_empty());
    }
    let report = img("a 1.00000000000000001x, b 1x");
    assert_eq!(
        report.author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
    assert_eq!(density(&report, 0), 1.0);
    assert_eq!(density(&report, 1), 1.0);
    let report = img("a 02w, b 2w");
    assert!(
        author_errors(&report).contains(&HtmlSrcsetAuthorViolation::DuplicateWidth {
            first: 0,
            duplicate: 1
        })
    );
    assert_eq!(strings(&report), ["a", "b"]);
    assert!(report.diagnostics().is_empty());
    let report = img("a 1w, b 2x");
    assert!(author_errors(&report).contains(&HtmlSrcsetAuthorViolation::MixedWidthDescriptors));
    assert_eq!(strings(&report), ["a", "b"]);
    assert!(
        HtmlSrcsetValue::new(vec![
            native("a", native_density("1")),
            native("b", native_density("10e-1"))
        ])
        .is_err()
    );
    assert!(
        HtmlSrcsetValue::new(vec![
            native("a", native_width(1)),
            native("b", HtmlSrcsetDescriptor::Unspecified)
        ])
        .is_err()
    );
}

#[test]
fn urls_use_nonwhitespace_collection_and_omitted_density_remains_absent() {
    let report = img("a, b 2x");
    assert_eq!(strings(&report), ["a", "b"]);
    assert_eq!(
        report.candidates().unwrap()[0].descriptor(),
        &HtmlParsedSrcsetDescriptor::Unspecified
    );
    assert_eq!(density(&report, 1), 2.0);
    let report = img("a,b 2x");
    assert_eq!(strings(&report), ["a,b"]);
    assert_eq!(density(&report, 0), 2.0);
    assert_eq!(
        report.author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
    assert_eq!(
        strings(&img("data:image/png;base64,AAAA 2x")),
        ["data:image/png;base64,AAAA"]
    );
    assert_eq!(strings(&img("a\u{a0}b 2x")), ["a\u{a0}b"]);
    for whitespace in ["\t", "\n", "\u{c}", "\r", " "] {
        let report = img(&format!("a{whitespace}2x,{whitespace}b{whitespace}3x"));
        assert_eq!(strings(&report), ["a", "b"]);
        assert_eq!(
            report.author(),
            HtmlSrcsetAuthorState::HtmlConstraintsConforming
        );
    }
}

#[test]
fn leading_extra_and_terminal_commas_keep_recovery_separate_from_author_grammar() {
    for (value, errors_expected) in [
        ("a,", vec![]),
        ("a,,", vec![HtmlSrcsetParseError::MultipleTrailingCommas]),
        (",a", vec![HtmlSrcsetParseError::SkippedCommas]),
    ] {
        let report = img(value);
        assert_eq!(strings(&report), ["a"]);
        assert_eq!(errors(&report), errors_expected);
        assert_eq!(report.author(), HtmlSrcsetAuthorState::Nonconforming);
    }
    assert_eq!(strings(&img("a 1x,, b 2x")), ["a", "b"]);
    assert_eq!(
        errors(&img("a 1x,, b 2x")),
        [HtmlSrcsetParseError::SkippedCommas]
    );
    assert_eq!(
        img("a ,b 2x").author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
}

#[test]
fn strict_descriptor_guards_do_not_use_permissive_numeric_prefix_acceptance() {
    for value in [
        "a +1x",
        "a 1.x",
        "a 1e+x",
        "a 1X",
        "a +2w",
        "a 2W",
        "a 1\u{a0}x",
    ] {
        let report = img(value);
        assert!(report.candidates().unwrap().is_empty(), "{value}");
        assert_eq!(
            errors(&report),
            [HtmlSrcsetParseError::RejectedCandidate],
            "{value}"
        );
        assert_eq!(report.author(), HtmlSrcsetAuthorState::Nonconforming);
    }
    for value in ["a 1x 2x, b 3x", "a 2w 3x, b 3x", "a 0w, b 3x", "a 0h, b 3x"] {
        assert_eq!(strings(&img(value)), ["b"]);
    }
}

#[test]
fn density_zero_and_underflow_survive_but_negative_nonzero_is_rejected() {
    let report = img("a 0x, b -1x, c 2x");
    assert_eq!(strings(&report), ["a", "c"]);
    assert_eq!(density(&report, 0).to_bits(), 0);
    assert_eq!(density(&report, 1), 2.0);
    assert_eq!(errors(&report), [HtmlSrcsetParseError::RejectedCandidate]);
    for (value, author) in [
        ("1e-999", HtmlSrcsetAuthorState::HtmlConstraintsConforming),
        ("-1e-999", HtmlSrcsetAuthorState::Nonconforming),
        ("-0", HtmlSrcsetAuthorState::Nonconforming),
    ] {
        let report = img(&format!("a {value}x"));
        assert_eq!(density(&report, 0).to_bits(), 0);
        assert_eq!(report.author(), author);
        assert!(report.diagnostics().is_empty());
    }
}

#[test]
fn source_selected_density_overflow_omits_only_the_failed_candidate() {
    let report = img("a 1e999x, b 2x");
    assert_eq!(strings(&report), ["b"]);
    assert_eq!(density(&report, 0), 2.0);
    assert_eq!(
        errors(&report),
        [HtmlSrcsetParseError::DensityConversionOverflow]
    );
    assert_eq!(
        report.author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
    let diagnostic = report.diagnostics()[0];
    assert_eq!(diagnostic.candidate_index(), Some(0));
    assert_eq!(
        (diagnostic.range().start(), diagnostic.range().end()),
        (2, 8)
    );
    let report = img("a 1e999x");
    assert!(report.candidates().unwrap().is_empty());
    assert_eq!(report.processing(), HtmlSrcsetProcessingState::Candidates);
    assert_eq!(
        errors(&report),
        [HtmlSrcsetParseError::DensityConversionOverflow]
    );
    assert!(
        report
            .overflow_recovery_source_clause()
            .contains("73aa6c89e2cb")
    );
    assert_eq!(strings(&img("a -1e999x, b 2x")), ["b"]);
}

#[test]
fn future_height_errors_can_retain_width_candidates_in_either_order() {
    let report = img("a 2w 3h, b 3h 2w, c 3h, d 2h 3h 2w");
    assert_eq!(strings(&report), ["a", "b"]);
    for candidate in report.candidates().unwrap() {
        match candidate.descriptor() {
            HtmlParsedSrcsetDescriptor::Width(width) => assert_eq!(width.value().to_u64(), Ok(2)),
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(
        errors(&report),
        [
            HtmlSrcsetParseError::FutureHeightDescriptor,
            HtmlSrcsetParseError::FutureHeightDescriptor,
            HtmlSrcsetParseError::FutureHeightDescriptor,
            HtmlSrcsetParseError::RejectedCandidate,
            HtmlSrcsetParseError::FutureHeightDescriptor,
            HtmlSrcsetParseError::FutureHeightDescriptor,
            HtmlSrcsetParseError::RejectedCandidate
        ]
    );
    assert_eq!(report.author(), HtmlSrcsetAuthorState::Nonconforming);
    assert_eq!(strings(&img("a 2x 3h, b 3h 2x, c 1x")), ["c"]);
}

#[test]
fn descriptor_parentheses_preserve_commas_until_the_first_closing_parenthesis_or_eof() {
    let report = img("a (foo,bar), b 2x");
    assert_eq!(strings(&report), ["b"]);
    assert_eq!(errors(&report), [HtmlSrcsetParseError::RejectedCandidate]);
    let report = img("a (foo, b 2x");
    assert!(report.candidates().unwrap().is_empty());
    assert_eq!(errors(&report), [HtmlSrcsetParseError::RejectedCandidate]);
    // Parentheses do not nest: first right parenthesis restores comma handling.
    assert_eq!(strings(&img("a ((foo), b 2x")), ["b"]);
}

#[test]
fn exact_unbounded_width_and_large_exponent_do_not_expand_or_cap_language_values() {
    let magnitude = "340282366920938463463374607431768211456";
    let report = img(&format!("a {magnitude}w"));
    match report.candidates().unwrap()[0].descriptor() {
        HtmlParsedSrcsetDescriptor::Width(width) => {
            assert_eq!(width.value().decimal_digits(), magnitude)
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        report.author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
    let report = img("a 1e999999999999999999999999999999999999999x, b 2x");
    assert_eq!(strings(&report), ["b"]);
    assert_eq!(
        report.author(),
        HtmlSrcsetAuthorState::HtmlConstraintsConforming
    );
}

#[test]
fn raw_utf16_names_and_ranges_survive_all_observations_without_lossy_conversion() {
    let units = vec![0xd800, 0xd83d, 0xde00, 44, 32, 98, 32, 50, 120];
    let raw = attr("srcset", DocumentString::from_utf16(units.clone()));
    let name = QualifiedName::new(Namespace::html(), Some(DocumentString::from("h")), "img");
    let report = HtmlValueInterpreter::new(20)
        .img_srcset(&name, Some(&raw))
        .unwrap();
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.raw().unwrap().code_units(), &units);
    assert_eq!(report.element(), &name);
    let first = &report.candidates().unwrap()[0];
    assert_eq!(first.url().code_units(), &[0xd800, 0xd83d, 0xde00]);
    assert_eq!((first.url_range().start(), first.url_range().end()), (0, 3));
    assert_eq!((first.range().start(), first.range().end()), (0, 4));
    assert_eq!(report.candidates().unwrap()[1].url_range().start(), 5);
    let unknown = attr(
        "srcset",
        DocumentString::from_utf16(vec![97, 32, 0xdc00, 120]),
    );
    let report = HtmlValueInterpreter::new(20)
        .img_srcset(&element("img"), Some(&unknown))
        .unwrap();
    assert!(report.candidates().unwrap().is_empty());
    assert_eq!(report.raw_attribute(), Some(&unknown));
}

#[test]
fn missing_empty_whitespace_and_unapplied_roles_have_distinct_outcomes() {
    let interpreter = HtmlValueInterpreter::new(20);
    let missing = interpreter
        .img_srcset(&element("img"), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(
        missing.processing(),
        HtmlSrcsetProcessingState::MissingAttribute
    );
    assert_eq!(missing.author(), HtmlSrcsetAuthorState::Missing);
    assert_eq!(missing.candidates(), None);
    for value in ["", " \t\n\r\u{c}"] {
        let report = img(value);
        assert_eq!(report.author(), HtmlSrcsetAuthorState::Nonconforming);
        assert!(report.candidates().unwrap().is_empty());
        assert!(author_errors(&report).contains(&HtmlSrcsetAuthorViolation::EmptyList));
    }
    for (name, raw, expected) in [
        (
            element("source"),
            attr("srcset", "a"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            element("img"),
            attr("imagesrcset", "a"),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "img"),
            attr("srcset", "a"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            element("img"),
            Attribute::new(QualifiedName::new(Namespace::svg(), None, "srcset"), "a"),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = interpreter.img_srcset(&name, Some(&raw)).unwrap();
        assert_eq!(report.applicability(), expected);
        assert_eq!(report.processing(), HtmlSrcsetProcessingState::Unapplied);
        assert_eq!(report.author(), HtmlSrcsetAuthorState::Unapplied);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn exact_three_roles_preserve_distinct_defining_clauses_and_inventory_rows() {
    let interpreter = HtmlValueInterpreter::new(20);
    let reports = [
        interpreter
            .img_srcset(&element("img"), Some(&attr("srcset", "a 2x")))
            .unwrap(),
        interpreter
            .source_srcset(&element("source"), Some(&attr("srcset", "a 2x")))
            .unwrap(),
        interpreter
            .link_imagesrcset(&element("link"), Some(&attr("imagesrcset", "a 2x")))
            .unwrap(),
    ];
    for (report, anchor, rows) in [
        (reports[0].clone(), "#attr-img-srcset", &[152][..]),
        (reports[1].clone(), "#attr-source-srcset", &[152][..]),
        (reports[2].clone(), "#attr-link-imagesrcset", &[74][..]),
    ] {
        assert_eq!(strings(&report), ["a"]);
        assert!(report.source_clause().ends_with(anchor));
        assert_eq!(report.inventory_rows(), rows);
        assert!(
            report
                .grammar_source_clause()
                .ends_with("#srcset-attributes")
        );
        assert!(
            report
                .processing_source_clause()
                .ends_with("#parsing-a-srcset-attribute")
        );
        assert_eq!(
            report.external_conformance(),
            HtmlSrcsetExternalConformance::Unchecked
        );
    }
}

#[test]
fn img_presence_context_checks_sizes_and_src_without_erasing_lexical_errors() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (values, violations) in [
        (
            vec![("srcset", "a 1w")],
            vec![HtmlSrcsetContextViolation::SizesRequired],
        ),
        (vec![("srcset", "a 1w"), ("sizes", "")], vec![]),
        (
            vec![("srcset", "a 1x"), ("sizes", "")],
            vec![HtmlSrcsetContextViolation::WidthDescriptorsRequiredForSizes],
        ),
        (
            vec![],
            vec![HtmlSrcsetContextViolation::SrcOrSrcsetRequired],
        ),
        (vec![("src", "")], vec![]),
    ] {
        let context = HtmlSrcsetContext::Img {
            attributes: attributes(&values),
        };
        let report = interpreter
            .img_srcset_with_context(&element("img"), &context)
            .unwrap();
        assert_eq!(report.context_violations(), violations);
        assert_eq!(report.context(), Some(&context));
    }
    let context = HtmlSrcsetContext::Img {
        attributes: attributes(&[("srcset", "a 1w 2x")]),
    };
    let report = interpreter
        .img_srcset_with_context(&element("img"), &context)
        .unwrap();
    assert_eq!(report.author(), HtmlSrcsetAuthorState::Nonconforming);
    assert_eq!(
        report.context_author(),
        HtmlSrcsetContextAuthorState::Nonconforming
    );
    assert_eq!(
        report.context_violations(),
        [HtmlSrcsetContextViolation::SizesRequired]
    );
    assert_eq!(
        img("a 1w").context_author(),
        HtmlSrcsetContextAuthorState::Unchecked
    );
}

#[test]
fn source_picture_context_uses_actual_following_image_loading_and_sizes_text() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (loading, sizes, expected) in [
        ("lazy", "auto", false),
        ("LaZy", "AuTo, 50vw", false),
        ("eager", "auto", true),
        ("lazy", " auto", true),
        ("lazy", "automatic", true),
    ] {
        let context = HtmlSrcsetContext::Source {
            attributes: attributes(&[("srcset", "a 1w")]),
            parent: HtmlSrcsetParentObservation::Element(element("picture")),
            following_image: HtmlSrcsetFollowingImageObservation::Element {
                name: element("img"),
                attributes: attributes(&[("loading", loading), ("sizes", sizes)]),
            },
        };
        let report = interpreter
            .source_srcset_with_context(&element("source"), &context)
            .unwrap();
        assert_eq!(
            report
                .context_violations()
                .contains(&HtmlSrcsetContextViolation::SizesRequired),
            expected
        );
        assert_eq!(strings(&report), ["a"]);
    }
    let context = HtmlSrcsetContext::Source {
        attributes: attributes(&[("srcset", "a 1w")]),
        parent: HtmlSrcsetParentObservation::Element(element("picture")),
        following_image: HtmlSrcsetFollowingImageObservation::Unavailable,
    };
    assert_eq!(
        interpreter
            .source_srcset_with_context(&element("source"), &context)
            .unwrap()
            .context_author(),
        HtmlSrcsetContextAuthorState::Unchecked
    );
    let context = HtmlSrcsetContext::Source {
        attributes: attributes(&[]),
        parent: HtmlSrcsetParentObservation::Element(element("picture")),
        following_image: HtmlSrcsetFollowingImageObservation::Absent,
    };
    assert_eq!(
        interpreter
            .source_srcset_with_context(&element("source"), &context)
            .unwrap()
            .context_violations(),
        [HtmlSrcsetContextViolation::SourceSrcsetRequired]
    );
    let context = HtmlSrcsetContext::Source {
        attributes: attributes(&[("srcset", "a")]),
        parent: HtmlSrcsetParentObservation::Element(element("audio")),
        following_image: HtmlSrcsetFollowingImageObservation::Absent,
    };
    let report = interpreter
        .source_srcset_with_context(&element("source"), &context)
        .unwrap();
    assert_eq!(
        report.context_violations(),
        [HtmlSrcsetContextViolation::SourceParentNotPicture]
    );
    assert_eq!(strings(&report), ["a"]);
}

#[test]
fn link_preload_image_and_imagesizes_context_is_separate_from_literal_candidates() {
    let interpreter = HtmlValueInterpreter::new(100);
    let context = HtmlSrcsetContext::Link {
        attributes: attributes(&[
            ("imagesrcset", "a 1w"),
            ("rel", "alternate\tPrElOaD"),
            ("as", "ImAgE"),
            ("imagesizes", ""),
        ]),
    };
    let report = interpreter
        .link_imagesrcset_with_context(&element("link"), &context)
        .unwrap();
    assert_eq!(
        report.context_author(),
        HtmlSrcsetContextAuthorState::Conforming
    );
    assert!(report.context_violations().is_empty());
    let context = HtmlSrcsetContext::Link {
        attributes: attributes(&[
            ("imagesrcset", "a 1w"),
            ("rel", "preloaded"),
            ("as", " image"),
        ]),
    };
    let report = interpreter
        .link_imagesrcset_with_context(&element("link"), &context)
        .unwrap();
    assert_eq!(
        report.context_violations(),
        [
            HtmlSrcsetContextViolation::PreloadRelRequired,
            HtmlSrcsetContextViolation::ImageAsStateRequired,
            HtmlSrcsetContextViolation::ImagesizesRequired
        ]
    );
    assert_eq!(strings(&report), ["a"]);
    let context = HtmlSrcsetContext::Link {
        attributes: attributes(&[
            ("imagesrcset", "a 1x"),
            ("rel", "preload"),
            ("as", "image"),
            ("imagesizes", ""),
        ]),
    };
    assert_eq!(
        interpreter
            .link_imagesrcset_with_context(&element("link"), &context)
            .unwrap()
            .context_violations(),
        [HtmlSrcsetContextViolation::WidthDescriptorsRequiredForSizes]
    );
    let wrong = interpreter
        .img_srcset_with_context(&element("img"), &context)
        .unwrap();
    assert_eq!(
        wrong.applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(wrong.processing(), HtmlSrcsetProcessingState::Unapplied);
}

#[test]
fn finite_raw_admission_precedes_wrong_role_and_rechecks_surrounding_values() {
    let interpreter = HtmlValueInterpreter::new(3);
    let raw = attr("imagesrcset", "abcd");
    assert!(
        matches!(interpreter.img_srcset(&element("span"), Some(&raw)), Err(HtmlSrcsetInterpretationError::RawAdmission(error)) if error.required() == 4)
    );
    let context = HtmlSrcsetContext::Link {
        attributes: attributes(&[("imagesrcset", "a"), ("rel", "preload")]),
    };
    assert!(matches!(
        interpreter.link_imagesrcset_with_context(&element("link"), &context),
        Err(HtmlSrcsetInterpretationError::RawAdmission(_))
    ));
    let report = HtmlValueInterpreter::new(3)
        .img_srcset(&element("img"), Some(&attr("srcset", "a")))
        .unwrap();
    assert_eq!(report.input_code_units(), 1);
}

#[test]
fn unavailable_and_absent_parent_facts_are_distinct_and_prefixed_raw_attributes_stay_exact() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (parent, expected) in [
        (
            HtmlSrcsetParentObservation::Unavailable,
            HtmlSrcsetContextAuthorState::Unchecked,
        ),
        (
            HtmlSrcsetParentObservation::Absent,
            HtmlSrcsetContextAuthorState::Nonconforming,
        ),
    ] {
        let context = HtmlSrcsetContext::Source {
            attributes: attributes(&[("srcset", "a")]),
            parent,
            following_image: HtmlSrcsetFollowingImageObservation::Unavailable,
        };
        let report = interpreter
            .source_srcset_with_context(&element("source"), &context)
            .unwrap();
        assert_eq!(report.context_author(), expected);
        assert_eq!(report.context(), Some(&context));
        assert_eq!(strings(&report), ["a"]);
    }
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some(DocumentString::from("p")), "srcset"),
        "a 2x",
    );
    let report = interpreter.img_srcset(&element("img"), Some(&raw)).unwrap();
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(strings(&report), ["a"]);
    assert_eq!(
        report.author_fields().unwrap().as_document_string(),
        DocumentString::from("a 2x")
    );
}

#[test]
fn malformed_other_candidates_do_not_hide_established_nonwidth_sizes_violations() {
    let interpreter = HtmlValueInterpreter::new(100);
    for raw in ["a 1x, b bad", "b bad, a 1x", "a, b bad"] {
        let img_context = HtmlSrcsetContext::Img {
            attributes: attributes(&[("srcset", raw), ("sizes", "")]),
        };
        let source_context = HtmlSrcsetContext::Source {
            attributes: attributes(&[("srcset", raw), ("sizes", "")]),
            parent: HtmlSrcsetParentObservation::Element(element("picture")),
            following_image: HtmlSrcsetFollowingImageObservation::Element {
                name: element("img"),
                attributes: attributes(&[]),
            },
        };
        let link_context = HtmlSrcsetContext::Link {
            attributes: attributes(&[
                ("imagesrcset", raw),
                ("imagesizes", ""),
                ("rel", "preload"),
                ("as", "image"),
            ]),
        };
        let reports = [
            interpreter
                .img_srcset_with_context(&element("img"), &img_context)
                .unwrap(),
            interpreter
                .source_srcset_with_context(&element("source"), &source_context)
                .unwrap(),
            interpreter
                .link_imagesrcset_with_context(&element("link"), &link_context)
                .unwrap(),
        ];
        for report in reports {
            assert_eq!(report.author(), HtmlSrcsetAuthorState::Nonconforming);
            assert_eq!(
                report.context_violations(),
                [HtmlSrcsetContextViolation::WidthDescriptorsRequiredForSizes],
                "a source-established nonwidth candidate remains decisive: {raw}"
            );
            assert_eq!(
                report.context_author(),
                HtmlSrcsetContextAuthorState::Nonconforming
            );
            assert_eq!(strings(&report), ["a"]);
        }
    }
}
