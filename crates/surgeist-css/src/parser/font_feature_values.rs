use std::cell::RefCell;
use std::rc::Rc;

use cssparser::{ParseError, Parser, ParserState};
use surgeist_css_parser::Token;
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, QualifiedRule, RuleGrammar};

use super::Recovered;
use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, grammar_error, location_at, offset_of,
};
use super::declaration_unit::DeclarationText;
use super::descriptor_stream::{
    Boundary, DescriptorGrammar, DescriptorRecovery, NativeValueError, Value, comma_list,
    consume_remaining, declaration_position, drive_descriptors_into, expect_exhausted, fail,
    from_remaining, native, parse_descriptor_checked, parse_descriptor_text, parse_scope,
    validate_root,
};
use super::property_input::StreamExt;
use super::recovery::RecoveryState;
use super::syntax_types::{Attached, Stream, SyntaxError};
use crate::component::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssNumericTokenKind,
    CssSerializedValue, CssValueOrigin, CssValueTokenRef,
};
use crate::declaration::{CssAuthoredDeclarationValue, CssSubstitutionDependentValue};
use crate::diagnostic::error::{
    descriptor_name_error, invalid_syntax, unexpected_syntax_token, with_at_rule_prelude_context,
    with_descriptor_context,
};
use crate::diagnostic::{CssError, CssFeatureId, CssRecoveryAction, CssRecoveryDiagnostic};
use crate::numeric::expression::NumericInputContext;
use crate::rule::font_face::CssFontFaceFamily;
use crate::rule::font_feature_values::descriptor_values::{
    FontFeatureDisplayData, FontFeatureValueData,
};
use crate::rule::font_feature_values::{
    CssFontFeatureDisplayOccurrence, CssFontFeatureDisplayValue, CssFontFeatureValue,
    CssFontFeatureValueBlock, CssFontFeatureValueDefinition, CssFontFeatureValueIndex,
    CssFontFeatureValueKind, CssFontFeatureValueName, CssFontFeatureValuesItem,
    CssFontFeatureValuesRule,
};
use crate::source::{CssParsedOrigin, CssSourcePosition, CssSourceSnapshot};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("later.rule.font-feature-values")];

pub(super) fn parse_families<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<Vec<CssFontFaceFamily>, ParseError<'i, CssError>> {
    parse_families_observed(source, input, recovery, |_, _| Ok(()))
}

// One grammar for rule ingress and raw family-list ingress. Observation adds
// provenance only; existing rule parsing needs no member-origin allocation.
pub(super) fn parse_families_observed<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
    mut observe: impl FnMut(
        std::ops::Range<usize>,
        cssparser::SourceLocation,
    ) -> Result<(), ParseError<'static, CssError>>,
) -> Result<Vec<CssFontFaceFamily>, ParseError<'i, CssError>> {
    recovery.check_specialized_components(source, input, "later.rule.font-feature-values")?;
    let snapshot = recovery.source_snapshot().clone();
    parse_scope(input, &snapshot, |input, numeric| {
        comma_list(input, |input| {
            let start = input.site();
            let location = input.site();
            let name = from_remaining(
                numeric,
                input,
                super::typography::parse_non_generic_font_family_name,
            )?;
            expect_exhausted(input)?;
            let family = CssFontFaceFamily::try_new(name.as_str())
                .ok_or_else(|| fail(numeric, location, invalid_syntax))?;
            let at = input.site();
            observe(
                offset_of(start)..offset_of(at),
                super::descriptor_stream::location(numeric, location),
            )
            .map_err(|error| SyntaxError::custom(at, error))?;
            Ok(family)
        })
    })
    .map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "font-feature-values",
            "later.rule.font-feature-values",
            "a nonempty list of non-generic font family names",
        )
    })
}

pub(super) fn parse_rule<'i>(
    source: &'i str,
    families: Vec<CssFontFaceFamily>,
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssFontFeatureValuesRule, ParseError<'i, CssError>> {
    let recovered = parse_outer_body(source, input, recovery)?;
    diagnostics.extend(recovered.diagnostics);
    CssFontFeatureValuesRule::try_new(families, recovered.syntax)
        .map(|rule| rule.with_position(position(start)))
        .map_err(|_| invalid_syntax(start.source_location()))
}

/// The items of the outer block the native parser `input` is inside.
pub(super) fn parse_outer_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: RecoveryState,
) -> Result<Recovered<Vec<CssFontFeatureValuesItem>>, ParseError<'i, CssError>> {
    let snapshot = recovery.source_snapshot().clone();
    let (members, diagnostics) = parse_scope(input, &snapshot, |contents, _| {
        Ok(drive_members(contents, source, None, &recovery))
    })?;
    let items = members
        .into_iter()
        .map(|member| match member {
            Member::Item(item) => item,
            Member::Definition(_) => unreachable!("outer body returns only mixed rule items"),
        })
        .collect();
    Ok(Recovered {
        syntax: items,
        diagnostics,
    })
}

