use surgeist_color::{Color, ColorSpace, PredefinedColorSpace};
use surgeist_css::*;

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
fn color(text: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.value().clone()
}
#[test]
fn literal_checked_values_and_exact_expressions_resolve_through_the_public_front_door() {
    let facts = facts();
    for (text, channels, alpha) in [
        ("red", [1.0, 0.0, 0.0], 1.0),
        (
            "#1234",
            [17.0 / 255.0, 34.0 / 255.0, 51.0 / 255.0],
            68.0 / 255.0,
        ),
        ("rgb(round(UP, 5.1, 2) 0 0)", [6.0 / 255.0, 0.0, 0.0], 1.0),
        (
            "rgb(clamp(none, 5, none) 0 0)",
            [5.0 / 255.0, 0.0, 0.0],
            1.0,
        ),
        (
            "rgb(calc(2 + 3 * 4) 0 255 / .5)",
            [14.0 / 255.0, 0.0, 1.0],
            0.5,
        ),
        (
            "light-dark(rgb(255 0 0), currentcolor)",
            [1.0, 0.0, 0.0],
            1.0,
        ),
        ("rgb(from red r g b / .5)", [1.0, 0.0, 0.0], 0.5),
        ("color-mix(in srgb, red 25%, blue)", [0.25, 0.0, 0.75], 1.0),
    ] {
        let root = color(text);
        let result = resolve_authored_color(&root, &facts, facts.identity(), limits())
            .unwrap_or_else(|error| panic!("{text}: {error:?}"));
        assert!(std::ptr::eq(result.authored(), &root));
        assert!(std::ptr::eq(result.facts(), &facts));
        let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
            panic!("{text}: standard failure")
        };
        assert_eq!(
            value.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb),
            "{text}"
        );
        assert_eq!(value.channels(), channels.map(Some), "{text}");
        assert_eq!(value.alpha(), Some(alpha), "{text}");
    }
    assert_eq!(
        Color::from_srgba8([255, 0, 0, 255]).channels(),
        [Some(1.0), Some(0.0), Some(0.0)]
    );
}

#[test]
fn checked_branch_families_and_unreached_nonfinite_expressions_keep_literal_results() {
    let id = CssColorFactIdentity::new(41, 7);
    let palette = [CssColorPaletteEntry::new(
        CssSystemColor::CanvasText,
        id,
        Color::from_srgba8([255, 0, 0, 255]),
    )];
    let facts = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Current(Color::from_srgba8([0, 255, 0, 255])),
        CssColorSchemeFact::Light,
        &palette,
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    let rgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
    for (text, space, channels, alpha) in [
        ("currentcolor", rgb, [0.0, 1.0, 0.0], 1.0),
        ("transparent", rgb, [0.0, 0.0, 0.0], 0.0),
        ("CanvasText", rgb, [1.0, 0.0, 0.0], 1.0),
        ("hsl(0 100% 50%)", rgb, [1.0, 0.0, 0.0], 1.0),
        ("hwb(0 0% 0%)", rgb, [1.0, 0.0, 0.0], 1.0),
        ("lab(50 0 0)", ColorSpace::Lab, [50.0, 0.0, 0.0], 1.0),
        ("lch(50 0 0)", ColorSpace::Lch, [50.0, 0.0, 0.0], 1.0),
        ("oklab(.5 0 0)", ColorSpace::Oklab, [0.5, 0.0, 0.0], 1.0),
        ("oklch(.5 0 0)", ColorSpace::Oklch, [0.5, 0.0, 0.0], 1.0),
        ("color(srgb 1 0 0)", rgb, [1.0, 0.0, 0.0], 1.0),
        ("alpha(from red / .5)", rgb, [1.0, 0.0, 0.0], 0.5),
        ("contrast-color(white)", rgb, [0.0, 0.0, 0.0], 1.0),
        ("device-cmyk(0 1 1 0)", rgb, [1.0, 0.0, 0.0], 1.0),
        (
            "light-dark(red,lab(50 calc(infinity) 0))",
            rgb,
            [1.0, 0.0, 0.0],
            1.0,
        ),
    ] {
        let root = color(text);
        let result = resolve_authored_color(&root, &facts, id, limits())
            .unwrap_or_else(|e| panic!("{text}: {e:?}"));
        let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
            panic!("{text}: standard failure")
        };
        assert_eq!(value.space(), space, "{text}");
        assert_eq!(value.channels(), channels.map(Some), "{text}");
        assert_eq!(value.alpha(), Some(alpha), "{text}");
    }
}

#[test]
fn custom_profile_payloads_are_contiguous_and_keep_their_original_channel_order() {
    let id = CssColorFactIdentity::new(41, 7);
    let resource = CssColorFactIdentity::new(8, 9);
    let profile = CssColorProfileName::try_new("--p").unwrap();
    let intent = CssColorProfileRenderingIntent::RelativeColorimetric;
    let names = [CssColorProfileComponentName::try_new("x").unwrap()];
    let bindings = [CssColorProfileBinding::new(
        &profile,
        resource,
        CssColorProfileAvailability::Loaded,
        intent,
        5,
        &names,
    )];
    let inputs = [0.1, 0.2, 0.3, 0.4, 0.5];
    let target = ColorSpace::Predefined(PredefinedColorSpace::XyzD50);
    let converted = Color::try_new(target, [Some(0.2), Some(0.3), Some(0.4)], Some(1.0)).unwrap();
    let conversions = [CssColorProfileConversion::Resolved(
        CssColorProfileConversionRecord::new(resource, &inputs, intent, target, converted),
    )];
    let facts = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &bindings,
        &conversions,
        CssDeviceColorFact::Unknown,
    );
    for text in [
        "color(--p .1 .2 .3 .4 .5 / .5)",
        "color(from red --p .1 .2 .3 .4 .5 / .5)",
    ] {
        let root = color(text);
        let result = resolve_authored_color(&root, &facts, id, limits())
            .unwrap_or_else(|e| panic!("{text}: {e:?}"));
        let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
            panic!("used")
        };
        assert_eq!(value.space(), target);
        assert_eq!(value.channels(), [Some(0.2), Some(0.3), Some(0.4)]);
        assert_eq!(value.alpha(), Some(0.5));
    }
}

