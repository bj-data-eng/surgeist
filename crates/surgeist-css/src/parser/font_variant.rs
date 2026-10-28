//! Fonts 4 `font-variant` and its longhands.
//!
//! Each multi-value longhand is one `||` group; the shorthand is a `||` of
//! those whole groups and its single-keyword members. Values 4 §2.2 groupings
//! are significant, so a group's values stay contiguous in the shorthand while
//! their internal order stays free.
use super::combinators::{Combinator, Operand, Tail, group_operand, keyword, parse_group};
use super::property_input::Stream;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use crate::rule::font_feature_values::CssFontFeatureValueName;
use crate::values::font::font_variant::*;
use cssparser::Token;

const LIGATURES: [[(&str, CssFontVariantLigatureState); 2]; 4] = [
    [
        ("common-ligatures", CssFontVariantLigatureState::Enabled),
        ("no-common-ligatures", CssFontVariantLigatureState::Disabled),
    ],
    [
        (
            "discretionary-ligatures",
            CssFontVariantLigatureState::Enabled,
        ),
        (
            "no-discretionary-ligatures",
            CssFontVariantLigatureState::Disabled,
        ),
    ],
    [
        ("historical-ligatures", CssFontVariantLigatureState::Enabled),
        (
            "no-historical-ligatures",
            CssFontVariantLigatureState::Disabled,
        ),
    ],
    [
        ("contextual", CssFontVariantLigatureState::Enabled),
        ("no-contextual", CssFontVariantLigatureState::Disabled),
    ],
];
const CAPS: [(&str, CssFontVariantCaps); 6] = [
    ("small-caps", CssFontVariantCaps::SmallCaps),
    ("all-small-caps", CssFontVariantCaps::AllSmallCaps),
    ("petite-caps", CssFontVariantCaps::PetiteCaps),
    ("all-petite-caps", CssFontVariantCaps::AllPetiteCaps),
    ("unicase", CssFontVariantCaps::Unicase),
    ("titling-caps", CssFontVariantCaps::TitlingCaps),
];
const POSITIONS: [(&str, CssFontVariantPosition); 2] = [
    ("sub", CssFontVariantPosition::Sub),
    ("super", CssFontVariantPosition::Super),
];
const EMOJI: [(&str, CssFontVariantEmoji); 3] = [
    ("text", CssFontVariantEmoji::Text),
    ("emoji", CssFontVariantEmoji::Emoji),
    ("unicode", CssFontVariantEmoji::Unicode),
];
const FIGURES: [(&str, CssFontVariantNumericFigure); 2] = [
    ("lining-nums", CssFontVariantNumericFigure::LiningNums),
    ("oldstyle-nums", CssFontVariantNumericFigure::OldstyleNums),
];
const SPACINGS: [(&str, CssFontVariantNumericSpacing); 2] = [
    (
        "proportional-nums",
        CssFontVariantNumericSpacing::ProportionalNums,
    ),
    ("tabular-nums", CssFontVariantNumericSpacing::TabularNums),
];
const FRACTIONS: [(&str, CssFontVariantNumericFraction); 2] = [
    (
        "diagonal-fractions",
        CssFontVariantNumericFraction::DiagonalFractions,
    ),
    (
        "stacked-fractions",
        CssFontVariantNumericFraction::StackedFractions,
    ),
];
const EAST_ASIAN_VARIANTS: [(&str, CssFontVariantEastAsianVariant); 6] = [
    ("jis78", CssFontVariantEastAsianVariant::Jis78),
    ("jis83", CssFontVariantEastAsianVariant::Jis83),
    ("jis90", CssFontVariantEastAsianVariant::Jis90),
    ("jis04", CssFontVariantEastAsianVariant::Jis04),
    ("simplified", CssFontVariantEastAsianVariant::Simplified),
    ("traditional", CssFontVariantEastAsianVariant::Traditional),
];
const EAST_ASIAN_WIDTHS: [(&str, CssFontVariantEastAsianWidth); 2] = [
    ("full-width", CssFontVariantEastAsianWidth::FullWidth),
    (
        "proportional-width",
        CssFontVariantEastAsianWidth::ProportionalWidth,
    ),
];

