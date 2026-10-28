# Syntax 3 conformance coverage

This page maps the clauses of the pinned CSS Syntax Module Level 3 edition to
the tests of this crate. It records what the tests establish and what they do
not, so a reader can tell a covered clause from an unexamined one.

**Only the rows marked "Claimed" or "Claimed in part" are claimed.** Every
other row has the status "Not claimed": its clause has no implementation or no
test in this crate yet. A row changes only in the commit that adds the
implementation and the tests that exercise its clause, and that commit cites the
tests by name. Browser conformance is not claimed anywhere on this page.

## Edition and evidence classes

The [CSS Syntax 3 edition](../../../references/SOURCE-EDITIONS.md#css-syntax-3-edition)
selected for Surgeist is the 1 October 2026 Candidate Recommendation Draft. Its
[local copy][syntax] is the only source of expected behavior for these rows.
The conventions of other libraries are not an authority for a clause.

Each covered row will name its evidence by class:

- **Specification-derived**: a literal input and expected output written from
  the clause text or one of its examples.
- **Differential**: a comparison with another implementation's output on the
  same input. It detects disagreement; it never establishes a clause, and a
  disagreement is resolved from the specification text.
- **Property**: an invariant checked over generated inputs.
- **External vectors**: a retained corpus of third-party conformance tests,
  with its provenance recorded with the corpus.

## Clause map

Tests named without a path are in `tests/lexer.rs`; `properties.rs`,
`protocol.rs`, `token.rs` and `cssparser_oracle.rs` are in `tests/` too, and
tests in `src/` are named by their file.

| Clause | Subject | Status | Tests |
| --- | --- | --- | --- |
| [3.2][input-byte-stream] | The input byte stream: decoding and fallback encoding | Not claimed | None |
| [3.3][input-preprocessing] | Preprocessing the input stream: the filter code points algorithm, as the stepwise `InputCursor` applies it and as the `Lexer` applies it while reading | Claimed in part. Specification-derived: over UTF-8 bytes and UTF-16 units, a CR, an FF and a CR LF pair each become one LF whose interval covers every original unit it replaced, a NUL and a lone surrogate become U+FFFD, and consecutive and terminal CRs are not lost; in the lexer a newline so made is whitespace and ends a string, a backslash before it starts no escape, and positions stay original bytes. Property: the lexer, the stepwise tokenizer over bytes and the one over UTF-16 units agree on generated and raw text, including NUL, CR and FF. Not claimed: a whole-string filter function (none exists yet) and the WPT `input-preprocessing` vectors, which are not adapted | `original_utf8_intervals_and_css_preprocessing`, `utf16_pairs_and_lone_surrogates_preserve_units`, `consecutive_cr_and_terminal_cr_are_not_lost`, `a_carriage_return_line_feed_pair_is_one_whitespace_covering_both_original_units`, `a_nul_becomes_a_replacement_character_that_is_an_ident_start`, `utf8_scalars_report_the_original_byte_interval`, `preprocessing_turns_carriage_returns_and_form_feeds_into_line_feeds_and_keeps_original_bytes`, `preprocessing_turns_null_into_the_replacement_character_which_is_an_ident_code_point`, `a_scalar_that_preprocessing_would_have_replaced_is_refused_and_changes_nothing`, and `generated_text_is_lexed_totally_consistently_across_input_kinds_and_round_trips` and `every_short_text_is_lexed_as_the_stepwise_tokenizer_reads_it` (`tests/properties.rs`) |
| [4][tokenization] | The tokens and their values: the token list, the type flag of a hash, the sign and type flag of a number, and the range of a unicode-range token | Claimed in part (specification-derived, `tests/token.rs`): each `Token` variant reports its kind, a number keeps its exact spelling, sign and type flag, converts to the value its spelling states and to an integer only for the integer flag when it fits, and a unicode-range token's end before its start is an empty range. A number is built only by the lexer, so these tests take their numbers from it. `Comment` is the one documented extension (section 9 allows an implementation to preserve comments). Not claimed: the railroad diagrams of 4.1, which are non-normative | `every_token_reports_its_kind_and_only_whitespace_and_comments_are_trivia`, `a_number_keeps_its_exact_spelling_sign_and_type_flag`, `a_number_converts_to_the_value_its_spelling_states`, `an_integer_is_available_only_for_the_integer_flag_and_when_it_fits`, `a_unicode_range_holds_its_bounds_and_an_end_before_the_start_is_empty`, `options_are_a_value_with_a_default_that_allows_nothing_and_rebases_nothing` |
| [4.1][token-diagrams] | Token railroad diagrams | Not claimed (non-normative) | None |
| [4.2][tokenizer-definitions] | Definitions: the code-point classes digit, hex digit, uppercase letter, lowercase letter, letter, ident-start code point, ident code point, non-printable code point, newline and whitespace | Claimed in part (specification-derived, `tests/lexical.rs`) | See [4.2 code-point class tests](#42-code-point-class-tests) |
| [4.2][tokenizer-definitions] | Definitions: the non-ASCII ident code point | Not claimed for the pinned edition. The function implements the 2021 definition (every code point from U+0080), which the pinned edition replaces with a list of ranges; the tests pin the 2021 behavior, not the pinned one, and the cssparser oracle agrees with it for the same reason | `non_ascii_ident_code_point_is_every_code_point_from_u0080_under_the_2021_rule`, `the_non_ascii_ident_code_point_is_the_2021_rule_until_the_pinned_edition_is_adopted` |
| [4.2][tokenizer-definitions] | Definitions: the stream terms (next input code point, current input code point, reconsume, EOF code point) and the maximum allowed code point | Claimed in part: EOF is the end of the text (an escape, string, url and comment each end there as their clause says) and reconsuming is not observable in the lexer's output. Not claimed otherwise | `the_end_of_the_text_is_the_eof_token_and_produces_no_lexeme` and the EOF cases of 4.3.2 to 4.3.7 below |
| [4.3.1][consume-token] | Consume a token: every branch of the first code point (whitespace, string, number sign, apostrophe, parentheses, plus, comma, hyphen-minus, full stop, colon, semicolon, less-than, commercial at, brackets, reverse solidus, braces, digit, `U`/`u`, ident-start, EOF, anything else) | Claimed (specification-derived). Not claimed: the non-ASCII ident code point (2021 rule, as for 4.2) | `whitespace_is_one_token_for_the_longest_run`, `single_code_point_tokens_are_their_own_tokens`, `any_other_code_point_is_a_delim_token_with_that_code_point`, `a_number_sign_starts_a_hash_only_before_an_ident_code_point_or_a_valid_escape`, `a_commercial_at_starts_an_at_keyword_only_before_an_ident_sequence`, `a_less_than_sign_is_a_cdo_only_before_exclamation_mark_and_two_hyphens`, `a_hyphen_minus_is_tried_as_a_number_then_a_cdc_then_an_ident_then_a_delim`, `a_hyphen_minus_before_a_backslash_and_a_newline_starts_no_ident_sequence`, `a_plus_sign_and_a_full_stop_start_a_number_only_when_one_would_start`, `a_reverse_solidus_starts_an_ident_like_token_only_with_a_valid_escape`, `an_ident_start_code_point_starts_an_ident_like_token`, `a_unicode_range_is_produced_only_when_allowed_and_the_input_starts_one`, `the_end_of_the_text_is_the_eof_token_and_produces_no_lexeme` |
| [4.3.2][consume-comment] | Consume comments, as the `Comment` extension token | Claimed (specification-derived): a comment runs to the first `*/`, the asterisk of the opening cannot close it, comments are not recognized inside a string or url, and one that reaches EOF is a parse error | `a_comment_runs_to_the_first_asterisk_solidus`, `a_comment_is_recognized_between_tokens_and_inside_none`, `a_comment_that_reaches_eof_is_unterminated_and_a_parse_error`, `comments_strings_bad_strings_and_cdo_cdc_keep_boundaries` (`src/lexer/protocol/tokenizer.rs`) |
| [4.3.3][consume-numeric-token] and [4.3.13][consume-number] | Numeric tokens and the spelling of a number | Claimed (specification-derived): the sign, digits, fraction and exponent, the cases where a part does not apply (`1.`, `1e`, `1e+`, `0.0.2`), dimension before percentage before number, the type flag, and the exact spelling. The value is `Number::to_f64`, which parses the spelling with the standard library's correctly rounded conversion; the tests check it on hand-worked values only | `a_number_is_a_sign_digits_a_fraction_and_an_exponent`, `a_number_ends_where_no_further_part_applies`, `a_numeric_token_is_a_dimension_before_an_ident_sequence_and_a_percentage_before_a_percent_sign`, `the_number_representation_excludes_the_unit_and_percent_sign_and_keeps_the_exact_spelling`, `exact_number_boundary_preserves_dimension_and_incomplete_exponent` (`src/lexer/protocol/number.rs`), `numeric_digits_are_flat_checked_and_not_converted_by_the_lexer` (`src/lexer/protocol/tokenizer.rs`) |
| [4.3.4][consume-ident-like-token] | Ident-like tokens | Claimed (specification-derived): ident, function, `url(` selecting an unquoted url exactly for an ASCII case-insensitive `url` (also when written with escapes), a quote or whitespace and a quote after `url(` making a function token and a separate string. The whitespace token between `url(` and a quote carries the whole consumed run as its interval; the clause is silent on intervals | `an_ident_followed_by_a_left_parenthesis_is_a_function_token_that_includes_it`, `url_followed_by_a_parenthesis_and_no_quote_is_a_url_token`, `url_followed_by_a_quote_is_a_function_token_and_the_string_is_separate`, `only_url_exactly_selects_url_consumption`, `url_name_prefixes_at_keywords_and_dimensions_do_not_select_url_consumption` (`src/lexer/protocol/tokenizer.rs`), `quoted_url_keeps_function_opening_and_actual_arbitrary_whitespace_separate` (same file) |
| [4.3.5][consume-string-token] | String tokens and bad-string tokens | Claimed (specification-derived): the ending quote, EOF, a newline reconsumed into a bad string, a backslash before EOF, before a newline, and before an escape | `a_string_runs_to_its_own_quote`, `a_string_that_reaches_eof_is_unterminated_and_a_parse_error`, `a_newline_in_a_string_makes_a_bad_string_and_is_reconsumed`, `a_backslash_in_a_string_continues_a_line_ends_at_eof_or_escapes` |
| [4.3.6][consume-url-token] and [4.3.15][consume-remnants-of-bad-url] | Url tokens, bad-url tokens and the remnants of a bad url | Claimed (specification-derived): whitespace trimming, the closing parenthesis, EOF, a quote, a left parenthesis, a non-printable code point and an invalid escape making a bad url, whitespace followed by anything else making a bad url, and the remnants stopping at an unescaped right parenthesis or EOF while consuming escaped ones | `an_unquoted_url_is_whitespace_trimmed_text_up_to_a_right_parenthesis`, `a_url_that_reaches_eof_is_unterminated_and_a_parse_error`, `a_quote_a_parenthesis_a_non_printable_or_an_invalid_escape_makes_a_bad_url`, `whitespace_followed_by_anything_but_a_right_parenthesis_is_a_bad_url_without_a_parse_error`, `the_remnants_of_a_bad_url_stop_at_an_unescaped_right_parenthesis_only`, `the_remnants_of_a_bad_url_examine_the_first_code_point_after_the_whitespace`, `unquoted_urls_decode_escaped_names_values_and_retain_exact_source`, `url_eof_is_distinct_from_closure_and_escape_eof_emits_replacement`, `bad_url_recovery_consumes_escaped_closing_parentheses_and_retains_eof`, `escaped_function_names_and_hashes_have_exact_source_intervals` (the last four in `src/lexer/protocol/tokenizer.rs`) |
| [4.3.7][consume-escaped-code-point] and [4.3.12][consume-name] | Escaped code points and ident sequences | Claimed (specification-derived): one to six hex digits and one optional whitespace, zero, a surrogate and a value above the maximum code point becoming U+FFFD, an escape at EOF becoming U+FFFD, an escaped non-hex code point standing for itself, a backslash-newline ending the name, and the worked example `\26 B` and `\000026B` for "&B" | `an_escaped_code_point_is_one_to_six_hex_digits_and_one_optional_whitespace`, `an_ident_sequence_consumes_ident_code_points_and_valid_escapes`, `a_backslash_at_eof_is_a_valid_escape_for_the_ident_that_stands_for_u_fffd`, `hexadecimal_escapes_retain_optional_terminator_and_six_digit_limit`, `invalid_scalar_escapes_and_eof_follow_css_syntax` (the last two in `src/lexer/protocol/name.rs`) |
| [4.3.8][valid-escape] | Check if two code points are a valid escape | Claimed (specification-derived, `tests/lexical.rs`) | `escape_is_valid_when_a_backslash_is_followed_by_a_non_newline`, `escape_is_invalid_when_the_backslash_is_followed_by_a_newline`, `escape_is_valid_when_the_backslash_is_followed_by_eof`, `escape_is_invalid_when_the_first_code_point_is_not_a_backslash`, `escape_treats_carriage_return_and_form_feed_as_ordinary_second_code_points` |
| [4.3.9][would-start-ident] | Check if three code points would start an ident sequence | Claimed in part: for non-ASCII input the 2021 non-ASCII ident code point rule above differs from the pinned clause, so only ASCII input is claimed (specification-derived, `tests/lexical.rs`) | `ident_starts_after_hyphen_minus_when_the_second_is_an_ident_start`, `ident_starts_after_hyphen_minus_when_the_second_is_another_hyphen_minus`, `ident_starts_after_hyphen_minus_when_the_second_and_third_are_a_valid_escape`, `ident_does_not_start_after_hyphen_minus_with_a_backslash_and_a_newline`, `ident_does_not_start_after_hyphen_minus_followed_by_anything_else`, `ident_starts_with_an_ident_start_code_point_whatever_follows`, `ident_starts_with_a_backslash_when_the_first_two_are_a_valid_escape`, `ident_does_not_start_with_a_backslash_before_a_newline`, `ident_does_not_start_with_anything_else` |
| [4.3.10][would-start-number] | Check if three code points would start a number | Claimed (specification-derived, `tests/lexical.rs`) | `number_starts_with_a_sign_followed_by_a_digit`, `number_starts_with_a_sign_a_full_stop_and_a_digit`, `number_does_not_start_with_a_sign_a_full_stop_and_a_non_digit`, `number_does_not_start_with_a_sign_followed_by_anything_else`, `number_starts_with_a_full_stop_followed_by_a_digit`, `number_does_not_start_with_a_full_stop_followed_by_a_non_digit`, `number_starts_with_a_digit_whatever_follows`, `number_does_not_start_with_anything_else` |
| [4.3.11][would-start-unicode-range] | Check if three code points would start a unicode-range | Claimed (specification-derived, `tests/lexical.rs`) | `unicode_range_starts_with_u_plus_and_a_question_mark_or_hex_digit`, `unicode_range_does_not_start_when_the_first_code_point_is_not_u`, `unicode_range_does_not_start_when_the_second_code_point_is_not_plus_sign`, `unicode_range_does_not_start_when_the_third_code_point_is_neither_question_mark_nor_hex` |
| [4.3.14][consume-unicode-range-token] | Consume a unicode-range token, through the lexer's `unicode_ranges` option | Claimed (specification-derived): up to six hex digits, question marks filling to six and ending the token, an optional end after a hyphen and a hex digit, an end before the start, bounds above the maximum code point, and the same text without the option reading as an ident and a number | `a_unicode_range_is_produced_only_when_allowed_and_the_input_starts_one`, `a_unicode_range_is_up_to_six_hex_digits_question_marks_and_an_optional_end`, `the_origin_rebases_every_offset_and_error_range` (a sub-range re-lexed with the option reports its place in the whole text). Not claimed: unicode ranges through the stepwise tokenizer, which does not offer them |
| [4.3][tokenizer-algorithms] | The parse errors of the tokenizer algorithms, as `LexicalError` events | Claimed (specification-derived). One event for each "this is a parse error" sentence that no token records: a comment, string or url that reaches EOF, a newline in a string, a quote, left parenthesis, non-printable code point or invalid escape in a url, a backslash before a newline outside a string, and an escape whose backslash ends the text. Not events, because the clause words them without a parse error: whitespace followed by anything but a right parenthesis in a url (4.3.6), EOF while consuming the remnants of a bad url (4.3.15), and a backslash before EOF in a string (4.3.5) | `a_comment_that_reaches_eof_is_unterminated_and_a_parse_error`, `a_string_that_reaches_eof_is_unterminated_and_a_parse_error`, `a_newline_in_a_string_makes_a_bad_string_and_is_reconsumed`, `a_url_that_reaches_eof_is_unterminated_and_a_parse_error`, `a_quote_a_parenthesis_a_non_printable_or_an_invalid_escape_makes_a_bad_url`, `whitespace_followed_by_anything_but_a_right_parenthesis_is_a_bad_url_without_a_parse_error`, `the_remnants_of_a_bad_url_stop_at_an_unescaped_right_parenthesis_only`, `a_backslash_at_eof_is_a_valid_escape_for_the_ident_that_stands_for_u_fffd`, `a_backslash_before_a_newline_outside_a_string_is_a_delim_and_a_parse_error`, `a_backslash_in_a_string_continues_a_line_ends_at_eof_or_escapes` |
| [4.3][tokenizer-algorithms] | One state machine, two drivers | Claimed (property and specification-derived): the free-running `Lexer` and the stepwise `TokenCursor` give the same tokens, decoded texts and extents for every text of up to three symbols over an alphabet with a code point of each class of 4.3.1, of four over a subset, and of up to five over multi-code-point symbols (`url(`, an escape and its terminator, an escaped parenthesis, a comment opener, `U+`), and for generated and raw text; a host that refuses an append once, asks again and answers wrongly in between gets the same output; the lexical errors the stepwise flags show are the lexer's; the lexemes tile the text and the lexer never panics | `every_short_text_is_lexed_as_the_stepwise_tokenizer_reads_it`, `generated_text_is_lexed_totally_consistently_across_input_kinds_and_round_trips` (`tests/properties.rs`), `the_properties_detect_a_wrong_lexer_result`, `the_lexemes_of_any_text_tile_it` |
| [9][serialization] | The round-trip requirement, as it constrains the tokenizer | Claimed in part (property): serializing the lexer's tokens with a serializer written in `tests/properties.rs` from section 9's table and the escaping of CSSOM 2.1, and lexing the result, gives the same tokens modulo whitespace and comments, over generated and raw text with no bad string or bad url. The crate has no serializer yet, so serialization itself is not claimed | `generated_text_is_lexed_totally_consistently_across_input_kinds_and_round_trips` (`tests/properties.rs`) |
| Differential | The `Lexer` against cssparser 0.37.0 as a compiled black box, on the spec-derived inputs and on 4,000 generated inputs: token kinds, spellings, extents and block structure | Compatibility evidence only; it establishes no clause. Every difference on the spec-derived inputs is a row of `tests/oracle/differences.tsv` and cites the sections of the pinned edition that make the lexer's output correct; a difference on a generated input is checked by class, so it must belong to one of the four classes below, each demonstrated by literal rows, and the generator must reach every class. The hand-written cases for the classes are in `tests/lexer.rs`: V-1 (4.3.6, 4.3.15: a backslash after the whitespace in a url), V-2 (4.3.1: `~=`, `\|=`, `^=`, `$=`, `*=` are two delim tokens), V-3 (4.3.1, 4.3.3, 4.3.9: a hyphen-minus before a backslash and a newline starts no ident), V-4 (4.3.1, 4.3.9: the hash type flag in the same case). cssparser reports no lexical errors, so none are compared | `every_difference_from_cssparser_is_listed_and_cites_the_pinned_edition`, `every_difference_on_a_generated_input_belongs_to_a_listed_class`, `the_comparison_itself_sees_a_difference_in_kind_range_and_structure` (`tests/cssparser_oracle.rs`) |
| [5.2][css-tree] and [5.3][parser-definitions] | Parsing results and token streams | Not claimed | None |
| [5.4][parser-entry-points] | Parser entry points | Not claimed | None |
| [5.5][parser-algorithms] | Parser algorithms | Not claimed | None |
| [6][anb-microsyntax] | The `An+B` microsyntax | Not claimed | None |

### 4.2 code-point class tests

All in `tests/lexical.rs`. Each counts the members of its class over every
Unicode scalar value and compares the count with the number the clause's ranges
imply, so a class with an extra member fails as surely as one missing a member.

- Digit: `digit_is_exactly_u0030_to_u0039_inclusive`,
  `digit_excludes_non_ascii_decimal_digits`.
- Hex digit: `hex_digit_is_digit_or_a_to_f_in_either_case`.
- Uppercase letter, lowercase letter, letter:
  `uppercase_letter_is_exactly_u0041_to_u005a_inclusive`,
  `lowercase_letter_is_exactly_u0061_to_u007a_inclusive`,
  `letter_is_an_uppercase_or_a_lowercase_letter_and_nothing_else`.
- Ident-start code point and ident code point (with the 2021 non-ASCII rule):
  `ident_start_is_a_letter_a_non_ascii_ident_code_point_or_low_line`,
  `ident_code_point_is_an_ident_start_a_digit_or_hyphen_minus`.
- Non-printable code point:
  `non_printable_is_u0000_to_u0008_u000b_u000e_to_u001f_and_u007f`,
  `non_printable_excludes_the_control_code_points_that_are_whitespace_or_form_feed`.
- Newline and whitespace:
  `newline_is_only_u000a_because_preprocessing_converts_cr_and_ff`,
  `whitespace_is_newline_tab_and_space`.

### Readings the tests record

- Section 4.3.8 returns true when the second code point is EOF, because EOF is
  not a newline and the section returns false only for a newline. Section
  4.3.7's EOF branch ("consume an escaped code point") exists for that case.
  The "Changes from the 20 February 2014 Candidate Recommendation" entry that
  calls a final backslash "not a valid escape" contradicts the normative text;
  the normative text governs.
- The checks and classes describe code points after input preprocessing (3.3),
  as the specification's "newline" definition says: only U+000A is a newline,
  and U+000D and U+000C are ordinary code points to these functions.
- Section 4.3.6 words the bad url that follows whitespace without "this is a
  parse error", and section 4.3.15 words EOF without one, so the lexer reports
  no event for them even though the token is a `<bad-url-token>`.
- Section 4.3.4 leaves the interval of the whitespace between `url(` and a quote
  unstated; the lexer reports one whitespace token covering the whole run the
  clause consumes, and the function token ends at the parenthesis.
- Section 9 says the tokenizer produces no comment token. The lexer reports each
  comment as a `Comment` token, which a consumer ignores like whitespace; this is
  the one extension, and consecutive comments are consecutive tokens.
- The WPT tests the edition cites for these clauses (`inclusive-ranges.html`,
  `ident-three-code-points.html` and the others of section 4) are not adapted
  yet; the claims above rest on specification-derived tests, properties and the
  differential only.

### Support items without a clause

The `input` and `lexer::protocol` types that are not clauses (the requests, records and errors of the host protocol) and their layout pins in `tests/layout_pins.rs` map to no clause and claim none. The errors (`InputError`, `ProtocolError`, `NumberOverflow`, `LexicalError`) are structs with private causes and `is_*` predicates; their tests are `protocol_errors_name_their_cause_and_describe_it`, `a_digit_count_that_overflows_names_its_counter_and_leaves_the_cursor_unchanged` and `a_number_overflow_describes_itself_and_is_an_error` (`src/lexer/protocol/number.rs`), and the doctests. The performance gate (`tests/lexer_performance.rs`) is a measurement, not a conformance claim. `Offset`, the `limits` module (`Limits`, `DepthExcess`, `Meter`, `Failure`,
`Refused`, `grow`, `copy_str`, `copy_boxed`) and `match_ignore_ascii_case!` are
not defined by Syntax 3, so they map to no clause and claim none. They are
tested in `tests/limits.rs` and `tests/match_ignore_ascii_case.rs`.

Sections 7 and 8 define how grammars and style sheets use the syntax. They give
the syntax a CSS meaning and are outside this crate; `surgeist-css` records
their coverage.

[syntax]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md
[input-byte-stream]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#input-byte-stream
[input-preprocessing]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#input-preprocessing
[token-diagrams]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#token-diagrams
[tokenizer-definitions]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#tokenizer-definitions
[valid-escape]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#starts-with-a-valid-escape
[would-start-ident]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#would-start-an-identifier
[would-start-number]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#starts-with-a-number
[would-start-unicode-range]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#starts-a-unicode-range
[tokenization]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#tokenization
[consume-unicode-range-token]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-unicode-range-token
[tokenizer-algorithms]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#tokenizer-algorithms
[consume-token]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-token
[consume-comment]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-comment
[consume-numeric-token]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-numeric-token
[consume-ident-like-token]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-ident-like-token
[consume-string-token]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-string-token
[consume-url-token]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-url-token
[consume-escaped-code-point]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-escaped-code-point
[consume-name]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-name
[consume-number]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-number
[consume-remnants-of-bad-url]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#consume-remnants-of-bad-url
[css-tree]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#css-tree
[parser-definitions]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-definitions
[parser-entry-points]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-entry-points
[parser-algorithms]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-algorithms
[anb-microsyntax]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#anb-microsyntax
[serialization]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#serialization
