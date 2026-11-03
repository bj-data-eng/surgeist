//! Audited fixed-action preference calls. No caller can select an action,
//! supply a buffer, construct a handle, or call the raw binding.

use crate::{Error, LogicalFont, NonClientMetrics, Result};
use std::{ffi::c_void, mem::size_of, num::NonZeroU32};

#[repr(C)]
#[derive(Default)]
struct LogFont {
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
    face_name: [u16; 32],
}

#[repr(C)]
#[derive(Default)]
struct Metrics {
    size: u32,
    border_width: i32,
    scroll_width: i32,
    scroll_height: i32,
    caption_width: i32,
    caption_height: i32,
    caption_font: LogFont,
    small_caption_width: i32,
    small_caption_height: i32,
    small_caption_font: LogFont,
    menu_width: i32,
    menu_height: i32,
    menu_font: LogFont,
    status_font: LogFont,
    message_font: LogFont,
    padded_border_width: i32,
}

// ABI sizes and alignment are fixed by the documented Windows structures on
// both 32- and 64-bit Windows: neither contains pointers or architecture-sized integers.
const _: () = assert!(size_of::<LogFont>() == 92 && std::mem::align_of::<LogFont>() == 4);
const _: () = assert!(size_of::<Metrics>() == 504 && std::mem::align_of::<Metrics>() == 4);
const _: () = assert!(std::mem::offset_of!(LogFont, face_name) == 28);
const _: () = assert!(std::mem::offset_of!(Metrics, caption_font) == 24);
const _: () = assert!(std::mem::offset_of!(Metrics, small_caption_font) == 124);
const _: () = assert!(std::mem::offset_of!(Metrics, menu_font) == 224);
const _: () = assert!(std::mem::offset_of!(Metrics, status_font) == 316);
const _: () = assert!(std::mem::offset_of!(Metrics, message_font) == 408);
const _: () = assert!(std::mem::offset_of!(Metrics, padded_border_width) == 500);

#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoForDpi(
        action: u32,
        size: u32,
        output: *mut c_void,
        flags: u32,
        dpi: u32,
    ) -> i32;
}

impl LogFont {
    fn into_owned(self, reference_dpi: NonZeroU32) -> Result<LogicalFont> {
        if !(0..=1000).contains(&self.weight) {
            return Err(Error::InvalidData);
        }
        let end = self
            .face_name
            .iter()
            .position(|unit| *unit == 0)
            .ok_or(Error::InvalidData)?;
        let face_name_utf16 = self.face_name[..end].to_vec();
        let face_name = String::from_utf16(&face_name_utf16).map_err(|_| Error::InvalidData)?;
        Ok(LogicalFont {
            reference_dpi,
            face_name,
            face_name_utf16,
            height: self.height,
            width: self.width,
            escapement: self.escapement,
            orientation: self.orientation,
            weight: self.weight,
            italic: self.italic,
            underline: self.underline,
            strike_out: self.strike_out,
            char_set: self.char_set,
            out_precision: self.out_precision,
            clip_precision: self.clip_precision,
            quality: self.quality,
            pitch_and_family: self.pitch_and_family,
        })
    }
}

fn failure() -> Error {
    Error::Native {
        operation: "SystemParametersInfoForDpi",
        code: winsafe::GetLastError().raw(),
    }
}

pub(super) fn non_client_metrics(dpi: NonZeroU32) -> Result<NonClientMetrics> {
    let mut output = Metrics {
        size: size_of::<Metrics>() as u32,
        ..Metrics::default()
    };
    winsafe::SetLastError(winsafe::co::ERROR::SUCCESS);
    // SAFETY: SPI_GETNONCLIENTMETRICS (0x29) requires NONCLIENTMETRICSW.
    // Metrics and nested LogFont have the documented repr(C) Windows layout,
    // including the Vista+ padded border field. cbSize/uiParam equal the complete
    // writable buffer size. The initialized, aligned local remains exclusively
    // borrowed and alive for this synchronous call; Windows retains no pointer.
    // flags=0 performs a query, and dpi is nonzero. Only this fixed action uses it.
    let succeeded = unsafe {
        SystemParametersInfoForDpi(
            0x29,
            output.size,
            (&mut output as *mut Metrics).cast(),
            0,
            dpi.get(),
        )
    };
    if succeeded == 0 {
        return Err(failure());
    }
    Ok(NonClientMetrics {
        reference_dpi: dpi,
        border_width: output.border_width,
        scroll_width: output.scroll_width,
        scroll_height: output.scroll_height,
        caption_width: output.caption_width,
        caption_height: output.caption_height,
        caption_font: output.caption_font.into_owned(dpi)?,
        small_caption_width: output.small_caption_width,
        small_caption_height: output.small_caption_height,
        small_caption_font: output.small_caption_font.into_owned(dpi)?,
        menu_width: output.menu_width,
        menu_height: output.menu_height,
        menu_font: output.menu_font.into_owned(dpi)?,
        status_font: output.status_font.into_owned(dpi)?,
        message_font: output.message_font.into_owned(dpi)?,
        padded_border_width: output.padded_border_width,
    })
}