fn bank(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}
#[expect(
    clippy::result_large_err,
    reason = "The real controlled fixture propagates the actual inline owned path and origin without allocating or cloning an error"
)]
fn step<'path, 'css, 'authored: 'css, 'facts>(
    op: &mut CssAuthoredColorOperation<'path, 'css, 'authored, 'facts>,
    banks: &mut [Vec<CssColorSlot<'css>>; 3],
) -> Result<(), CssColorOperationError> {
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
        CssColorOperationError::Unavailable(CssColorUnavailable::CurrentColor)
    );
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert!(CssAuthoredColorOperation::take(&mut op).is_err());
    assert!(op.is_some());
    assert!(CssAuthoredColorOperation::recover(&mut op).is_err());
    assert!(op.is_some());
    op.as_mut().unwrap().abandon();
    op.as_mut().unwrap().abandon();
    let recovery = CssAuthoredColorOperation::recover(&mut op).unwrap();
    assert!(std::ptr::eq(recovery.authored(), &root));
    assert!(std::ptr::eq(recovery.facts(), &facts));
    assert!(recovery.capacity_observation().is_none());
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
    assert!(result.is_current());
    drop(reused);
    drop(banks);
    drop(result);
    path.cleanup().unwrap();
    assert_eq!(path.capacity(), actual);
}

#[test]
fn numerical_error_moves_the_reached_branch_path_before_stopping() {
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
    let error = loop {
        if let Err(error) = step(op.as_mut().unwrap(), &mut banks) {
            break error;
        }
    };
    let CssColorOperationError::Numerical {
        error,
        site: CssColorResolutionSite::Authored { path, origin },
    } = &error
    else {
        panic!("{error:?}")
    };
    assert_eq!(
        *error,
        surgeist_color::ColorError::NonFinite { component: 1 }
    );
    assert_eq!(path.steps(), [CssColorPathStep::Second]);
    assert!(path.capacity() >= 1);
    assert_eq!(*origin, None);
    assert!(CssAuthoredColorOperation::take(&mut op).is_err());
    op.as_mut().unwrap().abandon();
    let _ = CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    drop(banks);
    assert_eq!(buffer.len(), 0);
    assert_eq!(
        buffer.capacity(),
        0,
        "the external numerical error owns the sole allocation"
    );
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
            assert!(guard.reserve_path().is_err());
        }
        assert_eq!(
            op.as_ref().unwrap().action(),
            CssAuthoredColorActionDescription::Stopped
        );
        let recovered = CssAuthoredColorOperation::recover(&mut op).unwrap();
        assert!(recovered.capacity_observation().is_some());
        drop(op);
        drop(banks);
    }
    assert!(first.capacity_observation().is_some());
    assert!(second.capacity_observation().is_some());
    let retained = first.capacity();
    assert!(first.cleanup().is_err());
    first.capacity_observed().unwrap();
    assert!(first.capacity_observed().is_err());
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
fn completed_standard_failure_owns_its_branch_path_and_take_does_not_clone_it() {
    let facts = facts();
    let root = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::StandardFailure(failure) = result.outcome() else {
        panic!("a reached calculated negative weight is standard invalidity")
    };
    assert_eq!(failure.kind, CssColorStandardFailureKind::InvalidComponents);
    assert_eq!(
        failure.path.steps(),
        [CssColorPathStep::First, CssColorPathStep::MixWeight(0)]
    );
    assert!(failure.path.capacity() >= 2);
    let Some(CssValueOrigin::Parsed(origin)) = &failure.origin else {
        panic!("actual calculation opener")
    };
    assert_eq!(
        origin.source().as_str(),
        "light-dark(color-mix(in srgb,red calc(-1%),blue),blue)"
    );
    assert_eq!(origin.span().start().byte_offset().value(), 33);
}

#[test]
fn initial_foreign_facts_and_ordinary_denials_keep_their_original_classes() {
    let facts = facts();
    let root = color("red");
    let mut denied = limits();
    denied.actions = 0;
    assert!(matches!(
        resolve_authored_color(&root, &facts, CssColorFactIdentity::new(0, 0), denied),
        Err(CssColorOrdinaryError::Operation(
            CssColorOperationError::Facts(CssColorFactError::Foreign)
        ))
    ));
    assert!(matches!(
        resolve_authored_color(&root, &facts, facts.identity(), denied),
        Err(CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::Actions,
            required: 1,
            allowed: 0
        })
    ));
    let root = color("rgb(1 2 3)");
    denied = limits();
    denied.path_steps = 0;
    assert!(matches!(
        resolve_authored_color(&root, &facts, facts.identity(), denied),
        Err(CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::PathSteps,
            required: 1,
            allowed: 0
        })
    ));
    denied = limits();
    denied.algorithm_records = 0;
    assert!(matches!(
        resolve_authored_color(&root, &facts, facts.identity(), denied),
        Err(CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::AlgorithmRecords,
            required: 1,
            allowed: 0
        })
    ));
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
fn original_programmatic_and_parsed_scalars_keep_spelling_and_resolve_unsigned_zero() {
    // Values 4 (WD-20240312): signed zeros cannot be written in CSS; 0, +0 and -0
    // all produce the unsigned zero. The authored spelling is still retained.
    let facts = facts();
    let number = |text: &str| {
        CssColorComponent::Number(
            CssColorNumberLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap(),
        )
    };
    let root = CssColor::from_lab(
        CssLabColor::try_new(number("50"), number("-0"), number("1e-999"), None).unwrap(),
    );
    let CssColorComponent::Number(a) = root.lab_value().unwrap().a() else {
        panic!("number")
    };
    assert_eq!(a.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(a.numeric().representation(), "-0");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("used")
    };
    assert_eq!(value.channels()[1].unwrap().to_bits(), 0.0f64.to_bits());
    assert_eq!(value.channels()[2], Some(0.0));
    let root = color("lab(50 -0 1e-999)");
    let CssColorComponent::Number(a) = root.lab_value().unwrap().a() else {
        panic!("number")
    };
    let CssValueOrigin::Parsed(origin) = a.origin() else {
        panic!("original parsed origin")
    };
    assert_eq!(origin.source().as_str(), "lab(50 -0 1e-999)");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("used")
    };
    assert_eq!(value.channels()[1].unwrap().to_bits(), 0.0f64.to_bits());
    assert_eq!(a.numeric().representation(), "-0");
}

