use std::cell::RefCell;
use std::rc::Rc;

use cssparser::{ParseError, ParseErrorKind, Parser};
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, QualifiedRule, RuleGrammar};

use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, grammar_error, location_at, offset_of,
};
use super::declaration_unit::DeclarationText;
use super::descriptor_stream::{DescriptorGrammar, DescriptorRecovery, drive_descriptors};
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;
use surgeist_css_parser::stream::StopToken;

pub(crate) mod unicode_range;

use super::descriptor_stream::{
    NativeValueError, Value, at_final_comma, byte_offset, collect_scope, consume_remaining,
    expect_exhausted, fail, from_component, from_here, from_remaining, inside_consumed, location,
    native, parse_descriptor_checked, parse_scope, validate_root,
};
use super::font_controls::parse_font_language_override;
use super::font_settings::{parse_font_feature_settings, parse_font_variation_settings};
use super::property_input::StreamExt;
use super::recovery::{RecoveryState, comma_member_span, recovery_action_for_error};
use super::syntax_types::{Attached, Stream, SyntaxError};
use super::typography::{
    common_font_style_keyword, parse_absolute_font_weight, parse_font_oblique_angle,
    parse_font_width, parse_non_generic_font_family_name,
};
use super::url::parse_url;
use super::values::parse_nonnegative_percentage;
use crate::diagnostic::error::{
    descriptor_name_error, from_parse_error, unexpected_at, with_descriptor_context,
};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::numeric::expression::NumericInputContext;
use crate::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontDisplay, CssFontFaceDescriptor,
    CssFontFaceDescriptorKind, CssFontFaceDescriptorValue, CssFontFaceDescriptors,
    CssFontFaceFamily, CssFontFaceRule, CssFontFaceSource, CssFontFaceSourceList,
    CssFontFaceUrlSource, CssFontFormat, CssFontFormatHint, CssFontFormatString, CssFontLocalName,
    CssFontTechHint, CssUnicodeRangeList,
};
use crate::rule::font_face::{
    CssFontMetricOverride, CssFontNamedInstance, CssFontNamedInstanceString,
};
use crate::values::common::CssUrl;
use crate::values::font::CssFontFamilyName;
use crate::values::font::{
    CssFontFaceObliqueRange, CssFontFaceStyle, CssFontFaceWeight, CssFontFaceWidth,
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

pub(crate) fn parse_font_face_rule<'i>(
    source: &str,
    contents: &mut Stream<'i>,
    position: crate::source::CssSourcePosition,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssFontFaceRule {
    let descriptors = parse_body(source, contents, diagnostics, recovery);
    CssFontFaceRule::new(descriptors).with_position(position)
}

/// The descriptors of the block `contents` holds. Syntax 3 (2026 CRD) §7.1:
/// qualified rules and at-rules are invalid in this `<declaration-list>` and
/// are dropped one by one.
pub(super) fn parse_body<'i>(
    source: &str,
    contents: &mut Stream<'i>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssFontFaceDescriptors {
    let shared = Rc::new(RefCell::new(Vec::new()));
    let mut grammar = FontFaceDescriptorParser {
        source,
        recovery: recovery.clone(),
        diagnostics: Rc::clone(&shared),
    };
    let mut readmit = FontFaceDescriptorParser {
        source,
        recovery: recovery.clone(),
        diagnostics: Rc::clone(&shared),
    };
    let (descriptors, mut dropped) = drive_descriptors(
        contents,
        &mut grammar,
        &mut readmit,
        &DescriptorRecovery {
            source,
            recovery: &recovery,
            production: "baseline.rule.font-face",
            declaration: crate::diagnostic::CssRecoveryAction::DropDescriptor,
            at_rule_first: false,
        },
    );
    diagnostics.append(&mut dropped);
    drop((grammar, readmit));
    diagnostics.extend(shared.take());
    CssFontFaceDescriptors::new(descriptors)
}

struct FontFaceDescriptorParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
    /// What the recovery of the members of descriptor values reported.
    diagnostics: Rc<RefCell<Vec<crate::diagnostic::CssRecoveryDiagnostic>>>,
}

