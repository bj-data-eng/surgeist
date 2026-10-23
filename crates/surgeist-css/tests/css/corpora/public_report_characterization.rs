//! Characterization of public-parser report observables.
//!
//! `fixtures/public-report-characterization.tsv` is the characterization
//! golden file, captured from the crate's own output by `characterization`.
//! Its report cases (ids under `catalog.` and `focused.`) record the clean
//! flag, retained structure identities, authored declarations and diagnostic
//! tuples of the sheet and style-attribute fronts in the first eight columns;
//! this module reads those columns. It is not a correctness oracle: a matching
//! row shows only that observed behavior is unchanged, and re-capturing a row
//! is never evidence that a behavior change is right. Rows whose behavior was
//! reconciled with the specifications are retired from the report case table
//! in favor of the spec-cited cases in `report_expectations.rs`. Typed value
//! expectations below are handwritten from each row's authored value and cite
//! their specification basis.
//!
//! The value capability in `authored_declarations` (`public` or `typed-value`)
//! is this harness's classification of how a test reads the value, not a
//! product contract: the capture writes `typed-value` for every typed known
//! property value, `public` for custom and substitution-dependent slices.
//! Letter-spacing and the two list-style samples were relabelled from `public`
//! to `typed-value` when the file became captured; their values and typed
//! checks are unchanged.

use std::collections::BTreeSet;

use surgeist_css::{
    CssDeclaration, CssDeclarationContextRef, CssErrorCode, CssImportance, CssPropertyNameRef,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssScopedRule, parse_sheet,
    parse_style_attribute,
};

const FIXTURE: &str = include_str!("../../fixtures/public-report-characterization.tsv");
// Case inputs and feature labels retain their capture provenance. Validation
// is unconditional, so every historical feature profile runs through the same
// public API.
// Selected value expectations follow CSS Variables 1 §2.1 (2022-06-16): a
// CSS-wide keyword followed by ordinary custom-property tokens is
// declaration-value, not a global value.
// https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#syntax
// Selected report expectations follow Fonts4 section 4.1 (missing
// matching descriptors do not invalidate an authored font face) and the authored
// child/declaration-run model introduced by 54a4f4e4b21a3bb506bc4e37adb6c724fb69e773.
// https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-rule
// The late-namespace diagnostic describes the initial-layer exception defined
// by https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#layer-empty.
// Deep style inputs retain each authored ancestor through parse_sheet's public
// depth-256 boundary; dropping level 257 does not flatten its retained parents.
// The Unicode-range boundary case identifies its original out-of-domain end
// endpoint token, following Syntax 3 §§4.3.14 and 5.5.11 and the descriptor token-origin contract.
// Fonts 4 §6.9.1 also retains named font-feature-values rules and their valid
// styleset entries, without projecting descriptors into property declarations.
// CSS Text 4 §8.2 retains the authored 0.1em exactly.
// https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-letter-spacing
// Lists 3 accepts functional symbols() styles and positional-keyword custom
// style names after the position slot is filled.
// https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-list-style
const HEADER: &str = "case_id\tentry\tfeature\tinput\tclean\tretained\tauthored_declarations\tdiagnostics\targument\tshape\tspecified\tcssom\tcharges\tfacts";
const COLUMNS: usize = 14;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Row {
    case_id: String,
    entry: String,
    feature: String,
    input: String,
    clean: String,
    retained: String,
    authored_declarations: String,
    diagnostics: String,
    /// The characterization columns after the report columns; this module
    /// carries them unchanged.
    characterization: [String; COLUMNS - 8],
}

impl Row {
    fn fields(&self) -> Vec<&str> {
        let mut fields = vec![
            self.case_id.as_str(),
            &self.entry,
            &self.feature,
            &self.input,
            &self.clean,
            &self.retained,
            &self.authored_declarations,
            &self.diagnostics,
        ];
        fields.extend(self.characterization.iter().map(String::as_str));
        fields
    }

    /// A sheet or style-attribute report case, as opposed to the other
    /// characterization fronts sharing the golden file.
    fn is_report_case(&self) -> bool {
        self.case_id.starts_with("catalog.") || self.case_id.starts_with("focused.")
    }
}

fn parse_fixture(source: &str) -> Result<Vec<Row>, String> {
    let mut lines = source.lines();
    let header = lines.next().ok_or("fixture is empty")?;
    if header != HEADER {
        return Err(format!("unknown or reordered columns: `{header}`"));
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut ids = BTreeSet::new();
    for (line_index, line) in lines.enumerate() {
        if line.is_empty() {
            return Err(format!("blank row at fixture line {}", line_index + 2));
        }
        let raw = line.split('\t').collect::<Vec<_>>();
        if raw.len() != COLUMNS {
            return Err(format!(
                "fixture line {} has {} columns, expected {COLUMNS}",
                line_index + 2,
                raw.len()
            ));
        }
        let fields = raw
            .into_iter()
            .map(unescape)
            .collect::<Result<Vec<_>, _>>()?;
        if fields
            .iter()
            .enumerate()
            .any(|(index, field)| index != 3 && field.is_empty())
        {
            return Err(format!(
                "absent required observable at fixture line {}",
                line_index + 2
            ));
        }
        let row = Row {
            case_id: fields[0].clone(),
            entry: fields[1].clone(),
            feature: fields[2].clone(),
            input: fields[3].clone(),
            clean: fields[4].clone(),
            retained: fields[5].clone(),
            authored_declarations: fields[6].clone(),
            diagnostics: fields[7].clone(),
            characterization: std::array::from_fn(|index| fields[8 + index].clone()),
        };
        if !ids.insert(row.case_id.clone()) {
            return Err(format!("duplicate case ID `{}`", row.case_id));
        }
        if let Some(previous) = rows.last()
            && previous.case_id >= row.case_id
        {
            return Err(format!(
                "noncanonical case order: `{}` before `{}`",
                previous.case_id, row.case_id
            ));
        }
        if !row.is_report_case() {
            rows.push(row);
            continue;
        }
        match row.entry.as_str() {
            "sheet" | "style" => {}
            value => return Err(format!("{}: unknown entry point `{value}`", row.case_id)),
        }
        match row.feature.as_str() {
            "both" | "default" | "app-strict" => {}
            value => return Err(format!("{}: unknown feature mode `{value}`", row.case_id)),
        }
        match row.clean.as_str() {
            "true" | "false" => {}
            value => return Err(format!("{}: invalid clean state `{value}`", row.case_id)),
        }
        validate_retained_field(&row.retained, &row.case_id)?;
        validate_authored_field(&row)?;
        validate_diagnostics_field(&row.diagnostics, &row.case_id)?;
        rows.push(row);
    }
    if rows.is_empty() {
        return Err("fixture has no cases".to_owned());
    }
    Ok(rows)
}

fn sequence_members<'a>(
    field: &'a str,
    field_name: &str,
    case_id: &str,
) -> Result<Vec<&'a str>, String> {
    if field == "-" {
        return Ok(Vec::new());
    }
    let members = field.split('~').collect::<Vec<_>>();
    if members
        .iter()
        .any(|member| member.is_empty() || *member == "-")
    {
        return Err(format!(
            "{case_id}: malformed {field_name} sequence `{field}`"
        ));
    }
    Ok(members)
}

fn validate_retained_field(field: &str, case_id: &str) -> Result<(), String> {
    for item in sequence_members(field, "retained", case_id)? {
        if let Some(id) = item.strip_prefix("rule:") {
            if !is_prefixed_stable_id(id, "baseline.rule.")
                && id != "later.rule.namespace"
                && id != "later.rule.counter-style"
                && id != "later.rule.page"
                && id != "later.rule.font-feature-values"
                && id != "nested-declarations"
            {
                return Err(format!(
                    "{case_id}: malformed retained rule identity `{id}`"
                ));
            }
        } else if let Some(id) = item.strip_prefix("property:") {
            if !is_property_identity(id) {
                return Err(format!(
                    "{case_id}: malformed retained property identity `{id}`"
                ));
            }
        } else {
            return Err(format!(
                "{case_id}: malformed retained sequence member `{item}`"
            ));
        }
    }
    Ok(())
}

fn is_prefixed_stable_id(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix).is_some_and(is_stable_slug)
}

fn is_stable_slug(slug: &str) -> bool {
    !slug.is_empty()
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_property_identity(id: &str) -> bool {
    is_prefixed_stable_id(id, "baseline.property.")
        || id.strip_prefix("custom:--").is_some_and(|name| {
            !name.is_empty()
                && name.chars().all(|character| {
                    character == '-'
                        || character == '_'
                        || character.is_alphanumeric()
                        || !character.is_ascii()
                })
        })
}

fn validate_diagnostics_field(field: &str, case_id: &str) -> Result<(), String> {
    for diagnostic in sequence_members(field, "diagnostic", case_id)? {
        validate_diagnostic(diagnostic, case_id)?;
    }
    Ok(())
}

fn validate_diagnostic(diagnostic: &str, case_id: &str) -> Result<(), String> {
    let (code, remainder) = diagnostic
        .split_once('/')
        .ok_or_else(|| format!("{case_id}: malformed diagnostic code/root `{diagnostic}`"))?;
    if !is_diagnostic_code(code) {
        return Err(format!("{case_id}: unknown diagnostic code `{code}`"));
    }
    let (root_and_payload, action_and_coordinates) = remainder
        .rsplit_once('/')
        .ok_or_else(|| format!("{case_id}: malformed diagnostic code/root `{diagnostic}`"))?;
    validate_diagnostic_root_and_payload(code, root_and_payload, case_id)?;

    let (action, coordinates) = action_and_coordinates
        .split_once('@')
        .ok_or_else(|| format!("{case_id}: malformed recovery action/coordinates"))?;
    if !matches!(
        action,
        "DropDeclaration"
            | "DropDescriptor"
            | "DropQualifiedRule"
            | "DropAtRule"
            | "DropKeyframeBlock"
            | "DropSelectorListItem"
            | "ReplaceMediaQueryWithNever"
            | "RetainWithImplicitClosure"
            | "IgnoreLegacyToken"
            | "StopAtNestingLimit"
    ) {
        return Err(format!("{case_id}: unknown recovery action `{action}`"));
    }

    let (position, span) = coordinates
        .split_once('>')
        .ok_or_else(|| format!("{case_id}: ill-formed diagnostic position `{coordinates}`"))?;
    let position = parse_coordinate(position, "diagnostic position", case_id)?;
    let (start, end) = span
        .split_once('-')
        .ok_or_else(|| format!("{case_id}: ill-formed diagnostic span `{span}`"))?;
    let start = parse_coordinate(start, "diagnostic span start", case_id)?;
    let end = parse_span_end(end, case_id)?;
    let start_line_column = (start.line, start.column);
    let position_line_column = (position.line, position.column);
    let end_line_column = (end.line, end.column);
    if start.byte > end.byte
        || start_line_column > end_line_column
        || position.byte < start.byte
        || position.byte > end.byte
        || position_line_column < start_line_column
        || position_line_column > end_line_column
    {
        return Err(format!(
            "{case_id}: reversed or invalid diagnostic span `{span}`"
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct FixtureCoordinate {
    byte: usize,
    line: usize,
    column: usize,
}

fn parse_coordinate(
    serialized: &str,
    field_name: &str,
    case_id: &str,
) -> Result<FixtureCoordinate, String> {
    let fields = serialized.split(':').collect::<Vec<_>>();
    if fields.len() != 3 {
        return Err(format!("{case_id}: ill-formed {field_name} `{serialized}`"));
    }
    Ok(FixtureCoordinate {
        byte: parse_decimal(fields[0], field_name, case_id)?,
        line: parse_decimal(fields[1], field_name, case_id)?,
        column: parse_decimal(fields[2], field_name, case_id)?,
    })
}

fn parse_span_end(serialized: &str, case_id: &str) -> Result<FixtureCoordinate, String> {
    let fields = serialized.split(':').collect::<Vec<_>>();
    if fields.len() != 4 {
        return Err(format!(
            "{case_id}: ill-formed diagnostic span end `{serialized}`"
        ));
    }
    let coordinate = FixtureCoordinate {
        byte: parse_decimal(fields[0], "diagnostic span end", case_id)?,
        line: parse_decimal(fields[1], "diagnostic span end", case_id)?,
        column: parse_decimal(fields[2], "diagnostic span end", case_id)?,
    };
    let repeated_end_byte = parse_decimal(fields[3], "diagnostic span end", case_id)?;
    if coordinate.byte != repeated_end_byte {
        return Err(format!(
            "{case_id}: inconsistent diagnostic span endpoint `{serialized}`"
        ));
    }
    Ok(coordinate)
}

fn parse_decimal(serialized: &str, field_name: &str, case_id: &str) -> Result<usize, String> {
    if serialized.is_empty() || !serialized.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "{case_id}: nondecimal {field_name} coordinate `{serialized}`"
        ));
    }
    serialized
        .parse()
        .map_err(|_| format!("{case_id}: out-of-range {field_name} coordinate `{serialized}`"))
}

fn is_diagnostic_code(code: &str) -> bool {
    matches!(
        code,
        "UnexpectedEnd"
            | "UnexpectedToken"
            | "InvalidEncodingDeclaration"
            | "InvalidAtRulePlacement"
            | "InvalidAtRulePrelude"
            | "InvalidAtRuleBody"
            | "UnknownAtRule"
            | "UnsupportedAtRule"
            | "InvalidQualifiedRule"
            | "InvalidSelector"
            | "InvalidMediaQuery"
            | "UnknownProperty"
            | "UnsupportedProperty"
            | "InvalidPropertyValue"
            | "InvalidDeclarationAnnotation"
            | "UnknownDescriptor"
            | "UnsupportedDescriptor"
            | "InvalidDescriptorValue"
            | "InvalidDescriptorCombination"
            | "InvalidColorSyntax"
            | "NestingLimit"
    )
}

fn validate_diagnostic_root_and_payload(
    code: &str,
    serialized: &str,
    case_id: &str,
) -> Result<(), String> {
    let (root, payload) = serialized
        .split_once(':')
        .ok_or_else(|| format!("{case_id}: malformed diagnostic code/root `{serialized}`"))?;
    if root != code {
        return Err(format!(
            "{case_id}: diagnostic code/root family mismatch `{code}`/`{root}`"
        ));
    }
    if payload.is_empty() {
        return Err(format!("{case_id}: empty diagnostic payload for `{root}`"));
    }

    match root {
        "UnexpectedEnd" | "UnknownAtRule" | "UnknownProperty" => {
            validate_payload_parts(payload, 1, root, case_id)
        }
        "InvalidAtRulePlacement" | "UnsupportedAtRule" | "UnknownDescriptor" | "NestingLimit" => {
            validate_payload_parts(payload, 2, root, case_id)
        }
        "UnsupportedProperty" => {
            let parts = payload.split(':').collect::<Vec<_>>();
            if parts.len() != 2 || !is_prefixed_stable_id(parts[0], "baseline.property.") {
                return Err(format!(
                    "{case_id}: malformed diagnostic payload for `{root}`"
                ));
            }
            validate_nonempty_parts(&parts, root, case_id)
        }
        "UnsupportedDescriptor" | "InvalidDescriptorCombination" => {
            validate_payload_parts(payload, 3, root, case_id)
        }
        "UnexpectedToken" => validate_token_payload(payload, 1, false, root, case_id),
        "InvalidEncodingDeclaration" => validate_token_payload(payload, 1, true, root, case_id),
        "InvalidAtRulePrelude" | "InvalidAtRuleBody" | "InvalidDescriptorValue" => {
            validate_token_payload(payload, 3, true, root, case_id)
        }
        "InvalidQualifiedRule" | "InvalidSelector" | "InvalidMediaQuery" | "InvalidColorSyntax" => {
            validate_token_payload(payload, 2, true, root, case_id)
        }
        "InvalidPropertyValue" => {
            let mut parts = payload.splitn(3, ':');
            let property = parts.next().unwrap_or_default();
            let expectation = parts.next().unwrap_or_default();
            let token = parts.next().unwrap_or_default();
            if !is_prefixed_stable_id(property, "baseline.property.") || expectation.is_empty() {
                return Err(format!(
                    "{case_id}: malformed diagnostic payload for `{root}`"
                ));
            }
            validate_token(token, true, root, case_id)
        }
        "InvalidDeclarationAnnotation" => {
            let (context, token) = split_token_suffix(payload, root, case_id)?;
            validate_declaration_context(context, root, case_id)?;
            validate_token(token, false, root, case_id)
        }
        _ => unreachable!("validated diagnostic code/root"),
    }
}

fn validate_payload_parts(
    payload: &str,
    expected: usize,
    root: &str,
    case_id: &str,
) -> Result<(), String> {
    let parts = payload.split(':').collect::<Vec<_>>();
    if parts.len() != expected {
        return Err(format!(
            "{case_id}: malformed diagnostic payload for `{root}`"
        ));
    }
    validate_nonempty_parts(&parts, root, case_id)
}

fn validate_nonempty_parts(parts: &[&str], root: &str, case_id: &str) -> Result<(), String> {
    if parts.iter().any(|part| part.is_empty()) {
        return Err(format!(
            "{case_id}: malformed diagnostic payload for `{root}`"
        ));
    }
    Ok(())
}

fn validate_token_payload(
    payload: &str,
    prefix_parts: usize,
    allow_absent_token: bool,
    root: &str,
    case_id: &str,
) -> Result<(), String> {
    let parts = payload.splitn(prefix_parts + 1, ':').collect::<Vec<_>>();
    if parts.len() != prefix_parts + 1 || parts[..prefix_parts].iter().any(|part| part.is_empty()) {
        return Err(format!(
            "{case_id}: malformed diagnostic payload for `{root}`"
        ));
    }
    validate_token(parts[prefix_parts], allow_absent_token, root, case_id)
}

fn validate_token(
    token: &str,
    allow_absent: bool,
    root: &str,
    case_id: &str,
) -> Result<(), String> {
    if allow_absent && token == "-" {
        return Ok(());
    }
    let (kind, authored) = token
        .split_once(':')
        .ok_or_else(|| format!("{case_id}: malformed diagnostic payload for `{root}`"))?;
    if !is_token_kind(kind) || authored.is_empty() {
        return Err(format!(
            "{case_id}: malformed diagnostic payload for `{root}`"
        ));
    }
    Ok(())
}

fn split_token_suffix<'a>(
    payload: &'a str,
    root: &str,
    case_id: &str,
) -> Result<(&'a str, &'a str), String> {
    for kind in TOKEN_KINDS {
        let marker = format!(":{kind}:");
        if let Some(index) = payload.rfind(&marker) {
            let context = &payload[..index];
            let token = &payload[index + 1..];
            if !context.is_empty() {
                return Ok((context, token));
            }
        }
    }
    Err(format!(
        "{case_id}: malformed diagnostic payload for `{root}`"
    ))
}

fn validate_declaration_context(context: &str, root: &str, case_id: &str) -> Result<(), String> {
    let valid = context
        .strip_prefix("known:")
        .or_else(|| context.strip_prefix("keyframe:"))
        .is_some_and(|id| is_prefixed_stable_id(id, "baseline.property."))
        || context
            .strip_prefix("custom:")
            .or_else(|| context.strip_prefix("keyframe-custom:"))
            .is_some_and(|name| is_property_identity(&format!("custom:{name}")))
        || context.strip_prefix("descriptor:").is_some_and(|details| {
            let parts = details.split(':').collect::<Vec<_>>();
            parts.len() == 2 && parts.iter().all(|part| !part.is_empty())
        })
        || context == "future";
    if !valid {
        return Err(format!(
            "{case_id}: malformed diagnostic payload for `{root}`"
        ));
    }
    Ok(())
}

// This validates archived diagnostic rows, including token names emitted by
// their historical provider vocabulary. Runtime emission below follows the
// selected public CssTokenKind enum; frozen capture provenance stays intact.
const TOKEN_KINDS: &[&str] = &[
    "Ident",
    "AtKeyword",
    "Hash",
    "IdHash",
    "String",
    "Url",
    "Delim",
    "Number",
    "Percentage",
    "Dimension",
    "Whitespace",
    "Comment",
    "Colon",
    "Semicolon",
    "Comma",
    "IncludeMatch",
    "DashMatch",
    "PrefixMatch",
    "SuffixMatch",
    "SubstringMatch",
    "Cdo",
    "Cdc",
    "Function",
    "ParenthesisBlock",
    "SquareBracketBlock",
    "CurlyBracketBlock",
    "BadUrl",
    "BadString",
    "CloseParenthesis",
    "CloseSquareBracket",
    "CloseCurlyBracket",
];

fn is_token_kind(kind: &str) -> bool {
    TOKEN_KINDS.contains(&kind)
}

fn validate_authored_field(row: &Row) -> Result<(), String> {
    let retained = row
        .retained
        .split('~')
        .filter_map(|item| item.strip_prefix("property:"))
        .collect::<Vec<_>>();
    let authored = parse_authored_declarations(&row.authored_declarations, &row.case_id)?;
    let authored_ids = authored
        .iter()
        .map(|declaration| declaration.id.as_str())
        .collect::<Vec<_>>();
    if retained != authored_ids {
        return Err(format!(
            "{}: retained/authored declaration identity mismatch",
            row.case_id
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuthoredDeclaration<'a> {
    id: String,
    value_capability: &'a str,
    value: &'a str,
    importance_capability: &'a str,
    importance: &'a str,
}

struct FrozenDeclarationCursor<'a> {
    case_id: &'a str,
    input: &'a str,
    authored_declarations: Vec<AuthoredDeclaration<'a>>,
    authored_index: usize,
}

impl<'a> FrozenDeclarationCursor<'a> {
    fn new(row: &'a Row) -> Self {
        Self {
            case_id: &row.case_id,
            input: &row.input,
            authored_declarations: parse_authored_declarations(
                &row.authored_declarations,
                &row.case_id,
            )
            .expect("frozen authored-declaration expectation"),
            authored_index: 0,
        }
    }

    fn next(&mut self) -> AuthoredDeclaration<'a> {
        let authored = self
            .authored_declarations
            .get(self.authored_index)
            .unwrap_or_else(|| panic!("{}: unexpected retained declaration", self.case_id))
            .clone();
        self.authored_index += 1;
        authored
    }

    fn finish(&self) {
        assert!(
            self.authored_declarations
                .get(self.authored_index)
                .is_none(),
            "{}: fixture contains an unmatched authored declaration observable",
            self.case_id,
        );
    }
}

fn parse_authored_declarations<'a>(
    field: &'a str,
    case_id: &str,
) -> Result<Vec<AuthoredDeclaration<'a>>, String> {
    if field == "-" {
        return Ok(Vec::new());
    }
    field
        .split('~')
        .map(|item| {
            let (id, observation) = item
                .split_once('=')
                .ok_or_else(|| format!("{case_id}: malformed authored declaration `{item}`"))?;
            let (value_observation, importance_observation) = observation
                .rsplit_once('@')
                .ok_or_else(|| format!("{case_id}: missing authored importance in `{item}`"))?;
            let (value_capability, value) = value_observation
                .split_once(':')
                .ok_or_else(|| format!("{case_id}: missing value capability in `{item}`"))?;
            let (importance_capability, importance) = importance_observation
                .split_once(':')
                .ok_or_else(|| format!("{case_id}: missing importance capability in `{item}`"))?;
            if !matches!(value_capability, "public" | "typed-value") {
                return Err(format!(
                    "{case_id}: unknown authored-value capability `{value_capability}`"
                ));
            }
            if !matches!(importance_capability, "public" | "keyframe-grammar") {
                return Err(format!(
                    "{case_id}: unknown importance capability `{importance_capability}`"
                ));
            }
            if !matches!(importance, "normal" | "important")
                || (importance_capability == "keyframe-grammar" && importance != "normal")
            {
                return Err(format!(
                    "{case_id}: invalid authored importance `{importance_observation}`"
                ));
            }
            Ok(AuthoredDeclaration {
                id: id.to_owned(),
                value_capability,
                value,
                importance_capability,
                importance,
            })
        })
        .collect()
}

