//! Numerical construction and output using only surgeist-color.
use surgeist_color::{
    Color, ColorSpace, DigitSlot, Finite, InterpolationMethod, OperationOwner,
    PredefinedColorSpace, Serialization, SerializationAction, SerializationRequest,
    SerializationStep, Transformation, UnitInterval,
};

/// Serializes `value` by servicing every action of the controlled
/// continuation; the caller owns the digit bank and the output.
fn serialize(value: Color, request: SerializationRequest) -> String {
    let mut owner = OperationOwner::new();
    let mut serialization = Serialization::new(&mut owner, value, request);
    let mut digits: Vec<DigitSlot<'_>> = Vec::new();
    let mut output = Vec::new();
    loop {
        let action = match serialization.step() {
            SerializationStep::Complete(_) => break,
            SerializationStep::Pending(action) => action,
        };
        serialization = match action {
            SerializationAction::Compute(compute) => compute.compute(),
            SerializationAction::ReadDigit(read) => {
                let slot = &digits[read.index()];
                read.read(slot)
            }
            SerializationAction::AppendDigit(write) => {
                let (record, receipt) = write.record();
                digits.push(record);
                receipt.stored(&digits[digits.len() - 1])
            }
            SerializationAction::ReplaceDigit(write) => {
                let index = write.index();
                let (record, receipt) = write.record();
                digits[index] = record;
                receipt.stored(&digits[index])
            }
            SerializationAction::Emit(emit) => {
                output.push(emit.unit());
                emit.emitted()
            }
        };
    }
    output.into_iter().map(char::from).collect()
}

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
    println!(
        "{}",
        serialize(quantized, SerializationRequest::HtmlCompatible)
    );
    assert_eq!(
        serialize(red, SerializationRequest::HtmlCompatible),
        "#ff0000"
    );
    Ok(())
}
