use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeObservation,
    HtmlAutocompleteContactHint, HtmlAutocompleteContextAuthorState, HtmlAutocompleteCredential,
    HtmlAutocompleteCredentialOutcome, HtmlAutocompleteDefaultField, HtmlAutocompleteDefaultReason,
    HtmlAutocompleteDetails, HtmlAutocompleteMantle, HtmlAutocompleteMode,
    HtmlAutocompleteProcessed, HtmlAutocompleteProcessingState, HtmlAutocompleteReport,
    HtmlAutocompleteSection, HtmlAutocompleteSectionError, HtmlAutocompleteSyntaxError,
    HtmlAutocompleteSyntaxState, HtmlAutocompleteValue, HtmlAutofillControlGroup,
    HtmlAutofillField, HtmlAutofillFieldCategory, HtmlAutofillFieldIdentity, HtmlFormAutocomplete,
    HtmlInputType, HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, Namespace,
    QualifiedName,
};

fn raw(value: &str) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Autocomplete.qualified_name(), value)
}
fn textarea(value: &str) -> HtmlAutocompleteReport {
    HtmlValueInterpreter::new(200)
        .control_autocomplete(
            &KnownHtmlElement::Textarea.qualified_name(),
            Some(&raw(value)),
        )
        .unwrap()
}
fn input(value: &str, state: HtmlInputType) -> HtmlAutocompleteReport {
    HtmlValueInterpreter::new(200)
        .control_autocomplete_for_input(
            &KnownHtmlElement::Input.qualified_name(),
            Some(&raw(value)),
            state,
        )
        .unwrap()
}
fn processed(report: &HtmlAutocompleteReport) -> &HtmlAutocompleteProcessed {
    let HtmlAutocompleteProcessingState::Processed(value) = report.processing() else {
        panic!("expected processed value: {:?}", report.processing());
    };
    value
}
fn assert_default(
    report: &HtmlAutocompleteReport,
    reason: HtmlAutocompleteDefaultReason,
    field: HtmlAutocompleteDefaultField,
) {
    let HtmlAutocompleteProcessingState::Defaulted(value) = report.processing() else {
        panic!("expected source default: {:?}", report.processing());
    };
    assert_eq!(value.reason(), reason);
    assert_eq!(value.field(), field);
    assert!(value.idl_value().is_empty());
    assert!(value.scope().tokens().is_empty());
    assert_eq!(value.mode_hint(), None);
    assert_eq!(value.contact_hint(), None);
}