#[test]
fn programmatic_and_parsed_calculated_failures_keep_their_actual_origins() {
    let facts = facts();
    let function = CssComponentValue::try_function(
        "calc",
        CssComponentValues::try_new(vec![CssComponentValue::try_token("-1%").unwrap()]).unwrap(),
    )
    .unwrap();
    let programmatic = CssPercentageCalculation::try_from_components(
        CssComponentValues::try_new(vec![function]).unwrap(),
    )
    .unwrap();
    let recovered =
        CssPercentageCalculation::try_from_components(parse_component_values("calc(-1%)").unwrap())
            .unwrap();
    let CssComponentValueRef::Function(function) = recovered.components().items()[0].view() else {
        panic!("function")
    };
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::Parsed(_)
    ));
    for (calculation, origin_kind) in [(programmatic, 0), (recovered, 1)] {
        let weight = CssColorMixWeight::try_calculation(calculation).unwrap();
        let root = CssColor::from_color_mix(
            CssColorMix::try_new(
                None,
                vec![
                    CssColorMixComponent::new(
                        CssColor::from_named(CssNamedColor::try_new("red").unwrap()),
                        Some(weight),
                    ),
                    CssColorMixComponent::new(CssColor::transparent(), None),
                ],
            )
            .unwrap(),
        );
        let original_origin = root.color_mix_value().unwrap().components()[0]
            .weight()
            .unwrap()
            .calculation()
            .unwrap()
            .origin();
        let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
        let CssAuthoredColorOutcome::StandardFailure(failure) = result.outcome() else {
            panic!("negative weight")
        };
        assert_eq!(failure.path.steps(), [CssColorPathStep::MixWeight(0)]);
        assert_eq!(failure.kind, CssColorStandardFailureKind::InvalidComponents);
        match (&failure.origin, origin_kind) {
            (Some(CssValueOrigin::Programmatic), 0) => {}
            (Some(CssValueOrigin::Parsed(origin)), 1) => {
                assert_eq!(origin.source().as_str(), "calc(-1%)");
                let CssValueOrigin::Parsed(original) = original_origin else {
                    panic!("original parsed")
                };
                assert!(
                    std::ptr::eq(origin.source().as_str(), original.source().as_str()),
                    "the actual source backing is shared, not reconstructed"
                );
            }
            actual => panic!("wrong original origin: {actual:?}"),
        }
    }
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
    let recovery = CssAuthoredColorOperation::recover(&mut op).unwrap();
    assert!(std::ptr::eq(recovery.authored(), &root));
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
    assert!(CssAuthoredColorOperation::take(&mut op).is_err());
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
            matches!(resolve_authored_color(&root,&facts,facts.identity(),short),Err(CssColorOrdinaryError::Limit {resource:actual,..}) if actual==resource),
            "{resource:?}"
        );
    }
}

#[test]
fn parser_produced_eof_recovery_remains_resolvable_and_retains_implicit_closures() {
    let text = "color: light-dark(red,rgb(0 0 calc(2 + 3";
    let report = parse_declaration_list_text(text);
    assert!(!report.is_clean());
    assert_eq!(report.syntax().len(), 1);
    let CssKnownPropertyValueRef::Color(value) = report.syntax().as_slice()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("color")
    };
    let root = value.value();
    let CssColorComponent::NumberCalculation(calculation) = &root
        .light_dark_value()
        .unwrap()
        .dark()
        .rgb_value()
        .unwrap()
        .channels()[2]
    else {
        panic!("calculation")
    };
    let CssComponentValueRef::Function(function) = calculation.components().items()[0].view()
    else {
        panic!("function")
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("actual EOF closure")
    };
    assert_eq!(opening.source().as_str(), text);
    assert_eq!(at.source().as_str(), text);
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(41, 7),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Dark,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    let result = resolve_authored_color(root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("recovered checked syntax remains valid")
    };
    assert_eq!(value.channels(), [Some(0.0), Some(0.0), Some(5.0 / 255.0)]);
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    let source = &text[7..];
    let parsed = parse_used_color(
        CssColorInput::Utf8(source.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits {
            input_units: source.len(),
            algorithm_records: 100_000,
            arithmetic_records: 10_000,
            evaluation_records: 10_000,
            actions: 30_000_000,
        },
    )
    .unwrap();
    let CssColorOrdinaryOutcome::Used(source_value) = parsed.outcome() else {
        panic!("canonical source EOF recovery")
    };
    assert_eq!(
        source_value.channels(),
        [Some(0.0), Some(0.0), Some(5.0 / 255.0)]
    );
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
) -> (CssColorOperationError, CssAuthoredColorCleanupLayout) {
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
    assert!(CssAuthoredColorOperation::take(&mut op).is_err());
    op.as_mut().unwrap().abandon();
    let recovery = CssAuthoredColorOperation::recover(&mut op).unwrap();
    assert!(std::ptr::eq(recovery.authored(), root));
    assert!(std::ptr::eq(recovery.facts(), facts));
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
        CssColorOperationError::Unavailable(CssColorUnavailable::Scheme)
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
        CssColorOperationError::Facts(CssColorFactError::DuplicatePalette(
            CssSystemColor::CanvasText
        ))
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
        CssColorOperationError::Unsupported(CssColorUnsupported::ProfileConversion)
    );
    assert_eq!(cleanup.path_len, 0);
    let (error, cleanup) = stopped_error(&color("color(from red --p x)"), &unsupported);
    assert_eq!(
        error,
        CssColorOperationError::Unsupported(CssColorUnsupported::ReverseProfile)
    );
    assert_eq!(cleanup.path_len, 1);
    assert!(
        cleanup.owned_origin_bytes > 0,
        "actual parsed reference origin remains prepared until cleanup"
    );
}

