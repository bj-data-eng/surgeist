use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use crate::query::container::{
    CssContainer, CssContainerName, CssContainerNameList, CssContainerNames, CssContainerType,
};

pub(super) fn parse_container_type<'i>(
    input: &mut Stream<'i>,
) -> Result<CssContainerType, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let first = input.consume_ident().map_err(basic)?;
    let initial = match first.to_ascii_lowercase().as_str() {
        "normal" => CssContainerType::Normal,
        "size" => CssContainerType::Size,
        "inline-size" => CssContainerType::InlineSize,
        "scroll-state" => CssContainerType::ScrollState,
        _ => {
            return Err(unexpected_at(keyword_location));
        }
    };
    if input.only_whitespace_remains() {
        return Ok(initial);
    }
    let keyword_location = input.site();
    let second = input.consume_ident().map_err(basic)?;
    let combined = match (initial, second.to_ascii_lowercase().as_str()) {
        (CssContainerType::Size, "scroll-state") | (CssContainerType::ScrollState, "size") => {
            CssContainerType::SizeScrollState
        }
        (CssContainerType::InlineSize, "scroll-state")
        | (CssContainerType::ScrollState, "inline-size") => CssContainerType::InlineSizeScrollState,
        _ => {
            return Err(unexpected_at(keyword_location));
        }
    };
    input.require_empty().map_err(basic)?;
    Ok(combined)
}

fn names<'i>(
    input: &mut Stream<'i>,
    slash_ends: bool,
) -> Result<CssContainerNames, SyntaxError<'i, Error>> {
    let mut names = Vec::new();
    let mut none = false;
    while !input.only_whitespace_remains() {
        if slash_ends {
            let state = input.mark();
            let slash = input.attempt(|input| input.consume_delim('/')).is_ok();
            input.restore(state);
            if slash {
                break;
            }
        }
        let keyword_location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        if none {
            return Err(unexpected_at(keyword_location));
        }
        if ident.eq_ignore_ascii_case("none") && names.is_empty() {
            none = true;
            continue;
        }
        let name = CssContainerName::try_from_decoded(ident.to_string())
            .ok_or_else(|| unexpected_at(keyword_location))?;
        names.push(name);
    }
    if none {
        return Ok(CssContainerNames::None);
    }
    CssContainerNameList::try_new(names)
        .map(CssContainerNames::Names)
        .ok_or_else(|| unexpected_at(input.site()))
}
pub(super) fn parse_container_names<'i>(
    input: &mut Stream<'i>,
) -> Result<CssContainerNames, SyntaxError<'i, Error>> {
    names(input, false)
}
pub(super) fn parse_container<'i>(
    input: &mut Stream<'i>,
) -> Result<CssContainer, SyntaxError<'i, Error>> {
    let names = names(input, true)?;
    let kind = if input.only_whitespace_remains() {
        CssContainerType::Normal
    } else {
        input.consume_delim('/').map_err(basic)?;
        parse_container_type(input)?
    };
    Ok(CssContainer::new(names, kind))
}
