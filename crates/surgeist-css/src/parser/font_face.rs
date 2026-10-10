use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, Delimiter, ParseError,
    ParseErrorKind, Parser, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
    Token, match_ignore_ascii_case,
};

mod unicode_range;

use super::font_controls::parse_font_language_override;
use super::font_settings::{parse_font_feature_settings, parse_font_variation_settings};
use super::recovery::{
    RecoveryLoopOutcome, RecoveryProgress, RecoveryState, comma_member_span,
    recovery_action_for_error,
};
use super::typography::{
    common_font_style_keyword, parse_absolute_font_weight, parse_font_oblique_angle,
    parse_font_width, parse_non_generic_font_family_name,
};
use super::url::parse_url;
use super::values::parse_nonnegative_percentage;
use super::{block_item_diagnostic, is_declaration_recovery_unit, parse_descriptor_boundary};
use crate::error::{
    CssFeatureId, Error, basic, descriptor_name_error, from_parse_error, incomplete_descriptor_at,
    unexpected_at, with_descriptor_context,
};
use crate::syntax::*;
use crate::{
    CssFontFaceObliqueRange, CssFontFaceStyle, CssFontFaceWeight, CssFontFaceWidth,
    CssFontMetricOverride, CssFontNamedInstance, CssFontNamedInstanceString,
};

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
    CssFeatureId::new("official.descriptor.font-variation-settings"),
    CssFeatureId::new("official.descriptor.font-named-instance"),
    CssFeatureId::new("official.descriptor.font-language-override"),
    CssFeatureId::new("official.descriptor.ascent-override"),
    CssFeatureId::new("official.descriptor.descent-override"),
    CssFeatureId::new("official.descriptor.line-gap-override"),
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
    let descriptors = parse_body(source, input, diagnostics, recovery)?;
    Ok(CssFontFaceRule::new(descriptors).with_position(
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
}

pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> std::result::Result<CssFontFaceDescriptors, ParseError<'i, Error>> {
    let mut descriptors = Vec::new();
    let mut descriptor_parser = FontFaceDescriptorParser {
        source,
        recovery,
        diagnostics: Vec::new(),
    };

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

    diagnostics.extend(descriptor_parser.diagnostics);
    Ok(CssFontFaceDescriptors::new(descriptors))
}

struct FontFaceDescriptorParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
}

pub(super) fn parse_contents(
    source: &str,
    limits: crate::CssComponentValueLimits,
) -> crate::CssParseReport<Option<CssFontFaceDescriptors>> {
    let (descriptors, diagnostics) = super::declaration_block::parse_contents(
        source,
        limits,
        crate::CssParserContext::default(),
        |recovery| FontFaceDescriptorParser {
            source,
            recovery,
            diagnostics: Vec::new(),
        },
    )
    .into_parts();
    crate::CssParseReport::new(descriptors.map(CssFontFaceDescriptors::new), diagnostics)
}

impl super::declaration_block::Receiver for FontFaceDescriptorParser<'_> {
    type Declaration = CssFontFaceDescriptor;
    fn check_name<'i>(
        &self,
        name: &str,
        location: cssparser::SourceLocation,
    ) -> Result<(), ParseError<'i, Error>> {
        descriptor_kind(name, location).map(|_| ())
    }
    fn parse_value<'i>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, '_>,
        start: &ParserState,
        _recovery: &RecoveryState,
    ) -> Result<CssFontFaceDescriptor, ParseError<'i, Error>>
    where
        Self: 'i,
    {
        DeclarationParser::parse_value(self, name, input, start)
    }
    fn take_diagnostics(&mut self) -> Vec<crate::CssRecoveryDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

fn descriptor_kind<'i>(
    name: &str,
    location: cssparser::SourceLocation,
) -> Result<CssFontFaceDescriptorKind, ParseError<'i, Error>> {
    CssFontFaceDescriptorKind::from_css_name(name)
        .ok_or_else(|| descriptor_name_error(location, "font-face", name))
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

