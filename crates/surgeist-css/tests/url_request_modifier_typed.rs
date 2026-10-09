#![forbid(unsafe_code)]
//! Functional new-API checks from Values 5 WD 2024-11-11 §4.1.1.
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#request-url-modifiers
//! The constructor and inspection API retain authored request syntax, without
//! applying Fetch operations or cryptographic interpretation.
use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueRef, CssComponentValues,
    CssIdent, CssImageValue, CssKnownPropertyValueRef, CssRequestUrlModifierRef as Request, CssUrl,
    CssUrlCrossOrigin as CrossOrigin, CssUrlFunction, CssUrlModifier, CssUrlModifierFunction,
    CssUrlReferrerPolicy as Policy, CssValueOrigin, CssValueTokenRef, parse_component_values,
    parse_style_attribute,
};

fn function(name: &str, text: &str) -> CssUrlModifierFunction {
    CssUrlModifierFunction::try_new(
        CssIdent::try_new(name).unwrap(),
        parse_component_values(text).unwrap(),
    )
    .unwrap()
}

fn parsed(text: &str) -> CssUrl {
    let report = parse_style_attribute(&format!("background-image:{text}"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundImage(images) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("background image")
    };
    let [CssImageValue::Url(url)] = images.images().images() else {
        panic!("URL")
    };
    url.clone()
}

#[test]
fn both_crossorigin_choices_are_typed_without_changing_authored_identity() {
    for (keyword, expected) in [
        ("anonymous", CrossOrigin::Anonymous),
        ("use-credentials", CrossOrigin::UseCredentials),
    ] {
        let arguments = format!(" /**/ {} /**/ ", keyword.to_ascii_uppercase());
        let constructed = function("CrOsSoRiGiN", &arguments);
        assert_eq!(constructed.name(), "CrOsSoRiGiN");
        assert_eq!(constructed.arguments().as_css(), arguments);
        assert_eq!(
            constructed.request_modifier(),
            Some(Request::CrossOrigin(expected))
        );
        let url = parsed(&format!("src(\"#asset\" CrOsSoRiGiN({arguments}))"));
        let [CssUrlModifier::Function(parsed)] = url.modifiers() else {
            panic!("function")
        };
        assert_eq!(parsed.request_modifier(), constructed.request_modifier());
        assert_eq!(
            parsed.argument_components().serialize().unwrap().as_css(),
            arguments
        );
        assert!(url.is_local_url());
        assert_eq!(url.function(), CssUrlFunction::Src);
    }
}

#[test]
fn all_eight_referrer_policy_keywords_have_exact_typed_identity() {
    for (keyword, expected) in [
        ("no-referrer", Policy::NoReferrer),
        (
            "no-referrer-when-downgrade",
            Policy::NoReferrerWhenDowngrade,
        ),
        ("same-origin", Policy::SameOrigin),
        ("origin", Policy::Origin),
        ("strict-origin", Policy::StrictOrigin),
        ("origin-when-cross-origin", Policy::OriginWhenCrossOrigin),
        (
            "strict-origin-when-cross-origin",
            Policy::StrictOriginWhenCrossOrigin,
        ),
        ("unsafe-url", Policy::UnsafeUrl),
    ] {
        let constructed = function("REFERRERPOLICY", &keyword.to_ascii_uppercase());
        assert_eq!(
            constructed.request_modifier(),
            Some(Request::ReferrerPolicy(expected))
        );
        let url = parsed(&format!("url(\"asset\" referrerpolicy({keyword}))"));
        let [CssUrlModifier::Function(parsed)] = url.modifiers() else {
            panic!("function")
        };
        assert_eq!(
            parsed.request_modifier(),
            Some(Request::ReferrerPolicy(expected))
        );
    }
}

#[test]
fn integrity_view_borrows_decoded_string_without_hash_validation_or_token_loss() {
    for (arguments, decoded) in [
        (r#""""#, ""),
        (r#""sha\32 56-not-a-hash""#, "sha256-not-a-hash"),
        (r#"' whitespace  '"#, " whitespace  "),
    ] {
        let constructed = function("INTEGRITY", arguments);
        assert_eq!(
            constructed.request_modifier(),
            Some(Request::Integrity(decoded))
        );
        assert_eq!(constructed.arguments().as_css(), arguments);
        let [component] = constructed.argument_components().items() else {
            panic!("one string")
        };
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::String(value)) if value == decoded)
        );
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("source")
        };
        assert_eq!(origin.source().as_str(), arguments);
        let url = CssUrl::from_parts(
            CssUrlFunction::Src,
            "asset",
            vec![CssUrlModifier::Function(constructed.clone())],
        );
        let reparse = parsed(&url.serialize_specified().unwrap());
        let [CssUrlModifier::Function(reparsed)] = reparse.modifiers() else {
            panic!("function")
        };
        assert_eq!(reparsed.request_modifier(), constructed.request_modifier());
    }
}

