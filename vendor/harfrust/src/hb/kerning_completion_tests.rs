use super::*;
use crate::hb::completion_fixture::{glyphs, with_font, words};
use crate::{Direction, ShapingRefusal, ShapingStatus};
use read_fonts::{FontData, FontRead};

fn machine(flags: u16) -> Vec<u8> {
    let mut data = words(&[4, 8, 14, 18, 0, 2]);
    data.extend_from_slice(&[1, 1, 0, 1, 1, 1]);
    data.extend(words(&[14, 0, 14, flags, 3]));
    data
}

#[test]
fn kern_work_zero_refuses_forced_not_normal_advance() {
    with_font(|font| {
        let plan = hb_ot_shape_plan_t::new(font, Direction::LeftToRight, None, None, &[]);
        for flags in [0, 0x4000] {
            let bytes = machine(flags);
            let table = aat::StateTable::read(FontData::new(&bytes)).unwrap();
            let mut buffer = glyphs(2);
            buffer.max_ops = 1;
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            let cache = ClassCache::new();
            ac.machine_class_cache = Some(&cache);
            apply_state_machine_kerning(&mut ac, &table, false);
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
fn kern_eight_pushes_pop_reuse_and_ninth_push_have_actual_context_outcomes() {
    with_font(|font| {
        let plan = hb_ot_shape_plan_t::new(font, Direction::LeftToRight, None, None, &[]);
        let bytes = machine(0);
        let table = aat::StateTable::read(FontData::new(&bytes)).unwrap();
        let push = aat::StateEntry::read(FontData::new(&words(&[14, 0x8000]))).unwrap();
        let pop = aat::StateEntry::read(FontData::new(&words(&[14, 26]))).unwrap();
        for overflow in [false, true] {
            let mut buffer = glyphs(9);
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            let mut driver = StateMachineDriver {
                stack: [0; 8],
                depth: 0,
            };
            for index in 0..8 {
                ac.buffer.idx = index;
                state_machine_transition(&mut ac, &table, &push, false, &mut driver);
            }
            assert_eq!(driver.depth, 8);
            assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
            if !overflow {
                state_machine_transition(&mut ac, &table, &pop, false, &mut driver);
                assert_eq!(driver.depth, 7);
            }
            state_machine_transition(&mut ac, &table, &push, false, &mut driver);
            assert_eq!(driver.depth, if overflow { 0 } else { 8 });
            assert_eq!(
                ac.buffer.shaping_status(),
                if overflow {
                    ShapingStatus::Refused(ShapingRefusal::ContextLimit)
                } else {
                    ShapingStatus::Complete
                }
            );
        }
        let mut buffer = glyphs(1);
        let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
        let mut driver = StateMachineDriver {
            stack: [0; 8],
            depth: 0,
        };
        state_machine_transition(&mut ac, &table, &push, false, &mut driver);
        let missing = aat::StateEntry::read(FontData::new(&words(&[14, 0x3FFE]))).unwrap();
        state_machine_transition(&mut ac, &table, &missing, false, &mut driver);
        assert_eq!(driver.depth, 0);
        assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
    });
}
use alloc::vec::Vec;
