use icu_segmenter::{
    iterators::{GraphemeClusterBreakIterator, SentenceBreakIterator, WordBreakIterator},
    scaffold::{Latin1, Utf16},
};
use surgeist_js_gc::heap::Trace as HeapTrace;

use crate::{
    Context, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject, iterable::create_iter_result_object_trusted},
    context::intrinsics::Intrinsics,
    js_string,
    object::ObjectEdge,
    property::Attribute,
    realm::Realm,
};

use super::{Segmenter, create_segment_data_object};

pub(crate) enum NativeSegmentIterator<'l, 's> {
    GraphemeUtf16(GraphemeClusterBreakIterator<'l, 's, Utf16>),
    WordUtf16(WordBreakIterator<'l, 's, Utf16>),
    SentenceUtf16(SentenceBreakIterator<'l, 's, Utf16>),
    GraphemeLatin1(GraphemeClusterBreakIterator<'l, 's, Latin1>),
    WordLatin1(WordBreakIterator<'l, 's, Latin1>),
    SentenceLatin1(SentenceBreakIterator<'l, 's, Latin1>),
}

impl Iterator for NativeSegmentIterator<'_, '_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NativeSegmentIterator::GraphemeUtf16(g) => g.next(),
            NativeSegmentIterator::WordUtf16(w) => w.next(),
            NativeSegmentIterator::SentenceUtf16(s) => s.next(),
            NativeSegmentIterator::GraphemeLatin1(g) => g.next(),
            NativeSegmentIterator::WordLatin1(w) => w.next(),
            NativeSegmentIterator::SentenceLatin1(s) => s.next(),
        }
    }
}

impl NativeSegmentIterator<'_, '_> {
    /// If the iterator is a word break iterator, returns `Some(true)` when the segment preceding
    /// the current boundary is word-like.
    pub(crate) fn is_word_like(&self) -> Option<bool> {
        match self {
            Self::WordLatin1(w) => Some(w.is_word_like()),
            Self::WordUtf16(w) => Some(w.is_word_like()),
            _ => None,
        }
    }
}

#[derive(Debug, JsData, HeapTrace)]
pub(crate) struct SegmentIterator {
    segmenter: ObjectEdge,
    string: JsString,
    next_segment_index: usize,
}

impl IntrinsicObject for SegmentIterator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)?
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Segmenter String Iterator"),
                Attribute::CONFIGURABLE,
            )?
            .static_method(Self::next, js_string!("next"), 0)?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> crate::JsResult<JsObject> {
        Ok(intrinsics.objects().iterator_prototypes().segment()?)
    }
}

impl SegmentIterator {
    /// [`CreateSegmentIterator ( segmenter, string )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-createsegmentiterator
    pub(crate) fn create(
        segmenter: JsObject,
        string: JsString,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        Ok(
            // 1. Let internalSlotsList be « [[IteratingSegmenter]], [[IteratedString]], [[IteratedStringNextSegmentCodeUnitIndex]] ».
            // 2. Let iterator be OrdinaryObjectCreate(%SegmentIteratorPrototype%, internalSlotsList).
            // 3. Set iterator.[[IteratingSegmenter]] to segmenter.
            // 4. Set iterator.[[IteratedString]] to string.
            // 5. Set iterator.[[IteratedStringNextSegmentCodeUnitIndex]] to 0.
            // 6. Return iterator.
            JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                context
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .segment()?,
                Self {
                    segmenter: segmenter.edge(),
                    string,
                    next_segment_index: 0,
                },
            )?
            .upcast(),
        )
    }
    /// [`%SegmentIteratorPrototype%.next ( )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentiteratorprototype%.next
    fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let iterator be the this value.
        // 2. Perform ? RequireInternalSlot(iterator, [[IteratingSegmenter]]).
        let object = this.as_object();
        let mut iter = object
            .as_ref()
            .map(JsObject::downcast_mut::<Self>)
            .transpose()?
            .flatten()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`next` can only be called on a `Segment Iterator` object")
            })?;

        // 5. Let startIndex be iterator.[[IteratedStringNextSegmentCodeUnitIndex]].
        let start = iter.next_segment_index;

        // 4. Let string be iterator.[[IteratedString]].
        // 6. Let endIndex be ! FindBoundary(segmenter, string, startIndex, after).
        let boundary = iter
            .string
            .get(start..)
            .map(|string| -> JsResult<_> {
                // 3. Let segmenter be iterator.[[IteratingSegmenter]].
                let segmenter = JsObject::from_inner(iter.segmenter.clone());
                let segmenter = segmenter
                    .downcast_ref::<Segmenter>()?
                    .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;
                let mut segments = segmenter.native.segment(string.variant());
                // the first elem is always 0.
                segments.next();
                Ok(segments
                    .next()
                    .map(|end| (start + end, segments.is_word_like())))
            })
            .transpose()?
            .flatten();
        let Some((end, is_word_like)) = boundary else {
            drop(iter);
            // 7. If endIndex is not finite, then
            //     a. Return CreateIterResultObject(undefined, true).
            return create_iter_result_object_trusted(JsValue::undefined(), true, context);
        };
        // 8. Set iterator.[[IteratedStringNextSegmentCodeUnitIndex]] to endIndex.
        iter.next_segment_index = end;
        let string = iter.string.clone();
        drop(iter);

        // 9. Let segmentData be ! CreateSegmentDataObject(segmenter, string, startIndex, endIndex).
        let segment_data = create_segment_data_object(string, start..end, is_word_like, context)?;

        // 10. Return CreateIterResultObject(segmentData, false).
        create_iter_result_object_trusted(segment_data.into(), false, context)
    }
}
