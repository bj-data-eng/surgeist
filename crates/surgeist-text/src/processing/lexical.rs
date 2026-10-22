//! Private executed lexical work; only the correspondence survives this scratch.

use icu_properties::{CodePointMapData, CodePointSetData, props};

use super::{
    DeletionCause, DocumentSegmentBreaks, EffectiveInterval, InlineBoxKind, MappingKind,
    MappingPiece, Range, Result, Source, WhiteSpace, overflow, reserve,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Kind {
    Text,
    BidiControl,
    Space,
    Break,
    Barrier,
}

struct Token {
    source: Range,
    mode: WhiteSpace,
    kind: Kind,
    mapping: MappingKind,
    replacement: Option<u16>,
    context: Option<char>,
    before: Option<char>,
}

impl Token {
    fn deleted(&self) -> bool {
        matches!(self.mapping, MappingKind::Delete(_))
    }
    fn transparent(&self) -> bool {
        self.deleted() || self.kind == Kind::BidiControl
    }
    fn collapse(&self, kind: Kind) -> bool {
        self.mode == WhiteSpace::Collapse && self.kind == kind
    }
    fn delete(&mut self, cause: DeletionCause) {
        self.mapping = MappingKind::Delete(cause);
    }
}

pub(super) fn execute(
    source: &Source,
    breaks: &DocumentSegmentBreaks,
    modes: &[EffectiveInterval],
    box_order: &[usize],
    extra_barriers: &[usize],
) -> Result<(Vec<u16>, Vec<MappingPiece>)> {
    let mut tokens = tokenize(source, breaks, modes, box_order, extra_barriers)?;
    // Remove eligible runs after and before every surviving original delimiter,
    // including preserved delimiters before their eventual LF/SPACE emission.
    around_breaks(tokens.iter_mut());
    around_breaks(tokens.iter_mut().rev());

    let mut preceding_break = false;
    for token in &mut tokens {
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
    for token in &mut tokens {
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
    for token in &mut tokens {
        if token.transparent() {
            continue;
        }
        if token.mode == WhiteSpace::Collapse && matches!(token.kind, Kind::Space | Kind::Break) {
            if preceding_space {
                token.delete(DeletionCause::CollapsedSpace);
            }
            preceding_space = true;
        } else {
            preceding_space = false;
        }
    }
    emit(source, tokens)
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
                WhiteSpace::Preserve => {
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
                (WhiteSpace::Preserve, 0x000d)
                | (WhiteSpace::PreserveSpaces | WhiteSpace::Collapse, 0x0009 | 0x000d) => {
                    (Kind::Space, MappingKind::Replace, Some(0x0020), Some(' '))
                }
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
        });
        at = end;
    }
    Ok(tokens)
}

fn emit(source: &Source, tokens: Vec<Token>) -> Result<(Vec<u16>, Vec<MappingPiece>)> {
    let mut units = Vec::new();
    let mut map: Vec<MappingPiece> = Vec::new();
    for token in tokens {
        if token.kind == Kind::Barrier {
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
    Ok((units, map))
}
