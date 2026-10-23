//! The crate-private controlled authored protocol, driven directly (#1286).
//!
//! These cases moved from the public integration suite when the protocol
//! became crate-private (#1353, 5.7); the ordinary front's cases stay there.

use super::super::protocol::CssColorSlotKind;
use super::*;
use crate::color::ColorProtocolError;
use crate::*;
use surgeist_color::Color;

fn color(source: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(source).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{source}: {error:?}"));
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color: {declaration:?}")
    };
    value.value().clone()
}
fn facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(41, 7),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    )
}
fn limits() -> CssAuthoredColorOrdinaryLimits {
    CssAuthoredColorOrdinaryLimits {
        algorithm_records: 100_000,
        arithmetic_records: 10_000,
        evaluation_records: 10_000,
        path_steps: 100,
        actions: 30_000_000,
    }
}

fn bank(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}
fn step<'path, 'css, 'authored: 'css, 'facts>(
    op: &mut CssAuthoredColorOperation<'path, 'css, 'authored, 'facts>,
    banks: &mut [Vec<CssColorSlot<'css>>; 3],
) -> Result<(), AuthoredError> {
    use CssAuthoredColorActionDescription as A;
    let mut action = op.next_action();
    let _ = action.layout()?;
    match action.description() {
        A::ReadAuthored { path, .. } => {
            let layout = action.path_inspection_layout(path)?;
            assert_eq!(
                layout.step_read_bytes,
                std::mem::size_of_val(action.path_view(path)?)
            );
            action.read_authored()
        }
        A::ReadSite { index } => action.read_slot(CssColorBank::Algorithm, index, &banks[0][index]),
        A::ReadSlot { bank: b, index } => action.read_slot(b, index, &banks[bank(b)][index]),
        A::AppendSlot { bank: b, index } => {
            let slots = &mut banks[bank(b)];
            assert_eq!(index, slots.len());
            slots.push(action.record()?);
            action.written(b, index, &slots[index])
        }
        A::UpdateSlot { bank: b, index } => {
            let slot = &mut banks[bank(b)][index];
            *slot = action.record()?;
            action.written(b, index, slot)
        }
        A::ReservePath { .. } => {
            let _ = action.reservation_layout()?;
            let receipt = action.reserve_path()?;
            assert_eq!(receipt.status(), CssColorPathReservationStatus::Reserved);
            action.capacity_observed()
        }
        A::AppendPath(_) => action.append_path(),
        A::TruncatePath(_) => action.truncate_path(),
        A::PrepareSite => action.prepare_site(),
        A::RestoreSite => action.restore_site(),
        A::Compute(_) | A::ReadFact(_) => action.compute(),
        _ => panic!("no pending work"),
    }
}

#[test]
fn controlled_site_less_failure_retains_its_actual_backing_until_explicit_recovery() {
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(41, 7),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Dark,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    let root = color("light-dark(red, currentcolor)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    let error = loop {
        if let Err(error) = step(op.as_mut().unwrap(), &mut banks) {
            break error;
        }
    };
    assert_eq!(
        error,
        AuthoredError::Color(CssColorError::from(
            crate::color::ColorErrorKind::Unavailable(CssColorUnavailable::CurrentColor)
        ))
    );
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert_eq!(
        CssAuthoredColorOperation::take(&mut op).err(),
        Some(ColorProtocolError::Terminal)
    );
    assert!(op.is_some());
    assert_eq!(
        CssAuthoredColorOperation::recover(&mut op).err(),
        Some(ColorProtocolError::Terminal)
    );
    assert!(op.is_some());
    op.as_mut().unwrap().abandon();
    op.as_mut().unwrap().abandon();
    let observation = CssAuthoredColorOperation::recover(&mut op).unwrap();
    assert!(observation.is_none());
    drop(op);
    drop(banks);
    assert_eq!(path.len(), 1);
    assert!(path.capacity() >= 1);
    let actual = path.capacity();
    path.cleanup().unwrap();
    assert_eq!(path.capacity(), actual);
    // The retained buffer and original owner can both be lent again after every
    // actual bank/checkpoint/handle borrowing that anchor has been retired.
    let other = CssColor::transparent();
    let mut reused = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &other,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    while reused.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        step(reused.as_mut().unwrap(), &mut banks).unwrap();
    }
    let result = CssAuthoredColorOperation::take(&mut reused).unwrap();
    assert!(
        matches!(result.outcome(),CssAuthoredColorOutcome::Used(value) if value.alpha()==Some(0.0))
    );
    drop(reused);
    drop(banks);
    drop(result);
    path.cleanup().unwrap();
    assert_eq!(path.capacity(), actual);
}

