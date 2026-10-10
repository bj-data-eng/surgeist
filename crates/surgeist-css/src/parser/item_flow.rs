//! Finite contiguous constituent parsing for the adopted Grid 3 item family.
//!
//! The shorthand has four distinct slots, with widths 1/2/2/1 top-level
//! components. Search visits only those finite productions, at depth at most
//! four, using the original parser and numeric input context throughout.
//! Complete candidates rank by occupied count, D/W/P/T occupancy, then earlier
//! source starts in that order. Grouped wrap/pack productions never interleave.

use cssparser::{ParseError, Parser, ParserState, match_ignore_ascii_case};

use super::grid::parse_flow_tolerance;
use crate::{
    CssFlowTolerance, CssItemDirection, CssItemFlow, CssItemPack, CssItemWrap, CssItemWrapMode,
    CssItemWrapOrder,
    error::{Error, basic, unexpected_at},
    numeric::NumericInputContext,
};

type ParseResult<'i, T> = Result<T, ParseError<'i, Error>>;

pub(super) fn parse_item_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> ParseResult<'i, CssItemDirection> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssItemDirection::Auto),
        "row" => Ok(CssItemDirection::Row),
        "column" => Ok(CssItemDirection::Column),
        "row-reverse" => Ok(CssItemDirection::RowReverse),
        "column-reverse" => Ok(CssItemDirection::ColumnReverse),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_item_wrap<'i, 't>(input: &mut Parser<'i, 't>) -> ParseResult<'i, CssItemWrap> {
    // The longhand prefers its valid two-facet production. Shorthand search
    // visits both widths independently because its neighboring slots overlap.
    input
        .try_parse(|input| parse_wrap_width(input, 2))
        .or_else(|_| parse_wrap_width(input, 1))
}

fn wrap_keyword<'i, 't>(input: &mut Parser<'i, 't>) -> ParseResult<'i, CssItemWrap> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssItemWrap::Mode(CssItemWrapMode::Auto)),
        "nowrap" => Ok(CssItemWrap::Mode(CssItemWrapMode::NoWrap)),
        "wrap" => Ok(CssItemWrap::Mode(CssItemWrapMode::Wrap)),
        "normal" => Ok(CssItemWrap::Order(CssItemWrapOrder::Normal)),
        "reverse" => Ok(CssItemWrap::Order(CssItemWrapOrder::Reverse)),
        "wrap-reverse" => Ok(CssItemWrap::WrapReverse),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

fn parse_wrap_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    width: usize,
) -> ParseResult<'i, CssItemWrap> {
    let first = wrap_keyword(input)?;
    if width == 1 {
        return Ok(first);
    }
    let second = wrap_keyword(input)?;
    match (first, second) {
        (CssItemWrap::Mode(mode), CssItemWrap::Order(order))
        | (CssItemWrap::Order(order), CssItemWrap::Mode(mode)) => {
            Ok(CssItemWrap::Both(mode, order))
        }
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_item_pack<'i, 't>(input: &mut Parser<'i, 't>) -> ParseResult<'i, CssItemPack> {
    input
        .try_parse(|input| parse_pack_width(input, 2))
        .or_else(|_| parse_pack_width(input, 1))
}

fn pack_keyword<'i, 't>(input: &mut Parser<'i, 't>) -> ParseResult<'i, CssItemPack> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssItemPack::Normal),
        "dense" => Ok(CssItemPack::Dense),
        "balance" => Ok(CssItemPack::Balance),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

fn parse_pack_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    width: usize,
) -> ParseResult<'i, CssItemPack> {
    let first = pack_keyword(input)?;
    if width == 1 {
        return Ok(first);
    }
    let second = pack_keyword(input)?;
    match (first, second) {
        (CssItemPack::Dense, CssItemPack::Balance) | (CssItemPack::Balance, CssItemPack::Dense) => {
            Ok(CssItemPack::DenseBalance)
        }
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

#[derive(Clone)]
struct Candidate {
    direction: Option<CssItemDirection>,
    wrap: Option<CssItemWrap>,
    pack: Option<CssItemPack>,
    tolerance: Option<CssFlowTolerance>,
    // D/W/P/T use high-to-low bits so integer ordering matches occupancy rank.
    occupied: u8,
    starts: [usize; 4],
}

impl Candidate {
    fn empty() -> Self {
        Self {
            direction: None,
            wrap: None,
            pack: None,
            tolerance: None,
            occupied: 0,
            starts: [usize::MAX; 4],
        }
    }

    fn preferred_to(&self, other: &Self) -> bool {
        self.occupied.count_ones() > other.occupied.count_ones()
            || (self.occupied.count_ones() == other.occupied.count_ones()
                && (self.occupied > other.occupied
                    || (self.occupied == other.occupied && self.starts < other.starts)))
    }

    fn into_flow(self) -> CssItemFlow {
        CssItemFlow::new(
            self.direction.unwrap_or_default(),
            self.wrap.unwrap_or_default(),
            self.pack.unwrap_or_default(),
            self.tolerance.unwrap_or_default(),
        )
    }
}

struct Complete {
    candidate: Candidate,
    end: ParserState,
}

struct Selection<'i> {
    best: Option<Complete>,
    failure: ParseError<'i, Error>,
    failure_offset: usize,
}

impl<'i> Selection<'i> {
    fn failed(&mut self, input: &Parser<'i, '_>, error: ParseError<'i, Error>) {
        let offset = input.position().byte_index();
        if offset >= self.failure_offset {
            self.failure = error;
            self.failure_offset = offset;
        }
    }
}

pub(super) fn parse_item_flow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> ParseResult<'i, CssItemFlow> {
    let mut selection = Selection {
        best: None,
        failure: unexpected_at(input.current_source_location()),
        failure_offset: input.position().byte_index(),
    };
    search(input, numeric, &Candidate::empty(), &mut selection);
    if let Some(complete) = selection.best {
        input.reset(&complete.end);
        Ok(complete.candidate.into_flow())
    } else {
        Err(selection.failure)
    }
}

fn search<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    candidate: &Candidate,
    selection: &mut Selection<'i>,
) {
    input.skip_whitespace();
    let start = input.state();
    if input.is_exhausted() {
        if candidate.occupied != 0
            && selection
                .best
                .as_ref()
                .is_none_or(|best| candidate.preferred_to(&best.candidate))
        {
            selection.best = Some(Complete {
                candidate: candidate.clone(),
                end: start,
            });
        }
        return;
    }
    if candidate.occupied == 15 {
        let error = unexpected_at(input.current_source_location());
        selection.failed(input, error);
        return;
    }
    for slot in 0..4 {
        let bit = 8 >> slot;
        if candidate.occupied & bit != 0 {
            continue;
        }
        let maximum_width = if slot == 1 || slot == 2 { 2 } else { 1 };
        for width in 1..=maximum_width {
            input.reset(&start);
            let mut next = candidate.clone();
            let result = match slot {
                0 => parse_item_direction(input).map(|value| next.direction = Some(value)),
                1 => parse_wrap_width(input, width).map(|value| next.wrap = Some(value)),
                2 => parse_pack_width(input, width).map(|value| next.pack = Some(value)),
                3 => parse_flow_tolerance(input, numeric).map(|value| next.tolerance = Some(value)),
                _ => unreachable!("four fixed item-flow constituents"),
            };
            match result {
                Ok(()) => {
                    next.occupied |= bit;
                    next.starts[slot] = start.position().byte_index();
                    search(input, numeric, &next, selection);
                }
                Err(error) => selection.failed(input, error),
            }
        }
    }
    input.reset(&start);
}
