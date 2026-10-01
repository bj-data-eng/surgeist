use super::values::{parse_length, parse_length_percentage};
use cssparser::{ParseError, Parser, ParserState, Token, match_ignore_ascii_case};

use super::values::{next_is_comma, next_is_delim};
use crate::error::{Error, unsupported_value};
use crate::syntax::*;

pub(super) fn next_starts_background_position<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(value)) => matches!(
            value.to_ascii_lowercase().as_str(),
            "left" | "right" | "top" | "bottom" | "center"
        ),
        Ok(Token::Dimension { .. } | Token::Percentage { .. }) => true,
        Ok(Token::Number { value, .. }) => *value == 0.0,
        Ok(Token::Function(name)) => crate::numeric::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

pub(super) fn parse_background_position_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundPosition, ParseError<'i, Error>> {
    let mut atoms = Vec::new();
    let mut states = Vec::new();
    while atoms.len() < 4 && next_starts_background_position(input) {
        states.push(input.state());
        atoms.push(parse_generic_position_atom(
            input,
            numeric,
            PositionGrammar::Physical,
        )?);
    }
    build_background_position(&atoms).ok_or_else(|| {
        invalid_generic_position_atom(input, &states[invalid_background_atom_index(&atoms)])
    })
}

pub(super) fn parse_mask_position_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPhysicalPositionList, ParseError<'i, Error>> {
    let mut positions = Vec::new();
    loop {
        positions.push(parse_physical_position(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "mask-position list has an empty item",
            ));
        }
    }
    CssPhysicalPositionList::try_new(positions)
        .ok_or_else(|| unsupported_value(input, None, "mask-position list is empty"))
}

pub(super) fn parse_background_position_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundPositionList, ParseError<'i, Error>> {
    let mut positions = Vec::new();
    loop {
        positions.push(parse_background_position(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background-position list has an empty item",
            ));
        }
    }
    CssBackgroundPositionList::try_new(positions)
        .ok_or_else(|| unsupported_value(input, None, "background-position list is empty"))
}

pub(super) fn parse_object_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPhysicalPosition, ParseError<'i, Error>> {
    parse_physical_position(input, numeric)
}

pub(super) fn parse_transform_origin<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransformOrigin, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input, numeric, PositionGrammar::Physical)?;

    if atoms.len() <= 2
        && let Some(position) = build_physical_position(&atoms)
    {
        return CssTransformOrigin::try_new(position, None)
            .ok_or_else(|| invalid_generic_position_atom(input, &states[0]));
    }

    // The selected transform-origin grammar requires two planar atoms before Z.
    // A vertical keyword followed by a length is not reinterpreted as planar + Z.
    if atoms.len() == 3 {
        let z_index = atoms.len() - 1;
        if let Some(position) = build_physical_position(&atoms[..z_index]) {
            input.reset(&states[z_index]);
            let z = parse_length(input, numeric, "transform-origin z")?;
            return CssTransformOrigin::try_new(position, Some(z))
                .ok_or_else(|| invalid_generic_position_atom(input, &states[0]));
        }
    }

    let invalid_index = if atoms.len() > 3 {
        3
    } else {
        invalid_atom_index(&atoms)
    };
    Err(invalid_generic_position_atom(input, &states[invalid_index]))
}

// Shapes consumers select the full Values 5 position grammar.
pub(super) fn parse_full_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input, numeric, PositionGrammar::Full)?;
    build_full_position(&atoms)
        .ok_or_else(|| invalid_generic_position_atom(input, &states[invalid_atom_index(&atoms)]))
}

/// Parses the entire position region, leaving caller-owned boundary keywords unconsumed.
pub(super) fn parse_full_position_bounded<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    boundaries: &[&str],
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    let (atoms, states) =
        parse_position_atoms_bounded(input, numeric, PositionGrammar::Full, boundaries)?;
    build_full_position(&atoms)
        .ok_or_else(|| invalid_generic_position_atom(input, &states[invalid_atom_index(&atoms)]))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PositionGrammar {
    Full,
    Physical,
}

#[derive(Clone, Copy, Debug)]
enum FlowEdge {
    Start,
    End,
}

