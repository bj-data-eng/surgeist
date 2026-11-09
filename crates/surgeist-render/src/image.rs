use super::{BackendErrorCode, Error, PhysicalSize, Result, Size};
use crate::accounting::{
    Account, CpuAccount, RenderCpuScope, RenderWork, SharedValue, StorageAccount,
};
use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// A compact image fingerprint or caller-supplied resource handle.
///
/// This copyable value is not a collision-free proof of pixel equality and
/// must not be used as the sole identity for a backend cache. Equality compares
/// only the underlying `u64`; the value carries no lifetime, generation, or
/// uniqueness guarantee. [`Image::from_rgba`] derives it deterministically,
/// while [`ImageId::new`] accepts a caller-managed handle.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageId(u64);

impl ImageId {
    /// Creates a compact image fingerprint or caller-supplied resource handle.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the compact underlying value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub(crate) struct ImageContentIdentity {
    fingerprint: ImageId,
    content_hash: u64,
    width: u32,
    height: u32,
    bytes: Arc<[u8]>,
}

impl ImageContentIdentity {
    fn new(fingerprint: ImageId, width: u32, height: u32, bytes: Arc<[u8]>) -> Self {
        let content_hash = stable_hash(&(width, height, bytes.as_ref()));
        Self {
            fingerprint,
            content_hash,
            width,
            height,
            bytes,
        }
    }

    /// Cached ordering facts only; exact history equality still compares bytes.
    pub(crate) const fn content_key(&self) -> (u64, u32, u32) {
        (self.content_hash, self.width, self.height)
    }

    pub(crate) const fn source_bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }

    #[cfg(test)]
    pub(crate) const fn fingerprint(&self) -> ImageId {
        self.fingerprint
    }
}

impl fmt::Debug for ImageContentIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImageContentIdentity")
            .field("fingerprint", &self.fingerprint)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl PartialEq for ImageContentIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.bytes == other.bytes
    }
}

impl Eq for ImageContentIdentity {}

impl Hash for ImageContentIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
    }
}

/// A validated, shareable RGBA8 image and its sampling configuration.
///
/// Exact render-owned content identity includes the dimensions and shared
/// pixel bytes. [`ImageId`] is retained only as a compact fingerprint, while
/// the Peniko blob carries its own unique backend identity. Image equality uses
/// exact content plus quality and extend policy rather than [`ImageId`] alone.
#[derive(Clone, Debug)]
pub struct Image {
    id: ImageId,
    content_identity: ImageContentIdentity,
    pub(crate) size: Size,
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) data: peniko::ImageData,
    pub(crate) sampling: ImageSampling,
    pub(crate) extend: Extend,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.content_identity == other.content_identity
            && self.sampling == other.sampling
            && self.extend == other.extend
    }
}

impl Image {
    /// Creates a validated RGBA8 image with a deterministic compact fingerprint.
    ///
    /// Independently constructed images receive distinct backend blob IDs even
    /// when their dimensions and bytes are equal. Returns an image-upload
    /// diagnostic when dimensions are not finite non-negative integer pixels,
    /// exceed `u32`, or the byte length is not exactly `width * height * 4`.
    pub fn from_rgba(size: Size, data: impl Into<Arc<[u8]>>) -> Result<Self> {
        let data = data.into();
        validate_rgba_image(size, data.len())?;
        let id = stable_hash(&(
            size.width().to_bits(),
            size.height().to_bits(),
            data.as_ref(),
        ));
        Self::from_validated_rgba_with_id(size, data, ImageId::new(id))
    }

    fn from_validated_rgba_with_id(size: Size, data: Arc<[u8]>, id: ImageId) -> Result<Self> {
        let width = image_dimension(size.width(), "width")?;
        let height = image_dimension(size.height(), "height")?;
        let content_identity = ImageContentIdentity::new(id, width, height, Arc::clone(&data));
        let image = peniko::ImageData {
            data: peniko::Blob::new(Arc::new(Arc::clone(&data))),
            format: peniko::ImageFormat::Rgba8,
            alpha_type: peniko::ImageAlphaType::Alpha,
            width,
            height,
        };
        Ok(Self {
            id,
            content_identity,
            size,
            bytes: data,
            data: image,
            sampling: ImageSampling::Quality(ImageQuality::Medium),
            extend: Extend::Pad,
        })
    }

