//! Actual media operands, phased observations and guarded report publication.
#![forbid(unsafe_code)]
use crate::html::support::unqualified_text_attribute as attr;
use surgeist_html::{bounded::*, *};
type Error = HtmlMediaInterpretationError;
fn matched<'a>(a: &'a Attribute, field: HtmlMediaField) -> HtmlMediaFieldMatch<'a, Attribute> {
    match_media_field(a, field, |_| Ok::<_, ()>(()))
        .unwrap()
        .unwrap()
}
fn no_error(_: usize) -> Result<(), &'static str> {
    Ok(())
}
fn admitted(owner: &mut HtmlMediaInterpretation<'_, QualifiedName, Attribute>) {
    owner.admit_media(|_| Ok::<_, &'static str>(())).unwrap();
    owner.inspect_namespace(no_error).unwrap();
    owner.derive_role(no_error).unwrap();
}
fn value(owner: &mut HtmlMediaInterpretation<'_, QualifiedName, Attribute>) -> Vec<usize> {
    owner.begin_value().unwrap();
    let mut work = Vec::new();
    while owner.value_action() != Ok(HtmlMediaCursorAction::Complete) {
        owner
            .advance_value(|n| {
                work.push(n);
                Ok::<_, &'static str>(())
            })
            .unwrap();
    }
    work
}
#[test]
fn field_match_is_bound_to_actual_candidate_and_admits_only_reached_units() {
    let correct = attr("media", "x");
    let wrong = attr("medib", "x");
    let upper = attr("Media", "x");
    let foreign = Attribute::new(QualifiedName::new(Namespace::html(), None, "media"), "x");
    for (candidate, expected, visits) in [
        (&correct, true, 6),
        (&wrong, false, 6),
        (&upper, false, 2),
        (&foreign, false, 1),
    ] {
        let mut work = Vec::new();
        let result = match_media_field(candidate, HtmlMediaField::Media, |n| {
            work.push(n);
            Ok::<_, &str>(())
        })
        .unwrap();
        assert_eq!(result.is_some(), expected);
        assert_eq!(work, vec![1; visits]);
        if let Some(result) = result {
            assert!(std::ptr::eq(result.attribute(), candidate));
            assert_eq!(result.field(), HtmlMediaField::Media);
        }
    }
    let mut count = 0;
    let failure = match_media_field(&correct, HtmlMediaField::Media, |_| {
        count += 1;
        if count == 3 {
            Err("field cancelled")
        } else {
            Ok(())
        }
    });
    assert!(matches!(failure, Err("field cancelled")));
    assert_eq!(count, 3);
}
#[test]
fn value_completion_precedes_theme_scan_and_assembly_shares_each_original_attribute_once() {
    let element = KnownHtmlElement::Meta.qualified_name();
    let raw = attr("media", " ALL ");
    let name = attr("name", "THEME-color");
    let mut slot = Some(HtmlMediaInterpretation::from_matched(
        &element,
        Some(matched(&raw, HtmlMediaField::Media)),
    ));
    assert_eq!(
        HtmlMediaInterpretation::try_finish(&mut slot).unwrap_err(),
        Error::NotReady
    );
    let owner = slot.as_mut().unwrap();
    admitted(owner);
    owner
        .supply_matched_name(Some(matched(&name, HtmlMediaField::Name)))
        .unwrap();
    owner
        .admit_name(|n| {
            assert!(std::ptr::eq(n.unwrap(), &name));
            Ok::<_, &str>(())
        })
        .unwrap();
    assert_eq!(value(owner), [1, 1, 1, 1, 1, 4]);
    assert_eq!(
        owner.value().unwrap().class(),
        HtmlMediaValueClass::Nontrivial
    );
    assert!(!owner.value().unwrap().restrictive_picture_value());
    assert_eq!(owner.theme_color(), Err(Error::NotReady));
    let mut assembled = false;
    assert_eq!(
        owner.assemble(|| {
            assembled = true;
            Ok::<_, &str>(())
        }),
        Err(HtmlMediaActionError::Protocol(Error::NotReady))
    );
    assert!(!assembled);
    let mut work = Vec::new();
    owner
        .begin_theme_color(|n| {
            work.push(n);
            Ok::<_, &str>(())
        })
        .unwrap();
    while owner.theme_color_complete() != Ok(true) {
        owner
            .advance_theme_color(|n| {
                work.push(n);
                Ok::<_, &str>(())
            })
            .unwrap();
    }
    assert_eq!(work, [1; 12]);
    assert_eq!(owner.theme_color(), Ok(true));
    let raw_count = std::sync::Arc::strong_count(raw.value().shared_code_units());
    let name_count = std::sync::Arc::strong_count(name.value().shared_code_units());
    owner
        .assemble(|| {
            assert_eq!(
                std::sync::Arc::strong_count(raw.value().shared_code_units()),
                raw_count
            );
            assert_eq!(
                std::sync::Arc::strong_count(name.value().shared_code_units()),
                name_count
            );
            Ok::<_, &str>(())
        })
        .unwrap();
    assert_eq!(
        std::sync::Arc::strong_count(raw.value().shared_code_units()),
        raw_count + 1
    );
    assert_eq!(
        std::sync::Arc::strong_count(name.value().shared_code_units()),
        name_count + 1
    );
    let report = HtmlMediaInterpretation::try_finish(&mut slot).unwrap();
    assert_eq!(
        report.meta_placement(),
        HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::AllowedForThemeColor)
    );
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.name_attribute(), Some(&name));
}
#[test]
fn supplied_absence_and_not_selected_name_remain_distinct_and_wrong_phase_refuses() {
    let element = KnownHtmlElement::Meta.qualified_name();
    let raw = attr("media", "screen");
    let mut unchecked =
        HtmlMediaInterpretation::from_matched(&element, Some(matched(&raw, HtmlMediaField::Media)));
    admitted(&mut unchecked);
    value(&mut unchecked);
    assert_eq!(
        unchecked.begin_theme_color(no_error),
        Err(HtmlMediaActionError::Protocol(Error::NameNotSupplied))
    );
    unchecked.assemble(|| Ok::<_, &str>(())).unwrap();
    let report = HtmlMediaInterpretation::try_finish(&mut Some(unchecked)).unwrap();
    assert_eq!(
        report.meta_placement(),
        HtmlMediaPlacementObservation::Unchecked
    );
    let mut absent =
        HtmlMediaInterpretation::from_matched(&element, Some(matched(&raw, HtmlMediaField::Media)));
    admitted(&mut absent);
    absent.supply_matched_name(None).unwrap();
    assert_eq!(
        absent.supply_matched_name(None),
        Err(Error::NameAlreadySupplied)
    );
    absent
        .admit_name(|n| {
            assert!(n.is_none());
            Ok::<_, &str>(())
        })
        .unwrap();
    value(&mut absent);
    absent.begin_theme_color(no_error).unwrap();
    assert_eq!(absent.theme_color(), Ok(false));
    assert_eq!(
        absent.advance_theme_color(no_error),
        Err(HtmlMediaActionError::Protocol(Error::WrongPhase))
    );
    absent.assemble(|| Ok::<_, &str>(())).unwrap();
    assert_eq!(
        HtmlMediaInterpretation::try_finish(&mut Some(absent))
            .unwrap()
            .meta_placement(),
        HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::Forbidden)
    );
}
#[test]
fn absent_media_allows_ordinary_placement_but_explicit_semantic_theme_scan_still_runs() {
    let element = KnownHtmlElement::Meta.qualified_name();
    let name = attr("name", "theme-color");
    let mut owner = HtmlMediaInterpretation::from_matched(&element, None);
    admitted(&mut owner);
    owner
        .supply_matched_name(Some(matched(&name, HtmlMediaField::Name)))
        .unwrap();
    owner.admit_name(|_| Ok::<_, &str>(())).unwrap();
    assert_eq!(value(&mut owner), [4]);
    assert_eq!(owner.theme_color(), Err(Error::NotReady));
    let mut work = Vec::new();
    owner
        .begin_theme_color(|n| {
            work.push(n);
            Ok::<_, &str>(())
        })
        .unwrap();
    while owner.theme_color_complete() != Ok(true) {
        owner
            .advance_theme_color(|n| {
                work.push(n);
                Ok::<_, &str>(())
            })
            .unwrap();
    }
    assert_eq!(work, [1; 12]);
    owner.assemble(|| Ok::<_, &str>(())).unwrap();
    assert_eq!(
        HtmlMediaInterpretation::try_finish(&mut Some(owner))
            .unwrap()
            .meta_placement(),
        HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::NotSpecified)
    );
}
#[test]
fn ordinary_both_raw_caps_precede_foreign_namespace_and_non_meta_shortcuts() {
    let media = attr("media", "x");
    let name = attr("name", "long");
    for element in [
        QualifiedName::new(Namespace::svg(), None, "meta"),
        KnownHtmlElement::Div.qualified_name(),
    ] {
        let error = HtmlValueInterpreter::new(1)
            .media_for_meta(&element, Some(&media), Some(&name))
            .unwrap_err();
        assert_eq!((error.capacity(), error.required()), (1, 4));
    }
}
#[test]
fn rejected_field_proof_and_callback_failure_preserve_owner_operands_and_forbid_reports() {
    let element = KnownHtmlElement::Meta.qualified_name();
    let raw = attr("media", "x");
    let name = attr("name", "theme-color");
    let mut owner = Some(HtmlMediaInterpretation::from_matched(
        &element,
        Some(matched(&raw, HtmlMediaField::Media)),
    ));
    admitted(owner.as_mut().unwrap());
    assert_eq!(
        owner
            .as_mut()
            .unwrap()
            .supply_matched_name(Some(matched(&raw, HtmlMediaField::Media))),
        Err(Error::WrongField)
    );
    owner
        .as_mut()
        .unwrap()
        .supply_matched_name(Some(matched(&name, HtmlMediaField::Name)))
        .unwrap();
    assert_eq!(
        owner
            .as_mut()
            .unwrap()
            .admit_name(|_| Err("original raw refusal")),
        Err(HtmlMediaActionError::Admission("original raw refusal"))
    );
    assert_eq!(
        HtmlMediaInterpretation::try_finish(&mut owner).unwrap_err(),
        Error::Stopped
    );
    assert_eq!(owner.as_ref().unwrap().role(), Err(Error::Stopped));
    let inputs = owner.take().unwrap().abort();
    assert!(std::ptr::eq(inputs.element(), &element));
    assert!(std::ptr::eq(inputs.raw().unwrap(), &raw));
    assert!(
        matches!(inputs.name(),HtmlMediaNameSupply::Present(actual) if std::ptr::eq(*actual,&name))
    );
}
#[test]
fn final_assembly_refusal_and_explicit_between_action_stop_never_publish_local_report() {
    let element = KnownHtmlElement::Style.qualified_name();
    let raw = attr("media", "screen");
    let mut owner = Some(HtmlMediaInterpretation::from_matched(
        &element,
        Some(matched(&raw, HtmlMediaField::Media)),
    ));
    admitted(owner.as_mut().unwrap());
    value(owner.as_mut().unwrap());
    assert_eq!(
        owner.as_mut().unwrap().assemble(|| Err("write cancelled")),
        Err(HtmlMediaActionError::Admission("write cancelled"))
    );
    assert_eq!(
        HtmlMediaInterpretation::try_finish(&mut owner).unwrap_err(),
        Error::Stopped
    );
    assert_eq!(owner.as_ref().unwrap().value(), Err(Error::Stopped));
    let mut second = Some(HtmlMediaInterpretation::from_matched(&element, None));
    admitted(second.as_mut().unwrap());
    value(second.as_mut().unwrap());
    second.as_mut().unwrap().stop();
    assert_eq!(
        HtmlMediaInterpretation::try_finish(&mut second).unwrap_err(),
        Error::Stopped
    );
    assert!(matches!(
        second.take().unwrap().abort().name(),
        HtmlMediaNameSupply::NotSupplied
    ));
}

