//! Native profile admission and directional conversion policy.

use std::fmt;

use crate::accounting::{RenderAccountingContext, RenderAttemptKind, RenderPhase};

use super::{
    Account, Direction, Header, ProfileError, device_channel_count, find_tag, lut, media_white,
    mpe, pcs, read_tag_table,
};
use crate::{Error, Result};

const MAX_DEVICE_CHANNELS: usize = 15;

/// ICC classes eligible for calibrated document and print colors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IccProfileClass {
    /// A device-to-PCS input profile.
    Input,
    /// A display profile.
    Display,
    /// An output or print profile.
    Output,
    /// A calibrated color-space profile.
    ColorSpace,
}

/// The rendering intent requested for an ICC transformation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IccRenderingIntent {
    /// Use the profile's perceptual transform.
    Perceptual,
    /// Use media-relative colorimetry.
    MediaRelativeColorimetric,
    /// Use the profile's saturation transform.
    Saturation,
    /// Use ICC-absolute colorimetry.
    AbsoluteColorimetric,
}

impl IccRenderingIntent {
    const fn mpe_tag(self, direction: Direction) -> &'static [u8; 4] {
        match (direction, self) {
            (Direction::ToPcs, Self::Perceptual) => b"D2B0",
            (Direction::ToPcs, Self::MediaRelativeColorimetric) => b"D2B1",
            (Direction::ToPcs, Self::Saturation) => b"D2B2",
            (Direction::ToPcs, Self::AbsoluteColorimetric) => b"D2B3",
            (Direction::FromPcs, Self::Perceptual) => b"B2D0",
            (Direction::FromPcs, Self::MediaRelativeColorimetric) => b"B2D1",
            (Direction::FromPcs, Self::Saturation) => b"B2D2",
            (Direction::FromPcs, Self::AbsoluteColorimetric) => b"B2D3",
        }
    }

    const fn legacy_tags(self, direction: Direction) -> (&'static [u8; 4], &'static [u8; 4]) {
        let tags = match direction {
            Direction::ToPcs => [b"A2B0", b"A2B1", b"A2B2"],
            Direction::FromPcs => [b"B2A0", b"B2A1", b"B2A2"],
        };
        let preferred = match self {
            Self::Perceptual => tags[0],
            Self::MediaRelativeColorimetric | Self::AbsoluteColorimetric => tags[1],
            Self::Saturation => tags[2],
        };
        (preferred, tags[0])
    }
}

/// The profile connection space representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IccPcsSpace {
    /// XYZ in D50 PCS units.
    XyzD50,
    /// CIELAB in D50 PCS units.
    Lab,
}

/// A finite PCS color with its representation and rendering intent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IccPcsColor {
    space: IccPcsSpace,
    coordinates: [f64; 3],
    intent: IccRenderingIntent,
}

impl IccPcsColor {
    /// Validates actual PCS coordinates against the finite ICC float range.
    ///
    /// # Errors
    ///
    /// Returns [`IccProfileError::InvalidCoordinate`] for an invalid coordinate.
    pub fn try_new(
        space: IccPcsSpace,
        coordinates: [f64; 3],
        intent: IccRenderingIntent,
    ) -> Result<Self> {
        validate_coordinates(&coordinates)?;
        Ok(Self {
            space,
            coordinates,
            intent,
        })
    }

    /// Returns the PCS representation.
    pub const fn space(self) -> IccPcsSpace {
        self.space
    }

    /// Returns actual XYZ or CIELAB units in PCS channel order.
    pub const fn coordinates(self) -> [f64; 3] {
        self.coordinates
    }

    /// Returns the intent that describes these connection coordinates.
    pub const fn intent(self) -> IccRenderingIntent {
        self.intent
    }
}

/// A device data-space signature and its checked channel count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IccDeviceSpace {
    signature: [u8; 4],
    channels: u8,
}

impl IccDeviceSpace {
    /// Returns the exact ICC data-space signature.
    pub const fn signature(self) -> [u8; 4] {
        self.signature
    }