#[derive(Clone, Debug)]
enum GenericPositionAtom {
    Horizontal(CssHorizontalPositionKeyword),
    Vertical(CssVerticalPositionKeyword),
    Center,
    Block(FlowEdge),
    Inline(FlowEdge),
    Relative(FlowEdge),
    Offset(CssSpecifiedLengthPercentage),
}

pub(super) fn parse_physical_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPhysicalPosition, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input, numeric, PositionGrammar::Physical)?;
    build_physical_position(&atoms)
        .ok_or_else(|| invalid_generic_position_atom(input, &states[invalid_atom_index(&atoms)]))
}

fn parse_position_atoms<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    grammar: PositionGrammar,
) -> std::result::Result<(Vec<GenericPositionAtom>, Vec<ParserState>), ParseError<'i, Error>> {
    parse_position_atoms_bounded(input, numeric, grammar, &[])
}
fn parse_position_atoms_bounded<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    grammar: PositionGrammar,
    boundaries: &[&str],
) -> std::result::Result<(Vec<GenericPositionAtom>, Vec<ParserState>), ParseError<'i, Error>> {
    let mut atoms = Vec::new();
    let mut states = Vec::new();
    while !input.is_exhausted()
        && !next_is_comma(input)
        && !next_is_delim(input, '/')
        && !boundaries
            .iter()
            .any(|word| super::values::next_is_ident(input, word))
    {
        states.push(input.state());
        atoms.push(parse_generic_position_atom(input, numeric, grammar)?);
        if atoms.len() > 4 {
            return Err(invalid_generic_position_atom(input, &states[4]));
        }
    }
    if atoms.is_empty() {
        return Err(unsupported_value(input, None, "position is empty"));
    }
    Ok((atoms, states))
}

fn parse_background_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundPosition, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input, numeric, PositionGrammar::Physical)?;
    build_background_position(&atoms).ok_or_else(|| {
        invalid_generic_position_atom(input, &states[invalid_background_atom_index(&atoms)])
    })
}

fn parse_generic_position_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    grammar: PositionGrammar,
) -> std::result::Result<GenericPositionAtom, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if grammar == PositionGrammar::Full {
            let logical = match ident.to_ascii_lowercase().as_str() {
                "x-start" => Some(GenericPositionAtom::Horizontal(
                    CssHorizontalPositionKeyword::XStart,
                )),
                "x-end" => Some(GenericPositionAtom::Horizontal(
                    CssHorizontalPositionKeyword::XEnd,
                )),
                "y-start" => Some(GenericPositionAtom::Vertical(
                    CssVerticalPositionKeyword::YStart,
                )),
                "y-end" => Some(GenericPositionAtom::Vertical(
                    CssVerticalPositionKeyword::YEnd,
                )),
                "block-start" => Some(GenericPositionAtom::Block(FlowEdge::Start)),
                "block-end" => Some(GenericPositionAtom::Block(FlowEdge::End)),
                "inline-start" => Some(GenericPositionAtom::Inline(FlowEdge::Start)),
                "inline-end" => Some(GenericPositionAtom::Inline(FlowEdge::End)),
                "start" => Some(GenericPositionAtom::Relative(FlowEdge::Start)),
                "end" => Some(GenericPositionAtom::Relative(FlowEdge::End)),
                _ => None,
            };
            if let Some(atom) = logical {
                return Ok(atom);
            }
        }
        return match_ignore_ascii_case! { &ident,
            "left" => Ok(GenericPositionAtom::Horizontal(CssHorizontalPositionKeyword::Left)),
            "right" => Ok(GenericPositionAtom::Horizontal(CssHorizontalPositionKeyword::Right)),
            "top" => Ok(GenericPositionAtom::Vertical(CssVerticalPositionKeyword::Top)),
            "bottom" => Ok(GenericPositionAtom::Vertical(CssVerticalPositionKeyword::Bottom)),
            "center" => Ok(GenericPositionAtom::Center),
            _ => {
                input.reset(&state);
                Err(invalid_generic_position_atom(input, &state))
            },
        };
    }
    input.reset(&state);
    let value = parse_length_percentage(input, numeric, "position")?;
    Ok(GenericPositionAtom::Offset(value))
}

fn invalid_generic_position_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
    state: &ParserState,
) -> ParseError<'i, Error> {
    input.reset(state);
    let location = input.current_source_location();
    match input.next() {
        Ok(token) => location.new_unexpected_token_error::<Error>(token.clone()),
        Err(error) => error.into(),
    }
}

