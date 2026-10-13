use surgeist_html::*;

fn name(ns: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(ns, None, local)
}
#[test]
fn nonce_expanded_identity_and_unrestricted_units() {
    let raw = DocumentString::from_utf16(vec![0, 32, 0xd800, 0xdc00, 0xffff]);
    for ns in [Namespace::html(), Namespace::svg(), Namespace::mathml()] {
        let element = name(ns, "unknown-element");
        let attribute = QualifiedName::new(Namespace::none(), Some("prefix".into()), "nonce");
        assert_eq!(
            nonce_attribute_change(&element, &attribute, Some(&raw)),
            HtmlNonceAttributeEffect::CopyValue(&raw)
        );
        assert_eq!(
            nonce_attribute_change(&element, &attribute, None),
            HtmlNonceAttributeEffect::ClearSlot
        );
        for attribute in [
            name(Namespace::none(), "Nonce"),
            name(Namespace::html(), "nonce"),
        ] {
            assert_eq!(
                nonce_attribute_change(&element, &attribute, Some(&raw)),
                HtmlNonceAttributeEffect::Ignore
            );
        }
    }
    assert_eq!(
        nonce_attribute_change(
            &name(Namespace::none(), "div"),
            &name(Namespace::none(), "nonce"),
            Some(&raw)
        ),
        HtmlNonceAttributeEffect::Ignore
    );
}
#[test]
fn nonce_insertion_conjunction_preserves_unknown_and_false_independently() {
    use HtmlHeaderDeliveredCsp as P;
    use HtmlNonceConnection as C;
    use HtmlNonceInsertionEffect as E;
    let element = name(Namespace::html(), "div");
    let nonempty = DocumentString::from(" A ");
    let empty = DocumentString::default();
    for content in [None, Some(&empty)] {
        assert_eq!(
            nonce_insertion_effect(&element, content, C::Unavailable, P::Unavailable),
            E::KeepContent
        );
    }
    assert_eq!(
        nonce_insertion_effect(
            &element,
            Some(&nonempty),
            C::NotBrowsingContextConnected,
            P::Unavailable
        ),
        E::KeepContent
    );
    assert_eq!(
        nonce_insertion_effect(&element, Some(&nonempty), C::Unavailable, P::Absent),
        E::KeepContent
    );
    assert_eq!(
        nonce_insertion_effect(&element, Some(&nonempty), C::Unavailable, P::Present),
        E::Unavailable(HtmlNonceMissing::Connection)
    );
    assert_eq!(
        nonce_insertion_effect(
            &element,
            Some(&nonempty),
            C::BrowsingContextConnected,
            P::Unavailable
        ),
        E::Unavailable(HtmlNonceMissing::HeaderDeliveredCsp)
    );
    assert_eq!(
        nonce_insertion_effect(
            &element,
            Some(&nonempty),
            C::BrowsingContextConnected,
            P::Present
        ),
        E::HideContentPreservingSlot
    );
}
fn builder(units: usize) -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 10, units, 10, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
#[test]
fn captured_slot_is_one_shot_independent_and_unit_capped() {
    let (mut b, d) = builder(1000);
    let e = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            OrderedAttributes::new(vec![Attribute::new(
                name(Namespace::none(), "nonce"),
                "content",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let raw = DocumentString::from_utf16(vec![0, 32, 0xd800, 0xffff]);
    b.establish_element_nonce(&e, raw.clone()).unwrap();
    assert_eq!(
        b.establish_element_nonce(&e, "different".into()),
        Err(HtmlCaptureError::NonceAlreadyEstablished)
    );
    let c = b.finish().unwrap();
    let node = c.node(&c.subject_id(&e).unwrap()).unwrap();
    let HtmlCapturedNodeKind::Element(e) = node.kind() else {
        panic!("element")
    };
    assert_eq!(e.cryptographic_nonce(), &HtmlCapturedNonce::Value(raw));
    assert_eq!(
        e.attributes().as_slice()[0].value(),
        &DocumentString::from("content")
    );
}
#[test]
fn captured_nonce_qualification_and_environment_establishment() {
    let (mut b, d) = builder(1000);
    let (mut other, od) = builder(1000);
    let foreign = other
        .add_element(
            &od,
            name(Namespace::html(), "p"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    assert_eq!(
        b.establish_element_nonce(&foreign, "x".into()),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    let wrong = b
        .add_element(
            &d,
            name(Namespace::none(), "p"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    assert_eq!(
        b.establish_element_nonce(&wrong, "x".into()),
        Err(HtmlCaptureError::NonceInapplicable)
    );
    let env = HtmlNonceDocumentEnvironment::new(
        HtmlBrowsingContextPresence::Absent,
        HtmlHeaderDeliveredCsp::Present,
    );
    b.establish_document_nonce_environment(&d, env).unwrap();
    assert_eq!(
        b.establish_document_nonce_environment(&d, env),
        Err(HtmlCaptureError::NonceAlreadyEstablished)
    );
    let c = b.finish().unwrap();
    let node = c.node(&c.subject_id(&d).unwrap()).unwrap();
    let HtmlCapturedNodeKind::Document(d) = node.kind() else {
        panic!("document")
    };
    assert_eq!(d.nonce_environment(), &env);
}
#[test]
fn captured_nonce_exact_and_one_short_unit_admission() {
    // HTML namespace (28 units) + local p (1) + slot AB (2).
    for cap in [30, 31] {
        let (mut b, d) = builder(cap);
        let e = b
            .add_element(
                &d,
                name(Namespace::html(), "p"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let result = b.establish_element_nonce(&e, "AB".into());
        if cap == 31 {
            result.unwrap();
        } else {
            assert_eq!(
                result,
                Err(HtmlCaptureError::LimitExceeded {
                    resource: HtmlCaptureResource::Utf16Units,
                    limit: 30,
                    required: 31
                })
            );
        }
        let c = b.finish().unwrap();
        let node = c.node(&c.subject_id(&e).unwrap()).unwrap();
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            panic!("element")
        };
        let expected = if cap == 31 {
            HtmlCapturedNonce::Value("AB".into())
        } else {
            HtmlCapturedNonce::Unavailable
        };
        assert_eq!(e.cryptographic_nonce(), &expected);
    }
}