impl<'i, 's: 'i> DeclarationParser<'i> for FontFaceDescriptorParser<'s> {
    type Declaration = CssFontFaceDescriptor;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let mut implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.descriptor")?;
        let position = crate::source::CssSourcePosition::from_cssparser(
            declaration_start.position(),
            declaration_start.source_location(),
        );
        let kind = descriptor_kind(name.as_ref(), declaration_start.source_location())?;
        let mut member_diagnostics = Vec::new();
        let numeric = crate::numeric::NumericInputContext::parsed(self.recovery.source_snapshot());
        let value = parse_authored_font_face_value(
            self.source,
            input,
            kind,
            self.recovery.source_snapshot(),
            &numeric,
            &mut member_diagnostics,
            &mut implicit_closures,
        )
        .map_err(|error| {
            if name.eq_ignore_ascii_case("font-stretch")
                && matches!(
                    &error.kind,
                    ParseErrorKind::Custom(error)
                        if matches!(error.kind(), crate::ErrorKind::InvalidDeclarationAnnotation(_))
                )
            {
                // The legacy spelling is the actual authored descriptor name.
                crate::error::invalid_descriptor_annotation(
                    error.location,
                    "font-face",
                    name.as_ref(),
                )
            } else {
                with_descriptor_context(error, "font-face", name.as_ref())
            }
        })?;
        // Only an enclosing value that passed its complete boundary retains members.
        self.diagnostics.extend(member_diagnostics);
        self.recovery.retain_component_closures(implicit_closures);
        Ok(CssFontFaceDescriptor::new(value).with_position(position))
    }
}

/// Applies the shared descriptor root and Env1 admission before ordinary grammar.
pub(super) fn parse_authored_font_face_value<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    kind: CssFontFaceDescriptorKind,
    snapshot: &crate::CssSourceSnapshot,
    numeric: &crate::numeric::NumericInputContext<'_>,
    member_diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssAuthoredFontFaceDescriptorValue, ParseError<'i, Error>> {
    super::descriptor_values::validate_root(input, "font-face", kind.css_name())?;
    let start = input.state();
    let components = match crate::CssComponentValues::collect_from_parser(input, snapshot) {
        Ok(components) => components,
        Err(error)
            if kind == CssFontFaceDescriptorKind::Src
                && matches!(
                    error.kind(),
                    crate::CssComponentValueErrorKind::BadString
                        | crate::CssComponentValueErrorKind::BadUrl
                ) =>
        {
            // The ordinary src parser can discard a bad member and retain later
            // sources. A bad token cannot qualify a value for Env1 deferral.
            input.reset(&start);
            return parse_font_face_value(
                source,
                input,
                kind,
                numeric,
                member_diagnostics,
                implicit_closures,
            )
            .map(CssAuthoredFontFaceDescriptorValue::Ordinary);
        }
        Err(error) => {
            return Err(crate::error::invalid_component_value(
                input.current_source_location(),
                error,
            ));
        }
    };
    input.reset(&start);
    parse_font_face_value_with_components(
        source,
        input,
        kind,
        components,
        numeric,
        member_diagnostics,
        implicit_closures,
    )
}

fn parse_font_face_value_with_components<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    kind: CssFontFaceDescriptorKind,
    components: crate::CssComponentValues,
    numeric: &crate::numeric::NumericInputContext<'_>,
    member_diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssAuthoredFontFaceDescriptorValue, ParseError<'i, Error>> {
    let pending = super::variables::descriptor_environment_qualifies(components.items(), numeric)
        .map_err(|error| {
        crate::error::invalid_component_value(input.current_source_location(), error)
    })?;
    if pending {
        return parse_descriptor_boundary(input, "font-face", kind.css_name(), |input| {
            super::descriptor_values::consume_remaining_components(input)?;
            Ok(CssAuthoredFontFaceDescriptorValue::pending(
                kind, components,
            ))
        });
    }
    parse_font_face_value(
        source,
        input,
        kind,
        numeric,
        member_diagnostics,
        implicit_closures,
    )
    .map(CssAuthoredFontFaceDescriptorValue::Ordinary)
}

/// Strict component bridge returns the first recovered src diagnostic separately.
pub(crate) fn construct_font_face_descriptor_value(
    kind: CssFontFaceDescriptorKind,
    components: &crate::CssComponentValues,
    serialized: &crate::CssSerializedValue,
) -> Result<(CssAuthoredFontFaceDescriptorValue, Option<Error>), Error> {
    let source = serialized.as_css();
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = cssparser::ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    super::descriptor_values::validate_root(&mut input, "font-face", kind.css_name())
        .map_err(|error| crate::error::from_parse_error(source, error))?;
    let numeric = crate::numeric::NumericInputContext::components(components, serialized);
    let mut member_diagnostics = Vec::new();
    let mut implicit_closures = Vec::new();
    let value = parse_font_face_value_with_components(
        source,
        &mut input,
        kind,
        components.clone(),
        &numeric,
        &mut member_diagnostics,
        &mut implicit_closures,
    )
    .map_err(|error| crate::error::from_parse_error(source, error))?;
    input
        .expect_exhausted()
        .map_err(|error| crate::error::from_parse_error(source, basic(error)))?;
    let recovered = member_diagnostics
        .first()
        .map(|diagnostic| diagnostic.error().clone());
    Ok((value, recovered))
}

