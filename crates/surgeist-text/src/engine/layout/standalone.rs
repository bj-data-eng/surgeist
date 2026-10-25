// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The raw chooser retains its own ordinary/emergency policy and metric profile.
//! Every candidate uses the same selected-line finalizer and genuine mixed cuts.
use super::super::{FinalizedLine, PreparedData, shape::ShapeContext};
use super::{data::*, finalize_standalone, placement};
use crate::protocol::prepared::AlternativeDefinition;
use crate::protocol::selection::standalone_request;
use crate::protocol::{
    PreparedAlternative, PreparedBreakKind, PreparedOpportunity, PreparedTermination,
};
use crate::{Result, ValidatedOptions, Wrap};
use std::sync::Arc;

/// Raw emitted history stays invocation-local. Indentation's each-line state is
/// separate from this actual first-formatted-line observation.
struct RawAcceptedLine {
    line: FinalizedLine,
    first: bool,
}

#[derive(Clone, Copy)]
pub(super) struct RawEntry<'a> {
    start: usize,
    incoming: Option<&'a crate::HyphenationContinuation>,
    accepted: &'a [RawAcceptedLine],
}

pub(super) enum RawRoleBasis<'a> {
    Actual { first: bool },
    Prospective { entry: RawEntry<'a> },
}

impl RawRoleBasis<'_> {
    pub(super) fn initial(&self, end: crate::LineEnd) -> crate::FormattedLineContext {
        match self {
            Self::Actual { first } => crate::FormattedLineContext::new(
                Some(*first),
                Some(end == crate::LineEnd::Paragraph),
            ),
            // A measurement can describe a receiving line even while its query
            // is on the actual first line. Never copy the query's role flags.
            Self::Prospective { .. } => crate::FormattedLineContext::default(),
        }
    }

    pub(super) fn resolve(
        &self,
        prepared: &Arc<PreparedData>,
        request: &crate::LineSelectionRequest,
        profile: crate::protocol::selection::LineMetricPolicy,
        scratch: &mut ShapeContext,
    ) -> Result<crate::FormattedLineContext> {
        use crate::{FlowSegmentRole, LineEnd};
        let Self::Prospective { entry } = self else {
            return Ok(self.initial(request.context.end));
        };
        let target = &request.termination;
        if !matches!(
            target,
            PreparedTermination::Measurement(_) | PreparedTermination::PhraseMeasurement(_)
        ) {
            return Err(invalid_raw_flow());
        }
        let handle = crate::PreparedText {
            inner: Arc::clone(prepared),
        };
        let query_start = crate::PreparedBoundary {
            prepared: Arc::clone(prepared),
            index: entry.start,
        };
        let flow = handle.measurement_flow(&query_start, entry.incoming, target)?;
        let phrase = matches!(target, PreparedTermination::PhraseMeasurement(_));
        let mut first = if phrase {
            entry.accepted.is_empty()
        } else {
            manual_entry_first(prepared, *entry, &flow.entry())?
        };
        let mut observations = Vec::new();
        for segment in flow.segments() {
            let mut context = request.context.clone();
            context.formatted_line = crate::FormattedLineContext::default();
            context.start = if segment.start().view().participant_offset() == 0 {
                crate::LineStart::Paragraph
            } else {
                crate::LineStart::Continuation
            };
            context.end = if segment.role() == FlowSegmentRole::Target {
                LineEnd::Soft
            } else {
                segment
                    .mandatory_end()
                    .map_or(LineEnd::Soft, |offer| match offer.view().kind() {
                        PreparedBreakKind::Forced => LineEnd::Forced,
                        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                        _ => LineEnd::Soft,
                    })
            };
            let observation_request = crate::FlowObservationRequest {
                context,
                atomics: crate::protocol::selection::standalone_atomics(
                    prepared,
                    segment.start().index,
                    segment.end().index,
                )?,
                controls: Vec::new(),
                line_strut: None,
                boundary_context: Vec::new(),
            };
            observations.push(super::flow::observe(
                &handle,
                &segment,
                &observation_request,
                profile,
                scratch,
            )?);
        }
        flow.validate_observations(&observations)?;
        let mut later_line = false;
        for observation in &observations {
            let segment = observation.segment();
            let forced = segment
                .mandatory_end()
                .is_some_and(|offer| offer.view().kind() == PreparedBreakKind::Forced);
            match segment.role() {
                FlowSegmentRole::EntryPrefix => {
                    let preceding = observation.has_retained_flow_participation() || forced;
                    if phrase && preceding {
                        return Err(invalid_raw_flow());
                    }
                    first &= !preceding;
                }
                FlowSegmentRole::PriorPartition => {
                    // This is Raw's admission + emitted-line contract, not an
                    // assertion that Native retained participation emits a line.
                    let admitted = segment.mandatory_end().is_some()
                        || raw_soft_admitted(prepared, segment.start().index, segment.end().index);
                    first &= !admitted;
                }
                FlowSegmentRole::Target => {
                    // A genuine manual cut schedules the receiving Paragraph
                    // even if its indication is Before-only/empty at body end.
                    later_line |=
                        segment.alternative().is_some() || segment.outgoing().is_some() || forced;
                }
                FlowSegmentRole::Remaining => {
                    // Neutral trailing custody can close this hypothetical
                    // unbroken line. A real break or receiving prefix cannot.
                    later_line |= observation.has_retained_flow_participation()
                        || forced
                        || segment.incoming().is_some();
                }
            }
        }
        // The raw call owns the complete single container. Its recipe completion
        // has no external child tail; actual trailing Forced/empty work above is
        // still required and never erased by a Soft measurement API ending.
        Ok(crate::FormattedLineContext::new(
            Some(first),
            Some(!later_line),
        ))
    }
}

