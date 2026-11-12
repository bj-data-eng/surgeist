use surgeist_html::{
    elements::details::{
        HtmlDetailsAttribute as A, HtmlDetailsAttributeEffect as E, HtmlDetailsState as S,
        details_attribute_effect,
    },
    values::{DocumentString, Namespace},
};

#[test]
fn actual_open_presence_changes_have_distinct_source_effects() {
    let empty = DocumentString::from("");
    let value = DocumentString::from("false");
    assert_eq!(
        details_attribute_effect(&Namespace::none(), A::Open, None, Some(empty.code_units())),
        E::Notify {
            old: S::Closed,
            new: S::Open,
            close_other: true
        }
    );
    assert_eq!(
        details_attribute_effect(&Namespace::none(), A::Open, Some(value.code_units()), None),
        E::Notify {
            old: S::Open,
            new: S::Closed,
            close_other: false
        }
    );
    assert_eq!(
        details_attribute_effect(
            &Namespace::none(),
            A::Open,
            Some(empty.code_units()),
            Some(value.code_units())
        ),
        E::None
    );
    assert_eq!(
        details_attribute_effect(&Namespace::none(), A::Open, None, None),
        E::None
    );
}

#[test]
fn name_hook_is_independent_of_old_new_presence_and_equality() {
    let name = DocumentString::from("g");
    for (old, new) in [
        (None, None),
        (None, Some(name.code_units())),
        (Some(name.code_units()), None),
        (Some(name.code_units()), Some(name.code_units())),
    ] {
        assert_eq!(
            details_attribute_effect(&Namespace::none(), A::Name, old, new),
            E::CloseGiven
        );
    }
}

#[test]
fn namespaced_attributes_and_other_names_have_no_details_change_hook() {
    let value = DocumentString::from("g");
    assert_eq!(
        details_attribute_effect(&Namespace::html(), A::Open, None, Some(value.code_units())),
        E::None
    );
    assert_eq!(
        details_attribute_effect(&Namespace::none(), A::Other, None, Some(value.code_units())),
        E::None
    );
}
