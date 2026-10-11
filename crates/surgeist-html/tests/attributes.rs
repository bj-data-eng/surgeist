use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability as Applicability,
    HtmlAttributeAuthorState as Author, HtmlAttributeConformanceError as Conformance,
    HtmlAttributeObservation as Observation, HtmlAttributeRecognition as Recognition,
    HtmlAutocapitalize, HtmlAutocapitalizeState, HtmlAutocorrect, HtmlBooleanAttribute,
    HtmlContentEditable, HtmlContentEditableState, HtmlDir, HtmlDirState, HtmlDraggable,
    HtmlDraggableState, HtmlEnterKeyHint, HtmlHidden, HtmlHiddenState, HtmlInputMode, HtmlPopover,
    HtmlPopoverState, HtmlShadowRootMode, HtmlSlotAssignment, HtmlSpellcheck, HtmlSpellcheckState,
    HtmlTranslate, HtmlTranslateState, HtmlValueInterpreter, HtmlWritingSuggestions,
    HtmlWritingSuggestionsState, KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};

fn element(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}
fn attr(local: &str, value: &str) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
}

#[test]
fn boolean_false_text_is_nonconforming_but_presence_is_true() {
    // HTML boolean-attributes: authored false is invalid, not Boolean false.
    let interpreter = HtmlValueInterpreter::new(100);
    let element = element("div");
    let false_text = attr("inert", "false");
    let result = interpreter
        .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&false_text))
        .unwrap();
    assert_eq!(result.intrinsic(), Some(&true));
    assert_eq!(
        result.author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
    assert_eq!(result.observation(), Observation::PresentNonempty);
    assert_eq!(result.raw_attribute(), Some(&false_text));
    let omitted = interpreter
        .boolean_attribute(HtmlBooleanAttribute::Inert, &element, None)
        .unwrap();
    assert_eq!(omitted.intrinsic(), Some(&false));
    assert_eq!(omitted.observation(), Observation::Missing);
    let empty = attr("inert", "");
    let result = interpreter
        .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&empty))
        .unwrap();
    assert_eq!(result.intrinsic(), Some(&true));
    assert_eq!(result.author(), Author::Conforming);
    assert_eq!(result.observation(), Observation::PresentEmpty);
    let canonical_name = attr("inert", "INERT");
    assert_eq!(
        interpreter
            .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&canonical_name))
            .unwrap()
            .author(),
        Author::Conforming
    );
    let spaced = attr("inert", " inert ");
    assert_eq!(
        interpreter
            .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&spaced))
            .unwrap()
            .author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
}

