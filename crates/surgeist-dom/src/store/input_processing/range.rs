//! Storage/admission adapter for HTML's public finite Range protocols.

use super::{DomError, InputProcessing, RangeInput, extent, sum};
use crate::DocumentString;
use std::{
    mem::{size_of, size_of_val},
    sync::atomic::AtomicBool,
};
use surgeist_html::bounded::{
    BestFiniteNumberCursor, FiniteDecimalIngressCursor, FloatingPointGrammarCursor,
    FloatingPointParseMode, HtmlInputRangeNumber, HtmlInputRangeSanitizationAction as Action,
    HtmlInputRangeSanitizationCursor, HtmlInputRangeSanitizationReply as Reply,
    HtmlInputRangeValue, RangeBufferHeader, RangeLatticeCursor, RangeMidpointCursor,
    RangeNumericError, RangeNumericHeaders, RangeNumericLayout, RangeNumericReply,
    RangeNumericRequest,
};
use surgeist_html::{HtmlFloatingPointNumber, HtmlInputNumericContent, HtmlInputNumericStep};

struct Storage {
    banks: [Vec<u8>; 8],
    output: Vec<u16>,
}

impl Storage {
    fn new() -> Self {
        Self {
            banks: std::array::from_fn(|_| Vec::new()),
            output: Vec::new(),
        }
    }

    fn headers(&self) -> RangeNumericHeaders {
        RangeNumericHeaders::new(
            std::array::from_fn(|i| {
                RangeBufferHeader::new(self.banks[i].len(), self.banks[i].capacity())
                    .expect("actual byte Vec header")
            }),
            RangeBufferHeader::new(self.output.len(), self.output.capacity())
                .expect("actual UTF16 Vec header"),
        )
    }

    fn clear(&mut self, meter: &mut InputProcessing, cancel: &AtomicBool) -> Result<(), DomError> {
        for bank in &mut self.banks {
            meter.charge(sum(&[1, size_of::<usize>()])?, cancel)?;
            bank.clear();
        }
        Ok(())
    }

    fn service(
        &mut self,
        meter: &mut InputProcessing,
        source: &[u16],
        request: RangeNumericRequest,
        layout: RangeNumericLayout,
        cancel: &AtomicBool,
    ) -> Result<RangeNumericReply, DomError> {
        meter.charge(
            sum(&[
                1,
                layout.register_bytes(),
                layout.register_bytes(),
                layout.request_bytes(),
                layout.reply_bytes(),
                request.read_bytes(),
                request.write_bytes(),
            ])?,
            cancel,
        )?;
        use RangeNumericReply as R;
        use RangeNumericRequest as Q;
        Ok(match request {
            Q::SourceUnit { index } => {
                R::Unit(*source.get(index).ok_or(DomError::InputProcessingProtocol)?)
            }
            Q::ReadCells { cells } => {
                let mut values = [None; 3];
                for (out, cell) in values.iter_mut().zip(cells) {
                    if let Some(cell) = cell {
                        *out = Some(
                            *self.banks[cell.role().index()]
                                .get(cell.index())
                                .ok_or(DomError::InputProcessingProtocol)?,
                        );
                    }
                }
                R::Cells(values)
            }
            Q::Reserve { role, required_len } => {
                let bank = &mut self.banks[role.index()];
                meter.reserve(bank, required_len, cancel)?;
                R::Reserved {
                    actual_capacity: bank.capacity(),
                    actual_len: bank.len(),
                }
            }
            Q::AppendCell { role, value } => {
                let bank = &mut self.banks[role.index()];
                if value > 9 || bank.len() == bank.capacity() {
                    return Err(DomError::InputProcessingProtocol);
                }
                bank.push(value);
                #[cfg(test)]
                super::committed(super::Commit::RangeCell, cancel);
                R::Ack
            }
            Q::WriteCell { cell, value } => {
                if value > 9 {
                    return Err(DomError::InputProcessingProtocol);
                }
                *self.banks[cell.role().index()]
                    .get_mut(cell.index())
                    .ok_or(DomError::InputProcessingProtocol)? = value;
                R::Ack
            }
            Q::PopCell { role } => {
                self.banks[role.index()]
                    .pop()
                    .ok_or(DomError::InputProcessingProtocol)?;
                R::Ack
            }
            Q::ScalarStep => R::Ack,
            Q::ReserveOutput { required_len } => {
                meter.reserve(&mut self.output, required_len, cancel)?;
                R::OutputReserved {
                    actual_capacity: self.output.capacity(),
                    actual_len: self.output.len(),
                }
            }
            Q::EmitUnit { unit } => {
                if self.output.len() == self.output.capacity() {
                    return Err(DomError::InputProcessingProtocol);
                }
                self.output.push(unit);
                #[cfg(test)]
                super::committed(super::Commit::OutputUnit, cancel);
                R::Ack
            }
        })
    }
}