#[test]
fn an_infinite_branch_channel_resolves_to_the_largest_finite_value() {
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(41, 7),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Dark,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    let root = color("light-dark(red,lab(50 calc(infinity) 0))");
    let mut owner = CssColorOperationOwner::new();
    let mut buffer = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut buffer,
        )
        .unwrap(),
    );
    // Values 4 §10.9.1: the dark branch's infinite Lab axis clamps to the
    // largest finite value instead of escaping the top-level calculation.
    while op.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    let result = CssAuthoredColorOperation::take(&mut op).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("{:?}", result.outcome())
    };
    assert_eq!(value.space(), surgeist_color::ColorSpace::Lab);
    assert_eq!(value.channels(), [Some(50.0), Some(f64::MAX), Some(0.0)]);
    drop(result);
    drop(op);
    drop(banks);
    assert_eq!(buffer.len(), 0);
    buffer.cleanup().unwrap();
}

#[test]
fn dropped_reservation_receipts_belong_only_to_their_actual_buffers() {
    let facts = facts();
    let root = color("rgb(1 2 3)");
    let mut first = CssColorPathBuffer::new();
    let mut second = CssColorPathBuffer::new();
    let mut first_owner = CssColorOperationOwner::new();
    let mut second_owner = CssColorOperationOwner::new();
    for (owner, buffer) in [
        (&mut first_owner, &mut first),
        (&mut second_owner, &mut second),
    ] {
        let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
        let mut op = Some(
            CssAuthoredColorOperation::resolve_authored(
                owner,
                &root,
                &facts,
                facts.identity(),
                buffer,
            )
            .unwrap(),
        );
        while !matches!(
            op.as_ref().unwrap().action(),
            CssAuthoredColorActionDescription::ReservePath { .. }
        ) {
            step(op.as_mut().unwrap(), &mut banks).unwrap();
        }
        {
            let mut guard = op.as_mut().unwrap().next_action();
            let receipt = guard.reserve_path().unwrap();
            assert_eq!(receipt.actual_len(), 0);
            assert!(receipt.actual_capacity() >= 1);
            assert_eq!(
                guard.reserve_path().err(),
                Some(ColorProtocolError::WrongAction)
            );
        }
        assert_eq!(
            op.as_ref().unwrap().action(),
            CssAuthoredColorActionDescription::Stopped
        );
        let observation = CssAuthoredColorOperation::recover(&mut op).unwrap();
        assert!(observation.is_some());
        drop(op);
        drop(banks);
    }
    assert!(first.capacity_observation().is_some());
    assert!(second.capacity_observation().is_some());
    let retained = first.capacity();
    assert_eq!(first.cleanup(), Err(ColorProtocolError::WrongAction));
    first.capacity_observed().unwrap();
    assert_eq!(
        first.capacity_observed(),
        Err(ColorProtocolError::WrongAction)
    );
    assert!(
        second.capacity_observation().is_some(),
        "acknowledgement does not target another buffer with equal-looking metadata"
    );
    first.cleanup().unwrap();
    assert_eq!(first.capacity(), retained);
    second.capacity_observed().unwrap();
    second.cleanup().unwrap();
}