/// Shared value grammar; callers commit recovery only after their complete boundary succeeds.
pub(super) fn parse_font_face_value<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    kind: CssFontFaceDescriptorKind,
    numeric: &crate::numeric::NumericInputContext<'_>,
    member_diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssFontFaceDescriptorValue, ParseError<'i, Error>> {
    parse_descriptor_boundary(input, "font-face", kind.css_name(), |input| {
        Ok(match kind {
            CssFontFaceDescriptorKind::FontFamily => {
                CssFontFaceDescriptorValue::FontFamily(parse_font_face_family(input)?)
            }
            CssFontFaceDescriptorKind::Src => {
                CssFontFaceDescriptorValue::Src(parse_font_face_source_list(
                    source,
                    input,
                    numeric,
                    member_diagnostics,
                    implicit_closures,
                )?)
            }
            CssFontFaceDescriptorKind::FontWeight => {
                CssFontFaceDescriptorValue::FontWeight(parse_font_face_weight(input, numeric)?)
            }
            CssFontFaceDescriptorKind::FontStyle => {
                CssFontFaceDescriptorValue::FontStyle(parse_font_face_style(input, numeric)?)
            }
            CssFontFaceDescriptorKind::FontWidth => {
                CssFontFaceDescriptorValue::FontWidth(parse_font_face_width(input, numeric)?)
            }
            CssFontFaceDescriptorKind::FontDisplay => {
                CssFontFaceDescriptorValue::FontDisplay(parse_font_display(input)?)
            }
            CssFontFaceDescriptorKind::UnicodeRange => {
                CssFontFaceDescriptorValue::UnicodeRange(parse_unicode_range_list(input)?)
            }
            CssFontFaceDescriptorKind::FontFeatureSettings => {
                CssFontFaceDescriptorValue::FontFeatureSettings(parse_font_feature_settings(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::FontVariationSettings => {
                CssFontFaceDescriptorValue::FontVariationSettings(parse_font_variation_settings(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::FontNamedInstance => {
                CssFontFaceDescriptorValue::FontNamedInstance(parse_font_named_instance(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::FontLanguageOverride => {
                CssFontFaceDescriptorValue::FontLanguageOverride(parse_font_language_override(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::AscentOverride => {
                CssFontFaceDescriptorValue::AscentOverride(parse_font_metric_override(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::DescentOverride => {
                CssFontFaceDescriptorValue::DescentOverride(parse_font_metric_override(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::LineGapOverride => {
                CssFontFaceDescriptorValue::LineGapOverride(parse_font_metric_override(
                    input, numeric,
                )?)
            }
        })
    })
    .map_err(|error| with_descriptor_context(error, "font-face", kind.css_name()))
}

fn parse_font_named_instance<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssFontNamedInstance, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssFontNamedInstance::Auto);
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = numeric
        .collect(input)
        .map_err(|_| unexpected_at(location))?;
    CssFontNamedInstanceString::try_from_component(component)
        .map(CssFontNamedInstance::String)
        .map_err(|_| unexpected_at(location))
}

fn parse_font_metric_override<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssFontMetricOverride, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontMetricOverride::Normal);
    }
    parse_nonnegative_percentage(input, numeric).map(CssFontMetricOverride::Percentage)
}

fn parse_font_face_family<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFaceFamily, ParseError<'i, Error>> {
    let family = parse_non_generic_font_family_name(input)?;
    CssFontFaceFamily::try_new(family.as_str())
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_font_face_source_list<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> std::result::Result<CssFontFaceSourceList, ParseError<'i, Error>> {
    let mut sources = Vec::new();
    let mut first_error = None;
    let mut preceding_comma = None;
    loop {
        let member_start = input.position().byte_index();
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            let parsed = parse_font_face_source(member, numeric)?;
            member.expect_exhausted().map_err(basic)?;
            Ok(parsed)
        });
        let member_end = input.position().byte_index();
        let following_comma = match input.next() {
            Ok(Token::Comma) => Some((member_end, input.position().byte_index())),
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => None,
            Ok(_) => {
                return Err(unexpected_at(input.current_source_location()));
            }
            Err(error) => return Err(basic(error)),
        };

        match result {
            Ok(parsed) => sources.push(parsed),
            Err(error) => {
                // Openings in a discarded member do not describe retained syntax.
                implicit_closures.retain(|opening| !(member_start..member_end).contains(opening));
                if first_error.is_none() {
                    first_error = Some(error.clone());
                }
                let action = recovery_action_for_error(
                    &error,
                    crate::CssRecoveryAction::DropFontSourceListItem,
                );
                let error =
                    from_parse_error(source, with_descriptor_context(error, "font-face", "src"));
                if let Some(span) = comma_member_span(
                    source,
                    member_start,
                    member_end,
                    following_comma,
                    preceding_comma,
                ) && let Some(diagnostic) =
                    crate::CssRecoveryDiagnostic::new(error, span, action)
                {
                    diagnostics.push(diagnostic);
                }
            }
        }

        let Some(comma) = following_comma else {
            break;
        };
        preceding_comma = Some(comma);
    }

    CssFontFaceSourceList::try_new(sources).ok_or_else(|| {
        first_error.unwrap_or_else(|| unexpected_at(input.current_source_location()))
    })
}

fn parse_font_face_source<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontFaceSource, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_function_matching("local"))
        .is_ok()
    {
        let name = input.parse_nested_block(parse_local_name)?;
        return CssFontLocalName::try_new(name.as_str())
            .map(CssFontFaceSource::Local)
            .ok_or_else(|| unexpected_at(input.current_source_location()));
    }

    let url = parse_font_source_url(input, numeric)?;
    let mut format = None;
    let mut tech = Vec::new();
    let mut saw_tech = false;

    while !input.is_exhausted() && !next_is_comma(input) {
        if input
            .try_parse(|input| input.expect_function_matching("format"))
            .is_ok()
        {
            if saw_tech {
                return Err(unexpected_at(input.current_source_location()));
            }
            if format.is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            format = Some(input.parse_nested_block(parse_font_format)?);
        } else if input
            .try_parse(|input| input.expect_function_matching("tech"))
            .is_ok()
        {
            if saw_tech {
                return Err(unexpected_at(input.current_source_location()));
            }
            tech = input.parse_nested_block(parse_font_tech_hints)?;
            saw_tech = true;
        } else {
            return Err(unexpected_at(input.current_source_location()));
        }
    }

    Ok(CssFontFaceSource::Url(CssFontFaceUrlSource::new(
        url, format, tech,
    )))
}

fn parse_font_source_url<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssUrl, ParseError<'i, Error>> {
    parse_url(input, numeric)
}

fn parse_local_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFamilyName, ParseError<'i, Error>> {
    let name = parse_non_generic_font_family_name(input)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(name)
}

fn parse_font_format<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFormat, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let format = if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let hint = CssFontFormatHint::from_ascii_name(ident.as_bytes())
            .ok_or_else(|| unexpected_at(location))?;
        CssFontFormat::Keyword(hint)
    } else {
        let value = input.expect_string_cloned().map_err(basic)?;
        CssFontFormat::String(CssFontFormatString::new(value.to_string()))
    };
    input.expect_exhausted().map_err(basic)?;

    Ok(format)
}

fn parse_font_tech_hints<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Vec<CssFontTechHint>, ParseError<'i, Error>> {
    let mut hints = Vec::new();
    loop {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        hints.push(
            CssFontTechHint::from_ascii_name(ident.as_bytes())
                .ok_or_else(|| unexpected_at(location))?,
        );

        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
        if input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }

    if hints.is_empty() {
        Err(unexpected_at(input.current_source_location()))
    } else {
        Ok(hints)
    }
}

fn parse_font_face_weight<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontFaceWeight, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssFontFaceWeight::Auto);
    }
    let start = parse_absolute_font_weight(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_absolute_font_weight(input, numeric)?)
    };
    Ok(CssFontFaceWeight::Range { start, end })
}

