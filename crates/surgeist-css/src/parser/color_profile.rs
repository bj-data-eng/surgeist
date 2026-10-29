use cssparser::{ParseError, Parser, ParserState};
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, QualifiedRule, RuleGrammar};

use super::block_units::{DeclarationUnit, GrammarError, NativeError, grammar_error, location_at};
use super::declaration_unit::DeclarationText;
use super::descriptor_stream::{
    Boundary, DescriptorGrammar, DescriptorRecovery, Value, comma_list, consume_remaining,
    declaration_position, drive_descriptors, expect_exhausted, fail, from_component, native,
    parse_descriptor_checked, parse_descriptor_text, parse_scope, validate_root,
};
use super::property_input::StreamExt;
use super::recovery::RecoveryState;
use super::syntax_types::{Attached, Stream, SyntaxError};
use crate::color::{CssColorProfileComponentName, CssColorProfileName};
use crate::component::{CssComponentValues, CssSerializedValue};
use crate::declaration::{CssAuthoredDeclarationValue, CssSubstitutionDependentValue};
use crate::diagnostic::error::{
    descriptor_name_error, from_parse_error, invalid_component_value, invalid_syntax,
    with_at_rule_prelude_context, with_descriptor_context,
};
use crate::diagnostic::{CssError, CssFeatureId, CssRecoveryAction, CssRecoveryDiagnostic};
use crate::numeric::expression::NumericInputContext;
use crate::rule::color_profile::rule::CssColorProfileDescriptorData;
use crate::rule::color_profile::{
    CssColorProfileDescriptor, CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
    CssColorProfileRenderingIntent, CssColorProfileRule, CssColorProfileRuleName,
};
use crate::source::CssSourcePosition;

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("interop.rule.color-profile")];
pub(super) static IMPLEMENTED_DESCRIPTORS: &[CssFeatureId] = &[
    CssFeatureId::new("interop.descriptor.color-profile.src"),
    CssFeatureId::new("interop.descriptor.color-profile.rendering-intent"),
    CssFeatureId::new("interop.descriptor.color-profile.components"),
];

pub(super) fn parse_name<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<CssColorProfileRuleName, ParseError<'i, CssError>> {
    recovery.check_specialized_components(source, input, "interop.rule.color-profile")?;
    parse_scope(input, recovery.source_snapshot(), |input, numeric| {
        let location = input.site();
        let ident = input.consume_ident()?;
        expect_exhausted(input)?;
        if ident.eq_ignore_ascii_case("device-cmyk") {
            Ok(CssColorProfileRuleName::DeviceCmyk)
        } else {
            CssColorProfileName::try_new(ident)
                .map(CssColorProfileRuleName::Custom)
                .ok_or_else(|| fail(numeric, location, invalid_syntax))
        }
    })
    .map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "color-profile",
            "interop.rule.color-profile",
            "one dashed identifier or device-cmyk",
        )
    })
}

pub(super) fn parse_rule<'i>(
    source: &'i str,
    name: CssColorProfileRuleName,
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssColorProfileRule, ParseError<'i, CssError>> {
    let descriptors = parse_body(source, input, diagnostics, recovery)?;
    Ok(CssColorProfileRule::new(name, descriptors).with_position(position(start)))
}

/// The descriptors of the block the native parser `input` is inside.
pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<Vec<CssColorProfileDescriptor>, ParseError<'i, CssError>> {
    let snapshot = recovery.source_snapshot().clone();
    let (descriptors, mut recovered) = parse_scope(input, &snapshot, |contents, _| {
        Ok(drive_body(contents, source, &recovery))
    })?;
    diagnostics.append(&mut recovered);
    Ok(descriptors)
}

fn drive_body<'i>(
    contents: &mut Stream<'i>,
    source: &str,
    recovery: &RecoveryState,
) -> (Vec<CssColorProfileDescriptor>, Vec<CssRecoveryDiagnostic>) {
    let mut grammar = BodyParser {
        source,
        recovery: recovery.clone(),
    };
    let mut readmit = BodyParser {
        source,
        recovery: recovery.clone(),
    };
    drive_descriptors(
        contents,
        &mut grammar,
        &mut readmit,
        &DescriptorRecovery {
            source,
            recovery,
            production: "interop.rule.color-profile",
            declaration: CssRecoveryAction::DropDescriptor,
            at_rule_first: true,
        },
    )
}

fn position(start: &ParserState) -> CssSourcePosition {
    CssSourcePosition::from_cssparser(start.position(), start.source_location())
}

struct BodyParser<'i> {
    source: &'i str,
    recovery: RecoveryState,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for BodyParser<'_> {
    type Item = CssColorProfileDescriptor;
    type Error = NativeError<'i>;

    // The body is a descriptor list: an at-rule is not valid in it.
    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        Err(SyntaxError::unknown_at_rule_error(
            rule.prelude().start_site(),
            rule.name(),
        ))
    }

    // Syntax 3 (2026 CRD) section 5.5.5 reads a unit that is not a
    // declaration as a qualified rule, which a descriptor list does not admit.
    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for BodyParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl<'i> DescriptorGrammar<'i, CssColorProfileDescriptor> for BodyParser<'_> {
    fn readmit(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssColorProfileDescriptor, NativeError<'i>> {
        self.admit(name, text)
    }
}

