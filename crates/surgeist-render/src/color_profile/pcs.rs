//! Numeric bridge between ICC PCS and the intrinsic D50 conversion kernels.

use surgeist_color::{Finite, PredefinedColorSpace, RectangularCoordinates, RectangularSpace};

use super::{
    Account, Direction, Result,
    native::{IccPcsSpace, IccProfileError},
};
use crate::accounting::RenderWork;

pub(super) const D50: [f64; 3] = [0.9642, 1.0, 0.8249];
const CSS_D50: [f64; 3] = [3457.0 / 3585.0, 1.0, 986.0 / 1195.0];
const XYZ: RectangularSpace = RectangularSpace::Predefined(PredefinedColorSpace::XyzD50);

pub(crate) fn lab_to_xyz(lab: [f64; 3], account: &Account) -> Result<[f64; 3]> {
    let xyz = convert(lab, RectangularSpace::Lab, XYZ, account)?;
    scale(xyz, D50, CSS_D50, account)
}

fn xyz_to_lab(xyz: [f64; 3], account: &Account) -> Result<[f64; 3]> {
    let xyz = scale(xyz, CSS_D50, D50, account)?;
    convert(xyz, XYZ, RectangularSpace::Lab, account)
}

pub(super) fn adjust_absolute(
    mut coordinates: [f64; 3],
    space: IccPcsSpace,
    direction: Direction,
    media_white: [f64; 3],
    account: &Account,
) -> Result<[f64; 3]> {
    if media_white == D50 {
        return Ok(coordinates);
    }
    let (target, source) = match direction {
        Direction::ToPcs => (media_white, D50),
        Direction::FromPcs => (D50, media_white),
    };
    if space == IccPcsSpace::Lab {
        coordinates = lab_to_xyz(coordinates, account)?;
        coordinates = scale(coordinates, target, source, account)?;
        xyz_to_lab(coordinates, account)
    } else {
        scale(coordinates, target, source, account)
    }
}

fn scale(
    mut coordinates: [f64; 3],
    target: [f64; 3],
    source: [f64; 3],
    account: &Account,
) -> Result<[f64; 3]> {
    for ((value, to), from) in coordinates.iter_mut().zip(target).zip(source) {
        *value = account.coordinate(|| *value * (to / from))?;
    }
    Ok(coordinates)
}

fn convert(
    input: [f64; 3],
    source: RectangularSpace,
    target: RectangularSpace,
    account: &Account,
) -> Result<[f64; 3]> {
    let mut channels = [None; 3];
    for (channel, value) in channels.iter_mut().zip(input) {
        *channel = Some(
            account
                .coordinate(|| Finite::new(value))?
                .ok_or(IccProfileError::NumericOutput)?,
        );
    }
    account.work(RenderWork::CoordinateVisits, 3)?;
    let mapped = RectangularCoordinates::new(source, channels).convert_to(target);
    let mut output = [0.0; 3];
    for (destination, value) in output.iter_mut().zip(mapped.channels()) {
        *destination = account
            .coordinate(|| value.map(Finite::get))?
            .ok_or(IccProfileError::NumericOutput)?;
    }
    Ok(output)
}
