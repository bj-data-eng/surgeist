//! The text of a numerical colour, through Color's controlled serialization.

use surgeist_color::{
    Color, DigitSlot, OperationOwner, Serialization, SerializationAction, SerializationRequest,
    SerializationStep,
};

/// Serializes `value` by servicing every action of the continuation, with
/// unbounded caller storage.
pub fn color_text(value: Color, request: SerializationRequest) -> String {
    let mut owner = OperationOwner::new();
    let mut serialization = Serialization::new(&mut owner, value, request);
    let mut digits: Vec<DigitSlot<'_>> = Vec::new();
    let mut output = Vec::new();
    loop {
        let action = match serialization.step() {
            SerializationStep::Complete(receipt) => {
                assert_eq!(receipt.length(), output.len());
                break;
            }
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
    String::from_utf8(output).expect("color serialization emits ASCII")
}