    /// Returns the number of channels defined by the signature.
    pub const fn channel_count(self) -> usize {
        self.channels as usize
    }
}

/// Checked device coordinates in the ICC-defined channel order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IccDeviceColor {
    space: IccDeviceSpace,
    channels: [f64; MAX_DEVICE_CHANNELS],
}

impl IccDeviceColor {
    /// Returns this color's device encoding.
    pub const fn space(self) -> IccDeviceSpace {
        self.space
    }

    /// Borrows the active channels in device order.
    pub fn channels(&self) -> &[f64] {
        &self.channels[..self.space.channel_count()]
    }
}

/// An inspected immutable profile with executable transforms qualified on demand.
#[derive(Clone, Copy)]
pub struct IccProfile<'a> {
    bytes: &'a [u8],
    header: Header,
    class: IccProfileClass,
    device: IccDeviceSpace,
    pcs: IccPcsSpace,
    media_white: [f64; 3],
}

impl fmt::Debug for IccProfile<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IccProfile")
            .field("class", &self.class)
            .field("device", &self.device)
            .field("pcs", &self.pcs)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl<'a> IccProfile<'a> {
    /// Checks profile structure and exposes its class and color spaces.
    ///
    /// The bytes remain in caller custody. Directional conversion validates
    /// the selected transform's payload and channel chain.
    ///
    /// # Errors
    ///
    /// Returns a typed ICC diagnostic for invalid, excluded, or unsupported
    /// profiles, or a typed accounting error if validation exhausts a limit.
    pub fn inspect(bytes: &'a [u8], context: &mut RenderAccountingContext) -> Result<Self> {
        context.run(
            RenderAttemptKind::IccProfile,
            RenderPhase::Preparation,
            |account| {
                account.phase(RenderPhase::IccProfilePrepare, || {
                    let account = account.at(RenderPhase::IccProfilePrepare);
                    let header = Header::read(bytes, &account).map_err(profile_error)?;
                    if !matches!(header.version[0], 2 | 4) {
                        return Err(IccProfileError::UnsupportedVersion.into());
                    }
                    let class = match &header.class {
                        b"scnr" => IccProfileClass::Input,
                        b"mntr" => IccProfileClass::Display,
                        b"prtr" => IccProfileClass::Output,
                        b"spac" => IccProfileClass::ColorSpace,
                        b"abst" | b"link" | b"nmcl" => {
                            return Err(IccProfileError::ExcludedClass.into());
                        }
                        _ => return Err(IccProfileError::InvalidProfile.into()),
                    };
                    let channels = device_channel_count(&header.data_space)
                        .ok_or(IccProfileError::InvalidProfile)?;
                    let device = IccDeviceSpace {
                        signature: header.data_space,
                        channels,
                    };
                    let pcs = match &header.pcs {
                        b"XYZ " => IccPcsSpace::XyzD50,
                        b"Lab " => IccPcsSpace::Lab,
                        _ => return Err(IccProfileError::InvalidProfile.into()),
                    };
                    let table = read_tag_table(bytes, header.native_tag_layout(), &account)
                        .map_err(profile_error)?;
                    let white =
                        media_white(&header, &table, bytes, &account).map_err(profile_error)?;
                    let media_white = if header.version[0] == 4 && class == IccProfileClass::Display
                    {
                        pcs::D50
                    } else {
                        white
                    };
                    Ok(Self {
                        bytes,
                        header,
                        class,
                        device,
                        pcs,
                        media_white,
                    })
                })
            },
        )
    }

    /// Returns the profile class.
    pub const fn class(self) -> IccProfileClass {
        self.class
    }

    /// Returns the device-space signature and channel count.
    pub const fn device_space(self) -> IccDeviceSpace {
        self.device
    }

    /// Returns the profile's connection space.
    pub const fn pcs_space(self) -> IccPcsSpace {
        self.pcs
    }

    /// Qualifies ordered device coordinates for this profile's encoding.
    ///
    /// Supply values in ICC device scale. A float transform can handle
    /// extended coordinates within the finite ICC range.
    ///
    /// # Errors
    ///
    /// Returns a typed outcome for invalid coordinates or channel count.
    pub fn device_color(&self, channels: &[f64]) -> Result<IccDeviceColor> {
        if channels.len() != self.device.channel_count() {
            return Err(IccProfileError::IncompatibleColor.into());
        }
        validate_coordinates(channels)?;
        let mut color = IccDeviceColor {
            space: self.device,
            channels: [0.0; MAX_DEVICE_CHANNELS],
        };
        color.channels[..channels.len()].copy_from_slice(channels);
        Ok(color)
    }

    /// Converts a calibrated device color to the requested PCS interpretation.
    ///
    /// # Errors
    ///
    /// Reports unavailable or malformed transforms, incompatible colors,
    /// numeric failures and Ledger admission refusals.
    pub fn to_pcs(
        &self,
        intent: IccRenderingIntent,
        color: &IccDeviceColor,
        context: &mut RenderAccountingContext,
    ) -> Result<IccPcsColor> {
        if color.space != self.device {
            return Err(IccProfileError::IncompatibleColor.into());
        }
        let mut coordinates = [0.0; 3];
        self.convert(
            Direction::ToPcs,
            intent,
            color.channels(),
            &mut coordinates,
            context,
        )?;
        Ok(IccPcsColor {
            space: self.pcs,
            coordinates,
            intent,
        })
    }

    /// Converts an intent-qualified PCS color to calibrated device coordinates.
    ///
    /// # Errors
    ///
    /// Reports unavailable or malformed transforms, incompatible colors,
    /// numeric failures and Ledger admission refusals.
    pub fn from_pcs(
        &self,
        color: IccPcsColor,
        context: &mut RenderAccountingContext,
    ) -> Result<IccDeviceColor> {
        if color.space != self.pcs {
            return Err(IccProfileError::IncompatibleColor.into());
        }
        let mut result = IccDeviceColor {
            space: self.device,
            channels: [0.0; MAX_DEVICE_CHANNELS],
        };
        self.convert(
            Direction::FromPcs,
            color.intent,
            &color.coordinates,
            &mut result.channels[..self.device.channel_count()],
            context,
        )?;
        Ok(result)
    }

    fn convert(
        &self,
        direction: Direction,
        intent: IccRenderingIntent,
        input: &[f64],
        output: &mut [f64],
        context: &mut RenderAccountingContext,
    ) -> Result<()> {
        context.run(
            RenderAttemptKind::IccProfile,
            RenderPhase::Preparation,
            |account| {
                let mut transform = account.phase(RenderPhase::IccProfilePrepare, || {
                    self.prepare(
                        direction,
                        intent,
                        input.len(),
                        output.len(),
                        &account.at(RenderPhase::IccProfilePrepare),
                    )
                    .map_err(profile_error)
                })?;
                account.phase(RenderPhase::IccProfileConvert, || {
                    let account = account.at(RenderPhase::IccProfileConvert);
                    match &mut transform {
                        NativeTransform::Mpe(mpe) => mpe.evaluate(input, output, &account),
                        NativeTransform::Lut(pipeline) => self.evaluate_relative(
                            direction,
                            intent,
                            input,
                            output,
                            &account,
                            |input, output| pipeline.evaluate(input, output, &account),
                        ),
                    }
                    .map_err(numeric_error)
                })
            },
        )
    }

    fn evaluate_relative(
        &self,
        direction: Direction,
        intent: IccRenderingIntent,
        input: &[f64],
        output: &mut [f64],
        account: &Account,
        evaluate: impl FnOnce(&[f64], &mut [f64]) -> Result<()>,
    ) -> Result<()> {
        if intent != IccRenderingIntent::AbsoluteColorimetric {
            return evaluate(input, output);
        }
        let adjust =
            |values| pcs::adjust_absolute(values, self.pcs, direction, self.media_white, account);
        match direction {
            Direction::ToPcs => {
                let destination = output
                    .first_chunk_mut::<3>()
                    .ok_or(IccProfileError::IncompatibleColor)?;
                let mut relative = [0.0; 3];
                evaluate(input, &mut relative)?;
                *destination = adjust(relative)?;
                Ok(())
            }
            Direction::FromPcs => {
                let values = input
                    .first_chunk::<3>()
                    .ok_or(IccProfileError::IncompatibleColor)?;
                evaluate(&adjust(*values)?, output)
            }
        }
    }

    fn prepare(
        &self,
        direction: Direction,
        intent: IccRenderingIntent,
        input: usize,
        output: usize,
        account: &Account,
    ) -> super::ProfileResult<NativeTransform<'a>> {
        let version = self.header.version[0];
        let table = read_tag_table(self.bytes, self.header.native_tag_layout(), account)?;
        if version == 4
            && let Some(tag) = find_tag(&table, intent.mpe_tag(direction), account)?
        {
            match mpe::prepare_borrowed(self.bytes, tag, input, output, account) {
                Ok(transform) => return Ok(NativeTransform::Mpe(transform)),
                Err(ProfileError::Unsupported) => (),
                Err(error) => return Err(error),
            }
        }

        let (preferred, fallback) = intent.legacy_tags(direction);
        let tag = match find_tag(&table, preferred, account)? {
            None if preferred != fallback => find_tag(&table, fallback, account)?,
            found => found,
        }
        .ok_or(ProfileError::Unsupported)?;
        let payload = tag.payload(self.bytes)?;
        let pipeline = lut::Lut::read(
            payload,
            version,
            (&self.header.data_space, &self.header.pcs),
            direction,
            input,
            output,
            account,
        )?;
        Ok(NativeTransform::Lut(pipeline))
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "a single attempt keeps its accounted MPE scratch handles inline"
)]
enum NativeTransform<'a> {
    Mpe(mpe::MpeTransform<'a>),
    Lut(lut::Lut<&'a [u8]>),
}

fn validate_coordinates(coordinates: &[f64]) -> Result<()> {
    if coordinates
        .iter()
        .all(|value| value.abs() <= f64::from(f32::MAX))
    {
        Ok(())
    } else {
        Err(IccProfileError::InvalidCoordinate.into())
    }
}

fn profile_error(error: ProfileError) -> Error {
    match error {
        ProfileError::Invalid => IccProfileError::InvalidProfile.into(),
        ProfileError::Unsupported => IccProfileError::UnsupportedTransform.into(),
        ProfileError::NonRgb => IccProfileError::IncompatibleColor.into(),
        ProfileError::Render(error) => numeric_error(error),
    }
}

fn numeric_error(error: Error) -> Error {
    match error.glyph_preparation_diagnostic() {
        Some(crate::text::GlyphPreparationError::Numeric {
            phase: crate::text::GlyphNumericPhase::BitmapColor,
            ..
        }) => IccProfileError::NumericOutput.into(),
        _ => error,
    }
}

/// A profile-admission or native ICC conversion refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IccProfileError {
    /// The profile framing, metadata or selected transform is malformed.
    InvalidProfile,
    /// Abstract, DeviceLink and NamedColor profiles are excluded from this operation.
    ExcludedClass,
    /// The datastream's ICC edition is outside the supported editions.
    UnsupportedVersion,
    /// The requested direction and intent have no executable transform.
    UnsupportedTransform,
    /// The source color and the designated profile encoding are incompatible.
    IncompatibleColor,
    /// An input value exceeds the finite ICC coordinate range.
    InvalidCoordinate,
    /// Correct evaluation failed to produce a representable finite output.
    NumericOutput,
}

impl fmt::Display for IccProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidProfile => "invalid ICC profile",
            Self::ExcludedClass => "ICC profile class is excluded",
            Self::UnsupportedVersion => "unsupported ICC profile version",
            Self::UnsupportedTransform => "ICC transform is unavailable",
            Self::IncompatibleColor => "color encoding is incompatible with the ICC profile",
            Self::InvalidCoordinate => "ICC coordinate is outside the finite supported range",
            Self::NumericOutput => "ICC transform did not produce a finite supported result",
        })
    }
}

impl std::error::Error for IccProfileError {}
