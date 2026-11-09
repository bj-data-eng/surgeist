//! Public error values state what failed in prose and chain their causes.
//!
//! Each expected message is the documented meaning of the variant: the rule of
//! the pinned HTML Standard the value breaks, or the resource limit it names.
//! A wrapper states its own operation and exposes the wrapped failure through
//! `Error::source`; its text does not repeat the cause.

use std::error::Error;

use crate::html::support::construction_limits;
use std::convert::Infallible;
use surgeist_html::bounded::{
    attributes::HtmlAutocompleteMaterializationError,
    numeric::{
        HtmlFloatingPointAllocationError, HtmlIntegerRecoveryError, HtmlIntegerUnitStepError,
        HtmlNumericMaterializationError, HtmlNumericParseError, HtmlRangeNumericError,
    },
};
use surgeist_html::{
    attributes::microdata::HtmlMicrodataTokenKind,
    attributes::{
        HtmlAttributeConformanceError, HtmlAttributeContextConformanceError,
        accept::HtmlAcceptSpecifierError,
        area::HtmlAreaCoordinatesAuthorError,
        autocomplete::{
            HtmlAutocompleteSectionError, HtmlAutocompleteSyntaxError, HtmlAutofillField,
        },
        command::HtmlCustomCommandError,
        encoding_label::HtmlUtf8EncodingLabel,
        image_values::{HtmlImageAttributeConformanceError, HtmlInputImageAttributeError},
        link_destination::HtmlLinkDestinationContextError,
        meter::{HtmlMeterAuthorRelation, HtmlMeterNativeError, HtmlMeterNumberRole},
        microdata::HtmlMicrodataTokenListError,
        navigable_target::HtmlNavigableTargetError,
        progress::HtmlProgressNumberAuthorError,
        sandbox::{HtmlSandboxRedundancy, HtmlSandboxValueError},
        scalar::HtmlNumericAttributeConformanceError,
        srcset::{
            HtmlSrcsetAuthorViolation, HtmlSrcsetContextViolation, HtmlSrcsetDensityError,
            HtmlSrcsetInterpretationError, HtmlSrcsetParseError, HtmlSrcsetUrlTokenError,
        },
        token_declarations::{HtmlIconSizeError, HtmlTokenDeclarationError},
    },
    construction::HtmlProcessingInstructionError,
    elements::input::{HtmlInputSelection, HtmlInputSelectionDirection, input_filename_setter},
    microsyntax::{
        numeric::{
            HtmlDecimalDigitsError, HtmlNonNegativeInteger, HtmlNonNegativeRational,
            HtmlRationalError,
        },
        temporal::HtmlTemporalError,
        tokens::{
            HtmlDuplicateTokenError, HtmlOrderedTokenSet, HtmlSpaceToken, HtmlTokenComparison,
            HtmlTokenError,
        },
    },
    values::{
        DocumentString, HtmlCustomElementNameError, HtmlValueAdmissionError, HtmlValueFailure,
        HtmlValueInterpreter, HtmlValueRepresentationError, HtmlValueSyntaxError,
    },
    vocabulary::KnownHtmlElement,
};

use surgeist_html::bounded::{
    media::{HtmlMediaActionError, HtmlMediaInterpretation},
    microdata::{HtmlMicrodataTokenActionError, HtmlMicrodataTokenInterpretation},
    rel::{HtmlRelActionError, HtmlRelInputs, HtmlRelInterpretation},
    temporal::HtmlTimeElementInterpretation,
};
use surgeist_html::{
    construction::{
        HtmlConstructionCause, HtmlConstructionLimits, HtmlConstructionResource, HtmlDocument,
        HtmlDocumentMode, HtmlFragment,
    },
    security::csp::{HtmlCspError, HtmlCspOwner, HtmlCspProcessorAuthority},
    values::{Attribute, Namespace, OrderedAttributes, QualifiedName},
};

/// The admission error of a three-unit value against a two-unit capacity.
fn refused_admission() -> HtmlValueAdmissionError {
    HtmlValueInterpreter::new(2)
        .admit_attribute_raw(Some(&DocumentString::from("abc")))
        .expect_err("three units exceed the two-unit capacity")
}

/// The duplicate-token error of the two spellings `a` and `A` compared ASCII
/// case-insensitively.
fn duplicate_token() -> HtmlDuplicateTokenError {
    let tokens = ["a", "A"].map(|token| HtmlSpaceToken::new(token).expect("valid token"));
    HtmlOrderedTokenSet::new(HtmlTokenComparison::AsciiCaseInsensitive, tokens)
        .expect_err("the spellings collide")
}

fn message(error: &dyn Error) -> String {
    error.to_string()
}

fn cause(error: &dyn Error) -> Option<String> {
    error.source().map(|source| source.to_string())
}

#[test]
fn numeric_parse_errors_name_the_violated_rule() {
    assert_eq!(
        message(&HtmlNumericParseError::InvalidSyntax),
        "value does not match the HTML numeric syntax"
    );
    assert_eq!(
        message(&HtmlNumericParseError::Negative),
        "value is negative where a nonnegative value is required"
    );
    assert_eq!(
        message(&HtmlNumericParseError::FloatingPointOverflow),
        "value overflows the finite binary64 range"
    );
    assert_eq!(
        message(&HtmlNumericParseError::ZeroDimension),
        "dimension is zero where a nonzero value is required"
    );
}

#[test]
fn integer_digit_boundaries_report_their_exact_numbers() {
    assert_eq!(
        message(&HtmlIntegerUnitStepError::RepresentationExhausted),
        "integer digit count cannot be represented on this platform"
    );
    assert_eq!(
        message(&HtmlIntegerUnitStepError::DigitLimitExceeded {
            limit: 4,
            required: 9
        }),
        "integer needs 9 digits, exceeding the limit of 4"
    );
    assert_eq!(
        message(&HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
            limit: 16,
            required: 40
        }),
        "integer needs 40 bytes of digit storage, exceeding the limit of 16"
    );
    assert_eq!(
        message(&HtmlIntegerUnitStepError::ActualCapacityLimitExceeded {
            limit: 16,
            requested: 10,
            capacity_bytes: 32
        }),
        "allocating 10 bytes of digit storage produced 32 bytes, exceeding the limit of 16"
    );
    assert_eq!(
        message(&HtmlIntegerUnitStepError::AllocationFailed { requested: 7 }),
        "allocating 7 bytes of digit storage failed"
    );
}

