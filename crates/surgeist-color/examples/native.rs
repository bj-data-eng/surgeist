//! Numerical construction and output using only surgeist-color.
use surgeist_color::{
    Color, ColorSpace, InterpolationMethod, PredefinedColorSpace, SerializationLimits,
    SerializationRequest, Transformation, serialize,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let srgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
    let extended = Color::try_new(
        ColorSpace::Predefined(PredefinedColorSpace::DisplayP3),
        [Some(1.1), Some(-0.0), Some(0.2)],
        Some(0.5),
    )?;
    let converted = extended.converted(srgb)?;
    let red = Color::from_srgba8([255, 0, 0, 255]);
    let mixed = converted.interpolated(red, InterpolationMethod::try_new(srgb, None)?, 0.25)?;
    let opaque = mixed.transformed(Transformation::ForceOpaqueAlpha)?;
    let quantized = opaque.transformed(Transformation::QuantizeSrgb)?;
    let limits = SerializationLimits {
        actions: 100_000,
        digits: 2048,
        output_bytes: 4096,
    };
    println!(
        "{}",
        serialize(quantized, SerializationRequest::HtmlCompatible, limits)?
    );
    assert_eq!(
        serialize(red, SerializationRequest::HtmlCompatible, limits)?,
        "#ff0000"
    );
    Ok(())
}