fn invalid_atom_index(atoms: &[GenericPositionAtom]) -> usize {
    match atoms.len() {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => {
            if build_full_position(&atoms[..2]).is_some() {
                2
            } else if !is_edge_atom(&atoms[0]) {
                0
            } else if !matches!(atoms[1], GenericPositionAtom::Offset(_)) {
                1
            } else if !is_edge_atom(&atoms[2]) {
                2
            } else if !matches!(atoms[3], GenericPositionAtom::Offset(_)) {
                3
            } else {
                2
            }
        }
        _ => 0,
    }
}

fn is_edge_atom(atom: &GenericPositionAtom) -> bool {
    match atom {
        GenericPositionAtom::Horizontal(keyword) => is_horizontal_edge(*keyword),
        GenericPositionAtom::Vertical(keyword) => is_vertical_edge(*keyword),
        GenericPositionAtom::Block(_)
        | GenericPositionAtom::Inline(_)
        | GenericPositionAtom::Relative(_) => true,
        GenericPositionAtom::Center | GenericPositionAtom::Offset(_) => false,
    }
}

fn build_cartesian_axes(
    atoms: &[GenericPositionAtom],
) -> Option<(CssHorizontalPosition, CssVerticalPosition)> {
    use GenericPositionAtom::{Center, Horizontal, Offset, Vertical};

    let (horizontal, vertical) = match atoms {
        [Horizontal(keyword)] => (horizontal_keyword(*keyword), CssVerticalPosition::Center),
        [Vertical(keyword)] => (CssHorizontalPosition::Center, vertical_keyword(*keyword)),
        [Center] => (CssHorizontalPosition::Center, CssVerticalPosition::Center),
        [Offset(offset)] => (
            CssHorizontalPosition::Offset(offset.clone()),
            CssVerticalPosition::Center,
        ),
        [Horizontal(horizontal), Vertical(vertical)]
        | [Vertical(vertical), Horizontal(horizontal)] => {
            (horizontal_keyword(*horizontal), vertical_keyword(*vertical))
        }
        [Horizontal(horizontal), Center] | [Center, Horizontal(horizontal)] => {
            (horizontal_keyword(*horizontal), CssVerticalPosition::Center)
        }
        [Vertical(vertical), Center] | [Center, Vertical(vertical)] => {
            (CssHorizontalPosition::Center, vertical_keyword(*vertical))
        }
        [Center, Center] => (CssHorizontalPosition::Center, CssVerticalPosition::Center),
        [Horizontal(horizontal), Offset(offset)] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Offset(offset.clone()),
        ),
        [Center, Offset(offset)] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Offset(offset.clone()),
        ),
        [Offset(offset), Vertical(vertical)] => (
            CssHorizontalPosition::Offset(offset.clone()),
            vertical_keyword(*vertical),
        ),
        [Offset(offset), Center] => (
            CssHorizontalPosition::Offset(offset.clone()),
            CssVerticalPosition::Center,
        ),
        [Offset(horizontal), Offset(vertical)] => (
            CssHorizontalPosition::Offset(horizontal.clone()),
            CssVerticalPosition::Offset(vertical.clone()),
        ),
        [
            Horizontal(horizontal),
            Offset(horizontal_offset),
            Vertical(vertical),
            Offset(vertical_offset),
        ] if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) => (
            horizontal_edge_offset(*horizontal, horizontal_offset.clone()),
            vertical_edge_offset(*vertical, vertical_offset.clone()),
        ),
        [
            Vertical(vertical),
            Offset(vertical_offset),
            Horizontal(horizontal),
            Offset(horizontal_offset),
        ] if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, horizontal_offset.clone()),
            vertical_edge_offset(*vertical, vertical_offset.clone()),
        ),
        _ => return None,
    };

    Some((horizontal, vertical))
}

fn invalid_background_atom_index(atoms: &[GenericPositionAtom]) -> usize {
    if atoms.len() == 4 && build_background_position(&atoms[..3]).is_some() {
        3
    } else {
        invalid_atom_index(atoms)
    }
}

