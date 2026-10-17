//! Independent literal insertion traces from HTML877d raw60684–60707/1479–1485.
use surgeist_html::{
    HtmlAutofocusInsertionAction as A, HtmlAutofocusInsertionCursor as C,
    HtmlAutofocusInsertionError as E, HtmlAutofocusInsertionReply as R,
};

const REPLIES: [R; 13] = [
    R::Autofocus(false),
    R::Autofocus(true),
    R::InDocumentTree(false),
    R::InDocumentTree(true),
    R::FullyActive(false),
    R::FullyActive(true),
    R::SandboxAutomaticFeatures(false),
    R::SandboxAutomaticFeatures(true),
    R::AllowFocus(false),
    R::AllowFocus(true),
    R::AutofocusProcessed(false),
    R::AutofocusProcessed(true),
    R::Applied,
];
fn step(c: &mut C, action: A, reply: R) {
    assert_eq!(c.action(), action);
    // Every other reply kind is wrong at this stage, regardless of its value.
    for wrong in REPLIES {
        if std::mem::discriminant(&wrong) != std::mem::discriminant(&reply) {
            let before = *c;
            assert_eq!(c.reply(wrong), Err(E::WrongReply));
            assert_eq!(*c, before);
        }
    }
    c.reply(reply).unwrap();
}
fn complete(c: &mut C) {
    assert_eq!(c.action(), A::Complete);
    for reply in REPLIES {
        let before = *c;
        assert_eq!(c.reply(reply), Err(E::Terminal));
        assert_eq!(*c, before);
    }
}
#[test]
fn absent_autofocus_or_ordinary_document_tree_membership_stops_before_document_capture() {
    let mut c = C::new();
    step(&mut c, A::ReadAutofocus, R::Autofocus(false));
    complete(&mut c);
    let mut c = C::new();
    step(&mut c, A::ReadAutofocus, R::Autofocus(true));
    step(&mut c, A::ReadInDocumentTree, R::InDocumentTree(false));
    complete(&mut c);
}
#[test]
fn inactive_target_sandboxed_automatic_features_or_denied_permission_stop_in_source_order() {
    for guard in 0..3 {
        let mut c = C::new();
        step(&mut c, A::ReadAutofocus, R::Autofocus(true));
        step(&mut c, A::ReadInDocumentTree, R::InDocumentTree(true));
        step(&mut c, A::CaptureTargetDocument, R::Applied);
        step(&mut c, A::ReadFullyActive, R::FullyActive(guard != 0));
        if guard == 0 {
            complete(&mut c);
            continue;
        }
        step(
            &mut c,
            A::ReadSandboxAutomaticFeatures,
            R::SandboxAutomaticFeatures(guard == 1),
        );
        if guard == 1 {
            complete(&mut c);
            continue;
        }
        step(&mut c, A::ReadAllowFocus, R::AllowFocus(false));
        complete(&mut c);
    }
}
fn admitted_target(c: &mut C) {
    step(c, A::ReadAutofocus, R::Autofocus(true));
    step(c, A::ReadInDocumentTree, R::InDocumentTree(true));
    step(c, A::CaptureTargetDocument, R::Applied);
    step(c, A::ReadFullyActive, R::FullyActive(true));
    step(
        c,
        A::ReadSandboxAutomaticFeatures,
        R::SandboxAutomaticFeatures(false),
    );
    step(c, A::ReadAllowFocus, R::AllowFocus(true));
    step(c, A::CaptureTargetTopDocument, R::Applied);
}
#[test]
fn already_processed_top_document_returns_without_touching_candidates() {
    let mut c = C::new();
    admitted_target(&mut c);
    step(
        &mut c,
        A::ReadAutofocusProcessed,
        R::AutofocusProcessed(true),
    );
    complete(&mut c);
}
#[test]
fn unprocessed_top_document_removes_then_appends_without_any_focusability_query() {
    let mut c = C::new();
    admitted_target(&mut c);
    step(
        &mut c,
        A::ReadAutofocusProcessed,
        R::AutofocusProcessed(false),
    );
    step(&mut c, A::RemoveCandidate, R::Applied);
    // Appending is a distinct required effect; removal acknowledgment cannot
    // certify that the actual original Element has been appended.
    assert_eq!(c.action(), A::AppendCandidate);
    step(&mut c, A::AppendCandidate, R::Applied);
    complete(&mut c);
}
#[test]
fn permission_suspension_preserves_target_capture_and_exact_pending_stage() {
    let mut c = C::new();
    step(&mut c, A::ReadAutofocus, R::Autofocus(true));
    step(&mut c, A::ReadInDocumentTree, R::InDocumentTree(true));
    step(&mut c, A::CaptureTargetDocument, R::Applied);
    step(&mut c, A::ReadFullyActive, R::FullyActive(true));
    step(
        &mut c,
        A::ReadSandboxAutomaticFeatures,
        R::SandboxAutomaticFeatures(false),
    );
    let waiting = c;
    assert_eq!(c.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(c, waiting);
    assert_eq!(c.action(), A::ReadAllowFocus);
    step(&mut c, A::ReadAllowFocus, R::AllowFocus(true));
    // The next capture is of target's actual top Document, not another target
    // node-Document capture or a repeat of already completed source guards.
    step(&mut c, A::CaptureTargetTopDocument, R::Applied);
    step(
        &mut c,
        A::ReadAutofocusProcessed,
        R::AutofocusProcessed(false),
    );
    step(&mut c, A::RemoveCandidate, R::Applied);
    step(&mut c, A::AppendCandidate, R::Applied);
    complete(&mut c);
}

#[test]
fn insertion_presence_uses_exact_expanded_name_and_ignores_boolean_value_spelling() {
    use surgeist_html::{Attribute, DocumentString, Namespace, QualifiedName, autofocus_presence};
    let element = QualifiedName::new(Namespace::html(), None, "dialog");
    for value in [
        DocumentString::from(""),
        DocumentString::from("autofocus"),
        DocumentString::from("false"),
        DocumentString::from(" arbitrary "),
        DocumentString::from_utf16(vec![0, 0xd800]),
    ] {
        let actual = Attribute::new(
            QualifiedName::new(Namespace::none(), None, "autofocus"),
            value,
        );
        assert!(autofocus_presence(&element, Some(&actual)));
    }
    // Expanded-name identity ignores a raw prefix, without inventing a namespace.
    let prefixed = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "autofocus"),
        "false",
    );
    assert!(autofocus_presence(&element, Some(&prefixed)));
    assert!(!autofocus_presence(&element, None));
    for local in [
        "",
        "Autofocus",
        "AUTOFOCUS",
        "autofocus ",
        " autofocus",
        "autofocusx",
    ] {
        let actual = Attribute::new(QualifiedName::new(Namespace::none(), None, local), "");
        assert!(!autofocus_presence(&element, Some(&actual)));
    }
    for namespace in [
        Namespace::html(),
        Namespace::svg(),
        Namespace::mathml(),
        Namespace::from_uri("urn:actual-attribute"),
    ] {
        let actual = Attribute::new(QualifiedName::new(namespace, None, "autofocus"), "");
        assert!(!autofocus_presence(&element, Some(&actual)));
    }
    let non_ascii_name = Attribute::new(
        QualifiedName::new(
            Namespace::none(),
            None,
            DocumentString::from_utf16(vec![97, 117, 116, 111, 102, 111, 99, 117, 0xd800]),
        ),
        "",
    );
    assert!(!autofocus_presence(&element, Some(&non_ascii_name)));
}

