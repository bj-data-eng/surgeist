//! Structural qualification of the selected RGB-to-PCS lookup table.
//! ICC.1:2022 §§10.10–10.12 and ICC.1:2001-04 §§6.5.7–6.5.8.

use super::{
    Account, Extent, ProfileError, ProfileResult, RenderWork, field, tag_kind, u16_at, u32_at,
};

const CHANNELS: usize = 3;
const LEGACY_MATRIX: [u32; 9] = [0x10000, 0, 0, 0, 0x10000, 0, 0, 0, 0x10000];

pub(super) fn validate(payload: &[u8], version: u8, account: &Account) -> ProfileResult<()> {
    match &tag_kind(payload, account)? {
        b"mft1" => legacy_tables(payload, false, account),
        b"mft2" => legacy_tables(payload, true, account),
        b"mAB " if version == 4 => staged_tables(payload, account),
        _ => Err(ProfileError::Invalid),
    }
}

fn legacy_tables(payload: &[u8], wide: bool, account: &Account) -> ProfileResult<()> {
    let [input, output, grid, reserved] = account.coordinate(|| field::<4>(payload, 8))??;
    if usize::from(input) != CHANNELS
        || usize::from(output) != CHANNELS
        || grid < 2
        || reserved != 0
    {
        return Err(ProfileError::Invalid);
    }
    // The embedded source is RGB, so the legacy XYZ matrix must be identity.
    for (index, expected) in LEGACY_MATRIX.into_iter().enumerate() {
        if account.coordinate(|| u32_at(payload, 12 + index * 4))?? != expected {
            return Err(ProfileError::Invalid);
        }
    }
    let (start, input_entries, output_entries, precision) = if wide {
        let input_entries = account.coordinate(|| u16_at(payload, 48))??;
        let output_entries = account.coordinate(|| u16_at(payload, 50))??;
        if !(2..=4096).contains(&input_entries) || !(2..=4096).contains(&output_entries) {
            return Err(ProfileError::Invalid);
        }
        (
            52,
            usize::from(input_entries),
            usize::from(output_entries),
            2,
        )
    } else {
        (48, 256, 256, 1)
    };
    // An 8-bit grid cubed and both 4,096-entry tables fit a 32-bit size.
    let length =
        start + CHANNELS * precision * (input_entries + usize::from(grid).pow(3) + output_entries);
    if payload.len() != length {
        return Err(ProfileError::Invalid);
    }
    Ok(())
}

fn staged_tables(payload: &[u8], account: &Account) -> ProfileResult<()> {
    const HEADER: usize = 32;

    if account.coordinate(|| field::<4>(payload, 8))?? != [3, 3, 0, 0] {
        return Err(ProfileError::Invalid);
    }
    let mut offsets = [0; 5];
    for (index, offset) in offsets.iter_mut().enumerate() {
        let raw = account.coordinate(|| u32_at(payload, 12 + index * 4))??;
        *offset = usize::try_from(raw).map_err(|_| ProfileError::Invalid)?;
        if *offset != 0 && (!(HEADER..payload.len()).contains(offset) || !offset.is_multiple_of(4))
        {
            return Err(ProfileError::Invalid);
        }
    }
    let [b, matrix, m, clut, a] = offsets;
    if b == 0 || (matrix == 0) != (m == 0) || (clut == 0) != (a == 0) {
        return Err(ProfileError::Invalid);
    }

    // A, B and M may share complete embedded curves. Keep the individual
    // extents so exact curve aliases remain distinct from partial overlaps.
    let mut stages = [None; 3 * CHANNELS + 2];
    for (start, group) in [b, m, a]
        .into_iter()
        .zip(stages[..3 * CHANNELS].as_chunks_mut::<CHANNELS>().0)
    {
        if start != 0 {
            let mut next = start;
            for slot in group {
                let tail = payload.get(next..).ok_or(ProfileError::Invalid)?;
                let encoded_len = match &tag_kind(tail, account)? {
                    b"curv" => {
                        let count = account.coordinate(|| u32_at(tail, 8))??;
                        super::sampled_curve_size(count)?
                    }
                    b"para" => {
                        let (_, count) = super::ParametricCurve::kind_and_count(tail, account)?;
                        12 + count * 4
                    }
                    _ => return Err(ProfileError::Invalid),
                };
                let extent = element(payload, next, encoded_len)?;
                next = extent.end;
                *slot = Some(extent);
            }
        }
    }
    if matrix != 0 {
        stages[3 * CHANNELS] = Some(element(payload, matrix, 48)?);
    }
    if clut != 0 {
        let tail = payload.get(clut..).ok_or(ProfileError::Invalid)?;
        let grid = account.coordinate(|| field::<16>(tail, 0))??;
        let [precision, 0, 0, 0] = account.coordinate(|| field::<4>(tail, 16))?? else {
            return Err(ProfileError::Invalid);
        };
        let [r, g, b, unused @ ..] = grid;
        if r < 2 || g < 2 || b < 2 || unused != [0; 13] || !matches!(precision, 1 | 2) {
            return Err(ProfileError::Invalid);
        }
        let points = usize::from(r) * usize::from(g) * usize::from(b);
        let length = 20 + points * CHANNELS * usize::from(precision);
        stages[3 * CHANNELS + 1] = Some(element(payload, clut, length)?);
    }

    for (index, extent) in stages.iter().enumerate() {
        let Some(extent) = extent else { continue };
        for (other_index, other) in stages[..index].iter().enumerate() {
            let Some(other) = other else { continue };
            account.work(RenderWork::IccProfileTagVisits, 2)?;
            if extent.start < other.end
                && other.start < extent.end
                && !(index < 3 * CHANNELS && other_index < 3 * CHANNELS && extent == other)
            {
                return Err(ProfileError::Invalid);
            }
        }
    }
    Ok(())
}

fn element(payload: &[u8], start: usize, size: usize) -> ProfileResult<Extent> {
    let end = start.checked_add(size).ok_or(ProfileError::Invalid)?;
    let padded = end
        .checked_next_multiple_of(4)
        .ok_or(ProfileError::Invalid)?;
    // Verify alignment between stages here. Alignment bytes after the
    // declared tag size belong to the outer tag table.
    let padding = payload
        .get(end..padded.min(payload.len()))
        .ok_or(ProfileError::Invalid)?;
    if padding.iter().any(|&byte| byte != 0) {
        return Err(ProfileError::Invalid);
    }
    Ok(Extent { start, end: padded })
}