/// The definitions of the block of `kind` the native parser `input` is inside.
pub(super) fn parse_definition_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    kind: CssFontFeatureValueKind,
    recovery: RecoveryState,
) -> Result<Recovered<Vec<CssFontFeatureValueDefinition>>, ParseError<'i, CssError>> {
    let snapshot = recovery.source_snapshot().clone();
    let (members, diagnostics) = parse_scope(input, &snapshot, |contents, _| {
        Ok(drive_members(contents, source, Some(kind), &recovery))
    })?;
    let definitions = members
        .into_iter()
        .map(|member| match member {
            Member::Definition(value) => value,
            Member::Item(_) => unreachable!("subsidiary bodies return only definitions"),
        })
        .collect();
    Ok(Recovered {
        syntax: definitions,
        diagnostics,
    })
}

/// Drives the units of the contents of a block: the outer block when `kind` is
/// none, and the block of the feature type `kind` otherwise.
fn drive_members<'i>(
    contents: &mut Stream<'i>,
    source: &str,
    kind: Option<CssFontFeatureValueKind>,
    recovery: &RecoveryState,
) -> (Vec<Member>, Vec<CssRecoveryDiagnostic>) {
    let sink = Rc::new(RefCell::new(Vec::new()));
    let mut grammar = BodyParser {
        source,
        recovery: recovery.clone(),
        kind,
        diagnostics: Rc::clone(&sink),
    };
    let mut readmit = BodyParser {
        source,
        recovery: recovery.clone(),
        kind,
        diagnostics: Rc::clone(&sink),
    };
    let members = drive_descriptors_into(
        contents,
        &mut grammar,
        &mut readmit,
        &DescriptorRecovery {
            source,
            recovery,
            production: "later.rule.font-feature-values",
            declaration: CssRecoveryAction::DropDescriptor,
            at_rule_first: true,
        },
        &sink,
    );
    drop((grammar, readmit));
    (members, sink.take())
}

fn position(start: &ParserState) -> CssSourcePosition {
    CssSourcePosition::from_cssparser(start.position(), start.source_location())
}

enum Member {
    Item(CssFontFeatureValuesItem),
    Definition(CssFontFeatureValueDefinition),
}

struct BodyParser<'i> {
    source: &'i str,
    recovery: RecoveryState,
    kind: Option<CssFontFeatureValueKind>,
    diagnostics: Rc<RefCell<Vec<CssRecoveryDiagnostic>>>,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for BodyParser<'_> {
    type Item = Member;
    type Error = NativeError<'i>;

    // The outer body admits the blocks of the feature types, whose own bodies
    // are definitions. A prelude that is rejected is reported at its start,
    // which tells it from a rejected block.
    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Member, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let name = rule.name();
        let prelude_site = rule.prelude().start_site();
        self.recovery
            .check_specialized_components_from(
                self.source,
                offset_of(prelude_site),
                "later.rule.font-feature-values",
            )
            .map_err(|error| grammar_error(prelude_site, error))?;
        let kind = self
            .kind
            .is_none()
            .then(|| CssFontFeatureValueKind::from_css_name(name))
            .flatten()
            .ok_or_else(|| SyntaxError::unknown_at_rule_error(prelude_site, name))?;
        expect_exhausted(&mut rule.prelude())?;
        let at = rule.start();
        let Some(mut block) = rule.block() else {
            return Err(grammar_error(
                at,
                unexpected_syntax_token(
                    location_at(snapshot, offset_of(rule.end())),
                    Token::Semicolon,
                ),
            ));
        };
        let opening = offset_of(block.start_site()).saturating_sub(1);
        let mut depth = self
            .recovery
            .enter_rule_block_at(self.source, opening, "later.rule.font-feature-values")
            .map_err(|error| grammar_error(at, error))?;
        let (definitions, recovered) =
            drive_members(&mut block, self.source, Some(kind), &self.recovery);
        let definitions = definitions
            .into_iter()
            .map(|member| match member {
                Member::Definition(value) => value,
                Member::Item(_) => unreachable!("subsidiary bodies return only definitions"),
            })
            .collect();
        let block = CssFontFeatureValueBlock::try_new(kind, definitions)
            .map_err(|_| grammar_error(at, invalid_syntax(location_at(snapshot, offset_of(at)))))?
            .with_position(declaration_position(snapshot, offset_of(at)));
        self.diagnostics.borrow_mut().extend(recovered);
        depth.retain();
        Ok(Member::Item(CssFontFeatureValuesItem::Block(block)))
    }

    // Syntax 3 (2026 CRD) section 5.5.5 reads a unit that is not a
    // declaration as a qualified rule, which this body does not admit.
    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Member, GrammarError<'i>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for BodyParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Member, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl<'i> DescriptorGrammar<'i, Member> for BodyParser<'_> {
    fn readmit(&mut self, name: &str, text: &DeclarationText) -> Result<Member, NativeError<'i>> {
        self.admit(name, text)
    }
}

