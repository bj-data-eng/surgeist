//! Independent literals from HTML #the-meter-element authoring requirements,
//! rather than its separately tested effective processing defaults and clamps.
use surgeist_html::*;

const KINDS: [KnownHtmlAttribute; 6] = [
    KnownHtmlAttribute::Value,
    KnownHtmlAttribute::Min,
    KnownHtmlAttribute::Max,
    KnownHtmlAttribute::Low,
    KnownHtmlAttribute::High,
    KnownHtmlAttribute::Optimum,
];
const ROLES: [HtmlMeterNumberRole; 6] = [
    HtmlMeterNumberRole::Value,
    HtmlMeterNumberRole::Minimum,
    HtmlMeterNumberRole::Maximum,
    HtmlMeterNumberRole::Low,
    HtmlMeterNumberRole::High,
    HtmlMeterNumberRole::Optimum,
];
const RELATIONS: [HtmlMeterAuthorRelation; 9] = [
    HtmlMeterAuthorRelation::MinimumValue,
    HtmlMeterAuthorRelation::ValueMaximum,
    HtmlMeterAuthorRelation::MinimumLow,
    HtmlMeterAuthorRelation::LowMaximum,
    HtmlMeterAuthorRelation::MinimumHigh,
    HtmlMeterAuthorRelation::HighMaximum,
    HtmlMeterAuthorRelation::MinimumOptimum,
    HtmlMeterAuthorRelation::OptimumMaximum,
    HtmlMeterAuthorRelation::LowHigh,
];

fn input(attributes: &[Option<Attribute>; 6]) -> HtmlMeterAttributeInput<'_, Attribute> {
    HtmlMeterAttributeInput::<Attribute> {
        value: attributes[0].as_ref(),
        minimum: attributes[1].as_ref(),
        maximum: attributes[2].as_ref(),
        low: attributes[3].as_ref(),
        high: attributes[4].as_ref(),
        optimum: attributes[5].as_ref(),
    }
}
fn attributes(values: [Option<&str>; 6]) -> [Option<Attribute>; 6] {
    std::array::from_fn(|i| values[i].map(|text| Attribute::new(KINDS[i].qualified_name(), text)))
}
fn read(values: [Option<&str>; 6]) -> HtmlMeterAttributeReport {
    HtmlValueInterpreter::new(4096)
        .meter_attributes(
            &KnownHtmlElement::Meter.qualified_name(),
            input(&attributes(values)),
        )
        .unwrap()
}
fn slot(report: &HtmlMeterAttributeReport, index: usize) -> &HtmlMeterNumberReport {
    [
        report.value(),
        report.minimum(),
        report.maximum(),
        report.low(),
        report.high(),
        report.optimum(),
    ][index]
}
fn conforming(report: &HtmlMeterAttributeReport) {
    for relation in RELATIONS {
        assert!(
            matches!(
                report.relation(relation),
                HtmlMeterRelationState::Conforming | HtmlMeterRelationState::NotRequired
            ),
            "{relation:?}: {:?}",
            report.relation(relation)
        );
    }
}
fn syntax(slot: &HtmlMeterNumberReport) {
    assert_eq!(
        slot.author(),
        HtmlMeterNumberAuthorState::Nonconforming(HtmlValueSyntaxError::InvalidNumericSyntax)
    );
    assert_eq!(
        slot.common().unwrap().author(),
        &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax)
    );
}
fn native(values: [Option<&str>; 6]) -> Result<HtmlMeterAttributes, HtmlMeterNativeError> {
    HtmlMeterAttributes::from_syntax(
        values[0]
            .expect("native fixture supplies required value")
            .into(),
        values[1].map(Into::into),
        values[2].map(Into::into),
        values[3].map(Into::into),
        values[4].map(Into::into),
        values[5].map(Into::into),
    )
}