    #[cfg(test)]
    pub(crate) fn from_rgba_with_id_for_test(
        size: Size,
        data: impl Into<Arc<[u8]>>,
        id: ImageId,
    ) -> Result<Self> {
        let data = data.into();
        validate_rgba_image(size, data.len())?;
        Self::from_validated_rgba_with_id(size, data, id)
    }

    #[cfg(test)]
    pub(crate) fn with_content_hash_for_test(mut self, hash: u64) -> Self {
        self.content_identity.content_hash = hash;
        self
    }

    /// Returns the compact content fingerprint.
    ///
    /// This value alone does not prove pixel equality or identify backend
    /// residency without collisions.
    #[must_use]
    pub const fn id(&self) -> ImageId {
        self.id
    }

    #[must_use]
    /// Returns the logical image dimensions, which represent integer pixel counts.
    pub const fn size(&self) -> Size {
        self.size
    }

    #[must_use]
    /// Returns this image with a different sampling-quality hint.
    ///
    /// The exact pixel content and backend blob identity are unchanged.
    pub const fn quality(mut self, quality: ImageQuality) -> Self {
        self.sampling = ImageSampling::Quality(quality);
        self
    }

    /// Selects the rendering hint supplied by the surrounding style owner.
    ///
    /// This replaces a previous quality or rendering selection. The caller
    /// applies inheritance and supplies the selected hint for each image;
    /// Render does not inspect elements or computed style.
    #[must_use]
    pub const fn rendering(mut self, rendering: ImageRendering) -> Self {
        self.sampling = ImageSampling::Rendering(rendering);
        self
    }

    /// Returns the selected rendering hint, if one was supplied.
    ///
    /// Selecting [`Self::quality`] clears the rendering hint.
    #[must_use]
    pub const fn rendering_hint(&self) -> Option<ImageRendering> {
        self.sampling.rendering_hint()
    }

    pub(crate) const fn sampling_quality(&self) -> ImageQuality {
        self.sampling.quality()
    }

    #[must_use]
    /// Returns this image with a different out-of-bounds sampling policy.
    ///
    /// The exact pixel content and backend blob identity are unchanged.
    pub const fn extend(mut self, extend: Extend) -> Self {
        self.extend = extend;
        self
    }

    pub(crate) const fn content_identity(&self) -> &ImageContentIdentity {
        &self.content_identity
    }
}

pub(crate) fn validate_rgba_image(size: Size, byte_len: usize) -> Result<()> {
    let width = image_dimension(size.width(), "width")?;
    let height = image_dimension(size.height(), "height")?;
    let expected_len = u64::from(width)
        .saturating_mul(u64::from(height))
        .saturating_mul(4);
    let actual_len = u64::try_from(byte_len).unwrap_or(u64::MAX);
    if actual_len != expected_len {
        return Err(Error::new(
            BackendErrorCode::ImageUploadFailed,
            format!(
                "RGBA image data length {byte_len} does not match {}x{} image size; expected {expected_len} bytes",
                width, height
            ),
        ));
    }
    Ok(())
}

pub(crate) fn image_dimension(value: f64, name: &str) -> Result<u32> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > f64::from(u32::MAX) {
        return Err(Error::new(
            BackendErrorCode::ImageUploadFailed,
            format!("image {name} {value} must be a finite non-negative integer pixel size"),
        ));
    }
    Ok(value as u32)
}

/// A backend sampling-quality hint for an image.
///
/// The default is [`Self::Medium`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ImageQuality {
    /// Prefer lower-cost, lower-quality sampling.
    Low,
    /// Use the balanced default sampling quality.
    #[default]
    Medium,
    /// Prefer higher-quality sampling.
    High,
}