#[test]
fn floating_point_allocation_boundaries_report_their_exact_numbers() {
    assert_eq!(
        message(&HtmlFloatingPointAllocationError::RepresentationExhausted),
        "floating-point token length cannot be represented on this platform"
    );
    assert_eq!(
        message(
            &HtmlFloatingPointAllocationError::RequestedCapacityLimitExceeded {
                limit: 8,
                required: 12
            }
        ),
        "floating-point token needs 12 bytes of storage, exceeding the limit of 8"
    );
    assert_eq!(
        message(
            &HtmlFloatingPointAllocationError::ActualCapacityLimitExceeded {
                limit: 8,
                requested: 6,
                capacity_bytes: 24
            }
        ),
        "allocating 6 bytes of floating-point token storage produced 24 bytes, exceeding the limit of 8"
    );
    assert_eq!(
        message(&HtmlFloatingPointAllocationError::AllocationFailed { requested: 5 }),
        "allocating 5 bytes of floating-point token storage failed"
    );
}

#[test]
fn numeric_wrappers_chain_the_wrapped_failure() {
    let syntax = HtmlIntegerRecoveryError::Syntax(HtmlNumericParseError::Negative);
    assert_eq!(message(&syntax), "integer recovery rejected the input");
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value is negative where a nonnegative value is required")
    );
    let allocation =
        HtmlIntegerRecoveryError::Allocation(HtmlIntegerUnitStepError::AllocationFailed {
            requested: 3,
        });
    assert_eq!(
        message(&allocation),
        "integer recovery could not store its digits"
    );
    assert_eq!(
        cause(&allocation).as_deref(),
        Some("allocating 3 bytes of digit storage failed")
    );

    let digits = HtmlNumericMaterializationError::Digits(HtmlDecimalDigitsError::Empty);
    assert_eq!(
        message(&digits),
        "numeric digits are not a valid decimal magnitude"
    );
    assert_eq!(
        cause(&digits).as_deref(),
        Some("HTML integer magnitude has no decimal digits")
    );
    let boundary = HtmlNumericMaterializationError::Boundary(
        HtmlIntegerUnitStepError::RepresentationExhausted,
    );
    assert_eq!(
        message(&boundary),
        "numeric materialization reached an allocation boundary"
    );
    assert_eq!(
        cause(&boundary).as_deref(),
        Some("integer digit count cannot be represented on this platform")
    );
}

#[test]
fn rational_with_zero_denominator_says_so() {
    let zero = HtmlNonNegativeInteger::from(0u128);
    let one = HtmlNonNegativeInteger::from(1u128);
    let error = HtmlNonNegativeRational::new(one, zero).expect_err("zero denominator");
    assert_eq!(error, HtmlRationalError::ZeroDenominator);
    assert_eq!(message(&error), "rational denominator is zero");
}

#[test]
fn range_numeric_errors_name_the_failed_step() {
    assert_eq!(
        message(&HtmlRangeNumericError::Integrity),
        "range numeric reply does not match its pending request"
    );
    assert_eq!(
        message(&HtmlRangeNumericError::Representation),
        "range numeric length cannot be represented on this platform"
    );
    assert_eq!(
        message(&HtmlRangeNumericError::Overflow),
        "range numeric value exceeds the representable range"
    );
    assert_eq!(
        message(&HtmlRangeNumericError::InvalidSyntax),
        "range numeric input does not match the HTML numeric syntax"
    );
}

#[test]
fn custom_element_name_errors_follow_the_valid_custom_element_name_rules() {
    assert_eq!(
        message(&HtmlCustomElementNameError::Empty),
        "custom-element name is empty"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii),
        "custom-element name does not start with a lowercase ASCII letter"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::ForbiddenCodeUnit {
            code_unit_offset: 3,
            code_unit: 0x41
        }),
        "custom-element name contains forbidden code unit U+0041 at UTF-16 offset 3"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::MissingHyphen),
        "custom-element name contains no hyphen"
    );
    assert_eq!(
        message(&HtmlCustomElementNameError::ReservedName),
        "custom-element name is reserved by HTML"
    );
}

#[test]
fn token_errors_locate_the_offending_code_unit() {
    assert_eq!(
        message(&HtmlTokenError::EmptySpaceToken),
        "space-separated token is empty"
    );
    assert_eq!(
        message(&HtmlTokenError::SpaceTokenWhitespace {
            code_unit_offset: 2
        }),
        "space-separated token contains ASCII whitespace at UTF-16 offset 2"
    );
    assert_eq!(
        message(&HtmlTokenError::CommaTokenBoundaryWhitespace {
            code_unit_offset: 0
        }),
        "comma-separated token has leading or trailing ASCII whitespace at UTF-16 offset 0"
    );
    assert_eq!(
        message(&HtmlTokenError::CommaTokenComma {
            code_unit_offset: 4
        }),
        "comma-separated token contains a comma at UTF-16 offset 4"
    );
}

#[test]
fn processing_instruction_errors_name_the_violated_constraint() {
    assert_eq!(
        message(&HtmlProcessingInstructionError::InvalidTarget),
        "processing-instruction target is not a valid XML name"
    );
    assert_eq!(
        message(&HtmlProcessingInstructionError::ReservedTarget),
        "processing-instruction target is reserved (xml or xml-stylesheet)"
    );
    assert_eq!(
        message(&HtmlProcessingInstructionError::DataContainsGreaterThan),
        "processing-instruction data contains '>'"
    );
    assert_eq!(
        message(&HtmlProcessingInstructionError::DataEndsWithQuestionMark),
        "processing-instruction data ends with '?'"
    );
}