pub(super) fn parse_contents(
    source: &str,
    limits: crate::component::CssComponentValueLimits,
) -> crate::CssParseReport<Option<CssFontFaceDescriptors>> {
    let (descriptors, diagnostics) = super::declaration_block::parse_contents(
        source,
        limits,
        crate::CssParserContext::default(),
        |recovery| FontFaceDescriptorParser {
            source,
            recovery,
            diagnostics: Rc::default(),
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
    ) -> Result<(), ParseError<'i, CssError>> {
        descriptor_kind(name, location).map(|_| ())
    }
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
        _recovery: &RecoveryState,
    ) -> Result<CssFontFaceDescriptor, ParseError<'i, CssError>> {
        FontFaceDescriptorParser::admit(self, name, text)
    }
    fn take_diagnostics(&mut self) -> Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        self.diagnostics.take()
    }
}

fn descriptor_kind<'i>(
    name: &str,
    location: cssparser::SourceLocation,
) -> Result<CssFontFaceDescriptorKind, ParseError<'i, CssError>> {
    CssFontFaceDescriptorKind::from_css_name(name)
        .ok_or_else(|| descriptor_name_error(location, "font-face", name))
}

// Fonts 4 §4.1: the body is a descriptor list, so a recognized at-rule is
// misplaced here; an unrecognized one stays unknown.
impl<'i> RuleGrammar<'i, Attached<'i>> for FontFaceDescriptorParser<'_> {
    type Item = CssFontFaceDescriptor;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let name = rule.name();
        let location = location_at(
            self.recovery.source_snapshot(),
            offset_of(rule.prelude().start_site()),
        );
        Err(grammar_error(
            rule.prelude().start_site(),
            if super::rule_list::at_rule_production(name).is_some() {
                crate::diagnostic::error::invalid_at_rule_placement(
                    location,
                    name,
                    "a font-face descriptor list",
                )
            } else {
                ParseError {
                    kind: ParseErrorKind::Basic(cssparser::BasicParseErrorKind::AtRuleInvalid(
                        cssparser::CowRcStr::from(name),
                    )),
                    location,
                }
            },
        ))
    }

    // Syntax 3 (2026 CRD) §5.5.5 reads a unit that is not a declaration as a
    // qualified rule, which a descriptor list does not admit.
    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for FontFaceDescriptorParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl<'i> DescriptorGrammar<'i, CssFontFaceDescriptor> for FontFaceDescriptorParser<'_> {
    fn readmit(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssFontFaceDescriptor, NativeError<'i>> {
        self.admit(name, text)
    }
}

impl FontFaceDescriptorParser<'_> {
    /// Admits the descriptor named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssFontFaceDescriptor, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let mut implicit_closures = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.descriptor",
        )?;
        let position = self.recovery.source_position(text.name_start());
        let kind = descriptor_kind(name, location_at(snapshot, text.name_start()))?;
        let mut member_diagnostics = Vec::new();
        let value = self
            .read_value(
                text,
                kind,
                &mut member_diagnostics,
                &mut implicit_closures,
            )
            .map_err(|error| {
                if name.eq_ignore_ascii_case("font-stretch")
                    && matches!(
                        &error.kind,
                        ParseErrorKind::Custom(error)
                            if matches!(error.kind(), crate::diagnostic::error::ErrorKind::InvalidDeclarationAnnotation(_))
                    )
                {
                    // The legacy spelling is the actual authored descriptor name.
                    crate::diagnostic::error::invalid_descriptor_annotation(
                        error.location,
                        "font-face",
                        name,
                    )
                } else {
                    with_descriptor_context(error, "font-face", name)
                }
            })?;
        // Only an enclosing value that passed its complete boundary retains members.
        self.diagnostics.borrow_mut().extend(member_diagnostics);
        self.recovery.retain_component_closures(implicit_closures);
        Ok(CssFontFaceDescriptor::new(value).with_position(position))
    }

    /// The value of the descriptor `kind` at `text`, whose whole text up to the
    /// end of the declaration is the scope the value grammar reads.
    fn read_value(
        &self,
        text: &DeclarationText,
        kind: CssFontFaceDescriptorKind,
        member_diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
        implicit_closures: &mut Vec<usize>,
    ) -> Result<CssAuthoredFontFaceDescriptorValue, NativeValueError> {
        let snapshot = self.recovery.source_snapshot();
        let numeric = NumericInputContext::parsed(snapshot);
        let document =
            crate::syntax_consumption::source_scope_document(snapshot, text.value_start..text.end)
                .map_err(|error| {
                    crate::diagnostic::error::invalid_component_value(
                        location_at(snapshot, text.value_start),
                        error,
                    )
                })?;
        let mut value = document.root();
        read_authored_value(
            self.source,
            &mut value,
            kind,
            &numeric,
            member_diagnostics,
            implicit_closures,
        )
    }
}

