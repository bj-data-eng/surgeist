//! Selected Color Adjustment 1 property grammars.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::property_input::{SiteExt, StreamExt};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::match_ignore_ascii_case;

use crate::values::color_adjust::{
    CssColorScheme, CssColorSchemeKeyword, CssColorSchemeName, CssForcedColorAdjust,
    CssPrintColorAdjust,
};
use crate::values::common::CssIdent;

struct SchemeBank<'a> {
    values: Vec<CssColorSchemeKeyword>,
    custom_bytes: usize,
    control: Option<&'a crate::values::color_adjust::utf16_value::Control<'a>>,
}
impl SchemeBank<'_> {
    fn finish(mut self) -> Vec<CssColorSchemeKeyword> {
        self.control = None;
        std::mem::take(&mut self.values)
    }
}
impl Drop for SchemeBank<'_> {
    fn drop(&mut self) {
        let bytes = self
            .values
            .capacity()
            .checked_mul(std::mem::size_of::<CssColorSchemeKeyword>())
            .and_then(|bytes| bytes.checked_add(self.custom_bytes));
        drop(std::mem::take(&mut self.values));
        if let Some(control) = self.control {
            let Some(bytes) = bytes else {
                unreachable!("admitted scheme backing")
            };
            control.released(bytes);
        }
    }
}
fn rejected<'i>(location: super::syntax_types::Site<'i>) -> SyntaxError<'i, Error> {
    if location.control().is_some() {
        location.end_of_stream()
    } else {
        unexpected_at(location)
    }
}
pub(super) fn parse_color_scheme<'i>(
    input: &mut Stream<'i>,
) -> Result<CssColorScheme, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("normal"))
        .is_ok()
    {
        return Ok(CssColorScheme::normal());
    }
    let mut schemes = SchemeBank {
        values: Vec::new(),
        custom_bytes: 0,
        control: input.control(),
    };
    let mut only = false;
    while !input.only_whitespace_remains() {
        input.discard_whitespace();
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        input.controlled_step()?;
        if ident.eq_ignore_ascii_case("only") {
            if only || schemes.values.is_empty() && input.only_whitespace_remains() {
                return Err(rejected(location));
            }
            only = true;
            if !schemes.values.is_empty() {
                input.require_empty().map_err(basic)?;
                break;
            }
        } else {
            let scheme = match_ignore_ascii_case! { &ident,
                "light" => CssColorSchemeKeyword::Light,
                "dark" => CssColorSchemeKeyword::Dark,
                _ => {
                    let name = if let Some(control) = input.control() {
                        let owned = control.string(ident.as_ref()).map_err(|_| rejected(location))?;
                        let capacity = owned.capacity();
                        match CssColorSchemeName::from_admitted_ident(CssIdent::from_admitted(owned)) {
                            Some(name) => {
                                let Some(total) = schemes.custom_bytes.checked_add(capacity) else {
                                    let _ = control.fail::<()>(crate::values::color_adjust::utf16_value::Failure::CapacityOverflow);
                                    return Err(rejected(location));
                                };
                                schemes.custom_bytes = total;
                                name
                            }
                            None => { control.released(capacity); return Err(rejected(location)); }
                        }
                    } else { CssColorSchemeName::try_new(CssIdent::new(ident)).map_err(|_| rejected(location))? };
                    CssColorSchemeKeyword::Custom(name)
                },
            };
            input.reserve_controlled(&mut schemes.values, 1)?;
            schemes.values.push(scheme);
        }
    }
    CssColorScheme::try_new(schemes.finish(), only).map_err(|_| rejected(input.site()))
}

pub(super) fn parse_forced_color_adjust<'i>(
    input: &mut Stream<'i>,
) -> Result<CssForcedColorAdjust, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssForcedColorAdjust::Auto),
        "none" => Ok(CssForcedColorAdjust::None),
        "preserve-parent-color" => Ok(CssForcedColorAdjust::PreserveParentColor),
        _ => Err(unexpected_at(location)),
    }
}

pub(super) fn parse_print_color_adjust<'i>(
    input: &mut Stream<'i>,
) -> Result<CssPrintColorAdjust, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "economy" => Ok(CssPrintColorAdjust::Economy),
        "exact" => Ok(CssPrintColorAdjust::Exact),
        _ => Err(unexpected_at(location)),
    }
}
