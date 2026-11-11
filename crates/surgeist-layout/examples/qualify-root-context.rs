//! Finite real-browser qualification of Layout's source-qualified root capture.
//! Run explicitly with the existing layout-golden-generate feature and cached browser.

#[path = "../tests/bin/surgeist-layout-generate/adapter.rs"]
mod adapter;
#[path = "../tests/layout/browser_parity/atomic_paragraph_metrics.rs"]
mod atomic_paragraph_metrics;
#[path = "../tests/bin/surgeist-layout-generate/browser_profile.rs"]
mod browser_profile;
#[path = "../tests/bin/surgeist-layout-generate/measurement/capture_helper_identity.rs"]
mod capture_helper_identity;
#[path = "../tests/bin/surgeist-layout-generate/measurement/empty_line_br_font_band_profile.rs"]
mod empty_line_br_font_band_profile;
#[path = "../tests/bin/surgeist-layout-generate/envelope.rs"]
mod envelope;
#[path = "../tests/layout/browser_parity/fixture_variants.rs"]
mod fixture_variants;
#[path = "../tests/layout/browser_parity/grid_alignment_reference.rs"]
mod grid_alignment_reference;
#[path = "../tests/bin/surgeist-layout-generate/measurement/horizontal_br_font_band_profile.rs"]
mod horizontal_br_font_band_profile;
#[path = "../tests/bin/surgeist-layout-generate/measurement.rs"]
mod measurement;
#[path = "../tests/bin/surgeist-layout-generate/measurement/original_flex_parent_profile.rs"]
mod original_flex_parent_profile;
#[path = "../tests/bin/surgeist-layout-generate/measurement/original_subgrid_body_profile.rs"]
mod original_subgrid_body_profile;
#[path = "../tests/layout/browser_parity/support.rs"]
pub mod parity_support;
#[path = "../tests/layout/browser_parity/pinned_font.rs"]
mod pinned_font;
#[path = "../tests/bin/surgeist-layout-generate/measurement/prepared_atomic_percentage_profile.rs"]
mod prepared_atomic_percentage_profile;
#[path = "../tests/bin/surgeist-layout-generate/source_owner.rs"]
mod source_owner;
#[path = "qualify-root-context/support.rs"]
mod support;
#[path = "../tests/bin/surgeist-layout-generate/measurement/vertical_br_font_band_profile.rs"]
mod vertical_br_font_band_profile;
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
    use parity_support::capture_authority::SpecializedProfile;
    use parity_support::exact_differences::PublishedEdition;
    use support::QualificationMode;
    match arguments {
        [] => Ok((None, QualificationMode::Regular)),
        [mode] if mode == "--publish-grid-lanes-original-diagnostics" => {
            Ok((None, QualificationMode::PublishGridLanesDiagnostics))
        }
        [mode] if mode == "--check-grid-lanes-original-diagnostics" => {
            Ok((None, QualificationMode::CheckGridLanesDiagnostics))
        }
        [mode] if mode == "--publish-root-context-v4" => Ok((None, QualificationMode::Publish)),
        [mode] if mode == "--check-published-root-context-v4" => {
            Ok((None, QualificationMode::CheckPublished))
        }
        [mode] if mode == "--publish-canonical-box-differences" => Ok((
            None,
            QualificationMode::PublishExact(PublishedEdition::CanonicalBoxes),
        )),
        [mode] if mode == "--check-published-canonical-box-differences" => Ok((
            None,
            QualificationMode::CheckPublishedExact(PublishedEdition::CanonicalBoxes),
        )),
        [mode] if mode == "--publish-measured-grid-differences" => Ok((
            None,
            QualificationMode::PublishExact(PublishedEdition::MeasuredGrid),
        )),
        [mode] if mode == "--check-published-measured-grid-differences" => Ok((
            None,
            QualificationMode::CheckPublishedExact(PublishedEdition::MeasuredGrid),
        )),
        [mode] if mode == "--publish-flex-text-differences" => Ok((
            None,
            QualificationMode::PublishExact(PublishedEdition::FlexText),
        )),
        [mode] if mode == "--check-published-flex-text-differences" => Ok((
            None,
            QualificationMode::CheckPublishedExact(PublishedEdition::FlexText),
        )),
        [mode] if mode == "--publish-grid-static-position-differences" => Ok((
            None,
            QualificationMode::PublishExact(PublishedEdition::GridStaticPosition),
        )),
        [mode] if mode == "--check-published-grid-static-position-differences" => Ok((
            None,
            QualificationMode::CheckPublishedExact(PublishedEdition::GridStaticPosition),
        )),
        [mode] if mode == "--publish-relative-inset-differences" => Ok((
            None,
            QualificationMode::PublishExact(PublishedEdition::RelativeInsets),
        )),
        [mode] if mode == "--check-published-relative-inset-differences" => Ok((
            None,
            QualificationMode::CheckPublishedExact(PublishedEdition::RelativeInsets),
        )),
        [mode] if mode == "--publish-wrapping-world-corner-differences" => Ok((
            None,
            QualificationMode::PublishExact(PublishedEdition::WrappingWorldCorners),
        )),
        [mode] if mode == "--check-published-wrapping-world-corner-differences" => Ok((
            None,
            QualificationMode::CheckPublishedExact(PublishedEdition::WrappingWorldCorners),
        )),
        [mode] if mode == "--publish-prepared-range-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::PreparedRange),
        )),
        [mode] if mode == "--check-published-prepared-range-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::PreparedRange),
        )),
        [mode] if mode == "--publish-flex-composition-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::FlexComposition),
        )),
        [mode] if mode == "--check-published-flex-composition-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::FlexComposition),
        )),
        [mode] if mode == "--publish-original-flex-parent-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::OriginalFlexParent),
        )),
        [mode] if mode == "--check-published-original-flex-parent-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::OriginalFlexParent),
        )),
        [mode] if mode == "--publish-original-percentage-flex-parent-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::OriginalPercentageFlexParent),
        )),
        [mode] if mode == "--check-published-original-percentage-flex-parent-qualification" => {
            Ok((
                None,
                QualificationMode::CheckPublishedSpecialized(
                    SpecializedProfile::OriginalPercentageFlexParent,
                ),
            ))
        }
        [mode] if mode == "--publish-original-subgrid-body-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::OriginalSubgridBody),
        )),
        [mode] if mode == "--check-published-original-subgrid-body-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::OriginalSubgridBody),
        )),
        [mode] if mode == "--publish-vertical-br-font-band-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::VerticalBrFontBand),
        )),
        [mode] if mode == "--check-published-vertical-br-font-band-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::VerticalBrFontBand),
        )),
        [mode] if mode == "--publish-empty-line-br-font-band-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::EmptyLineBrFontBand),
        )),
        [mode] if mode == "--check-published-empty-line-br-font-band-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::EmptyLineBrFontBand),
        )),
        [mode] if mode == "--publish-horizontal-br-font-band-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::HorizontalBrFontBand),
        )),
        [mode] if mode == "--check-published-horizontal-br-font-band-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(SpecializedProfile::HorizontalBrFontBand),
        )),
        [mode] if mode == "--publish-prepared-atomic-percentage-qualification" => Ok((
            None,
            QualificationMode::PublishSpecialized(SpecializedProfile::PreparedAtomicPercentage),
        )),
        [mode] if mode == "--check-published-prepared-atomic-percentage-qualification" => Ok((
            None,
            QualificationMode::CheckPublishedSpecialized(
                SpecializedProfile::PreparedAtomicPercentage,
            ),
        )),
        [command, name] if command == "--derive-published-qualification" => Ok((
            None,
            support::named_derivation_mode(name).ok_or("unknown terminal qualification edition")?,
        )),
        [command, name]
            if command == "--publish-matching-geometry"
                || command == "--check-published-matching-geometry" =>
        {
            let edition = support::declared_matching_edition(name)
                .ok_or("unknown matching geometry edition")?;
            Ok((
                None,
                if command == "--publish-matching-geometry" {
                    QualificationMode::PublishMatching(edition)
                } else {
                    QualificationMode::CheckPublishedMatching(edition)
                },
            ))
        }
        [flag, source] if flag == "--fixture" && !source.starts_with("--") => {
            Ok((Some(source.as_str()), QualificationMode::Regular))
        }
        [mode, flag, source]
            if mode == "--prepared-range-capture"
                && flag == "--fixture"
                && !source.starts_with("--") =>
        {
            Ok((
                Some(source.as_str()),
                QualificationMode::PreparedRangeCapture,
            ))
        }
        [mode, flag, source] if mode == "--stylesheet-custody" && flag == "--fixture" => {
            Ok((Some(source.as_str()), QualificationMode::StylesheetCustody))
        }
        [mode, flag, source] if mode == "--font-custody" && flag == "--fixture" => {
            Ok((Some(source.as_str()), QualificationMode::FontCustody))
        }
        [mode, flag, source] if mode == "--numeric-diagnostics" && flag == "--fixture" => {
            Ok((Some(source.as_str()), QualificationMode::NumericDiagnostics))
        }
        [mode, flag, source]
            if mode == "--paired-geometry-capture"
                && flag == "--fixture"
                && !source.starts_with("--") =>
        {
            Ok((
                Some(source.as_str()),
                QualificationMode::PairedGeometryCapture,
            ))
        }
        [mode, flag, source]
            if mode == "--element-scroll-size-diagnostics"
                && flag == "--fixture"
                && !source.starts_with("--") =>
        {
            Ok((
                Some(source.as_str()),
                QualificationMode::ElementScrollSizeDiagnostics,
            ))
        }
        [mode, flag, source]
            if mode == "--measured-leaf-diagnostics"
                && flag == "--fixture"
                && !source.starts_with("--") =>
        {
            Ok((
                Some(source.as_str()),
                QualificationMode::MeasuredLeafDiagnostics,
            ))
        }
        [mode, flag, source]
            if mode == "--flex-content-text-diagnostics"
                && flag == "--fixture"
                && !source.starts_with("--") =>
        {
            Ok((
                Some(source.as_str()),
                QualificationMode::FlexContentTextDiagnostics,
            ))
        }
        _ => Err(
            "usage: qualify-root-context --publish-grid-lanes-original-diagnostics | --check-grid-lanes-original-diagnostics | --derive-published-qualification <edition> | --publish-matching-geometry <declared-edition> | --check-published-matching-geometry <declared-edition> | --publish-root-context-v4 | --check-published-root-context-v4 | --publish-canonical-box-differences | --check-published-canonical-box-differences | --publish-measured-grid-differences | --check-published-measured-grid-differences | --publish-grid-static-position-differences | --check-published-grid-static-position-differences | --publish-relative-inset-differences | --check-published-relative-inset-differences | --publish-wrapping-world-corner-differences | --check-published-wrapping-world-corner-differences | --publish-prepared-range-qualification | --check-published-prepared-range-qualification | --publish-flex-composition-qualification | --check-published-flex-composition-qualification | --publish-original-flex-parent-qualification | --check-published-original-flex-parent-qualification | --publish-original-percentage-flex-parent-qualification | --check-published-original-percentage-flex-parent-qualification | --publish-original-subgrid-body-qualification | --check-published-original-subgrid-body-qualification | --publish-prepared-atomic-percentage-qualification | --check-published-prepared-atomic-percentage-qualification | --publish-horizontal-br-font-band-qualification | --check-published-horizontal-br-font-band-qualification | --publish-vertical-br-font-band-qualification | --check-published-vertical-br-font-band-qualification | --publish-empty-line-br-font-band-qualification | --check-published-empty-line-br-font-band-qualification | [--fixture <declared-source>] | --stylesheet-custody --fixture <declared-source> | --font-custody --fixture <declared-source> | --numeric-diagnostics --fixture <declared-source> | --paired-geometry-capture --fixture <declared-source> | --element-scroll-size-diagnostics --fixture <declared-source> | --measured-leaf-diagnostics --fixture <declared-source> | --flex-content-text-diagnostics --fixture <declared-source> | --prepared-range-capture --fixture <declared-source>",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_support::exact_differences::PublishedEdition;
    use support::QualificationMode;

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn paired_geometry_capture_requires_one_source_argument() {
        let source = "html/qualification/fixed-orthogonal-translation-0-75.html";
        let requested = arguments(&["--paired-geometry-capture", "--fixture", source]);
        assert_eq!(
            parse_arguments(&requested),
            Ok((Some(source), QualificationMode::PairedGeometryCapture))
        );
        for invalid in [
            vec!["--paired-geometry-capture"],
            vec!["--paired-geometry-capture", "--fixture"],
            vec!["--paired-geometry-capture", "--fixture", "--other"],
            vec!["--paired-geometry-capture", "--fixture", source, "extra"],
        ] {
            assert!(parse_arguments(&arguments(&invalid)).is_err());
        }
    }

    #[test]
    fn matching_geometry_commands_resolve_a_named_catalogue_edition() {
        let modes = [
            "--publish-matching-geometry",
            "--check-published-matching-geometry",
        ]
        .map(|command| {
            let requested = arguments(&[command, "box-controls"]);
            let (fixture, mode) = parse_arguments(&requested).expect(
                "a declared matching edition must be selectable without a fixture override",
            );
            assert_eq!(fixture, None);
            assert_ne!(mode, QualificationMode::Regular);
            for invalid in [
                vec![command],
                vec![command, "unknown-matching-edition"],
                vec![command, "../box-controls"],
                vec![command, "box-controls", "arbitrary-output"],
                vec![command, "box-controls", "--fixture", "html/other.html"],
            ] {
                assert!(
                    parse_arguments(&arguments(&invalid)).is_err(),
                    "matching publication must resolve one authored edition: {invalid:?}"
                );
            }
            mode
        });
        assert_ne!(modes[0], modes[1]);
    }

    #[test]
    fn canonical_publication_commands_select_the_fixed_owner_without_extra_arguments() {
        for (flag, mode) in [
            (
                "--publish-canonical-box-differences",
                QualificationMode::PublishExact(PublishedEdition::CanonicalBoxes),
            ),
            (
                "--check-published-canonical-box-differences",
                QualificationMode::CheckPublishedExact(PublishedEdition::CanonicalBoxes),
            ),
        ] {
            assert_eq!(parse_arguments(&arguments(&[flag])), Ok((None, mode)));
            assert!(parse_arguments(&arguments(&[flag, "--fixture", "html/other.html"])).is_err());
            assert!(parse_arguments(&arguments(&[flag, "arbitrary-output"])).is_err());
        }
    }

    #[test]
    fn measured_grid_and_flex_text_publication_commands_have_fixed_outputs() {
        for (flag, mode) in [
            (
                "--publish-measured-grid-differences",
                QualificationMode::PublishExact(PublishedEdition::MeasuredGrid),
            ),
            (
                "--check-published-measured-grid-differences",
                QualificationMode::CheckPublishedExact(PublishedEdition::MeasuredGrid),
            ),
            (
                "--publish-flex-text-differences",
                QualificationMode::PublishExact(PublishedEdition::FlexText),
            ),
            (
                "--check-published-flex-text-differences",
                QualificationMode::CheckPublishedExact(PublishedEdition::FlexText),
            ),
        ] {
            assert_eq!(parse_arguments(&arguments(&[flag])), Ok((None, mode)));
            assert!(parse_arguments(&arguments(&[flag, "arbitrary-output"])).is_err());
            assert!(parse_arguments(&arguments(&[flag, "--fixture", "html/other.html"])).is_err());
        }
    }

    #[test]
    fn native_policy_publication_selects_a_fixed_edition() {
        for flags in [
            [
                "--publish-grid-static-position-differences",
                "--check-published-grid-static-position-differences",
            ],
            [
                "--publish-relative-inset-differences",
                "--check-published-relative-inset-differences",
            ],
            [
                "--publish-wrapping-world-corner-differences",
                "--check-published-wrapping-world-corner-differences",
            ],
        ] {
            let modes = flags.map(|flag| {
                let arguments_for_flag = arguments(&[flag]);
                let (selection, mode) = parse_arguments(&arguments_for_flag).unwrap();
                assert_eq!(selection, None);
                assert!(parse_arguments(&arguments(&[flag, "arbitrary-output"])).is_err());
                assert!(
                    parse_arguments(&arguments(&[flag, "--fixture", "html/other.html"])).is_err()
                );
                mode
            });
            assert_ne!(modes[0], modes[1]);
        }
    }

    #[test]
    fn specialized_publication_selects_the_fixed_owner() {
        for command in [
            "--publish-prepared-range-qualification",
            "--check-published-prepared-range-qualification",
            "--publish-flex-composition-qualification",
            "--check-published-flex-composition-qualification",
            "--publish-original-subgrid-body-qualification",
            "--check-published-original-subgrid-body-qualification",
            "--publish-prepared-atomic-percentage-qualification",
            "--check-published-prepared-atomic-percentage-qualification",
            "--publish-horizontal-br-font-band-qualification",
            "--check-published-horizontal-br-font-band-qualification",
            "--publish-vertical-br-font-band-qualification",
            "--check-published-vertical-br-font-band-qualification",
            "--publish-empty-line-br-font-band-qualification",
            "--check-published-empty-line-br-font-band-qualification",
        ] {
            assert_eq!(parse_arguments(&arguments(&[command])).unwrap().0, None);
            assert!(parse_arguments(&arguments(&[command, "other-root"])).is_err());
            assert!(
                parse_arguments(&arguments(&[command, "--fixture", "html/other.html"])).is_err()
            );
        }
    }

    #[test]
    fn element_scroll_size_cli_requires_one_explicit_source() {
        assert_eq!(
            parse_arguments(&arguments(&[
                "--element-scroll-size-diagnostics",
                "--fixture",
                "html/grid/case.html",
            ])),
            Ok((
                Some("html/grid/case.html"),
                QualificationMode::ElementScrollSizeDiagnostics
            ))
        );
        for invalid in [
            vec!["--element-scroll-size-diagnostics"],
            vec!["--element-scroll-size-diagnostics", "--fixture"],
            vec![
                "--element-scroll-size-diagnostics",
                "--fixture",
                "--numeric-diagnostics",
            ],
            vec![
                "--element-scroll-size-diagnostics",
                "--fixture",
                "html/grid/case.html",
                "extra",
            ],
        ] {
            assert!(parse_arguments(&arguments(&invalid)).is_err());
        }
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
    #[test]
    fn font_custody_cli_requires_one_explicit_fixture_and_no_other_mode() {
        let args = arguments(&["--font-custody", "--fixture", "html/subgrid/named.html"]);
        assert_eq!(
            parse_arguments(&args),
            Ok((
                Some("html/subgrid/named.html"),
                QualificationMode::FontCustody
            ))
        );
        for values in [
            vec!["--font-custody"],
            vec!["--fixture", "html/subgrid/named.html", "--font-custody"],
            vec![
                "--font-custody",
                "--stylesheet-custody",
                "--fixture",
                "html/subgrid/named.html",
            ],
            vec![
                "--numeric-diagnostics",
                "--font-custody",
                "--fixture",
                "html/subgrid/named.html",
            ],
        ] {
            assert!(parse_arguments(&arguments(&values)).is_err());
        }
    }
    #[test]
    fn durable_modes_are_full_inventory_only_and_mutually_exclusive() {
        for (flag, mode) in [
            ("--publish-root-context-v4", QualificationMode::Publish),
            (
                "--check-published-root-context-v4",
                QualificationMode::CheckPublished,
            ),
        ] {
            assert_eq!(parse_arguments(&arguments(&[flag])), Ok((None, mode)));
            for extra in [
                "--fixture",
                "--numeric-diagnostics",
                "--font-custody",
                "--stylesheet-custody",
                "--publish-root-context-v4",
                "--check-published-root-context-v4",
            ] {
                assert!(parse_arguments(&arguments(&[flag, extra])).is_err());
            }
            assert!(
                parse_arguments(&arguments(&[flag, "--fixture", "html/grid/case.html"])).is_err()
            );
            assert!(parse_arguments(&arguments(&["--fixture", flag])).is_err());
        }
    }
}

#[cfg(test)]
#[path = "qualify-root-context/font_inheritance_tests.rs"]
mod font_inheritance_tests;
