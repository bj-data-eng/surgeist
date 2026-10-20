//! HTML877d §4.10.2 native categories and §4.10.18.3/source hook triggers.
use surgeist_html::bounded::{
    HtmlFormOwnerResetDecision as D, HtmlFormOwnerResetTrigger as T, form_owner_reset_trigger,
    native_form_association_with,
};
use surgeist_html::{
    DocumentString, HtmlNativeFormAssociationKind as K, KnownHtmlElement as E, Namespace,
    QualifiedName,
};

#[test]
fn native_categories_and_listedness_follow_the_defining_category_lists() {
    let cases = [
        (E::Img, K::Image, false),
        (E::Button, K::Button, true),
        (E::Fieldset, K::Fieldset, true),
        (E::Object, K::Object, true),
        (E::Textarea, K::Textarea, true),
        (E::Output, K::Output, true),
        (E::Input, K::Input, true),
        (E::Select, K::Select, true),
    ];
    for (element, expected, listed) in cases {
        assert_eq!(K::from_element(element), Some(expected));
        assert_eq!(expected.element(), element);
        assert_eq!(expected.listed(), listed);
    }
    for &element in E::ALL {
        if !cases.iter().any(|&(expected, _, _)| element == expected) {
            assert_eq!(K::from_element(element), None);
        }
    }
}

fn classify(name: &QualifiedName) -> Option<K> {
    native_form_association_with(|expected| {
        // A real adapter admits its operand reads here. Exact UTF-16 and
        // expanded-name identity determine the result, independently of prefix.
        Ok::<_, std::convert::Infallible>(
            name.namespace().uri().is_some_and(|namespace| {
                namespace
                    .code_units()
                    .iter()
                    .copied()
                    .eq("http://www.w3.org/1999/xhtml".encode_utf16())
            }) && name
                .local_name()
                .code_units()
                .iter()
                .copied()
                .eq(expected.local_name().encode_utf16()),
        )
    })
    .unwrap()
}

#[test]
fn borrowed_exact_names_classify_all_native_kinds_and_ignore_prefix() {
    for (local, expected) in [
        ("img", K::Image),
        ("button", K::Button),
        ("fieldset", K::Fieldset),
        ("object", K::Object),
        ("textarea", K::Textarea),
        ("output", K::Output),
        ("input", K::Input),
        ("select", K::Select),
    ] {
        let name = QualifiedName::new(Namespace::html(), Some("p".into()), local);
        assert_eq!(classify(&name), Some(expected));
    }
}

#[test]
fn foreign_uppercase_unknown_custom_and_unpaired_names_do_not_become_native() {
    for name in [
        QualifiedName::new(Namespace::none(), None, "input"),
        QualifiedName::new(Namespace::svg(), None, "select"),
        QualifiedName::new(Namespace::html(), None, "INPUT"),
        QualifiedName::new(Namespace::html(), None, "form"),
        QualifiedName::new(Namespace::html(), None, "details"),
        QualifiedName::new(Namespace::html(), None, "x-control"),
        QualifiedName::new(Namespace::html(), None, ""),
        QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16(vec![0xd800]),
        ),
    ] {
        assert_eq!(classify(&name), None);
    }
}

#[test]
fn admitted_comparison_error_stops_before_later_operand_reads() {
    let mut admitted = Vec::new();
    let result = native_form_association_with(|element| {
        admitted.push(element);
        match element {
            E::Img => Ok(false),
            E::Button => Err("comparison refused"),
            _ => panic!("no operand may be read after refusal"),
        }
    });
    assert_eq!(result, Err("comparison refused"));
    assert_eq!(admitted, [E::Img, E::Button]);
}

#[test]
fn first_matching_category_avoids_later_operand_reads() {
    let result = native_form_association_with(|element| {
        assert_eq!(element, E::Img, "matching img needs no later reads");
        Ok::<_, std::convert::Infallible>(true)
    });
    assert_eq!(result, Ok(Some(K::Image)));
}

#[test]
fn insertion_skips_parser_inserted_controls_and_resets_other_associated_controls() {
    assert_eq!(
        form_owner_reset_trigger(T::Insertion {
            parser_inserted: true
        }),
        D::Skip
    );
    assert_eq!(
        form_owner_reset_trigger(T::Insertion {
            parser_inserted: false
        }),
        D::Reset
    );
}

#[test]
fn removing_or_moving_without_owner_needs_no_tree_probe() {
    assert_eq!(
        form_owner_reset_trigger(T::RemovingOrMoving { has_owner: false }),
        D::Skip
    );
    assert_eq!(
        form_owner_reset_trigger(T::RemovingOrMoving { has_owner: true }),
        D::ResetIfOwnerInDifferentTree
    );
}

#[test]
fn qualified_form_attribute_and_id_events_require_listed_recipients() {
    assert_eq!(
        form_owner_reset_trigger(T::FormAttributeChanged { listed: false }),
        D::Skip
    );
    assert_eq!(
        form_owner_reset_trigger(T::FormAttributeChanged { listed: true }),
        D::Reset
    );
    assert_eq!(
        form_owner_reset_trigger(T::IdEvent { listed: false }),
        D::Skip
    );
    assert_eq!(
        form_owner_reset_trigger(T::IdEvent { listed: true }),
        D::ResetIfFormAttributePresent
    );
}
