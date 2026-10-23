//! Publication admission policy for native recovered parser resource failures.
use crate::{CssomError, CssomLimits};
use surgeist_css::{CssParseSource, CssParserContext, CssRecoveryDiagnostic};

pub(crate) fn check_parsed_source(
    source: Option<&CssParseSource>,
    diagnostics: &[CssRecoveryDiagnostic],
    context: CssParserContext,
    limits: &CssomLimits,
    missing: &'static str,
    mismatch: &'static str,
) -> Result<(), CssomError> {
    let source = source.ok_or(CssomError::InvalidInput(missing))?;
    if source.context() != context {
        return Err(CssomError::InvalidInput(mismatch));
    }
    if source.source().as_str().len() > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    check_resource_diagnostics(diagnostics)
}

/// Refuses publication of a parse that a nesting limit, an unavailable parser
/// thread or a component resource limit left incomplete.
pub(crate) fn check_resource_diagnostics(
    diagnostics: &[CssRecoveryDiagnostic],
) -> Result<(), CssomError> {
    for diagnostic in diagnostics {
        if diagnostic.error().is_resource_refusal() {
            return Err(CssomError::ParseResource(Box::new(diagnostic.clone())));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_css::{
        CssComponentValueError, CssComponentValueLimits, CssErrorCode, CssParserContext,
    };

    #[test]
    fn native_component_diagnostics_return_the_first_original_resource() {
        let context = CssParserContext::default();
        let mut resources = Vec::new();
        type Case = (
            &'static str,
            CssComponentValueLimits,
            fn(&CssComponentValueError) -> bool,
        );
        let cases: [Case; 3] = [
            (
                "width:f(f(1px));",
                CssComponentValueLimits::try_new(1, 100, 100).unwrap(),
                CssComponentValueError::is_nesting_limit,
            ),
            (
                "width:1px;",
                CssComponentValueLimits::try_new(10, 0, 100).unwrap(),
                CssComponentValueError::is_component_limit,
            ),
            (
                "width:1px;",
                CssComponentValueLimits::try_new(10, 100, 0).unwrap(),
                CssComponentValueError::is_byte_limit,
            ),
        ];
        for (source, limits, expected) in cases {
            let report = context.parse_declaration_block_contents_with_limits(source, limits);
            let diagnostic = report
                .diagnostics()
                .iter()
                .find(|diagnostic| diagnostic.error().component_error().is_some_and(expected))
                .expect("native driver reports the selected resource cause")
                .clone();
            resources.push(diagnostic);
        }
        let ordinary = context.parse_declaration_block_contents(
            "unknown-property:value; width:bogus; color:url(a b);",
        );
        assert!(!ordinary.diagnostics().is_empty());
        check_resource_diagnostics(ordinary.diagnostics()).unwrap();
        check_resource_diagnostics(&[]).unwrap();
        let mut diagnostics = ordinary.diagnostics().to_vec();
        diagnostics.extend(resources.iter().cloned());
        let Err(CssomError::ParseResource(first)) = check_resource_diagnostics(&diagnostics) else {
            panic!("resource admission must return the first native diagnostic");
        };
        // Equality includes the native error, original span and recovery action.
        assert_eq!(*first, resources[0]);
        resources.reverse();
        let Err(CssomError::ParseResource(first)) = check_resource_diagnostics(&resources) else {
            panic!("source encounter order determines the resource diagnostic");
        };
        assert_eq!(*first, resources[0]);
    }

    #[test]
    fn outer_parser_nesting_retains_native_grammar_and_recovery_provenance() {
        let source = format!(
            "{}.a {{ color:red }}{}",
            "@media all {".repeat(300),
            "}".repeat(300),
        );
        let report = CssParserContext::default().parse_sheet(&source);
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit)
            .expect("nested rendering groups reach the native grammar depth bound");
        let Err(CssomError::ParseResource(first)) =
            check_resource_diagnostics(std::slice::from_ref(diagnostic))
        else {
            panic!("outer nesting is a publication resource failure");
        };
        assert_eq!(*first, *diagnostic);
    }
}
