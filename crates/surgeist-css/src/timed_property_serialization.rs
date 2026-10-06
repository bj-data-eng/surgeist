//! Canonical authored timing lists, before list expansion or timing evaluation.
//!
//! Lists and shorthand items each charge one input and projection node. Names,
//! keywords and iteration-count variants charge one each; numeric children use
//! their existing domain policy. Duration wrappers add no node around a time.
//! Required disambiguating defaults add one projection node and no input node.
//! Punctuation adds only bytes. Explicit optional values are never suppressed.

use crate::{
    CssAnimation, CssAnimationDirection, CssAnimationDirectionList, CssAnimationFillMode,
    CssAnimationFillModeList, CssAnimationIterationCount, CssAnimationIterationCountList,
    CssAnimationList, CssAnimationName, CssAnimationNameList, CssAnimationPlayState,
    CssAnimationPlayStateList, CssDelayList, CssDurationList, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssTransition,
    CssTransitionList, CssTransitionProperty, CssTransitionPropertyList,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! list_source_visit {
    ($writer:expr, $visit:expr) => {
        ($visit)($writer)
    };
    ($writer:expr, $visit:expr, $source:ident) => {
        $writer.source_property(crate::CssKnownProperty::$source, $visit)
    };
}

macro_rules! list_provider {
    ($ty:ty, $getter:ident, $append:expr $(, source = $source:ident)?) => {
        impl $ty {
            /// Emits canonical authored values in retained list order.
            /// This performs no timing evaluation or contextual list expansion.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Emits atomically using cumulative input, projection and byte limits.
            /// Lists and items charge nodes; required defaults charge projection only.
            /// Numeric children retain their shared domain serialization policy.
            pub fn serialize_specified_with_limits(
                &self,
                limits: CssSpecifiedValueSerializationLimits,
            ) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }

            pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
                // The shorthand list names its actual admitted item-count owner.
                // Terminal lists keep the ordinary node and separator path.
                list_source_visit!(writer, |writer| node(writer) $(, $source)?)?;
                let values = self.$getter();
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        list_source_visit!(writer, |writer: &mut SpecifiedRuleWriter| writer.append(", ") $(, $source)?)?;
                    }
                    ($append)(writer, value)?;
                }
                Ok(())
            }
        }
    };
}

list_provider!(CssTransitionPropertyList, properties, transition_property);
list_provider!(
    CssDurationList,
    values,
    |writer, value: &crate::CssDuration| { value.append_to_rule_writer(writer) }
);
list_provider!(
    CssDelayList,
    values,
    |writer, value: &crate::CssTimeValue| { value.append_to_rule_writer(writer) }
);
list_provider!(
    CssTransitionList,
    values,
    transition,
    source = TransitionProperty
);
list_provider!(CssAnimationNameList, names, animation_name);
list_provider!(CssAnimationIterationCountList, values, iteration_count);
list_provider!(
    CssAnimationDirectionList,
    directions,
    |writer, value: &CssAnimationDirection| { direction(writer, *value) }
);
list_provider!(
    CssAnimationFillModeList,
    modes,
    |writer, value: &CssAnimationFillMode| { fill_mode(writer, *value) }
);
list_provider!(
    CssAnimationPlayStateList,
    states,
    |writer, value: &CssAnimationPlayState| { play_state(writer, *value) }
);
list_provider!(CssAnimationList, values, animation, source = AnimationName);

