use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token,
    UnicodeRange as ParsedUnicodeRange, match_ignore_ascii_case,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::typography::{parse_font_feature_settings, parse_non_generic_font_family_name};
use super::{block_item_diagnostic, is_declaration_recovery_unit, parse_descriptor_boundary};
use crate::error::{
    CssFeatureId, Error, basic, descriptor_name_error, invalid_at_rule_body, unsupported_value,
    unsupported_value_at, with_descriptor_context,
};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.rule.font-face")];

pub(super) static IMPLEMENTED_DESCRIPTORS: &[CssFeatureId] = &[
    CssFeatureId::new("baseline.descriptor.font-family"),
    CssFeatureId::new("baseline.descriptor.src"),
    CssFeatureId::new("baseline.descriptor.font-weight"),
    CssFeatureId::new("baseline.descriptor.font-style"),
    CssFeatureId::new("baseline.descriptor.font-stretch"),
    CssFeatureId::new("baseline.descriptor.font-display"),
    CssFeatureId::new("baseline.descriptor.unicode-range"),
    CssFeatureId::new("official.descriptor.font-feature-settings"),
    CssFeatureId::new("ext.descriptor.font-weight-range"),
    CssFeatureId::new("ext.descriptor.font-style-oblique-range"),
    CssFeatureId::new("ext.descriptor.font-stretch-range"),
];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.font-source"),
    CssFeatureId::new("ext.value.font-source-modern-hints"),
];

pub(super) fn parse_font_face_rule<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> std::result::Result<CssFontFaceRule, ParseError<'i, Error>> {
    let mut descriptors = Vec::new();
    let mut descriptor_parser = FontFaceDescriptorParser { source, recovery };

    let mut items = RuleBodyParser::new(input, &mut descriptor_parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        let Some(item) = items.next() else {
            break;
        };
        let retained = item.is_ok();
        let progress_outcome = progress.finish(items.input, retained);
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(descriptor) => {
                descriptors.push(descriptor);
            }
            Err((error, failed_unit)) if is_declaration_recovery_unit(failed_unit) => {
                if let Some(diagnostic) = block_item_diagnostic(
                    source,
                    error,
                    failed_unit,
                    unit_end,
                    crate::CssRecoveryAction::DropDescriptor,
                ) {
                    diagnostics.push(diagnostic);
                }
            }
            Err((error, _)) => return Err(error),
        }
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }

    let descriptors = CssFontFaceDescriptors::from_occurrences(descriptors).ok_or_else(|| {
        invalid_at_rule_body(
            input,
            "font-face",
            "baseline.rule.font-face",
            "font-family and src descriptors",
        )
    })?;

    Ok(CssFontFaceRule::new(
        descriptors,
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
}

struct FontFaceDescriptorParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

impl<'i> AtRuleParser<'i> for FontFaceDescriptorParser<'i> {
    type Prelude = ();
    type AtRule = CssFontFaceDescriptor;
    type Error = Error;
}

impl<'i> QualifiedRuleParser<'i> for FontFaceDescriptorParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssFontFaceDescriptor;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssFontFaceDescriptor, Error> for FontFaceDescriptorParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        false
    }
}

impl<'i> DeclarationParser<'i> for FontFaceDescriptorParser<'i> {
    type Declaration = CssFontFaceDescriptor;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.descriptor")?;
        let position = crate::source::CssSourcePosition::from_cssparser(
            declaration_start.position(),
            declaration_start.source_location(),
        );
        let result = (|| {
            Ok(match_ignore_ascii_case! { &name,
                "font-family" => CssFontFaceDescriptor::FontFamily(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "font-family", parse_font_face_family)?,
                        position,
                    ),
                ),
                "src" => CssFontFaceDescriptor::Src(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "src", parse_font_face_source_list)?,
                        position,
                    ),
                ),
                "font-weight" => CssFontFaceDescriptor::FontWeight(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "font-weight", parse_font_face_weight)?,
                        position,
                    ),
                ),
                "font-style" => CssFontFaceDescriptor::FontStyle(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "font-style", parse_font_face_style)?,
                        position,
                    ),
                ),
                "font-stretch" => CssFontFaceDescriptor::FontStretch(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "font-stretch", parse_font_face_stretch)?,
                        position,
                    ),
                ),
                "font-display" => CssFontFaceDescriptor::FontDisplay(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "font-display", parse_font_display)?,
                        position,
                    ),
                ),
                "unicode-range" => CssFontFaceDescriptor::UnicodeRange(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(input, "font-face", "unicode-range", parse_unicode_range_list)?,
                        position,
                    ),
                ),
                "font-feature-settings" => CssFontFaceDescriptor::FontFeatureSettings(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(
                            input,
                            "font-face",
                            "font-feature-settings",
                            parse_font_feature_settings,
                        )?,
                        position,
                    ),
                ),
                _ => return Err(descriptor_name_error(
                    declaration_start.source_location(),
                    "font-face",
                    name.as_ref(),
                )),
            })
        })()
        .map_err(|error| with_descriptor_context(error, "font-face", name.as_ref()))?;
        self.recovery.retain_component_closures(implicit_closures);
        Ok(result)
    }
}

