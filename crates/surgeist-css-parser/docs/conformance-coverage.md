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

| Clause | Subject | Status | Tests |
| --- | --- | --- | --- |
| [3.2][input-byte-stream] | The input byte stream: decoding and fallback encoding | Not claimed | None |
| [3.3][input-preprocessing] | Preprocessing the input stream | Not claimed | None |
| [4.1][token-diagrams] | Token set | Not claimed | None |
| [4.2][tokenizer-definitions] | Definitions: the code-point classes digit, hex digit, uppercase letter, lowercase letter, letter, ident-start code point, ident code point, non-printable code point, newline and whitespace | Claimed in part (specification-derived, `tests/lexical.rs`) | See [4.2 code-point class tests](#42-code-point-class-tests) |
| [4.2][tokenizer-definitions] | Definitions: the non-ASCII ident code point | Not claimed for the pinned edition. The function implements the 2021 definition (every code point from U+0080), which the pinned edition replaces with a list of ranges; the test pins the 2021 behavior, not the pinned one | `non_ascii_ident_code_point_is_every_code_point_from_u0080_under_the_2021_rule` |
| [4.2][tokenizer-definitions] | Definitions: the stream terms (next input code point, current input code point, reconsume, EOF code point) and the maximum allowed code point | Not claimed | None |
| [4.3.8][valid-escape] | Check if two code points are a valid escape | Claimed (specification-derived, `tests/lexical.rs`) | `escape_is_valid_when_a_backslash_is_followed_by_a_non_newline`, `escape_is_invalid_when_the_backslash_is_followed_by_a_newline`, `escape_is_valid_when_the_backslash_is_followed_by_eof`, `escape_is_invalid_when_the_first_code_point_is_not_a_backslash`, `escape_treats_carriage_return_and_form_feed_as_ordinary_second_code_points` |
| [4.3.9][would-start-ident] | Check if three code points would start an ident sequence | Claimed in part: for non-ASCII input the 2021 non-ASCII ident code point rule above differs from the pinned clause, so only ASCII input is claimed (specification-derived, `tests/lexical.rs`) | `ident_starts_after_hyphen_minus_when_the_second_is_an_ident_start`, `ident_starts_after_hyphen_minus_when_the_second_is_another_hyphen_minus`, `ident_starts_after_hyphen_minus_when_the_second_and_third_are_a_valid_escape`, `ident_does_not_start_after_hyphen_minus_with_a_backslash_and_a_newline`, `ident_does_not_start_after_hyphen_minus_followed_by_anything_else`, `ident_starts_with_an_ident_start_code_point_whatever_follows`, `ident_starts_with_a_backslash_when_the_first_two_are_a_valid_escape`, `ident_does_not_start_with_a_backslash_before_a_newline`, `ident_does_not_start_with_anything_else` |
| [4.3.10][would-start-number] | Check if three code points would start a number | Claimed (specification-derived, `tests/lexical.rs`) | `number_starts_with_a_sign_followed_by_a_digit`, `number_starts_with_a_sign_a_full_stop_and_a_digit`, `number_does_not_start_with_a_sign_a_full_stop_and_a_non_digit`, `number_does_not_start_with_a_sign_followed_by_anything_else`, `number_starts_with_a_full_stop_followed_by_a_digit`, `number_does_not_start_with_a_full_stop_followed_by_a_non_digit`, `number_starts_with_a_digit_whatever_follows`, `number_does_not_start_with_anything_else` |
| [4.3.11][would-start-unicode-range] | Check if three code points would start a unicode-range | Claimed (specification-derived, `tests/lexical.rs`) | `unicode_range_starts_with_u_plus_and_a_question_mark_or_hex_digit`, `unicode_range_does_not_start_when_the_first_code_point_is_not_u`, `unicode_range_does_not_start_when_the_second_code_point_is_not_plus_sign`, `unicode_range_does_not_start_when_the_third_code_point_is_neither_question_mark_nor_hex` |
| [4.3][tokenizer-algorithms] | The other tokenizer algorithms (4.3.1 to 4.3.7 and 4.3.12 to 4.3.15), including `unicode-range` tokens | Not claimed | None |
| [5.2][css-tree] and [5.3][parser-definitions] | Parsing results and token streams | Not claimed | None |
| [5.4][parser-entry-points] | Parser entry points | Not claimed | None |
| [5.5][parser-algorithms] | Parser algorithms | Not claimed | None |
| [6][anb-microsyntax] | The `An+B` microsyntax | Not claimed | None |
| [9][serialization] | Serialization and the round-trip requirement | Not claimed | None |

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
- The WPT tests the edition cites for these clauses (`inclusive-ranges.html`,
  `ident-three-code-points.html`) are not adapted yet; the claims above rest on
  specification-derived tests only.

### Support items without a clause

`Offset`, the `limits` module (`Limits`, `DepthExcess`, `Meter`, `Failure`,
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
[tokenizer-algorithms]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#tokenizer-algorithms
[css-tree]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#css-tree
[parser-definitions]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-definitions
[parser-entry-points]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-entry-points
[parser-algorithms]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-algorithms
[anb-microsyntax]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#anb-microsyntax
[serialization]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#serialization