impl BodyParser<'_> {
    /// Admits the declaration named `name` that lies at `text`.
    fn admit<'i>(&mut self, name: &str, text: &DeclarationText) -> Result<Member, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let implicit = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.descriptor",
        )?;
        let owner = self
            .kind
            .map_or("font-feature-values", CssFontFeatureValueKind::as_str);
        let name_origin = CssParsedOrigin::from_range(snapshot, text.name_range())
            .expect("actual declaration name");
        let start_location = location_at(snapshot, text.name_start());
        let position = declaration_position(snapshot, text.name_start());
        let result = if let Some(kind) = self.kind {
            let friendly = CssFontFeatureValueName::try_new(name.to_string())
                .map_err(|_| invalid_syntax(start_location))
                .map_err(|error| with_descriptor_context(error, owner, name))?;
            let value = parse_descriptor_text(
                snapshot,
                text.value_start,
                text.end,
                owner,
                name,
                Boundary::Value,
                |input, components, numeric, range| {
                    let data = feature_data(
                        input,
                        kind,
                        &components,
                        numeric,
                        crate::diagnostic::error::unexpected_component_value,
                    )?;
                    let origin = CssParsedOrigin::from_range(snapshot, range)
                        .expect("parser value boundaries belong to the original source");
                    Ok(CssFontFeatureValue::from_parsed(
                        kind, data, components, origin,
                    ))
                },
            )
            .map_err(|error| with_descriptor_context(error, owner, name))?;
            Member::Definition(
                CssFontFeatureValueDefinition::new(friendly, value)
                    .with_parsed_name(name_origin)
                    .with_position(position),
            )
        } else {
            if !name.eq_ignore_ascii_case("font-display") {
                return Err(with_descriptor_context(
                    descriptor_name_error(start_location, owner, name),
                    owner,
                    name,
                ));
            }
            let value = parse_descriptor_text(
                snapshot,
                text.value_start,
                text.end,
                owner,
                name,
                Boundary::ValidatedValue,
                |input, components, numeric, range| {
                    let data = display_data(input, &components, numeric)?;
                    let origin = CssParsedOrigin::from_range(snapshot, range)
                        .expect("parser value boundaries belong to the original source");
                    Ok(CssFontFeatureDisplayValue::from_parsed(
                        data, components, origin,
                    ))
                },
            )
            .map_err(|error| with_descriptor_context(error, owner, name))?;
            Member::Item(CssFontFeatureValuesItem::FontDisplay(
                CssFontFeatureDisplayOccurrence::new(value)
                    .with_parsed_name(name_origin)
                    .with_position(position),
            ))
        };
        self.recovery.retain_component_closures(implicit);
        Ok(result)
    }
}

// The rule owner and value fragments both report the responsible component
// (reference "Source coordinates and diagnostics") with this exact token grammar.
pub(super) fn indexes_from_components<'i>(
    values: &CssComponentValues,
    error_for_component: impl Fn(&CssComponentValue) -> ParseError<'i, CssError>,
) -> Result<Vec<CssFontFeatureValueIndex>, ParseError<'i, CssError>> {
    let mut indexes = Vec::new();
    for value in values.items() {
        match value.view() {
            CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_)) => {}
            CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                if number.kind() == CssNumericTokenKind::Integer =>
            {
                let index = CssFontFeatureValueIndex::try_from_decimal(number.representation())
                    .map_err(|_| error_for_component(value))?;
                indexes.push(match value.origin() {
                    CssValueOrigin::Parsed(origin) => index.with_origin(origin.clone()),
                    _ => index,
                });
            }
            _ => {
                return Err(error_for_component(value));
            }
        }
    }
    Ok(indexes)
}

fn qualifies<'i>(
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
    at: super::syntax_types::Site<'i>,
) -> Result<bool, super::descriptor_stream::ValueError<'i>> {
    super::variables::descriptor_environment_qualifies(components.items(), numeric).map_err(
        |error| {
            fail(numeric, at, |at| {
                crate::diagnostic::error::invalid_component_value(at, error)
            })
        },
    )
}

fn pending(input: &mut Stream<'_>) -> CssSubstitutionDependentValue {
    let start = input.site();
    consume_remaining(input);
    CssSubstitutionDependentValue::new(CssAuthoredDeclarationValue::new(
        input.authored_text_since(start),
    ))
}

