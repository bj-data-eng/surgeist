//! Report observables: retained structure identities, authored declarations
//! and recovery diagnostics, in the schema of the original report columns.

use surgeist_css::{
    CssComponentValueErrorKind, CssDeclaration, CssDeclarationContextRef, CssErrorCode,
    CssEscapeError, CssGlobalKeyword, CssImportance, CssKnownDeclaredValueRef, CssPropertyNameRef,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssScopedRule, CssTokenKind,
    CssTokenSummary, ErrorKind,
};

/// Retained identities and authored declarations collected in source order.
#[derive(Default)]
pub struct Retained {
    pub identities: Vec<String>,
    pub authored: Vec<String>,
}

impl Retained {
    pub fn rules(rules: &[CssRule]) -> Self {
        let mut retained = Self::default();
        for rule in rules {
            retained.rule(rule);
        }
        retained
    }

    pub fn declarations(declarations: &[CssDeclaration]) -> Self {
        let mut retained = Self::default();
        retained.declaration_list(declarations, "public");
        retained
    }

    pub fn scoped_rules(rules: &[CssScopedRule]) -> Self {
        let mut retained = Self::default();
        for rule in rules {
            retained.scoped_rule(rule);
        }
        retained
    }

    fn rule(&mut self, rule: &CssRule) {
        match rule {
            CssRule::Import(_) => self.identity("rule:baseline.rule.import"),
            CssRule::Namespace(_) => self.identity("rule:later.rule.namespace"),
            CssRule::CounterStyle(_) => self.identity("rule:later.rule.counter-style"),
            CssRule::Page(_) => self.identity("rule:later.rule.page"),
            CssRule::LayerStatement(_) => self.identity("rule:baseline.rule.layer-statement"),
            CssRule::LayerBlock(rule) => {
                self.identity("rule:baseline.rule.layer-block");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::FontFace(_) => self.identity("rule:baseline.rule.font-face"),
            CssRule::FontFeatureValues(_) => self.identity("rule:later.rule.font-feature-values"),
            CssRule::Keyframes(rule) => {
                self.identity("rule:baseline.rule.keyframes");
                for block in rule.blocks() {
                    let declarations = block
                        .declarations()
                        .iter()
                        .map(|declaration| declaration.source().clone())
                        .collect::<Vec<_>>();
                    self.declaration_list(&declarations, "keyframe-grammar");
                }
            }
            CssRule::Style(rule) => {
                self.identity("rule:baseline.rule.style");
                self.declaration_list(rule.declarations().as_slice(), "public");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::NestedDeclarations(rule) => {
                self.identity("rule:nested-declarations");
                self.declaration_list(rule.declarations().as_slice(), "public");
            }
            CssRule::Media(rule) => {
                self.identity("rule:baseline.rule.media");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::Supports(rule) => {
                self.identity("rule:baseline.rule.supports");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::Container(rule) => {
                self.identity("rule:baseline.rule.container");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::When(rule) => {
                self.identity("rule:when");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::Else(rule) => {
                self.identity("rule:else");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssRule::Scope(rule) => {
                self.identity("rule:baseline.rule.scope");
                for child in rule.rules().rules() {
                    self.scoped_rule(child);
                }
            }
            _ => self.identity("rule:future"),
        }
    }

    fn scoped_rule(&mut self, rule: &CssScopedRule) {
        match rule {
            CssScopedRule::NestedDeclarations(rule) => {
                self.rule(&CssRule::NestedDeclarations(rule.clone()))
            }
            CssScopedRule::CounterStyle(rule) => self.rule(&CssRule::CounterStyle(rule.clone())),
            CssScopedRule::FontFace(rule) => self.rule(&CssRule::FontFace(rule.clone())),
            CssScopedRule::Page(rule) => self.rule(&CssRule::Page(rule.clone())),
            CssScopedRule::Keyframes(rule) => self.rule(&CssRule::Keyframes(rule.clone())),
            CssScopedRule::FontFeatureValues(_) => {
                self.identity("rule:later.rule.font-feature-values")
            }
            CssScopedRule::Style(rule) => {
                self.identity("rule:baseline.rule.style");
                self.declaration_list(rule.declarations().as_slice(), "public");
                for child in rule.rules() {
                    self.rule(child);
                }
            }
            CssScopedRule::Media(rule) => {
                for child in rule.rules().rules() {
                    self.scoped_rule(child);
                }
            }
            CssScopedRule::Supports(rule) => {
                self.identity("rule:baseline.rule.supports");
                for child in rule.rules().rules() {
                    self.scoped_rule(child);
                }
            }
            CssScopedRule::Container(rule) => {
                for child in rule.rules().rules() {
                    self.scoped_rule(child);
                }
            }
            CssScopedRule::LayerStatement(_) => self.identity("rule:baseline.rule.layer-statement"),
            CssScopedRule::LayerBlock(rule) => {
                for child in rule.rules().rules() {
                    self.scoped_rule(child);
                }
            }
            CssScopedRule::Scope(rule) => {
                for child in rule.rules().rules() {
                    self.scoped_rule(child);
                }
            }
            _ => self.identity("rule:future"),
        }
    }

    fn identity(&mut self, identity: &str) {
        self.identities.push(identity.to_owned());
    }

    fn declaration_list(&mut self, declarations: &[CssDeclaration], importance_capability: &str) {
        for declaration in declarations {
            let id = property_id(declaration.property_name());
            self.identities.push(format!("property:{id}"));
            let importance = match declaration.importance() {
                CssImportance::Normal => "normal",
                CssImportance::Important => "important",
            };
            self.authored.push(format!(
                "{id}={}@{importance_capability}:{importance}",
                authored_value(declaration)
            ));
        }
    }
}

/// The value capability and authored value of one declaration.
fn authored_value(declaration: &CssDeclaration) -> String {
    if let Some(custom) = declaration.custom() {
        return match custom.value().value() {
            Some(value) => format!("public:{}", value.as_css()),
            None => format!(
                "typed-value:{}",
                custom
                    .value()
                    .global()
                    .map_or("<future-global>", global_keyword_css)
            ),
        };
    }
    let Some(known) = declaration.known() else {
        return "typed-value:<unavailable>".to_owned();
    };
    match known.declared_value() {
        CssKnownDeclaredValueRef::Property(_) => {
            format!("typed-value:{}", authored_slice(declaration))
        }
        CssKnownDeclaredValueRef::Global(keyword) => {
            format!("typed-value:{}", global_keyword_css(keyword))
        }
        CssKnownDeclaredValueRef::SubstitutionDependent(value) => {
            format!("public:{}", value.as_css())
        }
        _ => "typed-value:<unavailable>".to_owned(),
    }
}

/// The authored source text of a declaration's value, without the CSS
/// whitespace that surrounds it.
fn authored_slice(declaration: &CssDeclaration) -> String {
    let Some(origin) = declaration.parsed_value() else {
        return "<unavailable>".to_owned();
    };
    let span = origin.span();
    origin
        .source()
        .as_str()
        .get(span.start().byte_offset().value()..span.end().byte_offset().value())
        .unwrap_or("<unavailable>")
        .trim_matches([' ', '\t', '\n', '\r', '\u{c}'])
        .to_owned()
}

pub fn property_id(name: CssPropertyNameRef<'_>) -> String {
    match name {
        CssPropertyNameRef::Known(property) => property.stable_id().to_owned(),
        CssPropertyNameRef::Custom(name) => format!("custom:{}", name.as_str()),
        CssPropertyNameRef::SvgGlyphOrientationVertical => {
            "svg:glyph-orientation-vertical".to_owned()
        }
        _ => "future-property".to_owned(),
    }
}

fn global_keyword_css(keyword: CssGlobalKeyword) -> &'static str {
    match keyword {
        CssGlobalKeyword::Inherit => "inherit",
        CssGlobalKeyword::Initial => "initial",
        CssGlobalKeyword::Unset => "unset",
        CssGlobalKeyword::Revert => "revert",
        CssGlobalKeyword::RevertLayer => "revert-layer",
        _ => "<future-global>",
    }
}

pub fn diagnostics(diagnostics: &[CssRecoveryDiagnostic]) -> Vec<String> {
    diagnostics.iter().map(diagnostic).collect()
}

/// Code, kind facts, recovery action, position and span of one diagnostic.
fn diagnostic(diagnostic: &CssRecoveryDiagnostic) -> String {
    let error = diagnostic.error();
    let position = error.position();
    let span = diagnostic.span();
    format!(
        "{}/{}/{}@{}:{}:{}>{}:{}:{}-{}:{}:{}:{}",
        code_name(error.code()),
        kind_facts(error.kind()),
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

pub fn token(token: Option<&CssTokenSummary>) -> String {
    token.map_or_else(
        || "-".to_owned(),
        |token| format!("{}:{}", token_kind_name(token.kind()), token.authored()),
    )
}

fn token_kind_name(kind: CssTokenKind) -> &'static str {
    match kind {
        CssTokenKind::Ident => "Ident",
        CssTokenKind::AtKeyword => "AtKeyword",
        CssTokenKind::Hash => "Hash",
        CssTokenKind::IdHash => "IdHash",
        CssTokenKind::String => "String",
        CssTokenKind::Url => "Url",
        CssTokenKind::Delim => "Delim",
        CssTokenKind::Number => "Number",
        CssTokenKind::Percentage => "Percentage",
        CssTokenKind::Dimension => "Dimension",
        CssTokenKind::Whitespace => "Whitespace",
        CssTokenKind::Comment => "Comment",
        CssTokenKind::Colon => "Colon",
        CssTokenKind::Semicolon => "Semicolon",
        CssTokenKind::Comma => "Comma",
        CssTokenKind::Cdo => "Cdo",
        CssTokenKind::Cdc => "Cdc",
        CssTokenKind::Function => "Function",
        CssTokenKind::ParenthesisBlock => "ParenthesisBlock",
        CssTokenKind::SquareBracketBlock => "SquareBracketBlock",
        CssTokenKind::CurlyBracketBlock => "CurlyBracketBlock",
        CssTokenKind::BadUrl => "BadUrl",
        CssTokenKind::BadString => "BadString",
        CssTokenKind::CloseParenthesis => "CloseParenthesis",
        CssTokenKind::CloseSquareBracket => "CloseSquareBracket",
        CssTokenKind::CloseCurlyBracket => "CloseCurlyBracket",
        _ => "Future",
    }
}

/// The diagnostic kind and each populated fact, read through its accessors.
pub fn kind_facts(kind: &ErrorKind) -> String {
    match kind {
        ErrorKind::UnexpectedEnd(detail) => {
            format!("UnexpectedEnd:{}", detail.expectation().as_str())
        }
        ErrorKind::UnexpectedToken(detail) => format!(
            "UnexpectedToken:{}:{}",
            detail.expectation().as_str(),
            token(Some(detail.encountered()))
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
        ErrorKind::EscapeParseError(detail) => format!(
            "EscapeParseError:{}",
            match detail {
                CssEscapeError::EndOfInput => "end-of-input",
                CssEscapeError::Newline => "newline",
                _ => "future",
            }
        ),
        ErrorKind::InvalidComponentValue(detail) => format!(
            "InvalidComponentValue:{}",
            component_error_kind(detail.kind())
        ),
        ErrorKind::LegacyNavigationTarget(detail) => {
            format!("LegacyNavigationTarget:{}", detail.target())
        }
        ErrorKind::NamespaceRedeclaration(detail) => format!(
            "NamespaceRedeclaration:{}:{}:{}:{}",
            detail.prefix().map_or("-", |prefix| prefix.as_str()),
            detail.previous_position().byte_offset().value(),
            detail.previous_position().line().value(),
            detail.previous_position().column().value()
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

pub fn code_name(code: CssErrorCode) -> &'static str {
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
        CssErrorCode::EscapeParseError => "EscapeParseError",
        CssErrorCode::InvalidComponentValue => "InvalidComponentValue",
        CssErrorCode::LegacyNavigationTarget => "LegacyNavigationTarget",
        CssErrorCode::NamespaceRedeclaration => "NamespaceRedeclaration",
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
        CssRecoveryAction::DropFontSourceListItem => "DropFontSourceListItem",
        CssRecoveryAction::IgnoreUnterminatedComment => "IgnoreUnterminatedComment",
        CssRecoveryAction::PreserveInvalidSelectorListItem => "PreserveInvalidSelectorListItem",
        CssRecoveryAction::RecoverEscape => "RecoverEscape",
        CssRecoveryAction::RejectInput => "RejectInput",
        CssRecoveryAction::RetainLegacyNavigationTarget => "RetainLegacyNavigationTarget",
        CssRecoveryAction::RetainNonconformingRule => "RetainNonconformingRule",
        CssRecoveryAction::RetainSyntaxCandidate => "RetainSyntaxCandidate",
        _ => "Future",
    }
}

/// The component-value error kind, by name.
pub fn component_error_kind(kind: CssComponentValueErrorKind) -> &'static str {
    match kind {
        CssComponentValueErrorKind::BadString => "BadString",
        CssComponentValueErrorKind::BadUrl => "BadUrl",
        CssComponentValueErrorKind::ByteLimit => "ByteLimit",
        CssComponentValueErrorKind::CapacityOverflow => "CapacityOverflow",
        CssComponentValueErrorKind::ComponentLimit => "ComponentLimit",
        CssComponentValueErrorKind::EmptyInput => "EmptyInput",
        CssComponentValueErrorKind::InvalidDimension => "InvalidDimension",
        CssComponentValueErrorKind::InvalidFunction => "InvalidFunction",
        CssComponentValueErrorKind::InvalidIdentifier => "InvalidIdentifier",
        CssComponentValueErrorKind::InvalidNumber => "InvalidNumber",
        CssComponentValueErrorKind::InvalidString => "InvalidString",
        CssComponentValueErrorKind::InvalidToken => "InvalidToken",
        CssComponentValueErrorKind::NestingLimit => "NestingLimit",
        CssComponentValueErrorKind::TrailingInput => "TrailingInput",
        CssComponentValueErrorKind::UnmatchedClosingDelimiter => "UnmatchedClosingDelimiter",
        CssComponentValueErrorKind::UnserializableBoundary => "UnserializableBoundary",
        _ => "Future",
    }
}