/// A selected image-scaling hint, independent of stylesheet syntax and inheritance.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ImageRendering {
    /// Use the renderer's default, currently bilinear sampling.
    #[default]
    Auto,
    /// Smooth the scaled image using bilinear sampling.
    Smooth,
    /// Prefer higher-quality smoothing, currently bicubic sampling.
    ///
    /// Rendering does not currently degrade image quality under resource pressure.
    HighQuality,
    /// Expand each axis to its nearest positive integer multiple with nearest
    /// neighbor, then smoothly scale the expanded image to its destination.
    Pixelated,
    /// Preserve edges and source colors using nearest-neighbor sampling.
    CrispEdges,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageSampling {
    Quality(ImageQuality),
    Rendering(ImageRendering),
}

impl ImageSampling {
    pub(crate) const fn quality(self) -> ImageQuality {
        match self {
            Self::Quality(quality) => quality,
            Self::Rendering(ImageRendering::CrispEdges) => ImageQuality::Low,
            Self::Rendering(ImageRendering::HighQuality) => ImageQuality::High,
            Self::Rendering(
                ImageRendering::Auto | ImageRendering::Smooth | ImageRendering::Pixelated,
            ) => ImageQuality::Medium,
        }
    }

    pub(crate) const fn rendering_hint(self) -> Option<ImageRendering> {
        match self {
            Self::Quality(_) => None,
            Self::Rendering(rendering) => Some(rendering),
        }
    }
}

pub(crate) fn pixelated_multipliers(source_to_device: kurbo::Affine) -> Result<[u32; 2]> {
    let [a, b, c, d, _, _] = source_to_device.as_coeffs();
    let mut multiples = [1; 2];
    for (multiple, scale) in multiples.iter_mut().zip([a.hypot(b), c.hypot(d)]) {
        let rounded = scale.round().max(1.0);
        if !scale.is_finite() || rounded > f64::from(u32::MAX) {
            return Err(Error::invalid_value(
                "pixelated image scale",
                scale,
                "must be finite with a positive integer multiple representable as u32",
            ));
        }
        *multiple = rounded as u32;
    }
    Ok(multiples)
}

/// Image sampling behavior outside the source bounds.
///
/// The default is edge padding through [`Self::Pad`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Extend {
    /// Clamp sampling to the nearest edge pixel.
    #[default]
    Pad,
    /// Tile the image in the same orientation.
    Repeat,
    /// Tile the image while alternating orientation at each boundary.
    Reflect,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedMaskUploadKey {
    payload: SharedValue<MaskKeyPayload>,
}

#[derive(Debug)]
struct MaskKeyPayload {
    content_identity: ImageContentIdentity,
    physical_size: PhysicalSize,
    sampling: ImageSampling,
    extend: Extend,
    cpu: CpuAccount,
}
impl Drop for MaskKeyPayload {
    fn drop(&mut self) {
        self.cpu
            .at(RenderCpuScope::Cleanup)
            .maintenance(RenderWork::ImageIdentityDrops, 1);
    }
}

impl ResolvedMaskUploadKey {
    fn new_with_cpu(
        image: &Image,
        physical_size: PhysicalSize,
        sampling: ImageSampling,
        extend: Extend,
        cpu: &CpuAccount,
        forward: Option<&Account>,
    ) -> Result<Self> {
        let import = cpu.import_image_pixels(image.content_identity(), forward)?;
        match forward {
            Some(account) => account.work(RenderWork::ImageIdentityCopies, 1)?,
            None => cpu.maintenance(RenderWork::ImageIdentityCopies, 1),
        }
        let payload = MaskKeyPayload {
            content_identity: image.content_identity().clone(),
            physical_size,
            sampling,
            extend,
            cpu: cpu.clone(),
        };
        Ok(Self {
            payload: SharedValue::with_parts(
                payload,
                None,
                Some(import),
                &StorageAccount::Renderer(cpu.clone()),
            )?,
        })
    }