#[test]
fn existing_capacity_reservations_still_require_their_actual_observation() {
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
    let recovery = CssAuthoredColorOperation::recover(&mut op).unwrap();
    drop(op);
    drop(banks);
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
    assert!(std::ptr::eq(recovery.authored(), &root));
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
    assert!(first.is_current());
    assert!(second.is_current());
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
        CssColorOperationError::Protocol(CssColorProtocolError::WrongAction)
    );
    assert_eq!(
        op.as_ref().unwrap().action(),
        CssAuthoredColorActionDescription::Stopped
    );
    assert_eq!(before, op.as_ref().unwrap().cleanup_layout());
    assert_eq!(lengths, [banks[0].len(), banks[1].len(), banks[2].len()]);
    assert!(CssAuthoredColorOperation::take(&mut op).is_err());
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
        CssColorOperationError::Unsupported(CssColorUnsupported::ContextualUnit(
            CssNumericUnit::Length(CssLengthUnit::Em)
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
        Err(CssColorOrdinaryError::Operation(
            CssColorOperationError::Unsupported(CssColorUnsupported::ContextualUnit(
                CssNumericUnit::Length(CssLengthUnit::Em)
            ))
        ))
    ));
    let root = color("light-dark(red,rgb(calc(1em / 1em) 0 0))");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    assert!(
        matches!(result.outcome(),CssAuthoredColorOutcome::Used(value) if value.channels()==[Some(1.0),Some(0.0),Some(0.0)])
    );
}

fn completed_with_banks<'css, 'authored: 'css, 'facts>(
    owner: &'css mut CssColorOperationOwner,
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
) -> (
    CssAuthoredColorCheckpoint<'css, 'authored, 'facts>,
    [Vec<CssColorSlot<'css>>; 3],
) {
    let mut path = CssColorPathBuffer::new();
    let mut banks = std::array::from_fn(|_| Vec::new());
    let mut operation = Some(
        CssAuthoredColorOperation::resolve_authored(
            owner,
            root,
            facts,
            facts.identity(),
            &mut path,
        )
        .unwrap(),
    );
    while operation.as_ref().unwrap().action() != CssAuthoredColorActionDescription::Ready {
        step(operation.as_mut().unwrap(), &mut banks).unwrap();
    }
    let checkpoint = CssAuthoredColorOperation::take(&mut operation).unwrap();
    drop(operation);
    path.cleanup().unwrap();
    (checkpoint, banks)
}
fn completed<'css, 'authored: 'css, 'facts>(
    owner: &'css mut CssColorOperationOwner,
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
) -> CssAuthoredColorCheckpoint<'css, 'authored, 'facts> {
    let (checkpoint, banks) = completed_with_banks(owner, root, facts);
    drop(banks);
    checkpoint
}
fn native_digit(owner: &mut surgeist_color::OperationOwner) -> surgeist_color::DigitSlot<'_> {
    let mut formatter = surgeist_color::format::FiniteFormatter::new(
        owner,
        1.0,
        surgeist_color::format::RoundingMode::TowardPositiveInfinity,
    )
    .unwrap();
    loop {
        let mut action = formatter.next_action();
        match action.description() {
            surgeist_color::ActionDescription::Compute(_) => action.compute().unwrap(),
            surgeist_color::ActionDescription::AppendDigit { .. } => {
                break action.record().unwrap();
            }
            other => panic!("native digit fixture {other:?}"),
        }
    }
}

fn empty_banks() -> CssColorBankLengths {
    CssColorBankLengths::new(0, 0, 0)
}

#[test]
fn used_and_fallback_projection_only_consume_the_actual_matching_outcome() {
    let facts = facts();
    let root = color("red");
    let mut owner = CssColorOperationOwner::new();
    let mut slot = Some(completed(&mut owner, &root, &facts));
    assert!(matches!(
        CssAuthoredColorCheckpoint::fallback(&mut slot, Color::opaque_black()),
        Err(CssColorProtocolError::WrongAction)
    ));
    assert!(slot.as_ref().unwrap().is_current());
    assert!(std::ptr::eq(slot.as_ref().unwrap().authored(), &root));
    let used = CssAuthoredColorCheckpoint::take_used(&mut slot).unwrap();
    assert!(slot.is_none());
    assert_eq!(used.value().srgb_bytes(), Some([255, 0, 0]));
    assert!(used.standard_failure().is_none());
    assert!(used.is_current());
    assert_eq!(used.fact_identity(), facts.identity());
    assert!(std::ptr::eq(used.authored(), &root));
    assert!(std::ptr::eq(used.facts(), &facts));

    let invalid = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let mut owner2 = CssColorOperationOwner::new();
    let mut slot = Some(completed(&mut owner2, &invalid, &facts));
    let CssAuthoredColorOutcome::StandardFailure(cause) = slot.as_ref().unwrap().outcome() else {
        panic!("negative calculated weight")
    };
    let backing = cause.path.steps().as_ptr();
    let capacity = cause.path.capacity();
    assert!(matches!(
        CssAuthoredColorCheckpoint::take_used(&mut slot),
        Err(CssColorProtocolError::WrongAction)
    ));
    assert!(slot.as_ref().unwrap().is_current());
    let used = CssAuthoredColorCheckpoint::fallback(&mut slot, Color::from_srgba8([1, 2, 3, 255]))
        .unwrap();
    let cause = used.standard_failure().unwrap();
    assert_eq!(cause.kind, CssColorStandardFailureKind::InvalidComponents);
    assert_eq!(
        cause.path.steps(),
        [CssColorPathStep::First, CssColorPathStep::MixWeight(0)]
    );
    assert_eq!(cause.path.steps().as_ptr(), backing);
    assert_eq!(cause.path.capacity(), capacity);
    assert_eq!(used.value().srgb_bytes(), Some([1, 2, 3]));
    assert!(matches!(
        CssAuthoredColorCheckpoint::take_used(&mut slot),
        Err(CssColorProtocolError::EmptySlot)
    ));
}