#[test]
fn absent_value_is_required_while_optional_absence_and_comparisons_stay_distinct() {
    let report = read([None; 6]);
    assert_eq!(
        report.value().author(),
        HtmlMeterNumberAuthorState::MissingRequired
    );
    for i in 0..6 {
        let number = slot(&report, i);
        assert_eq!(
            number.attribute().observation(),
            HtmlAttributeObservation::Missing
        );
        assert_eq!(
            number.common().unwrap().author(),
            &HtmlValueAuthorState::Missing
        );
        assert_eq!(
            number.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Missing
        );
        if i != 0 {
            assert_eq!(number.author(), HtmlMeterNumberAuthorState::MissingOptional);
        }
    }
    for relation in [
        HtmlMeterAuthorRelation::MinimumValue,
        HtmlMeterAuthorRelation::ValueMaximum,
    ] {
        assert_eq!(
            report.relation(relation),
            HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::MissingRequiredValue)
        );
    }
    for relation in &RELATIONS[2..] {
        assert_eq!(
            report.relation(*relation),
            HtmlMeterRelationState::NotRequired
        );
    }
}

#[test]
fn missing_required_value_does_not_suppress_optional_number_relationships() {
    let report = read([None, None, None, Some("0.8"), Some("0.2"), Some("2")]);
    assert_eq!(
        report.value().author(),
        HtmlMeterNumberAuthorState::MissingRequired
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::LowHigh),
        HtmlMeterRelationState::Violated
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::OptimumMaximum),
        HtmlMeterRelationState::Violated
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::MinimumLow),
        HtmlMeterRelationState::Conforming
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::HighMaximum),
        HtmlMeterRelationState::Conforming
    );
}

#[test]
fn absent_bounds_supply_zero_and_one_only_for_authored_comparisons() {
    for value in ["0", "-0", "0.5", "1", "000.50", "5e-1"] {
        let report = read([Some(value), None, None, None, None, None]);
        conforming(&report);
        assert_eq!(
            report.value().author(),
            HtmlMeterNumberAuthorState::Conforming
        );
        assert_eq!(
            report.minimum().author(),
            HtmlMeterNumberAuthorState::MissingOptional
        );
        assert_eq!(
            report.maximum().author(),
            HtmlMeterNumberAuthorState::MissingOptional
        );
    }
    for (value, relation) in [
        ("-1", HtmlMeterAuthorRelation::MinimumValue),
        ("2", HtmlMeterAuthorRelation::ValueMaximum),
    ] {
        let report = read([Some(value), None, None, None, None, None]);
        assert_eq!(
            report.value().author(),
            HtmlMeterNumberAuthorState::Conforming
        );
        assert_eq!(report.relation(relation), HtmlMeterRelationState::Violated);
    }
}

#[test]
fn negative_numbers_and_optimum_outside_the_low_high_band_can_conform() {
    let values = [
        Some("-2"),
        Some("-10"),
        Some("0"),
        Some("-4"),
        Some("-1"),
        Some("-9"),
    ];
    let report = read(values);
    conforming(&report);
    for i in 0..6 {
        assert_eq!(
            slot(&report, i).author(),
            HtmlMeterNumberAuthorState::Conforming
        );
    }
    assert!(native(values).is_ok());
    for optimum in ["1", "9"] {
        let values = [
            Some("5"),
            Some("0"),
            Some("10"),
            Some("3"),
            Some("7"),
            Some(optimum),
        ];
        conforming(&read(values));
        assert!(native(values).is_ok());
    }
}