// Independent literal vocabulary/control groups from the defining autofill
// field table, not exported implementation metadata or inventory summaries.
const FIELDS: &[(HtmlAutofillField, &str, HtmlAutofillControlGroup)] = &[
    (
        HtmlAutofillField::Name,
        "name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::HonorificPrefix,
        "honorific-prefix",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::GivenName,
        "given-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AdditionalName,
        "additional-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::FamilyName,
        "family-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::HonorificSuffix,
        "honorific-suffix",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::Nickname,
        "nickname",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::OrganizationTitle,
        "organization-title",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::Username,
        "username",
        HtmlAutofillControlGroup::Username,
    ),
    (
        HtmlAutofillField::NewPassword,
        "new-password",
        HtmlAutofillControlGroup::Password,
    ),
    (
        HtmlAutofillField::CurrentPassword,
        "current-password",
        HtmlAutofillControlGroup::Password,
    ),
    (
        HtmlAutofillField::OneTimeCode,
        "one-time-code",
        HtmlAutofillControlGroup::Password,
    ),
    (
        HtmlAutofillField::Organization,
        "organization",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::StreetAddress,
        "street-address",
        HtmlAutofillControlGroup::Multiline,
    ),
    (
        HtmlAutofillField::AddressLine1,
        "address-line1",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AddressLine2,
        "address-line2",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AddressLine3,
        "address-line3",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AddressLevel4,
        "address-level4",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AddressLevel3,
        "address-level3",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AddressLevel2,
        "address-level2",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::AddressLevel1,
        "address-level1",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::Country,
        "country",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CountryName,
        "country-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::PostalCode,
        "postal-code",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcName,
        "cc-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcGivenName,
        "cc-given-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcAdditionalName,
        "cc-additional-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcFamilyName,
        "cc-family-name",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcNumber,
        "cc-number",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcExp,
        "cc-exp",
        HtmlAutofillControlGroup::Month,
    ),
    (
        HtmlAutofillField::CcExpMonth,
        "cc-exp-month",
        HtmlAutofillControlGroup::Numeric,
    ),
    (
        HtmlAutofillField::CcExpYear,
        "cc-exp-year",
        HtmlAutofillControlGroup::Numeric,
    ),
    (
        HtmlAutofillField::CcCsc,
        "cc-csc",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::CcType,
        "cc-type",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TransactionCurrency,
        "transaction-currency",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TransactionAmount,
        "transaction-amount",
        HtmlAutofillControlGroup::Numeric,
    ),
    (
        HtmlAutofillField::Language,
        "language",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::Bday,
        "bday",
        HtmlAutofillControlGroup::Date,
    ),
    (
        HtmlAutofillField::BdayDay,
        "bday-day",
        HtmlAutofillControlGroup::Numeric,
    ),
    (
        HtmlAutofillField::BdayMonth,
        "bday-month",
        HtmlAutofillControlGroup::Numeric,
    ),
    (
        HtmlAutofillField::BdayYear,
        "bday-year",
        HtmlAutofillControlGroup::Numeric,
    ),
    (
        HtmlAutofillField::Sex,
        "sex",
        HtmlAutofillControlGroup::Text,
    ),
    (HtmlAutofillField::Url, "url", HtmlAutofillControlGroup::Url),
    (
        HtmlAutofillField::Photo,
        "photo",
        HtmlAutofillControlGroup::Url,
    ),
    (HtmlAutofillField::Tel, "tel", HtmlAutofillControlGroup::Tel),
    (
        HtmlAutofillField::TelCountryCode,
        "tel-country-code",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TelNational,
        "tel-national",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TelAreaCode,
        "tel-area-code",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TelLocal,
        "tel-local",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TelLocalPrefix,
        "tel-local-prefix",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TelLocalSuffix,
        "tel-local-suffix",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::TelExtension,
        "tel-extension",
        HtmlAutofillControlGroup::Text,
    ),
    (
        HtmlAutofillField::Email,
        "email",
        HtmlAutofillControlGroup::Username,
    ),
    (
        HtmlAutofillField::Impp,
        "impp",
        HtmlAutofillControlGroup::Url,
    ),
];

#[test]
fn each_defined_field_constructs_and_interprets_its_real_keyword_and_group() {
    for &(field, keyword, group) in FIELDS {
        let attribute = HtmlAutocompleteDetails::new(field).into_attribute();
        assert_eq!(attribute, raw(keyword));
        let report = textarea(keyword);
        let HtmlAutocompleteSyntaxState::Valid(HtmlAutocompleteValue::Details(details)) =
            report.syntax()
        else {
            panic!("detail field required")
        };
        assert_eq!(details.field(), field);
        assert_eq!(field.control_group(), group);
        assert_eq!(field.canonical_keyword(), keyword);
        assert_eq!(
            report.context_author(),
            HtmlAutocompleteContextAuthorState::Conforming
        );
        assert_eq!(
            processed(&report).field_name(),
            &DocumentString::from(keyword)
        );
        assert_eq!(
            processed(&report).field_identity(),
            HtmlAutofillFieldIdentity::Detail(field)
        );
        let uppercase = keyword.to_ascii_uppercase();
        let report = textarea(&uppercase);
        assert_eq!(
            processed(&report).field_name(),
            &DocumentString::from(uppercase.clone())
        );
        assert_eq!(
            processed(&report).idl_value(),
            &DocumentString::from(uppercase)
        );
        assert_eq!(
            processed(&report).credential_type(),
            HtmlAutocompleteCredentialOutcome::Null
        );
    }
}