    /// Visible candidate equality. Hash-map bucket equality remains internal;
    /// callers admit their visible probes before invoking this comparison.
    pub(crate) fn matches_in(&self, other: &Self, observer: Option<&Account>) -> Result<bool> {
        match observer {
            Some(account) => self.matches_body(other, || {
                account.work(RenderWork::ImageIdentityByteComparisons, 1)
            }),
            None => {
                let cpu = self.payload.cpu.at(RenderCpuScope::Cleanup);
                let mut result = Ok(false);
                cpu.maintenance_phase(|| {
                    result = self.matches_body(other, || {
                        cpu.maintenance(RenderWork::ImageIdentityByteComparisons, 1);
                        Ok(())
                    });
                });
                result
            }
        }
    }
    fn matches_body(&self, other: &Self, mut compared: impl FnMut() -> Result<()>) -> Result<bool> {
        let left = &self.payload;
        let right = &other.payload;
        if left.content_identity.content_key() != right.content_identity.content_key()
            || left.physical_size != right.physical_size
            || left.sampling != right.sampling
            || left.extend != right.extend
        {
            return Ok(false);
        }
        let left_bytes = left.content_identity.source_bytes();
        let right_bytes = right.content_identity.source_bytes();
        if Arc::ptr_eq(left_bytes, right_bytes) {
            return Ok(true);
        }
        if left_bytes.len() != right_bytes.len() {
            return Ok(false);
        }
        for (left_byte, right_byte) in left_bytes.iter().zip(right_bytes.iter()) {
            compared()?;
            if left_byte != right_byte {
                return Ok(false);
            }
        }
        Ok(true)
    }

    #[cfg(test)]
    pub(crate) fn from_image_for_test(
        image: &Image,
        physical_size: PhysicalSize,
        quality: ImageQuality,
        extend: Extend,
    ) -> Self {
        let account = Account::for_test();
        Self::new_with_cpu(
            image,
            physical_size,
            ImageSampling::Quality(quality),
            extend,
            account.cpu().expect("mask key fixture CPU"),
            Some(&account),
        )
        .expect("mask key fixture")
    }
    #[cfg(test)]
    pub(crate) fn image_id(&self) -> ImageId {
        self.payload.content_identity.fingerprint()
    }
    #[cfg(test)]
    pub(crate) fn physical_size(&self) -> PhysicalSize {
        self.payload.physical_size
    }
    #[cfg(test)]
    pub(crate) fn quality(&self) -> ImageQuality {
        self.payload.sampling.quality()
    }
    #[cfg(test)]
    pub(crate) fn extend(&self) -> Extend {
        self.payload.extend
    }
}
impl PartialEq for ResolvedMaskUploadKey {
    fn eq(&self, other: &Self) -> bool {
        self.payload.physical_size == other.payload.physical_size
            && self.payload.sampling == other.payload.sampling
            && self.payload.extend == other.payload.extend
            && self.payload.content_identity == other.payload.content_identity
    }
}
impl Eq for ResolvedMaskUploadKey {}
impl Hash for ResolvedMaskUploadKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.payload.content_identity.hash(state);
        self.payload.physical_size.hash(state);
        match self.payload.sampling.quality() {
            ImageQuality::Low => 0_u8,
            ImageQuality::Medium => 1,
            ImageQuality::High => 2,
        }
        .hash(state);
        self.payload.sampling.rendering_hint().hash(state);
        match self.payload.extend {
            Extend::Pad => 0_u8,
            Extend::Repeat => 1,
            Extend::Reflect => 2,
        }
        .hash(state);
    }
}

/// Mapping policy for fitting an image into its destination rectangle.
///
/// The current default [`Self::None`] uses the same independent axis scaling as
/// [`Self::Fill`] and [`Self::Stretch`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ImageFit {
    /// Scale independently along each axis to fill the destination rectangle.
    Fill,
    /// Scale to fit entirely inside the destination while preserving aspect ratio.
    Contain,
    /// Scale to cover the destination while preserving aspect ratio.
    Cover,
    /// Stretch independently along each axis to match the destination rectangle.
    Stretch,
    /// Use the current default mapping, which scales independently to the destination rectangle.
    #[default]
    None,
}

/// An intrinsically valid headless-readback buffer of straight-alpha RGBA8 physical pixels.
///
/// The byte length always equals `width * height * 4`. Zero-area buffers are
/// represented by an empty byte vector. This resolved CPU-visible value is
/// produced only by explicit [`crate::Renderer::read_headless`] or by validated
/// caller construction; production rendering does not consume it as a fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageBuffer {
    size: PhysicalSize,
    rgba: Vec<u8>,
}