#[test]
fn token_declaration_errors_identify_the_token_and_chain_the_cause() {
    let duplicate = duplicate_token();
    assert_eq!(
        message(&HtmlTokenDeclarationError::Duplicate(duplicate)),
        "token declaration contains a duplicate token"
    );
    assert_eq!(
        message(&HtmlTokenDeclarationError::AccessKeyLength { token_index: 1 }),
        "accesskey token at index 1 is not exactly one code point"
    );
    assert_eq!(
        message(&HtmlTokenDeclarationError::InvalidBlockingToken { token_index: 2 }),
        "blocking token at index 2 is not a recognized keyword"
    );
    let size = HtmlTokenDeclarationError::InvalidIconSize {
        token_index: 0,
        reason: HtmlIconSizeError::LeadingZero,
    };
    assert_eq!(message(&size), "icon size token at index 0 is invalid");
    assert_eq!(
        cause(&size).as_deref(),
        Some("icon size dimension has a leading zero")
    );
}

#[test]
fn icon_size_errors_follow_the_sizes_grammar() {
    assert_eq!(
        message(&HtmlIconSizeError::ZeroDimension),
        "icon size has a zero width or height"
    );
    assert_eq!(
        message(&HtmlIconSizeError::LeadingZero),
        "icon size dimension has a leading zero"
    );
    assert_eq!(
        message(&HtmlIconSizeError::Separator),
        "icon size dimensions are not separated by a single 'x' or 'X'"
    );
    assert_eq!(
        message(&HtmlIconSizeError::DimensionDigits),
        "icon size dimension is not a sequence of ASCII digits"
    );
}

#[test]
fn meter_errors_name_the_attribute_roles_and_the_ordering_rule() {
    let syntax = HtmlMeterNativeError::Syntax {
        role: HtmlMeterNumberRole::Low,
        error: HtmlValueSyntaxError::InvalidNumericSyntax,
    };
    assert_eq!(
        message(&syntax),
        "meter low attribute is not a valid floating-point number"
    );
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value does not match the selected numeric syntax")
    );
    assert_eq!(
        message(&HtmlMeterNativeError::RelationViolation(
            HtmlMeterAuthorRelation::MinimumValue
        )),
        "meter min must not exceed value"
    );
    assert_eq!(
        message(&HtmlMeterNativeError::RelationViolation(
            HtmlMeterAuthorRelation::OptimumMaximum
        )),
        "meter optimum must not exceed max"
    );
    assert_eq!(
        message(&HtmlMeterNativeError::RepresentationUnavailable),
        "meter attributes cannot be compared in the selected numeric model"
    );
}

#[test]
fn autocomplete_materialization_errors_report_the_charged_resource() {
    assert_eq!(
        message(&HtmlAutocompleteMaterializationError::BytesLimit {
            limit: 10,
            required: 20
        }),
        "autocomplete materialization needs 20 allocated bytes, exceeding the limit of 10"
    );
    assert_eq!(
        message(&HtmlAutocompleteMaterializationError::OutputLimit {
            limit: 1,
            required: 2
        }),
        "autocomplete materialization needs 2 nested output records, exceeding the limit of 1"
    );
    assert_eq!(
        message(&HtmlAutocompleteMaterializationError::ScratchLimit {
            limit: 3,
            required: 5
        }),
        "autocomplete materialization needs 5 scratch records, exceeding the limit of 3"
    );
    assert_eq!(
        message(&HtmlAutocompleteMaterializationError::Allocation {
            requested_bytes: 64
        }),
        "allocating 64 bytes for autocomplete materialization failed"
    );
    assert_eq!(
        message(&HtmlAutocompleteMaterializationError::Representation),
        "autocomplete materialization size cannot be represented on this platform"
    );
    let admission = refused_admission();
    let refused = HtmlAutocompleteMaterializationError::RawAdmission(admission);
    assert_eq!(
        message(&refused),
        "autocomplete value was refused by raw-value admission"
    );
    assert_eq!(cause(&refused), Some(admission.to_string()));
}

#[test]
fn autocomplete_section_errors_chain_the_token_failure() {
    let token = HtmlAutocompleteSectionError::Token(HtmlTokenError::EmptySpaceToken);
    assert_eq!(
        message(&token),
        "autocomplete section is not a valid space-separated token"
    );
    assert_eq!(
        cause(&token).as_deref(),
        Some("space-separated token is empty")
    );
    assert_eq!(
        message(&HtmlAutocompleteSectionError::MissingSectionPrefix),
        "autocomplete section does not start with \"section-\""
    );
}

#[test]
fn input_image_and_microdata_and_accept_errors_state_their_rule() {
    assert_eq!(
        message(&HtmlInputImageAttributeError::EmptyAlt),
        "image-button alt attribute is empty"
    );
    assert_eq!(
        message(&HtmlInputImageAttributeError::EmptySource),
        "image-button src attribute is empty"
    );
    assert_eq!(
        message(&HtmlMicrodataTokenListError::Empty),
        "microdata token list contains no tokens"
    );
    assert_eq!(
        message(&HtmlAcceptSpecifierError::MissingLeadingDot),
        "accept file extension does not start with a full stop"
    );
    let token = HtmlAcceptSpecifierError::Token(HtmlTokenError::CommaTokenComma {
        code_unit_offset: 1,
    });
    assert_eq!(
        message(&token),
        "accept specifier is not a valid comma-separated token"
    );
    assert_eq!(
        cause(&token).as_deref(),
        Some("comma-separated token contains a comma at UTF-16 offset 1")
    );
}

#[test]
fn srcset_value_errors_follow_the_image_candidate_grammar() {
    assert_eq!(
        message(&HtmlSrcsetUrlTokenError::Empty),
        "srcset image candidate URL is empty"
    );
    assert_eq!(
        message(&HtmlSrcsetUrlTokenError::AsciiWhitespace),
        "srcset image candidate URL contains ASCII whitespace"
    );
    assert_eq!(
        message(&HtmlSrcsetUrlTokenError::EdgeComma),
        "srcset image candidate URL starts or ends with a comma"
    );
    assert_eq!(
        message(&HtmlSrcsetDensityError::InvalidSyntax),
        "srcset density is not a valid floating-point number"
    );
    assert_eq!(
        message(&HtmlSrcsetDensityError::Nonpositive),
        "srcset density is not greater than zero"
    );
    assert_eq!(
        message(&HtmlSrcsetDensityError::RepresentationExhausted),
        "srcset density size cannot be represented on this platform"
    );
}

