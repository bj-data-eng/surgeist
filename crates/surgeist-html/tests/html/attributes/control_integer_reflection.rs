use surgeist_html::bounded::attributes::{
    HtmlControlDimensionRule::{InputSize, TextareaCols, TextareaRows},
    HtmlIntegerAttributeRead::{IntegerUnit, ResultSelection, ValueHeader},
    control_length_reflection_with, control_length_setter_value,
};

#[test]
fn integer_reflection_uses_the_pinned_ranges_and_defaults() {
    for (raw, length, dimensions) in [
        (None, -1, [20, 20, 2]),
        (Some(""), -1, [20, 20, 2]),
        (Some("-0"), 0, [20, 20, 2]),
        (Some("-01"), -1, [20, 20, 2]),
        (Some(" \t+007tail"), 7, [7, 7, 7]),
        (Some("2147483647"), i32::MAX, [2_147_483_647; 3]),
        (Some("2147483648"), -1, [20, 20, 2]),
        (Some("999999999999999999999999999999"), -1, [20, 20, 2]),
    ] {
        let units = raw.map(|value| value.encode_utf16().collect::<Vec<_>>());
        assert_eq!(
            control_length_reflection_with(units.as_deref(), |_| Ok::<_, ()>(())),
            Ok(length),
        );
        for (rule, expected) in [InputSize, TextareaCols, TextareaRows]
            .into_iter()
            .zip(dimensions)
        {
            assert_eq!(
                rule.read_with(units.as_deref(), |_| Ok::<_, ()>(())),
                Ok(expected),
            );
        }
    }

    assert!(control_length_setter_value(i32::MIN).is_err());
    assert!(control_length_setter_value(-1).is_err());
    assert_eq!(control_length_setter_value(0).unwrap(), 0);
    assert_eq!(control_length_setter_value(i32::MAX).unwrap(), i32::MAX);
    assert!(InputSize.setter_value(0).is_err());
    for (rule, default) in [(InputSize, 20), (TextareaCols, 20), (TextareaRows, 2)] {
        assert_eq!(rule.setter_value(1).unwrap(), 1);
        assert_eq!(rule.setter_value(2_147_483_647).unwrap(), 2_147_483_647);
        assert_eq!(rule.setter_value(2_147_483_648).unwrap(), default);
        assert_eq!(rule.setter_value(u32::MAX).unwrap(), default);
    }
    assert_eq!(TextareaCols.setter_value(0).unwrap(), 20);
    assert_eq!(TextareaRows.setter_value(0).unwrap(), 2);
}

#[test]
fn integer_reflection_admits_exactly_the_reached_recovery_steps() {
    // Two digits and the first terminator complete the grammar scan. The
    // following isolated surrogate is outside the reached integer prefix.
    let raw = &[0x30, 0x37, 0x21, 0xd800];
    let present = [
        ValueHeader,
        IntegerUnit,
        IntegerUnit,
        IntegerUnit,
        IntegerUnit,
        IntegerUnit,
        ResultSelection,
    ];
    for (raw, expected, actions) in [
        (None, -1, &[ValueHeader][..]),
        (Some(&raw[..]), 7, &present[..]),
    ] {
        for limit in 0..=actions.len() {
            let mut position = 0;
            let actual = control_length_reflection_with(raw, |step| {
                assert_eq!(step, actions[position]);
                if position == limit {
                    return Err(step);
                }
                position += 1;
                Ok(())
            });
            assert_eq!(position, limit);
            assert_eq!(
                actual,
                actions.get(limit).copied().map_or(Ok(expected), Err)
            );
        }
    }
}
