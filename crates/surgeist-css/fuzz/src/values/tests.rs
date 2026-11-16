use super::*;
use crate::testing::{assert_detected_replayed_and_reduced, key};
use surgeist_fuzz::{OutcomeKind, engine};

fn case(lane: Lane, important: bool) -> ValueCase {
    ValueCase::new(lane, important).unwrap()
}

fn items(words: &[&str]) -> Vec<Item> {
    words
        .iter()
        .map(|word| Item {
            piece: Piece::Word(WORDS.iter().position(|w| w == word).unwrap() as u8),
            gap: 1,
        })
        .collect()
}

#[test]
fn every_table_names_a_real_property_and_the_curated_list_resolves() {
    for (spelling, ..) in ROWS {
        assert!(Name::named(spelling).is_ok(), "{spelling}");
    }
    for (spelling, ..) in CLOSURES {
        assert!(Name::named(spelling).is_ok(), "{spelling}");
    }
    for (index, spelling) in CURATED.iter().enumerate() {
        assert!(Name::curated(index as u8).is_ok(), "{spelling}");
    }
    // The unit-delimited INVALID entries are balanced, so the next `;` ends them.
    for unit in INVALID {
        let opens = unit.matches('{').count() + unit.matches('(').count();
        let closes = unit.matches('}').count() + unit.matches(')').count();
        assert_eq!(opens, closes, "{unit}");
    }
}

/// The metadata sweep: every known property accepts every universal form, with
/// and without `!important`, under the whole evaluator.
#[test]
fn every_known_property_accepts_every_universal_form_under_the_evaluator() {
    let target = ValueTarget::new();
    for property in 0..CssKnownProperty::all().len() {
        for (form, text) in FORMS.iter().enumerate() {
            for important in [false, true] {
                let lane = Lane::Global {
                    property: property as u16,
                    form: form as u8,
                    alias: 0,
                };
                let outcome = target.evaluate(&case(lane, important)).unwrap();
                assert_eq!(
                    outcome.kind(),
                    OutcomeKind::Evaluated,
                    "{} {} {important}: {:?}",
                    CssKnownProperty::all()[property].canonical_name(),
                    text,
                    outcome.detail()
                );
            }
        }
    }
}

#[test]
fn every_alias_writes_the_same_declaration_as_its_canonical_name() {
    let target = ValueTarget::new();
    let mut aliases = 0;
    for (index, known) in CssKnownProperty::all().iter().enumerate() {
        for alias in 1..=known.aliases().len() {
            let lane = Lane::Global {
                property: index as u16,
                form: 0,
                alias: alias as u8,
            };
            let outcome = target.evaluate(&case(lane, false)).unwrap();
            assert_eq!(
                outcome.kind(),
                OutcomeKind::Evaluated,
                "{}: {:?}",
                known.aliases()[alias - 1],
                outcome.detail()
            );
            aliases += 1;
        }
    }
    assert!(aliases > 0, "the metadata lists aliases");
}

#[test]
fn every_literal_captures_replays_and_checks_its_lane_properties() {
    let target = ValueTarget::new();
    let mut lanes = std::collections::HashSet::new();
    for id in 0..ValueTarget::LITERALS {
        let literal = target.literal(id).unwrap();
        let mut captured = Vec::new();
        let observed = engine::evaluate_case(&target, &literal, &mut |bytes| {
            captured = bytes.to_vec();
            Ok(())
        })
        .unwrap();
        assert_eq!(
            observed.outcome.kind(),
            OutcomeKind::Evaluated,
            "literal {id}: {:?}",
            observed.outcome.detail()
        );
        let checked: Vec<_> = observed
            .outcome
            .checked_properties()
            .iter()
            .map(|property| property.as_str().to_owned())
            .collect();
        match &literal.lane {
            Lane::Row { .. } => {
                assert!(
                    checked.iter().any(|p| p == KNOWN || p == ROUND_TRIP),
                    "{checked:?}"
                );
                assert!(checked.iter().any(|p| p == ROUND_TRIP), "{checked:?}");
            }
            Lane::Siblings { .. } => assert!(checked.iter().any(|p| p == SIBLINGS)),
            Lane::Closure { .. } => assert!(checked.iter().any(|p| p == CLOSURE)),
            _ => {}
        }
        lanes.insert(std::mem::discriminant(&literal.lane));
        assert_eq!(target.decode(&captured).unwrap(), literal);
        assert_eq!(target.encode(&literal).unwrap(), captured);
        assert_eq!(
            engine::replay(&target, &captured, &mut |_| Ok(())).unwrap(),
            observed
        );
    }
    assert_eq!(lanes.len(), 3, "rows, siblings and closures");
    assert!(target.literal(ValueTarget::LITERALS).is_err());
}

