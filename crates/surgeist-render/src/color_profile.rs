//! Media-relative RGB source transforms from an in-memory ICC datastream.
//! ICC.1:2022 §§7, 8.3–8.4, 8.10, 10.6, 10.18 and Annex F:
//! <https://www.color.org/specifications/ICC.1-2022-05.pdf>.
//! ICC.1:2001-04 §§6.1–6.3 and 6.5:
//! <https://www.color.org/specifications/ICC.1-2001-04.pdf>.

use std::{cmp::Ordering, mem::size_of};

use crate::accounting::{Account, AccountedVec, Hold, RenderResource, RenderWork};
use crate::error::BackendErrorCode;
use crate::{Error, Result};

pub(crate) enum Selection {
    Invalid,
    NonRgb,
    Unsupported,
    RgbMatrix(RgbMatrixProfile),
}

/// A media-relative input/display transform with validated matrix columns and
/// forward curves. Sampled curves refer to extents in its immutable custody.
pub(crate) struct RgbMatrixProfile {
    data: Box<ProfileData>,
    // Dispose the fixed allocation and any sampled backing before release.
    _hold: Hold,
}

struct ProfileData {
    columns: [[f64; 3]; 3],
    curves: [Curve; 3],
    sampled_curves: Option<AccountedVec<u8>>,
}

pub(crate) fn prepare(bytes: AccountedVec<u8>, account: &Account) -> Result<Selection> {
    match prepare_matrix(&bytes, account) {
        Ok(mut profile) => {
            let needs_samples = profile
                .data
                .curves
                .iter()
                .any(|curve| matches!(curve, Curve::Sampled { .. }));
            profile.data.sampled_curves = needs_samples.then_some(bytes);
            Ok(Selection::RgbMatrix(profile))
        }
        Err(ProfileError::Invalid) => Ok(Selection::Invalid),
        Err(ProfileError::NonRgb) => Ok(Selection::NonRgb),
        Err(ProfileError::Unsupported) => Ok(Selection::Unsupported),
        Err(ProfileError::Render(error)) => Err(error),
    }
}

impl RgbMatrixProfile {
    pub(crate) fn xyz_d50(&self, samples: [u8; 3], account: &Account) -> Result<[f64; 3]> {
        let mut xyz = [0.0; 3];
        let curve_bytes = self.data.sampled_curves.as_deref().unwrap_or_default();
        for ((curve, column), sample) in
            self.data.curves.iter().zip(&self.data.columns).zip(samples)
        {
            let linear = curve.evaluate(sample, curve_bytes, account)?;
            for (value, tint) in xyz.iter_mut().zip(column) {
                *value = account.coordinate(|| linear.mul_add(*tint, *value))?;
            }
        }
        Ok(xyz)
    }
}

enum ProfileError {
    Invalid,
    NonRgb,
    Unsupported,
    Render(Error),
}

impl From<Error> for ProfileError {
    fn from(error: Error) -> Self {
        Self::Render(error)
    }
}

type ProfileResult<T> = std::result::Result<T, ProfileError>;

