// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The raw chooser retains its own ordinary/emergency policy and metric profile.
//! Every candidate uses the same selected-line finalizer and genuine mixed cuts.
use super::super::{FinalizedLine, PreparedData, analysis::Boundary, shape::ShapeContext};
use super::{data::*, finalize_standalone, placement};
use crate::protocol::prepared::AlternativeDefinition;
use crate::protocol::selection::standalone_request;
use crate::protocol::{
    PreparedAlternative, PreparedBreakKind, PreparedOpportunity, PreparedTermination,
};
use crate::{Result, ValidatedOptions, Wrap};
use std::sync::Arc;

pub(super) fn layout(
    prepared: Arc<PreparedData>,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<PlacedLayout> {
    let authored = options.authored();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut paragraph_start = true;
    loop {
        let scope = lines.is_empty() || (authored.indent.each_line && paragraph_start);
        let enabled =
            authored.indent.first_line || authored.indent.each_line || authored.indent.hanging;
        let indent = if enabled && (scope ^ authored.indent.hanging) {
            authored.indent.amount
        } else {
            0.0
        };
        let line = choose(&prepared, start, indent, options, scratch)?;
        let request = &line.selected.request;
        start = request.end.index;
        paragraph_start = request.context.end == crate::LineEnd::Forced;
        let done = request.context.end == crate::LineEnd::Paragraph;
        lines.push(line);
        if done {
            break;
        }
    }
    if authored.width.is_none() {
        let width = lines
            .iter()
            .map(|line| {
                (line.data.metrics.advance + line.selected.request.context.indent.max(0.0)
                    - line.data.metrics.trailing_whitespace)
                    .max(0.0)
            })
            .fold(0.0, f64::max);
        let mut resolved = Vec::with_capacity(lines.len());
        for line in lines {
            let mut request = line.selected.request.clone();
            request.context.available_inline = Some(width);
            resolved.push(finalize_standalone(
                Arc::clone(&prepared),
                request,
                authored.quantize,
                scratch,
            )?);
        }
        lines = resolved;
    }
    placement::place(lines.into_iter().map(Arc::new).collect())
}

fn fits(line: &FinalizedLine, width: Option<f64>) -> Result<bool> {
    let Some(width) = width else { return Ok(true) };
    let available = super::finite(
        "raw available span",
        crate::TextNumericPhase::Finalization,
        width - line.selected.request.context.indent,
    )?;
    let measure = line
        .data
        .measurements
        .as_ref()
        .expect("finalized raw measures")
        .alignment();
    Ok(measure.advance() <= available && raw_occupied(line)? <= available)
}

fn raw_occupied(line: &FinalizedLine) -> Result<f64> {
    let mut start = line.data.metrics.offset;
    let mut end = start;
    for row in &line.data.clusters {
        if matches!(
            row.disposition,
            crate::SpacingSlotDisposition::Retained { trailing: true }
        ) {
            continue;
        }
        start = start.min(row.x.min(row.x + row.advance));
        end = end.max(row.x.max(row.x + row.advance));
    }
    for row in &line.data.boxes {
        if let FinalizedBoxGeometry::InFlow { flow, .. } = row.geometry {
            start = start.min(row.slot.x + flow.occupied_start);
            end = end.max(row.slot.x + flow.occupied_end);
        }
    }
    super::finite(
        "raw occupied span",
        crate::TextNumericPhase::Finalization,
        end - start,
    )
}

fn trial(
    prepared: &Arc<PreparedData>,
    start: usize,
    termination: PreparedTermination,
    alternative: Option<PreparedAlternative>,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<FinalizedLine> {
    let request = standalone_request(
        Arc::clone(prepared),
        start,
        termination,
        alternative,
        options.authored().width,
        indent,
        options.authored().alignment,
    )?;
    finalize_standalone(
        Arc::clone(prepared),
        request,
        options.authored().quantize,
        scratch,
    )
}

fn automatic_allowed(
    prepared: &Arc<PreparedData>,
    alternative: usize,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<bool> {
    let AlternativeDefinition::Automatic { cut, .. } =
        prepared.tables.alternatives[alternative].definition
    else {
        return Ok(true);
    };
    let word = prepared.tables.hyphenation_cuts[cut]
        .word
        .expect("automatic cut has a lexical owner");
    let handle = crate::PreparedWord {
        prepared: Arc::clone(prepared),
        index: word,
    };
    for portion in handle.view().portions()? {
        if !portion.has_manual_partition()
            || prepared.tables.hyphenation_cuts[cut].boundary <= portion.start
            || prepared.tables.hyphenation_cuts[cut].boundary > portion.end
        {
            continue;
        }
        let line = trial(
            prepared,
            portion.start,
            PreparedTermination::Measurement(portion.clone()),
            portion.alternative(),
            indent,
            options,
            scratch,
        )?;
        if fits(&line, options.authored().width)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn choose(
    prepared: &Arc<PreparedData>,
    start: usize,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<FinalizedLine> {
    let tables = &prepared.tables;
    let mut ordinary = None;
    let mut first = None;
    let mut emergency = None;
    let mut emergency_first = None;
    for (index, offer) in tables.opportunities.iter().enumerate() {
        let boundary = tables.boundaries[offer.boundary];
        if boundary.participant < tables.boundaries[start].participant
            || (boundary.participant == tables.boundaries[start].participant
                && offer.kind != PreparedBreakKind::ParagraphEnd)
        {
            continue;
        }
        let mandatory = matches!(
            offer.kind,
            PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
        );
        if !mandatory
            && (boundary.participant == tables.participants.len()
                || !tables.participants[tables.boundaries[start].participant..boundary.participant]
                    .iter()
                    .any(|participant| match *participant {
                        crate::protocol::prepared::ParticipantData::Text(i) => {
                            !tables.units[i].atoms.is_empty()
                        }
                        crate::protocol::prepared::ParticipantData::Atomic(i) => {
                            prepared.analyzed.processed.source().boxes()
                                [tables.atomics[i].occurrence]
                                .kind()
                                == crate::InlineBoxKind::InFlow
                        }
                        crate::protocol::prepared::ParticipantData::Control(_) => false,
                    }))
        {
            continue;
        }
        if options.authored().width.is_none() && !mandatory {
            continue;
        }
        let previous = boundary.atom.checked_sub(1);
        let wrapping = previous.is_none_or(|atom| {
            prepared.analyzed.styles[prepared.analyzed.analysis.infos[atom].1]
                .style
                .authored()
                .wrap
                == Wrap::Word
        });
        if !wrapping && !mandatory {
            continue;
        }
        let atomic = tables
            .participants
            .get(boundary.participant)
            .is_some_and(|row| {
                matches!(row, crate::protocol::prepared::ParticipantData::Atomic(_))
            })
            || boundary.participant.checked_sub(1).is_some_and(|i| {
                matches!(
                    tables.participants[i],
                    crate::protocol::prepared::ParticipantData::Atomic(_)
                )
            });
        let is_emergency = offer.kind == PreparedBreakKind::Allowed
            && offer.alternatives.is_empty()
            && !atomic
            && boundary.atom < prepared.analyzed.analysis.infos.len()
            && prepared.analyzed.analysis.infos[boundary.atom].0.boundary < Boundary::Line;
        let variants = (offer.kind != PreparedBreakKind::Discretionary)
            .then_some(None)
            .into_iter()
            .chain(offer.alternatives.iter().copied().map(Some));
        for variant in variants {
            if let Some(alternative) = variant
                && !automatic_allowed(prepared, alternative, indent, options, scratch)?
            {
                continue;
            }
            let line = trial(
                prepared,
                start,
                PreparedTermination::Ordinary(PreparedOpportunity {
                    prepared: Arc::clone(prepared),
                    index,
                }),
                variant.map(|index| PreparedAlternative {
                    prepared: Arc::clone(prepared),
                    index,
                }),
                indent,
                options,
                scratch,
            )?;
            let actual_fit = fits(&line, options.authored().width)?;
            if is_emergency {
                if actual_fit {
                    emergency = Some(line);
                } else if emergency_first.is_none() {
                    emergency_first = Some(line);
                }
            } else {
                if actual_fit {
                    ordinary = Some(line);
                } else if first.is_none() {
                    first = Some(line);
                }
            }
            if actual_fit {
                break;
            }
        }
        if mandatory {
            break;
        }
    }
    ordinary
        .or(emergency)
        .or(emergency_first)
        .or(first)
        .ok_or_else(|| {
            crate::Error::new(
                crate::ErrorCode::LayoutFailed,
                "raw inventory has no terminating candidate",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Font, FontCatalogueMode, InlineBoxKind, OverflowWrap, Range, Source, Style, System,
        SystemOptions,
    };

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