/// Applies the shared descriptor root and Env1 admission before ordinary grammar
/// to the value the native parser `input` holds, to the end of its scope.
pub(super) fn parse_authored_font_face_value<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    kind: CssFontFaceDescriptorKind,
    snapshot: &crate::source::CssSourceSnapshot,
    member_diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssAuthoredFontFaceDescriptorValue, ParseError<'i, CssError>> {
    parse_scope(input, snapshot, |value, numeric| {
        let at = value.site();
        read_authored_value(
            source,
            value,
            kind,
            numeric,
            member_diagnostics,
            implicit_closures,
        )
        .map_err(|error| SyntaxError::custom(at, error))
    })
}

/// [`parse_authored_font_face_value`] over the stream of the value.
fn read_authored_value<'i>(
    source: &str,
    input: &mut Stream<'i>,
    kind: CssFontFaceDescriptorKind,
    numeric: &NumericInputContext<'_>,
    member_diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssAuthoredFontFaceDescriptorValue, NativeValueError> {
    validate_root(input, numeric, "font-face", kind.as_str())
        .map_err(|error| native(numeric, &error))?;
    let start = input.mark();
    let components = match collect_scope(input, numeric) {
        Ok(components) => components,
        Err((error, _))
            if kind == CssFontFaceDescriptorKind::Src
                && matches!(
                    error.kind,
                    crate::component::values::CssComponentValueErrorKind::BadString
                        | crate::component::values::CssComponentValueErrorKind::BadUrl
                ) =>
        {
            // The ordinary src parser can discard a bad member and retain later
            // sources. A bad token cannot qualify a value for Env1 deferral.
            input.restore(start);
            return read_font_face_value(
                source,
                input,
                kind,
                numeric,
                member_diagnostics,
                implicit_closures,
            )
            .map(CssAuthoredFontFaceDescriptorValue::Ordinary);
        }
        Err((error, at)) => {
            return Err(crate::diagnostic::error::invalid_component_value(at, error));
        }
    };
    input.restore(start);
    read_value_with_components(
        source,
        input,
        kind,
        components,
        numeric,
        member_diagnostics,
        implicit_closures,
    )
}