#[test]
fn authored_numerical_success_commits_only_on_take_and_receipt_releases_the_numerical_borrow() {
    use surgeist_color::{
        Computation, NumericalRequest, OperationOwner, SerializationRequest, Transformation,
    };
    let facts = facts();
    let text = "rgb(100% 0 calc(16 + 1) / .5)";
    let root = color(text);
    let mut css_owner = CssColorOperationOwner::new();
    let mut resolved = Some(completed(&mut css_owner, &root, &facts));
    let mut used = Some(CssAuthoredColorCheckpoint::take_used(&mut resolved).unwrap());
    assert_eq!(
        used.as_ref().unwrap().value().channels(),
        [Some(1.0), Some(0.0), Some(17.0 / 255.0)]
    );
    assert_eq!(used.as_ref().unwrap().value().alpha(), Some(0.5));
    let mut numerical_owner = OperationOwner::new();
    for transformation in [
        Transformation::ForceOpaqueAlpha,
        Transformation::QuantizeSrgb,
    ] {
        let mut transform = Some(
            CssAuthoredUsedColorCheckpoint::transform(
                &mut used,
                &mut numerical_owner,
                transformation,
                empty_banks(),
            )
            .unwrap(),
        );
        assert!(used.is_none());
        assert!(matches!(
            CssAuthoredColorTransformation::take(&mut transform),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::NumericalProtocol(surgeist_color::ProtocolError::NotReady)
            ))
        ));
        assert!(transform.is_some());
        let action = transform.as_mut().unwrap().next_action();
        let layout = action.layout();
        assert_eq!(layout.numerical.input_unit_bytes, 0);
        assert_eq!(layout.numerical.slot_read_bytes, 0);
        assert_eq!(layout.numerical.fact_records, [0; 3]);
        assert!(layout.root_path_read_bytes >= std::mem::size_of::<CssColorComponentPath>());
        assert!(layout.root_path_write_bytes >= std::mem::size_of::<CssColorComponentPath>());
        action.compute().unwrap();
        used = Some(CssAuthoredColorTransformation::take(&mut transform).unwrap());
        assert!(transform.is_none());
    }
    assert_eq!(
        used.as_ref().unwrap().value().srgb_bytes(),
        Some([255, 0, 17])
    );
    assert_eq!(used.as_ref().unwrap().value().alpha(), Some(1.0));
    let mut serializer = Some(
        CssAuthoredUsedColorCheckpoint::serialize(
            &mut used,
            &mut numerical_owner,
            SerializationRequest::HtmlCompatible,
            empty_banks(),
            0,
        )
        .unwrap(),
    );
    assert!(matches!(
        CssAuthoredColorSerialization::take(&mut serializer),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::NumericalProtocol(surgeist_color::ProtocolError::NotReady)
        ))
    ));
    let mut digits = Vec::new();
    let mut output = Vec::new();
    loop {
        let pending = serializer.as_ref().unwrap().action();
        if pending == CssColorActionDescription::Serialized {
            break;
        }
        let mut action = serializer.as_mut().unwrap().next_action();
        let layout = action.layout();
        assert_eq!(layout.numerical.input_unit_bytes, 0);
        match pending {
            CssColorActionDescription::Compute(_) => action.compute().unwrap(),
            CssColorActionDescription::ReadNumericalDigit { index } => {
                action.read_numerical_digit(index, &digits[index]).unwrap()
            }
            CssColorActionDescription::AppendNumericalDigit { index } => {
                assert_eq!(index, digits.len());
                digits.push(action.numerical_record().unwrap());
                action.numerical_written(index, &digits[index]).unwrap();
            }
            CssColorActionDescription::UpdateNumericalDigit { index } => {
                digits[index] = action.numerical_record().unwrap();
                action.numerical_written(index, &digits[index]).unwrap();
            }
            CssColorActionDescription::EmitUnit { unit } => {
                assert_eq!(layout.numerical.output_unit_bytes, 1);
                output.push(unit);
                action.emitted(unit).unwrap();
            }
            other => panic!("{other:?}"),
        }
    }
    let receipt = CssAuthoredColorSerialization::take(&mut serializer).unwrap();
    assert_eq!(output, b"#ff0011");
    assert_eq!(receipt.length(), 7);
    assert_eq!(receipt.value().srgb_bytes(), Some([255, 0, 17]));
    assert!(receipt.is_current());
    assert!(receipt.standard_failure().is_none());
    assert!(std::ptr::eq(receipt.authored(), &root));
    assert!(std::ptr::eq(receipt.facts(), &facts));
    assert_eq!(receipt.fact_identity(), facts.identity());
    drop(digits);
    drop(serializer);
    let mut next = Some(
        Computation::new(
            &mut numerical_owner,
            NumericalRequest::WithAlpha {
                value: receipt.value(),
                alpha: None,
            },
        )
        .unwrap(),
    );
    next.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(Computation::take(&mut next).unwrap().alpha(), None);
    assert_eq!(receipt.length(), output.len());
    // Literal expected output above is the oracle; source parity is additional.
    let source = parse_used_color(
        CssColorInput::Utf8(text.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits {
            input_units: 1000,
            algorithm_records: 100_000,
            arithmetic_records: 10_000,
            evaluation_records: 10_000,
            actions: 30_000_000,
        },
    )
    .unwrap();
    let CssColorOrdinaryOutcome::Used(value) = source.outcome() else {
        panic!("source valid")
    };
    let expected = value
        .transformed(Transformation::ForceOpaqueAlpha)
        .unwrap()
        .transformed(Transformation::QuantizeSrgb)
        .unwrap();
    assert!(expected.identical(receipt.value()));
}