/// `<common-lig-values> || <discretionary-lig-values> ||
/// <historical-lig-values> || <contextual-alt-values>`.
#[derive(Default)]
struct Ligatures([Option<CssFontVariantLigatureState>; 4]);

impl Ligatures {
    fn operand<'i>(&mut self, index: usize, input: &mut Stream<'i>) -> Operand<'i> {
        keyword(&mut self.0[index], input, &LIGATURES[index])
    }

    fn finish(self) -> Option<CssFontVariantLigatureValues> {
        let [common, discretionary, historical, contextual] = self.0;
        CssFontVariantLigatureValues::try_new(common, discretionary, historical, contextual)
    }
}

/// `<numeric-figure-values> || <numeric-spacing-values> ||
/// <numeric-fraction-values> || ordinal || slashed-zero`.
#[derive(Default)]
struct Numeric {
    figure: Option<CssFontVariantNumericFigure>,
    spacing: Option<CssFontVariantNumericSpacing>,
    fraction: Option<CssFontVariantNumericFraction>,
    ordinal: Option<()>,
    slashed_zero: Option<()>,
}

impl Numeric {
    fn operand<'i>(&mut self, index: usize, input: &mut Stream<'i>) -> Operand<'i> {
        match index {
            0 => keyword(&mut self.figure, input, &FIGURES),
            1 => keyword(&mut self.spacing, input, &SPACINGS),
            2 => keyword(&mut self.fraction, input, &FRACTIONS),
            3 => keyword(&mut self.ordinal, input, &[("ordinal", ())]),
            4 => keyword(&mut self.slashed_zero, input, &[("slashed-zero", ())]),
            _ => unreachable!("five numeric operands"),
        }
    }

    fn finish(self) -> Option<CssFontVariantNumericValues> {
        CssFontVariantNumericValues::try_new(
            self.figure,
            self.spacing,
            self.fraction,
            self.ordinal.is_some(),
            self.slashed_zero.is_some(),
        )
    }
}

/// `<east-asian-variant-values> || <east-asian-width-values> || ruby`.
#[derive(Default)]
struct EastAsian {
    variant: Option<CssFontVariantEastAsianVariant>,
    width: Option<CssFontVariantEastAsianWidth>,
    ruby: Option<()>,
}

impl EastAsian {
    fn operand<'i>(&mut self, index: usize, input: &mut Stream<'i>) -> Operand<'i> {
        match index {
            0 => keyword(&mut self.variant, input, &EAST_ASIAN_VARIANTS),
            1 => keyword(&mut self.width, input, &EAST_ASIAN_WIDTHS),
            2 => keyword(&mut self.ruby, input, &[("ruby", ())]),
            _ => unreachable!("three east-asian operands"),
        }
    }

    fn finish(self) -> Option<CssFontVariantEastAsianValues> {
        CssFontVariantEastAsianValues::try_new(self.variant, self.width, self.ruby.is_some())
    }
}

/// `stylistic() || historical-forms || styleset() || character-variant() ||
/// swash() || ornaments() || annotation()`.
#[derive(Default)]
struct Alternates {
    stylistic: Option<CssFontFeatureValueName>,
    historical_forms: Option<()>,
    styleset: Option<CssFontVariantAlternateNames>,
    character_variant: Option<CssFontVariantAlternateNames>,
    swash: Option<CssFontFeatureValueName>,
    ornaments: Option<CssFontFeatureValueName>,
    annotation: Option<CssFontFeatureValueName>,
}