#[test]
fn srcset_author_violations_name_the_candidates_involved() {
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::EmptyList),
        "srcset has no image candidate strings"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::MissingCandidate),
        "srcset has a comma without an image candidate string"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::InvalidUrlToken),
        "srcset image candidate URL is not a valid URL token"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::InvalidDescriptor),
        "srcset image candidate descriptor is not a valid width or density"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::NonpositiveDescriptor),
        "srcset image candidate descriptor is not greater than zero"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::MultipleDescriptors),
        "srcset image candidate has more than one descriptor"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::FutureHeightDescriptor),
        "srcset image candidate uses the unsupported height descriptor"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::DuplicateWidth {
            first: 0,
            duplicate: 2
        }),
        "srcset image candidate 2 repeats the width of candidate 0"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::DuplicateDensity {
            first: 1,
            duplicate: 3
        }),
        "srcset image candidate 3 repeats the density of candidate 1"
    );
    assert_eq!(
        message(&HtmlSrcsetAuthorViolation::MixedWidthDescriptors),
        "srcset has a candidate without a width descriptor beside width descriptors"
    );
}

#[test]
fn srcset_interpretation_errors_chain_raw_admission() {
    let admission = refused_admission();
    let refused = HtmlSrcsetInterpretationError::RawAdmission(admission);
    assert_eq!(
        message(&refused),
        "srcset value was refused by raw-value admission"
    );
    assert_eq!(cause(&refused), Some(admission.to_string()));
    let exhausted = HtmlSrcsetInterpretationError::RepresentationExhausted;
    assert_eq!(
        message(&exhausted),
        "srcset interpretation size cannot be represented on this platform"
    );
    assert!(exhausted.source().is_none());
}

#[test]
fn temporal_syntax_failures_chain_through_value_syntax() {
    let syntax = HtmlValueSyntaxError::Temporal(HtmlTemporalError::ZeroYear);
    assert_eq!(message(&syntax), "value is not a valid temporal string");
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("HTML year must be greater than zero")
    );
}

#[test]
fn common_value_failures_split_syntax_from_representation() {
    use HtmlValueSyntaxError as Syntax;
    assert_eq!(
        message(&Syntax::EmptyLegacyColor),
        "legacy color value is empty"
    );
    assert_eq!(
        message(&Syntax::TransparentLegacyColor),
        "legacy color value is the transparent keyword"
    );
    assert_eq!(
        message(&Syntax::InvalidNumericSyntax),
        "value does not match the selected numeric syntax"
    );
    assert_eq!(
        message(&Syntax::NegativeNonnegativeInteger),
        "nonnegative integer recovery produced a negative integer"
    );
    assert_eq!(
        message(&Syntax::ZeroNonzeroDimension),
        "nonzero dimension recovery produced zero"
    );
    assert_eq!(
        message(&HtmlValueRepresentationError::FloatingPointOverflow),
        "decimal number rounds to infinity in binary64"
    );
    let syntax = HtmlValueFailure::Syntax(Syntax::EmptyLegacyColor);
    assert_eq!(
        message(&syntax),
        "value recovery failed on source-defined syntax"
    );
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("legacy color value is empty")
    );
    let representation =
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow);
    assert_eq!(
        message(&representation),
        "value recovery produced an unrepresentable value"
    );
    assert_eq!(
        cause(&representation).as_deref(),
        Some("decimal number rounds to infinity in binary64")
    );
}

#[test]
fn attribute_conformance_errors_name_the_violated_author_requirement() {
    use HtmlAttributeConformanceError as E;
    let plain = [
        (
            E::UnrecognizedKeyword,
            "attribute value is not a recognized keyword",
        ),
        (
            E::NonconformingKeyword,
            "attribute keyword is forbidden in authored markup",
        ),
        (
            E::BooleanSpelling,
            "boolean attribute value is neither empty nor the attribute's name",
        ),
        (
            E::BdoDirectionRequired,
            "bdo element requires a dir attribute",
        ),
        (
            E::BdoDirectionMustBeLtrOrRtl,
            "bdo element dir attribute must be ltr or rtl",
        ),
        (
            E::DetailsNameMustNotBeEmpty,
            "details name attribute must not be empty",
        ),
        (
            E::FormNameMustNotBeEmpty,
            "form name attribute must not be empty",
        ),
        (
            E::DirnameMustNotBeEmpty,
            "dirname attribute must not be empty",
        ),
    ];
    for (error, expected) in plain {
        assert_eq!(message(&error), expected);
        assert!(error.source().is_none());
    }
    assert_eq!(
        message(&E::InputPlaceholderContainsNewline {
            code_unit_offset: 5
        }),
        "input placeholder contains a line break at UTF-16 offset 5"
    );
    let id = E::InvalidIdToken(HtmlTokenError::EmptySpaceToken);
    assert_eq!(message(&id), "id attribute is not a valid token");
    assert_eq!(
        cause(&id).as_deref(),
        Some("space-separated token is empty")
    );
    let map = E::InvalidMapNameToken(HtmlTokenError::SpaceTokenWhitespace {
        code_unit_offset: 1,
    });
    assert_eq!(message(&map), "map name attribute is not a valid token");
    assert_eq!(
        cause(&map).as_deref(),
        Some("space-separated token contains ASCII whitespace at UTF-16 offset 1")
    );
    let target = E::InvalidNavigableTarget(HtmlNavigableTargetError::Empty);
    assert_eq!(
        message(&target),
        "attribute is not a valid navigable target name"
    );
    assert_eq!(
        cause(&target).as_deref(),
        Some("navigable target name is empty")
    );
    let label =
        E::InvalidUtf8EncodingLabel(HtmlUtf8EncodingLabel::new("latin1").expect_err("not UTF-8"));
    assert_eq!(
        message(&label),
        "encoding label is not the required UTF-8 spelling"
    );
    assert!(label.source().is_some());
    let name = E::InvalidCustomElementName(HtmlCustomElementNameError::MissingHyphen);
    assert_eq!(
        message(&name),
        "is attribute is not a valid custom-element name"
    );
    assert_eq!(
        cause(&name).as_deref(),
        Some("custom-element name contains no hyphen")
    );
}

