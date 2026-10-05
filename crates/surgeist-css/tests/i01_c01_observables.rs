use std::collections::BTreeSet;

use surgeist_css::{
    CssDeclaration, CssDeclarationContextRef, CssErrorCode, CssImportance, CssPropertyNameRef,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssScopedRule, ErrorKind, parse_sheet,
    parse_style_attribute,
};

const FIXTURE: &str = include_str!("fixtures/i01-c01-observables.tsv");
// Case inputs and feature labels retain their capture provenance. Selected
// value expectations follow CSS Variables 1 §2.1 (2022-06-16): a CSS-wide
// keyword followed by ordinary custom-property tokens is declaration-value,
// not a global value.
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
// endpoint token, following Syntax 3 §7.1 and the descriptor token-origin contract.
// Fonts 4 §6.9.1 also retains named font-feature-values rules and their valid
// styleset entries, without projecting descriptors into property declarations.
// CSS Text 4 §8.2 retains 0.1em exactly; its historical rounded f32 I01
// payload is no longer an exact projection of that specified value.
// https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-letter-spacing
// Lists 3 accepts functional symbols() styles and positional-keyword custom
// style names after the position slot is filled; the two archived inputs are
// unchanged while their selected acceptance observables follow that grammar.
// https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-list-style
const HEADER: &str =
    "case_id\tentry\tfeature\tinput\tclean\tretained\tvalues\tauthored_declarations\tdiagnostics";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Row {
    case_id: String,
    entry: String,
    feature: String,
    input: String,
    clean: String,
    retained: String,
    values: String,
    authored_declarations: String,
    diagnostics: String,
}

impl Row {
    fn fields(&self) -> [&str; 9] {
        [
            &self.case_id,
            &self.entry,
            &self.feature,
            &self.input,
            &self.clean,
            &self.retained,
            &self.values,
            &self.authored_declarations,
            &self.diagnostics,
        ]
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
        if raw.len() != 9 {
            return Err(format!(
                "fixture line {} has {} columns, expected 9",
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
            values: fields[6].clone(),
            authored_declarations: fields[7].clone(),
            diagnostics: fields[8].clone(),
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
        parse_semantic_values(&row.values, &row.case_id)?;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FrozenSemanticValue<'a> {
    id: &'a str,
    payload: &'a str,
    importance: &'a str,
}

fn parse_semantic_values<'a>(
    field: &'a str,
    case_id: &str,
) -> Result<Vec<FrozenSemanticValue<'a>>, String> {
    if field == "-" {
        return Ok(Vec::new());
    }
    field
        .split('~')
        .map(|item| {
            let (id, observation) = item
                .split_once('=')
                .ok_or_else(|| format!("{case_id}: malformed semantic value `{item}`"))?;
            let (payload, importance) = observation
                .rsplit_once('@')
                .ok_or_else(|| format!("{case_id}: missing semantic importance in `{item}`"))?;
            if !matches!(importance, "normal" | "important") {
                return Err(format!(
                    "{case_id}: invalid semantic importance `{importance}`"
                ));
            }
            Ok(FrozenSemanticValue {
                id,
                payload,
                importance,
            })
        })
        .collect()
}

struct FrozenDeclarationCursor<'a> {
    case_id: &'a str,
    input: &'a str,
    semantic_values: Vec<FrozenSemanticValue<'a>>,
    authored_declarations: Vec<AuthoredDeclaration<'a>>,
    semantic_index: usize,
    authored_index: usize,
}

impl<'a> FrozenDeclarationCursor<'a> {
    fn new(row: &'a Row) -> Self {
        Self {
            case_id: &row.case_id,
            input: &row.input,
            semantic_values: parse_semantic_values(&row.values, &row.case_id)
                .expect("frozen semantic-value expectation"),
            authored_declarations: parse_authored_declarations(
                &row.authored_declarations,
                &row.case_id,
            )
            .expect("frozen authored-declaration expectation"),
            semantic_index: 0,
            authored_index: 0,
        }
    }

    fn next(
        &mut self,
        includes_semantic_value: bool,
    ) -> (Option<FrozenSemanticValue<'a>>, AuthoredDeclaration<'a>) {
        let authored = self
            .authored_declarations
            .get(self.authored_index)
            .unwrap_or_else(|| panic!("{}: unexpected retained declaration", self.case_id))
            .clone();
        self.authored_index += 1;
        let semantic = includes_semantic_value.then(|| {
            let value = *self
                .semantic_values
                .get(self.semantic_index)
                .unwrap_or_else(|| panic!("{}: missing frozen semantic value", self.case_id));
            self.semantic_index += 1;
            value
        });
        (semantic, authored)
    }