fn build_background_position(atoms: &[GenericPositionAtom]) -> Option<CssBackgroundPosition> {
    use GenericPositionAtom::{Center, Horizontal, Offset, Vertical};

    let (horizontal, vertical) = match atoms {
        [Horizontal(horizontal), Offset(offset), Vertical(vertical)]
            if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) =>
        {
            (
                horizontal_edge_offset(*horizontal, offset.clone()),
                vertical_keyword(*vertical),
            )
        }
        [Horizontal(horizontal), Offset(offset), Center] if is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, offset.clone()),
            CssVerticalPosition::Center,
        ),
        [Vertical(vertical), Offset(offset), Horizontal(horizontal)]
            if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) =>
        {
            (
                horizontal_keyword(*horizontal),
                vertical_edge_offset(*vertical, offset.clone()),
            )
        }
        [Vertical(vertical), Offset(offset), Center] if is_vertical_edge(*vertical) => (
            CssHorizontalPosition::Center,
            vertical_edge_offset(*vertical, offset.clone()),
        ),
        [Horizontal(horizontal), Vertical(vertical), Offset(offset)]
            if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) =>
        {
            (
                horizontal_keyword(*horizontal),
                vertical_edge_offset(*vertical, offset.clone()),
            )
        }
        [Center, Vertical(vertical), Offset(offset)] if is_vertical_edge(*vertical) => (
            CssHorizontalPosition::Center,
            vertical_edge_offset(*vertical, offset.clone()),
        ),
        [Vertical(vertical), Horizontal(horizontal), Offset(offset)]
            if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) =>
        {
            (
                horizontal_edge_offset(*horizontal, offset.clone()),
                vertical_keyword(*vertical),
            )
        }
        [Center, Horizontal(horizontal), Offset(offset)] if is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, offset.clone()),
            CssVerticalPosition::Center,
        ),
        _ => build_cartesian_axes(atoms)?,
    };

    CssBackgroundPosition::try_new(horizontal, vertical)
}

const fn horizontal_keyword(keyword: CssHorizontalPositionKeyword) -> CssHorizontalPosition {
    match keyword {
        CssHorizontalPositionKeyword::Left => CssHorizontalPosition::Left,
        CssHorizontalPositionKeyword::Center => CssHorizontalPosition::Center,
        CssHorizontalPositionKeyword::Right => CssHorizontalPosition::Right,
        CssHorizontalPositionKeyword::XStart => CssHorizontalPosition::XStart,
        CssHorizontalPositionKeyword::XEnd => CssHorizontalPosition::XEnd,
    }
}

const fn vertical_keyword(keyword: CssVerticalPositionKeyword) -> CssVerticalPosition {
    match keyword {
        CssVerticalPositionKeyword::Top => CssVerticalPosition::Top,
        CssVerticalPositionKeyword::Center => CssVerticalPosition::Center,
        CssVerticalPositionKeyword::Bottom => CssVerticalPosition::Bottom,
        CssVerticalPositionKeyword::YStart => CssVerticalPosition::YStart,
        CssVerticalPositionKeyword::YEnd => CssVerticalPosition::YEnd,
    }
}

const fn is_horizontal_edge(keyword: CssHorizontalPositionKeyword) -> bool {
    matches!(
        keyword,
        CssHorizontalPositionKeyword::Left
            | CssHorizontalPositionKeyword::Right
            | CssHorizontalPositionKeyword::XStart
            | CssHorizontalPositionKeyword::XEnd
    )
}

const fn is_vertical_edge(keyword: CssVerticalPositionKeyword) -> bool {
    matches!(
        keyword,
        CssVerticalPositionKeyword::Top
            | CssVerticalPositionKeyword::Bottom
            | CssVerticalPositionKeyword::YStart
            | CssVerticalPositionKeyword::YEnd
    )
}

fn horizontal_edge_offset(
    keyword: CssHorizontalPositionKeyword,
    offset: CssSpecifiedLengthPercentage,
) -> CssHorizontalPosition {
    match keyword {
        CssHorizontalPositionKeyword::Left => CssHorizontalPosition::LeftOffset(offset),
        CssHorizontalPositionKeyword::Right => CssHorizontalPosition::RightOffset(offset),
        CssHorizontalPositionKeyword::XStart => CssHorizontalPosition::XStartOffset(offset),
        CssHorizontalPositionKeyword::XEnd => CssHorizontalPosition::XEndOffset(offset),
        CssHorizontalPositionKeyword::Center => CssHorizontalPosition::Center,
    }
}