fn frame(layout: RangeNumericLayout) -> Result<usize, DomError> {
    sum(&[
        layout.register_bytes(),
        layout.headers_bytes(),
        layout.request_bytes(),
        layout.reply_bytes(),
    ])
}

fn parse(
    meter: &mut InputProcessing,
    storage: &mut Storage,
    raw: &DocumentString,
    mode: FloatingPointParseMode,
    cancel: &AtomicBool,
) -> Result<HtmlInputRangeValue, DomError> {
    let grammar_bytes = size_of::<FloatingPointGrammarCursor<'_>>();
    meter.fixed(grammar_bytes, cancel)?;
    let parsed = (|| {
        let mut grammar = FloatingPointGrammarCursor::new(raw.code_units(), mode);
        while !grammar.is_finished() {
            meter.charge(
                sum(&[
                    1,
                    grammar_bytes,
                    if grammar.inspects_unit() { 2 } else { 0 },
                ])?,
                cancel,
            )?;
            grammar.advance();
        }
        if !grammar.is_valid() {
            return Ok(None);
        }
        grammar
            .range_decimal_parts()
            .map(Some)
            .map_err(|_| DomError::InputProcessingProtocol)
    })();
    meter.release(grammar_bytes);
    let Some(parts) = parsed? else {
        return Ok(HtmlInputRangeValue::InvalidGrammar);
    };
    storage.clear(meter, cancel)?;
    let layout = FiniteDecimalIngressCursor::layout();
    let fixed = sum(&[frame(layout)?, size_of_val(&parts)])?;
    meter.fixed(fixed, cancel)?;
    let result = (|| {
        let mut cursor = FiniteDecimalIngressCursor::new(parts, raw.len(), storage.headers())
            .map_err(DomError::InputNumeric)?;
        while let Some(request) = cursor.request() {
            let reply = storage.service(meter, raw.code_units(), request, layout, cancel)?;
            cursor.reply(reply).map_err(DomError::InputNumeric)?;
        }
        if cursor.headers() != storage.headers() {
            return Err(DomError::InputProcessingProtocol);
        }
        match cursor.result().ok_or(DomError::InputProcessingProtocol)? {
            Ok(bits) => Ok(HtmlInputRangeValue::Finite(
                HtmlInputRangeNumber::try_from(f64::from_bits(bits))
                    .map_err(|_| DomError::InputProcessingProtocol)?,
            )),
            Err(RangeNumericError::Overflow) => Ok(HtmlInputRangeValue::ConversionFailed),
            Err(error) => Err(DomError::InputNumeric(error)),
        }
    })();
    meter.release(fixed);
    result
}

fn content(
    meter: &mut InputProcessing,
    storage: &mut Storage,
    raw: Option<&DocumentString>,
    cancel: &AtomicBool,
) -> Result<HtmlInputNumericContent, DomError> {
    meter.charge(sum(&[1, size_of_val(&raw)])?, cancel)?;
    Ok(match raw {
        None => HtmlInputNumericContent::Absent,
        Some(raw) => match parse(
            meter,
            storage,
            raw,
            FloatingPointParseMode::Recovery,
            cancel,
        )? {
            HtmlInputRangeValue::Finite(value) => HtmlInputNumericContent::Value(
                HtmlFloatingPointNumber::try_from(value.value())
                    .map_err(|_| DomError::InputProcessingProtocol)?,
            ),
            _ => HtmlInputNumericContent::Failed,
        },
    })
}

fn step(
    meter: &mut InputProcessing,
    storage: &mut Storage,
    raw: Option<&DocumentString>,
    cancel: &AtomicBool,
) -> Result<HtmlInputNumericStep, DomError> {
    meter.charge(sum(&[1, size_of_val(&raw)])?, cancel)?;
    let Some(raw) = raw else {
        return Ok(HtmlInputNumericStep::Absent);
    };
    let mut any = raw.len() == HtmlInputNumericStep::ANY_KEYWORD.len();
    if any {
        for index in 0..HtmlInputNumericStep::ANY_KEYWORD.len() {
            meter.charge(3, cancel)?;
            if !HtmlInputNumericStep::any_keyword_unit_matches(index, raw.code_units()[index]) {
                any = false;
                break;
            }
        }
    }
    if any {
        return Ok(HtmlInputNumericStep::Any);
    }
    Ok(match content(meter, storage, Some(raw), cancel)? {
        HtmlInputNumericContent::Value(value) => HtmlInputNumericStep::Value(value),
        _ => HtmlInputNumericStep::Failed,
    })
}

