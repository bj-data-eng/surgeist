use surgeist_html::*;

#[test]
fn file_and_image_modes_preserve_distinct_resource_and_content_readback() {
    let raw = [0x0d, 0x0a, 0xd800, 88];
    let file = HtmlInputType::File;
    let image = HtmlInputType::Image;
    assert_eq!(file.value_mode(), HtmlInputValueMode::Filename);
    assert_eq!(image.value_mode(), HtmlInputValueMode::Default);
    for kind in [file, image] {
        assert_eq!(
            kind.sanitization(),
            HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
        );
        let HtmlInputValueSanitization::String(sanitization) = kind.sanitization() else {
            panic!("File and Image use the string sanitizer");
        };
        assert!(!kind.is_selectable());
        assert!(raw.into_iter().all(|u| sanitization.retain_unit(u)));
    }
    assert_eq!(file.value_mode().readback(&raw, Some(&raw), None), None);
    assert_eq!(
        file.value_mode()
            .readback(&raw, Some(&raw), Some(HtmlInputFileSelection::Empty)),
        Some(&[][..])
    );
    assert_eq!(
        image.value_mode().readback(&[], Some(&raw), None),
        Some(&raw[..])
    );
    assert_eq!(
        image.value_mode().readback(&raw, Some(&[]), None),
        Some(&[][..])
    );
    assert_eq!(image.value_mode().readback(&raw, None, None), Some(&[][..]));
}

#[test]
fn filename_setter_rejects_nonempty_before_resource_mutation_and_empty_clears() {
    assert_eq!(
        input_filename_setter(&[]),
        Ok(HtmlInputFileSelection::Empty)
    );
    for units in [&[88][..], &[0][..], &[0xd800][..], &[32][..]] {
        assert_eq!(
            input_filename_setter(units),
            Err(HtmlInputFilenameInvalidState)
        );
    }
    assert_eq!(
        HtmlInputFlags::from_retained(HtmlInputType::File, false, false, false, false, false)
            .value_setter_begin(),
        HtmlInputValueSetterStart::SetFilename
    );
}

fn effects(previous: HtmlInputType, next: HtmlInputType) -> Vec<HtmlInputTypeEffect> {
    let mut cursor = HtmlInputTypeChange::new(previous, next, 3);
    std::iter::from_fn(|| cursor.next_effect()).collect()
}
#[test]
fn filename_transition_never_writes_internal_value_to_content_and_return_installs_default() {
    use HtmlInputTypeEffect::*;
    assert_eq!(
        effects(HtmlInputType::Text, HtmlInputType::File),
        vec![
            SetCurrentEmptyForFilename,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(HtmlInputType::Image, HtmlInputType::File),
        vec![
            SetCurrentEmptyForFilename,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(HtmlInputType::File, HtmlInputType::Image),
        vec![
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::Preserve
            ))
        ]
    );
    assert_eq!(
        effects(HtmlInputType::File, HtmlInputType::Text),
        vec![
            SetCurrentFromDefaultAndClearDirty,
            SetRendering,
            SignalTypeChange,
            Sanitize(HtmlInputValueSanitization::String(
                HtmlInputSanitization::StripCrLf
            )),
            SetInitialCursorAndNoneDirection
        ]
    );
    assert!(effects(HtmlInputType::File, HtmlInputType::File).is_empty());
}

#[test]
fn file_image_reset_and_clone_flags_have_source_defined_independent_roles() {
    for kind in [HtmlInputType::File, HtmlInputType::Image] {
        let original = HtmlInputFlags::from_retained(kind, true, true, true, true, true);
        let reset = original.reset_begin().reset_checkedness(false);
        assert_eq!(reset.input_type(), kind);
        assert!(!reset.dirty_value());
        assert!(!reset.dirty_checkedness());
        assert!(!reset.user_validity());
        assert!(!reset.checkedness());
        assert!(reset.indeterminate());
        let clone = HtmlInputFlags::fresh()
            .type_changed(kind)
            .clone_flags_from(original);
        assert_eq!(clone.input_type(), kind);
        assert!(clone.dirty_value());
        assert!(clone.dirty_checkedness());
        assert!(clone.checkedness());
        assert!(clone.indeterminate());
        assert!(!clone.user_validity());
        let mut cursor = HtmlInputReset::new(original);
        assert!(matches!(
            cursor.next_effect(),
            Some(HtmlInputResetEffect::ClearDirtyAndUserValidity(_))
        ));
        assert_eq!(
            cursor.next_effect(),
            Some(HtmlInputResetEffect::ReplaceCurrentFromDefault)
        );
        assert_eq!(
            cursor.next_effect(),
            Some(HtmlInputResetEffect::SetCheckedFromPresence)
        );
        assert_eq!(
            cursor.next_effect(),
            Some(HtmlInputResetEffect::EmptySelectedFiles)
        );
        assert_eq!(
            cursor.next_effect(),
            Some(HtmlInputResetEffect::Sanitize(
                HtmlInputValueSanitization::String(HtmlInputSanitization::Preserve)
            ))
        );
        assert_eq!(cursor.next_effect(), None);
    }
}
