use cssparser::{ParseError, Parser, ParserState, Token, match_ignore_ascii_case};

use super::values::{
    LengthGrammar, next_is_comma, next_is_delim, parse_length_with,
    parse_literal_length_with_context,
};
use crate::error::{Error, unsupported_value};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

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
        atoms.push(parse_generic_position_atom(input, numeric)?);
    }
    build_background_position(&atoms).ok_or_else(|| {
        invalid_generic_position_atom(input, &states[invalid_background_atom_index(&atoms)])
    })
}

pub(super) fn parse_mask_position_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssMaskPositionList, ParseError<'i, Error>> {
    let mut positions = Vec::new();
    loop {
        let (current, legacy) = parse_generic_position(input, numeric)?;
        let legacy = (!position_has_calculation(&legacy)).then_some(legacy);
        positions.push(CssMaskPosition::new(current, legacy));
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
    CssMaskPositionList::try_new(positions)
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
) -> std::result::Result<CssObjectPosition, ParseError<'i, Error>> {
    parse_generic_position(input, numeric).map(|(position, _)| CssObjectPosition::new(position))
}

pub(super) fn parse_transform_origin<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransformOrigin, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input, numeric)?;

    if atoms.len() <= 2
        && let Some((position, legacy)) = build_generic_position(&atoms)
    {
        let legacy = (!position_has_calculation(&legacy)).then_some(legacy);
        return Ok(CssTransformOrigin::new(position, None, legacy));
    }

    if (2..=3).contains(&atoms.len()) {
        let z_index = atoms.len() - 1;
        if let Some((position, _)) = build_generic_position(&atoms[..z_index]) {
            let z = transform_origin_z(&atoms[z_index])
                .ok_or_else(|| invalid_generic_position_atom(input, &states[z_index]))?;
            let legacy = CssPosition::new(contextual_legacy_components(&atoms));
            let legacy = (!position_has_calculation(&legacy)).then_some(legacy);
            return Ok(CssTransformOrigin::new(position, Some(z), legacy));
        }
    }

    let invalid_index = if atoms.len() > 3 {
        3
    } else {
        invalid_atom_index(&atoms)
    };
    Err(invalid_generic_position_atom(input, &states[invalid_index]))
}

pub(super) fn parse_css_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    parse_generic_position(input, numeric).map(|(_, legacy)| legacy)
}

pub(super) fn parse_css_position_legacy<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    parse_css_position_legacy_components(input)
}

fn parse_css_position_legacy_components<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    let mut components = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) && !next_is_delim(input, '/') {
        components.push(parse_legacy_position_component(input, &components)?);
        if components.len() > 4 {
            return Err(unsupported_value(
                input,
                None,
                "position has too many components",
            ));
        }
    }
    CssPosition::try_new(components)
        .ok_or_else(|| unsupported_value(input, None, "position is empty"))
}

fn parse_legacy_position_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    previous: &[CssPositionComponent],
) -> std::result::Result<CssPositionComponent, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "left" => Ok(CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Left)),
            "right" => Ok(CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Right)),
            "top" => Ok(CssPositionComponent::Vertical(CssVerticalPositionKeyword::Top)),
            "bottom" => Ok(CssPositionComponent::Vertical(CssVerticalPositionKeyword::Bottom)),
            "center" => {
                let has_horizontal = previous.iter().any(|component| matches!(component, CssPositionComponent::Horizontal(_)));
                if has_horizontal {
                    Ok(CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center))
                } else {
                    Ok(CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center))
                }
            },
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("position", ident.as_ref()),
            )),
        };
    }
    input.reset(&state);
    parse_literal_length_with_context(input, LengthGrammar::Position, "position")
        .map(CssPositionComponent::Length)
}

#[derive(Clone, Debug)]
enum GenericPositionAtom {
    Horizontal(CssHorizontalPositionKeyword),
    Vertical(CssVerticalPositionKeyword),
    Center,
    Offset(CssPositionOffset),
}

fn parse_generic_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<(CssPositionValue, CssPosition), ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input, numeric)?;
    build_generic_position(&atoms)
        .ok_or_else(|| invalid_generic_position_atom(input, &states[invalid_atom_index(&atoms)]))
}

fn parse_position_atoms<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<(Vec<GenericPositionAtom>, Vec<ParserState>), ParseError<'i, Error>> {
    let mut atoms = Vec::new();
    let mut states = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) && !next_is_delim(input, '/') {
        states.push(input.state());
        atoms.push(parse_generic_position_atom(input, numeric)?);
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
    let (atoms, states) = parse_position_atoms(input, numeric)?;
    build_background_position(&atoms).ok_or_else(|| {
        invalid_generic_position_atom(input, &states[invalid_background_atom_index(&atoms)])
    })
}