fn prepare_matrix(bytes: &[u8], account: &Account) -> ProfileResult<RgbMatrixProfile> {
    let header = Header::read(bytes, account)?;
    let contiguous = matches!(header.version, [4, 0x40..=0x49, 0, 0]);
    let table = read_tag_table(bytes, contiguous, account)?;
    let version = header.version[0];
    if !matches!(version, 2 | 4) {
        return Err(ProfileError::Unsupported);
    }
    if &header.data_space != b"RGB " {
        return Err(ProfileError::NonRgb);
    }
    if !matches!(&header.class, b"scnr" | b"mntr") {
        return Err(ProfileError::Unsupported);
    }

    // These entries are required for both RGB source models. Relative PCS
    // matrix columns already include chromatic adaptation and scaling.
    required_tag(&table, b"desc", account)?;
    required_tag(&table, b"cprt", account)?;
    let white = xyz_tag(&table, bytes, b"wtpt", account)?;
    for component in white {
        if account.coordinate(|| component <= 0.0)? {
            return Err(ProfileError::Invalid);
        }
    }

    // ICC.1:2022 §8.10.2 skips the optional multi-process route when the CMM
    // lacks its operations. The media-relative AToB1 or fallback AToB0 then
    // outranks the matrix and requires a supported lookup-table decoder.
    for signature in [b"A2B1", b"A2B0"] {
        if let Some(tag) = find_tag(&table, signature, account)? {
            let payload = tag.payload(bytes)?;
            let kind = tag_kind(payload, account)?;
            let minimum = match &kind {
                b"mft1" => 48,
                b"mft2" => 52,
                b"mAB " if version == 4 => 32,
                _ => return Err(ProfileError::Invalid),
            };
            if payload.len() < minimum {
                return Err(ProfileError::Invalid);
            }
            return Err(ProfileError::Unsupported);
        }
    }

    // Only the XYZ PCS can be produced by the matrix/TRC transform.
    if &header.pcs != b"XYZ " {
        return Err(ProfileError::Invalid);
    }
    let hold = account.acquire(RenderResource::ScratchCpuBytes, size_of::<ProfileData>())?;
    let mut transform = Box::new(ProfileData {
        columns: [[0.0; 3]; 3],
        curves: [Curve::Identity, Curve::Identity, Curve::Identity],
        sampled_curves: None,
    });
    for (column, signature) in transform
        .columns
        .iter_mut()
        .zip([b"rXYZ", b"gXYZ", b"bXYZ"])
    {
        *column = xyz_tag(&table, bytes, signature, account)?;
    }
    for (curve, signature) in transform.curves.iter_mut().zip([b"rTRC", b"gTRC", b"bTRC"]) {
        let tag = required_tag(&table, signature, account)?;
        *curve = Curve::read(tag, bytes, version, account)?;
    }
    Ok(RgbMatrixProfile {
        data: transform,
        _hold: hold,
    })
}

fn xyz_tag(
    table: &[Tag],
    bytes: &[u8],
    signature: &[u8; 4],
    account: &Account,
) -> ProfileResult<[f64; 3]> {
    let payload = required_tag(table, signature, account)?.payload(bytes)?;
    if tag_kind(payload, account)? != *b"XYZ " || payload.len() != 20 {
        return Err(ProfileError::Invalid);
    }
    let mut components = [0.0; 3];
    for (value, offset) in components.iter_mut().zip([8, 12, 16]) {
        *value = account.coordinate(|| fixed(payload, offset))??;
    }
    Ok(components)
}

struct Header {
    version: [u8; 4],
    class: [u8; 4],
    data_space: [u8; 4],
    pcs: [u8; 4],
}

impl Header {
    fn read(bytes: &[u8], account: &Account) -> ProfileResult<Self> {
        let header = bytes.get(..128).ok_or(ProfileError::Invalid)?;
        let size = account.coordinate(|| u32_at(header, 0))??;
        if usize::try_from(size) != Ok(bytes.len()) || field::<4>(header, 36)? != *b"acsp" {
            return Err(ProfileError::Invalid);
        }
        let version = account.coordinate(|| field::<4>(header, 8))??;
        if !matches!(version, [1..=9, minor, 0, 0] if minor >> 4 <= 9 && (minor & 0xf) <= 9) {
            return Err(ProfileError::Invalid);
        }
        let class = field::<4>(header, 12)?;
        let data_space = field::<4>(header, 16)?;
        let pcs = field::<4>(header, 20)?;
        if matches!(version[0], 2 | 4) {
            let reserved = if version[0] == 2 { 84 } else { 100 };
            if !matches!(
                &class,
                b"scnr" | b"mntr" | b"prtr" | b"link" | b"spac" | b"abst" | b"nmcl"
            ) || !is_color_space(&data_space)
                || if &class == b"link" {
                    !is_color_space(&pcs)
                } else {
                    !matches!(&pcs, b"XYZ " | b"Lab ")
                }
                || account.coordinate(|| u32_at(header, 64))?? > 3
                || !header[reserved..].iter().all(|&byte| byte == 0)
            {
                return Err(ProfileError::Invalid);
            }

            // §7.2.16 specifies D50 rounded to four decimal places, not a
            // single permitted fixed-point encoding of its X and Z values.
            for (offset, expected) in [(68, 9642.0), (72, 10000.0), (76, 8249.0)] {
                let component = account.coordinate(|| fixed(header, offset))??;
                if account.coordinate(|| (component * 10000.0).round() != expected)? {
                    return Err(ProfileError::Invalid);
                }
            }
        }
        Ok(Self {
            version,
            class,
            data_space,
            pcs,
        })
    }
}