pub(super) fn icon_title_font(dpi: NonZeroU32) -> Result<LogicalFont> {
    let mut output = LogFont::default();
    winsafe::SetLastError(winsafe::co::ERROR::SUCCESS);
    // SAFETY: SPI_GETICONTITLELOGFONT (0x1f) requires exactly LOGFONTW and its
    // byte size. LogFont has the documented repr(C) fixed-width Windows layout.
    // This initialized, aligned, exclusively borrowed local buffer outlives the
    // synchronous call and its pointer is not retained. flags=0 is query-only;
    // dpi is nonzero. No caller can vary the action or output type.
    let succeeded = unsafe {
        SystemParametersInfoForDpi(
            0x1f,
            size_of::<LogFont>() as u32,
            (&mut output as *mut LogFont).cast(),
            0,
            dpi.get(),
        )
    };
    if succeeded == 0 {
        return Err(failure());
    }
    output.into_owned(dpi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_weight_keeps_default_and_endpoints_without_clamping() {
        for weight in [0, 1, 400, 1000] {
            let raw = LogFont {
                weight,
                ..LogFont::default()
            };
            assert_eq!(
                raw.into_owned(NonZeroU32::new(96).unwrap())
                    .unwrap()
                    .weight(),
                weight
            );
        }
        for weight in [i32::MIN, -1, 1001, i32::MAX] {
            let raw = LogFont {
                weight,
                ..LogFont::default()
            };
            assert_eq!(
                raw.into_owned(NonZeroU32::new(96).unwrap()),
                Err(Error::InvalidData)
            );
        }
    }

    #[test]
    fn preference_preserves_signed_dimensions_defaults_and_native_bytes() {
        let raw = LogFont {
            height: -15,
            width: 0,
            escapement: -900,
            orientation: 2700,
            weight: 0,
            italic: 2,
            underline: 1,
            strike_out: 3,
            char_set: 204,
            out_precision: 7,
            clip_precision: 16,
            quality: 5,
            pitch_and_family: 34,
            ..LogFont::default()
        };
        let font = raw.into_owned(NonZeroU32::new(144).unwrap()).unwrap();
        assert_eq!(
            (
                font.height(),
                font.width(),
                font.escapement(),
                font.orientation()
            ),
            (-15, 0, -900, 2700)
        );
        assert_eq!(
            (
                font.weight(),
                font.italic(),
                font.underline(),
                font.strike_out()
            ),
            (0, 2, 1, 3)
        );
        assert_eq!(
            (
                font.char_set(),
                font.out_precision(),
                font.clip_precision(),
                font.quality(),
                font.pitch_and_family()
            ),
            (204, 7, 16, 5, 34)
        );
        assert_eq!(font.reference_dpi().get(), 144);
        assert_eq!(font.face_name(), "");
    }

    #[test]
    fn name_retains_unicode_without_replacement_and_rejects_malformed_native_data() {
        let mut raw = LogFont::default();
        let name: Vec<u16> = "UI 🦀".encode_utf16().collect();
        raw.face_name[..name.len()].copy_from_slice(&name);
        let font = raw.into_owned(NonZeroU32::new(96).unwrap()).unwrap();
        assert_eq!(font.face_name(), "UI 🦀");
        assert_eq!(font.face_name_utf16(), name);
        let invalid = LogFont {
            face_name: [0xd800; 32],
            ..LogFont::default()
        };
        assert_eq!(
            invalid.into_owned(NonZeroU32::new(96).unwrap()),
            Err(Error::InvalidData)
        );
        let mut invalid = LogFont::default();
        invalid.face_name[0] = 0xd800;
        assert_eq!(
            invalid.into_owned(NonZeroU32::new(96).unwrap()),
            Err(Error::InvalidData)
        );
    }
}