#[test]
fn context_conformance_errors_name_the_missing_context() {
    use HtmlAttributeContextConformanceError as E;
    let cases = [
        (
            E::ImageMapRequiresAncestorAnchorWithHref,
            "image map attribute requires an ancestor a element with an href attribute",
        ),
        (
            E::ImageControlsRequireNonemptyAlt,
            "image controls attribute requires a nonempty alt attribute",
        ),
        (
            E::StylesheetDisabledRequiresStylesheetRel,
            "disabled attribute on a link requires the stylesheet relationship",
        ),
        (
            E::ReferrerPolicyRequiresHref,
            "referrerpolicy attribute requires an href attribute",
        ),
        (
            E::InputStateForbidsAttribute,
            "attribute is prohibited for the input element's type state",
        ),
        (
            E::ScriptContextForbidsAttribute,
            "attribute is prohibited for this script type and placement",
        ),
        (
            E::FormnovalidateRequiresSubmitButton,
            "formnovalidate attribute requires a submit button",
        ),
        (
            E::FormOverrideRequiresSubmitButton,
            "formmethod and formenctype attributes require a submit button",
        ),
        (
            E::TextareaHardWrapRequiresCols,
            "textarea with hard wrapping requires a cols attribute",
        ),
        (
            E::BlockingRequiresStylesheetOrExpectRel,
            "blocking attribute on a link requires the stylesheet or expect relationship",
        ),
        (
            E::IconSizesRequireIconRel,
            "sizes attribute on a link requires the icon or apple-touch-icon relationship",
        ),
        (
            E::SourceDimensionsRequirePictureParent,
            "width and height attributes on source require a picture parent",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(message(&error), expected);
    }
}

#[test]
fn navigable_target_and_custom_command_errors_follow_the_name_grammars() {
    assert_eq!(
        message(&HtmlNavigableTargetError::ReservedLeadingUnderscore),
        "navigable target name starts with an underscore"
    );
    assert_eq!(
        message(&HtmlNavigableTargetError::TabOrNewlineWithLessThan {
            tab_or_newline_code_unit_offset: 1,
            less_than_code_unit_offset: 4
        }),
        "navigable target name contains a tab or newline at UTF-16 offset 1 and a less-than sign at offset 4"
    );
    assert_eq!(
        message(&HtmlCustomCommandError::MissingPrefix),
        "custom command does not start with two hyphens"
    );
    assert_eq!(
        message(&HtmlCustomCommandError::BoundaryWhitespace),
        "custom command starts or ends with ASCII whitespace"
    );
}

#[test]
fn numeric_image_progress_link_and_area_conformance_errors_state_their_rule() {
    let common = HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(
        HtmlValueSyntaxError::InvalidNumericSyntax,
    ));
    assert_eq!(message(&common), "numeric attribute value is not valid");
    assert_eq!(
        cause(&common).as_deref(),
        Some("value recovery failed on source-defined syntax")
    );
    assert_eq!(
        message(&HtmlNumericAttributeConformanceError::ZeroMustBePositive),
        "numeric attribute value must be positive"
    );
    assert_eq!(
        message(&HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
        "numeric attribute value exceeds its maximum of 1000"
    );

    assert_eq!(
        message(&HtmlImageAttributeConformanceError::RequiredAttributeMissing),
        "required image attribute is missing"
    );
    assert_eq!(
        message(&HtmlImageAttributeConformanceError::EmptyAlt),
        "image alt attribute is empty"
    );
    assert_eq!(
        message(&HtmlImageAttributeConformanceError::EmptySource),
        "image src attribute is empty"
    );
    let dimension = HtmlImageAttributeConformanceError::CommonValue(
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow),
    );
    assert_eq!(
        message(&dimension),
        "image dimension attribute value is not valid"
    );
    assert!(dimension.source().is_some());

    let syntax = HtmlProgressNumberAuthorError::Syntax(HtmlValueSyntaxError::InvalidNumericSyntax);
    assert_eq!(
        message(&syntax),
        "progress attribute is not a valid floating-point number"
    );
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value does not match the selected numeric syntax")
    );
    assert_eq!(
        message(&HtmlProgressNumberAuthorError::NegativeValue),
        "progress value attribute is negative"
    );
    assert_eq!(
        message(&HtmlProgressNumberAuthorError::NonpositiveMaximum),
        "progress max attribute is not greater than zero"
    );

    assert_eq!(
        message(&HtmlLinkDestinationContextError::PreloadRequiresAs),
        "preload link requires an as attribute"
    );
    assert_eq!(
        message(&HtmlLinkDestinationContextError::PreloadRequiresPreloadDestination),
        "preload link as attribute must name a preload destination"
    );
    assert_eq!(
        message(&HtmlLinkDestinationContextError::ModulePreloadRequiresModuleDestination),
        "modulepreload link as attribute must name a script-like destination"
    );
    assert_eq!(
        message(&HtmlLinkDestinationContextError::AsRequiresPreloadOrModulePreloadRel),
        "as attribute requires the preload or modulepreload relationship"
    );

    use HtmlAreaCoordinatesAuthorError as Area;
    let area = [
        (
            Area::MissingRequired,
            "area shape requires a coords attribute",
        ),
        (
            Area::ForbiddenPresence,
            "default area shape forbids a coords attribute",
        ),
        (
            Area::InvalidListSyntax,
            "area coords is not a valid list of comma-separated integers",
        ),
        (
            Area::Cardinality,
            "area coords has the wrong number of values",
        ),
        (
            Area::NonintegerItem { index: 2 },
            "area coords item at index 2 is not an integer",
        ),
        (Area::NegativeRadius, "area circle radius is negative"),
        (
            Area::NonincreasingX,
            "area rectangle x coordinates must increase",
        ),
        (
            Area::NonincreasingY,
            "area rectangle y coordinates must increase",
        ),
    ];
    for (error, expected) in area {
        assert_eq!(message(&error), expected);
    }
}

