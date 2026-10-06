//! Ordinary authored container property values and programmatic canonical emission.
use crate::component_values::CssCanonicalBuilder;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueLimits,
    CssComponentValues, CssContainerName, CssSerializedValue, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin,
};

/// The six ordinary container-type states, without computed containment behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssContainerType {
    Normal,
    Size,
    InlineSize,
    ScrollState,
    SizeScrollState,
    InlineSizeScrollState,
}
impl CssContainerType {
    fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Normal => &["normal"],
            Self::Size => &["size"],
            Self::InlineSize => &["inline-size"],
            Self::ScrollState => &["scroll-state"],
            Self::SizeScrollState => &["size", "scroll-state"],
            Self::InlineSizeScrollState => &["inline-size", "scroll-state"],
        }
    }
    fn projection(&self) -> Projection<'_> {
        Projection {
            names: None,
            kind: Some(*self),
        }
    }
}

/// A checked nonempty ordered list; case and repeated names remain significant.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssContainerNameList {
    names: Vec<CssContainerName>,
}
impl CssContainerNameList {
    #[must_use]
    pub fn try_new(names: Vec<CssContainerName>) -> Option<Self> {
        (!names.is_empty()).then_some(Self { names })
    }
    #[must_use]
    pub fn names(&self) -> &[CssContainerName] {
        &self.names
    }
}

/// The ordinary container-name domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssContainerNames {
    None,
    Names(CssContainerNameList),
}
impl CssContainerNames {
    fn projection(&self) -> Projection<'_> {
        Projection {
            names: Some(self),
            kind: None,
        }
    }
}

/// A checked ordinary container shorthand. Omitted type is represented by Normal.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssContainer {
    names: CssContainerNames,
    container_type: CssContainerType,
}
impl CssContainer {
    #[must_use]
    pub const fn new(names: CssContainerNames, container_type: CssContainerType) -> Self {
        Self {
            names,
            container_type,
        }
    }
    #[must_use]
    pub const fn names(&self) -> &CssContainerNames {
        &self.names
    }
    #[must_use]
    pub const fn container_type(&self) -> CssContainerType {
        self.container_type
    }
    fn projection(&self) -> Projection<'_> {
        Projection {
            names: Some(&self.names),
            kind: (self.container_type != CssContainerType::Normal).then_some(self.container_type),
        }
    }
}

