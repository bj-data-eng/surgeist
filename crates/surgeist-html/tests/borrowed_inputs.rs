//! Exact language observations from an independently stored borrowed input.
use std::{cell::RefCell, rc::Rc, sync::Arc};
use surgeist_html::bounded::{HtmlElementNameRead, element_classification_with};
use surgeist_html::{
    DocumentString, HtmlAttributeInput, HtmlAttributesInput, HtmlElementClass, HtmlNameInput,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, is_html_integration_point,
};

type Reads = Rc<RefCell<Vec<&'static str>>>;

struct Name {
    namespace: Option<DocumentString>,
    prefix: Option<DocumentString>,
    local: DocumentString,
    reads: Reads,
    forbid_prefix: bool,
}

impl Name {
    fn new(namespace: Option<&str>, prefix: Option<&str>, local: &str) -> Self {
        Self {
            namespace: namespace.map(Into::into),
            prefix: prefix.map(Into::into),
            local: local.into(),
            reads: Reads::default(),
            forbid_prefix: false,
        }
    }
}

impl HtmlNameInput for Name {
    fn namespace_uri(&self) -> Option<&DocumentString> {
        self.reads.borrow_mut().push("namespace");
        self.namespace.as_ref()
    }
    fn prefix(&self) -> Option<&DocumentString> {
        assert!(!self.forbid_prefix, "rule inspected unused prefix");
        self.reads.borrow_mut().push("prefix");
        self.prefix.as_ref()
    }
    fn local_name(&self) -> &DocumentString {
        self.reads.borrow_mut().push("local");
        &self.local
    }
}

struct Attr {
    name: Name,
    value: DocumentString,
    value_reads: RefCell<usize>,
}

impl Attr {
    fn new(local: &str, value: impl Into<DocumentString>) -> Self {
        Self {
            name: Name::new(None, None, local),
            value: value.into(),
            value_reads: RefCell::new(0),
        }
    }
}

impl HtmlAttributeInput for Attr {
    type Name = Name;
    fn name(&self) -> &Name {
        &self.name
    }
    fn value(&self) -> &DocumentString {
        *self.value_reads.borrow_mut() += 1;
        &self.value
    }
}

const HTML: &str = "http://www.w3.org/1999/xhtml";

