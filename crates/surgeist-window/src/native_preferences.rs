//! Native baseline observations, independent of application or origin overrides.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::{ContrastPreference, ReductionPreference};
use crate::{Theme, UserPreferences};

/// Samples supported preferences at an existing event-loop opportunity.
/// No notification observer or independent wakeup is installed.
pub(crate) fn capture(color_scheme: Option<Theme>) -> UserPreferences {
    #[cfg(target_os = "macos")]
    {
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        AccessibilityPreferences {
            reduce_motion: workspace.accessibilityDisplayShouldReduceMotion(),
            reduce_transparency: workspace.accessibilityDisplayShouldReduceTransparency(),
            increase_contrast: workspace.accessibilityDisplayShouldIncreaseContrast(),
        }
        .into_preferences(color_scheme)
    }
    #[cfg(target_os = "windows")]
    {
        use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};

        let ui = UISettings::new().ok();
        WindowsAccessibilityPreferences {
            animations_enabled: ui.as_ref().and_then(|ui| ui.AnimationsEnabled().ok()),
            transparency_enabled: ui.as_ref().and_then(|ui| ui.AdvancedEffectsEnabled().ok()),
            high_contrast: AccessibilitySettings::new()
                .and_then(|settings| settings.HighContrast())
                .ok(),
        }
        .into_preferences(color_scheme)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        UserPreferences {
            color_scheme,
            ..UserPreferences::default()
        }
    }
}

#[cfg(any(target_os = "windows", test))]
struct WindowsAccessibilityPreferences {
    animations_enabled: Option<bool>,
    transparency_enabled: Option<bool>,
    high_contrast: Option<bool>,
}

#[cfg(any(target_os = "windows", test))]
impl WindowsAccessibilityPreferences {
    fn into_preferences(self, color_scheme: Option<Theme>) -> UserPreferences {
        let reduction = |enabled| {
            if enabled {
                ReductionPreference::NoPreference
            } else {
                ReductionPreference::Reduce
            }
        };
        UserPreferences {
            reduced_motion: self.animations_enabled.map(reduction),
            reduced_transparency: self.transparency_enabled.map(reduction),
            // Windows contrast themes select a palette, not an independently
            // measured preference for higher or lower contrast. Without palette
            // classification, retain that custom preference rather than More.
            contrast: self.high_contrast.map(|active| {
                if active {
                    ContrastPreference::Custom
                } else {
                    ContrastPreference::NoPreference
                }
            }),
            color_scheme,
            reduced_data: None,
        }
    }
}

#[cfg(any(target_os = "macos", test))]
struct AccessibilityPreferences {
    reduce_motion: bool,
    reduce_transparency: bool,
    increase_contrast: bool,
}

