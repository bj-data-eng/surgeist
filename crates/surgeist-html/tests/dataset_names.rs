//! Source-selected HTML dataset names retain exact UTF-16 and ASCII case rules.

use std::cell::Cell;

use surgeist_html::microsyntax::dataset::{HtmlDatasetAttributeName, HtmlDatasetPropertyName};

fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

#[test]
fn dataset_name_transform_preserves_every_other_code_unit() {
    for (attribute, property) in [
        ("data-", ""),
        ("data--a", "A"),
        ("data---a", "-A"),
        ("data-a-b--", "aB--"),
        ("data-a-ß-é:ξ", "a-ß-é:ξ"),
    ] {
        let attribute = units(attribute);
        let property = units(property);
        let read = HtmlDatasetPropertyName::from_attribute_with(&attribute, || Ok::<_, ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(read.len(), property.len());
        let mut copied = Vec::new();
        read.visit_with(
            || Ok::<_, ()>(()),
            |unit| {
                copied.push(unit);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(copied, property);

        let write = HtmlDatasetAttributeName::for_set_with(&property, || Ok::<_, ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(write.len(), attribute.len());
        assert!(
            write
                .matches_local_name_with(&attribute, || Ok::<_, ()>(()))
                .unwrap()
        );
        copied.clear();
        write
            .visit_with(
                || Ok::<_, ()>(()),
                |unit| {
                    copied.push(unit);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(copied, attribute);
    }

    let mut name = units("data-");
    name.extend([0xd800, 0x2d, 0xdc00]);
    let property = HtmlDatasetPropertyName::from_attribute_with(&name, || Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    let mut copied = Vec::new();
    property
        .visit_with(
            || Ok::<_, ()>(()),
            |unit| {
                copied.push(unit);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(copied, [0xd800, 0x2d, 0xdc00]);
    assert!(
        HtmlDatasetAttributeName::for_set_with(&copied, || Ok::<_, ()>(()))
            .unwrap()
            .unwrap()
            .matches_local_name_with(&name, || Ok::<_, ()>(()))
            .unwrap()
    );
}

#[test]
fn dataset_eligibility_set_syntax_and_delete_spelling_are_independent() {
    for attribute in ["Data-key", "data", "data-a-A", "notdata-x", "data-ÉZ"] {
        assert!(
            HtmlDatasetPropertyName::from_attribute_with(&units(attribute), || Ok::<_, ()>(()))
                .unwrap()
                .is_none()
        );
    }
    let bad = units("a-b:É");
    assert!(
        HtmlDatasetAttributeName::for_set_with(&bad, || Ok::<_, ()>(()))
            .unwrap()
            .is_none()
    );

    let delete = HtmlDatasetAttributeName::for_delete_with(&bad, || Ok::<_, ()>(())).unwrap();
    let mut output = Vec::new();
    delete
        .visit_with(
            || Ok::<_, ()>(()),
            |unit| {
                output.push(unit);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(output, units("data-a-b:É"));
}

#[test]
fn dataset_name_reads_and_writes_refuse_one_short_of_reached_work() {
    let attribute = units("data-real-name--end");
    let property = units("realName-End");
    let spent = Cell::new(0usize);
    let attempt = |budget| {
        spent.set(0);
        let step = || {
            if spent.get() == budget {
                return Err(());
            }
            spent.set(spent.get() + 1);
            Ok(())
        };
        let spelling = HtmlDatasetPropertyName::from_attribute_with(&attribute, step)?.unwrap();
        let mut output = Vec::new();
        spelling.visit_with(step, |unit| {
            output.push(unit);
            Ok(())
        })?;
        assert_eq!(output, property);
        let local = HtmlDatasetAttributeName::for_set_with(&output, step)?.unwrap();
        assert!(local.matches_local_name_with(&attribute, step)?);
        Ok(())
    };
    assert_eq!(attempt(usize::MAX), Ok(()));
    let exact = spent.get();
    assert_eq!(attempt(exact - 1), Err(()));
    assert_eq!(spent.get(), exact - 1);
    assert_eq!(attempt(exact), Ok(()));
}
