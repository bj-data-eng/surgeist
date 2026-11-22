//! Native profile admission and directional conversion policy.

use std::fmt;

use crate::accounting::{RenderAccountingContext, RenderAttemptKind, RenderPhase};

use super::{
    Header, ProfileError, device_channel_count, find_tag, media_white, mpe, read_tag_table,
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
    const fn tag(self, direction: Direction) -> &'static [u8; 4] {
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
}

#[derive(Clone, Copy)]
enum Direction {
    ToPcs,
    FromPcs,
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
                    media_white(&header, &table, bytes, &account).map_err(profile_error)?;
                    Ok(Self {
                        bytes,
                        header,
                        class,
                        device,
                        pcs,
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
                    let account = account.at(RenderPhase::IccProfilePrepare);
                    if self.header.version[0] != 4 {
                        return Err(IccProfileError::UnsupportedTransform.into());
                    }
                    let table =
                        read_tag_table(self.bytes, self.header.native_tag_layout(), &account)
                            .map_err(profile_error)?;
                    let tag = find_tag(&table, intent.tag(direction), &account)?
                        .ok_or(IccProfileError::UnsupportedTransform)?;
                    mpe::prepare_borrowed(self.bytes, tag, input.len(), output.len(), &account)
                        .map_err(profile_error)
                })?;
                account.phase(RenderPhase::IccProfileConvert, || {
                    transform
                        .evaluate(input, output, &account.at(RenderPhase::IccProfileConvert))
                        .map_err(numeric_error)
                })
            },
        )
    }
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