fn node(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

fn unrepresentable() -> CssSpecifiedValueSerializationError {
    CssSpecifiedValueSerializationError::new(
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
    )
}

fn transition_property(
    writer: &mut SpecifiedRuleWriter,
    value: &CssTransitionProperty,
) -> Result<()> {
    node(writer)?;
    match value {
        CssTransitionProperty::All => writer.append("all"),
        CssTransitionProperty::None => writer.append("none"),
        CssTransitionProperty::Custom(name) => writer.append_identifier(name.as_str()),
    }
}

fn animation_name(writer: &mut SpecifiedRuleWriter, value: &CssAnimationName) -> Result<()> {
    node(writer)?;
    match value {
        CssAnimationName::None => writer.append("none"),
        CssAnimationName::Custom(name) => writer.append_identifier(name.as_str()),
        CssAnimationName::String(name) => {
            if name.as_str().contains('\0') {
                return Err(unrepresentable());
            }
            writer.append_string(name.as_str())
        }
    }
}

fn iteration_count(
    writer: &mut SpecifiedRuleWriter,
    value: &CssAnimationIterationCount,
) -> Result<()> {
    node(writer)?;
    match value {
        CssAnimationIterationCount::Infinite => writer.append("infinite"),
        CssAnimationIterationCount::Number(number) => number.append_to_rule_writer(writer),
    }
}

fn direction(writer: &mut SpecifiedRuleWriter, value: CssAnimationDirection) -> Result<()> {
    node(writer)?;
    writer.append(match value {
        CssAnimationDirection::Normal => "normal",
        CssAnimationDirection::Reverse => "reverse",
        CssAnimationDirection::Alternate => "alternate",
        CssAnimationDirection::AlternateReverse => "alternate-reverse",
    })
}

fn fill_mode(writer: &mut SpecifiedRuleWriter, value: CssAnimationFillMode) -> Result<()> {
    node(writer)?;
    writer.append(match value {
        CssAnimationFillMode::None => "none",
        CssAnimationFillMode::Forwards => "forwards",
        CssAnimationFillMode::Backwards => "backwards",
        CssAnimationFillMode::Both => "both",
    })
}

fn play_state(writer: &mut SpecifiedRuleWriter, value: CssAnimationPlayState) -> Result<()> {
    node(writer)?;
    writer.append(match value {
        CssAnimationPlayState::Running => "running",
        CssAnimationPlayState::Paused => "paused",
    })
}

fn keyword(name: &str, keywords: &[&str]) -> bool {
    keywords
        .iter()
        .any(|value| name.eq_ignore_ascii_case(value))
}

fn easing_name(name: &str) -> bool {
    keyword(
        name,
        &[
            "ease",
            "linear",
            "ease-in",
            "ease-out",
            "ease-in-out",
            "step-start",
            "step-end",
        ],
    )
}

fn separator(writer: &mut SpecifiedRuleWriter, started: &mut bool) -> Result<()> {
    if *started {
        writer.append(" ")?;
    }
    *started = true;
    Ok(())
}

fn default_slot(writer: &mut SpecifiedRuleWriter, started: &mut bool, text: &str) -> Result<()> {
    separator(writer, started)?;
    writer.context.charge_projection(1)?;
    writer.append(text)
}

fn transition(writer: &mut SpecifiedRuleWriter, value: &CssTransition) -> Result<()> {
    writer.source_property(crate::CssKnownProperty::TransitionProperty, node)?;
    let mut started = false;
    let collision = matches!(value.property(), Some(CssTransitionProperty::Custom(name)) if easing_name(name.as_str()));
    if !collision && let Some(property) = value.property() {
        writer.source_property(crate::CssKnownProperty::TransitionProperty, |writer| {
            separator(writer, &mut started)?;
            transition_property(writer, property)
        })?;
    }
    if let Some(duration) = value.duration() {
        writer.source_property(crate::CssKnownProperty::TransitionDuration, |writer| {
            separator(writer, &mut started)?;
            duration.append_to_rule_writer(writer)
        })?;
    } else if value.delay().is_some() {
        writer.source_property(crate::CssKnownProperty::TransitionDuration, |writer| {
            default_slot(writer, &mut started, "0s")
        })?;
    }
    if let Some(easing) = value.timing_function() {
        writer.source_property(
            crate::CssKnownProperty::TransitionTimingFunction,
            |writer| {
                separator(writer, &mut started)?;
                easing.append_to_rule_writer(writer)
            },
        )?;
    } else if collision {
        writer.source_property(
            crate::CssKnownProperty::TransitionTimingFunction,
            |writer| default_slot(writer, &mut started, "ease"),
        )?;
    }
    if let Some(delay) = value.delay() {
        writer.source_property(crate::CssKnownProperty::TransitionDelay, |writer| {
            separator(writer, &mut started)?;
            delay.append_to_rule_writer(writer)
        })?;
    }
    if collision && let Some(property) = value.property() {
        writer.source_property(crate::CssKnownProperty::TransitionProperty, |writer| {
            separator(writer, &mut started)?;
            transition_property(writer, property)
        })?;
    }
    Ok(())
}

fn animation(writer: &mut SpecifiedRuleWriter, value: &CssAnimation) -> Result<()> {
    writer.source_property(crate::CssKnownProperty::AnimationName, node)?;
    let mut started = false;
    let name = match value.name() {
        Some(CssAnimationName::Custom(name)) => Some(name.as_str()),
        Some(CssAnimationName::None) => Some("none"),
        _ => None,
    };
    if let Some(duration) = value.duration() {
        writer.source_property(crate::CssKnownProperty::AnimationDuration, |writer| {
            separator(writer, &mut started)?;
            duration.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    } else if value.delay().is_some() {
        writer.source_property(crate::CssKnownProperty::AnimationDuration, |writer| {
            default_slot(writer, &mut started, "0s")
        })?;
    }
    if let Some(easing) = value.timing_function() {
        writer.source_property(crate::CssKnownProperty::AnimationTimingFunction, |writer| {
            separator(writer, &mut started)?;
            easing.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    } else if name.is_some_and(easing_name) {
        writer.source_property(crate::CssKnownProperty::AnimationTimingFunction, |writer| {
            default_slot(writer, &mut started, "ease")
        })?;
    }
    if let Some(delay) = value.delay() {
        writer.source_property(crate::CssKnownProperty::AnimationDelay, |writer| {
            separator(writer, &mut started)?;
            delay.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    if let Some(count) = value.iteration_count() {
        writer.source_property(crate::CssKnownProperty::AnimationIterationCount, |writer| {
            separator(writer, &mut started)?;
            iteration_count(writer, count)?;
            Ok(())
        })?;
    } else if name.is_some_and(|name| name.eq_ignore_ascii_case("infinite")) {
        writer.source_property(crate::CssKnownProperty::AnimationIterationCount, |writer| {
            default_slot(writer, &mut started, "1")
        })?;
    }
    if let Some(value) = value.direction() {
        writer.source_property(crate::CssKnownProperty::AnimationDirection, |writer| {
            separator(writer, &mut started)?;
            direction(writer, value)?;
            Ok(())
        })?;
    } else if name.is_some_and(|name| {
        keyword(
            name,
            &["normal", "reverse", "alternate", "alternate-reverse"],
        )
    }) {
        writer.source_property(crate::CssKnownProperty::AnimationDirection, |writer| {
            default_slot(writer, &mut started, "normal")
        })?;
    }
    if let Some(value) = value.fill_mode() {
        writer.source_property(crate::CssKnownProperty::AnimationFillMode, |writer| {
            separator(writer, &mut started)?;
            fill_mode(writer, value)?;
            Ok(())
        })?;
    } else if name.is_some_and(|name| keyword(name, &["none", "forwards", "backwards", "both"])) {
        writer.source_property(crate::CssKnownProperty::AnimationFillMode, |writer| {
            default_slot(writer, &mut started, "none")
        })?;
    }
    if let Some(value) = value.play_state() {
        writer.source_property(crate::CssKnownProperty::AnimationPlayState, |writer| {
            separator(writer, &mut started)?;
            play_state(writer, value)?;
            Ok(())
        })?;
    } else if name.is_some_and(|name| keyword(name, &["running", "paused"])) {
        writer.source_property(crate::CssKnownProperty::AnimationPlayState, |writer| {
            default_slot(writer, &mut started, "running")
        })?;
    }
    if let Some(name) = value.name() {
        writer.source_property(crate::CssKnownProperty::AnimationName, |writer| {
            separator(writer, &mut started)?;
            animation_name(writer, name)?;
            Ok(())
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssAnimationComponents, CssEasing, CssEasingKeyword, CssTimeLiteral, CssTimeUnit,
        CssTimeValue,
    };
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;

    fn time() -> CssTimeValue {
        CssTimeValue::from_literal(CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap())
    }

    fn shared(
        css: &str,
        input: usize,
        projection: usize,
        append: impl Fn(&mut SpecifiedRuleWriter) -> Result<()>,
    ) {
        let expected = format!("p:{css};{css}");
        let mut writer =
            SpecifiedRuleWriter::new(L::new(input * 2, projection * 2, expected.len()));
        writer.append("p:").unwrap();
        append(&mut writer).unwrap();
        writer.append(";").unwrap();
        append(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (
                L::new(input * 2 - 1, projection * 2, expected.len()),
                K::InputNodeLimit,
            ),
            (
                L::new(input * 2, projection * 2 - 1, expected.len()),
                K::ProjectionNodeLimit,
            ),
            (
                L::new(input * 2, projection * 2, expected.len() - 1),
                K::ByteLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("p:").unwrap();
            append(&mut writer).unwrap();
            writer.append(";").unwrap();
            assert_eq!(append(&mut writer).unwrap_err().kind(), kind);
        }

        // A discarded visit has all the same node costs but charges no bytes.
        let mut writer = SpecifiedRuleWriter::new(L::new(input * 2, projection * 2, css.len()));
        writer.without_output(&append).unwrap();
        assert!(writer.css.is_empty());
        append(&mut writer).unwrap();
        assert_eq!(writer.css, css);
        for (limits, kind) in [
            (L::new(input - 1, projection, 1), K::InputNodeLimit),
            (L::new(input, projection - 1, 1), K::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer
                .without_output(|writer| {
                    assert_eq!(writer.without_output(&append).unwrap_err().kind(), kind);
                    assert!(writer.context.output_suppressed());
                    Ok(())
                })
                .unwrap();
            assert!(!writer.context.output_suppressed());
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
    }

    #[test]
    fn every_timed_list_shares_prefix_sibling_and_suppressed_child_budgets() {
        let value = CssTransitionPropertyList::try_new(vec![CssTransitionProperty::All]).unwrap();
        shared("all", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value = CssDurationList::try_new(vec![
            crate::CssDuration::try_new(CssTimeValue::from_literal(
                CssTimeLiteral::try_new("1", CssTimeUnit::Seconds).unwrap(),
            ))
            .unwrap(),
        ])
        .unwrap();
        shared("1s", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value = CssDelayList::try_new(vec![time()]).unwrap();
        shared("-1s", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value = CssTransitionList::try_new(vec![
            CssTransition::try_new(
                None,
                None,
                Some(time()),
                Some(CssEasing::Keyword(CssEasingKeyword::Ease)),
            )
            .unwrap(),
        ])
        .unwrap();
        shared("0s ease -1s", 4, 5, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = CssAnimationNameList::try_new(vec![CssAnimationName::None]).unwrap();
        shared("none", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value =
            CssAnimationIterationCountList::try_new(vec![CssAnimationIterationCount::Infinite])
                .unwrap();
        shared("infinite", 2, 2, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value =
            CssAnimationDirectionList::try_new(vec![CssAnimationDirection::Normal]).unwrap();
        shared("normal", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value = CssAnimationFillModeList::try_new(vec![CssAnimationFillMode::None]).unwrap();
        shared("none", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value =
            CssAnimationPlayStateList::try_new(vec![CssAnimationPlayState::Running]).unwrap();
        shared("running", 2, 2, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = CssAnimationList::try_new(vec![
            CssAnimation::try_new(CssAnimationComponents {
                name: Some(CssAnimationName::None),
                ..Default::default()
            })
            .unwrap(),
        ])
        .unwrap();
        shared("none none", 3, 4, |writer| {
            value.append_to_rule_writer(writer)
        });
    }

    #[test]
    fn suppressed_iteration_numbers_still_consume_numeric_children() {
        let value =
            CssAnimationIterationCountList::try_new(vec![CssAnimationIterationCount::Number(
                crate::CssSpecifiedNonNegativeNumber::try_from_component(
                    crate::CssComponentValue::try_number("2.5").unwrap(),
                )
                .unwrap(),
            )])
            .unwrap();
        shared("2.5", 3, 3, |writer| value.append_to_rule_writer(writer));
    }

    #[test]
    fn suppressed_symbolic_delay_performs_the_same_projection_work() {
        let report = crate::parse_style_attribute("transition-delay:calc(1s + 2s)");
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let crate::CssKnownPropertyValueRef::TransitionDelay(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("delay")
        };
        let value = value.delays();
        // List + Calc + Sum + two scalar leaves: 5 input nodes.
        // List + two scalar leaves + their combined scalar: 4 projections.
        shared("calc(3s)", 5, 4, |writer| {
            value.append_to_rule_writer(writer)
        });
    }
}