#[test]
fn contact_category_and_each_hint_are_restricted_to_actual_contact_fields() {
    let contacts = [
        HtmlAutofillField::Tel,
        HtmlAutofillField::TelCountryCode,
        HtmlAutofillField::TelNational,
        HtmlAutofillField::TelAreaCode,
        HtmlAutofillField::TelLocal,
        HtmlAutofillField::TelLocalPrefix,
        HtmlAutofillField::TelLocalSuffix,
        HtmlAutofillField::TelExtension,
        HtmlAutofillField::Email,
        HtmlAutofillField::Impp,
    ];
    for &(field, _, _) in FIELDS {
        assert_eq!(
            field.category(),
            if contacts.contains(&field) {
                HtmlAutofillFieldCategory::Contact
            } else {
                HtmlAutofillFieldCategory::Normal
            }
        );
        let result = HtmlAutocompleteDetails::new(field)
            .with_contact_hint(HtmlAutocompleteContactHint::Home);
        assert_eq!(result.is_ok(), contacts.contains(&field));
    }
    for (hint, keyword) in [
        (HtmlAutocompleteContactHint::Home, "home"),
        (HtmlAutocompleteContactHint::Work, "work"),
        (HtmlAutocompleteContactHint::Mobile, "mobile"),
        (HtmlAutocompleteContactHint::Fax, "fax"),
        (HtmlAutocompleteContactHint::Pager, "pager"),
    ] {
        let native = HtmlAutocompleteDetails::new(HtmlAutofillField::Email)
            .with_contact_hint(hint)
            .unwrap()
            .into_attribute();
        assert_eq!(native, raw(&format!("{keyword} email")));
        let report = textarea(&format!("{keyword} email"));
        assert_eq!(processed(&report).contact_hint(), Some(hint));
        assert_eq!(
            processed(&report).scope().as_document_string(),
            DocumentString::from(keyword)
        );
    }
    assert_eq!(
        textarea("home name").syntax(),
        &HtmlAutocompleteSyntaxState::Invalid(
            HtmlAutocompleteSyntaxError::ContactHintRequiresContactField {
                field: HtmlAutofillField::Name
            }
        )
    );
}

#[test]
fn native_details_enforce_order_and_real_field_while_retaining_section_text() {
    let details = HtmlAutocompleteDetails::new(HtmlAutofillField::Tel)
        .with_section(HtmlAutocompleteSection::new("SECTION-Family").unwrap())
        .with_mode(HtmlAutocompleteMode::Billing)
        .with_contact_hint(HtmlAutocompleteContactHint::Mobile)
        .unwrap()
        .with_webauthn();
    assert_eq!(
        details.section().unwrap().value(),
        &DocumentString::from("SECTION-Family")
    );
    assert_eq!(details.mode(), Some(HtmlAutocompleteMode::Billing));
    assert_eq!(
        details.contact_hint(),
        Some(HtmlAutocompleteContactHint::Mobile)
    );
    assert!(details.has_webauthn());
    assert_eq!(
        details.into_attribute(),
        raw("SECTION-Family billing mobile tel webauthn")
    );
    assert_eq!(
        HtmlAutocompleteValue::Setting(HtmlFormAutocomplete::Off).into_attribute(),
        raw("off")
    );
    assert_eq!(
        HtmlAutocompleteValue::Details(HtmlAutocompleteDetails::new(HtmlAutofillField::Name))
            .into_attribute(),
        raw("name")
    );
    assert!(HtmlAutocompleteSection::new("section-").is_ok());
    assert_eq!(
        HtmlAutocompleteSection::new("blue"),
        Err(HtmlAutocompleteSectionError::MissingSectionPrefix)
    );
    assert!(matches!(
        HtmlAutocompleteSection::new("section-a b"),
        Err(HtmlAutocompleteSectionError::Token(_))
    ));
}

#[test]
fn processing_lowercases_only_section_and_matched_hints_preserving_field_case() {
    let report = textarea("  SECTION-Family\tSHIPPING\nHOME\rTeL\u{c}WEBAUTHN  ");
    assert!(matches!(
        report.syntax(),
        HtmlAutocompleteSyntaxState::Valid(_)
    ));
    let value = processed(&report);
    assert_eq!(value.field_name(), &DocumentString::from("WEBAUTHN"));
    assert_eq!(
        value.field_identity(),
        HtmlAutofillFieldIdentity::Credential(HtmlAutocompleteCredential::Webauthn)
    );
    assert_eq!(
        value.credential_type(),
        HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn)
    );
    assert_eq!(value.mode_hint(), Some(HtmlAutocompleteMode::Shipping));
    assert_eq!(
        value.contact_hint(),
        Some(HtmlAutocompleteContactHint::Home)
    );
    assert_eq!(
        value.scope().as_document_string(),
        DocumentString::from("section-family shipping home")
    );
    assert_eq!(
        value.idl_value(),
        &DocumentString::from("section-family shipping home TeL WEBAUTHN")
    );
    assert_eq!(
        report.tokens().unwrap().tokens()[0].value(),
        &DocumentString::from("SECTION-Family")
    );
    let report = textarea("BILLING FaMiLy-NaMe");
    assert_eq!(
        processed(&report).field_name(),
        &DocumentString::from("FaMiLy-NaMe")
    );
    assert_eq!(
        processed(&report).idl_value(),
        &DocumentString::from("billing FaMiLy-NaMe")
    );
}

