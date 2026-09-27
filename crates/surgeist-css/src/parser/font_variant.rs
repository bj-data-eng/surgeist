use crate::CssFontFeatureValueName;
use crate::error::{Error, basic, unsupported_value};
use crate::font_variant::*;
use cssparser::{ParseError, Parser, Token};

#[derive(Default)]
struct FontVariantComponents {
    common: Option<CssFontVariantLigatureState>,
    discretionary: Option<CssFontVariantLigatureState>,
    historical: Option<CssFontVariantLigatureState>,
    contextual: Option<CssFontVariantLigatureState>,
    position: Option<CssFontVariantPosition>,
    caps: Option<CssFontVariantCaps>,
    figure: Option<CssFontVariantNumericFigure>,
    spacing: Option<CssFontVariantNumericSpacing>,
    fraction: Option<CssFontVariantNumericFraction>,
    ordinal: bool,
    slashed_zero: bool,
    east_asian_variant: Option<CssFontVariantEastAsianVariant>,
    east_asian_width: Option<CssFontVariantEastAsianWidth>,
    ruby: bool,
    alternates: AlternateComponents,
    emoji: Option<CssFontVariantEmoji>,
}

#[derive(Default)]
struct AlternateComponents {
    stylistic: Option<CssFontFeatureValueName>,
    historical_forms: bool,
    styleset: Option<CssFontVariantAlternateNames>,
    character_variant: Option<CssFontVariantAlternateNames>,
    swash: Option<CssFontFeatureValueName>,
    ornaments: Option<CssFontFeatureValueName>,
    annotation: Option<CssFontFeatureValueName>,
}

impl AlternateComponents {
    fn apply_keyword(&mut self, ident: &str) -> Option<bool> {
        ident
            .eq_ignore_ascii_case("historical-forms")
            .then(|| set_flag_once(&mut self.historical_forms))
    }

    fn apply_function<'i, 't>(
        &mut self,
        name: &str,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Option<bool>, ParseError<'i, Error>> {
        let multi =
            name.eq_ignore_ascii_case("styleset") || name.eq_ignore_ascii_case("character-variant");
        if !multi
            && !["stylistic", "swash", "ornaments", "annotation"]
                .iter()
                .any(|known| name.eq_ignore_ascii_case(known))
        {
            return Ok(None);
        }
        let names = input.parse_nested_block(|nested| {
            let mut names = Vec::new();
            loop {
                let ident = nested.expect_ident_cloned().map_err(basic)?;
                let name = CssFontFeatureValueName::try_new(ident.as_ref()).map_err(|_| {
                    unsupported_value(nested, None, "invalid font feature value name")
                })?;
                names.push(name);
                if nested.is_exhausted() {
                    break;
                }
                nested.expect_comma().map_err(basic)?;
            }
            if !multi && names.len() != 1 {
                return Err(unsupported_value(
                    nested,
                    None,
                    "one font feature value name required",
                ));
            }
            Ok(names)
        })?;
        let applied = if multi {
            let values = CssFontVariantAlternateNames::try_new(names)
                .expect("name parser collected at least one name");
            if name.eq_ignore_ascii_case("styleset") {
                set_once(&mut self.styleset, values)
            } else {
                set_once(&mut self.character_variant, values)
            }
        } else {
            let value = names.into_iter().next().expect("single name required");
            if name.eq_ignore_ascii_case("stylistic") {
                set_once(&mut self.stylistic, value)
            } else if name.eq_ignore_ascii_case("swash") {
                set_once(&mut self.swash, value)
            } else if name.eq_ignore_ascii_case("ornaments") {
                set_once(&mut self.ornaments, value)
            } else {
                set_once(&mut self.annotation, value)
            }
        };
        Ok(Some(applied))
    }

    fn finish(self) -> Option<CssFontVariantAlternateValues> {
        CssFontVariantAlternateValues::try_new(
            self.stylistic,
            self.historical_forms,
            self.styleset,
            self.character_variant,
            self.swash,
            self.ornaments,
            self.annotation,
        )
    }
}

