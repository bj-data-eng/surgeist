//! The phrase of an `ErrorCode` and the text of the `Error` at real raise sites.
//!
//! Each test obtains the error from a documented public operation. The expected message is written
//! from the operation's documented rule, and the phrase is the one that `ErrorCode` prints for the
//! code: it must read true for what the operation did. `Error` itself prints only its message.

use surgeist_render::{
    ErrorCode, geometry::CoordinateSpaceKind, geometry::CoordinateSpaceTag, geometry::Rect,
    geometry::Size, geometry::Transform, image::Image, scene::BlendMode, scene::LayerComposition,
    scene::PorterDuffOperator, style::image::BackgroundAttachment,
    style::image::ImageAttachmentPlan,
};

#[test]
fn image_data_that_does_not_match_its_size_is_refused_before_any_upload() {
    let size = Size::try_new(1.0, 1.0).unwrap();
    let error = Image::from_rgba(size, vec![0_u8; 3]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ImageUploadFailed);
    assert_eq!(
        error.to_string(),
        "RGBA image data length 3 does not match 1x1 image size; expected 4 bytes"
    );
    // The operation only validates; nothing is uploaded, so the phrase says the data was refused.
    assert_eq!(error.code().to_string(), "image data refused");
}

#[test]
fn an_image_size_that_is_not_a_pixel_count_names_the_value_in_the_number_form() {
    for (width, text) in [(1.5, "1.5"), (1e300, "1e300")] {
        let size = Size::try_new(width, 1.0).unwrap();
        let error = Image::from_rgba(size, Vec::<u8>::new()).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ImageUploadFailed);
        assert_eq!(
            error.to_string(),
            format!("image width {text} must be a finite non-negative integer pixel size")
        );
    }
}

#[test]
fn a_negative_rectangle_width_is_an_invalid_value() {
    let error = Rect::try_new(0.0, 0.0, -1.0, 4.0).unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    assert_eq!(
        error.to_string(),
        "rectangle width value -1 is invalid: must be finite and non-negative"
    );
    assert_eq!(error.code().to_string(), "invalid value");
}

#[test]
fn plus_lighter_with_a_mixing_function_names_the_function() {
    // PlusLighter is a complete operation, so any other mixing function is refused by name.
    for (blend, name) in [
        (BlendMode::Multiply, "multiply"),
        (BlendMode::ColorDodge, "color dodge"),
        (BlendMode::SoftLight, "soft light"),
        (BlendMode::Luminosity, "luminosity"),
    ] {
        let error =
            LayerComposition::try_porter_duff(PorterDuffOperator::PlusLighter, blend).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert_eq!(
            error.to_string(),
            format!(
                "PlusLighter mixing function value {name} is invalid: must be Normal; PlusLighter is a complete operation"
            )
        );
    }
    let error = LayerComposition::try_porter_duff(PorterDuffOperator::SourceOver, BlendMode::Plus)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Porter-Duff mixing function value Plus is invalid: must be a color mixing function; Plus is a complete operation"
    );
}

#[test]
fn a_fixed_background_needs_a_viewport_tag_and_names_the_kind_it_got() {
    let identity = Transform::identity();
    let missing = ImageAttachmentPlan::try_new(BackgroundAttachment::Fixed, None).unwrap_err();
    assert_eq!(
        missing.to_string(),
        "background attachment coordinate space value none is invalid: fixed backgrounds require a viewport coordinate tag"
    );
    for (kind, name) in [
        (CoordinateSpaceKind::Local, "local"),
        (CoordinateSpaceKind::Surface, "surface"),
    ] {
        let tag = CoordinateSpaceTag::try_new(kind, identity).unwrap();
        let error =
            ImageAttachmentPlan::try_new(BackgroundAttachment::Fixed, Some(tag)).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert_eq!(
            error.to_string(),
            format!(
                "background attachment coordinate space value {name} is invalid: fixed backgrounds require a viewport coordinate tag"
            )
        );
    }
}
