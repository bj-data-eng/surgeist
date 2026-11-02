//! Public error values state what failed in prose and chain their causes.
//!
//! Each expected message is the documented meaning of the variant: the rule of
//! the pinned HTML Standard the value breaks, or the resource limit it names.
//! A wrapper states its own operation and exposes the wrapped failure through
//! `Error::source`; its text does not repeat the cause.

use std::error::Error;

use surgeist_html::bounded::{
    AutocompleteMaterializationError, FloatingPointAllocationError, IntegerRecoveryError,
    IntegerUnitStepError, NumericMaterializationError, NumericParseError, RangeNumericError,
};
use surgeist_html::{
    DocumentString, HtmlAcceptSpecifierError, HtmlAutocompleteSectionError,
    HtmlCustomElementNameError, HtmlDecimalDigitsError, HtmlDuplicateTokenError, HtmlIconSizeError,
    HtmlInputImageAttributeError, HtmlMeterAuthorRelation, HtmlMeterNativeError,
    HtmlMeterNumberRole, HtmlMicrodataTokenListError, HtmlNonNegativeInteger,
    HtmlNonNegativeRational, HtmlOrderedTokenSet, HtmlProcessingInstructionError,
    HtmlRationalError, HtmlSpaceToken, HtmlSrcsetAuthorViolation, HtmlSrcsetDensityError,
    HtmlSrcsetInterpretationError, HtmlSrcsetUrlTokenError, HtmlTokenComparison,
    HtmlTokenDeclarationError, HtmlTokenError, HtmlValueAdmissionError, HtmlValueInterpreter,
    HtmlValueSyntaxError,
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
        message(&NumericParseError::InvalidSyntax),
        "value does not match the HTML numeric syntax"
    );
    assert_eq!(
        message(&NumericParseError::Negative),
        "value is negative where a nonnegative value is required"
    );
    assert_eq!(
        message(&NumericParseError::FloatingPointOverflow),
        "value overflows the finite binary64 range"
    );
    assert_eq!(
        message(&NumericParseError::ZeroDimension),
        "dimension is zero where a nonzero value is required"
    );
}

#[test]
fn integer_digit_boundaries_report_their_exact_numbers() {
    assert_eq!(
        message(&IntegerUnitStepError::RepresentationExhausted),
        "integer digit count cannot be represented on this platform"
    );
    assert_eq!(
        message(&IntegerUnitStepError::DigitLimitExceeded {
            limit: 4,
            required: 9
        }),
        "integer needs 9 digits, exceeding the limit of 4"
    );
    assert_eq!(
        message(&IntegerUnitStepError::RequestedCapacityLimitExceeded {
            limit: 16,
            required: 40
        }),
        "integer needs 40 bytes of digit storage, exceeding the limit of 16"
    );
    assert_eq!(
        message(&IntegerUnitStepError::ActualCapacityLimitExceeded {
            limit: 16,
            requested: 10,
            capacity_bytes: 32
        }),
        "allocating 10 bytes of digit storage produced 32 bytes, exceeding the limit of 16"
    );
    assert_eq!(
        message(&IntegerUnitStepError::AllocationFailed { requested: 7 }),
        "allocating 7 bytes of digit storage failed"
    );
}

#[test]
fn floating_point_allocation_boundaries_report_their_exact_numbers() {
    assert_eq!(
        message(&FloatingPointAllocationError::RepresentationExhausted),
        "floating-point token length cannot be represented on this platform"
    );
    assert_eq!(
        message(
            &FloatingPointAllocationError::RequestedCapacityLimitExceeded {
                limit: 8,
                required: 12
            }
        ),
        "floating-point token needs 12 bytes of storage, exceeding the limit of 8"
    );
    assert_eq!(
        message(&FloatingPointAllocationError::ActualCapacityLimitExceeded {
            limit: 8,
            requested: 6,
            capacity_bytes: 24
        }),
        "allocating 6 bytes of floating-point token storage produced 24 bytes, exceeding the limit of 8"
    );
    assert_eq!(
        message(&FloatingPointAllocationError::AllocationFailed { requested: 5 }),
        "allocating 5 bytes of floating-point token storage failed"
    );
}

#[test]
fn numeric_wrappers_chain_the_wrapped_failure() {
    let syntax = IntegerRecoveryError::Syntax(NumericParseError::Negative);
    assert_eq!(message(&syntax), "integer recovery rejected the input");
    assert_eq!(
        cause(&syntax).as_deref(),
        Some("value is negative where a nonnegative value is required")
    );
    let allocation =
        IntegerRecoveryError::Allocation(IntegerUnitStepError::AllocationFailed { requested: 3 });
    assert_eq!(
        message(&allocation),
        "integer recovery could not store its digits"
    );
    assert_eq!(
        cause(&allocation).as_deref(),
        Some("allocating 3 bytes of digit storage failed")
    );

    let digits = NumericMaterializationError::Digits(HtmlDecimalDigitsError::Empty);
    assert_eq!(
        message(&digits),
        "numeric digits are not a valid decimal magnitude"
    );
    assert_eq!(
        cause(&digits).as_deref(),
        Some("HTML integer magnitude has no decimal digits")
    );
    let boundary =
        NumericMaterializationError::Boundary(IntegerUnitStepError::RepresentationExhausted);
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
        message(&RangeNumericError::Integrity),
        "range numeric reply does not match its pending request"
    );
    assert_eq!(
        message(&RangeNumericError::Representation),
        "range numeric length cannot be represented on this platform"
    );
    assert_eq!(
        message(&RangeNumericError::Overflow),
        "range numeric value exceeds the representable range"
    );
    assert_eq!(
        message(&RangeNumericError::InvalidSyntax),
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
        message(&AutocompleteMaterializationError::BytesLimit {
            limit: 10,
            required: 20
        }),
        "autocomplete materialization needs 20 allocated bytes, exceeding the limit of 10"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::OutputLimit {
            limit: 1,
            required: 2
        }),
        "autocomplete materialization needs 2 nested output records, exceeding the limit of 1"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::ScratchLimit {
            limit: 3,
            required: 5
        }),
        "autocomplete materialization needs 5 scratch records, exceeding the limit of 3"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::Allocation {
            requested_bytes: 64
        }),
        "allocating 64 bytes for autocomplete materialization failed"
    );
    assert_eq!(
        message(&AutocompleteMaterializationError::Representation),
        "autocomplete materialization size cannot be represented on this platform"
    );
    let admission = refused_admission();
    let refused = AutocompleteMaterializationError::RawAdmission(admission);
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
