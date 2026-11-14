//! The fixed uniform protocol consumes qualified integers, never floating offsets.
use crate::{
    DisplacementNumericPhase, DisplacementPreparationError, Error, Result,
    accounting::{Account, RenderWork},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DisplacementParameterFacts {
    pub(crate) source_size: [u32; 2],
    pub(crate) target_size: [u32; 2],
    pub(crate) source_base: [i32; 2],
    pub(crate) source_clip: [f64; 4],
    pub(crate) output_clip: [f64; 4],
    pub(crate) periodic_base: [f64; 2],
    pub(crate) periodic_step: [f64; 2],
    pub(crate) color: [f32; 4],
    pub(crate) stage: u32,
    pub(crate) source_kind: u32,
    pub(crate) channels: [u32; 2],
    pub(crate) raw_map: bool,
    pub(crate) map_size: [u32; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementParameterBytes([u8; 256]);
impl DisplacementParameterBytes {
    pub(crate) fn try_new(facts: DisplacementParameterFacts, account: &Account) -> Result<Self> {
        if facts.stage == 0
            || facts.stage > 4
            || facts.source_kind > 2
            || facts.channels.into_iter().any(|channel| channel > 3)
            || facts
                .target_size
                .into_iter()
                .chain(facts.source_size)
                .any(|size| size > 8_388_608)
            || (facts.raw_map && facts.map_size.contains(&0))
        {
            return Err(invalid(
                DisplacementNumericPhase::TableLayout,
                f64::from(facts.stage),
            ));
        }
        if facts.stage != 3 {
            for (base, length) in facts.source_base.into_iter().zip(facts.target_size) {
                let end = i64::from(base) + i64::from(length).saturating_sub(1);
                if i32::try_from(end).is_err() {
                    return Err(invalid(DisplacementNumericPhase::GridIndex, end as f64));
                }
            }
        }
        let mut words = [[0u32; 4]; 16];
        words[0] = [
            facts.source_size[0],
            facts.source_size[1],
            facts.target_size[0],
            facts.target_size[1],
        ];
        words[1] = [
            u32::from_ne_bytes(facts.source_base[0].to_ne_bytes()),
            u32::from_ne_bytes(facts.source_base[1].to_ne_bytes()),
            facts.channels[0],
            facts.channels[1],
        ];
        let mut floats = |row: usize, values: [f64; 4]| -> Result<()> {
            for (slot, value) in words[row].iter_mut().zip(values) {
                let uploaded = account.coordinate(|| value as f32)?;
                if !value.is_finite()
                    || !(uploaded == 0.0 || uploaded.is_normal())
                    || (value != 0.0 && uploaded == 0.0)
                {
                    return Err(invalid(DisplacementNumericPhase::Sampling, value));
                }
                *slot = uploaded.to_bits();
            }
            Ok(())
        };
        if facts.stage != 0 && facts.stage != 4 {
            floats(2, facts.source_clip)?;
        }
        if facts.stage != 0 {
            floats(3, facts.output_clip)?;
        }
        if facts.source_kind == 2 && (facts.stage == 1 || facts.stage == 2) {
            if facts.source_size.contains(&0)
                || facts.periodic_step.into_iter().any(|value| value <= 0.0)
            {
                return Err(invalid(
                    DisplacementNumericPhase::Sampling,
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
        if facts.source_kind == 1 && (facts.stage == 1 || facts.stage == 2) {
            if facts
                .color
                .into_iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
                || facts.color[..3].iter().any(|value| *value > facts.color[3])
            {
                return Err(invalid(
                    DisplacementNumericPhase::Sampling,
                    f64::from(facts.color[3]),
                ));
            }
            words[5] = facts.color.map(f32::to_bits);
        }
        words[6] = [
            facts.stage,
            facts.source_kind,
            u32::from(facts.raw_map),
            facts.target_size[0],
        ];
        words[7] = [facts.map_size[0], facts.map_size[1], 0, 0];
        account.work(RenderWork::SerializedBytes, 256)?;
        let mut bytes = [0; 256];
        for (index, word) in words.into_iter().flatten().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ok(Self(bytes))
    }
    pub(crate) fn as_bytes(&self) -> &[u8; 256] {
        &self.0
    }
}
fn invalid(phase: DisplacementNumericPhase, value: f64) -> Error {
    DisplacementPreparationError::Numeric { phase, value }.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn displacement_shader_accepts_integer_address_and_record_wire_layouts() {
        let source = crate::shader::pipeline::DISPLACEMENT_WGSL;
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
        let mut found = [false; 3];
        for (_, variable) in module.global_variables.iter() {
            let Some(binding) = &variable.binding else {
                continue;
            };
            match binding.binding {
                2 => {
                    let wgpu::naga::TypeInner::Struct { span, .. } =
                        module.types[variable.ty].inner
                    else {
                        panic!("uniform wire must be a structure");
                    };
                    assert_eq!(span, 256);
                    found[0] = true;
                }
                3 | 4 => {
                    let wgpu::naga::TypeInner::Array { stride, .. } =
                        module.types[variable.ty].inner
                    else {
                        panic!("storage wire must be an array");
                    };
                    assert_eq!(stride, if binding.binding == 3 { 8 } else { 16 });
                    found[usize::try_from(binding.binding - 2).unwrap()] = true;
                }
                _ => {}
            }
        }
        assert_eq!(found, [true; 3]);
    }
    #[test]
    fn displacement_uniform_preserves_signed_integer_rebase_and_typed_channels() {
        let bytes = DisplacementParameterBytes::try_new(
            DisplacementParameterFacts {
                source_size: [4, 3],
                target_size: [2, 2],
                source_base: [-1, 2],
                source_clip: [0.0, 0.0, 4.0, 3.0],
                output_clip: [0.25, 0.0, 2.0, 2.0],
                periodic_base: [0.0; 2],
                periodic_step: [1.0; 2],
                color: [0.0; 4],
                stage: 1,
                source_kind: 0,
                channels: [0, 3],
                raw_map: false,
                map_size: [0; 2],
            },
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(
            &bytes.as_bytes()[16..32],
            &[255, 255, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0]
        );
        assert_eq!(&bytes.as_bytes()[48..52], &0.25f32.to_le_bytes());
    }
}