#[test]
fn literal_pinned_webauthn_recovery_is_independent_of_native_author_grammar() {
    let report = textarea("webauthn");
    assert!(matches!(
        report.syntax(),
        HtmlAutocompleteSyntaxState::Invalid(_)
    ));
    let value = processed(&report);
    assert_eq!(value.field_name(), &DocumentString::from("webauthn"));
    assert_eq!(
        value.field_identity(),
        HtmlAutofillFieldIdentity::Credential(HtmlAutocompleteCredential::Webauthn)
    );
    assert_eq!(
        value.credential_type(),
        HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn)
    );
    assert_eq!(value.idl_value(), &DocumentString::from("webauthn"));
    assert!(value.scope().tokens().is_empty());
    for (text, field, idl) in [
        ("username webauthn", "webauthn", "username webauthn"),
        ("USERNAME WEBAUTHN", "WEBAUTHN", "USERNAME WEBAUTHN"),
    ] {
        let report = textarea(text);
        let HtmlAutocompleteSyntaxState::Valid(HtmlAutocompleteValue::Details(details)) =
            report.syntax()
        else {
            panic!("authored detail required")
        };
        assert_eq!(details.field(), HtmlAutofillField::Username);
        let value = processed(&report);
        assert_eq!(value.field_name(), &DocumentString::from(field));
        assert_eq!(value.idl_value(), &DocumentString::from(idl));
        assert_eq!(
            value.credential_type(),
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn)
        );
    }
    let report = textarea("home tel webauthn");
    assert_eq!(
        processed(&report).contact_hint(),
        Some(HtmlAutocompleteContactHint::Home)
    );
    assert_eq!(
        processed(&report).scope().as_document_string(),
        DocumentString::from("home")
    );
    assert_eq!(
        processed(&report).field_name(),
        &DocumentString::from("webauthn")
    );
    assert_eq!(
        processed(&report).idl_value(),
        &DocumentString::from("home tel webauthn")
    );
}

#[test]
fn invalid_credential_prefix_duplicates_and_extra_tokens_take_exact_default_branches() {
    for (text, reason) in [
        (
            "on webauthn",
            HtmlAutocompleteDefaultReason::InvalidCredentialPrefix,
        ),
        (
            "webauthn webauthn",
            HtmlAutocompleteDefaultReason::InvalidCredentialPrefix,
        ),
        (
            "username webauthn extra",
            HtmlAutocompleteDefaultReason::UnrecognizedFinalToken,
        ),
        (
            "name name",
            HtmlAutocompleteDefaultReason::UnrecognizedLeadingToken,
        ),
        (
            "shipping shipping name",
            HtmlAutocompleteDefaultReason::UnrecognizedLeadingToken,
        ),
        (
            "section-a section-b name",
            HtmlAutocompleteDefaultReason::ExtraLeadingTokens,
        ),
        (
            "extra off",
            HtmlAutocompleteDefaultReason::SettingWithExtraTokens,
        ),
        (
            "home name",
            HtmlAutocompleteDefaultReason::UnrecognizedLeadingToken,
        ),
    ] {
        let report = textarea(text);
        assert!(
            matches!(report.syntax(), HtmlAutocompleteSyntaxState::Invalid(_)),
            "{text}"
        );
        assert_default(
            &report,
            reason,
            HtmlAutocompleteDefaultField::FormOwnerRequired,
        );
    }
}