#[test]
fn actual_expression_observations_use_original_ordinals_and_stale_views_refuse() {
    let facts = facts();
    let root = color("rgb(calc(2 + 3 * 4) 0 0)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    let mut old = None;
    let mut saw_operand = false;
    let mut saw_stale = false;
    while op.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        if let CssAuthoredColorActionDescription::ReadAuthored {
            path: handle,
            observation,
        } = op.as_ref().unwrap().action()
        {
            let action = op.as_mut().unwrap().next_action();
            let _ = action.path_inspection_layout(handle).unwrap();
            let view = action.path_view(handle).unwrap();
            if observation == CssAuthoredColorObservation::NumericHeader
                && view
                    == [
                        CssColorPathStep::Channel(0),
                        CssColorPathStep::ExpressionOperand(0),
                        CssColorPathStep::ExpressionOperand(1),
                        CssColorPathStep::ExpressionOperand(1),
                    ]
            {
                saw_operand = true;
            }
            if let Some(old) = old
                && old != handle
            {
                assert!(action.path_view(old).is_err());
                saw_stale = true;
            }
            if old.is_none() {
                old = Some(handle);
            }
            action.read_authored().unwrap();
        } else {
            step(op.as_mut().unwrap(), &mut banks).unwrap();
        }
    }
    assert!(saw_operand);
    assert!(saw_stale);
    let checkpoint = CssAuthoredColorOperation::take(&mut op).unwrap();
    assert!(
        matches!(checkpoint.outcome(),CssAuthoredColorOutcome::Used(value) if value.channels()==[Some(14.0/255.0),Some(0.0),Some(0.0)])
    );
}

#[test]
fn dropping_an_action_after_site_preparation_preserves_actual_origin_cleanup_and_no_work_replays() {
    let facts = facts();
    let root = color("rgb(calc(2 + 3) 0 0)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    loop {
        if op.as_ref().unwrap().action() == CssAuthoredColorActionDescription::PrepareSite {
            let action = op.as_mut().unwrap().next_action();
            let handle = action.path_handle();
            let _ = action.path_inspection_layout(handle).unwrap();
            if action.path_view(handle).unwrap() == [CssColorPathStep::Channel(0)] {
                action.prepare_site().unwrap();
                if op.as_ref().unwrap().cleanup_layout().owned_origin_bytes > 0 {
                    break;
                }
            } else {
                action.prepare_site().unwrap();
            }
        } else {
            step(op.as_mut().unwrap(), &mut banks).unwrap();
        }
    }
    let before = op.as_ref().unwrap().cleanup_layout();
    assert_eq!(before.path_len, 1);
    assert!(before.path_capacity_bytes > 0);
    let lengths = std::array::from_fn::<_, 3, _>(|index| banks[index].len());
    drop(op.as_mut().unwrap().next_action());
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert_eq!(op.as_ref().unwrap().cleanup_layout(), before);
    assert_eq!(lengths, std::array::from_fn(|index| banks[index].len()));
    assert_eq!(CssAuthoredColorOperation::recover(&mut op), Ok(None));
    drop(op);
    drop(banks);
    assert_eq!(path.len(), 1);
    assert_eq!(path.capacity_bytes(), before.path_capacity_bytes);
    path.cleanup().unwrap();
}

#[test]
fn dropped_record_generation_abandons_before_acknowledgement_and_owner_reuse_is_real() {
    let facts = facts();
    let root = CssColor::transparent();
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    op.as_mut().unwrap().next_action().read_authored().unwrap();
    {
        let mut action = op.as_mut().unwrap().next_action();
        let record = action.record().unwrap();
        assert_eq!(record.kind(), CssColorSlotKind::AuthoredSite);
        assert!(action.record().is_err());
    }
    assert_eq!(op.as_ref().unwrap().bank_length(CssColorBank::Algorithm), 0);
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert_eq!(
        CssAuthoredColorOperation::take(&mut op).err(),
        Some(ColorProtocolError::Terminal)
    );
    let _ = CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    path.cleanup().unwrap();
    let mut other = CssColorOperationOwner::new();
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut other,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    op.as_mut().unwrap().abandon();
    CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    path.cleanup().unwrap();
    // No buffer owner lifetime survives either recovery; original owner can be
    // reused even while the independent root/fact recovery remains meaningful.
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    op.as_mut().unwrap().abandon();
    CssAuthoredColorOperation::recover(&mut op).unwrap();
}