#[cfg(any(target_os = "macos", test))]
impl AccessibilityPreferences {
    fn into_preferences(self, color_scheme: Option<Theme>) -> UserPreferences {
        // Host policy maps each independent AppKit flag. Increase-contrast=false
        // does not establish Less/Custom and does not suppress other flags.
        UserPreferences {
            reduced_motion: Some(if self.reduce_motion {
                ReductionPreference::Reduce
            } else {
                ReductionPreference::NoPreference
            }),
            reduced_transparency: Some(if self.reduce_transparency {
                ReductionPreference::Reduce
            } else {
                ReductionPreference::NoPreference
            }),
            contrast: Some(if self.increase_contrast {
                ContrastPreference::More
            } else {
                ContrastPreference::NoPreference
            }),
            color_scheme,
            reduced_data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_known_connection_cost_is_captured_as_reduced_data() {
        use windows::Networking::Connectivity::{NetworkCostType, NetworkInformation};

        // Derive the expected UA policy directly from Windows, independently of
        // capture. An absent profile, unknown cost or failed read is unavailable.
        let expected = || -> windows::core::Result<Option<ReductionPreference>> {
            let cost = NetworkInformation::GetInternetConnectionProfile()?.GetConnectionCost()?;
            let kind = cost.NetworkCostType()?;
            if !matches!(
                kind,
                NetworkCostType::Unrestricted | NetworkCostType::Fixed | NetworkCostType::Variable
            ) {
                return Ok(None);
            }
            let roaming = cost.Roaming()?;
            let over_limit = cost.OverDataLimit()?;
            let approaching_limit = cost.ApproachingDataLimit()?;
            Ok(Some(
                if kind == NetworkCostType::Unrestricted
                    && !roaming
                    && !over_limit
                    && !approaching_limit
                {
                    ReductionPreference::NoPreference
                } else {
                    ReductionPreference::Reduce
                },
            ))
        };
        let before = expected().ok().flatten();
        let captured = capture(Some(Theme::Dark));
        let after = expected().ok().flatten();
        if before != after {
            eprintln!("SKIP reduced-data comparison: native cost changed during capture");
            return;
        }
        eprintln!(
            "native reduced-data witness={before:?}, captured={:?}",
            captured.reduced_data
        );
        assert_eq!(captured.reduced_data, before);
        assert_eq!(captured.color_scheme, Some(Theme::Dark));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_capture_reports_actual_accessibility_preferences() {
        use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};

        let ui = UISettings::new().expect("Windows UI settings are available");
        let accessibility =
            AccessibilitySettings::new().expect("Windows accessibility settings are available");
        let animations = ui
            .AnimationsEnabled()
            .expect("animation effects observation");
        let transparency = ui
            .AdvancedEffectsEnabled()
            .expect("transparency effects observation");
        let high_contrast = accessibility
            .HighContrast()
            .expect("contrast theme observation");
        let captured = capture(Some(Theme::Dark));

        assert_eq!(
            captured.reduced_motion,
            Some(if animations {
                ReductionPreference::NoPreference
            } else {
                ReductionPreference::Reduce
            })
        );
        assert_eq!(
            captured.reduced_transparency,
            Some(if transparency {
                ReductionPreference::NoPreference
            } else {
                ReductionPreference::Reduce
            })
        );
        assert_eq!(
            captured.contrast,
            Some(if high_contrast {
                ContrastPreference::Custom
            } else {
                ContrastPreference::NoPreference
            })
        );
        assert_eq!(captured.color_scheme, Some(Theme::Dark));
        assert_eq!(captured.reduced_data, None);
    }

    #[test]
    fn accessibility_flags_map_independently_without_inventing_data_or_color_preferences() {
        for (motion, expected_motion) in [
            (false, ReductionPreference::NoPreference),
            (true, ReductionPreference::Reduce),
        ] {
            for (transparency, expected_transparency) in [
                (false, ReductionPreference::NoPreference),
                (true, ReductionPreference::Reduce),
            ] {
                for (contrast, expected_contrast) in [
                    (false, ContrastPreference::NoPreference),
                    (true, ContrastPreference::More),
                ] {
                    let preferences = AccessibilityPreferences {
                        reduce_motion: motion,
                        reduce_transparency: transparency,
                        increase_contrast: contrast,
                    }
                    .into_preferences(None);
                    assert_eq!(preferences.reduced_motion, Some(expected_motion));
                    assert_eq!(
                        preferences.reduced_transparency,
                        Some(expected_transparency)
                    );
                    assert_eq!(preferences.contrast, Some(expected_contrast));
                    assert_eq!(preferences.reduced_data, None);
                    assert_eq!(preferences.color_scheme, None);
                }
            }
        }
    }

    #[test]
    fn accessibility_capture_retains_the_independently_supplied_system_scheme() {
        for theme in [Theme::Light, Theme::Dark] {
            let preferences = AccessibilityPreferences {
                reduce_motion: true,
                reduce_transparency: true,
                increase_contrast: false,
            }
            .into_preferences(Some(theme));
            assert_eq!(preferences.color_scheme, Some(theme));
            assert_eq!(
                preferences.reduced_motion,
                Some(ReductionPreference::Reduce)
            );
            assert_eq!(
                preferences.reduced_transparency,
                Some(ReductionPreference::Reduce)
            );
            assert_eq!(preferences.contrast, Some(ContrastPreference::NoPreference));
        }
    }

    #[test]
    fn windows_preferences_preserve_each_known_flag_and_independent_unavailability() {
        for (animations, motion) in [
            (None, None),
            (Some(true), Some(ReductionPreference::NoPreference)),
            (Some(false), Some(ReductionPreference::Reduce)),
        ] {
            for (transparency, reduction) in [
                (None, None),
                (Some(true), Some(ReductionPreference::NoPreference)),
                (Some(false), Some(ReductionPreference::Reduce)),
            ] {
                for (contrast, expected_contrast) in [
                    (None, None),
                    (Some(false), Some(ContrastPreference::NoPreference)),
                    (Some(true), Some(ContrastPreference::Custom)),
                ] {
                    for theme in [None, Some(Theme::Light), Some(Theme::Dark)] {
                        let captured = WindowsAccessibilityPreferences {
                            animations_enabled: animations,
                            transparency_enabled: transparency,
                            high_contrast: contrast,
                        }
                        .into_preferences(theme);
                        assert_eq!(
                            captured,
                            UserPreferences {
                                reduced_motion: motion,
                                reduced_transparency: reduction,
                                contrast: expected_contrast,
                                color_scheme: theme,
                                reduced_data: None,
                            }
                        );
                    }
                }
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    #[test]
    fn unsupported_platform_leaves_accessibility_observations_unavailable() {
        assert_eq!(
            capture(Some(Theme::Dark)),
            UserPreferences {
                color_scheme: Some(Theme::Dark),
                ..UserPreferences::default()
            }
        );
    }
}