#[test]
fn settings_leave_credential_unassigned_unlike_successful_detail_processing() {
    let detail = textarea("name");
    for (text, canonical) in [("on", "on"), ("ON", "on"), ("off", "off"), ("oFf", "off")] {
        let setting = textarea(text);
        assert_eq!(
            processed(&setting).field_name(),
            &DocumentString::from(canonical)
        );
        assert_eq!(
            processed(&setting).idl_value(),
            &DocumentString::from(canonical)
        );
        assert_ne!(
            processed(&setting).credential_type(),
            processed(&detail).credential_type(),
            "setting early return does not assign the detail branch's null credential"
        );
    }
}

#[test]
fn each_successful_branch_reports_its_actual_credential_assignment() {
    for (text, expected) in [
        ("on", HtmlAutocompleteCredentialOutcome::NotAssigned),
        ("ON", HtmlAutocompleteCredentialOutcome::NotAssigned),
        ("off", HtmlAutocompleteCredentialOutcome::NotAssigned),
        ("oFf", HtmlAutocompleteCredentialOutcome::NotAssigned),
        ("name", HtmlAutocompleteCredentialOutcome::Null),
        ("NAME", HtmlAutocompleteCredentialOutcome::Null),
        (
            "username webauthn",
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn),
        ),
        (
            "USERNAME WEBAUTHN",
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn),
        ),
        (
            "webauthn",
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn),
        ),
    ] {
        let report = textarea(text);
        assert_eq!(processed(&report).credential_type(), expected, "{text}");
    }
}

#[test]
fn settings_are_single_tokens_and_hidden_mantle_does_not_author_gate_recovery() {
    for (text, expected) in [
        (" \tON\r", HtmlFormAutocomplete::On),
        ("oFf", HtmlFormAutocomplete::Off),
    ] {
        let report = input(text, HtmlInputType::Text);
        assert_eq!(report.mantle(), Some(HtmlAutocompleteMantle::Expectation));
        assert_eq!(
            report.syntax(),
            &HtmlAutocompleteSyntaxState::Valid(HtmlAutocompleteValue::Setting(expected))
        );
        assert_eq!(
            processed(&report).field_identity(),
            HtmlAutofillFieldIdentity::Setting(expected)
        );
        assert_eq!(
            processed(&report).field_name(),
            &DocumentString::from(if expected == HtmlFormAutocomplete::On {
                "on"
            } else {
                "off"
            })
        );
        let report = input(text, HtmlInputType::Hidden);
        assert_eq!(report.mantle(), Some(HtmlAutocompleteMantle::Anchor));
        let HtmlAutocompleteContextAuthorState::Nonconforming(violations) = report.context_author()
        else {
            panic!("anchor setting prohibited")
        };
        assert_eq!(violations.anchor_setting(), Some(expected));
        assert_default(
            &report,
            HtmlAutocompleteDefaultReason::SettingOnAnchor,
            HtmlAutocompleteDefaultField::Empty,
        );
    }
    let report = input("webauthn", HtmlInputType::Hidden);
    assert!(matches!(
        report.syntax(),
        HtmlAutocompleteSyntaxState::Invalid(_)
    ));
    assert_eq!(
        processed(&report).field_name(),
        &DocumentString::from("webauthn")
    );
}