pub(super) fn parse_css_position_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPositionValue, ParseError<'i, Error>> {
    parse_generic_position(input, numeric).map(|(current, _)| current)
}

fn parse_generic_position_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<GenericPositionAtom, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
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
    let value = parse_length_with(input, numeric, LengthGrammar::Position)?;
    let Some(offset) = CssPositionOffset::try_new(value) else {
        return Err(invalid_generic_position_atom(input, &state));
    };
    Ok(GenericPositionAtom::Offset(offset))
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

fn transform_origin_z(atom: &GenericPositionAtom) -> Option<CssTransformOriginZ> {
    let GenericPositionAtom::Offset(offset) = atom else {
        return None;
    };
    CssTransformOriginZ::try_new(offset.value().clone())
}

fn invalid_atom_index(atoms: &[GenericPositionAtom]) -> usize {
    match atoms.len() {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => {
            if build_generic_position(&atoms[..2]).is_some() {
                2
            } else if !matches!(
                atoms[0],
                GenericPositionAtom::Horizontal(
                    CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right
                ) | GenericPositionAtom::Vertical(
                    CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom
                )
            ) {
                0
            } else if !matches!(atoms[1], GenericPositionAtom::Offset(_)) {
                1
            } else if !matches!(
                atoms[2],
                GenericPositionAtom::Horizontal(
                    CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right
                ) | GenericPositionAtom::Vertical(
                    CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom
                )
            ) {
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

fn build_generic_position(
    atoms: &[GenericPositionAtom],
) -> Option<(CssPositionValue, CssPosition)> {
    use GenericPositionAtom::{Center, Horizontal, Offset, Vertical};

    let (horizontal, vertical, components) = match atoms {
        [Horizontal(keyword)] => (
            horizontal_keyword(*keyword),
            CssVerticalPosition::Center,
            vec![CssPositionComponent::Horizontal(*keyword)],
        ),
        [Vertical(keyword)] => (
            CssHorizontalPosition::Center,
            vertical_keyword(*keyword),
            vec![CssPositionComponent::Vertical(*keyword)],
        ),
        [Center] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Center,
            vec![CssPositionComponent::Horizontal(
                CssHorizontalPositionKeyword::Center,
            )],
        ),
        [Offset(offset)] => (
            CssHorizontalPosition::Offset(offset.clone()),
            CssVerticalPosition::Center,
            vec![CssPositionComponent::Length(offset.value().clone())],
        ),
        [Horizontal(horizontal), Vertical(vertical)] => (
            horizontal_keyword(*horizontal),
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Horizontal(*horizontal),
                CssPositionComponent::Vertical(*vertical),
            ],
        ),
        [Vertical(vertical), Horizontal(horizontal)] => (
            horizontal_keyword(*horizontal),
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Vertical(*vertical),
                CssPositionComponent::Horizontal(*horizontal),
            ],
        ),
        [Horizontal(horizontal), Center] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Horizontal(*horizontal),
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
            ],
        ),
        [Center, Horizontal(horizontal)] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
                CssPositionComponent::Horizontal(*horizontal),
            ],
        ),
        [Vertical(vertical), Center] => (
            CssHorizontalPosition::Center,
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Vertical(*vertical),
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
            ],
        ),
        [Center, Vertical(vertical)] => (
            CssHorizontalPosition::Center,
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
                CssPositionComponent::Vertical(*vertical),
            ],
        ),
        [Center, Center] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
            ],
        ),
        [Horizontal(horizontal), Offset(offset)] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Offset(offset.clone()),
            vec![
                CssPositionComponent::Horizontal(*horizontal),
                CssPositionComponent::Length(offset.value().clone()),
            ],
        ),
        [Center, Offset(offset)] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Offset(offset.clone()),
            vec![
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
                CssPositionComponent::Length(offset.value().clone()),
            ],
        ),
        [Offset(offset), Vertical(vertical)] => (
            CssHorizontalPosition::Offset(offset.clone()),
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Length(offset.value().clone()),
                CssPositionComponent::Vertical(*vertical),
            ],
        ),
        [Offset(offset), Center] => (
            CssHorizontalPosition::Offset(offset.clone()),
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Length(offset.value().clone()),
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
            ],
        ),
        [Offset(horizontal), Offset(vertical)] => (
            CssHorizontalPosition::Offset(horizontal.clone()),
            CssVerticalPosition::Offset(vertical.clone()),
            vec![
                CssPositionComponent::Length(horizontal.value().clone()),
                CssPositionComponent::Length(vertical.value().clone()),
            ],
        ),
        [
            Horizontal(horizontal),
            Offset(horizontal_offset),
            Vertical(vertical),
            Offset(vertical_offset),
        ] if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) => (
            horizontal_edge_offset(*horizontal, horizontal_offset.clone()),
            vertical_edge_offset(*vertical, vertical_offset.clone()),
            legacy_components(atoms),
        ),
        [
            Vertical(vertical),
            Offset(vertical_offset),
            Horizontal(horizontal),
            Offset(horizontal_offset),
        ] if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, horizontal_offset.clone()),
            vertical_edge_offset(*vertical, vertical_offset.clone()),
            legacy_components(atoms),
        ),
        _ => return None,
    };

    Some((
        CssPositionValue::new(horizontal, vertical),
        CssPosition::new(components),
    ))
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
        _ => {
            let (position, legacy) = build_generic_position(atoms)?;
            let legacy = (!position_has_calculation(&legacy)).then_some(legacy);
            return Some(CssBackgroundPosition::new(
                position.horizontal().clone(),
                position.vertical().clone(),
                legacy,
            ));
        }
    };

    let legacy = CssPosition::new(contextual_legacy_components(atoms));
    let legacy = (!position_has_calculation(&legacy)).then_some(legacy);
    Some(CssBackgroundPosition::new(horizontal, vertical, legacy))
}