fn drive_with_checkpoint<'raw>(
    owner: &mut HtmlMediaInterpretation<'raw, QualifiedName, Attribute>,
    name: &'raw Attribute,
    mut tick: impl FnMut() -> Result<(), &'static str>,
) -> Result<(), HtmlMediaActionError<&'static str>> {
    owner.admit_media(|_| tick())?;
    owner.inspect_namespace(|_| tick())?;
    owner.derive_role(|_| tick())?;
    owner
        .supply_matched_name(Some(matched(name, HtmlMediaField::Name)))
        .unwrap();
    owner.admit_name(|_| tick())?;
    owner.begin_value().unwrap();
    while owner.value_action() != Ok(HtmlMediaCursorAction::Complete) {
        owner.advance_value(|_| tick())?;
    }
    owner.begin_theme_color(|_| tick())?;
    while owner.theme_color_complete() != Ok(true) {
        owner.advance_theme_color(|_| tick())?;
    }
    owner.assemble(tick)
}
#[test]
fn every_raw_namespace_role_value_theme_and_assembly_admission_refusal_is_terminal() {
    let element = KnownHtmlElement::Meta.qualified_name();
    let raw = attr("media", "all");
    let name = attr("name", "theme-color");
    // One raw admission, 29 namespace checkpoints (header + 28 URI units), local role header
    // plus its remaining envelope,
    // one name admission, three value units + Finish, 12 theme checkpoints,
    // and one final assembly admission.
    const CHECKPOINTS: usize = 50;
    for stop in 0..CHECKPOINTS {
        let mut slot = Some(HtmlMediaInterpretation::from_matched(
            &element,
            Some(matched(&raw, HtmlMediaField::Media)),
        ));
        let mut visited = 0;
        let result = drive_with_checkpoint(slot.as_mut().unwrap(), &name, || {
            let next = visited;
            visited += 1;
            if next == stop {
                Err("original checkpoint stop")
            } else {
                Ok(())
            }
        });
        assert_eq!(
            result,
            Err(HtmlMediaActionError::Admission("original checkpoint stop"))
        );
        assert_eq!(visited, stop + 1);
        assert_eq!(
            HtmlMediaInterpretation::try_finish(&mut slot).unwrap_err(),
            Error::Stopped
        );
        assert_eq!(slot.as_ref().unwrap().value(), Err(Error::Stopped));
        assert!(std::ptr::eq(
            slot.take().unwrap().abort().raw().unwrap(),
            &raw
        ));
    }
    let mut owner =
        HtmlMediaInterpretation::from_matched(&element, Some(matched(&raw, HtmlMediaField::Media)));
    let mut visited = 0;
    drive_with_checkpoint(&mut owner, &name, || {
        visited += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(visited, CHECKPOINTS);
}
