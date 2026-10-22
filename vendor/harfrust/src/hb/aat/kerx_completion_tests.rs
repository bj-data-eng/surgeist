use super::*;
use crate::hb::completion_fixture::{extended_machine, glyphs, with_font, words};
use crate::{Direction, ShapingRefusal, ShapingStatus};
use read_fonts::{FontData, FontRead};

#[test]
fn kerx_work_zero_refuses_forced_not_normal_advance() {
    with_font(|font| {
        let plan = crate::ShapePlan::new(font, Direction::LeftToRight, None, None, &[]);
        let header: Vec<u8> = [12u32, 1, 0]
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect();
        let subtable = Subtable::read(FontData::new(&header)).unwrap();
        for flags in [0, 0x4000] {
            let bytes = extended_machine(&words(&[0, flags, 0xFFFF]), &words(&[0, 0, 0xFFFF]));
            let table = Subtable1 {
                state_table: aat::ExtendedStateTable::read(FontData::new(&bytes)).unwrap(),
                values: &[],
            };
            let mut buffer = glyphs(2);
            buffer.max_ops = 0;
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            let cache = ClassCache::new();
            ac.machine_class_cache = Some(&cache);
            let mut driver = Driver1 {
                stack: [0; 8],
                depth: 0,
            };
            apply_state_machine_kerning(
                &mut ac,
                &subtable,
                &table,
                &table.state_table,
                &mut driver,
            );
            assert_eq!(ac.buffer.idx, 2);
            assert_eq!(
                ac.buffer.shaping_status(),
                if flags == 0 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                }
            );
            assert!(ac.buffer.successful);
        }
    });
}

#[test]
fn kerx_stack_overflow_reset_pop_and_invalid_values_are_distinct() {
    with_font(|font| {
        let plan = crate::ShapePlan::new(font, Direction::LeftToRight, None, None, &[]);
        let bytes = extended_machine(&words(&[0, 0, 0xFFFF]), &words(&[0, 0, 0xFFFF]));
        let values = [BigEndian::from(3i16)];
        let table = Subtable1 {
            state_table: aat::ExtendedStateTable::read(FontData::new(&bytes)).unwrap(),
            values: &values,
        };
        let push = aat::StateEntry {
            new_state: 0,
            flags: 0x8000,
            payload: 0xFFFF.into(),
        };
        for action in 0..4 {
            let mut buffer = glyphs(9);
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            let mut driver = Driver1 {
                stack: [0; 8],
                depth: 0,
            };
            for index in 0..8 {
                ac.buffer.idx = index;
                assert!(driver
                    .transition(&table, &push, false, 0, &mut ac)
                    .is_some());
            }
            assert_eq!(driver.depth, 8);
            assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
            match action {
                0 => {} // Actual ninth push discards context.
                1 => {
                    // Actual pop creates capacity for reuse.
                    let pop = aat::StateEntry {
                        new_state: 0,
                        flags: 0,
                        payload: 0.into(),
                    };
                    assert!(driver.transition(&table, &pop, false, 0, &mut ac).is_some());
                    assert_eq!(driver.depth, 7);
                }
                2 => {
                    // Explicit font reset is defined behavior, not exhaustion.
                    let reset = aat::StateEntry {
                        new_state: 0,
                        flags: 0x2000,
                        payload: 0xFFFF.into(),
                    };
                    assert!(driver
                        .transition(&table, &reset, false, 0, &mut ac)
                        .is_some());
                    assert_eq!(driver.depth, 0);
                }
                _ => {
                    let missing = aat::StateEntry {
                        new_state: 0,
                        flags: 0,
                        payload: 100.into(),
                    };
                    assert!(driver
                        .transition(&table, &missing, false, 0, &mut ac)
                        .is_none());
                    assert_eq!(driver.depth, 7);
                }
            }
            assert!(driver
                .transition(&table, &push, false, 0, &mut ac)
                .is_some());
            assert_eq!(
                driver.depth,
                match action {
                    0 => 0,
                    2 => 1,
                    _ => 8,
                }
            );
            assert_eq!(
                ac.buffer.shaping_status(),
                if action == 0 {
                    ShapingStatus::Refused(ShapingRefusal::ContextLimit)
                } else {
                    ShapingStatus::Complete
                }
            );
            assert!(ac.buffer.successful);
        }
    });
}
