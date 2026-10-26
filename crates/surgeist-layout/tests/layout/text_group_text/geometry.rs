use super::*;

fn pair_geometry<S: l::LayoutScalar>(
    direction: l::Direction,
    group: l::TextGroupAlign,
    align: l::TextAlign,
    expected: [f64; 2],
    insets: (f64, f64),
) {
    let mut host = Host::<S>::new(direction, group);
    host.box_change(0, |root| root.text_align = align);
    let paragraph = Paragraph::pair(&mut host, 0);
    let batch = compute(&host, &[&paragraph], &[]);
    let layout = placed(&batch, &paragraph);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [t::Range::new(0, 6), t::Range::new(6, 10)]
    );
    assert_eq!(layout.nonformatted_lines().count(), 0);
    assert_eq!(
        glyph_extents(layout),
        [
            (expected[0], expected[0] + 60.0),
            (expected[1], expected[1] + 40.0)
        ]
    );
    let lines = receipts(&batch, &paragraph);
    assert_eq!(lines.len(), 2);
    for (index, (line, natural)) in lines.iter().zip([60.0, 40.0]).enumerate() {
        assert_eq!(
            line.selected_line()
                .finalize(t::SpacingAllocations::None)
                .unwrap()
                .measurements()
                .alignment()
                .advance(),
            natural
        );
        assert_eq!(line.feedback().advance, natural);
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(request.context.available_inline, Some(100.0));
        assert_eq!(
            (
                request.context.alignment_insets.line_left(),
                request.context.alignment_insets.line_right()
            ),
            insets
        );
        assert_eq!(
            request.context.line_box_direction,
            if direction == l::Direction::Ltr {
                t::LineBoxDirection::LeftToRight
            } else {
                t::LineBoxDirection::RightToLeft
            }
        );
        assert_eq!(
            request.context.end,
            if index == 0 {
                t::LineEnd::Forced
            } else {
                t::LineEnd::Paragraph
            }
        );
        let t::PreparedTermination::Ordinary(offer) = &request.termination else {
            panic!("grouping retains native mandatory opportunities")
        };
        assert_eq!(
            offer.view().kind(),
            if index == 0 {
                t::PreparedBreakKind::Forced
            } else {
                t::PreparedBreakKind::ParagraphEnd
            }
        );
        assert_eq!(offer.view().boundary(), request.end);
        assert_eq!(
            layout.lines()[index].bounds().size,
            t::Size::new(100.0, 10.0)
        );
        assert_eq!(layout.lines()[index].baseline(), 8.0 + 10.0 * index as f64);
        assert!(request.alternative.is_none());
        if index == 0 {
            assert_eq!(
                request.controls[0].occurrence,
                paragraph.binding.controls[0].occurrence
            );
        }
    }
    // Latin body glyphs remain LTR even when the actual containing line is RTL.
    // The source-leading A's caret/selection must move with its actual glyph.
    assert_eq!(
        layout
            .cursor(t::Cursor::new(0, t::Affinity::After))
            .unwrap()
            .rect()
            .origin
            .x,
        expected[0]
    );
    let selection = layout
        .selection(t::Selection::new(
            t::Cursor::new(0, t::Affinity::After),
            t::Cursor::new(1, t::Affinity::Before),
        ))
        .unwrap();
    assert_eq!(selection.rects().len(), 1);
    assert_eq!(selection.rects()[0].rect().origin.x, expected[0]);
    assert_eq!(selection.rects()[0].rect().size.width, 10.0);
    assert_eq!(
        node_output(&batch, 0).size,
        l::Size::new(S::from_f64(100.0), S::from_f64(20.0))
    );
}
fn alignment_matrix<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for (group, expected, insets) in [
            (l::TextGroupAlign::None, [20.0, 30.0], (0.0, 0.0)),
            (l::TextGroupAlign::Left, [0.0, 10.0], (0.0, 40.0)),
            (l::TextGroupAlign::Right, [40.0, 50.0], (40.0, 0.0)),
            (
                l::TextGroupAlign::Start,
                if direction == l::Direction::Ltr {
                    [0.0, 10.0]
                } else {
                    [40.0, 50.0]
                },
                if direction == l::Direction::Ltr {
                    (0.0, 40.0)
                } else {
                    (40.0, 0.0)
                },
            ),
            (
                l::TextGroupAlign::End,
                if direction == l::Direction::Ltr {
                    [40.0, 50.0]
                } else {
                    [0.0, 10.0]
                },
                if direction == l::Direction::Ltr {
                    (40.0, 0.0)
                } else {
                    (0.0, 40.0)
                },
            ),
        ] {
            pair_geometry::<S>(direction, group, l::TextAlign::Center, expected, insets);
        }
        pair_geometry::<S>(
            direction,
            l::TextGroupAlign::Center,
            l::TextAlign::Start,
            if direction == l::Direction::Ltr {
                [20.0, 20.0]
            } else {
                [20.0, 40.0]
            },
            (20.0, 20.0),
        );
    }
}
#[test]
fn real_text_group_padding_precedes_ordinary_alignment_without_replacing_the_outer_frame() {
    alignment_matrix::<f32>();
    alignment_matrix::<f64>();
}