#[test]
fn autocomplete_syntax_and_srcset_diagnostics_state_their_rule() {
    use HtmlAutocompleteSyntaxError as Syntax;
    assert_eq!(
        message(&Syntax::EmptyTokens),
        "autocomplete value has no tokens"
    );
    assert_eq!(
        message(&Syntax::MissingDetailField),
        "autocomplete value has no detail field"
    );
    assert_eq!(
        message(&Syntax::UnrecognizedDetailField { token_index: 2 }),
        "autocomplete token at index 2 is not a recognized detail field"
    );
    assert_eq!(
        message(&Syntax::ContactHintRequiresContactField {
            field: HtmlAutofillField::Name
        }),
        "autocomplete contact hint requires a contact field, but \"name\" is not one"
    );
    assert_eq!(
        message(&Syntax::UnexpectedToken { token_index: 3 }),
        "autocomplete token at index 3 is not expected at that position"
    );

    use HtmlSrcsetParseError as Parse;
    assert_eq!(
        message(&Parse::SkippedCommas),
        "srcset has a comma where an image candidate was expected"
    );
    assert_eq!(
        message(&Parse::MultipleTrailingCommas),
        "srcset image candidate URL is followed by more than one trailing comma"
    );
    assert_eq!(
        message(&Parse::FutureHeightDescriptor),
        "srcset image candidate uses the unsupported height descriptor"
    );
    assert_eq!(
        message(&Parse::RejectedCandidate),
        "srcset image candidate is rejected and ignored"
    );
    assert_eq!(
        message(&Parse::DensityConversionOverflow),
        "srcset density overflows the finite binary64 range"
    );

    use HtmlSrcsetContextViolation as Context;
    let context = [
        (
            Context::SrcOrSrcsetRequired,
            "img element requires a src or srcset attribute",
        ),
        (
            Context::SourceParentNotPicture,
            "source element with a srcset attribute requires a picture parent",
        ),
        (
            Context::SourceSrcsetRequired,
            "source element in a picture requires a srcset attribute",
        ),
        (
            Context::SizesRequired,
            "srcset with width descriptors requires a sizes attribute",
        ),
        (
            Context::WidthDescriptorsRequiredForSizes,
            "sizes attribute requires every srcset candidate to use a width descriptor",
        ),
        (
            Context::ImagesizesRequired,
            "imagesrcset with width descriptors requires an imagesizes attribute",
        ),
        (
            Context::PreloadRelRequired,
            "imagesrcset requires a preload link relationship",
        ),
        (
            Context::ImageAsStateRequired,
            "imagesrcset requires the image destination",
        ),
    ];
    for (error, expected) in context {
        assert_eq!(message(&error), expected);
    }
}

#[test]
fn input_refusals_state_what_was_refused() {
    let range = HtmlInputSelection::selection(5, 2, HtmlInputSelectionDirection::None)
        .expect_err("start after end");
    assert_eq!(
        message(&range),
        "input selection start 5 is after its end 2"
    );
    let filename = input_filename_setter(&[0x61]).expect_err("only the empty value is settable");
    assert_eq!(
        message(&filename),
        "a file input's value can only be set to the empty string"
    );
}

#[test]
fn construction_refusals_state_their_cause_and_answer_the_matching_predicate() {
    let mut builder = HtmlDocument::builder(construction_limits(), HtmlDocumentMode::NoQuirks)
        .expect("limits admit the document");
    let root = builder.root();
    let template = builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "template"),
            OrderedAttributes::default(),
        )
        .expect("template element");
    let contents_required = builder
        .append_text(&template, "wrong parent")
        .expect_err("template children belong to its contents");
    assert_eq!(
        message(&contents_required),
        "template children must be appended to its separate contents"
    );
    assert_eq!(
        contents_required.cause(),
        HtmlConstructionCause::TemplateContentsRequired
    );

    let other = HtmlFragment::builder(construction_limits()).expect("fragment builder");
    let foreign_root = other.root();
    let foreign = builder
        .append_text(&foreign_root, "foreign")
        .expect_err("handle belongs to another builder");
    assert_eq!(
        message(&foreign),
        "node handle belongs to a different builder"
    );
    assert_eq!(foreign.cause(), HtmlConstructionCause::ForeignBuilder);
}

#[test]
fn construction_limit_refusals_report_the_resource_limit_and_attempted_total() {
    let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 2, 2))
        .expect("limits admit the root");
    let root = builder.root();
    let refused = builder
        .append_text(&root, "too long")
        .expect_err("eight UTF-16 units exceed the limit of two");
    assert_eq!(
        refused.cause(),
        HtmlConstructionCause::LimitExceeded {
            resource: HtmlConstructionResource::Utf16Units,
            limit: 2,
            attempted: 8
        }
    );
    assert_eq!(
        message(&refused),
        "UTF-16 unit limit of 2 cannot admit a total of 8"
    );
    for (resource, text) in [
        (HtmlConstructionResource::Nodes, "node"),
        (HtmlConstructionResource::Edges, "edge"),
        (HtmlConstructionResource::Attributes, "attribute"),
        (HtmlConstructionResource::Depth, "depth"),
    ] {
        assert_eq!(resource.to_string(), text);
    }
}