fn parse_font_face_style<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontFaceStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("auto") {
        return Ok(CssFontFaceStyle::Auto);
    }
    if ident.eq_ignore_ascii_case("oblique") {
        if input.is_exhausted() {
            return Ok(CssFontFaceStyle::Oblique { range: None });
        }
        let start = parse_font_oblique_angle(input, numeric)?;
        let end = if input.is_exhausted() {
            None
        } else {
            Some(parse_font_oblique_angle(input, numeric)?)
        };
        return Ok(CssFontFaceStyle::Oblique {
            range: Some(CssFontFaceObliqueRange::new(start, end)),
        });
    }
    common_font_style_keyword(&ident)
        .map(CssFontFaceStyle::Keyword)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_font_face_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontFaceWidth, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssFontFaceWidth::Auto);
    }
    let start = parse_font_width(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_font_width(input, numeric)?)
    };
    Ok(CssFontFaceWidth::Range { start, end })
}

pub(super) fn parse_font_display<'i, 't>(
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
        _ => Err(unexpected_at(location)),
    }
}

fn parse_unicode_range_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUnicodeRangeList, ParseError<'i, Error>> {
    let mut ranges = Vec::new();
    loop {
        ranges.push(unicode_range::parse(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(incomplete_descriptor_at(
                input.current_source_location(),
                "font-face",
                "unicode-range",
            ));
        }
    }

    CssUnicodeRangeList::try_new(ranges)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn next_is_comma<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_comma = input.try_parse(Parser::expect_comma).is_ok();
    input.reset(&state);
    is_comma
}