fn justify<S: l::LayoutScalar>(direction: l::Direction) {
    let mut host = Host::<S>::new(direction, l::TextGroupAlign::Center);
    host.box_change(0, |root| {
        root.text_align = l::TextAlign::Justify;
        root.text_align_last = l::TextAlignLast::Justify;
        root.text_justification = l::TextJustification::Spaces;
    });
    let paragraph = Paragraph::pair(&mut host, 0);
    let batch = compute(&host, &[&paragraph], &[]);
    let layout = placed(&batch, &paragraph);
    assert_eq!(glyph_extents(layout), [(20.0, 80.0), (20.0, 80.0)]);
    for (index, (line, natural)) in receipts(&batch, &paragraph)
        .iter()
        .zip([60.0, 40.0])
        .enumerate()
    {
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(request.context.alignment, t::Alignment::Justify);
        assert_eq!(request.context.available_inline, Some(100.0));
        assert_eq!(
            (
                request.context.alignment_insets.line_left(),
                request.context.alignment_insets.line_right()
            ),
            (20.0, 20.0)
        );
        let before_expansion = selected.finalize(t::SpacingAllocations::None).unwrap();
        assert_eq!(
            before_expansion.measurements().alignment().advance(),
            natural
        );
        assert_eq!(line.measurements().alignment().advance(), 60.0);
        assert_eq!(line.feedback().advance - natural, [0.0, 20.0][index]);
        assert_eq!(line.spacing_slots().count(), [2, 1][index]);
        assert_eq!(
            line.selected_line().request().context.end,
            if index == 0 {
                t::LineEnd::Forced
            } else {
                t::LineEnd::Paragraph
            }
        );
        assert_eq!(
            layout.lines()[index].bounds().size,
            t::Size::new(100.0, 10.0)
        );
    }
    let gap = layout
        .selection(t::Selection::new(
            t::Cursor::new(7, t::Affinity::After),
            t::Cursor::new(8, t::Affinity::Before),
        ))
        .unwrap();
    assert_eq!(gap.rects().len(), 1);
    assert_eq!(gap.rects()[0].rect().origin.x, 30.0);
    assert_eq!(gap.rects()[0].rect().size.width, 30.0);
    assert_eq!(
        layout
            .cursor(t::Cursor::new(0, t::Affinity::After))
            .unwrap()
            .rect()
            .origin
            .x,
        20.0
    );
}
#[test]
fn centered_real_space_justification_expands_zero_and_twenty_inside_the_common_sixty_unit_frame() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        justify::<f32>(direction);
        justify::<f64>(direction);
    }
}

fn natural_frames<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let mut tabs = style();
    tabs.white_space = t::WhiteSpace::Preserve;
    tabs.tab_size = t::TabSize::length(40.0).unwrap();
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &[Part::Text("A\tB"), Part::Forced, Part::Text("CCCCCC")],
        tabs,
        t::ParagraphBaseDirection::LeftToRight,
        true,
    );
    let batch = compute(&host, &[&paragraph], &[]);
    let lines = receipts(&batch, &paragraph);
    assert_eq!(
        lines
            .iter()
            .map(|line| line
                .selected_line()
                .finalize(t::SpacingAllocations::None)
                .unwrap()
                .measurements()
                .alignment()
                .advance())
            .collect::<Vec<_>>(),
        [50.0, 60.0]
    );
    for line in &lines {
        assert_eq!(
            line.selected_line().request().context.tabs,
            t::TabContext::CssStops { origin: Some(0.0) }
        );
        assert_eq!(
            line.selected_line().request().context.available_inline,
            Some(100.0)
        );
    }
    let layout = placed(&batch, &paragraph);
    let first: Vec<_> = layout
        .run_views()
        .filter(|run| run.line() == 0)
        .flat_map(|run| {
            let origin = run.origin().x;
            run.glyphs()
                .filter(|glyph| glyph.advance() > 0.0)
                .map(move |glyph| (glyph.id(), origin + glyph.pen() + f64::from(glyph.x())))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(first, [(35, 20.0), (36, 60.0)]);
    assert_eq!(glyph_extents(layout)[1], (20.0, 80.0));

    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let mut hanging = style();
    hanging.hanging_punctuation =
        t::HangingPunctuation::new(false, Some(t::HangingPunctuationEnd::ForceEnd), false);
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &[Part::Text("AAAAA."), Part::Forced, Part::Text("BB")],
        hanging,
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&paragraph], &[]);
    let lines = receipts(&batch, &paragraph);
    assert_eq!(
        lines
            .iter()
            .map(|line| line
                .selected_line()
                .finalize(t::SpacingAllocations::None)
                .unwrap()
                .measurements()
                .alignment()
                .advance())
            .collect::<Vec<_>>(),
        [50.0, 20.0]
    );
    assert_eq!(lines[0].feedback().advance, 60.0);
    assert_eq!(
        glyph_extents(placed(&batch, &paragraph)),
        [(25.0, 85.0), (25.0, 45.0)]
    );
    for line in lines {
        assert_eq!(
            (
                line.selected_line()
                    .request()
                    .context
                    .alignment_insets
                    .line_left(),
                line.selected_line()
                    .request()
                    .context
                    .alignment_insets
                    .line_right()
            ),
            (25.0, 25.0)
        );
    }
}
#[test]
fn real_tabs_and_hanging_keep_their_outer_natural_basis_before_group_padding() {
    natural_frames::<f32>();
    natural_frames::<f64>();
}

