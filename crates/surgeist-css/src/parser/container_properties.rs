use crate::error::{Error, basic, unsupported_value};
use crate::{
    CssContainer, CssContainerName, CssContainerNameList, CssContainerNames, CssContainerType,
};
use cssparser::{ParseError, Parser};

pub(super) fn parse_container_type<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssContainerType, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    let initial = match first.to_ascii_lowercase().as_str() {
        "normal" => CssContainerType::Normal,
        "size" => CssContainerType::Size,
        "inline-size" => CssContainerType::InlineSize,
        "scroll-state" => CssContainerType::ScrollState,
        _ => {
            return Err(unsupported_value(
                input,
                None,
                "expected a container-type keyword",
            ));
        }
    };
    if input.is_exhausted() {
        return Ok(initial);
    }
    let second = input.expect_ident_cloned().map_err(basic)?;
    let combined = match (initial, second.to_ascii_lowercase().as_str()) {
        (CssContainerType::Size, "scroll-state") | (CssContainerType::ScrollState, "size") => {
            CssContainerType::SizeScrollState
        }
        (CssContainerType::InlineSize, "scroll-state")
        | (CssContainerType::ScrollState, "inline-size") => CssContainerType::InlineSizeScrollState,
        _ => {
            return Err(unsupported_value(
                input,
                None,
                "expected one size axis and scroll-state",
            ));
        }
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(combined)
}

fn names<'i, 't>(
    input: &mut Parser<'i, 't>,
    slash_ends: bool,
) -> Result<CssContainerNames, ParseError<'i, Error>> {
    let mut names = Vec::new();
    let mut none = false;
    while !input.is_exhausted() {
        if slash_ends {
            let state = input.state();
            let slash = input.try_parse(|input| input.expect_delim('/')).is_ok();
            input.reset(&state);
            if slash {
                break;
            }
        }
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if none {
            return Err(unsupported_value(
                input,
                None,
                "none must be the entire container-name value",
            ));
        }
        if ident.eq_ignore_ascii_case("none") && names.is_empty() {
            none = true;
            continue;
        }
        let name = CssContainerName::try_from_decoded(ident.to_string()).ok_or_else(|| {
            unsupported_value(input, None, "expected a non-reserved container name")
        })?;
        names.push(name);
    }
    if none {
        return Ok(CssContainerNames::None);
    }
    CssContainerNameList::try_new(names)
        .map(CssContainerNames::Names)
        .ok_or_else(|| unsupported_value(input, None, "expected a container name"))
}
pub(super) fn parse_container_names<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssContainerNames, ParseError<'i, Error>> {
    names(input, false)
}
pub(super) fn parse_container<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssContainer, ParseError<'i, Error>> {
    let names = names(input, true)?;
    let kind = if input.is_exhausted() {
        CssContainerType::Normal
    } else {
        input.expect_delim('/').map_err(basic)?;
        parse_container_type(input)?
    };
    Ok(CssContainer::new(names, kind))
}