#[test]
fn global_boolean_flags_apply_to_unknown_html_elements_and_template_flags_do_not() {
    let interpreter = HtmlValueInterpreter::new(100);
    let custom = element("x-thing");
    for (selected, name) in [
        (HtmlBooleanAttribute::Autofocus, "autofocus"),
        (HtmlBooleanAttribute::Inert, "inert"),
        (HtmlBooleanAttribute::Itemscope, "itemscope"),
        (HtmlBooleanAttribute::HeadingReset, "headingreset"),
    ] {
        let raw = attr(name, "");
        let report = interpreter
            .boolean_attribute(selected, &custom, Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(selected.into_attribute(true), Some(raw));
        assert_eq!(selected.into_attribute(false), None);
    }
    for (selected, name) in [
        (
            HtmlBooleanAttribute::ShadowRootClonable,
            "shadowrootclonable",
        ),
        (
            HtmlBooleanAttribute::ShadowRootCustomElementRegistry,
            "shadowrootcustomelementregistry",
        ),
        (
            HtmlBooleanAttribute::ShadowRootDelegatesFocus,
            "shadowrootdelegatesfocus",
        ),
        (
            HtmlBooleanAttribute::ShadowRootSerializable,
            "shadowrootserializable",
        ),
    ] {
        let raw = attr(name, "false");
        let report = interpreter
            .boolean_attribute(selected, &element("template"), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        let report = interpreter
            .boolean_attribute(selected, &custom, Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentElement);
        assert_eq!(report.author(), Author::Unapplied);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn enum_empty_defaults_are_specific_and_do_not_erase_empty_observation() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("div");
    macro_rules! empty {
        ($method:ident, $name:literal, $state:expr, $author:expr) => {{
            let raw = attr($name, "");
            let report = i.$method(&e, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&$state));
            assert_eq!(report.author(), $author);
            assert_eq!(report.observation(), Observation::PresentEmpty);
            assert_eq!(report.recognition(), Recognition::Empty);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }};
    }
    empty!(
        dir,
        "dir",
        HtmlDirState::Undefined,
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    empty!(
        draggable,
        "draggable",
        HtmlDraggableState::Auto,
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    empty!(
        contenteditable,
        "contenteditable",
        HtmlContentEditableState::Keyword(HtmlContentEditable::True),
        Author::Conforming
    );
    empty!(
        hidden,
        "hidden",
        HtmlHiddenState::Keyword(HtmlHidden::Hidden),
        Author::Conforming
    );
    empty!(
        popover,
        "popover",
        HtmlPopoverState::Keyword(HtmlPopover::Auto),
        Author::Conforming
    );
    empty!(
        translate,
        "translate",
        HtmlTranslateState::Keyword(HtmlTranslate::Yes),
        Author::Conforming
    );
    empty!(
        spellcheck,
        "spellcheck",
        HtmlSpellcheckState::Keyword(HtmlSpellcheck::True),
        Author::Conforming
    );
    empty!(
        autocorrect,
        "autocorrect",
        HtmlAutocorrect::On,
        Author::Conforming
    );
    empty!(
        writingsuggestions,
        "writingsuggestions",
        HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True),
        Author::Conforming
    );
    empty!(
        autocapitalize,
        "autocapitalize",
        HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences),
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
}

#[test]
fn missing_and_unknown_keywords_keep_different_default_and_author_states() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("div");
    macro_rules! defaults {
        ($method:ident, $name:literal, $missing:expr, $invalid:expr) => {{
            let report = i.$method(&e, None).unwrap();
            assert_eq!(report.intrinsic(), Some(&$missing));
            assert_eq!(report.author(), Author::Missing);
            assert_eq!(report.recognition(), Recognition::Missing);
            let raw = attr($name, "unknown");
            let report = i.$method(&e, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&$invalid));
            assert_eq!(
                report.author(),
                Author::Nonconforming(Conformance::UnrecognizedKeyword)
            );
            assert_eq!(report.recognition(), Recognition::Unrecognized);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }};
    }
    defaults!(dir, "dir", HtmlDirState::Undefined, HtmlDirState::Undefined);
    defaults!(
        contenteditable,
        "contenteditable",
        HtmlContentEditableState::Inherit,
        HtmlContentEditableState::Inherit
    );
    defaults!(
        draggable,
        "draggable",
        HtmlDraggableState::Auto,
        HtmlDraggableState::Auto
    );
    defaults!(
        hidden,
        "hidden",
        HtmlHiddenState::NotHidden,
        HtmlHiddenState::Keyword(HtmlHidden::Hidden)
    );
    defaults!(
        popover,
        "popover",
        HtmlPopoverState::NoPopover,
        HtmlPopoverState::Keyword(HtmlPopover::Manual)
    );
    defaults!(
        translate,
        "translate",
        HtmlTranslateState::Inherit,
        HtmlTranslateState::Inherit
    );
    defaults!(
        spellcheck,
        "spellcheck",
        HtmlSpellcheckState::Default,
        HtmlSpellcheckState::Default
    );
    defaults!(
        autocapitalize,
        "autocapitalize",
        HtmlAutocapitalizeState::Default,
        HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)
    );
    defaults!(
        autocorrect,
        "autocorrect",
        HtmlAutocorrect::On,
        HtmlAutocorrect::On
    );
    defaults!(
        writingsuggestions,
        "writingsuggestions",
        HtmlWritingSuggestionsState::Default,
        HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)
    );
}

