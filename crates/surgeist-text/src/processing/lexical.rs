//! Private executed lexical work and its immutable occurrence evidence.

use std::sync::Arc;

use icu_properties::{CodePointMapData, CodePointSetData, props};

use super::{
    DeletionCause, DocumentSegmentBreaks, EffectiveInterval, InlineBoxKind, MappingKind,
    MappingPiece, Range, Result, Source, WhiteSpace,
    lexical_inventory::{
        LexicalBarrierOrigin, LexicalCharacterRole, LexicalContext, LexicalDisposition,
        LexicalOutput, LexicalRecord, LexicalRecordKind, LexicalState,
    },
    overflow, reserve,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Kind {
    Text,
    BidiControl,
    Space,
    Break,
    Barrier,
    Marker,
}

struct Token {
    source: Range,
    mode: WhiteSpace,
    kind: Kind,
    mapping: MappingKind,
    replacement: Option<u16>,
    context: Option<char>,
    before: Option<char>,
    domain: usize,
    marker: Option<usize>,
    plaintext_reset: bool,
    atomic: Option<usize>,
    scope: Option<usize>,
    /// Genuine capture event, distinct from the transient trim-marker index.
    event: Option<usize>,
}

impl Token {
    fn deleted(&self) -> bool {
        matches!(self.mapping, MappingKind::Delete(_))
    }
    fn transparent(&self) -> bool {
        self.deleted() || matches!(self.kind, Kind::BidiControl | Kind::Marker)
    }
    fn collapse(&self, kind: Kind) -> bool {
        self.kind == kind
            && match kind {
                Kind::Space => self.mode.collapses_spaces(),
                Kind::Break => self.mode == WhiteSpace::Collapse,
                _ => false,
            }
    }
    fn delete(&mut self, cause: DeletionCause) {
        self.mapping = MappingKind::Delete(cause);
    }
}

/// Admitted semantic requests share tokenization, trim ordering, Phase I and
/// emission. Callers validate their genuinely different source/native/context
/// contracts before constructing this private request.
pub(super) enum ValidatedProcessingRequest<'a> {
    SourceLocal {
        source: &'a Source,
        breaks: &'a DocumentSegmentBreaks,
        modes: &'a [EffectiveInterval],
        box_order: &'a [usize],
        barriers: &'a [usize],
        decode_boundaries: &'a [usize],
    },
    Contextual {
        source: &'a Source,
        breaks: &'a DocumentSegmentBreaks,
        context: &'a super::contextual::ProcessingContext,
        capture: &'a Arc<super::contextual::ScopeCapture>,
        modes: &'a [EffectiveInterval],
    },
}
pub(super) fn execute(request: ValidatedProcessingRequest<'_>) -> Result<LexicalOutput> {
    match request {
        ValidatedProcessingRequest::SourceLocal {
            source,
            breaks,
            modes,
            box_order,
            barriers,
            decode_boundaries,
        } => {
            let mut tokens = tokenize(
                source,
                breaks,
                modes,
                box_order,
                barriers,
                decode_boundaries,
            )?;
            phase_one(&mut tokens);
            emit(source, tokens, LexicalContext::SourceLocal)
        }
        ValidatedProcessingRequest::Contextual {
            source,
            breaks,
            context,
            capture,
            modes,
        } => execute_contextual(source, breaks, context, capture, modes),
    }
}