    fn finish(&self) {
        assert!(
            self.semantic_values.get(self.semantic_index).is_none(),
            "{}: fixture contains an unmatched semantic declaration observable",
            self.case_id,
        );
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
            if !matches!(value_capability, "public" | "deferred-i01") {
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
    let rows = parse_fixture(FIXTURE).expect("valid I01 observable fixture");
    let mut removed_track_cases = 0;
    let mut migrated_content_cases = 0;
    let mut migrated_container_cases = 0;
    let mut migrated_tolerance_cases = 0;
    let mut migrated_auto_repeat_cases = 0;
    let mut migrated_unicode_cases = 0;
    let mut migrated_display_cases = 0;
    let mut migrated_overflow_auto_cases = 0;
    let mut migrated_alignment_cases = 0;
    let mut migrated_feature_tag_cases = 0;
    let mut migrated_thickness_cases = 0;
    let mut migrated_timing_name_cases = 0;
    for row in rows {
        // Fixture feature labels record the original capture profile. Validation is
        // now unconditional, so every historical profile runs through the same API.
        if assert_archived_timing_auto_name_rejection(&row) {
            migrated_timing_name_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_content3_contents_acceptance(&row) {
            migrated_content_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_inline_display_rejection(&row) {
            migrated_display_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_overflow_auto_rejection(&row) {
            migrated_overflow_auto_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_alignment_match_parent_rejection(&row) {
            migrated_alignment_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_feature_tag_rejection(&row) {
            migrated_feature_tag_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_signed_thickness_acceptance(&row) {
            migrated_thickness_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_unicode_feff_rejection(&row) {
            migrated_unicode_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_flow_tolerance_rejection(&row) {
            migrated_tolerance_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_intrinsic_auto_repeat_acceptance(&row) {
            migrated_auto_repeat_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_container_opaque_acceptance(&row) {
            migrated_container_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
        if assert_archived_track_alignment_rejection(&row) {
            removed_track_cases += 1;
            assert_strict_parity(&row);
            continue;
        }
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
    assert_eq!(
        migrated_container_cases, 3,
        "all three archived unknown-container rejections have explicit current witnesses"
    );
    assert_eq!(migrated_feature_tag_cases, 1);
    assert_eq!(migrated_timing_name_cases, 2);
    assert_eq!(
        migrated_thickness_cases, 1,
        "the archived negative thickness rejection has an exact current acceptance witness"
    );
    assert_eq!(
        migrated_tolerance_cases, 4,
        "all four archived old-name cases require current rejection witnesses"
    );
    assert_eq!(
        migrated_auto_repeat_cases, 3,
        "all three archived intrinsic auto-repeat cases require current acceptance witnesses"
    );
    assert_eq!(migrated_unicode_cases, 1);
    assert_eq!(migrated_display_cases, 1);
    assert_eq!(migrated_overflow_auto_cases, 3);
    assert_eq!(migrated_alignment_cases, 1);
    assert_eq!(migrated_content_cases, 1);
    assert_eq!(
        removed_track_cases, 8,
        "all eight obsolete track-property captures have current rejection witnesses"
    );
}

// Values 4 §4.2 reserves CSS-wide keywords and `default`, so `auto` is a
// case-sensitive custom name in Animations 1 §3 and Transitions 1 §2.1.
// Pin each archived rejection and independently check current authored syntax.
// https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
// https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#keyframes
// https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/#transition-property-property
fn assert_archived_timing_auto_name_rejection(row: &Row) -> bool {
    let (property, name_end, token_start, source_end, diagnostic) = match row.case_id.as_str() {
        "catalog.property.baseline.property.animation-name.boundary" => (
            "animation-name",
            14,
            16,
            20,
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.animation-name:a value accepted by the property's grammar:Ident:auto/DropDeclaration@16:0:16>0:0:0-20:0:20:20",
        ),
        "catalog.property.baseline.property.transition-property.boundary" => (
            "transition-property",
            19,
            21,
            25,
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.transition-property:a value accepted by the property's grammar:Ident:auto/DropDeclaration@21:0:21>0:0:0-25:0:25:25",
        ),
        _ => return false,
    };
    let input = format!("{property}: auto");
    assert_eq!(
        row.fields(),
        [
            row.case_id.as_str(),
            "style",
            "both",
            input.as_str(),
            "false",
            "-",
            "-",
            "-",
            diagnostic
        ],
    );
    let report = parse_style_attribute(&row.input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(report.diagnostics().is_empty());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained timing name declaration");
    };
    let occurrence = declaration.clone();
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let known = declaration.known().expect("known timing property");
    assert_eq!(known.property().canonical_name(), property);
    match known.property_value().expect("typed timing name") {
        surgeist_css::CssKnownPropertyValueRef::AnimationName(value) => {
            assert!(
                matches!(value.names().names(), [surgeist_css::CssAnimationName::Custom(name)] if name.as_str() == "auto")
            );
            assert_eq!(value.as_css(), "auto");
            assert_eq!(value.names().serialize_specified().unwrap(), "auto");
        }
        surgeist_css::CssKnownPropertyValueRef::TransitionProperty(value) => {
            assert!(
                matches!(value.properties().properties(), [surgeist_css::CssTransitionProperty::Custom(name)] if name.as_str() == "auto")
            );
            assert_eq!(value.as_css(), "auto");
            assert_eq!(value.properties().serialize_specified().unwrap(), "auto");
        }
        other => panic!("unexpected timing name value: {other:?}"),
    }
    let name = declaration.parsed_name().expect("parsed name provenance");
    assert_eq!(name.source().as_str(), row.input);
    assert_eq!(name.span().start().byte_offset().value(), 0);
    assert_eq!(name.span().end().byte_offset().value(), name_end);
    assert_eq!(declaration.position(), Some(name.span().start()));
    let parsed = declaration.parsed_value().expect("parsed value provenance");
    assert_eq!(parsed.source().as_str(), row.input);
    assert_eq!(parsed.span().start().byte_offset().value(), token_start - 1);
    assert_eq!(parsed.span().end().byte_offset().value(), source_end);
    let surgeist_css::CssValueOrigin::Parsed(token) = declaration
        .value_components()
        .items()
        .last()
        .unwrap()
        .origin()
    else {
        panic!("parsed name token provenance");
    };
    assert_eq!(token.source().as_str(), row.input);
    assert_eq!(token.span().start().byte_offset().value(), token_start);
    assert_eq!(token.span().end().byte_offset().value(), source_end);
    let canonical = format!("{property}: auto;");
    assert_eq!(declaration.to_specified_css().unwrap(), canonical);
    assert!(declaration.same_occurrence(&occurrence));
    assert_eq!(
        surgeist_css::validate_style_attribute(&row.input),
        Ok(report.syntax().clone())
    );
    assert_eq!(
        report.clone().into_validation_result(),
        Ok(report.syntax().clone())
    );
    let reparsed = parse_style_attribute(&canonical);
    assert!(reparsed.is_clean());
    assert_eq!(reparsed.syntax()[0].body(), declaration.body());
    assert_eq!(reparsed.syntax()[0].to_specified_css().unwrap(), canonical);
    true
}

// Content 3 admits `contents` as one generated-content item. The unchanged
// I01 input now has a current typed value, but no exact historical CssContent
// projection; validate both rather than inventing a legacy enum variant.
// https://www.w3.org/TR/2025/WD-css-content-3-20251204/#valdef-content-contents
fn assert_content3_contents_acceptance(row: &Row) -> bool {
    if row.case_id != "catalog.property.baseline.property.content.boundary" {
        return false;
    }
    assert_eq!(row.entry, "style");
    assert_eq!(row.feature, "both");
    assert_eq!(row.input, "content: contents");
    assert_eq!(row.clean, "true");
    assert_eq!(row.retained, "property:baseline.property.content");
    assert_eq!(
        row.values,
        "baseline.property.content=typed:contents@normal"
    );
    assert_eq!(
        row.authored_declarations,
        "baseline.property.content=deferred-i01:contents@public:normal"
    );
    assert_eq!(row.diagnostics, "-");
    let report = parse_style_attribute(&row.input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one current content declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let Some(surgeist_css::CssKnownPropertyValueRef::Content(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("checked content wrapper")
    };
    let surgeist_css::CssContentValue::Generated(generated) = value.value() else {
        panic!("generated contents")
    };
    assert!(matches!(
        generated.items(),
        [surgeist_css::CssContentValueItem::Contents]
    ));
    assert!(matches!(
        generated.body(),
        surgeist_css::CssGeneratedContentBodyRef::List([
            surgeist_css::CssContentValueItem::Contents
        ])
    ));

    assert_eq!(value.as_css(), "contents");
    assert_eq!(value.value().serialize_specified().unwrap(), "contents");
    assert!(matches!(
        declaration.value_components().items()[0].origin(),
        surgeist_css::CssValueOrigin::Parsed(_)
    ));
    true
}

// Selected Text Decoration 4 §2.4 admits unrestricted <length>/<percentage>
// authored thicknesses; the actual device-pixel floor belongs downstream.
// Preserve the archived rejection and inspect the exact signed specified value.
// https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/#text-decoration-thickness-property
fn assert_archived_signed_thickness_acceptance(row: &Row) -> bool {
    if row.case_id != "catalog.property.baseline.property.text-decoration-thickness.boundary" {
        return false;
    }
    assert_eq!(
        row.fields(),
        [
            row.case_id.as_str(),
            "style",
            "both",
            "text-decoration-thickness: -1px",
            "false",
            "-",
            "-",
            "-",
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.text-decoration-thickness:a value accepted by the property's grammar:Dimension:-1px/DropDeclaration@27:0:27>0:0:0-31:0:31:31",
        ]
    );
    let report = parse_style_attribute(&row.input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(report.diagnostics().is_empty());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained signed thickness declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let known = declaration.known().expect("known thickness property");
    assert_eq!(
        known.property(),
        surgeist_css::CssKnownProperty::TextDecorationThickness
    );
    let Some(surgeist_css::CssKnownPropertyValueRef::TextDecorationThickness(value)) =
        known.property_value()
    else {
        panic!("typed signed thickness")
    };
    let surgeist_css::CssTextDecorationThickness::Length(length) = value.value() else {
        panic!("signed length branch")
    };
    assert_captured_px(length.literal_component(), "-1");
    assert_eq!(value.as_css(), "-1px");
    let name = declaration.parsed_name().expect("original name provenance");
    assert_eq!(name.source().as_str(), row.input);
    assert_eq!(name.span().start().byte_offset().value(), 0);
    assert_eq!(name.span().end().byte_offset().value(), 25);
    let parsed = declaration
        .parsed_value()
        .expect("original value provenance");
    assert_eq!(parsed.source().as_str(), row.input);
    assert_eq!(parsed.span().start().byte_offset().value(), 26);
    assert_eq!(parsed.span().end().byte_offset().value(), 31);
    true
}

// The checked current tag grammar rejects the malformed quoted tag itself.
// The frozen capture diagnosed the following `on` token instead; keep those
// historical bytes and assert the current diagnostic at the same declaration.
fn assert_archived_feature_tag_rejection(row: &Row) -> bool {
    if row.case_id != "catalog.property.baseline.property.font-feature-settings.boundary" {
        return false;
    }
    assert_eq!(row.input, "font-feature-settings: \"abc\" on");
    assert_eq!(row.clean, "false");
    assert_eq!(row.retained, "-");
    assert_eq!(row.values, "-");
    assert_eq!(row.authored_declarations, "-");
    assert!(row.diagnostics.contains(":Ident:on/DropDeclaration@29"));
    let current = observe(row);
    assert_eq!(current.clean, "false");
    assert_eq!(current.retained, "-");
    assert_eq!(
        current.diagnostics,
        "InvalidPropertyValue/InvalidPropertyValue:baseline.property.font-feature-settings:a value accepted by the property's grammar:String:\"abc\"/DropDeclaration@23:0:23>0:0:0-31:0:31:31"
    );
    true
}

// Text 4 §7.4 admits match-parent on the last-line longhand. The immutable I01
// capture predates this grammar; assert its original rejection and the current
// typed acceptance independently rather than changing the fixture payload.
fn assert_archived_alignment_match_parent_rejection(row: &Row) -> bool {
    if row.case_id != "catalog.property.baseline.property.text-align-last.boundary" {
        return false;
    }
    assert_eq!(row.entry, "style");
    assert_eq!(row.feature, "both");
    assert_eq!(row.input, "text-align-last: match-parent");
    assert_eq!(row.clean, "false");
    assert_eq!(row.retained, "-");
    assert_eq!(row.values, "-");
    assert_eq!(row.authored_declarations, "-");
    assert_eq!(
        row.diagnostics,
        "InvalidPropertyValue/InvalidPropertyValue:baseline.property.text-align-last:a value accepted by the property's grammar:Ident:match-parent/DropDeclaration@17:0:17>0:0:0-29:0:29:29"
    );

    let report = parse_style_attribute(&row.input);
    assert!(report.is_clean());
    let [declaration] = report.syntax().as_slice() else {
        panic!("current last-line grammar retains one declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let known = declaration.known().expect("known last-line property");
    assert_eq!(
        known.property(),
        surgeist_css::CssKnownProperty::TextAlignLast
    );
    let Some(surgeist_css::CssKnownPropertyValueRef::TextAlignLast(value)) = known.property_value()
    else {
        panic!("current last-line value")
    };
    assert_eq!(
        value.value(),
        &surgeist_css::CssTextAlignLastValue::Keyword(surgeist_css::CssTextAlign::MatchParent)
    );

    assert_eq!(value.as_css(), "match-parent");
    let parsed = declaration
        .parsed_value()
        .expect("original value provenance");
    assert_eq!(parsed.span().start().byte_offset().value(), 16);
    assert_eq!(parsed.span().end().byte_offset().value(), 29);
    true
}

// Overflow 3 §3.1 admits `auto` on the authored overflow axes. Keep the three
// archived I01 rejection observations verbatim and assert the current values.
// https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#overflow-control
fn assert_archived_overflow_auto_rejection(row: &Row) -> bool {
    let (property, position) = match row.case_id.as_str() {
        "catalog.property.baseline.property.overflow-x.boundary" => ("overflow-x", 12),
        "catalog.property.baseline.property.overflow-y.boundary" => ("overflow-y", 12),
        "catalog.property.baseline.property.overflow.boundary" => ("overflow", 10),
        _ => return false,
    };
    assert_eq!(row.entry, "style");
    assert_eq!(row.feature, "both");
    assert_eq!(row.input, format!("{property}: auto"));
    assert_eq!(row.clean, "false");
    assert_eq!(row.retained, "-");
    assert_eq!(row.values, "-");
    assert_eq!(row.authored_declarations, "-");
    assert_eq!(
        row.diagnostics,
        format!(
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.{property}:a value accepted by the property's grammar:Ident:auto/DropDeclaration@{position}:0:{position}>0:0:0-{}:0:{}:{}",
            position + 4,
            position + 4,
            position + 4,
        )
    );

    let report = parse_style_attribute(&row.input);
    assert!(report.is_clean(), "{} current overflow", row.case_id);
    let [declaration] = report.syntax().as_slice() else {
        panic!("{} must retain one declaration", row.case_id);
    };
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let known = declaration.known().expect("known overflow property");
    assert_eq!(known.property().canonical_name(), property);
    assert_eq!(
        known.property().stable_id(),
        format!("baseline.property.{property}")
    );
    match known.property_value().expect("typed overflow value") {
        surgeist_css::CssKnownPropertyValueRef::Overflow(value) => {
            assert_eq!(value.value().x(), surgeist_css::CssOverflow::Auto);
            assert_eq!(value.value().authored_y(), None);
            assert_eq!(value.value().y(), surgeist_css::CssOverflow::Auto);
            assert_eq!(value.as_css(), "auto");
        }
        surgeist_css::CssKnownPropertyValueRef::OverflowX(value) => {
            assert_eq!(*value.value(), surgeist_css::CssOverflow::Auto);
            assert_eq!(value.as_css(), "auto");
        }
        surgeist_css::CssKnownPropertyValueRef::OverflowY(value) => {
            assert_eq!(*value.value(), surgeist_css::CssOverflow::Auto);
            assert_eq!(value.as_css(), "auto");
        }
        other => panic!("{} wrong typed value: {other:?}", row.case_id),
    }
    let parsed = declaration.parsed_value().expect("parsed overflow value");
    assert_eq!(parsed.span().start().byte_offset().value(), position - 1);
    assert_eq!(parsed.span().end().byte_offset().value(), position + 4);
    true
}

// Display3 §2 admits inline with flow inside. Keep the archived I01 rejection
// verbatim, and independently assert the selected current grammar and payload.
// https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#the-display-properties
fn assert_archived_inline_display_rejection(row: &Row) -> bool {
    if row.case_id != "catalog.property.baseline.property.display.boundary" {
        return false;
    }
    assert_eq!(row.entry, "style");
    assert_eq!(row.feature, "both");
    assert_eq!(row.input, "display: inline");
    assert_eq!(row.clean, "false");
    assert_eq!(row.retained, "-");
    assert_eq!(row.values, "-");
    assert_eq!(row.authored_declarations, "-");
    assert_eq!(
        row.diagnostics,
        "InvalidPropertyValue/InvalidPropertyValue:baseline.property.display:a value accepted by the property's grammar:Ident:inline/DropDeclaration@9:0:9>0:0:0-15:0:15:15"
    );
    let report = parse_style_attribute(&row.input);
    assert!(report.is_clean());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained display")
    };
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let surgeist_css::CssKnownPropertyValueRef::Display(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed display")
    };
    assert_eq!(
        *value.value(),
        surgeist_css::CssDisplayValue::OutsideInside {
            outside: surgeist_css::CssDisplayOutside::Inline,
            inside: surgeist_css::CssDisplayInside::Flow,
        }
    );
    assert_eq!(value.as_css(), "inline");

    let parsed = declaration.parsed_value().unwrap();
    // The retained value range includes the space immediately after the colon;
    // the property wrapper's ordinary as_css() excludes that boundary trivia.
    assert_eq!(parsed.span().start().byte_offset().value(), 8);
    assert_eq!(parsed.span().end().byte_offset().value(), 15);
    true
}

// Syntax 3 string input preserves U+FEFF as an identifier code point. The
// following at-keyword remains in that qualified rule's prelude, invalidating
// the complete rule through its block. Preserve the historical capture verbatim.
fn assert_archived_unicode_feff_rejection(row: &Row) -> bool {
    if row.case_id != "focused.stylesheet-recovery.13" {
        return false;
    }
    assert_eq!(row.entry, "sheet");
    assert_eq!(row.feature, "both");
    assert_eq!(
        row.input,
        "\u{feff} /* leading */ @charset \"UTF-8\"; .after { color: blue; }"
    );
    assert_eq!(row.clean, "true");
    assert_eq!(
        row.retained,
        "rule:baseline.rule.style~property:baseline.property.color"
    );
    assert_eq!(
        row.values,
        "baseline.property.color=typed:Rgba(CssRgbaColor { red: 0, green: 0, blue: 255, alpha: 1.0 })@normal"
    );
    assert_eq!(
        row.authored_declarations,
        "baseline.property.color=deferred-i01:blue@public:normal"
    );
    assert_eq!(row.diagnostics, "-");
    let report = parse_sheet(&row.input);
    assert!(!report.is_clean());
    assert!(report.syntax().encoding().is_none());
    assert!(report.syntax().rules().is_empty());
    assert_eq!(
        diagnostics_observable(report.diagnostics()),
        [
            "InvalidSelector/InvalidSelector:baseline.selector.complex:a supported selector:AtKeyword:@charset/DropQualifiedRule@18:0:16>0:0:0-59:0:57:59"
        ]
    );
    true
}

// Conditional 5 query-in-parens admits general-enclosed syntax. These three
// historical rejections remain unchanged in the archive, while current syntax
// retains the condition and independently expected nested style. The ordinary
// property style query and scroll-state feature now have recognized structure.
fn assert_archived_container_opaque_acceptance(row: &Row) -> bool {
    let (input, query) = match row.case_id.as_str() {
        "catalog.non-property.baseline.rule.container.boundary" => (
            "@container scroll-state(stuck: top) { .x { color: red; } }",
            "scroll-state(stuck: top)",
        ),
        "catalog.non-property.baseline.container.condition.boundary" => (
            "@container style(color: red) { .x { color: red; } }",
            "style(color: red)",
        ),
        "catalog.non-property.baseline.container.size-feature.boundary" => (
            "@container (unknown-size > 1px) { .x { color: red; } }",
            "(unknown-size > 1px)",
        ),
        _ => return false,
    };
    assert_eq!(row.input, input);
    assert_eq!(row.entry, "sheet");
    assert_eq!(row.clean, "false");
    assert_eq!(row.retained, "-");
    assert_eq!(row.values, "-");
    assert_eq!(row.authored_declarations, "-");
    assert!(row.diagnostics.starts_with("InvalidAtRulePrelude/"));
    let report = parse_sheet(input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Container(container)] = report.syntax().rules() else {
        panic!("retained container")
    };
    let value = container.prelude().entries()[0].query().unwrap();
    if query == "style(color: red)" {
        assert!(matches!(
            value.kind(),
            surgeist_css::CssContainerConditionKind::Style(_)
        ));
    } else if query == "scroll-state(stuck: top)" {
        let surgeist_css::CssContainerConditionKind::ScrollState(scroll) = value.kind() else {
            panic!("scroll-state")
        };
        let surgeist_css::CssContainerScrollQueryKind::Feature(
            surgeist_css::CssContainerScrollFeature::Stuck(operand),
        ) = scroll.kind()
        else {
            panic!("stuck feature")
        };
        assert!(matches!(
            operand.view(),
            surgeist_css::CssContainerStuckValueRef::Keyword(
                surgeist_css::CssContainerStuckKeyword::Top
            )
        ));
        assert_eq!(operand.serialize().unwrap().as_css(), "top");
    } else {
        assert!(matches!(
            value.kind(),
            surgeist_css::CssContainerConditionKind::GeneralEnclosed(_)
        ));
    }
    assert_eq!(value.serialize().unwrap().as_css(), format!("{query} "));
    let surgeist_css::CssValueOrigin::Parsed(query_origin) = value.origin() else {
        panic!("original query source")
    };
    assert_eq!(query_origin.source().as_str(), input);
    assert_eq!(
        query_origin.span().start().byte_offset().value(),
        input.find(query).unwrap()
    );
    let [CssRule::Style(style)] = container.rules() else {
        panic!("retained child style")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one selector")
    };
    assert_eq!(
        selector.selector(),
        &surgeist_css::CssSelector::Class("x".to_owned())
    );
    let [declaration] = style.declarations().as_slice() else {
        panic!("one color declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let parsed_value = declaration.parsed_value().unwrap();
    assert!(parsed_value.source().same_snapshot(query_origin.source()));
    assert_eq!(parsed_value.source().as_str(), input);
    let surgeist_css::CssKnownPropertyValueRef::Color(color) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    // The captured Rgba projection is archival; the sole authored color keeps
    // the named branch without resolving it to channels.
    assert_eq!(color.value().named().unwrap().name(), "red");
    true
}

// Grid3 relaxes the auto-repeat body to general track-size content. These three
// exact captured inputs therefore become valid; the archived rejection fields
// remain immutable evidence of their original Grid2 interpretation.
// https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat
fn assert_archived_intrinsic_auto_repeat_acceptance(row: &Row) -> bool {
    use surgeist_css::{
        CssGridAutoFlowAxis, CssGridAutoRepeatKind, CssGridAutoTrackComponent,
        CssGridTrackBreadthKind, CssGridTrackRepeatComponent, CssKnownProperty,
        CssKnownPropertyValueRef,
    };

    let (entry, input, retained, diagnostics, importance) = match row.case_id.as_str() {
        "catalog.property.baseline.property.grid.positive" => (
            "style",
            "grid: auto-flow dense 12px / repeat(auto-fit, 1fr)",
            "-",
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.grid:a value accepted by the property's grammar:Dimension:1fr/DropDeclaration@46:0:46>0:0:0-50:0:50:50",
            CssImportance::Normal,
        ),
        "focused.property-schema.baseline.property.grid.important" => (
            "sheet",
            ".test { GRID: auto-flow dense 12px / repeat(auto-fit, 1fr) !important; }",
            "rule:baseline.rule.style",
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.grid:a value accepted by the property's grammar:Dimension:1fr/DropDeclaration@54:0:54>8:0:8-70:0:70:70",
            CssImportance::Important,
        ),
        "focused.property-schema.baseline.property.grid.ordinary" => (
            "sheet",
            ".test { GRID: auto-flow dense 12px / repeat(auto-fit, 1fr); }",
            "rule:baseline.rule.style",
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.grid:a value accepted by the property's grammar:Dimension:1fr/DropDeclaration@54:0:54>8:0:8-59:0:59:59",
            CssImportance::Normal,
        ),
        _ => return false,
    };
    assert_eq!(
        row.fields(),
        [
            row.case_id.as_str(),
            entry,
            "both",
            input,
            "false",
            retained,
            "-",
            "-",
            diagnostics
        ]
    );
    let declaration = if entry == "style" {
        let report = parse_style_attribute(input);
        assert!(
            report.is_clean(),
            "{}: {:?}",
            row.case_id,
            report.diagnostics()
        );
        let [declaration] = report.syntax().as_slice() else {
            panic!("the formerly rejected Grid declaration is retained");
        };
        declaration.clone()
    } else {
        let report = parse_sheet(input);
        assert!(
            report.is_clean(),
            "{}: {:?}",
            row.case_id,
            report.diagnostics()
        );
        let [CssRule::Style(style)] = report.syntax().rules() else {
            panic!("the original style rule is retained");
        };
        assert!(style.rules().is_empty());
        let [selector] = style.selectors().selectors() else {
            panic!("one original selector");
        };
        assert_eq!(
            selector.selector(),
            &surgeist_css::CssSelector::Class("test".to_owned())
        );
        let [declaration] = style.declarations().as_slice() else {
            panic!("the original style rule now retains its Grid declaration");
        };
        declaration.clone()
    };
    assert_eq!(declaration.importance(), importance);
    let name = declaration
        .parsed_name()
        .expect("original property-name provenance");
    let name_start = if entry == "style" {
        0
    } else {
        ".test { ".len()
    };
    assert_eq!(name.source().as_str(), input);
    assert_eq!(name.span().start().byte_offset().value(), name_start);
    assert_eq!(name.span().start().line().value(), 0);
    assert_eq!(name.span().start().column().value() as usize, name_start);
    assert_eq!(
        name.span().end().byte_offset().value(),
        input.find(':').unwrap()
    );
    assert_eq!(declaration.position(), Some(name.span().start()));
    let authored = "auto-flow dense 12px / repeat(auto-fit, 1fr)";
    let origin = declaration
        .parsed_value()
        .expect("original value provenance");
    assert!(origin.source().same_snapshot(name.source()));
    let start = origin.span().start().byte_offset().value();
    let end = origin.span().end().byte_offset().value();
    assert_eq!(origin.source().as_str()[start..end].trim(), authored);

    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::Grid);
    let Some(CssKnownPropertyValueRef::Grid(value)) = known.property_value() else {
        panic!("a complete current Grid value");
    };
    assert_eq!(value.as_css(), authored);
    assert!(value.value().template_value().is_none());
    let flow = value.value().auto_flow().unwrap();
    assert_eq!(flow.axis(), CssGridAutoFlowAxis::Row);
    assert!(flow.dense());
    let [implicit] = value.value().auto_tracks().unwrap().sizes() else {
        panic!("one original implicit track size");
    };
    assert_eq!(
        implicit
            .breadth()
            .unwrap()
            .length_percentage()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "12px"
    );
    let explicit = value.value().explicit_tracks().unwrap();
    assert!(explicit.general_list().is_none());
    let [CssGridAutoTrackComponent::AutoRepeat(repeat)] =
        explicit.auto_list().unwrap().components()
    else {
        panic!("one automatic repeat in the explicit column axis");
    };
    assert_eq!(repeat.kind(), CssGridAutoRepeatKind::AutoFit);
    let [CssGridTrackRepeatComponent::TrackSize(size)] = repeat.content().components() else {
        panic!("one nonrecursive general track size");
    };
    let breadth = size.breadth().unwrap();
    assert_eq!(breadth.kind(), CssGridTrackBreadthKind::Fraction);
    assert_eq!(
        breadth.flex().unwrap().serialize_specified().unwrap(),
        "1fr"
    );

    true
}

// Grid3's selected publication replaces the old property identity. The archive
// remains a record of the original parser, including its literal Debug values:
// https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#propdef-flow-tolerance
// Current behavior rejects those exact unchanged inputs. It must not reconstruct
// an obsolete production value merely to satisfy the archived observation cursor.
fn assert_archived_flow_tolerance_rejection(row: &Row) -> bool {
    let (entry, input, clean, retained, values, authored, diagnostics) = match row.case_id.as_str()
    {
        "catalog.property.baseline.property.grid-flow-tolerance.boundary" => (
            "style",
            "grid-flow-tolerance: solid",
            "false",
            "-",
            "-",
            "-",
            "InvalidPropertyValue/InvalidPropertyValue:baseline.property.grid-flow-tolerance:a value accepted by the property's grammar:Ident:solid/DropDeclaration@21:0:21>0:0:0-26:0:26:26",
        ),
        "catalog.property.baseline.property.grid-flow-tolerance.positive" => (
            "style",
            "grid-flow-tolerance: infinite",
            "true",
            "property:baseline.property.grid-flow-tolerance",
            "baseline.property.grid-flow-tolerance=typed:Infinite@normal",
            "baseline.property.grid-flow-tolerance=deferred-i01:infinite@public:normal",
            "-",
        ),
        "focused.property-schema.baseline.property.grid-flow-tolerance.important" => (
            "sheet",
            ".test { GRID-FLOW-TOLERANCE: infinite !important; }",
            "true",
            "rule:baseline.rule.style~property:baseline.property.grid-flow-tolerance",
            "baseline.property.grid-flow-tolerance=typed:Infinite@important",
            "baseline.property.grid-flow-tolerance=deferred-i01:infinite@public:important",
            "-",
        ),
        "focused.property-schema.baseline.property.grid-flow-tolerance.ordinary" => (
            "sheet",
            ".test { GRID-FLOW-TOLERANCE: infinite; }",
            "true",
            "rule:baseline.rule.style~property:baseline.property.grid-flow-tolerance",
            "baseline.property.grid-flow-tolerance=typed:Infinite@normal",
            "baseline.property.grid-flow-tolerance=deferred-i01:infinite@public:normal",
            "-",
        ),
        _ => return false,
    };
    assert_eq!(
        row.fields(),
        [
            row.case_id.as_str(),
            entry,
            "both",
            input,
            clean,
            retained,
            values,
            authored,
            diagnostics
        ]
    );
    assert_current_obsolete_property_rejection(row);
    true
}

// CSSWG dropped both former Masonry properties on 4 October 2023; neither
// selected Alignment 3 nor Grid 3 defines them. Preserve the eight historical
// captures verbatim and assert today's unknown-property recovery independently.
// https://github.com/w3c/csswg-drafts/issues/8207#issuecomment-1747805578
fn assert_archived_track_alignment_rejection(row: &Row) -> bool {
    let (entry, input, property, important, boundary) = match row.case_id.as_str() {
        "catalog.property.baseline.property.align-tracks.boundary" => {
            ("style", "align-tracks: auto", "align-tracks", false, true)
        }
        "catalog.property.baseline.property.align-tracks.positive" => (
            "style",
            "align-tracks: center",
            "align-tracks",
            false,
            false,
        ),
        "catalog.property.baseline.property.justify-tracks.boundary" => (
            "style",
            "justify-tracks: auto",
            "justify-tracks",
            false,
            true,
        ),
        "catalog.property.baseline.property.justify-tracks.positive" => (
            "style",
            "justify-tracks: space-evenly",
            "justify-tracks",
            false,
            false,
        ),
        "focused.property-schema.baseline.property.align-tracks.important" => (
            "sheet",
            ".test { ALIGN-TRACKS: center !important; }",
            "align-tracks",
            true,
            false,
        ),
        "focused.property-schema.baseline.property.align-tracks.ordinary" => (
            "sheet",
            ".test { ALIGN-TRACKS: center; }",
            "align-tracks",
            false,
            false,
        ),
        "focused.property-schema.baseline.property.justify-tracks.important" => (
            "sheet",
            ".test { JUSTIFY-TRACKS: space-evenly !important; }",
            "justify-tracks",
            true,
            false,
        ),
        "focused.property-schema.baseline.property.justify-tracks.ordinary" => (
            "sheet",
            ".test { JUSTIFY-TRACKS: space-evenly; }",
            "justify-tracks",
            false,
            false,
        ),
        _ => return false,
    };
    let identity = format!("baseline.property.{property}");
    let importance = if important { "important" } else { "normal" };
    let (value, semantic) = if property == "align-tracks" {
        ("center", "Center")
    } else {
        ("space-evenly", "SpaceEvenly")
    };
    let retained = if boundary {
        "-".to_owned()
    } else {
        format!(
            "{}property:{identity}",
            if entry == "sheet" {
                "rule:baseline.rule.style~"
            } else {
                ""
            }
        )
    };
    let values = if boundary {
        "-".to_owned()
    } else {
        format!("{identity}=typed:{semantic}@{importance}")
    };
    let authored = if boundary {
        "-".to_owned()
    } else {
        format!("{identity}=deferred-i01:{value}@public:{importance}")
    };
    let diagnostics = if !boundary {
        "-"
    } else if property == "align-tracks" {
        "InvalidPropertyValue/InvalidPropertyValue:baseline.property.align-tracks:a value accepted by the property's grammar:Ident:auto/DropDeclaration@14:0:14>0:0:0-18:0:18:18"
    } else {
        "InvalidPropertyValue/InvalidPropertyValue:baseline.property.justify-tracks:a value accepted by the property's grammar:Ident:auto/DropDeclaration@16:0:16>0:0:0-20:0:20:20"
    };
    assert_eq!(
        row.fields(),
        [
            row.case_id.as_str(),
            entry,
            "both",
            input,
            if boundary { "false" } else { "true" },
            &retained,
            &values,
            &authored,
            diagnostics
        ]
    );
    assert_current_obsolete_property_rejection(row);
    true
}

#[test]
fn removed_track_declarations_preserve_adjacent_current_alignment_and_custom_semantics() {
    let input = "color: red; align-tracks: center; align-content: first baseline !important; justify-tracks: var(--old); --align-tracks: center";
    let report = parse_style_attribute(input);
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, name) in report
        .diagnostics()
        .iter()
        .zip(["align-tracks", "justify-tracks"])
    {
        let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
            panic!("unknown obsolete property")
        };
        assert_eq!(detail.name().as_str(), name);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            input.find(name).unwrap()
        );
    }
    let [color, alignment, custom] = report.syntax().as_slice() else {
        panic!("three adjacent declarations remain")
    };
    assert_eq!(
        color.known().unwrap().property(),
        surgeist_css::CssKnownProperty::Color
    );
    let Some(surgeist_css::CssKnownPropertyValueRef::Color(value)) =
        color.known().unwrap().property_value()
    else {
        panic!("adjacent color")
    };
    assert_eq!(value.as_css(), "red");
    assert_eq!(alignment.importance(), CssImportance::Important);
    let Some(surgeist_css::CssKnownPropertyValueRef::AlignContent(value)) =
        alignment.known().unwrap().property_value()
    else {
        panic!("current alignment")
    };
    assert_eq!(
        value.value().value(),
        surgeist_css::CssAlignmentValue::Baseline(surgeist_css::CssBaselinePosition::First)
    );
    assert_eq!(value.value().serialize_specified().unwrap(), "baseline");
    let custom = custom.custom().unwrap();
    assert_eq!(custom.name().as_str(), "--align-tracks");
    assert_eq!(custom.value().value().unwrap().as_css(), "center");
}

fn assert_current_obsolete_property_rejection(row: &Row) {
    let current_diagnostics = if row.entry == "style" {
        let report = parse_style_attribute(&row.input);
        assert!(!report.is_clean());
        assert!(
            report.syntax().is_empty(),
            "the obsolete property is dropped"
        );
        report.diagnostics().to_vec()
    } else {
        let report = parse_sheet(&row.input);
        assert!(!report.is_clean());
        let [CssRule::Style(style)] = report.syntax().rules() else {
            panic!("the containing style rule remains after dropping its obsolete declaration");
        };
        assert!(style.declarations().is_empty());
        assert!(style.rules().is_empty());
        let [selector] = style.selectors().selectors() else {
            panic!("the authored selector remains");
        };
        assert_eq!(
            selector.selector(),
            &surgeist_css::CssSelector::Class("test".to_owned())
        );
        report.diagnostics().to_vec()
    };
    let [diagnostic] = current_diagnostics.as_slice() else {
        panic!("exactly one unknown-property diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
        panic!("unknown authored property identity");
    };
    let property_start = if row.entry == "style" {
        0
    } else {
        ".test { ".len()
    };
    let property_end = row.input.find(':').unwrap();
    assert_eq!(
        detail.name().as_str(),
        &row.input[property_start..property_end]
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        property_start
    );
    assert_eq!(diagnostic.error().position().line().value(), 0);
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        row.input[..property_start].encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        property_start
    );
    let declaration_end = row
        .input
        .find(';')
        .map_or(row.input.len(), |index| index + 1);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
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
        .find(|row| row.diagnostics != "-")
        .expect("fixture must contain a recovery diagnostic");
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
            row.retained.contains("property:baseline.property.")
                && row.authored_declarations != "-"
                && row.values != "-"
        })
        .expect("fixture must contain a semantic property declaration");
    let property_id = declaration_row
        .retained
        .split('~')
        .find_map(|item| item.strip_prefix("property:"))
        .expect("retained property identity");
    let mut malformed_retained_property = declaration_row.clone();
    malformed_retained_property.retained = malformed_retained_property
        .retained
        .replace(property_id, "not-a-stable-id");
    malformed_retained_property.values = malformed_retained_property
        .values
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
                "/InvalidAtRulePrelude:container:baseline.rule.container:a supported @container prelude:Function:style(/",
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
            diagnostic.replacen(">0:0:0-51:0:51:51", ">0:0-51:0:51:51", 1),
            "diagnostic span",
            "ill-formed diagnostic span",
        ),
        (
            diagnostic.replacen(">0:0:0-51:0:51:51", ">51:0:51-0:0:0:0", 1),
            "diagnostic span",
            "reversed diagnostic span",
        ),
        (
            diagnostic.replacen("-51:0:51:51", "-51:0:51:50", 1),
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
    delimiter_payload.diagnostics = diagnostic.replacen(
        "Function:style(",
        "Url:url(https://example.test/@value:part)",
        1,
    );
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
    row.fields().map(escape).join("\t")
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
                declaration_observables(report.syntax().as_slice(), "public", true, &mut frozen);
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
                    let (semantic, authored) = frozen.next(false);
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
                        semantic,
                        &authored,
                        frozen,
                    );
                }
            }
        }
        CssRule::Style(rule) => {
            retained.push("rule:baseline.rule.style".to_owned());
            let ids =
                declaration_observables(rule.declarations().as_slice(), "public", true, frozen);
            retained.extend(ids);
            for child in rule.rules() {
                rule_observables(child, retained, frozen);
            }
        }
        CssRule::NestedDeclarations(rule) => {
            // This is an authored structural node, not another style selector or
            // a fabricated conformance-catalog identity.
            retained.push("rule:nested-declarations".to_owned());
            let ids =
                declaration_observables(rule.declarations().as_slice(), "public", true, frozen);
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
            let ids =
                declaration_observables(rule.declarations().as_slice(), "public", true, frozen);
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
    includes_semantic_value: bool,
    frozen: &mut FrozenDeclarationCursor<'_>,
) -> Vec<String> {
    let mut retained = Vec::new();
    declaration_values(
        declarations
            .iter()
            .map(|declaration| (declaration.property_name(), Some(declaration))),
        &mut retained,
        importance_capability,
        includes_semantic_value,
        frozen,
    );
    retained
}

fn declaration_values<'a>(
    declarations: impl Iterator<Item = (CssPropertyNameRef<'a>, Option<&'a CssDeclaration>)>,
    retained: &mut Vec<String>,
    importance_capability: &str,
    includes_semantic_value: bool,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    for (name, declaration) in declarations {
        let id = property_id(name);
        retained.push(format!("property:{id}"));
        if let Some(declaration) = declaration {
            let (semantic, authored) = frozen.next(includes_semantic_value);
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
            if let Some(semantic) = semantic {
                assert_eq!(semantic.id, id, "{} semantic identity", frozen.case_id);
                assert_eq!(
                    semantic.importance, importance,
                    "{} {id} semantic importance",
                    frozen.case_id
                );
            }
            assert_declaration_value(
                name,
                declaration.custom(),
                declaration.known(),
                semantic,
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
    semantic: Option<FrozenSemanticValue<'_>>,
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
            if let Some(semantic) = semantic {
                assert_eq!(
                    semantic.payload,
                    value.as_css(),
                    "{} {} frozen custom-property payload",
                    frozen.case_id,
                    authored.id
                );
            }
        } else {
            let keyword = custom.value().global().expect("symbolic custom global");
            assert_eq!(
                authored.value_capability, "deferred-i01",
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
            if let Some(semantic) = semantic {
                assert_eq!(
                    semantic.payload,
                    custom_global_semantic_payload(keyword),
                    "{} {} frozen custom-global payload",
                    frozen.case_id,
                    authored.id
                );
            }
        }
        return;
    }

    let Some(known) = known else {
        assert_eq!(
            authored.value_capability, "deferred-i01",
            "{} {} future authored-value capability",
            frozen.case_id, authored.id
        );
        assert_eq!(
            authored.value, "<unavailable>",
            "{} {} future authored slice",
            frozen.case_id, authored.id
        );
        if let Some(semantic) = semantic {
            assert_eq!(
                semantic.payload, "future",
                "{} {} future semantic payload",
                frozen.case_id, authored.id
            );
        }
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
            assert_known_property_value(known.property(), value, semantic, authored, frozen);
        }
        surgeist_css::CssKnownDeclaredValueRef::Global(value) => {
            assert_eq!(
                authored.value_capability, "deferred-i01",
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
            if let Some(semantic) = semantic {
                assert_eq!(
                    semantic.payload,
                    known_global_semantic_payload(value),
                    "{} {} frozen global payload",
                    frozen.case_id,
                    authored.id
                );
            }
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
            if let Some(semantic) = semantic {
                assert_eq!(
                    semantic.payload,
                    format!("substitution:{}", value.as_css()),
                    "{} {} frozen substitution payload",
                    frozen.case_id,
                    authored.id
                );
            }
        }
        _ => {
            assert_eq!(
                authored.value_capability, "deferred-i01",
                "{} {} future authored-value capability",
                frozen.case_id, authored.id
            );
            assert_eq!(
                authored.value, "<unavailable>",
                "{} {} future authored slice",
                frozen.case_id, authored.id
            );
            if let Some(semantic) = semantic {
                assert_eq!(
                    semantic.payload, "future",
                    "{} {} future semantic payload",
                    frozen.case_id, authored.id
                );
            }
        }
    }
}

// The immutable TSV's `typed:Rgba(...)` and aggregate Debug payloads recorded
// an I01 projection. They remain archival observations, not expected current
// output. These assertions replace that projection with exact authored branches
// and checked components, while declaration identity and authored text continue
// to be checked against the capture below.
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
            assert_eq!(actual.serialize_specified().unwrap(), expected);
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
    assert_eq!(repeat.count().integer().numeric().representation(), "2");
    let [RepeatMember::TrackSize(size)] = repeat.content().components() else {
        panic!("one repeated minmax")
    };
    let (min, max) = size.minmax().expect("captured minmax");
    assert_eq!(
        min.length_percentage()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "10px"
    );
    assert_eq!(max.flex().unwrap().serialize_specified().unwrap(), "1fr");
}

// The captured font witnesses have one fixed literal/generic list. Their old
// Debug payloads remain archived evidence, while the production API now exposes
// only the current model. Verify both the original record and its current
// meaning explicitly instead of restoring a lossy production I01 projection.
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
    captured_payload: &str,
    semantic: Option<FrozenSemanticValue<'_>>,
    authored: &AuthoredDeclaration<'_>,
    case_id: &str,
) {
    assert_eq!(
        authored.id, id,
        "{case_id}: captured font property identity"
    );
    assert_eq!(authored.value_capability, "deferred-i01");
    assert_ne!(authored.value, "<unavailable>");
    assert_eq!(
        current_css, authored.value,
        "{case_id}: captured authored font value"
    );
    if let Some(semantic) = semantic {
        assert_eq!(semantic.id, id);
        assert_eq!(
            semantic.payload, captured_payload,
            "{case_id}: original captured font payload"
        );
    }
}

// The six archived calculation witnesses predate exact lexical math trees.
// Keep their captured payloads: independently render the historical text schema,
// and check the current tree's operators, exact leaves, types and source spans.
fn assert_captured_sum_calculation(
    calculation: &surgeist_css::CssLengthPercentageCalculation,
    expected_css: &str,
    first: (&str, bool),
    second: (&str, bool),
    subtract: bool,
    source: &str,
) -> String {
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
    let mut old_terms = Vec::new();
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
        // Independent historical schema, derived from the explicit operand
        // expectations above. No fixture payload is read to build this text.
        let literal = number.parse::<f32>().unwrap();
        let variant = if percentage { "Percent" } else { "Px" };
        let operator = if index == 1 && subtract {
            "Subtract"
        } else {
            "Add"
        };
        old_terms.push(format!(
            "CssCalcLengthTerm {{ operator: {operator}, value: {variant}(CssFiniteNumber {{ value: {literal:?} }}) }}"
        ));
    }
    format!("Calc(Sum([{}]))", old_terms.join(", "))
}

fn assert_captured_numeric_metadata(
    id: &str,
    css: &str,
    old: &str,
    semantic: Option<FrozenSemanticValue<'_>>,
    authored: &AuthoredDeclaration<'_>,
) {
    assert_eq!(authored.id, id);
    assert_eq!(authored.value_capability, "deferred-i01");
    assert_eq!(css, authored.value);
    if let Some(semantic) = semantic {
        assert_eq!(semantic.id, id);
        assert_eq!(semantic.payload, format!("typed:{old}"));
    }
}

// This frozen I01 corpus records the former CssLength Debug payload. These
// cases have an explicitly checked equivalent in the new sizing domain; keep
// the historical expectation while inspecting the exact current value.
fn frozen_box_size_payload(value: &surgeist_css::CssBoxSize) -> String {
    use surgeist_css::{CssBoxSize, CssComponentValueRef, CssValueTokenRef};
    match value {
        CssBoxSize::LengthPercentage(length) => {
            let component = length.literal_component().expect("frozen ordinary literal");
            match component.view() {
                CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                    if number.representation() == "0" =>
                {
                    "Zero".into()
                }
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                    if unit.eq_ignore_ascii_case("px") =>
                {
                    let literal = number.representation();
                    assert!(matches!(literal, "1" | "2" | "3"));
                    let exact = literal.parse::<u8>().unwrap();
                    format!("Px(CssFiniteNumber {{ value: {exact}.0 }})")
                }
                _ => panic!("frozen sizing literal changed its unit or value"),
            }
        }
        CssBoxSize::MinContent => "MinContent".into(),
        CssBoxSize::MaxContent => "MaxContent".into(),
        CssBoxSize::FitContent => "FitContent".into(),
        _ => panic!("frozen sizing keyword changed its branch"),
    }
}

fn frozen_preferred_size_payload(value: &surgeist_css::CssSizeValue) -> String {
    match value {
        surgeist_css::CssSizeValue::Auto => "Auto".into(),
        surgeist_css::CssSizeValue::BoxSize(value) => frozen_box_size_payload(value),
        _ => panic!("frozen sizing value changed branch"),
    }
}

fn assert_frozen_spacing_literal(
    component: &surgeist_css::CssComponentValue,
    expected: &str,
) -> &'static str {
    use surgeist_css::{CssComponentValueRef, CssValueTokenRef};
    match (expected, component.view()) {
        ("0", CssComponentValueRef::Token(CssValueTokenRef::Number(number)))
            if number.representation() == "0" =>
        {
            "Zero"
        }
        ("1px", CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }))
            if number.representation() == "1" && unit.eq_ignore_ascii_case("px") =>
        {
            "Px(CssFiniteNumber { value: 1.0 })"
        }
        ("10px", CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }))
            if number.representation() == "10" && unit.eq_ignore_ascii_case("px") =>
        {
            "Px(CssFiniteNumber { value: 10.0 })"
        }
        ("12px", CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }))
            if number.representation() == "12" && unit.eq_ignore_ascii_case("px") =>
        {
            "Px(CssFiniteNumber { value: 12.0 })"
        }
        ("2%", CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)))
            if number.representation() == "2" =>
        {
            "Percent(CssFiniteNumber { value: 2.0 })"
        }
        ("5%", CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)))
            if number.representation() == "5" =>
        {
            "Percent(CssFiniteNumber { value: 5.0 })"
        }
        _ => panic!("frozen spacing literal changed numeric representation or unit: {expected}"),
    }
}