fn read_value_with_components<'i>(
    source: &str,
    input: &mut Stream<'i>,
    kind: CssFontFaceDescriptorKind,
    components: crate::component::CssComponentValues,
    numeric: &NumericInputContext<'_>,
    member_diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssAuthoredFontFaceDescriptorValue, NativeValueError> {
    let pending = super::variables::descriptor_environment_qualifies(components.items(), numeric)
        .map_err(|error| {
        crate::diagnostic::error::invalid_component_value(location(numeric, input.site()), error)
    })?;
    if pending {
        return parse_descriptor_checked(input, numeric, "font-face", kind.as_str(), |input| {
            consume_remaining(input);
            Ok(CssAuthoredFontFaceDescriptorValue::pending(
                kind, components,
            ))
        });
    }
    read_font_face_value(
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
    components: &crate::component::CssComponentValues,
    serialized: &crate::component::CssSerializedValue,
) -> Result<(CssAuthoredFontFaceDescriptorValue, Option<CssError>), CssError> {
    let source = serialized.as_css();
    let numeric = NumericInputContext::components(components, serialized);
    let mut member_diagnostics = Vec::new();
    let mut implicit_closures = Vec::new();
    let result = (|| {
        let document = crate::syntax_consumption::checked_document(
            components.items(),
            crate::syntax_consumption::SyntaxInputLimits::default(),
            0,
        )
        .map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                cssparser::SourceLocation { line: 0, column: 1 },
                error,
            )
        })?;
        let mut input = document.root();
        validate_root(&mut input, &numeric, "font-face", kind.as_str())
            .map_err(|error| native(&numeric, &error))?;
        let value = read_value_with_components(
            source,
            &mut input,
            kind,
            components.clone(),
            &numeric,
            &mut member_diagnostics,
            &mut implicit_closures,
        )?;
        expect_exhausted(&mut input).map_err(|error| native(&numeric, &error))?;
        Ok::<_, NativeValueError>(value)
    })();
    let value = result.map_err(|error| from_parse_error(source, error))?;
    let recovered = member_diagnostics
        .first()
        .map(|diagnostic| diagnostic.error().clone());
    Ok((value, recovered))
}