fn parse_font_face_family<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceFamily, ParseError<'i, Error>> {
    let family = parse_non_generic_font_family_name(input)?;
    CssFontFaceFamily::try_new(family.as_str())
        .ok_or_else(|| unsupported_value(input, None, "font-family descriptor is empty"))
}

fn parse_font_face_source_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceSourceList, ParseError<'i, Error>> {
    let mut sources = Vec::new();
    loop {
        sources.push(parse_font_face_source(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font-face src list has an empty item",
            ));
        }
    }

    CssFontFaceSourceList::try_new(sources)
        .ok_or_else(|| unsupported_value(input, None, "font-face src list is empty"))
}

fn parse_font_face_source<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceSource, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_function_matching("local"))
        .is_ok()
    {
        let name = input.parse_nested_block(parse_local_name)?;
        return CssFontLocalName::try_new(name.as_str())
            .map(CssFontFaceSource::Local)
            .ok_or_else(|| unsupported_value(input, None, "local font name is empty"));
    }

    let url = parse_font_source_url(input)?;
    let mut formats = None;
    let mut tech = Vec::new();
    let mut saw_tech = false;

    while !input.is_exhausted() && !next_is_comma(input) {
        if input
            .try_parse(|input| input.expect_function_matching("format"))
            .is_ok()
        {
            if saw_tech {
                return Err(unsupported_value(
                    input,
                    None,
                    "font source format hint must precede tech hint",
                ));
            }
            if formats.is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "font source has duplicate format hint",
                ));
            }
            formats = Some(input.parse_nested_block(parse_font_format_list)?);
        } else if input
            .try_parse(|input| input.expect_function_matching("tech"))
            .is_ok()
        {
            if saw_tech {
                return Err(unsupported_value(
                    input,
                    None,
                    "font source has duplicate tech hint",
                ));
            }
            tech = input.parse_nested_block(parse_font_tech_hints)?;
            saw_tech = true;
        } else {
            return Err(unsupported_value(
                input,
                None,
                "expected font source format() or tech() hint",
            ));
        }
    }

    CssFontFaceUrlSource::try_new_with_formats(url, formats, tech)
        .map(CssFontFaceSource::Url)
        .ok_or_else(|| unsupported_value(input, None, "font source URL is empty"))
}

fn parse_font_source_url<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<String, ParseError<'i, Error>> {
    input.expect_url().map_err(basic).map(|url| url.to_string())
}

fn parse_local_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFamilyName, ParseError<'i, Error>> {
    let name = parse_non_generic_font_family_name(input)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(name)
}

fn parse_font_format_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFormatList, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let hint = font_format_hint_from_str(ident.as_ref()).ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                format!("unsupported font format hint `{ident}`"),
            )
        })?;
        input.expect_exhausted().map_err(basic)?;
        return Ok(CssFontFormatList::new(vec![CssFontFormatString::new(
            hint.as_str(),
        )]));
    }

    let mut formats = Vec::new();
    loop {
        let item_location = input.current_source_location();
        let value = input.expect_string_cloned().map_err(basic)?;
        let Some(value) = CssFontFormatString::try_new(value.to_string()) else {
            return Err(unsupported_value_at(
                item_location,
                None,
                "font source format string is empty",
            ));
        };
        formats.push(value);
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font source format list has an empty item",
            ));
        }
    }

    Ok(CssFontFormatList::new(formats))
}

fn font_format_hint_from_str(value: &str) -> Option<CssFontFormatHint> {
    match value.to_ascii_lowercase().as_str() {
        "woff" => Some(CssFontFormatHint::Woff),
        "woff2" => Some(CssFontFormatHint::Woff2),
        "truetype" => Some(CssFontFormatHint::TrueType),
        "opentype" => Some(CssFontFormatHint::OpenType),
        "collection" => Some(CssFontFormatHint::Collection),
        "embedded-opentype" => Some(CssFontFormatHint::EmbeddedOpenType),
        "svg" => Some(CssFontFormatHint::Svg),
        _ => None,
    }
}

fn parse_font_tech_hints<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Vec<CssFontTechHint>, ParseError<'i, Error>> {
    let mut hints = Vec::new();
    loop {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        hints.push(font_tech_hint_from_str(ident.as_ref()).ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                format!("unsupported font technology hint `{ident}`"),
            )
        })?);

        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font technology hint list has an empty item",
            ));
        }
    }

    if hints.is_empty() {
        Err(unsupported_value(
            input,
            None,
            "font technology hint list is empty",
        ))
    } else {
        Ok(hints)
    }
}