#[test]
fn unfinished_interpretations_refuse_their_finish_and_hand_the_owner_back() {
    let rel = HtmlRelInterpretation::new(HtmlRelInputs::new(None));
    let refusal = rel.finish().expect_err("not interpreted");
    assert!(refusal.cause().is_not_ready());
    assert_eq!(
        message(refusal.cause()),
        "rel interpretation has an unfinished step"
    );
    assert_eq!(
        message(&refusal),
        "finish refused: rel interpretation has an unfinished step"
    );
    assert_eq!(
        cause(&refusal).as_deref(),
        Some("rel interpretation has an unfinished step")
    );
    let (rel, _) = refusal.into_parts();
    assert!(rel.abort().role().is_none());

    let element = KnownHtmlElement::Style.qualified_name();
    let media: HtmlMediaInterpretation<'_, QualifiedName, Attribute> =
        HtmlMediaInterpretation::from_matched(&element, None);
    let refusal = media.finish().expect_err("not assembled");
    assert!(refusal.cause().is_not_ready());
    assert_eq!(
        message(refusal.cause()),
        "media interpretation has an unfinished step"
    );

    let raw = DocumentString::from("a b");
    let microdata =
        HtmlMicrodataTokenInterpretation::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw));
    let refusal = microdata.finish().expect_err("tokens remain");
    assert!(refusal.cause().is_not_ready());
    assert_eq!(
        message(refusal.cause()),
        "microdata token interpretation has an unfinished step"
    );

    let time =
        HtmlTimeElementInterpretation::new(&raw, HtmlValueInterpreter::new(16)).expect("admitted");
    let refusal = time.finish().expect_err("not recognized");
    assert!(refusal.cause().is_not_ready());
    assert!(refusal.cause().processing_failure().is_none());
    assert_eq!(
        message(refusal.cause()),
        "time-element interpretation has an unfinished step"
    );
}

#[test]
fn csp_protocol_errors_state_the_misused_input() {
    let mut owner =
        HtmlCspOwner::new_with(&mut |_| Ok::<(), Infallible>(())).expect("owner admitted");
    let mut other =
        HtmlCspOwner::new_with(&mut |_| Ok::<(), Infallible>(())).expect("owner admitted");
    let association = owner
        .initial_document_with(&mut |_| Ok::<(), Infallible>(()))
        .expect("initial document");
    let content = DocumentString::from("default-src 'none'");
    let profile = HtmlCspProcessorAuthority::new_with(&mut |_| Ok::<(), Infallible>(()))
        .expect("authority")
        .profile_with(1, &mut |_| Ok::<(), Infallible>(()))
        .expect("profile");
    let refused = other
        .request_meta_insertion_with(&association, &profile, content, &mut |_| {
            Ok::<(), Infallible>(())
        })
        .expect_err("association belongs to another owner");
    assert_eq!(
        message(&refused),
        "CSP request rejected and its input retained"
    );
    assert_eq!(
        cause(&refused).as_deref(),
        Some("CSP protocol was violated")
    );
    assert!(matches!(refused.error, HtmlCspError::Protocol(ref error)
        if error.is_foreign_owner()
            && message(error) == "policy state belongs to a different CSP owner"));
}

#[test]
fn owned_causes_convert_into_their_wrapping_error() {
    let admission = refused_admission();
    assert_eq!(
        HtmlSrcsetInterpretationError::from(admission),
        HtmlSrcsetInterpretationError::RawAdmission(admission)
    );
    assert_eq!(
        HtmlAutocompleteMaterializationError::from(admission),
        HtmlAutocompleteMaterializationError::RawAdmission(admission)
    );
    assert_eq!(
        HtmlIntegerRecoveryError::from(HtmlNumericParseError::Negative),
        HtmlIntegerRecoveryError::Syntax(HtmlNumericParseError::Negative)
    );
    let allocation = HtmlIntegerUnitStepError::AllocationFailed { requested: 1 };
    assert_eq!(
        HtmlIntegerRecoveryError::from(allocation),
        HtmlIntegerRecoveryError::Allocation(allocation)
    );
    assert_eq!(
        HtmlAutocompleteSectionError::from(HtmlTokenError::EmptySpaceToken),
        HtmlAutocompleteSectionError::Token(HtmlTokenError::EmptySpaceToken)
    );
    let duplicate = duplicate_token();
    assert_eq!(
        HtmlTokenDeclarationError::from(duplicate.clone()),
        HtmlTokenDeclarationError::Duplicate(duplicate.clone())
    );
    assert_eq!(
        HtmlSandboxValueError::from(duplicate.clone()),
        HtmlSandboxValueError::Duplicate(duplicate.clone())
    );
    assert_eq!(
        HtmlMicrodataTokenListError::from(duplicate.clone()),
        HtmlMicrodataTokenListError::Duplicate(duplicate)
    );
}

#[test]
fn action_errors_distinguish_the_callers_refusal_from_protocol_misuse() {
    let admission = refused_admission();
    let protocol = HtmlRelInterpretation::new(HtmlRelInputs::new(None))
        .finish()
        .expect_err("not interpreted")
        .into_parts()
        .1;

    let refused: HtmlRelActionError<HtmlValueAdmissionError> =
        HtmlRelActionError::Admission(admission);
    assert_eq!(
        message(&refused),
        "rel interpretation was refused by the caller's admission"
    );
    assert_eq!(cause(&refused), Some(admission.to_string()));

    let misused: HtmlRelActionError<HtmlValueAdmissionError> = protocol.into();
    assert_eq!(
        message(&misused),
        "rel interpretation protocol was violated"
    );
    assert_eq!(
        cause(&misused).as_deref(),
        Some("rel interpretation has an unfinished step")
    );

    let refused: HtmlMediaActionError<HtmlValueAdmissionError> =
        HtmlMediaActionError::Admission(admission);
    assert_eq!(
        message(&refused),
        "media interpretation was refused by the caller's admission"
    );
    let refused: HtmlMicrodataTokenActionError<HtmlValueAdmissionError> =
        HtmlMicrodataTokenActionError::Admission(admission);
    assert_eq!(
        message(&refused),
        "microdata token interpretation was refused by the caller's admission"
    );
}