#[derive(Clone, Copy)]
enum Piece<'a> {
    Ident(&'a str),
    Space,
    Slash,
}
struct Projection<'a> {
    names: Option<&'a CssContainerNames>,
    kind: Option<CssContainerType>,
}
fn emission_error(kind: CssComponentValueErrorKind) -> CssComponentValueError {
    CssComponentValueError::new(kind, CssValueOrigin::Programmatic)
}
impl Projection<'_> {
    fn visit<E>(&self, mut push: impl FnMut(Piece<'_>) -> Result<(), E>) -> Result<(), E> {
        self.visit_fields(|_, piece| push(piece))
    }

    fn visit_fields<E>(
        &self,
        mut push: impl FnMut(usize, Piece<'_>) -> Result<(), E>,
    ) -> Result<(), E> {
        if let Some(names) = self.names {
            match names {
                CssContainerNames::None => push(0, Piece::Ident("none"))?,
                CssContainerNames::Names(list) => {
                    for (index, name) in list.names().iter().enumerate() {
                        if index != 0 {
                            push(0, Piece::Space)?;
                        }
                        push(0, Piece::Ident(name.as_str()))?;
                    }
                }
            }
            if self.kind.is_some() {
                push(1, Piece::Space)?;
                push(1, Piece::Slash)?;
                push(1, Piece::Space)?;
            }
        }
        if let Some(kind) = self.kind {
            for (index, keyword) in kind.keywords().iter().enumerate() {
                if index != 0 {
                    push(1, Piece::Space)?;
                }
                push(1, Piece::Ident(keyword))?;
            }
        }
        Ok(())
    }

    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        if matches!(self.names, Some(CssContainerNames::Names(_))) {
            writer.source_member(0, |writer| {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)
            })?;
        }
        self.visit_fields(|index, piece| {
            writer.source_member(index, |writer| match piece {
                Piece::Ident(name) => {
                    writer.context.charge_input(1)?;
                    writer.context.charge_projection(1)?;
                    writer.append_identifier(name)
                }
                Piece::Space => writer.append(" "),
                Piece::Slash => writer.append("/"),
            })
        })
    }
    fn components(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssComponentValues, CssComponentValueError> {
        // Raw decoded bytes plus fixed separators are a lower bound on escaped output.
        // Check it and token count before allocating or escaping caller-sized names.
        let mut count = 0usize;
        let mut bytes = 0usize;
        self.visit(|piece| -> Result<(), CssComponentValueError> {
            count = count
                .checked_add(1)
                .ok_or_else(|| emission_error(CssComponentValueErrorKind::CapacityOverflow))?;
            bytes = bytes
                .checked_add(match piece {
                    Piece::Ident(name) => name.len(),
                    Piece::Space | Piece::Slash => 1,
                })
                .ok_or_else(|| emission_error(CssComponentValueErrorKind::CapacityOverflow))?;
            if count > limits.max_components() {
                return Err(emission_error(CssComponentValueErrorKind::ComponentLimit));
            }
            if bytes > limits.max_css_bytes() {
                return Err(emission_error(CssComponentValueErrorKind::ByteLimit));
            }
            Ok(())
        })?;
        let mut items = Vec::new();
        self.visit(|piece| -> Result<(), CssComponentValueError> {
            items.push(match piece {
                Piece::Ident(name) => CssComponentValue::try_ident(name)?,
                Piece::Space => CssComponentValue::try_token(" ")?,
                Piece::Slash => CssComponentValue::try_token("/")?,
            });
            Ok(())
        })?;
        CssComponentValues::try_new_with_limits(items, limits)
    }
    fn serialize(&self, max_bytes: usize) -> Result<CssSerializedValue, CssComponentValueError> {
        let limits = CssComponentValueLimits::try_new(0, usize::MAX, max_bytes)
            .expect("leaf depth is valid");
        let components = self.components(limits)?;
        let mut builder = CssCanonicalBuilder::new(max_bytes);
        builder.push_components(components.items())?;
        builder.finish()
    }
}
macro_rules! emission {
    ($($ty:ty),+ $(,)?) => { $(impl $ty {
        /// Emits canonical components with explicitly programmatic token origins.
        pub fn to_components(&self) -> Result<CssComponentValues, CssComponentValueError> {
            self.to_components_with_limits(CssComponentValueLimits::default())
        }
        /// Emits canonical components under checked token and escaped-byte limits.
        pub fn to_components_with_limits(&self, limits: CssComponentValueLimits) -> Result<CssComponentValues, CssComponentValueError> {
            self.projection().components(limits)
        }
        /// Canonical semantic spelling; parsed declaration wrappers retain authored spelling.
        pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> { self.serialize_with_limit(usize::MAX) }
        /// Bounds canonical output, including identifier escapes and separators.
        pub fn serialize_with_limit(&self, max_bytes: usize) -> Result<CssSerializedValue, CssComponentValueError> { self.projection().serialize(max_bytes) }

        /// Emits canonical specified text without changing component provenance.
        pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }
        /// Emits atomically under cumulative input, projection, and final UTF-8 byte limits.
        /// Each retained name or keyword consumes one input and projection node;
        /// a nonempty name list and the shorthand each add one aggregate node.
        /// The shorthand's omitted Normal type still consumes one keyword node.
        /// Separators and escaping add bytes without adding semantic nodes.
        pub fn serialize_specified_with_limits(&self, limits: CssSpecifiedValueSerializationLimits) -> Result<String, CssSpecifiedValueSerializationError> {
            let mut writer = SpecifiedRuleWriter::new(limits);
            self.append_to_rule_writer(&mut writer)?;
            Ok(writer.css)
        }
    })+ };
}
emission!(CssContainerType, CssContainerNames, CssContainer);