impl ImageBuffer {
    /// Creates a buffer when the straight-alpha RGBA8 byte length matches its physical size.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when `width * height * 4`
    /// overflows addressable memory or differs from `rgba.len()`.
    pub fn try_new(size: PhysicalSize, rgba: Vec<u8>) -> Result<Self> {
        let expected_len = usize::try_from(size.width())
            .ok()
            .and_then(|width| {
                usize::try_from(size.height())
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixel_count| pixel_count.checked_mul(4))
            .ok_or_else(|| {
                Error::invalid_value(
                    "image buffer byte length",
                    format!("{}x{} RGBA8", size.width(), size.height()),
                    "must fit addressable memory",
                )
            })?;

        if rgba.len() != expected_len {
            return Err(Error::invalid_value(
                "image buffer RGBA data length",
                rgba.len(),
                "must equal width multiplied by height multiplied by 4",
            ));
        }

        Ok(Self { size, rgba })
    }

    /// Returns the physical pixel dimensions.
    #[must_use]
    pub const fn size(&self) -> PhysicalSize {
        self.size
    }

    /// Returns the tightly packed, row-major straight-alpha RGBA8 bytes.
    #[must_use]
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// Consumes the buffer and returns its tightly packed, row-major straight-alpha RGBA8 bytes.
    #[must_use]
    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }
}

/// Backend-phase upload facts for one normalized resolved alpha-mask image.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedMaskUploadDescriptor {
    image: Image,
    physical_size: PhysicalSize,
    row_bytes: u32,
    byte_len: u64,
}

impl PartialEq for ResolvedMaskUploadDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.physical_size == other.physical_size
            && self.image.sampling == other.image.sampling
            && self.image.extend == other.image.extend
            && self.image.content_identity == other.image.content_identity
    }
}

impl Eq for ResolvedMaskUploadDescriptor {}

impl ResolvedMaskUploadDescriptor {
    pub(crate) fn try_from_image(image: Image) -> Result<Self> {
        let width = image_dimension(image.size.width(), "width")?;
        let height = image_dimension(image.size.height(), "height")?;
        let physical_size = PhysicalSize::new(width, height);
        let row_bytes = width.checked_mul(4).ok_or_else(|| {
            Error::invalid_value(
                "resolved mask upload row length",
                width,
                "must fit in u32 bytes",
            )
        })?;
        let byte_len = u64::from(row_bytes)
            .checked_mul(u64::from(height))
            .ok_or_else(|| {
                Error::invalid_value(
                    "resolved mask upload byte length",
                    format!("{row_bytes}x{height}"),
                    "must fit in u64",
                )
            })?;
        let descriptor = Self {
            image,
            physical_size,
            row_bytes,
            byte_len,
        };
        descriptor.validate_upload_byte_len(descriptor.bytes().len())?;
        Ok(descriptor)
    }

    pub(crate) fn validate_upload_byte_len(&self, actual_len: usize) -> Result<()> {
        let actual_len = u64::try_from(actual_len).map_err(|_| {
            Error::invalid_value(
                "resolved mask upload byte length",
                actual_len,
                "must fit in u64",
            )
        })?;
        if actual_len != self.byte_len {
            return Err(Error::invalid_value(
                "resolved mask upload byte length",
                actual_len,
                "must equal width multiplied by height multiplied by four",
            ));
        }
        Ok(())
    }

    pub(crate) const fn image(&self) -> &Image {
        &self.image
    }

    pub(crate) const fn physical_size(&self) -> PhysicalSize {
        self.physical_size
    }

    pub(crate) const fn row_bytes(&self) -> u32 {
        self.row_bytes
    }

    pub(crate) const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    pub(crate) const fn quality(&self) -> ImageQuality {
        self.image.sampling_quality()
    }

    pub(crate) const fn rendering_hint(&self) -> Option<ImageRendering> {
        self.image.rendering_hint()
    }

    pub(crate) const fn extend(&self) -> Extend {
        self.image.extend
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.image.bytes
    }

    pub(crate) fn cache_key_in(&self, account: &Account) -> Result<ResolvedMaskUploadKey> {
        self.cache_key_with_cpu(account.cpu()?, Some(account))
    }
    pub(crate) fn cache_key_with_cpu(
        &self,
        cpu: &CpuAccount,
        forward: Option<&Account>,
    ) -> Result<ResolvedMaskUploadKey> {
        ResolvedMaskUploadKey::new_with_cpu(
            self.image(),
            self.physical_size,
            self.image.sampling,
            self.image.extend,
            cpu,
            forward,
        )
    }
}

