//! Host UI font observations, independent of authored CSS and text realization.
//!
//! Native facts come from Apple's [`NSFont`] and [`NSFontManager`]. The host role
//! policy follows WebKit's [system/small/status role choices] while using
//! AppKit's default menu size, rather than WebKit's explicit system menu size.
//!
//! [`NSFont`]: https://developer.apple.com/documentation/appkit/nsfont
//! [`NSFontManager`]: https://developer.apple.com/documentation/appkit/nsfontmanager
//! [system/small/status role choices]: https://github.com/WebKit/WebKit/blob/8fa4be1c3258173471bb4250e2ff7eb77544cd71/Source/WebCore/platform/graphics/cocoa/SystemFontDatabaseCocoa.mm#L39-L51

use std::{error, fmt};

/// A host UI font preference, rather than an installed generic font family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemFontRole {
    /// Captioned controls such as buttons.
    Caption,
    /// Icon labels.
    Icon,
    /// Menus and menu lists.
    Menu,
    /// Dialog messages.
    MessageBox,
    /// Small controls.
    SmallCaption,
    /// Status areas.
    StatusBar,
}

/// Failure to observe or validate a host UI font preference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemFontError {
    /// This target has no implemented native UI preference provider.
    UnsupportedPlatform,
    /// The native AppKit provider requires the application's main thread.
    MainThreadRequired,
    /// The native font did not provide a nonempty family name.
    MissingFamily,
    /// The font did not provide a nonempty face name.
    MissingFace,
    /// Point size or vertical font metrics violate their numeric contract.
    InvalidMetrics,
    /// AppKit font weight is outside its documented inclusive 0–15 range.
    InvalidWeight,
}

impl fmt::Display for SystemFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedPlatform => "native UI font observation is unsupported on this target",
            Self::MainThreadRequired => "native UI font observation requires the main thread",
            Self::MissingFamily => "a system font requires a nonempty family name",
            Self::MissingFace => "a system font requires a nonempty face name",
            Self::InvalidMetrics => "system font size and metrics are invalid",
            Self::InvalidWeight => "AppKit font weight must be between zero and fifteen",
        })
    }
}

impl error::Error for SystemFontError {}

/// AppKit's observed weight approximation on its native integer 0–15 scale.
///
/// This is not a CSS weight. Converting it to text-selection attributes belongs
/// to the consuming adapter, which must preserve the native face identity.
/// [`NSFontManager::weight(of:)`](https://developer.apple.com/documentation/appkit/nsfontmanager/weight(of:))
/// reports an approximation, rather than the font's continuous design weight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemFontWeight(u8);

impl SystemFontWeight {
    /// Validates an AppKit weight: 5 means normal; 9 or more means bold/heavier.
    pub fn from_appkit(value: i64) -> Result<Self, SystemFontError> {
        let value = u8::try_from(value).map_err(|_| SystemFontError::InvalidWeight)?;
        if value > 15 {
            return Err(SystemFontError::InvalidWeight);
        }
        Ok(Self(value))
    }

    /// Returns the original AppKit integer weight without a CSS conversion.
    #[must_use]
    pub const fn appkit(self) -> u8 {
        self.0
    }
}

/// Native style traits reported for the selected UI face.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemFontTraits {
    /// The native face is italic.
    pub italic: bool,
    /// The native face has the bold trait, independently of its numeric weight.
    pub bold: bool,
    /// The native face is condensed.
    pub condensed: bool,
    /// The native face is expanded.
    pub expanded: bool,
    /// The native face has small-capital glyphs.
    pub small_caps: bool,
}

/// Native point-size and vertical font metrics in typographic points.
///
/// One point is 1/72 inch. These values have not been converted to CSS pixels,
/// multiplied by window density or zoom, or substituted for shaped text metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemFontMetrics {
    point_size: f64,
    ascent: f64,
    descent: f64,
    leading: f64,
    cap_height: f64,
    x_height: f64,
}

impl SystemFontMetrics {
    /// Validates finite values, positive size, nonnegative ascent and heights,
    /// and nonpositive descent. Native leading may be negative.
    pub fn new(
        point_size: f64,
        ascent: f64,
        descent: f64,
        leading: f64,
        cap_height: f64,
        x_height: f64,
    ) -> Result<Self, SystemFontError> {
        if ![point_size, ascent, descent, leading, cap_height, x_height]
            .into_iter()
            .all(f64::is_finite)
            || point_size <= 0.0
            || ascent < 0.0
            || descent > 0.0
            || cap_height < 0.0
            || x_height < 0.0
        {
            return Err(SystemFontError::InvalidMetrics);
        }
        Ok(Self {
            point_size,
            ascent,
            descent,
            leading,
            cap_height,
            x_height,
        })
    }

