//! Finite real-browser qualification of Layout's source-qualified root capture.
//! Run explicitly with the existing layout-golden-generate feature and cached browser.

#[path = "../tests/bin/surgeist-layout-generate/adapter.rs"]
mod adapter;
#[path = "../tests/bin/surgeist-layout-generate/browser_profile.rs"]
mod browser_profile;
#[path = "../tests/bin/surgeist-layout-generate/envelope.rs"]
mod envelope;
#[path = "../tests/bin/surgeist-layout-generate/measurement.rs"]
mod measurement;
#[path = "../tests/layout/browser_parity/support.rs"]
pub mod parity_support;
#[path = "../tests/layout/browser_parity/pinned_font.rs"]
mod pinned_font;
#[path = "qualify-root-context/support.rs"]
mod support;
#[path = "../tests/bin/surgeist-layout-generate/xml.rs"]
mod xml;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(result) = surgeist_generator::browser::run_supervisor_from_env() {
        return result.map_err(Into::into);
    }
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let (fixture, mode) = parse_arguments(&arguments)?;
    support::run(fixture, mode)
}

fn parse_arguments(
    arguments: &[String],
) -> Result<(Option<&str>, support::QualificationMode), &'static str> {
    use support::QualificationMode;
    match arguments {
        [] => Ok((None, QualificationMode::Regular)),
        [flag, source] if flag == "--fixture" => {
            Ok((Some(source.as_str()), QualificationMode::Regular))
        }
        [mode, flag, source] if mode == "--stylesheet-custody" && flag == "--fixture" => {
            Ok((Some(source.as_str()), QualificationMode::StylesheetCustody))
        }
        [mode, flag, source] if mode == "--numeric-diagnostics" && flag == "--fixture" => {
            Ok((Some(source.as_str()), QualificationMode::NumericDiagnostics))
        }
        _ => Err(
            "usage: qualify-root-context [--fixture <declared-source>] | --stylesheet-custody --fixture <declared-source> | --numeric-diagnostics --fixture <declared-source>",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use support::QualificationMode;

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn numeric_diagnostics_cli_requires_one_explicit_declared_fixture() {
        let args = arguments(&["--numeric-diagnostics", "--fixture", "html/grid/case.html"]);
        assert_eq!(
            parse_arguments(&args),
            Ok((
                Some("html/grid/case.html"),
                QualificationMode::NumericDiagnostics
            ))
        );
        for values in [
            vec!["--numeric-diagnostics"],
            vec!["--fixture", "html/grid/case.html", "--numeric-diagnostics"],
            vec![
                "--numeric-diagnostics",
                "--stylesheet-custody",
                "--fixture",
                "html/grid/case.html",
            ],
        ] {
            assert!(parse_arguments(&arguments(&values)).is_err());
        }
    }

    #[test]
    fn numeric_diagnostics_cli_preserves_regular_and_stylesheet_modes() {
        assert_eq!(parse_arguments(&[]), Ok((None, QualificationMode::Regular)));
        let args = arguments(&["--fixture", "html/grid/case.html"]);
        assert_eq!(
            parse_arguments(&args),
            Ok((Some("html/grid/case.html"), QualificationMode::Regular))
        );
        let args = arguments(&["--stylesheet-custody", "--fixture", "html/grid/case.html"]);
        assert_eq!(
            parse_arguments(&args),
            Ok((
                Some("html/grid/case.html"),
                QualificationMode::StylesheetCustody
            ))
        );
    }
}