fn empty_receivers<S: l::LayoutScalar>(extra_force: bool) {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let parts = if extra_force {
        vec![
            Part::Text("AA A A"),
            Part::Forced,
            Part::Text(""),
            Part::Forced,
            Part::Text(""),
        ]
    } else {
        vec![Part::Text("AA A A"), Part::Forced, Part::Text("")]
    };
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &parts,
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&paragraph], &[]);
    let layout = placed(&batch, &paragraph);
    assert_eq!(layout.lines().len(), if extra_force { 2 } else { 1 });
    assert_eq!(layout.nonformatted_lines().count(), 1);
    assert_eq!(
        layout.source().text().code_units(),
        "AA A A".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(glyph_extents(layout)[0], (20.0, 80.0));
    let completion = layout.nonformatted_lines().next().unwrap();
    let terminal = completion.line.finalized_line().selected_line();
    assert_eq!(terminal.request().context.end, t::LineEnd::Paragraph);
    assert_eq!(
        terminal.request().context.alignment_insets,
        t::LineAlignmentInsets::ZERO
    );
    assert_eq!(
        completion.line.finalized_line().feedback().line_height,
        10.0
    );
    assert_eq!(
        node_output(&batch, 0).size.height,
        S::from_f64(if extra_force { 20.0 } else { 10.0 })
    );
    let lines = receipts(&batch, &paragraph);
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(
            line.selected_line().request().context.end,
            t::LineEnd::Forced
        );
        assert_eq!(
            line.selected_line().request().controls[0].occurrence,
            paragraph.binding.controls[index].occurrence
        );
        assert_eq!(
            (
                line.selected_line()
                    .request()
                    .context
                    .alignment_insets
                    .line_left(),
                line.selected_line()
                    .request()
                    .context
                    .alignment_insets
                    .line_right()
            ),
            (20.0, 20.0)
        );
    }
}
#[test]
fn real_forced_empty_line_participates_while_the_terminal_phantom_retains_only_custody() {
    for extra_force in [false, true] {
        empty_receivers::<f32>(extra_force);
        empty_receivers::<f64>(extra_force);
    }
}

fn mixed<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &[
            Part::Start(2.0),
            Part::Text("AA"),
            Part::Atomic(14.0),
            Part::Text(" A"),
            Part::End(4.0),
            Part::Forced,
            Part::Text("B BB"),
        ],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let atomic = paragraph.binding.atomics[0].node;
    let interior = Paragraph::new(
        &mut host,
        atomic,
        &[Part::Text("CCCCCCCCC")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&paragraph, &interior], &[]);
    let lines = receipts(&batch, &paragraph);
    assert_eq!(
        lines
            .iter()
            .map(|line| line
                .selected_line()
                .finalize(t::SpacingAllocations::None)
                .unwrap()
                .measurements()
                .alignment()
                .advance())
            .collect::<Vec<_>>(),
        [60.0, 40.0]
    );
    let start = paragraph.binding.controls[0].node;
    let end = paragraph.binding.controls[1].node;
    assert_eq!(node_output(&batch, start).location.x, S::from_f64(20.0));
    assert_eq!(node_output(&batch, end).location.x, S::from_f64(76.0));
    assert_eq!(node_output(&batch, atomic).location.x, S::from_f64(42.0));
    assert_eq!(node_output(&batch, atomic).size.width, S::from_f64(14.0));
    assert_eq!(
        lines[0].selected_line().request().atomics[0].occurrence,
        paragraph.binding.atomics[0].occurrence
    );
    assert_eq!(lines[0].selected_line().request().controls.len(), 3);
    let slot = lines[0].atomic_slots().next().unwrap();
    assert_eq!(slot.origin.x, 42.0);
    assert_eq!(slot.occurrence, paragraph.binding.atomics[0].occurrence);
    let t::AtomicLineAllocation::InFlow { flow, .. } = slot.allocation else {
        panic!("the real inline-block exterior retains its actual allocation");
    };
    assert_eq!(flow.advance, 14.0);
    // Atom exterior14 participates in its parent's60; its90-wide interior
    // is an independent context and cannot consume the parent's remaining40.
    assert_eq!(glyph_extents(placed(&batch, &paragraph))[0], (22.0, 76.0));
    assert_eq!(glyph_extents(placed(&batch, &interior))[0].0, 42.0);
}
#[test]
fn actual_atomic_exterior_endpoints_controls_and_independent_interior_share_the_final_parent_frame()
{
    mixed::<f32>();
    mixed::<f64>();
}

