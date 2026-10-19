use surgeist_html::*;

const SOURCE: &str = "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-th-scope";

fn th() -> QualifiedName {
    KnownHtmlElement::Th.qualified_name()
}

fn scope(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Scope.qualified_name(), value)
}

#[test]
fn four_exact_ascii_case_insensitive_keywords_select_conforming_states() {
    // Literal expectations from the four rows of HTML's th scope table.
    for (spelling, canonical, keyword, context) in [
        (
            "row",
            "row",
            HtmlTableHeaderScope::Row,
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "rOw",
            "row",
            HtmlTableHeaderScope::Row,
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "col",
            "col",
            HtmlTableHeaderScope::Column,
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "CoL",
            "col",
            HtmlTableHeaderScope::Column,
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "rowgroup",
            "rowgroup",
            HtmlTableHeaderScope::RowGroup,
            HtmlAttributeContextAuthorState::Unchecked,
        ),
        (
            "RoWgRoUp",
            "rowgroup",
            HtmlTableHeaderScope::RowGroup,
            HtmlAttributeContextAuthorState::Unchecked,
        ),
        (
            "colgroup",
            "colgroup",
            HtmlTableHeaderScope::ColumnGroup,
            HtmlAttributeContextAuthorState::Unchecked,
        ),
        (
            "CoLgRoUp",
            "colgroup",
            HtmlTableHeaderScope::ColumnGroup,
            HtmlAttributeContextAuthorState::Unchecked,
        ),
    ] {
        let attribute = scope(spelling);
        let report = HtmlValueInterpreter::new(attribute.value().len())
            .th_scope(&th(), Some(&attribute))
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(
            report.observation(),
            HtmlAttributeObservation::PresentNonempty
        );
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::Keyword {
                matched: canonical,
                canonical
            }
        );
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlTableHeaderScopeState::Keyword(keyword))
        );
        assert_eq!(report.context_author(), context);
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.raw(), Some(attribute.value()));
    }
}

#[test]
fn absent_and_empty_select_auto_with_distinct_raw_author_and_presence_evidence() {
    let interpreter = HtmlValueInterpreter::new(0);
    let missing = interpreter
        .th_scope(&th(), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(missing.intrinsic(), Some(&HtmlTableHeaderScopeState::Auto));
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(missing.raw_attribute(), None);
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.input_code_units(), 0);

    let attribute = scope("");
    let empty = interpreter.th_scope(&th(), Some(&attribute)).unwrap();
    assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(empty.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(
        empty.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::UnrecognizedKeyword)
    );
    assert_eq!(empty.intrinsic(), Some(&HtmlTableHeaderScopeState::Auto));
    assert_eq!(
        empty.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(empty.raw_attribute(), Some(&attribute));
    assert_eq!(empty.raw(), Some(attribute.value()));
}

#[test]
fn auto_unknown_padding_lists_and_non_ascii_lookalikes_remain_invalid_auto_content() {
    for spelling in [
        "auto",
        "AUTO",
        "column",
        "row-group",
        "row col",
        "row,col",
        " row",
        "row ",
        "\tcol",
        "col\n",
        "\rrowgroup",
        "colgroup\u{c}",
        "row\0",
        "rοw",
        "ｒｏｗ",
        "row\u{a0}",
        "unknown",
    ] {
        let attribute = scope(spelling);
        let report = HtmlValueInterpreter::new(attribute.value().len())
            .th_scope(&th(), Some(&attribute))
            .unwrap();
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::Unrecognized,
            "{spelling:?}"
        );
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(report.intrinsic(), Some(&HtmlTableHeaderScopeState::Auto));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
}

#[test]
fn isolated_surrogates_and_astral_suffixes_are_preserved_without_keyword_salvage() {
    for units in [
        vec![0xD800],
        vec![0xDC00],
        vec![114, 111, 119, 0xD800],
        vec![99, 111, 108, 0xD83D, 0xDE00],
    ] {
        let attribute = scope(DocumentString::from_utf16(units.clone()));
        let report = HtmlValueInterpreter::new(units.len())
            .th_scope(&th(), Some(&attribute))
            .unwrap();
        assert_eq!(report.raw().unwrap().code_units(), units);
        assert_eq!(report.input_code_units(), units.len());
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(report.intrinsic(), Some(&HtmlTableHeaderScopeState::Auto));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn canonical_native_keywords_emit_actual_attributes_without_proving_group_anchoring() {
    for (keyword, spelling, context) in [
        (
            HtmlTableHeaderScope::Row,
            "row",
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            HtmlTableHeaderScope::Column,
            "col",
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            HtmlTableHeaderScope::RowGroup,
            "rowgroup",
            HtmlAttributeContextAuthorState::Unchecked,
        ),
        (
            HtmlTableHeaderScope::ColumnGroup,
            "colgroup",
            HtmlAttributeContextAuthorState::Unchecked,
        ),
    ] {
        assert_eq!(keyword.canonical_keyword(), spelling);
        let attribute = keyword.into_attribute();
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Scope.qualified_name()
        );
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(attribute.name().prefix(), None);
        assert_eq!(
            attribute.value().code_units(),
            spelling.encode_utf16().collect::<Vec<_>>()
        );
        let report = HtmlValueInterpreter::new(8)
            .th_scope(&th(), Some(&attribute))
            .unwrap();
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlTableHeaderScopeState::Keyword(keyword))
        );
        assert_eq!(report.context_author(), context);
    }
}