#[test]
fn autocapitalize_aliases_are_recognized_and_canonical_authoring_is_explicit() {
    // attr-autocapitalize getter explicitly returns none/sentences for aliases.
    let i = HtmlValueInterpreter::new(100);
    for (raw_value, matched, canonical, state) in [
        ("OFF", "off", "none", HtmlAutocapitalize::None),
        ("none", "none", "none", HtmlAutocapitalize::None),
        ("ON", "on", "sentences", HtmlAutocapitalize::Sentences),
        (
            "sentences",
            "sentences",
            "sentences",
            HtmlAutocapitalize::Sentences,
        ),
        ("words", "words", "words", HtmlAutocapitalize::Words),
        (
            "characters",
            "characters",
            "characters",
            HtmlAutocapitalize::Characters,
        ),
    ] {
        let raw = attr("autocapitalize", raw_value);
        let report = i.autocapitalize(&element("input"), Some(&raw)).unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlAutocapitalizeState::Keyword(state))
        );
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(
            report.recognition(),
            Recognition::Keyword { matched, canonical }
        );
        assert_eq!(state.into_attribute(), attr("autocapitalize", canonical));
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn bdo_required_direction_conformance_is_independent_of_intrinsic_state() {
    let i = HtmlValueInterpreter::new(100);
    let bdo = element("bdo");
    let missing = i.dir(&bdo, None).unwrap();
    assert_eq!(missing.intrinsic(), Some(&HtmlDirState::Undefined));
    assert_eq!(
        missing.author(),
        Author::Nonconforming(Conformance::BdoDirectionRequired)
    );
    assert_eq!(
        missing.context_source_clause(),
        Some(
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-bdo-element"
        )
    );
    let auto = attr("dir", "auto");
    let report = i.dir(&bdo, Some(&auto)).unwrap();
    assert_eq!(
        report.intrinsic(),
        Some(&HtmlDirState::Keyword(HtmlDir::Auto))
    );
    assert_eq!(
        report.recognition(),
        Recognition::Keyword {
            matched: "auto",
            canonical: "auto"
        }
    );
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::BdoDirectionMustBeLtrOrRtl)
    );
    for value in ["ltr", "RTL"] {
        let raw = attr("dir", value);
        assert_eq!(
            i.dir(&bdo, Some(&raw)).unwrap().author(),
            Author::Conforming
        );
    }
    assert_eq!(
        i.dir(&element("div"), Some(&auto)).unwrap().author(),
        Author::Conforming
    );
    // Exact uppercase local name is an unknown HTML name, not the bdo identity.
    assert_eq!(
        i.dir(&element("BDO"), None).unwrap().author(),
        Author::Missing
    );
}

#[test]
fn keyboard_hints_have_no_guessed_missing_invalid_or_empty_default() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("input");
    assert_eq!(i.inputmode(&e, None).unwrap().intrinsic(), None);
    assert_eq!(i.enterkeyhint(&e, None).unwrap().intrinsic(), None);
    for value in ["", "unknown", " text ", "ｔｅｘｔ"] {
        let raw = attr("inputmode", value);
        let report = i.inputmode(&e, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), None);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
    }
    for value in ["", "unknown", " enter "] {
        let raw = attr("enterkeyhint", value);
        assert_eq!(i.enterkeyhint(&e, Some(&raw)).unwrap().intrinsic(), None);
    }
    assert_eq!(
        i.inputmode(&e, Some(&attr("inputmode", "DECIMAL")))
            .unwrap()
            .intrinsic(),
        Some(&HtmlInputMode::Decimal)
    );
    assert_eq!(
        i.enterkeyhint(&e, Some(&attr("enterkeyhint", "NEXT")))
            .unwrap()
            .intrinsic(),
        Some(&HtmlEnterKeyHint::Next)
    );
}

#[test]
fn keyboard_keyword_tables_select_their_own_concrete_hints() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("input");
    for (value, expected) in [
        ("none", HtmlInputMode::None),
        ("text", HtmlInputMode::Text),
        ("tel", HtmlInputMode::Tel),
        ("url", HtmlInputMode::Url),
        ("email", HtmlInputMode::Email),
        ("numeric", HtmlInputMode::Numeric),
        ("decimal", HtmlInputMode::Decimal),
        ("search", HtmlInputMode::Search),
    ] {
        let raw = attr("inputmode", value);
        let report = i.inputmode(&e, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&expected));
        assert_eq!(report.author(), Author::Conforming);
    }
    for (value, expected) in [
        ("enter", HtmlEnterKeyHint::Enter),
        ("done", HtmlEnterKeyHint::Done),
        ("go", HtmlEnterKeyHint::Go),
        ("next", HtmlEnterKeyHint::Next),
        ("previous", HtmlEnterKeyHint::Previous),
        ("search", HtmlEnterKeyHint::Search),
        ("send", HtmlEnterKeyHint::Send),
    ] {
        let raw = attr("enterkeyhint", value);
        let report = i.enterkeyhint(&e, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&expected));
        assert_eq!(report.author(), Author::Conforming);
    }
}