#[test]
fn missing_empty_and_unknown_defaults_keep_source_null_owner_unresolved() {
    let interpreter = HtmlValueInterpreter::new(64);
    for (state, field) in [
        (HtmlInputType::Hidden, HtmlAutocompleteDefaultField::Empty),
        (
            HtmlInputType::Text,
            HtmlAutocompleteDefaultField::FormOwnerRequired,
        ),
    ] {
        let report = interpreter
            .control_autocomplete_for_input(&KnownHtmlElement::Input.qualified_name(), None, state)
            .unwrap();
        assert_eq!(report.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(report.syntax(), &HtmlAutocompleteSyntaxState::Missing);
        assert_eq!(report.tokens(), None);
        assert_eq!(
            report.context_author(),
            HtmlAutocompleteContextAuthorState::NotRequired
        );
        assert_default(
            &report,
            HtmlAutocompleteDefaultReason::MissingAttribute,
            field,
        );
        for text in ["", " \t\u{c}\n\r"] {
            let report = input(text, state);
            assert_eq!(
                report.syntax(),
                &HtmlAutocompleteSyntaxState::Invalid(HtmlAutocompleteSyntaxError::EmptyTokens)
            );
            assert_default(&report, HtmlAutocompleteDefaultReason::EmptyTokens, field);
        }
        assert_default(
            &input("unknown", state),
            HtmlAutocompleteDefaultReason::UnrecognizedFinalToken,
            field,
        );
    }
}

#[test]
fn each_input_state_selects_real_processing_domain_and_mantle() {
    let states = [
        (HtmlInputType::Hidden, true),
        (HtmlInputType::Text, true),
        (HtmlInputType::Search, true),
        (HtmlInputType::Telephone, true),
        (HtmlInputType::Url, true),
        (HtmlInputType::Email, true),
        (HtmlInputType::Password, true),
        (HtmlInputType::Date, true),
        (HtmlInputType::Month, true),
        (HtmlInputType::Week, true),
        (HtmlInputType::Time, true),
        (HtmlInputType::LocalDateTime, true),
        (HtmlInputType::Number, true),
        (HtmlInputType::Range, true),
        (HtmlInputType::Color, true),
        (HtmlInputType::Checkbox, false),
        (HtmlInputType::Radio, false),
        (HtmlInputType::File, false),
        (HtmlInputType::Submit, false),
        (HtmlInputType::Image, false),
        (HtmlInputType::Reset, false),
        (HtmlInputType::Button, false),
    ];
    for (state, applies) in states {
        let report = input("name", state);
        assert!(matches!(
            report.syntax(),
            HtmlAutocompleteSyntaxState::Valid(_)
        ));
        assert_eq!(report.input_state(), Some(state));
        assert!(report.context_source_clause().is_some());
        assert_eq!(
            report.mantle(),
            Some(if state == HtmlInputType::Hidden {
                HtmlAutocompleteMantle::Anchor
            } else {
                HtmlAutocompleteMantle::Expectation
            })
        );
        if applies {
            assert_eq!(
                processed(&report).field_name(),
                &DocumentString::from("name")
            );
        } else {
            assert_eq!(
                report.processing(),
                &HtmlAutocompleteProcessingState::InapplicableInputState(state)
            );
            let HtmlAutocompleteContextAuthorState::Nonconforming(v) = report.context_author()
            else {
                panic!("input state violation required")
            };
            assert_eq!(v.prohibited_input_state(), Some(state));
            assert_eq!(v.inappropriate_field(), Some(HtmlAutofillField::Name));
        }
    }
}

#[test]
fn field_control_groups_restrict_authorship_without_restricting_parsed_processing() {
    for (text, allowed_state, disallowed_state, field) in [
        (
            "name",
            HtmlInputType::Text,
            HtmlInputType::Number,
            HtmlAutofillField::Name,
        ),
        (
            "street-address",
            HtmlInputType::Hidden,
            HtmlInputType::Text,
            HtmlAutofillField::StreetAddress,
        ),
        (
            "current-password",
            HtmlInputType::Password,
            HtmlInputType::Url,
            HtmlAutofillField::CurrentPassword,
        ),
        (
            "url",
            HtmlInputType::Url,
            HtmlInputType::Password,
            HtmlAutofillField::Url,
        ),
        (
            "username",
            HtmlInputType::Email,
            HtmlInputType::Telephone,
            HtmlAutofillField::Username,
        ),
        (
            "tel",
            HtmlInputType::Telephone,
            HtmlInputType::Email,
            HtmlAutofillField::Tel,
        ),
        (
            "bday-month",
            HtmlInputType::Number,
            HtmlInputType::Month,
            HtmlAutofillField::BdayMonth,
        ),
        (
            "cc-exp",
            HtmlInputType::Month,
            HtmlInputType::Date,
            HtmlAutofillField::CcExp,
        ),
        (
            "bday",
            HtmlInputType::Date,
            HtmlInputType::Month,
            HtmlAutofillField::Bday,
        ),
    ] {
        assert_eq!(
            input(text, allowed_state).context_author(),
            HtmlAutocompleteContextAuthorState::Conforming
        );
        let report = input(text, disallowed_state);
        let HtmlAutocompleteContextAuthorState::Nonconforming(v) = report.context_author() else {
            panic!("field/control violation required")
        };
        assert_eq!(v.inappropriate_field(), Some(field));
        assert_eq!(
            processed(&report).field_identity(),
            HtmlAutofillFieldIdentity::Detail(field)
        );
        let select = HtmlValueInterpreter::new(64)
            .control_autocomplete(&KnownHtmlElement::Select.qualified_name(), Some(&raw(text)))
            .unwrap();
        assert_eq!(
            select.context_author(),
            HtmlAutocompleteContextAuthorState::Conforming
        );
    }
}

#[test]
fn select_webauthn_author_violation_does_not_remove_literal_credential_processing() {
    let attribute = raw("username webauthn");
    let report = HtmlValueInterpreter::new(64)
        .control_autocomplete(&KnownHtmlElement::Select.qualified_name(), Some(&attribute))
        .unwrap();
    let HtmlAutocompleteContextAuthorState::Nonconforming(v) = report.context_author() else {
        panic!("webauthn select violation required")
    };
    assert!(v.webauthn_on_select());
    assert_eq!(
        processed(&report).field_name(),
        &DocumentString::from("webauthn")
    );
    assert_eq!(
        processed(&report).credential_type(),
        HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn)
    );
}