#[test]
fn numerical_initialization_refusals_preserve_the_original_checkpoint_and_phase() {
    use surgeist_color::{OperationOwner, SerializationRequest, Transformation};
    let facts = facts();
    let root = color("rgb(calc(1 + 2) 0 0)");
    let mut css_owner = CssColorOperationOwner::new();
    let (checkpoint, mut banks) = completed_with_banks(&mut css_owner, &root, &facts);
    let mut resolved = Some(checkpoint);
    let mut used = Some(CssAuthoredColorCheckpoint::take_used(&mut resolved).unwrap());
    let original = used.as_ref().unwrap().value();
    let mut numerical_owner = OperationOwner::new();
    for index in 0..3 {
        assert!(!banks[index].is_empty());
        let lengths = CssColorBankLengths::new(banks[0].len(), banks[1].len(), banks[2].len());
        assert!(matches!(
            CssAuthoredUsedColorCheckpoint::transform(
                &mut used,
                &mut numerical_owner,
                Transformation::ForceOpaqueAlpha,
                lengths
            ),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::StorageNotEmpty
            ))
        ));
        assert!(used.as_ref().unwrap().is_current());
        assert!(used.as_ref().unwrap().value().identical(original));
        assert!(matches!(
            CssAuthoredUsedColorCheckpoint::serialize(
                &mut used,
                &mut numerical_owner,
                SerializationRequest::Css,
                lengths,
                0
            ),
            Err(CssColorOperationError::Protocol(
                CssColorProtocolError::StorageNotEmpty
            ))
        ));
        assert!(used.as_ref().unwrap().is_current());
        drop(std::mem::take(&mut banks[index]));
    }
    drop(banks);
    let mut occupied_owner = OperationOwner::new();
    let occupied_digits = vec![native_digit(&mut occupied_owner)];
    assert!(matches!(
        CssAuthoredUsedColorCheckpoint::serialize(
            &mut used,
            &mut numerical_owner,
            SerializationRequest::Css,
            empty_banks(),
            occupied_digits.len()
        ),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::NumericalProtocol(
                surgeist_color::ProtocolError::StorageNotEmpty
            )
        ))
    ));
    drop(occupied_digits);
    assert!(used.as_ref().unwrap().is_current());
    assert!(std::ptr::eq(used.as_ref().unwrap().authored(), &root));
    assert!(std::ptr::eq(used.as_ref().unwrap().facts(), &facts));
    let init = CssAuthoredColorTransformation::initialization_layout();
    assert_eq!(
        init.register_bytes,
        CssAuthoredColorTransformation::register_bytes()
    );
    assert_eq!(
        init.checkpoint_bytes,
        CssAuthoredUsedColorCheckpoint::register_bytes()
    );
    assert_eq!(
        init.css_owner_bytes,
        std::mem::size_of::<CssColorOperationOwner>()
    );
    assert_eq!(init.numerical_owner_bytes, OperationOwner::layout_bytes());
    assert_eq!(
        init.bank_length_bytes,
        std::mem::size_of::<CssColorBankLengths>()
    );
    assert_eq!(init.numerical_digit_length_bytes, 0);
    let serialization_init = CssAuthoredColorSerialization::initialization_layout();
    assert_eq!(
        serialization_init.numerical_digit_length_bytes,
        std::mem::size_of::<usize>()
    );
    assert_eq!(
        serialization_init.register_bytes,
        CssAuthoredColorSerialization::register_bytes()
    );
    let mut transform = Some(
        CssAuthoredUsedColorCheckpoint::transform(
            &mut used,
            &mut numerical_owner,
            Transformation::QuantizeSrgb,
            empty_banks(),
        )
        .unwrap(),
    );
    let pending = transform.as_ref().unwrap().action();
    {
        let _guard = transform.as_mut().unwrap().next_action();
    }
    assert_eq!(transform.as_ref().unwrap().action(), pending);
    transform.as_mut().unwrap().abandon();
    assert!(CssAuthoredColorTransformation::take(&mut transform).is_err());
    let recovered = CssAuthoredColorTransformation::recover(&mut transform).unwrap();
    assert!(recovered.value().identical(original));
    assert!(recovered.is_current());
    assert!(transform.is_none());
}

#[test]
fn numerical_failure_transfers_one_root_site_and_recovers_the_distinct_original_fallback_cause() {
    use surgeist_color::{ColorError, OperationOwner, Transformation};
    let facts = facts();
    let root = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let mut css_owner = CssColorOperationOwner::new();
    let mut resolved = Some(completed(&mut css_owner, &root, &facts));
    let CssAuthoredColorOutcome::StandardFailure(cause) = resolved.as_ref().unwrap().outcome()
    else {
        panic!("negative mix weight")
    };
    let cause_ptr = cause.path.steps().as_ptr();
    let origin_source_ptr = match cause.origin.as_ref().unwrap() {
        CssValueOrigin::Parsed(origin) => origin.source().as_str().as_ptr(),
        _ => panic!("parsed"),
    };
    let fallback = Color::try_new(
        ColorSpace::Lab,
        [Some(50.0), Some(0.0), Some(0.0)],
        Some(0.5),
    )
    .unwrap();
    let mut used = Some(CssAuthoredColorCheckpoint::fallback(&mut resolved, fallback).unwrap());
    let mut numerical_owner = OperationOwner::new();
    let mut transform = Some(
        CssAuthoredUsedColorCheckpoint::transform(
            &mut used,
            &mut numerical_owner,
            Transformation::QuantizeSrgb,
            empty_banks(),
        )
        .unwrap(),
    );
    let error = transform
        .as_mut()
        .unwrap()
        .next_action()
        .compute()
        .unwrap_err();
    let CssColorOperationError::Numerical {
        error,
        site: CssColorResolutionSite::Authored { path, origin },
    } = error
    else {
        panic!("authored numerical site")
    };
    assert_eq!(error, ColorError::RequiresSrgb);
    assert!(path.steps().is_empty());
    assert_eq!(path.capacity(), 0);
    assert!(origin.is_none());
    assert_eq!(
        transform.as_ref().unwrap().action(),
        CssColorActionDescription::Stopped
    );
    assert_eq!(
        transform.as_mut().unwrap().next_action().compute(),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::Terminal
        ))
    );
    assert!(matches!(
        CssAuthoredColorTransformation::take(&mut transform),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::Terminal
        ))
    ));
    assert!(transform.is_some());
    transform.as_mut().unwrap().abandon();
    let recovered = CssAuthoredColorTransformation::recover(&mut transform).unwrap();
    assert!(recovered.value().identical(fallback));
    assert!(std::ptr::eq(recovered.authored(), &root));
    assert!(std::ptr::eq(recovered.facts(), &facts));
    let cause = recovered.standard_failure().unwrap();
    assert_eq!(
        cause.path.steps(),
        [CssColorPathStep::First, CssColorPathStep::MixWeight(0)]
    );
    assert_eq!(cause.path.steps().as_ptr(), cause_ptr);
    let CssValueOrigin::Parsed(origin) = cause.origin.as_ref().unwrap() else {
        panic!("parsed")
    };
    assert_eq!(origin.source().as_str().as_ptr(), origin_source_ptr);
    assert_eq!(origin.span().start().byte_offset().value(), 33);
    assert!(recovered.is_current());
}