#[test]
fn refused_name_headers_precede_getters_and_classification_never_reads_prefix() {
    let mut name = Name::new(Some(HTML), Some("unused"), "input");
    name.forbid_prefix = true;
    let reads = name.reads.clone();
    let refused = element_classification_with(&name, |read| {
        if read == HtmlElementNameRead::NamespaceHeader {
            return Err("namespace refused");
        }
        Ok(())
    });
    assert_eq!(refused, Err("namespace refused"));
    assert!(reads.borrow().is_empty());
    let refused = element_classification_with(&name, |read| {
        if read == HtmlElementNameRead::LocalNameHeader {
            return Err("local refused");
        }
        Ok(())
    });
    assert_eq!(refused, Err("local refused"));
    assert_eq!(&*reads.borrow(), &["namespace"]);
    reads.borrow_mut().clear();
    let class = element_classification_with(&name, |read| {
        if read == HtmlElementNameRead::NamespaceHeader {
            reads.borrow_mut().push("namespace admitted");
        } else if read == HtmlElementNameRead::LocalNameHeader {
            reads.borrow_mut().push("local admitted");
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    assert_eq!(
        class,
        HtmlElementClass::FormAssociated(surgeist_html::HtmlNativeFormAssociationKind::Input)
    );
    let actual = reads.borrow();
    for (index, value) in actual.iter().enumerate() {
        if *value == "namespace" {
            assert_eq!(actual[index - 1], "namespace admitted");
        } else if *value == "local" {
            assert_eq!(actual[index - 1], "local admitted");
        }
    }
}

#[test]
fn borrowed_vocabulary_preserves_exact_components_without_prefix_observation() {
    let mut name = Name::new(Some(HTML), Some("not-an-identity"), "input");
    name.forbid_prefix = true;
    assert_eq!(
        KnownHtmlElement::from_name(&name),
        Some(KnownHtmlElement::Input)
    );
    name.local = "INPUT".into();
    assert_eq!(KnownHtmlElement::from_name(&name), None);
    name.local = DocumentString::from_utf16(vec![0xd800]);
    assert_eq!(KnownHtmlElement::from_name(&name), None);
    name.namespace = None;
    name.local = "id".into();
    assert_eq!(
        KnownHtmlAttribute::from_name(&name),
        Some(KnownHtmlAttribute::Id)
    );
}

#[test]
fn raw_limit_refusal_precedes_name_applicability_or_report_capture() {
    let element = Name::new(Some("foreign"), None, "foreign");
    let raw = Attr::new("not-id", "ab");
    assert!(
        HtmlValueInterpreter::new(1)
            .id_attribute(&element, Some(&raw))
            .is_err()
    );
    assert!(element.reads.borrow().is_empty());
    assert!(raw.name.reads.borrow().is_empty());
    assert_eq!(*raw.value_reads.borrow(), 1);
}

#[test]
fn retained_individual_evidence_preserves_raw_units_prefix_and_backing() {
    let element = Name::new(Some(HTML), Some(""), "div");
    let mut raw = Attr::new("id", DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]));
    raw.name.prefix = Some(DocumentString::from(""));
    let report = HtmlValueInterpreter::new(3)
        .id_attribute(&element, Some(&raw))
        .unwrap();
    let evidence = report.raw_attribute().unwrap();
    assert_eq!(evidence.value().code_units(), &[0xd800, 0, 0xdc00]);
    assert_eq!(evidence.name().prefix().unwrap().code_units(), &[]);
    assert!(Arc::ptr_eq(
        evidence.value().shared_code_units(),
        raw.value.shared_code_units()
    ));
    assert!(Arc::ptr_eq(
        report.element().local_name().shared_code_units(),
        element.local.shared_code_units()
    ));
    assert!(Arc::ptr_eq(
        report.element().prefix().unwrap().shared_code_units(),
        element.prefix.as_ref().unwrap().shared_code_units()
    ));
}

struct Bank {
    rows: Vec<Attr>,
    visited: RefCell<Vec<usize>>,
}

impl HtmlAttributesInput for Bank {
    type Attribute = Attr;
    fn len(&self) -> usize {
        self.rows.len()
    }
    fn get(&self, index: usize) -> Option<&Attr> {
        self.visited.borrow_mut().push(index);
        self.rows.get(index)
    }
}

#[test]
fn integration_point_reads_ordered_candidates_until_original_encoding_match() {
    let mut name = Name::new(
        Some("http://www.w3.org/1998/Math/MathML"),
        None,
        "annotation-xml",
    );
    name.forbid_prefix = true;
    let bank = Bank {
        rows: vec![
            Attr::new("other", "ignored"),
            Attr::new("encoding", "TEXT/HTML"),
            Attr::new("later", "ignored"),
        ],
        visited: RefCell::default(),
    };
    assert!(is_html_integration_point(&name, &bank));
    assert_eq!(&*bank.visited.borrow(), &[0, 1]);
    assert_eq!(*bank.rows[0].value_reads.borrow(), 1);
    assert_eq!(*bank.rows[2].value_reads.borrow(), 0);
    // The same borrowed slice is another real collection implementation.
    assert!(is_html_integration_point(&name, bank.rows.as_slice()));
}

#[test]
fn script_type_admits_original_language_even_when_type_would_override_it() {
    let element = Name::new(Some(HTML), None, "script");
    let rows = [
        Attr::new("type", "module"),
        Attr::new("language", "too-long"),
    ];
    let refused =
        HtmlValueInterpreter::new(7).script_type_with_attributes(&element, rows.as_slice());
    assert!(refused.is_err());
    assert!(element.reads.borrow().is_empty());
    let admitted = HtmlValueInterpreter::new(8)
        .script_type_with_attributes(&element, rows.as_slice())
        .unwrap();
    assert_eq!(
        admitted.language_attribute().unwrap().value().code_units(),
        &"too-long".encode_utf16().collect::<Vec<_>>()
    );
    assert!(Arc::ptr_eq(
        admitted
            .language_attribute()
            .unwrap()
            .value()
            .shared_code_units(),
        rows[1].value.shared_code_units()
    ));
}

#[test]
fn controlled_media_retains_borrowed_operands_and_refuses_before_namespace_read() {
    use surgeist_html::bounded::{
        HtmlMediaActionError, HtmlMediaField, HtmlMediaInterpretation, match_media_field,
    };
    let element = Name::new(Some(HTML), None, "style");
    let raw = Attr::new("media", "screen");
    assert!(matches!(
        match_media_field(&raw, HtmlMediaField::Media, |_| Err("stop")),
        Err("stop")
    ));
    assert!(raw.name.reads.borrow().is_empty());
    let matched = match_media_field(&raw, HtmlMediaField::Media, |_| Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    let mut owner = HtmlMediaInterpretation::from_matched(&element, Some(matched));
    owner
        .admit_media(|candidate| {
            assert!(std::ptr::eq(candidate.unwrap(), &raw));
            Ok::<_, ()>(())
        })
        .unwrap();
    assert!(matches!(
        owner.inspect_namespace(|_| Err("stop")),
        Err(HtmlMediaActionError::Admission("stop"))
    ));
    assert!(element.reads.borrow().is_empty());
    let recovered = owner.abort();
    assert!(std::ptr::eq(recovered.element(), &element));
    assert!(std::ptr::eq(recovered.raw().unwrap(), &raw));
}

#[test]
fn media_role_refusal_precedes_both_name_header_getters() {
    use surgeist_html::bounded::{
        HtmlMediaActionError, HtmlMediaField, HtmlMediaInterpretation, match_media_field,
    };
    let element = Name::new(Some(HTML), None, "style");
    let raw = Attr::new("media", "screen");
    let matched = match_media_field(&raw, HtmlMediaField::Media, |_| Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    let mut owner = HtmlMediaInterpretation::from_matched(&element, Some(matched));
    owner.admit_media(|_| Ok::<_, ()>(())).unwrap();
    owner.inspect_namespace(|_| Ok::<_, ()>(())).unwrap();
    element.reads.borrow_mut().clear();
    assert!(matches!(
        owner.derive_role(|_| Err("role refused")),
        Err(HtmlMediaActionError::Admission("role refused"))
    ));
    assert!(
        element.reads.borrow().is_empty(),
        "unadmitted reads: {:?}",
        element.reads.borrow()
    );
}

#[test]
fn media_role_prepays_local_header_and_reuses_the_admitted_namespace_length() {
    use surgeist_html::bounded::{HtmlMediaField, HtmlMediaInterpretation, match_media_field};
    let element = Name::new(Some(HTML), None, "style");
    let raw = Attr::new("media", "screen");
    let matched = match_media_field(&raw, HtmlMediaField::Media, |_| Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    let mut owner = HtmlMediaInterpretation::from_matched(&element, Some(matched));
    owner.admit_media(|_| Ok::<_, ()>(())).unwrap();
    owner.inspect_namespace(|_| Ok::<_, ()>(())).unwrap();
    element.reads.borrow_mut().clear();
    let mut charges = Vec::new();
    owner
        .derive_role(|work| {
            if charges.is_empty() {
                assert!(element.reads.borrow().is_empty());
            } else {
                assert_eq!(&*element.reads.borrow(), &["local"]);
            }
            charges.push(work);
            Ok::<_, ()>(())
        })
        .unwrap();
    // Existing envelope: 28 namespace units plus six per candidate for the
    // 113 modern element spellings. Splitting its header does not raise it.
    assert_eq!(charges, [1, 705]);
    assert_eq!(
        owner.role().unwrap(),
        Some(surgeist_html::HtmlMediaElementRole::Style)
    );
}

#[test]
fn document_body_roles_borrow_names_and_admit_before_component_getters() {
    use surgeist_html::{HtmlDocumentBodyRole as Role, document_body_role_with};
    for (namespace, local, expected) in [
        (Some(HTML), "html", Role::Html),
        (Some(HTML), "body", Role::Body),
        (Some(HTML), "frameset", Role::Body),
        (Some(HTML), "BODY", Role::Other),
        (Some("http://www.w3.org/2000/svg"), "body", Role::Other),
        (None, "html", Role::Other),
    ] {
        let mut name = Name::new(namespace, Some("unused"), local);
        name.forbid_prefix = true;
        assert_eq!(
            document_body_role_with(&name, |_| Ok::<_, ()>(())).unwrap(),
            expected
        );
        if namespace != Some(HTML) {
            assert!(!name.reads.borrow().contains(&"local"));
        }
    }
    for refused in [
        HtmlElementNameRead::NamespaceHeader,
        HtmlElementNameRead::LocalNameHeader,
    ] {
        let name = Name::new(Some(HTML), None, "body");
        assert_eq!(
            document_body_role_with(&name, |read| {
                if read == refused {
                    Err("header refused")
                } else {
                    Ok(())
                }
            }),
            Err("header refused")
        );
        let expected: &[&str] = if refused == HtmlElementNameRead::NamespaceHeader {
            &[]
        } else {
            &["namespace"]
        };
        assert_eq!(name.reads.borrow().as_slice(), expected);
    }
}

#[test]
fn meter_admits_all_six_original_borrowed_slots_before_element_applicability() {
    use surgeist_html::HtmlMeterAttributeInput;
    let element = Name::new(Some("foreign"), None, "meter");
    let raw = Attr::new("optimum", "1234");
    let input = HtmlMeterAttributeInput {
        value: None,
        minimum: None,
        maximum: None,
        low: None,
        high: None,
        optimum: Some(&raw),
    };
    assert!(
        HtmlValueInterpreter::new(3)
            .meter_attributes(&element, input)
            .is_err()
    );
    assert!(element.reads.borrow().is_empty());
    assert!(raw.name.reads.borrow().is_empty());
    assert_eq!(*raw.value_reads.borrow(), 1);
}
