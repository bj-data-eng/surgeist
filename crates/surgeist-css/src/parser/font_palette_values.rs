use cssparser::{ParseError, Parser};
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, QualifiedRule, RuleGrammar};

use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, grammar_error, location_at, location_of, offset_of,
};
use super::declaration_unit::DeclarationText;
use super::descriptor_stream::{
    Boundary, DescriptorGrammar, DescriptorRecovery, Value, comma_list, consume_remaining,
    declaration_position, drive_descriptors, expect_exhausted, fail, from_component,
    from_remaining, native, parse_descriptor_checked, parse_descriptor_text, read_scope,
    validate_root,
};
use super::property_input::StreamExt;
use super::recovery::RecoveryState;
use super::syntax_types::{Attached, Stream, SyntaxError};
use crate::component::{CssComponentValues, CssSerializedValue};
use crate::declaration::{CssAuthoredDeclarationValue, CssSubstitutionDependentValue};
use crate::diagnostic::CssRecoveryDiagnostic;
use crate::diagnostic::error::{
    descriptor_name_error, from_parse_error, invalid_component_value, invalid_syntax,
    unexpected_at, with_at_rule_prelude_context, with_descriptor_context,
};
use crate::diagnostic::{CssError, CssFeatureId, CssRecoveryAction};
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssInteger, CssNonNegative, CssNumeric};
use crate::rule::font_face::CssFontFaceFamily;
use crate::rule::font_palette_values::rule::CssFontPaletteDescriptorData;
use crate::rule::font_palette_values::{
    CssFontPaletteBase, CssFontPaletteDescriptor, CssFontPaletteDescriptorKind,
    CssFontPaletteDescriptorValue, CssFontPaletteName, CssFontPaletteOverride,
    CssFontPaletteValuesRule,
};
use crate::source::CssSourcePosition;

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("later.rule.font-palette-values")];
pub(super) static IMPLEMENTED_DESCRIPTORS: &[CssFeatureId] = &[
    CssFeatureId::new("later.descriptor.font-palette-values.font-family"),
    CssFeatureId::new("later.descriptor.font-palette-values.base-palette"),
    CssFeatureId::new("later.descriptor.font-palette-values.override-colors"),
];

pub(super) fn parse_name<'i>(
    source: &str,
    input: &mut Stream<'i>,
    recovery: &RecoveryState,
) -> Result<CssFontPaletteName, ParseError<'i, CssError>> {
    recovery.check_specialized_components_from(
        source,
        offset_of(input.site()),
        "later.rule.font-palette-values",
    )?;
    read_scope(input, recovery.source_snapshot(), |input, numeric| {
        let location = input.site();
        let ident = input.consume_ident()?;
        expect_exhausted(input)?;
        CssFontPaletteName::try_new(ident).map_err(|_| fail(numeric, location, invalid_syntax))
    })
    .map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "font-palette-values",
            "later.rule.font-palette-values",
            "one dashed identifier",
        )
    })
}

pub(super) fn parse_rule<'i>(
    source: &str,
    name: CssFontPaletteName,
    contents: &mut Stream<'i>,
    position: CssSourcePosition,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssFontPaletteValuesRule, ParseError<'i, CssError>> {
    let descriptors = parse_body(source, contents, diagnostics, recovery);
    CssFontPaletteValuesRule::try_new(name, descriptors)
        .map(|rule| rule.with_position(position))
        .map_err(|_| invalid_syntax(location_of(position)))
}

/// The descriptors of the block `contents` holds.
pub(super) fn parse_body<'i>(
    source: &str,
    contents: &mut Stream<'i>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Vec<CssFontPaletteDescriptor> {
    let (descriptors, mut recovered) = drive_body(contents, source, &recovery);
    diagnostics.append(&mut recovered);
    descriptors
}

fn drive_body<'i>(
    contents: &mut Stream<'i>,
    source: &str,
    recovery: &RecoveryState,
) -> (Vec<CssFontPaletteDescriptor>, Vec<CssRecoveryDiagnostic>) {
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
            production: "later.rule.font-palette-values",
            declaration: CssRecoveryAction::DropDescriptor,
            at_rule_first: true,
        },
    )
}

struct BodyParser<'i> {
    source: &'i str,
    recovery: RecoveryState,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for BodyParser<'_> {
    type Item = CssFontPaletteDescriptor;
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

impl<'i> DescriptorGrammar<'i, CssFontPaletteDescriptor> for BodyParser<'_> {
    fn readmit(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssFontPaletteDescriptor, NativeError<'i>> {
        self.admit(name, text)
    }
}

impl BodyParser<'_> {
    /// Admits the descriptor named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssFontPaletteDescriptor, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let implicit = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.descriptor",
        )?;
        let kind = CssFontPaletteDescriptorKind::from_css_name(name).ok_or_else(|| {
            descriptor_name_error(
                location_at(snapshot, text.name_start()),
                "font-palette-values",
                name,
            )
        })?;
        let value = parse_text_value(snapshot, text.value_start, text.end, kind)
            .map_err(|error| with_descriptor_context(error, "font-palette-values", name))?;
        self.recovery.retain_component_closures(implicit);
        Ok(CssFontPaletteDescriptor::new(value)
            .with_position(declaration_position(snapshot, text.name_start())))
    }
}

