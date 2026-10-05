//! Private carrier preserving the selected query provider's typed failure.
use crate::{CssMediaCssomSerializationError, CssSpecifiedValueSerializationError};

#[derive(Debug)]
pub(crate) enum QueryRuleSerializationError {
    Value(CssSpecifiedValueSerializationError),
    Media(CssMediaCssomSerializationError),
}
impl From<CssSpecifiedValueSerializationError> for QueryRuleSerializationError {
    fn from(error: CssSpecifiedValueSerializationError) -> Self {
        Self::Value(error)
    }
}
impl From<CssMediaCssomSerializationError> for QueryRuleSerializationError {
    fn from(error: CssMediaCssomSerializationError) -> Self {
        Self::Media(error)
    }
}
impl From<crate::CssComponentValueError> for QueryRuleSerializationError {
    fn from(error: crate::CssComponentValueError) -> Self {
        Self::Value(crate::component_values::specified_component_error(error))
    }
}
impl From<crate::CssImportSerializationError> for QueryRuleSerializationError {
    fn from(error: crate::CssImportSerializationError) -> Self {
        match error {
            crate::CssImportSerializationError::Component(error) => error.into(),
            crate::CssImportSerializationError::Media(error) => {
                Self::Media(crate::media::bounded_error(error))
            }
            crate::CssImportSerializationError::InterpretationChanged { .. } => {
                Self::Value(CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
                ))
            }
        }
    }
}
impl From<crate::CssCustomMediaSerializationError> for QueryRuleSerializationError {
    fn from(error: crate::CssCustomMediaSerializationError) -> Self {
        match error {
            crate::CssCustomMediaSerializationError::Component(error) => error.into(),
            crate::CssCustomMediaSerializationError::Media(error) => {
                Self::Media(crate::media::bounded_error(error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssContainerPrelude, CssCustomMediaBody, CssNamespaceContext, CssRule,
        CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits, CssSupportsCondition,
        parse_component_values, parse_sheet,
        specified_serialization::SpecifiedSerializationContext,
    };

    #[test]
    fn suppressed_lexical_regions_charge_omitted_whitespace_without_allocating_text() {
        let condition = CssSupportsCondition::try_from_components(
            parse_component_values("  (display:grid)  ").unwrap(),
            &CssNamespaceContext::default(),
        )
        .unwrap();
        let prelude = CssContainerPrelude::try_from_components(
            parse_component_values("  (width>1px)  ").unwrap(),
        )
        .unwrap();
        let mut context = SpecifiedSerializationContext::new(Limits::new(14, 14, 0));
        context.replace_output_suppression(true);
        let mut output = String::new();
        // Each independently derived region is one aggregate + block + three
        // inner tokens + two root whitespace tokens = seven visited nodes.
        condition
            .append_specified(&mut context, &mut output)
            .unwrap();
        prelude.append_specified(&mut context, &mut output).unwrap();
        assert_eq!(output, "");
        assert_eq!(context.remaining_bytes(), 0);
        assert_eq!(
            context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn suppressed_import_projects_recovered_member_work_and_preserves_its_error_origin() {
        let report = parse_sheet("@import \"x\" screen,???;");
        let before = report.clone();
        let [CssRule::Import(rule)] = report.syntax().rules() else {
            panic!("import");
        };
        let rejected_origin = rule.media().unwrap().queries()[1].origin().clone();
        // Target1 + list1 + typed query1 + type component1 + Never1;
        // Never's selected `not all` contributes two additional projections.
        let mut context = SpecifiedSerializationContext::new(Limits::new(5, 7, 0));
        context.replace_output_suppression(true);
        let mut output = String::new();
        rule.append_specified(&mut context, &mut output).unwrap();
        assert!(output.is_empty());
        assert_eq!(
            context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        let mut context = SpecifiedSerializationContext::new(Limits::new(5, 6, 0));
        context.replace_output_suppression(true);
        let QueryRuleSerializationError::Media(error) = rule
            .append_specified(&mut context, &mut output)
            .unwrap_err()
        else {
            panic!("media error");
        };
        assert_eq!(error.origin(), &rejected_origin);
        assert!(
            matches!(error, CssMediaCssomSerializationError::Resource { error, .. } if error.kind() == Kind::ProjectionNodeLimit)
        );
        assert_eq!(report, before);
    }

    #[test]
    fn suppressed_custom_media_charges_boolean_or_recovered_provider_without_text() {
        for (source, input_nodes, projection_nodes) in [
            ("@custom-media --x true;", 2, 2),
            ("@custom-media --x false;", 2, 2),
            ("@custom-media --x ???;", 3, 5),
        ] {
            let report = parse_sheet(source);
            let [CssRule::CustomMedia(rule)] = report.syntax().rules() else {
                panic!("definition");
            };
            if let CssCustomMediaBody::Media(list) = rule.body() {
                assert!(matches!(list.queries(), [crate::CssMediaQuery::Never(_)]));
            }
            let mut context =
                SpecifiedSerializationContext::new(Limits::new(input_nodes, projection_nodes, 0));
            context.replace_output_suppression(true);
            let mut output = String::new();
            rule.append_specified(&mut context, &mut output).unwrap();
            assert!(output.is_empty());
            assert_eq!(
                context.charge_input(1).unwrap_err().kind(),
                Kind::InputNodeLimit
            );
            assert_eq!(
                context.charge_projection(1).unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
        }
    }

    #[test]
    fn import_probe_and_final_emission_share_remaining_bytes_without_double_charging() {
        let report = parse_sheet("@import \"x\" layer(theme) and (color);");
        let [CssRule::Import(rule)] = report.syntax().rules() else {
            panic!("import");
        };
        assert!(rule.layer().is_none() && rule.supports().is_none());
        let expected = rule.serialize().unwrap();
        let prefix = "prior ";
        let mut context = SpecifiedSerializationContext::new(Limits::new(
            128,
            128,
            prefix.len() + expected.as_css().len(),
        ));
        let mut output = String::new();
        context.append(&mut output, prefix).unwrap();
        rule.append_specified(&mut context, &mut output).unwrap();
        assert_eq!(output, format!("{prefix}{}", expected.as_css()));
        assert_eq!(context.remaining_bytes(), 0);
        let mut context = SpecifiedSerializationContext::new(Limits::new(
            128,
            128,
            prefix.len() + expected.as_css().len() - 1,
        ));
        let mut output = String::new();
        context.append(&mut output, prefix).unwrap();
        assert!(rule.append_specified(&mut context, &mut output).is_err());
        assert_eq!(output, prefix);
    }
}