fn midpoint(
    meter: &mut InputProcessing,
    storage: &mut Storage,
    min: HtmlInputRangeNumber,
    max: HtmlInputRangeNumber,
    cancel: &AtomicBool,
) -> Result<HtmlInputRangeNumber, DomError> {
    let layout = RangeMidpointCursor::layout();
    let fixed = frame(layout)?;
    meter.fixed(fixed, cancel)?;
    let result = (|| {
        let mut cursor = RangeMidpointCursor::new(min.value().to_bits(), max.value().to_bits())
            .map_err(DomError::InputNumeric)?;
        while let Some(request) = cursor.request() {
            let reply = storage.service(meter, &[], request, layout, cancel)?;
            cursor.reply(reply).map_err(DomError::InputNumeric)?;
        }
        HtmlInputRangeNumber::try_from(f64::from_bits(
            cursor.result().ok_or(DomError::InputProcessingProtocol)?,
        ))
        .map_err(|_| DomError::InputProcessingProtocol)
    })();
    meter.release(fixed);
    result
}

fn best(
    meter: &mut InputProcessing,
    storage: &mut Storage,
    value: HtmlInputRangeNumber,
    cancel: &AtomicBool,
) -> Result<Vec<u16>, DomError> {
    if !storage.output.is_empty() {
        return Err(DomError::InputProcessingProtocol);
    }
    storage.clear(meter, cancel)?;
    let layout = BestFiniteNumberCursor::layout();
    let fixed = frame(layout)?;
    meter.fixed(fixed, cancel)?;
    let result = (|| {
        let mut cursor = BestFiniteNumberCursor::new(value.value(), storage.headers())
            .map_err(DomError::InputNumeric)?;
        while let Some(request) = cursor.request() {
            let reply = storage.service(meter, &[], request, layout, cancel)?;
            cursor.reply(reply).map_err(DomError::InputNumeric)?;
        }
        if cursor.headers() != storage.headers() || cursor.result() != Some(storage.output.len()) {
            return Err(DomError::InputProcessingProtocol);
        }
        Ok(std::mem::take(&mut storage.output))
    })();
    meter.release(fixed);
    result
}

struct LatticeInput {
    current: HtmlInputRangeNumber,
    minimum: HtmlInputRangeNumber,
    maximum: Option<HtmlInputRangeNumber>,
    base: HtmlInputRangeNumber,
    step: HtmlInputRangeNumber,
}

fn lattice(
    meter: &mut InputProcessing,
    storage: &mut Storage,
    input: LatticeInput,
    cancel: &AtomicBool,
) -> Result<Option<HtmlInputRangeNumber>, DomError> {
    let layout = RangeLatticeCursor::layout();
    let fixed = sum(&[frame(layout)?, size_of::<LatticeInput>()])?;
    meter.fixed(fixed, cancel)?;
    let result = (|| {
        let mut cursor = RangeLatticeCursor::new(
            input.current.value().to_bits(),
            input.minimum.value().to_bits(),
            input.maximum.map(|value| value.value().to_bits()),
            input.base.value().to_bits(),
            input.step.value().to_bits(),
        )
        .map_err(DomError::InputNumeric)?;
        while let Some(request) = cursor.request() {
            let reply = storage.service(meter, &[], request, layout, cancel)?;
            cursor.reply(reply).map_err(DomError::InputNumeric)?;
        }
        cursor
            .result()
            .ok_or(DomError::InputProcessingProtocol)?
            .map(|bits| {
                HtmlInputRangeNumber::try_from(f64::from_bits(bits))
                    .map_err(|_| DomError::InputProcessingProtocol)
            })
            .transpose()
    })();
    meter.release(fixed);
    result
}