#[test]
fn checked_constructor_rejects_all_known_argument_arity_and_category_failures() {
    for (name, arguments) in [
        ("crossorigin", ""),
        ("crossorigin", "wrong"),
        ("crossorigin", "anonymous use-credentials"),
        ("crossorigin", "\"anonymous\""),
        ("crossorigin", "[anonymous]"),
        ("integrity", ""),
        ("integrity", "hash"),
        ("integrity", "1"),
        ("integrity", "\"one\" \"two\""),
        ("integrity", "f(\"hash\")"),
        ("referrerpolicy", ""),
        ("referrerpolicy", "default"),
        ("referrerpolicy", "\"origin\""),
        ("referrerpolicy", "origin, same-origin"),
        ("referrerpolicy", "{origin}"),
    ] {
        let error = CssUrlModifierFunction::try_new(
            CssIdent::try_new(name).unwrap(),
            parse_component_values(arguments).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            CssComponentValueErrorKind::InvalidFunction,
            "{name}({arguments})"
        );
    }
}

#[test]
fn checked_invalid_argument_errors_preserve_original_and_programmatic_origins() {
    let arguments = parse_component_values("\"hash\" extra").unwrap();
    let extra_origin = arguments.items()[2].origin().clone();
    let error = CssUrlModifierFunction::try_new(CssIdent::try_new("integrity").unwrap(), arguments)
        .unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidFunction);
    assert_eq!(error.origin(), &extra_origin);
    let error = CssUrlModifierFunction::try_new(
        CssIdent::try_new("crossorigin").unwrap(),
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("wrong").unwrap()]).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidFunction);
    assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
}

#[test]
fn unknown_names_and_bare_identifiers_do_not_become_recognized_requests() {
    for (name, arguments) in [
        ("crossorigin-extra", "wrong"),
        ("integrity-extra", "hash"),
        ("referrerpolicy-extra", "default"),
        ("future", "[x] f(1,\"s\")"),
        ("future", ""),
    ] {
        assert_eq!(function(name, arguments).request_modifier(), None);
    }
    let url = parsed(
        "src(\"asset\" crossorigin future() crossorigin(anonymous) integrity(\"one\") crossorigin(use-credentials) integrity(\"two\"))",
    );
    let requests: Vec<_> = url
        .modifiers()
        .iter()
        .filter_map(|modifier| match modifier {
            CssUrlModifier::Function(function) => function.request_modifier(),
            CssUrlModifier::Ident(_) => None,
            _ => panic!("future carrier variant"),
        })
        .collect();
    assert_eq!(
        requests,
        vec![
            Request::CrossOrigin(CrossOrigin::Anonymous),
            Request::Integrity("one"),
            Request::CrossOrigin(CrossOrigin::UseCredentials),
            Request::Integrity("two")
        ]
    );
}

#[test]
fn request_metadata_qualifies_authored_grammar_and_downstream_execution() {
    let record = surgeist_css::feature_metadata("required.value.request-url-modifier").unwrap();
    assert_eq!(record.status(), surgeist_css::CssSupportStatus::Partial);
    assert_eq!(record.source().id().as_str(), "I-VALUES5-REQUEST");
    assert_eq!(
        record.source().tier(),
        surgeist_css::CssSpecificationTier::Snapshot2026PreCrException
    );
    assert_eq!(
        record.source().url(),
        Some("https://www.w3.org/TR/2024/WD-css-values-5-20241111/")
    );
    assert!(
        record
            .supported_subset()
            .unwrap()
            .contains("eight referrerpolicy() keywords")
    );
    assert!(
        record
            .unsupported_remainder()
            .unwrap()
            .contains("Fetch request steps")
    );
    assert_eq!(
        function("referrerpolicy", "origin").request_modifier(),
        Some(Request::ReferrerPolicy(Policy::Origin))
    );
    assert!(
        CssUrlModifierFunction::try_new(
            CssIdent::try_new("referrerpolicy").unwrap(),
            parse_component_values("default").unwrap()
        )
        .is_err()
    );
}
