//! Safe, typed Windows observations with owned values and opaque window identities.
//!
//! Native handles and the generic `SystemParametersInfo` interface are private.
//! Observations can become stale immediately; callers own refresh policy.

#![deny(unsafe_code)]
#![forbid(unsafe_op_in_unsafe_fn)]

use std::{error, fmt};

mod fonts;
pub use fonts::{
    LogicalFont, NonClientMetrics, UiFontRole, UiFonts, icon_title_font_for_dpi,
    non_client_metrics_for_dpi, ui_fonts_for_dpi,
};

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod native_preferences;

/// Failure to obtain a complete native observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// This target does not implement Windows observations.
    UnsupportedPlatform,
    /// A native query failed; the code is the Windows error value when supplied.
    Native { operation: &'static str, code: u32 },
    /// A bounded enumeration contained more records than the query allows.
    EnumerationLimit,
    /// An input contains an embedded NUL, which Windows would truncate.
    InteriorNul,
    /// Native data cannot satisfy the observation's documented numeric contract.
    InvalidData,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("Windows observations are unsupported on this target")
            }
            Self::Native { operation, code } => {
                write!(f, "{operation} failed (Windows error {code})")
            }
            Self::EnumerationLimit => f.write_str("native enumeration exceeded its record limit"),
            Self::InteriorNul => f.write_str("native query input contains an embedded NUL"),
            Self::InvalidData => f.write_str("native observation contains invalid data"),
        }
    }
}

impl error::Error for Error {}

/// A complete query result.
pub type Result<T> = std::result::Result<T, Error>;

/// Native rectangle edges in the coordinate space documented by the query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// A native integer point in the query's coordinate space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// An owned GDI monitor observation in virtual-screen coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Monitor {
    pub device_name: String,
    pub frame: Rect,
    pub work_area: Rect,
}

/// A borrowed native window identity obtained by top-level enumeration.
///
/// This does not keep the operating-system window alive. Each method can fail
/// if it closes, and Windows can reuse an identity between observations.
pub struct Window {
    #[cfg(target_os = "windows")]
    native: winsafe::HWND,
    #[cfg(not(target_os = "windows"))]
    _private: (),
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window").finish_non_exhaustive()
    }
}

#[cfg(target_os = "windows")]
mod platform;

/// Enumerates every GDI monitor, failing the entire query on any native failure.
pub fn monitors() -> Result<Vec<Monitor>> {
    #[cfg(target_os = "windows")]
    {
        platform::monitors()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(Error::UnsupportedPlatform)
    }
}

/// Returns owned device-interface paths for one GDI display-device name.
///
/// Preserves empty, duplicate, and multiple identities for the consuming adapter
/// to assess ambiguity. At most 128 records are allowed; overflow or failure
/// rejects the whole query. Every iterator step starts with a cleared last error.
pub fn display_interfaces(device_name: &str) -> Result<Vec<String>> {
    if device_name.contains('\0') {
        return Err(Error::InteriorNul);
    }
    #[cfg(target_os = "windows")]
    {
        platform::display_interfaces(device_name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = device_name;
        Err(Error::UnsupportedPlatform)
    }
}

/// Enumerates current top-level identities; no handle constructor is exposed.
pub fn top_level_windows() -> Result<Vec<Window>> {
    #[cfg(target_os = "windows")]
    {
        platform::top_level_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(Error::UnsupportedPlatform)
    }
}

#[cfg(not(target_os = "windows"))]
impl Window {
    /// Reads the native window title.
    pub fn title(&self) -> Result<String> {
        Err(Error::UnsupportedPlatform)
    }
    /// Reads the owning process identifier.
    pub fn process_id(&self) -> Result<u32> {
        Err(Error::UnsupportedPlatform)
    }
    /// Reads the containing monitor without choosing a nearest-monitor fallback.
    pub fn monitor(&self) -> Result<Option<Monitor>> {
        Err(Error::UnsupportedPlatform)
    }
    /// Reads native client rectangle edges in client coordinates.
    pub fn client_rect(&self) -> Result<Rect> {
        Err(Error::UnsupportedPlatform)
    }
    /// Reads the client origin in screen coordinates.
    pub fn client_origin(&self) -> Result<Point> {
        Err(Error::UnsupportedPlatform)
    }
    /// Reads the window's DPI according to its DPI awareness.
    pub fn dpi(&self) -> Result<u32> {
        Err(Error::UnsupportedPlatform)
    }
}

#[cfg(any(target_os = "windows", test))]
fn collect_bounded<T>(mut values: impl Iterator<Item = Result<T>>, limit: usize) -> Result<Vec<T>> {
    let mut collected = Vec::new();
    loop {
        match values.next() {
            None => return Ok(collected),
            Some(value) => {
                let value = value?;
                if collected.len() == limit {
                    return Err(Error::EnumerationLimit);
                }
                collected.push(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_preserves_owned_empty_duplicate_and_ambiguous_records() {
        let records = ["", "same", "same", "other"].map(|value| Ok(value.to_owned()));
        assert_eq!(
            collect_bounded(records.into_iter(), 4).unwrap(),
            ["", "same", "same", "other"]
        );
    }

    #[test]
    fn collection_fails_whole_result_on_error_or_truncation() {
        assert_eq!(
            collect_bounded([Ok(1), Err(Error::InvalidData)].into_iter(), 2),
            Err(Error::InvalidData)
        );
        assert_eq!(
            collect_bounded([Ok(1), Ok(2)].into_iter(), 1),
            Err(Error::EnumerationLimit)
        );
        assert_eq!(collect_bounded::<u8>([].into_iter(), 0), Ok(vec![]));
        assert_eq!(collect_bounded([Ok(1)].into_iter(), 1), Ok(vec![1]));
    }

    #[test]
    fn display_name_with_nul_is_rejected_before_native_truncation() {
        assert_eq!(
            display_interfaces("display\0other"),
            Err(Error::InteriorNul)
        );
    }
}