#[test]
fn select_terminal_credential_violation_survives_invalid_detail_syntax() {
    for text in ["webauthn", "unknown webauthn"] {
        let report = HtmlValueInterpreter::new(64)
            .control_autocomplete(&KnownHtmlElement::Select.qualified_name(), Some(&raw(text)))
            .unwrap();
        assert!(matches!(
            report.syntax(),
            HtmlAutocompleteSyntaxState::Invalid(_)
        ));
        let HtmlAutocompleteContextAuthorState::Nonconforming(violations) = report.context_author()
        else {
            panic!(
                "terminal webauthn is source-forbidden on select independently of detail syntax"
            );
        };
        assert!(violations.webauthn_on_select());
    }
}

#[test]
fn missing_actual_input_context_preserves_syntax_and_never_guesses_defaults() {
    let interpreter = HtmlValueInterpreter::new(64);
    let element = KnownHtmlElement::Input.qualified_name();
    for value in [
        None,
        Some(raw("off")),
        Some(raw("name")),
        Some(raw("unknown")),
    ] {
        let report = interpreter
            .control_autocomplete(&element, value.as_ref())
            .unwrap();
        assert_eq!(
            report.processing(),
            &HtmlAutocompleteProcessingState::InputContextRequired
        );
        assert_eq!(report.mantle(), None);
        assert_eq!(
            report.context_author(),
            if value.is_none() {
                HtmlAutocompleteContextAuthorState::NotRequired
            } else {
                HtmlAutocompleteContextAuthorState::Unchecked
            }
        );
        assert_eq!(report.raw_attribute(), value.as_ref());
    }
}

#[test]
fn malformed_syntax_and_actual_state_prohibition_remain_independent() {
    let report = input("home name extra", HtmlInputType::Checkbox);
    assert!(matches!(
        report.syntax(),
        HtmlAutocompleteSyntaxState::Invalid(_)
    ));
    let HtmlAutocompleteContextAuthorState::Nonconforming(v) = report.context_author() else {
        panic!("state prohibition required")
    };
    assert_eq!(v.prohibited_input_state(), Some(HtmlInputType::Checkbox));
    assert_eq!(
        report.processing(),
        &HtmlAutocompleteProcessingState::InapplicableInputState(HtmlInputType::Checkbox)
    );
}

#[test]
fn ordered_grammar_rejects_repetition_wrong_order_and_non_ascii_separator() {
    for value in [
        "name name",
        "shipping billing name",
        "name shipping",
        "home name",
        "home home tel",
        "section-a section-b name",
        "section-a",
        "shipping",
        "home",
        "name\u{a0}webauthn",
        "name,username",
    ] {
        assert!(
            matches!(
                textarea(value).syntax(),
                HtmlAutocompleteSyntaxState::Invalid(_)
            ),
            "{value}"
        );
    }
    let report = textarea("section- name");
    assert!(matches!(
        report.syntax(),
        HtmlAutocompleteSyntaxState::Valid(_)
    ));
    assert_eq!(
        processed(&report).scope().as_document_string(),
        DocumentString::from("section-")
    );
}