fn unescape(field: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut chars = field.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('\\') => output.push('\\'),
            Some('t') => output.push('\t'),
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some(escaped) => return Err(format!("malformed escape `\\{escaped}`")),
            None => return Err("trailing fixture escape".to_owned()),
        }
    }
    Ok(output)
}

#[test]
fn authored_css_cases_match_selected_public_report_observables() {
    let rows = parse_fixture(FIXTURE).expect("valid observable fixture");
    for row in rows.into_iter().filter(Row::is_report_case) {
        let actual = observe(&row);
        assert_eq!(actual.clean, row.clean, "{} clean report", row.case_id);
        assert_eq!(
            actual.retained, row.retained,
            "{} retained syntax",
            row.case_id
        );
        assert_eq!(
            actual.diagnostics, row.diagnostics,
            "{} diagnostics",
            row.case_id
        );
        assert_strict_parity(&row);
    }
}

#[test]
fn malformed_observable_fixture_rows_are_rejected() {
    let rows = parse_fixture(FIXTURE).expect("valid fixture");
    assert!(
        parse_fixture(&FIXTURE.replacen(HEADER, &format!("{HEADER}\textra"), 1))
            .expect_err("unknown column must fail")
            .contains("unknown or reordered columns")
    );
    let duplicate = format!(
        "{HEADER}\n{}\n{}\n",
        render_row(&rows[0]),
        render_row(&rows[0])
    );
    assert!(
        parse_fixture(&duplicate)
            .expect_err("duplicate ID must fail")
            .contains(&rows[0].case_id)
    );
    let malformed = format!("{HEADER}\n{}\\q\n", render_row(&rows[0]));
    assert!(
        parse_fixture(&malformed)
            .expect_err("malformed escape must fail")
            .contains("malformed escape")
    );
    let noncanonical = format!(
        "{HEADER}\n{}\n{}\n",
        render_row(&rows[1]),
        render_row(&rows[0])
    );
    assert!(
        parse_fixture(&noncanonical)
            .expect_err("noncanonical order must fail")
            .contains("noncanonical case order")
    );

    let mut invalid_entry = rows[0].clone();
    invalid_entry.entry = "unknown".to_owned();
    assert!(
        parse_fixture(&format!("{HEADER}\n{}\n", render_row(&invalid_entry)))
            .expect_err("unknown entry point must fail")
            .contains("unknown entry point")
    );

    let mut invalid_feature = rows[0].clone();
    invalid_feature.feature = "unknown".to_owned();
    assert!(
        parse_fixture(&format!("{HEADER}\n{}\n", render_row(&invalid_feature)))
            .expect_err("unknown feature mode must fail")
            .contains("unknown feature mode")
    );

    let mut invalid_clean = rows[0].clone();
    invalid_clean.clean = "unknown".to_owned();
    assert!(
        parse_fixture(&format!("{HEADER}\n{}\n", render_row(&invalid_clean)))
            .expect_err("invalid clean state must fail")
            .contains("invalid clean state")
    );

    let mut missing = rows.clone();
    missing[0].retained.clear();
    let missing = format!(
        "{HEADER}\n{}\n",
        missing
            .iter()
            .map(render_row)
            .collect::<Vec<_>>()
            .join("\n")
    );

    let declaration_index = rows
        .iter()
        .position(|row| row.authored_declarations != "-")
        .expect("fixture must contain retained declarations");
    let mut missing_authored = rows.clone();
    missing_authored[declaration_index]
        .authored_declarations
        .clear();
    let missing_authored = format!(
        "{HEADER}\n{}\n",
        missing_authored
            .iter()
            .map(render_row)
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        parse_fixture(&missing_authored)
            .expect_err("missing authored declaration observable must fail")
            .contains("absent required observable"),
        "responsible case: {}",
        rows[declaration_index].case_id
    );

    let keyframe_index = rows
        .iter()
        .position(|row| {
            row.authored_declarations
                .contains("@keyframe-grammar:normal")
        })
        .expect("fixture must contain keyframe declaration observables");
    let mut missing_keyframe = rows.clone();
    let mut keyframe_declarations = missing_keyframe[keyframe_index]
        .authored_declarations
        .split('~')
        .collect::<Vec<_>>();
    keyframe_declarations.pop().expect("keyframe declaration");
    missing_keyframe[keyframe_index].authored_declarations =
        nonempty(keyframe_declarations.join("~"));
    let missing_keyframe = format!(
        "{HEADER}\n{}\n",
        missing_keyframe
            .iter()
            .map(render_row)
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        parse_fixture(&missing_keyframe)
            .expect_err("missing keyframe declaration observable must fail")
            .contains(&rows[keyframe_index].case_id)
    );
    assert!(
        parse_fixture(&missing)
            .expect_err("missing observable must fail")
            .contains("absent required observable"),
        "responsible case: {}",
        rows[0].case_id
    );

    let malformed_diagnostic_row = rows
        .iter()
        .find(|row| row.case_id == "catalog.non-property.baseline.rule.keyframes.boundary")
        .expect("fixture must contain the reserved-keyframes-name recovery diagnostic");
    let diagnostic = &malformed_diagnostic_row.diagnostics;

    let mut malformed_retained_rule = malformed_diagnostic_row.clone();
    malformed_retained_rule.retained = "rule:not-a-stable-id".to_owned();
    assert_fixture_row_rejected(
        &malformed_retained_rule,
        "malformed retained rule identity",
        "retained rule identity",
    );

    let declaration_row = rows
        .iter()
        .find(|row| {
            row.retained.contains("property:baseline.property.") && row.authored_declarations != "-"
        })
        .expect("fixture must contain a property declaration");
    let property_id = declaration_row
        .retained
        .split('~')
        .find_map(|item| item.strip_prefix("property:"))
        .expect("retained property identity");
    let mut malformed_retained_property = declaration_row.clone();
    malformed_retained_property.retained = malformed_retained_property
        .retained
        .replace(property_id, "not-a-stable-id");
    malformed_retained_property.authored_declarations = malformed_retained_property
        .authored_declarations
        .replace(property_id, "not-a-stable-id");
    assert_fixture_row_rejected(
        &malformed_retained_property,
        "malformed retained property identity",
        "retained property identity",
    );

    for retained in [
        "rule:baseline.rule.style~~rule:baseline.rule.media",
        "-~rule:baseline.rule.style",
    ] {
        let mut malformed_retained_sequence = malformed_diagnostic_row.clone();
        malformed_retained_sequence.retained = retained.to_owned();
        assert_fixture_row_rejected(
            &malformed_retained_sequence,
            "malformed retained sequence",
            "retained sequence",
        );
    }

    let diagnostic_cases = [
        (
            diagnostic.replacen("InvalidAtRulePrelude/", "NotACssError/", 1),
            "diagnostic code",
            "unknown diagnostic code",
        ),
        (
            diagnostic.replacen(
                "/InvalidAtRulePrelude:",
                "/UnexpectedEnd:",
                1,
            ),
            "diagnostic code/root family",
            "mismatched diagnostic root",
        ),
        (
            diagnostic.replacen(
                "/InvalidAtRulePrelude:keyframes:baseline.rule.keyframes:a supported keyframes name:Ident:none/",
                "/InvalidAtRulePrelude:/",
                1,
            ),
            "diagnostic payload",
            "empty diagnostic payload",
        ),
        (
            diagnostic.replacen("/DropAtRule@", "/KeepEverything@", 1),
            "recovery action",
            "unknown recovery action",
        ),
        (
            diagnostic.replacen("@11:0:11>", "@eleven:0:11>", 1),
            "diagnostic position",
            "nondecimal diagnostic coordinate",
        ),
        (
            diagnostic.replacen("@11:0:11>", "@11:0>", 1),
            "diagnostic position",
            "ill-formed diagnostic position",
        ),
        (
            diagnostic.replacen(">0:0:0-40:0:40:40", ">0:0-40:0:40:40", 1),
            "diagnostic span",
            "ill-formed diagnostic span",
        ),
        (
            diagnostic.replacen(">0:0:0-40:0:40:40", ">40:0:40-0:0:0:0", 1),
            "diagnostic span",
            "reversed diagnostic span",
        ),
        (
            diagnostic.replacen("-40:0:40:40", "-40:0:40:39", 1),
            "diagnostic span",
            "inconsistent diagnostic span endpoint",
        ),
        (
            format!("{diagnostic}~"),
            "diagnostic sequence",
            "empty diagnostic sequence member",
        ),
        (
            format!("-~{diagnostic}"),
            "diagnostic sequence",
            "empty-sequence marker mixed with diagnostics",
        ),
    ];
    for (malformed_diagnostics, expected_error, reason) in diagnostic_cases {
        let mut malformed = malformed_diagnostic_row.clone();
        malformed.diagnostics = malformed_diagnostics;
        assert_fixture_row_rejected(&malformed, expected_error, reason);
    }

    let mut delimiter_payload = malformed_diagnostic_row.clone();
    delimiter_payload.diagnostics =
        diagnostic.replacen("Ident:none", "Url:url(https://example.test/@value:part)", 1);
    parse_fixture(&format!("{HEADER}\n{}\n", render_row(&delimiter_payload)))
        .expect("authored token payload delimiters remain valid fixture data");
}

fn assert_fixture_row_rejected(row: &Row, expected_error: &str, reason: &str) {
    let source = format!("{HEADER}\n{}\n", render_row(row));
    let error = parse_fixture(&source).expect_err(reason);
    assert!(
        error.contains(expected_error),
        "{reason}: expected `{expected_error}` in `{error}`"
    );
}

#[test]
fn omitted_recovery_diagnostic_changes_the_public_report_observable() {
    let rows = parse_fixture(FIXTURE).expect("valid fixture");
    // Unknown property and invalid width independently require two diagnostics;
    // this witness does not depend on historical font-matching validation.
    let original = rows
        .iter()
        .find(|row| row.case_id == "focused.app-strict.multi-style")
        .expect("fixture contains the named two-error style attribute");
    assert_eq!(
        observe(original).diagnostics,
        original.diagnostics,
        "{} public report matches the complete authored diagnostic sequence",
        original.case_id
    );
    let mut omitted = original.clone();
    omitted.diagnostics = original
        .diagnostics
        .split_once('~')
        .expect("repeated diagnostic")
        .0
        .to_owned();
    assert_ne!(
        observe(&omitted).diagnostics,
        omitted.diagnostics,
        "{} public parser retains every recovery diagnostic in source order",
        omitted.case_id
    );
}