fn phase_one(tokens: &mut [Token]) {
    // Remove eligible runs after and before every surviving original delimiter,
    // including preserved delimiters before their eventual LF/SPACE emission.
    around_breaks(tokens.iter_mut());
    around_breaks(tokens.iter_mut().rev());

    let mut preceding_break = false;
    for token in tokens.iter_mut() {
        if token.transparent() {
            continue;
        }
        if token.collapse(Kind::Break) {
            if preceding_break {
                token.delete(DeletionCause::DuplicateSegmentBreak);
            }
            preceding_break = true;
        } else {
            preceding_break = false;
        }
    }

    // Linear neighbor sweeps over actual original scalar atoms. Pair decoding
    // happened before deletion, so transparent/deleted units cannot fuse them.
    let mut context = None;
    for token in tokens.iter_mut() {
        if token.transparent() {
            continue;
        }
        token.before = context;
        context = token.context;
    }
    let width = CodePointMapData::<props::EastAsianWidth>::new();
    let script = CodePointMapData::<props::Script>::new();
    let wide = |ch| {
        matches!(
            width.get(ch),
            props::EastAsianWidth::Fullwidth
                | props::EastAsianWidth::Wide
                | props::EastAsianWidth::Halfwidth
        ) && script.get(ch) != props::Script::Hangul
    };
    context = None;
    for token in tokens.iter_mut().rev() {
        if token.transparent() {
            continue;
        }
        if token.collapse(Kind::Break)
            && (token.before == Some('\u{200b}')
                || context == Some('\u{200b}')
                || token.before.is_some_and(&wide) && context.is_some_and(&wide))
        {
            token.delete(DeletionCause::ContextualSegmentBreak);
            continue;
        }
        context = token.context;
    }

    let mut preceding_space = false;
    for token in tokens.iter_mut() {
        if token.transparent() {
            continue;
        }
        if token.collapse(Kind::Space) || token.collapse(Kind::Break) {
            if preceding_space {
                token.delete(DeletionCause::CollapsedSpace);
            }
            preceding_space = true;
        } else {
            preceding_space = false;
        }
    }
}

fn around_breaks<'a>(tokens: impl Iterator<Item = &'a mut Token>) {
    let mut adjacent = false;
    for token in tokens {
        if token.transparent() {
            continue;
        }
        if token.kind == Kind::Break {
            adjacent = true;
        } else if adjacent && token.collapse(Kind::Space) {
            token.delete(DeletionCause::SpaceAroundSegmentBreak);
        } else {
            adjacent = false;
        }
    }
}

fn tokenize(
    source: &Source,
    breaks: &DocumentSegmentBreaks,
    modes: &[EffectiveInterval],
    box_order: &[usize],
    extra_barriers: &[usize],
    // Sorted original-unit cuts. Unlike barriers these emit no context token.
    decode_boundaries: &[usize],
) -> Result<Vec<Token>> {
    let mut barriers = Vec::new();
    for &ordinal in box_order {
        let box_ = source.boxes()[ordinal];
        if box_.kind() == InlineBoxKind::InFlow && barriers.last().copied() != Some(box_.index()) {
            reserve(&mut barriers, 1)?;
            barriers.push(box_.index());
        }
    }
    reserve(&mut barriers, extra_barriers.len())?;
    barriers.extend_from_slice(extra_barriers);
    barriers.sort_unstable();
    barriers.dedup();
    let bidi = CodePointSetData::new::<props::BidiControl>();
    let units = source.text().code_units();
    let mut tokens = Vec::new();
    let mut next_barrier = 0;
    let mut delimiter = 0;
    let mut interval = 0;
    let mut at = 0;
    while at < units.len() {
        if barriers.get(next_barrier) == Some(&at) {
            reserve(&mut tokens, 1)?;
            tokens.push(Token {
                source: Range::new(at, at),
                mode: WhiteSpace::Preserve,
                kind: Kind::Barrier,
                mapping: MappingKind::Copy,
                replacement: None,
                context: None,
                before: None,
                domain: 0,
                marker: None,
                plaintext_reset: false,
                atomic: None,
                scope: None,
                event: None,
            });
            next_barrier += 1;
        }
        while modes[interval].range.end <= at {
            interval += 1;
        }
        let mode = modes[interval].mode;
        let unit = units[at];
        let one_end = at.checked_add(1).ok_or_else(overflow)?;
        let delimiter_end = match breaks {
            DocumentSegmentBreaks::LineFeedDefault => (unit == 0x000a).then_some(one_end),
            DocumentSegmentBreaks::Defined(ranges) => ranges
                .get(delimiter)
                .filter(|range| range.start == at)
                .map(|range| {
                    delimiter += 1;
                    range.end
                }),
        };
        let mut end = delimiter_end.unwrap_or(one_end);
        let (kind, mapping, replacement, context) = if delimiter_end.is_some() {
            match mode {
                WhiteSpace::Discard => (
                    Kind::Break,
                    MappingKind::Delete(DeletionCause::Discard),
                    None,
                    None,
                ),
                WhiteSpace::Preserve | WhiteSpace::PreserveBreaks | WhiteSpace::BreakSpaces => {
                    (Kind::Break, MappingKind::Replace, Some(0x000a), Some('\n'))
                }
                WhiteSpace::PreserveSpaces | WhiteSpace::Collapse => {
                    (Kind::Break, MappingKind::Replace, Some(0x0020), Some(' '))
                }
            }
        } else if matches!(unit, 0x0020 | 0x0009 | 0x000d) {
            match (mode, unit) {
                (WhiteSpace::Discard, _) => (
                    Kind::Space,
                    MappingKind::Delete(DeletionCause::Discard),
                    None,
                    None,
                ),
                (WhiteSpace::Preserve | WhiteSpace::BreakSpaces, 0x000d)
                | (
                    WhiteSpace::PreserveSpaces | WhiteSpace::Collapse | WhiteSpace::PreserveBreaks,
                    0x0009 | 0x000d,
                ) => (Kind::Space, MappingKind::Replace, Some(0x0020), Some(' ')),
                _ => (
                    Kind::Space,
                    MappingKind::Copy,
                    None,
                    char::from_u32(u32::from(unit)),
                ),
            }
        } else {
            let next_delimiter = match breaks {
                DocumentSegmentBreaks::LineFeedDefault => false,
                DocumentSegmentBreaks::Defined(ranges) => ranges
                    .get(delimiter)
                    .is_some_and(|range| range.start == one_end),
            };
            let scalar = if (0xd800..=0xdbff).contains(&unit)
                && units
                    .get(one_end)
                    .is_some_and(|low| (0xdc00..=0xdfff).contains(low))
                && barriers.get(next_barrier) != Some(&one_end)
                && decode_boundaries.binary_search(&one_end).is_err()
                && !next_delimiter
            {
                end = one_end.checked_add(1).ok_or_else(overflow)?;
                char::from_u32(
                    0x10000
                        + ((u32::from(unit) - 0xd800) << 10)
                        + (u32::from(units[one_end]) - 0xdc00),
                )
            } else {
                char::from_u32(u32::from(unit))
            };
            let kind = if scalar.is_some_and(|ch| bidi.contains(ch)) {
                Kind::BidiControl
            } else {
                Kind::Text
            };
            (kind, MappingKind::Copy, None, scalar)
        };
        reserve(&mut tokens, 1)?;
        tokens.push(Token {
            source: Range::new(at, end),
            mode,
            kind,
            mapping,
            replacement,
            context,
            before: None,
            domain: 0,
            marker: None,
            plaintext_reset: false,
            atomic: None,
            scope: None,
            event: None,
        });
        at = end;
    }
    Ok(tokens)
}