#[test]
fn all_nine_comparisons_report_their_own_literal_violations() {
    let cases = [
        (
            [Some("-1"), None, None, None, None, None],
            HtmlMeterAuthorRelation::MinimumValue,
        ),
        (
            [Some("2"), None, None, None, None, None],
            HtmlMeterAuthorRelation::ValueMaximum,
        ),
        (
            [Some("0.5"), None, None, Some("-1"), None, None],
            HtmlMeterAuthorRelation::MinimumLow,
        ),
        (
            [Some("0.5"), None, None, Some("2"), None, None],
            HtmlMeterAuthorRelation::LowMaximum,
        ),
        (
            [Some("0.5"), None, None, None, Some("-1"), None],
            HtmlMeterAuthorRelation::MinimumHigh,
        ),
        (
            [Some("0.5"), None, None, None, Some("2"), None],
            HtmlMeterAuthorRelation::HighMaximum,
        ),
        (
            [Some("0.5"), None, None, None, None, Some("-1")],
            HtmlMeterAuthorRelation::MinimumOptimum,
        ),
        (
            [Some("0.5"), None, None, None, None, Some("2")],
            HtmlMeterAuthorRelation::OptimumMaximum,
        ),
        (
            [Some("0.5"), None, None, Some("0.8"), Some("0.2"), None],
            HtmlMeterAuthorRelation::LowHigh,
        ),
    ];
    for (values, violated) in cases {
        let report = read(values);
        for relation in RELATIONS {
            if relation == violated {
                assert_eq!(report.relation(relation), HtmlMeterRelationState::Violated);
            } else {
                assert!(
                    matches!(
                        report.relation(relation),
                        HtmlMeterRelationState::Conforming | HtmlMeterRelationState::NotRequired
                    ),
                    "{violated:?}, {relation:?}"
                );
            }
        }
        assert_eq!(
            native(values),
            Err(HtmlMeterNativeError::RelationViolation(violated))
        );
    }
}

#[test]
fn equality_of_all_six_numbers_satisfies_all_relations() {
    for value in ["0", "-0", "-10", "5", "1e999", "-1e999"] {
        let values = [Some(value); 6];
        let report = read(values);
        for relation in RELATIONS {
            assert_eq!(
                report.relation(relation),
                HtmlMeterRelationState::Conforming
            );
        }
        assert!(native(values).is_ok());
    }
}

#[test]
fn malformed_minimum_keeps_independently_decidable_upper_and_low_high_failures() {
    let report = read([
        Some("2"),
        Some("bad"),
        Some("1"),
        Some("0.5"),
        Some("0.25"),
        Some("0"),
    ]);
    syntax(report.minimum());
    for relation in [
        HtmlMeterAuthorRelation::MinimumValue,
        HtmlMeterAuthorRelation::MinimumLow,
        HtmlMeterAuthorRelation::MinimumHigh,
        HtmlMeterAuthorRelation::MinimumOptimum,
    ] {
        assert_eq!(
            report.relation(relation),
            HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::Syntax(
                HtmlMeterNumberRole::Minimum
            ))
        );
    }
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::ValueMaximum),
        HtmlMeterRelationState::Violated
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::LowHigh),
        HtmlMeterRelationState::Violated
    );
    for relation in [
        HtmlMeterAuthorRelation::LowMaximum,
        HtmlMeterAuthorRelation::HighMaximum,
        HtmlMeterAuthorRelation::OptimumMaximum,
    ] {
        assert_eq!(
            report.relation(relation),
            HtmlMeterRelationState::Conforming
        );
    }
}

#[test]
fn malformed_maximum_is_not_one_and_cannot_hide_a_lower_violation() {
    let report = read([Some("-1"), Some("0"), Some("bad"), None, None, None]);
    syntax(report.maximum());
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::MinimumValue),
        HtmlMeterRelationState::Violated
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::ValueMaximum),
        HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::Syntax(
            HtmlMeterNumberRole::Maximum
        ))
    );
    let report = read([Some("2"), None, Some("bad"), None, None, None]);
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::MinimumValue),
        HtmlMeterRelationState::Conforming
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::ValueMaximum),
        HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::Syntax(
            HtmlMeterNumberRole::Maximum
        ))
    );
}

#[test]
fn malformed_optimum_preserves_every_unrelated_relationship() {
    let report = read([
        Some("0.5"),
        Some("0"),
        Some("1"),
        Some("0.3"),
        Some("0.7"),
        Some("bad"),
    ]);
    syntax(report.optimum());
    for relation in RELATIONS {
        assert_eq!(
            report.relation(relation),
            if matches!(
                relation,
                HtmlMeterAuthorRelation::MinimumOptimum | HtmlMeterAuthorRelation::OptimumMaximum
            ) {
                HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::Syntax(
                    HtmlMeterNumberRole::Optimum,
                ))
            } else {
                HtmlMeterRelationState::Conforming
            }
        );
    }
}