impl FontVariantComponents {
    fn apply_ligature(&mut self, ident: &str) -> Option<bool> {
        if ident.eq_ignore_ascii_case("common-ligatures") {
            Some(set_once(
                &mut self.common,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-common-ligatures") {
            Some(set_once(
                &mut self.common,
                CssFontVariantLigatureState::Disabled,
            ))
        } else if ident.eq_ignore_ascii_case("discretionary-ligatures") {
            Some(set_once(
                &mut self.discretionary,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-discretionary-ligatures") {
            Some(set_once(
                &mut self.discretionary,
                CssFontVariantLigatureState::Disabled,
            ))
        } else if ident.eq_ignore_ascii_case("historical-ligatures") {
            Some(set_once(
                &mut self.historical,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-historical-ligatures") {
            Some(set_once(
                &mut self.historical,
                CssFontVariantLigatureState::Disabled,
            ))
        } else if ident.eq_ignore_ascii_case("contextual") {
            Some(set_once(
                &mut self.contextual,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-contextual") {
            Some(set_once(
                &mut self.contextual,
                CssFontVariantLigatureState::Disabled,
            ))
        } else {
            None
        }
    }

    fn apply_caps(&mut self, ident: &str) -> Option<bool> {
        let value = if ident.eq_ignore_ascii_case("small-caps") {
            CssFontVariantCaps::SmallCaps
        } else if ident.eq_ignore_ascii_case("all-small-caps") {
            CssFontVariantCaps::AllSmallCaps
        } else if ident.eq_ignore_ascii_case("petite-caps") {
            CssFontVariantCaps::PetiteCaps
        } else if ident.eq_ignore_ascii_case("all-petite-caps") {
            CssFontVariantCaps::AllPetiteCaps
        } else if ident.eq_ignore_ascii_case("unicase") {
            CssFontVariantCaps::Unicase
        } else if ident.eq_ignore_ascii_case("titling-caps") {
            CssFontVariantCaps::TitlingCaps
        } else {
            return None;
        };
        Some(set_once(&mut self.caps, value))
    }

    fn apply_position(&mut self, ident: &str) -> Option<bool> {
        let value = if ident.eq_ignore_ascii_case("sub") {
            CssFontVariantPosition::Sub
        } else if ident.eq_ignore_ascii_case("super") {
            CssFontVariantPosition::Super
        } else {
            return None;
        };
        Some(set_once(&mut self.position, value))
    }

    fn apply_numeric(&mut self, ident: &str) -> Option<bool> {
        if ident.eq_ignore_ascii_case("lining-nums") {
            Some(set_once(
                &mut self.figure,
                CssFontVariantNumericFigure::LiningNums,
            ))
        } else if ident.eq_ignore_ascii_case("oldstyle-nums") {
            Some(set_once(
                &mut self.figure,
                CssFontVariantNumericFigure::OldstyleNums,
            ))
        } else if ident.eq_ignore_ascii_case("proportional-nums") {
            Some(set_once(
                &mut self.spacing,
                CssFontVariantNumericSpacing::ProportionalNums,
            ))
        } else if ident.eq_ignore_ascii_case("tabular-nums") {
            Some(set_once(
                &mut self.spacing,
                CssFontVariantNumericSpacing::TabularNums,
            ))
        } else if ident.eq_ignore_ascii_case("diagonal-fractions") {
            Some(set_once(
                &mut self.fraction,
                CssFontVariantNumericFraction::DiagonalFractions,
            ))
        } else if ident.eq_ignore_ascii_case("stacked-fractions") {
            Some(set_once(
                &mut self.fraction,
                CssFontVariantNumericFraction::StackedFractions,
            ))
        } else if ident.eq_ignore_ascii_case("ordinal") {
            Some(set_flag_once(&mut self.ordinal))
        } else if ident.eq_ignore_ascii_case("slashed-zero") {
            Some(set_flag_once(&mut self.slashed_zero))
        } else {
            None
        }
    }

    fn apply_east_asian(&mut self, ident: &str) -> Option<bool> {
        let variant = if ident.eq_ignore_ascii_case("jis78") {
            Some(CssFontVariantEastAsianVariant::Jis78)
        } else if ident.eq_ignore_ascii_case("jis83") {
            Some(CssFontVariantEastAsianVariant::Jis83)
        } else if ident.eq_ignore_ascii_case("jis90") {
            Some(CssFontVariantEastAsianVariant::Jis90)
        } else if ident.eq_ignore_ascii_case("jis04") {
            Some(CssFontVariantEastAsianVariant::Jis04)
        } else if ident.eq_ignore_ascii_case("simplified") {
            Some(CssFontVariantEastAsianVariant::Simplified)
        } else if ident.eq_ignore_ascii_case("traditional") {
            Some(CssFontVariantEastAsianVariant::Traditional)
        } else {
            None
        };
        if let Some(variant) = variant {
            return Some(set_once(&mut self.east_asian_variant, variant));
        }

        let width = if ident.eq_ignore_ascii_case("full-width") {
            Some(CssFontVariantEastAsianWidth::FullWidth)
        } else if ident.eq_ignore_ascii_case("proportional-width") {
            Some(CssFontVariantEastAsianWidth::ProportionalWidth)
        } else {
            None
        };
        if let Some(width) = width {
            return Some(set_once(&mut self.east_asian_width, width));
        }

        ident
            .eq_ignore_ascii_case("ruby")
            .then(|| set_flag_once(&mut self.ruby))
    }

    fn ligatures(&self) -> Option<CssFontVariantLigatureValues> {
        CssFontVariantLigatureValues::try_new(
            self.common,
            self.discretionary,
            self.historical,
            self.contextual,
        )
    }

    fn numeric(&self) -> Option<CssFontVariantNumericValues> {
        CssFontVariantNumericValues::try_new(
            self.figure,
            self.spacing,
            self.fraction,
            self.ordinal,
            self.slashed_zero,
        )
    }

    fn east_asian(&self) -> Option<CssFontVariantEastAsianValues> {
        CssFontVariantEastAsianValues::try_new(
            self.east_asian_variant,
            self.east_asian_width,
            self.ruby,
        )
    }

    fn apply_emoji(&mut self, ident: &str) -> Option<bool> {
        let value = if ident.eq_ignore_ascii_case("text") {
            CssFontVariantEmoji::Text
        } else if ident.eq_ignore_ascii_case("emoji") {
            CssFontVariantEmoji::Emoji
        } else if ident.eq_ignore_ascii_case("unicode") {
            CssFontVariantEmoji::Unicode
        } else {
            return None;
        };
        Some(set_once(&mut self.emoji, value))
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> bool {
    if slot.is_some() {
        false
    } else {
        *slot = Some(value);
        true
    }
}

fn set_flag_once(slot: &mut bool) -> bool {
    if *slot {
        false
    } else {
        *slot = true;
        true
    }
}

fn next_font_variant_ident<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(cssparser::SourceLocation, String), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    Ok((location, ident.as_ref().to_owned()))
}

fn invalid_font_variant_keyword<'i>(
    location: cssparser::SourceLocation,
    ident: String,
) -> ParseError<'i, Error> {
    location.new_unexpected_token_error::<Error>(Token::Ident(ident.into()))
}

pub(super) fn parse_font_variant_caps<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantCaps, ParseError<'i, Error>> {
    let (location, ident) = next_font_variant_ident(input)?;
    let value = if ident.eq_ignore_ascii_case("normal") {
        CssFontVariantCaps::Normal
    } else if ident.eq_ignore_ascii_case("small-caps") {
        CssFontVariantCaps::SmallCaps
    } else if ident.eq_ignore_ascii_case("all-small-caps") {
        CssFontVariantCaps::AllSmallCaps
    } else if ident.eq_ignore_ascii_case("petite-caps") {
        CssFontVariantCaps::PetiteCaps
    } else if ident.eq_ignore_ascii_case("all-petite-caps") {
        CssFontVariantCaps::AllPetiteCaps
    } else if ident.eq_ignore_ascii_case("unicase") {
        CssFontVariantCaps::Unicase
    } else if ident.eq_ignore_ascii_case("titling-caps") {
        CssFontVariantCaps::TitlingCaps
    } else {
        return Err(invalid_font_variant_keyword(location, ident));
    };
    Ok(value)
}

pub(super) fn parse_font_variant_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantPosition, ParseError<'i, Error>> {
    let (location, ident) = next_font_variant_ident(input)?;
    if ident.eq_ignore_ascii_case("normal") {
        Ok(CssFontVariantPosition::Normal)
    } else if ident.eq_ignore_ascii_case("sub") {
        Ok(CssFontVariantPosition::Sub)
    } else if ident.eq_ignore_ascii_case("super") {
        Ok(CssFontVariantPosition::Super)
    } else {
        Err(invalid_font_variant_keyword(location, ident))
    }
}

pub(super) fn parse_font_variant_ligatures<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantLigatures, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantLigatures::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFontVariantLigatures::None);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        if components.apply_ligature(&ident) != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }
    components
        .ligatures()
        .map(CssFontVariantLigatures::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-ligatures requires a value"))
}

pub(super) fn parse_font_variant_numeric<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantNumeric, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantNumeric::Normal);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        if components.apply_numeric(&ident) != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }
    components
        .numeric()
        .map(CssFontVariantNumeric::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-numeric requires a value"))
}

pub(super) fn parse_font_variant_east_asian<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantEastAsian, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantEastAsian::Normal);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        if components.apply_east_asian(&ident) != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }
    components
        .east_asian()
        .map(CssFontVariantEastAsian::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-east-asian requires a value"))
}

