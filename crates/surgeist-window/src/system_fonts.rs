//! Host UI font observations, independent of authored CSS and text realization.
//!
//! macOS facts come from Apple's [`NSFont`] and [`NSFontManager`]. The host role
//! policy follows WebKit's [system/small/status role choices] while using
//! AppKit's default menu size, rather than WebKit's explicit system menu size.
//! Windows captures native [`LOGFONTW`] preferences at 96 DPI without selecting
//! a physical font or deriving metrics. Consumers own text-resource realization.
//!
//! [`LOGFONTW`]: https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-logfontw
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
    /// A Windows native query failed.
    NativeQuery {
        /// The failed native query.
        operation: &'static str,
        /// The Windows error code reported by the query.
        code: u32,
    },
    /// Native data could not represent a complete observation.
    InvalidNativeData,
}

impl fmt::Display for SystemFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedPlatform => "native UI font observation is unsupported on this target",
            Self::MainThreadRequired => "native UI font observation requires the main thread",
            Self::MissingFamily => "a system font requires a nonempty family name",
            Self::MissingFace => "a system font requires a nonempty face name",
            Self::InvalidMetrics => "system font size and metrics are invalid",
            Self::InvalidWeight => "system font weight is outside its native range",
            Self::NativeQuery { operation, code } => {
                return write!(
                    f,
                    "native UI font query {operation} failed (Windows error {code})"
                );
            }
            Self::InvalidNativeData => "native UI font data is invalid",
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
pub struct AppKitFontWeight(u8);