fn frozen_margin_payload(value: &surgeist_css::CssMarginValue, expected: &str) -> String {
    match (expected, value) {
        ("auto", surgeist_css::CssMarginValue::Auto) => "Auto".into(),
        (_, surgeist_css::CssMarginValue::LengthPercentage(value)) => {
            assert_frozen_spacing_literal(value.literal_component().unwrap(), expected).into()
        }
        _ => panic!("frozen margin branch changed: {expected}"),
    }
}

fn frozen_padding_payload(value: &surgeist_css::CssPaddingValue, expected: &str) -> String {
    assert_frozen_spacing_literal(
        value.length_percentage().literal_component().unwrap(),
        expected,
    )
    .into()
}

fn assert_known_property_value(
    property: surgeist_css::CssKnownProperty,
    value: surgeist_css::CssKnownPropertyValueRef<'_>,
    semantic: Option<FrozenSemanticValue<'_>>,
    authored: &AuthoredDeclaration<'_>,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    use surgeist_css::{
        CssBoxShadow, CssFilter, CssFilterAmount, CssFilterFunction, CssOutlineStyle,
        CssOutlineWidth, CssTextDecorationLineComponent, CssTextDecorationStyle,
        CssTextDecorationThickness,
    };
    // Captured scalar inputs now use the sole checked lexical owners. Keep the
    // archive immutable and assert its concrete semantics without recreating Debug.
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
            assert_eq!(value.value().serialize_specified().unwrap(), expected);
            if let Some(semantic) = semantic {
                let captured = match expected {
                    "0.5" => "typed:CssOpacity { value: CssFiniteNumber { value: 0.5 } }",
                    "1" => "typed:CssOpacity { value: CssFiniteNumber { value: 1.0 } }",
                    _ => panic!("unexpected captured semantic opacity"),
                };
                assert_eq!(semantic.payload, captured);
            }
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
            assert_eq!(value.value().serialize_specified().unwrap(), "-2");
            if let Some(semantic) = semantic {
                assert_eq!(semantic.payload, "typed:Integer(-2)");
            }
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
            if let Some(semantic) = semantic {
                assert_eq!(semantic.payload, "typed:Integer(-2)");
            }
            Some(value.as_css())
        }
        _ => None,
    };
    if let Some(css) = scalar_authored {
        assert_eq!(authored.id, property.stable_id());
        assert_eq!(authored.value_capability, "deferred-i01");
        assert_eq!(css, authored.value);
        if let Some(semantic) = semantic {
            assert_eq!(semantic.id, property.stable_id());
        }
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
            assert!(exact_number((scale.x()).literal_component(), "1.5"));
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
        assert_eq!(authored.value_capability, "deferred-i01");
        assert_eq!(css, authored.value);
        assert!(semantic.is_some_and(|item| item.id == property.stable_id()));
        return;
    }
    match (property, &value) {
        (
            surgeist_css::CssKnownProperty::ListStyleType,
            surgeist_css::CssKnownPropertyValueRef::ListStyleType(value),
        ) if frozen.case_id == "catalog.property.baseline.property.list-style-type.boundary" => {
            assert_eq!(authored.id, property.stable_id());
            assert_eq!(authored.value_capability, "public");
            assert_eq!(authored.value, "symbols(cyclic \"*\" \"+\")");
            assert_eq!(value.as_css(), authored.value);

            assert_eq!(
                value.value().serialize_specified().unwrap(),
                "symbols(cyclic \"*\" \"+\")"
            );
            let semantic = semantic.expect("new current list-style-type value");
            assert_eq!(semantic.id, property.stable_id());
            assert_eq!(semantic.payload, "typed:symbols(cyclic \"*\" \"+\")");
            return;
        }
        (
            surgeist_css::CssKnownProperty::ListStyle,
            surgeist_css::CssKnownPropertyValueRef::ListStyle(value),
        ) if frozen.case_id == "catalog.property.baseline.property.list-style.boundary" => {
            assert_eq!(authored.id, property.stable_id());
            assert_eq!(authored.value_capability, "public");
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
                value.value().serialize_specified().unwrap(),
                "inside outside"
            );
            let semantic = semantic.expect("new current list-style value");
            assert_eq!(semantic.id, property.stable_id());
            assert_eq!(semantic.payload, "typed:inside outside");
            return;
        }
        // The archived gap witnesses carried a single CssLength Debug payload.
        // Keep that historical observation in this fixture adapter while the
        // current property API exposes exact one/two-value gap models.
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
            let old = assert_frozen_spacing_literal(
                length.literal_component().expect("captured gap literal"),
                "12px",
            );
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::RowGap,
            surgeist_css::CssKnownPropertyValueRef::RowGap(value),
        ) => {
            assert_eq!(value.value(), &surgeist_css::CssGapValue::Normal);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "Normal",
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::ColumnGap,
            surgeist_css::CssKnownPropertyValueRef::ColumnGap(value),
        ) => {
            let surgeist_css::CssGapValue::LengthPercentage(length) = value.value() else {
                panic!("captured column gap is an exact numeric value")
            };
            let old = assert_frozen_spacing_literal(
                length
                    .literal_component()
                    .expect("captured column-gap literal"),
                "5%",
            );
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                old,
                semantic,
                authored,
            );
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
            let old = assert_captured_sum_calculation(
                length.calculation().expect("frozen width calculation"),
                "calc(100% - 12px)",
                ("100", true),
                ("12", false),
                true,
                frozen.input,
            );
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }

        (
            surgeist_css::CssKnownProperty::Width,
            surgeist_css::CssKnownPropertyValueRef::Width(value),
        ) => {
            let payload = frozen_preferred_size_payload(value.value());
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &payload,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::Height,
            surgeist_css::CssKnownPropertyValueRef::Height(value),
        ) => {
            let payload = frozen_preferred_size_payload(value.value());
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &payload,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MinWidth,
            surgeist_css::CssKnownPropertyValueRef::MinWidth(value),
        ) => {
            let payload = frozen_preferred_size_payload(value.value());
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &payload,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MinHeight,
            surgeist_css::CssKnownPropertyValueRef::MinHeight(value),
        ) => {
            let payload = frozen_preferred_size_payload(value.value());
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &payload,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MaxWidth,
            surgeist_css::CssKnownPropertyValueRef::MaxWidth(value),
        ) => {
            let payload = value
                .value()
                .box_size()
                .map(frozen_box_size_payload)
                .unwrap_or_else(|| "None".into());
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &payload,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MaxHeight,
            surgeist_css::CssKnownPropertyValueRef::MaxHeight(value),
        ) => {
            let payload = value
                .value()
                .box_size()
                .map(frozen_box_size_payload)
                .unwrap_or_else(|| "None".into());
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &payload,
                semantic,
                authored,
            );
            return;
        }

        (
            surgeist_css::CssKnownProperty::Left,
            surgeist_css::CssKnownPropertyValueRef::Left(value),
        ) if authored.value == "calc(3px + 4%)" => {
            let old = assert_captured_sum_calculation(
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }

        (
            surgeist_css::CssKnownProperty::MarginLeft,
            surgeist_css::CssKnownPropertyValueRef::MarginLeft(value),
        ) if authored.value == "calc(3px + 4%)" => {
            let old = assert_captured_sum_calculation(
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }

        (
            surgeist_css::CssKnownProperty::PaddingBottom,
            surgeist_css::CssKnownPropertyValueRef::PaddingBottom(value),
        ) if authored.value == "calc(3px + 4%)" => {
            let old = assert_captured_sum_calculation(
                value.value().length_percentage().calculation().unwrap(),
                "calc(3px + 4%)",
                ("3", false),
                ("4", true),
                false,
                frozen.input,
            );
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }

        (
            surgeist_css::CssKnownProperty::BorderBottomLeftRadius,
            surgeist_css::CssKnownPropertyValueRef::BorderBottomLeftRadius(value),
        ) if authored.value == "calc(1px + 2%)" => {
            let current = value.value();
            let horizontal = assert_captured_sum_calculation(
                current.horizontal().calculation().unwrap(),
                "calc(1px + 2%)",
                ("1", false),
                ("2", true),
                false,
                frozen.input,
            );
            let vertical = assert_captured_sum_calculation(
                current.vertical().calculation().unwrap(),
                "calc(1px + 2%)",
                ("1", false),
                ("2", true),
                false,
                frozen.input,
            );
            let old =
                format!("CssCornerRadius {{ horizontal: {horizontal}, vertical: {vertical} }}");
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::Padding,
            surgeist_css::CssKnownPropertyValueRef::Padding(value),
        ) if authored.value == "1px 2% calc(3px + 4%) 0" => {
            let current = value.value();
            assert_eq!(current.authored_values().len(), 4);
            let [top, right, bottom, left] = current.assigned_values();
            assert_frozen_spacing_literal(
                top.length_percentage().literal_component().unwrap(),
                "1px",
            );
            assert_frozen_spacing_literal(
                right.length_percentage().literal_component().unwrap(),
                "2%",
            );
            assert_frozen_spacing_literal(
                left.length_percentage().literal_component().unwrap(),
                "0",
            );
            let bottom = assert_captured_sum_calculation(
                bottom.length_percentage().calculation().unwrap(),
                "calc(3px + 4%)",
                ("3", false),
                ("4", true),
                false,
                frozen.input,
            );
            let old = format!(
                "CssEdges {{ top: Px(CssFiniteNumber {{ value: 1.0 }}), right: Percent(CssFiniteNumber {{ value: 2.0 }}), bottom: {bottom}, left: Zero }}"
            );
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
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
            let top = frozen_margin_payload(top, "auto");
            let right = frozen_margin_payload(right, "10px");
            let bottom = frozen_margin_payload(bottom, "5%");
            let left = frozen_margin_payload(left, "10px");
            let old = format!(
                "CssEdges {{ top: {top}, right: {right}, bottom: {bottom}, left: {left} }}"
            );
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginTop,
            surgeist_css::CssKnownPropertyValueRef::MarginTop(value),
        ) => {
            let old = frozen_margin_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginRight,
            surgeist_css::CssKnownPropertyValueRef::MarginRight(value),
        ) => {
            let old = frozen_margin_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginBottom,
            surgeist_css::CssKnownPropertyValueRef::MarginBottom(value),
        ) => {
            let old = frozen_margin_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::MarginLeft,
            surgeist_css::CssKnownPropertyValueRef::MarginLeft(value),
        ) => {
            let old = frozen_margin_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingTop,
            surgeist_css::CssKnownPropertyValueRef::PaddingTop(value),
        ) => {
            let old = frozen_padding_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingRight,
            surgeist_css::CssKnownPropertyValueRef::PaddingRight(value),
        ) => {
            let old = frozen_padding_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingBottom,
            surgeist_css::CssKnownPropertyValueRef::PaddingBottom(value),
        ) => {
            let old = frozen_padding_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::PaddingLeft,
            surgeist_css::CssKnownPropertyValueRef::PaddingLeft(value),
        ) => {
            let old = frozen_padding_payload(value.value(), authored.value);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                &old,
                semantic,
                authored,
            );
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
            assert_eq!(authored.value_capability, "public");
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
            assert_eq!(value.value().serialize_specified().unwrap(), "0.1em");
            if let Some(semantic) = semantic {
                assert_eq!(semantic.id, property.stable_id());
                assert_eq!(semantic.payload, "typed:LengthPercentage(0.1em)");
            }
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "Px(CssFiniteNumber { value: 16.0 })",
                semantic,
                authored,
            );
            return;
        }
        (
            surgeist_css::CssKnownProperty::LineHeight,
            surgeist_css::CssKnownPropertyValueRef::LineHeight(value),
        ) => {
            assert_eq!(value.line_height(), &surgeist_css::CssLineHeight::Normal);
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "Normal",
                semantic,
                authored,
            );
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
                r#"typed:CssFontFamilyList { families: [CssFontFamilyName { kind: Quoted, value: "Avenir Next" }, CssFontFamilyName { kind: IdentSequence, value: "sans-serif" }] }"#,
                semantic,
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "Italic",
                semantic,
                authored,
            );
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
            assert_eq!(number.serialize_specified().unwrap(), "725");
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "Number(CssFontWeightNumber { value: 725 })",
                semantic,
                authored,
            );
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
                r#"typed:CssFont { style: Some(Italic), variant: Some(SmallCaps), weight: Some(Number(CssFontWeightNumber { value: 700 })), stretch: Some(Condensed), size: Px(CssFiniteNumber { value: 16.0 }), line_height: Some(Normal), families: CssFontFamilyList { families: [CssFontFamilyName { kind: Quoted, value: "Avenir Next" }, CssFontFamilyName { kind: IdentSequence, value: "sans-serif" }] } }"#,
                semantic,
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "SemiCondensed",
                semantic,
                authored,
            );
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "SmallCaps",
                semantic,
                authored,
            );
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
            assert_captured_numeric_metadata(
                property.stable_id(),
                value.as_css(),
                "Features(CssFontFeatureList { features: [CssFontFeature { tag: \"kern\", value: Some(On) }, CssFontFeature { tag: \"liga\", value: Some(Integer(0)) }] })",
                semantic,
                authored,
            );
            return;
        }
        _ => {}
    }
    // Frozen timing Debug payloads are archival observations. Assert the selected
    // typed semantics of each captured ordinary timing declaration directly.
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
        assert_eq!(authored.value_capability, "deferred-i01");
        assert_eq!(css, authored.value);
        assert!(semantic.is_some_and(|item| item.id == property.stable_id()));
        return;
    }
    // The archive keeps the old Debug payload. Assert each captured Grid value
    // against the authored model's branch and independently fixed constituents.
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
                    .serialize_specified()
                    .unwrap(),
                "100px"
            );
            assert_eq!(
                flex.breadth()
                    .unwrap()
                    .flex()
                    .unwrap()
                    .serialize_specified()
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
                    .serialize_specified()
                    .unwrap(),
                "100px"
            );
            assert_eq!(
                flex.breadth()
                    .unwrap()
                    .flex()
                    .unwrap()
                    .serialize_specified()
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
                    .serialize_specified()
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
                size.fit_content().unwrap().serialize_specified().unwrap(),
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
            assert_eq!(value.value().serialize_specified().unwrap(), "span 2 main");
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
            assert_eq!(value.value().serialize_specified().unwrap(), "4");
            Some(value.as_css())
        }
        (
            surgeist_css::CssKnownProperty::GridRow,
            surgeist_css::CssKnownPropertyValueRef::GridRow(value),
        ) => {
            let range = value.value();
            assert!(matches!(range.start(), GridLine::Indexed(_)));
            assert!(matches!(range.authored_end(), Some(GridLine::Span(_))));
            assert_eq!(range.serialize_specified().unwrap(), "1 / span 2");
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
                area.serialize_specified().unwrap(),
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
                    .serialize_specified()
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
                    .serialize_specified()
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
        assert_eq!(authored.value_capability, "deferred-i01");
        assert_eq!(css, authored.value);
        assert!(semantic.is_some_and(|item| item.id == property.stable_id()));
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
        assert_eq!(authored.value_capability, "deferred-i01");
        assert_eq!(css, authored.value);
        assert!(semantic.is_some_and(|item| item.id == property.stable_id()));
        return;
    }
    // The immutable archive records the former URL/none Debug representation.
    // Compare its captured inputs with the sole authored image model explicitly.
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
        assert_eq!(authored.value_capability, "deferred-i01");
        assert_eq!(css, authored.value);
        assert!(semantic.is_some_and(|item| item.id == property.stable_id()));
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
    // Decode the selected immutable historical payloads into concrete expectations.
    // No live Debug rendering or production compatibility graph participates.
    use surgeist_css::*;
    match (property, value) {
        (CssKnownProperty::Display, CssKnownPropertyValueRef::Display(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Block"),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "BorderBox"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "BorderBox" => CssBoxSizing::BorderBox,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Position, CssKnownPropertyValueRef::Position(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Sticky"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Sticky" => CssLayoutPosition::Sticky,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Direction, CssKnownPropertyValueRef::Direction(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Rtl"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Rtl" => CssDirection::Rtl,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Overflow, CssKnownPropertyValueRef::Overflow(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Pair(CssOverflowAxes { x: Hidden, y: Scroll })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(typed.x(), CssOverflow::Hidden);
            assert_eq!(typed.authored_y(), Some(CssOverflow::Scroll));
        }
        (CssKnownProperty::OverflowX, CssKnownPropertyValueRef::OverflowX(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Clip"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Clip" => CssOverflow::Clip,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::OverflowY, CssKnownPropertyValueRef::OverflowY(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Visible"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Visible" => CssOverflow::Visible,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::FlexDirection, CssKnownPropertyValueRef::FlexDirection(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "ColumnReverse"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "ColumnReverse" => CssFlexDirection::ColumnReverse,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::FlexWrap, CssKnownPropertyValueRef::FlexWrap(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "WrapReverse"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "WrapReverse" => CssFlexWrap::WrapReverse,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Float, CssKnownPropertyValueRef::Float(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Left"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Left" => CssFloat::Left,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Clear, CssKnownPropertyValueRef::Clear(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Both"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Both" => CssClear::Both,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::AlignContent, CssKnownPropertyValueRef::AlignContent(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "SpaceBetween"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(typed.value(), CssAlignmentValue::SpaceBetween);
        }
        (CssKnownProperty::JustifyContent, CssKnownPropertyValueRef::JustifyContent(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "SafeCenter"),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "FirstBaseline"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(
                typed.value(),
                CssAlignmentValue::Baseline(CssBaselinePosition::First)
            );
        }
        (CssKnownProperty::AlignSelf, CssKnownPropertyValueRef::AlignSelf(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "SafeFlexEnd"),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Stretch"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(typed.value(), CssAlignmentValue::Stretch);
        }
        (CssKnownProperty::JustifySelf, CssKnownPropertyValueRef::JustifySelf(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Center"),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Content(CssPlaceContentAlignment { first: Center, second: End })"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Items(CssPlaceItemsAlignment { first: Stretch, second: Stretch })"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(typed.align().value(), CssAlignmentValue::Stretch);
            assert_eq!(typed.justify().value(), CssAlignmentValue::Stretch);
        }
        (CssKnownProperty::PlaceSelf, CssKnownPropertyValueRef::PlaceSelf(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Items(CssPlaceItemsAlignment { first: End, second: Center })"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Collapse"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Collapse" => CssVisibility::Collapse,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Content, CssKnownPropertyValueRef::Content(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Items(CssContentList { items: [String(CssContentString { value: \"Chapter \" })] })"
                        | "Items(CssContentList { items: [String(CssContentString { value: \"x\" })] })"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let CssContentValue::Generated(generated) = typed else {
                panic!("captured generated string");
            };
            assert!(generated.alternative().is_none());
            let [CssContentValueItem::String(text)] = generated.items() else {
                panic!("one captured string");
            };
            let decoded: String = serde_json::from_str(
                expected
                    .split("value: ")
                    .nth(1)
                    .unwrap()
                    .split(" }")
                    .next()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(text.as_str(), decoded);
        }
        (
            CssKnownProperty::ContentVisibility,
            CssKnownPropertyValueRef::ContentVisibility(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Auto"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Auto" => CssContentVisibility::Auto,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::ListStyleType, CssKnownPropertyValueRef::ListStyleType(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "CounterStyle(BuiltIn(Square))"),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Inside"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Inside" => CssListStylePosition::Inside,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::ListStyleImage, CssKnownPropertyValueRef::ListStyleImage(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Url(CssUrl { value: \"marker.svg\" })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert!(matches!(typed, CssImageValue::Url(url)
                if url.as_str() == "marker.svg"
                    && url.function() == surgeist_css::CssUrlFunction::Url
                    && url.modifiers().is_empty()));
        }
        (CssKnownProperty::ListStyle, CssKnownPropertyValueRef::ListStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssListStyle { style_type: Some(CounterStyle(BuiltIn(Square))), position: Some(Inside), image: Some(Url(CssUrl { value: \"marker.svg\" })) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Changes(CssCounterChangeList { changes: [CssCounterChange { name: CssCounterName { name: \"section\" }, value: Some(2) }] })"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Changes(CssCounterChangeList { changes: [CssCounterChange { name: CssCounterName { name: \"section\" }, value: Some(1) }] })"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Changes(CssCounterChangeList { changes: [CssCounterChange { name: CssCounterName { name: \"section\" }, value: Some(3) }] })"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Dimension(CssLengthDimension { value: CssFiniteNumber { value: 10.0 }, unit: Rem })"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_basis(typed);
        }
        (CssKnownProperty::WritingMode, CssKnownPropertyValueRef::WritingMode(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "VerticalRl"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "VerticalRl" => CssWritingMode::VerticalRl,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::TextAlign, CssKnownPropertyValueRef::TextAlign(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Start"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(
                typed,
                &CssTextAlignValue::Alignment(CssTextAlignAllValue::Keyword(CssTextAlign::Start))
            );
        }
        (CssKnownProperty::TextAlignLast, CssKnownPropertyValueRef::TextAlignLast(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Justify"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(
                typed,
                &CssTextAlignLastValue::Keyword(CssTextAlign::Justify)
            );
        }
        (CssKnownProperty::TextIndent, CssKnownPropertyValueRef::TextIndent(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssTextIndent { length: Dimension(CssLengthDimension { value: CssFiniteNumber { value: 1.0 }, unit: Rem }), hanging: true, each_line: true }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Super"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Super" => CssVerticalAlign::Super,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::TextWrap, CssKnownPropertyValueRef::TextWrap(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Balance"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Balance" => CssTextWrap::Balance,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::WhiteSpace, CssKnownPropertyValueRef::WhiteSpace(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "PreWrap"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "PreWrap" => CssWhiteSpace::PreWrap,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::WordBreak, CssKnownPropertyValueRef::WordBreak(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "KeepAll"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "KeepAll" => CssWordBreak::KeepAll,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::OverflowWrap, CssKnownPropertyValueRef::OverflowWrap(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Anywhere"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Anywhere" => CssOverflowWrap::Anywhere,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::TextOverflow, CssKnownPropertyValueRef::TextOverflow(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Ellipsis"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Ellipsis" => CssTextOverflow::Ellipsis,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (
            CssKnownProperty::TextDecorationLine,
            CssKnownPropertyValueRef::TextDecorationLine(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssTextDecorationLine { components: [Underline, Overline], none: false }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Wavy"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Wavy" => CssTextDecorationStyle::Wavy,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (
            CssKnownProperty::TextDecorationThickness,
            CssKnownPropertyValueRef::TextDecorationThickness(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Length(CssTextDecorationThicknessLength { length: Px(CssFiniteNumber { value: 2.0 }) })"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let CssTextDecorationThickness::Length(length) = typed else {
                panic!("captured thickness");
            };
            assert_captured_px(length.literal_component(), "2");
        }
        (CssKnownProperty::TextTransform, CssKnownPropertyValueRef::TextTransform(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Uppercase"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Uppercase" => CssTextTransform::Uppercase,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Inset, CssKnownPropertyValueRef::Inset(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssEdges { top: Auto, right: Px(CssFiniteNumber { value: 10.0 }), bottom: Percent(CssFiniteNumber { value: 5.0 }), left: Px(CssFiniteNumber { value: 10.0 }) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Auto"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(typed, &CssInsetValue::Auto);
        }
        (CssKnownProperty::Right, CssKnownPropertyValueRef::Right(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Px(CssFiniteNumber { value: 10.0 })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_inset(typed, "10", Some("px"));
        }
        (CssKnownProperty::Bottom, CssKnownPropertyValueRef::Bottom(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Percent(CssFiniteNumber { value: 5.0 })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_inset(typed, "5", Some("%"));
        }
        (
            CssKnownProperty::BoxDecorationBreak,
            CssKnownPropertyValueRef::BoxDecorationBreak(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Clone"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Clone" => CssBoxDecorationBreak::Clone,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderWidth, CssKnownPropertyValueRef::BorderWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssEdges { top: Px(CssFiniteNumber { value: 1.0 }), right: Px(CssFiniteNumber { value: 2.0 }), bottom: Px(CssFiniteNumber { value: 3.0 }), left: Px(CssFiniteNumber { value: 4.0 }) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Px(CssFiniteNumber { value: 1.0 })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_width(typed, "1");
        }
        (CssKnownProperty::BorderRightWidth, CssKnownPropertyValueRef::BorderRightWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Px(CssFiniteNumber { value: 2.0 })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_width(typed, "2");
        }
        (
            CssKnownProperty::BorderBottomWidth,
            CssKnownPropertyValueRef::BorderBottomWidth(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Px(CssFiniteNumber { value: 3.0 })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_width(typed, "3");
        }
        (CssKnownProperty::BorderLeftWidth, CssKnownPropertyValueRef::BorderLeftWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Px(CssFiniteNumber { value: 4.0 })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_archive_width(typed, "4");
        }
        (CssKnownProperty::BorderStyle, CssKnownPropertyValueRef::BorderStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssBorderStyles { top: None, right: Hidden, bottom: Dotted, left: Dashed }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Solid"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Solid" => CssBorderStyle::Solid,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderRightStyle, CssKnownPropertyValueRef::BorderRightStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Double"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Double" => CssBorderStyle::Double,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (
            CssKnownProperty::BorderBottomStyle,
            CssKnownPropertyValueRef::BorderBottomStyle(value),
        ) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Ridge"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Ridge" => CssBorderStyle::Ridge,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderLeftStyle, CssKnownPropertyValueRef::BorderLeftStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Outset"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Outset" => CssBorderStyle::Outset,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::BorderRadius, CssKnownPropertyValueRef::BorderRadius(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssBorderRadii { top_left: CssCornerRadius { horizontal: Px(CssFiniteNumber { value: 1.0 }), vertical: Px(CssFiniteNumber { value: 4.0 }) }, top_right: CssCornerRadius { horizontal: Px(CssFiniteNumber { value: 2.0 }), vertical: Px(CssFiniteNumber { value: 5.0 }) }, bottom_right: CssCornerRadius { horizontal: Px(CssFiniteNumber { value: 3.0 }), vertical: Px(CssFiniteNumber { value: 4.0 }) }, bottom_left: CssCornerRadius { horizontal: Px(CssFiniteNumber { value: 2.0 }), vertical: Px(CssFiniteNumber { value: 5.0 }) } }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssCornerRadius { horizontal: Px(CssFiniteNumber { value: 4.0 }), vertical: Percent(CssFiniteNumber { value: 10.0 }) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssCornerRadius { horizontal: Px(CssFiniteNumber { value: 1.0 }), vertical: Px(CssFiniteNumber { value: 1.0 }) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssCornerRadius { horizontal: Percent(CssFiniteNumber { value: 10.0 }), vertical: Percent(CssFiniteNumber { value: 10.0 }) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssFlexFactor { value: CssNonNegativeNumber { value: CssFiniteNumber { value: 2.0 } } }"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.factor();
            assert_archive_literal(typed.literal_component().unwrap(), "2", None);
        }
        (CssKnownProperty::FlexShrink, CssKnownPropertyValueRef::FlexShrink(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssFlexFactor { value: CssNonNegativeNumber { value: CssFiniteNumber { value: 0.0 } } }"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.factor();
            assert_archive_literal(typed.literal_component().unwrap(), "0", None);
        }
        (CssKnownProperty::Flex, CssKnownPropertyValueRef::Flex(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Components { grow: CssFlexFactor { value: CssNonNegativeNumber { value: CssFiniteNumber { value: 2.0 } } }, shrink: Some(CssFlexFactor { value: CssNonNegativeNumber { value: CssFiniteNumber { value: 0.0 } } }), basis: Some(Dimension(CssLengthDimension { value: CssFiniteNumber { value: 10.0 }, unit: Rem })) }"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "CssAspectRatio { value: CssFiniteNumber { value: 1.5 } }"
                ),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.ratio();
            let CssAspectRatioValue::Ratio(ratio) = typed else {
                panic!("captured ratio");
            };
            assert!(ratio.denominator().is_none());
            assert_archive_literal(ratio.numerator().literal_component().unwrap(), "1.5", None);
        }
        (CssKnownProperty::ScrollbarWidth, CssKnownPropertyValueRef::ScrollbarWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Thin"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Thin" => CssScrollbarWidth::Thin,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::Cursor, CssKnownPropertyValueRef::Cursor(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Keyword(Grab)"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            assert_eq!(typed, &CssCursor::Keyword(CssCursorKeyword::Grab));
        }
        (CssKnownProperty::PointerEvents, CssKnownPropertyValueRef::PointerEvents(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "None"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "None" => CssPointerEvents::None,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::UserSelect, CssKnownPropertyValueRef::UserSelect(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Text"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Text" => CssUserSelect::Text,
                _ => panic!("unknown captured keyword"),
            };
            assert_eq!(typed, &decoded);
        }
        (CssKnownProperty::OutlineWidth, CssKnownPropertyValueRef::OutlineWidth(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Length(Px(CssFiniteNumber { value: 2.0 }))"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let CssOutlineWidth::Length(length) = typed else {
                panic!("captured outline width");
            };
            assert_captured_px(length.literal_component(), "2");
        }
        (CssKnownProperty::Translate, CssKnownPropertyValueRef::Translate(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(
                    expected,
                    "Values(CssTranslateValues { values: [Px(CssFiniteNumber { value: 10.0 }), Px(CssFiniteNumber { value: 20.0 })] })"
                ),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Value(\"45deg\")"),
                "unknown archived {property:?} expectation: {expected}"
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
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Values(CssScaleValues { values: [1.5, 2.0] })"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let CssScale::Values(values) = typed else {
                panic!("captured scale");
            };
            assert_eq!(values.values().len(), 2);
            assert!(exact_number(values.values()[0].literal_component(), "1.5"));
            assert!(exact_number(values.values()[1].literal_component(), "2"));
        }
        (CssKnownProperty::OutlineStyle, CssKnownPropertyValueRef::OutlineStyle(value)) => {
            assert_archive_wrapper(property, value.as_css(), semantic, authored);
            let expected = semantic
                .expect("captured semantic value")
                .payload
                .strip_prefix("typed:")
                .unwrap();
            assert!(
                matches!(expected, "Auto"),
                "unknown archived {property:?} expectation: {expected}"
            );
            let typed = value.value();
            let decoded = match expected {
                "Auto" => CssOutlineStyle::Auto,
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

fn custom_global_semantic_payload(keyword: surgeist_css::CssGlobalKeyword) -> &'static str {
    match keyword {
        surgeist_css::CssGlobalKeyword::Inherit => "global:Some(Inherit)",
        surgeist_css::CssGlobalKeyword::Initial => "global:Some(Initial)",
        surgeist_css::CssGlobalKeyword::Unset => "global:Some(Unset)",
        surgeist_css::CssGlobalKeyword::Revert => "global:Some(Revert)",
        surgeist_css::CssGlobalKeyword::RevertLayer => "global:Some(RevertLayer)",
        _ => panic!("future CSS global keyword lacks a frozen custom-global payload"),
    }
}

fn known_global_semantic_payload(keyword: surgeist_css::CssGlobalKeyword) -> &'static str {
    match keyword {
        surgeist_css::CssGlobalKeyword::Inherit => "global:Inherit",
        surgeist_css::CssGlobalKeyword::Initial => "global:Initial",
        surgeist_css::CssGlobalKeyword::Unset => "global:Unset",
        surgeist_css::CssGlobalKeyword::Revert => "global:Revert",
        surgeist_css::CssGlobalKeyword::RevertLayer => "global:RevertLayer",
        _ => panic!("future CSS global keyword lacks a frozen known-global payload"),
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
        root_and_payload(error.kind()),
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
        surgeist_css::CssTokenKind::IncludeMatch => "IncludeMatch",
        surgeist_css::CssTokenKind::DashMatch => "DashMatch",
        surgeist_css::CssTokenKind::PrefixMatch => "PrefixMatch",
        surgeist_css::CssTokenKind::SuffixMatch => "SuffixMatch",
        surgeist_css::CssTokenKind::SubstringMatch => "SubstringMatch",
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

fn root_and_payload(kind: &ErrorKind) -> String {
    match kind {
        ErrorKind::UnexpectedEnd(detail) => {
            format!("UnexpectedEnd:{}", detail.expectation().as_str())
        }
        ErrorKind::UnexpectedToken(detail) => format!(
            "UnexpectedToken:{}:{}",
            detail.expectation().as_str(),
            token(Some(detail.encountered()))
        ),
        ErrorKind::InvalidEncodingDeclaration(detail) => format!(
            "InvalidEncodingDeclaration:{}:{}",
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::InvalidAtRulePlacement(detail) => format!(
            "InvalidAtRulePlacement:{}:{}",
            detail.name().as_str(),
            detail.expected_context().as_str()
        ),
        ErrorKind::InvalidAtRulePrelude(detail) => format!(
            "InvalidAtRulePrelude:{}:{}:{}:{}",
            detail.name().as_str(),
            detail.production().as_str(),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::InvalidAtRuleBody(detail) => format!(
            "InvalidAtRuleBody:{}:{}:{}:{}",
            detail.name().as_str(),
            detail.production().as_str(),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::UnknownAtRule(detail) => format!("UnknownAtRule:{}", detail.name().as_str()),
        ErrorKind::UnsupportedAtRule(detail) => format!(
            "UnsupportedAtRule:{}:{}",
            detail.name().as_str(),
            detail.feature().as_str()
        ),
        ErrorKind::InvalidQualifiedRule(detail) => format!(
            "InvalidQualifiedRule:{}:{}:{}",
            detail.production().as_str(),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::InvalidSelector(detail) => format!(
            "InvalidSelector:{}:{}:{}",
            detail.production().map_or("-", |value| value.as_str()),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::InvalidMediaQuery(detail) => format!(
            "InvalidMediaQuery:{}:{}:{}",
            detail.feature().map_or("-", |value| value.as_str()),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::UnknownProperty(detail) => format!("UnknownProperty:{}", detail.name().as_str()),
        ErrorKind::UnsupportedProperty(detail) => format!(
            "UnsupportedProperty:{}:{}",
            detail.name().as_str(),
            detail.feature().as_str()
        ),
        ErrorKind::InvalidPropertyValue(detail) => format!(
            "InvalidPropertyValue:{}:{}:{}",
            detail.property().stable_id(),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::InvalidDeclarationAnnotation(detail) => format!(
            "InvalidDeclarationAnnotation:{}:{}",
            declaration_context(detail.context()),
            token(Some(detail.encountered()))
        ),
        ErrorKind::UnknownDescriptor(detail) => format!(
            "UnknownDescriptor:{}:{}",
            detail.at_rule().as_str(),
            detail.descriptor().as_str()
        ),
        ErrorKind::UnsupportedDescriptor(detail) => format!(
            "UnsupportedDescriptor:{}:{}:{}",
            detail.at_rule().as_str(),
            detail.descriptor().as_str(),
            detail.feature().as_str()
        ),
        ErrorKind::InvalidDescriptorValue(detail) => format!(
            "InvalidDescriptorValue:{}:{}:{}:{}",
            detail.at_rule().as_str(),
            detail.descriptor().as_str(),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::InvalidDescriptorCombination(detail) => format!(
            "InvalidDescriptorCombination:{}:{}:{}",
            detail.at_rule().as_str(),
            detail.responsible().as_str(),
            detail
                .conflicting()
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ),
        ErrorKind::InvalidColorSyntax(detail) => format!(
            "InvalidColorSyntax:{}:{}:{}",
            detail.component().map_or("-", |value| value.as_str()),
            detail.expectation().as_str(),
            token(detail.encountered())
        ),
        ErrorKind::NestingLimit(detail) => format!(
            "NestingLimit:{}:{}",
            detail.limit(),
            detail.enclosing_production().as_str()
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
        CssErrorCode::InvalidEncodingDeclaration => "InvalidEncodingDeclaration",
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
        CssRecoveryAction::IgnoreLegacyToken => "IgnoreLegacyToken",
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
    semantic: Option<FrozenSemanticValue<'_>>,
    authored: &AuthoredDeclaration<'_>,
) {
    assert_eq!(authored.id, property.stable_id());
    assert_eq!(authored.value_capability, "deferred-i01");
    assert_ne!(authored.value, "<unavailable>");
    assert_eq!(css, authored.value);
    assert_eq!(semantic.unwrap().id, property.stable_id());
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