fn font_tech_hint_from_str(value: &str) -> Option<CssFontTechHint> {
    match value.to_ascii_lowercase().as_str() {
        "variations" => Some(CssFontTechHint::Variations),
        "color-colrv0" => Some(CssFontTechHint::ColorCOLRv0),
        "color-colrv1" => Some(CssFontTechHint::ColorCOLRv1),
        "color-svg" => Some(CssFontTechHint::ColorSVG),
        "color-sbix" => Some(CssFontTechHint::ColorSbix),
        "color-cbdt" => Some(CssFontTechHint::ColorCBDT),
        "features-opentype" => Some(CssFontTechHint::FeaturesOpenType),
        "features-aat" => Some(CssFontTechHint::FeaturesAAT),
        "features-graphite" => Some(CssFontTechHint::FeaturesGraphite),
        "incremental" => Some(CssFontTechHint::Incremental),
        _ => None,
    }
}

fn parse_font_face_weight<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceWeight, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "normal" => Ok(CssFontFaceWeight::normal()),
            "bold" => Ok(CssFontFaceWeight::bold()),
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("font-weight descriptor", ident.as_ref()),
            )),
        };
    }

    let first = parse_font_weight_number(input)?;
    if input.is_exhausted() {
        CssFontFaceWeight::try_single(first)
            .ok_or_else(|| unsupported_value(input, None, "invalid font-weight descriptor"))
    } else {
        let second = parse_font_weight_number(input)?;
        CssFontFaceWeight::try_range(first, second)
            .ok_or_else(|| unsupported_value(input, None, "invalid font-weight descriptor range"))
    }
}

fn parse_font_weight_number<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => Ok(*value),
        Token::Ident(ident) => Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("font-weight descriptor", ident.as_ref()),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_font_face_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssFontFaceStyle::Normal),
        "italic" => Ok(CssFontFaceStyle::Italic),
        "oblique" => {
            if input.is_exhausted() {
                return Ok(CssFontFaceStyle::Oblique(None));
            }
            let start = parse_angle_degrees(input, "font-style oblique angle")?;
            let end = if input.is_exhausted() {
                None
            } else {
                Some(parse_angle_degrees(input, "font-style oblique angle range")?)
            };
            CssFontFaceObliqueRange::try_new(start, end)
                .map(|range| CssFontFaceStyle::Oblique(Some(range)))
                .ok_or_else(|| unsupported_value(input, None, "invalid font-style oblique range"))
        },
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("font-style descriptor", ident.as_ref()),
        )),
    }
}

fn parse_angle_degrees<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("deg") => Ok(*value),
        Token::Dimension { unit, .. } => Err(unsupported_value_at(
            location,
            None,
            format!("{context} must use deg units, got `{unit}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_font_face_stretch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceStretch, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let keyword = match_ignore_ascii_case! { &ident,
            "ultra-condensed" => CssFontFaceStretchKeyword::UltraCondensed,
            "extra-condensed" => CssFontFaceStretchKeyword::ExtraCondensed,
            "condensed" => CssFontFaceStretchKeyword::Condensed,
            "semi-condensed" => CssFontFaceStretchKeyword::SemiCondensed,
            "normal" => CssFontFaceStretchKeyword::Normal,
            "semi-expanded" => CssFontFaceStretchKeyword::SemiExpanded,
            "expanded" => CssFontFaceStretchKeyword::Expanded,
            "extra-expanded" => CssFontFaceStretchKeyword::ExtraExpanded,
            "ultra-expanded" => CssFontFaceStretchKeyword::UltraExpanded,
            _ => return Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("font-stretch descriptor", ident.as_ref()),
            )),
        };
        return Ok(CssFontFaceStretch::from_keyword(keyword));
    }

    let first = parse_font_stretch_percent(input)?;
    if input.is_exhausted() {
        CssFontFaceStretch::try_single_percent(first)
            .ok_or_else(|| unsupported_value(input, None, "invalid font-stretch descriptor"))
    } else {
        let second = parse_font_stretch_percent(input)?;
        CssFontFaceStretch::try_range_percent(first, second)
            .ok_or_else(|| unsupported_value(input, None, "invalid font-stretch descriptor range"))
    }
}

fn parse_font_stretch_percent<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Percentage { unit_value, .. } => Ok(*unit_value * 100.0),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_font_display<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontDisplay, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontDisplay::Auto),
        "block" => Ok(CssFontDisplay::Block),
        "swap" => Ok(CssFontDisplay::Swap),
        "fallback" => Ok(CssFontDisplay::Fallback),
        "optional" => Ok(CssFontDisplay::Optional),
        _ => Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("font-display", ident.as_ref()),
        )),
    }
}

fn parse_unicode_range_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUnicodeRangeList, ParseError<'i, Error>> {
    let mut ranges = Vec::new();
    loop {
        ranges.push(parse_unicode_range(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "unicode-range list has an empty item",
            ));
        }
    }

    CssUnicodeRangeList::try_new(ranges)
        .ok_or_else(|| unsupported_value(input, None, "unicode-range list is empty"))
}

fn parse_unicode_range<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUnicodeRange, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let range = ParsedUnicodeRange::parse(input).map_err(basic)?;
    CssUnicodeRange::try_new(range.start, range.end)
        .ok_or_else(|| unsupported_value_at(location, None, "invalid unicode-range"))
}

fn next_is_comma<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_comma = input.try_parse(Parser::expect_comma).is_ok();
    input.reset(&state);
    is_comma
}
