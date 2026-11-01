use crate::Result;
use std::num::NonZeroU32;

/// A native Windows user-interface font role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFontRole {
    Caption,
    Icon,
    Menu,
    MessageBox,
    SmallCaption,
    StatusBar,
}

/// An owned logical font preference returned by Windows, before font mapping.
///
/// This is a `LOGFONTW` request, not the identity or metrics of a realized font.
/// Empty face names and zero height/weight retain their native default meaning.
/// Dimensions are logical units at [`Self::reference_dpi`], not points. Negative
/// height requests character height; positive height requests cell height.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalFont {
    pub(crate) reference_dpi: NonZeroU32,
    pub(crate) face_name: String,
    pub(crate) face_name_utf16: Vec<u16>,
    pub(crate) height: i32,
    pub(crate) width: i32,
    pub(crate) escapement: i32,
    pub(crate) orientation: i32,
    pub(crate) weight: i32,
    pub(crate) italic: u8,
    pub(crate) underline: u8,
    pub(crate) strike_out: u8,
    pub(crate) char_set: u8,
    pub(crate) out_precision: u8,
    pub(crate) clip_precision: u8,
    pub(crate) quality: u8,
    pub(crate) pitch_and_family: u8,
}

impl LogicalFont {
    /// DPI passed explicitly to the native preference query.
    #[must_use]
    pub const fn reference_dpi(&self) -> NonZeroU32 {
        self.reference_dpi
    }
    /// Native face-name request, decoded without replacement or normalization.
    #[must_use]
    pub fn face_name(&self) -> &str {
        &self.face_name
    }
    /// Original UTF-16 name code units preceding the native terminator.
    #[must_use]
    pub fn face_name_utf16(&self) -> &[u16] {
        &self.face_name_utf16
    }
    /// Signed logical height; zero leaves size selection to the font mapper.
    #[must_use]
    pub const fn height(&self) -> i32 {
        self.height
    }
    /// Logical average character width; zero leaves width selection to the mapper.
    #[must_use]
    pub const fn width(&self) -> i32 {
        self.width
    }
    /// Baseline escapement in tenths of a degree.
    #[must_use]
    pub const fn escapement(&self) -> i32 {
        self.escapement
    }
    /// Character orientation in tenths of a degree.
    #[must_use]
    pub const fn orientation(&self) -> i32 {
        self.orientation
    }
    /// Native 0–1000 requested weight; zero means the default weight.
    #[must_use]
    pub const fn weight(&self) -> i32 {
        self.weight
    }
    /// Native italic byte; nonzero requests italic.
    #[must_use]
    pub const fn italic(&self) -> u8 {
        self.italic
    }
    /// Native underline byte; nonzero requests underline.
    #[must_use]
    pub const fn underline(&self) -> u8 {
        self.underline
    }
    /// Native strikeout byte; nonzero requests strikeout.
    #[must_use]
    pub const fn strike_out(&self) -> u8 {
        self.strike_out
    }
    /// Native character-set selector.
    #[must_use]
    pub const fn char_set(&self) -> u8 {
        self.char_set
    }
    /// Native output-precision selector.
    #[must_use]
    pub const fn out_precision(&self) -> u8 {
        self.out_precision
    }
    /// Native clipping-precision flags.
    #[must_use]
    pub const fn clip_precision(&self) -> u8 {
        self.clip_precision
    }
    /// Native output-quality selector.
    #[must_use]
    pub const fn quality(&self) -> u8 {
        self.quality
    }
    /// Original combined native pitch and family bits.
    #[must_use]
    pub const fn pitch_and_family(&self) -> u8 {
        self.pitch_and_family
    }
}

/// An owned `NONCLIENTMETRICSW` observation at an explicit reference DPI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonClientMetrics {
    pub reference_dpi: NonZeroU32,
    pub border_width: i32,
    pub scroll_width: i32,
    pub scroll_height: i32,
    pub caption_width: i32,
    pub caption_height: i32,
    pub caption_font: LogicalFont,
    pub small_caption_width: i32,
    pub small_caption_height: i32,
    pub small_caption_font: LogicalFont,
    pub menu_width: i32,
    pub menu_height: i32,
    pub menu_font: LogicalFont,
    pub status_font: LogicalFont,
    pub message_font: LogicalFont,
    pub padded_border_width: i32,
}