fn emit(source: &Source, tokens: Vec<Token>, context: LexicalContext) -> Result<LexicalOutput> {
    let mut plaintext = Vec::new();
    let mut records = Vec::new();

    let mut units = Vec::new();
    let mut map: Vec<MappingPiece> = Vec::new();
    for token in tokens {
        let start = units.len();
        let end = start
            .checked_add(match token.mapping {
                MappingKind::Copy => token.source.len(),
                MappingKind::Replace => 1,
                MappingKind::Delete(_) => 0,
            })
            .ok_or_else(overflow)?;
        let kind = match token.kind {
            Kind::Marker => LexicalRecordKind::ScopeMarker {
                event: token
                    .event
                    .expect("contextual marker retains its capture event"),
            },
            Kind::Barrier => LexicalRecordKind::Barrier {
                origin: token.event.map_or(
                    LexicalBarrierOrigin::SourceLocalCut(token.source.start),
                    LexicalBarrierOrigin::CapturedEvent,
                ),
            },
            Kind::Text | Kind::BidiControl | Kind::Space | Kind::Break => {
                LexicalRecordKind::Character {
                    source: token.source,
                    mode: token.mode,
                    role: match token.kind {
                        Kind::Text => LexicalCharacterRole::Text,
                        Kind::BidiControl => LexicalCharacterRole::BidiControl,
                        Kind::Space => LexicalCharacterRole::Space,
                        Kind::Break => LexicalCharacterRole::SegmentBreak,
                        Kind::Marker | Kind::Barrier => unreachable!("character token"),
                    },
                    disposition: match token.mapping {
                        MappingKind::Delete(cause) => LexicalDisposition::Deleted {
                            processed: start,
                            cause,
                        },
                        MappingKind::Copy | MappingKind::Replace => LexicalDisposition::Emitted {
                            processed: Range::new(start, end),
                        },
                    },
                }
            }
        };
        let record = LexicalRecord {
            domain: token.domain,
            kind,
        };
        reserve(&mut plaintext, 1)?;
        plaintext.push(record.plaintext(start, token.plaintext_reset, token.atomic));
        reserve(&mut records, 1)?;
        records.push(record);
        if matches!(token.kind, Kind::Marker | Kind::Barrier) {
            continue;
        }
        if let Some(last) = map
            .last_mut()
            .filter(|piece| piece.kind == token.mapping && token.mapping != MappingKind::Replace)
        {
            last.source.end = token.source.end;
        } else {
            reserve(&mut map, 1)?;
            map.push(MappingPiece {
                source: token.source,
                processed: units.len(),
                kind: token.mapping,
            });
        }
        match token.mapping {
            MappingKind::Copy => {
                reserve(&mut units, token.source.len())?;
                units.extend_from_slice(
                    &source.text().code_units()[token.source.start..token.source.end],
                );
            }
            MappingKind::Replace => {
                reserve(&mut units, 1)?;
                units.push(
                    token
                        .replacement
                        .expect("Replace token supplies its actual output"),
                );
            }
            MappingKind::Delete(_) => {}
        }
    }
    Ok(LexicalOutput {
        units,
        map,
        plaintext_tokens: plaintext,
        lexical: LexicalState::Executed { context, records },
    })
}