pub(super) fn parse_font_variant_emoji<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantEmoji, ParseError<'i, Error>> {
    let (location, ident) = next_font_variant_ident(input)?;
    if ident.eq_ignore_ascii_case("normal") {
        Ok(CssFontVariantEmoji::Normal)
    } else {
        let mut components = FontVariantComponents::default();
        components
            .apply_emoji(&ident)
            .filter(|accepted| *accepted)
            .and(components.emoji)
            .ok_or_else(|| invalid_font_variant_keyword(location, ident))
    }
}

pub(super) fn parse_font_variant_alternates<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantAlternates, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantAlternates::Normal);
    }
    let mut components = AlternateComponents::default();
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let token = input.next().map_err(basic)?.clone();
        let applied = match token {
            Token::Ident(ref ident) => components.apply_keyword(ident),
            Token::Function(ref name) => components.apply_function(name, input)?,
            _ => None,
        };
        if applied != Some(true) {
            return Err(location.new_unexpected_token_error::<Error>(token));
        }
    }
    components
        .finish()
        .map(CssFontVariantAlternates::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-alternates requires a value"))
}

pub(super) fn parse_font_variant<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantValue::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFontVariantValue::None);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let token = input.next().map_err(basic)?.clone();
        let applied = match &token {
            Token::Ident(ident) => components
                .apply_ligature(ident)
                .or_else(|| components.apply_position(ident))
                .or_else(|| components.apply_caps(ident))
                .or_else(|| components.apply_numeric(ident))
                .or_else(|| components.apply_east_asian(ident))
                .or_else(|| components.alternates.apply_keyword(ident))
                .or_else(|| components.apply_emoji(ident)),
            Token::Function(name) => components.alternates.apply_function(name, input)?,
            _ => None,
        };
        if applied != Some(true) {
            return Err(location.new_unexpected_token_error::<Error>(token));
        }
    }

    CssFontVariantValues::try_new(
        components.ligatures(),
        components.position,
        components.caps,
        components.numeric(),
        components.east_asian(),
        components.alternates.finish(),
        components.emoji,
    )
    .map(CssFontVariantValue::Values)
    .ok_or_else(|| unsupported_value(input, None, "font-variant requires a value"))
}