#[test]
fn source_presence_guards_precede_irrelevant_operand_failures() {
    let report = read([None, Some("bad"), Some("bad"), Some("bad"), None, None]);
    for relation in [
        HtmlMeterAuthorRelation::MinimumValue,
        HtmlMeterAuthorRelation::ValueMaximum,
    ] {
        assert_eq!(
            report.relation(relation),
            HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::MissingRequiredValue)
        );
    }
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::LowHigh),
        HtmlMeterRelationState::NotRequired
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::MinimumOptimum),
        HtmlMeterRelationState::NotRequired
    );
    let report = read([Some("bad"), Some("bad"), Some("bad"), None, None, None]);
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::MinimumValue),
        HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::Syntax(
            HtmlMeterNumberRole::Minimum
        ))
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::ValueMaximum),
        HtmlMeterRelationState::Unavailable(HtmlMeterRelationUnavailable::Syntax(
            HtmlMeterNumberRole::Value
        ))
    );
}

#[test]
fn mathematical_precision_and_underflow_are_not_rounded_author_checks() {
    assert_eq!(
        native([
            Some("1.00000000000000001"),
            None,
            Some("1"),
            None,
            None,
            None
        ]),
        Err(HtmlMeterNativeError::RelationViolation(
            HtmlMeterAuthorRelation::ValueMaximum
        ))
    );
    assert_eq!(
        native([Some("-1e-999"), None, None, None, None, None]),
        Err(HtmlMeterNativeError::RelationViolation(
            HtmlMeterAuthorRelation::MinimumValue
        ))
    );
    let report = read([
        Some("1.00000000000000001"),
        None,
        Some("1"),
        None,
        None,
        None,
    ]);
    for number in [report.value(), report.maximum()] {
        assert!(
            matches!(number.common().unwrap().recovery(), HtmlValueRecoveryState::Value(number) if number.value() == 1.0)
        );
    }
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::ValueMaximum),
        HtmlMeterRelationState::Violated
    );
    let report = read([Some("-1e-999"), None, None, None, None, None]);
    assert!(
        matches!(report.value().common().unwrap().recovery(), HtmlValueRecoveryState::Value(number) if number.value() == 0.0)
    );
    assert_eq!(
        report.relation(HtmlMeterAuthorRelation::MinimumValue),
        HtmlMeterRelationState::Violated
    );
    conforming(&read([
        Some("-1e-999"),
        Some("-2e-999"),
        None,
        None,
        None,
        None,
    ]));
    conforming(&read([
        Some("2e-999"),
        Some("1e-999"),
        Some("3e-999"),
        None,
        None,
        None,
    ]));
}

#[test]
fn exact_overflow_constraints_are_separate_from_common_representation_failure() {
    let report = read([
        Some("1e999"),
        Some("1e999"),
        Some("1e999"),
        None,
        None,
        None,
    ]);
    conforming(&report);
    for number in [report.value(), report.minimum(), report.maximum()] {
        assert_eq!(number.author(), HtmlMeterNumberAuthorState::Conforming);
        assert_eq!(
            number.common().unwrap().author(),
            &HtmlValueAuthorState::Unrepresentable(
                HtmlValueRepresentationError::FloatingPointOverflow
            )
        );
        assert_eq!(
            number.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Failed(HtmlValueFailure::Representation(
                HtmlValueRepresentationError::FloatingPointOverflow
            ))
        );
    }
}

#[test]
fn arbitrary_exponents_require_no_machine_projection_or_expansion() {
    let e = "999999999999999999999999999999999999999999999999";
    let value = format!("2e{e}");
    let min = format!("1e{e}");
    let max = format!("3e{e}");
    let values = [
        Some(value.as_str()),
        Some(min.as_str()),
        Some(max.as_str()),
        None,
        None,
        None,
    ];
    conforming(&read(values));
    assert!(native(values).is_ok());
    let value = format!("2e-{e}");
    let min = format!("1e-{e}");
    let max = format!("3e-{e}");
    conforming(&read([
        Some(&value),
        Some(&min),
        Some(&max),
        None,
        None,
        None,
    ]));
}