fn execute_contextual(
    source: &Source,
    breaks: &DocumentSegmentBreaks,
    context: &super::contextual::ProcessingContext,
    capture: &Arc<super::contextual::ScopeCapture>,
    modes: &[EffectiveInterval],
) -> Result<LexicalOutput> {
    let mut decode_barriers = Vec::new();
    for pair in context.origins.windows(2) {
        if pair[0].1 != pair[1].1 {
            decode_barriers.push(pair[1].0.start);
        }
    }
    decode_barriers.extend(context.barriers.iter().map(|barrier| barrier.at));
    decode_barriers.sort_unstable();
    decode_barriers.dedup();
    let domain_at = |at| {
        context
            .origins
            .get(
                context
                    .origins
                    .partition_point(|(range, _)| range.end <= at),
            )
            .filter(|(range, _)| range.start <= at)
            .map(|(_, domain)| *domain)
    };
    let mut decode_boundaries = Vec::new();
    reserve(&mut decode_boundaries, context.decode_boundaries.len())?;
    for &(at, domain) in &context.decode_boundaries {
        // An empty child can own a Wbr at a parent's raw unit cut. Only that
        // control's actual incident original domain may consume its decode cut;
        // ordinary domain transitions already have their own decode barriers.
        if at.checked_sub(1).and_then(&domain_at) == Some(domain) && domain_at(at) == Some(domain) {
            decode_boundaries.push(at);
        }
    }
    let mut tokens = tokenize(
        source,
        breaks,
        modes,
        &[],
        &decode_barriers,
        &decode_boundaries,
    )?;
    tokens.retain(|token| token.kind != Kind::Barrier);
    for token in &mut tokens {
        let origin = context
            .origins
            .partition_point(|(range, _)| range.end <= token.source.start);
        token.domain = context
            .origins
            .get(origin)
            .map(|(_, domain)| *domain)
            .ok_or_else(|| super::contextual::invalid(crate::SourceProcessingError::Domain))?;
    }
    // All zero-extent actions precede the next original contribution. Their
    // stream order remains significant even at identical source offsets.
    let mut ordered: Vec<_> = tokens
        .into_iter()
        .map(|token| ((token.source.start, usize::MAX), token))
        .collect();
    for (i, boundary) in context.boundaries.iter().enumerate() {
        ordered.push((
            (boundary.at, boundary.order),
            Token {
                source: Range::new(boundary.at, boundary.at),
                domain: boundary.domain,
                kind: Kind::Marker,
                mode: WhiteSpace::Preserve,
                mapping: MappingKind::Copy,
                replacement: None,
                context: None,
                before: None,
                marker: Some(i),
                plaintext_reset: false,
                atomic: None,
                scope: None,
                event: Some(boundary.order),
            },
        ));
    }
    for barrier in &context.barriers {
        let (at, domain, order, reset, atomic, scope) = (
            barrier.at,
            barrier.domain,
            barrier.order,
            barrier.plaintext_reset,
            barrier.atomic,
            barrier.scope,
        );
        ordered.push((
            (at, order),
            Token {
                source: Range::new(at, at),
                domain,
                kind: Kind::Barrier,
                mode: WhiteSpace::Preserve,
                mapping: MappingKind::Copy,
                replacement: None,
                context: None,
                before: None,
                marker: None,
                plaintext_reset: reset,
                atomic,
                scope,
                event: Some(order),
            },
        ));
    }
    ordered.sort_by_key(|(key, _)| *key);
    let mut tokens: Vec<_> = ordered.into_iter().map(|(_, token)| token).collect();
    let mut scope_starts = std::collections::BTreeMap::new();
    for i in 0..tokens.len() {
        let Some(marker) = tokens[i].marker else {
            continue;
        };
        let boundary = &context.boundaries[marker];
        if boundary.enter {
            scope_starts.insert(boundary.scope, i);
        } else {
            let start = scope_starts[&boundary.scope];
            let opening = &context.boundaries[tokens[start].marker.expect("scope marker")];
            if opening.trim.discard_before {
                trim_exterior(
                    &mut tokens,
                    0..start,
                    opening.exterior_domain,
                    true,
                    opening.atomic,
                    Some(opening.scope),
                );
            }
            if boundary.trim.discard_after {
                let len = tokens.len();
                trim_exterior(
                    &mut tokens,
                    i + 1..len,
                    boundary.exterior_domain,
                    false,
                    None,
                    Some(boundary.scope),
                );
            }
            if boundary.trim.discard_inner {
                trim_inner(&mut tokens, start + 1..i, boundary.domain, boundary.block);
            }
        }
    }
    // The one Phase I implementation receives each true formatting domain.
    // Emit subsequently restores original envelope order; no copied Source is made.
    let mut domains: Vec<Vec<(usize, Token)>> =
        (0..context.domains.len()).map(|_| Vec::new()).collect();
    for (i, token) in tokens.into_iter().enumerate() {
        domains[token.domain].push((i, token));
    }
    let mut executed = Vec::new();
    for domain in domains {
        let (indices, mut tokens): (Vec<_>, Vec<_>) = domain.into_iter().unzip();
        phase_one(&mut tokens);
        executed.extend(indices.into_iter().zip(tokens));
    }
    executed.sort_by_key(|(index, _)| *index);
    emit(
        source,
        executed.into_iter().map(|(_, token)| token).collect(),
        LexicalContext::Captured(Arc::clone(capture)),
    )
}