/// The value of a descriptor from the text between `value_start` and `end`.
fn parse_text_value(
    snapshot: &crate::source::CssSourceSnapshot,
    value_start: usize,
    end: usize,
    kind: CssFontPaletteDescriptorKind,
) -> Result<CssFontPaletteDescriptorValue, ParseError<'static, CssError>> {
    parse_descriptor_text(
        snapshot,
        value_start,
        end,
        "font-palette-values",
        kind.as_str(),
        Boundary::Whole,
        |input, components, numeric, _| {
            let data = parse_value_data(input, kind, &components, numeric)?;
            expect_exhausted(input)?;
            Ok(CssFontPaletteDescriptorValue::from_parts(
                kind, components, data,
            ))
        },
    )
}

pub(super) fn parse_descriptor_value_from_parser<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontPaletteDescriptorKind,
    recovery: &RecoveryState,
) -> Result<CssFontPaletteDescriptorValue, ParseError<'i, CssError>> {
    let start = input.position().byte_index();
    let end = super::property_input::native_scope_end(input);
    let value = parse_text_value(recovery.source_snapshot(), start, end, kind)?;
    super::property_input::advance_native_to(input, end);
    Ok(value)
}

/// The component-native constructor uses the same grammar with strict numeric admission.
pub(crate) fn construct_descriptor_value(
    kind: CssFontPaletteDescriptorKind,
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<CssFontPaletteDescriptorData, CssError> {
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
        validate_root(&mut input, &numeric, "font-palette-values", kind.as_str())
            .map_err(|error| native(&numeric, &error))?;
        parse_descriptor_checked(
            &mut input,
            &numeric,
            "font-palette-values",
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
                "font-palette-values",
                kind.as_str(),
            ),
        )
    })
}

fn parse_value_data<'i>(
    input: &mut Stream<'i>,
    kind: CssFontPaletteDescriptorKind,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontPaletteDescriptorData> {
    let qualifying =
        super::variables::descriptor_substitution_qualifies(components.items(), numeric).map_err(
            |error| {
                fail(numeric, input.site(), |at| {
                    invalid_component_value(at, error)
                })
            },
        )?;
    if qualifying {
        let start = input.site();
        consume_remaining(input);
        return Ok(CssFontPaletteDescriptorData::Pending(
            CssSubstitutionDependentValue::new(CssAuthoredDeclarationValue::new(
                input.authored_text_since(start),
            )),
        ));
    }
    Ok(match kind {
        CssFontPaletteDescriptorKind::FontFamily => {
            CssFontPaletteDescriptorData::FontFamily(parse_families(input, numeric)?)
        }
        CssFontPaletteDescriptorKind::BasePalette => {
            CssFontPaletteDescriptorData::BasePalette(parse_base(input, numeric)?)
        }
        CssFontPaletteDescriptorKind::OverrideColors => {
            CssFontPaletteDescriptorData::OverrideColors(parse_overrides(input, numeric)?)
        }
    })
}

fn parse_families<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, Vec<CssFontFaceFamily>> {
    comma_list(input, |input| {
        let family = from_remaining(
            numeric,
            input,
            super::typography::parse_non_generic_font_family_name,
        )?;
        expect_exhausted(input)?;
        CssFontFaceFamily::try_new(family.as_str())
            .ok_or_else(|| fail(numeric, input.site(), invalid_syntax))
    })
}

fn parse_base<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssFontPaletteBase> {
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        if ident.eq_ignore_ascii_case("light") {
            return Ok(CssFontPaletteBase::Light);
        }
        if ident.eq_ignore_ascii_case("dark") {
            return Ok(CssFontPaletteBase::Dark);
        }
        input.restore(state);
    }
    parse_index(input, numeric).map(CssFontPaletteBase::Index)
}

fn parse_overrides<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, Vec<CssFontPaletteOverride>> {
    comma_list(input, |input| {
        let index = parse_index(input, numeric)?;
        let location = input.site();
        let color = from_component(numeric, input, |input| {
            super::color::parse_color(input, numeric)
        })?;
        let pair = CssFontPaletteOverride::try_new(index, color).map_err(|error| {
            if error.contextual_color().is_some() {
                fail(numeric, location, unexpected_at)
            } else {
                fail(numeric, location, invalid_syntax)
            }
        })?;
        expect_exhausted(input)?;
        Ok(pair)
    })
}

fn parse_index<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssNumeric<CssInteger, CssNonNegative>> {
    let location = input.site();
    let value = from_component(numeric, input, |input| {
        super::values::parse_integer_value(input, numeric)
    })?;
    value
        .try_narrow::<CssNonNegative>()
        .map_err(|_| fail(numeric, location, unexpected_at))
}