#[cfg(test)]
pub(crate) fn validate_image_buffer_rgba_len(size: PhysicalSize, byte_len: usize) -> Result<()> {
    if size.width() == 0 {
        return Err(Error::invalid_value(
            "image buffer width",
            size.width(),
            "must be greater than 0 device pixels",
        ));
    }
    if size.height() == 0 {
        return Err(Error::invalid_value(
            "image buffer height",
            size.height(),
            "must be greater than 0 device pixels",
        ));
    }
    let expected_len = u64::from(size.width())
        .checked_mul(u64::from(size.height()))
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .ok_or_else(|| {
            Error::invalid_value(
                "image buffer byte length",
                format!("{}x{}", size.width(), size.height()),
                "must fit in u64",
            )
        })?;
    let actual_len = u64::try_from(byte_len).unwrap_or(u64::MAX);
    if actual_len != expected_len {
        return Err(Error::invalid_value(
            "image buffer RGBA data length",
            byte_len,
            "must match width multiplied by height multiplied by 4",
        ));
    }
    Ok(())
}

/// Converts the public sampling-quality hint to its equivalent Peniko value.
impl From<ImageQuality> for peniko::ImageQuality {
    fn from(quality: ImageQuality) -> Self {
        match quality {
            ImageQuality::Low => Self::Low,
            ImageQuality::Medium => Self::Medium,
            ImageQuality::High => Self::High,
        }
    }
}

/// Converts the public extension policy to its equivalent Peniko value.
impl From<Extend> for peniko::Extend {
    fn from(extend: Extend) -> Self {
        match extend {
            Extend::Pad => Self::Pad,
            Extend::Repeat => Self::Repeat,
            Extend::Reflect => Self::Reflect,
        }
    }
}

