use std::collections::BTreeSet;

use surgeist_css::{
    CssDeclaration, CssDeclarationContextRef, CssErrorCode, CssImportance, CssPropertyNameRef,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssScopedRule, ErrorKind, parse_sheet,
    parse_style_attribute,
};

const FIXTURE: &str = include_str!("fixtures/i01-c01-observables.tsv");
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
    semantic_values: Vec<FrozenSemanticValue<'a>>,
    authored_declarations: Vec<AuthoredDeclaration<'a>>,
    semantic_index: usize,
    authored_index: usize,
}

impl<'a> FrozenDeclarationCursor<'a> {
    fn new(row: &'a Row) -> Self {
        Self {
            case_id: &row.case_id,
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
fn authored_css_cases_match_frozen_public_report_observables() {
    let rows = parse_fixture(FIXTURE).expect("valid I01 observable fixture");
    for row in rows {
        let applicable = match row.feature.as_str() {
            "both" => true,
            "default" => !cfg!(feature = "app-strict"),
            "app-strict" => cfg!(feature = "app-strict"),
            _ => unreachable!("validated fixture feature"),
        };
        if !applicable {
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
        #[cfg(feature = "app-strict")]
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
    let repeated_index = rows
        .iter()
        .position(|row| row.diagnostics.split_once('~').is_some())
        .expect("fixture contains an input with multiple recovery diagnostics");
    let mut repeated = rows.clone();
    repeated[repeated_index].diagnostics = repeated[repeated_index]
        .diagnostics
        .split_once('~')
        .expect("repeated diagnostic")
        .0
        .to_owned();
    assert_ne!(
        observe(&repeated[repeated_index]).diagnostics,
        repeated[repeated_index].diagnostics,
        "{} public parser retains every recovery diagnostic in source order",
        repeated[repeated_index].case_id
    );
    assert_eq!(
        observe(&rows[repeated_index]).diagnostics,
        rows[repeated_index].diagnostics,
        "{} public report matches the complete authored diagnostic sequence",
        rows[repeated_index].case_id
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
        CssScopedRule::Style(rule) => {
            retained.push("rule:baseline.rule.style".to_owned());
            let ids =
                declaration_observables(rule.declarations().as_slice(), "public", true, frozen);
            retained.extend(ids);
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

macro_rules! assert_property_specific_value {
    (
        $property:expr,
        $value:expr,
        $semantic:expr,
        $authored:expr,
        $frozen:expr;
        $($variant:ident,)*
    ) => {
        match ($property, $value) {
            $(
                (
                    surgeist_css::CssKnownProperty::$variant,
                    surgeist_css::CssKnownPropertyValueRef::$variant(value),
                ) => {
                    let expected_id =
                        surgeist_css::CssKnownProperty::$variant.stable_id();
                    assert_eq!(
                        $authored.id, expected_id,
                        "{} {} property-specific authored identity",
                        $frozen.case_id, expected_id
                    );
                    assert_eq!(
                        $authored.value_capability, "deferred-i01",
                        "{} {} authored-value capability",
                        $frozen.case_id, expected_id
                    );
                    assert_ne!(
                        $authored.value, "<unavailable>",
                        "{} {} deferred slice must remain explicit in the TSV",
                        $frozen.case_id, expected_id
                    );
                    assert_eq!(
                        value.as_css(),
                        $authored.value,
                        "{} {} concrete wrapper authored slice",
                        $frozen.case_id, expected_id
                    );
                    let typed = value.i01_subset().unwrap_or_else(|| {
                        panic!(
                            "{}: {} concrete wrapper lacks its typed I01 payload",
                            $frozen.case_id, expected_id
                        )
                    });
                    if let Some(semantic) = $semantic {
                        assert_eq!(
                            semantic.id, expected_id,
                            "{} {} property-specific semantic identity",
                            $frozen.case_id, expected_id
                        );
                        assert_eq!(
                            semantic.payload,
                            format!("typed:{typed:?}"),
                            "{} {} frozen I01 public Debug payload",
                            $frozen.case_id, expected_id
                        );
                    }
                }
            )*
            _ => panic!(
                "{}: known property identity and concrete value wrapper disagree",
                $frozen.case_id
            ),
        }
    };
}

fn assert_known_property_value(
    property: surgeist_css::CssKnownProperty,
    value: surgeist_css::CssKnownPropertyValueRef<'_>,
    semantic: Option<FrozenSemanticValue<'_>>,
    authored: &AuthoredDeclaration<'_>,
    frozen: &mut FrozenDeclarationCursor<'_>,
) {
    assert_property_specific_value!(
        property,
        value,
        semantic,
        authored,
        frozen;
            All,
            Display,
            BoxSizing,
            Position,
            Direction,
            Overflow,
            OverflowX,
            OverflowY,
            FlexDirection,
            FlexWrap,
            Float,
            Clear,
            AlignContent,
            JustifyContent,
            AlignItems,
            AlignSelf,
            JustifyItems,
            JustifySelf,
            PlaceContent,
            PlaceItems,
            PlaceSelf,
            Visibility,
            Content,
            ContentVisibility,
            ListStyleType,
            ListStylePosition,
            ListStyleImage,
            ListStyle,
            CounterReset,
            CounterIncrement,
            CounterSet,
            Width,
            Height,
            MinWidth,
            MinHeight,
            MaxWidth,
            MaxHeight,
            FlexBasis,
            Gap,
            RowGap,
            ColumnGap,
            GridFlowTolerance,
            GridTemplateRows,
            GridTemplateColumns,
            GridTemplateAreas,
            GridTemplate,
            GridAutoRows,
            GridAutoColumns,
            GridAutoFlow,
            GridRowStart,
            GridRowEnd,
            GridColumnStart,
            GridColumnEnd,
            GridRow,
            GridColumn,
            GridArea,
            Grid,
            FontSize,
            LineHeight,
            WritingMode,
            TextAlign,
            TextAlignLast,
            TextIndent,
            VerticalAlign,
            FontFamily,
            Font,
            FontWeight,
            FontStyle,
            FontStretch,
            FontVariant,
            FontFeatureSettings,
            LetterSpacing,
            TextWrap,
            WhiteSpace,
            WordBreak,
            OverflowWrap,
            TextOverflow,
            TextDecoration,
            TextDecorationLine,
            TextDecorationColor,
            TextDecorationStyle,
            TextDecorationThickness,
            TextTransform,
            Inset,
            Top,
            Right,
            Bottom,
            Left,
            ZIndex,
            BoxDecorationBreak,
            Margin,
            MarginTop,
            MarginRight,
            MarginBottom,
            MarginLeft,
            Padding,
            PaddingTop,
            PaddingRight,
            PaddingBottom,
            PaddingLeft,
            Border,
            BorderTop,
            BorderRight,
            BorderBottom,
            BorderLeft,
            BorderWidth,
            BorderTopWidth,
            BorderRightWidth,
            BorderBottomWidth,
            BorderLeftWidth,
            Color,
            Background,
            BackgroundColor,
            BorderColor,
            BorderTopColor,
            BorderRightColor,
            BorderBottomColor,
            BorderLeftColor,
            BackgroundImage,
            BackgroundPosition,
            BackgroundSize,
            BackgroundRepeat,
            BackgroundOrigin,
            BackgroundClip,
            BackgroundAttachment,
            BorderStyle,
            BorderTopStyle,
            BorderRightStyle,
            BorderBottomStyle,
            BorderLeftStyle,
            BorderRadius,
            BorderTopLeftRadius,
            BorderTopRightRadius,
            BorderBottomRightRadius,
            BorderBottomLeftRadius,
            BoxShadow,
            Opacity,
            FlexGrow,
            FlexShrink,
            Order,
            Flex,
            JustifyTracks,
            AlignTracks,
            AspectRatio,
            ScrollbarWidth,
            Cursor,
            PointerEvents,
            UserSelect,
            Outline,
            OutlineColor,
            OutlineStyle,
            OutlineWidth,
            Transform,
            TransformOrigin,
            Translate,
            Rotate,
            Scale,
            Filter,
            BackdropFilter,
            ClipPath,
            Mask,
            MaskImage,
            MaskSize,
            MaskPosition,
            MaskRepeat,
            TransitionProperty,
            TransitionDuration,
            TransitionDelay,
            TransitionTimingFunction,
            Transition,
            AnimationName,
            AnimationDuration,
            AnimationDelay,
            AnimationTimingFunction,
            AnimationIterationCount,
            AnimationDirection,
            AnimationFillMode,
            AnimationPlayState,
            Animation,
    );
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

#[cfg(feature = "app-strict")]
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
