use super::Result;

/// Immediate clipboard operations supplied by the active host.
///
/// Calls perform their side effect during the handler callback. Errors are
/// returned directly to that callback; successful writes are not rolled back if
/// later callback work fails.
pub trait Clipboard {
    /// Reads the current text, or `None` when the clipboard has no text value.
    fn read_text(&mut self) -> Result<Option<String>>;
    /// Replaces the current text with `text`.
    fn write_text(&mut self, text: &str) -> Result<()>;
    /// Reads the current RGBA image, or `None` when the clipboard has no image value.
    fn read_image(&mut self) -> Result<Option<ClipboardImage>>;
    /// Replaces the current image with the borrowed RGBA image data.
    fn write_image(&mut self, image: ClipboardImageRef<'_>) -> Result<()>;
}

/// An owned clipboard image with RGBA bytes and caller-supplied dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardImage {
    /// Image width metadata in pixels.
    pub width: u32,
    /// Image height metadata in pixels.
    pub height: u32,
    /// Pixel bytes in RGBA order, copied verbatim from a writer or reader.
    pub rgba: Vec<u8>,
}

/// A borrowed clipboard image with RGBA bytes and caller-supplied dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipboardImageRef<'a> {
    /// Image width metadata in pixels.
    pub width: u32,
    /// Image height metadata in pixels.
    pub height: u32,
    /// Pixel bytes in RGBA order, borrowed and copied for the write call.
    pub rgba: &'a [u8],
}

/// In-memory clipboard implementation for tests and display-free hosts.
#[derive(Clone, Debug, Default)]
pub struct MemoryClipboard {
    text: Option<String>,
    image: Option<ClipboardImage>,
}

impl MemoryClipboard {
    /// Creates an empty in-memory clipboard.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Clipboard for MemoryClipboard {
    fn read_text(&mut self) -> Result<Option<String>> {
        Ok(self.text.clone())
    }

    fn write_text(&mut self, text: &str) -> Result<()> {
        self.text = Some(text.to_owned());
        Ok(())
    }

    fn read_image(&mut self) -> Result<Option<ClipboardImage>> {
        Ok(self.image.clone())
    }

    fn write_image(&mut self, image: ClipboardImageRef<'_>) -> Result<()> {
        self.image = Some(ClipboardImage {
            width: image.width,
            height: image.height,
            rgba: image.rgba.to_vec(),
        });
        Ok(())
    }
}