fn is_color_space(signature: &[u8; 4]) -> bool {
    matches!(
        signature,
        b"XYZ "
            | b"Lab "
            | b"Luv "
            | b"YCbr"
            | b"Yxy "
            | b"RGB "
            | b"GRAY"
            | b"HSV "
            | b"HLS "
            | b"CMYK"
            | b"CMY "
            | [b'2'..=b'9' | b'A'..=b'F', b'C', b'L', b'R']
    )
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Extent {
    start: usize,
    end: usize,
}

impl Extent {
    fn payload(self, bytes: &[u8]) -> ProfileResult<&[u8]> {
        bytes.get(self.start..self.end).ok_or(ProfileError::Invalid)
    }
}

struct Tag {
    signature: [u8; 4],
    extent: Extent,
}

fn read_tag_table(
    bytes: &[u8],
    contiguous: bool,
    account: &Account,
) -> ProfileResult<AccountedVec<Tag>> {
    let count = account.coordinate(|| u32_at(bytes, 128))??;
    let count = usize::try_from(count).map_err(|_| ProfileError::Invalid)?;
    let table_end = count
        .checked_mul(12)
        .and_then(|size| size.checked_add(132))
        .ok_or(ProfileError::Invalid)?;
    let records = bytes.get(132..table_end).ok_or(ProfileError::Invalid)?;
    let mut tags = account.temporary(count)?;
    // Every tag is framed and indexed. Payloads outside the selected
    // transform remain opaque, including private and descriptive tags.
    for record in records.as_chunks::<12>().0 {
        account.work(RenderWork::IccProfileTagVisits, 1)?;
        let start = usize::try_from(u32_at(record, 4)?).map_err(|_| ProfileError::Invalid)?;
        let size = usize::try_from(u32_at(record, 8)?).map_err(|_| ProfileError::Invalid)?;
        let end = start.checked_add(size).ok_or(ProfileError::Invalid)?;
        if size < 8 || start < table_end || !start.is_multiple_of(4) || end > bytes.len() {
            return Err(ProfileError::Invalid);
        }
        tags.append_slot()?.write(Tag {
            signature: field(record, 0)?,
            extent: Extent { start, end },
        });
    }

    sort_tags(&mut tags, |a, b| a.extent.cmp(&b.extent), account)?;
    let mut previous = None;
    let mut next_start = table_end;
    for tag in tags.iter() {
        account.work(RenderWork::IccProfileTagVisits, 1)?;
        if previous == Some(tag.extent) {
            continue; // ICC allows exact aliases of tag data.
        }
        if tag.extent.start < next_start || (contiguous && tag.extent.start != next_start) {
            return Err(ProfileError::Invalid);
        }
        next_start = tag.extent.end;
        if contiguous {
            next_start = tag
                .extent
                .end
                .checked_next_multiple_of(4)
                .ok_or(ProfileError::Invalid)?;
            let padding = bytes
                .get(tag.extent.end..next_start)
                .ok_or(ProfileError::Invalid)?;
            if padding.iter().any(|&byte| byte != 0) {
                return Err(ProfileError::Invalid);
            }
        }
        previous = Some(tag.extent);
    }
    if contiguous && next_start != bytes.len() {
        return Err(ProfileError::Invalid);
    }

    sort_tags(&mut tags, |a, b| a.signature.cmp(&b.signature), account)?;
    for pair in tags.windows(2) {
        account.work(RenderWork::IccProfileTagVisits, 2)?;
        if pair[0].signature == pair[1].signature {
            return Err(ProfileError::Invalid);
        }
    }
    Ok(tags)
}

/// In-place fallible heapsort meters reached comparisons before inspection.
fn sort_tags(
    tags: &mut [Tag],
    compare: impl Fn(&Tag, &Tag) -> Ordering,
    account: &Account,
) -> Result<()> {
    let sift = |tags: &mut [Tag], mut root: usize| -> Result<()> {
        let end = tags.len();
        while root < end / 2 {
            let left = root * 2 + 1;
            let right = left + 1;
            let child = if right < end {
                account.work(RenderWork::IccProfileTagVisits, 2)?;
                if compare(&tags[left], &tags[right]).is_lt() {
                    right
                } else {
                    left
                }
            } else {
                left
            };
            account.work(RenderWork::IccProfileTagVisits, 2)?;
            if compare(&tags[root], &tags[child]).is_ge() {
                break;
            }
            tags.swap(root, child);
            root = child;
        }
        Ok(())
    };

    let len = tags.len();
    for root in (0..len / 2).rev() {
        sift(tags, root)?;
    }
    for end in (1..len).rev() {
        tags.swap(0, end);
        sift(&mut tags[..end], 0)?;
    }
    Ok(())
}

fn find_tag(tags: &[Tag], signature: &[u8; 4], account: &Account) -> Result<Option<Extent>> {
    let mut search = tags;
    while let Some((tag, right)) = search.get(search.len() / 2..).and_then(<[_]>::split_first) {
        account.work(RenderWork::IccProfileTagVisits, 1)?;
        match tag.signature.cmp(signature) {
            Ordering::Equal => return Ok(Some(tag.extent)),
            Ordering::Greater => search = &search[..search.len() / 2],
            Ordering::Less => search = right,
        }
    }
    Ok(None)
}

fn required_tag(tags: &[Tag], signature: &[u8; 4], account: &Account) -> ProfileResult<Extent> {
    find_tag(tags, signature, account)?.ok_or(ProfileError::Invalid)
}

enum Curve {
    Identity,
    Gamma(f64),
    Sampled { data: Extent, intervals: u32 },
    Parametric(ParametricCurve),
}

impl Curve {
    fn read(tag: Extent, bytes: &[u8], version: u8, account: &Account) -> ProfileResult<Self> {
        let payload = tag.payload(bytes)?;
        match &tag_kind(payload, account)? {
            b"curv" => {
                let count = account.coordinate(|| u32_at(payload, 8))??;
                let size = usize::try_from(count)
                    .ok()
                    .and_then(|count| count.checked_mul(2))
                    .and_then(|size| size.checked_add(12))
                    .ok_or(ProfileError::Invalid)?;
                if payload.len() != size {
                    return Err(ProfileError::Invalid);
                }
                match count {
                    0 => Ok(Self::Identity),
                    1 => {
                        let gamma = account.coordinate(|| u16_at(payload, 12))??;
                        if gamma == 0 {
                            return Err(ProfileError::Invalid);
                        }
                        Ok(Self::Gamma(f64::from(gamma) / 256.0))
                    }
                    _ => Ok(Self::Sampled {
                        data: Extent {
                            start: tag.start + 12,
                            end: tag.end,
                        },
                        intervals: count - 1,
                    }),
                }
            }
            b"para" if version == 4 => {
                Ok(Self::Parametric(ParametricCurve::read(payload, account)?))
            }
            _ => Err(ProfileError::Invalid),
        }
    }

    fn evaluate(&self, sample: u8, bytes: &[u8], account: &Account) -> Result<f64> {
        match self {
            Self::Identity => account.coordinate(|| f64::from(sample) / 255.0),
            Self::Gamma(gamma) => account.coordinate(|| (f64::from(sample) / 255.0).powf(*gamma)),
            Self::Parametric(curve) => {
                account.coordinate(|| curve.evaluate(f64::from(sample) / 255.0))
            }
            Self::Sampled { data, intervals } => {
                // The integer quotient selects the lower sample exactly.
                // At the final endpoint no successor is needed.
                let position = account.coordinate(|| u64::from(sample) * u64::from(*intervals))?;
                let (index, remainder) = (position / 255, position % 255);
                let values = bytes
                    .get(data.start..data.end)
                    .ok_or_else(sample_invariant)?;
                let lower = sample_value(values, index, account)?;
                if remainder == 0 {
                    return Ok(lower);
                }
                let upper = sample_value(values, index + 1, account)?;
                account.coordinate(|| (upper - lower).mul_add(remainder as f64 / 255.0, lower))
            }
        }
    }
}

fn sample_value(bytes: &[u8], index: u64, account: &Account) -> Result<f64> {
    let offset = usize::try_from(index * 2).map_err(|_| sample_invariant())?;
    let value = account
        .coordinate(|| u16_at(bytes, offset))?
        .map_err(|_| sample_invariant())?;
    Ok(f64::from(value) / 65535.0)
}

fn sample_invariant() -> Error {
    Error::new(
        BackendErrorCode::RenderFailed,
        "validated ICC curve sample is outside its extent",
    )
}

/// Table 68 maps the five serialized function forms to two branches. The
/// upper branch always has a real power base throughout its selected domain.
struct ParametricCurve {
    threshold: f64,
    gamma: f64,
    scale: f64,
    offset: f64,
    upper_shift: f64,
    lower_scale: f64,
    lower_shift: f64,
}

impl ParametricCurve {
    fn read(payload: &[u8], account: &Account) -> ProfileResult<Self> {
        let kind = account.coordinate(|| u16_at(payload, 8))??;
        let params = match kind {
            0 => 1,
            1 => 3,
            2 => 4,
            3 => 5,
            4 => 7,
            _ => return Err(ProfileError::Invalid),
        };
        if payload.len() != 12 + params * 4 || u16_at(payload, 10)? != 0 {
            return Err(ProfileError::Invalid);
        }
        let mut values = [0.0; 7];
        for (index, value) in values[..params].iter_mut().enumerate() {
            *value = account.coordinate(|| fixed(payload, 12 + index * 4))??;
        }
        account.coordinate(|| Self::validated(kind, values))?
    }

    fn validated(kind: u16, [g, a, b, c, d, e, f]: [f64; 7]) -> ProfileResult<Self> {
        let (scale, offset, threshold, upper_shift, lower_scale, lower_shift) = match kind {
            0 => (1.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            1 if a != 0.0 => (a, b, -b / a, 0.0, 0.0, 0.0),
            2 if a != 0.0 => (a, b, -b / a, c, 0.0, c),
            3 => (a, b, d, 0.0, c, 0.0),
            4 => (a, b, d, e, c, f),
            _ => return Err(ProfileError::Invalid),
        };
        if threshold <= 1.0 {
            // The affine power base takes its minimum at one endpoint of
            // the active interval. Types 1 and 2 define that transition as
            // the exact root, even when the encoded ratio rounds as f64.
            let minimum = if scale < 0.0 {
                scale + offset
            } else if matches!(kind, 1 | 2) && threshold >= 0.0 {
                0.0
            } else {
                scale.mul_add(threshold.max(0.0), offset)
            };
            if minimum < 0.0 || (minimum == 0.0 && g <= 0.0) {
                return Err(ProfileError::Invalid);
            }
        }
        Ok(Self {
            threshold,
            gamma: g,
            scale,
            offset,
            upper_shift,
            lower_scale,
            lower_shift,
        })
    }

    fn evaluate(&self, input: f64) -> f64 {
        let output = if input < self.threshold {
            self.lower_scale.mul_add(input, self.lower_shift)
        } else {
            // A rounded root can produce a tiny negative residue in the
            // otherwise validated real domain. The exact root is zero.
            let base = self.scale.mul_add(input, self.offset).max(0.0);
            base.powf(self.gamma) + self.upper_shift
        };
        output.clamp(0.0, 1.0)
    }
}

fn tag_kind(payload: &[u8], account: &Account) -> ProfileResult<[u8; 4]> {
    account.work(RenderWork::IccProfileTagVisits, 1)?;
    if field::<4>(payload, 4)? != [0; 4] {
        return Err(ProfileError::Invalid);
    }
    field(payload, 0)
}

fn field<const N: usize>(bytes: &[u8], offset: usize) -> ProfileResult<[u8; N]> {
    bytes
        .get(offset..)
        .and_then(|suffix| suffix.first_chunk())
        .copied()
        .ok_or(ProfileError::Invalid)
}

fn u16_at(bytes: &[u8], offset: usize) -> ProfileResult<u16> {
    Ok(u16::from_be_bytes(field(bytes, offset)?))
}

fn u32_at(bytes: &[u8], offset: usize) -> ProfileResult<u32> {
    Ok(u32::from_be_bytes(field(bytes, offset)?))
}

fn fixed(bytes: &[u8], offset: usize) -> ProfileResult<f64> {
    Ok(f64::from(i32::from_be_bytes(field(bytes, offset)?)) / 65536.0)
}