impl AppKitFontWeight {
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
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Native style traits reported for the selected UI face.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppKitFontTraits {
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
pub struct AppKitFontMetrics {
    point_size: f64,
    ascent: f64,
    descent: f64,
    leading: f64,
    cap_height: f64,
    x_height: f64,
}

impl AppKitFontMetrics {
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

/// An AppKit UI font observation, including its native PostScript identity.
///
/// A family can be a platform-private system family. Text must resolve the
/// observed face through a suitable native adapter rather than replacing it
/// with a guessed family or treating this observation as shaped font data.
#[derive(Clone, Debug, PartialEq)]
pub struct AppKitSystemFont {
    family: String,
    postscript_name: String,
    weight: AppKitFontWeight,
    traits: AppKitFontTraits,
    metrics: AppKitFontMetrics,
}

impl AppKitSystemFont {
    /// Records a host font, rejecting missing names without guessing fallbacks.
    pub fn new(
        family: impl Into<String>,
        postscript_name: impl Into<String>,
        weight: AppKitFontWeight,
        traits: AppKitFontTraits,
        metrics: AppKitFontMetrics,
    ) -> Result<Self, SystemFontError> {
        let family = family.into();
        let postscript_name = postscript_name.into();
        if family.trim().is_empty() {
            return Err(SystemFontError::MissingFamily);
        }
        if postscript_name.trim().is_empty() {
            return Err(SystemFontError::MissingFace);
        }
        Ok(Self {
            family,
            postscript_name,
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
    pub fn postscript_name(&self) -> &str {
        &self.postscript_name
    }
    /// Returns AppKit's observed weight approximation, without CSS conversion.
    #[must_use]
    pub const fn weight(&self) -> AppKitFontWeight {
        self.weight
    }
    /// Returns the observed native face traits.
    #[must_use]
    pub const fn traits(&self) -> AppKitFontTraits {
        self.traits
    }
    /// Returns native metrics, before density or zoom conversion.
    #[must_use]
    pub const fn metrics(&self) -> AppKitFontMetrics {
        self.metrics
    }
}

/// A native UI font observation with platform-specific facts.
///
/// AppKit supplies a selected native face and metrics. Windows supplies a
/// logical font preference whose physical resource remains unresolved.
#[derive(Clone, Debug, PartialEq)]
pub enum SystemFont {
    /// AppKit's observed native UI face.
    AppKit(AppKitSystemFont),
    /// Windows' observed logical font preference.
    Windows(WindowsSystemFont),
}

/// An owned Windows LOGFONT preference, without physical face realization.
///
/// Dimensions are native logical units at `reference_dpi`; height retains its
/// sign: positive requests cell height, negative requests character/em height,
/// and zero requests the native default. A zero weight or empty typeface name
/// likewise requests native selection defaults. These are not CSS values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowsSystemFont {
    face_name: String,
    face_name_utf16: Vec<u16>,
    reference_dpi: std::num::NonZeroU32,
    height: i32,
    width: i32,
    escapement: i32,
    orientation: i32,
    weight: i32,
    italic: u8,
    underline: u8,
    strike_out: u8,
    char_set: u8,
    out_precision: u8,
    clip_precision: u8,
    quality: u8,
    pitch_and_family: u8,
}

impl WindowsSystemFont {
    /// Returns the requested typeface name; this is not a PostScript identity.
    /// The native UTF-16 is decoded without replacement or normalization.
    #[must_use]
    pub fn face_name(&self) -> &str {
        &self.face_name
    }
    /// Returns the exact native UTF-16 typeface units, excluding the terminator.
    #[must_use]
    pub fn face_name_utf16(&self) -> &[u16] {
        &self.face_name_utf16
    }
    /// Returns the explicit DPI used to query the logical preference.
    #[must_use]
    pub const fn reference_dpi(&self) -> std::num::NonZeroU32 {
        self.reference_dpi
    }
    /// Returns signed native cell/em height, with zero meaning the default.
    #[must_use]
    pub const fn height(&self) -> i32 {
        self.height
    }
    /// Returns native average width, with zero requesting native aspect matching.
    #[must_use]
    pub const fn width(&self) -> i32 {
        self.width
    }
    /// Returns native escapement in tenths of a degree.
    #[must_use]
    pub const fn escapement(&self) -> i32 {
        self.escapement
    }
    /// Returns native character orientation in tenths of a degree.
    #[must_use]
    pub const fn orientation(&self) -> i32 {
        self.orientation
    }
    /// Returns native weight in 0–1000, with zero requesting the default.
    #[must_use]
    pub const fn weight(&self) -> i32 {
        self.weight
    }
    /// Returns the original italic byte; nonzero requests italics.
    #[must_use]
    pub const fn italic(&self) -> u8 {
        self.italic
    }
    /// Returns the original underline byte; nonzero requests underlining.
    #[must_use]
    pub const fn underline(&self) -> u8 {
        self.underline
    }
    /// Returns the original strikeout byte; nonzero requests strikeout.
    #[must_use]
    pub const fn strike_out(&self) -> u8 {
        self.strike_out
    }
    /// Returns the native character-set selector byte.
    #[must_use]
    pub const fn char_set(&self) -> u8 {
        self.char_set
    }
    /// Returns the native output-precision selector byte.
    #[must_use]
    pub const fn out_precision(&self) -> u8 {
        self.out_precision
    }
    /// Returns the native clipping-precision byte.
    #[must_use]
    pub const fn clip_precision(&self) -> u8 {
        self.clip_precision
    }
    /// Returns the native output-quality selector byte.
    #[must_use]
    pub const fn quality(&self) -> u8 {
        self.quality
    }
    /// Returns the original combined pitch/family byte without decomposition.
    #[must_use]
    pub const fn pitch_and_family(&self) -> u8 {
        self.pitch_and_family
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
    /// Windows reads nonclient and icon-title logical font preferences at 96 DPI.
    /// Names, signed logical dimensions and native selection attributes remain
    /// preferences; capture does not select a physical font or invent metrics.
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

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn capture_native() -> Result<SystemFontEnvironment, SystemFontError> {
    Err(SystemFontError::UnsupportedPlatform)
}

#[cfg(target_os = "windows")]
fn capture_native() -> Result<SystemFontEnvironment, SystemFontError> {
    use surgeist_typed_winsafe::{Error, UiFontRole, ui_fonts_for_dpi};

    // Explicit DPI avoids dependence on the caller's DPI-awareness context.
    let dpi = std::num::NonZeroU32::new(96).expect("96 DPI is nonzero");
    let fonts = ui_fonts_for_dpi(dpi).map_err(|error| match error {
        Error::UnsupportedPlatform => SystemFontError::UnsupportedPlatform,
        Error::Native { operation, code } => SystemFontError::NativeQuery { operation, code },
        _ => SystemFontError::InvalidNativeData,
    })?;
    let observe = |role| {
        let font = fonts.font(role);
        SystemFont::Windows(WindowsSystemFont {
            face_name: font.face_name().to_owned(),
            face_name_utf16: font.face_name_utf16().to_vec(),
            reference_dpi: font.reference_dpi(),
            height: font.height(),
            width: font.width(),
            escapement: font.escapement(),
            orientation: font.orientation(),
            weight: font.weight(),
            italic: font.italic(),
            underline: font.underline(),
            strike_out: font.strike_out(),
            char_set: font.char_set(),
            out_precision: font.out_precision(),
            clip_precision: font.clip_precision(),
            quality: font.quality(),
            pitch_and_family: font.pitch_and_family(),
        })
    };
    Ok(SystemFontEnvironment {
        caption: observe(UiFontRole::Caption),
        icon: observe(UiFontRole::Icon),
        menu: observe(UiFontRole::Menu),
        message_box: observe(UiFontRole::MessageBox),
        small_caption: observe(UiFontRole::SmallCaption),
        status_bar: observe(UiFontRole::StatusBar),
    })
}

#[cfg(target_os = "macos")]
fn capture_native() -> Result<SystemFontEnvironment, SystemFontError> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSFont, NSFontManager, NSFontTraitMask};

    let main_thread = MainThreadMarker::new().ok_or(SystemFontError::MainThreadRequired)?;
    let manager = NSFontManager::sharedFontManager(main_thread);
    let observe = |font: &NSFont| {
        let traits = manager.traitsOfFont(font);
        AppKitSystemFont::new(
            font.familyName()
                .ok_or(SystemFontError::MissingFamily)?
                .to_string(),
            font.fontName().to_string(),
            AppKitFontWeight::from_appkit(manager.weightOfFont(font) as i64)?,
            AppKitFontTraits {
                italic: traits.contains(NSFontTraitMask::ItalicFontMask),
                bold: traits.contains(NSFontTraitMask::BoldFontMask),
                condensed: traits.contains(NSFontTraitMask::CondensedFontMask),
                expanded: traits.contains(NSFontTraitMask::ExpandedFontMask),
                small_caps: traits.contains(NSFontTraitMask::SmallCapsFontMask),
            },
            AppKitFontMetrics::new(
                font.pointSize(),
                font.ascender(),
                font.descender(),
                font.leading(),
                font.capHeight(),
                font.xHeight(),
            )?,
        )
        .map(SystemFont::AppKit)
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

    fn metrics() -> AppKitFontMetrics {
        AppKitFontMetrics::new(12.0, 9.0, -3.0, -1.0, 8.0, 5.0).unwrap()
    }

    fn traits() -> AppKitFontTraits {
        AppKitFontTraits {
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
                AppKitFontWeight::from_appkit(value),
                Err(SystemFontError::InvalidWeight)
            );
        }
        for value in [0, 5, 9, 15] {
            assert_eq!(
                i64::from(AppKitFontWeight::from_appkit(value).unwrap().value()),
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
                    AppKitFontMetrics::new(
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
                AppKitFontMetrics::new(
                    values[0], values[1], values[2], values[3], values[4], values[5]
                ),
                Err(SystemFontError::InvalidMetrics)
            );
        }
        assert_eq!(metrics().leading(), -1.0);
    }

    #[test]
    fn font_identity_requires_names_and_preserves_native_spelling() {
        let weight = AppKitFontWeight::from_appkit(5).unwrap();
        assert_eq!(
            AppKitSystemFont::new(" ", "Face", weight, traits(), metrics()),
            Err(SystemFontError::MissingFamily)
        );
        assert_eq!(
            AppKitSystemFont::new("Family", "\t", weight, traits(), metrics()),
            Err(SystemFontError::MissingFace)
        );
        let font = AppKitSystemFont::new(
            ".Native Family",
            ".Native-Face",
            weight,
            traits(),
            metrics(),
        )
        .unwrap();
        assert_eq!(font.family(), ".Native Family");
        assert_eq!(font.postscript_name(), ".Native-Face");
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