/// The value of a `font-display` descriptor from the scope of the native
/// parser `input`.
pub(super) fn parse_display_value<'i>(
    input: &mut Parser<'i, '_>,
    snapshot: &CssSourceSnapshot,
) -> Result<FontFeatureDisplayData, ParseError<'i, CssError>> {
    let start = input.position().byte_index();
    let end = super::property_input::native_scope_end(input);
    let value = parse_descriptor_text(
        snapshot,
        start,
        end,
        "font-feature-values",
        "font-display",
        Boundary::ValidatedText,
        |input, components, numeric, _| display_data(input, &components, numeric),
    )?;
    super::property_input::advance_native_to(input, end);
    Ok(value)
}

fn display_data<'i>(
    input: &mut Stream<'i>,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, FontFeatureDisplayData> {
    if qualifies(components, numeric, input.site())? {
        return Ok(FontFeatureDisplayData::Pending(pending(input)));
    }
    let value = super::font_face::parse_font_display(input, numeric)?;
    expect_exhausted(input)?;
    Ok(FontFeatureDisplayData::Ordinary(value))
}

pub(crate) fn construct_display_value(
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<FontFeatureDisplayData, CssError> {
    let numeric = NumericInputContext::components(components, serialized);
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
        validate_root(&mut input, &numeric, "font-feature-values", "font-display")
            .map_err(|error| native(&numeric, &error))?;
        parse_descriptor_checked(
            &mut input,
            &numeric,
            "font-feature-values",
            "font-display",
            |input| display_data(input, components, &numeric),
        )
    })();
    result.map_err(|error| crate::diagnostic::error::from_parse_error(serialized.as_css(), error))
}

/// The value of a feature of `kind` from the scope of the native parser
/// `input`.
pub(super) fn parse_feature_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontFeatureValueKind,
    snapshot: &CssSourceSnapshot,
) -> Result<FontFeatureValueData, ParseError<'i, CssError>> {
    let start = input.position().byte_index();
    let end = super::property_input::native_scope_end(input);
    let value = parse_descriptor_text(
        snapshot,
        start,
        end,
        "font-feature-values",
        kind.as_str(),
        Boundary::Text,
        |input, components, numeric, _| {
            feature_data(
                input,
                kind,
                &components,
                numeric,
                crate::diagnostic::error::unexpected_component_value,
            )
        },
    )?;
    super::property_input::advance_native_to(input, end);
    Ok(value)
}

fn feature_data<'i>(
    input: &mut Stream<'i>,
    kind: CssFontFeatureValueKind,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
    error: impl Fn(&CssComponentValue, &'static str) -> NativeValueError,
) -> Value<'i, FontFeatureValueData> {
    // Annotation/boundary tokens are disallowed before whole-value qualification.
    for component in components.items() {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Semicolon | CssValueTokenRef::Delim('!'))
        ) {
            return Err(SyntaxError::custom(
                input.site(),
                error(component, "nonnegative integer tokens"),
            ));
        }
    }
    if qualifies(components, numeric, input.site())? {
        return Ok(FontFeatureValueData::Pending(pending(input)));
    }
    let indexes = indexes_from_components(components, |component| {
        error(component, "nonnegative integer tokens")
    })
    .map_err(|error| SyntaxError::custom(input.site(), error))?;
    consume_remaining(input);
    kind.validate_index_count(indexes.len()).map_err(|_| {
        let surplus = components
            .items()
            .iter()
            .filter(|component| {
                !matches!(
                    component.view(),
                    CssComponentValueRef::Comment(_)
                        | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                )
            })
            .enumerate()
            .find(|(index, _)| kind.validate_index_count(index + 1).is_err());
        let at = input.site();
        match surplus {
            Some((_, component)) => SyntaxError::custom(
                at,
                error(component, "the selected feature-value index count"),
            ),
            None => fail(numeric, at, |location| {
                crate::diagnostic::error::unexpected_end_at(
                    location,
                    "the selected feature-value index count",
                )
            }),
        }
    })?;
    Ok(FontFeatureValueData::Indexes(indexes))
}

pub(crate) fn construct_feature_value(
    kind: CssFontFeatureValueKind,
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<FontFeatureValueData, CssError> {
    let numeric = NumericInputContext::components(components, serialized);
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
        feature_data(
            &mut input,
            kind,
            components,
            &numeric,
            |component, expectation| {
                let index = components
                    .items()
                    .iter()
                    .position(|item| std::ptr::eq(item, component))
                    .expect("root grammar component");
                let offset = serialized
                    .component_offset_for_path(&[index])
                    .expect("serialized root component");
                crate::diagnostic::error::unexpected_component_value_at(
                    component,
                    expectation,
                    CssSourcePosition::from_byte_offset_in(serialized.as_css(), offset),
                )
            },
        )
        .map_err(|error| native(&numeric, &error))
    })();
    result.map_err(|error| crate::diagnostic::error::from_parse_error(serialized.as_css(), error))
}