#[test]
fn exact_raw_names_prefixes_and_unpaired_units_survive_only_selected_ascii_processing() {
    let element = QualifiedName::new(Namespace::html(), Some("p".into()), "textarea");
    let units = [
        "SECTION-".encode_utf16().collect::<Vec<_>>(),
        vec![0xd800],
        " NAME".encode_utf16().collect(),
    ]
    .concat();
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("q".into()), "autocomplete"),
        DocumentString::from_utf16(units.clone()),
    );
    let report = HtmlValueInterpreter::new(64)
        .control_autocomplete(&element, Some(&attribute))
        .unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.input_code_units(), units.len());
    assert!(matches!(
        report.syntax(),
        HtmlAutocompleteSyntaxState::Valid(_)
    ));
    let expected_section = ["section-".encode_utf16().collect::<Vec<_>>(), vec![0xd800]].concat();
    assert_eq!(
        processed(&report).scope().tokens()[0].value().code_units(),
        expected_section
    );
    assert_eq!(
        processed(&report).field_name(),
        &DocumentString::from("NAME")
    );
    assert_eq!(report.raw().unwrap().code_units(), units);
}

#[test]
fn exact_role_attribute_and_fact_family_rejections_retain_raw_without_processing() {
    let interpreter = HtmlValueInterpreter::new(64);
    let attribute = raw("name");
    for element in [
        KnownHtmlElement::Form.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "input"),
        QualifiedName::new(Namespace::html(), None, "INPUT"),
    ] {
        let report = interpreter
            .control_autocomplete(&element, Some(&attribute))
            .unwrap();
        assert_ne!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(
            report.processing(),
            &HtmlAutocompleteProcessingState::Unapplied
        );
        assert_eq!(report.syntax(), &HtmlAutocompleteSyntaxState::Unapplied);
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
    let textarea = KnownHtmlElement::Textarea.qualified_name();
    let report = interpreter
        .control_autocomplete_for_input(&textarea, Some(&attribute), HtmlInputType::Hidden)
        .unwrap();
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(report.input_state(), Some(HtmlInputType::Hidden));
    assert_eq!(
        report.processing(),
        &HtmlAutocompleteProcessingState::Unapplied
    );
    for name in [
        KnownHtmlAttribute::Title.qualified_name(),
        QualifiedName::new(Namespace::html(), None, "autocomplete"),
        QualifiedName::new(Namespace::none(), None, "AUTOCOMPLETE"),
    ] {
        let attribute = Attribute::new(name, "name");
        let report = interpreter
            .control_autocomplete(&textarea, Some(&attribute))
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
}

#[test]
fn raw_admission_precedes_role_and_state_checks_while_zero_admits_empty() {
    let interpreter = HtmlValueInterpreter::new(0);
    let element = KnownHtmlElement::Textarea.qualified_name();
    assert!(interpreter.control_autocomplete(&element, None).is_ok());
    assert!(
        interpreter
            .control_autocomplete(&element, Some(&raw("")))
            .is_ok()
    );
    let attribute = Attribute::new(
        KnownHtmlAttribute::Title.qualified_name(),
        DocumentString::from_utf16(vec![0xd800]),
    );
    let error = interpreter
        .control_autocomplete_for_input(&element, Some(&attribute), HtmlInputType::Checkbox)
        .unwrap_err();
    assert_eq!(error.required(), 1);
    assert_eq!(error.capacity(), 0);
}

#[test]
fn report_sources_select_the_control_clause_and_actual_input_state_without_new_inventory() {
    let report = input("username", HtmlInputType::Password);
    assert_eq!(report.attribute(), KnownHtmlAttribute::Autocomplete);
    assert_eq!(report.inventory_rows(), &[14]);
    assert!(report.source_clause().ends_with("/#attr-fe-autocomplete"));
    assert!(
        report
            .processing_source_clause()
            .ends_with("/#autofill-processing-model")
    );
    assert!(
        report
            .context_source_clause()
            .unwrap()
            .ends_with("/#password-state-(type=password)")
    );
    assert!(
        HtmlAutofillField::StreetAddress
            .source_clause()
            .ends_with("/#attr-fe-autocomplete-street-address")
    );
}
