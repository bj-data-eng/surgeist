//! Standalone `img` initialization and pre-resource mutation decisions.

pub use crate::internal::elements::image::{
    HtmlImageAttributeChange, HtmlImageAttributeEffect, HtmlImageDocumentActivity,
    HtmlImageMutationTargets, HtmlImageTreeChange, HtmlImageTreeEffect, HtmlImageTreeInapplicable,
    HtmlImageUpdateFlags, HtmlImageUpdateOrderError, HtmlImageUpdateStartDecision,
    HtmlImgFreshRequestState, HtmlImgFreshState, HtmlImgUnavailableRequest, image_attribute_effect,
    image_crossorigin_changed, image_loading_became_eager, image_referrerpolicy_changed,
    image_source_attribute_affects_following, image_tree_effect, image_update_is_superseded,
    image_update_start,
};
