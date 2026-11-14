use crate::accounting::{Account, RenderWork};
use crate::{Error, MorphologyNumericPhase, MorphologyPreparationError, Result};

/// Reached, rebased values for the fixed morphology uniform protocol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MorphologyParameterFacts {
    pub(crate) stage: u32,
    pub(crate) source_size: [u32; 2],
    pub(crate) target_size: [u32; 2],
    pub(crate) source_base: [i32; 2],
    pub(crate) source_clip: [f64; 4],
    pub(crate) output_clip: [f64; 4],
    pub(crate) periodic_base: [f64; 2],
    pub(crate) periodic_step: [f64; 2],
    pub(crate) periodic: bool,
    pub(crate) converted: bool,
    pub(crate) linear: bool,
    pub(crate) operator: u32,
    pub(crate) axis: u32,
    pub(crate) level: u32,
    pub(crate) span: u32,
    pub(crate) input_length: u32,
    pub(crate) lower_kind: u32,
    pub(crate) lower_delta: i32,
    pub(crate) upper_kind: u32,
    pub(crate) upper_delta: i32,
    pub(crate) valid_output: [i32; 2],
    pub(crate) constant_length: Option<u32>,
    pub(crate) color: [f32; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MorphologyParameterBytes([u8; 256]);

fn invalid(phase: MorphologyNumericPhase, value: f64) -> Error {
    MorphologyPreparationError::Numeric { phase, value }.into()
}

impl MorphologyParameterBytes {
    pub(crate) fn try_new(facts: MorphologyParameterFacts, account: &Account) -> Result<Self> {
        let layout_error = |value| invalid(MorphologyNumericPhase::TableLayout, value);
        if facts.stage > 8 || facts.axis > 1 || facts.operator > 1 {
            return Err(layout_error(f64::from(facts.stage)));
        }
        for size in facts.target_size.into_iter().chain(if facts.stage == 8 {
            [0; 2]
        } else {
            facts.source_size
        }) {
            if size > 8_388_608 {
                return Err(layout_error(f64::from(size)));
            }
        }
        let query = matches!(facts.stage, 4..=6);
        let table = facts.stage == 3;
        if (table || query)
            && (facts.level > 23
                || facts.span != 1_u32.checked_shl(facts.level).unwrap_or(0)
                || facts.input_length == 0
                || facts.input_length > 8_388_608
                || facts.span > facts.input_length
                || (table && facts.level == 0)
                || (facts.stage == 4 && facts.level != 0))
        {
            return Err(layout_error(f64::from(facts.span)));
        }
        if query {
            let axis =
                usize::try_from(facts.axis).map_err(|_| layout_error(f64::from(facts.axis)))?;
            let target_length = i64::from(facts.target_size[axis]);
            if facts.lower_kind > 2
                || facts.upper_kind > 2
                || facts.valid_output[0] < 0
                || facts.valid_output[1] < facts.valid_output[0]
                || i64::from(facts.valid_output[1]) > target_length
                || facts
                    .constant_length
                    .is_some_and(|length| length == 0 || length > facts.input_length)
                || (facts.periodic && facts.constant_length.is_none())
                || (facts.stage == 6 && facts.constant_length.is_none())
            {
                return Err(layout_error(f64::from(facts.input_length)));
            }
            if let Some(length) = facts.constant_length
                && ((facts.stage == 4 && length != 1)
                    || (facts.stage == 6 && length.ilog2() != facts.level))
            {
                return Err(layout_error(f64::from(length)));
            }
            for (kind, delta) in [
                (facts.lower_kind, facts.lower_delta),
                (facts.upper_kind, facts.upper_delta),
            ] {
                if kind == 2 || facts.periodic {
                    for j in [0, target_length.saturating_sub(1)] {
                        let value = j + i64::from(delta);
                        if i32::try_from(value).is_err() {
                            return Err(invalid(
                                MorphologyNumericPhase::EndpointMapping,
                                value as f64,
                            ));
                        }
                    }
                }
            }
        }
        if matches!(facts.stage, 0..=2 | 4..=7) {
            for (base, extent) in facts.source_base.into_iter().zip(facts.target_size) {
                let end = i64::from(base) + i64::from(extent).saturating_sub(1);
                if i32::try_from(end).is_err() {
                    return Err(invalid(MorphologyNumericPhase::GridIndex, end as f64));
                }
            }
        }
        let mut words = [[0_u32; 4]; 16];
        words[0] = [
            facts.source_size[0],
            facts.source_size[1],
            facts.target_size[0],
            facts.target_size[1],
        ];
        words[1] = [
            facts.source_base[0],
            facts.source_base[1],
            facts.valid_output[0],
            facts.valid_output[1],
        ]
        .map(|value| u32::from_ne_bytes(value.to_ne_bytes()));
        let mut floats = |row: usize, values: [f64; 4]| -> Result<()> {
            for (slot, value) in words[row].iter_mut().zip(values) {
                let uploaded = account.coordinate(|| value as f32)?;
                if !value.is_finite()
                    || !(uploaded == 0.0 || uploaded.is_normal())
                    || (value != 0.0 && uploaded == 0.0)
                {
                    return Err(invalid(MorphologyNumericPhase::Sampling, value));
                }
                *slot = uploaded.to_bits();
            }
            Ok(())
        };
        for (needed, row, clip, size) in [
            (
                !facts.periodic && matches!(facts.stage, 1 | 2),
                2,
                facts.source_clip,
                facts.source_size,
            ),
            (
                matches!(facts.stage, 1 | 7 | 8),
                3,
                facts.output_clip,
                facts.target_size,
            ),
        ] {
            if needed {
                if clip[0] < 0.0
                    || clip[1] < 0.0
                    || clip[2] < clip[0]
                    || clip[3] < clip[1]
                    || clip[2] > f64::from(size[0])
                    || clip[3] > f64::from(size[1])
                {
                    return Err(invalid(MorphologyNumericPhase::GridIndex, clip[0]));
                }
                floats(row, clip)?;
            }
        }
        if facts.periodic && matches!(facts.stage, 1 | 7) {
            if facts.source_size.contains(&0)
                || facts.periodic_step.into_iter().any(|value| value <= 0.0)
            {
                return Err(invalid(
                    MorphologyNumericPhase::Sampling,
                    facts.periodic_step[0],
                ));
            }
            floats(
                4,
                [
                    facts.periodic_base[0],
                    facts.periodic_base[1],
                    facts.periodic_step[0],
                    facts.periodic_step[1],
                ],
            )?;
        }
        words[5] = [
            facts.span,
            facts.level,
            u32::from(facts.periodic),
            facts.input_length,
        ];
        words[6] = [
            facts.lower_kind,
            u32::from_ne_bytes(facts.lower_delta.to_ne_bytes()),
            facts.upper_kind,
            u32::from_ne_bytes(facts.upper_delta.to_ne_bytes()),
        ];
        words[7] = [
            facts.stage,
            u32::from(facts.linear),
            facts.operator,
            facts.axis,
        ];
        if facts.stage == 8 {
            if facts
                .color
                .into_iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
                || facts.color[..3].iter().any(|value| *value > facts.color[3])
            {
                return Err(invalid(
                    MorphologyNumericPhase::Sampling,
                    f64::from(facts.color[3]),
                ));
            }
            words[8] = facts.color.map(f32::to_bits);
        }
        words[9] = [
            facts.constant_length.unwrap_or(0),
            u32::from(facts.constant_length.is_some()),
            u32::from(facts.converted),
            0,
        ];
        account.work(RenderWork::SerializedBytes, 256)?;
        let mut bytes = [0; 256];
        for (index, word) in words.into_iter().flatten().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ok(Self(bytes))
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 256] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> MorphologyParameterFacts {
        MorphologyParameterFacts {
            stage: 5,
            source_size: [5, 4],
            target_size: [7, 4],
            source_base: [0, 0],
            source_clip: [0.0, 0.0, 5.0, 4.0],
            output_clip: [0.0, 0.0, 7.0, 4.0],
            periodic_base: [0.0; 2],
            periodic_step: [1.0; 2],
            periodic: false,
            converted: true,
            linear: true,
            operator: 1,
            axis: 0,
            level: 1,
            span: 2,
            input_length: 6,
            lower_kind: 2,
            lower_delta: -2,
            upper_kind: 2,
            upper_delta: 3,
            valid_output: [0, 7],
            constant_length: None,
            color: [0.0; 4],
        }
    }

    #[test]
    fn exact_wire_preserves_signed_endpoints_and_stage_precision_flags() {
        let bytes = MorphologyParameterBytes::try_new(facts(), &Account::for_test()).unwrap();
        let word =
            |offset| u32::from_le_bytes(bytes.as_bytes()[offset..offset + 4].try_into().unwrap());
        assert_eq!(word(0), 5);
        assert_eq!(word(8), 7);
        assert_eq!(word(28), 7);
        assert_eq!(word(80), 2);
        assert_eq!(word(84), 1);
        assert_eq!(word(92), 6);
        assert_eq!(word(96), 2);
        assert_eq!(word(100), (-2_i32).cast_unsigned());
        assert_eq!(word(108), 3);
        assert_eq!(word(112), 5);
        assert_eq!(word(116), 1);
        assert_eq!(word(120), 1);
        assert_eq!(word(152), 1);
        assert!(bytes.as_bytes()[160..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn unsafe_endpoint_addition_and_unproved_table_span_are_refused() {
        let account = Account::for_test();
        let mut input = facts();
        input.lower_delta = i32::MAX;
        assert!(MorphologyParameterBytes::try_new(input, &account).is_err());
        input = facts();
        input.span = 3;
        assert!(MorphologyParameterBytes::try_new(input, &account).is_err());
        input = facts();
        input.level = 32;
        assert!(MorphologyParameterBytes::try_new(input, &account).is_err());
        input = facts();
        input.valid_output = [-1, 7];
        assert!(MorphologyParameterBytes::try_new(input, &account).is_err());
    }

    #[test]
    fn constant_has_no_reached_source_or_radius_mapping() {
        let mut input = facts();
        input.stage = 8;
        input.source_size = [u32::MAX; 2];
        input.source_base = [i32::MAX; 2];
        input.color = [0.125, 0.25, 0.0, 0.5];
        input.periodic_base = [f64::INFINITY; 2];
        let bytes = MorphologyParameterBytes::try_new(input, &Account::for_test()).unwrap();
        assert_eq!(&bytes.as_bytes()[128..132], &0.125_f32.to_le_bytes());
        assert_eq!(&bytes.as_bytes()[140..144], &0.5_f32.to_le_bytes());
    }
}