#[test]
fn normalizing_rows_are_fixed_points_but_not_structurally_equal() {
    // Handwritten from the crate reference: serialization may change the kind of
    // a value (a percentage becomes a number) and is then a fixed point.
    let parser = CssParser::default();
    let first = parser.parse_block_contents("opacity: 25%");
    let text = first.syntax()[0].serialize_specified(limits()).unwrap();
    assert_eq!(text, "opacity: 0.25;");
    let second = parser.parse_block_contents(&text);
    assert_ne!(first.syntax()[0], second.syntax()[0]);
    assert_eq!(
        second.syntax()[0].serialize_specified(limits()).unwrap(),
        text
    );
    // A spelling-only change keeps the value equal.
    let first = parser.parse_block_contents("width: 1.50px");
    let second = parser.parse_block_contents("width: 1.5px");
    assert_eq!(first.syntax()[0], second.syntax()[0]);
}

static WRONG_TEXT: [Row; 22] = {
    let mut rows = ROWS;
    // Planted: expects the four-value margin without its shortest form.
    rows[2] = (
        "margin",
        "1px 1px 1px 1px",
        "margin: 1px 1px 1px 1px;",
        "margin: 1px 1px 1px 1px;",
        false,
    );
    rows
};

static WRONG_RELATION: [Row; 22] = {
    let mut rows = ROWS;
    // Planted: claims a percentage-to-number normalization is spelling only.
    rows[12] = ("opacity", "25%", "opacity: 0.25;", "opacity: 0.25;", true);
    rows
};

static WRONG_NEXT: [Row; 22] = {
    let mut rows = ROWS;
    // Planted: expects the second text of the tiny duration to keep its `0s`.
    rows[20] = (
        "transition",
        "e 0.0000001s",
        "transition: e 0s;",
        "transition: e 0s;",
        false,
    );
    rows
};

fn always_exact(_text: &str) -> bool {
    // Planted: claims every number survives six-place text.
    true
}

#[test]
fn a_broken_second_text_is_detected_replayed_and_reduced() {
    let broken = ValueTarget::with_rows(&WRONG_NEXT);
    let sound = ValueTarget::new();
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case(Lane::Row { row: 20 }, false),
        &key(ROUND_TRIP, "normalized-form-not-a-fixed-point"),
    );
}

#[test]
fn a_broken_fixed_point_claim_is_detected_replayed_and_reduced() {
    let broken = ValueTarget::with_exactness(always_exact);
    let sound = ValueTarget::new();
    // `e 1e-38ms` rounds to `0s` in text, then loses its default once reparsed.
    let tiny = case(
        Lane::Recipe {
            property: CURATED
                .iter()
                .position(|name| *name == "transition")
                .unwrap() as u8,
            items: vec![
                Item {
                    piece: Piece::Word(WORDS.iter().position(|w| *w == "e").unwrap() as u8),
                    gap: 1,
                },
                Item {
                    piece: Piece::Number {
                        sign: 0,
                        integer: 1,
                        fraction: 0,
                        exponent: 2,
                        magnitude: 38,
                        unit: 5,
                    },
                    gap: 0,
                },
            ],
        },
        false,
    );
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &tiny,
        &key(ROUND_TRIP, "normalized-form-not-a-fixed-point"),
    );
}