fn trim_exterior(
    tokens: &mut [Token],
    range: std::ops::Range<usize>,
    domain: usize,
    reverse: bool,
    ignore_atomic: Option<usize>,
    ignore_scope: Option<usize>,
) {
    let indices: Vec<_> = if reverse {
        range.rev().collect()
    } else {
        range.collect()
    };
    for i in indices {
        let token = &mut tokens[i];
        if token.domain != domain
            || token.transparent()
            || ignore_atomic.is_some_and(|atomic| token.atomic == Some(atomic))
            || ignore_scope.is_some_and(|scope| token.scope == Some(scope))
        {
            continue;
        }
        // Genuine trim runs before Phase I. PreserveBreaks' original
        // delimiters remain collapsible for this explicit exterior trimming.
        if token.mode.collapses_spaces() && matches!(token.kind, Kind::Space | Kind::Break) {
            token.delete(DeletionCause::Trim);
        } else {
            break;
        }
    }
}
fn trim_inner(tokens: &mut [Token], range: std::ops::Range<usize>, domain: usize, block: bool) {
    let indices: Vec<_> = range
        .filter(|i| tokens[*i].domain == domain && !tokens[*i].transparent())
        .collect();
    for reverse in [false, true] {
        let ordered: Vec<_> = if reverse {
            indices.iter().copied().rev().collect()
        } else {
            indices.clone()
        };
        let mut whitespace = Vec::new();
        let mut through_break = 0;
        for i in ordered {
            if !matches!(tokens[i].kind, Kind::Space | Kind::Break) {
                break;
            }
            whitespace.push(i);
            if tokens[i].kind == Kind::Break {
                through_break = whitespace.len();
            }
        }
        let count = if block {
            through_break
        } else {
            whitespace.len()
        };
        for i in whitespace.into_iter().take(count) {
            tokens[i].delete(DeletionCause::Trim);
        }
    }
}