fn stable_hash<T: std::hash::Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod mask_key_accounting_tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
        RenderPhase, RenderProfile, RenderResource, RendererCpu,
    };

    fn image(bytes: [u8; 4]) -> Image {
        Image::from_rgba(Size::new(1.0, 1.0), Vec::from(bytes)).unwrap()
    }
    fn descriptor(image: Image) -> ResolvedMaskUploadDescriptor {
        ResolvedMaskUploadDescriptor::try_from_image(image).unwrap()
    }

    #[test]
    fn content_key_keeps_pixels_without_retaining_an_unused_blob_adapter() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let owner = cpu.at(RenderCpuScope::DeviceBacking);
        let image = image([1, 2, 3, 255]);
        let pixels = Arc::downgrade(&image.bytes);
        let (adapter, _) = image.data.data.clone().into_raw_parts();
        let adapter_lifetime = Arc::downgrade(&adapter);
        drop(adapter);
        let upload = descriptor(image);
        let key = upload.cache_key_with_cpu(&owner, None).unwrap();
        let clone = key.clone();

        // A persistent cache key retains the exact pixel identity, not a
        // renderable ImageData/Blob. Accounting must preserve that boundary.
        drop(upload);
        assert!(pixels.upgrade().is_some());
        assert!(
            adapter_lifetime.upgrade().is_none(),
            "pixel-only mask keys must not prolong the image Blob adapter",
        );
        drop(key);
        assert!(pixels.upgrade().is_some());
        drop(clone);
        assert!(pixels.upgrade().is_none());
        assert_eq!(cpu.snapshot().unwrap().fault(), None);
    }

    #[test]
    fn key_clones_share_payload_and_keep_original_source_until_last_drop() {
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
        )
        .unwrap();
        let owner = cpu.at(RenderCpuScope::DeviceBacking);
        let image = image([1, 2, 3, 255]);
        let source = Arc::downgrade(&image.bytes);
        let upload = descriptor(image);
        let key = upload.cache_key_with_cpu(&owner, None).unwrap();
        assert!(Arc::ptr_eq(
            key.payload.content_identity.source_bytes(),
            &upload.image().bytes
        ));
        let initial = cpu.snapshot().unwrap();
        let bytes = initial
            .scope_usage(RenderCpuScope::DeviceBacking)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes);
        assert!(bytes > 0);
        let clone = key.clone();
        let cloned = cpu.snapshot().unwrap();
        assert_eq!(
            cloned
                .scope_usage(RenderCpuScope::DeviceBacking)
                .unwrap()
                .live(RenderResource::RetainedCpuBytes),
            bytes
        );
        assert_eq!(cloned.usage().work(RenderWork::ImageIdentityCopies), 1);
        drop(upload);
        drop(key);
        assert!(source.upgrade().is_some());
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .scope_usage(RenderCpuScope::DeviceBacking)
                .unwrap()
                .live(RenderResource::RetainedCpuBytes),
            bytes
        );
        drop(clone);
        assert!(source.upgrade().is_none());
        let closed = cpu.snapshot().unwrap();
        assert_eq!(
            closed
                .scope_usage(RenderCpuScope::DeviceBacking)
                .unwrap()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
        assert_eq!(closed.usage().work(RenderWork::ImageIdentityDrops), 1);
    }

    #[test]
    fn comparisons_skip_shared_bytes_and_count_exact_prefix_through_first_difference() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let owner = cpu.at(RenderCpuScope::DeviceBacking);
        let first = descriptor(image([10, 20, 30, 255]).with_content_hash_for_test(7));
        let equal = descriptor(image([10, 20, 30, 255]).with_content_hash_for_test(7));
        let changed = descriptor(image([10, 20, 31, 255]).with_content_hash_for_test(7));
        let differently_sampled = descriptor(
            image([10, 20, 30, 255])
                .with_content_hash_for_test(7)
                .quality(ImageQuality::High),
        );
        let left = first.cache_key_with_cpu(&owner, None).unwrap();
        let right = equal.cache_key_with_cpu(&owner, None).unwrap();
        let collision = changed.cache_key_with_cpu(&owner, None).unwrap();
        let sampling_key = differently_sampled
            .cache_key_with_cpu(&owner, None)
            .unwrap();
        let account = Account::for_test();
        assert!(left.matches_in(&left.clone(), Some(&account)).unwrap());
        assert_eq!(
            account
                .work_count(RenderWork::ImageIdentityByteComparisons)
                .unwrap(),
            0
        );
        assert!(!left.matches_in(&sampling_key, Some(&account)).unwrap());
        assert_eq!(
            account
                .work_count(RenderWork::ImageIdentityByteComparisons)
                .unwrap(),
            0
        );
        assert!(left.matches_in(&right, Some(&account)).unwrap());
        assert_eq!(
            account
                .work_count(RenderWork::ImageIdentityByteComparisons)
                .unwrap(),
            4
        );
        assert!(!left.matches_in(&collision, Some(&account)).unwrap());
        assert_eq!(
            account
                .work_count(RenderWork::ImageIdentityByteComparisons)
                .unwrap(),
            7
        );
        assert_eq!(left, right);
        assert_ne!(left, collision);
    }

    #[test]
    fn comparison_refusal_preserves_accepted_byte_prefix_and_retry() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let owner = cpu.at(RenderCpuScope::DeviceBacking);
        let left = descriptor(image([10, 20, 30, 255]).with_content_hash_for_test(7))
            .cache_key_with_cpu(&owner, None)
            .unwrap();
        let right = descriptor(image([10, 20, 31, 255]).with_content_hash_for_test(7))
            .cache_key_with_cpu(&owner, None)
            .unwrap();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ImageIdentityByteComparisons, 2)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        assert!(
            context
                .run(
                    RenderAttemptKind::SceneInspection,
                    RenderPhase::SceneInspect,
                    |account| left.matches_in(&right, Some(account))
                )
                .is_err()
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ImageIdentityByteComparisons),
            2
        );
        context.set_options(RenderAccountingOptions::default());
        assert!(
            !context
                .run(
                    RenderAttemptKind::SceneInspection,
                    RenderPhase::SceneInspect,
                    |account| left.matches_in(&right, Some(account))
                )
                .unwrap()
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ImageIdentityByteComparisons),
            3
        );
        assert!(!left.matches_in(&right, None).unwrap());
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ImageIdentityByteComparisons),
            3
        );
    }
}