fn vertical_edge_offset(
    keyword: CssVerticalPositionKeyword,
    offset: CssSpecifiedLengthPercentage,
) -> CssVerticalPosition {
    match keyword {
        CssVerticalPositionKeyword::Top => CssVerticalPosition::TopOffset(offset),
        CssVerticalPositionKeyword::Bottom => CssVerticalPosition::BottomOffset(offset),
        CssVerticalPositionKeyword::YStart => CssVerticalPosition::YStartOffset(offset),
        CssVerticalPositionKeyword::YEnd => CssVerticalPosition::YEndOffset(offset),
        CssVerticalPositionKeyword::Center => CssVerticalPosition::Center,
    }
}

fn build_cartesian_position(atoms: &[GenericPositionAtom]) -> Option<CssCartesianPosition> {
    let (horizontal, vertical) = build_cartesian_axes(atoms)?;
    CssCartesianPosition::try_new(horizontal, vertical).ok()
}

fn build_physical_position(atoms: &[GenericPositionAtom]) -> Option<CssPhysicalPosition> {
    CssPhysicalPosition::try_from_cartesian(build_cartesian_position(atoms)?).ok()
}

fn build_full_position(atoms: &[GenericPositionAtom]) -> Option<CssPosition> {
    use GenericPositionAtom::{Block, Center, Inline, Offset, Relative};
    if atoms.iter().any(|atom| matches!(atom, Relative(_))) {
        let component = |atom: &GenericPositionAtom| match atom {
            Relative(FlowEdge::Start) => Some(CssRelativeAxisPosition::Start),
            Relative(FlowEdge::End) => Some(CssRelativeAxisPosition::End),
            Center => Some(CssRelativeAxisPosition::Center),
            _ => None,
        };
        let offset = |edge: FlowEdge, value: &CssSpecifiedLengthPercentage| {
            if matches!(edge, FlowEdge::Start) {
                CssRelativeAxisPosition::StartOffset(value.clone())
            } else {
                CssRelativeAxisPosition::EndOffset(value.clone())
            }
        };
        let (block, inline) = match atoms {
            [block, inline] => (component(block)?, component(inline)?),
            [
                Relative(block),
                Offset(block_offset),
                Relative(inline),
                Offset(inline_offset),
            ] => (offset(*block, block_offset), offset(*inline, inline_offset)),
            _ => return None,
        };
        return CssPosition::try_from_relative_axes(block, inline).ok();
    }
    if atoms
        .iter()
        .any(|atom| matches!(atom, Block(_) | Inline(_)))
    {
        let block_keyword = |edge: FlowEdge| {
            if matches!(edge, FlowEdge::Start) {
                CssBlockPosition::Start
            } else {
                CssBlockPosition::End
            }
        };
        let inline_keyword = |edge: FlowEdge| {
            if matches!(edge, FlowEdge::Start) {
                CssInlinePosition::Start
            } else {
                CssInlinePosition::End
            }
        };
        let (block, inline) = match atoms {
            [Block(block)] | [Block(block), Center] | [Center, Block(block)] => {
                (block_keyword(*block), CssInlinePosition::Center)
            }
            [Inline(inline)] | [Inline(inline), Center] | [Center, Inline(inline)] => {
                (CssBlockPosition::Center, inline_keyword(*inline))
            }
            [Block(block), Inline(inline)] | [Inline(inline), Block(block)] => {
                (block_keyword(*block), inline_keyword(*inline))
            }
            [
                Block(block),
                Offset(block_offset),
                Inline(inline),
                Offset(inline_offset),
            ]
            | [
                Inline(inline),
                Offset(inline_offset),
                Block(block),
                Offset(block_offset),
            ] => (
                if matches!(block, FlowEdge::Start) {
                    CssBlockPosition::StartOffset(block_offset.clone())
                } else {
                    CssBlockPosition::EndOffset(block_offset.clone())
                },
                if matches!(inline, FlowEdge::Start) {
                    CssInlinePosition::StartOffset(inline_offset.clone())
                } else {
                    CssInlinePosition::EndOffset(inline_offset.clone())
                },
            ),
            _ => return None,
        };
        return CssPosition::try_from_named_axes(block, inline).ok();
    }
    build_cartesian_position(atoms).map(CssPosition::from_cartesian)
}