#[test]
fn six_place_survival_is_decided_from_the_authored_text_alone() {
    for exact in [
        "10px",
        "0.5",
        "+.5px",
        "99999999999999999999",
        "0.005s",
        "e 1ms",
        "1.50px",
        "#12",
        "\"1e5\" red",
    ] {
        // Strings are scanned too, which is only conservative.
        if exact.contains("1e5") {
            assert!(!survives_six_places(exact), "{exact}");
        } else {
            assert!(survives_six_places(exact), "{exact}");
        }
    }
    for lossy in [
        "0.0000001px",
        "100.0000001%",
        "0.0005s",
        "1e-38ms",
        "1E+5",
        "calc(1px + 2px)",
        "clamp(1px, 2px, 3px)",
        "e 1e5",
    ] {
        assert!(!survives_six_places(lossy), "{lossy}");
    }
    // `1e` followed by a non-digit is a unit, not an exponent.
    assert!(survives_six_places("1e"));
    assert!(survives_six_places("1e+"));
}

#[test]
fn two_step_rows_serialize_with_exact_handwritten_texts_at_each_step() {
    let parser = CssParser::default();
    for (value, first, second) in [
        (
            "transition: e 0.0000001s",
            "transition: e 0s;",
            "transition: e;",
        ),
        (
            "border-image: none 100.0000001%",
            "border-image: 100%;",
            "border-image: none;",
        ),
    ] {
        let report = parser.parse_block_contents(value);
        let text = report.syntax()[0].serialize_specified(limits()).unwrap();
        assert_eq!(text, first, "{value}");
        let again = parser.parse_block_contents(&text);
        assert!(again.is_clean(), "{text}");
        let next = again.syntax()[0].serialize_specified(limits()).unwrap();
        assert_eq!(next, second, "{text}");
        let third = parser.parse_block_contents(&next);
        assert_eq!(
            third.syntax()[0].serialize_specified(limits()).unwrap(),
            second
        );
    }
}

#[test]
fn a_broken_known_value_text_is_detected_replayed_and_reduced() {
    let broken = ValueTarget::with_rows(&WRONG_TEXT);
    let sound = ValueTarget::new();
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case(Lane::Row { row: 2 }, false),
        &key(KNOWN, "known-value-text-differs"),
    );
}

#[test]
fn a_broken_spelling_only_claim_is_detected_replayed_and_reduced() {
    let broken = ValueTarget::with_rows(&WRONG_RELATION);
    let sound = ValueTarget::new();
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case(Lane::Row { row: 12 }, false),
        &key(ROUND_TRIP, "spelling-only-serialization-changes-the-value"),
    );
}

#[test]
fn a_broken_sibling_expectation_is_detected_replayed_and_reduced() {
    let broken = ValueTarget::with_rows(&WRONG_TEXT);
    let sound = ValueTarget::new();
    // Valid sibling `margin: 1px 1px 1px 1px` must be recovered beside the junk.
    assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &case(
            Lane::Siblings {
                before: 2,
                invalid: 0,
                after: 5,
                position: 0,
            },
            false,
        ),
        &key(SIBLINGS, "valid-siblings-not-recovered"),
    );
}

fn column_shifted(text: &str, offset: usize) -> (u32, u32) {
    // Planted: every column is one too large.
    let (line, column) = check::expected_coordinates(text, offset);
    (line, column + 1)
}