#[test]
fn controlled_and_ordinary_use_identical_actions_and_one_short_limits_refuse_the_real_resource() {
    let facts = facts();
    let root = color("rgb(calc(2 + 3) 0 0)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    let mut count = 0;
    while op.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        count += 1;
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    let lengths = [banks[0].len(), banks[1].len(), banks[2].len()];
    assert_eq!(
        lengths,
        [
            op.as_ref().unwrap().bank_length(CssColorBank::Algorithm),
            op.as_ref().unwrap().bank_length(CssColorBank::Arithmetic),
            op.as_ref().unwrap().bank_length(CssColorBank::Evaluation)
        ]
    );
    let checkpoint = CssAuthoredColorOperation::take(&mut op).unwrap();
    let CssAuthoredColorOutcome::Used(controlled) = checkpoint.outcome() else {
        panic!("used")
    };
    let controlled = *controlled;
    drop(op);
    drop(banks);
    drop(checkpoint);
    let path_steps = path.capacity();
    path.cleanup().unwrap();
    let exact = CssAuthoredColorOrdinaryLimits {
        algorithm_records: lengths[0],
        arithmetic_records: lengths[1],
        evaluation_records: lengths[2],
        path_steps,
        actions: count,
    };
    let result = resolve_authored_color(&root, &facts, facts.identity(), exact).unwrap();
    assert!(
        matches!(result.outcome(),CssAuthoredColorOutcome::Used(value) if value.identical(controlled))
    );
    assert_eq!(
        controlled.channels(),
        [Some(5.0 / 255.0), Some(0.0), Some(0.0)]
    );
    for (field, resource) in [
        (0, CssColorOrdinaryResource::AlgorithmRecords),
        (1, CssColorOrdinaryResource::ArithmeticRecords),
        (2, CssColorOrdinaryResource::EvaluationRecords),
        (3, CssColorOrdinaryResource::PathSteps),
        (4, CssColorOrdinaryResource::Actions),
    ] {
        let mut short = exact;
        match field {
            0 => short.algorithm_records -= 1,
            1 => short.arithmetic_records -= 1,
            2 => short.evaluation_records -= 1,
            3 => short.path_steps -= 1,
            _ => short.actions -= 1,
        };
        assert!(
            matches!(resolve_authored_color(&root,&facts,facts.identity(),short),Err(ref error) if error.exhausted_limit().is_some_and(|(actual, _, _)| actual == resource)),
            "{resource:?}"
        );
    }
}

#[test]
fn abandoning_a_completed_standard_failure_returns_its_sole_backing_for_accounted_cleanup() {
    let facts = facts();
    let root = color("color-mix(in srgb,red calc(-1%),blue)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    while op.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    let cleanup = op.as_ref().unwrap().cleanup_layout();
    assert_eq!(cleanup.path_len, 1);
    assert!(cleanup.owned_origin_bytes > 0);
    op.as_mut().unwrap().abandon();
    let _ = CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    drop(banks);
    assert_eq!(path.len(), 1);
    assert_eq!(path.capacity_bytes(), cleanup.path_capacity_bytes);
    path.cleanup().unwrap();
}

fn stopped_error(
    root: &CssColor,
    facts: &CssColorFacts<'_>,
) -> (AuthoredError, CssAuthoredColorCleanupLayout) {
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            root,
            facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    let error = loop {
        if let Err(error) = step(op.as_mut().unwrap(), &mut banks) {
            break error;
        }
    };
    let cleanup = op.as_ref().unwrap().cleanup_layout();
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert_eq!(
        CssAuthoredColorOperation::take(&mut op).err(),
        Some(ColorProtocolError::Terminal)
    );
    op.as_mut().unwrap().abandon();
    assert_eq!(CssAuthoredColorOperation::recover(&mut op), Ok(None));
    drop(op);
    drop(banks);
    assert_eq!(path.len(), cleanup.path_len);
    assert_eq!(path.capacity_bytes(), cleanup.path_capacity_bytes);
    path.cleanup().unwrap();
    (error, cleanup)
}

#[test]
fn reached_unavailable_unsupported_and_malformed_facts_remain_site_less_and_recoverable() {
    let id = CssColorFactIdentity::new(41, 7);
    let unavailable = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let (error, cleanup) = stopped_error(&color("light-dark(red,blue)"), &unavailable);
    assert_eq!(
        error,
        AuthoredError::Color(CssColorError::from(
            crate::color::ColorErrorKind::Unavailable(CssColorUnavailable::Scheme)
        ))
    );
    assert_eq!(cleanup.path_len, 0);
    assert!(cleanup.path_capacity_bytes > 0);
    let duplicate =
        [CssColorPaletteEntry::new(CssSystemColor::CanvasText, id, Color::opaque_black()); 2];
    let malformed = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Dark,
        &duplicate,
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let (error, cleanup) = stopped_error(&color("light-dark(red,CanvasText)"), &malformed);
    assert_eq!(
        error,
        AuthoredError::Color(CssColorError::from(crate::color::ColorErrorKind::Facts(
            CssColorFactDefect::DuplicatePalette(CssSystemColor::CanvasText)
        )))
    );
    assert_eq!(cleanup.path_len, 1);
    let root = color("red");
    assert!(
        matches!(
            resolve_authored_color(&root, &malformed, id, limits())
                .unwrap()
                .outcome(),
            CssAuthoredColorOutcome::Used(_)
        ),
        "unreached malformed palette is not read"
    );
    let profile = CssColorProfileName::try_new("--p").unwrap();
    let resource = CssColorFactIdentity::new(8, 9);
    let intent = CssColorProfileRenderingIntent::RelativeColorimetric;
    let names = [CssColorProfileComponentName::try_new("x").unwrap()];
    let bindings = [CssColorProfileBinding::new(
        &profile,
        resource,
        CssColorProfileAvailability::Loaded,
        intent,
        1,
        &names,
    )];
    let conversions = [CssColorProfileConversion::Unsupported { profile: resource }];
    let unsupported = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Light,
        &[],
        &bindings,
        &conversions,
        CssDeviceColorFact::Unknown,
    );
    let (error, cleanup) = stopped_error(&color("color(--p .1)"), &unsupported);
    assert_eq!(
        error,
        AuthoredError::Color(CssColorError::from(
            crate::color::ColorErrorKind::Unsupported(CssColorUnsupported::ProfileConversion)
        ))
    );
    assert_eq!(cleanup.path_len, 0);
    let (error, cleanup) = stopped_error(&color("color(from red --p x)"), &unsupported);
    assert_eq!(
        error,
        AuthoredError::Color(CssColorError::from(
            crate::color::ColorErrorKind::Unsupported(CssColorUnsupported::ReverseProfile)
        ))
    );
    assert_eq!(cleanup.path_len, 1);
    assert!(
        cleanup.owned_origin_bytes > 0,
        "actual parsed reference origin remains prepared until cleanup"
    );
}

#[test]
fn outstanding_capacity_reservations_require_their_actual_observation() {
    let facts = facts();
    let root = color("rgb(1 2 3)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    while !matches!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::ReservePath { .. }
    ) {
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    {
        let mut action = op.as_mut().unwrap().next_action();
        let receipt = action.reserve_path().unwrap();
        assert_eq!(receipt.old_capacity(), 0);
        assert!(receipt.actual_capacity() >= 1);
    }
    let observation = CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    drop(banks);
    assert!(observation.is_some());
    assert_eq!(observation, path.capacity_observation());
    let capacity = path.capacity();
    path.capacity_observed().unwrap();
    path.cleanup().unwrap();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    while !matches!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::ReservePath { .. }
    ) {
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    {
        let mut action = op.as_mut().unwrap().next_action();
        let receipt = action.reserve_path().unwrap();
        assert_eq!(receipt.old_len(), 0);
        assert_eq!(receipt.required_len(), 1);
        assert_eq!(receipt.old_capacity(), capacity);
        assert_eq!(receipt.actual_capacity(), capacity);
        assert_eq!(receipt.actual_len(), 0);
        action.capacity_observed().unwrap();
    }
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::AppendPath(CssColorPathStep::Channel(0))
    );
    op.as_mut().unwrap().next_action().append_path().unwrap();
    op.as_mut().unwrap().abandon();
    CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    drop(banks);
    assert_eq!(path.len(), 1);
    assert_eq!(path.capacity(), capacity);
    path.cleanup().unwrap();
    assert!(path.capacity_observation().is_none());
}

#[test]
fn completed_checkpoint_does_not_retain_the_path_buffer_borrow() {
    let facts = facts();
    let root = CssColor::transparent();
    let other_root = color("red");
    let mut owner = CssColorOperationOwner::new();
    let mut other_owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    while op.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    let first = CssAuthoredColorOperation::take(&mut op).unwrap();
    drop(op);
    drop(banks);
    path.cleanup().unwrap();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut other_owner,
            &other_root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    while op.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        step(op.as_mut().unwrap(), &mut banks).unwrap();
    }
    let second = CssAuthoredColorOperation::take(&mut op).unwrap();
    drop(op);
    drop(banks);
    path.cleanup().unwrap();
    assert!(std::ptr::eq(first.authored(), &root));
    assert!(std::ptr::eq(second.authored(), &other_root));
}