#[test]
fn serialization_failure_transfers_one_root_site_and_keeps_the_original_cause_for_recovery() {
    use surgeist_color::{ColorError, OperationOwner, SerializationRequest};
    let facts = facts();
    let root = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let mut css_owner = CssColorOperationOwner::new();
    let mut resolved = Some(completed(&mut css_owner, &root, &facts));
    let original = Color::try_new(
        ColorSpace::Hsl,
        [Some(0.0), Some(1e308), Some(1e308)],
        Some(1.0),
    )
    .unwrap();
    let mut used = Some(CssAuthoredColorCheckpoint::fallback(&mut resolved, original).unwrap());
    let cause_ptr = used
        .as_ref()
        .unwrap()
        .standard_failure()
        .unwrap()
        .path
        .steps()
        .as_ptr();
    let mut numerical_owner = OperationOwner::new();
    let mut serializer = Some(
        CssAuthoredUsedColorCheckpoint::serialize(
            &mut used,
            &mut numerical_owner,
            SerializationRequest::Css,
            empty_banks(),
            0,
        )
        .unwrap(),
    );
    let error = serializer
        .as_mut()
        .unwrap()
        .next_action()
        .compute()
        .unwrap_err();
    let CssColorOperationError::Numerical {
        error,
        site: CssColorResolutionSite::Authored { path, origin },
    } = error
    else {
        panic!("root site")
    };
    assert_eq!(error, ColorError::Unrepresentable);
    assert!(path.steps().is_empty());
    assert_eq!(path.capacity(), 0);
    assert!(origin.is_none());
    assert_eq!(
        serializer.as_ref().unwrap().action(),
        CssColorActionDescription::Stopped
    );
    assert!(matches!(
        CssAuthoredColorSerialization::take(&mut serializer),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::Terminal
        ))
    ));
    assert_eq!(
        serializer.as_mut().unwrap().next_action().compute(),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::Terminal
        ))
    );
    let recovered = CssAuthoredColorSerialization::recover(&mut serializer).unwrap();
    assert!(recovered.value().identical(original));
    assert_eq!(
        recovered.standard_failure().unwrap().path.steps().as_ptr(),
        cause_ptr
    );
    assert!(std::ptr::eq(recovered.authored(), &root));
    assert!(std::ptr::eq(recovered.facts(), &facts));
}

#[test]
fn computed_but_untaken_authored_transform_recovers_original_color_and_same_fallback_allocation() {
    use surgeist_color::{OperationOwner, Transformation};
    let facts = facts();
    let root = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let mut css_owner = CssColorOperationOwner::new();
    let mut resolved = Some(completed(&mut css_owner, &root, &facts));
    let original = Color::try_new(
        ColorSpace::Predefined(PredefinedColorSpace::Srgb),
        [Some(0.5), Some(0.0), Some(1.0)],
        Some(0.5),
    )
    .unwrap();
    let mut used = Some(CssAuthoredColorCheckpoint::fallback(&mut resolved, original).unwrap());
    let backing = used
        .as_ref()
        .unwrap()
        .standard_failure()
        .unwrap()
        .path
        .steps()
        .as_ptr();
    let mut numerical_owner = OperationOwner::new();
    let mut transform = Some(
        CssAuthoredUsedColorCheckpoint::transform(
            &mut used,
            &mut numerical_owner,
            Transformation::QuantizeSrgb,
            empty_banks(),
        )
        .unwrap(),
    );
    transform.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(
        transform.as_ref().unwrap().action(),
        CssColorActionDescription::UsedReady
    );
    transform.as_mut().unwrap().abandon();
    let recovered = CssAuthoredColorTransformation::recover(&mut transform).unwrap();
    assert!(recovered.value().identical(original));
    assert_eq!(recovered.value().srgb_bytes(), None);
    assert_eq!(recovered.value().alpha(), Some(0.5));
    assert_eq!(
        recovered.standard_failure().unwrap().path.steps().as_ptr(),
        backing
    );
    assert!(recovered.is_current());
    drop(transform);
    let mut used = Some(recovered);
    let mut committed = Some(
        CssAuthoredUsedColorCheckpoint::transform(
            &mut used,
            &mut numerical_owner,
            Transformation::QuantizeSrgb,
            empty_banks(),
        )
        .unwrap(),
    );
    committed.as_mut().unwrap().next_action().compute().unwrap();
    let committed = CssAuthoredColorTransformation::take(&mut committed).unwrap();
    assert_eq!(committed.value().srgb_bytes(), Some([128, 0, 255]));
    // QuantizeSrgb quantizes RGB channels and preserves resolved alpha.
    assert_eq!(committed.value().alpha(), Some(0.5));
    assert_eq!(
        committed.standard_failure().unwrap().path.steps().as_ptr(),
        backing
    );
    assert!(std::ptr::eq(committed.authored(), &root));
    assert!(std::ptr::eq(committed.facts(), &facts));
}