#[test]
fn foreign_and_mismatched_expanded_names_are_unapplied_and_exactly_retained() {
    let i = HtmlValueInterpreter::new(100);
    let foreign_element = QualifiedName::new(Namespace::svg(), Some("s".into()), "div");
    let raw = attr("hidden", "until-found");
    let report = i.hidden(&foreign_element, Some(&raw)).unwrap();
    assert_eq!(
        report.applicability(),
        Applicability::ForeignElementNamespace
    );
    assert_eq!(report.author(), Author::Unapplied);
    assert_eq!(report.recognition(), Recognition::Unapplied);
    assert_eq!(report.intrinsic(), None);
    assert_eq!(report.element(), &foreign_element);
    assert_eq!(report.raw_attribute(), Some(&raw));
    for name in [
        QualifiedName::new(Namespace::none(), None, "HIDDEN"),
        QualifiedName::new(Namespace::svg(), Some("s".into()), "hidden"),
        QualifiedName::new(
            Namespace::none(),
            None,
            DocumentString::from_utf16(vec![0xd800]),
        ),
    ] {
        let raw = Attribute::new(name, "false");
        let report = i.hidden(&element("div"), Some(&raw)).unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentAttribute);
        assert_eq!(report.author(), Author::Unapplied);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn prefixes_do_not_change_expanded_identity_and_raw_surrogates_are_not_folded() {
    let i = HtmlValueInterpreter::new(100);
    let e = QualifiedName::new(
        Namespace::html(),
        Some(DocumentString::from_utf16(vec![0xd800])),
        "x-widget",
    );
    let name = QualifiedName::new(Namespace::none(), Some("p".into()), "popover");
    let raw = Attribute::new(name, DocumentString::from_utf16(vec![0xd800, 0x20, 0xdc00]));
    let report = i.popover(&e, Some(&raw)).unwrap();
    assert_eq!(report.applicability(), Applicability::Applicable);
    assert_eq!(
        report.intrinsic(),
        Some(&HtmlPopoverState::Keyword(HtmlPopover::Manual))
    );
    assert_eq!(report.recognition(), Recognition::Unrecognized);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.element(), &e);
    assert_eq!(report.raw().unwrap().code_units(), &[0xd800, 0x20, 0xdc00]);
}

#[test]
fn raw_value_admission_precedes_even_unapplied_name_processing() {
    let i = HtmlValueInterpreter::new(0);
    let foreign = QualifiedName::new(Namespace::svg(), None, "thing");
    let raw = attr("other", "x");
    let error = i.hidden(&foreign, Some(&raw)).unwrap_err();
    assert_eq!(error.required(), 1);
    assert_eq!(error.capacity(), 0);
    assert_eq!(
        i.hidden(&element("div"), None).unwrap().intrinsic(),
        Some(&HtmlHiddenState::NotHidden)
    );
    let raw = attr("hidden", "");
    assert_eq!(
        i.hidden(&element("div"), Some(&raw)).unwrap().intrinsic(),
        Some(&HtmlHiddenState::Keyword(HtmlHidden::Hidden))
    );
    let raw = Attribute::new(
        KnownHtmlAttribute::Hidden.qualified_name(),
        DocumentString::from_utf16(vec![0xd83d, 0xde00]),
    );
    assert_eq!(
        HtmlValueInterpreter::new(1)
            .hidden(&element("div"), Some(&raw))
            .unwrap_err()
            .required(),
        2
    );
}

