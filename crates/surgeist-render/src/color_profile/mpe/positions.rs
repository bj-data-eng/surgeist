//! Shared ICC MPE and curve-set offset/size directory validation.

use crate::color_profile::{
    Account, AccountedVec, Extent, ProfileError, ProfileResult, RenderWork, sort_tags, u32_at,
};

pub(super) struct Positions {
    ordered: AccountedVec<Located>,
    unique: usize,
}

pub(super) struct Located {
    pub(super) extent: Extent,
    pub(super) index: usize,
}

impl Positions {
    pub(super) fn read(
        bytes: &[u8],
        table_start: usize,
        count: usize,
        minimum: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        if count == 0 {
            return Err(ProfileError::Invalid);
        }
        let data_start = count
            .checked_mul(8)
            .and_then(|size| table_start.checked_add(size))
            .ok_or(ProfileError::Invalid)?;
        let records = bytes
            .get(table_start..data_start)
            .ok_or(ProfileError::Invalid)?;
        let mut ordered = account.temporary::<Located>(count)?;
        for (index, record) in records.as_chunks::<8>().0.iter().enumerate() {
            account.work(RenderWork::IccProfileTagVisits, 1)?;
            let start = usize::try_from(u32_at(record, 0)?).map_err(|_| ProfileError::Invalid)?;
            let size = usize::try_from(u32_at(record, 4)?).map_err(|_| ProfileError::Invalid)?;
            let end = start.checked_add(size).ok_or(ProfileError::Invalid)?;
            if start < data_start || !start.is_multiple_of(4) || size < minimum || end > bytes.len()
            {
                return Err(ProfileError::Invalid);
            }
            ordered.append_slot()?.write(Located {
                extent: Extent { start, end },
                index,
            });
        }

        sort_tags(&mut ordered, |a, b| a.extent.cmp(&b.extent), account)?;
        let mut previous = None;
        let mut next = data_start;
        let mut unique = 0;
        for record in ordered.iter() {
            account.work(RenderWork::IccProfileTagVisits, 1)?;
            let extent = record.extent;
            if Some(extent) == previous {
                continue;
            }
            if extent.start != next {
                return Err(ProfileError::Invalid);
            }
            next = extent
                .end
                .checked_next_multiple_of(4)
                .ok_or(ProfileError::Invalid)?;
            if bytes
                .get(extent.end..next.min(bytes.len()))
                .ok_or(ProfileError::Invalid)?
                .iter()
                .any(|&byte| byte != 0)
            {
                return Err(ProfileError::Invalid);
            }
            unique += 1;
            previous = Some(extent);
        }
        // The final alignment bytes may belong to the surrounding tag.
        if next.min(bytes.len()) != bytes.len() {
            return Err(ProfileError::Invalid);
        }
        Ok(Self { ordered, unique })
    }

    pub(super) fn physical(&self) -> &[Located] {
        &self.ordered
    }

    pub(super) const fn unique_count(&self) -> usize {
        self.unique
    }
}
