//! Report observables: retained structure identities, authored declarations
//! and recovery diagnostics, in the schema of the original report columns.

use crate::css::support::failure_classes::{ComponentFailure, component_failure};
use surgeist_css::{
    CssColorComponentName, CssDeclaration, CssDeclarationContextRef, CssError, CssErrorCode,
    CssEscapeFailure, CssGlobalKeyword, CssGrammarExpectation, CssImportance,
    CssKnownDeclaredValueRef, CssKnownProperty, CssProductionId, CssPropertyNameRef,
    CssPropertyValueError, CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssScopedRule,
    CssTokenKind, CssTokenSummary,
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
        kind_facts(error),
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

/// The facts of a stylesheet diagnostic or of a checked property-value
/// grammar failure, which reports the same facts without a position.
pub trait GrammarDetail {
    fn grammar_code(&self) -> Option<CssErrorCode>;
    fn expectation(&self) -> Option<CssGrammarExpectation>;
    fn encountered(&self) -> Option<&CssTokenSummary>;
    fn production(&self) -> Option<CssProductionId>;
    fn known_property(&self) -> Option<CssKnownProperty>;
    fn color_component(&self) -> Option<&CssColorComponentName>;
    fn nesting_limit(&self) -> Option<u32>;
    fn declaration_context(&self) -> Option<CssDeclarationContextRef<'_>>;
    fn component_kind(&self) -> Option<ComponentFailure>;
    /// The complete diagnostic, when there is one.
    fn diagnostic(&self) -> Option<&CssError>;
}

impl GrammarDetail for CssError {
    fn grammar_code(&self) -> Option<CssErrorCode> {
        Some(self.code())
    }
    fn expectation(&self) -> Option<CssGrammarExpectation> {
        self.expectation()
    }
    fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered()
    }
    fn production(&self) -> Option<CssProductionId> {
        self.production()
    }
    fn known_property(&self) -> Option<CssKnownProperty> {
        self.known_property()
    }
    fn color_component(&self) -> Option<&CssColorComponentName> {
        self.color_component()
    }
    fn nesting_limit(&self) -> Option<u32> {
        self.nesting_limit()
    }
    fn declaration_context(&self) -> Option<CssDeclarationContextRef<'_>> {
        self.declaration_context()
    }
    fn component_kind(&self) -> Option<ComponentFailure> {
        self.component_error().map(component_failure)
    }
    fn diagnostic(&self) -> Option<&CssError> {
        Some(self)
    }
}

impl GrammarDetail for CssPropertyValueError {
    fn grammar_code(&self) -> Option<CssErrorCode> {
        self.code()
    }
    fn expectation(&self) -> Option<CssGrammarExpectation> {
        self.expectation()
    }
    fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered()
    }
    fn production(&self) -> Option<CssProductionId> {
        self.production()
    }
    fn known_property(&self) -> Option<CssKnownProperty> {
        self.known_property()
    }
    fn color_component(&self) -> Option<&CssColorComponentName> {
        self.color_component()
    }
    fn nesting_limit(&self) -> Option<u32> {
        self.nesting_limit()
    }
    fn declaration_context(&self) -> Option<CssDeclarationContextRef<'_>> {
        self.declaration_context()
    }
    fn component_kind(&self) -> Option<ComponentFailure> {
        self.component_error().map(component_failure)
    }
    fn diagnostic(&self) -> Option<&CssError> {
        None
    }
}

