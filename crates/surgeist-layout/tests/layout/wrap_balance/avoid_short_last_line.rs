//! Direction-resolved capture, Paragraph-only grouping and nowrap.
use super::*;

fn direction_and_hard_groups<S: l::scalar::LayoutScalar>() {
    with_style::<S>(l::text_flow::TextWrapStyle::AvoidShortLastLine);
}
pub(super) fn with_style<S: l::scalar::LayoutScalar>(style: l::text_flow::TextWrapStyle) {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        for forced in [false, true] {
            let mut fixture = Fixture::<S>::new(direction, t::style::Wrap::Word, forced);
            let ordinary = fixture.compute();
            let mut expected = vec![];
            if forced {
                expected.extend([
                    (
                        t::source::Range::new(0, 7),
                        60.0,
                        t::selection::LineEnd::Soft,
                    ),
                    (
                        t::source::Range::new(7, 8),
                        10.0,
                        t::selection::LineEnd::Forced,
                    ),
                ]);
            }
            let start = if forced { 8 } else { 0 };
            expected.extend([
                (
                    t::source::Range::new(start, start + 4),
                    30.0,
                    t::selection::LineEnd::Soft,
                ),
                (
                    t::source::Range::new(start + 4, start + 8),
                    40.0,
                    t::selection::LineEnd::Paragraph,
                ),
            ]);
            let before = fixture
                .prepared
                .captured_owner()
                .unwrap()
                .members()
                .collect::<Vec<_>>();
            set_normalized_wrap_style(&mut fixture, style);
            let actual = compute_changed_wrap_style(&fixture);
            fixture.assert_lines(&actual, &expected, direction);
            assert_real_capacity_and_tab_basis(&fixture, &actual, expected.len());
            let replay = fixture.compute();
            fixture.assert_lines(&replay, &expected, direction);
            assert_eq!(
                fixture
                    .prepared
                    .captured_owner()
                    .unwrap()
                    .members()
                    .collect::<Vec<_>>(),
                before
            );
            let ordinary_expected = if forced {
                vec![
                    (
                        t::source::Range::new(0, 7),
                        60.0,
                        t::selection::LineEnd::Soft,
                    ),
                    (
                        t::source::Range::new(7, 8),
                        10.0,
                        t::selection::LineEnd::Forced,
                    ),
                    (
                        t::source::Range::new(8, 15),
                        60.0,
                        t::selection::LineEnd::Soft,
                    ),
                    (
                        t::source::Range::new(15, 16),
                        10.0,
                        t::selection::LineEnd::Paragraph,
                    ),
                ]
            } else {
                vec![
                    (
                        t::source::Range::new(0, 7),
                        60.0,
                        t::selection::LineEnd::Soft,
                    ),
                    (
                        t::source::Range::new(7, 8),
                        10.0,
                        t::selection::LineEnd::Paragraph,
                    ),
                ]
            };
            fixture.assert_lines(&ordinary, &ordinary_expected, direction);
        }
        let mut nowrap = Fixture::<S>::new(direction, t::style::Wrap::None, false);
        let auto = nowrap.compute();
        set_normalized_wrap_style(&mut nowrap, style);
        let selected = compute_changed_wrap_style(&nowrap);
        let expected = [(
            t::source::Range::new(0, 8),
            80.0,
            t::selection::LineEnd::Paragraph,
        )];
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