fn render_row(row: &Row) -> String {
    row.fields()
        .into_iter()
        .map(escape)
        .collect::<Vec<_>>()
        .join("\t")
}

fn escape(field: &str) -> String {
    field
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

struct Observation {
    clean: String,
    retained: String,
    diagnostics: String,
}

fn observe(expected: &Row) -> Observation {
    let mut frozen = FrozenDeclarationCursor::new(expected);
    let (clean, retained, diagnostics) = match expected.entry.as_str() {
        "sheet" => {
            let report = parse_sheet(&expected.input);
            let retained = sheet_observables(report.syntax().rules(), &mut frozen);
            (
                report.is_clean(),
                retained,
                diagnostics_observable(report.diagnostics()),
            )
        }
        "style" => {
            let report = parse_style_attribute(&expected.input);
            let retained =
                declaration_observables(report.syntax().as_slice(), "public", &mut frozen);
            (
                report.is_clean(),
                retained,
                diagnostics_observable(report.diagnostics()),
            )
        }
        _ => unreachable!("validated fixture entry"),
    };
    frozen.finish();
    Observation {
        clean: clean.to_string(),
        retained: nonempty(retained.join("~")),
        diagnostics: nonempty(diagnostics.join("~")),
    }
}

fn nonempty(value: String) -> String {
    if value.is_empty() {
        "-".to_owned()
    } else {
        value
    }
}

fn sheet_observables(rules: &[CssRule], frozen: &mut FrozenDeclarationCursor<'_>) -> Vec<String> {
    let mut retained = Vec::new();
    for rule in rules {
        rule_observables(rule, &mut retained, frozen);
    }
    retained
}

fn rule_observables(
    rule: &CssRule,
    retained: &mut Vec<String>,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    match rule {
        CssRule::Import(_) => retained.push("rule:baseline.rule.import".to_owned()),
        CssRule::Namespace(_) => retained.push("rule:later.rule.namespace".to_owned()),
        CssRule::CounterStyle(_) => retained.push("rule:later.rule.counter-style".to_owned()),
        CssRule::Page(_) => retained.push("rule:later.rule.page".to_owned()),
        CssRule::LayerStatement(_) => {
            retained.push("rule:baseline.rule.layer-statement".to_owned())
        }
        CssRule::LayerBlock(rule) => {
            retained.push("rule:baseline.rule.layer-block".to_owned());
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssRule::FontFace(_) => retained.push("rule:baseline.rule.font-face".to_owned()),
        CssRule::FontFeatureValues(_) => {
            retained.push("rule:later.rule.font-feature-values".to_owned())
        }
        CssRule::Keyframes(rule) => {
            retained.push("rule:baseline.rule.keyframes".to_owned());
            for block in rule.blocks() {
                for declaration in block.declarations().iter() {
                    let name = declaration.property_name();
                    let id = property_id(name);
                    retained.push(format!("property:{id}"));
                    let authored = frozen.next();
                    assert_eq!(authored.id, id, "{} declaration identity", frozen.case_id);
                    assert_eq!(
                        authored.importance_capability, "keyframe-grammar",
                        "{} {id} importance capability",
                        frozen.case_id
                    );
                    assert_eq!(
                        authored.importance, "normal",
                        "{} {id} authored importance",
                        frozen.case_id
                    );
                    assert_declaration_value(
                        name,
                        declaration.custom(),
                        declaration.known(),
                        &authored,
                        frozen,
                    );
                }
            }
        }
        CssRule::Style(rule) => {
            retained.push("rule:baseline.rule.style".to_owned());
            let ids = declaration_observables(rule.declarations().as_slice(), "public", frozen);
            retained.extend(ids);
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssRule::NestedDeclarations(rule) => {
            // This is an authored structural node, not another style selector or
            // a fabricated conformance-catalog identity.
            retained.push("rule:nested-declarations".to_owned());
            let ids = declaration_observables(rule.declarations().as_slice(), "public", frozen);
            retained.extend(ids);
        }
        CssRule::Media(rule) => {
            retained.push("rule:baseline.rule.media".to_owned());
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssRule::Supports(rule) => {
            retained.push("rule:baseline.rule.supports".to_owned());
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssRule::Container(rule) => {
            retained.push("rule:baseline.rule.container".to_owned());
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssRule::Scope(rule) => {
            retained.push("rule:baseline.rule.scope".to_owned());
            for child in rule.rules().rules() {
                scoped_rule_observables(child, retained, frozen);
            }
        }
        _ => retained.push("rule:future".to_owned()),
    }
}

fn scoped_rule_observables(
    rule: &CssScopedRule,
    retained: &mut Vec<String>,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    match rule {
        CssScopedRule::NestedDeclarations(rule) => {
            rule_observables(&CssRule::NestedDeclarations(rule.clone()), retained, frozen)
        }
        CssScopedRule::CounterStyle(rule) => {
            rule_observables(&CssRule::CounterStyle(rule.clone()), retained, frozen)
        }
        CssScopedRule::FontFace(rule) => {
            rule_observables(&CssRule::FontFace(rule.clone()), retained, frozen)
        }
        CssScopedRule::Page(rule) => {
            rule_observables(&CssRule::Page(rule.clone()), retained, frozen)
        }
        CssScopedRule::Keyframes(rule) => {
            rule_observables(&CssRule::Keyframes(rule.clone()), retained, frozen)
        }
        CssScopedRule::FontFeatureValues(_) => {
            retained.push("rule:later.rule.font-feature-values".to_owned())
        }
        CssScopedRule::Style(rule) => {
            retained.push("rule:baseline.rule.style".to_owned());
            let ids = declaration_observables(rule.declarations().as_slice(), "public", frozen);
            retained.extend(ids);
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssScopedRule::Media(rule) => {
            for child in rule.rules().rules() {
                scoped_rule_observables(child, retained, frozen);
            }
        }
        CssScopedRule::Supports(rule) => {
            retained.push("rule:baseline.rule.supports".to_owned());
            for child in rule.rules().rules() {
                scoped_rule_observables(child, retained, frozen);
            }
        }
        CssScopedRule::Container(rule) => {
            for child in rule.rules().rules() {
                scoped_rule_observables(child, retained, frozen);
            }
        }
        CssScopedRule::LayerStatement(_) => {
            retained.push("rule:baseline.rule.layer-statement".to_owned())
        }
        CssScopedRule::LayerBlock(rule) => {
            for child in rule.rules().rules() {
                scoped_rule_observables(child, retained, frozen);
            }
        }
        CssScopedRule::Scope(rule) => {
            for child in rule.rules().rules() {
                scoped_rule_observables(child, retained, frozen);
            }
        }
        _ => retained.push("rule:future".to_owned()),
    }
}

fn declaration_observables(
    declarations: &[CssDeclaration],
    importance_capability: &str,
    frozen: &mut FrozenDeclarationCursor<'_>,
) -> Vec<String> {
    let mut retained = Vec::new();
    declaration_values(
        declarations
            .iter()
            .map(|declaration| (declaration.property_name(), Some(declaration))),
        &mut retained,
        importance_capability,
        frozen,
    );
    retained
}

fn declaration_values<'a>(
    declarations: impl Iterator<Item = (CssPropertyNameRef<'a>, Option<&'a CssDeclaration>)>,
    retained: &mut Vec<String>,
    importance_capability: &str,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    for (name, declaration) in declarations {
        let id = property_id(name);
        retained.push(format!("property:{id}"));
        if let Some(declaration) = declaration {
            let authored = frozen.next();
            let importance = match declaration.importance() {
                CssImportance::Normal => "normal",
                CssImportance::Important => "important",
            };
            assert_eq!(authored.id, id, "{} declaration identity", frozen.case_id);
            assert_eq!(
                authored.importance_capability, importance_capability,
                "{} {id} importance capability",
                frozen.case_id
            );
            assert_eq!(
                authored.importance, importance,
                "{} {id} authored importance",
                frozen.case_id
            );
            assert_declaration_value(
                name,
                declaration.custom(),
                declaration.known(),
                &authored,
                frozen,
            );
        }
    }
}

fn property_id(name: CssPropertyNameRef<'_>) -> String {
    match name {
        CssPropertyNameRef::Known(property) => property.stable_id().to_owned(),
        CssPropertyNameRef::Custom(name) => format!("custom:{}", name.as_str()),
        _ => "future-property".to_owned(),
    }
}

fn assert_declaration_value(
    name: CssPropertyNameRef<'_>,
    custom: Option<&surgeist_css::CssCustomDeclaration>,
    known: Option<&surgeist_css::CssKnownDeclaration>,
    authored: &AuthoredDeclaration<'_>,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    if let Some(custom) = custom {
        if let Some(value) = custom.value().value() {
            assert_eq!(
                authored.value_capability, "public",
                "{} {} authored-value capability",
                frozen.case_id, authored.id
            );
            assert_eq!(
                authored.value,
                value.as_css(),
                "{} {} publicly exposed authored slice",
                frozen.case_id,
                authored.id
            );
        } else {
            let keyword = custom.value().global().expect("symbolic custom global");
            assert_eq!(
                authored.value_capability, "typed-value",
                "{} {} authored-value capability",
                frozen.case_id, authored.id
            );
            assert_ne!(
                authored.value, "<unavailable>",
                "{} {} deferred slice must remain explicit in the TSV",
                frozen.case_id, authored.id
            );
            assert_eq!(
                authored.value,
                global_keyword_css(keyword),
                "{} {} custom-global authored slice",
                frozen.case_id,
                authored.id
            );
        }
        return;
    }

    let Some(known) = known else {
        assert_eq!(
            authored.value_capability, "typed-value",
            "{} {} future authored-value capability",
            frozen.case_id, authored.id
        );
        assert_eq!(
            authored.value, "<unavailable>",
            "{} {} future authored slice",
            frozen.case_id, authored.id
        );
        return;
    };
    assert_eq!(
        property_id(name),
        known.property().stable_id(),
        "{} known property/declaration identity",
        frozen.case_id
    );
    match known.declared_value() {
        surgeist_css::CssKnownDeclaredValueRef::Property(value) => {
            assert_known_property_value(known.property(), value, authored, frozen);
        }
        surgeist_css::CssKnownDeclaredValueRef::Global(value) => {
            assert_eq!(
                authored.value_capability, "typed-value",
                "{} {} authored-value capability",
                frozen.case_id, authored.id
            );
            assert_ne!(
                authored.value, "<unavailable>",
                "{} {} deferred slice must remain explicit in the TSV",
                frozen.case_id, authored.id
            );
            assert_eq!(
                authored.value,
                global_keyword_css(value),
                "{} {} global authored slice",
                frozen.case_id,
                authored.id
            );
        }
        surgeist_css::CssKnownDeclaredValueRef::SubstitutionDependent(value) => {
            assert_eq!(
                authored.value_capability, "public",
                "{} {} authored-value capability",
                frozen.case_id, authored.id
            );
            assert_eq!(
                authored.value,
                value.as_css(),
                "{} {} publicly exposed authored slice",
                frozen.case_id,
                authored.id
            );
        }
        _ => {
            assert_eq!(
                authored.value_capability, "typed-value",
                "{} {} future authored-value capability",
                frozen.case_id, authored.id
            );
            assert_eq!(
                authored.value, "<unavailable>",
                "{} {} future authored slice",
                frozen.case_id, authored.id
            );
        }
    }
}

// Color 4 §§5.1, 5.2 and 6.2: a named, hex or `transparent` color keeps its
// authored branch and is not resolved to channels by the parser.
// https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#named-colors
fn assert_captured_color(color: &surgeist_css::CssColor, expected: &str) {
    match expected {
        "red" | "blue" | "black" | "white" => {
            assert_eq!(color.named().expect("named color").name(), expected);
        }
        "transparent" => assert!(color.is_transparent()),
        "#fff" => assert_eq!(color.hex_value().expect("hex color").digits(), "fff"),
        _ => panic!("uncatalogued captured color: {expected}"),
    }
}

fn assert_captured_border(border: &surgeist_css::CssBorder, expected: &str) {
    use surgeist_css::{CssBorderStyle as Style, CssBorderWidth as Width};
    let (width, style, color) = match expected {
        "solid 2px #fff" => (Some("2px"), Some(Style::Solid), Some("#fff")),
        "#fff" => (None, None, Some("#fff")),
        "dashed black" => (None, Some(Style::Dashed), Some("black")),
        "1px" => (Some("1px"), None, None),
        "black dotted" => (None, Some(Style::Dotted), Some("black")),
        _ => panic!("uncatalogued captured border: {expected}"),
    };
    assert_eq!(border.style(), style);
    match (border.width(), width) {
        (Some(Width::Length(actual)), Some(expected)) => {
            assert_eq!(
                actual.serialize_specified(Default::default()).unwrap(),
                expected
            );
            assert!(matches!(
                actual.origin(),
                surgeist_css::CssValueOrigin::Parsed(_)
            ));
            assert!(actual.literal_component().is_some());
        }
        (None, None) => {}
        _ => panic!("captured border width differs: {expected}"),
    }
    match (border.color(), color) {
        (Some(actual), Some(expected)) => assert_captured_color(actual, expected),
        (None, None) => {}
        _ => panic!("captured border color differs: {expected:?}"),
    }
}

fn assert_captured_px(component: Option<&surgeist_css::CssComponentValue>, expected: &str) {
    assert!(exact_dimension(component, expected, "px"));
    assert!(matches!(
        component.unwrap().origin(),
        surgeist_css::CssValueOrigin::Parsed(_)
    ));
}

fn assert_captured_grid_columns(list: &surgeist_css::CssGridTrackList) {
    use surgeist_css::{
        CssGridGeneralTrackComponent as GeneralTrack, CssGridTrackRepeatComponent as RepeatMember,
    };
    let [GeneralTrack::Repeat(repeat)] = list.general_list().unwrap().components() else {
        panic!("one captured integer repeat")
    };
    assert!(
        matches!(repeat.count(), surgeist_css::CssPositiveIntegerValue::Literal(count) if count.integer().numeric().representation() == "2")
    );
    let [RepeatMember::TrackSize(size)] = repeat.content().components() else {
        panic!("one repeated minmax")
    };
    let (min, max) = size.minmax().expect("captured minmax");
    assert_eq!(
        min.length_percentage()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "10px"
    );
    assert_eq!(
        max.flex()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "1fr"
    );
}

// Fonts 4 §2.1: one quoted family name followed by the `sans-serif` generic.
// https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-family-prop
fn assert_captured_font_families(families: &surgeist_css::CssFontFamilyList) {
    assert_eq!(
        families.families(),
        &[
            surgeist_css::CssFontFamilyName::try_quoted("Avenir Next").unwrap(),
            surgeist_css::CssFontFamilyName::generic(surgeist_css::CssGenericFontFamily::SansSerif),
        ]
    );
}

fn assert_captured_font_metadata(
    id: &str,
    current_css: &str,
    authored: &AuthoredDeclaration<'_>,
    case_id: &str,
) {
    assert_eq!(
        authored.id, id,
        "{case_id}: captured font property identity"
    );
    assert_eq!(authored.value_capability, "typed-value");
    assert_ne!(authored.value, "<unavailable>");
    assert_eq!(
        current_css, authored.value,
        "{case_id}: captured authored font value"
    );
}

// Values 4 §10.1: each calc() witness keeps its authored two-term sum, with
// exact leaves, operators, types and source spans.
// https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-func
fn assert_captured_sum_calculation(
    calculation: &surgeist_css::CssLengthPercentageCalculation,
    expected_css: &str,
    first: (&str, bool),
    second: (&str, bool),
    subtract: bool,
    source: &str,
) {
    use surgeist_css::{
        CssCalculationExpressionRef, CssCalculationSumOperator, CssCalculationType,
        CssCalculationValueRef, CssNumericDimension, CssValueOrigin,
    };
    assert_eq!(
        calculation.result_type(),
        CssCalculationType::LengthPercentage
    );
    assert_eq!(
        calculation.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("expected whole calc function");
    };
    let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
        panic!("expected authored two-term sum");
    };
    assert_eq!(sum.len(), 2);
    let start = source
        .find(expected_css)
        .expect("original captured calculation");
    let CssValueOrigin::Parsed(whole) = calculation.components().items()[0].origin() else {
        panic!("expected original function provenance");
    };
    assert_eq!(whole.source().as_str(), source);
    assert_eq!(whole.span().start().byte_offset().value(), start);
    assert_eq!(
        whole.span().end().byte_offset().value(),
        start + "calc(".len()
    );
    let surgeist_css::CssComponentValueRef::Function(function) =
        calculation.components().items()[0].view()
    else {
        panic!("expected original function component");
    };
    let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
        panic!("expected authored closing delimiter");
    };
    assert!(closing.source().same_snapshot(whole.source()));
    assert_eq!(
        closing.span().start().byte_offset().value(),
        start + expected_css.len() - 1
    );
    assert_eq!(
        closing.span().end().byte_offset().value(),
        start + expected_css.len()
    );
    for (index, (number, percentage)) in [first, second].into_iter().enumerate() {
        let term = sum.term(index).unwrap();
        let expected_operator = if index == 0 {
            None
        } else if subtract {
            Some(CssCalculationSumOperator::Subtract)
        } else {
            Some(CssCalculationSumOperator::Add)
        };
        assert_eq!(term.operator(), expected_operator);
        if index == 0 {
            assert_eq!(term.operator_origin(), None);
        } else {
            let Some(CssValueOrigin::Parsed(operator)) = term.operator_origin() else {
                panic!("expected original sum operator provenance");
            };
            let offset = start + expected_css.find(if subtract { '-' } else { '+' }).unwrap();
            assert!(operator.source().same_snapshot(whole.source()));
            assert_eq!(operator.span().start().byte_offset().value(), offset);
            assert_eq!(operator.span().end().byte_offset().value(), offset + 1);
        }
        let CssCalculationExpressionRef::Value(value) = term.expression() else {
            panic!("expected exact numeric leaf");
        };
        let leaf = match (percentage, value) {
            (true, CssCalculationValueRef::Percentage(leaf)) => leaf,
            (false, CssCalculationValueRef::Length(leaf)) => leaf,
            _ => panic!("captured numeric domain changed"),
        };
        assert_eq!(leaf.representation(), number);
        assert_eq!(leaf.unit(), if percentage { None } else { Some("px") });
        let token = format!("{number}{}", if percentage { "%" } else { "px" });
        let offset = start + expected_css.find(&token).unwrap();
        let CssValueOrigin::Parsed(origin) = leaf.origin() else {
            panic!("expected original numeric token provenance");
        };
        assert!(origin.source().same_snapshot(whole.source()));
        assert_eq!(origin.span().start().byte_offset().value(), offset);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            offset + token.len()
        );
    }
}

fn assert_captured_numeric_metadata(id: &str, css: &str, authored: &AuthoredDeclaration<'_>) {
    assert_eq!(authored.id, id);
    assert_eq!(authored.value_capability, "typed-value");
    assert_eq!(css, authored.value);
}

// Sizing 3 §3.1 and Values 4 §§5, 6: each authored size is its exact
// keyword branch, unitless zero, px dimension or percentage token.
// https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/#sizing-values
fn assert_box_size(value: &surgeist_css::CssBoxSize, expected: &str) {
    use surgeist_css::CssBoxSize;
    match (expected, value) {
        ("min-content", CssBoxSize::MinContent)
        | ("max-content", CssBoxSize::MaxContent)
        | ("fit-content", CssBoxSize::FitContent) => {}
        (_, CssBoxSize::LengthPercentage(length)) => assert_spacing_literal(
            length.literal_component().expect("exact authored literal"),
            expected,
        ),
        _ => panic!("authored size `{expected}` changed its branch"),
    }
}

fn assert_preferred_size(value: &surgeist_css::CssSizeValue, expected: &str) {
    match (expected, value) {
        ("auto", surgeist_css::CssSizeValue::Auto) => {}
        (_, surgeist_css::CssSizeValue::BoxSize(value)) => assert_box_size(value, expected),
        _ => panic!("authored preferred size `{expected}` changed its branch"),
    }
}

fn assert_spacing_literal(component: &surgeist_css::CssComponentValue, expected: &str) {
    use surgeist_css::{CssComponentValueRef, CssValueTokenRef};
    let matches = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
            expected == "0" && number.representation() == "0"
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => expected
            .strip_suffix("px")
            .is_some_and(|literal| unit == "px" && number.representation() == literal),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => expected
            .strip_suffix('%')
            .is_some_and(|literal| number.representation() == literal),
        _ => false,
    };
    assert!(
        matches,
        "authored literal `{expected}` changed numeric representation or unit"
    );
}

fn assert_margin(value: &surgeist_css::CssMarginValue, expected: &str) {
    match (expected, value) {
        ("auto", surgeist_css::CssMarginValue::Auto) => {}
        (_, surgeist_css::CssMarginValue::LengthPercentage(value)) => {
            assert_spacing_literal(value.literal_component().unwrap(), expected)
        }
        _ => panic!("authored margin `{expected}` changed its branch"),
    }
}

fn assert_padding(value: &surgeist_css::CssPaddingValue, expected: &str) {
    assert_spacing_literal(
        value.length_percentage().literal_component().unwrap(),
        expected,
    );
}

fn assert_known_property_value(
    property: surgeist_css::CssKnownProperty,
    value: surgeist_css::CssKnownPropertyValueRef<'_>,
    authored: &AuthoredDeclaration<'_>,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    use surgeist_css::{
        CssBoxShadow, CssFilter, CssFilterAmount, CssFilterFunction, CssOutlineStyle,
        CssOutlineWidth, CssTextDecorationLineComponent, CssTextDecorationStyle,
        CssTextDecorationThickness,
    };
    // Color 4 §4.2 <opacity-value> and Values 4 §5.1 <integer>: each scalar
    // keeps its exact authored numeric representation.
    let scalar_authored = match (property, &value) {
        (
            surgeist_css::CssKnownProperty::Opacity,
            surgeist_css::CssKnownPropertyValueRef::Opacity(value),
        ) => {
            let surgeist_css::CssOpacityValue::Scalar(scalar) = value.value() else {
                panic!("captured ordinary opacity")
            };
            assert_eq!(scalar.kind(), surgeist_css::CssOpacityScalarKind::Number);
            assert_eq!(scalar.numeric().representation(), authored.value);
            let expected = match authored.value {
                "0" => "0",
                ".5" | "0.5" => "0.5",
                "1" => "1",
                _ => panic!("unexpected captured opacity"),
            };
            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                expected
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Order,
            surgeist_css::CssKnownPropertyValueRef::Order(value),
        ) => {
            assert_eq!(authored.value, "-2");
            let surgeist_css::CssIntegerValue::Literal(literal) = value.value() else {
                panic!("captured order integer")
            };
            assert_eq!(literal.numeric().representation(), "-2");
            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "-2"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::ZIndex,
            surgeist_css::CssKnownPropertyValueRef::ZIndex(value),
        ) => {
            assert_eq!(authored.value, "-2");
            let surgeist_css::CssZIndexValue::Integer(surgeist_css::CssIntegerValue::Literal(
                literal,
            )) = value.value()
            else {
                panic!("captured stacking integer")
            };
            assert_eq!(literal.numeric().representation(), "-2");
            Some(value.as_css())
        }
        _ => None,
    };
    if let Some(css) = scalar_authored {
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "typed-value");
        assert_eq!(css, authored.value);
        return;
    }
    let migrated_authored = match (property, &value) {
        (
            surgeist_css::CssKnownProperty::BackgroundSize,
            surgeist_css::CssKnownPropertyValueRef::BackgroundSize(value),
        ) => {
            use surgeist_css::{CssBackgroundSize as Size, CssBackgroundSizeComponent as Part};
            assert_eq!(authored.value, "cover, 10px auto");
            assert!(matches!(value.sizes().sizes(), [
                Size::Cover,
                Size::Explicit {
                    width: Part::Length(length),
                    height: Some(Part::Auto),
                },
            ] if exact_dimension(length.literal_component(), "10", "px")));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::MaskSize,
            surgeist_css::CssKnownPropertyValueRef::MaskSize(value),
        ) => {
            assert_eq!(authored.value, "contain");
            assert!(matches!(
                value.sizes().sizes(),
                [surgeist_css::CssBackgroundSize::Contain]
            ));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BackgroundRepeat,
            surgeist_css::CssKnownPropertyValueRef::BackgroundRepeat(value),
        ) => {
            use surgeist_css::{CssBackgroundRepeat as Repeat, CssBackgroundRepeatStyle as Style};
            assert_eq!(authored.value, "repeat-x, no-repeat round");
            assert!(matches!(
                value.repeats().repeats(),
                [
                    Repeat::RepeatX,
                    Repeat::Axes {
                        x: Style::NoRepeat,
                        y: Style::Round
                    },
                ]
            ));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::MaskRepeat,
            surgeist_css::CssKnownPropertyValueRef::MaskRepeat(value),
        ) => {
            assert_eq!(authored.value, "repeat");
            assert!(matches!(
                value.repeats().repeats(),
                [surgeist_css::CssBackgroundRepeat::Axes {
                    x: surgeist_css::CssBackgroundRepeatStyle::Repeat,
                    y: surgeist_css::CssBackgroundRepeatStyle::Repeat,
                },]
            ));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BackgroundOrigin,
            surgeist_css::CssKnownPropertyValueRef::BackgroundOrigin(value),
        ) => {
            assert_eq!(authored.value, "content-box");
            assert_eq!(
                value.boxes().boxes(),
                [surgeist_css::CssBackgroundBox::ContentBox]
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BackgroundClip,
            surgeist_css::CssKnownPropertyValueRef::BackgroundClip(value),
        ) => {
            assert_eq!(authored.value, "padding-box");
            assert_eq!(
                value.boxes().boxes(),
                [surgeist_css::CssBackgroundBox::PaddingBox]
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BackgroundAttachment,
            surgeist_css::CssKnownPropertyValueRef::BackgroundAttachment(value),
        ) => {
            assert_eq!(authored.value, "fixed, local");
            assert_eq!(
                value.attachments().attachments(),
                [
                    surgeist_css::CssBackgroundAttachment::Fixed,
                    surgeist_css::CssBackgroundAttachment::Local,
                ]
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Transform,
            surgeist_css::CssKnownPropertyValueRef::Transform(value),
        ) => {
            use surgeist_css::*;
            let CssTransform::Functions(functions) = value.value() else {
                panic!("captured transform function list");
            };
            let [
                CssTransformFunction::Translate(translation),
                CssTransformFunction::Rotate(angle),
                CssTransformFunction::Scale(scale),
            ] = functions.functions()
            else {
                panic!("captured translate, rotate, scale order");
            };
            assert!(exact_dimension(
                translation.x().literal_component(),
                "10",
                "px"
            ));
            assert!(exact_dimension(
                translation.y().unwrap().literal_component(),
                "20",
                "px"
            ));
            assert!(
                matches!(angle, CssAngleOrZero::Angle(value) if value.literal().is_some_and(|literal| literal.numeric().representation() == "45" && literal.unit() == CssAngleUnit::Degrees))
            );
            assert!(exact_number(
                scale_number(scale.x()).literal_component(),
                "1.5"
            ));
            assert!(scale.y().is_none());
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Color,
            surgeist_css::CssKnownPropertyValueRef::Color(value),
        ) => {
            assert_captured_color(value.value(), authored.value);
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BackgroundColor,
            surgeist_css::CssKnownPropertyValueRef::BackgroundColor(value),
        ) => {
            assert_captured_color(value.value(), "transparent");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderColor,
            surgeist_css::CssKnownPropertyValueRef::BorderColor(value),
        ) => {
            let colors = value.value();
            assert_eq!(colors.kind(), surgeist_css::CssBoxSideKind::Physical);
            let [color] = colors.authored_values() else {
                panic!("captured one-value border-color shorthand");
            };
            assert_captured_color(color, "black");
            for color in colors.assigned_values() {
                assert_captured_color(color, "black");
            }
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderTopColor,
            surgeist_css::CssKnownPropertyValueRef::BorderTopColor(value),
        ) => {
            assert_captured_color(value.value(), "black");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderRightColor,
            surgeist_css::CssKnownPropertyValueRef::BorderRightColor(value),
        ) => {
            assert_captured_color(value.value(), "white");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderBottomColor,
            surgeist_css::CssKnownPropertyValueRef::BorderBottomColor(value),
        ) => {
            assert_captured_color(value.value(), "transparent");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderLeftColor,
            surgeist_css::CssKnownPropertyValueRef::BorderLeftColor(value),
        ) => {
            assert_captured_color(value.value(), "#fff");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::OutlineColor,
            surgeist_css::CssKnownPropertyValueRef::OutlineColor(value),
        ) => {
            assert_captured_color(
                value.value().color().expect("explicit outline color"),
                "black",
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::TextDecorationColor,
            surgeist_css::CssKnownPropertyValueRef::TextDecorationColor(value),
        ) => {
            assert_captured_color(value.value(), "black");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Background,
            surgeist_css::CssKnownPropertyValueRef::Background(value),
        ) => {
            let [layer] = value.background().layers() else {
                panic!("captured one-layer background");
            };
            assert!(layer.image().is_none());
            assert!(layer.position().is_none());
            assert!(layer.size().is_none());
            assert!(layer.repeat().is_none());
            assert!(layer.attachment().is_none());
            assert!(layer.boxes().is_none());
            assert_captured_color(layer.color().expect("background color"), "#fff");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Border,
            surgeist_css::CssKnownPropertyValueRef::Border(value),
        ) => {
            assert_captured_border(value.value(), "solid 2px #fff");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderTop,
            surgeist_css::CssKnownPropertyValueRef::BorderTop(value),
        ) => {
            assert_captured_border(value.value(), "black dotted");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderRight,
            surgeist_css::CssKnownPropertyValueRef::BorderRight(value),
        ) => {
            assert_captured_border(value.value(), "1px");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderBottom,
            surgeist_css::CssKnownPropertyValueRef::BorderBottom(value),
        ) => {
            assert_captured_border(value.value(), "#fff");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BorderLeft,
            surgeist_css::CssKnownPropertyValueRef::BorderLeft(value),
        ) => {
            assert_captured_border(value.value(), "dashed black");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Outline,
            surgeist_css::CssKnownPropertyValueRef::Outline(value),
        ) => {
            let outline = value.value();
            assert!(matches!(outline.width(), Some(CssOutlineWidth::Thick)));
            assert_eq!(outline.style(), Some(CssOutlineStyle::Dotted));
            assert_captured_color(
                outline
                    .color()
                    .expect("outline color")
                    .color()
                    .expect("explicit color"),
                "white",
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::TextDecoration,
            surgeist_css::CssKnownPropertyValueRef::TextDecoration(value),
        ) => {
            let decoration = value.value();
            assert!(matches!(
                decoration.line().expect("decoration line").components(),
                [CssTextDecorationLineComponent::Underline]
            ));
            assert_captured_color(decoration.color().expect("decoration color"), "white");
            assert_eq!(decoration.style(), Some(CssTextDecorationStyle::Dotted));
            let Some(CssTextDecorationThickness::Length(thickness)) = decoration.thickness() else {
                panic!("captured decoration length");
            };
            assert_captured_px(thickness.literal_component(), "3");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BoxShadow,
            surgeist_css::CssKnownPropertyValueRef::BoxShadow(value),
        ) => {
            let CssBoxShadow::Shadows(shadows) = value.value() else {
                panic!("captured box shadow list");
            };
            let [shadow] = shadows.shadows() else {
                panic!("captured one box shadow");
            };
            assert!(shadow.inset());
            assert_captured_px(shadow.offset_x().literal_component(), "1");
            assert_captured_px(shadow.offset_y().literal_component(), "2");
            assert_captured_px(shadow.blur_radius().expect("blur").literal_component(), "3");
            assert_captured_px(
                shadow.spread_radius().expect("spread").literal_component(),
                "4",
            );
            assert_captured_color(shadow.color().expect("shadow color"), "black");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Filter,
            surgeist_css::CssKnownPropertyValueRef::Filter(value),
        ) => {
            let CssFilter::Functions(functions) = value.value() else {
                panic!("captured filter function list");
            };
            let [
                CssFilterFunction::Blur(blur),
                CssFilterFunction::Opacity(amount),
            ] = functions.functions()
            else {
                panic!("captured ordered blur and opacity");
            };
            assert_captured_px(blur.length().literal_component(), "4");
            assert!(matches!(
                amount,
                CssFilterAmount::Percentage(number)
                    if exact_percentage(number.literal_component(), "50")
            ));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::BackdropFilter,
            surgeist_css::CssKnownPropertyValueRef::BackdropFilter(value),
        ) => {
            assert!(matches!(value.value(), CssFilter::None));
            Some(value.as_css())
        }
        _ => None,
    };
    if let Some(css) = migrated_authored {
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "typed-value");
        assert_eq!(css, authored.value);
        return;
    }
    match (property, &value) {
        (
            surgeist_css::CssKnownProperty::ListStyleType,
            surgeist_css::CssKnownPropertyValueRef::ListStyleType(value),
        ) if frozen.case_id == "catalog.property.baseline.property.list-style-type.boundary" => {
            assert_eq!(authored.id, property.stable_id());
            assert_eq!(authored.value_capability, "typed-value");
            assert_eq!(authored.value, "symbols(cyclic \"*\" \"+\")");
            assert_eq!(value.as_css(), authored.value);

            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "symbols(cyclic \"*\" \"+\")"
            );
            let surgeist_css::CssListStyleTypeValue::CounterStyle(style) = value.value() else {
                panic!("functional counter style")
            };
            assert!(style.named().is_none());
            return;
        }
        (
            surgeist_css::CssKnownProperty::ListStyle,
            surgeist_css::CssKnownPropertyValueRef::ListStyle(value),
        ) if frozen.case_id == "catalog.property.baseline.property.list-style.boundary" => {
            assert_eq!(authored.id, property.stable_id());
            assert_eq!(authored.value_capability, "typed-value");
            assert_eq!(authored.value, "inside outside");
            assert_eq!(value.as_css(), authored.value);
            assert_eq!(
                value.value().position(),
                Some(surgeist_css::CssListStylePosition::Inside)
            );
            let Some(surgeist_css::CssListStyleTypeValue::CounterStyle(style)) =
                value.value().style_type()
            else {
                panic!("custom style name after positional keyword")
            };
            assert_eq!(style.named().unwrap().as_str(), "outside");
            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "inside outside"
            );
            assert!(value.value().image().is_none());
            return;
        }
        // Box Alignment 3 §8.2: one authored gap value applies to both axes.
        // https://www.w3.org/TR/2026/WD-css-align-3-20260130/#gap-shorthand
        (
            surgeist_css::CssKnownProperty::Gap,
            surgeist_css::CssKnownPropertyValueRef::Gap(value),
        ) => {
            let gap = value.value();
            assert!(gap.authored_column().is_none());
            assert_eq!(gap.row(), gap.column());
            let surgeist_css::CssGapValue::LengthPercentage(length) = gap.row() else {
                panic!("captured gap is an exact numeric value")
            };
            assert_spacing_literal(
                length.literal_component().expect("captured gap literal"),
                "12px",
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::RowGap,
            surgeist_css::CssKnownPropertyValueRef::RowGap(value),
        ) => {
            assert_eq!(value.value(), &surgeist_css::CssGapValue::Normal);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::ColumnGap,
            surgeist_css::CssKnownPropertyValueRef::ColumnGap(value),
        ) => {
            let surgeist_css::CssGapValue::LengthPercentage(length) = value.value() else {
                panic!("captured column gap is an exact numeric value")
            };
            assert_spacing_literal(
                length
                    .literal_component()
                    .expect("captured column-gap literal"),
                "5%",
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::Width,
            surgeist_css::CssKnownPropertyValueRef::Width(value),
        ) if authored.value == "calc(100% - 12px)" => {
            let surgeist_css::CssSizeValue::BoxSize(surgeist_css::CssBoxSize::LengthPercentage(
                length,
            )) = value.value()
            else {
                panic!("frozen width calculation changed branch")
            };
            assert_captured_sum_calculation(
                length.calculation().expect("frozen width calculation"),
                "calc(100% - 12px)",
                ("100", true),
                ("12", false),
                true,
                frozen.input,
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }

        (
            surgeist_css::CssKnownProperty::Width,
            surgeist_css::CssKnownPropertyValueRef::Width(value),
        ) => {
            assert_preferred_size(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::Height,
            surgeist_css::CssKnownPropertyValueRef::Height(value),
        ) => {
            assert_preferred_size(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MinWidth,
            surgeist_css::CssKnownPropertyValueRef::MinWidth(value),
        ) => {
            assert_preferred_size(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MinHeight,
            surgeist_css::CssKnownPropertyValueRef::MinHeight(value),
        ) => {
            assert_preferred_size(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MaxWidth,
            surgeist_css::CssKnownPropertyValueRef::MaxWidth(value),
        ) => {
            match value.value().box_size() {
                Some(size) => assert_box_size(size, authored.value),
                None => assert_eq!(authored.value, "none"),
            }
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MaxHeight,
            surgeist_css::CssKnownPropertyValueRef::MaxHeight(value),
        ) => {
            match value.value().box_size() {
                Some(size) => assert_box_size(size, authored.value),
                None => assert_eq!(authored.value, "none"),
            }
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }

        (
            surgeist_css::CssKnownProperty::Left,
            surgeist_css::CssKnownPropertyValueRef::Left(value),
        ) if authored.value == "calc(3px + 4%)" => {
            assert_captured_sum_calculation(
                match value.value() {
                    surgeist_css::CssInsetValue::LengthPercentage(v) => v.calculation().unwrap(),
                    _ => panic!("captured inset calculation"),
                },
                "calc(3px + 4%)",
                ("3", false),
                ("4", true),
                false,
                frozen.input,
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }

        (
            surgeist_css::CssKnownProperty::MarginLeft,
            surgeist_css::CssKnownPropertyValueRef::MarginLeft(value),
        ) if authored.value == "calc(3px + 4%)" => {
            assert_captured_sum_calculation(
                value
                    .value()
                    .length_percentage()
                    .unwrap()
                    .calculation()
                    .unwrap(),
                "calc(3px + 4%)",
                ("3", false),
                ("4", true),
                false,
                frozen.input,
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }

        (
            surgeist_css::CssKnownProperty::PaddingBottom,
            surgeist_css::CssKnownPropertyValueRef::PaddingBottom(value),
        ) if authored.value == "calc(3px + 4%)" => {
            assert_captured_sum_calculation(
                value.value().length_percentage().calculation().unwrap(),
                "calc(3px + 4%)",
                ("3", false),
                ("4", true),
                false,
                frozen.input,
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }

        (
            surgeist_css::CssKnownProperty::BorderBottomLeftRadius,
            surgeist_css::CssKnownPropertyValueRef::BorderBottomLeftRadius(value),
        ) if authored.value == "calc(1px + 2%)" => {
            let current = value.value();
            assert_captured_sum_calculation(
                current.horizontal().calculation().unwrap(),
                "calc(1px + 2%)",
                ("1", false),
                ("2", true),
                false,
                frozen.input,
            );
            assert_captured_sum_calculation(
                current.vertical().calculation().unwrap(),
                "calc(1px + 2%)",
                ("1", false),
                ("2", true),
                false,
                frozen.input,
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::Padding,
            surgeist_css::CssKnownPropertyValueRef::Padding(value),
        ) if authored.value == "1px 2% calc(3px + 4%) 0" => {
            let current = value.value();
            assert_eq!(current.authored_values().len(), 4);
            let [top, right, bottom, left] = current.assigned_values();
            assert_spacing_literal(top.length_percentage().literal_component().unwrap(), "1px");
            assert_spacing_literal(right.length_percentage().literal_component().unwrap(), "2%");
            assert_spacing_literal(left.length_percentage().literal_component().unwrap(), "0");
            assert_captured_sum_calculation(
                bottom.length_percentage().calculation().unwrap(),
                "calc(3px + 4%)",
                ("3", false),
                ("4", true),
                false,
                frozen.input,
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }

        (
            surgeist_css::CssKnownProperty::Margin,
            surgeist_css::CssKnownPropertyValueRef::Margin(value),
        ) => {
            assert_eq!(authored.value, "auto 10px 5%");
            let current = value.value();
            assert_eq!(current.kind(), surgeist_css::CssBoxSideKind::Physical);
            assert_eq!(current.authored_values().len(), 3);
            let [top, right, bottom, left] = current.assigned_values();
            assert_margin(top, "auto");
            assert_margin(right, "10px");
            assert_margin(bottom, "5%");
            assert_margin(left, "10px");
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginTop,
            surgeist_css::CssKnownPropertyValueRef::MarginTop(value),
        ) => {
            assert_margin(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginRight,
            surgeist_css::CssKnownPropertyValueRef::MarginRight(value),
        ) => {
            assert_margin(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginBottom,
            surgeist_css::CssKnownPropertyValueRef::MarginBottom(value),
        ) => {
            assert_margin(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginLeft,
            surgeist_css::CssKnownPropertyValueRef::MarginLeft(value),
        ) => {
            assert_margin(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingTop,
            surgeist_css::CssKnownPropertyValueRef::PaddingTop(value),
        ) => {
            assert_padding(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingRight,
            surgeist_css::CssKnownPropertyValueRef::PaddingRight(value),
        ) => {
            assert_padding(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingBottom,
            surgeist_css::CssKnownPropertyValueRef::PaddingBottom(value),
        ) => {
            assert_padding(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingLeft,
            surgeist_css::CssKnownPropertyValueRef::PaddingLeft(value),
        ) => {
            assert_padding(value.value(), authored.value);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::LetterSpacing,
            surgeist_css::CssKnownPropertyValueRef::LetterSpacing(value),
        ) => {
            use surgeist_css::{
                CssComponentValueRef, CssTextSpacingAdjustment, CssValueOrigin, CssValueTokenRef,
            };
            assert_eq!(authored.id, property.stable_id());
            assert_eq!(authored.value_capability, "typed-value");
            assert_eq!(authored.value, "0.1em");
            assert_eq!(value.as_css(), "0.1em");

            let CssTextSpacingAdjustment::LengthPercentage(length) = value.value() else {
                panic!("exact current letter spacing")
            };
            let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
                length.literal_component().unwrap().view()
            else {
                panic!("exact authored dimension")
            };
            assert_eq!(number.representation(), "0.1");
            assert_eq!(unit, "em");
            assert!(matches!(length.origin(), CssValueOrigin::Parsed(_)));
            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "0.1em"
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontSize,
            surgeist_css::CssKnownPropertyValueRef::FontSize(value),
        ) => {
            assert_eq!(
                value.size(),
                &surgeist_css::CssFontSize::LengthPercentage(
                    surgeist_css::CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                        surgeist_css::CssComponentValue::try_token("16px").unwrap()
                    )
                    .unwrap()
                )
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::LineHeight,
            surgeist_css::CssKnownPropertyValueRef::LineHeight(value),
        ) => {
            assert_eq!(value.line_height(), &surgeist_css::CssLineHeight::Normal);
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontFamily,
            surgeist_css::CssKnownPropertyValueRef::FontFamily(value),
        ) => {
            assert_captured_font_families(value.families());
            assert_captured_font_metadata(
                "baseline.property.font-family",
                value.as_css(),
                authored,
                frozen.case_id,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontStyle,
            surgeist_css::CssKnownPropertyValueRef::FontStyle(value),
        ) => {
            assert_eq!(
                value.value(),
                &surgeist_css::CssFontStyle::Keyword(surgeist_css::CssFontStyleKeyword::Italic)
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontWeight,
            surgeist_css::CssKnownPropertyValueRef::FontWeight(value),
        ) => {
            let surgeist_css::CssFontWeight::Absolute(surgeist_css::CssAbsoluteFontWeight::Number(
                number,
            )) = value.value()
            else {
                panic!("{}: expected captured numeric font weight", frozen.case_id);
            };
            assert_eq!(
                number.serialize_specified(Default::default()).unwrap(),
                "725"
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::Font,
            surgeist_css::CssKnownPropertyValueRef::Font(value),
        ) => {
            let surgeist_css::CssFontValue::Explicit(font) = value.font() else {
                panic!("{}: expected captured explicit font", frozen.case_id);
            };
            assert_eq!(
                font.style(),
                Some(&surgeist_css::CssFontStyle::Keyword(
                    surgeist_css::CssFontStyleKeyword::Italic
                ))
            );
            assert_eq!(
                font.variant(),
                Some(surgeist_css::CssFontVariant::SmallCaps)
            );
            assert_eq!(
                font.weight(),
                Some(&surgeist_css::CssFontWeight::Absolute(
                    surgeist_css::CssAbsoluteFontWeight::Number(
                        surgeist_css::CssFontWeightNumber::try_from_component(
                            surgeist_css::CssComponentValue::try_number("700").unwrap()
                        )
                        .unwrap()
                    )
                ))
            );
            assert_eq!(
                font.stretch(),
                Some(surgeist_css::CssFontWidthKeyword::Condensed)
            );
            assert_eq!(
                font.size(),
                &surgeist_css::CssFontSize::LengthPercentage(
                    surgeist_css::CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                        surgeist_css::CssComponentValue::try_token("16px").unwrap()
                    )
                    .unwrap()
                )
            );
            assert_eq!(
                font.line_height(),
                Some(&surgeist_css::CssLineHeight::Normal)
            );
            assert_captured_font_families(font.families());
            assert_captured_font_metadata(
                "baseline.property.font",
                value.as_css(),
                authored,
                frozen.case_id,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontWidth,
            surgeist_css::CssKnownPropertyValueRef::FontWidth(value),
        ) => {
            assert_eq!(
                value.value(),
                &surgeist_css::CssFontWidth::Keyword(
                    surgeist_css::CssFontWidthKeyword::SemiCondensed
                )
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontVariant,
            surgeist_css::CssKnownPropertyValueRef::FontVariant(value),
        ) => {
            let surgeist_css::CssFontVariantValue::Values(values) = value.variant() else {
                panic!("{}: expected captured small-caps variant", frozen.case_id);
            };
            assert_eq!(
                values.caps(),
                Some(surgeist_css::CssFontVariantCaps::SmallCaps)
            );
            assert!(values.ligatures().is_none());
            assert!(values.alternates().is_none());
            assert!(values.numeric().is_none());
            assert!(values.east_asian().is_none());
            assert!(values.position().is_none());
            assert!(values.emoji().is_none());
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        (
            surgeist_css::CssKnownProperty::FontFeatureSettings,
            surgeist_css::CssKnownPropertyValueRef::FontFeatureSettings(value),
        ) => {
            let surgeist_css::CssAuthoredFontFeatureSettings::Features(list) = value.settings()
            else {
                panic!("{}: expected captured feature list", frozen.case_id);
            };
            let [kern, liga] = list.features() else {
                panic!("{}: expected two captured features", frozen.case_id);
            };
            assert_eq!(kern.tag().as_str(), "kern");
            assert!(matches!(
                kern.value(),
                surgeist_css::CssAuthoredFontFeatureValue::On
            ));
            assert_eq!(liga.tag().as_str(), "liga");
            assert!(
                matches!(liga.value(), surgeist_css::CssAuthoredFontFeatureValue::Index(index) if index.i32_value() == Some(0))
            );
            assert_captured_numeric_metadata(property.stable_id(), value.as_css(), authored);
            return;
        }
        _ => {}
    }
    // Animations 1 and Transitions 1: assert the typed semantics of each
    // ordinary timing declaration directly.
    use surgeist_css::{
        CssAnimationDirection as Direction, CssAnimationFillMode as Fill,
        CssAnimationIterationCount as Iteration, CssAnimationName as Name,
        CssAnimationPlayState as Play, CssEasing, CssEasingKeyword as Keyword, CssTimeUnit as Unit,
        CssTransitionProperty as TransitionPropertyValue,
    };
    let timing_authored = match (property, value) {
        (
            surgeist_css::CssKnownProperty::TransitionProperty,
            surgeist_css::CssKnownPropertyValueRef::TransitionProperty(value),
        ) => {
            let [
                TransitionPropertyValue::Custom(a),
                TransitionPropertyValue::Custom(b),
            ] = value.properties().properties()
            else {
                panic!("captured transition properties");
            };
            assert_eq!((a.as_str(), b.as_str()), ("opacity", "transform"));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::TransitionDuration,
            surgeist_css::CssKnownPropertyValueRef::TransitionDuration(value),
        ) => {
            assert!(
                matches!({ let values = value.durations().values(); if values.len() == 2 { (values[0].time().literal(),values[1].time().literal()) } else { (None,None) } }, (Some(a), Some(b))
                if a.numeric().representation() == "150" && a.unit() == Unit::Milliseconds
                    && b.numeric().representation() == "2" && b.unit() == Unit::Seconds)
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::TransitionDelay,
            surgeist_css::CssKnownPropertyValueRef::TransitionDelay(value),
        ) => {
            assert!(
                matches!({ let values = value.delays().values(); if values.len() == 1 { values[0].literal() } else { None } }, Some(a)
                if a.numeric().representation() == "20" && a.unit() == Unit::Milliseconds)
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::TransitionTimingFunction,
            surgeist_css::CssKnownPropertyValueRef::TransitionTimingFunction(value),
        ) => {
            let [
                CssEasing::Keyword(Keyword::EaseIn),
                CssEasing::CubicBezier(bezier),
            ] = value.timing_functions().values()
            else {
                panic!("captured transition easings");
            };
            assert!(exact_number(bezier.x1().value().literal_component(), "0.1"));
            assert!(exact_number(bezier.y1().literal_component(), "0.2"));
            assert!(exact_number(bezier.x2().value().literal_component(), "0.3"));
            assert!(exact_number(bezier.y2().literal_component(), "1"));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Transition,
            surgeist_css::CssKnownPropertyValueRef::Transition(value),
        ) => {
            let [first, second] = value.transitions().values() else {
                panic!("two captured transitions");
            };
            assert!(
                matches!(first.property(), Some(TransitionPropertyValue::Custom(name)) if name.as_str() == "opacity")
            );
            assert!(
                matches!(first.duration().and_then(|value| value.time().literal()), Some(t) if t.numeric().representation() == "150" && t.unit() == Unit::Milliseconds)
            );
            assert!(
                matches!(first.delay().and_then(|value| value.literal()), Some(t) if t.numeric().representation() == "20" && t.unit() == Unit::Milliseconds)
            );
            assert!(matches!(
                first.timing_function(),
                Some(CssEasing::Keyword(Keyword::EaseIn))
            ));
            assert!(
                matches!(second.property(), Some(TransitionPropertyValue::Custom(name)) if name.as_str() == "transform")
            );
            assert!(
                matches!(second.duration().and_then(|value| value.time().literal()), Some(t) if t.numeric().representation() == "2" && t.unit() == Unit::Seconds)
            );
            assert!(second.delay().is_none());
            assert!(matches!(
                second.timing_function(),
                Some(CssEasing::Keyword(Keyword::Linear))
            ));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationName,
            surgeist_css::CssKnownPropertyValueRef::AnimationName(value),
        ) => {
            assert!(
                matches!(value.names().names(), [Name::Custom(name), Name::None] if name.as_str() == "fade")
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationDuration,
            surgeist_css::CssKnownPropertyValueRef::AnimationDuration(value),
        ) => {
            assert!(
                matches!({ let values = value.durations().values(); if values.len() == 1 { values[0].time().literal() } else { None } }, Some(t) if t.numeric().representation() == "1" && t.unit() == Unit::Seconds)
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationDelay,
            surgeist_css::CssKnownPropertyValueRef::AnimationDelay(value),
        ) => {
            assert!(
                matches!({ let values = value.delays().values(); if values.len() == 1 { values[0].literal() } else { None } }, Some(t) if t.numeric().representation() == "200" && t.unit() == Unit::Milliseconds)
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationTimingFunction,
            surgeist_css::CssKnownPropertyValueRef::AnimationTimingFunction(value),
        ) => {
            assert!(matches!(
                value.timing_functions().values(),
                [CssEasing::Keyword(Keyword::EaseOut)]
            ));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationIterationCount,
            surgeist_css::CssKnownPropertyValueRef::AnimationIterationCount(value),
        ) => {
            assert!(
                matches!(value.iteration_counts().values(), [Iteration::Number(n), Iteration::Infinite] if exact_number(n.literal_component(), "2"))
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationDirection,
            surgeist_css::CssKnownPropertyValueRef::AnimationDirection(value),
        ) => {
            assert_eq!(value.directions().directions(), &[Direction::Alternate]);
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationFillMode,
            surgeist_css::CssKnownPropertyValueRef::AnimationFillMode(value),
        ) => {
            assert_eq!(value.fill_modes().modes(), &[Fill::Both]);
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::AnimationPlayState,
            surgeist_css::CssKnownPropertyValueRef::AnimationPlayState(value),
        ) => {
            assert_eq!(value.play_states().states(), &[Play::Running, Play::Paused]);
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Animation,
            surgeist_css::CssKnownPropertyValueRef::Animation(value),
        ) => {
            let [item] = value.animations().values() else {
                panic!("one captured animation");
            };
            assert!(matches!(item.name(), Some(Name::Custom(name)) if name.as_str() == "fade"));
            assert!(
                matches!(item.duration().and_then(|value| value.time().literal()), Some(t) if t.numeric().representation() == "1" && t.unit() == Unit::Seconds)
            );
            assert!(
                matches!(item.delay().and_then(|value| value.literal()), Some(t) if t.numeric().representation() == "200" && t.unit() == Unit::Milliseconds)
            );
            assert!(matches!(
                item.timing_function(),
                Some(CssEasing::Keyword(Keyword::EaseIn))
            ));
            assert!(
                matches!(item.iteration_count(), Some(Iteration::Number(n)) if exact_number(n.literal_component(), "3"))
            );
            assert_eq!(item.direction(), Some(Direction::Alternate));
            assert_eq!(item.fill_mode(), Some(Fill::Both));
            assert_eq!(item.play_state(), Some(Play::Running));
            Some(value.as_css())
        }
        _ => None,
    };
    if let Some(css) = timing_authored {
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "typed-value");
        assert_eq!(css, authored.value);
        return;
    }
    // Grid 2: assert each Grid value against the authored model's branch and
    // independently fixed constituents.
    use surgeist_css::{
        CssGridAutoFlowAxis, CssGridAutoRepeatKind as RepeatKind,
        CssGridAutoTrackComponent as AutoTrack, CssGridGeneralTrackComponent as GeneralTrack,
        CssGridLine as GridLine, CssGridTemplateAreaCell as AreaCell,
        CssGridTemplateAreas as Areas, CssGridTrackBreadthKind as BreadthKind,
        CssGridTrackRepeatComponent as RepeatMember, CssGridTrackSizeKind as SizeKind,
    };
    let grid_authored = match (property, value) {
        (
            surgeist_css::CssKnownProperty::GridTemplateRows,
            surgeist_css::CssKnownPropertyValueRef::GridTemplateRows(value),
        ) => {
            let [
                GeneralTrack::LineNames(names),
                GeneralTrack::TrackSize(length),
                GeneralTrack::TrackSize(flex),
            ] = value.value().general_list().unwrap().components()
            else {
                panic!("captured row tracks")
            };
            assert_eq!(names.names()[0].ident().as_str(), "top");
            assert_eq!(
                length
                    .breadth()
                    .unwrap()
                    .length_percentage()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "100px"
            );
            assert_eq!(
                flex.breadth()
                    .unwrap()
                    .flex()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "1fr"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridTemplateColumns,
            surgeist_css::CssKnownPropertyValueRef::GridTemplateColumns(value),
        ) => {
            assert_captured_grid_columns(value.value());
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridTemplateAreas,
            surgeist_css::CssKnownPropertyValueRef::GridTemplateAreas(value),
        ) => {
            let Areas::Rows(rows) = value.value() else {
                panic!("captured area matrix")
            };
            let [top, bottom] = rows.rows() else {
                panic!("two captured area rows")
            };
            for (row, expected) in [(top, ["header", "header"]), (bottom, ["nav", "main"])] {
                let [AreaCell::Named(first), AreaCell::Named(second)] = row.cells() else {
                    panic!("two named cells")
                };
                assert_eq!([first.as_str(), second.as_str()], expected);
            }
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridTemplate,
            surgeist_css::CssKnownPropertyValueRef::GridTemplate(value),
        ) => {
            let rows = value.value().rows().expect("captured template rows");
            let [
                GeneralTrack::TrackSize(length),
                GeneralTrack::TrackSize(flex),
            ] = rows.general_list().unwrap().components()
            else {
                panic!("two row tracks")
            };
            assert_eq!(
                length
                    .breadth()
                    .unwrap()
                    .length_percentage()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "100px"
            );
            assert_eq!(
                flex.breadth()
                    .unwrap()
                    .flex()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "1fr"
            );
            assert_captured_grid_columns(value.value().columns().unwrap());
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridAutoRows,
            surgeist_css::CssKnownPropertyValueRef::GridAutoRows(value),
        ) => {
            let [size] = value.value().sizes() else {
                panic!("one implicit row track")
            };
            let (min, max) = size.minmax().expect("captured minmax");
            assert_eq!(
                min.length_percentage()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "10px"
            );
            assert_eq!(max.kind(), BreadthKind::Auto);
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridAutoColumns,
            surgeist_css::CssKnownPropertyValueRef::GridAutoColumns(value),
        ) => {
            let [size] = value.value().sizes() else {
                panic!("one implicit column track")
            };
            assert_eq!(size.kind(), SizeKind::FitContent);
            assert_eq!(
                size.fit_content()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "20%"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridAutoFlow,
            surgeist_css::CssKnownPropertyValueRef::GridAutoFlow(value),
        ) => {
            let surgeist_css::CssGridAutoFlow::ExplicitAxis(flow) = value.value() else {
                panic!("explicit flow")
            };
            assert_eq!(flow.axis(), CssGridAutoFlowAxis::Column);
            assert!(flow.dense());
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridRowStart,
            surgeist_css::CssKnownPropertyValueRef::GridRowStart(value),
        ) => {
            let GridLine::Span(span) = value.value() else {
                panic!("row start span")
            };
            assert!(span.integer().is_some());
            assert_eq!(span.name().unwrap().ident().as_str(), "main");
            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "span 2 main"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridRowEnd,
            surgeist_css::CssKnownPropertyValueRef::GridRowEnd(value),
        ) => {
            assert!(matches!(value.value(), GridLine::Auto));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridColumnStart,
            surgeist_css::CssKnownPropertyValueRef::GridColumnStart(value),
        ) => {
            let GridLine::Name(name) = value.value() else {
                panic!("named column start")
            };
            assert_eq!(name.ident().as_str(), "nav");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridColumnEnd,
            surgeist_css::CssKnownPropertyValueRef::GridColumnEnd(value),
        ) => {
            let GridLine::Indexed(index) = value.value() else {
                panic!("indexed column end")
            };
            assert!(index.name().is_none());
            assert_eq!(
                value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "4"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridRow,
            surgeist_css::CssKnownPropertyValueRef::GridRow(value),
        ) => {
            let range = value.value();
            assert!(matches!(range.start(), GridLine::Indexed(_)));
            assert!(matches!(range.authored_end(), Some(GridLine::Span(_))));
            assert_eq!(
                range.serialize_specified(Default::default()).unwrap(),
                "1 / span 2"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridColumn,
            surgeist_css::CssKnownPropertyValueRef::GridColumn(value),
        ) => {
            let range = value.value();
            let GridLine::Name(start) = range.start() else {
                panic!("named column start")
            };
            let Some(GridLine::Name(end)) = range.authored_end() else {
                panic!("named column end")
            };
            assert_eq!(
                (start.ident().as_str(), end.ident().as_str()),
                ("nav", "main")
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridArea,
            surgeist_css::CssKnownPropertyValueRef::GridArea(value),
        ) => {
            let area = value.value();
            let GridLine::Name(start) = area.row_start() else {
                panic!("named area row")
            };
            assert_eq!(start.ident().as_str(), "header");
            assert!(matches!(
                area.authored_column_start(),
                Some(GridLine::Indexed(_))
            ));
            assert!(matches!(area.authored_row_end(), Some(GridLine::Span(_))));
            let Some(GridLine::Name(end)) = area.authored_column_end() else {
                panic!("named area column")
            };
            assert_eq!(end.ident().as_str(), "main");
            assert_eq!(
                area.serialize_specified(Default::default()).unwrap(),
                "header / 1 / span 2 / main"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Grid,
            surgeist_css::CssKnownPropertyValueRef::Grid(value),
        ) => {
            let grid = value.value();
            assert!(grid.template_value().is_none());
            let flow = grid.auto_flow().unwrap();
            assert_eq!(flow.axis(), CssGridAutoFlowAxis::Row);
            assert!(flow.dense());
            let [implicit] = grid.auto_tracks().unwrap().sizes() else {
                panic!("one implicit track")
            };
            assert_eq!(
                implicit
                    .breadth()
                    .unwrap()
                    .length_percentage()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "12px"
            );
            let [AutoTrack::AutoRepeat(repeat)] = grid
                .explicit_tracks()
                .unwrap()
                .auto_list()
                .unwrap()
                .components()
            else {
                panic!("one explicit repeat")
            };
            assert_eq!(repeat.kind(), RepeatKind::AutoFit);
            let [RepeatMember::TrackSize(size)] = repeat.content().components() else {
                panic!("one repeated track")
            };
            assert_eq!(
                size.breadth()
                    .unwrap()
                    .flex()
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                "1fr"
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::ClipPath,
            surgeist_css::CssKnownPropertyValueRef::ClipPath(value),
        ) => {
            assert_eq!(authored.value, "circle(50% at center)");
            let surgeist_css::CssClipPath::BasicShape(clip_shape) = value.value() else {
                panic!("captured typed circle");
            };
            let surgeist_css::CssBasicShape::Circle(circle) = clip_shape.shape() else {
                panic!("captured typed circle");
            };
            let surgeist_css::CssCircleRadius::LengthPercentage(radius) = circle.radius() else {
                panic!("captured percentage radius");
            };
            assert!(exact_percentage(radius.literal_component(), "50"));
            let surgeist_css::CssPositionRef::Cartesian(position) =
                circle.position().expect("captured center position").view()
            else {
                panic!("captured Cartesian center")
            };
            assert!(matches!(
                position.horizontal(),
                surgeist_css::CssHorizontalPosition::Center
            ));
            assert!(matches!(
                position.vertical(),
                surgeist_css::CssVerticalPosition::Center
            ));
            Some(value.as_css())
        }
        _ => None,
    };
    if let Some(css) = grid_authored {
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "typed-value");
        assert_eq!(css, authored.value);
        return;
    }
    let position_authored = match (property, value) {
        (
            surgeist_css::CssKnownProperty::BackgroundPosition,
            surgeist_css::CssKnownPropertyValueRef::BackgroundPosition(value),
        ) => {
            use surgeist_css::{
                CssHorizontalPosition as Horizontal, CssVerticalPosition as Vertical,
            };
            let [position] = value.positions().positions() else {
                panic!("captured one background-position layer")
            };
            assert!(
                matches!(position.horizontal(), Horizontal::LeftOffset(offset) if exact_dimension(offset.literal_component(), "10", "px"))
            );
            assert!(
                matches!(position.vertical(), Vertical::TopOffset(offset) if exact_percentage(offset.literal_component(), "20"))
            );
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::MaskPosition,
            surgeist_css::CssKnownPropertyValueRef::MaskPosition(value),
        ) => {
            use surgeist_css::{
                CssHorizontalPosition as Horizontal, CssVerticalPosition as Vertical,
            };
            let [position] = value.positions().positions() else {
                panic!("captured one mask-position layer")
            };
            assert!(matches!(position.horizontal(), Horizontal::Center));
            assert!(matches!(position.vertical(), Vertical::Center));
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::TransformOrigin,
            surgeist_css::CssKnownPropertyValueRef::TransformOrigin(value),
        ) => {
            use surgeist_css::{
                CssHorizontalPosition as Horizontal, CssVerticalPosition as Vertical,
            };
            let origin = value.origin();
            assert!(matches!(origin.horizontal(), Horizontal::Center));
            assert!(matches!(origin.vertical(), Vertical::Top));
            assert!(origin.z().is_none());
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::Mask,
            surgeist_css::CssKnownPropertyValueRef::Mask(value),
        ) => {
            use surgeist_css::{
                CssBackgroundRepeat, CssBackgroundRepeatStyle, CssBackgroundSize,
                CssHorizontalPosition as Horizontal, CssImageValue, CssMaskLayer, CssMaskList,
                CssPhysicalPosition, CssUrl, CssVerticalPosition as Vertical,
            };
            let expected = CssMaskList::try_new(vec![
                CssMaskLayer::try_new(
                    Some(CssImageValue::Url(CssUrl::new("mask.png"))),
                    Some(
                        CssPhysicalPosition::try_new(Horizontal::Center, Vertical::Center).unwrap(),
                    ),
                    Some(CssBackgroundSize::Contain),
                    Some(CssBackgroundRepeat::Axes {
                        x: CssBackgroundRepeatStyle::NoRepeat,
                        y: CssBackgroundRepeatStyle::NoRepeat,
                    }),
                    None,
                    None,
                    None,
                )
                .unwrap(),
            ])
            .unwrap();
            assert_eq!(value.value(), &expected);
            let [layer] = value.value().layers() else {
                panic!("one captured mask layer")
            };
            let position = layer.position().expect("captured center position");
            assert!(matches!(position.horizontal(), Horizontal::Center));
            assert!(matches!(position.vertical(), Vertical::Center));
            Some(value.as_css())
        }
        _ => None,
    };
    if let Some(css) = position_authored {
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "typed-value");
        assert_eq!(css, authored.value);
        return;
    }
    // Images 3 and Backgrounds 3: compare each authored image list with the
    // authored image model explicitly.
    let image_authored = match (property, value) {
        (
            surgeist_css::CssKnownProperty::BackgroundImage,
            surgeist_css::CssKnownPropertyValueRef::BackgroundImage(value),
        ) => Some((value.as_css(), value.images())),
        (
            surgeist_css::CssKnownProperty::MaskImage,
            surgeist_css::CssKnownPropertyValueRef::MaskImage(value),
        ) => Some((value.as_css(), value.images())),
        _ => None,
    };
    if let Some((css, images)) = image_authored {
        use surgeist_css::CssImageValue;
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "typed-value");
        assert_eq!(css, authored.value);
        match (property, authored.value) {
            (_, "url(\"\")") => assert!(
                matches!(images.images(), [CssImageValue::Url(url)] if url.as_str().is_empty())
            ),
            (surgeist_css::CssKnownProperty::BackgroundImage, "url(\"hero.png\"), none") => {
                assert!(
                    matches!(images.images(), [CssImageValue::Url(url), CssImageValue::None]
                    if url.as_str() == "hero.png")
                )
            }
            (surgeist_css::CssKnownProperty::MaskImage, "url(mask.png), none") => assert!(
                matches!(images.images(), [CssImageValue::Url(url), CssImageValue::None]
                    if url.as_str() == "mask.png")
            ),
            _ => panic!("unexpected captured image-list witness: {}", frozen.case_id),
        }
        return;
    }
    // Each arm maps its row's authored value to the typed value its property
    // grammar defines.
    use surgeist_css::*;
    match (property, value) {
        (CssKnownProperty::Display, CssKnownPropertyValueRef::Display(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "block"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed,
                &CssDisplayValue::OutsideInside {
                    outside: CssDisplayOutside::Block,
                    inside: CssDisplayInside::Flow
                }
            );
        }
        (CssKnownProperty::BoxSizing, CssKnownPropertyValueRef::BoxSizing(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "border-box"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "border-box" => CssBoxSizing::BorderBox,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Position, CssKnownPropertyValueRef::Position(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "sticky"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "sticky" => CssLayoutPosition::Sticky,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Direction, CssKnownPropertyValueRef::Direction(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "rtl"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "rtl" => CssDirection::Rtl,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Overflow, CssKnownPropertyValueRef::Overflow(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "hidden scroll"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.x(), CssOverflow::Hidden);
            assert_eq!(typed.authored_y(), Some(CssOverflow::Scroll));
        }
        (CssKnownProperty::OverflowX, CssKnownPropertyValueRef::OverflowX(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "clip"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "clip" => CssOverflow::Clip,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::OverflowY, CssKnownPropertyValueRef::OverflowY(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "visible"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "visible" => CssOverflow::Visible,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::FlexDirection, CssKnownPropertyValueRef::FlexDirection(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "column-reverse"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "column-reverse" => CssFlexDirection::ColumnReverse,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::FlexWrap, CssKnownPropertyValueRef::FlexWrap(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "wrap-reverse"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "wrap-reverse" => CssFlexWrap::WrapReverse,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Float, CssKnownPropertyValueRef::Float(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "left"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "left" => CssFloat::Left,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Clear, CssKnownPropertyValueRef::Clear(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "both"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "both" => CssClear::Both,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::AlignContent, CssKnownPropertyValueRef::AlignContent(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "space-between"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.value(), CssAlignmentValue::SpaceBetween);
        }
        (CssKnownProperty::JustifyContent, CssKnownPropertyValueRef::JustifyContent(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "safe center"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.value(),
                CssAlignmentValue::Position {
                    overflow: Some(CssOverflowPosition::Safe),
                    position: CssAlignmentPosition::Center
                }
            );
        }
        (CssKnownProperty::AlignItems, CssKnownPropertyValueRef::AlignItems(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "first baseline"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.value(),
                CssAlignmentValue::Baseline(CssBaselinePosition::First)
            );
        }
        (CssKnownProperty::AlignSelf, CssKnownPropertyValueRef::AlignSelf(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "safe flex-end"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.value(),
                CssAlignmentValue::Position {
                    overflow: Some(CssOverflowPosition::Safe),
                    position: CssAlignmentPosition::FlexEnd
                }
            );
        }
        (CssKnownProperty::JustifyItems, CssKnownPropertyValueRef::JustifyItems(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "stretch"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.value(), CssAlignmentValue::Stretch);
        }
        (CssKnownProperty::JustifySelf, CssKnownPropertyValueRef::JustifySelf(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "center"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.value(),
                CssAlignmentValue::Position {
                    overflow: None,
                    position: CssAlignmentPosition::Center
                }
            );
        }
        (CssKnownProperty::PlaceContent, CssKnownPropertyValueRef::PlaceContent(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "center end"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.align().value(),
                CssAlignmentValue::Position {
                    overflow: None,
                    position: CssAlignmentPosition::Center
                }
            );
            assert_eq!(
                typed.justify().value(),
                CssAlignmentValue::Position {
                    overflow: None,
                    position: CssAlignmentPosition::End
                }
            );
        }
        (CssKnownProperty::PlaceItems, CssKnownPropertyValueRef::PlaceItems(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "stretch"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.align().value(), CssAlignmentValue::Stretch);
            assert_eq!(typed.justify().value(), CssAlignmentValue::Stretch);
        }
        (CssKnownProperty::PlaceSelf, CssKnownPropertyValueRef::PlaceSelf(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "end center"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.align().value(),
                CssAlignmentValue::Position {
                    overflow: None,
                    position: CssAlignmentPosition::End
                }
            );
            assert_eq!(
                typed.justify().value(),
                CssAlignmentValue::Position {
                    overflow: None,
                    position: CssAlignmentPosition::Center
                }
            );
        }
        (CssKnownProperty::Visibility, CssKnownPropertyValueRef::Visibility(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "collapse"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "collapse" => CssVisibility::Collapse,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Content, CssKnownPropertyValueRef::Content(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "\"Chapter \"" | "\"x\""),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssContentValue::Generated(generated) = typed else {
                panic!("captured generated string");
            };
            assert!(generated.alternative().is_none());
            let [CssContentValueItem::String(text)] = generated.items() else {
                panic!("one captured string");
            };
            // Content 3 <string>: the item is the string's decoded value.
            let decoded = match authored.value {
                "\"Chapter \"" => "Chapter ",
                "\"x\"" => "x",
                _ => panic!("unknown authored content string"),
            };
            assert_eq!(text.as_str(), decoded);
        }
        (
            CssKnownProperty::ContentVisibility,
            CssKnownPropertyValueRef::ContentVisibility(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "auto"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "auto" => CssContentVisibility::Auto,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::ListStyleType, CssKnownPropertyValueRef::ListStyleType(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "square"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssListStyleTypeValue::CounterStyle(style) = typed else {
                panic!("captured named style");
            };
            assert_eq!(style.named().unwrap().as_str(), "square");
        }
        (
            CssKnownProperty::ListStylePosition,
            CssKnownPropertyValueRef::ListStylePosition(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "inside"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "inside" => CssListStylePosition::Inside,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::ListStyleImage, CssKnownPropertyValueRef::ListStyleImage(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "url(marker.svg)"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert!(matches!(typed, CssImageValue::Url(url)
                if url.as_str() == "marker.svg"
                    && url.function() == surgeist_css::CssUrlFunction::Url
                    && url.modifiers().is_empty()));
        }
        (CssKnownProperty::ListStyle, CssKnownPropertyValueRef::ListStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "url(marker.svg) inside square"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.position(), Some(CssListStylePosition::Inside));
            assert!(
                matches!(typed.style_type(), Some(CssListStyleTypeValue::CounterStyle(style)) if style.named().unwrap().as_str() == "square")
            );
            assert!(matches!(typed.image(), Some(CssImageValue::Url(url))
                    if url.as_str() == "marker.svg"
                        && url.function() == surgeist_css::CssUrlFunction::Url
                        && url.modifiers().is_empty()));
        }
        (CssKnownProperty::CounterReset, CssKnownPropertyValueRef::CounterReset(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "section 2"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let [change] = typed.changes().unwrap() else {
                panic!("one captured counter");
            };
            assert_eq!(change.name().as_str(), "section");
            let Some(CssIntegerValue::Literal(integer)) = change.value() else {
                panic!("captured explicit integer");
            };
            assert_eq!(integer.numeric().representation(), "2");
        }
        (CssKnownProperty::CounterIncrement, CssKnownPropertyValueRef::CounterIncrement(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "section 1"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let [change] = typed.changes().unwrap() else {
                panic!("one captured counter");
            };
            assert_eq!(change.name().as_str(), "section");
            let Some(CssIntegerValue::Literal(integer)) = change.value() else {
                panic!("captured explicit integer");
            };
            assert_eq!(integer.numeric().representation(), "1");
        }
        (CssKnownProperty::CounterSet, CssKnownPropertyValueRef::CounterSet(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "section 3"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let [change] = typed.changes().unwrap() else {
                panic!("one captured counter");
            };
            assert_eq!(change.name().as_str(), "section");
            let Some(CssIntegerValue::Literal(integer)) = change.value() else {
                panic!("captured explicit integer");
            };
            assert_eq!(integer.numeric().representation(), "3");
        }
        (CssKnownProperty::FlexBasis, CssKnownPropertyValueRef::FlexBasis(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "10rem"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_basis(typed);
        }
        (CssKnownProperty::WritingMode, CssKnownPropertyValueRef::WritingMode(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "vertical-rl"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "vertical-rl" => CssWritingMode::VerticalRl,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::TextAlign, CssKnownPropertyValueRef::TextAlign(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "start"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed,
                &CssTextAlignValue::Alignment(CssTextAlignAllValue::Keyword(CssTextAlign::Start))
            );
        }
        (CssKnownProperty::TextAlignLast, CssKnownPropertyValueRef::TextAlignLast(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "justify"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed,
                &CssTextAlignLastValue::Keyword(CssTextAlign::Justify)
            );
        }
        (CssKnownProperty::TextIndent, CssKnownPropertyValueRef::TextIndent(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1rem hanging each-line"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert!(typed.hanging());
            assert!(typed.each_line());
            assert!(exact_dimension(
                typed.length().literal_component(),
                "1",
                "rem"
            ));
        }
        (CssKnownProperty::VerticalAlign, CssKnownPropertyValueRef::VerticalAlign(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "super"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "super" => CssVerticalAlign::Super,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::TextWrap, CssKnownPropertyValueRef::TextWrap(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "balance"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "balance" => CssTextWrap::try_new(None, Some(CssTextWrapStyle::Balance)).unwrap(),
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::WhiteSpace, CssKnownPropertyValueRef::WhiteSpace(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "pre-wrap"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "pre-wrap" => CssWhiteSpace::from_keyword(CssWhiteSpaceKeyword::PreWrap),
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::WordBreak, CssKnownPropertyValueRef::WordBreak(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "keep-all"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "keep-all" => CssWordBreak::KeepAll,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::OverflowWrap, CssKnownPropertyValueRef::OverflowWrap(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "anywhere"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "anywhere" => CssOverflowWrap::Anywhere,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::TextOverflow, CssKnownPropertyValueRef::TextOverflow(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "ellipsis"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "ellipsis" => CssTextOverflow::Ellipsis,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (
            CssKnownProperty::TextDecorationLine,
            CssKnownPropertyValueRef::TextDecorationLine(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "underline overline"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert!(!typed.is_none());
            assert_eq!(
                typed.components(),
                &[
                    CssTextDecorationLineComponent::Underline,
                    CssTextDecorationLineComponent::Overline
                ]
            );
        }
        (
            CssKnownProperty::TextDecorationStyle,
            CssKnownPropertyValueRef::TextDecorationStyle(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "wavy"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "wavy" => CssTextDecorationStyle::Wavy,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (
            CssKnownProperty::TextDecorationThickness,
            CssKnownPropertyValueRef::TextDecorationThickness(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "2px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssTextDecorationThickness::Length(length) = typed else {
                panic!("captured thickness");
            };
            assert_captured_px(length.literal_component(), "2");
        }
        (CssKnownProperty::TextTransform, CssKnownPropertyValueRef::TextTransform(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "uppercase"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "uppercase" => CssTextTransform::Transforms(
                    CssTextTransformSet::try_new(
                        Some(CssTextTransformCase::Uppercase),
                        false,
                        false,
                    )
                    .unwrap(),
                ),
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Inset, CssKnownPropertyValueRef::Inset(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "auto 10px 5%"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.kind(), CssBoxSideKind::Physical);
            assert_eq!(typed.authored_values().len(), 3);
            let [a, b, c, d] = typed.assigned_values();
            assert_eq!(a, &CssInsetValue::Auto);
            assert_archive_inset(b, "10", Some("px"));
            assert_archive_inset(c, "5", Some("%"));
            assert_eq!(b, d);
        }
        (CssKnownProperty::Top, CssKnownPropertyValueRef::Top(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "auto"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed, &CssInsetValue::Auto);
        }
        (CssKnownProperty::Right, CssKnownPropertyValueRef::Right(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "10px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_inset(typed, "10", Some("px"));
        }
        (CssKnownProperty::Bottom, CssKnownPropertyValueRef::Bottom(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "5%"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_inset(typed, "5", Some("%"));
        }
        (
            CssKnownProperty::BoxDecorationBreak,
            CssKnownPropertyValueRef::BoxDecorationBreak(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "clone"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "clone" => CssBoxDecorationBreak::Clone,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderWidth, CssKnownPropertyValueRef::BorderWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1px 2px 3px 4px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.authored_values().len(), 4);
            for (width, expected) in typed
                .assigned_values()
                .into_iter()
                .zip(["1", "2", "3", "4"])
            {
                assert_archive_width(width, expected);
            }
        }
        (CssKnownProperty::BorderTopWidth, CssKnownPropertyValueRef::BorderTopWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_width(typed, "1");
        }
        (CssKnownProperty::BorderRightWidth, CssKnownPropertyValueRef::BorderRightWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "2px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_width(typed, "2");
        }
        (
            CssKnownProperty::BorderBottomWidth,
            CssKnownPropertyValueRef::BorderBottomWidth(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "3px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_width(typed, "3");
        }
        (CssKnownProperty::BorderLeftWidth, CssKnownPropertyValueRef::BorderLeftWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "4px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_width(typed, "4");
        }
        (CssKnownProperty::BorderStyle, CssKnownPropertyValueRef::BorderStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "none hidden dotted dashed"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(
                typed.authored_values(),
                &[
                    CssBorderStyle::None,
                    CssBorderStyle::Hidden,
                    CssBorderStyle::Dotted,
                    CssBorderStyle::Dashed
                ]
            );
            assert_eq!(
                typed.assigned_values(),
                [
                    &CssBorderStyle::None,
                    &CssBorderStyle::Hidden,
                    &CssBorderStyle::Dotted,
                    &CssBorderStyle::Dashed
                ]
            );
        }
        (CssKnownProperty::BorderTopStyle, CssKnownPropertyValueRef::BorderTopStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "solid"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "solid" => CssBorderStyle::Solid,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderRightStyle, CssKnownPropertyValueRef::BorderRightStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "double"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "double" => CssBorderStyle::Double,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (
            CssKnownProperty::BorderBottomStyle,
            CssKnownPropertyValueRef::BorderBottomStyle(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "ridge"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "ridge" => CssBorderStyle::Ridge,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderLeftStyle, CssKnownPropertyValueRef::BorderLeftStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "outset"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "outset" => CssBorderStyle::Outset,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderRadius, CssKnownPropertyValueRef::BorderRadius(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1px 2px 3px / 4px 5px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed.horizontal_values().len(), 3);
            assert_eq!(typed.authored_vertical_values().unwrap().len(), 2);
            for (component, expected) in typed.horizontal_values().iter().zip(["1", "2", "3"]) {
                assert_archive_literal(
                    component.literal_component().unwrap(),
                    expected,
                    Some("px"),
                );
            }
            for (component, expected) in typed
                .authored_vertical_values()
                .unwrap()
                .iter()
                .zip(["4", "5"])
            {
                assert_archive_literal(
                    component.literal_component().unwrap(),
                    expected,
                    Some("px"),
                );
            }
            for (corner, (h, v)) in [
                typed.top_left(),
                typed.top_right(),
                typed.bottom_right(),
                typed.bottom_left(),
            ]
            .into_iter()
            .zip([("1", "4"), ("2", "5"), ("3", "4"), ("2", "5")])
            {
                assert_archive_literal(
                    corner.horizontal().literal_component().unwrap(),
                    h,
                    Some("px"),
                );
                assert_archive_literal(
                    corner.vertical().literal_component().unwrap(),
                    v,
                    Some("px"),
                );
            }
        }
        (
            CssKnownProperty::BorderTopLeftRadius,
            CssKnownPropertyValueRef::BorderTopLeftRadius(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "4px 10%"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_literal(
                typed.horizontal().literal_component().unwrap(),
                "4",
                Some("px"),
            );
            assert_archive_literal(
                typed
                    .authored_vertical()
                    .unwrap()
                    .literal_component()
                    .unwrap(),
                "10",
                Some("%"),
            );
        }
        (
            CssKnownProperty::BorderTopRightRadius,
            CssKnownPropertyValueRef::BorderTopRightRadius(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_literal(
                typed.horizontal().literal_component().unwrap(),
                "1",
                Some("px"),
            );
            assert!(typed.authored_vertical().is_none());
            assert_eq!(typed.horizontal(), typed.vertical());
        }
        (
            CssKnownProperty::BorderBottomRightRadius,
            CssKnownPropertyValueRef::BorderBottomRightRadius(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "10%"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_archive_literal(
                typed.horizontal().literal_component().unwrap(),
                "10",
                Some("%"),
            );
            assert!(typed.authored_vertical().is_none());
            assert_eq!(typed.horizontal(), typed.vertical());
        }
        (CssKnownProperty::FlexGrow, CssKnownPropertyValueRef::FlexGrow(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "2"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.factor();
            assert_archive_literal(typed.literal_component().unwrap(), "2", None);
        }
        (CssKnownProperty::FlexShrink, CssKnownPropertyValueRef::FlexShrink(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "0"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.factor();
            assert_archive_literal(typed.literal_component().unwrap(), "0", None);
        }
        (CssKnownProperty::Flex, CssKnownPropertyValueRef::Flex(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "2 0 10rem"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssFlexValue::Components(components) = typed else {
                panic!("captured flex components");
            };
            assert_archive_literal(
                components.grow().unwrap().literal_component().unwrap(),
                "2",
                None,
            );
            assert_archive_literal(
                components.shrink().unwrap().literal_component().unwrap(),
                "0",
                None,
            );
            assert_archive_basis(components.basis().unwrap());
        }
        (CssKnownProperty::AspectRatio, CssKnownPropertyValueRef::AspectRatio(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1.5"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.ratio();
            let CssAspectRatioValue::Ratio(ratio) = typed else {
                panic!("captured ratio");
            };
            assert!(ratio.denominator().is_none());
            assert_archive_literal(ratio.numerator().literal_component().unwrap(), "1.5", None);
        }
        (CssKnownProperty::ScrollbarWidth, CssKnownPropertyValueRef::ScrollbarWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "thin"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "thin" => CssScrollbarWidth::Thin,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Cursor, CssKnownPropertyValueRef::Cursor(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "grab"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            assert_eq!(typed, &CssCursor::Keyword(CssCursorKeyword::Grab));
        }
        (CssKnownProperty::PointerEvents, CssKnownPropertyValueRef::PointerEvents(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "none"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "none" => CssPointerEvents::None,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::UserSelect, CssKnownPropertyValueRef::UserSelect(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "text"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "text" => CssUserSelect::Text,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::OutlineWidth, CssKnownPropertyValueRef::OutlineWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "2px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssOutlineWidth::Length(length) = typed else {
                panic!("captured outline width");
            };
            assert_captured_px(length.literal_component(), "2");
        }
        (CssKnownProperty::Translate, CssKnownPropertyValueRef::Translate(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "10px 20px"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssTranslate::Values(values) = typed else {
                panic!("captured translate");
            };
            assert_captured_px(values.x().literal_component(), "10");
            assert_captured_px(
                values
                    .y()
                    .expect("second captured translate coordinate")
                    .literal_component(),
                "20",
            );
            assert!(values.z().is_none());
        }
        (CssKnownProperty::Rotate, CssKnownPropertyValueRef::Rotate(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "45deg"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let CssRotate::Value(typed) = value.value() else {
                panic!("archived ordinary rotation");
            };
            assert!(typed.axis().is_none());
            assert!(typed.keyword_axis_origin().is_none());
            let literal = typed.angle().literal().expect("archived ordinary angle");
            assert_eq!(literal.numeric().representation(), "45");
            assert_eq!(literal.unit(), CssAngleUnit::Degrees);
        }
        (CssKnownProperty::Scale, CssKnownPropertyValueRef::Scale(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "1.5 2"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let CssScale::Values(values) = typed else {
                panic!("captured scale");
            };
            assert_eq!(values.values().len(), 2);
            assert!(exact_number(
                scale_number(&values.values()[0]).literal_component(),
                "1.5"
            ));
            assert!(exact_number(
                scale_number(&values.values()[1]).literal_component(),
                "2"
            ));
        }
        (CssKnownProperty::OutlineStyle, CssKnownPropertyValueRef::OutlineStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), authored);
            assert!(
                matches!(authored.value, "auto"),
                "unexpected {property:?} authored value: {}",
                authored.value
            );
            let typed = value.value();
            let decoded = match authored.value {
                "auto" => CssOutlineStyle::Auto,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        _ => panic!(
            "{}: no decoded expectation for {property:?}",
            frozen.case_id
        ),
    }
}
fn global_keyword_css(keyword: surgeist_css::CssGlobalKeyword) -> &'static str {
    match keyword {
        surgeist_css::CssGlobalKeyword::Inherit => "inherit",
        surgeist_css::CssGlobalKeyword::Initial => "initial",
        surgeist_css::CssGlobalKeyword::Unset => "unset",
        surgeist_css::CssGlobalKeyword::Revert => "revert",
        surgeist_css::CssGlobalKeyword::RevertLayer => "revert-layer",
        _ => "<future-global>",
    }
}

fn diagnostics_observable(diagnostics: &[CssRecoveryDiagnostic]) -> Vec<String> {
    diagnostics.iter().map(diagnostic_observable).collect()
}

fn diagnostic_observable(diagnostic: &CssRecoveryDiagnostic) -> String {
    let error = diagnostic.error();
    let position = error.position();
    let span = diagnostic.span();
    format!(
        "{}/{}/{}@{}:{}:{}>{}:{}:{}-{}:{}:{}:{}",
        code_name(error.code()),
        root_and_payload(error),
        action_name(diagnostic.action()),
        position.byte_offset().value(),
        position.line().value(),
        position.column().value(),
        span.start().byte_offset().value(),
        span.start().line().value(),
        span.start().column().value(),
        span.end().byte_offset().value(),
        span.end().line().value(),
        span.end().column().value(),
        span.end().byte_offset().value(),
    )
}

fn token(token: Option<&surgeist_css::CssTokenSummary>) -> String {
    token.map_or_else(
        || "-".to_owned(),
        |token| format!("{}:{}", token_kind_name(token.kind()), token.authored()),
    )
}

fn token_kind_name(kind: surgeist_css::CssTokenKind) -> &'static str {
    match kind {
        surgeist_css::CssTokenKind::Ident => "Ident",
        surgeist_css::CssTokenKind::AtKeyword => "AtKeyword",
        surgeist_css::CssTokenKind::Hash => "Hash",
        surgeist_css::CssTokenKind::IdHash => "IdHash",
        surgeist_css::CssTokenKind::String => "String",
        surgeist_css::CssTokenKind::Url => "Url",
        surgeist_css::CssTokenKind::Delim => "Delim",
        surgeist_css::CssTokenKind::Number => "Number",
        surgeist_css::CssTokenKind::Percentage => "Percentage",
        surgeist_css::CssTokenKind::Dimension => "Dimension",
        surgeist_css::CssTokenKind::Whitespace => "Whitespace",
        surgeist_css::CssTokenKind::Comment => "Comment",
        surgeist_css::CssTokenKind::Colon => "Colon",
        surgeist_css::CssTokenKind::Semicolon => "Semicolon",
        surgeist_css::CssTokenKind::Comma => "Comma",
        surgeist_css::CssTokenKind::Cdo => "Cdo",
        surgeist_css::CssTokenKind::Cdc => "Cdc",
        surgeist_css::CssTokenKind::Function => "Function",
        surgeist_css::CssTokenKind::ParenthesisBlock => "ParenthesisBlock",
        surgeist_css::CssTokenKind::SquareBracketBlock => "SquareBracketBlock",
        surgeist_css::CssTokenKind::CurlyBracketBlock => "CurlyBracketBlock",
        surgeist_css::CssTokenKind::BadUrl => "BadUrl",
        surgeist_css::CssTokenKind::BadString => "BadString",
        surgeist_css::CssTokenKind::CloseParenthesis => "CloseParenthesis",
        surgeist_css::CssTokenKind::CloseSquareBracket => "CloseSquareBracket",
        surgeist_css::CssTokenKind::CloseCurlyBracket => "CloseCurlyBracket",
        _ => panic!("future CSS token kind lacks a frozen oracle name"),
    }
}

fn root_and_payload(error: &surgeist_css::CssError) -> String {
    let name = |value: Option<&str>| value.unwrap_or("-").to_owned();
    let expectation = || error.expectation().map_or("-", |value| value.as_str());
    let at_rule = || name(error.at_rule().map(|value| value.as_str()));
    let production = || error.production().map_or("-", |value| value.as_str());
    let descriptor = || name(error.descriptor().map(|value| value.as_str()));
    let feature = || error.feature().map_or("-", |value| value.as_str());
    let property_name = || name(error.property_name().map(|value| value.as_str()));
    match error.code() {
        CssErrorCode::UnexpectedEnd => format!("UnexpectedEnd:{}", expectation()),
        CssErrorCode::UnexpectedToken => format!(
            "UnexpectedToken:{}:{}",
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidAtRulePlacement => {
            format!("InvalidAtRulePlacement:{}:{}", at_rule(), expectation())
        }
        CssErrorCode::InvalidAtRulePrelude => format!(
            "InvalidAtRulePrelude:{}:{}:{}:{}",
            at_rule(),
            production(),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidAtRuleBody => format!(
            "InvalidAtRuleBody:{}:{}:{}:{}",
            at_rule(),
            production(),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::UnknownAtRule => format!("UnknownAtRule:{}", at_rule()),
        CssErrorCode::UnsupportedAtRule => {
            format!("UnsupportedAtRule:{}:{}", at_rule(), feature())
        }
        CssErrorCode::InvalidQualifiedRule => format!(
            "InvalidQualifiedRule:{}:{}:{}",
            production(),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidSelector => format!(
            "InvalidSelector:{}:{}:{}",
            production(),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidMediaQuery => format!(
            "InvalidMediaQuery:{}:{}:{}",
            error.media_feature().map_or("-", |value| value.as_str()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::UnknownProperty => format!("UnknownProperty:{}", property_name()),
        CssErrorCode::UnsupportedProperty => {
            format!("UnsupportedProperty:{}:{}", property_name(), feature())
        }
        CssErrorCode::InvalidPropertyValue => format!(
            "InvalidPropertyValue:{}:{}:{}",
            error
                .known_property()
                .map_or("-", |property| property.stable_id()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidDeclarationAnnotation => format!(
            "InvalidDeclarationAnnotation:{}:{}",
            error
                .declaration_context()
                .map_or_else(|| "-".to_owned(), declaration_context),
            token(error.encountered())
        ),
        CssErrorCode::UnknownDescriptor => {
            format!("UnknownDescriptor:{}:{}", at_rule(), descriptor())
        }
        CssErrorCode::UnsupportedDescriptor => format!(
            "UnsupportedDescriptor:{}:{}:{}",
            at_rule(),
            descriptor(),
            feature()
        ),
        CssErrorCode::InvalidDescriptorValue => format!(
            "InvalidDescriptorValue:{}:{}:{}:{}",
            at_rule(),
            descriptor(),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidDescriptorCombination => format!(
            "InvalidDescriptorCombination:{}:{}:{}",
            at_rule(),
            descriptor(),
            error
                .conflicting_descriptors()
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ),
        CssErrorCode::InvalidColorSyntax => format!(
            "InvalidColorSyntax:{}:{}:{}",
            error.color_component().map_or("-", |value| value.as_str()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::NestingLimit => format!(
            "NestingLimit:{}:{}",
            error
                .nesting_limit()
                .map_or_else(|| "-".to_owned(), |limit| limit.to_string()),
            production()
        ),
        _ => "Future".to_owned(),
    }
}

fn declaration_context(context: CssDeclarationContextRef<'_>) -> String {
    match context {
        CssDeclarationContextRef::KnownProperty(property) => {
            format!("known:{}", property.stable_id())
        }
        CssDeclarationContextRef::CustomProperty(name) => format!("custom:{}", name.as_str()),
        CssDeclarationContextRef::Keyframe(property) => {
            format!("keyframe:{}", property.stable_id())
        }
        CssDeclarationContextRef::KeyframeCustomProperty(name) => {
            format!("keyframe-custom:{}", name.as_str())
        }
        CssDeclarationContextRef::Descriptor {
            at_rule,
            descriptor,
        } => format!("descriptor:{}:{}", at_rule.as_str(), descriptor.as_str()),
        _ => "future".to_owned(),
    }
}

fn code_name(code: CssErrorCode) -> &'static str {
    match code {
        CssErrorCode::UnexpectedEnd => "UnexpectedEnd",
        CssErrorCode::UnexpectedToken => "UnexpectedToken",
        CssErrorCode::InvalidAtRulePlacement => "InvalidAtRulePlacement",
        CssErrorCode::InvalidAtRulePrelude => "InvalidAtRulePrelude",
        CssErrorCode::InvalidAtRuleBody => "InvalidAtRuleBody",
        CssErrorCode::UnknownAtRule => "UnknownAtRule",
        CssErrorCode::UnsupportedAtRule => "UnsupportedAtRule",
        CssErrorCode::InvalidQualifiedRule => "InvalidQualifiedRule",
        CssErrorCode::InvalidSelector => "InvalidSelector",
        CssErrorCode::InvalidMediaQuery => "InvalidMediaQuery",
        CssErrorCode::UnknownProperty => "UnknownProperty",
        CssErrorCode::UnsupportedProperty => "UnsupportedProperty",
        CssErrorCode::InvalidPropertyValue => "InvalidPropertyValue",
        CssErrorCode::InvalidDeclarationAnnotation => "InvalidDeclarationAnnotation",
        CssErrorCode::UnknownDescriptor => "UnknownDescriptor",
        CssErrorCode::UnsupportedDescriptor => "UnsupportedDescriptor",
        CssErrorCode::InvalidDescriptorValue => "InvalidDescriptorValue",
        CssErrorCode::InvalidDescriptorCombination => "InvalidDescriptorCombination",
        CssErrorCode::InvalidColorSyntax => "InvalidColorSyntax",
        CssErrorCode::NestingLimit => "NestingLimit",
        _ => "Future",
    }
}

fn action_name(action: CssRecoveryAction) -> &'static str {
    match action {
        CssRecoveryAction::DropDeclaration => "DropDeclaration",
        CssRecoveryAction::DropDescriptor => "DropDescriptor",
        CssRecoveryAction::DropQualifiedRule => "DropQualifiedRule",
        CssRecoveryAction::DropAtRule => "DropAtRule",
        CssRecoveryAction::DropKeyframeBlock => "DropKeyframeBlock",
        CssRecoveryAction::DropSelectorListItem => "DropSelectorListItem",
        CssRecoveryAction::ReplaceMediaQueryWithNever => "ReplaceMediaQueryWithNever",
        CssRecoveryAction::RetainWithImplicitClosure => "RetainWithImplicitClosure",
        CssRecoveryAction::StopAtNestingLimit => "StopAtNestingLimit",
        _ => "Future",
    }
}

fn assert_strict_parity(row: &Row) {
    match row.entry.as_str() {
        "sheet" => {
            let ordinary = parse_sheet(&row.input);
            let strict = surgeist_css::validate_sheet(&row.input);
            if ordinary.is_clean() {
                assert_eq!(
                    strict,
                    Ok(ordinary.syntax().clone()),
                    "{} strict sheet",
                    row.case_id
                );
            } else {
                assert_eq!(
                    strict.expect_err("recovered sheet").diagnostics(),
                    ordinary.diagnostics(),
                    "{} strict sheet",
                    row.case_id
                );
            }
        }
        "style" => {
            let ordinary = parse_style_attribute(&row.input);
            let strict = surgeist_css::validate_style_attribute(&row.input);
            if ordinary.is_clean() {
                assert_eq!(
                    strict,
                    Ok(ordinary.syntax().clone()),
                    "{} strict style",
                    row.case_id
                );
            } else {
                assert_eq!(
                    strict.expect_err("recovered style").diagnostics(),
                    ordinary.diagnostics(),
                    "{} strict style",
                    row.case_id
                );
            }
        }
        _ => unreachable!(),
    }
}

fn assert_archive_wrapper(
    property: surgeist_css::CssKnownProperty,
    css: &str,
    authored: &AuthoredDeclaration<'_>,
) {
    assert_eq!(authored.id, property.stable_id());
    assert_eq!(authored.value_capability, "typed-value");
    assert_ne!(authored.value, "<unavailable>");
    assert_eq!(css, authored.value);
}
fn assert_archive_literal(
    component: &surgeist_css::CssComponentValue,
    expected: &str,
    expected_unit: Option<&str>,
) {
    use surgeist_css::{CssComponentValueRef, CssValueTokenRef};
    let (number, unit) = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => (number, None),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => (number, Some("%")),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
            (number, Some(unit))
        }
        _ => panic!("captured literal numeric leaf"),
    };
    assert_eq!(number.representation(), expected);
    assert_eq!(unit, expected_unit);
}
fn assert_archive_width(width: &surgeist_css::CssBorderWidth, expected: &str) {
    let surgeist_css::CssBorderWidth::Length(length) = width else {
        panic!("captured exact border width");
    };
    assert_archive_literal(length.literal_component().unwrap(), expected, Some("px"));
}
fn assert_archive_inset(inset: &surgeist_css::CssInsetValue, expected: &str, unit: Option<&str>) {
    let surgeist_css::CssInsetValue::LengthPercentage(length) = inset else {
        panic!("captured inset literal");
    };
    assert_archive_literal(length.literal_component().unwrap(), expected, unit);
}
fn assert_archive_basis(basis: &surgeist_css::CssFlexBasisValue) {
    use surgeist_css::{CssBoxSize, CssFlexBasisRef, CssSizeValue};
    let CssFlexBasisRef::Size(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length))) =
        basis.view()
    else {
        panic!("captured flex basis");
    };
    assert_archive_literal(length.literal_component().unwrap(), "10", Some("rem"));
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}

fn exact_number(component: Option<&surgeist_css::CssComponentValue>, representation: &str) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Number(number))) if number.representation() == representation)
}

fn scale_number(
    value: &surgeist_css::CssTransformScaleComponent,
) -> &surgeist_css::CssSpecifiedNumber {
    let surgeist_css::CssTransformScaleComponent::Number(value) = value else {
        panic!("number factor")
    };
    value
}
