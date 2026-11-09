//! Locked native serialization is the sole non-test descendant allowed to
//! borrow the intact vector owner's raw storage during the dependency call.

use super::{Account, AccountedVec, RenderResource, StorageSizing};
use crate::Result;
use crate::accounting::RenderWork;
use vello_encoding::{Encoding, StreamOffsets};

/// Locked Vello packed format: tags pad to 1024 bytes, then five word-aligned
/// streams. Late glyph sizes include the exact preflight PathEncoder streams,
/// plus one transform per occurrence and one paint transform per run.
pub(crate) fn packed_size(
    encoding: &Encoding,
    glyph_sizes: &StreamOffsets,
    account: &Account,
) -> Result<usize> {
    let sizing = StorageSizing::new(account, RenderResource::RetainedCpuBytes);
    let open = encoding.n_open_clips as usize;
    let tags = sizing.add(
        sizing.add(encoding.path_tags.len(), glyph_sizes.path_tags)?,
        open,
    )?;
    let mut size = sizing.mul(sizing.add(tags, 1023)? / 1024, 1024)?;
    for words in [
        sizing.add(encoding.path_data.len(), glyph_sizes.path_data)?,
        sizing.add(
            sizing.add(encoding.draw_tags.len(), glyph_sizes.draw_tags)?,
            open,
        )?,
        sizing.add(encoding.draw_data.len(), glyph_sizes.draw_data)?,
    ] {
        size = sizing.add(size, sizing.mul(words, 4)?)?;
    }
    size = sizing.add(
        size,
        sizing.mul(
            sizing.add(encoding.transforms.len(), glyph_sizes.transforms)?,
            std::mem::size_of::<vello_encoding::Transform>(),
        )?,
    )?;
    sizing.add(
        size,
        sizing.mul(
            sizing.add(encoding.styles.len(), glyph_sizes.styles)?,
            std::mem::size_of::<vello_encoding::Style>(),
        )?,
    )
}

/// Locked serialization owns one intact admitted vector, bound to its actual
/// immutable encoding and the size calculated from that encoding's streams.
pub(crate) struct PackedProducer<'a> {
    encoding: &'a Encoding,
    expected_size: usize,
    values: AccountedVec<u8>,
    account: &'a Account,
}

/// The dependency's upload type is unnameable outside its private module. This
/// view borrows its actual upload slice without another backing or upload copy.
pub(crate) struct ResolvedImages<'a> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) images: &'a [(peniko::ImageData, u32, u32)],
}

pub(crate) struct PackedResolution<'a> {
    packed: AccountedVec<u8>,
    layout: vello_encoding::Layout,
    ramps: vello_encoding::Ramps<'a>,
    images: ResolvedImages<'a>,
}

impl<'a> PackedResolution<'a> {
    pub(crate) fn into_parts(
        self,
    ) -> (
        AccountedVec<u8>,
        vello_encoding::Layout,
        vello_encoding::Ramps<'a>,
        ResolvedImages<'a>,
    ) {
        (self.packed, self.layout, self.ramps, self.images)
    }

    #[cfg(test)]
    pub(crate) fn packed_for_test(&self) -> &AccountedVec<u8> {
        &self.packed
    }
}

impl<'a> PackedProducer<'a> {
    pub(crate) fn new(
        encoding: &'a Encoding,
        glyph_sizes: &StreamOffsets,
        account: &'a Account,
    ) -> Result<Self> {
        let expected_size = packed_size(encoding, glyph_sizes, account)?;
        Ok(Self {
            encoding,
            expected_size,
            values: account.retained(expected_size.max(4))?,
            account,
        })
    }

    pub(crate) fn resolve<'r>(
        mut self,
        resolver: &'r mut vello_encoding::Resolver,
    ) -> Result<PackedResolution<'r>> {
        let (layout, ramps, images) = self.values.with_producer(self.expected_size, |values| {
            self.account
                .work(RenderWork::SerializedBytes, self.expected_size)?;
            let (layout, ramps, images) = resolver.resolve(self.encoding, values);
            Ok((
                layout,
                ramps,
                ResolvedImages {
                    width: images.width,
                    height: images.height,
                    images: images.images,
                },
            ))
        })?;
        Ok(PackedResolution {
            packed: self.values,
            layout,
            ramps,
            images,
        })
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.values.capacity()
    }
}