#[test]
fn a_broken_coordinate_model_is_detected_replayed_and_reduced_through_a_recipe() {
    let broken = ValueTarget::with_checker(Checker {
        coordinates: column_shifted,
    });
    let sound = ValueTarget::new();
    // `width: red ( 10px` is invalid, so its diagnostic needs a position.
    let planted = case(
        Lane::Recipe {
            property: 0,
            items: items(&["red", "(", "10px", "\"s\""]),
        },
        false,
    );
    let reduced = assert_detected_replayed_and_reduced(
        &broken,
        &sound,
        &planted,
        &key(check::COORDINATES, "line-column-disagree"),
    );
    // Same-failure reduction shrinks the recipe while a diagnostic remains.
    let Lane::Recipe { items, .. } = sound.decode(&reduced).unwrap().lane else {
        panic!("recipe lane");
    };
    assert!(items.len() < 4);
}

/// Each known failure still reproduces its exact key, replays from canonical
/// bytes and reduces without losing it. When the product behavior is corrected
/// this test fails: move the case to the checked regressions and delete it here.
#[test]
fn known_failures_reproduce_replay_and_reduce_with_their_exact_keys() {
    use std::{sync::atomic::AtomicBool, time::Duration};
    use surgeist_fuzz::reduction::{ReductionLimits, ReductionStatus, reduce_direct};
    let target = ValueTarget::new();
    let expected = [
        (AGREEMENT, "block-front-rejects-a-clean-value"),
        (ROUND_TRIP, "serialization-does-not-reparse-cleanly"),
    ];
    assert_eq!(expected.len(), usize::from(ValueTarget::KNOWN_FAILURES));
    for (id, (property, discriminator)) in expected.into_iter().enumerate() {
        let known = target.known_failure(id as u8).unwrap();
        let mut captured = Vec::new();
        let observed = engine::evaluate_case(&target, &known, &mut |bytes| {
            captured = bytes.to_vec();
            Ok(())
        })
        .unwrap();
        let wanted = key(property, discriminator);
        assert_eq!(
            observed.outcome.failure_key(),
            Some(&wanted),
            "known failure {id} no longer reproduces: {:?}",
            observed.outcome.detail()
        );
        assert_eq!(
            engine::replay(&target, &captured, &mut |_| Ok(())).unwrap(),
            observed
        );
        let limits = ReductionLimits::new(400, Duration::from_secs(60), 1 << 20, 2).unwrap();
        let report =
            reduce_direct(&target, &captured, &wanted, limits, &AtomicBool::new(false)).unwrap();
        assert_eq!(report.status(), ReductionStatus::Complete);
        let best = report.canonical_input().unwrap_or(report.best_input());
        assert_eq!(best, captured, "known failure {id} is already minimal");
    }
    assert!(target.known_failure(ValueTarget::KNOWN_FAILURES).is_err());
}

#[test]
fn recipes_with_diagnostics_check_their_front_properties_without_a_round_trip() {
    let target = ValueTarget::new();
    let outcome = target
        .evaluate(&case(
            Lane::Recipe {
                property: 0,
                items: items(&["bold"]),
            },
            false,
        ))
        .unwrap();
    assert_eq!(outcome.kind(), OutcomeKind::Evaluated);
    let checked: Vec<_> = outcome
        .checked_properties()
        .iter()
        .map(|p| p.as_str())
        .collect();
    assert!(checked.contains(&check::COORDINATES) && !checked.contains(&ROUND_TRIP));
}