#[test]
fn authored_serialization_qualifies_real_digits_and_output_and_preserves_cause_in_receipt() {
    use surgeist_color::{Computation, NumericalRequest, OperationOwner, SerializationRequest};
    let facts = facts();
    let root = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let mut css_owner = CssColorOperationOwner::new();
    let mut resolved = Some(completed(&mut css_owner, &root, &facts));
    let mut used = Some(
        CssAuthoredColorCheckpoint::fallback(&mut resolved, Color::from_srgba8([3, 0, 0, 255]))
            .unwrap(),
    );
    let cause_ptr = used
        .as_ref()
        .unwrap()
        .standard_failure()
        .unwrap()
        .path
        .steps()
        .as_ptr();
    let mut foreign_owner = OperationOwner::new();
    let foreign = native_digit(&mut foreign_owner);
    let mut numerical_owner = OperationOwner::new();
    let mut serializer = Some(
        CssAuthoredUsedColorCheckpoint::serialize(
            &mut used,
            &mut numerical_owner,
            SerializationRequest::Css,
            empty_banks(),
            0,
        )
        .unwrap(),
    );
    let mut digits = Vec::new();
    let mut output = Vec::new();
    let mut rejected_read = false;
    let mut rejected_write = false;
    let mut rejected_output = false;
    loop {
        let pending = serializer.as_ref().unwrap().action();
        if pending == CssColorActionDescription::Serialized {
            break;
        }
        let mut action = serializer.as_mut().unwrap().next_action();
        let layout = action.layout();
        assert_eq!(layout.numerical.input_unit_bytes, 0);
        assert_eq!(layout.numerical.fact_records, [0; 3]);
        match pending {
            CssColorActionDescription::Compute(_) => {
                assert!(matches!(
                    action.numerical_record(),
                    Err(CssColorOperationError::Protocol(
                        CssColorProtocolError::NumericalProtocol(
                            surgeist_color::ProtocolError::WrongAction
                        )
                    ))
                ));
                action.compute().unwrap();
            }
            CssColorActionDescription::ReadNumericalDigit { index } => {
                assert!(
                    layout.numerical.numerical_read_bytes
                        >= std::mem::size_of::<surgeist_color::DigitSlot<'_>>()
                            + OperationOwner::layout_bytes()
                );
                if !rejected_read {
                    assert_eq!(
                        action.read_numerical_digit(index, &foreign),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::NumericalProtocol(
                                surgeist_color::ProtocolError::ForeignSlot
                            )
                        ))
                    );
                    assert_eq!(serializer.as_ref().unwrap().action(), pending);
                    serializer
                        .as_mut()
                        .unwrap()
                        .next_action()
                        .read_numerical_digit(index, &digits[index])
                        .unwrap();
                    rejected_read = true;
                } else {
                    action.read_numerical_digit(index, &digits[index]).unwrap();
                }
            }
            CssColorActionDescription::AppendNumericalDigit { index } => {
                assert_eq!(index, digits.len());
                let record = action.numerical_record().unwrap();
                if !rejected_write {
                    assert_eq!(
                        action.numerical_written(index + 1, &record),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::NumericalProtocol(
                                surgeist_color::ProtocolError::WrongIndex
                            )
                        ))
                    );
                    assert_eq!(serializer.as_ref().unwrap().action(), pending);
                    digits.push(record);
                    serializer
                        .as_mut()
                        .unwrap()
                        .next_action()
                        .numerical_written(index, &digits[index])
                        .unwrap();
                    rejected_write = true;
                } else {
                    digits.push(record);
                    action.numerical_written(index, &digits[index]).unwrap();
                }
            }
            CssColorActionDescription::UpdateNumericalDigit { index } => {
                digits[index] = action.numerical_record().unwrap();
                action.numerical_written(index, &digits[index]).unwrap();
            }
            CssColorActionDescription::EmitUnit { unit } => {
                if !rejected_output {
                    assert_eq!(
                        action.emitted(unit.wrapping_add(1)),
                        Err(CssColorOperationError::Protocol(
                            CssColorProtocolError::NumericalProtocol(
                                surgeist_color::ProtocolError::WrongReply
                            )
                        ))
                    );
                    assert_eq!(serializer.as_ref().unwrap().action(), pending);
                    serializer
                        .as_mut()
                        .unwrap()
                        .next_action()
                        .emitted(unit)
                        .unwrap();
                    rejected_output = true;
                } else {
                    action.emitted(unit).unwrap();
                }
                output.push(unit);
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(rejected_read && rejected_write && rejected_output);
    let receipt = CssAuthoredColorSerialization::take(&mut serializer).unwrap();
    assert_eq!(output, b"rgb(3, 0, 0)");
    assert_eq!(receipt.length(), b"rgb(3, 0, 0)".len());
    assert_eq!(
        receipt.standard_failure().unwrap().path.steps().as_ptr(),
        cause_ptr
    );
    assert_eq!(
        receipt.standard_failure().unwrap().path.steps(),
        [CssColorPathStep::First, CssColorPathStep::MixWeight(0)]
    );
    assert!(std::ptr::eq(receipt.authored(), &root));
    assert!(std::ptr::eq(receipt.facts(), &facts));
    assert!(matches!(
        CssAuthoredColorSerialization::take(&mut serializer),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::EmptySlot
        ))
    ));
    drop(digits);
    drop(serializer);
    let mut next = Some(
        Computation::new(
            &mut numerical_owner,
            NumericalRequest::WithAlpha {
                value: receipt.value(),
                alpha: None,
            },
        )
        .unwrap(),
    );
    next.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(Computation::take(&mut next).unwrap().alpha(), None);
    assert!(receipt.is_current());
}

#[test]
fn abandoning_an_unacknowledged_numerical_record_recovers_original_without_publishing_partial_output()
 {
    use surgeist_color::{OperationOwner, SerializationRequest};
    let facts = facts();
    let root = color("rgb(3 0 0)");
    let mut css_owner = CssColorOperationOwner::new();
    let mut resolved = Some(completed(&mut css_owner, &root, &facts));
    let mut used = Some(CssAuthoredColorCheckpoint::take_used(&mut resolved).unwrap());
    let original = used.as_ref().unwrap().value();
    let mut numerical_owner = OperationOwner::new();
    let mut serializer = Some(
        CssAuthoredUsedColorCheckpoint::serialize(
            &mut used,
            &mut numerical_owner,
            SerializationRequest::Css,
            empty_banks(),
            0,
        )
        .unwrap(),
    );
    let mut partial = Vec::new();
    loop {
        let pending = serializer.as_ref().unwrap().action();
        if matches!(
            pending,
            CssColorActionDescription::AppendNumericalDigit { .. }
        ) {
            {
                let mut action = serializer.as_mut().unwrap().next_action();
                let _record = action.numerical_record().unwrap();
            }
            assert_eq!(serializer.as_ref().unwrap().action(), pending);
            break;
        }
        let action = serializer.as_mut().unwrap().next_action();
        match pending {
            CssColorActionDescription::Compute(_) => action.compute().unwrap(),
            CssColorActionDescription::EmitUnit { unit } => {
                action.emitted(unit).unwrap();
                partial.push(unit);
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(!partial.is_empty());
    serializer.as_mut().unwrap().abandon();
    assert_eq!(
        serializer.as_ref().unwrap().action(),
        CssColorActionDescription::Stopped
    );
    assert!(matches!(
        CssAuthoredColorSerialization::take(&mut serializer),
        Err(CssColorOperationError::Protocol(
            CssColorProtocolError::NumericalProtocol(surgeist_color::ProtocolError::Terminated)
        ))
    ));
    let recovered = CssAuthoredColorSerialization::recover(&mut serializer).unwrap();
    assert!(recovered.value().identical(original));
    assert!(std::ptr::eq(recovered.authored(), &root));
    assert!(std::ptr::eq(recovered.facts(), &facts));
    assert!(recovered.standard_failure().is_none());
    partial.clear();
    assert!(partial.is_empty());
    assert!(serializer.is_none());
}