#[test]
fn a_wrong_completion_after_prepare_returns_protocol_without_releasing_backing() {
    let facts = facts();
    let root = color("rgb(calc(2 + 3) 0 0)");
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut op = Some(
        CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    loop {
        if op.as_ref().unwrap().action() == CssAuthoredColorActionDescription::PrepareSite {
            op.as_mut().unwrap().next_action().prepare_site().unwrap();
            if op.as_ref().unwrap().cleanup_layout().owned_origin_bytes > 0 {
                break;
            }
        } else {
            step(op.as_mut().unwrap(), &mut banks).unwrap();
        }
    }
    let before = op.as_ref().unwrap().cleanup_layout();
    let lengths = [banks[0].len(), banks[1].len(), banks[2].len()];
    let error = op
        .as_mut()
        .unwrap()
        .next_action()
        .read_authored()
        .unwrap_err();
    assert_eq!(
        error,
        AuthoredError::Protocol(ColorProtocolError::WrongAction)
    );
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert_eq!(before, op.as_ref().unwrap().cleanup_layout());
    assert_eq!(lengths, [banks[0].len(), banks[1].len(), banks[2].len()]);
    assert_eq!(
        CssAuthoredColorOperation::take(&mut op).err(),
        Some(ColorProtocolError::Terminal)
    );
    let _ = CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    drop(banks);
    assert_eq!(path.len(), before.path_len);
    assert_eq!(path.capacity_bytes(), before.path_capacity_bytes);
    path.cleanup().unwrap();
}

#[test]
fn legitimate_contextual_unit_leaves_preserve_source_authored_reachability_and_original_paths() {
    let facts = facts();
    let text = "rgb(calc(1em / 1em) 0 0)";
    let root = color(text);
    let (error, cleanup) = stopped_error(&root, &facts);
    assert_eq!(
        error,
        AuthoredError::Color(CssColorError::from(
            crate::color::ColorErrorKind::Unsupported(CssColorUnsupported::ContextualUnit(
                CssNumericUnit::Length(CssLengthUnit::Em)
            ))
        ))
    );
    assert_eq!(cleanup.path_len, 3);
    assert!(cleanup.owned_origin_bytes > 0);
    let source = parse_used_color(
        CssColorInput::Utf8(text.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits {
            input_units: text.len(),
            algorithm_records: 100_000,
            arithmetic_records: 10_000,
            evaluation_records: 10_000,
            actions: 30_000_000,
        },
    );
    assert!(matches!(
        source,
        Err(ref error) if error.color_error().and_then(CssColorError::unsupported)
            == Some(CssColorUnsupported::ContextualUnit(CssNumericUnit::Length(CssLengthUnit::Em)))
    ));
    let root = color("light-dark(red,rgb(calc(1em / 1em) 0 0))");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    assert!(
        matches!(result.outcome(),CssAuthoredColorOutcome::Used(value) if value.channels()==[Some(1.0),Some(0.0),Some(0.0)])
    );
}
