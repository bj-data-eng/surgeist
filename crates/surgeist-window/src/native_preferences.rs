//! Native baseline observations, independent of application or origin overrides.

#[cfg(any(target_os = "macos", test))]
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
    #[cfg(not(target_os = "macos"))]
    {
        UserPreferences {
            color_scheme,
            ..UserPreferences::default()
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

    #[cfg(not(target_os = "macos"))]
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