impl Alternates {
    fn operand<'i>(&mut self, index: usize, input: &mut Stream<'i>) -> Operand<'i> {
        match index {
            0 => single_name(&mut self.stylistic, input, "stylistic"),
            1 => keyword(
                &mut self.historical_forms,
                input,
                &[("historical-forms", ())],
            ),
            2 => name_list(&mut self.styleset, input, "styleset"),
            3 => name_list(&mut self.character_variant, input, "character-variant"),
            4 => single_name(&mut self.swash, input, "swash"),
            5 => single_name(&mut self.ornaments, input, "ornaments"),
            6 => single_name(&mut self.annotation, input, "annotation"),
            _ => unreachable!("seven alternates operands"),
        }
    }

    fn finish(self) -> Option<CssFontVariantAlternateValues> {
        CssFontVariantAlternateValues::try_new(
            self.stylistic,
            self.historical_forms.is_some(),
            self.styleset,
            self.character_variant,
            self.swash,
            self.ornaments,
            self.annotation,
        )
    }
}

/// A feature-value function commits once its name matches; malformed
/// arguments are the value's failure.
fn feature_names<'i>(
    input: &mut Stream<'i>,
    function: &str,
    multiple: bool,
) -> std::result::Result<Option<Vec<CssFontFeatureValueName>>, SyntaxError<'i, Error>> {
    if !matches!(input.consume_after_whitespace(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case(function))
    {
        return Ok(None);
    }
    input
        .within_block(|nested| {
            let mut names = Vec::new();
            loop {
                nested.discard_whitespace();
                let location = nested.site();
                let ident = nested.consume_ident().map_err(basic)?;
                let name = CssFontFeatureValueName::try_new(ident.as_ref())
                    .map_err(|_| unexpected_at(location))?;
                names.push(name);
                if nested.only_whitespace_remains() {
                    break;
                }
                // Fonts 4: a single-name function ends after its name, so the
                // token after it is responsible.
                if !multiple {
                    return Err(unexpected_at(nested.site()));
                }
                nested.consume_comma().map_err(basic)?;
            }
            Ok(names)
        })
        .map(Some)
}

fn single_name<'i>(
    slot: &mut Option<CssFontFeatureValueName>,
    input: &mut Stream<'i>,
    function: &str,
) -> Operand<'i> {
    let Some(names) = feature_names(input, function, false)? else {
        return Ok(false);
    };
    *slot = names.into_iter().next();
    Ok(true)
}

fn name_list<'i>(
    slot: &mut Option<CssFontVariantAlternateNames>,
    input: &mut Stream<'i>,
    function: &str,
) -> Operand<'i> {
    let Some(names) = feature_names(input, function, true)? else {
        return Ok(false);
    };
    *slot = CssFontVariantAlternateNames::try_new(names);
    Ok(true)
}

fn next_font_variant_ident<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<(super::property_input::Site<'i>, String), SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    Ok((location, ident.as_ref().to_owned()))
}

fn invalid_font_variant_keyword<'i>(
    location: super::property_input::Site<'i>,
    ident: String,
) -> SyntaxError<'i, Error> {
    location.unexpected::<Error>(Token::Ident(ident.into()))
}

fn keyword_or_normal<'i, T: Copy>(
    input: &mut Stream<'i>,
    normal: T,
    table: &[(&str, T)],
) -> std::result::Result<T, SyntaxError<'i, Error>> {
    let (location, ident) = next_font_variant_ident(input)?;
    if ident.eq_ignore_ascii_case("normal") {
        return Ok(normal);
    }
    table
        .iter()
        .find(|(name, _)| ident.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
        .ok_or_else(|| invalid_font_variant_keyword(location, ident))
}

pub(crate) fn parse_font_variant_caps<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantCaps, SyntaxError<'i, Error>> {
    keyword_or_normal(input, CssFontVariantCaps::Normal, &CAPS)
}

pub(crate) fn parse_font_variant_position<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantPosition, SyntaxError<'i, Error>> {
    keyword_or_normal(input, CssFontVariantPosition::Normal, &POSITIONS)
}