fn invalid_raw_flow() -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::InvalidMeasurementFlow)
}

fn manual_entry_first(
    prepared: &Arc<PreparedData>,
    entry: RawEntry<'_>,
    word_entry: &crate::PreparedBoundary,
) -> Result<bool> {
    let at = word_entry.view().participant_offset();
    for row in entry.accepted {
        let request = &row.line.selected.request;
        let from = request.start.view().participant_offset();
        let to = request.end.view().participant_offset();
        if from <= at && at < to {
            if !row.first {
                return Ok(false);
            }
            if from == at {
                return Ok(true);
            }
            let participant = crate::PreparedText {
                inner: Arc::clone(prepared),
            }
            .participants()
            .nth(at)
            .ok_or_else(invalid_raw_flow)?;
            let line = receipt(&row.line);
            return Ok(!line
                .prefix_at(&crate::TextSeam::BeforeParticipant(participant))?
                .has_retained_flow_participation());
        }
    }
    if at < prepared.tables.boundaries[entry.start].participant {
        // A backwards whole-word hypothesis needs the actual containing prefix
        // receipt. Its absence is not permission to guess first/nonfirst.
        return Err(invalid_raw_flow());
    }
    Ok(entry.accepted.is_empty())
}

pub(super) fn layout(
    prepared: Arc<PreparedData>,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<PlacedLayout> {
    let authored = options.authored();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut paragraph_start = true;
    let mut incoming = None;
    loop {
        let scope = lines.is_empty() || (authored.indent.each_line && paragraph_start);
        let enabled =
            authored.indent.first_line || authored.indent.each_line || authored.indent.hanging;
        let indent = if enabled && (scope ^ authored.indent.hanging) {
            authored.indent.amount
        } else {
            0.0
        };
        let line = choose(
            &prepared,
            RawEntry {
                start,
                incoming: incoming.as_ref(),
                accepted: &lines,
            },
            indent,
            options,
            scratch,
        )?;
        let request = &line.selected.request;
        incoming = request
            .alternative
            .clone()
            .and_then(crate::HyphenationContinuation::from_alternative);
        start = request.end.index;
        paragraph_start = request.context.end == crate::LineEnd::Forced;
        let done = request.context.end == crate::LineEnd::Paragraph;
        let first = lines.is_empty();
        lines.push(RawAcceptedLine { line, first });
        if done {
            break;
        }
    }
    if authored.width.is_none() {
        let width = lines
            .iter()
            .map(|row| {
                (row.line
                    .data
                    .measurements
                    .as_ref()
                    .expect("finalized raw measures")
                    .max_content_contribution()
                    .advance()
                    + row.line.selected.request.context.indent.max(0.0))
                .max(0.0)
            })
            .fold(0.0, f64::max);
        let mut resolved = Vec::with_capacity(lines.len());
        for row in lines {
            let mut request = row.line.selected.request.clone();
            request.context.available_inline = Some(width);
            resolved.push(finalize_standalone(
                Arc::clone(&prepared),
                request,
                authored.quantize,
                scratch,
                &mut RawRoleBasis::Actual { first: row.first },
            )?);
        }
        return placement::place(resolved.into_iter().map(Arc::new).collect());
    }
    placement::place(lines.into_iter().map(|row| Arc::new(row.line)).collect())
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
        .fit();
    let occupied = super::finite(
        "raw occupied span",
        crate::TextNumericPhase::Finalization,
        measure.occupied_end() - measure.occupied_start(),
    )?;
    Ok(measure.advance() <= available && occupied <= available)
}

/// Raw's actual nonmandatory admission is intentionally distinct from Native's
/// post-discard Soft participation rule.
fn raw_soft_admitted(prepared: &PreparedData, start: usize, end: usize) -> bool {
    use crate::protocol::prepared::ParticipantData;
    let tables = &prepared.tables;
    let from = tables.boundaries[start].participant;
    let to = tables.boundaries[end].participant;
    to != tables.participants.len()
        && tables.participants[from..to]
            .iter()
            .any(|participant| match *participant {
                ParticipantData::Text(i) => !tables.units[i].atoms.is_empty(),
                ParticipantData::Atomic(i) => {
                    prepared.analyzed.processed.source().boxes()[tables.atomics[i].occurrence]
                        .kind()
                        == crate::InlineBoxKind::InFlow
                }
                ParticipantData::Control(_) => false,
            })
}

fn automatic_allowed(
    prepared: &Arc<PreparedData>,
    alternative: usize,
    entry: RawEntry<'_>,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
    conditional: Option<&crate::PreparedConditionalOpportunity>,
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
    let portions = if let Some(conditional) = conditional {
        conditional.manual_portions()?
    } else {
        handle.view().portions()?
    };
    for portion in portions {
        if !portion.has_manual_partition()
            || prepared.tables.hyphenation_cuts[cut].boundary <= portion.start
            || prepared.tables.hyphenation_cuts[cut].boundary > portion.end
        {
            continue;
        }
        let mut request = standalone_request(
            Arc::clone(prepared),
            portion.start,
            PreparedTermination::Measurement(portion.clone()),
            portion.alternative(),
            options.authored().width,
            indent,
            options.authored().alignment,
        )?;
        request.incoming = portion.incoming();
        let line = finalize_standalone(
            Arc::clone(prepared),
            request,
            options.authored().quantize,
            scratch,
            &mut RawRoleBasis::Prospective { entry },
        )?;
        if fits(&line, options.authored().width)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn choose(
    prepared: &Arc<PreparedData>,
    entry: RawEntry<'_>,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<FinalizedLine> {
    let RawEntry {
        start, incoming, ..
    } = entry;
    let tables = &prepared.tables;
    let mut ordinary = None;
    let mut first = None;
    let mut emergency = None;
    let mut emergency_first = None;
    for (index, offer) in
        tables
            .opportunities
            .iter()
            .enumerate()
            .skip(tables.opportunities.partition_point(|row| {
                tables.boundaries[row.boundary].participant < tables.boundaries[start].participant
            }))
    {
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
        if !mandatory && !raw_soft_admitted(prepared, start, offer.boundary) {
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
            && !prepared.analyzed.analysis.infos[boundary.atom]
                .0
                .has_ordinary_line_candidate();
        let variants = (offer.kind != PreparedBreakKind::Discretionary)
            .then_some(None)
            .into_iter()
            .chain(offer.alternatives.iter().copied().map(Some));
        for variant in variants {
            if let Some(alternative) = variant
                && !automatic_allowed(prepared, alternative, entry, indent, options, scratch, None)?
            {
                continue;
            }
            let mut request = standalone_request(
                Arc::clone(prepared),
                start,
                PreparedTermination::Ordinary(PreparedOpportunity {
                    prepared: Arc::clone(prepared),
                    index,
                }),
                variant.map(|index| PreparedAlternative {
                    prepared: Arc::clone(prepared),
                    index,
                }),
                options.authored().width,
                indent,
                options.authored().alignment,
            )?;
            request.incoming = incoming.cloned();
            let line = finalize_standalone(
                Arc::clone(prepared),
                request,
                options.authored().quantize,
                scratch,
                &mut RawRoleBasis::Actual {
                    first: entry.accepted.is_empty(),
                },
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
    if ordinary.is_none()
        && options.authored().width.is_some()
        && let Some(frame) = first.as_ref()
        && tables.auto_phrase_between(start, frame.selected.request.end.index)
        && let Some(line) = choose_phrase(prepared, entry, indent, options, scratch, frame)?
    {
        return Ok(line);
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

fn receipt(line: &FinalizedLine) -> crate::FinalizedLine {
    crate::FinalizedLine {
        inner: Arc::new(FinalizedLine {
            selected: Arc::clone(&line.selected),
            mapping: Arc::clone(&line.mapping),
            data: line.data.clone(),
        }),
    }
}

fn choose_phrase(
    prepared: &Arc<PreparedData>,
    entry: RawEntry<'_>,
    indent: f64,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
    frame: &FinalizedLine,
) -> Result<Option<FinalizedLine>> {
    use crate::{
        ConditionalDisposition, ConditionalPurpose, ConditionalStage, PhraseOverflowDecision,
    };
    let RawEntry {
        start, incoming, ..
    } = entry;
    let handle = crate::PreparedText {
        inner: Arc::clone(prepared),
    };
    let PreparedTermination::Ordinary(ceiling) = &frame.selected.request.termination else {
        return Ok(None);
    };
    // Every earlier ordinary variant is an actual rejected hyphen alternative:
    // a meaningful None variant would already have been the first failed frame.
    let mut skipped = Vec::new();
    for (index, offer) in prepared
        .tables
        .opportunities
        .iter()
        .enumerate()
        .take(ceiling.index)
        .skip(
            prepared
                .tables
                .opportunities
                .partition_point(|row| row.boundary <= start),
        )
    {
        if offer.boundary <= start {
            continue;
        }
        for alternative in &offer.alternatives {
            let mut request = standalone_request(
                Arc::clone(prepared),
                start,
                PreparedTermination::Ordinary(PreparedOpportunity {
                    prepared: Arc::clone(prepared),
                    index,
                }),
                Some(PreparedAlternative {
                    prepared: Arc::clone(prepared),
                    index: *alternative,
                }),
                options.authored().width,
                indent,
                options.authored().alignment,
            )?;
            request.incoming = incoming.cloned();
            let line = finalize_standalone(
                Arc::clone(prepared),
                request,
                options.authored().quantize,
                scratch,
                &mut RawRoleBasis::Actual {
                    first: entry.accepted.is_empty(),
                },
            )?;
            skipped.push(ConditionalDisposition::ConsumerHyphenationPolicy(receipt(
                &line,
            )));
        }
    }
    let search = handle.conditional_search(
        &crate::PreparedBoundary {
            prepared: Arc::clone(prepared),
            index: start,
        },
        incoming,
        ceiling,
        ConditionalPurpose::Overflow,
        &skipped,
        Some(&receipt(frame)),
    )?;
    let mut decisions = Vec::new();
    for portion in search.phrase_portions() {
        let authored = options.authored();
        let enabled =
            authored.indent.first_line || authored.indent.each_line || authored.indent.hanging;
        let fresh_indent = if portion.start().index == start {
            indent
        } else if enabled && authored.indent.hanging {
            authored.indent.amount
        } else {
            0.0
        };
        let mut request = standalone_request(
            Arc::clone(prepared),
            portion.start().index,
            PreparedTermination::PhraseMeasurement(portion.clone()),
            None,
            authored.width,
            fresh_indent,
            authored.alignment,
        )?;
        request.incoming = portion.incoming();
        let line = finalize_standalone(
            Arc::clone(prepared),
            request,
            authored.quantize,
            scratch,
            &mut RawRoleBasis::Prospective { entry },
        )?;
        decisions.push(if fits(&line, authored.width)? {
            PhraseOverflowDecision::Fits(receipt(&line))
        } else {
            PhraseOverflowDecision::Overflows(receipt(&line))
        });
    }
    let mut batch = search.begin(&decisions)?;
    let mut restored_fallback = None;
    loop {
        let mut best = None;
        let mut first = None;
        let mut exhausted = Vec::new();
        for candidate in batch.opportunities() {
            let alternative = candidate.alternative();
            let allowed = if let Some(alternative) = &alternative {
                automatic_allowed(
                    prepared,
                    alternative.index,
                    entry,
                    indent,
                    options,
                    scratch,
                    Some(candidate),
                )?
            } else {
                true
            };
            let mut request = standalone_request(
                Arc::clone(prepared),
                start,
                PreparedTermination::Conditional(candidate.clone()),
                alternative,
                options.authored().width,
                indent,
                options.authored().alignment,
            )?;
            request.incoming = incoming.cloned();
            let line = finalize_standalone(
                Arc::clone(prepared),
                request,
                options.authored().quantize,
                scratch,
                &mut RawRoleBasis::Actual {
                    first: entry.accepted.is_empty(),
                },
            )?;
            let actual_fit = fits(&line, options.authored().width)?;
            if allowed && actual_fit {
                best = Some(line);
            } else {
                exhausted.push(if allowed {
                    ConditionalDisposition::Exhausted(receipt(&line))
                } else {
                    ConditionalDisposition::ConsumerHyphenationPolicy(receipt(&line))
                });
                if allowed && first.is_none() {
                    first = Some(line);
                }
            }
        }
        if best.is_some() {
            return Ok(best);
        }
        if batch.stage() == ConditionalStage::Arbitrary {
            return Ok(first.or(restored_fallback));
        }
        if restored_fallback.is_none() {
            restored_fallback = first;
        }
        let Some(next) = batch.advance(&exhausted)? else {
            return Ok(None);
        };
        batch = next;
    }
}

#[cfg(test)]
mod punctuation_tests;

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
    fn phrase_relaxation_uses_normal_prefixes_and_real_nonfitting_progress() {
        let (mut system, mut style) = system();
        system
            .register_fonts(
                include_bytes!("../../../tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf").to_vec(),
            )
            .unwrap();
        style.font = Font::new().family("Surgeist Phrase Ahem");
        style.locale = Some("ja".to_owned());
        style.word_break = crate::WordBreak::AutoPhrase;
        style.size = 10.0;
        for (width, expected) in [
            (20.0, vec![Range::new(0, 2), Range::new(2, 3)]),
            (
                5.0,
                vec![Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)],
            ),
        ] {
            let output = system
                .layout(
                    Source::new("日本へ"),
                    style.clone(),
                    crate::Options {
                        width: Some(width),
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
                expected
            );
            assert!(output.inner.lines.iter().all(|line| Arc::ptr_eq(
                &output.inner.prepared,
                &line.finalized.selected.prepared
            )));
        }
    }

    #[test]
    fn unsupported_phrase_locale_restores_authored_hyphen_before_raw_arbitrary_cuts() {
        let (mut system, mut style) = system();
        style.size = 10.0;
        style.word_break = crate::WordBreak::AutoPhrase;
        style.hyphens = crate::Hyphens::Manual;
        style.hyphenate_character = crate::HyphenateCharacter::String(crate::Text::from("A"));
        style.overflow_wrap = OverflowWrap::Anywhere;
        let output = system
            .layout(
                Source::new("AB\u{ad}CD"),
                style,
                crate::Options {
                    width: Some(30.0),
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
            [Range::new(0, 3), Range::new(3, 5)]
        );
        assert_eq!(output.inner.lines[0].finalized.data.metrics.advance, 30.0);
        assert!(
            matches!(output.inner.lines[0].finalized.selected.request.termination,
            PreparedTermination::Conditional(ref cut) if cut.stage()==crate::ConditionalStage::Hyphens)
        );
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
    fn trailing_out_of_flow_anchors_do_not_stop_original_l1_whitespace_reset() {
        let mut observed = Vec::new();
        for (kind, at) in [
            (None, 6),
            (Some(InlineBoxKind::OutOfFlow), 6),
            (Some(InlineBoxKind::OutOfFlow), 5),
            (Some(InlineBoxKind::InFlow), 6),
        ] {
            let (mut system, style) = system();
            let mut source = Source::new("X\u{202b}\u{05d0}  \u{202c}");
            if let Some(kind) = kind {
                source.inline_box(crate::InlineBox::new(
                    crate::Id::from_u64(91222),
                    kind,
                    at,
                    crate::Size::new(0.0, 0.0),
                ));
            }
            let output = system
                .layout(
                    source,
                    style,
                    crate::Options {
                        quantize: false,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                output.source().text().code_units(),
                [88, 0x202b, 0x5d0, 32, 32, 0x202c]
            );
            let line = &output.inner.lines[0];
            assert_eq!(output.inner.prepared.analyzed.analysis.base_level, 0);
            assert_eq!(
                line.finalized
                    .data
                    .clusters
                    .iter()
                    .find(|c| c.source.start == 2)
                    .unwrap()
                    .level,
                1
            );
            let whitespace: Vec<_> = [3, 4]
                .into_iter()
                .map(|i| {
                    let c = line
                        .finalized
                        .data
                        .clusters
                        .iter()
                        .find(|c| c.source.start == i)
                        .unwrap();
                    assert_eq!(c.advance, 20.0);
                    c.level
                })
                .collect();
            assert_eq!(line.finalized.data.boxes.len(), usize::from(kind.is_some()));
            observed.push((kind, at, whitespace));
        }
        assert_eq!(
            observed,
            [
                (None, 6, vec![0, 0]),
                (Some(InlineBoxKind::OutOfFlow), 6, vec![0, 0]),
                (Some(InlineBoxKind::OutOfFlow), 5, vec![0, 0]),
                (Some(InlineBoxKind::InFlow), 6, vec![1, 1])
            ]
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