/// Shared value grammar; callers commit recovery only after their complete boundary succeeds.
fn read_font_face_value<'i>(
    source: &str,
    input: &mut Stream<'i>,
    kind: CssFontFaceDescriptorKind,
    numeric: &NumericInputContext<'_>,
    member_diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Result<CssFontFaceDescriptorValue, NativeValueError> {
    parse_descriptor_checked(input, numeric, "font-face", kind.as_str(), |input| {
        Ok(match kind {
            CssFontFaceDescriptorKind::FontFamily => {
                CssFontFaceDescriptorValue::FontFamily(parse_font_face_family(input, numeric)?)
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
                CssFontFaceDescriptorValue::FontDisplay(parse_font_display(input, numeric)?)
            }
            CssFontFaceDescriptorKind::UnicodeRange => {
                CssFontFaceDescriptorValue::UnicodeRange(parse_unicode_range_list(input, numeric)?)
            }
            CssFontFaceDescriptorKind::FontFeatureSettings => {
                CssFontFaceDescriptorValue::FontFeatureSettings(from_remaining(
                    numeric,
                    input,
                    |input| parse_font_feature_settings(input, numeric),
                )?)
            }
            CssFontFaceDescriptorKind::FontVariationSettings => {
                CssFontFaceDescriptorValue::FontVariationSettings(from_remaining(
                    numeric,
                    input,
                    |input| parse_font_variation_settings(input, numeric),
                )?)
            }
            CssFontFaceDescriptorKind::FontNamedInstance => {
                CssFontFaceDescriptorValue::FontNamedInstance(parse_font_named_instance(
                    input, numeric,
                )?)
            }
            CssFontFaceDescriptorKind::FontLanguageOverride => {
                CssFontFaceDescriptorValue::FontLanguageOverride(from_component(
                    numeric,
                    input,
                    |input| parse_font_language_override(input, numeric),
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
    .map_err(|error| with_descriptor_context(error, "font-face", kind.as_str()))
}

fn parse_font_named_instance<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontNamedInstance> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssFontNamedInstance::Auto);
    }
    input.discard_whitespace();
    let location = input.site();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|_| fail(numeric, location, unexpected_at))?;
    CssFontNamedInstanceString::try_from_component(component)
        .map(CssFontNamedInstance::String)
        .map_err(|_| fail(numeric, location, unexpected_at))
}

fn parse_font_metric_override<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontMetricOverride> {
    if input
        .attempt(|input| input.consume_keyword("normal"))
        .is_ok()
    {
        return Ok(CssFontMetricOverride::Normal);
    }
    from_component(numeric, input, |input| {
        parse_nonnegative_percentage(input, numeric)
    })
    .map(CssFontMetricOverride::Percentage)
}

fn parse_font_face_family<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFaceFamily> {
    let family = from_remaining(numeric, input, parse_non_generic_font_family_name)?;
    CssFontFaceFamily::try_new(family.as_str())
        .ok_or_else(|| fail(numeric, input.site(), unexpected_at))
}

fn parse_font_face_source_list<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    implicit_closures: &mut Vec<usize>,
) -> Value<'i, CssFontFaceSourceList> {
    let mut sources = Vec::new();
    let mut first_error = None;
    let mut preceding_comma = None;
    loop {
        let member_start = byte_offset(numeric, input.site());
        let result = input.within_list_until(StopToken::Comma, |member| {
            let parsed = parse_font_face_source(member, numeric)?;
            expect_exhausted(member)?;
            Ok(parsed)
        });
        let member_end = byte_offset(numeric, input.site());
        let following_comma = match input.consume_after_whitespace() {
            Ok(Token::Comma) => Some((member_end, byte_offset(numeric, input.site()))),
            Err(error) if error.is_end_of_stream() => None,
            Ok(_) => return Err(fail(numeric, input.site(), unexpected_at)),
            Err(error) => return Err(error.into()),
        };

        match result {
            Ok(parsed) => sources.push(parsed),
            Err(error) => {
                let error = native(numeric, &error);
                // Openings in a discarded member do not describe retained syntax.
                implicit_closures.retain(|opening| !(member_start..member_end).contains(opening));
                if first_error.is_none() {
                    first_error = Some(error.clone());
                }
                let action = recovery_action_for_error(
                    &error,
                    crate::diagnostic::CssRecoveryAction::DropFontSourceListItem,
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
                    crate::diagnostic::CssRecoveryDiagnostic::new(error, span, action)
                {
                    diagnostics.push(diagnostic);
                }
            }
        }

        let Some(comma) = following_comma else {
            break;
        };
        // Fonts 4 "parse a list" splits with Syntax 3 (2026 CRD) §5.4.10.
        if at_final_comma(input) {
            break;
        }
        preceding_comma = Some(comma);
    }

    CssFontFaceSourceList::try_new(sources).ok_or_else(|| {
        SyntaxError::custom(
            input.site(),
            first_error.unwrap_or_else(|| unexpected_at(location(numeric, input.site()))),
        )
    })
}

fn parse_font_face_source<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFaceSource> {
    if input
        .attempt(|input| input.consume_function_named("local"))
        .is_ok()
    {
        let name = input.within_block(|input| parse_local_name(input, numeric))?;
        return CssFontLocalName::try_new(name.as_str())
            .map(CssFontFaceSource::Local)
            .ok_or_else(|| fail(numeric, input.site(), unexpected_at));
    }

    let url = parse_font_source_url(input, numeric)?;
    let mut format = None;
    let mut tech = Vec::new();
    let mut saw_tech = false;

    while !input.only_whitespace_remains() && !next_is_comma(input) {
        if input
            .attempt(|input| input.consume_function_named("format"))
            .is_ok()
        {
            if saw_tech || format.is_some() {
                return Err(fail(numeric, inside_consumed(input.site()), unexpected_at));
            }
            format = Some(input.within_block(|input| parse_font_format(input, numeric))?);
        } else if input
            .attempt(|input| input.consume_function_named("tech"))
            .is_ok()
        {
            if saw_tech {
                return Err(fail(numeric, inside_consumed(input.site()), unexpected_at));
            }
            tech = input.within_block(|input| parse_font_tech_hints(input, numeric))?;
            saw_tech = true;
        } else {
            return Err(fail(numeric, input.site(), unexpected_at));
        }
    }

    Ok(CssFontFaceSource::Url(CssFontFaceUrlSource::new(
        url, format, tech,
    )))
}

fn parse_font_source_url<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssUrl> {
    from_component(numeric, input, |input| parse_url(input, numeric))
}

fn parse_local_name<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFamilyName> {
    let name = from_remaining(numeric, input, parse_non_generic_font_family_name)?;
    expect_exhausted(input)?;
    Ok(name)
}

fn parse_font_format<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFormat> {
    let location = input.site();
    let format = if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let hint = CssFontFormatHint::from_ascii_name(ident.as_bytes())
            .ok_or_else(|| fail(numeric, location, unexpected_at))?;
        CssFontFormat::Keyword(hint)
    } else {
        let value = from_here(input, Stream::consume_string)?;
        CssFontFormat::String(CssFontFormatString::new(value.to_string()))
    };
    expect_exhausted(input)?;

    Ok(format)
}

fn parse_font_tech_hints<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, Vec<CssFontTechHint>> {
    let mut hints = Vec::new();
    loop {
        let location = input.site();
        let ident = from_here(input, Stream::consume_ident)?;
        hints.push(
            CssFontTechHint::from_ascii_name(ident.as_bytes())
                .ok_or_else(|| fail(numeric, location, unexpected_at))?,
        );

        if input.only_whitespace_remains() {
            break;
        }
        from_here(input, Stream::consume_comma)?;
        if input.only_whitespace_remains() {
            return Err(fail(numeric, input.site(), unexpected_at));
        }
    }

    if hints.is_empty() {
        Err(fail(numeric, input.site(), unexpected_at))
    } else {
        Ok(hints)
    }
}

fn parse_font_face_weight<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFaceWeight> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssFontFaceWeight::Auto);
    }
    let start = from_component(numeric, input, |input| {
        parse_absolute_font_weight(input, numeric)
    })?;
    let end = if input.only_whitespace_remains() {
        None
    } else {
        Some(from_component(numeric, input, |input| {
            parse_absolute_font_weight(input, numeric)
        })?)
    };
    Ok(CssFontFaceWeight::Range { start, end })
}