#[test]
fn the_largest_admitted_recipe_stays_within_the_descriptor_limits() {
    let target = ValueTarget::new();
    let limits = target.descriptor().limits();
    let longest = |table: &[&str]| table.iter().map(|text| text.len()).max().unwrap();
    let unit = UNITS
        .iter()
        .position(|u| u.len() == longest(&UNITS))
        .unwrap() as u8;
    let gap = GAPS.iter().position(|g| g.len() == longest(&GAPS)).unwrap() as u8;
    let number = Item {
        piece: Piece::Number {
            sign: 2,
            integer: MAX_DIGITS,
            fraction: MAX_DIGITS,
            exponent: 2,
            magnitude: MAX_EXPONENT,
            unit,
        },
        gap,
    };
    let largest = case(
        Lane::Recipe {
            property: 0,
            items: vec![number; MAX_PIECES],
        },
        true,
    );
    let Lane::Recipe { items, .. } = &largest.lane else {
        unreachable!()
    };
    // Sign, digits, point, `e-`, three exponent digits and the longest unit,
    // plus a separator after all but the last item.
    let per_number = 1 + usize::from(MAX_DIGITS) * 2 + 1 + 2 + 3 + longest(&UNITS);
    let per_word = longest(&WORDS);
    let per_item = per_number.max(per_word);
    let bound = MAX_PIECES * per_item + (MAX_PIECES - 1) * longest(&GAPS);
    let text = recipe_text(items);
    assert_eq!(
        text.len(),
        MAX_PIECES * per_number + (MAX_PIECES - 1) * longest(&GAPS)
    );
    assert!(text.len() <= bound);
    assert!(bound as u64 <= limits.text_units(), "{bound}");
    credit(target.descriptor(), &largest).unwrap();
    let encoded = target.encode(&largest).unwrap();
    assert!(encoded.len() as u64 <= limits.bytes());
    assert_eq!(target.decode(&encoded).unwrap(), largest);
    // Words alone can be the longer item; the bound already takes the larger.
    let words = case(
        Lane::Recipe {
            property: 0,
            items: vec![
                Item {
                    piece: Piece::Word(
                        WORDS.iter().position(|w| w.len() == per_word).unwrap() as u8
                    ),
                    gap,
                };
                MAX_PIECES
            ],
        },
        false,
    );
    credit(target.descriptor(), &words).unwrap();
    let outcome = target.evaluate(&largest).unwrap();
    assert_ne!(
        outcome.kind(),
        OutcomeKind::Failure,
        "{:?}",
        outcome.detail()
    );
}

#[test]
fn numbers_are_built_from_digit_counts_without_the_serializer() {
    assert_eq!(number_text(0, 1, 0, 0, 0, 0), "1");
    assert_eq!(number_text(1, 3, 2, 0, 0, 1), "-123.45px");
    assert_eq!(number_text(2, 0, 1, 1, 308, 2), "+.4e308%");
    assert_eq!(number_text(0, 2, 0, 2, 400, 0), "12e-400");
    assert_eq!(number_text(0, 10, 0, 0, 0, 0), "1234567891");
}

#[test]
fn generation_is_deterministic_bounded_and_reaches_every_lane_and_check() {
    let target = ValueTarget::new();
    let mut lanes = std::collections::HashSet::new();
    let mut checked_round_trips = 0;
    let mut rejected_recipes = 0;
    for seed in 0_u32..700 {
        let entropy: Vec<u8> = (0..120_u32)
            .map(|slot| (seed.wrapping_mul(2_654_435_761).wrapping_add(slot * 40_503) >> 8) as u8)
            .collect();
        let Generation::Generated(generated) = target.generate(&entropy).unwrap() else {
            panic!("value generation never discards");
        };
        assert_eq!(
            target.generate(&entropy).unwrap(),
            Generation::Generated(generated.clone())
        );
        lanes.insert(std::mem::discriminant(&generated.lane));
        let observed = engine::evaluate_case(&target, &generated, &mut |_| Ok(())).unwrap();
        assert_ne!(
            observed.outcome.kind(),
            OutcomeKind::Failure,
            "seed {seed}: {:?}",
            observed.outcome.detail()
        );
        let checked = observed.outcome.checked_properties();
        checked_round_trips += usize::from(checked.iter().any(|p| p.as_str() == ROUND_TRIP));
        rejected_recipes += usize::from(
            matches!(generated.lane, Lane::Recipe { .. })
                && !checked.iter().any(|p| p.as_str() == nodes::RETAINED),
        );
    }
    assert_eq!(lanes.len(), 5);
    assert!(checked_round_trips > 200, "{checked_round_trips}");
    assert!(rejected_recipes > 20, "{rejected_recipes}");
}