fn real_float_bands<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    let float = host.push(
        Some(0),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            float: l::Float::Left,
            size: l::Size::new(
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(20.0)).unwrap()),
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
            ),
            ..l::NodeInputOf::default()
        }),
    );
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &[Part::Text("AAAAA"), Part::Forced, Part::Text("BBBBBB")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&paragraph], &[]);
    let lines = receipts(&batch, &paragraph);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.selected_line().request().context.available_inline)
            .collect::<Vec<_>>(),
        [Some(80.0), Some(100.0)]
    );
    assert_eq!(
        lines
            .iter()
            .map(|line| line
                .selected_line()
                .finalize(t::SpacingAllocations::None)
                .unwrap()
                .measurements()
                .alignment()
                .advance())
            .collect::<Vec<_>>(),
        [50.0, 60.0]
    );
    for line in &lines {
        let selected = line.selected_line();
        let inset = selected.request().context.alignment_insets;
        assert_eq!((inset.line_left(), inset.line_right()), (15.0, 15.0));
    }
    assert_eq!(
        glyph_extents(placed(&batch, &paragraph)),
        [(35.0, 85.0), (15.0, 75.0)]
    );
    let actual_float = node_output(&batch, float);
    assert_eq!(actual_float.location, l::Point::new(S::ZERO, S::ZERO));
    assert_eq!(
        actual_float.size,
        l::Size::new(S::from_f64(20.0), S::from_f64(10.0))
    );
}
#[test]
fn common_text_padding_uses_actual_offset_float_bands_and_does_not_move_the_float() {
    real_float_bands::<f32>();
    real_float_bands::<f64>();
}

fn signed_indent_and_overflow<S: l::LayoutScalar>() {
    for (indent, inset, start) in [(-10.0, 25.0, 15.0), (10.0, 15.0, 25.0)] {
        let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
        host.box_change(0, |root| {
            root.text_indent = l::TextIndentOf {
                amount: l::LengthPercentageOf::px(S::from_f64(indent)).unwrap(),
                each_line: true,
                hanging: false,
            }
        });
        let paragraph = Paragraph::pair(&mut host, 0);
        let batch = compute(&host, &[&paragraph], &[]);
        assert_eq!(
            glyph_extents(placed(&batch, &paragraph)),
            [(start, start + 60.0), (start, start + 40.0)]
        );
        for line in receipts(&batch, &paragraph) {
            let selected = line.selected_line();
            let context = &selected.request().context;
            assert_eq!(context.indent, indent);
            assert_eq!(
                (
                    context.alignment_insets.line_left(),
                    context.alignment_insets.line_right()
                ),
                (inset, inset)
            );
            assert_eq!(context.available_inline, Some(100.0));
        }
    }
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::Center);
    host.box_change(0, |root| root.text_align = l::TextAlign::Center);
    let paragraph = Paragraph::new(
        &mut host,
        0,
        &[Part::Text("AAAAAAAAAAAA"), Part::Forced, Part::Text("BBBB")],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let batch = compute(&host, &[&paragraph], &[]);
    assert_eq!(
        glyph_extents(placed(&batch, &paragraph)),
        [(0.0, 120.0), (30.0, 70.0)]
    );
    for line in receipts(&batch, &paragraph) {
        assert_eq!(
            line.selected_line().request().context.alignment_insets,
            t::LineAlignmentInsets::ZERO
        );
    }
}
#[test]
fn signed_indent_changes_only_real_usable_slack_and_negative_remainder_adds_no_group_padding() {
    signed_indent_and_overflow::<f32>();
    signed_indent_and_overflow::<f64>();
}