#[test]
fn declarative_modes_are_template_only_and_do_not_claim_root_creation() {
    let i = HtmlValueInterpreter::new(100);
    let template = KnownHtmlElement::Template.qualified_name();
    assert_eq!(i.shadowrootmode(&template, None).unwrap().intrinsic(), None);
    let bad = attr("shadowrootmode", "");
    let report = i.shadowrootmode(&template, Some(&bad)).unwrap();
    assert_eq!(report.intrinsic(), None);
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    for (value, expected) in [
        ("OPEN", HtmlShadowRootMode::Open),
        ("closed", HtmlShadowRootMode::Closed),
    ] {
        let raw = attr("shadowrootmode", value);
        assert_eq!(
            i.shadowrootmode(&template, Some(&raw)).unwrap().intrinsic(),
            Some(&expected)
        );
        let report = i.shadowrootmode(&element("div"), Some(&raw)).unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentElement);
        assert_eq!(report.intrinsic(), None);
    }
    assert_eq!(
        HtmlShadowRootMode::Open.into_shadowrootmode_attribute(),
        attr("shadowrootmode", "open")
    );
    assert_eq!(
        i.shadowrootslotassignment(&template, None)
            .unwrap()
            .intrinsic(),
        Some(&HtmlSlotAssignment::Named)
    );
    let invalid = attr("shadowrootslotassignment", "wrong");
    assert_eq!(
        i.shadowrootslotassignment(&template, Some(&invalid))
            .unwrap()
            .intrinsic(),
        Some(&HtmlSlotAssignment::Named)
    );
    let manual = HtmlSlotAssignment::Manual.into_shadowrootslotassignment_attribute();
    assert_eq!(manual, attr("shadowrootslotassignment", "manual"));
    assert_eq!(
        i.shadowrootslotassignment(&template, Some(&manual))
            .unwrap()
            .intrinsic(),
        Some(&HtmlSlotAssignment::Manual)
    );
}

#[test]
fn native_global_keywords_construct_real_attributes_and_typed_states() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("div");
    macro_rules! native {
        ($method:ident, $name:literal, $keyword:literal, $native:expr, $state:expr) => {{
            let raw = $native.into_attribute();
            assert_eq!(raw, attr($name, $keyword));
            let report = i.$method(&e, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&$state));
            assert_eq!(report.author(), Author::Conforming);
        }};
    }
    native!(
        dir,
        "dir",
        "rtl",
        HtmlDir::Rtl,
        HtmlDirState::Keyword(HtmlDir::Rtl)
    );
    native!(
        contenteditable,
        "contenteditable",
        "plaintext-only",
        HtmlContentEditable::PlaintextOnly,
        HtmlContentEditableState::Keyword(HtmlContentEditable::PlaintextOnly)
    );
    native!(
        draggable,
        "draggable",
        "false",
        HtmlDraggable::False,
        HtmlDraggableState::Keyword(HtmlDraggable::False)
    );
    native!(
        hidden,
        "hidden",
        "until-found",
        HtmlHidden::UntilFound,
        HtmlHiddenState::Keyword(HtmlHidden::UntilFound)
    );
    native!(
        popover,
        "popover",
        "hint",
        HtmlPopover::Hint,
        HtmlPopoverState::Keyword(HtmlPopover::Hint)
    );
    native!(
        translate,
        "translate",
        "no",
        HtmlTranslate::No,
        HtmlTranslateState::Keyword(HtmlTranslate::No)
    );
    native!(
        spellcheck,
        "spellcheck",
        "false",
        HtmlSpellcheck::False,
        HtmlSpellcheckState::Keyword(HtmlSpellcheck::False)
    );
    native!(
        autocorrect,
        "autocorrect",
        "off",
        HtmlAutocorrect::Off,
        HtmlAutocorrect::Off
    );
    native!(
        writingsuggestions,
        "writingsuggestions",
        "false",
        HtmlWritingSuggestions::False,
        HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::False)
    );
    native!(
        inputmode,
        "inputmode",
        "numeric",
        HtmlInputMode::Numeric,
        HtmlInputMode::Numeric
    );
    native!(
        enterkeyhint,
        "enterkeyhint",
        "send",
        HtmlEnterKeyHint::Send,
        HtmlEnterKeyHint::Send
    );
}

#[test]
fn metadata_selects_actual_defining_headingreset_clause_and_existing_rows() {
    // The retained non-normative index incorrectly links headingreset to tabindex.
    let report = HtmlValueInterpreter::new(0)
        .boolean_attribute(HtmlBooleanAttribute::HeadingReset, &element("h1"), None)
        .unwrap();
    assert_eq!(report.attribute(), KnownHtmlAttribute::Headingreset);
    assert_eq!(report.inventory_rows(), &[63]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingreset"
    );
    assert_eq!(report.intrinsic(), Some(&false));
}