impl BodyParser<'_> {
    /// Admits the descriptor named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssColorProfileDescriptor, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let implicit = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.descriptor",
        )?;
        let kind = CssColorProfileDescriptorKind::from_css_name(name).ok_or_else(|| {
            descriptor_name_error(
                location_at(snapshot, text.name_start()),
                "color-profile",
                name,
            )
        })?;
        let value = parse_text_value(snapshot, text.value_start, text.end, kind)
            .map_err(|error| with_descriptor_context(error, "color-profile", name))?;
        self.recovery.retain_component_closures(implicit);
        Ok(CssColorProfileDescriptor::new(value)
            .with_position(declaration_position(snapshot, text.name_start())))
    }
}

/// The value of a descriptor from the text between `value_start` and `end`.
fn parse_text_value(
    snapshot: &crate::source::CssSourceSnapshot,
    value_start: usize,
    end: usize,
    kind: CssColorProfileDescriptorKind,
) -> Result<CssColorProfileDescriptorValue, ParseError<'static, CssError>> {
    parse_descriptor_text(
        snapshot,
        value_start,
        end,
        "color-profile",
        kind.as_str(),
        Boundary::Whole,
        |input, components, numeric, _| {
            let data = parse_value_data(input, kind, &components, numeric)?;
            expect_exhausted(input)?;
            Ok(CssColorProfileDescriptorValue::from_parts(
                kind, components, data,
            ))
        },
    )
}

pub(super) fn parse_descriptor_value_from_parser<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssColorProfileDescriptorKind,
    recovery: &RecoveryState,
) -> Result<CssColorProfileDescriptorValue, ParseError<'i, CssError>> {
    let start = input.position().byte_index();
    let end = super::property_input::native_scope_end(input);
    let value = parse_text_value(recovery.source_snapshot(), start, end, kind)?;
    super::property_input::advance_native_to(input, end);
    Ok(value)
}

/// The component-native constructor uses the same grammar with strict numeric admission.
pub(crate) fn construct_descriptor_value(
    kind: CssColorProfileDescriptorKind,
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<CssColorProfileDescriptorData, CssError> {
    let numeric = NumericInputContext::components(components, serialized);
    let result = (|| {
        let document = crate::syntax_consumption::checked_document(
            components.items(),
            crate::syntax_consumption::SyntaxInputLimits::default(),
            0,
        )
        .map_err(|error| {
            invalid_component_value(cssparser::SourceLocation { line: 0, column: 1 }, error)
        })?;
        let mut input = document.root();
        validate_root(&mut input, &numeric, "color-profile", kind.as_str())
            .map_err(|error| native(&numeric, &error))?;
        parse_descriptor_checked(
            &mut input,
            &numeric,
            "color-profile",
            kind.as_str(),
            |input| {
                let data = parse_value_data(input, kind, components, &numeric)?;
                expect_exhausted(input)?;
                Ok(data)
            },
        )
    })();
    result.map_err(|error| {
        from_parse_error(
            serialized.as_css(),
            crate::diagnostic::error::with_descriptor_value_context(
                error,
                "color-profile",
                kind.as_str(),
            ),
        )
    })
}

fn parse_value_data<'i>(
    input: &mut Stream<'i>,
    kind: CssColorProfileDescriptorKind,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssColorProfileDescriptorData> {
    let qualifying =
        super::variables::descriptor_environment_qualifies(components.items(), numeric).map_err(
            |error| {
                fail(numeric, input.site(), |at| {
                    invalid_component_value(at, error)
                })
            },
        )?;
    if qualifying {
        let start = input.site();
        consume_remaining(input);
        return Ok(CssColorProfileDescriptorData::Pending(
            CssSubstitutionDependentValue::new(CssAuthoredDeclarationValue::new(
                input.authored_text_since(start),
            )),
        ));
    }
    Ok(match kind {
        CssColorProfileDescriptorKind::Src => {
            CssColorProfileDescriptorData::Src(from_component(numeric, input, |input| {
                super::url::parse_url(input, numeric)
            })?)
        }
        CssColorProfileDescriptorKind::RenderingIntent => {
            let location = input.site();
            let ident = input.consume_ident()?;
            CssColorProfileDescriptorData::RenderingIntent(
                CssColorProfileRenderingIntent::from_css_name(ident)
                    .ok_or_else(|| fail(numeric, location, invalid_syntax))?,
            )
        }
        CssColorProfileDescriptorKind::Components => {
            CssColorProfileDescriptorData::Components(comma_list(input, |input| {
                let location = input.site();
                let ident = input.consume_ident()?;
                expect_exhausted(input)?;
                CssColorProfileComponentName::try_new(ident)
                    .ok_or_else(|| fail(numeric, location, invalid_syntax))
            })?)
        }
    })
}