#[test]
fn empty_whitespace_and_prefixes_keep_strict_and_common_recovery_separate() {
    for text in [
        "", " ", "+2tail", " 2", "2e+", "2 ", "NaN", "Infinity", "١", "\u{a0}2",
    ] {
        for i in 0..6 {
            let mut values = [Some("0.5"), None, None, None, None, None];
            values[i] = Some(text);
            let report = read(values);
            syntax(slot(&report, i));
            assert_eq!(
                slot(&report, i).attribute().raw(),
                Some(&DocumentString::from(text))
            );
            if matches!(text, "+2tail" | " 2" | "2e+" | "2 ") {
                assert!(
                    matches!(slot(&report, i).common().unwrap().recovery(), HtmlValueRecoveryState::Value(number) if number.value() == 2.0)
                );
            }
        }
    }
}

#[test]
fn source_metadata_and_prefixed_expanded_names_are_exact_for_each_slot() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "meter");
    let attributes = std::array::from_fn(|i| {
        Some(Attribute::new(
            QualifiedName::new(Namespace::none(), Some("p".into()), KINDS[i].local_name()),
            "0.5",
        ))
    });
    let report = HtmlValueInterpreter::new(10)
        .meter_attributes(&element, input(&attributes))
        .unwrap();
    for (i, (row, anchor)) in [
        (177, "attr-meter-value"),
        (98, "attr-meter-min"),
        (93, "attr-meter-max"),
        (91, "attr-meter-low"),
        (66, "attr-meter-high"),
        (114, "attr-meter-optimum"),
    ]
    .into_iter()
    .enumerate()
    {
        let number = slot(&report, i);
        assert_eq!(number.attribute().element(), &element);
        assert_eq!(number.attribute().raw_attribute(), attributes[i].as_ref());
        assert_eq!(number.attribute().attribute(), KINDS[i]);
        assert_eq!(number.attribute().inventory_rows(), &[row]);
        assert!(
            number
                .attribute()
                .source_clause()
                .ends_with(&format!("#{anchor}"))
        );
        assert_eq!(
            number.attribute().recognition(),
            HtmlAttributeRecognition::Unapplied
        );
        assert_eq!(
            number.attribute().author(),
            HtmlAttributeAuthorState::Unapplied
        );
    }
    conforming(&report);
}

#[test]
fn wrong_elements_and_namespaces_are_unapplied_even_when_slots_are_missing() {
    for element in [
        KnownHtmlElement::Progress.qualified_name(),
        QualifiedName::new(Namespace::none(), None, "meter"),
        QualifiedName::new(Namespace::html(), None, "METER"),
    ] {
        let report = HtmlValueInterpreter::new(0)
            .meter_attributes(&element, input(&attributes([None; 6])))
            .unwrap();
        for i in 0..6 {
            assert!(slot(&report, i).common().is_none());
            assert_eq!(
                slot(&report, i).author(),
                HtmlMeterNumberAuthorState::Unapplied
            );
        }
        for relation in RELATIONS {
            assert_eq!(report.relation(relation), HtmlMeterRelationState::Unapplied);
        }
    }
}

