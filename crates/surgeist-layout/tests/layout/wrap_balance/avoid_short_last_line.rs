//! Genuine direction-resolved capture, Paragraph-only grouping and nowrap.
use super::*;

fn direction_and_hard_groups<S: l::LayoutScalar>() {
    with_style::<S>(l::TextWrapStyle::AvoidShortLastLine);
}
pub(super) fn with_style<S: l::LayoutScalar>(style: l::TextWrapStyle) {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for forced in [false, true] {
            let mut fixture = Fixture::<S>::new(direction, t::Wrap::Word, forced);
            let ordinary = fixture.compute();
            let mut expected = vec![];
            if forced {
                expected.extend([
                    (t::Range::new(0, 7), 60.0, t::LineEnd::Soft),
                    (t::Range::new(7, 8), 10.0, t::LineEnd::Forced),
                ]);
            }
            let start = if forced { 8 } else { 0 };
            expected.extend([
                (t::Range::new(start, start + 4), 30.0, t::LineEnd::Soft),
                (
                    t::Range::new(start + 4, start + 8),
                    40.0,
                    t::LineEnd::Paragraph,
                ),
            ]);
            let before = fixture.prepared.captured_paragraph();
            set_normalized_wrap_style(&mut fixture, style);
            let actual = compute_changed_wrap_style(&fixture);
            fixture.assert_lines(&actual, &expected, direction);
            assert_real_capacity_and_tab_basis(&fixture, &actual, expected.len());
            let replay = fixture.compute();
            fixture.assert_lines(&replay, &expected, direction);
            assert_eq!(fixture.prepared.captured_paragraph(), before);
            let ordinary_expected = if forced {
                vec![
                    (t::Range::new(0, 7), 60.0, t::LineEnd::Soft),
                    (t::Range::new(7, 8), 10.0, t::LineEnd::Forced),
                    (t::Range::new(8, 15), 60.0, t::LineEnd::Soft),
                    (t::Range::new(15, 16), 10.0, t::LineEnd::Paragraph),
                ]
            } else {
                vec![
                    (t::Range::new(0, 7), 60.0, t::LineEnd::Soft),
                    (t::Range::new(7, 8), 10.0, t::LineEnd::Paragraph),
                ]
            };
            fixture.assert_lines(&ordinary, &ordinary_expected, direction);
        }
        let mut nowrap = Fixture::<S>::new(direction, t::Wrap::None, false);
        let auto = nowrap.compute();
        set_normalized_wrap_style(&mut nowrap, style);
        let selected = compute_changed_wrap_style(&nowrap);
        let expected = [(t::Range::new(0, 8), 80.0, t::LineEnd::Paragraph)];
        nowrap.assert_lines(&auto, &expected, direction);
        nowrap.assert_lines(&selected, &expected, direction);
    }
}
#[test]
fn avoid_short_last_line_real_ltr_rtl_captures_keep_forced_and_nowrap_authority_f32() {
    direction_and_hard_groups::<f32>();
}
#[test]
fn avoid_short_last_line_real_ltr_rtl_captures_keep_forced_and_nowrap_authority_f64() {
    direction_and_hard_groups::<f64>();
}