impl CssContainerType {
    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.projection().append_to_rule_writer(writer)
    }
}
impl CssContainerNames {
    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.projection().append_to_rule_writer(writer)
    }
}
impl CssContainer {
    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        self.projection().append_to_rule_writer(writer)?;
        if self.container_type == CssContainerType::Normal {
            writer.source_member(1, |writer| {
                writer.without_output(|writer| self.container_type.append_to_rule_writer(writer))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod specified_tests {
    use super::*;
    use crate::CssIsolation;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;

    fn names(values: &[&str]) -> CssContainerNames {
        CssContainerNames::Names(
            CssContainerNameList::try_new(
                values
                    .iter()
                    .map(|name| CssContainerName::try_from_decoded(*name).unwrap())
                    .collect(),
            )
            .unwrap(),
        )
    }

    fn compose(
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.append("prefix ")?;
        CssIsolation::Auto.append_to_rule_writer(writer)?;
        writer.append(" ")?;
        names(&["Pane", "pane"]).append_to_rule_writer(writer)?;
        writer.append(" ")?;
        CssContainerType::SizeScrollState.append_to_rule_writer(writer)
    }

    #[test]
    fn name_and_type_siblings_share_prior_work_and_final_byte_budget() {
        let expected = "prefix auto Pane pane size scroll-state";
        // One isolation keyword, a list and two names, then two type keywords.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(6, 6, expected.len()));
        compose(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (Limits::new(5, 6, expected.len()), Kind::InputNodeLimit),
            (Limits::new(6, 5, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(6, 6, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
        }
    }

    #[test]
    fn shorthand_omission_keeps_prior_sibling_work_and_charges_normal() {
        let expected = "auto none";
        // One prior keyword, shorthand aggregate, none, and suppressed normal.
        for (limits, failure) in [
            (Limits::new(4, 4, expected.len()), None),
            (
                Limits::new(3, 4, expected.len()),
                Some(Kind::InputNodeLimit),
            ),
            (
                Limits::new(4, 3, expected.len()),
                Some(Kind::ProjectionNodeLimit),
            ),
            (Limits::new(4, 4, expected.len() - 1), Some(Kind::ByteLimit)),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            CssIsolation::Auto
                .append_to_rule_writer(&mut writer)
                .unwrap();
            writer.append(" ").unwrap();
            let result = CssContainer::new(CssContainerNames::None, CssContainerType::Normal)
                .append_to_rule_writer(&mut writer);
            if let Some(kind) = failure {
                assert_eq!(result.unwrap_err().kind(), kind);
            } else {
                result.unwrap();
                assert_eq!(writer.css, expected);
            }
            assert!(!writer.context.output_suppressed());
        }
    }

    #[test]
    fn each_container_provider_suppresses_bytes_without_refunding_nodes() {
        type Emit = fn(&mut SpecifiedRuleWriter) -> Result<(), CssSpecifiedValueSerializationError>;
        let cases: [(Emit, usize); 5] = [
            (
                |w| CssContainerType::InlineSizeScrollState.append_to_rule_writer(w),
                2,
            ),
            (
                |w| names(&["1pane", "a b", "café"]).append_to_rule_writer(w),
                4,
            ),
            (
                |w| {
                    CssContainer::new(CssContainerNames::None, CssContainerType::Normal)
                        .append_to_rule_writer(w)
                },
                3,
            ),
            (
                |w| {
                    CssContainer::new(names(&["Pane", "pane"]), CssContainerType::Normal)
                        .append_to_rule_writer(w)
                },
                5,
            ),
            (
                |w| {
                    CssContainer::new(names(&["pane"]), CssContainerType::SizeScrollState)
                        .append_to_rule_writer(w)
                },
                5,
            ),
        ];
        for (emit, nodes) in cases {
            let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
            writer.without_output(emit).unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            assert_eq!(
                CssIsolation::Auto
                    .append_to_rule_writer(&mut writer)
                    .unwrap_err()
                    .kind(),
                Kind::InputNodeLimit
            );
            for (limits, kind) in [
                (Limits::new(nodes - 1, nodes, 1), Kind::InputNodeLimit),
                (Limits::new(nodes, nodes - 1, 1), Kind::ProjectionNodeLimit),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                writer
                    .without_output(|writer| {
                        assert_eq!(writer.without_output(emit).unwrap_err().kind(), kind);
                        assert!(writer.context.output_suppressed());
                        writer.append("discarded")
                    })
                    .unwrap();
                assert!(!writer.context.output_suppressed());
                assert!(writer.css.is_empty());
                writer.append("x").unwrap();
                assert_eq!(writer.css, "x");
            }
        }
    }

    #[test]
    fn suppressed_wide_names_stop_at_the_remaining_node_budget() {
        let value = CssContainerNames::Names(
            CssContainerNameList::try_new(
                (0..4096)
                    .map(|_| CssContainerName::try_from_decoded("pane").unwrap())
                    .collect(),
            )
            .unwrap(),
        );
        for (limits, kind) in [
            (Limits::new(2, 4097, 1), Kind::InputNodeLimit),
            (Limits::new(4097, 2, 1), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                writer
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert!(!writer.context.output_suppressed());
            assert!(writer.css.is_empty());
            let error = match kind {
                Kind::InputNodeLimit => writer.context.charge_input(1),
                Kind::ProjectionNodeLimit => writer.context.charge_projection(1),
                _ => unreachable!(),
            }
            .unwrap_err();
            assert_eq!(error.kind(), kind);
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
    }

    #[test]
    fn failed_escaped_scratch_preserves_prefix_and_only_final_bytes_are_charged() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 2, 4));
        writer.append("p").unwrap();
        let error = names(&["1"])
            .append_to_rule_writer(&mut writer)
            .unwrap_err();
        // The four-byte CSSOM identifier `\31 ` cannot fit after the prefix.
        assert_eq!(error.kind(), Kind::ByteLimit);
        assert_eq!(writer.css, "p");
        assert!(!writer.context.output_suppressed());
        writer.append("xyz").unwrap();
        assert_eq!(writer.css, "pxyz");
        assert_eq!(
            CssIsolation::Auto
                .append_to_rule_writer(&mut writer)
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
    }
}
