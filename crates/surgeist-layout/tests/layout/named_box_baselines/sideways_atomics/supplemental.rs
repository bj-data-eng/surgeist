//! Independent bidi, allocation-domain and physical box qualification.
use super::*;
mod geometry;
mod late;

fn length<S: l::LayoutScalar>(value: f64) -> l::LengthOf<S> {
    l::LengthOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn margin<S: l::LayoutScalar>(value: f64) -> l::LengthAutoOf<S> {
    l::LengthAutoOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}
pub(super) fn rtl_font() -> Vec<u8> {
    // Licensed Ahem outlines/metrics are retained. This cmap-only derivative
    // maps Hebrew Alef/Bet to original A/B, a genuine odd-level bidi fixture,
    // without claiming that these Latin outlines are Hebrew letter designs.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([0, 12, 0, 0]);
    for word in [40_u32, 0, 2, 65, 66, 35, 1488, 1489, 35] {
        cmap.extend(word.to_be_bytes());
    }
    let count = usize::from(u16::from_be_bytes(AHEM[4..6].try_into().unwrap()));
    let mut bytes = vec![0; 12 + 16 * count];
    bytes[..12].copy_from_slice(&AHEM[..12]);
    for index in 0..count {
        let at = 12 + 16 * index;
        let tag: [u8; 4] = AHEM[at..at + 4].try_into().unwrap();
        let mut data = if tag == *b"cmap" {
            cmap.clone()
        } else {
            table(AHEM, tag).to_vec()
        };
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let start = u32::try_from(bytes.len()).unwrap();
        bytes[at..at + 4].copy_from_slice(&tag);
        bytes[at + 4..at + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        bytes[at + 8..at + 12].copy_from_slice(&start.to_be_bytes());
        bytes[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend(data);
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    let at = (0..count)
        .map(|i| 12 + 16 * i)
        .find(|at| bytes[*at..*at + 4] == *b"head")
        .unwrap();
    let head = usize::try_from(u32::from_be_bytes(
        bytes[at + 8..at + 12].try_into().unwrap(),
    ))
    .unwrap();
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    for tag in [*b"glyf", *b"loca", *b"hmtx", *b"hhea", *b"maxp"] {
        assert_eq!(table(&bytes, tag), table(AHEM, tag));
    }
    let cmap = table(&bytes, *b"cmap");
    assert_eq!(u16::from_be_bytes(cmap[12..14].try_into().unwrap()), 12);
    assert_eq!(u32::from_be_bytes(cmap[24..28].try_into().unwrap()), 2);
    assert_eq!(
        [40, 44, 48].map(|at| u32::from_be_bytes(cmap[at..at + 4].try_into().unwrap())),
        [1488, 1489, 35]
    );
    qualify_font(&bytes);
    bytes
}

fn profile_line<S: l::LayoutScalar>(
    parent: &Captured,
    kind: t::InlineBoxKind,
    align: l::VerticalAlignOf<S>,
    containing: l::Direction,
    text_align: t::Alignment,
) -> t::FinalizedLine {
    let p = &parent.prepared;
    let rtl = p.analyzed_source().direction() == t::Direction::RightToLeft;
    let end = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let mut atomic = allocation(parent, kind);
    if let t::AtomicLineAllocation::InFlow { alignment, .. } = &mut atomic.allocation {
        *alignment = match align {
            l::VerticalAlignOf::Baseline => t::LineBoxAlignment::Baseline,
            l::VerticalAlignOf::Top => t::LineBoxAlignment::Top,
            l::VerticalAlignOf::Bottom => t::LineBoxAlignment::Bottom,
            l::VerticalAlignOf::Shift(offset) => t::LineBoxAlignment::Shift(
                t::BaselineShift::try_new(offset.get().to_f64())
                    .expect("a finite Layout shift widens to finite Text units"),
            ),
        };
    }
    p.select(t::LineSelectionRequest {
        inline_boxes: None,
        segment: p.source_segments().next().unwrap(),
        start: p.boundary_views().next().unwrap().handle(),
        end: end.view().boundary(),
        termination: t::PreparedTermination::Ordinary(end),
        participants: p.participants().collect(),
        alternative: None,
        incoming: None,
        atomics: vec![atomic],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: t::LineContext {
            alignment_insets: t::LineAlignmentInsets::ZERO,
            formatted_line: t::FormattedLineContext::default(),
            base_direction: if rtl {
                t::ParagraphBaseDirection::RightToLeft
            } else {
                t::ParagraphBaseDirection::LeftToRight
            },
            line_box_direction: if containing == l::Direction::Ltr {
                t::LineBoxDirection::LeftToRight
            } else {
                t::LineBoxDirection::RightToLeft
            },
            orientation: t::LineOrientation::SidewaysClockwise,
            start: t::LineStart::Paragraph,
            end: t::LineEnd::Paragraph,
            available_inline: Some(40.0),
            indent: 0.0,
            alignment: text_align,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::None,
        },
    })
    .unwrap()
    .finalize(surgeist_text::LineAllocations {
        spacing: t::SpacingAllocations::None,
        scopes: surgeist_text::ScopeAllocations::None,
    })
    .unwrap()
}

fn matrix<S: l::LayoutScalar>() {
    let bytes = rtl_font();
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let advance = if kind == t::InlineBoxKind::InFlow {
            34.0
        } else {
            20.0
        };
        for body in [
            t::ParagraphBaseDirection::LeftToRight,
            t::ParagraphBaseDirection::RightToLeft,
        ] {
            let raw = if body == t::ParagraphBaseDirection::LeftToRight {
                "AB"
            } else {
                "\u{5d0}\u{5d1}"
            };
            let parent = capture(
                &bytes,
                raw,
                Some((kind, t::Id::from_u64(2))),
                body,
                t::LineOrientation::SidewaysClockwise,
                10.0,
            );
            let level = if body == t::ParagraphBaseDirection::LeftToRight {
                0
            } else {
                1
            };
            assert!(
                parent
                    .prepared
                    .unit_views()
                    .all(|unit| unit.level().get() == level)
            );
            for direction in [l::Direction::Ltr, l::Direction::Rtl] {
                // An all-auto positioned box anchors its inline-start margin
                // edge. In RTL that is the bottom edge, not the physical top.
                let positioned_extent =
                    if kind == t::InlineBoxKind::OutOfFlow && direction == l::Direction::Rtl {
                        14.0
                    } else {
                        0.0
                    };
                for (align, in_flow_x) in [
                    (l::VerticalAlignOf::Baseline, 32.0),
                    (l::VerticalAlignOf::Top, 34.0),
                    (l::VerticalAlignOf::Bottom, 30.0),
                ] {
                    let qualified =
                        profile_line::<S>(&parent, kind, align, direction, t::Alignment::Start);
                    assert_eq!(
                        (
                            qualified.feedback().advance,
                            qualified.feedback().baseline,
                            qualified.feedback().line_height
                        ),
                        (advance, 8.0, 10.0)
                    );
                    let (mut host, bindings) = scene::<S>(&parent, &child, kind, direction, 40.0);
                    host.change(2, |input| input.vertical_align = align);
                    let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
                    let shift = if direction == l::Direction::Ltr {
                        0.0
                    } else {
                        40.0 - advance
                    };
                    let x = if kind == t::InlineBoxKind::OutOfFlow {
                        34.0
                    } else {
                        in_flow_x
                    };
                    let box_y = shift + 10.0 - positioned_extent;
                    let border = t::Rect::new(x, box_y, 6.0, 14.0);
                    let text = placed(&batch, 0);
                    assert_eq!(text.inline_boxes()[0].rect(), border);
                    let actual = receipt(&batch, 1);
                    let expected = explicit(
                        &parent,
                        actual,
                        t::LineAnchor {
                            inline_origin: 0.0,
                            baseline: 32.0,
                        },
                        t::AtomicPlacement {
                            occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                            border,
                            content: border,
                            baseline: x,
                        },
                    );
                    assert_eq!(text.key(), expected.key());
                    let ids = if level == 0 { [35, 36] } else { [36, 35] };
                    let second = if kind == t::InlineBoxKind::InFlow {
                        24.0
                    } else {
                        10.0
                    };
                    let glyphs = text
                        .run_views()
                        .flat_map(|run| {
                            assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
                            run.glyphs()
                                .map(|g| (g.id(), run.origin().y + g.pen()))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(glyphs, [(ids[0], shift), (ids[1], shift + second)]);
                    assert_eq!(text.source(), &parent.source);
                    unknown(node(batch.unrounded_entries(), 0).named_baselines);
                    assert_eq!(placed(&batch, 2).lines()[0].baseline(), box_y + 4.0);
                    assert_eq!(
                        text.hit(t::Point::new(x + 1.0, box_y + 2.0)),
                        t::Hit::InlineBox(t::Id::from_u64(2))
                    );
                }
                for (alignment, residual) in [
                    (l::TextAlign::Center, (40.0 - advance) / 2.0),
                    (
                        l::TextAlign::End,
                        if direction == l::Direction::Ltr {
                            40.0 - advance
                        } else {
                            0.0
                        },
                    ),
                ] {
                    let (mut host, bindings) = scene::<S>(&parent, &child, kind, direction, 40.0);
                    host.change(0, |input| {
                        input.text_align = alignment;
                        // The shared baseline fixture pins the last line to
                        // Start; this case requests ordinary Auto inheritance.
                        input.text_align_last = l::TextAlignLast::Auto;
                    });
                    let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
                    let x = if kind == t::InlineBoxKind::InFlow {
                        32.0
                    } else {
                        34.0
                    };
                    assert_eq!(
                        placed(&batch, 0).inline_boxes()[0].rect(),
                        t::Rect::new(x, residual + 10.0 - positioned_extent, 6.0, 14.0)
                    );
                }
            }
        }
    }
}

fn guards<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(2))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        qualify_parent(&parent, kind);
        for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
            let (mut host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
            host.change(0, |input| {
                input.inline_baseline_context = l::InlineBaselineContext::Named(name)
            });
            let error = run(&host, bindings, &[], available(40.0, 40.0)).unwrap_err();
            assert!(matches!(
                error.kind(),
                l::LayoutErrorKindOf::MissingContext(l::LayoutMissingContext::NamedInlineBaseline)
            ));
        }
        let (mut host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
        host.change(0, |input| input.writing_mode = l::WritingMode::SidewaysLr);
        let error = run(&host, bindings, &[], available(40.0, 40.0)).unwrap_err();
        assert!(matches!(
            error.kind(),
            l::LayoutErrorKindOf::UnsupportedCapability(
                l::LayoutUnsupportedCapability::TextWritingMode(l::WritingMode::SidewaysLr)
            )
        ));
    }
    let mut box_root = root(
        l::WritingMode::SidewaysLr,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    box_root.size = l::Size::new(px(40.0), px(40.0));
    let mut host = Host::<S>::new(box_root);
    let mut input = child_input(t::InlineBoxKind::InFlow);
    input.display = l::Display::Block;
    host.push(Some(0), l::LayoutInputOf::box_input(input));
    l::compute_layout(
        &host,
        0,
        l::LayoutRootRequestOf::viewport(available(40.0, 40.0)).unwrap(),
    )
    .unwrap();
}

#[test]
fn physical_clockwise_atoms_keep_bidi_containing_direction_and_current_alignment_domain() {
    matrix::<f32>();
    matrix::<f64>();
}
#[test]
fn physical_clockwise_atoms_keep_named_authority_and_counterclockwise_text_refusals() {
    guards::<f32>();
    guards::<f64>();
}