pub(super) fn run(
    meter: &mut InputProcessing,
    input: RangeInput<'_>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    meter.fixed(
        sum(&[
            size_of::<InputProcessing>(),
            size_of::<RangeInput<'_>>(),
            size_of::<Storage>(),
            size_of::<Option<Vec<u16>>>(),
            size_of::<DocumentString>(),
            size_of::<HtmlInputRangeValue>(),
            2 * size_of::<HtmlInputNumericContent>(),
        ])?,
        cancel,
    )?;
    let mut storage = Storage::new();
    let mut output: Option<Vec<u16>> = None;
    let current_value = parse(
        meter,
        &mut storage,
        input.current,
        FloatingPointParseMode::Author,
        cancel,
    )?;
    let min = content(meter, &mut storage, input.min, cancel)?;
    let max = content(meter, &mut storage, input.max, cancel)?;
    meter.fixed(
        sum(&[
            size_of::<HtmlInputRangeSanitizationCursor>(),
            size_of::<Action>(),
            size_of::<Reply>(),
        ])?,
        cancel,
    )?;
    let mut policy = HtmlInputRangeSanitizationCursor::new(current_value, min, max);
    loop {
        meter.charge(
            sum(&[
                1,
                2 * size_of::<HtmlInputRangeSanitizationCursor>(),
                size_of::<Action>(),
                size_of::<Reply>(),
            ])?,
            cancel,
        )?;
        let reply = match policy.action() {
            Action::Midpoint { minimum, maximum } => {
                Reply::Midpoint(midpoint(meter, &mut storage, minimum, maximum, cancel)?)
            }
            Action::Replace { value, .. } => {
                let replacement = best(meter, &mut storage, value, cancel)?;
                if let Some(previous) = output.replace(replacement) {
                    let bytes = extent(previous.capacity(), size_of::<u16>())?;
                    drop(previous);
                    meter.release(bytes);
                }
                Reply::Applied
            }
            Action::ReadStep => Reply::Step(step(meter, &mut storage, input.step, cancel)?),
            Action::ReadValueContent => {
                Reply::ValueContent(content(meter, &mut storage, input.default_value, cancel)?)
            }
            Action::Lattice {
                current,
                minimum,
                maximum,
                base,
                step,
            } => Reply::Lattice(lattice(
                meter,
                &mut storage,
                LatticeInput {
                    current,
                    minimum,
                    maximum,
                    base,
                    step,
                },
                cancel,
            )?),
            Action::Complete => break,
        };
        policy
            .reply(reply)
            .map_err(|_| DomError::InputProcessingProtocol)?;
    }
    match output {
        Some(units) => meter.output(units, cancel),
        None => {
            meter.charge(sum(&[1, size_of::<DocumentString>()])?, cancel)?;
            Ok(input.current.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_uses_actual_grammar_midpoint_clamp_lattice_and_lexical_preservation() {
        // HTML877d defaults, exact dyadic ties upward, and signed-zero/raw
        // preservation when no actual replacement is requested.
        for (raw, min, max, step, expected) in [
            ("", None, None, None, "50"),
            ("101", None, None, None, "100"),
            ("-1", None, None, None, "0"),
            ("1.5", None, None, Some("1"), "2"),
            ("bad", Some("6"), Some("2"), None, "6"),
            ("5.00", None, None, Some("AnY"), "5.00"),
            ("-0", None, None, Some("any"), "-0"),
            ("1e999", None, None, Some("any"), "1e999"),
        ] {
            let cancel = AtomicBool::new(false);
            let mut meter = super::super::tests::fixture();
            let min = min.map(DocumentString::from);
            let max = max.map(DocumentString::from);
            let step = step.map(DocumentString::from);
            let actual = meter
                .range(
                    RangeInput {
                        current: &DocumentString::from(raw),
                        min: min.as_ref(),
                        max: max.as_ref(),
                        step: step.as_ref(),
                        default_value: None,
                    },
                    &cancel,
                )
                .unwrap();
            assert_eq!(actual, DocumentString::from(expected), "raw={raw}");
            assert_eq!(meter.live, 0);
            assert!(meter.stopped.is_none());
        }
    }

    #[test]
    fn range_uses_actual_content_as_step_base_only_when_requested() {
        let cancel = AtomicBool::new(false);
        let mut meter = super::super::tests::fixture();
        // No finite min: value content 1 selects odd lattice points. 4 is
        // equidistant from 3 and 5, so source tie-breaking selects 5.
        assert_eq!(
            meter
                .range(
                    RangeInput {
                        current: &DocumentString::from("4"),
                        min: None,
                        max: None,
                        step: Some(&DocumentString::from("2")),
                        default_value: Some(&DocumentString::from("1")),
                    },
                    &cancel,
                )
                .unwrap(),
            DocumentString::from("5")
        );
        assert_eq!(meter.live, 0);
    }
}