#[test]
fn wrong_element_namespaces_and_exact_local_names_do_not_run_scope_grammar() {
    for (element, applicability) in [
        (
            KnownHtmlElement::Td.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            KnownHtmlElement::Div.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "TH"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "x-th"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "th"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "th"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
    ] {
        for value in [None, Some("rowgroup"), Some("auto")] {
            let attribute = value.map(scope);
            let report = HtmlValueInterpreter::new(8)
                .th_scope(&element, attribute.as_ref())
                .unwrap();
            assert_eq!(report.element(), &element);
            assert_eq!(report.raw_attribute(), attribute.as_ref());
            assert_eq!(report.applicability(), applicability);
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
            assert_eq!(report.intrinsic(), None);
        }
    }
}

#[test]
fn wrong_expanded_attribute_identity_keeps_raw_and_does_not_select_auto() {
    for name in [
        KnownHtmlAttribute::Headers.qualified_name(),
        QualifiedName::new(Namespace::none(), None, "SCOPE"),
        QualifiedName::new(Namespace::html(), None, "scope"),
        QualifiedName::new(Namespace::from_uri("urn:scope"), Some("a".into()), "scope"),
    ] {
        let attribute = Attribute::new(name, "rowgroup");
        let report = HtmlValueInterpreter::new(8)
            .th_scope(&th(), Some(&attribute))
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        assert_eq!(report.intrinsic(), None);
    }
}

#[test]
fn prefixes_and_normative_source_metadata_are_independent_of_lexical_defaults() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "th");
    for value in [None, Some("ROWGROUP"), Some(""), Some("auto")] {
        let attribute = value.map(|s| {
            Attribute::new(
                QualifiedName::new(Namespace::none(), Some("a".into()), "scope"),
                s,
            )
        });
        let report = HtmlValueInterpreter::new(8)
            .th_scope(&element, attribute.as_ref())
            .unwrap();
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), attribute.as_ref());
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.attribute(), KnownHtmlAttribute::Scope);
        assert_eq!(report.inventory_rows(), &[134]);
        assert_eq!(report.source_clause(), SOURCE);
        assert_eq!(report.context_source_clause(), Some(SOURCE));
        assert_eq!(report.vocabulary_source_clause(), None);
        assert_eq!(report.input_state(), None);
    }
    // Existing unrelated reports do not acquire scope provenance.
    let row_type = HtmlValueInterpreter::new(8)
        .ol_type(
            &KnownHtmlElement::Ol.qualified_name(),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(row_type.context_source_clause(), None);
}

#[test]
fn original_utf16_capacity_is_checked_before_role_and_keyword_processing() {
    let units = vec![114, 111, 119, 0xD83D, 0xDE00, 0xD800];
    let attribute = scope(DocumentString::from_utf16(units.clone()));
    for element in [th(), KnownHtmlElement::Div.qualified_name()] {
        let error = HtmlValueInterpreter::new(5)
            .th_scope(&element, Some(&attribute))
            .unwrap_err();
        assert_eq!(error.capacity(), 5);
        assert_eq!(error.required(), 6);
        let admitted = HtmlValueInterpreter::new(6)
            .th_scope(&element, Some(&attribute))
            .unwrap();
        assert_eq!(admitted.raw().unwrap().code_units(), units);
    }
    let wrong_attribute = Attribute::new(KnownHtmlAttribute::Title.qualified_name(), "rowgroup");
    let error = HtmlValueInterpreter::new(7)
        .th_scope(&th(), Some(&wrong_attribute))
        .unwrap_err();
    assert_eq!(error.capacity(), 7);
    assert_eq!(error.required(), 8);
    let group = scope("colgroup");
    let exact = HtmlValueInterpreter::new(8)
        .th_scope(&th(), Some(&group))
        .unwrap();
    assert_eq!(
        exact.intrinsic(),
        Some(&HtmlTableHeaderScopeState::Keyword(
            HtmlTableHeaderScope::ColumnGroup
        ))
    );
    assert_eq!(
        exact.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        HtmlValueInterpreter::new(7)
            .th_scope(&th(), Some(&group))
            .unwrap_err()
            .required(),
        8
    );
}
