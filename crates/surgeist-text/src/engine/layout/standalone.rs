// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Greedy standalone policy adapted from Parley 0.9 `layout/line_break.rs`.
//! Preliminary rows nominate only real eligible endpoints. Their actual fit is
//! measured by the selected-line finalizer. Progress is a mixed-item endpoint, including zero-source
//! atomic occurrences; no mutable paragraph or guessed convergence count exists.

use std::sync::Arc;

use super::super::{
    BreakKind, FinalizedLine, LineSelection, PreparedData, analysis::Boundary, shape::ShapeContext,
};
use super::{data::*, finalize_standalone, finite, placement};
use crate::{InlineBoxKind, OverflowWrap, Range, Result, TextNumericPhase, ValidatedOptions, Wrap};

pub(super) fn layout(
    prepared: Arc<PreparedData>,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<PlacedLayout> {
    let authored = options.authored();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut source_start = 0;
    let mut paragraph_start = true;
    loop {
        let scope = lines.is_empty() || (authored.indent.each_line && paragraph_start);
        let enabled =
            authored.indent.first_line || authored.indent.each_line || authored.indent.hanging;
        let indent = if enabled && (scope ^ authored.indent.hanging) {
            finite(
                "text indent",
                TextNumericPhase::Finalization,
                authored.indent.amount,
            )?
        } else {
            0.0
        };
        let line = choose(&prepared, start, source_start, indent, options, scratch)?;
        let end = line.selection.items.end;
        source_start = line.selection.source.end;
        paragraph_start = line.selection.break_kind == BreakKind::Explicit;
        let done = line.selection.break_kind == BreakKind::End;
        lines.push(line);
        if done {
            break;
        }
        start = end;
    }
    // An unconstrained paragraph still has one real shared alignment width.
    // Boundary selection does not depend on that later alignment choice. Re-run
    // the very same finalizer with complete context; never mutate prepared rows.
    if authored.width.is_none() {
        let width = lines
            .iter()
            .map(|line| {
                let metrics = line.finalized.data.metrics;
                (metrics.advance + line.selection.indent.max(0.0) - metrics.trailing_whitespace)
                    .max(0.0)
            })
            .fold(0.0, f64::max);
        let mut resolved = Vec::with_capacity(lines.len());
        for line in lines {
            let mut selection = line.selection;
            selection.available = Some(width);
            resolved.push(ChosenLine {
                finalized: finalize_standalone(
                    Arc::clone(&prepared),
                    selection.clone(),
                    authored.quantize,
                    scratch,
                )?,
                selection,
            });
        }
        lines = resolved;
    }
    placement::place(
        lines
            .into_iter()
            .map(|line| Arc::new(line.finalized))
            .collect(),
    )
}

/// Only actual break candidates are ever sent to the shaping finalizer.
/// Preliminary advances nominate the existing greedy endpoint; they never
/// supply committed geometry or establish that a selected line fits.
fn choose(
    prepared: &Arc<PreparedData>,
    start: usize,
    source_start: usize,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<ChosenLine> {
    let mut nomination = nominate(prepared, start, indent, options)?;
    if nomination.end == prepared.shaped.items.len() && nomination.kind != BreakKind::Explicit {
        nomination.kind = BreakKind::End;
    }
    loop {
        let selection = select(
            prepared,
            start,
            nomination.end,
            source_start,
            indent,
            nomination.kind,
            options,
        );
        let line = finalize_standalone(
            Arc::clone(prepared),
            selection.clone(),
            options.authored().quantize,
            scratch,
        )?;
        let fits = options.authored().width.is_none_or(|width| {
            line.data.metrics.advance - line.data.metrics.trailing_whitespace <= width - indent
        });
        if fits
            || nomination.unwrapped
            || initial_atomic_overflow(prepared, &line, nomination.initial_box)
        {
            return Ok(ChosenLine {
                finalized: line,
                selection,
            });
        }
        // These are source-eligible endpoints, not receipts for previously
        // shaped prefixes. Each retry strictly decreases the actual endpoint,
        // so no complete candidate/context is revisited and no pass cap exists.
        nomination.regular.retain(|end| *end < nomination.end);
        nomination.emergency.retain(|end| *end < nomination.end);
        if let Some(end) = nomination
            .regular
            .pop()
            .or_else(|| nomination.emergency.pop())
        {
            nomination.end = end;
            nomination.kind = BreakKind::Soft;
            nomination.initial_box = None;
        } else {
            // No earlier permitted boundary exists in this first fragment.
            // Existing standalone policy deliberately accepts its overflow.
            return Ok(ChosenLine {
                finalized: line,
                selection,
            });
        }
    }
}

/// Transient private chooser state; only the finalized selected owner is published.
struct ChosenLine {
    finalized: FinalizedLine,
    selection: LineSelection,
}

struct Nomination {
    end: usize,
    kind: BreakKind,
    regular: Vec<usize>,
    emergency: Vec<usize>,
    initial_box: Option<usize>,
    unwrapped: bool,
}

fn nominate(
    prepared: &PreparedData,
    start: usize,
    indent: f64,
    options: ValidatedOptions,
) -> Result<Nomination> {
    let items = &prepared.shaped.items;
    let mut result = Nomination {
        end: items.len(),
        kind: BreakKind::End,
        regular: Vec::new(),
        emergency: Vec::new(),
        initial_box: None,
        unwrapped: false,
    };
    if options.authored().width.is_none() {
        if let Some(offset) = items[start..].iter().position(|item| matches!(*item,
            ShapedItem::TextCluster(index) if prepared.shaped.clusters[index].whitespace==Whitespace::Newline)) {
            result.end=start+offset+1;
            result.kind=BreakKind::Explicit;
        }
        return Ok(result);
    }
    let available = finite(
        "standalone available span",
        TextNumericPhase::Finalization,
        options.authored().width.unwrap() - indent,
    )?;
    let mut cursor = start;
    let mut advance = 0.0;
    let mut previous_wrap = Wrap::Word;
    let mut any_text = false;
    let mut any_wrapped_text = false;
    while cursor < items.len() {
        let mut end = cursor + 1;
        match items[cursor] {
            ShapedItem::Anchor { .. } => {}
            ShapedItem::InlineBox { occurrence, .. } => {
                let item = prepared.analyzed.processed.source().boxes()[occurrence];
                let delta = if item.kind() == InlineBoxKind::InFlow {
                    item.size().width
                } else {
                    0.0
                };
                let next = finite(
                    "standalone preliminary atomic advance",
                    TextNumericPhase::Finalization,
                    advance + delta,
                )?;
                if next <= available || previous_wrap != Wrap::Word {
                    advance = next;
                    // Cached standalone policy permits a break after either
                    // box kind. OutOfFlow still contributes no width and is
                    // not an analysis/shaping barrier.
                    record(&mut result.regular, end, start);
                } else {
                    if advance == 0.0 {
                        result.end = end;
                        result.initial_box = Some(occurrence);
                    } else {
                        result.end = cursor;
                    }
                    result.kind = BreakKind::Soft;
                    return Ok(result);
                }
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &prepared.shaped.clusters[index];
                let style = &prepared.styles[cluster.style].style;
                let wrapping = previous_wrap == Wrap::Word;
                previous_wrap = style.wrap;
                any_text = true;
                any_wrapped_text |= style.wrap == Wrap::Word;
                if wrapping && !cluster.ligature_component && advance != 0.0 {
                    if cluster.boundary == Boundary::Line {
                        record(&mut result.regular, cursor, start);
                    } else if style.overflow_wrap != OverflowWrap::Normal {
                        record(&mut result.emergency, cursor, start);
                    }
                }
                if cluster.whitespace == Whitespace::Newline {
                    result.end = end;
                    result.kind = BreakKind::Explicit;
                    result.unwrapped = any_text && !any_wrapped_text;
                    return Ok(result);
                }
                let mut delta = cluster.advance;
                if cluster.ligature_start {
                    while end < items.len() {
                        let ShapedItem::TextCluster(next) = items[end] else {
                            break;
                        };
                        let next = &prepared.shaped.clusters[next];
                        if !next.ligature_component || next.run != cluster.run {
                            break;
                        }
                        delta = finite(
                            "standalone preliminary ligature advance",
                            TextNumericPhase::Finalization,
                            delta + next.advance,
                        )?;
                        end += 1;
                    }
                }
                let next = finite(
                    "standalone preliminary text advance",
                    TextNumericPhase::Finalization,
                    advance + delta,
                )?;
                if next <= available || !wrapping {
                    advance = next;
                } else if cluster.whitespace.is_space_or_nbsp() {
                    result.end = end;
                    result.kind = BreakKind::Soft;
                    return Ok(result);
                } else if let Some(&end) = result.regular.last().or_else(|| result.emergency.last())
                {
                    result.end = end;
                    result.kind = BreakKind::Soft;
                    return Ok(result);
                } else {
                    advance = next;
                }
            }
        }
        cursor = end;
    }
    result.unwrapped = any_text && !any_wrapped_text;
    Ok(result)
}

fn record(endpoints: &mut Vec<usize>, end: usize, start: usize) {
    if end > start && endpoints.last() != Some(&end) {
        endpoints.push(end);
    }
}

fn initial_atomic_overflow(
    prepared: &PreparedData,
    line: &FinalizedLine,
    occurrence: Option<usize>,
) -> bool {
    let Some(occurrence) = occurrence else {
        return false;
    };
    let text: f64 = line
        .data
        .clusters
        .iter()
        .map(|cluster| cluster.advance)
        .sum();
    let other_boxes: f64 = line
        .data
        .boxes
        .iter()
        .filter(|item| item.occurrence != occurrence)
        .filter(|item| {
            prepared.analyzed.processed.source().boxes()[item.occurrence].kind()
                == InlineBoxKind::InFlow
        })
        .map(|item| item.flow_advance())
        .sum();
    text + other_boxes == 0.0
}

fn select(
    prepared: &PreparedData,
    start: usize,
    end: usize,
    source_start: usize,
    indent: f64,
    kind: BreakKind,
    options: ValidatedOptions,
) -> LineSelection {
    let mut source_end = source_start;
    for item in &prepared.shaped.items[start..end] {
        match *item {
            ShapedItem::Anchor { .. } => {}
            ShapedItem::TextCluster(index) => {
                source_end = source_end.max(prepared.shaped.clusters[index].source.end);
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                source_end = source_end
                    .max(prepared.analyzed.processed.source().boxes()[occurrence].index());
            }
        }
    }
    if kind == BreakKind::End {
        source_end = prepared.analyzed.processed.source().text().len();
    }
    LineSelection {
        items: start..end,
        source: Range::new(source_start, source_end),
        break_kind: kind,
        available: options.authored().width,
        indent,
        alignment: options.authored().alignment,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Font, FontCatalogueMode, Source, Style, System, SystemOptions};

    fn system() -> (System, Style) {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let style = Style {
            font: Font::new().family("Ahem"),
            size: 20.0,
            ..Style::default()
        };
        (system, style)
    }

    #[test]
    fn chosen_lines_retain_one_preparation_and_actual_selected_geometry() {
        let (mut system, style) = system();
        let output = system
            .layout(
                Source::new("A X"),
                style,
                crate::Options {
                    width: Some(40.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            output
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 3)]
        );
        assert_eq!(output.metrics().size().width, 20.0);
        assert_eq!(output.metrics().full_width(), 40.0);
        for line in &output.inner.lines {
            assert!(Arc::ptr_eq(
                &output.inner.prepared,
                &line.finalized.selected.prepared
            ));
            assert_eq!(
                line.finalized.data.metrics.advance,
                line.finalized
                    .data
                    .clusters
                    .iter()
                    .map(|cluster| cluster.advance)
                    .sum::<f64>()
            );
        }
        assert_eq!(output.inner.lines[0].finalized.data.clusters[1].x, 20.0);
        assert_eq!(output.inner.lines[1].finalized.data.clusters[0].x, 0.0);
    }

    #[test]
    fn selected_line_l1_resets_original_trailing_whitespace_before_shared_l2() {
        let (mut system, style) = system();
        let output = system
            .layout(
                Source::new("X\u{202b}\u{05d0}  \u{202c}"),
                style,
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        let line = &output.inner.lines[0];
        assert_eq!(output.inner.prepared.analyzed.analysis.base_level, 0);
        for unit in [3, 4] {
            let cluster = line
                .finalized
                .data
                .clusters
                .iter()
                .find(|cluster| cluster.source.start == unit)
                .unwrap();
            assert_eq!(cluster.level, 0, "L1 trailing source whitespace");
        }
        assert_eq!(
            line.finalized
                .data
                .clusters
                .iter()
                .find(|cluster| cluster.source.start == 2)
                .unwrap()
                .level,
            1
        );
    }

    #[test]
    fn standalone_preserve_keeps_each_observed_separator_and_terminal_strut() {
        for delimiter in ["\r", "\n", "\u{2028}", "\u{2029}"] {
            let (mut system, style) = system();
            let source = Source::new(format!("A{delimiter}B"));
            let output = system
                .layout(
                    source,
                    style.clone(),
                    crate::Options {
                        quantize: false,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                output
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [Range::new(0, 2), Range::new(2, 3)]
            );
            assert_eq!(output.metrics().size(), crate::Size::new(20.0, 40.0));
            assert_eq!(output.metrics().first_baseline(), Some(16.0));
            assert_eq!(output.metrics().last_baseline(), Some(36.0));
            let terminal = system
                .layout(
                    Source::new(format!("A{delimiter}")),
                    style,
                    crate::Options {
                        quantize: false,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                terminal
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [Range::new(0, 2), Range::new(2, 2)]
            );
            assert_eq!(terminal.metrics().size(), crate::Size::new(20.0, 40.0));
            assert_eq!(terminal.metrics().last_baseline(), Some(36.0));
        }
    }

    #[test]
    fn standalone_preserve_keeps_characterized_crlf_breaks_without_fabricated_empty_glyph() {
        for (text, ranges, width) in [
            (
                "A\r\nB",
                vec![Range::new(0, 2), Range::new(2, 3), Range::new(3, 4)],
                20.0,
            ),
            (
                "A\r\n",
                vec![Range::new(0, 2), Range::new(2, 3), Range::new(3, 3)],
                20.0,
            ),
            (
                "\r\n",
                vec![Range::new(0, 1), Range::new(1, 2), Range::new(2, 2)],
                0.0,
            ),
        ] {
            let (mut system, style) = system();
            let output = system
                .layout(
                    Source::new(text),
                    style,
                    crate::Options {
                        quantize: false,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                output
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                ranges
            );
            assert_eq!(output.metrics().size(), crate::Size::new(width, 60.0));
            assert_eq!(
                output
                    .lines()
                    .iter()
                    .map(|line| line.baseline())
                    .collect::<Vec<_>>(),
                [16.0, 36.0, 56.0]
            );
        }
        let (mut system, style) = system();
        let empty = system
            .layout(
                Source::new(""),
                style,
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(empty.metrics().size(), crate::Size::new(0.0, 20.0));
        assert_eq!(empty.metrics().full_width(), 0.0);
        assert_eq!(empty.metrics().first_baseline(), Some(16.0));
        assert_eq!(empty.lines()[0].range(), Range::new(0, 0));
        assert!(
            empty
                .run_views()
                .all(|run| run.range() == Range::new(0, 0) && run.glyphs().next().is_none())
        );
    }

    #[test]
    fn constrained_long_words_keep_literal_geometry_for_normal_and_emergency_modes() {
        for mode in [OverflowWrap::Normal, OverflowWrap::Anywhere] {
            let (mut system, mut style) = system();
            style.overflow_wrap = mode;
            let output = system
                .layout(
                    Source::new("A".repeat(512)),
                    style,
                    crate::Options {
                        width: Some(10240.0),
                        quantize: false,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(output.lines().len(), 1);
            assert_eq!(output.lines()[0].range(), Range::new(0, 512));
            assert_eq!(output.metrics().size().width, 10240.0);
            assert!(!output.metrics().overflow());
            assert_eq!(
                output
                    .glyph_runs()
                    .iter()
                    .map(|run| run.glyphs().len())
                    .sum::<usize>(),
                512
            );
        }
        let (mut system, style) = system();
        let output = system
            .layout(
                Source::new("A".repeat(512)),
                style,
                crate::Options {
                    width: Some(1.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            output.lines().len(),
            1,
            "no ordinary break exists inside this word"
        );
        assert_eq!(output.lines()[0].range(), Range::new(0, 512));
        assert_eq!(output.metrics().full_width(), 10240.0);
        assert!(output.metrics().overflow());
    }

    fn words(values: &[u16]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_be_bytes())
            .collect()
    }

    // Real GPOS PairPos format1: A(35) before B(36) has xAdvance -500.
    // Ahem UPEM1000 and size20 therefore give the first glyph advance10.
    fn pair_adjusted_ahem() -> Vec<u8> {
        let mut gpos = words(&[1, 0, 10, 30, 44]);
        gpos.extend(words(&[1]));
        gpos.extend(*b"DFLT");
        gpos.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0]));
        gpos.extend(words(&[1]));
        gpos.extend(*b"kern");
        gpos.extend(words(&[8, 0, 1, 0]));
        gpos.extend(words(&[
            1,
            4,
            2,
            0,
            1,
            8,
            1,
            18,
            4,
            0,
            1,
            12,
            1,
            36,
            u16::from_be_bytes((-500_i16).to_be_bytes()),
            1,
            1,
            35,
        ]));
        crate::test_fonts::replace_tables(
            include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf"),
            vec![(*b"GPOS", gpos)],
        )
    }

    #[test]
    fn selected_boundary_feedback_uses_actual_pair_adjustment_and_an_earlier_real_candidate() {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(pair_adjusted_ahem()).unwrap();
        let style = Style {
            font: Font::new().family("Ahem"),
            size: 20.0,
            overflow_wrap: OverflowWrap::Anywhere,
            ..Style::default()
        };
        let output = system
            .layout(
                Source::new("AAB"),
                style,
                crate::Options {
                    width: Some(35.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        // Preliminary AA is 20+10, but an actual selected AA loses the
        // following B's pair adjustment and is40: it cannot occupy span35.
        assert_eq!(
            output
                .inner
                .prepared
                .shaped
                .glyphs
                .iter()
                .map(|glyph| glyph.advance)
                .collect::<Vec<_>>(),
            [20.0, 10.0, 20.0]
        );
        assert_eq!(
            output
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 3)]
        );
        assert_eq!(
            output
                .inner
                .lines
                .iter()
                .map(|line| line.finalized.data.metrics.advance)
                .collect::<Vec<_>>(),
            [20.0, 30.0]
        );
        assert_eq!(
            output
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.advance()))
                .collect::<Vec<_>>(),
            [20.0, 10.0, 20.0]
        );
    }

    #[test]
    fn mixed_atomic_occurrences_and_overflowing_space_keep_the_existing_break_priority() {
        let (mut system, style) = system();
        let mut source = Source::new("A X");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(1),
            InlineBoxKind::InFlow,
            0,
            crate::Size::new(20.0, 20.0),
        ));
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(2),
            InlineBoxKind::OutOfFlow,
            1,
            crate::Size::new(50.0, 50.0),
        ));
        let output = system
            .layout(
                source,
                style,
                crate::Options {
                    width: Some(40.0),
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            output
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 3)]
        );
        assert_eq!(output.metrics().size(), crate::Size::new(40.0, 40.0));
        assert_eq!(output.metrics().full_width(), 60.0);
        assert_eq!(
            output
                .inline_boxes()
                .iter()
                .map(|item| (item.id(), item.line()))
                .collect::<Vec<_>>(),
            [(crate::Id::from_u64(1), 0), (crate::Id::from_u64(2), 0)]
        );
    }

    #[test]
    fn terminal_oversized_box_ends_without_fake_empty_line_or_font_requirement() {
        for count in [1_u64, 128] {
            let mut system = System::new(SystemOptions {
                font_catalogue: FontCatalogueMode::RegisteredOnly,
            })
            .unwrap();
            let mut source = Source::new("");
            for index in 0..count {
                source.inline_box(crate::InlineBox::new(
                    crate::Id::from_u64(index + 1),
                    InlineBoxKind::InFlow,
                    0,
                    crate::Size::new(12.0, 10.0),
                ));
            }
            let output = system
                .layout(
                    source,
                    Style::default(),
                    crate::Options {
                        width: Some(8.0),
                        quantize: false,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(output.lines().len(), usize::try_from(count).unwrap());
            assert_eq!(
                output.metrics().size(),
                crate::Size::new(12.0, 10.0 * count as f64)
            );
            assert!(
                output
                    .lines()
                    .iter()
                    .all(|line| line.range() == Range::new(0, 0))
            );
            assert!(output.run_views().next().is_none());
            assert!(output.clusters().is_empty());
            assert!(output.metrics().overflow());
            assert_eq!(output.inline_boxes().len(), usize::try_from(count).unwrap());
            assert_eq!(
                output.inline_boxes().last().unwrap().line(),
                usize::try_from(count - 1).unwrap()
            );
        }
    }
}