#[test]
fn natural_arithmetic_and_value_pool_errors_state_the_misuse() {
    use surgeist_html::bounded::{
        numeric::HtmlNaturalArithmeticError as Natural, values::HtmlValuePoolError,
    };
    assert_eq!(
        message(&Natural::NonCanonical),
        "natural number digits are empty or have a leading zero"
    );
    assert_eq!(
        message(&Natural::InvalidDigit {
            index: 2,
            value: 12
        }),
        "natural number digit 12 at index 2 exceeds 9"
    );
    assert_eq!(
        message(&Natural::ScratchNotEmpty),
        "arithmetic scratch storage is not empty"
    );
    assert_eq!(
        message(&Natural::ScratchCapacity {
            required: 9,
            capacity: 4
        }),
        "arithmetic scratch storage holds 4 digits but 9 are required"
    );
    assert_eq!(
        message(&Natural::RepresentationExhausted),
        "arithmetic size cannot be represented on this platform"
    );
    assert_eq!(
        message(&Natural::NotReady),
        "arithmetic operation has an unfinished step"
    );
    assert_eq!(
        message(&Natural::EmptySlot),
        "arithmetic operation was already consumed"
    );
    assert_eq!(
        message(&HtmlValuePoolError::ForeignOwner),
        "value pool action belongs to a different pool owner"
    );
    assert_eq!(
        message(&HtmlValuePoolError::WrongAction),
        "action does not match the pending value pool operation"
    );
    assert_eq!(
        message(&HtmlValuePoolError::NotReady),
        "value pool operation has an unfinished step"
    );
    assert_eq!(
        message(&HtmlValuePoolError::RepresentationExhausted),
        "value pool accounting size cannot be represented on this platform"
    );
}

#[test]
fn duplicate_token_and_redundant_sandbox_errors_state_the_conflict_in_prose() {
    let duplicate = duplicate_token();
    assert_eq!(
        message(&duplicate),
        "token at index 1 duplicates the token at index 0 under ASCII case-insensitive comparison"
    );
    let case_sensitive = HtmlOrderedTokenSet::new(
        HtmlTokenComparison::CaseSensitive,
        ["a", "a"].map(|token| HtmlSpaceToken::new(token).expect("valid token")),
    )
    .expect_err("equal spellings collide");
    assert_eq!(
        message(&case_sensitive),
        "token at index 1 duplicates the token at index 0 under case-sensitive comparison"
    );
    let redundant =
        HtmlSandboxValueError::Redundant(HtmlSandboxRedundancy::TopNavigationAndByUserActivation);
    assert_eq!(
        message(&redundant),
        "sandbox lists both allow-top-navigation and allow-top-navigation-by-user-activation, which is redundant"
    );
}

#[test]
fn refused_pool_seal_returns_the_open_root_with_the_cause() {
    use surgeist_html::bounded::values::{HtmlStringPool, HtmlValuePoolError};
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().expect("fresh pool root");
    let _unresolved = pool.mark(&root).expect("nested mark");
    let refusal = match pool.prepare_seal(root) {
        Err(refusal) => refusal,
        Ok(_) => panic!("a root with an unresolved mark was sealed"),
    };
    assert_eq!(refusal.error(), HtmlValuePoolError::NotReady);
    assert_eq!(
        message(&refusal),
        "value pool seal refused: value pool operation has an unfinished step"
    );
    let (error, root) = refusal.into_parts();
    assert_eq!(error, HtmlValuePoolError::NotReady);
    pool.abort_root(root)
        .expect("the returned root is still open");
}

#[test]
fn wrappers_describe_their_own_operation_and_expose_the_cause_once() {
    use surgeist_html::attributes::{
        control_length::HtmlControlLengthError, input_bounds::HtmlInputNumericConstraintsError,
    };
    let admission = refused_admission();

    let duplicate = HtmlSandboxValueError::Duplicate(duplicate_token());
    assert_eq!(
        message(&duplicate),
        "sandbox value contains a duplicate keyword"
    );
    assert_eq!(cause(&duplicate), Some(duplicate_token().to_string()));

    let control = HtmlControlLengthError::Admission(admission);
    assert_eq!(
        message(&control),
        "control value was refused by raw-value admission"
    );
    assert_eq!(cause(&control), Some(admission.to_string()));
    assert_eq!(
        message(&HtmlControlLengthError::InputCodeUnitsOverflow),
        "control input UTF-16 length cannot be represented"
    );

    let constraints = HtmlInputNumericConstraintsError::Admission(admission);
    assert_eq!(
        message(&constraints),
        "input numeric constraint value was refused by raw-value admission"
    );
    assert_eq!(cause(&constraints), Some(admission.to_string()));
    assert_eq!(
        message(&HtmlInputNumericConstraintsError::InputCodeUnitsOverflow),
        "input numeric UTF-16 length cannot be represented"
    );

    let protocol: HtmlCspError<HtmlValueAdmissionError> = {
        let mut other = HtmlCspOwner::new_with(&mut |_| Ok::<(), Infallible>(())).unwrap();
        let mut owner = HtmlCspOwner::new_with(&mut |_| Ok::<(), Infallible>(())).unwrap();
        let association = owner
            .initial_document_with(&mut |_| Ok::<(), Infallible>(()))
            .unwrap();
        let profile = HtmlCspProcessorAuthority::new_with(&mut |_| Ok::<(), Infallible>(()))
            .unwrap()
            .profile_with(1, &mut |_| Ok::<(), Infallible>(()))
            .unwrap();
        let rejected = other
            .request_meta_insertion_with(
                &association,
                &profile,
                DocumentString::from("default-src 'none'"),
                &mut |_| Ok::<(), Infallible>(()),
            )
            .unwrap_err();
        match rejected.error {
            HtmlCspError::Protocol(error) => HtmlCspError::Protocol(error),
            HtmlCspError::Interrupted(never) => match never {},
        }
    };
    assert_eq!(message(&protocol), "CSP protocol was violated");
    assert_eq!(
        cause(&protocol).as_deref(),
        Some("policy state belongs to a different CSP owner")
    );
    let interrupted: HtmlCspError<HtmlValueAdmissionError> = HtmlCspError::Interrupted(admission);
    assert_eq!(
        message(&interrupted),
        "CSP processing was interrupted by the caller's admission"
    );
    assert_eq!(cause(&interrupted), Some(admission.to_string()));
}

#[test]
fn accept_specifier_converts_a_token_failure_with_from() {
    let token = HtmlTokenError::CommaTokenComma {
        code_unit_offset: 0,
    };
    assert_eq!(
        HtmlAcceptSpecifierError::from(token),
        HtmlAcceptSpecifierError::Token(token)
    );
}