#[test]
fn wrong_roles_remain_exact_and_only_block_dependent_relations() {
    for i in 0..6 {
        for name in [
            KnownHtmlAttribute::Title.qualified_name(),
            QualifiedName::new(Namespace::html(), None, KINDS[i].local_name()),
            QualifiedName::new(
                Namespace::none(),
                None,
                KINDS[i].local_name().to_ascii_uppercase(),
            ),
        ] {
            let mut attributes = attributes([Some("0.5"); 6]);
            attributes[i] = Some(Attribute::new(name, "bad"));
            let report = HtmlValueInterpreter::new(10)
                .meter_attributes(
                    &KnownHtmlElement::Meter.qualified_name(),
                    input(&attributes),
                )
                .unwrap();
            assert_eq!(
                slot(&report, i).attribute().applicability(),
                HtmlAttributeApplicability::DifferentAttribute
            );
            assert_eq!(
                slot(&report, i).attribute().raw_attribute(),
                attributes[i].as_ref()
            );
            assert!(slot(&report, i).common().is_none());
            let dependent: &[HtmlMeterAuthorRelation] = match i {
                0 => &RELATIONS[..2],
                1 => &[
                    HtmlMeterAuthorRelation::MinimumValue,
                    HtmlMeterAuthorRelation::MinimumLow,
                    HtmlMeterAuthorRelation::MinimumHigh,
                    HtmlMeterAuthorRelation::MinimumOptimum,
                ],
                2 => &[
                    HtmlMeterAuthorRelation::ValueMaximum,
                    HtmlMeterAuthorRelation::LowMaximum,
                    HtmlMeterAuthorRelation::HighMaximum,
                    HtmlMeterAuthorRelation::OptimumMaximum,
                ],
                3 => &[
                    HtmlMeterAuthorRelation::MinimumLow,
                    HtmlMeterAuthorRelation::LowMaximum,
                    HtmlMeterAuthorRelation::LowHigh,
                ],
                4 => &[
                    HtmlMeterAuthorRelation::MinimumHigh,
                    HtmlMeterAuthorRelation::HighMaximum,
                    HtmlMeterAuthorRelation::LowHigh,
                ],
                _ => &[
                    HtmlMeterAuthorRelation::MinimumOptimum,
                    HtmlMeterAuthorRelation::OptimumMaximum,
                ],
            };
            for relation in RELATIONS {
                assert_eq!(
                    report.relation(relation),
                    if dependent.contains(&relation) {
                        HtmlMeterRelationState::Unapplied
                    } else {
                        HtmlMeterRelationState::Conforming
                    }
                );
            }
        }
    }
}

#[test]
fn every_original_raw_slot_is_admitted_before_wrong_role_or_element_processing() {
    for i in 0..6 {
        let mut attributes = attributes([None; 6]);
        attributes[i] = Some(Attribute::new(
            KnownHtmlAttribute::Title.qualified_name(),
            "12345",
        ));
        let error = HtmlValueInterpreter::new(4)
            .meter_attributes(&KnownHtmlElement::Div.qualified_name(), input(&attributes))
            .unwrap_err();
        assert_eq!((error.capacity(), error.required()), (4, 5));
    }
    let all = attributes([Some("0"); 6]);
    assert!(
        HtmlValueInterpreter::new(1)
            .meter_attributes(&KnownHtmlElement::Meter.qualified_name(), input(&all))
            .is_ok()
    );
    let missing = HtmlValueInterpreter::new(0)
        .meter_attributes(
            &KnownHtmlElement::Meter.qualified_name(),
            input(&attributes([None; 6])),
        )
        .unwrap();
    assert_eq!(
        missing.value().author(),
        HtmlMeterNumberAuthorState::MissingRequired
    );
    let empty = attributes([Some(""), None, None, None, None, None]);
    syntax(
        HtmlValueInterpreter::new(0)
            .meter_attributes(&KnownHtmlElement::Meter.qualified_name(), input(&empty))
            .unwrap()
            .value(),
    );
}

#[test]
fn utf16_surrogates_remain_exact_and_common_recovery_observes_only_the_prefix() {
    for units in [
        vec![0xD800],
        vec![0xD83D, 0xDE00],
        vec![u16::from(b'2'), 0xD800],
    ] {
        for i in 0..6 {
            let mut attributes = attributes([None; 6]);
            let raw = Attribute::new(
                KINDS[i].qualified_name(),
                DocumentString::from_utf16(units.clone()),
            );
            attributes[i] = Some(raw.clone());
            let error = HtmlValueInterpreter::new(units.len() - 1)
                .meter_attributes(&KnownHtmlElement::Div.qualified_name(), input(&attributes))
                .unwrap_err();
            assert_eq!(
                (error.capacity(), error.required()),
                (units.len() - 1, units.len())
            );
            let report = HtmlValueInterpreter::new(3)
                .meter_attributes(
                    &KnownHtmlElement::Meter.qualified_name(),
                    input(&attributes),
                )
                .unwrap();
            syntax(slot(&report, i));
            assert_eq!(slot(&report, i).attribute().raw_attribute(), Some(&raw));
            if units.first() == Some(&u16::from(b'2')) {
                assert!(
                    matches!(slot(&report, i).common().unwrap().recovery(), HtmlValueRecoveryState::Value(number) if number.value() == 2.0)
                );
            } else {
                assert!(matches!(
                    slot(&report, i).common().unwrap().recovery(),
                    HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(_))
                ));
            }
        }
    }
}