/// All six actual Windows role preferences with no retained native resources.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiFonts {
    caption: LogicalFont,
    icon: LogicalFont,
    menu: LogicalFont,
    message_box: LogicalFont,
    small_caption: LogicalFont,
    status_bar: LogicalFont,
}

impl UiFonts {
    fn from_preferences(metrics: NonClientMetrics, icon: LogicalFont) -> Self {
        Self {
            caption: metrics.caption_font,
            icon,
            menu: metrics.menu_font,
            message_box: metrics.message_font,
            small_caption: metrics.small_caption_font,
            status_bar: metrics.status_font,
        }
    }

    /// Returns the actual logical preference for this Windows role.
    #[must_use]
    pub const fn font(&self, role: UiFontRole) -> &LogicalFont {
        match role {
            UiFontRole::Caption => &self.caption,
            UiFontRole::Icon => &self.icon,
            UiFontRole::Menu => &self.menu,
            UiFontRole::MessageBox => &self.message_box,
            UiFontRole::SmallCaption => &self.small_caption,
            UiFontRole::StatusBar => &self.status_bar,
        }
    }
}

/// Reads `SPI_GETNONCLIENTMETRICS` using `SystemParametersInfoForDpi`.
///
/// Requires Windows 10 version 1607 or newer. No thread DPI-awareness mutation
/// occurs. The returned logical dimensions use the requested DPI.
pub fn non_client_metrics_for_dpi(dpi: NonZeroU32) -> Result<NonClientMetrics> {
    #[cfg(target_os = "windows")]
    {
        crate::native_preferences::non_client_metrics(dpi)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = dpi;
        Err(crate::Error::UnsupportedPlatform)
    }
}

/// Reads `SPI_GETICONTITLELOGFONT` using `SystemParametersInfoForDpi`.
///
/// Requires Windows 10 version 1607 or newer; returns an unmapped preference.
pub fn icon_title_font_for_dpi(dpi: NonZeroU32) -> Result<LogicalFont> {
    #[cfg(target_os = "windows")]
    {
        crate::native_preferences::icon_title_font(dpi)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = dpi;
        Err(crate::Error::UnsupportedPlatform)
    }
}

/// Captures all six role preferences, failing without a partial result.
///
/// The two Windows queries do not promise an atomic OS preference snapshot.
pub fn ui_fonts_for_dpi(dpi: NonZeroU32) -> Result<UiFonts> {
    let metrics = non_client_metrics_for_dpi(dpi)?;
    let icon = icon_title_font_for_dpi(dpi)?;
    Ok(UiFonts::from_preferences(metrics, icon))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preference(name: &str, height: i32, weight: i32, italic: u8) -> LogicalFont {
        LogicalFont {
            reference_dpi: NonZeroU32::new(96).unwrap(),
            face_name: name.to_owned(),
            face_name_utf16: name.encode_utf16().collect(),
            height,
            width: 0,
            escapement: 0,
            orientation: 0,
            weight,
            italic,
            underline: 0,
            strike_out: 0,
            char_set: 1,
            out_precision: 0,
            clip_precision: 0,
            quality: 0,
            pitch_and_family: 0,
        }
    }

    #[test]
    fn each_role_retains_its_own_native_preference() {
        let caption = preference("Caption", -21, 700, 1);
        let icon = preference("Icon", 17, 400, 0);
        let menu = preference("Menu", -18, 500, 0);
        let message = preference("Message", -19, 600, 1);
        let small = preference("Small", 14, 300, 0);
        let status = preference("Status", -16, 200, 1);
        let metrics = NonClientMetrics {
            reference_dpi: NonZeroU32::new(96).unwrap(),
            border_width: 1,
            scroll_width: 17,
            scroll_height: 17,
            caption_width: 18,
            caption_height: 18,
            caption_font: caption.clone(),
            small_caption_width: 15,
            small_caption_height: 15,
            small_caption_font: small.clone(),
            menu_width: 18,
            menu_height: 18,
            menu_font: menu.clone(),
            status_font: status.clone(),
            message_font: message.clone(),
            padded_border_width: 4,
        };
        let fonts = UiFonts::from_preferences(metrics, icon.clone());
        for (role, expected) in [
            (UiFontRole::Caption, caption),
            (UiFontRole::Icon, icon),
            (UiFontRole::Menu, menu),
            (UiFontRole::MessageBox, message),
            (UiFontRole::SmallCaption, small),
            (UiFontRole::StatusBar, status),
        ] {
            assert_eq!(fonts.font(role), &expected, "role {role:?}");
        }
    }
}
