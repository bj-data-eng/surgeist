//! Numerical construction and output using only surgeist-color.
use surgeist_color::{
    Color, ColorSpace, Finite, InterpolationMethod, PredefinedColorSpace, SerializationLimits,
    SerializationRequest, Transformation, UnitInterval, serialize,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let srgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
    let extended = Color::new(
        ColorSpace::Predefined(PredefinedColorSpace::DisplayP3),
        [Finite::new(1.1), Finite::new(-0.0), Finite::new(0.2)],
        UnitInterval::new(0.5),
    );
    let converted = extended.converted(srgb);
    let red = Color::from_srgba8([255, 0, 0, 255]);
    let quarter = UnitInterval::new(0.25).ok_or("a quarter lies in zero to one")?;
    let mixed = converted.interpolated(red, InterpolationMethod::try_new(srgb, None)?, quarter);
    let opaque = mixed.transformed(Transformation::ForceOpaqueAlpha);
    let quantized = opaque.transformed(Transformation::QuantizeSrgb);
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