    /// Returns native size in points.
    #[must_use]
    pub const fn point_size(self) -> f64 {
        self.point_size
    }
    /// Returns native ascent above the baseline in points.
    #[must_use]
    pub const fn ascent(self) -> f64 {
        self.ascent
    }
    /// Returns native signed descent below the baseline in points.
    #[must_use]
    pub const fn descent(self) -> f64 {
        self.descent
    }
    /// Returns native leading in points, which may be negative.
    #[must_use]
    pub const fn leading(self) -> f64 {
        self.leading
    }
    /// Returns native capital height in points.
    #[must_use]
    pub const fn cap_height(self) -> f64 {
        self.cap_height
    }
    /// Returns native x-height in points.
    #[must_use]
    pub const fn x_height(self) -> f64 {
        self.x_height
    }
}

/// One complete observed UI font, including native face identity.
///
/// A family can be a platform-private system family. Text must resolve the
/// observed face through a suitable native adapter rather than replacing it
/// with a guessed family or treating this observation as shaped font data.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemFont {
    family: String,
    face: String,
    weight: SystemFontWeight,
    traits: SystemFontTraits,
    metrics: SystemFontMetrics,
}

impl SystemFont {
    /// Records a host font, rejecting missing names without guessing fallbacks.
    pub fn new(
        family: impl Into<String>,
        face: impl Into<String>,
        weight: SystemFontWeight,
        traits: SystemFontTraits,
        metrics: SystemFontMetrics,
    ) -> Result<Self, SystemFontError> {
        let family = family.into();
        let face = face.into();
        if family.trim().is_empty() {
            return Err(SystemFontError::MissingFamily);
        }
        if face.trim().is_empty() {
            return Err(SystemFontError::MissingFace);
        }
        Ok(Self {
            family,
            face,
            weight,
            traits,
            metrics,
        })
    }

    /// Returns the native family name without normalization.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }
    /// Returns the native face's PostScript name without normalization.
    #[must_use]
    pub fn face(&self) -> &str {
        &self.face
    }
    /// Returns AppKit's observed weight approximation, without CSS conversion.
    #[must_use]
    pub const fn weight(&self) -> SystemFontWeight {
        self.weight
    }
    /// Returns the observed native face traits.
    #[must_use]
    pub const fn traits(&self) -> SystemFontTraits {
        self.traits
    }
    /// Returns native metrics, before density or zoom conversion.
    #[must_use]
    pub const fn metrics(&self) -> SystemFontMetrics {
        self.metrics
    }
}

/// An owned observation of all six UI font roles, with no retained native objects.
///
/// Capture is on demand; call again when the host's UI preferences change. It
/// does not subscribe to changes or promise an atomic operating-system snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemFontEnvironment {
    caption: SystemFont,
    icon: SystemFont,
    menu: SystemFont,
    message_box: SystemFont,
    small_caption: SystemFont,
    status_bar: SystemFont,
}

impl SystemFontEnvironment {
    /// Captures actual native UI font preferences.
    ///
    /// macOS requires the application's main thread. Surgeist's macOS host
    /// policy selects AppKit's default system font for captions, icon labels and
    /// messages; the menu font for menus; the system font at small-system size
    /// for small captions; and the system font at label size for status areas.
    /// These choices are host policy, not a platform-independent CSS mapping.
    /// Other targets explicitly return [`SystemFontError::UnsupportedPlatform`].
    /// No authored font syntax, CSS weight conversion or text realization occurs.
    pub fn capture() -> Result<Self, SystemFontError> {
        capture_native()
    }