fn contextual_legacy_components(atoms: &[GenericPositionAtom]) -> Vec<CssPositionComponent> {
    let mut components = Vec::with_capacity(atoms.len());
    for atom in atoms {
        let component = match atom {
            GenericPositionAtom::Horizontal(keyword) => CssPositionComponent::Horizontal(*keyword),
            GenericPositionAtom::Vertical(keyword) => CssPositionComponent::Vertical(*keyword),
            GenericPositionAtom::Center => {
                if components
                    .iter()
                    .any(|component| matches!(component, CssPositionComponent::Horizontal(_)))
                {
                    CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center)
                } else {
                    CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center)
                }
            }
            GenericPositionAtom::Offset(offset) => {
                CssPositionComponent::Length(offset.value().clone())
            }
        };
        components.push(component);
    }
    components
}

fn position_has_calculation(position: &CssPosition) -> bool {
    position
        .components()
        .iter()
        .any(|component| matches!(component, CssPositionComponent::Length(CssLength::Calc(_))))
}

fn legacy_components(atoms: &[GenericPositionAtom]) -> Vec<CssPositionComponent> {
    atoms
        .iter()
        .map(|atom| match atom {
            GenericPositionAtom::Horizontal(keyword) => CssPositionComponent::Horizontal(*keyword),
            GenericPositionAtom::Vertical(keyword) => CssPositionComponent::Vertical(*keyword),
            GenericPositionAtom::Center => {
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center)
            }
            GenericPositionAtom::Offset(offset) => {
                CssPositionComponent::Length(offset.value().clone())
            }
        })
        .collect()
}

const fn horizontal_keyword(keyword: CssHorizontalPositionKeyword) -> CssHorizontalPosition {
    match keyword {
        CssHorizontalPositionKeyword::Left => CssHorizontalPosition::Left,
        CssHorizontalPositionKeyword::Center => CssHorizontalPosition::Center,
        CssHorizontalPositionKeyword::Right => CssHorizontalPosition::Right,
    }
}

const fn vertical_keyword(keyword: CssVerticalPositionKeyword) -> CssVerticalPosition {
    match keyword {
        CssVerticalPositionKeyword::Top => CssVerticalPosition::Top,
        CssVerticalPositionKeyword::Center => CssVerticalPosition::Center,
        CssVerticalPositionKeyword::Bottom => CssVerticalPosition::Bottom,
    }
}

const fn is_horizontal_edge(keyword: CssHorizontalPositionKeyword) -> bool {
    matches!(
        keyword,
        CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right
    )
}

const fn is_vertical_edge(keyword: CssVerticalPositionKeyword) -> bool {
    matches!(
        keyword,
        CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom
    )
}

fn horizontal_edge_offset(
    keyword: CssHorizontalPositionKeyword,
    offset: CssPositionOffset,
) -> CssHorizontalPosition {
    match keyword {
        CssHorizontalPositionKeyword::Left => CssHorizontalPosition::LeftOffset(offset),
        CssHorizontalPositionKeyword::Right => CssHorizontalPosition::RightOffset(offset),
        CssHorizontalPositionKeyword::Center => CssHorizontalPosition::Center,
    }
}

fn vertical_edge_offset(
    keyword: CssVerticalPositionKeyword,
    offset: CssPositionOffset,
) -> CssVerticalPosition {
    match keyword {
        CssVerticalPositionKeyword::Top => CssVerticalPosition::TopOffset(offset),
        CssVerticalPositionKeyword::Bottom => CssVerticalPosition::BottomOffset(offset),
        CssVerticalPositionKeyword::Center => CssVerticalPosition::Center,
    }
}