/// The diagnostic kind and each populated fact, read through its accessors.
pub fn kind_facts(error: &dyn GrammarDetail) -> String {
    let expectation = || error.expectation().map_or("-", |value| value.as_str());
    let Some(code) = error.grammar_code() else {
        return "Future".to_owned();
    };
    // Facts only a complete diagnostic reports.
    let diagnostic = error.diagnostic();
    let at_rule = || {
        diagnostic
            .and_then(CssError::at_rule)
            .map_or("-", |value| value.as_str())
    };
    let descriptor = || {
        diagnostic
            .and_then(CssError::descriptor)
            .map_or("-", |value| value.as_str())
    };
    let feature = || {
        diagnostic
            .and_then(CssError::feature)
            .map_or("-", |value| value.as_str())
    };
    let property_name = || {
        diagnostic
            .and_then(CssError::property_name)
            .map_or("-", |value| value.as_str())
    };
    match code {
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
            error.production().map_or("-", |value| value.as_str()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidAtRuleBody => format!(
            "InvalidAtRuleBody:{}:{}:{}:{}",
            at_rule(),
            error.production().map_or("-", |value| value.as_str()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::UnknownAtRule => format!("UnknownAtRule:{}", at_rule()),
        CssErrorCode::UnsupportedAtRule => {
            format!("UnsupportedAtRule:{}:{}", at_rule(), feature())
        }
        CssErrorCode::InvalidQualifiedRule => format!(
            "InvalidQualifiedRule:{}:{}:{}",
            error.production().map_or("-", |value| value.as_str()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidSelector => format!(
            "InvalidSelector:{}:{}:{}",
            error.production().map_or("-", |value| value.as_str()),
            expectation(),
            token(error.encountered())
        ),
        CssErrorCode::InvalidMediaQuery => format!(
            "InvalidMediaQuery:{}:{}:{}",
            diagnostic
                .and_then(CssError::media_feature)
                .map_or("-", |value| value.as_str()),
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
            diagnostic
                .map(CssError::conflicting_descriptors)
                .unwrap_or_default()
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
            error.production().map_or("-", |value| value.as_str())
        ),
        CssErrorCode::ParserUnavailable => format!(
            "ParserUnavailable:{}:{}",
            error
                .nesting_limit()
                .map_or_else(|| "-".to_owned(), |limit| limit.to_string()),
            error.production().map_or("-", |value| value.as_str())
        ),
        CssErrorCode::EscapeParseError => format!(
            "EscapeParseError:{}",
            match diagnostic.and_then(CssError::escape_failure) {
                Some(CssEscapeFailure::EndOfInput) => "end-of-input",
                Some(CssEscapeFailure::Newline) => "newline",
                _ => "future",
            }
        ),
        CssErrorCode::InvalidComponentValue => format!(
            "InvalidComponentValue:{}",
            error.component_kind().map_or("-", component_error_kind)
        ),
        CssErrorCode::LegacyNavigationTarget => format!(
            "LegacyNavigationTarget:{}",
            diagnostic
                .and_then(CssError::navigation_target)
                .unwrap_or("-")
        ),
        CssErrorCode::NamespaceRedeclaration => {
            let previous = diagnostic.and_then(CssError::previous_position);
            format!(
                "NamespaceRedeclaration:{}:{}:{}:{}",
                diagnostic
                    .and_then(CssError::namespace_prefix)
                    .map_or("-", |prefix| prefix.as_str()),
                previous.map_or_else(|| "-".to_owned(), |p| p.byte_offset().value().to_string()),
                previous.map_or_else(|| "-".to_owned(), |p| p.line().value().to_string()),
                previous.map_or_else(|| "-".to_owned(), |p| p.column().value().to_string())
            )
        }
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
        CssErrorCode::ParserUnavailable => "ParserUnavailable",
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
pub fn component_error_kind(kind: ComponentFailure) -> &'static str {
    match kind {
        ComponentFailure::BadString => "BadString",
        ComponentFailure::BadUrl => "BadUrl",
        ComponentFailure::ByteLimit => "ByteLimit",
        ComponentFailure::CapacityOverflow => "CapacityOverflow",
        ComponentFailure::ComponentLimit => "ComponentLimit",
        ComponentFailure::EmptyInput => "EmptyInput",
        ComponentFailure::InvalidDimension => "InvalidDimension",
        ComponentFailure::InvalidFunction => "InvalidFunction",
        ComponentFailure::InvalidIdentifier => "InvalidIdentifier",
        ComponentFailure::InvalidNumber => "InvalidNumber",
        ComponentFailure::InvalidString => "InvalidString",
        ComponentFailure::InvalidToken => "InvalidToken",
        ComponentFailure::NestingLimit => "NestingLimit",
        ComponentFailure::TrailingInput => "TrailingInput",
        ComponentFailure::UnmatchedClosingDelimiter => "UnmatchedClosingDelimiter",
        ComponentFailure::UnserializableBoundary => "UnserializableBoundary",
        ComponentFailure::ParserUnavailable => "ParserUnavailable",
    }
}