#[test]
fn the_strict_codec_round_trips_every_lane_and_rejects_bad_tables_and_bounds() {
    let target = ValueTarget::new();
    let lanes = [
        Lane::Global {
            property: 453,
            form: 7,
            alias: 0,
        },
        Lane::Row { row: 19 },
        Lane::Recipe {
            property: 35,
            items: vec![
                Item {
                    piece: Piece::Number {
                        sign: 2,
                        integer: 40,
                        fraction: 40,
                        exponent: 2,
                        magnitude: 999,
                        unit: 10,
                    },
                    gap: 5,
                },
                Item {
                    piece: Piece::Word(73),
                    gap: 0,
                },
            ],
        },
        Lane::Siblings {
            before: 19,
            invalid: 11,
            after: 0,
            position: 2,
        },
        Lane::Closure { row: 3 },
    ];
    for lane in lanes {
        let original = case(lane, true);
        let bytes = target.encode(&original).unwrap();
        assert_eq!(target.decode(&bytes).unwrap(), original);
        for length in 0..bytes.len() {
            assert!(target.decode(&bytes[..length]).is_err(), "{length}");
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(target.decode(&trailing).is_err());
    }
    for rejected in [
        Lane::Global {
            property: CssKnownProperty::all().len() as u16,
            form: 0,
            alias: 0,
        },
        Lane::Global {
            property: 0,
            form: FORMS.len() as u8,
            alias: 0,
        },
        Lane::Row {
            row: ROWS.len() as u8,
        },
        Lane::Recipe {
            property: CURATED.len() as u8,
            items: Vec::new(),
        },
        Lane::Recipe {
            property: 0,
            items: vec![
                Item {
                    piece: Piece::Word(0),
                    gap: 0
                };
                MAX_PIECES + 1
            ],
        },
        Lane::Recipe {
            property: 0,
            items: vec![Item {
                piece: Piece::Word(WORDS.len() as u8),
                gap: 0,
            }],
        },
        Lane::Recipe {
            property: 0,
            items: vec![Item {
                piece: Piece::Number {
                    sign: 0,
                    integer: MAX_DIGITS + 1,
                    fraction: 0,
                    exponent: 0,
                    magnitude: 0,
                    unit: 0,
                },
                gap: 0,
            }],
        },
        Lane::Siblings {
            before: 0,
            invalid: INVALID.len() as u8,
            after: 0,
            position: 0,
        },
        Lane::Siblings {
            before: 0,
            invalid: 0,
            after: 0,
            position: 3,
        },
        Lane::Closure {
            row: CLOSURES.len() as u8,
        },
    ] {
        assert!(ValueCase::new(rejected, false).is_err());
    }
    let mut tags = target.encode(&case(Lane::Row { row: 0 }, false)).unwrap();
    tags[5] = 2;
    assert!(target.decode(&tags).is_err());
    tags[5] = 0;
    tags[6] = 9;
    assert!(target.decode(&tags).is_err());
}

#[test]
fn reduction_candidates_are_valid_recipes_and_stop_when_refused() {
    let target = ValueTarget::new();
    let recipe = case(
        Lane::Recipe {
            property: 0,
            items: items(&["red", "(", "10px"]),
        },
        false,
    );
    let mut emitted = Vec::new();
    target
        .reduction_candidates(&recipe, &mut |candidate| {
            emitted.push(candidate);
            true
        })
        .unwrap();
    assert!(emitted.len() >= 5);
    let mut calls = 0;
    target
        .reduction_candidates(&recipe, &mut |_| {
            calls += 1;
            false
        })
        .unwrap();
    assert_eq!(calls, 1);
    let mut table = 0;
    target
        .reduction_candidates(&case(Lane::Row { row: 0 }, false), &mut |_| {
            table += 1;
            true
        })
        .unwrap();
    assert_eq!(table, 0);
}