    /// Returns the complete observation for the selected host role.
    #[must_use]
    pub const fn font(&self, role: SystemFontRole) -> &SystemFont {
        match role {
            SystemFontRole::Caption => &self.caption,
            SystemFontRole::Icon => &self.icon,
            SystemFontRole::Menu => &self.menu,
            SystemFontRole::MessageBox => &self.message_box,
            SystemFontRole::SmallCaption => &self.small_caption,
            SystemFontRole::StatusBar => &self.status_bar,
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn capture_native() -> Result<SystemFontEnvironment, SystemFontError> {
    Err(SystemFontError::UnsupportedPlatform)
}

#[cfg(target_os = "macos")]
fn capture_native() -> Result<SystemFontEnvironment, SystemFontError> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSFont, NSFontManager, NSFontTraitMask};

    let main_thread = MainThreadMarker::new().ok_or(SystemFontError::MainThreadRequired)?;
    let manager = NSFontManager::sharedFontManager(main_thread);
    let observe = |font: &NSFont| {
        let traits = manager.traitsOfFont(font);
        SystemFont::new(
            font.familyName()
                .ok_or(SystemFontError::MissingFamily)?
                .to_string(),
            font.fontName().to_string(),
            SystemFontWeight::from_appkit(manager.weightOfFont(font) as i64)?,
            SystemFontTraits {
                italic: traits.contains(NSFontTraitMask::ItalicFontMask),
                bold: traits.contains(NSFontTraitMask::BoldFontMask),
                condensed: traits.contains(NSFontTraitMask::CondensedFontMask),
                expanded: traits.contains(NSFontTraitMask::ExpandedFontMask),
                small_caps: traits.contains(NSFontTraitMask::SmallCapsFontMask),
            },
            SystemFontMetrics::new(
                font.pointSize(),
                font.ascender(),
                font.descender(),
                font.leading(),
                font.capHeight(),
                font.xHeight(),
            )?,
        )
    };
    // A zero size requests the platform's default for that UI font category.
    let system = observe(&NSFont::systemFontOfSize(0.0))?;
    Ok(SystemFontEnvironment {
        caption: system.clone(),
        icon: system.clone(),
        menu: observe(&NSFont::menuFontOfSize(0.0))?,
        message_box: system,
        small_caption: observe(&NSFont::systemFontOfSize(NSFont::smallSystemFontSize()))?,
        status_bar: observe(&NSFont::systemFontOfSize(NSFont::labelFontSize()))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> SystemFontMetrics {
        SystemFontMetrics::new(12.0, 9.0, -3.0, -1.0, 8.0, 5.0).unwrap()
    }

    fn traits() -> SystemFontTraits {
        SystemFontTraits {
            italic: false,
            bold: false,
            condensed: false,
            expanded: false,
            small_caps: false,
        }
    }

    #[test]
    fn native_weight_rejects_out_of_range_without_clamping() {
        for value in [i64::MIN, -1, 16, i64::MAX] {
            assert_eq!(
                SystemFontWeight::from_appkit(value),
                Err(SystemFontError::InvalidWeight)
            );
        }
        for value in [0, 5, 9, 15] {
            assert_eq!(
                i64::from(SystemFontWeight::from_appkit(value).unwrap().appkit()),
                value
            );
        }
    }

    #[test]
    fn metrics_reject_nonfinite_values_and_invalid_signs() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for index in 0..6 {
                let mut values = [12.0, 9.0, -3.0, 0.0, 8.0, 5.0];
                values[index] = value;
                assert_eq!(
                    SystemFontMetrics::new(
                        values[0], values[1], values[2], values[3], values[4], values[5]
                    ),
                    Err(SystemFontError::InvalidMetrics)
                );
            }
        }
        for values in [
            [0.0, 9.0, -3.0, 0.0, 8.0, 5.0],
            [-1.0, 9.0, -3.0, 0.0, 8.0, 5.0],
            [12.0, -1.0, -3.0, 0.0, 8.0, 5.0],
            [12.0, 9.0, 1.0, 0.0, 8.0, 5.0],
            [12.0, 9.0, -3.0, 0.0, -1.0, 5.0],
            [12.0, 9.0, -3.0, 0.0, 8.0, -1.0],
        ] {
            assert_eq!(
                SystemFontMetrics::new(
                    values[0], values[1], values[2], values[3], values[4], values[5]
                ),
                Err(SystemFontError::InvalidMetrics)
            );
        }
        assert_eq!(metrics().leading(), -1.0);
    }

    #[test]
    fn font_identity_requires_names_and_preserves_native_spelling() {
        let weight = SystemFontWeight::from_appkit(5).unwrap();
        assert_eq!(
            SystemFont::new(" ", "Face", weight, traits(), metrics()),
            Err(SystemFontError::MissingFamily)
        );
        assert_eq!(
            SystemFont::new("Family", "\t", weight, traits(), metrics()),
            Err(SystemFontError::MissingFace)
        );
        let font = SystemFont::new(
            ".Native Family",
            ".Native-Face",
            weight,
            traits(),
            metrics(),
        )
        .unwrap();
        assert_eq!(font.family(), ".Native Family");
        assert_eq!(font.face(), ".Native-Face");
        assert_eq!(font.metrics(), metrics());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn capture_from_worker_thread_reports_main_thread_requirement() {
        let result = std::thread::spawn(SystemFontEnvironment::capture)
            .join()
            .unwrap();
        assert_eq!(result, Err(SystemFontError::MainThreadRequired));
    }
}