#[test]
fn insertion_presence_requires_exact_html_svg_or_mathml_namespace_identity() {
    use surgeist_html::{Attribute, DocumentString, Namespace, QualifiedName, autofocus_presence};
    let actual = Attribute::new(
        QualifiedName::new(Namespace::none(), None, "autofocus"),
        "false",
    );
    // Independently spelled by-value URI identities count just like the named
    // namespace constructors. Prefix and Element spelling are not qualifiers;
    // the helper has no Document-class premise to accidentally require HTML.
    for namespace in [
        Namespace::html(),
        Namespace::svg(),
        Namespace::mathml(),
        Namespace::from_uri("http://www.w3.org/1999/xhtml"),
        Namespace::from_uri("http://www.w3.org/2000/svg"),
        Namespace::from_uri("http://www.w3.org/1998/Math/MathML"),
    ] {
        for prefix in [None, Some(DocumentString::from("actual-prefix"))] {
            let element = QualifiedName::new(namespace.clone(), prefix, "actual-element");
            assert!(autofocus_presence(&element, Some(&actual)));
            assert!(!autofocus_presence(&element, None));
        }
    }
    for namespace in [
        Namespace::none(),
        Namespace::from_uri("urn:ordinary-element"),
        Namespace::from_uri("http://www.w3.org/1999/XHTML"),
        Namespace::from_uri("http://www.w3.org/2000/SVG"),
        Namespace::from_uri("http://www.w3.org/1998/math/MathML"),
        Namespace::from_uri("http://www.w3.org/1999/xhtml "),
        Namespace::from_uri("http://www.w3.org/2000/sv"),
        Namespace::from_uri("http://www.w3.org/1998/Math/MathMLx"),
        Namespace::from_uri("http://www.w3.org/1999/xhtmL"),
    ] {
        let element = QualifiedName::new(namespace, Some("html".into()), "dialog");
        assert!(!autofocus_presence(&element, Some(&actual)));
    }
    // A same-length URI containing an isolated surrogate must not be normalized
    // or decoded into a recognized namespace identity.
    let mut invalid_uri: Vec<u16> = "http://www.w3.org/1998/Math/MathML"
        .encode_utf16()
        .collect();
    invalid_uri[0] = 0xd800;
    let element = QualifiedName::new(
        Namespace::from_uri(DocumentString::from_utf16(invalid_uri)),
        None,
        "dialog",
    );
    assert!(!autofocus_presence(&element, Some(&actual)));
}