fn parse_font_face_style<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFaceStyle> {
    let keyword_location = input.site();
    let ident = from_here(input, Stream::consume_ident)?;
    if ident.eq_ignore_ascii_case("auto") {
        return Ok(CssFontFaceStyle::Auto);
    }
    if ident.eq_ignore_ascii_case("oblique") {
        if input.only_whitespace_remains() {
            return Ok(CssFontFaceStyle::Oblique { range: None });
        }
        let start = from_component(numeric, input, |input| {
            parse_font_oblique_angle(input, numeric)
        })?;
        let end = if input.only_whitespace_remains() {
            None
        } else {
            Some(from_component(numeric, input, |input| {
                parse_font_oblique_angle(input, numeric)
            })?)
        };
        return Ok(CssFontFaceStyle::Oblique {
            range: Some(CssFontFaceObliqueRange::new(start, end)),
        });
    }
    common_font_style_keyword(ident)
        .map(CssFontFaceStyle::Keyword)
        .ok_or_else(|| fail(numeric, keyword_location, unexpected_at))
}

fn parse_font_face_width<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontFaceWidth> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssFontFaceWidth::Auto);
    }
    let start = from_component(numeric, input, |input| parse_font_width(input, numeric))?;
    let end = if input.only_whitespace_remains() {
        None
    } else {
        Some(from_component(numeric, input, |input| {
            parse_font_width(input, numeric)
        })?)
    };
    Ok(CssFontFaceWidth::Range { start, end })
}

/// `<font-display>` read from a token stream.
pub(super) fn parse_font_display<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontDisplay> {
    let location = input.site();
    let ident = from_here(input, Stream::consume_ident)?;
    match_ignore_ascii_case! { ident,
        "auto" => Ok(CssFontDisplay::Auto),
        "block" => Ok(CssFontDisplay::Block),
        "swap" => Ok(CssFontDisplay::Swap),
        "fallback" => Ok(CssFontDisplay::Fallback),
        "optional" => Ok(CssFontDisplay::Optional),
        _ => Err(fail(numeric, location, unexpected_at)),
    }
}

fn parse_unicode_range_list<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssUnicodeRangeList> {
    // List post-condition: a trailing comma is an incomplete descriptor.
    let ranges = unicode_range::parse_list(input, numeric)?;
    CssUnicodeRangeList::try_new(ranges).ok_or_else(|| fail(numeric, input.site(), unexpected_at))
}

fn next_is_comma(input: &Stream<'_>) -> bool {
    input.clone().consume_comma().is_ok()
}