#[test]
fn native_exact_syntax_is_preserved_and_projects_only_in_the_common_report() {
    for values in [
        [Some("000.50"), None, None, None, None, None],
        [
            Some("5e-1"),
            Some("-0.000"),
            Some("1E+0"),
            Some(".25"),
            Some(".75"),
            Some(".9"),
        ],
        [
            Some("1e999"),
            Some("1e999"),
            Some("1e999"),
            Some("1e999"),
            Some("1e999"),
            Some("1e999"),
        ],
        [
            Some("2e-999"),
            Some("1e-999"),
            Some("3e-999"),
            None,
            None,
            None,
        ],
    ] {
        let proof = native(values).unwrap();
        assert_eq!(proof.value(), &DocumentString::from(values[0].unwrap()));
        for (syntax, expected) in [
            proof.minimum(),
            proof.maximum(),
            proof.low(),
            proof.high(),
            proof.optimum(),
        ]
        .into_iter()
        .zip(&values[1..])
        {
            assert_eq!(syntax, expected.map(DocumentString::from).as_ref());
        }
        let output = proof.into_attributes();
        let mut index = 0;
        for i in 0..6 {
            if let Some(expected) = values[i] {
                assert_eq!(
                    output[index],
                    Attribute::new(KINDS[i].qualified_name(), expected)
                );
                index += 1;
            }
        }
        assert_eq!(output.len(), index);
        let mapped = std::array::from_fn(|i| {
            output
                .iter()
                .find(|raw| raw.name() == &KINDS[i].qualified_name())
                .cloned()
        });
        let report = HtmlValueInterpreter::new(4096)
            .meter_attributes(&KnownHtmlElement::Meter.qualified_name(), input(&mapped))
            .unwrap();
        conforming(&report);
    }
}

#[test]
fn native_rejects_each_malformed_slot_without_confusing_it_with_representation() {
    for i in 0..6 {
        for text in ["", "+0.5", "0.5tail", "NaN", "Infinity", "\u{a0}0"] {
            let mut values = [Some("0.5"); 6];
            values[i] = Some(text);
            assert_eq!(
                native(values),
                Err(HtmlMeterNativeError::Syntax {
                    role: ROLES[i],
                    error: HtmlValueSyntaxError::InvalidNumericSyntax
                })
            );
        }
    }
    let raw = DocumentString::from_utf16(vec![0xD800]);
    assert_eq!(
        HtmlMeterAttributes::from_syntax(raw, None, None, None, None, None),
        Err(HtmlMeterNativeError::Syntax {
            role: HtmlMeterNumberRole::Value,
            error: HtmlValueSyntaxError::InvalidNumericSyntax
        })
    );
}

#[test]
fn native_finite_numbers_use_existing_serialization_through_the_same_checked_boundary() {
    for value in [
        0.0,
        -0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::MAX,
        -f64::MAX,
    ] {
        let number = HtmlFloatingPointNumber::try_from(value).unwrap();
        let syntax = number.as_document_string();
        let proof = HtmlMeterAttributes::from_syntax(
            syntax.clone(),
            Some(syntax.clone()),
            Some(syntax.clone()),
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(proof.value(), &syntax);
        assert_eq!(proof.minimum(), Some(&syntax));
        assert_eq!(proof.maximum(), Some(&syntax));
        assert_eq!(proof.low(), None);
        assert_eq!(proof.high(), None);
        assert_eq!(proof.optimum(), None);
        let output = proof.into_attributes();
        assert_eq!(output.len(), 3);
        assert_eq!(output[0].value(), &syntax);
    }
}