pub(crate) fn parse_font_variant_emoji<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantEmoji, SyntaxError<'i, Error>> {
    keyword_or_normal(input, CssFontVariantEmoji::Normal, &EMOJI)
}

fn next_is_keyword<'i>(input: &mut Stream<'i>, keyword: &str) -> bool {
    input
        .attempt(|input| input.consume_keyword(keyword))
        .is_ok()
}

pub(crate) fn parse_font_variant_ligatures<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantLigatures, SyntaxError<'i, Error>> {
    if next_is_keyword(input, "normal") {
        return Ok(CssFontVariantLigatures::Normal);
    }
    if next_is_keyword(input, "none") {
        return Ok(CssFontVariantLigatures::None);
    }
    let mut ligatures = Ligatures::default();
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| ligatures.operand(operand, input),
    )?;
    ligatures
        .finish()
        .map(CssFontVariantLigatures::Values)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_font_variant_numeric<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantNumeric, SyntaxError<'i, Error>> {
    if next_is_keyword(input, "normal") {
        return Ok(CssFontVariantNumeric::Normal);
    }
    let mut numeric = Numeric::default();
    parse_group::<5>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| numeric.operand(operand, input),
    )?;
    numeric
        .finish()
        .map(CssFontVariantNumeric::Values)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_font_variant_east_asian<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantEastAsian, SyntaxError<'i, Error>> {
    if next_is_keyword(input, "normal") {
        return Ok(CssFontVariantEastAsian::Normal);
    }
    let mut east_asian = EastAsian::default();
    parse_group::<3>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| east_asian.operand(operand, input),
    )?;
    east_asian
        .finish()
        .map(CssFontVariantEastAsian::Values)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_font_variant_alternates<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantAlternates, SyntaxError<'i, Error>> {
    if next_is_keyword(input, "normal") {
        return Ok(CssFontVariantAlternates::Normal);
    }
    let mut alternates = Alternates::default();
    parse_group::<7>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| alternates.operand(operand, input),
    )?;
    alternates
        .finish()
        .map(CssFontVariantAlternates::Values)
        .ok_or_else(|| unexpected_at(input.site()))
}

/// `normal | none | [ <ligatures> || <caps> || <alternates> || <numeric> ||
/// <east-asian> || <position> || <emoji> ]`, each bracketed group whole.
pub(crate) fn parse_font_variant<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFontVariantValue, SyntaxError<'i, Error>> {
    if next_is_keyword(input, "normal") {
        return Ok(CssFontVariantValue::Normal);
    }
    if next_is_keyword(input, "none") {
        return Ok(CssFontVariantValue::None);
    }
    let mut ligatures = Ligatures::default();
    let mut numeric = Numeric::default();
    let mut east_asian = EastAsian::default();
    let mut alternates = Alternates::default();
    let (mut position, mut caps, mut emoji) = (None, None, None);
    parse_group::<7>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => group_operand::<4>(
                input,
                |_| false,
                |index, input| ligatures.operand(index, input),
            ),
            1 => keyword(&mut position, input, &POSITIONS),
            2 => keyword(&mut caps, input, &CAPS),
            3 => group_operand::<5>(
                input,
                |_| false,
                |index, input| numeric.operand(index, input),
            ),
            4 => group_operand::<3>(
                input,
                |_| false,
                |index, input| east_asian.operand(index, input),
            ),
            5 => group_operand::<7>(
                input,
                |_| false,
                |index, input| alternates.operand(index, input),
            ),
            6 => keyword(&mut emoji, input, &EMOJI),
            _ => unreachable!("seven font-variant operands"),
        },
    )?;
    CssFontVariantValues::try_new(
        ligatures.finish(),
        position,
        caps,
        numeric.finish(),
        east_asian.finish(),
        alternates.finish(),
        emoji,
    )
    .map(CssFontVariantValue::Values)
    .ok_or_else(|| unexpected_at(input.site()))
}
