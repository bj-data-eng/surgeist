//! Literal oracles from HTML877d #images-processing-model,
//! #reacting-to-dom-mutations and Img/Source HTML element tree hooks.
use crate::html::support::known_text_attribute as raw;
use std::cmp::Ordering;
use surgeist_html::*;

#[test]
fn fresh_image_retains_the_unavailable_request_and_distinct_null_slots() {
    let state = HtmlImgFreshState::fresh();
    let snapshot = state.clone();
    assert_eq!(
        state.current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(state.current_request().current_url().is_empty());
    assert_eq!(state.current_request().current_pixel_density(), 1);
    assert!(state.current_request().preferred_dimensions_are_null());
    assert!(!state.current_request().image_data_is_available());
    assert!(state.pending_request_is_null());
    assert!(state.last_selected_source_is_null());
    assert!(state.last_auto_sizes_width_is_null());
    assert!(state.lazy_load_resumption_is_null());
    assert!(state.dimension_attribute_source_is_self());
    assert_eq!(snapshot, state);
}

#[test]
fn inactive_update_waits_before_any_image_support_or_resource_step() {
    assert_eq!(
        image_update_start(HtmlImageDocumentActivity::NotFullyActive),
        HtmlImageUpdateStartDecision::WaitForFullyActive
    );
    assert_eq!(
        image_update_start(HtmlImageDocumentActivity::FullyActive),
        HtmlImageUpdateStartDecision::CheckImageSupport
    );
    assert_eq!(
        image_update_start(HtmlImageDocumentActivity::Unavailable),
        HtmlImageUpdateStartDecision::NeedsFullyActiveFact
    );
    // A suspension cannot mutate the request or create a pending image request.
    let initial = HtmlImgFreshState::fresh();
    assert!(initial.current_request().current_url().is_empty());
    assert!(initial.pending_request_is_null());
}

#[test]
fn later_started_update_still_supersedes_after_the_later_wait_aborts() {
    // Independent invocation-order fixture, not DOM scheduling or an abort API.
    let old_invocation = 41_u64;
    let latest_started = 42_u64;
    let mut latest_wait = Some(latest_started);
    assert_eq!(
        image_update_is_superseded(old_invocation.cmp(&latest_started)),
        Ok(true)
    );
    assert_eq!(latest_wait.take(), Some(42)); // later invocation aborts
    assert_eq!(latest_wait, None);
    assert_eq!(
        image_update_is_superseded(old_invocation.cmp(&latest_started)),
        Ok(true)
    );
    assert_eq!(
        image_update_is_superseded(latest_started.cmp(&latest_started)),
        Ok(false)
    );
    assert_eq!(
        image_update_is_superseded(Ordering::Greater),
        Err(HtmlImageUpdateOrderError)
    );
}

#[test]
fn invocation_flags_preserve_both_values_without_merging() {
    for (restart, omit) in [(false, false), (true, false), (false, true), (true, true)] {
        let flags = HtmlImageUpdateFlags::new(restart, omit);
        assert_eq!(flags.restart_animation(), restart);
        assert_eq!(flags.maybe_omit_events(), omit);
    }
    assert_eq!(
        HtmlImageUpdateFlags::default(),
        HtmlImageUpdateFlags::new(false, false)
    );
}

#[test]
fn only_an_equal_value_src_set_carries_the_explicit_animation_restart() {
    use HtmlImageAttributeChange as C;
    use HtmlImageAttributeEffect as E;
    use KnownHtmlAttribute as A;
    let ns = Namespace::none();
    assert_eq!(
        image_attribute_effect(&ns, A::Src, C::Set { same_value: true }),
        E::Update(HtmlImageUpdateFlags::new(true, false))
    );
    for change in [C::Set { same_value: false }, C::Removed] {
        assert_eq!(
            image_attribute_effect(&ns, A::Src, change),
            E::Update(HtmlImageUpdateFlags::new(false, false))
        );
    }
    for attr in [A::Srcset, A::Sizes] {
        for change in [C::Set { same_value: true }, C::Removed] {
            assert_eq!(
                image_attribute_effect(&ns, attr, change),
                E::Update(HtmlImageUpdateFlags::new(false, false))
            );
        }
    }
    assert_eq!(
        image_attribute_effect(
            &Namespace::from_uri("urn:test"),
            A::Src,
            C::Set { same_value: true }
        ),
        E::None
    );
    assert_eq!(image_attribute_effect(&ns, A::Alt, C::Removed), E::None);
    assert_eq!(
        image_attribute_effect(&ns, A::Crossorigin, C::Removed),
        E::CompareCrossOrigin
    );
    assert_eq!(
        image_attribute_effect(&ns, A::Referrerpolicy, C::Removed),
        E::CompareReferrerPolicy
    );
    assert_eq!(
        image_attribute_effect(&ns, A::Loading, C::Removed),
        E::CompareLoading
    );
}

#[test]
fn semantic_attribute_aliases_and_invalid_defaults_do_not_restart_images() {
    let interpreter = HtmlValueInterpreter::new(64);
    let img = KnownHtmlElement::Img.qualified_name();
    let cors = |value: Option<&str>| {
        let attr = value.map(|value| raw(KnownHtmlAttribute::Crossorigin, value));
        *interpreter
            .crossorigin(&img, attr.as_ref())
            .unwrap()
            .intrinsic()
            .unwrap()
    };
    assert!(!image_crossorigin_changed(
        cors(Some("ANONYMOUS")),
        cors(Some("invalid"))
    ));
    assert!(image_crossorigin_changed(cors(None), cors(Some(""))));
    assert!(image_crossorigin_changed(
        cors(Some("anonymous")),
        cors(Some("use-credentials"))
    ));
    let policy = |value: Option<&str>| {
        let attr = value.map(|value| raw(KnownHtmlAttribute::Referrerpolicy, value));
        *interpreter
            .referrerpolicy(&img, attr.as_ref())
            .unwrap()
            .intrinsic()
            .unwrap()
    };
    assert!(!image_referrerpolicy_changed(
        policy(None),
        policy(Some("invalid"))
    ));
    assert!(!image_referrerpolicy_changed(
        policy(Some("ORIGIN")),
        policy(Some("origin"))
    ));
    assert!(image_referrerpolicy_changed(
        policy(Some("origin")),
        policy(Some("same-origin"))
    ));
    assert!(!image_loading_became_eager(
        HtmlLoading::Eager,
        HtmlLoading::Eager
    ));
    assert!(!image_loading_became_eager(
        HtmlLoading::Eager,
        HtmlLoading::Lazy
    ));
    assert!(image_loading_became_eager(
        HtmlLoading::Lazy,
        HtmlLoading::Eager
    ));
}

#[test]
fn source_selection_attributes_affect_following_images_including_dimensions() {
    use KnownHtmlAttribute as A;
    for attr in [A::Srcset, A::Sizes, A::Media, A::Type, A::Width, A::Height] {
        assert!(image_source_attribute_affects_following(
            &Namespace::none(),
            attr
        ));
        assert!(!image_source_attribute_affects_following(
            &Namespace::from_uri("urn:test"),
            attr
        ));
    }
    for attr in [A::Src, A::Alt, A::Crossorigin, A::Loading] {
        assert!(!image_source_attribute_affects_following(
            &Namespace::none(),
            attr
        ));
    }
}

#[test]
fn picture_insertion_visits_all_images_but_removal_uses_the_actual_old_parent() {
    use HtmlImageMutationTargets as T;
    use HtmlImageTreeChange as C;
    use KnownHtmlElement as K;
    assert_eq!(
        image_tree_effect(K::Source, C::Inserted, Some(K::Picture), None, false)
            .unwrap()
            .targets(),
        T::ParentPictureImages
    );
    assert_eq!(
        image_tree_effect(K::Img, C::Inserted, Some(K::Picture), None, false)
            .unwrap()
            .targets(),
        T::ThisImage
    );
    for change in [C::Removed, C::Moved] {
        assert_eq!(
            image_tree_effect(K::Source, change, None, Some(K::Picture), true)
                .unwrap()
                .targets(),
            T::OldParentPictureImages
        );
        assert_eq!(
            image_tree_effect(K::Img, change, None, Some(K::Picture), true)
                .unwrap()
                .targets(),
            T::ThisImage
        );
        assert_eq!(
            image_tree_effect(K::Source, change, None, Some(K::Picture), false)
                .unwrap()
                .targets(),
            T::None
        );
        assert_eq!(
            image_tree_effect(K::Img, change, Some(K::Picture), Some(K::Div), true)
                .unwrap()
                .targets(),
            T::None
        );
    }
    assert_eq!(
        image_tree_effect(K::Picture, C::Inserted, None, None, true),
        Err(HtmlImageTreeInapplicable)
    );
}

#[test]
fn every_removed_image_clears_auto_width_and_source_media_checks_do_not_select_resources() {
    use HtmlImageTreeChange as C;
    use KnownHtmlElement as K;
    let nested = image_tree_effect(K::Img, C::Removed, None, Some(K::Picture), false).unwrap();
    assert!(nested.clear_auto_sizes_width());
    assert_eq!(nested.targets(), HtmlImageMutationTargets::None);
    for change in [C::Inserted, C::Moved] {
        let effect = image_tree_effect(K::Source, change, Some(K::Audio), None, true).unwrap();
        assert!(effect.check_media_parent_selection());
        assert!(!effect.clear_auto_sizes_width());
        assert_eq!(effect.targets(), HtmlImageMutationTargets::None);
    }
    assert!(
        !image_tree_effect(K::Source, C::Removed, Some(K::Video), None, true)
            .unwrap()
            .check_media_parent_selection()
    );
}
